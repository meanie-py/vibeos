use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    HANDLE, KPROCESSOR_MODE, LARGE_INTEGER, PETHREAD, PEPROCESS, PHANDLE,
    PLPCP_CONNECTION_MESSAGE, PLPCP_MESSAGE, PLPCP_PORT_OBJECT,
    PORT_VIEW, PPORT_VIEW, PREMOTE_PORT_VIEW, REMOTE_PORT_VIEW,
    PSECURITY_QUALITY_OF_SERVICE, PSID, PTOKEN, PTOKEN_USER, PUNICODE_STRING,
    SECURITY_QUALITY_OF_SERVICE, ACCESS_MASK,
    _LPCP_MESSAGE, _LPCP_CONNECTION_MESSAGE, _LPCP_PORT_OBJECT,
    PsGetCurrentThread,
    LpcpGetMessageFromThread,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn LpcpFreeToPortZone(Message: PLPCP_MESSAGE, Flags: ULONG);
    fn ObCloseHandle(Handle: HANDLE, AccessMode: KPROCESSOR_MODE) -> NTSTATUS;
    static mut LpcpLock: KGUARDED_MUTEX;
    static mut LpcpNextMessageId: ULONG;
    static mut LpcPortObjectType: PVOID;
    static mut MmSectionObjectType: PVOID;
    fn ObCreateObject(
        ProbeMode: KPROCESSOR_MODE,
        Type: PVOID,
        ObjectAttributes: PVOID,
        AccessMode: KPROCESSOR_MODE,
        Reserved: PVOID,
        ObjectSize: ULONG,
        PagedPoolCharge: ULONG,
        NonPagedPoolCharge: ULONG,
        Object: *mut PVOID,
    ) -> NTSTATUS;
    fn ObInsertObject(
        Object: PVOID,
        PassedAccessState: PVOID,
        DesiredAccess: ACCESS_MASK,
        AdditionalReferences: ULONG,
        ReferencedObject: *mut PVOID,
        Handle: PHANDLE,
    ) -> NTSTATUS;
    fn ObReferenceObjectByHandle(
        Handle: HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectType: PVOID,
        AccessMode: KPROCESSOR_MODE,
        Object: *mut PVOID,
        HandleInformation: PVOID,
    ) -> NTSTATUS;
    fn ObReferenceObjectByName(
        ObjectName: PUNICODE_STRING,
        Attributes: ULONG,
        PassedAccessState: PVOID,
        DesiredAccess: ACCESS_MASK,
        ObjectType: PVOID,
        AccessMode: KPROCESSOR_MODE,
        ParseContext: PVOID,
        Object: *mut PVOID,
    ) -> NTSTATUS;
    fn PsReferencePrimaryToken(Process: PEPROCESS) -> PTOKEN;
    fn PsDereferencePrimaryToken(Token: PTOKEN);
    fn SeCreateClientSecurity(
        Thread: PETHREAD,
        Qos: PSECURITY_QUALITY_OF_SERVICE,
        RemoteSession: BOOLEAN,
        ClientContext: PVOID,
    ) -> NTSTATUS;
    fn SeQueryInformationToken(
        Token: PTOKEN,
        TokenInformationClass: ULONG,
        TokenInformation: *mut PVOID,
    ) -> NTSTATUS;
    fn RtlEqualSid(Sid1: PSID, Sid2: PSID) -> BOOLEAN;
    fn SepCaptureSid(
        Sid: PSID,
        AccessMode: KPROCESSOR_MODE,
        PoolType: POOL_TYPE,
        CaptureIfKernel: BOOLEAN,
        CapturedSid: *mut PSID,
    ) -> NTSTATUS;
    fn SepReleaseSid(Sid: PSID, AccessMode: KPROCESSOR_MODE, CaptureIfKernel: BOOLEAN);
    fn ProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
    fn ProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG);
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
    fn MmMapViewOfSection(
        Section: PVOID,
        Process: PEPROCESS,
        BaseAddress: *mut PVOID,
        ZeroBits: ULONG_PTR,
        CommitSize: SIZE_T,
        SectionOffset: *mut LARGE_INTEGER,
        ViewSize: *mut SIZE_T,
        InheritDisposition: ULONG,
        AllocationType: ULONG,
        Win32Protect: ULONG,
    ) -> NTSTATUS;
    fn LpcpInitializePortQueue(Port: PLPCP_PORT_OBJECT) -> NTSTATUS;
    fn KeSetEvent(Event: PVOID, Increment: LONG, Wait: BOOLEAN) -> LONG;
    fn KeReadStateSemaphore(Semaphore: PVOID) -> LONG;
    fn KeWaitForSingleObject(
        Object: PVOID,
        WaitReason: ULONG,
        WaitMode: KPROCESSOR_MODE,
        Alertable: BOOLEAN,
        Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS;
}
extern "C" {
    fn ObReferenceObject(Object: PVOID);
}
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
    fn ProbeForWriteUlong(Address: *mut ULONG);
    fn LpcpAllocateFromPortZone() -> PLPCP_MESSAGE;
    fn InsertTailList(ListHead: PLIST_ENTRY, Entry: PLIST_ENTRY);
    fn LpcpCompleteWait(Semaphore: PVOID);
    fn LpcpConnectWait(Semaphore: PVOID, PreviousMode: KPROCESSOR_MODE);
}
unsafe extern "C" {
    fn ProbeForWriteHandle(Address: PHANDLE);
}
extern "system" {
    fn KeEnterCriticalRegion();
    fn KeLeaveCriticalRegion();
}

