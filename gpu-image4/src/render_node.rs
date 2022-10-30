use app_surface::AppSurface;
use bytemuck::Pod;
use idroid::node::BindingGroupSetting;
use idroid::vertex::Vertex;
use idroid::{geometry::Plane, math::Rect, AnyTexture};
use idroid::{BufferObj, MVPUniform};
use wgpu::util::DeviceExt;
use wgpu::{ShaderModule, StorageTextureAccess};

pub struct RenderNode {
    pub vertex_buf: BufferObj,
    pub index_buf: wgpu::Buffer,
    pub index_count: usize,
    pub bg_setting: BindingGroupSetting,
    pub pipeline: wgpu::RenderPipeline,
    pub viewport: (f32, f32, f32, f32),
}

#[allow(dead_code)]
impl RenderNode {
    pub fn new<T: Vertex + Pod>(
        app_surface: &AppSurface,
        uniform_buffers: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>,
        tex_views: Vec<(&AnyTexture, Option<StorageTextureAccess>)>,
        samplers: Vec<&wgpu::Sampler>,
        shader_module: &ShaderModule,
    ) -> Self {
        let device = &app_surface.device;
        let corlor_format = app_surface.config.format;
        let stages: Vec<wgpu::ShaderStages> = vec![
            wgpu::ShaderStages::VERTEX,
            wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
            wgpu::ShaderStages::FRAGMENT,
        ];

        let bg_setting = BindingGroupSetting::new(
            device,
            uniform_buffers,
            storage_buffers,
            tex_views,
            samplers,
            stages,
        );

        let (vertex_data, index_data) = Plane::new(1, 1).generate_vertices();
        let vertex_buf = BufferObj::create_buffer(
            device,
            Some(&vertex_data),
            None,
            wgpu::BufferUsages::VERTEX,
            Some("vertex buffer"),
        );
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let default_layout_attributes = T::vertex_attributes(0);
        let vertex_buffer_layouts = vec![wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &default_layout_attributes,
        }];

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bg_setting.bind_group_layout],
            push_constant_ranges: &[],
        });
        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_node pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "fs_main",
                targets: &[Some(corlor_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        RenderNode {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bg_setting,
            pipeline,
            viewport: (
                0.0,
                0.0,
                app_surface.config.width as f32,
                app_surface.config.height as f32,
            ),
        }
    }

    pub fn begin_render_pass(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bg_setting.bind_group, &[]);
        rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buf.buffer.slice(..));
        rpass.set_viewport(
            self.viewport.0,
            self.viewport.1,
            self.viewport.2,
            self.viewport.3,
            0.0,
            1.0,
        );
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
