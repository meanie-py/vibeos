//! waitsup.rs — waitsup.c: Pipes Waiting Support (the FSCTL_PIPE_WAIT wait queue).
//!
//! NpAddWaiter parks a wait IRP with a KTIMER+KDPC; NpTimerDispatch is the KDEFERRED_ROUTINE
//! that fires on timeout; NpCancelWaitQueueIrp is the DRIVER_CANCEL; NpCancelWaiter walks the
//! list satisfying matching waits. Faithful translation of the ReactOS C source.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

// Explicit imports to disambiguate the globs ---------------------------------
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock, KeBugCheckEx};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSetCancelRoutine};
use ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::ob::{ObReferenceObject, ObDereferenceObject};
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use core::mem::transmute;

// NpTimerDispatch is a KDEFERRED_ROUTINE (NTAPI); transmute the fn item to the bindgen slot
// type for KeInitializeDpc (contract §9).
type DpcFn = unsafe extern "system" fn(PKDPC, PVOID, PVOID, PVOID);
// NpCancelWaitQueueIrp is a DRIVER_CANCEL (NTAPI); transmute to PDRIVER_CANCEL for
// IoSetCancelRoutine.
type CancelFn = unsafe extern "system" fn(PDEVICE_OBJECT, PIRP);

// POOL_QUOTA_FAIL_INSTEAD_OF_RAISE — not in glue.rs / bindgen.
// sdk/include/xdk/iotypes.h:17: #define POOL_QUOTA_FAIL_INSTEAD_OF_RAISE 0x8
const POOL_QUOTA_FAIL_INSTEAD_OF_RAISE: ntkrnl_sys::ddk::POOL_TYPE = 0x8;

/// &Irp->Tail.Overlay.DriverContext[i] (bindgen anon-union path).
#[inline(always)]
unsafe fn irp_driver_context(Irp: PIRP, i: usize) -> *mut PVOID {
    addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[i])
}

pub unsafe extern "system" fn NpCancelWaitQueueIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let mut OldIrql: KIRQL = 0;
    let mut WaitEntry: PNP_WAIT_QUEUE_ENTRY;
    let WaitQueue: PNP_WAIT_QUEUE;

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    WaitQueue = *irp_driver_context(Irp, 0) as PNP_WAIT_QUEUE;

    KeAcquireSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _, addr_of_mut!(OldIrql));

    WaitEntry = *irp_driver_context(Irp, 1) as PNP_WAIT_QUEUE_ENTRY;
    if !WaitEntry.is_null() {
        RemoveEntryList(irp_list_entry(Irp));
        if KeCancelTimer(addr_of_mut!((*WaitEntry).Timer)) == 0 {
            (*WaitEntry).Irp = null_mut();
            WaitEntry = null_mut();
        }
    }

    KeReleaseSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _, OldIrql);

    if !WaitEntry.is_null() {
        ObDereferenceObject((*WaitEntry).FileObject as PVOID);
        ExFreePool(WaitEntry as PVOID);
    }

    (*Irp).IoStatus.Information = 0 as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
}

pub unsafe extern "system" fn NpTimerDispatch(
    Dpc: PKDPC,
    Context: PVOID,
    Argument1: PVOID,
    Argument2: PVOID,
) {
    let mut Irp: PIRP;
    let mut OldIrql: KIRQL = 0;
    let WaitEntry: PNP_WAIT_QUEUE_ENTRY = Context as PNP_WAIT_QUEUE_ENTRY;

    KeAcquireSpinLock(
        addr_of_mut!((*(*WaitEntry).WaitQueue).WaitLock) as *mut _,
        addr_of_mut!(OldIrql),
    );

    Irp = (*WaitEntry).Irp;
    if !Irp.is_null() {
        RemoveEntryList(irp_list_entry(Irp));

        if IoSetCancelRoutine(Irp, None).is_none() {
            *irp_driver_context(Irp, 1) = null_mut();
            Irp = null_mut();
        }
    }

    KeReleaseSpinLock(
        addr_of_mut!((*(*WaitEntry).WaitQueue).WaitLock) as *mut _,
        OldIrql,
    );

    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_TIMEOUT;
        IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    }

    ObDereferenceObject((*WaitEntry).FileObject as PVOID);
    ExFreePool(WaitEntry as PVOID);
}

