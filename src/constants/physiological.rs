//! Physiological parameter definitions

use serde::{Deserialize, Serialize};

/// ECG lead types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EcgLeadType {
    NotSelected = 0,
    I = 1,
    II = 2,
    III = 3,
    Avr = 4,
    Avl = 5,
    Avf = 6,
    V = 7,
}

impl EcgLeadType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(EcgLeadType::NotSelected),
            1 => Some(EcgLeadType::I),
            2 => Some(EcgLeadType::II),
            3 => Some(EcgLeadType::III),
            4 => Some(EcgLeadType::Avr),
            5 => Some(EcgLeadType::Avl),
            6 => Some(EcgLeadType::Avf),
            7 => Some(EcgLeadType::V),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EcgLeadType::NotSelected => "NOT_SELECTED",
            EcgLeadType::I => "I",
            EcgLeadType::II => "II",
            EcgLeadType::III => "III",
            EcgLeadType::Avr => "AVR",
            EcgLeadType::Avl => "AVL",
            EcgLeadType::Avf => "AVF",
            EcgLeadType::V => "V",
        }
    }
}

/// Heart rate source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HrSource {
    Unknown = 0,
    Ecg = 1,
    Bp1 = 2,
    Bp2 = 3,
    Bp3 = 4,
    Bp4 = 5,
    Pleth = 6,
    Bp5 = 7,
    Bp6 = 8,
    EcgMortara = 9,
    Bp7 = 10,
    Bp8 = 11,
    Pleth2 = 12,
}

impl HrSource {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(HrSource::Unknown),
            1 => Some(HrSource::Ecg),
            2 => Some(HrSource::Bp1),
            3 => Some(HrSource::Bp2),
            4 => Some(HrSource::Bp3),
            5 => Some(HrSource::Bp4),
            6 => Some(HrSource::Pleth),
            7 => Some(HrSource::Bp5),
            8 => Some(HrSource::Bp6),
            9 => Some(HrSource::EcgMortara),
            10 => Some(HrSource::Bp7),
            11 => Some(HrSource::Bp8),
            12 => Some(HrSource::Pleth2),
            _ => None,
        }
    }
}

/// Invasive pressure labels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum InvasivePressureLabel {
    NotDefined = 0,
    Art = 1,
    Cvp = 2,
    Pa = 3,
    Rap = 4,
    Rvp = 5,
    Lap = 6,
    Icp = 7,
    Abp = 8,
    P1 = 9,
    P2 = 10,
    P3 = 11,
    P4 = 12,
    P5 = 13,
    P6 = 14,
    Sp = 15,
    Fem = 16,
    Uac = 17,
    Uvc = 18,
    Icp2 = 19,
    P7 = 20,
    P8 = 21,
    Femv = 22,
}

impl InvasivePressureLabel {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(InvasivePressureLabel::NotDefined),
            1 => Some(InvasivePressureLabel::Art),
            2 => Some(InvasivePressureLabel::Cvp),
            3 => Some(InvasivePressureLabel::Pa),
            4 => Some(InvasivePressureLabel::Rap),
            5 => Some(InvasivePressureLabel::Rvp),
            6 => Some(InvasivePressureLabel::Lap),
            7 => Some(InvasivePressureLabel::Icp),
            8 => Some(InvasivePressureLabel::Abp),
            9 => Some(InvasivePressureLabel::P1),
            10 => Some(InvasivePressureLabel::P2),
            11 => Some(InvasivePressureLabel::P3),
            12 => Some(InvasivePressureLabel::P4),
            13 => Some(InvasivePressureLabel::P5),
            14 => Some(InvasivePressureLabel::P6),
            15 => Some(InvasivePressureLabel::Sp),
            16 => Some(InvasivePressureLabel::Fem),
            17 => Some(InvasivePressureLabel::Uac),
            18 => Some(InvasivePressureLabel::Uvc),
            19 => Some(InvasivePressureLabel::Icp2),
            20 => Some(InvasivePressureLabel::P7),
            21 => Some(InvasivePressureLabel::P8),
            22 => Some(InvasivePressureLabel::Femv),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            InvasivePressureLabel::NotDefined => "NOT_DEFINED",
            InvasivePressureLabel::Art => "ART",
            InvasivePressureLabel::Cvp => "CVP",
            InvasivePressureLabel::Pa => "PA",
            InvasivePressureLabel::Rap => "RAP",
            InvasivePressureLabel::Rvp => "RVP",
            InvasivePressureLabel::Lap => "LAP",
            InvasivePressureLabel::Icp => "ICP",
            InvasivePressureLabel::Abp => "ABP",
            InvasivePressureLabel::P1 => "P1",
            InvasivePressureLabel::P2 => "P2",
            InvasivePressureLabel::P3 => "P3",
            InvasivePressureLabel::P4 => "P4",
            InvasivePressureLabel::P5 => "P5",
            InvasivePressureLabel::P6 => "P6",
            InvasivePressureLabel::Sp => "SP",
            InvasivePressureLabel::Fem => "FEM",
            InvasivePressureLabel::Uac => "UAC",
            InvasivePressureLabel::Uvc => "UVC",
            InvasivePressureLabel::Icp2 => "ICP2",
            InvasivePressureLabel::P7 => "P7",
            InvasivePressureLabel::P8 => "P8",
            InvasivePressureLabel::Femv => "FEMV",
        }
    }
}

