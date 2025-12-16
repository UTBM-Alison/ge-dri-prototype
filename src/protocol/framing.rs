//! Frame parsing and byte stuffing/unstuffing for DRI protocol

use crate::DriError;
use crate::constants::{BIT5, CTRL_CHAR, FRAME_CHAR};
use log::{debug, trace};

/// A complete DRI frame with unstuffed data
#[derive(Debug, Clone)]
pub struct DriFrame {
    /// Raw unstuffed data (without frame characters and checksum)
    pub data: Vec<u8>,
    /// Checksum byte
    pub checksum: u8,
}

impl DriFrame {
    /// Create a new frame from unstuffed data and checksum
    pub fn new(data: Vec<u8>, checksum: u8) -> Self {
        Self { data, checksum }
    }

    /// Get the complete frame data including checksum
    pub fn complete_data(&self) -> Vec<u8> {
        let mut result = self.data.clone();
        result.push(self.checksum);
        result
    }

    /// Validate the frame checksum
    pub fn validate(&self) -> bool {
        super::checksum::validate_checksum(&self.complete_data())
    }
}

/// State machine for parsing DRI frames from a byte stream
#[derive(Debug)]
pub struct FrameParser {
    state: ParserState,
    buffer: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    /// Waiting for frame start (0x7E)
    WaitingForStart,
    /// Inside frame, collecting data
    InFrame,
    /// Next byte needs unstuffing (after 0x7D)
    NeedUnstuff,
}

impl FrameParser {
    /// Create a new frame parser
    pub fn new() -> Self {
        Self {
            state: ParserState::WaitingForStart,
            buffer: Vec::with_capacity(2048),
        }
    }

    /// Reset the parser state
    pub fn reset(&mut self) {
        self.state = ParserState::WaitingForStart;
        self.buffer.clear();
    }

    /// Process a single byte, potentially returning a complete frame
    ///
    /// Returns:
    /// - Ok(Some(frame)) if a complete frame was parsed
    /// - Ok(None) if more data is needed
    /// - Err if an error occurred
    pub fn process_byte(&mut self, byte: u8) -> Result<Option<DriFrame>, DriError> {
        trace!("Parser state: {:?}, byte: 0x{:02X}", self.state, byte);

        match self.state {
            ParserState::WaitingForStart => {
                if byte == FRAME_CHAR {
                    debug!("Frame start detected");
                    self.state = ParserState::InFrame;
                    self.buffer.clear();
                }
                Ok(None)
            }

            ParserState::InFrame => {
                if byte == FRAME_CHAR {
                    // End of frame
                    debug!("Frame end detected, buffer size: {}", self.buffer.len());
                    return self.finalize_frame();
                } else if byte == CTRL_CHAR {
                    // Next byte needs unstuffing
                    self.state = ParserState::NeedUnstuff;
                    Ok(None)
                } else {
                    // Normal data byte
                    self.buffer.push(byte);
                    Ok(None)
                }
            }

            ParserState::NeedUnstuff => {
                // Unstuff the byte by ORing with BIT5
                let unstuffed = byte | BIT5;
                trace!("Unstuffing: 0x{:02X} -> 0x{:02X}", byte, unstuffed);
                self.buffer.push(unstuffed);
                self.state = ParserState::InFrame;
                Ok(None)
            }
        }
    }

    /// Process multiple bytes
    pub fn process_bytes(&mut self, bytes: &[u8]) -> Result<Vec<DriFrame>, DriError> {
        let mut frames = Vec::new();

        for &byte in bytes {
            if let Some(frame) = self.process_byte(byte)? {
                frames.push(frame);
            }
        }

        Ok(frames)
    }

