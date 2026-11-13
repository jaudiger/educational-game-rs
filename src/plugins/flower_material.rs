use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const FLOWER_SHADER_PATH: &str = "shaders/flower.wgsl";

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct FlowerUniform {
    primary: LinearRgba,
    secondary: LinearRgba,
    accent: LinearRgba,
    motion: Vec4,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct FlowerMaterial {
    #[uniform(0)]
    uniform: FlowerUniform,
}

impl FlowerMaterial {
    pub fn new(primary: Color, secondary: Color, accent: Color) -> Self {
        Self {
            uniform: FlowerUniform {
                primary: primary.to_linear(),
                secondary: secondary.to_linear(),
                accent: accent.to_linear(),
                motion: Vec4::ZERO,
            },
        }
    }

    pub fn set_bend(&mut self, displacement: f32) {
        self.uniform.motion.x = displacement;
    }
}

impl UiMaterial for FlowerMaterial {
    fn fragment_shader() -> ShaderRef {
        FLOWER_SHADER_PATH.into()
    }
}
