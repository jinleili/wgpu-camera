use crate::display_node::DisplayNode;
use app_surface::AppSurface;
use bytemuck::Pod;
use idroid::{
    geometry::Plane,
    vertex::{PosTex, Vertex},
    BufferObj,
};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::{
    BindingType, Buffer, BufferBindingType, PipelineLayout, ShaderModule, ShaderStages, Texture,
    TextureFormat,
};
pub(crate) struct FragmentFilterNode {
    bind_groups: HashMap<String, wgpu::BindGroup>,
    display_node: DisplayNode,
}

impl FragmentFilterNode {
    pub fn new(app_surface: &AppSurface, shader_module: &ShaderModule) -> Self {
        Self {
            bind_groups: HashMap::new(),
            display_node: DisplayNode::new::<PosTex>(app_surface, shader_module),
        }
    }
}

impl crate::FilterNode for FragmentFilterNode {
    fn change_filter(&mut self, app_surface: &AppSurface, shader_module: &wgpu::ShaderModule) {
        self.display_node.change_filter(app_surface, shader_module);
    }

    fn update_viewport(&mut self, viewport: (f32, f32, f32, f32)) {
        self.display_node.viewport = viewport;
    }

    fn update_bind_group(
        &mut self,
        app_surface: &AppSurface,
        mvp_buffer: &Buffer,
        params_buffer: &Buffer,
        external_texture: &Texture,
        tex_key: String,
    ) {
        let texture_view = external_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.display_node.create_bind_group(
            app_surface,
            mvp_buffer,
            params_buffer,
            &texture_view,
        );
        self.bind_groups.insert(tex_key, bind_group);
    }

    fn remove_bind_group(&mut self, tex_key: String) {
        self.bind_groups.remove(&tex_key);
    }

    fn enter_frame(
        &mut self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        tex_key: String,
    ) {
        let bind_group = self.bind_groups.get(&tex_key);
        if bind_group.is_none() {
            return;
        }
        self.display_node
            .begin_render_pass(frame_view, encoder, bind_group)
    }
}
