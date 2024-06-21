use crate::states::States::{Load, Play};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::{Progress, ProgressCounter, ProgressPlugin, ProgressSystem};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ProgressPlugin::new(Load).continue_to(Play),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_loading_state(LoadingState::new(Load).load_collection::<CharacterAssets>())
        .add_systems(
            Update,
            (track_fake_long_task.track_progress(), print_progress)
                .chain()
                .run_if(in_state(Load))
                .after(LoadingStateSet(Load)),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct CharacterAssets {
    #[asset(path = "character.png")]
    pub image: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 48.0, tile_size_y = 48.0, columns = 8, rows = 24))]
    pub layout: Handle<TextureAtlasLayout>,
}

fn print_progress(
    progress: Option<Res<ProgressCounter>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    if let Some(progress) = progress.map(|counter| counter.progress()) {
        if progress.done > *last_done {
            *last_done = progress.done;
            info!(
                "[Frame {}] Changed progress: {:?}",
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                    .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                    .unwrap_or(0.),
                progress
            );
        }
    }
}

fn track_fake_long_task(time: Res<Time>) -> Progress {
    if time.elapsed_seconds_f64() > 4.0 {
        info!("Long fake task is completed");
        true.into()
    } else {
        false.into()
    }
}
