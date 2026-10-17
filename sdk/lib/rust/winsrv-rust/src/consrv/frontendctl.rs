use winsrv_sys::nt::*;
use super::term::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::console::ConSrvGetConsole;
use super::console::ConSrvReleaseConsole;


// CON_API macro expansion helper
// CON_API(Name, RequestType, RequestField) expands to a function signature
// We translate each expanded function directly

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleHardwareState(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleHardwareStateImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleHardwareStateImpl(
    _ApiMessage: *mut CONSOLE_API_MESSAGE,
    _Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleHardwareState(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvSetConsoleHardwareStateImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleHardwareStateImpl(
    _ApiMessage: *mut CONSOLE_API_MESSAGE,
    _Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleDisplayMode(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleDisplayModeImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleDisplayModeImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetDisplayModeRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetDisplayModeRequest
    );
    (*GetDisplayModeRequest).DisplayMode = TermGetDisplayMode(Console);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleDisplayMode(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvSetConsoleDisplayModeImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleDisplayModeImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let SetDisplayModeRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.SetDisplayModeRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetScreenBuffer(
        ProcessData,
        (*SetDisplayModeRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x40000000, // GENERIC_WRITE
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = if TermSetDisplayMode(Console, (*SetDisplayModeRequest).DisplayMode) != 0 {
        (*SetDisplayModeRequest).NewSBDim = (*Buff).ScreenBufferSize;
        STATUS_SUCCESS
    } else {
        STATUS_INVALID_PARAMETER
    };

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetLargestConsoleWindowSize(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetLargestConsoleWindowSizeImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetLargestConsoleWindowSizeImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetLargestWindowSizeRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetLargestWindowSizeRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetTextModeBuffer(
        ProcessData,
        (*GetLargestWindowSizeRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x80000000, // GENERIC_READ
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    TermGetLargestConsoleWindowSize(
        Console,
        core::ptr::addr_of_mut!((*GetLargestWindowSizeRequest).Size),
    );

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvShowConsoleCursor(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvShowConsoleCursorImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvShowConsoleCursorImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let ShowCursorRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.ShowCursorRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetScreenBuffer(
        ProcessData,
        (*ShowCursorRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x40000000, // GENERIC_WRITE
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*ShowCursorRequest).RefCount = TermShowMouseCursor(Console, (*ShowCursorRequest).Show);

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleCursor(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvSetConsoleCursorImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleCursorImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let SetCursorRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.SetCursorRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetGraphicsBuffer(
        ProcessData,
        (*SetCursorRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x40000000, // GENERIC_WRITE
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Success = TermSetMouseCursor(Console, (*SetCursorRequest).CursorHandle);

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    if Success != 0 { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL }
}

#[no_mangle]
pub unsafe extern "system" fn SrvConsoleMenuControl(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvConsoleMenuControlImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvConsoleMenuControlImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let MenuControlRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.MenuControlRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetScreenBuffer(
        ProcessData,
        (*MenuControlRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x40000000, // GENERIC_WRITE
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*MenuControlRequest).MenuHandle = TermMenuControl(
        Console,
        (*MenuControlRequest).CmdIdLow,
        (*MenuControlRequest).CmdIdHigh,
    );

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleMenuClose(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvSetConsoleMenuCloseImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleMenuCloseImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let SetMenuCloseRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.SetMenuCloseRequest
    );
    
    if TermSetMenuClose(Console, (*SetMenuCloseRequest).Enable) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadConsoleDesktop(
    ThreadId: usize,
    ConsoleDesktop: *mut HDESK,
) -> i32 {
    *ConsoleDesktop = core::ptr::null_mut();

    let mut CsrThread: PCSR_THREAD = core::ptr::null_mut();
    let Status = CsrLockThreadByClientId(ULongToHandle(ThreadId as u32), &mut CsrThread as *mut PCSR_THREAD);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(
        (*(*(*CsrThread).Process).ServerData.as_ptr().add(CONSRV_SERVERDLL_INDEX as usize)) as *mut CONSOLE_PROCESS_DATA,
        &mut Console as *mut PCONSRV_CONSOLE,
        1u8,
    );
    CsrUnlockThread(CsrThread);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    *ConsoleDesktop = TermGetThreadConsoleDesktop(Console);
    ConSrvReleaseConsole(Console, 1u8);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleWindow(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleWindowImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleWindowImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetWindowRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetWindowRequest
    );
    
    (*GetWindowRequest).WindowHandle = TermGetConsoleWindowHandle(Console);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleIcon(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvSetConsoleIconImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleIconImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let SetIconRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.SetIconRequest
    );
    
    if TermChangeIcon(Console, (*SetIconRequest).IconHandle) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleSelectionInfo(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleSelectionInfoImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleSelectionInfoImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetSelectionInfoRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetSelectionInfoRequest
    );
    
    if TermGetSelectionInfo(
        Console,
        core::ptr::addr_of_mut!((*GetSelectionInfoRequest).Info),
    ) != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleNumberOfFonts(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleNumberOfFontsImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleNumberOfFontsImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    _Console: PCONSRV_CONSOLE,
    _ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetNumFontsRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetNumFontsRequest
    );
    
    (*GetNumFontsRequest).NumFonts = 0;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleFontInfo(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleFontInfoImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleFontInfoImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    _Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetFontInfoRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetFontInfoRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetTextModeBuffer(
        ProcessData,
        (*GetFontInfoRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x80000000, // GENERIC_READ
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*GetFontInfoRequest).NumFonts = 0;

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleFontSize(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleFontSizeImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleFontSizeImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetFontSizeRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetFontSizeRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetTextModeBuffer(
        ProcessData,
        (*GetFontSizeRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x80000000, // GENERIC_READ
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // One font, index 0 (the C left this a FIXME and returned garbage). Any
    // other index fails ERROR_INVALID_PARAMETER, as on Windows 11.
    let Status = if (*GetFontSizeRequest).FontIndex != 0 {
        STATUS_INVALID_PARAMETER
    } else {
        TermGetFontSize(Console, core::ptr::addr_of_mut!((*GetFontSizeRequest).FontSize));
        STATUS_SUCCESS
    };

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCurrentFont(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvGetConsoleCurrentFontImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvGetConsoleCurrentFontImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let GetCurrentFontRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.GetCurrentFontRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetTextModeBuffer(
        ProcessData,
        (*GetCurrentFontRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x80000000, // GENERIC_READ
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*GetCurrentFontRequest).FontIndex = 0;
    if (*GetCurrentFontRequest).MaximumWindow != 0 {
        // Windows answers the MAXIMUM WINDOW size here, in characters -- the
        // dwMaximumWindowSize of GetConsoleScreenBufferInfo -- not a font size
        // (measured on Windows 11: 120x84 for a 120x9001 buffer).
        let mut Largest: COORD = core::mem::zeroed();
        TermGetLargestConsoleWindowSize(Console, &mut Largest);
        Largest.X = core::cmp::min(Largest.X, (*Buff).ScreenBufferSize.X);
        Largest.Y = core::cmp::min(Largest.Y, (*Buff).ScreenBufferSize.Y);
        (*GetCurrentFontRequest).FontSize = Largest;
    } else {
        TermGetFontSize(Console, core::ptr::addr_of_mut!((*GetCurrentFontRequest).FontSize));
    }

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleFont(
    ApiMessage_outer: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let _ = ReplyCode;
    let ProcessData_w = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console_w: PCONSRV_CONSOLE = core::ptr::null_mut();
    let Status = ConSrvGetConsole(ProcessData_w, core::ptr::addr_of_mut!(Console_w), 1u8);
    if !NT_SUCCESS(Status) { return Status; }
    let result = SrvSetConsoleFontImpl(ApiMessage_outer as *mut CONSOLE_API_MESSAGE, Console_w, ProcessData_w as *mut CONSOLE_PROCESS_DATA);
    ConSrvReleaseConsole(Console_w, 1u8);
    result
}

pub(crate) unsafe fn SrvSetConsoleFontImpl(
    ApiMessage: *mut CONSOLE_API_MESSAGE,
    _Console: PCONSRV_CONSOLE,
    ProcessData: *mut CONSOLE_PROCESS_DATA,
) -> i32 {
    let SetFontRequest = core::ptr::addr_of_mut!(
        (*ApiMessage).Data.SetFontRequest
    );
    
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();
    let Status = ConSrvGetTextModeBuffer(
        ProcessData,
        (*SetFontRequest).OutputHandle,
        &mut Buff as *mut _ as *mut PCONSOLE_IO_OBJECT,
        0x40000000, // GENERIC_WRITE
        1u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ConSrvReleaseScreenBuffer(Buff, 1u8);
    STATUS_SUCCESS
}