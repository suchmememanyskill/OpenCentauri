# Install Patched Firmware

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

You can either accept OTA updates directly on the device or repeat the steps in the [Installation section](#install-patched-firmware).

## Uninstall

Follow the steps again in the [Installation section](#install-patched-firmware). Instead of running `Install OpenCentauri (Online)`, run `Install Official 1.1.40 (Online)`
