use super::*;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::{
    fmt,
    io::{Error as IoError, Read, Seek, SeekFrom},
    mem::size_of,
};

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(u32),
    InvalidString,
    InvalidArmyRace(u8),
    InvalidRegimentAlignment(u8),
    InvalidRegimentMount(u8),
    InvalidRegimentClass(u8),
    InvalidRegimentMagicBook(u16),
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
            DecodeError::InvalidFormat(format) => write!(f, "invalid format: {}", format),
            DecodeError::InvalidString => write!(f, "invalid string"),
            DecodeError::InvalidArmyRace(v) => write!(f, "invalid army race: {}", v),
            DecodeError::InvalidRegimentAlignment(v) => {
                write!(f, "invalid regiment alignment: {}", v)
            }
            DecodeError::InvalidRegimentMount(v) => {
                write!(f, "invalid regiment mount: {}", v)
            }
            DecodeError::InvalidRegimentClass(v) => write!(f, "invalid regiment class: {}", v),
            DecodeError::InvalidRegimentMagicBook(v) => {
                write!(f, "invalid regiment magic book: {}", v)
            }
        }
    }
}

pub(crate) const FORMAT: u32 = 0x0000029e;
pub(crate) const HEADER_SIZE: usize = 192;
const SAVE_HEADER_SIZE: usize = 504;
pub(crate) const REGIMENT_BLOCK_SIZE: usize = 188;

pub(crate) struct Header {
    _format: u32,
    regiment_count: u32,
    /// The size of each regiment block in bytes.
    ///
    /// This is always 188 despite being encoded in the header.
    _regiment_block_size: u32,
    race: u8,
    unknown1: [u8; 3],  // purpose of bytes at index 13, 14, 15 is unknown
    unknown2: [u8; 34], // purpose of bytes at index 16-50 is unknown
    small_banner_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    small_banner_path_remainder: Vec<u8>,
    small_banner_disabled_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    small_banner_disabled_path_remainder: Vec<u8>,
    large_banner_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    large_banner_path_remainder: Vec<u8>,
    gold_from_treasures: u16,
    gold_in_coffers: u16,
    magic_items: [u8; 40],
    unknown3: [u8; 2], // purpose of bytes at index 190 and 191 is unknown
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

    pub fn decode(&mut self) -> Result<Army, DecodeError> {
        let start_pos = self.maybe_read_save_file()?;

        let header = self.read_header(start_pos)?;

        let race = ArmyRace::try_from(header.race)
            .map_err(|_| DecodeError::InvalidArmyRace(header.race))?;

        let regiments = self.read_regiments(&header)?;

        Ok(Army {
            race,
            unknown1: header.unknown1.to_vec(),
            unknown2: header.unknown2.to_vec(),
            regiments,
            small_banner_path: header.small_banner_path,
            small_banner_path_remainder: header.small_banner_path_remainder,
            small_banner_disabled_path: header.small_banner_disabled_path,
            small_banner_disabled_path_remainder: header.small_banner_disabled_path_remainder,
            large_banner_path: header.large_banner_path,
            large_banner_path_remainder: header.large_banner_path_remainder,
            gold_from_treasures: header.gold_from_treasures,
            gold_in_coffers: header.gold_in_coffers,
            magic_items: header.magic_items.to_vec(),
            unknown3: header.unknown3.to_vec(),
        })
    }

