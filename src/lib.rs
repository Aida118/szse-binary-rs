// src/lib.rs
//! # szse-binary-rs
//!
//! A dependency-free parser for the **Shenzhen Stock Exchange (SZSE) Binary
//! market-data protocol** (深圳证券交易所 Binary 行情数据接口规范).
//!
//! Every message is `header (8B) + body + checksum tail (4B)`. The header
//! carries `MsgType` and `BodyLength`; this crate decodes the body for each
//! supported `MsgType` into a typed struct.
//!
//! ## Quick start
//!
//! ```
//! use szse_binary_rs::{Message, parse_frame};
//!
//! # fn demo(frame: &[u8]) -> Result<(), szse_binary_rs::ParseError> {
//! // `frame` = one full message: header + body (+ optional checksum tail)
//! match parse_frame(frame)? {
//!     Message::TickTrade(t) => {
//!         println!("{} @ {:.4} x {:.0}", t.security_id_str(), t.last_px_f64(), t.last_qty_f64());
//!     }
//!     Message::Snapshot(s) => println!("snapshot for {}", s.header.security_id_str()),
//!     other => println!("got {other:?}"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Encoding notes
//!
//! - All integers are **big-endian** (数据字典 §5).
//! - Decimals are scaled integers, `Nx(y)`. See [`types::primitives`] for the
//!   per-field divisors (price `/10_000`, qty `/100`, `MDEntryPx` `/1_000_000`).
//! - Strings are UTF-8, right-padded with spaces; use the `*_str()` accessors.

mod error;
mod header;
mod utils;

pub mod messages;
pub mod types;

pub use error::ParseError;
pub use header::{MSG_HEADER_LEN, MsgHeader};
pub use messages::*;
pub use types::*;
pub use utils::{checksum, require_len, trimmed_str};

/// A decoded SZSE Binary message: header type + typed body.
///
/// Tick orders and snapshots need the `MsgType` to pick their extension, so
/// the dispatcher in [`parse_frame`] passes it through for you.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Logon(Logon),
    Logout(Logout),
    Heartbeat,
    ChannelHeartbeat(ChannelHeartbeat),
    Resend(Resend),
    BusinessReject(BusinessReject),
    /// Tick-by-tick trade plus the message type it arrived under.
    TickTrade(TickTrade),
    /// Tick-by-tick order with its stream-specific extension (if any).
    TickOrder {
        order: TickOrder,
        extension: Option<OrderExtension>,
    },
    Snapshot(Snapshot),
}

/// Parse one complete frame: read the 8-byte header, then decode the body
/// according to its `MsgType`.
///
/// `frame` must start at the message header. A trailing 4-byte checksum is
/// tolerated but not required — the body is sliced using `BodyLength` when
/// present, otherwise it extends to the end of `frame`. Unknown message
/// types yield [`ParseError::UnknownMsgType`].
///
/// This does **not** verify the checksum; call [`verify_checksum`] separately
/// if your transport does not already guarantee integrity.
pub fn parse_frame(frame: &[u8]) -> Result<Message, ParseError> {
    let header = MsgHeader::parse(frame)?;
    let body_len = header.body_length as usize;
    let avail = frame.len() - MSG_HEADER_LEN;
    // Trust BodyLength when it fits; otherwise fall back to whatever remains.
    let end = MSG_HEADER_LEN + body_len.min(avail);
    let body = &frame[MSG_HEADER_LEN..end];

    let msg = match header.msg_type {
        1 => Message::Logon(Logon::parse(body)?),
        2 => Message::Logout(Logout::parse(body)?),
        3 => Message::Heartbeat,
        390095 => Message::ChannelHeartbeat(ChannelHeartbeat::parse(body)?),
        390094 => Message::Resend(Resend::parse(body)?),
        8 => Message::BusinessReject(BusinessReject::parse(body)?),
        300191 | 300591 | 300791 | 300291 | 300391 | 300491 => {
            Message::TickTrade(TickTrade::parse(body)?)
        }
        300192 | 300592 | 300792 | 300292 => {
            let (order, extension) = TickOrder::parse_with_ext(body, header.msg_type)?;
            Message::TickOrder { order, extension }
        }
        300111 | 300611 | 303711 | 309011 | 309111 => {
            Message::Snapshot(Snapshot::parse(body, header.msg_type)?)
        }
        other => return Err(ParseError::UnknownMsgType(other)),
    };
    Ok(msg)
}

/// Verify the 4-byte checksum tail of a full frame (§4.1.2).
///
/// `frame` must include the trailing checksum: `header + body + checksum(4)`.
/// The checksum covers `header + body` (everything but the last 4 bytes).
pub fn verify_checksum(frame: &[u8]) -> Result<(), ParseError> {
    require_len(frame, MSG_HEADER_LEN + 4)?;
    let split = frame.len() - 4;
    let declared = u32::from_be_bytes(frame[split..].try_into().unwrap());
    let computed = checksum(&frame[..split]);
    if declared == computed {
        Ok(())
    } else {
        Err(ParseError::ChecksumMismatch { declared, computed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn framed(msg_type: u32, body: &[u8]) -> Vec<u8> {
        let mut f = Vec::new();
        f.extend_from_slice(&msg_type.to_be_bytes());
        f.extend_from_slice(&(body.len() as u32).to_be_bytes());
        f.extend_from_slice(body);
        f
    }

    #[test]
    fn dispatches_heartbeat() {
        let frame = framed(3, &[]);
        assert_eq!(parse_frame(&frame).unwrap(), Message::Heartbeat);
    }

    #[test]
    fn dispatches_tick_trade() {
        let mut body = vec![0u8; TICK_TRADE_BODY_LEN];
        body[0..2].copy_from_slice(&2011u16.to_be_bytes());
        body[57] = b'F';
        let frame = framed(300191, &body);
        match parse_frame(&frame).unwrap() {
            Message::TickTrade(t) => assert_eq!(t.channel_no, 2011),
            other => panic!("expected TickTrade, got {other:?}"),
        }
    }

    #[test]
    fn unknown_type_errors() {
        let frame = framed(424242, &[]);
        assert_eq!(parse_frame(&frame), Err(ParseError::UnknownMsgType(424242)));
    }

    #[test]
    fn checksum_round_trips() {
        let mut frame = framed(3, &[]);
        let cks = checksum(&frame);
        frame.extend_from_slice(&cks.to_be_bytes());
        assert!(verify_checksum(&frame).is_ok());

        // corrupt the body type and the checksum must fail
        frame[0] ^= 0xFF;
        assert!(matches!(
            verify_checksum(&frame),
            Err(ParseError::ChecksumMismatch { .. })
        ));
    }
}
