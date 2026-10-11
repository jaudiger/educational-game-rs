//! Rich text utilities for rendering inline stacked fractions.
//!
//! Provides a parser that detects `\d+/\d+` patterns in text and a spawning
//! helper that renders them as stacked fractions (numerator / bar / denominator)
//! while keeping surrounding text as plain [`Text`] entities.

use bevy::prelude::*;

use super::components::stacked_fraction;
use super::theme;

/// A segment of parsed text: either plain text or a fraction.
pub enum TextSegment {
    Plain(String),
    Fraction(u32, u32),
}

/// Parse a string into segments, detecting `\d+/\d+` fraction patterns.
///
/// The parser scans left-to-right and greedily matches sequences of ASCII
/// digits separated by a `/`. Anything that does not match is returned as
/// [`TextSegment::Plain`].
pub fn parse_fraction_segments(text: &str) -> Vec<TextSegment> {
    let mut segments: Vec<TextSegment> = Vec::new();
    let bytes = text.as_bytes();
    let mut pos = 0;
    let len = bytes.len();

    while pos < len {
        // Find next ASCII digit.
        let Some(offset) = bytes[pos..].iter().position(u8::is_ascii_digit) else {
            // No more digits, push the rest as plain text.
            push_plain(&mut segments, &text[pos..]);
            break;
        };
        let digit_start = pos + offset;

        // Push any preceding plain text.
        if digit_start > pos {
            push_plain(&mut segments, &text[pos..digit_start]);
        }

        // Read numerator digits.
        let num_end = bytes[digit_start..]
            .iter()
            .position(|b| !b.is_ascii_digit())
            .map_or(len, |offset| digit_start + offset);
        let num_str = &text[digit_start..num_end];

        // Check for '/' immediately followed by digit(s).
        if num_end < len
            && bytes[num_end] == b'/'
            && num_end + 1 < len
            && bytes[num_end + 1].is_ascii_digit()
        {
            let den_start = num_end + 1;
            let den_end = bytes[den_start..]
                .iter()
                .position(|b| !b.is_ascii_digit())
                .map_or(len, |offset| den_start + offset);
            let den_str = &text[den_start..den_end];

            if let (Ok(n), Ok(d)) = (num_str.parse::<u32>(), den_str.parse::<u32>()) {
                segments.push(TextSegment::Fraction(n, d));
                pos = den_end;
                continue;
            }
        }

        // Not a fraction, push the digits as plain text.
        push_plain(&mut segments, num_str);
        pos = num_end;
    }

    segments
}

/// Merge consecutive plain segments (or append to the last one).
fn push_plain(segments: &mut Vec<TextSegment>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(TextSegment::Plain(last)) = segments.last_mut() {
        last.push_str(text);
    } else {
        segments.push(TextSegment::Plain(text.to_owned()));
    }
}

/// Spawns content inside the shared centered text container.
pub fn spawn_text_block(
    parent: &mut ChildSpawnerCommands,
    spawn_content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent.spawn(text_block_node()).with_children(spawn_content);
}

fn text_block_node() -> Node {
    Node {
        width: percent(TEXT_BLOCK_WIDTH_PERCENT),
        margin: UiRect::horizontal(percent(TEXT_BLOCK_MARGIN_PERCENT)),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

const TEXT_BLOCK_WIDTH_PERCENT: f32 = 90.0;
const TEXT_BLOCK_MARGIN_PERCENT: f32 = 5.0;

/// Spawns a question text block with centered text.
pub fn spawn_question_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    window: Entity,
) {
    spawn_rich_text(
        parent,
        text,
        font_size,
        text_color,
        window,
        TextBlockAlignment::Center,
    );
}

/// Spawns a response text block with left-aligned text.
pub fn spawn_response_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    window: Entity,
) {
    spawn_rich_text(
        parent,
        text,
        font_size,
        text_color,
        window,
        TextBlockAlignment::Left,
    );
}

#[derive(Clone, Copy)]
enum TextBlockAlignment {
    Left,
    Center,
}

impl TextBlockAlignment {
    const fn text_justify(self) -> Justify {
        match self {
            Self::Left => Justify::Left,
            Self::Center => Justify::Center,
        }
    }

