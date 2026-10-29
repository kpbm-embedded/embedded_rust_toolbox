# 32_17_embedded_rust - why Embassy, defmt and RTIC

Moved out of README.md (2026-10-25 docs pass).


| Tool | Stars now / ~12 mo ago | Releases last 12 mo, latest | Why |
|---|---|---|---|
| Embassy | 10,007 / 7,606 (2025-10-18) | crates: embassy-rp 2, embassy-stm32 2, executor 1; latest 2026-03-20; commits daily | de-facto async framework, HALs for RP/STM32/nRF/ESP, used by Ariel OS and esp-rtos |
| defmt | 1,240 / 1,082 (2025-11-18) | 4 (1.0.1 .. 1.2.0), latest 2026-10-15 | 1.0 stable since 2025-04; the logging format probe-rs, Embassy, RTIC, esp-hal all speak |
| RTIC v2 | 2,420 / 2,145 (2025-11-02) | rtic 3 (2.3.0..2.4.0), rtic-monotonics 3.0.0 on 2026-10-21 | proven static-priority (SRP) scheduler, zero-cost locks, async software tasks |

Star counts 12 months ago are from Wayback Machine snapshots of the GitHub pages (the GitHub
stargazer API no longer lists timestamps). Articles: Memfault Interrupt, "Asynchronous Rust on
Cortex-M" <https://interrupt.memfault.com/blog/embedded-async-rust>; Ferrous Systems, "defmt 1.0"
<https://ferrous-systems.com/blog/defmt-1-0/>; RTIC v2 book <https://rtic.rs/2/book/en/>.

