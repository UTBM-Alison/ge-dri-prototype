//! JSON data writer

use crate::Result;
use crate::decode::{PhysiologicalData, WaveformData};
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Writer for JSON-formatted data
pub struct JsonWriter {
    file: File,
    record_count: usize,
}

impl JsonWriter {
    /// Create a new JSON writer
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            file,
            record_count: 0,
        })
    }

    /// Write physiological data as JSON
    pub fn write_physiological(&mut self, data: &PhysiologicalData) -> Result<()> {
        let json = serde_json::to_string(data)?;
        writeln!(self.file, "{}", json)?;
        self.record_count += 1;
        Ok(())
    }

    /// Write waveform data as JSON
    pub fn write_waveform(&mut self, data: &WaveformData) -> Result<()> {
        let json = serde_json::to_string(data)?;
        writeln!(self.file, "{}", json)?;
        self.record_count += 1;
        Ok(())
    }

    /// Flush buffered data to disk
    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        Ok(())
    }

    /// Get number of records written
    pub fn record_count(&self) -> usize {
        self.record_count
    }
}

impl Drop for JsonWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
