use core::fmt;
use std::io::{Error as IoError, Read, Seek};

use super::*;

pub(crate) const COMMAND_SIZE_BYTES: usize = 4;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
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
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {s}"),
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

    pub fn decode(&mut self) -> Result<Sequences, DecodeError> {
        // Read the first byte which indicates the total number of animation
        // sequences in the file.
        let mut sequence_count = [0u8; 1];
        self.reader.read_exact(&mut sequence_count)?;
        let total_sequences = sequence_count[0] as usize;

        let mut sequences = Vec::with_capacity(total_sequences);

        for _ in 0..total_sequences {
            // Read the command count for this sequence.
            let mut command_count = [0u8; 1];
            match self.reader.read_exact(&mut command_count) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Reached end of file before reading all sequences.
                    break;
                }
                Err(e) => return Err(e.into()),
            }

            let num_commands = command_count[0] as usize;
            let mut commands = Vec::with_capacity(num_commands);

            // Read all commands for this sequence.
            for _ in 0..num_commands {
                let mut buf = [0; COMMAND_SIZE_BYTES];
                self.reader.read_exact(&mut buf)?;
                let command = Self::parse_command(&buf);
                commands.push(command);
            }

            sequences.push(Sequence { commands });
        }

        Ok(Sequences(sequences))
    }

    fn parse_command(buf: &[u8; 4]) -> Command {
        let opcode = buf[0];
        let byte1 = buf[1];
        let byte2 = buf[2];
        let byte3 = buf[3];

        match opcode {
            0x01 => Command::Delay { time: byte1 },
            0x02 => Command::EndSequence,
            0x03 => Command::RotateToKeyframe {
                interpolation: byte1,
                time: byte2,
                keyframe_index: byte3,
            },
            0x05 => Command::Eyes {
                open: byte1 == 0x01,
            },
            0x06 => Command::Mouth { state: byte1 },
            0x08 => Command::Loop,
            0x09 => Command::LoopWithCounter {
                counter_high: byte1,
                counter_low: byte2,
            },
            0x0A => Command::StartTalking {
                facial_animation_index: byte1,
            },
            0x0B => Command::MouthAnimation {
                facial_animation_index: byte1,
            },
            0x0C => Command::EndMouthAnimation,
            0x13 => Command::InitialRotateToKeyframe {
                interpolation: byte1,
                time: byte2,
                keyframe_index: byte3,
            },
            _ => Command::Unknown {
                opcode,
                data: [byte1, byte2, byte3],
            },
        }
    }
}
