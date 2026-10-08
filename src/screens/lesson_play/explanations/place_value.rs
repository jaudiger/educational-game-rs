use bevy::prelude::*;

use crate::data::Language;
use crate::i18n::I18n;
use crate::ui::theme;

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

        let (before_mult, between) = match i18n.language {
            Language::French => (
                "Multiplier par ",
                format!(" c'est ajouter des zéros : {number_str} × "),
            ),
            Language::English => (
                "Multiplying by ",
                format!(" means adding zeros: {number_str} × "),
            ),
        };

        let text_span = |s: &str, color: Color| {
            (
                TextSpan::new(s),
                theme::typography::text(theme::fonts::HEADING, window),
                TextColor(color),
            )
        };

        parent.spawn((
            Node {
                align_self: AlignSelf::Stretch,
                width: percent(100.0),
                ..default()
            },
            Text::new(before_mult),
            theme::typography::text(theme::fonts::HEADING, window),
            TextColor(theme::colors::TEXT_DARK),
            TextLayout::justify(Justify::Center),
            children![
                text_span(mult_prefix, theme::colors::TEXT_DARK),
                text_span(mult_zeros, PV_ZERO_COLOR),
                text_span(&between, theme::colors::TEXT_DARK),
                text_span(mult_prefix, theme::colors::TEXT_DARK),
                text_span(mult_zeros, PV_ZERO_COLOR),
                text_span(" = ", theme::colors::TEXT_DARK),
                text_span(result_prefix, theme::colors::TEXT_DARK),
                text_span(result_zeros, PV_ZERO_COLOR),
                text_span(".", theme::colors::TEXT_DARK),
            ],
        ));
    }
}
