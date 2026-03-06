use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::theme;
use super::theme::DesignFontSize;

/// Stores the state of a text input field.
///
/// Attach to the outer `Button` entity (the clickable input box).
/// The shared systems handle focus management, keyboard input, border
/// colour changes, and cursor display automatically.
#[derive(Component, Reflect)]
pub struct TextInputState {
    pub text: String,
    pub focused: bool,
    pub max_length: usize,
}

impl TextInputState {
    pub const fn new(max_length: usize) -> Self {
        Self {
            text: String::new(),
            focused: false,
            max_length,
        }
    }

    #[must_use]
    pub const fn focused(mut self) -> Self {
        self.focused = true;
        self
    }
}

/// Marker for the `Text` child entity that displays the input content
/// (with a trailing `|` cursor when focused).
#[derive(Component, Reflect)]
pub struct TextInputDisplay;

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_text_input_focus,
                handle_text_input_keyboard,
                update_text_input_display,
            )
                .chain(),
        );
    }
}

fn handle_text_input_focus(
    mut query: Query<(&Interaction, &mut TextInputState, &mut BorderColor), Changed<Interaction>>,
) {
    for (interaction, mut state, mut border) in &mut query {
        if *interaction == Interaction::Pressed && !state.focused {
            state.focused = true;
            border.set_all(theme::colors::PRIMARY);
        }
    }
}

fn handle_text_input_keyboard(
    mut query: Query<(&mut TextInputState, &mut BorderColor)>,
    mut keyboard_events: MessageReader<KeyboardInput>,
) {
    let events: Vec<_> = keyboard_events.read().collect();

    for (mut state, mut border) in &mut query {
        if !state.focused {
            continue;
        }

        for event in &events {
            if event.state != ButtonState::Pressed {
                continue;
            }

            match &event.logical_key {
                Key::Backspace => {
                    state.text.pop();
                }
                Key::Escape => {
                    state.focused = false;
                    border.set_all(theme::colors::INPUT_BORDER);
                }
                Key::Character(c) => {
                    for ch in c.chars() {
                        if !ch.is_control() && state.text.len() < state.max_length {
                            state.text.push(ch);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn update_text_input_display(
    input_query: Query<(&TextInputState, &Children)>,
    mut text_query: Query<&mut Text, With<TextInputDisplay>>,
) {
    for (state, children) in &input_query {
        for child in children {
            if let Ok(mut text) = text_query.get_mut(*child) {
                let display = if state.text.is_empty() && state.focused {
                    "|".to_owned()
                } else if state.focused {
                    format!("{}|", state.text)
                } else {
                    state.text.clone()
                };
                **text = display;
            }
        }
    }
}

/// Returns a text input bundle (a clickable `Button` with a `Text` child).
///
/// The box responds to clicks (focus) and keyboard events (typing,
/// backspace, escape) via the shared systems registered by
/// [`TextInputPlugin`].
pub fn text_input(width: f32, state: TextInputState, window: Entity) -> impl Bundle {
    let border_color = if state.focused {
        theme::colors::PRIMARY
    } else {
        theme::colors::INPUT_BORDER
    };
    let initial_text = if state.focused { "|" } else { "" };

    (
        Button,
        Node {
            width: theme::scaled(width),
            height: theme::scaled(theme::sizes::INPUT_FIELD_HEIGHT),
            align_items: AlignItems::Center,
            padding: theme::scaled(theme::spacing::SMALL).horizontal(),
            border: px(2.0).all(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        BackgroundColor(theme::colors::INPUT_BG),
        BorderColor::all(border_color),
        state,
        children![(
            Text::new(initial_text),
            TextFont {
                font_size: theme::fonts::BODY,
                ..default()
            },
            TextColor(theme::colors::TEXT_DARK),
            TextInputDisplay,
            DesignFontSize {
                size: theme::fonts::BODY,
                window,
            },
        )],
    )
}
