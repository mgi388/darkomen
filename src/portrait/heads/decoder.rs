use std::{
    array::TryFromSliceError,
    fmt,
    io::{Error as IoError, Read, Seek},
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use super::*;

pub(crate) const HEADER_SIZE_BYTES: usize = 1;
pub(crate) const ENTRY_SIZE_BYTES: usize = 39;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
    TryFromSliceError(TryFromSliceError),
    InvalidHeadFlags(u8),
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
            DecodeError::InvalidHeadFlags(v) => write!(f, "invalid head flags: {v}"),
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

    pub fn decode(&mut self) -> Result<HeadsDatabase, DecodeError> {
        let entry_count = self.decode_header()?;
        let entries = self.read_entries(entry_count)?;

        Ok(HeadsDatabase { entries })
    }

    fn decode_header(&mut self) -> Result<u8, DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    fn read_entries(&mut self, entry_count: u8) -> Result<Vec<HeadEntry>, DecodeError> {
        let mut entries = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let mut buf = [0; ENTRY_SIZE_BYTES];
            self.reader.read_exact(&mut buf)?;

            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(&buf[0..2]);
            let mut name = String::new();
            decoder.read_to_string(&mut name)?;

            let unknown1 = buf[2];
            let flags_u8 = buf[3];
            let flags =
                HeadFlags::from_bits(flags_u8).ok_or(DecodeError::InvalidHeadFlags(flags_u8))?;
            let unknown2 = buf[4..12].to_vec();
            let unknown3 = buf[12];

            let feature_mesh_id_0 = buf[13];
            let feature_pos_0_x = buf[14];
            let feature_pos_0_y = buf[15];
            let feature_pos_0_z = buf[16];

            let feature_mesh_id_1 = buf[17];
            let feature_pos_1_x = buf[18];
            let feature_pos_1_y = buf[19];
            let feature_pos_1_z = buf[20];

            let unknown4 = buf[21];
            let unknown5 = buf[22];

            let accessory_mesh_id_0 = buf[23];
            let accessory_pos_0_x = buf[24];
            let accessory_pos_0_y = buf[25];
            let accessory_pos_0_z = buf[26];

            let accessory_mesh_id_1 = buf[27];
            let accessory_pos_1_x = buf[28];
            let accessory_pos_1_y = buf[29];
            let accessory_pos_1_z = buf[30];

            let accessory_mesh_id_2 = buf[31];
            let accessory_pos_2_x = buf[32];
            let accessory_pos_2_y = buf[33];
            let accessory_pos_2_z = buf[34];

            let accessory_mesh_id_3 = buf[35];
            let accessory_pos_3_x = buf[36];
            let accessory_pos_3_y = buf[37];
            let accessory_pos_3_z = buf[38];

            entries.push(HeadEntry {
                name,
                unknown1,
                flags,
                unknown2,
                unknown3,
                features: [
                    FeatureSlot {
                        mesh_id: feature_mesh_id_0,
                        position: [feature_pos_0_x, feature_pos_0_y, feature_pos_0_z],
                    },
                    FeatureSlot {
                        mesh_id: feature_mesh_id_1,
                        position: [feature_pos_1_x, feature_pos_1_y, feature_pos_1_z],
                    },
                ],
                unknown4,
                unknown5,
                accessories: [
                    AccessorySlot {
                        mesh_id: accessory_mesh_id_0,
                        position: [accessory_pos_0_x, accessory_pos_0_y, accessory_pos_0_z],
                    },
                    AccessorySlot {
                        mesh_id: accessory_mesh_id_1,
                        position: [accessory_pos_1_x, accessory_pos_1_y, accessory_pos_1_z],
                    },
                    AccessorySlot {
                        mesh_id: accessory_mesh_id_2,
                        position: [accessory_pos_2_x, accessory_pos_2_y, accessory_pos_2_z],
                    },
                    AccessorySlot {
                        mesh_id: accessory_mesh_id_3,
                        position: [accessory_pos_3_x, accessory_pos_3_y, accessory_pos_3_z],
                    },
                ],
            });
        }

        Ok(entries)
    }
}
