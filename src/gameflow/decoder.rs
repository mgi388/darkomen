use std::{
    array::TryFromSliceError,
    fmt,
    io::{Error as IoError, Read, Seek},
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use super::*;

pub(crate) const FORMAT: &[u8; 4] = b"TODW";
pub(crate) const HEADER_SIZE_BYTES: usize = 16;
pub(crate) const FOOTER_SIZE_BYTES: usize = 152;
pub(crate) const NOTES_SIZE_BYTES: usize = 80;
pub(crate) const MAP_FILE_NAME_SIZE_BYTES: usize = 40;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
    TryFromSliceError(TryFromSliceError),
}

impl std::error::Error for DecodeError {}

impl From<IoError> for DecodeError {
    fn from(error: IoError) -> Self {
        DecodeError::IoError(error)
    }
}

impl From<std::array::TryFromSliceError> for DecodeError {
    fn from(error: TryFromSliceError) -> Self {
        DecodeError::TryFromSliceError(error)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::IoError(e) => write!(f, "IO error: {e}"),
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {s}"),
            DecodeError::TryFromSliceError(e) => {
                write!(f, "could not convert slice to array: {e}")
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

    pub fn decode(&mut self) -> Result<Gameflow, DecodeError> {
        let (unknown1, unknown2, unknown3, path_count) = self.decode_header()?;
        let paths = self.read_paths(path_count)?;
        let (notes, map_file_name, unknown4) = self.read_footer()?;

        Ok(Gameflow {
            paths,
            unknown1,
            unknown2,
            unknown3,
            notes,
            map_file_name,
            unknown4,
        })
    }

    fn decode_header(&mut self) -> Result<(u32, u16, u16, u32), DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..4] != FORMAT {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let unknown1 = u32::from_le_bytes(buf[4..8].try_into()?);
        let unknown2 = u16::from_le_bytes(buf[8..10].try_into()?);
        let unknown3 = u16::from_le_bytes(buf[10..12].try_into()?);
        let path_count = u32::from_le_bytes(buf[12..16].try_into()?);

        Ok((unknown1, unknown2, unknown3, path_count))
    }

    fn read_paths(&mut self, path_count: u32) -> Result<Vec<Path>, DecodeError> {
        let mut paths = Vec::with_capacity(path_count as usize);

        for _ in 0..path_count {
            let mut buf = [0; 4];
            self.reader.read_exact(&mut buf)?;
            let control_point_count = u32::from_le_bytes(buf);

            let mut control_points = Vec::with_capacity(control_point_count as usize);
            for _ in 0..control_point_count {
                let mut buf = [0; 16];
                self.reader.read_exact(&mut buf)?;
                control_points.push(Point {
                    x: u32::from_le_bytes(buf[0..4].try_into()?),
                    y: u32::from_le_bytes(buf[4..8].try_into()?),
                    unknown1: u32::from_le_bytes(buf[8..12].try_into()?),
                    unknown2: u32::from_le_bytes(buf[12..16].try_into()?),
                });
            }

            let mut tail_buf = [0; 44];
            self.reader.read_exact(&mut tail_buf)?;
            let unknown1 = i32::from_le_bytes(tail_buf[0..4].try_into()?);
            let curve_point_spacing = i32::from_le_bytes(tail_buf[4..8].try_into()?);
            let unknown3 = i32::from_le_bytes(tail_buf[8..12].try_into()?);
            let unknown4 = i32::from_le_bytes(tail_buf[12..16].try_into()?);
            let previous_path_index = i32::from_le_bytes(tail_buf[16..20].try_into()?);
            let next_path_index = i32::from_le_bytes(tail_buf[20..24].try_into()?);
            let unknown7 = i32::from_le_bytes(tail_buf[24..28].try_into()?);
            // The last 16 bytes of the path seem to be read and ignored.
            let unknown8 = tail_buf[28..44].to_vec();

            paths.push(Path {
                control_points,
                unknown1,
                curve_point_spacing,
                unknown3,
                unknown4,
                previous_path_index,
                next_path_index,
                unknown7,
                unknown8,
            });
        }

        Ok(paths)
    }

    fn read_footer(&mut self) -> Result<(Vec<String>, String, Vec<u8>), DecodeError> {
        let mut buf = [0; FOOTER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let notes = self.read_notes(&buf[0..NOTES_SIZE_BYTES])?;
        let map_filename =
            self.read_string(&buf[NOTES_SIZE_BYTES..NOTES_SIZE_BYTES + MAP_FILE_NAME_SIZE_BYTES])?;
        let unknown = buf[NOTES_SIZE_BYTES + MAP_FILE_NAME_SIZE_BYTES..].to_vec();

        Ok((notes, map_filename, unknown))
    }

    fn read_notes(&mut self, buf: &[u8]) -> Result<Vec<String>, DecodeError> {
        let mut notes = Vec::new();
        let mut start = 0;
        for (i, &byte) in buf.iter().enumerate() {
            if byte == 0 {
                if start < i {
                    let mut decoder = DecodeReaderBytesBuilder::new()
                        .encoding(Some(WINDOWS_1252))
                        .build(&buf[start..i]);
                    let mut dest = String::new();
                    decoder.read_to_string(&mut dest)?;
                    if !dest.is_empty() {
                        notes.push(dest);
                    }
                }
                start = i + 1;
            }
        }
        // Handle case where there's no final null terminator.
        if start < buf.len() {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(&buf[start..]);
            let mut dest = String::new();
            decoder.read_to_string(&mut dest)?;
            if !dest.is_empty() {
                notes.push(dest);
            }
        }
        Ok(notes)
    }

    fn read_string(&mut self, buf: &[u8]) -> Result<String, DecodeError> {
        let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        let mut decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(&buf[..nul_pos]);
        let mut dest = String::new();

        decoder.read_to_string(&mut dest)?;

        Ok(dest)
    }
}
