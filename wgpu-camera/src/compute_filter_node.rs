use crate::display_node::DisplayNode;
use app_surface::AppSurface;
use bytemuck::Pod;
use idroid::{
    geometry::Plane,
    vertex::{PosTex, Vertex},
    BufferObj,
};
use std::collections::HashMap;
use wgpu::{BindGroupLayout, Buffer, ShaderModule, Texture};

pub(crate) struct ComputeFilterNode {
    bind_group_layout: BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    bind_groups: HashMap<String, wgpu::BindGroup>,
    display_node: DisplayNode,
}

impl ComputeFilterNode {
    pub fn new(
        app_surface: &AppSurface,
        compute_shader: &ShaderModule,
        display_shader: &ShaderModule,
    ) -> Self {
        let device = &app_surface.device;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    count: None,
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(0),
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    count: None,
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    count: None,
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                    },
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: Some(&pipeline_layout),
            module: compute_shader,
            entry_point: "cs_main",
            label: None,
        });
        let display_node = DisplayNode::new::<PosTex>(app_surface, display_shader);
        // display_node.bi
        Self {
            bind_group_layout,
            pipeline,
            bind_groups: HashMap::new(),
            display_node,
        }
    }
}

impl crate::FilterNode for ComputeFilterNode {
    fn change_filter(&mut self, app_surface: &AppSurface, shader_module: &wgpu::ShaderModule) {}

    fn update_viewport(&mut self, viewport: (f32, f32, f32, f32)) {
        self.display_node.viewport = viewport;
    }

    fn update_bind_group(
        &mut self,
        app_surface: &AppSurface,
        _mvp_buffer: &Buffer,
        params_buffer: &Buffer,
        external_texture: &Texture,
        external_tv: Option<wgpu::TextureView>,
        tex_key: String,
    ) {
        let texture_view = match external_tv {
            Some(tv) => tv,
            None => external_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        };
        let bind_group = app_surface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                ],
                label: None,
            });
        self.bind_groups.insert(tex_key, bind_group);
    }

    fn remove_bind_group(&mut self, tex_key: String) {}

    fn enter_frame(
        &mut self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        tex_key: String,
    ) {
        let bind_group = self.bind_groups.get(&tex_key);
        if bind_group.is_none() {
            return;
        }
        self.display_node
            .begin_render_pass(frame_view, encoder, None)
    }
}
