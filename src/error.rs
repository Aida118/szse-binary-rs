// src/error.rs
use std::fmt;

/// Errors that can occur while parsing an SZSE Binary message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Buffer is shorter than the field/message requires.
    BufferTooShort { needed: usize, got: usize },
    /// `MsgType` in the header is not handled by this parser.
    UnknownMsgType(u32),
    /// A single-byte enum field held a value outside the spec.
    ///
    /// `field` names the field (e.g. `"Side"`), `value` is the raw byte.
    InvalidEnum { field: &'static str, value: u8 },
    /// `BodyLength` in the header disagrees with the buffer / message layout.
    BodyLengthMismatch { declared: usize, actual: usize },
    /// Checksum tail did not match the computed value.
    ChecksumMismatch { declared: u32, computed: u32 },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::BufferTooShort { needed, got } => {
                write!(f, "buffer too short: need {needed} bytes, got {got}")
            }
            ParseError::UnknownMsgType(t) => write!(f, "unknown message type: {t}"),
            ParseError::InvalidEnum { field, value } => {
                write!(f, "invalid value {value:#04x} for field {field}")
            }
            ParseError::BodyLengthMismatch { declared, actual } => {
                write!(
                    f,
                    "body length mismatch: header says {declared}, layout needs {actual}"
                )
            }
            ParseError::ChecksumMismatch { declared, computed } => {
                write!(
                    f,
                    "checksum mismatch: message has {declared}, computed {computed}"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
