use std::{collections::HashMap, path::PathBuf};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

pub struct AssetPathsPlugin;

impl Plugin for AssetPathsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetPaths {
            gameflow_path: PathBuf::from("DARKOMEN/GAMEDATA/GAMEFLOW"),
            banners_path: PathBuf::from("DARKOMEN/GRAPHICS/BANNERS"),
            maps_path: PathBuf::from("DARKOMEN/GRAPHICS/MAPS"),
            pictures_path: PathBuf::from("DARKOMEN/GRAPHICS/PICTURES"),
            sprites_path: PathBuf::from("DARKOMEN/GRAPHICS/SPRITES"),
            books_path: PathBuf::from("DARKOMEN/GRAPHICS/BOOKS"),
            movies_path: PathBuf::from("DARKOMEN/MOVIES"),
            sound_path: PathBuf::from("DARKOMEN/SOUND/SOUND"),
            music_script_path: PathBuf::from("DARKOMEN/SOUND/SCRIPT"),
            music_path: PathBuf::from("DARKOMEN/SOUND/MUSIC"),
            sound_effect_packets_path: PathBuf::from("DARKOMEN/SOUND/H"),
        });
        app.register_type::<AssetPaths>();
    }
}

#[derive(Clone, Debug, Reflect, Resource)]
#[reflect(Debug, Resource)]
pub struct AssetPaths {
    pub gameflow_path: PathBuf,
    pub banners_path: PathBuf,
    pub maps_path: PathBuf,
    pub pictures_path: PathBuf,
    pub sprites_path: PathBuf,
    pub books_path: PathBuf,
    pub movies_path: PathBuf,
    pub sound_path: PathBuf,
    pub music_script_path: PathBuf,
    pub music_path: PathBuf,
    pub sound_effect_packets_path: PathBuf,
}

impl Default for AssetPaths {
    fn default() -> Self {
        Self {
            gameflow_path: PathBuf::from("DARKOMEN/GAMEDATA/GAMEFLOW"),
            banners_path: PathBuf::from("DARKOMEN/GRAPHICS/BANNERS"),
            maps_path: PathBuf::from("DARKOMEN/GRAPHICS/MAPS"),
            pictures_path: PathBuf::from("DARKOMEN/GRAPHICS/PICTURES"),
            sprites_path: PathBuf::from("DARKOMEN/GRAPHICS/SPRITES"),
            books_path: PathBuf::from("DARKOMEN/GRAPHICS/BOOKS"),
            movies_path: PathBuf::from("DARKOMEN/MOVIES"),
            sound_path: PathBuf::from("DARKOMEN/SOUND/SOUND"),
            music_script_path: PathBuf::from("DARKOMEN/SOUND/SCRIPT"),
            music_path: PathBuf::from("DARKOMEN/SOUND/MUSIC"),
            sound_effect_packets_path: PathBuf::from("DARKOMEN/SOUND/H"),
        }
    }
}

impl AssetPaths {
    pub fn resolve_path(&self, file_path: &str) -> PathBuf {
        let mut placeholders = HashMap::new();
        placeholders.insert("[BOOKS]", &self.books_path);
        placeholders.insert("[BANNERS]", &self.banners_path);
        placeholders.insert("[GAMEFLOW]", &self.gameflow_path);
        placeholders.insert("[MAPS]", &self.maps_path);
        placeholders.insert("[MOVIES]", &self.movies_path);
        placeholders.insert("[PICTURES]", &self.pictures_path);
        placeholders.insert("[SOUND]", &self.sound_effect_packets_path);

        // Bevy asset paths are meant to be virtual paths, not OS paths, so we
        // need to replace backslashes with forward slashes.
        //
        // See https://github.com/bevyengine/bevy/issues/10511.
        let file_path = file_path.replace("\\", "/");

        let file_path = PathBuf::from(file_path);
        let file_path_str = file_path.to_string_lossy();

        for (placeholder, path) in placeholders {
            if file_path_str.starts_with(placeholder) {
                let replaced_path = file_path_str.replacen(placeholder, &path.to_string_lossy(), 1);
                return PathBuf::from(replaced_path);
            }
        }

        file_path
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    macro_rules! test_resolve_path {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let asset_paths = AssetPaths::default();
                let result = asset_paths.resolve_path($input);
                assert_eq!(result, PathBuf::from($expected));
            }
        };
    }

    test_resolve_path!(
        test_books_path,
        "[BOOKS]/hgban.spr",
        "DARKOMEN/GRAPHICS/BOOKS/hgban.spr"
    );
    test_resolve_path!(
        test_banners_path,
        "[BANNERS]/banner.png",
        "DARKOMEN/GRAPHICS/BANNERS/banner.png"
    );
    test_resolve_path!(
        test_no_placeholder,
        "no_placeholder.txt",
        "no_placeholder.txt"
    );
    test_resolve_path!(
        test_backslashes,
        "[BOOKS]\\hgban.spr",
        "DARKOMEN/GRAPHICS/BOOKS/hgban.spr"
    );
}
