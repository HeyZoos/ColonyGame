use crate::ext::Vec2Ext;
use crate::villager::{find_path, Movement};
use bevy::prelude::*;
use big_brain::prelude::*;
use std::fmt::Debug;
use std::marker::PhantomData;

const MAX_DISTANCE: f32 = 0.1;

#[derive(Clone, Component, Debug)]
pub struct Bush;

pub struct AIPlugin;

impl Plugin for AIPlugin {
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
    mut query: Query<&mut Transform, With<T>>,
    mut thinkers: Query<&mut Transform, (With<HasThinker>, Without<T>)>,
    mut action_query: Query<(
        &Actor,
        &mut ActionState,
        &mut MoveToNearest<T>,
        &ActionSpan,
        &mut Movement,
    )>,
) {
    for (actor, mut action_state, mut move_to, span, mut movement) in &mut action_query {
        let _guard = span.span().enter();
        println!("hello?");

        match *action_state {
            ActionState::Requested => {
                debug!("Let's go find a {:?}", std::any::type_name::<T>());
                let actor_transform = thinkers.get_mut(actor.0).unwrap();
                let goal_transform = query
                    .iter_mut()
                    .map(|t| (t.translation, t))
                    .min_by(|(a, _), (b, _)| {
                        let delta_a = *a - actor_transform.translation;
                        let delta_b = *b - actor_transform.translation;
                        delta_a.length().partial_cmp(&delta_b.length()).unwrap()
                    })
                    .unwrap()
                    .1;

                move_to.goal = Some(goal_transform.translation.xy());

                movement.path = find_path(
                    &world,
                    actor_transform.translation.xy().to_grid_space().to_coord(),
                    move_to.goal.unwrap().to_grid_space().to_coord(),
                )
                .unwrap()
                .0;

                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                let actor_transform = thinkers.get_mut(actor.0).unwrap();
                let delta = move_to.goal.unwrap() - actor_transform.translation.xy();
                let distance = delta.length();

                trace!("Distance: {}", distance);

                if distance > MAX_DISTANCE {
                    trace!("Stepping closer.");
                    // Movement should be handled by the movement system
                } else {
                    debug!("We got there!");
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
