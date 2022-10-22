use crate::wgpu_canvas::WgpuCanvas;
use app_surface::{AppSurface, IOSViewObj};

#[no_mangle]
pub fn create_wgpu_canvas(ios_obj: IOSViewObj) -> *mut libc::c_void {
    let obj = WgpuCanvas::new(AppSurface::new(ios_obj));
    // 使用 Box 对 Rust 对象进行装箱操作。
    // 我们无法将 Rust 对象直接传递给外部语言，通过装箱来传递此对象的胖指针
    let box_obj = Box::new(obj);
    // into_raw 返回指针的同时，将此对象的内存管理权转交给调用方
    Box::into_raw(box_obj) as *mut libc::c_void
}

#[no_mangle]
pub fn enter_frame(obj: *mut libc::c_void) {
    // 获取到指针指代的 Rust 对象的可变借用
    let obj = unsafe { &mut *(obj as *mut WgpuCanvas) };
    obj.enter_frame();
}

#[no_mangle]
pub fn set_external_texture(
    obj: *mut libc::c_void,
    raw: *mut std::ffi::c_void,
    width: i32,
    height: i32,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuCanvas) };
    let texture_extent = wgpu::Extent3d {
        width: width as u32,
        height: height as u32,
        depth_or_array_layers: 1,
    };
    let external_texture = unsafe {
        let hal_tex = <hal::api::Metal as hal::Api>::Device::texture_from_raw(
            std::mem::transmute(raw),
            mtl::MTLPixelFormat::BGRA8Unorm,
            mtl::MTLTextureType::D2,
            1,
            1,
            hal::CopyExtent {
                width: texture_extent.width,
                height: texture_extent.height,
                depth: 1,
            },
        );
        obj.app_surface
            .device
            .create_texture_from_hal::<hal::api::Metal>(
                hal_tex,
                &wgpu::TextureDescriptor {
                    label: None,
                    size: texture_extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                },
            )
    };
    obj.set_external_texture(external_texture);
}
