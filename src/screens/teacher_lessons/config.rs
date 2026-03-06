use bevy::prelude::*;
use bevy_persistent::prelude::*;

use crate::data::content::{ContentLibrary, Lesson, MAX_QUESTION_REPETITIONS, QuestionType};
use crate::data::{ActiveSlot, Language, LessonSessionConfig, PlayerContext, SaveData};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{TeacherInDetailView, TeacherTab, tab_header};
use crate::screens::teacher_shared::question_type_label;
use crate::ui::components::{button_base, icon_button, standard_button};
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

use super::{
    ConfigHoverText, ConfigLessonButton, CountDecrementButton, CountIncrementButton, CountText,
    DraftQuestion, LessonConfigDraft, LessonsView, QuestionLabel, QuestionRow, ResetConfigButton,
    ReturnToTreeButton, SaveConfigButton, ScrollContent, ScrollFrame, ScrollIndicator,
    TeacherLessonsState, VisualToggleButton,
};

pub(super) fn spawn_config_view(
    parent: &mut ChildSpawner,
    i18n: &I18n,
    lesson_title: &str,
    draft: &LessonConfigDraft,
    active_tab: TeacherTab,
    window: Entity,
) {
    parent.spawn(tab_header(i18n, active_tab, window));
    spawn_config_header(
        parent,
        lesson_title,
        &i18n.t(&TranslationKey::ResetConfig),
        window,
    );
    spawn_question_counter_section(parent, i18n, draft, window);

    // Hover detail text: shows the full prompt of the hovered question row.
    // Lives outside the scroll frame to avoid layout-shift flicker.
    parent.spawn((
        Text::default(),
        TextFont {
            font_size: theme::fonts::SMALL,
            ..default()
        },
        TextColor(theme::colors::TEXT_MUTED),
        Node {
            min_height: theme::scaled(theme::fonts::SMALL + 4.0),
            overflow: Overflow::clip(),
            ..default()
        },
        ConfigHoverText,
        DesignFontSize {
            size: theme::fonts::SMALL,
            window,
        },
    ));

    spawn_button_row(parent, i18n, draft, window);
}

fn spawn_config_header(
    parent: &mut ChildSpawner,
    lesson_title: &str,
    reset_label: &str,
    window: Entity,
) {
    let lesson_title_owned = lesson_title.to_owned();
    let reset_label_owned = reset_label.to_owned();
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text::new(lesson_title_owned),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                Node {
                    flex_shrink: 1.0,
                    flex_grow: 1.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            ),
            (
                button_base(theme::colors::TOGGLE_INACTIVE),
                Node {
                    padding: UiRect::axes(
                        theme::scaled(theme::spacing::MEDIUM),
                        theme::scaled(theme::spacing::SMALL),
                    ),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(theme::scaled(6.0)),
                    ..default()
                },
                ResetConfigButton,
                children![(
                    Text::new(reset_label_owned),
                    TextFont {
                        font_size: theme::fonts::SMALL,
                        ..default()
                    },
                    TextColor(theme::colors::TEXT_DARK),
                    DesignFontSize {
                        size: theme::fonts::SMALL,
                        window,
                    },
                )],
            ),
        ],
    ));
}

fn spawn_question_counter_section(
    parent: &mut ChildSpawner,
    i18n: &I18n,
    draft: &LessonConfigDraft,
    window: Entity,
) {
    let i18n_owned = I18n::new(i18n.language);
    let questions = draft.questions.clone();

    parent.spawn((
        Node {
            flex_grow: 1.0,
            min_height: px(0.0),
            overflow: Overflow::scroll_y(),
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(theme::colors::INPUT_BORDER),
        ScrollFrame,
        Children::spawn(SpawnWith(move |scroll: &mut ChildSpawner| {
            scroll
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: theme::scaled(theme::spacing::SMALL),
                        padding: theme::scaled(theme::spacing::SMALL).all(),
                        width: percent(100.0),
                        ..default()
                    },
                    ScrollContent,
                ))
                .with_children(|col| {
                    let type_order = [
                        QuestionType::Mcq,
                        QuestionType::Visualization,
                        QuestionType::Comparison,
                        QuestionType::Identification,
                        QuestionType::NumericInput,
                    ];

                    for qt in type_order {
                        let questions_of_type: Vec<&DraftQuestion> =
                            questions.iter().filter(|q| q.question_type == qt).collect();
                        if questions_of_type.is_empty() {
                            continue;
                        }

                        col.spawn((
                            Text::new(question_type_label(qt, &i18n_owned)),
                            TextFont {
                                font_size: theme::fonts::BODY,
                                ..default()
                            },
                            TextColor(theme::colors::TEXT_DARK),
                            Node {
                                margin: theme::scaled(theme::spacing::SMALL).top(),
                                ..default()
                            },
                            DesignFontSize {
                                size: theme::fonts::BODY,
                                window,
                            },
                        ));

                        for q in questions_of_type {
                            spawn_question_counter_row(col, q, window);
                        }
                    }
                });

            // Scroll indicator (absolute, bottom-right, hidden by default)
            scroll.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: theme::scaled(theme::spacing::SMALL),
                    bottom: px(2.0),
                    ..default()
                },
                Text::new("\u{2195}"),
                TextFont {
                    font_size: theme::fonts::SMALL,
                    ..default()
                },
                TextColor(theme::colors::TEXT_MUTED),
                ScrollIndicator,
                Visibility::Hidden,
                DesignFontSize {
                    size: theme::fonts::SMALL,
                    window,
                },
            ));
        })),
    ));
}

