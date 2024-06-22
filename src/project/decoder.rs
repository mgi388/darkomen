use std::{
    ffi::CStr,
    fmt,
    io::{Error as IoError, Read, Seek},
    mem::size_of,
};

use super::*;

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

            let instance = Instance {
                prev: i32::from_le_bytes(b[0..4].try_into().unwrap()),
                next: i32::from_le_bytes(b[4..8].try_into().unwrap()),
                selected: i32::from_le_bytes(b[8..12].try_into().unwrap()),
                exclude_from_terrain: i32::from_le_bytes(b[12..16].try_into().unwrap()),
                position: Vec3::new(
                    u32::from_le_bytes(b[16..20].try_into().unwrap()) as f32 / 1024.,
                    u32::from_le_bytes(b[20..24].try_into().unwrap()) as f32 / 1024.,
                    u32::from_le_bytes(b[24..28].try_into().unwrap()) as f32 / 1024.,
                ),
                rotation: Vec3::new(
                    u32::from_le_bytes(b[28..32].try_into().unwrap()) as f32 / 4096.,
                    u32::from_le_bytes(b[32..36].try_into().unwrap()) as f32 / 4096.,
                    u32::from_le_bytes(b[36..40].try_into().unwrap()) as f32 / 4096.,
                ),
                aabb_min: Vec3::new(
                    i32::from_le_bytes(b[40..44].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[44..48].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[48..52].try_into().unwrap()) as f32 / 1024.,
                ),
                aabb_max: Vec3::new(
                    i32::from_le_bytes(b[52..56].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[56..60].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[60..64].try_into().unwrap()) as f32 / 1024.,
                ),
                furniture_model_slot: u32::from_le_bytes(b[64..68].try_into().unwrap()),
                model_id: i32::from_le_bytes(b[68..72].try_into().unwrap()),
                attackable: i32::from_le_bytes(b[72..76].try_into().unwrap()),
                toughness: i32::from_le_bytes(b[76..80].try_into().unwrap()),
                wounds: i32::from_le_bytes(b[80..84].try_into().unwrap()),
                unknown1: i32::from_le_bytes(b[84..88].try_into().unwrap()),
                owner_unit_index: i32::from_le_bytes(b[88..92].try_into().unwrap()),
                burnable: i32::from_le_bytes(b[92..96].try_into().unwrap()),
                sfx_code: u32::from_le_bytes(b[96..100].try_into().unwrap()),
                gfx_code: u32::from_le_bytes(b[100..104].try_into().unwrap()),
                locked: i32::from_le_bytes(b[104..108].try_into().unwrap()),
                exclude_from_terrain_shadow: i32::from_le_bytes(b[108..112].try_into().unwrap()),
                exclude_from_walk: i32::from_le_bytes(b[112..116].try_into().unwrap()),
                magic_item_code: u32::from_le_bytes(b[116..120].try_into().unwrap()),
                particle_effect_code: u32::from_le_bytes(b[120..124].try_into().unwrap()),
                furniture_dead_model_slot: u32::from_le_bytes(b[124..128].try_into().unwrap()),
                dead_model_id: i32::from_le_bytes(b[128..132].try_into().unwrap()),
                light: i32::from_le_bytes(b[132..136].try_into().unwrap()),
                light_radius: i32::from_le_bytes(b[136..140].try_into().unwrap()),
                light_ambient: i32::from_le_bytes(b[140..144].try_into().unwrap()),
                unknown2: i32::from_le_bytes(b[144..148].try_into().unwrap()),
                unknown3: i32::from_le_bytes(b[148..152].try_into().unwrap()),
            };

            instances.push(instance);
        }

        Ok(instances)
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
                    flags: TrackControlPointFlags::from_bits(flags as u8)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        path::{Path, PathBuf},
    };

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

        let file = File::open(d.clone()).unwrap();
        let project = Decoder::new(file).decode().unwrap();

        assert_eq!(project.water_model_file_name, None); // doesn't have a water model
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

        let file = File::open(d.clone()).unwrap();
        let _project = Decoder::new(file).decode().unwrap();
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
                if ext.to_string_lossy().to_uppercase() == "PRJ" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let file = File::open(path).unwrap();
                    let project = Decoder::new(file).decode().unwrap();

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
                                project.terrain.get_heightmap1_image()
                            } else {
                                project.terrain.get_heightmap2_image()
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
