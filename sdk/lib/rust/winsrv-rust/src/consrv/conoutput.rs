use winsrv_sys::nt::*;
use super::condrv::text::*;
use super::condrv::conoutput::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) and re-export globs vs sibling Rust impl.
use super::{ConsoleAllocHeap, ConsoleFreeHeap, GetType};
use super::console::ConSrvGetConsole;
use super::console::ConSrvReleaseConsole;
use super::handle::{ConSrvGetObject, ConSrvReleaseObject, ConSrvInsertObject};
use super::condrv::conoutput::{ConDrvCreateScreenBuffer, ConDrvDeleteScreenBuffer};

// ---- consrv handle macros (from conoutput.h), mirrored as Rust macros ----
macro_rules! ConSrvGetTextModeBuffer {
    ($ProcessData:expr, $Handle:expr, $Ptr:expr, $Access:expr, $LockConsole:expr) => {
        ConSrvGetObject(
            $ProcessData,
            $Handle,
            $Ptr as *mut _ as *mut PCONSOLE_IO_OBJECT,
            core::ptr::null_mut(),
            $Access,
            $LockConsole,
            TEXTMODE_BUFFER,
        )
    };
}

macro_rules! ConSrvGetScreenBuffer {
    ($ProcessData:expr, $Handle:expr, $Ptr:expr, $Access:expr, $LockConsole:expr) => {
        ConSrvGetObject(
            $ProcessData,
            $Handle,
            $Ptr as *mut _ as *mut PCONSOLE_IO_OBJECT,
            core::ptr::null_mut(),
            $Access,
            $LockConsole,
            SCREEN_BUFFER,
        )
    };
}

