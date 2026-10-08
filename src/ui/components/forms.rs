use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::scene::{Scene, SceneComponent, bsn};
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::{
    Checkbox, RadioButton, RadioGroup, Slider as SliderWidget, SliderRange, SliderStep,
    SliderThumb, SliderValue,
};

use crate::ui::theme;

use super::{CheckboxMark, RadioMark};

#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(StyledSliderProps)]
struct StyledSlider;

#[derive(Default)]
struct StyledSliderProps {
    min: f32,
    max: f32,
    value: f32,
    step: f32,
}

/// Returns a styled horizontal slider scene with a track and thumb.
///
/// The caller should attach `.observe(slider_self_update)` and its persistence
/// observer for `ValueChange<f32>` to the spawned entity.
pub fn slider_scene(min: f32, max: f32, value: f32, step: f32) -> impl Scene {
    bsn! {
        @StyledSlider {
            @min: {min},
            @max: {max},
            @value: {value},
            @step: {step},
        }
    }
}

impl StyledSlider {
    fn scene(props: StyledSliderProps) -> impl Scene {
        bsn! {
            SliderWidget::default()
            SliderValue({props.value})
            SliderRange::new(props.min, props.max)
            SliderStep({props.step})
            AutoDirectionalNavigation::default()
            TabIndex(0)
            Outline::new(
                Val::Px(theme::sizes::FOCUS_RING_WIDTH),
                Val::Px(theme::sizes::FOCUS_RING_OFFSET),
                Color::NONE,
            )
            Node {
                width: theme::scaled(theme::sizes::SLIDER_WIDTH),
                height: theme::scaled(theme::sizes::SLIDER_HEIGHT),
                align_items: AlignItems::Center,
            }
            Children [
                (
                    Node {
                        width: percent(100.0),
                        height: theme::scaled(theme::sizes::SLIDER_TRACK_HEIGHT),
                        border_radius: BorderRadius::all(theme::scaled(
                            theme::sizes::SLIDER_TRACK_HEIGHT / 2.0
                        )),
                        position_type: PositionType::Absolute,
                    }
                    BackgroundColor(theme::colors::TOGGLE_INACTIVE)
                ),
                (
                    SliderThumb
                    Node {
                        width: theme::scaled(theme::sizes::SLIDER_THUMB_SIZE),
                        height: theme::scaled(theme::sizes::SLIDER_THUMB_SIZE),
                        border_radius: BorderRadius::all(theme::scaled(
                            theme::sizes::SLIDER_THUMB_SIZE / 2.0
                        )),
                        position_type: PositionType::Absolute,
                    }
                    BackgroundColor(theme::colors::PRIMARY)
                ),
            ]
        }
    }
}

#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(StyledCheckboxProps)]
struct StyledCheckbox;

struct StyledCheckboxProps {
    label: String,
    window: Entity,
}

impl Default for StyledCheckboxProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            window: Entity::PLACEHOLDER,
        }
    }
}

/// Returns a styled checkbox scene with a label.
///
/// The caller adds `Checked` when selected and attaches the widget and
/// persistence observers for `ValueChange<bool>`.
pub fn checkbox_scene(label: &str, window: Entity) -> impl Scene {
    bsn! {
        @StyledCheckbox {
            @label: {label.to_owned()},
            @window: {window},
        }
    }
}

