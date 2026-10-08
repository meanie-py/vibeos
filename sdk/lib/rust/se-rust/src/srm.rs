use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, NT_SUCCESS, TRUE, FALSE, PAGE_SIZE, PAGE_READWRITE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "system" {
    fn RtlEqualLuid(luid1: *const LUID, luid2: *const LUID) -> BOOLEAN;
    static mut PsInitialSystemProcess: PVOID;
}
extern "C" {
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: *mut _UNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    );
}

#[inline]
unsafe fn InitializeListHead(ListHead: *mut _LIST_ENTRY) {
    (*ListHead).Flink = ListHead;
    (*ListHead).Blink = ListHead;
}

#[inline]
unsafe fn InsertHeadList(ListHead: *mut _LIST_ENTRY, Entry: *mut _LIST_ENTRY) {
    let OldFlink = (*ListHead).Flink;
    (*Entry).Flink = OldFlink;
    (*Entry).Blink = ListHead;
    (*OldFlink).Blink = Entry;
    (*ListHead).Flink = Entry;
}

#[inline]
unsafe fn RemoveEntryList(Entry: *mut _LIST_ENTRY) -> BOOLEAN {
    let OldFlink = (*Entry).Flink;
    let OldBlink = (*Entry).Blink;
    (*OldBlink).Flink = OldFlink;
    (*OldFlink).Blink = OldBlink;
    if OldFlink == OldBlink { 1 } else { 0 }
}

struct SEP_LOGON_SESSION_TERMINATED_NOTIFICATION {
    Next: *mut SEP_LOGON_SESSION_TERMINATED_NOTIFICATION,
    CallbackRoutine: PSE_LOGON_SESSION_TERMINATED_ROUTINE,
}

type PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION = *mut SEP_LOGON_SESSION_TERMINATED_NOTIFICATION;

extern "system" {
    static mut SeSystemAuthenticationId: LUID;
    static mut SeAnonymousAuthenticationId: LUID;
}

static mut SeRmCommandPort: HANDLE = core::ptr::null_mut();
static mut SeLsaInitEvent: HANDLE = core::ptr::null_mut();

static mut SepCommandPortViewBase: PVOID = core::ptr::null_mut();
static mut SepCommandPortViewRemoteBase: PVOID = core::ptr::null_mut();
static mut SepCommandPortViewBaseOffset: ULONG_PTR = 0;

static mut SepRmCommandMessagePort: HANDLE = core::ptr::null_mut();

static mut SepAdtAuditingEnabled: BOOLEAN = 0u8;
static mut SepAdtMinListLength: ULONG = 0x2000;
static mut SepAdtMaxListLength: ULONG = 0x3000;

const POLICY_AUDIT_EVENT_TYPE_COUNT: usize = 9;
static mut SeAuditingState: [UCHAR; POLICY_AUDIT_EVENT_TYPE_COUNT] = [0u8; POLICY_AUDIT_EVENT_TYPE_COUNT];

