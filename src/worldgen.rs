use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use grid_2d::{Grid, Size};
use iyes_progress::{dummy_system_wait_frames, dummy_system_wait_millis, Progress, ProgressSystem};
use noise::{NoiseFn, Perlin};
use rand::prelude::SliceRandom;
use rand::{thread_rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use wfc::overlapping::OverlappingPatterns;
use wfc::Wave;

use crate::agent::Bush;
use crate::reservations::Reservable;
use crate::states::States::{LoadPlay, Play};

pub const TILEMAP_SIZE: TilemapSize = TilemapSize::new(256, 256);
pub const TILEMAP_TILE_SIZE: TilemapTileSize = TilemapTileSize::new(16.0, 16.0);
pub const TILEMAP_TYPE: TilemapType = TilemapType::Square;

// master.png
const GRASS_TILE_ID: u16 = 17;

// mushrooms-flowers-stones.png
const BUSH_TILE_ID: u32 = 27;
const FLOWER_TILE_ID: u32 = 50;
const STONE_TILE_ID: u32 = 17;

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin).add_systems(
            Update,
            (
                generate_layer.track_progress(),
            )
                .run_if(in_state(LoadPlay)),
        );
        app.add_systems(Update, update_tile_transform_system.run_if(in_state(Play)));
    }
}

#[derive(Resource)]
pub struct World {
    pub wave: Wave,
    pub patterns: OverlappingPatterns<u16>,
}

fn generate_layer(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut next_layer_id: Local<u32>,
) -> Progress {
    // Load hand-crafted pattern made in the Tiled editor
    let mut tiled_loader = tiled::Loader::new();

    // Note that this tilemap needs to be a square
    let tiled_map = tiled_loader.load_tmx_map("assets/patterns.tmx").unwrap();

    if *next_layer_id < tiled_map.layers().len() as u32 {
        // For each tilemap layer
        let layer = tiled_map.layers().nth(*next_layer_id as usize).unwrap();

        // Convert the layer to a tile layer
        let tile_layer = layer.as_tile_layer().unwrap();

        // Each layer should only reference the master tileset
        let mut tileset = tiled_map.tilesets()[0].as_ref();

        // Convert the tile layer to a Vec<u16> which for wave function collapse
        let mut pattern = vec![];
        for y in (0..tile_layer.height().unwrap()).rev() {
            for x in 0..tile_layer.width().unwrap() {
                if let Some(tile) = tile_layer.get_tile(x as i32, y as i32) {
                    pattern.push(tile.id() as u16);
                    tileset = tile.get_tileset();
                } else {
                    pattern.push(255);
                }
            }
        }

        // Run wave function collapse
        let wave = wfc(patterns(pattern.clone()), 3);

        // Get the tileset asset
        let tileset_image = tileset.image.as_ref().expect("Image not found");
        let mut tileset_image_path = tileset_image.source.clone();
        tileset_image_path = PathBuf::from(tileset_image_path.strip_prefix("assets").unwrap());
        let texture_handle = assets.load(tileset_image_path);

        // Build tilemap based on WFC results
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(TILEMAP_SIZE);
        let children = populate_tilemap(
            &mut commands,
            &mut tile_storage,
            &wave,
            tilemap_entity,
            patterns(pattern.clone()),
        );

        // Store the wave as a resource for use in pathfinding and post-processing
        // Only store the wave generated by the land / grass layer
        if layer.name == "grass" {
            commands.insert_resource(World {
                wave,
                patterns: patterns(pattern.clone()),
            });
        }

        let grid_size = TILEMAP_TILE_SIZE.into();
        let map_type = TilemapType::default();
        commands
            .entity(tilemap_entity)
            .insert((
                Name::new("Tilemap"),
                TilemapBundle {
                    grid_size,
                    map_type,
                    size: TILEMAP_SIZE,
                    storage: tile_storage,
                    texture: TilemapTexture::Single(texture_handle),
                    tile_size: TILEMAP_TILE_SIZE,
                    transform: Transform::from_xyz(0.0, 0.0, *next_layer_id as f32),
                    ..Default::default()
                },
            ))
            .push_children(&children);

        *next_layer_id += 1;
    }

    let result = tiled_map.layers().len() as u32;
    dbg!((*next_layer_id == result).into())
}

// Populate Tilemap Function
fn populate_tilemap(
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
    wave: &Wave,
    tilemap_entity: Entity,
    patterns: OverlappingPatterns<u16>,
) -> Vec<Entity> {
    let mut children = vec![];

    for coordinate in wave.grid().coord_iter() {
        let cell = wave.grid().get(coordinate).unwrap();
        let id = cell.chosen_pattern_id().unwrap();
        let value = variants(*patterns.pattern_top_left_value(id));
        let tile_pos = TilePos {
            x: coordinate.x as u32,
            y: coordinate.y as u32,
        };

        let mut tile = commands.spawn((
            Name::new("Tile"),
            TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(value as u32),
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            },
        ));

        if value == 128 {
            tile.insert(AnimatedTile {
                start: 128,
                end: 131,
                speed: 0.5,
            });
        }

        tile_storage.set(&tile_pos, tile.id());

        // Collect children entities so that they can be organized under their tilemap
        children.push(tile.id());
    }

    children
}

