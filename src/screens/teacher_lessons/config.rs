use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::data::content::{ContentLibrary, Lesson, MAX_QUESTION_REPETITIONS, QuestionType};
use crate::data::{Language, LessonSessionConfig, PersistenceAction, PlayerSession, SaveDataMut};
use crate::i18n::{I18n, TranslationKey};
use crate::plugins::teacher::{TeacherScreenParam, TeacherViewOverlay};
use crate::screens::teacher_shared::exercise_type_label;
use crate::states::AppState;
use crate::ui::components::{
    ButtonActivated, ButtonVisualStyle, ConfirmationDialogAction, ConfirmationDialogActionEvent,
    HoverTooltip, TeacherListLevel, button_base, disclosure_chevron, disclosure_chevron_transform,
    icon_button, spawn_confirmation_modal, teacher_list_exercise_row, teacher_list_header_border,
    teacher_list_header_node, teacher_list_label, teacher_list_separator_gap,
};
use crate::ui::theme;

#[derive(Resource, Default)]
pub(super) struct ExerciseGroupDisclosureState {
    collapsed: HashSet<(String, QuestionType)>,
}

impl ExerciseGroupDisclosureState {
    fn is_expanded(&self, lesson_id: &str, exercise_type: QuestionType) -> bool {
        !self
            .collapsed
            .contains(&(lesson_id.to_owned(), exercise_type))
    }

    fn toggle(&mut self, lesson_id: &str, exercise_type: QuestionType) -> bool {
        let key = (lesson_id.to_owned(), exercise_type);
        if self.collapsed.remove(&key) {
            true
        } else {
            self.collapsed.insert(key);
            false
        }
    }

    pub(super) fn reset(&mut self) {
        self.collapsed.clear();
    }
}

#[derive(Component, Reflect)]
pub(super) struct ExerciseGroupToggle {
    lesson_id: String,
    exercise_type: QuestionType,
}

#[derive(Component, Reflect)]
pub(super) struct ExerciseGroupPanel {
    lesson_id: String,
    exercise_type: QuestionType,
}

#[derive(Component, Reflect)]
pub(super) struct ExerciseGroupChevron {
    lesson_id: String,
    exercise_type: QuestionType,
}

#[derive(Component, Reflect)]
pub(super) struct ConfigResetButton(pub(super) ConfigResetTarget);

#[derive(Clone, Debug, Reflect)]
pub(super) enum ConfigResetTarget {
    Theme(String),
    Lesson(String),
}

#[derive(Component, Reflect)]
pub(super) struct ConfigResetPopover;

#[derive(Component, Reflect)]
struct ConfigResetRequest(ConfigResetTarget);

#[derive(Component, Reflect)]
pub(super) struct CountButton {
    lesson_id: String,
    exercise_id: String,
    count_text: Entity,
    delta: isize,
}

#[derive(Component, Reflect)]
pub(super) struct CountText;

#[derive(Component, Reflect)]
pub(super) struct VisualToggleButton {
    lesson_id: String,
    exercise_id: String,
    default_show_visual: bool,
}

#[derive(Component, Reflect)]
pub(super) struct ExerciseLabel(String);

#[derive(Clone)]
pub(super) struct ExerciseConfigData {
    id: String,
    exercise_type: QuestionType,
    full_prompt: String,
    count: usize,
    has_visual: bool,
    show_visual: bool,
    default_show_visual: bool,
}

pub(super) struct ExerciseGroupData {
    lesson_id: String,
    exercise_type: QuestionType,
    label: String,
    count_label: String,
    expanded: bool,
    exercises: Vec<ExerciseConfigData>,
}

