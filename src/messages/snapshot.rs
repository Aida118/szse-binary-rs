// src/messages/snapshot.rs
use crate::error::ParseError;
use crate::utils::{be_i64, be_u16, be_u32, fixed, require_len, trimmed_str};

/// Fixed 65-byte snapshot header shared by all snapshot types
/// (快照行情, `T_MARKETDATA`). Stream-specific entries follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotHeader {
    /// Data generation time `YYYYMMDDHHMMSSsss` (数据生成时间).
    pub orig_time: i64,
    /// Channel code (频道代码).
    pub channel_no: u16,
    /// Market-data stream id, e.g. `"010"` equity auction (行情类别).
    pub md_stream_id: [u8; 3],
    /// Security code (证券代码).
    pub security_id: [u8; 8],
    /// Security code source, `"102 "` = SZSE (证券代码源).
    pub security_id_source: [u8; 4],
    /// Trading-phase code; byte 0 = phase, byte 1 = halt flag (交易阶段代码).
    pub trading_phase_code: [u8; 8],
    /// Previous close, `N13(4)` (昨收价).
    pub prev_close_px: i64,
    /// Number of trades (成交笔数).
    pub num_trades: i64,
    /// Total traded volume, `N15(2)` (成交总量).
    pub total_volume_trade: i64,
    /// Total traded value, `N18(4)` (成交总金额).
    pub total_value_trade: i64,
}

/// Wire size of the fixed snapshot header.
///
/// `8 + 2 + 3 + 8 + 4 + 8 + 8 + 8 + 8 + 8 = 65`
pub const SNAPSHOT_FIXED_HEADER_LEN: usize = 65;

impl SnapshotHeader {
    /// Parse the fixed 65-byte header from the body bytes.
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, SNAPSHOT_FIXED_HEADER_LEN)?;
        Ok(SnapshotHeader {
            orig_time: be_i64(buf, 0),
            channel_no: be_u16(buf, 8),
            md_stream_id: fixed(buf, 10),
            security_id: fixed(buf, 13),
            security_id_source: fixed(buf, 21),
            trading_phase_code: fixed(buf, 25),
            prev_close_px: be_i64(buf, 33),
            num_trades: be_i64(buf, 41),
            total_volume_trade: be_i64(buf, 49),
            total_value_trade: be_i64(buf, 57),
        })
    }

    pub fn security_id_str(&self) -> &str {
        trimmed_str(&self.security_id)
    }
    pub fn md_stream_id_str(&self) -> &str {
        trimmed_str(&self.md_stream_id)
    }
    pub fn trading_phase_code_str(&self) -> &str {
        trimmed_str(&self.trading_phase_code)
    }
}

/// One market-data entry (行情条目, `T_*MDENTRYITEM`).
///
/// Not every field is meaningful for every entry type: depth levels
/// (`0`/`1`) use all of them, while trade/statistic entries (`2`, `4`, `7`,
/// `8`, …) carry only `price`. `price_level`, `number_of_orders` and
/// `orders` are absent (left zero/empty) for the simpler snapshot variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MDEntry {
    /// Entry type code, e.g. `"0"` bid, `"1"` ask, `"2"` last (行情条目类别).
    pub entry_type: [u8; 2],
    /// Price; scale depends on type — `N18(6)` for `MDEntryPx` (价格).
    pub price: i64,
    /// Quantity, `N15(2)` (数量).
    pub size: i64,
    /// Depth level, 1-based (买卖盘档位).
    pub price_level: u16,
    /// Total orders at this level; `0` = not disclosed (价位总委托笔数).
    pub number_of_orders: i64,
    /// Per-order quantities for L2 depth (委托明细).
    pub orders: Vec<i64>,
}

impl MDEntry {
    /// Entry type as a trimmed string (e.g. `"0"`, `"x1"`).
    pub fn entry_type_str(&self) -> &str {
        trimmed_str(&self.entry_type)
    }
}

/// A fully decoded snapshot: the fixed header plus its typed payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub header: SnapshotHeader,
    pub body: SnapshotBody,
}