    fn maybe_read_save_file(&mut self) -> Result<u64, DecodeError> {
        let mut buf = [0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;

        let mut start_pos = 0;

        let format = u32::from_le_bytes(buf[0..size_of::<u32>()].try_into().unwrap());
        if format != FORMAT {
            // TODO: Skipped over reading save header.
            start_pos = SAVE_HEADER_SIZE as u64;
        }
        Ok(start_pos)
    }

    fn read_header(&mut self, start_pos: u64) -> Result<Header, DecodeError> {
        self.reader.seek(SeekFrom::Start(start_pos))?;

        let mut buf = [0; HEADER_SIZE];
        self.reader.read_exact(&mut buf)?;

        let small_banner_path_buf = &buf[50..82];
        let (small_banner_path_buf, small_banner_path_remainder) = small_banner_path_buf
            .iter()
            .enumerate()
            .find(|(_, &b)| b == 0)
            .map(|(i, _)| small_banner_path_buf.split_at(i + 1))
            .unwrap_or((small_banner_path_buf, &[]));

        let small_banner_disabled_path_buf = &buf[82..114];
        let (small_banner_disabled_path_buf, small_banner_disabled_path_remainder) =
            small_banner_disabled_path_buf
                .iter()
                .enumerate()
                .find(|(_, &b)| b == 0)
                .map(|(i, _)| small_banner_disabled_path_buf.split_at(i + 1))
                .unwrap_or((small_banner_disabled_path_buf, &[]));

        let large_banner_path_buf = &buf[114..146];
        let (large_banner_path_buf, large_banner_path_remainder) = large_banner_path_buf
            .iter()
            .enumerate()
            .find(|(_, &b)| b == 0)
            .map(|(i, _)| large_banner_path_buf.split_at(i + 1))
            .unwrap_or((large_banner_path_buf, &[]));

        Ok(Header {
            _format: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            regiment_count: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            _regiment_block_size: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            race: buf[12],
            unknown1: buf[13..16].try_into().unwrap(),
            unknown2: buf[16..50].try_into().unwrap(),
            small_banner_path: self.read_string(small_banner_path_buf)?,
            small_banner_path_remainder: small_banner_path_remainder.to_vec(),
            small_banner_disabled_path: self.read_string(small_banner_disabled_path_buf)?,
            small_banner_disabled_path_remainder: small_banner_disabled_path_remainder.to_vec(),
            large_banner_path: self.read_string(large_banner_path_buf)?,
            large_banner_path_remainder: large_banner_path_remainder.to_vec(),
            gold_from_treasures: u16::from_le_bytes(buf[146..148].try_into().unwrap()),
            gold_in_coffers: u16::from_le_bytes(buf[148..150].try_into().unwrap()),
            magic_items: buf[150..190].try_into().unwrap(),
            unknown3: buf[190..192].try_into().unwrap(),
        })
    }

    fn read_regiments(&mut self, header: &Header) -> Result<Vec<Regiment>, DecodeError> {
        let mut regiments = Vec::with_capacity(header.regiment_count as usize);

        for _ in 0..header.regiment_count {
            regiments.push(self.read_regiment()?);
        }

        Ok(regiments)
    }

    fn read_regiment(&mut self) -> Result<Regiment, DecodeError> {
        let mut buf = vec![0; REGIMENT_BLOCK_SIZE];
        self.reader.read_exact(&mut buf)?;

        let alignment = RegimentAlignment::try_from(buf[56])
            .map_err(|_| DecodeError::InvalidRegimentAlignment(buf[56]))?;
        let mount = RegimentMount::try_from(buf[73])
            .map_err(|_| DecodeError::InvalidRegimentMount(buf[73]))?;
        let (typ, race) = Regiment::decode_class(buf[76])
            .map_err(|_| -> DecodeError { DecodeError::InvalidRegimentClass(buf[76]) })?;
        let magic_book_u16 = u16::from_le_bytes(buf[160..162].try_into().unwrap());
        let magic_book = RegimentMagicBook::try_from(magic_book_u16)
            .map_err(|_| DecodeError::InvalidRegimentMagicBook(magic_book_u16))?;

        Ok(Regiment {
            status: buf[0..2].try_into().unwrap(),
            unknown1: buf[2..4].try_into().unwrap(),
            id: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            unknown2: buf[6..8].try_into().unwrap(),
            wizard_type: buf[8],
            max_armor: buf[9],
            cost: u16::from_le_bytes(buf[10..12].try_into().unwrap()),
            banner_index: u16::from_le_bytes(buf[12..14].try_into().unwrap()),
            unknown3: buf[14..16].try_into().unwrap(),
            regiment_attributes: buf[16..20].try_into().unwrap(),
            sprite_index: u16::from_le_bytes(buf[20..22].try_into().unwrap()),
            name: self.read_string(&buf[22..54])?,
            name_id: u16::from_le_bytes(buf[54..56].try_into().unwrap()),
            alignment,
            max_troops: buf[57],
            alive_troops: buf[58],
            ranks: buf[59],
            unknown4: buf[60..64].try_into().unwrap(),
            troop_attributes: TroopAttributes {
                movement: buf[64],
                weapon_skill: buf[65],
                ballistic_skill: buf[66],
                strength: buf[67],
                toughness: buf[68],
                wounds: buf[69],
                initiative: buf[70],
                attacks: buf[71],
                leadership: buf[72],
            },
            mount,
            armor: buf[74],
            weapon: buf[75],
            typ,
            race,
            point_value: buf[77],
            missile_weapon: buf[78],
            unknown5: buf[79],
            unknown6: buf[80..84].try_into().unwrap(),
            leader: Leader {
                sprite_index: u16::from_le_bytes(buf[84..86].try_into().unwrap()),
                name: self.read_string(&buf[86..118])?,
                name_remainder: buf[118..127].to_vec(),
                attributes: TroopAttributes {
                    movement: buf[127],
                    weapon_skill: buf[128],
                    ballistic_skill: buf[129],
                    strength: buf[130],
                    toughness: buf[131],
                    wounds: buf[132],
                    initiative: buf[133],
                    attacks: buf[134],
                    leadership: buf[135],
                },
                mount: buf[136],
                armor: buf[137],
                weapon: buf[138],
                unit_type: buf[139],
                point_value: buf[140],
                missile_weapon: buf[141],
                unknown1: buf[142..146].try_into().unwrap(),
                head_id: u16::from_le_bytes(buf[146..148].try_into().unwrap()),
                x: buf[148..152].try_into().unwrap(),
                y: buf[152..156].try_into().unwrap(),
            },
            experience: u16::from_le_bytes(buf[156..158].try_into().unwrap()),
            duplicate_id: buf[158],
            min_armor: buf[159],
            magic_book,
            magic_items: [
                u16::from_le_bytes(buf[162..164].try_into().unwrap()),
                u16::from_le_bytes(buf[164..166].try_into().unwrap()),
                u16::from_le_bytes(buf[166..168].try_into().unwrap()),
            ],
            unknown7: buf[168..180].try_into().unwrap(),
            purchased_armor: buf[180],
            max_purchasable_armor: buf[181],
            repurchased_troops: buf[182],
            max_purchasable_troops: buf[183],
            book_profile: buf[184..188].try_into().unwrap(),
        })
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