pub(super) fn build_exercise_configs(
    lesson: &Lesson,
    language: Language,
    existing_config: Option<&LessonSessionConfig>,
) -> Vec<ExerciseConfigData> {
    lesson
        .questions
        .iter()
        .map(|entry| {
            let definition = &entry.definition;
            let default_show_visual = definition.effective_show_visual(None).unwrap_or(false);
            let configured_visual =
                existing_config.and_then(|config| config.show_visuals.get(&entry.id).copied());
            ExerciseConfigData {
                id: entry.id.clone(),
                exercise_type: definition.question_type(),
                full_prompt: definition.prompt_label(language),
                count: existing_config
                    .and_then(|config| config.counts.get(&entry.id).copied())
                    .unwrap_or(1),
                has_visual: definition.has_optional_visual(),
                show_visual: definition
                    .effective_show_visual(configured_visual)
                    .unwrap_or(false),
                default_show_visual,
            }
        })
        .collect()
}

pub(super) fn build_exercise_groups(
    lesson_id: &str,
    exercises: &[ExerciseConfigData],
    i18n: &I18n,
    disclosures: &ExerciseGroupDisclosureState,
) -> Vec<ExerciseGroupData> {
    let mut exercises_by_type = HashMap::<QuestionType, Vec<ExerciseConfigData>>::new();
    for exercise in exercises {
        exercises_by_type
            .entry(exercise.exercise_type)
            .or_default()
            .push(ExerciseConfigData {
                id: exercise.id.clone(),
                exercise_type: exercise.exercise_type,
                full_prompt: exercise.full_prompt.clone(),
                count: exercise.count,
                has_visual: exercise.has_visual,
                show_visual: exercise.show_visual,
                default_show_visual: exercise.default_show_visual,
            });
    }
    let mut exercise_groups = exercises_by_type.into_iter().collect::<Vec<_>>();
    exercise_groups.sort_by_key(|(exercise_type, _)| exercise_type_sort_order(*exercise_type));
    exercise_groups
        .into_iter()
        .map(|(exercise_type, exercises)| ExerciseGroupData {
            lesson_id: lesson_id.to_owned(),
            exercise_type,
            label: exercise_type_label(exercise_type, i18n),
            count_label: i18n
                .t(&TranslationKey::ExerciseCount(exercises.len()))
                .into_owned(),
            expanded: disclosures.is_expanded(lesson_id, exercise_type),
            exercises,
        })
        .collect()
}

const fn exercise_type_sort_order(exercise_type: QuestionType) -> u8 {
    match exercise_type {
        QuestionType::Mcq => 0,
        QuestionType::Visualization => 1,
        QuestionType::Comparison => 2,
        QuestionType::Identification => 3,
        QuestionType::NumericInput => 4,
    }
}

pub(super) fn spawn_lesson_config_panel(
    parent: &mut ChildSpawner,
    groups: Vec<ExerciseGroupData>,
    window: Entity,
) {
    parent.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            ..default()
        },
        Children::spawn(SpawnIter(
            groups
                .into_iter()
                .map(move |group| exercise_group_bundle(group, window)),
        )),
    ));
}

fn exercise_group_bundle(group: ExerciseGroupData, window: Entity) -> impl Bundle + use<> {
    let lesson_id = group.lesson_id;
    let exercise_type = group.exercise_type;
    let expanded = group.expanded;
    let label = group.label;
    let count_label = group.count_label;
    let exercises = group.exercises;
    let button_lesson_id = lesson_id.clone();
    let panel_lesson_id = lesson_id.clone();
    let chevron_lesson_id = lesson_id.clone();
    (
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            section.spawn((
                button_base(theme::button_styles::DISCLOSURE),
                ExerciseGroupToggle {
                    lesson_id: button_lesson_id,
                    exercise_type,
                },
                Node {
                    width: percent(100.0),
                    ..teacher_list_header_node()
                },
                teacher_list_header_border(),
                Children::spawn(SpawnWith(move |header: &mut ChildSpawner| {
                    header.spawn((
                        disclosure_chevron(expanded, theme::colors::TEXT_DARK, window),
                        ExerciseGroupChevron {
                            lesson_id: chevron_lesson_id,
                            exercise_type,
                        },
                    ));
                    header.spawn((
                        teacher_list_label(
                            label,
                            TeacherListLevel::ExerciseType,
                            theme::colors::TEXT_DARK,
                            window,
                        ),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                    header.spawn((
                        Text::new(count_label),
                        theme::typography::text(theme::fonts::SMALL, window),
                        TextColor(theme::colors::TEXT_MUTED),
                    ));
                })),
            ));
            section.spawn((
                ExerciseGroupPanel {
                    lesson_id: panel_lesson_id,
                    exercise_type,
                },
                Node {
                    display: disclosure_display(expanded),
                    flex_direction: FlexDirection::Column,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    padding: UiRect::top(teacher_list_separator_gap()),
                    ..default()
                },
                Children::spawn(SpawnWith(move |items: &mut ChildSpawner| {
                    for exercise in exercises {
                        spawn_exercise_counter_row(items, &lesson_id, &exercise, window);
                    }
                })),
            ));
        })),
    )
}

