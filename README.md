# GE DRI Protocol Parser

A Rust implementation for communicating with GE Healthcare patient monitors (CARESCAPE B650/B850, S/5) using the Datex-Ohmeda Record Interface (DRI) protocol over serial RS232.

This tool can capture physiological data (heart rate, SpO2, blood pressure, temperatures, CO2, ventilator parameters, etc.) and waveforms (ECG, plethysmograph, etc.) from compatible monitors.

---

## Building

Build all binaries:
```bash
cargo build --release
```

Binaries will be in `./target/release/`

---

## Binaries

### Main Application

Full-featured data collection with CSV/JSON export and interactive configuration.
```bash
cargo run --bin ge-dri-prototype
```

Or after building:
```bash
./target/release/ge-dri-prototype
```

### Diagnostic Tool

Simple diagnostic mode that auto-starts and logs all received data to console. Useful for testing connectivity.
```bash
cargo run --bin diagnostic
```

### Faker (Simulator)

Simulates a GE monitor for testing without real hardware. Generates fake physiological data and waveforms.
```bash
cargo run --bin faker -- --port COM3
```

Or on Linux:
```bash
cargo run --bin faker -- --port /dev/ttyUSB0
```

---

## Serial Connection

- **Baud rate:** 19200
- **Data bits:** 8
- **Parity:** Even
- **Stop bits:** 1
- **Flow control:** RTS/CTS (hardware)