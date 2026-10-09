use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::conutils::stream::{CON_STREAM, CON_STREAM_MODE};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct COORD {
    pub X: i16,
    pub Y: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SMALL_RECT {
    pub Left: i16,
    pub Top: i16,
    pub Right: i16,
    pub Bottom: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CONSOLE_SCREEN_BUFFER_INFO {
    pub dwSize: COORD,
    pub dwCursorPosition: COORD,
    pub wAttributes: u16,
    pub srWindow: SMALL_RECT,
    pub dwMaximumWindowSize: COORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CONSOLE_CURSOR_INFO {
    pub dwSize: u32,
    pub bVisible: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CHAR_INFO {
    pub Char: u16,
    pub Attributes: u16,
}

extern "system" {
    fn GetConsoleOutputCP() -> u32;
    fn GetProcessHeap() -> *mut c_void;
    fn HeapAlloc(hHeap: *mut c_void, dwFlags: u32, dwBytes: usize) -> *mut c_void;
    fn HeapFree(hHeap: *mut c_void, dwFlags: u32, lpMem: *mut c_void) -> i32;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const u8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
    fn SetLastError(dwErrCode: u32);
    fn WideCharToMultiByte(CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut u8, cbMultiByte: i32, lpDefaultChar: *const u8, lpUsedDefaultChar: *mut i32) -> i32;
    fn WriteConsoleW(hConsoleOutput: *mut c_void, lpBuffer: *const u16, nNumberOfCharsToWrite: u32, lpNumberOfCharsWritten: *mut u32, lpReserved: *mut c_void) -> i32;
    fn WriteFile(hFile: *mut c_void, lpBuffer: *const c_void, nNumberOfBytesToWrite: u32, lpNumberOfBytesWritten: *mut u32, lpOverlapped: *mut c_void) -> i32;
    fn EnterCriticalSection(CriticalSection: *mut RTL_CRITICAL_SECTION);
    fn LeaveCriticalSection(CriticalSection: *mut RTL_CRITICAL_SECTION);
    fn K32LoadStringExW(hInstance: HINSTANCE, uID: UINT, LanguageId: LANGID, lpBuffer: PWSTR, cchBufferMax: i32) -> i32;
    fn FormatMessageW(dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, lpBuffer: *mut u16, nSize: u32, Arguments: *const va_list) -> u32;
    fn LocalFree(hMem: *mut c_void) -> *mut c_void;
    fn FormatMessageSafeW(dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, lpBuffer: *mut u16, nSize: u32, Arguments: *mut va_list) -> u32;
}
unsafe extern "C" {
    fn ConStreamGetOSHandle(Stream: *mut CON_STREAM) -> *mut c_void;
    fn IsConsoleHandle(hOutput: *mut c_void) -> i32;
    fn IsTTYHandle(hOutput: *mut c_void) -> i32;
}

extern "C" {
    // wcslen is msvcrt cdecl (`_wcslen`, no @N) — must NOT be in the stdcall
    // Win32 block above, else i386 mangles it to `_wcslen@4` and link fails.
    fn wcslen(s: *const u16) -> usize;
    fn va_start(ap: *mut va_list, ...);
    fn va_end(ap: *mut va_list);
    // msvcrt cdecl (`@ cdecl _vsnwprintf` in msvcrt.spec). The original
    // ConPrintfV called the strsafe FORCEINLINE StringCchVPrintfExW, which only
    // resolved on x86 because clang emitted an out-of-line weak copy from a C TU
    // that included <strsafe.h>; arm64 clang emitted no such copy → undefined at
    // link. Calling _vsnwprintf directly (what strsafe does internally) is
    // arch-neutral — see pciidex-rust/src/inlines.rs for the same import.
    fn _vsnwprintf(buffer: *mut u16, count: usize, format: *const u16, argptr: va_list) -> i32;
}

pub const CON_RC_STRING_MAX_SIZE: usize = 4096;

const FORMAT_MESSAGE_ALLOCATE_BUFFER: u32 = 0x00000100;
const FORMAT_MESSAGE_FROM_STRING: u32 = 0x00000400;
const FORMAT_MESSAGE_FROM_HMODULE: u32 = 0x00000800;
const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x00001000;
const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x00000200;
const FORMAT_MESSAGE_ARGUMENT_ARRAY: u32 = 0x00002000;

extern "C" {
    pub fn ConPrintf(Stream: *mut CON_STREAM, szStr: *const u16, ...) -> i32;
    pub fn ConResPutsEx(Stream: *mut CON_STREAM, hInstance: HINSTANCE, uID: u32, LanguageId: LANGID) -> i32;
    pub fn ConResPrintfEx(Stream: *mut CON_STREAM, hInstance: HINSTANCE, uID: UINT, LanguageId: LANGID, ...) -> i32;
    pub fn ConResPrintf(Stream: *mut CON_STREAM, uID: u32, ...) -> i32;
    pub fn ConMsgPrintf(Stream: *mut CON_STREAM, dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, ...) -> i32;
    pub fn ConResMsgPrintfEx(Stream: *mut CON_STREAM, hInstance: HINSTANCE, dwFlags: DWORD, uID: UINT, LanguageId: LANGID, ...) -> i32;
    pub fn ConResMsgPrintf(Stream: *mut CON_STREAM, dwFlags: u32, uID: u32, ...) -> i32;
}

unsafe fn write_console(Stream: *mut CON_STREAM, szStr: *const u16, len: u32) -> i32 {
    let TotalLen: u32 = len;
    let mut dwNumBytes: u32 = 0;

    if szStr.is_null() || len == 0 {
        return 0;
    }

    if (*Stream).IsConsole != 0 {
        let mut cstr: *const u16 = szStr;
        let mut remaining = len;

        while remaining > 0 {
            let cchWrite = if remaining < 32767 { remaining } else { 32767 };
            WriteConsoleW(
                (*Stream).hHandle,
                cstr,
                cchWrite,
                &mut dwNumBytes,
                core::ptr::null_mut(),
            );
            cstr = cstr.add(cchWrite as usize);
            remaining -= cchWrite;
        }

        return TotalLen as i32;
    }

    if (*Stream).Mode == CON_STREAM_MODE::WideText || (*Stream).Mode == CON_STREAM_MODE::UTF16Text {
        let mut cstr: *const u16 = szStr;
        let mut remaining = len;

        while remaining > 0 {
            let mut p: *const u16 = cstr;
            while remaining > 0 && *p != 0x000A {
                p = p.add(1);
                remaining -= 1;
            }
            let byte_count = ((p as usize).wrapping_sub(cstr as usize)) as u32 * 2;
            WriteFile(
                (*Stream).hHandle,
                cstr as *const c_void,
                byte_count,
                &mut dwNumBytes,
                core::ptr::null_mut(),
            );
            if remaining > 0 && *p == 0x000A {
                let is_not_cr = if p as usize == cstr as usize {
                    true
                } else {
                    *(p.sub(1)) != 0x000D
                };
                if is_not_cr {
                    let crlf: [u16; 2] = [0x000D, 0x000A];
                    WriteFile(
                        (*Stream).hHandle,
                        crlf.as_ptr() as *const c_void,
                        4,
                        &mut dwNumBytes,
                        core::ptr::null_mut(),
                    );
                } else {
                    let lf: [u16; 1] = [0x000A];
                    WriteFile(
                        (*Stream).hHandle,
                        lf.as_ptr() as *const c_void,
                        2,
                        &mut dwNumBytes,
                        core::ptr::null_mut(),
                    );
                }
                p = p.add(1);
                remaining -= 1;
            }
            cstr = p;
        }
    } else if (*Stream).Mode == CON_STREAM_MODE::UTF8Text || (*Stream).Mode == CON_STREAM_MODE::AnsiText {
        let _CP_THREAD_ACP: u32 = 3;
        let INVALID_CP: u32 = 0xFFFFFFFF;
        let MB_LEN_MAX: usize = 5;

        let CodePage: u32;
        if (*Stream).CodePage == INVALID_CP {
            CodePage = GetConsoleOutputCP();
        } else {
            CodePage = (*Stream).CodePage;
        }

        let buffer = HeapAlloc(
            GetProcessHeap(),
            8,
            len as usize * MB_LEN_MAX,
        ) as *mut u8;
        if buffer.is_null() {
            SetLastError(8);
            return 0;
        }
        let mut remaining = WideCharToMultiByte(
            CodePage,
            0,
            szStr,
            len as i32,
            buffer,
            (len as usize * MB_LEN_MAX) as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as u32;

        let mut cursor: *const u8 = buffer as *const u8;
        while remaining > 0 {
            let mut p: *const u8 = cursor;
            while remaining > 0 && *p != b'\n' {
                p = p.add(1);
                remaining -= 1;
            }
            let byte_count = ((p as usize).wrapping_sub(cursor as usize)) as u32;
            WriteFile(
                (*Stream).hHandle,
                cursor as *const c_void,
                byte_count,
                &mut dwNumBytes,
                core::ptr::null_mut(),
            );
            if remaining > 0 && *p == b'\n' {
                let is_not_cr = if p as usize == cursor as usize {
                    true
                } else {
                    *(p.sub(1)) != b'\r'
                };
                if is_not_cr {
                    WriteFile(
                        (*Stream).hHandle,
                        b"\r\n".as_ptr() as *const c_void,
                        2,
                        &mut dwNumBytes,
                        core::ptr::null_mut(),
                    );
                } else {
                    WriteFile(
                        (*Stream).hHandle,
                        b"\n".as_ptr() as *const c_void,
                        1,
                        &mut dwNumBytes,
                        core::ptr::null_mut(),
                    );
                }
                p = p.add(1);
                remaining -= 1;
            }
            cursor = p;
        }
        HeapFree(GetProcessHeap(), 0, buffer as *mut c_void);
    } else {
        WriteFile(
            (*Stream).hHandle,
            szStr as *const c_void,
            len,
            &mut dwNumBytes,
            core::ptr::null_mut(),
        );
    }

    TotalLen as i32
}

#[no_mangle]
pub unsafe extern "system" fn ConWrite(
    Stream: *mut CON_STREAM,
    szStr: *const u16,
    len: u32,
) -> i32 {
    write_console(Stream, szStr, len)
}

#[no_mangle]
pub unsafe extern "C" fn ConStreamWrite(
    Stream: *mut CON_STREAM,
    szStr: *const u16,
    len: u32,
) -> i32 {
    let result_len: i32;

    EnterCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

    let write_func = (*Stream).WriteFunc;
    if let Some(func) = write_func {
        result_len = func(Stream, szStr, len);
    } else {
        result_len = write_console(Stream, szStr, len);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

    result_len
}

#[no_mangle]
pub unsafe extern "C" fn ConPuts(
    Stream: *mut CON_STREAM,
    szStr: *const u16,
) -> i32 {
    let mut Len: i32;

    Len = wcslen(szStr) as i32;
    EnterCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));
    if let Some(func) = (*Stream).WriteFunc {
        Len = func(Stream, szStr, Len as u32);
    } else {
        Len = write_console(Stream, szStr, Len as u32);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

    if Len < 0 {
        Len = 0;
    }

    Len
}

#[no_mangle]
pub unsafe extern "C" fn ConPrintfV(
    Stream: *mut CON_STREAM,
    szStr: *const u16,
    args: va_list,
) -> i32 {
    let mut Len: i32;
    let mut bufSrc: [u16; CON_RC_STRING_MAX_SIZE] = [0u16; CON_RC_STRING_MAX_SIZE];
    let pEnd: *mut u16;

    // strsafe StringCchVPrintfExW(dest, cch, &pEnd, NULL, 0, fmt, args) semantics
    // via the msvcrt cdecl primitive it wraps: bounded format, guaranteed NUL, and
    // pEnd → the terminating NUL (so the existing pEnd-based length math below is
    // unchanged). _vsnwprintf returns the wide-char count, or <0 when the result
    // (incl. NUL) does not fit `cch`.
    {
        let cch = bufSrc.len();
        let mut n = _vsnwprintf(bufSrc.as_mut_ptr(), cch, szStr, args);
        if n < 0 || (n as usize) >= cch {
            n = (cch - 1) as i32; // truncated — terminate at the last slot
        }
        bufSrc[n as usize] = 0;
        pEnd = bufSrc.as_mut_ptr().add(n as usize);
    }

    Len = if pEnd.is_null() {
        wcslen(bufSrc.as_ptr()) as i32
    } else {
        ((pEnd as usize).wrapping_sub(bufSrc.as_ptr() as usize) / 2) as i32
    };

    EnterCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));
    if let Some(func) = (*Stream).WriteFunc {
        Len = func(Stream, bufSrc.as_ptr() as *const u16, Len as u32);
    } else {
        Len = write_console(Stream, bufSrc.as_ptr() as *const u16, Len as u32);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

    if Len < 0 {
        Len = 0;
    }

    Len
}

#[no_mangle]
pub unsafe extern "C" fn ConResPuts(stream: *mut CON_STREAM, uID: u32) -> i32 {
    let mut szStr: *mut u16 = core::ptr::null_mut();
    let Len = K32LoadStringExW(
        core::ptr::null_mut(),
        uID,
        ((ntdll_sys::nt::SUBLANG_NEUTRAL as u16) << 10)
            | (ntdll_sys::nt::LANG_NEUTRAL as u16) as LANGID,
        core::ptr::addr_of_mut!(szStr) as PWSTR,
        0,
    );

    if !szStr.is_null() && Len != 0 {
        let result_len;
        EnterCriticalSection(core::ptr::addr_of_mut!((*stream).Lock));
        if let Some(func) = (*stream).WriteFunc {
            result_len = func(stream, szStr, Len as u32);
        } else {
            result_len = write_console(stream, szStr, Len as u32);
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*stream).Lock));
        if result_len < 0 { 0 } else { result_len }
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn ConResPrintfExV(
    Stream: *mut CON_STREAM,
    hInstance: HINSTANCE,
    uID: u32,
    LanguageId: LANGID,
    args: va_list,
) -> i32 {
    let Len: i32;
    let mut bufSrc: [u16; CON_RC_STRING_MAX_SIZE] = [0u16; CON_RC_STRING_MAX_SIZE];
    Len = K32LoadStringExW(hInstance, uID, LanguageId, bufSrc.as_mut_ptr(), CON_RC_STRING_MAX_SIZE as i32);
    if Len != 0 {
        return ConPrintfV(Stream, bufSrc.as_ptr(), args);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ConResPrintfV(
    Stream: *mut CON_STREAM,
    uID: u32,
    args: va_list,
) -> i32 {
    ConResPrintfExV(
        Stream,
        core::ptr::null_mut(),
        uID,
        ((ntdll_sys::nt::SUBLANG_NEUTRAL as u16) << 10) | (ntdll_sys::nt::LANG_NEUTRAL as u16),
        args,
    )
}

#[no_mangle]
pub unsafe extern "C" fn ConMsgPuts(
    Stream: *mut CON_STREAM,
    mut dwFlags: u32,
    lpSource: *const c_void,
    dwMessageId: u32,
    dwLanguageId: u32,
) -> i32 {
    let mut Len: i32;
    let dwLength: u32;
    let mut lpMsgBuf: *mut u16 = core::ptr::null_mut();

    dwFlags |= FORMAT_MESSAGE_ALLOCATE_BUFFER;
    dwFlags |= FORMAT_MESSAGE_IGNORE_INSERTS;
    dwFlags &= !FORMAT_MESSAGE_ARGUMENT_ARRAY;

    dwLength = FormatMessageW(
        dwFlags,
        lpSource,
        dwMessageId,
        dwLanguageId,
        core::ptr::addr_of_mut!(lpMsgBuf) as *mut u16,
        0,
        core::ptr::null_mut(),
    );

    Len = dwLength as i32;

    if !lpMsgBuf.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));
        if let Some(func) = (*Stream).WriteFunc {
            Len = func(Stream, lpMsgBuf, dwLength);
        } else {
            Len = write_console(Stream, lpMsgBuf, dwLength);
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

        if Len < 0 {
            Len = 0;
        }
        LocalFree(lpMsgBuf as *mut c_void);
    }

    Len
}

#[no_mangle]
pub unsafe extern "C" fn ConMsgPrintf2V(
    Stream: *mut CON_STREAM,
    mut dwFlags: u32,
    lpSource: *const c_void,
    dwMessageId: u32,
    dwLanguageId: u32,
    args: va_list,
) -> i32 {
    let mut Len: i32;
    let dwLength: u32;
    let mut lpMsgBuf: *mut u16 = core::ptr::null_mut();

    dwFlags |= FORMAT_MESSAGE_ALLOCATE_BUFFER;
    dwFlags |= FORMAT_MESSAGE_IGNORE_INSERTS;
    dwFlags &= !FORMAT_MESSAGE_ARGUMENT_ARRAY;

    dwLength = FormatMessageW(
        dwFlags,
        lpSource,
        dwMessageId,
        dwLanguageId,
        core::ptr::addr_of_mut!(lpMsgBuf) as *mut u16,
        0,
        core::ptr::null_mut(),
    );

    Len = dwLength as i32;

    if !lpMsgBuf.is_null() {
        Len = ConPrintfV(Stream, lpMsgBuf, args);
        if Len < 0 {
            Len = 0;
        }
        LocalFree(lpMsgBuf as *mut c_void);
    }

    Len
}

#[no_mangle]
pub unsafe extern "C" fn ConMsgPrintfV(
    Stream: *mut CON_STREAM,
    mut dwFlags: u32,
    lpSource: *const c_void,
    dwMessageId: u32,
    dwLanguageId: u32,
    Arguments: *mut va_list,
) -> i32 {
    let mut Len: i32;
    let dwLength: u32;
    let mut lpMsgBuf: *mut u16 = core::ptr::null_mut();

    dwFlags |= FORMAT_MESSAGE_ALLOCATE_BUFFER;

    dwLength = FormatMessageSafeW(
        dwFlags,
        lpSource,
        dwMessageId,
        dwLanguageId,
        core::ptr::addr_of_mut!(lpMsgBuf) as *mut u16,
        0,
        Arguments,
    );

    Len = dwLength as i32;

    if !lpMsgBuf.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));
        if let Some(func) = (*Stream).WriteFunc {
            Len = func(Stream, lpMsgBuf, dwLength);
        } else {
            Len = write_console(Stream, lpMsgBuf, dwLength);
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

        if Len < 0 {
            Len = 0;
        }
        LocalFree(lpMsgBuf as *mut c_void);
    }

    Len
}

#[no_mangle]
pub unsafe extern "C" fn ConResMsgPrintfExV(
    Stream: *mut CON_STREAM,
    hInstance: HINSTANCE,
    mut dwFlags: u32,
    uID: UINT,
    LanguageId: LANGID,
    Arguments: *mut va_list,
) -> i32 {
    let mut Len: i32;
    let dwLength: u32;
    let mut lpMsgBuf: *mut u16 = core::ptr::null_mut();
    let mut bufSrc: [u16; CON_RC_STRING_MAX_SIZE] = [0u16; CON_RC_STRING_MAX_SIZE];

    Len = K32LoadStringExW(hInstance, uID, LanguageId, bufSrc.as_mut_ptr(), CON_RC_STRING_MAX_SIZE as i32);
    if Len == 0 {
        return Len;
    }

    // Sanitize dwFlags (faithful to outstream.c ConResMsgPrintfExV): always
    // allocate, and since the string was already manually loaded, clear
    // FROM_HMODULE/FROM_SYSTEM and force FROM_STRING.
    dwFlags |= FORMAT_MESSAGE_ALLOCATE_BUFFER;
    dwFlags &= !(FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM);
    dwFlags |= FORMAT_MESSAGE_FROM_STRING;

    dwLength = FormatMessageSafeW(
        dwFlags,
        bufSrc.as_ptr() as *const c_void,
        0,
        0,
        core::ptr::addr_of_mut!(lpMsgBuf) as *mut u16,
        0,
        Arguments,
    );

    Len = dwLength as i32;

    if !lpMsgBuf.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));
        if let Some(func) = (*Stream).WriteFunc {
            Len = func(Stream, lpMsgBuf, dwLength);
        } else {
            Len = write_console(Stream, lpMsgBuf, dwLength);
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock));

        if Len < 0 {
            Len = 0;
        }
        LocalFree(lpMsgBuf as *mut c_void);
    }

    Len
}

