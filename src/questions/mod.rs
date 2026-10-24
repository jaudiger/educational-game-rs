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
use bevy::window::PrimaryWindow;

use crate::ui::components::{button_base, stacked_fraction, wrapping_standard_button};
use crate::ui::rich_text::{TextSegment, parse_fraction_segments, spawn_question_text};
use crate::ui::theme;

const TWO_COLUMN_CELL_BASIS: f32 = 400.0;
const THREE_COLUMN_CELL_BASIS: f32 = 280.0;
const MAX_VISUAL_COLUMNS: usize = 3;
const GRID_ROUNDING_ALLOWANCE_DESIGN_PX: f32 = 1.0;
const RESPONSIVE_ROW_HYSTERESIS: f32 = 0.02;

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

/// Marks a visual group that stays in one row or stacks as a whole.
#[derive(Component, Reflect)]
pub struct ResponsiveVisualGroup {
    /// Maximum fraction of the parent content width used by the row.
    max_row_occupancy: f32,
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
            max_width: theme::scaled(responsive_grid_max_width(
                TWO_COLUMN_CELL_BASIS,
                2,
                gap_design,
            )),
        },
        _ => ResponsiveGridLayout {
            cell_basis: theme::scaled(THREE_COLUMN_CELL_BASIS),
            max_width: theme::scaled(responsive_grid_max_width(
                THREE_COLUMN_CELL_BASIS,
                3,
                gap_design,
            )),
        },
    }
}

const fn responsive_grid_max_width(cell_basis: f32, columns: usize, gap: f32) -> f32 {
    match columns {
        1 => cell_basis,
        2 => cell_basis * 2.0 + gap + GRID_ROUNDING_ALLOWANCE_DESIGN_PX,
        _ => cell_basis * 3.0 + gap * 2.0 + GRID_ROUNDING_ALLOWANCE_DESIGN_PX,
    }
}

/// Spawns answer choices in a responsive grid with centered cells.
pub fn spawn_choice_grid(
    parent: &mut ChildSpawnerCommands,
    choice_count: usize,
    spawn_choice: impl FnMut(&mut ChildSpawnerCommands, usize),
) {
    let gap_design = theme::spacing::MEDIUM;
    let layout = responsive_grid_layout(2, gap_design);
    spawn_responsive_grid(
        parent,
        choice_count,
        layout,
        theme::scaled(gap_design),
        |_| layout.cell_basis,
        ResponsiveGridMode::ChoiceGrid,
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
    let (layout, mode) = match visual_layout {
        QuestionVisualLayout::Grid { max_columns } => {
            let max_columns = max_columns
                .clamp(1, MAX_VISUAL_COLUMNS)
                .min(cell_sizing.len());
            (
                responsive_grid_layout(max_columns, gap_design),
                ResponsiveGridMode::QuestionVisualGrid,
            )
        }
        QuestionVisualLayout::SingleRowOrColumn { max_row_occupancy } => (
            ResponsiveGridLayout {
                cell_basis: Val::Auto,
                max_width: theme::layout::FILL_PARENT,
            },
            ResponsiveGridMode::SingleRowOrColumn { max_row_occupancy },
        ),
    };
    spawn_responsive_grid(
        parent,
        cell_sizing.len(),
        layout,
        theme::scaled(gap_design),
        |index| match cell_sizing[index] {
            QuestionVisualSizing::Preferred => layout.cell_basis,
            QuestionVisualSizing::Intrinsic => Val::Auto,
        },
        mode,
        spawn_visual,
    );
}

#[derive(Clone, Copy)]
enum ResponsiveGridMode {
    ChoiceGrid,
    QuestionVisualGrid,
    SingleRowOrColumn { max_row_occupancy: f32 },
}

fn spawn_responsive_grid(
    parent: &mut ChildSpawnerCommands,
    item_count: usize,
    layout: ResponsiveGridLayout,
    gap: Val,
    cell_basis: impl Fn(usize) -> Val,
    mode: ResponsiveGridMode,
    mut spawn_item: impl FnMut(&mut ChildSpawnerCommands, usize),
) {
    let (flex_direction, flex_wrap) = match mode {
        ResponsiveGridMode::ChoiceGrid | ResponsiveGridMode::QuestionVisualGrid => {
            (FlexDirection::Row, FlexWrap::Wrap)
        }
        ResponsiveGridMode::SingleRowOrColumn { .. } => (FlexDirection::Column, FlexWrap::NoWrap),
    };
    let mut grid = parent.spawn(Node {
        flex_direction,
        flex_wrap,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: gap,
        row_gap: gap,
        width: theme::layout::FILL_PARENT,
        max_width: layout.max_width,
        ..default()
    });
    if let ResponsiveGridMode::SingleRowOrColumn { max_row_occupancy } = mode {
        grid.insert(ResponsiveVisualGroup { max_row_occupancy });
    }
    grid.with_children(|grid| {
        for index in 0..item_count {
            let preferred_basis = cell_basis(index);
            let mut node = Node {
                flex_basis: preferred_basis,
                flex_grow: if matches!(mode, ResponsiveGridMode::SingleRowOrColumn { .. }) {
                    0.0
                } else {
                    1.0
                },
                min_width: px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            };
            if matches!(mode, ResponsiveGridMode::SingleRowOrColumn { .. }) {
                node.flex_shrink = 0.0;
            }
            let mut cell = grid.spawn(node);
            cell.with_children(|cell| spawn_item(cell, index));
        }
    });
}

/// Uses live cell sizes and parent content width to keep the layout stable.
pub fn update_responsive_visual_groups(
    primary_window: Single<&Window, With<PrimaryWindow>>,
    layout_nodes: Query<&ComputedNode>,
    mut groups: Query<(
        &ResponsiveVisualGroup,
        &ChildOf,
        &Children,
        &ComputedNode,
        &mut Node,
    )>,
) {
    for (group, child_of, children, computed, mut style) in &mut groups {
        let Ok(parent_layout) = layout_nodes.get(child_of.parent()) else {
            continue;
        };
        let parent_inset = parent_layout.content_inset();
        let available_width =
            (parent_layout.size().x - parent_inset.min_inset.x - parent_inset.max_inset.x).max(0.0);
        if available_width <= 0.0 {
            continue;
        }

        let gap = style
            .column_gap
            .resolve(
                computed.inverse_scale_factor.recip(),
                computed.size().x,
                primary_window.physical_size().as_vec2(),
                computed.em_size,
                computed.rem_size,
            )
            .unwrap_or_default();
        let Some((_, required_width)) =
            children
                .iter()
                .try_fold((0_usize, 0.0_f32), |(count, total), child| {
                    layout_nodes.get(child).ok().map(|cell| {
                        let gap = if count == 0 { 0.0 } else { gap };
                        (count + 1, total + gap + cell.size().x)
                    })
                })
        else {
            continue;
        };

        let max_row_width = available_width * group.max_row_occupancy;
        let row_threshold = if style.flex_direction == FlexDirection::Column {
            max_row_width * (1.0 - RESPONSIVE_ROW_HYSTERESIS)
        } else {
            max_row_width
        };
        let direction = if required_width <= row_threshold {
            FlexDirection::Row
        } else {
            FlexDirection::Column
        };
        if style.flex_direction != direction {
            style.flex_direction = direction;
        }
    }
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
