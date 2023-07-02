//! Helper classes to convert between coordinate systems
//!
//! ## Graphics Scope
//!
//! Graphics scope is the coordinate system that using by the graphics API. So it will be always in
//! the range of 0 to 1. This scope is not visible to user.
//!
//! ## Canvas Scope
//!
//! Canvas scope is the visible coordinate system to users in the workbook canvas. We are emulating
//! this coordinate system on top of the graphics scope. The measurement unit is nano meter. This
//! coordinate system should sync with the outside world coordinate system.
//!
//! ## Screen Scope
//!
//! The coordinate system inside in a single screen. The measurement unit is pixel. And the
//! coordinate system is varying screen to screen.

use euclid::{Point2D, Transform2D};

/// Scope that users seeing in the screens
pub struct ScreenScope;

/// Scope that users seeing in the canvas
pub struct CanvasScope;

/// Scope that graphics API using for the canvas
pub struct GraphicScope;

pub type ScreenPoint = Point2D<u32, ScreenScope>;

pub type CanvasPoint = Point2D<f32, CanvasScope>;

pub type GraphicPoint = Point2D<f32, GraphicScope>;

/// Getting the transformation matrix to convert from graphics scope to canvas scope
///
/// - `ppcm` :- Pixels per centimeter. Calculate the pixel count per several centimeters and get the
/// average for a more accurate value
/// - `zoom` :- Zoom value set by the UI
/// - `canvas_width` :- Width of the wgpu canvas in pixel
/// - `canvas_height` :- Height of the wgpu canvas in pixel
pub fn graphic_to_canvas(ppcm: f32, zoom: f32, canvas_width: u32, canvas_height: u32 ) -> Transform2D<f32, GraphicScope, CanvasScope> {
    unimplemented!()
}

/// Getting the transformation matrix to convert from canvas scope to screen scope
///
/// - `screen_min` :- Coordinate of left-top corner of the screen. The coordinate should be in
/// canvas scope
/// - `ppcm` :- pixels per a centimeter in canvas scope.
pub fn canvas_to_screen(screen_min: CanvasPoint, ppcm: f32) -> Transform2D<f32, CanvasScope, ScreenScope> {
    unimplemented!()
}

/// Getting the transformation matrix to convert from screen scope to canvas scope
///
/// - `screen_min` :- Coordinate of left-top corner of the screen. The coordinate should be in
/// canvas scope
/// - `ppcm` :- pixels per a centimeter in canvas scope.
pub fn screen_to_canvas(screen_min: CanvasPoint, ppcm: f32) -> Transform2D<f32, ScreenScope, CanvasScope> {
    canvas_to_screen(screen_min, ppcm).inverse().unwrap()
}

/// Getting the transformation matrix to convert from canvas scope to graphics scope
///
/// - `ppcm` :- Pixels per centimeter. Calculate the pixel count per several centimeters and get the
/// average for a more accurate value
/// - `zoom` :- Zoom value set by the UI
/// - `canvas_width` :- Width of the wgpu canvas in pixel
/// - `canvas_height` :- Height of the wgpu canvas in pixel
pub fn canvas_to_graphic(ppcm: f32, zoom: f32, canvas_width: u32, canvas_height: u32) -> Transform2D<f32, CanvasScope, GraphicScope> {
    graphic_to_canvas(ppcm, zoom, canvas_width, canvas_height).inverse().unwrap()
}
