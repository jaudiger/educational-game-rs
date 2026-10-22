use bevy::prelude::*;
use bevy::text::{FontSize, FontSource};
use bevy::ui_widgets::{Checkbox, RadioButton};
use bevy::window::WindowResized;

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
#[derive(Component, Reflect, Clone)]
pub struct DesignFontSize {
    pub size: f32,
    pub window: Entity,
}

impl Default for DesignFontSize {
    fn default() -> Self {
        Self {
            size: 0.0,
            window: Entity::PLACEHOLDER,
        }
    }
}

/// Typography components shared by reusable UI constructors.
///
/// This keeps the current design-pixel scaling contract in one place while
/// leaving room for richer Bevy text components without changing every caller.
pub mod typography {
    use bevy::prelude::*;
    use bevy::scene::{Scene, bsn};

    use super::DesignFontSize;

    /// Returns the current game typography bundle for a target window.
    ///
    /// This keeps the font unspecified so the theme observer supplies the branded game font.
    pub fn text(size: f32, window: Entity) -> impl Bundle + use<> {
        (
            TextFont::from_font_size(FontSize::Px(size)),
            DesignFontSize { size, window },
        )
    }

    /// Returns the current game typography scene for a target window.
    pub fn text_scene(size: f32, window: Entity) -> impl Scene {
        bsn! {
            TextFont {
                font_size: FontSize::Px(size),
            }
            DesignFontSize {
                size: {size},
                window: {window},
            }
        }
    }
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
        let question_bg: Handle<Image> = asset_server.load("ui/question-background.png");
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
    on: On<Add<TextFont>>,
    mut query: Query<&mut TextFont>,
    fonts: Res<GameFonts>,
) {
    if let Ok(mut text_font) = query.get_mut(on.event_target())
        && matches!(text_font.font, FontSource::Handle(ref handle) if handle == &Handle::<Font>::default())
    {
        text_font.font = FontSource::Handle(fonts.regular.clone());
    }
}

/// Immediately scales a [`TextFont`] when a [`DesignFontSize`] is added,
/// preventing a one-frame flash at the unscaled size.
fn scale_font_on_add(
    trigger: On<Add<DesignFontSize>>,
    mut query: Query<(&DesignFontSize, &mut TextFont)>,
    windows: Query<&Window>,
) {
    let entity = trigger.event_target();
    if let Ok((design, mut font)) = query.get_mut(entity)
        && let Ok(window) = windows.get(design.window)
    {
        let vmin_val = window.width().min(window.height());
        let scale = vmin_val / REFERENCE_VMIN;
        font.font_size = FontSize::Px(design.size * scale);
    }
}

/// Re-scales every [`DesignFontSize`] entity whose target window changed size.
fn scale_fonts_on_window_resize(
    mut resized_windows: MessageReader<WindowResized>,
    mut fonts: Query<(&DesignFontSize, &mut TextFont)>,
) {
    for resized_window in resized_windows.read() {
        let vmin_val = resized_window.width.min(resized_window.height);
        let scale = vmin_val / REFERENCE_VMIN;
        for (design, mut font) in &mut fonts {
            if design.window == resized_window.window {
                font.font_size = FontSize::Px(design.size * scale);
            }
        }
    }
}

pub mod layout {
    use bevy::prelude::Val;

    pub const FILL_PARENT: Val = Val::Percent(100.0);
}

pub mod colors {
    use bevy::prelude::Color;

