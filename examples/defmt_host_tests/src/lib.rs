//! no_std building blocks that firmware uses and the host tests: a fixed-size moving average,
//! CRC-8/SMBUS and COBS framing (zero-free packets for a serial link). Pure logic, no HAL, so
//! `cargo test` runs it on the PC in milliseconds; with `--features defmt` the types get
//! `defmt::Format` for logging on the target.
#![cfg_attr(not(test), no_std)]

/// Moving average over the last `N` samples (integer, no float unit needed).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MovingAverage<const N: usize> {
    buf: [i32; N],
    pos: usize,
    len: usize,
    sum: i64,
}

impl<const N: usize> Default for MovingAverage<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> MovingAverage<N> {
    pub const fn new() -> Self {
        Self { buf: [0; N], pos: 0, len: 0, sum: 0 }
    }
    /// Adds a sample and returns the current average.
    pub fn push(&mut self, x: i32) -> i32 {
        if self.len == N {
            self.sum -= self.buf[self.pos] as i64;
        } else {
            self.len += 1;
        }
        self.buf[self.pos] = x;
        self.sum += x as i64;
        self.pos = (self.pos + 1) % N;
        (self.sum / self.len as i64) as i32
    }
}

/// CRC-8/SMBUS (poly 0x07, init 0x00), as used by many I2C sensors.
pub fn crc8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |mut crc, &b| {
        crc ^= b;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x07 } else { crc << 1 };
        }
        crc
    })
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CobsError {
    OutputTooSmall,
    Malformed,
}

/// COBS-encodes `input` into `out` (no trailing 0x00 delimiter); returns the encoded length.
pub fn cobs_encode(input: &[u8], out: &mut [u8]) -> Result<usize, CobsError> {
    let mut code_pos = 0;
    let mut code = 1u8;
    let mut o = 1;
    for &b in input {
        if o >= out.len() {
            return Err(CobsError::OutputTooSmall);
        }
        if b == 0 {
            out[code_pos] = code;
            code_pos = o;
            o += 1;
            code = 1;
        } else {
            out[o] = b;
            o += 1;
            code += 1;
            if code == 0xff {
                out[code_pos] = code;
                code_pos = o;
                o += 1;
                code = 1;
            }
        }
    }
    if code_pos >= out.len() {
        return Err(CobsError::OutputTooSmall);
    }
    out[code_pos] = code;
    Ok(o)
}

/// Decodes a COBS frame (without the 0x00 delimiter) into `out`; returns the decoded length.
pub fn cobs_decode(input: &[u8], out: &mut [u8]) -> Result<usize, CobsError> {
    let (mut i, mut o) = (0, 0);
    while i < input.len() {
        let code = input[i] as usize;
        if code == 0 || i + code > input.len() {
            return Err(CobsError::Malformed);
        }
        i += 1;
        for _ in 1..code {
            if o >= out.len() {
                return Err(CobsError::OutputTooSmall);
            }
            out[o] = input[i];
            o += 1;
            i += 1;
        }
        if code != 0xff && i < input.len() {
            if o >= out.len() {
                return Err(CobsError::OutputTooSmall);
            }
            out[o] = 0;
            o += 1;
        }
    }
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_average_fills_then_slides() {
        let mut m = MovingAverage::<4>::new();
        assert_eq!(m.push(4), 4);
        assert_eq!(m.push(8), 6);
        assert_eq!(m.push(12), 8);
        assert_eq!(m.push(16), 10);
        assert_eq!(m.push(20), 14); // 8+12+16+20
    }

    #[test]
    fn moving_average_negative() {
        let mut m = MovingAverage::<2>::new();
        m.push(-10);
        assert_eq!(m.push(-20), -15);
    }

    #[test]
    fn crc8_known_vectors() {
        assert_eq!(crc8(b""), 0x00);
        assert_eq!(crc8(b"123456789"), 0xF4); // CRC-8/SMBUS check value
    }

    #[test]
    fn cobs_reference_vectors() {
        let cases: &[(&[u8], &[u8])] = &[
            (&[0x00], &[0x01, 0x01]),
            (&[0x00, 0x00], &[0x01, 0x01, 0x01]),
            (&[0x11, 0x22, 0x00, 0x33], &[0x03, 0x11, 0x22, 0x02, 0x33]),
            (&[0x11, 0x22, 0x33, 0x44], &[0x05, 0x11, 0x22, 0x33, 0x44]),
        ];
        for (raw, enc) in cases {
            let mut buf = [0u8; 16];
            let n = cobs_encode(raw, &mut buf).unwrap();
            assert_eq!(&buf[..n], *enc);
            let mut dec = [0u8; 16];
            let m = cobs_decode(enc, &mut dec).unwrap();
            assert_eq!(&dec[..m], *raw);
        }
    }

    #[test]
    fn cobs_roundtrip_long_and_no_zero_in_output() {
        let raw: Vec<u8> = (0..600u32).map(|i| (i % 7) as u8).collect();
        let mut enc = vec![0u8; 700];
        let n = cobs_encode(&raw, &mut enc).unwrap();
        assert!(!enc[..n].contains(&0));
        let mut dec = vec![0u8; 700];
        let m = cobs_decode(&enc[..n], &mut dec).unwrap();
        assert_eq!(&dec[..m], &raw[..]);
    }

    #[test]
    fn cobs_errors() {
        let mut tiny = [0u8; 2];
        assert_eq!(cobs_encode(&[1, 2, 3], &mut tiny), Err(CobsError::OutputTooSmall));
        let mut out = [0u8; 8];
        assert_eq!(cobs_decode(&[0x00], &mut out), Err(CobsError::Malformed));
        assert_eq!(cobs_decode(&[0x05, 0x11], &mut out), Err(CobsError::Malformed));
    }
}
