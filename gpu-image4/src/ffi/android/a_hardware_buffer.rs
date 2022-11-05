#[cfg_attr(target_os = "android", path = "util.rs")]
mod util;

pub fn test() {
    let device_desc = wgpu::DeviceDescriptor::default();
    let instance = util::create_instance();
    let (device, queue) = util::create_device(&instance, &device_desc);
    log::info!("-----------: {:?}, {:?}", device, queue);
}
