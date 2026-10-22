use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::fraction_bar;
use super::multiplication_grid::spawn_multiplication_grid;
use super::registry::{QuestionRoot, register_question_systems};
use crate::data::content::QuestionVisual;
use crate::data::{LessonSession, QuestionContainer, QuestionInstance};
use crate::i18n::I18n;
use crate::ui::theme;

/// Handles multiple-choice question UI and answer submission.
pub struct McqPlugin;

impl Plugin for McqPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionInstance::Mcq(_)),
            spawn_mcq_ui,
            interaction::handle_mcq_click,
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
    let QuestionInstance::Mcq(mcq) = &question.instance else {
        return;
    };

    let hide_visual = question.hide_optional_visual;
    let show_grid = !hide_visual;

    commands.entity(*container).with_children(|parent| {
        parent.spawn((McqRoot, QuestionRoot)).with_children(|root| {
            super::spawn_question_card(root, |card| {
                super::spawn_question_prompt(card, mcq.prompt.get(i18n.language), window);
                if show_grid {
                    match &mcq.question_visual {
                        Some(QuestionVisual::MultiplicationGrid { rows, cols }) => {
                            super::spawn_question_visuals(card, 1, 1, |visual, _| {
                                spawn_multiplication_grid(visual, *rows, *cols);
                            });
                        }
                        Some(QuestionVisual::FractionAddition { a, b, c }) => {
                            spawn_fraction_addition_visuals(card, *a, *b, *c, window);
                        }
                        None => {}
                    }
                }
            });
            super::spawn_controls_card(root, |card| {
                super::spawn_indexed_choices(
                    card,
                    &mcq.choices,
                    |i| McqChoice { index: i },
                    window,
                );
            });
        });
    });
}

/// Spawns the fraction-addition bars and plus sign as separate visual cells.
fn spawn_fraction_addition_visuals(
    parent: &mut ChildSpawnerCommands,
    a: u32,
    b: u32,
    c: u32,
    window: Entity,
) {
    const MINI_BAR_WIDTH: f32 = 200.0;
    const MINI_BAR_HEIGHT: f32 = 40.0;

    super::spawn_question_visuals(parent, 3, 3, |visual, index| match index {
        0 => {
            visual.spawn(fraction_bar(
                b,
                a,
                theme::colors::PRIMARY,
                false,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
            ));
        }
        1 => {
            visual.spawn((
                Text::new("+"),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(theme::colors::TEXT_DARK),
            ));
        }
        _ => {
            visual.spawn(fraction_bar(
                b,
                c,
                theme::colors::SECONDARY,
                false,
                MINI_BAR_WIDTH,
                MINI_BAR_HEIGHT,
            ));
        }
    });
}

mod interaction;
