//! Physiological data decoding

use crate::constants::{
    PhdbClass, PhdbSubrecordType,
    physiological::{
        AnesthesiaAgent, EcgLeadType, HrSource, InvasivePressureLabel, TemperatureLabel,
    },
};
use crate::protocol::DriHeader;
use crate::{DriError, Result};
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};

use super::subrecords::*;

/// Physiological data record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalData {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Record class
    pub class: PhdbClass,
    /// Subrecord type (displayed, trend 10s, trend 60s)
    pub subtype: PhdbSubrecordType,

    // ECG data
    pub ecg_hr: Option<i16>,
    pub ecg_st1: Option<i16>,
    pub ecg_st2: Option<i16>,
    pub ecg_st3: Option<i16>,
    pub ecg_rr: Option<i16>,
    pub ecg_hr_source: Option<HrSource>,
    pub ecg_lead1: Option<EcgLeadType>,
    pub ecg_lead2: Option<EcgLeadType>,
    pub ecg_lead3: Option<EcgLeadType>,

    // Blood pressure
    pub nibp_sys: Option<i16>,
    pub nibp_dia: Option<i16>,
    pub nibp_mean: Option<i16>,
    pub nibp_hr: Option<i16>,

    // Invasive pressures (6 channels)
    pub invp1_sys: Option<i16>,
    pub invp1_dia: Option<i16>,
    pub invp1_mean: Option<i16>,
    pub invp1_label: Option<InvasivePressureLabel>,

    // SpO2
    pub spo2: Option<i16>,
    pub spo2_pr: Option<i16>,
    pub spo2_ir_amp: Option<i16>,

    // Temperatures (4 channels)
    pub temp1: Option<i16>,
    pub temp1_label: Option<TemperatureLabel>,
    pub temp2: Option<i16>,
    pub temp2_label: Option<TemperatureLabel>,

    // Gases
    pub co2_et: Option<i16>,
    pub co2_fi: Option<i16>,
    pub co2_rr: Option<i16>,
    pub o2_et: Option<i16>,
    pub o2_fi: Option<i16>,
    pub n2o_et: Option<i16>,
    pub n2o_fi: Option<i16>,

    // Anesthesia agent
    pub aa_et: Option<i16>,
    pub aa_fi: Option<i16>,
    pub aa_mac: Option<i16>,
    pub aa_agent: Option<AnesthesiaAgent>,

    // Spirometry
    pub flow_rr: Option<i16>,
    pub flow_ppeak: Option<i16>,
    pub flow_peep: Option<i16>,
    pub flow_tv_insp: Option<i16>,
    pub flow_tv_exp: Option<i16>,
    pub flow_mv_exp: Option<i16>,
}

impl PhysiologicalData {
    /// Create empty physiological data
    pub fn empty(timestamp: DateTime<Utc>, class: PhdbClass, subtype: PhdbSubrecordType) -> Self {
        Self {
            timestamp,
            class,
            subtype,
            ecg_hr: None,
            ecg_st1: None,
            ecg_st2: None,
            ecg_st3: None,
            ecg_rr: None,
            ecg_hr_source: None,
            ecg_lead1: None,
            ecg_lead2: None,
            ecg_lead3: None,
            nibp_sys: None,
            nibp_dia: None,
            nibp_mean: None,
            nibp_hr: None,
            invp1_sys: None,
            invp1_dia: None,
            invp1_mean: None,
            invp1_label: None,
            spo2: None,
            spo2_pr: None,
            spo2_ir_amp: None,
            temp1: None,
            temp1_label: None,
            temp2: None,
            temp2_label: None,
            co2_et: None,
            co2_fi: None,
            co2_rr: None,
            o2_et: None,
            o2_fi: None,
            n2o_et: None,
            n2o_fi: None,
            aa_et: None,
            aa_fi: None,
            aa_mac: None,
            aa_agent: None,
            flow_rr: None,
            flow_ppeak: None,
            flow_peep: None,
            flow_tv_insp: None,
            flow_tv_exp: None,
            flow_mv_exp: None,
        }
    }
}

/// Decode physiological data from a frame
pub fn decode_physiological(header: &DriHeader, data: &[u8]) -> Result<PhysiologicalData> {
    if header.subrecords.is_empty() {
        return Err(DriError::IncompleteFrame.into());
    }

    // Get the first subrecord
    let subrecord = &header.subrecords[0];
    let subtype = PhdbSubrecordType::from_u8(subrecord.sr_type)
        .ok_or(DriError::InvalidSubrecordType(subrecord.sr_type))?;

    // Get subrecord data
    let sub_data = header.get_subrecord_data(data, 0)?;

    // Parse timestamp (first 4 bytes of subrecord)
    let timestamp_raw = read_u32(sub_data, 0).ok_or(DriError::IncompleteFrame)?;
    let timestamp = DateTime::from_timestamp(timestamp_raw as i64, 0).unwrap_or_else(|| Utc::now());

    debug!(
        "Decoding physiological data: timestamp={}, subtype={:?}",
        timestamp, subtype
    );

    // Determine class from the last 2 bytes of subrecord (offset depends on size)
    // For now, assume Basic class
    let class = PhdbClass::Basic;

    let mut phys = PhysiologicalData::empty(timestamp, class, subtype);

    // Decode Basic class data (starts at offset 4, after timestamp)
    decode_basic_class(&mut phys, sub_data, 4)?;

    Ok(phys)
}

