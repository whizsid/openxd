use egui::Ui;

pub mod menu;
pub mod status_bar;

pub trait UIComponent {
    /// Drawing the UI
    fn draw(&mut self, ui: &mut Ui);
}
