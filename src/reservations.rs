use bevy::prelude::*;
use derive_builder::Builder;

pub struct ReservationsPlugin;

impl Plugin for ReservationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Reservation>()
            .add_systems(Update, reservation_system);
    }
}

#[derive(Builder, Event)]
pub struct Reservation {
    requester: Entity,
    target: Entity,
}

#[derive(Component)]
pub struct Reservable;

#[derive(Builder, Component)]
pub struct Reserved {
    owner: Entity,
}

fn reservation_system(
    mut commands: Commands,
    mut reservations: EventReader<Reservation>,
    query: Query<Entity, (With<Reservable>, Without<Reserved>)>,
) {
    for reservation in reservations.read() {
        if let Ok(target) = query.get(reservation.target) {
            trace!("{:?} has reserved {:?}", reservation.requester, target);
            commands.entity(target).insert(Reserved {
                owner: reservation.requester,
            });
        } else {
            error!(
                "{:?} failed to reserve {:?}",
                reservation.requester,
                reservation.target
            );
        }
    }
}
