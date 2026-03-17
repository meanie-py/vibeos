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
 * IopValidateOpenPacket — static __inline in io_x.h.
 * Validates that an OPEN_PACKET structure is properly formed.
 */
BOOLEAN
NTAPI
IopValidateOpenPacket_shim(IN POPEN_PACKET OpenPacket)
{
    return IopValidateOpenPacket(OpenPacket);
}

/*
 * IoGetDevObjExtension — C macro in io.h.
 * Returns the device object extension pointer.
 */
PEXTENDED_DEVOBJ_EXTENSION
NTAPI
IoGetDevObjExtension_shim(IN PDEVICE_OBJECT DeviceObject)
{
    return IoGetDevObjExtension(DeviceObject);
}

/*
 * IopUnlockFileObject — static __inline in io_x.h.
 * Wraps InterlockedExchange(Busy,FALSE) + event signal + ObDereferenceObject.
 */
VOID
NTAPI
IopUnlockFileObject_shim(IN PFILE_OBJECT FileObject)
{
    IopUnlockFileObject(FileObject);
}

/*
 * IopUpdateTransferCount — static __inline in io_x.h.
 * Updates per-process and global I/O transfer counters.
 */
VOID
NTAPI
IopUpdateTransferCount_shim(IN IOP_TRANSFER_TYPE Type, IN ULONG TransferCount)
{
    IopUpdateTransferCount(Type, TransferCount);
}


/*
 * IopLockFileObject — static __inline in io_x.h.
 * Acquires the file object lock for synchronous I/O.
 */
NTSTATUS
NTAPI
IopLockFileObject_shim(IN PFILE_OBJECT FileObject, IN KPROCESSOR_MODE WaitMode)
{
    return IopLockFileObject(FileObject, WaitMode);
}

/*
 * IopUpdateOperationCount — static __inline in io_x.h.
 * Updates per-process I/O operation counts.
 */
