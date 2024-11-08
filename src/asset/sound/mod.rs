use bevy_app::prelude::*;

use crate::asset::sound::{
    mad::MonoAudioAssetPlugin, music_script::MusicScriptAssetPlugin, sad::StereoAudioAssetPlugin,
};

pub mod mad;
pub mod music_script;
pub mod sad;

pub struct SoundAssetPlugin;

impl Plugin for SoundAssetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MonoAudioAssetPlugin>() {
            app.add_plugins(MonoAudioAssetPlugin);
        }
        if !app.is_plugin_added::<StereoAudioAssetPlugin>() {
            app.add_plugins(StereoAudioAssetPlugin);
        }
        if !app.is_plugin_added::<MusicScriptAssetPlugin>() {
            app.add_plugins(MusicScriptAssetPlugin);
        }
    }
}
