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
 * IopInitializePlugPlayServices — Rust-defined stdcall entry point.
 * Calling it through core::hint::black_box() in iomgr.rs miscompiles on i386
 * into an immediate `ud2`. Route through a C shim so LLVM can't fold the
 * continuation in IoInitSystem away.
 */
NTSTATUS
IopInitializePlugPlayServices_shim(VOID)
{
    return IopInitializePlugPlayServices();
}

/*
 * IopValidateOpenPacket — static __inline in io_x.h.
 * Body in Rust at sdk/lib/rust/io-inlines/src/lib.rs::_rs_IopValidateOpenPacket;
 * trampoline keeps the `_shim`-suffixed ABI anchor symbol.
 */
extern BOOLEAN _rs_IopValidateOpenPacket(POPEN_PACKET OpenPacket);

BOOLEAN
NTAPI
IopValidateOpenPacket_shim(IN POPEN_PACKET OpenPacket)
{
    return _rs_IopValidateOpenPacket(OpenPacket);
}

/*
 * IoGetDevObjExtension — C macro in io.h.
 * Body in Rust at sdk/lib/rust/io-inlines/src/lib.rs::_rs_IoGetDevObjExtension.
 */
extern PEXTENDED_DEVOBJ_EXTENSION _rs_IoGetDevObjExtension(PDEVICE_OBJECT DeviceObject);

