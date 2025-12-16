//! Physiological data decoding

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Import from constants
use crate::constants::dri_types::{PhdbClass, PhdbSubrecordType};
use crate::constants::physiological::{
    AnesthesiaAgent, EcgLeadType, HrSource, InvasivePressureLabel, TemperatureLabel,
};
use crate::constants::scaling::{
    SCALE_AWP_100, SCALE_COMPLIANCE_100, SCALE_IR_AMP_10, SCALE_MAC_100, SCALE_PERCENT_100,
    SCALE_PRESSURE_100, SCALE_ST_100, SCALE_TEMP_100, SCALE_VOLUME_10, scale_valid_i16,
};
use crate::constants::special_values::is_invalid;

// Import from same module
use super::status_bits::*;
use super::subrecords::*;

/// Physiological data record with properly scaled values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalData {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Record class
    pub class: PhdbClass,
    /// Subrecord type (displayed, trend 10s, trend 60s)
    pub subtype: PhdbSubrecordType,

    // ECG data (with status)
    pub ecg_status: EcgStatus,
    pub ecg_hr: Option<f64>,  // beats/min (no scaling)
    pub ecg_st1: Option<f64>, // mm (scaled from 1/100)
    pub ecg_st2: Option<f64>, // mm (scaled from 1/100)
    pub ecg_st3: Option<f64>, // mm (scaled from 1/100)
    pub ecg_rr: Option<f64>,  // breaths/min (no scaling)
    pub ecg_hr_source: Option<HrSource>,
    pub ecg_lead1: Option<EcgLeadType>,
    pub ecg_lead2: Option<EcgLeadType>,
    pub ecg_lead3: Option<EcgLeadType>,

    // NIBP (with status)
    pub nibp_status: NibpStatus,
    pub nibp_sys: Option<f64>,  // mmHg (scaled from 1/100)
    pub nibp_dia: Option<f64>,  // mmHg (scaled from 1/100)
    pub nibp_mean: Option<f64>, // mmHg (scaled from 1/100)
    pub nibp_hr: Option<f64>,   // beats/min (no scaling)

    // Invasive pressures
    pub invp1_status: GenericStatus,
    pub invp1_sys: Option<f64>,  // mmHg (scaled from 1/100)
    pub invp1_dia: Option<f64>,  // mmHg (scaled from 1/100)
    pub invp1_mean: Option<f64>, // mmHg (scaled from 1/100)
    pub invp1_hr: Option<f64>,   // beats/min (no scaling)
    pub invp1_label: Option<InvasivePressureLabel>,

    // SpO2 (with status)
    pub spo2_status: Spo2Status,
    pub spo2: Option<f64>,        // % (scaled from 1/100)
    pub spo2_pr: Option<f64>,     // beats/min (no scaling)
    pub spo2_ir_amp: Option<f64>, // % (scaled from 1/10)

    // Temperatures
    pub temp1_status: GenericStatus,
    pub temp1: Option<f64>, // °C (scaled from 1/100)
    pub temp1_label: Option<TemperatureLabel>,
    pub temp2_status: GenericStatus,
    pub temp2: Option<f64>, // °C (scaled from 1/100)
    pub temp2_label: Option<TemperatureLabel>,

    // CO2 (with status)
    pub co2_status: Co2Status,
    pub co2_et: Option<f64>, // % (scaled from 1/100)
    pub co2_fi: Option<f64>, // % (scaled from 1/100)
    pub co2_rr: Option<f64>, // breaths/min (no scaling)

    // O2 (with status)
    pub o2_status: GasStatus,
    pub o2_et: Option<f64>, // % (scaled from 1/100)
    pub o2_fi: Option<f64>, // % (scaled from 1/100)

    // N2O (with status)
    pub n2o_status: GasStatus,
    pub n2o_et: Option<f64>, // % (scaled from 1/100)
    pub n2o_fi: Option<f64>, // % (scaled from 1/100)

    // Anesthesia agent (with status)
    pub aa_status: GasStatus,
    pub aa_et: Option<f64>,  // % (scaled from 1/100)
    pub aa_fi: Option<f64>,  // % (scaled from 1/100)
    pub aa_mac: Option<f64>, // (scaled from 1/100)
    pub aa_agent: Option<AnesthesiaAgent>,

    // Spirometry/Ventilator (with status)
    pub flow_status: FlowVolStatus,
    pub flow_rr: Option<f64>,         // breaths/min (no scaling)
    pub flow_ppeak: Option<f64>,      // cmH2O (scaled from 1/100)
    pub flow_peep: Option<f64>,       // cmH2O (scaled from 1/100)
    pub flow_pplat: Option<f64>,      // cmH2O (scaled from 1/100)
    pub flow_tv_insp: Option<f64>,    // ml (scaled from 1/10)
    pub flow_tv_exp: Option<f64>,     // ml (scaled from 1/10)
    pub flow_compliance: Option<f64>, // ml/cmH2O (scaled from 1/100)
    pub flow_mv_exp: Option<f64>,     // l/min (scaled from 1/100)
}

