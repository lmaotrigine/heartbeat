#![allow(clippy::needless_pass_by_value)]

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use proc_macro_error::proc_macro_error;
use quote::quote;

mod ast;
mod escape;
mod generate;
mod parse;

#[proc_macro]
#[proc_macro_error]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

fn expand(input: TokenStream) -> TokenStream {
    let output_ident = TokenTree::Ident(Ident::new("__html_output", Span::mixed_site()));
    let size_hint = input.to_string().len();
    let markups = parse::parse(input);
    let stmts = generate::generate(markups, output_ident.clone());
    quote!({
        extern crate alloc;
        extern crate html;
        let mut #output_ident = alloc::string::String::with_capacity(#size_hint);
        #stmts
        html::PreEscaped(#output_ident)
    })
}