    const fn flex_justify(self) -> JustifyContent {
        match self {
            Self::Left => JustifyContent::FlexStart,
            Self::Center => JustifyContent::Center,
        }
    }
}

fn spawn_rich_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    window: Entity,
    alignment: TextBlockAlignment,
) {
    let segments = parse_fraction_segments(text);
    let has_fraction = segments
        .iter()
        .any(|segment| matches!(segment, TextSegment::Fraction(_, _)));

    if !has_fraction {
        spawn_text_block(parent, |block| {
            block.spawn((
                Text::new(text),
                Node {
                    width: percent(100.0),
                    ..default()
                },
                theme::typography::text(font_size, window),
                TextColor(text_color),
                TextLayout::justify(alignment.text_justify()),
            ));
        });
        return;
    }

    let gap = theme::scaled(font_size * 0.28);

    spawn_text_block(parent, |block| {
        block
            .spawn(Node {
                width: percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: alignment.flex_justify(),
                flex_wrap: FlexWrap::Wrap,
                column_gap: gap,
                row_gap: theme::scaled(theme::spacing::SMALL),
                ..default()
            })
            .with_children(|row| {
                spawn_segments(row, &segments, font_size, text_color, window, gap);
            });
    });
}

/// If the segment after `current` is a Plain segment whose first word is
/// trailing punctuation, return that word.
fn find_trailing_punct(segments: &[TextSegment], current: usize) -> Option<String> {
    if let Some(TextSegment::Plain(s)) = segments.get(current + 1) {
        let word = s.split_whitespace().next()?;
        if is_trailing_punctuation(word) {
            return Some(word.to_owned());
        }
    }
    None
}

/// Spawn a single word as a `Text` entity into the wrapping row.
fn spawn_word(
    row: &mut ChildSpawnerCommands,
    word: &str,
    font_size: f32,
    text_color: Color,
    window: Entity,
) {
    row.spawn((
        Text::new(word),
        theme::typography::text(font_size, window),
        TextColor(text_color),
    ));
}

/// Wraps a fraction and its trailing punctuation in a nowrap flex group.
#[allow(clippy::too_many_arguments)]
fn spawn_fraction_with_punct(
    row: &mut ChildSpawnerCommands,
    n: u32,
    d: u32,
    punct: &str,
    font_size: f32,
    text_color: Color,
    window: Entity,
    gap: Val,
) {
    row.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: gap,
        ..default()
    })
    .with_children(|group| {
        group.spawn(stacked_fraction(
            n, d, font_size, text_color, text_color, window,
        ));
        group.spawn((
            Text::new(punct),
            theme::typography::text(font_size, window),
            TextColor(text_color),
        ));
    });
}

/// Spawn mixed text and fraction entities into a wrapping row.
///
/// Plain text is split into individual word entities.  When a fraction is
/// immediately followed by trailing punctuation (e.g. `?`), both are wrapped
/// in a nowrap group so they cannot be separated by line breaks.
fn spawn_segments(
    row: &mut ChildSpawnerCommands,
    segments: &[TextSegment],
    font_size: f32,
    text_color: Color,
    window: Entity,
    gap: Val,
) {
    // Track whether the first word of the next Plain segment was already
    // consumed as trailing punctuation of a preceding fraction.
    let mut skip_first_word = false;

    for (i, segment) in segments.iter().enumerate() {
        match segment {
            TextSegment::Plain(s) if !s.is_empty() => {
                let mut words = s.split_whitespace();
                if skip_first_word {
                    words.next();
                    skip_first_word = false;
                }
                for word in words {
                    spawn_word(row, word, font_size, text_color, window);
                }
            }
            TextSegment::Fraction(n, d) => {
                if let Some(punct) = find_trailing_punct(segments, i) {
                    skip_first_word = true;
                    spawn_fraction_with_punct(
                        row, *n, *d, &punct, font_size, text_color, window, gap,
                    );
                } else {
                    row.spawn(stacked_fraction(
                        *n, *d, font_size, text_color, text_color, window,
                    ));
                }
            }
            TextSegment::Plain(_) => {}
        }
    }
}

/// Returns `true` when a word consists entirely of punctuation characters
/// that should stay attached to the preceding fraction.
fn is_trailing_punctuation(word: &str) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|c| matches!(c, '?' | '!' | '.' | ',' | ';' | ':' | ')' | ']' | '»'))
}
