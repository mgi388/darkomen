use std::io::Cursor;

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, AsyncReadExt, LoadContext};
use bevy_kira_components::{
    kira::sound::static_sound::StaticSoundSettings, prelude::*,
    sources::audio_file::AudioFilePlugin,
};
use derive_more::derive::{Display, Error, From};

use crate::sound::mad::{DecodeError, Decoder};

pub struct MonoAudioAssetPlugin;

impl Plugin for MonoAudioAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AudioFilePlugin>() {
            app.add_plugins(AudioFilePlugin);
        }

        app.init_asset::<MonoAudioAsset>()
            .init_asset_loader::<MonoAudioAssetLoader>();
    }
}

pub type MonoAudioAsset = AudioFile;

#[derive(Clone, Default)]
pub struct MonoAudioAssetLoader;

/// Possible errors that can be produced by [`MonoAudioAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum MonoAudioAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode mad: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for MonoAudioAssetLoader {
    type Asset = MonoAudioAsset;
    type Settings = ();
    type Error = MonoAudioAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let mad = decoder.decode()?;

        Ok(AudioFile::Static(
            mad.to_wav()?.into(),
            StaticSoundSettings::default(),
        ))
    }

    fn extensions(&self) -> &[&str] {
        &["MAD", "mad"]
    }
}
