use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::console::cffi::*;
use core::ffi::c_void;
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// CONTAINING_RECORD(addr, type, field): recover the struct base from a member ptr.
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $ty:ty, $field:ident) => {
        ($addr as *mut u8).offset(-(core::mem::offset_of!($ty, $field) as isize)) as *mut $ty
    };
}

extern "system" {
    fn GetCurrentThreadId() -> DWORD;
    fn GetCurrentThread() -> HANDLE;
    fn SetThreadPriority(hThread: HANDLE, nPriority: i32) -> BOOL;
    fn IsDebuggerPresent() -> BOOL;
    fn SetThreadLocale(lcid: LCID) -> BOOL;
}
extern "C" {
    fn MAKELCID(langid: LANGID, sortid: DWORD) -> LCID;
}

// winnt.h PtrToUlong: truncate a pointer to a ULONG.
#[inline]
unsafe fn PtrToUlong(p: *mut core::ffi::c_void) -> ULONG {
    (p as usize) as ULONG
}

// const zero-initializer for static RTL_CRITICAL_SECTION (matches C zero-init).
const fn k32_zeroed<T>() -> T {
    unsafe { core::mem::MaybeUninit::<T>::zeroed().assume_init() }
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

static mut BaseConFileName: LPCWSTR = CONSOLE_FILE_NAME;
static mut BaseConInputFileName: LPCWSTR = CONSOLE_INPUT_FILE_NAME;
static mut BaseConOutputFileName: LPCWSTR = CONSOLE_OUTPUT_FILE_NAME;
static mut InitialHandler: [PHANDLER_ROUTINE; 1] = [None];
static mut CtrlHandlers: *mut PHANDLER_ROUTINE = core::ptr::null_mut();
static mut NrCtrlHandlers: ULONG = 0;
static mut NrAllocatedHandlers: ULONG = 0;
static mut LastCloseNotify: BOOLEAN = 0;

pub static mut InputWaitHandle: HANDLE = INVALID_HANDLE_VALUE;

pub const EXENAME_LENGTH: u32 = 255 + 1;

static mut ExeNameLock: RTL_CRITICAL_SECTION = k32_zeroed();
static mut ExeNameInitialized: BOOLEAN = 0;
static mut ExeNameBuffer: [WCHAR; EXENAME_LENGTH as usize] = [0; EXENAME_LENGTH as usize];
static mut ExeNameLength: USHORT = 0;
static mut StartDirBuffer: [WCHAR; (MAX_PATH + 1) as usize] = [0; (MAX_PATH + 1) as usize];
static mut StartDirLength: USHORT = 0;
unsafe extern "system" fn DefaultConsoleCtrlHandler(Event: DWORD) -> BOOL {
    // DPRINT("Default handler called: %lx\n", Event);
    match Event {
        CTRL_C_EVENT => {
            // DPRINT("Ctrl-C Event\n");
        }
        CTRL_BREAK_EVENT => {
            // DPRINT("Ctrl-Break Event\n");
        }
        CTRL_CLOSE_EVENT => {
            // DPRINT("Ctrl Close Event\n");
        }
        CTRL_LAST_CLOSE_EVENT => {
            // DPRINT("Ctrl Last Close Event\n");
        }
        CTRL_LOGOFF_EVENT => {
            // DPRINT("Ctrl Logoff Event\n");
        }
        CTRL_SHUTDOWN_EVENT => {
            // DPRINT("Ctrl Shutdown Event\n");
        }
        _ => {}
    }

    ExitProcess(CONTROL_C_EXIT as UINT);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ConsoleControlDispatcher(
    lpThreadParameter: *mut core::ffi::c_void,
) -> u32 {
    let mut nExitCode: u32 = 0;
    let CodeAndFlag: u32 = PtrToUlong(lpThreadParameter);
    let nCode: u32 = CodeAndFlag & MAXLONG;
    let mut i: u32;
    let mut erException: EXCEPTION_RECORD = core::mem::zeroed();

    // DPRINT1("Console Dispatcher Active: %lx %lx\n", CodeAndFlag, nCode);
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST as i32);

    match nCode {
        0 => {
            // CTRL_C_EVENT
            if IsDebuggerPresent() != 0 {
                erException.ExceptionCode = if nCode == 0 {
                    // DBG_CONTROL_C
                    0
                } else {
                    // DBG_CONTROL_BREAK
                    1
                };
                erException.ExceptionFlags = 0;
                erException.ExceptionRecord = core::ptr::null_mut();
                erException.ExceptionAddress = DefaultConsoleCtrlHandler as *mut core::ffi::c_void;
                erException.NumberParameters = 0;

                // _SEH2_TRY
                {
                    RtlRaiseException(&mut erException);
                }
                // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
                {
                    RtlEnterCriticalSection(&raw mut ConsoleLock);

                    if (nCode != 0) ||
                        ((*(*(NtCurrentPeb())).ProcessParameters).ConsoleFlags != 1)
                    {
                        i = NrCtrlHandlers;
                        while i > 0 {
                            if (*CtrlHandlers.add((i - 1) as usize)).map_or(0, |h| h(nCode)) != 0 {
                                break;
                            }
                            i -= 1;
                        }
                    }

                    RtlLeaveCriticalSection(&raw mut ConsoleLock);
                }
                // _SEH2_END

                ExitThread(0);
            }
        }

        1 | 2 | 5 => {
            // CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT
            // break - do nothing
        }

        6 => {
            // CTRL_LAST_CLOSE_EVENT
            if LastCloseNotify == 0 {
                ExitThread(0);
            }
        }

        4 => {
            ExitProcess(0); // CONTROL_C_EXIT
        }

        _ => {
            // ASSERT(FALSE);
        }
    }

    // ASSERT(ConsoleInitialized);

    RtlEnterCriticalSection(&raw mut ConsoleLock);

    nExitCode = 0;
    if (nCode != 0) || ((*(*(NtCurrentPeb())).ProcessParameters).ConsoleFlags != 1) {
        i = NrCtrlHandlers;
        while i > 0 {
            if (i == 1) &&
                ((CodeAndFlag & MINLONG) != 0) &&
                ((nCode == 2) || (nCode == 5))
            {
                // DPRINT("Skipping system/service apps\n");
                break;
            }

            if (*CtrlHandlers.add((i - 1) as usize)).map_or(0, |h| h(nCode)) != 0 {
                match nCode {
                    1 | 6 | 2 | 5 => {
                        // CTRL_CLOSE_EVENT, CTRL_LAST_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT
                        nExitCode = CodeAndFlag;
                    }
                    _ => {}
                }
                break;
            }
            i -= 1;
        }
    }

    RtlLeaveCriticalSection(&raw mut ConsoleLock);

    ExitThread(nExitCode);
    return 0; // STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn InitializeCtrlHandling() {
    NrAllocatedHandlers = 1;
    NrCtrlHandlers = 1;
    CtrlHandlers = (*(&raw mut InitialHandler)).as_mut_ptr();
    *CtrlHandlers = core::mem::transmute::<unsafe extern "system" fn(DWORD) -> BOOL, PHANDLER_ROUTINE>(DefaultConsoleCtrlHandler);
}
#[no_mangle]
pub unsafe extern "C" fn InitExeName() {
    let Status: NTSTATUS;
    let Peb: PPEB = NtCurrentPeb();
    let CurrentDirectory: PCURDIR = &mut (*(*Peb).ProcessParameters).CurrentDirectory;
    let ImageEntry: PLDR_DATA_TABLE_ENTRY;

    if ExeNameInitialized != 0 {
        return;
    }

    /* Initialize the EXE name lock */
    Status = RtlInitializeCriticalSection(&raw mut ExeNameLock);
    if !NT_SUCCESS(Status) {
        return;
    }
    ExeNameInitialized = 1i32 as BOOLEAN;

    ImageEntry = CONTAINING_RECORD!(
        (*((*Peb).Ldr)).InLoadOrderModuleList.Flink,
        LDR_DATA_TABLE_ENTRY,
        InLoadOrderLinks
    );

    /* Retrieve the EXE name, NULL-terminate it... */
    ExeNameLength = min(
        core::mem::size_of::<[WCHAR; EXENAME_LENGTH as usize]>() as ULONG / (core::mem::size_of::<WCHAR>() as ULONG),
        (*ImageEntry).BaseDllName.Length as ULONG / (core::mem::size_of::<WCHAR>() as ULONG),
    ) as USHORT;
    RtlCopyMemory(
        (*(&raw mut ExeNameBuffer)).as_mut_ptr() as *mut core::ffi::c_void,
        (*ImageEntry).BaseDllName.Buffer as *const core::ffi::c_void,
        (*ImageEntry).BaseDllName.Length as SIZE_T,
    );
    ExeNameBuffer[ExeNameLength as usize] = UNICODE_NULL;

    /* ... and retrieve the current directory path and NULL-terminate it. */
    StartDirLength = min(
        core::mem::size_of::<[WCHAR; (MAX_PATH + 1) as usize]>() as ULONG / (core::mem::size_of::<WCHAR>() as ULONG),
        (*CurrentDirectory).DosPath.Length as ULONG / (core::mem::size_of::<WCHAR>() as ULONG),
    ) as USHORT;
    RtlCopyMemory(
        (*(&raw mut StartDirBuffer)).as_mut_ptr() as *mut core::ffi::c_void,
        (*CurrentDirectory).DosPath.Buffer as *const core::ffi::c_void,
        (*CurrentDirectory).DosPath.Length as SIZE_T,
    );
    StartDirBuffer[StartDirLength as usize] = UNICODE_NULL;
}
#[no_mangle]
pub unsafe extern "C" fn SetUpAppName(
    CaptureStrings: BOOLEAN,
    CurDirLength: LPDWORD,
    CurDir: *mut LPWSTR,
    AppNameLength: LPDWORD,
    AppName: *mut LPWSTR,
) {
    let mut Length: DWORD;

    /* Retrieve the needed buffer size */
    Length = (StartDirLength as DWORD + 1) * (core::mem::size_of::<WCHAR>() as ULONG) as DWORD;
    if *CurDirLength > 0 {
        Length = min(Length, *CurDirLength);
    }
    *CurDirLength = Length;

    /* Capture the data if needed, or, return a pointer to it */
    if CaptureStrings != 0 {
        /*
         * Length is always >= sizeof(WCHAR). Copy everything but the
         * possible trailing NULL character, and then NULL-terminate.
         */
        Length -= (core::mem::size_of::<WCHAR>() as ULONG) as DWORD;
        RtlCopyMemory(
            *CurDir as *mut core::ffi::c_void,
            (*(&raw const StartDirBuffer)).as_ptr() as *const core::ffi::c_void,
            Length as SIZE_T,
        );
        (*(*CurDir as *mut WCHAR).offset((Length / (core::mem::size_of::<WCHAR>() as ULONG) as DWORD) as isize)) = UNICODE_NULL;
    } else {
        *CurDir = (*(&raw const StartDirBuffer)).as_ptr() as LPWSTR;
    }

    /* Retrieve the needed buffer size */
    Length = (ExeNameLength as DWORD + 1) * (core::mem::size_of::<WCHAR>() as ULONG) as DWORD;
    if *AppNameLength > 0 {
        Length = min(Length, *AppNameLength);
    }
    *AppNameLength = Length;

    /* Capture the data if needed, or, return a pointer to it */
    if CaptureStrings != 0 {
        /*
         * Length is always >= sizeof(WCHAR). Copy everything but the
         * possible trailing NULL character, and then NULL-terminate.
         */
        Length -= (core::mem::size_of::<WCHAR>() as ULONG) as DWORD;
        RtlCopyMemory(
            *AppName as *mut core::ffi::c_void,
            (*(&raw const ExeNameBuffer)).as_ptr() as *const core::ffi::c_void,
            Length as SIZE_T,
        );
        (*(*AppName as *mut WCHAR).offset((Length / (core::mem::size_of::<WCHAR>() as ULONG) as DWORD) as isize)) = UNICODE_NULL;
    } else {
        *AppName = (*(&raw const ExeNameBuffer)).as_ptr() as LPWSTR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetCurrentExeName(
    ExeName: *mut u16,
    BufferSize: u16,
) -> u16 {
    let ExeLength: u16;

    if ExeNameInitialized != 0 {
        RtlEnterCriticalSection(&raw mut ExeNameLock);

        let mut BufferSize = BufferSize;
        if (BufferSize as u32) > (ExeNameLength as u32) * (core::mem::size_of::<u16>() as ULONG) as u32 {
            BufferSize = (ExeNameLength as u32 * (core::mem::size_of::<u16>() as ULONG) as u32) as u16;
        }

        core::ptr::copy_nonoverlapping((*(&raw const ExeNameBuffer)).as_ptr(), ExeName, (BufferSize as usize) / core::mem::size_of::<u16>());

        RtlLeaveCriticalSection(&raw mut ExeNameLock);
        ExeLength = BufferSize;
    } else {
        *ExeName = 0x0000u16; // UNICODE_NULL
        ExeLength = 0;
    }

    return ExeLength;
}
#[no_mangle]
pub unsafe extern "C" fn IntCheckForConsoleFileName(
    pszName: LPCWSTR,
    dwDesiredAccess: DWORD,
) -> LPCWSTR {
    let mut ConsoleName: LPCWSTR = pszName;
    let DeviceNameInfo: ULONG;

    DeviceNameInfo = RtlIsDosDeviceName_U(pszName);
    if DeviceNameInfo != 0 {
        ConsoleName = (ConsoleName as ULONG_PTR + ((DeviceNameInfo >> 16) & 0xFFFF) as ULONG_PTR) as LPCWSTR;
    }

    if _wcsicmp(ConsoleName, BaseConInputFileName) == 0 {
        return BaseConInputFileName;
    } else if _wcsicmp(ConsoleName, BaseConOutputFileName) == 0 {
        return BaseConOutputFileName;
    } else if _wcsicmp(ConsoleName, BaseConFileName) == 0 {
        if (dwDesiredAccess & (GENERIC_READ | GENERIC_WRITE)) == GENERIC_READ {
            return BaseConInputFileName;
        } else if (dwDesiredAccess & (GENERIC_READ | GENERIC_WRITE)) == GENERIC_WRITE {
            return BaseConOutputFileName;
        }
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ConsoleMenuControl(
    hConsoleOutput: HANDLE,
    dwCmdIdLow: DWORD,
    dwCmdIdHigh: DWORD,
) -> HANDLE {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let MenuControlRequest: PCONSOLE_MENUCONTROL = &mut ApiMessage.Data.MenuControlRequest;

    (*MenuControlRequest).ConsoleHandle = (*(*(NtCurrentPeb())).ProcessParameters).ConsoleHandle;
    (*MenuControlRequest).OutputHandle = hConsoleOutput;
    (*MenuControlRequest).CmdIdLow = dwCmdIdLow;
    (*MenuControlRequest).CmdIdHigh = dwCmdIdHigh;
    (*MenuControlRequest).MenuHandle = core::ptr::null_mut();

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepMenuControl),
        (core::mem::size_of::<CONSOLE_MENUCONTROL>() as ULONG),
    );

    return (*MenuControlRequest).MenuHandle as _;
}

#[no_mangle]
pub unsafe extern "system" fn DuplicateConsoleHandle(
    hSourceHandle: HANDLE,
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    dwOptions: DWORD,
) -> HANDLE {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let DuplicateHandleRequest: PCONSOLE_DUPLICATEHANDLE = &mut ApiMessage.Data.DuplicateHandleRequest;

    if (dwOptions & !(DUPLICATE_CLOSE_SOURCE | DUPLICATE_SAME_ACCESS)) != 0
        // C: `!(dwOptions & DUPLICATE_SAME_ACCESS)`. Logical not, so `== 0` — Rust's `!`
        // here was bitwise and always non-zero. (The line above IS the legitimate
        // `x & !(MASK)` unknown-flag test; only this one was wrong.)
        || ((dwOptions & DUPLICATE_SAME_ACCESS) == 0
            && (dwDesiredAccess & !(GENERIC_READ | GENERIC_WRITE)) != 0)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    (*DuplicateHandleRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*DuplicateHandleRequest).SourceHandle = hSourceHandle;
    (*DuplicateHandleRequest).DesiredAccess = dwDesiredAccess;
    (*DuplicateHandleRequest).InheritHandle = bInheritHandle as BOOLEAN;
    (*DuplicateHandleRequest).Options = dwOptions;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepDuplicateHandle),
        (core::mem::size_of::<CONSOLE_DUPLICATEHANDLE>() as ULONG) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return INVALID_HANDLE_VALUE;
    }

    return (*DuplicateHandleRequest).TargetHandle;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleHandleInformation(
    hHandle: HANDLE,
    lpdwFlags: LPDWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetHandleInfoRequest: PCONSOLE_GETHANDLEINFO = &mut ApiMessage.Data.GetHandleInfoRequest;

    (*GetHandleInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetHandleInfoRequest).Handle = hHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetHandleInformation),
        (core::mem::size_of::<CONSOLE_GETHANDLEINFO>() as ULONG) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0i32;
    }

    *lpdwFlags = (*GetHandleInfoRequest).Flags;

    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleHandleInformation(
    hHandle: HANDLE,
    dwMask: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetHandleInfoRequest: PCONSOLE_SETHANDLEINFO = &mut ApiMessage.Data.SetHandleInfoRequest;

    (*SetHandleInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetHandleInfoRequest).Handle = hHandle;
    (*SetHandleInfoRequest).Mask = dwMask;
    (*SetHandleInfoRequest).Flags = dwFlags;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetHandleInformation),
        (core::mem::size_of::<CONSOLE_SETHANDLEINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleDisplayMode(lpModeFlags: LPDWORD) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetDisplayModeRequest: PCONSOLE_GETDISPLAYMODE = &mut ApiMessage.Data.GetDisplayModeRequest;

    if lpModeFlags.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*GetDisplayModeRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetDisplayMode),
        (core::mem::size_of::<CONSOLE_GETDISPLAYMODE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    *lpModeFlags = (*GetDisplayModeRequest).DisplayMode; // ModeFlags

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleFontInfo(
    hConsoleOutput: HANDLE,
    bMaximumWindow: BOOL,
    nFontCount: DWORD,
    lpConsoleFontInfo: PCONSOLE_FONT_INFO,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetFontInfoRequest: PCONSOLE_GETFONTINFO = &mut ApiMessage.Data.GetFontInfoRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    (*GetFontInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetFontInfoRequest).OutputHandle = hConsoleOutput;
    (*GetFontInfoRequest).MaximumWindow = bMaximumWindow as BOOLEAN;
    (*GetFontInfoRequest).NumFonts = nFontCount;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, nFontCount * (core::mem::size_of::<CONSOLE_FONT_INFO>() as ULONG));
    if CaptureBuffer.is_null() {
        // DPRINT1("CsrAllocateCaptureBuffer failed!\n");
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrAllocateMessagePointer(
        CaptureBuffer,
        nFontCount * (core::mem::size_of::<CONSOLE_FONT_INFO>() as ULONG),
        &mut (*GetFontInfoRequest).FontInfo as *mut _ as *mut PVOID,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetFontInfo),
        core::mem::size_of_val(&*GetFontInfoRequest) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
    } else {
        RtlCopyMemory(
            lpConsoleFontInfo as *mut core::ffi::c_void,
            (*GetFontInfoRequest).FontInfo as *const core::ffi::c_void,
            (*GetFontInfoRequest).NumFonts as SIZE_T * (core::mem::size_of::<CONSOLE_FONT_INFO>() as SIZE_T),
        );
    }

    CsrFreeCaptureBuffer(CaptureBuffer);
    return (*GetFontInfoRequest).NumFonts;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleFontSize(
    hConsoleOutput: HANDLE,
    nFont: DWORD,
) -> COORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetFontSizeRequest: PCONSOLE_GETFONTSIZE = &mut ApiMessage.Data.GetFontSizeRequest;
    let Empty: COORD = COORD { X: 0, Y: 0 };

    (*GetFontSizeRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetFontSizeRequest).OutputHandle = hConsoleOutput;
    (*GetFontSizeRequest).FontIndex = nFont;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetFontSize),
        (core::mem::size_of::<CONSOLE_GETFONTSIZE>() as ULONG) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return Empty;
    }

    return (*GetFontSizeRequest).FontSize;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleHardwareState(
    hConsoleOutput: HANDLE,
    Flags: PDWORD,
    State: PDWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let HardwareStateRequest: PCONSOLE_GETSETHWSTATE =
        &mut ApiMessage.Data.HardwareStateRequest;

    if Flags.is_null() || State.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*HardwareStateRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*HardwareStateRequest).OutputHandle = hConsoleOutput;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetHardwareState),
        (core::mem::size_of::<CONSOLE_GETSETHWSTATE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    *Flags = (*HardwareStateRequest).Flags;
    *State = (*HardwareStateRequest).State;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleInputWaitHandle() -> HANDLE {
    InputWaitHandle
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentConsoleFont(
    hConsoleOutput: HANDLE,
    bMaximumWindow: BOOL,
    lpConsoleCurrentFont: *mut CONSOLE_FONT_INFO,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetCurrentFontRequest: *mut CONSOLE_GETCURRENTFONT = &mut ApiMessage.Data.GetCurrentFontRequest;

    (*GetCurrentFontRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetCurrentFontRequest).OutputHandle = hConsoleOutput;
    (*GetCurrentFontRequest).MaximumWindow = bMaximumWindow as BOOLEAN;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCurrentFont),
        (core::mem::size_of::<CONSOLE_GETCURRENTFONT>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    (*lpConsoleCurrentFont).dwFontSize = (*GetCurrentFontRequest).FontSize;
    (*lpConsoleCurrentFont).nFont = (*GetCurrentFontRequest).FontIndex;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumberOfConsoleFonts() -> u32 {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetNumFontsRequest: *mut CONSOLE_GETNUMFONTS = &mut ApiMessage.Data.GetNumFontsRequest;

    (*GetNumFontsRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetNumberOfFonts),
        (core::mem::size_of::<CONSOLE_GETNUMFONTS>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return (*GetNumFontsRequest).NumFonts;
}

#[no_mangle]
pub unsafe extern "system" fn InvalidateConsoleDIBits(
    hConsoleOutput: HANDLE,
    lpRect: PSMALL_RECT,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let InvalidateDIBitsRequest: PCONSOLE_INVALIDATEDIBITS =
        &mut ApiMessage.Data.InvalidateDIBitsRequest;

    if lpRect.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*InvalidateDIBitsRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*InvalidateDIBitsRequest).OutputHandle = hConsoleOutput;
    (*InvalidateDIBitsRequest).Region = *lpRect;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepInvalidateBitMapRect),
        (core::mem::size_of::<CONSOLE_INVALIDATEDIBITS>() as ULONG) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn OpenConsoleW(
    wsName: LPCWSTR,
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    dwShareMode: DWORD,
) -> HANDLE {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let OpenConsoleRequest: PCONSOLE_OPENCONSOLE = &mut ApiMessage.Data.OpenConsoleRequest;
    let HandleType: CONSOLE_HANDLE_TYPE;

    if !wsName.is_null() && _wcsicmp(wsName, BaseConInputFileName) == 0 {
        HandleType = HANDLE_INPUT;
    } else if !wsName.is_null() && _wcsicmp(wsName, BaseConOutputFileName) == 0 {
        HandleType = HANDLE_OUTPUT;
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    if (dwDesiredAccess & !(GENERIC_READ | GENERIC_WRITE)) != 0
        || (dwShareMode & !(FILE_SHARE_READ | FILE_SHARE_WRITE)) != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    (*OpenConsoleRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*OpenConsoleRequest).HandleType = HandleType;
    (*OpenConsoleRequest).DesiredAccess = dwDesiredAccess;
    (*OpenConsoleRequest).InheritHandle = bInheritHandle;
    (*OpenConsoleRequest).ShareMode = dwShareMode;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepOpenConsole),
        (core::mem::size_of::<CONSOLE_OPENCONSOLE>() as ULONG) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return INVALID_HANDLE_VALUE;
    }

    return (*OpenConsoleRequest).Handle;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCursor(
    hConsoleOutput: HANDLE,
    hCursor: HCURSOR,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetCursorRequest: PCONSOLE_SETCURSOR = &mut ApiMessage.Data.SetCursorRequest;

    (*SetCursorRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetCursorRequest).OutputHandle = hConsoleOutput;
    (*SetCursorRequest).CursorHandle = hCursor as _;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCursor),
        (core::mem::size_of::<CONSOLE_SETCURSOR>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleDisplayMode(
    hConsoleOutput: HANDLE,
    dwFlags: DWORD,
    lpNewScreenBufferDimensions: PCOORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetDisplayModeRequest: PCONSOLE_SETDISPLAYMODE = &mut ApiMessage.Data.SetDisplayModeRequest;

    (*SetDisplayModeRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetDisplayModeRequest).OutputHandle = hConsoleOutput;
    (*SetDisplayModeRequest).DisplayMode = dwFlags;
    (*SetDisplayModeRequest).NewSBDim.X = 0;
    (*SetDisplayModeRequest).NewSBDim.Y = 0;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetDisplayMode),
        (core::mem::size_of::<CONSOLE_SETDISPLAYMODE>() as ULONG) as u32,
    );

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    if !lpNewScreenBufferDimensions.is_null() {
        *lpNewScreenBufferDimensions = (*SetDisplayModeRequest).NewSBDim;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleFont(
    hConsoleOutput: HANDLE,
    nFont: DWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetFontRequest: PCONSOLE_SETFONT = &mut ApiMessage.Data.SetFontRequest;

    (*SetFontRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetFontRequest).OutputHandle = hConsoleOutput;
    (*SetFontRequest).FontIndex = nFont;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetFont),
        (core::mem::size_of::<CONSOLE_SETFONT>() as ULONG) as u32,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleHardwareState(
    hConsoleOutput: HANDLE,
    Flags: DWORD,
    State: DWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let HardwareStateRequest: PCONSOLE_GETSETHWSTATE = &mut ApiMessage.Data.HardwareStateRequest;

    // DPRINT1("SetConsoleHardwareState(%lu, %lu) unimplemented!()\n", Flags, State);

    (*HardwareStateRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*HardwareStateRequest).OutputHandle = hConsoleOutput;
    (*HardwareStateRequest).Flags = Flags;
    (*HardwareStateRequest).State = State;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetHardwareState),
        (core::mem::size_of::<CONSOLE_GETSETHWSTATE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleKeyShortcuts(
    bSet: BOOL,
    bReserveKeys: BYTE,
    lpAppKeys: LPAPPKEY,
    dwNumAppKeys: DWORD,
) -> BOOL {
    // DPRINT1("SetConsoleKeyShortcuts(%lu, 0x%x, 0x%p, 0x%x) unimplemented!()\n",
    //         bSet, bReserveKeys, lpAppKeys, dwNumAppKeys);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0; // FALSE
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleMaximumWindowSize(
    hConsoleOutput: HANDLE,
    dwMaximumSize: COORD,
) -> BOOL {
    // DPRINT1("SetConsoleMaximumWindowSize(0x%p, {%d, %d}) does nothing\n",
    //         hConsoleOutput, dwMaximumSize.X, dwMaximumSize.Y);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleMenuClose(bEnable: BOOL) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetMenuCloseRequest: PCONSOLE_SETMENUCLOSE = &mut ApiMessage.Data.SetMenuCloseRequest;

    (*SetMenuCloseRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetMenuCloseRequest).Enable = bEnable;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetMenuClose),
        (core::mem::size_of::<CONSOLE_SETMENUCLOSE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsolePalette(
    hConsoleOutput: HANDLE,
    hPalette: HPALETTE,
    dwUsage: UINT,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetPaletteRequest: PCONSOLE_SETPALETTE = &mut ApiMessage.Data.SetPaletteRequest;

    (*SetPaletteRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetPaletteRequest).OutputHandle = hConsoleOutput;
    (*SetPaletteRequest).PaletteHandle = hPalette as _;
    (*SetPaletteRequest).Usage = dwUsage;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetPalette),
        (core::mem::size_of::<CONSOLE_SETPALETTE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ShowConsoleCursor(
    hConsoleOutput: HANDLE,
    bShow: BOOL,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ShowCursorRequest: PCONSOLE_SHOWCURSOR = &mut ApiMessage.Data.ShowCursorRequest;

    (*ShowCursorRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*ShowCursorRequest).OutputHandle = hConsoleOutput;
    (*ShowCursorRequest).Show = bShow;
    (*ShowCursorRequest).RefCount = 0;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepShowCursor),
        (core::mem::size_of::<CONSOLE_SHOWCURSOR>() as ULONG),
    );

    return (*ShowCursorRequest).RefCount;
}

#[no_mangle]
pub unsafe extern "system" fn VerifyConsoleIoHandle(
    hIoHandle: HANDLE,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let VerifyHandleRequest: PCONSOLE_VERIFYHANDLE = &mut ApiMessage.Data.VerifyHandleRequest;

    (*VerifyHandleRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*VerifyHandleRequest).Handle = hIoHandle;
    (*VerifyHandleRequest).IsValid = 0;

    if (*VerifyHandleRequest).ConsoleHandle.is_null() {
        return 0;
    }

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepVerifyIoHandle),
        (core::mem::size_of::<CONSOLE_VERIFYHANDLE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return (*VerifyHandleRequest).IsValid;
}

#[no_mangle]
pub unsafe extern "system" fn CloseConsoleHandle(hHandle: HANDLE) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let CloseHandleRequest: PCONSOLE_CLOSEHANDLE = &mut ApiMessage.Data.CloseHandleRequest;

    (*CloseHandleRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*CloseHandleRequest).Handle = hHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepCloseHandle),
        (core::mem::size_of::<CONSOLE_CLOSEHANDLE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn GetStdHandle(nStdHandle: DWORD) -> HANDLE {
    let Ppb: PRTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;
    let mut Handle: HANDLE = INVALID_HANDLE_VALUE;

    match nStdHandle {
        STD_INPUT_HANDLE => {
            Handle = (*Ppb).StandardInput;
        }
        STD_OUTPUT_HANDLE => {
            Handle = (*Ppb).StandardOutput;
        }
        STD_ERROR_HANDLE => {
            Handle = (*Ppb).StandardError;
        }
        _ => {}
    }

    if Handle == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_INVALID_HANDLE);
    }

    Handle
}
#[no_mangle]
pub unsafe extern "system" fn SetStdHandle(
    nStdHandle: DWORD,
    hHandle: HANDLE,
) -> BOOL {
    let Ppb: PRTL_USER_PROCESS_PARAMETERS = (*NtCurrentPeb()).ProcessParameters;

    if nStdHandle == STD_INPUT_HANDLE {
        (*Ppb).StandardInput = hHandle;
        return 1;
    } else if nStdHandle == STD_OUTPUT_HANDLE {
        (*Ppb).StandardOutput = hHandle;
        return 1;
    } else if nStdHandle == STD_ERROR_HANDLE {
        (*Ppb).StandardError = hHandle;
        return 1;
    }

    SetLastError(ERROR_INVALID_HANDLE);
    return 0;
}
unsafe fn IntAllocConsole(
    Title: LPWSTR,
    TitleLength: DWORD,
    Desktop: LPWSTR,
    DesktopLength: DWORD,
    CurDir: LPWSTR,
    CurDirLength: DWORD,
    AppName: LPWSTR,
    AppNameLength: DWORD,
    CtrlRoutine: LPTHREAD_START_ROUTINE,
    PropRoutine: LPTHREAD_START_ROUTINE,
    ConsoleStartInfo: PCONSOLE_START_INFO,
) -> BOOL {
    let mut Success: BOOL = 1;
    let Status: NTSTATUS;

    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let AllocConsoleRequest: PCONSOLE_ALLOCCONSOLE = &mut ApiMessage.Data.AllocConsoleRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    (*AllocConsoleRequest).CtrlRoutine = CtrlRoutine;
    (*AllocConsoleRequest).PropRoutine = PropRoutine;

    'quit: {
        CaptureBuffer = CsrAllocateCaptureBuffer(
            5,
            TitleLength + DesktopLength + CurDirLength + AppNameLength
                + (core::mem::size_of::<CONSOLE_START_INFO>() as ULONG) as u32,
        );
        if CaptureBuffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            Success = 0;
            break 'quit;
        }

        CsrCaptureMessageBuffer(
            CaptureBuffer,
            ConsoleStartInfo as *const _ as *mut _,
            (core::mem::size_of::<CONSOLE_START_INFO>() as ULONG) as u32,
            &mut (*AllocConsoleRequest).ConsoleStartInfo as *mut _ as *mut *mut _,
        );

        (*AllocConsoleRequest).TitleLength = TitleLength;
        CsrCaptureMessageBuffer(
            CaptureBuffer,
            Title as *const _ as *mut _,
            TitleLength,
            &mut (*AllocConsoleRequest).ConsoleTitle as *mut _ as *mut *mut _,
        );

        (*AllocConsoleRequest).DesktopLength = DesktopLength;
        CsrCaptureMessageBuffer(
            CaptureBuffer,
            Desktop as *const _ as *mut _,
            DesktopLength,
            &mut (*AllocConsoleRequest).Desktop as *mut _ as *mut *mut _,
        );

        (*AllocConsoleRequest).CurDirLength = CurDirLength;
        CsrCaptureMessageBuffer(
            CaptureBuffer,
            CurDir as *const _ as *mut _,
            CurDirLength,
            &mut (*AllocConsoleRequest).CurDir as *mut _ as *mut *mut _,
        );

        (*AllocConsoleRequest).AppNameLength = AppNameLength;
        CsrCaptureMessageBuffer(
            CaptureBuffer,
            AppName as *const _ as *mut _,
            AppNameLength,
            &mut (*AllocConsoleRequest).AppName as *mut _ as *mut *mut _,
        );

        CsrClientCallServer(
            &mut ApiMessage as *mut _ as *mut _,
            CaptureBuffer,
            CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepAlloc),
            core::mem::size_of_val(&*AllocConsoleRequest) as u32,
        );
        if !NT_SUCCESS(ApiMessage.Status) {
            BaseSetLastNTError(ApiMessage.Status);
            Success = 0;
            break 'quit;
        }

        Status = NtWaitForMultipleObjects(
            MAX_INIT_EVENTS,
            (*(*AllocConsoleRequest).ConsoleStartInfo).InitEvents.as_ptr() as *mut _,
            1, // WaitAny
            0, // FALSE
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Success = 0;
            break 'quit;
        }

        NtClose((*(*AllocConsoleRequest).ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize]);
        NtClose((*(*AllocConsoleRequest).ConsoleStartInfo).InitEvents[INIT_FAILURE as usize]);
        if Status != INIT_SUCCESS as NTSTATUS {
            (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle = core::ptr::null_mut();
            Success = 0;
        } else {
            core::ptr::copy_nonoverlapping(
                (*AllocConsoleRequest).ConsoleStartInfo as *const _,
                ConsoleStartInfo as *mut _,
                1,
            );
            Success = 1;
        }
    }

    if !CaptureBuffer.is_null() {
        CsrFreeCaptureBuffer(CaptureBuffer);
    }
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn AllocConsole() -> BOOL {
    let mut Success: BOOL;
    let mut ConsoleStartInfo: CONSOLE_START_INFO = core::mem::zeroed();

    let mut ConsoleTitle: PWCHAR = core::ptr::null_mut();
    let mut Desktop: PWCHAR = core::ptr::null_mut();
    let mut AppName: PWCHAR = core::ptr::null_mut();
    let mut CurDir: PWCHAR = core::ptr::null_mut();

    let mut TitleLength: ULONG = (MAX_PATH + 1) * (core::mem::size_of::<WCHAR>() as ULONG) as ULONG;
    let mut DesktopLength: ULONG = (MAX_PATH + 1) * (core::mem::size_of::<WCHAR>() as ULONG) as ULONG;
    let mut AppNameLength: ULONG = 128 * (core::mem::size_of::<WCHAR>() as ULONG) as ULONG;
    let mut CurDirLength: ULONG = (MAX_PATH + 1) * (core::mem::size_of::<WCHAR>() as ULONG) as ULONG;

    RtlEnterCriticalSection(&raw mut ConsoleLock);

    'quit: {
        if !(*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).ConsoleHandle.is_null() {
            // DPRINT1("AllocConsole: Allocating a console to a process already having one\n");
            SetLastError(ERROR_ACCESS_DENIED);
            Success = 0;
            break 'quit;
        }

        /* Set up the console properties */
        SetUpConsoleInfo(
            0,
            &mut TitleLength,
            &mut ConsoleTitle,
            &mut DesktopLength,
            &mut Desktop,
            &mut ConsoleStartInfo,
        );
        // DPRINT("ConsoleTitle = '%S' - Desktop = '%S'\n", ConsoleTitle, Desktop);

        /* Initialize the Input EXE name */
        InitExeName();
        SetUpAppName(
            0,
            &mut CurDirLength,
            &mut CurDir,
            &mut AppNameLength,
            &mut AppName,
        );
        // DPRINT("CurDir = '%S' - AppName = '%S'\n", CurDir, AppName);

        Success = IntAllocConsole(
            ConsoleTitle,
            TitleLength,
            Desktop,
            DesktopLength,
            CurDir,
            CurDirLength,
            AppName,
            AppNameLength,
            core::mem::transmute::<unsafe extern "system" fn(LPVOID) -> DWORD, LPTHREAD_START_ROUTINE>(ConsoleControlDispatcher),
            core::mem::transmute::<unsafe extern "system" fn(LPVOID) -> DWORD, LPTHREAD_START_ROUTINE>(PropDialogHandler),
            &mut ConsoleStartInfo,
        );
        if Success != 0 {
            /* Set up the handles */
            SetUpHandles(&ConsoleStartInfo);
            InputWaitHandle = ConsoleStartInfo.InputWaitHandle;

            /* Initialize Console Ctrl Handling */
            InitializeCtrlHandling();

            /* Sync the current thread's LangId with the console's one */
            SetTEBLangID();
        }
    }

    RtlLeaveCriticalSection(&raw mut ConsoleLock);
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn FreeConsole() -> BOOL {
    let mut Success: BOOL = 1;
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let FreeConsoleRequest: *mut CONSOLE_FREECONSOLE = &mut ApiMessage.Data.FreeConsoleRequest;
    let ConsoleHandle: HANDLE = (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleHandle;

    RtlEnterCriticalSection(&raw mut ConsoleLock);

    'quit: {
        /* We must have a non-trivial handle to close */
        if ConsoleHandle.is_null() // IsConsoleHandle(ConsoleHandle)
        {
            SetLastError(ERROR_INVALID_PARAMETER);
            Success = 0;
            break 'quit;
        }

        /* Set up the data to send to the Console Server */
        (*FreeConsoleRequest).ConsoleHandle = ConsoleHandle;

        /* Call the server */
        CsrClientCallServer(
            &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepFree),
            (core::mem::size_of::<CONSOLE_FREECONSOLE>() as ULONG) as u32
        );

        /* Check for success */
        if !NT_SUCCESS(ApiMessage.Status)
        {
            BaseSetLastNTError(ApiMessage.Status);
            Success = 0;
            break 'quit;
        }

        /* Reset the console handle */
        (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleHandle = core::ptr::null_mut();

        /* Close the associated input handle */
        CloseHandle(InputWaitHandle);
        InputWaitHandle = INVALID_HANDLE_VALUE;
    }

    RtlLeaveCriticalSection(&raw mut ConsoleLock);
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleScreenBufferInfo(
    hConsoleOutput: HANDLE,
    lpConsoleScreenBufferInfo: PCONSOLE_SCREEN_BUFFER_INFO,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ScreenBufferInfoRequest: PCONSOLE_GETSCREENBUFFERINFO =
        &mut ApiMessage.Data.ScreenBufferInfoRequest;

    if lpConsoleScreenBufferInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*ScreenBufferInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*ScreenBufferInfoRequest).OutputHandle = hConsoleOutput;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetScreenBufferInfo),
        (core::mem::size_of::<CONSOLE_GETSCREENBUFFERINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    (*lpConsoleScreenBufferInfo).dwSize = (*ScreenBufferInfoRequest).ScreenBufferSize;
    (*lpConsoleScreenBufferInfo).dwCursorPosition = (*ScreenBufferInfoRequest).CursorPosition;
    (*lpConsoleScreenBufferInfo).wAttributes = (*ScreenBufferInfoRequest).Attributes;
    (*lpConsoleScreenBufferInfo).srWindow.Left = (*ScreenBufferInfoRequest).ViewOrigin.X;
    (*lpConsoleScreenBufferInfo).srWindow.Top = (*ScreenBufferInfoRequest).ViewOrigin.Y;
    (*lpConsoleScreenBufferInfo).srWindow.Right = (*ScreenBufferInfoRequest).ViewOrigin.X
        + (*ScreenBufferInfoRequest).ViewSize.X
        - 1;
    (*lpConsoleScreenBufferInfo).srWindow.Bottom = (*ScreenBufferInfoRequest).ViewOrigin.Y
        + (*ScreenBufferInfoRequest).ViewSize.Y
        - 1;
    (*lpConsoleScreenBufferInfo).dwMaximumWindowSize = (*ScreenBufferInfoRequest).MaximumViewSize;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCursorPosition(
    hConsoleOutput: HANDLE,
    dwCursorPosition: COORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetCursorPositionRequest: PCONSOLE_SETCURSORPOSITION =
        &mut ApiMessage.Data.SetCursorPositionRequest;

    (*SetCursorPositionRequest).ConsoleHandle =
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetCursorPositionRequest).OutputHandle = hConsoleOutput;
    (*SetCursorPositionRequest).Position = dwCursorPosition;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCursorPosition),
        (core::mem::size_of::<CONSOLE_SETCURSORPOSITION>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleMode(
    hConsoleHandle: HANDLE,
    lpMode: LPDWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ConsoleModeRequest: PCONSOLE_GETSETCONSOLEMODE = &mut ApiMessage.Data.ConsoleModeRequest;

    if lpMode.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*ConsoleModeRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*ConsoleModeRequest).Handle = hConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetMode),
        (core::mem::size_of::<CONSOLE_GETSETCONSOLEMODE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    *lpMode = (*ConsoleModeRequest).Mode;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleMode(
    hConsoleHandle: HANDLE,
    dwMode: DWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ConsoleModeRequest: PCONSOLE_GETSETCONSOLEMODE = &mut ApiMessage.Data.ConsoleModeRequest;

    (*ConsoleModeRequest).ConsoleHandle = (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleHandle;
    (*ConsoleModeRequest).Handle = hConsoleHandle;
    (*ConsoleModeRequest).Mode = dwMode;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetMode),
        (core::mem::size_of::<CONSOLE_GETSETCONSOLEMODE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0i32;
    }

    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumberOfConsoleInputEvents(
    hConsoleInput: HANDLE,
    lpNumberOfEvents: LPDWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetNumInputEventsRequest: PCONSOLE_GETNUMINPUTEVENTS =
        &mut ApiMessage.Data.GetNumInputEventsRequest;

    (*GetNumInputEventsRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetNumInputEventsRequest).InputHandle = hConsoleInput;
    (*GetNumInputEventsRequest).NumberOfEvents = 0;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetNumberOfInputEvents),
        (core::mem::size_of::<CONSOLE_GETNUMINPUTEVENTS>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    if lpNumberOfEvents.is_null() {
        SetLastError(ERROR_INVALID_ACCESS);
        return 0;
    }

    *lpNumberOfEvents = (*GetNumInputEventsRequest).NumberOfEvents;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetLargestConsoleWindowSize(
    hConsoleOutput: HANDLE,
) -> COORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetLargestWindowSizeRequest: PCONSOLE_GETLARGESTWINDOWSIZE =
        &mut ApiMessage.Data.GetLargestWindowSizeRequest;

    (*GetLargestWindowSizeRequest).ConsoleHandle =
        (*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).ConsoleHandle;
    (*GetLargestWindowSizeRequest).OutputHandle = hConsoleOutput;
    (*GetLargestWindowSizeRequest).Size.X = 0;
    (*GetLargestWindowSizeRequest).Size.Y = 0;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetLargestWindowSize),
        (core::mem::size_of::<CONSOLE_GETLARGESTWINDOWSIZE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
    }

    return (*GetLargestWindowSizeRequest).Size;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCursorInfo(
    hConsoleOutput: HANDLE,
    lpConsoleCursorInfo: PCONSOLE_CURSOR_INFO,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let CursorInfoRequest: PCONSOLE_GETSETCURSORINFO = &mut ApiMessage.Data.CursorInfoRequest;

    if lpConsoleCursorInfo.is_null() {
        if hConsoleOutput.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
        } else {
            SetLastError(ERROR_INVALID_ACCESS);
        }
        return 0;
    }

    (*CursorInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*CursorInfoRequest).OutputHandle = hConsoleOutput;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCursorInfo),
        (core::mem::size_of::<PCONSOLE_GETSETCURSORINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    *lpConsoleCursorInfo = (*CursorInfoRequest).Info;

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCursorInfo(
    hConsoleOutput: HANDLE,
    lpConsoleCursorInfo: *const CONSOLE_CURSOR_INFO,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let CursorInfoRequest: PCONSOLE_GETSETCURSORINFO = &mut ApiMessage.Data.CursorInfoRequest;

    (*CursorInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*CursorInfoRequest).OutputHandle = hConsoleOutput;
    (*CursorInfoRequest).Info = *lpConsoleCursorInfo;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCursorInfo),
        (core::mem::size_of::<CONSOLE_GETSETCURSORINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNumberOfConsoleMouseButtons(
    lpNumberOfMouseButtons: LPDWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetMouseInfoRequest: PCONSOLE_GETMOUSEINFO =
        &mut ApiMessage.Data.GetMouseInfoRequest;

    (*GetMouseInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetMouseInfo),
        (core::mem::size_of::<CONSOLE_GETMOUSEINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    *lpNumberOfMouseButtons = (*GetMouseInfoRequest).NumButtons;
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleActiveScreenBuffer(
    hConsoleOutput: HANDLE,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetScreenBufferRequest: PCONSOLE_SETACTIVESCREENBUFFER =
        &mut ApiMessage.Data.SetScreenBufferRequest;

    (*SetScreenBufferRequest).ConsoleHandle =
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetScreenBufferRequest).OutputHandle = hConsoleOutput;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetActiveScreenBuffer),
        (core::mem::size_of::<CONSOLE_SETACTIVESCREENBUFFER>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn FlushConsoleInputBuffer(
    hConsoleInput: HANDLE,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let FlushInputBufferRequest: PCONSOLE_FLUSHINPUTBUFFER =
        &mut ApiMessage.Data.FlushInputBufferRequest;

    (*FlushInputBufferRequest).ConsoleHandle =
        (*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).ConsoleHandle;
    (*FlushInputBufferRequest).InputHandle = hConsoleInput;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepFlushInputBuffer),
        (core::mem::size_of::<CONSOLE_FLUSHINPUTBUFFER>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0i32;
    }

    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleScreenBufferSize(
    hConsoleOutput: HANDLE,
    dwSize: COORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetScreenBufferSizeRequest: PCONSOLE_SETSCREENBUFFERSIZE =
        &mut ApiMessage.Data.SetScreenBufferSizeRequest;

    (*SetScreenBufferSizeRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetScreenBufferSizeRequest).OutputHandle = hConsoleOutput;
    (*SetScreenBufferSizeRequest).Size = dwSize;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetScreenBufferSize),
        (core::mem::size_of::<CONSOLE_SETSCREENBUFFERSIZE>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}
unsafe fn IntScrollConsoleScreenBuffer(
    hConsoleOutput: HANDLE,
    lpScrollRectangle: *const SMALL_RECT,
    lpClipRectangle: *const SMALL_RECT,
    dwDestinationOrigin: COORD,
    lpFill: *const CHAR_INFO,
    bUnicode: BOOL,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let ScrollScreenBufferRequest: PCONSOLE_SCROLLSCREENBUFFER =
        &mut ApiMessage.Data.ScrollScreenBufferRequest;

    (*ScrollScreenBufferRequest).ConsoleHandle =
        (*(*(NtCurrentPeb())).ProcessParameters).ConsoleHandle;
    (*ScrollScreenBufferRequest).OutputHandle = hConsoleOutput;
    (*ScrollScreenBufferRequest).ScrollRectangle = *lpScrollRectangle;

    if !lpClipRectangle.is_null() {
        (*ScrollScreenBufferRequest).UseClipRectangle = 1;
        (*ScrollScreenBufferRequest).ClipRectangle = *lpClipRectangle;
    } else {
        (*ScrollScreenBufferRequest).UseClipRectangle = 0;
    }

    (*ScrollScreenBufferRequest).DestinationOrigin = dwDestinationOrigin;
    (*ScrollScreenBufferRequest).Fill = *lpFill;
    (*ScrollScreenBufferRequest).Unicode = bUnicode as BOOLEAN;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepScrollScreenBuffer),
        (core::mem::size_of::<CONSOLE_SCROLLSCREENBUFFER>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn ScrollConsoleScreenBufferA(
    hConsoleOutput: HANDLE,
    lpScrollRectangle: *const SMALL_RECT,
    lpClipRectangle: *const SMALL_RECT,
    dwDestinationOrigin: COORD,
    lpFill: *const CHAR_INFO,
) -> BOOL {
    return IntScrollConsoleScreenBuffer(
        hConsoleOutput,
        lpScrollRectangle,
        lpClipRectangle,
        dwDestinationOrigin,
        lpFill,
        0, // FALSE
    );
}

#[no_mangle]
pub unsafe extern "system" fn ScrollConsoleScreenBufferW(
    hConsoleOutput: HANDLE,
    lpScrollRectangle: *const SMALL_RECT,
    lpClipRectangle: *const SMALL_RECT,
    dwDestinationOrigin: COORD,
    lpFill: *const CHAR_INFO,
) -> BOOL {
    return IntScrollConsoleScreenBuffer(
        hConsoleOutput,
        lpScrollRectangle,
        lpClipRectangle,
        dwDestinationOrigin,
        lpFill,
        1, // TRUE
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleWindowInfo(
    hConsoleOutput: HANDLE,
    bAbsolute: BOOL,
    lpConsoleWindow: *const SMALL_RECT,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetWindowInfoRequest: PCONSOLE_SETWINDOWINFO = &mut ApiMessage.Data.SetWindowInfoRequest;

    if lpConsoleWindow.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*SetWindowInfoRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetWindowInfoRequest).OutputHandle = hConsoleOutput;
    (*SetWindowInfoRequest).Absolute = bAbsolute;
    (*SetWindowInfoRequest).WindowRect = *lpConsoleWindow;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetWindowInfo),
        (core::mem::size_of::<CONSOLE_SETWINDOWINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleTextAttribute(
    hConsoleOutput: HANDLE,
    wAttributes: WORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetTextAttribRequest: *mut CONSOLE_SETTEXTATTRIB = &mut ApiMessage.Data.SetTextAttribRequest;

    (*SetTextAttribRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetTextAttribRequest).OutputHandle = hConsoleOutput;
    (*SetTextAttribRequest).Attributes = wAttributes;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetTextAttribute),
        (core::mem::size_of::<CONSOLE_SETTEXTATTRIB>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0i32;
    }

    return 1i32;
}
unsafe fn AddConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE) -> BOOL {
    let mut NewCtrlHandlers: *mut PHANDLER_ROUTINE = core::ptr::null_mut();

    if HandlerRoutine.is_none() {
        (*(*NtCurrentPeb()).ProcessParameters).ConsoleFlags = 1;
        return 1;
    }

    if NrCtrlHandlers == NrAllocatedHandlers {
        NewCtrlHandlers = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((NrCtrlHandlers + 4) * (core::mem::size_of::<PHANDLER_ROUTINE>() as ULONG)) as SIZE_T,
        ) as *mut PHANDLER_ROUTINE;
        if NewCtrlHandlers.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }

        core::ptr::copy_nonoverlapping(
            CtrlHandlers,
            NewCtrlHandlers,
            (core::mem::size_of::<PHANDLER_ROUTINE>() * NrCtrlHandlers as usize) / core::mem::size_of::<PHANDLER_ROUTINE>(),
        );

        if NrAllocatedHandlers > 1 {
            RtlFreeHeap(RtlGetProcessHeap(), 0, CtrlHandlers as *mut core::ffi::c_void);
        }

        CtrlHandlers = NewCtrlHandlers;
        NrAllocatedHandlers += 4;
    }

    // ASSERT(NrCtrlHandlers < NrAllocatedHandlers);

    let idx = NrCtrlHandlers;
    NrCtrlHandlers += 1;
    *CtrlHandlers.add(idx as usize) = HandlerRoutine;
    return 1;
}
#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn RemoveConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE) -> BOOL {
    let mut i: ULONG;

    if HandlerRoutine.is_none() {
        (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleFlags = 0;
        return 1;
    }

    i = 0;
    while i < NrCtrlHandlers {
        if *CtrlHandlers.add(i as usize) == HandlerRoutine {
            if i < (NrCtrlHandlers - 1) {
                // C: memmove(&CtrlHandlers[i], &CtrlHandlers[i+1],
                //            (NrCtrlHandlers - i + 1) * sizeof(PHANDLER_ROUTINE));
                // ptr::copy counts ELEMENTS, not bytes — the count is the element
                // count (NrCtrlHandlers - i + 1); multiplying by size_of here would
                // copy 4x the elements = 16x the bytes = heap overflow.
                core::ptr::copy(
                    CtrlHandlers.offset((i + 1) as isize),
                    CtrlHandlers.offset(i as isize),
                    (NrCtrlHandlers - i + 1) as usize,
                );
            }

            NrCtrlHandlers -= 1;
            return 1;
        }
        i += 1;
    }

    SetLastError(ERROR_INVALID_PARAMETER);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCtrlHandler(
    HandlerRoutine: PHANDLER_ROUTINE,
    Add: BOOL,
) -> BOOL {
    let Ret: BOOL;

    RtlEnterCriticalSection(&raw mut ConsoleLock);

    if Add != 0 {
        Ret = AddConsoleCtrlHandler(HandlerRoutine);
    } else {
        Ret = RemoveConsoleCtrlHandler(HandlerRoutine);
    }

    RtlLeaveCriticalSection(&raw mut ConsoleLock);
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn GenerateConsoleCtrlEvent(
    dwCtrlEvent: DWORD,
    dwProcessGroupId: DWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GenerateCtrlEventRequest: PCONSOLE_GENERATECTRLEVENT =
        &mut ApiMessage.Data.GenerateCtrlEventRequest;

    if dwCtrlEvent != CTRL_C_EVENT && dwCtrlEvent != CTRL_BREAK_EVENT {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*GenerateCtrlEventRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GenerateCtrlEventRequest).CtrlEvent = dwCtrlEvent;
    (*GenerateCtrlEventRequest).ProcessGroupId = dwProcessGroupId;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGenerateCtrlEvent),
        (core::mem::size_of::<CONSOLE_GENERATECTRLEVENT>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}
unsafe fn IntGetConsoleTitle(
    lpConsoleTitle: LPVOID,
    mut dwNumChars: DWORD,
    bUnicode: BOOLEAN,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let TitleRequest: PCONSOLE_GETSETCONSOLETITLE = &mut ApiMessage.Data.TitleRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    if dwNumChars == 0 {
        return 0;
    }

    (*TitleRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*TitleRequest).Length = dwNumChars * (if bUnicode != 0 { (core::mem::size_of::<WCHAR>() as ULONG) } else { (core::mem::size_of::<CHAR>() as ULONG) }) as DWORD;
    (*TitleRequest).Unicode = bUnicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, (*TitleRequest).Length);
    if CaptureBuffer.is_null() {
        // DPRINT removed
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrAllocateMessagePointer(
        CaptureBuffer,
        (*TitleRequest).Length,
        &mut (*TitleRequest).Title as *mut _ as *mut PVOID,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut _ as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetTitle),
        (core::mem::size_of::<CONSOLE_GETSETCONSOLETITLE>() as ULONG) as DWORD,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        CsrFreeCaptureBuffer(CaptureBuffer);
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    dwNumChars = (*TitleRequest).Length / (if bUnicode != 0 { (core::mem::size_of::<WCHAR>() as ULONG) } else { (core::mem::size_of::<CHAR>() as ULONG) }) as DWORD;

    if dwNumChars > 0 {
        RtlCopyMemory(
            lpConsoleTitle,
            (*TitleRequest).Title as *const _,
            (*TitleRequest).Length as SIZE_T,
        );

        if bUnicode != 0 {
            *((lpConsoleTitle as LPWSTR).offset(dwNumChars as isize)) = UNICODE_NULL;
        } else {
            *((lpConsoleTitle as LPSTR).offset(dwNumChars as isize)) = crate::console::cffi::ANSI_NULL;
        }
    }

    CsrFreeCaptureBuffer(CaptureBuffer);

    dwNumChars
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleTitleW(
    lpConsoleTitle: LPWSTR,
    nSize: DWORD,
) -> DWORD {
    return IntGetConsoleTitle(lpConsoleTitle as LPVOID, nSize, 1);
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleTitleA(
    lpConsoleTitle: LPSTR,
    nSize: DWORD,
) -> DWORD {
    return IntGetConsoleTitle(lpConsoleTitle as LPVOID, nSize, 0);
}
unsafe fn IntSetConsoleTitle(
    lpConsoleTitle: *const core::ffi::c_void,
    bUnicode: BOOLEAN,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let TitleRequest: PCONSOLE_GETSETCONSOLETITLE = &mut ApiMessage.Data.TitleRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    let NumChars: ULONG = if !lpConsoleTitle.is_null() {
        if bUnicode != 0 {
            wcslen(lpConsoleTitle as *const u16) as ULONG
        } else {
            strlen(lpConsoleTitle as *const i8) as ULONG
        }
    } else {
        0
    };

    (*TitleRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*TitleRequest).Length = NumChars * if bUnicode != 0 {
        (core::mem::size_of::<WCHAR>() as ULONG) as ULONG
    } else {
        (core::mem::size_of::<CHAR>() as ULONG) as ULONG
    };
    (*TitleRequest).Unicode = bUnicode;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, (*TitleRequest).Length);
    if CaptureBuffer.is_null() {
        // DPRINT removed
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        lpConsoleTitle as *mut core::ffi::c_void,
        (*TitleRequest).Length,
        &mut (*TitleRequest).Title as *mut _ as *mut *mut core::ffi::c_void,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut _ as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetTitle),
        (core::mem::size_of::<CONSOLE_GETSETCONSOLETITLE>() as ULONG) as ULONG,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleTitleW(
    lpConsoleTitle: LPCWSTR,
) -> BOOL {
    return IntSetConsoleTitle(lpConsoleTitle as *const core::ffi::c_void, 1);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleTitleA(
    lpConsoleTitle: LPCSTR,
) -> BOOL {
    return IntSetConsoleTitle(lpConsoleTitle as *const core::ffi::c_void, 0);
}

#[no_mangle]
pub unsafe extern "system" fn CreateConsoleScreenBuffer(
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    lpSecurityAttributes: *const SECURITY_ATTRIBUTES,
    dwFlags: DWORD,
    lpScreenBufferData: LPVOID,
) -> HANDLE {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let CreateScreenBufferRequest: PCONSOLE_CREATESCREENBUFFER = &mut ApiMessage.Data.CreateScreenBufferRequest;
    let mut CaptureBuffer: PCSR_CAPTURE_BUFFER = core::ptr::null_mut();
    let GraphicsBufferInfo: PCONSOLE_GRAPHICS_BUFFER_INFO = lpScreenBufferData as PCONSOLE_GRAPHICS_BUFFER_INFO;

    if (dwDesiredAccess & !(GENERIC_READ | GENERIC_WRITE)) != 0
        || (dwShareMode & !(FILE_SHARE_READ | FILE_SHARE_WRITE)) != 0
        || (dwFlags != CONSOLE_TEXTMODE_BUFFER && dwFlags != CONSOLE_GRAPHICS_BUFFER)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }

    (*CreateScreenBufferRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*CreateScreenBufferRequest).DesiredAccess = dwDesiredAccess;
    (*CreateScreenBufferRequest).InheritHandle = if !lpSecurityAttributes.is_null() {
        (*lpSecurityAttributes).bInheritHandle
    } else {
        0
    };
    (*CreateScreenBufferRequest).ShareMode = dwShareMode;
    (*CreateScreenBufferRequest).ScreenBufferType = dwFlags;

    if dwFlags == CONSOLE_GRAPHICS_BUFFER {
        if (*CreateScreenBufferRequest).InheritHandle != 0 || GraphicsBufferInfo.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return INVALID_HANDLE_VALUE;
        }

        (*CreateScreenBufferRequest).GraphicsBufferInfo = *GraphicsBufferInfo;

        CaptureBuffer = CsrAllocateCaptureBuffer(1, (*GraphicsBufferInfo).dwBitMapInfoLength);
        if CaptureBuffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return INVALID_HANDLE_VALUE;
        }

        CsrCaptureMessageBuffer(
            CaptureBuffer,
            (*GraphicsBufferInfo).lpBitMapInfo as *mut core::ffi::c_void,
            (*GraphicsBufferInfo).dwBitMapInfoLength,
            &mut (*CreateScreenBufferRequest).GraphicsBufferInfo.lpBitMapInfo as *mut _ as *mut *mut core::ffi::c_void,
        );
    }

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepCreateScreenBuffer),
        (core::mem::size_of::<CONSOLE_CREATESCREENBUFFER>() as ULONG) as u32,
    );

    if !CaptureBuffer.is_null() {
        CsrFreeCaptureBuffer(CaptureBuffer);
    }

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return INVALID_HANDLE_VALUE;
    }

    if dwFlags == CONSOLE_GRAPHICS_BUFFER && !GraphicsBufferInfo.is_null() {
        (*GraphicsBufferInfo).hMutex = (*CreateScreenBufferRequest).hMutex;
        (*GraphicsBufferInfo).lpBitMap = (*CreateScreenBufferRequest).lpBitMap;
    }

    (*CreateScreenBufferRequest).OutputHandle
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCP() -> u32 {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetConsoleCPRequest: *mut CONSOLE_GETINPUTOUTPUTCP = &mut ApiMessage.Data.GetConsoleCPRequest;

    /* Get the Input Code Page */
    (*GetConsoleCPRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetConsoleCPRequest).OutputCP = 0;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCP),
        (core::mem::size_of::<CONSOLE_GETINPUTOUTPUTCP>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return (*GetConsoleCPRequest).CodePage;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCP(wCodePageID: UINT) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetConsoleCPRequest: PCONSOLE_SETINPUTOUTPUTCP = &mut ApiMessage.Data.SetConsoleCPRequest;

    /* Set the Input Code Page */
    (*SetConsoleCPRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetConsoleCPRequest).CodePage = wCodePageID;
    (*SetConsoleCPRequest).OutputCP = 0; /* FALSE */
    /* SetConsoleCPRequest->EventHandle; */

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCP),
        (core::mem::size_of::<CONSOLE_SETINPUTOUTPUTCP>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0; /* FALSE */
    }

    return 1; /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleOutputCP() -> u32 {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetConsoleCPRequest: PCONSOLE_GETINPUTOUTPUTCP = &mut ApiMessage.Data.GetConsoleCPRequest;

    /* Get the Output Code Page */
    (*GetConsoleCPRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetConsoleCPRequest).OutputCP = 1i32;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCP),
        (core::mem::size_of::<CONSOLE_GETINPUTOUTPUTCP>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return (*GetConsoleCPRequest).CodePage;
}
#[no_mangle]
pub unsafe extern "system" fn SetConsoleOutputCP(wCodePageID: UINT) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetConsoleCPRequest: PCONSOLE_SETINPUTOUTPUTCP = &mut ApiMessage.Data.SetConsoleCPRequest;

    /* Set the Output Code Page */
    (*SetConsoleCPRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetConsoleCPRequest).CodePage = wCodePageID;
    (*SetConsoleCPRequest).OutputCP = 1i32; /* TRUE */
    /* SetConsoleCPRequest->EventHandle; */

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCP),
        (core::mem::size_of::<CONSOLE_SETINPUTOUTPUTCP>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status)
    {
        BaseSetLastNTError(ApiMessage.Status);
        return 0i32; /* FALSE */
    }

    /* Sync the current thread's LangId with the console's one */
    SetTEBLangID();

    return 1i32; /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleProcessList(
    lpdwProcessList: LPDWORD,
    dwProcessCount: DWORD,
) -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetProcessListRequest: PCONSOLE_GETPROCESSLIST = &mut ApiMessage.Data.GetProcessListRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;
    let mut nProcesses: ULONG = 0;

    if lpdwProcessList.is_null() || dwProcessCount == 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    CaptureBuffer = CsrAllocateCaptureBuffer(1, dwProcessCount * (core::mem::size_of::<DWORD>() as ULONG) as ULONG);
    if CaptureBuffer.is_null() {
        // DPRINT removed
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*GetProcessListRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetProcessListRequest).ProcessCount = dwProcessCount;

    CsrAllocateMessagePointer(
        CaptureBuffer,
        dwProcessCount * (core::mem::size_of::<DWORD>() as ULONG) as ULONG,
        &mut (*GetProcessListRequest).ProcessIdsList as *mut _ as *mut *mut core::ffi::c_void,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetProcessList),
        (core::mem::size_of::<CONSOLE_GETPROCESSLIST>() as ULONG) as ULONG,
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
    } else {
        nProcesses = (*GetProcessListRequest).ProcessCount;
        if dwProcessCount >= nProcesses {
            RtlCopyMemory(
                lpdwProcessList as *mut core::ffi::c_void,
                (*GetProcessListRequest).ProcessIdsList as *const core::ffi::c_void,
                (nProcesses * (core::mem::size_of::<DWORD>() as ULONG)) as SIZE_T,
            );
        }
    }

    CsrFreeCaptureBuffer(CaptureBuffer);
    return nProcesses;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleSelectionInfo(
    lpConsoleSelectionInfo: PCONSOLE_SELECTION_INFO,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetSelectionInfoRequest: PCONSOLE_GETSELECTIONINFO =
        &mut ApiMessage.Data.GetSelectionInfoRequest;

    if lpConsoleSelectionInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    (*GetSelectionInfoRequest).ConsoleHandle =
        (*(*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessParameters).ConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetSelectionInfo),
        (core::mem::size_of::<CONSOLE_GETSELECTIONINFO>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    *lpConsoleSelectionInfo = (*GetSelectionInfoRequest).Info;

    return 1;
}
unsafe fn IntAttachConsole(
    ProcessId: DWORD,
    CtrlRoutine: LPTHREAD_START_ROUTINE,
    PropRoutine: LPTHREAD_START_ROUTINE,
    ConsoleStartInfo: PCONSOLE_START_INFO,
) -> BOOL {
    let mut Success: BOOL = 1;
    let mut Status: NTSTATUS;

    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let AttachConsoleRequest: PCONSOLE_ATTACHCONSOLE = &mut ApiMessage.Data.AttachConsoleRequest;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    (*AttachConsoleRequest).ProcessId = ProcessId;
    (*AttachConsoleRequest).CtrlRoutine = CtrlRoutine;
    (*AttachConsoleRequest).PropRoutine = PropRoutine;

    'quit: {
        CaptureBuffer = CsrAllocateCaptureBuffer(1, (core::mem::size_of::<CONSOLE_START_INFO>() as ULONG));
        if CaptureBuffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            Success = 0;
            break 'quit;
        }

        CsrCaptureMessageBuffer(
            CaptureBuffer,
            ConsoleStartInfo as *const _ as *mut _,
            (core::mem::size_of::<CONSOLE_START_INFO>() as ULONG),
            &mut (*AttachConsoleRequest).ConsoleStartInfo as *mut _ as *mut *mut _,
        );

        CsrClientCallServer(
            &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
            CaptureBuffer,
            CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepAttach),
            core::mem::size_of_val(&*AttachConsoleRequest) as u32,
        );
        if !NT_SUCCESS(ApiMessage.Status) {
            BaseSetLastNTError(ApiMessage.Status);
            Success = 0;
            break 'quit;
        }

        Status = NtWaitForMultipleObjects(
            MAX_INIT_EVENTS,
            (*(*AttachConsoleRequest).ConsoleStartInfo).InitEvents.as_ptr() as *mut _,
            1, // WaitAny
            0, // FALSE
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Success = 0;
            break 'quit;
        }

        NtClose((*(*AttachConsoleRequest).ConsoleStartInfo).InitEvents[INIT_SUCCESS as usize]);
        NtClose((*(*AttachConsoleRequest).ConsoleStartInfo).InitEvents[INIT_FAILURE as usize]);
        if Status != INIT_SUCCESS as NTSTATUS {
            (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle = core::ptr::null_mut();
            Success = 0;
        } else {
            RtlCopyMemory(
                ConsoleStartInfo as *mut _,
                (*AttachConsoleRequest).ConsoleStartInfo as *const _ as *mut _,
                (core::mem::size_of::<CONSOLE_START_INFO>() as ULONG) as SIZE_T,
            );
            Success = 1;
        }
    }

    if !CaptureBuffer.is_null() {
        CsrFreeCaptureBuffer(CaptureBuffer);
    }
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn AttachConsole(dwProcessId: DWORD) -> BOOL {
    let mut Success: BOOL;
    let mut ConsoleStartInfo: CONSOLE_START_INFO = core::mem::zeroed();
    let mut dummy: DWORD = 0;

    RtlEnterCriticalSection(&raw mut ConsoleLock);

    'quit: {
        if !(*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle.is_null() {
            SetLastError(ERROR_ACCESS_DENIED);
            Success = 0;
            break 'quit;
        }

        /* Set up the console properties */
        SetUpConsoleInfo(
            0,
            &mut dummy as *mut DWORD,
            core::ptr::null_mut(),
            &mut dummy as *mut DWORD,
            core::ptr::null_mut(),
            &mut ConsoleStartInfo as *mut CONSOLE_START_INFO,
        );

        Success = IntAttachConsole(
            dwProcessId,
            core::mem::transmute::<unsafe extern "system" fn(LPVOID) -> DWORD, LPTHREAD_START_ROUTINE>(ConsoleControlDispatcher),
            core::mem::transmute::<unsafe extern "system" fn(LPVOID) -> DWORD, LPTHREAD_START_ROUTINE>(PropDialogHandler),
            &mut ConsoleStartInfo as *mut CONSOLE_START_INFO,
        );
        if Success != 0 {
            /* Set up the handles */
            SetUpHandles(&ConsoleStartInfo as *const CONSOLE_START_INFO);
            InputWaitHandle = ConsoleStartInfo.InputWaitHandle;

            /* Initialize Console Ctrl Handling */
            InitializeCtrlHandling();

            /* Sync the current thread's LangId with the console's one */
            SetTEBLangID();
        }
    }

    RtlLeaveCriticalSection(&raw mut ConsoleLock);
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleWindow() -> HWND {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetWindowRequest: PCONSOLE_GETWINDOW = &mut ApiMessage.Data.GetWindowRequest;

    (*GetWindowRequest).ConsoleHandle = (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetConsoleWindow),
        (core::mem::size_of::<CONSOLE_GETWINDOW>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return core::ptr::null_mut();
    }

    return (*GetWindowRequest).WindowHandle as _;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleIcon(hIcon: HICON) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let SetIconRequest: PCONSOLE_SETICON = &mut ApiMessage.Data.SetIconRequest;

    (*SetIconRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*SetIconRequest).IconHandle = hIcon as _;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetIcon),
        (core::mem::size_of::<CONSOLE_SETICON>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0i32;
    }

    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleInputExeNameW(lpExeName: LPCWSTR) -> BOOL {
    let ExeLength: DWORD;
    let mut _SEH2_ret: BOOL = 0;

    ExeLength = lstrlenW(lpExeName) as DWORD;
    if (ExeLength == 0) || (ExeLength >= EXENAME_LENGTH as DWORD) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    RtlEnterCriticalSection(&raw mut ExeNameLock);
    // _SEH2_TRY (DUMMY_PSEH: _SEH2_EXCEPT handler is dead code; _SEH2_FINALLY always runs)
    {
        RtlCopyMemory(
            (*(&raw mut ExeNameBuffer)).as_mut_ptr() as *mut core::ffi::c_void,
            lpExeName as *const core::ffi::c_void,
            (ExeLength * (core::mem::size_of::<WCHAR>() as ULONG) as DWORD) as SIZE_T,
        );
        ExeNameLength = ExeLength as USHORT;
        _SEH2_ret = 1;
    }
    // _SEH2_FINALLY
    {
        RtlLeaveCriticalSection(&raw mut ExeNameLock);
    }
    return _SEH2_ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleInputExeNameA(lpExeName: LPCSTR) -> BOOL {
    let mut Status: NTSTATUS;
    let mut ExeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ExeNameA: ANSI_STRING = core::mem::zeroed();
    let mut Buffer: [WCHAR; EXENAME_LENGTH as usize] = [0; EXENAME_LENGTH as usize];

    RtlInitAnsiString(&mut ExeNameA, lpExeName);

    if (ExeNameA.Length == 0) || (ExeNameA.Length as u32 >= EXENAME_LENGTH as u32) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    ExeNameU.Length = 0;
    ExeNameU.MaximumLength = core::mem::size_of::<[WCHAR; EXENAME_LENGTH as usize]>() as USHORT;
    ExeNameU.Buffer = Buffer.as_mut_ptr();

    Status = RtlAnsiStringToUnicodeString(&mut ExeNameU, &mut ExeNameA, 0);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_BUFFER_OVERFLOW {
            SetLastError(ERROR_FILENAME_EXCED_RANGE);
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
        }
        return 0;
    }

    return SetConsoleInputExeNameW(ExeNameU.Buffer);
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleInputExeNameW(
    nBufferLength: DWORD,
    lpExeName: LPWSTR,
) -> DWORD {
    if nBufferLength <= ExeNameLength as DWORD {
        SetLastError(ERROR_BUFFER_OVERFLOW);
        return (ExeNameLength + 1) as DWORD;
    }

    RtlEnterCriticalSection(&raw mut ExeNameLock);
    // _SEH2_TRY
    {
        // Copy the input EXE name and NULL-terminate it
        RtlCopyMemory(
            lpExeName as *mut core::ffi::c_void,
            (*(&raw const ExeNameBuffer)).as_ptr() as *const core::ffi::c_void,
            (ExeNameLength as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
        );
        *lpExeName.offset(ExeNameLength as isize) = 0u16; // UNICODE_NULL
    }
    // _SEH2_FINALLY
    {
        RtlLeaveCriticalSection(&raw mut ExeNameLock);
    }
    // _SEH2_END

    // console.c returns TRUE. This returned 0, which GetConsoleInputExeNameA reads as
    // `ExeLength == 0` and bails on before converting, so the A export never filled its buffer.
    return TRUE as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleInputExeNameA(
    nBufferLength: DWORD,
    lpExeName: LPSTR,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut ExeLength: DWORD;
    let mut BufferU: _UNICODE_STRING = core::mem::zeroed();
    let mut BufferA: ANSI_STRING = core::mem::zeroed();
    let mut Buffer: [WCHAR; EXENAME_LENGTH as usize] = [0; EXENAME_LENGTH as usize];

    /* Get the UNICODE name */
    ExeLength = GetConsoleInputExeNameW(EXENAME_LENGTH, Buffer.as_mut_ptr());

    if (ExeLength == 0) || (ExeLength >= EXENAME_LENGTH) {
        return ExeLength;
    }

    /* Initialize the strings for conversion */
    RtlInitUnicodeString(&mut BufferU, Buffer.as_ptr());
    BufferA.Length = 0;
    BufferA.MaximumLength = nBufferLength as USHORT;
    BufferA.Buffer = lpExeName;

    /* Convert UNICODE name to ANSI, copying as much chars as it can fit */
    Status = RtlUnicodeStringToAnsiString(&mut BufferA, &BufferU, 0);
    if !NT_SUCCESS(Status) {
        if Status == STATUS_BUFFER_OVERFLOW {
            SetLastError(ERROR_BUFFER_OVERFLOW);
            return ExeLength + 1;
        }
        SetLastError(ERROR_INVALID_PARAMETER);
    }

    return ExeLength;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCharType(
    hConsole: HANDLE,
    Coord: COORD,
    Type: PDWORD,
) -> BOOL {
    STUB;
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleCursorMode(
    hConsole: HANDLE,
    pUnknown1: PBOOL,
    pUnknown2: PBOOL,
) -> BOOL {
    // STUB
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCursorMode(
    hConsole: HANDLE,
    Unknown1: BOOL,
    Unknown2: BOOL,
) -> BOOL {
    // STUB;
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleNlsMode(
    hConsole: HANDLE,
    lpMode: LPDWORD,
) -> BOOL {
    // STUB
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleNlsMode(
    hConsole: HANDLE,
    dwMode: DWORD,
) -> BOOL {
    // STUB
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleLocalEUDC(
    hConsoleHandle: HANDLE,
    wCodePoint: WORD,
    cFontSize: COORD,
    lpSB: PCHAR,
) -> BOOL {
    // STUB;
    0
}
unsafe fn IntRegisterConsoleIME(
    hWnd: HWND,
    dwThreadId: DWORD,
    mut cbDesktop: SIZE_T,
    mut pDesktop: PWSTR,
    pdwAttachToThreadId: PDWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let RegisterConsoleIME: PCONSOLE_REGISTERCONSOLEIME = &mut ApiMessage.Data.RegisterConsoleIME;
    let CaptureBuffer: PCSR_CAPTURE_BUFFER;

    if cbDesktop == 0 || pDesktop.is_null() {
        pDesktop = (*(*NtCurrentPeb()).ProcessParameters).DesktopInfo.Buffer;
        cbDesktop = (*(*NtCurrentPeb()).ProcessParameters).DesktopInfo.Length as SIZE_T;
    }

    cbDesktop = min(cbDesktop, ((MAX_PATH + 1) * (core::mem::size_of::<WCHAR>() as ULONG)) as SIZE_T);

    (*RegisterConsoleIME).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*RegisterConsoleIME).hWnd = hWnd as _;
    (*RegisterConsoleIME).dwThreadId = dwThreadId;
    (*RegisterConsoleIME).cbDesktop = cbDesktop as DWORD;

    CaptureBuffer = CsrAllocateCaptureBuffer(1, cbDesktop as ULONG);
    if CaptureBuffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    CsrCaptureMessageBuffer(
        CaptureBuffer,
        pDesktop as *mut core::ffi::c_void,
        cbDesktop as ULONG,
        &mut (*RegisterConsoleIME).pDesktop as *mut _ as *mut *mut _,
    );

    CsrClientCallServer(
        &mut ApiMessage as *mut _ as *mut _,
        CaptureBuffer,
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepRegisterConsoleIME),
        core::mem::size_of_val(&*RegisterConsoleIME) as u32,
    );

    CsrFreeCaptureBuffer(CaptureBuffer);

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    if !pdwAttachToThreadId.is_null() {
        let __try_result: i32;
        let __exception_code: u32;
        __try_result = {
            *pdwAttachToThreadId = (*RegisterConsoleIME).dwAttachToThreadId;
            0
        };
        if __try_result != 0 {
            __exception_code = __try_result as u32;
            loop {
                if __exception_code == EXCEPTION_EXECUTE_HANDLER {
                    BaseSetLastNTError(STATUS_ACCESS_VIOLATION);
                    return 0;
                }
                break;
            }
        }
    }

    return 1;
}
unsafe fn IntUnregisterConsoleIME(
    dwThreadId: DWORD,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let UnregisterConsoleIME: PCONSOLE_UNREGISTERCONSOLEIME = &mut ApiMessage.Data.UnregisterConsoleIME;

    (*UnregisterConsoleIME).ConsoleHandle = (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleHandle;
    (*UnregisterConsoleIME).dwThreadId = dwThreadId;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepUnregisterConsoleIME),
        (core::mem::size_of::<CONSOLE_UNREGISTERCONSOLEIME>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterConsoleIME(
    hWnd: HWND,
    pdwAttachToThreadId: LPDWORD,
) -> BOOL {
    return IntRegisterConsoleIME(
        hWnd,
        GetCurrentThreadId(),
        0,
        core::ptr::null_mut(),
        pdwAttachToThreadId,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RegisterConsoleOS2(bUnknown: BOOL) -> BOOL {
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleOS2OemFormat(bUnknown: BOOL) -> BOOL {
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterConsoleIME() -> BOOL {
    return IntUnregisterConsoleIME(GetCurrentThreadId());
}
#[no_mangle]
pub unsafe extern "C" fn SetTEBLangID() {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let LangIdRequest: PCONSOLE_GETLANGID = &mut ApiMessage.Data.LangIdRequest;

    (*LangIdRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetLangId),
        (core::mem::size_of::<CONSOLE_GETLANGID>() as ULONG),
    );
    if !NT_SUCCESS(ApiMessage.Status) {
        return;
    }

    if SetThreadLocale(MAKELCID((*LangIdRequest).LangId, SORT_DEFAULT)) == 0 {
        // DPRINT removed
    }
}
unsafe fn IntGetConsoleKeyboardLayoutName(
    pszLayoutName: *mut core::ffi::c_void,
    bAnsi: BOOL,
) -> BOOL {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let GetKbdLayoutNameRequest: *mut CONSOLE_GETKBDLAYOUTNAME =
        &mut ApiMessage.Data.GetKbdLayoutNameRequest;

    (*GetKbdLayoutNameRequest).ConsoleHandle = (*(*NtCurrentPeb()).ProcessParameters).ConsoleHandle;
    (*GetKbdLayoutNameRequest).Ansi = bAnsi;

    CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as *mut CSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetKeyboardLayoutName),
        (core::mem::size_of::<CONSOLE_GETKBDLAYOUTNAME>() as ULONG) as u32,
    );

    if !NT_SUCCESS(ApiMessage.Status) {
        BaseSetLastNTError(ApiMessage.Status);
        return 0;
    }

    let __try_result: i32;
    __try_result = {
        if bAnsi != 0 {
            strncpy(
                pszLayoutName as *mut i8,
                (*GetKbdLayoutNameRequest).LayoutBuffer.as_ptr() as *const i8,
                KL_NAMELENGTH as usize,
            );
        } else {
            wcsncpy(
                pszLayoutName as *mut u16,
                (*GetKbdLayoutNameRequest).LayoutBuffer.as_ptr() as *const u16,
                KL_NAMELENGTH as usize,
            );
        }
        0
    };
    if __try_result != 0 {
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleKeyboardLayoutNameA(
    pszLayoutName: LPSTR,
) -> BOOL {
    IntGetConsoleKeyboardLayoutName(pszLayoutName as *mut core::ffi::c_void, 1)
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleKeyboardLayoutNameW(
    pszLayoutName: LPWSTR,
) -> BOOL {
    return IntGetConsoleKeyboardLayoutName(pszLayoutName as *mut core::ffi::c_void, 0);
}

#[no_mangle]
pub unsafe extern "system" fn SetLastConsoleEventActive() -> DWORD {
    let mut ApiMessage: CONSOLE_API_MESSAGE = core::mem::zeroed();
    let NotifyLastCloseRequest: PCONSOLE_NOTIFYLASTCLOSE = &mut ApiMessage.Data.NotifyLastCloseRequest;

    /* Set the flag used by the console control dispatcher */
    LastCloseNotify = 1; /* TRUE */

    /* Set up the input arguments */
    (*NotifyLastCloseRequest).ConsoleHandle = (*(*((*NtCurrentTeb()).ProcessEnvironmentBlock)).ProcessParameters).ConsoleHandle;

    /* Call CSRSS; just return the NTSTATUS cast to DWORD */
    return CsrClientCallServer(
        &mut ApiMessage as *mut CONSOLE_API_MESSAGE as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepNotifyLastClose),
        (core::mem::size_of::<CONSOLE_NOTIFYLASTCLOSE>() as ULONG)
    ) as DWORD;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/console/console.c
//  * PURPOSE:         Win32 server console functions
//  * PROGRAMMERS:     James Tabor <jimtabor@adsl-64-217-116-74.dsl.hstntx.swbell.net>
//  *                  Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ********************************************************************/
//
// extern RTL_CRITICAL_SECTION ConsoleLock;
// extern BOOLEAN ConsoleInitialized;
//
// /* Console reserved "file" names */
// static LPCWSTR BaseConFileName       = CONSOLE_FILE_NAME;
// static LPCWSTR BaseConInputFileName  = CONSOLE_INPUT_FILE_NAME;
// static LPCWSTR BaseConOutputFileName = CONSOLE_OUTPUT_FILE_NAME;
//
// /* Console Control handling */
// static PHANDLER_ROUTINE InitialHandler[1];
// static PHANDLER_ROUTINE* CtrlHandlers;
// static ULONG NrCtrlHandlers;
// static ULONG NrAllocatedHandlers;
// static BOOLEAN LastCloseNotify = FALSE;
//
// extern BOOL WINAPI IsDebuggerPresent(VOID);
//
// /* Console Input facilities */
// HANDLE InputWaitHandle = INVALID_HANDLE_VALUE;
//
// #define EXENAME_LENGTH (255 + 1)
// static RTL_CRITICAL_SECTION ExeNameLock;
// static BOOLEAN ExeNameInitialized;
// static WCHAR ExeNameBuffer[EXENAME_LENGTH]; // NULL-terminated
// static USHORT ExeNameLength;    // Count in number of characters without NULL
// static WCHAR StartDirBuffer[MAX_PATH + 1];  // NULL-terminated
// static USHORT StartDirLength;   // Count in number of characters without NULL
//
//
// /* Default Console Control Handler ********************************************/
//
// static BOOL
// WINAPI
// DefaultConsoleCtrlHandler(DWORD Event)
// {
//     DPRINT("Default handler called: %lx\n", Event);
//     switch(Event)
//     {
//         case CTRL_C_EVENT:
//             DPRINT("Ctrl-C Event\n");
//             break;
//
//         case CTRL_BREAK_EVENT:
//             DPRINT("Ctrl-Break Event\n");
//             break;
//
//         case CTRL_CLOSE_EVENT:
//             DPRINT("Ctrl Close Event\n");
//             break;
//
//         case CTRL_LAST_CLOSE_EVENT:
//             DPRINT("Ctrl Last Close Event\n");
//             break;
//
//         case CTRL_LOGOFF_EVENT:
//             DPRINT("Ctrl Logoff Event\n");
//             break;
//
//         case CTRL_SHUTDOWN_EVENT:
//             DPRINT("Ctrl Shutdown Event\n");
//             break;
//     }
//
//     ExitProcess(CONTROL_C_EXIT as UINT);
//     return TRUE;
// }
//
// DWORD
// WINAPI
// ConsoleControlDispatcher(IN LPVOID lpThreadParameter)
// {
//     DWORD nExitCode = 0;
//     DWORD CodeAndFlag = PtrToUlong(lpThreadParameter);
//     DWORD nCode = CodeAndFlag & MAXLONG;
//     UINT i;
//     EXCEPTION_RECORD erException;
//
//     DPRINT1("Console Dispatcher Active: %lx %lx\n", CodeAndFlag, nCode);
//     SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST as i32);
//
//     switch(nCode)
//     {
//         case CTRL_C_EVENT:
//         case CTRL_BREAK_EVENT:
//         {
//             if (IsDebuggerPresent())
//             {
//                 erException.ExceptionCode = (nCode == CTRL_C_EVENT ?
//                                              DBG_CONTROL_C : DBG_CONTROL_BREAK);
//                 erException.ExceptionFlags = 0;
//                 erException.ExceptionRecord = NULL;
//                 erException.ExceptionAddress = DefaultConsoleCtrlHandler;
//                 erException.NumberParameters = 0;
//
//                 _SEH2_TRY
//                 {
//                     RtlRaiseException(&erException);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     RtlEnterCriticalSection(&mut ConsoleLock);
//
//                     if ((nCode != CTRL_C_EVENT) ||
//                         (NtCurrentPeb()->ProcessParameters->ConsoleFlags != 1))
//                     {
//                         for (i = NrCtrlHandlers; i > 0; i--)
//                         {
//                             if (CtrlHandlers[i - 1](nCode)) break;
//                         }
//                     }
//
//                     RtlLeaveCriticalSection(&mut ConsoleLock);
//                 }
//                 _SEH2_END;
//
//                 ExitThread(0);
//             }
//             break;
//         }
//
//         case CTRL_CLOSE_EVENT:
//         case CTRL_LOGOFF_EVENT:
//         case CTRL_SHUTDOWN_EVENT:
//             break;
//
//         case CTRL_LAST_CLOSE_EVENT:
//             /*
//              * In case the console app hasn't register for last close notification,
//              * just kill this console handler thread. We don't want that such apps
//              * get killed for unexpected reasons. On the contrary apps that registered
//              * can be killed because they expect to be.
//              */
//             if (!LastCloseNotify) ExitThread(0);
//             break;
//
//         case 4:
//             ExitProcess(CONTROL_C_EXIT as UINT);
//             break;
//
//         default:
//             ASSERT(FALSE);
//             break;
//     }
//
//     ASSERT(ConsoleInitialized);
//
//     RtlEnterCriticalSection(&mut ConsoleLock);
//
//     nExitCode = 0;
//     if ((nCode != CTRL_C_EVENT) || (NtCurrentPeb()->ProcessParameters->ConsoleFlags != 1))
//     {
//         for (i = NrCtrlHandlers; i > 0; i--)
//         {
//             if ((i == 1) &&
//                 (CodeAndFlag & MINLONG) &&
//                 ((nCode == CTRL_LOGOFF_EVENT) || (nCode == CTRL_SHUTDOWN_EVENT)))
//             {
//                 DPRINT("Skipping system/service apps\n");
//                 break;
//             }
//
//             if (CtrlHandlers[i - 1](nCode))
//             {
//                 switch(nCode)
//                 {
//                     case CTRL_CLOSE_EVENT:
//                     case CTRL_LAST_CLOSE_EVENT:
//                     case CTRL_LOGOFF_EVENT:
//                     case CTRL_SHUTDOWN_EVENT:
//                         nExitCode = CodeAndFlag;
//                         break;
//                 }
//                 break;
//             }
//         }
//     }
//
//     RtlLeaveCriticalSection(&mut ConsoleLock);
//
//     ExitThread(nExitCode);
//     return STATUS_SUCCESS;
// }
//
// VOID
// InitializeCtrlHandling(VOID)
// {
//     /* Initialize Console Ctrl Handler */
//     NrAllocatedHandlers = NrCtrlHandlers = 1;
//     CtrlHandlers = InitialHandler;
//     CtrlHandlers[0] = DefaultConsoleCtrlHandler;
// }
//
//
// /* Input EXE Name Support *****************************************************/
//
// VOID
// InitExeName(VOID)
// {
//     NTSTATUS Status;
//     PPEB Peb = NtCurrentPeb();
//     PCURDIR CurrentDirectory = &Peb->ProcessParameters->CurrentDirectory;
//     PLDR_DATA_TABLE_ENTRY ImageEntry;
//
//     if (ExeNameInitialized) return;
//
//     /* Initialize the EXE name lock */
//     Status = RtlInitializeCriticalSection(&ExeNameLock);
//     if (!NT_SUCCESS(Status)) return;
//     ExeNameInitialized = TRUE;
//
//     ImageEntry = CONTAINING_RECORD(Peb->Ldr->InLoadOrderModuleList.Flink,
//                                    LDR_DATA_TABLE_ENTRY,
//                                    InLoadOrderLinks);
//
//     /* Retrieve the EXE name, NULL-terminate it... */
//     ExeNameLength = min(sizeof(ExeNameBuffer)/sizeof(ExeNameBuffer[0]),
//                         ImageEntry->BaseDllName.Length / sizeof(WCHAR));
//     RtlCopyMemory(ExeNameBuffer,
//                   ImageEntry->BaseDllName.Buffer,
//                   ImageEntry->BaseDllName.Length);
//     ExeNameBuffer[ExeNameLength] = UNICODE_NULL;
//
//     /* ... and retrieve the current directory path and NULL-terminate it. */
//     StartDirLength = min(sizeof(StartDirBuffer)/sizeof(StartDirBuffer[0]),
//                          CurrentDirectory->DosPath.Length / sizeof(WCHAR));
//     RtlCopyMemory(StartDirBuffer,
//                   CurrentDirectory->DosPath.Buffer,
//                   CurrentDirectory->DosPath.Length);
//     StartDirBuffer[StartDirLength] = UNICODE_NULL;
// }
//
// /*
//  * NOTE:
//  * The "LPDWORD Length" parameters point on input to the maximum size of
//  * the buffers that can hold data (if != 0), and on output they hold the
//  * real size of the data. If "Length" are == 0 on input, then on output
//  * they receive the full size of the data.
//  * The "LPWSTR* String" parameters have a double meaning:
//  * - when "CaptureStrings" is TRUE, data is copied to the buffers pointed
//  *   by the pointers (*String).
//  * - when "CaptureStrings" is FALSE, "*String" are set to the addresses of
//  *   the source data.
//  */
// VOID
// SetUpAppName(IN BOOLEAN CaptureStrings,
//              IN OUT LPDWORD CurDirLength,
//              IN OUT LPWSTR* CurDir,
//              IN OUT LPDWORD AppNameLength,
//              IN OUT LPWSTR* AppName)
// {
//     DWORD Length;
//
//     /* Retrieve the needed buffer size */
//     Length = (StartDirLength + 1) * sizeof(WCHAR);
//     if (*CurDirLength > 0) Length = min(Length, *CurDirLength);
//     *CurDirLength = Length;
//
//     /* Capture the data if needed, or, return a pointer to it */
//     if (CaptureStrings)
//     {
//         /*
//          * Length is always >= sizeof(WCHAR). Copy everything but the
//          * possible trailing NULL character, and then NULL-terminate.
//          */
//         Length -= sizeof(WCHAR);
//         RtlCopyMemory(*CurDir, StartDirBuffer, Length);
//         (*CurDir)[Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     else
//     {
//         *CurDir = StartDirBuffer;
//     }
//
//     /* Retrieve the needed buffer size */
//     Length = (ExeNameLength + 1) * sizeof(WCHAR);
//     if (*AppNameLength > 0) Length = min(Length, *AppNameLength);
//     *AppNameLength = Length;
//
//     /* Capture the data if needed, or, return a pointer to it */
//     if (CaptureStrings)
//     {
//         /*
//          * Length is always >= sizeof(WCHAR). Copy everything but the
//          * possible trailing NULL character, and then NULL-terminate.
//          */
//         Length -= sizeof(WCHAR);
//         RtlCopyMemory(*AppName, ExeNameBuffer, Length);
//         (*AppName)[Length / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//     else
//     {
//         *AppName = ExeNameBuffer;
//     }
// }
//
// USHORT
// GetCurrentExeName(OUT PWCHAR ExeName,
//                   IN USHORT BufferSize)
// {
//     USHORT ExeLength;
//
//     if (ExeNameInitialized)
//     {
//         RtlEnterCriticalSection(&ExeNameLock);
//
//         if (BufferSize > ExeNameLength * sizeof(WCHAR))
//             BufferSize = ExeNameLength * sizeof(WCHAR);
//
//         RtlCopyMemory(ExeName, ExeNameBuffer, BufferSize);
//
//         RtlLeaveCriticalSection(&ExeNameLock);
//         ExeLength = BufferSize;
//     }
//     else
//     {
//         *ExeName  = UNICODE_NULL;
//         ExeLength = 0;
//     }
//
//     return ExeLength;
// }
//
// /* FUNCTIONS ******************************************************************/
//
// LPCWSTR
// IntCheckForConsoleFileName(IN LPCWSTR pszName,
//                            IN DWORD dwDesiredAccess)
// {
//     LPCWSTR ConsoleName = pszName;
//     ULONG DeviceNameInfo;
//
//     /*
//      * Check whether we deal with a DOS device, and if so,
//      * strip the path till the file name.
//      * Therefore, things like \\.\CON or C:\some_path\CONIN$
//      * are transformed into CON or CONIN$, for example.
//      */
//     DeviceNameInfo = RtlIsDosDeviceName_U(pszName);
//     if (DeviceNameInfo != 0)
//     {
//         ConsoleName = (LPCWSTR)((ULONG_PTR)ConsoleName + ((DeviceNameInfo >> 16) & 0xFFFF));
//     }
//
//     /* Return a standard console "file" name according to what we passed in parameters */
//     if (_wcsicmp(ConsoleName, BaseConInputFileName) == 0)
//     {
//         return BaseConInputFileName;
//     }
//     else if (_wcsicmp(ConsoleName, BaseConOutputFileName) == 0)
//     {
//         return BaseConOutputFileName;
//     }
//     else if (_wcsicmp(ConsoleName, BaseConFileName) == 0)
//     {
//         if ((dwDesiredAccess & (GENERIC_READ | GENERIC_WRITE)) == GENERIC_READ)
//         {
//             return BaseConInputFileName;
//         }
//         else if ((dwDesiredAccess & (GENERIC_READ | GENERIC_WRITE)) == GENERIC_WRITE)
//         {
//             return BaseConOutputFileName;
//         }
//     }
//
//     /* If we are there, that means that either the file name or the desired access are wrong */
//     return NULL;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See http://undoc.airesoft.co.uk/kernel32.dll/ConsoleMenuControl.php
//  */
// HMENU
// WINAPI
// DECLSPEC_HOTPATCH
// ConsoleMenuControl(HANDLE hConsoleOutput,
//                    DWORD dwCmdIdLow,
//                    DWORD dwCmdIdHigh)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_MENUCONTROL MenuControlRequest = &ApiMessage.Data.MenuControlRequest;
//
//     MenuControlRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     MenuControlRequest->OutputHandle  = hConsoleOutput;
//     MenuControlRequest->CmdIdLow      = dwCmdIdLow;
//     MenuControlRequest->CmdIdHigh     = dwCmdIdHigh;
//     MenuControlRequest->MenuHandle    = NULL;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepMenuControl),
//                         sizeof(*MenuControlRequest));
//
//     return MenuControlRequest->MenuHandle;
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// DuplicateConsoleHandle(HANDLE hSourceHandle,
//                        DWORD dwDesiredAccess,
//                        BOOL bInheritHandle,
//                        DWORD dwOptions)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_DUPLICATEHANDLE DuplicateHandleRequest = &ApiMessage.Data.DuplicateHandleRequest;
//
//     if ( (dwOptions & ~(DUPLICATE_CLOSE_SOURCE | DUPLICATE_SAME_ACCESS)) ||
//          (!(dwOptions & DUPLICATE_SAME_ACCESS) &&
//            (dwDesiredAccess & ~(GENERIC_READ | GENERIC_WRITE))) )
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     DuplicateHandleRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     DuplicateHandleRequest->SourceHandle  = hSourceHandle;
//     DuplicateHandleRequest->DesiredAccess = dwDesiredAccess;
//     DuplicateHandleRequest->InheritHandle = bInheritHandle;
//     DuplicateHandleRequest->Options       = dwOptions;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepDuplicateHandle),
//                         sizeof(*DuplicateHandleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     return DuplicateHandleRequest->TargetHandle;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetConsoleHandleInformation(IN HANDLE hHandle,
//                             OUT LPDWORD lpdwFlags)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETHANDLEINFO GetHandleInfoRequest = &ApiMessage.Data.GetHandleInfoRequest;
//
//     GetHandleInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetHandleInfoRequest->Handle        = hHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetHandleInformation),
//                         sizeof(*GetHandleInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *lpdwFlags = GetHandleInfoRequest->Flags;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetConsoleHandleInformation(IN HANDLE hHandle,
//                             IN DWORD dwMask,
//                             IN DWORD dwFlags)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETHANDLEINFO SetHandleInfoRequest = &ApiMessage.Data.SetHandleInfoRequest;
//
//     SetHandleInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetHandleInfoRequest->Handle        = hHandle;
//     SetHandleInfoRequest->Mask          = dwMask;
//     SetHandleInfoRequest->Flags         = dwFlags;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetHandleInformation),
//                         sizeof(*SetHandleInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetConsoleDisplayMode(LPDWORD lpModeFlags)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETDISPLAYMODE GetDisplayModeRequest = &ApiMessage.Data.GetDisplayModeRequest;
//
//     if (lpModeFlags == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     GetDisplayModeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetDisplayMode),
//                         sizeof(*GetDisplayModeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *lpModeFlags = GetDisplayModeRequest->DisplayMode; // ModeFlags
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See https://cboard.cprogramming.com/windows-programming/102187-console-font-size.html
//  */
// DWORD
// WINAPI
// GetConsoleFontInfo(IN HANDLE hConsoleOutput,
//                    IN BOOL bMaximumWindow,
//                    IN DWORD nFontCount,
//                    OUT PCONSOLE_FONT_INFO lpConsoleFontInfo)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETFONTINFO GetFontInfoRequest = &ApiMessage.Data.GetFontInfoRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     GetFontInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetFontInfoRequest->OutputHandle  = hConsoleOutput;
//     GetFontInfoRequest->MaximumWindow = bMaximumWindow;
//     GetFontInfoRequest->NumFonts      = nFontCount;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, nFontCount * sizeof(CONSOLE_FONT_INFO));
//     if (CaptureBuffer == NULL)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               nFontCount * sizeof(CONSOLE_FONT_INFO),
//                               (PVOID*)&GetFontInfoRequest->FontInfo);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetFontInfo),
//                         sizeof(*GetFontInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//     }
//     else
//     {
//         RtlCopyMemory(lpConsoleFontInfo,
//                       GetFontInfoRequest->FontInfo,
//                       GetFontInfoRequest->NumFonts * sizeof(CONSOLE_FONT_INFO));
//     }
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//     return GetFontInfoRequest->NumFonts;
// }
//
//
// /*
//  * @implemented
//  */
// COORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleFontSize(IN HANDLE hConsoleOutput,
//                    IN DWORD nFont)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETFONTSIZE GetFontSizeRequest = &ApiMessage.Data.GetFontSizeRequest;
//     COORD Empty = {0, 0};
//
//     GetFontSizeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetFontSizeRequest->OutputHandle  = hConsoleOutput;
//     GetFontSizeRequest->FontIndex     = nFont;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetFontSize),
//                         sizeof(*GetFontSizeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return Empty;
//     }
//
//     return GetFontSizeRequest->FontSize;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// BOOL
// WINAPI
// GetConsoleHardwareState(HANDLE hConsoleOutput,
//                         PDWORD Flags,
//                         PDWORD State)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETHWSTATE HardwareStateRequest = &ApiMessage.Data.HardwareStateRequest;
//
//     DPRINT1("GetConsoleHardwareState(%lu, 0x%p) UNIMPLEMENTED!\n", Flags, State);
//
//     if (Flags == NULL || State == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     HardwareStateRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     HardwareStateRequest->OutputHandle  = hConsoleOutput;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetHardwareState),
//                         sizeof(*HardwareStateRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *Flags = HardwareStateRequest->Flags;
//     *State = HardwareStateRequest->State;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// HANDLE
// WINAPI
// GetConsoleInputWaitHandle(VOID)
// {
//     return InputWaitHandle;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetCurrentConsoleFont(IN HANDLE hConsoleOutput,
//                       IN BOOL bMaximumWindow,
//                       OUT PCONSOLE_FONT_INFO lpConsoleCurrentFont)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETCURRENTFONT GetCurrentFontRequest = &ApiMessage.Data.GetCurrentFontRequest;
//
//     GetCurrentFontRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetCurrentFontRequest->OutputHandle  = hConsoleOutput;
//     GetCurrentFontRequest->MaximumWindow = bMaximumWindow;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCurrentFont),
//                         sizeof(*GetCurrentFontRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     lpConsoleCurrentFont->dwFontSize = GetCurrentFontRequest->FontSize;
//     lpConsoleCurrentFont->nFont      = GetCurrentFontRequest->FontIndex;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See https://cboard.cprogramming.com/windows-programming/102187-console-font-size.html
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetNumberOfConsoleFonts(VOID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETNUMFONTS GetNumFontsRequest = &ApiMessage.Data.GetNumFontsRequest;
//
//     GetNumFontsRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetNumberOfFonts),
//                         sizeof(*GetNumFontsRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     return GetNumFontsRequest->NumFonts;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See http://blog.airesoft.co.uk/2012/10/things-ms-can-do-that-they-dont-tell-you-about-console-graphics/
//  */
// BOOL
// WINAPI
// InvalidateConsoleDIBits(IN HANDLE hConsoleOutput,
//                         IN PSMALL_RECT lpRect)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_INVALIDATEDIBITS InvalidateDIBitsRequest = &ApiMessage.Data.InvalidateDIBitsRequest;
//
//     if (lpRect == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     InvalidateDIBitsRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     InvalidateDIBitsRequest->OutputHandle  = hConsoleOutput;
//     InvalidateDIBitsRequest->Region        = *lpRect;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepInvalidateBitMapRect),
//                         sizeof(*InvalidateDIBitsRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// HANDLE
// WINAPI
// OpenConsoleW(LPCWSTR wsName,
//              DWORD dwDesiredAccess,
//              BOOL bInheritHandle,
//              DWORD dwShareMode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_OPENCONSOLE OpenConsoleRequest = &ApiMessage.Data.OpenConsoleRequest;
//     CONSOLE_HANDLE_TYPE HandleType;
//
//     if (wsName && (_wcsicmp(wsName, BaseConInputFileName) == 0))
//     {
//         HandleType = HANDLE_INPUT;
//     }
//     else if (wsName && (_wcsicmp(wsName, BaseConOutputFileName) == 0))
//     {
//         HandleType = HANDLE_OUTPUT;
//     }
//     else
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     if ( (dwDesiredAccess & ~(GENERIC_READ | GENERIC_WRITE)) ||
//          (dwShareMode & ~(FILE_SHARE_READ | FILE_SHARE_WRITE)) )
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     OpenConsoleRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     OpenConsoleRequest->HandleType    = HandleType;
//     OpenConsoleRequest->DesiredAccess = dwDesiredAccess;
//     OpenConsoleRequest->InheritHandle = bInheritHandle;
//     OpenConsoleRequest->ShareMode     = dwShareMode;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepOpenConsole),
//                         sizeof(*OpenConsoleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     return OpenConsoleRequest->Handle;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See http://undoc.airesoft.co.uk/kernel32.dll/SetConsoleCursor.php
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleCursor(HANDLE  hConsoleOutput,
//                  HCURSOR hCursor)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETCURSOR SetCursorRequest = &ApiMessage.Data.SetCursorRequest;
//
//     SetCursorRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetCursorRequest->OutputHandle  = hConsoleOutput;
//     SetCursorRequest->CursorHandle  = hCursor;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCursor),
//                         sizeof(*SetCursorRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetConsoleDisplayMode(HANDLE hConsoleOutput,
//                       DWORD  dwFlags, // dwModeFlags
//                       PCOORD lpNewScreenBufferDimensions)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETDISPLAYMODE SetDisplayModeRequest = &ApiMessage.Data.SetDisplayModeRequest;
//
//     SetDisplayModeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetDisplayModeRequest->OutputHandle  = hConsoleOutput;
//     SetDisplayModeRequest->DisplayMode   = dwFlags; // ModeFlags ; dwModeFlags
//     SetDisplayModeRequest->NewSBDim.X    = 0;
//     SetDisplayModeRequest->NewSBDim.Y    = 0;
//     /* SetDisplayModeRequest->EventHandle; */
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetDisplayMode),
//                         sizeof(*SetDisplayModeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     if (lpNewScreenBufferDimensions)
//         *lpNewScreenBufferDimensions = SetDisplayModeRequest->NewSBDim;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See https://cboard.cprogramming.com/windows-programming/102187-console-font-size.html
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleFont(IN HANDLE hConsoleOutput,
//                IN DWORD nFont)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETFONT SetFontRequest = &ApiMessage.Data.SetFontRequest;
//
//     SetFontRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetFontRequest->OutputHandle  = hConsoleOutput;
//     SetFontRequest->FontIndex     = nFont;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetFont),
//                         sizeof(*SetFontRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// BOOL
// WINAPI
// SetConsoleHardwareState(HANDLE hConsoleOutput,
//                         DWORD Flags,
//                         DWORD State)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETHWSTATE HardwareStateRequest = &ApiMessage.Data.HardwareStateRequest;
//
//     DPRINT1("SetConsoleHardwareState(%lu, %lu) UNIMPLEMENTED!\n", Flags, State);
//
//     HardwareStateRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     HardwareStateRequest->OutputHandle  = hConsoleOutput;
//     HardwareStateRequest->Flags         = Flags;
//     HardwareStateRequest->State         = State;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetHardwareState),
//                         sizeof(*HardwareStateRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented (Undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleKeyShortcuts(
//      BOOL bSet,
//      BYTE bReserveKeys,
//      LPAPPKEY lpAppKeys,
//      DWORD dwNumAppKeys)
// {
//     DPRINT1("SetConsoleKeyShortcuts(%lu, 0x%x, 0x%p, 0x%x) UNIMPLEMENTED!\n",
//             bSet, bReserveKeys, lpAppKeys, dwNumAppKeys);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See http://undoc.airesoft.co.uk/kernel32.dll/SetConsoleMaximumWindowSize.php
//  *       Does nothing, returns TRUE only. Checked on Windows Server 2003.
//  */
// BOOL
// WINAPI
// SetConsoleMaximumWindowSize(HANDLE hConsoleOutput,
//                             COORD dwMaximumSize)
// {
//     DPRINT1("SetConsoleMaximumWindowSize(0x%p, {%d, %d}) does nothing\n",
//             hConsoleOutput, dwMaximumSize.X, dwMaximumSize.Y);
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleMenuClose(BOOL bEnable)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETMENUCLOSE SetMenuCloseRequest = &ApiMessage.Data.SetMenuCloseRequest;
//
//     SetMenuCloseRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetMenuCloseRequest->Enable        = bEnable;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetMenuClose),
//                         sizeof(*SetMenuCloseRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  * @note See http://comments.gmane.org/gmane.comp.lang.harbour.devel/27844 (DEAD_LINK)
//  *       Usage example: https://github.com/harbour/core/commit/d79a1b7b812cbde6ddf718ebfd6939a24f633e52
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsolePalette(HANDLE hConsoleOutput,
//                   HPALETTE hPalette,
//                   UINT dwUsage)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETPALETTE SetPaletteRequest = &ApiMessage.Data.SetPaletteRequest;
//
//     SetPaletteRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetPaletteRequest->OutputHandle  = hConsoleOutput;
//     SetPaletteRequest->PaletteHandle = hPalette;
//     SetPaletteRequest->Usage         = dwUsage;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetPalette),
//                         sizeof(*SetPaletteRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented (Undocumented)
//  * @note See http://undoc.airesoft.co.uk/kernel32.dll/ShowConsoleCursor.php
//  */
// INT
// WINAPI
// DECLSPEC_HOTPATCH
// ShowConsoleCursor(HANDLE hConsoleOutput,
//                   BOOL bShow)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SHOWCURSOR ShowCursorRequest = &ApiMessage.Data.ShowCursorRequest;
//
//     ShowCursorRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ShowCursorRequest->OutputHandle  = hConsoleOutput;
//     ShowCursorRequest->Show          = bShow;
//     ShowCursorRequest->RefCount      = 0;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepShowCursor),
//                         sizeof(*ShowCursorRequest));
//
//     return ShowCursorRequest->RefCount;
// }
//
//
// /*
//  * FUNCTION: Checks whether the given handle is a valid console handle.
//  *
//  * ARGUMENTS:
//  *      hIoHandle - Handle to be checked.
//  *
//  * RETURNS:
//  *      TRUE : Handle is a valid console handle.
//  *      FALSE: Handle is not a valid console handle.
//  *
//  * STATUS: Officially undocumented
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// VerifyConsoleIoHandle(HANDLE hIoHandle)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_VERIFYHANDLE VerifyHandleRequest = &ApiMessage.Data.VerifyHandleRequest;
//
//     VerifyHandleRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     VerifyHandleRequest->Handle        = hIoHandle;
//     VerifyHandleRequest->IsValid       = FALSE;
//
//     /* If the process is not attached to a console, return invalid handle */
//     if (VerifyHandleRequest->ConsoleHandle == NULL) return FALSE;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepVerifyIoHandle),
//                         sizeof(*VerifyHandleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return VerifyHandleRequest->IsValid;
// }
//
//
// /*
//  * @implemented (Undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// CloseConsoleHandle(HANDLE hHandle)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_CLOSEHANDLE CloseHandleRequest = &ApiMessage.Data.CloseHandleRequest;
//
//     CloseHandleRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     CloseHandleRequest->Handle        = hHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepCloseHandle),
//                         sizeof(*CloseHandleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// GetStdHandle(DWORD nStdHandle)
// /*
//  * FUNCTION: Get a handle for the standard input, standard output
//  * and a standard error device.
//  *
//  * ARGUMENTS:
//  *       nStdHandle - Specifies the device for which to return the handle.
//  *
//  * RETURNS: If the function succeeds, the return value is the handle
//  * of the specified device. Otherwise the value is INVALID_HANDLE_VALUE.
//  */
// {
//     PRTL_USER_PROCESS_PARAMETERS Ppb = NtCurrentPeb()->ProcessParameters;
//     HANDLE Handle = INVALID_HANDLE_VALUE;
//
//     switch (nStdHandle)
//     {
//         case STD_INPUT_HANDLE:
//             Handle = Ppb->StandardInput;
//             break;
//
//         case STD_OUTPUT_HANDLE:
//             Handle = Ppb->StandardOutput;
//             break;
//
//         case STD_ERROR_HANDLE:
//             Handle = Ppb->StandardError;
//             break;
//     }
//
//     /* If the returned handle is invalid, set last error */
//     if (Handle == INVALID_HANDLE_VALUE) SetLastError(ERROR_INVALID_HANDLE);
//
//     return Handle;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetStdHandle(DWORD  nStdHandle,
//              HANDLE hHandle)
// /*
//  * FUNCTION: Set the handle for the standard input, standard output or
//  * the standard error device.
//  *
//  * ARGUMENTS:
//  *        nStdHandle - Specifies the handle to be set.
//  *        hHandle - The handle to set.
//  *
//  * RETURNS: TRUE if the function succeeds, FALSE otherwise.
//  */
// {
//     PRTL_USER_PROCESS_PARAMETERS Ppb = NtCurrentPeb()->ProcessParameters;
//
//     /* No need to check if hHandle == INVALID_HANDLE_VALUE */
//
//     switch (nStdHandle)
//     {
//         case STD_INPUT_HANDLE:
//             Ppb->StandardInput = hHandle;
//             return TRUE;
//
//         case STD_OUTPUT_HANDLE:
//             Ppb->StandardOutput = hHandle;
//             return TRUE;
//
//         case STD_ERROR_HANDLE:
//             Ppb->StandardError = hHandle;
//             return TRUE;
//     }
//
//     /* nStdHandle was invalid, bail out */
//     SetLastError(ERROR_INVALID_HANDLE);
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// static BOOL
// IntAllocConsole(LPWSTR Title,
//                 DWORD TitleLength,
//                 LPWSTR Desktop,
//                 DWORD DesktopLength,
//                 LPWSTR CurDir,
//                 DWORD CurDirLength,
//                 LPWSTR AppName,
//                 DWORD AppNameLength,
//                 LPTHREAD_START_ROUTINE CtrlRoutine,
//                 LPTHREAD_START_ROUTINE PropRoutine,
//                 PCONSOLE_START_INFO ConsoleStartInfo)
// {
//     BOOL Success = TRUE;
//     NTSTATUS Status;
//
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_ALLOCCONSOLE AllocConsoleRequest = &ApiMessage.Data.AllocConsoleRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     AllocConsoleRequest->CtrlRoutine = CtrlRoutine;
//     AllocConsoleRequest->PropRoutine = PropRoutine;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(5, TitleLength   +
//                                                 DesktopLength +
//                                                 CurDirLength  +
//                                                 AppNameLength +
//                                                 sizeof(CONSOLE_START_INFO));
//     if (CaptureBuffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             ConsoleStartInfo,
//                             sizeof(CONSOLE_START_INFO),
//                             (PVOID*)&AllocConsoleRequest->ConsoleStartInfo);
//
//     AllocConsoleRequest->TitleLength = TitleLength;
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             Title,
//                             TitleLength,
//                             (PVOID*)&AllocConsoleRequest->ConsoleTitle);
//
//     AllocConsoleRequest->DesktopLength = DesktopLength;
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             Desktop,
//                             DesktopLength,
//                             (PVOID*)&AllocConsoleRequest->Desktop);
//
//     AllocConsoleRequest->CurDirLength = CurDirLength;
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             CurDir,
//                             CurDirLength,
//                             (PVOID*)&AllocConsoleRequest->CurDir);
//
//     AllocConsoleRequest->AppNameLength = AppNameLength;
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             AppName,
//                             AppNameLength,
//                             (PVOID*)&AllocConsoleRequest->AppName);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepAlloc),
//                         sizeof(*AllocConsoleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     // Is AllocConsoleRequest->ConsoleStartInfo->InitEvents aligned on handle boundary ????
//     Status = NtWaitForMultipleObjects(MAX_INIT_EVENTS,
//                                       AllocConsoleRequest->ConsoleStartInfo->InitEvents,
//                                       WaitAny, FALSE, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     NtClose(AllocConsoleRequest->ConsoleStartInfo->InitEvents[INIT_SUCCESS]);
//     NtClose(AllocConsoleRequest->ConsoleStartInfo->InitEvents[INIT_FAILURE]);
//     if (Status != INIT_SUCCESS)
//     {
//         NtCurrentPeb()->ProcessParameters->ConsoleHandle = NULL;
//         Success = FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(ConsoleStartInfo,
//                       AllocConsoleRequest->ConsoleStartInfo,
//                       sizeof(CONSOLE_START_INFO));
//         Success = TRUE;
//     }
//
// Quit:
//     if (CaptureBuffer) CsrFreeCaptureBuffer(CaptureBuffer);
//     return Success;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// AllocConsole(VOID)
// {
//     BOOL Success;
//     CONSOLE_START_INFO ConsoleStartInfo;
//
//     PWCHAR ConsoleTitle;
//     PWCHAR Desktop;
//     PWCHAR AppName;
//     PWCHAR CurDir;
//
//     ULONG TitleLength   = (MAX_PATH + 1) * sizeof(WCHAR);
//     ULONG DesktopLength = (MAX_PATH + 1) * sizeof(WCHAR);
//     ULONG AppNameLength = 128 * sizeof(WCHAR);
//     ULONG CurDirLength  = (MAX_PATH + 1) * sizeof(WCHAR);
//
//     RtlEnterCriticalSection(&mut ConsoleLock);
//
//     if (NtCurrentPeb()->ProcessParameters->ConsoleHandle)
//     {
//         DPRINT1("AllocConsole: Allocating a console to a process already having one\n");
//         SetLastError(ERROR_ACCESS_DENIED);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Set up the console properties */
//     SetUpConsoleInfo(FALSE,
//                      &TitleLength,
//                      &ConsoleTitle,
//                      &DesktopLength,
//                      &Desktop,
//                      &ConsoleStartInfo);
//     DPRINT("ConsoleTitle = '%S' - Desktop = '%S'\n",
//            ConsoleTitle, Desktop);
//
//     /* Initialize the Input EXE name */
//     InitExeName();
//     SetUpAppName(FALSE,
//                  &CurDirLength,
//                  &CurDir,
//                  &AppNameLength,
//                  &AppName);
//     DPRINT("CurDir = '%S' - AppName = '%S'\n",
//            CurDir, AppName);
//
//     Success = IntAllocConsole(ConsoleTitle,
//                               TitleLength,
//                               Desktop,
//                               DesktopLength,
//                               CurDir,
//                               CurDirLength,
//                               AppName,
//                               AppNameLength,
//                               ConsoleControlDispatcher,
//                               PropDialogHandler,
//                               &ConsoleStartInfo);
//     if (Success)
//     {
//         /* Set up the handles */
//         SetUpHandles(&ConsoleStartInfo);
//         InputWaitHandle = ConsoleStartInfo.InputWaitHandle;
//
//         /* Initialize Console Ctrl Handling */
//         InitializeCtrlHandling();
//
//         /* Sync the current thread's LangId with the console's one */
//         SetTEBLangID();
//     }
//
// Quit:
//     RtlLeaveCriticalSection(&mut ConsoleLock);
//     return Success;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// FreeConsole(VOID)
// {
//     BOOL Success = TRUE;
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_FREECONSOLE FreeConsoleRequest = &ApiMessage.Data.FreeConsoleRequest;
//     HANDLE ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     RtlEnterCriticalSection(&mut ConsoleLock);
//
//     /* We must have a non-trivial handle to close */
//     if (ConsoleHandle == NULL) // IsConsoleHandle(ConsoleHandle)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Set up the data to send to the Console Server */
//     FreeConsoleRequest->ConsoleHandle = ConsoleHandle;
//
//     /* Call the server */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepFree),
//                         sizeof(*FreeConsoleRequest));
//
//     /* Check for success */
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Reset the console handle */
//     NtCurrentPeb()->ProcessParameters->ConsoleHandle = NULL;
//
//     /* Close the associated input handle */
//     CloseHandle(InputWaitHandle);
//     InputWaitHandle = INVALID_HANDLE_VALUE;
//
// Quit:
//     RtlLeaveCriticalSection(&mut ConsoleLock);
//     return Success;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetConsoleScreenBufferInfo(HANDLE hConsoleOutput,
//                            PCONSOLE_SCREEN_BUFFER_INFO lpConsoleScreenBufferInfo)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSCREENBUFFERINFO ScreenBufferInfoRequest = &ApiMessage.Data.ScreenBufferInfoRequest;
//
//     if (lpConsoleScreenBufferInfo == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     ScreenBufferInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ScreenBufferInfoRequest->OutputHandle  = hConsoleOutput;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetScreenBufferInfo),
//                         sizeof(*ScreenBufferInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     lpConsoleScreenBufferInfo->dwSize              = ScreenBufferInfoRequest->ScreenBufferSize;
//     lpConsoleScreenBufferInfo->dwCursorPosition    = ScreenBufferInfoRequest->CursorPosition;
//     lpConsoleScreenBufferInfo->wAttributes         = ScreenBufferInfoRequest->Attributes;
//     lpConsoleScreenBufferInfo->srWindow.Left       = ScreenBufferInfoRequest->ViewOrigin.X;
//     lpConsoleScreenBufferInfo->srWindow.Top        = ScreenBufferInfoRequest->ViewOrigin.Y;
//     lpConsoleScreenBufferInfo->srWindow.Right      = ScreenBufferInfoRequest->ViewOrigin.X + ScreenBufferInfoRequest->ViewSize.X - 1;
//     lpConsoleScreenBufferInfo->srWindow.Bottom     = ScreenBufferInfoRequest->ViewOrigin.Y + ScreenBufferInfoRequest->ViewSize.Y - 1;
//     lpConsoleScreenBufferInfo->dwMaximumWindowSize = ScreenBufferInfoRequest->MaximumViewSize;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleCursorPosition(HANDLE hConsoleOutput,
//                          COORD dwCursorPosition)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETCURSORPOSITION SetCursorPositionRequest = &ApiMessage.Data.SetCursorPositionRequest;
//
//     SetCursorPositionRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetCursorPositionRequest->OutputHandle  = hConsoleOutput;
//     SetCursorPositionRequest->Position      = dwCursorPosition;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCursorPosition),
//                         sizeof(*SetCursorPositionRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetConsoleMode(HANDLE hConsoleHandle,
//                LPDWORD lpMode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETCONSOLEMODE ConsoleModeRequest = &ApiMessage.Data.ConsoleModeRequest;
//
//     if (lpMode == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     ConsoleModeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ConsoleModeRequest->Handle        = hConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetMode),
//                         sizeof(*ConsoleModeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *lpMode = ConsoleModeRequest->Mode;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleMode(HANDLE hConsoleHandle,
//                DWORD dwMode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETCONSOLEMODE ConsoleModeRequest = &ApiMessage.Data.ConsoleModeRequest;
//
//     ConsoleModeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ConsoleModeRequest->Handle        = hConsoleHandle;
//     ConsoleModeRequest->Mode          = dwMode;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetMode),
//                         sizeof(*ConsoleModeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNumberOfConsoleInputEvents(HANDLE hConsoleInput,
//                               LPDWORD lpNumberOfEvents)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETNUMINPUTEVENTS GetNumInputEventsRequest = &ApiMessage.Data.GetNumInputEventsRequest;
//
//     GetNumInputEventsRequest->ConsoleHandle  = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetNumInputEventsRequest->InputHandle    = hConsoleInput;
//     GetNumInputEventsRequest->NumberOfEvents = 0;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetNumberOfInputEvents),
//                         sizeof(*GetNumInputEventsRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     if (lpNumberOfEvents == NULL)
//     {
//         SetLastError(ERROR_INVALID_ACCESS);
//         return FALSE;
//     }
//
//     *lpNumberOfEvents = GetNumInputEventsRequest->NumberOfEvents;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// COORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetLargestConsoleWindowSize(HANDLE hConsoleOutput)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETLARGESTWINDOWSIZE GetLargestWindowSizeRequest = &ApiMessage.Data.GetLargestWindowSizeRequest;
//
//     GetLargestWindowSizeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetLargestWindowSizeRequest->OutputHandle  = hConsoleOutput;
//     GetLargestWindowSizeRequest->Size.X = 0;
//     GetLargestWindowSizeRequest->Size.Y = 0;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetLargestWindowSize),
//                         sizeof(*GetLargestWindowSizeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//     }
//
//     DPRINT("GetLargestConsoleWindowSize, X = %d, Y = %d\n", GetLargestWindowSizeRequest->Size.X, GetLargestWindowSizeRequest->Size.Y);
//     return GetLargestWindowSizeRequest->Size;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetConsoleCursorInfo(HANDLE hConsoleOutput,
//                      PCONSOLE_CURSOR_INFO lpConsoleCursorInfo)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETCURSORINFO CursorInfoRequest = &ApiMessage.Data.CursorInfoRequest;
//
//     if (!lpConsoleCursorInfo)
//     {
//         if (!hConsoleOutput)
//             SetLastError(ERROR_INVALID_HANDLE);
//         else
//             SetLastError(ERROR_INVALID_ACCESS);
//
//         return FALSE;
//     }
//
//     CursorInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     CursorInfoRequest->OutputHandle  = hConsoleOutput;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCursorInfo),
//                         sizeof(*CursorInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *lpConsoleCursorInfo = CursorInfoRequest->Info;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetConsoleCursorInfo(HANDLE hConsoleOutput,
//                      CONST CONSOLE_CURSOR_INFO *lpConsoleCursorInfo)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETCURSORINFO CursorInfoRequest = &ApiMessage.Data.CursorInfoRequest;
//
//     CursorInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     CursorInfoRequest->OutputHandle  = hConsoleOutput;
//     CursorInfoRequest->Info          = *lpConsoleCursorInfo;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCursorInfo),
//                         sizeof(*CursorInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetNumberOfConsoleMouseButtons(LPDWORD lpNumberOfMouseButtons)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETMOUSEINFO GetMouseInfoRequest = &ApiMessage.Data.GetMouseInfoRequest;
//
//     GetMouseInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetMouseInfo),
//                         sizeof(*GetMouseInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *lpNumberOfMouseButtons = GetMouseInfoRequest->NumButtons;
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleActiveScreenBuffer(HANDLE hConsoleOutput)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETACTIVESCREENBUFFER SetScreenBufferRequest = &ApiMessage.Data.SetScreenBufferRequest;
//
//     SetScreenBufferRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetScreenBufferRequest->OutputHandle  = hConsoleOutput;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetActiveScreenBuffer),
//                         sizeof(*SetScreenBufferRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// FlushConsoleInputBuffer(HANDLE hConsoleInput)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_FLUSHINPUTBUFFER FlushInputBufferRequest = &ApiMessage.Data.FlushInputBufferRequest;
//
//     FlushInputBufferRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     FlushInputBufferRequest->InputHandle   = hConsoleInput;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepFlushInputBuffer),
//                         sizeof(*FlushInputBufferRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleScreenBufferSize(HANDLE hConsoleOutput,
//                            COORD dwSize)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETSCREENBUFFERSIZE SetScreenBufferSizeRequest = &ApiMessage.Data.SetScreenBufferSizeRequest;
//
//     SetScreenBufferSizeRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetScreenBufferSizeRequest->OutputHandle  = hConsoleOutput;
//     SetScreenBufferSizeRequest->Size          = dwSize;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetScreenBufferSize),
//                         sizeof(*SetScreenBufferSizeRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static
// BOOL
// IntScrollConsoleScreenBuffer(HANDLE hConsoleOutput,
//                              CONST SMALL_RECT* lpScrollRectangle,
//                              CONST SMALL_RECT* lpClipRectangle,
//                              COORD dwDestinationOrigin,
//                              CONST CHAR_INFO* lpFill,
//                              BOOL bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SCROLLSCREENBUFFER ScrollScreenBufferRequest = &ApiMessage.Data.ScrollScreenBufferRequest;
//
//     ScrollScreenBufferRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     ScrollScreenBufferRequest->OutputHandle  = hConsoleOutput;
//     ScrollScreenBufferRequest->ScrollRectangle = *lpScrollRectangle;
//
//     if (lpClipRectangle != NULL)
//     {
//         ScrollScreenBufferRequest->UseClipRectangle = TRUE;
//         ScrollScreenBufferRequest->ClipRectangle = *lpClipRectangle;
//     }
//     else
//     {
//         ScrollScreenBufferRequest->UseClipRectangle = FALSE;
//     }
//
//     ScrollScreenBufferRequest->DestinationOrigin = dwDestinationOrigin;
//     ScrollScreenBufferRequest->Fill    = *lpFill;
//     ScrollScreenBufferRequest->Unicode = bUnicode;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepScrollScreenBuffer),
//                         sizeof(*ScrollScreenBufferRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ScrollConsoleScreenBufferA(HANDLE hConsoleOutput,
//                            CONST SMALL_RECT* lpScrollRectangle,
//                            CONST SMALL_RECT* lpClipRectangle,
//                            COORD dwDestinationOrigin,
//                            CONST CHAR_INFO* lpFill)
// {
//     return IntScrollConsoleScreenBuffer(hConsoleOutput,
//                                         lpScrollRectangle,
//                                         lpClipRectangle,
//                                         dwDestinationOrigin,
//                                         lpFill,
//                                         FALSE);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ScrollConsoleScreenBufferW(HANDLE hConsoleOutput,
//                            CONST SMALL_RECT *lpScrollRectangle,
//                            CONST SMALL_RECT *lpClipRectangle,
//                            COORD dwDestinationOrigin,
//                            CONST CHAR_INFO *lpFill)
// {
//     return IntScrollConsoleScreenBuffer(hConsoleOutput,
//                                         lpScrollRectangle,
//                                         lpClipRectangle,
//                                         dwDestinationOrigin,
//                                         lpFill,
//                                         TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetConsoleWindowInfo(HANDLE hConsoleOutput,
//                      BOOL bAbsolute,
//                      CONST SMALL_RECT *lpConsoleWindow)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETWINDOWINFO SetWindowInfoRequest = &ApiMessage.Data.SetWindowInfoRequest;
//
//     if (lpConsoleWindow == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     SetWindowInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetWindowInfoRequest->OutputHandle  = hConsoleOutput;
//     SetWindowInfoRequest->Absolute      = bAbsolute;
//     SetWindowInfoRequest->WindowRect    = *lpConsoleWindow;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetWindowInfo),
//                         sizeof(*SetWindowInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleTextAttribute(HANDLE hConsoleOutput,
//                         WORD wAttributes)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETTEXTATTRIB SetTextAttribRequest = &ApiMessage.Data.SetTextAttribRequest;
//
//     SetTextAttribRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetTextAttribRequest->OutputHandle  = hConsoleOutput;
//     SetTextAttribRequest->Attributes    = wAttributes;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetTextAttribute),
//                         sizeof(*SetTextAttribRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static
// BOOL
// AddConsoleCtrlHandler(PHANDLER_ROUTINE HandlerRoutine)
// {
//     PHANDLER_ROUTINE* NewCtrlHandlers = NULL;
//
//     if (HandlerRoutine == NULL)
//     {
//         NtCurrentPeb()->ProcessParameters->ConsoleFlags = TRUE;
//         return TRUE;
//     }
//
//     if (NrCtrlHandlers == NrAllocatedHandlers)
//     {
//         NewCtrlHandlers = RtlAllocateHeap(RtlGetProcessHeap(),
//                                           0,
//                                           (NrCtrlHandlers + 4) * sizeof(PHANDLER_ROUTINE));
//         if (NewCtrlHandlers == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return FALSE;
//         }
//
//         memmove(NewCtrlHandlers, CtrlHandlers, sizeof(PHANDLER_ROUTINE) * NrCtrlHandlers);
//
//         if (NrAllocatedHandlers > 1) RtlFreeHeap(RtlGetProcessHeap(), 0, CtrlHandlers);
//
//         CtrlHandlers = NewCtrlHandlers;
//         NrAllocatedHandlers += 4;
//     }
//
//     ASSERT(NrCtrlHandlers < NrAllocatedHandlers);
//
//     CtrlHandlers[NrCtrlHandlers++] = HandlerRoutine;
//     return TRUE;
// }
//
//
// static
// BOOL
// RemoveConsoleCtrlHandler(PHANDLER_ROUTINE HandlerRoutine)
// {
//     ULONG i;
//
//     if (HandlerRoutine == NULL)
//     {
//         NtCurrentPeb()->ProcessParameters->ConsoleFlags = FALSE;
//         return TRUE;
//     }
//
//     for (i = 0; i < NrCtrlHandlers; i++)
//     {
//         if (CtrlHandlers[i] == HandlerRoutine)
//         {
//             if (i < (NrCtrlHandlers - 1))
//             {
//                 memmove(&CtrlHandlers[i],
//                         &CtrlHandlers[i+1],
//                         (NrCtrlHandlers - i + 1) * sizeof(PHANDLER_ROUTINE));
//             }
//
//             NrCtrlHandlers--;
//             return TRUE;
//         }
//     }
//
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleCtrlHandler(PHANDLER_ROUTINE HandlerRoutine,
//                       BOOL Add)
// {
//     BOOL Ret;
//
//     RtlEnterCriticalSection(&mut ConsoleLock);
//
//     if (Add)
//         Ret = AddConsoleCtrlHandler(HandlerRoutine);
//     else
//         Ret = RemoveConsoleCtrlHandler(HandlerRoutine);
//
//     RtlLeaveCriticalSection(&mut ConsoleLock);
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// GenerateConsoleCtrlEvent(DWORD dwCtrlEvent,
//                          DWORD dwProcessGroupId)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GENERATECTRLEVENT GenerateCtrlEventRequest = &ApiMessage.Data.GenerateCtrlEventRequest;
//
//     if (dwCtrlEvent != CTRL_C_EVENT && dwCtrlEvent != CTRL_BREAK_EVENT)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     GenerateCtrlEventRequest->ConsoleHandle  = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GenerateCtrlEventRequest->CtrlEvent      = dwCtrlEvent;
//     GenerateCtrlEventRequest->ProcessGroupId = dwProcessGroupId;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGenerateCtrlEvent),
//                         sizeof(*GenerateCtrlEventRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// static DWORD
// IntGetConsoleTitle(LPVOID lpConsoleTitle, DWORD dwNumChars, BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETCONSOLETITLE TitleRequest = &ApiMessage.Data.TitleRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     if (dwNumChars == 0) return 0;
//
//     TitleRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     TitleRequest->Length        = dwNumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     TitleRequest->Unicode       = bUnicode;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, TitleRequest->Length);
//     if (CaptureBuffer == NULL)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               TitleRequest->Length,
//                               (PVOID*)&TitleRequest->Title);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetTitle),
//                         sizeof(*TitleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         CsrFreeCaptureBuffer(CaptureBuffer);
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     dwNumChars = TitleRequest->Length / (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//
//     if (dwNumChars > 0)
//     {
//         RtlCopyMemory(lpConsoleTitle, TitleRequest->Title, TitleRequest->Length);
//
//         if (bUnicode)
//             ((LPWSTR)lpConsoleTitle)[dwNumChars] = UNICODE_NULL;
//         else
//             ((LPSTR)lpConsoleTitle)[dwNumChars] = ANSI_NULL;
//     }
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     return dwNumChars;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleTitleW(LPWSTR lpConsoleTitle,
//                  DWORD nSize)
// {
//     return IntGetConsoleTitle(lpConsoleTitle, nSize, TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleTitleA(LPSTR lpConsoleTitle,
//                  DWORD nSize)
// {
//     return IntGetConsoleTitle(lpConsoleTitle, nSize, FALSE);
// }
//
//
// static BOOL
// IntSetConsoleTitle(CONST VOID *lpConsoleTitle, BOOLEAN bUnicode)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSETCONSOLETITLE TitleRequest = &ApiMessage.Data.TitleRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     ULONG NumChars = (ULONG)(lpConsoleTitle ? (bUnicode ? wcslen(lpConsoleTitle) : strlen(lpConsoleTitle)) : 0);
//
//     TitleRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     TitleRequest->Length        = NumChars * (bUnicode ? sizeof(WCHAR) : sizeof(CHAR));
//     TitleRequest->Unicode       = bUnicode;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, TitleRequest->Length);
//     if (CaptureBuffer == NULL)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             (PVOID)lpConsoleTitle,
//                             TitleRequest->Length,
//                             (PVOID*)&TitleRequest->Title);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetTitle),
//                         sizeof(*TitleRequest));
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleTitleW(LPCWSTR lpConsoleTitle)
// {
//     return IntSetConsoleTitle(lpConsoleTitle, TRUE);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleTitleA(LPCSTR lpConsoleTitle)
// {
//     return IntSetConsoleTitle(lpConsoleTitle, FALSE);
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateConsoleScreenBuffer(DWORD dwDesiredAccess,
//                           DWORD dwShareMode,
//                           CONST SECURITY_ATTRIBUTES *lpSecurityAttributes,
//                           DWORD dwFlags,
//                           LPVOID lpScreenBufferData)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_CREATESCREENBUFFER CreateScreenBufferRequest = &ApiMessage.Data.CreateScreenBufferRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer = NULL;
//     PCONSOLE_GRAPHICS_BUFFER_INFO GraphicsBufferInfo = lpScreenBufferData;
//
//     if ( (dwDesiredAccess & ~(GENERIC_READ | GENERIC_WRITE))    ||
//          (dwShareMode & ~(FILE_SHARE_READ | FILE_SHARE_WRITE))  ||
//          (dwFlags != CONSOLE_TEXTMODE_BUFFER && dwFlags != CONSOLE_GRAPHICS_BUFFER) )
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     CreateScreenBufferRequest->ConsoleHandle    = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     CreateScreenBufferRequest->DesiredAccess    = dwDesiredAccess;
//     CreateScreenBufferRequest->InheritHandle    =
//         (lpSecurityAttributes ? lpSecurityAttributes->bInheritHandle : FALSE);
//     CreateScreenBufferRequest->ShareMode        = dwShareMode;
//     CreateScreenBufferRequest->ScreenBufferType = dwFlags;
//
//     if (dwFlags == CONSOLE_GRAPHICS_BUFFER)
//     {
//         if (CreateScreenBufferRequest->InheritHandle || GraphicsBufferInfo == NULL)
//         {
//             SetLastError(ERROR_INVALID_PARAMETER);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         CreateScreenBufferRequest->GraphicsBufferInfo = *GraphicsBufferInfo;
//
//         CaptureBuffer = CsrAllocateCaptureBuffer(1, GraphicsBufferInfo->dwBitMapInfoLength);
//         if (CaptureBuffer == NULL)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         CsrCaptureMessageBuffer(CaptureBuffer,
//                                 (PVOID)GraphicsBufferInfo->lpBitMapInfo,
//                                 GraphicsBufferInfo->dwBitMapInfoLength,
//                                 (PVOID*)&CreateScreenBufferRequest->GraphicsBufferInfo.lpBitMapInfo);
//     }
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepCreateScreenBuffer),
//                         sizeof(*CreateScreenBufferRequest));
//
//     if (CaptureBuffer) CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return INVALID_HANDLE_VALUE;
//     }
//
//     if (dwFlags == CONSOLE_GRAPHICS_BUFFER && GraphicsBufferInfo)
//     {
//         GraphicsBufferInfo->hMutex   = CreateScreenBufferRequest->hMutex  ; // CreateScreenBufferRequest->GraphicsBufferInfo.hMutex  ;
//         GraphicsBufferInfo->lpBitMap = CreateScreenBufferRequest->lpBitMap; // CreateScreenBufferRequest->GraphicsBufferInfo.lpBitMap;
//     }
//
//     return CreateScreenBufferRequest->OutputHandle;
// }
//
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleCP(VOID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETINPUTOUTPUTCP GetConsoleCPRequest = &ApiMessage.Data.GetConsoleCPRequest;
//
//     /* Get the Input Code Page */
//     GetConsoleCPRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetConsoleCPRequest->OutputCP      = FALSE;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCP),
//                         sizeof(*GetConsoleCPRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     return GetConsoleCPRequest->CodePage;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleCP(UINT wCodePageID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETINPUTOUTPUTCP SetConsoleCPRequest = &ApiMessage.Data.SetConsoleCPRequest;
//
//     /* Set the Input Code Page */
//     SetConsoleCPRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetConsoleCPRequest->CodePage      = wCodePageID;
//     SetConsoleCPRequest->OutputCP      = FALSE;
//     /* SetConsoleCPRequest->EventHandle; */
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCP),
//                         sizeof(*SetConsoleCPRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// UINT
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleOutputCP(VOID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETINPUTOUTPUTCP GetConsoleCPRequest = &ApiMessage.Data.GetConsoleCPRequest;
//
//     /* Get the Output Code Page */
//     GetConsoleCPRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetConsoleCPRequest->OutputCP      = TRUE;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetCP),
//                         sizeof(*GetConsoleCPRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return 0;
//     }
//
//     return GetConsoleCPRequest->CodePage;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleOutputCP(UINT wCodePageID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETINPUTOUTPUTCP SetConsoleCPRequest = &ApiMessage.Data.SetConsoleCPRequest;
//
//     /* Set the Output Code Page */
//     SetConsoleCPRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetConsoleCPRequest->CodePage      = wCodePageID;
//     SetConsoleCPRequest->OutputCP      = TRUE;
//     /* SetConsoleCPRequest->EventHandle; */
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetCP),
//                         sizeof(*SetConsoleCPRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     /* Sync the current thread's LangId with the console's one */
//     SetTEBLangID();
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetConsoleProcessList(LPDWORD lpdwProcessList,
//                       DWORD dwProcessCount)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETPROCESSLIST GetProcessListRequest = &ApiMessage.Data.GetProcessListRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//     ULONG nProcesses = 0;
//
//     if (lpdwProcessList == NULL || dwProcessCount == 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, dwProcessCount * sizeof(DWORD));
//     if (CaptureBuffer == NULL)
//     {
//         DPRINT1("CsrAllocateCaptureBuffer failed!\n");
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return 0;
//     }
//
//     GetProcessListRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetProcessListRequest->ProcessCount  = dwProcessCount;
//
//     CsrAllocateMessagePointer(CaptureBuffer,
//                               dwProcessCount * sizeof(DWORD),
//                               (PVOID*)&GetProcessListRequest->ProcessIdsList);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetProcessList),
//                         sizeof(*GetProcessListRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//     }
//     else
//     {
//         nProcesses = GetProcessListRequest->ProcessCount;
//         if (dwProcessCount >= nProcesses)
//         {
//             RtlCopyMemory(lpdwProcessList, GetProcessListRequest->ProcessIdsList, nProcesses * sizeof(DWORD));
//         }
//     }
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//     return nProcesses;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetConsoleSelectionInfo(PCONSOLE_SELECTION_INFO lpConsoleSelectionInfo)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETSELECTIONINFO GetSelectionInfoRequest = &ApiMessage.Data.GetSelectionInfoRequest;
//
//     if (lpConsoleSelectionInfo == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     GetSelectionInfoRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetSelectionInfo),
//                         sizeof(*GetSelectionInfoRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     *lpConsoleSelectionInfo = GetSelectionInfoRequest->Info;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  * @note Strongly inspired by AllocConsole.
//  */
// static BOOL
// IntAttachConsole(DWORD ProcessId,
//                  LPTHREAD_START_ROUTINE CtrlRoutine,
//                  LPTHREAD_START_ROUTINE PropRoutine,
//                  PCONSOLE_START_INFO ConsoleStartInfo)
// {
//     BOOL Success = TRUE;
//     NTSTATUS Status;
//
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_ATTACHCONSOLE AttachConsoleRequest = &ApiMessage.Data.AttachConsoleRequest;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     AttachConsoleRequest->ProcessId   = ProcessId;
//     AttachConsoleRequest->CtrlRoutine = CtrlRoutine;
//     AttachConsoleRequest->PropRoutine = PropRoutine;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, sizeof(CONSOLE_START_INFO));
//     if (CaptureBuffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             ConsoleStartInfo,
//                             sizeof(CONSOLE_START_INFO),
//                             (PVOID*)&AttachConsoleRequest->ConsoleStartInfo);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepAttach),
//                         sizeof(*AttachConsoleRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     // Is AttachConsoleRequest->ConsoleStartInfo->InitEvents aligned on handle boundary ????
//     Status = NtWaitForMultipleObjects(MAX_INIT_EVENTS,
//                                       AttachConsoleRequest->ConsoleStartInfo->InitEvents,
//                                       WaitAny, FALSE, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     NtClose(AttachConsoleRequest->ConsoleStartInfo->InitEvents[INIT_SUCCESS]);
//     NtClose(AttachConsoleRequest->ConsoleStartInfo->InitEvents[INIT_FAILURE]);
//     if (Status != INIT_SUCCESS)
//     {
//         NtCurrentPeb()->ProcessParameters->ConsoleHandle = NULL;
//         Success = FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(ConsoleStartInfo,
//                       AttachConsoleRequest->ConsoleStartInfo,
//                       sizeof(CONSOLE_START_INFO));
//         Success = TRUE;
//     }
//
// Quit:
//     if (CaptureBuffer) CsrFreeCaptureBuffer(CaptureBuffer);
//     return Success;
// }
//
// BOOL
// WINAPI
// AttachConsole(DWORD dwProcessId)
// {
//     BOOL Success;
//     CONSOLE_START_INFO ConsoleStartInfo;
//     DWORD dummy;
//
//     RtlEnterCriticalSection(&mut ConsoleLock);
//
//     if (NtCurrentPeb()->ProcessParameters->ConsoleHandle)
//     {
//         DPRINT1("AttachConsole: Attaching a console to a process already having one\n");
//         SetLastError(ERROR_ACCESS_DENIED);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Set up the console properties */
//     SetUpConsoleInfo(FALSE,
//                      &dummy,
//                      NULL,
//                      &dummy,
//                      NULL,
//                      &ConsoleStartInfo);
//
//     Success = IntAttachConsole(dwProcessId,
//                                ConsoleControlDispatcher,
//                                PropDialogHandler,
//                                &ConsoleStartInfo);
//     if (Success)
//     {
//         /* Set up the handles */
//         SetUpHandles(&ConsoleStartInfo);
//         InputWaitHandle = ConsoleStartInfo.InputWaitHandle;
//
//         /* Initialize Console Ctrl Handling */
//         InitializeCtrlHandling();
//
//         /* Sync the current thread's LangId with the console's one */
//         SetTEBLangID();
//     }
//
// Quit:
//     RtlLeaveCriticalSection(&mut ConsoleLock);
//     return Success;
// }
//
//
// /*
//  * @implemented
//  */
// HWND
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleWindow(VOID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETWINDOW GetWindowRequest = &ApiMessage.Data.GetWindowRequest;
//
//     GetWindowRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetConsoleWindow),
//                         sizeof(*GetWindowRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return (HWND)NULL;
//     }
//
//     return GetWindowRequest->WindowHandle;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleIcon(HICON hIcon)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_SETICON SetIconRequest = &ApiMessage.Data.SetIconRequest;
//
//     SetIconRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     SetIconRequest->IconHandle    = hIcon;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepSetIcon),
//                         sizeof(*SetIconRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * \name SetConsoleInputExeNameW
//  * \brief Sets the console input file name from a unicode string.
//  * \param lpExeName Pointer to a unicode string with the name.
//  * \return TRUE if successful, FALSE if unsuccessful.
//  * \remarks If lpExeName is 0 or the string length is 0 or greater than 255,
//  *          the function fails and sets last error to ERROR_INVALID_PARAMETER.
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleInputExeNameW(IN LPCWSTR lpExeName)
// {
//     DWORD ExeLength;
//
//     ExeLength = lstrlenW(lpExeName) as DWORD;
//     if ((ExeLength == 0) || (ExeLength >= EXENAME_LENGTH))
//     {
//         /* Fail if string is empty or too long */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     RtlEnterCriticalSection(&ExeNameLock);
//     _SEH2_TRY
//     {
//         /* Set the input EXE name, not NULL terminated */
//         RtlCopyMemory(ExeNameBuffer, lpExeName, ExeLength * sizeof(WCHAR));
//         ExeNameLength = (USHORT)ExeLength;
//     }
//     _SEH2_FINALLY
//     {
//         RtlLeaveCriticalSection(&ExeNameLock);
//     }
//     _SEH2_END;
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * \name SetConsoleInputExeNameA
//  * \brief Sets the console input file name from an ansi string.
//  * \param lpExeName Pointer to an ansi string with the name.
//  * \return TRUE if successful, FALSE if unsuccessful.
//  * \remarks If lpExeName is 0 or the string length is 0 or greater than 255,
//  *          the function fails and sets last error to ERROR_INVALID_PARAMETER.
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleInputExeNameA(IN LPCSTR lpExeName)
// {
//     NTSTATUS Status;
// #ifdef USE_TEB_STATIC_USTR
//     PUNICODE_STRING ExeNameU;
// #else
//     UNICODE_STRING ExeNameU;
// #endif
//     ANSI_STRING ExeNameA;
// #ifndef USE_TEB_STATIC_USTR
//     WCHAR Buffer[EXENAME_LENGTH];
// #endif
//
// #ifdef USE_TEB_STATIC_USTR
//     /*
//      * Use the TEB static UNICODE string for storage. It is already
//      * initialized at process creation time by the Memory Manager.
//      */
//     ExeNameU = &NtCurrentTeb()->StaticUnicodeString;
// #endif
//
//     /* Initialize string for conversion */
//     RtlInitAnsiString(&ExeNameA, lpExeName);
//
//     if ((ExeNameA.Length == 0) || (ExeNameA.Length >= EXENAME_LENGTH))
//     {
//         /* Fail if string is empty or too long */
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
// #ifndef USE_TEB_STATIC_USTR
//     ExeNameU.Length = 0;
//     ExeNameU.MaximumLength = (USHORT)sizeof(Buffer);
//     ExeNameU.Buffer = Buffer;
// #endif
//
// #ifdef USE_TEB_STATIC_USTR
//     Status = RtlAnsiStringToUnicodeString(ExeNameU, &ExeNameA, FALSE);
// #else
//     Status = RtlAnsiStringToUnicodeString(&ExeNameU, &ExeNameA, FALSE);
// #endif
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail if string is empty or too long */
//         if (Status == STATUS_BUFFER_OVERFLOW)
//             SetLastError(ERROR_FILENAME_EXCED_RANGE);
//         else
//             SetLastError(ERROR_INVALID_PARAMETER);
//
//         return FALSE;
//     }
//
// #ifdef USE_TEB_STATIC_USTR
//     return SetConsoleInputExeNameW(ExeNameU->Buffer);
// #else
//     return SetConsoleInputExeNameW(ExeNameU.Buffer);
// #endif
// }
//
//
// /******************************************************************************
//  * \name GetConsoleInputExeNameW
//  * \brief Retrieves the console input file name as unicode string.
//  * \param nBufferLength Length of the buffer in WCHARs.
//  *        Specify 0 to receive the needed buffer length.
//  * \param lpBuffer Pointer to a buffer that receives the string.
//  * \return Needed buffer size if \p nBufferLength is 0.
//  *         Otherwise 1 if successful, 2 if buffer is too small.
//  * \remarks Sets last error value to ERROR_BUFFER_OVERFLOW if the buffer
//  *          is not big enough.
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleInputExeNameW(IN DWORD nBufferLength,
//                         OUT LPWSTR lpExeName)
// {
//     if (nBufferLength <= ExeNameLength)
//     {
//         /* Buffer is not large enough! Return the correct size. */
//         SetLastError(ERROR_BUFFER_OVERFLOW);
//         return ExeNameLength + 1;
//     }
//
//     RtlEnterCriticalSection(&ExeNameLock);
//     _SEH2_TRY
//     {
//         /* Copy the input EXE name and NULL-terminate it */
//         RtlCopyMemory(lpExeName, ExeNameBuffer, ExeNameLength * sizeof(WCHAR));
//         lpExeName[ExeNameLength] = UNICODE_NULL;
//     }
//     _SEH2_FINALLY
//     {
//         RtlLeaveCriticalSection(&ExeNameLock);
//     }
//     _SEH2_END;
//
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * \name GetConsoleInputExeNameA
//  * \brief Retrieves the console input file name as ansi string.
//  * \param nBufferLength Length of the buffer in CHARs.
//  * \param lpBuffer Pointer to a buffer that receives the string.
//  * \return 1 if successful, 2 if buffer is too small.
//  * \remarks Sets last error value to ERROR_BUFFER_OVERFLOW if the buffer
//  *          is not big enough. The buffer receives as much characters as fit.
//  */
// DWORD
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleInputExeNameA(IN DWORD nBufferLength,
//                         OUT LPSTR lpExeName)
// {
//     NTSTATUS Status;
//     DWORD ExeLength;
//     UNICODE_STRING BufferU;
//     ANSI_STRING BufferA;
//     WCHAR Buffer[EXENAME_LENGTH];
//
//     /* Get the UNICODE name */
//     ExeLength = GetConsoleInputExeNameW(EXENAME_LENGTH, Buffer);
//
//     if ((ExeLength == 0) || (ExeLength >= EXENAME_LENGTH))
//         return ExeLength;
//
//     /* Initialize the strings for conversion */
//     RtlInitUnicodeString(&BufferU, Buffer);
//     BufferA.Length = 0;
//     BufferA.MaximumLength = (USHORT)nBufferLength;
//     BufferA.Buffer = lpExeName;
//
//     /* Convert UNICODE name to ANSI, copying as much chars as it can fit */
//     Status = RtlUnicodeStringToAnsiString(&BufferA, &BufferU, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_BUFFER_OVERFLOW)
//         {
//             SetLastError(ERROR_BUFFER_OVERFLOW);
//             return ExeLength + 1;
//         }
//         SetLastError(ERROR_INVALID_PARAMETER);
//     }
//
//     return ExeLength;
// }
//
// BOOL
// WINAPI
// GetConsoleCharType(HANDLE hConsole, COORD Coord, PDWORD Type)
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleCursorMode(HANDLE hConsole, PBOOL pUnknown1, PBOOL pUnknown2)
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleCursorMode(HANDLE hConsole, BOOL Unknown1, BOOL Unknown2)
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleNlsMode(HANDLE hConsole, LPDWORD lpMode)
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleNlsMode(HANDLE hConsole, DWORD dwMode)
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleLocalEUDC(
//      HANDLE hConsoleHandle,
//      WORD wCodePoint,
//      COORD cFontSize,
//      PCHAR lpSB)
// {
//     STUB;
//     return FALSE;
// }
//
// static BOOL
// IntRegisterConsoleIME(
//      HWND hWnd,
//      DWORD dwThreadId,
//      SIZE_T cbDesktop,
//      PWSTR pDesktop,
//      PDWORD pdwAttachToThreadId)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_REGISTERCONSOLEIME RegisterConsoleIME = &ApiMessage.Data.RegisterConsoleIME;
//     PCSR_CAPTURE_BUFFER CaptureBuffer;
//
//     if (!cbDesktop || !pDesktop)
//     {
//         pDesktop = NtCurrentPeb()->ProcessParameters->DesktopInfo.Buffer;
//         cbDesktop = NtCurrentPeb()->ProcessParameters->DesktopInfo.Length;
//     }
//
//     cbDesktop = min(cbDesktop, (MAX_PATH + 1) * sizeof(WCHAR));
//
//     RegisterConsoleIME->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     RegisterConsoleIME->hWnd = hWnd;
//     RegisterConsoleIME->dwThreadId = dwThreadId;
//     RegisterConsoleIME->cbDesktop = cbDesktop;
//
//     CaptureBuffer = CsrAllocateCaptureBuffer(1, cbDesktop as ULONG);
//     if (!CaptureBuffer)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     CsrCaptureMessageBuffer(CaptureBuffer,
//                             pDesktop,
//                             cbDesktop,
//                             (PVOID*)&RegisterConsoleIME->pDesktop);
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         CaptureBuffer,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepRegisterConsoleIME),
//                         sizeof(*RegisterConsoleIME));
//
//     CsrFreeCaptureBuffer(CaptureBuffer);
//
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     if (pdwAttachToThreadId)
//     {
//         _SEH2_TRY
//         {
//             *pdwAttachToThreadId = RegisterConsoleIME->dwAttachToThreadId;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             BaseSetLastNTError(STATUS_ACCESS_VIOLATION);
//             _SEH2_YIELD(return FALSE);
//         }
//         _SEH2_END;
//     }
//
//     return TRUE;
// }
//
// static BOOL
// IntUnregisterConsoleIME(
//      DWORD dwThreadId)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_UNREGISTERCONSOLEIME UnregisterConsoleIME = &ApiMessage.Data.UnregisterConsoleIME;
//
//     UnregisterConsoleIME->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     UnregisterConsoleIME->dwThreadId = dwThreadId;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepUnregisterConsoleIME),
//                         sizeof(*UnregisterConsoleIME));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* This function is called by CONIME.EXE */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// RegisterConsoleIME(
//      HWND hWnd,
//      LPDWORD pdwAttachToThreadId)
// {
//     return IntRegisterConsoleIME(hWnd,
//                                  GetCurrentThreadId(),
//                                  0,
//                                  NULL,
//                                  pdwAttachToThreadId);
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// RegisterConsoleOS2(BOOL bUnknown)
// {
//     STUB;
//     return FALSE;
// }
//
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetConsoleOS2OemFormat(BOOL bUnknown)
// {
//     STUB;
//     return FALSE;
// }
//
// /* This function is called by CONIME.EXE */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// UnregisterConsoleIME(VOID)
// {
//     return IntUnregisterConsoleIME(GetCurrentThreadId());
// }
//
// /**
//  * @brief
//  * Internal helper function used to synchronize the current
//  * thread's language ID with the one from the console.
//  **/
// VOID
// SetTEBLangID(VOID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETLANGID LangIdRequest = &ApiMessage.Data.LangIdRequest;
//
//     /* Retrieve the "best-suited" language ID corresponding
//      * to the active console output code page. */
//     LangIdRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetLangId),
//                         sizeof(*LangIdRequest));
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         /*
//          * No best console language ID: keep the current thread's one.
//          * Since this internal function only modifies an optional setting,
//          * don't set any last error, as it could otherwise mess with the
//          * main last error set by the caller.
//          */
//         return;
//     }
//
//     /*
//      * We succeeded, set the current thread's language ID by
//      * modifying its locale -- Windows <= 2003 does not have
//      * the concept of a separate thread UI language.
//      * Ignore the returned value.
//      */
//     if (!SetThreadLocale(MAKELCID(LangIdRequest->LangId, SORT_DEFAULT)))
//     {
//         DPRINT1("SetTEBLangID: Could not set thread locale to console lang ID %lu\n",
//                 LangIdRequest->LangId);
//     }
// }
//
// static
// BOOL
// IntGetConsoleKeyboardLayoutName(OUT PVOID pszLayoutName,
//                                 IN BOOL bAnsi)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_GETKBDLAYOUTNAME GetKbdLayoutNameRequest = &ApiMessage.Data.GetKbdLayoutNameRequest;
//
//     /* Set up the data to send to the Console Server */
//     GetKbdLayoutNameRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//     GetKbdLayoutNameRequest->Ansi          = bAnsi;
//
//     /* Call the server */
//     CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                         NULL,
//                         CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepGetKeyboardLayoutName),
//                         sizeof(*GetKbdLayoutNameRequest));
//
//     /* Check for success */
//     if (!NT_SUCCESS(ApiMessage.Status))
//     {
//         BaseSetLastNTError(ApiMessage.Status);
//         return FALSE;
//     }
//
//     /* Retrieve the results */
//     _SEH2_TRY
//     {
//         /* Copy only KL_NAMELENGTH == 9 characters, ANSI or UNICODE */
//         if (bAnsi)
//             strncpy(pszLayoutName, (PCHAR)GetKbdLayoutNameRequest->LayoutBuffer, KL_NAMELENGTH);
//         else
//             wcsncpy(pszLayoutName, (PWCHAR)GetKbdLayoutNameRequest->LayoutBuffer, KL_NAMELENGTH);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         SetLastError(ERROR_INVALID_ACCESS);
//         _SEH2_YIELD(return FALSE);
//     }
//     _SEH2_END;
//
//     return TRUE;
// }
//
// /*
//  * @implemented (undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleKeyboardLayoutNameA(OUT LPSTR pszLayoutName)
// {
//     return IntGetConsoleKeyboardLayoutName(pszLayoutName, TRUE);
// }
//
// /*
//  * @implemented (undocumented)
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// GetConsoleKeyboardLayoutNameW(OUT LPWSTR pszLayoutName)
// {
//     return IntGetConsoleKeyboardLayoutName(pszLayoutName, FALSE);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SetLastConsoleEventActive(VOID)
// {
//     CONSOLE_API_MESSAGE ApiMessage;
//     PCONSOLE_NOTIFYLASTCLOSE NotifyLastCloseRequest = &ApiMessage.Data.NotifyLastCloseRequest;
//
//     /* Set the flag used by the console control dispatcher */
//     LastCloseNotify = TRUE;
//
//     /* Set up the input arguments */
//     NotifyLastCloseRequest->ConsoleHandle = NtCurrentPeb()->ProcessParameters->ConsoleHandle;
//
//     /* Call CSRSS; just return the NTSTATUS cast to DWORD */
//     return CsrClientCallServer((PCSR_API_MESSAGE)&ApiMessage,
//                                NULL,
//                                CSR_CREATE_API_NUMBER(CONSRV_SERVERDLL_INDEX, ConsolepNotifyLastClose),
//                                sizeof(*NotifyLastCloseRequest));
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
