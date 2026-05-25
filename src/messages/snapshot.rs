// src/messages/snapshot.rs
use crate::error::ParseError;
use crate::utils::require_len;

/// 快照消息固定头部分 (65字节)
#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub orig_time: i64,
    pub channel_no: u16,
    pub md_stream_id: [u8; 3],
    pub security_id: [u8; 8],
    pub security_id_source: [u8; 4],
    pub trading_phase_code: [u8; 8],
    pub prev_close_px: i64,
    pub num_trades: i64,
    pub total_volume_trade: i64,
    pub total_value_trade: i64,
}

pub const SNAPSHOT_FIXED_HEADER_LEN: usize = 65;

impl SnapshotHeader {
    pub fn parse(buf: &[u8]) -> Result<Self, ParseError> {
        require_len(buf, SNAPSHOT_FIXED_HEADER_LEN)?;
        let mut off = 0;
        let orig_time = i64::from_be_bytes(buf[off..off+8].try_into().unwrap()); off += 8;
        let channel_no = u16::from_be_bytes(buf[off..off+2].try_into().unwrap()); off += 2;
        let mut md_stream_id = [0u8; 3];
        md_stream_id.copy_from_slice(&buf[off..off+3]); off += 3;
        let mut security_id = [0u8; 8];
        security_id.copy_from_slice(&buf[off..off+8]); off += 8;
        let mut security_id_source = [0u8; 4];
        security_id_source.copy_from_slice(&buf[off..off+4]); off += 4;
        let mut trading_phase_code = [0u8; 8];
        trading_phase_code.copy_from_slice(&buf[off..off+8]); off += 8;
        let prev_close_px = i64::from_be_bytes(buf[off..off+8].try_into().unwrap()); off += 8;
        let num_trades = i64::from_be_bytes(buf[off..off+8].try_into().unwrap()); off += 8;
        let total_volume_trade = i64::from_be_bytes(buf[off..off+8].try_into().unwrap()); off += 8;
        let total_value_trade = i64::from_be_bytes(buf[off..off+8].try_into().unwrap()); off += 8;

        Ok(SnapshotHeader {
            orig_time, channel_no, md_stream_id, security_id, security_id_source,
            trading_phase_code, prev_close_px, num_trades, total_volume_trade, total_value_trade,
        })
    }

    pub fn security_id_str(&self) -> &str {
        std::str::from_utf8(&self.security_id).unwrap_or("").trim_end()
    }
}

/// 行情条目 (MDEntry)
#[derive(Debug, Clone)]
pub struct MDEntry {
    pub entry_type: [u8; 2],
    pub price: i64,
    pub size: i64,
    pub price_level: u16,
    pub number_of_orders: i64,
    pub orders: Vec<i64>, // 委托明细 (Level-2)
}

// 解析扩展字段的函数后续实现