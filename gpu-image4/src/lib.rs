mod wgpu_canvas;
pub use wgpu_canvas::WgpuCanvas;

#[cfg_attr(target_os = "ios", path = "ffi/ios.rs")]
#[cfg_attr(target_os = "android", path = "ffi/android.rs", allow(non_snake_case))]
mod ffi;
pub use ffi::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    Original,
    AsciiArt,
    CrossHatch,
    EdgeDetection,
}

mod render_node;
mod shader_manager;

#[repr(C)]
pub struct ExternalTextureObj {
    pub width: i32,
    pub height: i32,
    pub raw: *mut std::ffi::c_void,
}
