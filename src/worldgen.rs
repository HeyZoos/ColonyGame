use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use grid_2d::{Grid, Size};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use wfc::overlapping::OverlappingPatterns;
use wfc::Wave;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const TILE_SIZE: f32 = 16.0;

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let texture_handle = assets.load("grass.png");

    let tilemap_size = TilemapSize {
        x: WIDTH,
        y: HEIGHT,
    };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(tilemap_size);

    let pattern = vec![
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 1, 1, 1, 1, 2, 1, 1, 1, 2, 
        2, 1, 0, 0, 1, 2, 1, 0, 1, 2,
        2, 1, 0, 0, 1, 2, 1, 1, 1, 2, 
        2, 1, 1, 1, 1, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2
    ];

    let wave = wfc(patterns(pattern.clone()), 3);
    populate_tilemap(&mut commands, &mut tile_storage, &wave, tilemap_entity, patterns(pattern.clone()));
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tilemap_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&tilemap_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

// Populate Tilemap Function
fn populate_tilemap(
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
    wave: &Wave,
    tilemap_entity: Entity,
    patterns: OverlappingPatterns<u8>
) {
    for coordinate in wave.grid().coord_iter() {
        let cell =  wave.grid().get(coordinate).unwrap();
        let id = cell.chosen_pattern_id().unwrap();
        let value = patterns.pattern_top_left_value(id);
        let tile_pos = TilePos { x: coordinate.x as u32, y: coordinate.y as u32 };
        
        let tile = commands
            .spawn(TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(*value as u32),
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            })
            .id();
        
        tile_storage.set(&tile_pos, tile);
    }
}

// Vec<u8> -> OverlappingPatterns<u8>
fn patterns(pattern: Vec<u8>) -> OverlappingPatterns<u8> {
    let sqrt = (pattern.len() as f64).sqrt() as u32;
    let grid = Grid::new_iterator(Size::new(sqrt, sqrt), pattern.into_iter());
    OverlappingPatterns::new(
        grid,
        std::num::NonZeroU32::new(3).unwrap(),
        &[wfc::Orientation::Original]
    )
}

// OverlappingPatterns<u8>, u64 -> Wave
fn wfc(patterns: OverlappingPatterns<u8>, seed: u64) -> Wave {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let global_stats = patterns.global_stats();

    let runner = wfc::RunOwn::new_wrap_forbid(
        Size::new(WIDTH, HEIGHT),
        &global_stats,
        wfc::wrap::WrapXY,
        wfc::ForbidNothing,
        &mut rng,
    );

    runner.collapse_retrying(
        wfc::retry::NumTimes(20),
        &mut rng,
    ).unwrap()
}
