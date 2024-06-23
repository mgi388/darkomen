mod decoder;

use bitflags::bitflags;
use glam::Vec3;
use image::{DynamicImage, GenericImage, Rgba};
use serde::Serialize;

pub use decoder::{DecodeError, Decoder};

#[derive(Clone, Debug, Serialize)]
pub struct Project {
    pub base_model_file_name: String,
    pub water_model_file_name: Option<String>,
    pub furniture_model_file_names: Vec<String>,
    pub instances: Vec<Instance>,
    pub terrain: Terrain,
    pub attributes: Attributes,
    excl: Vec<u8>,
    pub background_music_script_file_name: String,
    pub tracks: Vec<Track>,
    edit: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Instance {
    prev: i32,
    next: i32,
    selected: i32,
    pub exclude_from_terrain: i32,
    pub position: Vec3,
    pub rotation: Vec3,
    pub aabb_min: Vec3,
    pub aabb_max: Vec3,
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

#[derive(Clone, Debug, Serialize)]
pub struct Terrain {
    pub width: u32,
    pub height: u32,
    // A list of large blocks for the first heightmap.
    pub heightmap1_blocks: Vec<TerrainBlock>,
    // A list of large blocks for the second heightmap.
    pub heightmap2_blocks: Vec<TerrainBlock>,
    // Offsets is a list of offsets for 8x8 block. Height offset for each block
    // based on minimum height.
    pub offsets: Vec<Vec<u8>>,
}

impl Terrain {
    pub fn get_heightmap1_image(&self) -> DynamicImage {
        self.get_heightmap_image(&self.heightmap1_blocks)
    }

    pub fn get_heightmap2_image(&self) -> DynamicImage {
        self.get_heightmap_image(&self.heightmap2_blocks)
    }

    /// TODO: Not really working perfectly.
    fn get_heightmap_image(&self, blocks: &Vec<TerrainBlock>) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(self.width, self.height);

        let mut row = 0;
        let mut col = 0;

        for block in blocks {
            let offsets = &self.offsets[block.offset_index as usize];

            col += 1;
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

                    let color_part1 = offsets[(x + y * 8) as usize] as u32;
                    let color_part2 = block.minimum / 257;

                    // TODO: Clamped this to avoid panics, but possibly
                    // indicates a bug.
                    let color = (color_part1 + color_part2).clamp(0, 255) as u8;

                    img.put_pixel(target_x, target_y, Rgba([color, color, color, 255]));
                }
            }
        }

        img
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
        (additional as f32 / 8.) + (b.minimum as f32 / 1024.)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TerrainBlock {
    pub minimum: u32,
    pub offset_index: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Attributes {
    pub width: u32,
    pub height: u32,
    pub unknown: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Track {
    pub control_points: Vec<TrackControlPoint>,
    pub points: Vec<Vec3>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrackControlPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub flags: TrackControlPointFlags,
}

bitflags! {
    #[derive(Clone, Debug, Serialize)]
    pub struct TrackControlPointFlags: u8 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const UNKNOWN_FLAG_2 = 1 << 1;
    }
}
