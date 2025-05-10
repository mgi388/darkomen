mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use derive_more::derive::{Display, Error, From};
use glam::UVec2;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct ScriptState {
    pub program_counter: u32,
    ///  - 0x13 (19u32) is saved after a cutscene (which could also be just
    ///    before a battle would start).
    ///  - 0x3A (58u32) is saved at the victory screen.
    pub unknown0: u32,
    /// The base address in the script where the campaign should start executing
    /// from. In the English version of the game, this is `0x4C3C48`. In the
    /// German version of the game, this is `0x4C3D90`. Combine with
    /// `execution_offset_index
    /// * 4` to get the address to start executing from.
    pub base_execution_address: u32,
    /// In the English version of the game, this is `0x4CCD28`. In the German
    /// version of the game, this is `0x4CCE70`.
    pub unknown_address: u32,
    /// Initialized to 0 on init.
    pub local_variable: u32,
    /// Initialized to 1 on init.
    pub unknown1: u32,
    /// Initialized to 20 on init.
    pub stack_pointer: u32,
    unknown2: Vec<u8>,
    unknown2_hex: Vec<String>,  // TODO: Remove, debug only.
    unknown2_as_u32s: Vec<u32>, // TODO: Remove, debug only.
    /// The offset index to add to the base execution address to get the address
    /// to start executing from. To account for alignment, multiply this value
    /// by 4 before adding it to the base address.
    pub execution_offset_index: u32,
    pub unknown3: u32,
    /// Initialized to 0 on init.
    pub nest_if: u32,
    /// Initialized to 0 on init.
    pub nest_gosub: u32,
    /// Initialized to 0 on init.
    pub nest_loop: u32,
    pub unknown4: u32,
    /// Initialized to the current tick count on init, which is the number of
    /// milliseconds since the operating system started. The purpose of this is
    /// not yet known.
    pub elapsed_millis: u32,
    /// Initialized to 0 on init.
    pub unknown5: u32,
    /// Initialized to 0 on init.
    pub unknown6: u32,
    unknown7: Vec<u8>,
    unknown7_hex: Vec<String>,  // TODO: Remove, debug only.
    unknown7_as_u32s: Vec<u32>, // TODO: Remove, debug only.
}

