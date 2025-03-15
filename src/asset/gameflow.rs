use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_derive::Deref;
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::gameflow::{DecodeError, Decoder};

pub struct GameflowAssetPlugin;

impl Plugin for GameflowAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<GameflowAsset>()
            .init_asset_loader::<GameflowAssetLoader>()
            .register_asset_reflect::<GameflowAsset>();
    }
}

#[derive(Clone, Debug, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Deserialize, Serialize)]
pub struct GameflowPath {
    /// The points that the gameflow path follows in order.
    pub points: Vec<UVec2>,
}

#[derive(Asset, Clone, Debug, Deref, Reflect)]
#[reflect(Debug)]
pub struct GameflowAsset {
    /// The paths that the gameflow follows. Each path is a series of points
    /// that the gameflow follows in order.
    pub paths: Vec<GameflowPath>,
}

#[derive(Clone, Default)]
pub struct GameflowAssetLoader;

#[derive(Clone, Copy, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
pub struct GameflowAssetLoaderSettings;

/// Possible errors that can be produced by [`GameflowAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum GameflowAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode gameflow: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for GameflowAssetLoader {
    type Asset = GameflowAsset;
    type Settings = GameflowAssetLoaderSettings;
    type Error = GameflowAssetLoaderError;
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

        let g = decoder.decode()?;

        Ok(GameflowAsset {
            paths: g
                .paths
                .into_iter()
                .map(|p| GameflowPath {
                    points: p.points.into_iter().map(|p| UVec2::new(p.x, p.y)).collect(),
                })
                .collect(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["DOT", "dot"]
    }
}
