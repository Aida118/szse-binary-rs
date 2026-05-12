//! # szse-binary-rs
//!
//! Parser for the Shenzhen Stock Exchange (SZSE) Binary market data protocol.
//!
//! Based on: 深圳证券交易所 Binary 行情数据接口规范 Ver1.17 (2025-03)
//!
//! ## Supported Messages
//! - Message Header (消息头)
//! - Tick-by-tick Trade / 逐笔成交 (MsgType=300191)
//! - Tick-by-tick Order / 逐笔委托 (MsgType=300192)
//!
//! ## Note on types
//! All integers are big-endian (network byte order) per the spec.
//! Price fields use N13(4): Int64 value 186400 means price 18.6400.
//! Qty fields use N15(2), Amt fields use N18(4).

// ─────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Buffer is shorter than required
    BufferTooShort { needed: usize, got: usize },
    /// Message type is not recognised by this parser
    UnknownMsgType(u32),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::BufferTooShort { needed, got } =>
                write!(f, "buffer too short: need {} bytes, got {}", needed, got),
            ParseError::UnknownMsgType(t) =>
                write!(f, "unknown message type: {}", t),
        }
    }
}

// ─────────────────────────────────────────────
// Session-layer header  (消息头, section 4.2.1)
// ─────────────────────────────────────────────

/// Every SZSE Binary message begins with this 8-byte header.
///
/// Layout (all big-endian):
/// | Offset | Size | Field      |
/// |--------|------|------------|
/// | 0      | 4    | MsgType    |
/// | 4      | 4    | BodyLength |
#[derive(Debug, Clone, PartialEq)]
pub struct MsgHeader {
    /// Message type, e.g. 300191 = tick-by-tick trade
    pub msg_type: u32,
    /// Length of the message body in bytes (excludes header and checksum tail)
    pub body_length: u32,
}

pub const MSG_HEADER_LEN: usize = 8;

impl MsgHeader {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, MSG_HEADER_LEN)?;
        Ok(MsgHeader {
            msg_type:    u32::from_be_bytes(buf[0..4].try_into().unwrap()),
            body_length: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
        })
    }
}

// ─────────────────────────────────────────────
// Tick-by-tick Trade  逐笔成交 (MsgType=300191)
// section 4.5.6, table 4-15
// ─────────────────────────────────────────────

/// Execution type in a tick-by-tick trade message.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecType {
    /// '4' – order cancelled (撤销)
    Cancelled,
    /// 'F' – trade executed (成交)
    Trade,
}

/// Tick-by-tick trade message (逐笔成交, MsgType=300191).
///
/// Carried in channels 201x–205x depending on security type.
/// The `appl_seq_num` is shared with tick-by-tick orders on the same channel.
///
/// Price unit: N13(4), i.e. divide raw Int64 by 10_000 to get yuan.
/// Qty   unit: N15(2), divide by 100 to get shares.
#[derive(Debug, Clone)]
pub struct TickTrade {
    /// Channel code (频道代码)
    pub channel_no: u16,
    /// Sequential record number within the channel, starting from 1
    pub appl_seq_num: i64,
    /// Market data stream ID, e.g. "011" = equity auction trade (行情类别)
    pub md_stream_id: [u8; 3],
    /// Bid-side order index; 0 = no corresponding order (买方委托索引)
    pub bid_appl_seq_num: i64,
    /// Offer-side order index; 0 = no corresponding order (卖方委托索引)
    pub offer_appl_seq_num: i64,
    /// Security code, ASCII, space-padded (证券代码)
    pub security_id: [u8; 8],
    /// Security code source, e.g. "102 " = SZSE (证券代码源)
    pub security_id_source: [u8; 4],
    /// Trade price, N13(4) — divide by 10_000 for yuan (成交价格)
    pub last_px: i64,
    /// Trade quantity, N15(2) — divide by 100 for shares (成交数量)
    pub last_qty: i64,
    /// Execution type (成交类别)
    pub exec_type: ExecType,
    /// Transaction timestamp YYYYMMDDHHMMSSsss (委托时间)
    pub transact_time: i64,
}

/// Wire size of a TickTrade body (without header or checksum).
/// channel_no(2) + appl_seq_num(8) + md_stream_id(3) +
/// bid(8) + offer(8) + security_id(8) + source(4) +
/// last_px(8) + last_qty(8) + exec_type(1) + transact_time(8) = 66 bytes
pub const TICK_TRADE_BODY_LEN: usize = 66;

