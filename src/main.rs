use anyhow::Result;
use env_logger::Env;
use log::{error, info, warn};

use ge_dri_prototype::{
    decode::Decoder,
    device::{SerialDevice, select_port},
    storage::{CsvWriter, JsonWriter, RawWriter},
    ui,
};

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Display banner
    ui::display_banner();

    // Step 1: Select serial port
    ui::progress("Scanning for serial ports...");
    let port_name = select_port()?;
    ui::success(&format!("Selected port: {}", port_name));

    // Step 2: Open serial device
    ui::progress("Opening serial connection...");
    let mut device = SerialDevice::open(&port_name)?;
    ui::success("Serial port opened successfully");

    // Step 3: Configure data requests
    println!("\n📊 Configuration:");
    println!("─────────────────────────────────────────────────────────");

    let interval = ui::get_input("Update interval for displayed values (seconds, min 5)", "5")?
        .parse::<u16>()
        .unwrap_or(5)
        .max(5);

    let waveforms_str = ui::get_input(
        "Waveforms to request (comma-separated, e.g., ECG1,PLETH)",
        "ECG1,PLETH",
    )?;

    let waveforms: Vec<&str> = waveforms_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Step 4: Request data
    ui::progress("Requesting data from monitor...");
    device.request_displayed_values(interval)?;

    if !waveforms.is_empty() {
        device.request_waveforms(&waveforms)?;
        ui::success(&format!(
            "Requesting: {} (every {} sec) + waveforms: {}",
            "Displayed values",
            interval,
            waveforms.join(", ")
        ));
    } else {
        ui::success(&format!(
            "Requesting: {} (every {} sec)",
            "Displayed values", interval
        ));
    }

    // Step 5: Setup storage
    ui::progress("Initializing storage...");
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

    let csv_path = format!("output_{}.csv", timestamp);
    let json_path = format!("output_{}.json", timestamp);
    let raw_path = format!("output_{}.raw", timestamp);

    let mut csv_writer = CsvWriter::new(&csv_path)?;
    let mut json_writer = JsonWriter::new(&json_path)?;
    let mut raw_writer = RawWriter::new(&raw_path)?;

    ui::success(&format!(
        "Output files: {}, {}, {}",
        csv_path, json_path, raw_path
    ));

    // Step 6: Create decoder
    let mut decoder = Decoder::new();

    // Step 7: Main data collection loop
    println!("\n📡 Data Collection:");
    println!("─────────────────────────────────────────────────────────");
    ui::info("Press Ctrl+C to stop...\n");

    let mut frame_count = 0;
    let mut phys_count = 0;
    let mut wave_count = 0;
    let start_time = std::time::Instant::now();

    loop {
        match device.read_frame() {
            Ok(frame) => {
                frame_count += 1;

                // Save raw data
                if let Err(e) = raw_writer.write(&frame.complete_data()) {
                    error!("Failed to write raw data: {}", e);
                }

                // Decode and process
                match decoder.decode_frame(&frame) {
                    Ok(Some(record)) => {
                        match record {
                            ge_dri_prototype::decode::DriRecord::Physiological(phys) => {
                                phys_count += 1;

                                // Write to CSV and JSON
                                let _ = csv_writer.write_physiological(&phys);
                                let _ = json_writer.write_physiological(&phys);

                                // Display key vitals
                                let hr = phys
                                    .ecg_hr
                                    .map(|v| format!("{}", v))
                                    .unwrap_or("--".to_string());
                                let spo2 = phys
                                    .spo2
                                    .map(|v| format!("{}", v))
                                    .unwrap_or("--".to_string());
                                let nibp = format!(
                                    "{}/{}",
                                    phys.nibp_sys
                                        .map(|v| format!("{}", v))
                                        .unwrap_or("--".to_string()),
                                    phys.nibp_dia
                                        .map(|v| format!("{}", v))
                                        .unwrap_or("--".to_string())
                                );

                                info!(
                                    "Frame {:4} | HR: {:>3} | SpO2: {:>3} | BP: {:>7} | Time: {:?}",
                                    frame_count,
                                    hr,
                                    spo2,
                                    nibp,
                                    phys.timestamp.format("%H:%M:%S")
                                );
                            }
                            ge_dri_prototype::decode::DriRecord::Waveform(wave) => {
                                wave_count += 1;

                                // Write to CSV and JSON
                                let _ = csv_writer.write_waveform(&wave);
                                let _ = json_writer.write_waveform(&wave);

                                info!(
                                    "Frame {:4} | Waveform: {:?} | Samples: {} | Rate: {} Hz",
                                    frame_count,
                                    wave.waveform_type.name(),
                                    wave.samples.len(),
                                    wave.sample_rate
                                );
                            }
                        }
                    }
                    Ok(None) => {
                        // Frame decoded but no complete record yet
                    }
                    Err(e) => {
                        warn!("Failed to decode frame {}: {}", frame_count, e);
                    }
                }

                // Periodic statistics
                if frame_count % 100 == 0 {
                    let elapsed = start_time.elapsed().as_secs();
                    println!("\n📈 Statistics after {} seconds:", elapsed);
                    println!(
                        "   Frames: {} | Physiological: {} | Waveforms: {}",
                        frame_count, phys_count, wave_count
                    );
                    println!();
                }
            }
            Err(e) => {
                error!("Failed to read frame: {}", e);

                if ui::confirm("Connection error. Try to reconnect?")? {
                    ui::progress("Attempting to reconnect...");
                    device = SerialDevice::open(&port_name)?;
                    device.request_displayed_values(interval)?;
                    if !waveforms.is_empty() {
                        device.request_waveforms(&waveforms)?;
                    }
                    ui::success("Reconnected successfully");
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}
