use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::I18n;
use crate::ui::theme;

use super::renderer::{ExplanationRenderer, spawn_colored_row, spawn_words};

/// Same-numerator comparison: the shared numerator is highlighted in blue.
pub(super) struct ComparisonSameNumRenderer {
    numerator: u32,
}

impl ComparisonSameNumRenderer {
    pub(super) const fn new(numerator: u32) -> Self {
        Self { numerator }
    }
}

impl ExplanationRenderer for ComparisonSameNumRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let font_size = theme::fonts::HEADING;
        let num_color = theme::colors::PRIMARY;
        let dark = theme::colors::TEXT_DARK;

        let num_str = self.numerator.to_string();

        let (intro, middle) = match i18n.language {
            Language::French => (
                "Ils mangent le même nombre de parts (",
                "), mais moins le gâteau a de parts, plus elles sont grosses!",
            ),
            Language::English => (
                "They eat the same number of slices (",
                "), but fewer slices in a pie means bigger slices!",
            ),
        };

        spawn_colored_row(parent, font_size, |row| {
            spawn_words(row, intro, dark, font_size, window);
            spawn_words(row, &num_str, num_color, font_size, window);
            spawn_words(row, middle, dark, font_size, window);
        });
    }
}
