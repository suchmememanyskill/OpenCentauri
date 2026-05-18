# Install COSMOS

!!! Warning "Beta Software!"
    COSMOS is still under heavy development. Don't install this firmware if you actually depend on using your printer. Functionality is not set in stone and may change at any time.

!!! Danger "**Stop: Before you add any plugins**"

    The stock mainboard is extremely resource limited and there is currently very little overhead to run any plugins, packages, or features not in standard klipper/kalico. Do not attempt to install others unless you _really_ know what you are doing!

## Installation
*COSMOS is not considered stable yet. When it is stable, it will be added to the OpenCentauri Installer*

!!! Note
    As COSMOS is still under heavy development, it is important to read [the changelogs](https://github.com/OpenCentauri/cosmos/releases) on GitHub before installing a new version. It may contain additional instructions for migration from an old version.

The process for installing COSMOS is pretty much the same as for installing patched-OC firmware.

=== "Local/Offline"

    1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release.
    1. Insert a fat32-formatted thumbdrive into your pc.
    1. Extract the install_opencentauri folder from oc-installer.zip onto the root of the thumbdrive
    1. [Download the latest COSMOS update.swu](https://github.com/OpenCentauri/cosmos/releases/latest/download/update.swu) and place it inside the install_opencentauri folder on your thumbdrive.
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon.
    1. On your Centauri Carbon, navigate to the files tab, then tap the Usb Drive menu, then tap the install_opencentauri folder.
    1. Touch hold the IMPORT_ME_DO_NOT_PRINT file, then tap Import.
    1. Run Install OpenCentauri (Local).
    1. Restart when the install process finished.
    1. Remove the thumbdrive.

## Update

=== "Via Screen (Online)"

    1. Navigate to the settings tab on the printer's screen.
    1. Click `Update COSMOS`.
    1. Wait for the process to complete.

=== "Via SSH (Online)"

    1. Log into your printer via SSH.
    1. Run the command `update-cosmos`
    1. Wait for the process to complete.

=== "Via USB (Offline)"

    1. [Download the latest COSMOS update.swu](https://github.com/OpenCentauri/cosmos/releases/latest/download/update.swu).
    1. Rename it to `emergency.swu`
    1. Insert a fat32-formatted thumbdrive into your pc.
    1. Put `emergency.swu` on the root of the thumbdrive.
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon.
    1. Wait for the update process to complete. This should be indicated by an on-screen UI prompt (only appears if Klipper is loaded)

=== "Via SSH (Manual, Offline)"

    You can also flash an arbitrary .swu via SSH. Just place the .swu somewhere accessible to the machine (like in /user-resource via SCP/SSH), SSH into your machine, then run `flash /path/to/.swu`.

!!! Note "After Setup"
    After setup, connect your printer to the internet via the on screen UI. Connect via a web browser to your printer's IP, this will load up the web interface of the printer. Under the Macros section, run `FULL CALIBRATION` to run through basic machine calibrations (resonance compensation, bed mesh, extruder pid calibration).

    If you are using OrcaSlicer, don't forget to change the connectivity settings. New settings are:

    - Host Type: Octo/Klipper
    - Hostname, IP or URL: ip.of.your.printer

## Uninstall

!!! Warning
    Please do not try to flash an official .swu or OpenCentauri patched .swu directly. This will skip downgrading the toolhead/bed and will cause a brick!

=== "Via Screen (Online)"

    1. Navigate to the settings tab on the printer's screen.
    1. Click `Switch to OC Patched`.
    1. Wait for the process to complete.

=== "Via SSH (Online)"

    1. Log into your printer via SSH.
    1. Run the command `switch-to-oc-patched`
    1. Wait for the process to complete.