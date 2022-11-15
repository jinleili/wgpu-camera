use wgpu::util::DeviceExt;
use wgpu::{
    BindingType, Buffer, BufferBindingType, PipelineLayout, ShaderModule, ShaderStages, Texture,
    TextureFormat,
};

pub struct ComputeNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::ComputePipeline,
    pub workgroup_size: (u32, u32),
}

impl ComputeNode {
    pub fn new(uniforms: u32, storages: u32, storage_textures: u32, sampler_textures: u32) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut b_index = 0_u32;
        for i in 0..uniforms.len() {
            let buffer_obj = uniforms[i];
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(0),
                },
                count: None,
            });
            b_index += 1;
        }

        for i in 0..storages.len() {
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(0),
                },
                count: None,
            });
            b_index += 1;
        }
        let view_dimension = wgpu::TextureViewDimension::D2;

        for i in 0..storage_textures.len() {
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    view_dimension,
                    access: wgpu::StorageTextureAccess::Write,
                    format: any_tex.format,
                },
                count: None,
            });
            b_index += 1;
        }
        for i in 0..sampler_textures.len() {
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension,
                    multisampled: false,
                },
                count: None,
            });
            b_index += 1;
        }
    }
}
