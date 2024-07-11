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
const FORMAT: &str = "Dark Omen Battle file 1.10      ";

const BASE_BLOCK_ID: &str = "BASE";
const WATER_BLOCK_ID: &str = "WATR";
const FURNITURE_BLOCK_ID: &str = "FURN";
const INSTANCES_BLOCK_ID: &str = "INST";
const TERRAIN_BLOCK_ID: &str = "TERR";
const ATTRIBUTES_BLOCK_ID: &str = "ATTR";
const EXCL_BLOCK_ID: &str = "EXCL";
const MUSIC_BLOCK_ID: &str = "MUSC";
const TRACKS_BLOCK_ID: &str = "TRAC";
const EDIT_BLOCK_ID: &str = "EDIT";

const HEADER_SIZE: usize = 32;
const BLOCK_HEADER_SIZE: usize = 8;
const FURNITURE_BLOCK_HEADER_SIZE: usize = BLOCK_HEADER_SIZE + 4;
const INSTANCES_BLOCK_HEADER_SIZE: usize = BLOCK_HEADER_SIZE + 4 + 4;
const TERRAIN_BLOCK_HEADER_SIZE: usize = BLOCK_HEADER_SIZE + 20;
const ATTRIBUTES_BLOCK_HEADER_SIZE: usize = BLOCK_HEADER_SIZE;
const EXCL_BLOCK_HEADER_SIZE: usize = 8;
const MUSIC_BLOCK_DATA_SIZE: usize = 20;
const TRACKS_BLOCK_HEADER_SIZE: usize = 8;

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
        let background_music_script_file_name = self.read_music()?;
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
            background_music_script_file_name,
            tracks,
            edit,
        })
    }

    fn decode_header(&mut self) -> Result<(), DecodeError> {
        let mut buf = [0; HEADER_SIZE];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..HEADER_SIZE] != FORMAT.as_bytes() {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&buf[0..HEADER_SIZE]).to_string(),
            ));
        }

        Ok(())
    }

    fn read_block(&mut self, id: &str) -> Result<Vec<u8>, DecodeError> {
        let mut buf = vec![0; BLOCK_HEADER_SIZE];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..4] != id.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let data_size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
        let mut data = vec![0; data_size];
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

        let file_name_str = CStr::from_bytes_with_nul(&file_name)
            .map_err(|_| DecodeError::InvalidString)?
            .to_string_lossy()
            .into_owned();

        Ok(if file_name_str.is_empty() {
            None
        } else {
            Some(file_name_str)
        })
    }

    fn read_furniture_block(&mut self) -> Result<Vec<String>, DecodeError> {
        let mut buf = vec![0; FURNITURE_BLOCK_HEADER_SIZE];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..4] != FURNITURE_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let count = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]) as usize;
        let data_size =
            u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize + (4 * count) - 4;

        let mut data = vec![0; data_size];
        self.reader.read_exact(&mut data)?;

        let mut pos = 0;
        let mut file_names = Vec::with_capacity(count);
        for _ in 0..count {
            let size = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            let file_name = CStr::from_bytes_with_nul(&data[pos + 4..pos + 4 + size])
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            file_names.push(file_name);
            pos += 4 + size;
        }

        Ok(file_names)
    }

    fn read_instances(&mut self) -> Result<Vec<Instance>, DecodeError> {
        let mut header = vec![0; INSTANCES_BLOCK_HEADER_SIZE];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != INSTANCES_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let count = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
        let instance_size = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;

        let mut buf = vec![0; size];
        self.reader.read_exact(&mut buf)?;

        let mut instances = Vec::with_capacity(count);
        for i in 0..count {
            let b = &buf[i * instance_size..(i + 1) * instance_size];
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
            position: Vec3::new(
                u32::from_le_bytes(buf[16..20].try_into().unwrap()) as f32 / 1024.,
                u32::from_le_bytes(buf[20..24].try_into().unwrap()) as f32 / 1024.,
                u32::from_le_bytes(buf[24..28].try_into().unwrap()) as f32 / 1024.,
            ),
            rotation: Vec3::new(
                u32::from_le_bytes(buf[28..32].try_into().unwrap()) as f32 / 4096.,
                u32::from_le_bytes(buf[32..36].try_into().unwrap()) as f32 / 4096.,
                u32::from_le_bytes(buf[36..40].try_into().unwrap()) as f32 / 4096.,
            ),
            aabb_min: Vec3::new(
                i32::from_le_bytes(buf[40..44].try_into().unwrap()) as f32 / 1024.,
                i32::from_le_bytes(buf[44..48].try_into().unwrap()) as f32 / 1024.,
                i32::from_le_bytes(buf[48..52].try_into().unwrap()) as f32 / 1024.,
            ),
            aabb_max: Vec3::new(
                i32::from_le_bytes(buf[52..56].try_into().unwrap()) as f32 / 1024.,
                i32::from_le_bytes(buf[56..60].try_into().unwrap()) as f32 / 1024.,
                i32::from_le_bytes(buf[60..64].try_into().unwrap()) as f32 / 1024.,
            ),
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
            magic_item_code: u32::from_le_bytes(buf[116..120].try_into().unwrap()),
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
        let mut header = vec![0; TERRAIN_BLOCK_HEADER_SIZE];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != TERRAIN_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let _size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize; // size, not used
        let width = u32::from_le_bytes(header[8..12].try_into().unwrap());
        let height = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let compressed_block_count =
            u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;
        let uncompressed_block_count =
            u32::from_le_bytes(header[20..24].try_into().unwrap()) as usize;
        let heightmaps_size = u32::from_le_bytes(header[24..28].try_into().unwrap()) as usize; // size in bytes of chunk that contains both heightmaps

        // This check just helps prove that the size of the heightmaps chunk
        // also lets us get the uncompressed block count.
        if heightmaps_size / 2 / size_of::<TerrainBlock>() != uncompressed_block_count {
            return Err(DecodeError::Invalid(
                "uncompressed block count and map block size mismatch".to_string(),
            ));
        }

        // First heightmap.
        let mut buf = vec![0; heightmaps_size / 2]; // size of one heightmap's blocks chunk
        self.reader.read_exact(&mut buf)?;

        let mut heightmap1_blocks = Vec::with_capacity(uncompressed_block_count);
        for i in 0..uncompressed_block_count {
            let minimum = u32::from_le_bytes(buf[i * 8..i * 8 + 4].try_into().unwrap());
            let offset_index = u32::from_le_bytes(buf[i * 8 + 4..i * 8 + 8].try_into().unwrap());
            if offset_index % 64 != 0 {
                // TODO: Wrong error type.
                return Err(DecodeError::InvalidBlockFormat(format!(
                    "heightmap 1: offset index is not a multiple of 64, got {}",
                    offset_index
                )));
            }
            let offset_index = offset_index / 64;
            heightmap1_blocks.push(TerrainBlock {
                minimum,
                offset_index,
            });
        }

        // Second heightmap.
        let mut buf = vec![0; heightmaps_size / 2]; // size of one heightmap's blocks chunk
        self.reader.read_exact(&mut buf)?;

        let mut heightmap2_blocks = Vec::with_capacity(uncompressed_block_count);
        for i in 0..uncompressed_block_count {
            let minimum = u32::from_le_bytes(buf[i * 8..i * 8 + 4].try_into().unwrap());
            let offset_index = u32::from_le_bytes(buf[i * 8 + 4..i * 8 + 8].try_into().unwrap());
            if offset_index % 64 != 0 {
                // TODO: Wrong error type.
                return Err(DecodeError::InvalidBlockFormat(format!(
                    "heightmap 2: offset index is not a multiple of 64, got {}",
                    offset_index
                )));
            }
            let offset_index = offset_index / 64;
            heightmap2_blocks.push(TerrainBlock {
                minimum,
                offset_index,
            });
        }

        // Read offsets.
        let mut buf = vec![0; 4];
        self.reader.read_exact(&mut buf)?;
        let offsets_size = u32::from_le_bytes(buf.try_into().unwrap()) as usize;

        if compressed_block_count * 64 != offsets_size {
            // TODO: Wrong error type.
            return Err(DecodeError::InvalidBlockFormat(format!(
                "compressed block count and offsets size mismatch: got {}, {}",
                compressed_block_count, offsets_size
            )));
        }

        let mut buf = vec![0; offsets_size];
        self.reader.read_exact(&mut buf)?;

        let mut offsets = Vec::with_capacity(compressed_block_count);
        for i in 0..compressed_block_count {
            offsets.push(buf[i * 64..(i + 1) * 64].to_vec());
        }

        Ok(Terrain {
            width,
            height,
            heightmap1_blocks,
            heightmap2_blocks,
            offsets,
        })
    }

    fn read_attributes(&mut self) -> Result<Attributes, DecodeError> {
        let mut header = vec![0; ATTRIBUTES_BLOCK_HEADER_SIZE];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != ATTRIBUTES_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize + 64; // stored size is short by 64 bytes for some reason

        let mut buf = vec![0; size];
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

    fn read_excl(&mut self) -> Result<Vec<u8>, DecodeError> {
        let mut header = vec![0; EXCL_BLOCK_HEADER_SIZE];
        self.reader.read_exact(&mut header)?;

        if &header[0..4] != EXCL_BLOCK_ID.as_bytes() {
            return Err(DecodeError::InvalidBlockFormat(
                String::from_utf8_lossy(&header[0..4]).to_string(),
            ));
        }

        let _count = u32::from_le_bytes(
            header[4..4 + size_of::<u32>()]
                .try_into()
                .map_err(|_| DecodeError::InvalidData)?,
        ) as usize;

        // It's not possible to know the size of the EXCL block data, so read
        // until the next block.
        let buf = self.read_until_block(MUSIC_BLOCK_ID)?;

        Ok(buf)
    }

    fn read_music(&mut self) -> Result<String, DecodeError> {
        // Note: It's expected that the EXCL block was read before this because
        // it consumes the MUSC header.
        let mut buf = vec![0; MUSIC_BLOCK_DATA_SIZE];
        self.reader.read_exact(&mut buf)?;

        Ok(
            String::from_utf8_lossy(CStr::from_bytes_until_nul(&buf).unwrap().to_bytes())
                .to_string(),
        )
    }

    fn read_tracks(&mut self) -> Result<Vec<Track>, DecodeError> {
        let mut header = vec![0; TRACKS_BLOCK_HEADER_SIZE];
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
}
