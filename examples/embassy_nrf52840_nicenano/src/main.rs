//! Embassy on the nice!nano / Pro Micro nRF52840: blue LED on P0.15, the on-die temperature sensor read
//! asynchronously (TEMP interrupt), and the RTC1 time driver. Linked for the UF2 bootloader layout.
#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::temp::{self, Temp};
use embassy_nrf::bind_interrupts;
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TEMP => temp::InterruptHandler;
});

#[embassy_executor::task]
async fn thermometer(mut t: Temp<'static>) {
    let mut ticker = Ticker::every(Duration::from_secs(2));
    loop {
        let c = t.read().await; // I30F2 fixed point, 0.25 °C steps
        info!("die temperature {} C", c.to_num::<f32>());
        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("nice!nano nRF52840 app at 0x26000");
    let t = Temp::new(p.TEMP, Irqs);
    spawner.spawn(unwrap!(thermometer(t)));
    let mut led = Output::new(p.P0_15, Level::Low, OutputDrive::Standard);
    loop {
        led.set_high();
        Timer::after_millis(100).await;
        led.set_low();
        Timer::after_millis(900).await;
    }
}

