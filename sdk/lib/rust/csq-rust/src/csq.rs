// csq.c — Rust translation of the ReactOS Cancel-Safe IRP Queue library.
// DeepSeek+c2rust translation (Batch 4 kernel-lib residue), hand-integrated.
// C kept on disk per the never-delete rule.
//
// The IO_CSQ callback fields are `Option<unsafe extern "stdcall"/"C" fn(...)>`
// (PIO_CSQ_*); invoked via `((*Csq).Field).unwrap()(...)`. The cancel routine is
// stored/cleared via IoSetCancelRoutine (PDRIVER_CANCEL = same Option), with a
// transmute bridge for our extern "system" IopCsqCancelRoutine. The per-IRP queue
// linkage lives in Irp->Tail.Overlay.DriverContext[3] (deep bindgen anon path).
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ddk::{
    IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCancelRoutine,
    PDEVICE_OBJECT, PIRP, _IRP, PIO_CSQ, _IO_CSQ, PIO_CSQ_IRP_CONTEXT,
    PIO_CSQ_INSERT_IRP, PIO_CSQ_INSERT_IRP_EX, PIO_CSQ_REMOVE_IRP, PIO_CSQ_PEEK_NEXT_IRP,
    PIO_CSQ_ACQUIRE_LOCK, PIO_CSQ_RELEASE_LOCK, PIO_CSQ_COMPLETE_CANCELED_IRP, PDRIVER_CANCEL,
};
use core::ffi::c_void;

// csq.h: IO_TYPE_CSQ_IRP_CONTEXT=1, IO_TYPE_CSQ=2, IO_TYPE_CSQ_EX=3.
const IO_TYPE_CSQ_IRP_CONTEXT: u32 = 1;
const IO_TYPE_CSQ: u32 = 2;
const IO_TYPE_CSQ_EX: u32 = 3;

// Irp->Tail.Overlay.DriverContext[3] — the slot CSQ uses to stash the queue/context.
#[inline(always)]
unsafe fn driver_ctx3(Irp: PIRP) -> *mut PVOID {
    core::ptr::addr_of_mut!(
        (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3]
    )
}

