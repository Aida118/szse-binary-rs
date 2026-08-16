# szse-binary-rs

[![CI](https://github.com/Aida118/szse-binary-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Aida118/szse-binary-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)

**English** | [简体中文](README.zh-CN.md)

A dependency-free Rust parser for the **Shenzhen Stock Exchange (SZSE) Binary
market-data protocol** (深圳证券交易所 Binary 行情数据接口规范).

Decodes the session-layer handshake and the real-time market-data feed
(snapshots, tick-by-tick orders and trades) that an SZSE MDGW gateway sends
to a client (VSS). Zero runtime dependencies.

> Status: usable but pre-1.0. Field layouts are derived from a production C++
> decoder and the official spec; feedback and PRs on real captured frames are
> very welcome.

## Supported messages

| Message | MsgType | Status |
|---------|---------|--------|
| Message header 消息头 | — | ✅ |
| Logon 登录 | 1 | ✅ |
| Logout 注销 (+ session status) | 2 | ✅ |
| Heartbeat 心跳 | 3 | ✅ |
| Business reject 业务拒绝 | 8 | ✅ |
| Resend 重传 | 390094 | ✅ |
| Channel heartbeat 频道心跳 | 390095 | ✅ |
| Tick trade 逐笔成交 | 300191 / 300591 / 300791 / … | ✅ |
| Tick order 逐笔委托 (+ 300192/300592/300792 ext) | 300192 / 300592 / 300792 / … | ✅ |
| Auction snapshot 集中竞价快照 (full depth book) | 300111 | ✅ |
| After-hours snapshot 盘后定价快照 | 300611 / 303711 | ✅ |
| Index snapshot 指数快照 | 309011 | ✅ |
| Volume-stat snapshot 成交量统计 | 309111 | ✅ |
| Security status 证券实时状态 | 390013 | 🔜 |

## Quick start

```rust
use szse_binary_rs::{parse_frame, Message};

// `frame` = one full message read off the feed: header + body
match parse_frame(&frame)? {
    Message::TickTrade(t) => println!(
        "{} traded {:.0} shares @ {:.4} yuan",
        t.security_id_str(), t.last_qty_f64(), t.last_px_f64(),
    ),
    Message::Snapshot(s) => println!("snapshot for {}", s.header.security_id_str()),
    Message::TickOrder { order, .. } => println!("order on {}", order.security_id_str()),
    other => println!("{other:?}"),
}
```

Or parse a specific body directly when you already know the type:

```rust
use szse_binary_rs::{MsgHeader, TickTrade, MSG_HEADER_LEN};

let header = MsgHeader::parse(&buf[..MSG_HEADER_LEN])?;
if header.msg_type == 300191 {
    let trade = TickTrade::parse(&buf[MSG_HEADER_LEN..])?;
}
```

## Design notes

- **Zero external dependencies** (only `criterion` as a dev-dependency for benches).
- All integers are **big-endian** per the spec (数据字典 §5).
- Decimals are scaled integers, `Nx(y)`. Scales differ by field:
  - Price `N13(4)` → `/10_000` (yuan)
  - Qty `N15(2)` → `/100`
  - Amt `N18(4)` → `/10_000`
  - `MDEntryPx` `N18(6)` → `/1_000_000`
  Use the `*_f64()` accessors or the helpers in `types::primitives`. Keep the
  raw `i64` when you need exact arithmetic.
- Strings are UTF-8, space-padded; `*_str()` accessors trim them.
- `parse_frame` does **not** verify the checksum (most transports already
  guarantee integrity). Call `verify_checksum` if you need it (§4.1.2).

## Roadmap

- [x] Session-layer messages (Logon, Logout, Heartbeat)
- [x] Common messages (channel heartbeat, resend, business reject)
- [x] Snapshot messages (300111, 300611, 303711, 309011, 309111)
- [x] Tick order extension fields (300192 / 300592 / 300792)
- [x] Top-level `Message` enum + `parse_frame` dispatch
- [ ] Security status (390013) and market status (390019)
- [ ] Bond-specific snapshots/ticks (300211, 300292, 300392, …)
- [ ] `no_std` support
- [ ] Fuzz harness over captured frames

## License

MIT
