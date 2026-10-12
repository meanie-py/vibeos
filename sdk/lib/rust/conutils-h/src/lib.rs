//! conutils-h — the declarations of sdk/lib/conutils (stream.h, outstream.h), once.
//!
//! conutils-rust IS conutils, but it is a staticlib: a consumer crate cannot depend on it
//! without linking a second copy of every symbol beside the one the `conutils` CMake library
//! already brings in. So each console app re-declared the header in its prelude (diskpart
//! by hand), and cacls, logoff, runas and shutdown -- ported together -- each found
//! `CON_STREAM`, `StdOut` and `ConInitStdStreams` defined nowhere. A header the tree ports
//! per module wants a shared crate (the strsafe-rust / win32macros-rust pattern); `port
//! resolve` re-exports these names by name into any prelude that needs them.
//!
//! Types are core primitives only, identical on all three targets: PCON_STREAM is a pointer
//! to an opaque struct (stream.h calls it a "shadow type"), UINT/DWORD are u32, INT/BOOL
//! i32, LANGID u16, HANDLE/PVOID `*mut c_void`, WCHAR u16. The calling conventions are the
//! header's: plain prototypes are cdecl (`extern "C"`), `ConWrite` is `__stdcall`.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::{c_char, c_void};

/// stream.h: `typedef struct _CON_STREAM CON_STREAM, *PCON_STREAM;` -- implementation-
/// specific, only ever handled through a pointer.
#[repr(C)]
pub struct CON_STREAM {
    _opaque: [u8; 0],
}
pub type PCON_STREAM = *mut CON_STREAM;

/// stream.h `CON_STREAM_MODE`: a C enum, so `int`.
pub type CON_STREAM_MODE = i32;
pub const Binary: CON_STREAM_MODE = 0;
pub const AnsiText: CON_STREAM_MODE = 1;
pub const WideText: CON_STREAM_MODE = 2;
pub const UTF16Text: CON_STREAM_MODE = 3;
pub const UTF8Text: CON_STREAM_MODE = 4;

/// stream.h: `#define INVALID_CP ((UINT)-1)`.
pub const INVALID_CP: u32 = u32::MAX;

/// stream.h: `typedef INT (__stdcall *CON_WRITE_FUNC)(PCON_STREAM, PCTCH, DWORD);`
pub type CON_WRITE_FUNC = Option<unsafe extern "system" fn(PCON_STREAM, *const u16, u32) -> i32>;

extern "C" {
    // stream.h: `extern CON_STREAM csStdIn;` and siblings (conutils-rust's instances).
    pub static mut csStdIn: CON_STREAM;
    pub static mut csStdOut: CON_STREAM;
    pub static mut csStdErr: CON_STREAM;

    // stream.h
    pub fn ConStreamInitEx(
        Stream: PCON_STREAM,
        Handle: *mut c_void,
        Mode: CON_STREAM_MODE,
        CacheCodePage: u32,
        WriteFunc: CON_WRITE_FUNC,
    ) -> i32;
    pub fn ConStreamInit(
        Stream: PCON_STREAM,
        Handle: *mut c_void,
        Mode: CON_STREAM_MODE,
        CacheCodePage: u32,
    ) -> i32;
    pub fn ConStreamSetMode(Stream: PCON_STREAM, Mode: CON_STREAM_MODE, CacheCodePage: u32) -> i32;
    pub fn ConStreamSetCacheCodePage(Stream: PCON_STREAM, CacheCodePage: u32) -> i32;
    pub fn ConStreamGetOSHandle(Stream: PCON_STREAM) -> *mut c_void;
    pub fn ConStreamSetOSHandle(Stream: PCON_STREAM, Handle: *mut c_void) -> i32;

    // outstream.h
    pub fn ConStreamWrite(Stream: PCON_STREAM, szStr: *const u16, len: u32) -> i32;
    pub fn ConPuts(Stream: PCON_STREAM, szStr: *const u16) -> i32;
    pub fn ConPrintf(Stream: PCON_STREAM, szStr: *const u16, ...) -> i32;
    pub fn ConResPutsEx(Stream: PCON_STREAM, hInstance: *mut c_void, uID: u32, LanguageId: u16) -> i32;
    pub fn ConResPuts(Stream: PCON_STREAM, uID: u32) -> i32;
    pub fn ConResPrintfEx(Stream: PCON_STREAM, hInstance: *mut c_void, uID: u32, LanguageId: u16, ...) -> i32;
    pub fn ConResPrintf(Stream: PCON_STREAM, uID: u32, ...) -> i32;
    // The `va_list` forms (outstream.h). A `va_list` is a `char *` on every Windows target;
    // mode's UnderlinedResPrintf hands its caller's list straight on.
    pub fn ConPrintfV(Stream: PCON_STREAM, szStr: *const u16, args: *mut c_char) -> i32;
    pub fn ConResPrintfV(Stream: PCON_STREAM, uID: u32, args: *mut c_char) -> i32;
    pub fn ConMsgPuts(
        Stream: PCON_STREAM,
        dwFlags: u32,
        lpSource: *const c_void,
        dwMessageId: u32,
        dwLanguageId: u32,
    ) -> i32;
    pub fn ConMsgPrintf(
        Stream: PCON_STREAM,
        dwFlags: u32,
        lpSource: *const c_void,
        dwMessageId: u32,
        dwLanguageId: u32,
        ...
    ) -> i32;
    pub fn ConResMsgPrintfEx(
        Stream: PCON_STREAM,
        hInstance: *mut c_void,
        dwFlags: u32,
        uID: u32,
        LanguageId: u16,
        ...
    ) -> i32;
    pub fn ConResMsgPrintf(Stream: PCON_STREAM, dwFlags: u32, uID: u32, ...) -> i32;
    pub fn ConSetThreadUILanguage(LangId: u16) -> u16;
}