/// Decode Basic class physiological data
fn decode_basic_class(phys: &mut PhysiologicalData, data: &[u8], offset: usize) -> Result<()> {
    let mut pos = offset;

    // ECG Group (16 bytes)
    if let Some(ecg) = parse_ecg_group(data, pos) {
        phys.ecg_hr = ecg.hr;
        phys.ecg_st1 = ecg.st1;
        phys.ecg_st2 = ecg.st2;
        phys.ecg_st3 = ecg.st3;
        phys.ecg_rr = ecg.imp_rr;
        phys.ecg_hr_source = ecg.hr_source;
        phys.ecg_lead1 = ecg.lead1;
        phys.ecg_lead2 = ecg.lead2;
        phys.ecg_lead3 = ecg.lead3;
    }
    pos += 16;

    // Invasive Pressure Groups (4 channels, 14 bytes each = 56 bytes)
    // We'll parse just the first one for now
    if let Some(invp) = parse_invp_group(data, pos) {
        phys.invp1_sys = invp.sys;
        phys.invp1_dia = invp.dia;
        phys.invp1_mean = invp.mean;
        phys.invp1_label = invp.label;
    }
    pos += 14 * 4; // Skip all 4 pressure channels

    // NIBP Group (14 bytes)
    if let Some(nibp) = parse_nibp_group(data, pos) {
        phys.nibp_sys = nibp.sys;
        phys.nibp_dia = nibp.dia;
        phys.nibp_mean = nibp.mean;
        phys.nibp_hr = nibp.hr;
    }
    pos += 14;

    // Temperature Groups (4 channels, 8 bytes each = 32 bytes)
    if let Some(temp) = parse_temp_group(data, pos) {
        phys.temp1 = temp.temp;
        phys.temp1_label = temp.label;
    }
    if let Some(temp) = parse_temp_group(data, pos + 8) {
        phys.temp2 = temp.temp;
        phys.temp2_label = temp.label;
    }
    pos += 8 * 4;

    // SpO2 Group (14 bytes)
    if let Some(spo2) = parse_spo2_group(data, pos) {
        phys.spo2 = spo2.spo2;
        phys.spo2_pr = spo2.pr;
        phys.spo2_ir_amp = spo2.ir_amp;
    }
    pos += 14;

    // CO2 Group (14 bytes)
    if let Some(co2) = parse_co2_group(data, pos) {
        phys.co2_et = co2.et;
        phys.co2_fi = co2.fi;
        phys.co2_rr = co2.rr;
    }
    pos += 14;

    // O2 Group (10 bytes)
    if let Some(o2) = parse_o2_group(data, pos) {
        phys.o2_et = o2.et;
        phys.o2_fi = o2.fi;
    }
    pos += 10;

    // N2O Group (10 bytes)
    if let Some(n2o) = parse_n2o_group(data, pos) {
        phys.n2o_et = n2o.et;
        phys.n2o_fi = n2o.fi;
    }
    pos += 10;

    // AA Group (12 bytes)
    if let Some(aa) = parse_aa_group(data, pos) {
        phys.aa_et = aa.et;
        phys.aa_fi = aa.fi;
        phys.aa_mac = aa.mac_sum;
        phys.aa_agent = aa.agent;
    }
    pos += 12;

    // Flow & Volume Group (22 bytes)
    if let Some(flow) = parse_flow_vol_group(data, pos) {
        phys.flow_rr = flow.rr;
        phys.flow_ppeak = flow.ppeak;
        phys.flow_peep = flow.peep;
        phys.flow_tv_insp = flow.tv_insp;
        phys.flow_tv_exp = flow.tv_exp;
        phys.flow_mv_exp = flow.mv_exp;
    }

    debug!(
        "Decoded physiological data: HR={:?}, SpO2={:?}, NIBP={:?}/{:?}",
        phys.ecg_hr, phys.spo2, phys.nibp_sys, phys.nibp_dia
    );

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

fn parse_ecg_group(data: &[u8], offset: usize) -> Option<EcgGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let hr = read_valid_i16(data, offset + 6);
    let st1 = read_valid_i16(data, offset + 8);
    let st2 = read_valid_i16(data, offset + 10);
    let st3 = read_valid_i16(data, offset + 12);
    let imp_rr = read_valid_i16(data, offset + 14);

    // Extract HR source from status bits 3-6
    let hr_source_bits = header.get_bits(3, 6) as u8;
    let hr_source = HrSource::from_u8(hr_source_bits);

    // Extract lead configuration from label
    let lead1 = EcgLeadType::from_u8(extract_label_bits(header.label, 8, 11) as u8);
    let lead2 = EcgLeadType::from_u8(extract_label_bits(header.label, 4, 7) as u8);
    let lead3 = EcgLeadType::from_u8(extract_label_bits(header.label, 0, 3) as u8);

    Some(EcgGroup {
        hr,
        st1,
        st2,
        st3,
        imp_rr,
        hr_source,
        lead1,
        lead2,
        lead3,
    })
}

