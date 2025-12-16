//! Subrecord parsing utilities

use crate::constants::special_values;

/// Read a 16-bit signed integer from data (little-endian)
pub fn read_i16(data: &[u8]) -> i16 {
    i16::from_le_bytes([data[0], data[1]])
}

/// Read a 32-bit unsigned integer from data (little-endian)
pub fn read_u32(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// Read a 16-bit unsigned integer from data (little-endian)
pub fn read_u16(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}

/// Read and validate an i16, returning None if invalid
pub fn read_valid_i16(data: &[u8]) -> Option<i16> {
    let value = read_i16(data);
    special_values::check_valid(value)
}

/// Group header structure (common to many parameter groups)
#[derive(Debug, Clone, Copy)]
pub struct GroupHeader {
    pub status: u32,
    pub label: u16,
}

impl GroupHeader {
    /// Parse group header from data (6 bytes: 4 bytes status + 2 bytes label)
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 6 {
            return Err(anyhow::anyhow!("Group header data too short"));
        }

        let status = read_u32(&data[0..4]);
        let label = read_u16(&data[4..6]);

        Ok(GroupHeader { status, label })
    }

    /// Check if module exists
    pub fn exists(&self) -> bool {
        (self.status & 0x01) != 0
    }

    /// Check if module is active
    pub fn active(&self) -> bool {
        (self.status & 0x02) != 0
    }

    /// Get specific status bit
    pub fn get_bit(&self, bit: u8) -> bool {
        if bit >= 32 {
            return false;
        }
        (self.status & (1 << bit)) != 0
    }

    /// Get bits in range
    pub fn get_bits(&self, start: u8, end: u8) -> u32 {
        if start >= 32 || end >= 32 || start > end {
            return 0;
        }
        let mask = ((1u64 << (end - start + 1)) - 1) as u32;
        (self.status >> start) & mask
    }
}

/// Extract bits from a u16 label field
pub fn extract_label_bits(label: u16, start: u8, end: u8) -> u16 {
    if start >= 16 || end >= 16 || start > end {
        return 0;
    }
    let mask = ((1u32 << (end - start + 1)) - 1) as u16;
    (label >> start) & mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_i16() {
        let data = vec![0x34, 0x12, 0xFF, 0xFF];
        assert_eq!(read_i16(&data[0..2]), 0x1234);
        assert_eq!(read_i16(&data[2..4]), -1);
    }

    #[test]
    fn test_group_header() {
        let header = GroupHeader {
            status: 0b11, // exists=1, active=1
            label: 0,
        };
        assert!(header.exists());
        assert!(header.active());
    }

    #[test]
    fn test_get_bits() {
        let header = GroupHeader {
            status: 0b11110000,
            label: 0,
        };
        assert_eq!(header.get_bits(4, 7), 0b1111);
    }
}