const fn disclosure_display(expanded: bool) -> Display {
    if expanded {
        Display::Flex
    } else {
        Display::None
    }
}

pub(super) fn handle_exercise_group_disclosure(
    mut activations: MessageReader<ButtonActivated>,
    toggles: Query<&ExerciseGroupToggle>,
    mut panels: Query<(&ExerciseGroupPanel, &mut Node)>,
    mut chevrons: Query<(&ExerciseGroupChevron, &mut UiTransform)>,
    mut disclosures: ResMut<ExerciseGroupDisclosureState>,
) {
    for event in activations.read() {
        let Ok(toggle) = toggles.get(event.0) else {
            continue;
        };
        let lesson_id = &toggle.lesson_id;
        let exercise_type = toggle.exercise_type;
        let expanded = disclosures.toggle(lesson_id, exercise_type);
        for (panel, mut node) in &mut panels {
            if panel.lesson_id == *lesson_id && panel.exercise_type == exercise_type {
                node.display = disclosure_display(expanded);
            }
        }
        for (chevron, mut transform) in &mut chevrons {
            if chevron.lesson_id == *lesson_id && chevron.exercise_type == exercise_type {
                *transform = disclosure_chevron_transform(expanded);
            }
        }
    }
}

fn spawn_exercise_counter_row(
    parent: &mut ChildSpawner,
    lesson_id: &str,
    exercise: &ExerciseConfigData,
    window: Entity,
) {
    let exercise = exercise.clone();
    let full_prompt = exercise.full_prompt.clone();
    let tooltip = HoverTooltip {
        message: full_prompt.clone(),
        window,
    };
    let lesson_id = lesson_id.to_owned();

    parent.spawn((
        teacher_list_exercise_row(),
        tooltip,
        Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
            row.spawn((
                teacher_list_label(
                    format!("\u{2022} {full_prompt}"),
                    TeacherListLevel::Exercise,
                    theme::colors::TEXT_DARK,
                    window,
                ),
                Node {
                    flex_shrink: 1.0,
                    flex_grow: 1.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                ExerciseLabel(full_prompt),
            ));
            spawn_counter_controls(row, &lesson_id, &exercise, window);
        })),
    ));
}

fn spawn_counter_controls(
    row: &mut ChildSpawner,
    lesson_id: &str,
    exercise: &ExerciseConfigData,
    window: Entity,
) {
    let lesson_id_owned = lesson_id.to_owned();
    let exercise_id_owned = exercise.id.clone();
    let count_str_owned = exercise.count.to_string();
    row.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(0.0),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|controls| {
        if exercise.has_visual {
            let toggle_style = if exercise.show_visual {
                theme::button_styles::ACTION_PRIMARY
            } else {
                theme::button_styles::NEUTRAL
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
                        toggle_style,
                        window,
                    ),
                    VisualToggleButton {
                        lesson_id: lesson_id_owned.clone(),
                        exercise_id: exercise_id_owned.clone(),
                        default_show_visual: exercise.default_show_visual,
                    },
                ));
        }

        let dec_entity = controls.spawn_empty().id();
        let text_entity = controls.spawn_empty().id();
        let inc_entity = controls.spawn_empty().id();

        controls.world_mut().entity_mut(dec_entity).insert((
            counter_button("\u{2212}", theme::button_styles::DANGER, window),
            CountButton {
                lesson_id: lesson_id_owned.clone(),
                exercise_id: exercise_id_owned.clone(),
                count_text: text_entity,
                delta: -1,
            },
        ));
        controls.world_mut().entity_mut(text_entity).insert((
            Text::new(count_str_owned),
            theme::typography::text(theme::fonts::SMALL, window),
            TextColor(theme::colors::TEXT_DARK),
            TextLayout::justify(Justify::Center),
            CountText,
            Node {
                min_width: theme::scaled(24.0),
                ..default()
            },
        ));
        controls.world_mut().entity_mut(inc_entity).insert((
            counter_button("+", theme::button_styles::ACTION_SECONDARY, window),
            CountButton {
                lesson_id: lesson_id_owned,
                exercise_id: exercise_id_owned,
                count_text: text_entity,
                delta: 1,
            },
        ));
    });
}

