use std::collections::HashMap;

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use bevy_persistent::prelude::*;

use crate::data::{AnswerResult, GameSettings};
use crate::questions::AnswerSubmitted;
use crate::states::AppState;

/// Duration of music fade transitions in seconds.
const FADE_DURATION: f32 = 2.0;

/// Background music tracks. Each variant maps to an asset path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum MusicTrack {
    Menu,
    Exploration,
    Lesson,
}

impl MusicTrack {
    const fn asset_path(self) -> &'static str {
        match self {
            Self::Menu => "audio/music/menu.ogg",
            Self::Exploration => "audio/music/exploration.ogg",
            Self::Lesson => "audio/music/lesson.ogg",
        }
    }

    /// Per-track volume multiplier applied on top of the user's music volume.
    /// Lesson music is softer to help concentration.
    const fn volume_multiplier(self) -> f32 {
        match self {
            Self::Menu | Self::Exploration => 0.75,
            Self::Lesson => 0.25,
        }
    }
}

/// Sound effect kinds. Each variant maps to an asset path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub enum SfxKind {
    Click,
    Correct,
    Incorrect,
}

impl SfxKind {
    const fn asset_path(self) -> &'static str {
        match self {
            Self::Click => "audio/sfx/click.ogg",
            Self::Correct => "audio/sfx/correct.ogg",
            Self::Incorrect => "audio/sfx/incorrect.ogg",
        }
    }
}

/// Returns the desired music track for a given `AppState`.
///
/// States within the same "group" share a track so that transitions
/// between them (e.g. `Home`, `SaveSlots`, `Settings`) do not
/// restart the music.
const fn music_for_state(state: AppState) -> MusicTrack {
    match state {
        AppState::Home | AppState::SaveSlots | AppState::Settings => MusicTrack::Menu,
        AppState::MapExploration => MusicTrack::Exploration,
        AppState::LessonPlay | AppState::LessonSummary => MusicTrack::Lesson,
    }
}

/// Pre-loaded audio asset handles.
#[derive(Resource, Reflect)]
pub struct AudioAssets {
    #[reflect(ignore)]
    music: HashMap<MusicTrack, Handle<AudioSource>>,
    #[reflect(ignore)]
    sfx: HashMap<SfxKind, Handle<AudioSource>>,
}

/// Tracks the currently playing music track to avoid restarting the same
/// track on intra-group state transitions (e.g. `Home` to `SaveSlots`).
#[derive(Resource, Deref, Reflect)]
struct CurrentMusic(MusicTrack);

/// Marker for the active background music entity.
#[derive(Component, Reflect)]
pub struct BackgroundMusic;

/// Marker for one-shot sound effect entities.
#[derive(Component, Reflect)]
struct SoundEffect;

/// Drives a fade-in on a per-entity basis.
#[derive(Component, Reflect)]
struct FadeIn {
    /// Target linear volume.
    target: f32,
}

/// Drives a fade-out on a per-entity basis. The entity is despawned
/// once volume reaches near-silence.
#[derive(Component, Reflect)]
struct FadeOut;

/// Triggered when a button click sound should play.
#[derive(Event)]
struct PlayClickSound;

/// Manages background music crossfades and one-shot sound effects.
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        // Preload assets at startup.
        app.add_systems(Startup, preload_audio_assets);

        // Register music transition systems for every AppState variant.
        app.add_systems(OnEnter(AppState::Home), on_state_enter_music);
        app.add_systems(OnEnter(AppState::SaveSlots), on_state_enter_music);
        app.add_systems(OnEnter(AppState::Settings), on_state_enter_music);
        app.add_systems(OnEnter(AppState::MapExploration), on_state_enter_music);
        app.add_systems(OnEnter(AppState::LessonPlay), on_state_enter_music);
        app.add_systems(OnEnter(AppState::LessonSummary), on_state_enter_music);

        // Fade systems run every frame.
        app.add_systems(Update, (fade_in_system, fade_out_system));

        // Sync music volume when settings change.
        app.add_systems(
            Update,
            sync_music_volume.run_if(resource_changed::<Persistent<GameSettings>>),
        );

        // Button click detection.
        app.add_systems(Update, detect_button_clicks);

        // Observers for one-shot sounds.
        app.add_observer(on_answer_submitted);
        app.add_observer(on_play_click_sound);
    }
}

/// Loads all audio handles into the [`AudioAssets`] resource.
fn preload_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music = [
        MusicTrack::Menu,
        MusicTrack::Exploration,
        MusicTrack::Lesson,
    ]
    .into_iter()
    .map(|t| (t, asset_server.load(t.asset_path())))
    .collect();

    let sfx = [SfxKind::Click, SfxKind::Correct, SfxKind::Incorrect]
        .into_iter()
        .map(|k| (k, asset_server.load(k.asset_path())))
        .collect();

    commands.insert_resource(AudioAssets { music, sfx });
}

