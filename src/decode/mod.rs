//! Data decoding module

pub mod physiological;
pub mod subrecords;
pub mod waveforms;

use crate::Result;
use crate::constants::DriMainType;
use crate::protocol::{DriFrame, DriHeader};
use log::{debug, warn};

pub use physiological::PhysiologicalData;
pub use waveforms::WaveformData;

/// Decoded DRI record
#[derive(Debug, Clone)]
pub enum DriRecord {
    /// Physiological data (trends, displayed values)
    Physiological(PhysiologicalData),
    /// Waveform data
    Waveform(WaveformData),
}

/// Main decoder for DRI frames
pub struct Decoder {
    /// Buffer for incomplete data
    _buffer: Vec<u8>,
}

impl Decoder {
    /// Create a new decoder
    pub fn new() -> Self {
        Self {
            _buffer: Vec::new(),
        }
    }

    /// Decode a frame into a record
    ///
    /// Returns:
    /// - Ok(Some(record)) if a complete record was decoded
    /// - Ok(None) if the frame was valid but no record could be extracted yet
    /// - Err if an error occurred
    pub fn decode_frame(&mut self, frame: &DriFrame) -> Result<Option<DriRecord>> {
        // Parse header
        let header = DriHeader::parse(&frame.data)?;

        // Extract data portion
        let data = header.extract_data(&frame.data)?;

        debug!(
            "Decoding frame: type={:?}, subrecords={}, data_len={}",
            header.r_maintype,
            header.subrecords.len(),
            data.len()
        );

        // Decode based on main type
        match header.r_maintype {
            DriMainType::Phdb => {
                let phys = physiological::decode_physiological(&header, data)?;
                Ok(Some(DriRecord::Physiological(phys)))
            }
            DriMainType::Wave => {
                let waves = waveforms::decode_waveforms(&header, data)?;
                // For now, return the first waveform
                // In the future, we might want to handle multiple waveforms differently
                if let Some(wave) = waves.into_iter().next() {
                    Ok(Some(DriRecord::Waveform(wave)))
                } else {
                    Ok(None)
                }
            }
            DriMainType::Alarm => {
                warn!("Alarm records not yet implemented");
                Ok(None)
            }
            DriMainType::Network => {
                warn!("Network management records not yet implemented");
                Ok(None)
            }
            DriMainType::Fo => {
                warn!("Anesthesia record keeping not yet implemented");
                Ok(None)
            }
        }
    }

    /// Reset the decoder state
    pub fn reset(&mut self) {
        self._buffer.clear();
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}
