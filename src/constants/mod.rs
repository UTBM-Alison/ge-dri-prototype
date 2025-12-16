//! Constants and type definitions for the DRI protocol

pub mod dri_types;
pub mod physiological;
pub mod special_values;
pub mod waveforms;

// Re-export commonly used types
pub use dri_types::{DriLevel, DriMainType, PhdbClass, PhdbSubrecordType};
pub use physiological::{EcgLeadType, InvasivePressureLabel, ParameterGroup};
pub use special_values::SpecialValue;
pub use waveforms::{WaveformInfo, WaveformType};

/// Frame character for DRI protocol
pub const FRAME_CHAR: u8 = 0x7E;

/// Control character for byte stuffing
pub const CTRL_CHAR: u8 = 0x7D;

/// Bit mask for byte stuffing
pub const BIT5: u8 = 0x20;

/// Complement of BIT5
pub const BIT5_COMPL: u8 = 0x5F;

/// Maximum DRI record size (header + data)
pub const MAX_RECORD_SIZE: usize = 1490;

/// DRI header size
pub const HEADER_SIZE: usize = 40;

/// Maximum data size
pub const MAX_DATA_SIZE: usize = 1450;

/// Maximum number of subrecords per record
pub const MAX_SUBRECORDS: usize = 8;

/// End of subrecord list marker
pub const EOL_SUBRECORD_LIST: u8 = 0xFF;
