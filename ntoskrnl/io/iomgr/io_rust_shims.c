/*
 * io_rust_shims.c — surviving C-side helpers for the Rust IO subsystem.
 *
 * Most of what used to live here (probe wrappers that became io-inlines,
 * captures(none) workarounds, the entire `_c`-suffixed PnP delegation
 * family, the `IopInitializeDriverModule` body) has been promoted to
 * Rust impls in `ntoskrnl-rust` or to the sibling `io-inlines` rlib.
 * See `docs/engineering/shim-elimination.md` for the audit trail; the
 * per-trampoline removal commits live in
 * `git log --oneline -- ntoskrnl/io/iomgr/io_rust_shims.c`.
 *
 * What's left:
 *   - Macro / FORCEINLINE bridges that need the C preprocessor:
 *     `HalInitPnpDriver_shim` (HALDISPATCH dispatch macro from haltypes.h),
 *     `HalGetDmaAdapter` ditto — kept so iomgr.rs can stay extern-decl
 *     symmetric with the rest of the HAL macros, even though pnpdma.rs
 *     itself goes through HalDispatchTable directly,
 *     `IopIsValidPhysicalDeviceObject` (DeviceNode flags macro from io.h),
 *     `IopDeviceNodeSetFlag` (macro from io.h),
 *     `CmiGetNextResourceDescriptor` (FORCEINLINE from cmreslist.h).
 *   - Probe helpers: `ProbeForWriteHandle` and `ProbeForReadLargeInteger`
 *     wrap probe.h macros and provide the manual range-check + volatile
 *     copy that Rust callers can't easily express through the macro.
 *   - `ObReferenceObject` cdecl bridge — i386 fastcall→cdecl repackage.
 *   - `RtlUnicodeStringPrintf_4wZ_shim` — variadic wrapper for the fixed
 *     "%wZ%wZ#%wZ%wZ" format used by deviface.rs (Rust can't safely
 *     cross the `va_arg` boundary).
 */

#include <ntoskrnl.h>
#include <ntstrsafe.h>
#define NDEBUG
#include <debug.h>
#include "internal/io_i.h"

/*
 * HalInitPnpDriver — macro in haltypes.h that calls HALDISPATCH->HalInitPnpDriver.
 * Rust can't expand this macro, needs a C shim.
 */
NTSTATUS
HalInitPnpDriver_shim(VOID)
{
    return HalInitPnpDriver();
}

/*
 * ObReferenceObject — C macro that calls ObfReferenceObject (FASTCALL).
 * On i386, Rust extern "C" generates cdecl symbol; needs wrapper.
 * shim_exports.rs supplies the cdecl `_ObReferenceObject`; this provides
 * the NTAPI variant for callers that go through the bindgen `extern "system"`
 * declaration.
 */
#undef ObReferenceObject
VOID
NTAPI
ObReferenceObject(IN PVOID Object)
{
    ObfReferenceObject(Object);
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
 * pnpdma.rs currently calls through HalDispatchTable directly, but the
 * trampoline stays as the canonical NTAPI symbol so future callers don't
 * have to know about the dispatch-table indirection.
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
 * IopDeviceNodeSetFlag — C macro in io.h → real function for Rust.
 */
#undef IopDeviceNodeSetFlag
VOID
NTAPI
IopDeviceNodeSetFlag(IN PDEVICE_NODE DeviceNode, IN ULONG Flag)
{
    DeviceNode->Flags |= Flag;
}
