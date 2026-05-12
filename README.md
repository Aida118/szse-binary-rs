# szse-binary-rs

A Rust parser for the **Shenzhen Stock Exchange (SZSE) Binary market data protocol**.

Based on the official spec: 深圳证券交易所 Binary 行情数据接口规范 **Ver 1.17** (2025-03).

## Status

Early stage — contributions and feedback welcome.

| Message | Type | Status |
|---------|------|--------|
| Message Header 消息头 | — | ✅ |
| Tick-by-tick Trade 逐笔成交 | 300191 | ✅ |
| Tick-by-tick Order 逐笔委托 | 300192 | ✅ |
| Snapshot 行情快照 | 300111 | 🔜 |
| Security Status 证券实时状态 | 390013 | 🔜 |

## Quick Start

```rust
use szse_binary_rs::{MsgHeader, TickTrade};

// buf: raw bytes received from the SZSE MDGW feed
let header = MsgHeader::parse(&buf[..8])?;
if header.msg_type == 300191 {
    let trade = TickTrade::parse(&buf[8..])?;
    println!("{} traded at {:.4} yuan x {:.0} shares",
        trade.security_id_str(),
        trade.last_px_f64(),
        trade.last_qty_f64(),
    );
}
```

## Design Notes

- Zero external dependencies
- All integers decoded as big-endian per the spec
- Price: `Int64 / 10_000` → yuan (N13(4))
- Quantity: `Int64 / 100` → shares (N15(2))

## Roadmap

- [ ] Snapshot messages (300111, 300211, …)
- [ ] Session-layer Logon / Heartbeat
- [ ] `no_std` support
- [ ] Benchmark vs C++ baseline

## License

MIT
