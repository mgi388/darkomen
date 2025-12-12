mod decoder;
mod encoder;

use bevy_derive::{Deref, DerefMut};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

/// A list of animation sequences.
///
/// Each sequence is a list of commands that control keyframe playback, facial
/// animations (eyes/mouth), and sound synchronization for portrait animations.
#[derive(Clone, Default, Deref, DerefMut, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Sequences(pub Vec<Sequence>);

/// A single animation sequence composed of commands.
#[derive(Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Default, Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub struct Sequence {
    pub commands: Vec<Command>,
}

/// A command in an animation sequence.
///
/// Commands control various aspects of portrait animation including:
///
/// - Keyframe interpolation (rotation).
/// - Facial animation (eyes/mouth).
/// - Sound synchronization.
/// - Timing and delays.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Deserialize, Serialize)
)]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub enum Command {
    /// Delay command - introduces a parallel execution delay.
    ///
    /// The delay is relative to the previous keyframe rotation command. If
    /// delay time exceeds rotation time, the segment is extended.
    ///
    ///   - Byte 1: delay time (same scale as rotation commands).
    Delay { time: u8 },

    /// End of sequence marker - appears before Loop (0x08) command.
    ///
    /// Connected to mouth/sound animation timing. Removing this can cause mouth
    /// animation to end much earlier.
    EndSequence,

    /// Rotate to keyframe (standard).
    ///
    /// Interpolates body/head rotation to the specified keyframe.
    ///
    ///   - Byte 1: interpolation control (0x00 or 0x04, affects curve
    ///     behavior).
    ///   - Byte 2: animation time/acceleration (0 = no rotation).
    ///   - Byte 3: target keyframe index.
    RotateToKeyframe {
        interpolation: u8,
        time: u8,
        keyframe_index: u8,
    },

    /// Eyes state command.
    ///
    /// Controls whether eyes are open or closed.
    ///
    ///   - Byte 1: 0x00 = closed, 0x01 = open.
    ///
    /// Note: May be ignored if no mouth animation command (0x0A) is present.
    Eyes { open: bool },

    /// Mouth animation command (used in 126.SEQ).
    ///
    /// Controls mouth movement for facial animation.
    ///
    ///   - Byte 1: mouth state (e.g., 0x11 = oow, 0x06/0x02/0x10 = various
    ///     states).
    Mouth { state: u8 },

    /// End of animation set marker - causes the animation set to loop.
    ///
    /// Found at the end of animation sets. If the animation was started, it
    /// will loop back to the beginning.
    Loop,

    /// Loop with frame counter.
    ///
    /// Similar to Loop (0x08) but includes counter/state tracking.
    ///
    ///   - Byte 1: loop counter (high byte).
    ///   - Byte 2: loop counter (low byte) or frame to jump to.
    LoopWithCounter { counter_high: u8, counter_low: u8 },

    /// Start talking - triggers mouth animation and audio.
    ///
    /// Uses mouth animation sequence (126.SEQ or 127.SEQ in battle) with audio
    /// playback.
    ///
    ///   - Byte 1: facial animation sequence index to use.
    ///
    /// May be overridden to loop while animation plays or stop when sound ends.
    StartTalking { facial_animation_index: u8 },

    /// Mouth animation without audio.
    ///
    /// Similar to StartTalking (0x0A) but does not trigger audio playback.
    ///
    ///   - Byte 1: facial animation sequence index to use.
    MouthAnimation { facial_animation_index: u8 },

    /// End mouth animation.
    ///
    /// Stops the currently playing facial/mouth animation.
    EndMouthAnimation,

    /// Rotate to keyframe (initial) - same as 0x03 but only appears first.
    ///
    /// Identical to [`Command::RotateToKeyframe`] but used as the first command
    /// in a sequence. The 0x13 opcode only occurs as the first command.
    InitialRotateToKeyframe {
        interpolation: u8,
        time: u8,
        keyframe_index: u8,
    },

    /// Unknown command with raw bytes for unknown opcodes.
    Unknown { opcode: u8, data: [u8; 3] },
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

    fn roundtrip_test(original_bytes: &[u8], sequences: &Sequences) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(sequences).unwrap();

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
            "sequences",
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
            if ext.to_string_lossy().to_uppercase() != "SEQ" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let sequences = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &sequences);

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
            ron::ser::to_writer_pretty(&mut output_file, &sequences, Default::default()).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
