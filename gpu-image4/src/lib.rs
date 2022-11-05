mod wgpu_canvas;
pub use wgpu_canvas::WgpuCanvas;

#[cfg_attr(target_os = "ios", path = "ffi/ios.rs")]
#[cfg_attr(
    target_os = "android",
    path = "ffi/android/mod.rs",
    allow(non_snake_case)
)]
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

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[allow(dead_code)]
pub(crate) fn cchar_to_string(cchar: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(cchar) };
    let r_str = match c_str.to_str() {
        Err(_) => "",
        Ok(string) => string,
    };
    r_str.to_string()
}

#[allow(dead_code)]
pub(crate) fn string_to_cchar(r_string: String) -> *mut c_char {
    CString::new(r_string).unwrap().into_raw()
}
