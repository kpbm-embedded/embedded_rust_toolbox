//! Embassy on the Raspberry Pi Pico: interrupt-driven buffered UART0 (TX GP0, RX GP1, 115200 8N1).
//! A reader task collects lines and hands them to the writer over an embassy-sync Channel; the
//! writer echoes them back upper-cased. Wire GP0-GP1 together or use the Debug Probe's UART.
#![no_std]
#![no_main]

use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

const LINE: usize = 64;
type Line = ([u8; LINE], usize);
static LINES: Channel<CriticalSectionRawMutex, Line, 4> = Channel::new();

#[embassy_executor::task]
async fn reader(mut rx: BufferedUartRx) {
    let mut line = [0u8; LINE];
    let mut len = 0;
    loop {
        let mut b = [0u8; 1];
        if rx.read_exact(&mut b).await.is_err() {
            warn!("uart rx error");
            continue;
        }
        match b[0] {
            b'\r' | b'\n' if len > 0 => {
                LINES.send((line, len)).await;
                len = 0;
            }
            c if len < LINE => {
                line[len] = c;
                len += 1;
            }
            _ => {}
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    static TX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let mut cfg = Config::default();
    cfg.baudrate = 115_200;
    let uart = BufferedUart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        Irqs,
        &mut TX_BUF.init([0; 64])[..],
        &mut RX_BUF.init([0; 64])[..],
        cfg,
    );
    let (mut tx, rx) = uart.split();
    spawner.spawn(unwrap!(reader(rx)));
    unwrap!(tx.write_all(b"embassy_pico_uart: type a line\r\n").await);
    loop {
        let (mut buf, n) = LINES.receive().await;
        buf[..n].make_ascii_uppercase();
        info!("echo {} bytes", n);
        unwrap!(tx.write_all(&buf[..n]).await);
        unwrap!(tx.write_all(b"\r\n").await);
    }
}
