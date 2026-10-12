use std::collections::HashSet;

use bevy::prelude::*;

use crate::data::content::{Lesson, Theme};
use crate::data::save::LessonSessionConfig;
use crate::data::{Language, PlayerSession, SaveData};
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::{
    ButtonActivated, TeacherListLevel, button_base, disclosure_chevron,
    disclosure_chevron_transform, icon_button, teacher_list_content_node, teacher_list_frame,
    teacher_list_header_border, teacher_list_header_button_node, teacher_list_header_node,
    teacher_list_indent, teacher_list_label,
};
use crate::ui::theme;

use super::config::{self, ConfigResetButton, ConfigResetTarget, ExerciseGroupDisclosureState};

#[derive(Resource, Default)]
pub(super) struct ThemeDisclosureState {
    collapsed: HashSet<String>,
}

impl ThemeDisclosureState {
    fn is_expanded(&self, theme_id: &str) -> bool {
        !self.collapsed.contains(theme_id)
    }

    fn toggle(&mut self, theme_id: &str) -> bool {
        if self.collapsed.remove(theme_id) {
            true
        } else {
            self.collapsed.insert(theme_id.to_owned());
            false
        }
    }

    pub(super) fn reset(&mut self) {
        self.collapsed.clear();
    }
}

#[derive(Resource, Default)]
pub(super) struct LessonDisclosureState {
    expanded: HashSet<String>,
}

impl LessonDisclosureState {
    fn is_expanded(&self, lesson_id: &str) -> bool {
        self.expanded.contains(lesson_id)
    }

    fn toggle(&mut self, lesson_id: &str) -> bool {
        if self.expanded.remove(lesson_id) {
            false
        } else {
            self.expanded.insert(lesson_id.to_owned());
            true
        }
    }

    pub(super) fn reset(&mut self) {
        self.expanded.clear();
    }
}

#[derive(Component, Reflect)]
pub(super) struct LessonTreeScroll;

#[derive(Component, Reflect)]
pub(super) struct ThemeDisclosureButton(String);

#[derive(Component, Reflect)]
pub(super) struct ThemeDisclosurePanel(String);

#[derive(Component, Reflect)]
pub(super) struct ThemeDisclosureChevron(String);

#[derive(Component, Reflect)]
pub(super) struct LessonDisclosureButton(String);

#[derive(Component, Reflect)]
pub(super) struct LessonDisclosurePanel(String);

#[derive(Component, Reflect)]
pub(super) struct LessonDisclosureChevron(String);

struct LessonTreeRowSpec {
    lesson_id: String,
    title_key: TranslationKey,
    has_custom_config: bool,
    expanded: bool,
    groups: Vec<config::ExerciseGroupData>,
}

pub(super) struct ThemeTreeSpec {
    theme_id: String,
    title_key: TranslationKey,
    available: bool,
    expanded: bool,
    lessons: Vec<LessonTreeRowSpec>,
}

