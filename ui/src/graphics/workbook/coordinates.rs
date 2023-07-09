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

/// A 2d Point in the screen
///
/// This point will take i32 as pixel coordinates. Because users can specify coordinates
/// in outside of the screen. But only in-screen things are rendered. They can use those
/// minus values to animate purposes.
pub type ScreenPoint = Point2D<i32, ScreenScope>;

pub type CanvasPoint = Point2D<f32, CanvasScope>;

pub type GraphicPoint = Point2D<f32, GraphicScope>;

/// Getting the transformation matrix to convert from graphics scope to canvas scope
///
/// - `ppcm` :- Pixels per centimeter. Calculate the pixel count per several centimeters and get the
/// average for a more accurate value
/// - `zoom` :- Zoom value set by the UI
/// - `canvas_width` :- Width of the wgpu canvas in pixel
/// - `canvas_height` :- Height of the wgpu canvas in pixel
/// - `offset_x` :- X axis offset in nano meters after scrolled
/// - `offset_y` :- Y axis offset in nano meters after scrolled
pub fn graphic_to_canvas(
    ppcm: f32,
    zoom: f32,
    canvas_width: u32,
    canvas_height: u32,
    offset_x: f32,
    offset_y: f32,
) -> Transform2D<f32, GraphicScope, CanvasScope> {
    // Equation:-
    //
    // ```ignore
    // x` = [(canvas_width x 10 x 1000 x zoom)/(ppcm x 2)]x + offset_x + (canvas_width x 10 x 1000 x zoom)/(ppcm x 2)
    //      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                       a                                                         b
    //
    // y` = [(canvas_height x 10 x 1000 x zoom)/(ppcm x 2)]y + offset_y + (canvas_height x 10 x 1000 x zoom)/(ppcm x 2)
    //      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                       c                                                          d
    // ```ignore
    //
    // Matrix representation (row-major):-
    //
    // ```ignore
    // | x' |   | a 0 b |   | x |
    // | y' | = | 0 c d | x | y |
    // | w  |   | 0 0 1 |   | 1 |
    // ```
    //
    // Column Major:-
    //
    // ```ignore
    // | a 0 0 |    | m11 m12 0 |
    // | 0 c 0 | => | m21 m22 0 |
    // | b d 1 |    | m31 m32 1 |
    // ```

    Transform2D::new(
        ((canvas_width as f32) * 10000.0 * zoom) / (ppcm * 2.0),
        0.0,
        0.0,
        -((canvas_height as f32) * 10000.0 * zoom) / (ppcm * 2.0),
        offset_x + ((canvas_width as f32) * 10000.0 * zoom) / (ppcm * 2.0),
        offset_y + ((canvas_height as f32) * 10000.0 * zoom) / (ppcm * 2.0),
    )
}

/// Getting the transformation matrix to convert from canvas scope to screen scope
///
/// - `ppcm` :- pixels per a centimeter in canvas scope.
/// - `offset_x` :- x coordinate of the left top corner of the screen
/// - `offset_y` :- y coordinate of the left top corner of the screen
pub fn canvas_to_screen(
    ppcm: f32,
    offset_x: f32,
    offset_y: f32,
) -> Transform2D<f32, CanvasScope, ScreenScope> {
    // Equation
    //
    // ```ignore
    // x' = (ppcm/10000) x x - (offset_x*ppcm)/10000
    //      ^^^^^^^^^^^^       ^^^^^^^^^^^^^^^^^^^^^
    //           a                        b
    // y' = (ppcm/10000) x y - (offset_y*ppcm)/10000
    //      ^^^^^^^^^^^^       ^^^^^^^^^^^^^^^^^^^^^
    //           c                        d
    // ```
    //
    // Matrix representation (row-major):-
    //
    // ```ignore
    // | x' |   | a 0 b |   | x |
    // | y' | = | 0 c d | x | y |
    // | w  |   | 0 0 1 |   | 1 |
    // ```
    //
    // Column Major:-
    //
    // ```ignore
    // | a 0 0 |    | m11 m12 0 |
    // | 0 c 0 | => | m21 m22 0 |
    // | b d 1 |    | m31 m32 1 |
    // ```

    Transform2D::new(
        ppcm / 10000.0,
        0.0,
        0.0,
        ppcm / 10000.0,
        -(offset_x * ppcm) / 10000.0,
        -(offset_y * ppcm) / 10000.0,
    )
}

/// Getting the transformation matrix to convert from screen scope to canvas scope
///
/// - `ppcm` :- pixels per a centimeter in canvas scope.
/// - `offset_x` :- x coordinate of the left top corner of the screen
/// - `offset_y` :- y coordinate of the left top corner of the screen
pub fn screen_to_canvas(
    ppcm: f32,
    offset_x: f32,
    offset_y: f32,
) -> Transform2D<f32, ScreenScope, CanvasScope> {
    canvas_to_screen(ppcm, offset_x, offset_y)
        .inverse()
        .unwrap()
}

