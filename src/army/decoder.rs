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
    InvalidRegimentStatus(u16),
    InvalidMageClass(u8),
    InvalidRegimentAttributes(u32),
    InvalidRegimentAlignment(u8),
    InvalidRegimentMount(u8),
    InvalidWeapon(u8),
    InvalidProjectile(u8),
    InvalidRegimentClass(u8),
    InvalidSpellBook(u16),
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
            DecodeError::InvalidRegimentStatus(v) => write!(f, "invalid regiment status: {}", v),
            DecodeError::InvalidMageClass(v) => write!(f, "invalid mage class: {}", v),
            DecodeError::InvalidRegimentAttributes(v) => {
                write!(f, "invalid regiment attributes: {}", v)
            }
            DecodeError::InvalidRegimentAlignment(v) => {
                write!(f, "invalid regiment alignment: {}", v)
            }
            DecodeError::InvalidWeapon(v) => write!(f, "invalid weapon: {}", v),
            DecodeError::InvalidProjectile(v) => write!(f, "invalid projectile: {}", v),
            DecodeError::InvalidRegimentMount(v) => {
                write!(f, "invalid regiment mount: {}", v)
            }
            DecodeError::InvalidRegimentClass(v) => write!(f, "invalid regiment class: {}", v),
            DecodeError::InvalidSpellBook(v) => {
                write!(f, "invalid spell book: {}", v)
            }
        }
    }
}

pub(crate) const FORMAT: u32 = 0x0000029e;
pub(crate) const HEADER_SIZE_BYTES: usize = 192;
const SAVE_GAME_HEADER_SIZE_BYTES: usize = 504;
pub(crate) const REGIMENT_SIZE_BYTES: usize = 188;

pub(crate) struct Header {
    _format: u32,
    regiment_count: u32,
    /// The size of each regiment block in bytes.
    ///
    /// This is always 188 despite being encoded in the header.
    _regiment_size_bytes: u32,
    race: u8,
    unknown1: [u8; 3], // always seems to be 0, could be padding
    default_name_index: u16,
    name: String,
    /// There are some bytes after the nul-terminated string. Not sure what they
    /// are for.
    name_remainder: Vec<u8>,
    small_banner_path: String,
    /// There are some bytes after the nul-terminated string. Not sure what they
    /// are for.
    small_banner_path_remainder: Vec<u8>,
    small_banner_disabled_path: String,
    /// There are some bytes after the nul-terminated string. Not sure what they
    /// are for.
    small_banner_disabled_path_remainder: Vec<u8>,
    large_banner_path: String,
    /// There are some bytes after the nul-terminated string. Not sure what they
    /// are for.
    large_banner_path_remainder: Vec<u8>,
    last_battle_gold: u16,
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
        let (start_pos, save_game_header) = self.maybe_read_save_game_header()?;

        let header = self.read_header(start_pos)?;

        let race = ArmyRace::try_from(header.race)
            .map_err(|_| DecodeError::InvalidArmyRace(header.race))?;

        let regiments = self.read_regiments(&header)?;

        let mut save_game_footer = Vec::new();
        self.reader.read_to_end(&mut save_game_footer)?;

