use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::{ProgressCounter, ProgressPlugin};

use crate::states::States::{LoadMenu, LoadPlay, Menu, Play, Worldgen};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ProgressPlugin::new(LoadMenu).continue_to(Menu),
            ProgressPlugin::new(LoadPlay).continue_to(Play),
            ProgressPlugin::new(Worldgen).continue_to(LoadPlay),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(
            Update,
            (print_progress,)
                .chain()
                .run_if(in_state(LoadMenu))
                .after(LoadingStateSet(LoadMenu)),
        )
        .add_systems(
            Update,
            (print_progress,)
                .chain()
                .run_if(in_state(Worldgen))
                .after(LoadingStateSet(Worldgen)),
        )
        .add_systems(
            Update,
            (print_progress,)
                .chain()
                .run_if(in_state(LoadPlay))
                .after(LoadingStateSet(LoadPlay)),
        );
    }
}

fn print_progress(
    progress: Option<Res<ProgressCounter>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    if let Some(progress) = progress.map(|counter| counter.progress()) {
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
