/*
 * io_rust_shims.c — C functions not yet translated to Rust for the IO subsystem.
 *
 * IopInitializeDriverModule is a complex function that was not translated in
 * the IO Tier 1 batch. It remains in C and links against both C and Rust symbols.
 */

#include <ntoskrnl.h>
#include <ntstrsafe.h>
#define NDEBUG
#include <debug.h>
#include "internal/io_i.h"

/* ========================================================================
 * Extern declarations for Rust-defined symbols used by IopInitializeDriverModule
 * ======================================================================== */

extern UNICODE_STRING IopHardwareDatabaseKey;
extern BOOLEAN PnpSystemInit;
extern BOOLEAN PnPBootDriversLoaded;

NTSTATUS NTAPI IopInvalidDeviceRequest(PDEVICE_OBJECT DeviceObject, PIRP Irp);
VOID NTAPI IopReadyDeviceObjects(PDRIVER_OBJECT Driver);
VOID NTAPI IopReinitializeDrivers(VOID);

/* ========================================================================
 * FORCEINLINE/macro shims
 * ======================================================================== */

/*
 * HalInitPnpDriver — macro in haltypes.h that calls HALDISPATCH->HalInitPnpDriver
 * Rust can't expand this macro, needs a C shim.
 */
NTSTATUS
HalInitPnpDriver_shim(VOID)
{
    return HalInitPnpDriver();
}

/*
 * io-inlines FORCEINLINE shims (IopValidateOpenPacket, IoGetDevObjExtension,
 * IopUnlockFileObject, IopUpdateTransferCount, IopLockFileObject,
 * IopUpdateOperationCount) — REMOVED. The io-inlines rlib
 * (sdk/lib/rust/io-inlines/src/lib.rs) now exports the `X_shim` symbols
 * directly with the right ABI (`extern "system"` = NTAPI), so the C
 * trampolines that delegated to `_rs_X` are pure dead weight.
 */

/*
 * ObReferenceObject — C macro that calls ObfReferenceObject (FASTCALL).
 * On i386, Rust extern "C" generates cdecl symbol; needs wrapper.
 */
#undef ObReferenceObject
VOID
NTAPI
ObReferenceObject(IN PVOID Object)
{
    ObfReferenceObject(Object);
}

/*
 * ObDereferenceObject — C macro that calls ObfDereferenceObject (FASTCALL).
 */
#undef ObDereferenceObject
VOID
NTAPI
ObDereferenceObject(IN PVOID Object)
{
    ObfDereferenceObject(Object);
}

/*
 * ExRaiseStatus — C macro that expands to RtlRaiseStatus.
 * Rust extern "system" needs stdcall (NTAPI) wrapper.
 */
#undef ExRaiseStatus
VOID
NTAPI
ExRaiseStatus(IN NTSTATUS Status)
{
    RtlRaiseStatus(Status);
}

/*
 * Probe helper macros from probe.h — Rust extern "system" needs function wrappers.
 */
#undef ProbeForWriteHandle
VOID
NTAPI
ProbeForWriteHandle(IN PHANDLE Ptr)
{
    ProbeForWrite(Ptr, sizeof(HANDLE), sizeof(HANDLE));
}

#undef ProbeForWriteIoStatusBlock
VOID
NTAPI
ProbeForWriteIoStatusBlock(IN PIO_STATUS_BLOCK Ptr)
{
    ProbeForWrite(Ptr, sizeof(IO_STATUS_BLOCK), sizeof(ULONG));
}

#undef ProbeForReadLargeInteger
LARGE_INTEGER
NTAPI
ProbeForReadLargeInteger(IN const LARGE_INTEGER *Ptr)
{
    LARGE_INTEGER Result;
    if ((ULONG_PTR)Ptr + sizeof(LARGE_INTEGER) < (ULONG_PTR)Ptr ||
        (ULONG_PTR)Ptr + sizeof(LARGE_INTEGER) > (ULONG_PTR)MmUserProbeAddress) {
        ExRaiseAccessViolation();
    }
    Result = *(volatile const LARGE_INTEGER *)Ptr;
    return Result;
}

