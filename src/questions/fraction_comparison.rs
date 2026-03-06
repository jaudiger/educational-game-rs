//! Fraction-comparison question type.
//!
//! Displays two pre-coloured fraction bars (one per character) and three
//! choice buttons: character A, character B, or "equal". The student picks
//! who ate more.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::{BAR_HEIGHT, BAR_WIDTH, fraction_bar};
use super::registry::{QuestionRoot, register_question_systems, submit_answer};
use crate::data::content::ComparisonAnswer;
use crate::data::{AnswerResult, LessonSession, QuestionContainer, QuestionDefinition};
use crate::i18n::{I18n, TranslationKey};
use crate::states::LessonPhase;
use crate::ui::components::standard_button;
use crate::ui::rich_text::spawn_rich_text;
use crate::ui::theme;

/// Handles fraction comparison question UI and answer submission.
pub struct FractionComparisonPlugin;

impl Plugin for FractionComparisonPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionDefinition::FractionComparison(_)),
            spawn_comparison_ui,
            handle_comparison_click,
        );
    }
}

#[derive(Component, Reflect)]
#[require(Node = super::question_root_node())]
struct FractionComparisonRoot;

#[derive(Component, Reflect)]
struct ComparisonChoice {
    answer: ComparisonAnswer,
}

fn spawn_comparison_ui(
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
    let QuestionDefinition::FractionComparison(def) = &question.definition else {
        return;
    };

    let show_bar = !question.hide_visual;

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((FractionComparisonRoot, QuestionRoot))
            .with_children(|root| {
                super::spawn_question_prompt(root, def.prompt.get(i18n.language), window);

                // Character rows grouped with tighter spacing
                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: percent(100.0),
                    row_gap: theme::scaled(theme::spacing::MEDIUM),
                    ..default()
                })
                .with_children(|group| {
                    spawn_character_row(
                        group,
                        &i18n,
                        &def.character_a,
                        def.fraction_a,
                        theme::colors::PRIMARY,
                        show_bar,
                        window,
                    );
                    spawn_character_row(
                        group,
                        &i18n,
                        &def.character_b,
                        def.fraction_b,
                        theme::colors::SECONDARY,
                        show_bar,
                        window,
                    );
                });

                // Choice buttons
                spawn_choice_buttons(root, &def.character_a, &def.character_b, &i18n, window);
            });
    });
}

fn spawn_character_row(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    name: &str,
    fraction: (u32, u32),
    bar_color: Color,
    show_bar: bool,
    window: Entity,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            width: percent(100.0),
            row_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        })
        .with_children(|row| {
            // Label: "Alice mange 3/8"
            spawn_rich_text(
                row,
                &i18n.t(&TranslationKey::CharacterAte(
                    name.to_owned(),
                    fraction.0,
                    fraction.1,
                )),
                theme::fonts::HEADING,
                theme::colors::TEXT_DARK,
                window,
            );

            // Pre-coloured bar (hidden when teacher disables the visual)
            if show_bar {
                row.spawn(fraction_bar(
                    fraction.1, fraction.0, bar_color, false, BAR_WIDTH, BAR_HEIGHT,
                ));
            }
        });
}

fn spawn_choice_buttons(
    parent: &mut ChildSpawnerCommands,
    name_a: &str,
    name_b: &str,
    i18n: &I18n,
    window: Entity,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            margin: theme::scaled(theme::spacing::LARGE).top(),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                comparison_button(name_a, ComparisonAnswer::A, window),
                AutoFocus,
            ));
            row.spawn(comparison_button(name_b, ComparisonAnswer::B, window));
            row.spawn(comparison_button(
                &i18n.t(&TranslationKey::EqualAmount),
                ComparisonAnswer::Equal,
                window,
            ));
        });
}

fn comparison_button(label: &str, answer: ComparisonAnswer, window: Entity) -> impl Bundle + use<> {
    (
        standard_button(
            label,
            theme::colors::SECONDARY,
            theme::scaled(180.0),
            window,
        ),
        ComparisonChoice { answer },
    )
}

type ComparisonChoiceQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static ComparisonChoice),
    (Changed<Interaction>, With<Button>),
>;

fn handle_comparison_click(
    query: ComparisonChoiceQuery<'_, '_>,
    session: Res<LessonSession>,
    mut commands: Commands,
    mut next_phase: ResMut<NextState<LessonPhase>>,
) {
    for (interaction, choice) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(question) = session.current() else {
            return;
        };
        let QuestionDefinition::FractionComparison(def) = &question.definition else {
            return;
        };

        let result = if choice.answer == def.answer {
            AnswerResult::Correct
        } else {
            AnswerResult::Incorrect
        };

        submit_answer(&mut commands, &mut next_phase, result);
        return;
    }
}
