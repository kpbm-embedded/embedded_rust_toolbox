//! defmt-test tests that run on the target CPU (here QEMU's Cortex-M3; the same file runs on a board
//! with probe-rs). Each #[test] gets the state from #[init]; results come back as defmt frames.
#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt as _;
use defmt_semihosting as _;
use panic_semihosting as _;
use qemu_defmt_test::{q, Median5, Pi, ONE};

#[defmt_test::tests]
mod tests {
    use super::*;
    use defmt::{assert, assert_eq};

    #[init]
    fn init() -> Pi {
        Pi::new(ONE / 2, ONE / 4, q(10))
    }

    #[test]
    fn proportional_only_at_start(pi: &mut Pi) {
        let out = pi.step(q(4), q(0));
        // p = 0.5 * 4 = 2, i = 0.25 * 4 = 1
        assert_eq!(out, q(3));
    }

    #[test]
    fn integrator_is_clamped(pi: &mut Pi) {
        for _ in 0..100 {
            pi.step(q(100), q(0));
        }
        assert_eq!(pi.integrator(), q(10));
    }

    #[test]
    fn negative_error_winds_down(pi: &mut Pi) {
        let out = pi.step(q(0), q(8));
        assert!(out < q(10));
    }

    #[test]
    fn median_drops_spike() {
        let mut m = Median5::default();
        let mut last = 0;
        for v in [100, 101, 4000, 99, 102] {
            last = m.push(v);
        }
        assert_eq!(last, 101);
    }

    #[test]
    fn hardware_float_is_absent() {
        // thumbv7m has no FPU: f32 math is soft-float and must still be exact for this value
        let x = core::hint::black_box(1.5f32) * 2.0;
        assert_eq!(x, 3.0);
    }
}
