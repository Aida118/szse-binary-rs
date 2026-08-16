// src/messages/common.rs
//!
//! Common application-layer messages (公共消息, §4.3): channel heartbeat,
//! resend request/response, and business reject.

use crate::error::ParseError;
use crate::utils::{be_i64, be_u16, be_u32, fixed, require_len, trimmed_str};

/// Channel heartbeat (频道心跳, MsgType=390095, `T_CHANNELNO`).
///
/// Emitted every ~3 s per idle channel so consumers can detect a stalled
/// feed and track the last sequence number seen on that channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelHeartbeat {
    /// Channel code (频道代码).
    pub channel_no: u16,
    /// Last market-data record number on this channel (最后一条消息的记录号).
    pub appl_last_seq_num: i64,
    /// End-of-channel flag: `0` = open, `1` = ended (频道结束标志).
    pub end_of_channel: u16,
}

/// Wire size of [`ChannelHeartbeat`]: `2 + 8 + 2 = 12`.
pub const CHANNEL_HEARTBEAT_BODY_LEN: usize = 12;

impl ChannelHeartbeat {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, CHANNEL_HEARTBEAT_BODY_LEN)?;
        Ok(ChannelHeartbeat {
            channel_no: be_u16(buf, 0),
            appl_last_seq_num: be_i64(buf, 2),
            end_of_channel: be_u16(buf, 10),
        })
    }

    /// Whether this heartbeat marks the end of the channel for the day.
    pub fn is_end_of_channel(&self) -> bool {
        self.end_of_channel == 1
    }
}

/// Resend request / response (重传消息, MsgType=390094, `T_RESEND`).
///
/// The client requests retransmission of tick data or announcements; the
/// gateway replies with the same message, filling `resend_status` /
/// `reject_text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resend {
    /// `1` = tick market data, `2` = announcement (重发种类).
    pub resend_type: u8,
    /// Channel code (频道代码).
    pub channel_no: u16,
    /// Begin sequence (inclusive); valid when `resend_type == 1` (起始序号).
    pub appl_beg_seq_num: i64,
    /// End sequence (inclusive); `0` = up to the gateway's max (结束序号).
    pub appl_end_seq_num: i64,
    /// Announcement id; valid when `resend_type == 2` (公告唯一标识).
    pub news_id: [u8; 8],
    /// Reply only: `1` done, `2` partial, `3` no permission, `4` unavailable.
    pub resend_status: u8,
    /// Reply only: error text if rejected (文本).
    pub reject_text: [u8; 16],
}

/// Wire size of [`Resend`]: `1 + 2 + 8 + 8 + 8 + 1 + 16 = 44`.
pub const RESEND_BODY_LEN: usize = 44;

impl Resend {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, RESEND_BODY_LEN)?;
        Ok(Resend {
            resend_type: buf[0],
            channel_no: be_u16(buf, 1),
            appl_beg_seq_num: be_i64(buf, 3),
            appl_end_seq_num: be_i64(buf, 11),
            news_id: fixed(buf, 19),
            resend_status: buf[27],
            reject_text: fixed(buf, 28),
        })
    }

    pub fn news_id_str(&self) -> &str {
        trimmed_str(&self.news_id)
    }
    pub fn reject_text_str(&self) -> &str {
        trimmed_str(&self.reject_text)
    }
}

/// Business reject (业务拒绝消息, MsgType=8, `T_BUSINESSREJECT`).
///
/// Sent when a message is session-layer valid but breaks a business rule
/// (e.g. a malformed resend or user-info report).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessReject {
    /// Sequence number of the rejected message (被拒绝消息的消息序号).
    pub ref_seq_num: i64,
    /// Type of the rejected message (被拒绝的消息类型).
    pub ref_msg_type: u32,
    /// Business-layer id of the rejected message (业务层 ID).
    pub business_reject_ref_id: [u8; 10],
    /// Reject reason code (拒绝原因).
    pub business_reject_reason: u16,
    /// Reject reason text (拒绝原因说明).
    pub business_reject_text: [u8; 50],
}

/// Wire size of [`BusinessReject`]: `8 + 4 + 10 + 2 + 50 = 74`.
pub const BUSINESS_REJECT_BODY_LEN: usize = 74;

impl BusinessReject {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, BUSINESS_REJECT_BODY_LEN)?;
        Ok(BusinessReject {
            ref_seq_num: be_i64(buf, 0),
            ref_msg_type: be_u32(buf, 8),
            business_reject_ref_id: fixed(buf, 12),
            business_reject_reason: be_u16(buf, 22),
            business_reject_text: fixed(buf, 24),
        })
    }

    pub fn business_reject_ref_id_str(&self) -> &str {
        trimmed_str(&self.business_reject_ref_id)
    }
    pub fn business_reject_text_str(&self) -> &str {
        trimmed_str(&self.business_reject_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_channel_heartbeat() {
        let mut buf = vec![0u8; CHANNEL_HEARTBEAT_BODY_LEN];
        buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
        buf[2..10].copy_from_slice(&98765i64.to_be_bytes());
        buf[10..12].copy_from_slice(&1u16.to_be_bytes());
        let hb = ChannelHeartbeat::parse(&buf).unwrap();
        assert_eq!(hb.channel_no, 2011);
        assert_eq!(hb.appl_last_seq_num, 98765);
        assert!(hb.is_end_of_channel());
    }

    #[test]
    fn parses_resend() {
        let mut buf = vec![b' '; RESEND_BODY_LEN];
        buf[0] = 1;
        buf[1..3].copy_from_slice(&2011u16.to_be_bytes());
        buf[3..11].copy_from_slice(&10i64.to_be_bytes());
        buf[11..19].copy_from_slice(&20i64.to_be_bytes());
        buf[27] = 1;
        let r = Resend::parse(&buf).unwrap();
        assert_eq!(r.resend_type, 1);
        assert_eq!(r.appl_beg_seq_num, 10);
        assert_eq!(r.appl_end_seq_num, 20);
        assert_eq!(r.resend_status, 1);
    }

    #[test]
    fn parses_business_reject() {
        let mut buf = vec![b' '; BUSINESS_REJECT_BODY_LEN];
        buf[0..8].copy_from_slice(&7i64.to_be_bytes());
        buf[8..12].copy_from_slice(&390094u32.to_be_bytes());
        buf[22..24].copy_from_slice(&3u16.to_be_bytes());
        let msg = b"no permission";
        buf[24..24 + msg.len()].copy_from_slice(msg);
        let br = BusinessReject::parse(&buf).unwrap();
        assert_eq!(br.ref_seq_num, 7);
        assert_eq!(br.ref_msg_type, 390094);
        assert_eq!(br.business_reject_reason, 3);
        assert_eq!(br.business_reject_text_str(), "no permission");
    }
}
