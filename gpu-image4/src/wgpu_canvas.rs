use crate::{render_node::RenderNode, shader_manager::ShaderManager, FilterType};
use app_surface::{AppSurface, SurfaceFrame};
use idroid::vertex::PosTex;
use idroid::{geometry::Plane, math::Rect};
use idroid::{BufferObj, MVPUniform};

pub struct WgpuCanvas {
    pub app_surface: AppSurface,
    shader_manager: ShaderManager,
    mvp_buffer: BufferObj,
    params_buffer: BufferObj,
    view_node: Option<RenderNode>,
    current_filter: FilterType,
    img_size: (f32, f32),
}

#[allow(dead_code)]
impl WgpuCanvas {
    pub fn new(app_surface: AppSurface) -> Self {
        let shader_manager = ShaderManager::new(&app_surface.device);
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

        let instance = WgpuCanvas {
            app_surface,
            shader_manager,
            mvp_buffer,
            params_buffer,
            view_node: None,
            current_filter: FilterType::AsciiArt,
            img_size: (0.0, 0.0),
        };
        if let Some(callback) = instance.app_surface.callback_to_app {
            callback(0);
        }
        instance
    }

    pub fn set_filter(&mut self, ty: crate::FilterType, input_param: f32) {
        self.create_render_node_if_needed();
        self.view_node.as_mut().map(|node| {
            node.change_filter(&self.app_surface, self.shader_manager.get_shader_ref(ty));
            self.current_filter = ty;
        });
        self.update_filter_params(input_param);
    }

    pub fn change_filter_param(&self, input_param: f32) {
        self.update_filter_params(input_param);
    }

    pub fn update_filter_params(&self, input_param: f32) {
        let params_data = match self.current_filter {
            FilterType::AsciiArt => {
                let ascii_width = if input_param == 0.0 {
                    8.0 * self.app_surface.scale_factor
                } else {
                    input_param
                };
                vec![
                    1.0 / self.img_size.0 * ascii_width,
                    1.0 / self.img_size.1 * ascii_width,
                    ascii_width,
                    ascii_width / 2.0,
                ]
            }
            FilterType::CrossHatch => {
                let density = if input_param == 0.0 {
                    10.0 * self.app_surface.scale_factor
                } else {
                    input_param
                };
                vec![density, density / 2.0, 1.0 * self.app_surface.scale_factor]
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
        self.img_size = img_size;
        let sw = self.app_surface.config.width as f32;
        let sh = self.app_surface.config.height as f32;

        self.create_render_node_if_needed();
        self.view_node.as_mut().map(|node| {
            node.viewport = (
                (sw - img_size.0) / 2.0,
                (sh - img_size.1) / 2.0,
                img_size.0,
                img_size.1,
            );
            node.update_binding_group(
                &self.app_surface,
                &self.mvp_buffer.buffer,
                &self.params_buffer.buffer,
                &external_texture,
            )
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

    fn create_render_node_if_needed(&mut self) {
        if self.view_node.is_none() {
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
            let filter_type = FilterType::Original;
            let node = RenderNode::new::<PosTex>(
                &self.app_surface,
                vec![&self.mvp_buffer],
                vec![&self.params_buffer],
                vec![(&texture, None)],
                &self.shader_manager.get_shader_ref(filter_type),
            );

            self.view_node = Some(node);
            self.current_filter = filter_type;
        }
    }
}
