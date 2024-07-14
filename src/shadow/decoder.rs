use super::*;
use std::{
    fmt,
    io::{Error as IoError, Read, Seek},
    mem::size_of,
};

/// The format ID used in all .SHD files.
pub(crate) const FORMAT: &str = "SHAD";

pub(crate) const HEADER_SIZE: usize = 28;
pub(crate) const BLOCK_HEADER_SIZE: usize = 8;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    Invalid(String),
    InvalidFormat(String),
    InvalidOffsetIndex(u32),
    InvalidOffsetsBlockSize(usize, usize),
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
            DecodeError::IoError(e) => write!(f, "IO error: {}", e),
            DecodeError::Invalid(s) => write!(f, "invalid: {}", s),
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {}", s),
            DecodeError::InvalidOffsetIndex(index) => {
                write!(f, "offset index {} is not a multiple of 64", index)
            }
            DecodeError::InvalidOffsetsBlockSize(offset_count, offsets_block_size) => {
                write!(
                    f,
                    "invalid offsets block size {}, should be offset count ({}) x 64",
                    offsets_block_size, offset_count
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

    pub fn decode(&mut self) -> Result<Shadow, DecodeError> {
        let terrain = self.read_terrain()?;

        Ok(Shadow { terrain })
    }

    fn read_terrain(&mut self) -> Result<Terrain, DecodeError> {
        let mut header = vec![0; HEADER_SIZE];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != FORMAT.as_bytes() {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let _size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize; // size, not used
        let width = u32::from_le_bytes(header[8..12].try_into().unwrap());
        let height = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let offset_count = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;
        let uncompressed_block_count =
            u32::from_le_bytes(header[20..24].try_into().unwrap()) as usize;
        let heightmap_block_size = u32::from_le_bytes(header[24..28].try_into().unwrap()) as usize; // size in bytes of heightmap block

        // This check just helps prove that the size of the heightmap chunk
        // also lets us get the uncompressed block count.
        if heightmap_block_size / size_of::<TerrainBlock>() != uncompressed_block_count {
            return Err(DecodeError::Invalid(
                "uncompressed block count and heightmap block size mismatch".to_string(),
            ));
        }

        // Heightmap.
        let heightmap_blocks = self.read_heightmap_blocks(uncompressed_block_count)?;

        // Read offsets.
        let mut buf = vec![0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;
        let offsets_size = u32::from_le_bytes(buf.try_into().unwrap()) as usize;

        if offset_count * 64 != offsets_size {
            return Err(DecodeError::InvalidOffsetsBlockSize(
                offset_count,
                offsets_size,
            ));
        }

        let mut buf = vec![0; offsets_size];
        self.reader.read_exact(&mut buf)?;

        let mut offsets = Vec::with_capacity(offset_count);
        for i in 0..offset_count {
            offsets.push(buf[i * 64..(i + 1) * 64].to_vec());
        }

        Ok(Terrain {
            width,
            height,
            heightmap_blocks,
            offsets,
        })
    }

    fn read_heightmap_blocks(&mut self, count: usize) -> Result<Vec<TerrainBlock>, DecodeError> {
        let mut blocks = Vec::with_capacity(count);
        for _ in 0..count {
            blocks.push(self.read_terrain_block()?);
        }
        Ok(blocks)
    }

    fn read_terrain_block(&mut self) -> Result<TerrainBlock, DecodeError> {
        let mut buf = vec![0; size_of::<TerrainBlock>()];
        self.reader.read_exact(&mut buf)?;

        let min_height = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let offset_index = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if offset_index % 64 != 0 {
            return Err(DecodeError::InvalidOffsetIndex(offset_index));
        }
        let offset_index = offset_index / 64;

        Ok(TerrainBlock {
            min_height,
            offset_index,
        })
    }
}
