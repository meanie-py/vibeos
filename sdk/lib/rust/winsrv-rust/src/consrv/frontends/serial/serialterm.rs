use winsrv_sys::nt::*;
use super::super::super::{VkKeyScanW};
use super::super::super::coninput::ConioProcessInputEvent;
use crate::consrv::{ConsoleAllocHeap, ConsoleFreeHeap};
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Disambiguate winsrv_sys::winsrv:: (bindgen) vs sibling Rust impl.
use super::super::super::condrv::console::ConDrvValidateConsoleUnsafe;


// Win32 API not bindgen-exposed by winsrv-sys (kernel32 outside FN_ALLOW).
extern "system" {
    pub fn DeviceIoControl(
        hDevice: HANDLE,
        dwIoControlCode: u32,
        lpInBuffer: *const core::ffi::c_void,
        nInBufferSize: u32,
        lpOutBuffer: *mut core::ffi::c_void,
        nOutBufferSize: u32,
        lpBytesReturned: *mut u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> BOOL;
    pub fn Sleep(dwMilliseconds: u32);
}

const IOCTL_ARM64INPUT_INJECT_BYTE: u32 = (0x0000000B << 16) | (0x800 << 2) | (0 << 14) | (0) ;
const IOCTL_ARM64INPUT_INJECT_MOUSE: u32 = (0x0000000F << 16) | (0x800 << 2) | (0 << 14) | (0) ;

const MOUSE_LEFT_BUTTON_DOWN: u32 = 0x0001;
const MOUSE_LEFT_BUTTON_UP: u32 = 0x0002;
const MOUSE_RIGHT_BUTTON_DOWN: u32 = 0x0004;
const MOUSE_RIGHT_BUTTON_UP: u32 = 0x0008;

#[repr(C)]
struct SERIAL_CONSOLE_DATA {
    hSerialDevice: HANDLE,
    hKeyboardDevice: HANDLE,
    hMouseDevice: HANDLE,
    hInputThread: HANDLE,
    Terminate: u8,
    Console: PCONSRV_CONSOLE,
    ActiveBuffer: PCONSOLE_SCREEN_BUFFER,
}

type PSERIAL_CONSOLE_DATA = *mut SERIAL_CONSOLE_DATA;

#[repr(C)]
struct MOUSE_INPUT_DATA {
    UnitId: u16,
    Flags: u16,
    __bindgen_anon_1: MOUSE_INPUT_DATA_UNION,
    RawButtons: u32,
    LastX: i32,
    LastY: i32,
    ExtraInformation: u32,
}

#[repr(C)]
union MOUSE_INPUT_DATA_UNION {
    Buttons: u32,
    ButtonFlags: u32,
    ButtonData: u32,
}

pub(crate) unsafe fn SerialProcessKeyByte(SerialData: PSERIAL_CONSOLE_DATA, ch: u8) {
    let Console = (*SerialData).Console;
    let mut er: INPUT_RECORD = core::mem::zeroed();

    core::ptr::write_bytes(&mut er as *mut INPUT_RECORD as *mut u8, 0, core::mem::size_of::<INPUT_RECORD>());
    er.EventType = KEY_EVENT as u16;
    er.Event.KeyEvent.bKeyDown = 1;
    er.Event.KeyEvent.wRepeatCount = 1;
    er.Event.KeyEvent.dwControlKeyState = 0;

    if ch == b'\r' {
        er.Event.KeyEvent.wVirtualKeyCode = VK_RETURN as u16;
        er.Event.KeyEvent.wVirtualScanCode = 0x1C;
        er.Event.KeyEvent.uChar.UnicodeChar = '\r' as u16;
    } else if ch == 0x08 || ch == 0x7F {
        er.Event.KeyEvent.wVirtualKeyCode = VK_BACK as u16;
        er.Event.KeyEvent.wVirtualScanCode = 0x0E;
        er.Event.KeyEvent.uChar.UnicodeChar = '\u{0008}' as u16;
    } else if ch == 0x03 {
        er.Event.KeyEvent.wVirtualKeyCode = 'C' as u16;
        er.Event.KeyEvent.wVirtualScanCode = 0x2E;
        er.Event.KeyEvent.uChar.UnicodeChar = 0x03;
        er.Event.KeyEvent.dwControlKeyState = LEFT_CTRL_PRESSED;
    } else if ch == b'\t' {
        er.Event.KeyEvent.wVirtualKeyCode = VK_TAB as u16;
        er.Event.KeyEvent.wVirtualScanCode = 0x0F;
        er.Event.KeyEvent.uChar.UnicodeChar = '\t' as u16;
    } else {
        let mut wch: u16 = 0;
        MultiByteToWideChar(
            (*Console).InputCodePage,
            0,
            &ch as *const u8 as LPCSTR,
            1,
            &mut wch as *mut u16,
            1,
        );
        er.Event.KeyEvent.uChar.UnicodeChar = wch;
        er.Event.KeyEvent.wVirtualKeyCode = (VkKeyScanW(wch) & 0xFF) as u16;
        er.Event.KeyEvent.wVirtualScanCode = 0;
    }

    if ConDrvValidateConsoleUnsafe(
        Console as PCONSOLE,
        CONSOLE_RUNNING,
        1,
    ) != 0
    {
        ConioProcessInputEvent(Console, &mut er);
        er.Event.KeyEvent.bKeyDown = 0;
        ConioProcessInputEvent(Console, &mut er);
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
    }

    if !(*SerialData).hKeyboardDevice.is_null()
        && (*SerialData).hKeyboardDevice != INVALID_HANDLE_VALUE
    {
        let mut Ignored: u32 = 0;
        DeviceIoControl(
            (*SerialData).hKeyboardDevice,
            IOCTL_ARM64INPUT_INJECT_BYTE,
            &ch as *const u8 as *mut core::ffi::c_void,
            1,
            core::ptr::null_mut(),
            0,
            &mut Ignored,
            core::ptr::null_mut(),
        );
    }
}

pub(crate) unsafe fn SerialProcessMouseEscape(SerialData: PSERIAL_CONSOLE_DATA, buf: *const u8) {
    let action = *buf.add(0);
    let x = (*buf.add(1) as u16) | ((*buf.add(2) as u16) << 8);
    let y = (*buf.add(3) as u16) | ((*buf.add(4) as u16) << 8);

    if (*SerialData).hMouseDevice.is_null()
        || (*SerialData).hMouseDevice == INVALID_HANDLE_VALUE
    {
        return;
    }

    let mut mid: MOUSE_INPUT_DATA = core::mem::zeroed();
    core::ptr::write_bytes(&mut mid as *mut MOUSE_INPUT_DATA as *mut u8, 0, core::mem::size_of::<MOUSE_INPUT_DATA>());
    mid.Flags = MOUSE_MOVE_ABSOLUTE as u16;
    mid.LastX = x as i32;
    mid.LastY = y as i32;

    mid.__bindgen_anon_1.ButtonFlags = match action {
        0 => 0,
        1 => MOUSE_LEFT_BUTTON_DOWN,
        2 => MOUSE_LEFT_BUTTON_UP,
        3 => MOUSE_RIGHT_BUTTON_DOWN,
        4 => MOUSE_RIGHT_BUTTON_UP,
        _ => return,
    };

    let mut Ignored: u32 = 0;
    DeviceIoControl(
        (*SerialData).hMouseDevice,
        IOCTL_ARM64INPUT_INJECT_MOUSE,
        &mid as *const MOUSE_INPUT_DATA as *mut core::ffi::c_void,
        core::mem::size_of::<MOUSE_INPUT_DATA>() as u32,
        core::ptr::null_mut(),
        0,
        &mut Ignored,
        core::ptr::null_mut(),
    );
}

unsafe extern "system" fn SerialInputThread(Param: *mut core::ffi::c_void) -> u32 {
    let SerialData = Param as PSERIAL_CONSOLE_DATA;
    let mut ch: u8 = 0;
    let mut BytesRead: u32 = 0;
    let mut esc_state: i32 = 0;
    let mut esc_buf: [u8; 5] = [0; 5];
    let mut esc_pos: i32 = 0;

    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);

    while (*SerialData).Terminate == 0 {
        if ReadFile(
            (*SerialData).hSerialDevice,
            &mut ch as *mut u8 as *mut core::ffi::c_void,
            1,
            &mut BytesRead,
            core::ptr::null_mut(),
        ) == 0
            || BytesRead == 0
        {
            Sleep(50);
            continue;
        }

        match esc_state {
            0 => {
                if ch == 0x1B {
                    esc_state = 1;
                    continue;
                }
                SerialProcessKeyByte(SerialData, ch);
                continue;
            }
            1 => {
                if ch == b'[' {
                    esc_state = 2;
                    continue;
                }
                SerialProcessKeyByte(SerialData, 0x1B);
                esc_state = 0;
                SerialProcessKeyByte(SerialData, ch);
                continue;
            }
            2 => {
                if ch == b'M' {
                    esc_state = 3;
                    esc_pos = 0;
                    continue;
                }
                SerialProcessKeyByte(SerialData, 0x1B);
                SerialProcessKeyByte(SerialData, b'[');
                esc_state = 0;
                SerialProcessKeyByte(SerialData, ch);
                continue;
            }
            3 => {
                esc_buf[esc_pos as usize] = ch;
                esc_pos += 1;
                if esc_pos == 5 {
                    SerialProcessMouseEscape(SerialData, esc_buf.as_ptr());
                    esc_state = 0;
                }
                continue;
            }
            _ => {}
        }
    }

    0
}

