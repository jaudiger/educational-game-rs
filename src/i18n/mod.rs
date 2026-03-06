pub mod keys;

use std::borrow::Cow;

use bevy::prelude::*;

pub use keys::TranslationKey;

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
    /// Returns `Cow<'static, str>`: static variants are zero-alloc (`Borrowed`),
    /// parameterized variants (e.g. `SlotN`, `StudentsOf`) allocate (`Owned`).
    pub fn t(&self, key: &TranslationKey) -> Cow<'static, str> {
        key.translate(self.language)
    }
}
