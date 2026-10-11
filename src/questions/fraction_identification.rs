//! Fraction-identification question type.
//!
//! Displays a pre-coloured fraction bar and MCQ-style choice buttons. The
//! student identifies which fraction the bar represents.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_bar::{BAR_HEIGHT, BAR_WIDTH, fraction_bar};
use super::registry::{QuestionRoot, register_question_systems};
use crate::data::{LessonSession, QuestionContainer, QuestionInstance};
use crate::i18n::{I18n, TranslationKey};
use crate::ui::theme;

/// Handles fraction identification question UI and answer submission.
pub struct FractionIdentificationPlugin;

impl Plugin for FractionIdentificationPlugin {
    fn build(&self, app: &mut App) {
        register_question_systems(
            app,
            |d| matches!(d, QuestionInstance::FractionIdentification(_)),
            spawn_identification_ui,
            interaction::handle_identification_click,
        );
    }
}

#[derive(Component, Reflect)]
#[require(Node = super::question_root_node())]
struct FractionIdentificationRoot;

#[derive(Component, Reflect)]
struct IdentificationChoice {
    index: usize,
}

fn spawn_identification_ui(
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
    let QuestionInstance::FractionIdentification(def) = &question.instance else {
        return;
    };

    commands.entity(*container).with_children(|parent| {
        parent
            .spawn((FractionIdentificationRoot, QuestionRoot))
            .with_children(|root| {
                super::spawn_question_prompt(
                    root,
                    &i18n.t(&TranslationKey::WhatFractionColored),
                    window,
                );

                // Pre-coloured bar
                root.spawn(fraction_bar(
                    def.denominator,
                    def.numerator,
                    theme::colors::PRIMARY,
                    false,
                    BAR_WIDTH,
                    BAR_HEIGHT,
                ));

                super::spawn_indexed_choices(
                    root,
                    &def.choices,
                    |i| IdentificationChoice { index: i },
                    window,
                );
            });
    });
}

mod interaction;