impl PhysiologicalData {
    /// Create an empty physiological data record
    pub fn empty(timestamp: DateTime<Utc>, class: PhdbClass, subtype: PhdbSubrecordType) -> Self {
        Self {
            timestamp,
            class,
            subtype,

            // ECG
            ecg_status: EcgStatus::default(),
            ecg_hr: None,
            ecg_st1: None,
            ecg_st2: None,
            ecg_st3: None,
            ecg_rr: None,
            ecg_hr_source: None,
            ecg_lead1: None,
            ecg_lead2: None,
            ecg_lead3: None,

            // NIBP
            nibp_status: NibpStatus::default(),
            nibp_sys: None,
            nibp_dia: None,
            nibp_mean: None,
            nibp_hr: None,

            // INVP1
            invp1_status: GenericStatus::default(),
            invp1_sys: None,
            invp1_dia: None,
            invp1_mean: None,
            invp1_hr: None,
            invp1_label: None,

            // SpO2
            spo2_status: Spo2Status::default(),
            spo2: None,
            spo2_pr: None,
            spo2_ir_amp: None,

            // Temperatures
            temp1_status: GenericStatus::default(),
            temp1: None,
            temp1_label: None,
            temp2_status: GenericStatus::default(),
            temp2: None,
            temp2_label: None,

            // CO2
            co2_status: Co2Status::default(),
            co2_et: None,
            co2_fi: None,
            co2_rr: None,

            // O2
            o2_status: GasStatus::default(),
            o2_et: None,
            o2_fi: None,

            // N2O
            n2o_status: GasStatus::default(),
            n2o_et: None,
            n2o_fi: None,

            // AA
            aa_status: GasStatus::default(),
            aa_et: None,
            aa_fi: None,
            aa_mac: None,
            aa_agent: None,

            // Flow/Volume
            flow_status: FlowVolStatus::default(),
            flow_rr: None,
            flow_ppeak: None,
            flow_peep: None,
            flow_pplat: None,
            flow_tv_insp: None,
            flow_tv_exp: None,
            flow_compliance: None,
            flow_mv_exp: None,
        }
    }
}

/// Decode physiological data from a DRI subrecord
pub fn decode_physiological(
    subrecord_data: &[u8],
    subtype: PhdbSubrecordType,
    class: PhdbClass,
) -> Result<PhysiologicalData> {
    if subrecord_data.len() < 1088 {
        return Err(anyhow!(
            "Physiological subrecord too short: {} bytes",
            subrecord_data.len()
        ));
    }

    // Parse timestamp (first 4 bytes)
    let timestamp_raw = read_u32(&subrecord_data[0..4]);
    let timestamp = DateTime::from_timestamp(timestamp_raw as i64, 0)
        .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp_raw))?;

    // Create empty physiological data structure
    let mut phys = PhysiologicalData::empty(timestamp, class, subtype);

    // Decode based on class (data starts at offset 4, after timestamp)
    let class_data = &subrecord_data[4..];

    match class {
        PhdbClass::Basic => {
            decode_basic_class(class_data, &mut phys)?;
        }
        PhdbClass::Ext1 => {
            // TODO: Implement Ext1 class decoding in Phase 2
            log::debug!("Ext1 class decoding not yet implemented");
        }
        PhdbClass::Ext2 => {
            // TODO: Implement Ext2 class decoding in Phase 2
            log::debug!("Ext2 class decoding not yet implemented");
        }
        PhdbClass::Ext3 => {
            // TODO: Implement Ext3 class decoding in Phase 2
            log::debug!("Ext3 class decoding not yet implemented");
        }
    }

    Ok(phys)
}

