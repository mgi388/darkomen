mod decoder;
mod encoder;

use bevy_derive::{Deref, DerefMut};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use glam::{EulerRot, Quat};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

/// A list of keyframes each containing rotation states for body and head.
///
/// Each keyframe represents a pose that can be interpolated between during
/// portrait animations. The rotations are stored as two-byte pairs for pitch,
/// yaw, and roll.
#[derive(Clone, Default, Deref, DerefMut, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Keyframes(pub Vec<Keyframe>);

/// A single keyframe containing body and head rotations.
#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Keyframe {
    /// Body rotation (pitch, yaw, roll).
    pub body_rotation: Rotation,
    /// Head rotation (pitch, yaw, roll).
    pub head_rotation: Rotation,
}

/// A rotation in 3D space stored as pitch, yaw, and roll.
///
/// Each component is stored as two bytes representing a rotation angle. The
/// game uses these for interpolating between keyframes during animations.
#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Rotation {
    /// Pitch rotation (rotation around X axis).
    pub pitch: RotationValue,
    /// Yaw rotation (rotation around Y axis).
    pub yaw: RotationValue,
    /// Roll rotation (rotation around Z axis).
    pub roll: RotationValue,
}

impl Rotation {
    /// Converts the rotation to a quaternion.
    ///
    /// Uses the ZYX Euler rotation order (roll, yaw, pitch).
    #[inline]
    pub fn to_quat(&self) -> Quat {
        let pitch = self.pitch.as_radians();
        let yaw = self.yaw.as_radians();
        let roll = self.roll.as_radians();
        Quat::from_euler(EulerRot::ZYX, roll, yaw, pitch)
    }
}

/// A single rotation value stored as two bytes.
///
/// The two-byte format provides high-precision angle representation:
///
/// - Second byte (bits 0-3): Coarse angle in 22.5° increments.
/// - First byte: Fine adjustment in 22.5°/256 increments.
#[derive(Clone, Copy, Default, Deref, DerefMut, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct RotationValue(pub [u8; 2]);

impl RotationValue {
    /// Creates a new rotation value from two bytes.
    #[inline]
    pub const fn new(bytes: [u8; 2]) -> Self {
        Self(bytes)
    }

    /// Returns the rotation in degrees.
    ///
    /// Formula: (second_byte * 22.5) + (first_byte * 22.5 / 256.0).
    #[inline]
    pub fn as_degrees(self) -> f32 {
        // Second byte contributes 22.5 degrees per unit.
        let second_contribution = (self.0[1] & 0x0F) as f32 * 22.5;
        // First byte contributes 22.5/256 degrees per unit.
        let first_contribution = self.0[0] as f32 * (22.5 / 256.0);

        second_contribution + first_contribution
    }

    /// Returns the rotation in radians.
    #[inline]
    pub fn as_radians(self) -> f32 {
        self.as_degrees().to_radians()
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

    fn roundtrip_test(original_bytes: &[u8], keyframes: &Keyframes) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(keyframes).unwrap();

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
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
            "PORTRAIT",
            "SCRIPT",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "decoded",
            "portrait",
            "keyframes",
        ]
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
            if ext.to_string_lossy().to_uppercase() != "KEY" {
                return;
            }

            let file_name = path.file_name().unwrap().to_string_lossy();

            // Skip 3.KEY and 29.KEY. These appear to be corrupted assets (they
            // contain text like "orc body" and "OrcHead group" in their
            // headers).
            if file_name == "3.KEY" || file_name == "29.KEY" {
                println!("Skipping {:?} (corrupted asset)", file_name);
                return;
            }

            println!("Decoding {:?}", file_name);

            let original_bytes = std::fs::read(path).unwrap();
            println!("  File size: {} bytes", original_bytes.len());

            let file = File::open(path).unwrap();
            let keyframes = match Decoder::new(file).decode() {
                Ok(k) => k,
                Err(e) => {
                    println!("  Error decoding: {}", e);
                    println!("  Skipping file that doesn't conform to expected format");
                    return;
                }
            };

            roundtrip_test(&original_bytes, &keyframes);

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