VOID
NTAPI
IopUpdateOperationCount_shim(IN IOP_TRANSFER_TYPE Type)
{
    IopUpdateOperationCount(Type);
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
 * IopQueueIrpToThread — FORCEINLINE in io_x.h.
 * Queues an IRP to the current thread's IRP list.
 */
VOID
NTAPI
IopQueueIrpToThread_shim(IN PIRP Irp)
{
    IopQueueIrpToThread(Irp);
}

/*
 * IopUnQueueIrpFromThread — FORCEINLINE in io_x.h.
 * Removes an IRP from its thread's IRP list.
 */
VOID
NTAPI
IopUnQueueIrpFromThread_shim(IN PIRP Irp)
{
    IopUnQueueIrpFromThread(Irp);
}

/*
 * IoCopyCurrentIrpStackLocationToNext — FORCEINLINE in iofuncs.h.
 * Copies the current IRP stack location to the next one.
 * Uses CDECL (not NTAPI) so Rust can link with extern "C".
 */
VOID
__cdecl
IoCopyCurrentIrpStackLocationToNext_shim(IN PIRP Irp)
{
    IoCopyCurrentIrpStackLocationToNext(Irp);
}


/*
 * RtlAnsiStringToUnicodeString — NTAPI (stdcall) in ntdll/rtl.
 * Rust extern "stdcall" on i686-pc-windows-gnu doesn't generate @12 decoration
 * for this function. Use a cdecl shim to avoid symbol mismatch.
 */
NTSTATUS
__cdecl
RtlAnsiStringToUnicodeString_shim(
    PUNICODE_STRING DestinationString,
    PANSI_STRING SourceString,
    BOOLEAN AllocateDestinationString)
{
    return RtlAnsiStringToUnicodeString(DestinationString, SourceString, AllocateDestinationString);
}

/* ========================================================================
 * ObCreateObject/ObInsertObject/ObReferenceObjectByHandle shims for device.rs
 *
 * Workaround for LLVM cross-CGU `captures(none)` attribute propagation bug:
 * when Ob* functions (defined in ob/*.rs, same crate) land in a different
 * codegen unit, LLVM propagates `captures(none)` to the declaration, then
 * concludes the output pointer is never written → eliminates the success path.
 * Calling through C shims avoids this.
 * ======================================================================== */

NTSTATUS
NTAPI
ObCreateObject_shim(
    IN KPROCESSOR_MODE ProbeMode,
    IN POBJECT_TYPE ObjectType,
    IN POBJECT_ATTRIBUTES ObjectAttributes,
    IN KPROCESSOR_MODE OwnershipMode,
    IN OUT PVOID ParseContext,
    IN ULONG ObjectBodySize,
    IN ULONG PagedPoolCharge,
    IN ULONG NonPagedPoolCharge,
    OUT PVOID *Object)
{
    return ObCreateObject(ProbeMode, ObjectType, ObjectAttributes,
                          OwnershipMode, ParseContext, ObjectBodySize,
                          PagedPoolCharge, NonPagedPoolCharge, Object);
}

NTSTATUS
NTAPI
ObInsertObject_shim(
    IN PVOID Object,
    IN PACCESS_STATE AccessState,
    IN ACCESS_MASK DesiredAccess,
    IN ULONG ObjectPointerBias,
    OUT PVOID *NewObject,
    OUT PHANDLE Handle)
{
    return ObInsertObject(Object, AccessState, DesiredAccess,
                          ObjectPointerBias, NewObject, Handle);
}

NTSTATUS
NTAPI
ObReferenceObjectByHandle_shim(
    IN HANDLE Handle,
    IN ACCESS_MASK DesiredAccess,
    IN POBJECT_TYPE ObjectType,
    IN KPROCESSOR_MODE AccessMode,
    OUT PVOID *Object,
    OUT POBJECT_HANDLE_INFORMATION HandleInformation)
{
    return ObReferenceObjectByHandle(Handle, DesiredAccess, ObjectType,
                                     AccessMode, Object, HandleInformation);
}

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
 * IopGetRegistryValue shim
 *
 * Workaround for LLVM cross-CGU `captures(none)` attribute propagation bug:
 * when IopGetRegistryValue (defined in pnpmgr.rs) is called from pnpinit.rs
 * (different codegen unit), LLVM propagates `captures(none)` to the
 * declaration, then concludes the output pointer (Information) is never
 * written → eliminates the success path. Calling through a C shim avoids this.
 * ======================================================================== */
NTSTATUS NTAPI IopGetRegistryValue(IN HANDLE Handle, IN PWSTR ValueName,
    OUT PKEY_VALUE_FULL_INFORMATION *Information);
NTSTATUS NTAPI IopGetRegistryValue_shim(IN HANDLE Handle, IN PWSTR ValueName,
    OUT PKEY_VALUE_FULL_INFORMATION *Information)
{
    return IopGetRegistryValue(Handle, ValueName, Information);
}

/* ========================================================================
 * IopOpenRegistryKeyEx / IopCreateRegistryKeyEx shims
 *
 * Same captures(none) workaround as IopGetRegistryValue_shim: these functions
 * write to output pointer parameters (*KeyHandle / *Handle). When called from
 * a different Rust CGU, LLVM may eliminate the write.
 * ======================================================================== */
NTSTATUS NTAPI IopOpenRegistryKeyEx(IN PHANDLE KeyHandle, IN HANDLE ParentKey,
    IN PUNICODE_STRING Name, IN ACCESS_MASK DesiredAccess);
NTSTATUS NTAPI IopOpenRegistryKeyEx_shim(IN PHANDLE KeyHandle, IN HANDLE ParentKey,
    IN PUNICODE_STRING Name, IN ACCESS_MASK DesiredAccess)
{
    return IopOpenRegistryKeyEx(KeyHandle, ParentKey, Name, DesiredAccess);
}

NTSTATUS NTAPI IopCreateRegistryKeyEx(OUT PHANDLE Handle, IN HANDLE RootHandle,
    IN PUNICODE_STRING KeyName, IN ACCESS_MASK DesiredAccess,
    IN ULONG CreateOptions, OUT PULONG Disposition);
NTSTATUS NTAPI IopCreateRegistryKeyEx_shim(OUT PHANDLE Handle, IN HANDLE RootHandle,
    IN PUNICODE_STRING KeyName, IN ACCESS_MASK DesiredAccess,
    IN ULONG CreateOptions, OUT PULONG Disposition)
{
    return IopCreateRegistryKeyEx(Handle, RootHandle, KeyName, DesiredAccess,
                                  CreateOptions, Disposition);
}

/* ========================================================================
 * PiInitCacheGroupInformation shim
 *
 * The entire function must be in C because LLVM's cross-CGU captures(none)
 * optimization eliminates the success path for IopGetRegistryValue output
 * pointers in the Rust version, even with volatile reads. The compiler
 * hard-codes the else branch despite runtime values being correct.
 * ======================================================================== */
extern PUNICODE_STRING PiInitGroupOrderTable;
extern USHORT PiInitGroupOrderTableCount;
NTSTATUS PnpRegMultiSzToUnicodeStrings(
    IN PKEY_VALUE_FULL_INFORMATION KeyValueInformation,
    OUT PUNICODE_STRING *UnicodeStringList,
    OUT PULONG Count);

CODE_SEG("INIT")
NTSTATUS NTAPI
PiInitCacheGroupInformation_shim(VOID)
{
    HANDLE KeyHandle;
    NTSTATUS Status;
    PKEY_VALUE_FULL_INFORMATION KeyValueInformation;
    PUNICODE_STRING GroupTable;
    ULONG Count;
    UNICODE_STRING GroupString =
        RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet"
                            L"\\Control\\ServiceGroupOrder");

    Status = IopOpenRegistryKeyEx(&KeyHandle, NULL, &GroupString, KEY_READ);
    if (NT_SUCCESS(Status))
    {
        Status = IopGetRegistryValue(KeyHandle, L"List", &KeyValueInformation);
        ZwClose(KeyHandle);

        if (NT_SUCCESS(Status))
        {
            if ((KeyValueInformation->Type == REG_MULTI_SZ) &&
                (KeyValueInformation->DataLength))
            {
                Status = PnpRegMultiSzToUnicodeStrings(KeyValueInformation,
                                                        &GroupTable,
                                                        &Count);
                PiInitGroupOrderTable = GroupTable;
                PiInitGroupOrderTableCount = (USHORT)Count;
            }
            else
            {
                Status = STATUS_UNSUCCESSFUL;
            }
            ExFreePool(KeyValueInformation);
        }
    }
    return Status;
}

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

/*
 * RtlStringCbPrintfW — FORCEINLINE variadic in ntstrsafe.h.
 * Rust can't call variadic inlines. This shim handles the specific
 * "%lx&%lx" format used in IopGetParentIdPrefix.
 */
NTSTATUS
NTAPI
RtlStringCbPrintfW_2lx(
    IN OUT PWSTR pszDest,
    IN SIZE_T cbDest,
    IN ULONG Value1,
    IN ULONG Value2)
{
    return RtlStringCbPrintfW(pszDest, cbDest, L"%lx&%lx", Value1, Value2);
}

/* ========================================================================
 * IopInitializeDriverModule — from driver.c
 * Creates a driver object, initializes it, and calls DriverEntry.
 * ======================================================================== */

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
