use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn ERR(fmt: *const u8, ...);
    fn TRACE(fmt: *const u8, ...);
    fn GetSocketStructure(Handle: SOCKET) -> PSOCKET_INFORMATION;
    fn MsafdReturnWithErrno(Status: NTSTATUS, lpErrno: LPINT, arg3: u32, arg4: *mut core::ffi::c_void) -> i32;
    fn MsafdWaitForAlert(SockEvent: HANDLE);
    fn SetSocketInformation( Socket: PSOCKET_INFORMATION, info_type: u32, data: *const core::ffi::c_void, arg4: *const core::ffi::c_void, arg5: *const core::ffi::c_void, arg6: *const core::ffi::c_void, arg7: *const core::ffi::c_void );
}
#[no_mangle]
pub unsafe extern "C" fn WSPEventSelect(
    Handle: SOCKET,
    hEventObject: WSAEVENT,
    lNetworkEvents: i32,
    lpErrno: LPINT,
) -> i32 {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut EventSelectInfo: AFD_EVENT_SELECT_INFO = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let Status: NTSTATUS;
    let BlockMode: BOOLEAN;
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    TRACE!(b"WSPEventSelect (%lx) %lx %lx\n\0".as_ptr(), Handle, hEventObject, lNetworkEvents);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK;
        }
        return SOCKET_ERROR;
    }

    let mut Status = NtCreateEvent(
        core::ptr::addr_of_mut!(SockEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    BlockMode = 1u8;
    SetSocketInformation(
        Socket,
        AFD_INFO_BLOCKING_MODE,
        core::ptr::addr_of!(BlockMode) as *const core::ffi::c_void,
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
    );
    (*(*Socket).SharedData).NonBlocking = 1u8;

    if !(*Socket).EventObject.is_null() {
        (*(*Socket).SharedData).hWnd = core::ptr::null_mut();
        (*(*Socket).SharedData).wMsg = 0;
        (*(*Socket).SharedData).AsyncEvents = 0;
        (*(*Socket).SharedData).SequenceNumber += 1;
    }

    EventSelectInfo.EventObject = hEventObject;
    EventSelectInfo.Events = 0;

    if (lNetworkEvents & FD_READ) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_RECEIVE;
    }

    if (lNetworkEvents & FD_WRITE) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_SEND;
    }

    if (lNetworkEvents & FD_OOB) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_OOB_RECEIVE;
    }

    if (lNetworkEvents & FD_ACCEPT) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_ACCEPT;
    }

    if (lNetworkEvents & FD_CONNECT) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL;
    }

    if (lNetworkEvents & FD_CLOSE) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE;
    }

    if (lNetworkEvents & FD_QOS) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_QOS;
    }

    if (lNetworkEvents & FD_GROUP_QOS) != 0 {
        EventSelectInfo.Events |= AFD_EVENT_GROUP_QOS;
    }

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        SockEvent,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IOSB),
        IOCTL_AFD_EVENT_SELECT,
        core::ptr::addr_of_mut!(EventSelectInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<AFD_EVENT_SELECT_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);

    if Status != STATUS_SUCCESS {
        ERR!(b"Got status 0x%08x.\n\0".as_ptr(), Status);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut());
    }

    (*Socket).EventObject = hEventObject;
    (*Socket).NetworkEvents = lNetworkEvents;

    TRACE!(b"Leaving\n\0".as_ptr());

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn WSPEnumNetworkEvents(
    Handle: SOCKET,
    hEventObject: WSAEVENT,
    lpNetworkEvents: LPWSANETWORKEVENTS,
    lpErrno: LPINT,
) -> INT {
    let mut EnumReq: AFD_ENUM_NETWORK_EVENTS_INFO = core::mem::zeroed();
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut SockEvent: HANDLE;

    TRACE!(b"Called (lpNetworkEvents %x)\n\0".as_ptr() as _, lpNetworkEvents);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK;
        }
        return SOCKET_ERROR;
    }
    if lpNetworkEvents.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT;
        }
        return SOCKET_ERROR;
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        ERR!(b"Could not make an event %x\n\0".as_ptr() as _, Status);
        return SOCKET_ERROR;
    }

    EnumReq.Event = hEventObject;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        SockEvent,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut IOSB,
        IOCTL_AFD_ENUM_NETWORK_EVENTS,
        &mut EnumReq as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<AFD_ENUM_NETWORK_EVENTS_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);

    if Status != STATUS_SUCCESS {
        ERR!(b"Status 0x%08x\n\0".as_ptr() as _, Status);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut());
    }

    (*lpNetworkEvents).lNetworkEvents = 0;

    if EnumReq.PollEvents & AFD_EVENT_RECEIVE != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_READ;
        (*lpNetworkEvents).iErrorCode[FD_READ_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_READ_BIT as usize]);
    }

    if EnumReq.PollEvents & AFD_EVENT_SEND != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_WRITE;
        (*lpNetworkEvents).iErrorCode[FD_WRITE_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_WRITE_BIT as usize]);
    }

    if EnumReq.PollEvents & AFD_EVENT_OOB_RECEIVE != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_OOB;
        (*lpNetworkEvents).iErrorCode[FD_OOB_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_OOB_BIT as usize]);
    }

    if EnumReq.PollEvents & AFD_EVENT_ACCEPT != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_ACCEPT;
        (*lpNetworkEvents).iErrorCode[FD_ACCEPT_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_ACCEPT_BIT as usize]);
    }

    if EnumReq.PollEvents & (AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL) != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_CONNECT;
        (*lpNetworkEvents).iErrorCode[FD_CONNECT_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_CONNECT_BIT as usize]);
    }

    if EnumReq.PollEvents & (AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE) != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_CLOSE;
        (*lpNetworkEvents).iErrorCode[FD_CLOSE_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_CLOSE_BIT as usize]);
    }

    if EnumReq.PollEvents & AFD_EVENT_QOS != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_QOS;
        (*lpNetworkEvents).iErrorCode[FD_QOS_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_QOS_BIT as usize]);
    }

    if EnumReq.PollEvents & AFD_EVENT_GROUP_QOS != 0 {
        (*lpNetworkEvents).lNetworkEvents |= FD_GROUP_QOS;
        (*lpNetworkEvents).iErrorCode[FD_GROUP_QOS_BIT as usize] = TranslateNtStatusError(EnumReq.EventStatus[FD_GROUP_QOS_BIT as usize]);
    }

    TRACE!(b"Leaving\n\0".as_ptr() as _);

    MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, core::ptr::null_mut())
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Ancillary Function Driver DLL
//  * FILE:        dll/win32/msafd/misc/event.c
//  * PURPOSE:     Event handling
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *              Alex Ionescu (alex@relsoft.net)
//  * REVISIONS:
//  *   CSH  15/06/2001 - Created
//  *   Alex 16/07/2004 - Complete Rewrite
//  */
//
// #include <msafd.h>
//
// int
// WSPAPI
// WSPEventSelect(
//     IN SOCKET Handle,
//     IN WSAEVENT hEventObject,
//     IN long lNetworkEvents,
//     OUT LPINT lpErrno)
// {
//     IO_STATUS_BLOCK        IOSB;
//     AFD_EVENT_SELECT_INFO  EventSelectInfo;
//     PSOCKET_INFORMATION    Socket = NULL;
//     NTSTATUS               Status;
//     BOOLEAN                BlockMode;
//     HANDLE                 SockEvent;
//
//     TRACE("WSPEventSelect (%lx) %lx %lx\n", Handle, hEventObject, lNetworkEvents);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent, EVENT_ALL_ACCESS,
//                            NULL, SynchronizationEvent, FALSE);
//
//     if (!NT_SUCCESS(Status)) return SOCKET_ERROR;
//
//     /* Set Socket to Non-Blocking */
//     BlockMode = TRUE;
//     SetSocketInformation(Socket, AFD_INFO_BLOCKING_MODE, &BlockMode, NULL, NULL, NULL, NULL);
//     Socket->SharedData->NonBlocking = TRUE;
//
//     /* Deactivate Async Select if there is one */
//     if (Socket->EventObject) {
//         Socket->SharedData->hWnd = NULL;
//         Socket->SharedData->wMsg = 0;
//         Socket->SharedData->AsyncEvents = 0;
//         Socket->SharedData->SequenceNumber++; // This will kill Async Select after the next completion
//     }
//
//     /* Set Structure Info */
//     EventSelectInfo.EventObject = hEventObject;
//     EventSelectInfo.Events = 0;
//
//     /* Set Events to wait for */
//     if (lNetworkEvents & FD_READ) {
//         EventSelectInfo.Events |= AFD_EVENT_RECEIVE;
//     }
//
//     if (lNetworkEvents & FD_WRITE) {
//         EventSelectInfo.Events |= AFD_EVENT_SEND;
//     }
//
//     if (lNetworkEvents & FD_OOB) {
//         EventSelectInfo.Events |= AFD_EVENT_OOB_RECEIVE;
//     }
//
//     if (lNetworkEvents & FD_ACCEPT) {
//         EventSelectInfo.Events |= AFD_EVENT_ACCEPT;
//     }
//
//     if (lNetworkEvents & FD_CONNECT) {
//         EventSelectInfo.Events |= AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL;
//     }
//
//     if (lNetworkEvents & FD_CLOSE) {
//         EventSelectInfo.Events |= AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE;
//     }
//
//     if (lNetworkEvents & FD_QOS) {
//         EventSelectInfo.Events |= AFD_EVENT_QOS;
//     }
//
//     if (lNetworkEvents & FD_GROUP_QOS) {
//         EventSelectInfo.Events |= AFD_EVENT_GROUP_QOS;
//     }
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_EVENT_SELECT,
//                                    &EventSelectInfo,
//                                    sizeof(EventSelectInfo),
//                                    NULL,
//                                    0);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING) {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose (SockEvent);
//
//     if (Status != STATUS_SUCCESS)
//     {
//         ERR("Got status 0x%08x.\n", Status);
//         return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
//     }
//
//     /* Set Socket Data*/
//     Socket->EventObject = hEventObject;
//     Socket->NetworkEvents = lNetworkEvents;
//
//     TRACE("Leaving\n");
//
//     return 0;
// }
//
//
// INT
// WSPAPI
// WSPEnumNetworkEvents(
//     IN  SOCKET Handle,
//     IN  WSAEVENT hEventObject,
//     OUT LPWSANETWORKEVENTS lpNetworkEvents,
//     OUT LPINT lpErrno)
// {
//     AFD_ENUM_NETWORK_EVENTS_INFO EnumReq;
//     IO_STATUS_BLOCK              IOSB;
//     PSOCKET_INFORMATION          Socket = NULL;
//     NTSTATUS                     Status;
//     HANDLE                       SockEvent;
//
//     TRACE("Called (lpNetworkEvents %x)\n", lpNetworkEvents);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//     if (!lpNetworkEvents)
//     {
//        if (lpErrno) *lpErrno = WSAEFAULT;
//        return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent, EVENT_ALL_ACCESS,
//                            NULL, SynchronizationEvent, FALSE);
//
//     if( !NT_SUCCESS(Status) ) {
//         ERR("Could not make an event %x\n", Status);
//         return SOCKET_ERROR;
//     }
//
//     EnumReq.Event = hEventObject;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_ENUM_NETWORK_EVENTS,
//                                    &EnumReq,
//                                    sizeof(EnumReq),
//                                    NULL,
//                                    0);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING) {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose (SockEvent);
//
//     if (Status != STATUS_SUCCESS)
//     {
//         ERR("Status 0x%08x\n", Status);
//         return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
//     }
//
//     lpNetworkEvents->lNetworkEvents = 0;
//
//     /* Set Events to wait for */
//     if (EnumReq.PollEvents & AFD_EVENT_RECEIVE) {
//         lpNetworkEvents->lNetworkEvents |= FD_READ;
//         lpNetworkEvents->iErrorCode[FD_READ_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_READ_BIT]);
//     }
//
//     if (EnumReq.PollEvents & AFD_EVENT_SEND) {
//         lpNetworkEvents->lNetworkEvents |= FD_WRITE;
//         lpNetworkEvents->iErrorCode[FD_WRITE_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_WRITE_BIT]);
//     }
//
//     if (EnumReq.PollEvents & AFD_EVENT_OOB_RECEIVE) {
//         lpNetworkEvents->lNetworkEvents |= FD_OOB;
//         lpNetworkEvents->iErrorCode[FD_OOB_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_OOB_BIT]);
//     }
//
//     if (EnumReq.PollEvents & AFD_EVENT_ACCEPT) {
//         lpNetworkEvents->lNetworkEvents |= FD_ACCEPT;
//         lpNetworkEvents->iErrorCode[FD_ACCEPT_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_ACCEPT_BIT]);
//     }
//
//     if (EnumReq.PollEvents &
//             (AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL)) {
//         lpNetworkEvents->lNetworkEvents |= FD_CONNECT;
//         lpNetworkEvents->iErrorCode[FD_CONNECT_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_CONNECT_BIT]);
//     }
//
//     if (EnumReq.PollEvents &
//             (AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE)) {
//         lpNetworkEvents->lNetworkEvents |= FD_CLOSE;
//         lpNetworkEvents->iErrorCode[FD_CLOSE_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_CLOSE_BIT]);
//     }
//
//     if (EnumReq.PollEvents & AFD_EVENT_QOS) {
//         lpNetworkEvents->lNetworkEvents |= FD_QOS;
//         lpNetworkEvents->iErrorCode[FD_QOS_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_QOS_BIT]);
//     }
//
//     if (EnumReq.PollEvents & AFD_EVENT_GROUP_QOS) {
//         lpNetworkEvents->lNetworkEvents |= FD_GROUP_QOS;
//         lpNetworkEvents->iErrorCode[FD_GROUP_QOS_BIT] = TranslateNtStatusError(EnumReq.EventStatus[FD_GROUP_QOS_BIT]);
//     }
//
//     TRACE("Leaving\n");
//
//     return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, NULL);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
