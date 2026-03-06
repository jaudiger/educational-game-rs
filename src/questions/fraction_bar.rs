//! Shared helper for rendering fraction bars (rows of rectangular slices).

use bevy::input_focus::AutoFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;

use crate::ui::theme;

/// Default total width of the bar in pixels.
pub const BAR_WIDTH: f32 = 500.0;
/// Default height of each slice in pixels.
pub const BAR_HEIGHT: f32 = 80.0;
/// Gap between slices in pixels.
const SLICE_GAP: f32 = 3.0;
/// Border radius for the outermost corners.
const CORNER_RADIUS: f32 = 6.0;

/// Background colour for an uncoloured slice.
pub const COLOR_UNCOLORED: Color = Color::srgb(0.73, 0.73, 0.78);
/// Border colour drawn around every slice.
const COLOR_SLICE_BORDER: Color = Color::srgb(0.60, 0.60, 0.65);
/// Border thickness around each slice in pixels.
const SLICE_BORDER: f32 = 1.5;

/// Marker component placed on each interactive slice of a fraction bar.
#[derive(Component, Reflect)]
#[require(Button)]
pub struct FractionSlice {
    pub colored: bool,
}

/// Returns a fraction bar bundle.
///
/// * `denominator`: total number of slices.
/// * `colored_count`: how many leading slices start coloured.
/// * `bar_color`: colour used for coloured slices.
/// * `interactive`: if `true`, each slice gets a [`Button`] and [`FractionSlice`].
/// * `bar_width`: total width of the bar in pixels.
/// * `bar_height`: height of each slice in pixels.
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
            column_gap: px(SLICE_GAP),
            align_items: AlignItems::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for i in 0..denominator {
                let is_colored = i < colored_count;
                let bg = if is_colored {
                    bar_color
                } else {
                    COLOR_UNCOLORED
                };

                let mut cmd = parent.spawn((
                    Node {
                        width: px(slice_width),
                        height: px(bar_height),
                        border: UiRect::all(px(SLICE_BORDER)),
                        border_radius: slice_border_radius(i, denominator),
                        ..default()
                    },
                    BackgroundColor(bg),
                    BorderColor::all(COLOR_SLICE_BORDER),
                ));

                if interactive {
                    cmd.insert((
                        FractionSlice {
                            colored: is_colored,
                        },
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
        return BorderRadius::all(px(CORNER_RADIUS));
    }
    if index == 0 {
        return BorderRadius {
            top_left: px(CORNER_RADIUS),
            bottom_left: px(CORNER_RADIUS),
            top_right: px(0.0),
            bottom_right: px(0.0),
        };
    }
    if index == total - 1 {
        return BorderRadius {
            top_left: px(0.0),
            bottom_left: px(0.0),
            top_right: px(CORNER_RADIUS),
            bottom_right: px(CORNER_RADIUS),
        };
    }
    BorderRadius::ZERO
}
