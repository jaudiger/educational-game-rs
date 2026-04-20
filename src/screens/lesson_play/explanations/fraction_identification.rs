use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::{I18n, TranslationKey};
use crate::ui::components::stacked_fraction;
use crate::ui::theme::{self, DesignFontSize};

use super::renderer::{ExplanationRenderer, spawn_colored_row, spawn_words};

/// Numerator in blue, denominator in orange, same colours carried into the
/// inline stacked fraction.
pub(super) struct FractionIdentificationRenderer {
    numerator: u32,
    denominator: u32,
}

impl FractionIdentificationRenderer {
    pub(super) const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl ExplanationRenderer for FractionIdentificationRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        spawn_fraction_parts_text(
            parent,
            i18n,
            self.numerator,
            self.denominator,
            false,
            window,
        );
    }
}

/// Shared rendering logic for identification and visualization variants.
/// `is_visualization` toggles the alternate phrasing used when the question
/// asks the student to colour a fraction bar.
pub(super) fn spawn_fraction_parts_text(
    parent: &mut ChildSpawnerCommands,
    i18n: &I18n,
    numerator: u32,
    denominator: u32,
    is_visualization: bool,
    window: Entity,
) {
    let font_size = theme::fonts::HEADING;
    let num_color = theme::colors::PRIMARY;
    let den_color = theme::colors::SECONDARY;
    let dark = theme::colors::TEXT_DARK;

    let num_str = numerator.to_string();
    let den_str = denominator.to_string();

    let (before_den, between, after_num) = match (i18n.language, is_visualization) {
        (Language::French, false) => (
            format!("{} Il y a", i18n.t(&TranslationKey::Explanation)),
            "parts et".to_owned(),
            "sont coloriées : c'est".to_owned(),
        ),
        (Language::French, true) => (
            format!("{} Il y a", i18n.t(&TranslationKey::Explanation)),
            "parts au total et".to_owned(),
            "sont coloriées : c'est la fraction".to_owned(),
        ),
        (Language::English, false) => (
            format!("{} There are", i18n.t(&TranslationKey::Explanation)),
            "parts and".to_owned(),
            "are colored: that's".to_owned(),
        ),
        (Language::English, true) => (
            format!("{} There are", i18n.t(&TranslationKey::Explanation)),
            "parts in total and".to_owned(),
            "are colored: that's the fraction".to_owned(),
        ),
    };

    spawn_colored_row(parent, font_size, |row| {
        spawn_words(row, &before_den, dark, font_size, window);
        spawn_words(row, &den_str, den_color, font_size, window);
        spawn_words(row, &between, dark, font_size, window);
        spawn_words(row, &num_str, num_color, font_size, window);
        spawn_words(row, &after_num, dark, font_size, window);
        row.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: theme::scaled(font_size * 0.28),
            ..default()
        })
        .with_children(|group| {
            group.spawn(stacked_fraction(
                numerator,
                denominator,
                font_size,
                num_color,
                den_color,
                window,
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
