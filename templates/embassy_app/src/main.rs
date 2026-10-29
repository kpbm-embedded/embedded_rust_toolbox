//! @NAME@: Embassy app for the Raspberry Pi Pico (RP2040). Two tasks: LED blink on GP25 and a
//! periodic log line. Change `embassy-rp` features / memory.x for other RP2040 boards.
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn status() {
    let mut t = Ticker::every(Duration::from_secs(2));
    loop {
        t.next().await;
        info!("@NAME@ alive, uptime {} ms", embassy_time::Instant::now().as_millis());
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);
    spawner.spawn(defmt::unwrap!(status()));
    loop {
        led.toggle();
        Timer::after_millis(250).await;
    }
}
