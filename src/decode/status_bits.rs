//! Status bit structures for all parameter groups

use serde::{Deserialize, Serialize};

/// ECG status flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct EcgStatus {
    pub exists: bool,
    pub active: bool,
    pub asystole: bool,
    pub noise: bool,
    pub artifact: bool,
    pub learning: bool,
    pub pacer_on: bool,
    pub channel1_off: bool,
    pub channel2_off: bool,
    pub channel3_off: bool,
}

impl EcgStatus {
    pub fn from_status(status: u32) -> Self {
        Self {
            exists: (status & (1 << 0)) != 0,
            active: (status & (1 << 1)) != 0,
            asystole: (status & (1 << 2)) != 0,
            noise: (status & (1 << 7)) != 0,
            artifact: (status & (1 << 8)) != 0,
            learning: (status & (1 << 9)) != 0,
            pacer_on: (status & (1 << 10)) != 0,
            channel1_off: (status & (1 << 11)) != 0,
            channel2_off: (status & (1 << 12)) != 0,
            channel3_off: (status & (1 << 13)) != 0,
        }
    }
}

/// NIBP status flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct NibpStatus {
    pub exists: bool,
    pub active: bool,
    pub auto_mode: bool,
    pub stat_mode: bool,
    pub measuring: bool,
    pub stasis_on: bool,
    pub calibrating: bool,
    pub data_older_than_60s: bool,
}

impl NibpStatus {
    pub fn from_label(label: u16) -> Self {
        Self {
            exists: true, // Determined by group header
            active: true, // Determined by group header
            auto_mode: (label & (1 << 3)) != 0,
            stat_mode: (label & (1 << 4)) != 0,
            measuring: (label & (1 << 5)) != 0,
            stasis_on: (label & (1 << 6)) != 0,
            calibrating: (label & (1 << 7)) != 0,
            data_older_than_60s: (label & (1 << 8)) != 0,
        }
    }
}

/// CO2 status flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Co2Status {
    pub exists: bool,
    pub active: bool,
    pub apnea_co2: bool,
    pub calibrating_sensor: bool,
    pub zeroing_sensor: bool,
    pub occlusion: bool,
    pub air_leak: bool,
    pub apnea_from_resp: bool,
    pub apnea_deactivated: bool,
    pub wet_condition: bool,
}

impl Co2Status {
    pub fn from_status(status: u32) -> Self {
        Self {
            exists: (status & (1 << 0)) != 0,
            active: (status & (1 << 1)) != 0,
            apnea_co2: (status & (1 << 2)) != 0,
            calibrating_sensor: (status & (1 << 3)) != 0,
            zeroing_sensor: (status & (1 << 4)) != 0,
            occlusion: (status & (1 << 5)) != 0,
            air_leak: (status & (1 << 6)) != 0,
            apnea_from_resp: (status & (1 << 7)) != 0,
            apnea_deactivated: (status & (1 << 8)) != 0,
            wet_condition: (status & (1 << 9)) != 0,
        }
    }
}

/// SpO2 status flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Spo2Status {
    pub exists: bool,
    pub active: bool,
}

impl Spo2Status {
    pub fn from_status(status: u32) -> Self {
        Self {
            exists: (status & (1 << 0)) != 0,
            active: (status & (1 << 1)) != 0,
        }
    }
}

/// Flow & Volume status flags (Ventilator)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct FlowVolStatus {
    pub exists: bool,
    pub active: bool,
    pub disconnection: bool,
    pub calibrating: bool,
    pub zeroing: bool,
    pub obstruction: bool,
    pub leak: bool,
    pub measurement_off: bool,
    pub tv_base: TidalVolumeBase,
}

/// Tidal volume base (temperature and pressure correction)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum TidalVolumeBase {
    #[default]
    Atpd = 0, // Ambient temperature and pressure, dry
    Ntpd = 1, // Normal temperature and pressure, dry
    Btps = 2, // Body temperature and pressure, saturated
    Stpd = 3, // Standard temperature and pressure, dry
}

impl FlowVolStatus {
    pub fn from_status(status: u32) -> Self {
        let tv_base_bits = ((status >> 8) & 0x03) as u8;
        let tv_base = match tv_base_bits {
            0 => TidalVolumeBase::Atpd,
            1 => TidalVolumeBase::Ntpd,
            2 => TidalVolumeBase::Btps,
            3 => TidalVolumeBase::Stpd,
            _ => TidalVolumeBase::Atpd,
        };

        Self {
            exists: (status & (1 << 0)) != 0,
            active: (status & (1 << 1)) != 0,
            disconnection: (status & (1 << 2)) != 0,
            calibrating: (status & (1 << 3)) != 0,
            zeroing: (status & (1 << 4)) != 0,
            obstruction: (status & (1 << 5)) != 0,
            leak: (status & (1 << 6)) != 0,
            measurement_off: (status & (1 << 7)) != 0,
            tv_base,
        }
    }
}

/// O2/N2O/AA Gas status flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct GasStatus {
    pub exists: bool,
    pub active: bool,
    pub calibrating: bool,
    pub measurement_off: bool,
}

impl GasStatus {
    pub fn from_status(status: u32) -> Self {
        Self {
            exists: (status & (1 << 0)) != 0,
            active: (status & (1 << 1)) != 0,
            calibrating: (status & (1 << 2)) != 0,
            measurement_off: (status & (1 << 3)) != 0,
        }
    }
}

/// Generic status flags (for parameters with just exists/active)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct GenericStatus {
    pub exists: bool,
    pub active: bool,
}

impl GenericStatus {
    pub fn from_status(status: u32) -> Self {
        Self {
            exists: (status & (1 << 0)) != 0,
            active: (status & (1 << 1)) != 0,
        }
    }
}
