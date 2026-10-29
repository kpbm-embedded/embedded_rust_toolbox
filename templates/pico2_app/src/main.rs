//! Embassy on the Raspberry Pi Pico 2 (RP2350, Cortex-M33): the IMAGE_DEF block the boot ROM needs,
//! picotool binary-info entries, LED GP25 and a PWM "breathing" task using the FPU (thumbv8m.main-hf).
#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pwm::{Config as PwmConfig, Pwm, SetDutyCycle};
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

/// Tells the RP2350 boot ROM this is a secure Arm executable (without it the ROM does not boot it).
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

/// Shown by `picotool info` (program name, version, description).
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 3] = [
    embassy_rp::binary_info::rp_program_name!(c"embassy_pico2"),
    embassy_rp::binary_info::rp_program_description!(c"Box embedded-rust: Pico 2 blink + PWM breathe"),
    embassy_rp::binary_info::rp_cargo_version!(),
];

#[embassy_executor::task]
async fn breathe(mut pwm: Pwm<'static>) {
    let mut ticker = Ticker::every(Duration::from_millis(20));
    let mut phase = 0.0f32;
    loop {
        // triangle wave squared: a cheap perceptual brightness curve, computed on the M33's FPU
        let tri = if phase < 1.0 { phase } else { 2.0 - phase };
        let duty = (tri * tri * 100.0) as u8;
        let _ = pwm.set_duty_cycle_percent(duty.min(100));
        phase = (phase + 0.02) % 2.0;
        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Pico 2 (RP2350) up");
    let mut cfg = PwmConfig::default();
    cfg.top = 10_000;
    let pwm = Pwm::new_output_a(p.PWM_SLICE7, p.PIN_14, cfg); // GP14: an external LED
    spawner.spawn(unwrap!(breathe(pwm)));
    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