    pub const PRIMARY: Color = Color::srgb(0.2, 0.5, 0.9);
    pub const SECONDARY: Color = Color::srgb(0.95, 0.6, 0.1);
    pub const BACK_BUTTON: Color = PRIMARY;
    pub const ACTION_PLAY: Color = Color::srgb(0.3333, 0.5961, 0.8863);
    pub const ACTION_CHOOSE: Color = Color::srgb(0.2588, 0.6549, 0.5490);
    pub const SUCCESS: Color = Color::srgb(0.2, 0.8, 0.3);
    pub const ERROR: Color = Color::srgb(0.9, 0.2, 0.2);
    pub const BACKGROUND: Color = Color::srgb(0.95, 0.95, 0.97);
    pub const CARD_BG: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const LIST_SURFACE: Color = Color::srgb(0.9686, 0.9804, 0.9922);
    pub const ACTIVITY_BADGE: Color = Color::srgb(0.84, 0.91, 0.98);
    pub const TEXT_DARK: Color = Color::srgb(0.15, 0.15, 0.2);
    pub const NAVIGATION_INK: Color = Color::srgb(0.0588, 0.0902, 0.1647);
    pub const TEXT_LIGHT: Color = Color::srgb(1.0, 1.0, 1.0);
    pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.55);
    pub const INPUT_BORDER: Color = Color::srgb(0.7, 0.7, 0.75);
    pub const TOGGLE_INACTIVE: Color = Color::srgb(0.8, 0.8, 0.85);
    pub const CARD_OVERLAY: Color = Color::srgba(1.0, 1.0, 1.0, 0.75);
    pub const CARD_BORDER: Color = Color::srgba(0.0, 0.0, 0.0, 0.18);
    pub const FOCUS_RING: Color = Color::srgb(1.0, 0.85, 0.0);
    pub const PASTEL_BLUE: Color = Color::srgb(0.54, 0.77, 0.93);
    pub const PASTEL_PEACH: Color = Color::srgb(1.0, 0.74, 0.55);
    pub const PASTEL_LAVENDER: Color = Color::srgb(0.73, 0.68, 0.91);
    pub const PASTEL_MINT: Color = Color::srgb(0.55, 0.82, 0.70);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct ButtonColorStates {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl ButtonColorStates {
    pub const fn new(normal: Color, hovered: Color, pressed: Color) -> Self {
        Self {
            normal,
            hovered,
            pressed,
        }
    }

    pub const fn flat(color: Color) -> Self {
        Self::new(color, color, color)
    }

    pub const fn resolve(self, is_hovered: bool, is_pressed: bool) -> Color {
        if is_pressed {
            self.pressed
        } else if is_hovered {
            self.hovered
        } else {
            self.normal
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct ButtonPalette {
    pub background: ButtonColorStates,
    pub foreground: Color,
    pub border: Option<ButtonColorStates>,
}

impl ButtonPalette {
    pub const fn new(background: ButtonColorStates, foreground: Color) -> Self {
        Self {
            background,
            foreground,
            border: None,
        }
    }

    pub const fn with_border(
        background: ButtonColorStates,
        foreground: Color,
        border: ButtonColorStates,
    ) -> Self {
        Self {
            background,
            foreground,
            border: Some(border),
        }
    }
}

pub mod button_styles {
    use super::{ButtonColorStates, ButtonPalette, colors};
    use bevy::prelude::Color;

    pub const BACK: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::flat(colors::BACK_BUTTON),
        colors::NAVIGATION_INK,
    );
    pub const ACTION_PRIMARY: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            colors::ACTION_PLAY,
            Color::srgb(0.3137, 0.5765, 0.8627),
            Color::srgb(0.3098, 0.5686, 0.8510),
        ),
        colors::TEXT_DARK,
    );
    pub const ACTION_SECONDARY: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            colors::ACTION_CHOOSE,
            Color::srgb(0.2392, 0.6235, 0.5255),
            Color::srgb(0.2314, 0.6157, 0.5176),
        ),
        colors::TEXT_DARK,
    );
    pub const ANSWER: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            Color::srgb(0.9569, 0.6510, 0.4235),
            Color::srgb(0.9176, 0.6039, 0.3725),
            Color::srgb(0.8706, 0.5451, 0.3059),
        ),
        colors::TEXT_DARK,
    );
    pub const NEUTRAL: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            Color::srgb(0.7882, 0.8118, 0.8471),
            Color::srgb(0.7255, 0.7608, 0.8118),
            Color::srgb(0.6627, 0.7098, 0.7647),
        ),
        colors::TEXT_DARK,
    );
    pub const DANGER: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            Color::srgb(0.9137, 0.5686, 0.5686),
            Color::srgb(0.8863, 0.5255, 0.5255),
            Color::srgb(0.8471, 0.4784, 0.4784),
        ),
        colors::TEXT_DARK,
    );
    pub const ACTIVITY_ITEM: ButtonPalette = ButtonPalette::with_border(
        ButtonColorStates::new(
            Color::srgb(0.8980, 0.9333, 0.9686),
            Color::srgb(0.6902, 0.8039, 0.9137),
            Color::srgb(0.5843, 0.7412, 0.8824),
        ),
        colors::TEXT_DARK,
        ButtonColorStates::new(
            Color::srgb(0.5608, 0.6588, 0.7490),
            Color::srgb(0.2588, 0.4196, 0.5686),
            Color::srgb(0.2039, 0.3529, 0.4863),
        ),
    );
    pub const ACTIVITY_ITEM_SELECTED: ButtonPalette = ButtonPalette::with_border(
        ACTION_PRIMARY.background,
        colors::TEXT_DARK,
        ButtonColorStates::new(
            Color::srgb(0.2588, 0.4196, 0.5686),
            Color::srgb(0.2039, 0.3529, 0.4863),
            Color::srgb(0.1569, 0.2706, 0.3725),
        ),
    );
    pub const ACTIVITY_CATEGORY: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::new(
            Color::srgb(0.88, 0.93, 0.99),
            Color::srgb(0.82, 0.90, 0.98),
            Color::srgb(0.76, 0.86, 0.97),
        ),
        colors::TEXT_DARK,
    );
    #[allow(dead_code)]
    pub const DISCLOSURE: ButtonPalette =
        ButtonPalette::new(ButtonColorStates::flat(Color::NONE), colors::TEXT_DARK);
    pub const UNAVAILABLE: ButtonPalette = ButtonPalette::new(
        ButtonColorStates::flat(colors::TOGGLE_INACTIVE),
        colors::TEXT_DARK,
    );
    pub const FRACTION_SLICE_COLORED: ButtonPalette = ACTION_PRIMARY;
    pub const FRACTION_SLICE_UNCOLORED: ButtonPalette = NEUTRAL;
}

