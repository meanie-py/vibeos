/*
 * PROJECT:     ReactOS KMDF: driver initialization static library
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     Main file
 * COPYRIGHT:   Copyright 2021 Max Korostil <mrmks04@yandex.ru>
 */

#include <ntddk.h>
#include <windef.h>
#include <fxldr.h>
#include "wdf.h"


#define WDFENTRY_TAG 'EFDW'

// supplied by the driver this library is linked into
extern
NTSTATUS
NTAPI
DriverEntry(
    _In_ PDRIVER_OBJECT DriverObject,
    _In_ PUNICODE_STRING RegistryPath);

// The Rust kmdf-sys dispatch (sdk/lib/rust/kmdf-sys) reads the framework
// function table through the versioned symbol `WdfFunctions_01017` (the real
// KMDF wdf.h convention). WdfVersionBind writes the bound table pointer to
// whatever BindInfo.FuncTable points at, so this global MUST be the exact same
// symbol the Rust dispatch dereferences — otherwise every Wdf* call indexes a
// NULL table (BUGCHECK 0x1E at the first WdfDriverCreate).
const WDFFUNC *WdfFunctions_01017;
PWDF_DRIVER_GLOBALS WdfDriverGlobals;
WDF_BIND_INFO BindInfo =
{
    .Size = sizeof(BindInfo),
    .Component = L"KmdfLibrary",
    .Version.Major = __WDF_MAJOR_VERSION,
    .Version.Minor = __WDF_MINOR_VERSION,
    .Version.Build = __WDF_BUILD_NUMBER,
    .FuncCount = WdfFunctionTableNumEntries,
    .FuncTable = (WDFFUNC *)&WdfFunctions_01017
};
PDRIVER_UNLOAD pOriginalUnload = NULL;
UNICODE_STRING gRegistryPath;

static
VOID
FxDriverUnloadCommon(VOID)
{
    WdfVersionUnbind(&gRegistryPath, &BindInfo, (PWDF_COMPONENT_GLOBALS)WdfDriverGlobals);
}

VOID
NTAPI
FxDriverUnload(
    _In_ PDRIVER_OBJECT DriverObject)
{
    if (pOriginalUnload != NULL)
    {
        pOriginalUnload(DriverObject);
    }
    FxDriverUnloadCommon();
}

NTSTATUS
NTAPI
FxDriverEntry(
    _In_ PDRIVER_OBJECT DriverObject,
    _In_ PUNICODE_STRING RegistryPath)
{
    NTSTATUS status;


    if (DriverObject == NULL)
    {
        return DriverEntry(DriverObject, RegistryPath);
    }

    // Copy registry path
    gRegistryPath.MaximumLength = RegistryPath->Length + sizeof(UNICODE_NULL);
    gRegistryPath.Buffer = ExAllocatePoolWithTag(PagedPool,
                                                 gRegistryPath.MaximumLength,
                                                 WDFENTRY_TAG);

    if (gRegistryPath.Buffer == NULL)
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(&gRegistryPath, RegistryPath);

    // Bind wdf driver to framework
    status = WdfVersionBind(DriverObject,
                            RegistryPath,
                            &BindInfo,
                            (PWDF_COMPONENT_GLOBALS*)(&WdfDriverGlobals));


    if (!NT_SUCCESS(status))
    {
        return status;
    }

    // Call original entry point
    status = DriverEntry(DriverObject, RegistryPath);
    if (!NT_SUCCESS(status))
    {
        FxDriverUnloadCommon();
        return status;
    }

    if (WdfDriverGlobals->DisplaceDriverUnload)
    {
        pOriginalUnload = DriverObject->DriverUnload;
        DriverObject->DriverUnload = FxDriverUnload;
    }
    return STATUS_SUCCESS;
}