extern "system" {
    // outstream.h: `INT __stdcall ConWrite(...)`.
    pub fn ConWrite(Stream: PCON_STREAM, szStr: *const u16, len: u32) -> i32;
    // utils.h: `INT WINAPI K32LoadStringW(...)` (conutils' own, not kernel32's).
    pub fn K32LoadStringW(hInstance: *mut c_void, uID: u32, lpBuffer: *mut u16, nBufferMax: i32) -> i32;
    fn GetStdHandle(nStdHandle: u32) -> *mut c_void;
}

/// stream.h: `#define StdIn (&csStdIn)`.
pub const StdIn: PCON_STREAM = core::ptr::addr_of_mut!(csStdIn);
/// stream.h: `#define StdOut (&csStdOut)`.
pub const StdOut: PCON_STREAM = core::ptr::addr_of_mut!(csStdOut);
/// stream.h: `#define StdErr (&csStdErr)`.
pub const StdErr: PCON_STREAM = core::ptr::addr_of_mut!(csStdErr);

const STD_INPUT_HANDLE: u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
const STD_ERROR_HANDLE: u32 = -12i32 as u32;

/// stream.h `ConInitStdStreamsAndMode`, the non-USE_CRT arm: each standard stream on its
/// Win32 handle. A statement macro, so a function is the faithful port.
pub unsafe fn ConInitStdStreamsAndMode(Mode: CON_STREAM_MODE, CacheCodePage: u32) {
    unsafe {
        ConStreamInit(StdIn, GetStdHandle(STD_INPUT_HANDLE), Mode, CacheCodePage);
        ConStreamInit(StdOut, GetStdHandle(STD_OUTPUT_HANDLE), Mode, CacheCodePage);
        ConStreamInit(StdErr, GetStdHandle(STD_ERROR_HANDLE), Mode, CacheCodePage);
    }
}

/// stream.h: `#define ConInitStdStreams() ConInitStdStreamsAndMode(AnsiText, INVALID_CP)`.
pub unsafe fn ConInitStdStreams() {
    unsafe { ConInitStdStreamsAndMode(AnsiText, INVALID_CP) }
}

/// stream.h `ConStdStreamsSetCacheCodePage(InputCodePage, OutputCodePage)` (statement macro).
pub unsafe fn ConStdStreamsSetCacheCodePage(InputCodePage: u32, OutputCodePage: u32) {
    unsafe {
        ConStreamSetCacheCodePage(StdIn, InputCodePage);
        ConStreamSetCacheCodePage(StdOut, OutputCodePage);
        ConStreamSetCacheCodePage(StdErr, OutputCodePage);
    }
}
