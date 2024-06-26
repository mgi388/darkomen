use super::*;
use image::{DynamicImage, GenericImage, Pixel, Rgb, Rgba};
use std::{
    fmt,
    io::{Error as IoError, Read, Seek, SeekFrom},
};

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
            DecodeError::IoError(error) => write!(f, "IO error: {}", error),
            DecodeError::InvalidFormat(format) => write!(f, "invalid format: {}", format),
        }
    }
}

/// The sprite format ID used in all .SPR files.
/// "WHDO" is an initialism for "Warhammer: Dark Omen".
const FORMAT: &str = "WHDO";

const HEADER_SIZE: usize = 32;
const FRAME_HEADER_SIZE: usize = 32;

#[derive(Clone, Debug)]
struct Header {
    _file_size: u16,
    _frame_header_offset: u16,
    frame_data_offset: u16,
    _color_table_offset: u16,
    color_table_entries: u16,
    _palette_count: u16,
    frame_count: u16,
}

#[derive(Clone, Debug)]
#[repr(u8)]
enum CompressionType {
    None,
    Packbits,
    ZeroRuns,
}

impl From<u8> for CompressionType {
    fn from(value: u8) -> Self {
        match value {
            0 => CompressionType::None,
            1 => CompressionType::Packbits,
            2 => CompressionType::ZeroRuns,
            _ => panic!("invalid compression type"),
        }
    }
}

#[derive(Clone, Debug)]
struct FrameHeader {
    frame_type: FrameType,
    compression_type: CompressionType,
    _color_count: u16,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    data_offset: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    color_table_offset: u32,
    _padding: u32, // last 4 bytes are not used
}

const DEFAULT_ASPECT_RATIO: f32 = 16. / 9.;

pub struct Decoder<R>
where
    R: Read + Seek,
{
    reader: R,
    columns: Option<usize>,
    aspect_ratio: Option<Box<dyn Fn(u32) -> f32>>,
}

