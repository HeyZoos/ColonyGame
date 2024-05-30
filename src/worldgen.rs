use bevy::prelude::*;

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands, 
    assets: Res<AssetServer>, 
    mut layouts: ResMut<Assets<TextureAtlasLayout>>
) {
    let texture = assets.load("grass.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 4, 4, None, None);
    let handle = layouts.add(layout);

    commands.spawn(SpriteSheetBundle {
        texture,
        atlas: TextureAtlas {
            layout: handle,
            index: 0
        },
        ..default()
    });
}
