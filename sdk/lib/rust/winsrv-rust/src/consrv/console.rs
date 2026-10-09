use winsrv_sys::nt::*;
use super::{wcslen, wcsrchr, wcsncpy, _wcsicmp, PrivateExtractIconsW};
use super::condrv::coninput::*;
use super::term::*;
use super::history::*;
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
// Disambiguate sibling-Rust handle helpers vs winsrv:: bindgen glob.
use super::handle::ConSrvGetObject;
use super::handle::ConSrvReleaseObject;
// Helpers used by the recovered ConSrvInitConsole translation.
use super::condrv::console::ConDrvInitConsole;
use super::condrv::console::ConDrvAttachTerminal;
use super::frontends::terminal::ConSrvInitTerminal;
use super::frontends::terminal::ConSrvDeinitTerminal;
use crate::concfg::settings::ConCfgGetDefaultSettings;
use crate::concfg::settings::ConCfgReadUserSettings;

use core::ffi::c_void;

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


// COM externs (objbase.h / shlobj.h) — bindgen FN_ALLOW doesn't pull
// these in. Stubs so the IShellLink path in ConSrvOpenLNKLink links.
extern "system" {
    pub fn CoInitialize(pvReserved: PVOID) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoCreateInstance(
        rclsid: *const GUID,
        pUnkOuter: *mut c_void,
        dwClsContext: DWORD,
        riid: *const GUID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
}
extern "system" {
    // IShellLinkW vtbl wrappers — supplied by the per-process shell32.
    // C source uses `IShellLinkW_*` macros; we declare them as
    // extern symbols for the linker to satisfy from a shim layer.
    pub fn IShellLinkW_GetShowCmd(pshl: LPVOID, ShowCmd: *mut i32) -> HRESULT;
    pub fn IShellLinkW_GetIconLocation(
        pshl: LPVOID,
        IconPath: *mut WCHAR,
        cchIconPath: i32,
        IconIndex: *mut i32,
    ) -> HRESULT;
    pub fn IShellLinkW_Release(pshl: LPVOID) -> ULONG;
    pub fn IPersistFile_QueryInterface(
        pshl: LPVOID,
        riid: *const GUID,
        ppvObject: *mut LPVOID,
    ) -> HRESULT;
    pub fn IPersistFile_Load(ppf: LPVOID, pszFileName: *const WCHAR, dwMode: DWORD) -> HRESULT;
    pub fn IPersistFile_Release(ppf: LPVOID) -> ULONG;
}
extern "C" {
    pub static CLSID_ShellLink: GUID;
    pub static IID_IShellLinkW: GUID;
    pub static IID_IPersistFile: GUID;
}

// alias.c-internal helper (file-static in C). The cohabitation flip
// would promote it to `#[no_mangle] pub fn IntDeleteAllAliases`; for
// now declare extern so console.rs links.
extern "C" {
    // alias.rs defines IntDeleteAllAliases cdecl (§9.30 cc-match rule)
    pub fn IntDeleteAllAliases(Console: PCONSRV_CONSOLE);
}

// C `FIELD_OFFSET(t, f)` → Rust `core::mem::offset_of!(t, f)` macro.
// Same call shape so the translation can keep `FIELD_OFFSET(T, F)`.
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {
        core::mem::offset_of!($t, $f)
    };
}

// ---- file-scope preamble (translated) ----
pub(crate) static mut ConsoleListSize: ULONG = 0;
pub(crate) static mut ConsoleList: *mut PCONSRV_CONSOLE = core::ptr::null_mut();
pub(crate) static mut ListLock: RTL_RESOURCE = unsafe { core::mem::zeroed() };

// C source has these `// static` commented-out (console.c:30,35).
// They feed the dead-code `ConDrvInsertConsole` only. Keep as
// file-scope stubs to satisfy the translator.
#[allow(non_upper_case_globals)]
pub(crate) static mut CurrentConsoleID: ULONG = 0;
pub(crate) static mut ConDrvConsoleList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

