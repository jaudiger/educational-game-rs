use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::scene::{Scene, SceneComponent, bsn};
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

/// Scene component for the fixed action button hierarchy.
#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ActionButtonProps)]
pub struct ActionButton;

pub struct ActionButtonProps {
    label: String,
    bg_color: Color,
    text_color: Color,
    window: Entity,
}

impl Default for ActionButtonProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            bg_color: Color::default(),
            text_color: Color::default(),
            window: Entity::PLACEHOLDER,
        }
    }
}

/// Returns the BSN scene for a compact action button.
///
/// The scene is applied to an empty entity so callers can compose markers on
/// the resulting button without changing its parent relationship.
pub fn action_button_scene(
    label: &str,
    bg_color: Color,
    text_color: Color,
    window: Entity,
) -> impl Scene {
    bsn! {
        @ActionButton {
            @label: {label.to_owned()},
            @bg_color: {bg_color},
            @text_color: {text_color},
            @window: {window},
        }
    }
}

impl ActionButton {
    fn scene(props: ActionButtonProps) -> impl Scene {
        bsn! {
            Button
            BackgroundColor({props.bg_color})
            AnimatedButton
            AutoDirectionalNavigation::default()
            TabIndex(0)
            Outline::new(
                Val::Px(theme::sizes::FOCUS_RING_WIDTH),
                Val::Px(theme::sizes::FOCUS_RING_OFFSET),
                Color::NONE,
            )
            Node {
                min_width: theme::scaled(120.0),
                height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
                padding: UiRect::axes(theme::scaled(theme::spacing::SMALL), theme::scaled(0.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                border_radius: {BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS))},
            }
            Children [(
                Text({props.label})
                TextFont {
                    font_size: FontSize::Px(theme::fonts::BUTTON_SMALL),
                }
                DesignFontSize {
                    size: theme::fonts::BUTTON_SMALL,
                    window: {props.window},
                }
                TextColor({props.text_color})
                TextLayout::justify(Justify::Center)
            )]
        }
    }
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
            theme::typography::text(font_size, window),
            TextColor(text_color),
            TextLayout::justify(Justify::Center),
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
            theme::typography::text(theme::fonts::BUTTON, window),
            TextColor(theme::colors::TEXT_LIGHT),
            TextLayout::justify(Justify::Center),
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
            theme::typography::text(theme::fonts::BUTTON_SMALL, window),
            TextColor(text_color),
            TextLayout::justify(Justify::Center),
        )],
    )
}