/// Stream-specific snapshot payload, selected by `MsgType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotBody {
    /// 300111 — auction depth book (集中竞价/质押/分销/期权).
    Auction(Vec<MDEntry>),
    /// 300611 / 303711 — after-hours fixed-price (盘后定价): type+px+size.
    AfterHours(Vec<MDEntry>),
    /// 309011 — index snapshot (指数行情): type+px only.
    Index(Vec<MDEntry>),
    /// 309111 — volume-statistic indicator (成交量统计指标): sample count.
    VolumeStat { stock_num: u32 },
}

impl Snapshot {
    /// Parse a complete snapshot body for the given `msg_type`.
    pub fn parse(buf: &[u8], msg_type: u32) -> Result<Self, ParseError> {
        let header = SnapshotHeader::parse(buf)?;
        let ext = &buf[SNAPSHOT_FIXED_HEADER_LEN..];
        let body = match msg_type {
            300111 => SnapshotBody::Auction(parse_depth_entries(ext)?),
            300611 | 303711 => SnapshotBody::AfterHours(parse_simple_entries(ext, true)?),
            309011 => SnapshotBody::Index(parse_simple_entries(ext, false)?),
            309111 => {
                require_len(ext, 4)?;
                SnapshotBody::VolumeStat {
                    stock_num: be_u32(ext, 0),
                }
            }
            other => return Err(ParseError::UnknownMsgType(other)),
        };
        Ok(Snapshot { header, body })
    }
}

/// Parse the 300111 extension: `NoMDEntries` then variable-width entries
/// each carrying an `OrderQty[NoOrders]` tail.
fn parse_depth_entries(ext: &[u8]) -> Result<Vec<MDEntry>, ParseError> {
    require_len(ext, 4)?;
    let count = be_u32(ext, 0) as usize;
    let mut entries = Vec::with_capacity(count);
    let mut off = 4;
    for _ in 0..count {
        // fixed part: type(2)+px(8)+size(8)+level(2)+numOrders(8)+noOrders(4) = 32
        require_len(ext, off + 32)?;
        let entry_type = fixed(ext, off);
        let price = be_i64(ext, off + 2);
        let size = be_i64(ext, off + 10);
        let price_level = be_u16(ext, off + 18);
        let number_of_orders = be_i64(ext, off + 20);
        let no_orders = be_u32(ext, off + 28) as usize;
        off += 32;

        require_len(ext, off + no_orders * 8)?;
        let mut orders = Vec::with_capacity(no_orders);
        for _ in 0..no_orders {
            orders.push(be_i64(ext, off));
            off += 8;
        }
        entries.push(MDEntry {
            entry_type,
            price,
            size,
            price_level,
            number_of_orders,
            orders,
        });
    }
    Ok(entries)
}

