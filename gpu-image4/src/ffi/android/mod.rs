use crate::wgpu_canvas::WgpuCanvas;
use android_logger::Config;
use app_surface::AppSurface;
use jni::objects::JClass;
use jni::sys::{jint, jlong, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use log::{info, Level};

const TEX_KEY: &'static str = "any string";
#[cfg_attr(target_os = "android", path = "a_hardware_buffer.rs")]
mod a_hardware_buffer;

#[cfg(target_os = "android")]
#[link(name = "camera2ndk")]
extern "C" {}

#[no_mangle]
#[jni_fn("name.jinleili.gpuimage4.RustBridge")]
pub fn createWgpuCanvas(env: *mut JNIEnv, _: JClass, surface: jobject) -> jlong {
    android_logger::init_once(Config::default().with_min_level(Level::Info));
    let canvas = WgpuCanvas::new(AppSurface::new(env as *mut _, surface));
    info!("WgpuCanvas created!");
    unsafe {
        camera();
    }
    Box::into_raw(Box::new(canvas)) as jlong
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

unsafe fn camera() {
    use ash::vk;
    let manager = ndk_sys::ACameraManager_create();
    let mut ids: *mut ndk_sys::ACameraIdList = std::ptr::null_mut();
    // let res = ndk_sys::ACameraManager_getCameraIdList(manager, &mut ids as *mut *mut _);
    let res = ndk_sys::ACameraManager_getCameraIdList(manager, &mut ids as _);
    if res == ndk_sys::camera_status_t::ACAMERA_OK {
        log::error!("Failed to acquire camera list.");
    }
    if (*ids).numCameras < 1 {
        log::error!("No cameras found.")
    }

    //
    let slice = unsafe { std::slice::from_raw_parts((*ids).cameraIds, (*ids).numCameras as _) };
    let selected_camera = slice[0];
    let mut device: *mut ndk_sys::ACameraDevice = std::ptr::null_mut();
    let res = ndk_sys::ACameraManager_openCamera(
        manager,
        selected_camera,
        std::ptr::null_mut(),
        &mut device as _,
    );

    log::info!("camera res: {:?},  ids: {:?}", res, &ids);
}
