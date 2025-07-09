use super::*;
use std::{
    fmt,
    io::{Error as IoError, Read, Seek},
    mem::size_of,
};

/// The format ID used in all .SHD files.
pub(crate) const FORMAT: &str = "SHAD";

pub(crate) const HEADER_SIZE_BYTES: usize = 28;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    Invalid(String),
    InvalidFormat(String),
    InvalidHeightOffsetsIndex(u32),
    InvalidHeightOffsetsSize(usize, usize),
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
            DecodeError::Invalid(s) => write!(f, "invalid: {s}"),
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {s}"),
            DecodeError::InvalidHeightOffsetsIndex(index) => {
                write!(f, "height offsets index {index} is not a multiple of 64")
            }
            DecodeError::InvalidHeightOffsetsSize(offset_count, height_offsets_size_bytes) => {
                write!(
                    f,
                    "invalid height offsets size {height_offsets_size_bytes}, should be offset count ({offset_count}) x 64",
                )
            }
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

    pub fn decode(&mut self) -> Result<Lightmap, DecodeError> {
        let lightmap = self.read_lightmap()?;

        Ok(lightmap)
    }

    fn read_lightmap(&mut self) -> Result<Lightmap, DecodeError> {
        let mut header = vec![0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != FORMAT.as_bytes() {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let _total_size_bytes = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize; // total block size in bytes, not used
        let width = u32::from_le_bytes(header[8..12].try_into().unwrap());
        let height = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let offset_count = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;
        let block_count = u32::from_le_bytes(header[20..24].try_into().unwrap()) as usize;
        let blocks_size_bytes = u32::from_le_bytes(header[24..28].try_into().unwrap()) as usize; // size in bytes of blocks

        // This check just helps prove that the size of the blocks chunk also
        // lets us get the block count.
        if blocks_size_bytes / size_of::<LightmapBlock>() != block_count {
            return Err(DecodeError::Invalid(
                "block count and blocks size mismatch".to_string(),
            ));
        }

        // Read blocks.
        let blocks = self.read_blocks(block_count)?;

        // Read height offsets.
        let mut buf = vec![0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;
        let height_offsets_size_bytes = u32::from_le_bytes(buf.try_into().unwrap()) as usize;

        if offset_count * 64 != height_offsets_size_bytes {
            return Err(DecodeError::InvalidHeightOffsetsSize(
                offset_count,
                height_offsets_size_bytes,
            ));
        }

        let mut buf = vec![0; height_offsets_size_bytes];
        self.reader.read_exact(&mut buf)?;

        let mut height_offsets = Vec::with_capacity(offset_count);
        for i in 0..offset_count {
            height_offsets.push(buf[i * 64..(i + 1) * 64].to_vec());
        }

        Ok(Lightmap {
            width,
            height,
            blocks,
            height_offsets,
        })
    }

    fn read_blocks(&mut self, count: usize) -> Result<Vec<LightmapBlock>, DecodeError> {
        let mut blocks = Vec::with_capacity(count);
        for _ in 0..count {
            blocks.push(self.read_block()?);
        }
        Ok(blocks)
    }

    fn read_block(&mut self) -> Result<LightmapBlock, DecodeError> {
        let mut buf = vec![0; size_of::<LightmapBlock>()];
        self.reader.read_exact(&mut buf)?;

        let base_height = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let height_offsets_index = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if height_offsets_index % 64 != 0 {
            return Err(DecodeError::InvalidHeightOffsetsIndex(height_offsets_index));
        }
        let height_offsets_index = height_offsets_index / 64;

        Ok(LightmapBlock {
            base_height,
            height_offsets_index,
        })
    }
}
