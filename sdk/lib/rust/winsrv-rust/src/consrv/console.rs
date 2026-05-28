use winsrv_sys::nt::*;
use super::{wcslen, wcsrchr, wcsncpy, _wcsicmp, PrivateExtractIconsW};
use super::condrv::text::*;
use super::condrv::conoutput::*;
use super::condrv::coninput::*;
use super::condrv::console::*;
use super::handle::*;
use super::lineinput::*;
use super::history::*;
use super::init::*;
use super::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::handle::ConSrvFreeHandlesTable;
use super::handle::ConSrvInsertObject;
use super::condrv::console::ConDrvDeleteConsole;
use super::condrv::console::ConDrvPause;
use super::condrv::console::ConDrvUnpause;
use super::condrv::console::ConDrvValidateConsoleState;
use super::condrv::console::ConDrvValidateConsoleUnsafe;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub(crate) static mut ConsoleListSize: ULONG = 0;
pub(crate) static mut ConsoleList: *mut PCONSRV_CONSOLE = core::ptr::null_mut();
pub(crate) static mut ListLock: RTL_RESOURCE = unsafe { core::mem::zeroed() };

pub const CONSOLE_VALID_CONTROL_MODES: ULONG = ENABLE_EXTENDED_FLAGS | ENABLE_INSERT_MODE | ENABLE_QUICK_EDIT_MODE as u16;

// console.c file-scope macros — bindgen doesn't see file-static macros.
#[inline]
pub(crate) unsafe fn ConSrvLockConsoleListExclusive() -> BOOLEAN {
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(ListLock), 1)
}

#[inline]
pub(crate) unsafe fn ConSrvLockConsoleListShared() -> BOOLEAN {
    RtlAcquireResourceShared(core::ptr::addr_of_mut!(ListLock), 1)
}

#[inline]
pub(crate) unsafe fn ConSrvUnlockConsoleList() {
    RtlReleaseResource(core::ptr::addr_of_mut!(ListLock));
}

pub(crate) unsafe fn ConDrvInsertConsole(Console: PCONSOLE) -> NTSTATUS {
    ConSrvLockConsoleListExclusive();

    InsertTailList(
        core::ptr::addr_of_mut!(ConDrvConsoleList),
        core::ptr::addr_of_mut!((*Console).ListEntry)
    );

    _InterlockedExchange(
        core::ptr::addr_of_mut!((*Console).ConsoleID) as PLONG,
        CurrentConsoleID
    );
    _InterlockedIncrement(core::ptr::addr_of_mut!(CurrentConsoleID) as PLONG);

    ConSrvUnlockConsoleList();
    STATUS_SUCCESS
}

pub(crate) unsafe fn InsertConsole(
    Handle: PHANDLE,
    Console: PCONSRV_CONSOLE
) -> NTSTATUS {
    const CONSOLE_HANDLES_INCREMENT: ULONG = 2 * 3;

    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut i: ULONG = 0;
    let mut Block: *mut PCONSRV_CONSOLE;

    ConSrvLockConsoleListExclusive();

    if !ConsoleList.is_null() {
        while i < ConsoleListSize {
            if (*ConsoleList.offset(i as isize)).is_null() {
                break;
            }
            i += 1;
        }
    }

    if i >= ConsoleListSize {
        Block = ConsoleAllocHeap(
            HEAP_ZERO_MEMORY,
            (ConsoleListSize + CONSOLE_HANDLES_INCREMENT) as usize * core::mem::size_of::<PCONSRV_CONSOLE>()
        ) as *mut PCONSRV_CONSOLE;
        
        if Block.is_null() {
            Status = 0xC0000001u32;
            ConSrvUnlockConsoleList();
            return Status;
        }

        if !ConsoleList.is_null() {
            core::ptr::copy_nonoverlapping(
                ConsoleList,
                Block,
                ConsoleListSize as usize
            );
            ConsoleFreeHeap(ConsoleList as *mut core::ffi::c_void);
        }
        ConsoleList = Block;
        ConsoleListSize += CONSOLE_HANDLES_INCREMENT;
    }

    *ConsoleList.offset(i as isize) = Console;
    *Handle = ULongToHandle((i << 2) | 0x3);

    ConSrvUnlockConsoleList();
    Status
}

