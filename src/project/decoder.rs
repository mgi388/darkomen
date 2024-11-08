use super::*;
use std::{
    ffi::CStr,
    fmt,
    io::{Error as IoError, Read, Seek},
    mem::size_of,
};

/// The format ID used in all .PRJ files.
///
/// Trailing spaces intended.
pub(crate) const FORMAT: &str = "Dark Omen Battle file 1.10      ";

pub(crate) const BASE_BLOCK_ID: &str = "BASE";
pub(crate) const WATER_BLOCK_ID: &str = "WATR";
pub(crate) const FURNITURE_BLOCK_ID: &str = "FURN";
pub(crate) const INSTANCES_BLOCK_ID: &str = "INST";
pub(crate) const TERRAIN_BLOCK_ID: &str = "TERR";
pub(crate) const ATTRIBUTES_BLOCK_ID: &str = "ATTR";
pub(crate) const EXCL_BLOCK_ID: &str = "EXCL";
pub(crate) const MUSIC_BLOCK_ID: &str = "MUSC";
pub(crate) const TRACKS_BLOCK_ID: &str = "TRAC";
pub(crate) const EDIT_BLOCK_ID: &str = "EDIT";

pub(crate) const HEADER_SIZE_BYTES: usize = 32;
pub(crate) const BLOCK_HEADER_SIZE_BYTES: usize = 8;
pub(crate) const FURNITURE_BLOCK_HEADER_SIZE_BYTES: usize = BLOCK_HEADER_SIZE_BYTES + 4;
pub(crate) const INSTANCES_BLOCK_HEADER_SIZE_BYTES: usize = BLOCK_HEADER_SIZE_BYTES + 4 + 4;
pub(crate) const INSTANCE_SIZE_BYTES: usize = 152;
pub(crate) const TERRAIN_BLOCK_HEADER_SIZE_BYTES: usize = BLOCK_HEADER_SIZE_BYTES + 20;
pub(crate) const ATTRIBUTES_BLOCK_HEADER_SIZE_BYTES: usize = BLOCK_HEADER_SIZE_BYTES;
pub(crate) const EXCL_BLOCK_HEADER_SIZE_BYTES: usize = 8;
pub(crate) const MUSIC_BLOCK_DATA_SIZE_BYTES: usize = 20;
pub(crate) const TRACKS_BLOCK_HEADER_SIZE_BYTES: usize = 8;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    Invalid(String),
    InvalidFormat(String),
    InvalidBlockFormat(String),
    InvalidString,
    InvalidData,
    InvalidTerrainBlockCount(usize),
    InvalidTrackControlPointFlags(i32),
    InvalidHeightOffsetsIndex(u32),
    InvalidHeightOffsetsSize(usize, usize),
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
            DecodeError::Invalid(s) => write!(f, "invalid: {}", s),
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {}", s),
            DecodeError::InvalidBlockFormat(s) => write!(f, "invalid block format: {}", s),
            DecodeError::InvalidString => write!(f, "invalid string"),
            DecodeError::InvalidData => write!(f, "invalid data"),
            DecodeError::InvalidTerrainBlockCount(c) => {
                write!(f, "invalid terrain block count: {}", c)
            }
            DecodeError::InvalidTrackControlPointFlags(flags) => {
                write!(f, "invalid track control point flags: {}", flags)
            }
            DecodeError::InvalidHeightOffsetsIndex(index) => {
                write!(f, "height offsets index {} is not a multiple of 64", index)
            }
            DecodeError::InvalidHeightOffsetsSize(offset_count, height_offsets_size_bytes) => {
                write!(
                    f,
                    "invalid height offsets size {}, should be offset count ({}) x 64",
                    height_offsets_size_bytes, offset_count
                )
            }
        }
    }
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

    pub fn decode(&mut self) -> Result<Project, DecodeError> {
        self.decode_header()?;

        let base_model_file_name = self.read_base()?;
        let water_model_file_name = self.read_water()?;
        let furniture_model_file_names = self.read_furniture_block()?;
        let instances = self.read_instances()?;
        let terrain = self.read_terrain()?;
        let attributes = self.read_attributes()?;
        let excl = self.read_excl()?;
        let music_script_file_name = self.read_music()?;
        let tracks = self.read_tracks()?;
        let edit = self.read_edit()?;

        Ok(Project {
            base_model_file_name,
            water_model_file_name,
            furniture_model_file_names,
            instances,
            terrain,
            attributes,
            excl,
            music_script_file_name,
            tracks,
            edit,
        })
    }

    fn decode_header(&mut self) -> Result<(), DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..HEADER_SIZE_BYTES] != FORMAT.as_bytes() {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&buf[0..HEADER_SIZE_BYTES]).to_string(),
            ));
        }

        Ok(())
    }

    fn read_block(&mut self, id: &str) -> Result<Vec<u8>, DecodeError> {
        let mut buf = vec![0; BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..4] != id.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let data_size_bytes = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
        let mut data = vec![0; data_size_bytes];
        self.reader.read_exact(&mut data)?;

        Ok(data)
    }

    fn read_base(&mut self) -> Result<String, DecodeError> {
        let file_name = self.read_block(BASE_BLOCK_ID)?;

        Ok(CStr::from_bytes_with_nul(&file_name)
            .map_err(|_| DecodeError::InvalidString)?
            .to_string_lossy()
            .into_owned())
    }

    fn read_water(&mut self) -> Result<Option<String>, DecodeError> {
        let file_name = self.read_block(WATER_BLOCK_ID)?;

        let file_name_string = CStr::from_bytes_with_nul(&file_name)
            .map_err(|_| DecodeError::InvalidString)?
            .to_string_lossy()
            .into_owned();

        Ok(if file_name_string.is_empty() {
            None
        } else {
            Some(file_name_string)
        })
    }

    fn read_furniture_block(&mut self) -> Result<Vec<String>, DecodeError> {
        let mut buf = vec![0; FURNITURE_BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..4] != FURNITURE_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let count = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]) as usize;
        let data_size_bytes =
            u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize + (4 * count) - 4;

        let mut data = vec![0; data_size_bytes];
        self.reader.read_exact(&mut data)?;

        let mut pos = 0;
        let mut file_names = Vec::with_capacity(count);
        for _ in 0..count {
            let size_bytes = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            let file_name = CStr::from_bytes_with_nul(&data[pos + 4..pos + 4 + size_bytes])
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            file_names.push(file_name);
            pos += 4 + size_bytes;
        }

        Ok(file_names)
    }

    fn read_instances(&mut self) -> Result<Vec<Instance>, DecodeError> {
        let mut header = vec![0; INSTANCES_BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != INSTANCES_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let size_bytes = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let count = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
        let instance_size_bytes = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;

        let mut buf = vec![0; size_bytes];
        self.reader.read_exact(&mut buf)?;

        let mut instances = Vec::with_capacity(count);
        for i in 0..count {
            let b = &buf[i * instance_size_bytes..(i + 1) * instance_size_bytes];
            instances.push(self.read_instance(b)?);
        }

        Ok(instances)
    }

    fn read_instance(&mut self, buf: &[u8]) -> Result<Instance, DecodeError> {
        Ok(Instance {
            prev: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
            next: i32::from_le_bytes(buf[4..8].try_into().unwrap()),
            selected: i32::from_le_bytes(buf[8..12].try_into().unwrap()),
            exclude_from_terrain: i32::from_le_bytes(buf[12..16].try_into().unwrap()),
            position: self.read_dvec3_from_u32s(&buf[16..28], 1024.)?,
            rotation: self.read_dvec3_from_u32s(&buf[28..40], 4096.)?,
            aabb_min: self.read_dvec3_from_i32s(&buf[40..52], 1024.)?,
            aabb_max: self.read_dvec3_from_i32s(&buf[52..64], 1024.)?,
            furniture_model_slot: u32::from_le_bytes(buf[64..68].try_into().unwrap()),
            model_id: i32::from_le_bytes(buf[68..72].try_into().unwrap()),
            attackable: i32::from_le_bytes(buf[72..76].try_into().unwrap()),
            toughness: i32::from_le_bytes(buf[76..80].try_into().unwrap()),
            wounds: i32::from_le_bytes(buf[80..84].try_into().unwrap()),
            unknown1: i32::from_le_bytes(buf[84..88].try_into().unwrap()),
            owner_unit_index: i32::from_le_bytes(buf[88..92].try_into().unwrap()),
            burnable: i32::from_le_bytes(buf[92..96].try_into().unwrap()),
            sfx_code: u32::from_le_bytes(buf[96..100].try_into().unwrap()),
            gfx_code: u32::from_le_bytes(buf[100..104].try_into().unwrap()),
            locked: i32::from_le_bytes(buf[104..108].try_into().unwrap()),
            exclude_from_terrain_shadow: i32::from_le_bytes(buf[108..112].try_into().unwrap()),
            exclude_from_walk: i32::from_le_bytes(buf[112..116].try_into().unwrap()),
            magic_item_id: u32::from_le_bytes(buf[116..120].try_into().unwrap()),
            particle_effect_code: u32::from_le_bytes(buf[120..124].try_into().unwrap()),
            furniture_dead_model_slot: u32::from_le_bytes(buf[124..128].try_into().unwrap()),
            dead_model_id: i32::from_le_bytes(buf[128..132].try_into().unwrap()),
            light: i32::from_le_bytes(buf[132..136].try_into().unwrap()),
            light_radius: i32::from_le_bytes(buf[136..140].try_into().unwrap()),
            light_ambient: i32::from_le_bytes(buf[140..144].try_into().unwrap()),
            unknown2: i32::from_le_bytes(buf[144..148].try_into().unwrap()),
            unknown3: i32::from_le_bytes(buf[148..152].try_into().unwrap()),
        })
    }

    fn read_terrain(&mut self) -> Result<Terrain, DecodeError> {
        let mut header = vec![0; TERRAIN_BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != TERRAIN_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let _total_size_bytes = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize; // total block size in bytes, not used
        let width = u32::from_le_bytes(header[8..12].try_into().unwrap());
        let height = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let offset_count = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;
        let heightmap_block_count = u32::from_le_bytes(header[20..24].try_into().unwrap()) as usize;
        let heightmaps_blocks_size_bytes =
            u32::from_le_bytes(header[24..28].try_into().unwrap()) as usize; // size in bytes of chunk that contains both heightmaps blocks
        let heightmap_blocks_size_bytes = heightmaps_blocks_size_bytes / 2; // size in bytes of one heightmap's block

        // This check just helps prove that the size of the heightmap blocks
        // chunk also lets us get the heightmap block count.
        if heightmap_blocks_size_bytes / size_of::<TerrainBlock>() != heightmap_block_count {
            return Err(DecodeError::Invalid(
                "heightmap block count and heightmap blocks size mismatch".to_string(),
            ));
        }

        // Read first heightmap blocks.
        let heightmap1_blocks = self.read_heightmap_blocks(heightmap_block_count)?;

        // Read second heightmap blocks.
        let heightmap2_blocks = self.read_heightmap_blocks(heightmap_block_count)?;

        // Read height offsets.
        let mut buf = vec![0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;
        let height_offsets_size_bytes = u32::from_le_bytes(buf.try_into().unwrap()) as usize;

        if offset_count * 64 != height_offsets_size_bytes {
            return Err(DecodeError::InvalidHeightOffsetsSize(
                offset_count,
                height_offsets_size_bytes,
            ));
        }

        let mut buf = vec![0; height_offsets_size_bytes];
        self.reader.read_exact(&mut buf)?;

        let mut height_offsets = Vec::with_capacity(offset_count);
        for i in 0..offset_count {
            height_offsets.push(buf[i * 64..(i + 1) * 64].to_vec());
        }

        Ok(Terrain {
            width,
            height,
            heightmap1_blocks,
            heightmap2_blocks,
            height_offsets,
        })
    }

    fn read_heightmap_blocks(&mut self, count: usize) -> Result<Vec<TerrainBlock>, DecodeError> {
        let mut blocks = Vec::with_capacity(count);
        for _ in 0..count {
            blocks.push(self.read_terrain_block()?);
        }
        Ok(blocks)
    }

    fn read_terrain_block(&mut self) -> Result<TerrainBlock, DecodeError> {
        let mut buf = vec![0; size_of::<TerrainBlock>()];
        self.reader.read_exact(&mut buf)?;

        let base_height = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let height_offsets_index = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if height_offsets_index % 64 != 0 {
            return Err(DecodeError::InvalidHeightOffsetsIndex(height_offsets_index));
        }
        let height_offsets_index = height_offsets_index / 64;

        Ok(TerrainBlock {
            base_height,
            height_offsets_index,
        })
    }

    fn read_attributes(&mut self) -> Result<Attributes, DecodeError> {
        let mut header = vec![0; ATTRIBUTES_BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != ATTRIBUTES_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let size_bytes = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize + 64; // stored size is short by 64 bytes for some reason

        let mut buf = vec![0; size_bytes];
        self.reader.read_exact(&mut buf)?;

        let width = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let unknown = buf[8..].to_vec();

        Ok(Attributes {
            width,
            height,
            unknown,
        })
    }

    fn read_excl(&mut self) -> Result<Excl, DecodeError> {
        let mut header = vec![0; EXCL_BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != EXCL_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let unknown1 = u32::from_le_bytes(
            header[4..4 + size_of::<u32>()]
                .try_into()
                .map_err(|_| DecodeError::InvalidData)?,
        );

        // It's not possible to know the size of the EXCL block data, so read
        // until the next block.
        let unknown2 = self.read_until_block(MUSIC_BLOCK_ID)?;

        Ok(Excl { unknown1, unknown2 })
    }

    fn read_music(&mut self) -> Result<String, DecodeError> {
        // Note: It's expected that the EXCL block was read before this because
        // it consumes the MUSC header.
        let mut buf = vec![0; MUSIC_BLOCK_DATA_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        Ok(
            String::from_utf8_lossy(CStr::from_bytes_until_nul(&buf).unwrap().to_bytes())
                .to_string(),
        )
    }

    fn read_tracks(&mut self) -> Result<Vec<Track>, DecodeError> {
        let mut header = vec![0; TRACKS_BLOCK_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != TRACKS_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let track_count =
            u32::from_le_bytes(header[4..4 + size_of::<u32>()].try_into().unwrap()) as usize;

        // It's not possible to know the size of the TRAC block data, so read
        // until the next block.
        let buf = self.read_until_block(EDIT_BLOCK_ID)?;

        let mut tracks = Vec::with_capacity(track_count);

        let mut c: usize = 0; // cursor

        for _ in 0..track_count {
            let control_point_count =
                u32::from_le_bytes(buf[c..c + size_of::<u32>()].try_into().unwrap()) as usize;
            c += size_of::<u32>();

            let point_count =
                u32::from_le_bytes(buf[c..c + size_of::<u32>()].try_into().unwrap()) as usize;
            c += size_of::<u32>();

            let mut control_points = Vec::with_capacity(control_point_count);

            for _ in 0..control_point_count {
                let x = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap()) as f32
                    / 1024.;
                c += size_of::<i32>();
                let y = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap()) as f32
                    / 1024.;
                c += size_of::<i32>();
                let z = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap()) as f32
                    / 1024.;
                c += size_of::<i32>();
                let flags = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap());
                c += size_of::<i32>();

                match flags {
                    0..=2 => {}
                    _ => return Err(DecodeError::InvalidTrackControlPointFlags(flags)),
                }

                control_points.push(TrackControlPoint {
                    x,
                    y,
                    z,
                    flags: TrackControlPointFlags::from_bits(flags as u32)
                        .expect("track control point flags should be valid"),
                });
            }

            tracks.push(Track {
                control_points,
                points: Vec::with_capacity(point_count),
            });
        }

        (0..track_count).for_each(|i| {
            for _ in 0..tracks[i].points.capacity() {
                let x = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap()) as f32
                    / 1024.;
                c += size_of::<i32>();
                let y = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap()) as f32
                    / 1024.;
                c += size_of::<i32>();
                let z = i32::from_le_bytes(buf[c..c + size_of::<i32>()].try_into().unwrap()) as f32
                    / 1024.;
                c += size_of::<i32>();

                tracks[i].points.push(Vec3::new(x, y, z));
            }
        });

        Ok(tracks)
    }

    fn read_edit(&mut self) -> Result<Vec<u8>, DecodeError> {
        // Note: It's expected that the TRAC block was read before this because
        // it consumes the EDIT header.

        // Read until the end of the reader because this is the last block.
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;

        Ok(buf)
    }

    /// Read until the next block with the given ID.
    /// The block ID is not included in the returned buffer.
    fn read_until_block(&mut self, id: &str) -> Result<Vec<u8>, DecodeError> {
        let mut buf = Vec::new();
        let mut last_four = Vec::with_capacity(4);
        let found;

        loop {
            let mut byte = [0; 1];
            self.reader.read_exact(&mut byte)?;

            buf.push(byte[0]);
            if last_four.len() == 4 {
                last_four.remove(0);
            }
            last_four.push(byte[0]);

            if last_four == id.bytes().collect::<Vec<u8>>() {
                found = true;
                break;
            }
        }

        Ok(if found {
            buf[0..buf.len() - id.len()].to_vec()
        } else {
            buf
        })
    }

    fn read_dvec3_from_u32s(&mut self, buf: &[u8], multiplier: f64) -> Result<DVec3, DecodeError> {
        let x = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as f64 / multiplier;
        let y = u32::from_le_bytes(buf[4..8].try_into().unwrap()) as f64 / multiplier;
        let z = u32::from_le_bytes(buf[8..12].try_into().unwrap()) as f64 / multiplier;

        Ok(DVec3::new(x, y, z))
    }

    fn read_dvec3_from_i32s(&mut self, buf: &[u8], multiplier: f64) -> Result<DVec3, DecodeError> {
        let x = i32::from_le_bytes(buf[0..4].try_into().unwrap()) as f64 / multiplier;
        let y = i32::from_le_bytes(buf[4..8].try_into().unwrap()) as f64 / multiplier;
        let z = i32::from_le_bytes(buf[8..12].try_into().unwrap()) as f64 / multiplier;

        Ok(DVec3::new(x, y, z))
    }
}
