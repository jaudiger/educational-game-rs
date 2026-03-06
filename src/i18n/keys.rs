use std::borrow::Cow;

use super::Language;

/// All translatable strings in the application.
///
/// Static variants return `Cow::Borrowed(&'static str)` via `translate()`.
/// Parameterized variants (containing data) return `Cow::Owned(String)`.
/// The `I18n::t()` method dispatches to the active language via `translate()`.
#[derive(Clone, Debug, Default)]
pub enum TranslationKey {
    // Common
    Add,
    #[default]
    AppTitle,
    Back,
    Cancel,
    ComingSoon,
    Create,
    Delete,
    NameLabel,
    Play,
    Quit,
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
    Explanation,
    FinishLesson,
    IncorrectAnswer,
    NextQuestion,
    QuestionProgress(usize, usize),

    // Lesson Summary
    SummaryEncouragement,
    SummaryGood,
    SummaryPerfect,
    SummaryPercentage(u32),
    SummaryScore(u32, u32),
    SummaryTitle,

    // Lessons
    LessonFractions,
    LessonMultiplicationTables,
    LessonMultiplyByPowerOf10,

    // Map / Lesson
    BackToWorldMap,
    BestPercent(u32),
    LessonsCompleted(usize, usize),
    WorldMap,

    // Save Slots
    CreateSaveSlotN(usize),
    DeleteSlotN(usize),
    Empty,
    SelectClass,
    SelectSave,
    SlotN(usize),

    // Settings
    ExplanationsLabel,
    ExplanationsOn,
    GamepadNavigationLabel,
    GamepadNavigationOn,
    LanguageEnglish,
    LanguageFrench,
    LanguageLabel,
    MapThemeLabel,
    MapThemeOcean,
    MapThemeSky,
    MapThemeSpace,
    Mode,
    ModeClass,
    ModeIndividual,
    MusicVolumeLabel,
    SfxVolumeLabel,

    // Teacher
    NoLessonsCompleted,
    NoStudentsYet,
    QuestionTypeComparison,
    QuestionTypeIdentification,
    QuestionTypeMcq,

    QuestionTypeNumericInput,
    QuestionTypeVisualization,
    RemoveStudentConfirm(String),
    ResetAllStatsConfirm,
    ResetConfig,
    ResetLessonStatsConfirm(String),
    ResetTypeStatsConfirm(String, String),
    SaveConfig,
    StudentStats(String),
    StudentsOf(String),
    TabLessons,
    TabStudents,
    TeacherDashboard,
    Total,

