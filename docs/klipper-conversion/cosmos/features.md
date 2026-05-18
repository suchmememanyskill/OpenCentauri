# COSMOS Extra Features

This page outlines features specific to COSMOS (not to Klipper!), like how configuration works, what macros we added and what recovery options we added.

## Custom Macros

### FULL_CALIBRATION
Performs a full calibration of the machine. Currently includes resonance compensation calibration on the X and Y axis, bed meshing, and PID calibration of the extruder.

Args:

- EXTRUDER_TEMP: number. Optional.
- BED_TEMP: number. Optional.

Example: `FULL_CALIBRATION EXTRUDER_TEMP=220 BED_TEMP=60`

### LOAD_FILAMENT
Loads filament into the toolhead. After loading, a prompt asks if you want to extrude more.

Args:

- EXTRUDER_TEMP: number. Optional.

Example: `LOAD_FILAMENT EXTRUDER_TEMP=250`

### UNLOAD_FILAMENT
Unloads filament from the toolhead. This utilises the filament cutter and keeps the extruder cold.

Example: `UNLOAD_FILAMENT`

## Klipper Configuration

COSMOS uses Klipper under the hood, and allows the user to edit its configuration. The configuration can be edited via the webui.

Relevant files are:

- `printer.cfg`: User facing configuration. Add your custom Klipper macros in here. SAVE_CONFIG saves its configuration to this file as well.
- `klipper-readonly/*.cfg`: System configuration. If these files are edited, they will stop receiving any future updates. If you wish to override a section, copy paste the relevant section to your printer.cfg, and editing it there. Nothing stops you from editing the files, though.

??? Note "Revert klipper-readonly .cfg's"
    Run the following SSH command: `rm -f /data/overlay-etc/upper/klipper/config/klipper-readonly/* && sync && echo 3 > /proc/sys/vm/drop_caches`

## UI Configuration

See `cosmos.conf` for configuration on which user interface is used.

Screen options:

- grumpyscreen

Webui options:

- mainsail
- fluidd

Restart your printers for your changes to take effect.

## Recovery

=== "USB"

    COSMOS offers 2 recovery options on USB devices

    ### Wifi recovery: Set a wifi network via USB

    Create a file called `wpa_supplicant.conf` on the root of your USB drive. Use the following template:

    ```
    ctrl_interface=/var/run/wpa_supplicant
    network={
        ssid="ssid"
        psk="plaintext_password"
    }
    ```

    When the USB is inserted, the `wpa_supplicant.conf` will be deleted. Reboot your machine for the changes to take effect.

    ### Firmware recovery: Install an .swu via USB

    Copy a .swu file as `emergency.swu` on the root of your USB device. When the USB is inserted, it will move the file to `emergency.swu.installed` and install it. If klipper is running, it will show a pop up dialog with the installation progress. Reboot after the intstallation has finished.

=== "UART/FEL"

    UART/FEL is supported under COSMOS. See [the UART/FEL setup docs page](../../hardware/CC1/fel-uart-setup.md) for more information.

