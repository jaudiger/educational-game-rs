use bevy::prelude::*;
use bevy::ui_render::ui_material::MaterialNode;

use crate::plugins::cloud_material::CloudUiMaterial;
use crate::ui::components::full_screen_overlay_node;

#[derive(Component, Reflect)]
pub(super) struct SummaryCurtain {
    pub(super) color: Color,
}

#[derive(Component, Reflect)]
pub(super) struct SummaryCloud {
    pub(super) exit_direction: Vec2,
}

#[derive(Component, Reflect)]
pub(super) struct SummaryDepartureFade;

pub(super) const SUMMARY_CURTAIN_COLOR: Color = Color::srgb(0.85, 0.93, 0.98);

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

pub(super) fn summary_departure_fade(color: Color) -> impl Bundle + use<> {
    (
        full_screen_overlay_node(),
        BackgroundColor(color.with_alpha(0.0)),
        ZIndex(10),
        SummaryDepartureFade,
        Pickable::IGNORE,
    )
}

pub(super) fn summary_curtain(
    color: Color,
    cloud_material: Handle<CloudUiMaterial>,
) -> impl Bundle + use<> {
    (
        full_screen_overlay_node(),
        BackgroundColor(color),
        SummaryCurtain { color },
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
    cloud_material: Handle<CloudUiMaterial>,
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