/// Temperature labels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum TemperatureLabel {
    NotUsed = 0,
    Eso = 1,
    Naso = 2,
    Tymp = 3,
    Rect = 4,
    Blad = 5,
    Axil = 6,
    Skin = 7,
    Airw = 8,
    Room = 9,
    Myo = 10,
    T1 = 11,
    T2 = 12,
    T3 = 13,
    T4 = 14,
    Core = 15,
    Surf = 16,
    T5 = 17,
    T6 = 18,
}

impl TemperatureLabel {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(TemperatureLabel::NotUsed),
            1 => Some(TemperatureLabel::Eso),
            2 => Some(TemperatureLabel::Naso),
            3 => Some(TemperatureLabel::Tymp),
            4 => Some(TemperatureLabel::Rect),
            5 => Some(TemperatureLabel::Blad),
            6 => Some(TemperatureLabel::Axil),
            7 => Some(TemperatureLabel::Skin),
            8 => Some(TemperatureLabel::Airw),
            9 => Some(TemperatureLabel::Room),
            10 => Some(TemperatureLabel::Myo),
            11 => Some(TemperatureLabel::T1),
            12 => Some(TemperatureLabel::T2),
            13 => Some(TemperatureLabel::T3),
            14 => Some(TemperatureLabel::T4),
            15 => Some(TemperatureLabel::Core),
            16 => Some(TemperatureLabel::Surf),
            17 => Some(TemperatureLabel::T5),
            18 => Some(TemperatureLabel::T6),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TemperatureLabel::NotUsed => "NOT_USED",
            TemperatureLabel::Eso => "ESO",
            TemperatureLabel::Naso => "NASO",
            TemperatureLabel::Tymp => "TYMP",
            TemperatureLabel::Rect => "RECT",
            TemperatureLabel::Blad => "BLAD",
            TemperatureLabel::Axil => "AXIL",
            TemperatureLabel::Skin => "SKIN",
            TemperatureLabel::Airw => "AIRW",
            TemperatureLabel::Room => "ROOM",
            TemperatureLabel::Myo => "MYO",
            TemperatureLabel::T1 => "T1",
            TemperatureLabel::T2 => "T2",
            TemperatureLabel::T3 => "T3",
            TemperatureLabel::T4 => "T4",
            TemperatureLabel::Core => "CORE",
            TemperatureLabel::Surf => "SURF",
            TemperatureLabel::T5 => "T5",
            TemperatureLabel::T6 => "T6",
        }
    }
}

/// Anesthesia agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum AnesthesiaAgent {
    Unknown = 0,
    None = 1,
    Hal = 2, // Halothane
    Enf = 3, // Enflurane
    Iso = 4, // Isoflurane
    Des = 5, // Desflurane
    Sev = 6, // Sevoflurane
}

impl AnesthesiaAgent {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(AnesthesiaAgent::Unknown),
            1 => Some(AnesthesiaAgent::None),
            2 => Some(AnesthesiaAgent::Hal),
            3 => Some(AnesthesiaAgent::Enf),
            4 => Some(AnesthesiaAgent::Iso),
            5 => Some(AnesthesiaAgent::Des),
            6 => Some(AnesthesiaAgent::Sev),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AnesthesiaAgent::Unknown => "UNKNOWN",
            AnesthesiaAgent::None => "NONE",
            AnesthesiaAgent::Hal => "HAL",
            AnesthesiaAgent::Enf => "ENF",
            AnesthesiaAgent::Iso => "ISO",
            AnesthesiaAgent::Des => "DES",
            AnesthesiaAgent::Sev => "SEV",
        }
    }
}

/// Parameter groups in physiological data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterGroup {
    Ecg,
    InvasivePressure,
    Nibp,
    Temperature,
    Spo2,
    Co2,
    O2,
    N2o,
    AnesthesiaAgent,
    FlowVolume,
    CardiacOutput,
    Nmt,
    EcgExtra,
    Svo2,
}
