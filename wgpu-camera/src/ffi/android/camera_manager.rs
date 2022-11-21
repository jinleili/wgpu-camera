use ndk::native_window::NativeWindow;
use ndk_sys::{
    camera_status_t, ACameraCaptureFailure, ACameraCaptureSession,
    ACameraCaptureSession_captureCallbacks, ACameraCaptureSession_stateCallbacks, ACameraDevice,
    ACameraDevice_StateCallbacks, ACameraMetadata, ACameraWindowType, ACaptureRequest,
};
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_uint, c_void};

#[link(name = "camera2ndk")]
extern "C" {}

pub(crate) struct CameraManager {
    capture_session: *mut ACameraCaptureSession,
    request: *mut ACaptureRequest,
    pub sensor_orientation: i32,
}

impl CameraManager {
    pub unsafe fn new(native_window: &NativeWindow) -> Self {
        let manager = ndk_sys::ACameraManager_create();
        let mut ids = MaybeUninit::uninit();

        let res = ndk_sys::ACameraManager_getCameraIdList(manager, ids.as_mut_ptr());
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Failed to acquire camera list.");
        }
        let ids = ids.assume_init();
        if (*ids).numCameras < 1 {
            log::error!("No cameras found.")
        }
        // Select back-facing camera
        let back_facing =
            ndk_sys::acamera_metadata_enum_acamera_lens_facing::ACAMERA_LENS_FACING_BACK.0 as u8;
        let ids_slice = std::slice::from_raw_parts((*ids).cameraIds, (*ids).numCameras as _);
        let mut selected_camera = ids_slice[0];
        let mut sensor_orientation = 0_i32;
        for i in 0..(*ids).numCameras {
            let id = ids_slice[i as usize];
            let mut metadata = MaybeUninit::uninit();
            let res = ndk_sys::ACameraManager_getCameraCharacteristics(
                manager,
                id,
                metadata.as_mut_ptr(),
            );
            if res != camera_status_t::ACAMERA_OK {
                continue;
            }
            let metadata = metadata.assume_init();
            let mut entry = MaybeUninit::uninit();
            let res = ndk_sys::ACameraMetadata_getConstEntry(
                metadata,
                ndk_sys::acamera_metadata_tag::ACAMERA_LENS_FACING.0 as _,
                entry.as_mut_ptr(),
            );
            if res == camera_status_t::ACAMERA_OK {
                let entry: ndk_sys::ACameraMetadata_const_entry = entry.assume_init();
                let slice = std::slice::from_raw_parts(entry.data.u8_, 1);
                let facing = slice[0];
                if facing == back_facing {
                    selected_camera = id;

                    // Get camera sensor orientation angle
                    let mut rotation = MaybeUninit::uninit();
                    if ndk_sys::ACameraMetadata_getConstEntry(
                        metadata,
                        ndk_sys::acamera_metadata_tag::ACAMERA_SENSOR_ORIENTATION.0 as _,
                        rotation.as_mut_ptr(),
                    ) == camera_status_t::ACAMERA_OK
                    {
                        let rotation = rotation.assume_init();
                        sensor_orientation = std::slice::from_raw_parts(rotation.data.i32_, 1)[0];
                        log::info!("sensor_orientation: {}", sensor_orientation);
                    }
                }
            }
            ndk_sys::ACameraMetadata_free(metadata);
        }
        // Delete IdList will cause open camera failure
        // ndk_sys::ACameraManager_deleteCameraIdList(ids);

        // Open camera device
        let mut device = MaybeUninit::uninit();
        let mut res = ndk_sys::ACameraManager_openCamera(
            manager,
            selected_camera,
            get_device_callbacks(),
            device.as_mut_ptr(),
        );
        if res == camera_status_t::ACAMERA_ERROR_INVALID_PARAMETER {
            log::error!("Invalid parameter to open camera.");
        }
        let device = device.assume_init();

        // Camera session
        let mut container = MaybeUninit::uninit();
        res = ndk_sys::ACaptureSessionOutputContainer_create(container.as_mut_ptr());
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Capture session output container creation failed.");
        }
        let container = container.assume_init();

        let mut session_output = MaybeUninit::uninit();
        res = ndk_sys::ACaptureSessionOutput_create(
            native_window.ptr().as_ptr(),
            session_output.as_mut_ptr(),
        );
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Capture session image reader output creation failed.");
        }
        let session_output = session_output.assume_init();

        res = ndk_sys::ACaptureSessionOutputContainer_add(container, session_output);
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Couldn't add image reader output session to container.");
        }

        let mut capture_session = MaybeUninit::uninit();
        res = ndk_sys::ACameraDevice_createCaptureSession(
            device,
            container,
            get_state_callbacks(),
            capture_session.as_mut_ptr(),
        );
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Couldn't create capture session.");
        }
        let capture_session = capture_session.assume_init();

        // Request
        let mut request = MaybeUninit::uninit();
        res = ndk_sys::ACameraDevice_createCaptureRequest(
            device,
            ndk_sys::ACameraDevice_request_template::TEMPLATE_PREVIEW,
            request.as_mut_ptr(),
        );
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Couldn't create capture request.");
        }
        let request = request.assume_init();

        let mut target = MaybeUninit::uninit();
        res =
            ndk_sys::ACameraOutputTarget_create(native_window.ptr().as_ptr(), target.as_mut_ptr());
        if res != camera_status_t::ACAMERA_OK {
            log::error!("Couldn't create camera output target.");
        }
        let target = target.assume_init();
        res = ndk_sys::ACaptureRequest_addTarget(request, target);

        if res != camera_status_t::ACAMERA_OK {
            log::error!("Couldn't add capture request to camera output target.");
        }

        Self {
            capture_session,
            request,
            sensor_orientation,
        }
    }

    pub unsafe fn capture(&mut self) {
        ndk_sys::ACameraCaptureSession_capture(
            self.capture_session,
            get_capture_callbacks(),
            1,
            &mut self.request as _,
            std::ptr::null_mut::<c_int>(),
        );
    }

    pub unsafe fn start_capturing(&mut self) {
        ndk_sys::ACameraCaptureSession_setRepeatingRequest(
            self.capture_session,
            get_capture_callbacks(),
            1,
            &mut self.request as _,
            std::ptr::null_mut::<c_int>(),
        );
    }

    pub unsafe fn stop_capturing(&self) {
        ndk_sys::ACameraCaptureSession_stopRepeating(self.capture_session);
    }
}

