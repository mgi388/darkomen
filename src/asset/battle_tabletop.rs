use bevy_app::prelude::*;
use bevy_asset::{io::Reader, prelude::*, AssetLoader, AsyncReadExt, LoadContext};
use bevy_reflect::prelude::*;
use derive_more::derive::{Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::battle::*;

use super::army::*;

pub struct BattleTabletopAssetPlugin;

impl Plugin for BattleTabletopAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ArmyAssetPlugin>() {
            app.add_plugins(ArmyAssetPlugin);
        }

        app.init_asset::<BattleTabletopAsset>()
            .init_asset_loader::<BattleTabletopAssetLoader>()
            .register_asset_reflect::<BattleTabletopAsset>();
    }
}

#[derive(Asset, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct BattleTabletopAsset {
    source: Blueprint,

    pub player_army: Option<Handle<ArmyAsset>>,
    pub enemy_army: Option<Handle<ArmyAsset>>,
}

impl BattleTabletopAsset {
    #[inline(always)]
    pub fn get(&self) -> &Blueprint {
        &self.source
    }

    #[inline(always)]
    pub fn objectives(&self) -> &[Objective] {
        &self.source.objectives
    }

    #[inline(always)]
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.source.obstacles
    }

    #[inline(always)]
    pub fn regions(&self) -> &[Region] {
        &self.source.regions
    }

    #[inline(always)]
    pub fn nodes(&self) -> &[Node] {
        &self.source.nodes
    }
}

#[derive(Clone, Debug, Default)]
pub struct BattleTabletopAssetLoader;

#[derive(Clone, Debug, Default, Deserialize, Reflect, Serialize)]
#[reflect(Debug, Default, Deserialize, Serialize)]
pub struct BattleTabletopAssetLoaderSettings {
    pub load_player_army: bool,
    pub player_army_loader_settings: Option<ArmyAssetLoaderSettings>,
    pub load_enemy_army: bool,
    pub enemy_army_loader_settings: Option<ArmyAssetLoaderSettings>,
}

/// Possible errors that can be produced by [BattleTabletopAssetLoader].
#[non_exhaustive]
#[derive(Debug, Display, Error, From)]
pub enum BattleTabletopAssetLoaderError {
    /// An [IO](std::io) error.
    #[display("could not load asset: {_0}")]
    Io(std::io::Error),
    /// A [DecodeError] error.
    #[display("could not decode battle tabletop: {_0}")]
    DecodeError(DecodeError),
}

impl AssetLoader for BattleTabletopAssetLoader {
    type Asset = BattleTabletopAsset;
    type Settings = BattleTabletopAssetLoaderSettings;
    type Error = BattleTabletopAssetLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a BattleTabletopAssetLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let parent_path = load_context
            .path()
            .parent()
            .expect("parent path should exist")
            .to_path_buf();

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let reader = std::io::Cursor::new(bytes);

        let mut decoder = Decoder::new(reader);

        let btb = decoder.decode()?;

        Ok(BattleTabletopAsset {
            source: btb.clone(),
            player_army: if settings.load_player_army {
                let mut b = load_context.loader();
                if let Some(s) = settings.player_army_loader_settings.clone() {
                    b = b.with_settings(move |settings| {
                        *settings = s.clone();
                    });
                }
                Some(b.load(parent_path.join(btb.player_army).with_extension("ARM")))
            } else {
                None
            },
            enemy_army: if settings.load_enemy_army {
                let mut b = load_context.loader();
                if let Some(s) = settings.enemy_army_loader_settings.clone() {
                    b = b.with_settings(move |settings| {
                        *settings = s.clone();
                    });
                }
                Some(b.load(parent_path.join(btb.enemy_army).with_extension("ARM")))
            } else {
                None
            },
        })
    }

    fn extensions(&self) -> &[&str] {
        &["BTB", "btb"]
    }
}
