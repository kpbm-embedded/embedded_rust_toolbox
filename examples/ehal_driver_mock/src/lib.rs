//! A platform-independent driver for the TI TMP117 I2C temperature sensor, written against the
//! embedded-hal 1.0 traits only (I2c + DelayNs + InputPin for the ALERT line). Any HAL works:
//! embassy-stm32, rp2040-hal, nrf, esp-hal. The tests run on the PC with embedded-hal-mock, which
//! checks every I2C transaction byte by byte.
#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_hal::i2c::I2c;

pub const DEFAULT_ADDR: u8 = 0x48;
const REG_TEMP: u8 = 0x00;
const REG_CONFIG: u8 = 0x01;
const REG_HIGH: u8 = 0x02;
const REG_DEVICE_ID: u8 = 0x0F;
const DEVICE_ID: u16 = 0x0117;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    I2c(E),
    WrongDevice(u16),
    NotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    Continuous = 0b00,
    Shutdown = 0b01,
    OneShot = 0b11,
}

pub struct Tmp117<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C: I2c> Tmp117<I2C> {
    /// Checks the device ID register before handing out the driver.
    pub fn new(i2c: I2C, addr: u8) -> Result<Self, Error<I2C::Error>> {
        let mut dev = Self { i2c, addr };
        let id = dev.read_reg(REG_DEVICE_ID)? & 0x0FFF;
        if id != DEVICE_ID {
            return Err(Error::WrongDevice(id));
        }
        Ok(dev)
    }

    fn read_reg(&mut self, reg: u8) -> Result<u16, Error<I2C::Error>> {
        let mut b = [0u8; 2];
        self.i2c.write_read(self.addr, &[reg], &mut b).map_err(Error::I2c)?;
        Ok(u16::from_be_bytes(b))
    }

    fn write_reg(&mut self, reg: u8, v: u16) -> Result<(), Error<I2C::Error>> {
        let [hi, lo] = v.to_be_bytes();
        self.i2c.write(self.addr, &[reg, hi, lo]).map_err(Error::I2c)
    }

    /// Temperature in milli-degrees Celsius (1 LSB = 7.8125 m°C).
    pub fn temperature_mc(&mut self) -> Result<i32, Error<I2C::Error>> {
        let raw = self.read_reg(REG_TEMP)? as i16;
        Ok(i32::from(raw) * 78_125 / 10_000)
    }

    /// Sets the conversion mode (bits 11:10 of CONFIG), keeping the other bits.
    pub fn set_mode(&mut self, mode: Mode) -> Result<(), Error<I2C::Error>> {
        let cfg = self.read_reg(REG_CONFIG)?;
        self.write_reg(REG_CONFIG, (cfg & !(0b11 << 10)) | ((mode as u16) << 10))
    }

    /// Starts a one-shot conversion and polls DATA_READY (CONFIG bit 13) with a delay between reads.
    pub fn one_shot_mc(&mut self, delay: &mut impl DelayNs, tries: u8) -> Result<i32, Error<I2C::Error>> {
        self.set_mode(Mode::OneShot)?;
        for _ in 0..tries {
            delay.delay_ms(16);
            if self.read_reg(REG_CONFIG)? & (1 << 13) != 0 {
                return self.temperature_mc();
            }
        }
        Err(Error::NotReady)
    }

    /// High alert limit in m°C (rounded down to the register resolution).
    pub fn set_high_limit_mc(&mut self, mc: i32) -> Result<(), Error<I2C::Error>> {
        let raw = (mc * 10_000 / 78_125) as i16;
        self.write_reg(REG_HIGH, raw as u16)
    }

    pub fn release(self) -> I2C {
        self.i2c
    }
}

