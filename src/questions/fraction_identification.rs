//! Fraction-identification question type.
//!
//! Displays a pre-coloured fraction shape and MCQ-style choice buttons. The
//! student identifies which fraction the shape represents.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::fraction_shape::spawn_fraction_representation;
use super::registry::{QuestionRoot, register_question_systems};
use crate::data::{LessonSession, QuestionContainer, QuestionInstance};
use crate::i18n::{I18n, TranslationKey};

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
    mut images: ResMut<Assets<Image>>,
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
                super::spawn_question_card(root, |card| {
                    super::spawn_question_prompt(
                        card,
                        &i18n.t(&TranslationKey::WhatFractionColored),
                        window,
                    );
                    super::spawn_question_visuals(
                        card,
                        super::QuestionVisualLayout::Grid { max_columns: 1 },
                        &[super::QuestionVisualSizing::Preferred],
                        |visual, _| {
                            spawn_fraction_representation(
                                visual,
                                &mut images,
                                def.numerator,
                                def.denominator,
                                def.mode,
                            );
                        },
                    );
                });
                super::spawn_controls_card(root, |card| {
                    super::spawn_indexed_choices(
                        card,
                        &def.choices,
                        |i| IdentificationChoice { index: i },
                        window,
                    );
                });
            });
    });
}

mod interaction;
