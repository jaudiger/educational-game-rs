use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use crate::ui::components::{HoverTooltip, list_separator_border};
use crate::ui::theme;

const TEACHER_LIST_ROW_HEIGHT: f32 = 36.0;

#[derive(Clone, Copy)]
pub enum TeacherListLevel {
    Theme,
    Lesson,
    ExerciseType,
    Exercise,
}

impl TeacherListLevel {
    const fn indent(self) -> f32 {
        match self {
            Self::Theme => 0.0,
            Self::Lesson => theme::spacing::SMALL,
            Self::ExerciseType => theme::spacing::MEDIUM,
            Self::Exercise => theme::spacing::LARGE,
        }
    }

    const fn font_size(self) -> f32 {
        match self {
            Self::Theme | Self::Lesson => theme::fonts::BODY,
            Self::ExerciseType | Self::Exercise => theme::fonts::SMALL,
        }
    }
}

pub fn teacher_list_label(
    label: String,
    level: TeacherListLevel,
    color: Color,
    window: Entity,
) -> impl Bundle + use<> {
    (
        Text::new(label),
        theme::typography::text(level.font_size(), window),
        TextColor(color),
    )
}

pub fn teacher_list_indent(parent: TeacherListLevel, child: TeacherListLevel) -> Val {
    theme::scaled(child.indent() - parent.indent())
}

pub fn teacher_list_separator_gap() -> Val {
    theme::scaled(theme::spacing::SMALL)
}

pub fn teacher_list_header_node() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: theme::scaled(theme::spacing::SMALL),
        min_height: theme::scaled(TEACHER_LIST_ROW_HEIGHT),
        padding: UiRect::bottom(theme::scaled(theme::spacing::SMALL)),
        border: px(1.0).bottom(),
        ..default()
    }
}

pub fn teacher_list_header_button_node() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: theme::scaled(theme::spacing::SMALL),
        flex_grow: 1.0,
        min_width: px(0.0),
        min_height: theme::scaled(TEACHER_LIST_ROW_HEIGHT),
        ..default()
    }
}

pub fn teacher_list_header_border() -> BorderColor {
    list_separator_border()
}

pub fn teacher_list_exercise_row() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: theme::scaled(theme::spacing::MEDIUM),
        margin: teacher_list_indent(TeacherListLevel::ExerciseType, TeacherListLevel::Exercise)
            .left(),
        min_height: theme::scaled(36.0),
        ..default()
    }
}

pub fn teacher_list_frame() -> impl Bundle + use<> {
    (
        ScrollArea,
        Node {
            width: percent(100.0),
            flex_grow: 1.0,
            flex_basis: px(0.0),
            min_height: px(0.0),
            border: px(1.0).all(),
            border_radius: BorderRadius::all(theme::scaled(theme::sizes::CARD_BORDER_RADIUS)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(theme::colors::CARD_BG),
        BorderColor::all(theme::colors::INPUT_BORDER),
    )
}

pub fn teacher_list_content_node() -> Node {
    Node {
        width: percent(100.0),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        row_gap: theme::scaled(theme::spacing::MEDIUM),
        padding: theme::scaled(theme::spacing::MEDIUM).all(),
        ..default()
    }
}

#[derive(Component, Reflect)]
pub struct TeacherListTruncatedLabel {
    label: String,
    prefix: String,
    window: Entity,
}

#[derive(Component, Reflect)]
pub struct TeacherListTruncatedLabelText;

pub fn teacher_list_truncated_label(
    label: String,
    prefix: &str,
    level: TeacherListLevel,
    color: Color,
    window: Entity,
) -> impl Bundle + use<> {
    (
        Node {
            min_width: px(0.0),
            flex_basis: px(0.0),
            flex_shrink: 1.0,
            flex_grow: 1.0,
            overflow: Overflow::clip(),
            ..default()
        },
        TeacherListTruncatedLabel {
            label,
            prefix: prefix.to_owned(),
            window,
        },
        children![(
            teacher_list_label(String::new(), level, color, window),
            TextLayout::no_wrap(),
            Node {
                width: percent(100.0),
                ..default()
            },
            TeacherListTruncatedLabelText,
        )],
    )
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn fit_teacher_list_label(
    label: &str,
    prefix: &str,
    available: f32,
    font_size: f32,
) -> (String, bool) {
    const FIRA_MONO_GLYPH_ADVANCE_FACTOR: f32 = 0.6;
    const ELLIPSIS: &str = "...";

    let char_width = font_size * FIRA_MONO_GLYPH_ADVANCE_FACTOR;
    let full = format!("{prefix}{label}");
    let full_width = full.chars().count() as f32 * char_width;
    if full_width <= available {
        return (full, false);
    }

    let prefix_width = prefix.chars().count() as f32 * char_width;
    let ellipsis_width = ELLIPSIS.chars().count() as f32 * char_width;
    if available < prefix_width + ellipsis_width {
        return if available >= ellipsis_width {
            (ELLIPSIS.to_owned(), true)
        } else {
            (String::new(), true)
        };
    }

    let max_chars = ((available - prefix_width - ellipsis_width) / char_width).floor() as usize;
    let visible: String = label.chars().take(max_chars).collect();
    (format!("{prefix}{visible}{ELLIPSIS}"), true)
}

#[allow(clippy::type_complexity)]
pub fn update_teacher_list_truncated_labels(
    mut commands: Commands,
    mut text_query: Query<(&mut Text, &TextFont, &ChildOf), With<TeacherListTruncatedLabelText>>,
    labels: Query<(
        Entity,
        &ComputedNode,
        &TeacherListTruncatedLabel,
        Option<&HoverTooltip>,
    )>,
) {
    for (mut text, font, parent) in &mut text_query {
        let Ok((entity, computed, label, tooltip)) = labels.get(parent.parent()) else {
            continue;
        };
        let FontSize::Px(font_size) = font.font_size else {
            continue;
        };
        let available = computed.size().x * computed.inverse_scale_factor;
        if available <= 0.0 {
            continue;
        }
        let (display_text, is_truncated) =
            fit_teacher_list_label(&label.label, &label.prefix, available, font_size);

        if **text != *display_text {
            display_text.clone_into(&mut *text);
        }

        if is_truncated && tooltip.is_none() {
            commands.entity(entity).insert(HoverTooltip {
                message: label.label.clone(),
                window: label.window,
            });
        } else if !is_truncated && tooltip.is_some() {
            commands.entity(entity).remove::<HoverTooltip>();
        }
    }
}
