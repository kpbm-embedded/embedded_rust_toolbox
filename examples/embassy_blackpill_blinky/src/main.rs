//! Embassy on the WeAct BlackPill (STM32F411CE): blink the LED on PC13 (active low) and change
//! the blink rate with the KEY button on PA0 (EXTI interrupt, awaited, no polling).
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    EXTI0 => exti::InterruptHandler<interrupt::typelevel::EXTI0>;
});

static PERIOD_MS: AtomicU32 = AtomicU32::new(500);

#[embassy_executor::task]
async fn button(mut key: ExtiInput<'static, Async>) {
    loop {
        key.wait_for_falling_edge().await;
        let p = match PERIOD_MS.load(Ordering::Relaxed) {
            500 => 100,
            100 => 1000,
            _ => 500,
        };
        PERIOD_MS.store(p, Ordering::Relaxed);
        info!("KEY: blink period {} ms", p);
        Timer::after_millis(50).await; // debounce
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC13, Level::High, Speed::Low); // high = off
    let key = ExtiInput::new(p.PA0, p.EXTI0, Pull::Up, Irqs);
    spawner.spawn(defmt::unwrap!(button(key)));
    info!("embassy_blackpill_blinky: LED PC13, KEY PA0");
    loop {
        led.toggle();
        Timer::after_millis(PERIOD_MS.load(Ordering::Relaxed) as u64).await;
    }
}
