use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

unsafe fn InitializeListHead(list: *mut LIST_ENTRY) {
    (*list).Flink = list;
    (*list).Blink = list;
}

unsafe fn IsListEmpty(list: *const LIST_ENTRY) -> bool {
    (*list).Flink == list
}

unsafe fn RtlZeroMemory(dst: *mut core::ffi::c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

extern "C" {
    fn ResetTerminal(Console: PCONSOLE);
    fn InitializeCriticalSection(cs: *mut RTL_CRITICAL_SECTION);
    fn EnterCriticalSection(cs: *mut RTL_CRITICAL_SECTION);
    fn LeaveCriticalSection(cs: *mut RTL_CRITICAL_SECTION);
    fn DeleteCriticalSection(cs: *mut RTL_CRITICAL_SECTION);
    fn TermSetCodePage(Console: PCONSOLE, CodePage: UINT) -> BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvValidateConsoleState(
    Console: PCONSOLE,
    ExpectedState: CONSOLE_STATE,
) -> BOOLEAN {
    ((*Console).State == ExpectedState) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvValidateConsoleUnsafe(
    Console: PCONSOLE,
    ExpectedState: CONSOLE_STATE,
    LockConsole: BOOLEAN,
) -> BOOLEAN {
    if Console.is_null() {
        return 0u8;
    }

    if LockConsole != 0 {
        EnterCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
    }

    if ConDrvValidateConsoleState(Console, ExpectedState) == 0 {
        if LockConsole != 0 {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        }
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvInitConsole(
    Console: PCONSOLE,
    ConsoleInfo: PCONSOLE_INFO,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ScreenBufferInfo: TEXTMODE_BUFFER_INFO;
    let mut NewBuffer: PCONSOLE_SCREEN_BUFFER = core::ptr::null_mut();

    if Console.is_null() || ConsoleInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    RtlZeroMemory(Console as *mut core::ffi::c_void, core::mem::size_of::<CONSOLE>());

    if (*ConsoleInfo).ScreenBufferSize.X == 0 {
        (*ConsoleInfo).ScreenBufferSize.X = 1;
    }
    if (*ConsoleInfo).ScreenBufferSize.Y == 0 {
        (*ConsoleInfo).ScreenBufferSize.Y = 1;
    }
    if (*ConsoleInfo).ScreenBufferSize.X < (*ConsoleInfo).ConsoleSize.X {
        (*ConsoleInfo).ScreenBufferSize.X = (*ConsoleInfo).ConsoleSize.X;
    }
    if (*ConsoleInfo).ScreenBufferSize.Y < (*ConsoleInfo).ConsoleSize.Y {
        (*ConsoleInfo).ScreenBufferSize.Y = (*ConsoleInfo).ConsoleSize.Y;
    }

    (*Console).State = CONSOLE_INITIALIZING;
    (*Console).ReferenceCount = 0;
    InitializeCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));

    ResetTerminal(Console);

    (*Console).ConsoleSize = (*ConsoleInfo).ConsoleSize;
    (*Console).FixedSize = 0u8;

    Status = ConDrvInitInputBuffer(Console, 0);
    if Status < 0 {
        DeleteCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        return Status;
    }

    if IsValidCodePage((*ConsoleInfo).CodePage) != 0 {
        (*Console).OutputCodePage = (*ConsoleInfo).CodePage;
        (*Console).InputCodePage = (*ConsoleInfo).CodePage;
    }

    ScreenBufferInfo.ScreenBufferSize = (*ConsoleInfo).ScreenBufferSize;
    ScreenBufferInfo.ViewSize = (*ConsoleInfo).ConsoleSize;
    ScreenBufferInfo.ScreenAttrib = (*ConsoleInfo).ScreenAttrib;
    ScreenBufferInfo.PopupAttrib = (*ConsoleInfo).PopupAttrib;
    ScreenBufferInfo.CursorSize = (*ConsoleInfo).CursorSize;
    ScreenBufferInfo.IsCursorVisible = 1u8;

    InitializeListHead(core::ptr::addr_of_mut!((*Console).BufferList));
    Status = ConDrvCreateScreenBuffer(
        core::ptr::addr_of_mut!(NewBuffer),
        Console,
        core::ptr::null_mut(),
        CONSOLE_TEXTMODE_BUFFER,
        core::ptr::addr_of_mut!(ScreenBufferInfo) as *mut core::ffi::c_void,
    );
    if Status < 0 {
        ConDrvDeinitInputBuffer(Console);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        return Status;
    }

    (*Console).ActiveBuffer = NewBuffer;
    (*Console).ConsolePaused = 0u8;

    (*Console).State = CONSOLE_RUNNING;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvAttachTerminal(
    Console: PCONSOLE,
    Terminal: PTERMINAL,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if Console.is_null() || Terminal.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    (*Console).TermIFace = *Terminal;
    (*Console).TermIFace.Console = Console;

    Status = ((*(*Console).TermIFace.Vtbl).InitTerminal.unwrap())(
        core::ptr::addr_of_mut!((*Console).TermIFace),
        Console,
    );
    if Status < 0 {
        (*Terminal).Console = core::ptr::null_mut();
        ResetTerminal(Console);
        return Status;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvDetachTerminal(Console: PCONSOLE) -> NTSTATUS {
    if Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    ((*(*Console).TermIFace.Vtbl).DeinitTerminal.unwrap())(
        core::ptr::addr_of_mut!((*Console).TermIFace),
    );

    ResetTerminal(Console);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvDeleteConsole(Console: PCONSOLE) {
    if ConDrvValidateConsoleUnsafe(Console, CONSOLE_RUNNING, 1u8) == 0
        && ConDrvValidateConsoleUnsafe(Console, CONSOLE_INITIALIZING, 1u8) == 0
    {
        return;
    }

    (*Console).State = CONSOLE_TERMINATING;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));

    ConDrvDetachTerminal(Console);

    if ConDrvValidateConsoleUnsafe(Console, CONSOLE_TERMINATING, 1u8) == 0 {
        return;
    }

    (*Console).State = CONSOLE_IN_DESTRUCTION;

    (*Console).ReferenceCount = 0;

    ConDrvDeleteScreenBuffer((*Console).ActiveBuffer);
    (*Console).ActiveBuffer = core::ptr::null_mut();
    if !IsListEmpty(core::ptr::addr_of!((*Console).BufferList)) {
        core::debug_assert!(false, "BUGBUGBUG!! screen buffer list not empty");
    }

    ConDrvDeinitInputBuffer(Console);

    (*Console).ConsolePaused = 0u8;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
    DeleteCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvPause(Console: PCONSOLE) {
    if (*Console).ConsolePaused != 0 {
        return;
    }
    (*Console).ConsolePaused = 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvUnpause(Console: PCONSOLE) {
    if (*Console).ConsolePaused == 0 {
        return;
    }
    (*Console).ConsolePaused = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetConsoleMode(
    Console: PCONSOLE,
    Object: PCONSOLE_IO_OBJECT,
    ConsoleMode: PUINT,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    if Console.is_null() || Object.is_null() || ConsoleMode.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *ConsoleMode = 0;

    if (*Object).Type == INPUT_BUFFER {
        let InputBuffer = Object as PCONSOLE_INPUT_BUFFER;
        *ConsoleMode = (*InputBuffer).Mode;
    } else if (*Object).Type == TEXTMODE_BUFFER || (*Object).Type == GRAPHICS_BUFFER {
        let Buffer = Object as PCONSOLE_SCREEN_BUFFER;
        *ConsoleMode = (*Buffer).Mode;
    } else {
        Status = STATUS_INVALID_HANDLE;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleMode(
    Console: PCONSOLE,
    Object: PCONSOLE_IO_OBJECT,
    ConsoleMode: ULONG,
) -> NTSTATUS {
    const CONSOLE_VALID_INPUT_MODES: ULONG = 0x0000001F;
    const CONSOLE_VALID_OUTPUT_MODES: ULONG = 0x00000003;

    let mut Status = STATUS_SUCCESS;

    if Console.is_null() || Object.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*Object).Type == INPUT_BUFFER {
        let InputBuffer = Object as PCONSOLE_INPUT_BUFFER;

        if (ConsoleMode & !CONSOLE_VALID_INPUT_MODES) != 0 {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            (*InputBuffer).Mode = ConsoleMode & CONSOLE_VALID_INPUT_MODES;
        }
    } else if (*Object).Type == TEXTMODE_BUFFER || (*Object).Type == GRAPHICS_BUFFER {
        let Buffer = Object as PCONSOLE_SCREEN_BUFFER;

        if (ConsoleMode & !CONSOLE_VALID_OUTPUT_MODES) != 0 {
            Status = STATUS_INVALID_PARAMETER;
        } else {
            (*Buffer).Mode = ConsoleMode & CONSOLE_VALID_OUTPUT_MODES;
        }
    } else {
        Status = STATUS_INVALID_HANDLE;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvGetConsoleCP(
    Console: PCONSOLE,
    CodePage: PUINT,
    OutputCP: BOOLEAN,
) -> NTSTATUS {
    if Console.is_null() || CodePage.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *CodePage = if OutputCP != 0 {
        (*Console).OutputCodePage
    } else {
        (*Console).InputCodePage
    };

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ConDrvSetConsoleCP(
    Console: PCONSOLE,
    CodePage: UINT,
    OutputCP: BOOLEAN,
) -> NTSTATUS {
    if Console.is_null() || IsValidCodePage(CodePage) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if OutputCP != 0 {
        if TermSetCodePage(Console, CodePage) == 0 {
            return STATUS_UNSUCCESSFUL;
        }

        (*Console).OutputCodePage = CodePage;
        STATUS_SUCCESS
    } else {
        (*Console).InputCodePage = CodePage;
        STATUS_SUCCESS
    }
}