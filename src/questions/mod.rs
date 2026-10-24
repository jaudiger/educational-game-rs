pub mod fraction_bar;
pub mod fraction_comparison;
pub mod fraction_identification;
pub mod fraction_shape;
pub mod fraction_visualization;
pub mod mcq;
pub mod multiplication_grid;
pub mod numeric_input;
pub mod numeric_keypad;
pub mod registry;

pub use fraction_comparison::FractionComparisonPlugin;
pub use fraction_identification::FractionIdentificationPlugin;
pub use fraction_visualization::FractionVisualizationPlugin;
pub use mcq::McqPlugin;
pub use numeric_input::NumericInputPlugin;
pub use registry::{AnswerSubmitted, QuestionRoot};

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use crate::ui::components::{button_base, stacked_fraction, wrapping_standard_button};
use crate::ui::layout::{self, CollectionItemLayout, CollectionLayout};
use crate::ui::rich_text::{TextSegment, parse_fraction_segments, spawn_question_text};
use crate::ui::theme;

const TWO_COLUMN_CELL_BASIS: f32 = 400.0;
const THREE_COLUMN_CELL_BASIS: f32 = 280.0;
const MAX_VISUAL_COLUMNS: usize = 3;

/// Shared [`Node`] layout for all question root markers.
///
/// Used as a `#[require(Node(...))]` default constructor so every question
/// type gets a consistent column layout without repeating it at each spawn site.
pub fn question_root_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: theme::scaled(theme::spacing::LARGE),
        width: theme::layout::FILL_PARENT,
        min_width: px(0.0),
        max_width: theme::scaled(960.0),
        margin: UiRect::vertical(Val::Auto),
        ..default()
    }
}

/// Marks the card containing the question prompt and visual stimulus.
#[derive(Component, Reflect)]
pub struct QuestionCard;

/// Marks the card containing answer and validation controls.
#[derive(Component, Reflect)]
pub struct ControlsCard;

/// Creates a question card for the prompt and its visual stimulus.
pub fn spawn_question_card(
    parent: &mut ChildSpawnerCommands,
    spawn_content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(question_card_node())
        .with_children(spawn_content);
}

/// Creates a controls card for answer choices and validation controls.
pub fn spawn_controls_card(
    parent: &mut ChildSpawnerCommands,
    spawn_content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(controls_card_node())
        .with_children(spawn_content);
}

fn question_card_node() -> impl Bundle + use<> {
    (
        layout_card(theme::scaled(theme::spacing::LARGE)),
        QuestionCard,
    )
}

fn controls_card_node() -> impl Bundle + use<> {
    (
        layout_card(theme::scaled(theme::spacing::MEDIUM)),
        ControlsCard,
    )
}

fn layout_card(row_gap: Val) -> impl Bundle + use<> {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap,
            width: theme::layout::FILL_PARENT,
            min_width: px(0.0),
            max_width: theme::scaled(960.0),
            padding: theme::scaled(theme::sizes::CARD_PADDING).all(),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_OVERLAY),
        BorderColor::all(theme::colors::CARD_BORDER),
    )
}

/// Selects how a question-visual group arranges its cells.
#[derive(Clone, Copy)]
pub enum QuestionVisualLayout {
    /// Wraps cells using the requested maximum column count.
    Grid { max_columns: usize },
    /// Keeps all cells on one row or stacks the group in one column.
    SingleRowOrColumn {
        /// Maximum fraction of available width used by the row.
        max_row_occupancy: f32,
    },
}

/// Selects the preferred basis for a question-visual cell.
#[derive(Clone, Copy)]
pub enum QuestionVisualSizing {
    Preferred,
    Intrinsic,
}

#[derive(Clone, Copy)]
struct ResponsiveGridLayout {
    cell_basis: Val,
    max_width: Val,
}

fn responsive_grid_layout(max_columns: usize, gap_design: f32) -> ResponsiveGridLayout {
    match max_columns.clamp(1, MAX_VISUAL_COLUMNS) {
        1 => ResponsiveGridLayout {
            cell_basis: theme::layout::FILL_PARENT,
            max_width: theme::layout::FILL_PARENT,
        },
        2 => ResponsiveGridLayout {
            cell_basis: theme::scaled(TWO_COLUMN_CELL_BASIS),
            max_width: theme::scaled(layout::responsive_grid_max_width(
                TWO_COLUMN_CELL_BASIS,
                2,
                gap_design,
            )),
        },
        _ => ResponsiveGridLayout {
            cell_basis: theme::scaled(THREE_COLUMN_CELL_BASIS),
            max_width: theme::scaled(layout::responsive_grid_max_width(
                THREE_COLUMN_CELL_BASIS,
                3,
                gap_design,
            )),
        },
    }
}

