use std::path::PathBuf;

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, LoadContext};
use bevy_ecs::prelude::*;
use bevy_kira_audio::AudioSource;
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{asset::paths::*, sound::script::*};

use super::sad::StereoAudioAssetPlugin;

pub struct MusicScriptAssetPlugin;

impl Plugin for MusicScriptAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPathsPlugin>() {
            app.add_plugins(AssetPathsPlugin);
        }
        if !app.is_plugin_added::<StereoAudioAssetPlugin>() {
            app.add_plugins(StereoAudioAssetPlugin);
        }

        app.init_asset::<MusicScriptAsset>()
            .init_asset_loader::<MusicScriptAssetLoader>()
            .register_asset_reflect::<MusicScriptAsset>();
    }
}

#[derive(Asset, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct MusicScriptAsset {
    script: Script,

    #[reflect(ignore)]
    pub samples: IndexMap<SampleId, Handle<AudioSource>>,
}

impl MusicScriptAsset {
    /// The state the script starts in.
    pub fn start_state(&self) -> Option<StateId> {
        self.script.start_state.clone()
    }

    /// The pattern the script starts in.
    pub fn start_pattern(&self) -> Option<PatternId> {
        self.script.start_pattern.clone()
    }

    /// The number of states present in the script.
    pub fn state_count(&self) -> usize {
        self.script.states.len()
    }

    /// Get the state keys present in the script.
    pub fn state_keys(&self) -> impl Iterator<Item = &StateId> {
        self.script.states.keys()
    }

    /// The number of samples present in the script.
    pub fn sample_count(&self) -> usize {
        self.script.samples.len()
    }

    /// The number of patterns present in the script.
    pub fn pattern_count(&self) -> usize {
        self.script.patterns.len()
    }

    /// Get the pattern with the given ID.
    pub fn get_pattern(&self, id: &PatternId) -> Option<&Pattern> {
        self.script.patterns.get(id)
    }
}

#[derive(Clone)]
pub struct MusicScriptAssetLoader {
    asset_paths: AssetPaths,
}

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
pub struct MusicScriptAssetLoaderSettings {
    pub music_path: PathBuf,
}

/// Possible errors that can be produced by [`MusicScriptAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum MusicScriptAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode script: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for MusicScriptAssetLoader {
    type Asset = MusicScriptAsset;
    type Settings = MusicScriptAssetLoaderSettings;
    type Error = MusicScriptAssetLoaderError;
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

        let script = decoder.decode()?;

        let music_path = if settings.music_path.to_string_lossy().is_empty() {
            self.asset_paths.music_path.clone()
        } else {
            settings.music_path.clone()
        };

        let mut samples = IndexMap::new();
        for (id, file_name) in script.samples.iter() {
            let sample_path = music_path.join(file_name).with_extension("SAD");
            samples.insert(id.clone(), load_context.load(sample_path));
        }

        Ok(MusicScriptAsset { script, samples })
    }

    fn extensions(&self) -> &[&str] {
        &["FSM", "fsm"]
    }
}

impl FromWorld for MusicScriptAssetLoader {
    fn from_world(world: &mut World) -> Self {
        let asset_paths = world.resource::<AssetPaths>();

        Self {
            asset_paths: asset_paths.clone(),
        }
    }
}
