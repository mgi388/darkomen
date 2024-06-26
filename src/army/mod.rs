mod decoder;
mod encoder;

use serde::Serialize;

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Debug, Clone, Serialize)]
pub struct Army {
    pub race: u8,
    unknown1: Vec<u8>,
    unknown2: Vec<u8>,
    pub regiments: Vec<Regiment>,
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
    pub gold_from_treasures: u16,
    pub gold_in_coffers: u16,
    pub magic_items: Vec<u8>,
    unknown3: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Regiment {
    status: [u8; 2],
    id: u16,

    /// The name of the regiment, e.g. "Grudgebringer Cavalry", "Zombies #1",
    /// "Imperial Steam Tank".
    name: String,

    name_id: u16,

    /// The regiment's alignment to good or evil.
    ///
    /// - 0x00 (decimal 0) is good.
    /// - 0x40 (decimal 64) is neutral.
    /// - 0x80 (decimal 128) is evil.
    alignment: u8,
    /// A bitfield for the regiment's type and race.
    ///
    /// The lower 3 bits determine the race. The higher 5 bits determine the
    /// regiment's type.
    typ: u8,
    /// The index into the list of sprite file names found in ENGREL.EXE for the
    /// regiment's banner.
    banner_index: u16,
    /// The index into the list of sprite file names found in ENGREL.EXE for the
    /// regiment's troop sprite.
    sprite_index: u16,
    /// The maximum number of troops allowed in this regiment.
    max_troops: u8,
    /// The number of troops currently alive in this regiment.
    alive_troops: u8,

    ranks: u8,
    regiment_attributes: [u8; 4],
    troop_attributes: TroopAttributes,
    mount: u8,
    armor: u8,
    weapon: u8,
    point_value: u8,
    missile_weapon: u8,

    /// The regiment's leader.
    leader: Leader,
    /// A number that represents the regiment's total experience.
    ///
    /// It is a number between 0 and 6000. If experience is <1000 then the
    /// regiment has a threat level of 1. If experience >=1000 and <3000 then
    /// the regiment has a threat level of 2. If experience >= 3000 and <6000
    /// then the regiment has a threat level of 3. If experience >= 6000 then
    /// the regiment has a threat level of 4.
    experience: u16,
    /// The regiment's minimum or base level of armor.
    ///
    /// This is displayed as the gold shields in the troop roster.
    min_armor: u8,
    /// The regiment's maximum level of armor.
    max_armor: u8,
    /// The magic book that is equipped to the regiment. A magic book is one of
    /// the magic items.
    ///
    /// This is an index into the list of magic items. In the original game, the
    /// value is either 22, 23, 24, 25 or 65535.
    ///
    /// A value of 22 means the Bright Book is equipped. A value of 23 means the
    /// Ice Book is equipped. A value of 65535 means the regiment does not have
    /// a magic book slot—only magic users can equip magic books.
    magic_book: u16,
    /// A list of magic items that are equipped to the regiment.
    ///
    /// Each magic item is an index into the list of magic items. A value of 1
    /// means the Grudgebringer Sword is equipped in that slot. A value of 65535
    /// means the regiment does not have anything equipped in that slot.
    magic_items: [u16; 3],

    cost: u16,

    wizard_type: u8,

    duplicate_id: u8,
    purchased_armor: u8,
    max_purchasable_armor: u8,
    repurchased_troops: u8,
    max_purchasable_troops: u8,
    book_profile: [u8; 4],

    unknown1: [u8; 2],
    unknown2: [u8; 2],
    unknown3: [u8; 2],
    unknown4: [u8; 4],
    unknown5: u8,
    unknown6: [u8; 4],
    unknown7: [u8; 12],
}

#[derive(Debug, Clone, Serialize)]
pub struct TroopAttributes {
    movement: u8,
    weapon_skill: u8,
    ballistic_skill: u8,
    strength: u8,
    toughness: u8,
    wounds: u8,
    initiative: u8,
    attacks: u8,
    leadership: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Leader {
    /// The name of the leader.
    name: String,
    /// There are some bytes after the null-terminated string. Not sure what
    /// they are for.
    name_remainder: Vec<u8>,
    /// The index into the list of sprite file names found in ENGREL.EXE for the
    /// leader's sprite.
    sprite_index: u16,

    attributes: TroopAttributes,
    mount: u8,
    armor: u8,
    weapon: u8,
    unit_type: u8,
    point_value: u8,
    missile_weapon: u8,
    unknown1: [u8; 4],
    /// The leader's 3D head ID.
    head_id: u16,
    x: [u8; 4],
    y: [u8; 4],
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

        assert_eq!(a.small_banner_path, "[BOOKS]\\hshield.spr");
        assert_eq!(a.small_banner_disabled_path, "[BOOKS]\\hgban.spr");
        assert_eq!(a.large_banner_path, "[BOOKS]\\hlban.spr");
        assert_eq!(a.regiments.len(), 4);
        assert_eq!(a.regiments[0].name, "Grudgebringer Cavalry");
        assert_eq!(a.regiments[0].leader.name, "Morgan Bernhardt");
        assert_eq!(a.regiments[0].mount, 1);
        assert_eq!(a.regiments[1].name, "Grudgebringer Infantry");
        assert_eq!(a.regiments[2].name, "Grudgebringer Crossbows");
        assert_eq!(a.regiments[3].name, "Grudgebringer Cannon");

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
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }

        visit_dirs(&d, &mut |path| {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_uppercase() == "ARM"
                    || ext.to_string_lossy().to_uppercase() == "AUD"
                    || ext.to_string_lossy().to_uppercase() == "ARE"
                {
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
                    ron::ser::to_writer_pretty(&mut output_file, &army, Default::default())
                        .unwrap();
                }
            }
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}