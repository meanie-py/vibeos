use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use crate::resources::*;
use core::ffi::c_void;

extern "C" {
    fn FlushConsoleInputBuffer(hConsoleInput: *mut core::ffi::c_void) -> i32;
    fn GetStdHandle(nStdHandle: u32) -> *mut core::ffi::c_void;
    fn ReadConsoleW( hConsoleInput: *mut core::ffi::c_void, lpBuffer: *mut core::ffi::c_void, nNumberOfCharsToRead: u32, lpNumberOfCharsRead: *mut u32, pInputControl: *mut core::ffi::c_void, ) -> i32;
    fn SetConsoleMode(hConsoleHandle: *mut core::ffi::c_void, dwMode: u32) -> i32;
    fn WriteConsoleW( hConsoleOutput: *mut core::ffi::c_void, lpBuffer: *const core::ffi::c_void, nNumberOfCharsToWrite: u32, lpNumberOfCharsWritten: *mut u32, lpReserved: *mut core::ffi::c_void, ) -> i32;
    fn wcslen(s: *const u16) -> usize;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_msgina as *mut __wine_debug_channel };

pub static mut GinaTextUI: GINA_UI = unsafe {
    _GINA_UI {
        Initialize: Some(TUIInitialize as unsafe extern "C" fn(PGINA_CONTEXT) -> BOOL),
        DisplayStatusMessage: Some(
            TUIDisplayStatusMessage
                as unsafe extern "C" fn(PGINA_CONTEXT, HDESK, DWORD, PWSTR, PWSTR) -> BOOL,
        ),
        RemoveStatusMessage: Some(
            TUIRemoveStatusMessage as unsafe extern "C" fn(PGINA_CONTEXT) -> BOOL,
        ),
        DisplaySASNotice: Some(TUIDisplaySASNotice as unsafe extern "C" fn(PGINA_CONTEXT) -> ()),
        LoggedOnSAS: Some(TUILoggedOnSAS as unsafe extern "C" fn(PGINA_CONTEXT, DWORD) -> INT),
        LoggedOutSAS: Some(TUILoggedOutSAS as unsafe extern "C" fn(PGINA_CONTEXT) -> INT),
        LockedSAS: Some(TUILockedSAS as unsafe extern "C" fn(PGINA_CONTEXT) -> INT),
        DisplayLockedNotice: Some(
            TUIDisplayLockedNotice as unsafe extern "C" fn(PGINA_CONTEXT) -> (),
        ),
    }
};

unsafe fn TUIInitialize(pgContext: PGINA_CONTEXT) -> BOOL {
    if ((*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/tui.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"TUIInitialize\0".as_ptr() as *const core::ffi::c_char,
            16i32,
            b"TUIInitialize(%p)\n\0".as_ptr() as *const core::ffi::c_char,
            pgContext,
        );
    }
    AllocConsole()
}
unsafe fn TUIDisplayStatusMessage(
    pgContext: PGINA_CONTEXT,
    hDesktop: HDESK,
    dwOptions: DWORD,
    pTitle: PWSTR,
    pMessage: PWSTR,
) -> BOOL {
    let _ = pgContext;
    let _ = hDesktop;
    let _ = dwOptions;
    let _ = pTitle;

    let mut result: DWORD = 0;
    let newLine: *const u16 = &[0x0Au16, 0] as *const u16;

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            c"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/tui.c".as_ptr(),
            c"TUIDisplayStatusMessage".as_ptr(),
            32i32,
            c"TUIDisplayStatusMessage(%ws)\n".as_ptr(),
            pMessage,
        );
    }

    (WriteConsoleW(
        GetStdHandle(STD_OUTPUT_HANDLE),
        pMessage as *const core::ffi::c_void,
        wcslen(pMessage as *const u16) as DWORD,
        &mut result,
        core::ptr::null_mut(),
    ) != 0
    && WriteConsoleW(
        GetStdHandle(STD_OUTPUT_HANDLE),
        newLine as *const core::ffi::c_void,
        wcslen(newLine as *const u16) as DWORD,
        &mut result,
        core::ptr::null_mut(),
    ) != 0) as i32
}

unsafe fn TUIRemoveStatusMessage(pgContext: PGINA_CONTEXT) -> i32 {
    // suppress unused parameter warning
    let _ = pgContext;
    1
}

