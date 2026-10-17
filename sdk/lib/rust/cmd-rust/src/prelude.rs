//! cmd-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const __WINE_DBCL_ERR: i32 = 1; // clang, trace.c
pub const __WINE_DBCL_FIXME: i32 = 0; // clang, trace.c
pub const __WINE_DBCL_TRACE: i32 = 3; // clang, trace.c
pub const __WINE_DBCL_WARN: i32 = 2; // clang, trace.c
// ==== end port facts ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BAT_TYPE: _BATCH_TYPE = _BATCH_TYPE_BAT_TYPE; // island enum
pub const CMD_TYPE: _BATCH_TYPE = _BATCH_TYPE_CMD_TYPE; // island enum
pub const C_AND: _COMMAND_TYPE = _COMMAND_TYPE_C_AND; // island enum
pub const C_BLOCK: _COMMAND_TYPE = _COMMAND_TYPE_C_BLOCK; // island enum
pub const C_COMMAND: _COMMAND_TYPE = _COMMAND_TYPE_C_COMMAND; // island enum
pub const C_FOR: _COMMAND_TYPE = _COMMAND_TYPE_C_FOR; // island enum
pub const C_IF: _COMMAND_TYPE = _COMMAND_TYPE_C_IF; // island enum
pub const C_MULTI: _COMMAND_TYPE = _COMMAND_TYPE_C_MULTI; // island enum
pub const C_OR: _COMMAND_TYPE = _COMMAND_TYPE_C_OR; // island enum
pub const C_PIPE: _COMMAND_TYPE = _COMMAND_TYPE_C_PIPE; // island enum
pub const C_QUIET: _COMMAND_TYPE = _COMMAND_TYPE_C_QUIET; // island enum
pub const C_REM: _COMMAND_TYPE = _COMMAND_TYPE_C_REM; // island enum
pub const FOREGROUND_WHITE: u32 = 7; // usetup-rust/src/types_amd64.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY; // access-rust/src/prelude.rs
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY; // console-rust/src/prelude.rs
pub const IF_CMDEXTVERSION: _IF_OPERATOR = _IF_OPERATOR_IF_CMDEXTVERSION; // island enum
pub const IF_DEFINED: _IF_OPERATOR = _IF_OPERATOR_IF_DEFINED; // island enum
pub const IF_EQU: _IF_OPERATOR = _IF_OPERATOR_IF_EQU; // island enum
pub const IF_ERRORLEVEL: _IF_OPERATOR = _IF_OPERATOR_IF_ERRORLEVEL; // island enum
pub const IF_EXIST: _IF_OPERATOR = _IF_OPERATOR_IF_EXIST; // island enum
pub const IF_GEQ: _IF_OPERATOR = _IF_OPERATOR_IF_GEQ; // island enum
pub const IF_GTR: _IF_OPERATOR = _IF_OPERATOR_IF_GTR; // island enum
pub const IF_LEQ: _IF_OPERATOR = _IF_OPERATOR_IF_LEQ; // island enum
pub const IF_LSS: _IF_OPERATOR = _IF_OPERATOR_IF_LSS; // island enum
pub const IF_NEQ: _IF_OPERATOR = _IF_OPERATOR_IF_NEQ; // island enum
pub const IF_STRINGEQ: _IF_OPERATOR = _IF_OPERATOR_IF_STRINGEQ; // island enum
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // apphelp-rust/src/prelude.rs
pub const INVALID_SET_FILE_POINTER: u32 = 4294967295; // msvcrt-sys/src/bindgen_amd64.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // cacls-rust/src/prelude.rs
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // comctl32-rust/src/prelude.rs
pub const NONE: _BATCH_TYPE = _BATCH_TYPE_NONE; // island enum
pub const REDIR_APPEND: _REDIR_MODE = _REDIR_MODE_REDIR_APPEND; // island enum
pub const REDIR_READ: _REDIR_MODE = _REDIR_MODE_REDIR_READ; // island enum
pub const REDIR_WRITE: _REDIR_MODE = _REDIR_MODE_REDIR_WRITE; // island enum
pub const STD_ERROR_HANDLE: u32 = 4294967284; // diskpart-rust/src/prelude.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // diskpart-rust/src/prelude.rs
pub const STD_OUTPUT_HANDLE: u32 = 4294967285; // diskpart-rust/src/prelude.rs
extern "C" {
    pub fn ConClearScreen(Screen: *const CON_SCREEN); // conutils-rust/src/conutils/screen.rs
    pub fn ConMsgPrintfV(Stream: *mut CON_STREAM, dwFlags: u32, lpSource: *const c_void, dwMessageId: u32, dwLanguageId: u32, Arguments: *mut va_list,) -> i32; // conutils-rust/src/conutils/outstream.rs
    pub fn ConPrintfV(Stream: *mut CON_STREAM, szStr: *const u16, args: va_list,) -> i32; // conutils-rust/src/conutils/outstream.rs
    pub fn ConResPaging(Pager: *mut CON_PAGER, PagePrompt: PAGE_PROMPT, StartPaging: i32, uID: u32,) -> i32; // conutils-rust/src/conutils/pager.rs
    pub fn ConWritePaging(Pager: *mut CON_PAGER, PagePrompt: PAGE_PROMPT, StartPaging: i32, szStr: *const u16, len: u32,) -> i32; // conutils-rust/src/conutils/pager.rs
    pub fn IsConsoleHandle(hHandle: HANDLE) -> i32; // conutils-rust/src/conutils/utils.rs
    pub fn __wine_dbg_get_channel_flags(channel: *mut __wine_debug_channel) -> ::core::ffi::c_uchar; // winegdi-rust/src/types_i386.rs
    pub fn __wine_dbg_set_functions( new_funcs: *const __wine_debug_functions, old_funcs: *mut __wine_debug_functions, size: usize, ); // winegdi-rust/src/types_i386.rs
    pub fn _errno() -> *mut c_int; // crt-shared-rust/src/prelude.rs
    pub fn _getdrive() -> ::core::ffi::c_int; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _pclose(file: *mut FILE) -> i32; // msvcrt-crt-rust/src/msvcrt/process.rs
    pub fn _vswprintf(str_: *mut u16, format: *const u16, args: *mut u8) -> i32; // msvcrt-crt-rust/src/printf_shells.rs
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn _wcsupr(s: LPWSTR) -> LPWSTR; // shell32-rust/src/prelude.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // dnsrslvr-rust/src/prelude.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn isalnum(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isalpha(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn iswctype(c: u16, mask: u16) -> c_int; // ucrt-rust/src/ctype.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn memchr(s: *const c_void, c: i32, n: usize) -> *mut c_void; // secur32-rust/src/externs.rs
    pub fn mk_wcwidth_cjk(ucs: WCHAR) -> i32; // winsrv-rust/src/consrv/shim.rs
    pub fn qsort(base: *mut c_void, num: usize, width: usize, comp: Compare); // ucrt-rust/src/stdlib/qsort.rs
    pub fn rand() -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn strlen(s: *const c_char) -> usize; // browser-rust/src/prelude.rs
    pub fn tolower(c: c_int) -> c_int; // kernel32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleCtrlHandler(HandlerRoutine: PHANDLER_ROUTINE, Add: BOOL) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn AllocConsole() -> BOOL; // kernel32-rust/src/console/console.rs
    pub fn Beep(arg1: DWORD, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreatePipe( arg1: PHANDLE, arg2: PHANDLE, arg3: LPSECURITY_ATTRIBUTES, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DuplicateHandle( arg1: HANDLE, arg2: HANDLE, arg3: HANDLE, arg4: PHANDLE, arg5: DWORD, arg6: BOOL, arg7: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn ExitProcess(arg1: UINT) -> !; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FillConsoleOutputAttribute(hOutput: *mut c_void, wAttribute: u16, nLength: u32, dwWriteCoord: COORD, lpNumberOfAttrsWritten: *mut u32) -> BOOL; // conutils-rust/src/conutils/screen.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FlushConsoleInputBuffer(hConsoleInput: HANDLE) -> BOOL; // kernel32-rust/src/file/fileinfo.rs
    pub fn FreeConsole() -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetConsoleCP() -> UINT; // winsrv-sys/src/bindgen_i386.rs
    pub fn GetConsoleOutputCP() -> UINT; // winsrv-sys/src/bindgen_i386.rs
    pub fn GetConsoleScreenBufferInfo( hConsoleOutput: HANDLE, lpConsoleScreenBufferInfo: PCONSOLE_SCREEN_BUFFER_INFO, ) -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetExitCodeProcess(arg1: HANDLE, arg2: PDWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileType(arg1: HANDLE) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLocalTime(arg1: LPSYSTEMTIME); // aclui-rust/src/types_i386.rs
    pub fn GetNumaHighestNodeNumber(HighestNodeNumber: PULONG,) -> BOOL; // kernel32-rust/src/client/sysinfo.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSystemTime(lpSystemTime: LPSYSTEMTIME); // aclui-rust/src/types_i386.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GlobalMemoryStatus(arg1: LPMEMORYSTATUS); // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MessageBeep(uType: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ResumeThread(arg1: HANDLE) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetConsoleCursorInfo( hConsoleOutput: HANDLE, lpConsoleCursorInfo: *const CONSOLE_CURSOR_INFO, ) -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn SetConsoleCursorPosition(hConsoleOutput: HANDLE, dwCursorPosition: COORD) -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn SetConsoleTextAttribute(hConsoleOutput: HANDLE, wAttributes: WORD) -> BOOL; // winsrv-sys/src/bindgen_i386.rs
    pub fn SetFileApisToOEM(); // aclui-rust/src/types_i386.rs
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetFileTime( arg1: HANDLE, arg2: *const FILETIME, arg3: *const FILETIME, arg4: *const FILETIME, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetHandleInformation(arg1: HANDLE, arg2: DWORD, arg3: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SetLocalTime(arg1: *const SYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessAffinityMask(arg1: HANDLE, arg2: DWORD_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetStdHandle(arg1: DWORD, arg2: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SystemTimeToFileTime(arg1: *const SYSTEMTIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TryEnterCriticalSection(arg1: LPCRITICAL_SECTION) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn VirtualAlloc(arg1: PVOID, arg2: SIZE_T, arg3: DWORD, arg4: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn VirtualFree(arg1: PVOID, arg2: SIZE_T, arg3: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForMultipleObjects( nCount: DWORD, lpHandles: *const HANDLE, bWaitAll: BOOL, dwMilliseconds: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// LPDIRSWITCHFLAGS: c2rust, dir.c
pub type LPDIRSWITCHFLAGS = *mut _DirSwitchesFlags;
// __wine_debug_functions: c2rust, trace.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __wine_debug_functions {
    pub get_temp_buffer: Option<unsafe extern "C" fn(size_t) -> *mut ::core::ffi::c_char>,
    pub release_temp_buffer: Option<unsafe extern "C" fn(*mut ::core::ffi::c_char, size_t) -> ()>,
    pub dbgstr_an: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
        ) -> *const ::core::ffi::c_char,
    >,
    pub dbgstr_wn: Option<
        unsafe extern "C" fn(*const WCHAR, ::core::ffi::c_int) -> *const ::core::ffi::c_char,
    >,
    pub dbg_vprintf: Option<
        unsafe extern "C" fn(*const ::core::ffi::c_char, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
    pub dbg_vlog: Option<
        unsafe extern "C" fn(
            __wine_debug_class,
            *mut __wine_debug_channel,
            *const ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn ConGetTextWidthW(pszText: PCWSTR) -> SIZE_T; // dir.c
    pub fn _itow(
        _Value: ::core::ffi::c_int,
        _Dest: *mut wchar_t,
        _Radix: ::core::ffi::c_int,
    ) -> *mut wchar_t; // cmd.c
    pub fn _snwprintf(
        _Dest: *mut wchar_t,
        _Count: size_t,
        _Format: *const wchar_t,
        ...
    ) -> ::core::ffi::c_int; // parser.c
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // cmd.c
    pub fn _wchdir(_Path: *const wchar_t) -> ::core::ffi::c_int; // cmd.c
    pub fn _wcsicmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // batch.c
    pub fn _wcslwr(_String: *mut wchar_t) -> *mut wchar_t; // dir.c
    pub fn _wcsnset(_Str: *mut wchar_t, _Val: wchar_t, _MaxCount: size_t) -> *mut wchar_t; // cmdinput.c
    pub fn _wfopen(_Filename: *const wchar_t, _Mode: *const wchar_t) -> *mut FILE; // for.c
    pub fn _wgetdcwd(
        _Drive: ::core::ffi::c_int,
        _DstBuf: *mut wchar_t,
        _SizeInWords: ::core::ffi::c_int,
    ) -> *mut wchar_t; // internal.c
    pub fn _wpopen(_Command: *const wchar_t, _Mode: *const wchar_t) -> *mut FILE; // for.c
    pub fn _wtoi(_Str: *const wchar_t) -> ::core::ffi::c_int; // choice.c
    pub fn _wtol(_Str: *const wchar_t) -> ::core::ffi::c_long; // cmd.c
    pub fn fgetws(
        _Dst: *mut wchar_t,
        _SizeInWords: ::core::ffi::c_int,
        _File: *mut FILE,
    ) -> *mut wchar_t; // for.c
    pub fn towlower(_C: wint_t) -> wint_t; // alias.c
    pub fn towupper(_C: wint_t) -> wint_t; // cmd.c
    pub fn wcscat(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // batch.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // alias.c
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // cmd.c
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // cmd.c
    pub fn wcscspn(_Str: *const wchar_t, _Control: *const wchar_t) -> size_t; // alias.c
    pub fn wcsncat(_Dest: *mut wchar_t, _Source: *const wchar_t, _Count: size_t) -> *mut wchar_t; // cmd.c
    pub fn wcsncmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // alias.c
    pub fn wcsncpy(_Dest: *mut wchar_t, _Source: *const wchar_t, _Count: size_t) -> *mut wchar_t; // batch.c
    pub fn wcspbrk(_Str: *const wchar_t, _Control: *const wchar_t) -> *mut wchar_t; // cmd.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // batch.c
    pub fn wcsspn(_Str: *const wchar_t, _Control: *const wchar_t) -> size_t; // alias.c
    pub fn wcsstr(_Str: *const wchar_t, _SubStr: *const wchar_t) -> *mut wchar_t; // color.c
    pub fn wcstok(_Str: *mut wchar_t, _Delim: *const wchar_t) -> *mut wchar_t; // where.c
    pub fn wcstol(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long; // choice.c
    pub fn wcstoul(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong; // cmd.c
}
extern "system" {
    pub fn AddConsoleAliasW(Source: LPCWSTR, Target: LPCWSTR, ExeName: LPCWSTR) -> BOOL; // alias.c
    pub fn CopyFileExW(
        _: LPCWSTR,
        _: LPCWSTR,
        _: LPPROGRESS_ROUTINE,
        _: LPVOID,
        _: LPBOOL,
        _: DWORD,
    ) -> BOOL; // copy.c
    pub fn CopyFileW(lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL) -> BOOL; // move.c
    pub fn CreateDirectoryW(lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES) -> BOOL; // internal.c
    pub fn CreateFileW(
        _: LPCWSTR,
        _: DWORD,
        _: DWORD,
        _: LPSECURITY_ATTRIBUTES,
        _: DWORD,
        _: DWORD,
        _: HANDLE,
    ) -> HANDLE; // batch.c
    pub fn CreateProcessW(
        _: LPCWSTR,
        _: LPWSTR,
        _: LPSECURITY_ATTRIBUTES,
        _: LPSECURITY_ATTRIBUTES,
        _: BOOL,
        _: DWORD,
        _: PVOID,
        _: LPCWSTR,
        _: LPSTARTUPINFOW,
        _: LPPROCESS_INFORMATION,
    ) -> BOOL; // cmd.c
    pub fn DeleteFileW(_: LPCWSTR) -> BOOL; // copy.c
    pub fn FillConsoleOutputCharacterW(
        hConsoleOutput: HANDLE,
        cCharacter: WCHAR,
        nLength: DWORD,
        dwWriteCoord: COORD,
        lpNumberOfCharsWritten: LPDWORD,
    ) -> BOOL; // prompt.c
    pub fn FindFirstFileW(_: LPCWSTR, _: LPWIN32_FIND_DATAW) -> HANDLE; // cmd.c
    pub fn FindNextFileW(_: HANDLE, _: LPWIN32_FIND_DATAW) -> BOOL; // copy.c
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeEnvironmentStringsW(_: LPWSTR) -> BOOL; // set.c
    pub fn GetCommandLineW() -> LPWSTR; // cmd.c
    pub fn GetConsoleAliasW(
        Source: LPCWSTR,
        TargetBuffer: LPWSTR,
        TargetBufferLength: DWORD,
        ExeName: LPCWSTR,
    ) -> DWORD; // alias.c
    pub fn GetConsoleAliasesLengthW(ExeName: LPCWSTR) -> DWORD; // alias.c
    pub fn GetConsoleAliasesW(AliasBuffer: LPWSTR, AliasBufferLength: DWORD, ExeName: LPCWSTR)
        -> DWORD; // alias.c
    pub fn GetConsoleTitleW(lpConsoleTitle: LPWSTR, nSize: DWORD) -> DWORD; // cmd.c
    pub fn GetCurrentDirectoryW(_: DWORD, _: LPWSTR) -> DWORD; // cmd.c
    pub fn GetDateFormatW(
        _: LCID,
        _: DWORD,
        _: *const SYSTEMTIME,
        _: LPCWSTR,
        _: LPWSTR,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // locale.c
    pub fn GetDiskFreeSpaceW(_: LPCWSTR, _: PDWORD, _: PDWORD, _: PDWORD, _: PDWORD) -> BOOL; // dir.c
    pub fn GetEnvironmentStringsW() -> LPWSTR; // set.c
    pub fn GetEnvironmentVariableW(_: LPCWSTR, _: LPWSTR, _: DWORD) -> DWORD; // cmd.c
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // copy.c
    pub fn GetFullPathNameW(_: LPCWSTR, _: DWORD, _: LPWSTR, _: *mut LPWSTR) -> DWORD; // batch.c
    pub fn GetLocaleInfoW(
        Locale: LCID,
        LCType: LCTYPE,
        lpLCData: LPWSTR,
        cchData: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // locale.c
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // cmd.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // cmd.c
    pub fn GetVersionExW(_: LPOSVERSIONINFOW) -> BOOL; // ver.c
    pub fn GetVolumeInformationW(
        _: LPCWSTR,
        _: LPWSTR,
        _: DWORD,
        _: PDWORD,
        _: PDWORD,
        _: PDWORD,
        _: LPWSTR,
        _: DWORD,
    ) -> BOOL; // dir.c
    pub fn GetVolumePathNameW(_: LPCWSTR, _: LPWSTR, _: DWORD) -> BOOL; // dir.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // cmd.c
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MoveFileExW(_: LPCWSTR, _: LPCWSTR, _: DWORD) -> BOOL; // move.c
    pub fn MoveFileW(_: LPCWSTR, _: LPCWSTR) -> BOOL; // ren.c
    pub fn ReadConsoleInputW(
        hConsoleInput: HANDLE,
        lpBuffer: PINPUT_RECORD,
        nLength: DWORD,
        lpNumberOfEventsRead: LPDWORD,
    ) -> BOOL; // choice.c
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        Reserved: DWORD,
        lpClass: LPWSTR,
        dwOptions: DWORD,
        samDesired: REGSAM,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        phkResult: PHKEY,
        lpdwDisposition: PDWORD,
    ) -> LSTATUS; // assoc.c
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // assoc.c
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // assoc.c
    pub fn RegOpenKeyExW(_: HKEY, _: LPCWSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // assoc.c
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpReserved: LPDWORD,
        lpcSubKeys: LPDWORD,
        lpcbMaxSubKeyLen: LPDWORD,
        lpcbMaxClassLen: LPDWORD,
        lpcValues: LPDWORD,
        lpcbMaxValueNameLen: LPDWORD,
        lpcbMaxValueLen: LPDWORD,
        lpcbSecurityDescriptor: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // assoc.c
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // assoc.c
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS; // assoc.c
    pub fn RemoveDirectoryW(_: LPCWSTR) -> BOOL; // internal.c
    pub fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD; // cmd.c
    pub fn SetConsoleTitleW(lpConsoleTitle: LPCWSTR) -> BOOL; // console.c
    pub fn SetCurrentDirectoryW(_: LPCWSTR) -> BOOL; // cmd.c
    pub fn SetEnvironmentVariableW(_: LPCWSTR, _: LPCWSTR) -> BOOL; // cmd.c
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // copy.c
    pub fn WriteConsoleInputW(
        hConsoleInput: HANDLE,
        lpBuffer: *const INPUT_RECORD,
        nLength: DWORD,
        lpNumberOfEventsWritten: LPDWORD,
    ) -> BOOL; // cmd.c
    pub fn WriteConsoleOutputCharacterW(
        hConsoleOutput: HANDLE,
        lpCharacter: LPCWSTR,
        nLength: DWORD,
        dwWriteCoord: COORD,
        lpNumberOfCharsWritten: LPDWORD,
    ) -> BOOL; // prompt.c
    pub fn lstrcmpW(_: LPCWSTR, _: LPCWSTR) -> ::core::ffi::c_int; // if.c
    pub fn lstrcmpiW(_: LPCWSTR, _: LPCWSTR) -> ::core::ffi::c_int; // if.c
    pub fn lstrlenW(_: LPCWSTR) -> ::core::ffi::c_int; // alias.c
}
pub use crate::prelude::AddConsoleAliasW as AddConsoleAlias; // sdk alias
pub use crate::prelude::ConGetTextWidthW as ConGetTextWidth; // sdk alias
pub use crate::prelude::CopyFileW as CopyFile; // sdk alias
pub use crate::prelude::CopyFileExW as CopyFileEx; // sdk alias
pub use crate::prelude::CreateDirectoryW as CreateDirectory; // sdk alias
pub use crate::prelude::CreateFileW as CreateFile; // sdk alias
pub use crate::prelude::CreateProcessW as CreateProcess; // sdk alias
pub use crate::prelude::DeleteFileW as DeleteFile; // sdk alias
pub use crate::prelude::FillConsoleOutputCharacterW as FillConsoleOutputCharacter; // sdk alias
pub use crate::prelude::FindFirstFileW as FindFirstFile; // sdk alias
pub use crate::prelude::FindNextFileW as FindNextFile; // sdk alias
pub use crate::prelude::FormatMessageW as FormatMessage; // sdk alias
pub use crate::prelude::FreeEnvironmentStringsW as FreeEnvironmentStrings; // sdk alias
pub use crate::prelude::GetCommandLineW as GetCommandLine; // sdk alias
pub use crate::prelude::GetConsoleAliasW as GetConsoleAlias; // sdk alias
pub use crate::prelude::GetConsoleAliasesW as GetConsoleAliases; // sdk alias
pub use crate::prelude::GetConsoleAliasesLengthW as GetConsoleAliasesLength; // sdk alias
pub use crate::prelude::GetConsoleTitleW as GetConsoleTitle; // sdk alias
pub use crate::prelude::GetCurrentDirectoryW as GetCurrentDirectory; // sdk alias
pub use crate::prelude::GetDateFormatW as GetDateFormat; // sdk alias
pub use crate::prelude::GetDiskFreeSpaceW as GetDiskFreeSpace; // sdk alias
pub use crate::prelude::GetEnvironmentStringsW as GetEnvironmentStrings; // sdk alias
pub use crate::prelude::GetEnvironmentVariableW as GetEnvironmentVariable; // sdk alias
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::GetFullPathNameW as GetFullPathName; // sdk alias
pub use crate::prelude::GetLocaleInfoW as GetLocaleInfo; // sdk alias
pub use crate::prelude::GetModuleFileNameW as GetModuleFileName; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetVersionExW as GetVersionEx; // sdk alias
pub use crate::prelude::GetVolumeInformationW as GetVolumeInformation; // sdk alias
pub use crate::prelude::GetVolumePathNameW as GetVolumePathName; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MoveFileW as MoveFile; // sdk alias
pub use crate::prelude::MoveFileExW as MoveFileEx; // sdk alias
pub use crate::prelude::ReadConsoleInputW as ReadConsoleInput; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegDeleteKeyW as RegDeleteKey; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryInfoKeyW as RegQueryInfoKey; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::RemoveDirectoryW as RemoveDirectory; // sdk alias
pub use crate::prelude::SearchPathW as SearchPath; // sdk alias
pub use crate::prelude::SetConsoleTitleW as SetConsoleTitle; // sdk alias
pub use crate::prelude::SetCurrentDirectoryW as SetCurrentDirectory; // sdk alias
pub use crate::prelude::SetEnvironmentVariableW as SetEnvironmentVariable; // sdk alias
pub use crate::prelude::SetFileAttributesW as SetFileAttributes; // sdk alias
pub use strsafe_rust::StringCchCatW as StringCchCat; // sdk alias
pub use strsafe_rust::StringCchCopyW as StringCchCopy; // sdk alias
pub use strsafe_rust::StringCchPrintfW as StringCchPrintf; // sdk alias
pub use crate::prelude::WriteConsoleInputW as WriteConsoleInput; // sdk alias
pub use crate::prelude::WriteConsoleOutputCharacterW as WriteConsoleOutputCharacter; // sdk alias
pub use crate::prelude::fgetws as _fgetts; // sdk alias
pub use crate::prelude::_itow as _itot; // sdk alias
pub use crate::prelude::_snwprintf as _sntprintf; // sdk alias
pub use crate::prelude::_swprintf as _stprintf; // sdk alias
pub use crate::prelude::_wchdir as _tchdir; // sdk alias
pub use crate::prelude::wcscat as _tcscat; // sdk alias
pub use crate::prelude::wcschr as _tcschr; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::wcscspn as _tcscspn; // sdk alias
pub use crate::prelude::_wcsicmp as _tcsicmp; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::_wcslwr as _tcslwr; // sdk alias
pub use crate::prelude::wcsncat as _tcsncat; // sdk alias
pub use crate::prelude::_wcsnicmp as _tcsncicmp; // sdk alias
pub use crate::prelude::wcsncmp as _tcsncmp; // sdk alias
pub use crate::prelude::wcsncpy as _tcsncpy; // sdk alias
pub use crate::prelude::_wcsnicmp as _tcsnicmp; // sdk alias
pub use crate::prelude::_wcsnset as _tcsnset; // sdk alias
pub use crate::prelude::wcspbrk as _tcspbrk; // sdk alias
pub use crate::prelude::wcsrchr as _tcsrchr; // sdk alias
pub use crate::prelude::wcsspn as _tcsspn; // sdk alias
pub use crate::prelude::wcsstr as _tcsstr; // sdk alias
pub use crate::prelude::wcstok as _tcstok; // sdk alias
pub use crate::prelude::wcstol as _tcstol; // sdk alias
pub use crate::prelude::wcstoul as _tcstoul; // sdk alias
pub use crate::prelude::_wcsupr as _tcsupr; // sdk alias
pub use crate::prelude::_wfopen as _tfopen; // sdk alias
pub use crate::prelude::_wgetdcwd as _tgetdcwd; // sdk alias
pub use crate::prelude::towlower as _totlower; // sdk alias
pub use crate::prelude::towupper as _totupper; // sdk alias
pub use crate::prelude::_wpopen as _tpopen; // sdk alias
pub use crate::prelude::_wtoi as _ttoi; // sdk alias
pub use crate::prelude::_wtol as _ttol; // sdk alias
pub use win32macros_rust::debugstr_w as debugstr_aw; // sdk alias
pub use crate::prelude::lstrcmpW as lstrcmp; // sdk alias
pub use crate::prelude::lstrcmpiW as lstrcmpi; // sdk alias
pub use crate::prelude::lstrlenW as lstrlen; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
pub static mut __wine_dbch_cmd: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x63\x6d\x64\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_cmd };
// ==== end wine debug channels ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
pub type OSVERSIONINFO = OSVERSIONINFOW;
pub type STARTUPINFO = STARTUPINFOW;
pub type WIN32_FIND_DATA = WIN32_FIND_DATAW;
// ==== end UNICODE TCHAR family ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn CmdTrace(type_0: INT, file: LPCSTR, line: INT, func: LPCSTR, fmt: LPCSTR, ...);
    pub fn ConFormatMessage(Stream: PCON_STREAM, MessageId: DWORD, ...);
    pub fn ConOutPrintfPaging(StartPaging: BOOL, szFormat: LPWSTR, ...) -> BOOL;
    pub fn DirPrintf(lpFlags: LPDIRSWITCHFLAGS, szFormat: LPTSTR, ...) -> BOOL;
    pub fn ErrorMessage(dwErrorCode: DWORD, szFormat: PCWSTR, ...);
}
// ==== end port variadic ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::cmd::misc::CheckCtrlBreak;
pub use crate::cmd::color::CommandColor;
pub use crate::cmd::delay::CommandDelay;
pub use crate::cmd::del::CommandDelete;
pub use crate::cmd::dir::CommandDir;
pub use crate::cmd::dirstack::CommandDirs;
pub use crate::cmd::echo::CommandEcho;
pub use crate::cmd::echo::CommandEchoerr;
pub use crate::cmd::echo::CommandEchos;
pub use crate::cmd::echo::CommandEchoserr;
pub use crate::cmd::internal::CommandExit;
pub use crate::cmd::free::CommandFree;
pub use crate::cmd::history::CommandHistory;
pub use crate::cmd::memory::CommandMemory;
pub use crate::cmd::dirstack::CommandPopd;
pub use crate::cmd::dirstack::CommandPushd;
pub use crate::cmd::internal::CommandRem;
pub use crate::cmd::screen::CommandScreen;
pub use crate::cmd::internal::CommandShowCommands;
pub use crate::cmd::timer::CommandTimer;
pub use crate::cmd::console::ConGetDefaultAttributes;
pub use crate::cmd::console::ConInFlush;
pub use crate::cmd::console::ConInKey;
pub use crate::cmd::console::ConOutChar;
pub use crate::cmd::console::ConOutResPaging;
pub use crate::cmd::console::ConRingBell;
pub use crate::cmd::console::ConSetScreenColor;
pub use crate::cmd::console::ConSetTitle;
pub use crate::cmd::cmd::DoCommand;
pub use crate::cmd::cmd::DoDelayedExpansion;
pub use crate::cmd::setlocal::DuplicateEnvironment;
pub use crate::cmd::parser::EchoCommand;
pub use crate::cmd::cmd::ExecuteCommandWithEcho;
pub use crate::cmd::r#for::ExecuteFor;
pub use crate::cmd::r#if::ExecuteIf;
pub use crate::cmd::misc::FilePromptYN;
pub use crate::cmd::misc::FilePromptYNA;
pub use crate::cmd::dir::FormatDate;
pub use crate::cmd::dir::FormatTime;
pub use crate::cmd::parser::FreeCommand;
pub use crate::cmd::redir::FreeRedirection;
pub use crate::cmd::locale::GetDateString;
pub use crate::cmd::dirstack::GetDirectoryStackDepth;
pub use crate::cmd::redir::GetHandle;
pub use crate::cmd::misc::GetPathCase;
pub use crate::cmd::internal::GetRootPath;
pub use crate::cmd::locale::GetTimeString;
pub use crate::cmd::prompt::HasInfoLine;
pub use crate::cmd::dirstack::InitDirectoryStack;
pub use crate::cmd::history::InitHistory;
pub use crate::cmd::locale::InitLocale;
pub use crate::cmd::ver::InitOSVersion;
pub use crate::cmd::prompt::InitPrompt;
pub use crate::cmd::misc::IsExistingDirectory;
pub use crate::cmd::misc::IsExistingFile;
pub use crate::cmd::misc::PagePrompt;
pub use crate::cmd::parser::ParseCommand;
pub use crate::cmd::parser::ParseErrorEx;
pub use crate::cmd::redir::PerformRedirection;
pub use crate::cmd::ver::PrintOSVersion;
pub use crate::cmd::prompt::PrintPrompt;
pub use crate::cmd::cmdinput::ReadCommand;
pub use crate::cmd::r#where::SearchForExecutable;
pub use crate::cmd::redir::SetHandle;
pub use crate::cmd::strtoclr::StringToColor;
pub use crate::cmd::misc::StripQuotes;
pub use crate::cmd::redir::UndoRedirection;
pub use crate::cmd::parser::UnparseCommand;
pub use crate::cmd::dir::_DirSwitchesFlags;
pub use crate::cmd::misc::_stpcpy;
pub use crate::cmd::misc::add_entry;
pub use crate::cmd::internal::cmd_chdir;
pub use crate::cmd::copy::cmd_copy;
pub use crate::cmd::ctty::cmd_ctty;
pub use crate::cmd::date::cmd_date;
pub use crate::cmd::setlocal::cmd_endlocal;
pub use crate::cmd::r#for::cmd_for;
pub use crate::cmd::goto::cmd_goto;
pub use crate::cmd::r#if::cmd_if;
pub use crate::cmd::internal::cmd_mkdir;
pub use crate::cmd::mklink::cmd_mklink;
pub use crate::cmd::r#move::cmd_move;
pub use crate::cmd::path::cmd_path;
pub use crate::cmd::pause::cmd_pause;
pub use crate::cmd::prompt::cmd_prompt;
pub use crate::cmd::ren::cmd_rename;
pub use crate::cmd::internal::cmd_rmdir;
pub use crate::cmd::set::cmd_set;
pub use crate::cmd::setlocal::cmd_setlocal;
pub use crate::cmd::shift::cmd_shift;
pub use crate::cmd::start::cmd_start;
pub use crate::cmd::time::cmd_time;
pub use crate::cmd::title::cmd_title;
pub use crate::cmd::r#type::cmd_type;
pub use crate::cmd::ver::cmd_ver;
pub use crate::cmd::verify::cmd_verify;
pub use crate::cmd::vol::cmd_vol;
pub use crate::cmd::error::error_bad_command;
pub use crate::cmd::error::error_cant_exec_program;
pub use crate::cmd::error::error_file_not_found;
pub use crate::cmd::error::error_invalid_parameter_format;
pub use crate::cmd::error::error_invalid_switch;
pub use crate::cmd::error::error_no_pipe;
pub use crate::cmd::error::error_out_of_memory;
pub use crate::cmd::error::error_parameter_format;
pub use crate::cmd::error::error_req_param_missing;
pub use crate::cmd::error::error_sfile_not_found;
pub use crate::cmd::error::error_too_many_parameters;
pub use crate::cmd::misc::freep;
pub use crate::cmd::cmd::nErrorLevel;
pub use crate::cmd::misc::split;
// ==== end cross-TU definitions ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsCJKCodePage { ($p0:expr $(,)?) => { ((((((((((((((((($p0 == 932) as c_int)) as isize) != 0) || ((((($p0 == 949) as c_int)) as isize) != 0)) as c_int)) as isize) != 0) || ((((($p0 == 950) as c_int)) as isize) != 0)) as c_int)) as isize) != 0) || ((((($p0 == 936) as c_int)) as isize) != 0)) as c_int) }; }
#[macro_export] macro_rules! __iscsym { ($p0:expr $(,)?) => { (((((isalnum((($p0) as _))) as isize) != 0) || ((((($p0 == 95) as c_int)) as isize) != 0)) as c_int) }; }
#[macro_export] macro_rules! __iscsymf { ($p0:expr $(,)?) => { (((((isalpha((($p0) as _))) as isize) != 0) || ((((($p0 == 95) as c_int)) as isize) != 0)) as c_int) }; }
#[macro_export] macro_rules! _alloca { ($p0:expr $(,)?) => { __builtin_alloca((($p0) as _)) }; }
#[macro_export] macro_rules! _istalpha { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((((0x0100 | 0x1) | 0x2)) as _)) }; }
#[macro_export] macro_rules! _istcntrl { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x20) as _)) }; }
#[macro_export] macro_rules! _istdigit { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x4) as _)) }; }
#[macro_export] macro_rules! _istprint { ($p0:expr $(,)?) => { iswctype((($p0) as _), (((((0x40 | 0x10) | ((0x0100 | 0x1) | 0x2)) | 0x4)) as _)) }; }
#[macro_export] macro_rules! _istspace { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x8) as _)) }; }
#[macro_export] macro_rules! cmd_alloc { ($p0:expr $(,)?) => { malloc((($p0) as _)) }; }
#[macro_export] macro_rules! cmd_dup { ($p0:expr $(,)?) => { _wcsdup((($p0) as _)) }; }
#[macro_export] macro_rules! cmd_free { ($p0:expr $(,)?) => { free((($p0) as _)) }; }
#[macro_export] macro_rules! cmd_realloc { ($p0:expr, $p1:expr $(,)?) => { realloc((($p0) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const D_OFF: *const u16 = (&[0x006f, 0x0066, 0x0066, 0x0000u16]).as_ptr(); // L"off"
pub const D_ON: *const u16 = (&[0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"on"
pub const STANDARD_SEPS: *const u16 = (&[0x002c, 0x003b, 0x003d, 0x0000u16]).as_ptr(); // L",;="
pub use crate::{IsCJKCodePage, __iscsym, __iscsymf, _alloca, _istalpha, _istcntrl, _istdigit, _istprint, _istspace, cmd_alloc, cmd_dup, cmd_free, cmd_realloc, max, min, va_end, va_start};
// ==== end SDK header macros ====

// Hand port, kept OUTSIDE the generated block so `port macros` leaves it alone: the banked
// item reads `bCtrlBreak` (a BOOL) as a bool and `(*Cmd).For` through PARSED_COMMAND's
// anonymous union, neither of which compiles.
// base/shell/cmd/for.c:81: #define Exiting(Cmd) (bCtrlBreak || (fc != (Cmd)->For.Context))
#[allow(unused_macros)]
macro_rules! Exiting {
    ($Cmd:expr $(,)?) => {
        ((bCtrlBreak != 0) || (fc != (*$Cmd).__bindgen_anon_1.For.Context))
    };
}
#[allow(unused_imports)]
pub(crate) use Exiting;

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/shell/cmd/console.h:56: #define ConErrFormatMessage(MessageId, ...) ConFormatMessage(StdErr, (MessageId), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConErrFormatMessage {
    ($message_id:expr $(, $arg:expr)* $(,)?) => {
        (ConFormatMessage(StdErr, ($message_id) $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ConErrFormatMessage;
// base/shell/cmd/console.h:44: #define ConErrPrintf(szStr, ...) ConPrintf(StdErr, (szStr), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConErrPrintf {
    ($sz_str:expr $(, $arg:expr)* $(,)?) => {
        (ConPrintf(StdErr, ($sz_str) $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ConErrPrintf;
// base/shell/cmd/console.h:32: #define ConErrPuts(szStr) ConPuts(StdErr, (szStr))
#[allow(unused_macros)]
macro_rules! ConErrPuts {
    ($szStr:expr $(,)?) => {
        (ConPuts(StdErr, ($szStr)))
    };
}
#[allow(unused_imports)]
pub(crate) use ConErrPuts;
// base/shell/cmd/console.h:50: #define ConErrResPrintf(uID, ...) ConResPrintf(StdErr, (uID), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConErrResPrintf {
    ($uid:expr $(, $arg:expr)* $(,)?) => {
        (ConResPrintf(StdErr, ($uid) $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ConErrResPrintf;
// base/shell/cmd/console.h:38: #define ConErrResPuts(uID) ConResPuts(StdErr, (uID))
#[allow(unused_macros)]
macro_rules! ConErrResPuts {
    ($uID:expr $(,)?) => {
        (ConResPuts(StdErr, ($uID)))
    };
}
#[allow(unused_imports)]
pub(crate) use ConErrResPuts;
// base/shell/cmd/console.h:53: #define ConOutFormatMessage(MessageId, ...) ConFormatMessage(StdOut, (MessageId), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConOutFormatMessage {
    ($message_id:expr $(, $arg:expr)* $(,)?) => {
        (ConFormatMessage(StdOut, ($message_id) $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutFormatMessage;
// base/shell/cmd/console.h:41: #define ConOutPrintf(szStr, ...) ConPrintf(StdOut, (szStr), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConOutPrintf {
    ($sz_str:expr $(, $arg:expr)* $(,)?) => {
        (ConPrintf(StdOut, ($sz_str) $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutPrintf;
// base/shell/cmd/console.h:29: #define ConOutPuts(szStr) ConPuts(StdOut, (szStr))
#[allow(unused_macros)]
macro_rules! ConOutPuts {
    ($szStr:expr $(,)?) => {
        (ConPuts(StdOut, ($szStr)))
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutPuts;
// base/shell/cmd/console.h:47: #define ConOutResPrintf(uID, ...) ConResPrintf(StdOut, (uID), ##__VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ConOutResPrintf {
    ($uid:expr $(, $arg:expr)* $(,)?) => {
        (ConResPrintf(StdOut, ($uid) $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutResPrintf;
// base/shell/cmd/console.h:35: #define ConOutResPuts(uID) ConResPuts(StdOut, (uID))
#[allow(unused_macros)]
macro_rules! ConOutResPuts {
    ($uID:expr $(,)?) => {
        (ConResPuts(StdOut, ($uID)))
    };
}
#[allow(unused_imports)]
pub(crate) use ConOutResPuts;
// base/shell/cmd/precomp.h:57: #define ERR(fmt, ...) CmdTrace(__WINE_DBCL_ERR, __FILE__, __LINE__, __FUNCTION__, fmt, ## __VA_ARGS__)
#[allow(unused_macros)]
macro_rules! ERR {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        (CmdTrace(__WINE_DBCL_ERR, concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char, (line!() as core::ffi::c_int), concat!(module_path!(), "\0").as_ptr() as *const core::ffi::c_char, $fmt $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use ERR;
// base/shell/cmd/for.c:85: #define ExitingOrGoto(Cmd) (Exiting(Cmd) || (bc && bc->current == NULL))
#[allow(unused_macros)]
macro_rules! ExitingOrGoto {
    ($Cmd:expr $(,)?) => {
        (Exiting!($Cmd) || (!bc.is_null() && (*bc).current.is_null()))
    };
}
#[allow(unused_imports)]
pub(crate) use ExitingOrGoto;
// base/shell/cmd/console.c:333: #define IS_FULL_WIDTH(wch) (((USHORT)(wch) >= 0x0080) && (mk_wcwidth_cjk(wch) == 2))
#[allow(unused_macros)]
macro_rules! IS_FULL_WIDTH {
    ($wch:expr $(,)?) => {
        ((($wch) as USHORT) >= 0x0080 && mk_wcwidth_cjk($wch) == 2)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_FULL_WIDTH;
// base/shell/cmd/for.c:77: #define RunInstance(Cmd) ExecuteCommandWithEcho((Cmd)->Subcommands)
#[allow(unused_macros)]
macro_rules! RunInstance {
    ($Cmd:expr $(,)?) => {
        (ExecuteCommandWithEcho((*$Cmd).Subcommands))
    };
}
#[allow(unused_imports)]
pub(crate) use RunInstance;
// base/shell/cmd/precomp.h:65: #define TRACE(fmt, ...) CmdTrace(__WINE_DBCL_TRACE, __FILE__, __LINE__, __FUNCTION__, fmt, ## __VA_ARGS__)
#[allow(unused_macros)]
macro_rules! TRACE {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        (CmdTrace(__WINE_DBCL_TRACE, concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char, (line!() as core::ffi::c_int), concat!(module_path!(), "\0").as_ptr() as *const core::ffi::c_char, $fmt $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use TRACE;
// base/shell/cmd/precomp.h:61: #define WARN(fmt, ...) CmdTrace(__WINE_DBCL_WARN, __FILE__, __LINE__, __FUNCTION__, fmt, ## __VA_ARGS__)
#[allow(unused_macros)]
macro_rules! WARN {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        (CmdTrace(__WINE_DBCL_WARN, concat!(file!(), "\0").as_ptr() as *const core::ffi::c_char, (line!() as core::ffi::c_int), concat!(module_path!(), "\0").as_ptr() as *const core::ffi::c_char, $fmt $(, $arg)*))
    };
}
#[allow(unused_imports)]
pub(crate) use WARN;
// base/shell/cmd/cmddbg.h:34: #define cmd_exit(code) ExitProcess(code)
#[allow(unused_macros)]
macro_rules! cmd_exit {
    ($code:expr $(,)?) => {
        (ExitProcess($code))
    };
}
#[allow(unused_imports)]
pub(crate) use cmd_exit;
// ==== end port macros ====

// ================= stdlib ======================================================================
/// The CRT's qsort comparator (search.h's `_CoreCrtNonSecureSearchSortCompareFunction`), as
/// ucrt-rust spells it: the extern `qsort` declaration above names it.
pub type Compare = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;

// ================= _alloca ====================================================================
/// C's `_alloca` for a runtime size, which Rust cannot place on the stack: each block is
/// `malloc`ed and freed when the frame drops -- the C function's return, as with `_alloca`.
/// No `alloc` crate (this crate is no_std): the blocks are chained through a header in the
/// allocation itself. The 16-byte header keeps malloc's alignment for the payload.
/// (set.c's PARSE_IDENT and assignment buffer, mklink.c's REPARSE_DATA_BUFFER.)
pub struct AllocaFrame(*mut AllocaBlock);

#[repr(C, align(16))]
pub struct AllocaBlock {
    next: *mut AllocaBlock,
}

impl AllocaFrame {
    pub const fn new() -> Self {
        AllocaFrame(core::ptr::null_mut())
    }

    /// `size` bytes, valid until this frame drops; null if malloc fails (the C's `_alloca`
    /// raises a stack-overflow exception there, and none of its callers checks).
    pub unsafe fn alloc(&mut self, size: usize) -> *mut c_void {
        let header = core::mem::size_of::<AllocaBlock>();
        let block = malloc(header.wrapping_add(size)) as *mut AllocaBlock;
        if block.is_null() {
            return core::ptr::null_mut();
        }
        (*block).next = self.0;
        self.0 = block;
        (block as *mut u8).add(header) as *mut c_void
    }
}

impl Drop for AllocaFrame {
    fn drop(&mut self) {
        let mut block = self.0;
        while !block.is_null() {
            // SAFETY: every block in the chain came from `malloc` in `alloc` above.
            unsafe {
                let next = (*block).next;
                free(block as *mut c_void);
                block = next;
            }
        }
    }
}