pub const CONSOLE_VALID_CONTROL_MODES: ULONG = (ENABLE_EXTENDED_FLAGS as u32) | (ENABLE_INSERT_MODE as u32) | (ENABLE_QUICK_EDIT_MODE as u32);

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
    // C source for this body is bracketed `#if 0`-style (lines using
    // ConDrvConsoleList / CurrentConsoleID / Console->ListEntry /
    // Console->ConsoleID are all commented out in win32ss/user/winsrv/
    // consrv/console.c:30-65). Keep the lock acquire/release for shape
    // but suppress the body so it compiles against bindgen _CONSOLE
    // (which lacks ListEntry / ConsoleID fields).
    let _ = Console;
    ConSrvLockConsoleListExclusive();
    // InsertTailList / _InterlockedExchange / _InterlockedIncrement —
    // dead in upstream C, suppressed here.
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
    let Block: *mut PCONSRV_CONSOLE;

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
            Status = 0xC0000001u32 as NTSTATUS;
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
    let Console: PCONSRV_CONSOLE;

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
    // NB: C `#if 0`'d. Console->ListEntry field doesn't exist on _CONSOLE or
    // _CONSRV_CONSOLE in the current header (commented out). Preserved here
    // as a no-op so the rest of the translation builds; callers no longer
    // expect this fn to mutate anything.
    if Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    ConSrvLockConsoleListExclusive();
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

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn ConSrvGetConsole(
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

#[no_mangle]
pub unsafe extern "C" fn ConSrvReleaseConsole(
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

// Recovered dropped translation of console.c:ConSrvInitConsole (NTAPI).
// Called by ConSrvAllocateConsole; previously only present as a winsrv-sys
// bindgen extern resolving to the now-removed C console.c.
#[no_mangle]
pub unsafe extern "system" fn ConSrvInitConsole(
    NewConsoleHandle: PHANDLE,
    NewConsole: *mut PCONSRV_CONSOLE,
    ConsoleInitInfo: PCONSOLE_INIT_INFO,
    ConsoleLeaderProcess: PCSR_PROCESS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ConsoleHandle: HANDLE = core::ptr::null_mut();
    let Console: PCONSRV_CONSOLE;

    // BYTE ConsoleInfoBuffer[sizeof(CONSOLE_STATE_INFO) + MAX_PATH * sizeof(WCHAR)]
    const CONSOLE_INFO_BUFFER_SIZE: usize =
        core::mem::size_of::<CONSOLE_STATE_INFO>() + (MAX_PATH as usize) * core::mem::size_of::<WCHAR>();
    let mut ConsoleInfoBuffer: [u8; CONSOLE_INFO_BUFFER_SIZE] = [0u8; CONSOLE_INFO_BUFFER_SIZE];
    let ConsoleInfo: PCONSOLE_STATE_INFO = ConsoleInfoBuffer.as_mut_ptr() as PCONSOLE_STATE_INFO;
    let mut DrvConsoleInfo: CONSOLE_INFO = core::mem::zeroed();

    let Length: usize;

    let mut Terminal: TERMINAL = core::mem::zeroed();

    if NewConsole.is_null() || ConsoleInitInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *NewConsole = core::ptr::null_mut();

    /* Load the console settings */
    RtlZeroMemory(
        ConsoleInfoBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        CONSOLE_INFO_BUFFER_SIZE,
    );
    (*ConsoleInfo).cbSize = CONSOLE_INFO_BUFFER_SIZE as ULONG;

    /* 1. Get the title of the console (initialize ConsoleInfo->ConsoleTitle) */
    let cap = ((*ConsoleInfo).cbSize as usize)
        .saturating_sub(core::mem::offset_of!(CONSOLE_STATE_INFO, ConsoleTitle))
        .saturating_sub(core::mem::size_of::<u16>())
        / core::mem::size_of::<WCHAR>();
    Length = core::cmp::min((*ConsoleInitInfo).TitleLength as usize, cap);
    let TitleDst = core::ptr::addr_of_mut!((*ConsoleInfo).ConsoleTitle) as *mut u16;
    wcsncpy(TitleDst, (*ConsoleInitInfo).ConsoleTitle, Length);
    *TitleDst.add(Length) = 0u16; // NULL-terminate it.

    /* 2. Impersonate the caller in order to retrieve settings in its context */
    if CsrImpersonateClient(core::ptr::null_mut()) == 0u8 {
        return STATUS_BAD_IMPERSONATION_LEVEL;
    }

    /* 3. Load the default settings */
    ConCfgGetDefaultSettings(ConsoleInfo);

    /*
     * 4. Load per-application terminal settings.
     */
    {
        if LoadShellLinkConsoleInfo(ConsoleInfo, ConsoleInitInfo) == 0u8 {
            (*(*ConsoleInitInfo).ConsoleStartInfo).dwStartupFlags &= !STARTF_TITLEISLINKNAME;
        }
    }

    /* 5. Load the remaining console settings via the registry. */
    if ((*(*ConsoleInitInfo).ConsoleStartInfo).dwStartupFlags & STARTF_TITLEISLINKNAME) == 0 {
        ConCfgReadUserSettings(ConsoleInfo, 0u8);

        if ((*(*ConsoleInitInfo).ConsoleStartInfo).dwStartupFlags & STARTF_USEFILLATTRIBUTE) != 0 {
            (*ConsoleInfo).ScreenAttributes =
                (*(*ConsoleInitInfo).ConsoleStartInfo).wFillAttribute as USHORT;
        }
        if ((*(*ConsoleInitInfo).ConsoleStartInfo).dwStartupFlags & STARTF_USECOUNTCHARS) != 0 {
            (*ConsoleInfo).ScreenBufferSize =
                (*(*ConsoleInitInfo).ConsoleStartInfo).dwScreenBufferSize;
        }
        if ((*(*ConsoleInitInfo).ConsoleStartInfo).dwStartupFlags & STARTF_USESIZE) != 0 {
            (*ConsoleInfo).WindowSize = (*(*ConsoleInitInfo).ConsoleStartInfo).dwWindowSize;
        }
    }

    /* 6. Revert impersonation */
    CsrRevertToSelf();

    /* Set-up the code page */
    (*ConsoleInfo).CodePage = GetOEMCP();

    /*
     * Initialize the ConSrv terminal and give it a chance to load
     * its own settings and override the console settings.
     */
    Status = ConSrvInitTerminal(
        core::ptr::addr_of_mut!(Terminal),
        ConsoleInfo,
        ConsoleInitInfo,
        (*ConsoleLeaderProcess).ProcessHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Initialize a new console via the driver */
    DrvConsoleInfo.ScreenBufferSize = (*ConsoleInfo).ScreenBufferSize;
    DrvConsoleInfo.ConsoleSize = (*ConsoleInfo).WindowSize;
    DrvConsoleInfo.CursorSize = (*ConsoleInfo).CursorSize;
    DrvConsoleInfo.ScreenAttrib = (*ConsoleInfo).ScreenAttributes;
    DrvConsoleInfo.PopupAttrib = (*ConsoleInfo).PopupAttributes;
    DrvConsoleInfo.CodePage = (*ConsoleInfo).CodePage;

    /* Allocate a new console */
    Console = ConsoleAllocHeap(HEAP_ZERO_MEMORY, core::mem::size_of::<CONSRV_CONSOLE>())
        as PCONSRV_CONSOLE;
    if Console.is_null() {
        ConSrvDeinitTerminal(core::ptr::addr_of_mut!(Terminal));
        return STATUS_NO_MEMORY;
    }

    Status = ConDrvInitConsole(Console as PCONSOLE, core::ptr::addr_of_mut!(DrvConsoleInfo));
    if !NT_SUCCESS(Status) {
        ConsoleFreeHeap(Console as PVOID);
        ConSrvDeinitTerminal(core::ptr::addr_of_mut!(Terminal));
        return Status;
    }

    /*** Register ConSrv features ***/

    /* Initialize the console title */
    ConsoleCreateUnicodeString(
        core::ptr::addr_of_mut!((*Console).OriginalTitle),
        core::ptr::addr_of!((*ConsoleInfo).ConsoleTitle) as PCWSTR,
    );
    ConsoleCreateUnicodeString(
        core::ptr::addr_of_mut!((*Console).Title),
        core::ptr::addr_of!((*ConsoleInfo).ConsoleTitle) as PCWSTR,
    );

    /* Initialize process support */
    InitializeListHead(core::ptr::addr_of_mut!((*Console).ProcessList));
    (*Console).NotifiedLastCloseProcess = core::ptr::null_mut();
    (*Console).NotifyLastClose = 0u8;
    (*Console).HasFocus = 0u8;

    /* Initialize pausing support */
    (*Console).PauseFlags = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Console).ReadWaitQueue));
    InitializeListHead(core::ptr::addr_of_mut!((*Console).WriteWaitQueue));

    /* Initialize the alias and history buffers */
    (*Console).Aliases = core::ptr::null_mut();
    InitializeListHead(core::ptr::addr_of_mut!((*Console).HistoryBuffers));
    (*Console).NumberOfHistoryBuffers = 0;
    (*Console).MaxNumberOfHistoryBuffers = (*ConsoleInfo).NumberOfHistoryBuffers;
    (*Console).HistoryBufferSize = (*ConsoleInfo).HistoryBufferSize;
    (*Console).HistoryNoDup = (*ConsoleInfo).HistoryNoDup as BOOLEAN;

    /* Initialize the Input Line Discipline */
    (*Console).LineBuffer = core::ptr::null_mut();
    (*Console).LinePos = 0;
    (*Console).LineMaxSize = 0;
    (*Console).LineSize = 0;
    (*Console).LineComplete = 0u8;
    (*Console).LineUpPressed = 0u8;
    (*Console).LineInsertToggle = (*ConsoleInfo).InsertMode as BOOLEAN;
    (*Console).InsertMode = (*ConsoleInfo).InsertMode as BOOLEAN;
    (*Console).QuickEdit = (*ConsoleInfo).QuickEdit as BOOLEAN;

    /* Popup windows */
    InitializeListHead(core::ptr::addr_of_mut!((*Console).PopupWindows));

    /* Colour table */
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*Console).Colors) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*ConsoleInfo).ColorTable) as *const core::ffi::c_void,
        core::mem::size_of_val(&(*ConsoleInfo).ColorTable),
    );

    /* Create the Initialization Events */
    Status = NtCreateEvent(
        core::ptr::addr_of_mut!((*Console).InitEvents[INIT_SUCCESS as usize]),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        NotificationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        ConDrvDeleteConsole(Console as PCONSOLE);
        ConSrvDeinitTerminal(core::ptr::addr_of_mut!(Terminal));
        return Status;
    }
    Status = NtCreateEvent(
        core::ptr::addr_of_mut!((*Console).InitEvents[INIT_FAILURE as usize]),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        NotificationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        NtClose((*Console).InitEvents[INIT_SUCCESS as usize]);
        ConDrvDeleteConsole(Console as PCONSOLE);
        ConSrvDeinitTerminal(core::ptr::addr_of_mut!(Terminal));
        return Status;
    }

    /*
     * Attach the ConSrv terminal to the console.
     * This call makes a copy of our local Terminal variable.
     */
    Status = ConDrvAttachTerminal(Console as PCONSOLE, core::ptr::addr_of_mut!(Terminal));
    if !NT_SUCCESS(Status) {
        NtClose((*Console).InitEvents[INIT_FAILURE as usize]);
        NtClose((*Console).InitEvents[INIT_SUCCESS as usize]);
        ConDrvDeleteConsole(Console as PCONSOLE);
        ConSrvDeinitTerminal(core::ptr::addr_of_mut!(Terminal));
        return Status;
    }

    /* All went right, so add the console to the list */
    let _ = InsertConsole(core::ptr::addr_of_mut!(ConsoleHandle), Console);

    // FIXME! We do not support at all asynchronous console creation!
    NtSetEvent((*Console).InitEvents[INIT_SUCCESS as usize], core::ptr::null_mut());

    /* Return the newly created console to the caller and a success code too */
    *NewConsoleHandle = ConsoleHandle;
    *NewConsole = Console;
    STATUS_SUCCESS
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
                    ) as usize;
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

    LinkName = wcsrchr((*ConsoleInfo).ConsoleTitle.as_mut_ptr(), PATH_SEPARATOR as u16);
    if LinkName == core::ptr::null_mut() {
        LinkName = (*ConsoleInfo).ConsoleTitle.as_mut_ptr();
    } else {
        LinkName = LinkName.offset(1);
    }

    Length = wcslen(LinkName);
    if Length <= 4 || _wcsicmp(LinkName.offset((Length - 4) as isize), w!(".lnk").as_ptr() as *const u16) != 0 {
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
                // ConsoleTitle is a C flexible array (`WCHAR ConsoleTitle[ANYSIZE_ARRAY]`,
                // concfg/settings.h:58) sized by cbSize, as the min() below computes. Its
                // terminator indexed bindgen's [_; 1] and panicked -- in csrss -- whenever a
                // console was started from a shortcut. One raw base, never a reference.
                let title = &raw mut (*ConsoleInfo).ConsoleTitle[0];
                hRes = IPersistFile_Load(ppf, title, 0);
                if SUCCEEDED(hRes) {
                    let mut ShowCmd: i32 = 0;

                    Length = min(
                        Length - 4,
                        ((*ConsoleInfo).cbSize as usize - FIELD_OFFSET!(_CONSOLE_STATE_INFO, ConsoleTitle) - core::mem::size_of::<u16>()) / core::mem::size_of::<u16>(),
                    );
                    wcsncpy(title, LinkName, Length);
                    *title.add(Length) = 0;

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
                ) as usize;
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

// C decl (console.c) is plain cdecl (no NTAPI): VOID ConioPause(PCONSRV_CONSOLE, UCHAR).
// Still-C conwnd.c references `_ConioPause` → #[no_mangle] extern "C".
#[no_mangle]
pub unsafe extern "C" fn ConioPause(Console: PCONSRV_CONSOLE, Flags: u8) {
    (*Console).PauseFlags |= Flags;
    ConDrvPause(Console as PCONSOLE);
}

#[no_mangle]
pub unsafe extern "C" fn ConioUnpause(Console: PCONSRV_CONSOLE, Flags: u8) {
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
pub unsafe extern "C" fn ConSrvAllocateConsole(
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
        (-1isize) as HANDLE,
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
        (-1isize) as HANDLE,
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
        (-1isize) as HANDLE,
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

#[no_mangle]
pub unsafe extern "C" fn ConSrvInheritConsole(
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

#[no_mangle]
pub unsafe extern "C" fn ConSrvRemoveConsole(
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
pub unsafe extern "C" fn ConSrvConsoleCtrlEventTimeout(
    CtrlEvent: ULONG,
    ProcessData: PCONSOLE_PROCESS_DATA,
    Timeout: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*ProcessData).CtrlRoutine.is_none() {
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

// C decl (console.c) is NTAPI → keep extern "system"; export the symbol
// (`_ConSrvGetConsoleLeaderProcess@4`) for the still-C conwnd.c/guiterm.c.
#[no_mangle]
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
    mut ProcessIdsList: *mut ULONG,
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

// ===========================================================================
// CSR API handlers (the Srv* entry points from console.c). These were dropped
// in the original machine translation; hand-translated here from the C source.
// ===========================================================================

// `CodePageToCharSet` is a real C function (win32ss/user/winsrv/concfg/font.c)
// that is not surfaced by the winsrv-sys bindgen catalog. Declare it extern so
// the SrvGetConsoleLangId path links against the per-process concfg.
// C decl (concfg/font.h) is plain cdecl (no NTAPI) → extern "C".
extern "C" {
    pub fn CodePageToCharSet(CodePage: UINT) -> BYTE;
}

// ConDrv* console-driver entry points used by the Srv* handlers below. Not
// surfaced by bindgen (sibling coninput.rs extern-declares its own ConDrv*
// the same way).
extern "system" {
    pub fn ConDrvGetConsoleMode(
        Console: PCONSOLE,
        Object: PCONSOLE_IO_OBJECT,
        ConsoleMode: *mut ULONG,
    ) -> NTSTATUS;
    pub fn ConDrvSetConsoleMode(
        Console: PCONSOLE,
        Object: PCONSOLE_IO_OBJECT,
        ConsoleMode: ULONG,
    ) -> NTSTATUS;
    pub fn ConDrvGetConsoleCP(
        Console: PCONSOLE,
        CodePage: *mut UINT,
        OutputCP: BOOLEAN,
    ) -> NTSTATUS;
    pub fn ConDrvSetConsoleCP(
        Console: PCONSOLE,
        CodePage: UINT,
        OutputCP: BOOLEAN,
    ) -> NTSTATUS;
}

// wincon.h: #define ATTACH_PARENT_PROCESS ((DWORD)-1)
const ATTACH_PARENT_PROCESS: DWORD = 0xFFFFFFFFu32;

// winnt.h string terminators.
const UNICODE_NULL: WCHAR = 0;
const ANSI_NULL: CHAR = 0;

// winnt.h: MAKELANGID(p, s) = ((((WORD)(s)) << 10) | (WORD)(p))
#[inline]
fn MAKELANGID(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

// reactos/cjkcode.h: IsCJKCharSet(CharSet) macro.
#[inline]
fn IsCJKCharSet(char_set: BYTE) -> bool {
    char_set as u32 == SHIFTJIS_CHARSET
        || char_set as u32 == HANGUL_CHARSET
        || char_set as u32 == GB2312_CHARSET
        || char_set as u32 == CHINESEBIG5_CHARSET
}

/* API_NUMBER: ConsolepAlloc */
#[no_mangle]
pub unsafe extern "system" fn SrvAllocConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let AllocConsoleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.AllocConsoleRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    SrvAllocConsoleImpl(ProcessData, ApiMessage, AllocConsoleRequest)
}

pub(crate) unsafe fn SrvAllocConsoleImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    ApiMessage: PCSR_API_MESSAGE,
    AllocConsoleRequest: PCONSOLE_ALLOCCONSOLE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut ConsoleInitInfo: CONSOLE_INIT_INFO = core::mem::zeroed();

    if !(*ProcessData).ConsoleHandle.is_null() {
        return STATUS_ACCESS_DENIED;
    }

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*AllocConsoleRequest).ConsoleStartInfo) as *mut PVOID,
        1,
        core::mem::size_of::<CONSOLE_START_INFO>() as ULONG,
    ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*AllocConsoleRequest).ConsoleTitle) as *mut PVOID,
            (*AllocConsoleRequest).TitleLength,
            core::mem::size_of::<BYTE>() as ULONG,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*AllocConsoleRequest).Desktop) as *mut PVOID,
            (*AllocConsoleRequest).DesktopLength,
            core::mem::size_of::<BYTE>() as ULONG,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*AllocConsoleRequest).CurDir) as *mut PVOID,
            (*AllocConsoleRequest).CurDirLength,
            core::mem::size_of::<BYTE>() as ULONG,
        ) == 0
        || CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*AllocConsoleRequest).AppName) as *mut PVOID,
            (*AllocConsoleRequest).AppNameLength,
            core::mem::size_of::<BYTE>() as ULONG,
        ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Initialize the console initialization info structure */
    ConsoleInitInfo.ConsoleStartInfo = (*AllocConsoleRequest).ConsoleStartInfo;
    ConsoleInitInfo.IsWindowVisible = 1u8; // The console window is always visible.
    ConsoleInitInfo.TitleLength = (*AllocConsoleRequest).TitleLength;
    ConsoleInitInfo.ConsoleTitle = (*AllocConsoleRequest).ConsoleTitle;
    ConsoleInitInfo.DesktopLength = (*AllocConsoleRequest).DesktopLength;
    ConsoleInitInfo.Desktop = (*AllocConsoleRequest).Desktop;
    ConsoleInitInfo.AppNameLength = (*AllocConsoleRequest).AppNameLength;
    ConsoleInitInfo.AppName = (*AllocConsoleRequest).AppName;
    ConsoleInitInfo.CurDirLength = (*AllocConsoleRequest).CurDirLength;
    ConsoleInitInfo.CurDir = (*AllocConsoleRequest).CurDir;

    /* Initialize a new Console owned by the Console Leader Process */
    Status = ConSrvAllocateConsole(
        ProcessData,
        core::ptr::addr_of_mut!((*(*AllocConsoleRequest).ConsoleStartInfo).InputHandle),
        core::ptr::addr_of_mut!((*(*AllocConsoleRequest).ConsoleStartInfo).OutputHandle),
        core::ptr::addr_of_mut!((*(*AllocConsoleRequest).ConsoleStartInfo).ErrorHandle),
        core::ptr::addr_of_mut!(ConsoleInitInfo),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Set the Property-Dialog and Control-Dispatcher handlers */
    (*ProcessData).PropRoutine = (*AllocConsoleRequest).PropRoutine;
    (*ProcessData).CtrlRoutine = (*AllocConsoleRequest).CtrlRoutine;

    STATUS_SUCCESS
}

