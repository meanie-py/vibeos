use crate::types::*;
use crate::glue::*;
use crate::cb;
use ntkrnl_sys::ddk::{IO_STACK_LOCATION, PIO_STACK_LOCATION};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IRP, IoAcquireCancelSpinLock, IoCsqRemoveNextIrp, IoMarkIrpPending, IoReleaseCancelSpinLock, IoSetCancelRoutine, LIST_ENTRY, PDEVICE_OBJECT, PIO_CSQ, PIRP, PKIRQL};
use core::ffi::c_void;

extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn USBPORT_USBDStatusToNtStatus(Urb: PURB, Status: u32) -> NTSTATUS;
    fn USBPORT_CompleteTransfer(Urb: PURB, USBDStatus: USBD_STATUS);
    fn USBPORT_EndpointWorker(Endpoint: PUSBPORT_ENDPOINT, Flag: BOOLEAN) -> BOOLEAN;
    fn USBPORT_FlushMapTransfers(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_InvalidateEndpointHandler( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, Type: ULONG, );
    fn USBPORT_GetEndpointState(Endpoint: PUSBPORT_ENDPOINT) -> ULONG;
    fn USBPORT_Wait(MiniPortExtension: PVOID, Milliseconds: ULONG,) -> NTSTATUS;
}
extern "C" {
    fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION;
    fn URB_FROM_IRP(Irp: PIRP) -> PURB;
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// (no output — all four `#define`s are object-like macros with empty replacement lists, i.e. NDEBUG-style flag macros with no value, so none translate to a constant)

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InsertIdleIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_QUEUE!(b"USBPORT_InsertIdleIrp: Irp - %p\n\0".as_ptr() as *const i8, Irp);

    FdoExtension = (Csq as usize).wrapping_sub(
        core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, IdleIoCsq),
    ) as *mut USBPORT_DEVICE_EXTENSION;

    InsertTailList(
        core::ptr::addr_of_mut!((*FdoExtension).IdleIrpList),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveIdleIrp(mut Csq: PIO_CSQ, mut Irp: PIRP) {
    DPRINT_QUEUE!(b"USBPORT_RemoveIdleIrp: Irp - %p\n\0".as_ptr() as *const i8, Irp);
    RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PeekNextIdleIrp(
    Csq: PIO_CSQ,
    Irp: PIRP,
    PeekContext: PVOID,
) -> PIRP {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut NextIrp: PIRP = core::ptr::null_mut();

    DPRINT_QUEUE!(
        b"USBPORT_PeekNextIdleIrp: Irp - %p, PeekContext - %p\n\0".as_ptr() as *const i8,
        Irp,
        PeekContext,
    );

    FdoExtension = ((Csq as usize)
        - core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, IdleIoCsq))
        as *mut USBPORT_DEVICE_EXTENSION;

    ListHead = core::ptr::addr_of_mut!((*FdoExtension).IdleIrpList) as PLIST_ENTRY;

    if !Irp.is_null() {
        NextEntry = (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink;
    } else {
        NextEntry = (*ListHead).Flink;
    }

    while NextEntry != ListHead {
        NextIrp = ((NextEntry as usize)
            - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
            as *mut IRP;

        if PeekContext.is_null() {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    return NextIrp;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AcquireIdleLock(Csq: PIO_CSQ, Irql: PKIRQL) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_QUEUE!(b"USBPORT_AcquireIdleLock: ... \n\0".as_ptr() as *const i8);

    FdoExtension = (Csq as usize)
        .wrapping_sub(
            core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, IdleIoCsq),
        ) as *mut USBPORT_DEVICE_EXTENSION as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsqSpinLock) as *mut usize), Irql);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ReleaseIdleLock(Csq: PIO_CSQ, Irql: KIRQL) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();

    DPRINT_QUEUE!(b"USBPORT_ReleaseIdleLock: ... \n\0".as_ptr() as *const i8);

    FdoExtension = (Csq as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, IdleIoCsq) as ULONG_PTR,
    ) as *mut USBPORT_DEVICE_EXTENSION;

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).IdleIoCsqSpinLock) as *mut usize), Irql);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CompleteCanceledIdleIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    FdoExtension = (Csq as usize)
        .wrapping_sub(core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, IdleIoCsq))
        as *mut USBPORT_DEVICE_EXTENSION;

    InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).IdleLockCounter));

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InsertBadRequest(Csq: PIO_CSQ, Irp: PIRP) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_QUEUE!(b"USBPORT_InsertBadRequest: Irp - %p\n\0".as_ptr() as *const i8, Irp);

    FdoExtension = ((Csq as usize)
        - (core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, BadRequestIoCsq)))
        as *mut USBPORT_DEVICE_EXTENSION;

    InsertTailList(
        core::ptr::addr_of_mut!((*FdoExtension).BadRequestList),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveBadRequest(Csq: PIO_CSQ, Irp: PIRP) {
    DPRINT_QUEUE!(b"USBPORT_RemoveBadRequest: Irp - %p\n\0".as_ptr() as *const i8, Irp);
    RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_PeekNextBadRequest(
    Csq: PIO_CSQ,
    Irp: PIRP,
    PeekContext: PVOID,
) -> PIRP {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut NextIrp: PIRP = core::ptr::null_mut();

    DPRINT_QUEUE!(
        b"USBPORT_PeekNextBadRequest: Irp - %p, PeekContext - %p\n\0".as_ptr() as *const i8,
        Irp,
        PeekContext,
    );

    FdoExtension = (Csq as usize)
        .wrapping_sub(
            core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, BadRequestIoCsq),
        ) as *mut USBPORT_DEVICE_EXTENSION;

    ListHead = core::ptr::addr_of_mut!((*FdoExtension).BadRequestList) as PLIST_ENTRY;

    if !Irp.is_null() {
        NextEntry = (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink;
    } else {
        NextEntry = (*ListHead).Flink;
    }

    while NextEntry != ListHead {
        NextIrp = (NextEntry as usize)
            .wrapping_sub(
                core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry),
            ) as *mut IRP;

        if PeekContext.is_null() {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    return NextIrp;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AcquireBadRequestLock(
    Csq: PIO_CSQ,
    Irql: PKIRQL,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    // DPRINT_QUEUE!("USBPORT_AcquireBadRequestLock: ... \n");

    FdoExtension = (Csq as usize).wrapping_sub(
        core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, BadRequestIoCsq),
    ) as *mut USBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsqSpinLock) as *mut usize),
        Irql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ReleaseBadRequestLock(Csq: PIO_CSQ, Irql: KIRQL) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_QUEUE!(b"USBPORT_ReleaseBadRequestLock: ... \n\0".as_ptr() as *const i8);

    FdoExtension = (Csq as usize).wrapping_sub(
        core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, BadRequestIoCsq),
    ) as *mut USBPORT_DEVICE_EXTENSION;

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsqSpinLock) as *mut usize),
        Irql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CompleteCanceledBadRequest(Csq: PIO_CSQ, Irp: PIRP) {
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_QUEUE!(b"USBPORT_CompleteCanceledBadRequest: Irp - %p\n\0".as_ptr() as *const i8, Irp);

    FdoExtension = ((Csq as usize)
        - (core::mem::offset_of!(USBPORT_DEVICE_EXTENSION, BadRequestIoCsq)))
        as *mut USBPORT_DEVICE_EXTENSION;

    InterlockedDecrement(core::ptr::addr_of_mut!((*FdoExtension).BadRequestLockCounter) as *mut core::ffi::c_long);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InsertIrpInTable(
    mut IrpTable: PUSBPORT_IRP_TABLE,
    mut Irp: PIRP,
) {
    let mut ix: ULONG;

    // DPRINT_CORE!("USBPORT_InsertIrpInTable: IrpTable - %p, Irp - %p\n", IrpTable, Irp);

    // ASSERT(IrpTable != NULL);

    loop {
        ix = 0 as ULONG;
        while ix < 0x200 {
            if (*IrpTable).irp[ix as usize].is_null() {
                (*IrpTable).irp[ix as usize] = Irp;

                if ix > 0 {
                    // DPRINT_CORE!("USBPORT_InsertIrpInTable: ix - %x\n", ix);
                }

                return;
            }
            ix = ix.wrapping_add(1);
        }

        if ix != 0x200 {
            KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
        }

        (*IrpTable).LinkNextTable = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<USBPORT_IRP_TABLE>() as SIZE_T,
            USB_PORT_TAG as ULONG,
        ) as PUSBPORT_IRP_TABLE;

        if (*IrpTable).LinkNextTable.is_null() {
            KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
        }

        RtlZeroMemory(
            (*IrpTable).LinkNextTable as *mut core::ffi::c_void,
            core::mem::size_of::<USBPORT_IRP_TABLE>(),
        );

        IrpTable = (*IrpTable).LinkNextTable as PUSBPORT_IRP_TABLE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveIrpFromTable(
    mut IrpTable: PUSBPORT_IRP_TABLE,
    Irp: PIRP,
) -> PIRP {
    let mut ix: ULONG;

    DPRINT_CORE!(
        b"USBPORT_RemoveIrpFromTable: IrpTable - %p, Irp - %p\n\0".as_ptr() as *const i8,
        IrpTable,
        Irp,
    );

    if IrpTable.is_null() {
        RtlAssert(
            b"IrpTable != NULL\0".as_ptr() as *const i8 as PVOID,
            b"queue.c\0".as_ptr() as *const i8 as PVOID,
            325 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    loop {
        ix = 0 as ULONG;
        while ix < 0x200 {
            if (*IrpTable).irp[ix as usize] == Irp {
                (*IrpTable).irp[ix as usize] = core::ptr::null_mut();

                if ix > 0 {
                    DPRINT_CORE!(
                        b"USBPORT_RemoveIrpFromTable: ix - %x\n\0".as_ptr() as *const i8,
                        ix,
                    );
                }

                return Irp;
            }
            ix = ix.wrapping_add(1);
        }

        if (*IrpTable).LinkNextTable.is_null() {
            break;
        }

        IrpTable = (*IrpTable).LinkNextTable as PUSBPORT_IRP_TABLE;
        continue;
    }

    // DPRINT removed
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemoveActiveTransferIrp(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> PIRP {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_CORE!(
        b"USBPORT_RemoveActiveTransferIrp: Irp - %p\n\0".as_ptr() as *const i8,
        Irp,
    );
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    return USBPORT_RemoveIrpFromTable((*FdoExtension).ActiveIrpTable, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemovePendingTransferIrp(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> PIRP {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_CORE!(
        b"USBPORT_RemovePendingTransferIrp: Irp - %p\n\0".as_ptr() as *const i8,
        Irp,
    );
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    return USBPORT_RemoveIrpFromTable((*FdoExtension).PendingIrpTable, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FindUrbInIrpTable(
    mut IrpTable: PUSBPORT_IRP_TABLE,
    Urb: PURB,
    Irp: PIRP,
) {
    const BUGCODE_USB_DRIVER: ULONG = 0xFE;

    let mut ix: ULONG;
    let mut irp: PIRP;
    let mut urbIn: PURB;

    // DPRINT_CORE!("USBPORT_FindUrbInIrpTable: IrpTable - %p, Urb - %p, Irp - %p\n", IrpTable, Urb, Irp);

    if IrpTable.is_null() {
        RtlAssert(
            b"IrpTable != NULL\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbport/queue.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            394 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    loop {
        ix = 0 as ULONG;
        while ix < 0x200 as core::ffi::c_ulong {
            irp = (*IrpTable).irp[ix as usize];

            if !irp.is_null() {
                urbIn = (*IoGetCurrentIrpStackLocation(irp))
                    .Parameters
                    .Others
                    .Argument1 as PURB;

                if urbIn == Urb {
                    if irp == Irp {
                        KeBugCheckEx(
                            BUGCODE_USB_DRIVER,
                            4 as ULONG_PTR,
                            irp as ULONG_PTR,
                            urbIn as ULONG_PTR,
                            0 as ULONG_PTR,
                        );
                    }

                    KeBugCheckEx(
                        BUGCODE_USB_DRIVER,
                        2 as ULONG_PTR,
                        irp as ULONG_PTR,
                        Irp as ULONG_PTR,
                        urbIn as ULONG_PTR,
                    );
                }
            }

            ix = ix.wrapping_add(1);
        }

        IrpTable = (*IrpTable).LinkNextTable as PUSBPORT_IRP_TABLE;
        if IrpTable.is_null() {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FindIrpInTable(
    mut IrpTable: PUSBPORT_IRP_TABLE,
    Irp: PIRP,
) -> PIRP {
    let mut ix: ULONG;
    let mut irp: PIRP;

    // DPRINT_CORE!("USBPORT_FindIrpInTable: IrpTable - %p, Irp - %p\n", IrpTable, Irp);

    if IrpTable.is_null() {
        RtlAssert(
            b"IrpTable != NULL\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbport/queue.c\0".as_ptr() as *const i8 as PVOID,
            443 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    loop {
        ix = 0 as ULONG;
        while ix < 0x200 as u32 {
            irp = (*IrpTable).irp[ix as usize];

            if !irp.is_null() && irp == Irp {
                return irp;
            }
            ix = ix.wrapping_add(1);
        }

        IrpTable = (*IrpTable).LinkNextTable as PUSBPORT_IRP_TABLE;
        if (*IrpTable).LinkNextTable.is_null() {
            break;
        }
    }

    // DPRINT_CORE!("USBPORT_FindIrpInTable: Not found!!!\n");
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FindActiveTransferIrp(
    FdoDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> PIRP {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_CORE!(
        b"USBPORT_FindActiveTransferIrp: Irp - %p\n\0".as_ptr() as *const i8,
        Irp,
    );
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    return USBPORT_FindIrpInTable((*FdoExtension).ActiveIrpTable, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CancelPendingTransferIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {

    let mut Urb: PURB;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let mut irp: PIRP;

    DPRINT_CORE!(
        b"USBPORT_CancelPendingTransferIrp: DeviceObject - %p, Irp - %p\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    Urb = URB_FROM_IRP(Irp);
    Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
    Endpoint = (*Transfer).Endpoint;

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*DeviceObject).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    irp = USBPORT_RemovePendingTransferIrp(FdoDevice, Irp);

    if irp.is_null() {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
            OldIrql,
        );
        return;
    }

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

    RemoveEntryList(core::ptr::addr_of_mut!((*Transfer).TransferLink));

    (*Transfer).TransferLink.Flink = core::ptr::null_mut();
    (*Transfer).TransferLink.Blink = core::ptr::null_mut();

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
        OldIrql,
    );

    USBPORT_CompleteTransfer((*Transfer).Urb, USBD_STATUS_CANCELED);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CancelActiveTransferIrp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {

    const TRANSFER_FLAG_CANCELED: ULONG = 0x00000001; // FIXED 2026-07-05: translation invented 0x10; verified vs usbport.h/usbmport.h
    const TRANSFER_FLAG_PARENT: ULONG = 0x00000400; // FIXED 2026-07-05: translation invented 0x200; verified vs usbport.h/usbmport.h
    const INVALIDATE_ENDPOINT_WORKER_THREAD: ULONG = 1;

    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Urb: PURB;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut irp: PIRP;
    let mut SplitTransfer: PUSBPORT_TRANSFER;
    let mut Entry: PLIST_ENTRY;
    let mut OldIrql: KIRQL = 0;

    DPRINT_CORE!(b"USBPORT_CancelActiveTransferIrp: Irp - %p\n\0".as_ptr() as *const i8, Irp);

    PdoExtension = (*DeviceObject).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
    FdoDevice = (*PdoExtension).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    irp = USBPORT_FindActiveTransferIrp(FdoDevice, Irp);

    if irp.is_null() {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
            OldIrql,
        );
        return;
    }

    Urb = URB_FROM_IRP(irp);
    Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
    Endpoint = (*Transfer).Endpoint;

    DPRINT_CORE!(
        b"USBPORT_CancelActiveTransferIrp: irp - %p, Urb - %p, Transfer - %p\n\0".as_ptr() as *const i8,
        irp,
        Urb,
        Transfer,
    );

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

    (*Transfer).Flags |= TRANSFER_FLAG_CANCELED;

    if (*Transfer).Flags & TRANSFER_FLAG_PARENT != 0 {
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Transfer).TransferSpinLock) as *mut usize));

        Entry = (*Transfer).SplitTransfersList.Flink;

        while !Entry.is_null() && Entry != core::ptr::addr_of_mut!((*Transfer).SplitTransfersList) {
            SplitTransfer = (Entry as usize)
                .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, SplitLink))
                as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

            (*SplitTransfer).Flags |= TRANSFER_FLAG_CANCELED;

            Entry = (*Entry).Flink;
        }

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Transfer).TransferSpinLock) as *mut usize));
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
        OldIrql,
    );

    USBPORT_InvalidateEndpointHandler(
        FdoDevice,
        Endpoint,
        INVALIDATE_ENDPOINT_WORKER_THREAD,
    );
    return;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushAbortList(Endpoint: PUSBPORT_ENDPOINT) {
    const USBD_STATUS_SUCCESS: u32 = 0;
    const TRANSFER_FLAG_ABORTED: u32 = 0x00000010;

    let mut Entry: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER;
    let AbortList: PLIST_ENTRY;
    let mut List: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut Urb: PURB;
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE = core::ptr::null_mut();

    // DPRINT_CORE!("USBPORT_FlushAbortList: Endpoint - %p\n", Endpoint);

    InitializeListHead(core::ptr::addr_of_mut!(List));

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    if IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).AbortList)) {
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );
        return;
    }

    Entry = (*Endpoint).PendingTransferList.Flink as PLIST_ENTRY;

    while !Entry.is_null() && Entry != core::ptr::addr_of_mut!((*Endpoint).PendingTransferList) {
        Transfer = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER;

        if (*Transfer).Flags & TRANSFER_FLAG_ABORTED != 0 {
            // DPRINT_CORE!("USBPORT_FlushAbortList: Aborted PendingTransfer  - %p\n", Transfer);

            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                (*Endpoint).EndpointOldIrql,
            );
            return;
        }

        Entry = (*Transfer).TransferLink.Flink as PLIST_ENTRY;
    }

    Entry = (*Endpoint).TransferList.Flink as PLIST_ENTRY;

    while !Entry.is_null() && Entry != core::ptr::addr_of_mut!((*Endpoint).TransferList) {
        Transfer = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER;

        if (*Transfer).Flags & TRANSFER_FLAG_ABORTED != 0 {
            // DPRINT_CORE!("USBPORT_FlushAbortList: Aborted ActiveTransfer - %p\n", Transfer);

            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                (*Endpoint).EndpointOldIrql,
            );
            return;
        }

        Entry = (*Transfer).TransferLink.Flink as PLIST_ENTRY;
    }

    AbortList = core::ptr::addr_of_mut!((*Endpoint).AbortList) as PLIST_ENTRY;

    while !IsListEmpty(AbortList) {
        //DbgBreakPoint();

        Irp = ((*AbortList).Flink as usize)
            .wrapping_sub(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
            as *mut IRP;

        RemoveHeadList(AbortList);
        InsertTailList(
            core::ptr::addr_of_mut!(List),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
        );
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    while !IsListEmpty(core::ptr::addr_of_mut!(List)) {
        //DbgBreakPoint();

        Irp = (List.Flink as usize)
            .wrapping_sub(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry))
            as *mut IRP;

        RemoveHeadList(core::ptr::addr_of_mut!(List));

        Urb = URB_FROM_IRP(Irp);

        DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
        InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock));

        Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);

        // DPRINT_CORE!("USBPORT_FlushAbortList: complete Irp - %p\n", Irp);

        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushCancelList(Endpoint: PUSBPORT_ENDPOINT) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Irp: PIRP;
    let mut OldIrql: KIRQL = 0;
    let mut PrevIrql: KIRQL = 0;

    DPRINT_CORE!(b"USBPORT_FlushCancelList: ... \n\0".as_ptr() as *const i8);

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

    while !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).CancelList)) {
        Transfer = ((*Endpoint).CancelList.Flink as usize
            - core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        RemoveHeadList(core::ptr::addr_of_mut!((*Endpoint).CancelList));

        Irp = (*Transfer).Irp;

        if !Irp.is_null() {
            // DPRINT removed

            IoAcquireCancelSpinLock(core::ptr::addr_of_mut!(PrevIrql));
            IoSetCancelRoutine(Irp, None);
            IoReleaseCancelSpinLock(PrevIrql);

            USBPORT_RemoveActiveTransferIrp(FdoDevice, Irp);
        }

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
            OldIrql,
        );

        if (*Endpoint).Flags as u32 & ENDPOINT_FLAG_NUKE as u32 != 0 {
            USBPORT_CompleteTransfer((*Transfer).Urb, USBD_STATUS_DEVICE_GONE);
        } else {
            if (*Transfer).Flags as u32 & TRANSFER_FLAG_DEVICE_GONE as u32 != 0 {
                USBPORT_CompleteTransfer((*Transfer).Urb, USBD_STATUS_DEVICE_GONE);
            } else {
                USBPORT_CompleteTransfer((*Transfer).Urb, USBD_STATUS_CANCELED);
            }
        }

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
        OldIrql,
    );

    USBPORT_FlushAbortList(Endpoint);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushPendingTransfers(mut Endpoint: PUSBPORT_ENDPOINT) {

    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut IsMapTransfer: BOOLEAN;
    let mut IsEnd: BOOLEAN = 0 as BOOLEAN;
    let mut List: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Urb: PURB;
    let mut Irp: PIRP;
    let mut OldIrql: KIRQL = 0;
    let mut Result: BOOLEAN;

    DPRINT_CORE!(
        b"USBPORT_FlushPendingTransfers: Endpoint - %p\n\0".as_ptr() as *const i8,
        Endpoint,
    );

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    // Diverges from the C, whose FlushPendingLock is a try-lock (CAS 0 -> 1, else return).
    // On SMP that loses work: CPU A is flushing this endpoint and has already found
    // PendingTransferList empty; CPU B queues a transfer (USBPORT_QueueTransferUrb queues
    // first, then calls here), fails the CAS and returns; A releases and leaves. Nothing
    // flushes this endpoint again, so the transfer sits pending forever -- caught live on
    // amd64 -smp 2 as usbstor's CBW stranded on its bulk-out endpoint (pending=1,
    // active=0, lock 0, endpoint idle) and all USB storage I/O stopped. Now the lock is a
    // count: a caller that finds it held bumps it and returns, and the holder makes
    // another pass instead of releasing while any such request is outstanding.
    let FlushLock = &*(core::ptr::addr_of_mut!((*Endpoint).FlushPendingLock)
        as *const core::sync::atomic::AtomicI32);
    if FlushLock.fetch_add(1, core::sync::atomic::Ordering::SeqCst) != 0 {
        DPRINT_CORE!(
            b"USBPORT_FlushPendingTransfers: Endpoint Locked \n\0".as_ptr() as *const i8,
        );
        return;
    }

    loop {
        IsMapTransfer = 0 as BOOLEAN;

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

        'block: {
            if (*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND != 0 {
                IsEnd = 1 as BOOLEAN;

                KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
                    OldIrql,
                );
                break 'block; // goto Next
            }

            'worker: {
                if (*Endpoint).Flags & ENDPOINT_FLAG_ROOTHUB_EP0 == 0 {
                    if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList)) {
                        List = (*Endpoint).TransferList.Flink;

                        while !List.is_null()
                            && List != core::ptr::addr_of_mut!((*Endpoint).TransferList)
                        {
                            Transfer = (List as usize
                                - core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
                                as PUSBPORT_TRANSFER;

                            if (*Transfer).Flags & TRANSFER_FLAG_SUBMITED == 0 {
                                KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                                    (*Endpoint).EndpointSpinLock
                                ) as *mut usize));
                                KeReleaseSpinLock(
                                    (core::ptr::addr_of_mut!(
                                        (*FdoExtension).FlushPendingTransferSpinLock
                                    ) as *mut usize),
                                    OldIrql,
                                );

                                IsEnd = 1 as BOOLEAN;
                                break 'worker; // goto Worker
                            }

                            List = (*Transfer).TransferLink.Flink;
                        }
                    }
                }

                List = (*Endpoint).PendingTransferList.Flink;

                if List.is_null()
                    || IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).PendingTransferList))
                {
                    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                        (*Endpoint).EndpointSpinLock
                    ) as *mut usize));
                    KeReleaseSpinLock(
                        (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
                        OldIrql,
                    );

                    IsEnd = 1 as BOOLEAN;
                    break 'worker; // goto Worker
                }

                Transfer = (List as usize
                    - core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
                    as PUSBPORT_TRANSFER;

                if !(*Transfer).Irp.is_null() {
                    DPRINT_CORE!(
                        b"USBPORT_FlushPendingTransfers: Transfer->Irp->CancelRoutine - %p\n\0"
                            .as_ptr() as *const i8,
                        (*(*Transfer).Irp).CancelRoutine,
                    );
                }

                if !(*Transfer).Irp.is_null()
                    && IoSetCancelRoutine((*Transfer).Irp, None).is_none()
                {
                    DPRINT_CORE!(
                        b"USBPORT_FlushPendingTransfers: Transfer->Irp - %p\n\0".as_ptr()
                            as *const i8,
                        (*Transfer).Irp,
                    );

                    Transfer = core::ptr::null_mut();
                    IsEnd = 1 as BOOLEAN;
                }

                if Transfer.is_null() {
                    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                        (*Endpoint).EndpointSpinLock
                    ) as *mut usize));
                    KeReleaseSpinLock(
                        (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
                        OldIrql,
                    );

                    if IsMapTransfer != 0 {
                        USBPORT_FlushMapTransfers(FdoDevice);
                        break 'block; // goto Next
                    }

                    break 'worker; // goto Worker
                }

                Irp = (*Transfer).Irp;
                Urb = (*Transfer).Urb;

                RemoveEntryList(core::ptr::addr_of_mut!((*Transfer).TransferLink));
                (*Transfer).TransferLink.Flink = core::ptr::null_mut();
                (*Transfer).TransferLink.Blink = core::ptr::null_mut();

                if !Irp.is_null() {
                    Irp = USBPORT_RemovePendingTransferIrp(FdoDevice, Irp);
                }

                KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                    (*Endpoint).EndpointSpinLock
                ) as *mut usize));
                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).FlushPendingTransferSpinLock) as *mut usize),
                    OldIrql,
                );

                KeAcquireSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
                    core::ptr::addr_of_mut!(OldIrql),
                );

                if !Irp.is_null() {
                    IoSetCancelRoutine(
                        Irp,
                        cb!(USBPORT_CancelActiveTransferIrp),
                    );

                    if (*Irp).Cancel != 0 && IoSetCancelRoutine(Irp, None).is_some() {
                        DPRINT_CORE!(
                            b"USBPORT_FlushPendingTransfers: irp - %p\n\0".as_ptr() as *const i8,
                            Irp,
                        );

                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
                            OldIrql,
                        );

                        USBPORT_CompleteTransfer((*Transfer).Urb, USBD_STATUS_CANCELED);
                        break 'worker; // goto Worker
                    }

                    USBPORT_FindUrbInIrpTable((*FdoExtension).ActiveIrpTable, Urb, Irp);
                    USBPORT_InsertIrpInTable((*FdoExtension).ActiveIrpTable, Irp);
                }

                IsMapTransfer = USBPORT_QueueActiveUrbToEndpoint(Endpoint, Urb);

                KeReleaseSpinLock(
                    (core::ptr::addr_of_mut!((*FdoExtension).FlushTransferSpinLock) as *mut usize),
                    OldIrql,
                );

                if IsMapTransfer != 0 {
                    USBPORT_FlushMapTransfers(FdoDevice);
                    break 'block; // goto Next
                }
            } // end 'worker

            // Worker:
            KeRaiseIrql(DISPATCH_LEVEL as KIRQL, core::ptr::addr_of_mut!(OldIrql));
            Result = USBPORT_EndpointWorker(Endpoint, 0 as BOOLEAN);
            KeLowerIrql(OldIrql);

            if Result != 0 {
                USBPORT_InvalidateEndpointHandler(
                    FdoDevice,
                    Endpoint,
                    INVALIDATE_ENDPOINT_WORKER_THREAD as ULONG,
                );
            }
        } // end 'block (Next:)

        // Next:
        if IsEnd != 0 {
            if FlushLock
                .compare_exchange(1, 0, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst)
                .is_err()
            {
                // Someone queued a transfer while this pass ran: absorb every such
                // request and look at the lists again.
                FlushLock.store(1, core::sync::atomic::Ordering::SeqCst);
                IsEnd = 0 as BOOLEAN;
                continue;
            }
            DPRINT_CORE!(
                b"USBPORT_FlushPendingTransfers: Endpoint Unlocked. Exit\n\0".as_ptr() as *const i8,
            );
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueuePendingUrbToEndpoint(
    Endpoint: PUSBPORT_ENDPOINT,
    Urb: PURB,
) {
    let mut Transfer: PUSBPORT_TRANSFER;

    DPRINT_CORE!(
        b"USBPORT_QueuePendingUrbToEndpoint: Endpoint - %p, Urb - %p\n\0".as_ptr() as *const i8,
        Endpoint,
        Urb,
    );

    Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
    //FIXME USBPORT_ResetEndpointIdle();
    InsertTailList(
        core::ptr::addr_of_mut!((*Endpoint).PendingTransferList),
        core::ptr::addr_of_mut!((*Transfer).TransferLink),
    );
    (*Urb).__bindgen_anon_1.UrbHeader.Status = USBD_STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueueActiveUrbToEndpoint(
    Endpoint: PUSBPORT_ENDPOINT,
    Urb: PURB,
) -> BOOLEAN {
    let mut Transfer: PUSBPORT_TRANSFER;
    let FdoDevice: PDEVICE_OBJECT;
    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut OldIrql: KIRQL = 0;

    DPRINT_CORE!(
        b"USBPORT_QueueActiveUrbToEndpoint: Endpoint - %p, Urb - %p\n\0".as_ptr() as *const i8,
        Endpoint,
        Urb,
    );

    Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    if ((*Endpoint).Flags as u32 & ENDPOINT_FLAG_NUKE as u32) != 0
        || ((*Transfer).Flags as u32 & TRANSFER_FLAG_ABORTED as u32) != 0
    {
        InsertTailList(
            core::ptr::addr_of_mut!((*Endpoint).CancelList),
            core::ptr::addr_of_mut!((*Transfer).TransferLink),
        );

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );

        return 0;
    }

    if (*Transfer).TransferParameters.TransferBufferLength == 0
        || ((*Endpoint).Flags as u32 & ENDPOINT_FLAG_DMA_TYPE as u32) == 0
    {
        InsertTailList(
            core::ptr::addr_of_mut!((*Endpoint).TransferList),
            core::ptr::addr_of_mut!((*Transfer).TransferLink),
        );

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );

        return 0;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MapTransferSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    InsertTailList(
        core::ptr::addr_of_mut!((*FdoExtension).MapTransferList),
        core::ptr::addr_of_mut!((*Transfer).TransferLink),
    );

    DeviceHandle = (*(*Transfer).Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
    InterlockedIncrement(
        core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock) as *mut core::ffi::c_long,
    );

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MapTransferSpinLock) as *mut usize),
        OldIrql,
    );

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueuePendingTransferIrp(mut Irp: PIRP) {
    const USBD_STATUS_CANCELED: USBD_STATUS = 0xC0010000u32 as USBD_STATUS;

    let mut Urb: PURB;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;

    DPRINT_CORE!(b"USBPORT_QueuePendingTransferIrp: Irp - %p\n\0".as_ptr() as *const i8, Irp);

    Urb = URB_FROM_IRP(Irp);

    Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
    Endpoint = (*Transfer).Endpoint;

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PENDING;
    IoMarkIrpPending(Irp);

    IoSetCancelRoutine(Irp, cb!(USBPORT_CancelPendingTransferIrp));

    if (*Irp).Cancel as i32 != 0 && !IoSetCancelRoutine(Irp, None).is_none() {
        USBPORT_CompleteTransfer(Urb, USBD_STATUS_CANCELED);
    } else {
        USBPORT_InsertIrpInTable((*FdoExtension).PendingIrpTable, Irp);
        USBPORT_QueuePendingUrbToEndpoint(Endpoint, Urb);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_QueueTransferUrb(mut Urb: PURB) {
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut Irp: PIRP;
    let mut DeviceHandle: PUSBPORT_DEVICE_HANDLE;
    let mut Parameters: PUSBPORT_TRANSFER_PARAMETERS;

    DPRINT_CORE!(b"USBPORT_QueueTransferUrb: Urb - %p\n\0".as_ptr() as *const i8, Urb);

    if (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags & USBD_DEFAULT_PIPE_TRANSFER != 0 {
        (*Urb).__bindgen_anon_1.UrbHeader.Function = URB_FUNCTION_CONTROL_TRANSFER as USHORT;
    }

    Transfer = (*Urb).__bindgen_anon_1.UrbControlTransfer.hca.Reserved8[0] as PUSBPORT_TRANSFER;
    Parameters = core::ptr::addr_of_mut!((*Transfer).TransferParameters) as PUSBPORT_TRANSFER_PARAMETERS;

    Endpoint = (*Transfer).Endpoint;
    (*Endpoint).Flags &= !ENDPOINT_FLAG_QUEUENE_EMPTY;

    (*Parameters).TransferBufferLength = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength;
    (*Parameters).TransferFlags = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags;

    (*Transfer).TransferBufferMDL = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferMDL;

    if (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferFlags & USBD_TRANSFER_DIRECTION_IN != 0 {
        (*Transfer).Direction = USBPORT_DMA_DIRECTION_FROM_DEVICE as ULONG;
    } else {
        (*Transfer).Direction = USBPORT_DMA_DIRECTION_TO_DEVICE as ULONG;
    }

    if (*Endpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*Parameters).SetupPacket) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*Urb).__bindgen_anon_1.UrbControlTransfer.SetupPacket) as *const core::ffi::c_void,
            core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
        );
    }

    DPRINT_URB!(
        b"... URB TransferBufferLength - %x\n\0".as_ptr() as *const i8,
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength,
    );

    (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength = 0 as ULONG;

    Irp = (*Transfer).Irp;

    if !Irp.is_null() {
        USBPORT_QueuePendingTransferIrp(Irp);
    } else {
        USBPORT_QueuePendingUrbToEndpoint(Endpoint, Urb);
    }

    DeviceHandle = (*Urb).__bindgen_anon_1.UrbHeader.UsbdDeviceHandle as PUSBPORT_DEVICE_HANDLE;
    InterlockedDecrement(core::ptr::addr_of_mut!((*DeviceHandle).DeviceHandleLock) as *mut core::ffi::c_long);

    USBPORT_FlushPendingTransfers(Endpoint);

    DPRINT_URB!(
        b"... URB TransferBufferLength - %x\n\0".as_ptr() as *const i8,
        (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength,
    );

    if (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength != 0 {
        let mut Buffer: PULONG;
        let mut BufferLength: ULONG;
        let mut BufferEnd: ULONG_PTR;
        let mut ix: ULONG;

        Buffer = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBuffer as PULONG;
        BufferLength = (*Urb).__bindgen_anon_1.UrbControlTransfer.TransferBufferLength;
        BufferEnd = (Buffer as ULONG_PTR).wrapping_add(BufferLength as ULONG_PTR);

        DPRINT_URB!(b"URB TransferBuffer - %p\n\0".as_ptr() as *const i8, Buffer);

        ix = 0 as ULONG;
        while (Buffer.offset(ix as isize) as ULONG_PTR) < BufferEnd {
            DPRINT_URB!(
                b"Buffer[%02X] - %p\n\0".as_ptr() as *const i8,
                ix,
                *Buffer.offset(ix as isize),
            );
            ix = ix.wrapping_add(1);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushAllEndpoints(mut FdoDevice: PDEVICE_OBJECT) {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Entry: PLIST_ENTRY;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut List: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut OldIrql: KIRQL = 0;

    DPRINT_CORE!(b"USBPORT_FlushAllEndpoints: ... \n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    InitializeListHead(core::ptr::addr_of_mut!(List));

    Entry = (*FdoExtension).EndpointList.Flink as PLIST_ENTRY;

    while !Entry.is_null() && Entry != core::ptr::addr_of_mut!((*FdoExtension).EndpointList) {
        Endpoint = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_ENDPOINT, EndpointLink))
            as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        if USBPORT_GetEndpointState(Endpoint) != USBPORT_ENDPOINT_CLOSED as ULONG {
            InsertTailList(
                core::ptr::addr_of_mut!(List),
                core::ptr::addr_of_mut!((*Endpoint).FlushLink),
            );
        }

        Entry = (*Endpoint).EndpointLink.Flink as PLIST_ENTRY;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
        OldIrql,
    );

    while !IsListEmpty(core::ptr::addr_of_mut!(List)) {
        Endpoint = (List.Flink as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_ENDPOINT, FlushLink))
            as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        RemoveHeadList(core::ptr::addr_of_mut!(List));

        (*Endpoint).FlushLink.Flink = core::ptr::null_mut();
        (*Endpoint).FlushLink.Blink = core::ptr::null_mut();

        if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).PendingTransferList)) {
            USBPORT_FlushPendingTransfers(Endpoint);
        }
    }

    DPRINT_CORE!(b"USBPORT_FlushAllEndpoints: exit\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_KillEndpointActiveTransfers(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> ULONG {
    let mut ActiveList: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut KilledTransfers: ULONG = 0;

    DPRINT_CORE!(b"USBPORT_KillEndpointActiveTransfers \n\0".as_ptr() as *const i8);

    ActiveList = (*Endpoint).TransferList.Flink as PLIST_ENTRY;

    while !ActiveList.is_null()
        && ActiveList != core::ptr::addr_of_mut!((*Endpoint).TransferList) as PLIST_ENTRY
    {
        KilledTransfers += 1;

        Transfer = (ActiveList as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_TRANSFER, TransferLink),
        ) as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        (*Transfer).Flags |= TRANSFER_FLAG_ABORTED as ULONG;

        ActiveList = (*Transfer).TransferLink.Flink as PLIST_ENTRY;
    }

    USBPORT_FlushPendingTransfers(Endpoint);
    USBPORT_FlushCancelList(Endpoint);

    return KilledTransfers;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushController(FdoDevice: PDEVICE_OBJECT) {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Entry: PLIST_ENTRY;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut KilledTransfers: ULONG;
    let EndpointList: PLIST_ENTRY;
    let mut OldIrql: KIRQL = 0;
    let mut FlushList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };

    DPRINT_CORE!(b"USBPORT_FlushController \n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    EndpointList = core::ptr::addr_of_mut!((*FdoExtension).EndpointList) as PLIST_ENTRY;

    loop {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        InitializeListHead(core::ptr::addr_of_mut!(FlushList));

        Entry = (*EndpointList).Flink as PLIST_ENTRY;

        if !IsListEmpty(EndpointList) {
            while !Entry.is_null() && Entry != EndpointList {
                Endpoint = (Entry as ULONG_PTR).wrapping_sub(
                    core::mem::offset_of!(USBPORT_ENDPOINT, EndpointLink) as ULONG_PTR,
                ) as PUSBPORT_ENDPOINT;

                if (*Endpoint).StateLast != USBPORT_ENDPOINT_REMOVE as ULONG
                    && (*Endpoint).StateLast != USBPORT_ENDPOINT_CLOSED as ULONG
                {
                    InterlockedIncrement(
                        core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
                    );
                    InsertTailList(
                        core::ptr::addr_of_mut!(FlushList),
                        core::ptr::addr_of_mut!((*Endpoint).FlushControllerLink),
                    );
                }

                Entry = (*Endpoint).EndpointLink.Flink as PLIST_ENTRY;
            }
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            OldIrql,
        );

        KilledTransfers = 0;
        while !IsListEmpty(core::ptr::addr_of_mut!(FlushList)) {
            Endpoint = (FlushList.Flink as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(USBPORT_ENDPOINT, FlushControllerLink) as ULONG_PTR,
            ) as PUSBPORT_ENDPOINT;

            RemoveHeadList(core::ptr::addr_of_mut!(FlushList));

            KilledTransfers = KilledTransfers
                .wrapping_add(USBPORT_KillEndpointActiveTransfers(FdoDevice, Endpoint));

            InterlockedDecrement(
                core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
            );
        }

        if KilledTransfers == 0 {
            break;
        }

        USBPORT_Wait(FdoDevice as PVOID, 100 as ULONG);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_BadRequestFlush(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Irp: PIRP;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    loop {
        Irp = IoCsqRemoveNextIrp(
            core::ptr::addr_of_mut!((*FdoExtension).BadRequestIoCsq),
            core::ptr::null_mut(),
        );

        if Irp.is_null() {
            break;
        }

        if DbgPrint(
            b"(%s:%d) USBPORT_BadRequestFlush: Irp - %p\n\0".as_ptr() as *const i8,
            b"queue.c\0".as_ptr() as *const i8,
            1344 as i32,
            Irp,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"queue.c\0".as_ptr() as *const i8,
                1344 as i32,
            );
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_NOT_CONNECTED;
        (*Irp).IoStatus.Information = (0 as _);
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AbortEndpoint(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
    Irp: PIRP,
) {

    let mut PendingList: PLIST_ENTRY;
    let mut PendingTransfer: PUSBPORT_TRANSFER;
    let mut ActiveList: PLIST_ENTRY;
    let mut ActiveTransfer: PUSBPORT_TRANSFER;

    // DPRINT_CORE!("USBPORT_AbortEndpoint: Irp - %p\n", Irp);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    if !Irp.is_null() {
        InsertTailList(
            core::ptr::addr_of_mut!((*Endpoint).AbortList),
            core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
        );
    }

    PendingList = (*Endpoint).PendingTransferList.Flink as PLIST_ENTRY;

    while !PendingList.is_null()
        && PendingList != core::ptr::addr_of_mut!((*Endpoint).PendingTransferList)
    {
        PendingTransfer = (PendingList as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        // DPRINT_CORE!("USBPORT_AbortEndpoint: Abort PendingTransfer - %p\n", PendingTransfer);

        (*PendingTransfer).Flags |= TRANSFER_FLAG_ABORTED as ULONG;

        PendingList = (*PendingTransfer).TransferLink.Flink as PLIST_ENTRY;
    }

    ActiveList = (*Endpoint).TransferList.Flink as PLIST_ENTRY;

    while !ActiveList.is_null()
        && ActiveList != core::ptr::addr_of_mut!((*Endpoint).TransferList)
    {
        ActiveTransfer = (ActiveList as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        // DPRINT_CORE!("USBPORT_AbortEndpoint: Abort ActiveTransfer - %p\n", ActiveTransfer);

        (*ActiveTransfer).Flags |= TRANSFER_FLAG_ABORTED as ULONG;

        if (*Endpoint).Flags as ULONG & ENDPOINT_FLAG_ABORTING as ULONG != 0 {
            (*ActiveTransfer).Flags |= TRANSFER_FLAG_DEVICE_GONE as ULONG;
        }

        ActiveList = (*ActiveTransfer).TransferLink.Flink as PLIST_ENTRY;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    USBPORT_InvalidateEndpointHandler(
        FdoDevice,
        Endpoint,
        INVALIDATE_ENDPOINT_INT_NEXT_SOF as ULONG,
    );

    USBPORT_FlushPendingTransfers(Endpoint);
    USBPORT_FlushCancelList(Endpoint);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort queue implementation
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBPORT_CORE
// #define NDEBUG_USBPORT_QUEUE
// #define NDEBUG_USBPORT_URB
// #include "usbdebug.h"
//
// VOID
// NTAPI
// USBPORT_InsertIdleIrp(IN PIO_CSQ Csq,
//                       IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_InsertIdleIrp: Irp - %p\n", Irp);
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      IdleIoCsq);
//
//     InsertTailList(&FdoExtension->IdleIrpList,
//                    &Irp->Tail.Overlay.ListEntry);
// }
//
// VOID
// NTAPI
// USBPORT_RemoveIdleIrp(IN PIO_CSQ Csq,
//                       IN PIRP Irp)
// {
//     DPRINT_QUEUE!("USBPORT_RemoveIdleIrp: Irp - %p\n", Irp);
//     RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
// }
//
// PIRP
// NTAPI
// USBPORT_PeekNextIdleIrp(IN PIO_CSQ Csq,
//                         IN PIRP Irp,
//                         IN PVOID PeekContext)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PLIST_ENTRY NextEntry;
//     PLIST_ENTRY ListHead;
//     PIRP NextIrp = NULL;
//
//     DPRINT_QUEUE!("USBPORT_PeekNextIdleIrp: Irp - %p, PeekContext - %p\n",
//                  Irp,
//                  PeekContext);
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      IdleIoCsq);
//
//     ListHead = &FdoExtension->IdleIrpList;
//
//     if (Irp)
//     {
//         NextEntry = Irp->Tail.Overlay.ListEntry.Flink;
//     }
//     else
//     {
//         NextEntry = ListHead->Flink;
//     }
//
//     while (NextEntry != ListHead)
//     {
//         NextIrp = CONTAINING_RECORD(NextEntry,
//                                     IRP,
//                                     Tail.Overlay.ListEntry);
//
//         if (!PeekContext)
//             break;
//
//         NextEntry = NextEntry->Flink;
//     }
//
//     return NextIrp;
// }
//
// VOID
// NTAPI
// USBPORT_AcquireIdleLock(IN PIO_CSQ Csq,
//                         IN PKIRQL Irql)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_AcquireIdleLock: ... \n");
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      IdleIoCsq);
//
//     KeAcquireSpinLock(&FdoExtension->IdleIoCsqSpinLock, Irql);
// }
//
// VOID
// NTAPI
// USBPORT_ReleaseIdleLock(IN PIO_CSQ Csq,
//                         IN KIRQL Irql)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_ReleaseIdleLock: ... \n");
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      IdleIoCsq);
//
//     KeReleaseSpinLock(&FdoExtension->IdleIoCsqSpinLock, Irql);
// }
//
// VOID
// NTAPI
// USBPORT_CompleteCanceledIdleIrp(IN PIO_CSQ Csq,
//                                 IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_CompleteCanceledIdleIrp: ... \n");
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      IdleIoCsq);
//
//     InterlockedDecrement(&FdoExtension->IdleLockCounter);
//
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// VOID
// NTAPI
// USBPORT_InsertBadRequest(IN PIO_CSQ Csq,
//                          IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_InsertBadRequest: Irp - %p\n", Irp);
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      BadRequestIoCsq);
//
//     InsertTailList(&FdoExtension->BadRequestList,
//                    &Irp->Tail.Overlay.ListEntry);
// }
//
// VOID
// NTAPI
// USBPORT_RemoveBadRequest(IN PIO_CSQ Csq,
//                          IN PIRP Irp)
// {
//     DPRINT_QUEUE!("USBPORT_RemoveBadRequest: Irp - %p\n", Irp);
//     RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
// }
//
// PIRP
// NTAPI
// USBPORT_PeekNextBadRequest(IN PIO_CSQ Csq,
//                            IN PIRP Irp,
//                            IN PVOID PeekContext)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PLIST_ENTRY NextEntry;
//     PLIST_ENTRY ListHead;
//     PIRP NextIrp = NULL;
//
//     DPRINT_QUEUE!("USBPORT_PeekNextBadRequest: Irp - %p, PeekContext - %p\n",
//                  Irp,
//                  PeekContext);
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      BadRequestIoCsq);
//
//     ListHead = &FdoExtension->BadRequestList;
//
//     if (Irp)
//     {
//         NextEntry = Irp->Tail.Overlay.ListEntry.Flink;
//     }
//     else
//     {
//         NextEntry = ListHead->Flink;
//     }
//
//     while (NextEntry != ListHead)
//     {
//         NextIrp = CONTAINING_RECORD(NextEntry,
//                                     IRP,
//                                     Tail.Overlay.ListEntry);
//
//         if (!PeekContext)
//             break;
//
//         NextEntry = NextEntry->Flink;
//     }
//
//     return NextIrp;
// }
//
// VOID
// NTAPI
// USBPORT_AcquireBadRequestLock(IN PIO_CSQ Csq,
//                               IN PKIRQL Irql)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_AcquireBadRequestLock: ... \n");
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      BadRequestIoCsq);
//
//     KeAcquireSpinLock(&FdoExtension->BadRequestIoCsqSpinLock, Irql);
// }
//
// VOID
// NTAPI
// USBPORT_ReleaseBadRequestLock(IN PIO_CSQ Csq,
//                               IN KIRQL Irql)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_ReleaseBadRequestLock: ... \n");
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      BadRequestIoCsq);
//
//     KeReleaseSpinLock(&FdoExtension->BadRequestIoCsqSpinLock, Irql);
// }
//
// VOID
// NTAPI
// USBPORT_CompleteCanceledBadRequest(IN PIO_CSQ Csq,
//                                    IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_QUEUE!("USBPORT_CompleteCanceledBadRequest: Irp - %p\n", Irp);
//
//     FdoExtension = CONTAINING_RECORD(Csq,
//                                      USBPORT_DEVICE_EXTENSION,
//                                      BadRequestIoCsq);
//
//     InterlockedDecrement(&FdoExtension->BadRequestLockCounter);
//
//     Irp->IoStatus.Status = STATUS_CANCELLED;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
// }
//
// VOID
// NTAPI
// USBPORT_InsertIrpInTable(IN PUSBPORT_IRP_TABLE IrpTable,
//                          IN PIRP Irp)
// {
//     ULONG ix;
//
//     DPRINT_CORE!("USBPORT_InsertIrpInTable: IrpTable - %p, Irp - %p\n",
//                 IrpTable,
//                 Irp);
//
//     ASSERT(IrpTable != NULL);
//
//     while (TRUE)
//     {
//         for (ix = 0; ix < 0x200; ix++)
//         {
//             if (IrpTable->irp[ix] == NULL)
//             {
//                 IrpTable->irp[ix] = Irp;
//
//                 if (ix > 0)
//                 {
//                     DPRINT_CORE!("USBPORT_InsertIrpInTable: ix - %x\n", ix);
//                 }
//
//                 return;
//             }
//         }
//
//         if (ix != 0x200)
//         {
//             KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
//         }
//
//         IrpTable->LinkNextTable = ExAllocatePoolWithTag(NonPagedPool,
//                                                         sizeof(USBPORT_IRP_TABLE),
//                                                         USB_PORT_TAG);
//
//         if (IrpTable->LinkNextTable == NULL)
//         {
//             KeBugCheckEx(BUGCODE_USB_DRIVER, 1, 0, 0, 0);
//         }
//
//         RtlZeroMemory(IrpTable->LinkNextTable, sizeof(USBPORT_IRP_TABLE));
//
//         IrpTable = IrpTable->LinkNextTable;
//     }
// }
//
// PIRP
// NTAPI
// USBPORT_RemoveIrpFromTable(IN PUSBPORT_IRP_TABLE IrpTable,
//                            IN PIRP Irp)
// {
//     ULONG ix;
//
//     DPRINT_CORE!("USBPORT_RemoveIrpFromTable: IrpTable - %p, Irp - %p\n",
//                 IrpTable,
//                 Irp);
//
//     ASSERT(IrpTable != NULL);
//
//     while (TRUE)
//     {
//         for (ix = 0; ix < 0x200; ix++)
//         {
//             if (IrpTable->irp[ix] == Irp)
//             {
//                 IrpTable->irp[ix] = NULL;
//
//                 if (ix > 0)
//                 {
//                     DPRINT_CORE!("USBPORT_RemoveIrpFromTable: ix - %x\n", ix);
//                 }
//
//                 return Irp;
//             }
//         }
//
//         if (IrpTable->LinkNextTable == NULL)
//             break;
//
//         IrpTable = IrpTable->LinkNextTable;
//         continue;
//     }
//
//     DPRINT1("USBPORT_RemoveIrpFromTable: return NULL. ix - %x\n", ix);
//     return NULL;
// }
//
// PIRP
// NTAPI
// USBPORT_RemoveActiveTransferIrp(IN PDEVICE_OBJECT FdoDevice,
//                                 IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_CORE!("USBPORT_RemoveActiveTransferIrp: Irp - %p\n", Irp);
//     FdoExtension = FdoDevice->DeviceExtension;
//     return USBPORT_RemoveIrpFromTable(FdoExtension->ActiveIrpTable, Irp);
// }
//
// PIRP
// NTAPI
// USBPORT_RemovePendingTransferIrp(IN PDEVICE_OBJECT FdoDevice,
//                                  IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_CORE!("USBPORT_RemovePendingTransferIrp: Irp - %p\n", Irp);
//     FdoExtension = FdoDevice->DeviceExtension;
//     return USBPORT_RemoveIrpFromTable(FdoExtension->PendingIrpTable, Irp);
// }
//
// VOID
// NTAPI
// USBPORT_FindUrbInIrpTable(IN PUSBPORT_IRP_TABLE IrpTable,
//                           IN PURB Urb,
//                           IN PIRP Irp)
// {
//     ULONG ix;
//     PIRP irp;
//     PURB urbIn;
//
//     DPRINT_CORE!("USBPORT_FindUrbInIrpTable: IrpTable - %p, Urb - %p, Irp - %p\n",
//                 IrpTable,
//                 Urb,
//                 Irp);
//
//     ASSERT(IrpTable != NULL);
//
//     do
//     {
//         for (ix = 0; ix < 0x200; ix++)
//         {
//             irp = IrpTable->irp[ix];
//
//             if (irp)
//             {
//                 urbIn = URB_FROM_IRP(irp);
//
//                 if (urbIn == Urb)
//                 {
//                     if (irp == Irp)
//                     {
//                         KeBugCheckEx(BUGCODE_USB_DRIVER,
//                                      4,
//                                      (ULONG_PTR)irp,
//                                      (ULONG_PTR)urbIn,
//                                      0);
//                     }
//
//                     KeBugCheckEx(BUGCODE_USB_DRIVER,
//                                  2,
//                                  (ULONG_PTR)irp,
//                                  (ULONG_PTR)Irp,
//                                  (ULONG_PTR)urbIn);
//                 }
//             }
//         }
//
//         IrpTable = IrpTable->LinkNextTable;
//     }
//     while (IrpTable);
// }
//
// PIRP
// NTAPI
// USBPORT_FindIrpInTable(IN PUSBPORT_IRP_TABLE IrpTable,
//                        IN PIRP Irp)
// {
//     ULONG ix;
//     PIRP irp;
//
//     DPRINT_CORE!("USBPORT_FindIrpInTable: IrpTable - %p, Irp - %p\n",
//                 IrpTable,
//                 Irp);
//
//     ASSERT(IrpTable != NULL);
//
//     do
//     {
//         for (ix = 0; ix < 0x200; ix++)
//         {
//             irp = IrpTable->irp[ix];
//
//             if (irp && irp == Irp)
//             {
//                 return irp;
//             }
//         }
//
//         IrpTable = IrpTable->LinkNextTable;
//     }
//     while (IrpTable->LinkNextTable);
//
//     DPRINT_CORE!("USBPORT_FindIrpInTable: Not found!!!\n");
//     return NULL;
// }
//
// PIRP
// NTAPI
// USBPORT_FindActiveTransferIrp(IN PDEVICE_OBJECT FdoDevice,
//                               IN PIRP Irp)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_CORE!("USBPORT_FindActiveTransferIrp: Irp - %p\n", Irp);
//     FdoExtension = FdoDevice->DeviceExtension;
//     return USBPORT_FindIrpInTable(FdoExtension->ActiveIrpTable, Irp);
// }
//
// VOID
// NTAPI
// USBPORT_CancelPendingTransferIrp(IN PDEVICE_OBJECT DeviceObject,
//                                  IN PIRP Irp)
// {
//     PURB Urb;
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_ENDPOINT Endpoint;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     KIRQL OldIrql;
//     PIRP irp;
//
//     DPRINT_CORE!("USBPORT_CancelPendingTransferIrp: DeviceObject - %p, Irp - %p\n",
//                 DeviceObject,
//                 Irp);
//
//     Urb = URB_FROM_IRP(Irp);
//     Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//     Endpoint = Transfer->Endpoint;
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = DeviceObject->DeviceExtension;
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     KeAcquireSpinLock(&FdoExtension->FlushPendingTransferSpinLock, &OldIrql);
//
//     irp = USBPORT_RemovePendingTransferIrp(FdoDevice, Irp);
//
//     if (!irp)
//     {
//         KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                           OldIrql);
//         return;
//     }
//
//     KeAcquireSpinLockAtDpcLevel(&Endpoint->EndpointSpinLock);
//
//     RemoveEntryList(&Transfer->TransferLink);
//
//     Transfer->TransferLink.Flink = NULL;
//     Transfer->TransferLink.Blink = NULL;
//
//     KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//     KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock, OldIrql);
//
//     USBPORT_CompleteTransfer(Transfer->Urb, USBD_STATUS_CANCELED);
// }
//
// VOID
// NTAPI
// USBPORT_CancelActiveTransferIrp(IN PDEVICE_OBJECT DeviceObject,
//                                 IN PIRP Irp)
// {
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PURB Urb;
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_ENDPOINT Endpoint;
//     PIRP irp;
//     PUSBPORT_TRANSFER SplitTransfer;
//     PLIST_ENTRY Entry;
//     KIRQL OldIrql;
//
//     DPRINT_CORE!("USBPORT_CancelActiveTransferIrp: Irp - %p\n", Irp);
//
//     PdoExtension = DeviceObject->DeviceExtension;
//     FdoDevice = PdoExtension->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     KeAcquireSpinLock(&FdoExtension->FlushTransferSpinLock, &OldIrql);
//
//     irp = USBPORT_FindActiveTransferIrp(FdoDevice, Irp);
//
//     if (!irp)
//     {
//         KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock, OldIrql);
//         return;
//     }
//
//     Urb = URB_FROM_IRP(irp);
//     Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//     Endpoint = Transfer->Endpoint;
//
//     DPRINT_CORE!("USBPORT_CancelActiveTransferIrp: irp - %p, Urb - %p, Transfer - %p\n",
//                 irp,
//                 Urb,
//                 Transfer);
//
//     KeAcquireSpinLockAtDpcLevel(&Endpoint->EndpointSpinLock);
//
//     Transfer->Flags |= TRANSFER_FLAG_CANCELED;
//
//     if (Transfer->Flags & TRANSFER_FLAG_PARENT)
//     {
//         KeAcquireSpinLockAtDpcLevel(&Transfer->TransferSpinLock);
//
//         Entry = Transfer->SplitTransfersList.Flink;
//
//         while (Entry && Entry != &Transfer->SplitTransfersList)
//         {
//             SplitTransfer = CONTAINING_RECORD(Entry,
//                                               USBPORT_TRANSFER,
//                                               SplitLink);
//
//             SplitTransfer->Flags |= TRANSFER_FLAG_CANCELED;
//
//             Entry = Entry->Flink;
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&Transfer->TransferSpinLock);
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//     KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock, OldIrql);
//
//     USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                       Endpoint,
//                                       INVALIDATE_ENDPOINT_WORKER_THREAD);
//     return;
// }
//
// VOID
// NTAPI
// USBPORT_FlushAbortList(IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PLIST_ENTRY Entry;
//     PUSBPORT_TRANSFER Transfer;
//     PLIST_ENTRY AbortList;
//     LIST_ENTRY List;
//     NTSTATUS Status;
//     PIRP Irp;
//     PURB Urb;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle = NULL;
//
//     DPRINT_CORE!("USBPORT_FlushAbortList: Endpoint - %p\n", Endpoint);
//
//     InitializeListHead(&List);
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     if (IsListEmpty(&Endpoint->AbortList))
//     {
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                           Endpoint->EndpointOldIrql);
//         return;
//     }
//
//     Entry = Endpoint->PendingTransferList.Flink;
//
//     while (Entry && Entry != &Endpoint->PendingTransferList)
//     {
//         Transfer = CONTAINING_RECORD(Entry,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         if (Transfer->Flags & TRANSFER_FLAG_ABORTED)
//         {
//             DPRINT_CORE!("USBPORT_FlushAbortList: Aborted PendingTransfer  - %p\n",
//                         Transfer);
//
//             KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                               Endpoint->EndpointOldIrql);
//             return;
//         }
//
//         Entry = Transfer->TransferLink.Flink;
//     }
//
//     Entry = Endpoint->TransferList.Flink;
//
//     while (Entry && Entry != &Endpoint->TransferList)
//     {
//         Transfer = CONTAINING_RECORD(Entry,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         if (Transfer->Flags & TRANSFER_FLAG_ABORTED)
//         {
//             DPRINT_CORE!("USBPORT_FlushAbortList: Aborted ActiveTransfer - %p\n",
//                         Transfer);
//
//             KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                               Endpoint->EndpointOldIrql);
//             return;
//         }
//
//         Entry = Transfer->TransferLink.Flink;
//     }
//
//     AbortList = &Endpoint->AbortList;
//
//     while (!IsListEmpty(AbortList))
//     {
//         //DbgBreakPoint();
//
//         Irp = CONTAINING_RECORD(AbortList->Flink,
//                                 IRP,
//                                 Tail.Overlay.ListEntry);
//
//         RemoveHeadList(AbortList);
//         InsertTailList(&List, &Irp->Tail.Overlay.ListEntry);
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     while (!IsListEmpty(&List))
//     {
//         //DbgBreakPoint();
//
//         Irp = CONTAINING_RECORD(List.Flink,
//                                 IRP,
//                                 Tail.Overlay.ListEntry);
//
//         RemoveHeadList(&List);
//
//         Urb = URB_FROM_IRP(Irp);
//
//         DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//         InterlockedDecrement(&DeviceHandle->DeviceHandleLock);
//
//         Status = USBPORT_USBDStatusToNtStatus(Urb, USBD_STATUS_SUCCESS);
//
//         DPRINT_CORE!("USBPORT_FlushAbortList: complete Irp - %p\n", Irp);
//
//         Irp->IoStatus.Status = Status;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_FlushCancelList(IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PUSBPORT_TRANSFER Transfer;
//     PIRP Irp;
//     KIRQL OldIrql;
//     KIRQL PrevIrql;
//
//     DPRINT_CORE!("USBPORT_FlushCancelList: ... \n");
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->FlushTransferSpinLock, &OldIrql);
//     KeAcquireSpinLockAtDpcLevel(&Endpoint->EndpointSpinLock);
//
//     while (!IsListEmpty(&Endpoint->CancelList))
//     {
//         Transfer = CONTAINING_RECORD(Endpoint->CancelList.Flink,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         RemoveHeadList(&Endpoint->CancelList);
//
//         Irp = Transfer->Irp;
//
//         if (Irp)
//         {
//             DPRINT("USBPORT_FlushCancelList: Irp - %p\n", Irp);
//
//             IoAcquireCancelSpinLock(&PrevIrql);
//             IoSetCancelRoutine(Irp, NULL);
//             IoReleaseCancelSpinLock(PrevIrql);
//
//             USBPORT_RemoveActiveTransferIrp(FdoDevice, Irp);
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//         KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock, OldIrql);
//
//         if (Endpoint->Flags & ENDPOINT_FLAG_NUKE)
//         {
//             USBPORT_CompleteTransfer(Transfer->Urb, USBD_STATUS_DEVICE_GONE);
//         }
//         else
//         {
//             if (Transfer->Flags & TRANSFER_FLAG_DEVICE_GONE)
//             {
//                 USBPORT_CompleteTransfer(Transfer->Urb,
//                                          USBD_STATUS_DEVICE_GONE);
//             }
//             else
//             {
//                 USBPORT_CompleteTransfer(Transfer->Urb,
//                                          USBD_STATUS_CANCELED);
//             }
//         }
//
//         KeAcquireSpinLock(&FdoExtension->FlushTransferSpinLock, &OldIrql);
//         KeAcquireSpinLockAtDpcLevel(&Endpoint->EndpointSpinLock);
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//     KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock, OldIrql);
//
//     USBPORT_FlushAbortList(Endpoint);
// }
//
// VOID
// NTAPI
// USBPORT_FlushPendingTransfers(IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     BOOLEAN IsMapTransfer;
//     BOOLEAN IsEnd = FALSE;
//     PLIST_ENTRY List;
//     PUSBPORT_TRANSFER Transfer;
//     PURB Urb;
//     PIRP Irp;
//     KIRQL OldIrql;
//     BOOLEAN Result;
//
//     DPRINT_CORE!("USBPORT_FlushPendingTransfers: Endpoint - %p\n", Endpoint);
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     if (InterlockedCompareExchange(&Endpoint->FlushPendingLock, 1, 0))
//     {
//         DPRINT_CORE!("USBPORT_FlushPendingTransfers: Endpoint Locked \n");
//         return;
//     }
//
//     while (TRUE)
//     {
//         IsMapTransfer = 0;
//
//         KeAcquireSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                           &OldIrql);
//
//         KeAcquireSpinLockAtDpcLevel(&Endpoint->EndpointSpinLock);
//
//         if (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND)
//         {
//             IsEnd = TRUE;
//
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//             KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                               OldIrql);
//             goto Next;
//         }
//
//         if (!(Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0))
//         {
//             if (!IsListEmpty(&Endpoint->TransferList))
//             {
//                 List = Endpoint->TransferList.Flink;
//
//                 while (List && List != &Endpoint->TransferList)
//                 {
//                     Transfer = CONTAINING_RECORD(List,
//                                                  USBPORT_TRANSFER,
//                                                  TransferLink);
//
//                     if (!(Transfer->Flags & TRANSFER_FLAG_SUBMITED))
//                     {
//                         KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//                         KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                                           OldIrql);
//
//                         IsEnd = TRUE;
//                         goto Worker;
//                     }
//
//                     List = Transfer->TransferLink.Flink;
//                 }
//             }
//         }
//
//         List = Endpoint->PendingTransferList.Flink;
//
//         if (List == NULL || IsListEmpty(&Endpoint->PendingTransferList))
//         {
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//             KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                               OldIrql);
//
//             IsEnd = TRUE;
//             goto Worker;
//         }
//
//         Transfer = CONTAINING_RECORD(List,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         if (Transfer->Irp)
//         {
//             DPRINT_CORE!("USBPORT_FlushPendingTransfers: Transfer->Irp->CancelRoutine - %p\n",
//                         Transfer->Irp->CancelRoutine);
//         }
//
//         if (Transfer->Irp &&
//             (IoSetCancelRoutine(Transfer->Irp, NULL) == NULL))
//         {
//             DPRINT_CORE!("USBPORT_FlushPendingTransfers: Transfer->Irp - %p\n",
//                         Transfer->Irp);
//
//             Transfer = NULL;
//             IsEnd = TRUE;
//         }
//
//         if (!Transfer)
//         {
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//             KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                               OldIrql);
//
//             if (IsMapTransfer)
//             {
//                 USBPORT_FlushMapTransfers(FdoDevice);
//                 goto Next;
//             }
//
//             goto Worker;
//         }
//
//         Irp = Transfer->Irp;
//         Urb = Transfer->Urb;
//
//         RemoveEntryList(&Transfer->TransferLink);
//         Transfer->TransferLink.Flink = NULL;
//         Transfer->TransferLink.Blink = NULL;
//
//         if (Irp)
//         {
//             Irp = USBPORT_RemovePendingTransferIrp(FdoDevice, Irp);
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//         KeReleaseSpinLock(&FdoExtension->FlushPendingTransferSpinLock,
//                           OldIrql);
//
//         KeAcquireSpinLock(&FdoExtension->FlushTransferSpinLock, &OldIrql);
//
//         if (Irp)
//         {
//             IoSetCancelRoutine(Irp, USBPORT_CancelActiveTransferIrp);
//
//             if (Irp->Cancel && IoSetCancelRoutine(Irp, NULL))
//             {
//                 DPRINT_CORE!("USBPORT_FlushPendingTransfers: irp - %p\n", Irp);
//
//                 KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock,
//                                   OldIrql);
//
//                 USBPORT_CompleteTransfer(Transfer->Urb, USBD_STATUS_CANCELED);
//                 goto Worker;
//             }
//
//             USBPORT_FindUrbInIrpTable(FdoExtension->ActiveIrpTable, Urb, Irp);
//             USBPORT_InsertIrpInTable(FdoExtension->ActiveIrpTable, Irp);
//         }
//
//         IsMapTransfer = USBPORT_QueueActiveUrbToEndpoint(Endpoint, Urb);
//
//         KeReleaseSpinLock(&FdoExtension->FlushTransferSpinLock, OldIrql);
//
//         if (IsMapTransfer)
//         {
//             USBPORT_FlushMapTransfers(FdoDevice);
//             goto Next;
//         }
//
// Worker:
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//         Result = USBPORT_EndpointWorker(Endpoint, FALSE);
//         KeLowerIrql(OldIrql);
//
//         if (Result)
//             USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                               Endpoint,
//                                               INVALIDATE_ENDPOINT_WORKER_THREAD);
//
// Next:
//         if (IsEnd)
//         {
//             InterlockedDecrement(&Endpoint->FlushPendingLock);
//             DPRINT_CORE!("USBPORT_FlushPendingTransfers: Endpoint Unlocked. Exit\n");
//             return;
//         }
//     }
// }
//
// VOID
// NTAPI
// USBPORT_QueuePendingUrbToEndpoint(IN PUSBPORT_ENDPOINT Endpoint,
//                                   IN PURB Urb)
// {
//     PUSBPORT_TRANSFER Transfer;
//
//     DPRINT_CORE!("USBPORT_QueuePendingUrbToEndpoint: Endpoint - %p, Urb - %p\n",
//                 Endpoint,
//                 Urb);
//
//     Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//     //FIXME USBPORT_ResetEndpointIdle();
//     InsertTailList(&Endpoint->PendingTransferList, &Transfer->TransferLink);
//     Urb->UrbHeader.Status = USBD_STATUS_PENDING;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_QueueActiveUrbToEndpoint(IN PUSBPORT_ENDPOINT Endpoint,
//                                  IN PURB Urb)
// {
//     PUSBPORT_TRANSFER Transfer;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     KIRQL OldIrql;
//
//     DPRINT_CORE!("USBPORT_QueueActiveUrbToEndpoint: Endpoint - %p, Urb - %p\n",
//                 Endpoint,
//                 Urb);
//
//     Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                       &Endpoint->EndpointOldIrql);
//
//     if ((Endpoint->Flags & ENDPOINT_FLAG_NUKE) ||
//         (Transfer->Flags & TRANSFER_FLAG_ABORTED))
//     {
//         InsertTailList(&Endpoint->CancelList, &Transfer->TransferLink);
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                           Endpoint->EndpointOldIrql);
//
//         //DPRINT_CORE!("USBPORT_QueueActiveUrbToEndpoint: return FALSE\n");
//         return FALSE;
//     }
//
//     if (Transfer->TransferParameters.TransferBufferLength == 0 ||
//         !(Endpoint->Flags & ENDPOINT_FLAG_DMA_TYPE))
//     {
//         InsertTailList(&Endpoint->TransferList, &Transfer->TransferLink);
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                           Endpoint->EndpointOldIrql);
//
//         //DPRINT_CORE!("USBPORT_QueueActiveUrbToEndpoint: return FALSE\n");
//         return FALSE;
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     KeAcquireSpinLock(&FdoExtension->MapTransferSpinLock, &OldIrql);
//
//     InsertTailList(&FdoExtension->MapTransferList, &Transfer->TransferLink);
//
//     DeviceHandle = Transfer->Urb->UrbHeader.UsbdDeviceHandle;
//     InterlockedIncrement(&DeviceHandle->DeviceHandleLock);
//
//     KeReleaseSpinLock(&FdoExtension->MapTransferSpinLock, OldIrql);
//
//     //DPRINT_CORE!("USBPORT_QueueActiveUrbToEndpoint: return TRUE\n");
//     return TRUE;
// }
//
// VOID
// NTAPI
// USBPORT_QueuePendingTransferIrp(IN PIRP Irp)
// {
//     PURB Urb;
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_ENDPOINT Endpoint;
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//
//     DPRINT_CORE!("USBPORT_QueuePendingTransferIrp: Irp - %p\n", Irp);
//
//     Urb = URB_FROM_IRP(Irp);
//
//     Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//     Endpoint = Transfer->Endpoint;
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Irp->IoStatus.Status = STATUS_PENDING;
//     IoMarkIrpPending(Irp);
//
//     IoSetCancelRoutine(Irp, USBPORT_CancelPendingTransferIrp);
//
//     if (Irp->Cancel && IoSetCancelRoutine(Irp, NULL))
//     {
//         USBPORT_CompleteTransfer(Urb, USBD_STATUS_CANCELED);
//     }
//     else
//     {
//         USBPORT_InsertIrpInTable(FdoExtension->PendingIrpTable, Irp);
//         USBPORT_QueuePendingUrbToEndpoint(Endpoint, Urb);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_QueueTransferUrb(IN PURB Urb)
// {
//     PUSBPORT_TRANSFER Transfer;
//     PUSBPORT_ENDPOINT Endpoint;
//     PIRP Irp;
//     PUSBPORT_DEVICE_HANDLE DeviceHandle;
//     PUSBPORT_TRANSFER_PARAMETERS Parameters;
//
//     DPRINT_CORE!("USBPORT_QueueTransferUrb: Urb - %p\n", Urb);
//
//     if (Urb->UrbControlTransfer.TransferFlags & USBD_DEFAULT_PIPE_TRANSFER)
//         Urb->UrbHeader.Function = URB_FUNCTION_CONTROL_TRANSFER;
//
//     Transfer = Urb->UrbControlTransfer.hca.Reserved8[0];
//     Parameters = &Transfer->TransferParameters;
//
//     Endpoint = Transfer->Endpoint;
//     Endpoint->Flags &= ~ENDPOINT_FLAG_QUEUENE_EMPTY;
//
//     Parameters->TransferBufferLength = Urb->UrbControlTransfer.TransferBufferLength;
//     Parameters->TransferFlags = Urb->UrbControlTransfer.TransferFlags;
//
//     Transfer->TransferBufferMDL = Urb->UrbControlTransfer.TransferBufferMDL;
//
//     if (Urb->UrbControlTransfer.TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//     {
//         Transfer->Direction = USBPORT_DMA_DIRECTION_FROM_DEVICE;
//     }
//     else
//     {
//         Transfer->Direction = USBPORT_DMA_DIRECTION_TO_DEVICE;
//     }
//
//     if (Endpoint->EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//     {
//         RtlCopyMemory(&Parameters->SetupPacket,
//                       Urb->UrbControlTransfer.SetupPacket,
//                       sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//     }
//
//     DPRINT_URB!("... URB TransferBufferLength - %x\n",
//            Urb->UrbControlTransfer.TransferBufferLength);
//
//     Urb->UrbControlTransfer.TransferBufferLength = 0;
//
//     Irp = Transfer->Irp;
//
//     if (Irp)
//     {
//         USBPORT_QueuePendingTransferIrp(Irp);
//     }
//     else
//     {
//         USBPORT_QueuePendingUrbToEndpoint(Endpoint, Urb);
//     }
//
//     DeviceHandle = Urb->UrbHeader.UsbdDeviceHandle;
//     InterlockedDecrement(&DeviceHandle->DeviceHandleLock);
//
//     USBPORT_FlushPendingTransfers(Endpoint);
//
//     DPRINT_URB!("... URB TransferBufferLength - %x\n",
//            Urb->UrbControlTransfer.TransferBufferLength);
//
//     if (Urb->UrbControlTransfer.TransferBufferLength)
//     {
//         PULONG Buffer;
//         ULONG BufferLength;
//         ULONG_PTR BufferEnd;
//         ULONG ix;
//
//         Buffer = Urb->UrbControlTransfer.TransferBuffer;
//         BufferLength = Urb->UrbControlTransfer.TransferBufferLength;
//         BufferEnd = (ULONG_PTR)Buffer + BufferLength;
//
//         DPRINT_URB!("URB TransferBuffer - %p\n", Buffer);
//
//         for (ix = 0; (ULONG_PTR)(Buffer + ix) < BufferEnd; ix++)
//         {
//             DPRINT_URB!("Buffer[%02X] - %p\n", ix, Buffer[ix]);
//         }
//     }
// }
//
// VOID
// NTAPI
// USBPORT_FlushAllEndpoints(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PLIST_ENTRY Entry;
//     PUSBPORT_ENDPOINT Endpoint;
//     LIST_ENTRY List;
//     KIRQL OldIrql;
//
//     DPRINT_CORE!("USBPORT_FlushAllEndpoints: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//     InitializeListHead(&List);
//
//     Entry = FdoExtension->EndpointList.Flink;
//
//     while (Entry && Entry != &FdoExtension->EndpointList)
//     {
//         Endpoint = CONTAINING_RECORD(Entry,
//                                      USBPORT_ENDPOINT,
//                                      EndpointLink);
//
//         if (USBPORT_GetEndpointState(Endpoint) != USBPORT_ENDPOINT_CLOSED)
//         {
//             InsertTailList(&List, &Endpoint->FlushLink);
//         }
//
//         Entry = Endpoint->EndpointLink.Flink;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//
//     while (!IsListEmpty(&List))
//     {
//         Endpoint = CONTAINING_RECORD(List.Flink,
//                                      USBPORT_ENDPOINT,
//                                      FlushLink);
//
//         RemoveHeadList(&List);
//
//         Endpoint->FlushLink.Flink = NULL;
//         Endpoint->FlushLink.Blink = NULL;
//
//         if (!IsListEmpty(&Endpoint->PendingTransferList))
//         {
//             USBPORT_FlushPendingTransfers(Endpoint);
//         }
//     }
//
//     DPRINT_CORE!("USBPORT_FlushAllEndpoints: exit\n");
// }
//
// ULONG
// NTAPI
// USBPORT_KillEndpointActiveTransfers(IN PDEVICE_OBJECT FdoDevice,
//                                     IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PLIST_ENTRY ActiveList;
//     PUSBPORT_TRANSFER Transfer;
//     ULONG KilledTransfers = 0;
//
//     DPRINT_CORE!("USBPORT_KillEndpointActiveTransfers \n");
//
//     ActiveList = Endpoint->TransferList.Flink;
//
//     while (ActiveList && ActiveList != &Endpoint->TransferList)
//     {
//         ++KilledTransfers;
//
//         Transfer = CONTAINING_RECORD(ActiveList,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         Transfer->Flags |= TRANSFER_FLAG_ABORTED;
//
//         ActiveList = Transfer->TransferLink.Flink;
//     }
//
//     USBPORT_FlushPendingTransfers(Endpoint);
//     USBPORT_FlushCancelList(Endpoint);
//
//     return KilledTransfers;
// }
//
// VOID
// NTAPI
// USBPORT_FlushController(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PLIST_ENTRY Entry;
//     PUSBPORT_ENDPOINT Endpoint;
//     ULONG KilledTransfers;
//     PLIST_ENTRY EndpointList;
//     KIRQL OldIrql;
//     LIST_ENTRY FlushList;
//
//     DPRINT_CORE!("USBPORT_FlushController \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     EndpointList = &FdoExtension->EndpointList;
//
//     while (TRUE)
//     {
//         KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//         InitializeListHead(&FlushList);
//
//         Entry = EndpointList->Flink;
//
//         if (!IsListEmpty(EndpointList))
//         {
//             while (Entry && Entry != EndpointList)
//             {
//                 Endpoint = CONTAINING_RECORD(Entry,
//                                              USBPORT_ENDPOINT,
//                                              EndpointLink);
//
//                 if (Endpoint->StateLast != USBPORT_ENDPOINT_REMOVE &&
//                     Endpoint->StateLast != USBPORT_ENDPOINT_CLOSED)
//                 {
//                     InterlockedIncrement(&Endpoint->LockCounter);
//                     InsertTailList(&FlushList, &Endpoint->FlushControllerLink);
//                 }
//
//                 Entry = Endpoint->EndpointLink.Flink;
//             }
//         }
//
//         KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//
//         KilledTransfers = 0;
//         while (!IsListEmpty(&FlushList))
//         {
//             Endpoint = CONTAINING_RECORD(FlushList.Flink,
//                                          USBPORT_ENDPOINT,
//                                          FlushControllerLink);
//
//             RemoveHeadList(&FlushList);
//
//             KilledTransfers += USBPORT_KillEndpointActiveTransfers(FdoDevice,
//                                                                    Endpoint);
//
//             InterlockedDecrement(&Endpoint->LockCounter);
//         }
//
//         if (!KilledTransfers)
//             break;
//
//         USBPORT_Wait(FdoDevice, 100);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_BadRequestFlush(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PIRP Irp;
//
//     DPRINT_QUEUE!("USBPORT_BadRequestFlush: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     while (TRUE)
//     {
//         Irp = IoCsqRemoveNextIrp(&FdoExtension->BadRequestIoCsq, 0);
//
//         if (!Irp)
//             break;
//
//         DPRINT1("USBPORT_BadRequestFlush: Irp - %p\n", Irp);
//
//         Irp->IoStatus.Status = STATUS_DEVICE_NOT_CONNECTED;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_AbortEndpoint(IN PDEVICE_OBJECT FdoDevice,
//                       IN PUSBPORT_ENDPOINT Endpoint,
//                       IN PIRP Irp)
// {
//     PLIST_ENTRY PendingList;
//     PUSBPORT_TRANSFER PendingTransfer;
//     PLIST_ENTRY ActiveList;
//     PUSBPORT_TRANSFER ActiveTransfer;
//
//     DPRINT_CORE!("USBPORT_AbortEndpoint: Irp - %p\n", Irp);
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     if (Irp)
//     {
//         InsertTailList(&Endpoint->AbortList, &Irp->Tail.Overlay.ListEntry);
//     }
//
//     PendingList = Endpoint->PendingTransferList.Flink;
//
//     while (PendingList && PendingList != &Endpoint->PendingTransferList)
//     {
//         PendingTransfer = CONTAINING_RECORD(PendingList,
//                                             USBPORT_TRANSFER,
//                                             TransferLink);
//
//         DPRINT_CORE!("USBPORT_AbortEndpoint: Abort PendingTransfer - %p\n",
//                     PendingTransfer);
//
//         PendingTransfer->Flags |= TRANSFER_FLAG_ABORTED;
//
//         PendingList = PendingTransfer->TransferLink.Flink;
//     }
//
//     ActiveList = Endpoint->TransferList.Flink;
//
//     while (ActiveList && ActiveList != &Endpoint->TransferList)
//     {
//         ActiveTransfer = CONTAINING_RECORD(ActiveList,
//                                            USBPORT_TRANSFER,
//                                            TransferLink);
//
//         DPRINT_CORE!("USBPORT_AbortEndpoint: Abort ActiveTransfer - %p\n",
//                     ActiveTransfer);
//
//         ActiveTransfer->Flags |= TRANSFER_FLAG_ABORTED;
//
//         if (Endpoint->Flags & ENDPOINT_FLAG_ABORTING)
//         {
//             ActiveTransfer->Flags |= TRANSFER_FLAG_DEVICE_GONE;
//         }
//
//         ActiveList = ActiveTransfer->TransferLink.Flink;
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                       Endpoint,
//                                       INVALIDATE_ENDPOINT_INT_NEXT_SOF);
//
//     USBPORT_FlushPendingTransfers(Endpoint);
//     USBPORT_FlushCancelList(Endpoint);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
