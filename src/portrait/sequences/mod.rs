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
    /// Delay command - pauses main sequence execution.
    ///
    /// **Timing:** `time` is in arbitrary game ticks (not frames). Dark Omen
    /// likely ran at ~20 FPS, so scale accordingly for your target frame rate.
    /// The delay timer decrements by 1 per game tick.
    ///
    /// **Behavior:** Blocks the main sequence (no new rotation/loop commands
    /// execute) but facial animations continue independently, creating an
    /// apparent "parallel" effect.
    ///
    /// **Implementation:** For Bevy at 64Hz, multiply by ~3.2x (20 FPS → 64 Hz).
    /// Make the scale factor configurable and tune by comparing with game videos.
    ///
    ///   - Byte 1: delay time in game ticks (0-255).
    Delay { time: u8 },

    /// End of sequence marker - prepares sequence for looping.
    ///
    /// Appears before Loop (0x08) command. Connected to mouth/sound animation
    /// timing - removing this can cause mouth animation to end prematurely.
    /// The exact mechanism is unclear from the C code but empirically verified.
    EndSequence,

    /// Rotate to keyframe (standard).
    ///
    /// Interpolates body/head rotation from current pose to target keyframe.
    ///
    /// **Interpolation:** Controls the rotation curve shape. Research suggests
    /// TCB splines (Kochanek-Bartels) with different tension/bias parameters:
    ///   - `0x00`: One curve profile (exact parameters unknown)
    ///   - `0x04`: Different curve profile (exact parameters unknown)
    ///
    /// **Note:** TCB parameters were discovered empirically, not from C code.
    /// For implementation, start with SLERP (spherical linear interpolation)
    /// and add curve variations later if needed.
    ///
    /// **Timing:** Same scale as Delay command (game ticks at ~20 FPS).
    ///
    ///   - Byte 1: interpolation curve type (0x00 or 0x04).
    ///   - Byte 2: animation time in game ticks (0 = instant/no rotation).
    ///   - Byte 3: target keyframe index (references .KEY file).
    RotateToKeyframe {
        interpolation_mode: u8,
        time: u8,
        keyframe_index: u8,
    },

    /// Eyes state command - controls eye open/closed state.
    ///
    /// **Context-dependent behavior:**
    ///   - **In main sequences**: Only takes effect when facial animation is
    ///     active (started by StartSpeaking or MouthAnimation commands).
    ///     Ignored otherwise.
    ///   - **In facial sequences (126.SEQ/127.SEQ)**: Sets eye state that
    ///     persists until the next Eyes command. Each Eyes command executes
    ///     for 1 tick but the state remains until changed.
    ///
    /// **Implementation:** When processing facial animations, set the eye state
    /// and leave it unchanged until the next Eyes command. Don't reset it every
    /// frame.
    ///
    ///   - Byte 1: 0x00 = closed, 0x01 = open.
    Eyes { open: bool },

    /// Mouth state command - sets mouth texture/sprite.
    ///
    /// **Timing:** Each Mouth command in facial sequences (126.SEQ/127.SEQ)
    /// executes for 1 game tick, but the mouth state **persists until the next
    /// Mouth command**. The facial animation is frame-by-frame data where each
    /// command specifies the mouth state, but the state doesn't reset between
    /// frames.
    ///
    /// **State Value Format:** Composed of texture index (low nibble) and flags
    /// (high nibble):
    ///   - **Low nibble (0x0F)**: Mouth texture/sprite index (0-5)
    ///     - 0 = closed
    ///     - 1 = slightly open
    ///     - 2 = more open
    ///   - **High nibble (0xF0)**: Flags or modifiers
    ///     - 0x10 (bit 4): Possibly emphasis or duration modifier
    ///
    /// **Common values:**
    ///   - 0 (0x00) = closed mouth, no flags
    ///   - 1 (0x01) = slightly open, no flags
    ///   - 2 (0x02) = more open, no flags
    ///   - 16 (0x10) = closed with flag set
    ///   - 17 (0x11) = slightly open with flag set (possibly "oow" shape)
    ///
    /// **Implementation:** Extract texture index with `state & 0x0F` and flags
    /// with `state & 0xF0`. Set the mouth state and leave it until the next
    /// Mouth command changes it. At Bevy's 64Hz vs Dark Omen's ~20 FPS, you'll
    /// need to either play at 3x speed or interpolate between states.
    ///
    ///   - Byte 1: encoded mouth frame (high nibble=column, low nibble=row into
    ///     the texture atlas).
    Mouth { encoded_frame: u8 },

    /// End of sequence marker - triggers looping or completion.
    ///
    /// When reached, either:
    ///   - Loops back to start if looping is enabled (flag-controlled)
    ///   - Marks animation as complete and cleans up state
    ///
    /// Found at the end of all sequences. Calls cleanup function which clears
    /// state flags and removes portrait from active list.
    Loop,

    /// Loop with counter - conditional looping based on counter state.
    ///
    /// Similar to Loop (0x08) but includes counter/state tracking. The exact
    /// behavior depends on counter value (stored at portrait state offset 132).
    ///
    /// **Note:** Precise semantics unclear from C code - may involve jump
    /// targets or frame references.
    ///
    ///   - Byte 1: counter high byte or state flag.
    ///   - Byte 2: counter low byte or jump target.
    LoopWithCounter { counter_high: u8, counter_low: u8 },

    /// Start speaking - triggers facial animation AND audio playback.
    ///
    /// **This is the only command that triggers audio.** References a facial
    /// animation sequence (from 126.SEQ for cutscenes, 127.SEQ for battle)
    /// and starts audio playback via DirectSound.
    ///
    /// **Behavior:** Sets audio flags (bits 4-5 at offset 130) and calls audio
    /// start function. Facial animation can loop until audio ends or stop when
    /// sound finishes (flag-controlled).
    ///
    ///   - Byte 1: facial animation sequence index (0-31 for 126.SEQ, 0-15 for
    ///     127.SEQ).
    StartSpeaking { facial_animation_index: u8 },

    /// Mouth animation without audio - silent facial animation.
    ///
    /// **Identical to StartSpeaking (0x0A) except NO audio is triggered.**
    /// Useful for idle animations, background character movement, or when
    /// audio is handled separately.
    ///
    ///   - Byte 1: facial animation sequence index (0-31 for 126.SEQ, 0-15 for
    ///     127.SEQ).
    MouthAnimation { facial_animation_index: u8 },

    /// End mouth animation - stops active facial animation.
    ///
    /// Terminates the currently playing facial animation (started by
    /// StartSpeaking or MouthAnimation). Does not affect audio playback.
    EndMouthAnimation,

    /// Rotate to keyframe (initial) - first command marker.
    ///
    /// **Functionally identical to [`Command::RotateToKeyframe`]** but with a
    /// different opcode (0x13 vs 0x03). In the C code, both opcodes call the
    /// same rotation function - there is no behavioral difference.
    ///
    /// **Purpose:** File format marker. The 0x13 opcode **only appears as the
    /// first command** in a sequence, making it easy to identify sequence
    /// boundaries when parsing raw binary data.
    ///
    /// **Implementation:** Treat identically to RotateToKeyframe but preserve
    /// the distinction for accurate encoding/decoding.
    ///
    ///   - Byte 1: interpolation curve type (0x00 or 0x04).
    ///   - Byte 2: animation time in game ticks.
    ///   - Byte 3: target keyframe index.
    InitialRotateToKeyframe {
        interpolation_mode: u8,
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
            let mut buffer = String::new();
            ron::ser::to_writer_pretty(&mut buffer, &sequences, Default::default()).unwrap();
            std::fs::write(output_path, buffer).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
