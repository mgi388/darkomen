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
            .init_asset_loader::<ArmyAssetLoader>()
            .register_asset_reflect::<ArmyAsset>();
    }
}

#[derive(Asset, Clone, Debug, Default, Reflect)]
#[reflect(Debug, Default)]
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
#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Eq, From, Hash, PartialEq, Reflect)]
#[reflect(Component, Debug, Default, Hash, PartialEq)]
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

#[derive(Clone)]
pub struct ArmyAssetLoader {
    paths: AssetPaths,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
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
        // - "ARE" is the extension for empty army files used in multiplayer.
        //   Empty army files' `regiment` field is empty.
        // - "AUD" extension is used in multiplayer, but not clear what it is
        //   used for.
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
