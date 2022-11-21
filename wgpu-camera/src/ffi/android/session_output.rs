use ndk::{
    hardware_buffer::{HardwareBuffer, HardwareBufferUsage},
    media::image_reader::{ImageFormat, ImageReader},
    native_window::NativeWindow,
};

pub struct SessionOutput {
    pub image_reader: ImageReader,
    pub native_window: NativeWindow,
}

impl SessionOutput {
    pub unsafe fn new(width: i32, height: i32) -> Self {
        // Format can only use PRIVATE, RGBA_8888 will cause error:
        // acquireImageLocked: Output buffer format: 0x7fa30c06, ImageReader configured format: 0x1
        let image_reader = ImageReader::new_with_usage(
            width,
            height,
            ImageFormat::PRIVATE,
            HardwareBufferUsage::GPU_SAMPLED_IMAGE,
            4,
        )
        .expect("Failed to create image reader.");

        let native_window = image_reader
            .get_window()
            .expect("Failed to obtain window handle.");

        Self {
            image_reader,
            native_window,
        }
    }

    pub unsafe fn get_latest_buffer(&mut self) -> Option<HardwareBuffer> {
        match self.image_reader.acquire_latest_image().unwrap() {
            Some(image) => {
                match image.get_hardware_buffer() {
                    Ok(buf) => {
                        return Some(buf);
                    }
                    _ => log::error!("Failed to acquire hardware buffer."),
                };
            }
            None => log::info!("camera AImageReader_acquireLatestImage: No buffer available."),
        };
        None
    }
}
