use crate::wgpu_canvas::WgpuCanvas;
use android_logger::Config;
use app_surface::AppSurface;
use jni::objects::JClass;
use jni::sys::{jint, jlong, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use log::{info, Level};

mod session_output;
use session_output::SessionOutput;
mod camera_manager;
use camera_manager::*;

const TEX_KEY: &'static str = "any string";
#[cfg_attr(target_os = "android", path = "a_hardware_buffer.rs")]
mod a_hardware_buffer;

#[cfg(target_os = "android")]
#[link(name = "camera2ndk")]
extern "C" {}

#[cfg(target_os = "android")]
#[link(name = "mediandk")]
extern "C" {}

#[no_mangle]
#[jni_fn("name.jinleili.gpuimage4.RustBridge")]
pub fn createWgpuCanvas(env: *mut JNIEnv, _: JClass, surface: jobject) -> jlong {
    android_logger::init_once(Config::default().with_min_level(Level::Info));
    let canvas = WgpuCanvas::new(AppSurface::new(env as *mut _, surface));
    info!("WgpuCanvas created!");
    unsafe {
        let session_out = SessionOutput::new(
            canvas.app_surface.config.width as i32,
            canvas.app_surface.config.height as i32,
        );
        camera(session_out.native_window);
    }
    Box::into_raw(Box::new(canvas)) as jlong
}

#[no_mangle]
#[jni_fn("name.jinleili.gpuimage4.RustBridge")]
pub fn camera(env: *mut JNIEnv, _: JClass, surface: jobject) {
    android_logger::init_once(Config::default().with_min_level(Level::Info));
    let app_surface = AppSurface::new(env as *mut _, surface);
    unsafe {
        camera(app_surface.native_window.get_raw_window());
    }
}

#[no_mangle]
#[jni_fn("name.jinleili.gpuimage4.RustBridge")]
pub fn enterFrame(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let wgpu_obj = unsafe { &mut *(obj as *mut WgpuCanvas) };
    wgpu_obj.enter_frame(TEX_KEY.to_string());
    // 将 obj 对象的内存管理权重新转交给调用方
    Box::into_raw(Box::new(obj));
}

#[no_mangle]
#[jni_fn("name.jinleili.gpuimage4.RustBridge")]
pub fn dropWgpuCanvas(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let _obj: Box<WgpuCanvas> = unsafe { Box::from_raw(obj as *mut _) };
}
