//! defmt (deferred formatting) without a probe: frames go out over Arm semihosting, QEMU writes them to
//! stdout and defmt-print decodes them with the ELF. Shows levels, derive(Format), display hints and a
//! timestamp; DEFMT_LOG=debug (in .cargo/config.toml) drops trace! at compile time.
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt::{debug, info, trace, warn, Format};
use defmt_semihosting as _;
use panic_semihosting as _;

static TICK: AtomicU32 = AtomicU32::new(0);
defmt::timestamp!("{=u32:us}", TICK.fetch_add(10, Ordering::Relaxed));

#[derive(Format)]
struct Reading {
    channel: u8,
    millivolts: u16,
}

#[derive(Format)]
enum State {
    Idle,
    Sampling { left: u8 },
}

#[entry]
fn main() -> ! {
    info!("defmt over semihosting on qemu lm3s6965evb");
    for ch in 0..3u8 {
        let r = Reading { channel: ch, millivolts: 1000 + u16::from(ch) * 250 };
        debug!("reading {}", r);
    }
    warn!("battery {=u16} mV below {=u16} mV", 3100, 3300);
    info!("state {} then {}", State::Sampling { left: 2 }, State::Idle);
    info!("frame {=[u8]:02x}", [0xde, 0xad, 0xbe, 0xef]);
    trace!("this line is removed at compile time (DEFMT_LOG=debug)");
    info!("done");
    debug::exit(debug::EXIT_SUCCESS);
    loop {}
}