fn counter_button(label: &str, palette: theme::ButtonPalette, window: Entity) -> impl Bundle {
    icon_button(30.0, 6.0, label, theme::fonts::SMALL, palette, window)
}

pub(super) fn handle_count_change(
    mut activations: MessageReader<ButtonActivated>,
    buttons: Query<&CountButton>,
    mut count_texts: Query<&mut Text, With<CountText>>,
    content: Res<ContentLibrary>,
    mut persistence: SaveDataMut<'_>,
    session: Option<Res<PlayerSession>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    let Some(session) = session else {
        return;
    };
    for entity in activated_entities {
        let Ok(button) = buttons.get(entity) else {
            continue;
        };
        let Some(class_save) = persistence
            .save_data
            .class_slots
            .get(session.slot_index)
            .and_then(Option::as_ref)
        else {
            continue;
        };
        let config = class_save.lesson_configs.get(&button.lesson_id);
        let current_count = config
            .and_then(|config| config.counts.get(&button.exercise_id).copied())
            .unwrap_or(1);
        let Some(new_count) = current_count.checked_add_signed(button.delta) else {
            continue;
        };
        if new_count > MAX_QUESTION_REPETITIONS || new_count == current_count {
            continue;
        }
        if new_count == 0
            && !lesson_has_other_selected_exercise(
                &content,
                &button.lesson_id,
                &button.exercise_id,
                config,
            )
        {
            continue;
        }
        let lesson_id = button.lesson_id.clone();
        let exercise_id = button.exercise_id.clone();
        let count_text_entity = button.count_text;
        persistence.update(PersistenceAction::TeacherLessonConfig, |data| {
            if let Some(class_save) = data
                .class_slots
                .get_mut(session.slot_index)
                .and_then(Option::as_mut)
            {
                set_exercise_count(class_save, &lesson_id, &exercise_id, new_count);
            }
        });
        if let Ok(mut text) = count_texts.get_mut(count_text_entity) {
            **text = new_count.to_string();
        }
    }
}

fn lesson_has_other_selected_exercise(
    content: &ContentLibrary,
    lesson_id: &str,
    exercise_id: &str,
    config: Option<&LessonSessionConfig>,
) -> bool {
    content
        .themes
        .iter()
        .flat_map(|theme| &theme.lessons)
        .find(|lesson| lesson.id == lesson_id)
        .is_some_and(|lesson| {
            lesson.questions.iter().any(|entry| {
                entry.id != exercise_id
                    && config
                        .and_then(|config| config.counts.get(&entry.id).copied())
                        .unwrap_or(1)
                        > 0
            })
        })
}

fn set_exercise_count(
    class_save: &mut crate::data::ClassSave,
    lesson_id: &str,
    exercise_id: &str,
    count: usize,
) {
    let config = class_save
        .lesson_configs
        .entry(lesson_id.to_owned())
        .or_insert_with(empty_lesson_config);
    if count == 1 {
        config.counts.remove(exercise_id);
    } else {
        config.counts.insert(exercise_id.to_owned(), count);
    }
    remove_empty_lesson_config(class_save, lesson_id);
}

