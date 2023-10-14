use std::sync::Arc;

use egui::{mutex::Mutex, Vec2, Rect};
use egui_wgpu::{CallbackTrait, RenderState};
use palette::rgb::Rgba;

use crate::graphics::{
    workbook::{
        coordinates::{CanvasPoint, ScreenPoint},
        line::{Edge, Line},
        rectangle::Rectangle,
        screen::{Screen, ScreenItems, ScreenWithChild},
        Background, Item, StrokeStyle, UserSelectedPoint, Workbook,
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
        gb.renderer.write().callback_resources.insert(workbook);
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
                    color: Color::RGBA(Rgba::new(1.0, 0.0, 0.0, 0.5)),
                    start_edge: Edge::Normal,
                    end_edge: Edge::Normal,
                    z_index: 1,
                }),
                Item::Rectangle(Rectangle {
                    center: UserSelectedPoint::Fixed(ScreenPoint::new(60, 100)),
                    rotation: 0.0,
                    width: 200,
                    height: 300,
                    z_index: 1,
                    backgrounds: vec![Background::Color(Color::RGBA([0.0, 0.0, 1.0, 0.5].into()))],
                }),
            ]),
        }];

        commands.push(WorkbookCanvasCommand::ChangeTab(screens));
    }
}

impl UIComponent for WorkbookCanvasComponent {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, _response) =
            ui.allocate_exact_size(available_size.clone(), egui::Sense::click_and_drag());

        let resized = available_size != self.prev_size;
        self.prev_size = available_size;

        let command_bus = self.command_bus.clone();

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(rect, CustomWgpuCallback {
            command_bus,
            resized,
            rect
        }));
    }
}

struct CustomWgpuCallback {
    command_bus: Arc<Mutex<Vec<WorkbookCanvasCommand>>>,
    resized: bool,
    rect: Rect,
}

impl CallbackTrait for CustomWgpuCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let workbook: &mut Workbook = resources.get_mut().unwrap();
        if self.resized {
            workbook.resize(
                self.rect.width() as u32,
                self.rect.height() as u32,
                self.rect.min.x as f64,
                self.rect.min.y as f64,
            );
        }

        let mut command_bus_locked = self.command_bus.lock();
        while let Some(command) = command_bus_locked.pop() {
            match command {
                WorkbookCanvasCommand::ChangeTab(screens) => {
                    workbook.reset(screens);
                }
            }
        }

        workbook.prepare(device, queue);
        vec![]
    }

    fn paint<'a>(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let workbook: &Workbook = resources.get().unwrap();
        workbook.paint(render_pass);
    }
}
