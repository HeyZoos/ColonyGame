use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::states::States::LoadMenu;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(LoadMenu)
                .load_collection::<CharacterAssets>()
                .load_collection::<UiAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct CharacterAssets {
    #[asset(path = "character.png")]
    pub image: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 48.0, tile_size_y = 48.0, columns = 8, rows = 24))]
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "image/buttons.png")]
    pub buttons_image: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 26.0, tile_size_y = 26.0, columns = 2, rows = 4))]
    pub buttons_layout: Handle<TextureAtlasLayout>,
}
