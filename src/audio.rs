use crate::crystal::CrystalTouched;

use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(OnEnter(GameState::Playing), start_audio)
            .add_systems(
                Update,
                explosion_sound.run_if(
                    resource_exists::<ExplosionAudio>().and_then(in_state(GameState::Playing)),
                ),
            );
        //.add_systems(Update, explosion_sound))
    }
}

#[derive(Resource)]
struct ExplosionAudio(Handle<AudioInstance>);

fn start_audio(mut commands: Commands, audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.pause();
    let handle = audio
        .play(audio_assets.crystal_explosion.clone())
        // TODO: make this configurable
        .with_volume(0.3)
        .handle();
    commands.insert_resource(ExplosionAudio(handle));
}

fn explosion_sound(
    q_game_over: Query<Entity, Added<CrystalTouched>>,
    audio: Res<ExplosionAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if !q_game_over.is_empty() {
        if let Some(instance) = audio_instances.get_mut(&audio.0) {
            debug!("BANG!");
            instance.seek_to(0.0);
            instance.resume(AudioTween::default());
        }
    }
}
