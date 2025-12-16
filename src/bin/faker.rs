//! GE Monitor Simulator - Fake DRI data generator for testing
//!
//! This simulates a GE CARESCAPE Monitor B650/B850 sending physiological data
//! and waveforms over a serial port.
//!
//! Usage:
//!   cargo run --bin faker -- --port COM3
//!   cargo run --bin faker -- --port /dev/ttyUSB0
//!
//! This will:
//! 1. Wait for physiological data requests
//! 2. Send displayed values every N seconds (as requested)
//! 3. Wait for waveform requests
//! 4. Send waveforms continuously
//!
//! Press Ctrl+C to stop

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use log::{debug, info};
use serialport::SerialPort;
use std::io::Write;
use std::thread;
use std::time::Duration;

// DRI Constants
const FRAME_CHAR: u8 = 0x7E;
const CTRL_CHAR: u8 = 0x7D;
const BIT5: u8 = 0x7C;
const HEADER_SIZE: usize = 40;

// Main types
const DRI_MT_PHDB: u16 = 0;
const DRI_MT_WAVE: u16 = 1;

// Physiological subrecord type
const DRI_PH_DISPL: u8 = 1;

// Physiological class (Basic)
const DRI_PHDBCL_BASIC: u8 = 0;

#[derive(Parser)]
#[command(name = "GE Monitor Faker")]
#[command(about = "Simulates a GE CARESCAPE Monitor sending DRI data")]
struct Args {
    /// Serial port to use
    #[arg(short, long)]
    port: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    info!("🏥 GE Monitor Simulator Starting");
    info!("Serial port: {}", args.port);

