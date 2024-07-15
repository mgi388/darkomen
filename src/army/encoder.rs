use super::*;
use decoder::{FORMAT, REGIMENT_SIZE_BYTES};
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
        self.writer.write_all(&army.save_file_header)?;
        self.write_header(army)?;
        self.write_regiments(army)?;
        self.writer.write_all(&army.save_file_footer)?;
        Ok(())
    }

    fn write_header(&mut self, army: &Army) -> Result<(), EncodeError> {
        self.writer.write_all(&FORMAT.to_le_bytes())?;
        self.writer
            .write_all(&(army.regiments.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(REGIMENT_SIZE_BYTES as u32).to_le_bytes())?;
        self.writer.write_all(&[Into::<u8>::into(army.race)])?;
        self.writer.write_all(&army.unknown1)?;
        self.writer
            .write_all(&army.default_name_index.to_le_bytes())?;
        self.write_string(&army.name)?;
        self.writer.write_all(&army.name_remainder)?;
        self.write_string(&army.small_banner_path)?;
        self.writer.write_all(&army.small_banner_path_remainder)?;
        self.write_string(&army.small_banner_disabled_path)?;
        self.writer
            .write_all(&army.small_banner_disabled_path_remainder)?;
        self.write_string(&army.large_banner_path)?;
        self.writer.write_all(&army.large_banner_path_remainder)?;
        self.writer
            .write_all(&army.last_battle_gold.to_le_bytes())?;
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
        self.writer
            .write_all(&Into::<u16>::into(r.status).to_le_bytes())?;
        self.writer.write_all(&r.unknown1)?;
        self.writer.write_all(&r.id.to_le_bytes())?;
        self.writer.write_all(&r.unknown2)?;
        self.writer.write_all(&[Into::<u8>::into(r.mage_class)])?;
        self.writer.write_all(&[r.max_armor])?;
        self.writer.write_all(&r.cost.to_le_bytes())?;
        self.writer.write_all(&r.banner_index.to_le_bytes())?;
        self.writer.write_all(&r.unknown3)?;
        self.writer.write_all(&r.attributes.bits().to_le_bytes())?;
        self.write_unit_profile(&r.unit_profile)?;
        self.writer.write_all(&[r.unknown4])?;
        self.writer.write_all(&r.unknown5)?;
        self.write_unit_profile(&r.leader_profile)?;
        self.writer.write_all(&r.unknown6)?;
        self.writer.write_all(&r.leader_head_id.to_le_bytes())?;
        self.write_last_battle_stats(&r.last_battle_stats)?;
        self.writer.write_all(&r.total_experience.to_le_bytes())?;
        self.writer.write_all(&[r.duplicate_id])?;
        self.writer.write_all(&[r.min_armor])?;
        self.writer
            .write_all(&Into::<u16>::into(r.magic_book).to_le_bytes())?;
        self.writer.write_all(&r.magic_items[0].to_le_bytes())?;
        self.writer.write_all(&r.magic_items[1].to_le_bytes())?;
        self.writer.write_all(&r.magic_items[2].to_le_bytes())?;
        self.writer.write_all(&r.unknown8)?;
        self.writer.write_all(&[r.purchased_armor])?;
        self.writer.write_all(&[r.max_purchasable_armor])?;
        self.writer.write_all(&[r.repurchased_troop_count])?;
        self.writer.write_all(&[r.max_purchasable_troop_count])?;
        self.writer.write_all(&r.book_profile)?;

        Ok(())
    }

    fn write_unit_profile(&mut self, u: &UnitProfile) -> Result<(), EncodeError> {
        self.writer.write_all(&u.sprite_index.to_le_bytes())?;
        self.write_string_with_limit(&u.name, 32)?;
        self.writer.write_all(&u.name_id.to_le_bytes())?;
        self.writer.write_all(&[Into::<u8>::into(u.alignment)])?;
        self.writer.write_all(&[u.max_troop_count])?;
        self.writer.write_all(&[u.alive_troop_count])?;
        self.writer.write_all(&[u.rank_count])?;
        self.writer.write_all(&u.unknown1)?;
        self.writer.write_all(&[u.stats.movement])?;
        self.writer.write_all(&[u.stats.weapon_skill])?;
        self.writer.write_all(&[u.stats.ballistic_skill])?;
        self.writer.write_all(&[u.stats.strength])?;
        self.writer.write_all(&[u.stats.toughness])?;
        self.writer.write_all(&[u.stats.wounds])?;
        self.writer.write_all(&[u.stats.initiative])?;
        self.writer.write_all(&[u.stats.attacks])?;
        self.writer.write_all(&[u.stats.leadership])?;
        self.writer.write_all(&[Into::<u8>::into(u.mount)])?;
        self.writer.write_all(&[u.armor])?;
        self.writer.write_all(&[Into::<u8>::into(u.weapon)])?;
        self.writer.write_all(&[Into::<u8>::into(u.class)])?;
        self.writer.write_all(&[u.point_value])?;
        self.writer.write_all(&[Into::<u8>::into(u.projectile)])?;

        Ok(())
    }

    fn write_last_battle_stats(&mut self, s: &LastBattleStats) -> Result<(), EncodeError> {
        self.writer.write_all(&s.unit_killed_count.to_le_bytes())?;
        self.writer.write_all(&s.unknown1)?;
        self.writer.write_all(&s.kill_count.to_le_bytes())?;
        self.writer.write_all(&s.experience.to_le_bytes())?;

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
