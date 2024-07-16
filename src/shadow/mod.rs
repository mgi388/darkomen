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
pub struct Lightmap {
    pub width: u32,
    pub height: u32,
    /// A list of large blocks for the lightmap.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub blocks: Vec<LightmapBlock>,
    /// A list of height offsets for an 8x8 block. Each item is a list which
    /// must have exactly 64 (8x8) u8s. A given height offset should be added to
    /// the base height of the block.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub height_offsets: Vec<Vec<u8>>,
}

impl Lightmap {
    fn normalized_offset_height(offset_height: u8) -> f32 {
        offset_height as f32 / 8.0
    }

    fn min_and_max_normalized_base_height(&self) -> (f32, f32) {
        self.blocks
            .iter()
            .map(|block| block.normalized_base_height())
            .fold((f32::MAX, f32::MIN), |(min, max), val| {
                (min.min(val), max.max(val))
            })
    }

    pub fn image(&self) -> DynamicImage {
        let mut img = DynamicImage::new_rgba8(self.width, self.height);

        let (min_normalized_base_height, max_normalized_base_height) =
            self.min_and_max_normalized_base_height();

        let mut row = 0;
        let mut col = 0;

        for block in &self.blocks {
            let height_offsets = &self.height_offsets[block.height_offsets_index as usize];

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

                    let offset_height = height_offsets[(x + y * 8) as usize];

                    let color = Lightmap::calculate_color(
                        min_normalized_base_height,
                        max_normalized_base_height,
                        block,
                        Lightmap::normalized_offset_height(offset_height),
                    );

                    img.put_pixel(target_x, target_y, Rgba([color, color, color, 255]));
                }
            }