unsafe fn DisplayResourceText(
    uIdResourceText: UINT,
    AddNewLine: BOOL,
) -> BOOL {
    let mut prompt: [WCHAR; 256] = [0; 256];
    let mut count: DWORD = 0;

    if LoadStringW(
        hDllInstance,
        uIdResourceText,
        prompt.as_mut_ptr(),
        prompt.len() as i32,
    ) == 0
    {
        return 0;
    }
    if WriteConsoleW(
        GetStdHandle((-11i32) as DWORD),
        prompt.as_ptr(),
        wcslen(prompt.as_ptr()) as DWORD,
        &mut count,
        core::ptr::null_mut(),
    ) == 0
    {
        return 0;
    }
    if AddNewLine != 0 {
        let newline: [WCHAR; 2] = [0x0A, 0];
        if WriteConsoleW(
            GetStdHandle((-11i32) as DWORD),
            newline.as_ptr(),
            wcslen(newline.as_ptr()) as DWORD,
            &mut count,
            core::ptr::null_mut(),
        ) == 0
        {
            return 0;
        }
    }
    1
}

unsafe fn TUIDisplaySASNotice(pgContext: PGINA_CONTEXT) {
    TRACE!(b"TUIDisplaySASNotice()\n".as_ptr() as *const i8);
    UNREFERENCED_PARAMETER(pgContext);
    DisplayResourceText(IDS_LOGGEDOUTSAS as UINT, 1u8);
    DisplayResourceText(IDS_PRESSCTRLALTDELETE as UINT, 1u8);
}

unsafe fn TUILoggedOnSAS(pgContext: PGINA_CONTEXT, dwSasType: DWORD) -> INT {
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 << __WINE_DBCL_TRACE as core::ffi::c_int) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/tui.c\0".as_ptr() as *const core::ffi::c_char,
            b"TUILoggedOnSAS\0".as_ptr() as *const core::ffi::c_char,
            112 as core::ffi::c_int,
            b"TUILoggedOnSAS()\n\0".as_ptr() as *const core::ffi::c_char,
        );
    }

    let _ = pgContext;

    if dwSasType != WLX_SAS_TYPE_CTRL_ALT_DEL as DWORD {
        return WLX_SAS_ACTION_NONE;
    }

    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 << __WINE_DBCL_FIXME as core::ffi::c_int) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/msgina/tui.c\0".as_ptr() as *const core::ffi::c_char,
            b"TUILoggedOnSAS\0".as_ptr() as *const core::ffi::c_char,
            122 as core::ffi::c_int,
            b"FIXME: TUILoggedOnSAS(): Let's suppose the user wants to log off...\n\0".as_ptr() as *const core::ffi::c_char,
        );
    }

    WLX_SAS_ACTION_LOGOFF
}

unsafe fn ReadString(
    uIdResourcePrompt: u32,
    Buffer: *mut u16,
    BufferLength: u32,
    ShowString: u8,
) -> u8 {

    const STD_INPUT_HANDLE: u32 = -10i32 as u32;
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const UNICODE_NULL: u16 = 0u16;

    let mut count: u32 = 0;
    let mut i: u32 = 0;
    let mut charToDisplay: [u16; 2] = [0u16, UNICODE_NULL];

    if SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), 0) == 0 {
        return 0u8;
    }
    if FlushConsoleInputBuffer(GetStdHandle(STD_INPUT_HANDLE)) == 0 {
        return 0u8;
    }
    if DisplayResourceText(uIdResourcePrompt, 0u8) == 0 {
        return 0u8;
    }

    i = 0;
    loop {
        let mut readChar: u16 = 0;
        if ReadConsoleW(
            GetStdHandle(STD_INPUT_HANDLE),
            core::ptr::addr_of_mut!(readChar).cast::<core::ffi::c_void>(),
            1,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(),
        ) == 0
        {
            return 0u8;
        }
        if readChar == b'\r'.into() || readChar == b'\n'.into() {
            charToDisplay[0] = b'\n' as u16;
            WriteConsoleW(
                GetStdHandle(STD_OUTPUT_HANDLE),
                charToDisplay.as_ptr() as *const core::ffi::c_void,
                wcslen(charToDisplay.as_ptr() as *const u16) as u32,
                core::ptr::addr_of_mut!(count),
                core::ptr::null_mut(),
            );
            break;
        } else {
            if ShowString != 0 {
                charToDisplay[0] = readChar;
                WriteConsoleW(
                    GetStdHandle(STD_OUTPUT_HANDLE),
                    charToDisplay.as_ptr() as *const core::ffi::c_void,
                    wcslen(charToDisplay.as_ptr() as *const u16) as u32,
                    core::ptr::addr_of_mut!(count),
                    core::ptr::null_mut(),
                );
            }
            let idx = i;
            i = i.wrapping_add(1);
            *Buffer.offset(idx as isize) = readChar;
        }
    }
    *Buffer.offset(i as isize) = UNICODE_NULL;

    if ShowString == 0 {
        static NEWLINE: &[u16; 2] = &[b'\n' as u16, 0u16];
        let mut result: u32 = 0;
        WriteConsoleW(
            GetStdHandle(STD_OUTPUT_HANDLE),
            NEWLINE.as_ptr() as *const core::ffi::c_void,
            wcslen(NEWLINE.as_ptr() as *const u16) as u32,
            core::ptr::addr_of_mut!(result),
            core::ptr::null_mut(),
        );
    }

    let _ = BufferLength;
    1u8
}

