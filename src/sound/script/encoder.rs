use super::*;
use std::fmt::Write;

#[derive(Debug)]
pub enum EncodeError {
    FmtError(std::fmt::Error),
}

impl std::error::Error for EncodeError {}

impl From<std::fmt::Error> for EncodeError {
    fn from(err: std::fmt::Error) -> Self {
        EncodeError::FmtError(err)
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::FmtError(e) => write!(f, "fmt error: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct Encoder<W: Write> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder { writer }
    }

    pub fn encode(&mut self, script: &Script) -> Result<(), EncodeError> {
        let mut wrote_newline = false;

        for (id, num) in &script.states {
            writeln!(self.writer, "state {} {}", id, num).unwrap();
            wrote_newline = true;
        }

        if let Some(start_state) = &script.start_state {
            writeln!(self.writer, "start-state {}", start_state).unwrap();
            wrote_newline = true;
        }

        if let Some(start_pattern) = &script.start_pattern {
            writeln!(self.writer, "start-pattern {}", start_pattern.as_str()).unwrap();
            wrote_newline = true;
        }

        for (id, file_name) in &script.samples {
            writeln!(self.writer, "sample {} {}", file_name, id).unwrap();
            wrote_newline = true;
        }

        for (id, pattern) in &script.patterns {
            writeln!(self.writer, "pattern {}", id.as_str()).unwrap();
            writeln!(self.writer, "{{").unwrap();

            for sequence in &pattern.sequences {
                writeln!(self.writer, "\tsequence").unwrap();
                writeln!(self.writer, "\t{{").unwrap();
                for seq in sequence.0.iter() {
                    writeln!(self.writer, "\t\t{}", seq).unwrap();
                }
                writeln!(self.writer, "\t}}").unwrap();
            }

            for state_table in &pattern.state_tables {
                writeln!(self.writer, "\tstate-table").unwrap();
                writeln!(self.writer, "\t{{").unwrap();

                let max_state_length = state_table
                    .0
                    .iter()
                    .map(|(state, _)| state.len())
                    .max()
                    .unwrap_or(0);

                for (state, pattern_id) in state_table.0.iter() {
                    let state_str = format!("{:<width$}", state, width = max_state_length);
                    writeln!(self.writer, "\t\t{}\t{}", state_str, pattern_id.as_str()).unwrap();
                }
                writeln!(self.writer, "\t}}").unwrap();
            }

            writeln!(self.writer, "}}").unwrap();

            self.writer.write_char('\n')?;
            wrote_newline = true;
        }

        // Ensure there's always a trailing newline.
        if !wrote_newline {
            self.writer.write_char('\n')?;
        }

        Ok(())
    }
}
