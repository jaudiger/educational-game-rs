use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;

use crate::ui::animation::AnimatedButton;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Returns the common button components shared by all interactive buttons.
///
/// Includes `Button`, `BackgroundColor`, `AnimatedButton`, focus navigation
/// (`AutoDirectionalNavigation`, `TabIndex(0)`), and focus ring `Outline`.
///
/// Does **not** include `Node` (layout) or children (text/icons). The caller
/// provides those. Compose via tuple:
/// ```ignore
/// parent.spawn((button_base(color), Node { ... }, children![...], MyMarker));
/// ```
pub fn button_base(bg_color: Color) -> impl Bundle {
    (
        Button,
        BackgroundColor(bg_color),
        AnimatedButton,
        AutoDirectionalNavigation::default(),
        TabIndex(0),
        Outline::new(
            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
            Color::NONE,
        ),
    )
}

/// Returns a compact action button for dialogs (popovers, creation forms).
///
/// Wraps `button_base` with a `Node` (min 120px wide, `BUTTON_HEIGHT` tall) and
/// centered text at `FONT_SIZE_BUTTON_SMALL`. The button grows to fit its label.
pub fn action_button(
    label: &str,
    bg_color: Color,
    text_color: Color,
    window: Entity,
) -> impl Bundle + use<> {
    (
        button_base(bg_color),
        Node {
            min_width: theme::scaled(120.0),
            height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
            padding: UiRect::axes(theme::scaled(theme::spacing::SMALL), theme::scaled(0.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        children![(
            Text::new(label),
            TextFont {
                font_size: theme::fonts::BUTTON_SMALL,
                ..default()
            },
            TextColor(text_color),
            TextLayout::new_with_justify(Justify::Center),
            DesignFontSize {
                size: theme::fonts::BUTTON_SMALL,
                window,
            },
        )],
    )
}

/// Returns a square icon-style button with centered text.
///
/// Wraps `button_base` with a square `Node` (`size` by `size`, given `radius`)
/// and a single text child.
pub fn icon_button(
    size: f32,
    radius: f32,
    label: &str,
    font_size: f32,
    bg_color: Color,
    text_color: Color,
    window: Entity,
) -> impl Bundle + use<> {
    (
        button_base(bg_color),
        Node {
            width: theme::scaled(size),
            height: theme::scaled(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(theme::scaled(radius)),
            ..default()
        },
        children![(
            Text::new(label),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(text_color),
            TextLayout::new_with_justify(Justify::Center),
            DesignFontSize {
                size: font_size,
                window,
            },
        )],
    )
}

/// Returns a standard button bundle (min `width` by `BUTTON_HEIGHT`, rounded corners).
///
/// The button uses `min_width` so it grows to fit its label when the text is
/// longer than the given minimum. Use inside `children![]` or `.spawn(...)`.
/// The caller can add marker components by wrapping in a tuple:
/// `(standard_button("Play", color, scaled(300.0), w), MyMarker)`.
pub fn standard_button(
    label: &str,
    bg_color: Color,
    min_width: Val,
    window: Entity,
) -> impl Bundle + use<> {
    (
        button_base(bg_color),
        Node {
            min_width,
            height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
            padding: UiRect::axes(
                theme::scaled(theme::sizes::BUTTON_PADDING),
                theme::scaled(0.0),
            ),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        children![(
            Text::new(label),
            TextFont {
                font_size: theme::fonts::BUTTON,
                ..default()
            },
            TextColor(theme::colors::TEXT_LIGHT),
            TextLayout::new_with_justify(Justify::Center),
            DesignFontSize {
                size: theme::fonts::BUTTON,
                window,
            },
        )],
    )
}

/// Returns a toggle button bundle (160px x `BUTTON_HEIGHT`) with active/inactive styling.
///
/// - Active: `COLOR_PRIMARY` background, light text
/// - Inactive: `COLOR_TOGGLE_INACTIVE` background, dark text
pub fn toggle_button(label: &str, active: bool, window: Entity) -> impl Bundle + use<> {
    let bg = if active {
        theme::colors::PRIMARY
    } else {
        theme::colors::TOGGLE_INACTIVE
    };
    let text_color = if active {
        theme::colors::TEXT_LIGHT
    } else {
        theme::colors::TEXT_DARK
    };
    (
        button_base(bg),
        Node {
            min_width: theme::scaled(160.0),
            height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
            padding: UiRect::axes(
                theme::scaled(theme::sizes::BUTTON_PADDING),
                theme::scaled(0.0),
            ),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS)),
            ..default()
        },
        children![(
            Text::new(label),
            TextFont {
                font_size: theme::fonts::BUTTON_SMALL,
                ..default()
            },
            TextColor(text_color),
            TextLayout::new_with_justify(Justify::Center),
            DesignFontSize {
                size: theme::fonts::BUTTON_SMALL,
                window,
            },
        )],
    )
}
