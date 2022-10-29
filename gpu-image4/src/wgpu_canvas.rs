use crate::camera_input::CameraInput;
use crate::Camera;
use app_surface::{AppSurface, SurfaceFrame};
use idroid::math::Rect;

pub struct WgpuCanvas {
    pub app_surface: AppSurface,
    camera_input: CameraInput,
}

#[allow(dead_code)]
impl WgpuCanvas {
    pub fn new(app_surface: AppSurface) -> Self {
        let camera_input = CameraInput::new(&app_surface);
        let instance = WgpuCanvas {
            app_surface,
            camera_input,
        };
        if let Some(callback) = instance.app_surface.callback_to_app {
            callback(0);
        }
        instance
    }

    pub fn set_external_texture(&mut self, external_texture: wgpu::Texture, img_size: (f32, f32)) {
        self.camera_input
            .update_external_texture(&self.app_surface, external_texture, img_size);
    }

    pub fn enter_frame(&mut self) {
        self.camera_input.enter_frame(&self.app_surface);

        if let Some(_callback) = self.app_surface.callback_to_app {
            // callback(1);
        }
    }

    pub fn resize(&mut self) {
        self.app_surface.resize_surface();
    }
}
