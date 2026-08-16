// src/types/mod.rs
pub mod primitives;

use crate::error::ParseError;

/// Buy/sell direction (买卖方向, `Side`).
///
/// Used by tick-by-tick orders (逐笔委托). The `Borrow`/`Lend` values
/// only appear in securities-lending feeds (转融通, MsgType=300792).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// `'1'` 买
    Buy,
    /// `'2'` 卖
    Sell,
    /// `'G'` 借入
    Borrow,
    /// `'F'` 出借
    Lend,
}

impl Side {
    /// Decode the single ASCII byte carried in the `Side` field.
    pub fn from_byte(b: u8) -> Result<Self, ParseError> {
        match b {
            b'1' => Ok(Side::Buy),
            b'2' => Ok(Side::Sell),
            b'G' => Ok(Side::Borrow),
            b'F' => Ok(Side::Lend),
            value => Err(ParseError::InvalidEnum {
                field: "Side",
                value,
            }),
        }
    }
}

/// Execution type in a tick-by-tick trade (成交类别, `ExecType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecType {
    /// `'4'` 撤销 — the referenced order was cancelled
    Cancelled,
    /// `'F'` 成交 — a trade was executed
    Trade,
}

impl ExecType {
    /// Decode the single ASCII byte carried in the `ExecType` field.
    pub fn from_byte(b: u8) -> Result<Self, ParseError> {
        match b {
            b'4' => Ok(ExecType::Cancelled),
            b'F' => Ok(ExecType::Trade),
            value => Err(ParseError::InvalidEnum {
                field: "ExecType",
                value,
            }),
        }
    }
}

/// Order type carried in the 300192 tick-order extension (订单类别, `OrdType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdType {
    /// `'1'` 市价
    Market,
    /// `'2'` 限价
    Limit,
    /// `'U'` 本方最优
    BestOwn,
}

impl OrdType {
    /// Decode the single ASCII byte carried in the `OrdType` field.
    pub fn from_byte(b: u8) -> Result<Self, ParseError> {
        match b {
            b'1' => Ok(OrdType::Market),
            b'2' => Ok(OrdType::Limit),
            b'U' => Ok(OrdType::BestOwn),
            value => Err(ParseError::InvalidEnum {
                field: "OrdType",
                value,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_round_trips() {
        assert_eq!(Side::from_byte(b'1'), Ok(Side::Buy));
        assert_eq!(Side::from_byte(b'F'), Ok(Side::Lend));
        assert_eq!(
            Side::from_byte(b'Z'),
            Err(ParseError::InvalidEnum {
                field: "Side",
                value: b'Z'
            })
        );
    }

    #[test]
    fn exec_type_rejects_unknown() {
        assert_eq!(ExecType::from_byte(b'F'), Ok(ExecType::Trade));
        assert!(ExecType::from_byte(b'0').is_err());
    }
}