fn spawn_question_counter_row(parent: &mut ChildSpawner, q: &DraftQuestion, window: Entity) {
    let full_prompt = q.full_prompt.clone();
    let count_str = q.count.to_string();
    let idx = q.index;
    let has_visual = q.has_visual;
    let show_visual = q.show_visual;
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: theme::scaled(theme::spacing::MEDIUM),
                margin: theme::scaled(theme::spacing::MEDIUM).left(),
                min_height: theme::scaled(36.0),
                ..default()
            },
            Interaction::None,
            QuestionRow(idx),
        ))
        .with_children(|row| {
            // Label: dynamically truncated by update_question_labels
            row.spawn((
                Text::new(format!("\u{2022} {full_prompt}")),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(theme::colors::TEXT_DARK),
                Node {
                    flex_shrink: 1.0,
                    flex_grow: 1.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                QuestionLabel(full_prompt),
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ));

            // Controls: [visual toggle]  [-] count [+]
            spawn_counter_controls(row, idx, has_visual, show_visual, &count_str, window);
        });
}

fn spawn_counter_controls(
    row: &mut ChildSpawner,
    idx: usize,
    has_visual: bool,
    show_visual: bool,
    count_str: &str,
    window: Entity,
) {
    row.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(0.0),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|controls| {
        // Visual toggle button (only shown if the question has an optional visual)
        // Wrapper node adds right margin to separate from the counter group.
        if has_visual {
            let toggle_bg = if show_visual {
                theme::colors::PRIMARY
            } else {
                theme::colors::TOGGLE_INACTIVE
            };
            controls
                .spawn(Node {
                    margin: theme::scaled(theme::spacing::SMALL).right(),
                    ..default()
                })
                .with_child((
                    icon_button(
                        30.0,
                        6.0,
                        "\u{25c9}",
                        theme::fonts::SMALL,
                        toggle_bg,
                        theme::colors::TEXT_LIGHT,
                        window,
                    ),
                    VisualToggleButton(idx),
                ));
        }

        controls.spawn((
            counter_button("\u{2212}", theme::colors::ERROR, window),
            CountDecrementButton(idx),
        ));
        controls.spawn((
            Text::new(count_str.to_owned()),
            TextFont {
                font_size: theme::fonts::SMALL,
                ..default()
            },
            TextColor(theme::colors::TEXT_DARK),
            TextLayout::new_with_justify(Justify::Center),
            CountText(idx),
            Node {
                min_width: theme::scaled(24.0),
                ..default()
            },
            DesignFontSize {
                size: theme::fonts::SMALL,
                window,
            },
        ));
        controls.spawn((
            counter_button("+", theme::colors::SUCCESS, window),
            CountIncrementButton(idx),
        ));
    });
}

fn spawn_button_row(
    parent: &mut ChildSpawner,
    i18n: &I18n,
    draft: &LessonConfigDraft,
    window: Entity,
) {
    let can_save = draft.has_any_selected();

    let save_bg = if can_save {
        theme::colors::SUCCESS
    } else {
        theme::colors::TOGGLE_INACTIVE
    };

    let back_text = i18n.t(&TranslationKey::Back).into_owned();
    let save_text = i18n.t(&TranslationKey::SaveConfig).into_owned();

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: theme::scaled(theme::spacing::MEDIUM),
            ..default()
        },
        Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
            row.spawn((
                standard_button(
                    &back_text,
                    theme::colors::PRIMARY,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                ReturnToTreeButton,
            ));
            row.spawn((
                standard_button(
                    &save_text,
                    save_bg,
                    theme::scaled(theme::sizes::BUTTON_WIDTH),
                    window,
                ),
                SaveConfigButton,
            ));
        })),
    ));
}

