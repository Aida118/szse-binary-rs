// src/types/primitives.rs
//!
//! Fixed-point scaling helpers.
//!
//! The SZSE spec encodes decimals as scaled integers, written `Nx(y)`
//! where `y` is the number of fractional digits (数据字典 §5, note 1).
//! The scale differs by field, so convert with the matching helper:
//!
//! | Field        | Spec    | Divisor   | Example                 |
//! |--------------|---------|-----------|-------------------------|
//! | Price        | N13(4)  | 10 000    | `186400` → `18.6400`    |
//! | Qty          | N15(2)  | 100       | `100000` → `1000.00`    |
//! | Amt          | N18(4)  | 10 000    | `1234500` → `123.4500`  |
//! | MDEntryPx    | N18(6)  | 1 000 000 | `18640000` → `18.640000`|
//!
//! These return `f64` for convenience. For exact arithmetic (e.g. risk
//! or accounting) keep the raw `i64` and scale at the very end.

/// Scale a price field, `N13(4)` → yuan (元).
#[inline]
pub fn price_to_f64(raw: i64) -> f64 {
    raw as f64 / 10_000.0
}

/// Scale a quantity field, `N15(2)` → shares/contracts.
#[inline]
pub fn qty_to_f64(raw: i64) -> f64 {
    raw as f64 / 100.0
}

/// Scale an amount field, `N18(4)` → yuan (元).
#[inline]
pub fn amt_to_f64(raw: i64) -> f64 {
    raw as f64 / 10_000.0
}

/// Scale an `MDEntryPx` snapshot price field, `N18(6)`.
///
/// Note this differs from the session-layer `Price` scale: snapshot
/// market-data entries carry six fractional digits, not four
/// (see `t_300611MDEntryItem` etc. in the C++ reference).
#[inline]
pub fn md_entry_px_to_f64(raw: i64) -> f64 {
    raw as f64 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_match_spec_examples() {
        assert_eq!(price_to_f64(186_400), 18.6400);
        assert_eq!(qty_to_f64(100_000), 1000.00);
        assert_eq!(amt_to_f64(1_234_500), 123.45);
        assert_eq!(md_entry_px_to_f64(18_640_000), 18.64);
    }
}
