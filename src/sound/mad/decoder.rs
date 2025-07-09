use super::*;
use crate::sound::audio::{adpcm::AdpcmBlock, pcm::Pcm16Block, BlockError};
use std::{
    fmt,
    io::{self, Read, Seek},
};

#[derive(Debug)]
pub enum DecodeError {
    IoError(io::Error),
    BlockError(BlockError),
}

impl std::error::Error for DecodeError {}

impl From<io::Error> for DecodeError {
    fn from(error: io::Error) -> Self {
        DecodeError::IoError(error)
    }
}

impl From<BlockError> for DecodeError {
    fn from(err: BlockError) -> DecodeError {
        DecodeError::BlockError(err)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::IoError(e) => write!(f, "IO error: {e}"),
            DecodeError::BlockError(e) => write!(f, "block error: {e}"),
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

    pub fn decode(&mut self) -> Result<MonoAudio, DecodeError> {
        let mut blocks = Vec::new();
        let mut sample99 = 0;
        let mut index99 = 0;

        let mut buf = [0u8; 4];

        loop {
            let n = self.reader.read(&mut buf)?;
            if n == 0 {
                // Some MAD streams don't have a trailing PCM block, so if we
                // encounter EOF, we are done and return the decoded stream.
                return Ok(MonoAudio {
                    blocks,
                    sample99,
                    index99,
                });
            }
            if n != 4 {
                return Err(DecodeError::IoError(io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!(
                        "could not read mono sample and index data: read {n} byte(s), expected 4",
                    ),
                )));
            }

            let sample = i16::from_le_bytes([buf[0], buf[1]]);
            let index = i16::from_le_bytes([buf[2], buf[3]]);

            if index == 99 {
                sample99 = sample;
                index99 = index;
                break;
            }

            const SIZE_BYTES: usize = 1020;
            let mut buf = vec![0u8; SIZE_BYTES];
            let n = self.reader.read(&mut buf)?;
            if n != SIZE_BYTES {
                return Err(DecodeError::IoError(io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!(
                        "could not read mono ADPCM data: read {n} byte(s), expected {SIZE_BYTES}",
                    ),
                )));
            }

            blocks.push(Block::AdpcmBlock(AdpcmBlock::new(sample, index, buf)));
        }

        // Read remaining bytes.
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;

        blocks.push(Block::Pcm16Block(Pcm16Block::from_bytes(&buf)?));

        Ok(MonoAudio {
            blocks,
            sample99,
            index99,
        })
    }
}
