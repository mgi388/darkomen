use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_derive::Deref;
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::light::{DecodeError, Decoder, Light};

pub struct LightAssetPlugin;

impl Plugin for LightAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LightsAsset>()
            .init_asset_loader::<LightsAssetLoader>();
        #[cfg(feature = "bevy_reflect")]
        app.register_asset_reflect::<LightsAsset>();
    }
}

#[derive(Asset, Clone, Deref)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(not(feature = "bevy_reflect"), derive(TypePath))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct LightsAsset(pub Vec<Light>);

#[derive(Clone, Default, TypePath)]
pub struct LightsAssetLoader;

#[derive(Clone, Copy, Default, Deserialize, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Default, Deserialize, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct LightsAssetLoaderSettings;

/// Possible errors that can be produced by [`LightsAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum LightsAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode lights: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for LightsAssetLoader {
    type Asset = LightsAsset;
    type Settings = LightsAssetLoaderSettings;
    type Error = LightsAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = std::io::Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let lights = decoder.decode()?;

        Ok(LightsAsset(lights))
    }

    fn extensions(&self) -> &[&str] {
        &["LIT", "lit"]
    }
}
