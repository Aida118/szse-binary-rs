// src/messages/tick_order.rs
use crate::error::ParseError;
use crate::types::{OrdType, Side};
use crate::utils::{be_i64, be_u8, be_u16, fixed, require_len, trimmed_str};

/// Tick-by-tick order message (逐笔委托, `T_STEPORDER`).
///
/// Covers the fixed 50-byte body shared by MsgType 300192 / 300592 /
/// 300792 / 300292 / 300392 / 300492. Stream-specific trailing fields are
/// decoded into [`OrderExtension`] via [`TickOrder::parse_with_ext`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickOrder {
    /// Channel code (频道代码).
    pub channel_no: u16,
    /// Sequential record number within the channel, from 1 (消息记录号).
    pub appl_seq_num: i64,
    /// Market-data stream id (行情类别).
    pub md_stream_id: [u8; 3],
    /// Security code, ASCII, space-padded (证券代码).
    pub security_id: [u8; 8],
    /// Security code source, `"102 "` = SZSE (证券代码源).
    pub security_id_source: [u8; 4],
    /// Order price, `N13(4)` (委托价格).
    pub price: i64,
    /// Order quantity, `N15(2)` (委托数量).
    pub order_qty: i64,
    /// Buy/sell direction (买卖方向).
    pub side: Side,
    /// Timestamp `YYYYMMDDHHMMSSsss` (委托时间).
    pub transact_time: i64,
}

/// Wire size of the fixed `TickOrder` body, before any extension fields.
///
/// `2 + 8 + 3 + 8 + 4 + 8 + 8 + 1 + 8 = 50`
pub const TICK_ORDER_BODY_LEN: usize = 50;

/// Stream-specific trailing fields appended after the 50-byte base body.
///
/// Which variant applies is determined by the message type, not the body
/// itself — see the C++ `t_300192ExtendFields` family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderExtension {
    /// 300192 — auction order type (集中竞价, `t_300192ExtendFields`).
    Auction { ord_type: OrdType },
    /// 300592 — negotiated trade (协议交易, `t_300592ExtendFields`).
    Negotiated {
        /// Pricing-quote id; empty = an intention quote (定价行情约定号).
        confirm_id: [u8; 8],
        /// Contact person (联系人).
        contactor: [u8; 12],
        /// Contact info (联系方式).
        contact_info: [u8; 30],
    },
    /// 300792 — securities lending (转融通, `t_300792ExtendFields`).
    SecurityLending {
        /// Term in days (期限).
        expiration_days: u16,
        /// Term type; `1` = fixed term (期限类型).
        expiration_type: u8,
    },
}

impl TickOrder {
    /// Parse only the fixed 50-byte base body (ignores extension fields).
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, TICK_ORDER_BODY_LEN)?;
        Ok(TickOrder {
            channel_no: be_u16(buf, 0),
            appl_seq_num: be_i64(buf, 2),
            md_stream_id: fixed(buf, 10),
            security_id: fixed(buf, 13),
            security_id_source: fixed(buf, 21),
            price: be_i64(buf, 25),
            order_qty: be_i64(buf, 33),
            side: Side::from_byte(buf[41])?,
            transact_time: be_i64(buf, 42),
        })
    }

    /// Parse the base body plus the extension selected by `msg_type`.
    ///
    /// Returns the [`TickOrder`] and, when the message type carries one, an
    /// [`OrderExtension`]. Unknown message types yield `UnknownMsgType`.
    pub fn parse_with_ext(
        buf: &[u8],
        msg_type: u32,
    ) -> Result<(Self, Option<OrderExtension>), ParseError> {
        let base = Self::parse(buf)?;
        let ext = &buf[TICK_ORDER_BODY_LEN..];
        let extension = match msg_type {
            300192 | 300292 => {
                require_len(ext, 1)?;
                Some(OrderExtension::Auction {
                    ord_type: OrdType::from_byte(ext[0])?,
                })
            }
            300592 => {
                require_len(ext, 50)?; // 8 + 12 + 30
                Some(OrderExtension::Negotiated {
                    confirm_id: fixed(ext, 0),
                    contactor: fixed(ext, 8),
                    contact_info: fixed(ext, 20),
                })
            }
            300792 => {
                require_len(ext, 3)?; // u16 + u8
                Some(OrderExtension::SecurityLending {
                    expiration_days: be_u16(ext, 0),
                    expiration_type: be_u8(ext, 2),
                })
            }
            other => return Err(ParseError::UnknownMsgType(other)),
        };
        Ok((base, extension))
    }

    /// Order price in yuan (元).
    pub fn price_f64(&self) -> f64 {
        self.price as f64 / 10_000.0
    }
    /// Order quantity in shares/contracts.
    pub fn order_qty_f64(&self) -> f64 {
        self.order_qty as f64 / 100.0
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

    fn base_buf() -> Vec<u8> {
        let mut buf = vec![0u8; TICK_ORDER_BODY_LEN];
        buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
        buf[2..10].copy_from_slice(&42i64.to_be_bytes());
        buf[10..13].copy_from_slice(b"011");
        buf[13..21].copy_from_slice(b"000001  ");
        buf[21..25].copy_from_slice(b"102 ");
        buf[25..33].copy_from_slice(&186_400i64.to_be_bytes());
        buf[33..41].copy_from_slice(&100_000i64.to_be_bytes());
        buf[41] = b'1';
        buf[42..50].copy_from_slice(&20_250_512_093_000_000i64.to_be_bytes());
        buf
    }

    #[test]
    fn parses_base_body() {
        let o = TickOrder::parse(&base_buf()).unwrap();
        assert_eq!(o.channel_no, 2011);
        assert_eq!(o.appl_seq_num, 42);
        assert_eq!(o.side, Side::Buy);
        assert_eq!(o.price_f64(), 18.64);
        assert_eq!(o.security_id_str(), "000001");
        assert_eq!(o.transact_time, 20_250_512_093_000_000);
    }

    #[test]
    fn parses_300192_auction_extension() {
        let mut buf = base_buf();
        buf.push(b'2'); // OrdType = Limit
        let (o, ext) = TickOrder::parse_with_ext(&buf, 300192).unwrap();
        assert_eq!(o.side, Side::Buy);
        assert_eq!(
            ext,
            Some(OrderExtension::Auction {
                ord_type: OrdType::Limit
            })
        );
    }

    #[test]
    fn parses_300792_lending_extension() {
        let mut buf = base_buf();
        buf.extend_from_slice(&182u16.to_be_bytes());
        buf.push(1);
        let (_, ext) = TickOrder::parse_with_ext(&buf, 300792).unwrap();
        assert_eq!(
            ext,
            Some(OrderExtension::SecurityLending {
                expiration_days: 182,
                expiration_type: 1
            })
        );
    }

    #[test]
    fn rejects_unknown_msg_type() {
        let buf = base_buf();
        assert_eq!(
            TickOrder::parse_with_ext(&buf, 999999),
            Err(ParseError::UnknownMsgType(999999))
        );
    }
}
