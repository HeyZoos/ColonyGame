use bevy::prelude::*;
use grid_2d::Coord;
use pathfinding::prelude::astar;
use std::time::Duration;

pub struct VillagerPlugin;

impl Plugin for VillagerPlugin {
	fn build(&self, app: &mut App) {
		app
            .add_systems(PostStartup, post_startup)
			.add_systems(Update, animate_sprite);
	}
}

fn post_startup(
	mut cmds: Commands, 
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    world: Res<crate::worldgen::World>
) {
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
    	AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating))
    ));

    let goal = Coord { x: 5, y : 5 };
    let result = astar(
        /* start */ &grid_2d::Coord { x: 0, y: 0 },
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
    dbg!(result);
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