#[no_mangle]
pub unsafe extern "C" fn ConResMsgPrintfV(
    Stream: *mut CON_STREAM,
    dwFlags: DWORD,
    uID: UINT,
    Arguments: *mut va_list,
) -> i32 {
    ConResMsgPrintfExV(
        Stream,
        core::ptr::null_mut(),
        dwFlags,
        uID,
        (((ntdll_sys::nt::SUBLANG_NEUTRAL as u16) << 10) | (ntdll_sys::nt::LANG_NEUTRAL as u16)) as LANGID,
        Arguments,
    )
}

#[no_mangle]
pub unsafe extern "C" fn ConClearLine(Stream: *mut CON_STREAM) {
    extern "system" {
        fn GetConsoleScreenBufferInfo(hConsoleOutput: *mut c_void, lpcsbi: *mut CONSOLE_SCREEN_BUFFER_INFO) -> i32;
        fn FillConsoleOutputCharacterW(hConsoleOutput: *mut c_void, cCharacter: u16, nLength: u32, dwWriteCoord: COORD, lpNumberOfCharsWritten: *mut u32) -> i32;
        fn SetConsoleCursorPosition(hConsoleOutput: *mut c_void, dwCursorPosition: COORD) -> i32;
    }

    let hOutput = ConStreamGetOSHandle(Stream);

    if IsConsoleHandle(hOutput) != 0 {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
        let mut dwWritten: u32 = 0;

        GetConsoleScreenBufferInfo(hOutput, &mut csbi);

        // C zeroes csbi.dwCursorPosition.X in place, then uses that (0, Y) coord
        // for BOTH the fill start and the cursor reposition — so the whole line
        // (including the part before the cursor) is erased.
        let new_pos = COORD { X: 0, Y: csbi.dwCursorPosition.Y };
        let clear_len = csbi.dwSize.X as u32;

        FillConsoleOutputCharacterW(
            hOutput,
            ' ' as u16,
            clear_len,
            new_pos,
            &mut dwWritten,
        );
        SetConsoleCursorPosition(hOutput, new_pos);
    } else if IsTTYHandle(hOutput) != 0 {
        let seq: [u16; 9] = [0x1B, 0x5B, 0x32, 0x4B, 0x1B, 0x5B, 0x31, 0x47, 0];
        ConPuts(Stream, seq.as_ptr());
    }
}
