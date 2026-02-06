use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ConfettiMaterial {
    #[uniform(0)]
    pub time: f32,
}

impl Material2d for ConfettiMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/confetti.wgsl".into()
    }
}
