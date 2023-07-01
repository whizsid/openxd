use std::sync::Arc;

use egui::PaintCallback;
use egui_wgpu::{CallbackFn, RenderState};

use crate::graphics::workbook::Workbook;

use super::UIComponent;

pub struct WorkbookCanvasComponent {}

impl WorkbookCanvasComponent {
    pub fn new(gb: &RenderState) -> WorkbookCanvasComponent {
        Workbook::init(gb);
        WorkbookCanvasComponent {}
    }

    pub fn change_tab(&mut self, tab_idx: usize) {}
}

impl UIComponent for WorkbookCanvasComponent {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        let cb = CallbackFn::new()
            .prepare(move |device, queue, ce, paint_callback_resources| {
                let resources: &Workbook = paint_callback_resources.get().unwrap();

                resources.prepare(device, queue);
                vec![]
            })
            .paint(move |_info, rpass, paint_callback_resources| {
                let resources: &Workbook = paint_callback_resources.get().unwrap();

                resources.paint(rpass);
            });

        let callback = PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}
