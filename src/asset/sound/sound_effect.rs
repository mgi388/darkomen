use std::{collections::HashMap, path::PathBuf};

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_kira_components::{prelude::*, sources::audio_file::AudioFilePlugin};
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::{asset::paths::*, sound::sfx::*};

pub struct SoundEffectAssetPlugin;

impl Plugin for SoundEffectAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPathsPlugin>() {
            app.add_plugins(AssetPathsPlugin);
        }
        if !app.is_plugin_added::<AudioFilePlugin>() {
            app.add_plugins(AudioFilePlugin);
        }

        app.init_asset::<PacketAsset>()
            .init_asset_loader::<PacketAssetLoader>()
            .register_asset_reflect::<PacketAsset>()
            .register_type::<PacketAssetHandle>();
    }
}

#[derive(Asset, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct PacketAsset {
    source: Packet,
    audio_files: HashMap<String, Handle<AudioFile>>,
}

/// A [`Handle`] to a [`PacketAsset`] asset.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Eq, From, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub struct PacketAssetHandle(pub Handle<PacketAsset>);

impl From<PacketAssetHandle> for AssetId<PacketAsset> {
    fn from(handle: PacketAssetHandle) -> Self {
        handle.id()
    }
}

impl From<&PacketAssetHandle> for AssetId<PacketAsset> {
    fn from(handle: &PacketAssetHandle) -> Self {
        handle.id()
    }
}

impl PacketAsset {
    pub fn sound_effect(&self, sfx_id: SfxId) -> Option<&Sfx> {
        self.source.sfxs.get(&sfx_id)
    }

    pub fn sound_effect_sound(&self, sound_effect: &Sfx, sound_index: usize) -> Option<Sound> {
        sound_effect.sounds.get(sound_index).cloned()
    }

    pub fn audio_file_handle(&self, sound: &Sound) -> Option<Handle<AudioFile>> {
        self.audio_files
            .get(if sound.file_stem == "!Null" {
                "null250"
            } else {
                sound.file_stem.as_str()
            })
            .cloned()
    }
}

#[derive(Clone)]
pub struct PacketAssetLoader {
    asset_paths: AssetPaths,
}

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
pub struct PacketAssetLoaderSettings {
    pub sound_path: PathBuf,
}

/// Possible errors that can be produced by [`PacketAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum PacketAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode packet: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for PacketAssetLoader {
    type Asset = PacketAsset;
    type Settings = PacketAssetLoaderSettings;
    type Error = PacketAssetLoaderError;
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

        let packet = decoder.decode()?;

        let sound_path = if settings.sound_path.to_string_lossy().is_empty() {
            self.asset_paths.sound_path.clone()
        } else {
            settings.sound_path.clone()
        };

        let file_names = packet
            .sfxs
            .values()
            .flat_map(|sfx| {
                sfx.sounds
                    .iter()
                    // TODO: There is no `!Null` audio file in the original
                    // game. There are other spacers like `null250`, and
                    // `silence2`. Maybe `!Null` was used by some artists as a
                    // placeholder spacer and in game they were replaced with
                    // actual audio files. For now, replace it so we don't try
                    // and load a non-existent file.
                    .map(|sound| {
                        if sound.file_stem == "!Null" {
                            "null250"
                        } else {
                            sound.file_stem.as_str()
                        }
                    })
            })
            .collect::<Vec<_>>();

        let mut audio_files = HashMap::new();

        for file_name in file_names {
            let sample_path = sound_path.join(file_name).with_extension("wav");
            audio_files.insert(file_name.to_string(), load_context.load(sample_path));
        }

        Ok(PacketAsset {
            source: packet,
            audio_files,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["H", "h"]
    }
}

impl FromWorld for PacketAssetLoader {
    fn from_world(world: &mut World) -> Self {
        let asset_paths = world.resource::<AssetPaths>();

        Self {
            asset_paths: asset_paths.clone(),
        }
    }
}
