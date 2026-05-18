# Embedded Firmware

The CC uses two different embedded firmware for the bed and the hotend boards with allegedly the same bootloader. The firmwares as of 2025-05-19 are a substitute of Klipper commit `28f60f7e` with a lot of changes.

## Official update method

To update both the hotend and the bed boards, the main firmware attempts to send a file through YModem protocol when the said boards are power cycled during bootup. The firmware files are not encrypted and are stored
on the `rootfsA` or `rootfsB` partition (whichever is active) in the `/lib/firmware` folder. The hotend firmware is called `upgrade-hotend.bin` and the bed board's is `upgrade-bed.bin`. On firmware 1.1.29 the hotend/bed firmware got moved into the `/app/resources` folder.

To replicate this behaviour outside of Elegoo's official software, a [flasher has been developed](https://github.com/suchmememanyskill/OpenCentauri/tree/main/mcu-flasher) that is able to push new firmware.

## STM32 flash structure

Start offset|End offset|Description
---|---|---
0x08000000|0x08007FFF|32Kb Bootloader
0x08008000|0x0800BFFF|16Kb Unknown
0x0800C000|...|Elegoo-Klipper Firmware

The firmware upgrade files include the 16Kb of unknown (mostly 0xFF with some random bytes) data so it must be flashed on address `0x08008000`.

## Flashing custom firmware (mcu-flasher)

Using [mcu-flasher](https://github.com/OpenCentauri/OpenCentauri/tree/main/mcu-flasher) a custom firmware can be flashed through the official Elegoo bootloader. Run the program with `-h` for more information.

## Flashing custom firmware (STM32CubeProgrammer)

!!! Warning "No going back"
    Flashing a new firmware will ERASE THE ELEGOO BOOTLOADER.

    Currently the stock mainboard CANNOT boot from custom firmwares nor the stock firmware without the original bootloader. This essentially means you cannot use the stock mainboard anymore after wiping the firmware from the hotend or bed boards!

    Ask in the discord for a backup of the original bootloader.

### Readout Protection

The hotend and bed boards come with the readout protection enabled by default. As long as you don't remove it, the stock flash is safe. If you instruct the programmer to deactivate the readout protection, the entire flash memory, with the bootloader and the firmware will be wiped.

### Hotend

1. Remove the hotend board (only the main one, the supplementary board is not needed).
1. On the back side, there is a 2x4 copper pad row. Short the `3.3v` and the `BOOT` with a tweezer
    - ![img](assets/HotendFlashPinShort.png){ width="400" }
1. Connect it to your PC via a USB-C cable (no adapter is needed) while shorting the pins
1. Keep it shorted for ~2 seconds, then you can let go
1. Open STM32CubeProgrammer software
1. On the top right, select the USB mode. If you shorted the pins correctly, it should find the hotend board. if not, you'll get a "No DFU detected".
    - ![img](assets/STM32CubeProgrammerMode.png){ width="400" }
1. Go to the second tab in the left side vertical button column
1. In the "Download" section, select the firmware file.
1. Click `Start Programming`

### Bed

1. Either via an added connector or solder jumper wires directly to the right side serial pins to a USB-TTL transceiver as

    - |Bed Board|USB-TTL|
    |--|--|
    |5v|5v|
    |RX|RX|
    |TX|TX|
    |GND|GND|

    - Note: Depending on the USB-TTL board, RX and TX lines might need to be switched.

2. Short the `BOOT` pin to `3.3v`
    - ![img](assets/BedBoardFlashPinShort.png){ width="400" }

3. Connect the USB-TTL board to your PC

4. Short the `RESET` pins on the bed board (no need to keep it shorted, just touch it)
    - ![img](assets/BedBoardResetPinShort.png){ width="400" }

5. In STM32CubeProgrammer, select UART mode and select your USB-TTL device
    - ![img](assets/STM32CubeProgrammerModeUART.png){ width="400" }

6. Go to the second tab in the left side vertical button column
7. In the "Download" section, select the firmware file.
8. Click `Start Programming`

## Firmware Creation

The STM32F402 is an STM32 variant specifically made for the Chinese market. For any configuration, use STM32F401RTC6 (sometimes just STM32F401).

### Building for stock bootloader

The following options need to be set in `make menuconfig` in klipper. See [our klipper fork](https://github.com/OpenCentauri/kalico/tree/main/mcu) for config examples.

#### Hotend

Option|Value
--|--
Micro-controller Architecture|STMicroelectronics STM32
Processor model|STM32F401
Bootloader offset|48KiB bootloader
Clock reference|24Mhz crystal
Communication interface|USB (on PA11/PA12)

#### Bed

Option|Value
--|--
Micro-controller Architecture|STMicroelectronics STM32
Processor model|STM32F401
Bootloader offset|48KiB bootloader
Clock reference|24Mhz crystal
Communication interface|Serial (on USART2 PA3/PA2)
Baud rate for serial port|115200

If the serial pins are used on the right side (non stock configuration), UART1 PA9 / PA10 can be used also.