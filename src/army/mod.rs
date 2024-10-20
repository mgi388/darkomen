mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct SaveFileHeader {
    /// The name displayed when loading the save file.
    pub display_name: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    display_name_remainder: Vec<u8>,
    /// The name suggested when saving the game.
    pub suggested_display_name: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    suggested_display_name_remainder: Vec<u8>,
    /// Protect Bogenhafen mission played.
    pub bogenhafen_mission: bool,
    /// Attacked Goblin Camp or helped Ragnar.
    pub goblin_camp_or_ragnar: bool,
    /// Attacked the goblin camp together with Munz.
    pub goblin_camp_mission: bool,
    /// Helps Ragnar but mission has not been started.
    pub ragnar_mission_pre_battle: bool,
    /// Attacked Greenskins in Vingtienne or helped Treeman.
    pub vingtienne_or_treeman: bool,
    /// Attacked the Greenskins near Vingtienne.
    pub vingtienne_mission: bool,
    /// Helped the treeman in Loren Lake mission.
    pub treeman_mission: bool,
    /// Manfred von Carstein defeated.
    pub carstein_defeated: bool,
    /// Hand of Nagash defeated.
    pub hand_of_nagash_defeated: bool,
    /// Black Grail defeated.
    pub black_grail_defeated: bool,
    pub unknown1: u32,
    /// Helmgart mission played.
    pub helmgart_mission: bool,
    /// Helped Ragnar defeat the trolls.
    pub ragnar_mission: bool,
    /// Talked with King Orion (Woodelf King).
    pub loren_king_met: bool,
    /// Helped Azkuz moving through the Axebite Pass.
    pub axebite_mission: bool,
    pub unknown2: u32,
    pub unknown3: u32,
    pub unknown4: u32,
    pub unknown5: u32,
    pub unknown6: u32,
    pub unknown7: u32,
    /// Previous fought battle won.
    pub previous_battle_won_1: bool,
    /// Previous fought battle won.
    pub previous_battle_won_2: bool,
    /// Answer for last asked question.
    pub previous_answer: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Army {
    save_file_header: Option<SaveFileHeader>,
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
    pub small_banner_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    small_banner_path_remainder: Vec<u8>,
    pub small_banner_disabled_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    small_banner_disabled_path_remainder: Vec<u8>,
    pub large_banner_path: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    large_banner_path_remainder: Vec<u8>,
    /// The amount of gold captured from treasures and earned in the last
    /// battle.
    pub last_battle_gold: u16,
    /// The amount of gold available to the army for buying new units and
    /// reinforcements.
    pub gold_in_coffers: u16,
    /// A list of magic items in the army's inventory.
    ///
    /// Each magic item is an index into the list of magic items. A value of 1
    /// means the Grudgebringer Sword is equipped in that slot. A value of 0
    /// means the army does not have anything in that slot.
    pub magic_items: Vec<u8>,
    unknown3: Vec<u8>,
    pub regiments: Vec<Regiment>,
    save_file_footer: Vec<u8>,
}

impl Army {
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

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum ArmyRace {
    #[default]
    Empire = 0,
    EmpireMultiplayer = 1,
    Greenskin = 2,
    GreenskinMultiplayer = 3,
    Undead = 4,
    UndeadMultiplayer = 5,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Regiment {
    pub status: RegimentStatus,
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
    unknown5: [u8; 4],
    /// The profile of the regiment's leader unit.
    ///
    /// Some of the fields are not used for leader units.
    pub leader_profile: UnitProfile,
    unknown6: [u8; 4],
    /// The leader's 3D head ID.
    pub leader_head_id: u16,

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
    pub gold_captured: u16,
    pub purchased_armor: u8,
    pub max_purchasable_armor: u8,
    pub repurchased_unit_count: u8,
    pub max_purchasable_unit_count: u8,
    pub book_profile: [u8; 4],
}

impl Regiment {
    /// Returns the display name of the regiment.
    #[inline(always)]
    pub fn display_name(&self) -> &str {
        self.unit_profile.name.as_str()
    }

    /// Returns the number of units in the regiment that are alive.
    #[inline(always)]
    pub fn alive_unit_count(&self) -> usize {
        self.unit_profile.alive_unit_count as usize
    }

    /// Returns the rank count.
    #[inline(always)]
    pub fn rank_count(&self) -> usize {
        self.unit_profile.rank_count as usize
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
}

#[repr(u16)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum RegimentStatus {
    #[default]
    None = 0,
    Unknown1 = 1,
    InactiveNotEncountered = 16,
    Active = 17,
    ActivePermanent = 19,
    ActiveAutodeploy = 27,
    Unknown2 = 50,
    Unknown3 = 51,
    Unknown4 = 59,
    Unknown5 = 81,
    ActiveNewTemporary = 273,
    Unknown6 = 283,
    InactiveDestroyed = 306,
    Unknown7 = 307,
    ActiveTemporary = 275,
    InactiveDeparted = 784,
    Unknown8 = 785,
    Unknown9 = 786,
    ActiveAboutToLeave = 787,
    Unknown10 = 818,
    Unknown11 = 819,
    Unknown12 = 848,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
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
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
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
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum RegimentClass {
    #[default]
    None = 0,
    HumanInfantryman = 8,
    WoodElfInfantryman = 9,
    DwarfInfantryman = 10,
    NightGoblinInfantryman = 11,
    OrcInfantryman = 12,
    UndeadInfantryman = 13,
    Townsperson = 14,
    Ogre = 15,
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
    Unknown1 = 71,
}

impl RegimentClass {
    pub fn is_infantry(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == RegimentType::Infantryman.into()
    }

    pub fn is_cavalry(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == RegimentType::Cavalryman.into()
    }

    pub fn is_archer(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == RegimentType::Archer.into()
    }

    pub fn is_artillery(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == RegimentType::ArtilleryUnit.into()
    }

    pub fn is_mage(&self) -> bool {
        Into::<u8>::into(*self) >> 3 == RegimentType::Mage.into()
    }

    pub fn is_human(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::Human.into()
    }

    pub fn is_wood_elf(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::WoodElf.into()
    }

    pub fn is_dwarf(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::Dwarf.into()
    }

    pub fn is_night_goblin(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::NightGoblin.into()
    }

    pub fn is_orc(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::Orc.into()
    }

    pub fn is_undead(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::Undead.into()
    }

    pub fn is_townsperson(&self) -> bool {
        Into::<u8>::into(*self) & 0x07 == RegimentRace::Townsfolk.into()
    }
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
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
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum RegimentRace {
    #[default]
    Human,
    WoodElf,
    Dwarf,
    NightGoblin,
    Orc,
    Undead,
    Townsfolk,
}

#[repr(u8)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum RegimentMount {
    #[default]
    None,
    Horse,
    Boar,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect_value(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
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
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct LastBattleStats {
    /// The number of units in the regiment that were killed in the last battle.
    pub unit_killed_count: u16,
    unknown1: [u8; 2], // always seems to be 0, could be padding
    /// The number of units the regiment killed in the last battle.
    pub kill_count: u16,
    /// The regiment's experience gained in the last battle.
    pub experience: u16,
}

#[repr(u16)]
#[derive(
    Clone, Copy, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
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
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum Weapon {
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
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum Projectile {
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

#[derive(Error, Debug)]
pub enum DecodeClassError {
    #[error(transparent)]
    InvalidType(#[from] TryFromPrimitiveError<RegimentType>),
    #[error(transparent)]
    InvalidRace(#[from] TryFromPrimitiveError<RegimentRace>),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct UnitProfile {
    /// The index into the list of sprite sheet file names found in ENGREL.EXE
    /// for the unit's sprite sheet.
    pub sprite_sheet_index: u16,
    /// The name of the regiment, e.g. "Grudgebringer Cavalry", "Zombies #1",
    /// "Imperial Steam Tank".
    pub name: String,
    pub name_id: u16,
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
    pub mount: RegimentMount,
    pub armor: u8,
    pub weapon: Weapon,
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
    pub projectile: Projectile,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
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
    fn test_regiment_class_is_townsperson() {
        assert!(!RegimentClass::HumanInfantryman.is_townsperson());
        assert!(!RegimentClass::HumanCavalryman.is_townsperson());
        assert!(RegimentClass::Townsperson.is_townsperson());
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

        assert_eq!(a.race, ArmyRace::Empire);
        assert_eq!(a.small_banner_path, "[BOOKS]\\hshield.spr");
        assert_eq!(a.small_banner_disabled_path, "[BOOKS]\\hgban.spr");
        assert_eq!(a.large_banner_path, "[BOOKS]\\hlban.spr");
        assert_eq!(a.regiments.len(), 4);
        assert_eq!(a.regiments[0].status, RegimentStatus::Active);
        assert_eq!(a.regiments[0].id, 1);
        assert_eq!(a.regiments[0].unit_profile.name, "Grudgebringer Cavalry");
        assert_eq!(
            a.regiments[0].unit_profile.class,
            RegimentClass::HumanCavalryman
        );
        assert_eq!(a.regiments[0].unit_profile.mount, RegimentMount::Horse);
        assert_eq!(a.regiments[0].leader_profile.name, "Morgan Bernhardt");
        assert_eq!(a.regiments[1].id, 2);
        assert_eq!(a.regiments[1].unit_profile.name, "Grudgebringer Infantry");
        assert_eq!(
            a.regiments[1].unit_profile.class,
            RegimentClass::HumanInfantryman
        );
        assert_eq!(a.regiments[2].id, 3);
        assert_eq!(a.regiments[2].unit_profile.name, "Grudgebringer Crossbows");
        assert_eq!(
            a.regiments[2].unit_profile.class,
            RegimentClass::HumanArcher
        );
        assert_eq!(a.regiments[3].id, 4);
        assert_eq!(a.regiments[3].unit_profile.name, "Grudgebringer Cannon");
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

        assert_eq!(a.regiments[4].unit_profile.name, "Bright Wizard");
        assert_eq!(a.regiments[4].mage_class, MageClass::BaseMage);
        assert_eq!(a.regiments[4].spell_book, SpellBook::BrightBook);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_file_000() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-files",
            "darkomen.000", // http://en.dark-omen.org/downloads/view-details/4.-savegames/1.-original-campaigns/save-game-1-1-trading-post.html
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_file_header = a.save_file_header.as_ref().unwrap();
        assert_eq!(save_file_header.display_name, "Grenzgrafschaften - 1026gc");
        assert_eq!(save_file_header.suggested_display_name, "Handelsposten 1");

        assert_eq!(a.regiments[0].status, RegimentStatus::ActiveAutodeploy);
        assert_eq!(a.regiments[0].last_battle_stats.kill_count, 10);
        assert_eq!(a.regiments[0].last_battle_stats.experience, 46);
        assert_eq!(a.regiments[0].total_experience, 46);

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_file_001() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-files",
            "darkomen.001", // http://en.dark-omen.org/downloads/view-details/4.-savegames/1.-original-campaigns/save-game-1-2-border-counties.html
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_file_header = a.save_file_header.as_ref().unwrap();
        assert_eq!(save_file_header.display_name, "Stadt Grissburg - 1410gc");
        assert_eq!(
            save_file_header.suggested_display_name,
            "Prinzen der Grenze 2"
        );

        assert_eq!(a.regiments[0].last_battle_stats.unit_killed_count, 3);
        assert_eq!(a.regiments[0].last_battle_stats.kill_count, 19);
        assert_eq!(a.regiments[0].last_battle_stats.experience, 175);
        assert_eq!(a.regiments[0].total_experience, 221); // 46 from the first battle plus 175 from the battle prior to this save equals 221

        roundtrip_test(&original_bytes, &a);
    }

    #[test]
    fn test_decode_save_file_en_000() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-files",
            "en",
            "darkomen.000",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        let save_file_header = a.save_file_header.as_ref().unwrap();
        assert_eq!(save_file_header.display_name, "Trading Post 1 - 56gc");
        assert_eq!(save_file_header.suggested_display_name, "Trading Post 1");

        assert_eq!(a.regiments[0].status, RegimentStatus::ActiveAutodeploy);
        assert_eq!(a.regiments[0].last_battle_stats.kill_count, 10);
        assert_eq!(a.regiments[0].last_battle_stats.experience, 48);
        assert_eq!(a.regiments[0].total_experience, 48);
        assert_eq!(a.regiments[0].gold_captured, 150);

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
    fn test_decode_all_save_files() {
        let d: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "army",
            "testdata",
            "save-files",
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
            if path.is_dir() {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let army = Decoder::new(file).decode().unwrap();

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
