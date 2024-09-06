use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::states::States::LoadMenu;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(LoadMenu)
                .load_collection::<AudioAssets>()
                .load_collection::<CharacterAssets>()
                .load_collection::<UiAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/hover.wav")]
    pub hover: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct CharacterAssets {
    #[asset(path = "character.png")]
    pub image: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 48, tile_size_y = 48, columns = 8, rows = 24))]
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "image/buttons.png")]
    pub buttons_image: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 26, tile_size_y = 26, columns = 2, rows = 4))]
    pub _buttons_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "image/xs.png")]
    pub xs_image: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 10, columns = 2, rows = 12))]
    pub _xs_layout: Handle<TextureAtlasLayout>,
}
