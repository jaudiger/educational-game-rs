//! Shared helper for rendering a numeric keypad (digits 0-9, delete, validate).

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use crate::ui::components::icon_button;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Marker for a digit button (0-9).
#[derive(Component, Reflect)]
pub struct KeypadDigitButton(pub u8);

/// Marker for the delete/backspace button.
#[derive(Component, Reflect)]
pub struct KeypadDeleteButton;

/// Marker for the validate/submit button.
#[derive(Component, Reflect)]
pub struct KeypadValidateButton;

/// Text node that displays the current keypad value.
#[derive(Component, Reflect)]
pub struct KeypadDisplay;

/// Width of a single key in the keypad.
const KEY_SIZE: f32 = 60.0;
/// Gap between keys.
const KEY_GAP: f32 = 6.0;
/// Border radius for keys.
const KEY_RADIUS: f32 = 8.0;

/// Returns a numeric keypad bundle (display + keypad grid).
///
/// Layout:
/// - Display row (shows current input)
/// - Row: 1 2 3
/// - Row: 4 5 6
/// - Row: 7 8 9
/// - Row: Del 0 OK
pub fn numeric_keypad(current_value: &str, window: Entity) -> impl Bundle {
    let value_owned = current_value.to_owned();
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: theme::scaled(KEY_GAP),
            margin: theme::scaled(theme::spacing::LARGE).top(),
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            // Display
            parent.spawn(display_node(&value_owned, window));

            // Digit rows (first row gets AutoFocus on digit 1)
            parent.spawn(digit_row_with_auto_focus(&[1u8, 2, 3], window));
            for row_digits in &[[4u8, 5, 6], [7, 8, 9]] {
                parent.spawn(digit_row(row_digits, window));
            }

            // Bottom row: Delete / 0 / Validate
            parent.spawn(bottom_row(window));
        })),
    )
}

fn display_node(value: &str, window: Entity) -> impl Bundle {
    (
        Node {
            width: theme::scaled(KEY_SIZE.mul_add(3.0, KEY_GAP * 2.0)),
            height: theme::scaled(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: px(2.0).all(),
            border_radius: BorderRadius::all(theme::scaled(KEY_RADIUS)),
            margin: theme::scaled(theme::spacing::SMALL).bottom(),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(theme::colors::INPUT_BORDER),
        children![(
            Text::new(value),
            TextFont {
                font_size: theme::fonts::HEADING,
                ..default()
            },
            TextColor(theme::colors::TEXT_DARK),
            KeypadDisplay,
            DesignFontSize {
                size: theme::fonts::HEADING,
                window,
            },
        )],
    )
}

fn digit_row(digits: &[u8; 3], window: Entity) -> impl Bundle {
    let d = *digits;
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(KEY_GAP),
            ..default()
        },
        Children::spawn(SpawnIter(d.into_iter().map(move |digit| {
            (
                key_button(&digit.to_string(), theme::colors::SECONDARY, window),
                KeypadDigitButton(digit),
            )
        }))),
    )
}

fn digit_row_with_auto_focus(digits: &[u8; 3], window: Entity) -> impl Bundle {
    let d = *digits;
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(KEY_GAP),
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for (i, digit) in d.into_iter().enumerate() {
                let mut cmd = parent.spawn((
                    key_button(&digit.to_string(), theme::colors::SECONDARY, window),
                    KeypadDigitButton(digit),
                ));
                if i == 0 {
                    cmd.insert(AutoFocus);
                }
            }
        })),
    )
}

fn bottom_row(window: Entity) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(KEY_GAP),
            ..default()
        },
        children![
            (
                key_button("⌫", theme::colors::ERROR, window),
                KeypadDeleteButton
            ),
            (
                key_button("0", theme::colors::SECONDARY, window),
                KeypadDigitButton(0)
            ),
            (
                key_button("OK", theme::colors::SUCCESS, window),
                KeypadValidateButton
            ),
        ],
    )
}

/// Returns a single key button bundle.
fn key_button(label: &str, bg: Color, window: Entity) -> impl Bundle + use<> {
    icon_button(
        KEY_SIZE,
        KEY_RADIUS,
        label,
        theme::fonts::BUTTON,
        bg,
        theme::colors::TEXT_LIGHT,
        window,
    )
}
