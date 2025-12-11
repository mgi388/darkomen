mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::{U8Vec2, U8Vec3};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Default, Deserialize, Hash, PartialEq, Serialize))]
    #[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
    pub struct HeadFlags: u8 {
        const NONE = 0;
        /// The character has no mouth model, e.g., because they wear a helmet
        /// (i.e., no BITS texture).
        const NO_MOUTH = 1 << 0;
        /// Hide accessory slot 1 in meet (non-battle) portraits. Slots are
        /// 0-indexed.
        const HIDE_ACCESSORY_1_IN_MEET = 1 << 1;
        /// Hide accessory slot 2 in meet (non-battle) portraits. Slots are
        /// 0-indexed.
        const HIDE_ACCESSORY_2_IN_MEET = 1 << 2;
        /// Hide accessory slot 3 in meet (non-battle) portraits. Slots are
        /// 0-indexed.
        const HIDE_ACCESSORY_3_IN_MEET = 1 << 3;
        /// The character does not have an injured variant (i.e., no HEADI or
        /// BITSI textures).
        const NO_INJURED_VARIANT = 1 << 4;
        /// The character does not have a death variant (i.e., no DEATH
        /// texture).
        const NO_DEATH_VARIANT = 1 << 5;
        const UNKNOWN_HEAD_FLAG_6 = 1 << 6;
        const UNKNOWN_HEAD_FLAG_7 = 1 << 7;
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct HeadsDatabase {
    pub entries: Vec<HeadEntry>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct HeadEntry {
    /// 2-character ASCII identifier for the head (e.g., "KZ", "MB", "GS").
    /// Used to load textures like "{name}_HEAD.BMP", "{name}_BODY.BMP".
    pub name: String,
    /// Feature flags that control which accessories are valid.
    pub flags: HeadFlags,
    pub(crate) unknown1: u8,
    pub(crate) unknown2: u8,
    pub mouth: Mouth,
    pub eyes: Eyes,
    /// Body model. Position values are scaled by 0.05 at runtime to get world
    /// coordinates.
    pub body: ModelSlot,
    /// Head model. Position values are scaled by 0.05 at runtime to get world
    /// coordinates.
    pub head: ModelSlot,
    pub(crate) unknown3: u8,
    pub(crate) unknown4: u8,
    /// Equipment/accessory models (swords, staffs, etc.). 4 slots available.
    /// Position values are scaled by 0.05 at runtime to get world coordinates.
    pub accessories: [ModelSlot; 4],
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct ModelSlot {
    /// Model ID (1-63). 0 means no model in this slot.
    pub model_id: u8,
    /// Position offset [x, y, z] in integer format. Multiply by 0.05 to get
    /// world coordinates.
    pub position: U8Vec3,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Mouth {
    /// The size [width, height] of the mouth. Combine with the position to
    /// determine the mouth rectangle in the head texture.
    pub size: U8Vec2,
    /// The top left position [x, y] to position the mouth in the head texture.
    pub position: U8Vec2,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Eyes {
    /// The size [width, height] of the eyes. Combine with the position to
    /// determine the eyes rectangle in the head texture.
    pub size: U8Vec2,
    /// The top left position [x, y] to position the eyes in the head texture.
    pub position: U8Vec2,
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

    fn roundtrip_test(original_bytes: &[u8], heads: &HeadsDatabase) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(heads).unwrap();

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
    fn test_decode_heads_db() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
            "PORTRAIT",
            "SCRIPT",
            "HEADS.DB",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();
        let file = File::open(d).unwrap();
        let heads = Decoder::new(file).decode().unwrap();

        assert_eq!(heads.entries.len(), 63);
        assert_eq!(heads.entries.first().unwrap().name, "MB");
        assert_eq!(
            heads.entries.first().unwrap().flags,
            HeadFlags::HIDE_ACCESSORY_1_IN_MEET | HeadFlags::HIDE_ACCESSORY_2_IN_MEET
        );
        assert_eq!(heads.entries.first().unwrap().body.model_id, 2);
        assert_eq!(heads.entries.first().unwrap().head.model_id, 13);

        roundtrip_test(&original_bytes, &heads);
    }

    #[test]
    fn test_encode_too_many_entries() {
        let heads = HeadsDatabase {
            entries: vec![HeadEntry::default(); 256],
        };

        let mut encoded_bytes = Vec::new();
        let result = Encoder::new(&mut encoded_bytes).encode(&heads);

        assert!(result.is_err());
        match result {
            Err(EncodeError::TooManyEntries) => (),
            _ => panic!("Expected TooManyEntries error"),
        }
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
            "PORTRAIT",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "portrait", "heads"]
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
            if ext.to_string_lossy().to_uppercase() != "DB" {
                return;
            }
            // Skip BACKUP.DB files because they don't start with the head count
            // so we can't decode them properly.
            if path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .ends_with("BACKUP")
            {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let heads = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &heads);

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

            // Write the complete database.
            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            ron::ser::to_writer_pretty(&mut output_file, &heads, Default::default()).unwrap();

            // Write individual head entries.
            let db_name = path.file_stem().unwrap().to_string_lossy();
            let individual_dir = output_dir.join(db_name.as_ref());
            std::fs::create_dir_all(&individual_dir).unwrap();

            for (index, entry) in heads.entries.iter().enumerate() {
                let individual_path =
                    individual_dir.join(format!("{:02}_{}.ron", index, entry.name));
                let mut individual_file = File::create(individual_path).unwrap();
                ron::ser::to_writer_pretty(&mut individual_file, entry, Default::default())
                    .unwrap();
            }
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
