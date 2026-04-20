use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::{I18n, TranslationKey};
use crate::ui::theme::{self, DesignFontSize};

use super::super::visuals::{PV_ZERO_COLOR, count_trailing_zeros};
use super::renderer::ExplanationRenderer;

/// Place-value multiplication: the trailing zeros of the multiplier and
/// result are highlighted in the place-value colour to match the table.
pub(super) struct PlaceValueRenderer {
    number: u32,
    multiplier: u32,
}

impl PlaceValueRenderer {
    pub(super) const fn new(number: u32, multiplier: u32) -> Self {
        Self { number, multiplier }
    }
}

impl ExplanationRenderer for PlaceValueRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        let result = self.number * self.multiplier;
        let zeros_added = count_trailing_zeros(self.multiplier);

        let mult_str = self.multiplier.to_string();
        let result_str = result.to_string();

        let mult_prefix = &mult_str[..mult_str.len() - zeros_added];
        let mult_zeros = &mult_str[mult_str.len() - zeros_added..];

        let result_prefix = &result_str[..result_str.len() - zeros_added];
        let result_zeros = &result_str[result_str.len() - zeros_added..];

        let number_str = self.number.to_string();

        let (before_mult, between, after_eq) = match i18n.language {
            Language::French => (
                format!("{} Multiplier par ", i18n.t(&TranslationKey::Explanation)),
                format!(" c'est ajouter des zéros : {number_str} × {mult_prefix}"),
                ".".to_owned(),
            ),
            Language::English => (
                format!("{} Multiplying by ", i18n.t(&TranslationKey::Explanation)),
                format!(" means adding zeros: {number_str} × {mult_prefix}"),
                ".".to_owned(),
            ),
        };

        let text_span = |s: &str, color: Color| {
            (
                Text::new(s),
                TextFont {
                    font_size: theme::fonts::HEADING,
                    ..default()
                },
                TextColor(color),
                DesignFontSize {
                    size: theme::fonts::HEADING,
                    window,
                },
            )
        };

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
                        flex_wrap: FlexWrap::Wrap,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(text_span(&before_mult, theme::colors::TEXT_DARK));
                        row.spawn(text_span(mult_prefix, theme::colors::TEXT_DARK));
                        row.spawn(text_span(mult_zeros, PV_ZERO_COLOR));
                        row.spawn(text_span(&between, theme::colors::TEXT_DARK));
                        row.spawn(text_span(mult_zeros, PV_ZERO_COLOR));
                        row.spawn(text_span(" = ", theme::colors::TEXT_DARK));
                        row.spawn(text_span(result_prefix, theme::colors::TEXT_DARK));
                        row.spawn(text_span(result_zeros, PV_ZERO_COLOR));
                        row.spawn(text_span(&after_eq, theme::colors::TEXT_DARK));
                    });
            });
    }
}
