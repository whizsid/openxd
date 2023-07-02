mod workbook;
mod line;
mod coordinates;

pub use workbook::Workbook;

pub enum StrokeStyle {
    Normal,
    Double,
    Dotted
}
