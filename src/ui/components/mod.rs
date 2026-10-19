mod buttons;
mod forms;
mod menu_background;
mod overlays;
#[cfg(not(feature = "demo"))]
mod production;

pub use buttons::*;
pub use forms::*;
pub use menu_background::*;
pub use overlays::*;
#[cfg(not(feature = "demo"))]
pub use production::*;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use super::theme;

/// Marks the inner check mark of a styled checkbox.
#[derive(Component, Reflect, Clone, Default)]
pub struct CheckboxMark;

/// Marks the inner dot of a styled radio button.
#[derive(Component, Reflect, Clone, Default)]
pub struct RadioMark;

/// Default [`TooltipLifetime`]: 2-second auto-dismiss timer.
fn default_tooltip_lifetime() -> TooltipLifetime {
    TooltipLifetime(Timer::from_seconds(2.0, TimerMode::Once))
}

/// Marks a tooltip popover for auto-dismiss.
#[derive(Component, Reflect)]
#[require(TooltipLifetime = default_tooltip_lifetime())]
pub struct TooltipPopover;

/// Timer component for auto-dismiss of tooltips.
#[derive(Component, Reflect)]
pub struct TooltipLifetime(pub Timer);

/// Attach to any UI node to show a tooltip on hover.
/// The entity must track pointer hover, which is added automatically.
#[derive(Component, Reflect)]
#[require(bevy::picking::hover::Hovered)]
pub struct HoverTooltip {
    pub message: String,
    pub window: Entity,
}

/// Marker for the tooltip entity spawned by the hover tooltip system.
#[derive(Component, Reflect)]
pub struct HoverTooltipPopover;

/// Returns the standard full-screen root layout used by every screen.
///
/// Provides a centered column filling the entire viewport, with the game
/// background color and keyboard-navigation support (`TabGroup`).
/// Compose via tuple with `DespawnOnExit`, `children![]`, and optional markers:
/// ```ignore
/// commands.spawn((
///     screen_root(),
///     DespawnOnExit(AppState::Home),
///     children![...],
/// ));
/// ```
pub fn screen_root() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::XLARGE),
            ..default()
        },
        BackgroundColor(theme::colors::BACKGROUND),
        TabGroup::new(0),
    )
}

/// Stamps the shared card styling (1px border, white background, gray border color)
/// onto a caller-provided `Node`. Returns a tuple suitable for spawning.
pub fn card_node(mut node: Node) -> (Node, BackgroundColor, BorderColor) {
    node.border = px(1.0).all();
    (
        node,
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(theme::colors::INPUT_BORDER),
    )
}

pub fn list_separator_border() -> BorderColor {
    BorderColor::all(Color::srgba(0.35, 0.48, 0.67, 0.3))
}

pub fn disclosure_chevron_transform(expanded: bool) -> UiTransform {
    UiTransform::from_rotation(if expanded {
        Rot2::degrees(90.0)
    } else {
        Rot2::IDENTITY
    })
}

pub fn disclosure_chevron(expanded: bool, color: Color, window: Entity) -> impl Bundle + use<> {
    (
        Text::new(">"),
        theme::typography::text(theme::fonts::BODY, window),
        TextColor(color),
        TextLayout::justify(Justify::Center),
        Node {
            width: theme::scaled(20.0),
            height: theme::scaled(24.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        disclosure_chevron_transform(expanded),
    )
}

/// Keeps an operator with its following visual when the parent row wraps.
pub fn operator_visual_group<B: Bundle>(
    operator: &'static str,
    font_size: f32,
    color: Color,
    window: Entity,
    visual: B,
) -> impl Bundle + use<B> {
    (
        Node {
            flex_direction: FlexDirection::Row,
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        },
        children![
            (
                Text::new(operator),
                theme::typography::text(font_size, window),
                TextColor(color),
            ),
            visual,
        ],
    )
}
