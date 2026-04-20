use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::stacked_fraction;
use crate::ui::theme;

use super::renderer::{ExplanationRenderer, spawn_colored_row, spawn_words};

/// Fraction value (a/b = result): `a` in blue, `b` in orange, `result` in
/// green, matching the whole-fraction bars.
pub(super) struct FractionValueRenderer {
    a: u32,
    b: u32,
    result: u32,
}

impl FractionValueRenderer {
    pub(super) const fn new(a: u32, b: u32, result: u32) -> Self {
        Self { a, b, result }
    }
}

impl ExplanationRenderer for FractionValueRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let font_size = theme::fonts::HEADING;
        let a_color = theme::colors::PRIMARY;
        let b_color = theme::colors::SECONDARY;
        let result_color = theme::colors::SUCCESS;
        let dark = theme::colors::TEXT_DARK;

        let a_str = self.a.to_string();
        let b_str = self.b.to_string();
        let result_str = self.result.to_string();

        let (intro, mid1, mid2, conclusion) = match i18n.language {
            Language::French => (
                format!("{} Si on partage", i18n.t(&TranslationKey::Explanation)),
                "parts en groupes de",
                ", on obtient",
                "groupes. Donc",
            ),
            Language::English => (
                format!("{} If we share", i18n.t(&TranslationKey::Explanation)),
                "parts into groups of",
                ", we get",
                "groups. So",
            ),
        };

        spawn_colored_row(parent, font_size, |row| {
            spawn_words(row, &intro, dark, font_size, window);
            spawn_words(row, &a_str, a_color, font_size, window);
            spawn_words(row, mid1, dark, font_size, window);
            spawn_words(row, &b_str, b_color, font_size, window);
            spawn_words(row, mid2, dark, font_size, window);
            spawn_words(row, &result_str, result_color, font_size, window);
            spawn_words(row, conclusion, dark, font_size, window);
            row.spawn(stacked_fraction(
                self.a, self.b, font_size, a_color, b_color, window,
            ));
            spawn_words(row, "=", dark, font_size, window);
            spawn_words(row, &result_str, result_color, font_size, window);
            spawn_words(row, ".", dark, font_size, window);
        });
    }
}
