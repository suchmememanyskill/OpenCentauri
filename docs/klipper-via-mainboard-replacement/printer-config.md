# !!! WORK IN PROGRESS !!!

The printer.cfg used for the BTT SKR Mini 3

```yaml
# Mainboard
[mcu]
serial: /dev/serial0
baud: 250000
restart_method: command

[mcu hotend]
serial: /dev/ttyHotend
restart_method: command

[printer]
max_velocity: 500
max_accel: 10000
kinematics: corexy

## ========================================================= Toolhead Movement =========================================================
[stepper_x]
step_pin: PB13
dir_pin: PB12
enable_pin: !PB14
microsteps: 32
rotation_distance: 40
endstop_pin: tmc2209_stepper_x:virtual_endstop
position_endstop: -1.20
position_min:-1.25
position_max: 256
homing_speed: 40
homing_retract_dist: 0

[tmc2209 stepper_x]
uart_pin: PC11
tx_pin: PC10
uart_address: 0
run_current: 1.0
interpolate: true
sense_resistor: 0.1
driver_sgthrs: 120
diag_pin: PC0

[stepper_y]
step_pin: PB10
dir_pin: PB2
enable_pin: !PB11
microsteps: 32
rotation_distance: 40
endstop_pin: tmc2209_stepper_y:virtual_endstop
position_endstop: -1.20
position_min: -1.25
position_max: 265
homing_speed: 40
homing_retract_dist: 0

[tmc2209 stepper_y]
uart_pin: PC11
tx_pin: PC10
uart_address: 2
run_current: 1.0
interpolate: true
driver_sgthrs: 120
sense_resistor: 0.1
diag_pin: PC1

## ========================================================= Bed Movement =========================================================
[stepper_z]
step_pin: PB0
dir_pin: PC5
enable_pin: !PB1
microsteps: 16
rotation_distance: 8
endstop_pin: PC15
#position_endstop: 5.5
homing_retract_dist: 7
position_min: -2
position_max: 256

[tmc2209 stepper_z]
uart_pin: PC11
tx_pin: PC10
uart_address: 1
run_current: 0.8
driver_sgthrs: 255
interpolate: false
diag_pin: PC2

[homing_override]
set_position_z: 0
gcode:
  G0 Z5 F600
  SET_TMC_CURRENT STEPPER=stepper_x CURRENT=0.500
  SET_TMC_CURRENT STEPPER=stepper_y CURRENT=0.500
  G4 P500 # Allow TMC stall flag to clear
  G28 X # The printer can get stuck on the left egde of the rear fan housing
  G4 P500 # Allow TMC stall flag to clear
  G28 Y # We don't home Y first because if the toolhead happens to be at the right edge, the filament cutter will engage
  G4 P500 # Allow TMC stall flag to clear
  G28 X # Home again after we cleared from the rear fan housing
  G28 Z # Finally zome Z
  SET_TMC_CURRENT STEPPER=stepper_x CURRENT=1.0 HOLDCURRENT=0.5
  SET_TMC_CURRENT STEPPER=stepper_y CURRENT=1.0 HOLDCURRENT=0.5

## ========================================================= Extruding =========================================================
[extruder]
# Pins
sensor_pin: hotend:PA3
heater_pin: hotend:PB6
step_pin: hotend:PC13
dir_pin: hotend:PC14
enable_pin: !hotend:PC15
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

[tmc2209 extruder]
tx_pin: hotend:PC6
uart_pin: hotend:PC7
interpolate: false
run_current: 0.8
hold_current: 0.5
uart_address: 3
sense_resistor: 0.1
driver_sgthrs: 10

## ========================================================= Bed Heating =========================================================
[heater_bed]
heater_pin: PC9
sensor_type: NTC 100K MGB18-104F39050L32
sensor_pin: PC4
control: pid
pid_Kp: 46.504839
pid_Ki: 1.453276
pid_Kd: 372.038710
min_temp: -10
max_temp: 125
pwm_cycle_time: 0.016

## ========================================================= Fans =========================================================
[heater_fan hotend_cooling_fan]
pin: hotend:PC8
heater_temp: 50
tachometer_pin: hotend:PA1

[fan]
pin: hotend:PB5
tachometer_pin: hotend:PA0

## ========================================================= Bed Mesh =========================================================
[bed_mesh]
speed: 80
horizontal_move_z: 7
mesh_min: 5, 5
mesh_max: 245, 250
probe_count: 3, 3
fade_end: 1.0
algorithm: lagrange
faulty_region_1_min: 245.0, 0.0
faulty_region_1_max: 256.0, 40.0

[probe]
pin: tmc2209_stepper_z:virtual_endstop
samples: 3
samples_result: average
z_offset: 0

## ========================================================= Extra temperatures =========================================================

[temperature_sensor Chamber]
sensor_type: NTC 100K MGB18-104F39050L32
sensor_pin: PA0

## ========================================================= Resonance Compensation =========================================================

[lis2dw]
spi_speed: 5000000
#spi_bus: hotend:spi1
spi_software_sclk_pin: hotend:PA5
spi_software_mosi_pin: hotend:PA7
spi_software_miso_pin: hotend:PA6
axes_map: x,z,-y
#rate: 1600
cs_pin: hotend:PA4

[resonance_tester]
accel_chip: lis2dw
probe_points: 128, 128, 5
min_freq: 5
max_freq: 75
accel_per_hz: 100
hz_per_sec: 1
max_smoothing: 0.05

## ========================================================= Macros =========================================================

[gcode_macro Run_Bed_Mesh]
gcode:
  SET_TMC_CURRENT STEPPER=stepper_z CURRENT=0.230 HOLDCURRENT=0.01
  G0 Z0 F100
  G4 P1000 # Allow TMC flag to clear
  BED_MESH_CALIBRATE PROFILE=PEI METHOD=automatic HORIZONTAL_MOVE_Z=3 PROBE="PROBE_SPEED=50"
  SET_TMC_CURRENT STEPPER=stepper_z CURRENT=0.800 HOLDCURRENT=0.5

[gcode_macro START_PRINT]
gcode:
  BED_MESH_PROFILE LOAD="PEI"

[gcode_macro UNLOAD_FILAMENT]
gcode:

[gcode_macro LOAD_FILAMENT]
gcode:

[include mainsail.cfg]
```