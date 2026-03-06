//! Rich text utilities for rendering inline stacked fractions.
//!
//! Provides a parser that detects `\d+/\d+` patterns in text and a spawning
//! helper that renders them as stacked fractions (numerator / bar / denominator)
//! while keeping surrounding text as plain [`Text`] entities.

use bevy::prelude::*;

use super::components::stacked_fraction;
use super::theme;
use super::theme::DesignFontSize;

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

/// Spawn rich text that automatically renders `\d+/\d+` patterns as stacked
/// fractions.
///
/// If the text contains no fractions, a simple [`Text`] entity is spawned.
/// Otherwise, a horizontal flex row with mixed text and fraction children is
/// created.
///
/// A full-width centering wrapper ensures the text block wraps at the correct
/// width (the parent's content area) while appearing centered as a whole.
pub fn spawn_rich_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    window: Entity,
) {
    let segments = parse_fraction_segments(text);
    let has_fraction = segments
        .iter()
        .any(|s| matches!(s, TextSegment::Fraction(_, _)));

    if !has_fraction {
        // Fast path: no fractions, single Text entity.
        parent.spawn((
            Text::new(text.to_owned()),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(text_color),
            DesignFontSize {
                size: font_size,
                window,
            },
        ));
        return;
    }

    // Slow path: mixed content, horizontal row with word-level wrapping.
    //
    // A two-layer structure is used:
    //
    //   Outer row:  `width: 100%`, `justify_content: Center`
    //     Inner row: `width: Auto`, `flex_wrap: Wrap`
    //       Word / fraction entities
    //
    // The outer row provides a definite main-axis width so the inner row
    // wraps at the correct boundary.  After wrapping, the inner row sizes
    // to its widest line and is centered by the outer row.  This avoids
    // per-line centering (which `justify_content: Center` on a single row
    // would produce) while still centering the text block as a whole.
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
        Text::new(word.to_owned()),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(text_color),
        DesignFontSize {
            size: font_size,
            window,
        },
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
            Text::new(punct.to_owned()),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(text_color),
            DesignFontSize {
                size: font_size,
                window,
            },
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
