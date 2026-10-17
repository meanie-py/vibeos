use crate::types::*;
// The crate's own functions: the definition is the declaration.
use crate::lock::LockRequest;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{KeCancelTimer, KeInitializeDpc, KeInitializeTimerEx, KeSetTimer, _TIMER_TYPE_NotificationTimer, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PKEVENT, PRKDPC};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSetCancelRoutine};

extern "C" {
    fn LostSocket(Irp: PIRP) -> NTSTATUS;
    fn SocketAcquireStateLock(FCB: PAFD_FCB) -> u8;
}

// ---- file-scope preamble (translated) ----
// AFD_HANDLES / SET_AFD_HANDLES (afd.h macros) and FIELD_OFFSET: provided locally.
// (The C `static KDEFERRED_ROUTINE SelectTimeout;` is just a forward declaration;
//  the real definition is the NTAPI fn `SelectTimeout` below.)
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => { core::mem::offset_of!($t, $f) };
}
#[inline]
unsafe fn AFD_HANDLES(x: PAFD_POLL_INFO) -> PAFD_HANDLE {
    (*x).Exclusive as usize as PAFD_HANDLE
}
#[inline]
unsafe fn SET_AFD_HANDLES(x: PAFD_POLL_INFO, y: PAFD_HANDLE) {
    (*x).Exclusive = y as usize as _;
}

unsafe fn PrintEvents(Events: ULONG) {
    let events_list: [*const i8; 12] = [
        b"AFD_EVENT_RECEIVE\0".as_ptr() as *const i8,
        b"AFD_EVENT_OOB_RECEIVE\0".as_ptr() as *const i8,
        b"AFD_EVENT_SEND\0".as_ptr() as *const i8,
        b"AFD_EVENT_DISCONNECT\0".as_ptr() as *const i8,
        b"AFD_EVENT_ABORT\0".as_ptr() as *const i8,
        b"AFD_EVENT_CLOSE\0".as_ptr() as *const i8,
        b"AFD_EVENT_CONNECT\0".as_ptr() as *const i8,
        b"AFD_EVENT_ACCEPT\0".as_ptr() as *const i8,
        b"AFD_EVENT_CONNECT_FAIL\0".as_ptr() as *const i8,
        b"AFD_EVENT_QOS\0".as_ptr() as *const i8,
        b"AFD_EVENT_GROUP_QOS\0".as_ptr() as *const i8,
        core::ptr::null(),
    ];
    let mut i: i32 = 0;

    while !events_list[i as usize].is_null() {
        if (Events & (1 << i)) != 0 {
            AFD_DbgPrint!(MID_TRACE, b"(%s )\0".as_ptr() as *const i8, events_list[i as usize]);
        }
        i += 1;
    }
}

