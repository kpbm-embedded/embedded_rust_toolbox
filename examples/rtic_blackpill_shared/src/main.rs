//! RTIC v2 on the BlackPill: shared resources with priority-ceiling locks, a hardware task on
//! the KEY button (PA0, EXTI0 falling edge), two async software tasks at different priorities and
//! an rtic-sync channel from the interrupt to a task.
//!   on_key (hw, prio 3): count presses (shared `presses`), send the count to the channel
//!   control (sw, prio 2): receives counts, picks a new blink period (shared `period_ms`)
//!   blink (sw, prio 1): toggles PC13 every `period_ms`
#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1_000);

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2, SPI1])]
mod app {
    use super::*;
    use rtic_sync::channel::{Receiver, Sender};
    use rtic_sync::make_channel;
    use stm32f4xx_hal::gpio::{Edge, ExtiPin, Input, Output, PA0, PC13};

    const CAP: usize = 4;

    #[shared]
    struct Shared {
        presses: u32,
        period_ms: u32,
    }

    #[local]
    struct Local {
        key: PA0<Input>,
        led: PC13<Output>,
        tx: Sender<'static, u32, CAP>,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        use stm32f4xx_hal::{prelude::*, rcc::Config};
        let mut rcc = cx.device.RCC.freeze(Config::hse(25.MHz()).sysclk(96.MHz()));
        Mono::start(cx.core.SYST, 96_000_000);
        let mut syscfg = cx.device.SYSCFG.constrain(&mut rcc);
        let gpioa = cx.device.GPIOA.split(&mut rcc);
        let gpioc = cx.device.GPIOC.split(&mut rcc);
        let mut key = gpioa.pa0.into_pull_up_input();
        key.make_interrupt_source(&mut syscfg);
        key.enable_interrupt(&mut cx.device.EXTI);
        key.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);
        let led = gpioc.pc13.into_push_pull_output();
        let (tx, rx) = make_channel!(u32, CAP);
        control::spawn(rx).ok();
        blink::spawn().ok();
        defmt::info!("rtic_blackpill_shared: press KEY (PA0)");
        (Shared { presses: 0, period_ms: 500 }, Local { key, led, tx })
    }

    #[task(binds = EXTI0, local = [key, tx], shared = [presses], priority = 3)]
    fn on_key(mut cx: on_key::Context) {
        cx.local.key.clear_interrupt_pending_bit();
        let n = cx.shared.presses.lock(|p| {
            *p += 1;
            *p
        });
        if cx.local.tx.try_send(n).is_err() {
            defmt::warn!("channel full, press {} dropped", n);
        }
    }

    #[task(shared = [period_ms], priority = 2)]
    async fn control(mut cx: control::Context, mut rx: Receiver<'static, u32, CAP>) {
        while let Ok(n) = rx.recv().await {
            let p = [500u32, 100, 1000][(n % 3) as usize];
            cx.shared.period_ms.lock(|v| *v = p);
            defmt::info!("press {}: period {} ms", n, p);
            Mono::delay(30.millis()).await; // debounce
        }
    }

    #[task(local = [led], shared = [period_ms], priority = 1)]
    async fn blink(mut cx: blink::Context) {
        loop {
            cx.local.led.toggle();
            let p = cx.shared.period_ms.lock(|v| *v);
            Mono::delay(p.millis()).await;
        }
    }
}
