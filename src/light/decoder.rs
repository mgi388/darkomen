use std::{
    fmt,
    io::{Error as IoError, Read, Seek},
};

use glam::Vec3;

use super::*;

pub(crate) const FORMAT: u32 = 1;

const HEADER_SIZE_BYTES: usize = 8;
const LIGHT_SIZE_BYTES: usize = 32;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
    InvalidLightFlags(u32),
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
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {}", s),
            DecodeError::InvalidLightFlags(v) => write!(f, "invalid light flags: {}", v),
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

    pub fn decode(&mut self) -> Result<Vec<Light>, DecodeError> {
        let light_count = self.decode_header()?;

        let lights = self.read_lights(light_count)?;

        Ok(lights)
    }

    fn decode_header(&mut self) -> Result<usize, DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        if u32::from_le_bytes(buf[0..4].try_into().unwrap()) != FORMAT {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let light_count = u32::from_le_bytes(buf[4..8].try_into().unwrap());

        Ok(light_count as usize)
    }

    fn read_lights(&mut self, light_count: usize) -> Result<Vec<Light>, DecodeError> {
        let mut buf = vec![0; light_count * LIGHT_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let mut lights = Vec::with_capacity(light_count);
        for i in 0..light_count {
            let b = &buf[i * LIGHT_SIZE_BYTES..(i + 1) * LIGHT_SIZE_BYTES];

            let flags_u32 = u32::from_le_bytes(b[12..16].try_into().unwrap());
            let light = Light {
                position: Vec3::new(
                    i32::from_le_bytes(b[0..4].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[4..8].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[8..12].try_into().unwrap()) as f32 / 1024.,
                ),
                flags: LightFlags::from_bits(flags_u32)
                    .ok_or(DecodeError::InvalidLightFlags(flags_u32))?,
                attenuation: i32::from_le_bytes(b[16..20].try_into().unwrap()) as f32 / 1024.,
                color: Vec3::new(
                    u32::from_le_bytes(b[20..24].try_into().unwrap()) as f32 / 256.,
                    u32::from_le_bytes(b[24..28].try_into().unwrap()) as f32 / 256.,
                    u32::from_le_bytes(b[28..32].try_into().unwrap()) as f32 / 256.,
                ),
            };

            lights.push(light);
        }

        Ok(lights)
    }
}