/// Spawns answer choices in a responsive grid with centered cells.
pub fn spawn_choice_grid(
    parent: &mut ChildSpawnerCommands,
    choice_count: usize,
    spawn_choice: impl FnMut(&mut ChildSpawnerCommands, usize),
) {
    let gap_design = theme::spacing::MEDIUM;
    let grid = responsive_grid_layout(2, gap_design);
    layout::spawn_collection(
        parent,
        choice_count,
        CollectionLayout::grid(grid.max_width, theme::scaled(gap_design)),
        |_| CollectionItemLayout::new(grid.cell_basis, 1.0, 1.0),
        spawn_choice,
    );
}

/// Spawns question visuals in a responsive layout below the question prompt.
pub fn spawn_question_visuals(
    parent: &mut ChildSpawnerCommands,
    visual_layout: QuestionVisualLayout,
    cell_sizing: &[QuestionVisualSizing],
    spawn_visual: impl FnMut(&mut ChildSpawnerCommands, usize),
) {
    if cell_sizing.is_empty() {
        return;
    }
    let gap_design = theme::spacing::MEDIUM;
    let (collection, grid, adaptive) = match visual_layout {
        QuestionVisualLayout::Grid { max_columns } => {
            let grid = responsive_grid_layout(max_columns.min(cell_sizing.len()), gap_design);
            (
                CollectionLayout::grid(grid.max_width, theme::scaled(gap_design)),
                Some(grid),
                false,
            )
        }
        QuestionVisualLayout::SingleRowOrColumn { max_row_occupancy } => (
            CollectionLayout::adaptive(max_row_occupancy, theme::scaled(gap_design)),
            None,
            true,
        ),
    };
    layout::spawn_collection(
        parent,
        cell_sizing.len(),
        collection,
        |index| {
            let basis = match (grid, cell_sizing[index]) {
                (Some(grid), QuestionVisualSizing::Preferred) => grid.cell_basis,
                (Some(_), QuestionVisualSizing::Intrinsic) | (None, _) => Val::Auto,
            };
            if adaptive {
                CollectionItemLayout::new(basis, 0.0, 0.0).with_layout_width()
            } else {
                CollectionItemLayout::new(basis, 1.0, 1.0)
            }
        },
        spawn_visual,
    );
}

/// Spawns a heading-size question prompt, supporting inline fraction notation.
pub fn spawn_question_prompt(parent: &mut ChildSpawnerCommands, text: &str, window: Entity) {
    spawn_question_text(
        parent,
        text,
        theme::fonts::HEADING,
        theme::colors::TEXT_DARK,
        window,
    );
}

/// Spawns responsive choice buttons from a string slice.
///
/// Fraction choices render as stacked fractions and other choices render as text.
/// The first button receives `AutoFocus`, and `make_marker(index)` attaches its marker.
pub fn spawn_indexed_choices<F, B>(
    parent: &mut ChildSpawnerCommands,
    choices: &[String],
    make_marker: F,
    window: Entity,
) where
    F: Fn(usize) -> B,
    B: Bundle,
{
    spawn_choice_grid(parent, choices.len(), |cell, index| {
        let choice_text = &choices[index];
        let segments = parse_fraction_segments(choice_text);
        let pure_fraction = match segments.as_slice() {
            [TextSegment::Fraction(n, d)] => Some((*n, *d)),
            _ => None,
        };

        if let Some((n, d)) = pure_fraction {
            let mut cmd = cell.spawn((
                button_base(theme::button_styles::ANSWER),
                Node {
                    min_width: px(0.0),
                    height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
                    width: theme::layout::FILL_PARENT,
                    padding: UiRect::axes(
                        theme::scaled(theme::sizes::BUTTON_PADDING),
                        theme::scaled(0.0),
                    ),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip(),
                    border_radius: BorderRadius::all(theme::scaled(
                        theme::sizes::BUTTON_BORDER_RADIUS,
                    )),
                    ..default()
                },
                make_marker(index),
            ));
            cmd.with_children(|btn| {
                btn.spawn(stacked_fraction(
                    n,
                    d,
                    theme::fonts::BUTTON,
                    theme::button_styles::ANSWER.foreground,
                    theme::button_styles::ANSWER.foreground,
                    window,
                ));
            });
            if index == 0 {
                cmd.insert(AutoFocus);
            }
        } else {
            let mut cmd = cell.spawn((
                wrapping_standard_button(
                    choice_text,
                    theme::button_styles::ANSWER,
                    theme::layout::FILL_PARENT,
                    window,
                ),
                make_marker(index),
            ));
            if index == 0 {
                cmd.insert(AutoFocus);
            }
        }
    });
}
