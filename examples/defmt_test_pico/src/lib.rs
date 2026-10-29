//! Code under test: fixed-point helpers for the RP2040's internal temperature sensor
//! (datasheet 4.9.5: T = 27 - (V - 0.706) / 0.001721, 12-bit ADC at 3.3 V).
#![no_std]

/// ADC raw (0..4095) -> millivolts at a 3.3 V reference.
pub fn adc_to_mv(raw: u16) -> u32 {
    (raw as u32 * 3300 + 2047) / 4095
}

/// ADC raw -> temperature in milli-degrees Celsius (integer math only, Cortex-M0+ has no FPU).
pub fn temp_milli_c(raw: u16) -> i32 {
    let uv = adc_to_mv(raw) as i32 * 1000; // microvolts
    27_000 - (uv - 706_000) * 1000 / 1721
}

/// Saturating 8-bit duty cycle from a percentage.
pub fn duty_u8(percent: u8) -> u8 {
    ((percent.min(100) as u16 * 255 + 50) / 100) as u8
}
