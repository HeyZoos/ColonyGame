use bevy::prelude::*;
use std::time::Duration;

pub struct VillagerPlugin;

impl Plugin for VillagerPlugin {
	fn build(&self, app: &mut App) {
		app
            .add_systems(Startup, startup)
			.add_systems(Update, animate_sprite);
	}
}

fn startup(
	mut cmds: Commands, 
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
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