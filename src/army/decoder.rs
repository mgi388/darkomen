use std::{
    fmt,
    io::{Error as IoError, Read, Seek, SeekFrom},
    mem::size_of,
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use super::*;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(u32),
    InvalidString,
    InvalidArmyRace(u8),
    InvalidRegimentFlags(u16),
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
            DecodeError::InvalidRegimentFlags(v) => write!(f, "invalid regiment flags: {}", v),
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
pub(crate) const SAVE_GAME_DISPLAY_NAME_SIZE_BYTES: usize = 90;
const SCRIPT_STATE_SIZE_BYTES: usize = 220;
pub(crate) const REGIMENT_SIZE_BYTES: usize = 188;
pub(crate) const SAVE_GAME_FOOTER_UNKNOWN1_SIZE_BYTES: usize = 1976;
pub(crate) const SAVE_GAME_CUTSCENE_ANIMATION_COUNT: usize = 38;
pub(crate) const SAVE_GAME_CUTSCENE_SIZE_BYTES: usize = 288;
pub(crate) const SAVE_GAME_ASSET_PATH_SIZE_BYTES: usize = 256;

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
    small_disabled_banner_path: String,
    /// There are some bytes after the nul-terminated string. Not sure what they
    /// are for.
    small_disabled_banner_path_remainder: Vec<u8>,
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

        let race =
            ArmyRace::from_bits(header.race).ok_or(DecodeError::InvalidArmyRace(header.race))?;

        let regiments = self.read_regiments(&header)?;

        let save_game_footer = self.maybe_read_save_game_footer()?;

        Ok(Army {
            save_game_header,
            race,
            unknown1: header.unknown1,
            default_name_index: header.default_name_index,
            name: header.name,
            name_remainder: header.name_remainder,
            small_banner_path: header.small_banner_path,
            small_banner_path_remainder: header.small_banner_path_remainder,
            small_disabled_banner_path: header.small_disabled_banner_path,
            small_disabled_banner_path_remainder: header
                .small_disabled_banner_path_remainder
                .clone(),
            small_disabled_banner_path_remainder_as_u16s: header
                .small_disabled_banner_path_remainder
                .clone()
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            small_disabled_banner_path_remainder_as_u32s: header
                .small_disabled_banner_path_remainder
                .clone()
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            large_banner_path: header.large_banner_path,
            large_banner_path_remainder: header.large_banner_path_remainder.clone(),
            large_banner_path_remainder_as_u16s: header
                .large_banner_path_remainder
                .clone()
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            large_banner_path_remainder_as_u32s: header
                .large_banner_path_remainder
                .clone()
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            last_battle_gold: header.last_battle_gold,
            gold_in_coffers: header.gold_in_coffers,
            magic_items: header.magic_items.to_vec(),
            unknown3: header.unknown3.to_vec(),
            regiments,
            save_game_footer,
        })
    }

    fn read_script_state(&mut self, buf: &[u8]) -> Result<ScriptState, DecodeError> {
        let unknown2 = buf[28..100].to_vec();
        let unknown7 = buf[136..].to_vec();

        Ok(ScriptState {
            program_counter: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            unknown0: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            base_execution_address: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            unknown_address: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            local_variable: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            unknown1: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            stack_pointer: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
            unknown2: unknown2.clone(),
            unknown2_hex: unknown2
                .chunks(16)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<String>>()
                        .join("")
                })
                .collect(),
            unknown2_as_u32s: unknown2
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            execution_offset_index: u32::from_le_bytes(buf[100..104].try_into().unwrap()),
            unknown3: u32::from_le_bytes(buf[104..108].try_into().unwrap()),
            nest_if: u32::from_le_bytes(buf[108..112].try_into().unwrap()),
            nest_gosub: u32::from_le_bytes(buf[112..116].try_into().unwrap()),
            nest_loop: u32::from_le_bytes(buf[116..120].try_into().unwrap()),
            unknown4: u32::from_le_bytes(buf[120..124].try_into().unwrap()),
            elapsed_millis: u32::from_le_bytes(buf[124..128].try_into().unwrap()),
            unknown5: u32::from_le_bytes(buf[128..132].try_into().unwrap()),
            unknown6: u32::from_le_bytes(buf[132..136].try_into().unwrap()),
            unknown7: unknown7.clone(),
            unknown7_hex: unknown7
                .chunks(16)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<String>>()
                        .join("")
                })
                .collect(),
            unknown7_as_u32s: unknown7
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
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

            let display_name_buf = &buf[0..SAVE_GAME_DISPLAY_NAME_SIZE_BYTES];
            let (display_name_buf, display_name_residual_bytes) = display_name_buf
                .iter()
                .enumerate()
                .find(|(_, &b)| b == 0)
                .map(|(i, _)| display_name_buf.split_at(i + 1))
                .unwrap_or((display_name_buf, &[]));

            let suggested_display_name_buf =
                &buf[SAVE_GAME_DISPLAY_NAME_SIZE_BYTES..SAVE_GAME_DISPLAY_NAME_SIZE_BYTES * 2];
            let (suggested_display_name_buf, suggested_display_name_residual_bytes) =
                suggested_display_name_buf
                    .iter()
                    .enumerate()
                    .find(|(_, &b)| b == 0)
                    .map(|(i, _)| suggested_display_name_buf.split_at(i + 1))
                    .unwrap_or((suggested_display_name_buf, &[]));

            let script_state_buf = buf[188..188 + SCRIPT_STATE_SIZE_BYTES].to_vec();

            let script_state = self.read_script_state(&script_state_buf)?;

            return Ok((
                SAVE_GAME_HEADER_SIZE_BYTES as u64,
                Some(SaveGameHeader {
                    display_name: self.read_string(display_name_buf)?,
                    display_name_residual_bytes: if display_name_residual_bytes
                        .iter()
                        .all(|&b| b == 0)
                    {
                        None
                    } else {
                        Some(
                            display_name_residual_bytes
                                .iter()
                                .rposition(|&b| b != 0) // find the last non-zero byte
                                .map(|pos| &display_name_residual_bytes[..=pos]) // include the last non-zero byte
                                .unwrap_or(display_name_residual_bytes)
                                .to_vec(),
                        )
                    },
                    suggested_display_name: self.read_string(suggested_display_name_buf)?,
                    suggested_display_name_residual_bytes: if suggested_display_name_residual_bytes
                        .iter()
                        .all(|&b| b == 0)
                    {
                        None
                    } else {
                        Some(
                            suggested_display_name_residual_bytes
                                .iter()
                                .rposition(|&b| b != 0) // find the last non-zero byte
                                .map(|pos| &suggested_display_name_residual_bytes[..=pos]) // include the last non-zero byte
                                .unwrap_or(suggested_display_name_residual_bytes)
                                .to_vec(),
                        )
                    },
                    unknown_bool1: buf[180] != 0,
                    unknown_bool2: buf[184] != 0,
                    script_state_hex: script_state_buf
                        .clone()
                        .chunks(16)
                        .map(|chunk| {
                            chunk
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<String>>()
                                .join("")
                        })
                        .collect(),
                    script_state_as_u32s: script_state_buf
                        .chunks_exact(4)
                        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                        .collect(),
                    script_state,
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

    fn maybe_read_save_game_footer(&mut self) -> Result<Option<SaveGameFooter>, DecodeError> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;

        if buf.is_empty() {
            return Ok(None);
        }

        let unknown1 = buf[0..SAVE_GAME_FOOTER_UNKNOWN1_SIZE_BYTES].to_vec();

        let background_image_path_offset_end =
            SAVE_GAME_FOOTER_UNKNOWN1_SIZE_BYTES + SAVE_GAME_ASSET_PATH_SIZE_BYTES;
        let background_image_path_buf =
            &buf[SAVE_GAME_FOOTER_UNKNOWN1_SIZE_BYTES..background_image_path_offset_end];
        let (background_image_path_buf, background_image_path_residual_bytes) =
            background_image_path_buf
                .iter()
                .enumerate()
                .find(|(_, &b)| b == 0)
                .map(|(i, _)| background_image_path_buf.split_at(i + 1))
                .unwrap_or((background_image_path_buf, &[]));
        let background_image_path = self.read_string(background_image_path_buf)?;

        let unknown2_offset_end = background_image_path_offset_end + 16;
        let unknown2 = buf[background_image_path_offset_end..unknown2_offset_end].to_vec();

        let animations_offset_end = unknown2_offset_end
            + SAVE_GAME_CUTSCENE_ANIMATION_COUNT * SAVE_GAME_CUTSCENE_SIZE_BYTES;
        let mut animations_buf =
            [0; SAVE_GAME_CUTSCENE_ANIMATION_COUNT * SAVE_GAME_CUTSCENE_SIZE_BYTES];
        animations_buf.copy_from_slice(&buf[unknown2_offset_end..animations_offset_end]);
        let mut animations = Vec::with_capacity(SAVE_GAME_CUTSCENE_ANIMATION_COUNT);
        for i in 0..SAVE_GAME_CUTSCENE_ANIMATION_COUNT {
            animations.push(self.read_cutscene_animation(
                &animations_buf
                    [i * SAVE_GAME_CUTSCENE_SIZE_BYTES..(i + 1) * SAVE_GAME_CUTSCENE_SIZE_BYTES],
            )?);
        }

        let unknown3 = buf[animations_offset_end..].to_vec();

        let hex: Vec<String> = buf
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect();

        Ok(Some(SaveGameFooter {
            unknown1: unknown1.clone(),
            unknown1_as_u16s: unknown1
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            unknown1_as_u32s: unknown1
                .clone()
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            background_image_path: if background_image_path.is_empty() {
                None
            } else {
                Some(background_image_path)
            },
            background_image_path_residual_bytes: if background_image_path_residual_bytes
                .iter()
                .all(|&b| b == 0)
            {
                None
            } else {
                Some(
                    background_image_path_residual_bytes
                        .iter()
                        .rposition(|&b| b != 0) // find the last non-zero byte
                        .map(|pos| &background_image_path_residual_bytes[..=pos]) // include the last non-zero byte
                        .unwrap_or(background_image_path_residual_bytes)
                        .to_vec(),
                )
            },
            unknown2: unknown2
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            cutscene_animations: animations,
            unknown3: unknown3.clone(),
            unknown3_as_u16s: unknown3
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            unknown3_as_u32s: unknown3
                .clone()
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
            hex,
        }))
    }

    fn read_cutscene_animation(&mut self, buf: &[u8]) -> Result<CutsceneAnimation, DecodeError> {
        // 16 bytes for enabled, unknown1, position, and path.
        const PATH_OFFSET_END: usize = 16 + SAVE_GAME_ASSET_PATH_SIZE_BYTES;

        Ok(CutsceneAnimation {
            enabled: u32::from_le_bytes(buf[0..4].try_into().unwrap()) != 0,
            unknown1: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            position: UVec2::new(
                u32::from_le_bytes(buf[8..12].try_into().unwrap()),
                u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            ),
            path: self.read_string(&buf[16..PATH_OFFSET_END])?,
            unknown2: u32::from_le_bytes(
                buf[PATH_OFFSET_END..PATH_OFFSET_END + 4]
                    .try_into()
                    .unwrap(),
            ),
            unknown3: u32::from_le_bytes(
                buf[PATH_OFFSET_END + 4..PATH_OFFSET_END + 8]
                    .try_into()
                    .unwrap(),
            ),
            sprite_count: u32::from_le_bytes(
                buf[PATH_OFFSET_END + 8..PATH_OFFSET_END + 12]
                    .try_into()
                    .unwrap(),
            ),
            frame_duration_millis: u32::from_le_bytes(
                buf[PATH_OFFSET_END + 12..PATH_OFFSET_END + 16]
                    .try_into()
                    .unwrap(),
            ),
        })
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

        let small_disabled_banner_path_buf = &buf[82..114];
        let (small_disabled_banner_path_buf, small_disabled_banner_path_remainder) =
            small_disabled_banner_path_buf
                .iter()
                .enumerate()
                .find(|(_, &b)| b == 0)
                .map(|(i, _)| small_disabled_banner_path_buf.split_at(i + 1))
                .unwrap_or((small_disabled_banner_path_buf, &[]));

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
            small_disabled_banner_path: self.read_string(small_disabled_banner_path_buf)?,
            small_disabled_banner_path_remainder: small_disabled_banner_path_remainder.to_vec(),
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
            flags: RegimentFlags::from_bits(status_u16)
                .ok_or(DecodeError::InvalidRegimentFlags(status_u16))?,
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
                display_name: self.read_string(&buf[22..54])?,
                display_name_id: u16::from_le_bytes(buf[54..56].try_into().unwrap()),
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
                unknown2: buf[79..83].try_into().unwrap(),
                unknown2_a: u16::from_le_bytes(buf[79..81].try_into().unwrap()),
                unknown2_b: u16::from_le_bytes(buf[81..83].try_into().unwrap()),
                unknown2_as_u32: u32::from_le_bytes(buf[79..83].try_into().unwrap()),
            },
            unknown4: buf[83],
            leader_profile: UnitProfile {
                sprite_sheet_index: u16::from_le_bytes(buf[84..86].try_into().unwrap()),
                display_name: self.read_string(&buf[86..118])?,
                display_name_id: u16::from_le_bytes(buf[118..120].try_into().unwrap()),
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
                unknown2: buf[142..146].try_into().unwrap(),
                unknown2_a: u16::from_le_bytes(buf[142..144].try_into().unwrap()),
                unknown2_b: u16::from_le_bytes(buf[144..146].try_into().unwrap()),
                unknown2_as_u32: u32::from_le_bytes(buf[142..146].try_into().unwrap()),
            },
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
            unknown1: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
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
