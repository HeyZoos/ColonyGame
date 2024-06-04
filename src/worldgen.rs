use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{seq::IteratorRandom, SeedableRng};
use std::collections::{self, HashMap, HashSet};

// Constants
const WIDTH: usize = 4;
const HEIGHT: usize = 4;
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

    // Just keep trying if it doesn't collapse
    while !grid.run() {
        grid = Grid::new(WIDTH, HEIGHT, get_constraints());
    }

    // Display the grid for debugging
    println!("Final result :)");
    grid.display();

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
        Tile::GrassCornerTopLeft => 0,
        Tile::GrassSideTop => 1,
        Tile::GrassCornerTopRight => 2,
        
        Tile::GrassSideLeft => 11,
        Tile::Grass => 12,
        Tile::GrassSideRight => 13,

        Tile::GrassCornerBottomLeft => 22,
        Tile::GrassSideBottom => 23,
        Tile::GrassCornerBottomRight => 24,

        Tile::Empty => 44,

        // Tile::Water => 40,
        // Tile::Sand => 40,
    };

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let tile_pos = TilePos {
                x: x as u32,
                y: y as u32,
            };
            if let Some(tile) = grid.cells[x][y].possibilities.iter().next() {
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
    GrassSideTop,
    GrassSideBottom,
    GrassSideLeft,
    GrassSideRight,
    GrassCornerTopLeft,
    GrassCornerTopRight,
    GrassCornerBottomLeft,
    GrassCornerBottomRight,
    Empty,
    // Sand,
    // Water
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
    let mut empty_neighbors = HashMap::new();
    empty_neighbors.insert(Direction::North, vec![Tile::Empty]);
    empty_neighbors.insert(Direction::South, vec![Tile::Empty]);
    empty_neighbors.insert(Direction::East, vec![Tile::Empty]);
    empty_neighbors.insert(Direction::West, vec![Tile::Empty]);

    let mut grass_neighbors = HashMap::new();
    grass_neighbors.insert(Direction::North, vec![Tile::Grass, Tile::GrassSideTop]);
    grass_neighbors.insert(Direction::South, vec![Tile::Grass, Tile::GrassSideBottom]);
    grass_neighbors.insert(Direction::East, vec![Tile::Grass, Tile::GrassSideRight]);
    grass_neighbors.insert(Direction::West, vec![Tile::Grass, Tile::GrassSideLeft]);

    let mut grass_side_top_neighbors = HashMap::new();
    grass_side_top_neighbors.insert(Direction::North, vec![Tile::Empty]);
    grass_side_top_neighbors.insert(Direction::South, vec![Tile::Grass]);
    grass_side_top_neighbors.insert(Direction::East, vec![Tile::GrassSideTop, Tile::GrassCornerTopRight]);
    grass_side_top_neighbors.insert(Direction::West, vec![Tile::GrassSideTop, Tile::GrassCornerTopLeft]);

    let mut grass_side_bottom_neighbors = HashMap::new();
    grass_side_bottom_neighbors.insert(Direction::North, vec![Tile::Grass]);
    grass_side_bottom_neighbors.insert(Direction::South, vec![Tile::Empty]);
    grass_side_bottom_neighbors.insert(Direction::East, vec![Tile::GrassSideBottom, Tile::GrassCornerBottomRight]);
    grass_side_bottom_neighbors.insert(Direction::West, vec![Tile::GrassSideBottom, Tile::GrassCornerBottomLeft]);

    let mut grass_side_left_neighbors = HashMap::new();
    grass_side_left_neighbors.insert(Direction::North, vec![Tile::GrassSideLeft, Tile::GrassCornerTopLeft]);
    grass_side_left_neighbors.insert(Direction::South, vec![Tile::GrassSideLeft, Tile::GrassCornerBottomLeft]);
    grass_side_left_neighbors.insert(Direction::East, vec![Tile::Grass]);
    grass_side_left_neighbors.insert(Direction::West, vec![Tile::Empty]);

    let mut grass_side_right_neighbors = HashMap::new();
    grass_side_right_neighbors.insert(Direction::North, vec![Tile::GrassSideRight, Tile::GrassCornerTopRight]);
    grass_side_right_neighbors.insert(Direction::South, vec![Tile::GrassSideRight, Tile::GrassCornerBottomRight]);
    grass_side_right_neighbors.insert(Direction::East, vec![Tile::Empty]);
    grass_side_right_neighbors.insert(Direction::West, vec![Tile::Grass]);

    let mut grass_corner_top_left_neighbors = HashMap::new();
    grass_corner_top_left_neighbors.insert(Direction::North, vec![Tile::Empty]);
    grass_corner_top_left_neighbors.insert(Direction::South, vec![Tile::GrassSideLeft]);
    grass_corner_top_left_neighbors.insert(Direction::East, vec![Tile::GrassSideTop]);
    grass_corner_top_left_neighbors.insert(Direction::West, vec![Tile::Empty]);

    let mut grass_corner_top_right_neighbors = HashMap::new();
    grass_corner_top_right_neighbors.insert(Direction::North, vec![Tile::Empty]);
    grass_corner_top_right_neighbors.insert(Direction::South, vec![Tile::GrassSideRight]);
    grass_corner_top_right_neighbors.insert(Direction::East, vec![Tile::Empty]);
    grass_corner_top_right_neighbors.insert(Direction::West, vec![Tile::GrassSideTop]);

    let mut grass_corner_bottom_left_neighbors = HashMap::new();
    grass_corner_bottom_left_neighbors.insert(Direction::North, vec![Tile::GrassSideLeft]);
    grass_corner_bottom_left_neighbors.insert(Direction::South, vec![Tile::Empty]);
    grass_corner_bottom_left_neighbors.insert(Direction::East, vec![Tile::GrassSideBottom]);
    grass_corner_bottom_left_neighbors.insert(Direction::West, vec![Tile::Empty]);

    let mut grass_corner_bottom_right_neighbors = HashMap::new();
    grass_corner_bottom_right_neighbors.insert(Direction::North, vec![Tile::GrassSideRight]);
    grass_corner_bottom_right_neighbors.insert(Direction::South, vec![Tile::Empty]);
    grass_corner_bottom_right_neighbors.insert(Direction::East, vec![Tile::Empty]);
    grass_corner_bottom_right_neighbors.insert(Direction::West, vec![Tile::GrassSideBottom]);

    // let mut water_neighbors = HashMap::new();
    // water_neighbors.insert(Direction::North, vec![Tile::Water, Tile::Sand]);
    // water_neighbors.insert(Direction::South, vec![Tile::Water, Tile::Sand]);
    // water_neighbors.insert(Direction::East, vec![Tile::Water, Tile::Sand]);
    // water_neighbors.insert(Direction::West, vec![Tile::Water, Tile::Sand]);

    // let mut sand_neighbors = HashMap::new();
    // sand_neighbors.insert(Direction::North, vec![Tile::Grass, Tile::Water, Tile::Sand]);
    // sand_neighbors.insert(Direction::South, vec![Tile::Grass, Tile::Water, Tile::Sand]);
    // sand_neighbors.insert(Direction::East, vec![Tile::Grass, Tile::Water, Tile::Sand]);
    // sand_neighbors.insert(Direction::West, vec![Tile::Grass, Tile::Water, Tile::Sand]);

    vec![
        // Grass constraints
        Constraint::new(Tile::Grass, grass_neighbors),
        Constraint::new(Tile::GrassSideTop, grass_side_top_neighbors),
        Constraint::new(Tile::GrassSideBottom, grass_side_bottom_neighbors),
        Constraint::new(Tile::GrassSideLeft, grass_side_left_neighbors),
        Constraint::new(Tile::GrassSideRight, grass_side_right_neighbors),
        Constraint::new(Tile::GrassCornerTopLeft, grass_corner_top_left_neighbors),
        Constraint::new(Tile::GrassCornerTopRight, grass_corner_top_right_neighbors),
        Constraint::new(Tile::GrassCornerBottomLeft, grass_corner_bottom_left_neighbors),
        Constraint::new(Tile::GrassCornerBottomRight, grass_corner_bottom_right_neighbors),
        Constraint::new(Tile::Empty, empty_neighbors)
        // Constraint::new(Tile::Water, water_neighbors),
        // Constraint::new(Tile::Sand, sand_neighbors),
    ]
}