/// Called on every `AppState` enter. Compares the desired track with
/// [`CurrentMusic`] and either no-ops (same track) or crossfades.
fn on_state_enter_music(
    mut commands: Commands,
    state: Res<State<AppState>>,
    settings: Res<Persistent<GameSettings>>,
    audio_assets: Option<Res<AudioAssets>>,
    current_music: Option<Res<CurrentMusic>>,
    music_query: Query<Entity, With<BackgroundMusic>>,
) {
    let track = music_for_state(**state);

    // If the desired track is the same as the current one, do nothing.
    if let Some(ref current) = current_music
        && track == current.0
    {
        return;
    }

    // Fade out any existing background music.
    for entity in &music_query {
        commands
            .entity(entity)
            .remove::<BackgroundMusic>()
            .remove::<FadeIn>()
            .insert(FadeOut);
    }

    // Spawn new track only if assets are loaded.
    let Some(assets) = audio_assets else {
        return;
    };
    let Some(handle) = assets.music.get(&track) else {
        return;
    };

    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::SILENT,
            ..default()
        },
        BackgroundMusic,
        FadeIn {
            target: settings.music_volume * track.volume_multiplier(),
        },
    ));

    commands.insert_resource(CurrentMusic(track));
}

/// Gradually increases volume toward [`FadeIn::target`], then removes the
/// component.
fn fade_in_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AudioSink, &FadeIn)>,
    time: Res<Time>,
) {
    for (entity, mut sink, fade) in &mut query {
        let target = Volume::Linear(fade.target);
        let new_volume = sink
            .volume()
            .fade_towards(target, time.delta_secs() / FADE_DURATION);
        sink.set_volume(new_volume);

        if new_volume.to_linear() >= fade.target - 0.001 {
            sink.set_volume(target);
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

/// Gradually decreases volume toward silence, then despawns the entity.
fn fade_out_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AudioSink), With<FadeOut>>,
    time: Res<Time>,
) {
    for (entity, mut sink) in &mut query {
        let new_volume = sink
            .volume()
            .fade_towards(Volume::SILENT, time.delta_secs() / FADE_DURATION);
        sink.set_volume(new_volume);

        if new_volume.to_linear() < 0.001 {
            commands.entity(entity).despawn();
        }
    }
}

/// Detects button press interactions and triggers [`PlayClickSound`].
fn detect_button_clicks(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.trigger(PlayClickSound);
            // Only one click sound per frame is enough.
            return;
        }
    }
}

/// Observer: plays the correct/incorrect SFX when an answer is submitted.
fn on_answer_submitted(
    event: On<AnswerSubmitted>,
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
    audio_assets: Option<Res<AudioAssets>>,
) {
    let Some(assets) = audio_assets else {
        return;
    };

    let kind = match event.result {
        AnswerResult::Correct => SfxKind::Correct,
        AnswerResult::Incorrect => SfxKind::Incorrect,
    };

    let Some(handle) = assets.sfx.get(&kind) else {
        return;
    };

    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(settings.sfx_volume),
            ..default()
        },
        SoundEffect,
    ));
}

/// Observer: plays the click SFX.
fn on_play_click_sound(
    _event: On<PlayClickSound>,
    mut commands: Commands,
    settings: Res<Persistent<GameSettings>>,
    audio_assets: Option<Res<AudioAssets>>,
) {
    let Some(assets) = audio_assets else {
        return;
    };

    let Some(handle) = assets.sfx.get(&SfxKind::Click) else {
        return;
    };

    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(settings.sfx_volume),
            ..default()
        },
        SoundEffect,
    ));
}

/// Keeps the volume of currently playing background music in sync with
/// `GameSettings::music_volume`. Skips entities that are still fading in
/// to avoid conflicting with the fade animation.
fn sync_music_volume(
    settings: Res<Persistent<GameSettings>>,
    current_music: Option<Res<CurrentMusic>>,
    mut query: Query<(&mut AudioSink, Option<&FadeIn>), With<BackgroundMusic>>,
) {
    let multiplier = current_music
        .as_ref()
        .map_or(1.0, |cm| cm.0.volume_multiplier());
    for (mut sink, fade_in) in &mut query {
        if fade_in.is_some() {
            continue;
        }
        let target = Volume::Linear(settings.music_volume * multiplier);
        if sink.volume() != target {
            sink.set_volume(target);
        }
    }
}