pub(super) fn build_tree_specs(
    themes: &[Theme],
    save_data: &SaveData,
    session: Option<&PlayerSession>,
    language: Language,
    theme_disclosures: &ThemeDisclosureState,
    lesson_disclosures: &LessonDisclosureState,
    exercise_disclosures: &ExerciseGroupDisclosureState,
) -> Vec<ThemeTreeSpec> {
    let class_save = session
        .and_then(|player_session| save_data.class_slots.get(player_session.slot_index))
        .and_then(Option::as_ref);
    let i18n = I18n::new(language);

    themes
        .iter()
        .map(|theme_data| {
            let lessons = if theme_data.available {
                theme_data
                    .lessons
                    .iter()
                    .map(|lesson| {
                        let existing_config =
                            class_save.and_then(|save| save.lesson_configs.get(&lesson.id));
                        let exercises =
                            config::build_exercise_configs(lesson, language, existing_config);
                        LessonTreeRowSpec {
                            lesson_id: lesson.id.clone(),
                            title_key: lesson.title_key.clone(),
                            has_custom_config: lesson_has_custom_config(lesson, existing_config),
                            expanded: lesson_disclosures.is_expanded(&lesson.id),
                            groups: config::build_exercise_groups(
                                &lesson.id,
                                &exercises,
                                &i18n,
                                exercise_disclosures,
                            ),
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            ThemeTreeSpec {
                theme_id: theme_data.id.clone(),
                title_key: theme_data.title_key.clone(),
                available: theme_data.available,
                expanded: theme_disclosures.is_expanded(&theme_data.id),
                lessons,
            }
        })
        .collect()
}

fn lesson_has_custom_config(lesson: &Lesson, config: Option<&LessonSessionConfig>) -> bool {
    config.is_some_and(|config| {
        let count_changed = lesson.questions.iter().any(|entry| {
            config
                .counts
                .get(&entry.id)
                .is_some_and(|&count| count != 1)
        });
        let visual_changed = lesson.questions.iter().any(|entry| {
            entry
                .definition
                .effective_show_visual(config.show_visuals.get(&entry.id).copied())
                != entry.definition.effective_show_visual(None)
        });
        count_changed || visual_changed
    })
}

pub(super) fn spawn_tree_view(
    parent: &mut ChildSpawner,
    themes: Vec<ThemeTreeSpec>,
    i18n: &I18n,
    show_config_controls: bool,
    scroll_position: ScrollPosition,
    window: Entity,
) {
    let i18n = I18n::new(i18n.language);
    parent.spawn((
        teacher_list_frame(),
        LessonTreeScroll,
        scroll_position,
        Children::spawn(SpawnWith(move |scroll: &mut ChildSpawner| {
            scroll.spawn((
                teacher_list_content_node(),
                Children::spawn(SpawnWith(move |list: &mut ChildSpawner| {
                    for theme_data in themes {
                        spawn_theme_section(list, theme_data, &i18n, show_config_controls, window);
                    }
                })),
            ));
        })),
    ));
}

fn spawn_theme_section(
    parent: &mut ChildSpawner,
    theme_data: ThemeTreeSpec,
    i18n: &I18n,
    show_config_controls: bool,
    window: Entity,
) {
    if !theme_data.available {
        spawn_unavailable_theme(parent, &theme_data, i18n, window);
        return;
    }

    let theme_id = theme_data.theme_id;
    let title = i18n.t(&theme_data.title_key).into_owned();
    let expanded = theme_data.expanded;
    let lessons = theme_data.lessons;
    let i18n = I18n::new(i18n.language);
    parent.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            section.spawn(theme_section_header(
                theme_id.clone(),
                title,
                expanded,
                show_config_controls,
                window,
            ));
            section.spawn(theme_section_panel(
                theme_id,
                expanded,
                lessons,
                i18n,
                show_config_controls,
                window,
            ));
        })),
    ));
}

fn spawn_unavailable_theme(
    parent: &mut ChildSpawner,
    theme_data: &ThemeTreeSpec,
    i18n: &I18n,
    window: Entity,
) {
    parent.spawn((
        Node {
            justify_content: JustifyContent::SpaceBetween,
            ..teacher_list_header_node()
        },
        teacher_list_header_border(),
        children![
            teacher_list_label(
                i18n.t(&theme_data.title_key).into_owned(),
                TeacherListLevel::Theme,
                theme::colors::TEXT_MUTED,
                window,
            ),
            (
                Text::new(i18n.t(&TranslationKey::ComingSoon)),
                theme::typography::text(theme::fonts::SMALL, window),
                TextColor(theme::colors::TEXT_MUTED),
            ),
        ],
    ));
}

