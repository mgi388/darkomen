use std::{
    ffi::CString,
    io::{BufWriter, Write},
};

use encoding_rs::WINDOWS_1252;

use crate::m3d::decoder::FORMAT;

use super::*;

#[derive(Debug)]
pub enum EncodeError {
    IoError(std::io::Error),
    InvalidString,
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
            EncodeError::IoError(e) => write!(f, "IO error: {}", e),
            EncodeError::InvalidString => write!(f, "invalid string"),
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

    pub fn encode(&mut self, m3d: &M3d) -> Result<(), EncodeError> {
        self.write_header(m3d)?;
        self.write_texture_descriptors(&m3d.texture_descriptors)?;
        self.write_objects(&m3d.objects)?;
        Ok(())
    }

    fn write_header(&mut self, m3d: &M3d) -> Result<(), EncodeError> {
        self.writer.write_all(FORMAT.as_bytes())?;
        self.writer.write_all(&m3d.header._magic.to_le_bytes())?;
        self.writer.write_all(&m3d.header._version.to_le_bytes())?;
        self.writer.write_all(&m3d.header._crc.to_le_bytes())?;
        self.writer.write_all(&m3d.header._not_crc.to_le_bytes())?;
        self.writer
            .write_all(&(m3d.texture_descriptors.len() as u16).to_le_bytes())?;
        self.writer
            .write_all(&(m3d.objects.len() as u16).to_le_bytes())?;
        Ok(())
    }

    fn write_texture_descriptors(
        &mut self,
        texture_descriptors: &[M3dTextureDescriptor],
    ) -> Result<(), EncodeError> {
        for descriptor in texture_descriptors {
            self.write_texture_descriptor(descriptor)?;
        }
        Ok(())
    }

    fn write_texture_descriptor(
        &mut self,
        descriptor: &M3dTextureDescriptor,
    ) -> Result<(), EncodeError> {
        self.write_string(&descriptor.path)?;
        self.writer.write_all(&descriptor.path_remainder)?;
        self.write_string(&descriptor.file_name)?;
        self.writer.write_all(&descriptor.file_name_remainder)?;
        Ok(())
    }

    fn write_objects(&mut self, objects: &[Object]) -> Result<(), EncodeError> {
        for object in objects {
            self.write_object(object)?;
        }
        Ok(())
    }

    fn write_object(&mut self, object: &Object) -> Result<(), EncodeError> {
        self.write_string(&object.name)?;
        self.writer.write_all(&object.name_remainder)?;
        self.writer.write_all(&object.parent_index.to_le_bytes())?;
        self.writer.write_all(&object.padding.to_le_bytes())?;
        self.write_vector(&object.translation)?;
        self.writer
            .write_all(&(object.vertices.len() as u16).to_le_bytes())?;
        self.writer
            .write_all(&(object.faces.len() as u16).to_le_bytes())?;
        self.writer.write_all(&object.flags.bits().to_le_bytes())?;
        self.writer.write_all(&object.unknown1.to_le_bytes())?;
        self.writer.write_all(&object.unknown2.to_le_bytes())?;

        for face in &object.faces {
            self.write_face(face)?;
        }

        for vertex in &object.vertices {
            self.write_vertex(vertex)?;
        }
        Ok(())
    }

    fn write_face(&mut self, face: &Face) -> Result<(), EncodeError> {
        for &index in &face.indices {
            self.writer.write_all(&index.to_le_bytes())?;
        }
        self.writer.write_all(&face.texture_index.to_le_bytes())?;
        self.write_vector(&face.normal)?;
        self.writer.write_all(&face.unknown1.to_le_bytes())?;
        self.writer.write_all(&face.unknown2.to_le_bytes())?;
        Ok(())
    }

    fn write_vertex(&mut self, vertex: &Vertex) -> Result<(), EncodeError> {
        self.write_vector(&vertex.position)?;
        self.write_vector(&vertex.normal)?;
        self.writer.write_all(&[
            vertex.color.x as u8,
            vertex.color.y as u8,
            vertex.color.z as u8,
            vertex.color.w as u8,
        ])?;
        self.writer.write_all(&vertex.uv.x.to_le_bytes())?;
        self.writer.write_all(&vertex.uv.y.to_le_bytes())?;
        self.writer.write_all(&vertex.index.to_le_bytes())?;
        self.writer.write_all(&vertex.unknown1.to_le_bytes())?;
        Ok(())
    }

    fn write_vector(&mut self, vec: &Vec3) -> Result<(), EncodeError> {
        self.writer.write_all(&vec.x.to_le_bytes())?;
        self.writer.write_all(&vec.y.to_le_bytes())?;
        self.writer.write_all(&vec.z.to_le_bytes())?;
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
