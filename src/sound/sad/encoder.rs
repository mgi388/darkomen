use super::*;
use crate::sound::audio::BlockError;
use std::io::{BufWriter, Write};

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    BlockError(BlockError),
}

impl std::error::Error for EncodeError {}

impl From<std::io::Error> for EncodeError {
    fn from(err: std::io::Error) -> Self {
        EncodeError::IoError(err)
    }
}

impl From<BlockError> for EncodeError {
    fn from(err: BlockError) -> EncodeError {
        EncodeError::BlockError(err)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {e}"),
            EncodeError::BlockError(e) => write!(f, "block error: {e}"),
        }
    }
}

#[derive(Debug)]
pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder {
            writer: BufWriter::new(writer),
        }
    }

    pub fn encode(&mut self, a: &StereoAudio) -> Result<(), EncodeError> {
        for i in 0..a.left_blocks.len() - 1 {
            let left_block = match &a.left_blocks[i] {
                Block::AdpcmBlock(b) => b,
                _ => {
                    return Err(EncodeError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("left block at position {i} is not an ADPCM block"),
                    )))
                }
            };
            self.writer.write_all(&left_block.sample.to_le_bytes())?;
            self.writer.write_all(&left_block.index.to_le_bytes())?;
            let right_block = match &a.right_blocks[i] {
                Block::AdpcmBlock(b) => b,
                _ => {
                    return Err(EncodeError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("right block at position {i} is not an ADPCM block"),
                    )))
                }
            };
            self.writer.write_all(&right_block.sample.to_le_bytes())?;
            self.writer.write_all(&right_block.index.to_le_bytes())?;

            let left_data = left_block.data.clone();
            let right_data = right_block.data.clone();

            for j in (0..left_data.len()).step_by(4) {
                for k in 0..4 {
                    self.writer.write_all(&[left_data[j + k]])?;
                }
                for k in 0..4 {
                    self.writer.write_all(&[right_data[j + k]])?;
                }
            }
        }

        self.writer.write_all(&a.left_sample99.to_le_bytes())?;
        self.writer.write_all(&a.left_index99.to_le_bytes())?;
        self.writer.write_all(&a.right_sample99.to_le_bytes())?;
        self.writer.write_all(&a.right_index99.to_le_bytes())?;

        let Some(last_left_block) = a.left_blocks.last() else {
            return Err(EncodeError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "last left block is empty",
            )));
        };
        let Some(last_right_block) = a.right_blocks.last() else {
            return Err(EncodeError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "last right block is empty",
            )));
        };

        let left_bytes = last_left_block.to_bytes()?;
        let right_bytes = last_right_block.to_bytes()?;

        for i in (0..left_bytes.len()).step_by(2) {
            self.writer.write_all(&[left_bytes[i], left_bytes[i + 1]])?;
            self.writer
                .write_all(&[right_bytes[i], right_bytes[i + 1]])?;
        }

        Ok(())
    }
}
