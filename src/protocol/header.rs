//! DRI record header parsing

use crate::DriError;
use crate::constants::{DriLevel, DriMainType, HEADER_SIZE, MAX_SUBRECORDS};
use chrono::{DateTime, Utc};
use log::debug;

/// DRI record header (40 bytes)
#[derive(Debug, Clone)]
pub struct DriHeader {
    /// Total length of record (including header)
    pub r_len: u16,
    /// Record number
    pub r_nbr: u8,
    /// DRI level the monitor supports
    pub dri_level: DriLevel,
    /// Plug identifier
    pub plug_id: u16,
    /// Timestamp (Unix time - seconds since 1970-01-01)
    pub r_time: u32,
    /// Main type of the record
    pub r_maintype: DriMainType,
    /// Subrecord descriptors (up to 8)
    pub subrecords: Vec<SubrecordDescriptor>,
}

/// Subrecord descriptor
#[derive(Debug, Clone, Copy)]
pub struct SubrecordDescriptor {
    /// Offset from start of data area
    pub offset: u16,
    /// Subrecord type
    pub sr_type: u8,
}

impl DriHeader {
    /// Parse a header from raw bytes
    ///
    /// The header is 40 bytes in little-endian format
    pub fn parse(data: &[u8]) -> Result<Self, DriError> {
        if data.len() < HEADER_SIZE {
            return Err(DriError::IncompleteFrame);
        }

        // Parse fields (all little-endian)
        let r_len = u16::from_le_bytes([data[0], data[1]]);
        let r_nbr = data[2];
        let dri_level_byte = data[3];
        let plug_id = u16::from_le_bytes([data[4], data[5]]);
        let r_time = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);

        // Reserved bytes at 10-15

        let r_maintype_raw = u16::from_le_bytes([data[16], data[17]]);

        // Parse DRI level
        let dri_level = DriLevel::from_u8(dri_level_byte)
            .ok_or(DriError::UnsupportedDriLevel(dri_level_byte))?;

        // Parse main type
        let r_maintype = DriMainType::from_u16(r_maintype_raw)
            .ok_or(DriError::InvalidSubrecordType(r_maintype_raw as u8))?;

        // Parse subrecord descriptors (8 descriptors, 3 bytes each)
        let mut subrecords = Vec::new();
        for i in 0..MAX_SUBRECORDS {
            let base = 18 + (i * 3);
            let offset = u16::from_le_bytes([data[base], data[base + 1]]);
            let sr_type = data[base + 2];

            // 0xFF marks end of subrecord list
            if sr_type == 0xFF {
                break;
            }

            subrecords.push(SubrecordDescriptor { offset, sr_type });
        }

        debug!(
            "Parsed header: len={}, type={:?}, level={:?}, subrecords={}",
            r_len,
            r_maintype,
            dri_level,
            subrecords.len()
        );

        Ok(DriHeader {
            r_len,
            r_nbr,
            dri_level,
            plug_id,
            r_time,
            r_maintype,
            subrecords,
        })
    }

    /// Get timestamp as DateTime
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.r_time as i64, 0).unwrap_or_else(|| Utc::now())
    }

    /// Get the data portion (everything after the header)
    pub fn extract_data<'a>(&self, frame_data: &'a [u8]) -> Result<&'a [u8], DriError> {
        if frame_data.len() < HEADER_SIZE {
            return Err(DriError::IncompleteFrame);
        }

        Ok(&frame_data[HEADER_SIZE..])
    }

    /// Get a specific subrecord's data
    pub fn get_subrecord_data<'a>(
        &self,
        data: &'a [u8],
        index: usize,
    ) -> Result<&'a [u8], DriError> {
        if index >= self.subrecords.len() {
            return Err(DriError::InvalidSubrecordType(index as u8));
        }

        let start = self.subrecords[index].offset as usize;

        // End is either the start of next subrecord or end of data
        let end = if index + 1 < self.subrecords.len() {
            self.subrecords[index + 1].offset as usize
        } else {
            data.len()
        };

        if start > data.len() || end > data.len() || start > end {
            return Err(DriError::IncompleteFrame);
        }

        Ok(&data[start..end])
    }
}