fn counter_button(label: &str, bg: Color, window: Entity) -> impl Bundle {
    icon_button(
        30.0,
        6.0,
        label,
        theme::fonts::SMALL,
        bg,
        theme::colors::TEXT_LIGHT,
        window,
    )
}

pub(super) fn build_draft_questions(
    lesson: &Lesson,
    language: Language,
    existing_config: Option<&LessonSessionConfig>,
) -> Vec<DraftQuestion> {
    lesson
        .questions
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let full_prompt = q.prompt_label(language);
            let count = existing_config
                .and_then(|c| c.counts.get(i).copied())
                .unwrap_or(1);
            let has_visual = q.has_optional_visual();
            let default_show_visual = q.default_show_visual();
            let show_visual = existing_config
                .and_then(|c| c.show_visuals.get(i).copied())
                .unwrap_or(default_show_visual);
            DraftQuestion {
                index: i,
                question_type: q.question_type(),
                full_prompt,
                count,
                has_visual,
                show_visual,
                default_show_visual,
            }
        })
        .collect()
}

/// Opens the config view for a lesson when the gear button is clicked.
pub(super) fn handle_config_button_click(
    query: Query<(&Interaction, &ConfigLessonButton), Changed<Interaction>>,
    mut commands: Commands,
    content: Res<ContentLibrary>,
    ctx: PlayerContext<'_>,
    i18n: Res<I18n>,
    mut lessons_state: ResMut<TeacherLessonsState>,
) {
    for (interaction, config_btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(theme_data) = content.theme(&config_btn.theme_id) else {
            continue;
        };
        let Some(lesson) = theme_data.lesson(&config_btn.lesson_id) else {
            continue;
        };

        let existing_config = ctx.active_slot.as_ref().and_then(|slot| {
            ctx.save_data.class_slots[slot.0]
                .as_ref()
                .and_then(|cs| cs.lesson_configs.get(&config_btn.lesson_id))
        });

        let questions = build_draft_questions(lesson, i18n.language, existing_config);
        let lesson_title = i18n.t(&lesson.title_key).into_owned();

        lessons_state.view = LessonsView::Config {
            lesson_id: config_btn.lesson_id.clone(),
            lesson_title,
            editing: LessonConfigDraft { questions },
        };
        commands.insert_resource(TeacherInDetailView);
        commands.trigger(super::RebuildLessons);

        return; // Only handle first press
    }
}

/// Increments the repetition count for a specific question.
pub(super) fn handle_count_increment(
    query: Query<(&Interaction, &CountIncrementButton), Changed<Interaction>>,
    mut lessons_state: ResMut<TeacherLessonsState>,
    mut text_query: Query<(&mut Text, &CountText)>,
    mut save_btn_query: Query<&mut BackgroundColor, With<SaveConfigButton>>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let idx = btn.0;
        let LessonsView::Config {
            ref mut editing, ..
        } = lessons_state.view
        else {
            continue;
        };
        let Some(q) = editing.questions.iter_mut().find(|q| q.index == idx) else {
            continue;
        };
        if q.count < MAX_QUESTION_REPETITIONS {
            q.count += 1;
            for (mut text, ct) in &mut text_query {
                if ct.0 == idx {
                    **text = q.count.to_string();
                }
            }
            update_save_button_state(&mut save_btn_query, editing);
        }
    }
}

/// Decrements the repetition count for a specific question.
pub(super) fn handle_count_decrement(
    query: Query<(&Interaction, &CountDecrementButton), Changed<Interaction>>,
    mut lessons_state: ResMut<TeacherLessonsState>,
    mut text_query: Query<(&mut Text, &CountText)>,
    mut save_btn_query: Query<&mut BackgroundColor, With<SaveConfigButton>>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let idx = btn.0;
        let LessonsView::Config {
            ref mut editing, ..
        } = lessons_state.view
        else {
            continue;
        };
        let Some(q) = editing.questions.iter_mut().find(|q| q.index == idx) else {
            continue;
        };
        if q.count > 0 {
            q.count -= 1;
            for (mut text, ct) in &mut text_query {
                if ct.0 == idx {
                    **text = q.count.to_string();
                }
            }
            update_save_button_state(&mut save_btn_query, editing);
        }
    }
}