/* API_NUMBER: ConsolepAttach */
#[no_mangle]
pub unsafe extern "system" fn SrvAttachConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let AttachConsoleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.AttachConsoleRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    SrvAttachConsoleImpl(ProcessData, ApiMessage, AttachConsoleRequest)
}

pub(crate) unsafe fn SrvAttachConsoleImpl(
    _ProcessData: PCONSOLE_PROCESS_DATA,
    ApiMessage: PCSR_API_MESSAGE,
    AttachConsoleRequest: PCONSOLE_ATTACHCONSOLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SourceProcess: PCSR_PROCESS = core::ptr::null_mut(); // The parent process.
    let TargetProcess: PCSR_PROCESS = (*CsrGetClientThread()).Process; // Ourselves.
    let mut ProcessId: HANDLE = ULongToHandle((*AttachConsoleRequest).ProcessId);
    let SourceProcessData: PCONSOLE_PROCESS_DATA;
    let TargetProcessData: PCONSOLE_PROCESS_DATA;

    TargetProcessData = ConsoleGetPerProcessData(TargetProcess);

    if !(*TargetProcessData).ConsoleHandle.is_null() {
        return STATUS_ACCESS_DENIED;
    }

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*AttachConsoleRequest).ConsoleStartInfo) as *mut PVOID,
        1,
        core::mem::size_of::<CONSOLE_START_INFO>() as ULONG,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Check whether we try to attach to the parent's console */
    if ProcessId == ULongToHandle(ATTACH_PARENT_PROCESS) {
        let mut ProcessInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
        let mut Length: ULONG = core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG;

        /* Get the real parent's PID */
        Status = NtQueryInformationProcess(
            (*TargetProcess).ProcessHandle,
            ProcessBasicInformation,
            core::ptr::addr_of_mut!(ProcessInfo) as PVOID,
            Length,
            core::ptr::addr_of_mut!(Length),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        ProcessId = ULongToHandle(ProcessInfo.InheritedFromUniqueProcessId as ULONG);
    }

    /* Lock the source process via its PID */
    Status = CsrLockProcessByClientId(ProcessId, core::ptr::addr_of_mut!(SourceProcess));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    SourceProcessData = ConsoleGetPerProcessData(SourceProcess);

    'Quit: {
        if (*SourceProcessData).ConsoleHandle.is_null() {
            Status = STATUS_INVALID_HANDLE;
            break 'Quit;
        }

        /*
         * Inherit the console from the parent,
         * if any, otherwise return an error.
         */
        Status = ConSrvInheritConsole(
            TargetProcessData,
            (*SourceProcessData).ConsoleHandle,
            1u8,
            core::ptr::addr_of_mut!((*(*AttachConsoleRequest).ConsoleStartInfo).InputHandle),
            core::ptr::addr_of_mut!((*(*AttachConsoleRequest).ConsoleStartInfo).OutputHandle),
            core::ptr::addr_of_mut!((*(*AttachConsoleRequest).ConsoleStartInfo).ErrorHandle),
            (*AttachConsoleRequest).ConsoleStartInfo,
        );
        if !NT_SUCCESS(Status) {
            break 'Quit;
        }

        /* Set the Property-Dialog and Control-Dispatcher handlers */
        (*TargetProcessData).PropRoutine = (*AttachConsoleRequest).PropRoutine;
        (*TargetProcessData).CtrlRoutine = (*AttachConsoleRequest).CtrlRoutine;

        Status = STATUS_SUCCESS;
    }

    /* Unlock the "source" process and exit */
    CsrUnlockProcess(SourceProcess);
    Status
}

