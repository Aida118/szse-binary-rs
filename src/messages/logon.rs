// src/messages/logon.rs
use crate::error::ParseError;
use crate::utils::{be_i32, fixed, require_len, trimmed_str};

/// Logon message (登录, MsgType=1, `T_LOGINV5`).
///
/// Sent by the client (VSS) as the first message after the TCP connect;
/// the gateway (MDGW) echoes a Logon to confirm, or replies with a
/// [`crate::Logout`] to reject (§2.2.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logon {
    /// Sender code, client-chosen (发送方代码).
    pub sender_comp_id: [u8; 20],
    /// Target code = gateway id (接收方代码).
    pub target_comp_id: [u8; 20],
    /// Heartbeat interval in seconds (心跳间隔).
    pub heart_bt_int: i32,
    /// Password (密码).
    pub password: [u8; 16],
    /// Binary protocol version, e.g. `"1.00"` (二进制协议版本).
    pub default_appl_ver_id: [u8; 32],
}

/// Wire size of the Logon body: `20 + 20 + 4 + 16 + 32 = 92`.
pub const LOGON_BODY_LEN: usize = 92;

impl Logon {
    /// Parse a `Logon` from the body bytes (after the 8-byte header).
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, LOGON_BODY_LEN)?;
        Ok(Logon {
            sender_comp_id: fixed(buf, 0),
            target_comp_id: fixed(buf, 20),
            heart_bt_int: be_i32(buf, 40),
            password: fixed(buf, 44),
            default_appl_ver_id: fixed(buf, 60),
        })
    }

    pub fn sender_comp_id_str(&self) -> &str {
        trimmed_str(&self.sender_comp_id)
    }
    pub fn target_comp_id_str(&self) -> &str {
        trimmed_str(&self.target_comp_id)
    }
    pub fn password_str(&self) -> &str {
        trimmed_str(&self.password)
    }
    pub fn default_appl_ver_id_str(&self) -> &str {
        trimmed_str(&self.default_appl_ver_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Right-pad `src` with spaces into a fixed `[u8; N]`, matching how
    /// SZSE pads string fields on the wire.
    fn padded<const N: usize>(src: &[u8]) -> [u8; N] {
        let mut a = [b' '; N];
        a[..src.len()].copy_from_slice(src);
        a
    }

    #[test]
    fn test_logon_parse() {
        let mut buf = vec![0u8; LOGON_BODY_LEN];
        buf[0..20].copy_from_slice(&padded::<20>(b"TEST_SENDER"));
        buf[20..40].copy_from_slice(&padded::<20>(b"MDGW001"));
        buf[40..44].copy_from_slice(&60i32.to_be_bytes());
        buf[44..60].copy_from_slice(&padded::<16>(b"mypassword"));
        buf[60..92].copy_from_slice(&padded::<32>(b"1.00"));

        let logon = Logon::parse(&buf).unwrap();
        assert_eq!(logon.sender_comp_id_str(), "TEST_SENDER");
        assert_eq!(logon.target_comp_id_str(), "MDGW001");
        assert_eq!(logon.heart_bt_int, 60);
        assert_eq!(logon.password_str(), "mypassword");
        assert_eq!(logon.default_appl_ver_id_str(), "1.00");
    }

    #[test]
    fn rejects_short_buffer() {
        assert!(matches!(
            Logon::parse(&[0u8; LOGON_BODY_LEN - 1]),
            Err(ParseError::BufferTooShort { .. })
        ));
    }
}
