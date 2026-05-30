/*
 * VideoPort driver — INT10 stubs
 *
 * The legacy INT10 / VGA-BIOS emulation path (HAL x86Bios* via the fast486
 * software CPU, plus the i386 V86-mode fallback) has been removed. It was
 * pre-Vista platform support, dead under UEFI: the display is the GOP linear
 * framebuffer (arm64vid.sys -> framebuf.dll), and the only INT10 consumers
 * (the vga/vbe video miniports) have been dropped.
 *
 * The generic videoprt driver is kept, so these minimal stubs remain to
 * satisfy the dispatch open path (IntInitializeInt10), the VideoPortInt10
 * export, and the Int10 service interface wired up in services.c. They take
 * no dependency on the HAL x86Bios* exports; every entry reports that INT10
 * is unsupported.
 */

#include "videoprt.h"

#include <ndk/kefuncs.h>

#define NDEBUG
#include <debug.h>

/* GLOBAL VARIABLES ***********************************************************/

#ifdef _M_IX86
/* No x86 emulator anymore. */
BOOLEAN VideoPortDisableX86Emulator = TRUE;
#endif

KMUTEX VideoPortInt10Mutex;

/* PUBLIC FUNCTIONS ***********************************************************/

NTSTATUS
IntInitializeInt10(VOID)
{
    /* INT10 BIOS emulation removed — nothing to initialize. The mutex is kept
     * initialized in case any miniport still queries the Int10 interface. */
    KeInitializeMutex(&VideoPortInt10Mutex, 0);
    return STATUS_SUCCESS;
}

VP_STATUS
NTAPI
IntInt10AllocateBuffer(
    _In_ PVOID Context,
    _Out_ PUSHORT Seg,
    _Out_ PUSHORT Off,
    _Inout_ PULONG Length)
{
    UNREFERENCED_PARAMETER(Context);
    UNREFERENCED_PARAMETER(Seg);
    UNREFERENCED_PARAMETER(Off);
    UNREFERENCED_PARAMETER(Length);
    return ERROR_INVALID_FUNCTION;
}

VP_STATUS
NTAPI
IntInt10FreeBuffer(
    _In_ PVOID Context,
    _In_ USHORT Seg,
    _In_ USHORT Off)
{
    UNREFERENCED_PARAMETER(Context);
    UNREFERENCED_PARAMETER(Seg);
    UNREFERENCED_PARAMETER(Off);
    return ERROR_INVALID_FUNCTION;
}

VP_STATUS
NTAPI
IntInt10ReadMemory(
    _In_ PVOID Context,
    _In_ USHORT Seg,
    _In_ USHORT Off,
    _Out_writes_bytes_(Length) PVOID Buffer,
    _In_ ULONG Length)
{
    UNREFERENCED_PARAMETER(Context);
    UNREFERENCED_PARAMETER(Seg);
    UNREFERENCED_PARAMETER(Off);
    UNREFERENCED_PARAMETER(Buffer);
    UNREFERENCED_PARAMETER(Length);
    return ERROR_INVALID_FUNCTION;
}

VP_STATUS
NTAPI
IntInt10WriteMemory(
    _In_ PVOID Context,
    _In_ USHORT Seg,
    _In_ USHORT Off,
    _In_reads_bytes_(Length) PVOID Buffer,
    _In_ ULONG Length)
{
    UNREFERENCED_PARAMETER(Context);
    UNREFERENCED_PARAMETER(Seg);
    UNREFERENCED_PARAMETER(Off);
    UNREFERENCED_PARAMETER(Buffer);
    UNREFERENCED_PARAMETER(Length);
    return ERROR_INVALID_FUNCTION;
}

VP_STATUS
NTAPI
IntInt10CallBios(
    _In_ PVOID Context,
    _Inout_ PINT10_BIOS_ARGUMENTS BiosArguments)
{
    UNREFERENCED_PARAMETER(Context);
    UNREFERENCED_PARAMETER(BiosArguments);
    return ERROR_INVALID_FUNCTION;
}

VP_STATUS
NTAPI
VideoPortInt10(
    IN PVOID HwDeviceExtension,
    IN PVIDEO_X86_BIOS_ARGUMENTS BiosArguments)
{
    UNREFERENCED_PARAMETER(HwDeviceExtension);
    UNREFERENCED_PARAMETER(BiosArguments);
    return ERROR_INVALID_FUNCTION;
}