/// Toggles the optional visual for a specific question on/off.
pub(super) fn handle_visual_toggle(
    query: Query<(&Interaction, &VisualToggleButton), Changed<Interaction>>,
    mut lessons_state: ResMut<TeacherLessonsState>,
    mut bg_query: Query<(&mut BackgroundColor, &VisualToggleButton)>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let idx = btn.0;
        let LessonsView::Config {
            ref mut editing, ..
        } = lessons_state.view
        else {
            continue;
        };
        let Some(q) = editing.questions.iter_mut().find(|q| q.index == idx) else {
            continue;
        };
        q.show_visual = !q.show_visual;
        let new_bg = if q.show_visual {
            theme::colors::PRIMARY
        } else {
            theme::colors::TOGGLE_INACTIVE
        };
        for (mut bg, toggle) in &mut bg_query {
            if toggle.0 == idx {
                *bg = BackgroundColor(new_bg);
            }
        }
    }
}

/// Set every count text node whose question is in `questions` back to "1".
fn reset_count_texts(text_query: &mut Query<(&mut Text, &CountText)>, questions: &[DraftQuestion]) {
    for (mut text, ct) in text_query {
        if questions.iter().any(|q| q.index == ct.0) {
            "1".clone_into(&mut *text);
        }
    }
}

/// Restore each visual-toggle button color to its default for the matching question.
fn reset_visual_toggles(
    toggle_bg_query: &mut Query<
        (&mut BackgroundColor, &VisualToggleButton),
        Without<SaveConfigButton>,
    >,
    questions: &[DraftQuestion],
) {
    for (mut bg, toggle) in toggle_bg_query {
        if let Some(q) = questions
            .iter()
            .find(|q| q.index == toggle.0 && q.has_visual)
        {
            *bg = BackgroundColor(if q.default_show_visual {
                theme::colors::PRIMARY
            } else {
                theme::colors::TOGGLE_INACTIVE
            });
        }
    }
}

/// Resets all question counts to the default value (1) and visual toggles to their defaults.
pub(super) fn handle_reset_config(
    query: Query<&Interaction, (Changed<Interaction>, With<ResetConfigButton>)>,
    mut lessons_state: ResMut<TeacherLessonsState>,
    mut text_query: Query<(&mut Text, &CountText)>,
    mut save_btn_query: Query<&mut BackgroundColor, With<SaveConfigButton>>,
    mut toggle_bg_query: Query<
        (&mut BackgroundColor, &VisualToggleButton),
        Without<SaveConfigButton>,
    >,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let LessonsView::Config {
            ref mut editing, ..
        } = lessons_state.view
        else {
            continue;
        };
        for q in &mut editing.questions {
            q.count = 1;
            if q.has_visual {
                q.show_visual = q.default_show_visual;
            }
        }
        reset_count_texts(&mut text_query, &editing.questions);
        reset_visual_toggles(&mut toggle_bg_query, &editing.questions);
        update_save_button_state(&mut save_btn_query, editing);
    }
}