struct InvpGroup {
    sys: Option<i16>,
    dia: Option<i16>,
    mean: Option<i16>,
    label: Option<InvasivePressureLabel>,
}

fn parse_invp_group(data: &[u8], offset: usize) -> Option<InvpGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let sys = read_valid_i16(data, offset + 6);
    let dia = read_valid_i16(data, offset + 8);
    let mean = read_valid_i16(data, offset + 10);
    let label = InvasivePressureLabel::from_u16(header.label);

    Some(InvpGroup {
        sys,
        dia,
        mean,
        label,
    })
}

struct NibpGroup {
    sys: Option<i16>,
    dia: Option<i16>,
    mean: Option<i16>,
    hr: Option<i16>,
}

fn parse_nibp_group(data: &[u8], offset: usize) -> Option<NibpGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let sys = read_valid_i16(data, offset + 6);
    let dia = read_valid_i16(data, offset + 8);
    let mean = read_valid_i16(data, offset + 10);
    let hr = read_valid_i16(data, offset + 12);

    Some(NibpGroup { sys, dia, mean, hr })
}

struct TempGroup {
    temp: Option<i16>,
    label: Option<TemperatureLabel>,
}

fn parse_temp_group(data: &[u8], offset: usize) -> Option<TempGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let temp = read_valid_i16(data, offset + 6);
    let label = TemperatureLabel::from_u16(header.label);

    Some(TempGroup { temp, label })
}

struct Spo2Group {
    spo2: Option<i16>,
    pr: Option<i16>,
    ir_amp: Option<i16>,
}

fn parse_spo2_group(data: &[u8], offset: usize) -> Option<Spo2Group> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let spo2 = read_valid_i16(data, offset + 6);
    let pr = read_valid_i16(data, offset + 8);
    let ir_amp = read_valid_i16(data, offset + 10);

    Some(Spo2Group { spo2, pr, ir_amp })
}

struct Co2Group {
    et: Option<i16>,
    fi: Option<i16>,
    rr: Option<i16>,
}

fn parse_co2_group(data: &[u8], offset: usize) -> Option<Co2Group> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let et = read_valid_i16(data, offset + 6);
    let fi = read_valid_i16(data, offset + 8);
    let rr = read_valid_i16(data, offset + 10);

    Some(Co2Group { et, fi, rr })
}

struct O2Group {
    et: Option<i16>,
    fi: Option<i16>,
}

fn parse_o2_group(data: &[u8], offset: usize) -> Option<O2Group> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let et = read_valid_i16(data, offset + 6);
    let fi = read_valid_i16(data, offset + 8);

    Some(O2Group { et, fi })
}

struct N2oGroup {
    et: Option<i16>,
    fi: Option<i16>,
}

fn parse_n2o_group(data: &[u8], offset: usize) -> Option<N2oGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let et = read_valid_i16(data, offset + 6);
    let fi = read_valid_i16(data, offset + 8);

    Some(N2oGroup { et, fi })
}

struct AaGroup {
    et: Option<i16>,
    fi: Option<i16>,
    mac_sum: Option<i16>,
    agent: Option<AnesthesiaAgent>,
}

fn parse_aa_group(data: &[u8], offset: usize) -> Option<AaGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let et = read_valid_i16(data, offset + 6);
    let fi = read_valid_i16(data, offset + 8);
    let mac_sum = read_valid_i16(data, offset + 10);
    let agent = AnesthesiaAgent::from_u16(header.label);

    Some(AaGroup {
        et,
        fi,
        mac_sum,
        agent,
    })
}

struct FlowVolGroup {
    rr: Option<i16>,
    ppeak: Option<i16>,
    peep: Option<i16>,
    tv_insp: Option<i16>,
    tv_exp: Option<i16>,
    mv_exp: Option<i16>,
}

fn parse_flow_vol_group(data: &[u8], offset: usize) -> Option<FlowVolGroup> {
    let header = parse_group_header(data, offset)?;
    if !header.exists() {
        return None;
    }

    let rr = read_valid_i16(data, offset + 6);
    let ppeak = read_valid_i16(data, offset + 8);
    let peep = read_valid_i16(data, offset + 10);
    let _pplat = read_valid_i16(data, offset + 12);
    let tv_insp = read_valid_i16(data, offset + 14);
    let tv_exp = read_valid_i16(data, offset + 16);
    let _compliance = read_valid_i16(data, offset + 18);
    let mv_exp = read_valid_i16(data, offset + 20);

    Some(FlowVolGroup {
        rr,
        ppeak,
        peep,
        tv_insp,
        tv_exp,
        mv_exp,
    })
}
