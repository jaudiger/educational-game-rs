use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const GRASS_SHADER_PATH: &str = "shaders/grass.wgsl";

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct GrassUniform {
    primary: LinearRgba,
    secondary: LinearRgba,
    motion: Vec4,
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
                motion: Vec4::ZERO,
            },
        }
    }

    pub fn set_bend(&mut self, displacement: f32) {
        self.uniform.motion.x = displacement;
    }
}

impl UiMaterial for GrassMaterial {
    fn fragment_shader() -> ShaderRef {
        GRASS_SHADER_PATH.into()
    }
}
