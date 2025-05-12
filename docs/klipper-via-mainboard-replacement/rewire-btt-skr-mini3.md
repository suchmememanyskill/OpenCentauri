# Rewiring CC pheripherials to BTT SKR Mini E3 V3

|Cable|CC MOBO Port|Target Board|Target Port|
|-----|-----|-----|-----|
|Camera|USB (hub)|Raspberry|USB|
|Hotbed CANbus|Unmarked (5 pin)|Raspberry|USB CAN Transciever|
|Hotend USB|USB-C (fix 24v)|Raspberry|USB (with high power adapter)
|Box Thermistor|T-Box|SKR E3|TH0|
|Box Fan (case)|BOX-F|SKR E3|FAN2|
|AUX Fan|FAN1|SKR E3|FAN0|
|Filament Runout|Filament|SKR E3|E0-Stop (5v)$_{Note2}$|
|X-Motor|X|SKR E3|XM|
|Y-Motor|X|SKR E3|YM|
|Z-Motor|X|SKR E3|ZM|
|MOBO Fan|BFAN|SKR E3|?? (3 port)
|Bead Heating|HBED|SKR E3|Bed Heater|
|Bed Thermistor|BED-T|SKR E3|THB|
|Z-Home Sensor|EXT|SKR E3|?? (24v)|

SKR <-> Raspberry Through EXP1

Notes:
1. Despite the CC MOBO driving the Z-home sensor with 24v, it also works fine with 5v.
1. The direction of cables are reversed, either the connector needs to be swapped manually, or an adapter is needed.