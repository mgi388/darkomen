use std::{
    fmt,
    io::{Error as IoError, Read, Seek},
    mem::size_of,
};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use super::*;

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

pub(crate) const MAX_STRING_SIZE_BYTES: usize = 32;

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

    pub fn decode(&mut self) -> Result<BattleTabletop, DecodeError> {
        self.read_btb_file_type()?;
        let (width, height, player_army, enemy_army, ctl, unknown1, unknown2, unknown3) =
            self.read_battle_header()?;
        let objectives = self.read_objectives()?;
        let (obstacles, obstacles_unknown1) = self.read_obstacles()?;
        let regions = self.read_regions()?;
        let nodes = self.read_nodes()?;
        self.read_btb_file_type()?;

        Ok(BattleTabletop {
            width,
            height,
            player_army,
            enemy_army,
            ctl,
            unknown1,
            unknown2,
            unknown3,
            objectives,
            obstacles,
            obstacles_unknown1,
            regions,
            nodes,
        })
    }

    fn read_btb_file_type(&mut self) -> Result<(), DecodeError> {
        let _ = self.read_object_header(0xbeafeed0)?;
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn read_battle_header(
        &mut self,
    ) -> Result<(u32, u32, String, String, String, String, String, Vec<i32>), DecodeError> {
        let _ = self.read_object_header(1)?;

        let width = self.read_int_tuple_property::<i32>(1, 1)?[0] as u32;
        let height = self.read_int_tuple_property::<i32>(2, 1)?[0] as u32;
        let (player_army, _) = self.read_string_property(1001)?;
        let (enemy_army, _) = self.read_string_property(1002)?;
        let (ctl, _) = self.read_string_property(1003)?;
        let (unknown1, _) = self.read_string_property(1004)?;
        let (unknown2, _) = self.read_string_property(1005)?;
        let unknown3 = self.read_int_tuple_property::<i32>(9, 2)?;

        Ok((
            width,
            height,
            player_army,
            enemy_army,
            ctl,
            unknown1,
            unknown2,
            unknown3,
        ))
    }

    fn read_objectives(&mut self) -> Result<Vec<Objective>, DecodeError> {
        let size = self.read_object_header(2)?;

        let mut objectives = Vec::new();

        let mut i = 0;
        while i < size {
            let tuple = self.read_int_tuple_property::<i32>(3, 3)?;

            objectives.push(Objective {
                id: tuple[0],
                value1: tuple[1],
                value2: tuple[2],
            });

            i += 20;
        }

        Ok(objectives)
    }

    fn read_obstacles(&mut self) -> Result<(Vec<Obstacle>, i32), DecodeError> {
        let size = self.read_object_header(3)?;

        let unknown1 = self.read_int_tuple_property::<i32>(8, 1)?[0];

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
                position: IVec2::new(x, y),
                z,
                radius: radius as u32,
                dir,
            });
        }

        Ok((obstacles, unknown1))
    }

    fn read_regions(&mut self) -> Result<Vec<Region>, DecodeError> {
        let mut regions = Vec::new();

        while self.peek_u32()? == 4 {
            let _ = self.read_object_header(4)?;
            let (display_name, display_name_residual_bytes) = self.read_string_property(1006)?;
            let flags = self.read_int_tuple_property::<u32>(5, 1)?[0];
            let position = self.read_int_tuple_property::<i32>(10, 2)?;

            let mut line_segments = Vec::new();

            while self.peek_u32()? == 502 {
                let line = self.read_int_tuple_property::<i32>(502, 4)?;

                line_segments.push(LineSegment {
                    start: IVec2::new(line[0], line[1]),
                    end: IVec2::new(line[2], line[3]),
                });
            }

            regions.push(Region {
                display_name,
                display_name_residual_bytes,
                flags: RegionFlags::from_bits(flags).expect("region flags should be valid"),
                position: IVec2::new(position[0], position[1]),
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
            let rotation = self.read_int_tuple_property::<i32>(7, 1)?[0];
            let node_id = self.read_int_tuple_property::<i32>(11, 1)?[0] as u32;
            let regiment_id = self.read_int_tuple_property::<i32>(12, 1)?[0] as u32;
            let script_id = self.read_int_tuple_property::<i32>(13, 1)?[0] as u32;

            nodes.push(Node {
                flags: NodeFlags::from_bits(flags).expect("node flags should be valid"),
                position: IVec2::new(x, y),
                radius,
                rotation,
                node_id,
                regiment_id,
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

    fn read_string_property(
        &mut self,
        expected_id: u32,
    ) -> Result<(String, Option<Vec<u8>>), DecodeError> {
        self.read_property_header(expected_id, MAX_STRING_SIZE_BYTES)?;

        let mut buf = vec![0; MAX_STRING_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let str_buf = &buf[0..MAX_STRING_SIZE_BYTES];
        let (str_buf, str_residual_bytes) = str_buf
            .iter()
            .enumerate()
            .find(|(_, &b)| b == 0)
            .map(|(i, _)| str_buf.split_at(i + 1))
            .unwrap_or((str_buf, &[]));

        let str = self.read_string(str_buf)?;
        let str_residual_bytes = if str_residual_bytes.iter().all(|&b| b == 0) {
            None
        } else {
            Some(
                str_residual_bytes
                    .iter()
                    .rposition(|&b| b != 0) // find the last non-zero byte
                    .map(|pos| &str_residual_bytes[..=pos]) // include the last non-zero byte
                    .unwrap_or(str_residual_bytes)
                    .to_vec(),
            )
        };

        Ok((str, str_residual_bytes))
    }

    fn peek_u32(&mut self) -> Result<u32, DecodeError> {
        let mut buf = [0; size_of::<u32>()];
        self.reader.read_exact(&mut buf)?;

        let value = u32::from_le_bytes(buf);

        self.reader
            .seek(std::io::SeekFrom::Current(-(size_of::<u32>() as i64)))?;

        Ok(value)
    }

    fn read_string(&mut self, buf: &[u8]) -> Result<String, DecodeError> {
        let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        let mut decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(&buf[..nul_pos]);
        let mut dest = String::new();

        decoder.read_to_string(&mut dest)?;

        Ok(dest)
    }
}
