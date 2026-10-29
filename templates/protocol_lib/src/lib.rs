//! A wire protocol for a UART/USB link without an allocator: messages are serde types, encoded with
//! postcard and framed with COBS (0x00 ends a frame, so a receiver can resync after noise).
//! heapless gives fixed-capacity Vec/String, an SPSC queue to hand frames from an ISR to the main loop,
//! and a byte-wise frame accumulator. Same code on the Pico, the BlackPill and a RISC-V MCU.
#![no_std]

use heapless::spsc::Queue;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub const MAX_FRAME: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    Ping(u32),
    Reading { channel: u8, millivolts: u16, t_ms: u32 },
    Log(String<32>),
    Batch(Vec<i16, 8>),
}

/// Encode into a COBS frame (with the trailing 0x00). postcard 1.x builds on heapless 0.7, so it
/// serializes into a plain buffer and the frame is copied into this crate's heapless 0.9 Vec.
pub fn encode(m: &Msg) -> Result<Vec<u8, MAX_FRAME>, postcard::Error> {
    let mut buf = [0u8; MAX_FRAME];
    let used = postcard::to_slice_cobs(m, &mut buf)?;
    Vec::from_slice(used).map_err(|_| postcard::Error::SerializeBufferFull)
}

/// Collects bytes until 0x00 and decodes the frame; noise between frames is dropped.
#[derive(Default)]
pub struct FrameReader {
    buf: Vec<u8, MAX_FRAME>,
    pub dropped: u32,
}

impl FrameReader {
    pub fn push(&mut self, b: u8) -> Option<Msg> {
        if self.buf.push(b).is_err() {
            self.buf.clear(); // overflow: drop and resync at the next 0x00
            self.dropped += 1;
            return None;
        }
        if b != 0 {
            return None;
        }
        let r = postcard::from_bytes_cobs::<Msg>(&mut self.buf).ok();
        if r.is_none() {
            self.dropped += 1;
        }
        self.buf.clear();
        r
    }
}

/// ISR -> main hand-off: the producer half lives in the interrupt, the consumer in the main loop.
pub type FrameQueue = Queue<Vec<u8, MAX_FRAME>, 4>;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use core::fmt::Write;

    fn roundtrip(m: Msg) {
        let f = encode(&m).unwrap();
        assert_eq!(*f.last().unwrap(), 0);
        assert!(!f[..f.len() - 1].contains(&0), "COBS frame has no zero inside");
        let mut r = FrameReader::default();
        let got: std::vec::Vec<Msg> = f.iter().filter_map(|&b| r.push(b)).collect();
        assert_eq!(got, std::vec![m]);
    }

    #[test]
    fn all_variants_roundtrip() {
        roundtrip(Msg::Ping(0));
        roundtrip(Msg::Reading { channel: 3, millivolts: 3300, t_ms: 123_456 });
        let mut s = String::new();
        write!(s, "temp {} C", -12).unwrap();
        roundtrip(Msg::Log(s));
        roundtrip(Msg::Batch(Vec::from_slice(&[0, -1, 256, i16::MIN, i16::MAX]).unwrap()));
    }

    #[test]
    fn postcard_is_compact() {
        // varint encoding: tag 1 + channel 1 + 3300 (2 bytes) + 123456 (3 bytes) = 7, + COBS overhead 2
        let f = encode(&Msg::Reading { channel: 3, millivolts: 3300, t_ms: 123_456 }).unwrap();
        assert_eq!(f.len(), 9);
    }

    #[test]
    fn reader_resyncs_after_noise() {
        let mut stream = std::vec![0x13, 0x37, 0x00]; // garbage frame
        stream.extend_from_slice(&encode(&Msg::Ping(7)).unwrap());
        let mut r = FrameReader::default();
        let got: std::vec::Vec<Msg> = stream.iter().filter_map(|&b| r.push(b)).collect();
        assert_eq!(got, std::vec![Msg::Ping(7)]);
        assert_eq!(r.dropped, 1);
    }

    #[test]
    fn capacity_errors_instead_of_panics() {
        let mut v: Vec<i16, 8> = Vec::new();
        for i in 0..8 {
            v.push(i).unwrap();
        }
        assert!(v.push(9).is_err());
        let mut s: String<32> = String::new();
        assert!(s.push_str(&"x".repeat(33)).is_err());
    }

    #[test]
    fn spsc_queue_isr_to_main() {
        let mut q: FrameQueue = Queue::new();
        let (mut prod, mut cons) = q.split();
        for n in 0..3 {
            prod.enqueue(encode(&Msg::Ping(n)).unwrap()).unwrap();
        }
        let mut r = FrameReader::default();
        let mut n = 0;
        while let Some(f) = cons.dequeue() {
            let m = f.iter().find_map(|&b| r.push(b)).unwrap();
            assert_eq!(m, Msg::Ping(n));
            n += 1;
        }
        assert_eq!(n, 3);
    }
}
