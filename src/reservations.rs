use bevy::prelude::*;
use derive_builder::Builder;

pub struct ReservationsPlugin;

impl Plugin for ReservationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReservationRequest>()
            .add_systems(Update, reservation_system);
    }
}

#[derive(Builder, Event)]
pub struct ReservationRequest {
    requester: Entity,
    target: Entity,
}

#[derive(Component)]
pub struct Reservable;

#[derive(Builder, Component)]
pub struct Reservation {
    pub target: Entity,
}

#[derive(Builder, Component)]
pub struct Reserved {
    owner: Entity,
}

fn reservation_system(
    mut commands: Commands,
    mut reservation_requests: EventReader<ReservationRequest>,
    query: Query<Entity, (With<Reservable>, Without<Reserved>)>,
) {
    for reservation_request in reservation_requests.read() {
        if let Ok(target) = query.get(reservation_request.target) {
            commands.entity(reservation_request.requester).insert(
                ReservationBuilder::default()
                    .target(target)
                    .build()
                    .unwrap(),
            );

            commands.entity(target).insert(
                ReservedBuilder::default()
                    .owner(reservation_request.requester)
                    .build()
                    .unwrap(),
            );

            trace!(
                "{:?} has reserved {:?}",
                reservation_request.requester,
                target
            );
        } else {
            error!(
                "{:?} failed to reserve {:?}",
                reservation_request.requester, reservation_request.target
            );
        }
    }
}
