use super::colour::brightness;
use super::font::measure;
use super::xml::{ElementList, Render, XmlContent, XmlElement};
use super::{FONT_FAMILY, FONT_SCALE_DOWN_VALUE, FONT_SCALE_UP_FACTOR};

fn colours_for_background(colour: &str) -> (&str, &str) {
    let brightness_threshold = 0.69;
    if brightness(colour) <= brightness_threshold {
        ("#fff", "#010101")
    } else {
        ("#333", "#ccc")
    }
}

fn round_up_to_odd(val: f32) -> f32 {
    if val.rem_euclid(2.0) == 0.0 {
        val + 1.0
    } else {
        val
    }
}

fn preferred_width(text: &str) -> f32 {
    if text.len() == 0 {
        return 0.0;
    }
    round_up_to_odd(measure(text))
}

fn get_accessible_text(label: Option<&str>, message: &str) -> String {
    match label {
        Some(label) => format!("{}: {}", label, message),
        None => message.to_string(),
    }
}
fn get_logo_element<'a>(
    logo: Option<&str>,
    horiz_padding: impl std::fmt::Display,
    badge_height: f32,
    logo_width: impl std::fmt::Display,
) -> XmlContent {
    let logo_height = 14.0;
    match logo {
        Some(logo) => XmlContent::Element(
            XmlElement::new("image")
                .attr("x", horiz_padding)
                .attr("y", 0.5 * (badge_height - logo_height))
                .attr("width", logo_width)
                .attr("height", logo_height)
                .attr("xlink:href", logo),
        ),
        None => XmlContent::Text(""),
    }
}

fn render_badge(
    content: Vec<XmlContent>,
    left_width: f32,
    right_width: f32,
    height: f32,
    accessible_text: &str,
) -> String {
    let width = left_width + right_width;
    let title = XmlElement::new("title").content(vec![XmlContent::Text(accessible_text)]);
    let body = ElementList::new(content);
    let svg = XmlElement::new("svg")
        .content(vec![XmlContent::Element(title), XmlContent::List(body)])
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("xmlns:xlink", "http://www.w3.org/1999/xlink")
        .attr("width", width)
        .attr("height", height)
        .attr("role", "img")
        .attr("aria-label", accessible_text);
    svg.render()
}

#[derive(Debug)]
pub struct Badge<'a> {
    horiz_padding: f32,
    label_margin: f32,
    message_margin: f32,
    label_width: f32,
    message_width: f32,
    left_width: f32,
    right_width: f32,
    width: f32,
    label_colour: &'a str,
    colour: &'a str,
    label: &'a str,
    message: &'a str,
    accessible_text: String,
    logo_element: XmlContent<'a>,
}

static HEIGHT: f32 = 20.0;
static VERTICAL_MARGIN: f32 = 0.0;
static SHADOW: bool = true;

impl<'a> Badge<'a> {
    pub fn new(
        label: Option<&'a str>,
        message: &'a str,
        logo: Option<&'a str>,
        logo_width: f32,
        logo_padding: f32,
        colour: Option<&'a str>,
        label_colour: Option<&'a str>,
    ) -> Self {
        let colour = colour.unwrap_or("#4c1");
        let horiz_padding = 5.0;
        let has_logo = logo.unwrap_or_default().len() > 0;
        let total_logo_width = logo_width + logo_padding;
        let accessible_text = get_accessible_text(label, message);
        let has_label = label.unwrap_or_default().len() > 0;
        let label_colour = if has_label || has_logo {
            label_colour.unwrap_or("#555")
        } else {
            colour
        };
        let label_margin = total_logo_width + 1.0;
        let label_width = preferred_width(label.unwrap_or(""));
        let left_width = if has_label {
            label_width + 2.0 * horiz_padding + total_logo_width
        } else {
            0.0
        };
        let message_width = preferred_width(message);
        let mut message_margin = left_width - message.len().min(1) as f32;
        if !has_label {
            if has_logo {
                message_margin += total_logo_width + horiz_padding;
            } else {
                message_margin += 1.0;
            }
        }
        let mut right_width = message_width + 2.0 * horiz_padding;
        if has_logo && !has_label {
            right_width += total_logo_width + horiz_padding - 1.0;
        }
        let width = left_width + right_width;
        Self {
            horiz_padding,
            label_margin,
            message_margin,
            label_width,
            message_width,
            left_width,
            right_width,
            width,
            label_colour,
            colour,
            label: label.unwrap_or(""),
            message,
            accessible_text,
            logo_element: get_logo_element(logo, horiz_padding, HEIGHT, logo_width),
        }
    }