/// Shows/hides the scroll indicator based on content overflow.
pub(super) fn update_scroll_indicator(
    frame_node: Single<&ComputedNode, With<ScrollFrame>>,
    content_node: Single<&ComputedNode, With<ScrollContent>>,
    mut indicator_query: Query<&mut Visibility, With<ScrollIndicator>>,
) {
    let has_overflow = content_node.unrounded_size().y > frame_node.unrounded_size().y;
    for mut vis in &mut indicator_query {
        *vis = if has_overflow {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Dynamically truncates question labels to fit the available width,
/// appending "..." when the full prompt is too long.  Reacts to layout
/// changes (e.g. window resize) so the text always uses the space
/// available.  `Overflow::clip()` on the node acts as a safety net.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(super) fn update_question_labels(mut query: Query<(&mut Text, &ComputedNode, &QuestionLabel)>) {
    /// Approximate average character width relative to font size.
    /// `FiraMono` is a monospace font with wider characters than proportional
    /// fonts; 0.6 closely matches its actual advance width.
    const CHAR_WIDTH_FACTOR: f32 = 0.6;
    const BULLET: &str = "\u{2022} ";
    const ELLIPSIS: &str = "...";
    /// Approximate pixel width of the "..." suffix at 1px font.
    const ELLIPSIS_WIDTH_FACTOR: f32 = 1.5;

    let char_width = theme::fonts::BODY * CHAR_WIDTH_FACTOR;
    let ellipsis_width = theme::fonts::BODY * ELLIPSIS_WIDTH_FACTOR;
    let bullet_width = BULLET.chars().count() as f32 * char_width;

    for (mut text, computed, ql) in &mut query {
        let available = computed.size().x;
        // Skip until layout has been computed at least once.
        if available <= 0.0 {
            continue;
        }

        let full = &ql.0;
        let full_with_bullet = format!("{BULLET}{full}");
        let full_width_est = full_with_bullet.chars().count() as f32 * char_width;

        let desired = if full_width_est <= available {
            full_with_bullet
        } else {
            let max_chars = ((available - bullet_width - ellipsis_width) / char_width)
                .max(0.0)
                .floor() as u32 as usize;
            let truncated: String = full.chars().take(max_chars).collect();
            format!("{BULLET}{truncated}{ELLIPSIS}")
        };

        if **text != *desired {
            desired.clone_into(&mut *text);
        }
    }
}

/// Updates the hover detail text when a question row is hovered.
/// Checks every frame (not `Changed<Interaction>`) because child buttons
/// inside the row steal the hover, making the row's `Interaction` flicker.
pub(super) fn update_config_hover_text(
    rows: Query<(&Interaction, &Children), With<QuestionRow>>,
    labels: Query<&QuestionLabel>,
    mut hover_text: Query<&mut Text, With<ConfigHoverText>>,
) {
    let mut hovered_prompt: Option<&str> = None;

    for (interaction, children) in &rows {
        if *interaction == Interaction::Hovered {
            for child in children.iter() {
                if let Ok(ql) = labels.get(child) {
                    hovered_prompt = Some(&ql.0);
                    break;
                }
            }
            if hovered_prompt.is_some() {
                break;
            }
        }
    }

    for mut text in &mut hover_text {
        match hovered_prompt {
            Some(prompt) => {
                if **text != *prompt {
                    prompt.clone_into(&mut *text);
                }
            }
            None => {
                if !text.is_empty() {
                    text.clear();
                }
            }
        }
    }
}

fn update_save_button_state(
    query: &mut Query<&mut BackgroundColor, With<SaveConfigButton>>,
    draft: &LessonConfigDraft,
) {
    let can_save = draft.has_any_selected();
    let bg = if can_save {
        theme::colors::SUCCESS
    } else {
        theme::colors::TOGGLE_INACTIVE
    };
    for mut bg_color in query.iter_mut() {
        *bg_color = BackgroundColor(bg);
    }
}

/// Saves the current config draft to persistence and returns to tree view.
pub(super) fn handle_save_config(
    query: Query<&Interaction, (Changed<Interaction>, With<SaveConfigButton>)>,
    mut commands: Commands,
    lessons_state: Res<TeacherLessonsState>,
    mut save_data: ResMut<Persistent<SaveData>>,
    active_slot: Option<Res<ActiveSlot>>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let LessonsView::Config {
            ref lesson_id,
            ref editing,
            ..
        } = lessons_state.view
        else {
            continue;
        };

        // Don't save if all counts are zero
        if !editing.has_any_selected() {
            continue;
        }

        let counts: Vec<usize> = editing.questions.iter().map(|q| q.count).collect();
        let show_visuals: Vec<bool> = editing.questions.iter().map(|q| q.show_visual).collect();
        let config = LessonSessionConfig {
            counts,
            show_visuals,
        };

        let Some(ref slot) = active_slot else {
            continue;
        };
        let lesson_id = lesson_id.clone();
        save_data
            .update(|data| {
                if let Some(ref mut class_save) = data.class_slots[slot.0] {
                    class_save
                        .lesson_configs
                        .insert(lesson_id.clone(), config.clone());
                }
            })
            .expect("failed to update save data");

        commands.remove_resource::<TeacherInDetailView>();
        commands.trigger(super::RebuildLessons);
    }
}

pub(super) fn handle_return_to_tree(
    query: Query<&Interaction, (Changed<Interaction>, With<ReturnToTreeButton>)>,
    mut commands: Commands,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<TeacherInDetailView>();
            commands.trigger(super::RebuildLessons);
        }
    }
}
