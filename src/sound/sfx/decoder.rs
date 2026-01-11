use super::{lexer::*, *};
use std::{
    collections::HashMap,
    fmt,
    io::{Error as IoError, Read, Seek},
};

struct IdentifierToken {
    value: String,
    line: usize,
    column: usize,
}

fn parse_directive(tokens: &[Token]) -> (IdentifierToken, IdentifierToken) {
    assert_eq!(tokens.len(), 2);

    let field = match &tokens[0] {
        Token::Identifier {
            value,
            line,
            column,
        } => IdentifierToken {
            value: value.to_string(),
            line: *line,
            column: *column,
        },
        _ => {
            let (line, column) = tokens[0].line_column();
            panic!(
                "{}:{}: expected directive field, found '{:?}'",
                line, column, tokens[0]
            );
        }
    };

    let value = match &tokens[1] {
        Token::Identifier {
            value,
            line,
            column,
        } => IdentifierToken {
            value: value.to_string(),
            line: *line,
            column: *column,
        },
        _ => {
            let (line, column) = tokens[1].line_column();
            panic!(
                "{}:{}: expected directive value, found '{:?}'",
                line, column, tokens[1]
            );
        }
    };

    (field, value)
}

fn parse_sound(tokens: &[Token]) -> (Sound, usize) {
    let mut sound = Sound::default();

    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Comment { .. } => {
                i += 1; // consume this token
            }
            Token::BeginDirective { line, column } => {
                i += 1; // consume this token

                let directive_tokens = &tokens[i..i + 2];
                if directive_tokens.len() < 2 {
                    panic!("{line}:{column}: expected directive field and value, found 'EOF'",);
                }
                let (field_token, value_token) = parse_directive(directive_tokens);
                let value = value_token.value;
                i += directive_tokens.len(); // consume the directive tokens

                match field_token.value.as_str() {
                    "SAMPLE" => {
                        sound.file_stem = value.to_string();
                    }
                    "FREQ" => {
                        sound.frequency = value
                            .parse()
                            .expect("frequency should be an unsigned 32-bit integer");
                    }
                    "FREQDEV" => {
                        sound.frequency_deviation = value
                            .parse()
                            .expect("frequency deviation should be an unsigned 32-bit integer");
                    }
                    "VOLUME" => {
                        sound.volume = value
                            .parse()
                            .expect("volume should be an unsigned 8-bit integer");
                    }
                    "LOOP" => {
                        let value: u8 = value
                            .parse()
                            .expect("looped should be an unsigned 8-bit integer");
                        sound.looped = value != 0;
                    }
                    "ATTACK" => {
                        sound.attack = value
                            .parse()
                            .expect("attack should be a signed 8-bit integer");
                    }
                    "RELEASE" => {
                        sound.release = value
                            .parse()
                            .expect("release should be a signed 8-bit integer");
                    }
                    _ => panic!(
                        "{}:{}: unexpected sound directive, found '{}' with value '{}'",
                        field_token.line, field_token.column, field_token.value, value,
                    ),
                }
            }
            Token::SoundDividerDirective { .. } => {
                break; // end of sound
            }
            Token::Define { .. } => {
                break; // end of sound
            }
            _ => {
                let (line, column) = tokens[i].line_column();
                panic!(
                    "{}:{} unexpected token {:?} while parsing sound",
                    line, column, tokens[i]
                );
            }
        }
    }

    (sound, i)
}

