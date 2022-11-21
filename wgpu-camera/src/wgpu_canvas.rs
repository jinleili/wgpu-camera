use crate::{
    fragment_filter_node::FragmentFilterNode, shader_manager::ShaderManager, FilterNode, FilterType,
};
use app_surface::{AppSurface, SurfaceFrame};
use idroid::{BufferObj, MVPUniform};
use nalgebra_glm as glm;

pub struct WgpuCanvas {
    pub app_surface: AppSurface,
    shader_manager: ShaderManager,
    mvp_buffer: BufferObj,
    params_buffer: BufferObj,
    view_node: Option<Box<dyn FilterNode>>,
    current_filter: FilterType,
    img_size: (f32, f32),
    opaque_background_color: bool,
}

#[allow(dead_code)]
impl WgpuCanvas {
    pub fn new(app_surface: AppSurface) -> Self {
        let shader_manager = ShaderManager::new(&app_surface.device);
        let (p_mat, vm_mat) =
            idroid::utils::matrix_helper::perspective_fullscreen_mvp((&app_surface.config).into());
        let mvp_buffer = BufferObj::create_uniform_buffer(
            &app_surface.device,
            &MVPUniform {
                mvp_matrix: (p_mat * vm_mat).into(),
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
            opaque_background_color: false,
        };
        if let Some(callback) = instance.app_surface.callback_to_app {
            callback(0);
        }
        instance
    }

    pub fn set_camera_sensor_orientation(&mut self, angle: f32) {
        let (p_mat, mut vm_mat) = idroid::utils::matrix_helper::perspective_fullscreen_mvp(
            (&self.app_surface.config).into(),
        );
        vm_mat = glm::rotate(
            &vm_mat,
            angle / 180.0 * (-std::f32::consts::PI),
            &glm::vec3(0.0, 0.0, 1.0),
        );

        let uniform: [[f32; 4]; 4] = (p_mat * vm_mat).into();
        self.app_surface.queue.write_buffer(
            &self.mvp_buffer.buffer,
            0,
            bytemuck::cast_slice(&uniform),
        );
    }

    pub fn set_filter(
        &mut self,
        ty: crate::FilterType,
        opaque_background_color: bool,
        input_param: f32,
    ) {
        self.create_render_node_if_needed();
        self.opaque_background_color = opaque_background_color;
        self.view_node.as_mut().map(|node| {
            node.change_filter(&self.app_surface, self.shader_manager.get_shader_ref(ty));
            self.current_filter = ty;
        });
        self.update_filter_params(input_param);
    }

    pub fn change_filter_param(&self, input_param: f32) {
        self.update_filter_params(input_param);
    }

    pub fn set_external_sampler(&mut self, sampler: wgpu::Sampler) {
        self.view_node.as_mut().map(|node| {
            node.update_sampler(sampler);
        });
    }

    pub fn set_external_texture(
        &mut self,
        external_texture: wgpu::Texture,
        tex_key: String,
        img_size: (f32, f32),
    ) {
        self.set_external_tv(external_texture, None, tex_key, img_size);
    }

    pub fn set_external_tv(
        &mut self,
        external_texture: wgpu::Texture,
        external_tv: Option<wgpu::TextureView>,
        tex_key: String,
        img_size: (f32, f32),
    ) {
        self.img_size = img_size;
        let sw = self.app_surface.config.width as f32;
        let sh = self.app_surface.config.height as f32;
        let w_ratio = sw / img_size.0;
        let h_ratio = sh / img_size.1;
        let viewport = if w_ratio > h_ratio {
            let h = img_size.1 * w_ratio;
            (0.0, (sh - h) / 2.0, sw, h)
        } else {
            let w = img_size.0 * h_ratio;
            ((sw - w) / 2.0, 0.0, w, sh)
        };
        self.create_render_node_if_needed();
        self.view_node.as_mut().map(|node| {
            node.update_viewport(viewport);
            node.update_bind_group(
                &self.app_surface,
                &self.mvp_buffer.buffer,
                &self.params_buffer.buffer,
                &external_texture,
                external_tv,
                tex_key,
            )
        });
    }

    pub fn remove_texture(&mut self, tex_key: String) {
        self.view_node
            .as_mut()
            .map(|node| node.remove_bind_group(tex_key));
    }

    pub fn enter_frame(&mut self, tex_key: String) {
        if let Some(view_node) = &mut self.view_node {
            let device = &self.app_surface.device;
            let queue = &self.app_surface.queue;
            let (frame, view) = self.app_surface.get_current_frame_view();
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                view_node.enter_frame(&view, &mut encoder, tex_key);
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
            let filter_type = FilterType::Original;
            let node = FragmentFilterNode::new(
                &self.app_surface,
                &self.shader_manager.get_shader_ref(filter_type),
            );

            self.view_node = Some(Box::new(node));
            self.current_filter = filter_type;
        }
    }

    fn update_filter_params(&self, input_param: f32) {
        let opaque_background_color = if self.opaque_background_color {
            1.0
        } else {
            0.0
        };
        let params_data = match self.current_filter {
            FilterType::AsciiArt => {
                let ascii_width = if input_param == 0.0 {
                    8.0 * self.app_surface.scale_factor
                } else {
                    input_param.min(64.0).max(8.0)
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
                    input_param.min(64.0).max(10.0)
                };
                vec![
                    density,
                    density / 2.0,
                    density * 0.08,
                    0.8,
                    0.6,
                    0.3,
                    0.15,
                    opaque_background_color,
                ]
            }
            FilterType::EdgeDetection => {
                let noise_suppression = if input_param == 0.0 {
                    0.05
                } else {
                    input_param.min(0.33).max(0.05)
                };
                vec![noise_suppression, opaque_background_color]
            }
            _ => vec![0.0],
        };
        self.app_surface.queue.write_buffer(
            &self.params_buffer.buffer,
            0,
            bytemuck::cast_slice(&params_data),
        );
    }
}
