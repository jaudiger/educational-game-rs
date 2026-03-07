//! Lesson mascot: a visual callback to the active cursor theme.
//!
//! Spawns an animated decorative element on the lesson play screen,
//! absolutely positioned on the background behind the question container.
//! The mascot is visible through the card's semi-transparent overlay.
//! It matches the current [`MapTheme`] from settings.
//!
//! # Extensibility
//!
//! Each [`MapTheme`] variant provides its own mascot via a dedicated spawn
//! function. To add a mascot for a new theme:
//! 1. Create a `spawn_xxx_mascot` function following the pattern of
//!    [`spawn_sky_balloon_mascot`].
//! 2. Add the corresponding match arm in [`spawn_lesson_mascot`].
//! 3. Add any theme-specific animation systems to [`LessonMascotPlugin`].

use bevy::prelude::*;

use crate::data::MapTheme;
use crate::states::AppState;
use crate::ui::theme;

/// Registers animation systems for the lesson play mascot.
pub struct LessonMascotPlugin;

impl Plugin for LessonMascotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_swaying_envelope, animate_pulled_basket)
                .run_if(in_state(AppState::LessonPlay)),
        );
    }
}

/// Marker for the balloon envelope node, animated with a pendular sway
/// whose pivot is simulated at the top-center of the image.
#[derive(Component, Reflect)]
struct SwayingEnvelope;

/// Marker for the balloon basket, subtly pulled by the envelope's sway
/// with a dephased rotation and horizontal follow.
#[derive(Component, Reflect)]
struct PulledBasket;

/// Design-pixel width for the balloon envelope (aspect ratio 256:320 = 0.8).
const SKY_ENVELOPE_WIDTH: f32 = 196.0;
/// Design-pixel height for the balloon envelope.
const SKY_ENVELOPE_HEIGHT: f32 = 245.0;
/// Design-pixel width for the balloon basket (aspect ratio 128:80 = 1.6).
const SKY_BASKET_WIDTH: f32 = 129.0;
/// Design-pixel height for the balloon basket.
const SKY_BASKET_HEIGHT: f32 = 81.0;
/// Vertical overlap between envelope bottom and basket top (design pixels).
const SKY_BASKET_OVERLAP: f32 = 12.0;

/// Angular frequency of the pendular sway (rad/s).
const SWAY_FREQUENCY: f32 = 1.0;
/// Maximum rotation angle of the envelope (rad).
const SWAY_AMPLITUDE: f32 = 0.005;
/// Maximum rotation angle of the basket (rad), dephased to follow the envelope.
const BASKET_SWAY_AMPLITUDE: f32 = 0.008;
/// Phase delay of the basket relative to the envelope (seconds).
const BASKET_PHASE_DELAY: f32 = 0.3;
/// Maximum horizontal pull on the basket from the envelope sway (px).
const BASKET_PULL_PX: f32 = 3.0;
/// Vertical bob amplitude (px) for the gentle lift-off effect.
const BOB_AMPLITUDE: f32 = 2.0;
/// Vertical bob frequency (rad/s).
const BOB_FREQUENCY: f32 = 0.8;

/// Spawns the lesson mascot for the current [`MapTheme`].
///
/// Called during lesson play screen setup. The mascot is absolutely
/// positioned on the background (middle-right, at the grass level) and
/// must be spawned **before** the top bar and question container so that
/// it renders behind them in z-order. The card's semi-transparent overlay
/// lets the mascot show through.
///
/// Each theme variant dispatches to its own spawn function. Themes
/// without a mascot yet simply spawn nothing.
pub fn spawn_lesson_mascot(
    parent: &mut ChildSpawnerCommands,
    map_theme: MapTheme,
    asset_server: &AssetServer,
) {
    match map_theme {
        MapTheme::Sky => spawn_sky_balloon_mascot(parent, asset_server),
        // Future themes: add a spawn function and match arm here.
        MapTheme::Ocean | MapTheme::Space => {}
    }
}

