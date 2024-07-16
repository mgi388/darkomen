// TODO: Fix error messages in parser.
// TODO: Add error handling tests.

use super::{lexer::*, *};
use indexmap::IndexMap;
use std::{
    fmt,
    io::{Error as IoError, Read, Seek},
};

#[derive(Debug)]
struct ParseError {
    message: String,
    line_column: Option<(usize, usize)>,
}

impl ParseError {
    fn new(message: String, line_column: Option<(usize, usize)>) -> Self {
        Self {
            message,
            line_column,
        }
    }

    fn with_line_column(self, line_column: (usize, usize)) -> Self {
        Self {
            line_column: Some(line_column),
            ..self
        }
    }

    fn line(&self) -> usize {
        self.line_column.unwrap().0
    }

    fn column(&self) -> usize {
        self.line_column.unwrap().1
    }
}

fn parse_pattern(tokens: &[Token]) -> Result<(String, Pattern, usize), ParseError> {
    let mut tokens_iter = tokens.iter().enumerate().peekable();

    let mut i: usize;

    let id = match tokens_iter.next() {
        Some((_, Token::String { value, .. })) => value.to_string(),
        Some((_, token)) => {
            let line_column = token.line_column();
            return Err(ParseError::new(
                format!("expected string token for pattern ID, found '{:?}'", token),
                Some(line_column),
            ));
        }
        None => {
            return Err(ParseError::new(
                "expected a string after 'pattern', found 'EOF'".to_string(),
                None,
            ))
        }
    };

    match tokens_iter.next() {
        Some((index, Token::OpenBrace { .. })) => {
            i = index + 1; // consume the token
        }
        Some((_, token)) => {
            let line_column = token.line_column();
            return Err(ParseError::new(
                format!("expected '{{', found '{:?}'", token),
                Some(line_column),
            ));
        }
        None => {
            return Err(ParseError::new(
                "expected '{{', found 'EOF'".to_string(),
                None,
            ))
        }
    };

    let mut sequences = Vec::new();
    let mut state_tables = Vec::new();

    while let Token::Keyword { value: keyword, .. } = &tokens[i] {
        match keyword.as_str() {
            "sequence" => {
                i += 1; // consume the sequence keyword token

                match &tokens[i] {
                    Token::OpenBrace { .. } => {}
                    _ => {
                        let (line, column) = tokens[i].line_column();
                        panic!(
                            "{}:{}: expected '{{', found '{:?}'",
                            line, column, tokens[i]
                        );
                    }
                };
                i += 1; // consume the open brace token

                let mut sequence = Sequence(Vec::new());
                while let Token::String { value: seq, .. } = &tokens[i] {
                    i += 1; // consume the sequence string token
                    sequence.0.push(seq.clone());
                }
                sequences.push(sequence);

                match &tokens[i] {
                    Token::CloseBrace { .. } => {}
                    _ => {
                        let (line, column) = tokens[i].line_column();
                        panic!(
                            "{}:{}: expected '}}', found '{:?}'",
                            line, column, tokens[i]
                        );
                    }
                };
                i += 1; // consume the close brace token
            }
            "state-table" => {
                i += 1; // consume the state-table keyword token

                match &tokens[i] {
                    Token::OpenBrace { .. } => {}
                    _ => {
                        let (line, column) = tokens[i].line_column();
                        panic!(
                            "{}:{}: expected '{{', found '{:?}'",
                            line, column, tokens[i]
                        );
                    }
                };
                i += 1; // consume the open brace token

                let mut state_table = StateTable(IndexMap::new());
                while let Token::String { value: state, .. } = &tokens[i] {
                    i += 1; // consume the state string token

                    if let Token::String {
                        value: pattern_id, ..
                    } = &tokens[i]
                    {
                        i += 1; // consume the pattern ID string token
                        state_table
                            .0
                            .insert(state.clone(), PatternId::new(pattern_id.clone()));
                    } else {
                        let (line, column) = tokens[i + 1].line_column();
                        panic!(
                                "{}:{}: expected string token for pattern ID in state table, found '{:?}'",
                                line, column, tokens[i]
                            );
                    }
                }
                state_tables.push(state_table);

                match &tokens[i] {
                    Token::CloseBrace { .. } => {}
                    _ => {
                        let (line, column) = tokens[i].line_column();
                        panic!(
                            "{}:{}: expected '}}', found '{:?}'",
                            line, column, tokens[i]
                        );
                    }
                };
                i += 1; // consume the close brace token
            }
            _ => {
                let (line, column) = tokens[i].line_column();
                panic!(
                    "{}:{}: unexpected keyword in pattern: {}",
                    line, column, keyword,
                );
            }
        }
    }

    match &tokens[i] {
        Token::CloseBrace { .. } => {}
        _ => {
            let (line, column) = tokens[i].line_column();
            panic!(
                "{}:{}: expected '}}', found '{:?}'",
                line, column, tokens[i]
            );
        }
    };
    i += 1; // consume the close brace token

    Ok((
        id,
        Pattern {
            sequences,
            state_tables,
        },
        i,
    ))
}

