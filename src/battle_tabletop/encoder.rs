use std::{
    ffi::CString,
    fmt,
    io::{BufWriter, Write},
};

use encoding_rs::WINDOWS_1252;

use super::*;

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
    StringTooLong,
}

impl std::error::Error for EncodeError {}

impl From<std::io::Error> for EncodeError {
    fn from(err: std::io::Error) -> Self {
        EncodeError::IoError(err)
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::IoError(e) => write!(f, "IO error: {e}"),
            EncodeError::InvalidString => write!(f, "invalid string"),
            EncodeError::StringTooLong => write!(f, "string too long"),
        }
    }
}

trait Int: Copy + Sized {
    fn to_le_bytes(self) -> [u8; 4];
}

impl Int for i32 {
    fn to_le_bytes(self) -> [u8; 4] {
        i32::to_le_bytes(self)
    }
}

impl Int for u32 {
    fn to_le_bytes(self) -> [u8; 4] {
        u32::to_le_bytes(self)
    }
}

const MAX_STRING_SIZE_BYTES: usize = 32;

pub struct Encoder<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder {
            writer: BufWriter::new(writer),
        }
    }

    pub fn encode(&mut self, battle_tabletop: &BattleTabletop) -> Result<(), EncodeError> {
        self.write_btb_file_type()?;
        self.write_battle_header(battle_tabletop)?;
        self.write_objectives(&battle_tabletop.objectives)?;
        self.write_obstacles(battle_tabletop, &battle_tabletop.obstacles)?;
        self.write_regions(&battle_tabletop.regions)?;
        self.write_nodes(&battle_tabletop.nodes)?;
        self.write_btb_file_type()?;
        Ok(())
    }

    fn write_btb_file_type(&mut self) -> Result<(), EncodeError> {
        self.write_object_header(0xbeafeed0, 0)?;
        Ok(())
    }

    fn write_battle_header(&mut self, bt: &BattleTabletop) -> Result<(), EncodeError> {
        let size = 12 + // width
            12 + // height
            (8 + MAX_STRING_SIZE_BYTES) * 5 + // 5 strings
            16; // int tuple of 2
        self.write_object_header(1, size)?;

        self.write_int_tuple_property(1, &[bt.width as i32])?;
        self.write_int_tuple_property(2, &[bt.height as i32])?;
        self.write_string_property(1001, &bt.army1_file_stem, None)?;
        self.write_string_property(1002, &bt.army2_file_stem, None)?;
        self.write_string_property(1003, &bt.ctl_file_stem, None)?;
        self.write_string_property(1004, &bt.unknown1, None)?;
        self.write_string_property(1005, &bt.unknown2, None)?;
        self.write_int_tuple_property(9, &bt.unknown3)?;

        Ok(())
    }

    fn write_objectives(&mut self, objectives: &[Objective]) -> Result<(), EncodeError> {
        let size = objectives.len() * 20; // objectives
        self.write_object_header(2, size)?;

        for objective in objectives {
            self.write_int_tuple_property(3, &[objective.id, objective.value1, objective.value2])?;
        }
        Ok(())
    }

    fn write_obstacles(
        &mut self,
        bt: &BattleTabletop,
        obstacles: &[Obstacle],
    ) -> Result<(), EncodeError> {
        let size = 12 + obstacles.len() * 80; // unknown + obstacles
        self.write_object_header(3, size)?;

        self.write_int_tuple_property(8, &[bt.obstacles_unknown1])?;

        for obstacle in obstacles {
            self.write_property_header(501, 72)?;
            self.write_int_tuple_property(5, &[obstacle.flags.bits()])?;
            self.write_int_tuple_property(1, &[obstacle.position.x])?;
            self.write_int_tuple_property(2, &[obstacle.position.y])?;
            self.write_int_tuple_property(4, &[obstacle.height])?;
            self.write_int_tuple_property(6, &[obstacle.radius as i32])?;
            self.write_int_tuple_property(7, &[obstacle.unknown])?;
        }
        Ok(())
    }

    fn write_regions(&mut self, regions: &[Region]) -> Result<(), EncodeError> {
        for region in regions {
            let size = 8 + MAX_STRING_SIZE_BYTES + // name
                12 + // flags
                16 + // pos tuple
                region.line_segments.len() * 24; // line segments
            self.write_object_header(4, size)?;

            self.write_string_property(
                1006,
                &region.display_name,
                region.display_name_residual_bytes.as_ref(),
            )?;
            self.write_int_tuple_property(5, &[region.flags.bits()])?;
            self.write_int_tuple_property(10, &[region.position.x, region.position.y])?;

            for line in &region.line_segments {
                self.write_int_tuple_property(
                    502,
                    &[line.start.x, line.start.y, line.end.x, line.end.y],
                )?;
            }
        }
        Ok(())
    }

    fn write_nodes(&mut self, nodes: &[Node]) -> Result<(), EncodeError> {
        let size = 12 + nodes.len() * 104; // count + nodes
        self.write_object_header(5, size)?;

        self.write_int_tuple_property(8, &[nodes.len() as i32])?;

        for node in nodes {
            self.write_property_header(503, 96)?;
            self.write_int_tuple_property(5, &[node.flags.bits()])?;
            self.write_int_tuple_property(1, &[node.position.x])?;
            self.write_int_tuple_property(2, &[node.position.y])?;
            self.write_int_tuple_property(6, &[node.radius as i32])?;
            self.write_int_tuple_property(7, &[node.rotation])?;
            self.write_int_tuple_property(11, &[node.node_id as i32])?;
            self.write_int_tuple_property(12, &[node.regiment_id as i32])?;
            self.write_int_tuple_property(13, &[node.script_id as i32])?;
        }
        Ok(())
    }

    fn write_object_header(&mut self, id: u32, size: usize) -> Result<(), EncodeError> {
        self.writer.write_all(&id.to_le_bytes())?;
        self.writer.write_all(&(size as u32).to_le_bytes())?;
        Ok(())
    }

    fn write_property_header(&mut self, id: u32, size: usize) -> Result<(), EncodeError> {
        self.writer.write_all(&id.to_le_bytes())?;
        self.writer.write_all(&((size + 8) as u32).to_le_bytes())?;
        Ok(())
    }

    fn write_int_tuple_property<T: Int>(
        &mut self,
        id: u32,
        values: &[T],
    ) -> Result<(), EncodeError> {
        let size = values.len() * 4;
        self.write_property_header(id, size)?;
        for &value in values {
            self.writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    fn write_string_property(
        &mut self,
        id: u32,
        s: &str,
        residual_bytes: Option<&Vec<u8>>,
    ) -> Result<(), EncodeError> {
        self.write_property_header(id, MAX_STRING_SIZE_BYTES)?;

        let c_string = CString::new(s).map_err(|_| EncodeError::InvalidString)?;
        let bytes = c_string.as_bytes_with_nul();

        if bytes.len() > MAX_STRING_SIZE_BYTES {
            return Err(EncodeError::StringTooLong);
        }

        self.write_padded_string(s, residual_bytes, MAX_STRING_SIZE_BYTES)?;

        Ok(())
    }

    fn write_padded_string(
        &mut self,
        s: &str,
        residual_bytes: Option<&Vec<u8>>,
        total_size: usize,
    ) -> Result<(), EncodeError> {
        let bytes_written = self.write_string(s)?;

        if let Some(residual) = residual_bytes {
            let padding_size = total_size - (bytes_written + residual.len());
            let padding = vec![0; padding_size];
            self.writer.write_all(residual)?;
            self.writer.write_all(&padding)?;
        } else {
            let padding_size = total_size - bytes_written;
            let padding = vec![0; padding_size];
            self.writer.write_all(&padding)?;
        }

        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<usize, EncodeError> {
        let (windows_1252_bytes, _, _) = WINDOWS_1252.encode(s);

        let c_string = CString::new(windows_1252_bytes).map_err(|_| EncodeError::InvalidString)?;
        let bytes = c_string.as_bytes_with_nul();

        self.writer.write_all(bytes)?;

        Ok(bytes.len())
    }
}