pub(super) fn handle_visual_toggle(
    mut activations: MessageReader<ButtonActivated>,
    buttons: Query<&VisualToggleButton>,
    mut styles: Query<(&mut ButtonVisualStyle, &VisualToggleButton)>,
    mut persistence: SaveDataMut<'_>,
    session: Option<Res<PlayerSession>>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    let Some(session) = session else {
        return;
    };
    for entity in activated_entities {
        let Ok(button) = buttons.get(entity) else {
            continue;
        };
        let Some(class_save) = persistence
            .save_data
            .class_slots
            .get(session.slot_index)
            .and_then(Option::as_ref)
        else {
            continue;
        };
        let current_visual = class_save
            .lesson_configs
            .get(&button.lesson_id)
            .and_then(|config| config.show_visuals.get(&button.exercise_id).copied())
            .unwrap_or(button.default_show_visual);
        let new_visual = !current_visual;
        let lesson_id = button.lesson_id.clone();
        let exercise_id = button.exercise_id.clone();
        let default_show_visual = button.default_show_visual;
        persistence.update(PersistenceAction::TeacherLessonConfig, |data| {
            if let Some(class_save) = data
                .class_slots
                .get_mut(session.slot_index)
                .and_then(Option::as_mut)
            {
                set_exercise_visual(
                    class_save,
                    &lesson_id,
                    &exercise_id,
                    default_show_visual,
                    new_visual,
                );
            }
        });
        for (mut style, toggle) in &mut styles {
            if toggle.lesson_id == lesson_id && toggle.exercise_id == exercise_id {
                style.0 = if new_visual {
                    theme::button_styles::ACTION_PRIMARY
                } else {
                    theme::button_styles::NEUTRAL
                };
            }
        }
    }
}

fn set_exercise_visual(
    class_save: &mut crate::data::ClassSave,
    lesson_id: &str,
    exercise_id: &str,
    default_show_visual: bool,
    show_visual: bool,
) {
    let config = class_save
        .lesson_configs
        .entry(lesson_id.to_owned())
        .or_insert_with(empty_lesson_config);
    if show_visual == default_show_visual {
        config.show_visuals.remove(exercise_id);
    } else {
        config
            .show_visuals
            .insert(exercise_id.to_owned(), show_visual);
    }
    remove_empty_lesson_config(class_save, lesson_id);
}

fn empty_lesson_config() -> LessonSessionConfig {
    LessonSessionConfig {
        counts: HashMap::new(),
        show_visuals: HashMap::new(),
    }
}

fn remove_empty_lesson_config(class_save: &mut crate::data::ClassSave, lesson_id: &str) {
    if class_save
        .lesson_configs
        .get(lesson_id)
        .is_some_and(|config| config.counts.is_empty() && config.show_visuals.is_empty())
    {
        class_save.lesson_configs.remove(lesson_id);
    }
}

pub(super) fn handle_reset_click(
    mut commands: Commands,
    mut activations: MessageReader<ButtonActivated>,
    buttons: Query<&ConfigResetButton>,
    existing_popovers: Query<Entity, With<ConfigResetPopover>>,
    content: Res<ContentLibrary>,
    teacher: TeacherScreenParam<'_, '_>,
) {
    let activated_entities: Vec<Entity> = activations.read().map(|event| event.0).collect();
    let Ok(window) = teacher.teacher.window.single() else {
        return;
    };
    let Ok(camera) = teacher.teacher.camera.single() else {
        return;
    };
    for entity in activated_entities {
        let Ok(button) = buttons.get(entity) else {
            continue;
        };
        for popover in &existing_popovers {
            commands.entity(popover).try_despawn();
        }
        let confirm_message = match &button.0 {
            ConfigResetTarget::Theme(theme_id) => {
                let title = content
                    .themes
                    .iter()
                    .find(|theme| theme.id == *theme_id)
                    .map_or_else(
                        || theme_id.clone(),
                        |theme| theme.title.get(teacher.i18n.language).to_owned(),
                    );
                teacher
                    .i18n
                    .t(&TranslationKey::ResetThemeConfigConfirm(title))
                    .into_owned()
            }
            ConfigResetTarget::Lesson(lesson_id) => teacher
                .i18n
                .t(&TranslationKey::ResetLessonConfigConfirm(lesson_label(
                    lesson_id,
                    &content,
                    &teacher.i18n,
                )))
                .into_owned(),
        };
        let modal = spawn_confirmation_modal(
            &mut commands,
            &confirm_message,
            &teacher.i18n.t(&TranslationKey::Delete),
            &teacher.i18n.t(&TranslationKey::Cancel),
            theme::button_styles::DANGER,
            window,
            Some(camera),
        );
        commands
            .entity(modal)
            .insert((
                ConfigResetPopover,
                ConfigResetRequest(button.0.clone()),
                TeacherViewOverlay,
                DespawnOnExit(AppState::ThemeExploration),
            ))
            .observe(handle_confirm_config_reset);
    }
}

