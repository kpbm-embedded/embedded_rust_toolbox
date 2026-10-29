//! Rust firmware that links a Zig static library (zig/dsp.zig) and is called back by it, on QEMU
//! lm3s6965evb: Rust -> Zig (CRC-32, Q15 FIR) and Zig -> Rust (rust_on_block). The CRC is checked
//! against a Rust implementation, the FIR against known values.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

unsafe extern "C" {
    fn zig_crc32(data: *const u8, len: usize) -> u32;
    fn zig_fir_q15(input: *const i16, output: *mut i16, len: usize, block: usize);
}

static BLOCKS: AtomicU32 = AtomicU32::new(0);

/// Called from Zig (declared there as `extern fn rust_on_block`).
#[unsafe(no_mangle)]
pub extern "C" fn rust_on_block(index: u32, peak: i16) {
    BLOCKS.fetch_add(1, Ordering::Relaxed);
    hprintln!("zig -> rust: block {} peak {}", index, peak);
}

fn crc32_rust(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

#[entry]
fn main() -> ! {
    let msg = b"123456789";
    // SAFETY: valid pointer/length pair; the Zig function only reads it
    let z = unsafe { zig_crc32(msg.as_ptr(), msg.len()) };
    let r = crc32_rust(msg);
    hprintln!("crc32(\"123456789\"): zig 0x{:08x} rust 0x{:08x}", z, r);

    let input: [i16; 8] = [400, 400, 400, 400, -800, -800, -800, -800];
    let mut output = [0i16; 8];
    // SAFETY: both buffers hold `len` samples
    unsafe { zig_fir_q15(input.as_ptr(), output.as_mut_ptr(), input.len(), 4) };
    hprintln!("fir: {:?}", output);

    let ok = z == 0xCBF4_3926 && z == r && output == [100, 200, 300, 400, 100, -200, -500, -800] && BLOCKS.load(Ordering::Relaxed) == 2;
    hprintln!("result: {}", if ok { "PASS" } else { "FAIL" });
    debug::exit(if ok { debug::EXIT_SUCCESS } else { debug::EXIT_FAILURE });
    loop {}
}
