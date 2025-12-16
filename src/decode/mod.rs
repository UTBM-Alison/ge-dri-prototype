//! Data decoding module

pub mod physiological;
pub mod status_bits;
pub mod subrecords;
pub mod waveforms;

// Re-export main types for convenience
pub use physiological::PhysiologicalData;
pub use waveforms::WaveformData;

use crate::constants::dri_types::{DriMainType, PhdbClass, PhdbSubrecordType};
use crate::protocol::DriHeader;
use anyhow::{Result, anyhow};
use log::debug;
use serde::{Deserialize, Serialize};

/// Decoded DRI record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DriRecord {
    /// Physiological data record
    Physiological(PhysiologicalData),
    /// Waveform data record
    Waveform { waveforms: Vec<WaveformData> },
}

/// Main decoder
pub struct Decoder;

impl Decoder {
    /// Create a new decoder
    pub fn new() -> Self {
        Self
    }

    /// Decode a DRI frame
    pub fn decode_frame(&self, header: &DriHeader, data: &[u8]) -> Result<Option<DriRecord>> {
        match header.r_maintype {
            DriMainType::Phdb => {
                // Get the first subrecord to determine type and class
                if header.subrecords.is_empty() {
                    return Err(anyhow!("No subrecords in physiological data frame"));
                }

                // Get subrecord type from first subrecord
                let subtype =
                    PhdbSubrecordType::from_u8(header.subrecords[0].sr_type).ok_or_else(|| {
                        anyhow!("Invalid subrecord type: {}", header.subrecords[0].sr_type)
                    })?;

                // Get subrecord data
                let sub_data = header.get_subrecord_data(data, 0)?;

                // Determine class from the last word of the subrecord (offset 1086-1087 in 1088-byte subrecord)
                // Bits 8-11 contain the class
                if sub_data.len() < 1088 {
                    return Err(anyhow!("Physiological subrecord too short"));
                }

                let cl_drilvl_subt = u16::from_le_bytes([sub_data[1086], sub_data[1087]]);
                let class_bits = ((cl_drilvl_subt >> 8) & 0x0F) as u8;
                let class = PhdbClass::from_u8(class_bits)
                    .ok_or_else(|| anyhow!("Invalid class: {}", class_bits))?;

                debug!(
                    "Decoding physiological data: subtype={:?}, class={:?}",
                    subtype, class
                );

                let phys = physiological::decode_physiological(sub_data, subtype, class)?;
                Ok(Some(DriRecord::Physiological(phys)))
            }
            DriMainType::Wave => {
                let waveforms = waveforms::decode_waveforms(header, data)?;
                if waveforms.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(DriRecord::Waveform { waveforms }))
                }
            }
            DriMainType::Alarm => {
                debug!("Alarm records not yet implemented");
                Ok(None)
            }
            DriMainType::Network => {
                debug!("Network management records not yet implemented");
                Ok(None)
            }
            DriMainType::Fo => {
                debug!("Event records not yet implemented");
                Ok(None)
            }
        }
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}
