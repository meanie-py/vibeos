use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "C" {
    fn fflush(stream: *mut FILE) -> i32;
    fn _fileno(stream: *mut FILE) -> i32;
    fn _setmode(fd: i32, mode: i32) -> i32;
    fn _get_osfhandle(fd: i32) -> *mut c_void;
}
extern "system" {
    fn IsConsoleHandle(hConsole: *mut c_void) -> i32;
    fn InitializeCriticalSection(lpCriticalSection: *mut c_void);
    fn ConWrite(Stream: *mut CON_STREAM, szStr: *const u16, len: u32) -> i32;
    fn FlushFileBuffers(hFile: HANDLE) -> i32;
    fn _open_osfhandle(osfhandle: isize, mode: i32) -> i32;
    fn _fdopen(fd: i32, mode: *const i8) -> *mut FILE;
}

// ---- TYPE DEFINITIONS (from conutils headers) ----

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CON_STREAM_MODE {
    Binary = 0,
    AnsiText = 1,
    WideText = 2,
    UTF16Text = 3,
    UTF8Text = 4,
}

pub type CON_WRITE_FUNC = Option<unsafe extern "system" fn(Stream: *mut CON_STREAM, szStr: *const u16, len: u32) -> i32>;

pub const INVALID_HANDLE_VALUE: *mut c_void = (-1_isize) as *mut c_void;

// CON_STREAM (from stream_private.h — non-USE_CRT variant)
#[repr(C)]
pub struct CON_STREAM {
    pub WriteFunc: CON_WRITE_FUNC,
    pub IsInitialized: BOOL,
    pub Lock: RTL_CRITICAL_SECTION,
    pub hHandle: HANDLE,
    pub IsConsole: BOOL,
    pub Mode: CON_STREAM_MODE,
    pub CodePage: UINT,
}

// ---- file-scope globals ----
// These are the canonical CON_STREAM instances consumed by C callers (cmd.exe's
// StdIn/StdOut/StdErr = &csStd*). The original cohab relied on a stale C stream.c
// object in libconutils.a to provide them on x86; arm64 (clean, stub-only) had no
// definer → undefined at link. #[no_mangle] makes conutils-rust itself the single
// canonical provider on every arch (stream.c is no longer built).
#[no_mangle]
pub static mut csStdIn: CON_STREAM = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut csStdOut: CON_STREAM = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut csStdErr: CON_STREAM = unsafe { core::mem::zeroed() };

#[cfg(feature = "USE_CRT")]
const _O_BINARY: i32 = 0x8000;
#[cfg(feature = "USE_CRT")]
const _O_TEXT: i32 = 0x4000;
#[cfg(feature = "USE_CRT")]
const _O_WTEXT: i32 = 0x10000;
#[cfg(feature = "USE_CRT")]
const _O_U16TEXT: i32 = 0x20000;
#[cfg(feature = "USE_CRT")]
const _O_U8TEXT: i32 = 0x40000;

#[cfg(feature = "USE_CRT")]
static ConToCRTMode: [i32; 5] = [
    _O_BINARY,  // Binary    (untranslated)
    _O_TEXT,    // AnsiText  (translated)
    _O_WTEXT,   // WideText  (UTF16 with BOM; translated)
    _O_U16TEXT, // UTF16Text (UTF16 without BOM; translated)
    _O_U8TEXT,  // UTF8Text  (UTF8  without BOM; translated)
];

const CP_UTF8: u32 = 65001;
const INVALID_CP: u32 = (-1i32) as u32;

// ---- functions ----

