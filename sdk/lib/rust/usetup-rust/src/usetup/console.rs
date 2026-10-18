use spapisup_rust::spapisup::cabinet::CabinetClose;
use spapisup_rust::spapisup::cabinet::CabinetExtractFile;
use spapisup_rust::spapisup::cabinet::CabinetFindFirst;
use spapisup_rust::spapisup::cabinet::CabinetInitialize;
use spapisup_rust::spapisup::cabinet::CabinetOpen;
use spapisup_rust::spapisup::cabinet::CabinetSetCabinetName;
use spapisup_rust::spapisup::cabinet::CabinetSetEventHandlers;
use crate::usetup::keytrans::IntTranslateKey;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_38821E379CB2: [u16; 15] = [
    0x005Cu16, 0x003F, 0x003F, 0x005C, 0x0042, 0x006C, 0x0075, 0x0065, 0x0053, 0x0063,
    0x0072, 0x0065, 0x0065, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_7A95474577CC: [u16; 11] = [
    0x0025u16, 0x0075, 0x002D, 0x0038, 0x0078, 0x0038, 0x002E, 0x0062, 0x0069, 0x006E,
    0x0000,
];
pub static RUSTCONV_WSTR_8CB72516B703: [u16; 23] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x004B, 0x0065,
    0x0079, 0x0062, 0x006F, 0x0061, 0x0072, 0x0064, 0x0043, 0x006C, 0x0061, 0x0073,
    0x0073, 0x0030, 0x0000,
];
pub static RUSTCONV_WSTR_9618145ECC2C: [u16; 25] = [
    0x005Cu16, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F,
    0x0074, 0x005C, 0x0076, 0x0067, 0x0061, 0x0066, 0x006F, 0x006E, 0x0074, 0x0073,
    0x002E, 0x0063, 0x0061, 0x0062, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_15 {
    pub Status: NTSTATUS,
    pub Pointer: PVOID,
}

static mut InputQueueEmpty: BOOLEAN = 0;

static mut WaitForInput: BOOLEAN = 0;

static mut InputDataQueue: KEYBOARD_INPUT_DATA = _KEYBOARD_INPUT_DATA {
    UnitId: 0,
    MakeCode: 0,
    Flags: 0,
    Reserved: 0,
    ExtraInformation: 0,
};

static mut InputIosb: _IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

static mut LastLoadedCodepage: UINT = 0;

#[repr(C)]
pub struct _CONSOLE_CABINET_CONTEXT {
    pub CabinetContext: CABINET_CONTEXT,
    pub Data: PVOID,
    pub Size: ULONG,
}
pub type CONSOLE_CABINET_CONTEXT = _CONSOLE_CABINET_CONTEXT;
pub type PCONSOLE_CABINET_CONTEXT = *mut _CONSOLE_CABINET_CONTEXT;

unsafe extern "C" fn ConsoleCreateFileHandler(
    CabinetContext: PCABINET_CONTEXT,
    FileSize: ULONG,
) -> PVOID {
    let ConsoleCabinetContext: PCONSOLE_CABINET_CONTEXT = CabinetContext as PCONSOLE_CABINET_CONTEXT;
    (*ConsoleCabinetContext).Data = RtlAllocateHeap(ProcessHeap, 0, (FileSize) as SIZE_T);
    if (*ConsoleCabinetContext).Data.is_null() {
        DPRINT!("Failed to allocate %d bytes\n", FileSize);
        return core::ptr::null_mut();
    }
    (*ConsoleCabinetContext).Size = FileSize;
    return (*ConsoleCabinetContext).Data;
}

#[no_mangle]
pub unsafe extern "system" fn AllocConsole() -> BOOL {
    let mut Status: i32;
    let mut ScreenName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_38821E379CB2.as_ptr());
    let mut KeyboardName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_8CB72516B703.as_ptr());
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Enable: u32;

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut ScreenName,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenFile(
        &mut StdOutput,
        FILE_ALL_ACCESS,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_OPEN,
        FILE_SYNCHRONOUS_IO_ALERT,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    Enable = TRUE;
    Status = NtDeviceIoControlFile(
        StdOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_RESET_SCREEN) as ULONG,
        &mut Enable as *mut u32 as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as ULONG,
        core::ptr::null_mut(),
        0,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        NtClose(StdOutput);
        return (FALSE) as BOOL;
    }

    SetConsoleOutputCP(437);

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut KeyboardName,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenFile(
        &mut StdInput,
        FILE_ALL_ACCESS,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_OPEN,
        0,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        NtClose(StdOutput);
        return (FALSE) as BOOL;
    }

    InputQueueEmpty = (TRUE) as u8;
    WaitForInput = (FALSE) as u8;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AttachConsole(dwProcessId: DWORD) -> BOOL {
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn FreeConsole() -> BOOL {
    InputQueueEmpty = (TRUE) as u8;
    WaitForInput = (FALSE) as u8;

    if StdInput != INVALID_HANDLE_VALUE {
        NtClose(StdInput);
    }

    if StdOutput != INVALID_HANDLE_VALUE {
        NtClose(StdOutput);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WriteConsole(
    hConsoleOutput: HANDLE,
    lpBuffer: *const core::ffi::c_void,
    nNumberOfCharsToWrite: DWORD,
    lpNumberOfCharsWritten: LPDWORD,
    lpReserved: LPVOID,
) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: NTSTATUS = 0;
    Status = NtWriteFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        lpBuffer as LPVOID,
        nNumberOfCharsToWrite,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }
    *lpNumberOfCharsWritten = IoStatusBlock.Information as DWORD;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetStdHandle(nStdHandle: DWORD) -> HANDLE {
    match nStdHandle {
        STD_INPUT_HANDLE => return StdInput,
        STD_OUTPUT_HANDLE => return StdOutput,
        _ => return INVALID_HANDLE_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "system" fn FlushConsoleInputBuffer(hConsoleInput: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut InputData: KEYBOARD_INPUT_DATA = core::mem::zeroed();

    if WaitForInput != 0 {
        NtCancelIoFile(hConsoleInput, &mut IoStatusBlock);
    }

    InputQueueEmpty = (TRUE) as u8;
    WaitForInput = (FALSE) as u8;

    loop {
        Offset.QuadPart = 0;
        Status = NtReadFile(
            hConsoleInput,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            &mut InputData as *mut KEYBOARD_INPUT_DATA as LPVOID,
            core::mem::size_of::<KEYBOARD_INPUT_DATA>() as u32,
            &mut Offset,
            core::ptr::null_mut(),
        );
        if Status == STATUS_PENDING {
            Timeout.QuadPart = -100;
            Status = NtWaitForSingleObject(hConsoleInput, (FALSE) as BOOLEAN, &mut Timeout);
            if Status == STATUS_TIMEOUT {
                NtCancelIoFile(hConsoleInput, &mut IoStatusBlock);
                return (TRUE) as BOOL;
            }
        }
        if ((NT_SUCCESS!(Status)) == 0) {
            break;
        }
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PeekConsoleInput(
    hConsoleInput: HANDLE,
    lpBuffer: PINPUT_RECORD,
    nLength: DWORD,
    lpNumberOfEventsRead: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut InputData: KEYBOARD_INPUT_DATA;

    if InputQueueEmpty != 0 {
        if WaitForInput == 0 {
            Offset.QuadPart = 0;
            Status = NtReadFile(
                hConsoleInput,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut InputIosb,
                &mut InputDataQueue as *mut _ as LPVOID,
                core::mem::size_of::<KEYBOARD_INPUT_DATA>() as u32,
                &mut Offset,
                core::ptr::null_mut(),
            );
            if ((NT_SUCCESS!(Status)) == 0) {
                return (FALSE) as BOOL;
            }
            if Status == STATUS_PENDING {
                *lpNumberOfEventsRead = 0;
                WaitForInput = (TRUE) as u8;
                return (TRUE) as BOOL;
            }
        } else {
            Timeout.QuadPart = -100;
            Status = NtWaitForSingleObject(hConsoleInput, (FALSE) as BOOLEAN, &mut Timeout);
            if Status == STATUS_TIMEOUT {
                *lpNumberOfEventsRead = 0;
                WaitForInput = (TRUE) as u8;
                return (TRUE) as BOOL;
            }
            WaitForInput = (FALSE) as u8;
            if ((NT_SUCCESS!(Status)) == 0) {
                return (FALSE) as BOOL;
            }
        }

        InputQueueEmpty = (FALSE) as u8;
        WaitForInput = (FALSE) as u8;
    }

    InputData = InputDataQueue;

    (*lpBuffer).EventType = (KEY_EVENT) as u16;
    Status = IntTranslateKey(hConsoleInput, &mut InputData, &mut (*lpBuffer).Event.KeyEvent);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    *lpNumberOfEventsRead = 1;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ReadConsoleInput(
    hConsoleInput: HANDLE,
    lpBuffer: PINPUT_RECORD,
    nLength: DWORD,
    lpNumberOfEventsRead: LPDWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut InputData: KEYBOARD_INPUT_DATA;

    if InputQueueEmpty != 0 {
        if WaitForInput == 0 {
            Offset.QuadPart = 0;
            Status = NtReadFile(
                hConsoleInput,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut InputIosb,
                &mut InputDataQueue as *mut _ as LPVOID,
                core::mem::size_of::<KEYBOARD_INPUT_DATA>() as u32,
                &mut Offset,
                core::ptr::null_mut(),
            );
            if Status == STATUS_PENDING {
                WaitForInput = (TRUE) as u8;
                Status = NtWaitForSingleObject(
                    hConsoleInput,
                    (FALSE) as BOOLEAN,
                    core::ptr::null_mut(),
                );
                WaitForInput = (FALSE) as u8;
                Status = InputIosb.__bindgen_anon_1.Status;
            }
            if ((NT_SUCCESS!(Status)) == 0) {
                return (FALSE) as BOOL;
            }
        } else {
            Status = NtWaitForSingleObject(
                hConsoleInput,
                (FALSE) as BOOLEAN,
                core::ptr::null_mut(),
            );
            WaitForInput = (FALSE) as u8;
            Status = InputIosb.__bindgen_anon_1.Status;
            if ((NT_SUCCESS!(Status)) == 0) {
                return (FALSE) as BOOL;
            }
        }
    }

    InputData = InputDataQueue;
    InputQueueEmpty = (TRUE) as u8;

    (*lpBuffer).EventType = (KEY_EVENT) as u16;
    Status = IntTranslateKey(
        hConsoleInput,
        &mut InputData,
        &mut (*lpBuffer).Event.KeyEvent,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    *lpNumberOfEventsRead = 1;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WriteConsoleOutputCharacterA(hConsoleOutput: HANDLE, lpCharacter: LPCSTR, nLength: DWORD, dwWriteCoord: COORD, lpNumberOfCharsWritten: LPDWORD) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Buffer: PCHAR = core::ptr::null_mut();
    let mut pCoord: *mut COORD = core::ptr::null_mut();
    let mut pText: PCHAR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    Buffer = RtlAllocateHeap(
        ProcessHeap,
        0,
        (nLength.wrapping_add(core::mem::size_of::<COORD>() as DWORD)) as SIZE_T,
    ) as PCHAR;
    pCoord = Buffer as *mut COORD;
    pText = pCoord.add(1) as PCHAR;

    *pCoord = dwWriteCoord;
    memcpy(
        pText as *mut core::ffi::c_void,
        lpCharacter as *const core::ffi::c_void,
        nLength as usize,
    );

    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER) as ULONG,
        core::ptr::null_mut(),
        0,
        Buffer as LPVOID,
        nLength.wrapping_add(core::mem::size_of::<COORD>() as DWORD),
    );

    RtlFreeHeap(ProcessHeap, 0, Buffer as LPVOID);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    *lpNumberOfCharsWritten = IoStatusBlock.Information as DWORD;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WriteConsoleOutputCharacterW(hConsoleOutput: HANDLE, lpCharacter: LPCWSTR, nLength: DWORD, dwWriteCoord: COORD, lpNumberOfCharsWritten: LPDWORD) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Buffer: PCHAR = core::ptr::null_mut();
    let mut pCoord: *mut COORD = core::ptr::null_mut();
    let mut pText: PCHAR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut UnicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut OemString: OEM_STRING = OEM_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut OemLength: ULONG = 0;

    UnicodeString.Length = (nLength as usize).wrapping_mul(core::mem::size_of::<u16>()) as u16;
    UnicodeString.MaximumLength = (nLength as usize).wrapping_mul(core::mem::size_of::<u16>()) as u16;
    UnicodeString.Buffer = lpCharacter as PWSTR;

    OemLength = RtlUnicodeStringToOemSize(&mut UnicodeString);

    Buffer = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((OemLength as usize).wrapping_add(core::mem::size_of::<COORD>())) as SIZE_T,
    ) as PCHAR;

    if Buffer.is_null() {
        return (FALSE) as BOOL;
    }

    pCoord = Buffer as *mut COORD;
    pText = pCoord.add(1) as PCHAR;

    *pCoord = dwWriteCoord;

    OemString.Length = 0;
    OemString.MaximumLength = OemLength as u16;
    OemString.Buffer = pText;

    Status = RtlUnicodeStringToOemString(
        &mut OemString,
        &mut UnicodeString,
        (FALSE) as BOOLEAN,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        // goto done
    } else {
        Status = NtDeviceIoControlFile(
            hConsoleOutput,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            (IOCTL_CONSOLE_WRITE_OUTPUT_CHARACTER) as ULONG,
            core::ptr::null_mut(),
            0,
            Buffer as LPVOID,
            (nLength as usize).wrapping_add(core::mem::size_of::<COORD>()) as u32,
        );
    }

    // done:
    RtlFreeHeap(ProcessHeap, 0, Buffer as LPVOID);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    *lpNumberOfCharsWritten = IoStatusBlock.Information as DWORD;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FillConsoleOutputAttribute(hConsoleOutput: HANDLE, wAttribute: WORD, nLength: DWORD, dwWriteCoord: COORD, lpNumberOfAttrsWritten: LPDWORD) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Buffer: OUTPUT_ATTRIBUTE = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Buffer.wAttribute = wAttribute;
    Buffer.nLength = nLength;
    Buffer.dwCoord = dwWriteCoord;

    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_FILL_OUTPUT_ATTRIBUTE) as ULONG,
        &mut Buffer as *mut OUTPUT_ATTRIBUTE as LPVOID,
        core::mem::size_of::<OUTPUT_ATTRIBUTE>() as u32,
        &mut Buffer as *mut OUTPUT_ATTRIBUTE as LPVOID,
        core::mem::size_of::<OUTPUT_ATTRIBUTE>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    *lpNumberOfAttrsWritten = Buffer.dwTransfered;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FillConsoleOutputCharacterA(hConsoleOutput: HANDLE, cCharacter: CHAR, nLength: DWORD, dwWriteCoord: COORD, lpNumberOfCharsWritten: LPDWORD) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Buffer: OUTPUT_CHARACTER = OUTPUT_CHARACTER {
        cCharacter: 0,
        nLength: 0,
        dwCoord: COORD { X: 0, Y: 0 },
        dwTransfered: 0,
    };
    let mut Status: NTSTATUS = 0;
    Buffer.cCharacter = cCharacter;
    Buffer.nLength = nLength;
    Buffer.dwCoord = dwWriteCoord;
    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_FILL_OUTPUT_CHARACTER) as ULONG,
        &mut Buffer as *mut OUTPUT_CHARACTER as LPVOID,
        core::mem::size_of::<OUTPUT_CHARACTER>() as u32,
        &mut Buffer as *mut OUTPUT_CHARACTER as LPVOID,
        core::mem::size_of::<OUTPUT_CHARACTER>() as u32,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }
    *lpNumberOfCharsWritten = Buffer.dwTransfered;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetConsoleScreenBufferInfo(hConsoleOutput: HANDLE, lpConsoleScreenBufferInfo: PCONSOLE_SCREEN_BUFFER_INFO) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_GET_SCREEN_BUFFER_INFO) as ULONG,
        core::ptr::null_mut(),
        0,
        lpConsoleScreenBufferInfo as *mut core::ffi::c_void,
        core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>() as ULONG,
    );
    return NT_SUCCESS!(Status);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCursorInfo(hConsoleOutput: HANDLE, lpConsoleCursorInfo: *const CONSOLE_CURSOR_INFO) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: NTSTATUS = 0;
    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_SET_CURSOR_INFO) as ULONG,
        lpConsoleCursorInfo as *const CONSOLE_CURSOR_INFO as *mut core::ffi::c_void,
        core::mem::size_of::<CONSOLE_CURSOR_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );
    return NT_SUCCESS!(Status);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleCursorPosition(hConsoleOutput: HANDLE, dwCursorPosition: COORD) -> BOOL {
    let mut ConsoleScreenBufferInfo: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = GetConsoleScreenBufferInfo(hConsoleOutput, &mut ConsoleScreenBufferInfo);
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    ConsoleScreenBufferInfo.dwCursorPosition.X = dwCursorPosition.X;
    ConsoleScreenBufferInfo.dwCursorPosition.Y = dwCursorPosition.Y;

    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_SET_SCREEN_BUFFER_INFO) as ULONG,
        &mut ConsoleScreenBufferInfo as *mut _ as LPVOID,
        core::mem::size_of::<CONSOLE_SCREEN_BUFFER_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );
    return NT_SUCCESS!(Status) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleTextAttribute(
    hConsoleOutput: HANDLE,
    mut wAttributes: WORD,
) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = { let mut __lit: _IO_STATUS_BLOCK = core::mem::zeroed(); __lit.__bindgen_anon_1.Status = 0; __lit.Information = 0; __lit };
    let mut Status: NTSTATUS = 0;
    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_SET_TEXT_ATTRIBUTE) as ULONG,
        &mut wAttributes as *mut WORD as LPVOID,
        core::mem::size_of::<USHORT>() as u32,
        core::ptr::null_mut(),
        0,
    );
    return NT_SUCCESS!(Status);
}

