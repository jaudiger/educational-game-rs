use bevy::prelude::*;

use crate::data::content::{Lesson, Theme};
use crate::data::{ActiveSlot, SaveData};
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::icon_button;
use crate::ui::theme;
use crate::ui::theme::DesignFontSize;

use super::ConfigLessonButton;

pub(super) fn spawn_tree_view(
    parent: &mut ChildSpawner,
    themes: &[Theme],
    i18n: &I18n,
    show_config_buttons: bool,
    save_data: &SaveData,
    active_slot: Option<&ActiveSlot>,
    window: Entity,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: theme::scaled(theme::spacing::SMALL),
            flex_grow: 1.0,
            overflow: Overflow::scroll_y(),
            ..default()
        })
        .with_children(|list| {
            for theme_data in themes {
                let theme_color = if theme_data.available {
                    theme::colors::TEXT_DARK
                } else {
                    theme::colors::TEXT_MUTED
                };

                list.spawn((
                    Text::new(i18n.t(&theme_data.title_key)),
                    TextFont {
                        font_size: theme::fonts::BODY,
                        ..default()
                    },
                    TextColor(theme_color),
                    Node {
                        margin: theme::scaled(theme::spacing::SMALL).top(),
                        ..default()
                    },
                    DesignFontSize {
                        size: theme::fonts::BODY,
                        window,
                    },
                ));

                if !theme_data.available {
                    list.spawn((
                        Text::new(i18n.t(&TranslationKey::ComingSoon)),
                        TextFont {
                            font_size: theme::fonts::SMALL,
                            ..default()
                        },
                        TextColor(theme::colors::TEXT_MUTED),
                        Node {
                            margin: theme::scaled(theme::spacing::MEDIUM).left(),
                            ..default()
                        },
                        DesignFontSize {
                            size: theme::fonts::SMALL,
                            window,
                        },
                    ));
                    continue;
                }

                for lesson in &theme_data.lessons {
                    spawn_lesson_tree_row(
                        list,
                        &theme_data.id,
                        lesson,
                        i18n,
                        show_config_buttons,
                        save_data,
                        active_slot,
                        window,
                    );
                }
            }
        });
}

fn lesson_has_custom_config(
    lesson: &Lesson,
    save_data: &SaveData,
    active_slot: Option<&ActiveSlot>,
) -> bool {
    active_slot
        .and_then(|slot| save_data.class_slots[slot.0].as_ref())
        .and_then(|cs| cs.lesson_configs.get(&lesson.id))
        .is_some_and(|config| {
            let count_changed = config.counts.iter().any(|&c| c != 1);
            let visual_changed = lesson.questions.iter().enumerate().any(|(i, q)| {
                q.has_optional_visual()
                    && config
                        .show_visuals
                        .get(i)
                        .is_some_and(|&v| v != q.default_show_visual())
            });
            count_changed || visual_changed
        })
}

#[allow(clippy::too_many_arguments)]
fn spawn_lesson_tree_row(
    parent: &mut ChildSpawner,
    theme_id: &str,
    lesson: &Lesson,
    i18n: &I18n,
    show_config_button: bool,
    save_data: &SaveData,
    active_slot: Option<&ActiveSlot>,
    window: Entity,
) {
    let is_available = lesson.available;
    let text_color = if is_available {
        theme::colors::TEXT_DARK
    } else {
        theme::colors::TEXT_MUTED
    };

    let has_config = lesson_has_custom_config(lesson, save_data, active_slot);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: theme::scaled(theme::spacing::MEDIUM).horizontal(),
            min_height: theme::scaled(36.0),
            ..default()
        })
        .with_children(|row| {
            let label = if is_available {
                i18n.t(&lesson.title_key).into_owned()
            } else {
                format!(
                    "{} {}",
                    i18n.t(&lesson.title_key),
                    i18n.t(&TranslationKey::ComingSoon)
                )
            };

            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: theme::fonts::BODY,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    flex_shrink: 1.0,
                    flex_grow: 1.0,
                    overflow: Overflow::clip(),
                    ..default()
                },
                DesignFontSize {
                    size: theme::fonts::BODY,
                    window,
                },
            ));

            if show_config_button && is_available {
                let gear_bg = if has_config {
                    theme::colors::PRIMARY
                } else {
                    theme::colors::TOGGLE_INACTIVE
                };
                let gear_text_color = if has_config {
                    theme::colors::TEXT_LIGHT
                } else {
                    theme::colors::TEXT_DARK
                };

                row.spawn((
                    icon_button(
                        36.0,
                        6.0,
                        "\u{2261}",
                        theme::fonts::SMALL,
                        gear_bg,
                        gear_text_color,
                        window,
                    ),
                    ConfigLessonButton {
                        theme_id: theme_id.to_owned(),
                        lesson_id: lesson.id.clone(),
                    },
                ));
            }
        });
}
