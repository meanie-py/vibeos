/*
 * PROJECT:     Freeldr UEFI Extension
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     Uefi freeldr core header
 * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
 */

#pragma once

/* INCLUDES ******************************************************************/
#include <freeldr.h>

/* UEFI Headers */
#include <Uefi.h>
#include <DevicePath.h>
#include <LoadedImage.h>
#include <GraphicsOutput.h>
#include <UgaDraw.h>
#include <AppleGraphInfo.h>
#include <BlockIo.h>
#include <Acpi.h>
#include <GlobalVariable.h>
#include <debug.h>
#include <machuefi.h>

/*
 * BIOS/ACPI memory structures for ARM64 UEFI
 * These are normally defined in pcbios.h for PC architectures.
 * ARM64 uses UEFI/ACPI directly, but we need compatible structures
 * to pass ACPI information to the OS loader.
 */
#if defined(_M_ARM64) && !defined(BIOS_MEMORY_MAP_DEFINED)
#define BIOS_MEMORY_MAP_DEFINED

#include <pshpack1.h>

/* Memory map entry structure (compatible with E820 format) */
typedef struct
{
    ULONGLONG BaseAddress;
    ULONGLONG Length;
    ULONG Type;
    union
    {
        ULONG ExtendedAttributesAsULONG;
        struct
        {
            ULONG Enabled:1;
            ULONG NonVolatile:1;
            ULONG SlowAccess:1;
            ULONG ErrorLog:1;
            ULONG Reserved:28;
        } ExtendedAttributes;
    };
} BIOS_MEMORY_MAP, *PBIOS_MEMORY_MAP;

/* ACPI BIOS data structure for passing RSDP information */
typedef struct _ACPI_BIOS_DATA
{
    PHYSICAL_ADDRESS RSDTAddress;
    ULONGLONG Count;
    BIOS_MEMORY_MAP MemoryMap[1]; /* Count of memory map entries */
} ACPI_BIOS_DATA, *PACPI_BIOS_DATA;

#include <poppack.h>

#endif /* _M_ARM64 && !BIOS_MEMORY_MAP_DEFINED */

/* Hardware resource allocation tag */
#ifndef TAG_HW_RESOURCE_LIST
#define TAG_HW_RESOURCE_LIST 'lRwH'
#endif

//TODO: this version of the struct is temporary
typedef struct _REACTOS_INTERNAL_BGCONTEXT
{
    ULONG_PTR    BaseAddress;
    ULONG        BufferSize;
    UINT32       ScreenWidth;
    UINT32       ScreenHeight;
    UINT32       PixelsPerScanLine;
    UINT32       PixelFormat;
} REACTOS_INTERNAL_BGCONTEXT, *PREACTOS_INTERNAL_BGCONTEXT;

VOID __cdecl BootMain(IN PCCH CmdLine);
