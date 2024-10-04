mod decoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::{UVec4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};

/// Dark Omen's format for 3D models.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct M3d {
    pub texture_descriptors: Vec<TextureDescriptor>,
    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct TextureDescriptor {
    /// Path appears to be a directory on the original Dark Omen developer's
    /// machine. It does not seem to be used for anything useful and might best
    /// be treated as an Easter Egg.
    pub path: String,
    /// The name of the texture image file, e.g. "nflgrs01.bmp".
    pub file_name: String,
}

impl TextureDescriptor {
    /// Texture flags embedded in the prefix of the file name, e.g.
    /// `_1WOOD8.bmp`, `_2wtpool.bmp`. The prefix is either: `_1`, `_2`, or no
    /// prefix.
    ///
    /// - `_1` seems like it's possibly just color keying
    /// - `_2` are all water (and jewel) textures, so must possibly to do with
    ///   transparency, translucency or animation.

    pub fn is_color_keyed(&self) -> bool {
        self.file_name.to_ascii_lowercase().starts_with("_1")
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect_value(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct ObjectFlags: u32 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const CUSTOM_TRANSLATION_ENABLED = 1 << 1;
        const UNKNOWN_FLAG_3 = 1 << 2;
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Object {
    pub name: String,
    pub parent_index: i16,
    pub padding: i16,
    pub translation: Vec3,
    pub flags: ObjectFlags,
    pub unknown1: u32,
    pub unknown2: u32,
    pub faces: Vec<Face>,
    pub vertices: Vec<Vertex>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Face {
    pub indices: [u16; 3],
    pub texture_index: u16,
    pub normal: Vec3,
    pub unknown1: u32,
    pub unknown2: u32,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: UVec4,
    pub uv: Vec2,
    pub index: u32,
    pub unknown1: u32,
}
