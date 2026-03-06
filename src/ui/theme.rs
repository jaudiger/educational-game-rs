use bevy::prelude::*;
use bevy::ui_widgets::{Checkbox, RadioButton};

use super::components::{CheckboxMark, RadioMark};
use super::widget_styles;

/// Reference design vmin (min dimension of the 1280x720 default window).
/// All design-pixel constants are calibrated for this value.
const REFERENCE_VMIN: f32 = 720.0;

/// Converts a design-pixel value (calibrated for 720 px vmin) to a
/// viewport-relative [`Val`].  Bevy resolves `VMin` per camera/window,
/// so the primary window and teacher window each scale independently.
pub fn scaled(design_px: f32) -> Val {
    vmin(design_px / REFERENCE_VMIN * 100.0)
}

/// Stores the design-time font size and the target window for scaling.
///
/// Attach next to a [`TextFont`] component.  An observer scales the font
/// immediately on spawn; a system re-scales whenever the target window is
/// resized.
#[derive(Component, Reflect)]
pub struct DesignFontSize {
    pub size: f32,
    pub window: Entity,
}

/// Preloaded font handles shared across the entire UI.
#[derive(Resource, Reflect)]
pub struct GameFonts {
    pub regular: Handle<Font>,
}

/// Preloaded image handles shared across the entire UI.
#[derive(Resource, Reflect)]
pub struct GameImages {
    pub question_background: Handle<Image>,
}

/// Loads shared font and image assets, sets up the default-font override
/// observer, and registers widget update systems.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        let font_handle: Handle<Font> = asset_server.load("fonts/FiraMono-Regular.ttf");
        let question_bg: Handle<Image> = asset_server.load("question-background.png");
        app.insert_resource(GameFonts {
            regular: font_handle,
        });
        app.insert_resource(GameImages {
            question_background: question_bg,
        });
        app.add_observer(override_default_font);
        app.add_observer(scale_font_on_add);
        app.add_systems(
            Update,
            (
                widget_styles::update_checked_visuals::<Checkbox, CheckboxMark>,
                widget_styles::update_checked_visuals::<RadioButton, RadioMark>,
                widget_styles::update_slider_thumb_position,
                widget_styles::dismiss_tooltip_timer,
                widget_styles::handle_hover_tooltips,
                scale_fonts_on_window_resize,
            ),
        );
    }
}

fn override_default_font(
    on: On<Add, TextFont>,
    mut query: Query<&mut TextFont>,
    fonts: Res<GameFonts>,
) {
    if let Ok(mut text_font) = query.get_mut(on.event_target())
        && text_font.font.id() == Handle::<Font>::default().id()
    {
        text_font.font = fonts.regular.clone();
    }
}

/// Immediately scales a [`TextFont`] when a [`DesignFontSize`] is added,
/// preventing a one-frame flash at the unscaled size.
fn scale_font_on_add(
    trigger: On<Add, DesignFontSize>,
    mut query: Query<(&DesignFontSize, &mut TextFont)>,
    windows: Query<&Window>,
) {
    let entity = trigger.event_target();
    if let Ok((design, mut font)) = query.get_mut(entity)
        && let Ok(window) = windows.get(design.window)
    {
        let vmin_val = window.width().min(window.height());
        let scale = vmin_val / REFERENCE_VMIN;
        font.font_size = design.size * scale;
    }
}

/// Re-scales every [`DesignFontSize`] entity whose target window changed size.
fn scale_fonts_on_window_resize(
    changed_windows: Query<(Entity, &Window), Changed<Window>>,
    mut fonts: Query<(&DesignFontSize, &mut TextFont)>,
) {
    for (window_entity, window) in &changed_windows {
        let vmin_val = window.width().min(window.height());
        let scale = vmin_val / REFERENCE_VMIN;
        for (design, mut font) in &mut fonts {
            if design.window == window_entity {
                font.font_size = design.size * scale;
            }
        }
    }
}

pub mod colors {
    use bevy::prelude::Color;