// Cell and Grid Definitions
#[derive(Clone, Debug)]
struct Cell {
    possibilities: HashSet<Tile>,
    collapsed: bool,
}

impl Cell {
    fn new(possible_tiles: &Vec<Tile>) -> Self {
        Cell {
            possibilities: possible_tiles.iter().cloned().collect(),
            collapsed: false
        }
    }

    fn collapse(&mut self, tile: Tile) {
        self.possibilities.clear();
        self.possibilities.insert(tile);
        self.collapsed = true;
    }
}

struct Grid {
    cells: Vec<Vec<Cell>>,
    constraints: HashMap<Tile, Constraint>,
}

impl Grid {
    fn display(&self) {
        println!("---------------");
        for y in (0..HEIGHT).rev() {
            for x in 0..WIDTH {
                if !self.cells[x][y].collapsed {
                    print!("?");
                } else {
                    let tile_char = self.cells[x][y]
                        .possibilities
                        .iter()
                        .next()
                        .map_or(' ', |&tile| tile_to_char(tile));
                    print!("{}", tile_char);
                }
            }
            println!();
        }
        println!("---------------");
    }

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
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(123);
        let mut min_possibilities = usize::MAX;
        let mut candidates = vec![];

        // This loop finds cells with the minimum number of possibilities as candidates for collapse
        // in the wave function collapse algorithm. It iterates through all cells and keeps track of 
        // the cell with the fewest remaining valid tile options. Cells with the same minimum possibility
        // count are also considered as candidates. 
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