unsafe fn TUILoggedOutSAS(pgContext: PGINA_CONTEXT) -> i32 {
    let mut UserName: [u16; 256] = [0u16; 256];
    let mut Password: [u16; 256] = [0u16; 256];
    let mut SubStatus: NTSTATUS = STATUS_SUCCESS;

    TRACE!(b"TUILoggedOutSAS()\n\0".as_ptr() as *const i8);

    if ReadString(IDS_ASKFORUSER, UserName.as_mut_ptr(), UserName.len() as u32, 1u8) == 0 {
        return WLX_SAS_ACTION_NONE;
    }
    if ReadString(IDS_ASKFORPASSWORD, Password.as_mut_ptr(), Password.len() as u32, 0u8) == 0 {
        return WLX_SAS_ACTION_NONE;
    }

    let Status = DoLoginTasks(
        pgContext,
        UserName.as_mut_ptr(),
        core::ptr::null_mut(),
        Password.as_mut_ptr(),
        &mut SubStatus,
    );

    if Status == STATUS_SUCCESS {
        if CreateProfile(
            pgContext,
            UserName.as_mut_ptr(),
            core::ptr::null_mut(),
            Password.as_mut_ptr(),
        ) != 0
        {
            return WLX_SAS_ACTION_LOGON;
        }
    }

    WLX_SAS_ACTION_NONE
}

unsafe fn TUILockedSAS(pgContext: PGINA_CONTEXT) -> i32 {
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut UserName: [u16; 256] = [0u16; 256];
    let mut Password: [u16; 256] = [0u16; 256];
    let mut SubStatus: NTSTATUS = 0;
    let mut Status: NTSTATUS;

    TRACE!(b"TUILockedSAS()\n\0".as_ptr() as *const i8);

    let _ = pgContext; // UNREFERENCED_PARAMETER

    if DisplayResourceText(IDS_LOGGEDOUTSAS, 1u8) == 0 {
        return WLX_SAS_ACTION_UNLOCK_WKSTA;
    }

    if ReadString(IDS_ASKFORUSER, UserName.as_mut_ptr(), 256u32, 1u8) == 0 {
        return WLX_SAS_ACTION_NONE;
    }

    if ReadString(IDS_ASKFORPASSWORD, Password.as_mut_ptr(), 256u32, 0u8) == 0 {
        return WLX_SAS_ACTION_NONE;
    }

    Status = ConnectToLsa(pgContext);
    if !NT_SUCCESS(Status) {
        WARN!(b"ConnectToLsa() failed\n\0".as_ptr() as *const i8);
        return WLX_SAS_ACTION_NONE;
    }

    Status = MyLogonUser(
        (*pgContext).LsaHandle,
        (*pgContext).AuthenticationPackage,
        UserName.as_mut_ptr() as LPWSTR,
        core::ptr::null_mut(),
        Password.as_mut_ptr() as LPWSTR,
        &mut hToken,
        &mut SubStatus,
    );
    if !NT_SUCCESS(Status) {
        WARN!(b"MyLogonUser() failed\n\0".as_ptr() as *const i8);
        return WLX_SAS_ACTION_NONE;
    }

    CloseHandle(hToken);
    return WLX_SAS_ACTION_UNLOCK_WKSTA;
}

unsafe fn TUIDisplayLockedNotice(pgContext: *mut _GINA_CONTEXT) {}
