//! no_std code to test on the target CPU: a fixed-point (Q16.16) PI controller with anti-windup and a
//! fixed-size moving-median filter. Tested in tests/on_qemu.rs with defmt-test on a QEMU Cortex-M3.
#![no_std]

/// Q16.16 fixed point.
pub type Q16 = i32;
pub const ONE: Q16 = 1 << 16;

pub fn q(v: i32) -> Q16 {
    v << 16
}

/// PI controller; the integrator is clamped to +-`limit` (anti-windup).
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct Pi {
    pub kp: Q16,
    pub ki: Q16,
    pub limit: Q16,
    integ: Q16,
}

impl Pi {
    pub const fn new(kp: Q16, ki: Q16, limit: Q16) -> Self {
        Self { kp, ki, limit, integ: 0 }
    }
    pub fn step(&mut self, setpoint: Q16, measured: Q16) -> Q16 {
        let err = setpoint.saturating_sub(measured);
        let i = (i64::from(self.ki) * i64::from(err)) >> 16;
        self.integ = (i64::from(self.integ) + i).clamp(-i64::from(self.limit), i64::from(self.limit)) as Q16;
        let p = (i64::from(self.kp) * i64::from(err)) >> 16;
        (p + i64::from(self.integ)).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as Q16
    }
    pub fn integrator(&self) -> Q16 {
        self.integ
    }
}

/// Median of the last 5 samples (spike filter).
#[derive(Default)]
pub struct Median5 {
    buf: [u16; 5],
    n: usize,
    head: usize,
}

impl Median5 {
    pub fn push(&mut self, v: u16) -> u16 {
        self.buf[self.head] = v;
        self.head = (self.head + 1) % 5;
        self.n = (self.n + 1).min(5);
        let mut s = [0u16; 5];
        s[..self.n].copy_from_slice(&self.buf[..self.n]);
        s[..self.n].sort_unstable();
        s[self.n / 2]
    }
}
