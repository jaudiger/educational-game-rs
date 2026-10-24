pub mod assessment;
pub mod content;
#[cfg(not(feature = "demo"))]
pub mod persistence;
#[cfg(not(feature = "demo"))]
pub mod production;
#[cfg(not(feature = "demo"))]
mod production_system_params;
pub mod progress;
#[cfg(not(feature = "demo"))]
pub mod save;
#[cfg(not(feature = "demo"))]
pub mod save_write;
pub mod system_params;

pub use assessment::{QuestionResponse, assess_response};
pub use content::{
    AnswerResult, ContentLibrary, ExplanationVisual, FractionShape, QuestionInstance,
};
#[cfg(not(feature = "demo"))]
pub use persistence::{PersistencePaths, load_or_default, write_json_atomic};
#[cfg(not(feature = "demo"))]
pub use production::{GameMode, ProductionSettings};
#[cfg(not(feature = "demo"))]
pub use production_system_params::{
    PersistenceMut, ProductionPlayerContext, ProductionSettingsMut, SaveDataMut,
};
pub use progress::{
    ActiveTheme, ExplorationTheme, GameSettings, Language, LastAnswer, LessonLaunchSelection,
    LessonSession, QuestionContainer, SelectedLesson, SettingsChanged,
};
#[cfg(not(feature = "demo"))]
pub use save::{
    ActiveStudent, ClassSave, ClassStudent, IndividualSave, LessonProgress, LessonScore,
    LessonSessionConfig, PlayerSession, SaveData, get_current_progress,
};
#[cfg(not(feature = "demo"))]
pub use save_write::{
    PersistedSettings, PersistenceAction, PersistenceStatus, persist_game_settings,
    update_save_data,
};
pub use system_params::GameSettingsMut;
