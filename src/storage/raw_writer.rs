//! Raw binary writer for DRI frames

use crate::protocol::DriFrame;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct RawWriter {
    file: File,
}

impl RawWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self { file })
    }

    /// Write a complete DRI frame to the raw file
    pub fn write_frame(&mut self, frame: &DriFrame) -> Result<()> {
        // Write frame start character
        self.file.write_all(&[0x7E])?;

        // Write the frame data
        self.file.write_all(&frame.data)?;

        // Write the checksum
        self.file.write_all(&[frame.checksum])?;

        // Write frame end character
        self.file.write_all(&[0x7E])?;

        self.file.flush()?;
        Ok(())
    }
}
