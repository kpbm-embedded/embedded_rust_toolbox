//! RTIC v2 on QEMU's lm3s6965evb (Cortex-M3), no board: the same concurrency model as the BlackPill
//! examples, checked by running it.
//!   GPIOA (hardware task, prio 3): a "button" interrupt, pended from software here; counts presses
//!     (shared, priority-ceiling lock) and sends them over an rtic-sync channel
//!   control (async, prio 2): receives the presses, picks a blink period (shared `period_ms`)
//!   blink (async, prio 1): "toggles the LED" every period with the SysTick monotonic
//!   stimulus (async, prio 1): pends GPIOA three times, then ends the run with a semihosting exit
#![no_std]
#![no_main]

use panic_semihosting as _;
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1_000);

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use super::*;
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;
    use rtic_sync::channel::{Receiver, Sender};
    use rtic_sync::make_channel;

    const CAP: usize = 4;

    #[shared]
    struct Shared {
        presses: u32,
        period_ms: u32,
        toggles: u32,
    }

    #[local]
    struct Local {
        tx: Sender<'static, u32, CAP>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // QEMU's lm3s6965evb runs the core (and SysTick) at 12 MHz after reset
        Mono::start(cx.core.SYST, 12_000_000);
        let (tx, rx) = make_channel!(u32, CAP);
        control::spawn(rx).ok();
        blink::spawn().ok();
        stimulus::spawn().ok();
        hprintln!("rtic init done");
        (Shared { presses: 0, period_ms: 20, toggles: 0 }, Local { tx })
    }

    #[task(binds = GPIOA, local = [tx], shared = [presses], priority = 3)]
    fn on_button(mut cx: on_button::Context) {
        let n = cx.shared.presses.lock(|p| {
            *p += 1;
            *p
        });
        if cx.local.tx.try_send(n).is_err() {
            hprintln!("channel full, press {} dropped", n);
        }
    }

    #[task(shared = [period_ms], priority = 2)]
    async fn control(mut cx: control::Context, mut rx: Receiver<'static, u32, CAP>) {
        while let Ok(n) = rx.recv().await {
            let p = [20u32, 5, 40][(n % 3) as usize];
            cx.shared.period_ms.lock(|v| *v = p);
            hprintln!("press {}: period {} ms", n, p);
        }
    }

    #[task(shared = [period_ms, toggles], priority = 1)]
    async fn blink(mut cx: blink::Context) {
        let mut next = Mono::now();
        loop {
            cx.shared.toggles.lock(|t| *t += 1);
            next += cx.shared.period_ms.lock(|p| *p).millis();
            Mono::delay_until(next).await;
        }
    }

    #[task(shared = [presses, toggles], priority = 1)]
    async fn stimulus(mut cx: stimulus::Context) {
        let t0 = Mono::now();
        for _ in 0..3 {
            Mono::delay(50.millis()).await;
            rtic::pend(Interrupt::GPIOA);
        }
        Mono::delay(50.millis()).await;
        let ms = (Mono::now() - t0).as_millis();
        let presses = cx.shared.presses.lock(|p| *p);
        let toggles = cx.shared.toggles.lock(|t| *t);
        hprintln!("presses {} toggles {} elapsed {} ms", presses, toggles, ms);
        let ok = presses == 3 && toggles > 5 && ms >= 200;
        hprintln!("result: {}", if ok { "PASS" } else { "FAIL" });
        debug::exit(if ok { debug::EXIT_SUCCESS } else { debug::EXIT_FAILURE });
    }
}
