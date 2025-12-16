//! Interactive serial port selection

use crate::Result;
use dialoguer::Select;
use serialport::SerialPortInfo;

/// Interactively select a serial port from available ports
pub fn select_port() -> Result<String> {
    let ports = serialport::available_ports()?;

    if ports.is_empty() {
        anyhow::bail!("No serial ports found! Please check your connections.");
    }

    println!("\n🔌 Available Serial Ports:");
    println!("─────────────────────────────────────────────────────────");

    let port_descriptions: Vec<String> = ports.iter().map(|p| format_port_info(p)).collect();

    let selection = Select::new()
        .with_prompt("Select the serial port connected to your GE monitor")
        .items(&port_descriptions)
        .default(0)
        .interact()?;

    Ok(ports[selection].port_name.clone())
}

/// Format port information for display
fn format_port_info(port: &SerialPortInfo) -> String {
    let port_name = &port.port_name;

    match &port.port_type {
        serialport::SerialPortType::UsbPort(usb_info) => {
            let manufacturer = usb_info.manufacturer.as_deref().unwrap_or("Unknown");
            let product = usb_info.product.as_deref().unwrap_or("Unknown");
            let serial = usb_info.serial_number.as_deref().unwrap_or("");

            format!(
                "{:<20} │ USB: {} - {} {} (VID:{:04X} PID:{:04X})",
                port_name,
                manufacturer,
                product,
                if !serial.is_empty() {
                    format!("[{}]", serial)
                } else {
                    String::new()
                },
                usb_info.vid,
                usb_info.pid
            )
        }
        serialport::SerialPortType::PciPort => {
            format!("{:<20} │ PCI Device", port_name)
        }
        serialport::SerialPortType::BluetoothPort => {
            format!("{:<20} │ Bluetooth Device", port_name)
        }
        serialport::SerialPortType::Unknown => {
            format!("{:<20} │ Unknown Device", port_name)
        }
    }
}

/// List all available ports (for debugging)
pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
    Ok(serialport::available_ports()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        // This test just ensures the function doesn't panic
        let result = list_ports();
        assert!(result.is_ok());
    }
}
