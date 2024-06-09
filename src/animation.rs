use crate::villager::{AnimationIndices, Pathfinder};
use bevy::prelude::*;
use seldom_state::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animation_indices_in_idle_state);
        app.add_systems(Update, update_animation_indices_in_moving_state);
    }
}

#[derive(Clone, Component)]
struct IdleState;

#[derive(Clone, Component)]
struct MovingState;

#[derive(Bundle)]
pub struct AnimationBundle {
    idle: IdleState,
    state: StateMachine,
}

impl Default for AnimationBundle {
    fn default() -> Self {
        let trigger_when_moving = move |In(entity): In<Entity>, pathfinders: Query<&Pathfinder>| {
            let pathfinder = pathfinders.get(entity).unwrap();
            pathfinder.current_target.is_some()
        };

        let trigger_when_idle = move |In(entity): In<Entity>, pathfinders: Query<&Pathfinder>| {
            let pathfinder = pathfinders.get(entity).unwrap();
            pathfinder.current_target.is_none()
        };

        let state = StateMachine::default()
            .trans::<IdleState, _>(trigger_when_moving, MovingState)
            .trans::<MovingState, _>(trigger_when_idle, IdleState)
            .set_trans_logging(true);

        Self {
            idle: IdleState,
            state,
        }
    }
}

fn update_animation_indices_in_idle_state(
    mut animation_indices: Query<(&mut AnimationIndices, &mut TextureAtlas), With<IdleState>>,
) {
    for (mut animation_index, mut atlas) in animation_indices.iter_mut() {
        if animation_index.first != 0 {
            animation_index.first = 0;
            animation_index.last = 7;
            atlas.index = animation_index.first;
        }
    }
}

fn update_animation_indices_in_moving_state(
    mut animation_indices: Query<(&mut AnimationIndices, &mut TextureAtlas), With<MovingState>>,
) {
    for (mut animation_index, mut atlas) in animation_indices.iter_mut() {
        if animation_index.first != 32 {
            animation_index.first = 32;
            animation_index.last = 39;
            atlas.index = animation_index.first;
        }
    }
}
