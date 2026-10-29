//! @NAME@: a no_std library. Logic here is unit-tested on the host (`cargo test --target <host>`),
//! cross-compiled for the boards, and logs with defmt when built with `--features defmt`.
#![cfg_attr(not(test), no_std)]

/// Clamp-and-scale: maps `x` in `in_lo..=in_hi` to `out_lo..=out_hi` (integer, saturating).
pub fn map_range(x: i32, in_lo: i32, in_hi: i32, out_lo: i32, out_hi: i32) -> i32 {
    let x = x.clamp(in_lo.min(in_hi), in_lo.max(in_hi));
    let num = (x - in_lo) as i64 * (out_hi - out_lo) as i64;
    out_lo + (num / (in_hi - in_lo) as i64) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    Low,
    Ok,
    High,
}

/// Classifies a reading with hysteresis-free thresholds.
pub fn classify(x: i32, lo: i32, hi: i32) -> Level {
    if x < lo {
        Level::Low
    } else if x > hi {
        Level::High
    } else {
        Level::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_range_ends_and_middle() {
        assert_eq!(map_range(0, 0, 4095, 0, 3300), 0);
        assert_eq!(map_range(4095, 0, 4095, 0, 3300), 3300);
        assert_eq!(map_range(5000, 0, 4095, 0, 3300), 3300);
        assert_eq!(map_range(2048, 0, 4096, 0, 100), 50);
    }

    #[test]
    fn classify_levels() {
        assert_eq!(classify(1, 10, 20), Level::Low);
        assert_eq!(classify(15, 10, 20), Level::Ok);
        assert_eq!(classify(30, 10, 20), Level::High);
    }
}
