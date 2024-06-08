use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use grid_2d::{Grid, Size};
use rand::prelude::SliceRandom;
use rand::{SeedableRng, thread_rng};
use rand_chacha::ChaCha8Rng;
use wfc::overlapping::OverlappingPatterns;
use wfc::Wave;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const TILE_SIZE: f32 = 16.0;

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);
        app.add_systems(Startup, startup);
    }
}

#[derive(Resource)]
pub struct World {
    pub wave: Wave
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    // Load hand-crafted pattern made in the Tiled editor
    let mut tiled_loader = tiled::Loader::new();

    // Note that this tilemap needs to be a square
    let tiled_map = tiled_loader.load_tmx_map("assets/patterns.tmx").unwrap();

    // For each tilemap layer
    for layer_idx in (0..tiled_map.layers().len()).rev() {
        let layer = tiled_map.layers().nth(layer_idx).unwrap();

        // Convert the layer to a tile layer
        let tile_layer = layer.as_tile_layer().unwrap();

        // Each layer should only reference the master tileset
        let mut tileset = tiled_map.tilesets()[0].as_ref();

        // Convert the tile layer to a Vec<u16> which for wave function collapse
        let mut pattern = vec![];
        for y in (0..tile_layer.height().unwrap()).rev()  {
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
        let tilemap_size = TilemapSize { x: WIDTH, y: HEIGHT, };
        let tilemap_entity = commands.spawn_empty().id();
        let mut tile_storage = TileStorage::empty(tilemap_size);
        populate_tilemap(&mut commands, &mut tile_storage, &wave, tilemap_entity, patterns(pattern.clone()));

        // Store the wave as a resource for use in pathfinding and post-processing
        commands.insert_resource(World { wave });

        let tile_size = TilemapTileSize { x: TILE_SIZE, y: TILE_SIZE, };
        let grid_size = tile_size.into();
        let map_type = TilemapType::default();
        commands.entity(tilemap_entity).insert(TilemapBundle {
            grid_size,
            map_type,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&tilemap_size, &grid_size, &map_type, layer_idx as f32),
            ..Default::default()
        });
    }
}

// Populate Tilemap Function
fn populate_tilemap(
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
    wave: &Wave,
    tilemap_entity: Entity,
    patterns: OverlappingPatterns<u16>
) {
    for coordinate in wave.grid().coord_iter() {
        let cell =  wave.grid().get(coordinate).unwrap();
        let id = cell.chosen_pattern_id().unwrap();
        let value = variants(*patterns.pattern_top_left_value(id));
        let tile_pos = TilePos { x: coordinate.x as u32, y: coordinate.y as u32 };
        
        let mut tile = commands
            .spawn(TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(value as u32),
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            });
        
        if value == 128 {
            tile.insert(AnimatedTile {
                start: 128,
                end: 131,
                speed: 0.5
            });
        }
        
        tile_storage.set(&tile_pos, tile.id());
    }
}

// u16 -> u16
fn variants(tilemap_idx: u16) -> u16 {
    let mut rng = thread_rng();
    // If a value is grass, randomly choose one of the variants
    match tilemap_idx {
        17 => *[
            17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
            17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
            17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, // Weight the plain grass tile more heavily
            85, 96, 97, 98, 99, 100, 101].choose(&mut rng).unwrap() as u16,
        _ => tilemap_idx
    }
}

// Vec<u16> -> OverlappingPatterns<u16>
fn patterns(pattern: Vec<u16>) -> OverlappingPatterns<u16> {
    let sqrt = (pattern.len() as f64).sqrt() as u32;
    let grid = Grid::new_iterator(Size::new(sqrt, sqrt), pattern.into_iter());
    OverlappingPatterns::new(
        grid,
        std::num::NonZeroU32::new(2).unwrap(),
        &[wfc::Orientation::Original]
    )
}

// OverlappingPatterns<u16>, u64 -> Wave
fn wfc(patterns: OverlappingPatterns<u16>, seed: u64) -> Wave {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let global_stats = patterns.global_stats();

    let runner = wfc::RunOwn::new_wrap_forbid(
        Size::new(WIDTH, HEIGHT),
        &global_stats,
        wfc::wrap::WrapNone,
        wfc::ForbidNothing,
        &mut rng,
    );

    runner.collapse_retrying(
        wfc::retry::NumTimes(20),
        &mut rng,
    ).unwrap()
}