unsafe fn CopyBackStatus(
    HandleArray: PAFD_HANDLE,
    HandleCount: UINT,
) {
    let mut i: UINT = 0;

    while i < HandleCount {
        (*HandleArray.offset(i as isize)).Events = (*HandleArray.offset(i as isize)).Status as u32;
        (*HandleArray.offset(i as isize)).Status = 0;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ZeroEvents(
    HandleArray: PAFD_HANDLE,
    HandleCount: UINT,
) {
    let mut i: UINT = 0;
    while i < HandleCount {
        (*HandleArray.add(i as usize)).Status = 0;
        (*HandleArray.add(i as usize)).Events = 0;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SignalSocket(
    Poll: PAFD_ACTIVE_POLL,
    _Irp: PIRP,
    PollReq: PAFD_POLL_INFO,
    Status: NTSTATUS,
) {
    let mut i: UINT;
    let Irp: PIRP = if !_Irp.is_null() { _Irp } else { (*Poll).Irp };

    AFD_DbgPrint!(MID_TRACE, b"Called (Status %x)\n\0".as_ptr() as PCHAR, Status);

    if !Poll.is_null() {
        KeCancelTimer(core::ptr::addr_of_mut!((*Poll).Timer));
        RemoveEntryList(core::ptr::addr_of_mut!((*Poll).ListEntry));
        ExFreePoolWithTag(Poll as PVOID, TAG_AFD_ACTIVE_POLL);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information =
        (FIELD_OFFSET!(AFD_POLL_INFO, Handles) + core::mem::size_of::<AFD_HANDLE>() * (*PollReq).HandleCount as usize) as _;

    CopyBackStatus(
        (*PollReq).Handles.as_mut_ptr(),
        (*PollReq).HandleCount,
    );

    i = 0;
    while i < (*PollReq).HandleCount {
        AFD_DbgPrint!(
            MAX_TRACE,
            b"Handle(%x): Got %x,%x\n\0".as_ptr() as PCHAR,
            // AFD_HANDLES(), not `Handles[i]`: AFD_POLL_INFO ends in a flexible
            // `AFD_HANDLE Handles[1]` that the caller over-allocates to HandleCount, and
            // bindgen's `[T; 1]` makes every index >= 1 a panic. The helper already
            // existed in this file and several sites were simply left behind.
            (*AFD_HANDLES(PollReq).add(i as usize)).Handle,
            (*AFD_HANDLES(PollReq).add(i as usize)).Events,
            (*AFD_HANDLES(PollReq).add(i as usize)).Status,
        );
        i += 1;
    }

    UnlockHandles(AFD_HANDLES(PollReq), (*PollReq).HandleCount);

    if !(*Irp).MdlAddress.is_null() {
        UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
    }

    AFD_DbgPrint!(MID_TRACE, b"Completing\n\0".as_ptr() as PCHAR);
    IoSetCancelRoutine(Irp, None);
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as CCHAR);
    AFD_DbgPrint!(MID_TRACE, b"Done\n\0".as_ptr() as PCHAR);
}
unsafe extern "system" fn SelectTimeout(
    _Dpc: PKDPC,
    DeferredContext: PVOID,
    _SystemArgument1: PVOID,
    _SystemArgument2: PVOID,
) {
    let Poll: PAFD_ACTIVE_POLL = DeferredContext as PAFD_ACTIVE_POLL;
    let PollReq: PAFD_POLL_INFO;
    let Irp: PIRP;
    let mut OldIrql: KIRQL = 0;
    let DeviceExt: PAFD_DEVICE_EXTENSION;

    Irp = (*Poll).Irp;
    DeviceExt = (*Poll).DeviceExt;
    PollReq = (*Irp).AssociatedIrp.SystemBuffer as PAFD_POLL_INFO;

    ZeroEvents((*PollReq).Handles.as_mut_ptr(), (*PollReq).HandleCount);

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize , &mut OldIrql);
    SignalSocket(Poll, core::ptr::null_mut(), PollReq, STATUS_TIMEOUT);
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize , OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn KillSelectsForFCB(
    DeviceExt: PAFD_DEVICE_EXTENSION,
    FileObject: PFILE_OBJECT,
    OnlyExclusive: BOOLEAN,
) {
    let mut OldIrql: KIRQL = core::mem::zeroed();
    let mut ListEntry: PLIST_ENTRY;
    let mut Poll: PAFD_ACTIVE_POLL;
    let mut Irp: PIRP;
    let mut PollReq: PAFD_POLL_INFO;
    let mut HandleArray: PAFD_HANDLE;
    let mut i: UINT;

    AFD_DbgPrint!(MID_TRACE, b"Killing selects that refer to %p\n\0".as_ptr() as PCHAR, FileObject);

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize , core::ptr::addr_of_mut!(OldIrql));

    ListEntry = (*DeviceExt).Polls.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*DeviceExt).Polls) {
        Poll = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(AFD_ACTIVE_POLL, ListEntry) as isize)) as PAFD_ACTIVE_POLL;
        ListEntry = (*ListEntry).Flink;
        Irp = (*Poll).Irp;
        PollReq = (*Irp).AssociatedIrp.SystemBuffer as PAFD_POLL_INFO;
        HandleArray = AFD_HANDLES(PollReq);

        i = 0;
        while i < (*PollReq).HandleCount {
            AFD_DbgPrint!(MAX_TRACE, b"Req: %u, This %p\n\0".as_ptr() as PCHAR, (*HandleArray.offset(i as isize)).Handle, FileObject);
            if (*HandleArray.offset(i as isize)).Handle as PVOID == FileObject as PVOID &&
                (OnlyExclusive == 0 || (OnlyExclusive != 0 && (*Poll).Exclusive != 0)) {
                ZeroEvents((*PollReq).Handles.as_mut_ptr(), (*PollReq).HandleCount);
                SignalSocket(Poll, core::ptr::null_mut(), PollReq, STATUS_CANCELLED);
            }
            i += 1;
        }
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize , OldIrql);

    AFD_DbgPrint!(MID_TRACE, b"Done\n\0".as_ptr() as PCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn AfdSelect(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut FCB: PAFD_FCB;
    let mut FileObject: PFILE_OBJECT;
    let PollReq: PAFD_POLL_INFO = (*Irp).AssociatedIrp.SystemBuffer as PAFD_POLL_INFO;
    let DeviceExt: PAFD_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PAFD_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let mut Signalled: UINT = 0;
    let Exclusive: ULONG = (*PollReq).Exclusive as ULONG;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called (HandleCount %u Timeout %d)\n\0".as_ptr() as PCHAR,
        (*PollReq).HandleCount,
        (*PollReq).Timeout.QuadPart as INT,
    );

    SET_AFD_HANDLES(
        PollReq,
        LockHandles(
            (*PollReq).Handles.as_mut_ptr(),
            (*PollReq).HandleCount,
        ),
    );

    if AFD_HANDLES(PollReq).is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        (*Irp).IoStatus.Information = 0;
        IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    if Exclusive != 0 {
        for i in 0..(*PollReq).HandleCount as usize {
            if (*AFD_HANDLES(PollReq).add(i)).Handle == 0 {
                continue;
            }

            KillSelectsForFCB(
                DeviceExt,
                (*AFD_HANDLES(PollReq).add(i)).Handle as PFILE_OBJECT,
                1u8,
            );
        }
    }

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize,&mut OldIrql);

    for i in 0..(*PollReq).HandleCount as usize {
        if (*AFD_HANDLES(PollReq).add(i)).Handle == 0 {
            continue;
        }

        FileObject = (*AFD_HANDLES(PollReq).add(i)).Handle as PFILE_OBJECT;
        FCB = (*FileObject).FsContext as PAFD_FCB;

        AFD_DbgPrint!(MID_TRACE, b"AFD: Select Events: \0".as_ptr() as PCHAR);
        PrintEvents((*AFD_HANDLES(PollReq).add(i)).Events);
        AFD_DbgPrint!(MID_TRACE, b"\n\0".as_ptr() as PCHAR);

        // Events and Status live in the CALLER's array (PollReq->Handles, copied back
        // to user mode by CopyBackStatus); AFD_HANDLES() is AFD's locked copy and is
        // only for the FILE_OBJECT. Writing Status there, as this did, left every
        // handle's result 0, so select() reported nothing ready.
        let Out = (*PollReq).Handles.as_mut_ptr().add(i);
        (*Out).Status = ((*Out).Events & (*FCB).PollState) as NTSTATUS;
        if (*Out).Status != 0 {
            AFD_DbgPrint!(
                MID_TRACE,
                b"Signalling %p with %x\n\0".as_ptr() as PCHAR,
                FCB,
                (*FCB).PollState,
            );
            Signalled += 1;
        }
    }

    if Signalled != 0 {
        Status = STATUS_SUCCESS;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        SignalSocket(core::ptr::null_mut(), Irp, PollReq, Status);
    } else {
        let Poll: PAFD_ACTIVE_POLL = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<AFD_ACTIVE_POLL>(),
            TAG_AFD_ACTIVE_POLL,
        ) as PAFD_ACTIVE_POLL;

        if !Poll.is_null() {
            (*Poll).Irp = Irp;
            (*Poll).DeviceExt = DeviceExt;
            (*Poll).Exclusive = Exclusive as u8;

            KeInitializeTimerEx(&mut (*Poll).Timer, _TIMER_TYPE_NotificationTimer);

            KeInitializeDpc(
                &mut (*Poll).TimeoutDpc as *mut _ as PRKDPC,
                Some(core::mem::transmute(SelectTimeout as *const ())),
                Poll as PVOID,
            );

            InsertTailList(
                &mut (*DeviceExt).Polls,
                &mut (*Poll).ListEntry,
            );

            KeSetTimer(
                &mut (*Poll).Timer,
                (*PollReq).Timeout,
                &mut (*Poll).TimeoutDpc,
            );

            Status = STATUS_PENDING;
            IoMarkIrpPending(Irp);
            IoSetCancelRoutine(Irp, Some(AfdCancelHandler));
        } else {
            AFD_DbgPrint!(
                MAX_TRACE,
                b"FIXME: do something with the IRP!\n\0".as_ptr() as PCHAR,
            );
            Status = STATUS_NO_MEMORY;
        }
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize,OldIrql);

    AFD_DbgPrint!(
        MID_TRACE,
        b"Returning %x\n\0".as_ptr() as PCHAR,
        Status,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn AfdEventSelect(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let Status: NTSTATUS;
    let EventSelectInfo = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as PAFD_EVENT_SELECT_INFO;
    let FCB = (*FileObject).FsContext as PAFD_FCB;

    UNREFERENCED_PARAMETER!(DeviceObject as *mut core::ffi::c_void);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if EventSelectInfo.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called (Event %p Triggers %u)\n\0".as_ptr() as PCHAR,
        (*EventSelectInfo).EventObject,
        (*EventSelectInfo).Events,
    );

    if !(*FCB).EventSelect.is_null() {
        ObDereferenceObject((*FCB).EventSelect as PVOID);
    }
    (*FCB).EventSelect = core::ptr::null_mut();

    if !(*EventSelectInfo).EventObject.is_null() && (*EventSelectInfo).Events != 0 {
        Status = ObReferenceObjectByHandle(
            (*EventSelectInfo).EventObject as PVOID,
            EVENT_ALL_ACCESS,
            ExEventObjectType as *mut _,
            1i8,
            core::ptr::addr_of_mut!((*FCB).EventSelect) as *mut PVOID,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            AFD_DbgPrint!(
                MIN_TRACE,
                b"Failed reference event (0x%x)\n\0".as_ptr() as PCHAR,
                Status,
            );
            (*FCB).EventSelect = core::ptr::null_mut();
        } else {
            (*FCB).EventSelectTriggers = (*EventSelectInfo).Events;
        }
    } else {
        (*FCB).EventSelect = core::ptr::null_mut();
        (*FCB).EventSelectTriggers = 0;
        Status = STATUS_SUCCESS;
    }

    if !(*FCB).EventSelect.is_null()
        && ((*FCB).PollState & ((*FCB).EventSelectTriggers & !(*FCB).EventSelectDisabled)) != 0
    {
        AFD_DbgPrint!(
            MID_TRACE,
            b"Setting event %p\n\0".as_ptr() as PCHAR,
            (*FCB).EventSelect,
        );

        KeSetEvent((*FCB).EventSelect as PVOID, IO_NETWORK_INCREMENT as i32, 0u8);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Returning %x\n\0".as_ptr() as PCHAR,
        Status,
    );

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdEnumEvents(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let EnumReq = LockRequest(Irp, IrpSp, 1u8, core::ptr::null_mut()) as PAFD_ENUM_NETWORK_EVENTS_INFO;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut UserEvent: PKEVENT = core::ptr::null_mut();
    let Status: NTSTATUS;

    UNREFERENCED_PARAMETER!(DeviceObject as *mut core::ffi::c_void);

    AFD_DbgPrint!(MID_TRACE, b"Called (FCB %p)\n\0".as_ptr() as *const i8, FCB);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if EnumReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if !(*EnumReq).Event.is_null() {
        Status = ObReferenceObjectByHandle(
            (*EnumReq).Event,
            EVENT_ALL_ACCESS,
            ExEventObjectType as *mut _,
            1i8,
            &mut UserEvent as *mut PKEVENT as *mut *mut core::ffi::c_void,
            core::ptr::null_mut()
        );
        if !NT_SUCCESS(Status) {
            AFD_DbgPrint!(MIN_TRACE, b"Unable to reference event %x\n\0".as_ptr() as *const i8, Status);
            return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
        }

        KeClearEvent(UserEvent as PVOID);
        ObDereferenceObject(UserEvent as *mut core::ffi::c_void);
    }

    (*EnumReq).PollEvents = (*FCB).PollState & !(*FCB).EventSelectDisabled;
    core::ptr::copy_nonoverlapping(
        (*FCB).PollStatus.as_ptr() as *const u8,
        (*EnumReq).EventStatus.as_mut_ptr() as *mut u8,
        core::mem::size_of_val(&(*EnumReq).EventStatus)
    );

    (*FCB).EventSelectDisabled |= (*FCB).PollState & (*FCB).EventSelectTriggers;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

unsafe fn UpdatePollWithFCB(Poll: PAFD_ACTIVE_POLL, _FileObject: PFILE_OBJECT) -> BOOLEAN {
    let mut i: UINT;
    let mut FCB: PAFD_FCB;
    let mut Signalled: UINT = 0;
    // The poll request is the IRP's SYSTEM BUFFER, as in the C. This used to cast the
    // IRP itself, so HandleCount and the handle array were read out of the IRP header:
    // a socket that became ready while select() was already waiting never woke it
    // (a UDP reply AFD had queued sat there until the select timed out).
    let PollReq: PAFD_POLL_INFO = (*(*Poll).Irp).AssociatedIrp.SystemBuffer as PAFD_POLL_INFO;

    i = 0;
    while i < (*PollReq).HandleCount {
        if (*(AFD_HANDLES(PollReq).offset(i as isize))).Handle == 0 {
            i += 1;
            continue;
        }

        let FileObject = (*(AFD_HANDLES(PollReq).offset(i as isize))).Handle as PFILE_OBJECT;
        FCB = (*FileObject).FsContext as PAFD_FCB;

        // The caller's array, as in AfdSelect (see there).
        let Out = (*PollReq).Handles.as_mut_ptr().add(i as usize);
        (*Out).Status = ((*Out).Events & (*FCB).PollState) as NTSTATUS;

        if (*Out).Status != 0 {
            AFD_DbgPrint!(
                MID_TRACE,
                b"Signalling %p with %x\n\0".as_ptr() as PCHAR,
                FCB,
                (*FCB).PollState
            );
            Signalled += 1;
        }

        i += 1;
    }

    if Signalled != 0 { 1 } else { 0 }
}
#[no_mangle]
pub unsafe extern "C" fn PollReeval(DeviceExt: PAFD_DEVICE_EXTENSION, FileObject: PFILE_OBJECT) {
    let mut Poll: PAFD_ACTIVE_POLL;
    let mut ThePollEnt: PLIST_ENTRY;
    let FCB: PAFD_FCB;
    let mut OldIrql: KIRQL = 0;
    let mut PollReq: PAFD_POLL_INFO;

    AFD_DbgPrint!(MID_TRACE, b"Called: DeviceExt %p FileObject %p\n\0".as_ptr() as PCHAR, DeviceExt, FileObject);

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize,&mut OldIrql);

    FCB = (*FileObject).FsContext as PAFD_FCB;

    if FCB.is_null() {
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize,OldIrql);
        return;
    }

    ThePollEnt = (*DeviceExt).Polls.Flink;

    while ThePollEnt != &mut (*DeviceExt).Polls as PLIST_ENTRY {
        Poll = (ThePollEnt as *mut u8).offset(-(core::mem::offset_of!(AFD_ACTIVE_POLL, ListEntry) as isize)) as PAFD_ACTIVE_POLL;
        PollReq = (*(*Poll).Irp).AssociatedIrp.SystemBuffer as PAFD_POLL_INFO;
        AFD_DbgPrint!(MID_TRACE, b"Checking poll %p\n\0".as_ptr() as PCHAR, Poll);

        if UpdatePollWithFCB(Poll, FileObject) != 0 {
            ThePollEnt = (*ThePollEnt).Flink;
            AFD_DbgPrint!(MID_TRACE, b"Signalling socket\n\0".as_ptr() as PCHAR);
            SignalSocket(Poll, core::ptr::null_mut(), PollReq, STATUS_SUCCESS);
        } else {
            ThePollEnt = (*ThePollEnt).Flink;
        }
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize,OldIrql);

    if !(*FCB).EventSelect.is_null() &&
       ((*FCB).PollState & ((*FCB).EventSelectTriggers & !(*FCB).EventSelectDisabled)) != 0
    {
        AFD_DbgPrint!(MID_TRACE, b"Setting event %p\n\0".as_ptr() as PCHAR, (*FCB).EventSelect);

        KeSetEvent((*FCB).EventSelect as PVOID, IO_NETWORK_INCREMENT as i32, 0u8);
    }

    AFD_DbgPrint!(MID_TRACE, b"Leaving\n\0".as_ptr() as PCHAR);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/select.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// static VOID PrintEvents( ULONG Events ) {
//     char *events_list[] = { "AFD_EVENT_RECEIVE",
//                             "AFD_EVENT_OOB_RECEIVE",
//                             "AFD_EVENT_SEND",
//                             "AFD_EVENT_DISCONNECT",
//                             "AFD_EVENT_ABORT",
//                             "AFD_EVENT_CLOSE",
//                             "AFD_EVENT_CONNECT",
//                             "AFD_EVENT_ACCEPT",
//                             "AFD_EVENT_CONNECT_FAIL",
//                             "AFD_EVENT_QOS",
//                             "AFD_EVENT_GROUP_QOS",
//                             NULL };
//     int i;
//
//     for( i = 0; events_list[i]; i++ )
//         if( Events & (1 << i) ) AFD_DbgPrint!(MID_TRACE,("%s ", events_list[i] ));
// }
//
// static VOID CopyBackStatus( PAFD_HANDLE HandleArray,
//                             UINT HandleCount ) {
//     UINT i;
//
//     for( i = 0; i < HandleCount; i++ ) {
//         HandleArray[i].Events = HandleArray[i].Status;
//         HandleArray[i].Status = 0;
//     }
// }
//
// VOID ZeroEvents( PAFD_HANDLE HandleArray,
//                  UINT HandleCount ) {
//     UINT i;
//
//     for( i = 0; i < HandleCount; i++ ) {
//         HandleArray[i].Status = 0;
//         HandleArray[i].Events = 0;
//     }
// }
//
//
// /* you must pass either Poll OR Irp */
// VOID SignalSocket(
//    PAFD_ACTIVE_POLL Poll OPTIONAL,
//    PIRP _Irp OPTIONAL,
//    PAFD_POLL_INFO PollReq,
//    NTSTATUS Status
//    )
// {
//     UINT i;
//     PIRP Irp = _Irp ? _Irp : Poll->Irp;
//     AFD_DbgPrint!(MID_TRACE,("Called (Status %x)\n", Status));
//
//     if (Poll)
//     {
//         KeCancelTimer( &Poll->Timer );
//         RemoveEntryList( &Poll->ListEntry );
//         ExFreePoolWithTag(Poll, TAG_AFD_ACTIVE_POLL);
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information =
//         FIELD_OFFSET(AFD_POLL_INFO, Handles) + sizeof(AFD_HANDLE) * PollReq->HandleCount;
//     CopyBackStatus( PollReq->Handles,
//                     PollReq->HandleCount );
//     for( i = 0; i < PollReq->HandleCount; i++ ) {
//         AFD_DbgPrint
//             (MAX_TRACE,
//              ("Handle(%x): Got %x,%x\n",
//               PollReq->Handles[i].Handle,
//               PollReq->Handles[i].Events,
//               PollReq->Handles[i].Status));
//     }
//     UnlockHandles( AFD_HANDLES(PollReq), PollReq->HandleCount );
//     if( Irp->MdlAddress ) UnlockRequest( Irp, IoGetCurrentIrpStackLocation( Irp ) );
//     AFD_DbgPrint!(MID_TRACE,("Completing\n"));
//     (void)IoSetCancelRoutine(Irp, NULL);
//     IoCompleteRequest( Irp, IO_NETWORK_INCREMENT );
//     AFD_DbgPrint!(MID_TRACE,("Done\n"));
// }
//
// static KDEFERRED_ROUTINE SelectTimeout;
// static VOID NTAPI SelectTimeout( PKDPC Dpc,
//                            PVOID DeferredContext,
//                            PVOID SystemArgument1,
//                            PVOID SystemArgument2 ) {
//     PAFD_ACTIVE_POLL Poll = DeferredContext;
//     PAFD_POLL_INFO PollReq;
//     PIRP Irp;
//     KIRQL OldIrql;
//     PAFD_DEVICE_EXTENSION DeviceExt;
//
//     UNREFERENCED_PARAMETER(Dpc);
//     UNREFERENCED_PARAMETER(SystemArgument1);
//     UNREFERENCED_PARAMETER(SystemArgument2);
//
//     AFD_DbgPrint!(MID_TRACE,("Called\n"));
//
//     Irp = Poll->Irp;
//     DeviceExt = Poll->DeviceExt;
//     PollReq = Irp->AssociatedIrp.SystemBuffer;
//
//     ZeroEvents( PollReq->Handles, PollReq->HandleCount );
//
//     KeAcquireSpinLock( &DeviceExt->Lock, &OldIrql );
//     SignalSocket( Poll, NULL, PollReq, STATUS_TIMEOUT );
//     KeReleaseSpinLock( &DeviceExt->Lock, OldIrql );
//
//     AFD_DbgPrint!(MID_TRACE,("Timeout\n"));
// }
//
// VOID KillSelectsForFCB( PAFD_DEVICE_EXTENSION DeviceExt,
//                         PFILE_OBJECT FileObject,
//                         BOOLEAN OnlyExclusive ) {
//     KIRQL OldIrql;
//     PLIST_ENTRY ListEntry;
//     PAFD_ACTIVE_POLL Poll;
//     PIRP Irp;
//     PAFD_POLL_INFO PollReq;
//     PAFD_HANDLE HandleArray;
//     UINT i;
//
//     AFD_DbgPrint!(MID_TRACE,("Killing selects that refer to %p\n", FileObject));
//
//     KeAcquireSpinLock( &DeviceExt->Lock, &OldIrql );
//
//     ListEntry = DeviceExt->Polls.Flink;
//     while ( ListEntry != &DeviceExt->Polls ) {
//         Poll = CONTAINING_RECORD(ListEntry, AFD_ACTIVE_POLL, ListEntry);
//         ListEntry = ListEntry->Flink;
//         Irp = Poll->Irp;
//         PollReq = Irp->AssociatedIrp.SystemBuffer;
//         HandleArray = AFD_HANDLES(PollReq);
//
//         for( i = 0; i < PollReq->HandleCount; i++ ) {
//             AFD_DbgPrint!(MAX_TRACE,("Req: %u, This %p\n",
//                                     HandleArray[i].Handle, FileObject));
//             if( (PVOID)HandleArray[i].Handle == FileObject &&
//                 (!OnlyExclusive || (OnlyExclusive && Poll->Exclusive)) ) {
//                 ZeroEvents( PollReq->Handles, PollReq->HandleCount );
//                 SignalSocket( Poll, NULL, PollReq, STATUS_CANCELLED );
//             }
//         }
//     }
//
//     KeReleaseSpinLock( &DeviceExt->Lock, OldIrql );
//
//     AFD_DbgPrint!(MID_TRACE,("Done\n"));
// }
//
// NTSTATUS NTAPI
// AfdSelect( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//            PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_NO_MEMORY;
//     PAFD_FCB FCB;
//     PFILE_OBJECT FileObject;
//     PAFD_POLL_INFO PollReq = Irp->AssociatedIrp.SystemBuffer;
//     PAFD_DEVICE_EXTENSION DeviceExt = DeviceObject->DeviceExtension;
//     KIRQL OldIrql;
//     UINT i, Signalled = 0;
//     ULONG Exclusive = PollReq->Exclusive;
//
//     UNREFERENCED_PARAMETER(IrpSp);
//
//     AFD_DbgPrint!(MID_TRACE,("Called (HandleCount %u Timeout %d)\n",
//                             PollReq->HandleCount,
//                             (INT)(PollReq->Timeout.QuadPart)));
//
//     SET_AFD_HANDLES(PollReq,
//                     LockHandles( PollReq->Handles, PollReq->HandleCount ));
//
//     if( !AFD_HANDLES(PollReq) ) {
//         Irp->IoStatus.Status = STATUS_NO_MEMORY;
//         Irp->IoStatus.Information = 0;
//         IoCompleteRequest( Irp, IO_NETWORK_INCREMENT );
//         return STATUS_NO_MEMORY;
//     }
//
//     if( Exclusive ) {
//         for( i = 0; i < PollReq->HandleCount; i++ ) {
//             if( !AFD_HANDLES(PollReq)[i].Handle ) continue;
//
//             KillSelectsForFCB( DeviceExt,
//                                (PFILE_OBJECT)AFD_HANDLES(PollReq)[i].Handle,
//                                TRUE );
//         }
//     }
//
//     KeAcquireSpinLock( &DeviceExt->Lock, &OldIrql );
//
//     for( i = 0; i < PollReq->HandleCount; i++ ) {
//         if( !AFD_HANDLES(PollReq)[i].Handle ) continue;
//
//         FileObject = (PFILE_OBJECT)AFD_HANDLES(PollReq)[i].Handle;
//         FCB = FileObject->FsContext;
//
//         AFD_DbgPrint!(MID_TRACE, ("AFD: Select Events: "));
//         PrintEvents( PollReq->Handles[i].Events );
//         AFD_DbgPrint!(MID_TRACE,("\n"));
//
//         PollReq->Handles[i].Status =
//             PollReq->Handles[i].Events & FCB->PollState;
//         if( PollReq->Handles[i].Status ) {
//             AFD_DbgPrint!(MID_TRACE,("Signalling %p with %x\n",
//                                     FCB, FCB->PollState));
//             Signalled++;
//         }
//     }
//
//     if( Signalled ) {
//         Status = STATUS_SUCCESS;
//         Irp->IoStatus.Status = Status;
//         SignalSocket( NULL, Irp, PollReq, Status );
//     } else {
//
//        PAFD_ACTIVE_POLL Poll = NULL;
//
//        Poll = ExAllocatePoolWithTag(NonPagedPool,
//                                     sizeof(AFD_ACTIVE_POLL),
//                                     TAG_AFD_ACTIVE_POLL);
//
//        if (Poll){
//           Poll->Irp = Irp;
//           Poll->DeviceExt = DeviceExt;
//           Poll->Exclusive = Exclusive;
//
//           KeInitializeTimerEx( &Poll->Timer, NotificationTimer );
//
//           KeInitializeDpc( (PRKDPC)&Poll->TimeoutDpc, SelectTimeout, Poll );
//
//           InsertTailList( &DeviceExt->Polls, &Poll->ListEntry );
//
//           KeSetTimer( &Poll->Timer, PollReq->Timeout, &Poll->TimeoutDpc );
//
//           Status = STATUS_PENDING;
//           IoMarkIrpPending( Irp );
//           (void)IoSetCancelRoutine(Irp, AfdCancelHandler);
//        } else {
//           AFD_DbgPrint!(MAX_TRACE, ("FIXME: do something with the IRP!\n"));
//           Status = STATUS_NO_MEMORY;
//        }
//     }
//
//     KeReleaseSpinLock( &DeviceExt->Lock, OldIrql );
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return Status;
// }
//
// NTSTATUS NTAPI
// AfdEventSelect( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                 PIO_STACK_LOCATION IrpSp ) {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     NTSTATUS Status = STATUS_NO_MEMORY;
//     PAFD_EVENT_SELECT_INFO EventSelectInfo =
//         (PAFD_EVENT_SELECT_INFO)LockRequest( Irp, IrpSp, FALSE, NULL );
//     PAFD_FCB FCB = FileObject->FsContext;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) {
//         return LostSocket( Irp );
//     }
//
//     if ( !EventSelectInfo ) {
//          return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY, Irp,
//                                         0 );
//     }
//     AFD_DbgPrint!(MID_TRACE,("Called (Event %p Triggers %u)\n",
//                             EventSelectInfo->EventObject,
//                             EventSelectInfo->Events));
//
//     if( FCB->EventSelect ) ObDereferenceObject( FCB->EventSelect );
//     FCB->EventSelect = NULL;
//
//     if( EventSelectInfo->EventObject && EventSelectInfo->Events ) {
//         Status = ObReferenceObjectByHandle( (PVOID)EventSelectInfo->
//                                             EventObject,
//                                             EVENT_ALL_ACCESS,
//                                             *ExEventObjectType,
//                                             UserMode,
//                                             (PVOID *)&FCB->EventSelect,
//                                             NULL );
//
//         if( !NT_SUCCESS(Status) )
//         {
//             AFD_DbgPrint!(MIN_TRACE,("Failed reference event (0x%x)\n", Status));
//             FCB->EventSelect = NULL;
//         }
//         else
//             FCB->EventSelectTriggers = EventSelectInfo->Events;
//     } else {
//         FCB->EventSelect = NULL;
//         FCB->EventSelectTriggers = 0;
//         Status = STATUS_SUCCESS;
//     }
//
//     if((FCB->EventSelect) &&
//        (FCB->PollState & (FCB->EventSelectTriggers & ~FCB->EventSelectDisabled)))
//     {
//         AFD_DbgPrint!(MID_TRACE,("Setting event %p\n", FCB->EventSelect));
//
//         /* Set the application's event */
//         KeSetEvent( FCB->EventSelect, IO_NETWORK_INCREMENT, FALSE );
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp,
//                                    0 );
// }
//
// NTSTATUS NTAPI
// AfdEnumEvents( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                PIO_STACK_LOCATION IrpSp ) {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_ENUM_NETWORK_EVENTS_INFO EnumReq =
//         (PAFD_ENUM_NETWORK_EVENTS_INFO)LockRequest( Irp, IrpSp, TRUE, NULL );
//     PAFD_FCB FCB = FileObject->FsContext;
//     PKEVENT UserEvent;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called (FCB %p)\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) ) {
//         return LostSocket( Irp );
//     }
//
//     if ( !EnumReq ) {
//          return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY, Irp, 0 );
//     }
//
//     /* An event may optionally be provided for us to clear */
//     if (EnumReq->Event != NULL)
//     {
//         Status = ObReferenceObjectByHandle(EnumReq->Event,
//                                            EVENT_ALL_ACCESS,
//                                            *ExEventObjectType,
//                                            UserMode,
//                                            (PVOID *)&UserEvent,
//                                            NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             AFD_DbgPrint!(MIN_TRACE,("Unable to reference event %x\n", Status));
//             return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
//         }
//
//         /* Clear the event */
//         KeClearEvent(UserEvent);
//         ObDereferenceObject(UserEvent);
//     }
//
//     /* Copy the poll state, masking out disabled events */
//     EnumReq->PollEvents = (FCB->PollState & ~FCB->EventSelectDisabled);
//     RtlCopyMemory( EnumReq->EventStatus,
//                    FCB->PollStatus,
//                    sizeof(EnumReq->EventStatus) );
//
//     /* Disable the events that triggered the select until the reenabling function is called */
//     FCB->EventSelectDisabled |= (FCB->PollState & FCB->EventSelectTriggers);
//
//     return UnlockAndMaybeComplete( FCB, STATUS_SUCCESS, Irp, 0 );
// }
//
// /* * * NOTE ALWAYS CALLED AT DISPATCH_LEVEL * * */
// static BOOLEAN UpdatePollWithFCB( PAFD_ACTIVE_POLL Poll, PFILE_OBJECT FileObject ) {
//     UINT i;
//     PAFD_FCB FCB;
//     UINT Signalled = 0;
//     PAFD_POLL_INFO PollReq = Poll->Irp->AssociatedIrp.SystemBuffer;
//
//     ASSERT( KeGetCurrentIrql() == DISPATCH_LEVEL );
//
//     for( i = 0; i < PollReq->HandleCount; i++ ) {
//         if( !AFD_HANDLES(PollReq)[i].Handle ) continue;
//
//         FileObject = (PFILE_OBJECT)AFD_HANDLES(PollReq)[i].Handle;
//         FCB = FileObject->FsContext;
//
//         PollReq->Handles[i].Status = PollReq->Handles[i].Events & FCB->PollState;
//         if( PollReq->Handles[i].Status ) {
//             AFD_DbgPrint!(MID_TRACE,("Signalling %p with %x\n",
//                                     FCB, FCB->PollState));
//             Signalled++;
//         }
//     }
//
//     return Signalled ? 1 : 0;
// }
//
// VOID PollReeval( PAFD_DEVICE_EXTENSION DeviceExt, PFILE_OBJECT FileObject ) {
//     PAFD_ACTIVE_POLL Poll = NULL;
//     PLIST_ENTRY ThePollEnt = NULL;
//     PAFD_FCB FCB;
//     KIRQL OldIrql;
//     PAFD_POLL_INFO PollReq;
//
//     AFD_DbgPrint!(MID_TRACE,("Called: DeviceExt %p FileObject %p\n",
//                             DeviceExt, FileObject));
//
//     KeAcquireSpinLock( &DeviceExt->Lock, &OldIrql );
//
//     /* Take care of any event select signalling */
//     FCB = (PAFD_FCB)FileObject->FsContext;
//
//     if( !FCB ) {
//         KeReleaseSpinLock( &DeviceExt->Lock, OldIrql );
//         return;
//     }
//
//     /* Now signal normal select irps */
//     ThePollEnt = DeviceExt->Polls.Flink;
//
//     while( ThePollEnt != &DeviceExt->Polls ) {
//         Poll = CONTAINING_RECORD( ThePollEnt, AFD_ACTIVE_POLL, ListEntry );
//         PollReq = Poll->Irp->AssociatedIrp.SystemBuffer;
//         AFD_DbgPrint!(MID_TRACE,("Checking poll %p\n", Poll));
//
//         if( UpdatePollWithFCB( Poll, FileObject ) ) {
//             ThePollEnt = ThePollEnt->Flink;
//             AFD_DbgPrint!(MID_TRACE,("Signalling socket\n"));
//             SignalSocket( Poll, NULL, PollReq, STATUS_SUCCESS );
//         } else
//             ThePollEnt = ThePollEnt->Flink;
//     }
//
//     KeReleaseSpinLock( &DeviceExt->Lock, OldIrql );
//
//     if((FCB->EventSelect) &&
//        (FCB->PollState & (FCB->EventSelectTriggers & ~FCB->EventSelectDisabled)))
//     {
//         AFD_DbgPrint!(MID_TRACE,("Setting event %p\n", FCB->EventSelect));
//
//         /* Set the application's event */
//         KeSetEvent( FCB->EventSelect, IO_NETWORK_INCREMENT, FALSE );
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Leaving\n"));
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================