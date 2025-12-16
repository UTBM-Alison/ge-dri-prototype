//! DRI protocol type definitions

use serde::{Deserialize, Serialize};

/// DRI Interface Level - indicates protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DriLevel {
    Level95 = 2,
    Level97 = 3,
    Level98 = 4,
    Level99 = 5,
    Level00 = 6,  // 2001
    Level01 = 7,  // 2002
    Level02 = 8,  // 2003
    Level03 = 9,  // 2005
    Level04 = 10, // 2009
}

impl DriLevel {
    /// Convert from raw byte value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            2 => Some(DriLevel::Level95),
            3 => Some(DriLevel::Level97),
            4 => Some(DriLevel::Level98),
            5 => Some(DriLevel::Level99),
            6 => Some(DriLevel::Level00),
            7 => Some(DriLevel::Level01),
            8 => Some(DriLevel::Level02),
            9 => Some(DriLevel::Level03),
            10 => Some(DriLevel::Level04),
            _ => None,
        }
    }

    /// Get the year string for display
    pub fn year_str(&self) -> &'static str {
        match self {
            DriLevel::Level95 => "'95",
            DriLevel::Level97 => "'97",
            DriLevel::Level98 => "'98",
            DriLevel::Level99 => "'99",
            DriLevel::Level00 => "'01",
            DriLevel::Level01 => "'02",
            DriLevel::Level02 => "'03",
            DriLevel::Level03 => "'05",
            DriLevel::Level04 => "'09",
        }
    }
}

/// Main record types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum DriMainType {
    /// Physiological data and related transmission requests
    Phdb = 0,
    /// Waveform data and related transmission requests
    Wave = 1,
    /// Alarm data (network interface only)
    Alarm = 4,
    /// Network management data (network interface only)
    Network = 5,
    /// Anesthesia record keeping event data (network interface only)
    Fo = 8,
}

impl DriMainType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(DriMainType::Phdb),
            1 => Some(DriMainType::Wave),
            4 => Some(DriMainType::Alarm),
            5 => Some(DriMainType::Network),
            8 => Some(DriMainType::Fo),
            _ => None,
        }
    }
}

/// Physiological database subrecord types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PhdbSubrecordType {
    /// Transmission request
    XmitReq = 0,
    /// Displayed values
    Displ = 1,
    /// 10 second trended values
    Trend10s = 2,
    /// 60 second trended values
    Trend60s = 3,
    /// Auxiliary data
    Aux = 4,
}

impl PhdbSubrecordType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PhdbSubrecordType::XmitReq),
            1 => Some(PhdbSubrecordType::Displ),
            2 => Some(PhdbSubrecordType::Trend10s),
            3 => Some(PhdbSubrecordType::Trend60s),
            4 => Some(PhdbSubrecordType::Aux),
            _ => None,
        }
    }
}

/// Physiological data record classes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PhdbClass {
    /// Basic physiological data
    Basic = 0,
    /// Extended class 1 (Arrhythmia, 12-lead ECG)
    Ext1 = 1,
    /// Extended class 2 (NMT2, EEG, Entropy, BIS)
    Ext2 = 2,
    /// Extended class 3 (Gas exchange, more spirometry, tonometry)
    Ext3 = 3,
}

impl PhdbClass {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PhdbClass::Basic),
            1 => Some(PhdbClass::Ext1),
            2 => Some(PhdbClass::Ext2),
            3 => Some(PhdbClass::Ext3),
            _ => None,
        }
    }
}

/// Bit masks for requesting physiological data classes
pub const PHDBCL_REQ_BASIC_MASK: u32 = 0x0000;
pub const PHDBCL_DENY_BASIC_MASK: u32 = 0x0001;
pub const PHDBCL_REQ_EXT1_MASK: u32 = 0x0002;
pub const PHDBCL_REQ_EXT2_MASK: u32 = 0x0004;
pub const PHDBCL_REQ_EXT3_MASK: u32 = 0x0008;

/// Request all physiological data classes
pub const PHDBCL_REQ_ALL: u32 =
    PHDBCL_REQ_BASIC_MASK | PHDBCL_REQ_EXT1_MASK | PHDBCL_REQ_EXT2_MASK | PHDBCL_REQ_EXT3_MASK;