macro_rules! ConSrvReleaseScreenBuffer {
    ($Buff:expr, $IsConsoleLocked:expr) => {
        ConSrvReleaseObject(
            core::ptr::addr_of_mut!((*$Buff).Header),
            $IsConsoleLocked,
        )
    };
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations needed for this translation unit.
//
// All functions reference types already imported from ntkrnl_sys (PCONSOLE, PTEXTMODE_SCREEN_BUFFER, etc.)
// and contain no static data, typedefs, enums, or constant definitions at file scope.

pub(crate) unsafe extern "system" fn WriteConsoleThread(
    _WaitList: PLIST_ENTRY,
    WaitThread: PCSR_THREAD,
    WaitApiMessage: PCSR_API_MESSAGE,
    _WaitContext: *mut core::ffi::c_void,
    _WaitArgument1: *mut core::ffi::c_void,
    _WaitArgument2: *mut core::ffi::c_void,
    WaitFlags: u32,
) -> u8 {
    let Status: i32;

    if (WaitFlags & CsrProcessTerminating) != 0 {
        Status = STATUS_THREAD_IS_TERMINATING;
        'Quit: {
            if Status != STATUS_PENDING {
                (*WaitApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
            }
            return if Status == STATUS_PENDING { 0u8 } else { 1u8 };
        }
    }

    Status = DoWriteConsole(WaitApiMessage, WaitThread, 0u8);

    if Status != STATUS_PENDING {
        (*WaitApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
    }

    if Status == STATUS_PENDING { 0u8 } else { 1u8 }
}

pub(crate) unsafe fn DoWriteConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ClientThread: PCSR_THREAD,
    CreateWaitBlock: u8,
) -> i32 {
    let mut Status: i32;
    let WriteConsoleRequest: PCONSOLE_WRITECONSOLE = 
        core::ptr::addr_of_mut!((*((ApiMessage as PCONSOLE_API_MESSAGE))).Data.WriteConsoleRequest);
    let mut ScreenBuffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    let Buffer: *mut core::ffi::c_void;
    let mut NrCharactersWritten: u32 = 0;
    let CharSize: u32 = if (*WriteConsoleRequest).Unicode != 0 {
        core::mem::size_of::<u16>() as u32
    } else {
        core::mem::size_of::<u8>() as u32
    };

    Status = ConSrvGetTextModeBuffer!(
        ConsoleGetPerProcessData((*ClientThread).Process),
        (*WriteConsoleRequest).OutputHandle,
        &mut ScreenBuffer,
        GENERIC_WRITE,
        0u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*WriteConsoleRequest).UsingStaticBuffer != 0
        && (*WriteConsoleRequest).NumBytes <= core::mem::size_of_val(&(*WriteConsoleRequest).StaticBuffer) as u32
    {
        Buffer = (*WriteConsoleRequest).StaticBuffer.as_mut_ptr() as *mut core::ffi::c_void;
    } else {
        Buffer = (*WriteConsoleRequest).Buffer;
    }

    Status = ConDrvWriteConsole(
        (*ScreenBuffer).Header.Console,
        ScreenBuffer,
        (*WriteConsoleRequest).Unicode,
        Buffer,
        (*WriteConsoleRequest).NumBytes / CharSize,
        &mut NrCharactersWritten,
    );

    if Status == STATUS_PENDING {
        if CreateWaitBlock != 0 {
            let Console: PCONSRV_CONSOLE = (*ScreenBuffer).Header.Console as PCONSRV_CONSOLE;

            if CsrCreateWait(
                core::ptr::addr_of_mut!((*Console).WriteWaitQueue),
                Some(WriteConsoleThread),
                ClientThread,
                ApiMessage,
                core::ptr::null_mut(),
            ) == 0
            {
                Status = STATUS_NO_MEMORY;
                ConSrvReleaseScreenBuffer!(ScreenBuffer, 0u8);
                return Status;
            }
        }
    } else {
        (*WriteConsoleRequest).NumBytes = NrCharactersWritten * CharSize;
    }

    ConSrvReleaseScreenBuffer!(ScreenBuffer, 0u8);
    Status
}

/* ===========================================================================
 * PUBLIC SERVER APIS (conoutput.c) — hand-translated CON_API handlers
 * ===========================================================================
 *
 * Each CON_API(Name, TYPE, RequestName) expands to:
 *   - `Name`     : the CSR entry point (#[no_mangle] extern "system"); acquires
 *                  the console via ConSrvGetConsole, calls `NameImpl`, releases.
 *   - `NameImpl` : the body (ProcessData, Console, ApiMessage, Request, ReplyCode).
 *
 * extern "system" callees `ConDrvWriteConsoleOutputVDM` and the CODE_TYPE field-size
 * RTL_FIELD_SIZE() macros are translated to direct size_of expressions.
 */

extern "system" {
    pub fn ConDrvWriteConsoleOutputVDM(
        Console: PCONSOLE,
        Buffer: PTEXTMODE_SCREEN_BUFFER,
        CharInfo: PCHAR_CELL,
        CharInfoSize: COORD,
        WriteRegion: PSMALL_RECT,
    ) -> NTSTATUS;
}

/* API_NUMBER: ConsolepInvalidateBitMapRect */
unsafe fn SrvInvalidateBitMapRectImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    InvalidateDIBitsRequest: *mut CONSOLE_INVALIDATEDIBITS,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetScreenBuffer!(
        ProcessData,
        (*InvalidateDIBitsRequest).OutputHandle,
        &mut Buffer,
        GENERIC_READ,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* In text-mode only, draw the VDM buffer if present */
    if GetType(Buffer) == TEXTMODE_BUFFER && !(*Console).VDMBuffer.is_null() {
        let TextBuffer = Buffer as PTEXTMODE_SCREEN_BUFFER;

        ConDrvWriteConsoleOutputVDM(
            Console as PCONSOLE,
            TextBuffer,
            (*Console).VDMBuffer as PCHAR_CELL,
            (*Console).VDMBufferSize,
            core::ptr::addr_of_mut!((*InvalidateDIBitsRequest).Region),
        );
    }

    Status = ConDrvInvalidateBitMapRect(
        Console as PCONSOLE,
        Buffer,
        core::ptr::addr_of_mut!((*InvalidateDIBitsRequest).Region),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvInvalidateBitMapRect(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let InvalidateDIBitsRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.InvalidateDIBitsRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvInvalidateBitMapRectImpl(
        ProcessData, Console, ApiMessage, InvalidateDIBitsRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetPalette */
unsafe fn SrvSetConsolePaletteImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetPaletteRequest: *mut CONSOLE_SETPALETTE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetScreenBuffer!(
        ProcessData,
        (*SetPaletteRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /*
     * Make the palette handle public, so that it can be
     * used by other threads calling GDI functions on it.
     */
    NtUserConsoleControl(
        ConsoleMakePalettePublic as CONSOLECONTROL,
        core::ptr::addr_of_mut!((*SetPaletteRequest).PaletteHandle) as PVOID,
        core::mem::size_of_val(&(*SetPaletteRequest).PaletteHandle) as ULONG,
    );

    Status = ConDrvSetConsolePalette(
        Console as PCONSOLE,
        Buffer,
        (*SetPaletteRequest).PaletteHandle,
        (*SetPaletteRequest).Usage,
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsolePalette(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetPaletteRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetPaletteRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsolePaletteImpl(
        ProcessData, Console, ApiMessage, SetPaletteRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepGetCursorInfo */
unsafe fn SrvGetConsoleCursorInfoImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    CursorInfoRequest: *mut CONSOLE_GETSETCURSORINFO,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*CursorInfoRequest).OutputHandle,
        &mut Buffer,
        GENERIC_READ,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvGetConsoleCursorInfo(
        Console as PCONSOLE,
        Buffer,
        core::ptr::addr_of_mut!((*CursorInfoRequest).Info),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleCursorInfo(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let CursorInfoRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.CursorInfoRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvGetConsoleCursorInfoImpl(
        ProcessData, Console, ApiMessage, CursorInfoRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetCursorInfo */
unsafe fn SrvSetConsoleCursorInfoImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    CursorInfoRequest: *mut CONSOLE_GETSETCURSORINFO,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*CursorInfoRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvSetConsoleCursorInfo(
        Console as PCONSOLE,
        Buffer,
        core::ptr::addr_of_mut!((*CursorInfoRequest).Info),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleCursorInfo(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let CursorInfoRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.CursorInfoRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsoleCursorInfoImpl(
        ProcessData, Console, ApiMessage, CursorInfoRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetCursorPosition */
unsafe fn SrvSetConsoleCursorPositionImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetCursorPositionRequest: *mut CONSOLE_SETCURSORPOSITION,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*SetCursorPositionRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvSetConsoleCursorPosition(
        Console as PCONSOLE,
        Buffer,
        core::ptr::addr_of_mut!((*SetCursorPositionRequest).Position),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleCursorPosition(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetCursorPositionRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetCursorPositionRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsoleCursorPositionImpl(
        ProcessData, Console, ApiMessage, SetCursorPositionRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepCreateScreenBuffer */
unsafe fn SrvCreateConsoleScreenBufferImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    CreateScreenBufferRequest: *mut CONSOLE_CREATESCREENBUFFER,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Process: PCSR_PROCESS = (*CsrGetClientThread()).Process;
    let mut Buff: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();

    let mut ScreenBufferInfo: PVOID = core::ptr::null_mut();
    let mut TextModeInfo: TEXTMODE_BUFFER_INFO = TEXTMODE_BUFFER_INFO {
        ScreenBufferSize: COORD { X: 80, Y: 25 },
        ViewSize: COORD { X: 80, Y: 25 },
        ScreenAttrib: DEFAULT_SCREEN_ATTRIB as USHORT,
        PopupAttrib: DEFAULT_POPUP_ATTRIB as USHORT,
        CursorSize: CSR_DEFAULT_CURSOR_SIZE,
        IsCursorVisible: 1u8,
    };
    let mut GraphicsInfo: GRAPHICS_BUFFER_INFO = core::mem::zeroed();
    GraphicsInfo.Info = (*CreateScreenBufferRequest).GraphicsBufferInfo; // HACK for MSVC

    if (*CreateScreenBufferRequest).ScreenBufferType == CONSOLE_TEXTMODE_BUFFER {
        ScreenBufferInfo = core::ptr::addr_of_mut!(TextModeInfo) as PVOID;

        /* If we have an active screen buffer, use its attributes as the new ones */
        if !(*Console).ActiveBuffer.is_null()
            && GetType((*Console).ActiveBuffer) == TEXTMODE_BUFFER
        {
            let Buffer = (*Console).ActiveBuffer as PTEXTMODE_SCREEN_BUFFER;

            TextModeInfo.ScreenAttrib = (*Buffer).ScreenDefaultAttrib;
            TextModeInfo.PopupAttrib = (*Buffer).PopupDefaultAttrib;

            TextModeInfo.CursorSize = (*Buffer).CursorInfo.dwSize;
            TextModeInfo.IsCursorVisible = if (*Buffer).CursorInfo.bVisible != 0 { 1u8 } else { 0u8 };

            /* Use the current view size */
            TextModeInfo.ScreenBufferSize = (*Buffer).ViewSize;
            TextModeInfo.ViewSize = (*Buffer).ViewSize;
        } else {
            /* Use the current console size */
            TextModeInfo.ScreenBufferSize = (*Console).ConsoleSize;
            TextModeInfo.ViewSize = (*Console).ConsoleSize;
        }

        /* Normalize the screen buffer size if needed */
        if TextModeInfo.ScreenBufferSize.X == 0 { TextModeInfo.ScreenBufferSize.X = 1; }
        if TextModeInfo.ScreenBufferSize.Y == 0 { TextModeInfo.ScreenBufferSize.Y = 1; }
    } else if (*CreateScreenBufferRequest).ScreenBufferType == CONSOLE_GRAPHICS_BUFFER {
        /* Get information from the graphics buffer information structure */
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*CreateScreenBufferRequest).GraphicsBufferInfo.lpBitMapInfo) as *mut PVOID,
            (*CreateScreenBufferRequest).GraphicsBufferInfo.dwBitMapInfoLength,
            core::mem::size_of::<u8>() as ULONG,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }

        ScreenBufferInfo = core::ptr::addr_of_mut!(GraphicsInfo) as PVOID;

        /* Initialize shared variables */
        GraphicsInfo.Info.hMutex = INVALID_HANDLE_VALUE;
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*CreateScreenBufferRequest).hMutex),
            INVALID_HANDLE_VALUE,
        );
        GraphicsInfo.Info.lpBitMap = core::ptr::null_mut();
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*CreateScreenBufferRequest).lpBitMap),
            core::ptr::null_mut(),
        );

        /* A graphics screen buffer is never inheritable */
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*CreateScreenBufferRequest).InheritHandle),
            0,
        );
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ConDrvCreateScreenBuffer(
        core::ptr::addr_of_mut!(Buff),
        Console as PCONSOLE,
        (*Process).ProcessHandle,
        (*CreateScreenBufferRequest).ScreenBufferType,
        ScreenBufferInfo,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Insert the new handle inside the process handles table */
    RtlEnterCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    Status = ConSrvInsertObject(
        ProcessData,
        core::ptr::addr_of_mut!((*CreateScreenBufferRequest).OutputHandle),
        core::ptr::addr_of_mut!((*Buff).Header),
        (*CreateScreenBufferRequest).DesiredAccess,
        if (*CreateScreenBufferRequest).InheritHandle != 0 { 1u8 } else { 0u8 },
        (*CreateScreenBufferRequest).ShareMode,
    );

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!((*ProcessData).HandleTableLock));

    if !NT_SUCCESS(Status) {
        ConDrvDeleteScreenBuffer(Buff);
        return Status;
    }

    if (*CreateScreenBufferRequest).ScreenBufferType == CONSOLE_GRAPHICS_BUFFER {
        let Buffer = Buff as PGRAPHICS_SCREEN_BUFFER;
        /*
         * Initialize the graphics buffer information structure
         * and give it back to the client.
         */
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*CreateScreenBufferRequest).hMutex),
            (*Buffer).ClientMutex,
        );
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*CreateScreenBufferRequest).lpBitMap),
            (*Buffer).ClientBitMap,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvCreateConsoleScreenBuffer(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let CreateScreenBufferRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.CreateScreenBufferRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvCreateConsoleScreenBufferImpl(
        ProcessData, Console, ApiMessage, CreateScreenBufferRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetActiveScreenBuffer */
unsafe fn SrvSetConsoleActiveScreenBufferImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetScreenBufferRequest: *mut CONSOLE_SETACTIVESCREENBUFFER,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetScreenBuffer!(
        ProcessData,
        (*SetScreenBufferRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvSetConsoleActiveScreenBuffer(Console as PCONSOLE, Buffer);

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleActiveScreenBuffer(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetScreenBufferRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetScreenBufferRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsoleActiveScreenBufferImpl(
        ProcessData, Console, ApiMessage, SetScreenBufferRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepReadConsoleOutput */
unsafe fn SrvReadConsoleOutputImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    ReadOutputRequest: *mut CONSOLE_READOUTPUT,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();
    let NumCells: ULONG;
    let CharInfo: PCHAR_INFO;

    NumCells = (ConioRectWidth(core::ptr::addr_of_mut!((*ReadOutputRequest).ReadRegion)) as ULONG)
        .wrapping_mul(ConioRectHeight(core::ptr::addr_of_mut!((*ReadOutputRequest).ReadRegion)) as ULONG);

    if NumCells <= 1 {
        CharInfo = core::ptr::addr_of_mut!((*ReadOutputRequest).StaticBuffer);
    } else {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*ReadOutputRequest).CharInfo) as *mut PVOID,
            NumCells,
            core::mem::size_of::<CHAR_INFO>() as ULONG,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }

        CharInfo = (*ReadOutputRequest).CharInfo;
    }

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*ReadOutputRequest).OutputHandle,
        &mut Buffer,
        GENERIC_READ,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvReadConsoleOutput(
        Console as PCONSOLE,
        Buffer,
        (*ReadOutputRequest).Unicode,
        CharInfo,
        core::ptr::addr_of_mut!((*ReadOutputRequest).ReadRegion),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvReadConsoleOutput(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ReadOutputRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ReadOutputRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvReadConsoleOutputImpl(
        ProcessData, Console, ApiMessage, ReadOutputRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepWriteConsoleOutput */
unsafe fn SrvWriteConsoleOutputImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    WriteOutputRequest: *mut CONSOLE_WRITEOUTPUT,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Process: PCSR_PROCESS = (*CsrGetClientThread()).Process;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();
    let NumCells: ULONG;
    let CharInfo: PCHAR_INFO;

    NumCells = (ConioRectWidth(core::ptr::addr_of_mut!((*WriteOutputRequest).WriteRegion)) as ULONG)
        .wrapping_mul(ConioRectHeight(core::ptr::addr_of_mut!((*WriteOutputRequest).WriteRegion)) as ULONG);

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*WriteOutputRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*WriteOutputRequest).UseVirtualMemory == 0 {
        if NumCells <= 1 {
            CharInfo = core::ptr::addr_of_mut!((*WriteOutputRequest).StaticBuffer);
        } else {
            if CsrValidateMessageBuffer(
                ApiMessage,
                core::ptr::addr_of_mut!((*WriteOutputRequest).CharInfo) as *mut PVOID,
                NumCells,
                core::mem::size_of::<CHAR_INFO>() as ULONG,
            ) == 0
            {
                Status = STATUS_INVALID_PARAMETER;
                ConSrvReleaseScreenBuffer!(Buffer, 1u8);
                return Status;
            }

            CharInfo = (*WriteOutputRequest).CharInfo;
        }
    } else {
        /* This was not the case: we use a heap buffer. Retrieve its contents. */
        let Size: ULONG = NumCells.wrapping_mul(core::mem::size_of::<CHAR_INFO>() as ULONG);

        CharInfo = ConsoleAllocHeap(HEAP_ZERO_MEMORY, Size as usize) as PCHAR_INFO;
        if CharInfo.is_null() {
            Status = STATUS_NO_MEMORY;
            ConSrvReleaseScreenBuffer!(Buffer, 1u8);
            return Status;
        }

        Status = NtReadVirtualMemory(
            (*Process).ProcessHandle,
            (*WriteOutputRequest).CharInfo as PVOID,
            CharInfo as PVOID,
            Size as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            ConsoleFreeHeap(CharInfo as PVOID);
            ConSrvReleaseScreenBuffer!(Buffer, 1u8);
            return Status;
        }
    }

    Status = ConDrvWriteConsoleOutput(
        Console as PCONSOLE,
        Buffer,
        (*WriteOutputRequest).Unicode,
        CharInfo,
        core::ptr::addr_of_mut!((*WriteOutputRequest).WriteRegion),
    );

    /* Free the temporary buffer if we used the process' heap buffer */
    if (*WriteOutputRequest).UseVirtualMemory != 0 && !CharInfo.is_null() {
        ConsoleFreeHeap(CharInfo as PVOID);
    }

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvWriteConsoleOutput(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let WriteOutputRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.WriteOutputRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvWriteConsoleOutputImpl(
        ProcessData, Console, ApiMessage, WriteOutputRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepWriteConsole */
unsafe fn SrvWriteConsoleImpl(
    _ProcessData: PCONSOLE_PROCESS_DATA,
    _Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    WriteConsoleRequest: *mut CONSOLE_WRITECONSOLE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if (*WriteConsoleRequest).UsingStaticBuffer != 0
        && (*WriteConsoleRequest).NumBytes <= core::mem::size_of_val(&(*WriteConsoleRequest).StaticBuffer) as ULONG
    {
        /* Adjust the internal pointer (static buffer in the original ApiMessage). */
    } else {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*WriteConsoleRequest).Buffer) as *mut PVOID,
            (*WriteConsoleRequest).NumBytes,
            core::mem::size_of::<u8>() as ULONG,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }
    }

    Status = DoWriteConsole(ApiMessage, CsrGetClientThread(), 1u8);

    if Status == STATUS_PENDING {
        if !ReplyCode.is_null() {
            core::ptr::write_volatile(ReplyCode, CsrReplyPending);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvWriteConsole(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let WriteConsoleRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.WriteConsoleRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvWriteConsoleImpl(
        ProcessData, Console, ApiMessage, WriteConsoleRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepReadConsoleOutputString */
unsafe fn SrvReadConsoleOutputStringImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    ReadOutputCodeRequest: *mut CONSOLE_READOUTPUTCODE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();
    let CodeSize: ULONG;
    let pCode: PVOID;

    match (*ReadOutputCodeRequest).CodeType {
        x if x == CODE_ASCII as CODE_TYPE => {
            CodeSize = core::mem::size_of::<CHAR>() as ULONG;
        }
        x if x == CODE_UNICODE as CODE_TYPE => {
            CodeSize = core::mem::size_of::<WCHAR>() as ULONG;
        }
        x if x == CODE_ATTRIBUTE as CODE_TYPE => {
            CodeSize = core::mem::size_of::<WORD>() as ULONG;
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    if (*ReadOutputCodeRequest).NumCodes.wrapping_mul(CodeSize)
        <= core::mem::size_of_val(&(*ReadOutputCodeRequest).CodeStaticBuffer) as ULONG
    {
        pCode = (*ReadOutputCodeRequest).CodeStaticBuffer.as_mut_ptr() as PVOID;
    } else {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*ReadOutputCodeRequest).pCode) as *mut PVOID,
            (*ReadOutputCodeRequest).NumCodes,
            CodeSize,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }

        pCode = (*ReadOutputCodeRequest).pCode;
    }

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*ReadOutputCodeRequest).OutputHandle,
        &mut Buffer,
        GENERIC_READ,
        1u8
    );
    if !NT_SUCCESS(Status) {
        (*ReadOutputCodeRequest).NumCodes = 0;
        return Status;
    }

    Status = ConDrvReadConsoleOutputString(
        Console as PCONSOLE,
        Buffer,
        (*ReadOutputCodeRequest).CodeType,
        pCode,
        (*ReadOutputCodeRequest).NumCodes,
        core::ptr::addr_of_mut!((*ReadOutputCodeRequest).Coord),
        core::ptr::addr_of_mut!((*ReadOutputCodeRequest).NumCodes),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvReadConsoleOutputString(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ReadOutputCodeRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ReadOutputCodeRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvReadConsoleOutputStringImpl(
        ProcessData, Console, ApiMessage, ReadOutputCodeRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepWriteConsoleOutputString */
unsafe fn SrvWriteConsoleOutputStringImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    ApiMessage: PCSR_API_MESSAGE,
    WriteOutputCodeRequest: *mut CONSOLE_WRITEOUTPUTCODE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();
    let CodeSize: ULONG;
    let pCode: PVOID;

    match (*WriteOutputCodeRequest).CodeType {
        x if x == CODE_ASCII as CODE_TYPE => {
            CodeSize = core::mem::size_of::<CHAR>() as ULONG;
        }
        x if x == CODE_UNICODE as CODE_TYPE => {
            CodeSize = core::mem::size_of::<WCHAR>() as ULONG;
        }
        x if x == CODE_ATTRIBUTE as CODE_TYPE => {
            CodeSize = core::mem::size_of::<WORD>() as ULONG;
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    if (*WriteOutputCodeRequest).NumCodes.wrapping_mul(CodeSize)
        <= core::mem::size_of_val(&(*WriteOutputCodeRequest).CodeStaticBuffer) as ULONG
    {
        pCode = (*WriteOutputCodeRequest).CodeStaticBuffer.as_mut_ptr() as PVOID;
    } else {
        if CsrValidateMessageBuffer(
            ApiMessage,
            core::ptr::addr_of_mut!((*WriteOutputCodeRequest).pCode) as *mut PVOID,
            (*WriteOutputCodeRequest).NumCodes,
            CodeSize,
        ) == 0
        {
            return STATUS_INVALID_PARAMETER;
        }

        pCode = (*WriteOutputCodeRequest).pCode;
    }

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*WriteOutputCodeRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        (*WriteOutputCodeRequest).NumCodes = 0;
        return Status;
    }

    Status = ConDrvWriteConsoleOutputString(
        Console as PCONSOLE,
        Buffer,
        (*WriteOutputCodeRequest).CodeType,
        pCode,
        (*WriteOutputCodeRequest).NumCodes,
        core::ptr::addr_of_mut!((*WriteOutputCodeRequest).Coord),
        core::ptr::addr_of_mut!((*WriteOutputCodeRequest).NumCodes),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvWriteConsoleOutputString(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let WriteOutputCodeRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.WriteOutputCodeRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvWriteConsoleOutputStringImpl(
        ProcessData, Console, ApiMessage, WriteOutputCodeRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepFillConsoleOutput */
unsafe fn SrvFillConsoleOutputImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    FillOutputRequest: *mut CONSOLE_FILLOUTPUTCODE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();
    let CodeType: CODE_TYPE = (*FillOutputRequest).CodeType;

    if CodeType != CODE_ASCII as CODE_TYPE
        && CodeType != CODE_UNICODE as CODE_TYPE
        && CodeType != CODE_ATTRIBUTE as CODE_TYPE
    {
        return STATUS_INVALID_PARAMETER;
    }

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*FillOutputRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        (*FillOutputRequest).NumCodes = 0;
        return Status;
    }

    Status = ConDrvFillConsoleOutput(
        Console as PCONSOLE,
        Buffer,
        CodeType,
        (*FillOutputRequest).Code,
        (*FillOutputRequest).NumCodes,
        core::ptr::addr_of_mut!((*FillOutputRequest).WriteCoord),
        core::ptr::addr_of_mut!((*FillOutputRequest).NumCodes),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvFillConsoleOutput(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let FillOutputRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.FillOutputRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvFillConsoleOutputImpl(
        ProcessData, Console, ApiMessage, FillOutputRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepGetScreenBufferInfo */
unsafe fn SrvGetConsoleScreenBufferInfoImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    ScreenBufferInfoRequest: *mut CONSOLE_GETSCREENBUFFERINFO,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*ScreenBufferInfoRequest).OutputHandle,
        &mut Buffer,
        GENERIC_READ,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvGetConsoleScreenBufferInfo(
        Console as PCONSOLE,
        Buffer,
        core::ptr::addr_of_mut!((*ScreenBufferInfoRequest).ScreenBufferSize),
        core::ptr::addr_of_mut!((*ScreenBufferInfoRequest).CursorPosition),
        core::ptr::addr_of_mut!((*ScreenBufferInfoRequest).ViewOrigin),
        core::ptr::addr_of_mut!((*ScreenBufferInfoRequest).ViewSize),
        core::ptr::addr_of_mut!((*ScreenBufferInfoRequest).MaximumViewSize),
        core::ptr::addr_of_mut!((*ScreenBufferInfoRequest).Attributes),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvGetConsoleScreenBufferInfo(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ScreenBufferInfoRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ScreenBufferInfoRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvGetConsoleScreenBufferInfoImpl(
        ProcessData, Console, ApiMessage, ScreenBufferInfoRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetTextAttribute */
unsafe fn SrvSetConsoleTextAttributeImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetTextAttribRequest: *mut CONSOLE_SETTEXTATTRIB,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*SetTextAttribRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvSetConsoleTextAttribute(
        Console as PCONSOLE,
        Buffer,
        (*SetTextAttribRequest).Attributes,
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleTextAttribute(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetTextAttribRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetTextAttribRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsoleTextAttributeImpl(
        ProcessData, Console, ApiMessage, SetTextAttribRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetScreenBufferSize */
unsafe fn SrvSetConsoleScreenBufferSizeImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetScreenBufferSizeRequest: *mut CONSOLE_SETSCREENBUFFERSIZE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*SetScreenBufferSizeRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvSetConsoleScreenBufferSize(
        Console as PCONSOLE,
        Buffer,
        core::ptr::addr_of_mut!((*SetScreenBufferSizeRequest).Size),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleScreenBufferSize(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetScreenBufferSizeRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetScreenBufferSizeRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsoleScreenBufferSizeImpl(
        ProcessData, Console, ApiMessage, SetScreenBufferSizeRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepScrollScreenBuffer */
unsafe fn SrvScrollConsoleScreenBufferImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    ScrollScreenBufferRequest: *mut CONSOLE_SCROLLSCREENBUFFER,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*ScrollScreenBufferRequest).OutputHandle,
        &mut Buffer,
        GENERIC_WRITE,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvScrollConsoleScreenBuffer(
        Console as PCONSOLE,
        Buffer,
        (*ScrollScreenBufferRequest).Unicode,
        core::ptr::addr_of_mut!((*ScrollScreenBufferRequest).ScrollRectangle),
        if (*ScrollScreenBufferRequest).UseClipRectangle != 0 { 1u8 } else { 0u8 },
        core::ptr::addr_of_mut!((*ScrollScreenBufferRequest).ClipRectangle),
        core::ptr::addr_of_mut!((*ScrollScreenBufferRequest).DestinationOrigin),
        (*ScrollScreenBufferRequest).Fill,
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvScrollConsoleScreenBuffer(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let ScrollScreenBufferRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.ScrollScreenBufferRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvScrollConsoleScreenBufferImpl(
        ProcessData, Console, ApiMessage, ScrollScreenBufferRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}

/* API_NUMBER: ConsolepSetWindowInfo */
unsafe fn SrvSetConsoleWindowInfoImpl(
    ProcessData: PCONSOLE_PROCESS_DATA,
    Console: PCONSRV_CONSOLE,
    _ApiMessage: PCSR_API_MESSAGE,
    SetWindowInfoRequest: *mut CONSOLE_SETWINDOWINFO,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Buffer: PTEXTMODE_SCREEN_BUFFER = core::ptr::null_mut();

    Status = ConSrvGetTextModeBuffer!(
        ProcessData,
        (*SetWindowInfoRequest).OutputHandle,
        &mut Buffer,
        GENERIC_READ,
        1u8
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ConDrvSetConsoleWindowInfo(
        Console as PCONSOLE,
        Buffer,
        if (*SetWindowInfoRequest).Absolute != 0 { 1u8 } else { 0u8 },
        core::ptr::addr_of_mut!((*SetWindowInfoRequest).WindowRect),
    );

    ConSrvReleaseScreenBuffer!(Buffer, 1u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleWindowInfo(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let SetWindowInfoRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as PCONSOLE_API_MESSAGE)).Data.SetWindowInfoRequest
    );
    let ProcessData = ConsoleGetPerProcessData((*CsrGetClientThread()).Process);
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    let Status = ConSrvGetConsole(ProcessData, core::ptr::addr_of_mut!(Console), 1u8);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    let Status = SrvSetConsoleWindowInfoImpl(
        ProcessData, Console, ApiMessage, SetWindowInfoRequest, ReplyCode);

    ConSrvReleaseConsole(Console, 1u8);
    Status
}
