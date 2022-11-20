use ash::vk;
use hal::{api::Vulkan, Api};
use ndk::{
    hardware_buffer::{HardwareBuffer, HardwareBufferUsage},
    media::image_reader::{Image, ImageFormat, ImageReader},
    native_window::NativeWindow,
};

pub struct SessionOutput {
    pub image_reader: ImageReader,
    pub native_window: NativeWindow,
    pub image: *mut vk::Image,
    // images: [Option<Image>; 4],
    // buffers: [Option<HardwareBuffer>; 4],
    cur_index: usize,
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

        let mut image = std::ptr::null_mut::<vk::Image>();
        Self {
            image_reader,
            native_window,
            image,
            // images: [None; 4],
            // buffers: [None; 4],
            cur_index: 0,
        }
    }

    pub unsafe fn get_latest_buffer(&mut self) -> Option<HardwareBuffer> {
        match self.image_reader.acquire_latest_image().unwrap() {
            Some(image) => {
                log::info!("camera AImageReader_acquireLatestImage OK!");
                match image.get_hardware_buffer() {
                    Ok(buf) => {
                        // self.cur_index += 1;
                        // if self.cur_index == self.images.len() {
                        //     self.cur_index = 0;
                        // }
                        // self.images[self.cur_index] = Some(image);
                        // self.buffers[self.cur_index] = Some(buf);
                        log::info!("camera AImage_getHardwareBuffer OK!");
                        // return self.buffers[self.cur_index].as_ref();
                        return Some(buf);
                        // return None;
                    }
                    _ => log::error!("Failed to acquire hardware buffer."),
                };
            }
            None => log::info!("camera AImageReader_acquireLatestImage: No buffer available."),
        };
        None
    }
}