/*
 * RtlUnicodeStringPrintf_4wZ — wrapper for RtlUnicodeStringPrintf with
 * format "%wZ%wZ#%wZ%wZ" and 4 PUNICODE_STRING args.
 * RtlUnicodeStringPrintf is FORCEINLINE from ntstrsafe.h (variadic).
 */
NTSTATUS
NTAPI
RtlUnicodeStringPrintf_4wZ_shim(
    PUNICODE_STRING DestinationString,
    PUNICODE_STRING Arg1,
    PUNICODE_STRING Arg2,
    PUNICODE_STRING Arg3,
    PUNICODE_STRING Arg4)
{
    return RtlUnicodeStringPrintf(DestinationString,
                                  L"%wZ%wZ#%wZ%wZ",
                                  Arg1, Arg2, Arg3, Arg4);
}

/*
 * IopQueueIrpToThread / IopUnQueueIrpFromThread / IoCopyCurrentIrpStackLocationToNext
 * — REMOVED. io-inlines rlib exports the `_shim`-suffixed symbols directly
 * (NTAPI for the IRP-thread ones, cdecl for IoCopyCurrentIrpStackLocationToNext
 * which Rust callers in rawfs.rs declare via `extern "C"`).
 */


/*
 * RtlAnsiStringToUnicodeString_shim / RtlUnicodeStringToAnsiString_shim /
 * RtlInitAnsiString_shim / RtlFreeUnicodeString_shim removed (Phase 3) —
 * Rust callers in `iomgr.rs` / `arcname.rs` / `fstub/disksup.rs` now
 * declare these as `extern "system"` directly, which generates the
 * right `_Name@N` stdcall decoration on i386 and cdecl on amd64/arm64.
 */

/* ========================================================================
 * Ob*_shim wrappers for device.rs — DELETED 2026-05-06 (Phase 4 re-probe).
 *
 * `ObCreateObjectAddress_shim`, `ObInsertObject_shim`, and
 * `ObReferenceObjectByHandle_shim` repackaged the output object as `ULONG_PTR`
 * to dodge an LLVM cross-CGU captures(none) attribute propagation bug. The
 * workspace `lto = false` + `codegen-units = 256` firewall now keeps the
 * callee body in a different CGU than the caller, so the propagation can't
 * happen — `device.rs` calls `ObCreateObject` / `ObInsertObject` /
 * `ObReferenceObjectByHandle` directly with `*mut PVOID` output, the same
 * shape the rest of the kernel (ex/timer.rs, ex/sem.rs, ex/mutant.rs,
 * ex/evtpair.rs, lpc/{connect,send,complete}.rs) uses. `ObCreateObject_shim`
 * (passthrough trampoline with no callers) was also deleted.
 * ======================================================================== */

/* ========================================================================
 * Debug tracing for Rust pnpmgr (temporary)
 * ======================================================================== */
void __cdecl pnp_trace(const char *msg)
{
    DbgPrint("%s", msg);
}
void __cdecl pnp_trace_ulong(const char *msg, ULONG val)
{
    DbgPrint("%s%lx\n", msg, val);
}


/* ========================================================================
 * IopQueryDeviceCapabilities — DELETED 2026-05-06.
 *
 * Rust impl in `pnpmgr/devaction.rs::IopQueryDeviceCapabilities` replaces
 * the C version. The historical "Rust version causes amd64-specific hang"
 * was a captures(none) ghost (same family as Ob*_shim / IopGetBusTypeGuidIndex
 * / Pi*DeviceAction).
 * ======================================================================== */

/* `IopGetBusTypeGuidIndex_c` deleted 2026-05-06 — Rust impl in
 * `pnpmgr.rs::IopGetBusTypeGuidIndex` replaces it. The historical
 * "amd64-specific hang" was almost certainly the same captures(none)
 * pattern that bit `device.rs`; the workspace `lto = false` +
 * `codegen-units = 256` firewall (commit 2ba8dab4) eliminates it. */

