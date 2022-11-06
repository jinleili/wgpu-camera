use ndk_sys::{
    media_status_t, AHardwareBuffer, AImageReader, AImageReader_acquireLatestImage,
    AImageReader_getWindow, AImageReader_newWithUsage, ANativeWindow,
};

pub struct SessionOutput {
    pub image_reader: *mut AImageReader,
    pub native_window: *mut ANativeWindow,
}

impl SessionOutput {
    pub unsafe fn new(width: i32, height: i32) -> Self {
        let mut image_reader = std::ptr::null_mut::<AImageReader>();
        let mut res = AImageReader_newWithUsage(
            width,
            height,
            ndk_sys::AIMAGE_FORMATS::AIMAGE_FORMAT_PRIVATE.0 as _,
            ndk_sys::AHardwareBuffer_UsageFlags::AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE.0 as _,
            4,
            &mut image_reader as _,
        );
        if res != media_status_t::AMEDIA_OK {
            log::error!("Failed to create image reader.");
        }

        let mut native_window = std::ptr::null_mut::<ANativeWindow>();
        res = AImageReader_getWindow(image_reader, &mut native_window as _);
        if res != media_status_t::AMEDIA_OK {
            log::error!("Failed to obtain window handle.");
        }
        Self {
            image_reader,
            native_window,
        }
    }
}