impl TickTrade {
    /// Parse a TickTrade from the **body** bytes (after the 8-byte header).
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, TICK_TRADE_BODY_LEN)?;

        let exec_type = match buf[56] {
            b'4' => ExecType::Cancelled,
            b'F' => ExecType::Trade,
            other => return Err(ParseError::UnknownMsgType(other as u32)),
        };

        Ok(TickTrade {
            channel_no:           u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            appl_seq_num:         i64::from_be_bytes(buf[2..10].try_into().unwrap()),
            md_stream_id:         buf[10..13].try_into().unwrap(),
            bid_appl_seq_num:     i64::from_be_bytes(buf[13..21].try_into().unwrap()),
            offer_appl_seq_num:   i64::from_be_bytes(buf[21..29].try_into().unwrap()),
            security_id:          buf[29..37].try_into().unwrap(),
            security_id_source:   buf[37..41].try_into().unwrap(),
            last_px:              i64::from_be_bytes(buf[41..49].try_into().unwrap()),
            last_qty:             i64::from_be_bytes(buf[49..57].try_into().unwrap()),
            exec_type,
            transact_time:        i64::from_be_bytes(buf[57..65].try_into().unwrap()),
        })
    }

    /// Return price in yuan (元) as f64. Convenience helper.
    pub fn last_px_f64(&self) -> f64 { self.last_px as f64 / 10_000.0 }

    /// Return quantity in shares as f64.
    pub fn last_qty_f64(&self) -> f64 { self.last_qty as f64 / 100.0 }

    /// Return security code as a trimmed UTF-8 string.
    pub fn security_id_str(&self) -> &str {
        std::str::from_utf8(&self.security_id)
            .unwrap_or("")
            .trim_end()
    }
}

// ─────────────────────────────────────────────
// Tick-by-tick Order  逐笔委托 (MsgType=300192)
// section 4.5.5, table 4-14
// ─────────────────────────────────────────────

/// Buy/sell direction (买卖方向).
#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    Buy,   // '1'
    Sell,  // '2'
    Borrow, // 'G' 借入
    Lend,   // 'F' 出借
}

/// Order type (订单类别, only for MsgType=300192 extension field).
#[derive(Debug, Clone, PartialEq)]
pub enum OrdType {
    Market,       // '1' 市价
    Limit,        // '2' 限价
    BestOwn,      // 'U' 本方最优
}

/// Tick-by-tick order message (逐笔委托, MsgType=300192).
#[derive(Debug, Clone)]
pub struct TickOrder {
    pub channel_no:         u16,
    pub appl_seq_num:       i64,
    pub md_stream_id:       [u8; 3],
    pub security_id:        [u8; 8],
    pub security_id_source: [u8; 4],
    /// Limit price, N13(4)
    pub price:              i64,
    /// Order quantity, N15(2)
    pub order_qty:          i64,
    pub side:               Side,
    pub transact_time:      i64,
    /// Extension field: order type (from 300192 extension)
    pub ord_type:           OrdType,
}

/// Wire size of TickOrder body including the 300192 extension byte.
/// channel_no(2) + appl_seq_num(8) + md_stream_id(3) +
/// security_id(8) + source(4) + price(8) + order_qty(8) +
/// side(1) + transact_time(8) + ord_type(1) = 51 bytes
pub const TICK_ORDER_BODY_LEN: usize = 51;

impl TickOrder {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, TICK_ORDER_BODY_LEN)?;

        let side = match buf[41] {
            b'1' => Side::Buy,
            b'2' => Side::Sell,
            b'G' => Side::Borrow,
            b'F' => Side::Lend,
            other => return Err(ParseError::UnknownMsgType(other as u32)),
        };

        let ord_type = match buf[50] {
            b'1' => OrdType::Market,
            b'2' => OrdType::Limit,
            b'U' => OrdType::BestOwn,
            other => return Err(ParseError::UnknownMsgType(other as u32)),
        };

        Ok(TickOrder {
            channel_no:         u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            appl_seq_num:       i64::from_be_bytes(buf[2..10].try_into().unwrap()),
            md_stream_id:       buf[10..13].try_into().unwrap(),
            security_id:        buf[13..21].try_into().unwrap(),
            security_id_source: buf[21..25].try_into().unwrap(),
            price:              i64::from_be_bytes(buf[25..33].try_into().unwrap()),
            order_qty:          i64::from_be_bytes(buf[33..41].try_into().unwrap()),
            side,
            transact_time:      i64::from_be_bytes(buf[42..50].try_into().unwrap()),
            ord_type,
        })
    }

    pub fn price_f64(&self) -> f64 { self.price as f64 / 10_000.0 }
    pub fn order_qty_f64(&self) -> f64 { self.order_qty as f64 / 100.0 }
    pub fn security_id_str(&self) -> &str {
        std::str::from_utf8(&self.security_id)
            .unwrap_or("")
            .trim_end()
    }
}

