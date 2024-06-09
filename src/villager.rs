use crate::animation::AnimationBundle;
use bevy::prelude::*;
use bevy::utils::petgraph::matrix_graph::Zero;
use extend::ext;
use grid_2d::Coord;
use pathfinding::prelude::astar;
use std::time::Duration;

pub struct VillagerPlugin;

impl Plugin for VillagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, post_startup)
            .add_systems(Update, animate_sprite)
            .add_systems(Update, movement_system);
    }
}

fn post_startup(
    mut cmds: Commands,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<crate::worldgen::World>,
) {
    let goal = Coord { x: 10, y: 5 };
    let result = astar(
        /* start */ &Coord { x: 0, y: 0 },
        /* successors */
        |&Coord { x, y }| {
            let mut next_coords = vec![
                Coord { x: x + 1, y },
                Coord { x: x - 1, y },
                Coord { x, y: y + 1 },
                Coord { x, y: y - 1 },
            ];

            next_coords.retain(|&coord| {
                if let Some(cell) = world.wave.grid().get(coord) {
                    cell.chosen_pattern_id().unwrap() != 255
                } else {
                    false
                }
            });

            next_coords.into_iter().map(|c| (c, 1))
        },
        /* heuristic */ |coord| coord.distance2(goal) / 3,
        /* success */ |coord| *coord == goal,
    );

    // Load the character sprite sheet
    let texture: Handle<Image> = assets.load("character.png");

    // Create a TextureAtlas from the sprite sheet
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 8, 24, None, None);

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_indices = AnimationIndices { first: 0, last: 7 };

    // Spawn an animated character using the sprite sheet
    cmds.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        },
        animation_indices,
        AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        // Assign the path to a villager
        Speed(16.0),
        Movement::new(result.unwrap().0),
        AnimationBundle::default(),
    ));
}

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

#[derive(Component)]
struct Speed(f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Converts a `Direction` to a `Vec2`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::villager::Direction;
    ///
    /// let dir = Direction::Up;
    /// assert_eq!(dir.to_vec2(), Vec2::new(0.0, 1.0));
    /// ```
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, 1.0),
            Direction::Down => Vec2::new(0.0, -1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
        }
    }
}

#[derive(Component)]
pub struct Movement {
    pub path: Vec<Coord>,
}

impl Movement {
    fn new(path: Vec<Coord>) -> Self {
        Movement { path }
    }

    pub fn target(&self) -> Option<Vec2> {
        self.path.first().map(|v| v.to_vec2().to_world_space())
    }
}

fn movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &Speed, &mut Movement)>) {
    let delta = time.delta_seconds();

    if delta.is_zero() {
        return;
    }

    for (mut transform, speed, mut movement) in query.iter_mut() {
        if let Some(target) = movement.target() {

            // Check if we have reached the current target
            if transform.translation.xy().distance(target) < 0.1 {
                // Move to the next target in the path
                movement.path.remove(0);
            }

            if let Some(target) = movement.target() {
                // Calculate and normalize the heading vector towards the current target
                let heading = transform.translation.xy().towards(&target);

                // Move the villager towards the current target
                transform.translation.x += heading.x * speed.0 * delta;
                transform.translation.y += heading.y * speed.0 * delta;
            }
        }
    }
}

#[ext]
pub impl Coord {
    /// Converts a `Coord` to a `Vec2`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use grid_2d::Coord;
    /// use bevy_game::villager::CoordExt;
    ///
    /// let coord = Coord { x: 3, y: 4 };
    /// let vec = coord.to_vec2();
    /// assert_eq!(vec, Vec2::new(3.0, 4.0));
    /// ```
    fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

#[ext]
pub impl Vec2 {
    /// Converts a `Vec2` to world space (16x16 units).
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::villager::Vec2Ext;
    ///
    /// let vec = Vec2::new(2.0, 3.0);
    /// let world_space_vec = vec.to_world_space();
    /// assert_eq!(world_space_vec, Vec2::new(32.0, 48.0));
    /// ```
    fn to_world_space(&self) -> Vec2 {
        *self * 16.0
    }

    /// Returns a normalized vector pointing from `self` towards `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::villager::Vec2Ext;
    ///
    /// let vec1 = Vec2::new(1.0, 1.0);
    /// let vec2 = Vec2::new(4.0, 5.0);
    /// let direction = vec1.towards(&vec2);
    /// assert!((direction - Vec2::new(0.6, 0.8)).length() < 1e-5);
    /// ```
    fn towards(&self, other: &Vec2) -> Vec2 {
        (*other - *self).normalize()
    }

    /// Converts a `Vec2` to a `Direction`, rounding the vector components.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::villager::Direction;
    /// use bevy_game::villager::Vec2Ext;
    ///
    /// let vec = Vec2::new(0.1, 0.9);
    /// assert_eq!(vec.to_direction(), Some(Direction::Up));
    ///
    /// let invalid_vec = Vec2::new(1.1, 1.1);
    /// assert_eq!(invalid_vec.to_direction(), None);
    /// ```
    fn to_direction(&self) -> Option<Direction> {
        match self.round() {
            v if v == Vec2::new(0.0, 1.0) => Some(Direction::Up),
            v if v == Vec2::new(0.0, -1.0) => Some(Direction::Down),
            v if v == Vec2::new(-1.0, 0.0) => Some(Direction::Left),
            v if v == Vec2::new(1.0, 0.0) => Some(Direction::Right),
            _ => None,
        }
    }

    /// Returns the `Direction` from `self` towards `other`, if the direction is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_game::villager::Direction;
    /// use bevy_game::villager::Vec2Ext;
    ///
    /// let vec1 = Vec2::new(1.0, 1.0);
    /// let vec2 = Vec2::new(4.0, 1.0);
    /// let direction = vec1.to_direction_towards(&vec2);
    /// assert_eq!(direction, Some(Direction::Right));
    ///
    /// let vec1 = Vec2::new(0.0, 0.0);
    /// let vec2 = Vec2::new(1.0, 0.0);
    /// let direction = vec1.to_direction_towards(&vec2);
    /// assert_eq!(direction, Some(Direction::Right));
    /// ```
    fn to_direction_towards(&self, other: &Vec2) -> Option<Direction> {
        self.towards(other).normalize().to_direction()
    }
}