    pub const PRIMARY: Color = Color::srgb(0.2, 0.5, 0.9);
    pub const PRIMARY_HOVER: Color = Color::srgb(0.3, 0.6, 1.0);
    pub const SECONDARY: Color = Color::srgb(0.95, 0.6, 0.1);
    pub const SUCCESS: Color = Color::srgb(0.2, 0.8, 0.3);
    pub const ERROR: Color = Color::srgb(0.9, 0.2, 0.2);
    pub const BACKGROUND: Color = Color::srgb(0.95, 0.95, 0.97);
    pub const CARD_BG: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const TEXT_DARK: Color = Color::srgb(0.15, 0.15, 0.2);
    pub const TEXT_LIGHT: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.55);
    pub const INPUT_BG: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const INPUT_BORDER: Color = Color::srgb(0.7, 0.7, 0.75);
    pub const TOGGLE_INACTIVE: Color = Color::srgb(0.8, 0.8, 0.85);
    pub const CARD_OVERLAY: Color = Color::srgba(1.0, 1.0, 1.0, 0.65);
    pub const CARD_BORDER: Color = Color::srgba(0.0, 0.0, 0.0, 0.18);
    pub const FOCUS_RING: Color = Color::srgb(1.0, 0.85, 0.0);
}

pub mod fonts {
    pub const HERO: f32 = 64.0;
    pub const TITLE: f32 = 48.0;
    pub const HEADING: f32 = 32.0;
    pub const BODY: f32 = 24.0;
    pub const SMALL: f32 = 18.0;
    pub const BUTTON: f32 = 28.0;
    pub const BUTTON_SMALL: f32 = 22.0;
}

pub mod spacing {
    pub const SMALL: f32 = 8.0;
    pub const MEDIUM: f32 = 16.0;
    pub const LARGE: f32 = 32.0;
    pub const XLARGE: f32 = 48.0;
}

pub mod animation {
    pub const BUTTON_HOVER_DURATION: f32 = 0.12;
    pub const BUTTON_PRESS_DURATION: f32 = 0.08;
    pub const SCREEN_ENTRY_DURATION: f32 = 0.3;
    pub const FEEDBACK_CORRECT_DURATION: f32 = 0.4;
    pub const FEEDBACK_INCORRECT_DURATION: f32 = 0.3;
    pub const BUTTON_HOVER_SCALE: f32 = 1.05;
    pub const BUTTON_PRESS_SCALE: f32 = 0.95;
    pub const SCREEN_ENTRY_START_Y: f32 = 20.0;
    pub const SCREEN_ENTRY_START_SCALE: f32 = 0.98;
    pub const FLOATING_AMPLITUDE: f32 = 3.0;
    pub const FLOATING_AMPLITUDE_MUTED: f32 = 1.5;
    pub const FLOATING_SPEED: f32 = 0.8;
    pub const FLOATING_PHASE_OFFSET: f32 = 1.2;
}

pub mod sizes {
    pub const BUTTON_WIDTH: f32 = 300.0;
    pub const BUTTON_HEIGHT: f32 = 60.0;
    pub const BUTTON_PADDING: f32 = 16.0;
    pub const BUTTON_BORDER_RADIUS: f32 = 12.0;
    pub const CARD_PADDING: f32 = 24.0;
    pub const CARD_BORDER_RADIUS: f32 = 16.0;
    pub const CARD_MIN_WIDTH: f32 = 250.0;
    pub const CARD_MIN_HEIGHT: f32 = 150.0;
    pub const INPUT_FIELD_HEIGHT: f32 = 50.0;
    pub const SLIDER_WIDTH: f32 = 300.0;
    pub const SLIDER_HEIGHT: f32 = 24.0;
    pub const SLIDER_TRACK_HEIGHT: f32 = 6.0;
    pub const SLIDER_THUMB_SIZE: f32 = 24.0;
    pub const CHECKBOX_SIZE: f32 = 32.0;
    pub const CHECKBOX_MARK_SIZE: f32 = 18.0;
    pub const RADIO_SIZE: f32 = 32.0;
    pub const RADIO_MARK_SIZE: f32 = 16.0;
    pub const POPOVER_MIN_WIDTH: f32 = 280.0;
    pub const TOOLTIP_BORDER_RADIUS: f32 = 8.0;
    pub const SKY_CARD_WIDTH: f32 = 350.0;
    pub const SKY_CARD_HEIGHT: f32 = 90.0;
    pub const SKY_CARD_BORDER_RADIUS: f32 = 20.0;
    pub const SKY_CARD_PADDING_H: f32 = 24.0;
    pub const SKY_CARD_PADDING_V: f32 = 16.0;
    pub const FOCUS_RING_WIDTH: f32 = 3.0;
    pub const FOCUS_RING_OFFSET: f32 = 2.0;
}