// ─────────────────────────────────────────────
// Internal helper
// ─────────────────────────────────────────────

fn require_len(buf: &[u8], needed: usize) -> Result<(), ParseError> {
    if buf.len() < needed {
        Err(ParseError::BufferTooShort { needed, got: buf.len() })
    } else {
        Ok(())
    }
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MsgHeader ──────────────────────────────

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
        // MsgType = 300191 = 0x0004_939F
        buf[0..4].copy_from_slice(&300191u32.to_be_bytes());
        buf[4..8].copy_from_slice(&66u32.to_be_bytes());
        let h = MsgHeader::parse(&buf).unwrap();
        assert_eq!(h.msg_type, 300191);
        assert_eq!(h.body_length, 66);
    }

    // ── TickTrade ──────────────────────────────

    fn sample_tick_trade_buf() -> Vec<u8> {
        let mut buf = vec![0u8; TICK_TRADE_BODY_LEN];
        // channel_no = 2011
        buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
        // appl_seq_num = 1
        buf[2..10].copy_from_slice(&1i64.to_be_bytes());
        // md_stream_id = "011"
        buf[10..13].copy_from_slice(b"011");
        // bid = 100, offer = 200
        buf[13..21].copy_from_slice(&100i64.to_be_bytes());
        buf[21..29].copy_from_slice(&200i64.to_be_bytes());
        // security_id = "000001  " (平安银行)
        buf[29..37].copy_from_slice(b"000001  ");
        // security_id_source = "102 "
        buf[37..41].copy_from_slice(b"102 ");
        // last_px = 186400 → 18.6400 yuan
        buf[41..49].copy_from_slice(&186400i64.to_be_bytes());
        // last_qty = 100000 → 1000.00 shares
        buf[49..57].copy_from_slice(&100000i64.to_be_bytes());
        // exec_type = 'F' (成交)
        buf[56] = b'F';
        // transact_time = 20250512093000000
        buf[57..65].copy_from_slice(&20250512093000000i64.to_be_bytes());
        buf
    }

    #[test]
    fn tick_trade_parses_correctly() {
        let buf = sample_tick_trade_buf();
        let t = TickTrade::parse(&buf).unwrap();
        assert_eq!(t.channel_no, 2011);
        assert_eq!(t.appl_seq_num, 1);
        assert_eq!(t.exec_type, ExecType::Trade);
        assert_eq!(t.last_px_f64(), 18.6400);
        assert_eq!(t.security_id_str(), "000001");
    }

    #[test]
    fn tick_trade_too_short() {
        let buf = [0u8; 10];
        assert!(matches!(
            TickTrade::parse(&buf),
            Err(ParseError::BufferTooShort { .. })
        ));
    }

    // ── TickOrder ──────────────────────────────

    #[test]
    fn tick_order_buy_limit() {
        let mut buf = vec![0u8; TICK_ORDER_BODY_LEN];
        buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
        buf[2..10].copy_from_slice(&42i64.to_be_bytes());
        buf[10..13].copy_from_slice(b"011");
        buf[13..21].copy_from_slice(b"000001  ");
        buf[21..25].copy_from_slice(b"102 ");
        buf[25..33].copy_from_slice(&186400i64.to_be_bytes()); // 18.64 yuan
        buf[33..41].copy_from_slice(&100000i64.to_be_bytes()); // 1000 shares
        buf[41] = b'1'; // Buy
        buf[42..50].copy_from_slice(&20250512093000000i64.to_be_bytes());
        buf[50] = b'2'; // Limit

        let o = TickOrder::parse(&buf).unwrap();
        assert_eq!(o.side, Side::Buy);
        assert_eq!(o.ord_type, OrdType::Limit);
        assert_eq!(o.price_f64(), 18.64);
        assert_eq!(o.security_id_str(), "000001");
    }
}