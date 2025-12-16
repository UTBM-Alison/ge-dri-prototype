use anyhow::Result;
use env_logger::Env;
use log::{error, info};

use ge_dri_prototype::{
    decode::Decoder,
    device::{SerialDevice, select_port},
    storage::{CsvWriter, RawWriter},
};

fn main() -> Result<()> {
    // Initialize logger (RUST_LOG=debug for verbose output)
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("GE DRI Protocol Parser - Prototype v0.1.0");
    info!("==========================================");

    // Step 1: Select serial port interactively
    let port_name = select_port()?;
    info!("Selected port: {}", port_name);

    // Step 2: Open serial device
    let mut device = SerialDevice::open(&port_name)?;
    info!("Serial port opened successfully");

    // Step 3: Request data from monitor
    info!("Requesting data transmission...");
    device.request_displayed_values(5)?; // Every 5 seconds
    device.request_waveforms(&["ECG1", "PLETH"])?;
    info!("Data transmission requested");

    // Step 4: Setup storage
    let mut csv_writer = CsvWriter::new("output.csv")?;
    let mut raw_writer = RawWriter::new("output.raw")?;
    info!("Storage initialized: output.csv, output.raw");

    // Step 5: Create decoder
    let mut decoder = Decoder::new();

    // Step 6: Main data collection loop
    info!("Starting data collection (Press Ctrl+C to stop)...");
    info!("==========================================");

    let mut frame_count = 0;
    loop {
        match device.read_frame() {
            Ok(frame) => {
                frame_count += 1;

                // Save raw data
                if let Err(e) = raw_writer.write(&frame) {
                    error!("Failed to write raw data: {}", e);
                }

                // Decode and process
                match decoder.decode_frame(&frame) {
                    Ok(Some(record)) => match record {
                        ge_dri_prototype::decode::DriRecord::Physiological(phys) => {
                            info!(
                                "Frame {}: Physiological data - HR: {:?}, SpO2: {:?}",
                                frame_count, phys.ecg_hr, phys.spo2
                            );

                            if let Err(e) = csv_writer.write_physiological(&phys) {
                                error!("Failed to write CSV: {}", e);
                            }
                        }
                        ge_dri_prototype::decode::DriRecord::Waveform(wave) => {
                            info!(
                                "Frame {}: Waveform data - Type: {:?}, Samples: {}",
                                frame_count,
                                wave.waveform_type,
                                wave.samples.len()
                            );

                            if let Err(e) = csv_writer.write_waveform(&wave) {
                                error!("Failed to write CSV: {}", e);
                            }
                        }
                    },
                    Ok(None) => {
                        // Frame decoded but no complete record yet
                    }
                    Err(e) => {
                        error!("Failed to decode frame {}: {}", frame_count, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read frame: {}", e);
                // Optionally: implement reconnection logic here
            }
        }
    }
}
