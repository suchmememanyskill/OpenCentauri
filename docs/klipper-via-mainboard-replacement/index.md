# **Not Complete**

Some features are still lacking Klipper supports that the Elegoo CC uses.

# ***THIS IS A ONE-WAY TRIP***
Elegoo uses a modified version of Klipper, which is also outdated (Klipper commit #28f60f7e)

# ***AND IS BY FAR NOT A DROP-IN UPGRADE***
Elegoo used a series of customized connectors and solutions. You have to do a range of manual labour.

# Preword

The complexity of using the CC with a Klipper is still high, but possible with limitations. Currently Klipper has very poor support for Load Cells,
no support for load cell probing ([currently in PR](https://github.com/Klipper3d/klipper/pull/6871)) and unknown support for probing with multiple load cells.

# Requirements:

- Solder + Soldering Iron
- Dupont Crimp Tool
- Dupont Headers + terminals
- JST-XH Female headers + terminals (optional but comes really handy)
- Jumper Wires
- 1mm copper wires
- CC, obviously
- BTT SKR Mini E3 V3
- USB-TTL Transciever and/or ST-Link
- USB-C Female Breakout board (4 or 5 pin, doesn't matter)
- STM32Cube Programmer (free to download)

# Steps

1. [Flashing Hotend board](pages/flash-hotend.md)
1. [Flashing bed board](pages/flash-bed.md)
1. [Making cable connectors](pages/custom-connectors.md)
1. [Connecting things](pages/connection-map.md)
1. [Configuring Raspberry](pages/config-raspy.md)
1. [Printer.cfg](pages/config-mobo.md)


[Printer Config](printer-config.md)

[Mainsail Config](mainsail-config.md)

[Hotend-Wiring](recognize-hotend.md)

[UDEV Config](udev-setup.md)

[Rewiring to BTT SKR Mini E3 V3](rewire-btt-skr-mini3.md)
