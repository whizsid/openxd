use std::sync::Arc;

use egui::{mutex::Mutex, PaintCallback, Vec2};
use egui_wgpu::{CallbackFn, RenderState};
use palette::rgb::Rgba;

use crate::graphics::{
    workbook::{
        coordinates::{CanvasPoint, ScreenPoint},
        line::{Edge, Line},
        screen::{Screen, ScreenItems, ScreenWithChild},
        Item, StrokeStyle, Workbook, UserSelectedPoint,
    },
    Color,
};

use super::UIComponent;

pub enum WorkbookCanvasCommand {
    ChangeTab(Vec<ScreenWithChild>),
}

pub struct WorkbookCanvasComponent {
    command_bus: Arc<Mutex<Vec<WorkbookCanvasCommand>>>,
    prev_size: Vec2,
}

impl WorkbookCanvasComponent {
    pub fn new(gb: &RenderState) -> WorkbookCanvasComponent {
        let workbook = Workbook::new(gb, 50.0);
        gb.renderer
            .write()
            .paint_callback_resources
            .insert(workbook);
        WorkbookCanvasComponent {
            prev_size: Vec2 { x: 0.0, y: 0.0 },
            command_bus: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn change_tab(&mut self, tab_idx: usize) {
        let mut commands = self.command_bus.lock();

        let screens = vec![ScreenWithChild {
            meta: Screen::new(
                CanvasPoint::new(20000.0, 20000.0),
                71500.0,
                146700.0,
                (1170, 2532),
                String::from("Test"),
            ),
            items: ScreenItems::Items(vec![
                Item::Line(Line {
                    start: UserSelectedPoint::Fixed(ScreenPoint::new(-130, -190)),
                    end: UserSelectedPoint::Fixed(ScreenPoint::new(1200, 1000)),
                    width: 100,
                    stroke_style: StrokeStyle::Dotted,
                    color: Color::RGBA(Rgba::new(0.0, 1.0, 0.0, 1.0)),
                    start_edge: Edge::Normal,
                    end_edge: Edge::Normal,
                    z_index: 1,
                }),
            ]),
        }];

        commands.push(WorkbookCanvasCommand::ChangeTab(screens));
    }
}

impl UIComponent for WorkbookCanvasComponent {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, response) =
            ui.allocate_exact_size(available_size.clone(), egui::Sense::click_and_drag());

        let resized = available_size != self.prev_size;
        self.prev_size = available_size;

        let rect_cloned = rect.clone();
        let command_bus = self.command_bus.clone();
        let cb = CallbackFn::new()
            .prepare(move |device, queue, ce, paint_callback_resources| {
                let workbook: &mut Workbook = paint_callback_resources.get_mut().unwrap();
                if resized {
                    workbook.resize(available_size.x as u32, available_size.y as u32, rect_cloned.min.x, rect_cloned.min.y);
                }

                let mut command_bus_locked = command_bus.lock();
                while let Some(command) = command_bus_locked.pop() {
                    match command {
                        WorkbookCanvasCommand::ChangeTab(screens) => {
                            workbook.reset(screens);
                        }
                    }
                }

                workbook.prepare(device, queue);
                vec![]
            })
            .paint(move |_info, rpass, paint_callback_resources| {
                let workbook: &Workbook = paint_callback_resources.get().unwrap();
                workbook.paint(rpass);
            });

        let callback = PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}