pub unsafe extern "system" fn NpInitializeWaitQueue(WaitQueue: PNP_WAIT_QUEUE) {
    InitializeListHead(addr_of_mut!((*WaitQueue).WaitList));
    KeInitializeSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _);
}

unsafe fn NpEqualUnicodeString(
    String1: *const UNICODE_STRING,
    String2: *const UNICODE_STRING,
) -> BOOLEAN {
    let EqualLength: SIZE_T;

    if (*String1).Length != (*String2).Length {
        return FALSE;
    }

    EqualLength = RtlCompareMemory(
        (*String1).Buffer as *const _,
        (*String2).Buffer as *const _,
        (*String1).Length as _,
    );
    (EqualLength == (*String1).Length as _) as BOOLEAN
}

pub unsafe extern "system" fn NpCancelWaiter(
    WaitQueue: PNP_WAIT_QUEUE,
    PipePath: *mut UNICODE_STRING,
    Status: NTSTATUS,
    List: *mut LIST_ENTRY,
) -> NTSTATUS {
    let mut PipePathUpper: UNICODE_STRING = core::mem::zeroed();
    let mut OldIrql: KIRQL = 0;
    let Buffer: *mut u16;
    let mut NextEntry: *mut LIST_ENTRY;
    let mut WaitEntry: PNP_WAIT_QUEUE_ENTRY;
    let mut Linkage: PNP_WAIT_QUEUE_ENTRY;
    let mut WaitIrp: PIRP;
    let mut WaitBuffer: PFILE_PIPE_WAIT_FOR_BUFFER;
    let mut WaitName: UNICODE_STRING = core::mem::zeroed();
    let mut PipeName: UNICODE_STRING = core::mem::zeroed();

    Linkage = null_mut();

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (*PipePath).Length as _,
        NPFS_WAIT_BLOCK_TAG,
    ) as *mut u16;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlInitEmptyUnicodeString(addr_of_mut!(PipePathUpper), Buffer, (*PipePath).Length);
    RtlUpcaseUnicodeString(addr_of_mut!(PipePathUpper), PipePath as *const _, FALSE);

    KeAcquireSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _, addr_of_mut!(OldIrql));

    NextEntry = (*WaitQueue).WaitList.Flink;
    while NextEntry != addr_of_mut!((*WaitQueue).WaitList) {
        WaitIrp = irp_from_list_entry(NextEntry);
        NextEntry = (*NextEntry).Flink;
        WaitEntry = *irp_driver_context(WaitIrp, 1) as PNP_WAIT_QUEUE_ENTRY;

        if (*WaitEntry).AliasName.Length != 0 {
            // ASSERT(FALSE);
            /* We have an alias. Use that for comparison */
            WaitName = (*WaitEntry).AliasName;
            PipeName = PipePathUpper;
        } else {
            /* Use the name from the wait buffer to compare */
            WaitBuffer = (*WaitIrp).AssociatedIrp.SystemBuffer as PFILE_PIPE_WAIT_FOR_BUFFER;
            WaitName.Buffer = (*WaitBuffer).Name.as_mut_ptr();
            WaitName.Length = (*WaitBuffer).NameLength as USHORT;
            WaitName.MaximumLength = WaitName.Length;

            /* WaitName doesn't have a leading backslash,
             * so skip the one in PipePathUpper for the comparison */
            PipeName.Buffer = PipePathUpper.Buffer.add(1);
            PipeName.Length = PipePathUpper.Length - core::mem::size_of::<u16>() as USHORT;
            PipeName.MaximumLength = PipeName.Length;
        }

        /* Can't use RtlEqualUnicodeString with a spinlock held */
        if NpEqualUnicodeString(addr_of!(WaitName), addr_of!(PipeName)) != FALSE {
            /* Found a matching wait. Cancel it */
            RemoveEntryList(irp_list_entry(WaitIrp));
            if KeCancelTimer(addr_of_mut!((*WaitEntry).Timer)) != 0 {
                (*WaitEntry).WaitQueue = Linkage as PNP_WAIT_QUEUE;
                Linkage = WaitEntry;
            } else {
                (*WaitEntry).Irp = null_mut();
                *irp_driver_context(WaitIrp, 1) = null_mut();
            }

            if IoSetCancelRoutine(WaitIrp, None).is_some() {
                (*WaitIrp).IoStatus.Information = 0 as _;
                (*WaitIrp).IoStatus.__bindgen_anon_1.Status = Status;
                InsertTailList(List, irp_list_entry(WaitIrp));
            } else {
                *irp_driver_context(WaitIrp, 1) = null_mut();
            }
        }
    }

    KeReleaseSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _, OldIrql);

    ExFreePoolWithTag(Buffer as PVOID, NPFS_WAIT_BLOCK_TAG);

    while !Linkage.is_null() {
        WaitEntry = Linkage;
        Linkage = (*Linkage).WaitQueue as PNP_WAIT_QUEUE_ENTRY;
        ObDereferenceObject((*WaitEntry).FileObject as PVOID);
        ExFreePool(WaitEntry as PVOID);
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpAddWaiter(
    WaitQueue: PNP_WAIT_QUEUE,
    WaitTime: LARGE_INTEGER,
    Irp: PIRP,
    AliasName: *mut UNICODE_STRING,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;
    let mut WaitEntry: PNP_WAIT_QUEUE_ENTRY;
    let WaitBuffer: PFILE_PIPE_WAIT_FOR_BUFFER;
    let mut DueTime: LARGE_INTEGER;
    let mut i: ULONG;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    WaitEntry = ExAllocatePoolWithQuotaTag(
        NonPagedPool | POOL_QUOTA_FAIL_INSTEAD_OF_RAISE,
        core::mem::size_of::<NP_WAIT_QUEUE_ENTRY>() as _,
        NPFS_WRITE_BLOCK_TAG,
    ) as PNP_WAIT_QUEUE_ENTRY;
    if WaitEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeDpc(
        addr_of_mut!((*WaitEntry).Dpc),
        transmute::<DpcFn, PKDEFERRED_ROUTINE>(NpTimerDispatch),
        WaitEntry as PVOID,
    );
    KeInitializeTimer(addr_of_mut!((*WaitEntry).Timer));

    if !AliasName.is_null() {
        (*WaitEntry).AliasName = *AliasName;
    } else {
        (*WaitEntry).AliasName.Length = 0;
        (*WaitEntry).AliasName.Buffer = null_mut();
    }

    (*WaitEntry).WaitQueue = WaitQueue;
    (*WaitEntry).Irp = Irp;

    WaitBuffer = (*Irp).AssociatedIrp.SystemBuffer as PFILE_PIPE_WAIT_FOR_BUFFER;
    if (*WaitBuffer).TimeoutSpecified != FALSE {
        DueTime = (*WaitBuffer).Timeout;
    } else {
        DueTime = WaitTime;
    }

    i = 0;
    // (*WaitBuffer).Name is a [WCHAR; 1] flexible-array member; the real buffer extends past
    // it in the allocation. Use raw pointer arithmetic to avoid a Rust bounds-check panic.
    let NamePtr = (*WaitBuffer).Name.as_mut_ptr();
    while i < (*WaitBuffer).NameLength / core::mem::size_of::<u16>() as ULONG {
        let p = NamePtr.add(i as usize);
        *p = RtlUpcaseUnicodeChar(*p);
        i += 1;
    }

    *irp_driver_context(Irp, 0) = WaitQueue as PVOID;
    *irp_driver_context(Irp, 1) = WaitEntry as PVOID;

    KeAcquireSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _, addr_of_mut!(OldIrql));

    IoSetCancelRoutine(Irp, transmute::<CancelFn, PDRIVER_CANCEL>(NpCancelWaitQueueIrp));

    if (*Irp).Cancel != FALSE && IoSetCancelRoutine(Irp, None).is_some() {
        Status = STATUS_CANCELLED;
    } else {
        InsertTailList(addr_of_mut!((*WaitQueue).WaitList), irp_list_entry(Irp));

        IoMarkIrpPending(Irp);
        Status = STATUS_PENDING;

        (*WaitEntry).FileObject = (*IoStack).FileObject;
        ObReferenceObject((*WaitEntry).FileObject as PVOID);

        KeSetTimer(
            addr_of_mut!((*WaitEntry).Timer),
            DueTime,
            addr_of_mut!((*WaitEntry).Dpc),
        );
        WaitEntry = null_mut();
    }

    KeReleaseSpinLock(addr_of_mut!((*WaitQueue).WaitLock) as *mut _, OldIrql);
    if !WaitEntry.is_null() {
        ExFreePool(WaitEntry as PVOID);
    }

    Status
}

/* EOF */
