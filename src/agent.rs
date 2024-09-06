/// NOTE: Avoid using action state cancelled
use crate::animation::GatheringTag;
use crate::blackboard::Blackboard;
use crate::ext::Vec2Ext;
use crate::reservations::{
    RemoveReservation, Reservable, Reservation, ReservationRequest, ReservationRequestBuilder,
    Reserved,
};
use crate::states::States::Play;
use crate::villager::{find_path, Movement};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;
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
            )
                .run_if(in_state(Play)),
        );

        app.add_systems(Update, update_z_system.run_if(in_state(Play)));
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
    reservables: Res<KDTree2<Reservable>>,
    reserved_tiles: Query<(Entity, &mut TilePos), (With<T>, With<Reserved>)>,
    mut agents_without_reservation: Query<
        (&mut Blackboard, &mut Transform, &mut Movement),
        (With<HasThinker>, Without<Reservation>),
    >,
    mut agents_with_reservation: Query<
        (&mut Blackboard, &mut Transform, &mut Movement, &Reservation),
        (With<HasThinker>,),
    >,
    mut action_query: Query<(&Actor, &mut ActionState, &mut MoveToNearest<T>, &ActionSpan)>,
    mut reservation_request_writer: EventWriter<ReservationRequest>,
) {
    for (actor, mut action_state, mut move_to, span) in &mut action_query {
        let _guard = span.span().enter();

        match *action_state {
            ActionState::Requested => {
                if let Ok((_, transform, _)) = agents_without_reservation.get_mut(actor.0) {
                    // Get k nearest reservable entities
                    let targets = reservables.k_nearest_neighbour(transform.translation.xy(), 10);

                    // Attempt to search the nearest 10 possible targets
                    for (target_position, target_entity) in targets.iter() {
                        let path_option = find_path(
                            &world,
                            transform.translation.xy().to_tilepos(),
                            target_position.xy().to_tilepos(),
                        );

                        if path_option.is_some() {
                            trace!(
                                "Found reachable {:?} (World Position {}, {}) - attempting a to create a reservation on {:?} for {:?}",
                                std::any::type_name::<T>(),
                                target_position.x,
                                target_position.y,
                                target_position,
                                actor.0
                            );

                            reservation_request_writer.send(
                                ReservationRequestBuilder::default()
                                    .requester(actor.0)
                                    .target(target_entity.unwrap())
                                    .build()
                                    .unwrap(),
                            );

                            // This shouldn't be here but whatever
                            move_to.goal = Some(target_position.xy());

                            *action_state = ActionState::Requested;
                            return;
                        }
                    }
                }

                if let Ok(agent) = agents_with_reservation.get_mut(actor.0) {
                    let mut blackboard = agent.0;
                    let agent_transform = agent.1;
                    let mut agent_movement = agent.2;
                    let reservation = agent.3;

                    let start_coord = agent_transform.translation.xy().to_tilepos();
                    let goal_tile = reserved_tiles.get(reservation.target);

                    if let Ok((goal_tile_entity, &goal_tile_position)) = goal_tile {
                        let path_option = find_path(&world, start_coord, goal_tile_position);

                        if let Some(mut path) = path_option {
                            // We don't want to include the first goal if it is the same as the start
                            if path.first() == Some(&start_coord) {
                                path.remove(0);
                            }

                            trace!("Set path to {:?}", std::any::type_name::<T>());
                            agent_movement.path = path;
                            *action_state = ActionState::Executing;
                            blackboard.insert("bush", json!(goal_tile_entity));
                        }
                    } else {
                        *action_state = ActionState::Failure;
                    }
                }
            }
            ActionState::Executing => {
                let (_, actor_transform, _actor_movement, _reservation) =
                    agents_with_reservation.get_mut(actor.0).unwrap();
                let delta = move_to.goal.unwrap() - actor_transform.translation.xy();
                let distance = delta.length();
                if distance > MAX_DISTANCE {
                    // Movement should be handled by the movement system
                } else {
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
    tilepos_q: Query<&TilePos>,
    mut remove_reservation_event_writer: EventWriter<RemoveReservation>,
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

                                let tilepos = *tilepos_q.get(entity_id).unwrap();
                                remove_reservation_event_writer.send(RemoveReservation { tilepos });
                            } else {
                                *action_state = ActionState::Failure;
                            }
                        }

                        blackboard.remove("bush");
                        commands.entity(actor.0).remove::<GatheringTag>();
                        commands.entity(actor.0).remove::<GatheringTimer>();
                        commands.entity(actor.0).remove::<Reservation>();
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
