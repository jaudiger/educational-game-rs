use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::{
    Checkbox, RadioButton, RadioGroup, Slider, SliderRange, SliderStep, SliderThumb, SliderValue,
};

use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

use super::{CheckboxMark, RadioMark};

/// Returns a styled horizontal slider bundle with track + thumb.
///
/// The caller should attach `.observe(slider_self_update)` and their own
/// persistence observer for `ValueChange<f32>`.
pub fn slider(min: f32, max: f32, value: f32, step: f32) -> impl Bundle {
    (
        Slider::default(),
        SliderValue(value),
        SliderRange::new(min, max),
        SliderStep(step),
        AutoDirectionalNavigation::default(),
        TabIndex(0),
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
        Node {
            width: theme::scaled(theme::sizes::SLIDER_WIDTH),
            height: theme::scaled(theme::sizes::SLIDER_HEIGHT),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            // Track background
            (
                Node {
                    width: percent(100.0),
                    height: theme::scaled(theme::sizes::SLIDER_TRACK_HEIGHT),
                    border_radius: BorderRadius::all(theme::scaled(
                        theme::sizes::SLIDER_TRACK_HEIGHT / 2.0
                    )),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(theme::colors::TOGGLE_INACTIVE),
            ),
            // Thumb (absolutely positioned; left is set by update_slider_thumb_position)
            (
                SliderThumb,
                Node {
                    width: theme::scaled(theme::sizes::SLIDER_THUMB_SIZE),
                    height: theme::scaled(theme::sizes::SLIDER_THUMB_SIZE),
                    border_radius: BorderRadius::all(theme::scaled(
                        theme::sizes::SLIDER_THUMB_SIZE / 2.0
                    )),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(theme::colors::PRIMARY),
            ),
        ],
    )
}

/// Returns a styled checkbox bundle with a label.
///
/// **Does not include `Checked`**. The caller must conditionally insert it:
/// ```ignore
/// let mut cmd = parent.spawn(checkbox("label", true, window));
/// if checked { cmd.insert(Checked); }
/// ```
///
/// The caller should also attach `.insert(observe(checkbox_self_update))` and
/// their own persistence observer for `ValueChange<bool>`.
pub fn checkbox(label: &str, mark_visible: bool, window: Entity) -> impl Bundle + use<> {
    let mark_visibility = if mark_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    (
        Checkbox,
        AutoDirectionalNavigation::default(),
        TabIndex(0),
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        },
        children![
            // Checkbox box
            (
                Node {
                    width: theme::scaled(theme::sizes::CHECKBOX_SIZE),
                    height: theme::scaled(theme::sizes::CHECKBOX_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: px(2.0).all(),
                    border_radius: BorderRadius::all(theme::scaled(6.0)),
                    ..default()
                },
                BackgroundColor(theme::colors::CARD_BG),
                BorderColor::all(theme::colors::INPUT_BORDER),
                // Check mark (inner square)
                children![(
                    Node {
                        width: theme::scaled(theme::sizes::CHECKBOX_MARK_SIZE),
                        height: theme::scaled(theme::sizes::CHECKBOX_MARK_SIZE),
                        border_radius: BorderRadius::all(theme::scaled(3.0)),
                        ..default()
                    },
                    BackgroundColor(theme::colors::PRIMARY),
                    mark_visibility,
                    CheckboxMark,
                )],
            ),
            // Label
            (
                Text::new(label),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ),
        ],
    )
}

/// Returns a styled radio group bundle (flex row).
///
/// The caller adds radio button children via `Children::spawn(SpawnWith(...))`
/// and attaches an observer for `ValueChange<Entity>` to handle selection.
pub fn radio_group() -> impl Bundle {
    (
        RadioGroup,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
    )
}

/// Returns a styled radio button bundle with a circle indicator + label.
///
/// **Does not include `Checked`**. The caller must conditionally insert it:
/// ```ignore
/// let mut cmd = parent.spawn(radio_button("label", true, window));
/// if checked { cmd.insert(Checked); }
/// ```
///
/// Must be spawned as a child of a `RadioGroup`.
pub fn radio_button(label: &str, mark_visible: bool, window: Entity) -> impl Bundle + use<> {
    let mark_visibility = if mark_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    (
        RadioButton,
        AutoDirectionalNavigation::default(),
        TabIndex(0),
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        },
        children![
            // Outer circle
            (
                Node {
                    width: theme::scaled(theme::sizes::RADIO_SIZE),
                    height: theme::scaled(theme::sizes::RADIO_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: px(2.0).all(),
                    border_radius: BorderRadius::all(theme::scaled(theme::sizes::RADIO_SIZE / 2.0)),
                    ..default()
                },
                BackgroundColor(theme::colors::CARD_BG),
                BorderColor::all(theme::colors::INPUT_BORDER),
                // Inner dot
                children![(
                    Node {
                        width: theme::scaled(theme::sizes::RADIO_MARK_SIZE),
                        height: theme::scaled(theme::sizes::RADIO_MARK_SIZE),
                        border_radius: BorderRadius::all(theme::scaled(
                            theme::sizes::RADIO_MARK_SIZE / 2.0
                        )),
                        ..default()
                    },
                    BackgroundColor(theme::colors::PRIMARY),
                    mark_visibility,
                    RadioMark,
                )],
            ),
            // Label
            (
                Text::new(label),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ),
        ],
    )
}

/// Returns a styled radio button bundle with muted styling (for unavailable options).
///
/// **Does not include `Checked`**. The caller must conditionally insert it.
///
/// Must be spawned as a child of a `RadioGroup`.
pub fn radio_button_muted(
    label: &str,
    suffix: &str,
    mark_visible: bool,
    window: Entity,
) -> impl Bundle + use<> {
    let label_text = format!("{label} {suffix}");
    let mark_visibility = if mark_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    (
        RadioButton,
        AutoDirectionalNavigation::default(),
        TabIndex(0),
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        },
        children![
            // Outer circle
            (
                Node {
                    width: theme::scaled(theme::sizes::RADIO_SIZE),
                    height: theme::scaled(theme::sizes::RADIO_SIZE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: px(2.0).all(),
                    border_radius: BorderRadius::all(theme::scaled(theme::sizes::RADIO_SIZE / 2.0)),
                    ..default()
                },
                BackgroundColor(theme::colors::CARD_BG),
                BorderColor::all(theme::colors::INPUT_BORDER),
                children![(
                    Node {
                        width: theme::scaled(theme::sizes::RADIO_MARK_SIZE),
                        height: theme::scaled(theme::sizes::RADIO_MARK_SIZE),
                        border_radius: BorderRadius::all(theme::scaled(
                            theme::sizes::RADIO_MARK_SIZE / 2.0
                        )),
                        ..default()
                    },
                    BackgroundColor(theme::colors::PRIMARY),
                    mark_visibility,
                    RadioMark,
                )],
            ),
            // Muted label
            (
                Text::new(label_text),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_MUTED),
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ),
        ],
    )
}

/// Returns a stacked fraction layout (numerator / bar / denominator).
///
/// Used inline within text rows to render mathematical fractions in school
/// notation. The digit font size is reduced to 80 % of the given `font_size`
/// so the fraction integrates well alongside plain text.
pub fn stacked_fraction(
    numerator: u32,
    denominator: u32,
    font_size: f32,
    numerator_color: Color,
    denominator_color: Color,
    window: Entity,
) -> impl Bundle + use<> {
    let fraction_font = font_size * 0.8;
    let bar_color = if numerator_color == denominator_color {
        numerator_color
    } else {
        theme::colors::TEXT_DARK
    };
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(theme::scaled(2.0)),
            ..default()
        },
        children![
            // Numerator
            (
                Text::new(numerator.to_string()),
                TextFont {
                    font_size: fraction_font,
                    ..default()
                },
                TextColor(numerator_color),
                DesignFontSize {
                    size: fraction_font,
                    window,
                },
            ),
            // Fraction bar
            (
                Node {
                    height: theme::scaled(2.0),
                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                BackgroundColor(bar_color),
            ),
            // Denominator
            (
                Text::new(denominator.to_string()),
                TextFont {
                    font_size: fraction_font,
                    ..default()
                },
                TextColor(denominator_color),
                DesignFontSize {
                    size: fraction_font,
                    window,
                },
            ),
        ],
    )
}
