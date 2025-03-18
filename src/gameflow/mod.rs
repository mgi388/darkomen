mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Deserialize, Serialize)
)]
pub struct Gameflow {
    /// The paths that the gameflow follows.
    pub paths: Vec<Path>,
    pub(crate) unknown1: u32,
    /// Always 40. Possibly an animation duration.
    pub(crate) unknown2: u16,
    pub(crate) unknown3: u16,
    /// Notes is probably a relic from the gameflow editor. In `CH1_ALL.DOT.ron`
    /// it looks like the first note is truncated, so this field probably
    /// suffers the same nul-termination issue seen in other game files.
    pub(crate) notes: Vec<String>,
    /// The name of the map file that this gameflow is associated with. This is
    /// not used in the original game, but is probably used by the gameflow
    /// editor.
    pub map_file_name: String,
    pub(crate) unknown4: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Deserialize, Serialize)
)]
pub struct Path {
    /// The control points used to make a curve that represents the path.
    pub control_points: Vec<Point>,
    /// Always 5.
    pub unknown1: i32,
    /// Distance in pixels between interpolated points along the path's curve.
    ///
    /// Used by the curve generation algorithm to determine rendering
    /// granularity.
    ///
    /// Always 10 in the original game.
    pub curve_point_spacing: i32,
    /// Always 0.
    pub unknown3: i32,
    /// Always 1.
    pub unknown4: i32,
    /// Optional index of the previous path in the gameflow. Set to -1 if there
    /// is no previous path.
    ///
    /// This is used to link paths together in the gameflow. The first path in
    /// the gameflow has a previous path index of -1. The last path in the
    /// gameflow has a next path index of -1.
    ///
    /// This doesn't seem to be used in the original game, but is probably used
    /// by the gameflow editor.
    pub previous_path_index: i32,
    /// Optional index of the next path in the gameflow. Set to -1 if there is
    /// no next path.
    ///
    /// This is used to link paths together in the gameflow. The first path in
    /// the gameflow has a previous path index of -1. The last path in the
    /// gameflow has a next path index of -1.
    ///
    /// This doesn't seem to be used in the original game, but is probably used
    /// by the gameflow editor.
    pub next_path_index: i32,
    /// Always 0.
    pub unknown7: i32,
    pub unknown8: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Deserialize, Serialize)
)]
pub struct Point {
    /// The x-coordinate of the point.
    pub x: u32,
    /// The y-coordinate of the point.
    pub y: u32,
    /// Sometimes 1, usually 0. Each gameflow file has one point with a value of
    /// 1 for this field.
    pub(crate) unknown1: u32,
    /// Always 0 in the game files.
    pub(crate) unknown2: u32,
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        path::{Path, PathBuf},
    };

    use pretty_assertions::assert_eq;

    use super::*;

    fn roundtrip_test(original_bytes: &[u8], gameflow: &Gameflow) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(gameflow).unwrap();

        let original_bytes = original_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let encoded_bytes = encoded_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(original_bytes, encoded_bytes);
    }

    #[test]
    fn test_decode_ch1_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "GAMEFLOW",
            "CH1_ALL.DOT",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();
        let file = File::open(d).unwrap();
        let gameflow = Decoder::new(file).decode().unwrap();

        assert_eq!(gameflow.paths.len(), 11);
        assert_eq!(gameflow.unknown1, 103);
        assert_eq!(gameflow.unknown2, 40);
        assert_eq!(gameflow.unknown3, 6);
        assert_eq!(
            gameflow.notes,
            vec![
                "_allz.dot]".to_string(),
                "\\SrcCode\\public\\MP_Dots\\editor\\dots\\M1_ENG.bmp".to_string()
            ]
        );
        assert_eq!(gameflow.map_file_name, "M1_ENG.bmp".to_string());
        assert_eq!(gameflow.paths.first().unwrap().control_points.len(), 5);
        assert_eq!(
            gameflow
                .paths
                .first()
                .unwrap()
                .control_points
                .first()
                .unwrap()
                .x,
            89
        );
        assert_eq!(
            gameflow
                .paths
                .first()
                .unwrap()
                .control_points
                .first()
                .unwrap()
                .y,
            71
        );

        roundtrip_test(&original_bytes, &gameflow);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "GAMEFLOW",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "gameflows"]
            .iter()
            .collect();

        std::fs::create_dir_all(&root_output_dir).unwrap();

        fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) {
            println!("Reading dir {:?}", dir.display());

            let mut paths = std::fs::read_dir(dir)
                .unwrap()
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()
                .unwrap();

            paths.sort();

            for path in paths {
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }

        visit_dirs(&d, &mut |path| {
            let Some(ext) = path.extension() else {
                return;
            };
            if ext.to_string_lossy().to_uppercase() != "DOT" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();
            let file = File::open(path).unwrap();
            let gameflow = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &gameflow);

            let parent_dir = path
                .components()
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .skip(1) // skip the file name
                .take_while(|c| c.as_os_str() != "DARKOMEN")
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .collect::<PathBuf>();
            let output_dir = root_output_dir.join(parent_dir);
            std::fs::create_dir_all(&output_dir).unwrap();

            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            ron::ser::to_writer_pretty(&mut output_file, &gameflow, Default::default()).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
