//! Embassy on QEMU's mps2-an385 (Cortex-M3), no board and no HAL: an own embassy-time driver on SysTick
//! (1 kHz tick, generic timer queue from embassy-time-queue-utils), the thread executor, two tasks with a
//! Channel, a Ticker, Timer::after and a select() timeout. Exits through semihosting with the result.
#![no_std]
#![no_main]

use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Waker;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;
use cortex_m_semihosting::{debug, hprintln};
use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Ticker, Timer};
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;
use panic_semihosting as _;

/// mps2-an385 runs the core at 25 MHz: SysTick reload for a 1 kHz tick (embassy-time tick-hz-1_000).
const CORE_HZ: u32 = 25_000_000;

// ---------- the time driver: SysTick counts ms, the queue wakes expired timers ----------
static TICKS: AtomicU32 = AtomicU32::new(0);

struct SysTickDriver {
    queue: Mutex<RefCell<Queue>>,
}

impl Driver for SysTickDriver {
    fn now(&self) -> u64 {
        u64::from(TICKS.load(Ordering::Relaxed))
    }
    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut q = self.queue.borrow_ref_mut(cs);
            if q.schedule_wake(at, waker) {
                // the queue is checked on every tick, so an already-due timer is handled at the next one
                q.next_expiration(self.now());
            }
        });
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: SysTickDriver = SysTickDriver {
    queue: Mutex::new(RefCell::new(Queue::new())),
});

#[exception]
fn SysTick() {
    let now = u64::from(TICKS.fetch_add(1, Ordering::Relaxed) + 1);
    critical_section::with(|cs| {
        DRIVER.queue.borrow_ref_mut(cs).next_expiration(now);
    });
}

// ---------- the application ----------
static READINGS: Channel<CriticalSectionRawMutex, u32, 4> = Channel::new();

#[embassy_executor::task]
async fn sampler() {
    let mut ticker = Ticker::every(Duration::from_millis(10));
    for n in 1..=5u32 {
        ticker.next().await;
        READINGS.send(n * 100).await;
    }
    // then go quiet: the consumer's select() must time out
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(CORE_HZ / 1_000 - 1);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    hprintln!("embassy on qemu mps2-an385, SysTick time driver");
    let t0 = Instant::now();
    Timer::after_millis(20).await;
    let slept = t0.elapsed().as_millis();
    hprintln!("Timer::after_millis(20) took {} ms", slept);

    spawner.spawn(sampler().unwrap());
    let (mut sum, mut got) = (0u32, 0u32);
    let timed_out = loop {
        match select(READINGS.receive(), Timer::after_millis(50)).await {
            Either::First(v) => {
                got += 1;
                sum += v;
                hprintln!("reading {} at {} ms", v, t0.elapsed().as_millis());
            }
            Either::Second(()) => break true,
        }
    };
    hprintln!("received {} readings, sum {}, timeout {}", got, sum, timed_out);
    let ok = slept >= 20 && got == 5 && sum == 1500 && timed_out;
    hprintln!("result: {}", if ok { "PASS" } else { "FAIL" });
    debug::exit(if ok { debug::EXIT_SUCCESS } else { debug::EXIT_FAILURE });
}
