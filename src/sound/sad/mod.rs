mod decoder;
mod encoder;

use super::audio::{Block, BlockTrait};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io;

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct StereoAudio {
    pub left_blocks: Vec<Block>,
    pub right_blocks: Vec<Block>,

    // Note: Storing these so that a re-encoded stream is correct. sample 99 is
    // always a different value and index 99 is always equal to 99. Not sure if
    // sample 99 needs to end up in the decoded stream—currently it does not.
    left_sample99: i16,
    left_index99: i16,
    right_sample99: i16,
    right_index99: i16,
}

impl StereoAudio {
    pub fn channel_count(&self) -> usize {
        2
    }

    pub fn to_wav(&self) -> Result<Vec<u8>, io::Error> {
        let mut buffer = io::Cursor::new(Vec::new());

        {
            let mut writer = WavWriter::new(
                &mut buffer,
                WavSpec {
                    channels: self.channel_count() as u16,
                    sample_rate: 22050,
                    bits_per_sample: 16,
                    sample_format: SampleFormat::Int,
                },
            )
            .map_err(io::Error::other)?;

            let mut samples = Vec::new();

            for i in 0..self.left_blocks.len() {
                let left_pcm16_block = &self.left_blocks[i].as_pcm16_block();
                let right_pcm16_block = &self.right_blocks[i].as_pcm16_block();

                for j in 0..left_pcm16_block.data.len() {
                    samples.push(left_pcm16_block.data[j]);
                    samples.push(right_pcm16_block.data[j]);
                }
            }

            let mut sample_writer = writer.get_i16_writer(samples.len() as u32);

            for sample in samples {
                sample_writer.write_sample(sample);
            }

            sample_writer.flush().map_err(io::Error::other)?;
        }

        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Write;
    use pretty_assertions::assert_eq;
    use sha2::{Digest, Sha256};
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        path::{Path, PathBuf},
    };

    fn roundtrip_test(original_bytes: &[u8], a: &StereoAudio) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(a).unwrap();

        let original_bytes = original_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{b:02X}"))
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
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(original_bytes, encoded_bytes);
    }

    #[test]
    fn test_decode_09eerie() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "SOUND",
            "MUSIC",
            "09EERIE.SAD",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();

        let file = File::open(d).unwrap();
        let a = Decoder::new(file).decode().unwrap();

        roundtrip_test(&original_bytes, &a);

        // Instead of comparing the decoded audio to a known-good audio file, we
        // can compare the SHA-256 hash of the decoded audio to a known-good
        // hash. This way, we can ensure that the audio is decoded correctly
        // without needing to store a known-good audio file in the repository.
        let wav = a.to_wav().unwrap();
        let mut hasher = Sha256::new();
        hasher.update(wav.as_slice());
        let result = hasher.finalize();
        let result_str = format!("{result:x}");
        assert_eq!(
            result_str,
            "28fb332692962c24f137fc6fffadaf47290cc4011c17b2b238c07c1235a108be"
        );
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [std::env::var("DARKOMEN_PATH").unwrap().as_str(), "DARKOMEN"]
            .iter()
            .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "sads"]
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
            if ext.to_string_lossy().to_uppercase() != "SAD" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let original_bytes = std::fs::read(path).unwrap();

            let file = File::open(path).unwrap();
            let a = Decoder::new(file).decode().unwrap();

            roundtrip_test(&original_bytes, &a);

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

            let output_path = append_ext("wav", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            output_file
                .write_all(a.to_wav().unwrap().as_slice())
                .unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
