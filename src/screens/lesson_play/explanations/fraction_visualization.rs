use bevy::prelude::*;

use crate::i18n::I18n;

use super::fraction_identification::spawn_fraction_parts_text;
use super::renderer::ExplanationRenderer;

/// Same colouring scheme as identification, with phrasing adapted to the
/// "colour the bar" task (numerator in blue, denominator in orange).
pub(super) struct FractionVisualizationRenderer {
    numerator: u32,
    denominator: u32,
}

impl FractionVisualizationRenderer {
    pub(super) const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl ExplanationRenderer for FractionVisualizationRenderer {
    fn spawn(&self, parent: &mut ChildSpawnerCommands, i18n: &I18n, window: Entity) {
        spawn_fraction_parts_text(parent, i18n, self.numerator, self.denominator, true, window);
    }
}
