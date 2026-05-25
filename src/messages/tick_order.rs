// src/messages/tick_order.rs
use crate::error::ParseError;
use crate::types::{Side, OrdType};
use crate::utils::require_len;

#[derive(Debug, Clone)]
pub struct TickOrder {
    pub channel_no: u16,
    pub appl_seq_num: i64,
    pub md_stream_id: [u8; 3],
    pub security_id: [u8; 8],
    pub security_id_source: [u8; 4],
    pub price: i64,
    pub order_qty: i64,
    pub side: Side,
    pub transact_time: i64,
    pub ord_type: OrdType,
}

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
            channel_no: u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            appl_seq_num: i64::from_be_bytes(buf[2..10].try_into().unwrap()),
            md_stream_id: buf[10..13].try_into().unwrap(),
            security_id: buf[13..21].try_into().unwrap(),
            security_id_source: buf[21..25].try_into().unwrap(),
            price: i64::from_be_bytes(buf[25..33].try_into().unwrap()),
            order_qty: i64::from_be_bytes(buf[33..41].try_into().unwrap()),
            side,
            transact_time: i64::from_be_bytes(buf[42..50].try_into().unwrap()),
            ord_type,
        })
    }

    pub fn price_f64(&self) -> f64 { self.price as f64 / 10_000.0 }
    pub fn order_qty_f64(&self) -> f64 { self.order_qty as f64 / 100.0 }
    pub fn security_id_str(&self) -> &str {
        std::str::from_utf8(&self.security_id).unwrap_or("").trim_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_order_parse() {
        let mut buf = vec![0u8; TICK_ORDER_BODY_LEN];
        buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
        buf[2..10].copy_from_slice(&42i64.to_be_bytes());
        buf[10..13].copy_from_slice(b"011");
        buf[13..21].copy_from_slice(b"000001  ");
        buf[21..25].copy_from_slice(b"102 ");
        buf[25..33].copy_from_slice(&186400i64.to_be_bytes());
        buf[33..41].copy_from_slice(&100000i64.to_be_bytes());
        buf[41] = b'1';
        buf[42..50].copy_from_slice(&20250512093000000i64.to_be_bytes());
        buf[50] = b'2';
        let o = TickOrder::parse(&buf).unwrap();
        assert_eq!(o.side, Side::Buy);
        assert_eq!(o.ord_type, OrdType::Limit);
        assert_eq!(o.price_f64(), 18.64);
        assert_eq!(o.security_id_str(), "000001");
    }
}