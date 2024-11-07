use bevy_app::prelude::*;

use crate::asset::graphics::sprite_sheet::SpriteSheetAssetPlugin;

mod graphics;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::asset::graphics::sprite_sheet::*;
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SpriteSheetAssetPlugin>() {
            app.add_plugins(SpriteSheetAssetPlugin);
        }
    }
}
