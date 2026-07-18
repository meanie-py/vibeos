use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::conutils::stream::CON_STREAM;
use crate::conutils::outstream::{COORD, CONSOLE_SCREEN_BUFFER_INFO, CONSOLE_CURSOR_INFO};

#[repr(C)]
pub struct CON_SCREEN {
    pub Stream: *mut CON_STREAM,
    pub csbi: CONSOLE_SCREEN_BUFFER_INFO,
    pub cci: CONSOLE_CURSOR_INFO,
}

extern "system" {
    fn ConStreamGetOSHandle(Stream: *mut CON_STREAM) -> *mut c_void;
    fn IsConsoleHandle(hOutput: *mut c_void) -> BOOL;
    fn IsTTYHandle(hOutput: *mut c_void) -> BOOL;
    fn ConPuts(Stream: *mut CON_STREAM, szText: *const u16) -> i32;
    fn GetConsoleScreenBufferInfo(hOutput: *mut c_void, lpcsbi: *mut CONSOLE_SCREEN_BUFFER_INFO) -> BOOL;
    fn FillConsoleOutputAttribute(hOutput: *mut c_void, wAttribute: u16, nLength: u32, dwWriteCoord: COORD, lpNumberOfAttrsWritten: *mut u32) -> BOOL;
    fn FillConsoleOutputCharacterW(hOutput: *mut c_void, cCharacter: u16, nLength: u32, dwWriteCoord: COORD, lpNumberOfCharsWritten: *mut u32) -> BOOL;
    fn SetConsoleCursorPosition(hOutput: *mut c_void, dwCursorPosition: COORD) -> BOOL;
    fn CreateFileW(lpFileName: *const u16, dwDesiredAccess: u32, dwShareMode: u32, lpSecurityAttributes: *mut c_void, dwCreationDisposition: u32, dwFlagsAndAttributes: u32, hTemplateFile: *mut c_void) -> *mut c_void;
    fn CloseHandle(hObject: *mut c_void) -> i32;
}
unsafe extern "C" {
    fn ConStreamWrite(Stream: *mut CON_STREAM, Buffer: *const u16, nLength: u32) -> i32;
}

const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const FILE_SHARE_READ: u32 = 0x00000001;
const FILE_SHARE_WRITE: u32 = 0x00000002;
const OPEN_EXISTING: u32 = 3;

#[no_mangle]
pub unsafe extern "C" fn ConGetScreenInfo(
    Screen: *mut CON_SCREEN,
    pcsbi: *mut CONSOLE_SCREEN_BUFFER_INFO,
) -> i32 {
    let Success: i32;
    let hOutput: *mut c_void;

    if Screen.is_null() || pcsbi.is_null() {
        return 0;
    }

    hOutput = ConStreamGetOSHandle((*Screen).Stream);

    if IsTTYHandle(hOutput) == 0 {
        return 0;
    }

    if IsConsoleHandle(hOutput) != 0 {
        Success = GetConsoleScreenBufferInfo(hOutput, core::ptr::addr_of_mut!((*Screen).csbi));
    } else {
        let hOutput_temp: *mut c_void;
        let conout: [u16; 8] = [0x43, 0x4F, 0x4E, 0x4F, 0x55, 0x54, 0x24, 0x00]; // "CONOUT$\0"
        hOutput_temp = CreateFileW(
            conout.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );

        Success = if IsConsoleHandle(hOutput_temp) != 0 {
            GetConsoleScreenBufferInfo(hOutput_temp, core::ptr::addr_of_mut!((*Screen).csbi))
        } else {
            0
        };

        CloseHandle(hOutput_temp);
    }

    if Success != 0 {
        core::ptr::write_volatile(pcsbi, (*Screen).csbi);
    }

    Success
}

#[no_mangle]
pub unsafe extern "C" fn ConClearScreen(Screen: *const CON_SCREEN) {
    let hOutput: HANDLE;

    if Screen.is_null() {
        return;
    }

    hOutput = ConStreamGetOSHandle((*Screen).Stream);

    if IsConsoleHandle(hOutput) != 0 {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();

        GetConsoleScreenBufferInfo(hOutput, &mut csbi);

        let mut coPos: COORD = core::mem::zeroed();
        coPos.X = 0;
        coPos.Y = 0;

        let mut dwWritten: u32 = 0;

        FillConsoleOutputAttribute(
            hOutput,
            csbi.wAttributes,
            (csbi.dwSize.X as u32) * (csbi.dwSize.Y as u32),
            coPos,
            core::ptr::addr_of_mut!(dwWritten),
        );

        FillConsoleOutputCharacterW(
            hOutput,
            ' ' as u16,
            (csbi.dwSize.X as u32) * (csbi.dwSize.Y as u32),
            coPos,
            core::ptr::addr_of_mut!(dwWritten),
        );

        SetConsoleCursorPosition(hOutput, coPos);
    } else if IsTTYHandle(hOutput) != 0 {
        let tty_clear: [u16; 11] = [
            0x1B, 0x5B, 0x32, 0x4A,
            0x1B, 0x5B, 0x31, 0x3B, 0x31, 0x48,
            0,
        ];
        ConPuts((*Screen).Stream, tty_clear.as_ptr() as *const u16);
    } else {
        let ch: u16 = 0x0C; // form feed
        ConStreamWrite((*Screen).Stream, &ch as *const u16, 1);
    }
}