fn parse_sfx(tokens: &[Token]) -> (Sfx, usize) {
    let mut sfx = Sfx::default();

    assert!(tokens.len() >= 2);

    let mut i = 0;

    sfx.name = match &tokens[i] {
        Token::Identifier { value, .. } => value.to_string(),
        _ => {
            let (line, column) = tokens[i].line_column();
            panic!(
                "{}:{}: expected SFX name, found '{:?}'",
                line, column, tokens[i]
            );
        }
    };
    i += 1; // consume the name token

    sfx.id = match &tokens[i] {
        Token::Identifier { value: v, .. } => v
            .parse()
            .expect("SFX ID should be an unsigned 8-bit integer"),
        _ => {
            let (line, column) = tokens[i].line_column();
            panic!(
                "{}:{}: expected SFX ID, found '{:?}'",
                line, column, tokens[i]
            );
        }
    };
    i += 1; // consume the ID token

    while i < tokens.len() {
        match &tokens[i] {
            Token::Comment { .. } => {
                i += 1; // consume this token
            }
            Token::BeginDirective { line, column } => {
                i += 1; // consume this token

                let directive_tokens = &tokens[i..i + 2];
                if directive_tokens.len() < 2 {
                    panic!("{line}:{column}: expected directive field and value, found 'EOF'",);
                }
                let (field_token, value_token) = parse_directive(directive_tokens);
                let value = value_token.value;
                i += directive_tokens.len(); // consume the directive tokens

                match field_token.value.as_str() {
                    "NAME" => {
                        sfx.name = value.to_string();
                    }
                    "PRIORITY" => {
                        sfx.priority = value
                            .parse()
                            .expect("priority should be an unsigned 8-bit integer");
                    }
                    "TYPE" => {
                        let value: u8 = value.parse().expect("type should be an 8-bit integer");
                        sfx.typ = SfxType::try_from(value).expect("type should be valid");
                    }
                    "FLAGS" => {
                        sfx.flags = SfxFlags::from_bits(
                            value.parse().expect("flags should be an 8-bit integer"),
                        )
                        .expect("flags should be valid");
                    }
                    "SNDS" => {
                        sfx.sounds = Vec::with_capacity(
                            value.parse().expect("sound count should be an integer"),
                        );
                    }
                    _ => panic!(
                        "{}:{}: unexpected SFX directive, found '{}' with value '{}'",
                        field_token.line, field_token.column, field_token.value, value,
                    ),
                }
            }
            Token::SoundDividerDirective { .. } => {
                i += 1; // consume this token

                let (sound, pos) = parse_sound(&tokens[i..]);
                sfx.sounds.push(sound);
                i += pos; // consume the tokens the sound parsing consumed
            }
            Token::Define { .. } => {
                break; // end of sfx
            }
            _ => {
                let (line, column) = tokens[i].line_column();
                panic!(
                    "{}:{}: unexpected token {:?} while parsing SFX",
                    line, column, tokens[i],
                );
            }
        }
    }

    (sfx, i)
}

fn parse_packet(tokens: &[Token]) -> (HashMap<u8, Sfx>, usize) {
    let mut sfxs = HashMap::new();

    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Comment { .. } => {
                i += 1; // consume this token
            }
            Token::Define { .. } => {
                i += 1; // consume this token

                let (sfx, pos) = parse_sfx(&tokens[i..]);
                sfxs.insert(sfx.id, sfx);
                i += pos; // consume the tokens the SFX parsing consumed
            }
            _ => {
                let (line, column) = tokens[i].line_column();
                panic!(
                    "{}:{}: unexpected token {:?} while parsing packet",
                    line, column, tokens[i],
                );
            }
        }
    }

    (sfxs, i)
}

fn parse(tokens: &[Token]) -> Packet {
    let mut packet = Packet::default();

    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Comment { .. } => {
                i += 1; // consume this token
            }
            Token::BeginDirective { line, column } => {
                i += 1; // consume this token

                let directive_tokens = &tokens[i..i + 2];
                if directive_tokens.len() < 2 {
                    panic!("{line}:{column}: expected directive field and value, found 'EOF'",);
                }
                let (field_token, value_token) = parse_directive(directive_tokens);
                let value = value_token.value;
                i += directive_tokens.len(); // consume the directive tokens

                match field_token.value.as_str() {
                    "PACKET" => {
                        packet.name = value.to_string();
                        let (sfxs, pos) = parse_packet(&tokens[i..]);
                        packet.sfxs = sfxs;
                        i += pos; // consume the tokens the packet parsing consumed
                    }
                    _ => panic!(
                        "{}:{}: unexpected packet directive, found '{}' with value '{}'",
                        field_token.line, field_token.column, field_token.value, value,
                    ),
                }
            }
            _ => {
                let (line, column) = tokens[i].line_column();
                panic!("{}:{}: unexpected token {:?}", line, column, tokens[i]);
            }
        }
    }

    packet
}

#[derive(Debug)]
pub enum DecodeError {
    IoError(IoError),
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
            DecodeError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

#[derive(Debug)]
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

    pub fn decode(&mut self) -> Result<Packet, DecodeError> {
        let mut buffer = String::new();
        self.reader.read_to_string(&mut buffer)?;

        let tokens = lex(buffer.as_str());

        let script = parse(&tokens);

        Ok(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        ffi::{OsStr, OsString},
        fs::{self, File},
        path::{Path, PathBuf},
    };

    #[test]
    fn test_decode() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "SOUND",
            "H",
            "WATAFALL.H",
        ]
        .iter()
        .collect();

        let packet = Decoder::new(fs::File::open(d).unwrap()).decode().unwrap();

