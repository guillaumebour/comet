// Taken from https://docs.rs/incremental-writer/latest/incremental_writer/
// Removed println!
use serde::Serialize;
use std::io::{Seek, SeekFrom, Write};

#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct IncrementalJsonWriter<T: FileExt + Write + Seek> {
    buffer: T,
}

impl<T: FileExt + Write + Seek> IncrementalJsonWriter<T> {
    pub fn new(buffer: T) -> Self {
        IncrementalJsonWriter::<T> { buffer }
    }

    pub fn write_json<U: Serialize>(&mut self, element: &U) -> Result<usize> {
        self.write(serde_json::to_string_pretty(&element)?.as_bytes())
    }

    #[cfg(unix)]
    fn write_at_offset(&mut self, bytes: &[u8], offset: u64) -> Result<usize> {
        let bytes_written = self.buffer.write_at(bytes, offset)?;
        self.buffer
            .seek(SeekFrom::Current((bytes_written - 2) as i64))
            .map(|_| bytes_written)
    }

    #[cfg(windows)]
    fn write_at_offset(&mut self, bytes: &[u8], offset: u64) -> Result<usize> {
        self.buffer.seek_write(bytes, offset)
    }
}

impl<T: FileExt + Write + Seek> Write for IncrementalJsonWriter<T> {
    fn write(&mut self, element: &[u8]) -> Result<usize> {
        let mut current = self.buffer.seek(SeekFrom::Current(0))?;
        let mut bytes = vec![];

        if current == 0 {
            self.buffer.write(b"[\n\n]")?;
            current = self.buffer.seek(SeekFrom::Current(0))?;
        } else {
            bytes.extend(b",\n");
        }

        bytes.extend(element);
        bytes.push(b'\n');
        bytes.push(b']');

        let written = self.write_at_offset(&bytes, current - 2)?;
        Ok(written)
    }
    fn flush(&mut self) -> Result<()> {
        self.buffer.flush()
    }
}
