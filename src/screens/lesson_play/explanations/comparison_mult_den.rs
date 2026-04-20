use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::stacked_fraction;
use crate::ui::theme;

use super::renderer::{ExplanationRenderer, spawn_colored_row, spawn_words};

/// Multiple-denominator comparison: fraction A in blue, fraction B in
/// orange, rendered as coloured stacked fractions.
pub(super) struct ComparisonMultDenRenderer {
    na: u32,
    da: u32,
    nb: u32,
    db: u32,
}

impl ComparisonMultDenRenderer {
    pub(super) const fn new(na: u32, da: u32, nb: u32, db: u32) -> Self {
        Self { na, da, nb, db }
    }
}

impl ExplanationRenderer for ComparisonMultDenRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let font_size = theme::fonts::HEADING;
        let dark = theme::colors::TEXT_DARK;

        let (intro, middle, conclusion) = match i18n.language {
            Language::French => (
                format!("{} Pour comparer", i18n.t(&TranslationKey::Explanation)),
                "et",
                "il faut couper les gâteaux en parts de la même taille.",
            ),
            Language::English => (
                format!("{} To compare", i18n.t(&TranslationKey::Explanation)),
                "and",
                "we need to cut the pies into same-sized slices.",
            ),
        };

        spawn_colored_row(parent, font_size, |row| {
            spawn_words(row, &intro, dark, font_size, window);
            row.spawn(stacked_fraction(
                self.na, self.da, font_size, dark, dark, window,
            ));
            spawn_words(row, middle, dark, font_size, window);
            row.spawn(stacked_fraction(
                self.nb, self.db, font_size, dark, dark, window,
            ));
            spawn_words(row, conclusion, dark, font_size, window);
        });
    }
}
