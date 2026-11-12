use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const STONE_SHADER_PATH: &str = "shaders/stone.wgsl";
pub const STONE_HALF_WIDTH_RATIO: f32 = 0.3875;

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct StoneUniform {
    primary: LinearRgba,
    secondary: LinearRgba,
    accent: LinearRgba,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct StoneMaterial {
    #[uniform(0)]
    uniform: StoneUniform,
}

impl StoneMaterial {
    pub fn new(primary: Color, secondary: Color, accent: Color) -> Self {
        Self {
            uniform: StoneUniform {
                primary: primary.to_linear(),
                secondary: secondary.to_linear(),
                accent: accent.to_linear(),
            },
        }
    }
}

impl UiMaterial for StoneMaterial {
    fn fragment_shader() -> ShaderRef {
        STONE_SHADER_PATH.into()
    }
}
