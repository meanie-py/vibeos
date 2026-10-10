#[allow(unused_imports)]
use crate::prelude::*;
// rustconv-assembly: whole-tu
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_8FA741F58E83: [u16; 18] = [
    0x0049u16, 0x006E, 0x0061, 0x0063, 0x0063, 0x0065, 0x0073, 0x0073, 0x0069, 0x0062,
    0x006C, 0x0065, 0x0052, 0x0061, 0x006E, 0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_9CE7561B3242: [u16; 18] = [
    0x0052u16, 0x0065, 0x0073, 0x0065, 0x0072, 0x0076, 0x0065, 0x0064, 0x0052, 0x0065,
    0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_DBFF551DC4C1: [u16; 14] = [
    0x004Du16, 0x006D, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0052, 0x0061,
    0x006E, 0x0067, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_FC252766FB17: [u16; 16] = [
    0x0041u16, 0x006C, 0x006C, 0x006F, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E,
    0x004F, 0x0072, 0x0064, 0x0065, 0x0072, 0x0000,
];

use ntkrnl_sys::ddk::{ARBITER_ORDERING, ARBITER_ORDERING_LIST, IO_RESOURCE_DESCRIPTOR, IO_RESOURCE_REQUIREMENTS_LIST, _OBJECT_ATTRIBUTES, PARBITER_INSTANCE, PARBITER_ORDERING, PARBITER_ORDERING_LIST, PARB_TRANSLATE_ORDERING, PHANDLE, PIO_RESOURCE_DESCRIPTOR, PIO_RESOURCE_LIST, PIO_RESOURCE_REQUIREMENTS_LIST, PKEY_VALUE_FULL_INFORMATION, PRTL_RANGE_LIST, RtlAddRange, RtlAppendUnicodeToString, RtlInitUnicodeString, UINT16, UINT64, ZwClose, ZwOpenKey, ZwQueryValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const ARBP_POLICY_ROOT: [::core::ffi::c_ushort; 61] = unsafe {
    ::core::mem::transmute::<
        [u8; 122],
        [::core::ffi::c_ushort; 61],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0A\0r\0b\0i\0t\0e\0r\0s\0\\\0\0\0",
    )
};

static mut ArbpMmConfigStart: ULONGLONG = 1 as ULONGLONG;

static mut ArbpMmConfigEnd: ULONGLONG = 0 as ULONGLONG;

pub const ARBITER_ORDERING_GRANULARITY: u32 = 16;

pub const ARBITER_ORDERING_LIMIT: u32 = 1024;

pub const ARBP_LONGEST_SUBKEY: [u16; 18] = [
    b'R' as u16, b'e' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'e' as u16, b'd' as u16,
    b'R' as u16, b'e' as u16, b's' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, b's' as u16,
    0,
];

pub type PARB_RANGE_CALLBACK = ::core::option::Option<
    unsafe extern "system" fn(
        Arbiter: PARBITER_INSTANCE,
        Context: PVOID,
        Start: UINT64,
        End: UINT64,
    ),
>;
unsafe extern "C" fn ArbpCreateOrderingList(OrderingList: PARBITER_ORDERING_LIST) -> NTSTATUS {
    PAGED_CODE!();

    (*OrderingList).Orderings = ExAllocatePoolWithTag(
        PagedPool,
        (ARBITER_ORDERING_GRANULARITY as SIZE_T)
            .wrapping_mul(core::mem::size_of::<ARBITER_ORDERING>() as SIZE_T),
        TAG_ARBITER,
    ) as PARBITER_ORDERING;
    if (*OrderingList).Orderings.is_null() {
        (*OrderingList).Count = 0 as UINT16;
        (*OrderingList).Maximum = 0 as UINT16;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*OrderingList).Count = 0 as UINT16;
    (*OrderingList).Maximum = ARBITER_ORDERING_GRANULARITY as UINT16;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibFreeOrderingList(OrderingList: PARBITER_ORDERING_LIST) {
    if !(*OrderingList).Orderings.is_null() {
        ExFreePoolWithTag((*OrderingList).Orderings as PVOID, TAG_ARBITER as ULONG);
    }

    (*OrderingList).Orderings = core::ptr::null_mut();
    (*OrderingList).Count = 0 as UINT16;
    (*OrderingList).Maximum = 0 as UINT16;
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibAddOrdering(
    OrderingList: PARBITER_ORDERING_LIST,
    Start: UINT64,
    End: UINT64,
) -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"sdk/lib/drivers/arbiter/ordering.c\0".as_ptr() as PVOID,
            83,
            core::ptr::null_mut(),
        );
    }

    if End < Start {
        return STATUS_INVALID_PARAMETER;
    }

    if (*OrderingList).Count == (*OrderingList).Maximum {
        let mut NewMaximum: ULONG;
        let mut NewArray: PARBITER_ORDERING;

        NewMaximum = if (*OrderingList).Maximum != 0 {
            ((*OrderingList).Maximum as ULONG).wrapping_mul(2 as ULONG)
        } else {
            ARBITER_ORDERING_GRANULARITY as ULONG
        };
        if NewMaximum > ARBITER_ORDERING_LIMIT as ULONG {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        NewArray = ExAllocatePoolWithTag(
            PagedPool,
            (NewMaximum as SIZE_T).wrapping_mul(core::mem::size_of::<ARBITER_ORDERING>() as SIZE_T),
            TAG_ARBITER as ULONG,
        ) as PARBITER_ORDERING;
        if NewArray.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if !(*OrderingList).Orderings.is_null() {
            RtlCopyMemory(
                NewArray as PVOID,
                (*OrderingList).Orderings as *const core::ffi::c_void,
                ((*OrderingList).Count as usize).wrapping_mul(core::mem::size_of::<ARBITER_ORDERING>()),
            );
            ExFreePoolWithTag(
                (*OrderingList).Orderings as PVOID,
                TAG_ARBITER as ULONG,
            );
        }

        (*OrderingList).Maximum = NewMaximum as UINT16;
        (*OrderingList).Orderings = NewArray;
    }

    (*(*OrderingList).Orderings.offset((*OrderingList).Count as isize)).Start = Start;
    (*(*OrderingList).Orderings.offset((*OrderingList).Count as isize)).End = End;
    (*OrderingList).Count = (*OrderingList).Count.wrapping_add(1);
    return STATUS_SUCCESS;
}
unsafe extern "C" fn ArbpExcludeOrderingRange(
    OrderingList: PARBITER_ORDERING_LIST,
    Start: UINT64,
    End: UINT64,
) -> NTSTATUS {
    let mut Pruned: ARBITER_ORDERING_LIST = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Index: UINT16;

    PAGED_CODE!();

    if End < Start {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ArbpCreateOrderingList(&mut Pruned);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Index = 0;
    while (Index as i32) < ((*OrderingList).Count as i32) {
        let CurrentStart: UINT64 = (*(*OrderingList).Orderings.offset(Index as isize)).Start;
        let CurrentEnd: UINT64 = (*(*OrderingList).Orderings.offset(Index as isize)).End;

        Status = STATUS_SUCCESS;

        if CurrentEnd < Start || CurrentStart > End {
            Status = ArbiterLibAddOrdering(&mut Pruned, CurrentStart, CurrentEnd);
        } else {
            if CurrentStart < Start {
                Status = ArbiterLibAddOrdering(&mut Pruned, CurrentStart, Start - 1);
            }
            if NT_SUCCESS(Status) && CurrentEnd > End {
                Status = ArbiterLibAddOrdering(&mut Pruned, End + 1, CurrentEnd);
            }
        }

        if !NT_SUCCESS(Status) {
            ArbiterLibFreeOrderingList(&mut Pruned);
            return Status;
        }

        Index = Index.wrapping_add(1);
    }

    ArbiterLibFreeOrderingList(OrderingList);
    *OrderingList = Pruned;
    return STATUS_SUCCESS;
}
unsafe extern "C" fn ArbpOpenPolicySubkey(SubkeyName: PCWSTR, KeyHandle: PHANDLE) -> NTSTATUS {
    // sizeof(ARBP_POLICY_ROOT ARBP_LONGEST_SUBKEY) / sizeof(WCHAR): both literals, one NUL.
    let mut PathBuffer: [WCHAR; ARBP_POLICY_ROOT.len() + ARBP_LONGEST_SUBKEY.len() - 1] =
        [0; ARBP_POLICY_ROOT.len() + ARBP_LONGEST_SUBKEY.len() - 1];
    let mut KeyPath: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    RtlInitEmptyUnicodeString(
        &raw mut KeyPath,
        PathBuffer.as_mut_ptr(),
        core::mem::size_of_val(&PathBuffer) as USHORT,
    );
    Status = RtlAppendUnicodeToString(&raw mut KeyPath, ARBP_POLICY_ROOT.as_ptr());
    if NT_SUCCESS(Status) {
        Status = RtlAppendUnicodeToString(&raw mut KeyPath, SubkeyName);
    }
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut KeyPath;
    ObjectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    return ZwOpenKey(KeyHandle, KEY_READ, &raw mut ObjectAttributes);
}
unsafe extern "C" fn ArbpReadPolicyValue(
    KeyHandle: HANDLE,
    ValueName: PCWSTR,
    Value: *mut PKEY_VALUE_FULL_INFORMATION,
) -> NTSTATUS {
    let mut NameString: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    RtlInitUnicodeString(&mut NameString, ValueName);

    Status = ZwQueryValueKey(
        KeyHandle,
        &mut NameString,
        KeyValueFullInformationAlign64,
        core::ptr::null_mut(),
        0,
        &mut Size,
    );
    if Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW {
        return if NT_SUCCESS(Status) { STATUS_UNSUCCESSFUL } else { Status };
    }

    Buffer = ExAllocatePoolWithTag(
        PagedPool,
        Size as SIZE_T,
        TAG_ARBITER as ULONG,
    ) as PKEY_VALUE_FULL_INFORMATION;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = ZwQueryValueKey(
        KeyHandle,
        &mut NameString,
        KeyValueFullInformationAlign64,
        Buffer as PVOID,
        Size,
        &mut ResultLength,
    );

    if NT_SUCCESS(Status)
        && (((*Buffer).DataOffset as ULONGLONG).wrapping_add((*Buffer).DataLength as ULONGLONG)
            > ResultLength as ULONGLONG)
    {
        Status = STATUS_UNSUCCESSFUL;
    }

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(Buffer as PVOID, TAG_ARBITER as ULONG);
        return Status;
    }

    *Value = Buffer;
    STATUS_SUCCESS
}
unsafe extern "C" fn ArbpDescriptorMatchesArbiter(
    Arbiter: PARBITER_INSTANCE,
    DescriptorType: UCHAR,
) -> BOOLEAN {
    if DescriptorType == (*Arbiter).ResourceType as UCHAR {
        return TRUE;
    }
    (DescriptorType == (CmResourceTypeMemoryLarge as u8)
        && (*Arbiter).ResourceType == (CmResourceTypeMemory as i32)) as BOOLEAN
}
unsafe extern "C" fn ArbpForEachRegistryRange(
    Arbiter: PARBITER_INSTANCE,
    Subkey: PCWSTR,
    ValueName: PCWSTR,
    Translate: PARB_TRANSLATE_ORDERING,
    Callback: PARB_RANGE_CALLBACK,
    Context: PVOID,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Value: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    if !NT_SUCCESS(ArbpOpenPolicySubkey(Subkey, &raw mut KeyHandle)) {
        return STATUS_SUCCESS;
    }

    Status = ArbpReadPolicyValue(KeyHandle, ValueName, &raw mut Value);

    /* Follow a REG_SZ indirection to the named sibling value. */
    if NT_SUCCESS(Status) && (*Value).Type == REG_SZ as ULONG {
        let TargetName: PWSTR = (Value as PUCHAR).add((*Value).DataOffset as usize) as PWSTR;
        let TargetChars: ULONG =
            ((*Value).DataLength as usize / core::mem::size_of::<WCHAR>()) as ULONG;
        let mut Target: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();

        if TargetChars != 0 as ULONG
            && *TargetName.add((TargetChars - 1 as ULONG) as usize)
                == UNICODE_NULL as WCHAR
            && NT_SUCCESS(ArbpReadPolicyValue(KeyHandle, TargetName as PCWSTR, &raw mut Target))
        {
            ExFreePoolWithTag(Value as PVOID, TAG_ARBITER as ULONG);
            Value = Target;
        }
    }

    if NT_SUCCESS(Status)
        && (*Value).Type == REG_RESOURCE_REQUIREMENTS_LIST as ULONG
        && (*Value).DataLength as usize >= core::mem::size_of::<IO_RESOURCE_REQUIREMENTS_LIST>() as usize
    {
        let Requirements: PIO_RESOURCE_REQUIREMENTS_LIST =
            (Value as PUCHAR).add((*Value).DataOffset as usize) as PIO_RESOURCE_REQUIREMENTS_LIST;
        let DataEnd: PUCHAR = (Requirements as PUCHAR).add((*Value).DataLength as usize);
        let Alternative: PIO_RESOURCE_LIST =
            (*Requirements).List.as_mut_ptr().add(0 as usize);
        let mut Index: ULONG = 0 as ULONG;

        'lp: while Index < (*Alternative).Count {
            'body: {
                let Descriptor: PIO_RESOURCE_DESCRIPTOR =
                    (*Alternative).Descriptors.as_mut_ptr().add(Index as usize);
                let mut Window: IO_RESOURCE_DESCRIPTOR = core::mem::zeroed();
                let mut Minimum: UINT64 = 0;
                let mut Maximum: UINT64 = 0;
                let mut Length: UINT64 = 0;
                let mut Alignment: UINT64 = 0;

                /* Never trust the stored Count past the actual value data. */
                if (Descriptor.add(1 as usize) as PUCHAR) > DataEnd {
                    break 'lp;
                }

                /*
                 * Policy windows are written in bus-relative terms, so an arbiter
                 * that allocates in a different space translates each one first.
                 * A window it cannot translate comes back CmResourceTypeNull and
                 * is dropped by the type test below.
                 */
                if Translate.is_some() {
                    Status = Translate.unwrap()(&raw mut Window, Descriptor);
                    if !NT_SUCCESS(Status) {
                        break 'lp;
                    }
                } else {
                    Window = *Descriptor;
                }

                if ArbpDescriptorMatchesArbiter(Arbiter, Window.Type) == 0 {
                    break 'body;
                }

                /*
                 * Decode through the arbiter's own callback: the bounds live in a
                 * different union member for every resource type.
                 */
                Status = (*Arbiter).UnpackRequirement.expect("non-null function pointer")(
                    &raw mut Window,
                    &raw mut Minimum,
                    &raw mut Maximum,
                    &raw mut Length,
                    &raw mut Alignment,
                );
                if !NT_SUCCESS(Status) {
                    break 'lp;
                }
                Callback.unwrap()(Arbiter, Context, Minimum, Maximum);
            }
            Index = Index.wrapping_add(1);
        }
    }

    if !Value.is_null() {
        ExFreePoolWithTag(Value as PVOID, TAG_ARBITER as ULONG);
    }
    ZwClose(KeyHandle);
    return STATUS_SUCCESS;
}
unsafe extern "system" fn ArbpAddOrderingCallback(
    Arbiter: PARBITER_INSTANCE,
    Context: PVOID,
    Start: ULONGLONG,
    End: ULONGLONG,
) {
    let _ = Context;
    ArbiterLibAddOrdering(
        core::ptr::addr_of_mut!((*Arbiter).OrderingList),
        Start as UINT64,
        End as UINT64,
    );
}
unsafe extern "system" fn ArbpAddReservedCallback(
    Arbiter: PARBITER_INSTANCE,
    Context: PVOID,
    Start: ULONGLONG,
    End: ULONGLONG,
) {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Context);

    if NT_SUCCESS(ArbiterLibAddOrdering(
        core::ptr::addr_of_mut!((*Arbiter).ReservedList),
        Start as UINT64,
        End as UINT64,
    )) {
        ArbpExcludeOrderingRange(
            core::ptr::addr_of_mut!((*Arbiter).OrderingList),
            Start as UINT64,
            End as UINT64,
        );
    }
}
unsafe extern "system" fn ArbpAddRangeCallback(
    Arbiter: PARBITER_INSTANCE,
    Context: PVOID,
    Start: ULONGLONG,
    End: ULONGLONG,
) {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Arbiter);
    RtlAddRange(Context as PRTL_RANGE_LIST, Start, End, ARBITER_RANGE_INACCESSIBLE as UCHAR,
                RTL_RANGE_LIST_ADD_IF_CONFLICT as ULONG, core::ptr::null_mut(), core::ptr::null_mut());
}
unsafe extern "system" fn ArbpMmConfigCallback(
    Arbiter: PARBITER_INSTANCE,
    Context: PVOID,
    Start: ULONGLONG,
    End: ULONGLONG,
) {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Arbiter);

    ArbpMmConfigStart = Start;
    ArbpMmConfigEnd = End;
    RtlAddRange(
        Context as PRTL_RANGE_LIST,
        Start,
        End,
        (ARBITER_RANGE_BOOT_ALLOCATED as u8),
        RTL_RANGE_LIST_ADD_IF_CONFLICT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibDefaultAssignmentOrdering(
    Arbiter: PARBITER_INSTANCE,
    AllocationOrderName: PCWSTR,
    ReservedResourcesName: PCWSTR,
    TranslateOrderingFunction: PARB_TRANSLATE_ORDERING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    ArbiterLibFreeOrderingList(&raw mut (*Arbiter).OrderingList);
    ArbiterLibFreeOrderingList(&raw mut (*Arbiter).ReservedList);

    Status = ArbpCreateOrderingList(&raw mut (*Arbiter).OrderingList);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    Status = ArbpCreateOrderingList(&raw mut (*Arbiter).ReservedList);
    if !NT_SUCCESS(Status) {
        ArbiterLibFreeOrderingList(&raw mut (*Arbiter).OrderingList);
        return Status;
    }

    ArbpForEachRegistryRange(
        Arbiter,
        RUSTCONV_WSTR_FC252766FB17.as_ptr(),
        AllocationOrderName,
        TranslateOrderingFunction,
        Some(ArbpAddOrderingCallback),
        core::ptr::null_mut(),
    );

    Status = ArbiterLibAddOrdering(
        &raw mut (*Arbiter).OrderingList,
        0,
        ARBITER_MAXIMUM_ADDRESS,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ArbpForEachRegistryRange(
        Arbiter,
        RUSTCONV_WSTR_9CE7561B3242.as_ptr(),
        ReservedResourcesName,
        TranslateOrderingFunction,
        Some(ArbpAddReservedCallback),
        core::ptr::null_mut(),
    )
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibAddInaccessibleAllocationRange(
    Arbiter: PARBITER_INSTANCE,
    OrderingName: PCWSTR,
    RangeList: PRTL_RANGE_LIST,
) -> NTSTATUS {
    PAGED_CODE!();
    return ArbpForEachRegistryRange(
        Arbiter,
        RUSTCONV_WSTR_8FA741F58E83.as_ptr(),
        OrderingName,
        None,
        Some(ArbpAddRangeCallback),
        RangeList as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibAddMmConfigRangeAsBootReserved(
    Arbiter: PARBITER_INSTANCE,
    RangeList: PRTL_RANGE_LIST,
) -> NTSTATUS {
    PAGED_CODE!();
    return ArbpForEachRegistryRange(
        Arbiter,
        RUSTCONV_WSTR_9CE7561B3242.as_ptr(),
        RUSTCONV_WSTR_DBFF551DC4C1.as_ptr(),
        None,
        Some(ArbpMmConfigCallback),
        RangeList as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibIsConflictWithMmConfigRange(Start: ULONGLONG, End: ULONGLONG) -> BOOLEAN {
    if ArbpMmConfigStart > ArbpMmConfigEnd {
        return FALSE;
    }
    (Start <= ArbpMmConfigEnd && ArbpMmConfigStart <= End) as BOOLEAN
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Arbitration Library
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Ordering lists and the registry assignment ordering
//  * COPYRIGHT:   Copyright 2026 Justin Miller <justin.miller@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntifs.h>
// #include <ndk/rtlfuncs.h>
// #include "arbiter.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define ARBITER_ORDERING_GRANULARITY  16
// #define ARBITER_ORDERING_LIMIT        1024
//
// #define ARBP_POLICY_ROOT   L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Arbiters\\"
// #define ARBP_LONGEST_SUBKEY L"ReservedResources"
//
// /*
//  * The PCI-Express enhanced-config (MMCONFIG / ECAM) MMIO window the memory
//  * arbiter must never hand out. Recorded by
//  * ArbiterLibAddMmConfigRangeAsBootReserved; Start > End (the initial state)
//  */
// static ULONGLONG ArbpMmConfigStart = 1;
// static ULONGLONG ArbpMmConfigEnd = 0;
//
// /* ORDERING LISTS *************************************************************/
//
// /* Set up an empty ordering array at its starting capacity. */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpCreateOrderingList(
//           PARBITER_ORDERING_LIST OrderingList)
// {
//     PAGED_CODE();
//
//     OrderingList->Orderings = ExAllocatePoolWithTag(PagedPool,
//                                                     ARBITER_ORDERING_GRANULARITY * sizeof(ARBITER_ORDERING),
//                                                     TAG_ARBITER);
//     if (OrderingList->Orderings == NULL)
//     {
//         OrderingList->Count = 0;
//         OrderingList->Maximum = 0;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     OrderingList->Count = 0;
//     OrderingList->Maximum = ARBITER_ORDERING_GRANULARITY;
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("PAGE")
// VOID
// NTAPI
// ArbiterLibFreeOrderingList(
//             PARBITER_ORDERING_LIST OrderingList)
// {
//     PAGED_CODE();
//
//     if (OrderingList->Orderings != NULL)
//         ExFreePoolWithTag(OrderingList->Orderings, TAG_ARBITER);
//
//     OrderingList->Orderings = NULL;
//     OrderingList->Count = 0;
//     OrderingList->Maximum = 0;
// }
//
// /* Append one [Start, End] window (both bounds inclusive) at the tail
//  * (= lowest preference so far). */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibAddOrdering(
//             PARBITER_ORDERING_LIST OrderingList,
//          UINT64 Start,
//          UINT64 End)
// {
//     PAGED_CODE();
//
//     if (End < Start)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Double the array capacity when it fills up, up to the sanity cap. */
//     if (OrderingList->Count == OrderingList->Maximum)
//     {
//         ULONG NewMaximum;
//         PARBITER_ORDERING NewArray;
//
//         NewMaximum = OrderingList->Maximum ? (ULONG)OrderingList->Maximum * 2
//                                            : ARBITER_ORDERING_GRANULARITY;
//         if (NewMaximum > ARBITER_ORDERING_LIMIT)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         NewArray = ExAllocatePoolWithTag(PagedPool,
//                                          NewMaximum * sizeof(ARBITER_ORDERING),
//                                          TAG_ARBITER);
//         if (NewArray == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         if (OrderingList->Orderings != NULL)
//         {
//             RtlCopyMemory(NewArray, OrderingList->Orderings,
//                           OrderingList->Count * sizeof(ARBITER_ORDERING));
//             ExFreePoolWithTag(OrderingList->Orderings, TAG_ARBITER);
//         }
//
//         OrderingList->Maximum = (UINT16)NewMaximum;
//         OrderingList->Orderings = NewArray;
//     }
//
//     OrderingList->Orderings[OrderingList->Count].Start = Start;
//     OrderingList->Orderings[OrderingList->Count].End = End;
//     OrderingList->Count++;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Removes [Start, End] from every window of an ordering list,
//  * splitting windows that straddle it.
//  *
//  * @param[in,out] OrderingList
//  * The ordering list to prune. The pruned set is built in a
//  * temporary list and swapped in on success; on failure the
//  * original list is left untouched.
//  *
//  * @param[in] Start
//  * The inclusive start of the range to remove.
//  *
//  * @param[in] End
//  * The inclusive end of the range to remove. Must not be below
//  * Start.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, STATUS_INVALID_PARAMETER for a
//  * backwards range, or STATUS_INSUFFICIENT_RESOURCES.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpExcludeOrderingRange(
//             PARBITER_ORDERING_LIST OrderingList,
//          UINT64 Start,
//          UINT64 End)
// {
//     ARBITER_ORDERING_LIST Pruned;
//     NTSTATUS Status;
//     UINT16 Index;
//
//     PAGED_CODE();
//
//     if (End < Start)
//         return STATUS_INVALID_PARAMETER;
//
//     Status = ArbpCreateOrderingList(&Pruned);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     for (Index = 0; Index < OrderingList->Count; ++Index)
//     {
//         UINT64 CurrentStart = OrderingList->Orderings[Index].Start;
//         UINT64 CurrentEnd = OrderingList->Orderings[Index].End;
//
//         Status = STATUS_SUCCESS;
//
//         if (CurrentEnd < Start || CurrentStart > End)
//         {
//             /* Wholly outside the hole: survives unchanged. */
//             Status = ArbiterLibAddOrdering(&Pruned, CurrentStart, CurrentEnd);
//         }
//         else
//         {
//             if (CurrentStart < Start)  /* Left fragment survives */
//                 Status = ArbiterLibAddOrdering(&Pruned, CurrentStart, Start - 1);
//
//             if (NT_SUCCESS(Status) && CurrentEnd > End)  /* Right fragment survives */
//                 Status = ArbiterLibAddOrdering(&Pruned, End + 1, CurrentEnd);
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             ArbiterLibFreeOrderingList(&Pruned);
//             return Status;
//         }
//     }
//
//     ArbiterLibFreeOrderingList(OrderingList);
//     *OrderingList = Pruned;
//     return STATUS_SUCCESS;
// }
//
// /* REGISTRY-DESCRIBED RANGES **************************************************/
//
// /*
//  * An arbiter refines its allocation policy from HKLM\...\Control\Arbiters.
//  * Each value there is a REG_RESOURCE_REQUIREMENTS_LIST whose descriptors give
//  * [MinimumAddress, MaximumAddress] ranges, filtered by the arbiter's resource
//  * This can get written to by chipset drivers.
//  */
//
// typedef VOID
// (NTAPI *PARB_RANGE_CALLBACK)(
//          PARBITER_INSTANCE Arbiter,
//              PVOID Context,
//          ULONGLONG Start,
//          ULONGLONG End);
//
// /* Open one subkey of the Arbiters policy key by composing its full path. */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpOpenPolicySubkey(
//          PCWSTR SubkeyName,
//           PHANDLE KeyHandle)
// {
//     WCHAR PathBuffer[sizeof(ARBP_POLICY_ROOT ARBP_LONGEST_SUBKEY) / sizeof(WCHAR)];
//     UNICODE_STRING KeyPath;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     RtlInitEmptyUnicodeString(&KeyPath, PathBuffer, sizeof(PathBuffer));
//     Status = RtlAppendUnicodeToString(&KeyPath, ARBP_POLICY_ROOT);
//     if (NT_SUCCESS(Status))
//         Status = RtlAppendUnicodeToString(&KeyPath, SubkeyName);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     InitializeObjectAttributes(&ObjectAttributes, &KeyPath,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE, NULL, NULL);
//     return ZwOpenKey(KeyHandle, KEY_READ, &ObjectAttributes);
// }
//
// /* Read one value into a freshly-allocated KEY_VALUE_FULL_INFORMATION. */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpReadPolicyValue(
//          HANDLE KeyHandle,
//          PCWSTR ValueName,
//           PKEY_VALUE_FULL_INFORMATION *Value)
// {
//     UNICODE_STRING NameString;
//     PKEY_VALUE_FULL_INFORMATION Buffer;
//     ULONG Size = 0;
//     ULONG ResultLength = 0;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&NameString, ValueName);
//
//     /* Size probe: only a buffer-size result means the value is readable. */
//     Status = ZwQueryValueKey(KeyHandle, &NameString, KeyValueFullInformationAlign64,
//                              NULL, 0, &Size);
//     if (Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_BUFFER_OVERFLOW)
//         return NT_SUCCESS(Status) ? STATUS_UNSUCCESSFUL : Status;
//
//     Buffer = ExAllocatePoolWithTag(PagedPool, Size, TAG_ARBITER);
//     if (Buffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = ZwQueryValueKey(KeyHandle, &NameString, KeyValueFullInformationAlign64,
//                              Buffer, Size, &ResultLength);
//
//     if (NT_SUCCESS(Status) &&
//         ((ULONGLONG)Buffer->DataOffset + Buffer->DataLength > ResultLength))
//     {
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(Buffer, TAG_ARBITER);
//         return Status;
//     }
//
//     *Value = Buffer;
//     return STATUS_SUCCESS;
// }
//
// /* Does a policy descriptor apply to this arbiter's resource type? */
// CODE_SEG("PAGE")
// static
// BOOLEAN
// ArbpDescriptorMatchesArbiter(
//          PARBITER_INSTANCE Arbiter,
//          UCHAR DescriptorType)
// {
//     PAGED_CODE();
//
//     if (DescriptorType == (UCHAR)Arbiter->ResourceType)
//         return TRUE;
//
//     /* Large-memory descriptors also satisfy the plain memory arbiter. */
//     return (DescriptorType == CmResourceTypeMemoryLarge &&
//             Arbiter->ResourceType == CmResourceTypeMemory);
// }
//
// /*
//  * Walk the policy ranges stored under HKLM\...\Control\Arbiters\<Subkey>,
//  * This is used by machine-specific policy tables.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpForEachRegistryRange(
//          PARBITER_INSTANCE Arbiter,
//          PCWSTR Subkey,
//          PCWSTR ValueName,
//              PARB_TRANSLATE_ORDERING Translate,
//          PARB_RANGE_CALLBACK Callback,
//              PVOID Context)
// {
//     HANDLE KeyHandle;
//     PKEY_VALUE_FULL_INFORMATION Value = NULL;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     if (!NT_SUCCESS(ArbpOpenPolicySubkey(Subkey, &KeyHandle)))
//         return STATUS_SUCCESS;
//
//     Status = ArbpReadPolicyValue(KeyHandle, ValueName, &Value);
//
//     /* Follow a REG_SZ indirection to the named sibling value. */
//     if (NT_SUCCESS(Status) && Value->Type == REG_SZ)
//     {
//         PWSTR TargetName = (PWSTR)((PUCHAR)Value + Value->DataOffset);
//         ULONG TargetChars = Value->DataLength / sizeof(WCHAR);
//         PKEY_VALUE_FULL_INFORMATION Target = NULL;
//
//         if (TargetChars != 0 &&
//             TargetName[TargetChars - 1] == UNICODE_NULL &&
//             NT_SUCCESS(ArbpReadPolicyValue(KeyHandle, TargetName, &Target)))
//         {
//             ExFreePoolWithTag(Value, TAG_ARBITER);
//             Value = Target;
//         }
//     }
//
//     if (NT_SUCCESS(Status) &&
//         Value->Type == REG_RESOURCE_REQUIREMENTS_LIST &&
//         Value->DataLength >= sizeof(IO_RESOURCE_REQUIREMENTS_LIST))
//     {
//         PIO_RESOURCE_REQUIREMENTS_LIST Requirements;
//         PIO_RESOURCE_LIST Alternative;
//         PUCHAR DataEnd;
//         ULONG Index;
//
//         Requirements = (PIO_RESOURCE_REQUIREMENTS_LIST)((PUCHAR)Value + Value->DataOffset);
//         DataEnd = (PUCHAR)Requirements + Value->DataLength;
//         Alternative = &Requirements->List[0];
//
//         for (Index = 0; Index < Alternative->Count; ++Index)
//         {
//             PIO_RESOURCE_DESCRIPTOR Descriptor = &Alternative->Descriptors[Index];
//             IO_RESOURCE_DESCRIPTOR Window;
//             UINT64 Minimum, Maximum, Length, Alignment;
//
//             /* Never trust the stored Count past the actual value data. */
//             if ((PUCHAR)(Descriptor + 1) > DataEnd)
//                 break;
//
//             /*
//              * Policy windows are written in bus-relative terms, so an arbiter
//              * that allocates in a different space translates each one first.
//              * A window it cannot translate comes back CmResourceTypeNull and
//              * is dropped by the type test below.
//              */
//             if (Translate != NULL)
//             {
//                 Status = Translate(&Window, Descriptor);
//                 if (!NT_SUCCESS(Status))
//                     break;
//             }
//             else
//             {
//                 Window = *Descriptor;
//             }
//
//             if (!ArbpDescriptorMatchesArbiter(Arbiter, Window.Type))
//                 continue;
//
//             /*
//              * Decode through the arbiter's own callback: the bounds live in a
//              * different union member for every resource type.
//              */
//             Status = Arbiter->UnpackRequirement(&Window, &Minimum, &Maximum,
//                                                 &Length, &Alignment);
//             if (!NT_SUCCESS(Status))
//
//                 break;
//             Callback(Arbiter, Context, Minimum, Maximum);
//         }
//     }
//
//     if (Value != NULL)
//         ExFreePoolWithTag(Value, TAG_ARBITER);
//     ZwClose(KeyHandle);
//     return STATUS_SUCCESS;
// }
//
// /* Prepend a registry-described preferred range ahead of the full-range default. */
// CODE_SEG("PAGE")
// static
// VOID
// NTAPI
// ArbpAddOrderingCallback(
//          PARBITER_INSTANCE Arbiter,
//              PVOID Context,
//          ULONGLONG Start,
//          ULONGLONG End)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Context);
//     ArbiterLibAddOrdering(&Arbiter->OrderingList, Start, End);
// }
//
// /**
//  * @brief
//  * Range callback moving one reserved window into last-resort
//  * territory: recorded in the arbiter's ReservedList and pruned
//  * out of every ordering window, including the full-range
//  * fallback.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose lists are adjusted.
//  *
//  * @param[in] Context
//  * Unused.
//  *
//  * @param[in] Start
//  * The inclusive start of the reserved window.
//  *
//  * @param[in] End
//  * The inclusive end of the reserved window.
//  *
//  * @remarks
//  * Once pruned here, the window can only ever be offered by the
//  * engine's (PREFERRED_)RESERVED pass, the one that is usable only
//  * when nothing else fits. This is what keeps allocations from touching ranges
//  * like COM1/COM2 and the VGA windows that the PCStandard policy table
//  * reserves, even though no enumerated device owns them yet.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// NTAPI
// ArbpAddReservedCallback(
//          PARBITER_INSTANCE Arbiter,
//              PVOID Context,
//          ULONGLONG Start,
//          ULONGLONG End)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Context);
//
//     if (NT_SUCCESS(ArbiterLibAddOrdering(&Arbiter->ReservedList, Start, End)))
//         ArbpExcludeOrderingRange(&Arbiter->OrderingList, Start, End);
// }
//
// /**
//  * @brief
//  * Range callback carving one window out of a range list as an
//  * unowned blocking range, so it is never handed to a device.
//  *
//  * @param[in] Arbiter
//  * Unused.
//  *
//  * @param[in] Context
//  * The RTL_RANGE_LIST to add the blocking range to.
//  *
//  * @param[in] Start
//  * The inclusive start of the window.
//  *
//  * @param[in] End
//  * The inclusive end of the window.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// NTAPI
// ArbpAddRangeCallback(
//          PARBITER_INSTANCE Arbiter,
//              PVOID Context,
//          ULONGLONG Start,
//          ULONGLONG End)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Arbiter);
//     RtlAddRange((PRTL_RANGE_LIST)Context, Start, End, ARBITER_RANGE_INACCESSIBLE,
//                 RTL_RANGE_LIST_ADD_IF_CONFLICT, NULL, NULL);
// }
//
// /**
//  * @brief
//  * Range callback recording the MMCONFIG (ECAM) window and
//  * reserving it in a range list as a boot allocation.
//  *
//  * @param[in] Arbiter
//  * Unused.
//  *
//  * @param[in] Context
//  * The RTL_RANGE_LIST to reserve the window in.
//  *
//  * @param[in] Start
//  * The inclusive start of the ECAM window.
//  *
//  * @param[in] End
//  * The inclusive end of the ECAM window.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// NTAPI
// ArbpMmConfigCallback(
//          PARBITER_INSTANCE Arbiter,
//              PVOID Context,
//          ULONGLONG Start,
//          ULONGLONG End)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Arbiter);
//
//     ArbpMmConfigStart = Start;
//     ArbpMmConfigEnd = End;
//     RtlAddRange((PRTL_RANGE_LIST)Context, Start, End, ARBITER_RANGE_BOOT_ALLOCATED,
//                 RTL_RANGE_LIST_ADD_IF_CONFLICT, NULL, NULL);
// }
//
// /**
//  * @brief
//  * Builds the arbiter's allocation ordering from the registry
//  * policy, most-preferred window first.
//  *
//  * @param[in,out] Arbiter
//  * The arbiter instance whose OrderingList and ReservedList are
//  * (re)created. Callers may rebuild an existing ordering against a
//  * different policy table (see pcix!ario_ApplyBrokenVideoHack).
//  *
//  * @param[in] AllocationOrderName
//  * The AllocationOrder value naming this arbiter's preferred
//  * windows. The engine walks the resulting list in index order, so
//  * index 0 is the most-preferred window.
//  *
//  * @param[in] ReservedResourcesName
//  * The ReservedResources value naming this arbiter's last-resort
//  * windows: each is recorded in the ReservedList for the engine's
//  * final pass and pruned out of every ordering window, so it is
//  * only ever offered once all orderings fail.
//  *
//  * @param[in] TranslateOrderingFunction
//  * Optional per-arbiter descriptor translation.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the ordering was built. Absent
//  * registry policy is not a failure; the ordering then holds only
//  * the full-range fallback, so the arbiter can always search the
//  * whole space.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibDefaultAssignmentOrdering(
//             PARBITER_INSTANCE Arbiter,
//          PCWSTR AllocationOrderName,
//          PCWSTR ReservedResourcesName,
//              PARB_TRANSLATE_ORDERING TranslateOrderingFunction)
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     ArbiterLibFreeOrderingList(&Arbiter->OrderingList);
//     ArbiterLibFreeOrderingList(&Arbiter->ReservedList);
//
//     Status = ArbpCreateOrderingList(&Arbiter->OrderingList);
//     if (!NT_SUCCESS(Status))
//         return Status;
//     Status = ArbpCreateOrderingList(&Arbiter->ReservedList);
//     if (!NT_SUCCESS(Status))
//     {
//         ArbiterLibFreeOrderingList(&Arbiter->OrderingList);
//         return Status;
//     }
//
//     ArbpForEachRegistryRange(Arbiter, L"AllocationOrder", AllocationOrderName,
//                              TranslateOrderingFunction, ArbpAddOrderingCallback, NULL);
//
//     Status = ArbiterLibAddOrdering(&Arbiter->OrderingList, 0, ARBITER_MAXIMUM_ADDRESS);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /*
//      * The reserved windows come last, after the fallback is in place, so the
//      * pruning punches them out of the fallback too. Otherwise the fallback
//      * pass would hand them out like any other range.
//      */
//     return ArbpForEachRegistryRange(Arbiter, L"ReservedResources", ReservedResourcesName,
//                                     TranslateOrderingFunction, ArbpAddReservedCallback,
//                                     NULL);
// }
//
// /**
//  * @brief
//  * Carves the firmware-reported inaccessible ranges
//  * (Arbiters\InaccessibleRange\<OrderingName>) out of a range list
//  * as unowned blocking ranges, so they are never handed to a
//  * device.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance the ranges are filtered for.
//  *
//  * @param[in] OrderingName
//  * The InaccessibleRange value to read.
//  *
//  * @param[in,out] RangeList
//  * The range list to carve the windows out of.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, including when nothing is recorded
//  * (an absent value reserves nothing).
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibAddInaccessibleAllocationRange(
//          PARBITER_INSTANCE Arbiter,
//          PCWSTR OrderingName,
//             PRTL_RANGE_LIST RangeList)
// {
//     PAGED_CODE();
//     return ArbpForEachRegistryRange(Arbiter, L"InaccessibleRange", OrderingName,
//                                     NULL, ArbpAddRangeCallback, RangeList);
// }
//
// /**
//  * @brief
//  * Reserves the PCI-Express enhanced-config (MMCONFIG / ECAM) MMIO
//  * window (Arbiters\ReservedResources\MmConfigRange) in a range
//  * list, and records it for ArbiterLibIsConflictWithMmConfigRange.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance the window is filtered for.
//  *
//  * @param[in,out] RangeList
//  * The range list to reserve the window in.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, including on a legacy pre-PCIe machine
//  * where the value is simply absent and nothing is reserved.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibAddMmConfigRangeAsBootReserved(
//          PARBITER_INSTANCE Arbiter,
//             PRTL_RANGE_LIST RangeList)
// {
//     PAGED_CODE();
//     return ArbpForEachRegistryRange(Arbiter, L"ReservedResources", L"MmConfigRange",
//                                     NULL, ArbpMmConfigCallback, RangeList);
// }
//
// /**
//  * @brief
//  * Determines whether a range overlaps the recorded MMCONFIG
//  * window.
//  *
//  * @param[in] Start
//  * The inclusive start of the range to test.
//  *
//  * @param[in] End
//  * The inclusive end of the range to test.
//  *
//  * @return
//  * Returns TRUE if [Start, End] overlaps the recorded window,
//  * FALSE otherwise or when no window was ever recorded.
//  */
// BOOLEAN
// NTAPI
// ArbiterLibIsConflictWithMmConfigRange(
//          ULONGLONG Start,
//          ULONGLONG End)
// {
//     if (ArbpMmConfigStart > ArbpMmConfigEnd)
//         return FALSE;  /* No MMCONFIG window recorded */
//
//     return (BOOLEAN)(Start <= ArbpMmConfigEnd && ArbpMmConfigStart <= End);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
