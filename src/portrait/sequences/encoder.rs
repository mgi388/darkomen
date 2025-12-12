use core::fmt;
use std::io::{BufWriter, Write};

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {e}"),
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

    pub fn encode(&mut self, sequences: &Sequences) -> Result<(), EncodeError> {
        self.writer.write_all(&[sequences.0.len() as u8])?;

        for sequence in &sequences.0 {
            self.write_sequence(sequence)?;
        }
        Ok(())
    }

    fn write_sequence(&mut self, sequence: &Sequence) -> Result<(), EncodeError> {
        self.writer.write_all(&[sequence.commands.len() as u8])?;

        for command in &sequence.commands {
            self.write_command(command)?;
        }

        Ok(())
    }

    fn write_command(&mut self, command: &Command) -> Result<(), EncodeError> {
        let bytes = match command {
            Command::Delay { time } => [0x01, *time, 0x00, 0x00],
            Command::EndSequence => [0x02, 0x00, 0x00, 0x00],
            Command::RotateToKeyframe {
                interpolation,
                time,
                keyframe_index,
            } => [0x03, *interpolation, *time, *keyframe_index],
            Command::Eyes { open } => [0x05, if *open { 0x01 } else { 0x00 }, 0x00, 0x00],
            Command::Mouth { state } => [0x06, *state, 0x00, 0x00],
            Command::Loop => [0x08, 0x00, 0x00, 0x00],
            Command::LoopWithCounter {
                counter_high,
                counter_low,
            } => [0x09, *counter_high, *counter_low, 0x00],
            Command::StartTalking {
                facial_animation_index,
            } => [0x0A, *facial_animation_index, 0x00, 0x00],
            Command::MouthAnimation {
                facial_animation_index,
            } => [0x0B, *facial_animation_index, 0x00, 0x00],
            Command::EndMouthAnimation => [0x0C, 0x00, 0x00, 0x00],
            Command::InitialRotateToKeyframe {
                interpolation,
                time,
                keyframe_index,
            } => [0x13, *interpolation, *time, *keyframe_index],
            Command::Unknown { opcode, data } => [*opcode, data[0], data[1], data[2]],
        };

        self.writer.write_all(&bytes)?;
        Ok(())
    }
}
