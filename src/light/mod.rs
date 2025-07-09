mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::Vec3;
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Deserialize, Serialize)
)]
pub struct Light {
    pub position: Vec3,
    pub flags: LightFlags,
    pub attenuation: f32,
    pub color: Vec3,
}

impl Light {
    /// Returns `true` if the light is a directional light.
    pub fn is_directional_light(&self) -> bool {
        self.flags.contains(LightFlags::DIRECTIONAL)
    }

    /// Returns `true` if the light is a point light. There is no explicit
    /// "point light" flag. Instead, if a light is not a directional light and
    /// not a true point light, it is a point light.
    pub fn is_point_light(&self) -> bool {
        !(self.is_directional_light() || self.is_true_point())
    }

    /// Returns `true` if the light is a true point light.
    pub fn is_true_point(&self) -> bool {
        self.flags.contains(LightFlags::TRUE_POINT)
    }

    /// Returns `true` if the light has shadows enabled.
    pub fn is_shadows_enabled(&self) -> bool {
        self.flags.contains(LightFlags::SHADOWS)
    }

    pub fn is_light(&self) -> bool {
        self.flags.contains(LightFlags::LIGHT)
    }

    pub fn is_not_light(&self) -> bool {
        !self.flags.contains(LightFlags::LIGHT)
    }

    pub fn is_furniture(&self) -> bool {
        self.flags.contains(LightFlags::FURNITURE)
    }

    pub fn is_terrain(&self) -> bool {
        self.flags.contains(LightFlags::TERRAIN)
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Deserialize, Hash, PartialEq, Serialize))]
    pub struct LightFlags: u32 {
        const NONE = 0;
        /// Without this flag, the light is labeled as "no shadows".
        const SHADOWS = 1 << 0;
        /// Without this flag, the light is labeled as "no light".
        const LIGHT = 1 << 1;
        const DIRECTIONAL = 1 << 2;
        const TRUE_POINT = 1 << 3;
        const FURNITURE = 1 << 4;
        const TERRAIN = 1 << 5;
        const UNKNOWN_LIGHT_FLAG_4 = 1 << 6;
        const UNKNOWN_LIGHT_FLAG_5 = 1 << 7;
        const UNKNOWN_LIGHT_FLAG_6 = 1 << 8;
        const UNKNOWN_LIGHT_FLAG_7 = 1 << 9;
        const UNKNOWN_LIGHT_FLAG_8 = 1 << 10;
        const UNKNOWN_LIGHT_FLAG_9 = 1 << 11;
        const UNKNOWN_LIGHT_FLAG_10 = 1 << 12;
        const UNKNOWN_LIGHT_FLAG_11 = 1 << 13;
        const UNKNOWN_LIGHT_FLAG_12 = 1 << 14;
        const UNKNOWN_LIGHT_FLAG_13 = 1 << 15;
    }
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

    fn roundtrip_test(original_bytes: &[u8], lights: &Vec<Light>) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(lights).unwrap();

        let original_bytes = original_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{b:02X}"))
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
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(original_bytes, encoded_bytes);
    }

    #[test]
    fn test_decode_b1_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "B1_01.LIT",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let lights = Decoder::new(file).decode().unwrap();

        assert_eq!(lights.len(), 3);

        roundtrip_test(&original_bytes, &lights);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "lights"]
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
            if ext.to_string_lossy().to_uppercase() != "LIT" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let lights = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &lights);

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
            ron::ser::to_writer_pretty(&mut output_file, &lights, Default::default()).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
