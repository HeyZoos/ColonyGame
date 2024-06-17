/// NOTE: Avoid using action state cancelled

use crate::animation::GatheringTag;
use crate::blackboard::Blackboard;
use crate::ext::Vec2Ext;
use crate::villager::{find_path, Movement};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use big_brain::prelude::*;
use rand::prelude::IteratorRandom;
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
        
        app.add_systems(Update, update_z_system);
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

                let mut goal_transforms: Vec<_> = tiles
                    .iter_mut()
                    .map(|(entity, t)| {
                        let x = t.x as f32 * 16.0;
                        let y = t.y as f32 * 16.0;
                        (entity, t, Vec2 { x, y })
                    })
                    .collect();

                // Order by distance
                goal_transforms.sort_by(|(_, _, a), (_, _, b)| {
                    let delta_a = *a - actor_transform.translation.xy();
                    let delta_b = *b - actor_transform.translation.xy();
                    delta_a.length().partial_cmp(&delta_b.length()).unwrap()
                });

                // Choose randomly from the top 10 closest
                let goal_transform = goal_transforms
                    .iter()
                    .take(10)
                    .choose(&mut rand::thread_rng());

                if let Some((entity, _tilepos, goal)) = goal_transform {
                    info!(
                        "Found {:?} at ({}, {})",
                        std::any::type_name::<T>(),
                        goal.x,
                        goal.y
                    );
                    move_to.goal = Some(goal.xy());
                    *action_state = ActionState::Executing;
                    blackboard.insert("bush", json!(*entity));
                }

                let start_coord = actor_transform.translation.xy().to_grid_space().to_coord();

                let path_option = find_path(
                    &world,
                    start_coord,
                    move_to.goal.unwrap().to_grid_space().to_coord(),
                );

                if let Some(mut path) = path_option {
                    // We don't want to include the first goal if it is the same as the start
                    if path.0.first() == Some(&start_coord) {
                        path.0.remove(0);
                    }

                    info!("Set path to {:?}", std::any::type_name::<T>());
                    actor_movement.path = path.0;
                    *action_state = ActionState::Executing;
                } else {
                    *action_state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                let (_, actor_transform, _actor_movement) = agents.get_mut(actor.0).unwrap();
                let delta = move_to.goal.unwrap() - actor_transform.translation.xy();
                let distance = delta.length();

                trace!("Distance: {}", distance);

                if distance > MAX_DISTANCE {
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

#[derive(Component)]
pub struct GatheringTimer(Timer);

pub fn gather_action_system(
    time: Res<Time>,
    mut commands: Commands,
    mut agents: Query<
        (
            &mut Blackboard,
            &mut Transform,
            &mut Movement,
            &mut GatheringTimer,
        ),
        (With<HasThinker>, Without<Bush>),
    >,
    mut action_query: Query<(&Actor, &mut ActionState, &GatherAction, &ActionSpan)>,
) {
    for (actor, mut action_state, _action, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                // This tag will cause the animation state machine to put it in the gathering state
                commands.entity(actor.0).insert(GatheringTag {});

                // Add a timer for how long to stay gathering
                commands
                    .entity(actor.0)
                    .insert(GatheringTimer(Timer::from_seconds(3.0, TimerMode::Once)));

                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                // Update the timer
                if let Ok((mut blackboard, _, _, mut timer)) = agents.get_mut(actor.0) {
                    timer.0.tick(time.delta());

                    if timer.0.finished() {
                        let value = blackboard.get("bush");
                        if let Some(raw_entity_number) = value.as_number() {
                            let raw_entity_u64 = raw_entity_number.as_u64().unwrap();
                            let raw_entity_u32 = raw_entity_u64 as u32;

                            let entity_option =
                                commands.get_entity(Entity::from_raw(raw_entity_u32));

                            if let Some(entity) = entity_option {
                                let entity_id = entity.id(); // Store the entity ID to avoid multiple mutable borrows
                                commands.entity(entity_id).despawn();
                                *action_state = ActionState::Success;
                            } else {
                                *action_state = ActionState::Failure;
                            }
                        }

                        blackboard.remove("bush");
                        commands.entity(actor.0).remove::<GatheringTag>();
                        commands.entity(actor.0).remove::<GatheringTimer>();
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_z_system(mut query: Query<&mut Transform, With<HasThinker>>) {
    for mut transform in query.iter_mut() {
        // Inverse the y value relationship to z and add to the base value of 10.0
        transform.translation.z = 10.0 + (10000.0 - transform.translation.y) / 100.0;
    }
}
