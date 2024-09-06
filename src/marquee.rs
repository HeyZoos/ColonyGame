use crate::reservations::Reservable;
use crate::states::States::Play;
use crate::ENTITY_SIZE_IN_PIXELS;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, Group, Sensor};
use bevy_rapier2d::pipeline::CollisionEvent;
use std::collections::HashSet;

pub const SELECTABLE_GROUP: Group = Group::GROUP_1;
pub const SELECTION_GROUP: Group = Group::GROUP_2;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_input_handler.run_if(in_state(Play)));
        app.add_systems(Update, draw_marquee_selection.run_if(in_state(Play)));
        app.add_systems(Update, mouse_motion_handler.run_if(in_state(Play)));
        app.add_systems(Update, handle_collision_events.run_if(in_state(Play)));
    }
}

#[derive(Component)]
struct MarqueeSelection {
    start: Vec2,
    end: Vec2,
    selected: HashSet<Entity>,
}

impl MarqueeSelection {
    fn display_gizmos(&self, gizmos: &mut Gizmos) {
        gizmos.circle_2d(self.start, 0.125, Color::YELLOW);
        gizmos.circle_2d(self.end, 0.125, Color::YELLOW);
    }
}

fn mouse_input_handler(
    mut commands: Commands,
    mut events: EventReader<MouseButtonInput>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_marquee: Query<Entity, With<MarqueeSelection>>,
) {
    for event in events.read() {
        let (camera, camera_transform) = q_camera.single();
        let window = q_window.single();
        if let Some(cursor_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            match event.state {
                ButtonState::Pressed => {
                    let marquee = spawn_cube(&mut commands, cursor_position.extend(0.0));
                    commands
                        .entity(marquee)
                        .insert(MarqueeSelection {
                            start: cursor_position,
                            end: cursor_position,
                            selected: HashSet::default(),
                        })
                        .insert(CollisionGroups::new(SELECTION_GROUP, SELECTABLE_GROUP));
                }
                ButtonState::Released => {
                    if let Ok(marquee_entity) = q_marquee.get_single() {
                        commands.entity(marquee_entity).despawn();
                    }
                }
            }
        }
    }
}

fn draw_marquee_selection(mut gizmos: Gizmos, q_marquee: Query<&MarqueeSelection>) {
    if let Ok(marquee) = q_marquee.get_single() {
        marquee.display_gizmos(&mut gizmos);
    }
}

fn spawn_cube(commands: &mut Commands, translation: Vec3) -> Entity {
    commands
        .spawn(Collider::cuboid(
            ENTITY_SIZE_IN_PIXELS / 2.0,
            ENTITY_SIZE_IN_PIXELS / 2.0,
        ))
        .insert(TransformBundle::from(Transform {
            translation,
            ..default()
        }))
        .insert(ActiveCollisionTypes::STATIC_STATIC)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Sensor)
        .insert(CollisionGroups::new(SELECTABLE_GROUP, SELECTION_GROUP))
        .id()
}

fn mouse_motion_handler(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut events: EventReader<CursorMoved>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_marquee: Query<(Entity, &mut MarqueeSelection)>,
) {
    if let Ok((entity, mut marquee)) = q_marquee.get_single_mut() {
        marquee.display_gizmos(&mut gizmos);
        for event in events.read() {
            let (camera, camera_transform) = q_camera.single();
            if let Some(cursor_position) = camera.viewport_to_world_2d(camera_transform, event.position) {
                marquee.end = cursor_position;

                let half_extents = (marquee.start - marquee.end).abs() / 2.0;
                let midpoint = (marquee.start + marquee.end) / 2.0;

                commands
                    .entity(entity)
                    .try_insert(Collider::cuboid(half_extents.x, half_extents.y))
                    .try_insert(Transform::from_xyz(midpoint.x, midpoint.y, 0.0));
            }
        }
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut q_marquee: Query<(Entity, &mut MarqueeSelection)>,
) {
    if let Ok((marquee_entity, mut marquee)) = q_marquee.get_single_mut() {
        for event in events.read() {
            match event {
                CollisionEvent::Started(e1, e2, _flags) => {
                    if *e1 == marquee_entity {
                        marquee.selected.insert(*e2);
                        // TODO(jesse): Refactor this to make this event driven with selectable events
                        commands.entity(*e2).insert(Reservable);
                    } else if *e2 == marquee_entity {
                        marquee.selected.insert(*e1);
                        // TODO(jesse): Refactor this to make this event driven with selectable events
                        commands.entity(*e1).insert(Reservable);
                    }
                }
                CollisionEvent::Stopped(e1, e2, _flags) => {
                    if *e1 == marquee_entity {
                        marquee.selected.remove(e2);
                    } else if *e2 == marquee_entity {
                        marquee.selected.remove(e1);
                    }
                }
            }

            dbg!(&marquee.selected);
        }
    }
}
