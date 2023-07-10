use std::marker::PhantomData;

use egui_wgpu::RenderState;
use euclid::Transform2D;
use lyon_tessellation::{FillTessellator, StrokeTessellator};
use wgpu::{Device, Queue, RenderPass};

use super::{
    coordinates::{canvas_to_ndc, screen_to_canvas, CanvasScope, NdcScope, FbScope, canvas_to_fb},
    line::{Line, LineRenderPipeline},
    screen::{IndexedScreenItems, IndexedScreenWithChild, Screen, ScreenItems, ScreenWithChild},
    IndexedItem, Item,
};

pub struct Workbook {
    /// Stroke tessellator to use in advanced graphics
    stroke_tessellator: StrokeTessellator,
    /// Fill tessellator to use in advanced graphics
    fill_tessellator: FillTessellator,
    /// Zoom level
    zoom: f32,
    /// Scrolled offset
    offset_x: f32,
    /// Scrolled offset
    offset_y: f32,
    /// Pixels per centimeter value of the user's monitor
    ppcm: f32,

    canvas_width: u32,
    canvas_height: u32,

    canvas_min_x: f32,
    canvas_min_y: f32,

    /// Canvas to NDC transformer
    transform_to_ndc: Transform2D<f32, CanvasScope, NdcScope>,
    /// Canvas to framebuffer transformer
    transform_to_fb: Transform2D<f32, CanvasScope, FbScope>,
    /// To render lines
    line_render_pipeline: LineRenderPipeline,
    screens: Vec<IndexedScreenWithChild>,
}

impl Workbook {
    pub fn new(render_state: &RenderState, ppcm: f32) -> Workbook {
        let line_render_pipeline = LineRenderPipeline::new(&render_state, vec![]);


        Workbook {
            line_render_pipeline,
            fill_tessellator: FillTessellator::new(),
            stroke_tessellator: StrokeTessellator::new(),
            transform_to_ndc: Transform2D {
                m11: 0.0,
                m12: 0.0,
                m21: 0.0,
                m22: 0.0,
                m31: 0.0,
                m32: 0.0,
                _unit: PhantomData,
            },
            transform_to_fb: Transform2D {
                m11: 0.0,
                m12: 0.0,
                m21: 0.0,
                m22: 0.0,
                m31: 0.0,
                m32: 0.0,
                _unit: PhantomData,
            },
            canvas_width: 0,
            canvas_height: 0,
            canvas_min_x: 0.0,
            canvas_min_y: 0.0,
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            ppcm,
            screens: vec![],
        }
    }

    pub fn reset(&mut self, screens: Vec<ScreenWithChild>) {
        self.line_render_pipeline.reset();

        let mut indexed_screens = vec![];

        for screen in screens {
            let min = screen.meta.tl();
            let ppcm = screen.meta.get_ppcm();
            let sc_transform = screen_to_canvas(ppcm, min.x, min.y);
            let sndc_transform = sc_transform.then(&self.transform_to_ndc);
            let sfb_transform = sc_transform.then(&self.transform_to_fb);

            match screen.items {
                ScreenItems::Items(items) => {
                    let mut indexed_items = Vec::new();
                    for item in items {
                        match item {
                            Item::Line(line) => {
                                let line_index = self
                                    .line_render_pipeline
                                    .add(line.to_line_raw(screen.meta.clone(), sndc_transform, sfb_transform));
                                indexed_items.push(IndexedItem::Line { line, line_index });
                            }
                        }
                    }
                    let indexed_screen = IndexedScreenWithChild {
                        meta: screen.meta,
                        items: IndexedScreenItems::Items(indexed_items),
                    };
                    indexed_screens.push(indexed_screen);
                }
                ScreenItems::Proxy => {
                    indexed_screens.push(IndexedScreenWithChild {
                        meta: screen.meta,
                        items: IndexedScreenItems::Proxy,
                    });
                }
            }
        }

        self.screens = indexed_screens;
    }

    pub fn add_line(&mut self, screen: Screen, line: Line) {
        let min = screen.tl();
        let sc_transform = screen_to_canvas(screen.get_ppcm(), min.x, min.y);
        let sndc_transform = sc_transform.then(&self.transform_to_ndc);
        let sfb_transform = sc_transform.then(&self.transform_to_fb);
        let line_raw = line.to_line_raw(screen, sndc_transform, sfb_transform);
        self.line_render_pipeline.add(line_raw);
    }

    pub fn zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
        self.update_transform_out();
    }

    pub fn scroll(&mut self, offset_x: f32, offset_y: f32) {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
        self.update_transform_out();
    }

    pub fn resize(&mut self, canvas_width: u32, canvas_height: u32, canvas_min_x: f32, canvas_min_y: f32) {
        dbg!(canvas_width, canvas_height, canvas_min_x, canvas_min_y);
        self.canvas_width = canvas_width;
        self.canvas_height = canvas_height;
        self.canvas_min_x = canvas_min_x;
        self.canvas_min_y = canvas_min_y;
        self.update_transform_out();
    }

    fn update_transform_out(&mut self) {
        self.transform_to_ndc = canvas_to_ndc(
            self.ppcm,
            self.zoom,
            self.canvas_width,
            self.canvas_height,
            self.offset_x,
            self.offset_y,
        );

        self.transform_to_fb = canvas_to_fb(
            self.ppcm,
            self.zoom,
            self.offset_x,
            self.offset_y,
            self.canvas_min_x,
            self.canvas_min_y
        );

        self.reset(
            self.screens
                .iter()
                .map(|s| s.remove_indexes())
                .collect::<Vec<_>>(),
        );
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        self.line_render_pipeline.prepare(device, queue);
    }

    pub fn paint<'rpass>(&'rpass self, rpass: &mut RenderPass<'rpass>) {
        self.line_render_pipeline.paint(rpass);
    }
}
