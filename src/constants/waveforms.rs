//! Waveform type definitions and sampling rates

use serde::{Deserialize, Serialize};

/// Waveform types available in DRI protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WaveformType {
    /// Waveform command (control)
    Cmd = 0,
    /// ECG channel 1
    Ecg1 = 1,
    /// ECG channel 2
    Ecg2 = 2,
    /// ECG channel 3
    Ecg3 = 3,
    /// Invasive pressure channel 1
    Invp1 = 4,
    /// Invasive pressure channel 2
    Invp2 = 5,
    /// Invasive pressure channel 3
    Invp3 = 6,
    /// Invasive pressure channel 4
    Invp4 = 7,
    /// Plethysmograph (SpO2)
    Pleth = 8,
    /// CO2 waveform
    Co2 = 9,
    /// O2 waveform
    O2 = 10,
    /// N2O waveform
    N2o = 11,
    /// Anesthesia agent waveform
    Aa = 12,
    /// Airway pressure
    Awp = 13,
    /// Airway flow
    Flow = 14,
    /// Respiratory waveform
    Resp = 15,
    /// Invasive pressure channel 5
    Invp5 = 16,
    /// Invasive pressure channel 6
    Invp6 = 17,
    /// EEG channel 1
    Eeg1 = 18,
    /// EEG channel 2
    Eeg2 = 19,
    /// EEG channel 3
    Eeg3 = 20,
    /// EEG channel 4
    Eeg4 = 21,
    /// Airway volume
    Vol = 23,
    /// Tonometry catheter pressure
    TonoPress = 24,
    /// Spirometry loop status
    SpiLoopStatus = 29,
    /// Entropy waveform (100 Hz)
    Ent100 = 32,
    /// BIS waveform
    EegBis = 35,
    /// Invasive pressure channel 7
    Invp7 = 36,
    /// Invasive pressure channel 8
    Invp8 = 37,
    /// Secondary plethysmograph
    Pleth2 = 38,
}

impl WaveformType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(WaveformType::Cmd),
            1 => Some(WaveformType::Ecg1),
            2 => Some(WaveformType::Ecg2),
            3 => Some(WaveformType::Ecg3),
            4 => Some(WaveformType::Invp1),
            5 => Some(WaveformType::Invp2),
            6 => Some(WaveformType::Invp3),
            7 => Some(WaveformType::Invp4),
            8 => Some(WaveformType::Pleth),
            9 => Some(WaveformType::Co2),
            10 => Some(WaveformType::O2),
            11 => Some(WaveformType::N2o),
            12 => Some(WaveformType::Aa),
            13 => Some(WaveformType::Awp),
            14 => Some(WaveformType::Flow),
            15 => Some(WaveformType::Resp),
            16 => Some(WaveformType::Invp5),
            17 => Some(WaveformType::Invp6),
            18 => Some(WaveformType::Eeg1),
            19 => Some(WaveformType::Eeg2),
            20 => Some(WaveformType::Eeg3),
            21 => Some(WaveformType::Eeg4),
            23 => Some(WaveformType::Vol),
            24 => Some(WaveformType::TonoPress),
            29 => Some(WaveformType::SpiLoopStatus),
            32 => Some(WaveformType::Ent100),
            35 => Some(WaveformType::EegBis),
            36 => Some(WaveformType::Invp7),
            37 => Some(WaveformType::Invp8),
            38 => Some(WaveformType::Pleth2),
            _ => None,
        }
    }

    /// Get the string name for this waveform
    pub fn name(&self) -> &'static str {
        match self {
            WaveformType::Cmd => "CMD",
            WaveformType::Ecg1 => "ECG1",
            WaveformType::Ecg2 => "ECG2",
            WaveformType::Ecg3 => "ECG3",
            WaveformType::Invp1 => "INVP1",
            WaveformType::Invp2 => "INVP2",
            WaveformType::Invp3 => "INVP3",
            WaveformType::Invp4 => "INVP4",
            WaveformType::Pleth => "PLETH",
            WaveformType::Co2 => "CO2",
            WaveformType::O2 => "O2",
            WaveformType::N2o => "N2O",
            WaveformType::Aa => "AA",
            WaveformType::Awp => "AWP",
            WaveformType::Flow => "FLOW",
            WaveformType::Resp => "RESP",
            WaveformType::Invp5 => "INVP5",
            WaveformType::Invp6 => "INVP6",
            WaveformType::Eeg1 => "EEG1",
            WaveformType::Eeg2 => "EEG2",
            WaveformType::Eeg3 => "EEG3",
            WaveformType::Eeg4 => "EEG4",
            WaveformType::Vol => "VOL",
            WaveformType::TonoPress => "TONO_PRESS",
            WaveformType::SpiLoopStatus => "SPI_LOOP_STATUS",
            WaveformType::Ent100 => "ENT_100",
            WaveformType::EegBis => "EEG_BIS",
            WaveformType::Invp7 => "INVP7",
            WaveformType::Invp8 => "INVP8",
            WaveformType::Pleth2 => "PLETH2",
        }
    }

    /// Get waveform information (sample rate, unit, etc.)
    pub fn info(&self) -> WaveformInfo {
        get_waveform_info(*self)
    }
}

