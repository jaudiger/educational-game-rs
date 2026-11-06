use bevy::prelude::*;

use crate::ui::theme;

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

#[derive(Component, Reflect)]
#[require(UiTransform)]
pub(super) struct SummarySunPulse;

#[derive(Component, Reflect)]
#[require(UiTransform)]
pub(super) struct SummarySunRays;

#[derive(Component, Reflect)]
#[require(UiTransform)]
pub(super) struct SummarySunSparkle {
    phase: f32,
}

#[derive(Clone, Copy)]
pub(super) struct SummaryThemeStyle {
    pub(super) background: Color,
    pub(super) accent: Color,
    pub(super) curtain: Color,
    pub(super) cloud_primary: Color,
    pub(super) cloud_secondary: Color,
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
            cloud_primary: Color::srgb(1.0, 1.0, 1.0),
            cloud_secondary: Color::srgb(0.93, 0.97, 1.0),
        }
    }
}

pub(super) fn summary_backdrop() -> impl Bundle + use<> {
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
        children![summary_sun()],
    )
}

fn summary_sun() -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(80.0),
            top: percent(7.0),
            width: theme::scaled(152.0),
            height: theme::scaled(152.0),
            ..default()
        },
        UiTransform::default(),
        SummaryBackdropDecoration {
            exit_direction: Vec2::new(1.0, -0.5),
        },
        Pickable::IGNORE,
        children![sun_pulse()],
    )
}

fn sun_pulse() -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: theme::layout::FILL_PARENT,
            height: theme::layout::FILL_PARENT,
            ..default()
        },
        UiTransform::default(),
        SummarySunPulse,
        Pickable::IGNORE,
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent.spawn(sun_rays());
            parent.spawn(sun_disc());
            for sparkle in SUN_SPARKLES {
                parent.spawn(sun_sparkle(sparkle));
            }
        })),
    )
}

#[derive(Clone, Copy)]
struct SunRaySpec {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    rotation_degrees: f32,
}

const SUN_RAYS: [SunRaySpec; 8] = [
    SunRaySpec {
        left: 48.0,
        top: 0.0,
        width: 7.0,
        height: 21.0,
        rotation_degrees: 0.0,
    },
    SunRaySpec {
        left: 72.0,
        top: 8.0,
        width: 7.0,
        height: 21.0,
        rotation_degrees: 45.0,
    },
    SunRaySpec {
        left: 85.0,
        top: 48.0,
        width: 21.0,
        height: 7.0,
        rotation_degrees: 0.0,
    },
    SunRaySpec {
        left: 72.0,
        top: 72.0,
        width: 7.0,
        height: 21.0,
        rotation_degrees: -45.0,
    },
    SunRaySpec {
        left: 48.0,
        top: 85.0,
        width: 7.0,
        height: 21.0,
        rotation_degrees: 0.0,
    },
    SunRaySpec {
        left: 21.0,
        top: 72.0,
        width: 7.0,
        height: 21.0,
        rotation_degrees: 45.0,
    },
    SunRaySpec {
        left: 0.0,
        top: 48.0,
        width: 21.0,
        height: 7.0,
        rotation_degrees: 0.0,
    },
    SunRaySpec {
        left: 21.0,
        top: 8.0,
        width: 7.0,
        height: 21.0,
        rotation_degrees: -45.0,
    },
];

fn sun_rays() -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            top: px(0.0),
            width: theme::layout::FILL_PARENT,
            height: theme::layout::FILL_PARENT,
            ..default()
        },
        UiTransform::default(),
        SummarySunRays,
        Pickable::IGNORE,
        Children::spawn(SpawnIter(SUN_RAYS.into_iter().map(sun_ray))),
    )
}

fn sun_ray(spec: SunRaySpec) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(spec.left),
            top: percent(spec.top),
            width: theme::scaled(spec.width),
            height: theme::scaled(spec.height),
            border_radius: BorderRadius::all(percent(50.0)),
            ..default()
        },
        UiTransform::from_rotation(Rot2::degrees(spec.rotation_degrees)),
        BackgroundColor(Color::srgb(1.0, 0.65, 0.13)),
        Pickable::IGNORE,
    )
}

