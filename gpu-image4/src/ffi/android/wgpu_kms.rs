//! Example to show how to use external memory objects with wgpu.
//!
//! This example will present directly to a display using kernel mode setting.
//!
//! You will probably need to run this from a tty, or else you will get "Operation not supported" errors

use std::{
    borrow::Cow,
    ffi::CStr,
    os::unix::prelude::{AsRawFd, RawFd},
    time::Duration,
};

use ash::vk;
use drm::control::{connector, CrtcListFilter};
use hal::{api::Vulkan, Api};
use nix::{fcntl::OFlag, sys::stat::Mode};

struct DrmDevice(RawFd);

impl AsRawFd for DrmDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl drm::Device for DrmDevice {}
impl drm::control::Device for DrmDevice {}

fn main() {
    // Start by opening a DRM device that will be used for scanout.
    //
    // We will just assume the DRM device is card0 for the sake of keeping this example short.
    // If you use a multi-gpu setup, you may need to change this.
    const DEVICE: &str = "/dev/dri/card0";

    let fd = nix::fcntl::open(DEVICE, OFlag::O_RDWR | OFlag::O_CLOEXEC, Mode::empty())
        .expect("Failed to open DRM device");
    let gbm = gbm::Device::new(DrmDevice(fd)).expect("Failed to create gbm device");

    // Test that we can scanout and render to the desired format.
    if !gbm.is_format_supported(
        gbm::Format::Argb8888,
        gbm::BufferObjectFlags::SCANOUT | gbm::BufferObjectFlags::RENDERING,
    ) {
        eprintln!("Required format is not supported");
        std::process::exit(1);
    }

    // Create a buffer object that will be use for scanout and rendered to.
    let mut bo = gbm.create_buffer_object_with_modifiers::<()>(
        // use 1920x1080 as the resolution. This may need to be adjusted if the hardware cannot scan out that
        // resolution.
        1920,
        1080,
        // Format selection is a much more complicated process. We use Argb8888 because it is widely supported.
        gbm::Format::Argb8888,
        std::iter::once(gbm::Modifier::Linear),
        // gbm::BufferObjectFlags::SCANOUT | gbm::BufferObjectFlags::RENDERING
    )
    .expect("Failed to create buffer object to render to");

    // Export the buffer object we just created as a Dmabuf for import into wgpu.
    //
    // In order to import the dmabuf into a graphics api, we need information about the planes.
    let dmabuf = bo.fd().expect("Failed to export GBM bo as dmabuf");
    // Get the modifier of the buffer object. This will be needed later to import into wgpu
    let modifier = bo.modifier().unwrap();
    let offset = bo.offset(0).unwrap();
    let stride = bo.stride().unwrap();
    assert_ne!(
        modifier,
        gbm::Modifier::Invalid,
        "GBM returned Invalid modifier"
    );

    let device_desc = wgpu::DeviceDescriptor::default();

    dbg!("Wgpu init");
    // Now we can bring wgpu into the fray.
    let instance = create_instance();
    let (device, queue) = create_device(&instance, &device_desc);

    dbg!("DRM init");
    let (physical_device, memory_fd) = unsafe {
        device.as_hal::<Vulkan, _, _>(|device| {
            let device = device.unwrap();

            let memory_fd = ash::extensions::khr::ExternalMemoryFd::new(
                device.shared_instance().raw_instance(),
                device.raw_device(),
            );

            (device.raw_physical_device(), memory_fd)
        })
    };

    // Test whether the device supports the format and the modifier of the dmabuf.
    let (texture, image, memory) = unsafe {
        let instance = instance.as_hal::<Vulkan>().unwrap();
        let raw_instance = instance.shared_instance().raw_instance();

        // Getting the drm modifier list is a two call process
        let mut modifier_list = vk::DrmFormatModifierPropertiesListEXT::builder();
        let mut props = vk::FormatProperties2::builder().push_next(&mut modifier_list);

        raw_instance.get_physical_device_format_properties2(
            physical_device,
            vk::Format::B8G8R8A8_UNORM,
            &mut props,
        );
        drop(props); // Intentionally drop the properties let go of the borrow on modifier list

        let mut modifiers = Vec::with_capacity(modifier_list.drm_format_modifier_count as usize);
        modifier_list.p_drm_format_modifier_properties = modifiers.as_mut_ptr();
        let mut props = vk::FormatProperties2::builder().push_next(&mut modifier_list);
        raw_instance.get_physical_device_format_properties2(
            physical_device,
            vk::Format::B8G8R8A8_UNORM,
            &mut props,
        );
        drop(props); // Intentionally drop the properties let go of the borrow on modifier list

        modifiers.set_len(modifier_list.drm_format_modifier_count as usize);

        // No valid modifiers in Vulkan, we cannot import the image.
        if modifiers.is_empty() {
            eprintln!("Vulkan does not support the required format with any modifiers");
            std::process::exit(1);
        }

        // Find our specific modifier in the list.
        let _ = modifiers
            .iter()
            .find(|properties| properties.drm_format_modifier == u64::from(modifier))
            .copied()
            .expect("Vulkan does not support the required modifier");

        // We found our modifier, therefore we know that the format is supported.

        // TODO: We're shooting in the dark and there is a lot of validation we need.

        // Figure out the memory type of the dmabuf
        let dmabuf_properties = memory_fd
            .get_memory_fd_properties(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT, dmabuf)
            .expect("Failed to get dmabuf memory properties");

        // Import the texture
        //
        // 1. Create an image with the specified modifier
        // 2. Bind the dmabuf memory to the image.
        let (image, memory) = device.as_hal::<Vulkan, _, _>(|device| {
            let device = device.unwrap();
            let raw_device = device.raw_device();

            // Tell Vulkan we want an image that will be imported for dmabuf external memory.
            let mut external_image_info = vk::ExternalMemoryImageCreateInfo::builder()
                .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
            let subresource_layout = [vk::SubresourceLayout::builder()
                .row_pitch(stride as u64)
                .offset(offset as u64)
                .build()];
            let mut modifier_info = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::builder()
                .drm_format_modifier(u64::from(modifier))
                .plane_layouts(&subresource_layout);
            let image_info = vk::ImageCreateInfo::builder()
                // We tell Vulkan what modifier we want.
                .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
                // We will render to the image
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .extent(vk::Extent3D {
                    width: 1920,
                    height: 1080,
                    depth: 1,
                })
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .array_layers(1)
                .mip_levels(1)
                .format(vk::Format::B8G8R8A8_UNORM)
                .image_type(vk::ImageType::TYPE_2D)
                .samples(vk::SampleCountFlags::TYPE_1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .push_next(&mut external_image_info)
                .push_next(&mut modifier_info);

            dbg!("Create image");
            let image = raw_device
                .create_image(&image_info, None)
                .expect("Failed to create image");
            dbg!("Get image mem reqs");
            let mem_requirements = raw_device.get_image_memory_requirements(image);

            // Now that we have the image, we need to import the dmabuf as memory and bind it to the image.
            // TODO: Find the memory type
            // let mem_properties = raw_instance.get_physical_device_memory_properties(physical_device);
            // let memory_types = &mem_properties.memory_types[..mem_properties.memory_type_count as usize];

            let mut import_memory_info = vk::ImportMemoryFdInfoKHR::builder()
                .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
                // Note: Vulkan consumes the fd, if you need to keep the fd around, you will need to `dup()` it.
                .fd(dmabuf);
            let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::builder().image(image);
            let memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.size)
                // FIXME: This is wrong
                .memory_type_index(0)
                // .memory_type_index(dmabuf_properties.memory_type_bits)
                .push_next(&mut dedicated_info)
                .push_next(&mut import_memory_info);

            dbg!("Import memory");
            let memory = raw_device
                .allocate_memory(&memory_info, None)
                .expect("Failed to allocate memory");
            dbg!("Bind image memory");
            raw_device
                .bind_image_memory(image, memory, 0)
                .expect("Failed to bind image to memory");

            (image, memory)
        });

        let texture_desc = hal::TextureDescriptor {
            label: Some("Dmabuf imported texture"),
            size: wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: hal::TextureUses::COLOR_TARGET,
            memory_flags: hal::MemoryFlags::PREFER_COHERENT,
        };

        // Now we can tell wgpu about our texture
        let hal_texture = <<Vulkan as Api>::Device>::texture_from_raw(
            image,
            &texture_desc,
            Some(vk::QUEUE_FAMILY_EXTERNAL),
            None,
        );

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Dmabuf imported texture"),
            size: wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };

        let texture = device.create_texture_from_hal::<Vulkan>(hal_texture, &texture_desc);

        (texture, image, memory)
    };

    // Render a triangle
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::TextureFormat::Bgra8Unorm.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // Render
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&render_pipeline);
        rpass.draw(0..3, 0..1);
    }

    let index = queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(index));

    use drm::control::Device;

    // Create a framebuffer
    let fb = gbm.add_framebuffer(&bo, 32, 32).unwrap();

    let resource_handles = gbm.resource_handles().unwrap();
    let connectors = resource_handles
        .connectors()
        .iter()
        .map(|&connector| gbm.get_connector(connector, true).unwrap())
        .collect::<Vec<_>>();
    let crtc = resource_handles
        .crtcs()
        .iter()
        .map(|&crtc| gbm.get_crtc(crtc))
        .next()
        .unwrap()
        .unwrap();

    // Filter each connector until we find one that's connected.
    let conn = connectors
        .iter()
        .find(|&i| i.state() == connector::State::Connected)
        .cloned()
        .expect("No connected connectors");

    // Get a matching mode
    let mode = conn
        .modes()
        .iter()
        .find(|mode| mode.size() == (1920, 1080))
        .cloned()
        .expect("Failed to find valid mode");

    // And present
    gbm.set_crtc(
        crtc.handle(),
        Some(fb),
        (0, 0),
        &[conn.handle()],
        Some(mode),
    )
    .expect("Failed to present");

    std::thread::sleep(Duration::from_secs(15));
}

// Initializes the wgpu instance.
fn create_instance() -> wgpu::Instance {
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

fn create_device(
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

fn is_suitable_adapter(adapter: &wgpu::Adapter, features: wgpu::Features) -> bool {
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
