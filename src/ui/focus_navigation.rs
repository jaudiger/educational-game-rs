//! Keyboard and gamepad navigation for accessibility.
//!
//! Adds arrow-key / D-pad directional navigation, Tab cycling, and
//! Enter / Space / Gamepad-South activation to all focusable UI elements.

use bevy::input_focus::directional_navigation::DirectionalNavigationPlugin;
use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::math::{CompassOctant, Dir2};
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigator;
use bevy::ui_widgets::{Activate, Button};

use crate::data::GameSettings;

use super::theme;

pub struct FocusNavigationPlugin;

impl Plugin for FocusNavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DirectionalNavigationPlugin);

        // Focus ring visibility is managed manually:
        // shown on keyboard/gamepad input, hidden on mouse click.
        app.insert_resource(InputFocusVisible(false));
        app.init_resource::<DirectionalInput>();

        app.add_systems(
            PreUpdate,
            (process_directional_input, handle_directional_navigation)
                .chain()
                .run_if(gamepad_navigation_enabled),
        )
        .add_systems(
            Update,
            (
                gamepad_activate_focused.run_if(gamepad_navigation_enabled),
                hide_focus_on_mouse_click.run_if(gamepad_navigation_enabled),
                update_focus_ring,
            ),
        );
    }
}

fn gamepad_navigation_enabled(settings: Res<GameSettings>) -> bool {
    settings.gamepad_navigation
}

/// Temporary resource holding the navigation direction for this frame.
#[derive(Resource, Default)]
struct DirectionalInput {
    direction: Option<CompassOctant>,
    activate: bool,
}

/// Reads arrow keys and returns directional axis deltas.
fn keyboard_nav_delta(keyboard: &ButtonInput<KeyCode>) -> (i8, i8) {
    let mut east_west: i8 = 0;
    let mut north_south: i8 = 0;
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        east_west += 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        east_west -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        north_south += 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        north_south -= 1;
    }
    (east_west, north_south)
}

/// Reads D-pad and South button across all connected gamepads, returning accumulated axis deltas and activate flag.
fn gamepad_nav_delta(gamepads: &Query<&Gamepad>) -> (i8, i8, bool) {
    let mut east_west: i8 = 0;
    let mut north_south: i8 = 0;
    let mut activate = false;
    for gamepad in gamepads {
        if gamepad.just_pressed(GamepadButton::DPadRight) {
            east_west += 1;
        }
        if gamepad.just_pressed(GamepadButton::DPadLeft) {
            east_west -= 1;
        }
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            north_south += 1;
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            north_south -= 1;
        }
        if gamepad.just_pressed(GamepadButton::South) {
            activate = true;
        }
    }
    (east_west, north_south, activate)
}

fn process_directional_input(
    mut input: ResMut<DirectionalInput>,
    mut focus_visible: ResMut<InputFocusVisible>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {
    let (kew, kns) = keyboard_nav_delta(&keyboard);
    let (gew, gns, ga) = gamepad_nav_delta(&gamepads);
    let east_west = kew + gew;
    let north_south = kns + gns;
    let activate = ga;

    let direction = Dir2::from_xy(f32::from(east_west), f32::from(north_south))
        .ok()
        .map(CompassOctant::from);

    input.direction = direction;
    input.activate = activate;

    if direction.is_some() || activate {
        focus_visible.0 = true;
    }
}

fn handle_directional_navigation(
    input: Res<DirectionalInput>,
    mut navigator: AutoDirectionalNavigator,
) {
    if let Some(direction) = input.direction {
        let _ = navigator.navigate(direction);
    }
}

/// Triggers the focused button for gamepad activation.
fn gamepad_activate_focused(
    input: Res<DirectionalInput>,
    focus: Res<InputFocus>,
    buttons: Query<(), With<Button>>,
    mut commands: Commands,
) {
    if !input.activate {
        return;
    }
    let Some(entity) = focus.get() else { return };
    if buttons.contains(entity) {
        commands.trigger(Activate { entity });
    }
}

/// Hides the focus ring when the user clicks with the mouse.
fn hide_focus_on_mouse_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut focus_visible: ResMut<InputFocusVisible>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        focus_visible.0 = false;
    }
}

/// Updates the `Outline` of focusable entities to show/hide the focus ring.
/// When gamepad navigation is disabled, clears any lingering focus rings.
fn update_focus_ring(
    focus: Res<InputFocus>,
    focus_visible: Res<InputFocusVisible>,
    settings: Res<GameSettings>,
    mut outlines: Query<(Entity, &mut Outline)>,
) {
    let show_for = if settings.gamepad_navigation && focus_visible.0 {
        focus.get()
    } else {
        None
    };

    for (entity, mut outline) in &mut outlines {
        if Some(entity) == show_for {
            outline.color = theme::colors::FOCUS_RING;
        } else if outline.color == theme::colors::FOCUS_RING {
            outline.color = Color::NONE;
        }
    }
}
