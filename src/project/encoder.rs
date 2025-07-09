use super::*;
use decoder::{
    ATTRIBUTES_BLOCK_ID, BASE_BLOCK_ID, EDIT_BLOCK_ID, EXCL_BLOCK_ID, FORMAT, FURNITURE_BLOCK_ID,
    INSTANCES_BLOCK_ID, INSTANCE_SIZE_BYTES, MUSIC_BLOCK_DATA_SIZE_BYTES, MUSIC_BLOCK_ID,
    TERRAIN_BLOCK_HEADER_SIZE_BYTES, TERRAIN_BLOCK_ID, TRACKS_BLOCK_ID, WATER_BLOCK_ID,
};
use encoding_rs::WINDOWS_1252;
use std::{
    ffi::CString,
    io::{BufWriter, Write},
    mem::size_of,
};

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
    StringTooLong,
    HeightmapBlockCountMismatch,
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
            EncodeError::IoError(e) => write!(f, "IO error: {e}"),
            EncodeError::InvalidString => write!(f, "invalid string"),
            EncodeError::StringTooLong => write!(f, "string too long"),
            EncodeError::HeightmapBlockCountMismatch => write!(f, "heightmap block count mismatch"),
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

    pub fn encode(&mut self, p: &Project) -> Result<(), EncodeError> {
        self.write_header()?;
        self.write_base(p)?;
        self.write_water(p)?;
        self.write_furniture(p)?;
        self.write_instances(&p.instances)?;
        self.write_terrain(&p.terrain)?;
        self.write_attributes(&p.attributes)?;
        self.write_excl(&p.excl)?;
        self.write_music(p)?;
        self.write_tracks(p)?;
        self.write_edit(p)?;
        Ok(())
    }

    fn write_header(&mut self) -> Result<(), EncodeError> {
        self.write_string(FORMAT)?;
        Ok(())
    }

    fn write_base(&mut self, p: &Project) -> Result<(), EncodeError> {
        let c_string = self.make_c_string(&p.base_model_file_name)?;
        let bytes = c_string.as_bytes_with_nul();

        self.write_block_header(BASE_BLOCK_ID, bytes.len() as u32)?;
        self.writer.write_all(bytes)?;

        Ok(())
    }

    fn write_water(&mut self, p: &Project) -> Result<(), EncodeError> {
        let c_string = p
            .water_model_file_name
            .as_ref()
            .map(|s| self.make_c_string(s))
            .transpose()?;
        let bytes = match &c_string {
            Some(c_str) => c_str.as_bytes_with_nul(),
            None => &[0u8],
        };

        self.write_block_header(WATER_BLOCK_ID, bytes.len() as u32)?;
        self.writer.write_all(bytes)?;

        Ok(())
    }

    fn write_furniture(&mut self, p: &Project) -> Result<(), EncodeError> {
        let file_names: Result<Vec<Vec<u8>>, EncodeError> = p
            .furniture_model_file_names
            .iter()
            .map(|s| {
                let c_string = self.make_c_string(s)?;
                Ok(c_string.into_bytes_with_nul())
            })
            .collect();

        let file_names = file_names?;

        let mut data_size_bytes = 0;
        for file_name in &file_names {
            // A u32 for the length of the filename.
            data_size_bytes += size_of::<u32>() + file_name.len();
        }

        let count = file_names.len();

        self.write_block_header(
            FURNITURE_BLOCK_ID,
            (data_size_bytes - (4 * count) + 4) as u32,
        )?;
        self.writer.write_all(&(count as u32).to_le_bytes())?;

        for file_name in &file_names {
            self.writer
                .write_all(&(file_name.len() as u32).to_le_bytes())?;
            self.writer.write_all(file_name.as_slice())?;
        }

        Ok(())
    }

    fn write_instances(&mut self, instances: &Vec<Instance>) -> Result<(), EncodeError> {
        let count = instances.len() as u32;
        let instance_size_bytes = INSTANCE_SIZE_BYTES as u32;
        let data_size_bytes = instance_size_bytes * count;

        self.write_block_header(INSTANCES_BLOCK_ID, data_size_bytes)?;
        self.writer.write_all(&count.to_le_bytes())?;
        self.writer.write_all(&instance_size_bytes.to_le_bytes())?;

        for instance in instances {
            self.write_instance(instance)?;
        }

        Ok(())
    }

    fn write_instance(&mut self, i: &Instance) -> Result<(), EncodeError> {
        self.writer.write_all(&i.prev.to_le_bytes())?;
        self.writer.write_all(&i.next.to_le_bytes())?;
        self.writer.write_all(&i.selected.to_le_bytes())?;
        self.writer
            .write_all(&(if i.exclude_from_terrain { 1u32 } else { 0u32 }).to_le_bytes())?;
        self.write_dvec3_from_i32s(&i.position, 1024.0)?;
        self.write_dvec3_from_u32s(&i.rotation, 4096.0)?;
        self.write_dvec3_from_i32s(&i.aabb_min, 1024.0)?;
        self.write_dvec3_from_i32s(&i.aabb_max, 1024.0)?;
        self.writer
            .write_all(&i.furniture_model_slot.to_le_bytes())?;
        self.writer.write_all(&i.model_id.to_le_bytes())?;
        self.writer.write_all(&i.attackable.to_le_bytes())?;
        self.writer.write_all(&i.toughness.to_le_bytes())?;
        self.writer.write_all(&i.wounds.to_le_bytes())?;
        self.writer.write_all(&i.unknown1.to_le_bytes())?;
        self.writer.write_all(&i.owner_unit_index.to_le_bytes())?;
        self.writer.write_all(&i.burnable.to_le_bytes())?;
        self.writer.write_all(&i.sfx_code.to_le_bytes())?;
        self.writer.write_all(&i.gfx_code.to_le_bytes())?;
        self.writer.write_all(&i.locked.to_le_bytes())?;
        self.writer.write_all(
            &(if i.exclude_from_terrain_shadow {
                1u32
            } else {
                0u32
            })
            .to_le_bytes(),
        )?;
        self.writer
            .write_all(&(if i.exclude_from_walk { 1u32 } else { 0u32 }).to_le_bytes())?;
        self.writer.write_all(&i.magic_item_id.to_le_bytes())?;
        self.writer
            .write_all(&i.particle_effect_code.to_le_bytes())?;
        self.writer
            .write_all(&i.furniture_dead_model_slot.to_le_bytes())?;
        self.writer.write_all(&i.dead_model_id.to_le_bytes())?;
        self.writer.write_all(&i.light.to_le_bytes())?;
        self.writer.write_all(&i.light_radius.to_le_bytes())?;
        self.writer.write_all(&i.light_ambient.to_le_bytes())?;
        self.writer.write_all(&i.unknown2.to_le_bytes())?;
        self.writer.write_all(&i.unknown3.to_le_bytes())?;

        Ok(())
    }

    fn write_terrain(&mut self, t: &Terrain) -> Result<(), EncodeError> {
        // Make sure the block counts are the same for both heightmaps.
        if t.heightmap1_blocks.len() != t.heightmap2_blocks.len() {
            return Err(EncodeError::HeightmapBlockCountMismatch);
        }

        // Write the header.
        let heightmap_blocks_size_bytes = t.heightmap1_blocks.len() * size_of::<TerrainBlock>();
        let heightmaps_blocks_size_bytes = 2 * heightmap_blocks_size_bytes;
        let height_offsets_size_bytes = size_of::<u32>() + (t.height_offsets.len() * 64);
        let total_size_bytes = TERRAIN_BLOCK_HEADER_SIZE_BYTES - (2 * size_of::<u32>())
            + heightmap_blocks_size_bytes // only includes one of the heightmaps for some reason
            + height_offsets_size_bytes;
        self.write_block_header(TERRAIN_BLOCK_ID, total_size_bytes as u32)?;

        self.writer.write_all(&t.width.to_le_bytes())?;
        self.writer.write_all(&t.height.to_le_bytes())?;
        self.writer
            .write_all(&(t.height_offsets.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(t.heightmap1_blocks.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(heightmaps_blocks_size_bytes as u32).to_le_bytes())?;

        // Write blocks.
        self.write_heightmap_blocks(&t.heightmap1_blocks)?;
        self.write_heightmap_blocks(&t.heightmap2_blocks)?;

        // Write height offsets.
        let height_offsets_size_bytes = t.height_offsets.len() * 64;
        self.writer
            .write_all(&(height_offsets_size_bytes as u32).to_le_bytes())?;
        for offsets in &t.height_offsets {
            self.writer.write_all(offsets)?;
        }

        Ok(())
    }

    fn write_heightmap_blocks(&mut self, blocks: &Vec<TerrainBlock>) -> Result<(), EncodeError> {
        for block in blocks {
            let height_offsets_index = block.height_offsets_index * 64;
            self.writer.write_all(&block.base_height.to_le_bytes())?;
            self.writer.write_all(&height_offsets_index.to_le_bytes())?;
        }

        Ok(())
    }

    fn write_attributes(&mut self, a: &Attributes) -> Result<(), EncodeError> {
        // 2 u32s for width and height, and the rest is unknown.
        //
        // Stored size is short by 64 bytes for some reason.
        let data_size_bytes = (2 * size_of::<u32>() + a.unknown.len()) as u32 - 64;
        self.write_block_header(ATTRIBUTES_BLOCK_ID, data_size_bytes)?;

        self.writer.write_all(&a.width.to_le_bytes())?;
        self.writer.write_all(&a.height.to_le_bytes())?;
        self.writer.write_all(&a.unknown)?;

        Ok(())
    }

    fn write_excl(&mut self, excl: &Excl) -> Result<(), EncodeError> {
        self.write_string(EXCL_BLOCK_ID)?;
        self.writer.write_all(&excl.unknown1.to_le_bytes())?;
        self.writer.write_all(&excl.unknown2)?;
        Ok(())
    }

    fn write_music(&mut self, p: &Project) -> Result<(), EncodeError> {
        self.write_string(MUSIC_BLOCK_ID)?;

        let c_string = self.make_c_string(&p.music_script_file_name)?;
        self.write_c_string_with_limit(&c_string, MUSIC_BLOCK_DATA_SIZE_BYTES)?;

        Ok(())
    }

    fn write_tracks(&mut self, p: &Project) -> Result<(), EncodeError> {
        self.write_string(TRACKS_BLOCK_ID)?;
        self.writer
            .write_all(&(p.tracks.len() as u32).to_le_bytes())?;

        for track in &p.tracks {
            self.write_track_header(track)?;
        }

        for track in &p.tracks {
            self.write_track_points(track)?;
        }

        Ok(())
    }

    fn write_track_header(&mut self, t: &Track) -> Result<(), EncodeError> {
        self.writer
            .write_all(&(t.control_points.len() as u32).to_le_bytes())?;
        self.writer
            .write_all(&(t.points.len() as u32).to_le_bytes())?;

        for cp in &t.control_points {
            self.write_track_control_point(cp)?;
        }

        Ok(())
    }

    fn write_track_control_point(&mut self, cp: &TrackControlPoint) -> Result<(), EncodeError> {
        self.write_vec3_from_i32s(&Vec3::new(cp.x, cp.y, cp.z), 1024.0)?;
        self.writer.write_all(&cp.flags.bits().to_le_bytes())?;

        Ok(())
    }

    fn write_track_points(&mut self, t: &Track) -> Result<(), EncodeError> {
        for p in &t.points {
            self.write_vec3_from_i32s(p, 1024.0)?;
        }

        Ok(())
    }

    fn write_edit(&mut self, p: &Project) -> Result<(), EncodeError> {
        self.write_string(EDIT_BLOCK_ID)?;
        self.writer.write_all(&p.edit)?;

        Ok(())
    }

    fn write_block_header(&mut self, s: &str, data_size: u32) -> Result<(), EncodeError> {
        self.write_string(s)?;
        self.writer.write_all(&data_size.to_le_bytes())?;
        Ok(())
    }

    fn write_dvec3_from_u32s(&mut self, v: &DVec3, mul: f64) -> Result<(), EncodeError> {
        self.writer.write_all(&((v.x * mul) as u32).to_le_bytes())?;
        self.writer.write_all(&((v.y * mul) as u32).to_le_bytes())?;
        self.writer.write_all(&((v.z * mul) as u32).to_le_bytes())?;
        Ok(())
    }

    fn write_dvec3_from_i32s(&mut self, v: &DVec3, mul: f64) -> Result<(), EncodeError> {
        self.writer.write_all(&((v.x * mul) as i32).to_le_bytes())?;
        self.writer.write_all(&((v.y * mul) as i32).to_le_bytes())?;
        self.writer.write_all(&((v.z * mul) as i32).to_le_bytes())?;
        Ok(())
    }

    fn write_vec3_from_i32s(&mut self, v: &Vec3, mul: f32) -> Result<(), EncodeError> {
        self.writer.write_all(&((v.x * mul) as i32).to_le_bytes())?;
        self.writer.write_all(&((v.y * mul) as i32).to_le_bytes())?;
        self.writer.write_all(&((v.z * mul) as i32).to_le_bytes())?;
        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<(), EncodeError> {
        let c_string = self.make_c_string(s)?;
        let bytes = c_string.as_bytes();

        self.writer.write_all(bytes)?;

        Ok(())
    }

    fn write_c_string_with_limit(&mut self, s: &CString, limit: usize) -> Result<(), EncodeError> {
        let bytes = s.as_bytes_with_nul();

        if bytes.len() > limit {
            return Err(EncodeError::StringTooLong);
        }

        self.writer.write_all(bytes)?;

        let padding_size_bytes = limit - bytes.len();
        let padding = vec![0; padding_size_bytes];
        self.writer.write_all(&padding)?;

        Ok(())
    }

    fn make_c_string(&mut self, s: &str) -> Result<CString, EncodeError> {
        let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(s);
        let c_string = CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
        Ok(c_string)
    }
}
