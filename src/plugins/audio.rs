use std::collections::HashMap;

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

use crate::data::{AnswerResult, GameSettings};
use crate::questions::AnswerSubmitted;
use crate::states::AppState;
use crate::ui::components::ButtonActivated;

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
        AppState::Home | AppState::Settings | AppState::SaveSlots => MusicTrack::Menu,
        AppState::ThemeExploration => MusicTrack::Exploration,
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
    start: f32,
    target: f32,
    elapsed: f32,
}

/// Drives a fade-out over the configured duration.
#[derive(Component, Reflect)]
struct FadeOut {
    start: f32,
    elapsed: f32,
}

/// Triggered when a button click sound should play.
#[derive(Event)]
struct PlayClickSound;

/// Manages background music crossfades and one-shot sound effects.
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, preload_audio_assets);
        app.add_systems(
            Update,
            (
                update_music_for_state.run_if(state_changed::<AppState>),
                sync_music_volume.run_if(resource_changed::<GameSettings>),
                fade_in_system,
                fade_out_system,
            )
                .chain(),
        );

        app.add_systems(Update, detect_button_clicks);

        // Observers for one-shot sounds.
        app.add_observer(on_answer_submitted.run_if(resource_exists::<AudioAssets>));
        app.add_observer(on_play_click_sound.run_if(resource_exists::<AudioAssets>));
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

/// Starts music for the current state and crossfades away from the previous track.
fn update_music_for_state(
    mut commands: Commands,
    state: Res<State<AppState>>,
    settings: Res<GameSettings>,
    audio_assets: Res<AudioAssets>,
    current_music: Option<Res<CurrentMusic>>,
    music_query: Query<(Entity, Option<&AudioSink>), With<BackgroundMusic>>,
) {
    let track = music_for_state(**state);
    if current_music
        .as_ref()
        .is_some_and(|current| current.0 == track)
    {
        return;
    }
    let Some(handle) = audio_assets.music.get(&track) else {
        return;
    };

    for (entity, sink) in &music_query {
        let start = sink.map_or(0.0, |sink| sink.volume().to_linear());
        commands
            .entity(entity)
            .remove::<BackgroundMusic>()
            .remove::<FadeIn>()
            .insert(FadeOut {
                start,
                elapsed: 0.0,
            });
    }

    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::SILENT,
            ..default()
        },
        BackgroundMusic,
        FadeIn {
            start: 0.0,
            target: settings.music_volume * track.volume_multiplier(),
            elapsed: 0.0,
        },
    ));
    commands.insert_resource(CurrentMusic(track));
}

/// Fades music in over the configured duration.
fn fade_in_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AudioSink, &mut FadeIn)>,
    time: Res<Time>,
) {
    for (entity, mut sink, mut fade) in &mut query {
        fade.elapsed = (fade.elapsed + time.delta_secs()).min(FADE_DURATION);
        let progress = fade.elapsed / FADE_DURATION;
        let volume = (fade.target - fade.start).mul_add(progress, fade.start);
        sink.set_volume(Volume::Linear(volume));

        if fade.elapsed >= FADE_DURATION {
            sink.set_volume(Volume::Linear(fade.target));
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

/// Fades music out over the configured duration, then despawns its player.
fn fade_out_system(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&mut AudioSink>, &mut FadeOut)>,
    time: Res<Time>,
) {
    for (entity, sink, mut fade) in &mut query {
        fade.elapsed = (fade.elapsed + time.delta_secs()).min(FADE_DURATION);
        let progress = fade.elapsed / FADE_DURATION;
        if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(fade.start * (1.0 - progress)));
        }

        if fade.elapsed >= FADE_DURATION {
            commands.entity(entity).despawn();
        }
    }
}

/// Triggers a click sound for activated buttons.
fn detect_button_clicks(mut commands: Commands, mut activations: MessageReader<ButtonActivated>) {
    if activations.read().count() > 0 {
        commands.trigger(PlayClickSound);
    }
}

/// Plays the correct or incorrect SFX when an answer is submitted.
fn on_answer_submitted(
    event: On<AnswerSubmitted>,
    mut commands: Commands,
    settings: Res<GameSettings>,
    audio_assets: Res<AudioAssets>,
) {
    let kind = match event.result {
        AnswerResult::Correct => SfxKind::Correct,
        AnswerResult::Incorrect => SfxKind::Incorrect,
    };
    spawn_sfx(&mut commands, &audio_assets, kind, settings.sfx_volume);
}

/// Plays the click SFX.
fn on_play_click_sound(
    _event: On<PlayClickSound>,
    mut commands: Commands,
    settings: Res<GameSettings>,
    audio_assets: Res<AudioAssets>,
) {
    spawn_sfx(
        &mut commands,
        &audio_assets,
        SfxKind::Click,
        settings.sfx_volume,
    );
}

fn spawn_sfx(commands: &mut Commands, assets: &AudioAssets, kind: SfxKind, volume: f32) {
    let Some(handle) = assets.sfx.get(&kind) else {
        return;
    };
    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(volume),
            ..default()
        },
        SoundEffect,
    ));
}

/// Keeps the volume of active music in sync with the user's setting.
fn sync_music_volume(
    settings: Res<GameSettings>,
    current_music: Option<Res<CurrentMusic>>,
    mut fading_query: Query<(&mut FadeIn, Option<&AudioSink>), With<BackgroundMusic>>,
    mut steady_query: Query<&mut AudioSink, (With<BackgroundMusic>, Without<FadeIn>)>,
) {
    let multiplier = current_music
        .as_ref()
        .map_or(1.0, |current| current.0.volume_multiplier());
    let target = settings.music_volume * multiplier;

    for (mut fade, sink) in &mut fading_query {
        if (fade.target - target).abs() <= f32::EPSILON {
            continue;
        }
        fade.start = sink.map_or(0.0, |sink| sink.volume().to_linear());
        fade.target = target;
        fade.elapsed = 0.0;
    }

    let target = Volume::Linear(target);
    for mut sink in &mut steady_query {
        if sink.volume() != target {
            sink.set_volume(target);
        }
    }
}
