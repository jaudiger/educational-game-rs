use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const BUSH_SHADER_PATH: &str = "shaders/bush.wgsl";

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct BushUniform {
    primary: LinearRgba,
    highlight: LinearRgba,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct BushMaterial {
    #[uniform(0)]
    uniform: BushUniform,
}

impl BushMaterial {
    pub fn new(primary: Color, highlight: Color) -> Self {
        Self {
            uniform: BushUniform {
                primary: primary.to_linear(),
                highlight: highlight.to_linear(),
            },
        }
    }
}

impl UiMaterial for BushMaterial {
    fn fragment_shader() -> ShaderRef {
        BUSH_SHADER_PATH.into()
    }
}
