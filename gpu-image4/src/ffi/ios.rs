use crate::wgpu_canvas::WgpuCanvas;
use app_surface::{AppSurface, IOSViewObj};
use idroid::math::{Position, Rect};

#[no_mangle]
pub fn create_wgpu_canvas(ios_obj: IOSViewObj) -> *mut libc::c_void {
    let wgpu_obj = WgpuCanvas::new(AppSurface::new(ios_obj));
    let box_obj = Box::new(wgpu_obj);
    Box::into_raw(box_obj) as *mut libc::c_void
}

#[no_mangle]
pub fn set_filter(wgpu_obj: *mut libc::c_void, ty: crate::FilterType, param: f32) {
    let wgpu_obj = unsafe { &mut *(wgpu_obj as *mut WgpuCanvas) };
    wgpu_obj.set_filter(ty, param);
}

#[no_mangle]
pub fn change_filter_param(wgpu_obj: *mut libc::c_void, param: f32) {
    let wgpu_obj = unsafe { &mut *(wgpu_obj as *mut WgpuCanvas) };
    wgpu_obj.change_filter_param(param);
}

#[no_mangle]
pub fn set_external_texture(
    wgpu_obj: *mut libc::c_void,
    raw: *mut std::ffi::c_void,
    width: i32,
    height: i32,
) {
    let obj = unsafe { &mut *(wgpu_obj as *mut WgpuCanvas) };
    // let w_ratio = obj.app_surface.config.width as f32 / width as f32;
    // let h_ratio = obj.app_surface.config.height as f32 / height as f32;
    // let (w, h) = if h_ratio > w_ratio {
    //     let w = obj.app_surface.config.width as f32 / (width as f32 * h_ratio);
    //     (w, 1.0)
    // } else {
    //     let h = obj.app_surface.config.height as f32 / (height as f32 * w_ratio);
    //     (1.0, h)
    // };
    // let tex_rect = Rect::new(w, h, (0.5, 0.5).into());

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
    obj.set_external_texture(external_texture, (width as f32, height as f32));
}

#[no_mangle]
pub fn enter_frame(wgpu_obj: *mut libc::c_void) {
    let wgpu_obj = unsafe { &mut *(wgpu_obj as *mut WgpuCanvas) };
    wgpu_obj.enter_frame();
}
