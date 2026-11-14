use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_render::UiMaterialPlugin;
use bevy::ui_render::ui_material::UiMaterial;
use bevy::window::PrimaryWindow;

use crate::states::ActiveLesson;
use crate::ui::theme;

const SUN_SHADER_PATH: &str = "shaders/sun.wgsl";
const SUN_ART_SIZE: f32 = 152.0;
const SUN_REFERENCE_NODE_SIZE: f32 = 184.0;
const SUN_NODE_SIZE: f32 = 96.0;
const SUN_ART_SCALE: f32 = SUN_REFERENCE_NODE_SIZE / SUN_ART_SIZE;
const DEFAULT_GLARE_ANGLE: f32 = -2.356_194_5;
const DEFAULT_GLARE_STRENGTH: f32 = 0.25;
const ACTIVE_GLARE_STRENGTH: f32 = 1.0;
const GLARE_DEAD_ZONE: f32 = 6.0;
const GLARE_SMOOTHING: f32 = 10.0;

pub struct SunMaterialPlugin;

impl Plugin for SunMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<SunMaterial>::default())
            .add_systems(Update, update_sun_glare.run_if(in_state(ActiveLesson)));
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

#[derive(Component, Reflect)]
#[require(RelativeCursorPosition)]
pub struct SunGlare {
    angle: f32,
    strength: f32,
}

impl Default for SunGlare {
    fn default() -> Self {
        Self {
            angle: DEFAULT_GLARE_ANGLE,
            strength: DEFAULT_GLARE_STRENGTH,
        }
    }
}

pub fn sky_sun_material() -> SunMaterial {
    let direction = Vec2::new(DEFAULT_GLARE_ANGLE.cos(), DEFAULT_GLARE_ANGLE.sin());
    SunMaterial::new(direction, DEFAULT_GLARE_STRENGTH, SUN_ART_SCALE)
}

pub fn sky_sun(material: Handle<SunMaterial>) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            right: theme::scaled(80.0),
            top: theme::scaled(8.0),
            width: theme::scaled(SUN_NODE_SIZE),
            height: theme::scaled(SUN_NODE_SIZE),
            ..default()
        },
        MaterialNode(material),
        SunGlare::default(),
        Pickable::IGNORE,
    )
}

fn update_sun_glare(
    time: Res<Time>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<SunMaterial>>,
    mut suns: Query<(
        &RelativeCursorPosition,
        &ComputedNode,
        &MaterialNode<SunMaterial>,
        &mut SunGlare,
    )>,
) {
    let interpolation = 1.0 - (-GLARE_SMOOTHING * time.delta_secs()).exp();
    let dead_zone_squared = GLARE_DEAD_ZONE * GLARE_DEAD_ZONE;

    for (cursor, node, material_node, mut glare) in &mut suns {
        let local_position = if primary_window.focused {
            cursor
                .normalized
                .map(|normalized| normalized * node.size())
                .filter(|position| position.is_finite())
        } else {
            None
        };
        let (target_angle, target_strength) = match local_position {
            Some(position)
                if (position * node.inverse_scale_factor()).length_squared()
                    <= dead_zone_squared =>
            {
                (glare.angle, glare.strength)
            }
            Some(position) => (position.y.atan2(position.x), ACTIVE_GLARE_STRENGTH),
            None => (DEFAULT_GLARE_ANGLE, DEFAULT_GLARE_STRENGTH),
        };

        let angle_delta = (target_angle - glare.angle + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let angle_change = angle_delta * interpolation;
        let strength_change = (target_strength - glare.strength) * interpolation;
        if angle_change.abs() <= f32::EPSILON && strength_change.abs() <= f32::EPSILON {
            continue;
        }

        glare.angle += angle_change;
        glare.strength += strength_change;
        let direction = Vec2::new(glare.angle.cos(), glare.angle.sin());
        if (angle_change.abs() > f32::EPSILON || strength_change.abs() > f32::EPSILON)
            && let Some(mut material) = materials.get_mut(&material_node.0)
        {
            material.set_glare(direction, glare.strength);
        }
    }
}
