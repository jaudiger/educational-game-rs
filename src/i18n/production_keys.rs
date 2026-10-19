use std::borrow::Cow;

use super::Language;

#[derive(Clone, Debug)]
pub enum ProductionTranslationKey {
    Add,
    Cancel,
    Create,
    Delete,
    NameLabel,
    Quit,
    SaveWriteFailed,
    FinishLesson,
    NextQuestion,
    SummaryEncouragement,
    SummaryGood,
    SummaryPerfect,
    SummaryPercentage(u32),
    SummaryScore(u32, u32),
    SummaryTitle,
    BestPercent(u32),
    ChooseActivity,
    PlaySeries,
    LessonsCompleted(usize, usize),
    CreateSaveSlotN(usize),
    DeleteSlotN(usize),
    Empty,
    SelectClass,
    SelectSave,
    SlotN(usize),
    Mode,
    ModeClass,
    ModeIndividual,
    NoLessonsCompleted,
    NoStudentsYet,
    RemoveStudentConfirm(String),
    ResetAllStatsConfirm,
    ResetLessonConfigConfirm(String),
    ResetLessonStatsConfirm(String),
    ResetThemeConfigConfirm(String),
    ResetExerciseStatsConfirm(String, String),
    ResetTypeStatsConfirm(String, String),
    StudentStats(String),
    StudentsOf(String),
    TabLessons,
    TabStudents,
    TeacherDashboard,
    Total,
}

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
                Language::French => $fr.into(),
                Language::English => $en.into(),
            }, )*
            $( $rest )*
        }
    };
}

impl ProductionTranslationKey {
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    pub fn translate(&self, language: Language) -> Cow<'static, str> {
        translate! {
            self, language;
            universal: [
                Mode                       => "Mode",
                Total                      => "Total",
            ];
            bilingual: [
                Add                       => ("Ajouter",                                  "Add"),
                Cancel                    => ("Annuler",                                   "Cancel"),
                Create                    => ("Cr\u{00e9}er",                              "Create"),
                Delete                    => ("Supprimer",                                 "Delete"),
                NameLabel                 => ("Nom :",                                     "Name:"),
                Quit                      => ("Quitter",                                   "Quit"),
                SaveWriteFailed            => ("\u{00c9}chec de sauvegarde. Les changements peuvent \u{00ea}tre perdus \u{00e0} la fermeture.", "Save failed. Changes may be lost when the app closes."),
                FinishLesson              => ("Terminer la le\u{00e7}on",                  "Finish lesson"),
                NextQuestion              => ("Question suivante",                          "Next question"),
                SummaryEncouragement      => ("Continue, tu vas y arriver !",              "Keep going, you'll get there!"),
                SummaryGood               => ("Bon travail !",                             "Good job!"),
                SummaryPerfect            => ("Parfait ! Bravo !",                         "Perfect! Well done!"),
                SummaryTitle              => ("R\u{00e9}sum\u{00e9}",                      "Summary"),
                ChooseActivity          => ("Choisir une activit\u{00e9}",                "Choose an activity"),
                PlaySeries              => ("Jouer une s\u{00e9}rie",                    "Play a series"),
                Empty                     => ("Vide",                                      "Empty"),
                SelectClass               => ("Choisir une classe",                        "Select Class"),
                SelectSave                => ("Choisir une sauvegarde",                    "Select Save"),
                ModeClass                 => ("Classe",                                    "Class"),
                ModeIndividual            => ("Individuel",                                "Individual"),
                NoLessonsCompleted        => ("(aucune le\u{00e7}on termin\u{00e9}e)",     "(no lessons completed)"),
                NoStudentsYet             => ("Aucun \u{00e9}l\u{00e8}ve pour l'instant",  "No students yet"),
                ResetAllStatsConfirm      => ("R\u{00e9}initialiser toutes les stats ?",   "Reset all stats?"),
                TabLessons                => ("Le\u{00e7}ons",                             "Lessons"),
                TabStudents               => ("\u{00c9}l\u{00e8}ves",                      "Students"),
                TeacherDashboard          => ("Tableau enseignant",                        "Teacher Dashboard"),
            ];
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
            Self::ResetLessonConfigConfirm(lesson) => match language {
                Language::French  => format!("R\u{00e9}initialiser la configuration de {lesson} ?").into(),
                Language::English => format!("Reset the configuration for {lesson}?").into(),
            },
            Self::ResetThemeConfigConfirm(theme) => match language {
                Language::French  => format!("R\u{00e9}initialiser la configuration du th\u{00e8}me {theme} ?").into(),
                Language::English => format!("Reset the configuration for the {theme} theme?").into(),
            },
            Self::ResetLessonStatsConfirm(lesson) => match language {
                Language::French  => format!("R\u{00e9}initialiser les stats pour {lesson} ?").into(),
                Language::English => format!("Reset stats for {lesson}?").into(),
            },
            Self::ResetTypeStatsConfirm(lesson, qtype) => match language {
                Language::French  => format!("R\u{00e9}initialiser les stats {qtype} pour {lesson} ?").into(),
                Language::English => format!("Reset {qtype} stats for {lesson}?").into(),
            },
            Self::ResetExerciseStatsConfirm(lesson, exercise) => match language {
                Language::French  => format!("R\u{00e9}initialiser les stats {exercise} pour {lesson} ?").into(),
                Language::English => format!("Reset {exercise} stats for {lesson}?").into(),
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
