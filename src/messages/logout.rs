// src/messages/logout.rs
use crate::error::ParseError;
use crate::utils::{be_i32, fixed, require_len, trimmed_str};

/// Logout message (注销, MsgType=2, `T_LOGOUTV5`).
///
/// Sent by either side to end the session, or by the gateway to reject a
/// Logon. `session_status` carries the reason (§4.2.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logout {
    /// Session status / logout reason (退出时的会话状态).
    pub session_status: i32,
    /// Free-text detail (文本).
    pub text: [u8; 200],
}

/// Wire size of the Logout body: `4 + 200 = 204`.
pub const LOGOUT_BODY_LEN: usize = 204;

/// Decoded meaning of [`Logout::session_status`] (§4.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,                  // 0 会话活跃
    PasswordChanged,         // 1 会话口令已更改
    PasswordExpiring,        // 2 将过期的会话口令
    NewPasswordNonCompliant, // 3 新会话口令不符合规范
    LogoutComplete,          // 4 会话退登完成
    InvalidUserOrPassword,   // 5 不合法的用户名或口令
    AccountLocked,           // 6 账户锁定
    LoginNotAllowedNow,      // 7 当前时间不允许登录
    PasswordExpired,         // 8 口令过期
    MsgSeqNumTooLow,         // 9 收到的 MsgSeqNum 太小
    NextExpectedTooHigh,     // 10 收到的 NextExpectedMsgSeqNum 太大
    Other,                   // 101 其他
    InvalidMessage,          // 102 无效消息
    Unknown(i32),
}

impl Logout {
    /// Parse a `Logout` from the body bytes (after the 8-byte header).
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, LOGOUT_BODY_LEN)?;
        Ok(Logout {
            session_status: be_i32(buf, 0),
            text: fixed(buf, 4),
        })
    }

    /// Free-text detail as a trimmed string.
    pub fn text_str(&self) -> &str {
        trimmed_str(&self.text)
    }

    /// Decode `session_status` into a named reason.
    pub fn status(&self) -> SessionStatus {
        match self.session_status {
            0 => SessionStatus::Active,
            1 => SessionStatus::PasswordChanged,
            2 => SessionStatus::PasswordExpiring,
            3 => SessionStatus::NewPasswordNonCompliant,
            4 => SessionStatus::LogoutComplete,
            5 => SessionStatus::InvalidUserOrPassword,
            6 => SessionStatus::AccountLocked,
            7 => SessionStatus::LoginNotAllowedNow,
            8 => SessionStatus::PasswordExpired,
            9 => SessionStatus::MsgSeqNumTooLow,
            10 => SessionStatus::NextExpectedTooHigh,
            101 => SessionStatus::Other,
            102 => SessionStatus::InvalidMessage,
            other => SessionStatus::Unknown(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logout_parse() {
        let mut buf = vec![b' '; LOGOUT_BODY_LEN];
        buf[0..4].copy_from_slice(&5i32.to_be_bytes());
        let msg = b"Invalid username or password";
        buf[4..4 + msg.len()].copy_from_slice(msg);

        let logout = Logout::parse(&buf).unwrap();
        assert_eq!(logout.session_status, 5);
        assert_eq!(logout.status(), SessionStatus::InvalidUserOrPassword);
        assert_eq!(logout.text_str(), "Invalid username or password");
    }

    #[test]
    fn rejects_short_buffer() {
        assert!(matches!(
            Logout::parse(&[0u8; LOGOUT_BODY_LEN - 1]),
            Err(ParseError::BufferTooShort { .. })
        ));
    }
}
