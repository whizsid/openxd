mod workbook;
pub mod line;
pub mod screen;
pub mod coordinates;

pub use workbook::Workbook;

use self::line::Line;

#[derive(Clone)]
pub enum StrokeStyle {
    Normal,
    Double,
    Dotted
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexRaw {
    position: [f32; 2],
}

impl VertexRaw {
    pub fn new(position: [f32; 2]) -> VertexRaw {
        VertexRaw { position }
    }
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
