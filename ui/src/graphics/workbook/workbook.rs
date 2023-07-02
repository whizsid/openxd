use egui_wgpu::RenderState;
use lyon_tessellation::{StrokeTessellator, FillTessellator};
use wgpu::{ShaderModuleDescriptor, BindGroupLayoutDescriptor, util::{DeviceExt, BufferInitDescriptor}, Device, Queue, RenderPass, BufferUsages, ShaderSource};

pub struct Workbook {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    stroke_tessellator: StrokeTessellator,
    fill_tessellator: FillTessellator,
}

impl Workbook {
    pub fn init(render_state: &RenderState) {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.

        let device = &render_state.device;

        let straight_line_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("straight_line_shader"),
            source: ShaderSource::Wgsl(include_str!("straight_line.wgsl").into()),
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("./workbook.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0.0]),
            usage: BufferUsages::COPY_DST
                | BufferUsages::MAP_WRITE
                | BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(Workbook{
                pipeline,
                bind_group,
                uniform_buffer,
                fill_tessellator: FillTessellator::new(),
                stroke_tessellator: StrokeTessellator::new()
            });
    }

    pub fn prepare(&self, device: &Device, queue: &Queue) {
        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[0.0]));
    }

    pub fn paint<'rpass>(&'rpass self, rpass: &mut RenderPass<'rpass>) {
         // Draw our triangle!
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}