        // This code block performs a random collapse on a chosen candidate cell and implements backtracking.
        // It first chooses a random cell from the `candidates` list. Then, it randomly chooses a 
        // possible tile from that cell.
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
                if let Some(allowed_neighbors) = self.constraints[&tile].allowed_neighbors.get(&direction) {
                    self.cells[nx][ny].possibilities.retain(|&neighbor| allowed_neighbors.contains(&neighbor));
                } 
            }
        }
    }

    fn run(&mut self) -> bool {
        while let Some((x, y, tile)) = self.random_collapse() {
            self.propagate(x, y, tile);
            println!("Placed tile {:#?} at {}, {}", tile, x, y);
            if self.cells.iter().any(|row| row.iter().any(|cell| cell.possibilities.is_empty())) {
                println!("Placing tile {:#?} at {}, {} results in a contradiction, retrying", tile, x, y);
                self.display();
                return false;
            } else {
                // No contradiction, continue collapsing
            }
        }

        // Check if all cells have a single possibility (solution found)
        self.cells.iter().all(|row| row.iter().all(|cell| cell.possibilities.len() == 1))
    }
}

fn tile_to_char(tile: Tile) -> char {
    match tile {
        Tile::Grass => 'G',
        Tile::GrassSideTop => '┬',
        Tile::GrassSideBottom => '┴',
        Tile::GrassSideLeft => '├',
        Tile::GrassSideRight => '┤',
        Tile::GrassCornerTopLeft => '⌜',
        Tile::GrassCornerTopRight => '⌝',
        Tile::GrassCornerBottomLeft => '⌞',
        Tile::GrassCornerBottomRight => '⌟',
        Tile::Empty => ' ',
        // Tile::Sand => 'S',
        // Tile::Water => 'W',
    }
}