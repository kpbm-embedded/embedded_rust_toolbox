/* Arm MPS2 AN385 (QEMU mps2-an385): 4 MB SSRAM1 as code memory at 0, 4 MB SSRAM2/3 as data */
MEMORY {
    FLASH : ORIGIN = 0x00000000, LENGTH = 4M
    RAM   : ORIGIN = 0x20000000, LENGTH = 4M
}
