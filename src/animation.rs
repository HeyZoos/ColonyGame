use crate::villager::{AnimationIndices, Movement};
use bevy::prelude::*;
use seldom_state::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animation_indices_in_idle_state);
        app.add_systems(Update, update_animation_indices_in_moving_up_state);
        app.add_systems(Update, update_animation_indices_in_moving_down_state);
        app.add_systems(Update, update_animation_indices_in_moving_left_state);
        app.add_systems(Update, update_animation_indices_in_moving_right_state);
    }
}

#[derive(Clone, Component)]
struct IdleState;

#[derive(Clone, Component)]
struct MovingUpState;

#[derive(Clone, Component)]
struct MovingDownState;

#[derive(Clone, Component)]
struct MovingLeftState;

#[derive(Clone, Component)]
struct MovingRightState;

#[derive(Bundle)]
pub struct AnimationBundle {
    idle: IdleState,
    state: StateMachine,
}

impl Default for AnimationBundle {
    fn default() -> Self {
        let trigger_when_moving_up =
            move |In(entity): In<Entity>,
                  transforms: Query<&Transform>,
                  pathfinders: Query<&Movement>| {
                let transform = transforms.get(entity).unwrap();
                let pathfinder = pathfinders.get(entity).unwrap();
                if let Some(current_target) = pathfinder.current_target {
                    let current_target_translation = Vec3 {
                        x: current_target.x as f32 * 16.0,
                        y: current_target.y as f32 * 16.0,
                        z: transform.translation.z,
                    };

                    (current_target_translation - transform.translation)
                        .normalize()
                        .y
                        > 0.9
                } else {
                    false
                }
            };

        let trigger_when_moving_down =
            move |In(entity): In<Entity>,
                  transforms: Query<&Transform>,
                  pathfinders: Query<&Movement>| {
                let transform = transforms.get(entity).unwrap();
                let pathfinder = pathfinders.get(entity).unwrap();
                if let Some(current_target) = pathfinder.current_target {
                    let current_target_translation = Vec3 {
                        x: current_target.x as f32 * 16.0,
                        y: current_target.y as f32 * 16.0,
                        z: transform.translation.z,
                    };

                    (current_target_translation - transform.translation)
                        .normalize()
                        .y
                        < -0.9
                } else {
                    false
                }
            };

        let trigger_when_moving_right =
            move |In(entity): In<Entity>,
                  transforms: Query<&Transform>,
                  pathfinders: Query<&Movement>| {
                let transform = transforms.get(entity).unwrap();
                let pathfinder = pathfinders.get(entity).unwrap();
                if let Some(current_target) = pathfinder.current_target {
                    let current_target_translation = Vec3 {
                        x: current_target.x as f32 * 16.0,
                        y: current_target.y as f32 * 16.0,
                        z: transform.translation.z,
                    };

                    (current_target_translation - transform.translation)
                        .normalize()
                        .x
                        > 0.9
                } else {
                    false
                }
            };

        let trigger_when_moving_left =
            move |In(entity): In<Entity>,
                  transforms: Query<&Transform>,
                  pathfinders: Query<&Movement>| {
                let transform = transforms.get(entity).unwrap();
                let pathfinder = pathfinders.get(entity).unwrap();
                if let Some(current_target) = pathfinder.current_target {
                    let current_target_translation = Vec3 {
                        x: current_target.x as f32 * 16.0,
                        y: current_target.y as f32 * 16.0,
                        z: transform.translation.z,
                    };

                    (current_target_translation - transform.translation)
                        .normalize()
                        .x
                        < -0.9
                } else {
                    false
                }
            };

        let trigger_when_idle = move |In(entity): In<Entity>, pathfinders: Query<&Movement>| {
            let pathfinder = pathfinders.get(entity).unwrap();
            pathfinder.current_target.is_none()
        };

        let state = StateMachine::default()
            .trans::<IdleState, _>(trigger_when_moving_up, MovingUpState)
            .trans::<IdleState, _>(trigger_when_moving_down, MovingDownState)
            .trans::<IdleState, _>(trigger_when_moving_left, MovingLeftState)
            .trans::<IdleState, _>(trigger_when_moving_right, MovingRightState)
            .trans::<MovingUpState, _>(trigger_when_idle, IdleState)
            .trans::<MovingUpState, _>(trigger_when_moving_down, MovingDownState)
            .trans::<MovingUpState, _>(trigger_when_moving_left, MovingLeftState)
            .trans::<MovingUpState, _>(trigger_when_moving_right, MovingRightState)
            .trans::<MovingDownState, _>(trigger_when_idle, IdleState)
            .trans::<MovingDownState, _>(trigger_when_moving_up, MovingUpState)
            .trans::<MovingDownState, _>(trigger_when_moving_left, MovingLeftState)
            .trans::<MovingDownState, _>(trigger_when_moving_right, MovingRightState)
            .trans::<MovingLeftState, _>(trigger_when_idle, IdleState)
            .trans::<MovingLeftState, _>(trigger_when_moving_up, MovingUpState)
            .trans::<MovingLeftState, _>(trigger_when_moving_down, MovingDownState)
            .trans::<MovingLeftState, _>(trigger_when_moving_right, MovingRightState)
            .trans::<MovingRightState, _>(trigger_when_idle, IdleState)
            .trans::<MovingRightState, _>(trigger_when_moving_up, MovingUpState)
            .trans::<MovingRightState, _>(trigger_when_moving_down, MovingDownState)
            .trans::<MovingRightState, _>(trigger_when_moving_left, MovingLeftState)
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

fn update_animation_indices_in_moving_up_state(
    mut query: Query<(&mut AnimationIndices, &mut TextureAtlas), With<MovingUpState>>,
) {
    for (mut animation_index, mut atlas) in query.iter_mut() {
        if animation_index.first != 40 {
            animation_index.first = 40;
            animation_index.last = 47;
            atlas.index = animation_index.first;
        }
    }
}

fn update_animation_indices_in_moving_down_state(
    mut query: Query<(&mut AnimationIndices, &mut TextureAtlas), With<MovingDownState>>,
) {
    for (mut animation_index, mut atlas) in query.iter_mut() {
        if animation_index.first != 32 {
            animation_index.first = 32;
            animation_index.last = 39;
            atlas.index = animation_index.first;
        }
    }
}

fn update_animation_indices_in_moving_right_state(
    mut query: Query<(&mut AnimationIndices, &mut TextureAtlas), With<MovingRightState>>,
) {
    for (mut animation_index, mut atlas) in query.iter_mut() {
        if animation_index.first != 48 {
            animation_index.first = 48;
            animation_index.last = 55;
            atlas.index = animation_index.first;
        }
    }
}

fn update_animation_indices_in_moving_left_state(
    mut query: Query<(&mut AnimationIndices, &mut TextureAtlas), With<MovingLeftState>>,
) {
    for (mut animation_index, mut atlas) in query.iter_mut() {
        if animation_index.first != 56 {
            animation_index.first = 56;
            animation_index.last = 63;
            atlas.index = animation_index.first;
        }
    }
}
