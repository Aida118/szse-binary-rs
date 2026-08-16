// src/header.rs
use crate::error::ParseError;
use crate::utils::require_len;

/// 消息头 (8字节)
#[derive(Debug, Clone, PartialEq)]
pub struct MsgHeader {
    pub msg_type: u32,
    pub body_length: u32,
}

pub const MSG_HEADER_LEN: usize = 8;

impl MsgHeader {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, MSG_HEADER_LEN)?;
        Ok(MsgHeader {
            msg_type: u32::from_be_bytes(buf[0..4].try_into().unwrap()),
            body_length: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_too_short() {
        let buf = [0u8; 5];
        assert_eq!(
            MsgHeader::parse(&buf),
            Err(ParseError::BufferTooShort { needed: 8, got: 5 })
        );
    }

    #[test]
    fn header_parses_msg_type() {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&300191u32.to_be_bytes());
        buf[4..8].copy_from_slice(&66u32.to_be_bytes());
        let h = MsgHeader::parse(&buf).unwrap();
        assert_eq!(h.msg_type, 300191);
        assert_eq!(h.body_length, 66);
    }
}
