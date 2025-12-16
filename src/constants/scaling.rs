//! Scaling factors and unit conversions for physiological parameters

/// Scaling factor for percentage values (stored as 1/100 %)
pub const SCALE_PERCENT_100: f64 = 0.01;

/// Scaling factor for temperature (stored as 1/100 °C)
pub const SCALE_TEMP_100: f64 = 0.01;

/// Scaling factor for pressure (stored as 1/100 mmHg)
pub const SCALE_PRESSURE_100: f64 = 0.01;

/// Scaling factor for ST levels (stored as 1/100 mm)
pub const SCALE_ST_100: f64 = 0.01;

/// Scaling factor for flow (stored as 1/10 l/min)
pub const SCALE_FLOW_10: f64 = 0.1;

/// Scaling factor for volume (stored as 1/10 ml)
pub const SCALE_VOLUME_10: f64 = 0.1;

/// Scaling factor for compliance (stored as 1/100 ml/cmH2O)
pub const SCALE_COMPLIANCE_100: f64 = 0.01;

/// Scaling factor for MAC (stored as 1/100 %)
pub const SCALE_MAC_100: f64 = 0.01;

/// Scaling factor for airway pressure (stored as 1/100 cmH2O)
pub const SCALE_AWP_100: f64 = 0.01;

/// Scaling factor for SpO2 IR amplitude (stored as 1/10 %)
pub const SCALE_IR_AMP_10: f64 = 0.1;

/// Scaling factor for impedance (stored as 1/100 Ω)
pub const SCALE_IMPEDANCE_100: f64 = 0.01;

/// Apply scaling to an optional i16 value
pub fn scale_i16(value: Option<i16>, scale: f64) -> Option<f64> {
    value.map(|v| v as f64 * scale)
}

/// Apply scaling to i16 value, return None if invalid
pub fn scale_valid_i16(value: i16, scale: f64) -> Option<f64> {
    if crate::constants::special_values::is_invalid(value) {
        None
    } else {
        Some(value as f64 * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_i16() {
        assert_eq!(scale_i16(Some(9800), SCALE_PERCENT_100), Some(98.0));
        assert_eq!(scale_i16(Some(3700), SCALE_TEMP_100), Some(37.0));
        assert_eq!(scale_i16(None, SCALE_PERCENT_100), None);
    }

    #[test]
    fn test_scale_valid_i16() {
        assert_eq!(scale_valid_i16(9800, SCALE_PERCENT_100), Some(98.0));
        assert_eq!(scale_valid_i16(-32767, SCALE_PERCENT_100), None); // DATA_INVALID
    }
}
