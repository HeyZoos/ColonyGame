use crate::ext::*;
use crate::villager::{AnimationIndices, Direction, Movement};
use bevy::prelude::*;
use seldom_state::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animation_indices_in_idle_state);
        app.add_systems(Update, update_animation_indices_in_moving_state);
        app.add_systems(Update, update_animation_indices_in_gathering_state);
    }
}

#[derive(Clone, Component)]
struct IdleState;

#[derive(Clone, Component)]
struct MovingState;

#[derive(Clone, Component)]
struct GatheringState;

#[derive(Clone, Component)]
pub struct GatheringTag;

#[derive(Bundle)]
pub struct AnimationBundle {
    idle: IdleState,
    state: StateMachine,
}

impl Default for AnimationBundle {
    fn default() -> Self {
        let trigger_when_moving =
            move |In(entity): In<Entity>, movements: Query<&Movement, Without<GatheringTag>>| {
                if let Ok(movement) = movements.get(entity) {
                    movement.target().is_some()
                } else {
                    false
                }
            };

        let trigger_when_idle =
            move |In(entity): In<Entity>, movements: Query<&Movement, Without<GatheringTag>>| {
                if let Ok(movement) = movements.get(entity) {
                    movement.target().is_none()
                } else {
                    false
                }
            };

        let state = StateMachine::default()
            .trans::<IdleState, _>(trigger_when_moving, MovingState)
            .trans::<IdleState, _>(trigger_when_gathering, GatheringState)
            .trans::<MovingState, _>(trigger_when_gathering, GatheringState)
            .trans::<GatheringState, _>(trigger_when_not_gathering, IdleState)
            .trans::<MovingState, _>(trigger_when_idle, IdleState)
            .set_trans_logging(true);

        Self {
            idle: IdleState,
            state,
        }
    }
}

fn trigger_when_gathering(
    In(entity): In<Entity>,
    query: Query<&Movement, With<GatheringTag>>,
) -> bool {
    query.contains(entity)
}

fn trigger_when_not_gathering(
    In(entity): In<Entity>,
    query: Query<&Movement, With<GatheringTag>>,
) -> bool {
    !query.contains(entity)
}

fn update_animation_indices_in_idle_state(
    mut query: Query<(&Movement, &mut AnimationIndices, &mut TextureAtlas), With<IdleState>>,
) {
    for (movement, mut animation_index, mut atlas) in query.iter_mut() {
        match movement.direction {
            Direction::Up => {
                if animation_index.first != 8 {
                    animation_index.first = 8;
                    animation_index.last = 15;
                    atlas.index = animation_index.first;
                }
            }
            Direction::Down => {
                if animation_index.first != 0 {
                    animation_index.first = 0;
                    animation_index.last = 7;
                    atlas.index = animation_index.first;
                }
            }
            Direction::Left => {
                if animation_index.first != 16 {
                    animation_index.first = 16;
                    animation_index.last = 23;
                    atlas.index = animation_index.first;
                }
            }
            Direction::Right => {
                if animation_index.first != 24 {
                    animation_index.first = 24;
                    animation_index.last = 31;
                    atlas.index = animation_index.first;
                }
            }
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
            if let Some(direction) = transform.translation.xy().to_direction_towards(&target) {
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

fn update_animation_indices_in_gathering_state(
    mut query: Query<(&Movement, &mut AnimationIndices, &mut TextureAtlas), With<GatheringState>>,
) {
    for (movement, mut animation_index, mut atlas) in query.iter_mut() {
        match movement.direction {
            Direction::Up => {
                if animation_index.first != 104 {
                    animation_index.first = 104;
                    animation_index.last = 111;
                    atlas.index = animation_index.first;
                }
            }
            Direction::Down => {
                if animation_index.first != 96 {
                    animation_index.first = 96;
                    animation_index.last = 103;
                    atlas.index = animation_index.first;
                }
            }
            Direction::Left => {
                if animation_index.first != 112 {
                    animation_index.first = 112;
                    animation_index.last = 119;
                    atlas.index = animation_index.first;
                }
            }
            Direction::Right => {
                if animation_index.first != 120 {
                    animation_index.first = 120;
                    animation_index.last = 127;
                    atlas.index = animation_index.first;
                }
            }
        }
    }
}
