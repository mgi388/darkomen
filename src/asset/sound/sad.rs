use std::io::Cursor;

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_kira_audio::prelude::*;
use derive_more::derive::{Display, Error, From};

use crate::sound::sad::{DecodeError, Decoder};

pub struct StereoAudioAssetPlugin;

impl Plugin for StereoAudioAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AudioPlugin>() {
            app.add_plugins(AudioPlugin);
        }

        app.init_asset::<StereoAudioAsset>()
            .init_asset_loader::<StereoAudioAssetLoader>();
    }
}

pub type StereoAudioAsset = AudioSource;

#[derive(Clone, Default)]
pub struct StereoAudioAssetLoader;

/// Possible errors that can be produced by [`StereoAudioAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum StereoAudioAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode sad: {_0}")]
    DecodeError(DecodeError),
    #[display("could not transform asset: {_0}")]
    FromFileError(FromFileError),
}

impl AssetLoader for StereoAudioAssetLoader {
    type Asset = StereoAudioAsset;
    type Settings = ();
    type Error = StereoAudioAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let sad = decoder.decode()?;

        let sound = StaticSoundData::from_cursor(Cursor::new(sad.to_wav()?))?;

        Ok(StereoAudioAsset { sound })
    }

    fn extensions(&self) -> &[&str] {
        &["SAD", "sad"]
    }
}
