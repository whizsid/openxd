use lyon_tessellation::{path::Path, math::Point};

use crate::graphics::Color;

use super::StrokeStyle;

pub enum Edge {
    Rounded,
    Normal,
    Square1P2X,
    Square1P4X,
    Square1P5X,
    Square2X,
}

pub struct Line {
    /// Start coordinate related to the screen
    pub start: u32,
    /// End coordinate related to the screen
    pub end: u32,
    /// Width of the line
    pub width: f32,
    /// Style of the stroke
    pub stroke_style: StrokeStyle,
    /// Color of the stroke
    pub color: Color,
    /// Style of the start point of stroke
    pub start_edge: Edge,
    /// Style of the end point of stroke
    pub end_edge: Edge,
}

impl Line {

    /// The line path without the area of edges
    ///
    /// Example:-
    /// ```
    /// [ ]---------------------[ ]
    ///  ^ ^                   ^ ^
    ///  1 2                   3 4
    /// ```
    /// 
    /// 1 = Start Point
    /// 2 = End of the first edge area
    /// 3 = End of the last edge area
    /// 4 = End point
    pub fn path_without_edges(&self, screen_min: Point) -> Path {
        
    }

    /// The area of the starting edge
    pub fn start_edge_path(&self, screen_min: Point) -> Path {
    }

    /// The area of the last edge
    pub fn end_edge_path(&self, screen_min: Point) -> Path {

    }
}
