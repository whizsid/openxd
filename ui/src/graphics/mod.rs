//! Main canvas to draw screens and UIs
pub mod workbook;

pub use lyon_tessellation::path::Path;
pub use lyon_tessellation::math::Point;

pub struct RGBAColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f32,
}

pub enum Color {
    RGBA(RGBAColor),
}

impl Color {
    #[inline]
    pub fn new_red(r:u8, g:u8, b:u8, alpha: f32) -> Color {
        Color::RGBA(RGBAColor::new(r, g, b, alpha))
    }
}

impl RGBAColor {
    pub fn new(r: u8, g: u8, b: u8, alpha: f32) -> RGBAColor {
        RGBAColor {
            red: r,
            green: g,
            blue: b,
            alpha,
        }
    }
}
