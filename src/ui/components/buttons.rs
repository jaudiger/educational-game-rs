use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::events::PointerLeave;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::{Activate, Button};

use crate::ui::animation::AnimatedButton;
use crate::ui::components::PointerHovered;
use crate::ui::theme::{self, ButtonPalette};

#[derive(Message, Clone, Copy)]
pub struct ButtonActivated(pub Entity);

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[require(bevy::picking::hover::Hovered, PointerHovered)]
pub struct ButtonVisualStyle(pub ButtonPalette);

pub struct ButtonActivationPlugin;

impl Plugin for ButtonActivationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ButtonActivated>()
            .add_observer(forward_button_activation)
            .add_observer(cancel_button_press_on_pointer_leave)
            .add_systems(Update, update_button_visuals);
    }
}

fn forward_button_activation(event: On<Activate>, mut messages: MessageWriter<ButtonActivated>) {
    messages.write(ButtonActivated(event.entity));
}

fn cancel_button_press_on_pointer_leave(
    event: On<PointerLeave>,
    buttons: Query<(Entity, Has<Pressed>), With<Button>>,
    mut commands: Commands,
) {
    if let Ok((button, true)) = buttons.get(event.entity) {
        commands.entity(button).remove::<Pressed>();
    }
}

#[allow(clippy::type_complexity)]
fn update_button_visuals(
    mut buttons: Query<(
        &PointerHovered,
        Has<Pressed>,
        &ButtonVisualStyle,
        &mut BackgroundColor,
        Option<&mut BorderColor>,
    )>,
) {
    for (hovered, pressed, style, mut background, border) in &mut buttons {
        let is_hovered = hovered.0;
        let background_color = style.0.background.resolve(is_hovered, pressed);
        if background.0 != background_color {
            background.0 = background_color;
        }
        if let (Some(border_states), Some(mut border)) = (style.0.border, border) {
            let border_color = border_states.resolve(is_hovered, pressed);
            if border.top != border_color
                || border.right != border_color
                || border.bottom != border_color
                || border.left != border_color
            {
                border.set_all(border_color);
            }
        }
    }
}

/// Returns the shared interactive button components with a semantic palette.
///
/// The caller supplies layout and children. The visual system owns background
/// and palette border colors for every interaction state.
pub fn button_base(palette: ButtonPalette) -> impl Bundle {
    (
        Button,
        BackgroundColor(palette.background.normal),
        ButtonVisualStyle(palette),
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

/// Returns a square icon-style button with centered text.
pub fn icon_button(
    size: f32,
    radius: f32,
    label: &str,
    font_size: f32,
    palette: ButtonPalette,
    window: Entity,
) -> impl Bundle + use<> {
    (
        button_base(palette),
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
            TextColor(palette.foreground),
            TextLayout::justify(Justify::Center),
        )],
    )
}

/// Returns a standard button whose width grows to fit its label.
pub fn standard_button(
    label: &str,
    palette: ButtonPalette,
    min_width: Val,
    window: Entity,
) -> impl Bundle + use<> {
    (
        button_base(palette),
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
            TextColor(palette.foreground),
            TextLayout::justify(Justify::Center),
        )],
    )
}

#[cfg(not(feature = "demo"))]
pub mod production;
#[cfg(not(feature = "demo"))]
pub use production::*;
