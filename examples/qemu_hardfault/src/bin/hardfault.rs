//! A fault on purpose (undefined instruction), caught by a HardFault handler that prints the stacked
//! exception frame and the fault status registers (HFSR, CFSR), then exits through semihosting.
#![no_std]
#![no_main]

use cortex_m::peripheral::SCB;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::{debug, hprintln};
use panic_semihosting as _;

#[entry]
fn main() -> ! {
    hprintln!("executing an undefined instruction (udf) on purpose");
    unsafe { core::arch::asm!("udf #0") };
    hprintln!("not reached");
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    // SAFETY: read-only access to the System Control Block fault status registers
    let scb = unsafe { &*SCB::PTR };
    let hfsr = scb.hfsr.read();
    let cfsr = scb.cfsr.read();
    hprintln!("HardFault: pc=0x{:08x} lr=0x{:08x} xpsr=0x{:08x}", ef.pc(), ef.lr(), ef.xpsr());
    hprintln!("HFSR=0x{:08x} FORCED={} CFSR=0x{:08x} UNDEFINSTR={}", hfsr, (hfsr >> 30) & 1, cfsr, (cfsr >> 16) & 1);
    let ok = (cfsr >> 16) & 1 == 1;
    debug::exit(if ok { debug::EXIT_SUCCESS } else { debug::EXIT_FAILURE });
    loop {}
}