/// Waveform metadata
#[derive(Debug, Clone)]
pub struct WaveformInfo {
    pub waveform_type: WaveformType,
    pub samples_per_second: u16,
    pub unit: &'static str,
    pub description: &'static str,
}

/// Get waveform information for a given type
pub fn get_waveform_info(wf_type: WaveformType) -> WaveformInfo {
    match wf_type {
        WaveformType::Ecg1 | WaveformType::Ecg2 | WaveformType::Ecg3 => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 300,
            unit: "μV",
            description: "ECG waveform",
        },
        WaveformType::Invp1
        | WaveformType::Invp2
        | WaveformType::Invp3
        | WaveformType::Invp4
        | WaveformType::Invp5
        | WaveformType::Invp6
        | WaveformType::Invp7
        | WaveformType::Invp8 => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 100,
            unit: "mmHg (1/100)",
            description: "Invasive blood pressure",
        },
        WaveformType::Pleth | WaveformType::Pleth2 => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 100,
            unit: "% (1/10)",
            description: "Plethysmograph",
        },
        WaveformType::Co2 => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "% (1/100)",
            description: "CO2 concentration",
        },
        WaveformType::O2 => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "% (1/100)",
            description: "O2 concentration",
        },
        WaveformType::N2o => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "% (1/100)",
            description: "N2O concentration",
        },
        WaveformType::Aa => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "% (1/100)",
            description: "Anesthesia agent",
        },
        WaveformType::Awp => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "cmH2O (1/10)",
            description: "Airway pressure",
        },
        WaveformType::Flow => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "l/min (1/10)",
            description: "Airway flow",
        },
        WaveformType::Vol => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "ml",
            description: "Airway volume",
        },
        WaveformType::Resp => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "Ω (1/100)",
            description: "ECG impedance respiration",
        },
        WaveformType::Eeg1 | WaveformType::Eeg2 | WaveformType::Eeg3 | WaveformType::Eeg4 => {
            WaveformInfo {
                waveform_type: wf_type,
                samples_per_second: 100,
                unit: "μV (1/10)",
                description: "EEG channel",
            }
        }
        WaveformType::TonoPress => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "mmHg (1/10)",
            description: "Tonometry catheter pressure",
        },
        WaveformType::SpiLoopStatus => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 25,
            unit: "bit pattern",
            description: "Spirometry loop status",
        },
        WaveformType::Ent100 => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 100,
            unit: "μV (1/10)",
            description: "Entropy",
        },
        WaveformType::EegBis => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 300,
            unit: "μV",
            description: "BIS",
        },
        WaveformType::Cmd => WaveformInfo {
            waveform_type: wf_type,
            samples_per_second: 0,
            unit: "",
            description: "Command",
        },
    }
}

/// Maximum total sample rate allowed (samples/second)
pub const MAX_TOTAL_SAMPLE_RATE: u16 = 600;

/// Calculate total sample rate for a set of waveforms
pub fn calculate_total_sample_rate(waveforms: &[WaveformType]) -> u16 {
    waveforms
        .iter()
        .map(|wf| wf.info().samples_per_second)
        .sum()
}

/// Validate that a set of waveforms doesn't exceed max sample rate
pub fn validate_waveform_set(waveforms: &[WaveformType]) -> Result<(), String> {
    let total = calculate_total_sample_rate(waveforms);
    if total > MAX_TOTAL_SAMPLE_RATE {
        Err(format!(
            "Total sample rate {} exceeds maximum {}",
            total, MAX_TOTAL_SAMPLE_RATE
        ))
    } else {
        Ok(())
    }
}
