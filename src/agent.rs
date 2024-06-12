use crate::blackboard::Blackboard;
use crate::ext::Vec2Ext;
use crate::villager::{find_path, Movement};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use big_brain::prelude::*;
use serde_json::json;
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
                (move_to_nearest_system::<Bush>, gather_action_system).in_set(BigBrainSet::Actions),
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
    mut tiles: Query<(Entity, &mut TilePos), With<T>>,
    mut agents: Query<
        (&mut Blackboard, &mut Transform, &mut Movement),
        (With<HasThinker>, Without<T>),
    >,
    mut action_query: Query<(&Actor, &mut ActionState, &mut MoveToNearest<T>, &ActionSpan)>,
) {
    for (actor, mut action_state, mut move_to, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                let (mut blackboard, actor_transform, mut actor_movement) =
                    agents.get_mut(actor.0).unwrap();
                let goal_transform = tiles
                    .iter_mut()
                    .map(|(entity, t)| {
                        let x = t.x as f32 * 16.0;
                        let y = t.y as f32 * 16.0;
                        (entity, t, Vec2 { x, y })
                    })
                    // TODO(jesse): Order by distance and choose randomly from the top 10 closest
                    // This is a hack so they won't get stuck looking for inaccessible items
                    // It will also have the added benefit of spreading the agents out
                    .min_by(|(_, _, a), (_, _, b)| {
                        let delta_a = *a - actor_transform.translation.xy();
                        let delta_b = *b - actor_transform.translation.xy();
                        delta_a.length().partial_cmp(&delta_b.length()).unwrap()
                    });

                if let Some((entity, _tilepos, goal)) = goal_transform {
                    info!(
                        "Found {:?} at ({}, {})",
                        std::any::type_name::<T>(),
                        goal.x,
                        goal.y
                    );
                    move_to.goal = Some(goal.xy());
                    *action_state = ActionState::Executing;
                    blackboard.insert("bush", json!(entity));
                }

                let path_option = find_path(
                    &world,
                    actor_transform.translation.xy().to_grid_space().to_coord(),
                    move_to.goal.unwrap().to_grid_space().to_coord(),
                );

                if let Some(path) = path_option {
                    info!("Set path to {:?}", std::any::type_name::<T>());
                    actor_movement.path = path.0;
                    *action_state = ActionState::Executing;
                }
            }
            ActionState::Executing => {
                let (_, actor_transform, _actor_movement) = agents.get_mut(actor.0).unwrap();
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

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct GatherAction;

pub fn gather_action_system(
    mut commands: Commands,
    mut agents: Query<
        (&mut Blackboard, &mut Transform, &mut Movement),
        (With<HasThinker>, Without<Bush>),
    >,
    mut action_query: Query<(&Actor, &mut ActionState, &GatherAction, &ActionSpan)>,
) {
    for (actor, mut action_state, _action, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                info!("Gathering!");

                let (mut blackboard, _, _) = agents.get_mut(actor.0).unwrap();

                let value = blackboard.get("bush");
                let raw_entity_number = value.as_number().unwrap();
                let raw_entity_u64 = raw_entity_number.as_u64().unwrap();
                let raw_entity_u32 = raw_entity_u64 as u32;

                commands.entity(Entity::from_raw(raw_entity_u32)).despawn();

                blackboard.remove("bush");

                *action_state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
