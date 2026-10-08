use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAcquireCancelSpinLock, IoAllocateMdl, IoFreeMdl, _LOCK_OPERATION_IoModifyAccess as IoModifyAccess, _LOCK_OPERATION_IoReadAccess as IoReadAccess, IoReleaseCancelSpinLock, KPROCESSOR_MODE, KWAIT_REASON, KeReleaseMutex, MmUnlockPages, ObReferenceObjectByHandle, PIO_STACK_LOCATION, PIRP, PLARGE_INTEGER, PDEVICE_OBJECT, PDRIVER_CANCEL};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSetCancelRoutine};

extern "system" {
    fn KeWaitForMutexObject( Mutex: PVOID, WaitReason: KWAIT_REASON, WaitMode: KPROCESSOR_MODE, Alertable: BOOLEAN, Timeout: PLARGE_INTEGER, ) -> NTSTATUS;
    fn AfdCancelHandler(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct AFD_LOCK_ENTRY {
    pub ListEntry: _LIST_ENTRY,
    pub Irp: PIRP,
    pub FileObject: *mut _FILE_OBJECT,
    pub Mdl: *mut _MDL,
    pub Buffer: PVOID,
    pub Length: ULONG,
    pub Offset: LARGE_INTEGER,
}

pub type PAFD_LOCK_ENTRY = *mut AFD_LOCK_ENTRY;
#[no_mangle]
pub unsafe extern "C" fn GetLockedData(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> PVOID {
    let _ = IrpSp;
    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]
}
#[no_mangle]
pub unsafe extern "C" fn LockRequest(
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
    Output: BOOLEAN,
    LockMode: *mut KPROCESSOR_MODE,
) -> PVOID {
    let mut LockFailed: BOOLEAN;

    (*Irp).MdlAddress = core::ptr::null_mut();

    match (*IrpSp).MajorFunction as u8 {
        IRP_MJ_DEVICE_CONTROL | IRP_MJ_INTERNAL_DEVICE_CONTROL => {
            (*Irp).MdlAddress = IoAllocateMdl(
                (*IrpSp).Parameters.DeviceIoControl.Type3InputBuffer,
                (*IrpSp).Parameters.DeviceIoControl.InputBufferLength,
                0u8,
                0u8,
                core::ptr::null_mut(),
            );
            if !(*Irp).MdlAddress.is_null() {
                // C: _SEH2_TRY { MmProbeAndLockPages } _SEH2_EXCEPT { LockFailed = TRUE; }
                LockFailed = (ntkrnl_sys::probe::MmTryProbeAndLockPages(
                    (*Irp).MdlAddress,
                    (*Irp).RequestorMode,
                    IoModifyAccess as _,
                ) < 0) as BOOLEAN;

                if LockFailed != 0 {
                    IoFreeMdl((*Irp).MdlAddress);
                    (*Irp).MdlAddress = core::ptr::null_mut();
                    return core::ptr::null_mut();
                }

                (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] =
                    MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority);
                if (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1].is_null() {
                    MmUnlockPages((*Irp).MdlAddress);
                    IoFreeMdl((*Irp).MdlAddress);
                    (*Irp).MdlAddress = core::ptr::null_mut();
                    return core::ptr::null_mut();
                }

                (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = ExAllocatePoolWithTag(
                    NonPagedPool,
                    MmGetMdlByteCount((*Irp).MdlAddress) as usize,
                    TAG_AFD_DATA_BUFFER,
                );

                if (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0].is_null() {
                    MmUnlockPages((*Irp).MdlAddress);
                    IoFreeMdl((*Irp).MdlAddress);
                    (*Irp).MdlAddress = core::ptr::null_mut();
                    return core::ptr::null_mut();
                }

                RtlCopyMemory(
                    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0],
                    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1],
                    MmGetMdlByteCount((*Irp).MdlAddress) as usize,
                );

                if Output == 0 {
                    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = core::ptr::null_mut();
                }

                if !LockMode.is_null() {
                    *LockMode = 1i8;
                }
            } else {
                return core::ptr::null_mut();
            }
        }

        IRP_MJ_READ | IRP_MJ_WRITE => {
            (*Irp).MdlAddress = IoAllocateMdl(
                (*Irp).UserBuffer,
                if (*IrpSp).MajorFunction as u8 == IRP_MJ_READ {
                    (*IrpSp).Parameters.Read.Length
                } else {
                    (*IrpSp).Parameters.Write.Length
                },
                0u8,
                0u8,
                core::ptr::null_mut(),
            );
            if !(*Irp).MdlAddress.is_null() {
                // C: _SEH2_TRY { MmProbeAndLockPages } _SEH2_EXCEPT { LockFailed = TRUE; }
                LockFailed = (ntkrnl_sys::probe::MmTryProbeAndLockPages(
                    (*Irp).MdlAddress,
                    (*Irp).RequestorMode,
                    IoModifyAccess as _,
                ) < 0) as BOOLEAN;

                if LockFailed != 0 {
                    IoFreeMdl((*Irp).MdlAddress);
                    (*Irp).MdlAddress = core::ptr::null_mut();
                    return core::ptr::null_mut();
                }

                let AfdInfo = ExAllocatePoolWithTag(
                    NonPagedPool,
                    (core::mem::size_of::<AFD_RECV_INFO>() + core::mem::size_of::<AFD_WSABUF>())
                        as usize,
                    TAG_AFD_DATA_BUFFER,
                ) as PAFD_RECV_INFO;

                if AfdInfo.is_null() {
                    MmUnlockPages((*Irp).MdlAddress);
                    IoFreeMdl((*Irp).MdlAddress);
                    (*Irp).MdlAddress = core::ptr::null_mut();
                    return core::ptr::null_mut();
                }

                (*AfdInfo).BufferArray = (AfdInfo as usize + core::mem::size_of::<AFD_RECV_INFO>())
                    as PAFD_WSABUF;
                (*AfdInfo).BufferCount = 1;

                (*AfdInfo).AfdFlags = 0;
                (*AfdInfo).TdiFlags = 0;

                (*(*AfdInfo).BufferArray.offset(0)).buf =
                    MmGetSystemAddressForMdl((*Irp).MdlAddress) as PCHAR;
                (*(*AfdInfo).BufferArray.offset(0)).len = MmGetMdlByteCount((*Irp).MdlAddress);

                (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = AfdInfo as PVOID;

                (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = core::ptr::null_mut();

                if !LockMode.is_null() {
                    *LockMode = 0i8;
                }
            } else {
                return core::ptr::null_mut();
            }
        }

        _ => {
            return core::ptr::null_mut();
        }
    }

    return GetLockedData(Irp, IrpSp);
}
#[no_mangle]
pub unsafe extern "C" fn UnlockRequest(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) {
    let _ = IrpSp;

    if !(*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1].is_null() {
        RtlCopyMemory(
            (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1],
            (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0],
            MmGetMdlByteCount((*Irp).MdlAddress) as usize,
        );
    }

    ExFreePoolWithTag(
        (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0],
        TAG_AFD_DATA_BUFFER,
    );
    MmUnlockPages((*Irp).MdlAddress);
    IoFreeMdl((*Irp).MdlAddress);
    (*Irp).MdlAddress = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn LockBuffers(
    Buf: PAFD_WSABUF,
    Count: UINT,
    AddressBuf: PVOID,
    AddressLen: PINT,
    Write: BOOLEAN,
    LockAddress: BOOLEAN,
    LockMode: KPROCESSOR_MODE,
) -> PAFD_WSABUF {
    let Lock = if LockAddress != 0 { 2 } else { 0 };
    let Size = (core::mem::size_of::<AFD_WSABUF>() + core::mem::size_of::<AFD_MAPBUF>()) * (Count as usize + Lock);
    let NewBuf = ExAllocatePoolWithTag(PagedPool, Size, TAG_AFD_WSA_BUFFER) as PAFD_WSABUF;
    let mut LockFailed: BOOLEAN;
    let MapBuf: PAFD_MAPBUF;

    AFD_DbgPrint!(MID_TRACE, b"Called(%p)\n\0".as_ptr() as PCHAR, NewBuf);

    if !NewBuf.is_null() {
        RtlZeroMemory(NewBuf as PVOID, Size);

        MapBuf = NewBuf.offset(Count as isize + Lock as isize ) as PAFD_MAPBUF;

        {
            RtlCopyMemory(
                NewBuf as PVOID,
                Buf as PVOID,
                core::mem::size_of::<AFD_WSABUF>() * Count as usize,
            );
            let mut Count = Count;
            if LockAddress != 0 {
                if !AddressBuf.is_null() && !AddressLen.is_null() {
                    (*NewBuf.offset(Count as isize)).buf = AddressBuf as PCHAR;
                    (*NewBuf.offset(Count as isize)).len = *AddressLen as u32;
                    (*NewBuf.offset((Count + 1) as isize)).buf = AddressLen as PCHAR;
                    (*NewBuf.offset((Count + 1) as isize)).len = core::mem::size_of::<INT>() as u32;
                }
                Count += 2;
            }

            for i in 0..Count {
                AFD_DbgPrint!(
                    MID_TRACE,
                    b"Locking buffer %u (%p:%u)\n\0".as_ptr() as PCHAR,
                    i,
                    (*NewBuf.offset(i as isize)).buf,
                    (*NewBuf.offset(i as isize)).len,
                );

                if !(*NewBuf.offset(i as isize)).buf.is_null() && (*NewBuf.offset(i as isize)).len != 0 {
                    (*MapBuf.offset(i as isize)).Mdl = IoAllocateMdl(
                        (*NewBuf.offset(i as isize)).buf as PVOID,
                        (*NewBuf.offset(i as isize)).len,
                        0,
                        0,
                        core::ptr::null_mut(),
                    );
                } else {
                    (*MapBuf.offset(i as isize)).Mdl = core::ptr::null_mut();
                    continue;
                }

                AFD_DbgPrint!(MID_TRACE, b"NewMdl @ %p\n\0".as_ptr() as PCHAR, (*MapBuf.offset(i as isize)).Mdl);

                if !(*MapBuf.offset(i as isize)).Mdl.is_null() {
                    AFD_DbgPrint!(MID_TRACE, b"Probe and lock pages\n\0".as_ptr() as PCHAR);
                    // C: _SEH2_TRY { MmProbeAndLockPages } _SEH2_EXCEPT { LockFailed = TRUE; }
                    LockFailed = (ntkrnl_sys::probe::MmTryProbeAndLockPages(
                        (*MapBuf.offset(i as isize)).Mdl,
                        LockMode,
                        if Write != 0 { IoModifyAccess } else { IoReadAccess } as _,
                    ) < 0) as BOOLEAN;
                    AFD_DbgPrint!(MID_TRACE, b"MmProbeAndLock finished\n\0".as_ptr() as PCHAR);

                    if LockFailed != 0 {
                        AFD_DbgPrint!(MIN_TRACE, b"Failed to lock pages\n\0".as_ptr() as PCHAR);
                        IoFreeMdl((*MapBuf.offset(i as isize)).Mdl);
                        (*MapBuf.offset(i as isize)).Mdl = core::ptr::null_mut();
                        ExFreePoolWithTag(NewBuf as PVOID, TAG_AFD_WSA_BUFFER);
                        return core::ptr::null_mut();
                    }
                } else {
                    ExFreePoolWithTag(NewBuf as PVOID, TAG_AFD_WSA_BUFFER);
                    return core::ptr::null_mut();
                }
            }
        }
    }

    AFD_DbgPrint!(MID_TRACE, b"Leaving %p\n\0".as_ptr() as PCHAR, NewBuf);
    NewBuf
}
#[no_mangle]
pub unsafe extern "C" fn UnlockBuffers(Buf: PAFD_WSABUF, Count: UINT, Address: BOOL) {
    let Lock = if Address != 0 { 2 } else { 0 };
    let Map = Buf.add((Count + Lock) as usize) as PAFD_MAPBUF;

    if Buf.is_null() {
        return;
    }

    for i in 0..(Count + Lock) {
        if !(*Map.add(i as usize)).Mdl.is_null() {
            MmUnlockPages((*Map.add(i as usize)).Mdl);
            IoFreeMdl((*Map.add(i as usize)).Mdl);
            (*Map.add(i as usize)).Mdl = core::ptr::null_mut();
        }
    }

    ExFreePoolWithTag(Buf as PVOID, TAG_AFD_WSA_BUFFER);
}
#[no_mangle]
pub unsafe extern "C" fn LockHandles(HandleArray: PAFD_HANDLE, HandleCount: UINT) -> PAFD_HANDLE {
    let mut i: UINT;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    let FileObjects = ExAllocatePoolWithTag(
        NonPagedPool,
        (HandleCount as usize) * core::mem::size_of::<AFD_HANDLE>(),
        TAG_AFD_POLL_HANDLE,
    ) as PAFD_HANDLE;

    i = 0;
    while !FileObjects.is_null() && i < HandleCount {
        (*FileObjects.offset(i as isize)).Status = 0;
        (*FileObjects.offset(i as isize)).Events = (*HandleArray.offset(i as isize)).Events;
        (*FileObjects.offset(i as isize)).Handle = 0;
        if (*HandleArray.offset(i as isize)).Handle == 0 {
            i += 1;
            continue;
        }
        if NT_SUCCESS(Status) {
            Status = ObReferenceObjectByHandle(
                (*HandleArray.offset(i as isize)).Handle as PVOID,
                FILE_ALL_ACCESS,
                core::ptr::null_mut(),
                0i32 as KPROCESSOR_MODE,
                core::ptr::addr_of_mut!((*FileObjects.offset(i as isize)).Handle) as *mut PVOID,
                core::ptr::null_mut(),
            );
        }

        if !NT_SUCCESS(Status) {
            AFD_DbgPrint!(MIN_TRACE, b"Failed to reference handles (0x%x)\n\0".as_ptr() as PCHAR, Status);
            (*FileObjects.offset(i as isize)).Handle = 0;
        }
        i += 1;
    }

    if !NT_SUCCESS(Status) {
        UnlockHandles(FileObjects, HandleCount);
        return core::ptr::null_mut();
    }

    FileObjects
}
#[no_mangle]
pub unsafe extern "C" fn UnlockHandles(HandleArray: PAFD_HANDLE, HandleCount: UINT) {
    let mut i: UINT;

    i = 0;
    while i < HandleCount {
        if (*HandleArray.offset(i as isize)).Handle != 0 {
            ObDereferenceObject((*HandleArray.offset(i as isize)).Handle as PVOID);
        }
        i += 1;
    }

    ExFreePoolWithTag(HandleArray as PVOID, TAG_AFD_POLL_HANDLE);
}
#[no_mangle]
pub unsafe extern "C" fn SocketAcquireStateLock(FCB: PAFD_FCB) -> BOOLEAN {
    if FCB.is_null() {
        return 0u8;
    }

    let result = KeWaitForMutexObject(
        core::ptr::addr_of_mut!((*FCB).Mutex) as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    if result == 0 { 1u8 } else { 0u8 }
}
#[no_mangle]
pub unsafe extern "C" fn SocketStateUnlock(FCB: PAFD_FCB) {
    KeReleaseMutex(core::ptr::addr_of_mut!((*FCB).Mutex), 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn UnlockAndMaybeComplete(
    FCB: PAFD_FCB,
    Status: NTSTATUS,
    Irp: PIRP,
    Information: UINT,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = Information as _;
    if !(*Irp).MdlAddress.is_null() {
        UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
    }
    IoSetCancelRoutine(Irp, None);
    SocketStateUnlock(FCB);
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LostSocket(Irp: PIRP) -> NTSTATUS {
    let Status = STATUS_FILE_CLOSED;
    AFD_DbgPrint!(MIN_TRACE, b"Called.\n\0".as_ptr() as PCHAR);
    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    if !(*Irp).MdlAddress.is_null() {
        UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
    }
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn QueueUserModeIrp(FCB: PAFD_FCB, Irp: PIRP, Function: UINT) -> NTSTATUS {
    let Status: NTSTATUS;

    InsertTailList(
        &mut (*FCB).PendingIrpList[Function as usize],
        &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry
    );

    IoAcquireCancelSpinLock(&mut (*Irp).CancelIrql);
    if (*Irp).Cancel == 0 {
        IoSetCancelRoutine(
            Irp,
            core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP), PDRIVER_CANCEL>(
                AfdCancelHandler,
            ),
        );
        IoReleaseCancelSpinLock((*Irp).CancelIrql);
        IoMarkIrpPending(Irp);
        Status = STATUS_PENDING;
    } else {
        AfdCancelHandler(
            (*IoGetCurrentIrpStackLocation(Irp)).DeviceObject,
            Irp
        );
        Status = STATUS_CANCELLED;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LeaveIrpUntilLater(FCB: PAFD_FCB, Irp: PIRP, Function: UINT) -> NTSTATUS {
    let Status: NTSTATUS;

    Status = QueueUserModeIrp(FCB, Irp, Function);

    SocketStateUnlock(FCB);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/lock.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// PVOID GetLockedData(PIRP Irp, PIO_STACK_LOCATION IrpSp)
// {
//     ASSERT(Irp->MdlAddress);
//     ASSERT(Irp->Tail.Overlay.DriverContext[0]);
//
//     UNREFERENCED_PARAMETER(IrpSp);
//
//     return Irp->Tail.Overlay.DriverContext[0];
// }
//
// /* Lock a method_neither request so it'll be available from DISPATCH_LEVEL */
// PVOID LockRequest( PIRP Irp,
//                    PIO_STACK_LOCATION IrpSp,
//                    BOOLEAN Output,
//                    KPROCESSOR_MODE *LockMode) {
//     BOOLEAN LockFailed = FALSE;
//
//     ASSERT(!Irp->MdlAddress);
//
//     switch (IrpSp->MajorFunction)
//     {
//         case IRP_MJ_DEVICE_CONTROL:
//         case IRP_MJ_INTERNAL_DEVICE_CONTROL:
//             ASSERT(IrpSp->Parameters.DeviceIoControl.Type3InputBuffer);
//             ASSERT(IrpSp->Parameters.DeviceIoControl.InputBufferLength);
//
//
//             Irp->MdlAddress =
//             IoAllocateMdl( IrpSp->Parameters.DeviceIoControl.Type3InputBuffer,
//                           IrpSp->Parameters.DeviceIoControl.InputBufferLength,
//                           FALSE,
//                           FALSE,
//                           NULL );
//             if( Irp->MdlAddress ) {
//                 _SEH2_TRY {
//                     MmProbeAndLockPages( Irp->MdlAddress, Irp->RequestorMode, IoModifyAccess );
//                 } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//                     LockFailed = TRUE;
//                 } _SEH2_END;
//
//                 if( LockFailed ) {
//                     AFD_DbgPrint!(MIN_TRACE,("Failed to lock pages\n"));
//                     IoFreeMdl( Irp->MdlAddress );
//                     Irp->MdlAddress = NULL;
//                     return NULL;
//                 }
//
//                 /* The mapped address goes in index 1 */
//                 Irp->Tail.Overlay.DriverContext[1] = MmGetSystemAddressForMdlSafe(Irp->MdlAddress, NormalPagePriority);
//                 if (!Irp->Tail.Overlay.DriverContext[1])
//                 {
//                     AFD_DbgPrint!(MIN_TRACE,("Failed to get mapped address\n"));
//                     MmUnlockPages(Irp->MdlAddress);
//                     IoFreeMdl( Irp->MdlAddress );
//                     Irp->MdlAddress = NULL;
//                     return NULL;
//                 }
//
//                 /* The allocated address goes in index 0 */
//                 Irp->Tail.Overlay.DriverContext[0] = ExAllocatePoolWithTag(NonPagedPool,
//                                                                            MmGetMdlByteCount(Irp->MdlAddress),
//                                                                            TAG_AFD_DATA_BUFFER);
//
//                 if (!Irp->Tail.Overlay.DriverContext[0])
//                 {
//                     AFD_DbgPrint!(MIN_TRACE,("Failed to allocate memory\n"));
//                     MmUnlockPages(Irp->MdlAddress);
//                     IoFreeMdl( Irp->MdlAddress );
//                     Irp->MdlAddress = NULL;
//                     return NULL;
//                 }
//
//                 RtlCopyMemory(Irp->Tail.Overlay.DriverContext[0],
//                               Irp->Tail.Overlay.DriverContext[1],
//                               MmGetMdlByteCount(Irp->MdlAddress));
//
//                 /* If we don't want a copy back, we zero the mapped address pointer */
//                 if (!Output)
//                 {
//                     Irp->Tail.Overlay.DriverContext[1] = NULL;
//                 }
//
//                 /* We're using a user-mode buffer directly */
//                 if (LockMode != NULL)
//                 {
//                     *LockMode = UserMode;
//                 }
//             }
//             else return NULL;
//             break;
//
//         case IRP_MJ_READ:
//         case IRP_MJ_WRITE:
//             ASSERT(Irp->UserBuffer);
//
//             Irp->MdlAddress =
//             IoAllocateMdl(Irp->UserBuffer,
//                           (IrpSp->MajorFunction == IRP_MJ_READ) ?
//                                 IrpSp->Parameters.Read.Length : IrpSp->Parameters.Write.Length,
//                           FALSE,
//                           FALSE,
//                           NULL );
//             if( Irp->MdlAddress ) {
//                 PAFD_RECV_INFO AfdInfo;
//
//                 _SEH2_TRY {
//                     MmProbeAndLockPages( Irp->MdlAddress, Irp->RequestorMode, IoModifyAccess );
//                 } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//                     LockFailed = TRUE;
//                 } _SEH2_END;
//
//                 if( LockFailed ) {
//                     AFD_DbgPrint!(MIN_TRACE,("Failed to lock pages\n"));
//                     IoFreeMdl( Irp->MdlAddress );
//                     Irp->MdlAddress = NULL;
//                     return NULL;
//                 }
//
//                 /* We need to create the info struct that AFD expects for all send/recv requests */
//                 AfdInfo = ExAllocatePoolWithTag(NonPagedPool,
//                                                 sizeof(AFD_RECV_INFO) + sizeof(AFD_WSABUF),
//                                                 TAG_AFD_DATA_BUFFER);
//
//                 if (!AfdInfo)
//                 {
//                     AFD_DbgPrint!(MIN_TRACE,("Failed to allocate memory\n"));
//                     MmUnlockPages(Irp->MdlAddress);
//                     IoFreeMdl( Irp->MdlAddress );
//                     Irp->MdlAddress = NULL;
//                     return NULL;
//                 }
//
//                 /* We'll append the buffer array to this struct */
//                 AfdInfo->BufferArray = (PAFD_WSABUF)(AfdInfo + 1);
//                 AfdInfo->BufferCount = 1;
//
//                 /* Setup the default flags values */
//                 AfdInfo->AfdFlags = 0;
//                 AfdInfo->TdiFlags = 0;
//
//                 /* Now build the buffer array */
//                 AfdInfo->BufferArray[0].buf = MmGetSystemAddressForMdl(Irp->MdlAddress);
//                 AfdInfo->BufferArray[0].len = MmGetMdlByteCount(Irp->MdlAddress);
//
//                 /* Store the struct where AFD expects */
//                 Irp->Tail.Overlay.DriverContext[0] = AfdInfo;
//
//                 /* Don't copy anything out */
//                 Irp->Tail.Overlay.DriverContext[1] = NULL;
//
//                 /* We're using a placeholder buffer that we allocated */
//                 if (LockMode != NULL)
//                 {
//                     *LockMode = KernelMode;
//                 }
//             }
//             else return NULL;
//             break;
//
//         default:
//             ASSERT(FALSE);
//             return NULL;
//     }
//
//     return GetLockedData(Irp, IrpSp);
// }
//
// VOID UnlockRequest( PIRP Irp, PIO_STACK_LOCATION IrpSp )
// {
//     ASSERT(Irp->MdlAddress);
//     ASSERT(Irp->Tail.Overlay.DriverContext[0]);
//
//     UNREFERENCED_PARAMETER(IrpSp);
//
//     /* Check if we need to copy stuff back */
//     if (Irp->Tail.Overlay.DriverContext[1] != NULL)
//     {
//         RtlCopyMemory(Irp->Tail.Overlay.DriverContext[1],
//                       Irp->Tail.Overlay.DriverContext[0],
//                       MmGetMdlByteCount(Irp->MdlAddress));
//     }
//
//     ExFreePoolWithTag(Irp->Tail.Overlay.DriverContext[0], TAG_AFD_DATA_BUFFER);
//     MmUnlockPages( Irp->MdlAddress );
//     IoFreeMdl( Irp->MdlAddress );
//     Irp->MdlAddress = NULL;
// }
//
// /* Note: We add an extra buffer if LockAddress is true.  This allows us to
//  * treat the address buffer as an ordinary client buffer.  It's only used
//  * for datagrams. */
//
// PAFD_WSABUF LockBuffers( PAFD_WSABUF Buf, UINT Count,
//                          PVOID AddressBuf, PINT AddressLen,
//                          BOOLEAN Write, BOOLEAN LockAddress,
//                          KPROCESSOR_MODE LockMode) {
//     UINT i;
//     /* Copy the buffer array so we don't lose it */
//     UINT Lock = LockAddress ? 2 : 0;
//     UINT Size = (sizeof(AFD_WSABUF) + sizeof(AFD_MAPBUF)) * (Count + Lock);
//     PAFD_WSABUF NewBuf = ExAllocatePoolWithTag(PagedPool, Size, TAG_AFD_WSA_BUFFER);
//     BOOLEAN LockFailed = FALSE;
//     PAFD_MAPBUF MapBuf;
//
//     AFD_DbgPrint!(MID_TRACE,("Called(%p)\n", NewBuf));
//
//     if( NewBuf ) {
//         RtlZeroMemory(NewBuf, Size);
//
//         MapBuf = (PAFD_MAPBUF)(NewBuf + Count + Lock);
//
//         _SEH2_TRY {
//             RtlCopyMemory( NewBuf, Buf, sizeof(AFD_WSABUF) * Count );
//             if( LockAddress ) {
//                 if (AddressBuf && AddressLen) {
//                     NewBuf[Count].buf = AddressBuf;
//                     NewBuf[Count].len = *AddressLen;
//                     NewBuf[Count + 1].buf = (PVOID)AddressLen;
//                     NewBuf[Count + 1].len = sizeof(*AddressLen);
//                 }
//                 Count += 2;
//             }
//         } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//             AFD_DbgPrint!(MIN_TRACE,("Access violation copying buffer info "
//                                     "from userland (%p %p)\n",
//                                     Buf, AddressLen));
//             ExFreePoolWithTag(NewBuf, TAG_AFD_WSA_BUFFER);
//             _SEH2_YIELD(return NULL);
//         } _SEH2_END;
//
//         for( i = 0; i < Count; i++ ) {
//             AFD_DbgPrint!(MID_TRACE,("Locking buffer %u (%p:%u)\n",
//                                     i, NewBuf[i].buf, NewBuf[i].len));
//
//             if( NewBuf[i].buf && NewBuf[i].len ) {
//                 MapBuf[i].Mdl = IoAllocateMdl( NewBuf[i].buf,
//                                                NewBuf[i].len,
//                                                FALSE,
//                                                FALSE,
//                                                NULL );
//             } else {
//                 MapBuf[i].Mdl = NULL;
//                 continue;
//             }
//
//             AFD_DbgPrint!(MID_TRACE,("NewMdl @ %p\n", MapBuf[i].Mdl));
//
//             if( MapBuf[i].Mdl ) {
//                 AFD_DbgPrint!(MID_TRACE,("Probe and lock pages\n"));
//                 _SEH2_TRY {
//                     MmProbeAndLockPages( MapBuf[i].Mdl, LockMode,
//                                          Write ? IoModifyAccess : IoReadAccess );
//                 } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//                     LockFailed = TRUE;
//                 } _SEH2_END;
//                 AFD_DbgPrint!(MID_TRACE,("MmProbeAndLock finished\n"));
//
//                 if( LockFailed ) {
//             AFD_DbgPrint!(MIN_TRACE,("Failed to lock pages\n"));
//                     IoFreeMdl( MapBuf[i].Mdl );
//                     MapBuf[i].Mdl = NULL;
//                     ExFreePoolWithTag(NewBuf, TAG_AFD_WSA_BUFFER);
//                     return NULL;
//                 }
//             } else {
//                 ExFreePoolWithTag(NewBuf, TAG_AFD_WSA_BUFFER);
//                 return NULL;
//             }
//         }
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Leaving %p\n", NewBuf));
//
//     return NewBuf;
// }
//
// VOID UnlockBuffers( PAFD_WSABUF Buf, UINT Count, BOOL Address ) {
//     UINT Lock = Address ? 2 : 0;
//     PAFD_MAPBUF Map = (PAFD_MAPBUF)(Buf + Count + Lock);
//     UINT i;
//
//     if( !Buf ) return;
//
//     for( i = 0; i < Count + Lock; i++ ) {
//         if( Map[i].Mdl ) {
//             MmUnlockPages( Map[i].Mdl );
//             IoFreeMdl( Map[i].Mdl );
//             Map[i].Mdl = NULL;
//         }
//     }
//
//     ExFreePoolWithTag(Buf, TAG_AFD_WSA_BUFFER);
//     Buf = NULL;
// }
//
// /* Produce a kernel-land handle array with handles replaced by object
//  * pointers.  This will allow the system to do proper alerting */
// PAFD_HANDLE LockHandles( PAFD_HANDLE HandleArray, UINT HandleCount ) {
//     UINT i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAFD_HANDLE FileObjects = ExAllocatePoolWithTag(NonPagedPool,
//                                                     HandleCount * sizeof(AFD_HANDLE),
//                                                     TAG_AFD_POLL_HANDLE);
//
//     for( i = 0; FileObjects && i < HandleCount; i++ ) {
//         FileObjects[i].Status = 0;
//         FileObjects[i].Events = HandleArray[i].Events;
//         FileObjects[i].Handle = 0;
//         if( !HandleArray[i].Handle ) continue;
//         if( NT_SUCCESS(Status) ) {
//                 Status = ObReferenceObjectByHandle
//                     ( (PVOID)HandleArray[i].Handle,
//                       FILE_ALL_ACCESS,
//                       NULL,
//                        KernelMode,
//                        (PVOID*)&FileObjects[i].Handle,
//                        NULL );
//         }
//
//         if( !NT_SUCCESS(Status) )
//         {
//             AFD_DbgPrint!(MIN_TRACE,("Failed to reference handles (0x%x)\n", Status));
//             FileObjects[i].Handle = 0;
//         }
//     }
//
//     if( !NT_SUCCESS(Status) ) {
//         UnlockHandles( FileObjects, HandleCount );
//         return NULL;
//     }
//
//     return FileObjects;
// }
//
// VOID UnlockHandles( PAFD_HANDLE HandleArray, UINT HandleCount ) {
//     UINT i;
//
//     for( i = 0; i < HandleCount; i++ ) {
//         if( HandleArray[i].Handle )
//             ObDereferenceObject( (PVOID)HandleArray[i].Handle );
//     }
//
//     ExFreePoolWithTag(HandleArray, TAG_AFD_POLL_HANDLE);
//     HandleArray = NULL;
// }
//
// BOOLEAN SocketAcquireStateLock( PAFD_FCB FCB ) {
//     if( !FCB ) return FALSE;
//
//     return !KeWaitForMutexObject(&FCB->Mutex,
//                                  Executive,
//                                  KernelMode,
//                                  FALSE,
//                                  NULL);
// }
//
// VOID SocketStateUnlock( PAFD_FCB FCB ) {
//     KeReleaseMutex(&FCB->Mutex, FALSE);
// }
//
// NTSTATUS NTAPI UnlockAndMaybeComplete
// ( PAFD_FCB FCB, NTSTATUS Status, PIRP Irp,
//   UINT Information ) {
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = Information;
//     if ( Irp->MdlAddress ) UnlockRequest( Irp, IoGetCurrentIrpStackLocation( Irp ) );
//     (void)IoSetCancelRoutine(Irp, NULL);
//     SocketStateUnlock( FCB );
//     IoCompleteRequest( Irp, IO_NETWORK_INCREMENT );
//     return Status;
// }
//
//
// NTSTATUS LostSocket( PIRP Irp ) {
//     NTSTATUS Status = STATUS_FILE_CLOSED;
//     AFD_DbgPrint!(MIN_TRACE,("Called.\n"));
//     Irp->IoStatus.Information = 0;
//     Irp->IoStatus.Status = Status;
//     if ( Irp->MdlAddress ) UnlockRequest( Irp, IoGetCurrentIrpStackLocation( Irp ) );
//     IoCompleteRequest( Irp, IO_NO_INCREMENT );
//     return Status;
// }
//
// NTSTATUS QueueUserModeIrp(PAFD_FCB FCB, PIRP Irp, UINT Function)
// {
//     NTSTATUS Status;
//
//     /* Add the IRP to the queue in all cases (so AfdCancelHandler will work properly) */
//     InsertTailList( &FCB->PendingIrpList[Function],
//                    &Irp->Tail.Overlay.ListEntry );
//
//     /* Acquire the cancel spin lock and check the cancel bit */
//     IoAcquireCancelSpinLock(&Irp->CancelIrql);
//     if (!Irp->Cancel)
//     {
//         /* We are not cancelled; we're good to go so
//          * set the cancel routine, release the cancel spin lock,
//          * mark the IRP as pending, and
//          * return STATUS_PENDING to the caller
//          */
//         (void)IoSetCancelRoutine(Irp, AfdCancelHandler);
//         IoReleaseCancelSpinLock(Irp->CancelIrql);
//         IoMarkIrpPending(Irp);
//         Status = STATUS_PENDING;
//     }
//     else
//     {
//         /* We were already cancelled before we were able to register our cancel routine
//          * so we are to call the cancel routine ourselves right here to cancel the IRP
//          * (which handles all the stuff we do above) and return STATUS_CANCELLED to the caller
//          */
//         AfdCancelHandler(IoGetCurrentIrpStackLocation(Irp)->DeviceObject,
//                          Irp);
//         Status = STATUS_CANCELLED;
//     }
//
//     return Status;
// }
//
// NTSTATUS LeaveIrpUntilLater( PAFD_FCB FCB, PIRP Irp, UINT Function ) {
//     NTSTATUS Status;
//
//     Status = QueueUserModeIrp(FCB, Irp, Function);
//
//     SocketStateUnlock( FCB );
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================