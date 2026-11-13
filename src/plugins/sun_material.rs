use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::UiMaterialPlugin;
use bevy::ui_render::ui_material::UiMaterial;

const SUN_SHADER_PATH: &str = "shaders/sun.wgsl";

pub struct SunMaterialPlugin;

impl Plugin for SunMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<SunMaterial>::default());
    }
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct SunUniform {
    glare: Vec4,
    geometry: Vec4,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct SunMaterial {
    #[uniform(0)]
    uniform: SunUniform,
}

impl SunMaterial {
    pub const fn new(glare_direction: Vec2, glare_strength: f32, art_scale: f32) -> Self {
        Self {
            uniform: SunUniform {
                glare: Vec4::new(glare_direction.x, glare_direction.y, glare_strength, 0.0),
                geometry: Vec4::new(art_scale, 0.0, 0.0, 0.0),
            },
        }
    }

    pub const fn set_glare(&mut self, direction: Vec2, strength: f32) {
        self.uniform.glare = Vec4::new(direction.x, direction.y, strength, 0.0);
    }
}

impl UiMaterial for SunMaterial {
    fn fragment_shader() -> ShaderRef {
        SUN_SHADER_PATH.into()
    }
}
