use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::UiMaterialPlugin;
use bevy::ui_render::ui_material::UiMaterial;

const SUN_SHADER_PATH: &str = "shaders/sun.wgsl";
const SUN_LENS_FLARE_SHADER_PATH: &str = "shaders/sun_lens_flare.wgsl";

pub struct SunMaterialPlugin;

impl Plugin for SunMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UiMaterialPlugin::<SunMaterial>::default(),
            UiMaterialPlugin::<SunLensFlareMaterial>::default(),
        ));
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

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct SunLensFlareUniform {
    source: Vec4,
    cursor: Vec4,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct SunLensFlareMaterial {
    #[uniform(0)]
    uniform: SunLensFlareUniform,
}

impl SunLensFlareMaterial {
    pub const fn new() -> Self {
        Self {
            uniform: SunLensFlareUniform {
                source: Vec4::ZERO,
                cursor: Vec4::ZERO,
            },
        }
    }

    pub fn source(&self) -> (Vec2, f32, f32) {
        let source = self.uniform.source;
        (Vec2::new(source.x, source.y), source.z, source.w)
    }

    pub fn cursor_position(&self) -> Vec2 {
        let cursor = self.uniform.cursor;
        Vec2::new(cursor.x, cursor.y)
    }

    pub const fn set_source(
        &mut self,
        position: Vec2,
        radius: f32,
        cursor_position: Vec2,
        strength: f32,
    ) {
        self.uniform.source = Vec4::new(position.x, position.y, radius, strength);
        self.uniform.cursor = Vec4::new(cursor_position.x, cursor_position.y, 0.0, 0.0);
    }
}

impl UiMaterial for SunLensFlareMaterial {
    fn fragment_shader() -> ShaderRef {
        SUN_LENS_FLARE_SHADER_PATH.into()
    }
}
