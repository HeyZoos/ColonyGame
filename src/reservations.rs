use bevy::prelude::*;
use bevy_spatial::*;
use derive_builder::Builder;

pub struct ReservationsPlugin;

impl Plugin for ReservationsPlugin {
    fn build(&self, app: &mut App) {
        info!("ReservationsPlugin#build");
        app.add_event::<ReservationRequest>()
            .add_systems(Update, reservation_system)
            // This will create a `KDTree2<Reservable>` resource which can be used for querying
            .add_plugins(
                AutomaticUpdate::<Reservable>::new().with_spatial_ds(SpatialStructure::KDTree2),
            );
    }
}

#[derive(Builder, Event)]
pub struct ReservationRequest {
    requester: Entity,
    target: Entity,
}

#[derive(Component, Default)]
pub struct Reservable;

#[derive(Builder, Component)]
pub struct Reservation {
    pub target: Entity,
}

#[derive(Component)]
pub struct Reserved;

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

            commands.entity(target).insert(Reserved);

            // This will remove the entity from the `KDTree2<Reservable>` resource
            commands.entity(target).remove::<Reservable>();

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
