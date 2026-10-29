//! Bare-metal RISC-V (riscv-rt) on QEMU's `virt` board: a polled 16550 UART driver written against the
//! memory map, machine CSRs (misa, mhartid), the cycle counter, and a clean exit through the SiFive
//! test device (0x0010_0000), which makes QEMU return the result as its exit code.
#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use riscv::register::{mhartid, misa};
use riscv_rt::entry;

const UART0: usize = 0x1000_0000; // ns16550a
const TEST_DEV: usize = 0x0010_0000; // sifive,test0: 0x5555 = pass, (code << 16) | 0x3333 = fail

struct Uart;
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            // SAFETY: fixed MMIO addresses of the virt machine
            unsafe {
                while (((UART0 + 5) as *const u8).read_volatile() & 0x20) == 0 {} // LSR.THRE
                (UART0 as *mut u8).write_volatile(b);
            }
        }
        Ok(())
    }
}

fn exit(code: u32) -> ! {
    let v = if code == 0 { 0x5555 } else { (code << 16) | 0x3333 };
    // SAFETY: the test device of the virt machine
    unsafe { (TEST_DEV as *mut u32).write_volatile(v) };
    loop {
        riscv::asm::wfi();
    }
}

fn isqrt(n: u32) -> u32 {
    let (mut lo, mut hi) = (0u32, n.min(65_535) + 1);
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if mid * mid <= n { lo = mid } else { hi = mid }
    }
    lo
}

#[entry]
fn main() -> ! {
    let mut u = Uart;
    let isa = misa::read();
    let mut ext = [b'-'; 26];
    let mut n = 0;
    for c in b'A'..=b'Z' {
        if isa.has_extension(c as char) {
            ext[n] = c;
            n += 1;
        }
    }
    let ext = core::str::from_utf8(&ext[..n]).unwrap_or("?");
    writeln!(u, "hello from Rust on qemu riscv32 virt").ok();
    writeln!(u, "hart {} misa extensions {}", mhartid::read(), ext).ok();
    let c0 = riscv::register::mcycle::read();
    let r = isqrt(1_000_000);
    let c1 = riscv::register::mcycle::read();
    writeln!(u, "isqrt(1000000) = {} ({} cycles)", r, c1.wrapping_sub(c0)).ok();
    let ok = r == 1000 && ext.contains('I') && ext.contains('M') && ext.contains('A') && ext.contains('C');
    writeln!(u, "result: {}", if ok { "PASS" } else { "FAIL" }).ok();
    exit(if ok { 0 } else { 1 })
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    writeln!(Uart, "PANIC: {}", info).ok();
    exit(2)
}
