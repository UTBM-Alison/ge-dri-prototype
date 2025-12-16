//! GE Healthcare Datex-Ohmeda Record (DRI) Protocol Parser
//!
//! This library provides functionality to parse and decode data from
//! GE Healthcare patient monitors (S/5, CARESCAPE B650/B850) using
//! the Datex-Ohmeda Record Interface protocol.

pub mod constants;
pub mod decode;
pub mod device;
pub mod protocol;
pub mod storage;
pub mod ui;

// Re-export commonly used types
pub use constants::{DriLevel, DriMainType, SpecialValue};
pub use decode::{PhysiologicalData, WaveformData};
pub use device::SerialDevice;
pub use protocol::{DriFrame, DriHeader};

/// Result type alias for this crate
pub type Result<T> = anyhow::Result<T>;

/// Error types specific to DRI protocol
#[derive(Debug, thiserror::Error)]
pub enum DriError {
    #[error("Invalid frame: checksum mismatch")]
    ChecksumError,

    #[error("Invalid frame: incomplete data")]
    IncompleteFrame,

    #[error("Invalid frame: bad framing")]
    FramingError,

    #[error("Unsupported DRI level: {0}")]
    UnsupportedDriLevel(u8),

    #[error("Invalid subrecord type: {0}")]
    InvalidSubrecordType(u8),

    #[error("Serial port error: {0}")]
    SerialError(#[from] serialport::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