#[no_mangle]
pub unsafe extern "C" fn ConStreamInitEx(
    Stream: *mut CON_STREAM,
    Handle: *mut c_void,
    Mode: CON_STREAM_MODE,
    CacheCodePage: u32,
    WriteFunc: CON_WRITE_FUNC,
) -> i32 {
    if Stream.is_null() || Handle.is_null() || (Mode as i32) > CON_STREAM_MODE::UTF8Text as i32 {
        return 0;
    }

    #[cfg(feature = "USE_CRT")]
    {
        (*Stream).WriteFunc = WriteFunc;
        // CRT path — not implemented in Rust yet; caller provides FILE* as Handle
        (*Stream).IsInitialized = 1;
        return 1;
    }

    #[cfg(not(feature = "USE_CRT"))]
    {
        if Handle == INVALID_HANDLE_VALUE {
            return 0;
        }

        if (*Stream).IsInitialized == 0 {
            InitializeCriticalSection(core::ptr::addr_of_mut!((*Stream).Lock) as *mut c_void);
            (*Stream).IsInitialized = 1;
        }

        (*Stream).hHandle = Handle;
        (*Stream).IsConsole = IsConsoleHandle((*Stream).hHandle);

        (*Stream).Mode = Mode;
        if Mode == CON_STREAM_MODE::AnsiText {
            (*Stream).CodePage = CacheCodePage;
        } else if Mode == CON_STREAM_MODE::UTF8Text {
            (*Stream).CodePage = CP_UTF8;
        } else {
            (*Stream).CodePage = INVALID_CP;
        }
    }

    if WriteFunc.is_some() {
        (*Stream).WriteFunc = WriteFunc;
    } else {
        (*Stream).WriteFunc = Some(ConWrite);
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn ConStreamInit(
    stream: *mut CON_STREAM,
    handle: *mut c_void,
    mode: CON_STREAM_MODE,
    cache_code_page: u32,
) -> i32 {
    ConStreamInitEx(stream, handle, mode, cache_code_page, Some(ConWrite))
}

#[no_mangle]
pub unsafe extern "C" fn ConStreamSetMode(
    Stream: *mut CON_STREAM,
    Mode: CON_STREAM_MODE,
    CacheCodePage: u32,
) -> i32 {
    if Stream.is_null() || (Mode as i32) > CON_STREAM_MODE::UTF8Text as i32 {
        return 0;
    }

    #[cfg(feature = "USE_CRT")]
    {
        if (*Stream).IsInitialized == 0 {
            return 0;
        }
        // CRT-only: fflush + setmode would go here
    }

    #[cfg(not(feature = "USE_CRT"))]
    {
        (*Stream).Mode = Mode;
        if Mode == CON_STREAM_MODE::AnsiText {
            (*Stream).CodePage = CacheCodePage;
        } else if Mode == CON_STREAM_MODE::UTF8Text {
            (*Stream).CodePage = CP_UTF8;
        } else {
            (*Stream).CodePage = INVALID_CP;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn ConStreamSetCacheCodePage(
    Stream: *mut CON_STREAM,
    CacheCodePage: u32,
) -> i32 {
    if Stream.is_null() {
        return 0;
    }

    let mode = (*Stream).Mode;
    if mode == CON_STREAM_MODE::AnsiText {
        (*Stream).CodePage = CacheCodePage;
    } else if mode == CON_STREAM_MODE::UTF8Text {
        (*Stream).CodePage = CP_UTF8;
    } else {
        (*Stream).CodePage = INVALID_CP;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn ConStreamGetOSHandle(Stream: *mut CON_STREAM) -> *mut c_void {
    if Stream.is_null() {
        return INVALID_HANDLE_VALUE;
    }
    #[cfg(not(feature = "USE_CRT"))]
    {
        return (*Stream).hHandle;
    }
    #[cfg(feature = "USE_CRT")]
    {
        return INVALID_HANDLE_VALUE;
    }
}

#[no_mangle]
pub unsafe extern "C" fn ConStreamSetOSHandle(
    Stream: *mut CON_STREAM,
    Handle: HANDLE,
) -> i32 {
    if Stream.is_null() {
        return 0;
    }

    #[cfg(feature = "USE_CRT")]
    {
        if (*Stream).IsInitialized == 0 {
            return 0;
        }
        let fdOut: i32 = _open_osfhandle(Handle as isize, 0x4000i32 /* _O_TEXT */);
        let fpOut: *mut FILE = _fdopen(fdOut, b"w\0".as_ptr() as *const i8);
        if !fpOut.is_null() {
            (*Stream).hHandle = fpOut as HANDLE;
        }
        return 1;
    }

    #[cfg(not(feature = "USE_CRT"))]
    {
        if (*Stream).hHandle != INVALID_HANDLE_VALUE {
            FlushFileBuffers((*Stream).hHandle);
        }
        (*Stream).hHandle = Handle;
        (*Stream).IsConsole = IsConsoleHandle((*Stream).hHandle);
        return 1;
    }
}
