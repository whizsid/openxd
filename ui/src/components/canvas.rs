use std::sync::Arc;

use egui::{PaintCallback, mutex::Mutex};
use egui_glow::CallbackFn;

use crate::canvas::Canvas;

use super::UIComponent;

pub struct CanvasComponent {
    canvas: Arc<Mutex<Canvas>>,
}

impl CanvasComponent {
    pub fn new(gl: Arc<glow::Context>) -> CanvasComponent {
        let canvas = Canvas::new(&gl);
        CanvasComponent {
            canvas: Arc::new(Mutex::new(canvas))
        }
    }

    pub fn change_tab(&mut self, tab_idx: usize) {

    }
}

impl UIComponent for CanvasComponent {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        let canvas = self.canvas.clone();

        let callback = PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                canvas.lock().paint(painter.gl());
            })),
        };

        ui.painter().add(callback);
    }

    fn exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.canvas.lock().destroy(gl);
        }
    }
}
