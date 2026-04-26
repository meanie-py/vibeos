/*
 * PROJECT:         ReactOS Boot Loader
 * LICENSE:         GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * FILE:            boot/freeldr/freeldr/include/arch/arm64/hardware.h
 * PURPOSE:         Hardware definitions for ARM64 UEFI boot
 * COPYRIGHT:       Copyright 2024 ReactOS ARM64 Port
 */

#pragma once

/* ARM64 uses UEFI, so many BIOS-specific functions are stubs */

/* BIOS drive mapping function - provided by arch/uefi/stubs.c */
UCHAR DriveMapGetBiosDriveNumber(PCSTR DeviceName);

/* Reboot is implemented in arch/uefi/uefildr.c */
DECLSPEC_NORETURN
VOID __cdecl Reboot(VOID);

/* Memory configuration for ARM64 freeldr */
#define FREELDR_BASE        0x00100000
#define FREELDR_PE_BASE     0x00100000
#define MAX_FREELDR_PE_SIZE 0x1000000

/* Disk read buffer - defined in arch/uefi/uefidisk.c for UEFI builds */
/* No need to define here as disk.h has the extern declaration */

/* Boot drive/partition - defined in arch/uefi/uefidisk.c */
extern UCHAR FrldrBootDrive;
extern ULONG FrldrBootPartition;
