//! Raw binary data writer

use crate::Result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Writer for raw binary DRI frames
pub struct RawWriter {
    file: File,
    bytes_written: usize,
}

impl RawWriter {
    /// Create a new raw writer
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            file,
            bytes_written: 0,
        })
    }

    /// Write raw frame data
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.file.write_all(data)?;
        self.bytes_written += data.len();
        Ok(())
    }

    /// Flush buffered data to disk
    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        Ok(())
    }

    /// Get total bytes written
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl Drop for RawWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
