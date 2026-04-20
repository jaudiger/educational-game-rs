use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::stacked_fraction;
use crate::ui::theme::{self, DesignFontSize};

use super::renderer::{ExplanationRenderer, spawn_colored_row, spawn_words};

/// Fraction addition: `a` in blue, `c` in orange, sum/result in green,
/// matching the three fraction bars in the visual.
pub(super) struct FractionAdditionRenderer {
    a: u32,
    b: u32,
    c: u32,
}

impl FractionAdditionRenderer {
    pub(super) const fn new(a: u32, b: u32, c: u32) -> Self {
        Self { a, b, c }
    }
}

impl ExplanationRenderer for FractionAdditionRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let font_size = theme::fonts::HEADING;
        let a_color = theme::colors::PRIMARY;
        let c_color = theme::colors::SECONDARY;
        let sum_color = theme::colors::SUCCESS;
        let dark = theme::colors::TEXT_DARK;

        let sum = self.a + self.c;
        let a_str = self.a.to_string();
        let b_str = self.b.to_string();
        let c_str = self.c.to_string();
        let sum_str = sum.to_string();

        let (intro, conclusion) = match i18n.language {
            Language::French => (
                format!(
                    "{} Les deux fractions ont le même dénominateur ({b_str}), on additionne les numérateurs :",
                    i18n.t(&TranslationKey::Explanation)
                ),
                "Donc",
            ),
            Language::English => (
                format!(
                    "{} Both fractions have the same denominator ({b_str}), so we add the numerators:",
                    i18n.t(&TranslationKey::Explanation)
                ),
                "So",
            ),
        };

        spawn_colored_row(parent, font_size, |row| {
            spawn_words(row, &intro, dark, font_size, window);
            spawn_words(row, &a_str, a_color, font_size, window);
            spawn_words(row, "+", dark, font_size, window);
            spawn_words(row, &c_str, c_color, font_size, window);
            spawn_words(row, "=", dark, font_size, window);
            spawn_words(row, &sum_str, sum_color, font_size, window);
            spawn_words(row, ".", dark, font_size, window);
            spawn_words(row, conclusion, dark, font_size, window);
            row.spawn(stacked_fraction(
                self.a, self.b, font_size, a_color, dark, window,
            ));
            spawn_words(row, "+", dark, font_size, window);
            row.spawn(stacked_fraction(
                self.c, self.b, font_size, c_color, dark, window,
            ));
            spawn_words(row, "=", dark, font_size, window);
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: theme::scaled(font_size * 0.28),
                ..default()
            })
            .with_children(|group| {
                group.spawn(stacked_fraction(
                    sum, self.b, font_size, sum_color, dark, window,
                ));
                group.spawn((
                    Text::new("."),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    TextColor(dark),
                    DesignFontSize {
                        size: font_size,
                        window,
                    },
                ));
            });
        });
    }
}
