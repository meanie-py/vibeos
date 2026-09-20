use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    HANDLE, KPROCESSOR_MODE, LARGE_INTEGER, PETHREAD, PEPROCESS, PHANDLE,
    PLPCP_CONNECTION_MESSAGE, PLPCP_MESSAGE, PLPCP_PORT_OBJECT,
    PORT_MESSAGE, PORT_VIEW, PPORT_MESSAGE, PPORT_VIEW, PREMOTE_PORT_VIEW,
    REMOTE_PORT_VIEW, ACCESS_MASK,
    _LPCP_MESSAGE, _LPCP_CONNECTION_MESSAGE, _LPCP_PORT_OBJECT,
    LpcpGetMessageFromThread, PsGetCurrentThread,
    PsLookupProcessThreadByCid,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut LpcpLock: KGUARDED_MUTEX;
    static mut LpcPortObjectType: PVOID;

    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(address: PVOID, length: SIZE_T, alignment: ULONG);
    fn ProbeForWrite(address: PVOID, length: SIZE_T, alignment: ULONG);
    fn ObCreateObject(
        object_attributes_access_mode: KPROCESSOR_MODE,
        object_type: PVOID,
        object_attributes: PVOID,
        access_mode: KPROCESSOR_MODE,
        reserved: PVOID,
        object_body_size: ULONG,
        page_pool_charge: ULONG,
        non_paged_pool_charge: ULONG,
        object: *mut PVOID,
    ) -> NTSTATUS;
    fn ObInsertObject(
        object: PVOID,
        access_state: PVOID,
        desired_access: ACCESS_MASK,
        object_pointer_bias: ULONG,
        new_object: *mut PVOID,
        handle: PHANDLE,
    ) -> NTSTATUS;
    fn ObReferenceObjectByHandle(
        handle: HANDLE,
        access: ACCESS_MASK,
        object_type: PVOID,
        mode: KPROCESSOR_MODE,
        object: *mut PVOID,
        info: PVOID,
    ) -> NTSTATUS;
    fn ObCloseHandle(handle: HANDLE, mode: KPROCESSOR_MODE) -> NTSTATUS;
    fn MmMapViewOfSection(
        section: PVOID,
        process: PEPROCESS,
        base_address: *mut PVOID,
        zero_bits: ULONG_PTR,
        commit_size: SIZE_T,
        section_offset: *mut LARGE_INTEGER,
        view_size: *mut SIZE_T,
        inherit_disposition: ULONG,
        allocation_type: ULONG,
        page_protection: ULONG,
    ) -> NTSTATUS;
}
extern "C" {
    fn ProbeForWriteHandle(address: *mut HANDLE);
}
// NTAPI (stdcall on i386) — defs in shim_exports/common_inlines are extern "system"
extern "system" {
    fn LpcpCompleteWait(semaphore: PVOID);
}
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
}

const LPCP_PORT_TYPE_MASK: ULONG = 0x0000000F;
const LPCP_COMMUNICATION_PORT: ULONG = 0x00000003;

const PORT_ALL_ACCESS: ACCESS_MASK = 0x001F0001;
const LPC_REPLY: ULONG = 2;
const PAGE_READWRITE: ULONG = 0x04;

