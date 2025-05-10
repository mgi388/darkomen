use std::{
    ffi::CString,
    io::{BufWriter, Write},
};

use encoding_rs::WINDOWS_1252;

use crate::army::decoder::*;

use super::*;

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
        self.maybe_write_save_game_header(army)?;
        self.write_header(army)?;
        self.write_regiments(army)?;
        self.maybe_write_save_game_footer(army)?;
        Ok(())
    }

    fn write_script_state(&mut self, s: &ScriptState) -> Result<(), EncodeError> {
        self.writer.write_all(&s.program_counter.to_le_bytes())?;
        self.writer.write_all(&s.unknown0.to_le_bytes())?;
        self.writer
            .write_all(&s.base_execution_address.to_le_bytes())?;
        self.writer.write_all(&s.unknown_address.to_le_bytes())?;
        self.writer.write_all(&s.local_variable.to_le_bytes())?;
        self.writer.write_all(&s.unknown1.to_le_bytes())?;
        self.writer.write_all(&s.stack_pointer.to_le_bytes())?;
        self.writer.write_all(&s.unknown2)?;
        self.writer
            .write_all(&s.execution_offset_index.to_le_bytes())?;
        self.writer.write_all(&s.unknown3.to_le_bytes())?;
        self.writer.write_all(&s.nest_if.to_le_bytes())?;
        self.writer.write_all(&s.nest_gosub.to_le_bytes())?;
        self.writer.write_all(&s.nest_loop.to_le_bytes())?;
        self.writer.write_all(&s.unknown4.to_le_bytes())?;
        self.writer.write_all(&s.elapsed_millis.to_le_bytes())?;
        self.writer.write_all(&s.unknown5.to_le_bytes())?;
        self.writer.write_all(&s.unknown6.to_le_bytes())?;
        self.writer.write_all(&s.unknown7)?;

        Ok(())
    }

    fn write_script_variables(&mut self, v: &ScriptVariables) -> Result<(), EncodeError> {
        self.writer.write_all(
            &(if v.bogenhafen_mission_completed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.goblin_camp_or_ragnar_troll_mission_accepted {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.goblin_camp_mission_accepted {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.ragnar_troll_mission_accepted {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.vingtienne_or_treeman_mission_accepted {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.vingtienne_mission_accepted {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.treeman_mission_accepted {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.count_carstein_destroyed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.hand_of_nagash_destroyed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer
            .write_all(&(if v.black_grail_destroyed { 1u32 } else { 0u32 }).to_le_bytes())?;
        self.writer.write_all(
            &(if v.bogenhafen_mission_failed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.helmgart_mission_victorious {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.troll_country_mission_victorious {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer
            .write_all(&(if v.loren_king_met { 1u32 } else { 0u32 }).to_le_bytes())?;
        self.writer.write_all(
            &(if v.axebite_mission_completed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.wood_elf_glade_guards_destroyed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(
            &(if v.imperial_steam_tank_destroyed {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer.write_all(&v.unknown1.to_le_bytes())?;
        self.writer.write_all(&v.unknown2.to_le_bytes())?;
        self.writer.write_all(&v.unknown3.to_le_bytes())?;
        self.writer.write_all(&v.unknown4.to_le_bytes())?;
        self.writer.write_all(&v.meet_action.to_le_bytes())?;
        self.writer.write_all(
            &(if v.meet_continue_or_replay_selected {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer
            .write_all(&(if v.heroic_choice_made { 1u32 } else { 0u32 }).to_le_bytes())?;

        Ok(())
    }

    fn maybe_write_save_game_header(&mut self, army: &Army) -> Result<(), EncodeError> {
        let Some(header) = army.save_game_header.as_ref() else {
            return Ok(());
        };

        self.write_padded_string(
            &header.display_name,
            header.display_name_residual_bytes.as_ref(),
            SAVE_GAME_DISPLAY_NAME_SIZE_BYTES,
        )?;

        self.write_padded_string(
            &header.suggested_display_name,
            header.suggested_display_name_residual_bytes.as_ref(),
            SAVE_GAME_DISPLAY_NAME_SIZE_BYTES,
        )?;

        self.writer
            .write_all(&(if header.unknown_bool1 { 1u32 } else { 0u32 }).to_le_bytes())?;
        self.writer
            .write_all(&(if header.unknown_bool2 { 1u32 } else { 0u32 }).to_le_bytes())?;

        self.write_script_state(&header.script_state)?;

        self.write_script_variables(&header.script_variables)?;

        Ok(())
    }

    fn maybe_write_save_game_footer(&mut self, army: &Army) -> Result<(), EncodeError> {
        let Some(footer) = army.save_game_footer.as_ref() else {
            return Ok(());
        };

        self.writer.write_all(&footer.unknown1)?;

        for o in &footer.objectives {
            self.writer.write_all(&o.unknown1.to_le_bytes())?;
            self.writer.write_all(&o.id.to_le_bytes())?;
            self.writer.write_all(&o.unknown2.to_le_bytes())?;
            self.writer.write_all(&o.result.to_le_bytes())?;
            self.writer.write_all(&o.unknown4.to_le_bytes())?;
            self.writer.write_all(&o.unknown5.to_le_bytes())?;
        }

        for v in footer.travel_path_history.iter() {
            self.writer.write_all(&v.to_le_bytes())?;
        }
        self.writer.write_all(&u32::MAX.to_le_bytes().repeat(
            TRAVEL_PATH_HISTORY_CAPACITY.saturating_sub(footer.travel_path_history.len()),
        ))?;

        let background_image_path = footer.background_image_path.as_ref().map_or("", |s| s);
        self.write_padded_string(
            background_image_path,
            footer.background_image_path_residual_bytes.as_ref(),
            SAVE_GAME_ASSET_PATH_SIZE_BYTES,
        )?;

        self.writer.write_all(&footer.unknown2.to_le_bytes())?;
        self.writer
            .write_all(&footer.victory_message_index.to_le_bytes())?;
        self.writer
            .write_all(&footer.defeat_message_index.to_le_bytes())?;
        self.writer.write_all(&footer.rng_seed.to_le_bytes())?;

        for s in &footer.meet_animated_sprites {
            self.write_meet_animated_sprite(s)?;
        }

        self.writer.write_all(&footer.unknown3)?;

        Ok(())
    }

    fn write_meet_animated_sprite(&mut self, s: &MeetAnimatedSprite) -> Result<(), EncodeError> {
        self.writer
            .write_all(&(if s.enabled { 1u32 } else { 0u32 }).to_le_bytes())?;
        self.writer.write_all(&s.unknown1.to_le_bytes())?;
        self.writer.write_all(&s.position.x.to_le_bytes())?;
        self.writer.write_all(&s.position.y.to_le_bytes())?;
        self.write_string_with_limit(&s.path, SAVE_GAME_ASSET_PATH_SIZE_BYTES)?;
        self.writer.write_all(&s.unknown2.to_le_bytes())?;
        self.writer.write_all(&s.unknown3.to_le_bytes())?;
        self.writer.write_all(&s.sprite_count.to_le_bytes())?;
        self.writer
            .write_all(&s.frame_duration_millis.to_le_bytes())?;

        Ok(())
    }

    fn write_header(&mut self, army: &Army) -> Result<(), EncodeError> {
        self.writer.write_all(&FORMAT.to_le_bytes())?;
        self.writer
            .write_all(&(army.regiments.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(REGIMENT_SIZE_BYTES as u32).to_le_bytes())?;
        self.writer.write_all(&army.race.bits().to_le_bytes())?;
        self.writer.write_all(&army.unknown1)?;
        self.writer
            .write_all(&army.default_name_index.to_le_bytes())?;
        self.write_string(&army.name)?;
        self.writer.write_all(&army.name_remainder)?;
        self.write_string(&army.small_banners_path)?;
        self.writer.write_all(&army.small_banners_path_remainder)?;
        self.write_string(&army.disabled_small_banners_path)?;
        self.writer
            .write_all(&army.disabled_small_banners_path_remainder)?;
        self.write_string(&army.large_banners_path)?;
        self.writer.write_all(&army.large_banners_path_remainder)?;
        self.writer
            .write_all(&army.last_battle_captured_gold.to_le_bytes())?;
        self.writer.write_all(&army.total_gold.to_le_bytes())?;
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
        self.writer.write_all(&r.flags.bits().to_le_bytes())?;
        self.writer.write_all(&r.unknown1)?;
        self.writer.write_all(&r.id.to_le_bytes())?;
        self.writer.write_all(&[Into::<u8>::into(r.mage_class)])?;
        self.writer.write_all(&[r.max_armor])?;
        self.writer.write_all(&r.cost.to_le_bytes())?;
        self.writer
            .write_all(&r.banner_sprite_sheet_index.to_le_bytes())?;
        self.writer.write_all(&r.unknown3)?;
        self.writer.write_all(&r.attributes.bits().to_le_bytes())?;
        self.write_unit_profile(&r.unit_profile)?;
        self.writer.write_all(&[r.unknown4])?;
        self.write_unit_profile(&r.leader_profile)?;
        self.writer.write_all(&r.leader_head_id.to_le_bytes())?;
        self.write_last_battle_stats(&r.last_battle_stats)?;
        self.writer.write_all(&r.total_experience.to_le_bytes())?;
        self.writer.write_all(&[r.duplicate_id])?;
        self.writer.write_all(&[r.min_armor])?;
        self.writer
            .write_all(&Into::<u16>::into(r.spell_book).to_le_bytes())?;
        self.writer.write_all(&r.magic_items[0].to_le_bytes())?;
        self.writer.write_all(&r.magic_items[1].to_le_bytes())?;
        self.writer.write_all(&r.magic_items[2].to_le_bytes())?;
        self.writer.write_all(&r.spells[0].to_le_bytes())?;
        self.writer.write_all(&r.spells[1].to_le_bytes())?;
        self.writer.write_all(&r.spells[2].to_le_bytes())?;
        self.writer.write_all(&r.spells[3].to_le_bytes())?;
        self.writer.write_all(&r.spells[4].to_le_bytes())?;
        self.writer
            .write_all(&r.last_battle_captured_gold.to_le_bytes())?;
        self.writer.write_all(&[r.purchased_armor])?;
        self.writer.write_all(&[r.max_purchasable_armor])?;
        self.writer.write_all(&[r.repurchased_unit_count])?;
        self.writer.write_all(&[r.max_purchasable_unit_count])?;
        self.writer.write_all(&r.book_profile_index.to_le_bytes())?;

        Ok(())
    }

    fn write_unit_profile(&mut self, u: &UnitProfile) -> Result<(), EncodeError> {
        self.writer.write_all(&u.sprite_sheet_index.to_le_bytes())?;
        self.write_string_with_limit(&u.display_name, 32)?;
        self.writer.write_all(&u.display_name_index.to_le_bytes())?;
        self.writer.write_all(&[Into::<u8>::into(u.alignment)])?;
        self.writer.write_all(&[u.max_unit_count])?;
        self.writer.write_all(&[u.alive_unit_count])?;
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
        self.writer.write_all(&[Into::<u8>::into(u.mount_class)])?;
        self.writer.write_all(&[u.armor])?;
        self.writer.write_all(&[Into::<u8>::into(u.weapon_class)])?;
        self.writer.write_all(&[Into::<u8>::into(u.class)])?;
        self.writer.write_all(&[u.point_value])?;
        self.writer
            .write_all(&[Into::<u8>::into(u.projectile_class)])?;
        self.writer.write_all(&u.unknown2)?;

        Ok(())
    }

    fn write_last_battle_stats(&mut self, s: &LastBattleStats) -> Result<(), EncodeError> {
        self.writer.write_all(&s.unit_killed_count.to_le_bytes())?;
        self.writer.write_all(&s.unknown1.to_le_bytes())?;
        self.writer.write_all(&s.kill_count.to_le_bytes())?;
        self.writer.write_all(&s.experience.to_le_bytes())?;

        Ok(())
    }

    fn write_padded_string(
        &mut self,
        s: &str,
        residual_bytes: Option<&Vec<u8>>,
        total_size: usize,
    ) -> Result<(), EncodeError> {
        let bytes_written = self.write_string(s)?;

        if let Some(residual) = residual_bytes {
            let padding_size = total_size - (bytes_written + residual.len());
            let padding = vec![0; padding_size];
            self.writer.write_all(residual)?;
            self.writer.write_all(&padding)?;
        } else {
            let padding_size = total_size - bytes_written;
            let padding = vec![0; padding_size];
            self.writer.write_all(&padding)?;
        }

        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<usize, EncodeError> {
        let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(s);

        let c_string = CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
        let bytes = c_string.as_bytes_with_nul();

        self.writer.write_all(bytes)?;

        Ok(bytes.len())
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