/* ========================================================================
 * IopInitiatePnpIrp_c — DELETED 2026-05-06.
 *
 * Was dead code: the Rust impl in `pnpmgr.rs::IopInitiatePnpIrp` (line 704)
 * already exists and is the linked symbol; this C trampoline had no
 * remaining callers. The historical "Rust version causes an amd64-specific
 * hang" comment turned out to be moot — the Rust version had already been
 * promoted past the trampoline and works fine.
 * ======================================================================== */

/* ========================================================================
 * IopGetRegistryValue / IopOpenRegistryKeyEx / IopCreateRegistryKeyEx /
 * IopLoadDriver — all four `_shim` trampolines DELETED 2026-05-06.
 *
 * Were captures(none) workarounds ("LLVM cross-CGU captures(none) attribute
 * propagation bug" — output pointer writes elided). Re-probed: the workspace
 * `lto = false` + `codegen-units = 256` firewall is sufficient. pnpinit.rs and
 * devaction.rs now declare and call the underlying NTAPI / cdecl Rust impls
 * directly. Same captures(none)-ghost re-probe pattern as the Ob*_shim trio,
 * IopInitializePlugPlayServices_shim, and the entire `_c`-suffixed PnP family
 * eliminated earlier on 2026-05-06. PiInitCacheGroupInformation is the last
 * remaining io captures(none) holdout — see below — and needs its own probe
 * because its in-source comment claims "even with volatile reads" the original
 * Rust version saw the success path elided.
 * ======================================================================== */

/* ========================================================================
 * PiInitCacheGroupInformation_shim — DELETED 2026-05-06.
 *
 * Was a captures(none) workaround documented as "even with volatile reads"
 * needing the entire body in C. Re-probed after the registry+driver shim
 * round above eliminated the underlying IopGetRegistryValue/_OpenRegistryKey
 * shims that this function transitively depended on: the cgu=256 + lto=false
 * workspace firewall is sufficient. pnpinit.rs::PiInitCacheGroupInformation
 * (the Rust impl already living next to the call site, with volatile reads)
 * is now the linked symbol; the call site at pnpinit.rs:385 was switched
 * over from the `_shim` variant. Smoke 7/7 PASS on i386 + amd64.
 * ======================================================================== */

/* ========================================================================
 * PnP manager macro/FORCEINLINE shims for Rust pnpmgr
 * ======================================================================== */

/*
 * IopIsValidPhysicalDeviceObject — macro in io.h.
 * Checks DeviceNode exists and has DNF_ENUMERATED flag.
 * The macro name is #define'd in io.h, so we #undef it and provide a real function.
 */
