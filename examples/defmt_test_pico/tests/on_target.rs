//! On-target unit tests with defmt-test: each #[test] runs on the Pico itself and reports through
//! defmt/RTT; `probe-rs run --chip RP2040 <this test ELF>` (printed by `rust-box plan`) shows
//! "(1/4) running ..." lines and exits with the test status.
#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use rp2040_hal as _; // critical-section impl + interrupt vectors

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[defmt_test::tests]
mod tests {
    use defmt::assert_eq;
    use defmt_test_pico::{adc_to_mv, duty_u8, temp_milli_c};

    #[init]
    fn init() -> u32 {
        defmt::info!("defmt_test_pico: on-target tests");
        0
    }

    #[test]
    fn adc_full_scale(_n: &mut u32) {
        assert_eq!(adc_to_mv(0), 0);
        assert_eq!(adc_to_mv(4095), 3300);
    }

    #[test]
    fn temp_at_27c(_n: &mut u32) {
        // 0.706 V -> 876 counts -> about 27 C
        let t = temp_milli_c(876);
        defmt::assert!((26_000..28_000).contains(&t), "t = {}", t);
    }

    #[test]
    fn duty_saturates(_n: &mut u32) {
        assert_eq!(duty_u8(0), 0);
        assert_eq!(duty_u8(50), 128);
        assert_eq!(duty_u8(250), 255);
    }

    #[test]
    fn state_is_shared(n: &mut u32) {
        *n += 1;
        assert_eq!(*n, 1);
    }
}
