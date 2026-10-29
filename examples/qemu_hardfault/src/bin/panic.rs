//! An own #[panic_handler]: prints the location and message over semihosting and exits with
//! code 1, so a test runner (or CI on qemu) sees the failure.
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use cortex_m as _; // links the critical-section implementation (single core)
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

fn checked_reading(raw: u16) -> u16 {
    assert!(raw < 4096, "sensor value out of range: {}", raw);
    raw
}

#[entry]
fn main() -> ! {
    hprintln!("reading 1234 -> {}", checked_reading(1234));
    let bad = core::hint::black_box(4242);
    hprintln!("reading {} -> {}", bad, checked_reading(bad));
    debug::exit(debug::EXIT_SUCCESS);
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        hprintln!("PANIC at {}:{}: {}", loc.file(), loc.line(), info.message());
    }
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}
