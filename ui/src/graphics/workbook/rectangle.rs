use colorgrad::Gradient;
use euclid::{Transform2D, Angle, Point2D};

use crate::graphics::{instance_buffer::InstanceBuffer, Color};

use super::{
    coordinates::{FbScope, NdcScope, ScreenScope},
    screen::Screen, UserSelectedPoint,
};

pub enum Background {
    Color(Color),
    Image,
    Gradient(Gradient),
    Brightness(f32),
    Contrast(f32),
    Invert,
}

pub struct Rounds {
    pub tr: f32,
    pub tl: f32,
    pub br: f32,
    pub bl: f32,
}

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub center: UserSelectedPoint,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
}

impl Rectangle {
    pub fn to_rectangle_raw(
        &self,
        screen: Screen,
        transform_ndc: Transform2D<f64, ScreenScope, NdcScope>,
        transform_fb: Transform2D<f64, ScreenScope, FbScope>,
    ) -> RectangleRaw {

        let center = self.center.get_fixed_exact_point(screen.resolution());
        let cx = center.x;
        let cy = center.y;

        let hw = (self.width as f64) / 2.0; // Half width
        let hh = (self.height as f64) / 2.0; // Half height
        let angle = Angle {
            radians: self.rotation,
        };
        let (s, c) = angle.sin_cos();
        let s = s as f64;
        let c = c as f64;

        let trs: Point2D<f64, ScreenScope> = Point2D::new(cx + hw * c - hh * s, cy + hw * s + hh * c);
        let tls: Point2D<f64, ScreenScope> = Point2D::new(cx - hw * c - hh * s, cy - hw * s + hh * c);
        let bls: Point2D<f64, ScreenScope> = Point2D::new(cx - hw * c + hh * s, cy - hw * s - hh * c);
        let brs: Point2D<f64, ScreenScope> = Point2D::new(cx + hw * c + hh * s, cy + hw * s - hh * c);

        let tlg: Point2D<f32, NdcScope> = transform_ndc.transform_point(tls).cast();
        let trg: Point2D<f32, NdcScope> = transform_ndc.transform_point(trs).cast();
        let brg: Point2D<f32, NdcScope> = transform_ndc.transform_point(brs).cast();
        let blg: Point2D<f32, NdcScope> = transform_ndc.transform_point(bls).cast();

        RectangleRaw {
            tl: tlg.to_array(),
            tr: trg.to_array(),
            bl: blg.to_array(),
            br: brg.to_array(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleRaw {
    tl: [f32; 2],
    tr: [f32; 2],
    bl: [f32; 2],
    br: [f32; 2],
}

impl RectangleRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RectangleRaw>() as wgpu::BufferAddress,
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
            ],
        }
    }
}

pub struct RectangleRenderPipeline {
    rectangle_pipeline: wgpu::RenderPipeline,
    buffer: InstanceBuffer<RectangleRaw>,
}

impl RectangleRenderPipeline {
    pub fn new(
        render_state: &egui_wgpu::RenderState,
        rects: Vec<RectangleRaw>,
    ) -> RectangleRenderPipeline {
        let device = render_state.device.clone();
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rectangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rectangle.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rectangle Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let buffer = InstanceBuffer::new(&device, rects);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rectangle Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[RectangleRaw::desc()],
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

        RectangleRenderPipeline {
            rectangle_pipeline: render_pipeline,
            buffer,
        }
    }

    pub fn add(&mut self, rect: RectangleRaw) -> usize {
        self.buffer.add(rect)
    }

    pub fn modify(&mut self, id: usize, rect: RectangleRaw) {
        self.buffer.replace(id, rect);
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
            rpass.set_pipeline(&self.rectangle_pipeline);
            rpass.set_vertex_buffer(0, self.buffer.as_slice());
            rpass.draw(0..6, 0..instance_size);
        }
    }
}
