use super::*;
use image::{DynamicImage, GenericImage, Rgba};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{
    fmt,
    io::{Error as IoError, Read, Seek, SeekFrom},
};

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
    InvalidFormat(String),
    InvalidSpriteType(u8),
    InvalidCompression(u8),
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
            DecodeError::IoError(error) => write!(f, "IO error: {error}"),
            DecodeError::InvalidFormat(format) => write!(f, "invalid format: {format}"),
            DecodeError::InvalidSpriteType(v) => write!(f, "invalid sprite type: {v}"),
            DecodeError::InvalidCompression(v) => write!(f, "invalid compression: {v}"),
        }
    }
}

/// The sprite format ID used in all .SPR files.
///
/// "WHDO" is probably an initialism for "Warhammer: Dark Omen".
const FORMAT: &str = "WHDO";

const HEADER_SIZE_BYTES: usize = 32;
const SPRITE_HEADER_SIZE_BYTES: usize = 32;

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct Header {
    _file_size_bytes: u16,
    _sprite_header_offset: u16,
    sprite_data_offset: u16,
    _color_table_offset: u16,
    color_table_entries: u16,
    _palette_count: u16,
    sprite_count: u16,
}

#[repr(u8)]
#[derive(Clone, Copy, Default, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(all(feature = "bevy_reflect", feature = "debug"), reflect(Debug))]
pub enum Compression {
    #[default]
    None = 0,
    Packbits = 1,
    ZeroRuns = 2,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct SpriteHeader {
    typ: SpriteType,
    compression: Compression,
    _color_count: u16,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    data_offset: u32,
    compressed_size_bytes: u32,
    uncompressed_size_bytes: u32,
    color_table_offset: u32,
    _padding: u32, // last 4 bytes are not used
}

#[repr(u8)]
#[derive(Clone, Copy, Default, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive)]
#[cfg_attr(feature = "debug", derive(Debug))]
enum SpriteType {
    /// Indicates the sprite is a repeat of a previous sprite.
    Repeat = 0,
    /// Indicates the sprite should be flipped along the x axis.
    FlipX = 1,
    /// Indicates the sprite should be flipped along the y axis.
    FlipY = 2,
    /// Indicates the sprite should be flipped along the x and y axes.
    FlipXY = 3,
    /// Indicates a normal sprite.
    #[default]
    Normal = 4,
    /// Indicates the sprite is empty. There is no sprite or palette data
    /// associated with the sprite. The sprite's width and height are 0.
    Empty = 5,
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

    pub fn decode(&mut self) -> Result<SpriteSheet, DecodeError> {
        let header = self.decode_header()?;

        let sprite_headers = self.decode_sprite_headers(header.clone())?;

        let color_table = self.decode_color_table(header.clone())?;

        let mut textures = Vec::with_capacity(sprite_headers.len());
        let mut texture_descriptors = Vec::with_capacity(sprite_headers.len());

        for h in sprite_headers.iter() {
            self.reader.seek(SeekFrom::Start(u64::from(
                (header.sprite_data_offset as u32) + h.data_offset,
            )))?;

            let mut buf = vec![0; h.uncompressed_size_bytes as usize];

            match h.compression {
                Compression::None => {
                    self.reader.read_exact(&mut buf)?;
                }
                Compression::Packbits => {
                    let mut reader =
                        PackBitsReader::new(&mut self.reader, h.compressed_size_bytes as u64);
                    reader.read_exact(&mut buf)?;
                }
                Compression::ZeroRuns => {
                    let mut reader =
                        ZeroRunsReader::new(&mut self.reader, h.compressed_size_bytes as u64);
                    reader.read_exact(&mut buf)?;
                }
            }

            let flip_x = h.typ == SpriteType::FlipX || h.typ == SpriteType::FlipXY;
            let flip_y = h.typ == SpriteType::FlipY || h.typ == SpriteType::FlipXY;

            let mut texture = DynamicImage::new_rgba8(h.width as u32, h.height as u32);

            for (i, &b) in buf.iter().enumerate() {
                let x = i as u32 % h.width as u32;
                let y = i as u32 / h.width as u32;

                let mut color = color_table[h.color_table_offset as usize + b as usize];

                // TODO: Color replacements that should probably be done in a
                // shader.

                // If R, G and B are < 8 then the pixel is transparent.
                if color.0[0] < 8 && color.0[1] < 8 && color.0[2] < 8 {
                    color = Rgba([0, 0, 0, 0]);
                }

                // If R, G and B are each exactly 8, then the pixel is full
                // black, i.e., "black" hack.
                if color.0[0] == 8 && color.0[1] == 8 && color.0[2] == 8 {
                    color = Rgba([0, 0, 0, 255]);
                }

                // If color is cyan then the pixel is part of the sprite's
                // shadow.
                if color.0[0] == 0 && color.0[1] == 255 && color.0[2] == 255 {
                    color = Rgba([0, 0, 0, 200]); // 78% transparency
                }

                let x = if flip_x { h.width as u32 - x - 1 } else { x };
                let y = if flip_y { h.height as u32 - y - 1 } else { y };

                texture.put_pixel(x, y, color);
            }

            textures.push(texture);

            texture_descriptors.push(TextureDescriptor {
                x: h.x,
                y: h.y,
                width: h.width,
                height: h.height,
            });
        }

        Ok(SpriteSheet {
            textures,
            texture_descriptors,
        })
    }