fn theme_section_header(
    theme_id: String,
    title: String,
    expanded: bool,
    show_config_controls: bool,
    window: Entity,
) -> impl Bundle + use<> {
    let toggle_theme_id = theme_id.clone();
    let chevron_theme_id = theme_id.clone();
    (
        Node {
            width: percent(100.0),
            ..teacher_list_header_node()
        },
        teacher_list_header_border(),
        Children::spawn(SpawnWith(move |header: &mut ChildSpawner| {
            header.spawn((
                button_base(Color::NONE),
                ThemeDisclosureButton(toggle_theme_id),
                teacher_list_header_button_node(),
                Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
                    row.spawn((
                        disclosure_chevron(expanded, theme::colors::TEXT_DARK, window),
                        ThemeDisclosureChevron(chevron_theme_id),
                    ));
                    row.spawn((
                        teacher_list_label(
                            title,
                            TeacherListLevel::Theme,
                            theme::colors::TEXT_DARK,
                            window,
                        ),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                })),
            ));
            if show_config_controls {
                header.spawn((
                    reset_icon_button(window),
                    ConfigResetButton(ConfigResetTarget::Theme(theme_id)),
                ));
            }
        })),
    )
}

fn theme_section_panel(
    theme_id: String,
    expanded: bool,
    lessons: Vec<LessonTreeRowSpec>,
    i18n: I18n,
    show_config_controls: bool,
    window: Entity,
) -> impl Bundle + use<> {
    (
        ThemeDisclosurePanel(theme_id),
        Node {
            display: disclosure_display(expanded),
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            padding: UiRect {
                left: teacher_list_indent(TeacherListLevel::Theme, TeacherListLevel::Lesson),
                top: theme::scaled(theme::spacing::SMALL),
                ..default()
            },
            ..default()
        },
        Children::spawn(SpawnWith(move |items: &mut ChildSpawner| {
            for lesson in lessons {
                spawn_lesson_tree_row(items, lesson, &i18n, show_config_controls, window);
            }
        })),
    )
}

fn spawn_lesson_tree_row(
    parent: &mut ChildSpawner,
    lesson: LessonTreeRowSpec,
    i18n: &I18n,
    show_config_controls: bool,
    window: Entity,
) {
    let title = i18n.t(&lesson.title_key).into_owned();
    let lesson_id = lesson.lesson_id;
    let expanded = lesson.expanded;
    let has_custom_config = lesson.has_custom_config;
    let groups = lesson.groups;
    let panel_lesson_id = lesson_id.clone();

    parent.spawn((
        Node {
            width: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |section: &mut ChildSpawner| {
            section.spawn(lesson_tree_header(
                lesson_id.clone(),
                title,
                expanded,
                has_custom_config,
                show_config_controls,
                window,
            ));
            if show_config_controls {
                section.spawn(lesson_config_panel(
                    panel_lesson_id,
                    expanded,
                    groups,
                    window,
                ));
            }
        })),
    ));
}

