use winedebug_rust::{TRACE, WARN};
// The crate's own functions: the definition is the declaration.
use crate::client::handle::CloseHandle;
use crate::file::create::CreateFileW;
use crate::file::rw::ReadFile;
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use ENCODING::*;
// HeapAlloc/HeapFree are pure ntdll forwarders (no @N symbol) → the nls_subst Rust
// wrappers; RtlUshortByteSwap is a FORCEINLINE intrinsic (no export) → also imported.
use crate::string::nls_subst::{HeapAlloc, HeapFree, RtlUshortByteSwap};

// ---- cohab glue: Wine debug macros (no-ops, as in the other ported modules) ----

// ---- cohab glue: kernel32 Rust exports this module calls (real stdcall @N exports) ----
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: LPDWORD) -> DWORD;
    fn GetFileTime(hFile: HANDLE, lpCreationTime: LPFILETIME, lpLastAccessTime: LPFILETIME, lpLastWriteTime: LPFILETIME) -> BOOL;
    fn GetFullPathNameW(lpFileName: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR) -> DWORD;
    fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: LPFILETIME);
    fn GetWindowsDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT, lpWideCharStr: LPWSTR, cchWideChar: INT) -> INT;
    fn WideCharToMultiByte(CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: INT, lpMultiByteStr: LPSTR, cbMultiByte: INT, lpDefaultChar: *const CHAR, lpUsedDefaultChar: *mut BOOL) -> INT;
    fn WriteFile(hFile: HANDLE, lpBuffer: LPCVOID, nNumberOfBytesToWrite: DWORD, lpNumberOfBytesWritten: LPDWORD, lpOverlapped: LPOVERLAPPED) -> BOOL;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR;
}

// ---- cohab glue: profile.c statics / function-like macros ----
#[inline]
unsafe fn IS_ENTRY_COMMENT(s: *const WCHAR) -> c_int {
    (*s == ';' as WCHAR) as c_int          // #define IS_ENTRY_COMMENT(str) ((str)[0] == ';')
}
#[inline]
unsafe fn ZeroMemory(dest: PVOID, len: usize) {
    core::ptr::write_bytes(dest as *mut u8, 0, len);
}
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 206 as ULONG;

static mut bom_utf8: [::core::ffi::c_char; 3] = [
    0xef as ::core::ffi::c_int as ::core::ffi::c_char,
    0xbb as ::core::ffi::c_int as ::core::ffi::c_char,
    0xbf as ::core::ffi::c_int as ::core::ffi::c_char,
];

// profile.c: `#define CurProfile (MRUProfile[0])`. The translation made CurProfile a
// SEPARATE static, so the MRU rotation (which assigns CurProfile to mean MRUProfile[0]) left
// the cache holding the old head twice and a profile nothing pointed at.
static mut MRUProfile: [*mut PROFILE; 10] = [
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
    ::core::ptr::null::<PROFILE>() as *mut PROFILE,
];

static mut emptystringW: [WCHAR; 1] = [0 as ::core::ffi::c_int as WCHAR];

static mut wininiW: [WCHAR; 8] = [
    'w' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    '.' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut critsect_debug: RTL_CRITICAL_SECTION_DEBUG = RTL_CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: ::core::ptr::null::<_RTL_CRITICAL_SECTION>() as *mut _RTL_CRITICAL_SECTION,
    ProcessLocksList: LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    EntryCount: 0,
    ContentionCount: 0,
    __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { WineDebugString: 0 },
};

static mut PROFILE_CritSect: RTL_CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut hex: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"0123456789ABCDEF") };

// (c2rust emitted a `static INIT_ARRAY = [run_static_initializers]` .init_array hook
//  here; dropped — all statics in this module are const-initialized, nothing to run.)

#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub enum ENCODING {
    ENCODING_ANSI = 1,
    ENCODING_UTF8,
    ENCODING_UTF16LE,
    ENCODING_UTF16BE,
}

#[repr(C)]
pub struct tagPROFILEKEY {
    pub value: *mut WCHAR,
    pub next: *mut tagPROFILEKEY,
    pub name: [WCHAR; 1],
}
pub type PROFILEKEY = tagPROFILEKEY;
pub type PPROFILEKEY = *mut tagPROFILEKEY;

#[repr(C)]
pub struct tagPROFILESECTION {
    pub key: *mut tagPROFILEKEY,
    pub next: *mut tagPROFILESECTION,
    pub name: [WCHAR; 1],
}
pub type PROFILESECTION = tagPROFILESECTION;
pub type PPROFILESECTION = *mut tagPROFILESECTION;

#[repr(C)]
pub struct PROFILE {
    pub changed: BOOL,
    pub section: *mut PROFILESECTION,
    pub filename: *mut WCHAR,
    pub LastWriteTime: FILETIME,
    pub encoding: ENCODING,
}