impl StyledCheckbox {
    fn scene(props: StyledCheckboxProps) -> impl Scene {
        bsn! {
            Checkbox
            AutoDirectionalNavigation::default()
            TabIndex(0)
            Outline::new(
                Val::Px(theme::sizes::FOCUS_RING_WIDTH),
                Val::Px(theme::sizes::FOCUS_RING_OFFSET),
                Color::NONE,
            )
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: theme::scaled(theme::spacing::SMALL),
            }
            Children [
                (
                    Node {
                        width: theme::scaled(theme::sizes::CHECKBOX_SIZE),
                        height: theme::scaled(theme::sizes::CHECKBOX_SIZE),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: {px(2.0).all()},
                        border_radius: BorderRadius::all(theme::scaled(6.0)),
                    }
                    BackgroundColor(theme::colors::CARD_BG)
                    BorderColor::all(theme::colors::INPUT_BORDER)
                    Children [(
                        Node {
                            width: theme::scaled(theme::sizes::CHECKBOX_MARK_SIZE),
                            height: theme::scaled(theme::sizes::CHECKBOX_MARK_SIZE),
                            border_radius: BorderRadius::all(theme::scaled(3.0)),
                        }
                        BackgroundColor(theme::colors::PRIMARY)
                        Visibility::Hidden
                        CheckboxMark
                    )]
                ),
                (
                    Text({props.label})
                    theme::typography::text_scene(theme::fonts::BODY, props.window)
                    TextColor(theme::colors::TEXT_DARK)
                ),
            ]
        }
    }
}

/// Returns a styled radio group bundle.
///
/// The caller adds radio-button scenes and a `ValueChange<Entity>` observer.
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

#[derive(SceneComponent, Default, Clone, Reflect)]
#[scene(StyledRadioButtonProps)]
struct StyledRadioButton;

struct StyledRadioButtonProps {
    label: String,
    text_color: Color,
    window: Entity,
}

impl Default for StyledRadioButtonProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            text_color: Color::default(),
            window: Entity::PLACEHOLDER,
        }
    }
}

/// Returns a styled radio-button scene.
///
/// The caller adds `Checked` when selected and attaches its radio-group marker.
pub fn radio_button_scene(label: &str, window: Entity) -> impl Scene {
    styled_radio_button_scene(label.to_owned(), theme::colors::TEXT_DARK, window)
}

/// Returns a muted radio-button scene for unavailable options.
pub fn radio_button_muted_scene(label: &str, suffix: &str, window: Entity) -> impl Scene {
    styled_radio_button_scene(
        format!("{label} {suffix}"),
        theme::colors::TEXT_MUTED,
        window,
    )
}

fn styled_radio_button_scene(label: String, text_color: Color, window: Entity) -> impl Scene {
    bsn! {
        @StyledRadioButton {
            @label: {label},
            @text_color: {text_color},
            @window: {window},
        }
    }
}

impl StyledRadioButton {
    fn scene(props: StyledRadioButtonProps) -> impl Scene {
        bsn! {
            RadioButton
            AutoDirectionalNavigation::default()
            TabIndex(0)
            Outline::new(
                Val::Px(theme::sizes::FOCUS_RING_WIDTH),
                Val::Px(theme::sizes::FOCUS_RING_OFFSET),
                Color::NONE,
            )
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: theme::scaled(theme::spacing::SMALL),
            }
            Children [
                (
                    Node {
                        width: theme::scaled(theme::sizes::RADIO_SIZE),
                        height: theme::scaled(theme::sizes::RADIO_SIZE),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: {px(2.0).all()},
                        border_radius: BorderRadius::all(theme::scaled(
                            theme::sizes::RADIO_SIZE / 2.0
                        )),
                    }
                    BackgroundColor(theme::colors::CARD_BG)
                    BorderColor::all(theme::colors::INPUT_BORDER)
                    Children [(
                        Node {
                            width: theme::scaled(theme::sizes::RADIO_MARK_SIZE),
                            height: theme::scaled(theme::sizes::RADIO_MARK_SIZE),
                            border_radius: BorderRadius::all(theme::scaled(
                                theme::sizes::RADIO_MARK_SIZE / 2.0
                            )),
                        }
                        BackgroundColor(theme::colors::PRIMARY)
                        Visibility::Hidden
                        RadioMark
                    )]
                ),
                (
                    Text({props.label})
                    theme::typography::text_scene(theme::fonts::BODY, props.window)
                    TextColor({props.text_color})
                ),
            ]
        }
    }
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
                theme::typography::text(fraction_font, window),
                TextColor(numerator_color),
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
                theme::typography::text(fraction_font, window),
                TextColor(denominator_color),
            ),
        ],
    )
}