/// Spawns an animated balloon (envelope + basket) for the Sky theme.
///
/// The balloon reuses the cursor assets displayed as UI image nodes. It
/// is absolutely positioned in the middle-right area of the background,
/// resting on the painted grass. The envelope gently sways while the
/// basket follows with a dephased pull.
fn spawn_sky_balloon_mascot(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    let envelope_image = asset_server.load("cursor/balloon_envelope.png");
    let basket_image = asset_server.load("cursor/balloon_basket.png");

    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: percent(8.0),
            bottom: percent(47.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|children| {
            // Balloon envelope (animated sway)
            children.spawn((
                SwayingEnvelope,
                Node {
                    width: theme::scaled(SKY_ENVELOPE_WIDTH),
                    height: theme::scaled(SKY_ENVELOPE_HEIGHT),
                    ..default()
                },
                ImageNode {
                    image: envelope_image,
                    ..default()
                },
            ));
            // Balloon basket (pulled by envelope sway, overlaps envelope bottom)
            children.spawn((
                PulledBasket,
                Node {
                    width: theme::scaled(SKY_BASKET_WIDTH),
                    height: theme::scaled(SKY_BASKET_HEIGHT),
                    margin: UiRect::top(theme::scaled(-SKY_BASKET_OVERLAP)),
                    ..default()
                },
                ImageNode {
                    image: basket_image,
                    ..default()
                },
            ));
        });
}

/// Applies a gentle pendular sway to the balloon envelope.
///
/// The rotation pivot is simulated at the top-center of the image by
/// combining a center-based rotation with a compensating translation:
///
/// ```text
/// offset.x = -(h/2) * sin(theta)
/// offset.y =  (h/2) * (cos(theta) - 1)
/// ```
///
/// where `h` is the rendered height from [`ComputedNode`].
fn animate_swaying_envelope(
    time: Res<Time>,
    mut query: Query<(&mut UiTransform, &ComputedNode), With<SwayingEnvelope>>,
) {
    let elapsed = time.elapsed_secs();
    let angle = (elapsed * SWAY_FREQUENCY).sin() * SWAY_AMPLITUDE;

    for (mut ui_transform, computed) in &mut query {
        let half_height = computed.size().y * computed.inverse_scale_factor() / 2.0;
        // Gentle vertical bob (simulates lift-off from the ground)
        let bob_y = (elapsed * BOB_FREQUENCY).sin() * BOB_AMPLITUDE;

        ui_transform.rotation = Rot2::radians(angle);
        ui_transform.translation = Val2 {
            x: px(-(half_height * angle.sin())),
            y: px(half_height.mul_add(angle.cos() - 1.0, bob_y)),
        };
    }
}

/// Applies a subtle pull to the basket, following the envelope's sway
/// with a phase delay and reduced amplitude, as if attached by ropes.
fn animate_pulled_basket(time: Res<Time>, mut query: Query<&mut UiTransform, With<PulledBasket>>) {
    let elapsed = time.elapsed_secs();
    // Dephased rotation (follows the envelope with a delay)
    let angle = ((elapsed - BASKET_PHASE_DELAY) * SWAY_FREQUENCY).sin() * BASKET_SWAY_AMPLITUDE;
    // Horizontal pull following the envelope's current tilt direction
    let pull_x = (elapsed * SWAY_FREQUENCY).sin() * BASKET_PULL_PX;

    for mut ui_transform in &mut query {
        // Dephased vertical bob (basket follows envelope lift with delay)
        let bob_y = ((elapsed - BASKET_PHASE_DELAY) * BOB_FREQUENCY).sin() * (BOB_AMPLITUDE * 0.6);

        ui_transform.rotation = Rot2::radians(angle);
        ui_transform.translation = Val2 {
            x: px(pull_x),
            y: px(bob_y),
        };
    }
}
