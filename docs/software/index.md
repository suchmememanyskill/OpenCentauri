# Software

Information about the software running on the Centauri Carbon.

State: Research

This page contains miscellaneous notes.

### OS

The Centauri Carbon runs on top of Tinalinux. The kernel has version 5.4.61. The installed version of glibc is 2.23.

### Is the Centauri Carbon running Klipper

The hotend and bed use a fairly standard Klipper setup, with bed-specific extensions (hx711s and dirctl) for the pressure sensors. The DSP (used as a Klipper MCU) runs Klipper MCU code that has been extended and modified for DSP use.

The mainboard host runs a monolithic app that exposes the web UI, camera, API, screen UI, machine configuration, and most importantly Klippy (transpiled to C++).

The version of Klipper used on the DSP is `v0.9.1-616-g28f60f7e-dirty-20220408_035823-fluiddpi`.

See the [Custom Gcode](custom-gcode.md) page for instructions on dumping the .cfg files.

!!! note
    Because Klippy is heavily modified, not everything is supported. Modifying the Klipper .cfg may lead to a bricked machine.

### Speed profiles

Speed setting | Speed multiplier
---|---
Silent|50%
Balanced|100%
Sport|130%
Ludicrous|160%