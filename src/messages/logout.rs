// src/messages/logout.rs
use crate::error::ParseError;
use crate::utils::require_len;

/// 注销消息 (MsgType=2)
#[derive(Debug, Clone, PartialEq)]
pub struct Logout {
    pub session_status: i32,
    pub text: [u8; 200],
}

pub const LOGOUT_BODY_LEN: usize = 4 + 200; // 204

impl Logout {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, LOGOUT_BODY_LEN)?;
        let session_status = i32::from_be_bytes(buf[0..4].try_into().unwrap());
        let mut text = [0u8; 200];
        text.copy_from_slice(&buf[4..204]);
        Ok(Logout { session_status, text })
    }

    pub fn text_str(&self) -> &str {
        std::str::from_utf8(&self.text).unwrap_or("").trim_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logout_parse() {
        let mut buf = vec![0u8; LOGOUT_BODY_LEN];
        buf[0..4].copy_from_slice(&0i32.to_be_bytes());
        buf[4..204].copy_from_slice(b"Normal logout                                          ");
        let logout = Logout::parse(&buf).unwrap();
        assert_eq!(logout.session_status, 0);
        assert_eq!(logout.text_str(), "Normal logout");
    }
}