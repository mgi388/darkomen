use super::*;
use std::{
    ffi::CStr,
    fmt,
    io::{Error as IoError, Read, Seek},
    mem::size_of,
};

trait Int: Copy + Sized {
    const SIZE: usize;
    fn from_le_bytes(bytes: &[u8]) -> Self;
}

impl Int for i32 {
    const SIZE: usize = size_of::<Self>();
    fn from_le_bytes(bytes: &[u8]) -> Self {
        i32::from_le_bytes(bytes.try_into().expect("bytes should be converted"))
    }
}

impl Int for u32 {
    const SIZE: usize = size_of::<Self>();
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u32::from_le_bytes(bytes.try_into().expect("bytes should be converted"))
    }
}

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidObjectHeaderId(u32),
    InvalidPropertyHeaderId(u32),
    InvalidPropertySize(u32),
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
            DecodeError::InvalidObjectHeaderId(id) => write!(f, "invalid object header ID: {}", id),
            DecodeError::InvalidPropertyHeaderId(id) => {
                write!(f, "invalid property header ID: {}", id)
            }
            DecodeError::InvalidPropertySize(size) => {
                write!(f, "invalid property size: {}", size)
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

    pub fn decode(&mut self) -> Result<Blueprint, DecodeError> {
        self.check_btb_file_type();

        let (width, height, player, enemy, ctl) = self.read_battle_header()?;
        let objectives = self.read_objectives()?;
        let obstacles = self.read_obstacles()?;
        let regions = self.read_regions()?;
        let nodes = self.read_nodes()?;

        Ok(Blueprint {
            width,
            height,
            player,
            enemy,
            ctl,
            objectives,
            obstacles,
            regions,
            nodes,
        })
    }

    fn check_btb_file_type(&mut self) {
        let _ = self.read_object_header(0xbeafeed0);
    }

    fn read_battle_header(&mut self) -> Result<(u32, u32, String, String, String), DecodeError> {
        let _ = self.read_object_header(1)?;

        let width = self.read_int_tuple_property::<i32>(1, 1)?[0] as u32;
        let height = self.read_int_tuple_property::<i32>(2, 1)?[0] as u32;
        let player = self.read_string_property(1001)?;
        let enemy = self.read_string_property(1002)?;
        let ctl = self.read_string_property(1003)?;
        let _ = self.read_string_property(1004)?;
        let _ = self.read_string_property(1005)?;
        let _ = self.read_int_tuple_property::<i32>(9, 2)?;

        Ok((width, height, player, enemy, ctl))
    }

    fn read_objectives(&mut self) -> Result<Vec<Objective>, DecodeError> {
        let size = self.read_object_header(2)?;

        let mut objectives = Vec::new();

        let mut i = 0;
        while i < size {
            let tuple = self.read_int_tuple_property::<i32>(3, 3)?;

            objectives.push(Objective {
                typ: tuple[0],
                val1: tuple[1],
                val2: tuple[2],
            });

            i += 20;
        }

        Ok(objectives)
    }

    fn read_obstacles(&mut self) -> Result<Vec<Obstacle>, DecodeError> {
        let size = self.read_object_header(3)?;

        let _unknown = self.read_int_tuple_property::<i32>(8, 1)?[0];

        let obstactle_count = (size - 12) / 80;

        let mut obstacles = Vec::with_capacity(obstactle_count);

        for _ in 0..obstactle_count {
            let _ = self.read_property_header(501, 72);

            let flags = self.read_int_tuple_property::<u32>(5, 1)?[0];
            let x = self.read_int_tuple_property::<i32>(1, 1)?[0];
            let y = self.read_int_tuple_property::<i32>(2, 1)?[0];
            let z = self.read_int_tuple_property::<i32>(4, 1)?[0];
            let radius = self.read_int_tuple_property::<i32>(6, 1)?[0];
            let dir = self.read_int_tuple_property::<i32>(7, 1)?[0];

            obstacles.push(Obstacle {
                flags: ObstacleFlags::from_bits(flags).expect("obstacle flags should be valid"),
                x,
                y,
                z,
                radius: radius as u32,
                dir,
            });
        }

        Ok(obstacles)
    }

    fn read_regions(&mut self) -> Result<Vec<Region>, DecodeError> {
        let mut regions = Vec::new();

        while self.peek_u32()? == 4 {
            let _ = self.read_object_header(4)?;
            let name = self.read_string_property(1006)?;
            let flags = self.read_int_tuple_property::<u32>(5, 1)?[0];
            let _pos = self.read_int_tuple_property::<i32>(10, 2)?;

            let mut line_segments = Vec::new();

            while self.peek_u32()? == 502 {
                let line = self.read_int_tuple_property::<i32>(502, 4)?;

                line_segments.push(LineSegment {
                    start_x: line[0],
                    start_y: line[1],
                    end_x: line[2],
                    end_y: line[3],
                });
            }

            regions.push(Region {
                name,
                flags: RegionFlags::from_bits(flags).expect("region flags should be valid"),
                line_segments,
            });
        }

        Ok(regions)
    }

    fn read_nodes(&mut self) -> Result<Vec<Node>, DecodeError> {
        let _ = self.read_object_header(5)?;

        let node_count = self.read_int_tuple_property::<i32>(8, 1)?[0] as usize;

        let mut nodes = Vec::with_capacity(node_count);

        for _ in 0..node_count {
            let _ = self.read_property_header(503, 96);

            let flags = self.read_int_tuple_property::<u32>(5, 1)?[0];
            let x = self.read_int_tuple_property::<i32>(1, 1)?[0];
            let y = self.read_int_tuple_property::<i32>(2, 1)?[0];
            let radius = self.read_int_tuple_property::<i32>(6, 1)?[0] as u32;
            let direction = self.read_int_tuple_property::<i32>(7, 1)?[0];
            let node_id = self.read_int_tuple_property::<i32>(11, 1)?[0] as u32;
            let uuid = self.read_int_tuple_property::<i32>(12, 1)?[0] as u32;
            let script_id = self.read_int_tuple_property::<i32>(13, 1)?[0] as u32;

            nodes.push(Node {
                flags: NodeFlags::from_bits(flags).expect("node flags should be valid"),
                x,
                y,
                radius,
                direction,
                node_id,
                uuid,
                script_id,
            });
        }

        Ok(nodes)
    }

    fn read_object_header(&mut self, expected_id: u32) -> Result<usize, DecodeError> {
        let mut buf = [0; size_of::<u32>() * 2];
        self.reader.read_exact(&mut buf)?;

        let id = u32::from_le_bytes(buf[0..size_of::<u32>()].try_into().unwrap());
        if id != expected_id {
            return Err(DecodeError::InvalidObjectHeaderId(id));
        }

        let size = u32::from_le_bytes(buf[size_of::<u32>()..].try_into().unwrap()) as usize;

        Ok(size)
    }

    fn read_int_tuple_property<T: Int>(
        &mut self,
        expected_id: u32,
        arity: usize,
    ) -> Result<Vec<T>, DecodeError> {
        self.read_property_header(expected_id, T::SIZE * arity)?;

        let mut buf = vec![0; T::SIZE * arity];
        self.reader.read_exact(&mut buf)?;

        let mut result = Vec::new();

        for i in 0..arity {
            result.push(T::from_le_bytes(
                buf[i * T::SIZE..(i + 1) * T::SIZE].try_into().unwrap(),
            ));
        }

        Ok(result)
    }

    fn read_property_header(
        &mut self,
        expected_id: u32,
        expected_size: usize,
    ) -> Result<(), DecodeError> {
        let mut buf = [0; size_of::<u32>() * 2];
        self.reader.read_exact(&mut buf)?;

        let id = u32::from_le_bytes(buf[0..size_of::<u32>()].try_into().unwrap());
        if id != expected_id {
            return Err(DecodeError::InvalidPropertyHeaderId(id));
        }

        let size = u32::from_le_bytes(buf[size_of::<u32>()..].try_into().unwrap());
        // The size value includes the ID and size fields so subtract it.
        let actual_size = size - (size_of::<u32>() as u32 * 2);
        if actual_size != expected_size as u32 {
            return Err(DecodeError::InvalidPropertySize(actual_size));
        }

        Ok(())
    }

    fn read_string_property(&mut self, expected_id: u32) -> Result<String, DecodeError> {
        const MAX_STRING_SIZE: usize = 32;
        self.read_property_header(expected_id, MAX_STRING_SIZE)?;

        let mut buf = vec![0; MAX_STRING_SIZE];
        self.reader.read_exact(&mut buf)?;

        Ok(
            String::from_utf8_lossy(CStr::from_bytes_until_nul(&buf).unwrap().to_bytes())
                .to_string(),
        )
    }

    fn peek_u32(&mut self) -> Result<u32, DecodeError> {
        let mut buf = [0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;

        let value = u32::from_le_bytes(buf);

        self.reader
            .seek(std::io::SeekFrom::Current(-(size_of::<u32>() as i64)))?;

        Ok(value)
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
            "B1_01.BTB",
        ]
        .iter()
        .collect();

        let file = File::open(d.clone()).unwrap();
        let b = Decoder::new(file).decode().unwrap();

        assert_eq!(b.width, 1440);
        assert_eq!(b.height, 1600);
        assert_eq!(b.player, "B101mrc");
        assert_eq!(b.enemy, "B101nme");
        assert_eq!(b.ctl, "B101");
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

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "btbs"]
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
                if ext.to_string_lossy().to_uppercase() == "BTB" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let file = File::open(path).unwrap();
                    let b = Decoder::new(file).decode().unwrap();

                    for o in &b.obstacles {
                        // Should either block movement or projectiles.
                        assert!(
                            o.flags.contains(ObstacleFlags::BLOCKS_MOVEMENT)
                                || o.flags.contains(ObstacleFlags::BLOCKS_PROJECTILES)
                        );
                        // Should not be any disabled obstacles.
                        assert!(o.flags.contains(ObstacleFlags::IS_ENABLED));
                    }

                    let output_path =
                        append_ext("ron", root_output_dir.join(path.file_name().unwrap()));
                    let mut output_file = File::create(output_path).unwrap();
                    ron::ser::to_writer_pretty(&mut output_file, &b, Default::default()).unwrap();
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
