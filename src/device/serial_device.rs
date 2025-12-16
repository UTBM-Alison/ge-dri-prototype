//! Serial device communication with GE monitors

use crate::Result;
use crate::constants::WaveformType;
use crate::constants::dri_types::PHDBCL_REQ_ALL;
use crate::protocol::framing::create_frame;
use crate::protocol::header::{create_phdb_request, create_waveform_request};
use crate::protocol::{DriFrame, FrameParser};
use log::{debug, info, warn};
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

/// Waveform request types
const WF_REQ_CONT_START: u16 = 0;
const WF_REQ_CONT_STOP: u16 = 1;

/// Serial device connected to a GE monitor
pub struct SerialDevice {
    port: Box<dyn SerialPort>,
    parser: FrameParser,
}

impl SerialDevice {
    /// Open a serial port connection to a GE monitor
    ///
    /// # Arguments
    /// * `port_name` - Serial port name (e.g., "/dev/ttyUSB0" or "COM3")
    ///
    /// # GE Monitor Serial Settings
    /// - Baud rate: 19200
    /// - Data bits: 8
    /// - Parity: Even
    /// - Stop bits: 1
    /// - Flow control: RTS/CTS
    pub fn open(port_name: &str) -> Result<Self> {
        info!("Opening serial port: {}", port_name);

        let port = serialport::new(port_name, 19200)
            .timeout(Duration::from_millis(1000))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::Even)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::Hardware)
            .open()?;

        info!("Serial port opened successfully");

        Ok(Self {
            port,
            parser: FrameParser::new(),
        })
    }

    /// Request displayed values (current physiological data)
    ///
    /// # Arguments
    /// * `interval` - Update interval in seconds (minimum 5)
    pub fn request_displayed_values(&mut self, interval: u16) -> Result<()> {
        let interval = interval.max(5); // Minimum 5 seconds

        info!("Requesting displayed values every {} seconds", interval);

        let header = create_phdb_request(
            1, // DRI_PH_DISPL
            interval,
            PHDBCL_REQ_ALL,
        );

        let frame = create_frame(&header);
        self.write_frame(&frame)?;

        Ok(())
    }

    /// Request 60-second trended values
    pub fn request_trend_60s(&mut self) -> Result<()> {
        info!("Requesting 60-second trend values");

        let header = create_phdb_request(
            3, // DRI_PH_60S_TREND
            1, // Interval (positive, but exact value doesn't matter for trends)
            PHDBCL_REQ_ALL,
        );

        let frame = create_frame(&header);
        self.write_frame(&frame)?;

        Ok(())
    }

    /// Request waveform data
    ///
    /// # Arguments
    /// * `waveform_names` - Array of waveform names (e.g., ["ECG1", "PLETH"])
    ///
    /// # Sample Rate Limit
    /// Total sample rate must not exceed 600 samples/second
    pub fn request_waveforms(&mut self, waveform_names: &[&str]) -> Result<()> {
        // Convert names to WaveformType
        let waveforms: Vec<WaveformType> = waveform_names
            .iter()
            .filter_map(|name| self.parse_waveform_name(name))
            .collect();

        if waveforms.is_empty() {
            anyhow::bail!("No valid waveforms specified");
        }

        // Validate sample rate
        crate::constants::waveforms::validate_waveform_set(&waveforms)?;

        info!("Requesting waveforms: {:?}", waveform_names);

        // Convert to u8 values
        let waveform_types: Vec<u8> = waveforms.iter().map(|wf| *wf as u8).collect();

        let header = create_waveform_request(&waveform_types, WF_REQ_CONT_START);
        let frame = create_frame(&header);
        self.write_frame(&frame)?;

        Ok(())
    }

    /// Stop waveform transmission
    pub fn stop_waveforms(&mut self) -> Result<()> {
        info!("Stopping waveform transmission");

        let header = create_waveform_request(&[], WF_REQ_CONT_STOP);
        let frame = create_frame(&header);
        self.write_frame(&frame)?;

        Ok(())
    }

    /// Stop all data transmission
    pub fn stop_all(&mut self) -> Result<()> {
        info!("Stopping all data transmission");

        // Stop displayed values
        let header = create_phdb_request(1, 0, 0);
        let frame = create_frame(&header);
        self.write_frame(&frame)?;

        // Stop trends
        let header = create_phdb_request(3, 0, 0);
        let frame = create_frame(&header);
        self.write_frame(&frame)?;

        // Stop waveforms
        self.stop_waveforms()?;

        Ok(())
    }

    /// Read one complete frame from the device
    ///
    /// This will block until a complete frame is received or timeout occurs
    pub fn read_frame(&mut self) -> Result<DriFrame> {
        let mut buffer = [0u8; 2048];

        loop {
            match self.port.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        continue;
                    }

                    debug!("Read {} bytes from serial port", bytes_read);

                    let frames = self.parser.process_bytes(&buffer[..bytes_read])?;

                    if !frames.is_empty() {
                        return Ok(frames[0].clone());
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Timeout is normal, just continue
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    /// Try to read a frame without blocking (non-blocking read)
    pub fn try_read_frame(&mut self) -> Result<Option<DriFrame>> {
        let mut buffer = [0u8; 2048];

        // Set a very short timeout for non-blocking behavior
        self.port.set_timeout(Duration::from_millis(10))?;

        match self.port.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return Ok(None);
                }

                let frames = self.parser.process_bytes(&buffer[..bytes_read])?;

                Ok(frames.into_iter().next())
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Write a frame to the device
    fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        debug!("Writing {} bytes to serial port", frame.len());
        self.port.write_all(frame)?;
        self.port.flush()?;
        Ok(())
    }

    /// Parse waveform name to WaveformType
    fn parse_waveform_name(&self, name: &str) -> Option<WaveformType> {
        match name.to_uppercase().as_str() {
            "ECG1" => Some(WaveformType::Ecg1),
            "ECG2" => Some(WaveformType::Ecg2),
            "ECG3" => Some(WaveformType::Ecg3),
            "PLETH" => Some(WaveformType::Pleth),
            "PLETH2" => Some(WaveformType::Pleth2),
            "CO2" => Some(WaveformType::Co2),
            "O2" => Some(WaveformType::O2),
            "N2O" => Some(WaveformType::N2o),
            "AA" => Some(WaveformType::Aa),
            "INVP1" => Some(WaveformType::Invp1),
            "INVP2" => Some(WaveformType::Invp2),
            "INVP3" => Some(WaveformType::Invp3),
            "INVP4" => Some(WaveformType::Invp4),
            "INVP5" => Some(WaveformType::Invp5),
            "INVP6" => Some(WaveformType::Invp6),
            "INVP7" => Some(WaveformType::Invp7),
            "INVP8" => Some(WaveformType::Invp8),
            "AWP" => Some(WaveformType::Awp),
            "FLOW" => Some(WaveformType::Flow),
            "RESP" => Some(WaveformType::Resp),
            "EEG1" => Some(WaveformType::Eeg1),
            "EEG2" => Some(WaveformType::Eeg2),
            "EEG3" => Some(WaveformType::Eeg3),
            "EEG4" => Some(WaveformType::Eeg4),
            "ENT_100" | "ENT100" => Some(WaveformType::Ent100),
            _ => {
                warn!("Unknown waveform name: {}", name);
                None
            }
        }
    }

    /// Get port name
    pub fn port_name(&self) -> Result<String> {
        Ok(self.port.name().unwrap_or_else(|| "Unknown".to_string()))
    }

    /// Clear the parser buffer (useful after errors)
    pub fn reset_parser(&mut self) {
        self.parser.reset();
    }
}

impl Drop for SerialDevice {
    fn drop(&mut self) {
        info!("Closing serial device");
        let _ = self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_waveform_names() {
        let device = SerialDevice {
            port: serialport::new("dummy", 19200)
                .timeout(Duration::from_millis(1))
                .open()
                .unwrap(),
            parser: FrameParser::new(),
        };

        assert_eq!(device.parse_waveform_name("ECG1"), Some(WaveformType::Ecg1));
        assert_eq!(device.parse_waveform_name("ecg1"), Some(WaveformType::Ecg1));
        assert_eq!(
            device.parse_waveform_name("PLETH"),
            Some(WaveformType::Pleth)
        );
        assert_eq!(device.parse_waveform_name("INVALID"), None);
    }
}
