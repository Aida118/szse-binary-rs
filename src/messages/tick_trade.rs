// src/messages/tick_trade.rs
use crate::error::ParseError;
use crate::types::ExecType;
use crate::utils::{be_i64, be_u16, fixed, require_len, trimmed_str};

/// Tick-by-tick trade message body (逐笔成交, `T_STEPTRADE`).
///
/// Shared layout for MsgType 300191 / 300591 / 300791 / 300291 / 300391 /
/// 300491 — only the (optional) trailing extension fields differ. This
/// struct covers the fixed 66-byte body; the header (8 bytes) is parsed
/// separately via [`crate::MsgHeader`].
///
/// Within one channel, `appl_seq_num` is shared with tick orders and
/// increments from 1 (§4.4.6 note 1), which lets a consumer rebuild the
/// exact order/trade interleaving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickTrade {
    /// Channel code (频道代码).
    pub channel_no: u16,
    /// Sequential record number within the channel, from 1 (消息记录号).
    pub appl_seq_num: i64,
    /// Market-data stream id, e.g. `"011"` equity auction (行情类别).
    pub md_stream_id: [u8; 3],
    /// Bid-side order index; `0` = no matching order (买方委托索引).
    pub bid_appl_seq_num: i64,
    /// Offer-side order index; `0` = no matching order (卖方委托索引).
    pub offer_appl_seq_num: i64,
    /// Security code, ASCII, space-padded (证券代码).
    pub security_id: [u8; 8],
    /// Security code source, `"102 "` = SZSE (证券代码源).
    pub security_id_source: [u8; 4],
    /// Trade price, `N13(4)` (成交价格).
    pub last_px: i64,
    /// Trade quantity, `N15(2)` (成交数量).
    pub last_qty: i64,
    /// `'4'` cancel / `'F'` trade (成交类别).
    pub exec_type: ExecType,
    /// Timestamp `YYYYMMDDHHMMSSsss` (委托时间).
    pub transact_time: i64,
}

/// Wire size of the fixed `TickTrade` body, excluding header and checksum.
///
/// `2 + 8 + 3 + 8 + 8 + 8 + 4 + 8 + 8 + 1 + 8 = 66`
pub const TICK_TRADE_BODY_LEN: usize = 66;

impl TickTrade {
    /// Parse a `TickTrade` from the **body** bytes (after the 8-byte header).
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, TICK_TRADE_BODY_LEN)?;
        Ok(TickTrade {
            channel_no: be_u16(buf, 0),
            appl_seq_num: be_i64(buf, 2),
            md_stream_id: fixed(buf, 10),
            bid_appl_seq_num: be_i64(buf, 13),
            offer_appl_seq_num: be_i64(buf, 21),
            security_id: fixed(buf, 29),
            security_id_source: fixed(buf, 37),
            last_px: be_i64(buf, 41),
            last_qty: be_i64(buf, 49),
            exec_type: ExecType::from_byte(buf[57])?,
            transact_time: be_i64(buf, 58),
        })
    }

    /// Trade price in yuan (元).
    pub fn last_px_f64(&self) -> f64 {
        self.last_px as f64 / 10_000.0
    }
    /// Trade quantity in shares/contracts.
    pub fn last_qty_f64(&self) -> f64 {
        self.last_qty as f64 / 100.0
    }
    /// Security code as a trimmed string.
    pub fn security_id_str(&self) -> &str {
        trimmed_str(&self.security_id)
    }
    /// Market-data stream id as a trimmed string.
    pub fn md_stream_id_str(&self) -> &str {
        trimmed_str(&self.md_stream_id)
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
        buf[41..49].copy_from_slice(&186_400i64.to_be_bytes());
        buf[49..57].copy_from_slice(&100_000i64.to_be_bytes());
        buf[57] = b'F';
        buf[58..66].copy_from_slice(&20_250_512_093_000_000i64.to_be_bytes());
        buf
    }

    #[test]
    fn parses_all_fields_at_correct_offsets() {
        let buf = sample_tick_trade_buf();
        let t = TickTrade::parse(&buf).unwrap();
        assert_eq!(t.channel_no, 2011);
        assert_eq!(t.appl_seq_num, 1);
        assert_eq!(t.md_stream_id_str(), "011");
        assert_eq!(t.bid_appl_seq_num, 100);
        assert_eq!(t.offer_appl_seq_num, 200);
        assert_eq!(t.security_id_str(), "000001");
        assert_eq!(t.last_px_f64(), 18.6400);
        assert_eq!(t.last_qty_f64(), 1000.0);
        assert_eq!(t.exec_type, ExecType::Trade);
        // regression guard: transact_time must come from [58,66), not [57,65)
        assert_eq!(t.transact_time, 20_250_512_093_000_000);
    }

    #[test]
    fn rejects_short_buffer() {
        let buf = vec![0u8; TICK_TRADE_BODY_LEN - 1];
        assert!(matches!(
            TickTrade::parse(&buf),
            Err(ParseError::BufferTooShort { .. })
        ));
    }

    #[test]
    fn rejects_invalid_exec_type() {
        let mut buf = sample_tick_trade_buf();
        buf[57] = b'Z';
        assert_eq!(
            TickTrade::parse(&buf),
            Err(ParseError::InvalidEnum {
                field: "ExecType",
                value: b'Z'
            })
        );
    }
}
