//! GE DRI Protocol Diagnostic Tool
//!
//! A simple diagnostic tool to verify connectivity with GE CARESCAPE monitors.
//! Automatically starts listening for all data types and logs everything to console.
//!
//! Usage:
//!   cargo run --bin diagnostic
//!
//! This will:
//! 1. Let you select the serial port
//! 2. Request all physiological data every 5 seconds
//! 3. Request common waveforms (ECG1, PLETH)
//! 4. Log ALL received data in a verbose, readable format
//!
//! Press Ctrl+C to stop

use anyhow::Result;
use std::io::Write;
use std::time::Instant;

use ge_dri_prototype::decode::{Decoder, DriRecord};
use ge_dri_prototype::device::SerialDevice;
use ge_dri_prototype::protocol::DriHeader;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          GE DRI Protocol - DIAGNOSTIC MODE                   ║");
    println!("║  Listening for ALL data from your GE monitor                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Select serial port (interactive)
    let port_name = ge_dri_prototype::device::select_port()?;
    println!("✅ Selected port: {}", port_name);

    // Connect to device
    println!("🔌 Connecting to monitor...");
    let mut device = SerialDevice::open(&port_name)?;
    println!("✅ Connected successfully!");
    println!();

    // Fixed settings for diagnostic mode
    let interval: u16 = 5; // 5 seconds
    let waveforms = vec!["ECG1", "PLETH"];

    println!("📋 DIAGNOSTIC SETTINGS:");
    println!("   • Physiological data interval: {} seconds", interval);
    println!("   • Waveforms: {}", waveforms.join(", "));
    println!();

    // Request data from monitor
    println!("📡 Requesting data from monitor...");
    device.request_displayed_values(interval)?;
    device.request_waveforms(&waveforms)?;
    println!("✅ Requests sent!");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("                    LISTENING FOR DATA...");
    println!("                    Press Ctrl+C to stop");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    let decoder = Decoder::new();
    let start_time = Instant::now();
    let mut frame_count: u32 = 0;
    let mut phys_count: u32 = 0;
    let mut wave_count: u32 = 0;

    loop {
        match device.read_frame() {
            Ok(frame) => {
                frame_count += 1;
                let elapsed = start_time.elapsed().as_secs();

                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!(
                    "📦 FRAME #{} ({}s elapsed) - {} bytes",
                    frame_count,
                    elapsed,
                    frame.data.len()
                );

                // Parse header
                let header = match DriHeader::parse(&frame.data) {
                    Ok(h) => h,
                    Err(e) => {
                        println!("   ❌ Header parse error: {}", e);
                        continue;
                    }
                };

                println!(
                    "   📋 Header: type={:?}, level={:?}, time={}",
                    header.r_maintype,
                    header.dri_level,
                    header.timestamp()
                );
                println!("   📋 Subrecords: {}", header.subrecords.len());

                // Extract data
                let data = match header.extract_data(&frame.data) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("   ❌ Data extract error: {}", e);
                        continue;
                    }
                };

                // Decode
                match decoder.decode_frame(&header, data) {
                    Ok(Some(record)) => {
                        match &record {
                            DriRecord::Physiological(phys) => {
                                phys_count += 1;
                                println!();
                                println!(
                                    "   🏥 PHYSIOLOGICAL DATA (#{}) - class={:?}, subtype={:?}",
                                    phys_count, phys.class, phys.subtype
                                );
                                println!(
                                    "   ─────────────────────────────────────────────────────"
                                );

                                // ECG
                                println!("   💓 ECG:");
                                println!(
                                    "      • Status: exists={}, active={}, asystole={}, noise={}",
                                    phys.ecg_status.exists,
                                    phys.ecg_status.active,
                                    phys.ecg_status.asystole,
                                    phys.ecg_status.noise
                                );
                                print_value("      • Heart Rate", phys.ecg_hr, "bpm");
                                print_value("      • ST1", phys.ecg_st1, "mm");
                                print_value("      • ST2", phys.ecg_st2, "mm");
                                print_value("      • ST3", phys.ecg_st3, "mm");
                                print_value("      • Resp Rate (imp)", phys.ecg_rr, "/min");
                                if let Some(src) = &phys.ecg_hr_source {
                                    println!("      • HR Source: {:?}", src);
                                }
                                if let Some(lead) = &phys.ecg_lead1 {
                                    println!("      • Lead 1: {:?}", lead);
                                }

                                // SpO2
                                println!("   🩸 SpO2:");
                                println!(
                                    "      • Status: exists={}, active={}",
                                    phys.spo2_status.exists, phys.spo2_status.active
                                );
                                print_value("      • SpO2", phys.spo2, "%");
                                print_value("      • Pulse Rate", phys.spo2_pr, "bpm");
                                print_value("      • IR Amplitude", phys.spo2_ir_amp, "%");

                                // NIBP
                                println!("   🩺 NIBP:");
                                println!(
                                    "      • Status: exists={}, active={}, measuring={}",
                                    phys.nibp_status.exists,
                                    phys.nibp_status.active,
                                    phys.nibp_status.measuring
                                );
                                print_value("      • Systolic", phys.nibp_sys, "mmHg");
                                print_value("      • Diastolic", phys.nibp_dia, "mmHg");
                                print_value("      • Mean", phys.nibp_mean, "mmHg");
                                print_value("      • HR", phys.nibp_hr, "bpm");

                                // Invasive Pressure 1
                                if phys.invp1_status.exists {
                                    println!("   📈 Invasive Pressure 1:");
                                    println!(
                                        "      • Status: exists={}, active={}",
                                        phys.invp1_status.exists, phys.invp1_status.active
                                    );
                                    if let Some(label) = &phys.invp1_label {
                                        println!("      • Label: {:?}", label);
                                    }
                                    print_value("      • Systolic", phys.invp1_sys, "mmHg");
                                    print_value("      • Diastolic", phys.invp1_dia, "mmHg");
                                    print_value("      • Mean", phys.invp1_mean, "mmHg");
                                }

                                // Temperature
                                println!("   🌡️  Temperature:");
                                println!(
                                    "      • Temp1 Status: exists={}, active={}",
                                    phys.temp1_status.exists, phys.temp1_status.active
                                );
                                if let Some(label) = &phys.temp1_label {
                                    println!("      • Temp1 Label: {:?}", label);
                                }
                                print_value("      • Temp1", phys.temp1, "°C");
                                if phys.temp2_status.exists {
                                    print_value("      • Temp2", phys.temp2, "°C");
                                }

                                // CO2
                                println!("   💨 CO2:");
                                println!(
                                    "      • Status: exists={}, active={}, apnea={}",
                                    phys.co2_status.exists,
                                    phys.co2_status.active,
                                    phys.co2_status.apnea_co2
                                );
                                print_value("      • EtCO2", phys.co2_et, "%");
                                print_value("      • FiCO2", phys.co2_fi, "%");
                                print_value("      • Resp Rate", phys.co2_rr, "/min");

                                // O2
                                println!("   🫁 O2:");
                                println!(
                                    "      • Status: exists={}, active={}",
                                    phys.o2_status.exists, phys.o2_status.active
                                );
                                print_value("      • EtO2", phys.o2_et, "%");
                                print_value("      • FiO2", phys.o2_fi, "%");

                                // N2O
                                if phys.n2o_status.exists {
                                    println!("   🔵 N2O:");
                                    print_value("      • EtN2O", phys.n2o_et, "%");
                                    print_value("      • FiN2O", phys.n2o_fi, "%");
                                }

                                // Anesthesia Agent
                                if phys.aa_status.exists {
                                    println!("   💊 Anesthesia Agent:");
                                    if let Some(agent) = &phys.aa_agent {
                                        println!("      • Agent: {:?}", agent);
                                    }
                                    print_value("      • Et", phys.aa_et, "%");
                                    print_value("      • Fi", phys.aa_fi, "%");
                                    print_value("      • MAC", phys.aa_mac, "");
                                }

                                // Ventilator / Flow & Volume
                                println!("   🌬️  Ventilator (Flow & Volume):");
                                println!(
                                    "      • Status: exists={}, active={}, disconnection={}",
                                    phys.flow_status.exists,
                                    phys.flow_status.active,
                                    phys.flow_status.disconnection
                                );
                                print_value("      • Resp Rate", phys.flow_rr, "/min");
                                print_value("      • Ppeak", phys.flow_ppeak, "cmH2O");
                                print_value("      • PEEP", phys.flow_peep, "cmH2O");
                                print_value("      • Pplat", phys.flow_pplat, "cmH2O");
                                print_value("      • TV insp", phys.flow_tv_insp, "ml");
                                print_value("      • TV exp", phys.flow_tv_exp, "ml");
                                print_value("      • Compliance", phys.flow_compliance, "ml/cmH2O");
                                print_value("      • MV exp", phys.flow_mv_exp, "L/min");

                                println!();
                            }
                            DriRecord::Waveform { waveforms } => {
                                wave_count += 1;
                                println!();
                                println!(
                                    "   📈 WAVEFORM DATA (#{}) - {} waveforms",
                                    wave_count,
                                    waveforms.len()
                                );
                                println!(
                                    "   ─────────────────────────────────────────────────────"
                                );

                                for wf in waveforms {
                                    println!(
                                        "   • {:?}: {} samples @ {} Hz (gap={}, pacer={}, lead_off={})",
                                        wf.waveform_type,
                                        wf.samples.len(),
                                        wf.sample_rate,
                                        wf.status.gap,
                                        wf.status.pacer_detected,
                                        wf.status.lead_off
                                    );

                                    // Show first few samples
                                    if !wf.samples.is_empty() {
                                        let preview: Vec<String> = wf
                                            .samples
                                            .iter()
                                            .take(10)
                                            .map(|s| s.to_string())
                                            .collect();
                                        println!(
                                            "     First 10 samples: [{}{}]",
                                            preview.join(", "),
                                            if wf.samples.len() > 10 { ", ..." } else { "" }
                                        );

                                        // Calculate min/max/avg
                                        let min = wf.samples.iter().min().unwrap_or(&0);
                                        let max = wf.samples.iter().max().unwrap_or(&0);
                                        let sum: i64 = wf.samples.iter().map(|&x| x as i64).sum();
                                        let avg = sum as f64 / wf.samples.len() as f64;
                                        println!(
                                            "     Stats: min={}, max={}, avg={:.1}",
                                            min, max, avg
                                        );
                                    }
                                }
                                println!();
                            }
                        }
                    }
                    Ok(None) => {
                        println!("   ⚪ No decodable data in this frame");
                    }
                    Err(e) => {
                        println!("   ❌ Decode error: {}", e);
                    }
                }

                // Summary line
                println!(
                    "   📊 TOTALS: {} frames, {} phys records, {} waveform batches",
                    frame_count, phys_count, wave_count
                );
            }
            Err(e) => {
                println!();
                println!("❌ Read error: {}", e);
                println!("   Waiting for more data...");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}

/// Helper function to print optional values nicely
fn print_value(label: &str, value: Option<f64>, unit: &str) {
    match value {
        Some(v) => println!("{}: {:.2} {}", label, v, unit),
        None => println!("{}: --", label),
    }
}
