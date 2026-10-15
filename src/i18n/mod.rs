pub mod keys;
#[cfg(not(feature = "demo"))]
mod production_keys;

use std::borrow::Cow;

use bevy::prelude::*;

pub use keys::TranslationKey;
#[cfg(not(feature = "demo"))]
pub use production_keys::ProductionTranslationKey;

pub trait Translate {
    fn translate_to(&self, language: Language) -> Cow<'static, str>;
}

impl Translate for TranslationKey {
    fn translate_to(&self, language: Language) -> Cow<'static, str> {
        self.translate(language)
    }
}

#[cfg(not(feature = "demo"))]
impl Translate for ProductionTranslationKey {
    fn translate_to(&self, language: Language) -> Cow<'static, str> {
        self.translate(language)
    }
}

/// Supported UI languages for translation.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, serde::Deserialize, serde::Serialize,
)]
pub enum Language {
    #[default]
    French,
    English,
}

/// Central translation resource. Inserted at startup from `GameSettings::language`.
#[derive(Resource, Reflect)]
pub struct I18n {
    pub language: Language,
}

impl I18n {
    pub const fn new(language: Language) -> Self {
        Self { language }
    }

    /// Translate a key to the active language.
    ///
    /// Returns borrowed static text or an owned formatted translation.
    pub fn t(&self, key: &impl Translate) -> Cow<'static, str> {
        key.translate_to(self.language)
    }
}
