//! Embassy on the Raspberry Pi Pico (RP2040): blink the on-board LED (GP25) from one async task
//! and print a heartbeat from a second task, both driven by embassy-time (no busy waits).
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn heartbeat() {
    let mut ticker = Ticker::every(Duration::from_secs(5));
    let mut n: u32 = 0;
    loop {
        ticker.next().await;
        n += 1;
        info!("heartbeat {} (uptime {} ms)", n, embassy_time::Instant::now().as_millis());
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);
    spawner.spawn(defmt::unwrap!(heartbeat()));
    info!("embassy_pico_blinky: LED on GP25");
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
