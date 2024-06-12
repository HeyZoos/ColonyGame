use crate::ext::Vec2Ext;
use crate::villager::{find_path, Movement};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use big_brain::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

const MAX_DISTANCE: f32 = 1.0;

#[derive(Clone, Component, Debug)]
pub struct Bush;

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (move_to_nearest_system::<Bush>,).in_set(BigBrainSet::Actions),
                (work_need_scorer_system,).in_set(BigBrainSet::Scorers),
            ),
        );
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct MoveToNearest<T: Clone + Component + Debug> {
    _marker: PhantomData<T>,
    goal: Option<Vec2>,
}

impl<T: Clone + Component + Debug> MoveToNearest<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            goal: None,
        }
    }
}

impl<T: Clone + Component + Debug> Default for MoveToNearest<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn move_to_nearest_system<T: Clone + Component + Debug>(
    world: Res<crate::worldgen::World>,
    mut tiles: Query<&mut TilePos, With<T>>,
    mut thinkers: Query<(&mut Transform, &mut Movement), (With<HasThinker>, Without<T>)>,
    mut action_query: Query<(&Actor, &mut ActionState, &mut MoveToNearest<T>, &ActionSpan)>,
) {
    for (actor, mut action_state, mut move_to, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                let (actor_transform, mut actor_movement) = thinkers.get_mut(actor.0).unwrap();
                let goal_transform = tiles
                    .iter_mut()
                    .map(|t| {
                        (
                            Vec2 {
                                x: t.x as f32 * 16.0,
                                y: t.y as f32 * 16.0,
                            },
                            t,
                        )
                    })
                    .min_by(|(a, _), (b, _)| {
                        let delta_a = *a - actor_transform.translation.xy();
                        let delta_b = *b - actor_transform.translation.xy();
                        delta_a.length().partial_cmp(&delta_b.length()).unwrap()
                    });

                if let Some((goal, _tilepos)) = goal_transform {
                    info!(
                        "Found {:?} at ({}, {})",
                        std::any::type_name::<T>(),
                        goal.x,
                        goal.y
                    );
                    move_to.goal = Some(goal.xy());
                    *action_state = ActionState::Executing;
                }

                let path_option = find_path(
                    &world,
                    actor_transform.translation.xy().to_grid_space().to_coord(),
                    move_to.goal.unwrap().to_grid_space().to_coord(),
                );

                if let Some(path) = path_option {
                    info!("Set path to {:?}", std::any::type_name::<T>());
                    actor_movement.path = path.0;
                }

                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                let (actor_transform, _actor_movement) = thinkers.get_mut(actor.0).unwrap();
                let delta = move_to.goal.unwrap() - actor_transform.translation.xy();
                let distance = delta.length();

                trace!("Distance: {}", distance);

                if distance > MAX_DISTANCE {
                    trace!("Stepping closer.");
                    // Movement should be handled by the movement system
                } else {
                    info!("We got there!");
                    *action_state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct WorkNeedScorer;

pub fn work_need_scorer_system(mut query: Query<(&Actor, &mut Score), With<WorkNeedScorer>>) {
    for (Actor(_actor), mut score) in &mut query {
        score.set(1.0);
    }
}

#[derive(Clone, Component, Debug)]
pub struct GatherAction;

pub fn gather_action_system(
    world: Res<crate::worldgen::World>,
    mut bushes: Query<&mut TilePos, With<Bush>>,
    mut thinkers: Query<(&mut Transform, &mut Movement), (With<HasThinker>, Without<Bush>)>,
    mut action_query: Query<(&Actor, &mut ActionState, &mut GatherAction, &ActionSpan)>,
) {
    for (actor, mut action_state, mut move_to, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                *action_state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
