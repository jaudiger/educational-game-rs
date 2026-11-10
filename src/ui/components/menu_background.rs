use crate::plugins::menu_decoration::{MenuDecorationShape, MenuDecorationVisual};
use crate::ui::animation::FloatingMenuDecoration;
use crate::ui::theme;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct MenuBackgroundStyle(f32);

impl MenuBackgroundStyle {
    pub const fn new(opacity: f32) -> Self {
        Self(opacity)
    }

    const fn opacity(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy)]
enum HorizontalAnchor {
    Left(Val),
    Right(Val),
}

#[derive(Clone, Copy)]
enum VerticalAnchor {
    Top(Val),
    Bottom(Val),
}

#[derive(Clone, Copy)]
struct Decoration {
    width: f32,
    height: f32,
    horizontal: HorizontalAnchor,
    vertical: VerticalAnchor,
    color: Color,
    shape: MenuDecorationShape,
    radius: Val,
    phase: f32,
    amplitude: f32,
    speed: f32,
}

const DECORATIONS: [Decoration; 5] = [
    Decoration {
        width: 250.0,
        height: 250.0,
        horizontal: HorizontalAnchor::Left(Val::Percent(-8.0)),
        vertical: VerticalAnchor::Top(Val::Percent(5.0)),
        color: theme::colors::PASTEL_BLUE,
        shape: MenuDecorationShape::Ellipse,
        radius: Val::Percent(50.0),
        phase: 0.0,
        amplitude: 12.0,
        speed: std::f32::consts::TAU / 12.0,
    },
    Decoration {
        width: 320.0,
        height: 170.0,
        horizontal: HorizontalAnchor::Right(Val::Percent(-5.0)),
        vertical: VerticalAnchor::Top(Val::Percent(-6.0)),
        color: theme::colors::PASTEL_PEACH,
        shape: MenuDecorationShape::RoundedRectangle,
        radius: Val::Px(72.0),
        phase: std::f32::consts::PI,
        amplitude: 14.0,
        speed: std::f32::consts::TAU / 14.0,
    },
    Decoration {
        width: 300.0,
        height: 210.0,
        horizontal: HorizontalAnchor::Left(Val::Percent(12.0)),
        vertical: VerticalAnchor::Bottom(Val::Percent(-12.0)),
        color: theme::colors::PASTEL_LAVENDER,
        shape: MenuDecorationShape::RoundedRectangle,
        radius: Val::Px(54.0),
        phase: 0.8,
        amplitude: 11.0,
        speed: std::f32::consts::TAU / 10.0,
    },
    Decoration {
        width: 230.0,
        height: 230.0,
        horizontal: HorizontalAnchor::Right(Val::Percent(-8.0)),
        vertical: VerticalAnchor::Bottom(Val::Percent(7.0)),
        color: theme::colors::PASTEL_MINT,
        shape: MenuDecorationShape::Ellipse,
        radius: Val::Percent(50.0),
        phase: 2.4,
        amplitude: 13.0,
        speed: std::f32::consts::TAU / 13.0,
    },
    Decoration {
        width: 110.0,
        height: 190.0,
        horizontal: HorizontalAnchor::Left(Val::Percent(-4.0)),
        vertical: VerticalAnchor::Top(Val::Percent(43.0)),
        color: theme::colors::PASTEL_PEACH,
        shape: MenuDecorationShape::Ellipse,
        radius: Val::Percent(50.0),
        phase: 4.0,
        amplitude: 10.0,
        speed: std::f32::consts::TAU / 15.0,
    },
];

pub fn menu_background(style: MenuBackgroundStyle) -> impl Bundle + use<> {
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
        Children::spawn(SpawnIter(
            DECORATIONS
                .into_iter()
                .map(move |decoration| decoration_bundle(decoration, style)),
        )),
    )
}

fn decoration_bundle(decoration: Decoration, style: MenuBackgroundStyle) -> impl Bundle + use<> {
    let (left, right) = match decoration.horizontal {
        HorizontalAnchor::Left(value) => (value, Val::Auto),
        HorizontalAnchor::Right(value) => (Val::Auto, value),
    };
    let (top, bottom) = match decoration.vertical {
        VerticalAnchor::Top(value) => (value, Val::Auto),
        VerticalAnchor::Bottom(value) => (Val::Auto, value),
    };
    (
        Node {
            position_type: PositionType::Absolute,
            left,
            right,
            top,
            bottom,
            width: theme::scaled(decoration.width),
            height: theme::scaled(decoration.height),
            border_radius: BorderRadius::all(decoration.radius),
            ..default()
        },
        MenuDecorationVisual::new(decoration.shape, decoration.color, style.opacity()),
        Pickable::IGNORE,
        FloatingMenuDecoration {
            phase: decoration.phase,
            amplitude: decoration.amplitude,
            speed: decoration.speed,
        },
    )
}
