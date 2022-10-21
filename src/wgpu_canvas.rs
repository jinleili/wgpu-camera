use crate::camera_input::CameraInput;
use crate::Camera;
use app_surface::{AppSurface, SurfaceFrame};

pub struct WgpuCanvas {
    pub app_surface: AppSurface,
    camera_input: Option<CameraInput>,
}

#[allow(dead_code)]
impl WgpuCanvas {
    pub fn new(app_surface: AppSurface) -> Self {
        let instance = WgpuCanvas {
            app_surface,
            camera_input: None,
        };
        if let Some(callback) = instance.app_surface.callback_to_app {
            callback(0);
        }
        instance
    }

    pub fn set_external_texture(&mut self, external_tex: crate::ExternalTextureObj) {
        println!("set_external_texture");
        self.camera_input = Some(CameraInput::new(&self.app_surface, &external_tex));
    }

    pub fn enter_frame(&mut self) {
        if let Some(camera) = &mut self.camera_input {
            camera.enter_frame(&self.app_surface);
        }
        if let Some(_callback) = self.app_surface.callback_to_app {
            // callback(1);
        }
    }

    pub fn resize(&mut self) {
        self.app_surface.resize_surface();
    }
}
