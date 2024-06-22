#![allow(clippy::type_complexity)]

pub mod agent;
pub mod animation;
mod assets;
pub mod audio;
pub mod blackboard;
pub mod ext;
mod inspector;
pub mod loading;
pub mod menu;
pub mod reservations;
mod states;
pub mod villager;
pub mod worldgen;

use crate::animation::AnimationPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::villager::VillagerPlugin;
use crate::worldgen::WorldgenPlugin;

use crate::agent::AgentPlugin;
use crate::reservations::ReservationsPlugin;
use bevy::app::App;
use bevy::prelude::*;
use bevy_pancam::PanCamPlugin;
use big_brain::BigBrainPlugin;
use seldom_state::StateMachinePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AgentPlugin,
            AnimationPlugin,
            LoadingPlugin,
            assets::AssetsPlugin,
            BigBrainPlugin::new(PreUpdate),
            inspector::InspectorPlugin,
            InternalAudioPlugin,
            MenuPlugin,
            PanCamPlugin,
            ReservationsPlugin,
            states::StatesPlugin,
            StateMachinePlugin,
            VillagerPlugin,
            WorldgenPlugin,
        ));
    }
}
