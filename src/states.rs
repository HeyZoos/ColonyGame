use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<States>();
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum States {
    #[default]
    LoadMenu,
    Menu,
    Worldgen,
    LoadPlay,
    Play,
}
