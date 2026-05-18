# FEL & UART Bench Setup

This guide describes how to set up a Centauri Carbon mainboard on a bench for FEL access/eMMC recovery and UART access to u-Boot and Linux.

!!! info
    This setup is useful for:
    
    - Accessing FEL mode for eMMC recovery
    - Serial console access to u-Boot and Linux
    - Advanced debugging and development work

## Hardware Requirements

You will need the following components:

- **24V Power Supply**
- **3.3V USB Serial UART Dongle** (e.g., [Amazon Link](https://amzn.to/3La06pv))
    - *Alternative:* A Raspberry Pi or ESP32 acting as a serial interface
- **FEL USB Connection** – Choose one of the following options:
    - *Option A:* **Modified USB-A to USB-C Cable** (connects to board's USB-C port) – The VCC (Red) wire must be severed inside the cable, leaving DATA+, DATA-, and Ground intact
    - *Option B:* **USB Power Blocker** (connects to a USB-A to C cable via board's USB-C port) – A USB-A power blocker dongle combined with a normal USB-A to C cable (e.g., [Amazon Link](https://amzn.to/49mVp3F))
    - *Option C:* **Soldered J5 USB Header** (requires soldering) – Solder headers to the unpopulated J5 connector and use a USB-A male to dupont female cable
- **Dupont Jumper Cables** ([Link 1](https://amzn.to/44RJQAe), [Link 2](https://amzn.to/3NsB5GE))

## Critical Warnings

!!! danger "VOLTAGE DANGER"
    The CC mainboard outputs **24V** over the USB-C VCC wire. If you use a standard USB cable without a power blocker or modification, **you will destroy whatever device you plug it into.**

!!! warning "Safety Precautions"
    - **Ground Continuity:** Ensure continuity of Ground between all peripherals
    - **Loose Wires:** If a 24V power/ground wire comes loose, it can cause power to flow across the UART, which **will destroy one or more board chips**
    - **Power Sequencing:** **Do not** plug or unplug components (even the USB) while anything is powered up. Insert the USB and USB-UART connections while the board is **powered down**

## Step-by-Step Hookup

!!! important
    Complete all steps with the power supply **disconnected**. Only apply power after all connections are securely in place.

### 1. Power Connection

Connect the **24V VCC** and **Ground** wires to your external power supply.

See the [mainboard pinout](mainboard.md#24v-input) for the exact pin locations.

![24V Hookup](assets/24V-HOOKUP.jpg)

### 2. UART Connection

Connect the **3.3V Serial UART Tx, Rx, and Ground** between the CC mainboard UART headers and your serial interface (USB Dongle, Pi, etc.).

**Pin Connections:**

- TX (Transmit from board) → RX on your serial adapter
- RX (Receive to board) → TX on your serial adapter  
- GND → GND

!!! important
    Do not connect the VCC pin on the Serial UART

See the [mainboard UART0 pinout](mainboard.md#uart0) for the exact pin locations.

![CC Board UART](assets/CC-Board-UART.jpg)

![USB-UART Adapter](assets/USB-UART.jpg)

### 3. FEL / USB Connection

There are two methods to connect to the board for FEL mode access. Choose the method that works best for your setup.

#### Method A/B: USB-C Cable Connection (Recommended for Temporary Setup)

Connect a modified USB cable (or cable + blocker) from the USB-C port to your host computer.

**Requirements:**

- Ensure VCC is severed/blocked
- Ensure Data+, Data-, and Ground are intact

![USB-C Cable Mod](assets/USBC-CABLE-Mod.jpg)

#### Method C: Soldered USB Header (Recommended for Permanent Bench Setup)

!!! note
    This method requires soldering headers to the unpopulated J5 USB connector on the board.

**Step 1: Solder the unpopulated USB header J5**

![J5 Location](../../software/assets/J5circled.jpg){ width="400" }

**Step 2: Connect to your computer**

Connect the mainboard to your PC using a USB-A male to dupont female cable.
The pinout of the dupont connectors is as follows:

| Pin NR | Marking | Function | Remarks |
|--------|---------|----------|---------|
| 1 | 5V | 5V | Closest to the type-C connector |
| 2 | DP | DP | DP and DM are switched compared to standard USB pinout |
| 3 | DM | DM | |
| 4 | GND | GND | |

!!! warning
    Note that DP and DM are switched compared to standard USB pinout.

---

For instructions on entering FEL mode and installing drivers, see the [FEL mode software guide](../../software/FEL-mode.md).

### 4. Full Setup

The complete system when connected should look like this:

![Full System Connected](assets/USBCFEL-plus-UART.jpg)

## Related Documentation

- [Mainboard Pinout](mainboard.md) - Detailed mainboard pin information
- [FEL Mode](../../software/FEL-mode.md) - How to boot into FEL mode
- [eMMC Tap Points](emmc-tapping.md) - eMMC recovery via tap points
