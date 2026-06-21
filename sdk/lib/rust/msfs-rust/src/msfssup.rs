//! msfssup.rs — msfssup.c: the IO_CSQ cancel-safe-queue callbacks (insert / remove / peek /
//! acquire-lock / release-lock / complete-canceled) plus the MsfsTimeout DPC that fails a
//! parked read after its mailslot ReadTimeout elapses.
#![allow(non_snake_case)]

use core::ptr::null_mut;

use ntkrnl_sys::base::{InsertTailList, RemoveEntryList, LONG, PVOID};
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};
use ntkrnl_sys::ddk::{
    IoCsqRemoveIrp, PFILE_OBJECT, PIO_CSQ, PIRP, PKDPC, PKIRQL, KIRQL,
};
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::ex::ExFreePoolWithTag;
use ntkrnl_sys::ke::KeSetEvent;
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::helpers::{irp_from_list_entry, irp_list_entry};
use crate::types::*;
use crate::CONTAINING_RECORD;

pub unsafe extern "system" fn MsfsInsertIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let Fcb = CONTAINING_RECORD!(Csq, MSFS_FCB, CancelSafeQueue);
    InsertTailList(addr_of_pending_queue(Fcb), irp_list_entry(Irp));
}

pub unsafe extern "system" fn MsfsRemoveIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let _ = Csq;
    RemoveEntryList(irp_list_entry(Irp));
}

pub unsafe extern "system" fn MsfsPeekNextIrp(
    Csq: PIO_CSQ,
    Irp: PIRP,
    PeekContext: PVOID,
) -> PIRP {
    let Fcb = CONTAINING_RECORD!(Csq, MSFS_FCB, CancelSafeQueue);
    let ListHead = addr_of_pending_queue(Fcb);

    let mut NextIrp: PIRP = null_mut();
    let mut NextEntry = if Irp.is_null() {
        (*ListHead).Flink
    } else {
        (*irp_list_entry(Irp)).Flink
    };

    while NextEntry != ListHead {
        NextIrp = irp_from_list_entry(NextEntry);
        let Stack = IoGetCurrentIrpStackLocation(NextIrp);

        if !PeekContext.is_null() {
            if (*Stack).FileObject == PeekContext as PFILE_OBJECT {
                break;
            }
        } else {
            break;
        }

        NextIrp = null_mut();
        NextEntry = (*NextEntry).Flink;
    }

    NextIrp
}

pub unsafe extern "system" fn MsfsAcquireLock(Csq: PIO_CSQ, Irql: PKIRQL) {
    let Fcb = CONTAINING_RECORD!(Csq, MSFS_FCB, CancelSafeQueue);
    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Fcb).QueueLock) as *mut _, Irql);
}

pub unsafe extern "system" fn MsfsReleaseLock(Csq: PIO_CSQ, Irql: KIRQL) {
    let Fcb = CONTAINING_RECORD!(Csq, MSFS_FCB, CancelSafeQueue);
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Fcb).QueueLock) as *mut _, Irql);
}

pub unsafe extern "system" fn MsfsCompleteCanceledIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let _ = Csq;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
}

pub unsafe extern "system" fn MsfsTimeout(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let _ = (Dpc, SystemArgument1, SystemArgument2);
    let Context = DeferredContext as PMSFS_DPC_CTX;

    /* Try to get the IRP */
    let Irp = IoCsqRemoveIrp((*Context).Csq, core::ptr::addr_of_mut!((*Context).CsqContext));
    if !Irp.is_null() {
        /* It timed out, complete it (it's ours) and free context */
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_TIMEOUT;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        ExFreePoolWithTag(Context as _, TAG_NAME);
    } else {
        /* We were racing with writing and failed, signal we're done */
        KeSetEvent(
            core::ptr::addr_of_mut!((*Context).Event) as PVOID,
            IO_NO_INCREMENT as LONG,
            FALSE,
        );
    }
}

#[inline(always)]
unsafe fn addr_of_pending_queue(Fcb: PMSFS_FCB) -> *mut ntkrnl_sys::ddk::LIST_ENTRY {
    core::ptr::addr_of_mut!((*Fcb).PendingIrpQueue)
}
