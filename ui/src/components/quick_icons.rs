use std::{fmt::Debug, rc::Rc, collections::BTreeMap};

use egui::{Color32, FontFamily, FontId, Rgba, Rounding, Style, TextStyle};

use crate::{client::ClientTransport, external::External, scopes::ApplicationScope};

use super::UIComponent;

pub struct QuickIconsComponent<
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    bg_fill: Color32,
    rounding: Rounding,
    text_style: BTreeMap<TextStyle, FontId>,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    QuickIconsComponent<TE, EE, T, E>
{
    pub fn new(
        app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
        style: &Style,
    ) -> QuickIconsComponent<TE, EE, T, E> {
        let bg_fill = (Rgba::from(style.visuals.window_fill()) * Rgba::from_gray(0.7)).into();
        QuickIconsComponent {
            app_scope,
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

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> UIComponent
    for QuickIconsComponent<TE, EE, T, E>
{
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
        if ui.button(" ").clicked() {
            println!("Clicked");
        }
    }
}
