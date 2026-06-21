//! create.rs — create.c: MsfsCreate (client open), MsfsCreateMailslot (server create),
//! MsfsClose. Faithful translation; the KeLockMutex/KeUnlockMutex macros lower to
//! KeWaitForSingleObject / KeReleaseMutex (helpers.rs).
#![allow(non_snake_case)]

use core::mem::size_of;
use core::ptr::{addr_of_mut, null_mut};

use ntkrnl_sys::base::{
    InitializeListHead, InsertTailList, RemoveEntryList, ULONG, ULONG_PTR,
};
use ntkrnl_sys::base::PVOID;
use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::ddk::{
    IoCsqInitialize, PDEVICE_OBJECT, PIRP, PIO_CSQ, PIO_CSQ_ACQUIRE_LOCK, PIO_CSQ_COMPLETE_CANCELED_IRP,
    PIO_CSQ_INSERT_IRP, PIO_CSQ_PEEK_NEXT_IRP, PIO_CSQ_RELEASE_LOCK, PIO_CSQ_REMOVE_IRP,
    PKIRQL, PMAILSLOT_CREATE_PARAMETERS, RtlCompareUnicodeString, RtlCopyUnicodeString,
    KIRQL, NTSTATUS,
};
use core::mem::transmute;

// The msfs IO_CSQ callbacks are NTAPI; `extern "system"` matches per arch. The bindgen
// PIO_CSQ_* slot types are `Option<extern "C" fn>`, so transmute each fn item to its slot
// type (null-niche compatible — same as the DispatchFn pattern in entry.rs / mup-rust).
type CsqInsert = unsafe extern "system" fn(PIO_CSQ, PIRP);
type CsqRemove = unsafe extern "system" fn(PIO_CSQ, PIRP);
type CsqPeek = unsafe extern "system" fn(PIO_CSQ, PIRP, PVOID) -> PIRP;
type CsqAcquire = unsafe extern "system" fn(PIO_CSQ, PKIRQL);
type CsqRelease = unsafe extern "system" fn(PIO_CSQ, KIRQL);
type CsqComplete = unsafe extern "system" fn(PIO_CSQ, PIRP);
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;

use crate::glue::*;
use crate::helpers::{KeLockMutex, KeUnlockMutex};
use crate::msfssup::{
    MsfsAcquireLock, MsfsCompleteCanceledIrp, MsfsInsertIrp, MsfsPeekNextIrp,
    MsfsReleaseLock, MsfsRemoveIrp,
};
use crate::types::*;
use crate::CONTAINING_RECORD;

/// Creates the client side.
pub unsafe extern "system" fn MsfsCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension = (*DeviceObject).DeviceExtension as PMSFS_DEVICE_EXTENSION;
    let FileObject = (*IoStack).FileObject;

    let Ccb = ExAllocatePoolWithTag(NonPagedPool, size_of::<MSFS_CCB>() as ULONG_PTR, TAG_CCB)
        as PMSFS_CCB;
    if Ccb.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    KeLockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));
    let mut current: PMSFS_FCB = null_mut();
    let mut current_entry = (*DeviceExtension).FcbListHead.Flink;
    while current_entry != addr_of_mut!((*DeviceExtension).FcbListHead) {
        current = CONTAINING_RECORD!(current_entry, MSFS_FCB, FcbListEntry);
        if RtlCompareUnicodeString(
            addr_of_mut!((*FileObject).FileName),
            addr_of_mut!((*current).Name),
            TRUE,
        ) == 0
        {
            break;
        }
        current_entry = (*current_entry).Flink;
    }

    if current_entry == addr_of_mut!((*DeviceExtension).FcbListHead) {
        ExFreePoolWithTag(Ccb as _, TAG_CCB);
        KeUnlockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_UNSUCCESSFUL;
    }

    let Fcb = current;

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _, addr_of_mut!(oldIrql));
    InsertTailList(
        addr_of_mut!((*Fcb).CcbListHead),
        addr_of_mut!((*Ccb).CcbListEntry),
    );
    KeReleaseSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _, oldIrql);

    (*Fcb).ReferenceCount += 1;
    (*Ccb).Fcb = Fcb;

    KeUnlockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));

    (*FileObject).FsContext = Fcb as _;
    (*FileObject).FsContext2 = Ccb as _;
    (*FileObject).Flags |= FO_MAILSLOT;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    STATUS_SUCCESS
}