pub mod fonts {
    pub const HERO: f32 = 64.0;
    pub const TITLE: f32 = 48.0;
    pub const HEADING: f32 = 32.0;
    pub const BODY: f32 = 24.0;
    pub const SMALL: f32 = 18.0;
    pub const BUTTON: f32 = 28.0;
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
    pub const SLIDER_WIDTH: f32 = 300.0;
    pub const SLIDER_HEIGHT: f32 = 24.0;
    pub const SLIDER_TRACK_HEIGHT: f32 = 6.0;
    pub const SLIDER_THUMB_SIZE: f32 = 24.0;
    pub const CHECKBOX_SIZE: f32 = 32.0;
    pub const CHECKBOX_MARK_SIZE: f32 = 18.0;
    pub const RADIO_SIZE: f32 = 32.0;
    pub const RADIO_MARK_SIZE: f32 = 16.0;
    pub const TOOLTIP_BORDER_RADIUS: f32 = 8.0;
    pub const SKY_CARD_WIDTH: f32 = 350.0;
    pub const SKY_CARD_HEIGHT: f32 = 90.0;
    pub const SKY_CARD_BORDER_RADIUS: f32 = 20.0;
    pub const SKY_CARD_PADDING_H: f32 = 24.0;
    pub const SKY_CARD_PADDING_V: f32 = 16.0;
    pub const FOCUS_RING_WIDTH: f32 = 3.0;
    pub const FOCUS_RING_OFFSET: f32 = 2.0;
}

#[cfg(not(feature = "demo"))]
pub mod production;
