# szse-binary-rs

[![CI](https://github.com/Aida118/szse-binary-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Aida118/szse-binary-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)

[English](README.md) | **简体中文**

一个零依赖的 Rust 库，用于解析**深圳证券交易所（SZSE）Binary 行情数据接口规范**。

解码会话层握手消息与实时行情数据（快照、逐笔委托、逐笔成交），
数据由深交所 MDGW 网关下发给客户端（VSS）。运行时零依赖。

> 状态：可用，但尚未到 1.0。字段布局依据一份生产环境的 C++ 解码器与官方
> 规范推导而来；欢迎针对真实抓包帧提交反馈和 PR。

## 支持的消息

| 消息 | MsgType | 状态 |
|------|---------|------|
| 消息头 | — | ✅ |
| 登录 Logon | 1 | ✅ |
| 注销 Logout（含会话状态） | 2 | ✅ |
| 心跳 Heartbeat | 3 | ✅ |
| 业务拒绝 Business reject | 8 | ✅ |
| 重传 Resend | 390094 | ✅ |
| 频道心跳 Channel heartbeat | 390095 | ✅ |
| 逐笔成交 Tick trade | 300191 / 300591 / 300791 / … | ✅ |
| 逐笔委托 Tick order（含 300192/300592/300792 扩展） | 300192 / 300592 / 300792 / … | ✅ |
| 集中竞价快照（完整深度盘口） | 300111 | ✅ |
| 盘后定价快照 | 300611 / 303711 | ✅ |
| 指数快照 | 309011 | ✅ |
| 成交量统计快照 | 309111 | ✅ |
| 证券实时状态 | 390013 | 🔜 |

## 快速上手

```rust
use szse_binary_rs::{parse_frame, Message};

// `frame` = 从行情流读到的一条完整消息：消息头 + 消息体
match parse_frame(&frame)? {
    Message::TickTrade(t) => println!(
        "{} 成交 {:.0} 股 @ {:.4} 元",
        t.security_id_str(), t.last_qty_f64(), t.last_px_f64(),
    ),
    Message::Snapshot(s) => println!("{} 的快照", s.header.security_id_str()),
    Message::TickOrder { order, .. } => println!("{} 的委托", order.security_id_str()),
    other => println!("{other:?}"),
}
```

当你已经知道消息类型时，也可以直接解析对应的消息体：

```rust
use szse_binary_rs::{MsgHeader, TickTrade, MSG_HEADER_LEN};

let header = MsgHeader::parse(&buf[..MSG_HEADER_LEN])?;
if header.msg_type == 300191 {
    let trade = TickTrade::parse(&buf[MSG_HEADER_LEN..])?;
}
```

## 设计说明

- **零外部依赖**（仅 `criterion` 作为 benchmark 的开发依赖）。
- 所有整数均为**大端序**（数据字典 §5）。
- 小数用放大后的整数表示，记作 `Nx(y)`。不同字段的放大倍数不同：
  - 价格 `N13(4)` → `/10_000`（元）
  - 数量 `N15(2)` → `/100`
  - 金额 `N18(4)` → `/10_000`
  - `MDEntryPx` `N18(6)` → `/1_000_000`

  可用 `*_f64()` 访问器或 `types::primitives` 里的辅助函数。需要精确运算时请保留原始 `i64`。
- 字符串为 UTF-8、右侧空格填充；`*_str()` 访问器会自动去除填充。
- `parse_frame` **不校验**校验和（多数传输层已保证完整性）。需要时请另行调用 `verify_checksum`（§4.1.2）。

## 项目定位

本库刻意只做**解码**，对照两份深交所规范：

- **Binary 行情数据接口规范**（实时流）→ `src/messages`，已完成大部分。
- **数据文件交换接口规范**（基础文件：`securities.xml`、交易参数、`mktdt00`、DBF 等）→ 规划中的 `files` 模块。

网络接收与向订阅者下发属于环境耦合逻辑，不纳入本库，仅通过 examples 演示。

## 路线图

- [x] 会话层消息（登录、注销、心跳）
- [x] 公共消息（频道心跳、重传、业务拒绝）
- [x] 快照消息（300111、300611、303711、309011、309111）
- [x] 逐笔委托扩展字段（300192 / 300592 / 300792）
- [x] 顶层 `Message` 枚举 + `parse_frame` 分发
- [ ] 证券实时状态（390013）与市场实时状态（390019）
- [ ] 债券快照/逐笔（300211、300292、300392、…）
- [ ] 基础文件解码（`securities.xml`、`cashauctionparams`、`mktdt00`、DBF）
- [ ] `no_std` 支持
- [ ] 针对真实抓包帧的 fuzz 测试

## 许可证

MIT