    // Open serial port with GE monitor settings
    let mut port = serialport::new(&args.port, 19200)
        .timeout(Duration::from_millis(100))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::Hardware)
        .open()?;

    info!("✅ Serial port opened successfully");
    info!("Waiting for requests from client...");

    let mut phdb_interval = 0u16;
    let mut waveforms_requested: Vec<u8> = Vec::new();
    let mut frame_number = 0u8;

    // Simulation state
    let mut hr = 75.0;
    let mut spo2 = 98.0;
    let mut nibp_sys = 120.0;
    let mut nibp_dia = 80.0;
    let mut temp = 37.0;
    let mut etco2 = 5.2;
    let mut rr = 16.0;
    let mut peep = 5.0;
    let mut ppeak = 20.0;
    let mut tv = 500.0;

    // Waveform phase
    let mut waveform_phase = 0.0;

    let start_time = std::time::Instant::now();
    let mut last_phdb_send = start_time;

    loop {
        // Check for incoming requests
        let mut buffer = [0u8; 256];
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                debug!("Received {} bytes", n);

                // Parse request (simplified - just look for main type)
                if let Some(request) = parse_request(&buffer[..n]) {
                    match request {
                        Request::PhdbRequest { interval, .. } => {
                            phdb_interval = interval;
                            info!("📊 Physiological data requested (interval: {}s)", interval);
                        }
                        Request::WaveformRequest { waveforms } => {
                            waveforms_requested = waveforms.clone();
                            info!("📈 Waveforms requested: {:?}", waveforms);
                        }
                        Request::StopAll => {
                            info!("🛑 Stop request received");
                            phdb_interval = 0;
                            waveforms_requested.clear();
                        }
                    }
                }
            }
            Ok(_) => {}                                                  // No data
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {} // Timeout is ok
            Err(e) => {
                log::error!("Read error: {}", e);
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        }

        // Send physiological data if requested
        if phdb_interval > 0 {
            let elapsed = start_time.elapsed().as_secs();
            if elapsed - last_phdb_send.elapsed().as_secs() >= phdb_interval as u64 {
                // Update vitals with realistic variations
                hr = vary_value(hr, 75.0, 5.0);
                spo2 = vary_value(spo2, 98.0, 2.0);
                nibp_sys = vary_value(nibp_sys, 120.0, 10.0);
                nibp_dia = vary_value(nibp_dia, 80.0, 5.0);
                temp = vary_value(temp, 37.0, 0.3);
                etco2 = vary_value(etco2, 5.2, 0.5);
                rr = vary_value(rr, 16.0, 2.0);
                peep = vary_value(peep, 5.0, 0.5);
                ppeak = vary_value(ppeak, 20.0, 2.0);
                tv = vary_value(tv, 500.0, 50.0);

                info!(
                    "💓 HR: {:.0} | SpO2: {:.0}% | BP: {:.0}/{:.0} | Temp: {:.1}°C | EtCO2: {:.1}%",
                    hr, spo2, nibp_sys, nibp_dia, temp, etco2
                );

                let phdb_frame = create_phdb_frame(
                    frame_number,
                    hr,
                    spo2,
                    nibp_sys,
                    nibp_dia,
                    temp,
                    etco2,
                    rr,
                    peep,
                    ppeak,
                    tv,
                );

                send_frame(&mut *port, &phdb_frame)?;
                frame_number = frame_number.wrapping_add(1);
                last_phdb_send = std::time::Instant::now();
            }
        }

        // Send waveforms if requested (every 250ms for simplicity)
        if !waveforms_requested.is_empty() {
            let waveform_frame =
                create_waveform_frame(frame_number, &waveforms_requested, &mut waveform_phase, hr);

            send_frame(&mut *port, &waveform_frame)?;
            frame_number = frame_number.wrapping_add(1);
            thread::sleep(Duration::from_millis(250));
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

#[derive(Debug)]
enum Request {
    PhdbRequest { interval: u16 },
    WaveformRequest { waveforms: Vec<u8> },
    StopAll,
}

fn parse_request(data: &[u8]) -> Option<Request> {
    // Very simplified parser - just look for unstuffed frames
    let mut unstuffed = Vec::new();
    let mut i = 0;
    let mut in_frame = false;

    while i < data.len() {
        if data[i] == FRAME_CHAR {
            if in_frame && !unstuffed.is_empty() {
                // End of frame
                break;
            }
            in_frame = true;
            i += 1;
            continue;
        }

        if !in_frame {
            i += 1;
            continue;
        }

        if data[i] == CTRL_CHAR && i + 1 < data.len() {
            unstuffed.push(data[i + 1] | BIT5);
            i += 2;
        } else {
            unstuffed.push(data[i]);
            i += 1;
        }
    }

    if unstuffed.len() < HEADER_SIZE {
        return None;
    }

    // Parse main type
    let main_type = u16::from_le_bytes([unstuffed[16], unstuffed[17]]);

    match main_type {
        DRI_MT_PHDB => {
            if unstuffed.len() >= HEADER_SIZE + 3 {
                let interval =
                    u16::from_le_bytes([unstuffed[HEADER_SIZE + 1], unstuffed[HEADER_SIZE + 2]]);
                if interval == 0 {
                    Some(Request::StopAll)
                } else {
                    Some(Request::PhdbRequest { interval })
                }
            } else {
                None
            }
        }
        DRI_MT_WAVE => {
            if unstuffed.len() >= HEADER_SIZE + 12 {
                let req_type =
                    u16::from_le_bytes([unstuffed[HEADER_SIZE], unstuffed[HEADER_SIZE + 1]]);
                if req_type == 1 {
                    // Stop waveforms
                    Some(Request::StopAll)
                } else {
                    let mut waveforms = Vec::new();
                    for i in 0..8 {
                        let wf_type = unstuffed[HEADER_SIZE + 4 + i];
                        if wf_type == 0xFF {
                            break;
                        }
                        if wf_type != 0 {
                            waveforms.push(wf_type);
                        }
                    }
                    Some(Request::WaveformRequest { waveforms })
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn create_phdb_frame(
    frame_nbr: u8,
    hr: f64,
    spo2: f64,
    sys: f64,
    dia: f64,
    temp: f64,
    etco2: f64,
    rr: f64,
    peep: f64,
    ppeak: f64,
    tv: f64,
) -> Vec<u8> {
    let mut data = vec![0u8; HEADER_SIZE + 1088]; // Header + physiological data subrecord

    let timestamp = Utc::now().timestamp() as u32;

    // Header
    data[0..2].copy_from_slice(&((HEADER_SIZE + 1088) as u16).to_le_bytes());
    data[2] = frame_nbr;
    data[3] = 8; // DRI_LEVEL_02
    data[6..10].copy_from_slice(&timestamp.to_le_bytes());
    data[16..18].copy_from_slice(&DRI_MT_PHDB.to_le_bytes());

    // Subrecord descriptor
    data[18..20].copy_from_slice(&0u16.to_le_bytes()); // offset 0
    data[20] = DRI_PH_DISPL; // subrecord type
    data[21..23].copy_from_slice(&0u16.to_le_bytes());
    data[23] = 0xFF; // end marker

    // Physiological data (1088 bytes)
    let phys_start = HEADER_SIZE;

    // Timestamp (4 bytes)
    data[phys_start..phys_start + 4].copy_from_slice(&timestamp.to_le_bytes());

    // Basic class data starts at offset 4
    let basic_start = phys_start + 4;

    // ECG group (16 bytes at offset 4)
    write_group_header(&mut data[basic_start..], 0x0003); // exists + active
    write_i16(&mut data[basic_start + 6..], (hr as i16, 1)); // HR
    write_i16(&mut data[basic_start + 8..], (0, 100)); // ST1 (scaled by 100)
    write_i16(&mut data[basic_start + 10..], (0, 100)); // ST2
    write_i16(&mut data[basic_start + 12..], (0, 100)); // ST3
    write_i16(&mut data[basic_start + 14..], (rr as i16, 1)); // RR

    // Skip to NIBP (after 4 invasive pressure groups: 16 + 4*14 = 72)
    let nibp_start = basic_start + 72;
    write_group_header(&mut data[nibp_start..], 0x0003);
    write_i16(&mut data[nibp_start + 6..], ((sys * 100.0) as i16, 1));
    write_i16(&mut data[nibp_start + 8..], ((dia * 100.0) as i16, 1));
    write_i16(
        &mut data[nibp_start + 10..],
        (((sys + 2.0 * dia) / 3.0 * 100.0) as i16, 1),
    );
    write_i16(&mut data[nibp_start + 12..], (hr as i16, 1));

    // Temperatures (4x 8 bytes = 32 bytes)
    let temp_start = nibp_start + 14;
    write_group_header(&mut data[temp_start..], 0x0003);
    write_i16(&mut data[temp_start + 6..], ((temp * 100.0) as i16, 1));

    // SpO2 (14 bytes)
    let spo2_start = temp_start + 32;
    write_group_header(&mut data[spo2_start..], 0x0003);
    write_i16(&mut data[spo2_start + 6..], ((spo2 * 100.0) as i16, 1)); // SpO2
    write_i16(&mut data[spo2_start + 8..], (hr as i16, 1)); // PR
    write_i16(&mut data[spo2_start + 10..], (150, 1)); // IR amplitude (15.0%)

    // CO2 (14 bytes)
    let co2_start = spo2_start + 14;
    write_group_header(&mut data[co2_start..], 0x0003);
    write_i16(&mut data[co2_start + 6..], ((etco2 * 100.0) as i16, 1)); // EtCO2
    write_i16(&mut data[co2_start + 8..], (400, 1)); // FiCO2 (0.4%)
    write_i16(&mut data[co2_start + 10..], (rr as i16, 1)); // RR
    write_i16(&mut data[co2_start + 12..], (7600, 1)); // Ambient pressure (760 mmHg)

    // O2 (10 bytes)
    let o2_start = co2_start + 14;
    write_group_header(&mut data[o2_start..], 0x0003);
    write_i16(&mut data[o2_start + 6..], (2100, 1)); // EtO2 (21%)
    write_i16(&mut data[o2_start + 8..], (2100, 1)); // FiO2 (21%)

    // N2O (10 bytes)
    let n2o_start = o2_start + 10;
    write_group_header(&mut data[n2o_start..], 0x0001); // exists but not active

    // AA (12 bytes)
    let aa_start = n2o_start + 10;
    write_group_header(&mut data[aa_start..], 0x0001);

    // Flow & Volume (22 bytes)
    let flow_start = aa_start + 12;
    write_group_header(&mut data[flow_start..], 0x0003); // active
    write_i16(&mut data[flow_start + 6..], (rr as i16, 1)); // RR
    write_i16(&mut data[flow_start + 8..], ((ppeak * 100.0) as i16, 1)); // Ppeak
    write_i16(&mut data[flow_start + 10..], ((peep * 100.0) as i16, 1)); // PEEP
    write_i16(&mut data[flow_start + 12..], (0, 1)); // Pplat
    write_i16(&mut data[flow_start + 14..], ((tv * 10.0) as i16, 1)); // TV insp
    write_i16(&mut data[flow_start + 16..], ((tv * 10.0) as i16, 1)); // TV exp
    write_i16(&mut data[flow_start + 18..], (5000, 1)); // Compliance (50.0)
    write_i16(
        &mut data[flow_start + 20..],
        ((rr * tv / 1000.0 * 100.0) as i16, 1),
    ); // MV

    // Class marker at end (bytes 1086-1087)
    let class_offset = phys_start + 1086;
    let cl_drilvl_subt = (DRI_PHDBCL_BASIC as u16) << 8 | DRI_PH_DISPL as u16;
    data[class_offset..class_offset + 2].copy_from_slice(&cl_drilvl_subt.to_le_bytes());

    data
}

fn create_waveform_frame(frame_nbr: u8, waveforms: &[u8], phase: &mut f64, hr: f64) -> Vec<u8> {
    let timestamp = Utc::now().timestamp() as u32;

    // Calculate data size (header per waveform + samples)
    let samples_per_frame = 75; // 250ms * 300 samples/s = 75 samples for ECG
    let mut total_size = HEADER_SIZE;

    for _ in waveforms {
        total_size += 6 + (samples_per_frame * 2); // header + samples
    }

    let mut data = vec![0u8; total_size];

    // Header
    data[0..2].copy_from_slice(&(total_size as u16).to_le_bytes());
    data[2] = frame_nbr;
    data[3] = 8; // DRI_LEVEL_02
    data[6..10].copy_from_slice(&timestamp.to_le_bytes());
    data[16..18].copy_from_slice(&DRI_MT_WAVE.to_le_bytes());

    // Subrecords
    let mut offset = 0u16;
    for (i, &wf_type) in waveforms.iter().enumerate() {
        data[18 + i * 3..18 + i * 3 + 2].copy_from_slice(&offset.to_le_bytes());
        data[18 + i * 3 + 2] = wf_type;
        offset += (6 + samples_per_frame * 2) as u16;
    }
    data[18 + waveforms.len() * 3 + 2] = 0xFF; // end marker

    // Waveform data
    let mut data_offset = HEADER_SIZE;
    for &wf_type in waveforms {
        // Waveform header (6 bytes)
        data[data_offset..data_offset + 2]
            .copy_from_slice(&(samples_per_frame as u16).to_le_bytes());
        data[data_offset + 2..data_offset + 4].copy_from_slice(&0u16.to_le_bytes()); // status
        data_offset += 6;

        // Generate samples based on waveform type
        for i in 0..samples_per_frame {
            let sample = match wf_type {
                1 => generate_ecg_sample(phase, hr),   // ECG1
                8 => generate_pleth_sample(phase, hr), // PLETH
                9 => generate_co2_sample(phase, hr),   // CO2
                _ => 0,
            };
            data[data_offset..data_offset + 2].copy_from_slice(&sample.to_le_bytes());
            data_offset += 2;

            *phase += 0.01;
        }
    }

    data
}

fn generate_ecg_sample(phase: &f64, hr: f64) -> i16 {
    let freq = hr / 60.0; // Hz
    let t = phase * freq;
    let t_mod = t - t.floor();

    // Simplified ECG shape
    let value = if t_mod < 0.1 {
        // P wave
        (t_mod * 10.0).sin() * 200.0
    } else if t_mod < 0.15 {
        0.0
    } else if t_mod < 0.2 {
        // Q wave
        -300.0
    } else if t_mod < 0.25 {
        // R wave
        1500.0
    } else if t_mod < 0.3 {
        // S wave
        -500.0
    } else if t_mod < 0.5 {
        // ST segment
        0.0
    } else if t_mod < 0.65 {
        // T wave
        ((t_mod - 0.5) * 6.67).sin() * 400.0
    } else {
        0.0
    };

    value as i16
}

fn generate_pleth_sample(phase: &f64, hr: f64) -> i16 {
    let freq = hr / 60.0;
    let t = phase * freq;

    // Plethysmograph wave (0-100%)
    let value = 50.0 + 30.0 * (t * 2.0 * std::f64::consts::PI).sin();
    (value * 10.0) as i16 // Scale to 1/10%
}

fn generate_co2_sample(phase: &f64, hr: f64) -> i16 {
    let freq = hr / 60.0 / 4.0; // Slower than HR
    let t = phase * freq;
    let t_mod = t - t.floor();

    // Square-ish wave for CO2
    let value = if t_mod < 0.3 {
        400.0 // FiCO2 (0.4%)
    } else {
        520.0 // EtCO2 (5.2%)
    };

    (value * 100.0) as i16 // Scale to 1/100%
}

fn vary_value(current: f64, target: f64, max_change: f64) -> f64 {
    let diff = target - current;
    let change = (diff / 10.0).clamp(-max_change, max_change);
    current + change + (rand::random::<f64>() - 0.5) * max_change * 0.3
}

fn write_group_header(data: &mut [u8], status: u32) {
    data[0..4].copy_from_slice(&status.to_le_bytes());
    data[4..6].copy_from_slice(&0u16.to_le_bytes()); // label
}

fn write_i16(data: &mut [u8], value: (i16, i16)) {
    let scaled = value.0 * value.1;
    data[0..2].copy_from_slice(&scaled.to_le_bytes());
}

fn send_frame(port: &mut dyn SerialPort, data: &[u8]) -> Result<()> {
    // Calculate checksum on STUFFED data as per GE DRI protocol
    let mut checksum = 0u8;
    let mut stuffed = Vec::new();
    stuffed.push(FRAME_CHAR);

    // Stuff data and calculate checksum on stuffed bytes
    for &byte in data {
        if byte == FRAME_CHAR {
            stuffed.push(CTRL_CHAR);
            stuffed.push(byte & !BIT5);
            // Add BOTH stuffed bytes to checksum
            checksum = checksum.wrapping_add(CTRL_CHAR);
            checksum = checksum.wrapping_add(byte & !BIT5);
        } else if byte == CTRL_CHAR {
            stuffed.push(CTRL_CHAR);
            stuffed.push(byte & !BIT5);
            // Add BOTH stuffed bytes to checksum
            checksum = checksum.wrapping_add(CTRL_CHAR);
            checksum = checksum.wrapping_add(byte & !BIT5);
        } else {
            stuffed.push(byte);
            checksum = checksum.wrapping_add(byte);
        }
    }

    // Now stuff and add the checksum itself
    if checksum == FRAME_CHAR {
        stuffed.push(CTRL_CHAR);
        stuffed.push(checksum & !BIT5);
    } else if checksum == CTRL_CHAR {
        stuffed.push(CTRL_CHAR);
        stuffed.push(checksum & !BIT5);
    } else {
        stuffed.push(checksum);
    }

    stuffed.push(FRAME_CHAR);

    port.write_all(&stuffed)?;
    port.flush()?;

    Ok(())
}
