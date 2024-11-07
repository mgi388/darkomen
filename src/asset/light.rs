use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, AsyncReadExt, LoadContext};
use bevy_derive::Deref;
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::light::{DecodeError, Decoder, Light};

pub struct LightAssetPlugin;

impl Plugin for LightAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LightsAsset>()
            .init_asset_loader::<LightsAssetLoader>()
            .register_asset_reflect::<LightsAsset>();
    }
}

#[derive(Asset, Clone, Debug, Deref, Reflect)]
#[reflect(Debug)]
pub struct LightsAsset(pub Vec<Light>);

#[derive(Clone, Default)]
pub struct LightsAssetLoader;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct LightsAssetLoaderSettings;

/// Possible errors that can be produced by [`LightsAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum LightsAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode light: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for LightsAssetLoader {
    type Asset = LightsAsset;
    type Settings = LightsAssetLoaderSettings;
    type Error = LightsAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a LightsAssetLoaderSettings,
        _load_context: &'a mut LoadContext<'_>,
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
