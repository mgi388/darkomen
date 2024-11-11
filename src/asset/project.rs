use std::{marker::PhantomData, path::PathBuf};

use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, AsyncReadExt, LoadContext};
use bevy_ecs::prelude::*;
use bevy_pbr::prelude::*;
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::project::*;

use super::{
    battle_tabletop::*, light::*, lightmap::*, m3d::M3dAsset, paths::*, sound::music_script::*,
};

#[derive(Debug, Default)]
pub struct ProjectPlugin<MaterialT: Material + std::fmt::Debug>(PhantomData<MaterialT>);

impl<MaterialT: Material + std::fmt::Debug> Plugin for ProjectPlugin<MaterialT> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPathsPlugin>() {
            app.add_plugins(AssetPathsPlugin);
        }
        if !app.is_plugin_added::<MusicScriptAssetPlugin>() {
            app.add_plugins(MusicScriptAssetPlugin);
        }
        if !app.is_plugin_added::<LightAssetPlugin>() {
            app.add_plugins(LightAssetPlugin);
        }
        if !app.is_plugin_added::<LightmapAssetPlugin>() {
            app.add_plugins(LightmapAssetPlugin);
        }
        if !app.is_plugin_added::<BattleTabletopAssetPlugin>() {
            app.add_plugins(BattleTabletopAssetPlugin);
        }

        app.init_asset::<ProjectAsset<MaterialT>>()
            .init_asset_loader::<ProjectAssetLoader<MaterialT>>()
            .register_asset_reflect::<ProjectAsset<MaterialT>>();
    }
}

#[derive(Asset, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct ProjectAsset<MaterialT: Material + std::fmt::Debug> {
    source: Project,
    /// The ID of the project, e.g. `B1_01`. This is the same as the directory
    /// that the project file is in.
    pub id: String,
    /// The base model. This is always the chunked M3X version.
    pub base_model: Handle<M3dAsset<MaterialT>>,
    /// The water model, if any. This is always the chunked M3X version.
    pub water_model: Option<Handle<M3dAsset<MaterialT>>>,
    /// A list of furniture models required for instances in the project.
    pub furniture_models: Vec<Handle<M3dAsset<MaterialT>>>,
    /// The music script to play for the project.
    pub music_script: Handle<MusicScriptAsset>,
    /// The lights for the project.
    pub lights: Handle<LightsAsset>,
    /// The lightmap for the project.
    pub lightmap: Handle<LightmapAsset>,
    /// The battle tabletop for the project.
    pub battle_tabletop: Handle<BattleTabletopAsset>,
}

impl<MaterialT: Material + std::fmt::Debug> ProjectAsset<MaterialT> {
    #[inline(always)]
    pub fn get(&self) -> &Project {
        &self.source
    }

    #[inline(always)]
    pub fn instances(&self) -> &[Instance] {
        &self.source.instances
    }

    #[inline(always)]
    pub fn terrain(&self) -> &Terrain {
        &self.source.terrain
    }

    #[inline(always)]
    pub fn attributes(&self) -> &Attributes {
        &self.source.attributes
    }

    pub fn position_track(&self) -> Option<&Track> {
        self.source
            .tracks
            .iter()
            .enumerate()
            .find(|(i, _track)| *i == 0)
            .map(|(_, track)| track)
    }

    pub fn look_at_track(&self) -> Option<&Track> {
        self.source
            .tracks
            .iter()
            .enumerate()
            .find(|(i, _track)| *i == 1)
            .map(|(_, track)| track)
    }
}

#[derive(Clone, Debug)]
pub struct ProjectAssetLoader<MaterialT: Material + std::fmt::Debug> {
    _phantom: PhantomData<MaterialT>,

    asset_paths: AssetPaths,
}

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
pub struct ProjectAssetLoaderSettings {
    pub music_script_path: PathBuf,
    pub battle_tabletop_loader_settings: Option<BattleTabletopAssetLoaderSettings>,
}

/// Possible errors that can be produced by [`ProjectAssetLoader`].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum ProjectAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode project: {_0}")]
    DecodeError(DecodeError),
}

impl<MaterialT: Material + std::fmt::Debug> AssetLoader for ProjectAssetLoader<MaterialT> {
    type Asset = ProjectAsset<MaterialT>;
    type Settings = ProjectAssetLoaderSettings;
    type Error = ProjectAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a ProjectAssetLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let music_script_path = if settings.music_script_path.to_string_lossy().is_empty() {
            self.asset_paths.music_script_path.clone()
        } else {
            settings.music_script_path.clone()
        };

        let parent_path = load_context
            .path()
            .parent()
            .expect("parent path should exist")
            .to_path_buf();
        let id = parent_path
            .file_name()
            .expect("file name should exist")
            .to_string_lossy()
            .to_string();

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = std::io::Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let project = decoder.decode()?;

        Ok(ProjectAsset {
            source: project.clone(),
            id: id.clone(),
            base_model: load_context.load(parent_path.join(project.get_base_m3x_model_file_name())),
            water_model: project
                .get_water_m3x_model_file_name()
                .as_ref()
                .map(|file_name| load_context.load(parent_path.join(file_name))),
            furniture_models: project
                .furniture_model_file_names
                .iter()
                .map(|file_name| load_context.load(parent_path.join(file_name)))
                .collect(),
            music_script: load_context.load(music_script_path.join(project.music_script_file_name)),
            lights: load_context.load(parent_path.join(&id).with_extension("LIT")),
            lightmap: load_context.load(parent_path.join(&id).with_extension("SHD")),
            battle_tabletop: {
                let mut b = load_context.loader();
                if let Some(ref s) = settings.battle_tabletop_loader_settings {
                    let s = *s;
                    b = b.with_settings(move |settings| {
                        *settings = s;
                    });
                }
                b.load(parent_path.join(&id).with_extension("BTB"))
            },
        })
    }

    fn extensions(&self) -> &[&str] {
        &["PRJ", "prj"]
    }
}

impl<MaterialT: Material + std::fmt::Debug> FromWorld for ProjectAssetLoader<MaterialT> {
    fn from_world(world: &mut World) -> Self {
        let asset_paths = world.resource::<AssetPaths>();

        Self {
            _phantom: PhantomData,
            asset_paths: asset_paths.clone(),
        }
    }
}
