use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use super::{list_separator_border, theme};

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
