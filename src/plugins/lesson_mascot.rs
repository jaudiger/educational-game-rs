//! Shared composition and idle motion for theme mascots.
//!
//! Lesson screens select a mascot from the active exploration theme. The balloon
//! scene can also be reused by other screens, which provide their own layout and
//! own any screen-specific markers or transitions.

use bevy::prelude::*;
use bevy::scene::{EntityCommandsSceneExt, Scene, SceneComponent, bsn};

use crate::data::ExplorationTheme;
use crate::plugins::balloon_material::{BalloonPart, BalloonUiMaterial};
use crate::states::ActiveLesson;
use crate::ui::theme;

/// Registers idle animation systems for the shared Sky balloon.
pub struct LessonMascotPlugin;

impl Plugin for LessonMascotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_swaying_envelope, animate_pulled_basket).run_if(in_state(ActiveLesson)),
        );
    }
}

/// Marker for the balloon envelope node, animated with a pendular sway
/// whose pivot is simulated at the top-center of the visual.
#[derive(Component, Clone, Default, Reflect)]
struct SwayingEnvelope;

/// Marker for the balloon basket, subtly pulled by the envelope's sway
/// with a dephased rotation and horizontal follow.
#[derive(Component, Clone, Default, Reflect)]
struct PulledBasket;

#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(SkyBalloonMascotProps)]
struct SkyBalloonMascot;

#[derive(Default)]
struct SkyBalloonMascotProps {
    envelope_material: Handle<BalloonUiMaterial>,
    basket_material: Handle<BalloonUiMaterial>,
    layout: SkyBalloonLayout,
}

#[derive(Clone)]
pub struct SkyBalloonAssets {
    envelope_material: Handle<BalloonUiMaterial>,
    basket_material: Handle<BalloonUiMaterial>,
}

#[derive(Clone, Copy)]
pub struct SkyBalloonLayout {
    pub right: Val,
    pub bottom: Val,
    pub root_width: Val,
    pub envelope_width: f32,
    pub basket_width: f32,
    pub basket_overlap: f32,
}

impl Default for SkyBalloonLayout {
    fn default() -> Self {
        Self {
            right: percent(8.0),
            bottom: percent(47.0),
            root_width: Val::Auto,
            envelope_width: SKY_ENVELOPE_WIDTH,
            basket_width: SKY_BASKET_WIDTH,
            basket_overlap: SKY_BASKET_OVERLAP,
        }
    }
}

/// Design-pixel width for the balloon envelope.
const SKY_ENVELOPE_WIDTH: f32 = 196.0;
/// Design-pixel width for the balloon basket.
const SKY_BASKET_WIDTH: f32 = 129.0;
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

/// Spawns the lesson mascot for the current [`ExplorationTheme`].
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
    exploration_theme: ExplorationTheme,
    materials: &mut Assets<BalloonUiMaterial>,
) {
    match exploration_theme {
        ExplorationTheme::Sky => spawn_sky_balloon_mascot(parent, materials),
        // Future themes: add a spawn function and match arm here.
        ExplorationTheme::Ocean | ExplorationTheme::Space => {}
    }
}

pub fn create_sky_balloon_assets(materials: &mut Assets<BalloonUiMaterial>) -> SkyBalloonAssets {
    SkyBalloonAssets {
        envelope_material: materials.add(BalloonUiMaterial::new(BalloonPart::Envelope)),
        basket_material: materials.add(BalloonUiMaterial::new(BalloonPart::Basket)),
    }
}

/// Spawns an animated balloon (envelope + basket) for the Sky theme.
///
/// The balloon reuses the procedural cursor design in UI material nodes. It
/// is absolutely positioned in the middle-right area of the background,
/// resting on the painted grass. The envelope gently sways while the
/// basket follows with a dephased pull.
fn spawn_sky_balloon_mascot(
    parent: &mut ChildSpawnerCommands,
    materials: &mut Assets<BalloonUiMaterial>,
) {
    let assets = create_sky_balloon_assets(materials);
    parent
        .spawn_empty()
        .apply_scene(sky_balloon_scene(assets, SkyBalloonLayout::default()));
}

pub fn sky_balloon_scene(assets: SkyBalloonAssets, layout: SkyBalloonLayout) -> impl Scene {
    bsn! {
        @SkyBalloonMascot {
            @envelope_material: {assets.envelope_material},
            @basket_material: {assets.basket_material},
            @layout: {layout},
        }
    }
}

impl SkyBalloonMascot {
    fn scene(props: SkyBalloonMascotProps) -> impl Scene {
        let layout = props.layout;
        let envelope_height = BalloonPart::Envelope.height_for_width(layout.envelope_width);
        let basket_height = BalloonPart::Basket.height_for_width(layout.basket_width);

        bsn! {
            Node {
                position_type: PositionType::Absolute,
                right: {layout.right},
                bottom: {layout.bottom},
                width: {layout.root_width},
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
            }
            Children [
                SwayingEnvelope
                Node {
                    width: theme::scaled(layout.envelope_width),
                    height: theme::scaled(envelope_height),
                }
                MaterialNode<BalloonUiMaterial>({props.envelope_material})
                --
                PulledBasket
                Node {
                    width: theme::scaled(layout.basket_width),
                    height: theme::scaled(basket_height),
                    margin: UiRect::top(theme::scaled(-layout.basket_overlap)),
                }
                MaterialNode<BalloonUiMaterial>({props.basket_material})
            ]
        }
    }
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
