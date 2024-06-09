use crate::villager::{AnimationIndices, Direction, Movement, Vec2Ext};
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
        let trigger_when_moving_up = move |In(entity): In<Entity>, movements: Query<&Movement>| {
            let movement = movements.get(entity).unwrap();
            movement.target().is_some()
        };

        let trigger_when_idle = move |In(entity): In<Entity>, movements: Query<&Movement>| {
            let movement = movements.get(entity).unwrap();
            movement.target().is_none()
        };

        let state = StateMachine::default()
            .trans::<IdleState, _>(trigger_when_moving_up, MovingState)
            .trans::<MovingState, _>(trigger_when_idle, IdleState)
            .set_trans_logging(true);

        Self {
            idle: IdleState,
            state,
        }
    }
}

fn update_animation_indices_in_idle_state(
    mut query: Query<(&mut AnimationIndices, &mut TextureAtlas), With<IdleState>>,
) {
    for (mut animation_index, mut atlas) in query.iter_mut() {
        if animation_index.first != 0 {
            animation_index.first = 0;
            animation_index.last = 7;
            atlas.index = animation_index.first;
        }
    }
}

fn update_animation_indices_in_moving_state(
    mut query: Query<
        (
            &Transform,
            &Movement,
            &mut AnimationIndices,
            &mut TextureAtlas,
        ),
        With<MovingState>,
    >,
) {
    for (transform, movement, mut animation_index, mut atlas) in query.iter_mut() {
        if let Some(target) = movement.target() {
            if let Some(direction) = transform.translation.xy().towards(&target).to_direction() {
                match direction {
                    Direction::Up => {
                        if animation_index.first != 40 {
                            animation_index.first = 40;
                            animation_index.last = 47;
                            atlas.index = animation_index.first;
                        }
                    }
                    Direction::Down => {
                        if animation_index.first != 32 {
                            animation_index.first = 32;
                            animation_index.last = 39;
                            atlas.index = animation_index.first;
                        }
                    }
                    Direction::Left => {
                        if animation_index.first != 56 {
                            animation_index.first = 56;
                            animation_index.last = 63;
                            atlas.index = animation_index.first;
                        }
                    }
                    Direction::Right => {
                        if animation_index.first != 48 {
                            animation_index.first = 48;
                            animation_index.last = 55;
                            atlas.index = animation_index.first;
                        }
                    }
                }
            }
        }
    }
}
