use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::fraction_bar;
use super::multiplication_grid::spawn_prompt_and_grid;
use super::registry::{QuestionRoot, register_question_systems, submit_answer};
use crate::data::content::QuestionVisual;
use crate::data::{AnswerResult, LessonSession, QuestionContainer, QuestionDefinition};
use crate::i18n::I18n;
use crate::states::LessonPhase;
use crate::ui::rich_text::spawn_rich_text;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

/// Handles multiple-choice question UI and answer submission.
pub struct McqPlugin;

impl Plugin for McqPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionDefinition::Mcq(_)),
            spawn_mcq_ui,
            handle_mcq_click,
        );
    }
}

#[derive(Component, Reflect)]
#[require(Node = super::question_root_node())]
struct McqRoot;

#[derive(Component, Reflect)]
struct McqChoice {
    index: usize,
}

fn spawn_mcq_ui(
    mut commands: Commands,
    container: Single<Entity, With<QuestionContainer>>,
    session: Res<LessonSession>,
    i18n: Res<I18n>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let Some(question) = session.current() else {
        return;
    };
    let QuestionDefinition::Mcq(mcq) = &question.definition else {
        return;
    };

    let hide_visual = question.hide_visual;
    let show_grid = !hide_visual;

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((McqRoot, QuestionRoot))
            .with_children(|mcq_root| {
                if show_grid {
                    match &mcq.question_visual {
                        Some(QuestionVisual::MultiplicationGrid { rows, cols }) => {
                            spawn_prompt_and_grid(
                                mcq_root,
                                mcq.prompt.get(i18n.language),
                                *rows,
                                *cols,
                                window,
                            );
                            super::spawn_indexed_choices(
                                mcq_root,
                                &mcq.choices,
                                |i| McqChoice { index: i },
                                window,
                            );
                            return;
                        }
                        Some(QuestionVisual::FractionAddition { a, b, c }) => {
                            spawn_prompt_and_fraction_bars(
                                mcq_root,
                                mcq.prompt.get(i18n.language),
                                *a,
                                *b,
                                *c,
                                window,
                            );
                            super::spawn_indexed_choices(
                                mcq_root,
                                &mcq.choices,
                                |i| McqChoice { index: i },
                                window,
                            );
                            return;
                        }
                        None => {}
                    }
                }
                // Default: text prompt only.
                super::spawn_question_prompt(mcq_root, mcq.prompt.get(i18n.language), window);
                super::spawn_indexed_choices(
                    mcq_root,
                    &mcq.choices,
                    |i| McqChoice { index: i },
                    window,
                );
            });
    });
}

/// Spawn the text prompt followed by two fraction bars (`a/b` blue + `c/b` orange).
/// No result bar; the student must calculate the answer.
fn spawn_prompt_and_fraction_bars(
    parent: &mut ChildSpawnerCommands,
    prompt_text: &str,
    a: u32,
    b: u32,
    c: u32,
    window: Entity,
) {
    /// Width of each mini-bar in the fraction addition visual.
    const MINI_BAR_WIDTH: f32 = 200.0;
    /// Height of each mini-bar in the fraction addition visual.
    const MINI_BAR_HEIGHT: f32 = 40.0;

    // Prompt text
    spawn_rich_text(
        parent,
        prompt_text,
        theme::fonts::HEADING,
        theme::colors::TEXT_DARK,
        window,
    );

    // Fraction bars row: a/b (blue) + "+" + c/b (orange)
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            row_gap: theme::scaled(theme::spacing::SMALL),
            margin: theme::scaled(theme::spacing::MEDIUM).top(),
            ..default()
        })
        .with_children(|row| {
            // First operand: a/b
            row.spawn(fraction_bar(
                b,
                a,
                theme::colors::PRIMARY,
                false,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
            ));
            row.spawn((
                Text::new("+"),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ));
            // Second operand: c/b
            row.spawn(fraction_bar(
                b,
                c,
                theme::colors::SECONDARY,
                false,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
            ));
        });
}

type McqChoiceQuery<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static McqChoice), (Changed<Interaction>, With<Button>)>;

fn handle_mcq_click(
    query: McqChoiceQuery<'_, '_>,
    session: Res<LessonSession>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<LessonPhase>>,
) {
    for (interaction, choice) in &query {
        if *interaction == Interaction::Pressed {
            let Some(question) = session.current() else {
                return;
            };
            let QuestionDefinition::Mcq(mcq) = &question.definition else {
                return;
            };

            let result = if choice.index == mcq.correct_index {
                AnswerResult::Correct
            } else {
                AnswerResult::Incorrect
            };

            submit_answer(&mut commands, &mut next_phase, result);
            return;
        }
    }
}
