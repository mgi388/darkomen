use std::io::{Cursor, Read, Write};

use super::BlockError;

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Pcm16Block {
    pub data: Vec<i16>,
}

impl Pcm16Block {
    pub fn from_bytes(bs: &[u8]) -> Result<Self, BlockError> {
        let mut data = Vec::with_capacity(bs.len() / 2);
        let mut buf = Cursor::new(bs);
        for _ in 0..bs.len() / 2 {
            let mut bytes = [0u8; 2];
            buf.read_exact(&mut bytes)?;
            data.push(i16::from_le_bytes(bytes));
        }
        Ok(Self { data })
    }

    pub fn from_int16_slice(data: &[i16]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, BlockError> {
        let mut buf = Vec::with_capacity(self.data.len() * 2);
        for &v in &self.data {
            buf.write_all(&v.to_le_bytes())?;
        }
        Ok(buf)
    }
}
