This page contains misc information about some of the custom GCODE commands obtained by reverse engineering

## M8803

Executes the following 3 system commands:

- `cp /board-resource/printer.cfg /mnt/exUDISK/printer.cfg`
- `cp /board-resource/user_printer.cfg /mnt/exUDISK/user_printer.cfg`
- `cp /board-resource/unmodifiable.cfg /mnt/exUDISK/unmodifiable.cfg`

(Note: exUDISK is the USB Stick)

## M8807

Executes the following 2 system commands:

- `cp /mnt/exUDISK/printer.cfg /board-resource/printer.cfg`
- `cp /mnt/exUDISK/user_printer.cfg /board-resource/user_printer.cfg`

(Note: exUDISK is the USB Stick)

WARNING!! Invalid configs cause the printer to not boot!