pub(crate) unsafe fn RemoveConsoleByHandle(Handle: HANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Console: PCONSRV_CONSOLE;

    let ValidHandle: u8 = if (HandleToULong(Handle) & 0x3) == 0x3 { 1 } else { 0 };
    let Index: ULONG = HandleToULong(Handle) >> 2;

    if ValidHandle == 0 {
        return STATUS_INVALID_HANDLE;
    }

    ConSrvLockConsoleListExclusive();

    'Quit: {
        if Index >= ConsoleListSize {
            Status = STATUS_INVALID_HANDLE;
            break 'Quit;
        }

        Console = *ConsoleList.offset(Index as isize);
        if Console.is_null() {
            Status = STATUS_INVALID_HANDLE;
            break 'Quit;
        }

        *ConsoleList.offset(Index as isize) = core::ptr::null_mut();
    }

    ConSrvUnlockConsoleList();
    Status
}

pub(crate) unsafe fn RemoveConsoleByPointer(Console: PCONSRV_CONSOLE) -> NTSTATUS {
    let mut i: ULONG = 0;

    if Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    ConSrvLockConsoleListExclusive();

    if !ConsoleList.is_null() {
        i = 0;
        while i < ConsoleListSize {
            if *ConsoleList.offset(i as isize) == Console {
                *ConsoleList.offset(i as isize) = core::ptr::null_mut();
            }
            i = i.wrapping_add(1);
        }
    }

    ConSrvUnlockConsoleList();
    STATUS_SUCCESS
}

pub(crate) unsafe fn RemoveConsole(Console: PCONSOLE) -> NTSTATUS {
    if Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    ConSrvLockConsoleListExclusive();

    RemoveEntryList(core::ptr::addr_of_mut!((*Console).ListEntry));

    ConSrvUnlockConsoleList();
    return STATUS_SUCCESS;
}

pub(crate) unsafe fn ConsoleCreateUnicodeString(
    UniDest: PUNICODE_STRING,
    Source: PCWSTR
) -> u8 {
    let Size = (wcslen(Source) + 1) * core::mem::size_of::<u16>();
    if Size > MAXUSHORT as usize {
        return 0u8;
    }

    (*UniDest).Buffer = ConsoleAllocHeap(HEAP_ZERO_MEMORY, Size) as *mut u16;
    if (*UniDest).Buffer == core::ptr::null_mut() {
        return 0u8;
    }

    RtlCopyMemory(
        (*UniDest).Buffer as *mut core::ffi::c_void,
        Source as *const core::ffi::c_void,
        Size
    );
    (*UniDest).MaximumLength = Size as u16;
    (*UniDest).Length = (Size - core::mem::size_of::<u16>()) as u16;

    1u8
}

pub(crate) unsafe fn ConsoleFreeUnicodeString(UnicodeString: PUNICODE_STRING) {
    if !(*UnicodeString).Buffer.is_null() {
        ConsoleFreeHeap((*UnicodeString).Buffer as *mut core::ffi::c_void);
        RtlZeroMemory(UnicodeString as *mut core::ffi::c_void, core::mem::size_of::<_UNICODE_STRING>());
    }
}

pub unsafe extern "system" fn ConSrvValidateConsole(
    Console: *mut PCONSRV_CONSOLE,
    ConsoleHandle: HANDLE,
    ExpectedState: CONSOLE_STATE,
    LockConsole: BOOLEAN,
) -> BOOLEAN {
    let mut RetVal: BOOLEAN = 0u8;
    let mut ValidatedConsole: PCONSRV_CONSOLE;

    let ValidHandle: BOOLEAN = if (HandleToULong(ConsoleHandle) & 0x3) == 0x3 { 1u8 } else { 0u8 };
    let Index: ULONG = HandleToULong(ConsoleHandle) >> 2;

    if ValidHandle == 0u8 {
        return 0u8;
    }

    if Console.is_null() {
        return 0u8;
    }
    *Console = core::ptr::null_mut();

    ConSrvLockConsoleListShared();

    if Index >= ConsoleListSize || {
        ValidatedConsole = *ConsoleList.offset(Index as isize);
        ValidatedConsole.is_null()
    } {
        ConSrvUnlockConsoleList();
        return 0u8;
    }

    ValidatedConsole = *ConsoleList.offset(Index as isize);

    ConSrvUnlockConsoleList();

    RetVal = ConDrvValidateConsoleUnsafe(
        ValidatedConsole as PCONSOLE,
        ExpectedState,
        LockConsole,
    );
    if RetVal != 0u8 {
        *Console = ValidatedConsole;
    }

    RetVal
}

