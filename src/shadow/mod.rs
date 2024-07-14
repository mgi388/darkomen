mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use image::{DynamicImage, GenericImage, Rgba};
use serde::Serialize;

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Shadow {
    pub terrain: Terrain,
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Terrain {
    pub width: u32,
    pub height: u32,
    /// A list of large blocks for the heightmap.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub heightmap_blocks: Vec<TerrainBlock>,
    /// A list of offsets for 8x8 block. Height offset for each block based on
    /// minimum height. Each item is a list which must have exactly 64 (8x8)
    /// u8s.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub offsets: Vec<Vec<u8>>,
}

impl Terrain {
    fn normalized_offset_height(offset_height: u8) -> f32 {
        offset_height as f32 / 8.0
    }

    fn min_and_max_normalized_min_height(&self) -> (f32, f32) {
        self.heightmap_blocks
            .iter()
            .map(|block| block.normalized_min_height())
            .fold((f32::MAX, f32::MIN), |(min, max), val| {
                (min.min(val), max.max(val))
            })
    }

    pub fn heightmap_image(&self) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(self.width, self.height);

        let (min_normalized_min_height, max_normalized_min_height) =
            self.min_and_max_normalized_min_height();

        let mut row = 0;
        let mut col = 0;

        for block in &self.heightmap_blocks {
            let offsets = &self.offsets[block.offset_index as usize];

            if col * 8 >= self.width {
                col = 0;
                row += 1;
            }

            for y in 0..8 {
                let target_y = row * 8 + y;

                if target_y >= self.height {
                    break;
                }

                for x in 0..8 {
                    let target_x = col * 8 + x;

                    if target_x >= self.width {
                        break;
                    }

                    let offset_height = offsets[(x + y * 8) as usize];

                    let color = Terrain::calculate_color(
                        min_normalized_min_height,
                        max_normalized_min_height,
                        block,
                        Terrain::normalized_offset_height(offset_height),
                    );

                    img.put_pixel(target_x, target_y, Rgba([color, color, color, 255]));
                }
            }

            col += 1;
        }

        img.fliph() // needs to be flipped horizontally for some reason
    }

    fn calculate_color(
        min_normalized_min_height: f32,
        max_normalized_min_height: f32,
        block: &TerrainBlock,
        normalized_offset_height: f32,
    ) -> u8 {
        // The largest value that can be stored for a block's height is u16::MAX
        // because minimum height is an i32 and u16::MAX is the largest positive
        // value that can be stored in an i32. u16::MAX is then divided by 1024
        // to get the normalized maximum.
        //
        // Technically, if a block's minimum height was u16::MAX, and an offset
        // was any value other than 0, the combined height would overflow. But
        // in all the game files, the largest value for a block's minimum height
        // is below (u16::MAX - u8::MAX) so this is not a concern.
        const MAX_NORMALIZED_HEIGHT: f32 = u16::MAX as f32 / 1024.;

        // The largest value that can be stored for a block's offset height is
        // u8::MAX because offset height is a u8. u8::MAX is then divided by 8
        // to get the normalized maximum.
        const MAX_NORMALIZED_OFFSET_HEIGHT: f32 = u8::MAX as f32 / 8.;

        let scaled_value =
            (block.normalized_min_height() + normalized_offset_height) / (MAX_NORMALIZED_HEIGHT);

        let min = min_normalized_min_height / MAX_NORMALIZED_HEIGHT;
        let max =
            (max_normalized_min_height + MAX_NORMALIZED_OFFSET_HEIGHT) / MAX_NORMALIZED_HEIGHT;

        let normalized_value = normalize(scaled_value, min, max);

        // Convert the normalized value (between 0 and 1) to a color (between 0
        // and 255).
        let color = normalized_value * 255.;

        // Invert the color, otherwise shadows are white.
        let color = 255. - color;

        color as u8 // truncate any fractional part
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]

pub struct TerrainBlock {
    /// The minimum height of all 64 (8x8) values in the block. This is the base
    /// height for the block.
    pub min_height: i32,
    /// An index into the offsets list. Used to get the 64 (8x8) values that
    /// make up the block. The values are height offsets based on the minimum
    /// height. To get the height at a specific point, you need to combine the
    /// minimum height with the offset at that point.
    pub offset_index: u32,
}

impl TerrainBlock {
    /// Returns the normalized minimum height of the block by dividing the
    /// stored integer value by 1024. This conversion reflects the original
    /// intention for the height to be represented as a float.
    pub fn normalized_min_height(&self) -> f32 {
        self.min_height as f32 / 1024.0
    }
}

fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        path::{Path, PathBuf},
    };

    macro_rules! test_normalize {
        ($name:ident, $value:expr, $min:expr, $max:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let value = $value;
                let min = $min;
                let max = $max;
                let expected = $expected;

                let result = normalize(value, min, max);
                assert_eq!(result, expected);
            }
        };
    }

    test_normalize!(test_normalize_min, 0.0, 0.0, 1.0, 0.0);
    test_normalize!(test_normalize_max, 1.0, 0.0, 1.0, 1.0);
    test_normalize!(test_normalize_middle, 0.5, 0.0, 1.0, 0.5);
    test_normalize!(test_normalize_negative_min, -1.0, -1.0, 1.0, 0.0);
    test_normalize!(test_normalize_negative_max, 1.0, -1.0, 1.0, 1.0);
    test_normalize!(test_normalize_negative_middle, 0.0, -1.0, 1.0, 0.5);
    test_normalize!(test_normalize_large_range_low_end, 0.5, 0.0, 100.0, 0.005);
    test_normalize!(test_normalize_large_range_middle, 50.0, 0.0, 100.0, 0.5);
    test_normalize!(test_normalize_large_range_high_end, 99.5, 0.0, 100.0, 0.995);

    #[test]
    fn test_min_and_max_normalized_min_height() {
        let terrain = Terrain {
            heightmap_blocks: vec![
                TerrainBlock {
                    min_height: -1024,
                    offset_index: 0,
                },
                TerrainBlock {
                    min_height: 1024,
                    offset_index: 0,
                },
                TerrainBlock {
                    min_height: 2048,
                    offset_index: 0,
                },
            ],
            offsets: vec![vec![0; 64]; 1],
            ..Default::default()
        };

        let (min, max) = terrain.min_and_max_normalized_min_height();
        assert_eq!(min, -1.0);
        assert_eq!(max, 2.0);
    }

    fn roundtrip_test(original_bytes: &[u8], s: &Shadow) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(s).unwrap();

        let original_bytes = original_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let encoded_bytes = encoded_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(original_bytes, encoded_bytes);
    }

    #[test]
    fn test_decode_b1_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "B1_01.SHD",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let shadow = Decoder::new(file).decode().unwrap();

        assert_eq!(shadow.terrain.width, 184);
        assert_eq!(shadow.terrain.height, 200);

        roundtrip_test(&original_bytes, &shadow);
    }

    #[test]
    fn test_decode_mb4_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B4_01",
            "MB4_01.SHD",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let shadow = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &shadow);
    }

    #[test]
    fn test_decode_b4_09() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B4_09",
            "B4_09.SHD",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let shadow = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &shadow);
    }

    #[test]
    fn test_decode_b5_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B5_01",
            "B5_01.SHD",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d.clone()).unwrap();
        let shadow = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &shadow);
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

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "shadows"]
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
                if ext.to_string_lossy().to_uppercase() == "SHD" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let original_bytes = std::fs::read(path).unwrap();

                    let file = File::open(path).unwrap();
                    let shadow = Decoder::new(file).decode().unwrap();

                    roundtrip_test(&original_bytes, &shadow);

                    let has_invalid_offset_index =
                        shadow.terrain.heightmap_blocks.iter().any(|block| {
                            block.offset_index as usize >= shadow.terrain.offsets.len()
                        });
                    assert!(
                        !has_invalid_offset_index,
                        "found a block with an invalid offset index"
                    );

                    let output_path =
                        append_ext("ron", root_output_dir.join(path.file_name().unwrap()));
                    let mut output_file = File::create(output_path).unwrap();
                    ron::ser::to_writer_pretty(&mut output_file, &shadow, Default::default())
                        .unwrap();

                    // Write out the heightmap image.
                    {
                        let output_dir = root_output_dir.join("heightmaps");
                        std::fs::create_dir_all(&output_dir).unwrap();

                        let img = shadow.terrain.heightmap_image();

                        let output_path = output_dir
                            .join(path.file_stem().unwrap())
                            .with_extension("map.png");

                        img.save(output_path).unwrap();
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
