use std::collections::BTreeMap;

use egui::{Color32, FontFamily, FontId, Rgba, Rounding, Style, TextStyle};

use crate::scopes::ApplicationScope;

use super::UIComponent;

pub struct QuickIconsComponent {
    _app_scope: ApplicationScope,
    bg_fill: Color32,
    rounding: Rounding,
    text_style: BTreeMap<TextStyle, FontId>,
}

impl QuickIconsComponent {
    pub fn new(app_scope: ApplicationScope, style: &Style) -> QuickIconsComponent {
        let bg_fill = (Rgba::from(style.visuals.window_fill()) * Rgba::from_gray(0.7)).into();
        QuickIconsComponent {
            _app_scope: app_scope,
            bg_fill,
            rounding: Rounding::default(),
            text_style: [(
                TextStyle::Button,
                FontId::new(30.0, FontFamily::Name("system-ui".into())),
            )]
            .into(),
        }
    }
}

impl UIComponent for QuickIconsComponent {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let mut rect = ui.max_rect();
        let min_x = rect.min.x;
        rect.min.x = 0.0;
        rect.max.x -= min_x;
        ui.painter().rect_filled(rect, self.rounding, self.bg_fill);
        ui.style_mut().text_styles = self.text_style.clone();
        let button_padding = &mut ui.spacing_mut().button_padding;
        button_padding.x = 8.0;
        button_padding.y = 0.0;
        if ui.button("\"").clicked() {
            println!("Clicked");
        }
    }
}
