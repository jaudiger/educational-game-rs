pub mod content;
pub mod persistence;
pub mod progress;
pub mod save;
pub mod save_write;
pub mod system_params;

pub use content::{
    AnswerResult, ContentLibrary, ExplanationVisual, QuestionDefinition, ResolvedQuestion,
};
pub use persistence::{PersistencePaths, load_or_default, write_json_atomic};
pub use progress::{
    ActiveTheme, ExplorationTheme, GameMode, GameSettings, Language, LastAnswer, LessonSession,
    QuestionContainer, SelectedLesson,
};
pub use save::{
    ActiveStudent, ClassSave, ClassStudent, IndividualSave, LessonProgress, LessonSessionConfig,
    PlayerSession, SaveData, get_current_progress,
};
pub use save_write::{
    PersistenceAction, PersistenceStatus, persist_game_settings, update_game_settings,
    update_save_data,
};
pub use system_params::{GameSettingsMut, PersistenceMut, PlayerContext, SaveDataMut};