/// Parse 300611/303711 (`with_size = true`, 18B each) or 309011
/// (`with_size = false`, 10B each) entries.
fn parse_simple_entries(ext: &[u8], with_size: bool) -> Result<Vec<MDEntry>, ParseError> {
    require_len(ext, 4)?;
    let count = be_u32(ext, 0) as usize;
    let item_len = if with_size { 18 } else { 10 };
    let mut entries = Vec::with_capacity(count);
    let mut off = 4;
    for _ in 0..count {
        require_len(ext, off + item_len)?;
        let entry_type = fixed(ext, off);
        let price = be_i64(ext, off + 2);
        let size = if with_size { be_i64(ext, off + 10) } else { 0 };
        off += item_len;
        entries.push(MDEntry {
            entry_type,
            price,
            size,
            price_level: 0,
            number_of_orders: 0,
            orders: Vec::new(),
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_buf() -> Vec<u8> {
        let mut buf = vec![0u8; SNAPSHOT_FIXED_HEADER_LEN];
        buf[0..8].copy_from_slice(&20_250_512_093_000_000i64.to_be_bytes());
        buf[8..10].copy_from_slice(&1001u16.to_be_bytes());
        buf[10..13].copy_from_slice(b"010");
        buf[13..21].copy_from_slice(b"000001  ");
        buf[21..25].copy_from_slice(b"102 ");
        buf[25..33].copy_from_slice(b"T0      ");
        buf[33..41].copy_from_slice(&180_000i64.to_be_bytes());
        buf[41..49].copy_from_slice(&123i64.to_be_bytes());
        buf[49..57].copy_from_slice(&100_000i64.to_be_bytes());
        buf[57..65].copy_from_slice(&1_800_000i64.to_be_bytes());
        buf
    }

    #[test]
    fn parses_header() {
        let h = SnapshotHeader::parse(&header_buf()).unwrap();
        assert_eq!(h.channel_no, 1001);
        assert_eq!(h.security_id_str(), "000001");
        assert_eq!(h.md_stream_id_str(), "010");
        assert_eq!(h.num_trades, 123);
    }

    #[test]
    fn parses_300111_depth_with_orders() {
        let mut buf = header_buf();
        buf.extend_from_slice(&1u32.to_be_bytes()); // NoMDEntries = 1
        buf.extend_from_slice(b"0 "); // entry_type (bid), space-padded
        buf.extend_from_slice(&154_000i64.to_be_bytes()); // px
        buf.extend_from_slice(&320_000i64.to_be_bytes()); // size
        buf.extend_from_slice(&1u16.to_be_bytes()); // level
        buf.extend_from_slice(&2i64.to_be_bytes()); // numberOfOrders
        buf.extend_from_slice(&2u32.to_be_bytes()); // noOrders = 2
        buf.extend_from_slice(&100_000i64.to_be_bytes());
        buf.extend_from_slice(&220_000i64.to_be_bytes());

        let snap = Snapshot::parse(&buf, 300111).unwrap();
        match snap.body {
            SnapshotBody::Auction(entries) => {
                assert_eq!(entries.len(), 1);
                let e = &entries[0];
                assert_eq!(e.entry_type_str(), "0");
                assert_eq!(e.price, 154_000);
                assert_eq!(e.price_level, 1);
                assert_eq!(e.orders, vec![100_000, 220_000]);
            }
            other => panic!("expected Auction, got {other:?}"),
        }
    }

    #[test]
    fn parses_309011_index() {
        let mut buf = header_buf();
        buf.extend_from_slice(&1u32.to_be_bytes());
        buf.extend_from_slice(b"3 "); // current index, space-padded
        buf.extend_from_slice(&3_456_789_000i64.to_be_bytes());

        let snap = Snapshot::parse(&buf, 309011).unwrap();
        match snap.body {
            SnapshotBody::Index(e) => {
                assert_eq!(e.len(), 1);
                assert_eq!(e[0].entry_type_str(), "3");
                assert_eq!(e[0].price, 3_456_789_000);
                assert_eq!(e[0].size, 0);
            }
            other => panic!("expected Index, got {other:?}"),
        }
    }

    #[test]
    fn parses_309111_volume_stat() {
        let mut buf = header_buf();
        buf.extend_from_slice(&500u32.to_be_bytes());
        let snap = Snapshot::parse(&buf, 309111).unwrap();
        assert_eq!(snap.body, SnapshotBody::VolumeStat { stock_num: 500 });
    }

    #[test]
    fn truncated_orders_tail_is_rejected() {
        let mut buf = header_buf();
        buf.extend_from_slice(&1u32.to_be_bytes());
        buf.extend_from_slice(b"0 ");
        buf.extend_from_slice(&154_000i64.to_be_bytes());
        buf.extend_from_slice(&320_000i64.to_be_bytes());
        buf.extend_from_slice(&1u16.to_be_bytes());
        buf.extend_from_slice(&2i64.to_be_bytes());
        buf.extend_from_slice(&2u32.to_be_bytes()); // claims 2 orders
        buf.extend_from_slice(&100_000i64.to_be_bytes()); // but only 1 present
        assert!(matches!(
            Snapshot::parse(&buf, 300111),
            Err(ParseError::BufferTooShort { .. })
        ));
    }
}
