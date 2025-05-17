### Reconstruction of the original `printer.cfg` for the stock setup.

```yaml
[mcu]
serial: /dev/ttyACM1
baud: 1000000
restart_method: command

# TODO: MCU for hotend

# TODO: MCU For new Mobo

[printer]
max_velocity: 500
max_accel: 20000
# TODO: Will be upated for the last step of the Klipperification when the hotend and the hot bed are done
kinematics: none

[extruder]
# Pins
sensor_pin: PA3 ## TODO Update to hotend MCU prefix once we have multiple boards added
heater_pin: PB6 ## TODO Update to hotend MCU prefix once we have multiple boards added
step_pin: PC13 ## TODO Update to hotend MCU prefix once we have multiple boards added
dir_pin: PC14 ## TODO Update to hotend MCU prefix once we have multiple boards added
enable_pin: !PC15 ## TODO Update to hotend MCU prefix once we have multiple boards added
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

[heater_bed]
heater_pin: PG13
sensor_type: NTC 100K MGB18-104F39050L32
control: pid
pid_Kp: 46.504839
pid_Ki: 1.453276
pid_Kd: 372.038710
pwm_cycle_time: 0.016
min_temp: -10
max_temp: 125

[verify_heater heater_bed]
max_error: 120
check_gain_time: 20
hysteresis: 5
heating_gain: 2

[temperature_sensor box]
sensor_pin: PB13 ## TODO Update to hotend MCU prefix once we have multiple boards added
sensor_type: NTC 100K MGB18-104F39050L32
min_temp: -10
max_temp: 125

[heater_fan hotend_cooling_fan]
pin: PC8 ## TODO Update to hotend MCU prefix once we have multiple boards added
heater_temp: 50
tachometer_pin:	PA1 ## TODO Update to hotend MCU prefix once we have multiple boards added

[fan model]
pin: PB5 ## TODO Update to hotend MCU prefix once we have multiple boards added
tachometer_pin:	PA0 ## TODO Update to hotend MCU prefix once we have multiple boards added

# TODO, Config completely scrambled
# AUX Fan
# [fan model_helper_fan]

[fan box_fan] # Case-fan
pin: PG18 # TODO Need to be adjusted for the mobo

[controller_fan board_cooling_fan]
pin: PG16 # TODO Need to be adjusted for the mobo

[tmc2209 extruder]
stealthchop_threshold: 9999999999
tx_pin: PC6 ## TODO Update to hotend MCU prefix once we have multiple boards added
uart_pin: PC7 ## TODO Update to hotend MCU prefix once we have multiple boards added
interpolate: false
run_current: 0.8
hold_current: 0.5
uart_address: 3
sense_resistor: 0.1
driver_sgthrs: 10

[tmc2209 stepper_x]
tx_pin: PE9 # TODO Need to be adjusted for the mobo
uart_pin: PE6 # TODO Need to be adjusted for the mobo
uart_address: 0
interpolate: true
run_current: 1.0
sense_resistor: 0.1
microsteps: 16
driver_sgthrs: 120
diag_pin: !PG4 # TODO Need to be adjusted for the mobo

[tmc2209 stepper_y]
tx_pin: PE9 # TODO Need to be adjusted for the mobo
uart_pin: PE6 # TODO Need to be adjusted for the mobo
interpolate: true
run_current: 1.0
uart_address: 3
sense_resistor: 0.1
microsteps: 16
driver_sgthrs: 120
diag_pin: !PF6 # TODO Need to be adjusted for the mobo

[tmc2209 stepper_z]
tx_pin: PE9 # TODO Need to be adjusted for the mobo
uart_pin: PE6 # TODO Need to be adjusted for the mobo
interpolate: true
run_current: 0.8
uart_address: 1
stealthchop_threshold: 99999
sense_resistor: 0.1
microsteps: 16
driver_sgthrs: 120

[led led1]
red_pin: PC9 # TODO Update to hotend MCU prefix once we have multiple boards added
cycle_time: 0.016

[led led2]
red_pin: PG15 # TODO Need to be adjusted for the mobo
cycle_time: 0.016

[led led3]
red_pin: PC2 # TODO Need to be adjusted for the mobo
cycle_time: 0.016

[led boxLed]
red_pin: PG15
cycle_time: 0.016

[resonance_tester]
accel_chip: adxl345 X
probe_points: 128, 128, 5
min_freq: 5
max_freq: 75
accel_per_hz: 100
hz_per_sec: 1
max_smoothing: 0.05

[adxl345 X]
spi_speed: 5000000
spi_bus: spi1 ## Update to hotend MCU prefix once we have multiple boards added
axes_map: x,z,-y
rate: 1600
cs_pin: PA4 ## Update to hotend MCU prefix once we have multiple boards added

## !!!!!!!!!!!!!!!!!!!!! Fragments !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
sensor0_clk_pin: strain_gauge_mcu:PB13
sensor0_sdo_pin: ## MISSING

sensor1_clk_pin: strain_gauge_mcu:PC7
sensor1_sdo_pin: strain_gauge_mcu:PC8

sensor2_clk_pin: strain_gauge_mcu:PC6
sensor2_sdo_pin: # MISSING

sensor3_clk_pin: strain_gauge_mcu:PC9
sensor3_sdo_pin: strain_gauge_mcu:PA8
```