    // Themes (pedagogical)
    ThemeEnglish,
    ThemeFrench,
    ThemeMaths,
    ThemeScience,
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
        universal: [ $( $u_variant:ident => $u_text:expr ),* $(,)? ];
        bilingual: [ $( $b_variant:ident => ( $fr:expr, $en:expr ) ),* $(,)? ];
        $( $rest:tt )*
    ) => {
        match $self {
            $( Self::$u_variant => $u_text.into(), )*
            $( Self::$b_variant => match $lang {
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
                Mode                       => "Mode",
                QuestionTypeIdentification => "Identification",
                SfxVolumeLabel             => "Interface",
                Total                      => "Total",
            ];
            bilingual: [
                // Common
                Add                       => ("Ajouter",                                  "Add"),
                AppTitle                  => ("Jeu \u{00c9}ducatif",                      "Educational Game"),
                Back                      => ("Retour",                                    "Back"),
                Cancel                    => ("Annuler",                                   "Cancel"),
                ComingSoon                => ("(bient\u{00f4}t)",                          "(coming soon)"),
                Create                    => ("Cr\u{00e9}er",                              "Create"),
                Delete                    => ("Supprimer",                                 "Delete"),
                NameLabel                 => ("Nom :",                                     "Name:"),
                Play                      => ("Jouer",                                     "Play"),
                Quit                      => ("Quitter",                                   "Quit"),
                Settings                  => ("Param\u{00e8}tres",                         "Settings"),
                // Fraction Comparison
                EqualAmount               => ("Pareil",                                    "Equal"),
                // Fraction Identification
                WhatFractionColored       => ("Quelle fraction est colori\u{00e9}e ?",    "What fraction is colored?"),
                // Fraction Visualization
                Validate                  => ("Valider",                                   "Validate"),
                // Lesson Play
                CorrectAnswer             => ("Bonne r\u{00e9}ponse !",                    "Correct!"),
                Explanation               => ("Explication :",                             "Explanation:"),
                FinishLesson              => ("Terminer la le\u{00e7}on",                  "Finish lesson"),
                IncorrectAnswer           => ("Mauvaise r\u{00e9}ponse",                   "Incorrect"),
                NextQuestion              => ("Question suivante",                          "Next question"),
                // Lesson Summary
                SummaryEncouragement      => ("Continue, tu vas y arriver !",              "Keep going, you'll get there!"),
                SummaryGood               => ("Bon travail !",                             "Good job!"),
                SummaryPerfect            => ("Parfait ! Bravo !",                         "Perfect! Well done!"),
                SummaryTitle              => ("R\u{00e9}sum\u{00e9}",                      "Summary"),
                // Lessons
                LessonFractions           => ("Les fractions",                             "Fractions"),
                LessonMultiplicationTables => ("Tables de multiplication",                 "Multiplication Tables"),
                LessonMultiplyByPowerOf10 => ("Multiplier par 10, 100, 1000",             "Multiply by 10, 100, 1000"),
                // Map / Lesson
                BackToWorldMap            => ("Retour \u{00e0} la carte",                  "Back to map"),
                WorldMap                  => ("Carte du monde",                            "World Map"),
                // Save Slots
                Empty                     => ("Vide",                                      "Empty"),
                SelectClass               => ("Choisir une classe",                        "Select Class"),
                SelectSave                => ("Choisir une sauvegarde",                    "Select Save"),
                // Settings
                ExplanationsLabel         => ("Explications",                              "Explanations"),
                ExplanationsOn            => ("Oui",                                       "Yes"),
                GamepadNavigationLabel    => ("Manette / Clavier",                         "Gamepad / Keyboard"),
                GamepadNavigationOn       => ("Oui",                                       "Yes"),
                LanguageLabel             => ("Langue",                                    "Language"),
                MapThemeLabel             => ("Th\u{00e8}me visuel",                       "Visual Theme"),
                MapThemeOcean             => ("Oc\u{00e9}an",                              "Ocean"),
                MapThemeSky               => ("Ciel",                                      "Sky"),
                MapThemeSpace             => ("Espace",                                    "Space"),
                ModeClass                 => ("Classe",                                    "Class"),
                ModeIndividual            => ("Individuel",                                "Individual"),
                MusicVolumeLabel          => ("Musique",                                   "Music"),
                // Teacher
                NoLessonsCompleted        => ("(aucune le\u{00e7}on termin\u{00e9}e)",     "(no lessons completed)"),
                NoStudentsYet             => ("Aucun \u{00e9}l\u{00e8}ve pour l'instant",  "No students yet"),
                QuestionTypeComparison    => ("Comparaison",                               "Comparison"),
                QuestionTypeMcq           => ("QCM",                                       "MCQ"),
                QuestionTypeNumericInput  => ("Saisie num\u{00e9}rique",                   "Numeric Input"),
                QuestionTypeVisualization => ("Visualisation",                             "Visualization"),
                ResetAllStatsConfirm      => ("R\u{00e9}initialiser toutes les stats ?",   "Reset all stats?"),
                ResetConfig               => ("R\u{00e9}initialiser",                      "Reset"),
                SaveConfig                => ("Sauvegarder",                               "Save"),
                TabLessons                => ("Le\u{00e7}ons",                             "Lessons"),
                TabStudents               => ("\u{00c9}l\u{00e8}ves",                      "Students"),
                TeacherDashboard          => ("Tableau enseignant",                        "Teacher Dashboard"),
                // Themes (pedagogical)
                ThemeEnglish              => ("Anglais",                                   "English"),
                ThemeFrench               => ("Fran\u{00e7}ais",                           "French"),
                ThemeMaths                => ("Math\u{00e9}matiques",                      "Mathematics"),
                ThemeScience              => ("Sciences",                                  "Science"),
            ];
            // Parameterized variants need per-field destructuring and format! calls.
            Self::CharacterAte(name, n, d) => match language {
                Language::French  => format!("{name} mange {n}/{d}").into(),
                Language::English => format!("{name} eats {n}/{d}").into(),
            },
            Self::QuestionProgress(current, total) => match language {
                Language::French  => format!("Question {current}/{total}").into(),
                Language::English => format!("Question {current} of {total}").into(),
            },
            Self::SummaryPercentage(pct) => match language {
                Language::French  => format!("R\u{00e9}ussite : {pct} %").into(),
                Language::English => format!("Success: {pct}%").into(),
            },
            Self::SummaryScore(correct, total) => match language {
                Language::French  => format!("Score : {correct}/{total}").into(),
                Language::English => format!("Score: {correct}/{total}").into(),
            },
            Self::BestPercent(pct) => match language {
                Language::French  => format!("{pct} %").into(),
                Language::English => format!("{pct}%").into(),
            },
            Self::LessonsCompleted(done, total) => match language {
                Language::French  => format!("{done}/{total} le\u{00e7}ons").into(),
                Language::English => format!("{done}/{total} lessons").into(),
            },
            Self::CreateSaveSlotN(n) => match language {
                Language::French  => format!("Cr\u{00e9}er - Emplacement {n}").into(),
                Language::English => format!("Create Save - Slot {n}").into(),
            },
            Self::DeleteSlotN(n) => match language {
                Language::French  => format!("Supprimer l'emplacement {n} ?").into(),
                Language::English => format!("Delete Slot {n}?").into(),
            },
            Self::SlotN(n) => match language {
                Language::French  => format!("Emplacement {n}").into(),
                Language::English => format!("Slot {n}").into(),
            },
            Self::RemoveStudentConfirm(name) => match language {
                Language::French  => format!("Supprimer {name} ?").into(),
                Language::English => format!("Remove {name}?").into(),
            },
            Self::ResetLessonStatsConfirm(lesson) => match language {
                Language::French  => format!("R\u{00e9}initialiser les stats pour {lesson} ?").into(),
                Language::English => format!("Reset stats for {lesson}?").into(),
            },
            Self::ResetTypeStatsConfirm(lesson, qtype) => match language {
                Language::French  => format!("R\u{00e9}initialiser les stats {qtype} pour {lesson} ?").into(),
                Language::English => format!("Reset {qtype} stats for {lesson}?").into(),
            },
            Self::StudentStats(name) => match language {
                Language::French  => format!("{name} - Statistiques").into(),
                Language::English => format!("{name} - Stats").into(),
            },
            Self::StudentsOf(name) => match language {
                Language::French  => format!("\u{00c9}l\u{00e8}ves - {name}").into(),
                Language::English => format!("Students - {name}").into(),
            },
        }
    }
}
