//! Special data values indicating invalid or out-of-range measurements

/// Represents special measurement values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialValue {
    /// No valid data available
    Invalid,
    /// Data has not been updated
    NotUpdated,
    /// Data exceeds lower valid limit
    UnderRange,
    /// Data exceeds upper valid limit
    OverRange,
    /// Data has not been calibrated
    NotCalibrated,
}

/// Limit for special invalid data values
pub const DATA_INVALID_LIMIT: i16 = -32001;

/// No valid data
pub const DATA_INVALID: i16 = -32767;

/// Data not updated
pub const DATA_NOT_UPDATED: i16 = -32766;

/// Data discontinuity
pub const DATA_DISCONT: i16 = -32765;

/// Data under range
pub const DATA_UNDER_RANGE: i16 = -32764;

/// Data over range
pub const DATA_OVER_RANGE: i16 = -32763;

/// Data not calibrated
pub const DATA_NOT_CALIBRATED: i16 = -32762;

/// Check if a value represents invalid data
pub fn is_invalid(value: i16) -> bool {
    value <= DATA_INVALID_LIMIT
}

/// Convert raw i16 to Option, None if invalid
pub fn check_valid(value: i16) -> Option<i16> {
    if is_invalid(value) { None } else { Some(value) }
}

/// Get the special value type if the value is invalid
pub fn get_special_value(value: i16) -> Option<SpecialValue> {
    match value {
        DATA_INVALID => Some(SpecialValue::Invalid),
        DATA_NOT_UPDATED => Some(SpecialValue::NotUpdated),
        DATA_UNDER_RANGE => Some(SpecialValue::UnderRange),
        DATA_OVER_RANGE => Some(SpecialValue::OverRange),
        DATA_NOT_CALIBRATED => Some(SpecialValue::NotCalibrated),
        _ if value > DATA_INVALID_LIMIT => None,
        _ => Some(SpecialValue::Invalid),
    }
}
