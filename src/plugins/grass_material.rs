use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const GRASS_SHADER_PATH: &str = "shaders/grass.wgsl";

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct GrassUniform {
    primary: LinearRgba,
    secondary: LinearRgba,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct GrassMaterial {
    #[uniform(0)]
    uniform: GrassUniform,
}

impl GrassMaterial {
    pub fn new(primary: Color, secondary: Color) -> Self {
        Self {
            uniform: GrassUniform {
                primary: primary.to_linear(),
                secondary: secondary.to_linear(),
            },
        }
    }
}

impl UiMaterial for GrassMaterial {
    fn fragment_shader() -> ShaderRef {
        GRASS_SHADER_PATH.into()
    }
}
