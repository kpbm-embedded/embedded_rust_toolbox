//! One Embassy firmware for three WeAct STM32 boards (G431 Cortex-M4F, H562 Cortex-M33, H743 Cortex-M7):
//! the chip comes from a Cargo feature, embassy-stm32 generates the PAC, clocks, memory.x and the
//! time driver for it. A heartbeat task and a Signal-driven LED task; board pins differ per chip.
#![no_std]
#![no_main]

#[cfg(not(any(feature = "g431", feature = "h562", feature = "h743")))]
compile_error!("pick one board feature: g431, h562 or h743");

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_stm32::Peri;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "g431")]
const BOARD: &str = "WeAct G431CB";
#[cfg(feature = "h562")]
const BOARD: &str = "WeAct H562RG";
#[cfg(feature = "h743")]
const BOARD: &str = "WeAct H743VI";

static BLINK: Signal<ThreadModeRawMutex, u32> = Signal::new();

#[embassy_executor::task]
async fn led_task(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::Low, Speed::Low);
    loop {
        let n = BLINK.wait().await;
        for _ in 0..n {
            led.set_high();
            Timer::after_millis(50).await;
            led.set_low();
            Timer::after_millis(50).await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("{}: embassy-stm32, flash {} KiB", BOARD, embassy_stm32::flash::FLASH_SIZE / 1024);
    // user LED: G431 core board PC6, H562 core board PB2, MiniSTM32H7xx PE3 (check your board revision)
    #[cfg(feature = "g431")]
    let led = p.PC6.into();
    #[cfg(feature = "h562")]
    let led = p.PB2.into();
    #[cfg(feature = "h743")]
    let led = p.PE3.into();
    spawner.spawn(defmt::unwrap!(led_task(led)));

    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut n = 0u32;
    loop {
        ticker.next().await;
        n += 1;
        BLINK.signal(1 + n % 3);
        info!("heartbeat {}", n);
    }
}
