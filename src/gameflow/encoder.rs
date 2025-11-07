use std::{
    ffi::CString,
    fmt,
    io::{BufWriter, Write},
};

use encoding_rs::WINDOWS_1252;

use super::{decoder::*, *};

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
    StringTooLong,
    TooManyControlPoints,
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
            EncodeError::TooManyControlPoints => write!(f, "too many control points"),
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

    pub fn encode(&mut self, gameflow: &Gameflow) -> Result<(), EncodeError> {
        self.write_header(gameflow)?;
        self.write_paths(&gameflow.paths)?;
        self.write_footer(gameflow)?;
        Ok(())
    }

    fn write_header(&mut self, gameflow: &Gameflow) -> Result<(), EncodeError> {
        self.writer.write_all(FORMAT)?;
        self.writer.write_all(&gameflow.unknown1.to_le_bytes())?;
        self.writer.write_all(&gameflow.unknown2.to_le_bytes())?;
        self.writer.write_all(&gameflow.unknown3.to_le_bytes())?;
        self.writer
            .write_all(&(gameflow.paths.len() as u32).to_le_bytes())?;
        Ok(())
    }

    fn write_paths(&mut self, paths: &[Path]) -> Result<(), EncodeError> {
        for path in paths {
            if path.control_points.len() > MAX_CONTROL_POINTS {
                return Err(EncodeError::TooManyControlPoints);
            }

            self.writer
                .write_all(&(path.control_points.len() as u32).to_le_bytes())?;

            for point in &path.control_points {
                self.writer.write_all(&point.x.to_le_bytes())?;
                self.writer.write_all(&point.y.to_le_bytes())?;
                self.writer.write_all(&point.unknown1.to_le_bytes())?;
                self.writer.write_all(&point.unknown2.to_le_bytes())?;
            }

            self.writer.write_all(&path.unknown1.to_le_bytes())?;
            self.writer
                .write_all(&path.curve_point_spacing.to_le_bytes())?;
            self.writer.write_all(&path.unknown3.to_le_bytes())?;
            self.writer.write_all(&path.unknown4.to_le_bytes())?;
            self.writer
                .write_all(&path.previous_path_index.to_le_bytes())?;
            self.writer.write_all(&path.next_path_index.to_le_bytes())?;
            self.writer.write_all(&path.unknown7.to_le_bytes())?;
            self.writer.write_all(&path.unknown8)?;
        }
        Ok(())
    }

    fn write_footer(&mut self, gameflow: &Gameflow) -> Result<(), EncodeError> {
        self.write_notes(&gameflow.notes)?;
        self.write_string_with_limit(&gameflow.map_file_name, MAP_FILE_NAME_SIZE_BYTES)?;
        self.writer.write_all(&gameflow.unknown4)?;

        Ok(())
    }

    fn write_notes(&mut self, notes: &[String]) -> Result<(), EncodeError> {
        let mut total_bytes = 0;
        for note in notes {
            let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(note);
            let c_string =
                CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
            let bytes = c_string.as_bytes_with_nul();
            total_bytes += bytes.len();
            if total_bytes > NOTES_SIZE_BYTES {
                return Err(EncodeError::StringTooLong);
            }
            self.writer.write_all(bytes)?;
        }
        let remaining = NOTES_SIZE_BYTES - total_bytes;
        if remaining > 0 {
            self.writer.write_all(&vec![0; remaining])?;
        }
        Ok(())
    }

    fn write_string_with_limit(&mut self, s: &str, limit: usize) -> Result<(), EncodeError> {
        let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(s);

        let c_string = CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
        let bytes = c_string.as_bytes_with_nul();

        if bytes.len() > limit {
            return Err(EncodeError::StringTooLong);
        }

        self.writer.write_all(bytes)?;

        let padding_size_bytes = limit - bytes.len();
        let padding = vec![0; padding_size_bytes];
        self.writer.write_all(&padding)?;

        Ok(())
    }
}
