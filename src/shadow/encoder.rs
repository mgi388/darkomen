use super::*;
use decoder::{BLOCK_HEADER_SIZE, FORMAT, HEADER_SIZE};
use encoding_rs::WINDOWS_1252;
use std::{
    ffi::CString,
    io::{BufWriter, Write},
    mem::size_of,
};

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
    HeightmapBlockCountMismatch,
}

impl std::error::Error for EncodeError {}

impl From<std::io::Error> for EncodeError {
    fn from(err: std::io::Error) -> Self {
        EncodeError::IoError(err)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {}", e),
            EncodeError::InvalidString => write!(f, "invalid string"),
            EncodeError::HeightmapBlockCountMismatch => write!(f, "heightmap block count mismatch"),
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

    pub fn encode(&mut self, s: &Shadow) -> Result<(), EncodeError> {
        self.write_terrain(&s.terrain)?;
        Ok(())
    }

    fn write_terrain(&mut self, t: &Terrain) -> Result<(), EncodeError> {
        // Write the header.
        let heightmap_block_size = t.heightmap_blocks.len() * size_of::<TerrainBlock>();
        let offsets_block_size = size_of::<u32>() + (t.offsets.len() * 64);
        let block_size =
            HEADER_SIZE - BLOCK_HEADER_SIZE + heightmap_block_size + offsets_block_size;
        self.write_string(FORMAT)?;
        self.writer.write_all(&(block_size as u32).to_le_bytes())?;
        self.writer.write_all(&t.width.to_le_bytes())?;
        self.writer.write_all(&t.height.to_le_bytes())?;
        self.writer
            .write_all(&(t.offsets.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(t.heightmap_blocks.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(heightmap_block_size as u32).to_le_bytes())?;

        // Write the terrain data.
        self.write_heightmap_blocks(&t.heightmap_blocks)?;

        // Write the offsets.
        let offsets_block_size = t.offsets.len() * 64;
        self.writer
            .write_all(&(offsets_block_size as u32).to_le_bytes())?;
        for offset in &t.offsets {
            self.writer.write_all(offset)?;
        }

        Ok(())
    }

    fn write_heightmap_blocks(&mut self, blocks: &Vec<TerrainBlock>) -> Result<(), EncodeError> {
        for block in blocks {
            let offset_index = block.offset_index * 64;
            self.writer.write_all(&block.min_height.to_le_bytes())?;
            self.writer.write_all(&offset_index.to_le_bytes())?;
        }

        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<(), EncodeError> {
        let c_string = self.make_c_string(s)?;
        let bytes = c_string.as_bytes();

        self.writer.write_all(bytes)?;

        Ok(())
    }

    fn make_c_string(&mut self, s: &str) -> Result<CString, EncodeError> {
        let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(s);
        let c_string = CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
        Ok(c_string)
    }
}