/// Getting the transformation matrix to convert from canvas scope to graphics scope
///
/// - `ppcm` :- Pixels per centimeter. Calculate the pixel count per several centimeters and get the
/// average for a more accurate value
/// - `zoom` :- Zoom value set by the UI
/// - `canvas_width` :- Width of the wgpu canvas in pixel
/// - `canvas_height` :- Height of the wgpu canvas in pixel
/// - `offset_x` :- X axis offset in nano meters after scrolled
/// - `offset_y` :- Y axis offset in nano meters after scrolled
pub fn canvas_to_graphic(
    ppcm: f32,
    zoom: f32,
    canvas_width: u32,
    canvas_height: u32,
    offset_x: f32,
    offset_y: f32,
) -> Transform2D<f32, CanvasScope, GraphicScope> {
    graphic_to_canvas(ppcm, zoom, canvas_width, canvas_height, offset_x, offset_y)
        .inverse()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::graphic_to_canvas;
    use super::*;

    #[test]
    pub fn test_graphic_to_canvas_without_offset_and_zoom() {
        let transformed: CanvasPoint = graphic_to_canvas(50.0, 1.0, 1269, 714, 0.0, 0.0)
            .transform_point(GraphicPoint::new(0.1, 0.4));
        assert_eq!(transformed, CanvasPoint::new(25380.0, 57120.0));
    }

    #[test]
    pub fn test_graphic_to_canvas_without_offset() {
        let transformed: CanvasPoint = graphic_to_canvas(50.0, 0.1, 1269, 714, 0.0, 0.0)
            .transform_point(GraphicPoint::new(0.1, 0.4));
        assert_eq!(transformed, CanvasPoint::new(2538.0, 5712.0));
    }

    #[test]
    pub fn test_graphic_to_canvas_without_zoom() {
        let transformed: CanvasPoint = graphic_to_canvas(50.0, 1.0, 1269, 714, 200.0, 300.0)
            .transform_point(GraphicPoint::new(0.1, 0.4));
        assert_eq!(transformed, CanvasPoint::new(25580.0, 57420.0));
    }

    #[test]
    pub fn test_graphic_to_canvas() {
        let transformed: CanvasPoint = graphic_to_canvas(50.0, 0.1, 1269, 714, 200.0, 300.0)
            .transform_point(GraphicPoint::new(0.1, 0.4));
        assert_eq!(transformed, CanvasPoint::new(2738.0, 6012.0));
    }

    #[test]
    pub fn test_canvas_to_graphic_without_offset_and_zoom() {
        let transformed: GraphicPoint = canvas_to_graphic(50.0, 1.0, 1269, 714, 0.0, 0.0)
            .transform_point(CanvasPoint::new(25380.0, 57120.0));
        assert_eq!(transformed, GraphicPoint::new(0.099999994, 0.4));
    }

    #[test]
    pub fn test_canvas_to_graphic_without_offset() {
        let transformed: GraphicPoint = canvas_to_graphic(50.0, 0.1, 1269, 714, 0.0, 0.0)
            .transform_point(CanvasPoint::new(2538.0, 5712.0));
        assert_eq!(transformed, GraphicPoint::new(0.1, 0.4));
    }

    #[test]
    pub fn test_canvas_to_graphic_without_zoom() {
        let transformed: GraphicPoint = canvas_to_graphic(50.0, 1.0, 1269, 714, 200.0, 300.0)
            .transform_point(CanvasPoint::new(25580.0, 57420.0));
        assert_eq!(transformed, GraphicPoint::new(0.099999994, 0.4));
    }

    #[test]
    pub fn test_canvas_to_graphic() {
        let transformed: GraphicPoint = canvas_to_graphic(50.0, 0.1, 1269, 714, 200.0, 300.0)
            .transform_point(CanvasPoint::new(2738.0, 6012.0));
        assert_eq!(transformed, GraphicPoint::new(0.1, 0.4));
    }

    #[test]
    pub fn test_canvas_to_screen_without_offset() {
        let transformed: Point2D<f32, ScreenScope> = canvas_to_screen(163.63636363636363, 0.0, 0.0)
            .transform_point(CanvasPoint::new(20000.0, 30000.0));

        assert_eq!(
            ScreenPoint::new(transformed.x as i32, transformed.y as i32),
            ScreenPoint::new(327, 490)
        );
    }

    #[test]
    pub fn test_canvas_to_screen() {
        let transformed: Point2D<f32, ScreenScope> =
            canvas_to_screen(163.63636363636363, 15000.0, 15000.0)
                .transform_point(CanvasPoint::new(20000.0, 30000.0));

        assert_eq!(
            ScreenPoint::new(transformed.x as i32, transformed.y as i32),
            ScreenPoint::new(81, 245)
        );
    }

    #[test]
    pub fn test_screen_to_canvas_without_offset() {
        let transformed: CanvasPoint = screen_to_canvas(163.63636363636363, 0.0, 0.0)
            .transform_point(Point2D::new(327.0, 490.0));

        assert_eq!(transformed, CanvasPoint::new(19983.332, 29944.441));
    }

    #[test]
    pub fn test_screen_to_canvas() {
        let transformed: CanvasPoint = screen_to_canvas(163.63636363636363, 15000.0, 15000.0)
            .transform_point(Point2D::new(81.0, 245.0));

        assert_eq!(transformed, CanvasPoint::new(19949.998, 29972.219));
    }
}
