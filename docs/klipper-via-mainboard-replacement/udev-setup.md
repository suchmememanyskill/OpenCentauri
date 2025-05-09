The CC hotend, for some reason boots up a generic ACM device:

Extract from `dmesg`
```
[   89.084013] usb 1-1.2: USB disconnect, device number 3
[   89.313505] usb 1-1.2: new full-speed USB device number 4 using xhci_hcd
[   89.423364] usb 1-1.2: New USB device found, idVendor=1d50, idProduct=614e, bcdDevice= 2.00
[   89.423379] usb 1-1.2: New USB device strings: Mfr=1, Product=2, SerialNumber=3
[   89.423383] usb 1-1.2: Product: STM32 Virtual ComPort
[   89.423386] usb 1-1.2: Manufacturer: ShenZhenCBD
[   89.423390] usb 1-1.2: SerialNumber: 3664356E3233
```

But after ~10-20 second of idle, it shuts down and resets as an STM32 Klipper Serial Device:

Extract from `dmesg`
```
[  126.721014] usb 1-1.2: USB disconnect, device number 4
[  126.948951] usb 1-1.2: new full-speed USB device number 5 using xhci_hcd
[  127.060969] usb 1-1.2: New USB device found, idVendor=1d50, idProduct=614e, bcdDevice= 1.00
[  127.060984] usb 1-1.2: New USB device strings: Mfr=1, Product=2, SerialNumber=3
[  127.060988] usb 1-1.2: Product: stm32f401xc
[  127.060992] usb 1-1.2: Manufacturer: Klipper
[  127.060995] usb 1-1.2: SerialNumber: 39002C000B51333235353836
[  127.064802] cdc_acm 1-1.2:1.0: ttyACM1: USB ACM device
```

During this process, first it's assinged the device id `ttyACM0` (when no other tty devices are present) but after reset, it shows up as `ttyACM1`.

To have a constant simlink to the device, we can set up an udev rule for the device using the vendor and device id.

1. Go to `/etc/udev/rules.d`
2. Open a text editor with a **new** file with `.rules` extension, **do not edit the existing file**, e.g.: `sudo nano cc-hotend-device.rules`
3. Paste `KERNEL=="ttyACM[0-9]*", SUBSYSTEM=="tty", ATTRS{idVendor}=="1d50", ATTRS{idProduct}=="614e", SYMLINK="ttyHotend"`

To figure out your specific vendor id and product id, disconnect and connect the hotend from the Host, then type `dmesg`. Refer to the above log extracts to see what
is expected.