    fn decode_header(&mut self) -> Result<Header, DecodeError> {
        let mut buf = [0; HEADER_SIZE_BYTES];
        self.reader.read_exact(&mut buf)?;

        let format = String::from_utf8_lossy(&buf[0..4]).to_string();
        if format != FORMAT {
            return Err(DecodeError::InvalidFormat(format));
        }

        Ok(Header {
            _file_size_bytes: u16::from_le_bytes([buf[4], buf[5]]),
            _sprite_header_offset: u16::from_le_bytes([buf[8], buf[9]]),
            sprite_data_offset: u16::from_le_bytes([buf[12], buf[13]]),
            _color_table_offset: u16::from_le_bytes([buf[16], buf[17]]),
            color_table_entries: u16::from_le_bytes([buf[20], buf[21]]),
            _palette_count: u16::from_le_bytes([buf[24], buf[25]]),
            sprite_count: u16::from_le_bytes([buf[28], buf[29]]),
        })
    }

    fn decode_sprite_headers(&mut self, header: Header) -> Result<Vec<SpriteHeader>, DecodeError> {
        let mut headers = Vec::with_capacity(header.sprite_count as usize);

        for _ in 0..header.sprite_count {
            let mut buf = [0; SPRITE_HEADER_SIZE_BYTES];
            self.reader.read_exact(&mut buf)?;

            let typ =
                SpriteType::try_from(buf[0]).map_err(|_| DecodeError::InvalidSpriteType(buf[0]))?;
            let compression = Compression::try_from(buf[1])
                .map_err(|_| DecodeError::InvalidCompression(buf[1]))?;
            let color_count = u16::from_le_bytes(buf[2..4].try_into().unwrap());
            let x = i16::from_le_bytes(buf[4..6].try_into().unwrap());
            let y = i16::from_le_bytes(buf[6..8].try_into().unwrap());
            let width = u16::from_le_bytes(buf[8..10].try_into().unwrap());
            let height = u16::from_le_bytes(buf[10..12].try_into().unwrap());
            let data_offset = u32::from_le_bytes(buf[12..16].try_into().unwrap());
            let compressed_size_bytes = u32::from_le_bytes(buf[16..20].try_into().unwrap());
            let uncompressed_size_bytes = u32::from_le_bytes(buf[20..24].try_into().unwrap());
            let color_table_offset = u32::from_le_bytes(buf[24..28].try_into().unwrap());
            let _padding = u32::from_le_bytes(buf[28..32].try_into().unwrap());

            headers.push(SpriteHeader {
                typ,
                compression,
                _color_count: color_count,
                x,
                y,
                width,
                height,
                data_offset,
                compressed_size_bytes,
                uncompressed_size_bytes,
                color_table_offset,
                _padding,
            });
        }

        Ok(headers)
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

        assert_eq!(sheet.textures.len(), 104);
        assert_eq!(sheet.texture_descriptors.len(), 104);
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

        let sheet = Decoder::new(file).decode().unwrap();

        assert_eq!(sheet.textures.len(), 2);
        assert_eq!(sheet.texture_descriptors.len(), 2);
    }

    #[test]
    fn test_decode_mi() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GRAPHICS",
            "SPRITES",
            "MI.SPR",
        ]
        .iter()
        .collect();

        let file = File::open(d.clone()).unwrap();

        let sheet = Decoder::new(file).decode().unwrap();

        assert_eq!(sheet.textures.len(), 48);
        assert_eq!(sheet.texture_descriptors.len(), 48);

        // Check the first texture descriptor. These values are known to be
        // correct and a correct sprite anchor can be created from these values.
        assert_eq!(sheet.texture_descriptors[0].x, -22);
        assert_eq!(sheet.texture_descriptors[0].y, -55);
        assert_eq!(sheet.texture_descriptors[0].width, 46);
        assert_eq!(sheet.texture_descriptors[0].height, 44);

        // Check the first 8 texture descriptors y and height. There's nothing
        // special about these values, they are just known to have the same
        // value for this particular magic item.
        for i in 0..8 {
            assert_eq!(sheet.texture_descriptors[i].y, -55);
            assert_eq!(sheet.texture_descriptors[i].height, 44);
        }
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
            let Some(ext) = path.extension() else {
                return;
            };
            if ext.to_string_lossy().to_uppercase() != "SPR" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let file = File::open(path).unwrap();
            let sheet = Decoder::new(file).decode().unwrap();

            let parent_dir = path.components().rev().nth(1).unwrap();
            let output_dir = root_output_dir.join(parent_dir);
            std::fs::create_dir_all(&output_dir).unwrap();

            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut buffer = String::new();
            ron::ser::to_writer_pretty(&mut buffer, &sheet, Default::default()).unwrap();
            std::fs::write(output_path, buffer).unwrap();

            let output_dir = output_dir.join(path.file_stem().unwrap());
            std::fs::create_dir_all(&output_dir).unwrap();

            for (i, texture) in sheet.textures.iter().enumerate() {
                if texture.width() == 0 || texture.height() == 0 {
                    println!("Skipping empty image {:?}", path.file_name().unwrap());
                    continue;
                }
                let output_path = output_dir.join(format!("{i}.png"));
                texture.save(output_path).unwrap();
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
