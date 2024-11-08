use bevy_app::prelude::*;

use crate::asset::sound::{mad::MonoAudioAssetPlugin, sad::StereoAudioAssetPlugin};

pub mod mad;
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
    }
}