// u16 -> u16
fn variants(tilemap_idx: u16) -> u16 {
    let mut rng = thread_rng();
    let grass_variants = generate_grass_variants();
    match tilemap_idx {
        GRASS_TILE_ID => *grass_variants.choose(&mut rng).unwrap(),
        _ => tilemap_idx,
    }
}

// () -> Vec<u16>
fn generate_grass_variants() -> Vec<u16> {
    let mut variants = Vec::new();
    variants.extend(std::iter::repeat(GRASS_TILE_ID).take(70)); // Weight the plain grass tile more heavily
    variants.extend(&[85, 96, 97, 98, 99, 100, 101]);
    variants
}

// Vec<u16> -> OverlappingPatterns<u16>
fn patterns(pattern: Vec<u16>) -> OverlappingPatterns<u16> {
    let sqrt = (pattern.len() as f64).sqrt() as u32;
    let grid = Grid::new_iterator(Size::new(sqrt, sqrt), pattern.into_iter());
    OverlappingPatterns::new(
        grid,
        std::num::NonZeroU32::new(2).unwrap(),
        &[wfc::Orientation::Original],
    )
}

// OverlappingPatterns<u16>, u64 -> Wave
fn wfc(patterns: OverlappingPatterns<u16>, seed: u64) -> Wave {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let global_stats = patterns.global_stats();

    let runner = wfc::RunOwn::new_wrap_forbid(
        Size::new(TILEMAP_SIZE.x, TILEMAP_SIZE.y),
        &global_stats,
        wfc::wrap::WrapNone,
        wfc::ForbidNothing,
        &mut rng,
    );

    runner
        .collapse_retrying(wfc::retry::NumTimes(20), &mut rng)
        .unwrap()
}

fn resource_layer_startup_system(
    mut commands: Commands,
    world: Res<World>,
    assets: Res<AssetServer>,
) {
    let perlin = Perlin::new(3);

    // Define noise scale for resource placement
    let noise_scale = 0.1;

    // Get the tileset asset for resources
    let resource_tileset_path = "mushrooms-flowers-stones.png";
    let resource_texture_handle = assets.load(resource_tileset_path);

    // Create a new tilemap for resources
    let resource_tilemap_entity = commands.spawn_empty().id();
    let mut resource_tile_storage = TileStorage::empty(TILEMAP_SIZE);

    // Define resource types and their corresponding noise thresholds
    // Put the higher priority items higher
    let resource_types = [
        (STONE_TILE_ID, 0.7),
        (FLOWER_TILE_ID, 0.5),
        (BUSH_TILE_ID, 0.3),
    ];

    // Populate the resource tilemap
    for coord in world.wave.grid().coord_iter() {
        let x = coord.x;
        let y = coord.y;
        let noise_value = perlin.get([x as f64 * noise_scale, y as f64 * noise_scale]);

        let tile_pos = TilePos {
            x: x as u32,
            y: y as u32,
        };
        let wave_tile = world.wave.grid().get(coord).unwrap();
        let pattern_id = wave_tile.chosen_pattern_id().unwrap();
        let value = world.patterns.pattern_top_left_value(pattern_id);

        // Check if the current tile is grass
        if *value == GRASS_TILE_ID {
            for (resource_id, threshold) in resource_types.iter() {
                if noise_value > *threshold {
                    let mut resource_tile = commands.spawn(TileBundle {
                        position: tile_pos,
                        texture_index: TileTextureIndex(*resource_id),
                        tilemap_id: TilemapId(resource_tilemap_entity),
                        ..Default::default()
                    });
                    resource_tile_storage.set(&tile_pos, resource_tile.id());

                    if *resource_id == BUSH_TILE_ID {
                        resource_tile.insert(Name::new("Bush"));
                        resource_tile.insert(Bush);
                        resource_tile.insert(Reservable);
                        resource_tile.insert(Transform::default());
                    }

                    break; // Stop after placing the first valid resource
                }
            }
        }
    }

    let resource_grid_size = TILEMAP_TILE_SIZE.into();
    let resource_map_type = TilemapType::default();
    commands
        .entity(resource_tilemap_entity)
        .insert(TilemapBundle {
            grid_size: resource_grid_size,
            map_type: resource_map_type,
            size: TILEMAP_SIZE,
            storage: resource_tile_storage,
            texture: TilemapTexture::Single(resource_texture_handle),
            tile_size: TILEMAP_TILE_SIZE,
            transform: Transform::from_xyz(0.0, 0.0, 5.0),
            ..Default::default()
        });
}

/// Maintain the `Transform` component on tiles so that they can be used in spatial queries
fn update_tile_transform_system(mut q: Query<(&mut Transform, &TilePos)>) {
    for (mut transform, tilepos) in q.iter_mut() {
        transform.translation = tilepos
            .center_in_world(&TILEMAP_TILE_SIZE.into(), &TILEMAP_TYPE)
            .extend(0.0);
    }
}
