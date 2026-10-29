//! RTIC v2 on the BlackPill (STM32F411CE, stm32f4xx-hal): 96 MHz from the 25 MHz crystal, a SysTick
//! monotonic (rtic-monotonics) and one async software task that blinks PC13 with drift-free
//! delay_until().
#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1_000);

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART2])]
mod app {
    use super::*;
    use stm32f4xx_hal::gpio::{Output, PC13};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        use stm32f4xx_hal::{prelude::*, rcc::Config};
        let mut rcc = cx.device.RCC.freeze(Config::hse(25.MHz()).sysclk(96.MHz()));
        Mono::start(cx.core.SYST, 96_000_000);
        let gpioc = cx.device.GPIOC.split(&mut rcc);
        let led = gpioc.pc13.into_push_pull_output();
        defmt::info!("rtic_blackpill_blinky: 96 MHz, SysTick monotonic");
        blink::spawn().ok();
        (Shared {}, Local { led })
    }

    #[task(local = [led, n: u32 = 0], priority = 1)]
    async fn blink(cx: blink::Context) {
        let mut next = Mono::now();
        loop {
            cx.local.led.toggle();
            *cx.local.n += 1;
            defmt::debug!("blink {} at {} ms", *cx.local.n, Mono::now().duration_since_epoch().as_millis());
            next += 500.millis();
            Mono::delay_until(next).await;
        }
    }
}
