//! Bare-metal Cortex-M3 firmware that runs on QEMU: semihosting output, the CPUID register, a
//! SysTick-timed busy loop and a semihosting exit code (so a test can check the result).
#![no_std]
#![no_main]

use cortex_m::peripheral::{syst::SystClkSource, Peripherals};
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

fn fib(n: u32) -> u32 {
    let (mut a, mut b) = (0u32, 1u32);
    for _ in 0..n {
        (a, b) = (b, a.wrapping_add(b));
    }
    a
}

#[entry]
fn main() -> ! {
    let mut p = Peripherals::take().unwrap();
    let cpuid = p.CPUID.base.read();
    hprintln!("hello from Rust on qemu lm3s6965evb");
    hprintln!("CPUID 0x{:08x} partno 0x{:03x}", cpuid, (cpuid >> 4) & 0xfff);

    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(0x00ff_ffff);
    p.SYST.clear_current();
    p.SYST.enable_counter();
    let f = fib(30);
    hprintln!("fib(30) = {}", f);
    let ok = f == 832_040;
    hprintln!("result: {}", if ok { "PASS" } else { "FAIL" });
    debug::exit(if ok { debug::EXIT_SUCCESS } else { debug::EXIT_FAILURE });
    loop {
        cortex_m::asm::wfi();
    }
}
