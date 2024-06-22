use std::io::{self, Read, Take};

enum ZeroRunsReaderState {
    Header,
    Literal,
    Repeat,
}

/// Reader that unpacks ZeroRuns format.
pub(crate) struct ZeroRunsReader<R: Read> {
    reader: Take<R>,
    state: ZeroRunsReaderState,
    count: usize,
}

impl<R: Read> ZeroRunsReader<R> {
    /// Wraps a reader.
    pub fn new(reader: R, length: u64) -> Self {
        Self {
            reader: reader.take(length),
            state: ZeroRunsReaderState::Header,
            count: 0,
        }
    }
}

impl<R: Read> Read for ZeroRunsReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while let ZeroRunsReaderState::Header = self.state {
            if self.reader.limit() == 0 {
                return Ok(0);
            }
            let mut header: [u8; 1] = [0];
            self.reader.read_exact(&mut header)?;
            let h = header[0] as i8;
            if h >= 0 {
                self.state = ZeroRunsReaderState::Literal;
                self.count = h as usize + 1;
            } else {
                self.state = ZeroRunsReaderState::Repeat;
                self.count = (-h) as usize;
            }
        }

        let length = buf.len().min(self.count);
        let actual = match self.state {
            ZeroRunsReaderState::Literal => self.reader.read(&mut buf[..length])?,
            ZeroRunsReaderState::Repeat => {
                for b in &mut buf[..length] {
                    *b = 0;
                }
                length
            }
            ZeroRunsReaderState::Header => unreachable!(),
        };

        self.count -= actual;
        if self.count == 0 {
            self.state = ZeroRunsReaderState::Header;
        }
        Ok(actual)
    }
}
