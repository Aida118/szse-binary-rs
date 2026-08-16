// src/utils.rs
use crate::error::ParseError;

/// Ensure `buf` holds at least `needed` bytes, else `BufferTooShort`.
#[inline]
pub fn require_len(buf: &[u8], needed: usize) -> Result<(), ParseError> {
    if buf.len() < needed {
        Err(ParseError::BufferTooShort {
            needed,
            got: buf.len(),
        })
    } else {
        Ok(())
    }
}

// ── Big-endian fixed-width readers ──────────────────────────────
//
// The SZSE Binary protocol declares all integers as big-endian
// (高字节序 / BIG-ENDIAN, 数据字典 §5). These helpers assume the
// caller has already bounds-checked via `require_len`; offsets come
// from the byte-packed C++ structs, so they are always in range.

#[inline]
pub(crate) fn be_u8(buf: &[u8], off: usize) -> u8 {
    buf[off]
}

#[inline]
pub(crate) fn be_u16(buf: &[u8], off: usize) -> u16 {
    u16::from_be_bytes(buf[off..off + 2].try_into().unwrap())
}

#[inline]
pub(crate) fn be_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_be_bytes(buf[off..off + 4].try_into().unwrap())
}

#[inline]
pub(crate) fn be_i32(buf: &[u8], off: usize) -> i32 {
    i32::from_be_bytes(buf[off..off + 4].try_into().unwrap())
}

#[inline]
pub(crate) fn be_i64(buf: &[u8], off: usize) -> i64 {
    i64::from_be_bytes(buf[off..off + 8].try_into().unwrap())
}

/// Copy a fixed-size byte field into an owned array.
#[inline]
pub(crate) fn fixed<const N: usize>(buf: &[u8], off: usize) -> [u8; N] {
    buf[off..off + N].try_into().unwrap()
}

/// Decode a space-padded ASCII field into a trimmed `&str`.
///
/// SZSE strings are UTF-8 and right-padded with spaces (§5 note 2);
/// invalid bytes yield an empty string rather than panicking.
#[inline]
pub fn trimmed_str(buf: &[u8]) -> &str {
    std::str::from_utf8(buf).unwrap_or("").trim_end()
}

/// Compute the message checksum exactly as the spec's reference code
/// (§4.1.2): the unsigned 8-bit sum of every byte from the start of the
/// header through the end of the body, taken mod 256.
///
/// `frame` must span `[header .. end-of-body]` — i.e. exclude the 4-byte
/// checksum tail itself.
pub fn checksum(frame: &[u8]) -> u32 {
    let mut cks: u32 = 0;
    for &b in frame {
        cks = cks.wrapping_add(b as u32);
    }
    cks % 256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_len_reports_shortfall() {
        assert_eq!(
            require_len(&[0u8; 3], 8),
            Err(ParseError::BufferTooShort { needed: 8, got: 3 })
        );
        assert!(require_len(&[0u8; 8], 8).is_ok());
    }

    #[test]
    fn checksum_matches_reference() {
        // sum = 1+2+3 = 6, 6 % 256 = 6
        assert_eq!(checksum(&[1, 2, 3]), 6);
        // wraps mod 256: 255 + 255 = 510, 510 % 256 = 254
        assert_eq!(checksum(&[255, 255]), 254);
    }

    #[test]
    fn trimmed_str_strips_trailing_spaces() {
        assert_eq!(trimmed_str(b"000001  "), "000001");
        assert_eq!(trimmed_str(b"   "), "");
    }
}
