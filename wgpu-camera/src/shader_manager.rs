use crate::FilterType;
use wgpu::ShaderModule;
pub struct ShaderManager {
    pub original: ShaderModule,
    pub ascii_art: ShaderModule,
    pub cross_hatch: ShaderModule,
    pub edge_detection: ShaderModule,
}

impl ShaderManager {
    pub fn new(device: &wgpu::Device) -> Self {
        ShaderManager {
            original: create_shader_module(
                device,
                include_str!("../../wgsl_preprocessed/original.wgsl"),
                Some("original shader"),
            ),
            ascii_art: create_shader_module(
                device,
                include_str!("../../wgsl_preprocessed/ascii_art.wgsl"),
                Some("ascii_art shader"),
            ),
            cross_hatch: create_shader_module(
                device,
                include_str!("../../wgsl_preprocessed/cross_hatching.wgsl"),
                Some("cross_hatch shader"),
            ),
            edge_detection: create_shader_module(
                device,
                include_str!("../../wgsl_preprocessed/edge_detection.wgsl"),
                Some("edge_detection shader"),
            ),
        }
    }

    pub fn get_shader_ref(&self, ty: FilterType) -> &ShaderModule {
        match ty {
            FilterType::Original => &self.original,
            FilterType::AsciiArt => &self.ascii_art,
            FilterType::CrossHatch => &self.cross_hatch,
            FilterType::EdgeDetection => &self.edge_detection,
        }
    }
}

fn create_shader_module(device: &wgpu::Device, shader: &str, label: Option<&str>) -> ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader)),
    })
}
