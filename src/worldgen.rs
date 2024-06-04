use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{seq::IteratorRandom, thread_rng, SeedableRng};
use std::collections::{self, HashMap, HashSet};

// Constants
const WIDTH: usize = 3;
const HEIGHT: usize = 3;
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

    let constraints = TileConstraints::from_pattern(&vec![
        vec![
            Tile::GrassCornerTopLeft,
            Tile::GrassSideTop,
            Tile::GrassCornerTopRight,
        ],
        vec![Tile::GrassSideLeft, Tile::Grass, Tile::GrassSideRight],
        vec![
            Tile::GrassCornerBottomLeft,
            Tile::GrassSideBottom,
            Tile::GrassCornerBottomRight,
        ],
    ]);

    let mut grid = Grid::new(constraints.clone());

    // Just keep trying if it doesn't collapse
    while !grid.run() {
        grid = Grid::new(constraints.clone());
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
            collapsed: false,
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
    constraints: TileConstraints,
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

    fn new(constraints: TileConstraints) -> Self {
        let tiles: Vec<Tile> = constraints.tiles();
        Grid {
            cells: vec![vec![Cell::new(&tiles); HEIGHT]; WIDTH],
            constraints,
        }
    }

    fn random_collapse(&mut self) -> Option<(usize, usize, Tile)> {
        let mut rng = thread_rng();
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
        println!("Placed tile {:#?} at {}, {}", tile, x, y);
        let directions = vec![
            (x.wrapping_sub(1), y, Direction::West),
            (x + 1, y, Direction::East),
            (x, y.wrapping_sub(1), Direction::South),
            (x, y + 1, Direction::North),
        ];

        for (nx, ny, direction) in directions {
            if nx < WIDTH && ny < HEIGHT {
                if let Some(allowed_neighbors) = self.constraints.value[&tile].value.get(&direction)
                {
                    self.cells[nx][ny]
                        .possibilities
                        .retain(|&neighbor| allowed_neighbors.contains(&neighbor));
                    if self.cells[nx][ny].possibilities.is_empty() {
                        println!(
                            "Placing tile {:#?} at {}, {} results in a contradiction at {}, {}",
                            tile, x, y, nx, ny
                        );
                    }
                }
            }
        }
    }

    fn run(&mut self) -> bool {
        while let Some((x, y, tile)) = self.random_collapse() {
            self.propagate(x, y, tile);
            if self
                .cells
                .iter()
                .any(|row| row.iter().any(|cell| cell.possibilities.is_empty()))
            {
                self.display();
                return false;
            } else {
                // No contradiction, continue collapsing
            }
        }

        // Check if all cells have a single possibility (solution found)
        self.cells
            .iter()
            .all(|row| row.iter().all(|cell| cell.possibilities.len() == 1))
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

#[derive(Clone, Debug)]
pub struct DirectionalContraints {
    value: HashMap<Direction, HashSet<Tile>>,
}

impl DirectionalContraints {
    pub fn new() -> Self {
        let mut value = HashMap::new();
        value.insert(Direction::North, HashSet::new());
        value.insert(Direction::South, HashSet::new());
        value.insert(Direction::East, HashSet::new());
        value.insert(Direction::West, HashSet::new());
        Self { value }
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut constraints = self.clone();
        for (direction, tiles) in &other.value {
            constraints
                .value
                .entry(direction.clone())
                .or_insert_with(HashSet::new)
                .extend(tiles.iter().cloned());
        }
        constraints
    }
}

#[derive(Clone, Debug)]
pub struct TileConstraints {
    value: HashMap<Tile, DirectionalContraints>,
}

impl TileConstraints {
    pub fn new() -> Self {
        Self {
            value: HashMap::new(),
        }
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut constraints = self.clone();
        for (tile, directionals) in &other.value {
            constraints
                .value
                .entry(tile.clone())
                .and_modify(|entry| *entry = entry.concat(directionals))
                .or_insert_with(|| directionals.clone());
        }
        constraints
    }

    pub fn from_pattern(grid: &[Vec<Tile>]) -> Self {
        let mut constraints = TileConstraints::new();

        for (y, row) in grid.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let mut neighbors = DirectionalContraints::new();

                if y > 0 {
                    neighbors
                        .value
                        .get_mut(&Direction::North)
                        .unwrap()
                        .insert(grid[y - 1][x].clone());
                } else {
                    neighbors
                        .value
                        .get_mut(&Direction::North)
                        .unwrap()
                        .insert(Tile::Empty);
                }

                if y < grid.len() - 1 {
                    neighbors
                        .value
                        .get_mut(&Direction::South)
                        .unwrap()
                        .insert(grid[y + 1][x].clone());
                } else {
                    neighbors
                        .value
                        .get_mut(&Direction::South)
                        .unwrap()
                        .insert(Tile::Empty);
                }

                if x > 0 {
                    neighbors
                        .value
                        .get_mut(&Direction::West)
                        .unwrap()
                        .insert(grid[y][x - 1].clone());
                } else {
                    neighbors
                        .value
                        .get_mut(&Direction::West)
                        .unwrap()
                        .insert(Tile::Empty);
                }

                if x < row.len() - 1 {
                    neighbors
                        .value
                        .get_mut(&Direction::East)
                        .unwrap()
                        .insert(grid[y][x + 1].clone());
                } else {
                    neighbors
                        .value
                        .get_mut(&Direction::East)
                        .unwrap()
                        .insert(Tile::Empty);
                }

                constraints
                    .value
                    .entry(tile.clone())
                    .and_modify(|entry| *entry = entry.concat(&neighbors))
                    .or_insert_with(|| neighbors);
            }
        }

        constraints
    }

    pub fn keys(&self) -> Vec<Tile> {
        self.value.keys().cloned().collect()
    }

    pub fn tiles(&self) -> Vec<Tile> {
        self.keys()
    }
}
