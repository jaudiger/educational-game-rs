use std::borrow::Cow;

use super::Language;

/// Common strings translated by every edition.
///
/// Static variants return `Cow::Borrowed(&'static str)` via `translate()`.
/// Parameterized variants (containing data) return `Cow::Owned(String)`.
/// The `I18n::t()` method dispatches to the active language via `translate()`.
#[derive(Clone, Debug, Default)]
pub enum TranslationKey {
    // Common
    #[default]
    AppTitle,
    Back,
    ComingSoon,
    ExerciseCount(usize),
    Play,
    Settings,

    // Fraction Comparison
    CharacterAte(String, u32, u32),
    EqualAmount,

    // Fraction Identification
    WhatFractionColored,

    // Fraction Visualization
    Validate,

    // Lesson Play
    CorrectAnswer,
    IncorrectAnswer,
    QuestionProgress(usize, usize),
    ReturnToExerciseList,

    // Lesson Summary

    // Theme Exploration
    BackToLessons,
    BackToThemes,
    Themes,

    // Save Slots

    // Settings
    ExplanationsLabel,
    ExplanationsOn,
    GamepadNavigationLabel,
    GamepadNavigationOn,
    LanguageEnglish,
    LanguageFrench,
    LanguageLabel,
    ExplorationThemeLabel,
    ExplorationThemeOcean,
    ExplorationThemeSky,
    ExplorationThemeSpace,
    MusicVolumeLabel,
    SfxVolumeLabel,

    // Teacher
    ExerciseTypeComparison,
    ExerciseTypeIdentification,
    ExerciseTypeMcq,

    ExerciseTypeNumericInput,
    ExerciseTypeVisualization,
}

/// Generate a `match $self { ... }` expression from three sections of the translation table.
///
/// The `universal` section covers variants whose string is the same in every language.
/// The `bilingual` section covers static (no-field) variants with one string per language.
/// Any remaining token trees after the two sections become verbatim match arms, used for
/// parameterized variants that need field destructuring and `format!` calls.
macro_rules! translate {
    (
        $self:expr, $lang:expr;
        universal: [ $( $(#[$u_meta:meta])* $u_variant:ident => $u_text:expr ),* $(,)? ];
        bilingual: [ $( $(#[$b_meta:meta])* $b_variant:ident => ( $fr:expr, $en:expr ) ),* $(,)? ];
        $( $rest:tt )*
    ) => {
        match $self {
            $( $(#[$u_meta])* Self::$u_variant => $u_text.into(), )*
            $( $(#[$b_meta])* Self::$b_variant => match $lang {
                Language::French  => $fr.into(),
                Language::English => $en.into(),
            }, )*
            $( $rest )*
        }
    };
}

impl TranslationKey {
    /// Translate this key to the given language.
    ///
    /// Static variants are zero-alloc; parameterized variants allocate.
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    pub fn translate(&self, language: Language) -> Cow<'static, str> {
        translate! {
            self, language;
            universal: [
                LanguageEnglish            => "English",
                LanguageFrench             => "Fran\u{00e7}ais",
                ExerciseTypeIdentification => "Identification",
                SfxVolumeLabel             => "Interface",
            ];
            bilingual: [
                // Common
                AppTitle                  => ("Jeu \u{00c9}ducatif",                      "Educational Game"),
                Back                      => ("Retour",                                    "Back"),
                ComingSoon                => ("(bient\u{00f4}t)",                          "(coming soon)"),
                Play                      => ("Jouer",                                     "Play"),
                Settings                  => ("Param\u{00e8}tres",                         "Settings"),
                // Fraction Comparison
                EqualAmount               => ("Pareil",                                    "Equal"),
                // Fraction Identification
                WhatFractionColored       => ("Quelle fraction est colori\u{00e9}e ?",    "What fraction is colored?"),
                // Fraction Visualization
                Validate                  => ("Valider",                                   "Validate"),
                // Lesson Play
                CorrectAnswer             => ("Bonne r\u{00e9}ponse !",                    "Correct!"),
                IncorrectAnswer           => ("Mauvaise r\u{00e9}ponse",                   "Incorrect"),
                ReturnToExerciseList      => ("Retour a la liste des exercices",           "Return to exercises"),
                // Lesson Summary
                // Theme Exploration
                BackToLessons           => ("Retour aux le\u{00e7}ons",                      "Back to lessons"),
                BackToThemes            => ("Retour aux th\u{00e9}matiques",          "Back to themes"),
                Themes                  => ("Th\u{00e9}matiques",                        "Themes"),
                // Save Slots
                // Settings
                ExplanationsLabel         => ("Explications",                              "Explanations"),
                ExplanationsOn            => ("Oui",                                       "Yes"),
                GamepadNavigationLabel    => ("Manette / Clavier",                         "Gamepad / Keyboard"),
                GamepadNavigationOn       => ("Oui",                                       "Yes"),
                LanguageLabel             => ("Langue",                                    "Language"),
                ExplorationThemeLabel             => ("Th\u{00e8}me visuel",                       "Visual Theme"),
                ExplorationThemeOcean             => ("Oc\u{00e9}an",                              "Ocean"),
                ExplorationThemeSky               => ("Ciel",                                      "Sky"),
                ExplorationThemeSpace             => ("Espace",                                    "Space"),
                MusicVolumeLabel          => ("Musique",                                   "Music"),
                // Teacher
                ExerciseTypeComparison    => ("Comparaison",                               "Comparison"),
                ExerciseTypeMcq           => ("QCM",                                       "MCQ"),
                ExerciseTypeNumericInput  => ("Saisie num\u{00e9}rique",                   "Numeric Input"),
                ExerciseTypeVisualization => ("Visualisation",                             "Visualization"),
            ];
            // Parameterized variants need per-field destructuring and format! calls.
            Self::CharacterAte(name, n, d) => match language {
                Language::French  => format!("{name} mange {n}/{d}").into(),
                Language::English => format!("{name} eats {n}/{d}").into(),
            },
            Self::ExerciseCount(count) => match language {
                Language::French if *count == 1 => format!("{count} exercice").into(),
                Language::French => format!("{count} exercices").into(),
                Language::English if *count == 1 => format!("{count} exercise").into(),
                Language::English => format!("{count} exercises").into(),
            },
            Self::QuestionProgress(current, total) => match language {
                Language::French  => format!("Question {current}/{total}").into(),
                Language::English => format!("Question {current} of {total}").into(),
            },
        }
    }
}