unsafe extern "system" fn SerialInitFrontEnd(
    This: *mut FRONTEND,
    Console: PCONSRV_CONSOLE,
) -> i32 {
    if This.is_null() || Console.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let SerialData = ConsoleAllocHeap(
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SERIAL_CONSOLE_DATA>(),
    ) as PSERIAL_CONSOLE_DATA;
    if SerialData.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*SerialData).Console = Console;
    (*SerialData).ActiveBuffer = (*Console).ActiveBuffer;
    (*SerialData).Terminate = 0;

    static COM1_PATH: [u16; 12] = [
        b'\\' as u16, b'\\' as u16, b'.' as u16, b'\\' as u16,
        b'C' as u16, b'O' as u16, b'M' as u16, b'1' as u16,
        0, 0, 0, 0,
    ];

    let hSerial = CreateFileW(
        COM1_PATH.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hSerial == INVALID_HANDLE_VALUE {
        ConsoleFreeHeap(SerialData as *mut core::ffi::c_void);
        return STATUS_UNSUCCESSFUL;
    }
    (*SerialData).hSerialDevice = hSerial;

    static KEYBOARD_PATH: [u16; 22] = [
        b'\\' as u16, b'\\' as u16, b'.' as u16, b'\\' as u16,
        b'K' as u16, b'e' as u16, b'y' as u16, b'b' as u16,
        b'o' as u16, b'a' as u16, b'r' as u16, b'd' as u16,
        b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16,
        b's' as u16, b'0' as u16, 0, 0, 0, 0,
    ];

    (*SerialData).hKeyboardDevice = CreateFileW(
        KEYBOARD_PATH.as_ptr(),
        GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    static POINTER_PATH: [u16; 21] = [
        b'\\' as u16, b'\\' as u16, b'.' as u16, b'\\' as u16,
        b'P' as u16, b'o' as u16, b'i' as u16, b'n' as u16,
        b't' as u16, b'e' as u16, b'r' as u16, b'C' as u16,
        b'l' as u16, b'a' as u16, b's' as u16, b's' as u16,
        b'0' as u16, 0, 0, 0, 0,
    ];

    (*SerialData).hMouseDevice = CreateFileW(
        POINTER_PATH.as_ptr(),
        GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    (*This).Context = SerialData as *mut core::ffi::c_void;
    (*This).Context2 = core::ptr::null_mut();

    (*SerialData).hInputThread = core::ptr::null_mut();

    STATUS_SUCCESS
}

unsafe extern "system" fn SerialDeinitFrontEnd(This: *mut FRONTEND) {
    if This.is_null() || (*This).Context.is_null() {
        return;
    }

    let SerialData = (*This).Context as PSERIAL_CONSOLE_DATA;

    (*SerialData).Terminate = 1;

    if !(*SerialData).hSerialDevice.is_null() && (*SerialData).hSerialDevice != INVALID_HANDLE_VALUE {
        CloseHandle((*SerialData).hSerialDevice);
    }

    if !(*SerialData).hKeyboardDevice.is_null() && (*SerialData).hKeyboardDevice != INVALID_HANDLE_VALUE {
        CloseHandle((*SerialData).hKeyboardDevice);
    }

    if !(*SerialData).hMouseDevice.is_null() && (*SerialData).hMouseDevice != INVALID_HANDLE_VALUE {
        CloseHandle((*SerialData).hMouseDevice);
    }

    if !(*SerialData).hInputThread.is_null() {
        WaitForSingleObject((*SerialData).hInputThread, 2000);
        CloseHandle((*SerialData).hInputThread);
    }

    ConsoleFreeHeap(SerialData as *mut core::ffi::c_void);
    (*This).Context = core::ptr::null_mut();
}

unsafe extern "system" fn SerialDrawRegion(This: *mut FRONTEND, Region: *mut SMALL_RECT) {
    let _ = This;
    let _ = Region;
}

unsafe extern "system" fn SerialWriteStream(
    This: *mut FRONTEND,
    Region: *mut SMALL_RECT,
    CursorStartX: i16,
    CursorStartY: i16,
    ScrolledLines: u32,
    Buffer: *mut u16,
    Length: u32,
) {
    let _ = Region;
    let _ = CursorStartX;
    let _ = CursorStartY;
    let _ = ScrolledLines;

    if This.is_null() || (*This).Context.is_null() || Buffer.is_null() || Length == 0 {
        return;
    }

    let SerialData = (*This).Context as PSERIAL_CONSOLE_DATA;

    let AnsiLength = WideCharToMultiByte(
        (*(*SerialData).Console).OutputCodePage,
        0,
        Buffer,
        Length as i32,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    if AnsiLength == 0 {
        return;
    }

    let AnsiBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        AnsiLength as SIZE_T,
    ) as *mut i8;
    if AnsiBuffer.is_null() {
        return;
    }

    WideCharToMultiByte(
        (*(*SerialData).Console).OutputCodePage,
        0,
        Buffer,
        Length as i32,
        AnsiBuffer,
        AnsiLength,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    let mut BytesWritten: u32 = 0;
    WriteFile(
        (*SerialData).hSerialDevice,
        AnsiBuffer as *const core::ffi::c_void,
        AnsiLength as u32,
        &mut BytesWritten,
        core::ptr::null_mut(),
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiBuffer as *mut core::ffi::c_void);
}

unsafe extern "system" fn SerialRingBell(This: *mut FRONTEND) {
    let _ = This;
}

unsafe extern "system" fn SerialSetCursorInfo(
    This: *mut FRONTEND,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) -> i32 {
    let _ = This;
    let _ = ScreenBuffer;
    1
}

unsafe extern "system" fn SerialSetScreenInfo(
    This: *mut FRONTEND,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
    OldCursorX: i16,
    OldCursorY: i16,
) -> i32 {
    let _ = This;
    let _ = ScreenBuffer;
    let _ = OldCursorX;
    let _ = OldCursorY;
    1
}

unsafe extern "system" fn SerialResizeTerminal(This: *mut FRONTEND) {
    let _ = This;
}

unsafe extern "system" fn SerialSetActiveScreenBuffer(This: *mut FRONTEND) {
    let _ = This;
}

unsafe extern "system" fn SerialReleaseScreenBuffer(
    This: *mut FRONTEND,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
) {
    let _ = This;
    let _ = ScreenBuffer;
}

unsafe extern "system" fn SerialRefreshInternalInfo(This: *mut FRONTEND) {
    let _ = This;
}

unsafe extern "system" fn SerialChangeTitle(This: *mut FRONTEND) {
    let _ = This;
}

unsafe extern "system" fn SerialChangeIcon(This: *mut FRONTEND, IconHandle: HICON) -> i32 {
    let _ = This;
    let _ = IconHandle;
    1
}

unsafe extern "system" fn SerialGetThreadConsoleDesktop(This: *mut FRONTEND) -> HDESK {
    let _ = This;
    core::ptr::null_mut()
}

unsafe extern "system" fn SerialGetConsoleWindowHandle(This: *mut FRONTEND) -> HWND {
    let _ = This;
    core::ptr::null_mut()
}

unsafe extern "system" fn SerialGetLargestConsoleWindowSize(This: *mut FRONTEND, pSize: PCOORD) {
    let _ = This;
    if !pSize.is_null() {
        (*pSize).X = 80;
        (*pSize).Y = 25;
    }
}

unsafe extern "system" fn SerialGetSelectionInfo(
    This: *mut FRONTEND,
    pSelectionInfo: PCONSOLE_SELECTION_INFO,
) -> i32 {
    let _ = This;
    if !pSelectionInfo.is_null() {
        core::ptr::write_bytes(pSelectionInfo as *mut u8, 0, core::mem::size_of::<CONSOLE_SELECTION_INFO>());
    }
    1
}

unsafe extern "system" fn SerialSetPalette(
    This: *mut FRONTEND,
    PaletteHandle: HPALETTE,
    PaletteUsage: u32,
) -> i32 {
    let _ = This;
    let _ = PaletteHandle;
    let _ = PaletteUsage;
    1
}

unsafe extern "system" fn SerialSetCodePage(This: *mut FRONTEND, CodePage: u32) -> i32 {
    let _ = This;
    let _ = CodePage;
    1
}

unsafe extern "system" fn SerialGetDisplayMode(This: *mut FRONTEND) -> u32 {
    let _ = This;
    CONSOLE_FULLSCREEN_HARDWARE
}

unsafe extern "system" fn SerialSetDisplayMode(This: *mut FRONTEND, NewMode: u32) -> i32 {
    let _ = This;
    let _ = NewMode;
    1
}

unsafe extern "system" fn SerialShowMouseCursor(This: *mut FRONTEND, Show: i32) -> i32 {
    let _ = This;
    let _ = Show;
    0
}

unsafe extern "system" fn SerialSetMouseCursor(This: *mut FRONTEND, CursorHandle: HCURSOR) -> i32 {
    let _ = This;
    let _ = CursorHandle;
    1
}

unsafe extern "system" fn SerialMenuControl(
    This: *mut FRONTEND,
    CmdIdLow: u32,
    CmdIdHigh: u32,
) -> HMENU {
    let _ = This;
    let _ = CmdIdLow;
    let _ = CmdIdHigh;
    core::ptr::null_mut()
}

unsafe extern "system" fn SerialSetMenuClose(This: *mut FRONTEND, Enable: i32) -> i32 {
    let _ = This;
    let _ = Enable;
    1
}

/// A serial terminal has no font of ours; report the 8x16 text cell that its
/// 80x25 window size assumes, so a caller dividing by it gets a sane answer.
unsafe extern "system" fn SerialGetFontSize(This: *mut FRONTEND, pSize: PCOORD) {
    let _ = This;
    if !pSize.is_null() {
        (*pSize).X = 8;
        (*pSize).Y = 16;
    }
}

// `FRONTEND_VTBL` field types use `extern "stdcall"` on i386 and
// `extern "C"` on amd64/arm64 (per bindgen translating __stdcall).
// Our Serial* impls are `extern "system"` (same ABI per arch, distinct
// Rust type). Lazy-init via `*const ()` runtime-transmute pattern.
static mut SerialVtbl: FRONTEND_VTBL = FRONTEND_VTBL {
    InitFrontEnd: None,
    DeinitFrontEnd: None,
    DrawRegion: None,
    WriteStream: None,
    RingBell: None,
    SetCursorInfo: None,
    SetScreenInfo: None,
    ResizeTerminal: None,
    SetActiveScreenBuffer: None,
    ReleaseScreenBuffer: None,
    RefreshInternalInfo: None,
    ChangeTitle: None,
    ChangeIcon: None,
    GetThreadConsoleDesktop: None,
    GetConsoleWindowHandle: None,
    GetLargestConsoleWindowSize: None,
    GetSelectionInfo: None,
    SetPalette: None,
    SetCodePage: None,
    GetDisplayMode: None,
    SetDisplayMode: None,
    ShowMouseCursor: None,
    SetMouseCursor: None,
    MenuControl: None,
    SetMenuClose: None,
    GetFontSize: None,
};

pub(crate) unsafe fn InitSerialVtblIfNeeded() {
    if SerialVtbl.InitFrontEnd.is_some() {
        return;
    }
    SerialVtbl = FRONTEND_VTBL {
        InitFrontEnd: Some(core::mem::transmute(SerialInitFrontEnd as *const ())),
        DeinitFrontEnd: Some(core::mem::transmute(SerialDeinitFrontEnd as *const ())),
        DrawRegion: Some(core::mem::transmute(SerialDrawRegion as *const ())),
        WriteStream: Some(core::mem::transmute(SerialWriteStream as *const ())),
        RingBell: Some(core::mem::transmute(SerialRingBell as *const ())),
        SetCursorInfo: Some(core::mem::transmute(SerialSetCursorInfo as *const ())),
        SetScreenInfo: Some(core::mem::transmute(SerialSetScreenInfo as *const ())),
        ResizeTerminal: Some(core::mem::transmute(SerialResizeTerminal as *const ())),
        SetActiveScreenBuffer: Some(core::mem::transmute(SerialSetActiveScreenBuffer as *const ())),
        ReleaseScreenBuffer: Some(core::mem::transmute(SerialReleaseScreenBuffer as *const ())),
        RefreshInternalInfo: Some(core::mem::transmute(SerialRefreshInternalInfo as *const ())),
        ChangeTitle: Some(core::mem::transmute(SerialChangeTitle as *const ())),
        ChangeIcon: Some(core::mem::transmute(SerialChangeIcon as *const ())),
        GetThreadConsoleDesktop: Some(core::mem::transmute(SerialGetThreadConsoleDesktop as *const ())),
        GetConsoleWindowHandle: Some(core::mem::transmute(SerialGetConsoleWindowHandle as *const ())),
        GetLargestConsoleWindowSize: Some(core::mem::transmute(SerialGetLargestConsoleWindowSize as *const ())),
        GetSelectionInfo: Some(core::mem::transmute(SerialGetSelectionInfo as *const ())),
        SetPalette: Some(core::mem::transmute(SerialSetPalette as *const ())),
        SetCodePage: Some(core::mem::transmute(SerialSetCodePage as *const ())),
        GetDisplayMode: Some(core::mem::transmute(SerialGetDisplayMode as *const ())),
        SetDisplayMode: Some(core::mem::transmute(SerialSetDisplayMode as *const ())),
        ShowMouseCursor: Some(core::mem::transmute(SerialShowMouseCursor as *const ())),
        SetMouseCursor: Some(core::mem::transmute(SerialSetMouseCursor as *const ())),
        MenuControl: Some(core::mem::transmute(SerialMenuControl as *const ())),
        SetMenuClose: Some(core::mem::transmute(SerialSetMenuClose as *const ())),
        GetFontSize: Some(core::mem::transmute(SerialGetFontSize as *const ())),
    };
}

#[no_mangle]
pub unsafe extern "system" fn SerialLoadFrontEnd(
    FrontEnd: *mut FRONTEND,
    ConsoleInfo: *mut CONSOLE_STATE_INFO,
    ConsoleInitInfo: *mut CONSOLE_INIT_INFO,
    ConsoleLeaderProcessHandle: HANDLE,
) -> i32 {
    let _ = ConsoleInitInfo;
    let _ = ConsoleLeaderProcessHandle;

    if FrontEnd.is_null() || ConsoleInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    InitSerialVtblIfNeeded();
    (*FrontEnd).Vtbl = core::ptr::addr_of_mut!(SerialVtbl);
    (*FrontEnd).Context = core::ptr::null_mut();
    (*FrontEnd).Context2 = core::ptr::null_mut();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SerialUnloadFrontEnd(FrontEnd: *mut FRONTEND) -> i32 {
    if FrontEnd.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    if !(*FrontEnd).Context.is_null() {
        SerialDeinitFrontEnd(FrontEnd);
    }
    STATUS_SUCCESS
}