impl<R: Read + Seek> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Decoder {
            reader,
            columns: None,
            aspect_ratio: None,
        }
    }

    pub fn with_columns(mut self, columns: usize) -> Self {
        self.columns = Some(columns);
        self
    }

    pub fn with_aspect_ratio<F>(mut self, f: F) -> Self
    where
        F: Fn(u32) -> f32 + 'static,
    {
        self.aspect_ratio = Some(Box::new(f));
        self
    }

    pub fn decode(&mut self) -> Result<SpriteSheet, DecodeError> {
        let header = self.decode_header()?;

        let (mut frame_headers, frame_max_width, frame_max_height) =
            self.decode_frame_headers(header.clone())?;

        let color_table = self.decode_color_table(header.clone())?;

        let (columns, rows) = if frame_headers.is_empty() {
            (0, 0)
        } else if let Some(columns) = self.columns {
            let rows = (frame_headers.len() + columns - 1) / columns;
            (columns, rows)
        } else {
            let aspect_ratio = if let Some(f) = &self.aspect_ratio {
                f(header.frame_count as u32)
            } else {
                DEFAULT_ASPECT_RATIO
            };
            let columns = (frame_headers.len() as f32 * aspect_ratio).sqrt().ceil() as usize;
            let rows = (frame_headers.len() + columns - 1) / columns;
            (columns, rows)
        };

        let width = (columns * frame_max_width as usize) as u32;
        let height = (rows * frame_max_height as usize) as u32;

        let mut texture = DynamicImage::new_rgba8(width, height);

        for (index, fh) in frame_headers.iter_mut().enumerate() {
            self.reader.seek(SeekFrom::Start(u64::from(
                (header.frame_data_offset as u32) + fh.data_offset,
            )))?;

            let mut buf = vec![0; fh.uncompressed_size as usize];

            match fh.compression_type {
                CompressionType::None => {
                    self.reader.read_exact(&mut buf)?;
                }
                CompressionType::Packbits => {
                    let mut reader =
                        PackBitsReader::new(&mut self.reader, fh.compressed_size as u64);
                    reader.read_exact(&mut buf)?;
                }
                CompressionType::ZeroRuns => {
                    let mut reader =
                        ZeroRunsReader::new(&mut self.reader, fh.compressed_size as u64);
                    reader.read_exact(&mut buf)?;
                }
            }

            // Calculate the top-left coordinates for the frame.
            let x_offset = (index % columns) as u32 * frame_max_width as u32;
            let y_offset = (index / columns) as u32 * frame_max_height as u32;

            // Calculate the top-left coordinates to center the frame in its
            // allocated space.
            let x_pad = ((frame_max_width - fh.width) / 2) as u32;
            let y_pad = ((frame_max_height - fh.height) / 2) as u32;

            fh.x -= x_pad as i16;
            fh.y -= y_pad as i16;

            // Iterate over the buffer and copy the pixels into the new image.
            buf.iter().enumerate().for_each(|(i, &b)| {
                let x = i as u32 % fh.width as u32;
                let y = i as u32 / fh.width as u32;
                let mut color = color_table[fh.color_table_offset as usize + b as usize];

                // Convert cyan (r=0, g=255, b=255) to shadow with 45%
                // transparency.
                //
                // TODO: Only do in shader.
                if color.to_rgb() == Rgb([0u8, 255u8, 255u8]) {
                    color = Rgba([0u8, 0u8, 0u8, 115u8]);
                }

                texture.put_pixel(x + x_offset + x_pad, y + y_offset + y_pad, color);
            });
        }

        Ok(SpriteSheet {
            texture,
            frames: frame_headers
                .iter()
                .map(|fh| Frame {
                    frame_type: fh.frame_type.clone(),
                    x: fh.x,
                    y: fh.y,
                    width: fh.width,
                    height: fh.height,
                })
                .collect(),
            atlas_layout: AtlasLayout {
                tile_size: (frame_max_width, frame_max_height),
                columns,
                rows,
                padding: None,
                offset: None,
            },
        })
    }

    fn decode_header(&mut self) -> Result<Header, DecodeError> {
        let mut buf = [0; HEADER_SIZE];
        self.reader.read_exact(&mut buf)?;

        let format = String::from_utf8_lossy(&buf[0..4]).to_string();
        if format != FORMAT {
            return Err(DecodeError::InvalidFormat(format));
        }

        Ok(Header {
            _file_size: u16::from_le_bytes([buf[4], buf[5]]),
            _frame_header_offset: u16::from_le_bytes([buf[8], buf[9]]),
            frame_data_offset: u16::from_le_bytes([buf[12], buf[13]]),
            _color_table_offset: u16::from_le_bytes([buf[16], buf[17]]),
            color_table_entries: u16::from_le_bytes([buf[20], buf[21]]),
            _palette_count: u16::from_le_bytes([buf[24], buf[25]]),
            frame_count: u16::from_le_bytes([buf[28], buf[29]]),
        })
    }

    fn decode_frame_headers(
        &mut self,
        header: Header,
    ) -> Result<(Vec<FrameHeader>, u16, u16), DecodeError> {
        let mut frame_headers = Vec::with_capacity(header.frame_count as usize);

        let mut max_width = 0;
        let mut max_height = 0;

        for _ in 0..header.frame_count {
            let mut buf = [0; FRAME_HEADER_SIZE];
            self.reader.read_exact(&mut buf)?;

            let frame_type = FrameType::from(buf[0]);
            let compression_type = CompressionType::from(buf[1]);
            let color_count = u16::from_le_bytes(buf[2..4].try_into().unwrap());
            let x = i16::from_le_bytes(buf[4..6].try_into().unwrap());
            let y = i16::from_le_bytes(buf[6..8].try_into().unwrap());
            let width = u16::from_le_bytes(buf[8..10].try_into().unwrap());
            let height = u16::from_le_bytes(buf[10..12].try_into().unwrap());
            let data_offset = u32::from_le_bytes(buf[12..16].try_into().unwrap());
            let compressed_size = u32::from_le_bytes(buf[16..20].try_into().unwrap());
            let uncompressed_size = u32::from_le_bytes(buf[20..24].try_into().unwrap());
            let color_table_offset = u32::from_le_bytes(buf[24..28].try_into().unwrap());
            let _padding = u32::from_le_bytes(buf[28..32].try_into().unwrap());

            frame_headers.push(FrameHeader {
                frame_type,
                compression_type,
                _color_count: color_count,
                x,
                y,
                width,
                height,
                data_offset,
                compressed_size,
                uncompressed_size,
                color_table_offset,
                _padding,
            });

            max_width = max_width.max(width);
            max_height = max_height.max(height);
        }

        Ok((frame_headers, max_width, max_height))
    }

    fn decode_color_table(&mut self, header: Header) -> Result<Vec<Rgba<u8>>, DecodeError> {
        let mut buf = vec![0; 4 * header.color_table_entries as usize];
        self.reader.read_exact(&mut buf)?;

        let mut color_table = Vec::with_capacity(header.color_table_entries as usize);
        for i in 0..header.color_table_entries {
            let entry = &buf[4 * i as usize..4 * (i + 1) as usize];
            let b = entry[0];
            let g = entry[1];
            let r = entry[2];
            let mut a = 255;

            if b < 8 && g < 8 && r < 8 {
                a = 0;
            }

            color_table.push(Rgba([r, g, b, a]));
        }
        Ok(color_table)
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
    fn test_decode_bernhd() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
            "SPRITES",
            "BERNHD.SPR",
        ]
        .iter()
        .collect();

        let file = File::open(d.clone()).unwrap();

        let sheet = Decoder::new(file).decode().unwrap();

        assert_eq!(sheet.frames.len(), 104);
        assert_eq!(sheet.atlas_layout.tile_size, (59, 64));
        assert_eq!(sheet.atlas_layout.columns, 14);
        assert_eq!(sheet.atlas_layout.rows, 8);
        assert_eq!(sheet.atlas_layout.padding, None);
        assert_eq!(sheet.atlas_layout.offset, None);
    }

    #[test]
    fn test_decode_hbgrucav() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
            "BANNERS",
            "HBGRUCAV.SPR",
        ]
        .iter()
        .collect();

        let file = File::open(d.clone()).unwrap();

        let sprite = Decoder::new(file).decode().unwrap();

        assert_eq!(sprite.frames.len(), 2);
        assert_eq!(sprite.atlas_layout.tile_size, (32, 32));
        assert_eq!(sprite.atlas_layout.columns, 2);
        assert_eq!(sprite.atlas_layout.rows, 1);
        assert_eq!(sprite.atlas_layout.padding, None);
        assert_eq!(sprite.atlas_layout.offset, None);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "sprite-sheets"]
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
                if ext.to_string_lossy().to_uppercase() == "SPR" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let file = File::open(path).unwrap();
                    let sprite = Decoder::new(file).decode().unwrap();

                    let parent_dir = path.components().rev().nth(1).unwrap();
                    let output_dir = root_output_dir.join(parent_dir);
                    std::fs::create_dir_all(&output_dir).unwrap();

                    if sprite.texture.width() == 0 || sprite.texture.height() == 0 {
                        println!("Skipping empty image {:?}", path.file_name().unwrap());
                        return;
                    }

                    let output_path = append_ext("png", output_dir.join(path.file_name().unwrap()));
                    sprite.texture.save(output_path).unwrap();
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
