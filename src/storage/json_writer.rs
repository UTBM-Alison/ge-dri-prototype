//! JSON file writer for DRI data

use crate::decode::physiological::PhysiologicalData;
use crate::decode::waveforms::WaveformData;
use anyhow::Result;
use serde_json;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub struct JsonWriter {
    file: std::fs::File,
}

impl JsonWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self { file })
    }

    /// Write physiological data as JSON line
    pub fn write_physiological(&mut self, data: &PhysiologicalData) -> Result<()> {
        let json = serde_json::to_string(data)?;
        writeln!(self.file, "{}", json)?;
        self.file.flush()?;
        Ok(())
    }

    /// Write waveform data as JSON line
    pub fn write_waveform(&mut self, data: &WaveformData) -> Result<()> {
        let json = serde_json::to_string(data)?;
        writeln!(self.file, "{}", json)?;
        self.file.flush()?;
        Ok(())
    }
}
