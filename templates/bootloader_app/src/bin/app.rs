//! The application linked at 0x8000. Proves it runs from its own vector table: VTOR points at it and
//! its SysTick handler (only in this image's table) fires.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

static TICKS: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

#[entry]
fn main() -> ! {
    let mut p = cortex_m::Peripherals::take().unwrap();
    let vtor = p.SCB.vtor.read();
    hprintln!("app: running, VTOR=0x{:08x}", vtor);
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(12_000 - 1);
    p.SYST.clear_current();
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();
    while TICKS.load(Ordering::Relaxed) < 3 {
        cortex_m::asm::wfi();
    }
    hprintln!("app: {} SysTick interrupts through the relocated vector table", TICKS.load(Ordering::Relaxed));
    let ok = vtor == 0x8000;
    debug::exit(if ok { debug::EXIT_SUCCESS } else { debug::EXIT_FAILURE });
    loop {}
}
