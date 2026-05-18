# CC2 Toolhead

![Toolhead board image](./assets/cc2_toolheadboard.jpg){ width="800" }
/// caption
Credit to keefe826 on the OpenCentauri Discord.
///

The toolhead board is connected over a USB-C cable. Unlike the CC1, serial is used instead of USB protocol for communication. The toolhead board receives 24V power.



## Supplementary board

![Toolhead supplementary board image](./assets/supplementary_board.jpg){ width="600" }
/// caption
Credit to keefe826 on the OpenCentauri Discord.
///

The Toolhead board has a 2x4 pin port at the bottom. This connector links to a separate PCB that breaks out the required hotend connectors (temperature sensor, heater, and hotend fan).

## Filament Detector Board



The CC2 has an additional filament detector board connected to the bottom port on the opposite side of the supplementary board pins. It uses an optical sensor to detect filament entry into the extruder. A spring on the back of the board retains a lever that blocks the optical sensor when filament enters the extruder. A redesigned front and rear extruder shell accommodate both this detector board and the filament [multiplexer](CANVAS.md#filament-multiplexer).

This board also includes forward- and rear-facing Hall effect sensors. The forward sensor, located near the middle of the board, detects the toolhead cover using a small magnet in the CC2 toolhead. The rear sensor, located near the top-back side of the board and extending over the multiplexer, is used for tangle detection. A spring-loaded tab in the multiplexer extends under filament tension and triggers this sensor once tension exceeds a threshold.


![Filament detector board image](./assets/cc2_filamentdetector.jpg){ width="800" }
/// caption
Credit to keefe826 on the OpenCentauri Discord.
///

![Rear detector board image](./assets/cc2_fdback.jpg){ width="600" }
/// caption
Back side of detector board showing filament actuation lever, optical sensor, and multiplexer sensor. Credit to sune2573 on the OpenCentauri Discord.
///

![Filament detector board annotated with hall effect-based cover detection](./assets/cc2_fd2.jpg){ width="600" }
/// caption
Filament detector board annotated with hall effect-based cover detection. Credit to keefe826 on the OpenCentauri Discord.
///

## Filament cutter actuation sensor

A small board screwed into the hotend uses a Hall effect sensor to detect filament cutter actuation via a magnet in the filament cutter arm. It connects to the right side of the filament detector board.

![Fan shroud board](./assets/cc2_fanductboard.jpg){ width="400" }
/// caption
Fan shroud board. Credit to u/CalligrapherLoud778 on the Elegoo subreddit.
///
![Filament cutter magnet](./assets/filamentcutter_magnet.jpg){ width="380" }
/// caption
Filament cutter magnet location highlighted in red
///

## MCU

Metric|Value
---|---
MCU|
Vendor Id|
Product Id|
Device BCD|
Product|
Manufacturer|
Stepper driver|tmc2209

## Hardware

Metric|Value
---|---
Motor type|10T NEMA14 (round, 20.5mm long)
Motor P/N|BJY36D12-04V28
Motor MFG|SHENZHEN  KELI MOTOR  LTD
Extruder gear ratio|52:10
Extruder hobbed gear diameter|10mm nominal
Extruder hobbed gear material|SDK11 tool steel
Heater type|Ceramic plate-type PTC heater
Heater resistance|~9.6Ω
Heater power|60W
Thermistor Type| Glass bead NTC-150k*
Thermistor Beta| 4300
Fan manufacturer| Shenzhen Hua Xinrong Plastic Electronics Co., Ltd
Part cooling fan type|5020 custom radial fan integrated into duct, 4 pin (tach+5V PWM)
Part cooling fan P/N|EFC-05D24D
Part cooling fan power|0.50A @ 24V
Part cooling fan speed|12,000 RPM
Hotend fan type|3010 axial fan, 3 pin (tach)
Hotend fan P/N|
Hotend fan power|0.10A @ 24V
Hotend fan speed|12,000 RPM

/// caption
Credit to reddit user 6Y3ts_32a for thermistor resistance measurements.
///

![custom toolhead fan](./assets/cc2_toolhead.jpg){ width="800" }
/// caption
Credit to keefe826 on the OpenCentauri Discord.
///

![custom toolhead fan internals](./assets/cc2_fan.jpg){ width="800" }
/// caption
Credit to sune2573 on the OpenCentauri Discord.
///