// Desktop
use idroid::{load_texture, AnyTexture};
use std::fs::File;

pub fn get_a_texture(app_surface: &app_surface::AppSurface) -> (wgpu::Texture, wgpu::Extent3d) {
    // let img_name = "pepper512x512";
    // let img_name = "kodim768x512";
    // let img_path = format!("{}/../assets/{}.png", &env!("CARGO_MANIFEST_DIR"), img_name);
    // let decoder = png::Decoder::new(File::open(&img_path).unwrap());

    let img_data = include_bytes!("../../../assets/pepper512x512.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(img_data));

    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    let size = wgpu::Extent3d {
        width: info.width,
        height: info.height,
        depth_or_array_layers: 1,
    };
    let texture = app_surface.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    });
    app_surface.queue.write_texture(
        texture.as_image_copy(),
        &buf,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(info.width * 4),
            rows_per_image: None,
        },
        size,
    );
    (texture, size)
    // let (texture, _sampler) = load_texture::from_path(
    //     &img_path,
    //     app_surface,
    //     wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    //     false,
    // );
    // texture
}
