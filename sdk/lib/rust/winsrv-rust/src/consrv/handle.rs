use winsrv_sys::nt::*;
use super::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::console::ConSrvReleaseConsole;
use super::console::ConSrvGetConsole;
use super::condrv::conoutput::ConDrvDeleteScreenBuffer;
use super::condrv::console::ConDrvValidateConsoleUnsafe;

// NTSTATUS constants not in winsrv-sys nt re-exports.
const STATUS_SHARING_VIOLATION: NTSTATUS = 0xC0000043u32 as NTSTATUS;


// Local re-def of `_CONSOLE_IO_HANDLE` (bindgen sees the C definition only
// as a forward decl). The opaque bindgen stub is shadowed in this module
// scope by the real layout below.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _CONSOLE_IO_HANDLE {
    pub Object: PCONSOLE_IO_OBJECT,
    pub Access: ULONG,
    pub ShareMode: ULONG,
    pub Inheritable: BOOLEAN,
}

pub type CONSOLE_IO_HANDLE = _CONSOLE_IO_HANDLE;
pub type PCONSOLE_IO_HANDLE = *mut _CONSOLE_IO_HANDLE;

/// Entry `i` of a process's console handle table.
///
/// `CONSOLE_PROCESS_DATA::HandleTable` comes from winsrv-sys, whose bindgen view of
/// `CONSOLE_IO_HANDLE` is an OPAQUE 1-byte stub. Indexing the field directly
/// (`HandleTable.add(i) as PCONSOLE_IO_HANDLE`) therefore stepped ONE BYTE per
/// entry, while the table was allocated with this module's real element size.
/// Entries 0..2 overlapped, and writing handle 1 overwrote handle 0's `Object`.
/// The first console that ever got handles crashed csrss (AV in
/// ConSrvCloseHandle, BUGCHECK 0xF4) on its close. Always index through here.
#[inline(always)]
pub(crate) unsafe fn handle_at(ProcessData: PCONSOLE_PROCESS_DATA, i: usize) -> PCONSOLE_IO_HANDLE {
    ((*ProcessData).HandleTable as PCONSOLE_IO_HANDLE).add(i)
}

pub(crate) unsafe fn AdjustHandleCounts(
    Handle: PCONSOLE_IO_HANDLE,
    Change: LONG,
) -> LONG {
    let Object = (*Handle).Object;

    if ((*Handle).Access & GENERIC_READ) != 0 {
        (*Object).AccessRead = ((*Object).AccessRead as LONG + Change) as LONG;
    }
    if ((*Handle).Access & GENERIC_WRITE) != 0 {
        (*Object).AccessWrite = ((*Object).AccessWrite as LONG + Change) as LONG;
    }
    if ((*Handle).ShareMode & FILE_SHARE_READ) == 0 {
        (*Object).ExclusiveRead = ((*Object).ExclusiveRead as LONG + Change) as LONG;
    }
    if ((*Handle).ShareMode & FILE_SHARE_WRITE) == 0 {
        (*Object).ExclusiveWrite = ((*Object).ExclusiveWrite as LONG + Change) as LONG;
    }

    (*Object).ReferenceCount = ((*Object).ReferenceCount as LONG + Change) as LONG;

    (*Object).ReferenceCount as LONG
}