/* API_NUMBER: ConsolepFree */
#[no_mangle]
pub unsafe extern "system" fn SrvFreeConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let FreeConsoleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.FreeConsoleRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);

    SrvFreeConsoleImpl(ProcessData, ApiMessage, FreeConsoleRequest)
}

pub(crate) unsafe fn SrvFreeConsoleImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    _ApiMessage: PCSR_API_MESSAGE,
    FreeConsoleRequest: PCONSOLE_FREECONSOLE,
) -> NTSTATUS {
    /*
     * If this process doesn't have a console handle, bail out immediately.
     * Also the passed console handle should be the same as the process' one.
     */
    if (*FreeConsoleRequest).ConsoleHandle.is_null()
        || (*FreeConsoleRequest).ConsoleHandle != (*ProcessData).ConsoleHandle
    {
        return STATUS_INVALID_HANDLE; // STATUS_ACCESS_DENIED;
    }

    ConSrvRemoveConsole(ProcessData)
}

/* API_NUMBER: ConsolepGetMode */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleMode(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let ConsoleModeRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ConsoleModeRequest
    );
    let result = SrvGetConsoleModeImpl(ProcessData, Console, ApiMessage, ConsoleModeRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleModeImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    ConsoleModeRequest: PCONSOLE_GETSETCONSOLEMODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Object: PCONSOLE_IO_OBJECT = core::ptr::null_mut();
    let ConsoleMode = core::ptr::addr_of_mut!((*ConsoleModeRequest).Mode);

    Status = ConSrvGetObject(
        ProcessData,
        (*ConsoleModeRequest).Handle,
        core::ptr::addr_of_mut!(Object),
        core::ptr::null_mut(),
        GENERIC_READ,
        1u8,
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Get the standard console modes */
    Status = ConDrvGetConsoleMode(Console as PCONSOLE, Object, ConsoleMode);
    if NT_SUCCESS(Status) {
        /*
         * If getting the console modes succeeds, then retrieve
         * the extended CONSRV-specific input modes.
         */
        if (*Object).Type == INPUT_BUFFER {
            if (*Console).InsertMode != 0 || (*Console).QuickEdit != 0 {
                /* Windows also adds ENABLE_EXTENDED_FLAGS, even if it's not documented on MSDN */
                *ConsoleMode |= ENABLE_EXTENDED_FLAGS as ULONG;

                if (*Console).InsertMode != 0 {
                    *ConsoleMode |= ENABLE_INSERT_MODE as ULONG;
                }
                if (*Console).QuickEdit != 0 {
                    *ConsoleMode |= ENABLE_QUICK_EDIT_MODE as ULONG;
                }
            }
        }
    }

    ConSrvReleaseObject(Object, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetMode */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleMode(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let ConsoleModeRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ConsoleModeRequest
    );
    let result = SrvSetConsoleModeImpl(ProcessData, Console, ApiMessage, ConsoleModeRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleModeImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    ConsoleModeRequest: PCONSOLE_GETSETCONSOLEMODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Object: PCONSOLE_IO_OBJECT = core::ptr::null_mut();
    let mut ConsoleMode: ULONG = (*ConsoleModeRequest).Mode;

    Status = ConSrvGetObject(
        ProcessData,
        (*ConsoleModeRequest).Handle,
        core::ptr::addr_of_mut!(Object),
        core::ptr::null_mut(),
        GENERIC_WRITE,
        1u8,
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Set the standard console modes (without the CONSRV-specific input modes) */
    ConsoleMode &= !CONSOLE_VALID_CONTROL_MODES; // Remove CONSRV-specific input modes.
    Status = ConDrvSetConsoleMode(Console as PCONSOLE, Object, ConsoleMode);
    if NT_SUCCESS(Status) {
        /*
         * If setting the console modes succeeds, then set
         * the extended CONSRV-specific input modes.
         */
        if (*Object).Type == INPUT_BUFFER {
            ConsoleMode = (*ConsoleModeRequest).Mode;

            if (ConsoleMode & CONSOLE_VALID_CONTROL_MODES) != 0 {
                /*
                 * If we use control mode flags without ENABLE_EXTENDED_FLAGS,
                 * then consider the flags invalid.
                 */
                if (ConsoleMode & ENABLE_EXTENDED_FLAGS as ULONG) == 0 {
                    Status = STATUS_INVALID_PARAMETER;
                } else {
                    (*Console).InsertMode =
                        if (ConsoleMode & ENABLE_INSERT_MODE as ULONG) != 0 { 1u8 } else { 0u8 };
                    (*Console).QuickEdit =
                        if (ConsoleMode & ENABLE_QUICK_EDIT_MODE as ULONG) != 0 { 1u8 } else { 0u8 };
                }
            }
        }
    }

    ConSrvReleaseObject(Object, 1u8);
    Status
}

/* API_NUMBER: ConsolepGetTitle */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleTitle(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let TitleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.TitleRequest
    );
    let result = SrvGetConsoleTitleImpl(Console, ApiMessage, TitleRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleTitleImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    TitleRequest: PCONSOLE_GETSETCONSOLETITLE,
) -> NTSTATUS {
    let mut Length: ULONG;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*TitleRequest).Title) as *mut PVOID,
        (*TitleRequest).Length,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Copy title of the console to the user title buffer */
    if (*TitleRequest).Unicode != 0 {
        if (*TitleRequest).Length >= core::mem::size_of::<WCHAR>() as ULONG {
            Length = min(
                (*TitleRequest).Length - core::mem::size_of::<WCHAR>() as ULONG,
                (*Console).Title.Length as ULONG,
            );
            RtlCopyMemory(
                (*TitleRequest).Title,
                (*Console).Title.Buffer as PVOID,
                Length as usize,
            );
            *((*TitleRequest).Title as PWCHAR).add(Length as usize / core::mem::size_of::<WCHAR>()) =
                UNICODE_NULL;
            (*TitleRequest).Length = Length;
        } else {
            (*TitleRequest).Length = (*Console).Title.Length as ULONG;
        }
    } else {
        if (*TitleRequest).Length >= core::mem::size_of::<CHAR>() as ULONG {
            Length = min(
                (*TitleRequest).Length - core::mem::size_of::<CHAR>() as ULONG,
                (*Console).Title.Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG,
            );
            Length = WideCharToMultiByte(
                (*Console).InputCodePage,
                0,
                (*Console).Title.Buffer,
                Length as i32,
                (*TitleRequest).Title as LPSTR,
                Length as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as ULONG;
            *((*TitleRequest).Title as PCHAR).add(Length as usize) = ANSI_NULL;
            (*TitleRequest).Length = Length;
        } else {
            (*TitleRequest).Length =
                (*Console).Title.Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG;
        }
    }

    STATUS_SUCCESS
}

/* API_NUMBER: ConsolepSetTitle */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleTitle(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let TitleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.TitleRequest
    );
    let result = SrvSetConsoleTitleImpl(Console, ApiMessage, TitleRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleTitleImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    TitleRequest: PCONSOLE_GETSETCONSOLETITLE,
) -> NTSTATUS {
    let Buffer: PWCHAR;
    let Length: ULONG;

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*TitleRequest).Title) as *mut PVOID,
        (*TitleRequest).Length,
        core::mem::size_of::<BYTE>() as ULONG,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    if (*TitleRequest).Unicode != 0 {
        /* Length is in bytes */
        Length = (*TitleRequest).Length;
    } else {
        /* Use the console input CP for the conversion */
        Length = (MultiByteToWideChar(
            (*Console).InputCodePage,
            0,
            (*TitleRequest).Title as LPCSTR,
            (*TitleRequest).Length as i32,
            core::ptr::null_mut(),
            0,
        ) as ULONG)
            * core::mem::size_of::<WCHAR>() as ULONG;
    }

    /* Allocate a new buffer to hold the new title (NULL-terminated) */
    Buffer = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        Length as usize + core::mem::size_of::<WCHAR>(),
    ) as PWCHAR;
    if Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    /* Free the old title */
    ConsoleFreeUnicodeString(core::ptr::addr_of_mut!((*Console).Title));

    /* Copy title to console */
    (*Console).Title.Buffer = Buffer;
    (*Console).Title.Length = Length as USHORT;
    (*Console).Title.MaximumLength = (*Console).Title.Length + core::mem::size_of::<WCHAR>() as USHORT;

    if (*TitleRequest).Unicode != 0 {
        RtlCopyMemory(
            (*Console).Title.Buffer as PVOID,
            (*TitleRequest).Title,
            (*Console).Title.Length as usize,
        );
    } else {
        MultiByteToWideChar(
            (*Console).InputCodePage,
            0,
            (*TitleRequest).Title as LPCSTR,
            (*TitleRequest).Length as i32,
            (*Console).Title.Buffer,
            (*Console).Title.Length as i32 / core::mem::size_of::<WCHAR>() as i32,
        );
    }

    /* NULL-terminate */
    *(*Console)
        .Title
        .Buffer
        .add((*Console).Title.Length as usize / core::mem::size_of::<WCHAR>()) = UNICODE_NULL;

    TermChangeTitle(Console);

    STATUS_SUCCESS
}

