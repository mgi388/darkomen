mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::{DVec3, Vec3};
use image::{DynamicImage, GenericImage, Rgba};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Project {
    /// The base model file name, including the extension. E.g. `base.M3D`.
    ///
    /// The file name is relative to the directory where the project file is
    /// located.
    pub base_model_file_name: String,
    /// The water model file name, including the extension. E.g. `_7water.M3D`.
    /// If not present, the project has no water model.
    ///
    /// The file name is relative to the directory where the project file is
    /// located.
    ///
    /// Note: Some projects overload this field for other non-water models. E.g.
    /// in B1_07 this field is `_4tower.m3d` to render a tower instead of water.
    pub water_model_file_name: Option<String>,
    /// A list of furniture model file names, including the extension. This is
    /// used by instances to look up the model they use.
    ///
    /// The file names are relative to the directory where the project file is
    /// located.
    pub furniture_model_file_names: Vec<String>,
    pub instances: Vec<Instance>,
    pub terrain: Terrain,
    pub attributes: Attributes,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    excl: Excl,
    /// The background music script file name, including the extension. E.g.
    /// `battle1.fsm`.
    pub background_music_script_file_name: String,
    pub tracks: Vec<Track>,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    edit: Vec<u8>,
}

impl Project {
    /// Get the base model file name, including the extension, but with the
    /// extension replaced with `.M3X`. E.g. `base.M3D` becomes `base.M3X`.
    ///
    /// The M3X version is a chunked version of the M3D model and is the one
    /// rendered in game.
    pub fn get_base_m3x_model_file_name(&self) -> String {
        self.base_model_file_name
            .replace(".m3d", ".m3x")
            .replace(".M3D", ".M3X")
    }

