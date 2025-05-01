The machine.cfg dumped from the core dumps.

**!! Not complete !!**

# Raw Dump

This is just copied from the dumps and is not structured into a valid machine.cfg

```
step_pin: stm32:PC13
dir_pin: stm32:PC14
enable_pin: !stm32:PC15
heater_pin: stm32:PB6
sensor_pin: stm32:PA3
microsteps: 16.0
rotation_distance: 28.8
filament_diameter: 1.75
full_steps_per_rotation: (lost) (actually this may be a float in memory i will lose my shit)
gear_ratio: 52:10
nozzle_diameter: 0.4
pid_Kp: 28.993265
pid_Ki: 6.103818
pid_Kd: 34.429656
min_temp: -200
max_temp: 335
min_extrude_temp: 170
pressure_advance: 0.04
max_extrude_only_accel: 5000.0
max_extrude_only_velocity: 60.0
pressure_advance_smooth_time: 0.01
sensor_type: NTC 100K beta 4300

accel_chip: adxl345 X

cs_pin: stm32:PA4

spi_speed: 5000000
spi_bus: stm32:spi1
adxl_type: lis2dw12
axes_map: x,z,-y
rate: 1600
```