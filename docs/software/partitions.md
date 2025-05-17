The Centauri Carbon uses an A/B partition scheme for updates. When an update is performed, it will be written to the inactive partition and will be marked as active, so the next boot will use this previously unused partition. The current A/B slot is stored in the `env` partition.

The Centauri Carbon contains the following partitions:

Partition|Size|Offset|Description
---|---|---|---
boot-resource@mmcblk0p1|0x627000|0x143F000|Contains boot logos (Strangely also one from anycubic)
env@mmcblk0p2|0x3F000|0x1A66000|Contains bootup parameters and endpoints
env-redund@mmcblk0p3|0x3F000|0x1AA5000|
bootA@mmcblk0p4|0x627000|0x1AE4000|
rootfsA@mmcblk0p5|0x8017800|0x210B000|Operating system, slot A
dsp0A@mmcblk0p6|0xFC000|0xA122800|
bootB@mmcblk0p7|0x627000|0xA21E800|
rootfsB@mmcblk0p8|0x8017800|0xA845800|Operating system, slot B
dsp0B@mmcblk0p9|0xFC000|0x187BB100|
rootfs_data@mmcblk0p10|0x8017800|0x12959000|
user@mmcblk0p11|0x8017800|0x1A970800|Seemingly the storage location for logging
private@mmcblk0p12|0xFC000|0x22988000|Entirely empty
UDISK@mmcblk0p13|0x1A5577E00|0x22A84000|