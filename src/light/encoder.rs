use std::io::{BufWriter, Write};

use crate::light::decoder::FORMAT;

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {}", e),
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

    pub fn encode(&mut self, lights: &Vec<Light>) -> Result<(), EncodeError> {
        self.write_header(lights)?;
        self.write_lights(lights)?;
        Ok(())
    }

    fn write_header(&mut self, lights: &[Light]) -> Result<(), EncodeError> {
        self.writer.write_all(&FORMAT.to_le_bytes())?;
        self.writer
            .write_all(&(lights.len() as u32).to_le_bytes())?;

        Ok(())
    }

    fn write_lights(&mut self, lights: &Vec<Light>) -> Result<(), EncodeError> {
        for l in lights {
            self.write_light(l)?;
        }

        Ok(())
    }

    fn write_light(&mut self, l: &Light) -> Result<(), EncodeError> {
        self.write_position(&l.position)?;
        self.writer.write_all(&l.flags.bits().to_le_bytes())?;
        self.writer
            .write_all(&((l.attenuation * 1024.) as i32).to_le_bytes())?;
        self.write_color(&l.color)?;

        Ok(())
    }

    fn write_position(&mut self, v: &Vec3) -> Result<(), EncodeError> {
        self.writer
            .write_all(&((v.x * 1024.) as i32).to_le_bytes())?;
        self.writer
            .write_all(&((v.y * 1024.) as i32).to_le_bytes())?;
        self.writer
            .write_all(&((v.z * 1024.) as i32).to_le_bytes())?;
        Ok(())
    }

    fn write_color(&mut self, v: &Vec3) -> Result<(), EncodeError> {
        self.writer
            .write_all(&((v.x * 256.0) as u32).to_le_bytes())?;
        self.writer
            .write_all(&((v.y * 256.0) as u32).to_le_bytes())?;
        self.writer
            .write_all(&((v.z * 256.0) as u32).to_le_bytes())?;
        Ok(())
    }
}
