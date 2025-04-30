# Partition layout EMMC

!!! question "Speculation"

    The Centauri Carbon seems to make use of an A/B partition scheme, 
    likely to make updates while booted into the OS consistent.

    After a firmware update, i suspect the other partition gets selected for a seamless transition.

    Elegoo seemingly has not provided a way to go back to an older version of the OS, though.

The coredump contains the following on partitions:

- boot-resource@mmcblk0p1
- env@mmcblk0p2:env-redund@mmcblk0p3
- bootA@mmcblk0p4
- rootfsA@mmcblk0p5
- dsp0A@mmcblk0p6
- bootB@mmcblk0p7
- rootfsB@mmcblk0p8
- dsp0B@mmcblk0p9
- rootfs_data@mmcblk0p10
- user@mmcblk0p11
- private@mmcblk0p12
- UDISK@mmcblk0p13