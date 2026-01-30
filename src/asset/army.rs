use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::army::*;

use super::{graphics::sprite_sheet::*, paths::*};

pub struct ArmyAssetPlugin;

impl Plugin for ArmyAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPathsPlugin>() {
            app.add_plugins(AssetPathsPlugin);
        }
        if !app.is_plugin_added::<SpriteSheetAssetPlugin>() {
            app.add_plugins(SpriteSheetAssetPlugin);
        }

        app.init_asset::<ArmyAsset>()
            .init_asset_loader::<ArmyAssetLoader>();
        #[cfg(feature = "bevy_reflect")]
        app.register_asset_reflect::<ArmyAsset>();
    }
}

#[derive(Asset, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(not(feature = "bevy_reflect"), derive(TypePath))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(Default))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct ArmyAsset {
    source: Army,

    pub small_banners: Option<Handle<SpriteSheetAsset>>,
    pub disabled_small_banners: Option<Handle<SpriteSheetAsset>>,
    pub large_banners: Option<Handle<SpriteSheetAsset>>,
}

impl ArmyAsset {
    #[inline(always)]
    pub fn get(&self) -> &Army {
        &self.source
    }
}

/// A [`Handle`] to an [`ArmyAsset`] asset.
#[derive(Clone, Component, Default, Deref, DerefMut, Eq, From, Hash, PartialEq, Reflect)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Component, Default, Hash, PartialEq)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct ArmyAssetHandle(pub Handle<ArmyAsset>);

impl From<ArmyAssetHandle> for AssetId<ArmyAsset> {
    fn from(handle: ArmyAssetHandle) -> Self {
        handle.id()
    }
}

impl From<&ArmyAssetHandle> for AssetId<ArmyAsset> {
    fn from(handle: &ArmyAssetHandle) -> Self {
        handle.id()
    }
}

#[derive(Clone, TypePath)]
pub struct ArmyAssetLoader {
    paths: AssetPaths,
}

#[derive(Clone, Copy, Default, Deserialize, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Default, Deserialize, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct ArmyAssetLoaderSettings {
    pub load_small_banners: bool,
    pub load_disabled_small_banners: bool,
    pub load_large_banners: bool,
}

/// Possible errors that can be produced by [`ArmyAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum ArmyAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode army: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for ArmyAssetLoader {
    type Asset = ArmyAsset;
    type Settings = ArmyAssetLoaderSettings;
    type Error = ArmyAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = std::io::Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let army = decoder.decode()?;

        Ok(ArmyAsset {
            source: army.clone(),
            small_banners: if settings.load_small_banners {
                Some(load_context.load(self.paths.resolve_path(&army.small_banners_path)))
            } else {
                None
            },
            disabled_small_banners: if settings.load_disabled_small_banners {
                Some(load_context.load(self.paths.resolve_path(&army.disabled_small_banners_path)))
            } else {
                None
            },
            large_banners: if settings.load_large_banners {
                Some(load_context.load(self.paths.resolve_path(&army.large_banners_path)))
            } else {
                None
            },
        })
    }

    fn extensions(&self) -> &[&str] {
        // - "ARM" is the extension for army files.
        //
        // - "ARE" is the extension for empty army files used in multiplayer.
        //
        //   They contain:
        //
        //   - The army header/metadata (race, banner paths, etc.).
        //   - Starting gold amount.
        //   - An empty regiments list.
        //   - The starting point for creating a new multiplayer army.
        //
        // - "AUD" is the extension for catalog/template files used in
        //   multiplayer.
        //
        //   They contain:
        //
        //   - All available regiment types for a race.
        //   - Multiple experience tier variants of each unit (0, 1000, 3000,
        //     6000 XP).
        //   - Different cost tiers based on experience level.
        //
        //  They are essentially the "shopping catalog" for multiplayer army
        //  building.
        //
        // - "{xxx}", not included here, (where `{xxx}` is a 3-digit number) is
        //   the extension for save games, but they are not able to be
        //   automatically loaded by this loader.
        &["ARM", "arm", "AUD", "aud", "ARE", "are"]
    }
}

impl FromWorld for ArmyAssetLoader {
    fn from_world(world: &mut World) -> Self {
        let paths = world.resource::<AssetPaths>();

        Self {
            paths: paths.clone(),
        }
    }
}
