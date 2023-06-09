use egui::{Ui, Context};

pub mod menu;
pub mod status_bar;
pub mod dialog_container;
pub mod windows;
pub mod tabs;
pub mod quick_icons;
pub mod workbook_canvas;

pub trait UIComponent {
    /// Drawing the UI
    fn draw(&mut self, ui: &mut Ui);
}

/// Components that need to render on top level
pub trait TopLevelUIComponent {
    fn draw(&mut self, ctx: &Context);
}
