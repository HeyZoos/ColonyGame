use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};

// Constants
const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const TILE_SIZE: f32 = 16.0;

// Plugin Definition
pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);
        app.add_systems(Startup, startup);
    }
}

// Startup System
fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let texture_handle = assets.load("grass.png"); // TODO: This will eventually be a master tilemap containing all possible tiles

    let tilemap_size = TilemapSize {
        x: WIDTH as u32,
        y: HEIGHT as u32,
    };

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(tilemap_size);

    let constraints = get_constraints();
    let mut grid = Grid::new(WIDTH, HEIGHT, constraints);
    grid.run();

    populate_tilemap(&mut commands, &mut tile_storage, &grid, tilemap_entity);

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
    grid: &Grid,
    tilemap_entity: Entity,
) {
    let tile_id_mapping = |tile: Tile| match tile {
        Tile::Grass => 0,
        Tile::Water => 1,
        Tile::Sand => 2,
    };

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let tile_pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            let tile = grid.cells[x][y].possibilities.iter().next().unwrap();
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex(tile_id_mapping(*tile)),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }
}

// Direction Enum
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

// Tile and Constraint Definitions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tile {
    Grass,
    Water,
    Sand,
}

struct Constraint {
    tile: Tile,
    allowed_neighbors: HashMap<Direction, Vec<Tile>>,
}

impl Constraint {
    fn new(tile: Tile, allowed_neighbors: HashMap<Direction, Vec<Tile>>) -> Self {
        Constraint {
            tile,
            allowed_neighbors,
        }
    }
}

fn get_constraints() -> Vec<Constraint> {
    let mut grass_neighbors = HashMap::new();
    grass_neighbors.insert(Direction::North, vec![Tile::Grass, Tile::Sand]);
    grass_neighbors.insert(Direction::South, vec![Tile::Grass, Tile::Sand]);
    grass_neighbors.insert(Direction::East, vec![Tile::Grass, Tile::Sand]);
    grass_neighbors.insert(Direction::West, vec![Tile::Grass, Tile::Sand]);

    let mut water_neighbors = HashMap::new();
    water_neighbors.insert(Direction::North, vec![Tile::Water, Tile::Sand]);
    water_neighbors.insert(Direction::South, vec![Tile::Water, Tile::Sand]);
    water_neighbors.insert(Direction::East, vec![Tile::Water, Tile::Sand]);
    water_neighbors.insert(Direction::West, vec![Tile::Water, Tile::Sand]);

    let mut sand_neighbors = HashMap::new();
    sand_neighbors.insert(Direction::North, vec![Tile::Grass, Tile::Water, Tile::Sand]);
    sand_neighbors.insert(Direction::South, vec![Tile::Grass, Tile::Water, Tile::Sand]);
    sand_neighbors.insert(Direction::East, vec![Tile::Grass, Tile::Water, Tile::Sand]);
    sand_neighbors.insert(Direction::West, vec![Tile::Grass, Tile::Water, Tile::Sand]);

    vec![
        Constraint::new(Tile::Grass, grass_neighbors),
        Constraint::new(Tile::Water, water_neighbors),
        Constraint::new(Tile::Sand, sand_neighbors),
    ]
}

// Cell and Grid Definitions
#[derive(Clone, Debug)]
struct Cell {
    possibilities: HashSet<Tile>,
}

impl Cell {
    fn new(possible_tiles: &Vec<Tile>) -> Self {
        Cell {
            possibilities: possible_tiles.iter().cloned().collect(),
        }
    }

    fn collapse(&mut self, tile: Tile) {
        self.possibilities.clear();
        self.possibilities.insert(tile);
    }
}

struct Grid {
    cells: Vec<Vec<Cell>>,
    constraints: HashMap<Tile, Constraint>,
}

impl Grid {
    fn new(width: usize, height: usize, constraints: Vec<Constraint>) -> Self {
        let possible_tiles: Vec<Tile> = constraints.iter().map(|c| c.tile).collect();
        let mut constraint_map = HashMap::new();
        for constraint in constraints {
            constraint_map.insert(constraint.tile, constraint);
        }
        Grid {
            cells: vec![vec![Cell::new(&possible_tiles); height]; width],
            constraints: constraint_map,
        }
    }

    fn random_collapse(&mut self) -> Option<(usize, usize, Tile)> {
        let mut rng = thread_rng();
        let mut min_possibilities = usize::MAX;
        let mut candidates = vec![];

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let possibilities = self.cells[x][y].possibilities.len();
                if possibilities > 1 && possibilities < min_possibilities {
                    min_possibilities = possibilities;
                    candidates.clear();
                    candidates.push((x, y));
                } else if possibilities == min_possibilities {
                    candidates.push((x, y));
                }
            }
        }

        if let Some(&(x, y)) = candidates.iter().choose(&mut rng) {
            if let Some(&tile) = self.cells[x][y].possibilities.iter().choose(&mut rng) {
                self.cells[x][y].collapse(tile);
                return Some((x, y, tile));
            }
        }
        None
    }

    fn propagate(&mut self, x: usize, y: usize, tile: Tile) {
        let directions = vec![
            (x.wrapping_sub(1), y, Direction::West),
            (x + 1, y, Direction::East),
            (x, y.wrapping_sub(1), Direction::South),
            (x, y + 1, Direction::North),
        ];

        for (nx, ny, direction) in directions {
            if nx < WIDTH && ny < HEIGHT {
                if let Some(allowed_neighbors) =
                    self.constraints[&tile].allowed_neighbors.get(&direction)
                {
                    self.cells[nx][ny]
                        .possibilities
                        .retain(|&neighbor| allowed_neighbors.contains(&neighbor));
                }
            }
        }
    }

    fn run(&mut self) {
        while let Some((x, y, tile)) = self.random_collapse() {
            self.propagate(x, y, tile);
        }
    }
}