        assert_eq!(packet.name, "WaterFallingTears");
        assert_eq!(packet.sfxs.len(), 1);
        let index: u8 = 0;
        let sfx = packet.sfxs.get(&index);
        assert!(sfx.is_some(), "SFX is none");
        let sfx = sfx.unwrap();
        assert_eq!(sfx.sounds.len(), 1);
        assert_eq!(sfx.sounds[0].volume, 80);
        assert!(sfx.sounds[0].looped);
    }

    macro_rules! test_decode_error {
        ($name:ident, $file:expr, $expected:expr) => {
            #[test]
            #[should_panic(expected = $expected)]
            fn $name() {
                let file_as_string = $file;

                let _ = Decoder::new(std::io::Cursor::new(file_as_string.as_bytes().to_vec()))
                    .decode()
                    .unwrap();
            }
        };
    }

    test_decode_error!(test_decode_unexpected_char, r#":"#, "1:1: unexpected ':'");
    test_decode_error!(
        test_decode_eof_while_reading_comment,
        r#"/"#,
        "1:2: expected '/', found 'EOF'"
    );
    test_decode_error!(
        test_decode_partial_comment,
        r#"/ foo"#,
        "1:2: expected '/', found ' '"
    );
    test_decode_error!(
        test_decode_eof_while_reading_directive,
        r#"//#"#,
        "1:4: expected directive field, found 'EOF'"
    );
    test_decode_error!(
        test_decode_newline_while_reading_directive,
        r#"//#
"#,
        "2:1: expected directive field, found '\n'"
    );
    test_decode_error!(
        test_decode_eof_while_reading_sound_divider_directive,
        r#"//#-"#,
        "1:5: expected '-----', found 'EOF'"
    );
    test_decode_error!(
        test_decode_invalid_sound_divider_directive_whitespace,
        r#"//#- "#,
        "1:5: expected '-----', found ' '"
    );
    test_decode_error!(
        test_decode_invalid_sound_divider_directive_non_hyphen,
        r#"//#-_"#,
        "1:5: expected '-----', found '_'"
    );
    test_decode_error!(
        test_decode_invalid_sound_divider_directive_too_short,
        r#"//#----
"#,
        "1:8: expected '-----', found '\n'"
    );
    test_decode_error!(
        test_decode_invalid_sound_divider_too_long,
        r#"//#------"#,
        "1:9: expected '\n', found '-'"
    );
    test_decode_error!(
        test_decode_empty_directive_field,
        r#"//# :"#,
        "1:5: expected directive field, found ':'"
    );
    test_decode_error!(
        test_decode_eof_while_reading_sfx_define,
        r#"#"#,
        "1:2: expected 'define', found 'EOF'"
    );
    test_decode_error!(
        test_decode_invalid_sfx_define_char,
        r#"#:"#,
        "1:2: expected 'define', found ':'"
    );
    test_decode_error!(
        test_decode_newline_while_reading_sfx_define,
        r#"#
"#,
        "1:2: expected 'define', found '\n'"
    );
    test_decode_error!(
        test_decode_newline_while_reading_sfx_define_name,
        r#"#define
"#,
        "1:8: expected name, found '\n'"
    );
    test_decode_error!(
        test_decode_invalid_sfx_define_name,
        r#"#define :"#,
        "1:9: expected name, found ':'"
    );
    test_decode_error!(
        test_decode_newline_while_reading_sfx_define_id,
        r#"#define name
"#,
        "1:13: expected ID, found '\n'"
    );
    test_decode_error!(
        test_decode_invalid_sfx_define_id,
        r#"#define name :"#,
        "1:14: expected ID, found ':'"
    );
    test_decode_error!(
        test_decode_invalid_sfx_define_id_not_integer,
        r#"#define name foo"#,
        "1:14: expected ID, found 'f'"
    );
    test_decode_error!(
        test_decode_invalid_packet_directive_field,
        r#"//# FOO: bar"#,
        "1:5: unexpected packet directive, found 'FOO' with value 'bar'"
    );
    test_decode_error!(
        test_decode_invalid_sfx_directive_field,
        r#"//# PACKET: Foo
#define SFX_FOO           		 0
//# FOO: bar"#,
        "3:5: unexpected SFX directive, found 'FOO' with value 'bar'"
    );
    test_decode_error!(
        test_decode_invalid_sound_directive_field,
        r#"//# PACKET: Foo
#define SFX_FOO           		 0
//# NAME: Foo
//# PRIORITY: 200
//# TYPE: 1
//# FLAGS: 2
//# SNDS: 1
//#-----
//#     SAMPLE: foo
//#     FOO: bar"#,
        "10:9: unexpected sound directive, found 'FOO' with value 'bar'"
    );

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "SOUND",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "sound"]
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
            if ext.to_string_lossy().to_uppercase() != "H" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let file = File::open(path).unwrap();
            let packet = Decoder::new(file).decode().unwrap();

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
            let mut buffer = String::new();
            ron::ser::to_writer_pretty(&mut buffer, &packet, Default::default()).unwrap();
            std::fs::write(output_path, buffer).unwrap();
        });
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
