use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const LANDSCAPE_SHADER_PATH: &str = "shaders/landscape.wgsl";

#[derive(Clone, Copy, Debug, Reflect)]
pub struct PathGeometry {
    center_base: f32,
    first_bend: f32,
    second_bend: f32,
    third_bend: f32,
    first_start: f32,
    first_end: f32,
    second_start: f32,
    second_end: f32,
    third_start: f32,
    third_end: f32,
    width_min: f32,
    width_max: f32,
    width_start: f32,
    width_end: f32,
}

impl PathGeometry {
    pub const DEFAULT: Self = Self {
        center_base: 0.25,
        first_bend: 0.16,
        second_bend: -0.20,
        third_bend: -0.14,
        first_start: 0.25,
        first_end: 0.48,
        second_start: 0.48,
        second_end: 0.70,
        third_start: 0.70,
        third_end: 0.98,
        width_min: 0.012,
        width_max: 0.13,
        width_start: 0.25,
        width_end: 0.98,
    };

    pub fn center_percent(self, y: f32) -> f32 {
        let first = self.first_bend.mul_add(
            smoothstep(self.first_start, self.first_end, y),
            self.center_base,
        );
        let second = self
            .second_bend
            .mul_add(smoothstep(self.second_start, self.second_end, y), first);
        let offset = self
            .third_bend
            .mul_add(smoothstep(self.third_start, self.third_end, y), second);
        100.0_f32.mul_add(offset, 50.0)
    }

    pub fn half_width_percent(self, y: f32, aspect: f32) -> f32 {
        let path_width = (self.width_max - self.width_min).mul_add(
            smoothstep(self.width_start, self.width_end, y),
            self.width_min,
        );
        100.0 * path_width / aspect
    }

    const fn uniform(self, layer: LandscapeLayer) -> LandscapeUniform {
        LandscapeUniform {
            center_curve: Vec4::new(
                self.center_base,
                self.first_bend,
                self.second_bend,
                self.third_bend,
            ),
            center_start: Vec4::new(self.first_start, self.second_start, self.third_start, 0.0),
            center_end: Vec4::new(self.first_end, self.second_end, self.third_end, 0.0),
            path_width: Vec4::new(
                self.width_min,
                self.width_max,
                self.width_start,
                self.width_end,
            ),
            layer: layer.id(),
            padding: Vec3::ZERO,
        }
    }
}

fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (-2.0_f32).mul_add(t, 3.0)
}

#[derive(Clone, Copy, Debug, Reflect)]
#[repr(u32)]
pub enum LandscapeLayer {
    Sky = 0,
    Meadow = 1,
}

impl LandscapeLayer {
    const fn id(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct LandscapeUniform {
    center_curve: Vec4,
    center_start: Vec4,
    center_end: Vec4,
    path_width: Vec4,
    layer: u32,
    padding: Vec3,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub struct LandscapeMaterial {
    #[uniform(0)]
    uniform: LandscapeUniform,
}

impl LandscapeMaterial {
    pub const fn new(layer: LandscapeLayer) -> Self {
        Self {
            uniform: PathGeometry::DEFAULT.uniform(layer),
        }
    }
}

impl UiMaterial for LandscapeMaterial {
    fn fragment_shader() -> ShaderRef {
        LANDSCAPE_SHADER_PATH.into()
    }
}