const LPCP_PORT_TYPE_MASK: ULONG = 0x0000000F;
const LPCP_CONNECTION_PORT: ULONG = 0x00000001;
const LPCP_CLIENT_PORT: ULONG = 0x00000004;
const LPCP_NAME_DELETED: ULONG = 0x40000000;
const LPCP_SECURITY_DYNAMIC: ULONG = 0x80000000;
const LPCP_WAITABLE_PORT: ULONG = 0x20000000;

const PORT_CONNECT: ACCESS_MASK = 0x0001;
const PORT_ALL_ACCESS: ACCESS_MASK = 0x001F0001;

const SECTION_MAP_READ: ACCESS_MASK = 0x0004;
const SECTION_MAP_WRITE: ACCESS_MASK = 0x0002;

const ViewUnmap: ULONG = 2;
const PAGE_READWRITE: ULONG = 0x04;

const WrExecutive: ULONG = 0x00000007;
const KernelMode: KPROCESSOR_MODE = 0i8;

const SECURITY_DYNAMIC_TRACKING: u8 = 1;

const TokenUser: ULONG = 1;

/// Check if port has LPCP_NAME_DELETED flag set.
/// Extracted to separate function to work around LLVM i386 codegen bug
/// that misoptimizes this check when inlined into NtSecureConnectPort.
#[inline(never)]
unsafe fn lpcp_check_name_deleted(port: PLPCP_PORT_OBJECT) -> bool {
    let flags = core::ptr::read_volatile(&(*port).Flags as *const ULONG);
    (flags & 0x40000000u32) != 0
}

const LPC_CONNECTION_REQUEST: i16 = 10;

const TAG_SE: ULONG = 0x20206553;

#[no_mangle]
pub unsafe extern "system" fn LpcpFreeConMsg(
    message: *mut PLPCP_MESSAGE,
    connect_message: *mut PLPCP_CONNECTION_MESSAGE,
    current_thread: PETHREAD,
) -> PVOID {
    let message = opaque_ptr(message);
    let connect_message = opaque_ptr(connect_message);
    let current_thread = opaque_ptr(current_thread);
    let mut section_to_map: PVOID;
    let reply_message: PLPCP_MESSAGE;

    KeAcquireGuardedMutex(&raw mut LpcpLock);

    if !IsListEmpty(core::ptr::addr_of_mut!((*current_thread).__bindgen_anon_1.LpcReplyChain)) {
        RemoveEntryList(core::ptr::addr_of_mut!((*current_thread).__bindgen_anon_1.LpcReplyChain));
        InitializeListHead(core::ptr::addr_of_mut!((*current_thread).__bindgen_anon_1.LpcReplyChain));
    }

    reply_message = LpcpGetMessageFromThread(current_thread);
    if !reply_message.is_null() {
        *message = reply_message;

        if !IsListEmpty(core::ptr::addr_of_mut!((*reply_message).__bindgen_anon_1.Entry)) {
            RemoveEntryList(core::ptr::addr_of_mut!((*reply_message).__bindgen_anon_1.Entry));
            InitializeListHead(core::ptr::addr_of_mut!((*reply_message).__bindgen_anon_1.Entry));
        }

        (*current_thread).__bindgen_anon_6.LpcReceivedMessageId = 0;
        (*current_thread).__bindgen_anon_5.LpcReplyMessage = core::ptr::null_mut();

        *connect_message = (reply_message as usize + core::mem::size_of::<_LPCP_MESSAGE>()) as PLPCP_CONNECTION_MESSAGE;
        section_to_map = (**connect_message).SectionToMap;
        (**connect_message).SectionToMap = core::ptr::null_mut();
    } else {
        *message = core::ptr::null_mut();
        section_to_map = core::ptr::null_mut();
    }

    KeReleaseGuardedMutex(&raw mut LpcpLock);
    section_to_map
}