#[no_mangle]
pub unsafe extern "system" fn SetConsoleOutputCP(wCodepage: UINT) -> BOOL {
    static mut FontFile: *const u16 = RUSTCONV_WSTR_9618145ECC2C.as_ptr();
    let mut FontName: [u16; 20] = [0; 20];
    let mut ConsoleCabinetContext: CONSOLE_CABINET_CONTEXT = core::mem::zeroed();
    let CabinetContext: *mut CABINET_CONTEXT = &mut ConsoleCabinetContext.CabinetContext;
    let mut Search: CAB_SEARCH = core::mem::zeroed();
    let mut CabStatus: u32;
    let mut hConsoleOutput: *mut core::ffi::c_void;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: i32;

    if wCodepage == LastLoadedCodepage {
        return (TRUE) as BOOL;
    }

    hConsoleOutput = GetStdHandle(STD_OUTPUT_HANDLE);

    CabinetInitialize(CabinetContext);
    CabinetSetEventHandlers(
        CabinetContext,
        None,
        None,
        None,
        Some(ConsoleCreateFileHandler),
    );
    CabinetSetCabinetName(CabinetContext, FontFile);

    CabStatus = CabinetOpen(CabinetContext);
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT!(
            b"CabinetOpen('%S') returned 0x%08x\n\0".as_ptr() as *const i8,
            FontFile,
            CabStatus,
        );
        return (FALSE) as BOOL;
    }

    RtlStringCbPrintfW!(
        FontName.as_mut_ptr(),
        core::mem::size_of::<[u16; 20]>(),
        RUSTCONV_WSTR_7A95474577CC.as_ptr(),
        wCodepage,
    );
    CabStatus = CabinetFindFirst(CabinetContext, FontName.as_ptr(), &mut Search);
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT!(
            b"CabinetFindFirst('%S', '%S') returned 0x%08x\n\0".as_ptr() as *const i8,
            FontFile,
            FontName.as_ptr(),
            CabStatus,
        );
        CabinetClose(CabinetContext);
        return (FALSE) as BOOL;
    }

    CabStatus = CabinetExtractFile(CabinetContext, &mut Search);
    CabinetClose(CabinetContext);
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT!(
            b"CabinetExtractFile('%S', '%S') returned 0x%08x\n\0".as_ptr() as *const i8,
            FontFile,
            FontName.as_ptr(),
            CabStatus,
        );
        if !ConsoleCabinetContext.Data.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                ConsoleCabinetContext.Data,
            );
        }
        return (FALSE) as BOOL;
    }

    Status = NtDeviceIoControlFile(
        hConsoleOutput,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        (IOCTL_CONSOLE_LOADFONT) as ULONG,
        ConsoleCabinetContext.Data,
        ConsoleCabinetContext.Size,
        core::ptr::null_mut(),
        0,
    );

    RtlFreeHeap(
        ProcessHeap,
        0,
        ConsoleCabinetContext.Data,
    );

    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as BOOL;
    }

    LastLoadedCodepage = wCodepage;
    return (TRUE) as BOOL;
}