fn lesson_tree_header(
    lesson_id: String,
    title: String,
    expanded: bool,
    has_custom_config: bool,
    show_config_controls: bool,
    window: Entity,
) -> impl Bundle + use<> {
    let chevron_lesson_id = lesson_id.clone();
    (
        teacher_list_header_node(),
        teacher_list_header_border(),
        Children::spawn(SpawnWith(move |header: &mut ChildSpawner| {
            if show_config_controls {
                header.spawn((
                    button_base(Color::NONE),
                    LessonDisclosureButton(lesson_id.clone()),
                    teacher_list_header_button_node(),
                    Children::spawn(SpawnWith(move |row: &mut ChildSpawner| {
                        row.spawn((
                            disclosure_chevron(expanded, theme::colors::TEXT_DARK, window),
                            LessonDisclosureChevron(chevron_lesson_id),
                        ));
                        row.spawn((
                            teacher_list_label(
                                title,
                                TeacherListLevel::Lesson,
                                theme::colors::TEXT_DARK,
                                window,
                            ),
                            Node {
                                flex_grow: 1.0,
                                min_width: px(0.0),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                        ));
                        if has_custom_config {
                            row.spawn((
                                Node {
                                    width: px(6.0),
                                    height: px(6.0),
                                    border_radius: BorderRadius::all(px(3.0)),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                BackgroundColor(theme::colors::PRIMARY),
                            ));
                        }
                    })),
                ));
                header.spawn((
                    reset_icon_button(window),
                    ConfigResetButton(ConfigResetTarget::Lesson(lesson_id)),
                ));
            } else {
                header.spawn((
                    teacher_list_label(
                        title,
                        TeacherListLevel::Lesson,
                        theme::colors::TEXT_DARK,
                        window,
                    ),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
                if has_custom_config {
                    header.spawn((
                        Node {
                            width: px(6.0),
                            height: px(6.0),
                            border_radius: BorderRadius::all(px(3.0)),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(theme::colors::PRIMARY),
                    ));
                }
            }
        })),
    )
}

fn lesson_config_panel(
    lesson_id: String,
    expanded: bool,
    groups: Vec<config::ExerciseGroupData>,
    window: Entity,
) -> impl Bundle + use<> {
    (
        LessonDisclosurePanel(lesson_id),
        Node {
            display: disclosure_display(expanded),
            flex_direction: FlexDirection::Column,
            padding: UiRect {
                left: teacher_list_indent(TeacherListLevel::Lesson, TeacherListLevel::ExerciseType),
                top: theme::scaled(theme::spacing::SMALL),
                bottom: theme::scaled(theme::spacing::SMALL),
                ..default()
            },
            row_gap: theme::scaled(theme::spacing::SMALL),
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |panel: &mut ChildSpawner| {
            config::spawn_lesson_config_panel(panel, groups, window);
        })),
    )
}

fn reset_icon_button(window: Entity) -> impl Bundle + use<> {
    icon_button(
        28.0,
        4.0,
        "X",
        theme::fonts::SMALL,
        theme::colors::TOGGLE_INACTIVE,
        theme::colors::TEXT_DARK,
        window,
    )
}

const fn disclosure_display(expanded: bool) -> Display {
    if expanded {
        Display::Flex
    } else {
        Display::None
    }
}

pub(super) fn handle_theme_disclosure(
    mut activations: MessageReader<ButtonActivated>,
    toggles: Query<&ThemeDisclosureButton>,
    mut panels: Query<(&ThemeDisclosurePanel, &mut Node)>,
    mut chevrons: Query<(&ThemeDisclosureChevron, &mut UiTransform)>,
    mut disclosures: ResMut<ThemeDisclosureState>,
) {
    for event in activations.read() {
        let Ok(toggle) = toggles.get(event.0) else {
            continue;
        };
        let theme_id = &toggle.0;
        let expanded = disclosures.toggle(theme_id);
        for (panel, mut node) in &mut panels {
            if panel.0 == *theme_id {
                node.display = disclosure_display(expanded);
            }
        }
        for (chevron, mut transform) in &mut chevrons {
            if chevron.0 == *theme_id {
                *transform = disclosure_chevron_transform(expanded);
            }
        }
    }
}

pub(super) fn handle_lesson_disclosure(
    mut activations: MessageReader<ButtonActivated>,
    toggles: Query<&LessonDisclosureButton>,
    mut panels: Query<(&LessonDisclosurePanel, &mut Node)>,
    mut chevrons: Query<(&LessonDisclosureChevron, &mut UiTransform)>,
    mut disclosures: ResMut<LessonDisclosureState>,
) {
    for event in activations.read() {
        let Ok(toggle) = toggles.get(event.0) else {
            continue;
        };
        let lesson_id = &toggle.0;
        let expanded = disclosures.toggle(lesson_id);
        for (panel, mut node) in &mut panels {
            if panel.0 == *lesson_id {
                node.display = disclosure_display(expanded);
            }
        }
        for (chevron, mut transform) in &mut chevrons {
            if chevron.0 == *lesson_id {
                *transform = disclosure_chevron_transform(expanded);
            }
        }
    }
}