#[no_mangle]
pub unsafe extern "system" fn NtSecureConnectPort(
    port_handle: PHANDLE,
    port_name: PUNICODE_STRING,
    security_qos: PSECURITY_QUALITY_OF_SERVICE,
    client_view: PPORT_VIEW,
    server_sid: PSID,
    server_view: PREMOTE_PORT_VIEW,
    max_message_length: *mut ULONG,
    connection_information: PVOID,
    connection_information_length: *mut ULONG,
) -> NTSTATUS {
    let port_handle = opaque_ptr(port_handle);
    let port_name = opaque_ptr(port_name);
    let server_sid = opaque_ptr(server_sid);
    let max_message_length = opaque_ptr(max_message_length);
    let connection_information = opaque_ptr(connection_information);
    let connection_information_length = opaque_ptr(connection_information_length);
    let mut status = STATUS_SUCCESS;
    let previous_mode = KeGetPreviousMode();
    let thread = PsGetCurrentThread();
    let mut captured_qos: SECURITY_QUALITY_OF_SERVICE;
    let mut captured_client_view: PORT_VIEW = core::mem::zeroed();
    let mut captured_server_sid: PSID;
    let mut connection_info_length: ULONG = 0;
    let port: PLPCP_PORT_OBJECT;
    let client_port: PLPCP_PORT_OBJECT;
    let mut message: PLPCP_MESSAGE;
    let mut connect_message: PLPCP_CONNECTION_MESSAGE;
    let port_message_length: ULONG;
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut section_to_map: PVOID = core::ptr::null_mut();
    let mut section_offset: LARGE_INTEGER = core::mem::zeroed();
    let token: PTOKEN;
    let token_user_info: PTOKEN_USER;

    if previous_mode != KernelMode {
        ProbeForWriteHandle(port_handle);
        ProbeForRead(
            security_qos as PVOID,
            core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>(),
            core::mem::size_of::<ULONG>() as ULONG,
        );
        captured_qos = *(security_qos as *const SECURITY_QUALITY_OF_SERVICE);

        if !client_view.is_null() {
            ProbeForWrite(
                client_view as PVOID,
                core::mem::size_of::<PORT_VIEW>(),
                core::mem::size_of::<ULONG>() as ULONG,
            );
            captured_client_view = *(client_view as *const PORT_VIEW);

            if captured_client_view.Length != core::mem::size_of::<PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
        }

        if !server_view.is_null() {
            ProbeForWrite(
                server_view as PVOID,
                core::mem::size_of::<REMOTE_PORT_VIEW>(),
                core::mem::size_of::<ULONG>() as ULONG,
            );

            if (*(server_view as *const REMOTE_PORT_VIEW)).Length
                != core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG
            {
                return STATUS_INVALID_PARAMETER;
            }
        }

        if !max_message_length.is_null() {
            ProbeForWriteUlong(max_message_length);
        }

        if !connection_information_length.is_null() {
            ProbeForWriteUlong(connection_information_length);
            connection_info_length = *(connection_information_length as *const ULONG);
        }

        if !connection_information.is_null() {
            ProbeForWrite(
                connection_information,
                connection_info_length as SIZE_T,
                core::mem::size_of::<ULONG>() as ULONG,
            );
        }

        captured_server_sid = server_sid;
        if !server_sid.is_null() {
            status = SepCaptureSid(
                server_sid,
                previous_mode,
                PagedPool,
                1u8,
                &mut captured_server_sid,
            );
            if status < 0 {
                return status;
            }
        }
    } else {
        captured_qos = *security_qos;

        if !client_view.is_null() {
            if (*client_view).Length != core::mem::size_of::<PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
            captured_client_view = *client_view;
        }

        if !server_view.is_null() {
            if (*server_view).Length != core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG {
                return STATUS_INVALID_PARAMETER;
            }
        }

        if !connection_information_length.is_null() {
            connection_info_length = *connection_information_length;
        }

        captured_server_sid = server_sid;
    }

    let mut port_obj: PVOID = core::ptr::null_mut();
    status = ObReferenceObjectByName(
        port_name,
        0,
        core::ptr::null_mut(),
        PORT_CONNECT,
        LpcPortObjectType,
        previous_mode,
        core::ptr::null_mut(),
        &mut port_obj,
    );

    if status < 0 {
        if captured_server_sid != server_sid {
            SepReleaseSid(captured_server_sid, previous_mode, 1u8);
        }
        return status;
    }

    let port = port_obj as PLPCP_PORT_OBJECT;

    if ((*port).Flags & LPCP_PORT_TYPE_MASK) != LPCP_CONNECTION_PORT {
        ObDereferenceObject(port as PVOID);
        if captured_server_sid != server_sid {
            SepReleaseSid(captured_server_sid, previous_mode, 1u8);
        }
        return STATUS_INVALID_PORT_HANDLE;
    }

    if !server_sid.is_null() {
        if !(*port).__bindgen_anon_1.ServerProcess.is_null() {
            token = PsReferencePrimaryToken((*port).__bindgen_anon_1.ServerProcess);

            let mut token_user_ptr: PVOID = core::ptr::null_mut();
            status = SeQueryInformationToken(token, TokenUser, &mut token_user_ptr);
            token_user_info = token_user_ptr as PTOKEN_USER;

            PsDereferencePrimaryToken(token);

            if status >= 0 {
                if RtlEqualSid(captured_server_sid, (*token_user_info).User.Sid) == 0u8 {
                    status = STATUS_SERVER_SID_MISMATCH;
                }
                ExFreePoolWithTag(token_user_info as PVOID, TAG_SE);
            }
        } else {
            status = STATUS_SERVER_SID_MISMATCH;
        }

        if captured_server_sid != server_sid {
            SepReleaseSid(captured_server_sid, previous_mode, 1u8);
        }
    }

    if !server_sid.is_null() && status < 0 {
        ObDereferenceObject(port as PVOID);
        return status;
    }

    let mut client_port_obj: PVOID = core::ptr::null_mut();
    status = ObCreateObject(
        previous_mode,
        LpcPortObjectType,
        core::ptr::null_mut(),
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<_LPCP_PORT_OBJECT>() as ULONG,
        0,
        0,
        &mut client_port_obj,
    );

    if status < 0 {
        ObDereferenceObject(port as PVOID);
        return status;
    }

    let client_port = client_port_obj as PLPCP_PORT_OBJECT;

    core::ptr::write_bytes(client_port as *mut u8, 0, core::mem::size_of::<_LPCP_PORT_OBJECT>());
    (*client_port).Flags = LPCP_CLIENT_PORT;
    (*client_port).ConnectionPort = port;
    (*client_port).MaxMessageLength = (*port).MaxMessageLength;
    (*client_port).SecurityQos = captured_qos;
    InitializeListHead(core::ptr::addr_of_mut!((*client_port).LpcReplyChainHead));
    InitializeListHead(core::ptr::addr_of_mut!((*client_port).LpcDataInfoChainHead));

    if captured_qos.ContextTrackingMode == SECURITY_DYNAMIC_TRACKING {
        (*client_port).Flags |= LPCP_SECURITY_DYNAMIC;
    } else {
        status = SeCreateClientSecurity(
            thread,
            &captured_qos as *const SECURITY_QUALITY_OF_SERVICE as PSECURITY_QUALITY_OF_SERVICE,
            0u8,
            core::ptr::addr_of_mut!((*client_port).StaticSecurity) as PVOID,
        );
        if status < 0 {
            ObDereferenceObject(client_port as PVOID);
            return status;
        }
    }

    status = LpcpInitializePortQueue(client_port);
    if status < 0 {
        ObDereferenceObject(client_port as PVOID);
        return status;
    }

    if !client_view.is_null() {
        status = ObReferenceObjectByHandle(
            captured_client_view.SectionHandle,
            SECTION_MAP_READ | SECTION_MAP_WRITE,
            MmSectionObjectType,
            previous_mode,
            &mut section_to_map,
            core::ptr::null_mut(),
        );
        if status < 0 {
            ObDereferenceObject(client_port as PVOID);
            return status;
        }

        section_offset.QuadPart = captured_client_view.SectionOffset as i64;

        status = MmMapViewOfSection(
            section_to_map,
            PsGetCurrentProcess(),
            core::ptr::addr_of_mut!((*client_port).ClientSectionBase),
            0,
            0,
            &mut section_offset,
            core::ptr::addr_of_mut!(captured_client_view.ViewSize) as *mut SIZE_T,
            ViewUnmap,
            0,
            PAGE_READWRITE,
        );

        captured_client_view.SectionOffset = section_offset.__bindgen_anon_1.LowPart;

        if status < 0 {
            ObDereferenceObject(section_to_map);
            ObDereferenceObject(client_port as PVOID);
            return status;
        }

        captured_client_view.ViewBase = (*client_port).ClientSectionBase;

        (*client_port).__bindgen_anon_1.MappingProcess = PsGetCurrentProcess();
        ObReferenceObject((*client_port).__bindgen_anon_1.MappingProcess as PVOID);
    } else {
        section_to_map = core::ptr::null_mut();
    }

    if connection_info_length > (*port).MaxConnectionInfoLength as ULONG {
        connection_info_length = (*port).MaxConnectionInfoLength as ULONG;
    }

    message = LpcpAllocateFromPortZone();
    if message.is_null() {
        if !section_to_map.is_null() {
            ObDereferenceObject(section_to_map);
        }
        ObDereferenceObject(client_port as PVOID);
        return STATUS_NO_MEMORY;
    }

    connect_message = (message as usize + core::mem::size_of::<_LPCP_MESSAGE>()) as PLPCP_CONNECTION_MESSAGE;
    (*message).Request.__bindgen_anon_1.ClientId = (*thread).Cid;

    if !client_view.is_null() {
        (*message).Request.__bindgen_anon_2.ClientViewSize = captured_client_view.ViewSize;

        core::ptr::copy_nonoverlapping(
            &captured_client_view as *const PORT_VIEW as *const u8,
            core::ptr::addr_of_mut!((*connect_message).ClientView) as *mut u8,
            core::mem::size_of::<PORT_VIEW>(),
        );
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*connect_message).ServerView) as *mut u8,
            0,
            core::mem::size_of::<REMOTE_PORT_VIEW>(),
        );
    } else {
        (*message).Request.__bindgen_anon_2.ClientViewSize = 0;
        core::ptr::write_bytes(
            connect_message as *mut u8,
            0,
            core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>(),
        );
    }

    (*connect_message).ClientPort = core::ptr::null_mut();
    (*connect_message).SectionToMap = section_to_map;

    (*message).Request.u1.s1.DataLength = (connection_info_length
        + core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>() as ULONG) as i16;
    (*message).Request.u1.s1.TotalLength = (core::mem::size_of::<_LPCP_MESSAGE>()
        + (*message).Request.u1.s1.DataLength as usize) as i16;
    (*message).Request.u2.s2.Type = LPC_CONNECTION_REQUEST;

    if !connection_information.is_null() {
        core::ptr::copy_nonoverlapping(
            connection_information as *const u8,
            (connect_message as usize + core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>()) as *mut u8,
            connection_info_length as usize,
        );
    }

    status = STATUS_SUCCESS;

    KeAcquireGuardedMutex(&raw mut LpcpLock);

    if lpcp_check_name_deleted(port) {
        status = STATUS_OBJECT_NAME_NOT_FOUND;
    } else {
        (*message).RepliedToThread = core::ptr::null_mut();

        (*message).Request.MessageId = LpcpNextMessageId;
        LpcpNextMessageId += 1;
        if LpcpNextMessageId == 0 {
            LpcpNextMessageId = 1;
        }
        (*thread).LpcReplyMessageId = (*message).Request.MessageId;

        InsertTailList(
            core::ptr::addr_of_mut!((*port).MsgQueue.ReceiveHead),
            core::ptr::addr_of_mut!((*message).__bindgen_anon_1.Entry),
        );
        InsertTailList(
            core::ptr::addr_of_mut!((*port).LpcReplyChainHead),
            core::ptr::addr_of_mut!((*thread).__bindgen_anon_1.LpcReplyChain),
        );
        (*thread).__bindgen_anon_5.LpcReplyMessage = message as PVOID;

        ObReferenceObject(client_port as PVOID);
        (*connect_message).ClientPort = client_port;

        KeEnterCriticalRegion();
    }

    ObReferenceObject(port as PVOID);

    KeReleaseGuardedMutex(&raw mut LpcpLock);

    if status >= 0 {
        if (core::ptr::read_volatile(&(*port).Flags as *const ULONG) & LPCP_WAITABLE_PORT) != 0 {
            KeSetEvent(
                core::ptr::addr_of_mut!((*port).WaitEvent) as PVOID,
                1,
                0u8,
            );
        }

        LpcpCompleteWait((*port).MsgQueue.Semaphore as PVOID);
        KeLeaveCriticalRegion();

        LpcpConnectWait(
            core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID,
            previous_mode,
        );
    }

    section_to_map = LpcpFreeConMsg(&mut message, &mut connect_message, thread);

    if status < 0 {
        if KeReadStateSemaphore(core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID) != 0
        {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.LpcReplySemaphore) as PVOID,
                WrExecutive,
                KernelMode,
                0u8,
                core::ptr::null_mut(),
            );
        }

        if !message.is_null() {
            LpcpFreeToPortZone(message, 0);
        }
        if !section_to_map.is_null() {
            ObDereferenceObject(section_to_map);
        }
        ObDereferenceObject(client_port as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    if !message.is_null() {
        if ((*message).Request.u1.s1.DataLength as ULONG
            - core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>() as ULONG)
            < connection_info_length
        {
            connection_info_length = (*message).Request.u1.s1.DataLength as ULONG
                - core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>() as ULONG;
        }

        if !connection_information.is_null() {
            if !connection_information_length.is_null() {
                *connection_information_length = connection_info_length;
            }

            core::ptr::copy_nonoverlapping(
                (connect_message as usize + core::mem::size_of::<_LPCP_CONNECTION_MESSAGE>()) as *const u8,
                connection_information as *mut u8,
                connection_info_length as usize,
            );
        }

        if !(*client_port).ConnectedPort.is_null() {
            port_message_length = (*port).MaxMessageLength as ULONG;

            status = ObInsertObject(
                client_port as PVOID,
                core::ptr::null_mut(),
                PORT_ALL_ACCESS,
                0,
                core::ptr::null_mut(),
                &mut handle,
            );
            if status >= 0 {
                *port_handle = handle;

                if !max_message_length.is_null() {
                    *max_message_length = port_message_length;
                }

                if !client_view.is_null() {
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*connect_message).ClientView) as *const u8,
                        client_view as *mut u8,
                        core::mem::size_of::<PORT_VIEW>(),
                    );
                }

                if !server_view.is_null() {
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*connect_message).ServerView) as *const u8,
                        server_view as *mut u8,
                        core::mem::size_of::<REMOTE_PORT_VIEW>(),
                    );
                }
            }
        } else {
            if !section_to_map.is_null() {
                ObDereferenceObject(section_to_map);
            }

            KeAcquireGuardedMutex(&raw mut LpcpLock);

            if (*client_port).ConnectionPort.is_null() || lpcp_check_name_deleted(port)
            {
                status = STATUS_OBJECT_NAME_NOT_FOUND;
            } else {
                status = STATUS_PORT_CONNECTION_REFUSED;
            }

            KeReleaseGuardedMutex(&raw mut LpcpLock);

            ObDereferenceObject(client_port as PVOID);
        }

        LpcpFreeToPortZone(message, 0);
    } else {
        status = STATUS_PORT_CONNECTION_REFUSED;
        if !message.is_null() {
            LpcpFreeToPortZone(message, 0);
        }
        if !section_to_map.is_null() {
            ObDereferenceObject(section_to_map);
        }
        ObDereferenceObject(client_port as PVOID);
        ObDereferenceObject(port as PVOID);
        return status;
    }

    ObDereferenceObject(port as PVOID);

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtConnectPort(
    port_handle: PHANDLE,
    port_name: PUNICODE_STRING,
    security_qos: PSECURITY_QUALITY_OF_SERVICE,
    client_view: PPORT_VIEW,
    server_view: PREMOTE_PORT_VIEW,
    max_message_length: *mut ULONG,
    connection_information: PVOID,
    connection_information_length: *mut ULONG,
) -> NTSTATUS {
    let port_handle = opaque_ptr(port_handle);
    let port_name = opaque_ptr(port_name);
    let max_message_length = opaque_ptr(max_message_length);
    let connection_information = opaque_ptr(connection_information);
    let connection_information_length = opaque_ptr(connection_information_length);
    NtSecureConnectPort(
        port_handle,
        port_name,
        security_qos,
        client_view,
        core::ptr::null_mut(),
        server_view,
        max_message_length,
        connection_information,
        connection_information_length,
    )
}
