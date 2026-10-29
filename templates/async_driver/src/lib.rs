//! An async driver for SPI NOR flash (Winbond W25Q16/W25Q128 as on the Pico and WeAct boards), written
//! against embedded-hal-async's SpiDevice + DelayNs: JEDEC ID, status polling, page program with the
//! 256-byte page boundary split, read and 4 KB sector erase. Works under Embassy or RTIC (async tasks).
//! Tests: embedded-hal-mock's async SPI mock, driven by embassy_futures::block_on on the PC.
#![no_std]

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::{Operation, SpiDevice};

const CMD_JEDEC_ID: u8 = 0x9F;
const CMD_READ_STATUS1: u8 = 0x05;
const CMD_WRITE_ENABLE: u8 = 0x06;
const CMD_PAGE_PROGRAM: u8 = 0x02;
const CMD_READ_DATA: u8 = 0x03;
const CMD_SECTOR_ERASE: u8 = 0x20;
const STATUS_BUSY: u8 = 0x01;
pub const PAGE: u32 = 256;

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    Spi(E),
    Timeout,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct JedecId {
    pub manufacturer: u8,
    pub memory_type: u8,
    pub capacity_log2: u8,
}

pub struct SpiFlash<S> {
    spi: S,
}

fn addr(a: u32) -> [u8; 3] {
    let b = a.to_be_bytes();
    [b[1], b[2], b[3]]
}

impl<S: SpiDevice> SpiFlash<S> {
    pub fn new(spi: S) -> Self {
        Self { spi }
    }

    pub async fn jedec_id(&mut self) -> Result<JedecId, Error<S::Error>> {
        let mut id = [0u8; 3];
        self.spi.transaction(&mut [Operation::Write(&[CMD_JEDEC_ID]), Operation::Read(&mut id)]).await.map_err(Error::Spi)?;
        Ok(JedecId { manufacturer: id[0], memory_type: id[1], capacity_log2: id[2] })
    }

    async fn status(&mut self) -> Result<u8, Error<S::Error>> {
        let mut s = [0u8; 1];
        self.spi.transaction(&mut [Operation::Write(&[CMD_READ_STATUS1]), Operation::Read(&mut s)]).await.map_err(Error::Spi)?;
        Ok(s[0])
    }

    async fn wait_idle(&mut self, delay: &mut impl DelayNs, tries: u32) -> Result<(), Error<S::Error>> {
        for _ in 0..tries {
            if self.status().await? & STATUS_BUSY == 0 {
                return Ok(());
            }
            delay.delay_us(100).await;
        }
        Err(Error::Timeout)
    }

    async fn write_enable(&mut self) -> Result<(), Error<S::Error>> {
        self.spi.write(&[CMD_WRITE_ENABLE]).await.map_err(Error::Spi)
    }

    pub async fn read(&mut self, at: u32, buf: &mut [u8]) -> Result<(), Error<S::Error>> {
        let a = addr(at);
        let cmd = [CMD_READ_DATA, a[0], a[1], a[2]];
        self.spi.transaction(&mut [Operation::Write(&cmd), Operation::Read(buf)]).await.map_err(Error::Spi)
    }

    /// Programs `data` at `at`, split so that no chunk crosses a 256-byte page boundary.
    pub async fn program(&mut self, mut at: u32, mut data: &[u8], delay: &mut impl DelayNs) -> Result<(), Error<S::Error>> {
        while !data.is_empty() {
            let room = (PAGE - at % PAGE) as usize;
            let (chunk, rest) = data.split_at(room.min(data.len()));
            self.write_enable().await?;
            let a = addr(at);
            let cmd = [CMD_PAGE_PROGRAM, a[0], a[1], a[2]];
            self.spi.transaction(&mut [Operation::Write(&cmd), Operation::Write(chunk)]).await.map_err(Error::Spi)?;
            self.wait_idle(delay, 50).await?;
            at += chunk.len() as u32;
            data = rest;
        }
        Ok(())
    }

    pub async fn erase_sector(&mut self, at: u32, delay: &mut impl DelayNs) -> Result<(), Error<S::Error>> {
        self.write_enable().await?;
        let a = addr(at & !0xFFF);
        self.spi.write(&[CMD_SECTOR_ERASE, a[0], a[1], a[2]]).await.map_err(Error::Spi)?;
        self.wait_idle(delay, 4000).await
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embassy_futures::block_on;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::spi::{Mock, Transaction as T};
    use std::vec;
    use std::vec::Vec;

    fn status(v: u8) -> Vec<T<u8>> {
        vec![T::transaction_start(), T::write_vec(vec![CMD_READ_STATUS1]), T::read_vec(vec![v]), T::transaction_end()]
    }
    fn wren() -> Vec<T<u8>> {
        vec![T::transaction_start(), T::write_vec(vec![CMD_WRITE_ENABLE]), T::transaction_end()]
    }

    #[test]
    fn reads_jedec_id() {
        let mut spi = Mock::new(&[T::transaction_start(), T::write_vec(vec![0x9F]), T::read_vec(vec![0xEF, 0x40, 0x18]), T::transaction_end()]);
        let mut f = SpiFlash::new(spi.clone());
        let id = block_on(f.jedec_id()).unwrap();
        assert_eq!(id, JedecId { manufacturer: 0xEF, memory_type: 0x40, capacity_log2: 0x18 }); // W25Q128
        spi.done();
    }

    #[test]
    fn read_sends_24bit_address() {
        let mut spi = Mock::new(&[
            T::transaction_start(),
            T::write_vec(vec![0x03, 0x01, 0x23, 0x45]),
            T::read_vec(vec![1, 2, 3, 4]),
            T::transaction_end(),
        ]);
        let mut buf = [0u8; 4];
        block_on(SpiFlash::new(spi.clone()).read(0x012345, &mut buf)).unwrap();
        assert_eq!(buf, [1, 2, 3, 4]);
        spi.done();
    }

    #[test]
    fn program_splits_at_page_boundary() {
        let mut ex = Vec::new();
        ex.extend(wren());
        ex.extend([T::transaction_start(), T::write_vec(vec![0x02, 0x00, 0x00, 0xFE]), T::write_vec(vec![0xA0, 0xA1]), T::transaction_end()]);
        ex.extend(status(0x01)); // busy once
        ex.extend(status(0x00));
        ex.extend(wren());
        ex.extend([T::transaction_start(), T::write_vec(vec![0x02, 0x00, 0x01, 0x00]), T::write_vec(vec![0xA2]), T::transaction_end()]);
        ex.extend(status(0x00));
        let mut spi = Mock::new(&ex);
        block_on(SpiFlash::new(spi.clone()).program(0xFE, &[0xA0, 0xA1, 0xA2], &mut NoopDelay::new())).unwrap();
        spi.done();
    }

    #[test]
    fn erase_aligns_and_times_out() {
        let mut ex = Vec::new();
        ex.extend(wren());
        ex.extend([T::transaction_start(), T::write_vec(vec![0x20, 0x00, 0x10, 0x00]), T::transaction_end()]);
        for _ in 0..4000 {
            ex.extend(status(0x01));
        }
        let mut spi = Mock::new(&ex);
        let r = block_on(SpiFlash::new(spi.clone()).erase_sector(0x1234, &mut NoopDelay::new()));
        assert_eq!(r, Err(Error::Timeout));
        spi.done();
    }
}
