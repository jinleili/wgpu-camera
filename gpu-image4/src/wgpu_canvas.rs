use crate::{FilterType, RenderNode};
use app_surface::{AppSurface, SurfaceFrame};
use idroid::node::{ViewNode, ViewNodeBuilder};
use idroid::vertex::PosTex;
use idroid::{geometry::Plane, math::Rect};
use idroid::{BufferObj, MVPUniform};

pub struct WgpuCanvas {
    pub app_surface: AppSurface,
    shader_manager: crate::ShaderManager,
    mvp_buffer: BufferObj,
    params_buffer: BufferObj,
    viewport: (f32, f32, f32, f32),
    view_node: Option<RenderNode>,
    filter_ty: FilterType,
}

#[allow(dead_code)]
impl WgpuCanvas {
    pub fn new(app_surface: AppSurface) -> Self {
        let shader_manager = crate::ShaderManager::new(&app_surface.device);
        let screen_mvp = idroid::utils::matrix_helper::fullscreen_mvp((&app_surface.config).into());
        let mvp_buffer = BufferObj::create_uniform_buffer(
            &app_surface.device,
            &MVPUniform {
                mvp_matrix: screen_mvp.into(),
            },
            Some("MVPUniformObj"),
        );

        let storage_data = [0.0; 32];
        let mut params_buffer = BufferObj::create_storage_buffer(
            &app_surface.device,
            &storage_data,
            Some("Param Buffer"),
        );
        params_buffer.read_only = true;

        let viewport = (
            0.0,
            0.0,
            app_surface.config.width as f32,
            app_surface.config.height as f32,
        );
        let instance = WgpuCanvas {
            app_surface,
            shader_manager,
            mvp_buffer,
            params_buffer,
            viewport,
            view_node: None,
            filter_ty: FilterType::AsciiArt,
        };
        if let Some(callback) = instance.app_surface.callback_to_app {
            callback(0);
        }
        instance
    }

    pub fn update_filter_params(&self, img_size: (f32, f32)) {
        let params_data = match self.filter_ty {
            FilterType::AsciiArt => {
                let ascii_width = 8.0;
                vec![
                    1.0 / img_size.0 * ascii_width,
                    1.0 / img_size.1 * ascii_width,
                    ascii_width,
                    ascii_width / 2.0,
                ]
            }
            _ => vec![0.0],
        };
        self.app_surface.queue.write_buffer(
            &self.params_buffer.buffer,
            0,
            bytemuck::cast_slice(&params_data),
        );
    }

    pub fn set_external_texture(&mut self, external_texture: wgpu::Texture, img_size: (f32, f32)) {
        self.update_filter_params(img_size);

        let texture_view = external_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self
            .app_surface
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
        let view_node = match self.view_node {
            Some(ref mut node) => node,
            None => {
                let texture = idroid::load_texture::empty(
                    &self.app_surface.device,
                    self.app_surface.config.format,
                    wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    None,
                    Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    None,
                );
                let node = RenderNode::new::<PosTex>(
                    &self.app_surface,
                    vec![&self.mvp_buffer],
                    vec![&self.params_buffer],
                    vec![(&texture, None)],
                    vec![&sampler],
                    &self.shader_manager.ascii_art,
                );

                self.view_node = Some(node);
                self.view_node.as_mut().unwrap()
            }
        };

        let sw = self.app_surface.config.width as f32;
        let sh = self.app_surface.config.height as f32;
        view_node.viewport = (
            (sw - img_size.0) / 2.0,
            (sh - img_size.1) / 2.0,
            img_size.0,
            img_size.1,
        );

        let (vertex_data, _) = Plane::new(1, 1).generate_vertices();
        self.app_surface.queue.write_buffer(
            &view_node.vertex_buf.buffer,
            0,
            bytemuck::cast_slice(&vertex_data),
        );
        let sampler = self
            .app_surface
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
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
        view_node.bg_setting.bind_group =
            self.app_surface
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &view_node.bg_setting.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.mvp_buffer.buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.params_buffer.buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                    label: None,
                });
    }

    pub fn enter_frame(&mut self) {
        if let Some(view_node) = &self.view_node {
            let device = &self.app_surface.device;
            let queue = &self.app_surface.queue;
            let (frame, view) = self.app_surface.get_current_frame_view();
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                view_node.begin_render_pass(&view, &mut encoder);
            }
            queue.submit(Some(encoder.finish()));
            frame.present();
        }

        if let Some(_callback) = self.app_surface.callback_to_app {
            // callback(1);
        }
    }

    pub fn resize(&mut self) {
        self.app_surface.resize_surface();
    }
}