#[no_mangle]
pub unsafe extern "system" fn IopCsqCancelRoutine(_DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let mut Csq: PIO_CSQ;
    let mut Irql: KIRQL = 0;

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    // We could either get a context or just a csq
    Csq = *driver_ctx3(Irp) as PIO_CSQ;
    if (*Csq).Type == IO_TYPE_CSQ_IRP_CONTEXT {
        let Context: PIO_CSQ_IRP_CONTEXT = Csq as PIO_CSQ_IRP_CONTEXT;
        Csq = (*Context).Csq;
        (*Context).Irp = core::ptr::null_mut();
    }

    ((*Csq).CsqAcquireLock).unwrap()(Csq, core::ptr::addr_of_mut!(Irql));
    ((*Csq).CsqRemoveIrp).unwrap()(Csq, Irp);
    ((*Csq).CsqReleaseLock).unwrap()(Csq, Irql);
    ((*Csq).CsqCompleteCanceledIrp).unwrap()(Csq, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn IoCsqInitialize(
    Csq: PIO_CSQ,
    CsqInsertIrp: PIO_CSQ_INSERT_IRP,
    CsqRemoveIrp: PIO_CSQ_REMOVE_IRP,
    CsqPeekNextIrp: PIO_CSQ_PEEK_NEXT_IRP,
    CsqAcquireLock: PIO_CSQ_ACQUIRE_LOCK,
    CsqReleaseLock: PIO_CSQ_RELEASE_LOCK,
    CsqCompleteCanceledIrp: PIO_CSQ_COMPLETE_CANCELED_IRP,
) -> NTSTATUS {
    (*Csq).Type = IO_TYPE_CSQ;
    (*Csq).CsqInsertIrp = CsqInsertIrp;
    (*Csq).CsqRemoveIrp = CsqRemoveIrp;
    (*Csq).CsqPeekNextIrp = CsqPeekNextIrp;
    (*Csq).CsqAcquireLock = CsqAcquireLock;
    (*Csq).CsqReleaseLock = CsqReleaseLock;
    (*Csq).CsqCompleteCanceledIrp = CsqCompleteCanceledIrp;
    (*Csq).ReservePointer = core::ptr::null_mut();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoCsqInitializeEx(
    Csq: PIO_CSQ,
    CsqInsertIrpEx: PIO_CSQ_INSERT_IRP_EX,
    CsqRemoveIrp: PIO_CSQ_REMOVE_IRP,
    CsqPeekNextIrp: PIO_CSQ_PEEK_NEXT_IRP,
    CsqAcquireLock: PIO_CSQ_ACQUIRE_LOCK,
    CsqReleaseLock: PIO_CSQ_RELEASE_LOCK,
    CsqCompleteCanceledIrp: PIO_CSQ_COMPLETE_CANCELED_IRP,
) -> NTSTATUS {
    (*Csq).Type = IO_TYPE_CSQ_EX;
    (*Csq).CsqInsertIrp =
        core::mem::transmute::<PIO_CSQ_INSERT_IRP_EX, PIO_CSQ_INSERT_IRP>(CsqInsertIrpEx);
    (*Csq).CsqRemoveIrp = CsqRemoveIrp;
    (*Csq).CsqPeekNextIrp = CsqPeekNextIrp;
    (*Csq).CsqAcquireLock = CsqAcquireLock;
    (*Csq).CsqReleaseLock = CsqReleaseLock;
    (*Csq).CsqCompleteCanceledIrp = CsqCompleteCanceledIrp;
    (*Csq).ReservePointer = core::ptr::null_mut();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoCsqInsertIrp(
    Csq: PIO_CSQ,
    Irp: PIRP,
    Context: PIO_CSQ_IRP_CONTEXT,
) {
    IoCsqInsertIrpEx(Csq, Irp, Context, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn IoCsqInsertIrpEx(
    Csq: PIO_CSQ,
    Irp: PIRP,
    Context: PIO_CSQ_IRP_CONTEXT,
    InsertContext: PVOID,
) -> NTSTATUS {
    let mut Retval: NTSTATUS = STATUS_SUCCESS;
    let mut Irql: KIRQL = 0;

    ((*Csq).CsqAcquireLock).unwrap()(Csq, core::ptr::addr_of_mut!(Irql));

    'block: loop {
        IoMarkIrpPending(Irp);

        if !Context.is_null() {
            (*Context).Type = IO_TYPE_CSQ_IRP_CONTEXT;
            (*Context).Irp = Irp;
            (*Context).Csq = Csq;
            *driver_ctx3(Irp) = Context as PVOID;
        } else {
            *driver_ctx3(Irp) = Csq as PVOID;
        }

        // Step 1: queue the IRP
        if (*Csq).Type == IO_TYPE_CSQ {
            ((*Csq).CsqInsertIrp).unwrap()(Csq, Irp);
        } else {
            let pCsqInsertIrpEx: PIO_CSQ_INSERT_IRP_EX =
                core::mem::transmute::<PIO_CSQ_INSERT_IRP, PIO_CSQ_INSERT_IRP_EX>(
                    (*Csq).CsqInsertIrp,
                );
            Retval = pCsqInsertIrpEx.unwrap()(Csq, Irp, InsertContext);
            if Retval != STATUS_SUCCESS {
                break 'block;
            }
        }

        // Step 2: set our cancel routine
        IoSetCancelRoutine(
            Irp,
            core::mem::transmute::<
                Option<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP)>,
                PDRIVER_CANCEL,
            >(Some(IopCsqCancelRoutine)),
        );

        // Step 3: deal with an IRP that is already canceled
        if (*Irp).Cancel == 0 {
            break 'block;
        }
        // If our cancel routine is already gone, the IO manager is running it.
        if IoSetCancelRoutine(Irp, None).is_none() {
            break 'block;
        }

        *driver_ctx3(Irp) = core::ptr::null_mut();
        ((*Csq).CsqRemoveIrp).unwrap()(Csq, Irp);
        ((*Csq).CsqCompleteCanceledIrp).unwrap()(Csq, Irp);
        if !Context.is_null() {
            (*Context).Irp = core::ptr::null_mut();
        }
        break 'block;
    }

    ((*Csq).CsqReleaseLock).unwrap()(Csq, Irql);
    Retval
}

#[no_mangle]
pub unsafe extern "system" fn IoCsqRemoveIrp(
    Csq: PIO_CSQ,
    mut Context: PIO_CSQ_IRP_CONTEXT,
) -> PIRP {
    let mut Irql: KIRQL = 0;
    let mut Irp: PIRP;

    ((*Csq).CsqAcquireLock).unwrap()(Csq, core::ptr::addr_of_mut!(Irql));

    'block: loop {
        Irp = (*Context).Irp;
        if Irp.is_null() {
            break 'block;
        }
        // ASSERT(Context->Csq == Csq);
        if IoSetCancelRoutine(Irp, None).is_none() {
            // already gone — must not touch IRP or context
            Irp = core::ptr::null_mut();
            break 'block;
        }
        // ASSERT(Context == Irp->Tail.Overlay.DriverContext[3]);
        ((*Csq).CsqRemoveIrp).unwrap()(Csq, Irp);
        Context = InterlockedExchangePointer(driver_ctx3(Irp), core::ptr::null_mut())
            as PIO_CSQ_IRP_CONTEXT;
        if !Context.is_null() && (*Context).Type == IO_TYPE_CSQ_IRP_CONTEXT {
            (*Context).Irp = core::ptr::null_mut();
            // ASSERT(Context->Csq == Csq);
        }
        *driver_ctx3(Irp) = core::ptr::null_mut();
        break 'block;
    }

    ((*Csq).CsqReleaseLock).unwrap()(Csq, Irql);
    Irp
}

#[no_mangle]
pub unsafe extern "system" fn IoCsqRemoveNextIrp(Csq: PIO_CSQ, PeekContext: PVOID) -> PIRP {
    let mut Irql: KIRQL = 0;
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Context: PIO_CSQ_IRP_CONTEXT;

    ((*Csq).CsqAcquireLock).unwrap()(Csq, core::ptr::addr_of_mut!(Irql));

    loop {
        Irp = ((*Csq).CsqPeekNextIrp).unwrap()(Csq, Irp, PeekContext);
        if Irp.is_null() {
            break;
        }
        // If the cancel routine is gone, we're already canceled — skip.
        if IoSetCancelRoutine(Irp, None).is_none() {
            continue;
        }
        ((*Csq).CsqRemoveIrp).unwrap()(Csq, Irp);
        Context = InterlockedExchangePointer(driver_ctx3(Irp), core::ptr::null_mut())
            as PIO_CSQ_IRP_CONTEXT;
        if !Context.is_null() && (*Context).Type == IO_TYPE_CSQ_IRP_CONTEXT {
            (*Context).Irp = core::ptr::null_mut();
            // ASSERT(Context->Csq == Csq);
        }
        *driver_ctx3(Irp) = core::ptr::null_mut();
        break;
    }

    ((*Csq).CsqReleaseLock).unwrap()(Csq, Irql);
    Irp
}
