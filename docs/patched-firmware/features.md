# Patched Firmware Features

Features as of v0.3.0:

- SSH (user: "root", password: "OpenCentauri")
- Developer features (package manager, prevent booting Klipper, etc)
- Bootlogo can be replaced
    - [See available bootlogos on github](https://github.com/OpenCentauri/cc-fw-tools/tree/main/oc-patches/replace-bootlogo-patch){target="_blank"}
- Exhaust fan no longer automatically turns on during a print
    - Note: In OrcaSlicer the default profile still turns on this fan using a M106 P3 command when using PLA.
    - Note: On Elegoo filaments present in OrcaSlicer, the `Exhaust fan` section present on the Cooling tab inside of a Filament profile is respected. If it is enabled, then exhaust fan will turn on at the specified speed.
- Homing position has been set to the front right instead of the front left, to prevent wear on the USB toolhead cable
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