/// Creates the server side.
pub unsafe extern "system" fn MsfsCreateMailslot(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension = (*DeviceObject).DeviceExtension as PMSFS_DEVICE_EXTENSION;
    let FileObject = (*IoStack).FileObject;
    let Buffer: PMAILSLOT_CREATE_PARAMETERS = (*IoStack).Parameters.CreateMailslot.Parameters;

    let Fcb = ExAllocatePoolWithTag(NonPagedPool, size_of::<MSFS_FCB>() as ULONG_PTR, TAG_FCB)
        as PMSFS_FCB;
    if Fcb.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    (*Fcb).Name.Length = (*FileObject).FileName.Length;
    (*Fcb).Name.MaximumLength = (*Fcb).Name.Length + UNICODE_NULL_SIZE;
    (*Fcb).Name.Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (*Fcb).Name.MaximumLength as ULONG_PTR,
        TAG_NAME,
    ) as _;
    if (*Fcb).Name.Buffer.is_null() {
        ExFreePoolWithTag(Fcb as _, TAG_FCB);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    RtlCopyUnicodeString(addr_of_mut!((*Fcb).Name), addr_of_mut!((*FileObject).FileName));

    let Ccb = ExAllocatePoolWithTag(NonPagedPool, size_of::<MSFS_CCB>() as ULONG_PTR, TAG_CCB)
        as PMSFS_CCB;
    if Ccb.is_null() {
        ExFreePoolWithTag((*Fcb).Name.Buffer as _, TAG_NAME);
        ExFreePoolWithTag(Fcb as _, TAG_FCB);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    (*Fcb).ReferenceCount = 0;
    InitializeListHead(addr_of_mut!((*Fcb).CcbListHead));
    KeInitializeSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _);

    (*Fcb).MaxMessageSize = (*Buffer).MaximumMessageSize;
    (*Fcb).MessageCount = 0;
    (*Fcb).TimeOut = (*Buffer).ReadTimeout;

    InitializeListHead(addr_of_mut!((*Fcb).MessageListHead));
    KeInitializeSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _);

    KeInitializeSpinLock(addr_of_mut!((*Fcb).QueueLock) as *mut _);
    InitializeListHead(addr_of_mut!((*Fcb).PendingIrpQueue));
    IoCsqInitialize(
        addr_of_mut!((*Fcb).CancelSafeQueue),
        transmute::<CsqInsert, PIO_CSQ_INSERT_IRP>(MsfsInsertIrp),
        transmute::<CsqRemove, PIO_CSQ_REMOVE_IRP>(MsfsRemoveIrp),
        transmute::<CsqPeek, PIO_CSQ_PEEK_NEXT_IRP>(MsfsPeekNextIrp),
        transmute::<CsqAcquire, PIO_CSQ_ACQUIRE_LOCK>(MsfsAcquireLock),
        transmute::<CsqRelease, PIO_CSQ_RELEASE_LOCK>(MsfsReleaseLock),
        transmute::<CsqComplete, PIO_CSQ_COMPLETE_CANCELED_IRP>(MsfsCompleteCanceledIrp),
    );

    KeLockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));
    let mut current: PMSFS_FCB = null_mut();
    let mut current_entry = (*DeviceExtension).FcbListHead.Flink;
    while current_entry != addr_of_mut!((*DeviceExtension).FcbListHead) {
        current = CONTAINING_RECORD!(current_entry, MSFS_FCB, FcbListEntry);
        if RtlCompareUnicodeString(
            addr_of_mut!((*Fcb).Name),
            addr_of_mut!((*current).Name),
            TRUE,
        ) == 0
        {
            break;
        }
        current_entry = (*current_entry).Flink;
    }

    if current_entry != addr_of_mut!((*DeviceExtension).FcbListHead) {
        ExFreePoolWithTag((*Fcb).Name.Buffer as _, TAG_NAME);
        ExFreePoolWithTag(Fcb as _, TAG_FCB);
        ExFreePoolWithTag(Ccb as _, TAG_CCB);
        KeUnlockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_UNSUCCESSFUL;
    } else {
        InsertTailList(
            addr_of_mut!((*DeviceExtension).FcbListHead),
            addr_of_mut!((*Fcb).FcbListEntry),
        );
    }

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _, addr_of_mut!(oldIrql));
    InsertTailList(
        addr_of_mut!((*Fcb).CcbListHead),
        addr_of_mut!((*Ccb).CcbListEntry),
    );
    KeReleaseSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _, oldIrql);

    (*Fcb).ReferenceCount += 1;
    (*Fcb).ServerCcb = Ccb;
    (*Ccb).Fcb = Fcb;

    KeUnlockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));

    (*FileObject).FsContext = Fcb as _;
    (*FileObject).FsContext2 = Ccb as _;
    (*FileObject).Flags |= FO_MAILSLOT;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    STATUS_SUCCESS
}

