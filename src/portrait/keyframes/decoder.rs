use core::fmt;
use std::io::{Error as IoError, Read, Seek};

use super::*;

pub(crate) const HEADER_SIZE_BYTES: usize = 1;
pub(crate) const KEYFRAME_SIZE_BYTES: usize = 12;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
}

impl std::error::Error for DecodeError {}

impl From<IoError> for DecodeError {
    fn from(error: IoError) -> Self {
        DecodeError::IoError(error)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::IoError(e) => write!(f, "IO error: {e}"),
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {s}"),
        }
    }
}

pub struct Decoder<R>
where
    R: Read + Seek,
{
    reader: R,
}

impl<R: Read + Seek> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder { reader }
    }

    pub fn decode(&mut self) -> Result<Keyframes, DecodeError> {
        // .KEY files always have a 1-byte header containing the keyframe count.
        // The game may cap this count based on how many it wants to load.

        // Read the header byte (keyframe count).
        let mut header_buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header_buf)?;
        let keyframe_count = header_buf[0];

        let keyframes = self.read_keyframes(keyframe_count)?;

        Ok(Keyframes(keyframes))
    }

    fn read_keyframes(&mut self, keyframe_count: u8) -> Result<Vec<Keyframe>, DecodeError> {
        let mut keyframes = Vec::with_capacity(keyframe_count as usize);

        for _ in 0..keyframe_count {
            let mut buf = [0; KEYFRAME_SIZE_BYTES];
            self.reader.read_exact(&mut buf)?;

            let body_rotation = Rotation {
                pitch: RotationValue::new([buf[0], buf[1]]),
                yaw: RotationValue::new([buf[2], buf[3]]),
                roll: RotationValue::new([buf[4], buf[5]]),
            };

            let head_rotation = Rotation {
                pitch: RotationValue::new([buf[6], buf[7]]),
                yaw: RotationValue::new([buf[8], buf[9]]),
                roll: RotationValue::new([buf[10], buf[11]]),
            };

            keyframes.push(Keyframe {
                body_rotation,
                head_rotation,
            });
        }

        Ok(keyframes)
    }
}
