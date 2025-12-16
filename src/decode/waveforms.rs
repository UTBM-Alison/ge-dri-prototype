//! Waveform data decoding

use crate::constants::WaveformType;
use crate::protocol::DriHeader;
use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use super::subrecords::*;

/// Waveform data record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Waveform type
    pub waveform_type: WaveformType,
    /// Sample values
    pub samples: Vec<i16>,
    /// Sample rate (samples per second)
    pub sample_rate: u16,
    /// Status flags
    pub status: WaveformStatus,
}

/// Waveform status flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WaveformStatus {
    /// Gap in sampling (data lost)
    pub gap: bool,
    /// Pacer detected (ECG only)
    pub pacer_detected: bool,
    /// Lead off (ECG only)
    pub lead_off: bool,
}

impl WaveformStatus {
    /// Parse from status word
    pub fn from_u16(status: u16) -> Self {
        Self {
            gap: (status & 0x0001) != 0,
            pacer_detected: (status & 0x0004) != 0,
            lead_off: (status & 0x0008) != 0,
        }
    }
}

/// Waveform subrecord header (6 bytes)
struct WaveformHeader {
    act_len: u16,
    status: u16,
    _reserved: u16,
}

impl WaveformHeader {
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }

        let act_len = read_u16(&data[0..2]);
        let status = read_u16(&data[2..4]);
        let _reserved = read_u16(&data[4..6]);

        Some(Self {
            act_len,
            status,
            _reserved,
        })
    }
}

/// Decode waveform data from a frame
pub fn decode_waveforms(header: &DriHeader, data: &[u8]) -> Result<Vec<WaveformData>> {
    let mut waveforms = Vec::new();
    let timestamp = header.timestamp();

    // Iterate through subrecords
    for (i, subrecord) in header.subrecords.iter().enumerate() {
        // Get waveform type
        let waveform_type = match WaveformType::from_u8(subrecord.sr_type) {
            Some(wf) => wf,
            None => {
                warn!("Unknown waveform type: {}", subrecord.sr_type);
                continue;
            }
        };

        // Skip command subrecord
        if waveform_type == WaveformType::Cmd {
            continue;
        }

        // Get subrecord data
        let sub_data = match header.get_subrecord_data(data, i) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to get subrecord data: {}", e);
                continue;
            }
        };

        // Parse waveform header (first 6 bytes)
        let wf_header = match WaveformHeader::parse(sub_data) {
            Some(h) => h,
            None => {
                warn!("Failed to parse waveform header");
                continue;
            }
        };

        // Parse samples (after 6-byte header)
        let sample_count = wf_header.act_len as usize;
        let mut samples = Vec::with_capacity(sample_count);

        for sample_idx in 0..sample_count {
            let offset = 6 + (sample_idx * 2); // Each sample is 2 bytes
            if offset + 2 <= sub_data.len() {
                let sample = read_i16(&sub_data[offset..offset + 2]);
                samples.push(sample);
            } else {
                warn!(
                    "Failed to read sample {} for {:?} (offset {} exceeds data length {})",
                    sample_idx,
                    waveform_type,
                    offset,
                    sub_data.len()
                );
                break;
            }
        }

        let sample_rate = waveform_type.info().samples_per_second;
        let status = WaveformStatus::from_u16(wf_header.status);

        debug!(
            "Decoded waveform: type={:?}, samples={}, rate={}, gap={}",
            waveform_type,
            samples.len(),
            sample_rate,
            status.gap
        );

        waveforms.push(WaveformData {
            timestamp,
            waveform_type,
            samples,
            sample_rate,
            status,
        });
    }

    Ok(waveforms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_status() {
        let status = WaveformStatus::from_u16(0x0001);
        assert!(status.gap);
        assert!(!status.pacer_detected);

        let status = WaveformStatus::from_u16(0x0004);
        assert!(!status.gap);
        assert!(status.pacer_detected);
    }
}
