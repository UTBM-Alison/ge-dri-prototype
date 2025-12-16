//! GE DRI Protocol Parser - Main Application

use anyhow::Result;
use chrono::Local;
use ge_dri_prototype::decode::Decoder;
use ge_dri_prototype::device::SerialDevice;
use ge_dri_prototype::storage::{CsvWriter, JsonWriter, RawWriter};
use ge_dri_prototype::ui;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Display banner
    ui::display_banner();

    // Select serial port
    let port_name = ge_dri_prototype::device::select_port()?;
    ui::success(&format!("Selected port: {}", port_name));

    // Connect to device
    ui::info("Connecting to monitor...");
    let mut device = SerialDevice::open(&port_name)?;
    ui::success("Connected successfully!");

    // Configure data collection
    println!();
    ui::info("=== Data Collection Configuration ===");

    let interval = loop {
        let input = ui::get_input("Update interval in seconds (5-3600)", "10")?;
        if input.is_empty() {
            break 10;
        }
        match input.parse::<u16>() {
            Ok(val) if val >= 5 && val <= 3600 => break val,
            _ => ui::error("Invalid interval. Must be between 5 and 3600 seconds."),
        }
    };

    let waveforms_input = ui::get_input(
        "Waveforms to collect (comma-separated, e.g., ECG1,PLETH,CO2)",
        "ECG1,PLETH",
    )?;

    let waveforms: Vec<String> = if waveforms_input.is_empty() {
        vec!["ECG1".to_string(), "PLETH".to_string()]
    } else {
        waveforms_input
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .collect()
    };

    // Request data from monitor
    ui::info("Requesting data from monitor...");
    device.request_displayed_values(interval)?;

    // Convert String to &str for request_waveforms
    let waveform_refs: Vec<&str> = waveforms.iter().map(|s| s.as_str()).collect();
    device.request_waveforms(&waveform_refs)?;

    ui::success(&format!(
        "Requested displayed values ({}s interval) and waveforms: {}",
        interval,
        waveforms.join(", ")
    ));

    // Initialize storage
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let base_filename = format!("output_{}", timestamp);

    let mut csv_writer = CsvWriter::new(format!("{}.csv", base_filename))?;
    let mut json_writer = JsonWriter::new(format!("{}.json", base_filename))?;
    let mut raw_writer = RawWriter::new(format!("{}.raw", base_filename))?;

    ui::success(&format!(
        "Created output files: {}.{{csv,json,raw}}",
        base_filename
    ));

    // Initialize decoder
    let decoder = Decoder::new();

    // Main collection loop
    println!();
    ui::info("=== Starting Data Collection ===");
    ui::info("Press Ctrl+C to stop");
    println!();

    let mut frame_count = 0;

    loop {
        match device.read_frame() {
            Ok(frame) => {
                // Write raw frame
                raw_writer.write_frame(&frame)?;

                // Parse header from frame data
                let header = match ge_dri_prototype::protocol::DriHeader::parse(&frame.data) {
                    Ok(h) => h,
                    Err(e) => {
                        ui::error(&format!("Failed to parse header: {}", e));
                        continue;
                    }
                };

                // Extract data portion (after header - 40 bytes)
                let data = match header.extract_data(&frame.data) {
                    Ok(d) => d,
                    Err(e) => {
                        ui::error(&format!("Failed to extract data: {}", e));
                        continue;
                    }
                };

                // Decode frame with header and data
                match decoder.decode_frame(&header, data) {
                    Ok(Some(record)) => {
                        frame_count += 1;

                        // Write to storage
                        match &record {
                            ge_dri_prototype::decode::DriRecord::Physiological(phys) => {
                                csv_writer.write_physiological(phys)?;
                                json_writer.write_physiological(phys)?;

                                // Display live vitals
                                print!("\r");

                                // ECG
                                if let Some(hr) = phys.ecg_hr {
                                    print!(
                                        "{} HR: {:.0} bpm",
                                        if phys.ecg_status.active {
                                            "💚"
                                        } else {
                                            "⚪"
                                        },
                                        hr
                                    );
                                }

                                // SpO2
                                if let Some(spo2) = phys.spo2 {
                                    print!(" | SpO2: {:.1}%", spo2);
                                }

                                // Blood Pressure
                                if let Some(sys) = phys.nibp_sys {
                                    if let Some(dia) = phys.nibp_dia {
                                        print!(" | BP: {:.0}/{:.0}", sys, dia);
                                    }
                                }

                                // Temperature
                                if let Some(temp) = phys.temp1 {
                                    print!(" | Temp: {:.1}°C", temp);
                                }

                                // CO2
                                if let Some(etco2) = phys.co2_et {
                                    print!(" | EtCO2: {:.1}%", etco2);
                                }

                                // Ventilator data
                                if phys.flow_status.active {
                                    if let Some(rr) = phys.flow_rr {
                                        print!(" | RR: {:.0}", rr);
                                    }
                                    if let Some(peep) = phys.flow_peep {
                                        print!(" | PEEP: {:.1}", peep);
                                    }
                                    if let Some(tv) = phys.flow_tv_exp {
                                        print!(" | TV: {:.0}ml", tv);
                                    }
                                    if let Some(ppeak) = phys.flow_ppeak {
                                        print!(" | Ppeak: {:.1}", ppeak);
                                    }
                                }

                                // Flush output
                                use std::io::{self, Write};
                                io::stdout().flush()?;
                            }
                            ge_dri_prototype::decode::DriRecord::Waveform { waveforms } => {
                                for wf in waveforms {
                                    csv_writer.write_waveform(wf)?;
                                    json_writer.write_waveform(wf)?;
                                }
                            }
                        }

                        // Show statistics every 100 frames
                        if frame_count % 100 == 0 {
                            println!();
                            ui::success(&format!("📊 Processed {} frames", frame_count));
                            print!("Current vitals: ");
                        }
                    }
                    Ok(None) => {
                        // No data in frame (e.g., unsupported record type)
                    }
                    Err(e) => {
                        ui::error(&format!("Decode error: {}", e));
                    }
                }
            }
            Err(e) => {
                println!();
                ui::error(&format!("Read error: {}", e));

                // Ask user if they want to reconnect
                if ui::confirm("Connection lost. Try to reconnect?")? {
                    ui::info("Attempting to reconnect...");
                    match SerialDevice::open(&port_name) {
                        Ok(new_device) => {
                            device = new_device;
                            device.request_displayed_values(interval)?;

                            // Convert String to &str again
                            let waveform_refs: Vec<&str> =
                                waveforms.iter().map(|s| s.as_str()).collect();
                            device.request_waveforms(&waveform_refs)?;

                            ui::success("Reconnected successfully!");
                        }
                        Err(e) => {
                            ui::error(&format!("Reconnection failed: {}", e));
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }

    // Cleanup
    println!();
    ui::info("Stopping data collection...");
    device.stop_all()?;
    ui::success(&format!(
        "Collection stopped. Total frames: {}",
        frame_count
    ));

    Ok(())
}
