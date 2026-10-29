/* Application: flash after the 32 KB bootloader; its vector table is at 0x8000 (the bootloader sets VTOR).
   The last 4 bytes of the image hold no header: the bootloader validates the vector table instead. */
MEMORY {
    FLASH : ORIGIN = 0x00008000, LENGTH = 256K - 32K
    RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}
