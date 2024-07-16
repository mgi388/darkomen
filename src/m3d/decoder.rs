use super::*;
use glam::Vec3;
use std::{
    ffi::CStr,
    fmt,
    io::{Error as IoError, Read, Seek},
};

/// The format ID used in all .M3D files. The last part probably stands for "3D
/// model".
const FORMAT: &str = "PD3M";

const HEADER_SIZE_BYTES: usize = 24;
const TEXTURE_DESCRIPTOR_SIZE_BYTES: usize = 96;
const VECTOR_SIZE_BYTES: usize = 12;
const OBJECT_HEADER_SIZE_BYTES: usize = 52 + VECTOR_SIZE_BYTES;
const OBJECT_FACE_SIZE_BYTES: usize = 16 + VECTOR_SIZE_BYTES;
const OBJECT_VERTEX_SIZE_BYTES: usize = (2 * VECTOR_SIZE_BYTES) + 20;

struct Header {
    _magic: u32,
    _version: u32,
    _crc: u32,
    _not_crc: u32,
    texture_count: u16,
    object_count: u16,
}

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
    InvalidString,
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
            DecodeError::InvalidFormat(s) => write!(f, "invalid format: {}", s),
            DecodeError::InvalidString => write!(f, "invalid string"),
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

    pub fn decode(&mut self) -> Result<M3d, DecodeError> {
        let header = self.decode_header()?;

        let texture_descriptors = self.read_texture_descriptors(header.texture_count)?;

        let objects = self.read_objects(header.object_count)?;

        Ok(M3d {
            texture_descriptors,
            objects,
        })
    }

    fn decode_header(&mut self) -> Result<Header, DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        if &buf[0..4] != FORMAT.as_bytes() {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        Ok(Header {
            _magic: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            _version: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            _crc: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            _not_crc: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            texture_count: u16::from_le_bytes(buf[20..22].try_into().unwrap()),
            object_count: u16::from_le_bytes(buf[22..24].try_into().unwrap()),
        })
    }

    fn read_texture_descriptors(
        &mut self,
        count: u16,
    ) -> Result<Vec<TextureDescriptor>, DecodeError> {
        let mut descriptors = Vec::with_capacity(count as usize);

        for _ in 0..count {
            descriptors.push(self.read_texture_descriptor()?);
        }

        Ok(descriptors)
    }

    fn read_texture_descriptor(&mut self) -> Result<TextureDescriptor, DecodeError> {
        let mut buf = [0; TEXTURE_DESCRIPTOR_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let path = self.read_string(&buf[0..64])?;
        let file_name = self.read_string(&buf[64..])?;

        Ok(TextureDescriptor { path, file_name })
    }

    fn read_objects(&mut self, count: u16) -> Result<Vec<Object>, DecodeError> {
        let mut objects = Vec::with_capacity(count as usize);

        for _ in 0..count {
            objects.push(self.read_object()?);
        }

        Ok(objects)
    }

    fn read_object(&mut self) -> Result<Object, DecodeError> {
        let mut buf = [0; OBJECT_HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let vertex_count = u16::from_le_bytes(buf[48..50].try_into().unwrap());
        let face_count = u16::from_le_bytes(buf[50..52].try_into().unwrap());

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(self.read_face()?);
        }

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push(self.read_vertex()?);
        }

        Ok(Object {
            name: self.read_string(&buf[0..32])?,
            parent_index: i16::from_le_bytes(buf[32..34].try_into().unwrap()),
            padding: i16::from_le_bytes(buf[34..36].try_into().unwrap()),
            translation: self.read_vector(&buf[36..48])?,
            flags: ObjectFlags::from_bits(u32::from_le_bytes(buf[52..56].try_into().unwrap()))
                .expect("object flags should be valid"),
            unknown1: u32::from_le_bytes(buf[56..60].try_into().unwrap()),
            unknown2: u32::from_le_bytes(buf[60..64].try_into().unwrap()),
            faces,
            vertices,
        })
    }

    fn read_face(&mut self) -> Result<Face, DecodeError> {
        let mut buf = [0; OBJECT_FACE_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        Ok(Face {
            indices: [
                u16::from_le_bytes(buf[0..2].try_into().unwrap()),
                u16::from_le_bytes(buf[2..4].try_into().unwrap()),
                u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            ],
            texture_index: u16::from_le_bytes(buf[6..8].try_into().unwrap()),
            normal: self.read_vector(&buf[8..20])?,
            unknown1: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            unknown2: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
        })
    }

    fn read_vertex(&mut self) -> Result<Vertex, DecodeError> {
        let mut buf = [0; OBJECT_VERTEX_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        Ok(Vertex {
            position: self.read_vector(&buf[0..12])?,
            normal: self.read_vector(&buf[12..24])?,
            color: UVec4::new(
                buf[24] as u32,
                buf[25] as u32,
                buf[26] as u32,
                buf[27] as u32,
            ),
            uv: Vec2::new(
                f32::from_le_bytes(buf[28..32].try_into().unwrap()),
                f32::from_le_bytes(buf[32..36].try_into().unwrap()),
            ),
            index: u32::from_le_bytes(buf[36..40].try_into().unwrap()),
            unknown1: u32::from_le_bytes(buf[40..44].try_into().unwrap()),
        })
    }

    fn read_vector(&mut self, buf: &[u8]) -> Result<Vec3, DecodeError> {
        let x = f32::from_le_bytes(buf[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(buf[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(buf[8..12].try_into().unwrap());

        Ok(Vec3::new(x, y, z))
    }

    fn read_string(&mut self, buf: &[u8]) -> Result<String, DecodeError> {
        Ok(CStr::from_bytes_until_nul(buf)
            .map_err(|_| DecodeError::InvalidString)?
            .to_string_lossy()
            .into_owned())
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
    fn test_decode_b1_01_base() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "BASE.M3D",
        ]
        .iter()
        .collect();

        let file = File::open(d.clone()).unwrap();
        let m3d = Decoder::new(file).decode().unwrap();

        assert_eq!(m3d.texture_descriptors.len(), 37);
        assert_eq!(m3d.objects.len(), 4);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [std::env::var("DARKOMEN_PATH").unwrap().as_str(), "DARKOMEN"]
            .iter()
            .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "m3ds"]
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
                if ext.to_string_lossy().to_uppercase() == "M3D"
                    || ext.to_string_lossy().to_uppercase() == "M3X"
                {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let file = File::open(path).unwrap();
                    let m3d = Decoder::new(file).decode().unwrap();

                    let parent_dir = path
                        .components()
                        .collect::<Vec<_>>()
                        .iter()
                        .rev()
                        .skip(1) // skip the file name
                        .take_while(|c| c.as_os_str() != "DARKOMEN")
                        .collect::<Vec<_>>()
                        .iter()
                        .rev()
                        .collect::<PathBuf>();

                    let output_dir = root_output_dir.join(parent_dir);
                    std::fs::create_dir_all(&output_dir).unwrap();

                    let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
                    let mut output_file = File::create(output_path).unwrap();
                    ron::ser::to_writer_pretty(&mut output_file, &m3d, Default::default()).unwrap();
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
