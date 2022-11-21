use app_surface::AppSurface;
use bytemuck::Pod;
use idroid::{geometry::Plane, vertex::Vertex, BufferObj};
use wgpu::util::DeviceExt;
use wgpu::{
    BindingType, Buffer, BufferBindingType, PipelineLayout, ShaderModule, ShaderStages, Texture,
    TextureFormat,
};

pub struct DisplayNode {
    sampler: wgpu::Sampler,
    vertex_buf: BufferObj,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    array_stride: wgpu::BufferAddress,
    vertex_attributes: Vec<wgpu::VertexAttribute>,
    pipeline_layout: PipelineLayout,
    pipeline: wgpu::RenderPipeline,
    pub viewport: (f32, f32, f32, f32),
}

#[allow(dead_code)]
impl DisplayNode {
    pub fn new<T: Vertex + Pod>(app_surface: &AppSurface, shader_module: &ShaderModule) -> Self {
        let device = &app_surface.device;
        let corlor_format = app_surface.config.format;

        let sampler = app_surface.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(0),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(0),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

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

        let array_stride = std::mem::size_of::<T>() as wgpu::BufferAddress;
        let vertex_attributes = T::vertex_attributes(0);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        // Create the render pipeline
        let pipeline = Self::create_pipeline(
            device,
            corlor_format,
            &pipeline_layout,
            array_stride,
            &vertex_attributes,
            shader_module,
        );

        DisplayNode {
            sampler,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group_layout,
            bind_group: None,
            array_stride,
            vertex_attributes,
            pipeline_layout,
            pipeline,
            viewport: (
                0.0,
                0.0,
                app_surface.config.width as f32,
                app_surface.config.height as f32,
            ),
        }
    }

    pub fn create_bind_group(
        &self,
        app_surface: &AppSurface,
        mvp_buffer: &Buffer,
        params_buffer: &Buffer,
        texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        app_surface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: mvp_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: None,
            })
    }

    pub fn change_filter(&mut self, app_surface: &AppSurface, shader_module: &ShaderModule) {
        self.pipeline = Self::create_pipeline(
            &app_surface.device,
            app_surface.config.format,
            &self.pipeline_layout,
            self.array_stride,
            &self.vertex_attributes,
            shader_module,
        );
    }

    pub fn update_sampler(&mut self, sampler: wgpu::Sampler) {
        self.sampler = sampler;
    }

    pub fn begin_render_pass(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: Option<&wgpu::BindGroup>,
    ) {
        let bg = match bind_group {
            Some(bg) => bg,
            None => self.bind_group.as_ref().unwrap(),
        };
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, bg, &[]);
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

    fn create_pipeline(
        device: &wgpu::Device,
        corlor_format: TextureFormat,
        pipeline_layout: &PipelineLayout,
        array_stride: wgpu::BufferAddress,
        vertex_attributes: &[wgpu::VertexAttribute],
        shader_module: &ShaderModule,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_node pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: corlor_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
        })
    }
}
