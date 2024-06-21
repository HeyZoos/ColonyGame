#![allow(clippy::type_complexity)]

pub mod actions;
pub mod agent;
pub mod animation;
mod assets;
pub mod audio;
pub mod blackboard;
pub mod ext;
pub mod inspector;
pub mod loading;
pub mod menu;
pub mod player;
pub mod reservations;
mod states;
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

use crate::agent::AgentPlugin;
use crate::reservations::ReservationsPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pancam::PanCamPlugin;
use big_brain::BigBrainPlugin;
use seldom_state::StateMachinePlugin;

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
            AgentPlugin,
            AnimationPlugin,
            LoadingPlugin,
            assets::AssetsPlugin,
            BigBrainPlugin::new(PreUpdate),
            // crate::inspector::InspectorPlugin,
            InternalAudioPlugin,
            MenuPlugin,
            PanCamPlugin,
            PlayerPlugin,
            ReservationsPlugin,
            states::StatesPlugin,
            StateMachinePlugin,
            VillagerPlugin,
            WorldgenPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            // app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
