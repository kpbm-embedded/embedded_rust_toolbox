//! Minimal bootloader: checks the application's vector table at APP_BASE (initial SP inside RAM,
//! reset handler inside the application flash, Thumb bit set), then moves VTOR to it and jumps.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

const APP_BASE: u32 = 0x0000_8000;
const APP_END: u32 = 0x0004_0000;
const RAM: core::ops::Range<u32> = 0x2000_0000..0x2001_0001;

#[entry]
fn main() -> ! {
    // SAFETY: APP_BASE is mapped flash on this chip; we only read two words
    let (sp, reset) = unsafe {
        let vt = APP_BASE as *const u32;
        (vt.read_volatile(), vt.add(1).read_volatile())
    };
    hprintln!("boot: app vector table at 0x{:08x}: sp=0x{:08x} reset=0x{:08x}", APP_BASE, sp, reset);
    let valid = RAM.contains(&sp) && (APP_BASE..APP_END).contains(&(reset & !1)) && reset & 1 == 1;
    if !valid {
        hprintln!("boot: no valid application, staying in the bootloader");
        debug::exit(debug::EXIT_FAILURE);
        loop {}
    }
    hprintln!("boot: jumping to the application");
    let p = cortex_m::Peripherals::take().unwrap();
    // SAFETY: the vector table was validated above; bootload sets MSP and branches to the reset handler
    unsafe {
        p.SCB.vtor.write(APP_BASE);
        cortex_m::asm::bootload(APP_BASE as *const u32)
    }
}
