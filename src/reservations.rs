use crate::assets::UiAssets;
use crate::states::States::Play;
use crate::worldgen::{TILEMAP_SIZE, TILEMAP_TILE_SIZE, TILEMAP_TYPE};
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapId, TilemapTexture};
use bevy_ecs_tilemap::prelude::{TileBundle, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;
use bevy_spatial::*;
use derive_builder::Builder;

pub struct ReservationsPlugin;

impl Plugin for ReservationsPlugin {
    fn build(&self, app: &mut App) {
        info!("ReservationsPlugin#build");
        app.add_event::<ReservationRequest>()
            .add_event::<RemoveReservation>()
            .add_systems(Update, reservation_system)
            // This will create a `KDTree2<Reservable>` resource which can be used for querying
            .add_plugins(AutomaticUpdate::<Reservable>::new().with_spatial_ds(SpatialStructure::KDTree2));

        app.add_systems(Update, on_reserved_removed.run_if(in_state(Play)));

        app.add_systems(Update, on_reservable_added.run_if(in_state(Play)));

        app.add_systems(OnEnter(Play), mark_a_bush_as_reserved);
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

/// Tag component for the reservation tilemap used to visualize reservations
#[derive(Component)]
pub struct ReservationTilemap;

/// Event to publish which tile position is unreserved
#[derive(Builder, Event)]
pub struct RemoveReservation {
    pub tilepos: TilePos,
}

fn reservation_system(
    mut commands: Commands,
    mut reservation_requests: EventReader<ReservationRequest>,
    query: Query<Entity, (With<Reservable>, Without<Reserved>)>,
) {
    for reservation_request in reservation_requests.read() {
        if let Ok(target) = query.get(reservation_request.target) {
            commands
                .entity(reservation_request.requester)
                .insert(ReservationBuilder::default().target(target).build().unwrap());

            commands.entity(target).insert(Reserved);

            // This will remove the entity from the `KDTree2<Reservable>` resource
            commands.entity(target).remove::<Reservable>();

            trace!("{:?} has reserved {:?}", reservation_request.requester, target);
        } else {
            error!(
                "{:?} failed to reserve {:?}",
                reservation_request.requester, reservation_request.target
            );
        }
    }
}

fn mark_a_bush_as_reserved(
    mut commands: Commands,
    mut tilemaps: Query<(&Name, &mut TileStorage)>,
    ui_assets: Res<UiAssets>,
) {
    // Create a tilemap to hold reservations
    commands.spawn((
        ReservationTilemap,
        Name::new("Reservations"),
        TilemapBundle {
            grid_size: TILEMAP_TILE_SIZE.into(),
            map_type: TILEMAP_TYPE,
            size: TILEMAP_SIZE,
            storage: TileStorage::empty(TILEMAP_SIZE),
            texture: TilemapTexture::Single(ui_assets.xs_image.clone()),
            tile_size: TILEMAP_TILE_SIZE,
            transform: Transform::from_xyz(0.0, 0.0, 6.0),
            ..default()
        },
    ));
}

fn on_reservable_added(
    mut commands: Commands,
    changed: Query<(Entity, &TilePos), Added<Reservable>>,
    mut tilemaps: Query<(Entity, &mut TileStorage), With<ReservationTilemap>>,
) {
    let (tilemap_entity, mut tile_storage) = tilemaps.single_mut();

    // Add xs for all the newly reserved tiles
    for (_resource_layer_entity, tilepos) in changed.iter() {
        let reservation_layer_entity = commands
            .spawn(TileBundle {
                position: *tilepos,
                texture_index: TileTextureIndex(11),
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            })
            .id();

        tile_storage.set(tilepos, reservation_layer_entity);
    }
}

fn on_reserved_removed(
    mut cmds: Commands,
    mut removed: EventReader<RemoveReservation>,
    mut reservation_tilemap_q: Query<&mut TileStorage, With<ReservationTilemap>>,
) {
    let reservation_tilemap = reservation_tilemap_q.single_mut();
    for RemoveReservation { tilepos } in removed.read() {
        if let Some(tile) = reservation_tilemap.get(tilepos) {
            cmds.entity(tile).despawn();
        }
    }
}
