use app_surface::{AppSurface, SurfaceFrame};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;

pub struct CameraInput {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl CameraInput {
    pub fn new(app_surface: &AppSurface) -> Self {
        let config = &app_surface.config;
        let device = &app_surface.device;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
        });
        // unsafe {
        //     texture.as_hal_mut::<hal::api::Metal, _>(|hal| {
        //         hal.map(|hal_tex| {
        //             std::mem::swap(
        //                 hal_tex.raw_handle(),
        //                 &mut std::mem::transmute(external_tex.raw),
        //             )
        //         });
        //     });
        // }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../wgsl_shader/bufferless.wgsl"
            ))),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bufferless fullscreen pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            // the bufferless vertices are in clock-wise order
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let bind_group = create_bind_group(device, &pipeline.get_bind_group_layout(0), &texture);

        Self {
            bind_group,
            pipeline,
        }
    }

    pub fn update_external_texture(
        &mut self,
        app_surface: &AppSurface,
        external_texture: wgpu::Texture,
    ) {
        self.bind_group = create_bind_group(
            &app_surface.device,
            &self.pipeline.get_bind_group_layout(0),
            &external_texture,
        );
    }
}

impl crate::Camera for CameraInput {
    fn resize(&mut self, _app_surface: &AppSurface) {}
    fn enter_frame(&mut self, app_surface: &AppSurface) {
        let device = &app_surface.device;
        let queue = &app_surface.queue;
        let (frame, view) = app_surface.get_current_frame_view();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        // load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
) -> wgpu::BindGroup {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: None,
    })
}