static mut SepRmDbLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
static mut SepLogonSessions: PSEP_LOGON_SESSION_REFERENCES = core::ptr::null_mut();
static mut SepLogonNotifications: PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn SepRegQueryHelper(
    KeyName: PCWSTR,
    ValueName: PCWSTR,
    ValueType: ULONG,
    DataLength: ULONG,
    ValueData: PVOID,
) -> NTSTATUS {
    let ValueData = opaque_ptr(ValueData);
    let mut ValueNameString: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyNameString: _UNICODE_STRING = core::mem::zeroed();
    let mut ResultLength: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    #[repr(C)]
    struct KeyValueInfo {
        Partial: KEY_VALUE_PARTIAL_INFORMATION,
        Buffer: [UCHAR; 64],
    }
    let mut KeyValueInformation: KeyValueInfo = core::mem::zeroed();

    RtlInitUnicodeString(&mut KeyNameString, KeyName);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyNameString,
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let mut Status = ZwOpenKey(&mut KeyHandle, KEY_QUERY_VALUE, &mut ObjectAttributes);
    if Status < 0 {
        return Status;
    }

    RtlInitUnicodeString(&mut ValueNameString, ValueName);
    Status = ZwQueryValueKey(
        KeyHandle,
        &mut ValueNameString,
        2, // KeyValuePartialInformation
        &mut KeyValueInformation.Partial as *mut _ as PVOID,
        core::mem::size_of::<KeyValueInfo>() as ULONG,
        &mut ResultLength,
    );
    if Status < 0 {
        let CloseStatus = ZwClose(KeyHandle);
        ntkrnl_sys::ASSERT!(CloseStatus >= 0);
        return Status;
    }

    if (KeyValueInformation.Partial.Type != ValueType)
        || (KeyValueInformation.Partial.DataLength != DataLength)
    {
        Status = STATUS_OBJECT_TYPE_MISMATCH;
        let CloseStatus = ZwClose(KeyHandle);
        ntkrnl_sys::ASSERT!(CloseStatus >= 0);
        return Status;
    }

    if ValueType == REG_BINARY {
        core::ptr::copy_nonoverlapping(
            KeyValueInformation.Partial.Data.as_ptr(),
            ValueData as *mut u8,
            DataLength as usize,
        );
    } else if ValueType == REG_DWORD {
        *(ValueData as *mut ULONG) = *(KeyValueInformation.Partial.Data.as_ptr() as *const ULONG);
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    let CloseStatus = ZwClose(KeyHandle);
    ntkrnl_sys::ASSERT!(CloseStatus >= 0);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SeRmInitPhase0() -> BOOLEAN {
    KeInitializeGuardedMutex(&raw mut SepRmDbLock);

    let mut Status = SepRmCreateLogonSession(&raw mut SeSystemAuthenticationId);
    if Status < 0 {
        return 0u8;
    }

    Status = SepRmCreateLogonSession(&raw mut SeAnonymousAuthenticationId);
    if Status < 0 {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SeRmInitPhase1() -> BOOLEAN {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();

    RtlInitUnicodeString(&mut Name, w!("\\SeRmCommandPort").as_ptr() as *const u16);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Name,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let mut Status = ZwCreatePort(
        &raw mut SeRmCommandPort,
        &mut ObjectAttributes,
        core::mem::size_of::<ULONG>() as ULONG,
        256, // PORT_MAXIMUM_MESSAGE_LENGTH
        (2 * PAGE_SIZE) as ULONG,
    );
    if Status < 0 {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, w!("\\SeLsaInitEvent").as_ptr() as *const u16);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Name,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwCreateEvent(
        &raw mut SeLsaInitEvent,
        GENERIC_WRITE,
        &mut ObjectAttributes,
        0, // NotificationEvent (EVENT_TYPE: Notification=0, Synchronization=1)
        0u8,
    );
    if Status < 0 {
        return 0u8;
    }

    Status = PsCreateSystemThread(
        &mut ThreadHandle,
        THREAD_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::mem::transmute(SepRmCommandServerThread as unsafe extern "system" fn(PVOID)),
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return 0u8;
    }

    ObCloseHandle(ThreadHandle, 0i8); // KernelMode

    1u8
}

unsafe fn SepAdtInitializeBounds() {
    #[repr(C)]
    struct ListBoundsStruct {
        MaxLength: ULONG,
        MinLength: ULONG,
    }
    let mut ListBounds: ListBoundsStruct = core::mem::zeroed();

    let Status = SepRegQueryHelper(
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Lsa").as_ptr() as *const u16,
        w!("Bounds").as_ptr() as *const u16,
        REG_BINARY,
        core::mem::size_of::<ListBoundsStruct>() as ULONG,
        &mut ListBounds as *mut _ as PVOID,
    );
    if Status < 0 {
        return;
    }

    if (ListBounds.MaxLength < ListBounds.MinLength)
        || (ListBounds.MinLength < 16)
        || (ListBounds.MaxLength - ListBounds.MinLength < 16)
    {
        return;
    }

    SepAdtMinListLength = ListBounds.MinLength;
    SepAdtMaxListLength = ListBounds.MaxLength;
}

unsafe fn SepRmSetAuditEvent(Message: PSEP_RM_API_MESSAGE) -> NTSTATUS {
    SepAdtInitializeBounds();

    ntkrnl_sys::ASSERT!((*Message).ApiNumber == _RM_API_NUMBER_RmAuditSetCommand as ULONG);
    (*Message).ApiNumber = 0;

    SepAdtAuditingEnabled = (*Message).u.SetAuditEvent.Enabled;

    for i in 0..POLICY_AUDIT_EVENT_TYPE_COUNT {
        SeAuditingState[i] = (*Message).u.SetAuditEvent.Flags[i] as UCHAR;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SepRmInsertLogonSessionIntoToken(Token: PTOKEN) -> NTSTATUS {
    let Token = opaque_ptr(Token);
    ntkrnl_sys::ASSERT!(!Token.is_null());

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut LogonSession = SepLogonSessions;
    while !LogonSession.is_null() {
        if RtlEqualLuid(
            core::ptr::addr_of_mut!((*LogonSession).LogonId),
            core::ptr::addr_of_mut!((*Token).AuthenticationId),
        ) != 0
        {
            break;
        }
        LogonSession = (*LogonSession).Next;
    }

    if LogonSession.is_null() {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    (*Token).LogonSession = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<SEP_LOGON_SESSION_REFERENCES>() as SIZE_T,
        TAG_LOGON_SESSION,
    ) as PSEP_LOGON_SESSION_REFERENCES;
    if (*Token).LogonSession.is_null() {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*(*Token).LogonSession).Next = (*LogonSession).Next;
    (*(*Token).LogonSession).LogonId = (*LogonSession).LogonId;
    (*(*Token).LogonSession).ReferenceCount = (*LogonSession).ReferenceCount;
    (*(*Token).LogonSession).Flags = (*LogonSession).Flags;
    (*(*Token).LogonSession).pDeviceMap = (*LogonSession).pDeviceMap;
    InsertHeadList(
        core::ptr::addr_of_mut!((*LogonSession).TokenList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*(*Token).LogonSession).TokenList) as PLIST_ENTRY,
    );

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SepRmRemoveLogonSessionFromToken(Token: PTOKEN) -> NTSTATUS {
    let Token = opaque_ptr(Token);
    ntkrnl_sys::ASSERT!(!Token.is_null());

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut LogonSession = SepLogonSessions;
    while !LogonSession.is_null() {
        if RtlEqualLuid(
            core::ptr::addr_of_mut!((*LogonSession).LogonId),
            core::ptr::addr_of_mut!((*Token).AuthenticationId),
        ) != 0
        {
            break;
        }
        LogonSession = (*LogonSession).Next;
    }

    if LogonSession.is_null() {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*(*Token).LogonSession).TokenList) as PLIST_ENTRY);
    ExFreePoolWithTag((*Token).LogonSession as PVOID, TAG_LOGON_SESSION);

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);
    STATUS_SUCCESS
}

unsafe fn SepRmCreateLogonSession(LogonLuid: PLUID) -> NTSTATUS {
    let NewSession = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<SEP_LOGON_SESSION_REFERENCES>() as SIZE_T,
        TAG_LOGON_SESSION,
    ) as PSEP_LOGON_SESSION_REFERENCES;
    if NewSession.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*NewSession).LogonId = *LogonLuid;
    (*NewSession).ReferenceCount = 0;
    (*NewSession).Flags = 0;
    (*NewSession).pDeviceMap = core::ptr::null_mut();
    InitializeListHead(core::ptr::addr_of_mut!((*NewSession).TokenList) as PLIST_ENTRY);

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut CurrentSession = SepLogonSessions;
    while !CurrentSession.is_null() {
        if RtlEqualLuid(core::ptr::addr_of_mut!((*CurrentSession).LogonId), LogonLuid) != 0 {
            KeReleaseGuardedMutex(&raw mut SepRmDbLock);
            ExFreePoolWithTag(NewSession as PVOID, TAG_LOGON_SESSION);
            return STATUS_LOGON_SESSION_EXISTS;
        }
        CurrentSession = (*CurrentSession).Next;
    }

    (*NewSession).Next = SepLogonSessions;
    SepLogonSessions = NewSession;

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    STATUS_SUCCESS
}

unsafe fn SepRmDeleteLogonSession(LogonLuid: PLUID) -> NTSTATUS {
    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut SessionToDelete = SepLogonSessions;
    while !SessionToDelete.is_null() {
        if RtlEqualLuid(core::ptr::addr_of_mut!((*SessionToDelete).LogonId), LogonLuid) != 0 {
            if RtlEqualLuid(
                core::ptr::addr_of_mut!((*SessionToDelete).LogonId),
                &raw mut SeSystemAuthenticationId,
            ) != 0
                || RtlEqualLuid(
                    core::ptr::addr_of_mut!((*SessionToDelete).LogonId),
                    &raw mut SeAnonymousAuthenticationId,
                ) != 0
            {
                KeReleaseGuardedMutex(&raw mut SepRmDbLock);
                return STATUS_BAD_LOGON_SESSION_STATE;
            } else {
                break;
            }
        }
        SessionToDelete = (*SessionToDelete).Next;
    }

    if SessionToDelete.is_null() {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    if (*SessionToDelete).ReferenceCount != 0 {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_BAD_LOGON_SESSION_STATE;
    }

    if !(*SessionToDelete).pDeviceMap.is_null() {
        let Status = SepCleanupLUIDDeviceMapDirectory(LogonLuid);
        if Status < 0 {
            KeReleaseGuardedMutex(&raw mut SepRmDbLock);
            return Status;
        }

        ObfDereferenceDeviceMap((*SessionToDelete).pDeviceMap);
    }

    let Status = STATUS_SUCCESS;
    ExFreePoolWithTag(SessionToDelete as PVOID, TAG_LOGON_SESSION);

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);
    Status
}

#[no_mangle]
pub unsafe extern "C" fn SepRmReferenceLogonSession(LogonLuid: PLUID) -> NTSTATUS {
    let LogonLuid = opaque_ptr(LogonLuid);
    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut CurrentSession = SepLogonSessions;
    while !CurrentSession.is_null() {
        if RtlEqualLuid(core::ptr::addr_of_mut!((*CurrentSession).LogonId), LogonLuid) != 0 {
            (*CurrentSession).ReferenceCount += 1;

            KeReleaseGuardedMutex(&raw mut SepRmDbLock);
            return STATUS_SUCCESS;
        }
        CurrentSession = (*CurrentSession).Next;
    }

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    STATUS_NO_SUCH_LOGON_SESSION
}

unsafe fn SepCleanupLUIDDeviceMapDirectory(LogonLuid: PLUID) -> NTSTATUS {
    if LogonLuid.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let UseCurrentProc = ObReferenceObjectSafe(PsGetCurrentProcess() as PVOID) != 0;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();
    if UseCurrentProc {
        ObDereferenceObject(PsGetCurrentProcess() as PVOID);
    } else {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(PsInitialSystemProcess as *mut _EPROCESS)).Pcb),
            &mut ApcState,
        );
    }

    let mut Buffer: [u16; 63] = [0; 63];
    let written = {
        let mut pos = 0;
        let prefix = b"\\Sessions\\0\\DosDevices\\\0";
        for &ch in prefix {
            if pos >= 63 {
                break;
            }
            Buffer[pos] = ch as u16;
            pos += 1;
        }
        pos
    };

    let mut DirectoryName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut DirectoryName, Buffer.as_ptr());

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut DirectoryName,
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut Status = ZwOpenDirectoryObject(&mut DirectoryHandle, DIRECTORY_QUERY, &mut ObjectAttributes);
    if Status < 0 {
        if !UseCurrentProc {
            KeUnstackDetachProcess(&mut ApcState);
        }
        return Status;
    }

    let mut Context: ULONG = 0;
    let mut DirectoryInfo: POBJECT_DIRECTORY_INFORMATION = core::ptr::null_mut();
    let mut DirInfoLength: ULONG = 0;
    let mut LinksCount: ULONG = 100;
    let mut CurrentLinks: ULONG = 0;
    let mut LinksSize = LinksCount * core::mem::size_of::<HANDLE>() as ULONG;

    'alloc_links: loop {
        let LinksBuffer = ExAllocatePoolWithTag(PagedPool, LinksSize as SIZE_T, TAG_SE_HANDLES_TAB) as PHANDLE;
        if LinksBuffer.is_null() {
            ZwClose(DirectoryHandle);
            if !DirectoryInfo.is_null() {
                ExFreePoolWithTag(DirectoryInfo as PVOID, TAG_SE_DIR_BUFFER);
            }
            if !UseCurrentProc {
                KeUnstackDetachProcess(&mut ApcState);
            }
            return STATUS_NO_MEMORY;
        }

        let mut RestartScan = 1u8;
        loop {
            loop {
                let mut ReturnLength: ULONG = 0;
                Status = ZwQueryDirectoryObject(
                    DirectoryHandle,
                    DirectoryInfo as PVOID,
                    DirInfoLength,
                    1u8,
                    RestartScan,
                    &mut Context,
                    &mut ReturnLength,
                );
                if Status != STATUS_BUFFER_TOO_SMALL {
                    break;
                }

                DirInfoLength = ReturnLength;
                if !DirectoryInfo.is_null() {
                    ExFreePoolWithTag(DirectoryInfo as PVOID, TAG_SE_DIR_BUFFER);
                }

                DirectoryInfo = ExAllocatePoolWithTag(PagedPool, DirInfoLength as SIZE_T, TAG_SE_DIR_BUFFER)
                    as POBJECT_DIRECTORY_INFORMATION;
                if DirectoryInfo.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }
            }

            if Status < 0 {
                break;
            }

            RestartScan = 0u8;

            if CurrentLinks >= LinksCount {
                for i in 0..CurrentLinks {
                    ZwClose(*LinksBuffer.add(i as usize));
                }

                LinksCount += 20;
                CurrentLinks = 0;
                ExFreePoolWithTag(LinksBuffer as PVOID, TAG_SE_HANDLES_TAB);
                LinksSize = LinksCount * core::mem::size_of::<HANDLE>() as ULONG;

                continue 'alloc_links;
            }

            let mut LinkHandle: HANDLE = core::ptr::null_mut();
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                core::ptr::addr_of_mut!((*DirectoryInfo).Name),
                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
                DirectoryHandle,
                core::ptr::null_mut(),
            );
            if ZwOpenSymbolicLinkObject(&mut LinkHandle, SYMBOLIC_LINK_ALL_ACCESS, &mut ObjectAttributes) >= 0 {
                if ZwMakeTemporaryObject(LinkHandle) < 0 {
                    ZwClose(LinkHandle);
                } else {
                    *LinksBuffer.add(CurrentLinks as usize) = LinkHandle;
                    CurrentLinks += 1;
                }
            }
        }

        if Status == STATUS_NO_MORE_ENTRIES {
            Status = STATUS_SUCCESS;
        }

        for i in 0..CurrentLinks {
            ZwClose(*LinksBuffer.add(i as usize));
        }
        ExFreePoolWithTag(LinksBuffer as PVOID, TAG_SE_HANDLES_TAB);

        break;
    }

    if !DirectoryInfo.is_null() {
        ExFreePoolWithTag(DirectoryInfo as PVOID, TAG_SE_DIR_BUFFER);
    }

    ZwClose(DirectoryHandle);

    if !UseCurrentProc {
        KeUnstackDetachProcess(&mut ApcState);
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn SepRmDereferenceLogonSession(LogonLuid: PLUID) -> NTSTATUS {
    let LogonLuid = opaque_ptr(LogonLuid);
    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut CurrentSession = SepLogonSessions;
    while !CurrentSession.is_null() {
        if RtlEqualLuid(core::ptr::addr_of_mut!((*CurrentSession).LogonId), LogonLuid) != 0 {
            (*CurrentSession).ReferenceCount -= 1;
            let RefCount = (*CurrentSession).ReferenceCount;

            KeReleaseGuardedMutex(&raw mut SepRmDbLock);

            if RefCount == 0 {
                let DeviceMap = (*CurrentSession).pDeviceMap;
                if !DeviceMap.is_null() {
                    (*CurrentSession).pDeviceMap = core::ptr::null_mut();
                    SepCleanupLUIDDeviceMapDirectory(LogonLuid);
                    ObfDereferenceDeviceMap(DeviceMap);
                }
            }

            return STATUS_SUCCESS;
        }
        CurrentSession = (*CurrentSession).Next;
    }

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    STATUS_NO_SUCH_LOGON_SESSION
}

unsafe fn SepRmCommandServerThreadInit() -> BOOLEAN {
    let mut SecurityQos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut Message: SEP_RM_API_MESSAGE = core::mem::zeroed();
    let mut PortName: _UNICODE_STRING = core::mem::zeroed();
    let mut RemotePortView: REMOTE_PORT_VIEW = core::mem::zeroed();
    let mut PortView: PORT_VIEW = core::mem::zeroed();
    let mut SectionSize: LARGE_INTEGER = core::mem::zeroed();
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut PortHandle: HANDLE = core::ptr::null_mut();

    let mut Result = 1u8;

    let mut Status = ZwWaitForSingleObject(SeLsaInitEvent, 0u8, core::ptr::null_mut());
    if Status < 0 {
        Result = 0u8;
    } else {
        ObCloseHandle(SeLsaInitEvent, 0i8); // KernelMode

        Message.Header.u1.s1.TotalLength = core::mem::size_of::<SEP_RM_API_MESSAGE>() as i16;
        Message.Header.u1.s1.DataLength = 0;

        Status = ZwListenPort(SeRmCommandPort, &mut Message.Header);
        if Status < 0 {
            Result = 0u8;
        } else {
            RemotePortView.Length = core::mem::size_of::<REMOTE_PORT_VIEW>() as ULONG;

            Status = ZwAcceptConnectPort(
                &raw mut SepRmCommandMessagePort,
                core::ptr::null_mut(),
                &mut Message.Header,
                1u8,
                core::ptr::null_mut(),
                &mut RemotePortView,
            );
            if Status < 0 {
                Result = 0u8;
            } else {
                Status = ZwCompleteConnectPort(SepRmCommandMessagePort);
                if Status < 0 {
                    Result = 0u8;
                } else {
                    SectionSize.QuadPart = PAGE_SIZE as i64;
                    Status = ZwCreateSection(
                        &mut SectionHandle,
                        SECTION_ALL_ACCESS,
                        core::ptr::null_mut(),
                        &mut SectionSize,
                        PAGE_READWRITE,
                        0x8000000, // SEC_COMMIT
                        core::ptr::null_mut(),
                    );
                    if Status < 0 {
                        Result = 0u8;
                    } else {
                        PortView.Length = core::mem::size_of::<PORT_VIEW>() as ULONG;
                        PortView.SectionHandle = SectionHandle;
                        PortView.SectionOffset = 0;
                        PortView.ViewSize = SectionSize.__bindgen_anon_1.LowPart as SIZE_T;
                        PortView.ViewBase = core::ptr::null_mut();
                        PortView.ViewRemoteBase = core::ptr::null_mut();

                        SecurityQos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG;
                        SecurityQos.ImpersonationLevel = 2; // SecurityImpersonation
                        SecurityQos.ContextTrackingMode = 1; // SECURITY_DYNAMIC_TRACKING
                        SecurityQos.EffectiveOnly = 1u8;

                        RtlInitUnicodeString(&mut PortName, w!("\\SeLsaCommandPort").as_ptr() as *const u16);
                        Status = ZwConnectPort(
                            &mut PortHandle,
                            &mut PortName,
                            &mut SecurityQos,
                            &mut PortView,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        );
                        if Status < 0 {
                            Result = 0u8;
                        } else {
                            SepCommandPortViewBase = PortView.ViewBase;
                            SepCommandPortViewRemoteBase = PortView.ViewRemoteBase;
                            SepCommandPortViewBaseOffset =
                                SepCommandPortViewRemoteBase as ULONG_PTR - SepCommandPortViewBase as ULONG_PTR;
                        }
                    }
                }
            }
        }
    }

    if Result == 0 {
        if !PortHandle.is_null() {
            ObCloseHandle(PortHandle, 0i8); // KernelMode
        }
    }

    if !SectionHandle.is_null() {
        ObCloseHandle(SectionHandle, 0i8); // KernelMode
    }

    Result
}

unsafe extern "system" fn SepRmCommandServerThread(_StartContext: PVOID) {
    if SepRmCommandServerThreadInit() == 0 {
        return;
    }

    let mut Message: SEP_RM_API_MESSAGE = core::mem::zeroed();
    let mut ReplyMessage: PPORT_MESSAGE = core::ptr::null_mut();
    let mut DummyPortHandle: HANDLE = core::ptr::null_mut();

    loop {
        let Status = ZwReplyWaitReceivePort(
            SepRmCommandMessagePort,
            core::ptr::null_mut(),
            ReplyMessage,
            &mut Message.Header,
        );
        if Status < 0 {
            ReplyMessage = core::ptr::null_mut();
            continue;
        }

        if Message.Header.u2.s2.Type == _LPC_TYPE_LPC_CONNECTION_REQUEST as CSHORT {
            ZwAcceptConnectPort(
                &mut DummyPortHandle,
                core::ptr::null_mut(),
                &mut Message.Header,
                0u8,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            ReplyMessage = core::ptr::null_mut();
            continue;
        }

        if Message.Header.u2.s2.Type == _LPC_TYPE_LPC_PORT_CLOSED as CSHORT
            || Message.Header.u2.s2.Type == _LPC_TYPE_LPC_CLIENT_DIED as CSHORT
        {
            break;
        }

        if Message.Header.u2.s2.Type != _LPC_TYPE_LPC_REQUEST as CSHORT {
            ReplyMessage = core::ptr::null_mut();
            continue;
        }

        ReplyMessage = &mut Message.Header;

        // RM_API_NUMBER starts at 1 (srmp.h). Literal 0/1/2 arms here sent every lsass
        // RmCreateLogonSession to the DELETE handler, so each LogonUser with the right
        // password failed STATUS_NO_SUCH_LOGON_SESSION (found by modrun's logon suite).
        let ResultStatus = match Message.ApiNumber as _RM_API_NUMBER {
            _RM_API_NUMBER_RmAuditSetCommand => SepRmSetAuditEvent(&mut Message),
            _RM_API_NUMBER_RmCreateLogonSession => SepRmCreateLogonSession(&mut Message.u.LogonLuid),
            _RM_API_NUMBER_RmDeleteLogonSession => SepRmDeleteLogonSession(&mut Message.u.LogonLuid),
            _ => {
                ReplyMessage = core::ptr::null_mut();
                STATUS_INVALID_PARAMETER
            }
        };

        if !ReplyMessage.is_null() {
            Message.u.ResultStatus = ResultStatus;
        }
    }

    ObCloseHandle(SepRmCommandMessagePort, 0i8); // KernelMode
    ObCloseHandle(SeRmCommandPort, 0i8); // KernelMode
}

#[no_mangle]
pub unsafe extern "system" fn SeGetLogonIdDeviceMap(
    LogonId: PLUID,
    DeviceMap: *mut PDEVICE_MAP,
) -> NTSTATUS {
    let LogonId = opaque_ptr(LogonId);
    let DeviceMap = opaque_ptr(DeviceMap);
    if LogonId.is_null() || DeviceMap.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut CurrentSession = SepLogonSessions;
    while !CurrentSession.is_null() {
        if RtlEqualLuid(core::ptr::addr_of_mut!((*CurrentSession).LogonId), LogonId) != 0 {
            break;
        }
        CurrentSession = (*CurrentSession).Next;
    }

    if CurrentSession.is_null() {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_NO_SUCH_LOGON_SESSION;
    }

    if !(*CurrentSession).pDeviceMap.is_null() {
        *DeviceMap = (*CurrentSession).pDeviceMap;
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_SUCCESS;
    }

    let mut LocalMap: PDEVICE_MAP = core::ptr::null_mut();

    (*CurrentSession).ReferenceCount += 1;

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    let mut Buffer: [u16; 63] = [0; 63];
    let mut DirectoryName: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut DirectoryName, Buffer.as_ptr());

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut DirectoryName,
        OBJ_KERNEL_HANDLE | 0x00000080 | OBJ_CASE_INSENSITIVE, // OBJ_OPENIF
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    let mut DirectoryHandle: HANDLE = core::ptr::null_mut();
    let mut Status = ZwCreateDirectoryObject(&mut DirectoryHandle, DIRECTORY_ALL_ACCESS, &mut ObjectAttributes);
    if Status >= 0 {
        Status = ObSetDirectoryDeviceMap(&mut LocalMap, DirectoryHandle);
        if Status >= 0 {
            let mut LinkName: _UNICODE_STRING = core::mem::zeroed();
            let mut TargetName: _UNICODE_STRING = core::mem::zeroed();

            RtlInitUnicodeString(&mut LinkName, w!("Global").as_ptr() as *const u16);
            RtlInitUnicodeString(&mut TargetName, w!("\\Global??").as_ptr() as *const u16);

            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut LinkName,
                OBJ_KERNEL_HANDLE | 0x00000080 | OBJ_CASE_INSENSITIVE | OBJ_PERMANENT, // OBJ_OPENIF
                DirectoryHandle,
                core::ptr::null_mut(),
            );
            let mut LinkHandle: HANDLE = core::ptr::null_mut();
            Status = ZwCreateSymbolicLinkObject(&mut LinkHandle, SYMBOLIC_LINK_ALL_ACCESS, &mut ObjectAttributes, &mut TargetName);
            if Status < 0 {
                ObfDereferenceDeviceMap(LocalMap);
            } else {
                ZwClose(LinkHandle);
            }
        }

        ZwClose(DirectoryHandle);
    }

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    if Status >= 0 {
        if !(*CurrentSession).pDeviceMap.is_null() {
            ObfDereferenceDeviceMap(LocalMap);
        } else {
            (*CurrentSession).pDeviceMap = LocalMap;
        }

        *DeviceMap = (*CurrentSession).pDeviceMap;
    } else {
        *DeviceMap = core::ptr::null_mut();
    }

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    SepRmDereferenceLogonSession(core::ptr::addr_of_mut!((*CurrentSession).LogonId));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SeMarkLogonSessionForTerminationNotification(LogonId: PLUID) -> NTSTATUS {
    let LogonId = opaque_ptr(LogonId);
    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut SessionToMark = SepLogonSessions;
    while !SessionToMark.is_null() {
        if RtlEqualLuid(core::ptr::addr_of_mut!((*SessionToMark).LogonId), LogonId) != 0 {
            break;
        }
        SessionToMark = (*SessionToMark).Next;
    }

    if SessionToMark.is_null() {
        KeReleaseGuardedMutex(&raw mut SepRmDbLock);
        return STATUS_NOT_FOUND;
    }

    (*SessionToMark).Flags |= 0x1; // SEP_LOGON_SESSION_TERMINATION_NOTIFY

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SeRegisterLogonSessionTerminatedRoutine(
    CallbackRoutine: PSE_LOGON_SESSION_TERMINATED_ROUTINE,
) -> NTSTATUS {
    if CallbackRoutine.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    let Notification = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<SEP_LOGON_SESSION_TERMINATED_NOTIFICATION>() as SIZE_T,
        TAG_LOGON_NOTIFICATION,
    ) as PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION;
    if Notification.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    (*Notification).CallbackRoutine = CallbackRoutine;

    (*Notification).Next = SepLogonNotifications;
    SepLogonNotifications = Notification;

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    STATUS_SUCCESS
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn SeUnregisterLogonSessionTerminatedRoutine(
    CallbackRoutine: PSE_LOGON_SESSION_TERMINATED_ROUTINE,
) -> NTSTATUS {
    if CallbackRoutine.is_none() {
        return STATUS_INVALID_PARAMETER;
    }

    KeAcquireGuardedMutex(&raw mut SepRmDbLock);

    let mut Current = SepLogonNotifications;
    let mut Previous: PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION = core::ptr::null_mut();
    while !Current.is_null() {
        if (*Current).CallbackRoutine == CallbackRoutine {
            break;
        }
        Previous = Current;
        Current = (*Current).Next;
    }

    let Status = if Current.is_null() {
        STATUS_NOT_FOUND
    } else {
        if Previous.is_null() {
            SepLogonNotifications = (*Current).Next;
        } else {
            (*Previous).Next = (*Current).Next;
        }

        ExFreePoolWithTag(Current as PVOID, TAG_LOGON_NOTIFICATION);

        STATUS_SUCCESS
    };

    KeReleaseGuardedMutex(&raw mut SepRmDbLock);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security Reference Monitor Server
//  * COPYRIGHT:   Copyright Timo Kreuzer <timo.kreuzer@reactos.org>
//  *              Copyright Pierre Schweitzer <pierre@reactos.org>
//  *              Copyright 2021 George Bișoc <george.bisoc@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE DEFINITIONS ********************************************************/
//
// typedef struct _SEP_LOGON_SESSION_TERMINATED_NOTIFICATION
// {
//     struct _SEP_LOGON_SESSION_TERMINATED_NOTIFICATION *Next;
//     PSE_LOGON_SESSION_TERMINATED_ROUTINE CallbackRoutine;
// } SEP_LOGON_SESSION_TERMINATED_NOTIFICATION, *PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION;
//
// VOID
// NTAPI
// SepRmCommandServerThread(
//     _In_ PVOID StartContext);
//
// static
// NTSTATUS
// SepCleanupLUIDDeviceMapDirectory(
//     _In_ PLUID LogonLuid);
//
// static
// NTSTATUS
// SepRmCreateLogonSession(
//     _In_ PLUID LogonLuid);
//
//
// /* GLOBALS ********************************************************************/
//
// extern LUID SeSystemAuthenticationId;
// extern LUID SeAnonymousAuthenticationId;
//
// HANDLE SeRmCommandPort;
// HANDLE SeLsaInitEvent;
//
// PVOID SepCommandPortViewBase;
// PVOID SepCommandPortViewRemoteBase;
// ULONG_PTR SepCommandPortViewBaseOffset;
//
// static HANDLE SepRmCommandMessagePort;
//
// BOOLEAN SepAdtAuditingEnabled;
// ULONG SepAdtMinListLength = 0x2000;
// ULONG SepAdtMaxListLength = 0x3000;
//
// #define POLICY_AUDIT_EVENT_TYPE_COUNT 9 // (AuditCategoryAccountLogon - AuditCategorySystem + 1)
// UCHAR SeAuditingState[POLICY_AUDIT_EVENT_TYPE_COUNT];
//
// KGUARDED_MUTEX SepRmDbLock;
// PSEP_LOGON_SESSION_REFERENCES SepLogonSessions = NULL;
// PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION SepLogonNotifications = NULL;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /**
//  * @brief
//  * A private registry helper that returns the desired value
//  * data based on the specifics requested by the caller.
//  *
//  * @param[in] KeyName
//  * Name of the key.
//  *
//  * @param[in] ValueName
//  * Name of the registry value.
//  *
//  * @param[in] ValueType
//  * The type of the registry value.
//  *
//  * @param[in] DataLength
//  * The data length, in bytes, representing the size of the registry value.
//  *
//  * @param[out] ValueData
//  * The requested value data provided by the function.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the operations have completed successfully,
//  * otherwise a failure NTSTATUS code is returned.
//  */
// NTSTATUS
// NTAPI
// SepRegQueryHelper(
//     _In_ PCWSTR KeyName,
//     _In_ PCWSTR ValueName,
//     _In_ ULONG ValueType,
//     _In_ ULONG DataLength,
//     _Out_ PVOID ValueData)
// {
//     UNICODE_STRING ValueNameString;
//     UNICODE_STRING KeyNameString;
//     ULONG ResultLength;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle = NULL;
//     struct
//     {
//         KEY_VALUE_PARTIAL_INFORMATION Partial;
//         UCHAR Buffer[64];
//     } KeyValueInformation;
//     NTSTATUS Status, CloseStatus;
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&KeyNameString, KeyName);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyNameString,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = ZwOpenKey(&KeyHandle, KEY_QUERY_VALUE, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     RtlInitUnicodeString(&ValueNameString, ValueName);
//     Status = ZwQueryValueKey(KeyHandle,
//                              &ValueNameString,
//                              KeyValuePartialInformation,
//                              &KeyValueInformation.Partial,
//                              sizeof(KeyValueInformation),
//                              &ResultLength);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     if ((KeyValueInformation.Partial.Type != ValueType) ||
//         (KeyValueInformation.Partial.DataLength != DataLength))
//     {
//         Status = STATUS_OBJECT_TYPE_MISMATCH;
//         goto Cleanup;
//     }
//
//     if (ValueType == REG_BINARY)
//     {
//         RtlCopyMemory(ValueData, KeyValueInformation.Partial.Data, DataLength);
//     }
//     else if (ValueType == REG_DWORD)
//     {
//         *(PULONG)ValueData = *(PULONG)KeyValueInformation.Partial.Data;
//     }
//     else
//     {
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
// Cleanup:
//     CloseStatus = ZwClose(KeyHandle);
//     ASSERT(NT_SUCCESS( CloseStatus ));
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Manages the phase 0 initialization of the security reference
//  * monitoring module of the kernel.
//  *
//  * @return
//  * Returns TRUE when phase 0 initialization has completed without
//  * problems, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeRmInitPhase0(VOID)
// {
//     NTSTATUS Status;
//
//     /* Initialize the database lock */
//     KeInitializeGuardedMutex(&SepRmDbLock);
//
//     /* Create the system logon session */
//     Status = SepRmCreateLogonSession(&SeSystemAuthenticationId);
//     if (!NT_VERIFY(NT_SUCCESS(Status)))
//     {
//         return FALSE;
//     }
//
//     /* Create the anonymous logon session */
//     Status = SepRmCreateLogonSession(&SeAnonymousAuthenticationId);
//     if (!NT_VERIFY(NT_SUCCESS(Status)))
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Manages the phase 1 initialization of the security reference
//  * monitoring module of the kernel.
//  *
//  * @return
//  * Returns TRUE when phase 1 initialization has completed without
//  * problems, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeRmInitPhase1(VOID)
// {
//     UNICODE_STRING Name;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE ThreadHandle;
//     NTSTATUS Status;
//
//     /* Create the SeRm command port */
//     RtlInitUnicodeString(&Name, L"\\SeRmCommandPort");
//     InitializeObjectAttributes(&ObjectAttributes, &Name, 0, NULL, NULL);
//     Status = ZwCreatePort(&SeRmCommandPort,
//                           &ObjectAttributes,
//                           sizeof(ULONG),
//                           PORT_MAXIMUM_MESSAGE_LENGTH,
//                           2 * PAGE_SIZE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security: Rm Command Port creation failed: 0x%lx\n", Status);
//         return FALSE;
//     }
//
//     /* Create SeLsaInitEvent */
//     RtlInitUnicodeString(&Name, L"\\SeLsaInitEvent");
//     InitializeObjectAttributes(&ObjectAttributes, &Name, 0, NULL, NULL);
//     Status = ZwCreateEvent(&SeLsaInitEvent,
//                            GENERIC_WRITE,
//                            &ObjectAttributes,
//                            NotificationEvent,
//                            FALSE);
//     if (!NT_VERIFY((NT_SUCCESS(Status))))
//     {
//         DPRINT1("Security: LSA Init Event creation failed: 0x%lx\n", Status);
//         return FALSE;
//     }
//
//     /* Create the SeRm server thread */
//     Status = PsCreateSystemThread(&ThreadHandle,
//                                   THREAD_ALL_ACCESS,
//                                   NULL,
//                                   NULL,
//                                   NULL,
//                                   SepRmCommandServerThread,
//                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security: Rm Command Server Thread creation failed: 0x%lx\n", Status);
//         return FALSE;
//     }
//
//     ObCloseHandle(ThreadHandle, KernelMode);
//
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Initializes the local security authority audit bounds.
//  *
//  * @return
//  * Nothing.
//  */
// static
// VOID
// SepAdtInitializeBounds(VOID)
// {
//     struct
//     {
//         ULONG MaxLength;
//         ULONG MinLength;
//     } ListBounds;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     Status = SepRegQueryHelper(L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Lsa",
//                                L"Bounds",
//                                REG_BINARY,
//                                sizeof(ListBounds),
//                                &ListBounds);
//     if (!NT_SUCCESS(Status))
//     {
//         /* No registry values, so keep hardcoded defaults */
//         return;
//     }
//
//     /* Check if the bounds are valid */
//     if ((ListBounds.MaxLength < ListBounds.MinLength) ||
//         (ListBounds.MinLength < 16) ||
//         (ListBounds.MaxLength - ListBounds.MinLength < 16))
//     {
//         DPRINT1("ListBounds invalid: %lu, %lu\n", ListBounds.MinLength, ListBounds.MaxLength);
//         return;
//     }
//
//     /* Set the new bounds globally */
//     SepAdtMinListLength = ListBounds.MinLength;
//     SepAdtMaxListLength = ListBounds.MaxLength;
// }
//
// /**
//  * @brief
//  * Sets an audit event for future security auditing monitoring.
//  *
//  * @param[in,out] Message
//  * The reference monitoring API message. It is used to determine
//  * if the right API message number is provided, RmAuditSetCommand
//  * in this case.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// static
// NTSTATUS
// SepRmSetAuditEvent(
//     _Inout_ PSEP_RM_API_MESSAGE Message)
// {
//     ULONG i;
//     PAGED_CODE();
//
//     /* First re-initialize the bounds from the registry */
//     SepAdtInitializeBounds();
//
//     /* Make sure we have the right message and clear */
//     ASSERT(Message->ApiNumber == RmAuditSetCommand);
//     Message->ApiNumber = 0;
//
//     /* Store the enable flag in the global variable */
//     SepAdtAuditingEnabled = Message->u.SetAuditEvent.Enabled;
//
//     /* Loop all audit event types */
//     for (i = 0; i < POLICY_AUDIT_EVENT_TYPE_COUNT; i++)
//     {
//         /* Save the provided flags in the global array */
//         SeAuditingState[i] = (UCHAR)Message->u.SetAuditEvent.Flags[i];
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Inserts a logon session into an access token specified by the
//  * caller.
//  *
//  * @param[in,out] Token
//  * An access token where the logon session is about to be inserted
//  * in.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the logon session has been inserted into
//  * the token successfully. STATUS_NO_SUCH_LOGON_SESSION is returned when no logon
//  * session has been found with the matching ID of the token and as such
//  * we've failed to add the logon session to the token. STATUS_INSUFFICIENT_RESOURCES
//  * is returned if memory pool allocation for the new session has failed.
//  */
// NTSTATUS
// NTAPI
// SepRmInsertLogonSessionIntoToken(
//     _Inout_ PTOKEN Token)
// {
//     PSEP_LOGON_SESSION_REFERENCES LogonSession;
//     PAGED_CODE();
//
//     /* Ensure that our token is not some plain garbage */
//     ASSERT(Token);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     for (LogonSession = SepLogonSessions;
//          LogonSession != NULL;
//          LogonSession = LogonSession->Next)
//     {
//         /*
//          * The insertion of a logon session into the token has to be done
//          * only IF the authentication ID of the token matches with the ID
//          * of the logon itself.
//          */
//         if (RtlEqualLuid(&LogonSession->LogonId, &Token->AuthenticationId))
//         {
//             break;
//         }
//     }
//
//     /* If we reach this then we cannot proceed further */
//     if (LogonSession == NULL)
//     {
//         DPRINT1("SepRmInsertLogonSessionIntoToken(): Couldn't insert the logon session into the specific access token!\n");
//         KeReleaseGuardedMutex(&SepRmDbLock);
//         return STATUS_NO_SUCH_LOGON_SESSION;
//     }
//
//     /*
//      * Allocate the session that we are going
//      * to insert it to the token.
//      */
//     Token->LogonSession = ExAllocatePoolWithTag(PagedPool,
//                                                 sizeof(SEP_LOGON_SESSION_REFERENCES),
//                                                 TAG_LOGON_SESSION);
//     if (Token->LogonSession == NULL)
//     {
//         DPRINT1("SepRmInsertLogonSessionIntoToken(): Couldn't allocate new logon session into the memory pool!\n");
//         KeReleaseGuardedMutex(&SepRmDbLock);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /*
//      * Begin copying the logon session references data from the
//      * session whose ID matches with the token authentication ID to
//      * the new session we've allocated blocks of pool memory for it.
//      */
//     Token->LogonSession->Next = LogonSession->Next;
//     Token->LogonSession->LogonId = LogonSession->LogonId;
//     Token->LogonSession->ReferenceCount = LogonSession->ReferenceCount;
//     Token->LogonSession->Flags = LogonSession->Flags;
//     Token->LogonSession->pDeviceMap = LogonSession->pDeviceMap;
//     InsertHeadList(&LogonSession->TokenList, &Token->LogonSession->TokenList);
//
//     /* Release the database lock and we're done */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Removes a logon session from an access token.
//  *
//  * @param[in,out] Token
//  * An access token whose logon session is to be removed from it.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the logon session has been removed from
//  * the token successfully. STATUS_NO_SUCH_LOGON_SESSION is returned when no logon
//  * session has been found with the matching ID of the token and as such
//  * we've failed to remove the logon session from the token.
//  */
// NTSTATUS
// NTAPI
// SepRmRemoveLogonSessionFromToken(
//     _Inout_ PTOKEN Token)
// {
//     PSEP_LOGON_SESSION_REFERENCES LogonSession;
//     PAGED_CODE();
//
//     /* Ensure that our token is not some plain garbage */
//     ASSERT(Token);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     for (LogonSession = SepLogonSessions;
//          LogonSession != NULL;
//          LogonSession = LogonSession->Next)
//     {
//         /*
//          * Remove the logon session only when the IDs of the token and the
//          * logon match.
//          */
//         if (RtlEqualLuid(&LogonSession->LogonId, &Token->AuthenticationId))
//         {
//             break;
//         }
//     }
//
//     /* They don't match */
//     if (LogonSession == NULL)
//     {
//         DPRINT1("SepRmRemoveLogonSessionFromToken(): Couldn't remove the logon session from the access token!\n");
//         KeReleaseGuardedMutex(&SepRmDbLock);
//         return STATUS_NO_SUCH_LOGON_SESSION;
//     }
//
//     /* Now it's time to delete the logon session from the token */
//     RemoveEntryList(&Token->LogonSession->TokenList);
//     ExFreePoolWithTag(Token->LogonSession, TAG_LOGON_SESSION);
//
//     /* Release the database lock and we're done */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Creates a logon session. The security reference monitoring (SRM)
//  * module of Executive uses this as an internal kernel data for
//  * respective logon sessions management within the kernel,
//  * as in form of a SEP_LOGON_SESSION_REFERENCES data structure.
//  *
//  * @param[in] LogonLuid
//  * A logon ID represented as a LUID. This LUID is used to create
//  * our logon session and add it to the sessions database.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the logon has been created successfully.
//  * STATUS_LOGON_SESSION_EXISTS is returned if a logon session with
//  * the pointed logon ID in the call already exists.
//  * STATUS_INSUFFICIENT_RESOURCES is returned if logon session allocation
//  * has failed because of lack of memory pool resources.
//  */
// static
// NTSTATUS
// SepRmCreateLogonSession(
//     _In_ PLUID LogonLuid)
// {
//     PSEP_LOGON_SESSION_REFERENCES CurrentSession, NewSession;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("SepRmCreateLogonSession(%08lx:%08lx)\n",
//            LogonLuid->HighPart, LogonLuid->LowPart);
//
//     /* Allocate a new session structure */
//     NewSession = ExAllocatePoolWithTag(PagedPool,
//                                        sizeof(SEP_LOGON_SESSION_REFERENCES),
//                                        TAG_LOGON_SESSION);
//     if (NewSession == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Initialize it */
//     NewSession->LogonId = *LogonLuid;
//     NewSession->ReferenceCount = 0;
//     NewSession->Flags = 0;
//     NewSession->pDeviceMap = NULL;
//     InitializeListHead(&NewSession->TokenList);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop all existing sessions */
//     for (CurrentSession = SepLogonSessions;
//          CurrentSession != NULL;
//          CurrentSession = CurrentSession->Next)
//     {
//         /* Check if the LUID matches the new one */
//         if (RtlEqualLuid(&CurrentSession->LogonId, LogonLuid))
//         {
//             Status = STATUS_LOGON_SESSION_EXISTS;
//             goto Leave;
//         }
//     }
//
//     /* Insert the new session */
//     NewSession->Next = SepLogonSessions;
//     SepLogonSessions = NewSession;
//
//     Status = STATUS_SUCCESS;
//
// Leave:
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(NewSession, TAG_LOGON_SESSION);
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Deletes a logon session from the logon sessions database.
//  *
//  * @param[in] LogonLuid
//  * A logon ID represented as a LUID. This LUID is used to point
//  * the exact logon session saved within the database.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the logon session has been deleted successfully.
//  * STATUS_NO_SUCH_LOGON_SESSION is returned if the logon session with the submitted
//  * LUID doesn't exist. STATUS_BAD_LOGON_SESSION_STATE is returned if the logon session
//  * is still in use and we're not allowed to delete it, or if a system or anonymous session
//  * is submitted and we're not allowed to delete them as they're internal parts of the system.
//  * Otherwise a failure NTSTATUS code is returned.
//  */
// static
// NTSTATUS
// SepRmDeleteLogonSession(
//     _In_ PLUID LogonLuid)
// {
//     PSEP_LOGON_SESSION_REFERENCES SessionToDelete;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("SepRmDeleteLogonSession(%08lx:%08lx)\n",
//            LogonLuid->HighPart, LogonLuid->LowPart);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop over the existing logon sessions */
//     for (SessionToDelete = SepLogonSessions;
//          SessionToDelete != NULL;
//          SessionToDelete = SessionToDelete->Next)
//     {
//         /*
//          * Does the actual logon session exist in the
//          * saved logon sessions database with the LUID
//          * provided?
//          */
//         if (RtlEqualLuid(&SessionToDelete->LogonId, LogonLuid))
//         {
//             /* Did the caller supply one of these internal sessions? */
//             if (RtlEqualLuid(&SessionToDelete->LogonId, &SeSystemAuthenticationId) ||
//                 RtlEqualLuid(&SessionToDelete->LogonId, &SeAnonymousAuthenticationId))
//             {
//                 /* These logons are critical stuff, we can't delete them */
//                 DPRINT1("SepRmDeleteLogonSession(): We're not allowed to delete anonymous/system sessions!\n");
//                 Status = STATUS_BAD_LOGON_SESSION_STATE;
//                 goto Leave;
//             }
//             else
//             {
//                 /* We found the logon as exactly as we wanted, break the loop */
//                 break;
//             }
//         }
//     }
//
//     /*
//      * If we reach this then that means we've exhausted all the logon
//      * sessions and couldn't find one with the desired LUID.
//      */
//     if (SessionToDelete == NULL)
//     {
//         DPRINT1("SepRmDeleteLogonSession(): The logon session with this LUID doesn't exist!\n");
//         Status = STATUS_NO_SUCH_LOGON_SESSION;
//         goto Leave;
//     }
//
//     /* Is somebody still using this logon session? */
//     if (SessionToDelete->ReferenceCount != 0)
//     {
//         /* The logon session is still in use, we cannot delete it... */
//         DPRINT1("SepRmDeleteLogonSession(): The logon session is still in use!\n");
//         Status = STATUS_BAD_LOGON_SESSION_STATE;
//         goto Leave;
//     }
//
//     /* If we have a LUID device map, clean it */
//     if (SessionToDelete->pDeviceMap != NULL)
//     {
//         Status = SepCleanupLUIDDeviceMapDirectory(LogonLuid);
//         if (!NT_SUCCESS(Status))
//         {
//             /*
//              * We had one job on cleaning the device map directory
//              * of the logon session but we failed, quit...
//              */
//             DPRINT1("SepRmDeleteLogonSession(): Failed to clean the LUID device map directory of the logon (Status: 0x%lx)\n", Status);
//             goto Leave;
//         }
//
//         /* And dereference the device map of the logon */
//         ObfDereferenceDeviceMap(SessionToDelete->pDeviceMap);
//     }
//
//     /* If we're here then we've deleted the logon session successfully */
//     DPRINT("SepRmDeleteLogonSession(): Logon session deleted with success!\n");
//     Status = STATUS_SUCCESS;
//     ExFreePoolWithTag(SessionToDelete, TAG_LOGON_SESSION);
//
// Leave:
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//     return Status;
// }
//
// /**
//  * @brief
//  * References a logon session.
//  *
//  * @param[in] LogonLuid
//  * A valid LUID that points to the logon session in the database that
//  * we're going to reference it.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the logon has been referenced.
//  * STATUS_NO_SUCH_LOGON_SESSION is returned if the session couldn't be
//  * found otherwise.
//  */
// NTSTATUS
// SepRmReferenceLogonSession(
//     _In_ PLUID LogonLuid)
// {
//     PSEP_LOGON_SESSION_REFERENCES CurrentSession;
//
//     PAGED_CODE();
//
//     DPRINT("SepRmReferenceLogonSession(%08lx:%08lx)\n",
//            LogonLuid->HighPart, LogonLuid->LowPart);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop all existing sessions */
//     for (CurrentSession = SepLogonSessions;
//          CurrentSession != NULL;
//          CurrentSession = CurrentSession->Next)
//     {
//         /* Check if the LUID matches the new one */
//         if (RtlEqualLuid(&CurrentSession->LogonId, LogonLuid))
//         {
//             /* Reference the session */
//             ++CurrentSession->ReferenceCount;
//             DPRINT("ReferenceCount: %lu\n", CurrentSession->ReferenceCount);
//
//             /* Release the database lock */
//             KeReleaseGuardedMutex(&SepRmDbLock);
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     return STATUS_NO_SUCH_LOGON_SESSION;
// }
//
// /**
//  * @brief
//  * Cleans the DOS device map directory of a logon
//  * session.
//  *
//  * @param[in] LogonLuid
//  * A logon session ID where its DOS device map directory
//  * is to be cleaned.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the device map directory has been
//  * successfully cleaned from the logon session. STATUS_INVALID_PARAMETER
//  * is returned if the caller hasn't submitted any logon ID. STATUS_NO_MEMORY
//  * is returned if buffer allocation for links has failed. A failure
//  * NTSTATUS code is returned otherwise.
//  */
// static
// NTSTATUS
// SepCleanupLUIDDeviceMapDirectory(
//     _In_ PLUID LogonLuid)
// {
//     BOOLEAN UseCurrentProc;
//     KAPC_STATE ApcState;
//     WCHAR Buffer[63];
//     UNICODE_STRING DirectoryName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE DirectoryHandle, LinkHandle;
//     PHANDLE LinksBuffer;
//     POBJECT_DIRECTORY_INFORMATION DirectoryInfo;
//     ULONG LinksCount, LinksSize, DirInfoLength, ReturnLength, Context, CurrentLinks, i;
//     BOOLEAN RestartScan;
//
//     PAGED_CODE();
//
//     /* We need a logon LUID */
//     if (LogonLuid == NULL)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Use current process */
//     UseCurrentProc = ObReferenceObjectSafe(PsGetCurrentProcess());
//     if (UseCurrentProc)
//     {
//         ObDereferenceObject(PsGetCurrentProcess());
//     }
//     /* Unless it's gone, then use system process */
//     else
//     {
//         KeStackAttachProcess(&PsInitialSystemProcess->Pcb, &ApcState);
//     }
//
//     /* Initialize our directory name */
//     _snwprintf(Buffer,
//                sizeof(Buffer) / sizeof(WCHAR),
//                L"\\Sessions\\0\\DosDevices\\%08x-%08x",
//                LogonLuid->HighPart,
//                LogonLuid->LowPart);
//     RtlInitUnicodeString(&DirectoryName, Buffer);
//
//     /* And open it */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirectoryName,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenDirectoryObject(&DirectoryHandle,
//                                    DIRECTORY_QUERY,
//                                    &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         if (!UseCurrentProc)
//         {
//             KeUnstackDetachProcess(&ApcState);
//         }
//
//         return Status;
//     }
//
//     /* Some initialization needed for browsing all our links... */
//     Context = 0;
//     DirectoryInfo = NULL;
//     DirInfoLength = 0;
//     /* In our buffer, we'll store at max 100 HANDLE */
//     LinksCount = 100;
//     CurrentLinks = 0;
//     /* Which gives a certain size */
//     LinksSize = LinksCount * sizeof(HANDLE);
//
//     /*
//      * This label is hit if we need to store more than a hundred
//      * of links. In that case, we jump here after having cleaned
//      * and deleted previous buffer.
//      * All handles have been already closed
//      */
// AllocateLinksAgain:
//     LinksBuffer = ExAllocatePoolWithTag(PagedPool,
//                                         LinksSize,
//                                         TAG_SE_HANDLES_TAB);
//     if (LinksBuffer == NULL)
//     {
//         /*
//          * Failure path: no need to clear handles:
//          * already closed and the buffer is already gone
//          */
//         ZwClose(DirectoryHandle);
//
//         /*
//          * On the first round, DirectoryInfo is NULL,
//          * if we grow LinksBuffer, it has been allocated
//          */
//         if (DirectoryInfo != NULL)
//         {
//             ExFreePoolWithTag(DirectoryInfo, TAG_SE_DIR_BUFFER);
//         }
//
//         if (!UseCurrentProc)
//         {
//             KeUnstackDetachProcess(&ApcState);
//         }
//
//         return STATUS_NO_MEMORY;
//     }
//
//     /*
//      * We always restart scan, but on the first loop
//      * if we couldn't fit everything in our buffer,
//      * then, we continue scan.
//      * But we restart if link buffer was too small
//      */
//     for (RestartScan = TRUE; ; RestartScan = FALSE)
//     {
//         /*
//          * Loop until our buffer is big enough to store
//          * one entry
//          */
//         while (TRUE)
//         {
//             Status = ZwQueryDirectoryObject(DirectoryHandle,
//                                             DirectoryInfo,
//                                             DirInfoLength,
//                                             TRUE,
//                                             RestartScan,
//                                             &Context,
//                                             &ReturnLength);
//             /* Only handle buffer growth in that loop */
//             if (Status != STATUS_BUFFER_TOO_SMALL)
//             {
//                 break;
//             }
//
//             /* Get output length as new length */
//             DirInfoLength = ReturnLength;
//             /* Delete old buffer if any */
//             if (DirectoryInfo != NULL)
//             {
//                 ExFreePoolWithTag(DirectoryInfo, 'bDeS');
//             }
//
//             /* And reallocate a bigger one */
//             DirectoryInfo = ExAllocatePoolWithTag(PagedPool,
//                                                   DirInfoLength,
//                                                   TAG_SE_DIR_BUFFER);
//             /* Fail if we cannot allocate */
//             if (DirectoryInfo == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 break;
//             }
//         }
//
//         /* If querying the entry failed, quit */
//         if (!NT_SUCCESS(Status))
//         {
//             break;
//         }
//
//         /* We only look for symbolic links, the rest, we ignore */
//         if (wcscmp(DirectoryInfo->TypeName.Buffer, L"SymbolicLink"))
//         {
//             continue;
//         }
//
//         /* If our link buffer is out of space, reallocate */
//         if (CurrentLinks >= LinksCount)
//         {
//             /* First, close the links */
//             for (i = 0; i < CurrentLinks; ++i)
//             {
//                 ZwClose(LinksBuffer[i]);
//             }
//
//             /* Allow 20 more HANDLEs */
//             LinksCount += 20;
//             CurrentLinks = 0;
//             ExFreePoolWithTag(LinksBuffer, TAG_SE_HANDLES_TAB);
//             LinksSize = LinksCount * sizeof(HANDLE);
//
//             /* And reloop again */
//             goto AllocateLinksAgain;
//         }
//
//         /* Open the found link */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DirectoryInfo->Name,
//                                    OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                    DirectoryHandle,
//                                    NULL);
//         if (NT_SUCCESS(ZwOpenSymbolicLinkObject(&LinkHandle,
//                                                 SYMBOLIC_LINK_ALL_ACCESS,
//                                                 &ObjectAttributes)))
//         {
//             /* If we cannot make it temporary, just close the link handle */
//             if (!NT_SUCCESS(ZwMakeTemporaryObject(LinkHandle)))
//             {
//                 ZwClose(LinkHandle);
//             }
//             /* Otherwise, store it to defer deletion */
//             else
//             {
//                 LinksBuffer[CurrentLinks] = LinkHandle;
//                 ++CurrentLinks;
//             }
//         }
//     }
//
//     /* No more entries means we handled all links, that's not a failure */
//     if (Status == STATUS_NO_MORE_ENTRIES)
//     {
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Close all the links we stored, this will like cause their deletion */
//     for (i = 0; i < CurrentLinks; ++i)
//     {
//         ZwClose(LinksBuffer[i]);
//     }
//     /* And free our links buffer */
//     ExFreePoolWithTag(LinksBuffer, TAG_SE_HANDLES_TAB);
//
//     /* Free our directory info buffer - it might be NULL if we failed realloc */
//     if (DirectoryInfo != NULL)
//     {
//         ExFreePoolWithTag(DirectoryInfo, TAG_SE_DIR_BUFFER);
//     }
//
//     /* Close our session directory */
//     ZwClose(DirectoryHandle);
//
//     /* And detach from system */
//     if (!UseCurrentProc)
//     {
//         KeUnstackDetachProcess(&ApcState);
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * De-references a logon session. If the session has a reference
//  * count of 0 by the time the function has de-referenced the logon,
//  * that means the session is no longer used and can be safely deleted
//  * from the logon sessions database.
//  *
//  * @param[in] LogonLuid
//  * A logon session ID to de-reference.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the logon session has been de-referenced
//  * without issues. STATUS_NO_SUCH_LOGON_SESSION is returned if no
//  * such logon exists otherwise.
//  */
// NTSTATUS
// SepRmDereferenceLogonSession(
//     _In_ PLUID LogonLuid)
// {
//     ULONG RefCount;
//     PDEVICE_MAP DeviceMap;
//     PSEP_LOGON_SESSION_REFERENCES CurrentSession;
//
//     DPRINT("SepRmDereferenceLogonSession(%08lx:%08lx)\n",
//            LogonLuid->HighPart, LogonLuid->LowPart);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop all existing sessions */
//     for (CurrentSession = SepLogonSessions;
//          CurrentSession != NULL;
//          CurrentSession = CurrentSession->Next)
//     {
//         /* Check if the LUID matches the new one */
//         if (RtlEqualLuid(&CurrentSession->LogonId, LogonLuid))
//         {
//             /* Dereference the session */
//             RefCount = --CurrentSession->ReferenceCount;
//             DPRINT("ReferenceCount: %lu\n", CurrentSession->ReferenceCount);
//
//             /* Release the database lock */
//             KeReleaseGuardedMutex(&SepRmDbLock);
//
//             /* We're done with the session */
//             if (RefCount == 0)
//             {
//                 /* Get rid of the LUID device map */
//                 DeviceMap = CurrentSession->pDeviceMap;
//                 if (DeviceMap != NULL)
//                 {
//                     CurrentSession->pDeviceMap = NULL;
//                     SepCleanupLUIDDeviceMapDirectory(LogonLuid);
//                     ObfDereferenceDeviceMap(DeviceMap);
//                 }
//
//                 /* FIXME: Alert LSA and filesystems that a logon is about to be deleted */
//             }
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     return STATUS_NO_SUCH_LOGON_SESSION;
// }
//
// /**
//  * @brief
//  * Main SRM server thread initialization function. It deals
//  * with security manager and LSASS port connection, thus
//  * thereby allowing communication between the kernel side
//  * (the SRM) and user mode side (the LSASS) of the security
//  * world of the operating system.
//  *
//  * @return
//  * Returns TRUE if command server connection between SRM
//  * and LSASS has succeeded, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SepRmCommandServerThreadInit(VOID)
// {
//     SECURITY_QUALITY_OF_SERVICE SecurityQos;
//     SEP_RM_API_MESSAGE Message;
//     UNICODE_STRING PortName;
//     REMOTE_PORT_VIEW RemotePortView;
//     PORT_VIEW PortView;
//     LARGE_INTEGER SectionSize;
//     HANDLE SectionHandle;
//     HANDLE PortHandle;
//     NTSTATUS Status;
//     BOOLEAN Result;
//
//     SectionHandle = NULL;
//     PortHandle = NULL;
//
//     /* Assume success */
//     Result = TRUE;
//
//     /* Wait until LSASS is ready */
//     Status = ZwWaitForSingleObject(SeLsaInitEvent, FALSE, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security Rm Init: Waiting for LSA Init Event failed 0x%lx\n", Status);
//         goto Cleanup;
//     }
//
//     /* We don't need this event anymore */
//     ObCloseHandle(SeLsaInitEvent, KernelMode);
//
//     /* Initialize the connection message */
//     Message.Header.u1.s1.TotalLength = sizeof(Message);
//     Message.Header.u1.s1.DataLength = 0;
//
//     /* Only LSASS can connect, so handle the connection right now */
//     Status = ZwListenPort(SeRmCommandPort, &Message.Header);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security Rm Init: Listen to Command Port failed 0x%lx\n", Status);
//         goto Cleanup;
//     }
//
//     /* Set the Port View structure length */
//     RemotePortView.Length = sizeof(RemotePortView);
//
//     /* Accept the connection */
//     Status = ZwAcceptConnectPort(&SepRmCommandMessagePort,
//                                  NULL,
//                                  &Message.Header,
//                                  TRUE,
//                                  NULL,
//                                  &RemotePortView);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security Rm Init: Accept Connect to Command Port failed 0x%lx\n", Status);
//         goto Cleanup;
//     }
//
//     /* Complete the connection */
//     Status = ZwCompleteConnectPort(SepRmCommandMessagePort);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security Rm Init: Complete Connect to Command Port failed 0x%lx\n", Status);
//         goto Cleanup;
//     }
//
//     /* Create a section for messages */
//     SectionSize.QuadPart = PAGE_SIZE;
//     Status = ZwCreateSection(&SectionHandle,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &SectionSize,
//                              PAGE_READWRITE,
//                              SEC_COMMIT,
//                              NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security Rm Init: Create Memory Section for LSA port failed: 0x%lx\n", Status);
//         goto Cleanup;
//     }
//
//     /* Setup the PORT_VIEW structure */
//     PortView.Length = sizeof(PortView);
//     PortView.SectionHandle = SectionHandle;
//     PortView.SectionOffset = 0;
//     PortView.ViewSize = SectionSize.LowPart;
//     PortView.ViewBase = NULL;
//     PortView.ViewRemoteBase = NULL;
//
//     /* Setup security QOS */
//     SecurityQos.Length = sizeof(SecurityQos);
//     SecurityQos.ImpersonationLevel = SecurityImpersonation;
//     SecurityQos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//     SecurityQos.EffectiveOnly = TRUE;
//
//     /* Connect to LSASS */
//     RtlInitUnicodeString(&PortName, L"\\SeLsaCommandPort");
//     Status = ZwConnectPort(&PortHandle,
//                            &PortName,
//                            &SecurityQos,
//                            &PortView,
//                            NULL,
//                            0,
//                            0,
//                            0);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Security Rm Init: Connect to LSA Port failed 0x%lx\n", Status);
//         goto Cleanup;
//     }
//
//     /* Remember section base and view offset */
//     SepCommandPortViewBase = PortView.ViewBase;
//     SepCommandPortViewRemoteBase = PortView.ViewRemoteBase;
//     SepCommandPortViewBaseOffset = (ULONG_PTR)SepCommandPortViewRemoteBase -
//                                    (ULONG_PTR)SepCommandPortViewBase;
//
//     DPRINT("SepRmCommandServerThreadInit: done\n");
//
// Cleanup:
//     /* Check for failure */
//     if (!NT_SUCCESS(Status))
//     {
//         if (PortHandle != NULL)
//         {
//             ObCloseHandle(PortHandle, KernelMode);
//         }
//
//         Result = FALSE;
//     }
//
//     /* Did we create a section? */
//     if (SectionHandle != NULL)
//     {
//         ObCloseHandle(SectionHandle, KernelMode);
//     }
//
//     return Result;
// }
//
// /**
//  * @brief
//  * Manages the SRM server API commands, that is, receiving such API
//  * command messages from the user mode side of the security standpoint,
//  * the LSASS.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepRmCommandServerThread(
//     _In_ PVOID StartContext)
// {
//     SEP_RM_API_MESSAGE Message;
//     PPORT_MESSAGE ReplyMessage;
//     HANDLE DummyPortHandle;
//     NTSTATUS Status;
//
//     /* Initialize the server thread */
//     if (!SepRmCommandServerThreadInit())
//     {
//         DPRINT1("Security: Terminating Rm Command Server Thread\n");
//         return;
//     }
//
//     /* No reply yet */
//     ReplyMessage = NULL;
//
//     /* Start looping */
//     while (TRUE)
//     {
//         /* Wait for a message */
//         Status = ZwReplyWaitReceivePort(SepRmCommandMessagePort,
//                                         NULL,
//                                         ReplyMessage,
//                                         &Message.Header);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to get message: 0x%lx\n", Status);
//             ReplyMessage = NULL;
//             continue;
//         }
//
//         /* Check if this is a connection request */
//         if (Message.Header.u2.s2.Type == LPC_CONNECTION_REQUEST)
//         {
//             /* Reject connection request */
//             ZwAcceptConnectPort(&DummyPortHandle,
//                                 NULL,
//                                 &Message.Header,
//                                 FALSE,
//                                 NULL,
//                                 NULL);
//
//             /* Start over */
//             ReplyMessage = NULL;
//             continue;
//         }
//
//         /* Check if the port died */
//         if ((Message.Header.u2.s2.Type == LPC_PORT_CLOSED) ||
//             (Message.Header.u2.s2.Type == LPC_CLIENT_DIED))
//         {
//             /* LSASS is dead, so let's quit as well */
//             break;
//         }
//
//         /* Check if this is an actual request */
//         if (Message.Header.u2.s2.Type != LPC_REQUEST)
//         {
//             DPRINT1("SepRmCommandServerThread: unexpected message type: 0x%x\n",
//                     Message.Header.u2.s2.Type);
//
//             /* Restart without replying */
//             ReplyMessage = NULL;
//             continue;
//         }
//
//         ReplyMessage = &Message.Header;
//
//         switch (Message.ApiNumber)
//         {
//             case RmAuditSetCommand:
//                 Status = SepRmSetAuditEvent(&Message);
//                 break;
//
//             case RmCreateLogonSession:
//                 Status = SepRmCreateLogonSession(&Message.u.LogonLuid);
//                 break;
//
//             case RmDeleteLogonSession:
//                 Status = SepRmDeleteLogonSession(&Message.u.LogonLuid);
//                 break;
//
//             default:
//                 DPRINT1("SepRmDispatchRequest: invalid API number: 0x%lx\n",
//                         Message.ApiNumber);
//                 ReplyMessage = NULL;
//         }
//
//         Message.u.ResultStatus = Status;
//     }
//
//     /* Close the port handles */
//     ObCloseHandle(SepRmCommandMessagePort, KernelMode);
//     ObCloseHandle(SeRmCommandPort, KernelMode);
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @brief
//  * Retrieves the DOS device map from a logon session.
//  *
//  * @param[in] LogonId
//  * A valid logon session ID.
//  *
//  * @param[out] DeviceMap
//  * The returned device map buffer from the logon session.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the device map could be gathered
//  * from the logon session. STATUS_INVALID_PARAMETER is returned if
//  * one of the parameters aren't initialized (that is, the caller has
//  * submitted a NULL pointer variable). STATUS_NO_SUCH_LOGON_SESSION is
//  * returned if no such session could be found. A failure NTSTATUS code
//  * is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeGetLogonIdDeviceMap(
//     _In_ PLUID LogonId,
//     _Out_ PDEVICE_MAP *DeviceMap)
// {
//     NTSTATUS Status;
//     WCHAR Buffer[63];
//     PDEVICE_MAP LocalMap;
//     HANDLE DirectoryHandle, LinkHandle;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PSEP_LOGON_SESSION_REFERENCES CurrentSession;
//     UNICODE_STRING DirectoryName, LinkName, TargetName;
//
//     PAGED_CODE();
//
//     if  (LogonId == NULL ||
//          DeviceMap == NULL)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop all existing sessions */
//     for (CurrentSession = SepLogonSessions;
//          CurrentSession != NULL;
//          CurrentSession = CurrentSession->Next)
//     {
//         /* Check if the LUID matches the provided one */
//         if (RtlEqualLuid(&CurrentSession->LogonId, LogonId))
//         {
//             break;
//         }
//     }
//
//     /* No session found, fail */
//     if (CurrentSession == NULL)
//     {
//         /* Release the database lock */
//         KeReleaseGuardedMutex(&SepRmDbLock);
//
//         return STATUS_NO_SUCH_LOGON_SESSION;
//     }
//
//     /* The found session has a device map, return it! */
//     if (CurrentSession->pDeviceMap != NULL)
//     {
//         *DeviceMap = CurrentSession->pDeviceMap;
//
//         /* Release the database lock */
//         KeReleaseGuardedMutex(&SepRmDbLock);
//
//         return STATUS_SUCCESS;
//     }
//
//     /* At that point, we'll setup a new device map for the session */
//     LocalMap = NULL;
//
//     /* Reference the session so that it doesn't go away */
//     CurrentSession->ReferenceCount += 1;
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     /* Create our object directory given the LUID */
//     _snwprintf(Buffer,
//                sizeof(Buffer) / sizeof(WCHAR),
//                L"\\Sessions\\0\\DosDevices\\%08x-%08x",
//                LogonId->HighPart,
//                LogonId->LowPart);
//     RtlInitUnicodeString(&DirectoryName, Buffer);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirectoryName,
//                                OBJ_KERNEL_HANDLE | OBJ_OPENIF | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwCreateDirectoryObject(&DirectoryHandle,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Create the associated device map */
//         Status = ObSetDirectoryDeviceMap(&LocalMap, DirectoryHandle);
//         if (NT_SUCCESS(Status))
//         {
//             /* Make Global point to \Global?? in the directory */
//             RtlInitUnicodeString(&LinkName, L"Global");
//             RtlInitUnicodeString(&TargetName, L"\\Global??");
//
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &LinkName,
//                                        OBJ_KERNEL_HANDLE | OBJ_OPENIF | OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                        DirectoryHandle,
//                                        NULL);
//             Status = ZwCreateSymbolicLinkObject(&LinkHandle,
//                                                 SYMBOLIC_LINK_ALL_ACCESS,
//                                                 &ObjectAttributes,
//                                                 &TargetName);
//             if (!NT_SUCCESS(Status))
//             {
//                 ObfDereferenceDeviceMap(LocalMap);
//             }
//             else
//             {
//                 ZwClose(LinkHandle);
//             }
//         }
//
//         ZwClose(DirectoryHandle);
//     }
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* If we succeed... */
//     if (NT_SUCCESS(Status))
//     {
//         /* The session now has a device map? We raced with someone else */
//         if (CurrentSession->pDeviceMap != NULL)
//         {
//             /* Give up on our new device map */
//             ObfDereferenceDeviceMap(LocalMap);
//         }
//         /* Otherwise use our newly allocated device map */
//         else
//         {
//             CurrentSession->pDeviceMap = LocalMap;
//         }
//
//         /* Return the device map */
//         *DeviceMap = CurrentSession->pDeviceMap;
//     }
//     /* Zero output */
//     else
//     {
//         *DeviceMap = NULL;
//     }
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     /* We're done with the session */
//     SepRmDereferenceLogonSession(&CurrentSession->LogonId);
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Marks a logon session for future termination, given its logon ID. This triggers
//  * a callout (the registered callback) when the logon is no longer used by anyone,
//  * that is, no token is still referencing the speciffied logon session.
//  *
//  * @param[in] LogonId
//  * The ID of the logon session.
//  *
//  * @return
//  * STATUS_SUCCESS if the logon session is marked for termination notification successfully,
//  * STATUS_NOT_FOUND if the logon session couldn't be found otherwise.
//  */
// NTSTATUS
// NTAPI
// SeMarkLogonSessionForTerminationNotification(
//     _In_ PLUID LogonId)
// {
//     PSEP_LOGON_SESSION_REFERENCES SessionToMark;
//     PAGED_CODE();
//
//     DPRINT("SeMarkLogonSessionForTerminationNotification(%08lx:%08lx)\n",
//            LogonId->HighPart, LogonId->LowPart);
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop over the existing logon sessions */
//     for (SessionToMark = SepLogonSessions;
//          SessionToMark != NULL;
//          SessionToMark = SessionToMark->Next)
//     {
//         /* Does the logon with the given ID exist? */
//         if (RtlEqualLuid(&SessionToMark->LogonId, LogonId))
//         {
//             /* We found it */
//             break;
//         }
//     }
//
//     /*
//      * We've exhausted all the remaining logon sessions and
//      * couldn't find one with the provided ID.
//      */
//     if (SessionToMark == NULL)
//     {
//         DPRINT1("SeMarkLogonSessionForTerminationNotification(): Logon session couldn't be found!\n");
//         KeReleaseGuardedMutex(&SepRmDbLock);
//         return STATUS_NOT_FOUND;
//     }
//
//     /* Mark the logon session for termination */
//     SessionToMark->Flags |= SEP_LOGON_SESSION_TERMINATION_NOTIFY;
//     DPRINT("SeMarkLogonSessionForTerminationNotification(): Logon session marked for termination with success!\n");
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Registers a callback that will be called once a logon session
//  * terminates.
//  *
//  * @param[in] CallbackRoutine
//  * Callback routine address.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the callback routine was registered
//  * successfully. STATUS_INVALID_PARAMETER is returned if the caller
//  * did not provide a callback routine. STATUS_INSUFFICIENT_RESOURCES
//  * is returned if the callback notification data couldn't be allocated
//  * because of lack of memory pool resources.
//  */
// NTSTATUS
// NTAPI
// SeRegisterLogonSessionTerminatedRoutine(
//     _In_ PSE_LOGON_SESSION_TERMINATED_ROUTINE CallbackRoutine)
// {
//     PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION Notification;
//     PAGED_CODE();
//
//     /* Fail, if we don not have a callback routine */
//     if (CallbackRoutine == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Allocate a new notification item */
//     Notification = ExAllocatePoolWithTag(PagedPool,
//                                          sizeof(SEP_LOGON_SESSION_TERMINATED_NOTIFICATION),
//                                          TAG_LOGON_NOTIFICATION);
//     if (Notification == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Set the callback routine */
//     Notification->CallbackRoutine = CallbackRoutine;
//
//     /* Insert the new notification item into the list */
//     Notification->Next = SepLogonNotifications;
//     SepLogonNotifications = Notification;
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Un-registers a callback routine, previously registered by
//  * SeRegisterLogonSessionTerminatedRoutine function.
//  *
//  * @param[in] CallbackRoutine
//  * Callback routine address to un-register.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the callback routine was un-registered
//  * successfully. STATUS_INVALID_PARAMETER is returned if the caller
//  * did not provide a callback routine. STATUS_NOT_FOUND is returned
//  * if the callback notification item couldn't be found.
//  */
// NTSTATUS
// NTAPI
// SeUnregisterLogonSessionTerminatedRoutine(
//     _In_ PSE_LOGON_SESSION_TERMINATED_ROUTINE CallbackRoutine)
// {
//     PSEP_LOGON_SESSION_TERMINATED_NOTIFICATION Current, Previous = NULL;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Fail, if we don not have a callback routine */
//     if (CallbackRoutine == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Acquire the database lock */
//     KeAcquireGuardedMutex(&SepRmDbLock);
//
//     /* Loop all registered notification items */
//     for (Current = SepLogonNotifications;
//          Current != NULL;
//          Current = Current->Next)
//     {
//         /* Check if the callback routine matches the provided one */
//         if (Current->CallbackRoutine == CallbackRoutine)
//             break;
//
//         Previous = Current;
//     }
//
//     if (Current == NULL)
//     {
//         Status = STATUS_NOT_FOUND;
//     }
//     else
//     {
//         /* Remove the current notification item from the list */
//         if (Previous == NULL)
//             SepLogonNotifications = Current->Next;
//         else
//             Previous->Next = Current->Next;
//
//         /* Free the current notification item */
//         ExFreePoolWithTag(Current,
//                           TAG_LOGON_NOTIFICATION);
//
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Release the database lock */
//     KeReleaseGuardedMutex(&SepRmDbLock);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
