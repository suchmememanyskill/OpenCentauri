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

The board connects with serial (not over USB) to the mainboard.

## Screen

The screen is a generic `0430A046-I1I1100` LCD screen (capacitive touch screen version). The driver board is custom, probably just re-routes the screen wires to the mainboard that implements the actual driver.

# Mainboard Pins

### Z-Axis Optical Sensor (titled EXT on board)

|pin|voltage|
|---|----|
|+|24v|
|-|GND|
|s| 3.3v when bed is zeroed, 0v when not|

### Filament Runout Sensor

|pin|voltage|
|---|---|
|+|5v|
|-|GND|
|s|3v3 when filament is inserted, 0v when not|

### Light

|pin|Voltage|
|---|---|
|+|24v when Light is enabled, 0v when not*|
|-|GND|

\* Enabled/disabled with along the light on the camera

### Camera

The camera uses an USB 2.0 connection with a JST connector.

### Bed Heating

|pin|Voltage|
|---|---|
|+|24v when bed is heating, 0 when not|
|-|GND|

### Hotend

Serial-Over-USB connection but the mainboard's VBus is connected to the PSU 24v. The board boots from a simple 5v USB connection.