/// Decode Basic class physiological data
fn decode_basic_class(data: &[u8], phys: &mut PhysiologicalData) -> Result<()> {
    // ECG (offset 0, 16 bytes)
    if data.len() >= 16 {
        let (status, hr, st1, st2, st3, rr, hr_source, lead1, lead2, lead3) =
            parse_ecg_group(&data[0..16])?;
        phys.ecg_status = status;
        phys.ecg_hr = hr;
        phys.ecg_st1 = st1;
        phys.ecg_st2 = st2;
        phys.ecg_st3 = st3;
        phys.ecg_rr = rr;
        phys.ecg_hr_source = hr_source;
        phys.ecg_lead1 = lead1;
        phys.ecg_lead2 = lead2;
        phys.ecg_lead3 = lead3;
    }

    // INVP1 (offset 16, 14 bytes)
    if data.len() >= 30 {
        let (status, sys, dia, mean, hr, label) = parse_invp_group(&data[16..30])?;
        phys.invp1_status = status;
        phys.invp1_sys = sys;
        phys.invp1_dia = dia;
        phys.invp1_mean = mean;
        phys.invp1_hr = hr;
        phys.invp1_label = label;
    }

    // NIBP (offset 76, 14 bytes)
    if data.len() >= 90 {
        let (status, sys, dia, mean, hr) = parse_nibp_group(&data[76..90])?;
        phys.nibp_status = status;
        phys.nibp_sys = sys;
        phys.nibp_dia = dia;
        phys.nibp_mean = mean;
        phys.nibp_hr = hr;
    }

    // TEMP1 (offset 90, 8 bytes)
    if data.len() >= 98 {
        let (status, temp, label) = parse_temp_group(&data[90..98])?;
        phys.temp1_status = status;
        phys.temp1 = temp;
        phys.temp1_label = label;
    }

    // TEMP2 (offset 98, 8 bytes)
    if data.len() >= 106 {
        let (status, temp, label) = parse_temp_group(&data[98..106])?;
        phys.temp2_status = status;
        phys.temp2 = temp;
        phys.temp2_label = label;
    }

    // SpO2 (offset 122, 14 bytes)
    if data.len() >= 136 {
        let (status, spo2, pr, ir_amp) = parse_spo2_group(&data[122..136])?;
        phys.spo2_status = status;
        phys.spo2 = spo2;
        phys.spo2_pr = pr;
        phys.spo2_ir_amp = ir_amp;
    }

    // CO2 (offset 136, 14 bytes)
    if data.len() >= 150 {
        let (status, et, fi, rr) = parse_co2_group(&data[136..150])?;
        phys.co2_status = status;
        phys.co2_et = et;
        phys.co2_fi = fi;
        phys.co2_rr = rr;
    }

    // O2 (offset 150, 10 bytes)
    if data.len() >= 160 {
        let (status, et, fi) = parse_o2_group(&data[150..160])?;
        phys.o2_status = status;
        phys.o2_et = et;
        phys.o2_fi = fi;
    }

    // N2O (offset 160, 10 bytes)
    if data.len() >= 170 {
        let (status, et, fi) = parse_n2o_group(&data[160..170])?;
        phys.n2o_status = status;
        phys.n2o_et = et;
        phys.n2o_fi = fi;
    }

    // AA (offset 170, 12 bytes)
    if data.len() >= 182 {
        let (status, et, fi, mac, agent) = parse_aa_group(&data[170..182])?;
        phys.aa_status = status;
        phys.aa_et = et;
        phys.aa_fi = fi;
        phys.aa_mac = mac;
        phys.aa_agent = agent;
    }

    // Flow/Volume (offset 182, 22 bytes) - VENTILATOR DATA
    if data.len() >= 204 {
        let (status, rr, ppeak, peep, pplat, tv_insp, tv_exp, compliance, mv_exp) =
            parse_flow_vol_group(&data[182..204])?;
        phys.flow_status = status;
        phys.flow_rr = rr;
        phys.flow_ppeak = ppeak;
        phys.flow_peep = peep;
        phys.flow_pplat = pplat;
        phys.flow_tv_insp = tv_insp;
        phys.flow_tv_exp = tv_exp;
        phys.flow_compliance = compliance;
        phys.flow_mv_exp = mv_exp;
    }

    Ok(())
}

