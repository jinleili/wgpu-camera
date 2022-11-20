use ash::vk;
use ash::vk::{
    AndroidHardwareBufferFormatPropertiesANDROID, AndroidHardwareBufferPropertiesANDROID, Bool32,
    ExternalFormatANDROID, Format, SamplerAddressMode, SamplerMipmapMode,
    SamplerYcbcrConversionInfo, StructureType,
};
use hal::{api::Vulkan, Api};
use ndk::hardware_buffer::{HardwareBuffer, HardwareBufferDesc, HardwareBufferUsage};
use ndk_sys::AHardwareBuffer;
use std::mem::MaybeUninit;

pub(crate) unsafe fn get_external_texture(
    ahw_buffer: Option<HardwareBuffer>,
    ac: &mut crate::AndroidCamera,
) -> Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))> {
    if ahw_buffer.is_none() {
        return None;
    }
    let buffer = ahw_buffer.unwrap();
    let mut result = MaybeUninit::uninit();
    ndk_sys::AHardwareBuffer_describe(buffer.as_ptr(), result.as_mut_ptr());
    let desc = result.assume_init();
    log::info!("AHardwareBuffer_describe, {:?}", desc);

    let width = desc.width;
    let height = desc.height;

    let (image, format_info, sampler_info) = ac.device.as_hal::<Vulkan, _, _>(|device| {
        let device = device.unwrap().raw_device();
        // Get properties from a AHardwareBuffer
        let mut format_info = AndroidHardwareBufferFormatPropertiesANDROID::default();
        let mut properties_info = AndroidHardwareBufferPropertiesANDROID::default();
        properties_info.p_next =
            <*mut AndroidHardwareBufferFormatPropertiesANDROID>::cast(&mut format_info);
        let res = (ac.ahb_fn.get_android_hardware_buffer_properties_android)(
            device.handle(),
            buffer.as_ptr() as _,
            &mut properties_info as _,
        );
        if res != vk::Result::SUCCESS {
            log::error!("Couldn't get external buffer properties.: {:?}", res);
        }
        log::info!("format_info: {:?}", format_info);

        // Create an image to bind to this AHardwareBuffer
        let mut external_create_info = vk::ExternalMemoryImageCreateInfo::default();
        external_create_info.handle_types =
            vk::ExternalMemoryHandleTypeFlags::ANDROID_HARDWARE_BUFFER_ANDROID;
        let mut external_format = ExternalFormatANDROID::default();
        external_format.p_next =
            <*mut vk::ExternalMemoryImageCreateInfo>::cast(&mut external_create_info);

        let mut create_info = vk::ImageCreateInfo {
            s_type: StructureType::IMAGE_CREATE_INFO,
            p_next: <*const ExternalFormatANDROID>::cast(&mut external_format),
            flags: vk::ImageCreateFlags::from_raw(0),
            image_type: vk::ImageType::TYPE_2D,
            format: match format_info.format {
                Format::UNDEFINED => {
                    external_format.external_format = format_info.external_format;
                    Format::UNDEFINED
                }
                _ => format_info.format,
            },
            // format: vk::Format::R8G8B8A8_UNORM,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: desc.layers,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::STORAGE,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ::std::ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
        };

        let image = device
            .create_image(&create_info, None)
            .expect("Failed to create image");

        // Allocate device memory
        let mut import_info = vk::ImportAndroidHardwareBufferInfoANDROID::default();
        import_info.buffer = buffer.as_ptr() as _;
        let mut mem_allocate_info = vk::MemoryDedicatedAllocateInfo::default();
        mem_allocate_info.p_next =
            <*const vk::ImportAndroidHardwareBufferInfoANDROID>::cast(&import_info);
        mem_allocate_info.image = image;

        let mut allocate_info = vk::MemoryAllocateInfo {
            s_type: StructureType::MEMORY_ALLOCATE_INFO,
            p_next: <*const vk::MemoryDedicatedAllocateInfo>::cast(&mut mem_allocate_info),
            allocation_size: properties_info.allocation_size,
            // memory_type_index: {
            //     let mem_req = device.get_image_memory_requirements(image);
            //     find_memorytype_index(
            //         mem_req.memory_type_bits,
            //         &ac.pd_mem_properties,
            //         vk::MemoryPropertyFlags::from_raw(0),
            //     )
            //     .expect("Failed to find image memorytype index")
            // },
            memory_type_index: find_memorytype_index(
                properties_info.memory_type_bits,
                &ac.pd_mem_properties,
                vk::MemoryPropertyFlags::from_raw(0),
            )
            .expect("Failed to find image memorytype index"),
        };
        let device_mem = device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate image memory");

        // Bind image to the device memory
        device
            .bind_image_memory(image, device_mem, 0)
            .expect("Failed to bind image memory");

        let sampler_info = if ac.ycbcr_conv_info.is_none() {
            Some(create_ycbcr_sampler(device, &external_format, &format_info))
        } else {
            None
        };

        (image, format_info, sampler_info)
    });

    if let Some((vk_sampler, ycbcr_conv_info)) = sampler_info {
        let hal_sampler = <<Vulkan as Api>::Device>::sampler_from_raw(vk_sampler, None);
        let ycbcr_sampler = ac.device.create_sampler_from_hal::<Vulkan>(
            hal_sampler,
            &wgpu::SamplerDescriptor {
                label: Some("create_sampler_from_hal"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        );
        ac.set_ycbcr_sampler(ycbcr_sampler, ycbcr_conv_info);
    }

    let (texture, texture_view) = create_texture_and_view(ac, image, width, height, &format_info);

    Some((texture, texture_view, (width, height)))
}

unsafe fn create_texture_and_view(
    ac: &crate::AndroidCamera,
    image: vk::Image,
    width: u32,
    height: u32,
    format_info: &AndroidHardwareBufferFormatPropertiesANDROID,
) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture_desc = hal::TextureDescriptor {
        label: Some("AHardwareBuffer imported texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: hal::TextureUses::RESOURCE | hal::TextureUses::STORAGE_READ_WRITE,
        memory_flags: hal::MemoryFlags::PREFER_COHERENT,
    };

    let hal_texture = <<Vulkan as Api>::Device>::texture_from_raw(image, &texture_desc, None);

    let hal_texture_view = ac.device.as_hal::<Vulkan, _, _>(|device| {
        let hal_device = device.unwrap();

        let img_view_info = vk::ImageViewCreateInfo {
            s_type: StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: <*const SamplerYcbcrConversionInfo>::cast(ac.ycbcr_conv_info.as_ref().unwrap()),
            flags: vk::ImageViewCreateFlags::default(),
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: format_info.format,
            components: vk::ComponentMapping::default(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };
        let vk_img_view = hal_device
            .raw_device()
            .create_image_view(&img_view_info, None)
            .expect("Failed to create vk image view");

        hal_device.texture_view_from_raw(
            &hal_texture,
            vk_img_view,
            &hal::TextureViewDescriptor {
                label: Some("hal::TextureViewDescriptor"),
                format,
                dimension: wgpu::TextureViewDimension::D2,
                usage: hal::TextureUses::RESOURCE | hal::TextureUses::STORAGE_READ_WRITE,
                range: wgpu::ImageSubresourceRange::default(),
            },
            None,
        )
    });

    let texture = ac.device.create_texture_from_hal::<Vulkan>(
        hal_texture,
        &wgpu::TextureDescriptor {
            label: Some("AHardwareBuffer imported texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        },
    );

    let view = texture
        .create_view_from_hal::<Vulkan>(hal_texture_view, &wgpu::TextureViewDescriptor::default());

    (texture, view)
}

fn find_memorytype_index(
    memory_type_bits: u32,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            ((1 << index) & memory_type_bits) != 0 && (memory_type.property_flags & flags) == flags
        })
        .map(|(index, _memory_type)| index as _)
}

unsafe fn create_ycbcr_sampler(
    raw_device: &ash::Device,
    external_format: &ExternalFormatANDROID,
    format_info: &AndroidHardwareBufferFormatPropertiesANDROID,
) -> (vk::Sampler, SamplerYcbcrConversionInfo) {
    let conv_info = vk::SamplerYcbcrConversionCreateInfo {
        s_type: StructureType::SAMPLER_YCBCR_CONVERSION_CREATE_INFO,
        p_next: <*const ExternalFormatANDROID>::cast(external_format),
        format: format_info.format,
        ycbcr_model: match format_info.format {
            Format::UNDEFINED => format_info.suggested_ycbcr_model,
            // SD YUV
            _ => vk::SamplerYcbcrModelConversion::YCBCR_601,
        },
        ycbcr_range: format_info.suggested_ycbcr_range,
        components: format_info.sampler_ycbcr_conversion_components,
        x_chroma_offset: format_info.suggested_x_chroma_offset,
        y_chroma_offset: format_info.suggested_y_chroma_offset,
        chroma_filter: vk::Filter::LINEAR,
        force_explicit_reconstruction: vk::Bool32::default(),
    };

    let mut sampler_info = vk::SamplerCreateInfo {
        s_type: StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::default(),
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: SamplerMipmapMode::default(),
        address_mode_u: SamplerAddressMode::default(),
        address_mode_v: SamplerAddressMode::default(),
        address_mode_w: SamplerAddressMode::default(),
        mip_lod_bias: 0.0,
        anisotropy_enable: Bool32::default(),
        max_anisotropy: 1.0,
        compare_enable: Bool32::default(),
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.0,
        max_lod: 0.0,
        border_color: vk::BorderColor::default(),
        unnormalized_coordinates: Bool32::default(),
    };

    let ycbcr_conv_info = SamplerYcbcrConversionInfo {
        s_type: StructureType::SAMPLER_YCBCR_CONVERSION_INFO,
        p_next: ::std::ptr::null(),
        conversion: raw_device
            .create_sampler_ycbcr_conversion(&conv_info, None)
            .expect("Cannot create sampler ycbcr conversion"),
    };

    sampler_info.p_next = <*const SamplerYcbcrConversionInfo>::cast(&ycbcr_conv_info);

    let vk_sampler = raw_device
        .create_sampler(&sampler_info, None)
        .expect("Cannot create vk sampler");

    (vk_sampler, ycbcr_conv_info)
}