PEXTENDED_DEVOBJ_EXTENSION
NTAPI
IoGetDevObjExtension_shim(IN PDEVICE_OBJECT DeviceObject)
{
    return _rs_IoGetDevObjExtension(DeviceObject);
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
 * IopQueueIrpToThread — FORCEINLINE in io_x.h.
 * Body in Rust at sdk/lib/rust/io-inlines/src/lib.rs::_rs_IopQueueIrpToThread.
 */
extern void _rs_IopQueueIrpToThread(PIRP Irp);

VOID
NTAPI
IopQueueIrpToThread_shim(IN PIRP Irp)
{
    _rs_IopQueueIrpToThread(Irp);
}

/*
 * IopUnQueueIrpFromThread — FORCEINLINE in io_x.h.
 * Body in Rust at sdk/lib/rust/io-inlines/src/lib.rs::_rs_IopUnQueueIrpFromThread.
 */
extern void _rs_IopUnQueueIrpFromThread(PIRP Irp);

VOID
NTAPI
IopUnQueueIrpFromThread_shim(IN PIRP Irp)
{
    _rs_IopUnQueueIrpFromThread(Irp);
}

/*
 * IoCopyCurrentIrpStackLocationToNext — FORCEINLINE in iofuncs.h.
 * Body in Rust at sdk/lib/rust/io-inlines/src/lib.rs::_rs_IoCopyCurrentIrpStackLocationToNext.
 * Uses CDECL (not NTAPI) so Rust can link with extern "C".
 */
extern void _rs_IoCopyCurrentIrpStackLocationToNext(PIRP Irp);

VOID
__cdecl
IoCopyCurrentIrpStackLocationToNext_shim(IN PIRP Irp)
{
    _rs_IoCopyCurrentIrpStackLocationToNext(Irp);
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

/*
 * Additional RTL string helpers for i386 Rust callers.
 * Keep them cdecl for consistent linkage from extern "C" declarations.
 */
NTSTATUS
__cdecl
RtlUnicodeStringToAnsiString_shim(
    PANSI_STRING DestinationString,
    PUNICODE_STRING SourceString,
    BOOLEAN AllocateDestinationString)
{
    return RtlUnicodeStringToAnsiString(DestinationString, SourceString, AllocateDestinationString);
}

VOID
__cdecl
RtlInitAnsiString_shim(
    PANSI_STRING DestinationString,
    PCSZ SourceString)
{
    RtlInitAnsiString(DestinationString, SourceString);
}

VOID
__cdecl
RtlFreeUnicodeString_shim(
    PUNICODE_STRING UnicodeString)
{
    RtlFreeUnicodeString(UnicodeString);
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

/*
 * i386-specific LLVM workaround for Rust IoCreateDevice:
 * return the created object address as ULONG_PTR rather than PVOID because the
 * i386 Rust codegen keeps folding raw-pointer shim results back to NULL.
 * The Rust side performs the visible zero-init after the non-zero check so
 * later field reads are seen as initialized by the optimizer.
 */
ULONG_PTR
NTAPI
ObCreateObjectAddress_shim(
    IN KPROCESSOR_MODE ProbeMode,
    IN POBJECT_TYPE ObjectType,
    IN POBJECT_ATTRIBUTES ObjectAttributes,
    IN KPROCESSOR_MODE OwnershipMode,
    IN OUT PVOID ParseContext,
    IN ULONG ObjectBodySize,
    IN ULONG PagedPoolCharge,
    IN ULONG NonPagedPoolCharge,
    OUT NTSTATUS *Status)
{
    PVOID Object = NULL;

    *Status = ObCreateObject(ProbeMode, ObjectType, ObjectAttributes,
                             OwnershipMode, ParseContext, ObjectBodySize,
                             PagedPoolCharge, NonPagedPoolCharge, &Object);
    return (ULONG_PTR)Object;
}

NTSTATUS
NTAPI
ObInsertObject_shim(
    IN PVOID Object,
    IN PACCESS_STATE AccessState,
    IN ACCESS_MASK DesiredAccess,
    IN ULONG ObjectPointerBias,
    OUT PULONG_PTR NewObject,
    OUT PHANDLE Handle)
{
    PVOID LocalNewObject = NULL;
    NTSTATUS Status;

    Status = ObInsertObject(Object, AccessState, DesiredAccess,
                            ObjectPointerBias, &LocalNewObject, Handle);
    *NewObject = (ULONG_PTR)LocalNewObject;
    return Status;
}

NTSTATUS
NTAPI
ObReferenceObjectByHandle_shim(
    IN HANDLE Handle,
    IN ACCESS_MASK DesiredAccess,
    IN POBJECT_TYPE ObjectType,
    IN KPROCESSOR_MODE AccessMode,
    OUT PULONG_PTR Object,
    OUT POBJECT_HANDLE_INFORMATION HandleInformation)
{
    PVOID LocalObject = NULL;
    NTSTATUS Status;

    Status = ObReferenceObjectByHandle(Handle, DesiredAccess, ObjectType,
                                       AccessMode, &LocalObject, HandleInformation);
    *Object = (ULONG_PTR)LocalObject;
    return Status;
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
 * IopQueryDeviceCapabilities C implementation
 *
 * The Rust version causes an amd64-specific hang during PnP enumeration.
 * ======================================================================== */
NTSTATUS
NTAPI
IopQueryDeviceCapabilities_c(PDEVICE_NODE DeviceNode,
                             PDEVICE_CAPABILITIES DeviceCaps)
{
    IO_STATUS_BLOCK StatusBlock;
    IO_STACK_LOCATION Stack;
    NTSTATUS Status;
    HANDLE InstanceKey;
    UNICODE_STRING ValueName;

    RtlZeroMemory(DeviceCaps, sizeof(DEVICE_CAPABILITIES));
    DeviceCaps->Size = sizeof(DEVICE_CAPABILITIES);
    DeviceCaps->Version = 1;
    DeviceCaps->Address = -1;
    DeviceCaps->UINumber = -1;

    RtlZeroMemory(&Stack, sizeof(IO_STACK_LOCATION));
    Stack.Parameters.DeviceCapabilities.Capabilities = DeviceCaps;

    Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
                               &StatusBlock,
                               IRP_MN_QUERY_CAPABILITIES,
                               &Stack);
    if (!NT_SUCCESS(Status))
    {
        return Status;
    }

    DeviceNode->CapabilityFlags = 0;
    if (DeviceCaps->LockSupported)
        DeviceNode->CapabilityFlags |= 0x00000001;
    if (DeviceCaps->EjectSupported)
        DeviceNode->CapabilityFlags |= 0x00000002;
    if (DeviceCaps->Removable)
        DeviceNode->CapabilityFlags |= 0x00000004;
    if (DeviceCaps->DockDevice)
        DeviceNode->CapabilityFlags |= 0x00000008;
    if (DeviceCaps->UniqueID)
        DeviceNode->CapabilityFlags |= 0x00000010;
    if (DeviceCaps->SilentInstall)
        DeviceNode->CapabilityFlags |= 0x00000020;
    if (DeviceCaps->RawDeviceOK)
        DeviceNode->CapabilityFlags |= 0x00000040;
    if (DeviceCaps->SurpriseRemovalOK)
        DeviceNode->CapabilityFlags |= 0x00000080;
    if (DeviceCaps->HardwareDisabled)
        DeviceNode->CapabilityFlags |= 0x00000100;
    if (DeviceCaps->NonDynamic)
        DeviceNode->CapabilityFlags |= 0x00000200;

    if (DeviceCaps->NoDisplayInUI)
        DeviceNode->UserFlags |= DNUF_DONT_SHOW_IN_UI;
    else
        DeviceNode->UserFlags &= ~DNUF_DONT_SHOW_IN_UI;

    Status = IopCreateDeviceKeyPath(&DeviceNode->InstancePath, REG_OPTION_NON_VOLATILE, &InstanceKey);
    if (NT_SUCCESS(Status))
    {
        RtlInitUnicodeString(&ValueName, L"Capabilities");
        Status = ZwSetValueKey(InstanceKey, &ValueName, 0, REG_DWORD,
                               &DeviceNode->CapabilityFlags, sizeof(ULONG));
        if (DeviceCaps->UINumber != MAXULONG)
        {
            RtlInitUnicodeString(&ValueName, L"UINumber");
            Status = ZwSetValueKey(InstanceKey, &ValueName, 0, REG_DWORD,
                                   &DeviceCaps->UINumber, sizeof(ULONG));
        }
        ZwClose(InstanceKey);
    }

    return Status;
}

/* ========================================================================
 * IopGetBusTypeGuidIndex C implementation
 *
 * The Rust version in pnpmgr.rs causes an amd64-specific hang. The root cause
 * is unclear (code logic appears identical). Using C version avoids the issue.
 * ======================================================================== */
extern PIO_BUS_TYPE_GUID_LIST PnpBusTypeGuidList;

USHORT
NTAPI
IopGetBusTypeGuidIndex_c(LPGUID BusTypeGuid)
{
    USHORT i = 0, FoundIndex = 0xFFFF;
    ULONG NewSize;
    PVOID NewList;

    ExAcquireFastMutex(&PnpBusTypeGuidList->Lock);

    while (i < PnpBusTypeGuidList->GuidCount)
    {
         if (RtlCompareMemory(BusTypeGuid,
                              &PnpBusTypeGuidList->Guids[i],
                              sizeof(GUID)) == sizeof(GUID))
         {
              FoundIndex = i;
              goto Quickie;
         }
         i++;
    }

    if (PnpBusTypeGuidList->GuidCount)
    {
        NewSize = sizeof(IO_BUS_TYPE_GUID_LIST) +
                 (sizeof(GUID) * PnpBusTypeGuidList->GuidCount);
        NewList = ExAllocatePool(PagedPool, NewSize);
        if (!NewList)
        {
            ExFreePool(PnpBusTypeGuidList);
            goto Quickie;
        }
        NewSize -= sizeof(GUID);
        RtlCopyMemory(NewList, PnpBusTypeGuidList, NewSize);
        ExFreePool(PnpBusTypeGuidList);
        PnpBusTypeGuidList = NewList;
    }

    RtlCopyMemory(&PnpBusTypeGuidList->Guids[PnpBusTypeGuidList->GuidCount],
                  BusTypeGuid,
                  sizeof(GUID));
    FoundIndex = (USHORT)PnpBusTypeGuidList->GuidCount;
    PnpBusTypeGuidList->GuidCount++;

Quickie:
    ExReleaseFastMutex(&PnpBusTypeGuidList->Lock);
    return FoundIndex;
}

/* ========================================================================
 * IopInitiatePnpIrp C implementation
 *
 * The Rust version in pnpmgr.rs causes an amd64-specific hang during PnP
 * device enumeration. The exact cause is unknown but likely related to
 * IO_STACK_LOCATION stack alignment or volatile semantics differences.
 * Using the C version avoids the issue.
 * ======================================================================== */
NTSTATUS
NTAPI
IopInitiatePnpIrp_c(IN PDEVICE_OBJECT DeviceObject,
                     IN OUT PIO_STATUS_BLOCK IoStatusBlock,
                     IN UCHAR MinorFunction,
                     IN PIO_STACK_LOCATION Stack OPTIONAL)
{
    IO_STACK_LOCATION IoStackLocation;

    RtlZeroMemory(&IoStackLocation, sizeof(IO_STACK_LOCATION));
    IoStackLocation.MajorFunction = IRP_MJ_PNP;
    IoStackLocation.MinorFunction = MinorFunction;
    if (Stack)
    {
        RtlCopyMemory(&IoStackLocation.Parameters,
                      &Stack->Parameters,
                      sizeof(Stack->Parameters));
    }

    IoStatusBlock->Status = IopSynchronousCall(DeviceObject,
                                               &IoStackLocation,
                                               (PVOID)&IoStatusBlock->Information);
    return IoStatusBlock->Status;
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
 * IopLoadDriver shim
 *
 * This helper also writes through an output pointer (DriverObject). Keep the
 * same C boundary workaround as the registry helpers on the early PnP path.
 * ======================================================================== */
NTSTATUS IopLoadDriver(IN HANDLE ServiceHandle, OUT PDRIVER_OBJECT *DriverObject);
NTSTATUS NTAPI IopLoadDriver_shim(IN HANDLE ServiceHandle,
    OUT PDRIVER_OBJECT *DriverObject)
{
    return IopLoadDriver(ServiceHandle, DriverObject);
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

/* IopCompleteRequest: kept in C (irp.c) — Rust version has APC bugs */

#if 0 /* IopCompleteRequest_c — disabled, using irp.c version */
VOID
NTAPI
IopCompleteRequest_c(IN PKAPC Apc,
                     IN PKNORMAL_ROUTINE* NormalRoutine,
                     IN PVOID* NormalContext,
                     IN PVOID* SystemArgument1,
                     IN PVOID* SystemArgument2)
{
    PFILE_OBJECT FileObject;
    PIRP Irp;
    PMDL Mdl, NextMdl;
    PVOID Port, Key;
    BOOLEAN SignalCompletion;

    Irp = CONTAINING_RECORD(Apc, IRP, Tail.Apc);

    DPRINT("IopCompleteRequest(Irp %p)\n", Irp);

    FileObject = (PFILE_OBJECT)*SystemArgument1;
    SignalCompletion = (Irp->UserIosb != NULL && Irp->AssociatedIrp.SystemBuffer != NULL);

    if ((Irp->Flags & IRP_BUFFERED_IO) && (Irp->Flags & IRP_DEALLOCATE_BUFFER))
    {
        if ((Irp->Flags & IRP_INPUT_OPERATION) && NT_SUCCESS(Irp->IoStatus.Status))
        {
            _SEH2_TRY
            {
                RtlCopyMemory(Irp->UserBuffer,
                              Irp->AssociatedIrp.SystemBuffer,
                              Irp->IoStatus.Information);
            }
            _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
            {
                Irp->IoStatus.Status = _SEH2_GetExceptionCode();
            }
            _SEH2_END;
        }

        ExFreePool(Irp->AssociatedIrp.SystemBuffer);
    }

    Irp->Flags &= ~(IRP_BUFFERED_IO | IRP_DEALLOCATE_BUFFER | IRP_INPUT_OPERATION);

    if (Irp->UserIosb)
    {
        _SEH2_TRY
        {
            *Irp->UserIosb = Irp->IoStatus;
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            DPRINT1("IopCompleteRequest: UserIosb write failed\n");
        }
        _SEH2_END;
    }

    if ((Irp->Flags & IRP_CREATE_OPERATION) && FileObject)
    {
        if (!NT_SUCCESS(Irp->IoStatus.Status))
        {
            DPRINT("IopCompleteRequest: failed create\n");
            FileObject->Flags |= FO_FILE_OPEN_CANCELLED;
        }
    }

    if (Irp->UserEvent)
    {
        KeSetEvent(Irp->UserEvent, 0, FALSE);
    }
    else if (FileObject)
    {
        KeSetEvent(&FileObject->Event, 0, FALSE);
    }

    Port = (Irp->Flags & IRP_CREATE_OPERATION) ? NULL : Irp->Tail.Overlay.OriginalFileObject;
    Key = NULL;
    if (Port && FileObject)
    {
        Port = FileObject->CompletionContext;
        if (Port)
        {
            Key = FileObject->CompletionContext->Key;
            Port = FileObject->CompletionContext->Port;
        }
    }

    if (FileObject)
    {
        if (!(Irp->Flags & IRP_CREATE_OPERATION))
        {
            ObDereferenceObjectDeferDelete(FileObject);
        }
    }

    Irp->Tail.Overlay.PacketType = IopCompletionPacketIrp;
    if (Port && Irp->Overlay.AsynchronousParameters.UserApcRoutine == NULL)
    {
        if (NT_SUCCESS(IoSetIoCompletion(Port,
                                          Key,
                                          Irp->Overlay.AsynchronousParameters.UserApcContext,
                                          Irp->IoStatus.Status,
                                          Irp->IoStatus.Information,
                                          TRUE)))
        {
            Mdl = Irp->MdlAddress;
            while (Mdl)
            {
                NextMdl = Mdl->Next;
                IoFreeMdl(Mdl);
                Mdl = NextMdl;
            }
            IoFreeIrp(Irp);
            return;
        }
    }

    if (Irp->Overlay.AsynchronousParameters.UserApcRoutine)
    {
        KeInitializeApc(&Irp->Tail.Apc,
                        KeGetCurrentThread(),
                        CurrentApcEnvironment,
                        IopAbortIrpKernelApc,
                        NULL,
                        (PKNORMAL_ROUTINE)Irp->Overlay.AsynchronousParameters.UserApcRoutine,
                        Irp->RequestorMode,
                        Irp->Overlay.AsynchronousParameters.UserApcContext);
        KeInsertQueueApc(&Irp->Tail.Apc,
                          Irp->UserIosb,
                          NULL,
                          2);
    }
    else
    {
        Mdl = Irp->MdlAddress;
        while (Mdl)
        {
            NextMdl = Mdl->Next;
            IoFreeMdl(Mdl);
            Mdl = NextMdl;
        }
        if (Irp == IopReserveIrpAllocator.ReserveIrp)
        {
            IopFreeReserveIrp(0);
        }
        else
        {
            IoFreeIrp(Irp);
        }
    }
}
#endif /* IopCompleteRequest_c */

/* ========================================================================
 * devaction.rs C shims
 * ======================================================================== */

/* GUIDs needed by IoRequestDeviceEject_c */
#define INITGUID
#include <reactos/libs/umpnpmgr/sysguid.h>
#undef INITGUID

/* DEVICE_ACTION_REQUEST struct — needed by PiQueueDeviceAction_c */
typedef struct _DEVICE_ACTION_REQUEST
{
    LIST_ENTRY RequestListEntry;
    PDEVICE_OBJECT DeviceObject;
    PKEVENT CompletionEvent;
    NTSTATUS *CompletionStatus;
    DEVICE_ACTION Action;
} DEVICE_ACTION_REQUEST, *PDEVICE_ACTION_REQUEST;

/* Forward declarations for functions called by IoRequestDeviceEject_c */
NTSTATUS NTAPI IopQueryDeviceCapabilities(PDEVICE_NODE DeviceNode, PDEVICE_CAPABILITIES DeviceCaps);
NTSTATUS IopQueryRemoveDeviceRelations(PDEVICE_RELATIONS DeviceRelations, BOOLEAN Force);
NTSTATUS IopQueryRemoveChildDevices(PDEVICE_NODE ParentDeviceNode, BOOLEAN Force);
NTSTATUS IopPrepareDeviceForRemoval(PDEVICE_OBJECT DeviceObject, BOOLEAN Force);
VOID IopCancelRemoveDeviceRelations(PDEVICE_RELATIONS DeviceRelations);
VOID IopCancelRemoveChildDevices(PDEVICE_NODE ParentDeviceNode);
VOID IopSendRemoveDeviceRelations(PDEVICE_RELATIONS DeviceRelations);
VOID IopSendRemoveChildDevices(PDEVICE_NODE ParentDeviceNode);
NTSTATUS NTAPI IopSendEject(PDEVICE_OBJECT DeviceObject);
NTSTATUS PiIrpQueryDeviceText(PDEVICE_NODE DeviceNode, LCID LocaleId,
    DEVICE_TEXT_TYPE TextType, PWSTR *Text);

/*
 * PiSetDevNodeText_c — Sets DeviceDesc and LocationInformation registry values.
 * Delegated from Rust to C to avoid potential cross-CGU issues.
 */
VOID
PiSetDevNodeText_c(
    _In_ PDEVICE_NODE DeviceNode,
    _In_ HANDLE InstanceKey)
{
    PAGED_CODE();

    LCID localeId;

    NTSTATUS status = ZwQueryDefaultLocale(FALSE, &localeId);
    if (!NT_SUCCESS(status))
    {
        DPRINT1("ZwQueryDefaultLocale() failed with status %x\n", status);
        return;
    }

    // Step 1: Write the DeviceDesc value if does not exist
    UNICODE_STRING valDeviceDesc = RTL_CONSTANT_STRING(L"DeviceDesc");
    ULONG len;

    status = ZwQueryValueKey(InstanceKey, &valDeviceDesc, KeyValueBasicInformation, NULL, 0, &len);
    if (status == STATUS_OBJECT_NAME_NOT_FOUND)
    {
        PWSTR deviceDesc = NULL;
        status = PiIrpQueryDeviceText(DeviceNode, localeId, DeviceTextDescription, &deviceDesc);

        if (deviceDesc && deviceDesc[0] != UNICODE_NULL)
        {
            status = ZwSetValueKey(InstanceKey,
                                   &valDeviceDesc,
                                   0,
                                   REG_SZ,
                                   deviceDesc,
                                   ((ULONG)wcslen(deviceDesc) + 1) * sizeof(WCHAR));

            if (!NT_SUCCESS(status))
            {
                DPRINT1("ZwSetValueKey() failed (Status %x)\n", status);
            }
        }
        else
        {
            UNICODE_STRING unknownDeviceDesc = RTL_CONSTANT_STRING(L"Unknown device");
            DPRINT("Driver didn't return DeviceDesc (status %x)\n", status);

            status = ZwSetValueKey(InstanceKey,
                                   &valDeviceDesc,
                                   0,
                                   REG_SZ,
                                   unknownDeviceDesc.Buffer,
                                   unknownDeviceDesc.MaximumLength);
            if (!NT_SUCCESS(status))
            {
                DPRINT1("ZwSetValueKey() failed (Status %x)\n", status);
            }
        }

        if (deviceDesc)
        {
            ExFreePoolWithTag(deviceDesc, 0);
        }
    }

    // Step 2: LocationInformation is overwritten unconditionally
    PWSTR deviceLocationInfo = NULL;
    status = PiIrpQueryDeviceText(DeviceNode,
                                  localeId,
                                  DeviceTextLocationInformation,
                                  &deviceLocationInfo);

    if (deviceLocationInfo && deviceLocationInfo[0] != UNICODE_NULL)
    {
        UNICODE_STRING valLocationInfo = RTL_CONSTANT_STRING(L"LocationInformation");

        status = ZwSetValueKey(InstanceKey,
                               &valLocationInfo,
                               0,
                               REG_SZ,
                               deviceLocationInfo,
                               ((ULONG)wcslen(deviceLocationInfo) + 1) * sizeof(WCHAR));
        if (!NT_SUCCESS(status))
        {
            DPRINT1("ZwSetValueKey() failed (Status %x)\n", status);
        }
    }

    if (deviceLocationInfo)
    {
        ExFreePoolWithTag(deviceLocationInfo, 0);
    }
    else
    {
        DPRINT("Driver didn't return LocationInformation (status %x)\n", status);
    }
}

/*
 * IoRequestDeviceEject_c — Eject a device.
 * Delegated from Rust to C to avoid potential cross-CGU issues.
 */
VOID
NTAPI
IoRequestDeviceEject_c(IN PDEVICE_OBJECT PhysicalDeviceObject)
{
    PDEVICE_NODE DeviceNode = IopGetDeviceNode(PhysicalDeviceObject);
    PDEVICE_RELATIONS DeviceRelations;
    IO_STATUS_BLOCK IoStatusBlock;
    IO_STACK_LOCATION Stack;
    DEVICE_CAPABILITIES Capabilities;
    NTSTATUS Status;

    IopQueueTargetDeviceEvent(&GUID_DEVICE_KERNEL_INITIATED_EJECT,
                              &DeviceNode->InstancePath);

    if (IopQueryDeviceCapabilities(DeviceNode, &Capabilities) != STATUS_SUCCESS)
    {
        goto cleanup;
    }

    Stack.Parameters.QueryDeviceRelations.Type = EjectionRelations;

    Status = IopInitiatePnpIrp(PhysicalDeviceObject,
                               &IoStatusBlock,
                               IRP_MN_QUERY_DEVICE_RELATIONS,
                               &Stack);
    if (!NT_SUCCESS(Status))
    {
        DPRINT("IopInitiatePnpIrp() failed with status 0x%08lx\n", Status);
        DeviceRelations = NULL;
    }
    else
    {
        DeviceRelations = (PDEVICE_RELATIONS)IoStatusBlock.Information;
    }

    if (DeviceRelations)
    {
        Status = IopQueryRemoveDeviceRelations(DeviceRelations, FALSE);
        if (!NT_SUCCESS(Status))
            goto cleanup;
    }

    Status = IopQueryRemoveChildDevices(DeviceNode, FALSE);
    if (!NT_SUCCESS(Status))
    {
        if (DeviceRelations)
            IopCancelRemoveDeviceRelations(DeviceRelations);
        goto cleanup;
    }

    if (IopPrepareDeviceForRemoval(PhysicalDeviceObject, FALSE) != STATUS_SUCCESS)
    {
        if (DeviceRelations)
            IopCancelRemoveDeviceRelations(DeviceRelations);
        IopCancelRemoveChildDevices(DeviceNode);
        goto cleanup;
    }

    if (DeviceRelations)
        IopSendRemoveDeviceRelations(DeviceRelations);
    IopSendRemoveChildDevices(DeviceNode);

    DeviceNode->Problem = CM_PROB_HELD_FOR_EJECT;
    if (Capabilities.EjectSupported)
    {
        if (IopSendEject(PhysicalDeviceObject) != STATUS_SUCCESS)
        {
            goto cleanup;
        }
    }
    else
    {
        // DeviceNode->Flags |= DNF_DISABLED;
    }

    IopQueueTargetDeviceEvent(&GUID_DEVICE_EJECT,
                              &DeviceNode->InstancePath);

    return;

cleanup:
    IopQueueTargetDeviceEvent(&GUID_DEVICE_EJECT_VETOED,
                              &DeviceNode->InstancePath);
}

/*
 * PiQueueDeviceAction_c — Queue a device operation to a worker thread.
 * Delegated from Rust to C to avoid potential cross-CGU issues.
 */
extern BOOLEAN PnPBootDriversLoaded;
extern LIST_ENTRY IopDeviceActionRequestList;
extern KSPIN_LOCK IopDeviceActionLock;
extern KEVENT PiEnumerationFinished;
extern WORK_QUEUE_ITEM IopDeviceActionWorkItem;
extern BOOLEAN IopDeviceActionInProgress;

VOID NTAPI PipDeviceActionWorker(PVOID Context);

VOID
PiQueueDeviceAction_c(
    _In_ PDEVICE_OBJECT DeviceObject,
    _In_ DEVICE_ACTION Action,
    _In_opt_ PKEVENT CompletionEvent,
    _Out_opt_ NTSTATUS *CompletionStatus)
{
    PDEVICE_ACTION_REQUEST Request;
    KIRQL OldIrql;

    Request = ExAllocatePoolWithTag(NonPagedPoolMustSucceed, sizeof(*Request), TAG_IO);

    DPRINT("PiQueueDeviceAction: DeviceObject - %p, Request - %p\n",
        DeviceObject, Request);

    ObReferenceObject(DeviceObject);

    Request->DeviceObject = DeviceObject;
    Request->Action = Action;
    Request->CompletionEvent = CompletionEvent;
    Request->CompletionStatus = CompletionStatus;

    KeAcquireSpinLock(&IopDeviceActionLock, &OldIrql);
    InsertTailList(&IopDeviceActionRequestList, &Request->RequestListEntry);

    if (Action == PiActionEnumRootDevices || Action == PiActionAddBootDevices)
    {
        ASSERT(!IopDeviceActionInProgress);

        IopDeviceActionInProgress = TRUE;
        KeClearEvent(&PiEnumerationFinished);
        KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);

        PipDeviceActionWorker(NULL);
        return;
    }

    if (IopDeviceActionInProgress || !PnPBootDriversLoaded)
    {
        KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);
        return;
    }
    IopDeviceActionInProgress = TRUE;
    KeClearEvent(&PiEnumerationFinished);
    KeReleaseSpinLock(&IopDeviceActionLock, OldIrql);

    ExInitializeWorkItem(&IopDeviceActionWorkItem, PipDeviceActionWorker, NULL);
    ExQueueWorkItem(&IopDeviceActionWorkItem, DelayedWorkQueue);
}

/*
 * PiPerformSyncDeviceAction_c — Perform a device operation synchronously.
 * Delegated from Rust to C to avoid potential cross-CGU issues.
 */
NTSTATUS
PiPerformSyncDeviceAction_c(
    _In_ PDEVICE_OBJECT DeviceObject,
    _In_ DEVICE_ACTION Action)
{
    KEVENT opFinished;
    NTSTATUS status;

    KeInitializeEvent(&opFinished, SynchronizationEvent, FALSE);
    PiQueueDeviceAction_c(DeviceObject, Action, &opFinished, &status);
    KeWaitForSingleObject(&opFinished, Executive, KernelMode, FALSE, NULL);

    return status;
}