#undef IopIsValidPhysicalDeviceObject
BOOLEAN
NTAPI
IopIsValidPhysicalDeviceObject(
    IN PDEVICE_OBJECT PhysicalDeviceObject)
{
    PEXTENDED_DEVOBJ_EXTENSION ext;
    if (!PhysicalDeviceObject) return FALSE;
    ext = (PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension;
    if (!ext) return FALSE;
    if (!ext->DeviceNode) return FALSE;
    return (ext->DeviceNode->Flags & DNF_ENUMERATED) ? TRUE : FALSE;
}

/*
 * HalGetDmaAdapter — macro in haltypes.h: HALDISPATCH->HalGetDmaAdapter.
 * Rust pnpdma.rs declares extern "system" fn HalGetDmaAdapter(...).
 * We save the dispatch pointer, #undef the macro, and provide a real function.
 */
#undef HalGetDmaAdapter
PDMA_ADAPTER
NTAPI
HalGetDmaAdapter(
    IN PDEVICE_OBJECT PhysicalDeviceObject,
    IN PDEVICE_DESCRIPTION DeviceDescription,
    IN OUT PULONG NumberOfMapRegisters)
{
    return HALDISPATCH->HalGetDmaAdapter(PhysicalDeviceObject, DeviceDescription, NumberOfMapRegisters);
}

/*
 * CmiGetNextResourceDescriptor — FORCEINLINE in cmreslist.h.
 * Bindgen declares it as extern "C" but there's no real symbol since it's inline.
 * Provide a non-inline definition. We implement it directly to avoid
 * conflicting with the FORCEINLINE declaration in the header.
 */
PCM_FULL_RESOURCE_DESCRIPTOR
CmiGetNextResourceDescriptor(
    IN const CM_FULL_RESOURCE_DESCRIPTOR *ResourceDescriptor)
{
    const CM_PARTIAL_RESOURCE_DESCRIPTOR *LastPartialDescriptor;
    LastPartialDescriptor = &ResourceDescriptor->PartialResourceList.PartialDescriptors[
            ResourceDescriptor->PartialResourceList.Count - 1];
    /* CmiGetNextPartialDescriptor is also FORCEINLINE, expand it inline */
    return (PCM_FULL_RESOURCE_DESCRIPTOR)(
        (PCHAR)LastPartialDescriptor +
        sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR) +
        ((LastPartialDescriptor->Type == CmResourceTypeDeviceSpecific)
            ? LastPartialDescriptor->u.DeviceSpecificData.DataSize
            : 0));
}

/* ========================================================================
 * IopInitializeDriverModule — translated to Rust 2026-05-06.
 *
 * Direct port to `iomgr/driver.rs::IopInitializeDriverModule` (`#[no_mangle]
 * pub unsafe extern "C"`). Was the last "complex delegation" entry on the
 * io_rust_shims.c ledger. Mirrors the C cleanup paths bit-for-bit, including
 * the major-function fixup loop (Windows compatibility) and the legacy-flag
 * clear-on-no-device-objects HACK.
 *
 * The original C body is preserved below in `#if 0` so future debug sessions
 * can compare the two side-by-side without a git checkout.
 * ======================================================================== */

