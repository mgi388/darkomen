use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_image::Image;
use bevy_reflect::prelude::*;
use bevy_render::render_asset::RenderAssetUsages;
use derive_more::derive::{Display, Error, From};
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::shadow::{DecodeError, Decoder, Lightmap};

pub struct LightmapAssetPlugin;

impl Plugin for LightmapAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LightmapAsset>()
            .init_asset_loader::<LightmapAssetLoader>()
            .register_asset_reflect::<LightmapAsset>();
    }
}

#[derive(Asset, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct LightmapAsset {
    source: Lightmap,
    pub texture: Handle<Image>,
}

#[derive(Clone, Debug, Default)]
pub struct LightmapAssetLoader;

#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
pub struct LightmapAssetLoaderSettings;

/// Possible errors that can be produced by [`LightmapAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum LightmapAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// An error caused when the asset path cannot be determined.
    #[display("could not determine file path of asset")]
    IndeterminateFilePath,
    /// A [DecodeError] error.
    #[display("could not decode lightmap: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for LightmapAssetLoader {
    type Asset = LightmapAsset;
    type Settings = LightmapAssetLoaderSettings;
    type Error = LightmapAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = std::io::Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let lightmap = decoder.decode()?;

        let image = transform_image(lightmap.image());

        let texture = load_context.labeled_asset_scope("texture".to_string(), |_| {
            Image::from_dynamic(
                image.clone(),
                true,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
        });

        Ok(LightmapAsset {
            source: lightmap,
            texture,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["SHD", "shd"]
    }
}

fn transform_image(img: DynamicImage) -> DynamicImage {
    img.rotate270() // rotate 90 degrees counter-clockwise

    // For now, don't do anything else.
}
