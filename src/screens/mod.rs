pub mod home;
pub mod lesson_play;
pub mod lesson_summary;
pub mod save_slots;
pub mod settings;
pub mod theme_exploration;

pub mod teacher_lessons;
pub mod teacher_roster;
pub mod teacher_shared;
pub mod teacher_stats;

pub use home::HomeScreenPlugin;
pub use lesson_play::LessonPlayScreenPlugin;
pub use lesson_summary::LessonSummaryScreenPlugin;
pub use save_slots::SaveSlotsScreenPlugin;
pub use settings::SettingsScreenPlugin;
pub use theme_exploration::ThemeExplorationScreenPlugin;

pub use teacher_lessons::TeacherLessonsScreenPlugin;
pub use teacher_roster::TeacherRosterScreenPlugin;
pub use teacher_stats::TeacherStatsScreenPlugin;
