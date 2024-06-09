use crate::villager::Pathfinder;
use bevy::prelude::*;
use seldom_state::prelude::*;

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

        let state = StateMachine::default()
            .trans::<IdleState, _>(trigger_when_moving, MovingState)
            .set_trans_logging(true);

        Self {
            idle: IdleState,
            state,
        }
    }
}
