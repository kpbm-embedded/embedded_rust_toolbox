//! Zig side of the Rust <-> Zig example: compiled to a static library for the Cortex-M3
//! (thumb-freestanding-eabi) with 32_24_microzig's Zig, linked into the Rust firmware.
//! Exports C-ABI functions and calls one back into Rust (rust_on_block).

/// Implemented in Rust (src/main.rs), called from Zig for every processed block.
extern fn rust_on_block(index: u32, peak: i16) void;

/// CRC-32 (IEEE 802.3, reflected, poly 0xEDB88320), bitwise: small code, no table.
export fn zig_crc32(data: [*]const u8, len: usize) u32 {
    var crc: u32 = 0xFFFF_FFFF;
    for (data[0..len]) |b| {
        crc ^= b;
        var i: u4 = 0;
        while (i < 8) : (i += 1) {
            const mask: u32 = 0 -% (crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    return ~crc;
}

/// Q15 moving-average FIR over blocks of `block` samples; reports each block's peak to Rust.
export fn zig_fir_q15(input: [*]const i16, output: [*]i16, len: usize, block: usize) void {
    const taps = 4;
    var idx: usize = 0;
    var blk: u32 = 0;
    while (idx < len) : (blk += 1) {
        const end = @min(idx + block, len);
        var peak: i16 = 0;
        var n = idx;
        while (n < end) : (n += 1) {
            var acc: i32 = 0;
            var k: usize = 0;
            while (k < taps) : (k += 1) {
                if (n >= k) acc += input[n - k];
            }
            const y: i16 = @intCast(@divTrunc(acc, taps));
            output[n] = y;
            if (@abs(y) > @abs(peak)) peak = y;
        }
        rust_on_block(blk, peak);
        idx = end;
    }
}
