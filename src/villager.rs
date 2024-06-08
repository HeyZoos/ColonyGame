use bevy::prelude::*;
use grid_2d::Coord;
use pathfinding::prelude::astar;
use std::time::Duration;

pub struct VillagerPlugin;

impl Plugin for VillagerPlugin {
	fn build(&self, app: &mut App) {
		app
            .add_systems(PostStartup, post_startup)
			.add_systems(Update, animate_sprite)
            .add_systems(Update, pathfinding);
	}
}

fn post_startup(
	mut cmds: Commands, 
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<crate::worldgen::World>
) {
    let goal = Coord { x: 5, y : 5 };
    let result = astar(
        /* start */ &Coord { x: 0, y: 0 },
        /* successors */ |&Coord { x, y }| {
            let mut next_coords = vec![
                Coord { x: x + 1, y },
                Coord { x: x - 1, y },
                Coord { x, y: y + 1 },
                Coord { x, y: y - 1 }
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
        /* heuristic */ |coord| {
            coord.distance2(goal) / 3
        },
        /* success */ |coord| {
            *coord == goal
        });
    
    // Load the character sprite sheet
    let texture: Handle<Image> = assets.load("character.png");

    // Create a TextureAtlas from the sprite sheet
    let layout = TextureAtlasLayout::from_grid(
        Vec2::new(48.0, 48.0),
        8,
        24,
        None,
        None
    );

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation_indices = AnimationIndices { first: 0, last: 7 };

    // Spawn an animated character using the sprite sheet
    cmds.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first
            },
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        },
        animation_indices,
        AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        // Assign the path to a villager
        Speed(1.0),
        Pathfinder::new(result.unwrap().0)
    ));
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
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

#[derive(Component)]
struct Pathfinder {
    path: Vec<Coord>,
    current_target: Option<Coord>,
}

impl Pathfinder {
    fn new(path: Vec<Coord>) -> Self {
        let current_target = path.first().cloned();
        Pathfinder { path, current_target }
    }
}

fn pathfinding(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Speed, &mut Pathfinder)>,
) {
    for (mut transform, speed, mut pathfinder) in query.iter_mut() {
        let delta = time.delta_seconds();

        if let Some(current_target) = pathfinder.current_target {
            // Check if we have reached the current target
            if (transform.translation.x - current_target.x as f32).abs() < 0.1 &&
                (transform.translation.y - current_target.y as f32).abs() < 0.1 {
                // Move to the next target in the path
                pathfinder.path.remove(0);
                pathfinder.current_target = pathfinder.path.first().cloned();
            }

            // Calculate the direction vector towards the current target
            let direction = Coord {
                x: current_target.x - transform.translation.x as i32,
                y: current_target.y - transform.translation.y as i32,
            };

            // Normalize the directions
            let length = ((direction.x.pow(2) + direction.y.pow(2)) as f32).sqrt() as i32;
            let direction = if length != 0 {
                Coord {
                    x: direction.x / length,
                    y: direction.y / length,
                }
            } else {
                direction
            };

            // Move the villager towards the current target
            transform.translation.x += direction.x as f32 * speed.0 * delta;
            transform.translation.y += direction.y as f32 * speed.0 * delta;
        }
    }
}