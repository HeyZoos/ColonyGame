use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(crate::states::States::Load)
                .continue_to_state(crate::states::States::Play)
                .load_collection::<CharacterAssets>(),
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
