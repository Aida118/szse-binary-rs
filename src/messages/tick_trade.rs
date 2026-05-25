// src/messages/tick_trade.rs
use crate::error::ParseError;
use crate::types::ExecType;
use crate::utils::require_len;

/// 逐笔成交消息体 (不含消息头)
#[derive(Debug, Clone)]
pub struct TickTrade {
    pub channel_no: u16,
    pub appl_seq_num: i64,
    pub md_stream_id: [u8; 3],
    pub bid_appl_seq_num: i64,
    pub offer_appl_seq_num: i64,
    pub security_id: [u8; 8],
    pub security_id_source: [u8; 4],
    pub last_px: i64,
    pub last_qty: i64,
    pub exec_type: ExecType,
    pub transact_time: i64,
}

pub const TICK_TRADE_BODY_LEN: usize = 66;

impl TickTrade {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, TICK_TRADE_BODY_LEN)?;
        let exec_type = match buf[56] {
            b'4' => ExecType::Cancelled,
            b'F' => ExecType::Trade,
            other => return Err(ParseError::UnknownMsgType(other as u32)),
        };
        Ok(TickTrade {
            channel_no: u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            appl_seq_num: i64::from_be_bytes(buf[2..10].try_into().unwrap()),
            md_stream_id: buf[10..13].try_into().unwrap(),
            bid_appl_seq_num: i64::from_be_bytes(buf[13..21].try_into().unwrap()),
            offer_appl_seq_num: i64::from_be_bytes(buf[21..29].try_into().unwrap()),
            security_id: buf[29..37].try_into().unwrap(),
            security_id_source: buf[37..41].try_into().unwrap(),
            last_px: i64::from_be_bytes(buf[41..49].try_into().unwrap()),
            last_qty: i64::from_be_bytes(buf[49..57].try_into().unwrap()),
            exec_type,
            transact_time: i64::from_be_bytes(buf[57..65].try_into().unwrap()),
        })
    }

    pub fn last_px_f64(&self) -> f64 { self.last_px as f64 / 10_000.0 }
    pub fn last_qty_f64(&self) -> f64 { self.last_qty as f64 / 100.0 }
    pub fn security_id_str(&self) -> &str {
        std::str::from_utf8(&self.security_id).unwrap_or("").trim_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tick_trade_buf() -> Vec<u8> {
        let mut buf = vec![0u8; TICK_TRADE_BODY_LEN];
        buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
        buf[2..10].copy_from_slice(&1i64.to_be_bytes());
        buf[10..13].copy_from_slice(b"011");
        buf[13..21].copy_from_slice(&100i64.to_be_bytes());
        buf[21..29].copy_from_slice(&200i64.to_be_bytes());
        buf[29..37].copy_from_slice(b"000001  ");
        buf[37..41].copy_from_slice(b"102 ");
        buf[41..49].copy_from_slice(&186400i64.to_be_bytes());
        buf[49..57].copy_from_slice(&100000i64.to_be_bytes());
        buf[56] = b'F';
        buf[57..65].copy_from_slice(&20250512093000000i64.to_be_bytes());
        buf
    }

    #[test]
    fn test_tick_trade_parse() {
        let buf = sample_tick_trade_buf();
        let t = TickTrade::parse(&buf).unwrap();
        assert_eq!(t.channel_no, 2011);
        assert_eq!(t.appl_seq_num, 1);
        assert_eq!(t.exec_type, ExecType::Trade);
        assert_eq!(t.last_px_f64(), 18.6400);
        assert_eq!(t.security_id_str(), "000001");
    }
}