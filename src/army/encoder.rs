use super::*;
use decoder::{FORMAT, REGIMENT_BLOCK_SIZE};
use encoding_rs::WINDOWS_1252;
use std::{
    ffi::CString,
    io::{BufWriter, Write},
};

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
    StringTooLong,
}

impl std::error::Error for EncodeError {}

impl From<std::io::Error> for EncodeError {
    fn from(err: std::io::Error) -> Self {
        EncodeError::IoError(err)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {}", e),
            EncodeError::InvalidString => write!(f, "invalid string"),
            EncodeError::StringTooLong => write!(f, "string too long"),
        }
    }
}

#[derive(Debug)]
pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder {
            writer: BufWriter::new(writer),
        }
    }

    pub fn encode(&mut self, army: &Army) -> Result<(), EncodeError> {
        self.write_header(army)?;
        self.write_regiments(army)?;
        Ok(())
    }

    fn write_header(&mut self, army: &Army) -> Result<(), EncodeError> {
        // TODO: Ignoring save file header.

        let race: u8 = army.race.into();

        self.writer.write_all(&FORMAT.to_le_bytes())?;
        self.writer
            .write_all(&(army.regiments.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(REGIMENT_BLOCK_SIZE as u32).to_le_bytes())?;
        self.writer.write_all(&[race])?;
        self.writer.write_all(&army.unknown1)?;
        self.writer.write_all(&army.unknown2)?;
        self.write_string(&army.small_banner_path)?;
        self.writer.write_all(&army.small_banner_path_remainder)?;
        self.write_string(&army.small_banner_disabled_path)?;
        self.writer
            .write_all(&army.small_banner_disabled_path_remainder)?;
        self.write_string(&army.large_banner_path)?;
        self.writer.write_all(&army.large_banner_path_remainder)?;
        self.writer
            .write_all(&army.gold_from_treasures.to_le_bytes())?;
        self.writer.write_all(&army.gold_in_coffers.to_le_bytes())?;
        self.writer.write_all(&army.magic_items)?;
        self.writer.write_all(&army.unknown3)?;

        self.writer.flush()?;

        Ok(())
    }

    fn write_regiments(&mut self, army: &Army) -> Result<(), EncodeError> {
        for regiment in &army.regiments {
            self.write_regiment(regiment)?;
        }

        Ok(())
    }

    fn write_regiment(&mut self, r: &Regiment) -> Result<(), EncodeError> {
        let alignment: u8 = r.alignment.into();
        let mount: u8 = r.mount.into();
        let magic_book: u16 = r.magic_book.into();

        self.writer.write_all(&r.status)?;
        self.writer.write_all(&r.unknown1)?;
        self.writer.write_all(&r.id.to_le_bytes())?;
        self.writer.write_all(&r.unknown2)?;
        self.writer.write_all(&[r.wizard_type])?;
        self.writer.write_all(&[r.max_armor])?;
        self.writer.write_all(&r.cost.to_le_bytes())?;
        self.writer.write_all(&r.banner_index.to_le_bytes())?;
        self.writer.write_all(&r.unknown3)?;
        self.writer.write_all(&r.regiment_attributes)?;
        self.writer.write_all(&r.sprite_index.to_le_bytes())?;
        self.write_string_with_limit(&r.name, 32)?;
        self.writer.write_all(&r.name_id.to_le_bytes())?;
        self.writer.write_all(&[alignment])?;
        self.writer.write_all(&[r.max_troops])?;
        self.writer.write_all(&[r.alive_troops])?;
        self.writer.write_all(&[r.ranks])?;
        self.writer.write_all(&r.unknown4)?;
        self.writer.write_all(&[r.troop_attributes.movement])?;
        self.writer.write_all(&[r.troop_attributes.weapon_skill])?;
        self.writer
            .write_all(&[r.troop_attributes.ballistic_skill])?;
        self.writer.write_all(&[r.troop_attributes.strength])?;
        self.writer.write_all(&[r.troop_attributes.toughness])?;
        self.writer.write_all(&[r.troop_attributes.wounds])?;
        self.writer.write_all(&[r.troop_attributes.initiative])?;
        self.writer.write_all(&[r.troop_attributes.attacks])?;
        self.writer.write_all(&[r.troop_attributes.leadership])?;
        self.writer.write_all(&[mount])?;
        self.writer.write_all(&[r.armor])?;
        self.writer.write_all(&[r.weapon])?;
        self.writer.write_all(&[r.encode_class()])?;
        self.writer.write_all(&[r.point_value])?;
        self.writer.write_all(&[r.missile_weapon])?;
        self.writer.write_all(&[r.unknown5])?;
        self.writer.write_all(&r.unknown6)?;
        self.writer
            .write_all(&r.leader.sprite_index.to_le_bytes())?;
        self.write_string_with_limit(&r.leader.name, 32)?;
        self.writer.write_all(&r.leader.name_remainder)?;
        self.writer.write_all(&[r.leader.attributes.movement])?;
        self.writer.write_all(&[r.leader.attributes.weapon_skill])?;
        self.writer
            .write_all(&[r.leader.attributes.ballistic_skill])?;
        self.writer.write_all(&[r.leader.attributes.strength])?;
        self.writer.write_all(&[r.leader.attributes.toughness])?;
        self.writer.write_all(&[r.leader.attributes.wounds])?;
        self.writer.write_all(&[r.leader.attributes.initiative])?;
        self.writer.write_all(&[r.leader.attributes.attacks])?;
        self.writer.write_all(&[r.leader.attributes.leadership])?;
        self.writer.write_all(&[r.leader.mount])?;
        self.writer.write_all(&[r.leader.armor])?;
        self.writer.write_all(&[r.leader.weapon])?;
        self.writer.write_all(&[r.leader.unit_type])?;
        self.writer.write_all(&[r.leader.point_value])?;
        self.writer.write_all(&[r.leader.missile_weapon])?;
        self.writer.write_all(&r.leader.unknown1)?;
        self.writer.write_all(&r.leader.head_id.to_le_bytes())?;
        self.writer.write_all(&r.leader.x)?;
        self.writer.write_all(&r.leader.y)?;
        self.writer.write_all(&r.experience.to_le_bytes())?;
        self.writer.write_all(&[r.duplicate_id])?;
        self.writer.write_all(&[r.min_armor])?;
        self.writer.write_all(&magic_book.to_le_bytes())?;
        self.writer.write_all(&r.magic_items[0].to_le_bytes())?;
        self.writer.write_all(&r.magic_items[1].to_le_bytes())?;
        self.writer.write_all(&r.magic_items[2].to_le_bytes())?;
        self.writer.write_all(&r.unknown7)?;
        self.writer.write_all(&[r.purchased_armor])?;
        self.writer.write_all(&[r.max_purchasable_armor])?;
        self.writer.write_all(&[r.repurchased_troops])?;
        self.writer.write_all(&[r.max_purchasable_troops])?;
        self.writer.write_all(&r.book_profile)?;

        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<(), EncodeError> {
        let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(s);

        let c_string = CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
        let bytes = c_string.as_bytes_with_nul();

        self.writer.write_all(bytes)?;

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

        let padding_size = limit - bytes.len();
        let padding = vec![0; padding_size];
        self.writer.write_all(&padding)?;

        Ok(())
    }
}
