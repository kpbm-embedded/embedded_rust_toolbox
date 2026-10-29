//! Embassy on the BlackPill (STM32F411CE) at 96 MHz from the 25 MHz crystal: three tasks that
//! talk through embassy-sync primitives.
//!   sampler: a Ticker every 100 ms produces a fake sensor value -> Channel
//!   reporter: averages 10 samples and writes a line to USART1 (TX PA9, RX PA10, 115200)
//!   blinker: LED PC13, blink pattern chosen through a Signal (fast when the average is high)
//! select() races the Channel against a timeout, the same pattern as for real sensors.
#![no_std]
#![no_main]

use core::fmt::Write as _;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{BufferedUart, Config as UartConfig};
use embassy_stm32::{Config, bind_interrupts, peripherals, usart};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker, Timer};
use embedded_io_async::Write;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
});

static SAMPLES: Channel<CriticalSectionRawMutex, u16, 8> = Channel::new();
static FAST: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::task]
async fn sampler() {
    let mut t = Ticker::every(Duration::from_millis(100));
    let mut x: u16 = 0x1234;
    loop {
        t.next().await;
        x ^= x << 7; // xorshift as a stand-in for an ADC reading
        x ^= x >> 9;
        x ^= x << 8;
        SAMPLES.send(x % 1000).await;
    }
}

#[embassy_executor::task]
async fn blinker(mut led: Output<'static>) {
    let mut fast = false;
    loop {
        if let Some(f) = FAST.try_take() {
            fast = f;
        }
        led.toggle();
        Timer::after_millis(if fast { 100 } else { 400 }).await;
    }
}

struct Line<const N: usize>([u8; N], usize);
impl<const N: usize> core::fmt::Write for Line<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        let n = b.len().min(N - self.1);
        self.0[self.1..self.1 + n].copy_from_slice(&b[..n]);
        self.1 += n;
        Ok(())
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse { freq: Hertz(25_000_000), mode: HseMode::Oscillator });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL192,
            divp: Some(PllPDiv::DIV2), // 96 MHz SYSCLK
            divq: Some(PllQDiv::DIV4), // 48 MHz USB
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    static TX: StaticCell<[u8; 128]> = StaticCell::new();
    static RX: StaticCell<[u8; 16]> = StaticCell::new();
    let mut ucfg = UartConfig::default();
    ucfg.baudrate = 115_200;
    let mut uart = unwrap!(BufferedUart::new(p.USART1, p.PA10, p.PA9, TX.init([0; 128]), RX.init([0; 16]), Irqs, ucfg));

    let led = Output::new(p.PC13, Level::High, Speed::Low);
    spawner.spawn(unwrap!(sampler()));
    spawner.spawn(unwrap!(blinker(led)));
    info!("embassy_blackpill_tasks: 96 MHz, USART1 115200");

    let (mut sum, mut n) = (0u32, 0u32);
    loop {
        match select(SAMPLES.receive(), Timer::after_secs(1)).await {
            Either::First(v) => {
                sum += v as u32;
                n += 1;
            }
            Either::Second(()) => info!("no sample for 1 s"),
        }
        if n == 10 {
            let avg = sum / n;
            FAST.signal(avg > 500);
            let mut line = Line([0u8; 48], 0);
            let _ = write!(line, "avg {} of {} samples\r\n", avg, n);
            unwrap!(uart.write_all(&line.0[..line.1]).await);
            info!("avg {}", avg);
            (sum, n) = (0, 0);
        }
    }
}
