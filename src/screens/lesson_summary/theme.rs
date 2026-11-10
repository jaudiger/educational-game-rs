use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::window::PrimaryWindow;

use crate::plugins::cloud_material::ProceduralCloudUiMaterial;
use crate::ui::theme;

const SUMMARY_SUN_SHADER_PATH: &str = "shaders/summary_sun.wgsl";
const SUMMARY_LENS_FLARE_SHADER_PATH: &str = "shaders/summary_lens_flare.wgsl";
const SUMMARY_SUN_ART_SIZE: f32 = 152.0;
const SUMMARY_SUN_NODE_SIZE: f32 = 184.0;
const SUMMARY_SUN_ART_SCALE: f32 = SUMMARY_SUN_NODE_SIZE / SUMMARY_SUN_ART_SIZE;
const DEFAULT_GLARE_ANGLE: f32 = -2.356_194_5;
const DEFAULT_GLARE_STRENGTH: f32 = 0.25;
const ACTIVE_GLARE_STRENGTH: f32 = 1.0;
const GLARE_DEAD_ZONE: f32 = 6.0;
const GLARE_SMOOTHING: f32 = 10.0;

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct SummarySunUniform {
    glare: Vec4,
    geometry: Vec4,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub(super) struct SummarySunMaterial {
    #[uniform(0)]
    uniform: SummarySunUniform,
}

impl SummarySunMaterial {
    pub(super) fn new() -> Self {
        let direction = Vec2::new(DEFAULT_GLARE_ANGLE.cos(), DEFAULT_GLARE_ANGLE.sin());
        Self {
            uniform: SummarySunUniform {
                glare: Vec4::new(direction.x, direction.y, DEFAULT_GLARE_STRENGTH, 0.0),
                geometry: Vec4::new(SUMMARY_SUN_ART_SCALE, 0.0, 0.0, 0.0),
            },
        }
    }

    const fn set_glare(&mut self, direction: Vec2, strength: f32) {
        self.uniform.glare = Vec4::new(direction.x, direction.y, strength, 0.0);
    }
}

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct SummaryLensFlareUniform {
    source: Vec4,
    cursor: Vec4,
}

#[derive(Asset, AsBindGroup, Clone, Reflect)]
pub(super) struct SummaryLensFlareMaterial {
    #[uniform(0)]
    uniform: SummaryLensFlareUniform,
}

impl SummaryLensFlareMaterial {
    pub(super) const fn new() -> Self {
        Self {
            uniform: SummaryLensFlareUniform {
                source: Vec4::ZERO,
                cursor: Vec4::ZERO,
            },
        }
    }

    const fn set_source(
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

impl UiMaterial for SummaryLensFlareMaterial {
    fn fragment_shader() -> ShaderRef {
        SUMMARY_LENS_FLARE_SHADER_PATH.into()
    }
}

#[derive(Component, Reflect)]
#[require(RelativeCursorPosition)]
pub(super) struct SummarySunGlare {
    angle: f32,
    strength: f32,
    distance: f32,
}

impl Default for SummarySunGlare {
    fn default() -> Self {
        Self {
            angle: DEFAULT_GLARE_ANGLE,
            strength: DEFAULT_GLARE_STRENGTH,
            distance: 0.0,
        }
    }
}

#[derive(Component, Reflect)]
pub(super) struct SummaryLensFlare;

impl UiMaterial for SummarySunMaterial {
    fn fragment_shader() -> ShaderRef {
        SUMMARY_SUN_SHADER_PATH.into()
    }
}

#[derive(Component, Reflect)]
pub(super) struct SummaryCurtain {
    pub(super) color: Color,
}

#[derive(Component, Reflect)]
pub(super) struct SummaryCloud {
    pub(super) exit_direction: Vec2,
}

#[derive(Component, Reflect)]
#[require(UiTransform)]
pub(super) struct SummaryBackdropDecoration {
    pub(super) exit_direction: Vec2,
}

#[derive(Clone, Copy)]
pub(super) struct SummaryThemeStyle {
    pub(super) background: Color,
    pub(super) accent: Color,
    pub(super) curtain: Color,
}

#[derive(Clone, Copy)]
struct RevealShapeSpec {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    exit_direction: Vec2,
}

const REVEAL_SHAPES: [RevealShapeSpec; 8] = [
    RevealShapeSpec {
        left: -22.0,
        top: -18.0,
        width: 62.0,
        height: 58.0,
        exit_direction: Vec2::new(-1.0, -1.0),
    },
    RevealShapeSpec {
        left: 16.0,
        top: -34.0,
        width: 72.0,
        height: 62.0,
        exit_direction: Vec2::new(0.0, -1.0),
    },
    RevealShapeSpec {
        left: 67.0,
        top: -15.0,
        width: 60.0,
        height: 58.0,
        exit_direction: Vec2::new(1.0, -1.0),
    },
    RevealShapeSpec {
        left: -34.0,
        top: 22.0,
        width: 68.0,
        height: 58.0,
        exit_direction: Vec2::new(-1.0, 0.0),
    },
    RevealShapeSpec {
        left: 72.0,
        top: 20.0,
        width: 66.0,
        height: 60.0,
        exit_direction: Vec2::new(1.0, 0.0),
    },
    RevealShapeSpec {
        left: -19.0,
        top: 66.0,
        width: 61.0,
        height: 57.0,
        exit_direction: Vec2::new(-1.0, 1.0),
    },
    RevealShapeSpec {
        left: 20.0,
        top: 72.0,
        width: 70.0,
        height: 64.0,
        exit_direction: Vec2::new(0.0, 1.0),
    },
    RevealShapeSpec {
        left: 66.0,
        top: 64.0,
        width: 60.0,
        height: 56.0,
        exit_direction: Vec2::new(1.0, 1.0),
    },
];

impl SummaryThemeStyle {
    pub(super) const fn sky() -> Self {
        Self {
            background: Color::srgb(0.68, 0.85, 0.96),
            accent: Color::srgb(0.16, 0.43, 0.72),
            curtain: Color::srgb(0.85, 0.93, 0.98),
        }
    }
}

pub(super) fn summary_backdrop(sun_material: Handle<SummarySunMaterial>) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: theme::layout::FILL_PARENT,
            height: theme::layout::FILL_PARENT,
            ..default()
        },
        Pickable::IGNORE,
        children![summary_sun(sun_material)],
    )
}