    /// Get the water model file name, including the extension, but with the
    /// extension replaced with `.M3X`. E.g. `_7water.M3D` becomes
    /// `_7water.M3X`.
    ///
    /// The M3X version is a chunked version of the M3D model and is the one
    /// rendered in game.
    pub fn get_water_m3x_model_file_name(&self) -> Option<String> {
        self.water_model_file_name
            .as_ref()
            .map(|s| s.replace(".m3d", ".m3x").replace(".M3D", ".M3X"))
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Instance {
    prev: i32,
    next: i32,
    selected: i32,
    pub exclude_from_terrain: i32,
    pub position: DVec3,
    pub rotation: DVec3,
    pub aabb_min: DVec3,
    pub aabb_max: DVec3,
    /// Slot is 1-based, not 0-based. A slot of 1 refers to the first furniture
    /// model and a slot of 0 means the instance is not used.
    pub furniture_model_slot: u32,
    model_id: i32,
    attackable: i32,
    toughness: i32,
    wounds: i32,
    pub unknown1: i32,
    owner_unit_index: i32,
    burnable: i32,
    pub sfx_code: u32,
    /// Instances with a model can have a GFX code set, e.g. for the windmill
    /// model, it has animated sails and for some building models they have an
    /// animated flag or sign.
    pub gfx_code: u32,
    locked: i32,
    exclude_from_terrain_shadow: i32,
    exclude_from_walk: i32,
    pub magic_item_code: u32,
    pub particle_effect_code: u32,
    /// Slot is 1-based, not 0-based. A slot of 1 refers to the first furniture
    /// model and a slot of 0 means the instance is not used.
    pub furniture_dead_model_slot: u32,
    dead_model_id: i32,
    pub light: i32,
    light_radius: i32,
    light_ambient: i32,
    pub unknown2: i32,
    pub unknown3: i32,
}

impl Instance {
    pub fn position_lossy(&self) -> Vec3 {
        Vec3::new(
            self.position.x as f32,
            self.position.y as f32,
            self.position.z as f32,
        )
    }

    pub fn rotation_lossy(&self) -> Vec3 {
        Vec3::new(
            self.rotation.x as f32,
            self.rotation.y as f32,
            self.rotation.z as f32,
        )
    }

    pub fn aabb_min_lossy(&self) -> Vec3 {
        Vec3::new(
            self.aabb_min.x as f32,
            self.aabb_min.y as f32,
            self.aabb_min.z as f32,
        )
    }

    pub fn aabb_max_lossy(&self) -> Vec3 {
        Vec3::new(
            self.aabb_max.x as f32,
            self.aabb_max.y as f32,
            self.aabb_max.z as f32,
        )
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Terrain {
    pub width: u32,
    pub height: u32,
    /// A list of large blocks for the first heightmap.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub heightmap1_blocks: Vec<TerrainBlock>,
    /// A list of large blocks for the second heightmap.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub heightmap2_blocks: Vec<TerrainBlock>,
    /// A list of offsets for 8x8 block. Height offset for each block based on
    /// minimum height. Each item is a list which must have exactly 64 (8x8)
    /// u8s.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub offsets: Vec<Vec<u8>>,
}

impl Terrain {
    pub fn heightmap1_image(&self) -> DynamicImage {
        self.heightmap_image(&self.heightmap1_blocks)
    }

    pub fn heightmap2_image(&self) -> DynamicImage {
        self.heightmap_image(&self.heightmap2_blocks)
    }

    fn heightmap_image(&self, blocks: &Vec<TerrainBlock>) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(self.width, self.height);

        let mut row = 0;
        let mut col = 0;

        for block in blocks {
            let offsets = &self.offsets[block.offset_index as usize];

            if col * 8 >= self.width {
                col = 0;
                row += 1;
            }

            for y in 0..8 {
                let target_y = row * 8 + y;

                if target_y >= self.height {
                    break;
                }

                for x in 0..8 {
                    let target_x = col * 8 + x;

                    if target_x >= self.width {
                        break;
                    }

                    let color_part1 = offsets[(x + y * 8) as usize] as i32;
                    let color_part2 = block.min_height / 257;

                    // TODO: Clamped this to avoid panics, but possibly
                    // indicates a bug.
                    let color = (color_part1 + color_part2).clamp(0, 255) as u8;

                    img.put_pixel(target_x, target_y, Rgba([color, color, color, 255]));
                }
            }

            col += 1;
        }

        img.fliph() // needs to be flipped horizontally for some reason
    }

    /// TODO: Not really working perfectly.
    pub fn get_height(&self, map_num: u32, x: i32, y: i32) -> f32 {
        // TODO: Should we clamp max values / 8?
        let x = (x / 8).clamp(0, self.width as i32);
        let y = (y / 8).clamp(0, self.height as i32);

        // TODO: Understand what this is doing and means.
        let off_address = (((y >> 3) * self.width as i32 / 8) + (x >> 3)) as usize;
        let macro_block_address = ((y % 8) * 8 + (x % 8)) as usize;

        let blocks = match map_num {
            1 => &self.heightmap1_blocks,
            2 => &self.heightmap2_blocks,
            _ => panic!("invalid map number"),
        };

        // TODO: This clamps to avoid a panic but can we avoid this?
        let b = &blocks[off_address.min(blocks.len() - 1)];
        let macro_block = b.offset_index as usize;

        let additional = self.offsets[macro_block][macro_block_address];
        (additional as f32 / 8.) + (b.min_height as f32 / 1024.)
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct TerrainBlock {
    /// The minimum height of all 64 (8x8) values in the block. This is the base
    /// height for the block.
    pub min_height: i32,
    /// An index into the offsets list. Used to get the 64 (8x8) values that
    /// make up the block. The values are height offsets based on the minimum
    /// height. To get the height at a specific point, you need to combine the
    /// minimum height with the offset at that point.
    pub offset_index: u32,
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Attributes {
    pub width: u32,
    pub height: u32,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub unknown: Vec<u8>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Excl {
    pub unknown1: u32, // seems like a count, but unknown
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub unknown2: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Track {
    pub control_points: Vec<TrackControlPoint>,
    pub points: Vec<Vec3>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct TrackControlPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub flags: TrackControlPointFlags,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct TrackControlPointFlags(u32);

bitflags! {
    impl TrackControlPointFlags: u32 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const UNKNOWN_FLAG_2 = 1 << 1;
    }
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
    fn test_get_base_m3x_model_file_name() {
        let project = Project {
            base_model_file_name: "base.M3D".to_string(),
            ..Default::default()
        };

        assert_eq!(project.get_base_m3x_model_file_name(), "base.M3X");
    }

    #[test]
    fn test_get_water_m3x_model_file_name() {
        let project = Project {
            water_model_file_name: Some("_7water.M3D".to_string()),
            ..Default::default()
        };

        assert_eq!(
            project.get_water_m3x_model_file_name(),
            Some("_7water.M3X".to_string())
        );
    }

    fn roundtrip_test(original_bytes: &[u8], p: &Project) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(p).unwrap();

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
    fn test_decode_b1_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "B1_01.PRJ",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let project = Decoder::new(file).decode().unwrap();

        assert_eq!(project.base_model_file_name, "base.M3D");
        assert_eq!(
            project.water_model_file_name,
            Some("_7water.M3D".to_string())
        );
        assert_eq!(project.furniture_model_file_names.len(), 10);
        assert_eq!(project.furniture_model_file_names[0], "_4barrel.m3d");
        assert_eq!(project.furniture_model_file_names[9], "_khut3_d.m3d");
        assert_eq!(project.instances.len(), 37);
        assert_eq!(project.terrain.width, 184);
        assert_eq!(project.terrain.height, 200);
        assert_eq!(project.attributes.width, 184);
        assert_eq!(project.attributes.height, 200);
        assert_eq!(project.background_music_script_file_name, "battle1.fsm");
        assert_eq!(project.tracks.len(), 2);
        assert_eq!(project.tracks[0].control_points.len(), 6);
        assert_eq!(project.tracks[0].points.len(), 135);
        assert_eq!(project.tracks[1].control_points.len(), 6);
        assert_eq!(project.tracks[1].points.len(), 116);

        // TODO: Not sure if the heights here are correct.
        {
            // Line segment 1 of 'Sightedge' region from B1_01.BTB.
            assert_eq!(project.terrain.get_height(1, 8, 1592), 9.); // start pos
            assert_eq!(project.terrain.get_height(1, 8, 408), 19.); // end pos

            // A point with a negative x.
            assert_eq!(project.terrain.get_height(1, 1448, 1856), 50.); // start pos
            assert_eq!(project.terrain.get_height(1, -248, 1856), 48.); // end pos
        }

        roundtrip_test(&original_bytes, &project);
    }

    #[test]
    fn test_decode_mb4_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B4_01",
            "MB4_01.PRJ",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let project = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &project);
    }

    #[test]
    fn test_decode_b4_09() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B4_09",
            "B4_09.PRJ",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let project = Decoder::new(file).decode().unwrap();

        assert_eq!(project.water_model_file_name, None); // doesn't have a water model

        roundtrip_test(&original_bytes, &project);
    }

    #[test]
    fn test_decode_b5_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B5_01",
            "B5_01.PRJ",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let project = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &project);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "projects"]
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
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_uppercase() == "PRJ" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let original_bytes = std::fs::read(path).unwrap();

                    let file = File::open(path).unwrap();
                    let project = Decoder::new(file).decode().unwrap();

                    roundtrip_test(&original_bytes, &project);

                    // Each project should have 2 tracks.
                    assert_eq!(project.tracks.len(), 2);

                    // Each track should have 6 control points.
                    for track in &project.tracks {
                        assert_eq!(track.control_points.len(), 6);
                    }

                    // Each instance with a GFX code should have a furniture
                    // model slot, i.e. instances with GFX always have an
                    // associated furniture model.
                    for instance in &project.instances {
                        assert!(
                            instance.gfx_code == 0 || instance.furniture_model_slot != 0,
                            "instance with GFX code {} has no furniture model slot",
                            instance.gfx_code
                        );
                    }

                    let has_invalid_offset_index_in_heightmap1 =
                        project.terrain.heightmap1_blocks.iter().any(|block| {
                            block.offset_index as usize >= project.terrain.offsets.len()
                        });
                    assert!(
                        !has_invalid_offset_index_in_heightmap1,
                        "found a block with an invalid offset index in heightmap1"
                    );

                    let has_invalid_offset_index_in_heightmap2 =
                        project.terrain.heightmap2_blocks.iter().any(|block| {
                            block.offset_index as usize >= project.terrain.offsets.len()
                        });
                    assert!(
                        !has_invalid_offset_index_in_heightmap2,
                        "found a block with an invalid offset index in heightmap2"
                    );

                    let output_path =
                        append_ext("ron", root_output_dir.join(path.file_name().unwrap()));
                    let mut output_file = File::create(output_path).unwrap();
                    ron::ser::to_writer_pretty(&mut output_file, &project, Default::default())
                        .unwrap();

                    // Write out both heightmap images.
                    {
                        let output_dir = root_output_dir.join("heightmaps");
                        std::fs::create_dir_all(&output_dir).unwrap();

                        for map_num in 1..=2 {
                            let img = if map_num == 1 {
                                project.terrain.heightmap1_image()
                            } else {
                                project.terrain.heightmap2_image()
                            };

                            let output_path = output_dir
                                .join(path.file_stem().unwrap())
                                .with_extension(format!("map{}.png", map_num));

                            img.save(output_path).unwrap();
                        }
                    }
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
