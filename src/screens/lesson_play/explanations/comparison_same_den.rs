use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::{I18n, TranslationKey};
use crate::ui::theme;

use super::renderer::{ExplanationRenderer, spawn_colored_row, spawn_words};

/// Same-denominator comparison: `na` in blue (character A), `nb` in orange
/// (character B), matching the fraction bar colours.
#[allow(clippy::similar_names)]
pub(super) struct ComparisonSameDenRenderer {
    na: u32,
    nb: u32,
    denominator: u32,
}

impl ComparisonSameDenRenderer {
    #[allow(clippy::similar_names)]
    pub(super) const fn new(na: u32, nb: u32, denominator: u32) -> Self {
        Self {
            na,
            nb,
            denominator,
        }
    }
}

impl ExplanationRenderer for ComparisonSameDenRenderer {
    #[allow(clippy::similar_names)]
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let font_size = theme::fonts::HEADING;
        let a_color = theme::colors::PRIMARY;
        let b_color = theme::colors::SECONDARY;
        let dark = theme::colors::TEXT_DARK;

        let na_str = self.na.to_string();
        let nb_str = self.nb.to_string();
        let den_str = self.denominator.to_string();

        let (intro, middle, end) = match i18n.language {
            Language::French => (
                format!(
                    "{} Les gâteaux ont les mêmes parts ({den_str}), on compare juste le nombre de parts :",
                    i18n.t(&TranslationKey::Explanation)
                ),
                "et",
                ".",
            ),
            Language::English => (
                format!(
                    "{} The pies have the same slices ({den_str}), so we compare how many slices:",
                    i18n.t(&TranslationKey::Explanation)
                ),
                "and",
                ".",
            ),
        };

        spawn_colored_row(parent, font_size, |row| {
            spawn_words(row, &intro, dark, font_size, window);
            spawn_words(row, &na_str, a_color, font_size, window);
            spawn_words(row, middle, dark, font_size, window);
            spawn_words(row, &nb_str, b_color, font_size, window);
            spawn_words(row, end, dark, font_size, window);
        });
    }
}
