# Mainboard

Metric|Value
---|---
CPU|AllWinner R528-S3
Memory|128 MB Onboard
Storage|8gb eMMC
Stepper drivers|tmc2209

![Mainboard image](../assets/centauri-mobo.jpg){ width="400" }
/// caption
Credit to the Elegoo discord.
///

## Hotend

The hotend is connected over a USB-C cable. This USB-C interface carries 24v.

!!! question "Speculation"
    I suspect that D+/D- is either a direct serial connection or a serial-over-usb interface.

The hotend runs Klipper MCU firmware.

## Bed

The bed is its own Klipper MCU with an accelerometer and some pressure sensors. The heating is not controlled by the MCU, but via a seperate AC board.