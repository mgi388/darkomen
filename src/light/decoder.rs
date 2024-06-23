use super::*;
use glam::Vec3;
use std::{
    fmt,
    io::{Error as IoError, Read, Seek},
};

const FORMAT: u32 = 1;

const HEADER_SIZE: usize = 8;
const LIGHT_SIZE: usize = 32;

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
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

    pub fn decode(&mut self) -> Result<Vec<Light>, DecodeError> {
        let light_count = self.decode_header()?;

        let lights = self.read_lights(light_count)?;

        Ok(lights)
    }

    fn decode_header(&mut self) -> Result<usize, DecodeError> {
        let mut buf = [0; HEADER_SIZE];
        self.reader.read_exact(&mut buf)?;

        if u32::from_le_bytes(buf[0..4].try_into().unwrap()) != FORMAT {
            return Err(DecodeError::InvalidFormat(
                String::from_utf8_lossy(&buf[0..4]).to_string(),
            ));
        }

        let light_count = u32::from_le_bytes(buf[4..8].try_into().unwrap());

        Ok(light_count as usize)
    }

    fn read_lights(&mut self, light_count: usize) -> Result<Vec<Light>, DecodeError> {
        let mut buf = vec![0; light_count * LIGHT_SIZE];
        self.reader.read_exact(&mut buf)?;

        let mut lights = Vec::with_capacity(light_count);
        for i in 0..light_count {
            let b = &buf[i * LIGHT_SIZE..(i + 1) * LIGHT_SIZE];

            let instance = Light {
                position: Vec3::new(
                    i32::from_le_bytes(b[0..4].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[4..8].try_into().unwrap()) as f32 / 1024.,
                    i32::from_le_bytes(b[8..12].try_into().unwrap()) as f32 / 1024.,
                ),
                flags: u32::from_le_bytes(b[12..16].try_into().unwrap()),
                unknown: u32::from_le_bytes(b[16..20].try_into().unwrap()),
                color: Vec3::new(
                    u32::from_le_bytes(b[20..24].try_into().unwrap()) as f32 / 255.,
                    u32::from_le_bytes(b[24..28].try_into().unwrap()) as f32 / 255.,
                    u32::from_le_bytes(b[28..32].try_into().unwrap()) as f32 / 255.,
                ),
            };

            lights.push(instance);
        }

        Ok(lights)
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
            "B1_01.LIT",
        ]
        .iter()
        .collect();

        let file = File::open(d.clone()).unwrap();
        let lights = Decoder::new(file).decode().unwrap();

        assert_eq!(lights.len(), 3);
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

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "lights"]
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
                if ext.to_string_lossy().to_uppercase() == "LIT" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let file = File::open(path).unwrap();
                    let lights = Decoder::new(file).decode().unwrap();

                    let output_path =
                        append_ext("ron", root_output_dir.join(path.file_name().unwrap()));
                    let mut output_file = File::create(output_path).unwrap();
                    ron::ser::to_writer_pretty(&mut output_file, &lights, Default::default())
                        .unwrap();
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
