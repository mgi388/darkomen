use std::{
    array::TryFromSliceError,
    fmt,
    io::{Error as IoError, Read, Seek},
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use glam::{U8Vec2, U8Vec3};

use super::*;

pub(crate) const HEADER_SIZE_BYTES: usize = 1;
pub(crate) const ENTRY_SIZE_BYTES: usize = 39;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
    TryFromSliceError(TryFromSliceError),
    InvalidHeadFlags(u8),
}

impl std::error::Error for DecodeError {}

impl From<IoError> for DecodeError {
    fn from(error: IoError) -> Self {
        DecodeError::IoError(error)
    }
}

impl From<std::array::TryFromSliceError> for DecodeError {
    fn from(error: TryFromSliceError) -> Self {
        DecodeError::TryFromSliceError(error)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::IoError(e) => write!(f, "IO error: {e}"),
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {s}"),
            DecodeError::TryFromSliceError(e) => {
                write!(f, "could not convert slice to array: {e}")
            }
            DecodeError::InvalidHeadFlags(v) => write!(f, "invalid head flags: {v}"),
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

    pub fn decode(&mut self) -> Result<HeadsDatabase, DecodeError> {
        let entry_count = self.decode_header()?;
        let entries = self.read_entries(entry_count)?;

        Ok(HeadsDatabase { entries })
    }

    fn decode_header(&mut self) -> Result<u8, DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    fn read_entries(&mut self, entry_count: u8) -> Result<Vec<HeadEntry>, DecodeError> {
        let mut entries = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let mut buf = [0; ENTRY_SIZE_BYTES];
            self.reader.read_exact(&mut buf)?;

            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(&buf[0..2]);
            let mut name = String::new();
            decoder.read_to_string(&mut name)?;

            let flags_u8 = buf[2];
            let flags =
                HeadFlags::from_bits(flags_u8).ok_or(DecodeError::InvalidHeadFlags(flags_u8))?;

            let battle_sequences_id = buf[3];
            let meet_sequences_id = buf[4];

            let mouth = Self::read_mouth(&buf[5..9])?;
            let eyes = Self::read_eyes(&buf[9..13])?;

            let body = Self::read_model_slot(&buf[13..17])?;
            let head = Self::read_model_slot(&buf[17..21])?;

            let battle_keyframes_id = buf[21];
            let meet_keyframes_id = buf[22];

            let neck = Self::read_model_slot(&buf[23..27])?;

            let accessory_0 = Self::read_model_slot(&buf[27..31])?;
            let accessory_1 = Self::read_model_slot(&buf[31..35])?;

            let helmet_accessory = Self::read_model_slot(&buf[35..39])?;

            entries.push(HeadEntry {
                name,
                flags,
                battle_sequences_id,
                meet_sequences_id,
                mouth,
                eyes,
                body,
                head,
                battle_keyframes_id,
                meet_keyframes_id,
                neck,
                accessories: [accessory_0, accessory_1],
                head_accessory: helmet_accessory,
            });
        }

        Ok(entries)
    }

    fn read_model_slot(buf: &[u8]) -> Result<ModelSlot, DecodeError> {
        Ok(ModelSlot {
            model_id: buf[0],
            position: U8Vec3::new(buf[1], buf[2], buf[3]),
        })
    }

    fn read_mouth(buf: &[u8]) -> Result<Option<Mouth>, DecodeError> {
        let mouth = Mouth {
            size: U8Vec2::new(buf[0], buf[1]),
            position: U8Vec2::new(buf[2], buf[3]),
        };
        if mouth.size == U8Vec2::ZERO && mouth.position == U8Vec2::ZERO {
            return Ok(None);
        }
        Ok(Some(mouth))
    }

    fn read_eyes(buf: &[u8]) -> Result<Option<Eyes>, DecodeError> {
        let eyes = Eyes {
            size: U8Vec2::new(buf[0], buf[1]),
            position: U8Vec2::new(buf[2], buf[3]),
        };
        if eyes.size == U8Vec2::ZERO && eyes.position == U8Vec2::ZERO {
            return Ok(None);
        }
        Ok(Some(eyes))
    }
}