pub unsafe extern "system" fn MsfsClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let _ = DeviceObject;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let DeviceExtension = (*DeviceObject).DeviceExtension as PMSFS_DEVICE_EXTENSION;
    let FileObject = (*IoStack).FileObject;

    KeLockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));

    if (*DeviceExtension).FcbListHead.Flink == addr_of_mut!((*DeviceExtension).FcbListHead) {
        KeUnlockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_SUCCESS;
    }

    let Fcb = (*FileObject).FsContext as PMSFS_FCB;
    let Ccb = (*FileObject).FsContext2 as PMSFS_CCB;

    (*Fcb).ReferenceCount -= 1;
    if (*Fcb).ServerCcb == Ccb {
        /* delete all messages from message-list */
        let mut oldIrql: KIRQL = 0;
        KeAcquireSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, addr_of_mut!(oldIrql));

        while (*Fcb).MessageListHead.Flink != addr_of_mut!((*Fcb).MessageListHead) {
            let Message =
                CONTAINING_RECORD!((*Fcb).MessageListHead.Flink, MSFS_MESSAGE, MessageListEntry);
            RemoveEntryList((*Fcb).MessageListHead.Flink);
            ExFreePoolWithTag(Message as _, TAG_MSG);
        }

        (*Fcb).MessageCount = 0;
        KeReleaseSpinLock(addr_of_mut!((*Fcb).MessageListLock) as *mut _, oldIrql);
        (*Fcb).ServerCcb = null_mut();
    }

    let mut oldIrql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _, addr_of_mut!(oldIrql));
    RemoveEntryList(addr_of_mut!((*Ccb).CcbListEntry));
    KeReleaseSpinLock(addr_of_mut!((*Fcb).CcbListLock) as *mut _, oldIrql);
    ExFreePoolWithTag(Ccb as _, TAG_CCB);
    (*FileObject).FsContext2 = null_mut();

    if (*Fcb).ReferenceCount == 0 {
        RemoveEntryList(addr_of_mut!((*Fcb).FcbListEntry));
        ExFreePoolWithTag((*Fcb).Name.Buffer as _, TAG_NAME);
        ExFreePoolWithTag(Fcb as _, TAG_FCB);
    }

    KeUnlockMutex(addr_of_mut!((*DeviceExtension).FcbListLock));

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    STATUS_SUCCESS
}
