mod workbook;
pub mod line;
pub mod screen;
pub mod coordinates;

pub use workbook::Workbook;

use self::{line::Line, coordinates::ScreenPoint};

#[derive(Clone, Debug)]
pub enum UserSelectedPoint {
    Fixed(ScreenPoint),
    ResponsiveOffset(Box<UserSelectedPoint>, f32, f32),
    PixelOffset(Box<UserSelectedPoint>, i32, i32),
}

impl UserSelectedPoint {

    pub fn get_prev_fixed_point(&self, resolution: (u32, u32)) -> Option<ScreenPoint> {
        match self {
            Self::Fixed(_) => None,
            Self::ResponsiveOffset(point,_,_) => Some(point.get_fixed_point(resolution)),
            Self::PixelOffset(point,_,_) => Some(point.get_fixed_point(resolution))
        }
    }

    pub fn get_fixed_point(&self, resolution: (u32, u32)) -> ScreenPoint {
        match self {
            Self::Fixed(point) => point.clone(),
            Self::ResponsiveOffset(point, xo, yo) => {
                let point = point.get_fixed_point(resolution);

                let xov = ((resolution.0 as f32) * xo) as i32;
                let yov = ((resolution.1 as f32) * yo) as i32;

                ScreenPoint::new(point.x + xov, point.y + yov)
            },
            Self::PixelOffset(point, xov, yov) => {
                let point = point.get_fixed_point(resolution);
                ScreenPoint::new(point.x + xov, point.y + yov)
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
    Dotted
}

impl StrokeStyle {
    pub fn to_raw(&self) -> u32 {
        match self {
            Self::Normal => 0,
            Self::Double => 1,
            Self::Dashed => 2,
            Self::LongDashed => 3,
            Self::Diamond => 4,
            Self::Dotted => 5
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform2DRaw {
    trasnformation: [f32; 6]
}

/// Item with buffer indexes
pub enum IndexedItem {
    Line {
        /// Data
        line: Line,
        /// GPU buffer index of the file
        line_index: usize,
    }
}

pub enum Item {
    Line(Line) 
}
