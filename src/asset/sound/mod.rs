use bevy_app::prelude::*;

use crate::asset::sound::{
    mad::MonoAudioAssetPlugin, music_script::MusicScriptAssetPlugin, sad::StereoAudioAssetPlugin,
    sound_effect::SoundEffectAssetPlugin,
};

pub mod mad;
pub mod music_script;
pub mod sad;
pub mod sound_effect;

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
        if !app.is_plugin_added::<SoundEffectAssetPlugin>() {
            app.add_plugins(SoundEffectAssetPlugin);
        }
    }
}
