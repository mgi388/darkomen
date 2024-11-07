use bevy_app::prelude::*;

use crate::asset::{
    army::ArmyAssetPlugin, battle_tabletop::BattleTabletopAssetPlugin,
    graphics::sprite_sheet::SpriteSheetAssetPlugin, light::LightAssetPlugin,
    paths::AssetPathsPlugin,
};

mod army;
mod battle_tabletop;
mod graphics;
mod light;
mod paths;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::asset::army::*;
    #[doc(hidden)]
    pub use crate::asset::battle_tabletop::*;
    #[doc(hidden)]
    pub use crate::asset::graphics::sprite_sheet::*;
    #[doc(hidden)]
    pub use crate::asset::light::*;
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPathsPlugin>() {
            app.add_plugins(AssetPathsPlugin);
        }
        if !app.is_plugin_added::<SpriteSheetAssetPlugin>() {
            app.add_plugins(SpriteSheetAssetPlugin);
        }
        if !app.is_plugin_added::<LightAssetPlugin>() {
            app.add_plugins(LightAssetPlugin);
        }
        if !app.is_plugin_added::<ArmyAssetPlugin>() {
            app.add_plugins(ArmyAssetPlugin);
        }
        if !app.is_plugin_added::<BattleTabletopAssetPlugin>() {
            app.add_plugins(BattleTabletopAssetPlugin);
        }
    }
}
