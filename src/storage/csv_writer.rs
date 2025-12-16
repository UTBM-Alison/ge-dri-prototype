//! CSV data writer

use crate::Result;
use crate::decode::{PhysiologicalData, WaveformData};
use csv::Writer;
use std::fs::File;
use std::path::Path;

/// Writer for CSV-formatted data
pub struct CsvWriter {
    phys_writer: Option<Writer<File>>,
    wave_writer: Option<Writer<File>>,
    phys_path: String,
    wave_path: String,
    phys_count: usize,
    wave_count: usize,
}

impl CsvWriter {
    /// Create a new CSV writer
    ///
    /// This will create two files:
    /// - `<path>` for physiological data
    /// - `<path>.waveforms.csv` for waveform data
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let wave_path = if path_str.ends_with(".csv") {
            path_str.replace(".csv", ".waveforms.csv")
        } else {
            format!("{}.waveforms.csv", path_str)
        };

        Ok(Self {
            phys_writer: None,
            wave_writer: None,
            phys_path: path_str,
            wave_path,
            phys_count: 0,
            wave_count: 0,
        })
    }

    /// Write physiological data to CSV
    pub fn write_physiological(&mut self, data: &PhysiologicalData) -> Result<()> {
        // Lazy initialization of writer
        if self.phys_writer.is_none() {
            let file = File::create(&self.phys_path)?;
            let mut writer = Writer::from_writer(file);

            // Write header
            writer.write_record(&[
                "timestamp",
                "class",
                "subtype",
                "ecg_hr",
                "ecg_st1",
                "ecg_st2",
                "ecg_st3",
                "ecg_rr",
                "ecg_hr_source",
                "nibp_sys",
                "nibp_dia",
                "nibp_mean",
                "nibp_hr",
                "invp1_sys",
                "invp1_dia",
                "invp1_mean",
                "invp1_label",
                "spo2",
                "spo2_pr",
                "spo2_ir_amp",
                "temp1",
                "temp1_label",
                "temp2",
                "temp2_label",
                "co2_et",
                "co2_fi",
                "co2_rr",
                "o2_et",
                "o2_fi",
                "n2o_et",
                "n2o_fi",
                "aa_et",
                "aa_fi",
                "aa_mac",
                "aa_agent",
                "flow_rr",
                "flow_ppeak",
                "flow_peep",
                "flow_tv_insp",
                "flow_tv_exp",
                "flow_mv_exp",
            ])?;

            self.phys_writer = Some(writer);
        }

        let writer = self.phys_writer.as_mut().unwrap();

        // Write data row
        writer.write_record(&[
            data.timestamp.to_rfc3339(),
            format!("{:?}", data.class),
            format!("{:?}", data.subtype),
            format_option(data.ecg_hr),
            format_option(data.ecg_st1),
            format_option(data.ecg_st2),
            format_option(data.ecg_st3),
            format_option(data.ecg_rr),
            format_option_debug(&data.ecg_hr_source),
            format_option(data.nibp_sys),
            format_option(data.nibp_dia),
            format_option(data.nibp_mean),
            format_option(data.nibp_hr),
            format_option(data.invp1_sys),
            format_option(data.invp1_dia),
            format_option(data.invp1_mean),
            format_option_debug(&data.invp1_label),
            format_option(data.spo2),
            format_option(data.spo2_pr),
            format_option(data.spo2_ir_amp),
            format_option(data.temp1),
            format_option_debug(&data.temp1_label),
            format_option(data.temp2),
            format_option_debug(&data.temp2_label),
            format_option(data.co2_et),
            format_option(data.co2_fi),
            format_option(data.co2_rr),
            format_option(data.o2_et),
            format_option(data.o2_fi),
            format_option(data.n2o_et),
            format_option(data.n2o_fi),
            format_option(data.aa_et),
            format_option(data.aa_fi),
            format_option(data.aa_mac),
            format_option_debug(&data.aa_agent),
            format_option(data.flow_rr),
            format_option(data.flow_ppeak),
            format_option(data.flow_peep),
            format_option(data.flow_tv_insp),
            format_option(data.flow_tv_exp),
            format_option(data.flow_mv_exp),
        ])?;

        writer.flush()?;
        self.phys_count += 1;

        Ok(())
    }

    /// Write waveform data to CSV
    pub fn write_waveform(&mut self, data: &WaveformData) -> Result<()> {
        // Lazy initialization of writer
        if self.wave_writer.is_none() {
            let file = File::create(&self.wave_path)?;
            let mut writer = Writer::from_writer(file);

            // Write header
            writer.write_record(&[
                "timestamp",
                "waveform_type",
                "sample_rate",
                "sample_count",
                "gap",
                "pacer_detected",
                "lead_off",
                "samples",
            ])?;

            self.wave_writer = Some(writer);
        }

        let writer = self.wave_writer.as_mut().unwrap();

        // Format samples as JSON array for easier parsing
        let samples_json = serde_json::to_string(&data.samples)?;

        // Write data row
        writer.write_record(&[
            data.timestamp.to_rfc3339(),
            data.waveform_type.name().to_string(),
            data.sample_rate.to_string(),
            data.samples.len().to_string(),
            data.status.gap.to_string(),
            data.status.pacer_detected.to_string(),
            data.status.lead_off.to_string(),
            samples_json,
        ])?;

        writer.flush()?;
        self.wave_count += 1;

        Ok(())
    }

    /// Get number of physiological records written
    pub fn phys_count(&self) -> usize {
        self.phys_count
    }

    /// Get number of waveform records written
    pub fn wave_count(&self) -> usize {
        self.wave_count
    }

    /// Flush all writers
    pub fn flush(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.phys_writer {
            writer.flush()?;
        }
        if let Some(ref mut writer) = self.wave_writer {
            writer.flush()?;
        }
        Ok(())
    }
}

impl Drop for CsvWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

// Helper functions for formatting Option values

fn format_option<T: std::fmt::Display>(opt: Option<T>) -> String {
    match opt {
        Some(val) => val.to_string(),
        None => String::new(),
    }
}

fn format_option_debug<T: std::fmt::Debug>(opt: &Option<T>) -> String {
    match opt {
        Some(val) => format!("{:?}", val),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_option() {
        assert_eq!(format_option(Some(42)), "42");
        assert_eq!(format_option::<i32>(None), "");
    }
}