            col += 1;
        }

        img.fliph() // needs to be flipped horizontally for some reason
    }

    fn calculate_color(
        min_normalized_base_height: f32,
        max_normalized_base_height: f32,
        block: &LightmapBlock,
        normalized_offset_height: f32,
    ) -> u8 {
        // The largest value that can be stored for a block's height is u16::MAX
        // because base height is an i32 and u16::MAX is the largest positive
        // value that can be stored in an i32. u16::MAX is then divided by 1024
        // to get the normalized maximum.
        //
        // Technically, if a block's base height was u16::MAX, and an offset
        // height was any value other than 0, the combined height would
        // overflow. But in all the game files, the largest value for a block's
        // base height is below (u16::MAX - u8::MAX) so this is not a concern.
        const MAX_NORMALIZED_HEIGHT: f32 = u16::MAX as f32 / 1024.;

        // The largest value that can be stored for a block's offset height is
        // u8::MAX because offset height is a u8. u8::MAX is then divided by 8
        // to get the normalized maximum.
        const MAX_NORMALIZED_OFFSET_HEIGHT: f32 = u8::MAX as f32 / 8.;

        let normalized_height = block.normalized_base_height() + normalized_offset_height;

        let scaled_value = normalized_height / MAX_NORMALIZED_HEIGHT;

        let min = min_normalized_base_height / MAX_NORMALIZED_HEIGHT;
        let max =
            (max_normalized_base_height + MAX_NORMALIZED_OFFSET_HEIGHT) / MAX_NORMALIZED_HEIGHT;

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

pub struct LightmapBlock {
    /// The base height of all 64 (8x8) values in the block.
    pub base_height: i32,
    /// An index into the height offsets list. Used to get the 64 (8x8) values
    /// that make up the block. The values are height offsets based on the base
    /// height. To get the height at a specific point, combine the base height
    /// with the offset at that point.
    pub height_offsets_index: u32,
}

impl LightmapBlock {
    /// Returns the normalized base height of the block by dividing the stored
    /// integer value by 1024. This conversion reflects the original intention
    /// for the height to be represented as a float.
    pub fn normalized_base_height(&self) -> f32 {
        self.base_height as f32 / 1024.0
    }
}

fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, GenericImageView, RgbaImage};
    use pretty_assertions::assert_eq;
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        io,
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
    fn test_min_and_max_normalized_base_height() {
        let lightmap = Lightmap {
            blocks: vec![
                LightmapBlock {
                    base_height: -1024,
                    height_offsets_index: 0,
                },
                LightmapBlock {
                    base_height: 1024,
                    height_offsets_index: 0,
                },
                LightmapBlock {
                    base_height: 2048,
                    height_offsets_index: 0,
                },
            ],
            height_offsets: vec![vec![0; 64]; 1],
            ..Default::default()
        };

        let (min, max) = lightmap.min_and_max_normalized_base_height();
        assert_eq!(min, -1.0);
        assert_eq!(max, 2.0);
    }

    fn roundtrip_test(original_bytes: &[u8], l: &Lightmap) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(l).unwrap();

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
        let lightmap = Decoder::new(file).decode().unwrap();

        assert_eq!(lightmap.width, 184);
        assert_eq!(lightmap.height, 200);
        assert_eq!(lightmap.blocks.len(), 575);
        assert_eq!(lightmap.height_offsets.len(), 484);

        roundtrip_test(&original_bytes, &lightmap);
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
        let lightmap = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &lightmap);
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
        let lightmap = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &lightmap);
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
        let lightmap = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &lightmap);
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

            let mut paths = std::fs::read_dir(dir)
                .unwrap()
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()
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
                if ext.to_string_lossy().to_uppercase() == "SHD" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let original_bytes = std::fs::read(path).unwrap();

                    let file = File::open(path).unwrap();
                    let lightmap = Decoder::new(file).decode().unwrap();

                    roundtrip_test(&original_bytes, &lightmap);

                    let has_invalid_height_offsets_index = lightmap.blocks.iter().any(|block| {
                        block.height_offsets_index as usize >= lightmap.height_offsets.len()
                    });
                    assert!(
                        !has_invalid_height_offsets_index,
                        "found a block with an invalid height offsets index"
                    );

                    let img = lightmap.image();

                    // Compare against the golden image.
                    {
                        let golden_images_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join("src")
                            .join("shadow")
                            .join("testdata")
                            .join("images");
                        let golden_img_path = golden_images_path
                            .join(path.file_name().unwrap())
                            .with_extension("golden.png");

                        if !Path::new(&golden_img_path).exists() {
                            img.save(&golden_img_path).unwrap();
                        }

                        let golden_img = image::open(&golden_img_path).unwrap();

                        assert_eq!(img.dimensions(), golden_img.dimensions());

                        let pixels_equal = img
                            .pixels()
                            .zip(golden_img.clone().pixels())
                            .all(|(p1, p2)| p1 == p2);

                        if !pixels_equal {
                            // Write out the actual image so it can be visually
                            // compared against the golden.
                            img.save(
                                golden_images_path
                                    .join(path.file_name().unwrap())
                                    .with_extension("actual.png"),
                            )
                            .unwrap();

                            // Write out an image of the diff between the two.
                            let diff_bytes = img
                                .clone()
                                .into_bytes()
                                .into_iter()
                                .zip(golden_img.clone().into_bytes())
                                .map(|(p1, p2)| {
                                    if p1 > p2 {
                                        return p1 - p2;
                                    }
                                    p2 - p1
                                })
                                .map(|p| 255 - p) // inverting the diff fixes alpha going to 0 in the previous map
                                .collect::<Vec<_>>();
                            let diff_img = DynamicImage::ImageRgba8(
                                RgbaImage::from_raw(
                                    golden_img.width(),
                                    golden_img.height(),
                                    diff_bytes,
                                )
                                .unwrap(),
                            );
                            diff_img
                                .save(
                                    golden_images_path
                                        .join(path.file_name().unwrap())
                                        .with_extension("diff.png"),
                                )
                                .unwrap();
                        }

                        assert!(pixels_equal, "pixels do not match");
                    }

                    // Write out the decoded data for manual inspection.
                    {
                        // RON.
                        let output_path =
                            append_ext("ron", root_output_dir.join(path.file_name().unwrap()));
                        let mut output_file = File::create(output_path).unwrap();
                        ron::ser::to_writer_pretty(&mut output_file, &lightmap, Default::default())
                            .unwrap();

                        // Image.
                        let output_dir = root_output_dir.join("lightmaps");
                        std::fs::create_dir_all(&output_dir).unwrap();

                        let output_path = output_dir
                            .join(path.file_stem().unwrap())
                            .with_extension("lightmap.png");
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
