use super::*;
use crate::sound::audio::{adpcm::AdpcmBlock, pcm::Pcm16Block};
use std::{
    fmt,
    io::{self, Read, Seek},
};

#[derive(Debug)]
pub enum DecodeError {
    IoError(io::Error),
}

impl std::error::Error for DecodeError {}

impl From<io::Error> for DecodeError {
    fn from(error: io::Error) -> Self {
        DecodeError::IoError(error)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::IoError(e) => write!(f, "IO error: {e}"),
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

    pub fn decode(&mut self) -> Result<StereoAudio, DecodeError> {
        let mut left_blocks = Vec::new();
        let mut right_blocks = Vec::new();
        let left_sample99;
        let left_index99;
        let right_sample99;
        let right_index99;

        let mut buf = [0u8; 8];

        loop {
            let n = self.reader.read(&mut buf)?;
            if n != 8 {
                return Err(DecodeError::IoError(io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!(
                        "could not read stereo sample and index data: read {n} byte(s), expected 8",
                    ),
                )));
            }

            let left_sample = i16::from_le_bytes([buf[0], buf[1]]);
            let left_index = i16::from_le_bytes([buf[2], buf[3]]);
            let right_sample = i16::from_le_bytes([buf[4], buf[5]]);
            let right_index = i16::from_le_bytes([buf[6], buf[7]]);

            if left_index == 99 && right_index == 99 {
                left_sample99 = left_sample;
                left_index99 = left_index;
                right_sample99 = right_sample;
                right_index99 = right_index;
                break;
            }

            const SIZE_BYTES: usize = 1016;
            let mut buf = vec![0u8; SIZE_BYTES];
            let n = self.reader.read(&mut buf)?;
            if n != SIZE_BYTES {
                return Err(DecodeError::IoError(io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!(
                        "could not read stereo ADPCM data: read {n} byte(s), expected {SIZE_BYTES}",
                    ),
                )));
            }

            let mut left_data = vec![0u8; SIZE_BYTES / 2];
            let mut right_data = vec![0u8; SIZE_BYTES / 2];

            for i in 0..SIZE_BYTES / 8 {
                for j in 0..4 {
                    left_data[i * 4 + j] = buf[i * 8 + j];
                }
                for j in 4..8 {
                    right_data[i * 4 + j - 4] = buf[i * 8 + j];
                }
            }

            left_blocks.push(Block::AdpcmBlock(AdpcmBlock::new(
                left_sample,
                left_index,
                left_data,
            )));
            right_blocks.push(Block::AdpcmBlock(AdpcmBlock::new(
                right_sample,
                right_index,
                right_data,
            )));
        }

        // Read remaining bytes.
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;

        let mut left_buf = Vec::with_capacity(buf.len() / 4);
        let mut right_buf = Vec::with_capacity(buf.len() / 4);

        for i in 0..buf.len() / 4 {
            let left_sample = i16::from_le_bytes([buf[i * 4], buf[i * 4 + 1]]);
            let right_sample = i16::from_le_bytes([buf[i * 4 + 2], buf[i * 4 + 3]]);
            left_buf.push(left_sample);
            right_buf.push(right_sample);
        }

        left_blocks.push(Block::Pcm16Block(Pcm16Block::from_int16_slice(&left_buf)));
        right_blocks.push(Block::Pcm16Block(Pcm16Block::from_int16_slice(&right_buf)));

        Ok(StereoAudio {
            left_blocks,
            right_blocks,
            left_sample99,
            left_index99,
            right_sample99,
            right_index99,
        })
    }
}