/// Create a request header for physiological data
pub fn create_phdb_request(subtype: u8, interval: u16, class_mask: u32) -> Vec<u8> {
    let mut header = vec![0u8; HEADER_SIZE];

    // r_len = header size + request data size (9 bytes)
    let r_len = (HEADER_SIZE + 9) as u16;
    header[0..2].copy_from_slice(&r_len.to_le_bytes());

    // r_nbr = 0
    header[2] = 0;

    // dri_level = 0 (ignored by monitor)
    header[3] = 0;

    // plug_id = 0
    header[4..6].copy_from_slice(&0u16.to_le_bytes());

    // r_time = 0 (ignored by monitor)
    header[6..10].copy_from_slice(&0u32.to_le_bytes());

    // reserved bytes 10-15 = 0

    // r_maintype = DRI_MT_PHDB (0)
    header[16..18].copy_from_slice(&0u16.to_le_bytes());

    // First subrecord: offset 0, type 0 (request)
    header[18..20].copy_from_slice(&0u16.to_le_bytes());
    header[20] = 0;

    // Second subrecord: end marker
    header[21..23].copy_from_slice(&0u16.to_le_bytes());
    header[23] = 0xFF;

    // Rest of subrecords: zeros
    // (already zeroed)

    // Now add the request data (9 bytes)
    let mut request_data = vec![0u8; 9];
    request_data[0] = subtype; // phdb_rcrd_type
    request_data[1..3].copy_from_slice(&interval.to_le_bytes()); // tx_interval
    request_data[3..7].copy_from_slice(&class_mask.to_le_bytes()); // phdb_class_bf
    // reserved[2] = 0

    header.extend_from_slice(&request_data);
    header
}

/// Create a request header for waveform data
pub fn create_waveform_request(waveform_types: &[u8], request_type: u16) -> Vec<u8> {
    let mut header = vec![0u8; HEADER_SIZE];

    // r_len = header size + waveform request data size (32 bytes)
    let r_len = (HEADER_SIZE + 32) as u16;
    header[0..2].copy_from_slice(&r_len.to_le_bytes());

    // r_nbr = 0
    header[2] = 0;

    // dri_level = 0
    header[3] = 0;

    // plug_id = 0
    header[4..6].copy_from_slice(&0u16.to_le_bytes());

    // r_time = 0
    header[6..10].copy_from_slice(&0u32.to_le_bytes());

    // r_maintype = DRI_MT_WAVE (1)
    header[16..18].copy_from_slice(&1u16.to_le_bytes());

    // First subrecord: offset 0, type 0 (command)
    header[18..20].copy_from_slice(&0u16.to_le_bytes());
    header[20] = 0;

    // Second subrecord: end marker
    header[21..23].copy_from_slice(&0u16.to_le_bytes());
    header[23] = 0xFF;

    // Waveform request data (32 bytes)
    let mut request_data = vec![0u8; 32];
    request_data[0..2].copy_from_slice(&request_type.to_le_bytes()); // req_type
    request_data[2..4].copy_from_slice(&0u16.to_le_bytes()); // reserved

    // Waveform types (8 bytes)
    for (i, &wf_type) in waveform_types.iter().enumerate().take(8) {
        request_data[4 + i] = wf_type;
    }
    // Mark end of list if less than 8
    if waveform_types.len() < 8 {
        request_data[4 + waveform_types.len()] = 0xFF;
    }

    // Reserved bytes (20 bytes) already zero

    header.extend_from_slice(&request_data);
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_header() {
        let mut data = vec![0u8; HEADER_SIZE];

        // r_len = 40
        data[0..2].copy_from_slice(&40u16.to_le_bytes());

        // dri_level = 8 (Level02)
        data[3] = 8;

        // r_maintype = 0 (PHDB)
        data[16..18].copy_from_slice(&0u16.to_le_bytes());

        // End marker for subrecords
        data[23] = 0xFF;

        let header = DriHeader::parse(&data).unwrap();
        assert_eq!(header.r_len, 40);
        assert_eq!(header.dri_level, DriLevel::Level02);
        assert_eq!(header.r_maintype, DriMainType::Phdb);
    }
}
