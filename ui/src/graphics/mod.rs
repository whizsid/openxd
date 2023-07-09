//! Main canvas to draw screens and UIs
pub mod workbook;

pub mod instance_buffer;

pub use lyon_tessellation::math::Point;
pub use lyon_tessellation::path::Path;
use palette::rgb::Rgba;

#[derive(Clone)]
pub enum Color {
    RGBA(Rgba),
}

impl Color {
    pub fn to_raw(&self) -> [f32; 4] {
        match self {
            Color::RGBA(rgba) => {
                [rgba.red, rgba.green, rgba.blue, rgba.alpha]
            }
        }
    }
}
