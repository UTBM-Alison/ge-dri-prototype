//! Checksum calculation and validation for DRI frames

/// Calculate checksum for a byte slice
///
/// The checksum is the sum of all bytes modulo 256
pub fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte))
}

/// Validate that data has correct checksum
///
/// The last byte should be the checksum of all preceding bytes
pub fn validate_checksum(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let (payload, checksum) = data.split_at(data.len() - 1);
    let expected = calculate_checksum(payload);

    checksum[0] == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let checksum = calculate_checksum(&data);
        assert_eq!(checksum, 0x0A); // 1+2+3+4 = 10
    }

    #[test]
    fn test_checksum_wrapping() {
        let data = vec![0xFF, 0xFF, 0xFF];
        let checksum = calculate_checksum(&data);
        assert_eq!(checksum, 0xFD); // Wraps around
    }

    #[test]
    fn test_validate_checksum_valid() {
        let mut data = vec![0x01, 0x02, 0x03, 0x04];
        let checksum = calculate_checksum(&data);
        data.push(checksum);

        assert!(validate_checksum(&data));
    }

    #[test]
    fn test_validate_checksum_invalid() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0xFF]; // Wrong checksum
        assert!(!validate_checksum(&data));
    }
}
