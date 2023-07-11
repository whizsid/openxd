use euclid::{Transform2D, Vector2D, Point2D};

use crate::graphics::{instance_buffer::InstanceBuffer, Color};

use super::{
    coordinates::{NdcScope, ScreenScope, FbScope, ScreenPoint},
    StrokeStyle, UserSelectedPoint, screen::Screen,
};

#[derive(Clone)]
pub enum Edge {
    Rounded,
    Normal,
    Square1P2X,
    Square1P4X,
    Square1P5X,
    Square2X,
}

#[derive(Clone)]
pub struct Line {
    /// Start coordinate related to the screen
    pub start: UserSelectedPoint,
    /// End coordinate related to the screen
    pub end: UserSelectedPoint,
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
    pub fn to_line_raw(
        &self,
        screen: Screen,
        transform_ndc: Transform2D<f32, ScreenScope, NdcScope>,
        transform_fb: Transform2D<f32, ScreenScope, FbScope>
    ) -> LineRaw {
        let start = self.start.get_fixed_point(screen.resolution());
        let end = self.end.get_fixed_point(screen.resolution());
        let a: Vector2D<f32, ScreenScope> = start.cast().to_vector();
        let b: Vector2D<f32, ScreenScope> = end.cast().to_vector();
        let v: Vector2D<f32, ScreenScope> = Vector2D::new(b.x - a.x, b.y - a.y);
        let h = (self.width as f32) / 2.0;

        let nl = v.length();
        let n: Vector2D<f32, ScreenScope> = Vector2D::new(v.y/nl, -v.x/nl);

        let tl = a + n * h;
        let bl = a - n * h;
        let tr = b + n * h;
        let br = b - n * h;

        let trs = tr.to_point();
        let tls = tl.to_point();
        let brs = br.to_point();
        let bls = bl.to_point();

        let tlg = transform_ndc.transform_point(tls);
        let trg = transform_ndc.transform_point(trs);
        let brg = transform_ndc.transform_point(brs);
        let blg = transform_ndc.transform_point(bls);

        let trf = transform_fb.transform_point(trs);
        let brf = transform_fb.transform_point(brs);

        let width = ( (trf.y - brf.y).powi(2) + (trf.x - brf.x).powi(2) ).sqrt();
        let start = transform_fb.transform_point(start.cast());
        let end = transform_fb.transform_point(end.cast());

        let min: Point2D<f32, ScreenScope> = ScreenPoint::new(0,0).cast();
        let res = screen.resolution();
        let max: Point2D<f32, ScreenScope> = ScreenPoint::new(res.0 as i32, res.1 as i32).cast();

        let min_fb = transform_fb.transform_point(min);
        let max_fb = transform_fb.transform_point(max);

        LineRaw {
            tl: tlg.to_array(),
            tr: trg.to_array(),
            bl: blg.to_array(),
            br: brg.to_array(),
            depth: self.z_index as u32,
            color: self.color.to_raw(),
            width,
            stroke: self.stroke_style.to_raw(),
            start: start.to_array(),
            end: end.to_array(),
            bbox: [min_fb.x, min_fb.y, max_fb.x, max_fb.y]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineRaw {
    /// Top-left coordinates in NDC
    tl: [f32; 2],
    /// Top-right coordinates in NDC
    tr: [f32; 2],
    /// Bottom-left coordinates in NDC
    bl: [f32; 2],
    /// Bottom-right coordinates in NDC
    br: [f32; 2],
    /// Depth of the line
    depth: u32,
    /// RGBA Color
    color: [f32; 4],
    /// Width as in the framebuffer coordinate system
    width: f32,
    /// Stroke type
    stroke: u32,
    /// Start coordinate of the line in framebuffer coordinate system
    start: [f32; 2],
    /// End coordinate of the line in framebuffer coordinate system
    end: [f32; 2],
    /// Bounding box of the line in framebuffer coordinates (min_x, min_y, max_x, max_y)
    bbox: [f32;4]
}

impl LineRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<LineRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // tl
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tr
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // bl
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // br
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // depth
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32; 8], u32)>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // width
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32; 12], u32)>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
                // stroke
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32; 13], u32)>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Uint32,
                },
                // start
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32;13], [u32;2])>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // end
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32;15], [u32;2])>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // bbox
                wgpu::VertexAttribute {
                    offset: mem::size_of::<([f32;17], [u32;2])>() as wgpu::BufferAddress,
                    shader_location: 10,
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
        dbg!(&line);
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
