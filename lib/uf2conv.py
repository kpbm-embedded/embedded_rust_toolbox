#!/usr/bin/env python3
"""ELF -> UF2 for RP2040/RP2350 BOOTSEL and the nRF52840 UF2 bootloader (drag-and-drop). Writes a file only.

Usage: python -I uf2conv.py <fw.elf> [-o out.uf2] [--family rp2040|rp2350-arm|rp2350-riscv|nrf52840|0xHEX]
Takes the PT_LOAD segments with file data that lie in flash (XIP 0x10000000..0x11000000 for
RP2040/RP2350, 0x1000..0x100000 for nRF52840) and splits them into 256-byte UF2 payloads. Self-written for 32_17_embedded_rust
(standard library only); format: https://github.com/microsoft/uf2
"""
import argparse
import struct
import sys
from pathlib import Path

FAMILIES = {"rp2040": 0xE48BFF56, "rp2350-arm": 0xE48BFF59, "rp2350-riscv": 0xE48BFF5A, "nrf52840": 0xADA52840}
FLASH = (0x10000000, 0x11000000)  # RP2040/RP2350 XIP flash
# nRF52840 (Adafruit UF2 bootloader): internal flash above the MBR; the bootloader itself refuses
# blocks over the SoftDevice/bootloader, so the app is linked at 0x26000 (nice!nano)
FLASH_BY_FAMILY = {0xADA52840: (0x00001000, 0x00100000)}
MAGIC0, MAGIC1, MAGIC_END = 0x0A324655, 0x9E5D5157, 0x0AB16F30
FLAG_FAMILY = 0x00002000
PAYLOAD = 256


def load_segments(data: bytes, flash: tuple[int, int] = FLASH) -> list[tuple[int, bytes]]:
    """PT_LOAD segments with file data whose physical address lies in flash (default: RP2040 XIP)."""
    if data[:4] != b"\x7fELF" or data[4] != 1 or data[5] != 1:
        sys.exit("uf2conv: not a 32-bit little-endian ELF")
    (phoff,) = struct.unpack_from("<I", data, 28)
    phentsize, phnum = struct.unpack_from("<HH", data, 42)
    segs = []
    for i in range(phnum):
        p_type, p_offset, _vaddr, p_paddr, p_filesz = struct.unpack_from("<IIIII", data, phoff + i * phentsize)
        if p_type == 1 and p_filesz > 0 and flash[0] <= p_paddr < flash[1]:
            segs.append((p_paddr, data[p_offset:p_offset + p_filesz]))
    if not segs:
        sys.exit(f"uf2conv: no loadable segment in flash (0x{flash[0]:x}..0x{flash[1]:x}); wrong --family for this ELF?")
    return sorted(segs)


def to_uf2(segs: list[tuple[int, bytes]], family: int) -> tuple[bytes, int]:
    """Splits the segments into 256-byte UF2 blocks; returns the file content and block count."""
    pages = {}
    for addr, blob in segs:
        for off in range(len(blob)):
            a = addr + off
            page = a & ~(PAYLOAD - 1)
            buf = pages.setdefault(page, bytearray(PAYLOAD))
            buf[a - page] = blob[off]
    out = bytearray()
    total = len(pages)
    for n, page in enumerate(sorted(pages)):
        hdr = struct.pack("<IIIIIIII", MAGIC0, MAGIC1, FLAG_FAMILY, page, PAYLOAD, n, total, family)
        block = hdr + bytes(pages[page]) + bytes(476 - PAYLOAD) + struct.pack("<I", MAGIC_END)
        assert len(block) == 512
        out += block
    return bytes(out), total


def main() -> int:
    ap = argparse.ArgumentParser(prog="uf2conv", description=__doc__.splitlines()[0])
    ap.add_argument("elf", type=Path, help="firmware ELF (32-bit, linked for the family's flash)")
    ap.add_argument("-o", "--output", type=Path, help="UF2 file to write (default: <elf without .elf>.uf2)")
    ap.add_argument("--family", default="rp2040",
                    help=f"UF2 family: {', '.join(FAMILIES)} or a number like 0xe48bff56 (default rp2040)")
    a = ap.parse_args()
    try:
        fam = FAMILIES.get(a.family) or int(a.family, 0)
    except ValueError:
        ap.error(f"unknown family {a.family!r}")
    if not a.elf.is_file():
        ap.error(f"{a.elf}: no such file (build first: rust-box examples build)")
    uf2, blocks = to_uf2(load_segments(a.elf.read_bytes(), FLASH_BY_FAMILY.get(fam, FLASH)), fam)
    out = a.output or a.elf.with_suffix(".uf2")
    out.write_bytes(uf2)
    print(f"uf2conv: {out}: {blocks} blocks ({blocks * PAYLOAD} bytes of flash), family 0x{fam:08x}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
