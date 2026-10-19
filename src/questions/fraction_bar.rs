//! Shared helper for rendering fraction bars (rows of rectangular slices).

use bevy::input_focus::AutoFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::Button;

use crate::ui::components::ButtonVisualStyle;
use crate::ui::theme;

/// Default total width of the bar in design pixels.
pub const BAR_WIDTH: f32 = 500.0;
/// Default height of each slice in design pixels.
pub const BAR_HEIGHT: f32 = 80.0;
/// Gap between slices in design pixels.
const SLICE_GAP: f32 = 3.0;
/// Border radius for the outermost corners in design pixels.
const CORNER_RADIUS: f32 = 6.0;

/// Background colour for an uncoloured slice.
pub const COLOR_UNCOLORED: Color = Color::srgb(0.73, 0.73, 0.78);
/// Border colour drawn around every slice.
const COLOR_SLICE_BORDER: Color = Color::srgb(0.60, 0.60, 0.65);
/// Border thickness around each slice in design pixels.
const SLICE_BORDER: f32 = 1.5;

/// Marker component placed on each interactive slice of a fraction bar.
#[derive(Component, Reflect)]
#[require(Button)]
pub struct FractionSlice {
    pub colored: bool,
}

/// Returns a fraction bar with optional interactive slices and custom dimensions.
///
/// The slice count and colored count determine its contents, while the bar color
/// applies to colored slices. Interactive slices receive focus and hover states.
/// The supplied dimensions are design pixels and scale with the target viewport.
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
pub fn fraction_bar(
    denominator: u32,
    colored_count: u32,
    bar_color: Color,
    interactive: bool,
    bar_width: f32,
    bar_height: f32,
) -> impl Bundle {
    let slice_width = if denominator > 0 {
        SLICE_GAP.mul_add(-(denominator.saturating_sub(1) as f32), bar_width) / denominator as f32
    } else {
        bar_width
    };

    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(SLICE_GAP),
            align_items: AlignItems::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for i in 0..denominator {
                let is_colored = i < colored_count;
                let palette = if interactive {
                    if is_colored {
                        theme::button_styles::FRACTION_SLICE_COLORED
                    } else {
                        theme::button_styles::FRACTION_SLICE_UNCOLORED
                    }
                } else {
                    theme::ButtonPalette::new(
                        theme::ButtonColorStates::flat(if is_colored {
                            bar_color
                        } else {
                            COLOR_UNCOLORED
                        }),
                        theme::colors::TEXT_DARK,
                    )
                };
                let mut cmd = parent.spawn((
                    Node {
                        width: theme::scaled(slice_width),
                        height: theme::scaled(bar_height),
                        border: UiRect::all(theme::scaled(SLICE_BORDER)),
                        border_radius: slice_border_radius(i, denominator),
                        ..default()
                    },
                    BackgroundColor(palette.background.normal),
                    BorderColor::all(COLOR_SLICE_BORDER),
                ));

                if interactive {
                    cmd.insert((
                        FractionSlice {
                            colored: is_colored,
                        },
                        ButtonVisualStyle(palette),
                        AutoDirectionalNavigation::default(),
                        TabIndex(0),
                        Outline::new(
                            Val::Px(theme::sizes::FOCUS_RING_WIDTH),
                            Val::Px(theme::sizes::FOCUS_RING_OFFSET),
                            Color::NONE,
                        ),
                    ));
                    if i == 0 {
                        cmd.insert(AutoFocus);
                    }
                }
            }
        })),
    )
}

fn slice_border_radius(index: u32, total: u32) -> BorderRadius {
    if total <= 1 {
        return BorderRadius::all(theme::scaled(CORNER_RADIUS));
    }
    if index == 0 {
        return BorderRadius {
            top_left: theme::scaled(CORNER_RADIUS).into(),
            bottom_left: theme::scaled(CORNER_RADIUS).into(),
            top_right: theme::scaled(0.0).into(),
            bottom_right: theme::scaled(0.0).into(),
        };
    }
    if index == total - 1 {
        return BorderRadius {
            top_left: theme::scaled(0.0).into(),
            bottom_left: theme::scaled(0.0).into(),
            top_right: theme::scaled(CORNER_RADIUS).into(),
            bottom_right: theme::scaled(CORNER_RADIUS).into(),
        };
    }
    BorderRadius::ZERO
}
