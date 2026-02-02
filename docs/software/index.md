# Software

Information about the software running on the Centauri Carbon printers.

State: Research

## API Documentation

The Centauri Carbon printers use different communication protocols depending on the model:

| Printer | Protocol | Discovery | Main Transport |
|---------|----------|-----------|----------------|
| CC1 (Centauri Carbon) | SDCP v3.0.0 | UDP port 3000 | WebSocket |
| CC2 (Centauri Carbon 2) | MQTT-based | UDP port 52700 | MQTT |

See the API documentation for each printer:

- [CC1 API Documentation](CC1/api.md) - SDCP WebSocket protocol
- [CC2 API Documentation](CC2/api.md) - MQTT-based protocol with printer as broker

## General Notes

This page contains some misc notes.

### OS

The Centauri Carbon runs on top of Tinalinux. The kernel has version 5.4.61. The installed version of glibc is 2.23.

### Is the Centauri Carbon running Klipper

The hotend and bed uses a pretty standard install of klipper, with some extensions for the bed specifically (hx711s, dirctl) for the pressure sensors. The DSP (used as a klipper MCU) is running klipper mcu code, but extended/modified to run on a DSP. 

The mainboard host runs a monolithic app that exposes the webui, camera, api, screen ui, machine configuration, and most importantly klippy (transpiled to c++). 

The version of klipper used on the DSP is `v0.9.1-616-g28f60f7e-dirty-20220408_035823-fluiddpi`

See the [Custom Gcode](custom-gcode.md) page to see how to dump the .cfg's.

!!! note
    As klippy is heavily modified, not everything is supported. Modifying the klipper .cfg may lead to a bricked machine.

### Speed profiles

Speed setting | Speed multiplier
---|---
Silent|50%
Balanced|100%
Sport|130%
Ludicrous|160%

### Getting a coredump

Coredumps sadly have their executable memory stripped :(

But they still contain a lot of useful information, specifically the strings of running programs are pretty readable.

1. Insert a USB drive into your PC.
1. Create a folder called `Crash` on your USB drive.
1. Copy [a corrupt .gcode file](../assets/ECC_0.4_dust%20cover%20lr_PLA0.2_2h52m.gcode) to this new `Crash` folder.
1. Eject the USB
1. Put it inside the Centauri Carbon
1. Navigate to your USB drive, then press the `Crash` folder.
    - Your Centauri Carbon will now crash.
1. After a restart, go to settings > `Export Logs`

You now have a coredumps.tar.gz that has a coredump inside on your USB drive.

Coredumps can be loaded in IDA, Ghidra, BinaryNinja, or any other analyser of your choice.