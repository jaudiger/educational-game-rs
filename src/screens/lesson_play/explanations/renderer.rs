use bevy::prelude::*;

use crate::i18n::{I18n, TranslationKey};
use crate::ui::rich_text::spawn_rich_text;
use crate::ui::theme::{self, DesignFontSize};

/// Renders the text portion of a question explanation. One implementation
/// per explanation variant; dispatch picks the right one from the question
/// definition. Visual elements (fraction bars, grids, place-value tables)
/// are attached separately as `ExplanationVisual`.
pub(super) trait ExplanationRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity);
}

/// Plain text explanation drawn from the question's localized string. Used
/// as the fallback when no colour-coded renderer applies.
pub(super) struct PlainRenderer {
    text: String,
}

impl PlainRenderer {
    pub(super) const fn new(text: String) -> Self {
        Self { text }
    }
}

impl ExplanationRenderer for PlainRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let text = format!("{} {}", i18n.t(&TranslationKey::Explanation), self.text);
        spawn_rich_text(
            parent,
            &text,
            theme::fonts::HEADING,
            theme::colors::TEXT_DARK,
            window,
        );
    }
}

/// Two-layer centering row used by all colour-coded renderers. The outer
/// row stretches to the parent width and centres the inner row; the inner
/// row wraps at word boundaries and sizes to its content.
pub(super) fn spawn_colored_row(
    parent: &mut ChildSpawnerCommands,
    font_size: f32,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) {
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
                    column_gap: theme::scaled(font_size * 0.28),
                    row_gap: theme::scaled(theme::spacing::SMALL),
                    ..default()
                })
                .with_children(build);
        });
}

/// Spawns each whitespace-separated word as a separate `Text` entity so the
/// flex row can wrap at word boundaries.
pub(super) fn spawn_words(
    row: &mut ChildSpawnerCommands,
    text: &str,
    color: Color,
    font_size: f32,
    window: Entity,
) {
    for word in text.split_whitespace() {
        row.spawn((
            Text::new(word),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(color),
            DesignFontSize {
                size: font_size,
                window,
            },
        ));
    }
}
