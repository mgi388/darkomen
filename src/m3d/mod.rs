mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::{UVec4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub use decoder::*;
pub use encoder::*;

/// Dark Omen's format for 3D models.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct M3d {
    header: Header,
    pub texture_descriptors: Vec<M3dTextureDescriptor>,
    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub(crate) struct Header {
    _magic: u32,
    _version: u32,
    _crc: u32,
    _not_crc: u32,
    texture_count: u16,
    object_count: u16,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct M3dTextureDescriptor {
    /// Path appears to be a directory on the original Dark Omen developer's
    /// machine. It does not seem to be used for anything useful and might best
    /// be treated as an Easter Egg.
    pub path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    path_remainder: Vec<u8>,
    /// The name of the texture image file, e.g., "nflgrs01.bmp".
    pub file_name: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    file_name_remainder: Vec<u8>,
}

/// Texture flags embedded in the prefix of the file name, e.g., `_1WOOD8.bmp`,
/// `_2wtpool.bmp`. The prefix is either: `_1`, `_2`, or no prefix.
///
/// - `_1` seems like it's possibly just color keying.
/// - `_2` are all water (and jewel) textures, so must possibly to do with
///   transparency, translucency or animation.
impl M3dTextureDescriptor {
    /// Returns `true` if the texture descriptor indicates that the texture is
    /// color keyed.
    pub fn is_color_keyed(&self) -> bool {
        self.file_name.to_ascii_lowercase().starts_with("_1")
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
    #[cfg_attr(feature = "bevy_reflect", reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct ObjectFlags: u32 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const CUSTOM_TRANSLATION_ENABLED = 1 << 1;
        const UNKNOWN_FLAG_3 = 1 << 2;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Object {
    pub name: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    pub name_remainder: Vec<u8>,
    pub parent_index: i16,
    pub padding: i16,
    pub translation: Vec3,
    pub flags: ObjectFlags,
    pub unknown1: u32,
    pub unknown2: u32,
    pub faces: Vec<Face>,
    pub vertices: Vec<Vertex>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Face {
    pub indices: [u16; 3],
    pub texture_index: u16,
    pub normal: Vec3,
    pub unknown1: u32,
    pub unknown2: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
#[cfg_attr(
    feature = "bevy_reflect",
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: UVec4,
    pub uv: Vec2,
    pub index: u32,
    pub unknown1: u32,
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

    fn roundtrip_test(original_bytes: &[u8], m: &M3d) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(m).unwrap();

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
    fn test_decode_b1_01_base() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "BASE.M3D",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let m3d = Decoder::new(file).decode().unwrap();

        assert_eq!(m3d.texture_descriptors.len(), 37);
        assert_eq!(m3d.objects.len(), 4);

        roundtrip_test(&original_bytes, &m3d);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [std::env::var("DARKOMEN_PATH").unwrap().as_str(), "DARKOMEN"]
            .iter()
            .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "m3ds"]
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
            if !(ext.to_string_lossy().to_uppercase() == "M3D"
                || ext.to_string_lossy().to_uppercase() == "M3X")
            {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let m3d = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &m3d);

            // Check the header values. Not sure if these are correct field
            // names or what they are used for in-game, but their values are
            // consistent across all M3D files.
            assert_eq!(m3d.header._magic, 908342784);
            assert_eq!(m3d.header._version, 1);
            assert_eq!(m3d.header._crc, 0);
            assert_eq!(m3d.header._not_crc, 4294967295);

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
            ron::ser::to_writer_pretty(&mut output_file, &m3d, Default::default()).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
