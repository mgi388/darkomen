use super::*;
use decoder::{FORMAT, HEADER_SIZE_BYTES};
use std::{
    io::{BufWriter, Write},
    mem::size_of,
};

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
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

    pub fn encode(&mut self, l: &Lightmap) -> Result<(), EncodeError> {
        self.write_lightmap(l)?;
        Ok(())
    }

    fn write_lightmap(&mut self, l: &Lightmap) -> Result<(), EncodeError> {
        // Write the header.
        let blocks_size_bytes = l.blocks.len() * size_of::<LightmapBlock>();
        let height_offsets_size_bytes = size_of::<u32>() + (l.height_offsets.len() * 64);
        let total_size_bytes = HEADER_SIZE_BYTES - (2 * size_of::<u32>())
            + blocks_size_bytes
            + height_offsets_size_bytes;
        self.writer.write_all(FORMAT.as_bytes())?;
        self.writer
            .write_all(&(total_size_bytes as u32).to_le_bytes())?;
        self.writer.write_all(&l.width.to_le_bytes())?;
        self.writer.write_all(&l.height.to_le_bytes())?;
        self.writer
            .write_all(&(l.height_offsets.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(l.blocks.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(blocks_size_bytes as u32).to_le_bytes())?;

        // Write blocks.
        self.write_blocks(&l.blocks)?;

        // Write height offsets.
        let height_offsets_size_bytes = l.height_offsets.len() * 64;
        self.writer
            .write_all(&(height_offsets_size_bytes as u32).to_le_bytes())?;
        for offsets in &l.height_offsets {
            self.writer.write_all(offsets)?;
        }

        Ok(())
    }

    fn write_blocks(&mut self, blocks: &Vec<LightmapBlock>) -> Result<(), EncodeError> {
        for block in blocks {
            let height_offsets_index = block.height_offsets_index * 64;
            self.writer.write_all(&block.base_height.to_le_bytes())?;
            self.writer.write_all(&height_offsets_index.to_le_bytes())?;
        }

        Ok(())
    }
}
