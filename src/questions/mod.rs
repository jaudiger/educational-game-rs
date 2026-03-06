pub mod fraction_bar;
pub mod fraction_comparison;
pub mod fraction_identification;
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

use crate::ui::components::{button_base, stacked_fraction, standard_button};
use crate::ui::rich_text::{TextSegment, parse_fraction_segments, spawn_rich_text};
use crate::ui::theme;

/// Shared [`Node`] layout for all question root markers.
///
/// Used as a `#[require(Node(...))]` default constructor so every question
/// type gets a consistent column layout without repeating it at each spawn site.
pub fn question_root_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: theme::scaled(theme::spacing::LARGE),
        width: percent(100.0),
        ..default()
    }
}

/// Spawns a heading-size question prompt, supporting inline fraction notation.
pub fn spawn_question_prompt(parent: &mut ChildSpawnerCommands, text: &str, window: Entity) {
    spawn_rich_text(
        parent,
        text,
        theme::fonts::HEADING,
        theme::colors::TEXT_DARK,
        window,
    );
}

/// Spawns a column of choice buttons from a string slice.
///
/// Each choice is checked for fraction notation; fraction choices render as stacked
/// fraction buttons, plain text choices render as standard buttons. The first button
/// receives `AutoFocus`. `make_marker(index)` attaches the caller's choice marker.
pub fn spawn_indexed_choices<F, B>(
    parent: &mut ChildSpawnerCommands,
    choices: &[String],
    make_marker: F,
    window: Entity,
) where
    F: Fn(usize) -> B,
    B: Bundle,
{
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: theme::scaled(theme::spacing::MEDIUM),
            margin: theme::scaled(theme::spacing::LARGE).top(),
            ..default()
        })
        .with_children(|col| {
            for (index, choice_text) in choices.iter().enumerate() {
                let segments = parse_fraction_segments(choice_text);
                let pure_fraction = match segments.as_slice() {
                    [TextSegment::Fraction(n, d)] => Some((*n, *d)),
                    _ => None,
                };

                if let Some((n, d)) = pure_fraction {
                    let mut cmd = col.spawn((
                        button_base(theme::colors::SECONDARY),
                        Node {
                            min_width: theme::scaled(theme::sizes::BUTTON_WIDTH),
                            height: theme::scaled(theme::sizes::BUTTON_HEIGHT),
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
                            theme::colors::TEXT_LIGHT,
                            theme::colors::TEXT_LIGHT,
                            window,
                        ));
                    });
                    if index == 0 {
                        cmd.insert(AutoFocus);
                    }
                } else {
                    let mut cmd = col.spawn((
                        standard_button(
                            choice_text,
                            theme::colors::SECONDARY,
                            theme::scaled(theme::sizes::BUTTON_WIDTH),
                            window,
                        ),
                        make_marker(index),
                    ));
                    if index == 0 {
                        cmd.insert(AutoFocus);
                    }
                }
            }
        });
}