    /// Finalize the current frame
    fn finalize_frame(&mut self) -> Result<Option<DriFrame>, DriError> {
        if self.buffer.is_empty() {
            debug!("Empty frame, ignoring");
            self.state = ParserState::WaitingForStart;
            return Ok(None);
        }

        if self.buffer.len() < 2 {
            debug!("Frame too short ({}), ignoring", self.buffer.len());
            self.state = ParserState::WaitingForStart;
            return Err(DriError::IncompleteFrame);
        }

        // Last byte is checksum
        let checksum = self.buffer.pop().unwrap();
        let data = self.buffer.clone();

        let frame = DriFrame::new(data, checksum);

        // Validate checksum
        if !frame.validate() {
            debug!("Checksum validation failed");
            self.state = ParserState::WaitingForStart;
            return Err(DriError::ChecksumError);
        }

        debug!("Valid frame parsed, size: {}", frame.data.len());
        self.state = ParserState::WaitingForStart;
        Ok(Some(frame))
    }

    /// Get the current buffer size (for debugging)
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Stuff bytes for transmission (escape FRAME_CHAR and CTRL_CHAR)
pub fn stuff_bytes(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() + 10);

    for &byte in data {
        if byte == FRAME_CHAR || byte == CTRL_CHAR {
            result.push(CTRL_CHAR);
            result.push(byte & !BIT5);
        } else {
            result.push(byte);
        }
    }

    result
}

/// Create a complete frame ready for transmission
pub fn create_frame(data: &[u8]) -> Vec<u8> {
    let checksum = super::checksum::calculate_checksum(data);

    let mut payload = data.to_vec();
    payload.push(checksum);

    let stuffed = stuff_bytes(&payload);

    let mut frame = Vec::with_capacity(stuffed.len() + 2);
    frame.push(FRAME_CHAR);
    frame.extend_from_slice(&stuffed);
    frame.push(FRAME_CHAR);

    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_frame() {
        let mut parser = FrameParser::new();

        // Simple frame: 0x7E 0x01 0x02 0x03 0x06 0x7E
        // Checksum = 0x01 + 0x02 + 0x03 = 0x06
        let bytes = vec![0x7E, 0x01, 0x02, 0x03, 0x06, 0x7E];

        let frames = parser.process_bytes(&bytes).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data, vec![0x01, 0x02, 0x03]);
        assert_eq!(frames[0].checksum, 0x06);
        assert!(frames[0].validate());
    }

    #[test]
    fn test_byte_stuffing() {
        let mut parser = FrameParser::new();

        // Frame with 0x7E in data: 0x7E 0x7D 0x5E 0x01 <checksum> 0x7E
        // 0x7D 0x5E unstuffs to 0x7E
        let data = vec![0x7E];
        let checksum = super::super::checksum::calculate_checksum(&data);

        let bytes = vec![0x7E, 0x7D, 0x5E, checksum, 0x7E];

        let frames = parser.process_bytes(&bytes).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data, vec![0x7E]);
    }

    #[test]
    fn test_invalid_checksum() {
        let mut parser = FrameParser::new();

        // Frame with wrong checksum
        let bytes = vec![0x7E, 0x01, 0x02, 0x03, 0xFF, 0x7E];

        let result = parser.process_bytes(&bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DriError::ChecksumError));
    }

    #[test]
    fn test_multiple_frames() {
        let mut parser = FrameParser::new();

        // Two frames back to back
        let bytes = vec![
            0x7E, 0x01, 0x01, 0x7E, // First frame
            0x7E, 0x02, 0x02, 0x7E, // Second frame
        ];

        let frames = parser.process_bytes(&bytes).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].data, vec![0x01]);
        assert_eq!(frames[1].data, vec![0x02]);
    }

    #[test]
    fn test_create_frame() {
        let data = vec![0x01, 0x02, 0x03];
        let frame = create_frame(&data);

        // Should be: 0x7E <data> <checksum> 0x7E
        assert_eq!(frame[0], FRAME_CHAR);
        assert_eq!(frame[frame.len() - 1], FRAME_CHAR);

        // Parse it back
        let mut parser = FrameParser::new();
        let parsed = parser.process_bytes(&frame).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].data, data);
    }
}
