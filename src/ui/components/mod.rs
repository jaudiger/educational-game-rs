mod buttons;
mod forms;
mod overlays;

pub use buttons::*;
pub use forms::*;
pub use overlays::*;

use bevy::input_focus::tab_navigation::TabGroup;
use bevy::prelude::*;

use super::theme;

/// Marks the inner check mark of a styled checkbox.
#[derive(Component, Reflect)]
pub struct CheckboxMark;

/// Marks the inner dot of a styled radio button.
#[derive(Component, Reflect)]
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
/// The entity must have `Interaction` (automatic for `Button`,
/// add `Interaction::None` manually for plain nodes).
#[derive(Component, Reflect)]
pub struct HoverTooltip {
    pub message: String,
    pub window: Entity,
}

/// Marker for the tooltip entity spawned by the hover tooltip system.
#[derive(Component, Reflect)]
pub struct HoverTooltipPopover;

/// Marks a confirmation popover's confirm button.
#[derive(Component, Reflect)]
pub struct PopoverConfirmButton;

/// Marks a confirmation popover's cancel button.
#[derive(Component, Reflect)]
pub struct PopoverCancelButton;

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
