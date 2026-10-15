//! ftp-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// io.h/sys/stat.h POSIX names the C reaches through oldnames.a (`close` -> `_close`): bound to
// the msvcrt export they forward to, so the cell needs no oldnames for them. `fstat` is a
// __CRT_INLINE over `_fstat` (sys/stat.h:341), the island's `stat` is its layout.
extern "C" {
    #[link_name = "_close"]
    pub fn close(fd: c_int) -> c_int;
    #[link_name = "_read"]
    pub fn read(fd: c_int, buf: *mut c_void, n: c_uint) -> c_int;
    #[link_name = "_write"]
    pub fn write(fd: c_int, buf: *const c_void, n: c_uint) -> c_int;
    #[link_name = "_fstat"]
    pub fn fstat(fd: c_int, st: *mut stat) -> c_int;
    // sys/stat.h's `stat` is the same kind of inline, over `_stat`.
    #[link_name = "_stat"]
    pub fn stat(path: *const c_char, st: *mut stat) -> c_int;
    // `_fmode = O_BINARY`: _fmode is a DATA export; _set_fmode writes it on every arch
    // (msvcrt.spec:905), where `__p__fmode` is i386-only.
    pub fn _set_fmode(mode: c_int) -> c_int;
    // stdio.h; ftp.c's `vfprintf -> _doprnt` macro is `#ifdef NOVFPRINTF`, never this build.
    pub fn vfprintf(stream: *mut FILE, fmt: *const c_char, ap: va_list) -> c_int;
    // signal.h: `__p_sig_fn_t signal(int, __p_sig_fn_t)`, `typedef void (*__p_sig_fn_t)(int)`.
    // The handler is a nullable fn pointer, which is what command()/getreply() store and
    // restore (cmdabort); ucrt's `usize` spelling cannot carry it.
    pub fn signal(
        signum: c_int,
        handler: Option<unsafe extern "C" fn(c_int)>,
    ) -> Option<unsafe extern "C" fn(c_int)>;
    // `_timezone` / `_daylight` are msvcrt DATA exports with no all-arch accessor: read the
    // import library's `__imp_` slot (bootvid-rust/src/framebuf_impl.rs, KeLoaderBlock). On
    // i386 rustc prepends one `_` and the cdecl data name carries its own, hence `_imp___X`.
    #[cfg_attr(target_arch = "x86", link_name = "_imp___timezone")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp__timezone")]
    static __imp__timezone: *mut c_long;
    #[cfg_attr(target_arch = "x86", link_name = "_imp___daylight")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__imp__daylight")]
    static __imp__daylight: *mut c_int;
}
/// time.h `_timezone`, read through msvcrt's import slot.
#[inline(always)]
pub unsafe fn crt_timezone() -> c_long {
    **core::ptr::addr_of!(__imp__timezone)
}
/// time.h `_daylight`, read through msvcrt's import slot.
#[inline(always)]
pub unsafe fn crt_daylight() -> c_int {
    **core::ptr::addr_of!(__imp__daylight)
}

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const DEFAULT: DPI_HOSTING_BEHAVIOR = DPI_HOSTING_BEHAVIOR_DEFAULT; // island enum
pub const ENABLE_LINE_INPUT: u32 = 2; // at-rust/src/prelude.rs
pub const ENABLE_PROCESSED_INPUT: u32 = 1; // attrib-rust/src/types_amd64.rs
pub const SIGINT: u32 = 2; // msvcrt-sys/src/bindgen_amd64.rs
pub const SIG_IGN: usize = 1; // ucrt-rust/src/misc/signal.rs
pub const STD_ERROR_HANDLE: u32 = 4294967284; // cmd-rust/src/prelude.rs
pub const STD_INPUT_HANDLE: u32 = 4294967286; // at-rust/src/prelude.rs
extern "C" {
    pub fn __iob_func() -> *mut FILE; // crt-startup-rust/src/misc.rs
    pub fn _errno() -> *mut c_int; // cmd-rust/src/prelude.rs
    pub fn _pclose(file: *mut FILE) -> i32; // cmd-rust/src/prelude.rs
    pub fn _popen(command: LPCSTR, mode: LPCSTR) -> *mut FILE; // dbgprint_app-rust/src/prelude.rs
    pub fn exit(exitcode: i32); // findstr-rust/src/prelude.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // cmd-rust/src/prelude.rs
    pub fn feof(file: *mut FILE) -> i32; // msconfig-rust/src/prelude.rs
    pub fn ferror(file: *mut FILE) -> i32; // comp-rust/src/prelude.rs
    pub fn fflush(stream: *mut FILE) -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn fgets(buf: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char; // dbgprint_app-rust/src/prelude.rs
    pub fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE; // dhcpcsvc-rust/src/prelude.rs
    pub fn fprintf(stream: *mut FILE, format: *const c_char, ...) -> c_int; // dhcpcsvc-rust/src/prelude.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn fseek(file: *mut FILE, offset: i32, whence: i32) -> i32; // comp-rust/src/prelude.rs
    pub fn gmtime(t: *const time_t) -> *mut tm; // dhcpcsvc-rust/src/prelude.rs
    pub fn isdigit(c: i32) -> i32; // arp-rust/src/prelude.rs
    pub fn islower(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isspace(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn isupper(c: i32) -> i32; // cnt-ctype/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn perror(str: *const i8); // finger-rust/src/prelude.rs
    pub fn putchar(c: i32) -> i32; // finger-rust/src/prelude.rs
    pub fn sscanf(s: *const i8, fmt: *const i8, ...) -> i32; // arp-rust/src/prelude.rs
    pub fn vsprintf(buf: *mut c_char, fmt: *const c_char, ap: *mut c_char) -> c_int; // autochk-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn GetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, lpMode: LPDWORD) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
    pub fn SetConsoleMode(hConsoleHandle: *mut ::core::ffi::c_void, dwMode: u32) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileType(arg1: HANDLE) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSystemTimeAsFileTime(arg1: LPFILETIME); // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WSAGetLastError() -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn WSAStartup(wVersionRequired: WORD, lpWSAData: LPWSADATA) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn __WSAFDIsSet(arg1: SOCKET, arg2: *mut fd_set) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn accept(s: SOCKET, addr: *mut sockaddr, addrlen: *mut ::core::ffi::c_int) -> SOCKET; // tracert-rust/src/types_i386.rs
    pub fn bind( s: SOCKET, addr: *const sockaddr, namelen: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn closesocket(s: SOCKET) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn connect( s: SOCKET, name: *const sockaddr, namelen: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn gethostbyname(name: *const ::core::ffi::c_char) -> *mut hostent; // tracert-rust/src/types_i386.rs
    pub fn gethostname( name: *mut ::core::ffi::c_char, namelen: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn getservbyname( name: *const ::core::ffi::c_char, proto: *const ::core::ffi::c_char, ) -> *mut servent; // tracert-rust/src/types_i386.rs
    pub fn getsockname( s: SOCKET, name: *mut sockaddr, namelen: *mut ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn htons(hostshort: ::core::ffi::c_ushort) -> u_short; // tracert-rust/src/types_i386.rs
    pub fn inet_addr(cp: *const ::core::ffi::c_char) -> ::core::ffi::c_ulong; // tracert-rust/src/types_i386.rs
    pub fn inet_ntoa(in_: in_addr) -> *mut ::core::ffi::c_char; // tracert-rust/src/types_i386.rs
    pub fn listen(s: SOCKET, backlog: ::core::ffi::c_int) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn recv( s: SOCKET, buf: *mut ::core::ffi::c_char, len: ::core::ffi::c_int, flags: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn select( nfds: ::core::ffi::c_int, readfds: *mut fd_set, writefds: *mut fd_set, exceptfds: *mut fd_set, timeout: *const timeval, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn send( s: SOCKET, buf: *const ::core::ffi::c_char, len: ::core::ffi::c_int, flags: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn setsockopt( s: SOCKET, level: ::core::ffi::c_int, optname: ::core::ffi::c_int, optval: *const ::core::ffi::c_char, optlen: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn shutdown(s: SOCKET, how: ::core::ffi::c_int) -> ::core::ffi::c_int; // tracert-rust/src/types_i386.rs
    pub fn socket( af: ::core::ffi::c_int, type_: ::core::ffi::c_int, protocol: ::core::ffi::c_int, ) -> SOCKET; // tracert-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::ftp::cmdtab::NCMDS;
pub use crate::ftp::cmds::account;
pub use crate::ftp::ftp::allbinary;
pub use crate::ftp::main::altarg;
pub use crate::ftp::main::autologin;
pub use crate::ftp::main::bell;
pub use crate::ftp::main::bytename;
pub use crate::ftp::cmds::cd;
pub use crate::ftp::cmds::cdup;
pub use crate::ftp::main::code;
pub use crate::ftp::main::connected;
pub use crate::ftp::main::cpend;
pub use crate::ftp::main::crflag;
pub use crate::ftp::main::debug;
pub use crate::ftp::cmds::delete;
pub use crate::ftp::cmds::disconnect;
pub use crate::ftp::cmds::do_chmod;
pub use crate::ftp::cmds::do_umask;
pub use crate::ftp::main::doglob;
pub use crate::ftp::cmds::doproxy;
pub use crate::ftp::main::formname;
pub use crate::ftp::cmds::fsetmode;
pub use crate::ftp::cmds::get;
pub use crate::ftp::main::getcmd;
pub use crate::ftp::main::globerr;
pub use crate::ftp::main::hash;
pub use crate::ftp::main::help;
pub use crate::ftp::ftp::hostname;
pub use crate::ftp::cmds::idle;
pub use crate::ftp::main::interactive;
pub use crate::ftp::cmds::lcd;
pub use crate::ftp::cmds::ls;
pub use crate::ftp::main::macbuf;
pub use crate::ftp::cmds::macdef;
pub use crate::ftp::main::macnum;
pub use crate::ftp::main::macros;
pub use crate::ftp::cmds::makedir;
pub use crate::ftp::main::mapflag;
pub use crate::ftp::main::mapin;
pub use crate::ftp::main::mapout;
pub use crate::ftp::main::margc;
pub use crate::ftp::main::margv;
pub use crate::ftp::main::mcase;
pub use crate::ftp::cmds::mdelete;
pub use crate::ftp::main::mflag;
pub use crate::ftp::cmds::mget;
pub use crate::ftp::cmds::mls;
pub use crate::ftp::main::modename;
pub use crate::ftp::cmds::modtime;
pub use crate::ftp::cmds::mput;
pub use crate::ftp::cmds::newer;
pub use crate::ftp::main::ntflag;
pub use crate::ftp::main::ntin;
pub use crate::ftp::main::ntout;
pub use crate::ftp::main::options;
pub use crate::ftp::main::passivemode;
pub use crate::ftp::main::pasv;
pub use crate::ftp::main::portnum;
pub use crate::ftp::main::proxflag;
pub use crate::ftp::main::proxy;
pub use crate::ftp::cmds::put;
pub use crate::ftp::cmds::pwd;
pub use crate::ftp::cmds::quit;
pub use crate::ftp::cmds::quote;
pub use crate::ftp::ftp::recvrequest;
pub use crate::ftp::cmds::reget;
pub use crate::ftp::cmds::removedir;
pub use crate::ftp::cmds::renamefile;
pub use crate::ftp::cmds::restart;
pub use crate::ftp::ftp::restart_point;
pub use crate::ftp::cmds::rmthelp;
pub use crate::ftp::cmds::rmtstatus;
pub use crate::ftp::main::runique;
pub use crate::ftp::ruserpass::ruserpass;
pub use crate::ftp::main::sendport;
pub use crate::ftp::ftp::sendrequest;
pub use crate::ftp::cmds::setascii;
pub use crate::ftp::cmds::setbell;
pub use crate::ftp::cmds::setbinary;
pub use crate::ftp::cmds::setcase;
pub use crate::ftp::cmds::setcr;
pub use crate::ftp::cmds::setdebug;
pub use crate::ftp::cmds::setform;
pub use crate::ftp::cmds::setglob;
pub use crate::ftp::cmds::sethash;
pub use crate::ftp::cmds::setnmap;
pub use crate::ftp::cmds::setntrans;
pub use crate::ftp::cmds::setpassive;
pub use crate::ftp::cmds::setpeer;
pub use crate::ftp::cmds::setport;
pub use crate::ftp::cmds::setprompt;
pub use crate::ftp::cmds::setrunique;
pub use crate::ftp::cmds::setstruct;
pub use crate::ftp::cmds::setsunique;
pub use crate::ftp::cmds::settenex;
pub use crate::ftp::cmds::settrace;
pub use crate::ftp::cmds::settype;
pub use crate::ftp::cmds::setverbose;
pub use crate::ftp::cmds::shell;
pub use crate::ftp::cmds::site;
pub use crate::ftp::cmds::sizecmd;
pub use crate::ftp::cmds::status;
pub use crate::ftp::main::structname;
pub use crate::ftp::main::sunique;
pub use crate::ftp::cmds::syst;
pub use crate::ftp::main::trace;
pub use crate::ftp::main::typename;
pub use crate::ftp::cmds::user;
pub use crate::ftp::main::verbose;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// tm: c2rust, cmds.c
#[derive(Copy, Clone)]
#[repr(C, packed(8))]
pub struct tm {
    pub tm_sec: ::core::ffi::c_int,
    pub tm_min: ::core::ffi::c_int,
    pub tm_hour: ::core::ffi::c_int,
    pub tm_mday: ::core::ffi::c_int,
    pub tm_mon: ::core::ffi::c_int,
    pub tm_year: ::core::ffi::c_int,
    pub tm_wday: ::core::ffi::c_int,
    pub tm_yday: ::core::ffi::c_int,
    pub tm_isdst: ::core::ffi::c_int,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _stricmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // ruserpass.c
    pub fn _strnicmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // ruserpass.c
    pub fn _tzset(); // fake.c
    pub fn atoi(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // cmds.c
    pub fn atol(_Str: *const ::core::ffi::c_char) -> ::core::ffi::c_long; // cmds.c
    pub fn chdir(_Path: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // cmds.c
    pub fn fileno(_File: *mut FILE) -> ::core::ffi::c_int; // ftp.c
    pub fn getc(_File: *mut FILE) -> ::core::ffi::c_int; // ftp.c
    pub fn getchar() -> ::core::ffi::c_int; // cmds.c
    pub fn getcwd(
        _DstBuf: *mut ::core::ffi::c_char,
        _SizeInBytes: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn getenv(_VarName: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char; // main.c
    pub fn gets(_Buffer: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn lseek(
        _FileHandle: ::core::ffi::c_int,
        _Offset: ::core::ffi::c_long,
        _Origin: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long; // ftp.c
    pub fn mktemp(_TemplateName: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn printf(_Format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int; // cmds.c
    pub fn putc(_Ch: ::core::ffi::c_int, _File: *mut FILE) -> ::core::ffi::c_int; // ftp.c
    pub fn sprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // ruserpass.c
    pub fn strcat(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn strchr(
        _Str: *const ::core::ffi::c_char,
        _Val: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // cmds.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // cmds.c
    pub fn strncat(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
        _Count: size_t,
    ) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn strncmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // cmds.c
    pub fn strncpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
        _Count: size_t,
    ) -> *mut ::core::ffi::c_char; // cmds.c
    pub fn unlink(_Filename: *const ::core::ffi::c_char) -> ::core::ffi::c_int; // cmds.c
}
extern "system" {
    pub fn CreateProcessA(
        _: LPCSTR,
        _: LPSTR,
        _: LPSECURITY_ATTRIBUTES,
        _: LPSECURITY_ATTRIBUTES,
        _: BOOL,
        _: DWORD,
        _: PVOID,
        _: LPCSTR,
        _: LPSTARTUPINFOA,
        _: LPPROCESS_INFORMATION,
    ) -> BOOL; // cmds.c
    pub fn GetEnvironmentVariableA(_: LPCSTR, _: LPSTR, _: DWORD) -> DWORD; // cmds.c
}
pub use crate::prelude::CreateProcessA as CreateProcess; // sdk alias
pub use crate::prelude::GetEnvironmentVariableA as GetEnvironmentVariable; // sdk alias
pub use crate::prelude::strchr as index; // sdk alias
pub use crate::prelude::_stricmp as strcasecmp; // sdk alias
pub use crate::prelude::_strnicmp as strncasecmp; // sdk alias
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const INVALID_SOCKET: SOCKET = (((!0)) as SOCKET);
pub use crate::{va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/network/ftp/ftp.c:1217: #define UC(b) (((int)b)&0xff)
#[allow(unused_macros)]
macro_rules! UC {
    ($b:expr $(,)?) => {
        ((($b) as i32) & 0xff)
    };
}
#[allow(unused_imports)]
pub(crate) use UC;
// base/applications/network/ftp/fake.h:1: #define bcopy(s, d, l) memcpy((d),(s),(l))
#[allow(unused_macros)]
macro_rules! bcopy {
    ($s:expr, $d:expr, $l:expr $(,)?) => {
        (memcpy(($d), ($s), ($l)))
    };
}
#[allow(unused_imports)]
pub(crate) use bcopy;
// base/applications/network/ftp/fake.h:2: #define bzero(cp, l) memset((cp),0,(l))
#[allow(unused_macros)]
macro_rules! bzero {
    ($cp:expr, $l:expr $(,)?) => {
        (memset(($cp), 0, ($l)))
    };
}
#[allow(unused_imports)]
pub(crate) use bzero;
// base/applications/network/ftp/ftp.c:1275: #define nz(x) ((x) == 0 ? 1 : (x))
#[allow(unused_macros)]
macro_rules! nz {
    ($x:expr $(,)?) => {
        // `0 as _` / `1 as _`: the C macro is untyped; ptransfer applies it to a double.
        (if ($x) == 0 as _ { 1 as _ } else { $x })
    };
}
#[allow(unused_imports)]
pub(crate) use nz;
// ==== end port macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn command(fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub fn fprintfSocket(s: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ...) -> *const ::core::ffi::c_char;
}
// ==== end port variadic ====