#if 0
NTSTATUS
IopInitializeDriverModule(
    _In_ PLDR_DATA_TABLE_ENTRY ModuleObject,
    _In_ HANDLE ServiceHandle,
    _Out_ PDRIVER_OBJECT *OutDriverObject,
    _Out_ NTSTATUS *DriverEntryStatus)
{
    UNICODE_STRING DriverName, RegistryPath, ServiceName;
    NTSTATUS Status;

    PAGED_CODE();

    Status = IopGetDriverNames(ServiceHandle, &DriverName, &ServiceName);
    if (!NT_SUCCESS(Status))
    {
        MmUnloadSystemImage(ModuleObject);
        return Status;
    }

    DPRINT("Driver name: '%wZ'\n", &DriverName);

    PIMAGE_NT_HEADERS NtHeaders = RtlImageNtHeader(ModuleObject->DllBase);
    ASSERT(NtHeaders);
    ASSERT(ModuleObject->SizeOfImage == ROUND_TO_PAGES(NtHeaders->OptionalHeader.SizeOfImage));
    ASSERT(ModuleObject->EntryPoint == RVA(ModuleObject->DllBase, NtHeaders->OptionalHeader.AddressOfEntryPoint));

    /* Obtain the registry path for the DriverInit routine */
    PKEY_NAME_INFORMATION nameInfo;
    ULONG infoLength;
    Status = ZwQueryKey(ServiceHandle, KeyNameInformation, NULL, 0, &infoLength);
    if (Status == STATUS_BUFFER_TOO_SMALL)
    {
        nameInfo = ExAllocatePoolWithTag(NonPagedPool, infoLength, TAG_IO);
        if (nameInfo)
        {
            Status = ZwQueryKey(ServiceHandle,
                                KeyNameInformation,
                                nameInfo,
                                infoLength,
                                &infoLength);
            if (NT_SUCCESS(Status))
            {
                RegistryPath.Length = nameInfo->NameLength;
                RegistryPath.MaximumLength = nameInfo->NameLength;
                RegistryPath.Buffer = nameInfo->Name;
            }
            else
            {
                ExFreePoolWithTag(nameInfo, TAG_IO);
            }
        }
        else
        {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }
    else
    {
        Status = NT_SUCCESS(Status) ? STATUS_UNSUCCESSFUL : Status;
    }

    if (!NT_SUCCESS(Status))
    {
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        MmUnloadSystemImage(ModuleObject);
        return Status;
    }

    /* Create the driver object */
    ULONG ObjectSize = sizeof(DRIVER_OBJECT) + sizeof(EXTENDED_DRIVER_EXTENSION);
    OBJECT_ATTRIBUTES objAttrs;
    PDRIVER_OBJECT driverObject;
    InitializeObjectAttributes(&objAttrs,
                               &DriverName,
                               OBJ_PERMANENT | OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);

    Status = ObCreateObject(KernelMode,
                            IoDriverObjectType,
                            &objAttrs,
                            KernelMode,
                            NULL,
                            ObjectSize,
                            0,
                            0,
                            (PVOID*)&driverObject);
    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        MmUnloadSystemImage(ModuleObject);
        DPRINT1("Error while creating driver object \"%wZ\" status %x\n", &DriverName, Status);
        return Status;
    }

    DPRINT("Created driver object 0x%p for \"%wZ\"\n", driverObject, &DriverName);

    RtlZeroMemory(driverObject, ObjectSize);
    driverObject->Type = IO_TYPE_DRIVER;
    driverObject->Size = sizeof(DRIVER_OBJECT);

    /* Set the legacy flag if this is not a WDM driver */
    if (!(NtHeaders->OptionalHeader.DllCharacteristics & IMAGE_DLLCHARACTERISTICS_WDM_DRIVER))
        driverObject->Flags |= DRVO_LEGACY_DRIVER;

    driverObject->DriverSection = ModuleObject;
    driverObject->DriverStart = ModuleObject->DllBase;
    driverObject->DriverSize = ModuleObject->SizeOfImage;
    driverObject->DriverInit = ModuleObject->EntryPoint;
    driverObject->HardwareDatabase = &IopHardwareDatabaseKey;
    driverObject->DriverExtension = (PDRIVER_EXTENSION)(driverObject + 1);
    driverObject->DriverExtension->DriverObject = driverObject;

    /* Loop all Major Functions */
    for (INT i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
    {
        driverObject->MajorFunction[i] = IopInvalidDeviceRequest;
    }

    /* Add the Object and get its handle */
    HANDLE hDriver;
    Status = ObInsertObject(driverObject, NULL, FILE_READ_DATA, 0, NULL, &hDriver);
    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        return Status;
    }

    /* Now reference it */
    Status = ObReferenceObjectByHandle(hDriver,
                                       0,
                                       IoDriverObjectType,
                                       KernelMode,
                                       (PVOID*)&driverObject,
                                       NULL);

    /* Close the extra handle */
    ZwClose(hDriver);

    if (!NT_SUCCESS(Status))
    {
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        return Status;
    }

    /* Set up the service key name buffer */
    UNICODE_STRING serviceKeyName;
    serviceKeyName.Length = 0;
    serviceKeyName.MaximumLength = ServiceName.MaximumLength + sizeof(UNICODE_NULL);
    serviceKeyName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
                                                  serviceKeyName.MaximumLength,
                                                  TAG_IO);
    if (!serviceKeyName.Buffer)
    {
        ObMakeTemporaryObject(driverObject);
        ObDereferenceObject(driverObject);
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&ServiceName);
        RtlFreeUnicodeString(&DriverName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Copy the name and set it in the driver extension */
    RtlCopyUnicodeString(&serviceKeyName, &ServiceName);
    RtlFreeUnicodeString(&ServiceName);
    driverObject->DriverExtension->ServiceKeyName = serviceKeyName;

    /* Make a copy of the driver name to store in the driver object */
    UNICODE_STRING driverNamePaged;
    driverNamePaged.Length = 0;
    driverNamePaged.MaximumLength = DriverName.MaximumLength + sizeof(UNICODE_NULL);
    driverNamePaged.Buffer = ExAllocatePoolWithTag(PagedPool,
                                                   driverNamePaged.MaximumLength,
                                                   TAG_IO);
    if (!driverNamePaged.Buffer)
    {
        ObMakeTemporaryObject(driverObject);
        ObDereferenceObject(driverObject);
        ExFreePoolWithTag(nameInfo, TAG_IO);
        RtlFreeUnicodeString(&DriverName);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyUnicodeString(&driverNamePaged, &DriverName);
    driverObject->DriverName = driverNamePaged;

    /* Finally, call its init function */
    DPRINT("Calling DriverInit for '%wZ'\n", &DriverName);
    Status = driverObject->DriverInit(driverObject, &RegistryPath);
    *DriverEntryStatus = Status;
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("'%wZ' initialization failed, status (0x%08lx)\n", &DriverName, Status);
        Status = STATUS_FAILED_DRIVER_ENTRY;
    }

    /* HACK: We're going to say if we don't have any DOs from DriverEntry, then we're not legacy. */
    if (!driverObject->DeviceObject)
    {
        driverObject->Flags &= ~DRVO_LEGACY_DRIVER;
    }

    /* Windows does this fixup, keep it for compatibility */
    for (INT i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
    {
        if (!driverObject->MajorFunction[i])
        {
            DPRINT1("Driver <%wZ> set DriverObject->MajorFunction[%lu] to NULL!\n",
                    &driverObject->DriverName, i);
            driverObject->MajorFunction[i] = IopInvalidDeviceRequest;
        }
    }

    ExFreePoolWithTag(nameInfo, TAG_IO);
    RtlFreeUnicodeString(&DriverName);

    if (!NT_SUCCESS(Status))
    {
        ObMakeTemporaryObject(driverObject);
        ObDereferenceObject(driverObject);
        return Status;
    }

    *OutDriverObject = driverObject;

    MmFreeDriverInitialization((PLDR_DATA_TABLE_ENTRY)driverObject->DriverSection);

    /* Set the driver as initialized */
    IopReadyDeviceObjects(driverObject);

    if (PnpSystemInit) IopReinitializeDrivers();

    return STATUS_SUCCESS;
}
#endif /* 0 — IopInitializeDriverModule body now in iomgr/driver.rs */

/*
 * IopDeviceNodeSetFlag — C macro in io.h → real function for Rust.
 */
#undef IopDeviceNodeSetFlag
VOID
NTAPI
IopDeviceNodeSetFlag(IN PDEVICE_NODE DeviceNode, IN ULONG Flag)
{
    DeviceNode->Flags |= Flag;
}


/* ========================================================================
 * devaction.rs C shims
 * ======================================================================== */

/* `PiSetDevNodeText_c` deleted 2026-05-06 — Rust impl in
 * `pnpmgr/devaction.rs::PiSetDevNodeText` replaces it. Another
 * captures(none) ghost. */

/* `IoRequestDeviceEject_c` deleted 2026-05-06 — Rust impl in
 * `pnpmgr/devaction.rs::IoRequestDeviceEject` replaces it. Final
 * captures(none) ghost from the `_c`-suffixed PnP shim family. */

/* `PiQueueDeviceAction_c` and `PiPerformSyncDeviceAction_c` deleted
 * 2026-05-06 — Rust impls in `pnpmgr/devaction.rs::PiQueueDeviceAction` /
 * `PiPerformSyncDeviceAction` replace them. The historical "Delegated to C
 * to avoid potential cross-CGU issues" comment was a captures(none) ghost
 * (same family as the Ob*_shim trio in commit 2ba8dab4 and
 * IopGetBusTypeGuidIndex in commit 280782e5). The workspace `lto = false` +
 * `codegen-units = 256` firewall keeps the Rust callees in different CGUs
 * from the caller. */
