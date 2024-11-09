use bevy_app::prelude::*;

use crate::asset::{
    army::ArmyAssetPlugin, battle_tabletop::BattleTabletopAssetPlugin,
    graphics::sprite_sheet::SpriteSheetAssetPlugin, light::LightAssetPlugin,
    lightmap::LightmapAssetPlugin, paths::AssetPathsPlugin, sound::SoundAssetPlugin,
};

mod army;
mod battle_tabletop;
pub mod graphics;
mod light;
mod lightmap;
pub mod m3d;
mod paths;
pub mod sound;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::asset::army::*;
    #[doc(hidden)]
    pub use crate::asset::battle_tabletop::*;
    #[doc(hidden)]
    pub use crate::asset::graphics::sprite_sheet::*;
    #[doc(hidden)]
    pub use crate::asset::light::*;
    #[doc(hidden)]
    pub use crate::asset::lightmap::*;
    #[doc(hidden)]
    pub use crate::asset::m3d::*;
    #[doc(hidden)]
    pub use crate::asset::paths::*;
    #[doc(hidden)]
    pub use crate::asset::sound::mad::*;
    #[doc(hidden)]
    pub use crate::asset::sound::music_script::*;
    #[doc(hidden)]
    pub use crate::asset::sound::sad::*;
    #[doc(hidden)]
    pub use crate::asset::sound::*;
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
        if !app.is_plugin_added::<SoundAssetPlugin>() {
            app.add_plugins(SoundAssetPlugin);
        }
        if !app.is_plugin_added::<LightAssetPlugin>() {
            app.add_plugins(LightAssetPlugin);
        }
        if !app.is_plugin_added::<LightmapAssetPlugin>() {
            app.add_plugins(LightmapAssetPlugin);
        }
        if !app.is_plugin_added::<ArmyAssetPlugin>() {
            app.add_plugins(ArmyAssetPlugin);
        }
        if !app.is_plugin_added::<BattleTabletopAssetPlugin>() {
            app.add_plugins(BattleTabletopAssetPlugin);
        }
    }
}
