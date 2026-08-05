#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Flags derived from M3D file name prefixes.
    ///
    /// These are the raw bit values from the prefix parsing.
    #[repr(transparent)]
    #[derive(Clone, Copy, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(Default, Deserialize, Hash, PartialEq, Serialize), reflect(opaque))]
    #[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
    pub struct M3dFlags: u32 {
        const NONE = 0;

        /// All textures in the model are rendered with transparency.
        ///
        /// Prefixes `_1`, `_3`, `_7` in model file names.
        const TRANSPARENT = 1 << 0;

        /// All vertex UVs in the model are transformed every frame to create an
        /// animation effect, e.g., used for water models.
        ///
        /// Prefixes `_2`, `_3`, `_6`, `_7` model in file names.
        const UV_TRANSFORM_ANIMATED = 1 << 1;

        /// Prefixes `_4`, `_5`, `_6`, `_7`, `_K` model in file names.
        const UNKNOWN_FLAG_3 = 1 << 2;

        const UNKNOWN_FLAG_4 = 1 << 3;

        /// Prefix `_K` in model file names.
        const UNKNOWN_FLAG_5 = 1 << 4;
    }
}

impl M3dFlags {
    /// Parse M3D file name prefix into flags.
    pub fn from_file_name(file_name: &str) -> Self {
        let prefix_value = parse_prefix_value(file_name);
        Self::from_bits_truncate(prefix_value)
    }
}

bitflags! {
    /// Flags derived from texture/image file name prefixes.
    ///
    /// These control how textures are loaded and processed.
    #[repr(transparent)]
    #[derive(Clone, Copy, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque))]
    #[cfg_attr(feature = "bevy_reflect", reflect(Default, Deserialize, Hash, PartialEq, Serialize))]
    #[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
    pub struct TextureFlags: u32 {
        const NONE = 0;

        /// Apply alpha mapping to make pure black pixels transparent.
        ///
        /// Uses edge detection to preserve interior blacks while removing
        /// border blacks.
        ///
        /// Prefix `_1` in texture file names.
        const BLACK_ALPHA = 1 << 0;

        /// Unknown effect. Seen in textures used for water.
        ///
        /// Prefix `_2` in texture file names.
        const UNKNOWN_FLAG_1 = 1 << 1;
    }
}

impl TextureFlags {
    /// Parse texture file name prefix into flags.
    pub fn from_file_name(file_name: &str) -> Self {
        let prefix_value = parse_prefix_value(file_name);
        Self::from_bits_truncate(prefix_value)
    }
}

/// Parse file name prefix like `_1`, `_7`, `_K` into a numeric value.
///
/// Returns 0 if no underscore prefix or invalid format.
fn parse_prefix_value(file_name: &str) -> u32 {
    let bytes = file_name.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'_' {
        return 0;
    }

    match bytes[1] {
        b'0'..=b'9' => (bytes[1] - b'0') as u32,
        b'A'..=b'Z' => (bytes[1] - 55) as u32, // 'A' = 10
        b'a'..=b'z' => (bytes[1] - 87) as u32, // 'a' = 36
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_flags() {
        let prefixes = vec![
            "_0", "_1", "_2", "_3", "_4", "_5", "_6", "_7", "_8", "_9", "_A", "_B", "_C", "_D",
            "_E", "_F", "_G", "_H", "_I", "_J", "_K", "_L", "_M", "_N", "_O", "_P", "_Q", "_R",
            "_S", "_T", "_U", "_V",
        ];

        println!("\n=== M3D flags ===");
        for prefix in &prefixes {
            let file_name = format!("{}TEST.M3D", prefix);
            let flags = M3dFlags::from_file_name(&file_name);
            let raw_value = parse_prefix_value(&file_name);

            println!(
                "{}: raw={:2} (0b{:05b}) flags={:?}",
                prefix, raw_value, raw_value, flags,
            );
        }

        let prefixes = vec!["_0", "_1", "_2"];

        println!("\n=== Texture flags ===");
        for prefix in &prefixes {
            let file_name = format!("{}TEST.BMP", prefix);
            let flags = TextureFlags::from_file_name(&file_name);
            let raw_value = parse_prefix_value(&file_name);

            println!(
                "{}: raw={:2} (0b{:05b}) flags={:?}",
                prefix, raw_value, raw_value, flags,
            );
        }
    }
}
