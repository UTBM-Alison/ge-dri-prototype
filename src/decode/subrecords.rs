//! Subrecord parsing utilities

use crate::constants::special_values;

/// Read a 16-bit signed integer from data (little-endian)
pub fn read_i16(data: &[u8], offset: usize) -> Option<i16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(i16::from_le_bytes([data[offset], data[offset + 1]]))
}

/// Read a 32-bit unsigned integer from data (little-endian)
pub fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// Read a 16-bit unsigned integer from data (little-endian)
pub fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

/// Read and validate an i16, returning None if invalid
pub fn read_valid_i16(data: &[u8], offset: usize) -> Option<i16> {
    let value = read_i16(data, offset)?;
    special_values::check_valid(value)
}

/// Parse group header (6 bytes: 4 bytes status + 2 bytes label)
pub fn parse_group_header(data: &[u8], offset: usize) -> Option<GroupHeader> {
    if offset + 6 > data.len() {
        return None;
    }

    let status = read_u32(data, offset)?;
    let label = read_u16(data, offset + 4)?;

    Some(GroupHeader { status, label })
}

/// Group header structure (common to many parameter groups)
#[derive(Debug, Clone, Copy)]
pub struct GroupHeader {
    pub status: u32,
    pub label: u16,
}

impl GroupHeader {
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
        assert_eq!(read_i16(&data, 0), Some(0x1234));
        assert_eq!(read_i16(&data, 2), Some(-1));
        assert_eq!(read_i16(&data, 3), None); // Out of bounds
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