        Ok(Army {
            save_game_header,
            race,
            unknown1: header.unknown1,
            default_name_index: header.default_name_index,
            name: header.name,
            name_remainder: header.name_remainder,
            small_banner_path: header.small_banner_path,
            small_banner_path_remainder: header.small_banner_path_remainder,
            small_banner_disabled_path: header.small_banner_disabled_path,
            small_banner_disabled_path_remainder: header.small_banner_disabled_path_remainder,
            large_banner_path: header.large_banner_path,
            large_banner_path_remainder: header.large_banner_path_remainder,
            last_battle_gold: header.last_battle_gold,
            gold_in_coffers: header.gold_in_coffers,
            magic_items: header.magic_items.to_vec(),
            unknown3: header.unknown3.to_vec(),
            regiments,
            save_game_footer,
        })
    }

    fn maybe_read_save_game_header(
        &mut self,
    ) -> Result<(u64, Option<SaveGameHeader>), DecodeError> {
        let mut buf = [0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;

        let format = u32::from_le_bytes(buf[0..size_of::<u32>()].try_into().unwrap());

        if format != FORMAT {
            self.reader.seek(SeekFrom::Start(0))?;

            let mut buf = vec![0; SAVE_GAME_HEADER_SIZE_BYTES];
            self.reader.read_exact(&mut buf)?;

            let display_name_buf = &buf[0..90];
            let (display_name_buf, display_name_remainder) = display_name_buf
                .iter()
                .enumerate()
                .find(|(_, &b)| b == 0)
                .map(|(i, _)| display_name_buf.split_at(i + 1))
                .unwrap_or((display_name_buf, &[]));

            let suggested_display_name_buf = &buf[90..408];
            let (suggested_display_name_buf, suggested_display_name_remainder) =
                suggested_display_name_buf
                    .iter()
                    .enumerate()
                    .find(|(_, &b)| b == 0)
                    .map(|(i, _)| suggested_display_name_buf.split_at(i + 1))
                    .unwrap_or((suggested_display_name_buf, &[]));

            return Ok((
                SAVE_GAME_HEADER_SIZE_BYTES as u64,
                Some(SaveGameHeader {
                    display_name: self.read_string(display_name_buf)?,
                    display_name_remainder: display_name_remainder.to_vec(),
                    suggested_display_name: self.read_string(suggested_display_name_buf)?,
                    suggested_display_name_remainder: suggested_display_name_remainder.to_vec(),
                    bogenhafen_mission: buf[408] != 0,
                    goblin_camp_or_ragnar: buf[412] != 0,
                    goblin_camp_mission: buf[416] != 0,
                    ragnar_mission_pre_battle: buf[420] != 0,
                    vingtienne_or_treeman: buf[424] != 0,
                    vingtienne_mission: buf[428] != 0,
                    treeman_mission: buf[432] != 0,
                    carstein_defeated: buf[436] != 0,
                    hand_of_nagash_defeated: buf[440] != 0,
                    black_grail_defeated: buf[444] != 0,
                    unknown1: u32::from_le_bytes(buf[448..452].try_into().unwrap()),
                    helmgart_mission: buf[452] != 0,
                    ragnar_mission: buf[456] != 0,
                    loren_king_met: buf[460] != 0,
                    axebite_mission: buf[464] != 0,
                    unknown2: u32::from_le_bytes(buf[468..472].try_into().unwrap()),
                    unknown3: u32::from_le_bytes(buf[472..476].try_into().unwrap()),
                    unknown4: u32::from_le_bytes(buf[476..480].try_into().unwrap()),
                    unknown5: u32::from_le_bytes(buf[480..484].try_into().unwrap()),
                    unknown6: u32::from_le_bytes(buf[484..488].try_into().unwrap()),
                    unknown7: u32::from_le_bytes(buf[488..492].try_into().unwrap()),
                    previous_battle_won_1: buf[492] != 0,
                    previous_battle_won_2: buf[496] != 0,
                    previous_answer: u32::from_le_bytes(buf[500..504].try_into().unwrap()),
                }),
            ));
        }

        Ok((0, None))
    }

    fn read_header(&mut self, start_pos: u64) -> Result<Header, DecodeError> {
        self.reader.seek(SeekFrom::Start(start_pos))?;

        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let army_name_buf = &buf[18..50];
        let (army_name_buf, army_name_remainder) = army_name_buf
            .iter()
            .enumerate()
            .find(|(_, &b)| b == 0)
            .map(|(i, _)| army_name_buf.split_at(i + 1))
            .unwrap_or((army_name_buf, &[]));

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
            _regiment_size_bytes: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            race: buf[12],
            unknown1: buf[13..16].try_into().unwrap(),
            default_name_index: u16::from_le_bytes(buf[16..18].try_into().unwrap()),
            name: self.read_string(army_name_buf)?,
            name_remainder: army_name_remainder.to_vec(),
            small_banner_path: self.read_string(small_banner_path_buf)?,
            small_banner_path_remainder: small_banner_path_remainder.to_vec(),
            small_banner_disabled_path: self.read_string(small_banner_disabled_path_buf)?,
            small_banner_disabled_path_remainder: small_banner_disabled_path_remainder.to_vec(),
            large_banner_path: self.read_string(large_banner_path_buf)?,
            large_banner_path_remainder: large_banner_path_remainder.to_vec(),
            last_battle_gold: u16::from_le_bytes(buf[146..148].try_into().unwrap()),
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
        let mut buf = vec![0; REGIMENT_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let status_u16 = u16::from_le_bytes(buf[0..2].try_into().unwrap());
        let status = RegimentStatus::try_from(status_u16)
            .map_err(|_| DecodeError::InvalidRegimentStatus(status_u16))?;
        let attributes_u32 = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let attributes = RegimentAttributes::from_bits(attributes_u32)
            .ok_or(DecodeError::InvalidRegimentAttributes(attributes_u32))?;
        let mage_class =
            MageClass::try_from(buf[8]).map_err(|_| DecodeError::InvalidMageClass(buf[8]))?;
        let unit_alignment = RegimentAlignment::try_from(buf[56])
            .map_err(|_| DecodeError::InvalidRegimentAlignment(buf[56]))?;
        let unit_mount = RegimentMount::try_from(buf[73])
            .map_err(|_| DecodeError::InvalidRegimentMount(buf[73]))?;
        let unit_weapon =
            Weapon::try_from(buf[75]).map_err(|_| DecodeError::InvalidWeapon(buf[75]))?;
        let unit_class = RegimentClass::try_from(buf[76])
            .map_err(|_| DecodeError::InvalidRegimentClass(buf[76]))?;
        let unit_projectile =
            Projectile::try_from(buf[78]).map_err(|_| DecodeError::InvalidProjectile(buf[78]))?;
        let leader_alignment = RegimentAlignment::try_from(buf[120])
            .map_err(|_| DecodeError::InvalidRegimentAlignment(buf[120]))?;
        let leader_mount = RegimentMount::try_from(buf[136])
            .map_err(|_| DecodeError::InvalidRegimentMount(buf[136]))?;
        let leader_weapon =
            Weapon::try_from(buf[138]).map_err(|_| DecodeError::InvalidWeapon(buf[138]))?;
        let leader_class = RegimentClass::try_from(buf[139])
            .map_err(|_| DecodeError::InvalidRegimentClass(buf[139]))?;
        let leader_projectile =
            Projectile::try_from(buf[141]).map_err(|_| DecodeError::InvalidProjectile(buf[141]))?;
        let spell_book_u16 = u16::from_le_bytes(buf[160..162].try_into().unwrap());
        let spell_book = SpellBook::try_from(spell_book_u16)
            .map_err(|_| DecodeError::InvalidSpellBook(spell_book_u16))?;

        Ok(Regiment {
            status,
            unknown1: buf[2..4].try_into().unwrap(),
            id: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            mage_class,
            max_armor: buf[9],
            cost: u16::from_le_bytes(buf[10..12].try_into().unwrap()),
            banner_sprite_sheet_index: u16::from_le_bytes(buf[12..14].try_into().unwrap()),
            unknown3: buf[14..16].try_into().unwrap(),
            attributes,
            unit_profile: UnitProfile {
                sprite_sheet_index: u16::from_le_bytes(buf[20..22].try_into().unwrap()),
                name: self.read_string(&buf[22..54])?,
                name_id: u16::from_le_bytes(buf[54..56].try_into().unwrap()),
                alignment: unit_alignment,
                max_unit_count: buf[57],
                alive_unit_count: buf[58],
                rank_count: buf[59],
                unknown1: buf[60..64].into(),
                stats: self.read_unit_stats(&buf[64..73]),
                mount: unit_mount,
                armor: buf[74],
                weapon: unit_weapon,
                class: unit_class,
                point_value: buf[77],
                projectile: unit_projectile,
            },
            unknown4: buf[79],
            unknown5: buf[80..84].try_into().unwrap(),
            leader_profile: UnitProfile {
                sprite_sheet_index: u16::from_le_bytes(buf[84..86].try_into().unwrap()),
                name: self.read_string(&buf[86..118])?,
                name_id: u16::from_le_bytes(buf[118..120].try_into().unwrap()),
                alignment: leader_alignment,
                max_unit_count: buf[121],
                alive_unit_count: buf[122],
                rank_count: buf[123],
                unknown1: buf[124..127].into(),
                stats: self.read_unit_stats(&buf[127..136]),
                mount: leader_mount,
                armor: buf[137],
                weapon: leader_weapon,
                class: leader_class,
                point_value: buf[140],
                projectile: leader_projectile,
            },
            unknown6: buf[142..146].try_into().unwrap(),
            leader_head_id: u16::from_le_bytes(buf[146..148].try_into().unwrap()),
            last_battle_stats: self.read_last_battle_stats(&buf[148..156])?,
            total_experience: u16::from_le_bytes(buf[156..158].try_into().unwrap()),
            duplicate_id: buf[158],
            min_armor: buf[159],
            spell_book,
            magic_items: [
                u16::from_le_bytes(buf[162..164].try_into().unwrap()),
                u16::from_le_bytes(buf[164..166].try_into().unwrap()),
                u16::from_le_bytes(buf[166..168].try_into().unwrap()),
            ],
            spells: [
                u16::from_le_bytes(buf[168..170].try_into().unwrap()),
                u16::from_le_bytes(buf[170..172].try_into().unwrap()),
                u16::from_le_bytes(buf[172..174].try_into().unwrap()),
                u16::from_le_bytes(buf[174..176].try_into().unwrap()),
                u16::from_le_bytes(buf[176..178].try_into().unwrap()),
            ],
            gold_captured: u16::from_le_bytes(buf[178..180].try_into().unwrap()),
            purchased_armor: buf[180],
            max_purchasable_armor: buf[181],
            repurchased_unit_count: buf[182],
            max_purchasable_unit_count: buf[183],
            book_profile: buf[184..188].try_into().unwrap(),
        })
    }

    fn read_unit_stats(&mut self, buf: &[u8]) -> UnitStats {
        UnitStats {
            movement: buf[0],
            weapon_skill: buf[1],
            ballistic_skill: buf[2],
            strength: buf[3],
            toughness: buf[4],
            wounds: buf[5],
            initiative: buf[6],
            attacks: buf[7],
            leadership: buf[8],
        }
    }

    fn read_last_battle_stats(&mut self, buf: &[u8]) -> Result<LastBattleStats, DecodeError> {
        Ok(LastBattleStats {
            unit_killed_count: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            unknown1: buf[2..4].try_into().unwrap(),
            kill_count: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            experience: u16::from_le_bytes(buf[6..8].try_into().unwrap()),
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