pub const N_CACHED_PROFILES: INT = 10;
unsafe fn PROFILE_CopyEntry(
    buffer: LPWSTR,
    mut value: LPCWSTR,
    len: ::core::ffi::c_int,
    strip_quote: BOOL,
) {
    let mut quote: WCHAR = 0;
    if buffer.is_null() {
        return;
    }
    if strip_quote != 0
        && (*value as ::core::ffi::c_int == '\'' as i32
            || *value as ::core::ffi::c_int == '"' as i32)
    {
        if *value.offset(1) as ::core::ffi::c_int != 0
            && *value.offset(
                strlenW(value).wrapping_sub(1) as isize,
            ) as ::core::ffi::c_int
                == *value as ::core::ffi::c_int
        {
            quote = *value;
            value = value.offset(1);
        }
    }
    lstrcpynW(buffer, value, len);
    if quote != 0 && len >= lstrlenW(value) {
        *buffer.offset(
            strlenW(buffer).wrapping_sub(1) as isize,
        ) = 0;
    }
}
unsafe fn PROFILE_ByteSwapShortBuffer(
    mut buffer: *mut WCHAR,
    mut len: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0;
    let mut shortbuffer: *mut USHORT = buffer as *mut USHORT;
    i = 0 as ::core::ffi::c_int;
    while i < len {
        *shortbuffer.offset(i as isize) = RtlUshortByteSwap(*shortbuffer.offset(i as isize));
        i += 1;
    }
}
unsafe fn PROFILE_WriteMarker(hFile: HANDLE, encoding: ENCODING) {
    let mut dwBytesWritten: DWORD = 0;
    let mut bom: WCHAR = 0;
    match encoding as u32 {
        2 => {
            WriteFile(
                hFile,
                (*(&raw const bom_utf8)).as_ptr() as LPCVOID,
                core::mem::size_of::<[u8; 3]>() as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
        }
        3 => {
            bom = 0xFEFF;
            WriteFile(
                hFile,
                &bom as *const WCHAR as LPCVOID,
                core::mem::size_of::<WCHAR>() as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
        }
        4 => {
            bom = 0xFFFE;
            WriteFile(
                hFile,
                &bom as *const WCHAR as LPCVOID,
                core::mem::size_of::<WCHAR>() as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
        }
        1 | _ => {}
    }
}
unsafe fn PROFILE_WriteLine(
    hFile: HANDLE,
    szLine: *mut WCHAR,
    mut len: ::core::ffi::c_int,
    encoding: ENCODING,
) {
    let mut write_buffer: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut write_buffer_len: ::core::ffi::c_int = 0;
    let mut dwBytesWritten: DWORD = 0;

    TRACE!(b"writing: %s\n\0" as *const u8 as *const ::core::ffi::c_char, debugstr_wn(szLine, len));

    match encoding as ::core::ffi::c_uint {
        1 => {
            write_buffer_len = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                szLine as LPCWSTR,
                len,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            write_buffer = HeapAlloc(GetProcessHeap(), 0, write_buffer_len as usize) as *mut ::core::ffi::c_char;
            if write_buffer.is_null() {
                return;
            }
            len = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                szLine as LPCWSTR,
                len,
                write_buffer as LPSTR,
                write_buffer_len,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            WriteFile(
                hFile,
                write_buffer as LPCVOID,
                len as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
            HeapFree(GetProcessHeap(), 0, write_buffer as *mut ::core::ffi::c_void);
        }
        2 => {
            write_buffer_len = WideCharToMultiByte(
                CP_UTF8 as UINT,
                0 as DWORD,
                szLine as LPCWSTR,
                len,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            write_buffer = HeapAlloc(GetProcessHeap(), 0, write_buffer_len as usize) as *mut ::core::ffi::c_char;
            if write_buffer.is_null() {
                return;
            }
            len = WideCharToMultiByte(
                CP_UTF8 as UINT,
                0 as DWORD,
                szLine as LPCWSTR,
                len,
                write_buffer as LPSTR,
                write_buffer_len,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            WriteFile(
                hFile,
                write_buffer as LPCVOID,
                len as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
            HeapFree(GetProcessHeap(), 0, write_buffer as *mut ::core::ffi::c_void);
        }
        3 => {
            WriteFile(
                hFile,
                szLine as LPCVOID,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
        }
        4 => {
            PROFILE_ByteSwapShortBuffer(szLine, len);
            WriteFile(
                hFile,
                szLine as LPCVOID,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
                &mut dwBytesWritten as *mut DWORD,
                core::ptr::null_mut(),
            );
        }
        _ => {
            FIXME!("encoding type %d not implemented\n", encoding as ::core::ffi::c_uint);
        }
    }
}
unsafe fn PROFILE_Save(
    hFile: HANDLE,
    section: *const PROFILESECTION,
    encoding: ENCODING,
) {
    let mut key: *mut PROFILEKEY;
    let mut buffer: *mut WCHAR;
    let mut p: *mut WCHAR;

    PROFILE_WriteMarker(hFile, encoding);

    let mut current_section = section;
    while !current_section.is_null() {
        let mut len: ::core::ffi::c_int = 0;

        if (*current_section).name[0] != 0 {
            len = (len as u64).wrapping_add(
                strlenW((*current_section).name.as_ptr()) as u64 + 4
            ) as ::core::ffi::c_int;
        }

        key = (*current_section).key;
        while !key.is_null() {
            len = (len as u64).wrapping_add(
                strlenW((*key).name.as_ptr()) as u64 + 2
            ) as ::core::ffi::c_int;
            if !(*key).value.is_null() {
                len = (len as u64).wrapping_add(
                    strlenW((*key).value) as u64 + 1
                ) as ::core::ffi::c_int;
            }
            key = (*key).next;
        }

        buffer = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as *mut WCHAR;
        if buffer.is_null() {
            return;
        }

        p = buffer;
        if (*current_section).name[0] != 0 {
            *p = '[' as WCHAR;
            p = p.offset(1);
            strcpyW(p, (*current_section).name.as_ptr());
            p = p.offset(strlenW(p) as isize);
            *p = ']' as WCHAR;
            p = p.offset(1);
            *p = '\r' as WCHAR;
            p = p.offset(1);
            *p = '\n' as WCHAR;
            p = p.offset(1);
        }

        key = (*current_section).key;
        while !key.is_null() {
            strcpyW(p, (*key).name.as_ptr());
            p = p.offset(strlenW(p) as isize);
            if !(*key).value.is_null() {
                *p = '=' as WCHAR;
                p = p.offset(1);
                strcpyW(p, (*key).value);
                p = p.offset(strlenW(p) as isize);
            }
            *p = '\r' as WCHAR;
            p = p.offset(1);
            *p = '\n' as WCHAR;
            p = p.offset(1);
            key = (*key).next;
        }

        PROFILE_WriteLine(hFile, buffer, len, encoding);
        HeapFree(GetProcessHeap(), 0, buffer as *mut core::ffi::c_void);
        current_section = (*current_section).next;
    }
}
unsafe fn PROFILE_Free(mut section: *mut PROFILESECTION) {
    let mut next_section: *mut PROFILESECTION = core::ptr::null_mut();
    let mut key: *mut PROFILEKEY = core::ptr::null_mut();
    let mut next_key: *mut PROFILEKEY = core::ptr::null_mut();

    while !section.is_null() {
        key = (*section).key;
        while !key.is_null() {
            next_key = (*key).next;
            HeapFree(GetProcessHeap(), 0, (*key).value as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, key as *mut core::ffi::c_void);
            key = next_key;
        }
        next_section = (*section).next;
        HeapFree(GetProcessHeap(), 0, section as *mut core::ffi::c_void);
        section = next_section;
    }
}
unsafe fn PROFILE_isspaceW(mut c: WCHAR) -> BOOL {
    return (isspaceW(c) || c == 0x1a) as BOOL;
}
unsafe fn PROFILE_DetectTextEncoding(
    buffer: *const ::core::ffi::c_void,
    len: *mut ::core::ffi::c_int,
) -> ENCODING {
    let mut flags: ::core::ffi::c_int =
        (IS_TEXT_UNICODE_SIGNATURE | IS_TEXT_UNICODE_REVERSE_SIGNATURE | IS_TEXT_UNICODE_ODD_LENGTH) as c_int;
    if *len as usize >= core::mem::size_of::<[::core::ffi::c_char; 3]>()
        && memcmp(
            buffer,
            &raw const bom_utf8 as *const [::core::ffi::c_char; 3] as *const ::core::ffi::c_void,
            core::mem::size_of::<[::core::ffi::c_char; 3]>(),
        ) == 0
    {
        *len = core::mem::size_of::<[::core::ffi::c_char; 3]>() as ::core::ffi::c_int;
        return ENCODING_UTF8;
    }
    RtlIsTextUnicode(buffer, *len, &mut flags);
    if flags & IS_TEXT_UNICODE_SIGNATURE as c_int != 0 {
        *len = core::mem::size_of::<WCHAR>() as ::core::ffi::c_int;
        return ENCODING_UTF16LE;
    }
    if flags & IS_TEXT_UNICODE_REVERSE_SIGNATURE as c_int != 0 {
        *len = core::mem::size_of::<WCHAR>() as ::core::ffi::c_int;
        return ENCODING_UTF16BE;
    }
    *len = 0;
    return ENCODING_ANSI;
}
unsafe fn PROFILE_Load(
    hFile: HANDLE,
    pEncoding: *mut ENCODING,
) -> *mut PROFILESECTION {
    let mut buffer_base: *mut ::core::ffi::c_void = core::ptr::null_mut();
    let mut pBuffer: *mut ::core::ffi::c_void = core::ptr::null_mut();
    let mut szFile: *mut WCHAR = core::ptr::null_mut();
    let mut szLineStart: *const WCHAR = core::ptr::null();
    let mut szLineEnd: *const WCHAR = core::ptr::null();
    let mut szValueStart: *const WCHAR = core::ptr::null();
    let mut szEnd: *const WCHAR = core::ptr::null();
    let mut next_line: *const WCHAR = core::ptr::null();
    let mut line: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut section: *mut PROFILESECTION = core::ptr::null_mut();
    let mut first_section: *mut PROFILESECTION = core::ptr::null_mut();
    let mut next_section: *mut *mut PROFILESECTION = core::ptr::null_mut();
    let mut key: *mut PROFILEKEY = core::ptr::null_mut();
    let mut prev_key: *mut PROFILEKEY = core::ptr::null_mut();
    let mut next_key: *mut *mut PROFILEKEY = core::ptr::null_mut();
    let mut dwFileSize: DWORD = 0;

    TRACE!(b"%p\n\0" as *const u8 as *const ::core::ffi::c_char, hFile);

    dwFileSize = GetFileSize(hFile, core::ptr::null_mut());
    if dwFileSize == INVALID_FILE_SIZE || dwFileSize == 0 {
        return core::ptr::null_mut();
    }

    buffer_base = HeapAlloc(GetProcessHeap(), 0, dwFileSize as usize) as *mut ::core::ffi::c_void;
    if buffer_base.is_null() {
        return core::ptr::null_mut();
    }

    if ReadFile(
        hFile,
        buffer_base,
        dwFileSize,
        &mut dwFileSize as *mut DWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        HeapFree(GetProcessHeap(), 0, buffer_base);
        WARN!(
            b"Error %d reading file\n\0" as *const u8 as *const ::core::ffi::c_char,
            GetLastError(),
        );
        return core::ptr::null_mut();
    }

    len = dwFileSize as ::core::ffi::c_int;
    *pEncoding = PROFILE_DetectTextEncoding(buffer_base, &mut len as *mut ::core::ffi::c_int);
    pBuffer = (buffer_base as *mut ::core::ffi::c_char).offset(len as isize) as *mut ::core::ffi::c_void;
    dwFileSize = (dwFileSize as ::core::ffi::c_ulong).wrapping_sub(len as ::core::ffi::c_ulong) as DWORD;

    match *pEncoding as ::core::ffi::c_uint {
        1 => {
            TRACE!(b"ANSI encoding\n\0" as *const u8 as *const ::core::ffi::c_char);

            len = MultiByteToWideChar(
                0, // CP_ACP
                0,
                pBuffer as LPCSTR,
                dwFileSize as ::core::ffi::c_int,
                core::ptr::null_mut(),
                0,
            );
            szFile = HeapAlloc(
                GetProcessHeap(),
                0,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            if szFile.is_null() {
                HeapFree(GetProcessHeap(), 0, buffer_base);
                return core::ptr::null_mut();
            }
            MultiByteToWideChar(
                0, // CP_ACP
                0,
                pBuffer as LPCSTR,
                dwFileSize as ::core::ffi::c_int,
                szFile as LPWSTR,
                len,
            );
            szEnd = szFile.offset(len as isize);
        }
        2 => {
            TRACE!(b"UTF8 encoding\n\0" as *const u8 as *const ::core::ffi::c_char);

            len = MultiByteToWideChar(
                65001, // CP_UTF8
                0,
                pBuffer as LPCSTR,
                dwFileSize as ::core::ffi::c_int,
                core::ptr::null_mut(),
                0,
            );
            szFile = HeapAlloc(
                GetProcessHeap(),
                0,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            ) as *mut WCHAR;
            if szFile.is_null() {
                HeapFree(GetProcessHeap(), 0, buffer_base);
                return core::ptr::null_mut();
            }
            MultiByteToWideChar(
                65001, // CP_UTF8
                0,
                pBuffer as LPCSTR,
                dwFileSize as ::core::ffi::c_int,
                szFile as LPWSTR,
                len,
            );
            szEnd = szFile.offset(len as isize);
        }
        3 => {
            TRACE!(
                b"UTF16 Little Endian encoding\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            szFile = pBuffer as *mut WCHAR;
            szEnd = (pBuffer as *mut ::core::ffi::c_char).offset(dwFileSize as isize) as *mut WCHAR;
        }
        4 => {
            TRACE!(
                b"UTF16 Big Endian encoding\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            szFile = pBuffer as *mut WCHAR;
            szEnd = (pBuffer as *mut ::core::ffi::c_char).offset(dwFileSize as isize) as *mut WCHAR;
            PROFILE_ByteSwapShortBuffer(
                szFile,
                (dwFileSize as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
            );
        }
        _ => {
            FIXME!(
                "encoding type %d not implemented\n",
                *pEncoding as ::core::ffi::c_uint,
            );
            HeapFree(GetProcessHeap(), 0, buffer_base);
            return core::ptr::null_mut();
        }
    }

    first_section = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<PROFILESECTION>(),
    ) as *mut PROFILESECTION;
    if first_section.is_null() {
        if szFile != pBuffer as *mut WCHAR {
            HeapFree(GetProcessHeap(), 0, szFile as *mut ::core::ffi::c_void);
        }
        HeapFree(GetProcessHeap(), 0, buffer_base);
        return core::ptr::null_mut();
    }
    *((*first_section).name.as_mut_ptr()).offset(0) = 0 as WCHAR;
    (*first_section).key = core::ptr::null_mut();
    (*first_section).next = core::ptr::null_mut();
    next_section = &mut (*first_section).next as *mut *mut PROFILESECTION;
    next_key = &mut (*first_section).key as *mut *mut PROFILEKEY;
    prev_key = core::ptr::null_mut();
    next_line = szFile;

    // Every length below is a count of WCHARs, as the C's `ptr - ptr` is. The translation had
    // them as `(a as isize).wrapping_sub(b as isize)`, i.e. BYTES: every memchrW scanned twice
    // the line (past the end of the file buffer) and every section/key/value name was copied
    // at twice its length. GetPrivateProfileStringW on the livecd's unattend.inf then never
    // returned, which stalled setup's InstallLiveCD before it registered anything.
    while next_line < szEnd {
        szLineStart = next_line;
        next_line = memchrW(
            szLineStart,
            '\n' as i32 as WCHAR,
            szEnd.offset_from(szLineStart) as usize,
        );
        if next_line.is_null() {
            next_line = memchrW(
                szLineStart,
                '\r' as i32 as WCHAR,
                szEnd.offset_from(szLineStart) as usize,
            );
        }
        if next_line.is_null() {
            next_line = szEnd;
        } else {
            next_line = next_line.offset(1);
        }
        szLineEnd = next_line;

        line += 1;

        while szLineStart < szLineEnd && PROFILE_isspaceW(*szLineStart) != 0 {
            szLineStart = szLineStart.offset(1);
        }
        while szLineEnd > szLineStart && PROFILE_isspaceW(*szLineEnd.offset(-1)) != 0 {
            szLineEnd = szLineEnd.offset(-1);
        }

        if szLineStart >= szLineEnd {
            continue;
        }

        if *szLineStart as ::core::ffi::c_int == '[' as i32 {
            let mut szSectionEnd: *const WCHAR = core::ptr::null();
            szSectionEnd = memrchrW(
                szLineStart,
                ']' as i32 as WCHAR,
                szLineEnd.offset_from(szLineStart) as usize,
            );
            if szSectionEnd.is_null() {
                WARN!(
                    b"Invalid section header at line %d: %s\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    line,
                    debugstr_wn(
                        szLineStart,
                        szLineEnd.offset_from(szLineStart) as ::core::ffi::c_int,
                    ),
                );
            } else {
                szLineStart = szLineStart.offset(1);
                len = szSectionEnd.offset_from(szLineStart) as ::core::ffi::c_int;
                section = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    core::mem::size_of::<PROFILESECTION>()
                        .wrapping_add((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())),
                ) as *mut PROFILESECTION;
                if section.is_null() {
                    break;
                }
                memcpy(
                    (*section).name.as_mut_ptr() as *mut ::core::ffi::c_void,
                    szLineStart as *const ::core::ffi::c_void,
                    (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                *((*section).name.as_mut_ptr()).offset(len as isize) = '\0' as i32 as WCHAR;
                (*section).key = core::ptr::null_mut();
                (*section).next = core::ptr::null_mut();
                *next_section = section;
                next_section = &mut (*section).next as *mut *mut PROFILESECTION;
                next_key = &mut (*section).key as *mut *mut PROFILEKEY;
                prev_key = core::ptr::null_mut();

                TRACE!(
                    b"New section: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    debugstr_w((*section).name.as_mut_ptr()),
                );

                continue;
            }
        }

        len = szLineEnd.offset_from(szLineStart) as ::core::ffi::c_int;
        szValueStart = memchrW(
            szLineStart,
            '=' as i32 as WCHAR,
            szLineEnd.offset_from(szLineStart) as usize,
        );
        if !szValueStart.is_null() {
            let mut szNameEnd: *const WCHAR = szValueStart;
            while szNameEnd > szLineStart && PROFILE_isspaceW(*szNameEnd.offset(-1)) != 0 {
                szNameEnd = szNameEnd.offset(-1);
            }
            len = szNameEnd.offset_from(szLineStart) as ::core::ffi::c_int;
            szValueStart = szValueStart.offset(1);
            while szValueStart < szLineEnd && PROFILE_isspaceW(*szValueStart) != 0 {
                szValueStart = szValueStart.offset(1);
            }
        }

        if len != 0
            || prev_key.is_null()
            || *((*prev_key).name.as_mut_ptr()) as ::core::ffi::c_int != 0
        {
            key = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<PROFILEKEY>()
                    .wrapping_add((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())),
            ) as *mut PROFILEKEY;
            if key.is_null() {
                break;
            }
            memcpy(
                (*key).name.as_mut_ptr() as *mut ::core::ffi::c_void,
                szLineStart as *const ::core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            *((*key).name.as_mut_ptr()).offset(len as isize) = '\0' as i32 as WCHAR;
            if !szValueStart.is_null() {
                len = szLineEnd.offset_from(szValueStart) as ::core::ffi::c_int;
                (*key).value = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    ((len + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                ) as *mut WCHAR;
                memcpy(
                    (*key).value as *mut ::core::ffi::c_void,
                    szValueStart as *const ::core::ffi::c_void,
                    (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                *(*key).value.offset(len as isize) = '\0' as i32 as WCHAR;
            } else {
                (*key).value = core::ptr::null_mut();
            }

            (*key).next = core::ptr::null_mut();
            *next_key = key;
            next_key = &mut (*key).next as *mut *mut PROFILEKEY;
            prev_key = key;

            TRACE!(
                b"New key: name=%s, value=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w((*key).name.as_mut_ptr()),
                if !(*key).value.is_null() {
                    debugstr_w((*key).value)
                } else {
                    b"(none)\0" as *const u8 as *const WCHAR
                },
            );
        }
    }

    if szFile != pBuffer as *mut WCHAR {
        HeapFree(GetProcessHeap(), 0, szFile as *mut ::core::ffi::c_void);
    }
    HeapFree(GetProcessHeap(), 0, buffer_base);
    return first_section;
}
unsafe fn PROFILE_DeleteSection(
    mut section: *mut *mut PROFILESECTION,
    mut name: LPCWSTR,
) -> BOOL {
    while !(*section).is_null() {
        if strcmpiW((**section).name.as_ptr(), name) == 0 {
            let mut to_del: *mut PROFILESECTION = *section;
            *section = (*to_del).next;
            (*to_del).next = core::ptr::null_mut();
            PROFILE_Free(to_del);
            return 1;
        }
        section = &mut (**section).next as *mut *mut PROFILESECTION;
    }
    return 0;
}
unsafe fn PROFILE_DeleteKey(
    mut section: *mut *mut PROFILESECTION,
    section_name: LPCWSTR,
    key_name: LPCWSTR,
) -> BOOL {
    while !(*section).is_null() {
        if strcmpiW((**section).name.as_ptr(), section_name) == 0 {
            let mut key: *mut *mut PROFILEKEY = &mut (**section).key;
            while !(*key).is_null() {
                if strcmpiW((**key).name.as_ptr(), key_name) == 0 {
                    let to_del: *mut PROFILEKEY = *key;
                    *key = (*to_del).next;
                    HeapFree(GetProcessHeap(), 0, (*to_del).value as LPVOID);
                    HeapFree(GetProcessHeap(), 0, to_del as LPVOID);
                    return 1;
                }
                key = &mut (**key).next;
            }
        }
        section = &mut (**section).next;
    }
    0
}
unsafe fn PROFILE_DeleteAllKeys(section_name: LPCWSTR) {
    let mut section: *mut *mut PROFILESECTION = &raw mut (*MRUProfile[0]).section;
    while !(*section).is_null() {
        if strcmpiW((*(*section)).name.as_ptr(), section_name) == 0 {
            let mut key: *mut *mut PROFILEKEY = &raw mut (*(*section)).key;
            while !(*key).is_null() {
                let to_del: *mut PROFILEKEY = *key;
                *key = (*to_del).next;
                HeapFree(GetProcessHeap(), 0, (*to_del).value as LPVOID);
                HeapFree(GetProcessHeap(), 0, to_del as LPVOID);
                (*MRUProfile[0]).changed = 1i32;
            }
        }
        section = &raw mut (*(*section)).next as *mut *mut PROFILESECTION;
    }
}
unsafe fn PROFILE_Find(
    mut section: *mut *mut PROFILESECTION,
    mut section_name: LPCWSTR,
    mut key_name: LPCWSTR,
    mut create: BOOL,
    mut create_always: BOOL,
) -> *mut PROFILEKEY {
    let mut p: LPCWSTR = core::ptr::null();
    let mut seclen: c_int = 0;
    let mut keylen: c_int = 0;

    while PROFILE_isspaceW(*section_name) != 0 {
        section_name = section_name.offset(1);
    }
    if *section_name != 0 {
        p = section_name.offset(strlenW(section_name) as isize).offset(-1);
        while p > section_name && PROFILE_isspaceW(*p) != 0 {
            p = p.offset(-1);
        }
        seclen = (p.offset_from(section_name) + 1) as c_int;
    }

    while PROFILE_isspaceW(*key_name) != 0 {
        key_name = key_name.offset(1);
    }
    if *key_name != 0 {
        p = key_name.offset(strlenW(key_name) as isize).offset(-1);
        while p > key_name && PROFILE_isspaceW(*p) != 0 {
            p = p.offset(-1);
        }
        keylen = (p.offset_from(key_name) + 1) as c_int;
    }

    while !(*section).is_null() {
        if strncmpiW((**section).name.as_ptr(), section_name, seclen as usize) == 0
            && *((**section).name.as_ptr()).offset(seclen as isize) == 0
        {
            let mut key: *mut *mut PROFILEKEY = &raw mut (**section).key;

            while !(*key).is_null() {
                if create_always == 0 {
                    if strncmpiW((**key).name.as_ptr(), key_name, keylen as usize) == 0
                        && *((**key).name.as_ptr()).offset(keylen as isize) == 0
                    {
                        return *key;
                    }
                }
                key = &raw mut (**key).next as *mut *mut PROFILEKEY;
            }
            if create == 0 {
                return core::ptr::null_mut();
            }
            *key = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<PROFILEKEY>()
                    .wrapping_add(strlenW(key_name).wrapping_mul(core::mem::size_of::<WCHAR>())),
            ) as *mut PROFILEKEY;
            if (*key).is_null() {
                return core::ptr::null_mut();
            }
            strcpyW((**key).name.as_mut_ptr(), key_name);
            (**key).value = core::ptr::null_mut();
            (**key).next = core::ptr::null_mut();
            return *key;
        }
        section = &raw mut (**section).next as *mut *mut PROFILESECTION;
    }
    if create == 0 {
        return core::ptr::null_mut();
    }
    *section = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<PROFILESECTION>()
            .wrapping_add(strlenW(section_name).wrapping_mul(core::mem::size_of::<WCHAR>())),
    ) as *mut PROFILESECTION;
    if (*section).is_null() {
        return core::ptr::null_mut();
    }
    strcpyW((**section).name.as_mut_ptr(), section_name);
    (**section).next = core::ptr::null_mut();
    (**section).key = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<PROFILEKEY>()
            .wrapping_add(strlenW(key_name).wrapping_mul(core::mem::size_of::<WCHAR>())),
    ) as *mut PROFILEKEY;
    if (**section).key.is_null() {
        HeapFree(GetProcessHeap(), 0, *section as *mut c_void);
        return core::ptr::null_mut();
    }
    strcpyW((*(**section).key).name.as_mut_ptr(), key_name);
    (*(**section).key).value = core::ptr::null_mut();
    (*(**section).key).next = core::ptr::null_mut();
    return (**section).key;
}
unsafe fn PROFILE_FlushFile() -> BOOL {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut LastWriteTime: FILETIME = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

    if MRUProfile[0].is_null()
    {
        WARN!(b"No current profile!\n\0" as *const u8 as LPCSTR);
        return FALSE;
    }

    if (*MRUProfile[0]).changed == 0
    {
        return TRUE;
    }

    hFile = CreateFileW(
        (*MRUProfile[0]).filename,
        GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE
    {
        WARN!(
            b"could not save profile file %s (error was %d)\n\0" as *const u8 as LPCSTR,
            debugstr_w((*MRUProfile[0]).filename),
            GetLastError(),
        );
        return FALSE;
    }

    TRACE!(
        b"Saving %s\n\0" as *const u8 as LPCSTR,
        debugstr_w((*MRUProfile[0]).filename),
    );

    PROFILE_Save(hFile, (*MRUProfile[0]).section, (*MRUProfile[0]).encoding);

    if GetFileTime(
        hFile,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut LastWriteTime,
    ) != 0
    {
        (*MRUProfile[0]).LastWriteTime = LastWriteTime;
    }

    CloseHandle(hFile);
    (*MRUProfile[0]).changed = FALSE;
    return TRUE;
}
unsafe fn PROFILE_ReleaseFile() {
    PROFILE_FlushFile();
    PROFILE_Free((*MRUProfile[0]).section);
    HeapFree(
        GetProcessHeap(),
        0,
        (*MRUProfile[0]).filename as LPVOID,
    );
    (*MRUProfile[0]).changed = 0;
    (*MRUProfile[0]).section = core::ptr::null_mut();
    (*MRUProfile[0]).filename = core::ptr::null_mut();
    (*MRUProfile[0]).encoding = ENCODING_ANSI;
    ZeroMemory(
        &raw mut (*MRUProfile[0]).LastWriteTime as *mut ::core::ffi::c_void,
        core::mem::size_of::<FILETIME>(),
    );
}
unsafe fn is_not_current(ft: *mut FILETIME) -> BOOL {
    let mut Now: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut ftll: LONGLONG = 0;
    let mut nowll: LONGLONG = 0;
    GetSystemTimeAsFileTime(&mut Now as *mut FILETIME);
    ftll = (((*ft).dwHighDateTime as LONGLONG) << 32) + (*ft).dwLowDateTime as LONGLONG;
    nowll = ((Now.dwHighDateTime as LONGLONG) << 32) + Now.dwLowDateTime as LONGLONG;
    TRACE!(
        b"%08x;%08x\n\0".as_ptr() as LPCSTR,
        (ftll as u32).wrapping_add(21000000),
        nowll as u32,
    );
    return (ftll + 21000000 < nowll) as BOOL;
}
unsafe fn PROFILE_Open(
    mut filename: LPCWSTR,
    mut write_access: BOOL,
) -> BOOL {
    let mut buffer: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut LastWriteTime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut tempProfile: *mut PROFILE = core::ptr::null_mut();

    ZeroMemory(
        &mut LastWriteTime as *mut FILETIME as *mut core::ffi::c_void,
        core::mem::size_of::<FILETIME>(),
    );

    if MRUProfile[0].is_null() {
        i = 0;
        while i < N_CACHED_PROFILES {
            MRUProfile[i as usize] = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<PROFILE>(),
            ) as *mut PROFILE;
            if MRUProfile[i as usize].is_null() {
                break;
            }
            (*MRUProfile[i as usize]).changed = 0;
            (*MRUProfile[i as usize]).section = core::ptr::null_mut();
            (*MRUProfile[i as usize]).filename = core::ptr::null_mut();
            (*MRUProfile[i as usize]).encoding = ENCODING_ANSI;
            ZeroMemory(
                &mut (*MRUProfile[i as usize]).LastWriteTime as *mut FILETIME as *mut core::ffi::c_void,
                core::mem::size_of::<FILETIME>(),
            );
            i += 1;
        }
    }

    if filename.is_null() {
        filename = &raw const wininiW as *const WCHAR as LPCWSTR;
    }

    if (RtlDetermineDosPathNameType_U(filename as PCWSTR) as i32 == RtlPathTypeRelative as i32)
        && strchrW(filename, '\\' as i32 as WCHAR).is_null()
        && strchrW(filename, '/' as i32 as WCHAR).is_null()
    {
        static mut wszSeparator: [WCHAR; 2] = ['\\' as WCHAR, 0];
        let mut windirW: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        GetWindowsDirectoryW(windirW.as_mut_ptr(), MAX_PATH);
        strcpyW(buffer.as_mut_ptr(), windirW.as_mut_ptr());
        strcatW(buffer.as_mut_ptr(), (*(&raw mut wszSeparator)).as_mut_ptr());
        strcatW(buffer.as_mut_ptr(), filename);
    } else {
        let mut dummy: LPWSTR = core::ptr::null_mut();
        GetFullPathNameW(
            filename,
            (core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as u32)
                / core::mem::size_of::<WCHAR>() as u32,
            buffer.as_mut_ptr(),
            &mut dummy,
        );
    }

    TRACE!(
        b"path: %s\n\0".as_ptr() as LPCSTR,
        debugstr_w(buffer.as_mut_ptr()),
    );

    hFile = CreateFileW(
        buffer.as_mut_ptr(),
        (GENERIC_READ
            | if write_access != 0 {
                GENERIC_WRITE
            } else {
                0
            }) as u32,
        (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u32,
        core::ptr::null_mut(),
        OPEN_EXISTING as u32,
        FILE_ATTRIBUTE_NORMAL as u32,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE && GetLastError() != ERROR_FILE_NOT_FOUND {
        WARN!(
            b"Error %d opening file %s\n\0".as_ptr() as LPCSTR,
            GetLastError(),
            debugstr_w(buffer.as_mut_ptr()),
        );
        return 0;
    }

    i = 0;
    while i < N_CACHED_PROFILES {
        if !(*MRUProfile[i as usize]).filename.is_null()
            && strcmpiW(buffer.as_mut_ptr(), (*MRUProfile[i as usize]).filename) == 0
        {
            TRACE!(
                b"MRU Filename: %s, new filename: %s\n\0".as_ptr() as LPCSTR,
                debugstr_w((*MRUProfile[i as usize]).filename),
                debugstr_w(buffer.as_mut_ptr()),
            );
            if i != 0 {
                PROFILE_FlushFile();
                tempProfile = MRUProfile[i as usize];
                j = i;
                while j > 0 {
                    MRUProfile[j as usize] = MRUProfile[(j - 1) as usize];
                    j -= 1;
                }
                MRUProfile[0] = tempProfile;
            }

            if hFile != INVALID_HANDLE_VALUE {
                GetFileTime(
                    hFile,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut LastWriteTime,
                );
                if memcmp(
                    &(*MRUProfile[0]).LastWriteTime as *const FILETIME as *const core::ffi::c_void,
                    &LastWriteTime as *const FILETIME as *const core::ffi::c_void,
                    core::mem::size_of::<FILETIME>(),
                ) == 0
                    && is_not_current(&mut LastWriteTime) != 0
                {
                    TRACE!(
                        b"(%s): already opened (mru=%d)\n\0".as_ptr() as LPCSTR,
                        debugstr_w(buffer.as_mut_ptr()),
                        i,
                    );
                } else {
                    TRACE!(
                        b"(%s): already opened, needs refreshing (mru=%d)\n\0".as_ptr() as LPCSTR,
                        debugstr_w(buffer.as_mut_ptr()),
                        i,
                    );
                    PROFILE_Free((*MRUProfile[0]).section);
                    (*MRUProfile[0]).section = PROFILE_Load(hFile, &mut (*MRUProfile[0]).encoding);
                    (*MRUProfile[0]).LastWriteTime = LastWriteTime;
                }
                CloseHandle(hFile);
                return 1;
            } else {
                TRACE!(
                    b"(%s): already opened, not yet created (mru=%d)\n\0".as_ptr() as LPCSTR,
                    debugstr_w(buffer.as_mut_ptr()),
                    i,
                );
            }
            // profile.c returns TRUE here too: the profile is cached, just not on disk yet.
            // Falling through re-created it and dropped every unsaved key written so far.
            return 1;
        }
        i += 1;
    }

    PROFILE_FlushFile();

    if i == N_CACHED_PROFILES {
        tempProfile = MRUProfile[(N_CACHED_PROFILES - 1) as usize];
        i = N_CACHED_PROFILES - 1;
        while i > 0 {
            MRUProfile[i as usize] = MRUProfile[(i - 1) as usize];
            i -= 1;
        }
        MRUProfile[0] = tempProfile;
    }


    if !(*MRUProfile[0]).filename.is_null() {
        PROFILE_ReleaseFile();
    }

    (*MRUProfile[0]).filename = HeapAlloc(
        GetProcessHeap(),
        0,
        ((strlenW(buffer.as_mut_ptr()) + 1) * core::mem::size_of::<WCHAR>()) as usize,
    ) as *mut WCHAR;
    strcpyW((*MRUProfile[0]).filename, buffer.as_mut_ptr());

    if hFile != INVALID_HANDLE_VALUE {
        (*MRUProfile[0]).section = PROFILE_Load(hFile, &mut (*MRUProfile[0]).encoding);
        GetFileTime(
            hFile,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut (*MRUProfile[0]).LastWriteTime,
        );
        CloseHandle(hFile);
    } else {
        WARN!(
            b"profile file %s not found\n\0".as_ptr() as LPCSTR,
            debugstr_w(buffer.as_mut_ptr()),
        );
    }

    return 1;
}

unsafe extern "C" fn PROFILE_GetSection(
    mut section: *mut PROFILESECTION,
    section_name: LPCWSTR,
    mut buffer: LPWSTR,
    mut len: DWORD,
    return_values: BOOL,
) -> INT {
    let mut key: *mut PROFILEKEY;

    if buffer.is_null() {
        return 0;
    }

    TRACE!("%s,%p,%u\n", debugstr_w(section_name), buffer, len);

    while !section.is_null() {
        if strcmpiW((*section).name.as_ptr(), section_name) == 0 {
            let oldlen: UINT = len;
            key = (*section).key;
            while !key.is_null() {
                if len <= 2 {
                    break;
                }
                if *(*key).name.as_ptr() == 0 {
                    key = (*key).next;
                    continue;
                }
                if IS_ENTRY_COMMENT((*key).name.as_ptr()) != 0 {
                    key = (*key).next;
                    continue;
                }
                if return_values == 0 && (*key).value.is_null() {
                    key = (*key).next;
                    continue;
                }
                PROFILE_CopyEntry(buffer, (*key).name.as_ptr(), (len - 1) as ::core::ffi::c_int, 0);
                len -= (strlenW(buffer) + 1) as DWORD;
                buffer = buffer.offset(strlenW(buffer) as isize + 1);
                if len < 2 {
                    break;
                }
                if return_values != 0 && !(*key).value.is_null() {
                    *buffer.offset(-1) = '=' as u16;
                    PROFILE_CopyEntry(buffer, (*key).value, (len - 1) as ::core::ffi::c_int, 0);
                    len -= (strlenW(buffer) + 1) as DWORD;
                    buffer = buffer.offset(strlenW(buffer) as isize + 1);
                }
                key = (*key).next;
            }
            *buffer = 0;
            if len <= 1 {
                *buffer.offset(-1) = 0;
                return (oldlen - 2) as INT;
            }
            return (oldlen - len) as INT;
        }
        section = (*section).next;
    }
    *buffer.offset(0) = 0;
    *buffer.offset(1) = 0;
    return 0;
}
unsafe fn PROFILE_GetSectionNames(mut buffer: LPWSTR, mut len: UINT) -> INT {
    let mut buf: LPWSTR;
    let mut buflen: UINT;
    let mut tmplen: UINT;
    let mut section: *mut PROFILESECTION;

    TRACE!(b"(%p, %d)\n\0" as *const u8 as LPCSTR, buffer, len);

    if buffer.is_null() || len == 0 {
        return 0;
    }
    if len == 1 {
        *buffer = '\0' as WCHAR;
        return 0;
    }

    buflen = len.wrapping_sub(1);
    buf = buffer;
    section = (*MRUProfile[0]).section;
    while !section.is_null() {
        if (*section).name[0] != 0 {
            tmplen = strlenW((*section).name.as_ptr()).wrapping_add(1) as UINT;
            if tmplen >= buflen {
                if buflen > 0 {
                    memcpy(
                        buf as *mut ::core::ffi::c_void,
                        (*section).name.as_ptr() as *const ::core::ffi::c_void,
                        (buflen.wrapping_sub(1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                    );
                    buf = buf.offset(buflen.wrapping_sub(1) as isize);
                    let fresh0 = buf;
                    buf = buf.offset(1);
                    *fresh0 = '\0' as WCHAR;
                }
                *buf = '\0' as WCHAR;
                return len.wrapping_sub(2) as INT;
            }
            memcpy(
                buf as *mut ::core::ffi::c_void,
                (*section).name.as_ptr() as *const ::core::ffi::c_void,
                (tmplen as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            buf = buf.offset(tmplen as isize);
            buflen = buflen.wrapping_sub(tmplen);
        }
        section = (*section).next;
    }
    *buf = '\0' as WCHAR;
    buf.offset_from(buffer) as INT
}

unsafe extern "C" fn PROFILE_GetString(
    section: LPCWSTR,
    key_name: LPCWSTR,
    mut def_val: LPCWSTR,
    buffer: LPWSTR,
    len: DWORD,
) -> INT {
    let mut key: *mut PROFILEKEY = core::ptr::null_mut();
    static empty_strW: [WCHAR; 1] = [0];

    if buffer.is_null() || len == 0 {
        return 0;
    }

    if def_val.is_null() {
        def_val = empty_strW.as_ptr();
    }
    if !key_name.is_null() {
        key = PROFILE_Find(
            &mut (*MRUProfile[0]).section,
            section,
            key_name,
            0,
            0,
        );
        PROFILE_CopyEntry(
            buffer,
            if !key.is_null() && !(*key).value.is_null() {
                (*key).value
            } else {
                def_val
            },
            len as ::core::ffi::c_int,
            1,
        );
        TRACE!(
            b"(%s,%s,%s): returning %s\n\0".as_ptr() as *const i8,
            debugstr_w(section),
            debugstr_w(key_name),
            debugstr_w(def_val),
            debugstr_w(buffer),
        );
        return strlenW(buffer) as INT;
    }
    /* no "else" here ! */
    if !section.is_null() {
        let mut ret: INT = PROFILE_GetSection(
            (*MRUProfile[0]).section,
            section,
            buffer,
            len,
            0,
        );
        if *buffer.offset(0) == 0 {
            /* no luck -> def_val */
            PROFILE_CopyEntry(buffer, def_val, len as ::core::ffi::c_int, 1);
            ret = strlenW(buffer) as INT;
        }
        return ret;
    }
    *buffer.offset(0) = 0 as WCHAR;
    return 0;
}
unsafe fn PROFILE_SetString(
    section_name: LPCWSTR,
    key_name: LPCWSTR,
    mut value: LPCWSTR,
    create_always: BOOL,
) -> BOOL {
    if key_name.is_null() {
        TRACE!(b"(%s)\n\0" as *const u8 as LPCSTR, debugstr_w(section_name));
        (*MRUProfile[0]).changed |= PROFILE_DeleteSection(
            &raw mut (*MRUProfile[0]).section,
            section_name,
        );
        return TRUE;
    } else if value.is_null() {
        TRACE!(
            b"(%s,%s)\n\0" as *const u8 as LPCSTR,
            debugstr_w(section_name),
            debugstr_w(key_name),
        );
        (*MRUProfile[0]).changed |= PROFILE_DeleteKey(
            &raw mut (*MRUProfile[0]).section,
            section_name,
            key_name,
        );
        return TRUE;
    } else {
        let key: *mut PROFILEKEY = PROFILE_Find(
            &raw mut (*MRUProfile[0]).section,
            section_name,
            key_name,
            TRUE,
            create_always,
        );
        TRACE!(
            b"(%s,%s,%s):\n\0" as *const u8 as LPCSTR,
            debugstr_w(section_name),
            debugstr_w(key_name),
            debugstr_w(value),
        );
        if key.is_null() {
            return FALSE;
        }
        while PROFILE_isspaceW(*value) != 0 {
            value = value.offset(1);
        }
        if !(*key).value.is_null() {
            if strcmpW((*key).value, value) == 0 {
                TRACE!(b"  no change needed\n\0" as *const u8 as LPCSTR);
                return TRUE;
            }
            TRACE!(
                b"  replacing %s\n\0" as *const u8 as LPCSTR,
                debugstr_w((*key).value),
            );
            HeapFree(GetProcessHeap(), 0, (*key).value as LPVOID);
        } else {
            TRACE!(b"  creating key\n\0" as *const u8 as LPCSTR);
        }
        (*key).value = HeapAlloc(
            GetProcessHeap(),
            0,
            ((strlenW(value) + 1) * core::mem::size_of::<WCHAR>()) as usize,
        ) as LPWSTR;
        strcpyW((*key).value, value);
        (*MRUProfile[0]).changed = TRUE;
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetProfileIntA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: INT,
) -> UINT {
    return GetPrivateProfileIntA(section, entry, def_val, b"win.ini\0".as_ptr() as LPCSTR);
}

#[no_mangle]
pub unsafe extern "system" fn GetProfileIntW(
    section: LPCWSTR,
    entry: LPCWSTR,
    def_val: INT,
) -> UINT {
    return GetPrivateProfileIntW(section, entry, def_val, (*(&raw const wininiW)).as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn WriteProfileStringA(
    section: LPCSTR,
    entry: LPCSTR,
    string: LPCSTR,
) -> BOOL {
    return WritePrivateProfileStringA(section, entry, string, b"win.ini\0".as_ptr() as LPCSTR);
}

#[no_mangle]
pub unsafe extern "system" fn WriteProfileStringW(
    section: LPCWSTR,
    entry: LPCWSTR,
    string: LPCWSTR,
) -> BOOL {
    return WritePrivateProfileStringW(section, entry, string, (*(&raw const wininiW)).as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileIntW(
    section: LPCWSTR,
    entry: LPCWSTR,
    def_val: INT,
    filename: LPCWSTR,
) -> UINT {
    let mut buffer: [WCHAR; 30] = [0; 30];
    let mut bufferW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut result: ULONG = 0;

    if GetPrivateProfileStringW(
        section,
        entry,
        &raw const emptystringW as *const WCHAR as LPCWSTR,
        &mut buffer as *mut [WCHAR; 30] as LPWSTR,
        (core::mem::size_of::<[WCHAR; 30]>() as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        filename,
    ) == 0
    {
        return def_val as UINT;
    }

    if buffer[0] == 0 {
        return def_val as UINT;
    }

    RtlInitUnicodeString(&mut bufferW, &mut buffer as *mut [WCHAR; 30] as *mut WCHAR as PCWSTR);
    RtlUnicodeStringToInteger(&bufferW as *const _UNICODE_STRING as PCUNICODE_STRING, 0, &mut result);
    return result as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileIntA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: INT,
    filename: LPCSTR,
) -> UINT {
    let mut entryW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut res: UINT = 0;

    if !entry.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut entryW, entry as PCSZ);
    } else {
        entryW.Buffer = core::ptr::null_mut();
    }

    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as PCSZ);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    if !section.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut sectionW, section as PCSZ);
    } else {
        sectionW.Buffer = core::ptr::null_mut();
    }

    res = GetPrivateProfileIntW(
        sectionW.Buffer as LPCWSTR,
        entryW.Buffer as LPCWSTR,
        def_val,
        filenameW.Buffer as LPCWSTR,
    );

    RtlFreeUnicodeString(&raw mut sectionW);
    RtlFreeUnicodeString(&raw mut filenameW);
    RtlFreeUnicodeString(&raw mut entryW);

    return res;
}
#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileSectionW(
    section: LPCWSTR,
    buffer: LPWSTR,
    len: DWORD,
    filename: LPCWSTR,
) -> DWORD {
    let mut ret: i32 = 0;

    if section.is_null() || buffer.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    TRACE!(
        b"(%s, %p, %d, %s)\n\0".as_ptr() as LPCSTR,
        debugstr_w(section),
        buffer,
        len,
        debugstr_w(filename),
    );

    RtlEnterCriticalSection(&raw mut PROFILE_CritSect);

    if PROFILE_Open(filename, 0) != 0 {
        ret = PROFILE_GetSection(
            (*MRUProfile[0]).section,
            section,
            buffer,
            len,
            1,
        ) as i32;
    }

    RtlLeaveCriticalSection(&raw mut PROFILE_CritSect);

    ret as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileSectionA(
    section: LPCSTR,
    buffer: LPSTR,
    len: DWORD,
    filename: LPCSTR,
) -> DWORD {
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bufferW: LPWSTR = core::ptr::null_mut();
    let mut retW: INT = 0;
    let mut ret: INT = 0;

    if section.is_null() || buffer.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    bufferW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize).wrapping_mul(2).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as LPWSTR;

    RtlCreateUnicodeStringFromAsciiz(&raw mut sectionW, section as PCSZ);
    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as PCSZ);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    retW = GetPrivateProfileSectionW(
        sectionW.Buffer,
        bufferW,
        len.wrapping_mul(2),
        filenameW.Buffer,
    ) as INT;

    if retW != 0 {
        if retW as DWORD == len.wrapping_mul(2).wrapping_sub(2) {
            retW += 1;
        }
        ret = WideCharToMultiByte(
            CP_ACP,
            0,
            bufferW,
            retW + 1,
            buffer,
            len as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as INT;
        if ret == 0 || ret as DWORD == len {
            ret = len.wrapping_sub(2) as INT;
            *buffer.offset((len.wrapping_sub(2)) as isize) = 0;
            *buffer.offset((len.wrapping_sub(1)) as isize) = 0;
        } else {
            ret -= 1;
        }
    } else {
        *buffer.offset(0) = 0;
        *buffer.offset(1) = 0;
    }

    RtlFreeUnicodeString(&raw mut sectionW);
    RtlFreeUnicodeString(&raw mut filenameW);
    HeapFree(GetProcessHeap(), 0, bufferW as LPVOID);
    return ret as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetProfileSectionW(
    section: LPCWSTR,
    buffer: LPWSTR,
    len: DWORD,
) -> DWORD {
    GetPrivateProfileSectionW(section, buffer, len, (*(&raw const wininiW)).as_ptr())
}

// ============================================================================
// Hand-translated from wine/profile.c (__REACTOS__ branch) — the batch run
// dropped these 5 exports; faithful ports so the cohab flip stays symbol-complete.
// ============================================================================

/// GetPrivateProfileStringW (KERNEL32.@) — profile.c
#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileStringW(
    section: LPCWSTR,
    entry: LPCWSTR,
    mut def_val: LPCWSTR,
    buffer: LPWSTR,
    len: DWORD,
    filename: LPCWSTR,
) -> DWORD {
    let mut ret: INT;
    let mut defval_tmp: LPWSTR = core::ptr::null_mut();

    // strip any trailing ' ' of def_val.
    if !def_val.is_null() {
        let mut p: LPCWSTR = def_val.offset(strlenW(def_val) as isize - 1);
        while p > def_val && *p == ' ' as WCHAR {
            p = p.offset(-1);
        }
        if p >= def_val {
            let vlen: INT = (p.offset_from(def_val) as INT) + 1;
            defval_tmp = HeapAlloc(
                GetProcessHeap(),
                0,
                ((vlen + 1) as usize) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;
            memcpy(
                defval_tmp as *mut c_void,
                def_val as *const c_void,
                (vlen as usize) * core::mem::size_of::<WCHAR>(),
            );
            *defval_tmp.offset(vlen as isize) = 0;
            def_val = defval_tmp;
        }
    }

    RtlEnterCriticalSection(&raw mut PROFILE_CritSect);

    if PROFILE_Open(filename, 0) != 0 {
        if section.is_null() {
            ret = PROFILE_GetSectionNames(buffer, len as UINT);
        } else {
            // PROFILE_GetString can handle the 'entry == NULL' case
            ret = PROFILE_GetString(section, entry, def_val, buffer, len);
        }
    } else if !buffer.is_null() && !def_val.is_null() {
        lstrcpynW(buffer, def_val, len as INT);
        ret = strlenW(buffer) as INT;
    } else {
        ret = 0;
    }

    RtlLeaveCriticalSection(&raw mut PROFILE_CritSect);

    HeapFree(GetProcessHeap(), 0, defval_tmp as LPVOID);

    return ret as DWORD;
}

/// GetPrivateProfileStringA (KERNEL32.@) — profile.c
#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileStringA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: LPCSTR,
    buffer: LPSTR,
    len: DWORD,
    filename: LPCSTR,
) -> DWORD {
    let mut sectionW: _UNICODE_STRING = core::mem::zeroed();
    let mut entryW: _UNICODE_STRING = core::mem::zeroed();
    let mut def_valW: _UNICODE_STRING = core::mem::zeroed();
    let mut filenameW: _UNICODE_STRING = core::mem::zeroed();
    let bufferW: LPWSTR;
    let retW: INT;
    let mut ret: INT = 0;

    bufferW = if !buffer.is_null() {
        HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR
    } else {
        core::ptr::null_mut()
    };
    if !section.is_null() { RtlCreateUnicodeStringFromAsciiz(&raw mut sectionW, section as PCSZ); } else { sectionW.Buffer = core::ptr::null_mut(); }
    if !entry.is_null() { RtlCreateUnicodeStringFromAsciiz(&raw mut entryW, entry as PCSZ); } else { entryW.Buffer = core::ptr::null_mut(); }
    if !def_val.is_null() { RtlCreateUnicodeStringFromAsciiz(&raw mut def_valW, def_val as PCSZ); } else { def_valW.Buffer = core::ptr::null_mut(); }
    if !filename.is_null() { RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as PCSZ); } else { filenameW.Buffer = core::ptr::null_mut(); }

    retW = GetPrivateProfileStringW(
        sectionW.Buffer, entryW.Buffer, def_valW.Buffer, bufferW, len, filenameW.Buffer,
    ) as INT;
    if len != 0 && !buffer.is_null() {
        if retW != 0 {
            ret = WideCharToMultiByte(
                CP_ACP, 0, bufferW, retW, buffer, (len - 1) as INT, core::ptr::null(), core::ptr::null_mut(),
            );
            if ret == 0 {
                ret = (len - 1) as INT;
            }
        }
        *buffer.offset(ret as isize) = 0;
    }

    RtlFreeUnicodeString(&raw mut sectionW);
    RtlFreeUnicodeString(&raw mut entryW);
    RtlFreeUnicodeString(&raw mut def_valW);
    RtlFreeUnicodeString(&raw mut filenameW);
    HeapFree(GetProcessHeap(), 0, bufferW as LPVOID);
    return ret as DWORD;
}

/// GetProfileStringW (KERNEL32.@) — profile.c
#[no_mangle]
pub unsafe extern "system" fn GetProfileStringW(
    section: LPCWSTR,
    entry: LPCWSTR,
    def_val: LPCWSTR,
    buffer: LPWSTR,
    len: DWORD,
) -> DWORD {
    GetPrivateProfileStringW(section, entry, def_val, buffer, len, (*(&raw const wininiW)).as_ptr())
}

/// GetProfileStringA (KERNEL32.@) — profile.c
#[no_mangle]
pub unsafe extern "system" fn GetProfileStringA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: LPCSTR,
    buffer: LPSTR,
    len: DWORD,
) -> DWORD {
    GetPrivateProfileStringA(section, entry, def_val, buffer, len, b"win.ini\0".as_ptr() as LPCSTR)
}

/// GetProfileSectionA (KERNEL32.@) — profile.c
#[no_mangle]
pub unsafe extern "system" fn GetProfileSectionA(
    section: LPCSTR,
    buffer: LPSTR,
    len: DWORD,
) -> DWORD {
    GetPrivateProfileSectionA(section, buffer, len, b"win.ini\0".as_ptr() as LPCSTR)
}

#[no_mangle]
pub unsafe extern "system" fn WritePrivateProfileStringW(
    section: LPCWSTR,
    entry: LPCWSTR,
    string: LPCWSTR,
    filename: LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = 0;

    RtlEnterCriticalSection(&raw mut PROFILE_CritSect);

    if section.is_null() && entry.is_null() && string.is_null() {
        if filename.is_null() || PROFILE_Open(filename, 1) != 0 {
            if !MRUProfile[0].is_null() {
                PROFILE_ReleaseFile();
            }
        }
    } else if PROFILE_Open(filename, 1) != 0 {
        if section.is_null() {
            SetLastError(ERROR_FILE_NOT_FOUND as u32);
        } else {
            ret = PROFILE_SetString(section, entry, string, 0);
            if ret != 0 {
                ret = PROFILE_FlushFile();
            }
        }
    }

    RtlLeaveCriticalSection(&raw mut PROFILE_CritSect);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn WritePrivateProfileStringA(
    section: LPCSTR,
    entry: LPCSTR,
    string: LPCSTR,
    filename: LPCSTR,
) -> BOOL {
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut entryW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut stringW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: BOOL = 0;

    if !section.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut sectionW, section as PCSZ);
    } else {
        sectionW.Buffer = core::ptr::null_mut();
    }
    if !entry.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut entryW, entry as PCSZ);
    } else {
        entryW.Buffer = core::ptr::null_mut();
    }
    if !string.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut stringW, string as PCSZ);
    } else {
        stringW.Buffer = core::ptr::null_mut();
    }
    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as PCSZ);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    ret = WritePrivateProfileStringW(
        sectionW.Buffer as LPCWSTR,
        entryW.Buffer as LPCWSTR,
        stringW.Buffer as LPCWSTR,
        filenameW.Buffer as LPCWSTR,
    );

    RtlFreeUnicodeString(&raw mut sectionW);
    RtlFreeUnicodeString(&raw mut entryW);
    RtlFreeUnicodeString(&raw mut stringW);
    RtlFreeUnicodeString(&raw mut filenameW);

    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn WritePrivateProfileSectionW(
    section: LPCWSTR,
    mut string: LPCWSTR,
    filename: LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut p: LPWSTR = core::ptr::null_mut();

    RtlEnterCriticalSection(&raw mut PROFILE_CritSect);

    if section.is_null() && string.is_null() {
        if filename.is_null() || PROFILE_Open(filename, 1) != 0 {
            if !MRUProfile[0].is_null() {
                PROFILE_ReleaseFile();
            }
        }
    } else if PROFILE_Open(filename, 1) != 0 {
        if string.is_null() {
            ret = PROFILE_SetString(section, core::ptr::null(), core::ptr::null(), 0);
        } else {
            PROFILE_DeleteAllKeys(section);
            ret = 1;
            while *string != 0 && ret != 0 {
                let mut buf: LPWSTR = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (strlenW(string) + 1) * core::mem::size_of::<WCHAR>(),
                ) as LPWSTR;
                strcpyW(buf, string);
                p = strchrW(buf, '=' as i32 as WCHAR) as LPWSTR;
                if !p.is_null() {
                    *p = 0;
                    ret = PROFILE_SetString(section, buf as LPCWSTR, p.offset(1) as LPCWSTR, 1);
                }
                HeapFree(GetProcessHeap(), 0, buf as *mut core::ffi::c_void);
                string = string.offset(strlenW(string) as isize + 1);
            }
        }
        if ret != 0 {
            ret = PROFILE_FlushFile();
        }
    }

    RtlLeaveCriticalSection(&raw mut PROFILE_CritSect);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn WritePrivateProfileSectionA(
    section: LPCSTR,
    string: LPCSTR,
    filename: LPCSTR,
) -> BOOL {
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut stringW: LPWSTR;
    let mut ret: BOOL;

    if !string.is_null() {
        let mut lenA: INT;
        let mut lenW: INT;
        let mut p: LPCSTR = string;

        while *p != 0 {
            p = p.add(strlen(p as *const i8).wrapping_add(1));
        }
        lenA = (p.offset_from(string) as INT).wrapping_add(1);
        lenW = MultiByteToWideChar(
            0, // CP_ACP
            0,
            string,
            lenA,
            core::ptr::null_mut(),
            0,
        );
        stringW = HeapAlloc(GetProcessHeap(), 0, (lenW as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as LPWSTR;
        if !stringW.is_null() {
            MultiByteToWideChar(
                0, // CP_ACP
                0,
                string,
                lenA,
                stringW,
                lenW,
            );
        }
    } else {
        stringW = core::ptr::null_mut();
    }

    if !section.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut sectionW, section as *const i8);
    } else {
        sectionW.Buffer = core::ptr::null_mut();
    }

    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as *const i8);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    ret = WritePrivateProfileSectionW(
        sectionW.Buffer,
        stringW,
        filenameW.Buffer,
    );

    HeapFree(GetProcessHeap(), 0, stringW as *mut _);
    RtlFreeUnicodeString(&raw mut sectionW);
    RtlFreeUnicodeString(&raw mut filenameW);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn WriteProfileSectionA(
    section: LPCSTR,
    keys_n_values: LPCSTR,
) -> BOOL {
    return WritePrivateProfileSectionA(
        section,
        keys_n_values,
        b"win.ini\0".as_ptr() as LPCSTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn WriteProfileSectionW(
    section: LPCWSTR,
    keys_n_values: LPCWSTR,
) -> BOOL {
    return WritePrivateProfileSectionW(section, keys_n_values, &raw const wininiW as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileSectionNamesW(
    mut buffer: LPWSTR,
    mut size: DWORD,
    mut filename: LPCWSTR,
) -> DWORD {
    let mut ret: DWORD = 0;
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(PROFILE_CritSect));
    if PROFILE_Open(filename, 0) != 0 {
        ret = PROFILE_GetSectionNames(buffer, size) as DWORD;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(PROFILE_CritSect));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileSectionNamesA(
    mut buffer: LPSTR,
    mut size: DWORD,
    mut filename: LPCSTR,
) -> DWORD {
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut bufferW: LPWSTR = core::ptr::null_mut();
    let mut retW: INT = 0;
    let mut ret: INT = 0;

    bufferW = if !buffer.is_null() {
        HeapAlloc(
            GetProcessHeap(),
            0,
            (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as LPWSTR
    } else {
        core::ptr::null_mut()
    };

    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as PCSZ);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    retW = GetPrivateProfileSectionNamesW(bufferW, size, filenameW.Buffer as LPCWSTR) as INT;

    if retW != 0 && size != 0 {
        ret = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            bufferW as LPCWSTR,
            (retW as i32 + 1) as i32,
            buffer,
            (size as i32).wrapping_sub(1) as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as INT;

        if ret == 0 {
            ret = (size as i32).wrapping_sub(2) as INT;
            *buffer.offset((size as i32).wrapping_sub(1) as isize) = 0 as CHAR;
        } else {
            ret = (ret as i32 - 1) as INT;
        }
    } else if size != 0 {
        *buffer.offset(0) = '\0' as i32 as CHAR;
    }

    RtlFreeUnicodeString(&raw mut filenameW);
    HeapFree(GetProcessHeap(), 0, bufferW as LPVOID);
    return ret as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileStructW(
    section: LPCWSTR,
    key: LPCWSTR,
    buf: LPVOID,
    len: UINT,
    filename: LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = FALSE;

    RtlEnterCriticalSection(&raw mut PROFILE_CritSect);

    if PROFILE_Open(filename, FALSE) != 0 {
        let mut k: *mut PROFILEKEY = PROFILE_Find(
            &raw mut (*MRUProfile[0]).section,
            section,
            key,
            FALSE,
            FALSE,
        );
        if !k.is_null() {
            TRACE!(
                b"value (at %p): %s\n\0".as_ptr() as LPCSTR,
                (*k).value,
                debugstr_w((*k).value),
            );
            if ((strlenW((*k).value) - 2) / 2) == len as usize {
                let mut end: LPWSTR;
                let mut p: LPWSTR;
                let mut valid: BOOL = TRUE;
                let mut c: WCHAR = 0;
                let mut chksum: DWORD = 0;

                end = (*k).value.offset(strlenW((*k).value) as isize) as LPWSTR;
                p = (*k).value as LPWSTR;
                while p < end {
                    if !isxdigitW(*p) {
                        WARN!(
                            b"invalid char '%x' in file %s->[%s]->%s !\n\0".as_ptr() as LPCSTR,
                            *p as i32,
                            debugstr_w(filename),
                            debugstr_w(section),
                            debugstr_w(key),
                        );
                        valid = FALSE;
                        break;
                    }
                    p = p.offset(1);
                }
                if valid != 0 {
                    let mut highnibble: BOOL = TRUE;
                    let mut b: BYTE = 0;
                    let mut val: BYTE = 0;
                    let mut binbuf: LPBYTE = buf as LPBYTE;

                    end = end.offset(-(2 as isize));
                    p = (*k).value as LPWSTR;
                    while p < end {
                        c = toupperW(*p);
                        val = (if c as i32 > '9' as i32 {
                            c as i32 - 'A' as i32 + 10
                        } else {
                            c as i32 - '0' as i32
                        }) as BYTE;
                        if highnibble != 0 {
                            b = ((val as i32) << 4) as BYTE;
                        } else {
                            b = (b as i32 + val as i32) as BYTE;
                            let fresh10 = binbuf;
                            binbuf = binbuf.offset(1);
                            *fresh10 = b;
                            chksum = (chksum as u32).wrapping_add(b as u32) as DWORD;
                        }
                        highnibble ^= 1;
                        p = p.offset(1);
                    }
                    let fresh11 = p;
                    p = p.offset(1);
                    c = toupperW(*fresh11);
                    b = ((if c as i32 > '9' as i32 {
                        c as i32 - 'A' as i32 + 10
                    } else {
                        c as i32 - '0' as i32
                    }) << 4) as BYTE;
                    c = toupperW(*p);
                    b = (b as i32
                        + if c as i32 > '9' as i32 {
                            c as i32 - 'A' as i32 + 10
                        } else {
                            c as i32 - '0' as i32
                        }) as BYTE;
                    if b as u32 == chksum as u32 & 0xff {
                        ret = TRUE;
                    }
                }
            }
        }
    }

    RtlLeaveCriticalSection(&raw mut PROFILE_CritSect);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateProfileStructA(
    section: LPCSTR,
    key: LPCSTR,
    buffer: LPVOID,
    len: UINT,
    filename: LPCSTR,
) -> BOOL {
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut keyW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: INT = 0;

    if !section.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut sectionW, section);
    } else {
        sectionW.Buffer = core::ptr::null_mut();
    }
    if !key.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut keyW, key);
    } else {
        keyW.Buffer = core::ptr::null_mut();
    }
    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut filenameW, filename);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    ret = GetPrivateProfileStructW(
        sectionW.Buffer,
        keyW.Buffer,
        buffer,
        len,
        filenameW.Buffer,
    ) as INT;

    RtlFreeUnicodeString(&mut sectionW);
    RtlFreeUnicodeString(&mut keyW);
    RtlFreeUnicodeString(&mut filenameW);
    return ret as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WritePrivateProfileStructW(
    section: LPCWSTR,
    key: LPCWSTR,
    buf: LPVOID,
    bufsize: UINT,
    filename: LPCWSTR,
) -> BOOL {
    let mut ret: BOOL = FALSE;
    let mut binbuf: LPBYTE;
    let mut outstring: LPWSTR;
    let mut p: LPWSTR;
    let mut sum: DWORD = 0;

    if section.is_null() && key.is_null() && buf.is_null() {
        return WritePrivateProfileStringW(
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            filename,
        );
    }

    outstring = HeapAlloc(
        GetProcessHeap(),
        0,
        (bufsize as usize).wrapping_mul(2).wrapping_add(2).wrapping_add(1).wrapping_mul(core::mem::size_of::<WCHAR>()),
    ) as LPWSTR;
    p = outstring;
    binbuf = buf as LPBYTE;
    while binbuf < (buf as LPBYTE).add(bufsize as usize) {
        *p = hex[(*binbuf >> 4) as usize] as WCHAR;
        p = p.add(1);
        *p = hex[(*binbuf & 0xf) as usize] as WCHAR;
        p = p.add(1);
        sum = sum.wrapping_add(*binbuf as DWORD);
        binbuf = binbuf.add(1);
    }
    *p = hex[((sum & 0xf0) >> 4) as usize] as WCHAR;
    p = p.add(1);
    *p = hex[(sum & 0xf) as usize] as WCHAR;
    p = p.add(1);
    *p = '\0' as WCHAR;

    RtlEnterCriticalSection(&raw mut PROFILE_CritSect);

    if PROFILE_Open(filename, TRUE) != 0 {
        ret = PROFILE_SetString(section, key, outstring as LPCWSTR, FALSE);
        if ret != 0 {
            ret = PROFILE_FlushFile();
        }
    }

    RtlLeaveCriticalSection(&raw mut PROFILE_CritSect);

    HeapFree(GetProcessHeap(), 0, outstring as LPVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn WritePrivateProfileStructA(
    section: LPCSTR,
    key: LPCSTR,
    buf: LPVOID,
    bufsize: UINT,
    filename: LPCSTR,
) -> BOOL {
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut keyW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut filenameW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: INT = 0;

    if !section.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut sectionW, section as PCSZ);
    } else {
        sectionW.Buffer = core::ptr::null_mut();
    }
    if !key.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut keyW, key as PCSZ);
    } else {
        keyW.Buffer = core::ptr::null_mut();
    }
    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&raw mut filenameW, filename as PCSZ);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    ret = WritePrivateProfileStructW(
        sectionW.Buffer as LPCWSTR,
        keyW.Buffer as LPCWSTR,
        buf,
        bufsize,
        filenameW.Buffer as LPCWSTR,
    ) as INT;

    RtlFreeUnicodeString(&raw mut sectionW);
    RtlFreeUnicodeString(&raw mut keyW);
    RtlFreeUnicodeString(&raw mut filenameW);
    return ret as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn OpenProfileUserMapping() -> BOOL {
    FIXME!("(), stub!\n");
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CloseProfileUserMapping() -> BOOL {
    FIXME!("(), stub!\n");
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Profile functions
//  *
//  * Copyright 1993 Miguel de Icaza
//  * Copyright 1996 Alexandre Julliard
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #ifdef __REACTOS__
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(profile);
//
// #else /* __REACTOS__ */
//
// #include "config.h"
// #include "wine/port.h"
//
// #include <string.h>
// #include <stdarg.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "winnls.h"
// #include "winerror.h"
// #include "winternl.h"
// #include "wine/unicode.h"
// #include "wine/library.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(profile);
//
// #endif /* __REACTOS__ */
//
// static const char bom_utf8[] = {0xEF,0xBB,0xBF};
//
// typedef enum
// {
//     ENCODING_ANSI = 1,
//     ENCODING_UTF8,
//     ENCODING_UTF16LE,
//     ENCODING_UTF16BE
// } ENCODING;
//
// typedef struct tagPROFILEKEY
// {
//     WCHAR                 *value;
//     struct tagPROFILEKEY  *next;
//     WCHAR                  name[1];
// } PROFILEKEY;
//
// typedef struct tagPROFILESECTION
// {
//     struct tagPROFILEKEY       *key;
//     struct tagPROFILESECTION   *next;
//     WCHAR                       name[1];
// } PROFILESECTION;
//
//
// typedef struct
// {
//     BOOL             changed;
//     PROFILESECTION  *section;
//     WCHAR           *filename;
//     FILETIME LastWriteTime;
//     ENCODING encoding;
// } PROFILE;
//
//
// #define N_CACHED_PROFILES 10
//
// /* Cached profile files */
// static PROFILE *MRUProfile[N_CACHED_PROFILES]={NULL};
//
// #define MRUProfile[0] (MRUProfile[0])
//
// /* Check for comments in profile */
// #define IS_ENTRY_COMMENT(str)  ((str)[0] == ';')
//
// static const WCHAR emptystringW[] = {0};
// static const WCHAR wininiW[] = { 'w','i','n','.','i','n','i',0 };
//
// #ifdef __REACTOS__
//
// static RTL_CRITICAL_SECTION PROFILE_CritSect;
// static RTL_CRITICAL_SECTION_DEBUG critsect_debug =
// {
//     0, 0, &PROFILE_CritSect,
//     { &critsect_debug.ProcessLocksList, &critsect_debug.ProcessLocksList },
//       0, 0, { 0 }
// };
// static RTL_CRITICAL_SECTION PROFILE_CritSect = { &critsect_debug, -1, 0, 0, 0, 0 };
//
// #else /* __REACTOS__ */
//
// static CRITICAL_SECTION PROFILE_CritSect;
// static CRITICAL_SECTION_DEBUG critsect_debug =
// {
//     0, 0, &PROFILE_CritSect,
//     { &critsect_debug.ProcessLocksList, &critsect_debug.ProcessLocksList },
//       0, 0, { (DWORD_PTR)(__FILE__ ": PROFILE_CritSect") }
// };
// static CRITICAL_SECTION PROFILE_CritSect = { &critsect_debug, -1, 0, 0, 0, 0 };
//
// #endif /* __REACTOS__ */
//
// static const char hex[16] = "0123456789ABCDEF";
//
// /***********************************************************************
//  *           PROFILE_CopyEntry
//  *
//  * Copy the content of an entry into a buffer, removing quotes, and possibly
//  * translating environment variables.
//  */
// static void PROFILE_CopyEntry( LPWSTR buffer, LPCWSTR value, int len,
//                                BOOL strip_quote )
// {
//     WCHAR quote = '\0';
//
//     if(!buffer) return;
//
//     if (strip_quote && ((*value == '\'') || (*value == '\"')))
//     {
//         if (value[1] && (value[strlenW(value)-1] == *value)) quote = *value++;
//     }
//
//     lstrcpynW( buffer, value, len );
//     if (quote && (len >= lstrlenW(value))) buffer[strlenW(buffer)-1] = '\0';
// }
//
// /* byte-swaps shorts in-place in a buffer. len is in WCHARs */
// static inline void PROFILE_ByteSwapShortBuffer(WCHAR * buffer, int len)
// {
//     int i;
//     USHORT * shortbuffer = buffer;
//     for (i = 0; i < len; i++)
//         shortbuffer[i] = RtlUshortByteSwap(shortbuffer[i]);
// }
//
// /* writes any necessary encoding marker to the file */
// static inline void PROFILE_WriteMarker(HANDLE hFile, ENCODING encoding)
// {
//     DWORD dwBytesWritten;
//     WCHAR bom;
//     switch (encoding)
//     {
//     case ENCODING_ANSI:
//         break;
//     case ENCODING_UTF8:
//         WriteFile(hFile, bom_utf8, sizeof(bom_utf8), &dwBytesWritten, NULL);
//         break;
//     case ENCODING_UTF16LE:
//         bom = 0xFEFF;
//         WriteFile(hFile, &bom, sizeof(bom), &dwBytesWritten, NULL);
//         break;
//     case ENCODING_UTF16BE:
//         bom = 0xFFFE;
//         WriteFile(hFile, &bom, sizeof(bom), &dwBytesWritten, NULL);
//         break;
//     }
// }
//
// static void PROFILE_WriteLine( HANDLE hFile, WCHAR * szLine, int len, ENCODING encoding)
// {
//     char * write_buffer;
//     int write_buffer_len;
//     DWORD dwBytesWritten;
//
//     TRACE!("writing: %s\n", debugstr_wn(szLine, len));
//
//     switch (encoding)
//     {
//     case ENCODING_ANSI:
//         write_buffer_len = WideCharToMultiByte(CP_ACP, 0, szLine, len, NULL, 0, NULL, NULL);
//         write_buffer = HeapAlloc(GetProcessHeap(), 0, write_buffer_len);
//         if (!write_buffer) return;
//         len = WideCharToMultiByte(CP_ACP, 0, szLine, len, write_buffer, write_buffer_len, NULL, NULL);
//         WriteFile(hFile, write_buffer, len, &dwBytesWritten, NULL);
//         HeapFree(GetProcessHeap(), 0, write_buffer);
//         break;
//     case ENCODING_UTF8:
//         write_buffer_len = WideCharToMultiByte(CP_UTF8, 0, szLine, len, NULL, 0, NULL, NULL);
//         write_buffer = HeapAlloc(GetProcessHeap(), 0, write_buffer_len);
//         if (!write_buffer) return;
//         len = WideCharToMultiByte(CP_UTF8, 0, szLine, len, write_buffer, write_buffer_len, NULL, NULL);
//         WriteFile(hFile, write_buffer, len, &dwBytesWritten, NULL);
//         HeapFree(GetProcessHeap(), 0, write_buffer);
//         break;
//     case ENCODING_UTF16LE:
//         WriteFile(hFile, szLine, len * sizeof(WCHAR), &dwBytesWritten, NULL);
//         break;
//     case ENCODING_UTF16BE:
//         PROFILE_ByteSwapShortBuffer(szLine, len);
//         WriteFile(hFile, szLine, len * sizeof(WCHAR), &dwBytesWritten, NULL);
//         break;
//     default:
//         FIXME!("encoding type %d not implemented\n", encoding);
//     }
// }
//
// /***********************************************************************
//  *           PROFILE_Save
//  *
//  * Save a profile tree to a file.
//  */
// static void PROFILE_Save( HANDLE hFile, const PROFILESECTION *section, ENCODING encoding )
// {
//     PROFILEKEY *key;
//     WCHAR *buffer, *p;
//
//     PROFILE_WriteMarker(hFile, encoding);
//
//     for ( ; section; section = section->next)
//     {
//         int len = 0;
//
//         if (section->name[0]) len += strlenW(section->name) + 4;
//
//         for (key = section->key; key; key = key->next)
//         {
//             len += strlenW(key->name) + 2;
//             if (key->value) len += strlenW(key->value) + 1;
//         }
//
//         buffer = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!buffer) return;
//
//         p = buffer;
//         if (section->name[0])
//         {
//             *p++ = '[';
//             strcpyW( p, section->name );
//             p += strlenW(p);
//             *p++ = ']';
//             *p++ = '\r';
//             *p++ = '\n';
//         }
//
//         for (key = section->key; key; key = key->next)
//         {
//             strcpyW( p, key->name );
//             p += strlenW(p);
//             if (key->value)
//             {
//                 *p++ = '=';
//                 strcpyW( p, key->value );
//                 p += strlenW(p);
//             }
//             *p++ = '\r';
//             *p++ = '\n';
//         }
//         PROFILE_WriteLine( hFile, buffer, len, encoding );
//         HeapFree(GetProcessHeap(), 0, buffer);
//     }
// }
//
//
// /***********************************************************************
//  *           PROFILE_Free
//  *
//  * Free a profile tree.
//  */
// static void PROFILE_Free( PROFILESECTION *section )
// {
//     PROFILESECTION *next_section;
//     PROFILEKEY *key, *next_key;
//
//     for ( ; section; section = next_section)
//     {
//         for (key = section->key; key; key = next_key)
//         {
//             next_key = key->next;
//             HeapFree( GetProcessHeap(), 0, key->value );
//             HeapFree( GetProcessHeap(), 0, key );
//         }
//         next_section = section->next;
//         HeapFree( GetProcessHeap(), 0, section );
//     }
// }
//
// /* returns TRUE if a whitespace character, else FALSE */
// static inline BOOL PROFILE_isspaceW(WCHAR c)
// {
// 	/* ^Z (DOS EOF) is a space too  (found on CD-ROMs) */
// 	return isspaceW(c) || c == 0x1a;
// }
//
// static inline ENCODING PROFILE_DetectTextEncoding(const void * buffer, int * len)
// {
//     int flags = IS_TEXT_UNICODE_SIGNATURE |
//                 IS_TEXT_UNICODE_REVERSE_SIGNATURE |
//                 IS_TEXT_UNICODE_ODD_LENGTH;
//     if (*len >= sizeof(bom_utf8) && !memcmp(buffer, bom_utf8, sizeof(bom_utf8)))
//     {
//         *len = sizeof(bom_utf8);
//         return ENCODING_UTF8;
//     }
//     RtlIsTextUnicode(buffer, *len, &flags);
//     if (flags & IS_TEXT_UNICODE_SIGNATURE)
//     {
//         *len = sizeof(WCHAR);
//         return ENCODING_UTF16LE;
//     }
//     if (flags & IS_TEXT_UNICODE_REVERSE_SIGNATURE)
//     {
//         *len = sizeof(WCHAR);
//         return ENCODING_UTF16BE;
//     }
//     *len = 0;
//     return ENCODING_ANSI;
// }
//
//
// /***********************************************************************
//  *           PROFILE_Load
//  *
//  * Load a profile tree from a file.
//  */
// static PROFILESECTION *PROFILE_Load(HANDLE hFile, ENCODING * pEncoding)
// {
//     void *buffer_base, *pBuffer;
//     WCHAR * szFile;
//     const WCHAR *szLineStart, *szLineEnd;
//     const WCHAR *szValueStart, *szEnd, *next_line;
//     int line = 0, len;
//     PROFILESECTION *section, *first_section;
//     PROFILESECTION **next_section;
//     PROFILEKEY *key, *prev_key, **next_key;
//     DWORD dwFileSize;
//
//     TRACE!("%p\n", hFile);
//
//     dwFileSize = GetFileSize(hFile, NULL);
//     if (dwFileSize == INVALID_FILE_SIZE || dwFileSize == 0)
//         return NULL;
//
//     buffer_base = HeapAlloc(GetProcessHeap(), 0 , dwFileSize);
//     if (!buffer_base) return NULL;
//
//     if (!ReadFile(hFile, buffer_base, dwFileSize, &dwFileSize, NULL))
//     {
//         HeapFree(GetProcessHeap(), 0, buffer_base);
//         WARN!("Error %d reading file\n", GetLastError());
//         return NULL;
//     }
//     len = dwFileSize;
//     *pEncoding = PROFILE_DetectTextEncoding(buffer_base, &len);
//     /* len is set to the number of bytes in the character marker.
//      * we want to skip these bytes */
//     pBuffer = (char *)buffer_base + len;
//     dwFileSize -= len;
//     switch (*pEncoding)
//     {
//     case ENCODING_ANSI:
//         TRACE!("ANSI encoding\n");
//
//         len = MultiByteToWideChar(CP_ACP, 0, pBuffer, dwFileSize, NULL, 0);
//         szFile = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!szFile)
//         {
//             HeapFree(GetProcessHeap(), 0, buffer_base);
//             return NULL;
//         }
//         MultiByteToWideChar(CP_ACP, 0, pBuffer, dwFileSize, szFile, len);
//         szEnd = szFile + len;
//         break;
//     case ENCODING_UTF8:
//         TRACE!("UTF8 encoding\n");
//
//         len = MultiByteToWideChar(CP_UTF8, 0, pBuffer, dwFileSize, NULL, 0);
//         szFile = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!szFile)
//         {
//             HeapFree(GetProcessHeap(), 0, buffer_base);
//             return NULL;
//         }
//         MultiByteToWideChar(CP_UTF8, 0, pBuffer, dwFileSize, szFile, len);
//         szEnd = szFile + len;
//         break;
//     case ENCODING_UTF16LE:
//         TRACE!("UTF16 Little Endian encoding\n");
//         szFile = pBuffer;
//         szEnd = (WCHAR *)((char *)pBuffer + dwFileSize);
//         break;
//     case ENCODING_UTF16BE:
//         TRACE!("UTF16 Big Endian encoding\n");
//         szFile = pBuffer;
//         szEnd = (WCHAR *)((char *)pBuffer + dwFileSize);
//         PROFILE_ByteSwapShortBuffer(szFile, dwFileSize / sizeof(WCHAR));
//         break;
//     default:
//         FIXME!("encoding type %d not implemented\n", *pEncoding);
//         HeapFree(GetProcessHeap(), 0, buffer_base);
//         return NULL;
//     }
//
//     first_section = HeapAlloc( GetProcessHeap(), 0, sizeof(*section) );
//     if(first_section == NULL)
//     {
//         if (szFile != pBuffer)
//             HeapFree(GetProcessHeap(), 0, szFile);
//         HeapFree(GetProcessHeap(), 0, buffer_base);
//         return NULL;
//     }
//     first_section->name[0] = 0;
//     first_section->key  = NULL;
//     first_section->next = NULL;
//     next_section = &first_section->next;
//     next_key     = &first_section->key;
//     prev_key     = NULL;
//     next_line    = szFile;
//
//     while (next_line < szEnd)
//     {
//         szLineStart = next_line;
//         next_line = memchrW(szLineStart, '\n', szEnd - szLineStart);
//         if (!next_line) next_line = memchrW(szLineStart, '\r', szEnd - szLineStart);
//         if (!next_line) next_line = szEnd;
//         else next_line++;
//         szLineEnd = next_line;
//
//         line++;
//
//         /* get rid of white space */
//         while (szLineStart < szLineEnd && PROFILE_isspaceW(*szLineStart)) szLineStart++;
//         while ((szLineEnd > szLineStart) && PROFILE_isspaceW(szLineEnd[-1])) szLineEnd--;
//
//         if (szLineStart >= szLineEnd) continue;
//
//         if (*szLineStart == '[')  /* section start */
//         {
//             const WCHAR * szSectionEnd;
//             if (!(szSectionEnd = memrchrW( szLineStart, ']', szLineEnd - szLineStart )))
//             {
//                 WARN!("Invalid section header at line %d: %s\n",
//                     line, debugstr_wn(szLineStart, (int)(szLineEnd - szLineStart)) );
//             }
//             else
//             {
//                 szLineStart++;
//                 len = (int)(szSectionEnd - szLineStart);
//                 /* no need to allocate +1 for NULL terminating character as
//                  * already included in structure */
//                 if (!(section = HeapAlloc( GetProcessHeap(), 0, sizeof(*section) + len * sizeof(WCHAR) )))
//                     break;
//                 memcpy(section->name, szLineStart, len * sizeof(WCHAR));
//                 section->name[len] = '\0';
//                 section->key  = NULL;
//                 section->next = NULL;
//                 *next_section = section;
//                 next_section  = &section->next;
//                 next_key      = &section->key;
//                 prev_key      = NULL;
//
//                 TRACE!("New section: %s\n", debugstr_w(section->name));
//
//                 continue;
//             }
//         }
//
//         /* get rid of white space after the name and before the start
//          * of the value */
//         len = szLineEnd - szLineStart;
//         if ((szValueStart = memchrW( szLineStart, '=', szLineEnd - szLineStart )) != NULL)
//         {
//             const WCHAR *szNameEnd = szValueStart;
//             while ((szNameEnd > szLineStart) && PROFILE_isspaceW(szNameEnd[-1])) szNameEnd--;
//             len = szNameEnd - szLineStart;
//             szValueStart++;
//             while (szValueStart < szLineEnd && PROFILE_isspaceW(*szValueStart)) szValueStart++;
//         }
//
//         if (len || !prev_key || *prev_key->name)
//         {
//             /* no need to allocate +1 for NULL terminating character as
//              * already included in structure */
//             if (!(key = HeapAlloc( GetProcessHeap(), 0, sizeof(*key) + len * sizeof(WCHAR) ))) break;
//             memcpy(key->name, szLineStart, len * sizeof(WCHAR));
//             key->name[len] = '\0';
//             if (szValueStart)
//             {
//                 len = (int)(szLineEnd - szValueStart);
//                 key->value = HeapAlloc( GetProcessHeap(), 0, (len + 1) * sizeof(WCHAR) );
//                 memcpy(key->value, szValueStart, len * sizeof(WCHAR));
//                 key->value[len] = '\0';
//             }
//             else key->value = NULL;
//
//            key->next  = NULL;
//            *next_key  = key;
//            next_key   = &key->next;
//            prev_key   = key;
//
// #ifdef __REACTOS__
//            TRACE!("New key: name=%s, value=%s\n",
//                  debugstr_w(key->name),
//                  key->value ? debugstr_w(key->value) : L"(none)");
// #else
//            TRACE!("New key: name=%s, value=%s\n",
//                debugstr_w(key->name), key->value ? debugstr_w(key->value) : "(none)");
// #endif
//         }
//     }
//     if (szFile != pBuffer)
//         HeapFree(GetProcessHeap(), 0, szFile);
//     HeapFree(GetProcessHeap(), 0, buffer_base);
//     return first_section;
// }
//
//
// /***********************************************************************
//  *           PROFILE_DeleteSection
//  *
//  * Delete a section from a profile tree.
//  */
// static BOOL PROFILE_DeleteSection( PROFILESECTION **section, LPCWSTR name )
// {
//     while (*section)
//     {
//         if (!strcmpiW( (*section)->name, name ))
//         {
//             PROFILESECTION *to_del = *section;
//             *section = to_del->next;
//             to_del->next = NULL;
//             PROFILE_Free( to_del );
//             return TRUE;
//         }
//         section = &(*section)->next;
//     }
//     return FALSE;
// }
//
//
// /***********************************************************************
//  *           PROFILE_DeleteKey
//  *
//  * Delete a key from a profile tree.
//  */
// static BOOL PROFILE_DeleteKey( PROFILESECTION **section,
// 			       LPCWSTR section_name, LPCWSTR key_name )
// {
//     while (*section)
//     {
//         if (!strcmpiW( (*section)->name, section_name ))
//         {
//             PROFILEKEY **key = &(*section)->key;
//             while (*key)
//             {
//                 if (!strcmpiW( (*key)->name, key_name ))
//                 {
//                     PROFILEKEY *to_del = *key;
//                     *key = to_del->next;
//                     HeapFree( GetProcessHeap(), 0, to_del->value);
//                     HeapFree( GetProcessHeap(), 0, to_del );
//                     return TRUE;
//                 }
//                 key = &(*key)->next;
//             }
//         }
//         section = &(*section)->next;
//     }
//     return FALSE;
// }
//
//
// /***********************************************************************
//  *           PROFILE_DeleteAllKeys
//  *
//  * Delete all keys from a profile tree.
//  */
// static void PROFILE_DeleteAllKeys( LPCWSTR section_name)
// {
//     PROFILESECTION **section= &MRUProfile[0]->section;
//     while (*section)
//     {
//         if (!strcmpiW( (*section)->name, section_name ))
//         {
//             PROFILEKEY **key = &(*section)->key;
//             while (*key)
//             {
//                 PROFILEKEY *to_del = *key;
// 		*key = to_del->next;
//                 HeapFree( GetProcessHeap(), 0, to_del->value);
// 		HeapFree( GetProcessHeap(), 0, to_del );
// 		MRUProfile[0]->changed =TRUE;
//             }
//         }
//         section = &(*section)->next;
//     }
// }
//
//
// /***********************************************************************
//  *           PROFILE_Find
//  *
//  * Find a key in a profile tree, optionally creating it.
//  */
// static PROFILEKEY *PROFILE_Find( PROFILESECTION **section, LPCWSTR section_name,
//                                  LPCWSTR key_name, BOOL create, BOOL create_always )
// {
//     LPCWSTR p;
//     int seclen = 0, keylen = 0;
//
//     while (PROFILE_isspaceW(*section_name)) section_name++;
//     if (*section_name)
//     {
//         p = section_name + strlenW(section_name) - 1;
//         while ((p > section_name) && PROFILE_isspaceW(*p)) p--;
//         seclen = p - section_name + 1;
//     }
//
//     while (PROFILE_isspaceW(*key_name)) key_name++;
//     if (*key_name)
//     {
//         p = key_name + strlenW(key_name) - 1;
//         while ((p > key_name) && PROFILE_isspaceW(*p)) p--;
//         keylen = p - key_name + 1;
//     }
//
//     while (*section)
//     {
//         if (!strncmpiW((*section)->name, section_name, seclen) &&
//             ((*section)->name)[seclen] == '\0')
//         {
//             PROFILEKEY **key = &(*section)->key;
//
//             while (*key)
//             {
//                 /* If create_always is FALSE then we check if the keyname
//                  * already exists. Otherwise we add it regardless of its
//                  * existence, to allow keys to be added more than once in
//                  * some cases.
//                  */
//                 if(!create_always)
//                 {
//                     if ( (!(strncmpiW( (*key)->name, key_name, keylen )))
//                          && (((*key)->name)[keylen] == '\0') )
//                         return *key;
//                 }
//                 key = &(*key)->next;
//             }
//             if (!create) return NULL;
//             if (!(*key = HeapAlloc( GetProcessHeap(), 0, sizeof(PROFILEKEY) + strlenW(key_name) * sizeof(WCHAR) )))
//                 return NULL;
//             strcpyW( (*key)->name, key_name );
//             (*key)->value = NULL;
//             (*key)->next  = NULL;
//             return *key;
//         }
//         section = &(*section)->next;
//     }
//     if (!create) return NULL;
//     *section = HeapAlloc( GetProcessHeap(), 0, sizeof(PROFILESECTION) + strlenW(section_name) * sizeof(WCHAR) );
//     if(*section == NULL) return NULL;
//     strcpyW( (*section)->name, section_name );
//     (*section)->next = NULL;
//     if (!((*section)->key  = HeapAlloc( GetProcessHeap(), 0,
//                                         sizeof(PROFILEKEY) + strlenW(key_name) * sizeof(WCHAR) )))
//     {
//         HeapFree(GetProcessHeap(), 0, *section);
//         return NULL;
//     }
//     strcpyW( (*section)->key->name, key_name );
//     (*section)->key->value = NULL;
//     (*section)->key->next  = NULL;
//     return (*section)->key;
// }
//
//
// /***********************************************************************
//  *           PROFILE_FlushFile
//  *
//  * Flush the current profile to disk if changed.
//  */
// static BOOL PROFILE_FlushFile(void)
// {
//     HANDLE hFile = NULL;
//     FILETIME LastWriteTime;
//
//     if(!MRUProfile[0])
//     {
//         WARN!("No current profile!\n");
//         return FALSE;
//     }
//
//     if (!MRUProfile[0]->changed) return TRUE;
//
//     hFile = CreateFileW(MRUProfile[0]->filename, GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
//
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         WARN!("could not save profile file %s (error was %d)\n", debugstr_w(MRUProfile[0]->filename), GetLastError());
//         return FALSE;
//     }
//
//     TRACE!("Saving %s\n", debugstr_w(MRUProfile[0]->filename));
//     PROFILE_Save( hFile, MRUProfile[0]->section, MRUProfile[0]->encoding );
//     if(GetFileTime(hFile, NULL, NULL, &LastWriteTime))
//        MRUProfile[0]->LastWriteTime=LastWriteTime;
//     CloseHandle( hFile );
//     MRUProfile[0]->changed = FALSE;
//     return TRUE;
// }
//
//
// /***********************************************************************
//  *           PROFILE_ReleaseFile
//  *
//  * Flush the current profile to disk and remove it from the cache.
//  */
// static void PROFILE_ReleaseFile(void)
// {
//     PROFILE_FlushFile();
//     PROFILE_Free( MRUProfile[0]->section );
//     HeapFree( GetProcessHeap(), 0, MRUProfile[0]->filename );
//     MRUProfile[0]->changed = FALSE;
//     MRUProfile[0]->section = NULL;
//     MRUProfile[0]->filename  = NULL;
//     MRUProfile[0]->encoding = ENCODING_ANSI;
//     ZeroMemory(&MRUProfile[0]->LastWriteTime, sizeof(MRUProfile[0]->LastWriteTime));
// }
//
// /***********************************************************************
//  *
//  * Compares a file time with the current time. If the file time is
//  * at least 2.1 seconds in the past, return true.
//  *
//  * Intended as cache safety measure: The time resolution on FAT is
//  * two seconds, so files that are not at least two seconds old might
//  * keep their time even on modification, so don't cache them.
//  */
// static BOOL is_not_current(FILETIME * ft)
// {
//     FILETIME Now;
//     LONGLONG ftll, nowll;
//     GetSystemTimeAsFileTime(&Now);
//     ftll = ((LONGLONG)ft->dwHighDateTime << 32) + ft->dwLowDateTime;
//     nowll = ((LONGLONG)Now.dwHighDateTime << 32) + Now.dwLowDateTime;
//     TRACE!("%08x;%08x\n",(unsigned)ftll+21000000,(unsigned)nowll);
//     return ftll + 21000000 < nowll;
// }
//
// /***********************************************************************
//  *           PROFILE_Open
//  *
//  * Open a profile file, checking the cached file first.
//  */
// static BOOL PROFILE_Open( LPCWSTR filename, BOOL write_access )
// {
//     WCHAR buffer[MAX_PATH];
//     HANDLE hFile = INVALID_HANDLE_VALUE;
//     FILETIME LastWriteTime;
//     int i,j;
//     PROFILE *tempProfile;
//
//     ZeroMemory(&LastWriteTime, sizeof(LastWriteTime));
//
//     /* First time around */
//
//     if(!MRUProfile[0])
//        for(i=0;i<N_CACHED_PROFILES;i++)
//        {
//           MRUProfile[i]=HeapAlloc( GetProcessHeap(), 0, sizeof(PROFILE) );
//           if(MRUProfile[i] == NULL) break;
//           MRUProfile[i]->changed=FALSE;
//           MRUProfile[i]->section=NULL;
//           MRUProfile[i]->filename=NULL;
//           MRUProfile[i]->encoding=ENCODING_ANSI;
//           ZeroMemory(&MRUProfile[i]->LastWriteTime, sizeof(FILETIME));
//        }
//
//     if (!filename)
// 	filename = wininiW;
//
// #ifdef __REACTOS__
//     if ((RtlDetermineDosPathNameType_U(filename) == RtlPathTypeRelative) &&
// #else
//     if ((RtlDetermineDosPathNameType_U(filename) == RELATIVE_PATH) &&
// #endif
//         !strchrW(filename, '\\') && !strchrW(filename, '/'))
//     {
//         static const WCHAR wszSeparator[] = {'\\', 0};
//         WCHAR windirW[MAX_PATH];
//         GetWindowsDirectoryW( windirW, MAX_PATH );
//         strcpyW(buffer, windirW);
//         strcatW(buffer, wszSeparator);
//         strcatW(buffer, filename);
//     }
//     else
//     {
//         LPWSTR dummy;
//         GetFullPathNameW(filename, sizeof(buffer)/sizeof(buffer[0]), buffer, &dummy);
//     }
//
//     TRACE!("path: %s\n", debugstr_w(buffer));
//
//     hFile = CreateFileW(buffer, GENERIC_READ | (write_access ? GENERIC_WRITE : 0),
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL,
//                         OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
//
//     if ((hFile == INVALID_HANDLE_VALUE) && (GetLastError() != ERROR_FILE_NOT_FOUND))
//     {
//         WARN!("Error %d opening file %s\n", GetLastError(), debugstr_w(buffer));
//         return FALSE;
//     }
//
//     for(i=0;i<N_CACHED_PROFILES;i++)
//     {
//         if ((MRUProfile[i]->filename && !strcmpiW( buffer, MRUProfile[i]->filename )))
//         {
//             TRACE!("MRU Filename: %s, new filename: %s\n", debugstr_w(MRUProfile[i]->filename), debugstr_w(buffer));
//             if(i)
//             {
//                 PROFILE_FlushFile();
//                 tempProfile=MRUProfile[i];
//                 for(j=i;j>0;j--)
//                     MRUProfile[j]=MRUProfile[j-1];
//                 MRUProfile[0]=tempProfile;
//             }
//
//             if (hFile != INVALID_HANDLE_VALUE)
//             {
//                 GetFileTime(hFile, NULL, NULL, &LastWriteTime);
//                 if (!memcmp( &MRUProfile[0]->LastWriteTime, &LastWriteTime, sizeof(FILETIME) ) &&
//                     is_not_current(&LastWriteTime))
//                     TRACE!("(%s): already opened (mru=%d)\n",
//                           debugstr_w(buffer), i);
//                 else
//                 {
//                     TRACE!("(%s): already opened, needs refreshing (mru=%d)\n",
//                           debugstr_w(buffer), i);
//                     PROFILE_Free(MRUProfile[0]->section);
//                     MRUProfile[0]->section = PROFILE_Load(hFile, &MRUProfile[0]->encoding);
//                     MRUProfile[0]->LastWriteTime = LastWriteTime;
//                 }
//                 CloseHandle(hFile);
//                 return TRUE;
//             }
//             else TRACE!("(%s): already opened, not yet created (mru=%d)\n",
//                        debugstr_w(buffer), i);
//         }
//     }
//
//     /* Flush the old current profile */
//     PROFILE_FlushFile();
//
//     /* Make the oldest profile the current one only in order to get rid of it */
//     if(i==N_CACHED_PROFILES)
//       {
//        tempProfile=MRUProfile[N_CACHED_PROFILES-1];
//        for(i=N_CACHED_PROFILES-1;i>0;i--)
//           MRUProfile[i]=MRUProfile[i-1];
//        MRUProfile[0]=tempProfile;
//       }
//     if(MRUProfile[0]->filename) PROFILE_ReleaseFile();
//
//     /* OK, now that MRUProfile[0] is definitely free we assign it our new file */
//     MRUProfile[0]->filename  = HeapAlloc( GetProcessHeap(), 0, (strlenW(buffer)+1) * sizeof(WCHAR) );
//     strcpyW( MRUProfile[0]->filename, buffer );
//
//     if (hFile != INVALID_HANDLE_VALUE)
//     {
//         MRUProfile[0]->section = PROFILE_Load(hFile, &MRUProfile[0]->encoding);
//         GetFileTime(hFile, NULL, NULL, &MRUProfile[0]->LastWriteTime);
//         CloseHandle(hFile);
//     }
//     else
//     {
//         /* Does not exist yet, we will create it in PROFILE_FlushFile */
//         WARN!("profile file %s not found\n", debugstr_w(buffer) );
//     }
//     return TRUE;
// }
//
//
// /***********************************************************************
//  *           PROFILE_GetSection
//  *
//  * Returns all keys of a section.
//  * If return_values is TRUE, also include the corresponding values.
//  */
// static INT PROFILE_GetSection( PROFILESECTION *section, LPCWSTR section_name,
// #ifdef __REACTOS__
// 			       LPWSTR buffer, DWORD len, BOOL return_values )
// #else
// 			       LPWSTR buffer, UINT len, BOOL return_values )
// #endif
// {
//     PROFILEKEY *key;
//
//     if(!buffer) return 0;
//
//     TRACE!("%s,%p,%u\n", debugstr_w(section_name), buffer, len);
//
//     while (section)
//     {
//         if (!strcmpiW( section->name, section_name ))
//         {
//             UINT oldlen = len;
//             for (key = section->key; key; key = key->next)
//             {
//                 if (len <= 2) break;
//                 if (!*key->name) continue;  /* Skip empty lines */
//                 if (IS_ENTRY_COMMENT(key->name)) continue;  /* Skip comments */
//                 if (!return_values && !key->value) continue;  /* Skip lines w.o. '=' */
//                 PROFILE_CopyEntry( buffer, key->name, len - 1, 0 );
//                 len -= strlenW(buffer) + 1;
//                 buffer += strlenW(buffer) + 1;
// 		if (len < 2)
// 		    break;
// 		if (return_values && key->value) {
// 			buffer[-1] = '=';
// 			PROFILE_CopyEntry ( buffer, key->value, len - 1, 0 );
// 			len -= strlenW(buffer) + 1;
// 			buffer += strlenW(buffer) + 1;
//                 }
//             }
//             *buffer = '\0';
//             if (len <= 1)
//                 /*If either lpszSection or lpszKey is NULL and the supplied
//                   destination buffer is too small to hold all the strings,
//                   the last string is truncated and followed by two null characters.
//                   In this case, the return value is equal to cchReturnBuffer
//                   minus two. */
//             {
// 		buffer[-1] = '\0';
//                 return oldlen - 2;
//             }
//             return oldlen - len;
//         }
//         section = section->next;
//     }
//     buffer[0] = buffer[1] = '\0';
//     return 0;
// }
//
// /* See GetPrivateProfileSectionNamesA for documentation */
// #ifdef __REACTOS__
// static INT PROFILE_GetSectionNames( LPWSTR buffer, DWORD len )
// #else
// static INT PROFILE_GetSectionNames( LPWSTR buffer, UINT len )
// #endif
// {
//     LPWSTR buf;
//     UINT buflen,tmplen;
//     PROFILESECTION *section;
//
//     TRACE!("(%p, %d)\n", buffer, len);
//
//     if (!buffer || !len)
//         return 0;
//     if (len==1) {
//         *buffer='\0';
//         return 0;
//     }
//
//     buflen=len-1;
//     buf=buffer;
//     section = MRUProfile[0]->section;
//     while ((section!=NULL)) {
//         if (section->name[0]) {
//             tmplen = strlenW(section->name)+1;
//             if (tmplen >= buflen) {
//                 if (buflen > 0) {
//                     memcpy(buf, section->name, (buflen-1) * sizeof(WCHAR));
//                     buf += buflen-1;
//                     *buf++='\0';
//                 }
//                 *buf='\0';
//                 return len-2;
//             }
//             memcpy(buf, section->name, tmplen * sizeof(WCHAR));
//             buf += tmplen;
//             buflen -= tmplen;
//         }
//         section = section->next;
//     }
//     *buf='\0';
//     return buf-buffer;
// }
//
//
// /***********************************************************************
//  *           PROFILE_GetString
//  *
//  * Get a profile string.
//  *
//  * Tests with GetPrivateProfileString16, W95a,
//  * with filled buffer ("****...") and section "set1" and key_name "1" valid:
//  * section	key_name	def_val		res	buffer
//  * "set1"	"1"		"x"		43	[data]
//  * "set1"	"1   "		"x"		43	[data]		(!)
//  * "set1"	"  1  "'	"x"		43	[data]		(!)
//  * "set1"	""		"x"		1	"x"
//  * "set1"	""		"x   "		1	"x"		(!)
//  * "set1"	""		"  x   "	3	"  x"		(!)
//  * "set1"	NULL		"x"		6	"1\02\03\0\0"
//  * "set1"	""		"x"		1	"x"
//  * NULL		"1"		"x"		0	""		(!)
//  * ""		"1"		"x"		1	"x"
//  * NULL		NULL		""		0	""
//  *
//  *
//  */
// static INT PROFILE_GetString( LPCWSTR section, LPCWSTR key_name,
// #ifdef __REACTOS__
//                               LPCWSTR def_val, LPWSTR buffer, DWORD len )
// #else
//                               LPCWSTR def_val, LPWSTR buffer, UINT len )
// #endif
// {
//     PROFILEKEY *key = NULL;
//     static const WCHAR empty_strW[] = { 0 };
//
//     if(!buffer || !len) return 0;
//
//     if (!def_val) def_val = empty_strW;
//     if (key_name)
//     {
//         key = PROFILE_Find( &MRUProfile[0]->section, section, key_name, FALSE, FALSE);
//         PROFILE_CopyEntry( buffer, (key && key->value) ? key->value : def_val,
//                            len, TRUE );
//         TRACE!("(%s,%s,%s): returning %s\n",
//               debugstr_w(section), debugstr_w(key_name),
//               debugstr_w(def_val), debugstr_w(buffer) );
//         return strlenW( buffer );
//     }
//     /* no "else" here ! */
//     if (section)
//     {
//         INT ret = PROFILE_GetSection(MRUProfile[0]->section, section, buffer, len, FALSE);
//         if (!buffer[0]) /* no luck -> def_val */
//         {
//             PROFILE_CopyEntry(buffer, def_val, len, TRUE);
//             ret = strlenW(buffer);
//         }
//         return ret;
//     }
//     buffer[0] = '\0';
//     return 0;
// }
//
//
// /***********************************************************************
//  *           PROFILE_SetString
//  *
//  * Set a profile string.
//  */
// static BOOL PROFILE_SetString( LPCWSTR section_name, LPCWSTR key_name,
//                                LPCWSTR value, BOOL create_always )
// {
//     if (!key_name)  /* Delete a whole section */
//     {
//         TRACE!("(%s)\n", debugstr_w(section_name));
//         MRUProfile[0]->changed |= PROFILE_DeleteSection( &MRUProfile[0]->section,
//                                                       section_name );
//         return TRUE;         /* Even if PROFILE_DeleteSection() has failed,
//                                 this is not an error on application's level.*/
//     }
//     else if (!value)  /* Delete a key */
//     {
//         TRACE!("(%s,%s)\n", debugstr_w(section_name), debugstr_w(key_name) );
//         MRUProfile[0]->changed |= PROFILE_DeleteKey( &MRUProfile[0]->section,
//                                                   section_name, key_name );
//         return TRUE;          /* same error handling as above */
//     }
//     else  /* Set the key value */
//     {
//         PROFILEKEY *key = PROFILE_Find(&MRUProfile[0]->section, section_name,
//                                         key_name, TRUE, create_always );
//         TRACE!("(%s,%s,%s):\n",
//               debugstr_w(section_name), debugstr_w(key_name), debugstr_w(value) );
//         if (!key) return FALSE;
//
//         /* strip the leading spaces. We can safely strip \n\r and
//          * friends too, they should not happen here anyway. */
//         while (PROFILE_isspaceW(*value)) value++;
//
//         if (key->value)
//         {
//             if (!strcmpW( key->value, value ))
//             {
//                 TRACE!("  no change needed\n" );
//                 return TRUE;  /* No change needed */
//             }
//             TRACE!("  replacing %s\n", debugstr_w(key->value) );
//             HeapFree( GetProcessHeap(), 0, key->value );
//         }
//         else TRACE!("  creating key\n" );
//         key->value = HeapAlloc( GetProcessHeap(), 0, (strlenW(value)+1) * sizeof(WCHAR) );
//         strcpyW( key->value, value );
//         MRUProfile[0]->changed = TRUE;
//     }
//     return TRUE;
// }
//
//
// /********************* API functions **********************************/
//
//
// /***********************************************************************
//  *           GetProfileIntA   (KERNEL32.@)
//  */
// UINT WINAPI GetProfileIntA( LPCSTR section, LPCSTR entry, INT def_val )
// {
//     return GetPrivateProfileIntA( section, entry, def_val, "win.ini" );
// }
//
// /***********************************************************************
//  *           GetProfileIntW   (KERNEL32.@)
//  */
// UINT WINAPI GetProfileIntW( LPCWSTR section, LPCWSTR entry, INT def_val )
// {
//     return GetPrivateProfileIntW( section, entry, def_val, wininiW );
// }
//
// /***********************************************************************
//  *           GetPrivateProfileStringW   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetPrivateProfileStringW( LPCWSTR section, LPCWSTR entry,
// 				     LPCWSTR def_val, LPWSTR buffer,
// 				     DWORD len, LPCWSTR filename )
// #else
// INT WINAPI GetPrivateProfileStringW( LPCWSTR section, LPCWSTR entry,
// 				     LPCWSTR def_val, LPWSTR buffer,
// 				     UINT len, LPCWSTR filename )
// #endif
// {
//     int		ret;
//     LPWSTR	defval_tmp = NULL;
//
//     TRACE!("%s,%s,%s,%p,%u,%s\n", debugstr_w(section), debugstr_w(entry),
//           debugstr_w(def_val), buffer, len, debugstr_w(filename));
//
//     /* strip any trailing ' ' of def_val. */
//     if (def_val)
//     {
//         LPCWSTR p = def_val + strlenW(def_val) - 1;
//
//         while (p > def_val && *p == ' ')
//             p--;
//
//         if (p >= def_val)
//         {
//             int vlen = (int)(p - def_val) + 1;
//
//             defval_tmp = HeapAlloc(GetProcessHeap(), 0, (vlen + 1) * sizeof(WCHAR));
//             memcpy(defval_tmp, def_val, vlen * sizeof(WCHAR));
//             defval_tmp[vlen] = '\0';
//             def_val = defval_tmp;
//         }
//     }
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (PROFILE_Open( filename, FALSE )) {
// 	if (section == NULL)
//             ret = PROFILE_GetSectionNames(buffer, len);
// 	else
// 	    /* PROFILE_GetString can handle the 'entry == NULL' case */
//             ret = PROFILE_GetString( section, entry, def_val, buffer, len );
//     } else if (buffer && def_val) {
//        lstrcpynW( buffer, def_val, len );
//        ret = strlenW( buffer );
//     }
//     else
//        ret = 0;
//
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//
//     HeapFree(GetProcessHeap(), 0, defval_tmp);
//
//     TRACE!("returning %s, %d\n", debugstr_w(buffer), ret);
//
//     return ret;
// }
//
// /***********************************************************************
//  *           GetPrivateProfileStringA   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetPrivateProfileStringA( LPCSTR section, LPCSTR entry,
// 				     LPCSTR def_val, LPSTR buffer,
// 				     DWORD len, LPCSTR filename )
// #else
// INT WINAPI GetPrivateProfileStringA( LPCSTR section, LPCSTR entry,
// 				     LPCSTR def_val, LPSTR buffer,
// 				     UINT len, LPCSTR filename )
// #endif
// {
//     UNICODE_STRING sectionW, entryW, def_valW, filenameW;
//     LPWSTR bufferW;
//     INT retW, ret = 0;
//
//     bufferW = buffer ? HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR)) : NULL;
//     if (section) RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     else sectionW.Buffer = NULL;
//     if (entry) RtlCreateUnicodeStringFromAsciiz(&entryW, entry);
//     else entryW.Buffer = NULL;
//     if (def_val) RtlCreateUnicodeStringFromAsciiz(&def_valW, def_val);
//     else def_valW.Buffer = NULL;
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     retW = GetPrivateProfileStringW( sectionW.Buffer, entryW.Buffer,
//                                      def_valW.Buffer, bufferW, len,
//                                      filenameW.Buffer);
//     if (len && buffer)
//     {
//         if (retW)
//         {
//             ret = WideCharToMultiByte(CP_ACP, 0, bufferW, retW, buffer, len - 1, NULL, NULL);
//             if (!ret)
//                 ret = len - 1;
//         }
//         buffer[ret] = 0;
//     }
//
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&entryW);
//     RtlFreeUnicodeString(&def_valW);
//     RtlFreeUnicodeString(&filenameW);
//     HeapFree(GetProcessHeap(), 0, bufferW);
//     return ret;
// }
//
// /***********************************************************************
//  *           GetProfileStringA   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetProfileStringA( LPCSTR section, LPCSTR entry, LPCSTR def_val,
// 			      LPSTR buffer, DWORD len )
// #else
// INT WINAPI GetProfileStringA( LPCSTR section, LPCSTR entry, LPCSTR def_val,
// 			      LPSTR buffer, UINT len )
// #endif
// {
//     return GetPrivateProfileStringA( section, entry, def_val,
//                                      buffer, len, "win.ini" );
// }
//
// /***********************************************************************
//  *           GetProfileStringW   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetProfileStringW( LPCWSTR section, LPCWSTR entry,
// 			      LPCWSTR def_val, LPWSTR buffer, DWORD len )
// #else
// INT WINAPI GetProfileStringW( LPCWSTR section, LPCWSTR entry,
// 			      LPCWSTR def_val, LPWSTR buffer, UINT len )
// #endif
// {
//     return GetPrivateProfileStringW( section, entry, def_val,
// 				     buffer, len, wininiW );
// }
//
// /***********************************************************************
//  *           WriteProfileStringA   (KERNEL32.@)
//  */
// BOOL WINAPI WriteProfileStringA( LPCSTR section, LPCSTR entry,
// 				 LPCSTR string )
// {
//     return WritePrivateProfileStringA( section, entry, string, "win.ini" );
// }
//
// /***********************************************************************
//  *           WriteProfileStringW   (KERNEL32.@)
//  */
// BOOL WINAPI WriteProfileStringW( LPCWSTR section, LPCWSTR entry,
//                                      LPCWSTR string )
// {
//     return WritePrivateProfileStringW( section, entry, string, wininiW );
// }
//
//
// /***********************************************************************
//  *           GetPrivateProfileIntW   (KERNEL32.@)
//  */
// UINT WINAPI GetPrivateProfileIntW( LPCWSTR section, LPCWSTR entry,
//                                    INT def_val, LPCWSTR filename )
// {
//     WCHAR buffer[30];
//     UNICODE_STRING bufferW;
//     ULONG result;
//
//     if (GetPrivateProfileStringW( section, entry, emptystringW,
//                                    buffer, sizeof(buffer)/sizeof(WCHAR),
//                                    filename ) == 0)
//         return def_val;
//
//     /* FIXME: if entry can be found but it's empty, then Win16 is
//      * supposed to return 0 instead of def_val ! Difficult/problematic
//      * to implement (every other failure also returns zero buffer),
//      * thus wait until testing framework avail for making sure nothing
//      * else gets broken that way. */
//     if (!buffer[0]) return (UINT)def_val;
//
//     RtlInitUnicodeString( &bufferW, buffer );
//     RtlUnicodeStringToInteger( &bufferW, 0, &result);
//     return result;
// }
//
// /***********************************************************************
//  *           GetPrivateProfileIntA   (KERNEL32.@)
//  *
//  * FIXME: rewrite using unicode
//  */
// UINT WINAPI GetPrivateProfileIntA( LPCSTR section, LPCSTR entry,
// 				   INT def_val, LPCSTR filename )
// {
//     UNICODE_STRING entryW, filenameW, sectionW;
//     UINT res;
//     if(entry) RtlCreateUnicodeStringFromAsciiz(&entryW, entry);
//     else entryW.Buffer = NULL;
//     if(filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//     if(section) RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     else sectionW.Buffer = NULL;
//     res = GetPrivateProfileIntW(sectionW.Buffer, entryW.Buffer, def_val,
//                                 filenameW.Buffer);
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&filenameW);
//     RtlFreeUnicodeString(&entryW);
//     return res;
// }
//
// /***********************************************************************
//  *           GetPrivateProfileSectionW   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetPrivateProfileSectionW( LPCWSTR section, LPWSTR buffer,
// #else
// INT WINAPI GetPrivateProfileSectionW( LPCWSTR section, LPWSTR buffer,
// #endif
// 				      DWORD len, LPCWSTR filename )
// {
//     int ret = 0;
//
//     if (!section || !buffer)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     TRACE!("(%s, %p, %d, %s)\n", debugstr_w(section), buffer, len, debugstr_w(filename));
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (PROFILE_Open( filename, FALSE ))
//         ret = PROFILE_GetSection(MRUProfile[0]->section, section, buffer, len, TRUE);
//
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//
//     return ret;
// }
//
// /***********************************************************************
//  *           GetPrivateProfileSectionA   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetPrivateProfileSectionA( LPCSTR section, LPSTR buffer,
// #else
// INT WINAPI GetPrivateProfileSectionA( LPCSTR section, LPSTR buffer,
// #endif
//                                       DWORD len, LPCSTR filename )
// {
//     UNICODE_STRING sectionW, filenameW;
//     LPWSTR bufferW;
//     INT retW, ret = 0;
//
//     if (!section || !buffer)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     bufferW = HeapAlloc(GetProcessHeap(), 0, len * 2 * sizeof(WCHAR));
//     RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     retW = GetPrivateProfileSectionW(sectionW.Buffer, bufferW, len * 2, filenameW.Buffer);
//     if (retW)
//     {
//         if (retW == len * 2 - 2) retW++;  /* overflow */
//         ret = WideCharToMultiByte(CP_ACP, 0, bufferW, retW + 1, buffer, len, NULL, NULL);
//         if (!ret || ret == len)  /* overflow */
//         {
//             ret = len - 2;
//             buffer[len-2] = 0;
//             buffer[len-1] = 0;
//         }
//         else ret--;
//     }
//     else
//     {
//         buffer[0] = 0;
//         buffer[1] = 0;
//     }
//
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&filenameW);
//     HeapFree(GetProcessHeap(), 0, bufferW);
//     return ret;
// }
//
// /***********************************************************************
//  *           GetProfileSectionA   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetProfileSectionA( LPCSTR section, LPSTR buffer, DWORD len )
// #else
// INT WINAPI GetProfileSectionA( LPCSTR section, LPSTR buffer, DWORD len )
// #endif
// {
//     return GetPrivateProfileSectionA( section, buffer, len, "win.ini" );
// }
//
// /***********************************************************************
//  *           GetProfileSectionW   (KERNEL32.@)
//  */
// #ifdef __REACTOS__
// DWORD WINAPI GetProfileSectionW( LPCWSTR section, LPWSTR buffer, DWORD len )
// #else
// INT WINAPI GetProfileSectionW( LPCWSTR section, LPWSTR buffer, DWORD len )
// #endif
// {
//     return GetPrivateProfileSectionW( section, buffer, len, wininiW );
// }
//
//
// /***********************************************************************
//  *           WritePrivateProfileStringW   (KERNEL32.@)
//  */
// BOOL WINAPI WritePrivateProfileStringW( LPCWSTR section, LPCWSTR entry,
// 					LPCWSTR string, LPCWSTR filename )
// {
//     BOOL ret = FALSE;
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (!section && !entry && !string) /* documented "file flush" case */
//     {
//         if (!filename || PROFILE_Open( filename, TRUE ))
//         {
//             if (MRUProfile[0]) PROFILE_ReleaseFile();  /* always return FALSE in this case */
//         }
//     }
//     else if (PROFILE_Open( filename, TRUE ))
//     {
//         if (!section) {
//             SetLastError(ERROR_FILE_NOT_FOUND);
//         } else {
//             ret = PROFILE_SetString( section, entry, string, FALSE);
//             if (ret) ret = PROFILE_FlushFile();
//         }
//     }
//
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//     return ret;
// }
//
// /***********************************************************************
//  *           WritePrivateProfileStringA   (KERNEL32.@)
//  */
// BOOL WINAPI DECLSPEC_HOTPATCH WritePrivateProfileStringA( LPCSTR section, LPCSTR entry,
//                                                           LPCSTR string, LPCSTR filename )
// {
//     UNICODE_STRING sectionW, entryW, stringW, filenameW;
//     BOOL ret;
//
//     if (section) RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     else sectionW.Buffer = NULL;
//     if (entry) RtlCreateUnicodeStringFromAsciiz(&entryW, entry);
//     else entryW.Buffer = NULL;
//     if (string) RtlCreateUnicodeStringFromAsciiz(&stringW, string);
//     else stringW.Buffer = NULL;
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     ret = WritePrivateProfileStringW(sectionW.Buffer, entryW.Buffer,
//                                      stringW.Buffer, filenameW.Buffer);
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&entryW);
//     RtlFreeUnicodeString(&stringW);
//     RtlFreeUnicodeString(&filenameW);
//     return ret;
// }
//
// /***********************************************************************
//  *           WritePrivateProfileSectionW   (KERNEL32.@)
//  */
// BOOL WINAPI WritePrivateProfileSectionW( LPCWSTR section,
//                                          LPCWSTR string, LPCWSTR filename )
// {
//     BOOL ret = FALSE;
//     LPWSTR p;
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (!section && !string)
//     {
//         if (!filename || PROFILE_Open( filename, TRUE ))
//         {
//             if (MRUProfile[0]) PROFILE_ReleaseFile();  /* always return FALSE in this case */
//         }
//     }
//     else if (PROFILE_Open( filename, TRUE )) {
//         if (!string) {/* delete the named section*/
// 	    ret = PROFILE_SetString(section,NULL,NULL, FALSE);
//         } else {
// 	    PROFILE_DeleteAllKeys(section);
// 	    ret = TRUE;
// 	    while(*string && ret) {
//                 LPWSTR buf = HeapAlloc( GetProcessHeap(), 0, (strlenW(string)+1) * sizeof(WCHAR) );
//                 strcpyW( buf, string );
//                 if((p = strchrW( buf, '='))) {
//                     *p='\0';
//                     ret = PROFILE_SetString( section, buf, p+1, TRUE);
//                 }
//                 HeapFree( GetProcessHeap(), 0, buf );
//                 string += strlenW(string)+1;
//             }
//         }
//         if (ret) ret = PROFILE_FlushFile();
//     }
//
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//     return ret;
// }
//
// /***********************************************************************
//  *           WritePrivateProfileSectionA   (KERNEL32.@)
//  */
// BOOL WINAPI WritePrivateProfileSectionA( LPCSTR section,
//                                          LPCSTR string, LPCSTR filename)
//
// {
//     UNICODE_STRING sectionW, filenameW;
//     LPWSTR stringW;
//     BOOL ret;
//
//     if (string)
//     {
//         INT lenA, lenW;
//         LPCSTR p = string;
//
//         while(*p) p += strlen(p) + 1;
//         lenA = p - string + 1;
//         lenW = MultiByteToWideChar(CP_ACP, 0, string, lenA, NULL, 0);
//         if ((stringW = HeapAlloc(GetProcessHeap(), 0, lenW * sizeof(WCHAR))))
//             MultiByteToWideChar(CP_ACP, 0, string, lenA, stringW, lenW);
//     }
//     else stringW = NULL;
//     if (section) RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     else sectionW.Buffer = NULL;
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     ret = WritePrivateProfileSectionW(sectionW.Buffer, stringW, filenameW.Buffer);
//
//     HeapFree(GetProcessHeap(), 0, stringW);
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&filenameW);
//     return ret;
// }
//
// /***********************************************************************
//  *           WriteProfileSectionA   (KERNEL32.@)
//  */
// BOOL WINAPI WriteProfileSectionA( LPCSTR section, LPCSTR keys_n_values)
//
// {
//     return WritePrivateProfileSectionA( section, keys_n_values, "win.ini");
// }
//
// /***********************************************************************
//  *           WriteProfileSectionW   (KERNEL32.@)
//  */
// BOOL WINAPI WriteProfileSectionW( LPCWSTR section, LPCWSTR keys_n_values)
// {
//    return WritePrivateProfileSectionW(section, keys_n_values, wininiW);
// }
//
//
// /***********************************************************************
//  *           GetPrivateProfileSectionNamesW  (KERNEL32.@)
//  *
//  * Returns the section names contained in the specified file.
//  * FIXME: Where do we find this file when the path is relative?
//  * The section names are returned as a list of strings with an extra
//  * '\0' to mark the end of the list. Except for that the behavior
//  * depends on the Windows version.
//  *
//  * Win95:
//  * - if the buffer is 0 or 1 character long then it is as if it was of
//  *   infinite length.
//  * - otherwise, if the buffer is too small only the section names that fit
//  *   are returned.
//  * - note that this means if the buffer was too small to return even just
//  *   the first section name then a single '\0' will be returned.
//  * - the return value is the number of characters written in the buffer,
//  *   except if the buffer was too small in which case len-2 is returned
//  *
//  * Win2000:
//  * - if the buffer is 0, 1 or 2 characters long then it is filled with
//  *   '\0' and the return value is 0
//  * - otherwise if the buffer is too small then the first section name that
//  *   does not fit is truncated so that the string list can be terminated
//  *   correctly (double '\0')
//  * - the return value is the number of characters written in the buffer
//  *   except for the trailing '\0'. If the buffer is too small, then the
//  *   return value is len-2
//  * - Win2000 has a bug that triggers when the section names and the
//  *   trailing '\0' fit exactly in the buffer. In that case the trailing
//  *   '\0' is missing.
//  *
//  * Wine implements the observed Win2000 behavior (except for the bug).
//  *
//  * Note that when the buffer is big enough then the return value may be any
//  * value between 1 and len-1 (or len in Win95), including len-2.
//  */
// DWORD WINAPI GetPrivateProfileSectionNamesW( LPWSTR buffer, DWORD size,
// 					     LPCWSTR filename)
// {
//     DWORD ret = 0;
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (PROFILE_Open( filename, FALSE ))
//         ret = PROFILE_GetSectionNames(buffer, size);
//
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//
//     return ret;
// }
//
//
// /***********************************************************************
//  *           GetPrivateProfileSectionNamesA  (KERNEL32.@)
//  */
// DWORD WINAPI GetPrivateProfileSectionNamesA( LPSTR buffer, DWORD size,
// 					     LPCSTR filename)
// {
//     UNICODE_STRING filenameW;
//     LPWSTR bufferW;
//     INT retW, ret = 0;
//
//     bufferW = buffer ? HeapAlloc(GetProcessHeap(), 0, size * sizeof(WCHAR)) : NULL;
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     retW = GetPrivateProfileSectionNamesW(bufferW, size, filenameW.Buffer);
//     if (retW && size)
//     {
//         ret = WideCharToMultiByte(CP_ACP, 0, bufferW, retW+1, buffer, size-1, NULL, NULL);
//         if (!ret)
//         {
//             ret = size-2;
//             buffer[size-1] = 0;
//         }
//         else
//           ret = ret-1;
//     }
//     else if(size)
//         buffer[0] = '\0';
//
//     RtlFreeUnicodeString(&filenameW);
//     HeapFree(GetProcessHeap(), 0, bufferW);
//     return ret;
// }
//
// /***********************************************************************
//  *           GetPrivateProfileStructW (KERNEL32.@)
//  *
//  * Should match Win95's behaviour pretty much
//  */
// BOOL WINAPI GetPrivateProfileStructW (LPCWSTR section, LPCWSTR key,
//                                       LPVOID buf, UINT len, LPCWSTR filename)
// {
//     BOOL	ret = FALSE;
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (PROFILE_Open( filename, FALSE )) {
//         PROFILEKEY *k = PROFILE_Find ( &MRUProfile[0]->section, section, key, FALSE, FALSE);
// 	if (k) {
// 	    TRACE!("value (at %p): %s\n", k->value, debugstr_w(k->value));
// 	    if (((strlenW(k->value) - 2) / 2) == len)
// 	    {
// 		LPWSTR end, p;
// 		BOOL valid = TRUE;
// 		WCHAR c;
// 		DWORD chksum = 0;
//
// 	        end  = k->value + strlenW(k->value); /* -> '\0' */
// 	        /* check for invalid chars in ASCII coded hex string */
// 	        for (p=k->value; p < end; p++)
// 		{
//                     if (!isxdigitW(*p))
// 		    {
// 			WARN!("invalid char '%x' in file %s->[%s]->%s !\n",
//                              *p, debugstr_w(filename), debugstr_w(section), debugstr_w(key));
// 		        valid = FALSE;
// 		        break;
// 		    }
// 		}
// 		if (valid)
// 		{
// 		    BOOL highnibble = TRUE;
// 		    BYTE b = 0, val;
//                     LPBYTE binbuf = buf;
//
// 	            end -= 2; /* don't include checksum in output data */
// 	            /* translate ASCII hex format into binary data */
//                     for (p=k->value; p < end; p++)
//             	    {
// 	        	c = toupperW(*p);
// 			val = (c > '9') ?
// 				(c - 'A' + 10) : (c - '0');
//
// 			if (highnibble)
// 		    	    b = val << 4;
// 			else
// 			{
// 		    	    b += val;
// 		    	    *binbuf++ = b; /* feed binary data into output */
// 		    	    chksum += b; /* calculate checksum */
// 			}
// 			highnibble ^= 1; /* toggle */
//             	    }
// 		    /* retrieve stored checksum value */
// 		    c = toupperW(*p++);
// 		    b = ( (c > '9') ? (c - 'A' + 10) : (c - '0') ) << 4;
// 		    c = toupperW(*p);
// 		    b +=  (c > '9') ? (c - 'A' + 10) : (c - '0');
// 	            if (b == (chksum & 0xff)) /* checksums match ? */
//                         ret = TRUE;
//                 }
//             }
// 	}
//     }
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//
//     return ret;
// }
//
// /***********************************************************************
//  *           GetPrivateProfileStructA (KERNEL32.@)
//  */
// BOOL WINAPI GetPrivateProfileStructA (LPCSTR section, LPCSTR key,
// 				      LPVOID buffer, UINT len, LPCSTR filename)
// {
//     UNICODE_STRING sectionW, keyW, filenameW;
//     INT ret;
//
//     if (section) RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     else sectionW.Buffer = NULL;
//     if (key) RtlCreateUnicodeStringFromAsciiz(&keyW, key);
//     else keyW.Buffer = NULL;
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     ret = GetPrivateProfileStructW(sectionW.Buffer, keyW.Buffer, buffer, len,
//                                    filenameW.Buffer);
//     /* Do not translate binary data. */
//
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&keyW);
//     RtlFreeUnicodeString(&filenameW);
//     return ret;
// }
//
//
//
// /***********************************************************************
//  *           WritePrivateProfileStructW (KERNEL32.@)
//  */
// BOOL WINAPI WritePrivateProfileStructW (LPCWSTR section, LPCWSTR key,
//                                         LPVOID buf, UINT bufsize, LPCWSTR filename)
// {
//     BOOL ret = FALSE;
//     LPBYTE binbuf;
//     LPWSTR outstring, p;
//     DWORD sum = 0;
//
//     if (!section && !key && !buf)  /* flush the cache */
//         return WritePrivateProfileStringW( NULL, NULL, NULL, filename );
//
//     /* allocate string buffer for hex chars + checksum hex char + '\0' */
//     outstring = HeapAlloc( GetProcessHeap(), 0, (bufsize*2 + 2 + 1) * sizeof(WCHAR) );
//     p = outstring;
//     for (binbuf = (LPBYTE)buf; binbuf < (LPBYTE)buf+bufsize; binbuf++) {
//       *p++ = hex[*binbuf >> 4];
//       *p++ = hex[*binbuf & 0xf];
//       sum += *binbuf;
//     }
//     /* checksum is sum & 0xff */
//     *p++ = hex[(sum & 0xf0) >> 4];
//     *p++ = hex[sum & 0xf];
//     *p++ = '\0';
//
//     RtlEnterCriticalSection( &PROFILE_CritSect );
//
//     if (PROFILE_Open( filename, TRUE )) {
//         ret = PROFILE_SetString( section, key, outstring, FALSE);
//         if (ret) ret = PROFILE_FlushFile();
//     }
//
//     RtlLeaveCriticalSection( &PROFILE_CritSect );
//
//     HeapFree( GetProcessHeap(), 0, outstring );
//
//     return ret;
// }
//
// /***********************************************************************
//  *           WritePrivateProfileStructA (KERNEL32.@)
//  */
// BOOL WINAPI WritePrivateProfileStructA (LPCSTR section, LPCSTR key,
// 					LPVOID buf, UINT bufsize, LPCSTR filename)
// {
//     UNICODE_STRING sectionW, keyW, filenameW;
//     INT ret;
//
//     if (section) RtlCreateUnicodeStringFromAsciiz(&sectionW, section);
//     else sectionW.Buffer = NULL;
//     if (key) RtlCreateUnicodeStringFromAsciiz(&keyW, key);
//     else keyW.Buffer = NULL;
//     if (filename) RtlCreateUnicodeStringFromAsciiz(&filenameW, filename);
//     else filenameW.Buffer = NULL;
//
//     /* Do not translate binary data. */
//     ret = WritePrivateProfileStructW(sectionW.Buffer, keyW.Buffer, buf, bufsize,
//                                      filenameW.Buffer);
//
//     RtlFreeUnicodeString(&sectionW);
//     RtlFreeUnicodeString(&keyW);
//     RtlFreeUnicodeString(&filenameW);
//     return ret;
// }
//
//
// /***********************************************************************
//  *           OpenProfileUserMapping   (KERNEL32.@)
//  */
// BOOL WINAPI OpenProfileUserMapping(void) {
//     FIXME!("(), stub!\n");
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /***********************************************************************
//  *           CloseProfileUserMapping   (KERNEL32.@)
//  */
// BOOL WINAPI CloseProfileUserMapping(void) {
//     FIXME!("(), stub!\n");
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
