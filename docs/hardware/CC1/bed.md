![Bed overview](./assets/centauri-bed-overview.jpg){ width="600" }
/// caption
Credit to rabirx on the OpenCentauri Discord.
///

![Bed underside](./assets/bed1.jpg){ width="600" }
/// caption
Credit to baconmilkshake on the OpenCentauri Discord.
///


Front|Back
---|---
![Bed board image](./assets/centauri-bed.jpg){ width="800" }|![Bed board back image](./assets/centauri-bed-back.jpg){ width="800" }
Credit to rabirx on the OpenCentauri Discord.|Credit to rabirx on the OpenCentauri Discord.

The bed is its own Klipper MCU with an accelerometer and some pressure sensors. The heating is not controlled by the MCU, but via a seperate AC board.

The board connects with serial (not over USB) to the mainboard.

## MCU

Metric|Value
---|---
MCU|STM32F402RCT6
Vendor Id|1d50
Product Id|614e
Device BCD|2.00
Product|STM32 Virtual ComPort
Manufacturer|ShenZhenCBD

## Hardware
Metric|Value
---|---
Resistance|~48.4Ω
Operating Voltage| 220V/110V
Power|1000W (220V)/250W (110V)
Safety mechanisms|Gnd Present, Thermal Fuse
Thermistor type|NTC100K
Thickness|3mm aluminum plate, 1.5mm magnetic sheet