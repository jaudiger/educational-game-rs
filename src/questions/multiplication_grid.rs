//! Multiplication-grid visual helper.
//!
//! Provides a reusable function to render a visual grid of `rows` by `cols`
//! coloured cells.  Used by the MCQ and numeric-input question plugins
//! when the question carries a `QuestionVisual::MultiplicationGrid`.

use bevy::prelude::*;

use crate::ui::theme;

/// Maximum width of the grid area in design pixels.
const GRID_MAX_WIDTH: f32 = 350.0;
/// Maximum height of the grid area in design pixels.
const GRID_MAX_HEIGHT: f32 = 150.0;
/// Gap between grid cells in design pixels.
const CELL_GAP: f32 = 2.0;
/// Colour for grid cells.
const CELL_COLOR: Color = Color::srgb(0.4, 0.7, 0.95);

/// Returns a `Bundle` representing a visual grid of `rows` by `cols` coloured
/// cells.  The caller can spawn this directly into any parent entity.
#[allow(clippy::cast_precision_loss)]
fn multiplication_grid_visual(rows: u32, cols: u32) -> impl Bundle {
    // Compute cell size to fit within the max dimensions.
    let cell_w = if cols > 0 {
        CELL_GAP.mul_add(-((cols.saturating_sub(1)) as f32), GRID_MAX_WIDTH) / cols as f32
    } else {
        GRID_MAX_WIDTH
    };
    let cell_h = if rows > 0 {
        CELL_GAP.mul_add(-((rows.saturating_sub(1)) as f32), GRID_MAX_HEIGHT) / rows as f32
    } else {
        GRID_MAX_HEIGHT
    };
    let cell_size = cell_w.min(cell_h).clamp(10.0, 50.0);

    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(CELL_GAP),
            align_items: AlignItems::Center,
            ..default()
        },
        Children::spawn(SpawnIter((0..rows).map(move |_| {
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: theme::scaled(CELL_GAP),
                    ..default()
                },
                Children::spawn(SpawnIter((0..cols).map(move |_| {
                    (
                        Node {
                            width: theme::scaled(cell_size),
                            height: theme::scaled(cell_size),
                            border_radius: BorderRadius::all(theme::scaled(3.0)),
                            ..default()
                        },
                        BackgroundColor(CELL_COLOR),
                    )
                }))),
            )
        }))),
    )
}

/// Spawns a multiplication grid as a question visual.
pub fn spawn_multiplication_grid(parent: &mut ChildSpawnerCommands, rows: u32, cols: u32) {
    parent.spawn(multiplication_grid_visual(rows, cols));
}
