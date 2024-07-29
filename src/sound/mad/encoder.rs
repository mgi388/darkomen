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
            EncodeError::IoError(e) => write!(f, "IO error: {}", e),
            EncodeError::BlockError(e) => write!(f, "block error: {}", e),
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

    pub fn encode(&mut self, a: &MonoAudio) -> Result<(), EncodeError> {
        for i in 0..a.blocks.len() {
            if let Block::AdpcmBlock(b) = &a.blocks[i] {
                self.writer.write_all(&b.sample.to_le_bytes())?;
                self.writer.write_all(&b.index.to_le_bytes())?;
                self.writer.write_all(&b.data.clone())?;

                // Some MAD streams don't have a trailing PCM block, so if we
                // are at the last block and it was an ADPCM block, we are done
                // encoding.
                if i == a.blocks.len() - 1 {
                    return Ok(());
                }
            };
        }

        self.writer.write_all(&a.sample99.to_le_bytes())?;
        self.writer.write_all(&a.index99.to_le_bytes())?;

        let Some(last_block) = a.blocks.last() else {
            return Err(EncodeError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "last block is empty",
            )));
        };

        let bytes = last_block.to_bytes()?;
        self.writer.write_all(&bytes)?;

        Ok(())
    }
}
