//! CSV file writer for DRI data

use crate::decode::physiological::PhysiologicalData;
use crate::decode::waveforms::WaveformData;
use anyhow::Result;
use csv::Writer;
use std::fs::File;
use std::path::Path;

pub struct CsvWriter {
    main_writer: Option<Writer<File>>,
    waveform_writer: Option<Writer<File>>,
    main_path: String,
    waveform_path: String,
}

impl CsvWriter {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path_str = base_path.as_ref().to_string_lossy().to_string();
        let waveform_path = if base_path_str.ends_with(".csv") {
            base_path_str.replace(".csv", ".waveforms.csv")
        } else {
            format!("{}.waveforms.csv", base_path_str)
        };

        Ok(Self {
            main_writer: None,
            waveform_writer: None,
            main_path: base_path_str,
            waveform_path,
        })
    }

    /// Write physiological data
    pub fn write_physiological(&mut self, data: &PhysiologicalData) -> Result<()> {
        // Initialize writer on first call
        if self.main_writer.is_none() {
            let file = File::create(&self.main_path)?;
            let mut writer = Writer::from_writer(file);

            // Write header with all fields including status flags
            writer.write_record(&[
                "timestamp",
                "class",
                "subtype",
                // ECG
                "ecg_exists",
                "ecg_active",
                "ecg_asystole",
                "ecg_noise",
                "ecg_artifact",
                "ecg_learning",
                "ecg_pacer_on",
                "ecg_ch1_off",
                "ecg_ch2_off",
                "ecg_ch3_off",
                "ecg_hr",
                "ecg_st1_mm",
                "ecg_st2_mm",
                "ecg_st3_mm",
                "ecg_rr",
                "ecg_hr_source",
                "ecg_lead1",
                "ecg_lead2",
                "ecg_lead3",
                // NIBP
                "nibp_exists",
                "nibp_active",
                "nibp_auto_mode",
                "nibp_stat_mode",
                "nibp_measuring",
                "nibp_stasis",
                "nibp_calibrating",
                "nibp_old_data",
                "nibp_sys_mmhg",
                "nibp_dia_mmhg",
                "nibp_mean_mmhg",
                "nibp_hr",
                // INVP1
                "invp1_exists",
                "invp1_active",
                "invp1_label",
                "invp1_sys_mmhg",
                "invp1_dia_mmhg",
                "invp1_mean_mmhg",
                "invp1_hr",
                // SpO2
                "spo2_exists",
                "spo2_active",
                "spo2_percent",
                "spo2_pr",
                "spo2_ir_amp_percent",
                // Temperature 1
                "temp1_exists",
                "temp1_active",
                "temp1_label",
                "temp1_celsius",
                // Temperature 2
                "temp2_exists",
                "temp2_active",
                "temp2_label",
                "temp2_celsius",
                // CO2
                "co2_exists",
                "co2_active",
                "co2_apnea",
                "co2_calibrating",
                "co2_zeroing",
                "co2_occlusion",
                "co2_air_leak",
                "co2_apnea_resp",
                "co2_apnea_deactivated",
                "co2_wet",
                "co2_et_percent",
                "co2_fi_percent",
                "co2_rr",
                // O2
                "o2_exists",
                "o2_active",
                "o2_calibrating",
                "o2_meas_off",
                "o2_et_percent",
                "o2_fi_percent",
                // N2O
                "n2o_exists",
                "n2o_active",
                "n2o_calibrating",
                "n2o_meas_off",
                "n2o_et_percent",
                "n2o_fi_percent",
                // AA
                "aa_exists",
                "aa_active",
                "aa_calibrating",
                "aa_meas_off",
                "aa_agent",
                "aa_et_percent",
                "aa_fi_percent",
                "aa_mac",
                // Flow/Volume (Ventilator)
                "flow_exists",
                "flow_active",
                "flow_disconnection",
                "flow_calibrating",
                "flow_zeroing",
                "flow_obstruction",
                "flow_leak",
                "flow_meas_off",
                "flow_tv_base",
                "flow_rr",
                "flow_ppeak_cmh2o",
                "flow_peep_cmh2o",
                "flow_pplat_cmh2o",
                "flow_tv_insp_ml",
                "flow_tv_exp_ml",
                "flow_compliance_ml_per_cmh2o",
                "flow_mv_exp_l_per_min",
            ])?;

            self.main_writer = Some(writer);
        }

        // Write data row
        if let Some(writer) = &mut self.main_writer {
            writer.write_record(&[
                data.timestamp.to_rfc3339(),
                format!("{:?}", data.class),
                format!("{:?}", data.subtype),
                // ECG status
                data.ecg_status.exists.to_string(),
                data.ecg_status.active.to_string(),
                data.ecg_status.asystole.to_string(),
                data.ecg_status.noise.to_string(),
                data.ecg_status.artifact.to_string(),
                data.ecg_status.learning.to_string(),
                data.ecg_status.pacer_on.to_string(),
                data.ecg_status.channel1_off.to_string(),
                data.ecg_status.channel2_off.to_string(),
                data.ecg_status.channel3_off.to_string(),
                // ECG values
                format_option_f64(data.ecg_hr),
                format_option_f64(data.ecg_st1),
                format_option_f64(data.ecg_st2),
                format_option_f64(data.ecg_st3),
                format_option_f64(data.ecg_rr),
                format_option_debug(&data.ecg_hr_source),
                format_option_debug(&data.ecg_lead1),
                format_option_debug(&data.ecg_lead2),
                format_option_debug(&data.ecg_lead3),
                // NIBP status
                data.nibp_status.exists.to_string(),
                data.nibp_status.active.to_string(),
                data.nibp_status.auto_mode.to_string(),
                data.nibp_status.stat_mode.to_string(),
                data.nibp_status.measuring.to_string(),
                data.nibp_status.stasis_on.to_string(),
                data.nibp_status.calibrating.to_string(),
                data.nibp_status.data_older_than_60s.to_string(),
                // NIBP values
                format_option_f64(data.nibp_sys),
                format_option_f64(data.nibp_dia),
                format_option_f64(data.nibp_mean),
                format_option_f64(data.nibp_hr),
                // INVP1 status
                data.invp1_status.exists.to_string(),
                data.invp1_status.active.to_string(),
                // INVP1 values
                format_option_debug(&data.invp1_label),
                format_option_f64(data.invp1_sys),
                format_option_f64(data.invp1_dia),
                format_option_f64(data.invp1_mean),
                format_option_f64(data.invp1_hr),
                // SpO2 status
                data.spo2_status.exists.to_string(),
                data.spo2_status.active.to_string(),
                // SpO2 values
                format_option_f64(data.spo2),
                format_option_f64(data.spo2_pr),
                format_option_f64(data.spo2_ir_amp),
                // Temp1 status
                data.temp1_status.exists.to_string(),
                data.temp1_status.active.to_string(),
                // Temp1 values
                format_option_debug(&data.temp1_label),
                format_option_f64(data.temp1),
                // Temp2 status
                data.temp2_status.exists.to_string(),
                data.temp2_status.active.to_string(),
                // Temp2 values
                format_option_debug(&data.temp2_label),
                format_option_f64(data.temp2),
                // CO2 status
                data.co2_status.exists.to_string(),
                data.co2_status.active.to_string(),
                data.co2_status.apnea_co2.to_string(),
                data.co2_status.calibrating_sensor.to_string(),
                data.co2_status.zeroing_sensor.to_string(),
                data.co2_status.occlusion.to_string(),
                data.co2_status.air_leak.to_string(),
                data.co2_status.apnea_from_resp.to_string(),
                data.co2_status.apnea_deactivated.to_string(),
                data.co2_status.wet_condition.to_string(),
                // CO2 values
                format_option_f64(data.co2_et),
                format_option_f64(data.co2_fi),
                format_option_f64(data.co2_rr),
                // O2 status
                data.o2_status.exists.to_string(),
                data.o2_status.active.to_string(),
                data.o2_status.calibrating.to_string(),
                data.o2_status.measurement_off.to_string(),
                // O2 values
                format_option_f64(data.o2_et),
                format_option_f64(data.o2_fi),
                // N2O status
                data.n2o_status.exists.to_string(),
                data.n2o_status.active.to_string(),
                data.n2o_status.calibrating.to_string(),
                data.n2o_status.measurement_off.to_string(),
                // N2O values
                format_option_f64(data.n2o_et),
                format_option_f64(data.n2o_fi),
                // AA status
                data.aa_status.exists.to_string(),
                data.aa_status.active.to_string(),
                data.aa_status.calibrating.to_string(),
                data.aa_status.measurement_off.to_string(),
                // AA values
                format_option_debug(&data.aa_agent),
                format_option_f64(data.aa_et),
                format_option_f64(data.aa_fi),
                format_option_f64(data.aa_mac),
                // Flow status
                data.flow_status.exists.to_string(),
                data.flow_status.active.to_string(),
                data.flow_status.disconnection.to_string(),
                data.flow_status.calibrating.to_string(),
                data.flow_status.zeroing.to_string(),
                data.flow_status.obstruction.to_string(),
                data.flow_status.leak.to_string(),
                data.flow_status.measurement_off.to_string(),
                format!("{:?}", data.flow_status.tv_base),
                // Flow values
                format_option_f64(data.flow_rr),
                format_option_f64(data.flow_ppeak),
                format_option_f64(data.flow_peep),
                format_option_f64(data.flow_pplat),
                format_option_f64(data.flow_tv_insp),
                format_option_f64(data.flow_tv_exp),
                format_option_f64(data.flow_compliance),
                format_option_f64(data.flow_mv_exp),
            ])?;

            writer.flush()?;
        }

        Ok(())
    }

    /// Write waveform data
    pub fn write_waveform(&mut self, data: &WaveformData) -> Result<()> {
        // Initialize writer on first call
        if self.waveform_writer.is_none() {
            let file = File::create(&self.waveform_path)?;
            let mut writer = Writer::from_writer(file);

            writer.write_record(&[
                "timestamp",
                "waveform_type",
                "sample_rate",
                "sample_count",
                "gap",
                "pacer_detected",
                "lead_off",
                "samples_json",
            ])?;

            self.waveform_writer = Some(writer);
        }

        // Write data row
        if let Some(writer) = &mut self.waveform_writer {
            let samples_json = serde_json::to_string(&data.samples)?;

            writer.write_record(&[
                data.timestamp.to_rfc3339(),
                format!("{:?}", data.waveform_type),
                data.sample_rate.to_string(),
                data.samples.len().to_string(),
                data.status.gap.to_string(),
                data.status.pacer_detected.to_string(),
                data.status.lead_off.to_string(),
                samples_json,
            ])?;

            writer.flush()?;
        }

        Ok(())
    }
}

/// Format Option<f64> for CSV
fn format_option_f64(opt: Option<f64>) -> String {
    match opt {
        Some(val) => format!("{:.2}", val),
        None => String::new(),
    }
}

/// Format Option<Debug> for CSV
fn format_option_debug<T: std::fmt::Debug>(opt: &Option<T>) -> String {
    match opt {
        Some(val) => format!("{:?}", val),
        None => String::new(),
    }
}
