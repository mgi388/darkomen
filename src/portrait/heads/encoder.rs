use std::{
    fmt,
    io::{BufWriter, Write},
};

use encoding_rs::WINDOWS_1252;

use super::*;

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
    StringTooLong,
    TooManyEntries,
}

impl std::error::Error for EncodeError {}

impl From<std::io::Error> for EncodeError {
    fn from(err: std::io::Error) -> Self {
        EncodeError::IoError(err)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {e}"),
            EncodeError::InvalidString => write!(f, "invalid string"),
            EncodeError::StringTooLong => write!(f, "string too long"),
            EncodeError::TooManyEntries => write!(f, "too many entries"),
        }
    }
}

pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder {
            writer: BufWriter::new(writer),
        }
    }

    pub fn encode(&mut self, heads_database: &HeadsDatabase) -> Result<(), EncodeError> {
        self.write_header(heads_database)?;
        self.write_entries(&heads_database.entries)?;
        Ok(())
    }

    fn write_header(&mut self, heads_database: &HeadsDatabase) -> Result<(), EncodeError> {
        if heads_database.entries.len() > 255 {
            return Err(EncodeError::TooManyEntries);
        }
        self.writer
            .write_all(&[heads_database.entries.len() as u8])?;
        Ok(())
    }

    fn write_entries(&mut self, entries: &[HeadEntry]) -> Result<(), EncodeError> {
        for entry in entries {
            let (windows_1252_bytes, _, had_errors) = WINDOWS_1252.encode(&entry.name);
            if had_errors {
                return Err(EncodeError::InvalidString);
            }

            if windows_1252_bytes.len() > 2 {
                return Err(EncodeError::StringTooLong);
            }

            let mut name_bytes = [0u8; 2];
            name_bytes[..windows_1252_bytes.len()].copy_from_slice(&windows_1252_bytes);
            self.writer.write_all(&name_bytes)?;

            self.writer.write_all(&[entry.flags.bits()])?;
            self.writer.write_all(&[entry.battle_sequences_id])?;
            self.writer.write_all(&[entry.meet_sequences_id])?;

            self.write_mouth(&entry.mouth)?;
            self.write_eyes(&entry.eyes)?;

            self.write_model_slot(&entry.body)?;
            self.write_model_slot(&entry.head)?;

            self.writer.write_all(&[entry.battle_keyframes_id])?;
            self.writer.write_all(&[entry.meet_keyframes_id])?;

            self.write_model_slot(&entry.neck)?;

            for accessory in &entry.accessories {
                self.write_model_slot(accessory)?;
            }

            self.write_model_slot(&entry.head_accessory)?;
        }
        Ok(())
    }

    fn write_model_slot(&mut self, model_slot: &ModelSlot) -> Result<(), EncodeError> {
        self.writer.write_all(&[model_slot.model_id])?;
        self.writer
            .write_all(&model_slot.translation.x.to_le_bytes())?;
        self.writer
            .write_all(&model_slot.translation.y.to_le_bytes())?;
        self.writer
            .write_all(&model_slot.translation.z.to_le_bytes())?;
        Ok(())
    }

    fn write_mouth(&mut self, mouth: &Option<Mouth>) -> Result<(), EncodeError> {
        let mouth = mouth.as_ref().unwrap_or(&Mouth {
            size: U8Vec2::ZERO,
            position: U8Vec2::ZERO,
        });
        self.writer.write_all(&mouth.size.x.to_le_bytes())?;
        self.writer.write_all(&mouth.size.y.to_le_bytes())?;
        self.writer.write_all(&mouth.position.x.to_le_bytes())?;
        self.writer.write_all(&mouth.position.y.to_le_bytes())?;
        Ok(())
    }

    fn write_eyes(&mut self, eyes: &Option<Eyes>) -> Result<(), EncodeError> {
        let eyes = eyes.as_ref().unwrap_or(&Eyes {
            size: U8Vec2::ZERO,
            position: U8Vec2::ZERO,
        });
        self.writer.write_all(&eyes.size.x.to_le_bytes())?;
        self.writer.write_all(&eyes.size.y.to_le_bytes())?;
        self.writer.write_all(&eyes.position.x.to_le_bytes())?;
        self.writer.write_all(&eyes.position.y.to_le_bytes())?;
        Ok(())
    }
}
