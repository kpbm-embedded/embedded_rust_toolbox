/* nice!nano v2 / Pro Micro nRF52840 with the Adafruit nRF52 bootloader and SoftDevice S140 6.1.1:
     0x00000000..0x00026000  MBR + SoftDevice S140 (kept; this app does not enable it)
     0x00026000..0x000F4000  application
     0x000F4000..0x00100000  UF2 bootloader + settings
   RAM starts after the 8 bytes the MBR reserves. Linking at 0 would overwrite the bootloader (SWD only). */
MEMORY {
    FLASH : ORIGIN = 0x00026000, LENGTH = 0xF4000 - 0x26000
    RAM   : ORIGIN = 0x20000008, LENGTH = 256K - 8
}
