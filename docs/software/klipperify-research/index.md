# Necessary Steps

1. Set up Klipper on host machine
1. Connect Hotend to a Klipper host
    1. [Make the Host recognize the hotend - DONE](recognize-hotend.md)
        1. Update firmware on Hotend board
        1. Downgrade Klipper
    1. Make the Host read and control temperature - DONE
    1. Make the Host control the model fan
    1. Make the Host read the ADXL accelerometer
    1. Make the Host control the extruder motor
2. Connect hot bed to Klipper host
    1. Read accelerometer data (Does the bad have accelerometer?)
    1. Make the Host read the load cell data
3. Connect the wires from the CC-Mainboard to 3rd party board
    1. Control the bed heating
    1. Read the bed temperature
    1. Read the chamber temperature
    1. Control the case fan
    1. Control the AUX fan
    1. Control the X Y Z motors
    1. Read Z-Home sensor
    1. Set up sensorless homing
    1. Set up macro for Filament Change
4. Connect the stock camera to raspberry

[Full Config](full-config.md)