    fn get_text_element(&'a self, left_margin: f32, content: &'a str, colour: &str, text_width: f32) -> XmlContent {
        if content.len() == 0 {
            return XmlContent::Text("");
        }
        let (text_colour, shadow_colour) = colours_for_background(colour);
        let x = FONT_SCALE_UP_FACTOR * (left_margin + 0.5 * text_width + self.horiz_padding);
        let text = XmlElement::new("text")
            .content(vec![XmlContent::Text(content)])
            .attr("x", x)
            .attr("y", 140.0 + VERTICAL_MARGIN)
            .attr("transform", FONT_SCALE_DOWN_VALUE)
            .attr("fill", text_colour)
            .attr("textLength", FONT_SCALE_UP_FACTOR * text_width);
        let shadow_text = XmlElement::new("text")
            .content(vec![XmlContent::Text(content)])
            .attr("aria-hidden", true)
            .attr("x", x)
            .attr("y", 150.0 + VERTICAL_MARGIN)
            .attr("transform", FONT_SCALE_DOWN_VALUE)
            .attr("fill", shadow_colour)
            .attr("fill-opacity", ".3")
            .attr("textLength", FONT_SCALE_UP_FACTOR * text_width);
        let shadow = if SHADOW {
            XmlContent::Element(shadow_text)
        } else {
            XmlContent::Text("")
        };
        XmlContent::List(ElementList::new(vec![shadow, XmlContent::Element(text)]))
    }

    fn get_label_element(&self) -> XmlContent {
        self.get_text_element(self.label_margin, self.label, self.label_colour, self.label_width)
    }

    fn get_message_element(&self) -> XmlContent {
        self.get_text_element(self.message_margin, self.message, self.colour, self.message_width)
    }

    fn get_clip_path(&self, rx: f32) -> XmlContent {
        XmlContent::Element(
            XmlElement::new("clipPath")
                .content(vec![XmlContent::Element(
                    XmlElement::new("rect")
                        .attr("width", self.width)
                        .attr("height", HEIGHT)
                        .attr("rx", rx)
                        .attr("fill", "#fff"),
                )])
                .attr("id", "r"),
        )
    }

    fn get_background_group_element(&self, with_gradient: bool) -> XmlElement {
        let left_rect = XmlElement::new("rect")
            .attr("width", self.left_width)
            .attr("height", HEIGHT)
            .attr("fill", self.label_colour);
        let right_rect = XmlElement::new("rect")
            .attr("x", self.left_width)
            .attr("width", self.right_width)
            .attr("height", HEIGHT)
            .attr("fill", self.colour);
        let gradient = XmlElement::new("rect")
            .attr("width", self.width)
            .attr("height", HEIGHT)
            .attr("fill", "url(#s)");
        let content = if with_gradient {
            vec![
                XmlContent::Element(left_rect),
                XmlContent::Element(right_rect),
                XmlContent::Element(gradient),
            ]
        } else {
            vec![XmlContent::Element(left_rect), XmlContent::Element(right_rect)]
        };
        XmlElement::new("g").content(content)
    }

    #[inline]
    fn get_foreground_group_element(&'a self) -> XmlContent<'a> {
        XmlContent::Element(
            XmlElement::new("g")
                .content(vec![
                    self.logo_element.clone(),
                    self.get_label_element(),
                    self.get_message_element(),
                ])
                .attr("fill", "#fff")
                .attr("text-anchor", "middle")
                .attr("font-family", FONT_FAMILY)
                .attr("text-rendering", "geometricPrecision")
                .attr("font-size", 110),
        )
    }
}

impl<'a> Render for Badge<'a> {
    fn render(&self) -> String {
        let gradient = XmlElement::new("linearGradient")
            .content(vec![
                XmlContent::Element(
                    XmlElement::new("stop")
                        .attr("offset", 0)
                        .attr("stop-color", "#bbb")
                        .attr("stop-opacity", ".1"),
                ),
                XmlContent::Element(XmlElement::new("stop").attr("offset", 1).attr("stop-opacity", ".1")),
            ])
            .attr("id", "s")
            .attr("x2", 0)
            .attr("y2", "100%");
        let clip_path = self.get_clip_path(3.0);
        let background_group = self.get_background_group_element(true).attr("clip-path", "url(#r)");
        render_badge(
            vec![
                XmlContent::Element(gradient),
                clip_path,
                XmlContent::Element(background_group),
                self.get_foreground_group_element(),
            ],
            self.left_width,
            self.right_width,
            HEIGHT,
            self.accessible_text.as_str(),
        )
    }
}
