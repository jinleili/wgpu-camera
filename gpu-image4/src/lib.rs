mod wgpu_canvas;
use app_surface::AppSurface;
pub use wgpu_canvas::WgpuCanvas;

#[cfg_attr(target_os = "ios", path = "ffi/ios.rs")]
#[cfg_attr(target_os = "android", path = "ffi/android.rs", allow(non_snake_case))]
pub mod ffi;

#[cfg(all(target_os = "android", target_os = "ios"))]
pub use ffi::*;

mod shader_manager;
use shader_manager::ShaderManager;
mod render_node;
use render_node::RenderNode;

#[repr(C)]
pub struct ExternalTextureObj {
    pub width: i32,
    pub height: i32,
    pub raw: *mut std::ffi::c_void,
}

pub trait Camera {
    fn resize(&mut self, _app_surface: &AppSurface) {}
    fn enter_frame(&mut self, app_surface: &AppSurface);
}

enum FilterType {
    AsciiArt,
    CrollHatch,
    EdgeDetection,
}
