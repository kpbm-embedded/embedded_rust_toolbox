//! defmt on the BlackPill without an async runtime: all log levels, #[derive(Format)] on structs
//! and enums, display hints (hex, binary, slices, Debug2Format fallback), a cycle-counter
//! timestamp, and compile-time level filtering (DEFMT_LOG=debug in .cargo/config.toml compiles
//! the trace! call out of the binary; examples test checks that in the ELF's defmt table).
//! Logs leave through RTT; `probe-rs run` (see `rust-box plan`) decodes them on the host.
#![no_std]
#![no_main]

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use defmt::{Format, debug, error, info, trace, warn};
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};
use {defmt_rtt as _, panic_probe as _};

// 96 MHz core clock: timestamp in microseconds from the DWT cycle counter
defmt::timestamp!("{=u32:us}", DWT::cycle_count() / 96);

#[derive(Format, Clone, Copy)]
enum Mode {
    Idle,
    Sampling { channel: u8, rate_hz: u16 },
    Fault(u32),
}

#[derive(Format)]
struct Reading {
    raw: u16,
    millivolts: i32,
    flags: u8,
}

fn to_mv(raw: u16) -> i32 {
    (raw as i32 * 3300) / 4095
}

#[entry]
fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.freeze(Config::hse(25.MHz()).sysclk(96.MHz()));
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc13.into_push_pull_output();

    trace!("compiled out by DEFMT_LOG=debug");
    debug!("clocks: sysclk {} Hz", rcc.clocks.sysclk().raw());
    info!("defmt_blackpill_logging: start");
    let modes = [Mode::Idle, Mode::Sampling { channel: 3, rate_hz: 1000 }, Mode::Fault(0xdead_beef)];
    let mut n: u16 = 0;
    loop {
        let m = modes[(n % 3) as usize];
        let r = Reading { raw: n.wrapping_mul(97) % 4096, millivolts: to_mv(n.wrapping_mul(97) % 4096), flags: (n & 0x0f) as u8 };
        info!("#{=u16} mode={} reading={}", n, m, r);
        debug!("flags {=u8:#010b}, raw bytes {=[u8]:#04x}", r.flags, r.raw.to_le_bytes());
        if let Mode::Fault(code) = m {
            warn!("fault code {=u32:#x}", code);
        }
        if r.millivolts > 3200 {
            error!("over-voltage: {} mV", r.millivolts);
        }
        defmt::assert!(r.millivolts <= 3300, "ADC math broke");
        led.toggle();
        cortex_m::asm::delay(48_000_000); // ~0.5 s at 96 MHz
        n = n.wrapping_add(1);
    }
}
