use app_surface::{AppSurface, SurfaceFrame};
use bytemuck::{Pod, Zeroable};
use idroid::node::{ViewNode, ViewNodeBuilder};
use idroid::vertex::PosTex;
use idroid::{geometry::Plane, math::Rect};
use idroid::{BufferObj, MVPUniform};
use std::borrow::Cow;
pub struct CameraInput {
    viewport: (f32, f32, f32, f32),
    mvp_buffer: BufferObj,
    view_node: ViewNode,
}

impl CameraInput {
    pub fn new(app_surface: &AppSurface) -> Self {
        let config = &app_surface.config;
        let device = &app_surface.device;

        let texture = idroid::load_texture::empty(
            device,
            config.format,
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            None,
            Some(wgpu::TextureUsages::TEXTURE_BINDING),
            None,
        );

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
        let screen_mvp = idroid::utils::matrix_helper::fullscreen_mvp((&app_surface.config).into());
        let mvp_buffer = BufferObj::create_uniform_buffer(
            device,
            &MVPUniform {
                mvp_matrix: screen_mvp.into(),
            },
            Some("MVPUniformObj"),
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                // "../wgsl_preprocessed/edge_detection.wgsl"
                "../../wgsl_preprocessed/cross_hatching.wgsl"
            ))),
        });

        let (vertex_data, index_data) =
            Plane::new(1, 1).generate_vertices_by_texcoord(Rect::zero());
        let view_builder = ViewNodeBuilder::<PosTex>::new(vec![(&texture, None)], &shader)
            .with_uniform_buffers(vec![&mvp_buffer])
            .with_samplers(vec![])
            .with_vertices_and_indices((vertex_data, index_data))
            .with_shader_stages(vec![
                wgpu::ShaderStages::VERTEX,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::ShaderStages::FRAGMENT,
            ]);

        let view_node = view_builder.build(device);

        Self {
            viewport: (
                0.0,
                0.0,
                app_surface.config.width as f32,
                app_surface.config.height as f32,
            ),
            mvp_buffer,
            view_node,
        }
    }

    pub fn update_external_texture(
        &mut self,
        app_surface: &AppSurface,
        external_texture: wgpu::Texture,
        img_size: (f32, f32),
    ) {
        let sw = app_surface.config.width as f32;
        let sh = app_surface.config.height as f32;
        self.viewport = (
            (sw - img_size.0) / 2.0,
            (sh - img_size.1) / 2.0,
            img_size.0,
            img_size.1,
        );
        let (vertex_data, _) = Plane::new(1, 1).generate_vertices();
        app_surface.queue.write_buffer(
            &self.view_node.vertex_buf.as_ref().unwrap().buffer,
            0,
            bytemuck::cast_slice(&vertex_data),
        );
        let sampler = app_surface.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texture_view = external_texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.view_node.bg_setting.bind_group =
            app_surface
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.view_node.bg_setting.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.mvp_buffer.buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: None,
                })
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
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_viewport(
                self.viewport.0,
                self.viewport.1,
                self.viewport.2,
                self.viewport.3,
                0.0,
                1.0,
            );
            self.view_node.draw_render_pass(&mut rpass);
        }
        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