impl ScriptState {
    /// Returns the address to start executing from.
    pub fn execution_address(&self) -> u32 {
        self.base_execution_address + self.execution_offset_index * 4
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct ScriptVariables {
    /// Bogenhafen mission completed. This does not mean the mission was won or
    /// lost, just that it was completed.
    ///
    /// This is variable 0 in the WHMTG script.
    pub bogenhafen_mission_completed: bool,
    /// Attacked either the mission to attack the Goblin camp or the mission to
    /// help Ragnar fight the trolls.
    ///
    /// This is variable 1 in the WHMTG script.
    pub goblin_camp_or_ragnar_troll_mission_accepted: bool,
    /// Accepted the mission to attack the Goblin camp together with Munz.
    ///
    /// This is variable 2 in the WHMTG script.
    pub goblin_camp_mission_accepted: bool,
    /// Accepted the mission to help Ragnar fight the trolls.
    ///
    /// This is variable 3 in the WHMTG script.
    pub ragnar_troll_mission_accepted: bool,
    /// Accepted either the mission to attack the Greenskins near Vingtienne or
    /// the mission to help Treeman in Loren Lake.
    ///
    /// This is variable 4 in the WHMTG script.
    pub vingtienne_or_treeman_mission_accepted: bool,
    /// Accepted the mission to attack the Greenskins near Vingtienne.
    ///
    /// This is variable 5 in the WHMTG script.
    pub vingtienne_mission_accepted: bool,
    /// Accepted the mission to help Treeman in Loren Lake.
    ///
    /// This is variable 6 in the WHMTG script.
    pub treeman_mission_accepted: bool,
    /// Manfred von Carstein defeated.
    ///
    /// This is variable 7 in the WHMTG script.
    pub count_carstein_destroyed: bool,
    /// Hand of Nagash defeated.
    ///
    /// This is variable 8 in the WHMTG script.
    pub hand_of_nagash_destroyed: bool,
    /// Black Grail defeated.
    ///
    /// This is variable 9 in the WHMTG script.
    pub black_grail_destroyed: bool,
    /// Set to true if the enemy was victorious in the Bogenhafen mission.
    ///
    /// This is variable 10 in the WHMTG script.
    pub bogenhafen_mission_failed: bool,
    /// Helmgart mission played.
    ///
    /// This is variable 11 in the WHMTG script.
    pub helmgart_mission_victorious: bool,
    /// Helped Ragnar defeat the trolls.
    ///
    /// This is variable 12 in the WHMTG script.
    pub troll_country_mission_victorious: bool,
    /// Talked with King Orion (Woodelf King).
    ///
    /// This is variable 13 in the WHMTG script.
    pub loren_king_met: bool,
    /// Helped Azkuz move through the Axebite Pass. This does not mean the
    /// mission was won or lost, just that it was completed.
    ///
    /// This is variable 14 in the WHMTG script.
    pub axebite_mission_completed: bool,
    /// The Wood Elf Glade Guards are destroyed.
    ///
    /// This is variable 15 in the WHMTG script.
    pub wood_elf_glade_guards_destroyed: bool,
    /// The Imperial Steam Tank is destroyed.
    ///
    /// This is variable 16 in the WHMTG script.
    pub imperial_steam_tank_destroyed: bool,
    /// This is variable 17 in the WHMTG script.
    pub unknown1: u32,
    /// This is variable 18 in the WHMTG script.
    pub unknown2: u32,
    /// This is variable 19 in the WHMTG script.
    pub unknown3: u32,
    /// This is variable 20 in the WHMTG script.
    pub unknown4: u32,
    /// Meet action selected by the player.
    ///
    /// This is variable 21 in the WHMTG script.
    pub meet_action: u32,
    /// Indicates if the player selected to either continue campaign or replay
    /// meet. Set to true when player selects either "Continue Campaign" or
    /// "Replay Meet" in a meeting.
    ///
    /// This is variable 22 in the WHMTG script.
    pub meet_continue_or_replay_selected: bool,
    /// Indicates the player's decision at choice points in meets.
    ///
    /// - `true` = Player chose the "positive/heroic" option (first choice).
    /// - `false` = Player chose the "cautious/pragmatic" option (second
    ///   choice).
    ///
    /// Note: The value is inverted from the choice index. For example, for the
    /// "Stay and fight" and "Continue to Helmgart" choices, the value is `true`
    /// for the first choice and `false` for the second choice.
    ///
    /// This is variable 23 in the WHMTG script.
    pub heroic_choice_made: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct SaveGameHeader {
    /// The name displayed when loading the save game.
    pub display_name: String,
    /// The original game writes over the existing display name with the new
    /// path but the old bytes are not cleared first. This field is used to
    /// store the residual bytes, if there are any. If it's `None` then there
    /// are no residual bytes / all bytes are zero after the null-terminated
    /// string. If it's `Some`, then it contains the residual bytes, up to, but
    /// not including, the last nul-terminated string.
    display_name_residual_bytes: Option<Vec<u8>>,
    /// The name suggested when saving the game.
    pub suggested_display_name: String,
    /// The original game writes over the existing suggested display name with
    /// the new path but the old bytes are not cleared first. This field is used
    /// to store the residual bytes, if there are any. If it's `None` then there
    /// are no residual bytes / all bytes are zero after the null-terminated
    /// string. If it's `Some`, then it contains the residual bytes, up to, but
    /// not including, the last nul-terminated string.
    suggested_display_name_residual_bytes: Option<Vec<u8>>,
    pub unknown_bool1: bool,
    pub unknown_bool2: bool,
    script_state_hex: Vec<String>,  // TODO: Remove, debug only.
    script_state_as_u32s: Vec<u32>, // TODO: Remove, debug only.
    /// The script state of the save game. Used by the WHMTG scripting engine to
    /// run the next part of the campaign.
    pub script_state: ScriptState,
    /// The script variables set throughout the campaign. Useful for making
    /// decisions in the campaign.
    pub script_variables: ScriptVariables,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct MeetAnimatedSprite {
    /// Whether the animated sprite is enabled.
    pub enabled: bool,
    unknown1: u32,
    /// The position the top-left corner of the animated sprite should be placed
    /// on the screen.
    pub position: UVec2,
    /// The path to the sprite sheet file, e.g., "[SPRITES]\m_empbi1.spr".
    pub path: String,
    unknown2: u32,
    unknown3: u32,
    /// The number of sprites in the sprite sheet / the number of frames in the
    /// animated sprite.
    pub sprite_count: u32,
    /// The duration, in milliseconds, to display each frame of the animated
    /// sprite.
    pub frame_duration_millis: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Objective {
    pub unknown1: i32,
    /// The ID of the objective.
    ///
    /// Interesting IDs:
    ///
    /// - 1: Indicates if the enemy was victorious. When `result` is 1, the
    ///   enemy won the battle. When `result` is 0, the player won the battle.
    pub id: i32,
    pub unknown2: i32,
    /// The result of the objective.
    pub result: i32,
    pub unknown4: i32,
    pub unknown5: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct SaveGameFooter {
    unknown1: Vec<u8>,
    unknown1_as_u16s: Vec<u16>, // TODO: Remove, debug only.
    unknown1_as_u32s: Vec<u32>, // TODO: Remove, debug only.
    pub objectives: Vec<Objective>,
    /// A history of path indices the player has traveled, accumulated across
    /// travel map screens to display the full journey, e.g., from Altdorf, over
    /// the Black Mountains, through Teufelbad and to the current location.
    /// Reset by game scripts on events like map changes or new chapters.
    pub travel_path_history: Vec<i32>,
    /// The path to the background image file, e.g., "[PICTURES]\m_empn.bmp".
    pub background_image_path: Option<String>,
    /// The original game writes over the existing background image path with
    /// the new path but the old bytes are not cleared first. This field is used
    /// to store the residual bytes, if there are any. If it's `None` then there
    /// are no residual bytes / all bytes are zero after the null-terminated
    /// string. If it's `Some`, then it contains the residual bytes, up to, but
    /// not including, the last nul-terminated string.
    background_image_path_residual_bytes: Option<Vec<u8>>,
    /// Always 0.
    unknown2: u32,
    /// The index into the list of battle debrief messages found in ENGREL.EXE
    /// for the case where the player wins the battle.
    ///
    /// This is used to display a message to the player after winning a battle.
    pub victory_message_index: u32,
    /// The index into the list of battle debrief messages found in ENGREL.EXE
    /// for the case where the player loses the battle.
    ///
    /// This is used to display a message to the player after losing a battle.
    pub defeat_message_index: u32,
    rng_seed: u32,
    /// A list of aniamted sprites used on the meet screens shown in between
    /// battles.
    pub meet_animated_sprites: Vec<MeetAnimatedSprite>,
    unknown3: Vec<u8>,
    unknown3_as_u16s: Vec<u16>, // TODO: Remove, debug only.
    unknown3_as_u32s: Vec<u32>, // TODO: Remove, debug only.
    hex: Vec<String>,           // TODO: Remove, debug only.
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Army {
    /// An optional save game header if the army is a save game.
    pub save_game_header: Option<SaveGameHeader>,
    /// An optional save game footer if the army is a save game.
    pub save_game_footer: Option<SaveGameFooter>,
    /// The army's race.
    ///
    /// This is used in multiplayer mode to group armies by race.
    pub race: ArmyRace,
    unknown1: [u8; 3], // always seems to be 0, could be padding
    /// The index of the name to use when army name is empty.
    ///
    /// This is used to display the army name in multiplayer mode when no army
    /// name is set.
    pub default_name_index: u16,
    /// The name of the army. Displayed in multiplayer mode.
    pub name: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    name_remainder: Vec<u8>,
    pub small_banners_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    small_banners_path_remainder: Vec<u8>,
    pub disabled_small_banners_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    disabled_small_banners_path_remainder: Vec<u8>,
    disabled_small_banners_path_remainder_as_u16s: Vec<u16>, // TODO: Remove, debug only.
    disabled_small_banners_path_remainder_as_u32s: Vec<u32>, // TODO: Remove, debug only.
    pub large_banners_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    large_banners_path_remainder: Vec<u8>,
    large_banners_path_remainder_as_u16s: Vec<u16>, // TODO: Remove, debug only.
    large_banners_path_remainder_as_u32s: Vec<u32>, // TODO: Remove, debug only.
    /// The amount of gold captured from treasures and earned in the last
    /// battle.
    pub last_battle_captured_gold: u16,
    /// The total amount of gold available to the army for buying new units and
    /// armor.
    pub total_gold: u16,
    /// A list of magic items in the army's inventory.
    ///
    /// Each magic item is an index into the list of magic items. A value of 1
    /// means the Grudgebringer Sword is equipped in that slot. A value of 0
    /// means the army does not have anything in that slot.
    pub magic_items: Vec<u8>,
    unknown3: Vec<u8>,
    pub regiments: Vec<Regiment>,
}

impl Army {
    /// Returns the total amount of gold earned by the army in the last battle.
    ///
    /// The amount of gold earned is calculated by summing the experience of
    /// each regiment in the army and multiplying it by 1.5.
    ///
    /// The total amount is rounded down to the nearest integer.
    pub fn last_battle_earned_gold(&self) -> u32 {
        self.regiments
            .iter()
            .map(|regiment| regiment.last_battle_stats.experience as f32 * 1.5)
            .sum::<f32>() as u32
    }

    /// Returns the total amount of gold captured by the army in the last
    /// battle.
    pub fn last_battle_captured_gold(&self) -> u32 {
        self.regiments
            .iter()
            .map(|regiment| regiment.last_battle_captured_gold)
            .sum::<u16>() as u32
    }

    /// Returns true if the army has any magic items in its inventory.
    pub fn any_magic_items(&self) -> bool {
        self.magic_items.iter().any(|&item| item != 0)
    }

    /// Returns a list of all magic items in the army's inventory.
    pub fn all_magic_items(&self) -> Vec<u8> {
        self.magic_items
            .iter()
            .filter(|&&item| item != 0)
            .copied()
            .collect()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct ArmyRace: u8 {
        /// Empire army.
        const EMPIRE = 0;
        /// Multiplayer army.
        const MULTIPLAYER = 1 << 0;
        /// Greenskins army.
        const GREENSKINS = 1 << 1;
        /// Undead army.
        const UNDEAD = 1 << 2;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Regiment {
    pub flags: RegimentFlags,
    unknown1: [u8; 2],
    pub id: u32,
    pub mage_class: MageClass,
    /// The regiment's maximum level of armor.
    pub max_armor: u8,
    pub cost: u16,
    /// The index into the list of sprite sheet file names found in ENGREL.EXE
    /// for the regiment's banner.
    pub banner_sprite_sheet_index: u16,
    unknown3: [u8; 2],
    pub attributes: RegimentAttributes,
    /// The profile of the regiment's rank and file units.
    pub unit_profile: UnitProfile,
    unknown4: u8,
    /// The profile of the regiment's leader unit.
    ///
    /// Some of the fields are not used for leader units.
    pub leader_profile: UnitProfile,
    /// The leader's 3D head ID.
    pub leader_head_id: i16,

    /// The stats of the regiment's last battle.
    pub last_battle_stats: LastBattleStats,

    /// A number that represents the regiment's total experience.
    ///
    /// It is a number between 0 and 6000. If experience is <1000 then the
    /// regiment has a threat level of 1. If experience >=1000 and <3000 then
    /// the regiment has a threat level of 2. If experience >= 3000 and <6000
    /// then the regiment has a threat level of 3. If experience >= 6000 then
    /// the regiment has a threat level of 4.
    pub total_experience: u16,
    pub duplicate_id: u8,
    /// The regiment's minimum or base level of armor.
    ///
    /// This is displayed as the gold shields in the troop roster.
    pub min_armor: u8,
    /// The spell book that is equipped to the regiment. A spell book is one of
    /// the magic items.
    ///
    /// This is an index into the list of magic items. In the original game, the
    /// value is either 22, 23, 24, 25 or 65535.
    ///
    /// A value of 22 means the Bright Book is equipped. A value of 23 means the
    /// Ice Book is equipped. A value of 65535 means the regiment does not have
    /// a spell book slot. Only mages can equip spell books.
    pub spell_book: SpellBook,
    /// A list of magic items that are equipped to the regiment.
    ///
    /// Each magic item is an index into the list of magic items. A value of 1
    /// means the Grudgebringer Sword is equipped in that slot. A value of 65535
    /// means the regiment does not have anything equipped in that slot.
    pub magic_items: [u16; 3],
    /// A list of spells that the regiment can cast.
    ///
    /// Each spell is an index into the list of magic items unless the value is
    /// 0 or 65535. From testing changes to `SPARE9MR.ARM` in the original game,
    /// it doesn't seem like this can be changed to a specific set of spells.
    /// The changes seem to be ignored. It's possible that a CTL file overrides
    /// this value, or for player regiments, the threat level determines the
    /// number of spells to provision.
    ///
    /// See `GAMEDATA/1PBAT/B1_04/B104NME.ARM` for an example of all 0s in the
    /// spells field.
    ///
    /// See `GAMEDATA/1PBAT/B3_08/B308MRC.ARM` and
    /// `GAMEDATA/1PBAT/B3_08/B308NME.ARM` for an example with non-zero values.
    pub spells: [u16; 5],
    /// The amount of gold captured by the regiment in the last battle. The
    /// total amount of gold captured by the army can be calculated by summing
    /// the gold captured by each regiment.
    pub last_battle_captured_gold: u16,
    pub purchased_armor: u8,
    pub max_purchasable_armor: u8,
    pub repurchased_unit_count: u8,
    pub max_purchasable_unit_count: u8,
    pub book_profile_index: u32,
}

impl Regiment {
    /// The maximum threat rating for a regiment.
    pub const MAX_THREAT_RATING: u8 = 4;

    /// Returns the display name of the regiment.
    ///
    /// May be empty. The display name index is the preferred way to get the
    /// display name. This is so that the display name can be localized.
    #[inline(always)]
    pub fn display_name(&self) -> &str {
        self.unit_profile.display_name.as_str()
    }

    /// Returns the display name index of the regiment.
    ///
    /// The display name index is used to look up the display name string in the
    /// list of display names found in ENGREL.EXE. This allows the display name
    /// to be localized.
    ///
    /// This is an index into the list of display names found in ENGREL.EXE.
    #[inline(always)]
    pub fn display_name_index(&self) -> u16 {
        self.unit_profile.display_name_index
    }

    /// Marks the regiment as active.
    pub fn mark_active(&mut self) {
        self.flags.insert(RegimentFlags::ACTIVE);
    }

    /// Forces the regiment to be deployed.
    pub fn mark_must_deploy(&mut self) {
        self.flags.insert(RegimentFlags::MUST_DEPLOY);
    }

    /// Marks the regiment as temporary.
    pub fn mark_temporary(&mut self) {
        self.flags.insert(RegimentFlags::TEMPORARY);
    }

    /// Returns `true` if the regiment must be deployed.
    pub fn must_deploy(&self) -> bool {
        self.flags.contains(RegimentFlags::MUST_DEPLOY)
    }

    /// Returns `true` if the regiment is active.
    pub fn is_active(&self) -> bool {
        self.flags.contains(RegimentFlags::ACTIVE)
    }

    /// Returns `true` if the regiment is temporary.
    pub fn is_temporary(&self) -> bool {
        self.flags.contains(RegimentFlags::TEMPORARY)
    }

    /// Returns `true` if the regiment is deployable.
    pub fn is_deployable(&self) -> bool {
        self.flags.contains(RegimentFlags::ACTIVE)
            && !self.flags.contains(RegimentFlags::NON_DEPLOYABLE)
    }

    /// Returns the number of units in the regiment that are alive.
    #[inline(always)]
    pub fn alive_unit_count(&self) -> u8 {
        self.unit_profile.alive_unit_count
    }

    /// Returns the maximum number of units allowed in the regiment.
    #[inline(always)]
    pub fn max_unit_count(&self) -> u8 {
        self.unit_profile.max_unit_count
    }

    /// Returns the rank count.
    #[inline(always)]
    pub fn rank_count(&self) -> u8 {
        self.unit_profile.rank_count
    }

    /// A value from 1 to 4, inclusive, that indicates the regiment's threat
    /// rating.
    #[inline(always)]
    pub fn threat_rating(&self) -> u8 {
        (self.unit_profile.point_value >> 3) + 1
    }

    /// Returns `true` if the regiment is a mage.
    #[inline(always)]
    pub fn is_mage(&self) -> bool {
        self.mage_class != MageClass::None
    }

    /// Returns `true` if the regiment has any magic items equipped.
    pub fn any_magic_items(&self) -> bool {
        self.magic_items.iter().any(|&item| item != 65535)
    }

    /// Returns a list of all magic items equipped to the regiment.
    pub fn all_magic_items(&self) -> Vec<u16> {
        self.magic_items
            .iter()
            .filter(|&&item| item != 65535)
            .copied()
            .collect()
    }

    /// Returns the projectile class of the regiment.
    #[inline(always)]
    pub fn projectile_class(&self) -> ProjectileClass {
        self.leader_profile.projectile_class
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct RegimentFlags: u16 {
        /// No flags are set. This is the default state.
        const NONE = 0;
        /// Set if the regiment is active. Active regiments can be deployed to
        /// the battlefield. This is used when deciding if the regiment should
        /// be shown in the troop roster, or if the regiment is available in the
        /// army reserve. Also known as "available for hire".
        const ACTIVE = 1 << 0;
        /// Set if the regiment is deployed or was deployed in the last battle.
        ///
        /// This flag's meaning is context-dependent.
        ///
        /// During a battle, it indicates that the regiment is currently
        /// deployed to the battlefield. If a regiment is not deployed, then it
        /// remains in the army's reserve.
        ///
        /// Outside of a battle, it indicates that the regiment was deployed in
        /// the last battle. Among other things, this is used when deciding if
        /// the regiment should be shown on the debrief screen battle roster.
        const DEPLOYED = 1 << 1;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_2 = 1 << 2;
        /// Set if the regiment must be deployed to the battlefield. Regiments
        /// that must be deployed cannot be taken off the battlefield. The
        /// player is not allowed to put them in the army reserve.
        const MUST_DEPLOY = 1 << 3;
        /// TODO: Not sure what this flag is yet. This is used by almost every
        /// regiment across .ARM and save games. Removed this from a regiment
        /// and they battled fine and then the flag stayed off after the battle
        /// was finished (i.e., it wasn't reinstated after the battle).
        const UNKNOWN_REGIMENT_FLAG_4 = 1 << 4;
        /// Set if the regiment is heavily damaged. Heavily damaged regiments
        /// result in the leader's portrait being shown in the campaign with
        /// blood on their face.
        const HEAVILY_DAMAGED = 1 << 5;
        /// Set if the regiment is non-deployable. Non-deployable regiments
        /// cannot be deployed to the battlefield and do not appear in the army
        /// reserve. This overrides the [`RegimentFlags::ACTIVE`] flag when
        /// deciding if the regiment can be deployed. This is used for cases
        /// such as underground battles where artillery regiments like cannons
        /// and mortars are not available (you can imagine they stay above
        /// ground back at base). Regiments with the [`RegimentFlags::ACTIVE`]
        /// flag as well as the [`RegimentFlags::NON_DEPLOYABLE`] flag are shown
        /// in the troop roster but cannot be deployed to the battlefield.
        const NON_DEPLOYABLE = 1 << 6;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_7 = 1 << 7;
        /// Set if the regiment is temporarily in the army. In the troop roster,
        /// temporary regiments are shown with a green arrow next to the banner.
        const TEMPORARY = 1 << 8;
        /// Set if the regiment has departed.
        const DEPARTED = 1 << 9;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_10 = 1 << 10;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_11 = 1 << 11;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_12 = 1 << 12;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_13 = 1 << 13;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_14 = 1 << 14;
        /// The flag seems to be unused in any .ARM or save games. It's possible
        /// it's only set during battle.
        const UNKNOWN_REGIMENT_FLAG_15 = 1 << 15;
    }
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum MageClass {
    #[default]
    None = 0,
    BaseMage = 2,
    OrcAdept = 3,
    AdeptMage = 4,
    MasterMage = 5,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum RegimentAlignment {
    #[default]
    Good = 0,
    Neutral = 64,
    Evil = 128,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum RegimentClass {
    #[default]
    None = 0,
    HumanInfantryman = 8,
    WoodElfInfantryman = 9,
    DwarfInfantryman = 10,
    NightGoblinInfantryman = 11,
    OrcInfantryman = 12,
    UndeadInfantryman = 13,
    Townsfolk = 14,
    Ogre1 = 15,
    HumanCavalryman = 16,
    OrcCavalryman = 20,
    UndeadCavalryman = 21,
    HumanArcher = 24,
    WoodElfArcher = 25,
    NightGoblinArcher = 27,
    OrcArcher = 28,
    SkeletonArcher = 29,
    HumanArtilleryUnit = 32,
    OrcArtilleryUnit = 36,
    UndeadArtilleryUnit = 37,
    HumanMage = 40,
    NightGoblinShaman = 43,
    OrcShaman = 44,
    EvilMage = 45,
    DreadKing = 53,
    Monster = 55,
    UndeadChariot = 61,
    Fanatic = 67,
    Ogre2 = 71,
}

impl RegimentClass {
    pub fn is_infantry(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::Infantryman)
    }

    pub fn is_cavalry(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::Cavalryman)
    }

    pub fn is_archer(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::Archer)
    }

    pub fn is_artillery(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::ArtilleryUnit)
    }

    pub fn is_mage(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::Mage)
    }

    pub fn is_monster(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::Monster)
    }

    pub fn is_chariot(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == Into::<u8>::into(RegimentType::Chariot)
    }

    pub fn is_human(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::Human)
    }

    pub fn is_wood_elf(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::WoodElf)
    }

    pub fn is_dwarf(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::Dwarf)
    }

    pub fn is_night_goblin(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::NightGoblin)
    }

    pub fn is_orc(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::Orc)
    }

    pub fn is_undead(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::Undead)
    }

    pub fn is_townsfolk(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::Townsfolk)
    }

    pub fn is_ogre(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == Into::<u8>::into(RegimentRace::Ogre)
    }
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum RegimentType {
    #[default]
    Unknown,
    Infantryman,
    Cavalryman,
    Archer,
    ArtilleryUnit,
    Mage,
    Monster,
    Chariot,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum RegimentRace {
    #[default]
    Human,
    WoodElf,
    Dwarf,
    NightGoblin,
    Orc,
    Undead,
    Townsfolk,
    Ogre,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum MountClass {
    #[default]
    None,
    Horse,
    Boar,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct RegimentAttributes: u32 {
        const NONE = 0;
        /// The regiment never retreats from a fight and the retreat button is
        /// disabled.
        const NEVER_ROUTS = 1 << 0;
        const UNKNOWN_FLAG_2 = 1 << 1;
        /// Fear increases the chance that the enemy retreats during combat.
        const CAUSES_FEAR = 1 << 2;
        /// Stronger version of CAUSES_FEAR.
        const CAUSES_TERROR = 1 << 3;
        /// Used by elves (unknown use).
        const ELF_RACE = 1 << 4;
        /// Used by goblins (unknown use).
        const GOBLIN_RACE = 1 << 5;
        const HATES_GREENSKINS = 1 << 6;
        /// Regiment has the same movement speed on every terrain.
        const NOT_SLOWED_BY_DIFFICULT_TERRAIN = 1 << 7;
        /// Immune against any fear but can still rout.
        const IMMUNE_TO_FEAR_CAN_BE_ROUTED = 1 << 8;
        /// Slowly heals damage.
        const REGENERATES_WOUNDS = 1 << 9;
        /// Regiment never regroups when it is retreating.
        const NEVER_RALLIES_OR_REGROUPS = 1 << 10;
        /// Permanently follows retreating units.
        const ALWAYS_PURSUES = 1 << 11;
        /// Steam Tank flag. Can't enter close combat anymore.
        const ENGINE_OF_WAR_RULE = 1 << 12;
        /// Regiment becomes invulnerable.
        const INDESTRUCTIBLE = 1 << 13;
        const UNKNOWN_FLAG_15 = 1 << 14;
        /// Regiment gets lots of additional damage in close combat (used by
        /// skeletons).
        const SUFFERS_ADDITIONAL_WOUNDS = 1 << 15;
        /// If the regiment attacks an enemy using close or ranged combat, the
        /// enemy receives additional fear.
        const INFLICTING_CASUALTY_CAUSES_FEAR = 1 << 16;
        /// Regiment is less resistant against fear.
        const COWARDLY = 1 << 17;
        /// Regiment dies during retreat.
        const DESTROYED_IF_ROUTED = 1 << 18;
        /// Suffers additional damage when attacked by fire.
        const FLAMMABLE = 1 << 19;
        /// Regiment can see everything that isn't blocked by mountains or other
        /// obstacles.
        const THREE_SIXTY_DEGREE_VISION = 1 << 20;
        /// Regiment spawns fanatics if an enemy is near enough.
        const SPAWNS_FANATICS = 1 << 21;
        /// Used by wraiths (unknown use).
        const WRAITH_RACE = 1 << 22;
        /// Used by Treeman.
        const GIANT = 1 << 23;
        /// The goblins on the Trading Post map have this flag set (unknown
        /// use).
        const GOBLIN_FLAG_TRADING_POST_MAP_ONLY = 1 << 24;
        /// Regiment is completely immune against magic attacks.
        const IMPERVIOUS_TO_MAGIC = 1 << 25;
        /// Same as NEVER_ROUTS but the retreat button is enabled (and ignored).
        const NEVER_RETREATS = 1 << 26;
        /// Regiment has no item slots. Items can still be assigned.
        const NO_ITEM_SLOTS = 1 << 27;
        /// Fanatics have this flag.
        const FANATICS_FLAG = 1 << 28;
        /// Penalty when fighting elves.
        const FEARS_ELVES = 1 << 29;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct LastBattleStats {
    /// The number of units in the regiment that were killed in the last battle.
    pub unit_killed_count: u16,
    unknown1: u16,
    /// The number of units the regiment killed in the last battle.
    pub kill_count: u16,
    /// The regiment's experience gained in the last battle.
    pub experience: u16,
}

#[repr(u16)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum SpellBook {
    #[default]
    None = 65535,
    BrightBook = 22,
    IceBook = 23,
    WaaaghBook = 24,
    DarkBook = 25,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum WeaponClass {
    #[default]
    None,
    BasicHandWeapon,
    TwoHandedWeapon,
    Polearm,
    Flail,
    WightBlade,
}

#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    IntoPrimitive,
    PartialEq,
    PartialOrd,
    Serialize,
    TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum ProjectileClass {
    #[default]
    None,
    ShortBow = 7,
    NormalBow = 8,
    ElvenBow = 9,
    Crossbow = 10,
    Pistol = 11,
    Cannon = 12,
    Mortar = 13,
    SteamTankCannon = 14,
    RockLobber = 15,
    Ballista = 16,
    ScreamingSkullCatapult = 17,
}

#[derive(Debug, Display, Error, From)]
pub enum DecodeClassError {
    #[error(ignore)]
    InvalidType(TryFromPrimitiveError<RegimentType>),
    #[error(ignore)]
    InvalidRace(TryFromPrimitiveError<RegimentRace>),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct UnitProfile {
    /// The index into the list of sprite sheet file names found in ENGREL.EXE
    /// for the unit's sprite sheet.
    pub sprite_sheet_index: u16,
    /// The display name of the regiment, e.g., "Grudgebringer Cavalry",
    /// "Zombies #1", "Imperial Steam Tank".
    ///
    /// May be empty. The display name index is the preferred way to get the
    /// display name. This is so that the display name can be localized.
    pub display_name: String,
    /// The index into the list of display names found in ENGREL.EXE. This
    /// allows the display name to be localized.
    pub display_name_index: u16,
    /// The regiment's alignment to good or evil.
    ///
    /// - 0x00 (decimal 0) is good.
    /// - 0x40 (decimal 64) is neutral.
    /// - 0x80 (decimal 128) is evil.
    pub alignment: RegimentAlignment,
    /// The maximum number of units allowed in the regiment.
    pub max_unit_count: u8,
    /// The number of units currently alive in the regiment.
    pub alive_unit_count: u8,
    pub rank_count: u8,
    unknown1: Vec<u8>,
    pub stats: UnitStats,
    pub mount_class: MountClass,
    /// When you purchase armor, this goes up by 1 for each armor shield
    /// purchased.
    ///
    /// When you sell armor, this goes down by 1 for each armor shield sold.
    ///
    /// If you have not purchased any armor, this is the same as `min_armor`.
    ///
    /// This is displayed as the silver shields in the troop roster.
    pub armor: u8,
    pub weapon_class: WeaponClass,
    pub class: RegimentClass,
    /// A value from 0 to 31, inclusive, that indicates the regiment's threat
    /// rating.
    ///
    /// - 0-7: Threat rating 1
    /// - 8-15: Threat rating 2
    /// - 16-23: Threat rating 3
    /// - 24-31: Threat rating 4
    ///
    /// For example, the Dread King has the maximum value of 31 and a threat
    /// rating of 4.
    ///
    /// This is set in the `unit_profile`, but 0 in the `leader_profile`.
    pub point_value: u8,
    pub projectile_class: ProjectileClass,
    unknown2: [u8; 4],
    unknown2_a: u16,      // TODO: Remove, debug only.
    unknown2_b: u16,      // TODO: Remove, debug only.
    unknown2_as_u32: u32, // TODO: Remove, debug only.
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct UnitStats {
    pub movement: u8,
    pub weapon_skill: u8,
    pub ballistic_skill: u8,
    pub strength: u8,
    pub toughness: u8,
    pub wounds: u8,
    pub initiative: u8,
    pub attacks: u8,
    pub leadership: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        path::{Path, PathBuf},
    };

    #[test]
    fn test_regiment_threat_rating() {
        fn make_regiment(point_value: u8) -> Regiment {
            Regiment {
                unit_profile: UnitProfile {
                    point_value,
                    ..Default::default()
                },
                ..Default::default()
            }
        }

        assert_eq!(make_regiment(0).threat_rating(), 1);
        assert_eq!(make_regiment(1).threat_rating(), 1);
        assert_eq!(make_regiment(7).threat_rating(), 1);
        assert_eq!(make_regiment(8).threat_rating(), 2);
        assert_eq!(make_regiment(20).threat_rating(), 3);
        assert_eq!(make_regiment(31).threat_rating(), 4);
    }

    #[test]
    fn test_regiment_class_is_infantry() {
        assert!(RegimentClass::HumanInfantryman.is_infantry());
        assert!(!RegimentClass::HumanCavalryman.is_infantry());
        assert!(!RegimentClass::HumanArcher.is_infantry());
        assert!(!RegimentClass::HumanArtilleryUnit.is_infantry());
        assert!(!RegimentClass::HumanMage.is_infantry());
        assert!(!RegimentClass::Monster.is_infantry());
        assert!(!RegimentClass::Fanatic.is_infantry());
    }

    #[test]
    fn test_regiment_class_is_cavalry() {
        assert!(!RegimentClass::HumanInfantryman.is_cavalry());
        assert!(RegimentClass::HumanCavalryman.is_cavalry());
        assert!(!RegimentClass::HumanArcher.is_cavalry());
        assert!(!RegimentClass::HumanArtilleryUnit.is_cavalry());
        assert!(!RegimentClass::HumanMage.is_cavalry());
        assert!(!RegimentClass::Monster.is_cavalry());
        assert!(!RegimentClass::Fanatic.is_cavalry());
    }

    #[test]
    fn test_regiment_class_is_archer() {
        assert!(!RegimentClass::HumanInfantryman.is_archer());
        assert!(!RegimentClass::HumanCavalryman.is_archer());
        assert!(RegimentClass::HumanArcher.is_archer());
        assert!(!RegimentClass::HumanArtilleryUnit.is_archer());
        assert!(!RegimentClass::HumanMage.is_archer());
        assert!(!RegimentClass::Monster.is_archer());
        assert!(!RegimentClass::Fanatic.is_archer());
    }

    #[test]
    fn test_regiment_class_is_artillery() {
        assert!(!RegimentClass::HumanInfantryman.is_artillery());
        assert!(!RegimentClass::HumanCavalryman.is_artillery());
        assert!(!RegimentClass::HumanArcher.is_artillery());
        assert!(RegimentClass::HumanArtilleryUnit.is_artillery());
        assert!(!RegimentClass::HumanMage.is_artillery());
        assert!(!RegimentClass::Monster.is_artillery());
        assert!(!RegimentClass::Fanatic.is_artillery());
    }

    #[test]
    fn test_regiment_class_is_mage() {
        assert!(!RegimentClass::HumanInfantryman.is_mage());
        assert!(!RegimentClass::HumanCavalryman.is_mage());
        assert!(!RegimentClass::HumanArcher.is_mage());
        assert!(!RegimentClass::HumanArtilleryUnit.is_mage());
        assert!(RegimentClass::HumanMage.is_mage());
        assert!(!RegimentClass::Monster.is_mage());
        assert!(!RegimentClass::Fanatic.is_mage());
    }

    #[test]
    fn test_regiment_class_is_human() {
        assert!(RegimentClass::HumanInfantryman.is_human());
        assert!(RegimentClass::HumanCavalryman.is_human());
        assert!(!RegimentClass::WoodElfInfantryman.is_human());
    }

    #[test]
    fn test_regiment_class_is_wood_elf() {
        assert!(!RegimentClass::HumanInfantryman.is_wood_elf());
        assert!(!RegimentClass::HumanCavalryman.is_wood_elf());
        assert!(RegimentClass::WoodElfInfantryman.is_wood_elf());
    }

    #[test]
    fn test_regiment_class_is_dwarf() {
        assert!(!RegimentClass::HumanInfantryman.is_dwarf());
        assert!(!RegimentClass::HumanCavalryman.is_dwarf());
        assert!(RegimentClass::DwarfInfantryman.is_dwarf());
    }

    #[test]
    fn test_regiment_class_is_night_goblin() {
        assert!(!RegimentClass::HumanInfantryman.is_night_goblin());
        assert!(!RegimentClass::HumanCavalryman.is_night_goblin());
        assert!(RegimentClass::NightGoblinInfantryman.is_night_goblin());
    }

    #[test]
    fn test_regiment_class_is_orc() {
        assert!(!RegimentClass::HumanInfantryman.is_orc());
        assert!(!RegimentClass::HumanCavalryman.is_orc());
        assert!(RegimentClass::OrcInfantryman.is_orc());
    }

    #[test]
    fn test_regiment_class_is_undead() {
        assert!(!RegimentClass::HumanInfantryman.is_undead());
        assert!(!RegimentClass::HumanCavalryman.is_undead());
        assert!(RegimentClass::UndeadInfantryman.is_undead());
    }

    #[test]
    fn test_regiment_class_is_townsfolk() {
        assert!(!RegimentClass::HumanInfantryman.is_townsfolk());
        assert!(!RegimentClass::HumanCavalryman.is_townsfolk());
        assert!(RegimentClass::Townsfolk.is_townsfolk());
    }

    fn roundtrip_test(original_bytes: &[u8], army: &Army) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(army).unwrap();

        let original_bytes = original_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let encoded_bytes = encoded_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(original_bytes, encoded_bytes);
    }

    #[test]
    fn test_decode_plyr_alg() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PARM",
            "PLYR_ALG.ARM",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        assert_eq!(a.regiments[0].unit_profile.display_name, ""); // not set
        assert_eq!(a.regiments[0].unit_profile.display_name_index, 4);
        assert_eq!(a.regiments[0].book_profile_index, 4);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_b101mrc() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "B101MRC.ARM",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        assert!(a.race.contains(ArmyRace::EMPIRE));
        assert_eq!(a.small_banners_path, "[BOOKS]\\hshield.spr");
        assert_eq!(a.disabled_small_banners_path, "[BOOKS]\\hgban.spr");
        assert_eq!(a.large_banners_path, "[BOOKS]\\hlban.spr");
        assert_eq!(a.regiments.len(), 4);
        assert!(a.regiments[0].flags.contains(RegimentFlags::ACTIVE));
        assert_eq!(a.regiments[0].id, 1);
        assert_eq!(
            a.regiments[0].unit_profile.display_name,
            "Grudgebringer Cavalry"
        );
        assert_eq!(
            a.regiments[0].unit_profile.class,
            RegimentClass::HumanCavalryman
        );
        assert_eq!(a.regiments[0].unit_profile.mount_class, MountClass::Horse);
        assert_eq!(
            a.regiments[0].leader_profile.display_name,
            "Morgan Bernhardt"
        );
        assert_eq!(a.regiments[1].id, 2);
        assert_eq!(
            a.regiments[1].unit_profile.display_name,
            "Grudgebringer Infantry"
        );
        assert_eq!(
            a.regiments[1].unit_profile.class,
            RegimentClass::HumanInfantryman
        );
        assert_eq!(a.regiments[2].id, 3);
        assert_eq!(
            a.regiments[2].unit_profile.display_name,
            "Grudgebringer Crossbows"
        );
        assert_eq!(
            a.regiments[2].unit_profile.class,
            RegimentClass::HumanArcher
        );
        assert_eq!(a.regiments[3].id, 4);
        assert_eq!(
            a.regiments[3].unit_profile.display_name,
            "Grudgebringer Cannon"
        );
        assert_eq!(
            a.regiments[3].unit_profile.class,
            RegimentClass::HumanArtilleryUnit
        );

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_b103mrc() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_03",
            "B103MRC.ARM",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        assert_eq!(a.regiments[4].unit_profile.display_name, "Bright Wizard");
        assert_eq!(a.regiments[4].mage_class, MageClass::BaseMage);
        assert_eq!(a.regiments[4].spell_book, SpellBook::BrightBook);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_game_000() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-games",
            "darkomen.000", // http://en.dark-omen.org/downloads/view-details/4.-savegames/1.-original-campaigns/save-game-1-1-trading-post.html
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_game_header = a.save_game_header.as_ref().unwrap();
        assert_eq!(save_game_header.display_name, "Grenzgrafschaften - 1026gc");
        assert_eq!(save_game_header.suggested_display_name, "Handelsposten 1");
        assert_eq!(
            save_game_header.script_state.base_execution_address,
            0x4C3D90
        );

        let save_game_footer = a.save_game_footer.as_ref().unwrap();
        assert_eq!(save_game_footer.travel_path_history, vec![]);
        assert_eq!(save_game_footer.victory_message_index, 0);
        assert_eq!(save_game_footer.defeat_message_index, 1);
        assert_eq!(save_game_footer.rng_seed, 3011451320);

        assert!(a.regiments[0].flags.contains(RegimentFlags::MUST_DEPLOY));
        assert_eq!(a.regiments[0].last_battle_stats.kill_count, 10);
        assert_eq!(a.regiments[0].last_battle_stats.experience, 46);
        assert_eq!(a.regiments[0].total_experience, 46);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_game_001() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-games",
            "darkomen.001", // http://en.dark-omen.org/downloads/view-details/4.-savegames/1.-original-campaigns/save-game-1-2-border-counties.html
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_game_header = a.save_game_header.as_ref().unwrap();
        assert_eq!(save_game_header.display_name, "Stadt Grissburg - 1410gc");
        assert_eq!(
            save_game_header.suggested_display_name,
            "Prinzen der Grenze 2"
        );
        assert_eq!(
            save_game_header.script_state.base_execution_address,
            0x4C3D90
        );

        let save_game_footer = a.save_game_footer.as_ref().unwrap();
        assert_eq!(save_game_footer.objectives.len(), 27);
        assert_eq!(save_game_footer.objectives.first().unwrap().id, 26);
        assert_eq!(save_game_footer.travel_path_history, vec![0, 1]);

        assert_eq!(a.regiments[0].last_battle_stats.unit_killed_count, 3);
        assert_eq!(a.regiments[0].last_battle_stats.kill_count, 19);
        assert_eq!(a.regiments[0].last_battle_stats.experience, 175);
        assert_eq!(a.regiments[0].total_experience, 221); // 46 from the first battle plus 175 from the battle prior to this save equals 221

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_game_en_000() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-games",
            "en",
            "darkomen.000",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_game_header = a.save_game_header.as_ref().unwrap();
        assert_eq!(save_game_header.display_name, "Trading Post 1 - 56gc");
        assert_eq!(save_game_header.display_name_residual_bytes, None);
        assert_eq!(save_game_header.suggested_display_name, "Trading Post 1");
        assert_eq!(save_game_header.suggested_display_name_residual_bytes, None);
        assert_eq!(
            save_game_header.script_state.base_execution_address,
            0x4C3C48
        );
        assert_eq!(save_game_header.script_state.execution_offset_index, 370);

        assert_eq!(a.last_battle_earned_gold(), 316); // (48 + 89 + 74) * 1.5 = 316.5 = 316 (rounded down)
        assert_eq!(a.last_battle_captured_gold(), 150);

        assert!(a.regiments[0].flags.contains(RegimentFlags::MUST_DEPLOY));
        assert_eq!(a.regiments[0].last_battle_stats.kill_count, 10);
        assert_eq!(a.regiments[0].last_battle_stats.experience, 48);
        assert_eq!(a.regiments[0].total_experience, 48);
        assert_eq!(a.regiments[0].last_battle_captured_gold, 150);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_game_en_003() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-games",
            "en",
            "darkomen.003",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_game_header = a.save_game_header.as_ref().unwrap();
        assert_eq!(save_game_header.display_name, "Grissburg 1 - 883gc");
        assert_eq!(
            String::from_utf8(
                save_game_header
                    .display_name_residual_bytes
                    .as_ref()
                    .unwrap()
                    .to_vec()
            )
            .unwrap(),
            "83gc" // residual from "Border Princes 2 - 883gc" in the previous save game
        );
        assert_eq!(save_game_header.suggested_display_name, "Grissburg 1");
        assert_eq!(
            String::from_utf8(
                save_game_header
                    .suggested_display_name_residual_bytes
                    .as_ref()
                    .unwrap()
                    .to_vec()
            )
            .unwrap(),
            "es 2" // residual from "Border Princes 2" in the previous save game
        );
        assert_eq!(
            save_game_header.script_state.base_execution_address,
            0x4C3C48
        );
        assert_eq!(save_game_header.script_state.execution_offset_index, 489);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "armies"]
            .iter()
            .collect();

        std::fs::create_dir_all(&root_output_dir).unwrap();

        fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) {
            println!("Reading dir {:?}", dir.display());

            let mut paths = std::fs::read_dir(dir)
                .unwrap()
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()
                .unwrap();

            paths.sort();

            for path in paths {
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }

        visit_dirs(&d, &mut |path| {
            let Some(ext) = path.extension() else {
                return;
            };
            if !(ext.to_string_lossy().to_uppercase() == "ARM"
                || ext.to_string_lossy().to_uppercase() == "AUD"
                || ext.to_string_lossy().to_uppercase() == "ARE")
            {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let army = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &army);

            let parent_dir = path
                .components()
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .skip(1) // skip the file name
                .take_while(|c| c.as_os_str() != "DARKOMEN")
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .collect::<PathBuf>();
            let output_dir = root_output_dir.join(parent_dir);
            std::fs::create_dir_all(&output_dir).unwrap();

            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            ron::ser::to_writer_pretty(&mut output_file, &army, Default::default()).unwrap();
        });
    }

    #[test]
    fn test_decode_all_save_games() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-games",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded"].iter().collect();

        std::fs::create_dir_all(&root_output_dir).unwrap();

        fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) {
            println!("Reading dir {:?}", dir.display());

            let mut paths = std::fs::read_dir(dir)
                .unwrap()
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()
                .unwrap();

            paths.sort();

            for path in paths {
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }

        visit_dirs(&d, &mut |path| {
            let Some(ext) = path.extension() else {
                return;
            };
            // Skip unless the extension matches \d{3}.
            let ext = ext.to_string_lossy();
            if !(ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit())) {
                println!("Skipping {:?}", path.file_name().unwrap());
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let army = Decoder::new(file).decode().unwrap();

            // Every same game should at least have the following objectives.
            let save_game_footer = army.save_game_footer.as_ref().unwrap();
            let required_objective_ids = [1, 3, 4, 7, 26];
            for id in required_objective_ids {
                assert!(
                    save_game_footer.objectives.iter().any(|obj| obj.id == id),
                    "Save game {:?} is missing required objective ID: {}",
                    path.file_name().unwrap(),
                    id
                );
            }

            let parent_dir = path
                .components()
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .skip(1) // skip the file name
                .take_while(|c| c.as_os_str() != "testdata")
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .collect::<PathBuf>();

            let output_dir = root_output_dir.join(parent_dir);
            std::fs::create_dir_all(&output_dir).unwrap();

            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            ron::ser::to_writer_pretty(&mut output_file, &army, Default::default()).unwrap();

            roundtrip_test(&original_bytes, &army);
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
