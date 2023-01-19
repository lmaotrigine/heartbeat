mod vendor;
use regex::Regex;
use vendor::verdana::FONT_DATA;
static FONT_FAMILY: &'static str = "Verdana,Geneva,DejaVu Sans,sans-serif";
static FONT_SCALE_UP_FACTOR: f32 = 10.0;
static FONT_SCALE_DOWN_VALUE: &'static str = "scale(.1)";

mod colour;
mod font;
mod renderer;
mod xml;

use renderer::Badge;
use xml::Render;

pub fn make_badge(
    label: Option<&str>,
    message: &str,
    colour: Option<&str>,
    label_colour: Option<&str>,
    logo: Option<&str>,
    logo_width: Option<f32>,
) -> String {
    let logo_width = logo_width.unwrap_or(if logo.is_some() { 14.0 } else { 0.0 });
    let logo_padding = if logo.is_some() && label.unwrap_or_default().len() > 0 {
        3.0
    } else {
        0.0
    };
    Badge::new(label, message, logo, logo_width, logo_padding, colour, label_colour).render()
}