#[no_mangle]
pub unsafe extern "system" fn LpcpPrepareToWakeClient(thread: PETHREAD) {
    let thread = opaque_ptr(thread);
    if (*thread).__bindgen_anon_9.__bindgen_anon_1.LpcExitThreadCalled() == 0
        && !IsListEmpty(core::ptr::addr_of_mut!((*thread).__bindgen_anon_1.LpcReplyChain))
    {
        RemoveEntryList(core::ptr::addr_of_mut!((*thread).__bindgen_anon_1.LpcReplyChain));
        InitializeListHead(core::ptr::addr_of_mut!((*thread).__bindgen_anon_1.LpcReplyChain));
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtAcceptConnectPort(
    port_handle: PHANDLE,
    port_context: PVOID,
    reply_message: PPORT_MESSAGE,
    accept_connection: BOOLEAN,
    server_view: PPORT_VIEW,
    client_view: PREMOTE_PORT_VIEW,
) -> NTSTATUS {
    let port_handle = opaque_ptr(port_handle);
    let port_context = opaque_ptr(port_context);
    let previous_mode = KeGetPreviousMode();
    let mut captured_server_view: PORT_VIEW = core::mem::zeroed();
    let mut captured_reply_message: PORT_MESSAGE;
    let mut connection_info_length: ULONG;
    let mut connection_port: PLPCP_PORT_OBJECT;
    let mut server_port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut client_port: PLPCP_PORT_OBJECT;
    let mut connect_message: PLPCP_CONNECTION_MESSAGE;
    let mut message: PLPCP_MESSAGE;
    let mut client_section_to_map: PVOID = core::ptr::null_mut();
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut client_process: PEPROCESS = core::ptr::null_mut();
    let mut client_thread: PETHREAD = core::ptr::null_mut();
    let mut section_offset: LARGE_INTEGER = core::mem::zeroed();
    let mut status: NTSTATUS;

    if previous_mode != 0 {
        // VALIDATE-FIRST + FAULT-PROTECTED-CAPTURE. The C wraps this whole
        // block -- every probe and capture down through the client_view
        // length check below -- in ONE `_SEH2_TRY` whose handler is
        // `_SEH2_YIELD(return _SEH2_GetExceptionCode())` (complete.c:74-121).
        // The translation kept the ProbeForRead/ProbeForWrite calls and the
        // raw pointer derefs but dropped the handler, so under DUMMY_PSEH an
        // unprivileged caller passing any bad reply_message/server_view/
        // client_view bugchecked the kernel instead of getting
        // STATUS_ACCESS_VIOLATION back. A range check (what ProbeForRead/
        // ProbeForWrite do) proves where a buffer IS, never that it stays
        // mapped, so every capture below has to be fault-protected too --
        // probe_read_user does both the probe and a protected copy in one
        // call. The two STATUS_INVALID_PARAMETER returns for mismatched
        // view lengths stay ordinary early returns exactly as in the C
        // (they are `_SEH2_YIELD(return STATUS_INVALID_PARAMETER)` inside
        // the same TRY, not exceptions) -- they must not become propagated
        // probe errors.
        if let Err(s) = ntkrnl_sys::probe::probe_write_generic::<HANDLE>(
            port_handle as *mut HANDLE,
            // ProbeForWriteHandle: range-only, and it PRESERVES the caller's
            // value (`*p = *p`). probe_write_handle zeroes it -- that is
            // ProbeAndZeroHandle's contract, a different macro.
            1,
            previous_mode,
        ) {
            return s;
        }

        // C reads this as `*(volatile PORT_MESSAGE*)reply_message`; the
        // volatile was dropped in translation. Capturing the whole struct
        // in one fault-protected fetch restores the single-fetch semantics
        // volatile existed to guarantee -- a second, unprotected re-read
        // here could otherwise observe a value a hostile caller changed
        // between the probe and this read.
        captured_reply_message = match ntkrnl_sys::probe::probe_read_generic::<PORT_MESSAGE>(
            reply_message as *const PORT_MESSAGE,
            // complete.c:80 asks for sizeof(ULONG); align_of::<PORT_MESSAGE>() is 8.
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            Ok(v) => v,
            Err(s) => return s,
        };
        connection_info_length = captured_reply_message.u1.s1.DataLength as ULONG;
        // No capture here (this is buffer-range validation, not a read of a
        // typed value), so probe_user_buffer -- the probe half of
        // probe_read_user without the copy -- is the right replacement.
        if let Err(s) = ntkrnl_sys::probe::probe_user_buffer(
            (reply_message as usize + core::mem::size_of::<PORT_MESSAGE>()) as *const core::ffi::c_void,
            connection_info_length as usize,
            1,
            previous_mode,
        ) {
            return s;
        }

        if !server_view.is_null() {
            // Same dropped-volatile issue as reply_message above: capture
            // once via a fault-protected read instead of probe-then-raw-
            // deref, so a hostile caller can't change bytes between the
            // probe and the read.
            // The C probes this for WRITE (complete.c:92) before reading it,
            // and a read probe is not a substitute: only a write touch proves
            // the page is writable, which is what the later write-back of
            // ViewBase/ViewSize depends on. Touch first, then capture.
            if let Err(s) = ntkrnl_sys::probe::probe_write_generic::<PORT_VIEW>(
                server_view as *mut PORT_VIEW,
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                return s;
            }
            captured_server_view = match ntkrnl_sys::probe::probe_read_generic::<PORT_VIEW>(
                server_view as *const PORT_VIEW,
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                Ok(v) => v,
                Err(s) => return s,
            };
            if captured_server_view.Length != core::mem::size_of::<PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
        }

        if !client_view.is_null() {
            // Only .Length is consulted here; the rest of the
            // REMOTE_PORT_VIEW is captured later (complete.rs:355-359,
            // untouched by this pass). Still capture the whole struct in
            // one fault-protected read rather than probing then
            // dereferencing raw -- same dropped-volatile reasoning as
            // above, and it matches the C's ProbeForWrite(sizeof(REMOTE_PORT_VIEW)).
            // Probed for WRITE in the C (complete.c:106) -- same reasoning as
            // server_view above.
            if let Err(s) = ntkrnl_sys::probe::probe_write_generic::<REMOTE_PORT_VIEW>(
                client_view as *mut REMOTE_PORT_VIEW,
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                return s;
            }
            let captured_client_view: REMOTE_PORT_VIEW = match ntkrnl_sys::probe::probe_read_generic::<REMOTE_PORT_VIEW>(
                client_view as *const REMOTE_PORT_VIEW,
                core::mem::size_of::<ULONG>(),
                previous_mode,
            ) {
                Ok(v) => v,
                Err(s) => return s,
            };
            if captured_client_view.Length != core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
        }
    } else {
        captured_reply_message = *reply_message;
        connection_info_length = captured_reply_message.u1.s1.DataLength as ULONG;

        if !server_view.is_null() {
            if (*server_view).Length != core::mem::size_of::<PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            captured_server_view = *server_view;
        }

        if !client_view.is_null() {
            if (*client_view).Length != core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
        }
    }

    status = PsLookupProcessThreadByCid(
        core::ptr::addr_of_mut!(captured_reply_message.__bindgen_anon_1.ClientId),
        &mut client_process,
        &mut client_thread,
    );
    if status < 0 {
        return status;
    }

    KeAcquireGuardedMutex(&raw mut LpcpLock);

    if LpcpGetMessageFromThread(client_thread).is_null()
        || captured_reply_message.MessageId == 0
        || (*client_thread).LpcReplyMessageId != captured_reply_message.MessageId
    {
        KeReleaseGuardedMutex(&raw mut LpcpLock);
        ObDereferenceObject(client_process as PVOID);
        ObDereferenceObject(client_thread as PVOID);
        return STATUS_REPLY_MESSAGE_MISMATCH;
    }

    message = LpcpGetMessageFromThread(client_thread);
    connect_message = (message as usize + core::mem::size_of::<_LPCP_MESSAGE>()) as PLPCP_CONNECTION_MESSAGE;
    client_port = (*connect_message).ClientPort;
    connection_port = (*client_port).ConnectionPort;

    if (*connection_port).__bindgen_anon_1.ServerProcess != PsGetCurrentProcess() {
        KeReleaseGuardedMutex(&raw mut LpcpLock);
        ObDereferenceObject(client_process as PVOID);
        ObDereferenceObject(client_thread as PVOID);
        return STATUS_REPLY_MESSAGE_MISMATCH;
    }

    (*client_thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();
    (*client_thread).LpcReplyMessageId = 0;
    (*connect_message).ClientPort = core::ptr::null_mut();
    KeReleaseGuardedMutex(&raw mut LpcpLock);

    if connection_info_length > (*connection_port).MaxConnectionInfoLength as ULONG {
        connection_info_length = (*connection_port).MaxConnectionInfoLength as ULONG;
    }

    (*message).Request.u1.s1.DataLength = (connection_info_length as u16)
        .wrapping_add(core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>() as u16) as i16;
    (*message).Request.u1.s1.TotalLength = (core::mem::size_of::<_LPCP_MESSAGE>() as u16)
        .wrapping_add((*message).Request.u1.s1.DataLength as u16) as i16;
    (*message).Request.u2.s2.Type = LPC_REPLY as i16;
    (*message).Request.u2.s2.DataInfoOffset = 0;
    (*message).Request.__bindgen_anon_1.ClientId = captured_reply_message.__bindgen_anon_1.ClientId;
    (*message).Request.MessageId = captured_reply_message.MessageId;
    (*message).Request.__bindgen_anon_2.ClientViewSize = 0;

    // C wraps ONLY this RtlCopyMemory in `_SEH2_TRY` (complete.c:219-228);
    // its handler is `Status = _SEH2_GetExceptionCode();
    // _SEH2_YIELD(goto Cleanup);` -- unlike the probe failures above (which
    // are plain early returns, matching the C's OWN `_SEH2_YIELD(return
    // STATUS_INVALID_PARAMETER)`/`return status` shape before this point),
    // a fault HERE must still run the shared `Cleanup:` tail, because
    // client_thread/client_port/client_process (and, once set later,
    // client_section_to_map) are already live and owned by this function.
    // The source range was already probed for read at complete.rs:169-176
    // (before connection_info_length was possibly clamped down at :291-293,
    // so the probed range only ever shrinks), and the C has no ProbeForRead
    // of its own inside this TRY -- just the copy -- so this uses
    // safe_copy_raw: a bare fault-tolerant copy, no re-probe.
    if let Err(s) = ntkrnl_sys::probe::safe_copy_raw(
        (connect_message as usize + core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>()) as *mut core::ffi::c_void,
        (reply_message as usize + core::mem::size_of::<PORT_MESSAGE>()) as *const core::ffi::c_void,
        connection_info_length as usize,
    ) {
        status = s;
        // Cleanup: (complete.c's `Cleanup:` label). client_section_to_map is
        // still NULL here -- it isn't set until complete.rs:375-376, after
        // this site -- so that branch is a no-op, but it is kept for parity
        // with every other exit path in this function that reaches the same
        // label (see e.g. complete.rs:342-359's status<0 block, the closest
        // earlier example of this exact tail).
        if !client_section_to_map.is_null() {
            ObDereferenceObject(client_section_to_map);
        }
        if !client_thread.is_null() {
            KeAcquireGuardedMutex(&raw mut LpcpLock);
            (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
            LpcpPrepareToWakeClient(client_thread);
            KeReleaseGuardedMutex(&raw mut LpcpLock);
            LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
            ObDereferenceObject(client_thread as PVOID);
        }
        if !client_port.is_null() {
            ObDereferenceObject(client_port as PVOID);
        }
        ObDereferenceObject(client_process as PVOID);
        return status;
    }

    if accept_connection == 0 {
        // Connection refused - go to cleanup
        if !client_section_to_map.is_null() {
            ObDereferenceObject(client_section_to_map);
        }
        if !client_thread.is_null() {
            KeAcquireGuardedMutex(&raw mut LpcpLock);
            (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
            LpcpPrepareToWakeClient(client_thread);
            KeReleaseGuardedMutex(&raw mut LpcpLock);
            LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
            ObDereferenceObject(client_thread as PVOID);
        }
        if !client_port.is_null() {
            ObDereferenceObject(client_port as PVOID);
        }
        ObDereferenceObject(client_process as PVOID);
        return STATUS_SUCCESS;
    }

    status = ObCreateObject(
        previous_mode,
        LpcPortObjectType,
        core::ptr::null_mut(),
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<_LPCP_PORT_OBJECT>() as ULONG,
        0,
        0,
        &mut server_port as *mut PLPCP_PORT_OBJECT as *mut PVOID,
    );
    if status < 0 {
        if !client_section_to_map.is_null() {
            ObDereferenceObject(client_section_to_map);
        }
        if !client_thread.is_null() {
            KeAcquireGuardedMutex(&raw mut LpcpLock);
            (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
            LpcpPrepareToWakeClient(client_thread);
            KeReleaseGuardedMutex(&raw mut LpcpLock);
            LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
            ObDereferenceObject(client_thread as PVOID);
        }
        if !client_port.is_null() {
            ObDereferenceObject(client_port as PVOID);
        }
        ObDereferenceObject(client_process as PVOID);
        return status;
    }

    core::ptr::write_bytes(server_port as *mut u8, 0, core::mem::size_of::<_LPCP_PORT_OBJECT>());
    (*server_port).PortContext = port_context;
    (*server_port).Flags = LPCP_COMMUNICATION_PORT;
    (*server_port).MaxMessageLength = (*connection_port).MaxMessageLength;
    InitializeListHead(core::ptr::addr_of_mut!((*server_port).LpcReplyChainHead));
    InitializeListHead(core::ptr::addr_of_mut!((*server_port).LpcDataInfoChainHead));
    ObReferenceObject(connection_port as PVOID);
    (*server_port).ConnectionPort = connection_port;
    (*server_port).ConnectedPort = client_port;
    (*client_port).ConnectedPort = server_port;
    (*server_port).Creator = (*PsGetCurrentThread()).Cid;
    (*client_port).Creator = (*message).Request.__bindgen_anon_1.ClientId;

    KeAcquireGuardedMutex(&raw mut LpcpLock);
    client_section_to_map = (*connect_message).SectionToMap;
    (*connect_message).SectionToMap = core::ptr::null_mut();
    KeReleaseGuardedMutex(&raw mut LpcpLock);

    if !client_section_to_map.is_null() {
        section_offset.QuadPart = (*connect_message).ClientView.SectionOffset as i64;
        status = MmMapViewOfSection(
            client_section_to_map,
            PsGetCurrentProcess(),
            &mut (*server_port).ClientSectionBase,
            0,
            0,
            &mut section_offset,
            core::ptr::addr_of_mut!((*connect_message).ClientView.ViewSize) as *mut SIZE_T,
            2, // ViewUnmap
            0,
            PAGE_READWRITE,
        );
        (*connect_message).ClientView.SectionOffset = section_offset.__bindgen_anon_1.LowPart;
        if status >= 0 {
            (*connect_message).ClientView.ViewRemoteBase = (*server_port).ClientSectionBase;
            (*server_port).__bindgen_anon_1.MappingProcess = PsGetCurrentProcess();
            ObReferenceObject((*server_port).__bindgen_anon_1.MappingProcess as PVOID);
        } else {
            ObDereferenceObject(server_port as PVOID);
            if !client_section_to_map.is_null() {
                ObDereferenceObject(client_section_to_map);
            }
            if !client_thread.is_null() {
                KeAcquireGuardedMutex(&raw mut LpcpLock);
                (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
                LpcpPrepareToWakeClient(client_thread);
                KeReleaseGuardedMutex(&raw mut LpcpLock);
                LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
                ObDereferenceObject(client_thread as PVOID);
            }
            if !client_port.is_null() {
                ObDereferenceObject(client_port as PVOID);
            }
            ObDereferenceObject(client_process as PVOID);
            return status;
        }
    }

    ObReferenceObject(server_port as PVOID);
    status = ObInsertObject(
        server_port as PVOID,
        core::ptr::null_mut(),
        PORT_ALL_ACCESS,
        0,
        core::ptr::null_mut(),
        &mut handle,
    );
    if status < 0 {
        ObDereferenceObject(server_port as PVOID);
        if !client_section_to_map.is_null() {
            ObDereferenceObject(client_section_to_map);
        }
        if !client_thread.is_null() {
            KeAcquireGuardedMutex(&raw mut LpcpLock);
            (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
            LpcpPrepareToWakeClient(client_thread);
            KeReleaseGuardedMutex(&raw mut LpcpLock);
            LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
            ObDereferenceObject(client_thread as PVOID);
        }
        if !client_port.is_null() {
            ObDereferenceObject(client_port as PVOID);
        }
        ObDereferenceObject(client_process as PVOID);
        return status;
    }

    // C wraps ClientView->ViewBase/ViewSize + *PortHandle in ONE
    // `_SEH2_TRY` (complete.c:345-366). Unlike site 1 above, this handler
    // does explicit cleanup BEFORE falling into the shared `Cleanup:` tail:
    //     ObCloseHandle(Handle, PreviousMode);
    //     ObDereferenceObject(ServerPort);
    //     Status = _SEH2_GetExceptionCode();
    //     _SEH2_YIELD(goto Cleanup);
    // Handle was just inserted via ObInsertObject (complete.rs:420-427),
    // and ServerPort holds an EXTRA reference taken right before that
    // insert (complete.rs:419) specifically to keep the object alive until
    // it's fully wired up. A fault here must close that just-created
    // handle AND drop that extra reference, or both leak -- a bare
    // status-propagate (the shape every earlier error path in this
    // function uses) is wrong here precisely because those earlier paths
    // run before the handle/extra-ref exist.
    //
    // All three destinations were already probed for WRITE up front
    // (server_view/client_view at complete.rs:216-222; port_handle at
    // complete.rs:139-148), so -- same reasoning as site 1 -- this uses
    // safe_copy_raw (no re-probe), attempting the writes in the C's exact
    // order and stopping at the first fault so only one handler ever runs.
    let mut seh_status: Option<NTSTATUS> = None;
    if !client_view.is_null() {
        if let Err(s) = ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!((*client_view).ViewBase) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*connect_message).ClientView.ViewRemoteBase) as *const core::ffi::c_void,
            core::mem::size_of::<PVOID>(),
        ) {
            seh_status = Some(s);
        } else if let Err(s) = ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!((*client_view).ViewSize) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*connect_message).ClientView.ViewSize) as *const core::ffi::c_void,
            core::mem::size_of::<SIZE_T>(),
        ) {
            seh_status = Some(s);
        }
    }
    if seh_status.is_none() {
        if let Err(s) = ntkrnl_sys::probe::safe_copy_raw(
            port_handle as *mut core::ffi::c_void,
            core::ptr::addr_of!(handle) as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>(),
        ) {
            seh_status = Some(s);
        }
    }
    if let Some(s) = seh_status {
        ObCloseHandle(handle, previous_mode);
        ObDereferenceObject(server_port as PVOID);
        status = s;
        // Cleanup: tail, same as site 1 -- client_thread/client_port/
        // client_process are still live, and client_section_to_map may or
        // may not be set depending on whether the section-mapping branch
        // above ran; either way the null checks handle it.
        if !client_section_to_map.is_null() {
            ObDereferenceObject(client_section_to_map);
        }
        if !client_thread.is_null() {
            KeAcquireGuardedMutex(&raw mut LpcpLock);
            (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
            LpcpPrepareToWakeClient(client_thread);
            KeReleaseGuardedMutex(&raw mut LpcpLock);
            LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
            ObDereferenceObject(client_thread as PVOID);
        }
        if !client_port.is_null() {
            ObDereferenceObject(client_port as PVOID);
        }
        ObDereferenceObject(client_process as PVOID);
        return status;
    }

    if port_context.is_null() {
        (*server_port).PortContext = handle;
    }
    (*server_port).ClientThread = client_thread;

    KeAcquireGuardedMutex(&raw mut LpcpLock);
    (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
    KeReleaseGuardedMutex(&raw mut LpcpLock);
    client_thread = core::ptr::null_mut();
    ObDereferenceObject(server_port as PVOID);

    // Cleanup
    if !client_section_to_map.is_null() {
        ObDereferenceObject(client_section_to_map);
    }
    if !client_thread.is_null() {
        KeAcquireGuardedMutex(&raw mut LpcpLock);
        (*client_thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;
        LpcpPrepareToWakeClient(client_thread);
        KeReleaseGuardedMutex(&raw mut LpcpLock);
        LpcpCompleteWait(core::ptr::addr_of_mut!((*client_thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
        ObDereferenceObject(client_thread as PVOID);
    }
    if !client_port.is_null() {
        ObDereferenceObject(client_port as PVOID);
    }
    ObDereferenceObject(client_process as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtCompleteConnectPort(port_handle: HANDLE) -> NTSTATUS {
    let port_handle = opaque_ptr(port_handle);
    let previous_mode = KeGetPreviousMode();
    let mut port: PLPCP_PORT_OBJECT = core::ptr::null_mut();
    let mut thread: PETHREAD;

    let mut status = ObReferenceObjectByHandle(
        port_handle,
        PORT_ALL_ACCESS,
        LpcPortObjectType,
        previous_mode,
        &mut port as *mut PLPCP_PORT_OBJECT as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if ((*port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_COMMUNICATION_PORT {
        ObDereferenceObject(port as PVOID);
        return STATUS_INVALID_PORT_HANDLE;
    }

    KeAcquireGuardedMutex(&raw mut LpcpLock);

    if (*port).ClientThread.is_null() {
        KeReleaseGuardedMutex(&raw mut LpcpLock);
        ObDereferenceObject(port as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    thread = (*port).ClientThread;

    if LpcpGetMessageFromThread(thread).is_null() {
        KeReleaseGuardedMutex(&raw mut LpcpLock);
        ObDereferenceObject(port as PVOID);
        return STATUS_SUCCESS;
    }

    (*port).ClientThread = core::ptr::null_mut();
    LpcpPrepareToWakeClient(thread);
    KeReleaseGuardedMutex(&raw mut LpcpLock);
    LpcpCompleteWait(core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID);
    ObDereferenceObject(port as PVOID);
    ObDereferenceObject(thread as PVOID);
    status
}
