use crate::villager::{movement_system, AnimationIndices, Movement};
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::SquareDirection;
use seldom_state::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animation_indices_in_idle_state);
        app.add_systems(Update, update_animation_indices_in_moving_state.after(movement_system));
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
        let trigger_when_moving = move |In(entity): In<Entity>, movements: Query<&Movement, Without<GatheringTag>>| {
            if let Ok(movement) = movements.get(entity) {
                movement.target().is_some()
            } else {
                false
            }
        };

        let trigger_when_idle = move |In(entity): In<Entity>, movements: Query<&Movement, Without<GatheringTag>>| {
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

        Self { idle: IdleState, state }
    }
}

fn trigger_when_gathering(In(entity): In<Entity>, query: Query<&Movement, With<GatheringTag>>) -> bool {
    query.contains(entity)
}

fn trigger_when_not_gathering(In(entity): In<Entity>, query: Query<&Movement, With<GatheringTag>>) -> bool {
    !query.contains(entity)
}

fn update_animation_indices<T: Component>(
    query: &mut Query<(&Movement, &mut AnimationIndices, &mut TextureAtlas), With<T>>,
    animation_ranges: &[(SquareDirection, usize, usize)],
) {
    for (movement, mut animation_index, mut atlas) in query.iter_mut() {
        for &(direction, first, last) in animation_ranges {
            if movement.direction == direction && animation_index.first != first {
                animation_index.first = first;
                animation_index.last = last;
                atlas.index = animation_index.first;
            }
        }
    }
}

fn update_animation_indices_in_idle_state(
    mut query: Query<(&Movement, &mut AnimationIndices, &mut TextureAtlas), With<IdleState>>,
) {
    let animation_ranges = [
        (SquareDirection::North, 8, 15),
        (SquareDirection::South, 0, 7),
        (SquareDirection::West, 16, 23),
        (SquareDirection::East, 24, 31),
    ];
    update_animation_indices(&mut query, &animation_ranges);
}

fn update_animation_indices_in_moving_state(
    mut query: Query<(&Movement, &mut AnimationIndices, &mut TextureAtlas), With<MovingState>>,
) {
    let animation_ranges = [
        (SquareDirection::North, 40, 47),
        (SquareDirection::South, 32, 39),
        (SquareDirection::East, 48, 55),
        (SquareDirection::West, 56, 63),
    ];
    update_animation_indices(&mut query, &animation_ranges);
}

fn update_animation_indices_in_gathering_state(
    mut query: Query<(&Movement, &mut AnimationIndices, &mut TextureAtlas), With<GatheringState>>,
) {
    let animation_ranges = [
        (SquareDirection::North, 104, 111),
        (SquareDirection::South, 96, 103),
        (SquareDirection::East, 120, 127),
        (SquareDirection::West, 112, 119),
    ];
    update_animation_indices(&mut query, &animation_ranges);
}
