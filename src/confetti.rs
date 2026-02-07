use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ConfettiMaterial {
    /// x = time, y = framebuffer_width, z = framebuffer_height, w = unused
    #[uniform(0)]
    pub params: Vec4,
}

impl Material2d for ConfettiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/confetti.wgsl".into()
    }
}
