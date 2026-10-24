use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::scene::{Scene, SceneComponent, bsn};
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::Button;

use crate::ui::animation::AnimatedButton;
use crate::ui::theme::{self, ButtonPalette};

use super::ButtonVisualStyle;

#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(ActionButtonProps)]
pub struct ActionButton;

pub struct ActionButtonProps {
    label: String,
    palette: ButtonPalette,
    window: Entity,
}

impl Default for ActionButtonProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            palette: theme::button_styles::ACTION_SECONDARY,
            window: Entity::PLACEHOLDER,
        }
    }
}

/// Returns a scene for a compact action button.
///
/// The scene applies to an empty entity so callers can compose markers without
/// changing the resulting button's parent relationship.
pub fn action_button_scene(label: &str, palette: ButtonPalette, window: Entity) -> impl Scene {
    bsn! {
        @ActionButton {
            @label: {label.to_owned()},
            @palette: {palette},
            @window: {window},
        }
    }
}

impl ActionButton {
    fn scene(props: ActionButtonProps) -> impl Scene {
        bsn! {
            Button
            BackgroundColor({props.palette.background.normal})
            ButtonVisualStyle({props.palette})
            AnimatedButton
            AutoDirectionalNavigation::default()
            TabIndex(0)
            Outline::new(
                Val::Px(theme::sizes::FOCUS_RING_WIDTH),
                Val::Px(theme::sizes::FOCUS_RING_OFFSET),
                Color::NONE,
            )
            Node {
                min_width: px(0.0),
                max_width: theme::layout::FILL_PARENT,
                min_height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
                height: Val::Auto,
                padding: UiRect::axes(
                    theme::scaled(theme::spacing::SMALL),
                    theme::scaled(theme::spacing::SMALL / 2.0),
                ),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: {BorderRadius::all(theme::scaled(theme::sizes::BUTTON_BORDER_RADIUS))},
            }
            Children [
                Text({props.label})
                @theme::typography::text_scene(theme::production::fonts::BUTTON_SMALL, props.window)
                TextColor({props.palette.foreground})
                TextLayout::justify(Justify::Center)
                Node {
                    width: theme::layout::FILL_PARENT,
                    min_width: px(0.0),
                }
            ]
        }
    }
}