/* API_NUMBER: ConsolepGetCP */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCP(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let GetConsoleCPRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetConsoleCPRequest
    );
    let result = SrvGetConsoleCPImpl(Console, ApiMessage, GetConsoleCPRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleCPImpl(
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    GetConsoleCPRequest: PCONSOLE_GETINPUTOUTPUTCP,
) -> NTSTATUS {
    ConDrvGetConsoleCP(
        Console as PCONSOLE,
        core::ptr::addr_of_mut!((*GetConsoleCPRequest).CodePage),
        if (*GetConsoleCPRequest).OutputCP != 0 { 1u8 } else { 0u8 },
    )
}

/* API_NUMBER: ConsolepSetCP */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleCP(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let SetConsoleCPRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetConsoleCPRequest
    );
    let result = SrvSetConsoleCPImpl(Console, ApiMessage, SetConsoleCPRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleCPImpl(
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetConsoleCPRequest: PCONSOLE_SETINPUTOUTPUTCP,
) -> NTSTATUS {
    ConDrvSetConsoleCP(
        Console as PCONSOLE,
        (*SetConsoleCPRequest).CodePage,
        if (*SetConsoleCPRequest).OutputCP != 0 { 1u8 } else { 0u8 },
    )
}

/* API_NUMBER: ConsolepGetProcessList */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleProcessList(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let GetProcessListRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetProcessListRequest
    );
    let result = SrvGetConsoleProcessListImpl(Console, ApiMessage, GetProcessListRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleProcessListImpl(
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    GetProcessListRequest: PCONSOLE_GETPROCESSLIST,
) -> NTSTATUS {
    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!((*GetProcessListRequest).ProcessIdsList) as *mut PVOID,
        (*GetProcessListRequest).ProcessCount,
        core::mem::size_of::<DWORD>() as ULONG,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    ConSrvGetConsoleProcessList(
        Console,
        (*GetProcessListRequest).ProcessIdsList,
        (*GetProcessListRequest).ProcessCount,
        core::ptr::addr_of_mut!((*GetProcessListRequest).ProcessCount),
    )
}

/* API_NUMBER: ConsolepGenerateCtrlEvent */
#[no_mangle]
pub unsafe extern "system" fn SrvGenerateConsoleCtrlEvent(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let GenerateCtrlEventRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GenerateCtrlEventRequest
    );
    let result = SrvGenerateConsoleCtrlEventImpl(Console, ApiMessage, GenerateCtrlEventRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGenerateConsoleCtrlEventImpl(
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    GenerateCtrlEventRequest: PCONSOLE_GENERATECTRLEVENT,
) -> NTSTATUS {
    ConSrvConsoleProcessCtrlEvent(
        Console,
        (*GenerateCtrlEventRequest).ProcessGroupId,
        (*GenerateCtrlEventRequest).CtrlEvent,
    )
}

/* API_NUMBER: ConsolepNotifyLastClose */
#[no_mangle]
pub unsafe extern "system" fn SrvConsoleNotifyLastClose(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let NotifyLastCloseRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.NotifyLastCloseRequest
    );
    let result =
        SrvConsoleNotifyLastCloseImpl(ProcessData, Console, ApiMessage, NotifyLastCloseRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvConsoleNotifyLastCloseImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    _NotifyLastCloseRequest: PCONSOLE_NOTIFYLASTCLOSE,
) -> NTSTATUS {
    /* Only one process is allowed to be registered for last close notification */
    if (*Console).NotifyLastClose != 0 {
        return STATUS_ACCESS_DENIED;
    }

    (*Console).NotifyLastClose = 1u8;
    (*Console).NotifiedLastCloseProcess = ProcessData;
    STATUS_SUCCESS
}

/* API_NUMBER: ConsolepGetMouseInfo */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleMouseInfo(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let GetMouseInfoRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetMouseInfoRequest
    );
    let result = SrvGetConsoleMouseInfoImpl(Console, ApiMessage, GetMouseInfoRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleMouseInfoImpl(
    _Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    GetMouseInfoRequest: PCONSOLE_GETMOUSEINFO,
) -> NTSTATUS {
    /* Just retrieve the number of buttons of the mouse attached to this console */
    (*GetMouseInfoRequest).NumButtons = GetSystemMetrics(SM_CMOUSEBUTTONS as i32) as ULONG;
    STATUS_SUCCESS
}

/* API_NUMBER: ConsolepSetKeyShortcuts */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleKeyShortcuts(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepGetKeyboardLayoutName */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleKeyboardLayoutName(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let GetKbdLayoutNameRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.GetKbdLayoutNameRequest
    );
    let result = SrvGetConsoleKeyboardLayoutNameImpl(Console, ApiMessage, GetKbdLayoutNameRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleKeyboardLayoutNameImpl(
    _Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    GetKbdLayoutNameRequest: PCONSOLE_GETKBDLAYOUTNAME,
) -> NTSTATUS {
    /* Retrieve the keyboard layout name of the system */
    if (*GetKbdLayoutNameRequest).Ansi != 0 {
        GetKeyboardLayoutNameA((*GetKbdLayoutNameRequest).LayoutBuffer.as_mut_ptr() as PCHAR);
    } else {
        GetKeyboardLayoutNameW((*GetKbdLayoutNameRequest).LayoutBuffer.as_mut_ptr() as PWCHAR);
    }

    STATUS_SUCCESS
}

/* API_NUMBER: ConsolepCharType */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCharType(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepSetLocalEUDC */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleLocalEUDC(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepSetCursorMode */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleCursorMode(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepGetCursorMode */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCursorMode(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepGetNlsMode */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleNlsMode(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepSetNlsMode */
#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleNlsMode(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

/* API_NUMBER: ConsolepGetLangId */
#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleLangId(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }
    let LangIdRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.LangIdRequest
    );
    let result = SrvGetConsoleLangIdImpl(Console, ApiMessage, LangIdRequest);
    ConSrvReleaseConsole(Console, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleLangIdImpl(
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    LangIdRequest: PCONSOLE_GETLANGID,
) -> NTSTATUS {
    let UserCharSet: BYTE = CodePageToCharSet(GetACP());
    if !IsCJKCharSet(UserCharSet) {
        return STATUS_NOT_SUPPORTED;
    }

    /* Return a "best-suited" language ID corresponding
     * to the active console output code page. */
    match (*Console).OutputCodePage {
        /* ReactOS-specific: do nothing if the code page is UTF-8. */
        CP_UTF8 => return STATUS_NOT_SUPPORTED,
        CP_JAPANESE => {
            (*LangIdRequest).LangId =
                MAKELANGID(LANG_JAPANESE as u16, SUBLANG_DEFAULT as u16);
        }
        CP_KOREAN => {
            (*LangIdRequest).LangId =
                MAKELANGID(LANG_KOREAN as u16, SUBLANG_KOREAN as u16);
        }
        CP_CHINESE_SIMPLIFIED => {
            (*LangIdRequest).LangId =
                MAKELANGID(LANG_CHINESE as u16, SUBLANG_CHINESE_SIMPLIFIED as u16);
        }
        CP_CHINESE_TRADITIONAL => {
            (*LangIdRequest).LangId =
                MAKELANGID(LANG_CHINESE as u16, SUBLANG_CHINESE_TRADITIONAL as u16);
        }
        _ => {
            /* Default to en-US english otherwise */
            (*LangIdRequest).LangId =
                MAKELANGID(LANG_ENGLISH as u16, SUBLANG_ENGLISH_US as u16);
        }
    }

    STATUS_SUCCESS
}
