//! Device communication module

pub mod port_selector;
pub mod serial_device;

pub use port_selector::select_port;
pub use serial_device::SerialDevice;
