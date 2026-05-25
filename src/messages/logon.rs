// src/messages/logon.rs
use crate::error::ParseError;
use crate::utils::require_len;

/// 登录请求/响应消息 (MsgType=1)
#[derive(Debug, Clone, PartialEq)]
pub struct Logon {
    pub sender_comp_id: [u8; 20],
    pub target_comp_id: [u8; 20],
    pub heart_bt_int: i32,
    pub password: [u8; 16],
    pub default_appl_ver_id: [u8; 32],
}

pub const LOGON_BODY_LEN: usize = 20 + 20 + 4 + 16 + 32; // 92

impl Logon {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, LOGON_BODY_LEN)?;
        let mut sender = [0u8; 20];
        sender.copy_from_slice(&buf[0..20]);
        let mut target = [0u8; 20];
        target.copy_from_slice(&buf[20..40]);
        let heart_bt_int = i32::from_be_bytes(buf[40..44].try_into().unwrap());
        let mut password = [0u8; 16];
        password.copy_from_slice(&buf[44..60]);
        let mut ver_id = [0u8; 32];
        ver_id.copy_from_slice(&buf[60..92]);
        Ok(Logon {
            sender_comp_id: sender,
            target_comp_id: target,
            heart_bt_int,
            password,
            default_appl_ver_id: ver_id,
        })
    }

    pub fn sender_comp_id_str(&self) -> &str {
        std::str::from_utf8(&self.sender_comp_id).unwrap_or("").trim_end()
    }
    pub fn target_comp_id_str(&self) -> &str {
        std::str::from_utf8(&self.target_comp_id).unwrap_or("").trim_end()
    }
    pub fn password_str(&self) -> &str {
        std::str::from_utf8(&self.password).unwrap_or("").trim_end()
    }
    pub fn default_appl_ver_id_str(&self) -> &str {
        std::str::from_utf8(&self.default_appl_ver_id).unwrap_or("").trim_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logon_parse() {
        let mut buf = vec![0u8; LOGON_BODY_LEN];
        buf[0..20].copy_from_slice(b"TEST_SENDER          ");
        buf[20..40].copy_from_slice(b"MDGW001              ");
        buf[40..44].copy_from_slice(&60i32.to_be_bytes());
        buf[44..60].copy_from_slice(b"mypassword      ");
        buf[60..92].copy_from_slice(b"1.00                                ");
        let logon = Logon::parse(&buf).unwrap();
        assert_eq!(logon.sender_comp_id_str(), "TEST_SENDER");
        assert_eq!(logon.target_comp_id_str(), "MDGW001");
        assert_eq!(logon.heart_bt_int, 60);
        assert_eq!(logon.password_str(), "mypassword");
        assert_eq!(logon.default_appl_ver_id_str(), "1.00");
    }
}