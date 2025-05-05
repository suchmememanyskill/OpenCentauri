Metric|Value
---|---
CPU|AllWinner R528-S3
Memory|128 MB Onboard
Storage|8gb eMMC
Stepper drivers|tmc2209

Front|Back
---|---
![Mainboard image](../assets/centauri-mobo.jpg){ width="800" }|![Mainboard back image](../assets/centauri-mobo-back.jpg){ width="800" }
Credit to the Elegoo discord.|Credit to thijskunst on the OpenCentauri Discord.

## Mainboard Pins

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

\* Matches the state of the light on the camera module.

### Camera

The camera uses an USB 2.0 connection with a JST connector.

### Bed Heating

|pin|Voltage|
|---|---|
|+|24v when bed is heating, 0 when not|
|-|GND|

### Hotend

Serial-Over-USB connection but the USB's VBus is connected to the PSU's 24v line.