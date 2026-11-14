use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::UiMaterial;

const LANDSCAPE_SHADER_PATH: &str = "shaders/landscape.wgsl";

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub struct RidgeGeometry {
    center_x_percent: f32,
    baseline_y: f32,
    left_half_width: f32,
    right_half_width: f32,
    main_height: f32,
    shoulder_height: f32,
    shoulder_offset: f32,
    shoulder_half_width: f32,
}

impl RidgeGeometry {
    pub const DEFAULT: Self = Self {
        center_x_percent: 0.73,
        baseline_y: 0.34,
        left_half_width: 0.39,
        right_half_width: 0.30,
        main_height: 0.19,
        shoulder_height: 0.045,
        shoulder_offset: 0.26,
        shoulder_half_width: 0.27,
    };

    pub(crate) fn center_percent_for_offset(self, offset: f32, aspect: f32) -> f32 {
        self.center_x_percent + offset / aspect
    }

    pub(crate) fn surface_y_at_offset(self, offset: f32) -> f32 {
        let half_width = if offset < 0.0 {
            self.left_half_width
        } else {
            self.right_half_width
        };
        let ridge_offset = offset / half_width;
        let shoulder_offset = (offset + self.shoulder_offset) / self.shoulder_half_width;
        let ridge_height = self.shoulder_height.mul_add(
            hill_bump(shoulder_offset),
            self.main_height * hill_bump(ridge_offset),
        );
        self.baseline_y - ridge_height
    }
}

fn hill_bump(value: f32) -> f32 {
    let remaining = value.mul_add(-value, 1.0).max(0.0);
    remaining * remaining
}

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
    Ridge = 2,
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
    ridge: RidgeGeometry,
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
        let path = PathGeometry::DEFAULT;
        Self {
            uniform: LandscapeUniform {
                center_curve: Vec4::new(
                    path.center_base,
                    path.first_bend,
                    path.second_bend,
                    path.third_bend,
                ),
                center_start: Vec4::new(path.first_start, path.second_start, path.third_start, 0.0),
                center_end: Vec4::new(path.first_end, path.second_end, path.third_end, 0.0),
                path_width: Vec4::new(
                    path.width_min,
                    path.width_max,
                    path.width_start,
                    path.width_end,
                ),
                ridge: RidgeGeometry::DEFAULT,
                layer: layer.id(),
                padding: Vec3::ZERO,
            },
        }
    }
}

impl UiMaterial for LandscapeMaterial {
    fn fragment_shader() -> ShaderRef {
        LANDSCAPE_SHADER_PATH.into()
    }
}
