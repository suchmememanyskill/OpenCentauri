# Patched firmware

This section describes the installation process and features of OpenCentauri's patched firmware.

This firmware is based on Elegoo's 1.1.40 firmware, with patches that address unwanted behavior and bugs.

If you find bugs or want to suggest new features, please use the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools){target="_blank"} repository. Not everything is possible within the limits of patching existing firmware, especially large features or full overhauls.

This project is currently in beta, so expect some issues.

If you need support, feel free to [join the Discord](https://discord.gg/t6Cft3wNJ3).

If you'd like to support our work, you can do so on [Ko-Fi](https://ko-fi.com/opencentauri) :heart:! If spending money isn't your thing, we also have a [Makerworld page](https://makerworld.com/en/models/1924078-opencentauri-logo#profileId-2064746) where you can throw some boosts towards!

!!! warning "**Looking for Full Klipper/COSMOS?**"

    This is regular OpenCentauri patched from official Elegoo firmware, for Beta full Klipper/Kalico firmwarewith COSMOS go [here](../klipper-conversion/cosmos/cosmos.md)

## Features (as of v0.3.0)

- SSH (user: "root", password: "OpenCentauri")
- Developer features (package manager, prevent booting Klipper, etc)
- Bootlogo can be replaced
    - [See available bootlogos on github](https://github.com/OpenCentauri/cc-fw-tools/tree/main/oc-patches/replace-bootlogo-patch){target="_blank"}
- Exhaust fan no longer automatically turns on during a print
    - Note: In OrcaSlicer the default profile still turns on this fan using a M106 P3 command when using PLA.
    - Note: On Elegoo filaments present in OrcaSlicer, the `Exhaust fan` section present on the Cooling tab inside of a Filament profile is respected. If it is enabled, then exhaust fan will turn on at the specified speed.
- Homing position has been set to the front right instead of the front left
- The webui (and other integrations like octoeverywhere and home assistant) now accepts modifications during a print (bug introduced in firmware 1.1.29. This was fixed in firmware 1.1.42 but has been backported to 1.1.40)
- The webui's store button has been removed
- The webui's logo has been replaced with an OpenCentauri logo
- The webui's corner radius (white pixels near the corners) has been fixed
- Z offset can be adjusted while the printer is idle
- Files can be uploaded while the printer is printing
- Filament usage is reported via the API 
- Connectivity checks are blocked (web traffic issue)
- Official OTA updates have been replaced by OpenCentauri OTA updates
- Support for USB Ethernet adapters
- New gcode commands: 
    - `M8212` to turn off the chamber light
    - `M8213` to turn on the chamber light
    - `TEMPERATURE_WAIT SENSOR=box MINIMUM=XX` to wait until a certain temperature has been reached in the chamber

## Installation

=== "Online"
    !!! warning "Online requirement"

        Your Centauri Carbon needs to be connected to the internet in order to download OpenCentauri firmware. Choose the `Local/Offline` installation method if your Centauri Carbon is not connected to the internet.

    1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release
    1. Insert a fat32-formatted thumbdrive into your pc
    1. Extract the `install_opencentauri` folder from `oc-installer.zip` onto the root of the thumbdrive
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon
    1. On your Centauri Carbon, navigate to the files tab, then tap the `Usb Drive` menu, then tap the `install_opencentauri` folder
    1. Touch hold the `IMPORT_ME_DO_NOT_PRINT` file, then tap `Import`
    1. Run `Install OpenCentauri (Online)`
    1. Restart when the install process finished

=== "Local/Offline"

    1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release
    1. Insert a fat32-formatted thumbdrive into your pc
    1. Extract the `install_opencentauri` folder from `oc-installer.zip` onto the root of the thumbdrive
    1. [Download update.swu](https://github.com/OpenCentauri/cc-fw-tools/releases/latest/download/update.swu) and place it inside the `install_opencentauri` folder on your thumbdrive
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon
    1. On your Centauri Carbon, navigate to the files tab, then tap the `Usb Drive` menu, then tap the `install_opencentauri` folder
    1. Touch hold the `IMPORT_ME_DO_NOT_PRINT` file, then tap `Import`
    1. Run `Install OpenCentauri (Local)`
    1. Restart when the install process finished

You should now be greeted by the OpenCentauri splash screen :tada:

This firmware works well with [modified machine start and end gcode in OrcaSlicer](./modified_start_end_machine_gcode.md), which centers the purge line and turns on the chamber light at print start. Installing this is optional but recommended.

## Update

You can either accept OTA updates directly on the device or repeat the steps in the [Installation section](#installation).

## Uninstall

Follow the steps again in the [Installation section](#installation). Instead of running `Install OpenCentauri (Online)`, run `Install Official 1.1.40 (Online)`