## Overview

CANVAS is a multimaterial/multicolor module for the Centauri Carbon 2. It employs a Type B design based on [Happy-Hare nomenclature](https://github.com/moggieuk/Happy-Hare/wiki/Conceptual-MMU) with a filament multiplexer proximal to the toolhead for minimal retraction distance.


## CANVAS Module

![CANVAS core module](./assets/canvas.jpg){ width="800" }
/// caption
Credit to keefe826 on the OpenCentauri Discord.
///

The CANVAS core module mounts on the top frame insert of the CC2 alongside the tophat. The system superficially resembles the Flashforge IFS system but is internally distinct and uses independent hobbed gears and motors rather than a cam-based selector. Four permanent magnet stepper motors are used to control filament channels, likely combined with worm gearing. The motors are produced by Shenzhen Wanzhida Motor Manufacturing Co., Ltd.


![CANVAS internals](./assets/canvas_internals.jpg){ width="800" }

![CANVAS internals](./assets/canvas_internals3.jpg){ width="800" }
/// caption
CANVAS internals. Credit to u/CalligrapherLoud778 on the Elegoo subreddit.
///
![CANVAS motors](./assets/canvas_motors.jpg){ width="800" }
/// caption
CANVAS motors. Credit to u/CalligrapherLoud778 on the Elegoo subreddit.
///

###CANVAS Mainboard

Metric|Value
---|---
MCU|GD32F303RCT6
Vendor Id|
Product Id|
Device BCD|
Product|
Manufacturer|GigaDevice Semicon Beijing
Stepper driver|4xAT8833 (DRV8833 clone)

![CANVAS Mainboard](./assets/canvas_mainboard.jpg){ width="800" }
/// caption
CANVAS Mainboard. Credit to u/CalligrapherLoud778 on the Elegoo subreddit.
///

### RFID Board
An RFID reader board is present in the front of the shell to read filament information, it connects to the rear of the mainboard over I2C.

![CANVAS RFID Board](./assets/canvas_rfidboard.jpg){ width="800" }
/// caption
CANVAS RFID Board. Credit to u/CalligrapherLoud778 on the Elegoo subreddit.
///
![CANVAS RFID connection](./assets/canvas_internals2.jpg){ width="400" }
/// caption
RFID board seen connected to the rear of the CANVAS mainboard. Credit to u/CalligrapherLoud778 on the Elegoo subreddit.
///

### Filament Detector Boards
Filament detector boards are sent along the filament path for each channel and appear to use Hall effect sensors similarly to the IFS


## Spool Holders

## Filament Multiplexer

