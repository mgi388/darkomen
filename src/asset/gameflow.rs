use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::gameflow::{DecodeError, Decoder};

pub struct GameflowAssetPlugin;

impl Plugin for GameflowAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<GameflowAsset>()
            .init_asset_loader::<GameflowAssetLoader>();
        #[cfg(feature = "bevy_reflect")]
        app.register_asset_reflect::<GameflowAsset>();
    }
}

#[derive(Clone, Default, Deserialize, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Default, Deserialize, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct GameflowPath {
    /// The control points used to make a curve that represents the path.
    pub control_points: Vec<UVec2>,

    pub frames_per_point: u64,

    /// The distance in pixels between interpolated points along the path's
    /// curve.
    ///
    /// Used by the curve generation algorithm to determine rendering
    /// granularity.
    pub curve_point_spacing: u32,
}

#[derive(Asset, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(not(feature = "bevy_reflect"), derive(TypePath))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct GameflowAsset {
    /// The paths that the gameflow follows.
    pub paths: Vec<GameflowPath>,
    pub animation_frame_interval_millis_x2: u64,
}

#[derive(Clone, Default, TypePath)]
pub struct GameflowAssetLoader;

#[derive(Clone, Copy, Default, Deserialize, Reflect, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[reflect(Default, Deserialize, Serialize)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
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
                    control_points: p
                        .control_points
                        .into_iter()
                        .map(|p| UVec2::new(p.x, p.y))
                        .collect(),
                    frames_per_point: p.frames_per_point as u64,
                    curve_point_spacing: p.curve_point_spacing as u32,
                })
                .collect(),
            animation_frame_interval_millis_x2: g.animation_frame_interval_millis_x2 as u64,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["DOT", "dot"]
    }
}
