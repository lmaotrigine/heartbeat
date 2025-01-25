use std::collections::HashMap;

use proc_macro2::{Delimiter, Ident, Literal, Spacing, Span, TokenStream, TokenTree};
use proc_macro_error::{abort, abort_call_site, emit_error, SpanRange};
use syn::Lit;

use crate::ast;

pub fn parse(input: TokenStream) -> Vec<ast::Markup> {
    Parser::new(input).markups()
}

#[derive(Clone)]
struct Parser {
    current_attr: Option<String>,
    input: <TokenStream as IntoIterator>::IntoIter,
}

impl Iterator for Parser {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next()
    }
}

impl Parser {
    fn new(input: TokenStream) -> Self {
        Self {
            current_attr: None,
            input: input.into_iter(),
        }
    }

    fn with_input(&self, input: TokenStream) -> Self {
        Self {
            current_attr: self.current_attr.clone(),
            input: input.into_iter(),
        }
    }

    fn peek(&mut self) -> Option<TokenTree> {
        self.clone().next()
    }

    fn peek2(&mut self) -> Option<(TokenTree, Option<TokenTree>)> {
        let mut clone = self.clone();
        clone.next().map(|first| (first, clone.next()))
    }

    fn advance(&mut self) {
        self.next();
    }

    fn advance2(&mut self) {
        self.next();
        self.next();
    }

    fn markups(&mut self) -> Vec<ast::Markup> {
        let mut result = vec![];
        loop {
            match self.peek2() {
                None => break,
                Some((TokenTree::Punct(ref punct), _)) if punct.as_char() == ';' => self.advance(),
                Some((TokenTree::Punct(ref punct), Some(TokenTree::Ident(ref ident))))
                    if punct.as_char() == '@' && *ident == "let" =>
                {
                    self.advance2();
                    let keyword = TokenTree::Ident(ident.clone());
                    result.push(self.let_expr(punct.span(), keyword));
                }
                _ => result.push(self.markup()),
            }
        }
        result
    }