pub(crate) unsafe fn ConSrvCloseHandle(Handle: PCONSOLE_IO_HANDLE) {
    let Object = (*Handle).Object;
    if !Object.is_null() {
        if (*Object).Type == INPUT_BUFFER {
            let Console = (*Object).Console as PCONSRV_CONSOLE;

            CsrNotifyWait(
                core::ptr::addr_of_mut!((*Console).ReadWaitQueue),
                1u8,
                core::ptr::null_mut(),
                Handle as PVOID,
            );
            if IsListEmpty(core::ptr::addr_of_mut!((*Console).ReadWaitQueue) as PLIST_ENTRY) == 0 {
                CsrDereferenceWait(core::ptr::addr_of_mut!((*Console).ReadWaitQueue));
            }
        }

        if AdjustHandleCounts(Handle, -1) == 0 {
            if (*Object).Type == TEXTMODE_BUFFER || (*Object).Type == GRAPHICS_BUFFER {
                let Buffer = Object as PCONSOLE_SCREEN_BUFFER;
                if (*Buffer).ListEntry.Flink != (*Buffer).ListEntry.Blink {
                    ConDrvDeleteScreenBuffer(Buffer);
                }
            }
        }
    }
    core::ptr::write_bytes(Handle as *mut u8, 0, core::mem::size_of::<CONSOLE_IO_HANDLE>());
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvInheritHandlesTable(
    SourceProcessData: PCONSOLE_PROCESS_DATA,
    TargetProcessData: PCONSOLE_PROCESS_DATA,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*SourceProcessData).HandleTableLock));

    if !(*TargetProcessData).HandleTable.is_null() {
        Status = STATUS_UNSUCCESSFUL;
    } else {
        (*TargetProcessData).HandleTable = ConsoleAllocHeap(
            HEAP_ZERO_MEMORY,
            (*SourceProcessData).HandleTableSize as usize * core::mem::size_of::<CONSOLE_IO_HANDLE>(),
        ) as *mut _;

        if (*TargetProcessData).HandleTable.is_null() {
            Status = STATUS_NO_MEMORY;
        } else {
            (*TargetProcessData).HandleTableSize = (*SourceProcessData).HandleTableSize;

            let mut j = 0;
            for i in 0..(*SourceProcessData).HandleTableSize {
                if !(*(handle_at(SourceProcessData, i as usize))).Object.is_null()
                    && (*(handle_at(SourceProcessData, i as usize))).Inheritable != 0
                {
                    *(handle_at(TargetProcessData, j as usize)) =
                        *(handle_at(SourceProcessData, i as usize));
                    AdjustHandleCounts(handle_at(TargetProcessData, j as usize), 1);
                    j += 1;
                }
            }
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*SourceProcessData).HandleTableLock));
    Status
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvFreeHandlesTable(ProcessData: PCONSOLE_PROCESS_DATA) {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if !(*ProcessData).HandleTable.is_null() {
        if !(*ProcessData).ConsoleHandle.is_null() {
            for i in 0..(*ProcessData).HandleTableSize {
                ConSrvCloseHandle(handle_at(ProcessData, i as usize));
            }
        }
        ConsoleFreeHeap((*ProcessData).HandleTable as PVOID);
        (*ProcessData).HandleTable = core::ptr::null_mut();
    }

    (*ProcessData).HandleTableSize = 0;

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvInitObject(
    Object: PCONSOLE_IO_OBJECT,
    Type: CONSOLE_IO_OBJECT_TYPE,
    Console: PCONSOLE,
) {
    (*Object).Type = Type;
    (*Object).Console = Console;
    (*Object).ReferenceCount = 0;
    (*Object).AccessRead = 0;
    (*Object).AccessWrite = 0;
    (*Object).ExclusiveRead = 0;
    (*Object).ExclusiveWrite = 0;
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvInsertObject(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Handle: PHANDLE,
    Object: PCONSOLE_IO_OBJECT,
    Access: ULONG,
    Inheritable: BOOLEAN,
    ShareMode: ULONG,
) -> NTSTATUS {
    const IO_HANDLES_INCREMENT: usize = 2 * 3;

    let mut i = 0usize;

    if !(*ProcessData).HandleTable.is_null() {
        while i < (*ProcessData).HandleTableSize as usize {
            if (*(handle_at(ProcessData, i as usize))).Object.is_null() {
                break;
            }
            i += 1;
        }
    }

    if i >= (*ProcessData).HandleTableSize as usize {
        let Block = ConsoleAllocHeap(
            HEAP_ZERO_MEMORY,
            ((*ProcessData).HandleTableSize as usize + IO_HANDLES_INCREMENT) * core::mem::size_of::<CONSOLE_IO_HANDLE>(),
        ) as *mut CONSOLE_IO_HANDLE;

        if Block.is_null() {
            return STATUS_UNSUCCESSFUL;
        }

        if !(*ProcessData).HandleTable.is_null() {
            RtlCopyMemory(
                Block as PVOID,
                (*ProcessData).HandleTable as PVOID,
                (*ProcessData).HandleTableSize as usize * core::mem::size_of::<CONSOLE_IO_HANDLE>(),
            );
            ConsoleFreeHeap((*ProcessData).HandleTable as PVOID);
        }
        (*ProcessData).HandleTable = Block as *mut _;
        (*ProcessData).HandleTableSize += IO_HANDLES_INCREMENT as u32;
    }

    (*(handle_at(ProcessData, i as usize))).Object = Object;
    (*(handle_at(ProcessData, i as usize))).Access = Access;
    (*(handle_at(ProcessData, i as usize))).Inheritable = Inheritable;
    (*(handle_at(ProcessData, i as usize))).ShareMode = ShareMode;
    AdjustHandleCounts(handle_at(ProcessData, i as usize), 1);
    *Handle = ULongToHandle((i as u32) << 2 | 0x3);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvRemoveObject(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Handle: HANDLE,
) -> NTSTATUS {
    let Index = HandleToULong(Handle) >> 2;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if Index >= (*ProcessData).HandleTableSize
        || (*(handle_at(ProcessData, Index as usize))).Object.is_null()
    {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
        return STATUS_INVALID_HANDLE;
    }

    ConSrvCloseHandle(handle_at(ProcessData, Index as usize));

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvGetObject(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Handle: HANDLE,
    Object: *mut PCONSOLE_IO_OBJECT,
    Entry: *mut PVOID,
    Access: ULONG,
    LockConsole: BOOLEAN,
    Type: CONSOLE_IO_OBJECT_TYPE,
) -> NTSTATUS {
    let Index = HandleToULong(Handle) >> 2;
    let mut HandleEntry: PCONSOLE_IO_HANDLE = core::ptr::null_mut();
    let mut ObjectEntry: PCONSOLE_IO_OBJECT = core::ptr::null_mut();

    if !Entry.is_null() {
        *Entry = core::ptr::null_mut();
    }

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if IsConsoleHandle(Handle) && Index < (*ProcessData).HandleTableSize {
        HandleEntry = handle_at(ProcessData, Index as usize);
        ObjectEntry = (*HandleEntry).Object;
    }

    if HandleEntry.is_null()
        || ObjectEntry.is_null()
        || ((*HandleEntry).Access & Access) == 0
        || (Type != 0 && ((*ObjectEntry).Type & Type) == 0)
    {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
        return STATUS_INVALID_HANDLE;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if ConDrvValidateConsoleUnsafe((*ObjectEntry).Console, CONSOLE_RUNNING, LockConsole) != 0 {
        core::sync::atomic::AtomicI32::from_ptr(
            core::ptr::addr_of_mut!((*(*ObjectEntry).Console).ReferenceCount) as *mut i32,
        )
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        *Object = ObjectEntry;
        if !Entry.is_null() {
            *Entry = HandleEntry as PVOID;
        }

        STATUS_SUCCESS
    } else {
        STATUS_INVALID_HANDLE
    }
}

#[no_mangle]
pub unsafe extern "C" fn ConSrvReleaseObject(
    Object: PCONSOLE_IO_OBJECT,
    IsConsoleLocked: BOOLEAN,
) {
    let ObjectConsole = (*Object).Console as PCONSRV_CONSOLE;
    ConSrvReleaseConsole(ObjectConsole, IsConsoleLocked);
}

#[no_mangle]
pub unsafe extern "system" fn SrvOpenConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let OpenConsoleRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.OpenConsoleRequest;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let _gs = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(_gs) { return _gs; }

    let DesiredAccess = OpenConsoleRequest.DesiredAccess;
    let ShareMode = OpenConsoleRequest.ShareMode;
    let Object: PCONSOLE_IO_OBJECT;

    OpenConsoleRequest.Handle = INVALID_HANDLE_VALUE;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if OpenConsoleRequest.HandleType == HANDLE_OUTPUT {
        Object = core::ptr::addr_of_mut!((*(*Console).ActiveBuffer).Header);
    } else {
        Object = core::ptr::addr_of_mut!((*Console).InputBuffer.Header);
    }

    let Status = if ((DesiredAccess & GENERIC_READ) != 0 && (*Object).ExclusiveRead != 0)
        || ((DesiredAccess & GENERIC_WRITE) != 0 && (*Object).ExclusiveWrite != 0)
        || ((ShareMode & FILE_SHARE_READ) == 0 && (*Object).AccessRead != 0)
        || ((ShareMode & FILE_SHARE_WRITE) == 0 && (*Object).AccessWrite != 0)
    {
        STATUS_SHARING_VIOLATION
    } else {
        ConSrvInsertObject(
            ProcessData,
            core::ptr::addr_of_mut!(OpenConsoleRequest.Handle),
            Object,
            DesiredAccess,
            OpenConsoleRequest.InheritHandle as u8,
            ShareMode,
        )
    };

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
    ConSrvReleaseConsole(Console, 1u8);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvDuplicateHandle(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let DuplicateHandleRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.DuplicateHandleRequest;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    let SourceHandle = DuplicateHandleRequest.SourceHandle;
    let Index = HandleToULong(SourceHandle) >> 2;

    DuplicateHandleRequest.TargetHandle = INVALID_HANDLE_VALUE;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    let Status;
    if Index >= (*ProcessData).HandleTableSize
        || (*(handle_at(ProcessData, Index as usize))).Object.is_null()
    {
        Status = STATUS_INVALID_HANDLE;
    } else {
        let Entry = handle_at(ProcessData, Index as usize);
        let DesiredAccess = if (DuplicateHandleRequest.Options & DUPLICATE_SAME_ACCESS) != 0 {
            (*Entry).Access
        } else {
            let DesiredAccess = DuplicateHandleRequest.DesiredAccess;
            if ((*Entry).Access & DesiredAccess) == 0 {
                Status = STATUS_INVALID_PARAMETER;
                RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
                return Status;
            }
            DesiredAccess
        };

        Status = ConSrvInsertObject(
            ProcessData,
            core::ptr::addr_of_mut!(DuplicateHandleRequest.TargetHandle),
            (*Entry).Object,
            DesiredAccess,
            DuplicateHandleRequest.InheritHandle,
            (*Entry).ShareMode,
        );
        if Status >= 0 && (DuplicateHandleRequest.Options & DUPLICATE_CLOSE_SOURCE) != 0 {
            ConSrvCloseHandle(Entry);
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetHandleInformation(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let GetHandleInfoRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.GetHandleInfoRequest;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    let Handle = GetHandleInfoRequest.Handle;
    let Index = HandleToULong(Handle) >> 2;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    let Status = if !IsConsoleHandle(Handle)
        || Index >= (*ProcessData).HandleTableSize
        || (*(handle_at(ProcessData, Index as usize))).Object.is_null()
    {
        STATUS_INVALID_HANDLE
    } else {
        let Entry = handle_at(ProcessData, Index as usize);
        GetHandleInfoRequest.Flags = 0;
        if (*Entry).Inheritable != 0 {
            GetHandleInfoRequest.Flags |= HANDLE_FLAG_INHERIT;
        }
        STATUS_SUCCESS
    };

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetHandleInformation(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetHandleInfoRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.SetHandleInfoRequest;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    let Handle = SetHandleInfoRequest.Handle;
    let Index = HandleToULong(Handle) >> 2;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    let Status = if !IsConsoleHandle(Handle)
        || Index >= (*ProcessData).HandleTableSize
        || (*(handle_at(ProcessData, Index as usize))).Object.is_null()
    {
        STATUS_INVALID_HANDLE
    } else {
        let Entry = handle_at(ProcessData, Index as usize);
        if (SetHandleInfoRequest.Mask & HANDLE_FLAG_INHERIT) != 0 {
            (*Entry).Inheritable =
                if (SetHandleInfoRequest.Flags & HANDLE_FLAG_INHERIT) != 0 { 1 } else { 0 };
        }
        STATUS_SUCCESS
    };

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvCloseHandle(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let CloseHandleRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.CloseHandleRequest;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    ConSrvRemoveObject(ProcessData, CloseHandleRequest.Handle)
}

#[no_mangle]
pub unsafe extern "system" fn SrvVerifyConsoleIoHandle(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let VerifyHandleRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.VerifyHandleRequest;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    let IoHandle = VerifyHandleRequest.Handle;
    let Index = HandleToULong(IoHandle) >> 2;

    VerifyHandleRequest.IsValid = 0;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if IsConsoleHandle(IoHandle)
        && Index < (*ProcessData).HandleTableSize
        && !(*(handle_at(ProcessData, Index as usize))).Object.is_null()
    {
        VerifyHandleRequest.IsValid = 1;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    STATUS_SUCCESS
}