fn get_device_callbacks() -> *mut ACameraDevice_StateCallbacks {
    extern "C" fn on_device_state_changes(_ctx: *mut c_void, _dev: *mut ACameraDevice) {
        // TODO...
        // let camera = Box::from_raw(ctx);
        log::info!(" camera on_device_state_changes");
    }

    extern "C" fn on_device_error_changes(_ctx: *mut c_void, _dev: *mut ACameraDevice, err: i32) {
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

    let callbacks = ACameraDevice_StateCallbacks {
        // https://stackoverflow.com/questions/33929079/rust-ffi-passing-trait-object-as-context-to-call-callbacks-on
        context: (0 as *mut c_void),
        onDisconnected: Some(on_device_state_changes),
        onError: Some(on_device_error_changes),
    };
    let static_ref: &'static mut ACameraDevice_StateCallbacks = Box::leak(Box::new(callbacks));

    static_ref
}

fn get_state_callbacks() -> *mut ACameraCaptureSession_stateCallbacks {
    unsafe extern "C" fn on_session_ready(
        _context: *mut c_void,
        _session: *mut ACameraCaptureSession,
    ) {
        // 每当没有捕捉请求处理时都会回调该方法
        log::info!("camera on_session_ready");
    }
    unsafe extern "C" fn on_session_active(
        _context: *mut c_void,
        _session: *mut ACameraCaptureSession,
    ) {
        log::info!("camera on_session_active");
    }
    unsafe extern "C" fn on_session_close(
        _context: *mut c_void,
        _session: *mut ACameraCaptureSession,
    ) {
        log::info!("camera on_session_close");
    }

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

fn get_capture_callbacks() -> *mut ACameraCaptureSession_captureCallbacks {
    unsafe extern "C" fn on_capture_started(
        _context: *mut ::std::os::raw::c_void,
        _session: *mut ACameraCaptureSession,
        _request: *const ACaptureRequest,
        _timestamp: i64,
    ) {
        log::info!("camera on_capture_started");
    }
    unsafe extern "C" fn on_capture_result(
        _context: *mut ::std::os::raw::c_void,
        _session: *mut ACameraCaptureSession,
        _request: *mut ACaptureRequest,
        _result: *const ACameraMetadata,
    ) {
        log::info!("camera on_capture_result");
    }
    unsafe extern "C" fn on_capture_failed(
        _context: *mut ::std::os::raw::c_void,
        _session: *mut ACameraCaptureSession,
        _request: *mut ACaptureRequest,
        failure: *mut ACameraCaptureFailure,
    ) {
        log::error!("camera on_capture_failed: {:?}", failure);
    }
    unsafe extern "C" fn on_capture_abort(
        _context: *mut ::std::os::raw::c_void,
        _session: *mut ACameraCaptureSession,
        _sequenceId: ::std::os::raw::c_int,
    ) {
        log::error!("camera on_capture_abort");
    }
    unsafe extern "C" fn on_capture_buffer_lost(
        _context: *mut ::std::os::raw::c_void,
        _session: *mut ACameraCaptureSession,
        _request: *mut ACaptureRequest,
        _window: *mut ACameraWindowType,
        _frameNumber: i64,
    ) {
        log::error!("camera on_capture_buffer_lost");
    }

    let callbacks = ACameraCaptureSession_captureCallbacks {
        context: (0 as *mut c_void),
        onCaptureStarted: Some(on_capture_started),
        onCaptureProgressed: None,
        onCaptureCompleted: Some(on_capture_result),
        onCaptureFailed: Some(on_capture_failed),
        onCaptureSequenceCompleted: None,
        onCaptureSequenceAborted: Some(on_capture_abort),
        onCaptureBufferLost: Some(on_capture_buffer_lost),
    };
    let static_ref: &'static mut ACameraCaptureSession_captureCallbacks =
        Box::leak(Box::new(callbacks));

    static_ref
}
