use super::SessionOutput;
use ndk_sys::{
    camera_status_t, ACameraCaptureSession, ACameraCaptureSession_stateCallbacks, ACameraDevice,
    ACameraDevice_ErrorStateCallback, ACameraDevice_StateCallback, ACameraDevice_StateCallbacks,
    ANativeWindow,
};
use std::os::raw::{c_char, c_uint, c_void};

pub unsafe fn camera(a_native_window: *mut ANativeWindow) {
    use ash::vk;
    let manager = ndk_sys::ACameraManager_create();
    let mut ids: *mut ndk_sys::ACameraIdList = std::ptr::null_mut();
    let res = ndk_sys::ACameraManager_getCameraIdList(manager, &mut ids as _);
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Failed to acquire camera list.");
    }
    if (*ids).numCameras < 1 {
        log::error!("No cameras found.")
    }

    // Open camera device
    let slice = unsafe { std::slice::from_raw_parts((*ids).cameraIds, (*ids).numCameras as _) };
    let selected_camera = slice[1];
    let mut device: *mut ndk_sys::ACameraDevice = std::ptr::null_mut();
    let callbacks = std::ptr::null_mut::<ndk_sys::ACameraDevice_StateCallbacks>();
    let mut res = ndk_sys::ACameraManager_openCamera(
        manager,
        selected_camera,
        get_device_callbacks(),
        &mut device as _,
    );
    if res == camera_status_t::ACAMERA_ERROR_INVALID_PARAMETER {
        log::error!("Invalid parameter to open camera.");
    }

    // Camera session
    let mut container = std::ptr::null_mut::<ndk_sys::ACaptureSessionOutputContainer>();
    res = ndk_sys::ACaptureSessionOutputContainer_create(&mut container as _);
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Capture session output container creation failed.");
    }
    let mut session_output = std::ptr::null_mut::<ndk_sys::ACaptureSessionOutput>();
    res = ndk_sys::ACaptureSessionOutput_create(a_native_window, &mut session_output as _);
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Capture session image reader output creation failed.");
    }
    res = ndk_sys::ACaptureSessionOutputContainer_add(container, session_output);
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Couldn't add image reader output session to container.");
    }
    let mut capture_session = std::ptr::null_mut::<ACameraCaptureSession>();
    res = ndk_sys::ACameraDevice_createCaptureSession(
        device,
        container,
        get_capture_callbacks(),
        &mut capture_session as _,
    );
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Couldn't create capture session.");
    }

    // Request
    let mut request = std::ptr::null_mut::<ndk_sys::ACaptureRequest>();
    res = ndk_sys::ACameraDevice_createCaptureRequest(
        device,
        ndk_sys::ACameraDevice_request_template::TEMPLATE_PREVIEW,
        &mut request as _,
    );
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Couldn't create capture request.");
    }
    let mut target = std::ptr::null_mut::<ndk_sys::ACameraOutputTarget>();
    res = ndk_sys::ACameraOutputTarget_create(a_native_window, &mut target as _);
    if res != camera_status_t::ACAMERA_OK {
        log::error!("Couldn't create camera output target.");
    }
    res = ndk_sys::ACaptureRequest_addTarget(request, target);

    if res != camera_status_t::ACAMERA_OK {
        log::error!("Couldn't add capture request to camera output target.");
    }
    log::info!("xxxxx xxxxx");
}

fn get_device_callbacks() -> *mut ACameraDevice_StateCallbacks {
    let callbacks = ACameraDevice_StateCallbacks {
        // https://stackoverflow.com/questions/33929079/rust-ffi-passing-trait-object-as-context-to-call-callbacks-on
        context: (0 as *mut c_void),
        onDisconnected: Some(on_device_state_changes),
        onError: Some(on_device_error_changes),
    };
    let static_ref: &'static mut ACameraDevice_StateCallbacks = Box::leak(Box::new(callbacks));

    static_ref
}

fn get_capture_callbacks() -> *mut ACameraCaptureSession_stateCallbacks {
    let callbacks = ACameraCaptureSession_stateCallbacks {
        context: (0 as *mut c_void),
        onClosed: Some(on_session_close),
        onReady: Some(on_session_ready),
        onActive: Some(on_session_active),
    };
    let static_ref: &'static mut ACameraCaptureSession_stateCallbacks =
        Box::leak(Box::new(callbacks));

    static_ref
}

#[no_mangle]
extern "C" fn on_device_state_changes(ctx: *mut c_void, dev: *mut ACameraDevice) {
    // TODO...
    // let camera = Box::from_raw(ctx);
    log::info!(" camera on_device_state_changes");
}

#[no_mangle]
extern "C" fn on_device_error_changes(ctx: *mut c_void, dev: *mut ACameraDevice, err: i32) {
    match err as c_uint {
        ndk_sys::ERROR_CAMERA_DEVICE | ndk_sys::ERROR_CAMERA_SERVICE => {
            log::error!("Camera device has encountered a fatal error, .")
        }
        ndk_sys::ERROR_CAMERA_DISABLED => {
            log::error!("Camera device could not be opened due to a device policy.")
        }
        ndk_sys::ERROR_CAMERA_IN_USE => {
            log::error!("Camera device is in use already.")
        }
        ndk_sys::ERROR_MAX_CAMERAS_IN_USE => {
            log::error!("Camera device could not be opened because there are too many other open camera devices.")
        }
        _ => log::error!("Unknown camera error."),
    }
}

#[no_mangle]
unsafe extern "C" fn on_session_ready(context: *mut c_void, session: *mut ACameraCaptureSession) {
    log::info!(" camera on_session_ready");
}
#[no_mangle]
unsafe extern "C" fn on_session_active(context: *mut c_void, session: *mut ACameraCaptureSession) {
    log::info!(" camera on_session_active");
}
#[no_mangle]
unsafe extern "C" fn on_session_close(context: *mut c_void, session: *mut ACameraCaptureSession) {
    log::info!(" camera on_session_close");
}