            // Write RON file.
            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            ron::ser::to_writer_pretty(&mut output_file, &keyframes, Default::default()).unwrap();

            // Write TXT file with radians.
            let txt_path = append_ext("txt", output_dir.join(path.file_name().unwrap()));
            let mut txt_output = String::new();

            txt_output.push_str(&format!("File: {}\n", file_name));
            txt_output.push_str(&format!("Keyframe count: {}\n\n", keyframes.0.len()));

            for (i, keyframe) in keyframes.0.iter().enumerate() {
                txt_output.push_str(&format!("Keyframe {}:\n", i));
                txt_output.push_str(&format!(
                    "  Body rotation:\n    pitch: {} rad ({:02X} {:02X})\n    yaw:   {} rad ({:02X} {:02X})\n    roll:  {} rad ({:02X} {:02X})\n",
                    keyframe.body_rotation.pitch.as_radians(),
                    keyframe.body_rotation.pitch.0[0],
                    keyframe.body_rotation.pitch.0[1],
                    keyframe.body_rotation.yaw.as_radians(),
                    keyframe.body_rotation.yaw.0[0],
                    keyframe.body_rotation.yaw.0[1],
                    keyframe.body_rotation.roll.as_radians(),
                    keyframe.body_rotation.roll.0[0],
                    keyframe.body_rotation.roll.0[1],
                ));
                txt_output.push_str(&format!(
                    "  Head rotation:\n    pitch: {} rad ({:02X} {:02X})\n    yaw:   {} rad ({:02X} {:02X})\n    roll:  {} rad ({:02X} {:02X})\n",
                    keyframe.head_rotation.pitch.as_radians(),
                    keyframe.head_rotation.pitch.0[0],
                    keyframe.head_rotation.pitch.0[1],
                    keyframe.head_rotation.yaw.as_radians(),
                    keyframe.head_rotation.yaw.0[0],
                    keyframe.head_rotation.yaw.0[1],
                    keyframe.head_rotation.roll.as_radians(),
                    keyframe.head_rotation.roll.0[0],
                    keyframe.head_rotation.roll.0[1],
                ));
                txt_output.push('\n');
            }

            std::fs::write(txt_path, txt_output).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }

    #[test]
    fn test_rotation_value() {
        // Test 90 degrees (second byte = 0x04).
        let component = RotationValue::new([0x00, 0x04]);
        assert_eq!(component.as_degrees(), 90.0);
        assert!((component.as_radians() - std::f32::consts::FRAC_PI_2).abs() < 0.001);

        // Test 180 degrees (second byte = 0x08).
        let component = RotationValue::new([0x00, 0x08]);
        assert_eq!(component.as_degrees(), 180.0);
        assert!((component.as_radians() - std::f32::consts::PI).abs() < 0.001);

        // Test mixed bytes.
        let component = RotationValue::new([0x01, 0x04]);
        let expected = 90.0 + (22.5 / 256.0);
        assert!((component.as_degrees() - expected).abs() < 0.001);
    }

    #[test]
    fn test_rotation_to_quat() {
        use glam::EulerRot;

        // Test a rotation with specific pitch, yaw, and roll values.
        let rotation = Rotation {
            pitch: RotationValue::new([0x00, 0x04]), // 90 degrees
            yaw: RotationValue::new([0x00, 0x08]),   // 180 degrees
            roll: RotationValue::new([0x00, 0x02]),  // 45 degrees
        };

        let quat = rotation.to_quat();

        // Verify by reconstructing from expected euler angles.
        let expected_quat = Quat::from_euler(
            EulerRot::ZYX,
            rotation.roll.as_radians(),
            rotation.yaw.as_radians(),
            rotation.pitch.as_radians(),
        );

        // Quaternions should be equal (allowing for floating point error).
        assert!((quat.x - expected_quat.x).abs() < 0.001);
        assert!((quat.y - expected_quat.y).abs() < 0.001);
        assert!((quat.z - expected_quat.z).abs() < 0.001);
        assert!((quat.w - expected_quat.w).abs() < 0.001);
    }
}
