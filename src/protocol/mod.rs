//! DRI protocol layer - framing, headers, and checksum

pub mod checksum;
pub mod framing;
pub mod header;

pub use checksum::validate_checksum;
pub use framing::{DriFrame, FrameParser};
pub use header::DriHeader;
