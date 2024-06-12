#![allow(clippy::type_complexity)]

pub mod actions;
pub mod ai;
pub mod animation;
pub mod audio;
pub mod ext;
pub mod inspector;
pub mod loading;
pub mod menu;
pub mod player;
pub mod villager;
pub mod worldgen;

use crate::actions::ActionsPlugin;
use crate::animation::AnimationPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::villager::VillagerPlugin;
use crate::worldgen::WorldgenPlugin;

use crate::ai::AIPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCamPlugin;
use big_brain::BigBrainPlugin;
use seldom_state::StateMachinePlugin;
use crate::inspector::InspectorPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            ActionsPlugin,
            AIPlugin,
            AnimationPlugin,
            BigBrainPlugin::new(PreUpdate),
            InspectorPlugin,
            InternalAudioPlugin,
            LoadingPlugin,
            MenuPlugin,
            PanCamPlugin,
            PlayerPlugin,
            StateMachinePlugin,
            VillagerPlugin,
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
            WorldgenPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
