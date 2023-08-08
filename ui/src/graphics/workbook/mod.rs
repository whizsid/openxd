pub mod coordinates;
pub mod line;
pub mod rectangle;
pub mod screen;
mod workbook;

use euclid::Point2D;
pub use workbook::Workbook;

use self::{
    coordinates::{ScreenPoint, ScreenScope},
    line::Line,
    rectangle::Rectangle,
};

#[derive(Clone, Debug)]
pub enum UserSelectedPoint {
    Fixed(ScreenPoint),
    Calculated(Point2D<f64, ScreenScope>),
    ResponsiveOffset(Box<UserSelectedPoint>, f32, f32),
    PixelOffset(Box<UserSelectedPoint>, i32, i32),
}

impl UserSelectedPoint {
    pub fn get_prev_fixed_point(&self, resolution: (u32, u32)) -> Option<ScreenPoint> {
        match self {
            Self::Fixed(_) => None,
            Self::Calculated(_) => None,
            Self::ResponsiveOffset(point, _, _) => Some(point.get_fixed_point(resolution)),
            Self::PixelOffset(point, _, _) => Some(point.get_fixed_point(resolution)),
        }
    }

    pub fn get_fixed_point(&self, resolution: (u32, u32)) -> ScreenPoint {
        match self {
            Self::Calculated(point) => point.cast(),
            Self::Fixed(point) => point.clone(),
            Self::Calculated(point) => point.cast(),
            Self::ResponsiveOffset(point, xo, yo) => {
                let point = point.get_fixed_point(resolution);

                let xov = ((resolution.0 as f64) * (*xo as f64)) as i32;
                let yov = ((resolution.1 as f64) * (*yo as f64)) as i32;

                ScreenPoint::new(point.x + xov, point.y + yov)
            }
            Self::PixelOffset(point, xov, yov) => {
                let point = point.get_fixed_point(resolution);
                ScreenPoint::new(point.x + xov, point.y + yov)
            }
        }
    }

    /// When rendering user drew graphics, we can not ignore even
    /// a single pixel coordinate fraction.
    pub fn get_fixed_exact_point(&self, resolution: (u32, u32)) -> Point2D<f64, ScreenScope> {
        match self {
            Self::Calculated(point) => point.cast(),
            Self::Fixed(point) => point.cast(),
            Self::Calculated(point) => point.cast(),
            Self::ResponsiveOffset(point, xo, yo) => {
                let point = point.get_fixed_exact_point(resolution);

                let xov: f64 = (resolution.0 as f64) * (*xo as f64);
                let yov: f64 = (resolution.1 as f64) * (*yo as f64);

                Point2D::new(point.x + xov, point.y + yov)
            }
            Self::PixelOffset(point, xov, yov) => {
                let point = point.get_fixed_exact_point(resolution);
                Point2D::new(point.x + (*xov as f64), point.y + (*yov as f64))
            }
        }
    }
}

#[derive(Clone)]
pub enum StrokeStyle {
    Normal,
    Double,
    Dashed,
    LongDashed,
    Diamond,
    Dotted,
}

impl StrokeStyle {
    pub fn to_raw(&self) -> u32 {
        match self {
            Self::Normal => 0,
            Self::Double => 1,
            Self::Dashed => 2,
            Self::LongDashed => 3,
            Self::Diamond => 4,
            Self::Dotted => 5,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform2DRaw {
    trasnformation: [f32; 6],
}

/// Item with buffer indexes
pub enum IndexedItem {
    Line {
        /// Data
        line: Line,
        /// GPU buffer index of the file
        line_index: usize,
    },
    Rectangle {
        /// Data
        rectangle: Rectangle,
        /// GPU buffer index of the item
        rectangle_index: usize,
    },
}

pub enum Item {
    Line(Line),
    Rectangle(Rectangle),
}
