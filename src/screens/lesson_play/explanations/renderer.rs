use bevy::prelude::*;

use crate::data::content::{
    ExplanationColorRole, ExplanationPlaceholder, ExplanationValue, LocalizedExplanation,
};
use crate::i18n::Language;
use crate::ui::components::stacked_fraction;
use crate::ui::rich_text::spawn_rich_text;
use crate::ui::theme;

use super::super::visuals::PV_ZERO_COLOR;

enum Segment<'a> {
    Text(String),
    Value(&'a ExplanationValue),
}

pub(super) fn spawn_explanation_text(
    parent: &mut ChildSpawnerCommands,
    explanation: &LocalizedExplanation,
    language: Language,
    font_size: f32,
    window: Entity,
) {
    let text = explanation.text.get(language);
    let mut segments = parse_segments(text, &explanation.placeholders);
    if !segments
        .iter()
        .any(|segment| matches!(segment, Segment::Value(_)))
    {
        spawn_rich_text(parent, text, font_size, theme::colors::TEXT_DARK, window);
        return;
    }

    let punctuation = attach_value_punctuation(&mut segments);
    let gap = theme::scaled(font_size * 0.28);
    parent
        .spawn(Node {
            align_self: AlignSelf::Stretch,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|wrapper| {
            wrapper
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: gap,
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    ..default()
                })
                .with_children(|row| {
                    let mut index = 0;
                    while index < segments.len() {
                        match &segments[index] {
                            Segment::Text(text) => {
                                spawn_words(row, text, font_size, window);
                            }
                            Segment::Value(value) => {
                                let value = (*value).clone();
                                let (leading, trailing) = &punctuation[index];
                                if leading.is_empty() && trailing.is_empty() {
                                    spawn_value(row, &value, language, font_size, window);
                                } else {
                                    row.spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        flex_shrink: 0.0,
                                        ..default()
                                    })
                                    .with_children(|group| {
                                        if !leading.is_empty() {
                                            spawn_text(
                                                group,
                                                leading,
                                                theme::colors::TEXT_DARK,
                                                font_size,
                                                window,
                                            );
                                        }
                                        spawn_value(group, &value, language, font_size, window);
                                        if !trailing.is_empty() {
                                            spawn_text(
                                                group,
                                                trailing,
                                                theme::colors::TEXT_DARK,
                                                font_size,
                                                window,
                                            );
                                        }
                                    });
                                }
                            }
                        }
                        index += 1;
                    }
                });
        });
}

fn parse_segments<'a>(text: &str, placeholders: &'a [ExplanationPlaceholder]) -> Vec<Segment<'a>> {
    let mut segments = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            segments.push(Segment::Text(rest[..open].to_owned()));
        }
        let tail = &rest[open + 1..];
        let Some(close) = tail.find('}') else {
            segments.push(Segment::Text(rest[open..].to_owned()));
            return segments;
        };
        let name = &tail[..close];
        let mut matches = placeholders.iter().filter(|value| value.name == name);
        if is_valid_placeholder_name(name)
            && let (Some(placeholder), None) = (matches.next(), matches.next())
        {
            segments.push(Segment::Value(&placeholder.value));
        } else {
            segments.push(Segment::Text(rest[open..open + close + 2].to_owned()));
        }
        rest = &tail[close + 1..];
    }
    if !rest.is_empty() {
        segments.push(Segment::Text(rest.to_owned()));
    }
    segments
}

fn spawn_words(row: &mut ChildSpawnerCommands, text: &str, font_size: f32, window: Entity) {
    for word in text.split_whitespace() {
        spawn_text(row, word, theme::colors::TEXT_DARK, font_size, window);
    }
}

fn spawn_value(
    parent: &mut ChildSpawnerCommands,
    value: &ExplanationValue,
    language: Language,
    font_size: f32,
    window: Entity,
) {
    match value {
        ExplanationValue::Text(text) => {
            spawn_words(parent, text.get(language), font_size, window);
        }
        ExplanationValue::Number { value, role } => {
            let number = value.to_string();
            if let ExplanationColorRole::PlaceValue { highlighted_zeros } = role {
                let zero_count = (*highlighted_zeros).min(number.len());
                if zero_count > 0 {
                    let split = number.len() - zero_count;
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_shrink: 0.0,
                            ..default()
                        })
                        .with_children(|group| {
                            spawn_text(
                                group,
                                &number[..split],
                                theme::colors::TEXT_DARK,
                                font_size,
                                window,
                            );
                            spawn_text(group, &number[split..], PV_ZERO_COLOR, font_size, window);
                        });
                } else {
                    spawn_text(parent, &number, theme::colors::TEXT_DARK, font_size, window);
                }
            } else {
                spawn_text(parent, &number, role_color(*role), font_size, window);
            }
        }
        ExplanationValue::Fraction {
            numerator,
            denominator,
            numerator_role,
            denominator_role,
        } => {
            parent.spawn(stacked_fraction(
                *numerator,
                *denominator,
                font_size,
                role_color(*numerator_role),
                role_color(*denominator_role),
                window,
            ));
        }
    }
}

fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    color: Color,
    font_size: f32,
    window: Entity,
) {
    parent.spawn((
        Text::new(text),
        theme::typography::text(font_size, window),
        TextColor(color),
    ));
}

const fn role_color(role: ExplanationColorRole) -> Color {
    match role {
        ExplanationColorRole::Default => theme::colors::TEXT_DARK,
        ExplanationColorRole::Primary => theme::colors::PRIMARY,
        ExplanationColorRole::Secondary => theme::colors::SECONDARY,
        ExplanationColorRole::Success => theme::colors::SUCCESS,
        ExplanationColorRole::PlaceValue { .. } => PV_ZERO_COLOR,
    }
}

fn attach_value_punctuation(segments: &mut [Segment<'_>]) -> Vec<(String, String)> {
    let mut punctuation = vec![(String::new(), String::new()); segments.len()];
    for (index, attachment) in punctuation.iter_mut().enumerate() {
        if matches!(segments.get(index), Some(Segment::Value(_))) {
            *attachment = (
                take_trailing_punctuation(segments, index),
                take_leading_punctuation(segments, index + 1),
            );
        }
    }
    punctuation
}

fn take_trailing_punctuation(segments: &mut [Segment<'_>], index: usize) -> String {
    let Some(Segment::Text(text)) = index
        .checked_sub(1)
        .and_then(|previous| segments.get_mut(previous))
    else {
        return String::new();
    };
    let length = text
        .chars()
        .rev()
        .take_while(|character| is_explanation_punctuation(*character))
        .map(char::len_utf8)
        .sum::<usize>();
    text.drain(text.len() - length..).collect()
}

fn take_leading_punctuation(segments: &mut [Segment<'_>], index: usize) -> String {
    let Some(Segment::Text(text)) = segments.get_mut(index) else {
        return String::new();
    };
    let length = text
        .chars()
        .take_while(|character| is_explanation_punctuation(*character))
        .map(char::len_utf8)
        .sum::<usize>();
    text.drain(..length).collect()
}

fn is_valid_placeholder_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

const fn is_explanation_punctuation(character: char) -> bool {
    matches!(
        character,
        '?' | '!' | '.' | ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
    )
}