/// The ALERT pin is open-drain, active low.
pub fn alert_active<P: InputPin>(pin: &mut P) -> Result<bool, P::Error> {
    pin.is_low()
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction as PinT};
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as T};
    use std::vec;

    const A: u8 = DEFAULT_ADDR;
    fn id_ok() -> T {
        T::write_read(A, vec![REG_DEVICE_ID], vec![0x01, 0x17])
    }

    #[test]
    fn rejects_wrong_device() {
        let mut i2c = I2cMock::new(&[T::write_read(A, vec![REG_DEVICE_ID], vec![0x01, 0x19])]);
        assert_eq!(Tmp117::new(i2c.clone(), A).err(), Some(Error::WrongDevice(0x119)));
        i2c.done();
    }

    #[test]
    fn reads_positive_and_negative_temperature() {
        let mut i2c = I2cMock::new(&[
            id_ok(),
            T::write_read(A, vec![REG_TEMP], vec![0x0C, 0x80]), // 3200 * 7.8125 = 25000 m°C
            T::write_read(A, vec![REG_TEMP], vec![0xFF, 0x80]), // -128 * 7.8125 = -1000 m°C
        ]);
        let mut t = Tmp117::new(i2c.clone(), A).unwrap();
        assert_eq!(t.temperature_mc(), Ok(25_000));
        assert_eq!(t.temperature_mc(), Ok(-1_000));
        i2c.done();
    }

    #[test]
    fn set_mode_keeps_other_bits() {
        let mut i2c = I2cMock::new(&[
            id_ok(),
            T::write_read(A, vec![REG_CONFIG], vec![0x02, 0x20]),
            T::write(A, vec![REG_CONFIG, 0x06, 0x20]),
        ]);
        Tmp117::new(i2c.clone(), A).unwrap().set_mode(Mode::Shutdown).unwrap();
        i2c.done();
    }

    #[test]
    fn one_shot_polls_until_ready() {
        let mut i2c = I2cMock::new(&[
            id_ok(),
            T::write_read(A, vec![REG_CONFIG], vec![0x02, 0x20]),
            T::write(A, vec![REG_CONFIG, 0x0E, 0x20]),
            T::write_read(A, vec![REG_CONFIG], vec![0x0E, 0x20]), // not ready
            T::write_read(A, vec![REG_CONFIG], vec![0x2E, 0x20]), // DATA_READY
            T::write_read(A, vec![REG_TEMP], vec![0x0A, 0x00]),   // 2560 -> 20000 m°C
        ]);
        let mut t = Tmp117::new(i2c.clone(), A).unwrap();
        assert_eq!(t.one_shot_mc(&mut NoopDelay::new(), 5), Ok(20_000));
        i2c.done();
    }

    #[test]
    fn one_shot_times_out() {
        let mut i2c = I2cMock::new(&[
            id_ok(),
            T::write_read(A, vec![REG_CONFIG], vec![0x00, 0x00]),
            T::write(A, vec![REG_CONFIG, 0x0C, 0x00]),
            T::write_read(A, vec![REG_CONFIG], vec![0x0C, 0x00]),
            T::write_read(A, vec![REG_CONFIG], vec![0x0C, 0x00]),
        ]);
        let mut t = Tmp117::new(i2c.clone(), A).unwrap();
        assert_eq!(t.one_shot_mc(&mut NoopDelay::new(), 2), Err(Error::NotReady));
        i2c.done();
    }

    #[test]
    fn high_limit_encoding() {
        let mut i2c = I2cMock::new(&[id_ok(), T::write(A, vec![REG_HIGH, 0x14, 0x00])]); // 40 °C = 5120
        Tmp117::new(i2c.clone(), A).unwrap().set_high_limit_mc(40_000).unwrap();
        i2c.done();
    }

    #[test]
    fn alert_pin_active_low() {
        let mut pin = PinMock::new(&[PinT::get(State::High), PinT::get(State::Low)]);
        assert_eq!(alert_active(&mut pin), Ok(false));
        assert_eq!(alert_active(&mut pin), Ok(true));
        pin.done();
    }
}