    fn markup(&mut self) -> ast::Markup {
        let token = match self.peek() {
            Some(token) => token,
            None => {
                abort_call_site!("unexpected end of input");
            }
        };
        let markup = match token {
            TokenTree::Literal(literal) => {
                self.advance();
                self.literal(literal)
            }
            TokenTree::Punct(ref punct) if punct.as_char() == '@' => {
                self.advance();
                let at_span = punct.span();
                match self.next() {
                    Some(TokenTree::Ident(ident)) => {
                        let keyword = TokenTree::Ident(ident.clone());
                        match ident.to_string().as_str() {
                            "if" => {
                                let mut segments = vec![];
                                self.if_expr(at_span, vec![keyword], &mut segments);
                                ast::Markup::Special { segments }
                            }
                            "while" => self.while_expr(at_span, keyword),
                            "for" => self.for_expr(at_span, keyword),
                            "match" => self.match_expr(at_span, keyword),
                            "let" => {
                                let span = SpanRange {
                                    first: at_span,
                                    last: ident.span(),
                                };
                                abort!(span, "`@let` only works inside a block");
                            }
                            other => {
                                let span = SpanRange {
                                    first: at_span,
                                    last: ident.span(),
                                };
                                abort!(span, "unknown keyword `@{}`", other);
                            }
                        }
                    }
                    _ => {
                        abort!(at_span, "expected keyword after `@`");
                    }
                }
            }
            TokenTree::Ident(ident) => {
                let ident_string = ident.to_string();
                match ident_string.as_str() {
                    "if" | "for" | "while" | "match" | "let" => {
                        abort!(ident, "found keyword `{}`", ident_string; help = "should this be a `@{}`?", ident_string);
                    }
                    "true" | "false" => {
                        if let Some(attr_name) = &self.current_attr {
                            emit_error!(ident, r#"attribute value must be a string"#; help = "to declare an empty attribute, omit the equals sign: `{}`", attr_name; help = "to toggle the attribute, use square brackets: `{}[some_boolean_flag]`", attr_name);
                            return ast::Markup::ParseError {
                                span: SpanRange::single_span(ident.span()),
                            };
                        }
                    }
                    _ => {}
                }
                let name = self.try_namespaced_name().expect("identifier");
                self.element(name)
            }
            TokenTree::Punct(ref punct) if punct.as_char() == '.' || punct.as_char() == '#' => {
                let name = TokenTree::Ident(Ident::new("div", punct.span()));
                self.element(name.into())
            }
            TokenTree::Group(ref group) if group.delimiter() == Delimiter::Parenthesis => {
                self.advance();
                ast::Markup::Splice {
                    expr: group.stream(),
                    outer_span: SpanRange::single_span(group.span()),
                }
            }
            TokenTree::Group(ref group) if group.delimiter() == Delimiter::Brace => {
                self.advance();
                ast::Markup::Block(self.block(group.stream(), SpanRange::single_span(group.span())))
            }
            token => {
                abort!(token, "invalid syntax");
            }
        };
        markup
    }

    fn literal(&mut self, literal: Literal) -> ast::Markup {
        match Lit::new(literal.clone()) {
            Lit::Str(lit_str) => {
                return ast::Markup::Literal {
                    content: lit_str.value(),
                    span: SpanRange::single_span(literal.span()),
                }
            }
            Lit::Int(..) | Lit::Float(..) => {
                emit_error!(literal, r#"literal must be double-quoted: `"{}"`"#, literal);
            }
            Lit::Char(lit_char) => {
                emit_error!(literal, r#"literal must be double-quoted: `"{}"`"#, lit_char.value());
            }
            _ => {
                emit_error!(literal, "expected string");
            }
        }
        ast::Markup::ParseError {
            span: SpanRange::single_span(literal.span()),
        }
    }

    fn if_expr(&mut self, at_span: Span, prefix: Vec<TokenTree>, segments: &mut Vec<ast::Special>) {
        let mut head = prefix;
        let body = loop {
            match self.next() {
                Some(TokenTree::Group(ref block)) if block.delimiter() == Delimiter::Brace => {
                    break self.block(block.stream(), SpanRange::single_span(block.span()));
                }
                Some(token) => head.push(token),
                None => {
                    let mut span = ast::span_tokens(head);
                    span.first = at_span;
                    abort!(span, "expected body for this `@if`");
                }
            }
        };
        segments.push(ast::Special {
            at_span: SpanRange::single_span(at_span),
            head: head.into_iter().collect(),
            body,
        });
        self.else_if_expr(segments);
    }

    fn else_if_expr(&mut self, segments: &mut Vec<ast::Special>) {
        match self.peek2() {
            Some((TokenTree::Punct(ref punct), Some(TokenTree::Ident(ref else_keyword))))
                if punct.as_char() == '@' && *else_keyword == "else" =>
            {
                self.advance2();
                let at_span = punct.span();
                let else_keyword = TokenTree::Ident(else_keyword.clone());
                match self.peek() {
                    Some(TokenTree::Ident(ref if_keyword)) if *if_keyword == "if" => {
                        self.advance();
                        let if_keyword = TokenTree::Ident(if_keyword.clone());
                        self.if_expr(at_span, vec![else_keyword, if_keyword], segments)
                    }
                    _ => match self.next() {
                        Some(TokenTree::Group(ref group)) if group.delimiter() == Delimiter::Brace => {
                            let body = self.block(group.stream(), SpanRange::single_span(group.span()));
                            segments.push(ast::Special {
                                at_span: SpanRange::single_span(at_span),
                                head: vec![else_keyword].into_iter().collect(),
                                body,
                            });
                        }
                        _ => {
                            let span = SpanRange {
                                first: at_span,
                                last: else_keyword.span(),
                            };
                            abort!(span, "expected body for this `@else`");
                        }
                    },
                }
            }
            _ => {}
        }
    }

    fn while_expr(&mut self, at_span: Span, keyword: TokenTree) -> ast::Markup {
        let keyword_span = keyword.span();
        let mut head = vec![keyword];
        let body = loop {
            match self.next() {
                Some(TokenTree::Group(ref block)) if block.delimiter() == Delimiter::Brace => {
                    break self.block(block.stream(), SpanRange::single_span(block.span()));
                }
                Some(token) => head.push(token),
                None => {
                    let span = SpanRange {
                        first: at_span,
                        last: keyword_span,
                    };
                    abort!(span, "expected body for this `@while`");
                }
            }
        };
        ast::Markup::Special {
            segments: vec![ast::Special {
                at_span: SpanRange::single_span(at_span),
                head: head.into_iter().collect(),
                body,
            }],
        }
    }

    fn for_expr(&mut self, at_span: Span, keyword: TokenTree) -> ast::Markup {
        let keyword_span = keyword.span();
        let mut head = vec![keyword];
        loop {
            match self.next() {
                Some(TokenTree::Ident(ref in_keyword)) if *in_keyword == "in" => {
                    head.push(TokenTree::Ident(in_keyword.clone()));
                    break;
                }
                Some(token) => head.push(token),
                None => {
                    let span = SpanRange {
                        first: at_span,
                        last: keyword_span,
                    };
                    abort!(span, "missing `in` in `@for` loop");
                }
            }
        }
        let body = loop {
            match self.next() {
                Some(TokenTree::Group(ref block)) if block.delimiter() == Delimiter::Brace => {
                    break self.block(block.stream(), SpanRange::single_span(block.span()));
                }
                Some(token) => head.push(token),
                None => {
                    let span = SpanRange {
                        first: at_span,
                        last: keyword_span,
                    };
                    abort!(span, "expected body for this `@for`");
                }
            }
        };
        ast::Markup::Special {
            segments: vec![ast::Special {
                at_span: SpanRange::single_span(at_span),
                head: head.into_iter().collect(),
                body,
            }],
        }
    }

    fn match_expr(&mut self, at_span: Span, keyword: TokenTree) -> ast::Markup {
        let keyword_span = keyword.span();
        let mut head = vec![keyword];
        let (arms, arms_span) = loop {
            match self.next() {
                Some(TokenTree::Group(ref body)) if body.delimiter() == Delimiter::Brace => {
                    let span = SpanRange::single_span(body.span());
                    break (self.with_input(body.stream()).match_arms(), span);
                }
                Some(token) => head.push(token),
                None => {
                    let span = SpanRange {
                        first: at_span,
                        last: keyword_span,
                    };
                    abort!(span, "expected body for this `@match`");
                }
            }
        };
        ast::Markup::Match {
            at_span: SpanRange::single_span(at_span),
            head: head.into_iter().collect(),
            arms,
            arms_span,
        }
    }

    fn match_arms(&mut self) -> Vec<ast::MatchArm> {
        let mut arms = vec![];
        while let Some(arm) = self.match_arm() {
            arms.push(arm);
        }
        arms
    }

    fn match_arm(&mut self) -> Option<ast::MatchArm> {
        let mut head = vec![];
        loop {
            match self.peek2() {
                Some((TokenTree::Punct(ref eq), Some(TokenTree::Punct(ref gt))))
                    if eq.as_char() == '=' && gt.as_char() == '>' && eq.spacing() == Spacing::Joint =>
                {
                    self.advance2();
                    head.push(TokenTree::Punct(eq.clone()));
                    head.push(TokenTree::Punct(gt.clone()));
                    break;
                }
                Some((token, _)) => {
                    self.advance();
                    head.push(token);
                }
                None => {
                    if head.is_empty() {
                        return None;
                    } else {
                        let head_span = ast::span_tokens(head);
                        abort!(head_span, "unexpected end of @match pattern");
                    }
                }
            }
        }
        let body = match self.next() {
            Some(TokenTree::Group(ref body)) if body.delimiter() == Delimiter::Brace => {
                let body = self.block(body.stream(), SpanRange::single_span(body.span()));
                if let Some(TokenTree::Punct(ref punct)) = self.peek() {
                    if punct.as_char() == ',' {
                        self.advance();
                    }
                }
                body
            }
            Some(first_token) => {
                let mut span = SpanRange::single_span(first_token.span());
                let mut body = vec![first_token];
                loop {
                    match self.next() {
                        Some(TokenTree::Punct(ref punct)) if punct.as_char() == ',' => break,
                        Some(token) => {
                            span.last = token.span();
                            body.push(token);
                        }
                        None => break,
                    }
                }
                self.block(body.into_iter().collect(), span)
            }
            None => {
                let span = ast::span_tokens(head);
                abort!(span, "unexpected end of @match arm");
            }
        };
        Some(ast::MatchArm {
            head: head.into_iter().collect(),
            body,
        })
    }

    fn let_expr(&mut self, at_span: Span, keyword: TokenTree) -> ast::Markup {
        let mut tokens = vec![keyword];
        loop {
            match self.next() {
                Some(token) => match token {
                    TokenTree::Punct(ref punct) if punct.as_char() == '=' => {
                        tokens.push(token.clone());
                        break;
                    }
                    _ => tokens.push(token),
                },
                None => {
                    let mut span = ast::span_tokens(tokens);
                    span.first = at_span;
                    abort!(span, "unexpected end of `@let` expression");
                }
            }
        }
        loop {
            match self.next() {
                Some(token) => match token {
                    TokenTree::Punct(ref punct) if punct.as_char() == ';' => {
                        tokens.push(token.clone());
                        break;
                    }
                    _ => tokens.push(token),
                },
                None => {
                    let mut span = ast::span_tokens(tokens);
                    span.first = at_span;
                    abort!(span, "unexpected end of `@let` expression"; help = "are you missing a semicolon?");
                }
            }
        }
        ast::Markup::Let {
            at_span: SpanRange::single_span(at_span),
            tokens: tokens.into_iter().collect(),
        }
    }

    fn element(&mut self, name: TokenStream) -> ast::Markup {
        if self.current_attr.is_some() {
            let span = ast::span_tokens(name);
            abort!(span, "unexpected element");
        }
        let attrs = self.attrs();
        let body = match self.peek() {
            Some(TokenTree::Punct(ref punct)) if punct.as_char() == ';' || punct.as_char() == '/' => {
                self.advance();
                if punct.as_char() == '/' {
                    emit_error!(punct, "void elements must use `;`, not `/`"; help = "change this to `;`"; help = "this is to prevent ambiguity, as a slash is not appended in the output");
                }
                ast::ElementBody::Void {
                    semi_span: SpanRange::single_span(punct.span()),
                }
            }
            Some(_) => match self.markup() {
                ast::Markup::Block(block) => ast::ElementBody::Block { block },
                markup => {
                    let markup_span = markup.span();
                    abort!(markup_span, "element body must be wrapped in braces"; help = "this is to visually distinguish it from attributes");
                }
            },
            None => abort_call_site!("expected `;`, found end of macro"),
        };
        ast::Markup::Element { name, attrs, body }
    }

    fn attrs(&mut self) -> Vec<ast::Attr> {
        let mut attrs = vec![];
        loop {
            if let Some(name) = self.try_namespaced_name() {
                match self.peek() {
                    Some(TokenTree::Punct(ref punct)) if punct.as_char() == '=' => {
                        self.advance();
                        assert!(self.current_attr.is_none());
                        self.current_attr = Some(ast::name_to_string(name.clone()));
                        let attr_type = match self.attr_toggler() {
                            Some(toggler) => ast::AttrType::Optional { toggler },
                            None => {
                                let value = self.markup();
                                ast::AttrType::Normal { value }
                            }
                        };
                        self.current_attr = None;
                        attrs.push(ast::Attr::Named {
                            named_attr: ast::NamedAttr { name, attr_type },
                        });
                    }
                    Some(TokenTree::Punct(ref punct)) if punct.as_char() == '?' => {
                        self.advance();
                        let toggler = self.attr_toggler();
                        attrs.push(ast::Attr::Named {
                            named_attr: ast::NamedAttr {
                                name: name.clone(),
                                attr_type: ast::AttrType::Empty { toggler },
                            },
                        });
                    }
                    _ => {
                        let toggler = self.attr_toggler();
                        attrs.push(ast::Attr::Named {
                            named_attr: ast::NamedAttr {
                                name: name.clone(),
                                attr_type: ast::AttrType::Empty { toggler },
                            },
                        });
                    }
                }
            } else {
                match self.peek() {
                    Some(TokenTree::Punct(ref punct)) if punct.as_char() == '.' => {
                        self.advance();
                        let name = self.class_or_id_name();
                        let toggler = self.attr_toggler();
                        attrs.push(ast::Attr::Class {
                            dot_span: SpanRange::single_span(punct.span()),
                            name,
                            toggler,
                        });
                    }
                    Some(TokenTree::Punct(ref punct)) if punct.as_char() == '#' => {
                        self.advance();
                        let name = self.class_or_id_name();
                        attrs.push(ast::Attr::Id {
                            hash_span: SpanRange::single_span(punct.span()),
                            name,
                        });
                    }
                    _ => break,
                }
            }
        }
        let mut attr_map = HashMap::new();
        let mut has_class = false;
        for attr in &attrs {
            let name = match attr {
                ast::Attr::Class { .. } => {
                    if has_class {
                        continue;
                    }
                    has_class = true;
                    "class".to_string()
                }
                ast::Attr::Id { .. } => "id".to_string(),
                ast::Attr::Named { named_attr } => named_attr
                    .name
                    .clone()
                    .into_iter()
                    .map(|token| token.to_string())
                    .collect(),
            };
            let entry: &mut Vec<_> = attr_map.entry(name).or_default();
            entry.push(attr.span());
        }
        for (name, spans) in attr_map {
            if spans.len() > 1 {
                let mut spans = spans.into_iter();
                let first_span = spans.next().expect("spans should be non-empty");
                abort!(first_span, "duplicate attribute `{}`", name);
            }
        }
        attrs
    }

    fn class_or_id_name(&mut self) -> ast::Markup {
        if let Some(symbol) = self.try_name() {
            ast::Markup::Symbol { symbol }
        } else {
            self.markup()
        }
    }

    fn attr_toggler(&mut self) -> Option<ast::Toggler> {
        match self.peek() {
            Some(TokenTree::Group(ref group)) if group.delimiter() == Delimiter::Bracket => {
                self.advance();
                Some(ast::Toggler {
                    cond: group.stream(),
                    cond_span: SpanRange::single_span(group.span()),
                })
            }
            _ => None,
        }
    }

    fn try_name(&mut self) -> Option<TokenStream> {
        let mut result = vec![];
        if let Some(token @ TokenTree::Ident(_)) = self.peek() {
            self.advance();
            result.push(token);
        } else {
            return None;
        }
        let mut expect_ident = false;
        loop {
            expect_ident = match self.peek() {
                Some(TokenTree::Punct(ref punct)) if punct.as_char() == '-' => {
                    self.advance();
                    result.push(TokenTree::Punct(punct.clone()));
                    true
                }
                Some(TokenTree::Ident(ref ident)) if expect_ident => {
                    self.advance();
                    result.push(TokenTree::Ident(ident.clone()));
                    false
                }
                _ => break,
            };
        }
        Some(result.into_iter().collect())
    }

    fn try_namespaced_name(&mut self) -> Option<TokenStream> {
        let mut result = vec![self.try_name()?];
        if let Some(TokenTree::Punct(ref punct)) = self.peek() {
            if punct.as_char() == ':' {
                self.advance();
                result.push(TokenStream::from(TokenTree::Punct(punct.clone())));
                result.push(self.try_name()?);
            }
        }
        Some(result.into_iter().collect())
    }

    fn block(&mut self, body: TokenStream, outer_span: SpanRange) -> ast::Block {
        let markups = self.with_input(body).markups();
        ast::Block { markups, outer_span }
    }
}