fn sun_disc() -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(11.0),
            top: percent(11.0),
            width: percent(78.0),
            height: percent(78.0),
            border_radius: BorderRadius::all(percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.57, 0.12, 0.46)),
        Pickable::IGNORE,
        children![(
            Node {
                position_type: PositionType::Absolute,
                left: percent(8.0),
                top: percent(8.0),
                width: percent(84.0),
                height: percent(84.0),
                border_radius: BorderRadius::all(percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 0.65, 0.16)),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(11.0),
                    top: percent(11.0),
                    width: percent(78.0),
                    height: percent(78.0),
                    border_radius: BorderRadius::all(percent(50.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 0.86, 0.35)),
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(19.0),
                        top: percent(13.0),
                        width: percent(30.0),
                        height: percent(22.0),
                        border_radius: BorderRadius::all(percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 0.84, 0.72)),
                )],
            )],
        )],
    )
}

#[derive(Clone, Copy)]
struct SunSparkleSpec {
    left: f32,
    top: f32,
    size: f32,
    phase: f32,
}

const SUN_SPARKLES: [SunSparkleSpec; 4] = [
    SunSparkleSpec {
        left: 15.0,
        top: 27.0,
        size: 17.0,
        phase: 0.0,
    },
    SunSparkleSpec {
        left: 75.0,
        top: 26.0,
        size: 13.0,
        phase: 1.4,
    },
    SunSparkleSpec {
        left: 23.0,
        top: 69.0,
        size: 12.0,
        phase: 2.8,
    },
    SunSparkleSpec {
        left: 72.0,
        top: 68.0,
        size: 17.0,
        phase: 4.2,
    },
];

fn sun_sparkle(spec: SunSparkleSpec) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(spec.left),
            top: percent(spec.top),
            width: theme::scaled(spec.size),
            height: theme::scaled(spec.size),
            ..default()
        },
        UiTransform::default(),
        SummarySunSparkle { phase: spec.phase },
        Pickable::IGNORE,
        children![
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(42.0),
                    top: px(0.0),
                    width: percent(16.0),
                    height: theme::layout::FILL_PARENT,
                    border_radius: BorderRadius::all(percent(50.0)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                Pickable::IGNORE,
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    top: percent(42.0),
                    width: theme::layout::FILL_PARENT,
                    height: percent(16.0),
                    border_radius: BorderRadius::all(percent(50.0)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                Pickable::IGNORE,
            ),
        ],
    )
}

pub(super) fn animate_sun_pulse(
    time: Res<Time>,
    mut suns: Query<&mut UiTransform, With<SummarySunPulse>>,
) {
    let pulse = (time.elapsed_secs() * 0.8).sin();
    let scale = pulse.mul_add(0.0175, 1.0175);
    for mut transform in &mut suns {
        transform.scale = Vec2::splat(scale);
    }
}

pub(super) fn animate_sun_rays(
    time: Res<Time>,
    mut rays: Query<&mut UiTransform, With<SummarySunRays>>,
) {
    let angle = (time.elapsed_secs() * 0.7).sin() * 12.0;
    for mut transform in &mut rays {
        transform.rotation = Rot2::degrees(angle);
    }
}

pub(super) fn animate_sun_sparkles(
    time: Res<Time>,
    mut sparkles: Query<(&SummarySunSparkle, &mut UiTransform)>,
) {
    let elapsed = time.elapsed_secs();
    for (sparkle, mut transform) in &mut sparkles {
        let twinkle = (elapsed.mul_add(2.6, sparkle.phase)).sin();
        transform.scale = Vec2::splat(twinkle.mul_add(0.25, 0.9));
        transform.rotation = Rot2::radians(twinkle * 0.12);
    }
}

pub(super) fn summary_curtain(style: SummaryThemeStyle) -> impl Bundle + use<> {
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
        Children::spawn(SpawnIter(REVEAL_SHAPES.into_iter().map(move |spec| {
            summary_reveal_shape(spec, style.cloud_primary, style.cloud_secondary)
        }))),
    )
}

fn summary_reveal_shape(
    spec: RevealShapeSpec,
    primary: Color,
    secondary: Color,
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
        SummaryCloud {
            exit_direction: spec.exit_direction,
        },
        Pickable::IGNORE,
        children![
            cloud_lobe(15.0, 36.0, 64.0, 48.0, primary),
            cloud_lobe(0.0, 19.0, 49.0, 60.0, secondary),
            cloud_lobe(45.0, 11.0, 43.0, 67.0, primary),
            cloud_lobe(59.0, 33.0, 36.0, 49.0, secondary),
        ],
    )
}

fn cloud_lobe(left: f32, top: f32, width: f32, height: f32, color: Color) -> impl Bundle + use<> {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(left),
            top: percent(top),
            width: percent(width),
            height: percent(height),
            border_radius: BorderRadius::all(percent(50.0)),
            ..default()
        },
        BackgroundColor(color),
        Pickable::IGNORE,
    )
}
