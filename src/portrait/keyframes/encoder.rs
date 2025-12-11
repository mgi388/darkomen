use core::fmt;
use std::io::{BufWriter, Write};

use super::*;

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
}

impl std::error::Error for EncodeError {}

impl From<std::io::Error> for EncodeError {
    fn from(err: std::io::Error) -> Self {
        EncodeError::IoError(err)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder {
            writer: BufWriter::new(writer),
        }
    }

    pub fn encode(&mut self, keyframes: &Keyframes) -> Result<(), EncodeError> {
        self.write_header(keyframes)?;
        self.write_keyframes(&keyframes.0)?;
        Ok(())
    }

    fn write_header(&mut self, keyframes: &Keyframes) -> Result<(), EncodeError> {
        self.writer.write_all(&[keyframes.0.len() as u8])?;
        Ok(())
    }

    fn write_keyframes(&mut self, keyframes: &[Keyframe]) -> Result<(), EncodeError> {
        for keyframe in keyframes {
            self.write_rotation(&keyframe.body_rotation)?;
            self.write_rotation(&keyframe.head_rotation)?;
        }
        Ok(())
    }

    fn write_rotation(&mut self, rotation: &Rotation) -> Result<(), EncodeError> {
        self.writer.write_all(&rotation.pitch.0)?;
        self.writer.write_all(&rotation.yaw.0)?;
        self.writer.write_all(&rotation.roll.0)?;
        Ok(())
    }
}
