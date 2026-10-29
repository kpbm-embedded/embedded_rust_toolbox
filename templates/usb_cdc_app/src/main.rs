//! USB device on the Pico with embassy-usb: a CDC-ACM serial port (shows up as /dev/ttyACM*) that
//! echoes every line back upper-cased, with the device and the class running as two futures joined
//! in main. Needs only the Pico's USB port (no probe).
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);

    // 0xc0de/0xcafe is a test VID/PID: use your own for anything you ship
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Box embedded-rust");
    config.product = Some("Pico CDC echo");
    config.serial_number = Some("BOX-0001");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = State::new();
    let mut builder = Builder::new(driver, config, &mut config_descriptor, &mut bos_descriptor, &mut [], &mut control_buf);
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let mut usb = builder.build();

    let echo = async {
        loop {
            class.wait_connection().await;
            info!("host opened the port");
            let _ = echo_upper(&mut class).await;
            info!("host closed the port");
        }
    };
    join(usb.run(), echo).await;
}

struct Disconnected;
impl From<EndpointError> for Disconnected {
    fn from(e: EndpointError) -> Self {
        match e {
            EndpointError::BufferOverflow => defmt::panic!("buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

async fn echo_upper<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        buf[..n].make_ascii_uppercase();
        class.write_packet(&buf[..n]).await?;
    }
}
