use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    let texture_handle = assets.load("grass.png"); // TODO: This will eventually be a master tilemap containing all possible tiles
    let tilemap_size = TilemapSize { x: 16, y: 16 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(tilemap_size);

    let constraints = get_constraints();
    let mut grid = Grid::new(WIDTH, HEIGHT, constraints);
    grid.run();
    grid.display();

    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Tile {
    Grass,
    Water,
    Sand,
}

struct Constraint {
    tile: Tile,
    allowed_neighbors: Vec<Tile>,
}

impl Constraint {
    fn new(tile: Tile, allowed_neighbors: Vec<Tile>) -> Self {
        Constraint {
            tile,
            allowed_neighbors,
        }
    }
}

fn get_constraints() -> Vec<Constraint> {
    vec![
        Constraint::new(Tile::Grass, vec![Tile::Grass, Tile::Sand]),
        Constraint::new(Tile::Water, vec![Tile::Water, Tile::Sand]),
        Constraint::new(Tile::Sand, vec![Tile::Grass, Tile::Water, Tile::Sand]),
    ]
}

use std::collections::{HashMap, HashSet};

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

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
    constraints: HashMap<Tile, HashSet<Tile>>,
}

impl Grid {
    fn new(width: usize, height: usize, constraints: Vec<Constraint>) -> Self {
        let possible_tiles: Vec<Tile> = constraints.iter().map(|c| c.tile).collect();
        let mut constraint_map = HashMap::new();
        for constraint in constraints {
            constraint_map.insert(
                constraint.tile,
                constraint.allowed_neighbors.iter().cloned().collect(),
            );
        }
        Grid {
            cells: vec![vec![Cell::new(&possible_tiles); height]; width],
            constraints: constraint_map,
        }
    }
}

use rand::seq::IteratorRandom;
use rand::thread_rng;

impl Grid {
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
}

impl Grid {
    fn propagate(&mut self, x: usize, y: usize, tile: Tile) {
        let neighbors = vec![
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < WIDTH && ny < HEIGHT {
                let allowed_neighbors = &self.constraints[&tile];
                self.cells[nx][ny]
                    .possibilities
                    .retain(|&neighbor| allowed_neighbors.contains(&neighbor));
            }
        }
    }
}

impl Grid {
    fn run(&mut self) {
        while let Some((x, y, tile)) = self.random_collapse() {
            self.propagate(x, y, tile);
        }
    }

    fn display(&self) {
        for row in &self.cells {
            for cell in row {
                if cell.possibilities.len() == 1 {
                    print!("{:?} ", cell.possibilities.iter().next().unwrap());
                } else {
                    print!("? ");
                }
            }
            println!();
        }
    }
}
