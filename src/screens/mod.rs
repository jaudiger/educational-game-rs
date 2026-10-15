pub mod exercise_selection;
pub mod home;
pub mod lesson_play;
#[cfg(not(feature = "demo"))]
pub mod lesson_summary;
#[cfg(not(feature = "demo"))]
pub mod save_slots;
pub mod settings;
pub mod theme_exploration;

#[cfg(not(feature = "demo"))]
pub mod teacher_lessons;
#[cfg(not(feature = "demo"))]
pub mod teacher_roster;
pub mod teacher_shared;
#[cfg(not(feature = "demo"))]
pub mod teacher_stats;

pub use exercise_selection::ExerciseSelectionScreenPlugin;
pub use home::HomeScreenPlugin;
pub use lesson_play::LessonPlayScreenPlugin;
#[cfg(not(feature = "demo"))]
pub use lesson_summary::LessonSummaryScreenPlugin;
#[cfg(not(feature = "demo"))]
pub use save_slots::SaveSlotsScreenPlugin;
pub use settings::SettingsScreenPlugin;
pub use theme_exploration::ThemeExplorationScreenPlugin;

#[cfg(not(feature = "demo"))]
pub use teacher_lessons::TeacherLessonsScreenPlugin;
#[cfg(not(feature = "demo"))]
pub use teacher_roster::TeacherRosterScreenPlugin;
#[cfg(not(feature = "demo"))]
pub use teacher_stats::TeacherStatsScreenPlugin;
