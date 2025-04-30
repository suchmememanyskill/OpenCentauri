# Software

Information about the software running on the centauri carbon

State: Research

This page contains some misc notes.

### OS

The Centauri Carbon seems to be running a variant of TinaLinux/OpenWrt, considering the coredump references it.

### Is the Centauri Carbon running Klipper

!!! question "Speculation"
    I suspect so. The coredump contains a lot of references to Klippy, has klipper-esque logs, has bits of a klipper .cfg (/board-resource/printer.cfg), and has klipper-specific commands.

    Whatever it is, it seems to be very far gone from a standard klipper install...

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

### Emmc dump

!!! failure "Help needed"
    If you're able to dump an emmc and would like to contribute, please consider sending me an emmc dump of the Centauri Carbon.

    See [this issue on github](https://github.com/suchmememanyskill/OpenCentauri/issues/1) for more information