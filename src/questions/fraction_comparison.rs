//! Fraction-comparison question type.
//!
//! Displays two pre-coloured fraction bars (one per character) and three
//! choice buttons: character A, character B, or "equal". The student picks
//! who ate more.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::{BAR_HEIGHT, fraction_bar};
use super::registry::{QuestionRoot, register_question_systems};
use crate::data::content::ComparisonAnswer;
use crate::data::{LessonSession, QuestionContainer, QuestionInstance};
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::wrapping_standard_button;
use crate::ui::rich_text::spawn_question_text;
use crate::ui::theme;

const COMPARISON_BAR_WIDTH: f32 = 400.0;

/// Handles fraction comparison question UI and answer submission.
pub struct FractionComparisonPlugin;

impl Plugin for FractionComparisonPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionInstance::FractionComparison(_)),
            spawn_comparison_ui,
            interaction::handle_comparison_click,
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
    let QuestionInstance::FractionComparison(def) = &question.instance else {
        return;
    };

    let show_bar = !question.hide_optional_visual;

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((FractionComparisonRoot, QuestionRoot))
            .with_children(|root| {
                super::spawn_question_card(root, |card| {
                    super::spawn_question_prompt(card, def.prompt.get(i18n.language), window);

                    super::spawn_question_visuals(card, 2, 2, |visual, index| {
                        let (name, fraction, color) = match index {
                            0 => (&def.character_a, def.fraction_a, theme::colors::PRIMARY),
                            _ => (&def.character_b, def.fraction_b, theme::colors::SECONDARY),
                        };
                        spawn_character_row(visual, &i18n, name, fraction, color, show_bar, window);
                    });
                });
                super::spawn_controls_card(root, |card| {
                    spawn_choice_buttons(card, &def.character_a, &def.character_b, &i18n, window);
                });
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
            width: theme::layout::FILL_PARENT,
            row_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        })
        .with_children(|row| {
            // Label: "Alice mange 3/8"
            spawn_question_text(
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
                    fraction.1,
                    fraction.0,
                    bar_color,
                    false,
                    COMPARISON_BAR_WIDTH,
                    BAR_HEIGHT,
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
    super::spawn_choice_grid(parent, 3, |cell, index| {
        let (label, answer) = match index {
            0 => (name_a.to_owned(), ComparisonAnswer::A),
            1 => (name_b.to_owned(), ComparisonAnswer::B),
            _ => (
                i18n.t(&TranslationKey::EqualAmount).into_owned(),
                ComparisonAnswer::Equal,
            ),
        };
        let mut command = cell.spawn(comparison_button(&label, answer, window));
        if index == 0 {
            command.insert(AutoFocus);
        }
    });
}

fn comparison_button(label: &str, answer: ComparisonAnswer, window: Entity) -> impl Bundle + use<> {
    (
        wrapping_standard_button(
            label,
            theme::button_styles::ANSWER,
            theme::layout::FILL_PARENT,
            window,
        ),
        ComparisonChoice { answer },
    )
}

mod interaction;
