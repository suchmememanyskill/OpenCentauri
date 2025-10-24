# Patched firmware

This section describes the installation process and features of OpenCentauri's patched firmware.

This firmware is based on Elegoo's 1.1.40 firmware, with patches applied to patch out unwanted behaviour and bugs.

If you find any bugs with this firmware, or want to suggest new features, please do so on the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools){target="_blank"} repository. Not everything is possible within the limits of patching existing firmware, specifically large features or overhauls are unattainable.

This project is in a beta state currently, expect some issues!

If you need any support, feel free to [join the discord](https://discord.gg/t6Cft3wNJ3)

If you'd like to support our work, you can do so on [Ko-Fi](https://ko-fi.com/opencentauri) :heart:!

## Features (as of v0.1.0)

- SSH (user: "root", password: "OpenCentauri")
- Developer features (package manager, prevent booting Klipper, etc)
- Bootlogo can be replaced
    - [See available bootlogos on github](https://github.com/OpenCentauri/cc-fw-tools/tree/main/oc-patches/replace-bootlogo-patch){target="_blank"}
- Exhaust fan no longer automatically turns on during a print
    - Note: In OrcaSlicer the default profile still turns on this fan using a M106 P3 command when using PLA.
    - Note: On Elegoo filaments present in OrcaSlicer, the `Exhaust fan` section present on the Cooling tab inside of a Filament profile is respected. If it is enabled, then exhaust fan will turn on at the specified speed.
- Homing position has been set to the front right instead of the front left
- The webui (and other integrations like octoeverywhere and home assistant) now accepts modifications during a print (bug introduced in firmware 1.1.29)
- Connectivity checks (web traffic issue) have been blocked
- OTA updates have been blocked
- New gcode commands: 
    - M8212 to turn off the chamber light
    - M8213 to turn on the chamber light

## Installation

1. Download the centauri_carbon_developer_mode executable and run it on a computer on the same network as your Centauri Carbon. Follow the on screen instructions to enable developer mode.
    - [Windows build (x64)](https://drive.google.com/file/d/1CROOzsOPZa0S_523WJcTDxCNBs5pvNRz/view?usp=sharing){target="_blank"}
    - [MacOs build (Universal)](https://drive.google.com/file/d/1N6l0DHo1PaB8TD3hzHAWicqE6ILId-LG/view?usp=sharing){target="_blank"}
        - To run this executable, open the Terminal in the same folder as the centauri_carbon_developer_mode file, then run the following commands:
        - `chmod +x centauri_carbon_developer_mode`
        - `./centauri_carbon_developer_mode`
    - [Linux build (x64)](https://drive.google.com/file/d/1hPIMx2H8KXDDGo888rHW8m7f7IMhWHur/view?usp=sharing){target="_blank"}
    - All executables support being ran headless too: `centauri_carbon_developer_mode.exe [install/uninstall] [ip.of.your.cc]`
2. Insert a fat32-formatted thumbdrive into your pc.
3. Remove `update.bin` if this is present on your thumbdrive.
4. Create an `update` folder on the thumbdrive if it does not exist already.
5. Download `update.swu` from [the latest OC firmware](https://github.com/OpenCentauri/cc-fw-tools/releases){target="_blank"}, and copy it into the `update` folder on your thumbdrive.
6. Eject your thumbdrive from your pc and insert it into your Centauri Carbon.
7. On your Centauri Carbon, navigate to settings, then to `Check for updates`. There should be a red dot to the right of `Check for updates`.
8. Click on `new version detected`. Check if the update prompt says the following: `Update local FW`. If it does, click on Update.
9. Remove the thumbdrive after the machine reboots.

You should now be greeted by the OpenCentauri splash screen :tada:

This new firmware goes well with [modified machine start and end gcode in OrcaSlicer](./modified_start_end_machine_gcode.md). Specifically to center the purge line and to turn on the light of the chamber after starting a print. Installing this is optional but reccomended.

## Update

OpenCentauri builds already have developer mode enabled by default. You can install a newer version of opencentauri by following the instructions from [Installation](#installation) from step 2 onwards.

## Uninstall

1. Run the centauri_carbon_developer_mode executable again, this time selecting to disable developer mode.
2. Follow the local firmware update instructions on the [Updates page](../software/updates.md#updating-locally-via-usb).