pub unsafe extern "system" fn ConSrvGetConsole(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: *mut PCONSRV_CONSOLE,
    LockConsole: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INVALID_HANDLE;
    let mut GrabConsole: PCONSRV_CONSOLE = core::ptr::null_mut();

    *Console = core::ptr::null_mut();

    if ConSrvValidateConsole(
        core::ptr::addr_of_mut!(GrabConsole),
        (*ProcessData).ConsoleHandle,
        CONSOLE_RUNNING,
        LockConsole,
    ) != 0u8
    {
        _InterlockedIncrement(core::ptr::addr_of_mut!((*GrabConsole).ReferenceCount) as *mut i32);
        *Console = GrabConsole;
        Status = STATUS_SUCCESS;
    }

    Status
}

pub unsafe extern "system" fn ConSrvReleaseConsole(
    Console: PCONSRV_CONSOLE,
    IsConsoleLocked: u8,
) {
    let mut RefCount: i32 = 0;

    if Console.is_null() {
        return;
    }

    RefCount = _InterlockedDecrement(&mut (*Console).ReferenceCount);

    if IsConsoleLocked != 0 {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
    }

    if RefCount <= 0 {
        ConSrvDeleteConsole(Console);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvInitConsoleSupport() {
    ConsoleListSize = 0;
    ConsoleList = core::ptr::null_mut();
    RtlInitializeResource(core::ptr::addr_of_mut!(ListLock));
}

pub(crate) unsafe fn LoadShellLinkConsoleInfo(
    ConsoleInfo: PCONSOLE_STATE_INFO,
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
) -> u8 {
    const PATH_SEPARATOR: u16 = b'\\' as u16;

    let mut RetVal: u8 = 0u8;
    let mut hRes: HRESULT = 0i32;
    let mut Length: usize = 0;
    let mut LinkName: LPWSTR = core::ptr::null_mut();
    let mut IconPath: LPWSTR = core::ptr::null_mut();
    let mut Buffer: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];

    (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex = 0;

    if ((*(*ConsoleInitInfo).ConsoleStartInfo).dwStartupFlags & STARTF_TITLEISLINKNAME) == 0 {
        RetVal = 1u8;
        'Finish: loop {
            if RetVal != 0 {
                if IconPath == core::ptr::null_mut() {
                    Length = RtlDosSearchPath_U(
                        (*ConsoleInitInfo).CurDir,
                        (*ConsoleInitInfo).AppName,
                        core::ptr::null_mut(),
                        core::mem::size_of_val(&Buffer) as u32,
                        Buffer.as_mut_ptr(),
                        core::ptr::null_mut(),
                    );
                    if Length > 0 && (Length as usize) < core::mem::size_of_val(&Buffer) {
                        IconPath = Buffer.as_mut_ptr();
                    } else {
                        IconPath = (*ConsoleInitInfo).AppName;
                    }
                }

                if !IconPath.is_null() && *IconPath != 0 {
                    let mut hIcon: HICON = core::ptr::null_mut();
                    let mut hIconSm: HICON = core::ptr::null_mut();

                    PrivateExtractIconsW(
                        IconPath,
                        (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex,
                        32,
                        32,
                        &mut hIcon,
                        core::ptr::null_mut(),
                        1,
                        LR_COPYFROMRESOURCE,
                    );
                    PrivateExtractIconsW(
                        IconPath,
                        (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex,
                        16,
                        16,
                        &mut hIconSm,
                        core::ptr::null_mut(),
                        1,
                        LR_COPYFROMRESOURCE,
                    );

                    if hIcon != core::ptr::null_mut() {
                        (*(*ConsoleInitInfo).ConsoleStartInfo).hIcon = hIcon;
                    }
                    if hIconSm != core::ptr::null_mut() {
                        (*(*ConsoleInitInfo).ConsoleStartInfo).hIconSm = hIconSm;
                    }
                }
            }

            RetVal = 0u8;
            break;
        }
        return RetVal;
    }

    LinkName = wcsrchr((*ConsoleInfo).ConsoleTitle.as_mut_ptr(), PATH_SEPARATOR as i32);
    if LinkName == core::ptr::null_mut() {
        LinkName = (*ConsoleInfo).ConsoleTitle.as_mut_ptr();
    } else {
        LinkName = LinkName.offset(1);
    }

    Length = wcslen(LinkName);
    if Length <= 4 || _wcsicmp(LinkName.offset((Length - 4) as isize), b".\0l\0n\0k\0\0".as_ptr() as *const u16) != 0 {
        return 0u8;
    }

    hRes = CoInitialize(core::ptr::null_mut());
    if SUCCEEDED(hRes) {
        let mut pshl: LPVOID = core::ptr::null_mut();
        hRes = CoCreateInstance(
            &CLSID_ShellLink,
            core::ptr::null_mut(),
            1,
            &IID_IShellLinkW,
            &mut pshl,
        );
        if SUCCEEDED(hRes) {
            let mut ppf: LPVOID = core::ptr::null_mut();
            hRes = IPersistFile_QueryInterface(pshl, &IID_IPersistFile, &mut ppf);
            if SUCCEEDED(hRes) {
                hRes = IPersistFile_Load(ppf, (*ConsoleInfo).ConsoleTitle.as_mut_ptr(), 0);
                if SUCCEEDED(hRes) {
                    let mut ShowCmd: i32 = 0;

                    Length = min(
                        Length - 4,
                        ((*ConsoleInfo).cbSize as usize - FIELD_OFFSET(_CONSOLE_STATE_INFO, ConsoleTitle) - core::mem::size_of::<u16>()) / core::mem::size_of::<u16>(),
                    );
                    wcsncpy((*ConsoleInfo).ConsoleTitle.as_mut_ptr(), LinkName, Length);
                    (*ConsoleInfo).ConsoleTitle[Length] = 0;

                    hRes = IShellLinkW_GetShowCmd(pshl, &mut ShowCmd);
                    if SUCCEEDED(hRes) {
                        (*(*ConsoleInitInfo).ConsoleStartInfo).wShowWindow = ShowCmd as u16;
                    }

                    hRes = IShellLinkW_GetIconLocation(
                        pshl,
                        Buffer.as_mut_ptr(),
                        (core::mem::size_of_val(&Buffer) / core::mem::size_of::<u16>() - 1) as i32,
                        &mut (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex,
                    );
                    if !SUCCEEDED(hRes) {
                        (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex = 0;
                    } else {
                        IconPath = Buffer.as_mut_ptr();
                    }

                    RetVal = 1u8;
                }
                IPersistFile_Release(ppf);
            }
            IShellLinkW_Release(pshl);
        }
        CoUninitialize();
    }

    'Finish: loop {
        if RetVal != 0 {
            if IconPath == core::ptr::null_mut() {
                Length = RtlDosSearchPath_U(
                    (*ConsoleInitInfo).CurDir,
                    (*ConsoleInitInfo).AppName,
                    core::ptr::null_mut(),
                    core::mem::size_of_val(&Buffer) as u32,
                    Buffer.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
                if Length > 0 && (Length as usize) < core::mem::size_of_val(&Buffer) {
                    IconPath = Buffer.as_mut_ptr();
                } else {
                    IconPath = (*ConsoleInitInfo).AppName;
                }
            }

            if !IconPath.is_null() && *IconPath != 0 {
                let mut hIcon: HICON = core::ptr::null_mut();
                let mut hIconSm: HICON = core::ptr::null_mut();

                PrivateExtractIconsW(
                    IconPath,
                    (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex,
                    32,
                    32,
                    &mut hIcon,
                    core::ptr::null_mut(),
                    1,
                    LR_COPYFROMRESOURCE,
                );
                PrivateExtractIconsW(
                    IconPath,
                    (*(*ConsoleInitInfo).ConsoleStartInfo).IconIndex,
                    16,
                    16,
                    &mut hIconSm,
                    core::ptr::null_mut(),
                    1,
                    LR_COPYFROMRESOURCE,
                );

                if hIcon != core::ptr::null_mut() {
                    (*(*ConsoleInitInfo).ConsoleStartInfo).hIcon = hIcon;
                }
                if hIconSm != core::ptr::null_mut() {
                    (*(*ConsoleInitInfo).ConsoleStartInfo).hIconSm = hIconSm;
                }
            }
        }

        RetVal = 0u8;
        break;
    }

    RetVal
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvDeleteConsole(Console: PCONSRV_CONSOLE) {
    RemoveConsoleByPointer(Console);

    NtClose((*Console).InitEvents[INIT_FAILURE as usize]);
    NtClose((*Console).InitEvents[INIT_SUCCESS as usize]);

    if !(*Console).LineBuffer.is_null() {
        ConsoleFreeHeap((*Console).LineBuffer as *mut core::ffi::c_void);
    }

    IntDeleteAllAliases(Console);
    HistoryDeleteBuffers(Console);

    ConsoleFreeUnicodeString(core::ptr::addr_of_mut!((*Console).OriginalTitle));
    ConsoleFreeUnicodeString(core::ptr::addr_of_mut!((*Console).Title));

    ConDrvDeleteConsole(Console as PCONSOLE);
}

pub(crate) unsafe fn ConioPause(Console: PCONSRV_CONSOLE, Flags: u8) {
    (*Console).PauseFlags |= Flags;
    ConDrvPause(Console as PCONSOLE);
}

pub(crate) unsafe fn ConioUnpause(Console: PCONSRV_CONSOLE, Flags: u8) {
    (*Console).PauseFlags &= !Flags;

    if (*Console).PauseFlags == 0 {
        ConDrvUnpause(Console as PCONSOLE);

        CsrNotifyWait(
            core::ptr::addr_of_mut!((*Console).WriteWaitQueue),
            1u8,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if IsListEmpty(core::ptr::addr_of_mut!((*Console).WriteWaitQueue)) == 0 {
            CsrDereferenceWait(core::ptr::addr_of_mut!((*Console).WriteWaitQueue));
        }
    }
}

pub(crate) unsafe fn ConSrvInitProcessHandles(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    pInputHandle: PHANDLE,
    pOutputHandle: PHANDLE,
    pErrorHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut InputHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut OutputHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut ErrorHandle: HANDLE = INVALID_HANDLE_VALUE;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    Status = ConSrvInsertObject(
        ProcessData,
        core::ptr::addr_of_mut!(InputHandle),
        core::ptr::addr_of_mut!((*Console).InputBuffer.Header),
        GENERIC_READ | GENERIC_WRITE,
        1u8,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
    );
    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
        ConSrvFreeHandlesTable(ProcessData);
        return Status;
    }

    Status = ConSrvInsertObject(
        ProcessData,
        core::ptr::addr_of_mut!(OutputHandle),
        core::ptr::addr_of_mut!((*(*Console).ActiveBuffer).Header),
        GENERIC_READ | GENERIC_WRITE,
        1u8,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
    );
    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
        ConSrvFreeHandlesTable(ProcessData);
        return Status;
    }

    Status = ConSrvInsertObject(
        ProcessData,
        core::ptr::addr_of_mut!(ErrorHandle),
        core::ptr::addr_of_mut!((*(*Console).ActiveBuffer).Header),
        GENERIC_READ | GENERIC_WRITE,
        1u8,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
    );
    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));
        ConSrvFreeHandlesTable(ProcessData);
        return Status;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    *pInputHandle = InputHandle;
    *pOutputHandle = OutputHandle;
    *pErrorHandle = ErrorHandle;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvAllocateConsole(
    ProcessData: PCONSOLE_PROCESS_DATA,
    pInputHandle: PHANDLE,
    pOutputHandle: PHANDLE,
    pErrorHandle: PHANDLE,
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ConsoleHandle: HANDLE = core::ptr::null_mut();
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    ConSrvFreeHandlesTable(ProcessData);

    Status = ConSrvInitConsole(
        core::ptr::addr_of_mut!(ConsoleHandle),
        core::ptr::addr_of_mut!(Console),
        ConsoleInitInfo,
        (*ProcessData).Process,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*ProcessData).ConsoleHandle = ConsoleHandle;

    Status = ConSrvInitProcessHandles(
        ProcessData,
        Console,
        pInputHandle,
        pOutputHandle,
        pErrorHandle,
    );
    if !NT_SUCCESS(Status) {
        ConSrvDeleteConsole(Console);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        return Status;
    }

    Status = NtDuplicateObject(
        core::ptr::null_mut::<core::ffi::c_void>() as HANDLE,
        (*Console).InitEvents[INIT_SUCCESS as usize],
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::addr_of_mut!((*(*ConsoleInitInfo).ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize]),
        EVENT_ALL_ACCESS,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        ConSrvFreeHandlesTable(ProcessData);
        ConSrvDeleteConsole(Console);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        return Status;
    }

    Status = NtDuplicateObject(
        core::ptr::null_mut::<core::ffi::c_void>() as HANDLE,
        (*Console).InitEvents[INIT_FAILURE as usize],
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::addr_of_mut!((*(*ConsoleInitInfo).ConsoleStartInfo).InitEvents[INIT_FAILURE as usize]),
        EVENT_ALL_ACCESS,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        NtDuplicateObject(
            (*(*ProcessData).Process).ProcessHandle,
            (*(*ConsoleInitInfo).ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        ConSrvFreeHandlesTable(ProcessData);
        ConSrvDeleteConsole(Console);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        return Status;
    }

    Status = NtDuplicateObject(
        core::ptr::null_mut::<core::ffi::c_void>() as HANDLE,
        (*Console).InputBuffer.ActiveEvent,
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::addr_of_mut!((*(*ConsoleInitInfo).ConsoleStartInfo).InputWaitHandle),
        EVENT_ALL_ACCESS,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        NtDuplicateObject(
            (*(*ProcessData).Process).ProcessHandle,
            (*(*ConsoleInitInfo).ConsoleStartInfo).InitEvents[INIT_FAILURE as usize],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        NtDuplicateObject(
            (*(*ProcessData).Process).ProcessHandle,
            (*(*ConsoleInitInfo).ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        ConSrvFreeHandlesTable(ProcessData);
        ConSrvDeleteConsole(Console);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        return Status;
    }

    (*ProcessData).ConsoleApp = 1u8;
    (*(*ProcessData).Process).Flags |= CsrProcessIsConsoleApp;

    (*(*ConsoleInitInfo).ConsoleStartInfo).ConsoleHandle = (*ProcessData).ConsoleHandle;

    InsertHeadList(
        core::ptr::addr_of_mut!((*Console).ProcessList),
        core::ptr::addr_of_mut!((*ProcessData).ConsoleLink),
    );
    ConSrvSetProcessFocus((*ProcessData).Process, (*Console).HasFocus);

    _InterlockedIncrement(core::ptr::addr_of_mut!((*Console).ReferenceCount) as *mut i32);

    TermRefreshInternalInfo(Console);

    STATUS_SUCCESS
}

pub(crate) unsafe fn ConSrvInheritConsole(
    ProcessData: PCONSOLE_PROCESS_DATA,
    ConsoleHandle: HANDLE,
    CreateNewHandleTable: u8,
    pInputHandle: PHANDLE,
    pOutputHandle: PHANDLE,
    pErrorHandle: PHANDLE,
    ConsoleStartInfo: PCONSOLE_START_INFO,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    if ConSrvValidateConsole(
        core::ptr::addr_of_mut!(Console),
        ConsoleHandle,
        CONSOLE_RUNNING,
        1u8,
    ) == 0u8
    {
        return STATUS_UNSUCCESSFUL;
    }

    (*ProcessData).ConsoleHandle = ConsoleHandle;

    if CreateNewHandleTable != 0 {
        ConSrvFreeHandlesTable(ProcessData);

        Status = ConSrvInitProcessHandles(
            ProcessData,
            Console,
            pInputHandle,
            pOutputHandle,
            pErrorHandle,
        );
        if !NT_SUCCESS(Status) {
            (*ProcessData).ConsoleHandle = core::ptr::null_mut();
            LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
            return Status;
        }
    }

    Status = NtDuplicateObject(
        (-1isize) as HANDLE,
        (*Console).InitEvents[INIT_SUCCESS as usize],
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::addr_of_mut!((*ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize]),
        EVENT_ALL_ACCESS,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        ConSrvFreeHandlesTable(ProcessData);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        return Status;
    }

    Status = NtDuplicateObject(
        (-1isize) as HANDLE,
        (*Console).InitEvents[INIT_FAILURE as usize],
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::addr_of_mut!((*ConsoleStartInfo).InitEvents[INIT_FAILURE as usize]),
        EVENT_ALL_ACCESS,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        NtDuplicateObject(
            (*(*ProcessData).Process).ProcessHandle,
            (*ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        ConSrvFreeHandlesTable(ProcessData);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        return Status;
    }

    Status = NtDuplicateObject(
        (-1isize) as HANDLE,
        (*Console).InputBuffer.ActiveEvent,
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::addr_of_mut!((*ConsoleStartInfo).InputWaitHandle),
        EVENT_ALL_ACCESS,
        0,
        0,
    );
    if !NT_SUCCESS(Status) {
        NtDuplicateObject(
            (*(*ProcessData).Process).ProcessHandle,
            (*ConsoleStartInfo).InitEvents[INIT_FAILURE as usize],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        NtDuplicateObject(
            (*(*ProcessData).Process).ProcessHandle,
            (*ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize],
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            0,
            DUPLICATE_CLOSE_SOURCE,
        );
        ConSrvFreeHandlesTable(ProcessData);
        (*ProcessData).ConsoleHandle = core::ptr::null_mut();
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        return Status;
    }

    (*ProcessData).ConsoleApp = 1u8;
    (*(*ProcessData).Process).Flags |= CsrProcessIsConsoleApp;

    (*ConsoleStartInfo).ConsoleHandle = (*ProcessData).ConsoleHandle;

    InsertHeadList(
        core::ptr::addr_of_mut!((*Console).ProcessList),
        core::ptr::addr_of_mut!((*ProcessData).ConsoleLink),
    );
    ConSrvSetProcessFocus((*ProcessData).Process, (*Console).HasFocus);

    _InterlockedIncrement(core::ptr::addr_of_mut!((*Console).ReferenceCount) as *mut i32);

    TermRefreshInternalInfo(Console);

    Status = STATUS_SUCCESS;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
    return Status;
}

pub(crate) unsafe fn ConSrvRemoveConsole(
    ProcessData: PCONSOLE_PROCESS_DATA,
) -> NTSTATUS {
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let ConsoleLeaderProcess: PCONSOLE_PROCESS_DATA;

    (*ProcessData).ConsoleApp = 0u8;
    (*(*ProcessData).Process).Flags &= !CsrProcessIsConsoleApp;

    if ConSrvValidateConsole(
        core::ptr::addr_of_mut!(Console),
        (*ProcessData).ConsoleHandle,
        CONSOLE_RUNNING,
        1u8,
    ) == 0u8
    {
        return STATUS_UNSUCCESSFUL;
    }

    ConsoleLeaderProcess = ConSrvGetConsoleLeaderProcess(Console);

    ConSrvFreeHandlesTable(ProcessData);

    (*ProcessData).ConsoleHandle = core::ptr::null_mut();

    RemoveEntryList(core::ptr::addr_of_mut!((*ProcessData).ConsoleLink));

    if (*Console).NotifyLastClose != 0u8 {
        if ProcessData == (*Console).NotifiedLastCloseProcess {
            (*Console).NotifyLastClose = 0u8;
            (*Console).NotifiedLastCloseProcess = core::ptr::null_mut();
        } else if ProcessData == ConsoleLeaderProcess {
            (*Console).NotifyLastClose = 0u8;
            ConSrvConsoleCtrlEvent(CTRL_LAST_CLOSE_EVENT, (*Console).NotifiedLastCloseProcess);

            (*Console).NotifiedLastCloseProcess = core::ptr::null_mut();
        }
    }

    TermRefreshInternalInfo(Console);

    ConSrvReleaseConsole(Console, 1u8);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvConsoleCtrlEventTimeout(
    CtrlEvent: ULONG,
    ProcessData: PCONSOLE_PROCESS_DATA,
    Timeout: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*ProcessData).CtrlRoutine.is_null() {
        return Status;
    }

    let mut Thread: HANDLE = core::ptr::null_mut();

    Thread = CreateRemoteThread(
        (*(*ProcessData).Process).ProcessHandle,
        core::ptr::null_mut(),
        0,
        (*ProcessData).CtrlRoutine,
        UlongToPtr(CtrlEvent),
        0,
        core::ptr::null_mut(),
    );
    if Thread.is_null() {
        Status = RtlGetLastNtStatus();
    } else {
        WaitForSingleObject(Thread, Timeout);
    }

    CloseHandle(Thread);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvConsoleCtrlEvent(
    CtrlEvent: ULONG,
    ProcessData: PCONSOLE_PROCESS_DATA,
) -> NTSTATUS {
    ConSrvConsoleCtrlEventTimeout(CtrlEvent, ProcessData, 0)
}

pub unsafe extern "system" fn ConSrvGetConsoleLeaderProcess(
    Console: PCONSRV_CONSOLE,
) -> PCONSOLE_PROCESS_DATA {
    if Console.is_null() {
        return core::ptr::null_mut();
    }

    let blink = (*Console).ProcessList.Blink;
    let offset = core::mem::offset_of!(CONSOLE_PROCESS_DATA, ConsoleLink);
    (blink as *mut u8).offset(-(offset as isize)) as PCONSOLE_PROCESS_DATA
}

pub(crate) unsafe fn ConSrvGetConsoleProcessList(
    Console: PCONSRV_CONSOLE,
    ProcessIdsList: *mut ULONG,
    MaxIdListItems: ULONG,
    ProcessIdsTotal: *mut ULONG,
) -> NTSTATUS {
    let mut current: PCONSOLE_PROCESS_DATA;
    let mut current_entry: PLIST_ENTRY;

    if Console.is_null() || ProcessIdsList.is_null() || ProcessIdsTotal.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *ProcessIdsTotal = 0;

    current_entry = (*Console).ProcessList.Flink;
    while current_entry != core::ptr::addr_of_mut!((*Console).ProcessList) {
        current = (current_entry as *mut u8).offset(
            -(core::mem::offset_of!(CONSOLE_PROCESS_DATA, ConsoleLink) as isize)
        ) as PCONSOLE_PROCESS_DATA;
        
        *ProcessIdsTotal = (*ProcessIdsTotal).wrapping_add(1);
        if *ProcessIdsTotal <= MaxIdListItems {
            *ProcessIdsList = HandleToUlong((*(*current).Process).ClientId.UniqueProcess);
            ProcessIdsList = ProcessIdsList.offset(1);
        }
        
        current_entry = (*current_entry).Flink;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvConsoleProcessCtrlEvent(
    Console: PCONSRV_CONSOLE,
    ProcessGroupId: ULONG,
    CtrlEvent: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut current_entry: PLIST_ENTRY;
    let mut current: PCONSOLE_PROCESS_DATA;

    if ConDrvValidateConsoleState(Console as PCONSOLE, CONSOLE_RUNNING) == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    current_entry = (*Console).ProcessList.Flink;
    while current_entry != core::ptr::addr_of_mut!((*Console).ProcessList) {
        current = (current_entry as *mut u8).offset(
            -(core::mem::offset_of!(CONSOLE_PROCESS_DATA, ConsoleLink) as isize)
        ) as PCONSOLE_PROCESS_DATA;
        current_entry = (*current_entry).Flink;

        if ProcessGroupId == 0 || (*(*current).Process).ProcessGroupId == ProcessGroupId {
            Status = ConSrvConsoleCtrlEvent(CtrlEvent, current);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvSetProcessFocus(
    CsrProcess: PCSR_PROCESS,
    SetForeground: u8,
) {
    if SetForeground != 0 {
        CsrSetForegroundPriority(CsrProcess);
    } else {
        CsrSetBackgroundPriority(CsrProcess);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ConSrvSetConsoleProcessFocus(
    Console: PCONSRV_CONSOLE,
    SetForeground: u8,
) -> i32 {
    let mut current_entry: PLIST_ENTRY;
    let mut current: PCONSOLE_PROCESS_DATA;

    if ConDrvValidateConsoleState(Console as PCONSOLE, CONSOLE_RUNNING) == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    current_entry = (*Console).ProcessList.Flink;
    while current_entry != core::ptr::addr_of_mut!((*Console).ProcessList) {
        current = (current_entry as *mut u8).offset(
            -(core::mem::offset_of!(CONSOLE_PROCESS_DATA, ConsoleLink) as isize)
        ) as PCONSOLE_PROCESS_DATA;
        current_entry = (*current_entry).Flink;

        ConSrvSetProcessFocus((*current).Process, SetForeground);
    }

    STATUS_SUCCESS
}
