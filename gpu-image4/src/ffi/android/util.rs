use std::{
    borrow::Cow,
    ffi::CStr,
    os::unix::prelude::{AsRawFd, RawFd},
    time::Duration,
};

use ash::vk;
use hal::{api::Vulkan, Api};

// Initializes the wgpu instance.
pub fn create_instance() -> wgpu::Instance {
    // We really do need better initialization for Vulkan 1.0 in this example due to the required instance
    // extensions, but that is easily 1000 lines of code and I am lazy. Vulkan 1.1 has promoted all the
    // required instance extensions to core.
    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);

    if let Some(instance) = unsafe { instance.as_hal::<Vulkan>() } {
        // VK_VERSION_1_1
        if instance.shared_instance().driver_api_version() < 4198400 {
            panic!("Vulkan 1.1 is required for this example.")
        }
    }

    instance
}

pub fn create_device(
    instance: &wgpu::Instance,
    desc: &wgpu::DeviceDescriptor,
) -> (wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .enumerate_adapters(wgpu::Backends::VULKAN)
        .filter(|adapter| is_suitable_adapter(adapter, desc.features))
        .next()
        .expect("Failed to find suitable adapter");

    // We have a suitable adapter, we need to manually create the device
    let hal_device = unsafe {
        adapter.as_hal::<Vulkan, _, _>(|adapter| {
            // We only asked for Vulkan adapters
            let adapter = adapter.unwrap();
            let mut enabled_extensions = adapter.required_device_extensions(desc.features);
            enabled_extensions.extend(EXTRA_REQUIRED_EXTENSIONS);

            let phd_limits = &adapter.physical_device_capabilities().properties().limits;
            let uab_types = hal::UpdateAfterBindTypes::from_limits(&desc.limits, phd_limits);
            let mut phd_features =
                adapter.physical_device_features(&enabled_extensions, desc.features, uab_types);

            // Find a queue.
            let family_index = 0; //TODO
            let family_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(family_index)
                .queue_priorities(&[1.0])
                .build();
            let family_infos = [family_info];

            let str_pointers = enabled_extensions
                .iter()
                .map(|&s| {
                    // Safe because `required_extensions` entries have static lifetime.
                    s.as_ptr()
                })
                .collect::<Vec<_>>();

            let pre_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&family_infos)
                .enabled_extension_names(&str_pointers);
            let info = phd_features.add_to_device_create_builder(pre_info).build();

            let raw_device = adapter
                .shared_instance()
                .raw_instance()
                .create_device(adapter.raw_physical_device(), &info, None)
                .expect("Failed to create Vulkan device");

            adapter.device_from_raw(
                raw_device,
                true,
                &enabled_extensions,
                desc.features,
                uab_types,
                family_index,
                0,
            )
        })
    }
    .expect("Failed to create hal device");

    unsafe { adapter.create_device_from_hal(hal_device, desc, None) }
        .expect("Failed to create hal device")
}

const EXTRA_REQUIRED_EXTENSIONS: &[&CStr] = &[
    vk::KhrBindMemory2Fn::name(),
    vk::KhrExternalMemoryFn::name(),
    vk::KhrGetMemoryRequirements2Fn::name(),
    vk::ExtQueueFamilyForeignFn::name(),
    vk::AndroidExternalMemoryAndroidHardwareBufferFn::name(),
];

pub fn is_suitable_adapter(adapter: &wgpu::Adapter, features: wgpu::Features) -> bool {
    unsafe {
        adapter.as_hal::<Vulkan, _, _>(|adapter| {
            // We only asked for Vulkan adapters
            let adapter = adapter.unwrap();
            let physical_device = adapter.raw_physical_device();

            let mut required_extensions = adapter.required_device_extensions(features);
            required_extensions.extend(EXTRA_REQUIRED_EXTENSIONS);

            let extensions = adapter
                .shared_instance()
                .raw_instance()
                .enumerate_device_extension_properties(physical_device)
                .unwrap()
                .iter()
                .map(|properties| {
                    // We need to create an owned CString to prevent the CStr from pointing to dropped values.
                    CStr::from_ptr(&properties.extension_name as *const _).to_owned()
                })
                .collect::<Vec<_>>();

            if !required_extensions
                .iter()
                .all(|&extension| extensions.iter().any(|name| name.as_c_str() == extension))
            {
                return false;
            }

            true
        })
    }
}
