//! RTIC v2 on the Raspberry Pi Pico (rp2040-hal, Cortex-M0+ source-masking backend): the RP2040
//! 1 MHz TIMER as monotonic (rtic-monotonics), a fast blink task on GP25 and a slower periodic
//! "report" task at a higher priority sharing a counter through a lock.
#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use rtic_monotonics::rp2040::prelude::*;

rp2040_timer_monotonic!(Mono);

/// Second-stage bootloader for the Pico's W25Q16 flash (placed by memory.x).
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[rtic::app(device = rp2040_hal::pac, dispatchers = [SW0_IRQ, SW1_IRQ])]
mod app {
    use super::*;
    use embedded_hal::digital::StatefulOutputPin;
    use rp2040_hal::gpio::{self, bank0::Gpio25, FunctionSioOutput, Pin, PullDown};
    use rp2040_hal::{clocks::init_clocks_and_plls, sio::Sio, watchdog::Watchdog};

    const XTAL_HZ: u32 = 12_000_000;

    #[shared]
    struct Shared {
        toggles: u32,
    }

    #[local]
    struct Local {
        led: Pin<Gpio25, FunctionSioOutput, PullDown>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut resets = cx.device.RESETS;
        let mut watchdog = Watchdog::new(cx.device.WATCHDOG);
        let _clocks = init_clocks_and_plls(
            XTAL_HZ,
            cx.device.XOSC,
            cx.device.CLOCKS,
            cx.device.PLL_SYS,
            cx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();
        Mono::start(cx.device.TIMER, &resets, Priority::rtic_default());
        let sio = Sio::new(cx.device.SIO);
        let pins = gpio::Pins::new(cx.device.IO_BANK0, cx.device.PADS_BANK0, sio.gpio_bank0, &mut resets);
        let led = pins.gpio25.into_push_pull_output();
        defmt::info!("rtic_pico_monotonic: 125 MHz, TIMER monotonic");
        blink::spawn().ok();
        report::spawn().ok();
        (Shared { toggles: 0 }, Local { led })
    }

    #[task(local = [led], shared = [toggles], priority = 1)]
    async fn blink(mut cx: blink::Context) {
        loop {
            let _ = cx.local.led.toggle();
            cx.shared.toggles.lock(|t| *t += 1);
            Mono::delay(250.millis()).await;
        }
    }

    #[task(shared = [toggles], priority = 2)]
    async fn report(mut cx: report::Context) {
        let mut next = Mono::now();
        loop {
            next += 2.secs();
            Mono::delay_until(next).await;
            let t = cx.shared.toggles.lock(|t| *t);
            defmt::info!("{} toggles after {} ms", t, Mono::now().duration_since_epoch().as_millis());
        }
    }
}
