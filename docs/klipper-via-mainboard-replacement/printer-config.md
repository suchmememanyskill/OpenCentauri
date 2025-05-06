# !!! WORK IN PROGRESS !!!

Reconstruction for the Centauri Carbon Printer.cfg

```yaml
[mcu]
serial: /dev/ttyACM1
baud: 1000000
restart_method: command

[printer]
max_velocity: 500
max_accel: 20000
# TODO: Will be upated for the last step of the Klipperification when the hotend and the hot bed are done
kinematics: none 

[extruder]
# Pins
sensor_pin: PA3 ## Update to hotend MCU prefix once we have multiple boards added
heater_pin: PB6 ## Update to hotend MCU prefix once we have multiple boards added
step_pin: PC13 ## Update to hotend MCU prefix once we have multiple boards added
dir_pin: PC14 ## Update to hotend MCU prefix once we have multiple boards added
enable_pin: !PC15 ## Update to hotend MCU prefix once we have multiple boards added
# Metadata
nozzle_diameter: 0.4
filament_diameter: 1.75
# Heating
sensor_type: NTC 100K MGB18-104F39050L32
min_temp: -10
max_temp: 335
control: pid
pid_Kp: 28.993265
pid_Ki: 6.103818
pid_Kd: 34.429656
min_extrude_temp: 170
# Feeding
microsteps: 16
rotation_distance: 28.8
gear_ratio: 52:10
max_extrude_only_accel: 5000.0
max_extrude_only_velocity: 60.0
pressure_advance_smooth_time: 0.01
pressure_advance: 0.04

[heater_fan hotend_cooling_fan]
pin: PC8 ## Update to hotend MCU prefix once we have multiple boards added
heater_temp: 50
tachometer_pin:	PA1 ## Update to hotend MCU prefix once we have multiple boards added

[fan model]
pin: PB5 ## Update to hotend MCU prefix once we have multiple boards added
tachometer_pin:	PA0 ## Update to hotend MCU prefix once we have multiple boards added

[tmc2209 extruder]
stealthchop_threshold: 9999999999
tx_pin: PC6 ## Update to hotend MCU prefix once we have multiple boards added
uart_pin: PC7 ## Update to hotend MCU prefix once we have multiple boards added
interpolate: False
run_current: 0.8
hold_current: 0.5
uart_address: 3
sense_resistor: 0.1
driver_sgthrs: 10

################################# !!! DOES NOT WORK WITHOUT MATCHING KLIPPER VERSION !!!
[resonance_tester]
accel_chip: adxl345 X
probe_points: 128, 128, 5
min_freq: 5
max_freq: 75
accel_per_hz: 100
hz_per_sec: 1
max_smoothing: 0.05

################################# !!! DOES NOT WORK WITHOUT MATCHING KLIPPER VERSION !!!
[adxl345 X]
spi_speed: 5000000
spi_bus: spi1 ## Update to hotend MCU prefix once we have multiple boards added
axes_map: x,z,-y
rate: 1600
cs_pin: PA4 ## Update to hotend MCU prefix once we have multiple boards added

[include mainsail.cfg]
```