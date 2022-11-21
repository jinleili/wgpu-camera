use crate::wgpu_canvas::WgpuCanvas;
use android_logger::Config;
use app_surface::AppSurface;
use ash::vk;
use hal::api::Vulkan;
use jni::objects::JClass;
use jni::sys::{jlong, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use log::{info, Level};

mod session_output;
use session_output::SessionOutput;
mod camera_manager;
use camera_manager::*;

const TEX_KEY: &'static str = "any string";
#[cfg_attr(target_os = "android", path = "vk_image.rs")]
mod vk_image;

pub(crate) struct AndroidCamera {
    canvas: WgpuCanvas,
    pd_mem_properties: vk::PhysicalDeviceMemoryProperties,
    ahb_fn: vk::AndroidExternalMemoryAndroidHardwareBufferFn,
    session_output: SessionOutput,
    camera: CameraManager,
    ycbcr_conv_info: Option<vk::SamplerYcbcrConversionInfo>,
}

impl AndroidCamera {
    pub(crate) fn set_ycbcr_sampler(
        &mut self,
        ycbcr_sampler: wgpu::Sampler,
        ycbcr_conv_info: vk::SamplerYcbcrConversionInfo,
    ) {
        self.ycbcr_conv_info = Some(ycbcr_conv_info);
        self.canvas.set_external_sampler(ycbcr_sampler);
    }
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu_camera.RustBridge")]
pub fn create_cameracanvas(env: *mut JNIEnv, _: JClass, surface: jobject) -> jlong {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    log_panics::init();

    let mut canvas = WgpuCanvas::new(AppSurface::new(env as *mut _, surface));
    info!("WgpuCanvas created!");

    // let device_desc = wgpu::DeviceDescriptor::default();
    // let instance = util::create_instance();
    // let (device, queue) = util::create_device(&instance, &device_desc);

    // Load vkGetAndroidHardwareBufferXXX functions.
    let (ahb_fn, pd_mem_properties) = unsafe {
        let raw_instance = canvas
            .app_surface
            .instance
            .as_hal::<Vulkan>()
            .unwrap()
            .shared_instance()
            .raw_instance();
        canvas.app_surface.device.as_hal::<Vulkan, _, _>(|device| {
            let handle = device.unwrap().raw_device().handle();
            let load_fn = |name: &std::ffi::CStr| {
                std::mem::transmute(raw_instance.get_device_proc_addr(handle, name.as_ptr()))
            };
            let pd_mem_properties = raw_instance
                .get_physical_device_memory_properties(device.unwrap().raw_physical_device());
            let ahb_fn = vk::AndroidExternalMemoryAndroidHardwareBufferFn::load(load_fn);
            (ahb_fn, pd_mem_properties)
        })
    };

    let (camera, session_output) = unsafe {
        let output = SessionOutput::new(
            canvas.app_surface.config.width as i32,
            canvas.app_surface.config.height as i32,
        );
        let camera = CameraManager::new(&output.native_window);
        (camera, output)
    };
    canvas.set_camera_sensor_orientation(camera.sensor_orientation as f32);

    let android_canvas = AndroidCamera {
        canvas,
        pd_mem_properties,
        ahb_fn,
        session_output,
        camera,
        ycbcr_conv_info: None,
    };
    Box::into_raw(Box::new(android_canvas)) as jlong
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu_camera.RustBridge")]
pub fn capture_one_frame(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let wgpu_obj = unsafe { &mut *(obj as *mut AndroidCamera) };
    unsafe {
        wgpu_obj.camera.capture();
    }
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu_camera.RustBridge")]
pub fn start_capturing(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let wgpu_obj = unsafe { &mut *(obj as *mut AndroidCamera) };
    unsafe {
        wgpu_obj.camera.start_capturing();
    }
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu_camera.RustBridge")]
pub fn stop_capturing(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let wgpu_obj = unsafe { &mut *(obj as *mut AndroidCamera) };
    unsafe {
        wgpu_obj.camera.stop_capturing();
    }
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu_camera.RustBridge")]
pub fn enter_frame(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let mut wgpu_obj = unsafe { &mut *(obj as *mut AndroidCamera) };
    unsafe {
        let ahw_buffer = wgpu_obj.session_output.get_latest_buffer();
        if let Some((texture, view, size)) =
            vk_image::get_external_texture(ahw_buffer, &mut wgpu_obj)
        {
            log::info!("size: {:?}", size);
            wgpu_obj.canvas.set_external_tv(
                texture,
                Some(view),
                // None,
                TEX_KEY.to_string(),
                (size.0 as f32, size.1 as f32),
            );
        }
    }
    wgpu_obj.canvas.enter_frame(TEX_KEY.to_string());
}

#[no_mangle]
#[jni_fn("name.jinleili.wgpu_camera.RustBridge")]
pub fn drop_camera_canvas(_env: *mut JNIEnv, _: JClass, obj: jlong) {
    let _obj: Box<AndroidCamera> = unsafe { Box::from_raw(obj as *mut _) };
}
