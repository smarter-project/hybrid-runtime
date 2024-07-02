#!/bin/sh -fxv


echo "Resizing root filesystem partition"

DISK=/dev/mmcblk2
PDATA=$(fdisk -l $DISK | grep Linux)
PART_NUM=$(echo $PDATA | sed -e 's/^\/dev\/mmcblk2p\(.\).*/\1/')
PART_OFFSET=$(echo $PDATA | sed -e 's/^\/dev\/mmcblk2p\(.\)\s\s*\([0-9][0-9]*\).*/\2/' )
FREESPACE=$(sfdisk --list-free $DISK)
echo ", + " | sfdisk -N $PART_NUM --force $DISK
partx -u --nr :-1 $DISK
resize2fs ${DISK}p${PART_NUM}


