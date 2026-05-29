/*
 * PROJECT:     FreeLoader UEFI Support
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     Entry point and helpers
 * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
 */

#include <uefildr.h>

#include <debug.h>
DBG_DEFAULT_CHANNEL(WARNING);

/* GLOBALS ********************************************************************/

EFI_HANDLE GlobalImageHandle;
EFI_SYSTEM_TABLE *GlobalSystemTable;
PVOID UefiServiceStack;
PVOID BasicStack;

void _changestack(VOID);

/* FUNCTIONS ******************************************************************/

EFI_STATUS
EfiEntry(
    _In_ EFI_HANDLE ImageHandle,
    _In_ EFI_SYSTEM_TABLE *SystemTable)
{
    PCSTR CmdLine = ""; // FIXME: Determine a command-line from UEFI boot options

    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"UEFI EntryPoint: Starting freeldr from UEFI\r\n");
    GlobalImageHandle = ImageHandle;
    GlobalSystemTable = SystemTable;

    /* Load the default settings from the command-line */
    LoadSettings(CmdLine);

    /* Debugger pre-initialization */
    DebugInit(BootMgrInfo.DebugString);

    MachInit(CmdLine);
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] MachInit done\r\n");

    /* UI pre-initialization */
    if (!UiInitialize(FALSE))
    {
        UiMessageBoxCritical("Unable to initialize UI.");
        goto Quit;
    }
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] UI initialized\r\n");

    /* Initialize memory manager */
    if (!MmInitializeMemoryManager())
    {
        UiMessageBoxCritical("Unable to initialize memory manager.");
        goto Quit;
    }
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Memory manager initialized\r\n");

    /* Initialize I/O subsystem */
    FsInit();
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Filesystem initialized\r\n");

    /* Initialize the module list */
    if (!PeLdrInitializeModuleList())
    {
        UiMessageBoxCritical("Unable to initialize module list.");
        goto Quit;
    }
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Module list initialized\r\n");

    if (!MachInitializeBootDevices())
    {
        UiMessageBoxCritical("Error when detecting hardware.");
        goto Quit;
    }
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Boot devices initialized\r\n");

    /* 0x32000 is what UEFI defines, but we can go smaller if we want */
    BasicStack = (PVOID)((ULONG_PTR)0x32000 + (ULONG_PTR)MmAllocateMemoryWithType(0x32000, LoaderOsloaderStack));
    SystemTable->ConOut->OutputString(SystemTable->ConOut, L"[freeldr] Switching stack...\r\n");
    _changestack();

Quit:
    /* If we reach this point, something went wrong before, therefore reboot */
    Reboot();

    UNREACHABLE;
    return 0;
}

DECLSPEC_NORETURN
void
ExecuteLoaderCleanly(PVOID PreviousStack)
{
    TRACE("ExecuteLoaderCleanly Entry\n");
    UefiServiceStack = PreviousStack;

    RunLoader();

    /* RunLoader can return if no boot configuration was found */
    WARN("RunLoader returned, halting\n");
    Reboot();
    UNREACHABLE;
}

#if !defined(_M_ARM)
DECLSPEC_NORETURN
VOID __cdecl Reboot(VOID)
{
    //TODO: Replace with a true firmware reboot eventually
    WARN("Something has gone wrong - halting FreeLoader\n");
    for (;;)
    {
        NOTHING;
    }
}
#endif