fn parse(tokens: &[Token]) -> Script {
    let mut script = Script {
        states: IndexMap::new(),
        start_state: None,
        start_pattern: None,
        samples: IndexMap::new(),
        patterns: IndexMap::new(),
    };

    let mut i = 0;
    let mut context = "start of file";

    while i < tokens.len() {
        match &tokens[i] {
            Token::Keyword {
                value: keyword,
                line,
                column,
            } => {
                context = keyword;

                match keyword.as_str() {
                    "state" => {
                        if let Token::String { value: id, .. } = &tokens[i + 1] {
                            if let Token::Number { value: num, .. } = &tokens[i + 2] {
                                script.states.insert(id.clone(), *num);
                            }
                        }
                        i += 3;
                    }
                    "start-state" => {
                        if let Token::String { value: id, .. } = &tokens[i + 1] {
                            script.start_state = Some(id.clone());
                        }
                        i += 2;
                    }
                    "start-pattern" => {
                        if let Token::String { value: id, .. } = &tokens[i + 1] {
                            script.start_pattern = Some(PatternId::new(id.clone()));
                        }
                        i += 2;
                    }
                    "sample" => {
                        if let Token::String {
                            value: file_name, ..
                        } = &tokens[i + 1]
                        {
                            if let Token::Identifier { value: id, .. } = &tokens[i + 2] {
                                script.samples.insert(id.clone(), file_name.clone());
                                i += 3;
                            } else {
                                panic!(
                                    "expected identifier token for sample ID in context {}: {:?} at line {}, column {}",
                                    context, tokens[i + 2], line, column
                                );
                            }
                        } else {
                            panic!(
                                "expected string token for sample file name in context {}: {:?} at line {}, column {}",
                                context, tokens[i + 1], line, column
                            );
                        }
                    }
                    "pattern" => {
                        i += 1; // consume this token

                        let (id, pattern, pos) = match parse_pattern(&tokens[i..]) {
                            Ok(result) => result,
                            Err(mut err) => {
                                if err.line_column.is_none() {
                                    err = err.with_line_column((*line, *column));
                                }
                                panic!("{}:{}: {}", err.line(), err.column(), err.message)
                            }
                        };
                        script.patterns.insert(PatternId::new(id.clone()), pattern);
                        i += pos; // consume the tokens the pattern parsing consumed
                    }
                    _ => panic!(
                        "unexpected keyword in context {}: {} at line {}, column {}",
                        context, keyword, line, column
                    ),
                }
            }
            _ => {
                let (line, column) = tokens[i].line_column();
                panic!(
                    "unexpected token in context {}: {:?} at line {}, column {}",
                    context, tokens[i], line, column
                );
            }
        }
    }

    script
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
            DecodeError::IoError(e) => write!(f, "IO error: {}", e),
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

    pub fn decode(&mut self) -> Result<Script, DecodeError> {
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
            "SCRIPT",
            "EERIE9.FSM",
        ]
        .iter()
        .collect();

        let script = Decoder::new(fs::File::open(d).unwrap()).decode().unwrap();

        assert_eq!(script.states.get("sNormal"), Some(&303));
        assert_eq!(script.start_state, Some("sNormal".to_string()));
        assert_eq!(
            script.start_pattern,
            Some(PatternId::new("pEerie9".to_string()))
        );
        assert_eq!(script.samples.len(), 1);
        assert_eq!(script.samples.get("m9eerie"), Some(&"09eerie".to_string())); // samples should be keyed by ID not file name
        assert_eq!(script.patterns.len(), 1);
        assert_eq!(
            script
                .patterns
                .get(&PatternId::new("pEerie9".to_string()))
                .unwrap()
                .sequences
                .len(),
            1
        );
        assert_eq!(
            script
                .patterns
                .get(&PatternId::new("pEerie9".to_string()))
                .unwrap()
                .state_tables
                .len(),
            1
        );
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
        test_decode_eof_while_reading_sample_string,
        r#"sample"#,
        "1:7: expected a string after 'sample', found 'EOF'"
    );
    test_decode_error!(
        test_decode_newline_while_reading_sample_string,
        r#"sample
"#,
        "1:7: expected a string after 'sample', found '\n'"
    );
    test_decode_error!(
        test_decode_eof_while_reading_sample_identifier,
        r#"sample a"#,
        "1:9: expected an identifier after 'sample' string, found 'EOF'"
    );
    test_decode_error!(
        test_decode_newline_while_reading_sample_identifier,
        r#"sample a
"#,
        "1:9: expected an identifier after 'sample' string, found '\n'"
    );
    test_decode_error!(
        test_decode_eof_while_reading_pattern_id,
        r#"pattern"#,
        "1:8: expected a string after 'pattern', found 'EOF'"
    );

    macro_rules! round_trip_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                use pretty_assertions::assert_eq;
                use regex::Regex;
                use std::path::PathBuf;
                use std::fs;

                let file = $value;
                let d: PathBuf = [
                    std::env::var("DARKOMEN_PATH").unwrap().as_str(),
                    "DARKOMEN",
                    "SOUND",
                    "SCRIPT",
                    file,
                ]
                    .iter()
                    .collect();

                let original = fs::read_to_string(d.clone()).unwrap();
                let script = Decoder::new(fs::File::open(d.clone()).unwrap())
                    .decode()
                    .unwrap();

                let mut encoded = String::new();
                Encoder::new(&mut encoded).encode(&script).unwrap();

                fn normalize_file_contents(contents: &str) -> String {
                    let re_whitespace = Regex::new(r"\s+").unwrap();
                    let re_newlines = Regex::new(r"\n+").unwrap();

                    let normalized = contents
                        .lines()
                        .filter(|line| !line.trim_start().starts_with('#'))
                        .map(|line| {
                            let line = re_whitespace.replace_all(line.trim(), " ").to_string();
                            // BATTLE2.FSM has a typo in the script that needs
                            // to be fixed. This is the easiest way to still
                            // allow round trip tests even though it doesn't
                            // match the original file.
                            let line = line.replace("PBattleEnd", "pBattleEnd");
                            line
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .trim_start()
                        .to_string();

                    let mut normalized = re_newlines.replace_all(&normalized, "\n").to_string();

                    // Ensure there's always a trailing newline.
                    if !normalized.ends_with('\n') {
                        normalized = format!("{}\n", normalized);
                    }

                    normalized
                }

                let output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "sound", "SOUND", "SCRIPT"]
                    .iter()
                    .collect();

                std::fs::create_dir_all(&output_dir).unwrap();

                let file_stem = Path::new(&file).file_stem().unwrap().to_str().unwrap();
                let output_path = output_dir.join(format!("{}.encoded.fsm", file_stem));

                fs::write(output_path.clone(), encoded.clone()).unwrap();

                let original = normalize_file_contents(original.as_str());
                let encoded = normalize_file_contents(encoded.as_str());

                // Check that the encoded file is the same as the original.
                assert_eq!(original, encoded);

                // Check that the encoded file can be decoded back to the
                // original script and that it's the same.
                let decoded = Decoder::new(fs::File::open(output_path.clone()).unwrap())
                    .decode()
                    .unwrap();

                assert_eq!(script, decoded);

                let output_path = append_ext("ron", output_path);
                let mut output_file = File::create(output_path).unwrap();
                ron::ser::to_writer_pretty(&mut output_file, &script, Default::default())
                    .unwrap();
            }
        )*
        }
    }

    round_trip_tests! {
        test_battle1: "BATTLE1.FSM",
        test_battle2: "BATTLE2.FSM",
        test_eerie9: "EERIE9.FSM",
        test_eerie11: "EERIE11.FSM",
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [std::env::var("DARKOMEN_PATH").unwrap().as_str(), "DARKOMEN"]
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
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_uppercase() == "FSM" {
                    println!("Decoding {:?}", path.file_name().unwrap());

                    let file = File::open(path).unwrap();
                    let script = Decoder::new(file).decode().unwrap();

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
                    ron::ser::to_writer_pretty(&mut output_file, &script, Default::default())
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
