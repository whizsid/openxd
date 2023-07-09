use cgmath::{Angle, Rad};
use euclid::{Point2D, Transform2D, UnknownUnit};
use lyon_tessellation::{math::Point, path::Path};

use crate::graphics::{instance_buffer::InstanceBuffer, Color};

use super::{
    coordinates::{GraphicScope, ScreenPoint, ScreenScope},
    StrokeStyle,
};

enum EdgeMode {
    End,
    Start,
}

#[derive(Clone)]
pub enum Edge {
    Rounded,
    Normal,
    Square1P2X,
    Square1P4X,
    Square1P5X,
    Square2X,
}

impl Edge {
    fn get_path(&self, center: ScreenPoint, angle: f32, width: u32, mode: EdgeMode) -> Path {
        Path::new()
    }

    fn calculate_actual_line_edge(
        &self,
        center: ScreenPoint,
        angle: f32,
        width: u32,
        mode: EdgeMode,
    ) -> ScreenPoint {
        center
    }
}

#[derive(Clone)]
pub struct Line {
    /// Start coordinate related to the screen
    pub start: ScreenPoint,
    /// End coordinate related to the screen
    pub end: ScreenPoint,
    /// Width of the line in pixel
    pub width: u32,
    /// Style of the stroke
    pub stroke_style: StrokeStyle,
    /// Color of the stroke
    pub color: Color,
    /// Style of the start point of stroke
    pub start_edge: Edge,
    /// Style of the end point of stroke
    pub end_edge: Edge,
    /// Z index
    pub z_index: u16,
}

impl Line {
    /// The line path without the area of edges
    ///
    /// Example:-
    /// ```ignore
    /// [ ]---------------------[ ]
    ///  ^ ^                   ^ ^
    ///  1 2                   3 4
    /// ```
    ///
    /// 1 = Start Point
    /// 2 = End of the first edge area
    /// 3 = End of the last edge area
    /// 4 = End point
    pub fn path_without_edges(&self, screen_min: Point) -> Path {
        let angle = self.angle();
        let start_point = self.start_edge.calculate_actual_line_edge(
            self.start,
            angle,
            self.width,
            EdgeMode::Start,
        );
        let end_point =
            self.end_edge
                .calculate_actual_line_edge(self.end, angle, self.width, EdgeMode::End);

        let start_point: Point2D<f32, UnknownUnit> =
            Point2D::new(start_point.x as f32, start_point.y as f32);
        let end_point: Point2D<f32, UnknownUnit> =
            Point2D::new(end_point.x as f32, end_point.y as f32);

        let mut pb = Path::builder();
        pb.begin(start_point);
        pb.line_to(end_point);
        pb.build()
    }

    /// The area of the starting edge
    pub fn start_edge_path(&self, screen_min: Point) -> Path {
        self.start_edge
            .get_path(self.start, self.angle(), self.width, EdgeMode::Start)
    }

    /// The area of the last edge
    pub fn end_edge_path(&self, screen_min: Point) -> Path {
        self.end_edge
            .get_path(self.end, self.angle(), self.width, EdgeMode::End)
    }

    /// Calculating angle of the line
    pub fn angle(&self) -> f32 {
        let tangent = ((self.end.y - self.start.y) as f32) / ((self.end.x - self.start.x) as f32);
        let angle = Rad::atan(tangent);
        angle.0
    }

    pub fn to_line_raw(&self, transform: Transform2D<f32, ScreenScope, GraphicScope>) -> LineRaw {
        let a = self.start.x as f32;
        let b = self.start.y as f32;
        let c = self.end.x as f32;
        let d = self.end.y as f32;

        let w = self.width as f32;

        let cmn =
            ((c-a).powi(2) + (d-b).powi(2)).sqrt();

        let bl_x = a + w * (d-b)/ cmn;
        let bl_y = b - w * (c-a)/cmn;

        let tl_x = a - w * (d-b)/cmn;
        let tl_y = b + w * (c-a)/cmn;

        let br_x = d + w * (c-a)/cmn;
        let br_y = c - w * (d-b)/cmn;

        let tr_x = d - w * (c-a)/cmn;
        let tr_y = c + w * (d-b)/cmn;

        let trs = Point2D::new(tr_x, tr_y);
        let tls = Point2D::new(tl_x, tl_y);
        let brs = Point2D::new(br_x, br_y);
        let bls = Point2D::new(bl_x, bl_y);

        let tlg = transform.transform_point(tls);
        let trg = transform.transform_point(trs);
        let brg = transform.transform_point(brs);
        let blg = transform.transform_point(bls);

        LineRaw {
            tl: tlg.to_array(),
            tr: trg.to_array(),
            bl: blg.to_array(),
            br: brg.to_array(),
            depth: self.z_index as u32,
            color: self.color.to_raw(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineRaw {
    tl: [f32; 2],
    tr: [f32; 2],
    bl: [f32; 2],
    br: [f32; 2],
    depth: u32,
    color: [f32; 4],
}

impl LineRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<LineRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32; 8], u32)>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct LineRenderPipeline {
    line_pipeline: wgpu::RenderPipeline,
    buffer: InstanceBuffer<LineRaw>,
}

impl LineRenderPipeline {
    pub fn new(render_state: &egui_wgpu::RenderState, lines: Vec<LineRaw>) -> LineRenderPipeline {
        let device = render_state.device.clone();
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let buffer = InstanceBuffer::new(&device, lines);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[LineRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                ..wgpu::MultisampleState::default()
            },
            multiview: None,
        });

        LineRenderPipeline {
            line_pipeline: render_pipeline,
            buffer,
        }
    }

    pub fn add(&mut self, line: LineRaw) -> usize {
        self.buffer.add(line)
    }

    pub fn modify(&mut self, id: usize, line: LineRaw) {
        self.buffer.replace(id, line);
    }

    pub fn remove(&mut self, id: usize) {
        self.buffer.remove(id);
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.buffer.update(device, queue);
    }

    pub fn paint<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        let instance_size = self.buffer.len() as u32;
        if instance_size > 0 {
            rpass.set_pipeline(&self.line_pipeline);
            rpass.set_vertex_buffer(0, self.buffer.as_slice());
            rpass.draw(0..6, 0..instance_size);
        }
    }
}