fn lesson_label(lesson_id: &str, content: &ContentLibrary, i18n: &I18n) -> String {
    content
        .themes
        .iter()
        .flat_map(|theme| &theme.lessons)
        .find(|lesson| lesson.id == lesson_id)
        .map_or_else(
            || lesson_id.to_owned(),
            |lesson| lesson.title.get(i18n.language).to_owned(),
        )
}

fn handle_confirm_config_reset(
    event: On<ConfirmationDialogActionEvent>,
    requests: Query<&ConfigResetRequest>,
    content: Res<ContentLibrary>,
    session: Option<Res<PlayerSession>>,
    mut persistence: SaveDataMut<'_>,
) {
    if event.action != ConfirmationDialogAction::Confirm {
        return;
    }
    let Some(session) = session else {
        return;
    };
    let Ok(request) = requests.get(event.entity) else {
        return;
    };
    let target = request.0.clone();
    persistence.update(PersistenceAction::TeacherLessonConfig, |data| {
        let Some(class_save) = data
            .class_slots
            .get_mut(session.slot_index)
            .and_then(Option::as_mut)
        else {
            return;
        };
        match &target {
            ConfigResetTarget::Theme(theme_id) => {
                if let Some(theme) = content.themes.iter().find(|theme| theme.id == *theme_id) {
                    remove_lesson_configs(
                        &mut class_save.lesson_configs,
                        theme.lessons.iter().map(|lesson| lesson.id.as_str()),
                    );
                }
            }
            ConfigResetTarget::Lesson(lesson_id) => {
                remove_lesson_configs(
                    &mut class_save.lesson_configs,
                    std::iter::once(lesson_id.as_str()),
                );
            }
        }
    });
}

fn remove_lesson_configs<'a>(
    lesson_configs: &mut HashMap<String, LessonSessionConfig>,
    lesson_ids: impl IntoIterator<Item = &'a str>,
) {
    for lesson_id in lesson_ids {
        lesson_configs.remove(lesson_id);
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(super) fn update_exercise_labels(
    mut query: Query<(&mut Text, &ComputedNode, &TextFont, &ExerciseLabel)>,
) {
    const CHAR_WIDTH_FACTOR: f32 = 0.6;
    const BULLET: &str = "\u{2022} ";
    const ELLIPSIS: &str = "...";
    const ELLIPSIS_WIDTH_FACTOR: f32 = 1.5;

    for (mut text, computed, font, label) in &mut query {
        let FontSize::Px(font_size) = font.font_size else {
            continue;
        };
        let available = computed.size().x * computed.inverse_scale_factor;
        if available <= 0.0 {
            continue;
        }
        let char_width = font_size * CHAR_WIDTH_FACTOR;
        let ellipsis_width = font_size * ELLIPSIS_WIDTH_FACTOR;
        let bullet_width = BULLET.chars().count() as f32 * char_width;
        let full = &label.0;
        let full_with_bullet = format!("{BULLET}{full}");
        let full_width_est = full_with_bullet.chars().count() as f32 * char_width;
        let desired = if full_width_est <= available {
            full_with_bullet
        } else {
            let max_chars = ((available - bullet_width - ellipsis_width) / char_width)
                .max(0.0)
                .floor() as usize;
            let truncated: String = full.chars().take(max_chars).collect();
            format!("{BULLET}{truncated}{ELLIPSIS}")
        };

        if **text != *desired {
            desired.clone_into(&mut *text);
        }
    }
}