// Group parsing functions

struct EcgGroup {
    hr: Option<i16>,
    st1: Option<i16>,
    st2: Option<i16>,
    st3: Option<i16>,
    imp_rr: Option<i16>,
    hr_source: Option<HrSource>,
    lead1: Option<EcgLeadType>,
    lead2: Option<EcgLeadType>,
    lead3: Option<EcgLeadType>,
}

/// Parse ECG group (offset 0 in basic class, 16 bytes)
fn parse_ecg_group(
    data: &[u8],
) -> Result<(
    EcgStatus,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<HrSource>,
    Option<EcgLeadType>,
    Option<EcgLeadType>,
    Option<EcgLeadType>,
)> {
    if data.len() < 16 {
        return Err(anyhow!("ECG group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let ecg_status = EcgStatus::from_status(header.status);

    // HR - no scaling needed (already in beats/min)
    let hr_raw = read_i16(&data[6..8]);
    let hr = if is_invalid(hr_raw) {
        None
    } else {
        Some(hr_raw as f64)
    };

    // ST levels - scale from 1/100 mm to mm
    let st1 = scale_valid_i16(read_i16(&data[8..10]), SCALE_ST_100);
    let st2 = scale_valid_i16(read_i16(&data[10..12]), SCALE_ST_100);
    let st3 = scale_valid_i16(read_i16(&data[12..14]), SCALE_ST_100);

    // Impedance RR - no scaling needed
    let rr_raw = read_i16(&data[14..16]);
    let rr = if is_invalid(rr_raw) {
        None
    } else {
        Some(rr_raw as f64)
    };

    // Parse HR source from status bits 3-6
    let hr_source_bits = ((header.status >> 3) & 0x0F) as u8;
    let hr_source = HrSource::from_u8(hr_source_bits);

    // Parse ECG leads from label field
    let lead1_bits = (header.label & 0x0F) as u8;
    let lead2_bits = ((header.label >> 4) & 0x0F) as u8;
    let lead3_bits = ((header.label >> 8) & 0x0F) as u8;

    let lead1 = EcgLeadType::from_u8(lead1_bits);
    let lead2 = EcgLeadType::from_u8(lead2_bits);
    let lead3 = EcgLeadType::from_u8(lead3_bits);

    Ok((
        ecg_status, hr, st1, st2, st3, rr, hr_source, lead1, lead2, lead3,
    ))
}

struct InvpGroup {
    sys: Option<i16>,
    dia: Option<i16>,
    mean: Option<i16>,
    label: Option<InvasivePressureLabel>,
}

/// Parse invasive pressure group (14 bytes)
fn parse_invp_group(
    data: &[u8],
) -> Result<(
    GenericStatus,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<InvasivePressureLabel>,
)> {
    if data.len() < 14 {
        return Err(anyhow!("Invasive pressure group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let status = GenericStatus::from_status(header.status);
    let label = InvasivePressureLabel::from_u16(header.label);

    // Scale from 1/100 mmHg to mmHg
    let sys = scale_valid_i16(read_i16(&data[6..8]), SCALE_PRESSURE_100);
    let dia = scale_valid_i16(read_i16(&data[8..10]), SCALE_PRESSURE_100);
    let mean = scale_valid_i16(read_i16(&data[10..12]), SCALE_PRESSURE_100);

    // HR - no scaling
    let hr_raw = read_i16(&data[12..14]);
    let hr = if is_invalid(hr_raw) {
        None
    } else {
        Some(hr_raw as f64)
    };

    Ok((status, sys, dia, mean, hr, label))
}

struct NibpGroup {
    sys: Option<i16>,
    dia: Option<i16>,
    mean: Option<i16>,
    hr: Option<i16>,
}

/// Parse NIBP group (offset 76 in basic class, 14 bytes)
fn parse_nibp_group(
    data: &[u8],
) -> Result<(
    NibpStatus,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
)> {
    if data.len() < 14 {
        return Err(anyhow!("NIBP group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let nibp_status = NibpStatus::from_label(header.label);

    // Scale from 1/100 mmHg to mmHg
    let sys = scale_valid_i16(read_i16(&data[6..8]), SCALE_PRESSURE_100);
    let dia = scale_valid_i16(read_i16(&data[8..10]), SCALE_PRESSURE_100);
    let mean = scale_valid_i16(read_i16(&data[10..12]), SCALE_PRESSURE_100);

    // HR - no scaling
    let hr_raw = read_i16(&data[12..14]);
    let hr = if is_invalid(hr_raw) {
        None
    } else {
        Some(hr_raw as f64)
    };

    Ok((nibp_status, sys, dia, mean, hr))
}

struct TempGroup {
    temp: Option<i16>,
    label: Option<TemperatureLabel>,
}

/// Parse temperature group (8 bytes)
fn parse_temp_group(data: &[u8]) -> Result<(GenericStatus, Option<f64>, Option<TemperatureLabel>)> {
    if data.len() < 8 {
        return Err(anyhow!("Temperature group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let status = GenericStatus::from_status(header.status);
    let label = TemperatureLabel::from_u16(header.label);

    // Scale from 1/100 °C to °C
    let temp = scale_valid_i16(read_i16(&data[6..8]), SCALE_TEMP_100);

    Ok((status, temp, label))
}

struct Spo2Group {
    spo2: Option<i16>,
    pr: Option<i16>,
    ir_amp: Option<i16>,
}

/// Parse SpO2 group (offset 122 in basic class, 14 bytes)
fn parse_spo2_group(data: &[u8]) -> Result<(Spo2Status, Option<f64>, Option<f64>, Option<f64>)> {
    if data.len() < 14 {
        return Err(anyhow!("SpO2 group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let spo2_status = Spo2Status::from_status(header.status);

    // Scale from 1/100 % to %
    let spo2 = scale_valid_i16(read_i16(&data[6..8]), SCALE_PERCENT_100);

    // Pulse rate - no scaling
    let pr_raw = read_i16(&data[8..10]);
    let pr = if is_invalid(pr_raw) {
        None
    } else {
        Some(pr_raw as f64)
    };

    // IR amplitude - scale from 1/10 % to %
    let ir_amp = scale_valid_i16(read_i16(&data[10..12]), SCALE_IR_AMP_10);

    Ok((spo2_status, spo2, pr, ir_amp))
}

struct Co2Group {
    et: Option<i16>,
    fi: Option<i16>,
    rr: Option<i16>,
}

/// Parse CO2 group (offset 136 in basic class, 14 bytes)
fn parse_co2_group(data: &[u8]) -> Result<(Co2Status, Option<f64>, Option<f64>, Option<f64>)> {
    if data.len() < 14 {
        return Err(anyhow!("CO2 group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let co2_status = Co2Status::from_status(header.status);

    // Scale from 1/100 % to %
    let et = scale_valid_i16(read_i16(&data[6..8]), SCALE_PERCENT_100);
    let fi = scale_valid_i16(read_i16(&data[8..10]), SCALE_PERCENT_100);

    // RR - no scaling
    let rr_raw = read_i16(&data[10..12]);
    let rr = if is_invalid(rr_raw) {
        None
    } else {
        Some(rr_raw as f64)
    };

    Ok((co2_status, et, fi, rr))
}

struct O2Group {
    et: Option<i16>,
    fi: Option<i16>,
}

/// Parse O2 group (offset 150 in basic class, 10 bytes)
fn parse_o2_group(data: &[u8]) -> Result<(GasStatus, Option<f64>, Option<f64>)> {
    if data.len() < 10 {
        return Err(anyhow!("O2 group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let o2_status = GasStatus::from_status(header.status);

    // Scale from 1/100 % to %
    let et = scale_valid_i16(read_i16(&data[6..8]), SCALE_PERCENT_100);
    let fi = scale_valid_i16(read_i16(&data[8..10]), SCALE_PERCENT_100);

    Ok((o2_status, et, fi))
}

struct N2oGroup {
    et: Option<i16>,
    fi: Option<i16>,
}

/// Parse N2O group (offset 160 in basic class, 10 bytes)
fn parse_n2o_group(data: &[u8]) -> Result<(GasStatus, Option<f64>, Option<f64>)> {
    if data.len() < 10 {
        return Err(anyhow!("N2O group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let n2o_status = GasStatus::from_status(header.status);

    // Scale from 1/100 % to %
    let et = scale_valid_i16(read_i16(&data[6..8]), SCALE_PERCENT_100);
    let fi = scale_valid_i16(read_i16(&data[8..10]), SCALE_PERCENT_100);

    Ok((n2o_status, et, fi))
}

struct AaGroup {
    et: Option<i16>,
    fi: Option<i16>,
    mac_sum: Option<i16>,
    agent: Option<AnesthesiaAgent>,
}

/// Parse anesthesia agent group (offset 170 in basic class, 12 bytes)
fn parse_aa_group(
    data: &[u8],
) -> Result<(
    GasStatus,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<AnesthesiaAgent>,
)> {
    if data.len() < 12 {
        return Err(anyhow!("AA group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let aa_status = GasStatus::from_status(header.status);
    let agent = AnesthesiaAgent::from_u16(header.label);

    // Scale from 1/100 % to %
    let et = scale_valid_i16(read_i16(&data[6..8]), SCALE_PERCENT_100);
    let fi = scale_valid_i16(read_i16(&data[8..10]), SCALE_PERCENT_100);
    let mac = scale_valid_i16(read_i16(&data[10..12]), SCALE_MAC_100);

    Ok((aa_status, et, fi, mac, agent))
}

struct FlowVolGroup {
    rr: Option<i16>,
    ppeak: Option<i16>,
    peep: Option<i16>,
    tv_insp: Option<i16>,
    tv_exp: Option<i16>,
    mv_exp: Option<i16>,
}

/// Parse flow & volume group (offset 182 in basic class, 22 bytes)
fn parse_flow_vol_group(
    data: &[u8],
) -> Result<(
    FlowVolStatus,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
)> {
    if data.len() < 22 {
        return Err(anyhow!("Flow/volume group data too short"));
    }

    let header = GroupHeader::parse(&data[0..6])?;
    let flow_status = FlowVolStatus::from_status(header.status);

    // RR - no scaling
    let rr_raw = read_i16(&data[6..8]);
    let rr = if is_invalid(rr_raw) {
        None
    } else {
        Some(rr_raw as f64)
    };

    // Scale pressures from 1/100 cmH2O to cmH2O
    let ppeak = scale_valid_i16(read_i16(&data[8..10]), SCALE_AWP_100);
    let peep = scale_valid_i16(read_i16(&data[10..12]), SCALE_AWP_100);
    let pplat = scale_valid_i16(read_i16(&data[12..14]), SCALE_AWP_100);

    // Scale volumes from 1/10 ml to ml
    let tv_insp = scale_valid_i16(read_i16(&data[14..16]), SCALE_VOLUME_10);
    let tv_exp = scale_valid_i16(read_i16(&data[16..18]), SCALE_VOLUME_10);

    // Scale compliance from 1/100 ml/cmH2O to ml/cmH2O
    let compliance = scale_valid_i16(read_i16(&data[18..20]), SCALE_COMPLIANCE_100);

    // Scale MV from 1/100 l/min to l/min
    let mv_exp = scale_valid_i16(read_i16(&data[20..22]), SCALE_PERCENT_100);

    Ok((
        flow_status,
        rr,
        ppeak,
        peep,
        pplat,
        tv_insp,
        tv_exp,
        compliance,
        mv_exp,
    ))
}
