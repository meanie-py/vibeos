//! rw.rs — rw.c: MsfsRead (server dequeues a datagram, else parks on the CSQ with an
//! optional timeout) and MsfsWrite (client enqueues a datagram and wakes a parked reader).
#![allow(non_snake_case)]

use core::cmp::min;
use core::mem::{size_of, transmute};
use core::ptr::{addr_of_mut, copy_nonoverlapping, null_mut};

use ntkrnl_sys::base::{InsertTailList, LONG, PVOID, ULONG, ULONG_PTR};
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};
use ntkrnl_sys::ddk::{
    IoCsqInsertIrp, IoCsqRemoveNextIrp, KeCancelTimer, KeInitializeDpc,
    KeInitializeEvent, KeInitializeTimer, KeSetTimer, KeWaitForSingleObject,
    PDEVICE_OBJECT, PIRP, PKDEFERRED_ROUTINE, PKDPC, KIRQL, NTSTATUS,
};
use ntkrnl_sys::base::RemoveHeadList;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe;
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending};

use crate::glue::*;
use crate::helpers::irp_driver_context0;
use crate::msfssup::MsfsTimeout;
use crate::types::*;
use crate::CONTAINING_RECORD;

// MsfsTimeout is KDEFERRED_ROUTINE (NTAPI); transmute to the bindgen slot type.
type DpcFn = unsafe extern "system" fn(PKDPC, PVOID, PVOID, PVOID);

pub unsafe extern "system" fn MsfsRead(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _ = DeviceObject;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let FileObject = (*IoStack).FileObject;
    let Fcb = (*FileObject).FsContext as PMSFS_FCB;
    let Ccb = (*FileObject).FsContext2 as PMSFS_CCB;

    /* reading is not permitted on client side */
    if (*Fcb).ServerCcb != Ccb {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_ACCESS_DENIED;
    }

    let Length = (*IoStack).Parameters.Read.Length;
    let Buffer: PVOID = if !(*Irp).MdlAddress.is_null() {
        MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority)
    } else {
        (*Irp).UserBuffer
    };

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, addr_of_mut!(oldIrql));
    if (*Fcb).MessageCount > 0 {
        let Entry = RemoveHeadList(addr_of_mut!((*Fcb).MessageListHead));
        (*Fcb).MessageCount -= 1;
        KeReleaseSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, oldIrql);

        /* copy current message into buffer */
        let Message = CONTAINING_RECORD!(Entry, MSFS_MESSAGE, MessageListEntry);
        copy_nonoverlapping(
            (*Message).Buffer.as_ptr(),
            Buffer as *mut u8,
            min((*Message).Size, Length) as usize,
        );
        let LengthRead = (*Message).Size;

        ExFreePoolWithTag(Message as _, TAG_MSG);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = LengthRead as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    } else {
        KeReleaseSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, oldIrql);
    }

    let Timeout = (*Fcb).TimeOut;
    if Timeout.__bindgen_anon_1.HighPart == 0 && Timeout.__bindgen_anon_1.LowPart == 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_IO_TIMEOUT;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_IO_TIMEOUT;
    }

    let Context = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<MSFS_DPC_CTX>() as ULONG_PTR,
        TAG_NAME,
    ) as PMSFS_DPC_CTX;
    if Context.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeEvent(addr_of_mut!((*Context).Event), SynchronizationEvent, FALSE);
    IoCsqInsertIrp(
        addr_of_mut!((*Fcb).CancelSafeQueue),
        Irp,
        addr_of_mut!((*Context).CsqContext),
    );
    let Timer = addr_of_mut!((*Context).Timer);
    let Dpc = addr_of_mut!((*Context).Dpc);
    (*Context).Csq = addr_of_mut!((*Fcb).CancelSafeQueue);
    *irp_driver_context0(Irp) = Context as PVOID;

    /* No timer for INFINITY_WAIT */
    if Timeout.QuadPart != -1 {
        KeInitializeTimer(Timer);
        KeInitializeDpc(
            Dpc,
            transmute::<DpcFn, PKDEFERRED_ROUTINE>(MsfsTimeout),
            Context as PVOID,
        );
        KeSetTimer(Timer, Timeout, Dpc);
    }

    IoMarkIrpPending(Irp);

    STATUS_PENDING
}

pub unsafe extern "system" fn MsfsWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let FileObject = (*IoStack).FileObject;
    let Fcb = (*FileObject).FsContext as PMSFS_FCB;
    let Ccb = (*FileObject).FsContext2 as PMSFS_CCB;

    /* writing is not permitted on server side */
    if (*Fcb).ServerCcb == Ccb {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_ACCESS_DENIED;
    }

    let Length = (*IoStack).Parameters.Write.Length;
    let Buffer: PVOID = if !(*Irp).MdlAddress.is_null() {
        MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority)
    } else {
        (*Irp).UserBuffer
    };

    /* Allocate new message */
    let Message = ExAllocatePoolWithTag(
        NonPagedPool,
        (size_of::<MSFS_MESSAGE>() + Length as usize) as ULONG_PTR,
        TAG_MSG,
    ) as PMSFS_MESSAGE;
    if Message.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    (*Message).Size = Length;
    copy_nonoverlapping(Buffer as *const u8, (*Message).Buffer.as_mut_ptr(), Length as usize);

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, addr_of_mut!(oldIrql));
    InsertTailList(
        addr_of_mut!((*Fcb).MessageListHead),
        addr_of_mut!((*Message).MessageListEntry),
    );
    (*Fcb).MessageCount += 1;
    KeReleaseSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, oldIrql);

    let CsqIrp = IoCsqRemoveNextIrp(addr_of_mut!((*Fcb).CancelSafeQueue), null_mut());
    if !CsqIrp.is_null() {
        /* Get the context */
        let Context = *irp_driver_context0(CsqIrp) as PMSFS_DPC_CTX;
        /* DPC was queued, wait for it to fail (IRP is ours) */
        if (*Fcb).TimeOut.QuadPart != -1 && KeCancelTimer(addr_of_mut!((*Context).Timer)) == 0 {
            KeWaitForSingleObject(
                addr_of_mut!((*Context).Event) as PVOID,
                Executive,
                KernelMode,
                FALSE,
                null_mut(),
            );
        }

        /* Free context & attempt read */
        ExFreePoolWithTag(Context as _, TAG_NAME);
        MsfsRead(DeviceObject, CsqIrp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = Length as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    STATUS_SUCCESS
}