pub(super) fn summary_lens_flare(
    material: Handle<SummaryLensFlareMaterial>,
) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: theme::layout::FILL_PARENT,
            height: theme::layout::FILL_PARENT,
            ..default()
        },
        MaterialNode(material),
        SummaryLensFlare,
        SummaryBackdropDecoration {
            exit_direction: Vec2::new(1.0, -0.5),
        },
        Pickable::IGNORE,
    )
}

fn summary_sun(sun_material: Handle<SummarySunMaterial>) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            right: theme::scaled(0.0),
            top: theme::scaled(0.0),
            width: theme::scaled(SUMMARY_SUN_NODE_SIZE),
            height: theme::scaled(SUMMARY_SUN_NODE_SIZE),
            ..default()
        },
        MaterialNode(sun_material),
        SummarySunGlare::default(),
        SummaryBackdropDecoration {
            exit_direction: Vec2::new(1.0, -0.5),
        },
        Pickable::IGNORE,
    )
}

pub(super) fn update_summary_sun_glare(
    time: Res<Time>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<SummarySunMaterial>>,
    mut suns: Query<(
        &RelativeCursorPosition,
        &ComputedNode,
        &MaterialNode<SummarySunMaterial>,
        &mut SummarySunGlare,
    )>,
) {
    let interpolation = 1.0 - (-GLARE_SMOOTHING * time.delta_secs()).exp();
    let dead_zone_squared = GLARE_DEAD_ZONE * GLARE_DEAD_ZONE;

    for (cursor, node, material_node, mut glare) in &mut suns {
        let default_distance = node.size().min_element() * 2.5;
        let local_position = if primary_window.focused {
            cursor
                .normalized
                .map(|normalized| normalized * node.size())
                .filter(|position| position.is_finite())
        } else {
            None
        };
        let (target_angle, target_strength, target_distance) = match local_position {
            Some(position)
                if (position * node.inverse_scale_factor()).length_squared()
                    <= dead_zone_squared =>
            {
                let distance = if glare.distance > 0.0 {
                    glare.distance
                } else {
                    default_distance
                };
                (glare.angle, glare.strength, distance)
            }
            Some(position) => (
                position.y.atan2(position.x),
                ACTIVE_GLARE_STRENGTH,
                position.length(),
            ),
            None => (
                DEFAULT_GLARE_ANGLE,
                DEFAULT_GLARE_STRENGTH,
                default_distance,
            ),
        };

        let angle_delta = (target_angle - glare.angle + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let angle_change = angle_delta * interpolation;
        let strength_change = (target_strength - glare.strength) * interpolation;
        let distance_change = (target_distance - glare.distance) * interpolation;
        if angle_change.abs() <= f32::EPSILON
            && strength_change.abs() <= f32::EPSILON
            && distance_change.abs() <= f32::EPSILON
        {
            continue;
        }

        glare.angle += angle_change;
        glare.strength += strength_change;
        glare.distance += distance_change;
        let direction = Vec2::new(glare.angle.cos(), glare.angle.sin());
        if (angle_change.abs() > f32::EPSILON || strength_change.abs() > f32::EPSILON)
            && let Some(mut material) = materials.get_mut(&material_node.0)
        {
            material.set_glare(direction, glare.strength);
        }
    }
}

pub(super) fn update_summary_lens_flare(
    mut materials: ResMut<Assets<SummaryLensFlareMaterial>>,
    sun: Single<(&ComputedNode, &UiGlobalTransform, &SummarySunGlare), With<SummarySunGlare>>,
    flare: Single<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &MaterialNode<SummaryLensFlareMaterial>,
        ),
        With<SummaryLensFlare>,
    >,
) {
    let (sun_node, sun_transform, sun_glare) = *sun;
    let (flare_node, flare_transform, material_node) = *flare;
    if sun_node.is_empty() || flare_node.is_empty() {
        return;
    }
    let Some(inverse_flare_transform) = flare_transform.try_inverse() else {
        return;
    };

    let source_position = sun_transform.affine().transform_point2(Vec2::ZERO);
    let source_position = inverse_flare_transform.transform_point2(source_position)
        / flare_node.size()
        + Vec2::splat(0.5);
    let source_radius = sun_node.size().min_element() * 0.5 / SUMMARY_SUN_ART_SCALE;
    let cursor_offset =
        Vec2::new(sun_glare.angle.cos(), sun_glare.angle.sin()) * sun_glare.distance;
    let cursor_position = sun_transform.affine().transform_point2(cursor_offset);
    let cursor_position = inverse_flare_transform.transform_point2(cursor_position)
        / flare_node.size()
        + Vec2::splat(0.5);
    if !source_position.is_finite()
        || !cursor_position.is_finite()
        || !source_radius.is_finite()
        || source_radius <= 0.0
    {
        return;
    }

    if let Some(mut material) = materials.get_mut(&material_node.0) {
        let source = material.uniform.source;
        let cursor = material.uniform.cursor;
        let position_changed =
            Vec2::new(source.x, source.y).distance_squared(source_position) > 0.000_001;
        let cursor_changed =
            Vec2::new(cursor.x, cursor.y).distance_squared(cursor_position) > 0.000_001;
        let radius_changed = (source.z - source_radius).abs() > 0.01;
        let strength_changed = (source.w - sun_glare.strength).abs() > 0.001;
        if position_changed || cursor_changed || radius_changed || strength_changed {
            material.set_source(
                source_position,
                source_radius,
                cursor_position,
                sun_glare.strength,
            );
        }
    }
}

pub(super) fn summary_curtain(
    style: SummaryThemeStyle,
    cloud_material: Handle<ProceduralCloudUiMaterial>,
) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: theme::layout::FILL_PARENT,
            height: theme::layout::FILL_PARENT,
            ..default()
        },
        BackgroundColor(style.curtain),
        SummaryCurtain {
            color: style.curtain,
        },
        Pickable::IGNORE,
        Children::spawn(SpawnIter(
            REVEAL_SHAPES
                .into_iter()
                .map(move |spec| summary_reveal_shape(spec, cloud_material.clone())),
        )),
    )
}

fn summary_reveal_shape(
    spec: RevealShapeSpec,
    cloud_material: Handle<ProceduralCloudUiMaterial>,
) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(spec.left),
            top: percent(spec.top),
            width: percent(spec.width),
            height: percent(spec.height),
            ..default()
        },
        UiTransform::default(),
        MaterialNode(cloud_material),
        SummaryCloud {
            exit_direction: spec.exit_direction,
        },
        Pickable::IGNORE,
    )
}
