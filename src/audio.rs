use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::assets::AudioAssets;
use crate::states::States::Menu;

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_systems(Update, play_sound_on_clicked_system.run_if(in_state(Menu)));
    }
}

fn play_sound_on_clicked_system(
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in interactions.iter() {
        match interaction {
            Interaction::Pressed => {
                audio.play(audio_assets.hover.clone());
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
