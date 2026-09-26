//! `dll/win32/msvcrt/file.c`, translated whole — 189/189 exports, of which the 173
//! `#[no_mangle]` bodies here are the live implementation in msvcrt.dll.
//!
//! WIRING THIS MODULE IS THE FLIP. msvcrt's cohab is by LINK ORDER, not by removing
//! the `.c` from CMake: `libmsvcrt_crt_cargo.a` is linked ahead of `msvcrt_static`
//! (see that CMakeLists), so lld first-match takes every symbol defined here and
//! file.c's copies go dead in place under `--allow-multiple-definition`. There is no
//! second step to perform, and no CMake edit owed — `pub mod file;` in this
//! directory's `mod.rs` is the whole of it.
//!
//! Which is why it had to go in ATOMICALLY, and did. file.c's entry points share one
//! fd table and one FILE ring, so translating half of them would have left a C
//! `fopen` beside a Rust `fclose` — split state, not a partial port. The usual
//! "a function this module does not define simply stays C" safety of cohab does not
//! apply to this file.
//!
//! The 20 variadic entry points are the one genuine exception, and they are not a
//! split: see the block below.

use crate::prelude::*;
use core::ffi::c_void;

// NOT `use msvcrt_sys::*;` as well. prelude.rs already re-exports the whole island
// (`pub use msvcrt_sys::*`), so a second glob here imports every name TWICE from two
// different roots and every use of an overlapping one is E0659 "ambiguous".
//
// It also loses on the names that overlap deliberately: the prelude re-types a few
// island constants to the shape their uses need — `FALSE` as `u8` because the uses are
// BOOL-shaped, `WEOF` as `wint_t` — and an explicit item beats a glob, so INSIDE the
// prelude its versions win. Re-globbing the island here puts them back on equal footing
// and throws the choice away. One door.

// The printf engine, already Rust. file.c's `v*` bodies drive it directly — the same
// set printf_shells.rs imports. `ARGMAX` is printf.rs's name for the C's `_ARGMAX`
// (corecrt.h:277, value 100); it is aliased rather than redefined so there is exactly
// one definition of that bound in the crate.
use crate::printf::{
    arg_clbk_positional, arg_clbk_valist, create_positional_ctx_a, create_positional_ctx_w,
    pf_printf_a, pf_printf_w, ARGMAX as _ARGMAX, MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
    MSVCRT_PRINTF_POSITIONAL_PARAMS,
};

// CRT siblings that are ALREADY Rust in this crate. IMPORTED, not re-declared `extern`:
// both would link (they are `#[no_mangle]`), but only the import makes rustc check the
// signature against the definition. An `extern` block here would be an unchecked
// restatement — the same hazard the msvcrt-sys island exists to remove for Win32.
use crate::string_l::_strdup;
use crate::wcs_rest::wctomb;
use crate::{_wcsnicmp, memcmp, towlower, towupper, wcschr, wcslen, wcsncmp};

/// A `L"..."` wide-string constant, encoded at compile time.
///
/// The five call sites below used to spell these as `b"ccs\0".as_ptr() as *const wchar_t`
/// — a byte string reinterpreted as UTF-16. `b"TMP\0"` is the four BYTES `54 4D 50 00`,
/// which read as `wchar_t` is `0x4D54, 0x0050, <whatever follows in rodata>`: not "TMP",
/// not NUL-terminated, and never equal to anything it was compared against. It compiles
/// because the cast is legal and rustc has no idea what the bytes were supposed to mean.
///
/// The length is COMPUTED, never written down, which is the other half of the same class:
/// a hand-counted `[u16; N]` that is one short silently truncates the string (the
/// `SERVICES_KEY` incident, `tools/scan_wide_literals.py`).
macro_rules! wlit {
    ($name:ident, $b:literal) => {
        static $name: [wchar_t; { $b.len() + 1 }] = {
            let s: &[u8] = $b;
            let mut a = [0u16; { $b.len() + 1 }];
            let mut i = 0;
            while i < s.len() {
                a[i] = s[i] as u16;
                i += 1;
            }
            a
        };
    };
}

// The `ccs=` encoding suffix `msvcrt_get_flags` parses out of a WIDE mode string, and the
// environment variable `_wtempnam` consults. Every one of these was garbled.
wlit!(W_CCS, b"ccs");
wlit!(W_UTF8, b"utf-8");
wlit!(W_UTF16LE, b"utf-16le");
wlit!(W_UNICODE, b"unicode");
wlit!(W_TMP, b"TMP");
use super::mbcs::{mbstowcs, mbtowc};

// ---------------------------------------------------------------------------
// file.c's 20 VARIADIC entry points are deliberately NOT here.
//
// Stable Rust cannot DEFINE a C-variadic function (`c_variadic` is nightly and
// this crate is pinned stable), so they live in
// `dll/win32/msvcrt/rust_file_varargs_shim.c` as three-line `va_start`
// trampolines — the same arrangement wcs.c and scanf.c already use.
//
// Nothing else moved back to C. Each trampoline only starts the cursor and
// hands it to its `va_list` twin below, all of which are Rust:
//   fprintf -> vfprintf, printf_s -> vprintf_s, _fprintf_p -> _vfprintf_p_l, ...
//   _sopen  -> _sopen_dispatch, _wsopen -> _wsopen_dispatch  (both Rust, below)
//   _open   -> _sopen, _wopen -> _wsopen                     (both in the shim)
//
// tools/scan_varargs_shim_parity.py diffs that TU against file.c so the
// transcription cannot drift.
//
// Three of them are called from Rust below (`_creat`/`_wcreat` -> `_open`/`_wopen`,
// `_wfsopen` -> `_wsopen`), so they are declared here. DECLARING a C-variadic is
// ordinary stable Rust — only DEFINING one needs `c_variadic` — and the declaration
// must stay variadic because the call sites genuinely differ in arity: `_open` is
// called with both 2 and 3 arguments, matching the C, where the third is consumed
// only when `_O_CREAT` is set. Giving these a fixed arity to "simplify" would put a
// pmode slot on the stack that the callee does not read on the non-CREAT path.
// `_sopen` is deliberately absent: nothing in this file calls it.
// ---------------------------------------------------------------------------
// file.c's three function-like macros. They stay MACROS, not fns: an out-of-line
// no-op is the optimization barrier that uncloaks latent ABI mismatches, and for
// ARRAY_SIZE a fn cannot take the array by value without copying it.
//
//   file.c:306  #define LOCK_FILES()   do { EnterCriticalSection(&MSVCRT_file_cs); } while (0)
//   file.c:307  #define UNLOCK_FILES() do { LeaveCriticalSection(&MSVCRT_file_cs); } while (0)
//   winnt_old.h:4762  #define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
macro_rules! LOCK_FILES {
    () => {
        EnterCriticalSection(core::ptr::addr_of_mut!(MSVCRT_file_cs))
    };
}
macro_rules! UNLOCK_FILES {
    () => {
        LeaveCriticalSection(core::ptr::addr_of_mut!(MSVCRT_file_cs))
    };
}
/// `sizeof(x)/sizeof(x[0])` — the element COUNT, which on a Rust array is `.len()`.
/// Taken by `addr_of!` so this never moves or copies the (large, static) array.
macro_rules! ARRAY_SIZE {
    ($a:expr) => {
        ((*core::ptr::addr_of!($a)).len()) as u32
    };
}

// The three standard streams. msvcrt.h:332 makes these POINTER ARITHMETIC over its own
// `MSVCRT__iob`, not calls:
//
//   #define __acrt_iob_func(idx) (MSVCRT__iob+(idx))
//
// which is why they are macros here too rather than `extern` fns — there is no
// `__acrt_iob_func` symbol to import at this `_MSVCR_VER`, and an `extern` declaration
// would be an undefined reference at link. Note this is msvcrt's OWN table: the CRT
// header's `stdin (&_iob[0])` names a different array, and binding to that one would
// hand callers a FILE the rest of this file never locks or flushes.
macro_rules! stdin {
    () => {
        MSVCRT__iob.as_mut_ptr().add(0)
    };
}
macro_rules! stdout {
    () => {
        MSVCRT__iob.as_mut_ptr().add(1)
    };
}
macro_rules! stderr {
    () => {
        MSVCRT__iob.as_mut_ptr().add(2)
    };
}

extern "C" {
    pub fn _open(path: *const i8, flags: i32, ...) -> i32;
    pub fn _wopen(path: *const wchar_t, flags: i32, ...) -> i32;
    pub fn _wsopen(path: *const wchar_t, oflags: i32, shflags: i32, ...) -> i32;
}


// ---- file-scope preamble (translated) ----
pub const ALL_S_IREAD: u32 = (_S_IREAD | (_S_IREAD >> 3) | (_S_IREAD >> 6));
pub const ALL_S_IWRITE: u32 = (_S_IWRITE | (_S_IWRITE >> 3) | (_S_IWRITE >> 6));
pub const ALL_S_IEXEC: u32 = (_S_IEXEC | (_S_IEXEC >> 3) | (_S_IEXEC >> 6));

pub const MSVCRT_W_OK: i32 = 0x02;
pub const MSVCRT_R_OK: i32 = 0x04;

pub const WX_OPEN: u8 = 0x01;
pub const WX_ATEOF: u8 = 0x02;
pub const WX_READNL: u8 = 0x04;
pub const WX_PIPE: u8 = 0x08;
pub const WX_DONTINHERIT: u8 = 0x10;
pub const WX_APPEND: u8 = 0x20;
pub const WX_TTY: u8 = 0x40;
pub const WX_TEXT: u8 = 0x80;

pub static mut utf8_bom: [u8; 3] = [0xef, 0xbb, 0xbf];
pub static mut utf16_bom: [u8; 2] = [0xff, 0xfe];

pub const MSVCRT_INTERNAL_BUFSIZ: usize = 4096;

/// `file.c:94` — `enum textmode`. TU-LOCAL to file.c, so it cannot come from the
/// msvcrt-sys island for the same reason `ioinfo` does not: the island is built from
/// the module's HEADERS, and neither of these is in one. Spelled `i32` to match the
/// variants the translation already emitted below; it never crosses an ABI boundary
/// (file.c's accessors are `static inline`), so only self-consistency matters here.
// ── hand-integration: names the island cannot supply ────────────────────────
// wine's debug macros. MACROS, never no-op fns: an out-of-line no-op is an
// optimization barrier that uncloaks latent ABI mismatches (-> ud2 -> BUGCHECK).
// Same shape the other translated usermode crates use.

/// `include/crt/stdio.h:59-61`. Plain 0/1/2, but not in the island because they are
/// not on its allowlist; cited rather than assumed.
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;

/// `include/crt/errno.h:44` — `#define ENFILE 23`. Its neighbour EMFILE (24) IS in the
/// island; only this one was left off the allowlist.
pub const ENFILE: i32 = 23;

// The two NT/Win32 enum VARIANTS, under their bindgen-prefixed names. Aliased rather
// than re-declared as literals so the value stays derived from the header: writing
// `4` and `5` here is exactly the invented-constant class that shipped a BUGCHECK.
use msvcrt_sys::{
    _FILE_INFORMATION_CLASS_FileBasicInformation as FileBasicInformation,
    _FILE_INFORMATION_CLASS_FileStandardInformation as FileStandardInformation,
    _GET_FILEEX_INFO_LEVELS_GetFileExInfoStandard as GetFileExInfoStandard,
};

extern "C" {
    /// `msvcrt.h:254` — `extern unsigned int MSVCRT__commode;`, defined in data.c
    /// (still C), so this is an import and not a definition.
    pub static mut MSVCRT__commode: u32;
}

// ── mode-string characters, as CONSTANTS ────────────────────────────────────
// `WC_UC_R` is a cast EXPRESSION, and Rust match patterns cannot contain
// casts — which is why crate-repair declined all 20 of these rather than guess.
// A cast IS legal in a const initialiser, so the value still comes from the
// character itself; writing 82 and 114 here by hand would be the invented-constant
// class that has shipped a BUGCHECK before.
const WC_COMMA: wchar_t = ',' as wchar_t;
const WC_EQUALS: wchar_t = '=' as wchar_t;
const WC_LC_A: wchar_t = 'a' as wchar_t;
const WC_LC_B: wchar_t = 'b' as wchar_t;
const WC_LC_C: wchar_t = 'c' as wchar_t;
const WC_LC_N: wchar_t = 'n' as wchar_t;
const WC_LC_R: wchar_t = 'r' as wchar_t;
const WC_LC_T: wchar_t = 't' as wchar_t;
const WC_LC_W: wchar_t = 'w' as wchar_t;
const WC_PLUS: wchar_t = '+' as wchar_t;
const WC_SPACE: wchar_t = ' ' as wchar_t;
const WC_UC_A: wchar_t = 'A' as wchar_t;
const WC_UC_B: wchar_t = 'B' as wchar_t;
const WC_UC_D: wchar_t = 'D' as wchar_t;
const WC_UC_N: wchar_t = 'N' as wchar_t;
const WC_UC_R: wchar_t = 'R' as wchar_t;
const WC_UC_S: wchar_t = 'S' as wchar_t;
const WC_UC_T: wchar_t = 'T' as wchar_t;
const WC_UC_W: wchar_t = 'W' as wchar_t;


// ── Interlocked*: ATOMICS, not imports ──────────────────────────────────────
// winnt_old.h:8 — `#define InterlockedCompareExchange(a,b,c)
// _InterlockedCompareExchange((long*)a,b,c)` — a MACRO over the compiler intrinsic,
// which clang lowers to an inline `lock cmpxchg`. There is NO symbol, which is why
// the island has none and why an `extern` block here would be an undefined reference
// at link. (ps-rust declares them extern because the KERNEL genuinely exports them;
// usermode msvcrt does not.)
//
// Both return the PREVIOUS value, success or failure — which is exactly the pair of
// arms below, and is what the callers compare against their comparand.
#[inline]
unsafe fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG {
    let a = &*(dest as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(
        comparand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(prev) => prev,
        Err(prev) => prev,
    }
}

#[inline]
unsafe fn InterlockedCompareExchangePointer(
    dest: *mut *mut c_void,
    exchange: *mut c_void,
    comparand: *mut c_void,
) -> *mut c_void {
    let a = &*(dest as *const core::sync::atomic::AtomicPtr<c_void>);
    match a.compare_exchange(
        comparand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(prev) => prev,
        Err(prev) => prev,
    }
}

pub type textmode = i32;

pub const TEXTMODE_ANSI: i32 = 0;
pub const TEXTMODE_UTF8: i32 = 1;
pub const TEXTMODE_UTF16LE: i32 = 2;

pub const MSVCRT_MAX_FILES: usize = 2048;
pub const MSVCRT_FD_BLOCK_SIZE: usize = 32;

#[repr(C)]
pub struct ioinfo {
    pub handle: HANDLE,
    pub wxflag: u8,
    pub lookahead: [u8; 3],
    pub exflag: i32,
    pub crit: CRITICAL_SECTION,
    pub textmode: u8,
    pub pipech2: [u8; 2],
    pub startpos: i64,
    pub utf8translations: BOOL,
    pub dbcsBuffer: [u8; 1],
    pub dbcsBufferUsed: BOOL,
}

#[no_mangle]
pub static mut MSVCRT___badioinfo: ioinfo = ioinfo {
    handle: INVALID_HANDLE_VALUE,
    wxflag: WX_TEXT,
    lookahead: [0; 3],
    exflag: 0,
    crit: CRITICAL_SECTION {
        DebugInfo: core::ptr::null_mut(),
        LockCount: 0,
        RecursionCount: 0,
        OwningThread: core::ptr::null_mut(),
        LockSemaphore: core::ptr::null_mut(),
        SpinCount: 0,
    },
    textmode: 0,
    pipech2: [0; 2],
    startpos: 0,
    utf8translations: 0,
    dbcsBuffer: [0; 1],
    dbcsBufferUsed: 0,
};

#[no_mangle]
pub static mut MSVCRT___pioinfo: [*mut ioinfo; MSVCRT_MAX_FILES / MSVCRT_FD_BLOCK_SIZE] = [core::ptr::null_mut(); MSVCRT_MAX_FILES / MSVCRT_FD_BLOCK_SIZE];

pub const EF_UTF8: i32 = 0x01;
pub const EF_UTF16: i32 = 0x02;
pub const EF_CRIT_INIT: i32 = 0x04;
pub const EF_UNK_UNICODE: i32 = 0x08;

#[repr(C)]
pub struct file_crit {
    pub file: FILE,
    pub crit: CRITICAL_SECTION,
}

#[no_mangle]
pub static mut MSVCRT__iob: [FILE; (_IOB_ENTRIES) as usize] = [FILE {
    _ptr: core::ptr::null_mut(),
    _cnt: 0,
    _base: core::ptr::null_mut(),
    _flag: 0,
    _file: 0,
    _charbuf: 0,
    _bufsiz: 0,
    _tmpfname: core::ptr::null_mut(),
}; (_IOB_ENTRIES) as usize];

pub static mut MSVCRT_fstream: [*mut file_crit; MSVCRT_MAX_FILES / MSVCRT_FD_BLOCK_SIZE] = [core::ptr::null_mut(); MSVCRT_MAX_FILES / MSVCRT_FD_BLOCK_SIZE];

pub static mut MSVCRT_max_streams: i32 = 512;
pub static mut MSVCRT_stream_idx: i32 = 0;
pub static mut MSVCRT_umask: i32 = 0;
pub static mut tmpnam_unique: LONG = 0;
pub static mut tmpnam_s_unique: LONG = 0;

pub const WCEXE: u64 = (('e' as u64) << 32) | (('x' as u64) << 16) | ('e' as u64);
pub const WCBAT: u64 = (('b' as u64) << 32) | (('a' as u64) << 16) | ('t' as u64);
pub const WCCMD: u64 = (('c' as u64) << 32) | (('m' as u64) << 16) | ('d' as u64);
pub const WCCOM: u64 = (('c' as u64) << 32) | (('o' as u64) << 16) | ('m' as u64);

pub static mut MSVCRT_file_cs_debug: RTL_CRITICAL_SECTION_DEBUG = RTL_CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: core::ptr::null_mut(),
    ProcessLocksList: _LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    },
    EntryCount: 0,
    ContentionCount: 0,
    __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 {
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1__bindgen_ty_1 {
            Flags: 0,
            CreatorBackTraceIndexHigh: 0,
            SpareWORD: 0,
        },
    },
};

pub static mut MSVCRT_file_cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};
unsafe extern "C" fn ioinfo_is_crit_init(info: *mut ioinfo) -> u8 {
    if (*info).exflag & EF_CRIT_INIT != 0 { 1u8 } else { 0u8 }
}
unsafe extern "C" fn ioinfo_set_crit_init(info: *mut ioinfo) {
    (*info).exflag |= EF_CRIT_INIT;
}
unsafe extern "C" fn ioinfo_get_textmode(info: *mut ioinfo) -> textmode {
    if (*info).exflag & EF_UTF8 != 0 {
        return TEXTMODE_UTF8;
    }
    if (*info).exflag & EF_UTF16 != 0 {
        return TEXTMODE_UTF16LE;
    }
    TEXTMODE_ANSI
}
unsafe extern "C" fn ioinfo_set_textmode(info: *mut ioinfo, mode: textmode) {
    (*info).exflag &= EF_CRIT_INIT | EF_UNK_UNICODE;
    match mode {
        TEXTMODE_ANSI => {},
        TEXTMODE_UTF8 => {
            (*info).exflag |= EF_UTF8;
        },
        TEXTMODE_UTF16LE => {
            (*info).exflag |= EF_UTF16;
        },
        // `textmode` is a plain i32 alias here (file.c's enum is TU-local), so rustc
        // cannot see the three variants as exhaustive. The C switch has no default.
        _ => {}
    }
}
unsafe extern "C" fn ioinfo_set_unicode(info: *mut ioinfo, unicode: BOOL) {
    if unicode != 0 {
        (*info).exflag |= EF_UNK_UNICODE;
    } else {
        (*info).exflag &= !EF_UNK_UNICODE;
    }
}
unsafe extern "C" fn iob_get_file(i: i32) -> *mut FILE {
    MSVCRT__iob.as_mut_ptr().offset(i as isize)
}
unsafe extern "C" fn file_get_cs(f: *mut FILE) -> *mut CRITICAL_SECTION {
    if f < iob_get_file(0) || f >= iob_get_file((_IOB_ENTRIES) as i32) {
        return core::ptr::addr_of_mut!(((*(f as *mut file_crit)).crit));
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn msvcrt_stat64_to_stat(buf64: *const _stat64, buf: *mut _stat) {
    (*buf).st_dev = (*buf64).st_dev;
    (*buf).st_ino = (*buf64).st_ino;
    (*buf).st_mode = (*buf64).st_mode;
    (*buf).st_nlink = (*buf64).st_nlink;
    (*buf).st_uid = (*buf64).st_uid;
    (*buf).st_gid = (*buf64).st_gid;
    (*buf).st_rdev = (*buf64).st_rdev;
    (*buf).st_size = (*buf64).st_size as i32;
    (*buf).st_atime = (*buf64).st_atime as _;
    (*buf).st_mtime = (*buf64).st_mtime as _;
    (*buf).st_ctime = (*buf64).st_ctime as _;
}
unsafe extern "C" fn msvcrt_stat64_to_stati64(buf64: *const _stat64, buf: *mut _stati64) {
    (*buf).st_dev = (*buf64).st_dev;
    (*buf).st_ino = (*buf64).st_ino;
    (*buf).st_mode = (*buf64).st_mode;
    (*buf).st_nlink = (*buf64).st_nlink;
    (*buf).st_uid = (*buf64).st_uid;
    (*buf).st_gid = (*buf64).st_gid;
    (*buf).st_rdev = (*buf64).st_rdev;
    (*buf).st_size = (*buf64).st_size;
    (*buf).st_atime = (*buf64).st_atime as _;
    (*buf).st_mtime = (*buf64).st_mtime as _;
    (*buf).st_ctime = (*buf64).st_ctime as _;
}
unsafe extern "C" fn msvcrt_stat64_to_stat32(buf64: *const _stat64, buf: *mut _stat32) {
    (*buf).st_dev = (*buf64).st_dev;
    (*buf).st_ino = (*buf64).st_ino;
    (*buf).st_mode = (*buf64).st_mode;
    (*buf).st_nlink = (*buf64).st_nlink;
    (*buf).st_uid = (*buf64).st_uid;
    (*buf).st_gid = (*buf64).st_gid;
    (*buf).st_rdev = (*buf64).st_rdev;
    (*buf).st_size = (*buf64).st_size as i32;
    (*buf).st_atime = (*buf64).st_atime as _;
    (*buf).st_mtime = (*buf64).st_mtime as _;
    (*buf).st_ctime = (*buf64).st_ctime as _;
}
unsafe extern "C" fn msvcrt_stat64_to_stat64i32(buf64: *const _stat64, buf: *mut _stat64i32) {
    (*buf).st_dev = (*buf64).st_dev;
    (*buf).st_ino = (*buf64).st_ino;
    (*buf).st_mode = (*buf64).st_mode;
    (*buf).st_nlink = (*buf64).st_nlink;
    (*buf).st_uid = (*buf64).st_uid;
    (*buf).st_gid = (*buf64).st_gid;
    (*buf).st_rdev = (*buf64).st_rdev;
    (*buf).st_size = (*buf64).st_size as i32;
    (*buf).st_atime = (*buf64).st_atime;
    (*buf).st_mtime = (*buf64).st_mtime;
    (*buf).st_ctime = (*buf64).st_ctime;
}
unsafe extern "C" fn msvcrt_stat64_to_stat32i64(buf64: *const _stat64, buf: *mut _stat32i64) {
    (*buf).st_dev = (*buf64).st_dev;
    (*buf).st_ino = (*buf64).st_ino;
    (*buf).st_mode = (*buf64).st_mode;
    (*buf).st_nlink = (*buf64).st_nlink;
    (*buf).st_uid = (*buf64).st_uid;
    (*buf).st_gid = (*buf64).st_gid;
    (*buf).st_rdev = (*buf64).st_rdev;
    (*buf).st_size = (*buf64).st_size;
    (*buf).st_atime = ((*buf64).st_atime) as i32;
    (*buf).st_mtime = ((*buf64).st_mtime) as i32;
    (*buf).st_ctime = ((*buf64).st_ctime) as i32;
}
unsafe extern "C" fn time_to_filetime(time: __time64_t, ft: *mut FILETIME) {
    let secs_1601_to_1970: __int64 = ((369 * 365 + 89) * 86400 as __int64);

    let ticks: __int64 = (time + secs_1601_to_1970) * 10000000;
    (*ft).dwHighDateTime = (ticks >> 32) as u32;
    (*ft).dwLowDateTime = ticks as u32;
}
unsafe extern "C" fn get_ioinfo_nolock(fd: i32) -> *mut ioinfo {
    let mut ret: *mut ioinfo = core::ptr::null_mut();
    if fd >= 0 && fd < (MSVCRT_MAX_FILES) as i32 {
        ret = MSVCRT___pioinfo[(fd / (MSVCRT_FD_BLOCK_SIZE) as i32) as usize];
    }
    if ret.is_null() {
        return core::ptr::addr_of_mut!(MSVCRT___badioinfo);
    }

    ret.offset((fd % (MSVCRT_FD_BLOCK_SIZE) as i32) as isize)
}
unsafe extern "C" fn init_ioinfo_cs(info: *mut ioinfo) {
    if ioinfo_is_crit_init(info) == 0 {
        LOCK_FILES!();
        if ioinfo_is_crit_init(info) == 0 {
            InitializeCriticalSection(core::ptr::addr_of_mut!((*info).crit));
            ioinfo_set_crit_init(info);
        }
        UNLOCK_FILES!();
    }
}
unsafe extern "C" fn get_ioinfo(fd: i32) -> *mut ioinfo {
    let ret: *mut ioinfo = get_ioinfo_nolock(fd);
    if ret == core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
        return ret;
    }
    init_ioinfo_cs(ret);
    EnterCriticalSection(core::ptr::addr_of_mut!((*ret).crit));
    return ret;
}
unsafe extern "C" fn alloc_pioinfo_block(fd: i32) -> u8 {
    let block: *mut ioinfo;
    let mut i: i32;

    if fd < 0 || fd >= (MSVCRT_MAX_FILES) as i32 {
        *_errno() = ENFILE;
        return FALSE;
    }

    block = calloc(MSVCRT_FD_BLOCK_SIZE as _, core::mem::size_of::<ioinfo>() as _) as *mut ioinfo;
    if block.is_null() {
        *_errno() = (ENOMEM) as i32;
        return FALSE;
    }
    i = 0;
    while i < (MSVCRT_FD_BLOCK_SIZE) as i32 {
        (*block.offset(i as isize)).handle = INVALID_HANDLE_VALUE;
        if ioinfo_is_crit_init(block.offset(i as isize)) != 0 {
            InitializeCriticalSection(core::ptr::addr_of_mut!((*block.offset(i as isize)).crit));
        }
        i = i + 1;
    }
    // The C is `if (InterlockedCompareExchangePointer(&pioinfo[..], block, NULL))` — the CAS
    // returns the PREVIOUS value, so a non-NULL result means another thread published its
    // block first and OURS is the loser that must be freed. A NULL result means we won and
    // the table now owns `block`.
    //
    // Do not "improve" this to a comparison against `block`. That inverts it: the winning
    // path returns NULL, `NULL != block` is true, and the block that was just published into
    // MSVCRT___pioinfo gets freed out from under the table on EVERY successful allocation —
    // every fd block a dangling pointer, critical sections and all. It boots as far as the
    // first msvcrt load and wedges there.
    if !InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!(MSVCRT___pioinfo[(fd / (MSVCRT_FD_BLOCK_SIZE) as i32) as usize]) as *mut *mut core::ffi::c_void,
        block as *mut core::ffi::c_void,
        core::ptr::null_mut()
    ).is_null()
    {
        if ioinfo_is_crit_init(block.offset(0 as isize)) != 0 {
            i = 0;
            while i < (MSVCRT_FD_BLOCK_SIZE) as i32 {
                DeleteCriticalSection(core::ptr::addr_of_mut!((*block.offset(i as isize)).crit));
                i = i + 1;
            }
        }
        free(block as *mut core::ffi::c_void);
    }
    (TRUE) as u8
}
unsafe extern "C" fn get_ioinfo_alloc_fd(fd: i32) -> *mut ioinfo {
    let ret: *mut ioinfo = get_ioinfo(fd);
    if ret != core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
        return ret;
    }

    if alloc_pioinfo_block(fd) == 0 {
        return core::ptr::addr_of_mut!(MSVCRT___badioinfo);
    }

    get_ioinfo(fd)
}
unsafe extern "C" fn get_ioinfo_alloc(fd: *mut i32) -> *mut ioinfo {
    let mut i: i32;

    *fd = -1;
    i = 0;
    while i < (MSVCRT_MAX_FILES) as i32 {
        let mut info: *mut ioinfo = get_ioinfo_nolock(i);

        if info == core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
            if alloc_pioinfo_block(i) == 0 {
                return core::ptr::addr_of_mut!(MSVCRT___badioinfo);
            }
            info = get_ioinfo_nolock(i);
        }

        init_ioinfo_cs(info);
        if TryEnterCriticalSection(core::ptr::addr_of_mut!((*info).crit)) != 0 {
            if (*info).handle == INVALID_HANDLE_VALUE {
                *fd = i;
                return info;
            }
            LeaveCriticalSection(core::ptr::addr_of_mut!((*info).crit));
        }

        i += 1;
    }

    *_errno() = ENFILE;
    return core::ptr::addr_of_mut!(MSVCRT___badioinfo);
}
unsafe extern "C" fn release_ioinfo(info: *mut ioinfo) {
    if info != core::ptr::addr_of_mut!(MSVCRT___badioinfo) && ioinfo_is_crit_init(info) != 0 {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*info).crit));
    }
}
/// `&MSVCRT_fstream[idx]`, indexed the way the C indexes it: WITHOUT a bounds check.
///
/// Not an optimisation — a correctness requirement. `msvcrt_get_file`'s guard is
/// `i >= MSVCRT_max_streams`, and `MSVCRT_max_streams` is a runtime `static mut`, so LLVM
/// cannot prove the index in range and a plain `[idx]` compiles to a bounds check whose
/// panic arm terminates in `ud2`. Four exports inherited that trap through inlining
/// (`_fcloseall`, `_fflush_nolock`, `_rmtmp`, `msvcrt_free_io`) and `scan_ud2` fails the
/// build on it, correctly: a `ud2` reachable from `msvcrt_free_io` is on the process-exit
/// path of every program that links this DLL.
///
/// The index really is always in range, by an invariant no local analysis can see:
/// `_setmaxstdio` refuses any `newmax > MSVCRT_MAX_FILES`, so `i < MSVCRT_max_streams`
/// implies `i / MSVCRT_FD_BLOCK_SIZE < MSVCRT_MAX_FILES / MSVCRT_FD_BLOCK_SIZE`, which is
/// exactly this array's length.
///
/// Contrast `get_ioinfo_nolock`, which indexes the same-shaped `MSVCRT___pioinfo` and needs
/// none of this: it guards against the *constant* `MSVCRT_MAX_FILES`, so the bound is
/// provable and rustc elides the check on its own. The difference between the two is
/// provability, not safety — which is why only one of them needed touching.
#[inline]
unsafe fn fstream_slot(i: i32) -> *mut *mut file_crit {
    core::ptr::addr_of_mut!(MSVCRT_fstream)
        .cast::<*mut file_crit>()
        .add((i / (MSVCRT_FD_BLOCK_SIZE) as i32) as usize)
}

/// `#[inline(never)]` is load-bearing, and is the second half of the `ud2` fix above.
///
/// This function can return NULL (the `calloc` failure path below), and its callers deref the
/// result without checking — `_fcloseall` does `file->_flag`, `_rmtmp` does `file->_tmpfname`,
/// `msvcrt_flush_all_buffers` the same. **That is exactly what the C does**; see file.c. When
/// LLVM inlines this into those loops it can see the NULL reaching a dereference, concludes the
/// path is UB, and folds it to `unreachable` — which lands as `call __errno` immediately
/// followed by `ud2`, one per caller. Out of line, the caller gets an opaque pointer and emits
/// the ordinary load, so the behaviour is the C's to the instruction: a fault, not a trap.
///
/// This hides nothing. The NULL path is unreachable for every one of those callers anyway —
/// they walk `3..MSVCRT_stream_idx`, and a slot below `MSVCRT_stream_idx` was allocated when its
/// stream was created, so `calloc` is never reached with such an `i`. The latent
/// deref-without-check is the C's, is preserved rather than "fixed", and would fault identically
/// in both languages if it ever did happen.
#[inline(never)]
unsafe extern "C" fn msvcrt_get_file(i: i32) -> *mut FILE {
    let mut ret: *mut file_crit;

    if i >= MSVCRT_max_streams {
        return core::ptr::null_mut();
    }

    if i < (_IOB_ENTRIES) as i32 {
        return iob_get_file(i);
    }

    ret = *fstream_slot(i);
    if ret.is_null() {
        *fstream_slot(i) = calloc(
            MSVCRT_FD_BLOCK_SIZE as _,
            core::mem::size_of::<file_crit>() as _,
        ) as *mut file_crit;
        if (*fstream_slot(i)).is_null() {
            *_errno() = (ENOMEM) as i32;
            return core::ptr::null_mut();
        }

        ret = (*fstream_slot(i)).offset((i % (MSVCRT_FD_BLOCK_SIZE) as i32) as isize);
    } else {
        ret = ret.offset((i % (MSVCRT_FD_BLOCK_SIZE) as i32) as isize);
    }

    core::ptr::addr_of_mut!((*ret).file)
}
unsafe extern "C" fn msvcrt_free_fd(fd: i32) {
    let fdinfo = get_ioinfo(fd);

    if fdinfo != core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
        (*fdinfo).handle = INVALID_HANDLE_VALUE;
        (*fdinfo).wxflag = 0;
    }

    if fd < 3 {
        match fd {
            0 => {
                SetStdHandle(STD_INPUT_HANDLE, core::ptr::null_mut());
            }
            1 => {
                SetStdHandle(STD_OUTPUT_HANDLE, core::ptr::null_mut());
            }
            2 => {
                SetStdHandle(STD_ERROR_HANDLE, core::ptr::null_mut());
            }
            _ => {}
        }
    }
    release_ioinfo(fdinfo);
}
unsafe extern "C" fn msvcrt_set_fd(fdinfo: *mut ioinfo, hand: HANDLE, flag: i32) {
    (*fdinfo).handle = hand;
    (*fdinfo).wxflag = WX_OPEN | (flag as u8 & (WX_DONTINHERIT | WX_APPEND | WX_TEXT | WX_PIPE | WX_TTY));
    (*fdinfo).lookahead[0] = (b'\n' as i8) as u8;
    (*fdinfo).lookahead[1] = (b'\n' as i8) as u8;
    (*fdinfo).lookahead[2] = (b'\n' as i8) as u8;
    ioinfo_set_unicode(fdinfo, (FALSE) as i32);
    ioinfo_set_textmode(fdinfo, TEXTMODE_ANSI);

    if hand != MSVCRT_NO_CONSOLE {
        let index = (fdinfo as usize - MSVCRT___pioinfo[0] as usize) / core::mem::size_of::<ioinfo>();
        match index {
            0 => { SetStdHandle(STD_INPUT_HANDLE, hand); }
            1 => { SetStdHandle(STD_OUTPUT_HANDLE, hand); }
            2 => { SetStdHandle(STD_ERROR_HANDLE, hand); }
            _ => {}
        }
    }
}
unsafe extern "C" fn msvcrt_alloc_fd(hand: HANDLE, flag: i32) -> i32 {
    let mut fd: i32 = 0;
    let info: *mut ioinfo = get_ioinfo_alloc(core::ptr::addr_of_mut!(fd));

    if info == core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
        return -1;
    }

    msvcrt_set_fd(info, hand, flag);
    release_ioinfo(info);
    fd
}
unsafe extern "C" fn msvcrt_alloc_fp() -> *mut FILE {
    let mut i: i32 = 0;
    let mut file: *mut FILE;

    while i < MSVCRT_max_streams {
        file = msvcrt_get_file(i);
        if file.is_null() {
            return core::ptr::null_mut();
        }

        if (*file)._flag == 0 {
            if i == MSVCRT_stream_idx {
                let cs: *mut CRITICAL_SECTION = file_get_cs(file);
                if !cs.is_null() {
                    InitializeCriticalSectionEx(cs, 0, RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO);
                    (*(*cs).DebugInfo).__bindgen_anon_1.Spare[0] = b"file_crit.crit\0".as_ptr() as DWORD_PTR;
                }
                MSVCRT_stream_idx += 1;
            }
            return file;
        }
        i += 1;
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn msvcrt_init_fp(file: *mut FILE, fd: i32, stream_flags: u32) -> i32 {
    if (*get_ioinfo_nolock(fd)).wxflag & WX_OPEN == 0 {
        *__doserrno() = 0;
        *_errno() = (EBADF) as i32;
        return -1;
    }
    (*file)._ptr = core::ptr::null_mut();
    (*file)._base = core::ptr::null_mut();
    (*file)._cnt = 0;
    (*file)._file = fd;
    (*file)._flag = (stream_flags) as i32;
    (*file)._tmpfname = core::ptr::null_mut();

    0
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_create_io_inherit_block(size: *mut u16, block: *mut *mut u8) -> BOOL {
    let mut fd: i32;
    let mut last_fd: i32;
    let mut wxflag_ptr: *mut i8;
    let mut handle_ptr: *mut HANDLE;
    let mut fdinfo: *mut ioinfo;

    last_fd = (MSVCRT_MAX_FILES - 1) as i32;
    while last_fd >= 0 {
        if (*get_ioinfo_nolock(last_fd)).handle != INVALID_HANDLE_VALUE {
            break;
        }
        last_fd -= 1;
    }
    last_fd += 1;

    *size = (core::mem::size_of::<u32>() + (core::mem::size_of::<i8>() + core::mem::size_of::<HANDLE>()) * last_fd as usize) as u16;
    *block = calloc(1, (*size) as usize as _) as *mut u8;
    if (*block).is_null() {
        *size = 0;
        return FALSE as BOOL;
    }
    wxflag_ptr = (*block).add(core::mem::size_of::<u32>()) as *mut i8;
    handle_ptr = wxflag_ptr.add(last_fd as usize) as *mut HANDLE;

    *((*block) as *mut u32) = last_fd as u32;
    fd = 0;
    while fd < last_fd {
        fdinfo = get_ioinfo_nolock(fd);
        if ((*fdinfo).wxflag & (WX_OPEN | WX_DONTINHERIT)) == WX_OPEN {
            *wxflag_ptr = (*fdinfo).wxflag as i8;
            *handle_ptr = (*fdinfo).handle;
        } else {
            *wxflag_ptr = 0;
            *handle_ptr = INVALID_HANDLE_VALUE;
        }
        wxflag_ptr = wxflag_ptr.add(1);
        handle_ptr = handle_ptr.add(1);
        fd += 1;
    }
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_io() {
    let mut si: STARTUPINFOA = core::mem::zeroed();
    let mut i: i32;
    let mut fdinfo: *mut ioinfo;

    GetStartupInfoA(&mut si);
    if si.cbReserved2 >= core::mem::size_of::<u32>() as u16 && !si.lpReserved2.is_null() {
        let mut wxflag_ptr: *mut u8;
        let mut handle_ptr: *mut HANDLE;
        let mut count: u32;

        count = *(si.lpReserved2 as *const u32);
        wxflag_ptr = si.lpReserved2.add(core::mem::size_of::<u32>());
        handle_ptr = wxflag_ptr.add(count as usize) as *mut HANDLE;

        count = min!(count, (si.cbReserved2 as u32 - core::mem::size_of::<u32>() as u32) / (core::mem::size_of::<HANDLE>() as u32 + 1));
        count = min!(count, (MSVCRT_MAX_FILES) as u32);
        i = 0;
        while i < count as i32 {
            if (*wxflag_ptr & WX_OPEN) != 0 && GetFileType(*handle_ptr) != FILE_TYPE_UNKNOWN {
                fdinfo = get_ioinfo_alloc_fd(i);
                if fdinfo != &mut MSVCRT___badioinfo as *mut ioinfo {
                    msvcrt_set_fd(fdinfo, *handle_ptr, *wxflag_ptr as i32);
                }
                release_ioinfo(fdinfo);
            }

            wxflag_ptr = wxflag_ptr.add(1);
            handle_ptr = handle_ptr.add(1);
            i += 1;
        }
    }

    fdinfo = get_ioinfo_alloc_fd(STDIN_FILENO);
    if ((*fdinfo).wxflag & WX_OPEN) == 0 || (*fdinfo).handle == INVALID_HANDLE_VALUE {
        let mut h: HANDLE = GetStdHandle(STD_INPUT_HANDLE);
        let mut flags: u32 = (WX_OPEN | WX_TEXT) as u32;
        let type_: u32 = GetFileType(h);

        if type_ == FILE_TYPE_UNKNOWN {
            h = MSVCRT_NO_CONSOLE;
            flags |= (WX_TTY) as u32;
        } else if (type_ & 0xf) == FILE_TYPE_CHAR {
            flags |= (WX_TTY) as u32;
        } else if (type_ & 0xf) == FILE_TYPE_PIPE {
            flags |= (WX_PIPE) as u32;
        }

        msvcrt_set_fd(fdinfo, h, flags as i32);
    }
    release_ioinfo(fdinfo);

    fdinfo = get_ioinfo_alloc_fd(STDOUT_FILENO);
    if ((*fdinfo).wxflag & WX_OPEN) == 0 || (*fdinfo).handle == INVALID_HANDLE_VALUE {
        let mut h: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut flags: u32 = (WX_OPEN | WX_TEXT) as u32;
        let type_: u32 = GetFileType(h);

        if type_ == FILE_TYPE_UNKNOWN {
            h = MSVCRT_NO_CONSOLE;
            flags |= (WX_TTY) as u32;
        } else if (type_ & 0xf) == FILE_TYPE_CHAR {
            flags |= (WX_TTY) as u32;
        } else if (type_ & 0xf) == FILE_TYPE_PIPE {
            flags |= (WX_PIPE) as u32;
        }

        msvcrt_set_fd(fdinfo, h, flags as i32);
    }
    release_ioinfo(fdinfo);

    fdinfo = get_ioinfo_alloc_fd(STDERR_FILENO);
    if ((*fdinfo).wxflag & WX_OPEN) == 0 || (*fdinfo).handle == INVALID_HANDLE_VALUE {
        let mut h: HANDLE = GetStdHandle(STD_ERROR_HANDLE);
        let mut flags: u32 = (WX_OPEN | WX_TEXT) as u32;
        let type_: u32 = GetFileType(h);

        if type_ == FILE_TYPE_UNKNOWN {
            h = MSVCRT_NO_CONSOLE;
            flags |= (WX_TTY) as u32;
        } else if (type_ & 0xf) == FILE_TYPE_CHAR {
            flags |= (WX_TTY) as u32;
        } else if (type_ & 0xf) == FILE_TYPE_PIPE {
            flags |= (WX_PIPE) as u32;
        }

        msvcrt_set_fd(fdinfo, h, flags as i32);
    }
    release_ioinfo(fdinfo);

    i = 0;
    while i < 3 {
        let f: *mut FILE = iob_get_file(i);
        let cs: *mut CRITICAL_SECTION = file_get_cs(f);

        (*f)._file = if (*get_ioinfo_nolock(i)).handle == MSVCRT_NO_CONSOLE {
            MSVCRT_NO_CONSOLE_FD
        } else {
            i
        };
        (*f)._tmpfname = core::ptr::null_mut();
        (*f)._flag = if i == 0 { (_IOREAD) as i32 } else { (_IOWRT) as i32 };

        if !cs.is_null() {
            InitializeCriticalSectionEx(cs, 0, RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO);
            (*(*cs).DebugInfo).__bindgen_anon_1.Spare[0] = b"file.c: file_crit.crit\0".as_ptr() as DWORD_PTR;
        }
        i += 1;
    }
    MSVCRT_stream_idx = 3;
}
unsafe extern "C" fn msvcrt_flush_buffer(file: *mut FILE) -> i32 {
    let mut ret = 0;

    if ((*file)._flag & ((_IOREAD | _IOWRT)) as i32) == (_IOWRT) as i32 &&
            ((*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32) != 0 {
        let cnt = (*file)._ptr.offset_from((*file)._base) as i32;
        if cnt > 0 && _write((*file)._file, (*file)._base as *const core::ffi::c_void, cnt as u32) != cnt {
            (*file)._flag |= (_IOERR) as i32;
            ret = EOF;
        } else if ((*file)._flag & (_IORW) as i32) != 0 {
            (*file)._flag &= (!_IOWRT) as i32;
        }
    }

    (*file)._ptr = (*file)._base;
    (*file)._cnt = 0;
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _isatty(fd: i32) -> i32 {
    ((*get_ioinfo_nolock(fd)).wxflag & WX_TTY) as i32
}
unsafe extern "C" fn msvcrt_alloc_buffer(file: *mut FILE) -> i32 {
    if ((*file)._file == STDOUT_FILENO || (*file)._file == STDERR_FILENO)
        && _isatty((*file)._file) != 0
    {
        return (FALSE) as i32;
    }

    (*file)._base = calloc(1, MSVCRT_INTERNAL_BUFSIZ as _) as *mut i8;
    if !(*file)._base.is_null() {
        (*file)._bufsiz = (MSVCRT_INTERNAL_BUFSIZ) as i32;
        (*file)._flag |= (_IOMYBUF) as i32;
    } else {
        (*file)._base = core::ptr::addr_of_mut!((*file)._charbuf) as *mut i8;
        (*file)._bufsiz = 2;
        (*file)._flag |= (MSVCRT__NOBUF) as i32;
    }
    (*file)._ptr = (*file)._base;
    (*file)._cnt = 0;
    (TRUE) as i32
}
unsafe extern "C" fn add_std_buffer(file: *mut FILE) -> u8 {
    static mut buffers: [[i8; (BUFSIZ) as usize]; 2] = [[0; (BUFSIZ) as usize]; 2];

    if ((*file)._file != STDOUT_FILENO && (*file)._file != STDERR_FILENO)
        || ((*file)._flag & ((MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)) as i32) != 0
        || _isatty((*file)._file) == 0
    {
        return FALSE;
    }

    let buffer_index = if (*file)._file == STDOUT_FILENO {
        0
    } else {
        1
    };

    (*file)._ptr = buffers[buffer_index].as_mut_ptr();
    (*file)._base = buffers[buffer_index].as_mut_ptr();
    (*file)._bufsiz = (BUFSIZ) as i32;
    (*file)._cnt = (BUFSIZ) as i32;
    (*file)._flag |= (MSVCRT__USERBUF) as i32;
    (TRUE) as u8
}
unsafe extern "C" fn remove_std_buffer(file: *mut FILE) {
    msvcrt_flush_buffer(file);
    (*file)._ptr = core::ptr::null_mut();
    (*file)._base = core::ptr::null_mut();
    (*file)._bufsiz = 0;
    (*file)._cnt = 0;
    (*file)._flag &= (!MSVCRT__USERBUF) as i32;
}
unsafe extern "C" fn msvcrt_int_to_base32(mut num: i32, str: PCHAR) -> i32 {
    let mut p: PCHAR;
    let mut n = num;
    let mut digits = 0;

    while n != 0 {
        n >>= 5;
        digits += 1;
    }
    p = str.offset(digits as isize);
    *p = 0;
    p = p.offset(-1);
    while p >= str {
        *p = (((num & 31) + b'0' as i32) as u8) as i8;
        if *p > (b'9') as i8 {
            *p = ((*p as i32 + (b'a' as i32 - b'0' as i32 - 10)) as u8) as i8;
        }
        let num_mut = core::ptr::addr_of_mut!(num);
        *num_mut >>= 5;
        p = p.offset(-1);
    }

    digits
}
unsafe extern "C" fn msvcrt_int_to_base32_w(mut num: i32, str: *mut wchar_t) -> i32 {
    let mut p: *mut wchar_t;
    let mut n = num;
    let mut digits = 0;

    while n != 0 {
        n >>= 5;
        digits += 1;
    }
    p = str.offset(digits as isize);
    *p = 0;
    p = p.offset(-1);
    while p >= str {
        *p = ((num & 31) + '0' as i32) as wchar_t;
        if *p > '9' as wchar_t {
            *p = (*p as i32 + ('a' as i32 - '0' as i32 - 10)) as wchar_t;
        }
        // `num >>= 5`, and it has to MUTATE. This was `let num = num_local;`, which is a
        // fresh binding scoped to the loop body — it shadows for one iteration and dies,
        // leaving the outer `num` unshifted, so every base-32 digit came out of the same
        // value. `\s1lu6v.` became `\svvvvv.`, and the per-process uniqueness the encoder
        // exists for went with it. The narrow twin thirty lines up got the same C
        // statement right (via addr_of_mut!), which is the tell: one file, one statement,
        // two translations, one of them silent.
        num >>= 5;
        p = p.offset(-1);
    }

    digits
}

#[no_mangle]
pub unsafe extern "C" fn __iob_func() -> *mut FILE {
    return iob_get_file(0);
}

#[no_mangle]
pub unsafe extern "C" fn _access(filename: *const i8, mode: i32) -> i32 {
    let mut filenameW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !filename.is_null() {
        filenameW = wstrdupa_utf8(filename);
        if filenameW.is_null() {
            return -1;
        }
    }
    ret = _waccess(filenameW, mode);
    free((filenameW).cast());
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _access_s(filename: *const i8, mode: i32) -> i32 {
    if MSVCRT_CHECK_PMT!(!filename.is_null()) == 0 {
        return *_errno();
    }
    if MSVCRT_CHECK_PMT!((mode & !((MSVCRT_R_OK | MSVCRT_W_OK))) == 0 as i32) == 0 {
        return *_errno();
    }

    if _access(filename, mode) == -(1 as i32) {
        return *_errno();
    }
    0 as i32
}

#[no_mangle]
pub unsafe extern "C" fn _waccess(filename: *const wchar_t, mode: i32) -> i32 {
    let attr: u32 = GetFileAttributesW(filename);

    if filename.is_null() || attr == INVALID_FILE_ATTRIBUTES {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    if (attr & FILE_ATTRIBUTE_READONLY) != 0 && (mode & MSVCRT_W_OK) != 0 {
        msvcrt_set_errno((ERROR_ACCESS_DENIED) as i32);
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _waccess_s(filename: *const wchar_t, mode: i32) -> i32 {
    if MSVCRT_CHECK_PMT!(filename != core::ptr::null()) == 0 {
        return *_errno();
    }
    if MSVCRT_CHECK_PMT!((mode & !((MSVCRT_R_OK | MSVCRT_W_OK))) == 0 as i32) == 0 {
        return *_errno();
    }

    if _waccess(filename, mode) == -(1 as i32) {
        return *_errno();
    }
    return 0 as i32;
}

#[no_mangle]
pub unsafe extern "C" fn _chmod(path: *const i8, flags: i32) -> i32 {
    let mut pathW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !path.is_null() {
        pathW = wstrdupa_utf8(path);
        if pathW.is_null() {
            return -1;
        }
    }
    ret = _wchmod(pathW, flags);
    free((pathW).cast());
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _wchmod(path: *const wchar_t, flags: i32) -> i32 {
    let oldFlags: u32 = GetFileAttributesW(path);

    if oldFlags != INVALID_FILE_ATTRIBUTES {
        let newFlags: u32 = if (flags & (_S_IWRITE) as i32) != 0 {
            oldFlags & !FILE_ATTRIBUTE_READONLY
        } else {
            oldFlags | FILE_ATTRIBUTE_READONLY
        };

        if newFlags == oldFlags || SetFileAttributesW(path, newFlags) != 0 {
            return 0;
        }
    }
    msvcrt_set_errno((GetLastError()) as i32);
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _unlink(path: *const i8) -> i32 {
    let mut pathW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !path.is_null() {
        pathW = wstrdupa_utf8(path);
        if pathW.is_null() {
            return -1;
        }
    }
    ret = _wunlink(pathW);
    free(pathW as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _wunlink(path: *const wchar_t) -> i32 {
    if DeleteFileW(path) != 0 {
        return 0;
    }
    msvcrt_set_errno((GetLastError()) as i32);
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _commit(fd: i32) -> i32 {
    let info: *mut ioinfo = get_ioinfo(fd);
    let ret: i32;

    if (*info).handle == INVALID_HANDLE_VALUE {
        ret = -1;
    } else if FlushFileBuffers((*info).handle) == 0 {
        if GetLastError() == ERROR_INVALID_HANDLE {
            ret = 0;
        } else {
            msvcrt_set_errno((GetLastError()) as i32);
            ret = -1;
        }
    } else {
        ret = 0;
    }

    release_ioinfo(info);
    ret
}
unsafe extern "C" fn msvcrt_flush_all_buffers(mask: i32) -> i32 {
    let mut num_flushed: i32 = 0;
    let mut file: *mut FILE;

    LOCK_FILES!();
    let mut i: i32 = 0;
    while i < MSVCRT_stream_idx {
        file = msvcrt_get_file(i);

        if (*file)._flag != 0 {
            if ((*file)._flag & mask) != 0 {
                fflush(file);
                num_flushed += 1;
            }
        }
        i += 1;
    }
    UNLOCK_FILES!();

    num_flushed
}

#[no_mangle]
pub unsafe extern "C" fn _flushall() -> i32 {
    msvcrt_flush_all_buffers((_IOWRT | _IOREAD) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn fflush(file: *mut FILE) -> i32 {
    let ret: i32;

    if file.is_null() {
        msvcrt_flush_all_buffers((_IOWRT) as i32);
        ret = 0;
    } else {
        _lock_file(file);
        ret = _fflush_nolock(file);
        _unlock_file(file);
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fflush_nolock(file: *mut FILE) -> i32 {
    let res: i32;

    if file.is_null() {
        msvcrt_flush_all_buffers((_IOWRT) as i32);
        return 0;
    }

    res = msvcrt_flush_buffer(file);
    if res == 0 && ((*file)._flag & (MSVCRT__IOCOMMIT) as i32) != 0 {
        if _commit((*file)._file) != 0 {
            return EOF;
        } else {
            return 0;
        }
    }
    res
}

#[no_mangle]
pub unsafe extern "C" fn _close(fd: i32) -> i32 {
    let info: *mut ioinfo = get_ioinfo(fd);
    let ret: i32;

    if fd == MSVCRT_NO_CONSOLE_FD {
        *_errno() = (EBADF) as i32;
        ret = -1;
    } else if MSVCRT_CHECK_PMT_ERR!(((*info).wxflag & WX_OPEN) != 0, (EBADF) as i32) == 0 {
        ret = -1;
    } else if fd == STDOUT_FILENO && (*info).handle == (*get_ioinfo_nolock(2i32)).handle {
        msvcrt_free_fd(fd);
        ret = 0;
    } else if fd == STDERR_FILENO && (*info).handle == (*get_ioinfo_nolock(1i32)).handle {
        msvcrt_free_fd(fd);
        ret = 0;
    } else {
        ret = if CloseHandle((*info).handle) != 0 { 0 } else { -1 };
        msvcrt_free_fd(fd);
        if ret != 0 {
            msvcrt_set_errno((GetLastError()) as i32);
        }
    }
    release_ioinfo(info);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _dup2(od: i32, nd: i32) -> i32 {
    let info_od: *mut ioinfo;
    let info_nd: *mut ioinfo;
    let ret: i32;

    if od < nd {
        info_od = get_ioinfo(od);
        info_nd = get_ioinfo_alloc_fd(nd);
    } else {
        info_nd = get_ioinfo_alloc_fd(nd);
        info_od = get_ioinfo(od);
    }

    if info_nd == core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
        *_errno() = (EBADF) as i32;
        ret = -1;
    } else if (*info_od).wxflag & WX_OPEN != 0 {
        let mut handle: HANDLE = core::ptr::null_mut();

        if DuplicateHandle(
            GetCurrentProcess(),
            (*info_od).handle,
            GetCurrentProcess(),
            &mut handle,
            0,
            (TRUE) as i32,
            DUPLICATE_SAME_ACCESS,
        ) != 0
        {
            let wxflag: i32 = ((*info_od).wxflag & !WX_DONTINHERIT) as i32;

            if (*info_nd).wxflag & WX_OPEN != 0 {
                _close(nd);
            }

            msvcrt_set_fd(info_nd, handle, wxflag);
            ret = 0;
        } else {
            ret = -1;
            msvcrt_set_errno((GetLastError()) as i32);
        }
    } else {
        *_errno() = (EBADF) as i32;
        ret = -1;
    }

    release_ioinfo(info_od);
    release_ioinfo(info_nd);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _dup(od: i32) -> i32 {
    let mut fd: i32 = 0;
    let info: *mut ioinfo = get_ioinfo_alloc(core::ptr::addr_of_mut!(fd));

    let mut ret: i32 = if _dup2(od, fd) == 0 {
        fd
    } else {
        -1
    };
    release_ioinfo(info);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _eof(fd: i32) -> i32 {
    let info: *mut ioinfo = get_ioinfo(fd);
    let mut curpos: u32;
    let mut endpos: u32;
    let mut hcurpos: i32;
    let mut hendpos: i32;

    if (*info).handle == INVALID_HANDLE_VALUE {
        release_ioinfo(info);
        return -1;
    }

    if ((*info).wxflag & WX_ATEOF) != 0 {
        release_ioinfo(info);
        return (TRUE) as i32;
    }

    hcurpos = 0;
    hendpos = 0;
    curpos = SetFilePointer(
        (*info).handle,
        0,
        &mut hcurpos,
        FILE_CURRENT
    );
    endpos = SetFilePointer(
        (*info).handle,
        0,
        &mut hendpos,
        FILE_END
    );

    if curpos == endpos && hcurpos == hendpos {
        release_ioinfo(info);
        return (TRUE) as i32;
    }

    SetFilePointer(
        (*info).handle,
        (curpos) as i32,
        &mut hcurpos,
        FILE_BEGIN
    );
    release_ioinfo(info);
    return (FALSE) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn _fcloseall() -> i32 {
    let mut num_closed: i32 = 0;
    let mut file: *mut FILE;

    LOCK_FILES!();
    for i in 3..MSVCRT_stream_idx {
        file = msvcrt_get_file(i);

        if (*file)._flag != 0 && fclose(file) == 0 {
            num_closed += 1;
        }
    }
    UNLOCK_FILES!();

    num_closed
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_io() {
    let mut i: u32;
    let mut j: i32;

    _flushall();
    _fcloseall();

    i = 0;
    while i < ARRAY_SIZE!(MSVCRT___pioinfo) {
        if MSVCRT___pioinfo[i as usize].is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        j = 0;
        while j < (MSVCRT_FD_BLOCK_SIZE) as i32 {
            if ioinfo_is_crit_init(core::ptr::addr_of_mut!((*MSVCRT___pioinfo[i as usize].offset(j as isize)))) != 0 {
                DeleteCriticalSection(core::ptr::addr_of_mut!((*MSVCRT___pioinfo[i as usize].offset(j as isize)).crit));
            }
            j = j.wrapping_add(1);
        }
        free(MSVCRT___pioinfo[i as usize] as *mut _);
        i = i.wrapping_add(1);
    }

    j = 0;
    while j < MSVCRT_stream_idx {
        let file: *mut FILE = msvcrt_get_file(j);
        let cs: *mut CRITICAL_SECTION = file_get_cs(file);

        if !cs.is_null() {
            (*(*cs).DebugInfo).__bindgen_anon_1.Spare[0] = 0;
            DeleteCriticalSection(cs);
        }
        j = j.wrapping_add(1);
    }

    i = 0;
    while i < ARRAY_SIZE!(MSVCRT_fstream) {
        free(MSVCRT_fstream[i as usize] as *mut _);
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _lseeki64(fd: i32, offset: __int64, whence: i32) -> __int64 {
    let info = get_ioinfo(fd);
    let mut ofs: LARGE_INTEGER = core::mem::zeroed();

    if (*info).handle == INVALID_HANDLE_VALUE {
        *_errno() = (EBADF) as i32;
        release_ioinfo(info);
        return -1;
    }

    if whence < 0 || whence > 2 {
        release_ioinfo(info);
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    // The C is `ofs.QuadPart = offset; ofs.u.LowPart = SetFilePointer(h, ofs.u.LowPart,
    // &ofs.u.HighPart, whence)`. The translation rewrote it as pointer casts and reached
    // HighPart with `.offset(1)` — a hand-rolled layout offset, the exact pattern
    // scan_magic_offsets.py forbids. Use the union's own fields; the layout then comes
    // from the header via bindgen instead of from arithmetic.
    //
    // SetFilePointerEx is deliberately NOT used here, per file.c's own comment: the
    // MoleBox protection scheme expects msvcrt to call SetFilePointer only.
    ofs.QuadPart = offset;
    ofs.u.LowPart = SetFilePointer(
        (*info).handle,
        ofs.u.LowPart as LONG,
        core::ptr::addr_of_mut!(ofs.u.HighPart),
        (whence) as u32,
    );
    if ofs.u.LowPart != INVALID_SET_FILE_POINTER || GetLastError() == ERROR_SUCCESS {
        (*info).wxflag &= !WX_ATEOF;
        release_ioinfo(info);
        return ofs.QuadPart;
    }
    release_ioinfo(info);
    msvcrt_set_errno((GetLastError()) as i32);
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _lseek(fd: i32, offset: __msvcrt_long, whence: i32) -> __msvcrt_long {
    (_lseeki64(fd, (offset) as i64, whence)) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _lock_file(file: *mut FILE) {
    let cs: *mut CRITICAL_SECTION = file_get_cs(file);
    if cs.is_null() {
        _lock((_STREAM_LOCKS + ((file.offset_from(iob_get_file(0)) as i32)) as u32) as i32);
    } else {
        EnterCriticalSection(cs);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _unlock_file(file: *mut FILE) {
    let cs: *mut CRITICAL_SECTION = file_get_cs(file);
    if cs.is_null() {
        _unlock((_STREAM_LOCKS + ((file.offset_from(iob_get_file(0)) as i32)) as u32) as i32);
    } else {
        LeaveCriticalSection(cs);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _locking(fd: i32, mode: i32, nbytes: __msvcrt_long) -> i32 {
    let info: *mut ioinfo = get_ioinfo(fd);
    let mut ret: BOOL;
    let cur_locn: DWORD;

    if (*info).handle == INVALID_HANDLE_VALUE {
        release_ioinfo(info);
        return -1;
    }

    if mode < 0 || mode > 4 {
        release_ioinfo(info);
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    cur_locn = SetFilePointer((*info).handle, 0, core::ptr::null_mut(), FILE_CURRENT);
    if cur_locn == INVALID_SET_FILE_POINTER {
        release_ioinfo(info);
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    if mode == (_LK_LOCK) as i32 || mode == (_LK_RLCK) as i32 {
        let mut nretry: i32 = 10;
        ret = 1;
        while nretry > 0 {
            nretry -= 1;
            ret = LockFile((*info).handle, cur_locn, 0, (nbytes) as u32, 0);
            if ret != 0 {
                break;
            }
            Sleep(1);
        }
    } else if mode == (_LK_UNLCK) as i32 {
        ret = UnlockFile((*info).handle, cur_locn, 0, (nbytes) as u32, 0);
    } else {
        ret = LockFile((*info).handle, cur_locn, 0, (nbytes) as u32, 0);
    }

    release_ioinfo(info);
    if ret != 0 {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn _fseeki64(FILE: *mut FILE, offset: __int64, whence: i32) -> i32 {
    let ret: i32;

    _lock_file(FILE);
    ret = _fseeki64_nolock(FILE, offset, whence);
    _unlock_file(FILE);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _fseeki64_nolock(FILE: *mut FILE, mut offset: __int64, mut whence: i32) -> i32 {
    let ret: i32;

    if whence == (SEEK_CUR) as i32 && (*FILE)._flag & (_IOREAD) as i32 != 0 {
        whence = (SEEK_SET) as i32;
        offset += _ftelli64_nolock(FILE);
    }

    msvcrt_flush_buffer(FILE);
    if (*FILE)._flag & (_IORW) as i32 != 0 {
        (*FILE)._flag &= (!(_IOREAD | _IOWRT)) as i32;
    }
    (*FILE)._flag &= (!_IOEOF) as i32;
    ret = if _lseeki64((*FILE)._file, offset, whence) == -1 {
        -1
    } else {
        0
    };

    ret
}

#[no_mangle]
pub unsafe extern "C" fn fseek(file: *mut FILE, offset: __msvcrt_long, whence: i32) -> i32 {
    _fseeki64(file, (offset) as i64, whence)
}

#[no_mangle]
pub unsafe extern "C" fn _fseek_nolock(
    file: *mut FILE,
    offset: __msvcrt_long,
    whence: i32,
) -> i32 {
    _fseeki64_nolock(file, (offset) as i64, whence)
}

#[no_mangle]
pub unsafe extern "C" fn _chsize_s(fd: i32, size: __int64) -> i32 {
    let mut info: *mut ioinfo;
    let mut cur: __int64;
    let mut pos: __int64;
    let mut ret: BOOL = (FALSE) as i32;

    if MSVCRT_CHECK_PMT!(size >= 0) == 0 {
        return (EINVAL) as i32;
    }

    info = get_ioinfo(fd);
    if (*info).handle != INVALID_HANDLE_VALUE {
        cur = _lseeki64(fd, 0, (SEEK_CUR) as i32);
        if cur >= 0 {
            pos = _lseeki64(fd, size, (SEEK_SET) as i32);
            if pos >= 0 {
                ret = SetEndOfFile((*info).handle);
                if ret == 0 {
                    msvcrt_set_errno((GetLastError()) as i32);
                }
            }

            _lseeki64(fd, cur, (SEEK_SET) as i32);
        }
    }

    release_ioinfo(info);
    if ret != 0 {
        0
    } else {
        *_errno()
    }
}

#[no_mangle]
pub unsafe extern "C" fn _chsize(fd: i32, size: __msvcrt_long) -> i32 {
    if _chsize_s(fd, size as __int64) == 0 {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn clearerr(file: *mut FILE) {
    _lock_file(file);
    (*file)._flag &= (!(_IOERR | _IOEOF)) as i32;
    _unlock_file(file);
}

#[no_mangle]
pub unsafe extern "C" fn clearerr_s(file: *mut FILE) -> i32 {
    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }

    _lock_file(file);
    (*file)._flag &= (!(_IOERR | _IOEOF)) as i32;
    _unlock_file(file);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn rewind(file: *mut FILE) {
    _lock_file(file);
    _fseek_nolock(file, 0, (SEEK_SET) as i32);
    clearerr(file);
    _unlock_file(file);
}
unsafe extern "C" fn msvcrt_get_flags(mode: *const wchar_t, open_flags: *mut i32, stream_flags: *mut i32) -> i32 {
    let mut mode = mode;
    let plus = (wcschr(mode, WC_PLUS) != core::ptr::null_mut()) as i32;

    while *mode == WC_SPACE {
        mode = mode.offset(1);
    }

    match *mode {
        WC_UC_R | WC_LC_R => {
            mode = mode.offset(1);
            *open_flags = if plus != 0 { (_O_RDWR) as i32 } else { (_O_RDONLY) as i32 };
            *stream_flags = if plus != 0 { (_IORW) as i32 } else { (_IOREAD) as i32 };
        }
        WC_UC_W | WC_LC_W => {
            mode = mode.offset(1);
            *open_flags = (_O_CREAT | _O_TRUNC | if plus != 0 { _O_RDWR } else { _O_WRONLY }) as i32;
            *stream_flags = if plus != 0 { (_IORW) as i32 } else { (_IOWRT) as i32 };
        }
        WC_UC_A | WC_LC_A => {
            mode = mode.offset(1);
            *open_flags = (_O_CREAT | _O_APPEND | if plus != 0 { _O_RDWR } else { _O_WRONLY }) as i32;
            *stream_flags = if plus != 0 { (_IORW) as i32 } else { (_IOWRT) as i32 };
        }
        _ => {
            MSVCRT_INVALID_PMT!(0, (EINVAL) as i32);
            return -1;
        }
    }

    *stream_flags |= MSVCRT__commode as i32;

    while *mode != 0 && *mode != WC_COMMA {
        match *mode {
            WC_UC_B | WC_LC_B => {
                mode = mode.offset(1);
                *open_flags |= (_O_BINARY) as i32;
                *open_flags &= (!_O_TEXT) as i32;
            }
            WC_LC_T => {
                mode = mode.offset(1);
                *open_flags |= (_O_TEXT) as i32;
                *open_flags &= (!_O_BINARY) as i32;
            }
            WC_UC_D => {
                mode = mode.offset(1);
                *open_flags |= (_O_TEMPORARY) as i32;
            }
            WC_UC_T => {
                mode = mode.offset(1);
                *open_flags |= (_O_SHORT_LIVED) as i32;
            }
            WC_LC_C => {
                mode = mode.offset(1);
                *stream_flags |= (MSVCRT__IOCOMMIT) as i32;
            }
            WC_LC_N => {
                mode = mode.offset(1);
                *stream_flags &= (!MSVCRT__IOCOMMIT) as i32;
            }
            WC_UC_N => {
                mode = mode.offset(1);
                *open_flags |= (_O_NOINHERIT) as i32;
            }
            WC_PLUS | WC_SPACE | WC_LC_A | WC_LC_W => {
                mode = mode.offset(1);
            }
            WC_UC_S => {
                mode = mode.offset(1);
                if (*open_flags & (_O_RANDOM) as i32) == 0 {
                    *open_flags |= (_O_SEQUENTIAL) as i32;
                }
            }
            WC_UC_R => {
                mode = mode.offset(1);
                if (*open_flags & (_O_SEQUENTIAL) as i32) == 0 {
                    *open_flags |= (_O_RANDOM) as i32;
                }
            }
            _ => {
                ERR!("incorrect mode flag: %c\n", *mode.offset(-1) as core::ffi::c_int);
                mode = mode.offset(1);
            }
        }
    }

    if *mode == WC_COMMA {
        mode = mode.offset(1);
        while *mode == WC_SPACE {
            mode = mode.offset(1);
        }
        if MSVCRT_CHECK_PMT!(wcsncmp(W_CCS.as_ptr(), mode, 3) == 0) == 0 {
            return -1;
        }
        mode = mode.offset(3);
        while *mode == WC_SPACE {
            mode = mode.offset(1);
        }
        if MSVCRT_CHECK_PMT!(*mode == WC_EQUALS) == 0 {
            return -1;
        }
        mode = mode.offset(1);
        while *mode == WC_SPACE {
            mode = mode.offset(1);
        }

        if _wcsnicmp(W_UTF8.as_ptr(), mode, 5) == 0 {
            *open_flags |= (_O_U8TEXT) as i32;
            mode = mode.offset(5);
        } else if _wcsnicmp(W_UTF16LE.as_ptr(), mode, 8) == 0 {
            *open_flags |= (_O_U16TEXT) as i32;
            mode = mode.offset(8);
        } else if _wcsnicmp(W_UNICODE.as_ptr(), mode, 7) == 0 {
            *open_flags |= (_O_WTEXT) as i32;
            mode = mode.offset(7);
        } else {
            MSVCRT_INVALID_PMT!(0, (EINVAL) as i32);
            return -1;
        }

        while *mode == WC_SPACE {
            mode = mode.offset(1);
        }
    }

    if MSVCRT_CHECK_PMT!(*mode == 0) == 0 {
        return -1;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _fdopen(fd: i32, mode: *const i8) -> *mut FILE {
    let mut ret: *mut FILE;
    let mut modeW: *mut wchar_t = core::ptr::null_mut();

    if !mode.is_null() {
        modeW = msvcrt_wstrdupa(mode);
        if modeW.is_null() {
            return core::ptr::null_mut();
        }
    }

    ret = _wfdopen(fd, modeW);

    free(modeW as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _wfdopen(fd: i32, mode: *const wchar_t) -> *mut FILE {
    let mut open_flags: i32 = 0;
    let mut stream_flags: i32 = 0;
    let mut file: *mut FILE;

    if msvcrt_get_flags(mode, &mut open_flags, &mut stream_flags) == -1 {
        return core::ptr::null_mut();
    }

    LOCK_FILES!();
    file = msvcrt_alloc_fp();
    if file.is_null() {
        file = core::ptr::null_mut();
    } else if msvcrt_init_fp(file, fd, stream_flags as u32) == -1 {
        (*file)._flag = 0;
        file = core::ptr::null_mut();
    } else {
        TRACE!(b":fd (%d) mode (%s) FILE* (%p)\n\0".as_ptr() as *const i8, fd, debugstr_w(mode), file);
    }
    UNLOCK_FILES!();

    file
}

#[no_mangle]
pub unsafe extern "C" fn _filelength(fd: i32) -> __msvcrt_long {
    let curPos = _lseek(fd, 0, (SEEK_CUR) as i32);
    if curPos != -1 {
        let endPos = _lseek(fd, 0, (SEEK_END) as i32);
        if endPos != -1 {
            if endPos != curPos {
                _lseek(fd, curPos, (SEEK_SET) as i32);
            }
            return endPos;
        }
    }
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _filelengthi64(fd: i32) -> __int64 {
    let curPos = _lseeki64(fd, 0, (SEEK_CUR) as i32);
    if curPos != -1 {
        let endPos = _lseeki64(fd, 0, (SEEK_END) as i32);
        if endPos != -1 {
            if endPos != curPos {
                _lseeki64(fd, curPos, (SEEK_SET) as i32);
            }
            return endPos;
        }
    }
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _fileno(file: *mut FILE) -> i32 {
    // TRACE(":FILE* (%p) fd (%d)\n",file,file->_file);
    (*file)._file
}

#[no_mangle]
pub unsafe extern "C" fn _fstat64(fd: i32, buf: *mut _stat64) -> i32 {
    let info: *mut ioinfo = get_ioinfo(fd);
    let mut dw: u32 = 0;
    let type_: u32;

    if (*info).handle == INVALID_HANDLE_VALUE {
        release_ioinfo(info);
        return -1;
    }

    if buf.is_null() {
        msvcrt_set_errno((ERROR_INVALID_PARAMETER) as i32);
        release_ioinfo(info);
        return -1;
    }

    memset(buf as *mut core::ffi::c_void, 0, core::mem::size_of::<_stat64>());
    type_ = GetFileType((*info).handle);
    if type_ == FILE_TYPE_PIPE {
        (*buf).st_dev = fd as u32;
        (*buf).st_rdev = fd as u32;
        (*buf).st_mode = _S_IFIFO as u16;
        (*buf).st_nlink = 1;
    } else if type_ == FILE_TYPE_CHAR {
        (*buf).st_dev = fd as u32;
        (*buf).st_rdev = fd as u32;
        (*buf).st_mode = _S_IFCHR as u16;
        (*buf).st_nlink = 1;
    } else {
        let mut basic_info: FILE_BASIC_INFORMATION = core::mem::zeroed();
        let mut std_info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
        let mut io: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut status: i32;

        status = NtQueryInformationFile((*info).handle, &mut io, &mut basic_info as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32, FileBasicInformation);
        if status == 0 {
            status = NtQueryInformationFile((*info).handle, &mut io, &mut std_info as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32, FileStandardInformation);
        }
        if status != 0 {
            msvcrt_set_errno((ERROR_INVALID_PARAMETER) as i32);
            release_ioinfo(info);
            return -1;
        }
        // 0o444 / 0o222, NOT 0444 / 0222. The C is octal; Rust reads a leading zero as
        // part of a DECIMAL literal, so the original translation shipped 444|222 = 0o776
        // where msvcrt reports 0o666 — the execute bits set on every regular file, on all
        // three arches. Caught by wave 18b's first _fstat64 row.
        (*buf).st_mode = (_S_IFREG | 0o444) as u16;
        if (basic_info.FileAttributes & FILE_ATTRIBUTE_READONLY) == 0 {
            (*buf).st_mode |= 0o222 as u16;
        }
        (*buf).st_size = std_info.EndOfFile.QuadPart;
        RtlTimeToSecondsSince1970(&basic_info.LastAccessTime as *const _ as *mut LARGE_INTEGER, &mut dw);
        (*buf).st_atime = dw as i64;
        RtlTimeToSecondsSince1970(&basic_info.LastWriteTime as *const _ as *mut LARGE_INTEGER, &mut dw);
        (*buf).st_mtime = dw as i64;
        (*buf).st_ctime = dw as i64;
        (*buf).st_nlink = std_info.NumberOfLinks as i16;
    }
    release_ioinfo(info);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _fstati64(fd: i32, buf: *mut _stati64) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _fstat64(fd, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stati64(&buf64, buf);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _fstat(fd: i32, buf: *mut _stat) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _fstat64(fd, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stat(&buf64, buf);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fstat32(fd: i32, buf: *mut _stat32) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _fstat64(fd, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stat32(&buf64, buf);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fstat32i64(fd: i32, buf: *mut _stat32i64) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();

    let ret = _fstat64(fd, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stat32i64(&buf64, buf);
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fstat64i32(fd: i32, buf: *mut _stat64i32) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _fstat64(fd, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stat64i32(&buf64, buf);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _futime64(fd: i32, t: *mut __utimbuf64) -> i32 {
    let info: *mut ioinfo = get_ioinfo(fd);
    let mut at: FILETIME = core::mem::zeroed();
    let mut wt: FILETIME = core::mem::zeroed();

    if t.is_null() {
        time_to_filetime(_time64(core::ptr::null_mut()), &mut at);
        wt = at;
    } else {
        time_to_filetime((*t).actime, &mut at);
        time_to_filetime((*t).modtime, &mut wt);
    }

    if SetFileTime((*info).handle, core::ptr::null_mut(), &at, &wt) == 0 {
        release_ioinfo(info);
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    release_ioinfo(info);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _futime32(fd: i32, t: *mut __utimbuf32) -> i32 {
    if !t.is_null() {
        let mut t64: __utimbuf64 = core::mem::zeroed();
        t64.actime = ((*t).actime) as i64;
        t64.modtime = ((*t).modtime) as i64;
        return _futime64(fd, &mut t64);
    } else {
        return _futime64(fd, core::ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "C" fn _get_osfhandle(fd: i32) -> isize {
    let hand: HANDLE = (*get_ioinfo_nolock(fd)).handle;

    if hand == INVALID_HANDLE_VALUE {
        *_errno() = (EBADF) as i32;
    }
    hand as isize
}

#[no_mangle]
pub unsafe extern "C" fn _mktemp_s(pattern: *mut u8, size: usize) -> i32 {
    let mut len: u32;
    let mut wlen: u32;
    let mut xno: u32;
    let mut id: u32;
    let mut pathW: *mut wchar_t;

    if MSVCRT_CHECK_PMT!((pattern != core::ptr::null_mut())) == 0 {
        return (EINVAL) as i32;
    }

    len = 0;
    while len < size as u32 {
        if *pattern.offset(len as isize) == 0 {
            break;
        }
        len += 1;
    }

    if MSVCRT_CHECK_PMT!((len != size as u32 && len >= 6)) == 0 {
        if size != 0 {
            *pattern.offset(0 as isize) = 0;
        }
        return (EINVAL) as i32;
    }

    xno = 1;
    while xno <= 6 {
        if MSVCRT_CHECK_PMT!((*pattern.offset((len - xno) as isize) == b'X')) == 0 {
            return (EINVAL) as i32;
        }
        xno += 1;
    }

    id = GetCurrentProcessId();
    xno = 1;
    while xno < 6 {
        *pattern.offset((len - xno) as isize) = (id % 10 + b'0' as u32) as u8;
        id /= 10;
        xno += 1;
    }

    pathW = wstrdupa_utf8(pattern as *const i8);
    if pathW.is_null() {
        return *_errno();
    }

    wlen = (wcslen(pathW)) as u32;
    *pathW.offset((wlen - 6) as isize) = WC_LC_A;
    while *pathW.offset((wlen - 6) as isize) <= 'z' as wchar_t {
        if GetFileAttributesW(pathW) == INVALID_FILE_ATTRIBUTES {
            *pattern.offset((len - 6) as isize) = *pathW.offset((wlen - 6) as isize) as u8;
            free(pathW as *mut core::ffi::c_void);
            return 0;
        }
        *pathW.offset((wlen - 6) as isize) += 1;
    }
    free(pathW as *mut core::ffi::c_void);

    *pattern.offset(0 as isize) = 0;
    *_errno() = (EEXIST) as i32;
    return (EEXIST) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn _mktemp(pattern: *mut i8) -> *mut i8 {
    let mut pathW: *mut u16;
    let mut p: *mut u16;
    let mut numX: i32 = 0;
    let retVal: *mut i8 = pattern;
    let mut id: i32;
    let mut letter: u8 = b'a';
    let mut pattern_ptr = pattern;

    if pattern.is_null() {
        return core::ptr::null_mut();
    }

    while *pattern_ptr != 0 {
        numX = if *pattern_ptr == b'X' as i8 {
            pattern_ptr = pattern_ptr.offset(1);
            numX + 1
        } else {
            pattern_ptr = pattern_ptr.offset(1);
            0
        };
    }

    if numX < 6 {
        return core::ptr::null_mut();
    }

    pattern_ptr = pattern_ptr.offset(-1);
    id = (GetCurrentProcessId()) as i32;
    numX = 6;

    while numX > 0 {
        numX -= 1;
        let tempNum: i32 = id / 10;
        *pattern_ptr = (id - (tempNum * 10) + b'0' as i32) as i8;
        pattern_ptr = pattern_ptr.offset(-1);
        id = tempNum;
    }

    pattern_ptr = pattern_ptr.offset(1);
    pathW = wstrdupa_utf8(retVal);
    if pathW.is_null() {
        return core::ptr::null_mut();
    }

    p = pathW.offset(wcslen(pathW) as isize - 6);

    loop {
        *p = letter as u16;
        letter = letter.wrapping_add(1);
        if GetFileAttributesW(pathW) == INVALID_FILE_ATTRIBUTES {
            *pattern_ptr = *p as i8;
            free((pathW).cast());
            return retVal;
        }
        if letter > b'z' {
            break;
        }
    }

    free((pathW).cast());
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn _wmktemp_s(pattern: *mut wchar_t, size: usize) -> i32 {
    let mut len: u32;
    let mut xno: u32;
    let mut id: u32;

    if MSVCRT_CHECK_PMT!(pattern != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }

    len = 0;
    while len < size as u32 {
        if *pattern.add(len as usize) == 0 {
            break;
        }
        len += 1;
    }

    if MSVCRT_CHECK_PMT!(len != size as u32 && len >= 6) == 0 {
        if size != 0 {
            *pattern.add(0 as usize) = 0;
        }
        return (EINVAL) as i32;
    }

    xno = 1;
    while xno <= 6 {
        if MSVCRT_CHECK_PMT!(*pattern.add((len - xno) as usize) == 'X' as wchar_t) == 0 {
            return (EINVAL) as i32;
        }
        xno += 1;
    }

    id = GetCurrentProcessId();
    xno = 1;
    while xno < 6 {
        *pattern.add((len - xno) as usize) = (id % 10 + '0' as u32) as wchar_t;
        id /= 10;
        xno += 1;
    }

    *pattern.add((len - 6) as usize) = WC_LC_A;
    while *pattern.add((len - 6) as usize) <= 'z' as wchar_t {
        if GetFileAttributesW(pattern) == INVALID_FILE_ATTRIBUTES {
            return 0;
        }
        *pattern.add((len - 6) as usize) += 1;
    }

    *pattern.add(0 as usize) = 0;
    *_errno() = (EEXIST) as i32;
    return (EEXIST) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn _wmktemp(pattern: *mut wchar_t) -> *mut wchar_t {
    let mut numX: i32 = 0;
    let retVal: *mut wchar_t = pattern;
    let mut id: i32;
    let mut letter: wchar_t = WC_LC_A;
    let mut pattern = pattern;

    if pattern.is_null() {
        return core::ptr::null_mut();
    }

    while *pattern != 0 {
        numX = if *pattern == 'X' as wchar_t {
            pattern = pattern.offset(1);
            numX + 1
        } else {
            pattern = pattern.offset(1);
            0
        };
    }
    if numX < 6 {
        return core::ptr::null_mut();
    }
    pattern = pattern.offset(-1);
    id = (GetCurrentProcessId()) as i32;
    numX = 6;
    while numX != 0 {
        numX -= 1;
        let tempNum: i32 = id / 10;
        *pattern = (id - (tempNum * 10) + '0' as i32) as wchar_t;
        pattern = pattern.offset(-1);
        id = tempNum;
    }
    pattern = pattern.offset(1);
    loop {
        if GetFileAttributesW(retVal) == INVALID_FILE_ATTRIBUTES {
            return retVal;
        }
        *pattern = letter;
        letter += 1;
        if letter == '|' as wchar_t {
            break;
        }
    }
    core::ptr::null_mut()
}
unsafe extern "C" fn split_oflags(oflags: u32) -> u32 {
    let mut wxflags = 0;
    let unsupp: u32;

    if (oflags & _O_APPEND) != 0 {
        wxflags |= WX_APPEND;
    }
    if (oflags & _O_BINARY) != 0 {
        /* Nothing to do */
    } else if (oflags & _O_TEXT) != 0 {
        wxflags |= WX_TEXT;
    } else if (oflags & _O_WTEXT) != 0 {
        wxflags |= WX_TEXT;
    } else if (oflags & _O_U16TEXT) != 0 {
        wxflags |= WX_TEXT;
    } else if (oflags & _O_U8TEXT) != 0 {
        wxflags |= WX_TEXT;
    } else {
        let mut fmode: i32 = 0;
        _get_fmode(core::ptr::addr_of_mut!(fmode));
        if (fmode & (_O_BINARY) as i32) == 0 {
            wxflags |= WX_TEXT;
        }
    }
    if (oflags & _O_NOINHERIT) != 0 {
        wxflags |= WX_DONTINHERIT;
    }

    unsupp = oflags & !((_O_BINARY | _O_TEXT | _O_APPEND | _O_TRUNC | _O_EXCL | _O_CREAT |
                    _O_RDWR | _O_WRONLY | _O_TEMPORARY | _O_NOINHERIT | _O_SEQUENTIAL |
                    _O_RANDOM | _O_SHORT_LIVED | _O_WTEXT | _O_U16TEXT | _O_U8TEXT));
    if unsupp != 0 {
        ERR!(":unsupported oflags %#x\n", unsupp);
    }

    (wxflags) as u32
}

#[no_mangle]
pub unsafe extern "C" fn _pipe(pfds: *mut i32, psize: u32, textmode: i32) -> i32 {
    let mut ret: i32 = -1;
    let mut sa: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut readHandle: HANDLE = core::ptr::null_mut();
    let mut writeHandle: HANDLE = core::ptr::null_mut();

    if pfds.is_null() {
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    sa.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    sa.bInheritHandle = if (textmode & (_O_NOINHERIT) as i32) != 0 { 0 } else { 1 };
    sa.lpSecurityDescriptor = core::ptr::null_mut();

    if CreatePipe(
        core::ptr::addr_of_mut!(readHandle),
        core::ptr::addr_of_mut!(writeHandle),
        core::ptr::addr_of_mut!(sa),
        psize
    ) != 0 {
        let wxflags: u32 = split_oflags(textmode as u32);
        let mut fd: i32;

        fd = msvcrt_alloc_fd(readHandle, (wxflags | (WX_PIPE) as u32) as i32);
        if fd != -1 {
            *pfds.offset(0 as isize) = fd;
            fd = msvcrt_alloc_fd(writeHandle, (wxflags | (WX_PIPE) as u32) as i32);
            if fd != -1 {
                *pfds.offset(1 as isize) = fd;
                ret = 0;
            } else {
                _close(*pfds.offset(0 as isize));
                CloseHandle(writeHandle);
                *_errno() = (EMFILE) as i32;
            }
        } else {
            CloseHandle(readHandle);
            CloseHandle(writeHandle);
            *_errno() = (EMFILE) as i32;
        }
    } else {
        msvcrt_set_errno((GetLastError()) as i32);
    }

    ret
}
unsafe extern "C" fn check_bom(h: HANDLE, mut oflags: i32, seek: BOOL) -> i32 {
    let mut bom: [i8; 3] = [0; 3];
    let mut r: u32 = 0;

    if ReadFile(
        h,
        bom.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&utf8_bom) as u32,
        &mut r,
        core::ptr::null_mut(),
    ) == 0
    {
        return oflags;
    }

    if r == core::mem::size_of_val(&utf8_bom) as u32
        && memcmp(
            bom.as_ptr() as *const core::ffi::c_void,
            utf8_bom.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&utf8_bom),
        ) == 0
    {
        oflags = (oflags & (!(_O_WTEXT | _O_U16TEXT)) as i32) | (_O_U8TEXT) as i32;
    } else if r >= core::mem::size_of_val(&utf16_bom) as u32
        && memcmp(
            bom.as_ptr() as *const core::ffi::c_void,
            utf16_bom.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of_val(&utf16_bom),
        ) == 0
    {
        if seek != 0 && r > 2 {
            SetFilePointer(h, 2, core::ptr::null_mut(), FILE_BEGIN);
        }
        oflags = (oflags & (!(_O_WTEXT | _O_U8TEXT)) as i32) | (_O_U16TEXT) as i32;
    } else if seek != 0 {
        SetFilePointer(h, 0, core::ptr::null_mut(), FILE_BEGIN);
    }

    oflags
}
#[no_mangle]
pub unsafe extern "C" fn _wsopen_dispatch(
    path: *const wchar_t,
    oflags: i32,
    shflags: i32,
    pmode: i32,
    fd: *mut i32,
    secure: i32,
) -> i32 {
    let mut access = 0;
    let mut creation = 0;
    let attrib: DWORD;
    let mut sa: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut sharing: DWORD;
    let type_: DWORD;
    let mut wxflag: i32;
    let mut hand: HANDLE;

    if MSVCRT_CHECK_PMT!(fd != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    *fd = -1;
    if MSVCRT_CHECK_PMT!(path != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }

    wxflag = split_oflags(oflags as u32) as i32;
    match (oflags as u32) & (_O_RDONLY | _O_WRONLY | _O_RDWR) {
        _O_RDONLY => access |= GENERIC_READ,
        _O_WRONLY => access |= GENERIC_WRITE,
        _O_RDWR => access |= GENERIC_WRITE | GENERIC_READ,
        _ => {}
    }

    if (oflags & (_O_CREAT) as i32) != 0 {
        if secure != 0 && MSVCRT_CHECK_PMT!((pmode as u32 & !(_S_IREAD | _S_IWRITE)) == 0) == 0 {
            return (EINVAL) as i32;
        }

        if (oflags & (_O_EXCL) as i32) != 0 {
            creation = CREATE_NEW;
        } else if (oflags & (_O_TRUNC) as i32) != 0 {
            creation = CREATE_ALWAYS;
        } else {
            creation = OPEN_ALWAYS;
        }
    } else {
        if (oflags & (_O_TRUNC) as i32) != 0 {
            creation = TRUNCATE_EXISTING;
        } else {
            creation = OPEN_EXISTING;
        }
    }

    match shflags as u32 {
        _SH_DENYRW => {
            sharing = 0;
        }
        _SH_DENYWR => {
            sharing = FILE_SHARE_READ;
        }
        _SH_DENYRD => {
            sharing = FILE_SHARE_WRITE;
        }
        _SH_DENYNO => {
            sharing = FILE_SHARE_READ | FILE_SHARE_WRITE;
        }
        _ => {
            return (EINVAL) as i32;
        }
    }

    if (pmode & !MSVCRT_umask & (_S_IWRITE) as i32) == 0 {
        attrib = FILE_ATTRIBUTE_READONLY;
    } else {
        attrib = FILE_ATTRIBUTE_NORMAL;
    }

    let mut attrib = attrib;
    if (oflags & (_O_TEMPORARY) as i32) != 0 {
        attrib |= FILE_FLAG_DELETE_ON_CLOSE;
        access |= DELETE;
        sharing |= FILE_SHARE_DELETE;
    }

    if (oflags & (_O_RANDOM) as i32) != 0 {
        attrib |= FILE_FLAG_RANDOM_ACCESS;
    }
    if (oflags & (_O_SEQUENTIAL) as i32) != 0 {
        attrib |= FILE_FLAG_SEQUENTIAL_SCAN;
    }
    if (oflags & (_O_SHORT_LIVED) as i32) != 0 {
        attrib |= FILE_ATTRIBUTE_TEMPORARY;
    }

    sa.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
    sa.lpSecurityDescriptor = core::ptr::null_mut();
    sa.bInheritHandle = if (oflags & (_O_NOINHERIT) as i32) != 0 { 0 } else { 1 };

    let mut oflags = oflags;
    if ((oflags & ((_O_WTEXT | _O_U16TEXT | _O_U8TEXT)) as i32) != 0)
        && (creation == OPEN_ALWAYS || creation == OPEN_EXISTING)
        && ((access & GENERIC_READ) == 0)
    {
        hand = CreateFileW(
            path,
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &mut sa,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );
        if hand != INVALID_HANDLE_VALUE {
            oflags = check_bom(hand, oflags, (FALSE) as i32);
            CloseHandle(hand);
        }
    }

    hand = CreateFileW(path, access, sharing, &mut sa, creation, attrib, core::ptr::null_mut());
    if hand == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return *_errno();
    }

    if (oflags & ((_O_WTEXT | _O_U16TEXT | _O_U8TEXT)) as i32) != 0 {
        let mut size = LARGE_INTEGER {
            QuadPart: 0,
        };

        if ((access & GENERIC_WRITE) != 0)
            && (creation == OPEN_EXISTING || creation == OPEN_ALWAYS)
        {
            GetFileSizeEx(hand, &mut size);
        }

        if ((access & GENERIC_WRITE) != 0)
            && (creation == CREATE_NEW
                || creation == CREATE_ALWAYS
                || creation == TRUNCATE_EXISTING
                || ((creation == OPEN_EXISTING || creation == OPEN_ALWAYS)
                    && size.QuadPart == 0))
        {
            if (oflags & (_O_U8TEXT) as i32) != 0 {
                let mut written = 0;
                let mut tmp: DWORD = 0;

                while written != core::mem::size_of_val(&utf8_bom) as u32
                    && WriteFile(
                        hand,
                        (utf8_bom.as_ptr() as *const u8).add(written as usize) as *const _,
                        (core::mem::size_of_val(&utf8_bom) as u32 - written),
                        &mut tmp,
                        core::ptr::null_mut(),
                    ) != 0
                {
                    written += tmp;
                }
                if written != core::mem::size_of_val(&utf8_bom) as u32 {
                    CloseHandle(hand);
                    msvcrt_set_errno((GetLastError()) as i32);
                    return *_errno();
                }
            } else {
                let mut written = 0;
                let mut tmp: DWORD = 0;

                while written != core::mem::size_of_val(&utf16_bom) as u32
                    && WriteFile(
                        hand,
                        (utf16_bom.as_ptr() as *const u8).add(written as usize) as *const _,
                        (core::mem::size_of_val(&utf16_bom) as u32 - written),
                        &mut tmp,
                        core::ptr::null_mut(),
                    ) != 0
                {
                    written += tmp;
                }
                if written != core::mem::size_of_val(&utf16_bom) as u32 {
                    CloseHandle(hand);
                    msvcrt_set_errno((GetLastError()) as i32);
                    return *_errno();
                }
                oflags |= (_O_U16TEXT) as i32;
            }
        } else if (access & GENERIC_READ) != 0 {
            oflags = check_bom(hand, oflags, (TRUE) as i32);
        }
    }

    type_ = GetFileType(hand);
    if type_ == FILE_TYPE_CHAR {
        wxflag |= (WX_TTY) as i32;
    } else if type_ == FILE_TYPE_PIPE {
        wxflag |= (WX_PIPE) as i32;
    }

    *fd = msvcrt_alloc_fd(hand, wxflag);
    if *fd == -1 {
        return *_errno();
    }

    if (oflags & (_O_WTEXT) as i32) != 0 {
        ioinfo_set_unicode(get_ioinfo_nolock(*fd), (TRUE) as i32);
    }

    if (oflags & (_O_U16TEXT) as i32) != 0 {
        ioinfo_set_textmode(get_ioinfo_nolock(*fd), TEXTMODE_UTF16LE);
    } else if (oflags & (_O_U8TEXT) as i32) != 0 {
        ioinfo_set_textmode(get_ioinfo_nolock(*fd), TEXTMODE_UTF8);
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _wsopen_s(
    fd: *mut i32,
    path: *const wchar_t,
    oflags: i32,
    shflags: i32,
    pmode: i32,
) -> i32 {
    _wsopen_dispatch(path, oflags, shflags, pmode, fd, 1)
}

#[no_mangle]
pub unsafe extern "C" fn _sopen_dispatch(
    path: *const i8,
    oflags: i32,
    shflags: i32,
    pmode: i32,
    fd: *mut i32,
    secure: i32,
) -> i32 {
    let mut pathW: *mut u16 = core::ptr::null_mut();
    let ret: i32;

    if MSVCRT_CHECK_PMT!(fd != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    *fd = -1;
    if path != core::ptr::null() && {
        pathW = wstrdupa_utf8(path);
        pathW == core::ptr::null_mut()
    } {
        return *_errno();
    }

    ret = _wsopen_dispatch(pathW, oflags, shflags, pmode, fd, secure);
    free((pathW).cast());
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _sopen_s(
    fd: *mut i32,
    path: *const i8,
    oflags: i32,
    shflags: i32,
    pmode: i32,
) -> i32 {
    _sopen_dispatch(path, oflags, shflags, pmode, fd, 1)
}




#[no_mangle]
pub unsafe extern "C" fn _creat(path: *const i8, pmode: i32) -> i32 {
    let flags = _O_CREAT | _O_TRUNC | _O_RDWR;
    _open(path, (flags) as i32, pmode)
}

#[no_mangle]
pub unsafe extern "C" fn _wcreat(path: *const wchar_t, pmode: i32) -> i32 {
    let flags = _O_CREAT | _O_TRUNC | _O_RDWR;
    _wopen(path, (flags) as i32, pmode)
}

#[no_mangle]
pub unsafe extern "C" fn _open_osfhandle(handle: isize, mut oflags: i32) -> i32 {
    let mut flags: u32;
    let fd: i32;

    if (oflags & ((_O_BINARY | _O_TEXT)) as i32) == 0 {
        oflags |= (_O_BINARY) as i32;
    }

    flags = GetFileType(handle as HANDLE);
    if flags == FILE_TYPE_UNKNOWN && GetLastError() != NO_ERROR {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }

    if flags == FILE_TYPE_CHAR {
        flags = (WX_TTY) as u32;
    } else if flags == FILE_TYPE_PIPE {
        flags = (WX_PIPE) as u32;
    } else {
        flags = 0;
    }
    flags |= split_oflags(oflags as u32);

    fd = msvcrt_alloc_fd(handle as HANDLE, flags as i32);

    fd
}

#[no_mangle]
pub unsafe extern "C" fn _rmtmp() -> i32 {
    let mut num_removed: i32 = 0;
    let mut i: i32;
    let mut file: *mut FILE;

    LOCK_FILES!();
    i = 3;
    while i < MSVCRT_stream_idx {
        file = msvcrt_get_file(i);

        if !(*file)._tmpfname.is_null() {
            fclose(file);
            num_removed += 1;
        }
        i += 1;
    }
    UNLOCK_FILES!();

    if num_removed != 0 {
        TRACE!(b":removed (%d) temp files\n\0".as_ptr() as PCHAR, num_removed);
    }
    num_removed
}
unsafe extern "C" fn get_utf8_char_len(ch: i8) -> i32 {
    if (ch as i32 & 0xf8) == 0xf0 {
        return 4;
    } else if (ch as i32 & 0xf0) == 0xe0 {
        return 3;
    } else if (ch as i32 & 0xe0) == 0xc0 {
        return 2;
    }
    return 1;
}
unsafe extern "C" fn read_utf8(fdinfo: *mut ioinfo, buf: *mut wchar_t, count: u32) -> i32 {
    let hand: HANDLE = (*fdinfo).handle;
    let mut min_buf: [i8; 4] = [0; 4];
    let mut readbuf: *mut i8;
    let mut lookahead: i8 = 0;
    let mut readbuf_size: u32;
    let mut pos: u32 = 0;
    let mut num_read: u32 = 1;
    let mut char_len: u32;
    let mut i: u32;
    let mut j: u32;
    let mut count = count;

    count /= 2;
    readbuf_size = if count < 4 {
        4
    } else {
        count
    };
    if readbuf_size <= 4 {
        readbuf_size = 4;
        readbuf = min_buf.as_mut_ptr();
    } else {
        readbuf = malloc(readbuf_size as usize) as *mut i8;
        if readbuf.is_null() {
            readbuf_size = 4;
            readbuf = min_buf.as_mut_ptr();
        }
    }

    if (*fdinfo).lookahead[0] != (b'\n' as i8) as u8 {
        *readbuf.add(pos as usize) = ((*fdinfo).lookahead[0]) as i8;
        pos += 1;
        (*fdinfo).lookahead[0] = (b'\n' as i8) as u8;

        if (*fdinfo).lookahead[1] != (b'\n' as i8) as u8 {
            *readbuf.add(pos as usize) = ((*fdinfo).lookahead[1]) as i8;
            pos += 1;
            (*fdinfo).lookahead[1] = (b'\n' as i8) as u8;

            if (*fdinfo).lookahead[2] != (b'\n' as i8) as u8 {
                *readbuf.add(pos as usize) = ((*fdinfo).lookahead[2]) as i8;
                pos += 1;
                (*fdinfo).lookahead[2] = (b'\n' as i8) as u8;
            }
        }
    }

    if count < 4 {
        if pos == 0 && ReadFile(hand, readbuf as *mut core::ffi::c_void, 1, &mut num_read, core::ptr::null_mut()) == 0 {
            if GetLastError() == ERROR_BROKEN_PIPE {
                (*fdinfo).wxflag |= WX_ATEOF;
                return 0;
            } else {
                msvcrt_set_errno((GetLastError()) as i32);
                if GetLastError() == ERROR_ACCESS_DENIED {
                    *_errno() = (EBADF) as i32;
                }
                return -1;
            }
        } else if num_read == 0 {
            (*fdinfo).wxflag |= WX_ATEOF;
            return 0;
        } else {
            pos += 1;
        }

        char_len = get_utf8_char_len(*readbuf.add(0)) as u32;
        if char_len > pos {
            if ReadFile(hand, readbuf.add(pos as usize) as *mut core::ffi::c_void, char_len - pos, &mut num_read, core::ptr::null_mut()) != 0 {
                pos += num_read;
            }
        }

        if *readbuf.add(0) == b'\n' as i8 {
            (*fdinfo).wxflag |= WX_READNL;
        } else {
            (*fdinfo).wxflag &= !WX_READNL;
        }

        if *readbuf.add(0) == 0x1a {
            (*fdinfo).wxflag |= WX_ATEOF;
            return 0;
        }

        if *readbuf.add(0) == b'\r' as i8 {
            if ReadFile(hand, &mut lookahead as *mut i8 as *mut core::ffi::c_void, 1, &mut num_read, core::ptr::null_mut()) == 0 || num_read != 1 {
                *buf.add(0) = b'\r' as wchar_t;
            } else if lookahead == b'\n' as i8 {
                *buf.add(0) = b'\n' as wchar_t;
            } else {
                *buf.add(0) = b'\r' as wchar_t;
                if ((*fdinfo).wxflag & (WX_PIPE | WX_TTY)) != 0 {
                    (*fdinfo).lookahead[0] = (lookahead) as u8;
                } else {
                    SetFilePointer((*fdinfo).handle, -1, core::ptr::null_mut(), FILE_CURRENT);
                }
            }
            return 2;
        }

        num_read = (MultiByteToWideChar(CP_UTF8, 0, readbuf, pos as i32, buf, count as i32)) as u32;
        if num_read == 0 {
            msvcrt_set_errno((GetLastError()) as i32);
            return -1;
        }

        return (num_read * 2) as i32;
    }

    if ReadFile(hand, readbuf.add(pos as usize) as *mut core::ffi::c_void, readbuf_size - pos, &mut num_read, core::ptr::null_mut()) == 0 {
        if pos != 0 {
            num_read = 0;
        } else if GetLastError() == ERROR_BROKEN_PIPE {
            (*fdinfo).wxflag |= WX_ATEOF;
            if readbuf != min_buf.as_mut_ptr() {
                free(readbuf as *mut core::ffi::c_void);
            }
            return 0;
        } else {
            msvcrt_set_errno((GetLastError()) as i32);
            if GetLastError() == ERROR_ACCESS_DENIED {
                *_errno() = (EBADF) as i32;
            }
            if readbuf != min_buf.as_mut_ptr() {
                free(readbuf as *mut core::ffi::c_void);
            }
            return -1;
        }
    } else if pos == 0 && num_read == 0 {
        (*fdinfo).wxflag |= WX_ATEOF;
        if readbuf != min_buf.as_mut_ptr() {
            free(readbuf as *mut core::ffi::c_void);
        }
        return 0;
    }

    pos += num_read;
    if *readbuf.add(0) == b'\n' as i8 {
        (*fdinfo).wxflag |= WX_READNL;
    } else {
        (*fdinfo).wxflag &= !WX_READNL;
    }

    i = pos - 1;
    while i > 0 && i > pos - 4 {
        if (*readbuf.add(i as usize) as i32 & 0xc0) != 0x80 {
            break;
        }
        i -= 1;
    }
    char_len = get_utf8_char_len(*readbuf.add(i as usize)) as u32;
    if char_len + i <= pos {
        i += char_len;
    }

    if ((*fdinfo).wxflag & (WX_PIPE | WX_TTY)) != 0 {
        if i < pos {
            (*fdinfo).lookahead[0] = (*readbuf.add(i as usize)) as u8;
        }
        if i + 1 < pos {
            (*fdinfo).lookahead[1] = (*readbuf.add((i + 1) as usize)) as u8;
        }
        if i + 2 < pos {
            (*fdinfo).lookahead[2] = (*readbuf.add((i + 2) as usize)) as u8;
        }
    } else if i < pos {
        SetFilePointer((*fdinfo).handle, (i as i32) - (pos as i32), core::ptr::null_mut(), FILE_CURRENT);
    }
    pos = i;

    i = 0;
    j = 0;
    while i < pos {
        if *readbuf.add(i as usize) == 0x1a {
            (*fdinfo).wxflag |= WX_ATEOF;
            break;
        }

        if *readbuf.add(i as usize) == b'\r' as i8 && i + 1 == pos {
            if (*fdinfo).lookahead[0] != (b'\n' as i8) as u8 || ReadFile(hand, &mut lookahead as *mut i8 as *mut core::ffi::c_void, 1, &mut num_read, core::ptr::null_mut()) == 0 || num_read == 0 {
                *readbuf.add(j as usize) = b'\r' as i8;
                j += 1;
            } else if lookahead == b'\n' as i8 && j == 0 {
                *readbuf.add(j as usize) = b'\n' as i8;
                j += 1;
            } else {
                if lookahead != b'\n' as i8 {
                    *readbuf.add(j as usize) = b'\r' as i8;
                    j += 1;
                }

                if ((*fdinfo).wxflag & (WX_PIPE | WX_TTY)) != 0 {
                    (*fdinfo).lookahead[0] = (lookahead) as u8;
                } else {
                    SetFilePointer((*fdinfo).handle, -1, core::ptr::null_mut(), FILE_CURRENT);
                }
            }
        } else if *readbuf.add(i as usize) != b'\r' as i8 || *readbuf.add((i + 1) as usize) != b'\n' as i8 {
            *readbuf.add(j as usize) = *readbuf.add(i as usize);
            j += 1;
        }
        i += 1;
    }
    pos = j;

    num_read = (MultiByteToWideChar(CP_UTF8, 0, readbuf, pos as i32, buf, count as i32)) as u32;
    if num_read == 0 {
        msvcrt_set_errno((GetLastError()) as i32);
        if readbuf != min_buf.as_mut_ptr() {
            free(readbuf as *mut core::ffi::c_void);
        }
        return -1;
    }

    if readbuf != min_buf.as_mut_ptr() {
        free(readbuf as *mut core::ffi::c_void);
    }
    return (num_read * 2) as i32;
}
unsafe extern "C" fn read_i(fd: i32, fdinfo: *mut ioinfo, buf: *mut core::ffi::c_void, count: u32) -> i32 {
    let mut num_read: u32 = 0;
    let mut utf16: u32;
    let bufstart = buf as *mut u8;

    if count == 0 {
        return 0;
    }

    if (*fdinfo).wxflag & WX_ATEOF != 0 {
        return 0;
    }

    if count > 4 {
        TRACE!(
            b":fd (%d) handle (%p) buf (%p) len (%d)\n\0".as_ptr() as *const i8,
            fd,
            (*fdinfo).handle,
            buf,
            count
        );
    }

    if (*fdinfo).handle == INVALID_HANDLE_VALUE {
        *_errno() = (EBADF) as i32;
        return -1;
    }

    utf16 = if ioinfo_get_textmode(fdinfo) == TEXTMODE_UTF16LE { 1 } else { 0 };
    if ioinfo_get_textmode(fdinfo) != TEXTMODE_ANSI && count & 1 != 0 {
        *_errno() = (EINVAL) as i32;
        return -1;
    }

    if (*fdinfo).wxflag & WX_TEXT != 0 && ioinfo_get_textmode(fdinfo) == TEXTMODE_UTF8 {
        return read_utf8(fdinfo, buf as *mut wchar_t, count);
    }

    if (*fdinfo).lookahead[0 as usize] != b'\n' || ReadFile((*fdinfo).handle, bufstart as *mut core::ffi::c_void, count, &mut num_read, core::ptr::null_mut()) != 0 {
        if (*fdinfo).lookahead[0 as usize] != b'\n' {
            *bufstart.add(0 as usize) = (*fdinfo).lookahead[0 as usize];
            (*fdinfo).lookahead[0 as usize] = b'\n';

            if utf16 != 0 {
                *bufstart.add(1 as usize) = (*fdinfo).lookahead[1 as usize];
                (*fdinfo).lookahead[1 as usize] = b'\n';
            }

            if count > 1 + utf16 && ReadFile((*fdinfo).handle, bufstart.add((1 + utf16) as usize) as *mut core::ffi::c_void, count - 1 - utf16, &mut num_read, core::ptr::null_mut()) != 0 {
                num_read += 1 + utf16;
            } else {
                num_read = 1 + utf16;
            }
        }

        if utf16 != 0 && num_read & 1 != 0 {
            num_read -= 1;
        }

        if count != 0 && num_read == 0 {
            (*fdinfo).wxflag |= WX_ATEOF;
        } else if (*fdinfo).wxflag & WX_TEXT != 0 {
            let mut i: u32;
            let mut j: u32;

            if *bufstart.add(0 as usize) == b'\n' && (utf16 == 0 || *bufstart.add(1 as usize) == 0) {
                (*fdinfo).wxflag |= WX_READNL;
            } else {
                (*fdinfo).wxflag &= !WX_READNL;
            }

            i = 0;
            j = 0;
            while i < num_read {
                if *bufstart.add(i as usize) == 0x1a && (utf16 == 0 || *bufstart.add((i + 1) as usize) == 0) {
                    (*fdinfo).wxflag |= WX_ATEOF;
                    break;
                }

                if *bufstart.add(i as usize) == b'\r' && (utf16 == 0 || *bufstart.add((i + 1) as usize) == 0) && i + 1 + utf16 == num_read {
                    let mut lookahead: [u8; 2 as usize] = [0; 2 as usize];
                    let mut len: u32 = 0;

                    lookahead[1 as usize] = b'\n';
                    if ReadFile((*fdinfo).handle, lookahead.as_mut_ptr() as *mut core::ffi::c_void, 1 + utf16, &mut len, core::ptr::null_mut()) != 0 && len != 0 {
                        if lookahead[0 as usize] == b'\n' && (utf16 == 0 || lookahead[1 as usize] == 0) && j == 0 {
                            *bufstart.add(j as usize) = b'\n';
                            j += 1;
                            if utf16 != 0 {
                                *bufstart.add(j as usize) = 0;
                                j += 1;
                            }
                        } else {
                            if lookahead[0 as usize] != b'\n' || (utf16 != 0 && lookahead[1 as usize] != 0) {
                                *bufstart.add(j as usize) = b'\r';
                                j += 1;
                                if utf16 != 0 {
                                    *bufstart.add(j as usize) = 0;
                                    j += 1;
                                }
                            }

                            if (*fdinfo).wxflag & (WX_PIPE | WX_TTY) != 0 {
                                if lookahead[0 as usize] == b'\n' && (utf16 == 0 || lookahead[1 as usize] == 0) {
                                    *bufstart.add(j as usize) = b'\n';
                                    j += 1;
                                    if utf16 != 0 {
                                        *bufstart.add(j as usize) = 0;
                                        j += 1;
                                    }
                                } else {
                                    (*fdinfo).lookahead[0 as usize] = lookahead[0 as usize];
                                    (*fdinfo).lookahead[1 as usize] = lookahead[1 as usize];
                                }
                            } else {
                                SetFilePointer((*fdinfo).handle, -(1 as i32) - utf16 as i32, core::ptr::null_mut(), FILE_CURRENT);
                            }
                        }
                    } else {
                        *bufstart.add(j as usize) = b'\r';
                        j += 1;
                        if utf16 != 0 {
                            *bufstart.add(j as usize) = 0;
                            j += 1;
                        }
                    }
                } else if (*bufstart.add(i as usize) != b'\r' || (utf16 != 0 && *bufstart.add((i + 1) as usize) != 0)) || (*bufstart.add((i + 1 + utf16) as usize) != b'\n' || (utf16 != 0 && *bufstart.add((i + 3) as usize) != 0)) {
                    *bufstart.add(j as usize) = *bufstart.add(i as usize);
                    j += 1;
                    if utf16 != 0 {
                        *bufstart.add(j as usize) = *bufstart.add((i + 1) as usize);
                        j += 1;
                    }
                }

                i += 1 + utf16;
            }
            num_read = j;
        }
    } else {
        if GetLastError() == ERROR_BROKEN_PIPE {
            (*fdinfo).wxflag |= WX_ATEOF;
            return 0;
        } else {
            msvcrt_set_errno((GetLastError()) as i32);
            if GetLastError() == ERROR_ACCESS_DENIED {
                *_errno() = (EBADF) as i32;
            }
            return -1;
        }
    }

    if count > 4 {
        TRACE!(
            b"(%lu), %s\n\0".as_ptr() as *const i8,
            num_read,
            debugstr_an(buf as *const i8, num_read as i32)
        );
    }
    num_read as i32
}

#[no_mangle]
pub unsafe extern "C" fn _read(fd: i32, buf: *mut core::ffi::c_void, count: u32) -> i32 {
    let info: *mut ioinfo;
    let num_read: i32;

    if fd == MSVCRT_NO_CONSOLE_FD {
        *_errno() = (EBADF) as i32;
        return -1;
    }

    info = get_ioinfo(fd);
    num_read = read_i(fd, info, buf, count);
    release_ioinfo(info);
    return num_read;
}

#[no_mangle]
pub unsafe extern "C" fn _setmode(fd: i32, mode: i32) -> i32 {
    let info = get_ioinfo(fd);
    let mut ret = if (*info).wxflag & WX_TEXT != 0 {
        _O_TEXT
    } else {
        _O_BINARY
    };

    if ret == _O_TEXT && ioinfo_get_textmode(info) != TEXTMODE_ANSI {
        ret = _O_WTEXT;
    }

    if mode != (_O_TEXT) as i32 && mode != (_O_BINARY) as i32 && mode != (_O_WTEXT) as i32
        && mode != (_O_U16TEXT) as i32 && mode != (_O_U8TEXT) as i32 {
        *_errno() = (EINVAL) as i32;
        release_ioinfo(info);
        return -1;
    }

    if info == core::ptr::addr_of_mut!(MSVCRT___badioinfo) {
        *_errno() = (EBADF) as i32;
        return EOF;
    }

    if mode == (_O_BINARY) as i32 {
        (*info).wxflag &= !WX_TEXT;
        ioinfo_set_textmode(info, TEXTMODE_ANSI);
        release_ioinfo(info);
        return (ret) as i32;
    }

    (*info).wxflag |= WX_TEXT;
    if mode == (_O_TEXT) as i32 {
        ioinfo_set_textmode(info, TEXTMODE_ANSI);
    } else if mode == (_O_U8TEXT) as i32 {
        ioinfo_set_textmode(info, TEXTMODE_UTF8);
    } else {
        ioinfo_set_textmode(info, TEXTMODE_UTF16LE);
    }

    release_ioinfo(info);
    (ret) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _stat64(path: *const i8, buf: *mut _stat64) -> i32 {
    let mut pathW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !path.is_null() {
        pathW = wstrdupa_utf8(path);
        if pathW.is_null() {
            return -1;
        }
    }
    ret = _wstat64(pathW, buf);
    free((pathW).cast());
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _stati64(path: *const i8, buf: *mut _stati64) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _stat64(path, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stati64(&buf64, buf);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _stat(path: *const i8, buf: *mut _stat) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _stat64(path, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stat(&buf64, buf);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wstat64(path: *const wchar_t, buf: *mut _stat64) -> i32 {
    let mut dw: u32 = 0;
    let mut hfi: WIN32_FILE_ATTRIBUTE_DATA = core::mem::zeroed();
    let mut mode: u16 = (ALL_S_IREAD) as u16;
    let mut plen: i32;

    plen = (wcslen(path)) as i32;
    while plen != 0 && *path.offset((plen - 1) as isize) == WC_SPACE {
        plen -= 1;
    }

    if plen == 2 && *path.offset(1 as isize) == ':' as wchar_t {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    if plen >= 2
        && *path.offset((plen - 2) as isize) != ':' as wchar_t
        && (*path.offset((plen - 1) as isize) == '\\' as wchar_t
            || *path.offset((plen - 1) as isize) == '/' as wchar_t) {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    if GetFileAttributesExW(path, GetFileExInfoStandard, core::ptr::addr_of_mut!(hfi) as *mut c_void) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    memset(buf as *mut core::ffi::c_void, 0, core::mem::size_of::<_stat64>());

    if iswalpha(*path as wint_t) != 0 && *path.offset(1 as isize) == ':' as wchar_t {
        (*buf).st_dev = towupper(*path as wint_t) as u32 - 'A' as u32;
        (*buf).st_rdev = (*buf).st_dev;
    } else {
        (*buf).st_dev = _getdrive() as u32 - 1;
        (*buf).st_rdev = (*buf).st_dev;
    }

    if hfi.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
        mode |= (_S_IFDIR | ALL_S_IEXEC) as u16;
    } else {
        mode |= (_S_IFREG) as u16;
        if plen > 6 && *path.offset((plen - 4) as isize) == '.' as wchar_t {
            let ext: u64 = towlower(*path.offset((plen - 1) as isize) as wint_t) as u64
                | ((towlower(*path.offset((plen - 2) as isize) as wint_t) as u64) << 16)
                | ((towlower(*path.offset((plen - 3) as isize) as wint_t) as u64) << 32);
            if ext == WCEXE || ext == WCBAT || ext == WCCMD || ext == WCCOM {
                mode |= (ALL_S_IEXEC) as u16;
            }
        }
    }

    if hfi.dwFileAttributes & FILE_ATTRIBUTE_READONLY == 0 {
        mode |= (ALL_S_IWRITE) as u16;
    }

    (*buf).st_mode = mode;
    (*buf).st_nlink = 1;
    (*buf).st_size = ((hfi.nFileSizeHigh as i64) << 32) + hfi.nFileSizeLow as i64;
    RtlTimeToSecondsSince1970(core::ptr::addr_of_mut!(hfi.ftLastAccessTime) as *mut LARGE_INTEGER, &mut dw);
    (*buf).st_atime = dw as i64;
    RtlTimeToSecondsSince1970(core::ptr::addr_of_mut!(hfi.ftLastWriteTime) as *mut LARGE_INTEGER, &mut dw);
    (*buf).st_mtime = dw as i64;
    (*buf).st_ctime = dw as i64;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _wstati64(path: *const wchar_t, buf: *mut _stati64) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _wstat64(path, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stati64(&buf64, buf);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wstat(path: *const wchar_t, buf: *mut _stat) -> i32 {
    let mut buf64: _stat64 = core::mem::zeroed();
    let ret = _wstat64(path, &mut buf64);
    if ret == 0 {
        msvcrt_stat64_to_stat(&buf64, buf);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _tell(fd: i32) -> __msvcrt_long {
    _lseek(fd, 0, (SEEK_CUR) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn _telli64(fd: i32) -> __int64 {
    _lseeki64(fd, 0, (SEEK_CUR) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn _tempnam(dir: *const i8, prefix: *const i8) -> *mut i8 {
    let mut dirW: *mut wchar_t = core::ptr::null_mut();
    let mut prefixW: *mut wchar_t = core::ptr::null_mut();
    let retW: *mut wchar_t;
    let ret: *mut i8;

    if !dir.is_null() {
        dirW = wstrdupa_utf8(dir);
        if dirW.is_null() {
            return core::ptr::null_mut();
        }
    }
    if !prefix.is_null() {
        prefixW = wstrdupa_utf8(prefix);
        if prefixW.is_null() {
            free((dirW).cast());
            return core::ptr::null_mut();
        }
    }
    retW = _wtempnam(dirW, prefixW);
    free((dirW).cast());
    free((prefixW).cast());
    ret = astrdupw_utf8(retW);
    free((retW).cast());
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn _wtempnam(dir: *const wchar_t, prefix: *const wchar_t) -> *mut wchar_t {
    let mut tmpbuf: [wchar_t; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let tmp_dir = _wgetenv(W_TMP.as_ptr());
    let dir = if !tmp_dir.is_null() { tmp_dir } else { dir };

    if GetTempFileNameW(dir, prefix, 0, tmpbuf.as_mut_ptr()) != 0 {
        DeleteFileW(tmpbuf.as_ptr());
        return _wcsdup(tmpbuf.as_ptr());
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn _umask(umask: i32) -> i32 {
    let old_umask = MSVCRT_umask;
    MSVCRT_umask = umask;
    old_umask
}

#[no_mangle]
pub unsafe extern "C" fn _utime64(path: *const i8, t: *mut __utimbuf64) -> i32 {
    let fd = _open(path, (_O_WRONLY | _O_BINARY) as i32);

    if fd > 0 {
        let retVal = _futime64(fd, t);
        _close(fd);
        return retVal;
    }
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _utime32(path: *const i8, t: *mut __utimbuf32) -> i32 {
    if !t.is_null() {
        let mut t64: __utimbuf64 = core::mem::zeroed();
        t64.actime = ((*t).actime) as i64;
        t64.modtime = ((*t).modtime) as i64;
        return _utime64(path, &mut t64);
    } else {
        return _utime64(path, core::ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "C" fn _wutime64(path: *const wchar_t, t: *mut __utimbuf64) -> i32 {
    let fd = _wopen(path, (_O_WRONLY | _O_BINARY) as i32);

    if fd > 0 {
        let retVal = _futime64(fd, t);
        _close(fd);
        return retVal;
    }
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn _wutime32(path: *const wchar_t, t: *mut __utimbuf32) -> i32 {
    if !t.is_null() {
        let mut t64: __utimbuf64 = core::mem::zeroed();
        t64.actime = ((*t).actime) as i64;
        t64.modtime = ((*t).modtime) as i64;
        _wutime64(path, &mut t64)
    } else {
        _wutime64(path, core::ptr::null_mut())
    }
}

#[no_mangle]
pub unsafe extern "C" fn _write(fd: i32, buf: *const core::ffi::c_void, count: u32) -> i32 {
    let info = get_ioinfo(fd);
    let hand = (*info).handle;
    let mut num_written: u32 = 0;
    let mut i: u32;
    let mut console: i32 = (FALSE) as i32;

    if hand == INVALID_HANDLE_VALUE || fd == MSVCRT_NO_CONSOLE_FD {
        *_errno() = (EBADF) as i32;
        release_ioinfo(info);
        return -1;
    }

    if ioinfo_get_textmode(info) != TEXTMODE_ANSI && (count & 1) != 0 {
        *_errno() = (EINVAL) as i32;
        release_ioinfo(info);
        return -1;
    }

    if ((*info).wxflag & WX_APPEND) != 0 {
        _lseek(fd, 0, (FILE_END) as i32);
    }

    if ((*info).wxflag & WX_TEXT) == 0 {
        if WriteFile(hand, buf, count, &mut num_written, core::ptr::null_mut()) == 0
            || num_written != count
        {
            msvcrt_set_errno((GetLastError()) as i32);
            if GetLastError() == ERROR_ACCESS_DENIED {
                *_errno() = (EBADF) as i32;
            }
            num_written = (-1i32) as u32;
        }

        release_ioinfo(info);
        return num_written as i32;
    }

    if _isatty(fd) != 0 {
        console = VerifyConsoleIoHandle(hand);
    }

    i = 0;
    while i < count {
        let s = buf as *const u8;
        let mut lfbuf: [u8; 2048] = [0; 2048];
        let mut j: u32 = 0;

        if ioinfo_get_textmode(info) == TEXTMODE_ANSI && console != 0 {
            let mut conv: [u8; 2048] = [0; 2048];
            let mut len: usize = 0;

            while i < count
                && j < (core::mem::size_of_val(&conv) - 1) as u32
                && len < (core::mem::size_of_val(&lfbuf) - 1) / core::mem::size_of::<u16>()
            {
                if isleadbyte((*s.add(i as usize) as u32) as i32) != 0 {
                    *conv.as_mut_ptr().add(j as usize) = *s.add(i as usize);
                    j += 1;
                    i += 1;

                    if i == count {
                        *_errno() = (EINVAL) as i32;
                        release_ioinfo(info);
                        return -1;
                    }
                } else if *s.add(i as usize) == b'\n' {
                    *conv.as_mut_ptr().add(j as usize) = b'\r';
                    j += 1;
                    len += 1;
                }
                *conv.as_mut_ptr().add(j as usize) = *s.add(i as usize);
                i += 1;
                j += 1;
                len += 1;
            }

            len = mbstowcs(lfbuf.as_mut_ptr() as *mut u16, conv.as_ptr() as *const i8, len);
            if len == (-1i32) as usize {
                msvcrt_set_errno((GetLastError()) as i32);
                release_ioinfo(info);
                return -1;
            }
            j = (len * 2) as u32;
        } else if ioinfo_get_textmode(info) == TEXTMODE_ANSI {
            j = 0;
            while i < count && j < (core::mem::size_of_val(&lfbuf) - 1) as u32 {
                if *s.add(i as usize) == b'\n' {
                    lfbuf[j as usize] = b'\r';
                    j += 1;
                }
                lfbuf[j as usize] = *s.add(i as usize);
                i += 1;
                j += 1;
            }
        } else if ioinfo_get_textmode(info) == TEXTMODE_UTF16LE || console != 0 {
            j = 0;
            while i < count && j < (core::mem::size_of_val(&lfbuf) - 3) as u32 {
                if *s.add(i as usize) == b'\n' && *s.add((i + 1) as usize) == 0 {
                    lfbuf[j as usize] = b'\r';
                    j += 1;
                    lfbuf[j as usize] = 0;
                    j += 1;
                }
                lfbuf[j as usize] = *s.add(i as usize);
                j += 1;
                i += 1;
                lfbuf[j as usize] = *s.add(i as usize);
            }
        } else {
            let mut conv: [u8; 2048 / 4] =
                [0; 2048 / 4];

            j = 0;
            while i < count && j < (core::mem::size_of_val(&conv) - 3) as u32 {
                if *s.add(i as usize) == b'\n' && *s.add((i + 1) as usize) == 0 {
                    *conv.as_mut_ptr().add(j as usize) = b'\r';
                    j += 1;
                    *conv.as_mut_ptr().add(j as usize) = 0;
                    j += 1;
                }
                *conv.as_mut_ptr().add(j as usize) = *s.add(i as usize);
                j += 1;
                i += 1;
                *conv.as_mut_ptr().add(j as usize) = *s.add(i as usize);
            }

            j = WideCharToMultiByte(
                CP_UTF8,
                0,
                conv.as_ptr() as *const u16,
                (j / 2) as i32,
                lfbuf.as_mut_ptr() as *mut i8,
                core::mem::size_of_val(&lfbuf) as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as u32;
            if j == 0 {
                msvcrt_set_errno((GetLastError()) as i32);
                release_ioinfo(info);
                return -1;
            }
        }

        if console != 0 {
            j = j / 2;
            if WriteConsoleW(hand, lfbuf.as_mut_ptr() as *const core::ffi::c_void, j, &mut num_written, core::ptr::null_mut()) == 0 {
                num_written = (-1i32) as u32;
            }
        } else if WriteFile(hand, lfbuf.as_mut_ptr() as *const core::ffi::c_void, j, &mut num_written, core::ptr::null_mut()) == 0 {
            num_written = (-1i32) as u32;
        }

        if num_written != j {
            msvcrt_set_errno((GetLastError()) as i32);
            if GetLastError() == ERROR_ACCESS_DENIED {
                *_errno() = (EBADF) as i32;
            }
            release_ioinfo(info);
            return -1;
        }
    }

    release_ioinfo(info);
    count as i32
}

#[no_mangle]
pub unsafe extern "C" fn _putw(val: i32, file: *mut FILE) -> i32 {
    let len: i32;

    _lock_file(file);
    len = _write((*file)._file, &val as *const i32 as *const core::ffi::c_void, core::mem::size_of::<i32>() as u32);
    if len == core::mem::size_of::<i32>() as i32 {
        _unlock_file(file);
        return val;
    }

    (*file)._flag |= (_IOERR) as i32;
    _unlock_file(file);
    return EOF;
}

#[no_mangle]
pub unsafe extern "C" fn fclose(file: *mut FILE) -> i32 {
    let ret: i32;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return EOF;
    }

    _lock_file(file);
    ret = _fclose_nolock(file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fclose_nolock(file: *mut FILE) -> i32 {
    let r: i32;
    let flag: i32;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return EOF;
    }

    if ((*file)._flag & ((_IOREAD | _IOWRT | _IORW)) as i32) == 0 {
        (*file)._flag = 0;
        return EOF;
    }

    flag = (*file)._flag;
    free(((*file)._tmpfname).cast());
    (*file)._tmpfname = core::ptr::null_mut();

    if ((*file)._flag & (_IOWRT) as i32) != 0 {
        _fflush_nolock(file);
    }
    if ((*file)._flag & (_IOMYBUF) as i32) != 0 {
        free(((*file)._base).cast());
    }

    r = _close((*file)._file);
    (*file)._flag = 0;

    if (r == -1) || ((flag & (_IOERR) as i32) != 0) {
        EOF
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn feof(file: *mut FILE) -> i32 {
    ((*file)._flag & (_IOEOF) as i32) as i32
}

#[no_mangle]
pub unsafe extern "C" fn ferror(file: *mut FILE) -> i32 {
    ((*file)._flag & (_IOERR) as i32) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _filbuf(file: *mut FILE) -> i32 {
    let mut c: u8 = 0;

    if (*file)._flag & (_IOSTRG) as i32 != 0 {
        return EOF;
    }

    if (*file)._flag & ((MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)) as i32 == 0 {
        msvcrt_alloc_buffer(file);
    }

    if (*file)._flag & (_IOREAD) as i32 == 0 {
        if (*file)._flag & (_IORW) as i32 != 0 {
            (*file)._flag |= (_IOREAD) as i32;
        } else {
            return EOF;
        }
    }

    if (*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32 == 0 {
        let r = _read((*file)._file, &mut c as *mut u8 as *mut core::ffi::c_void, 1);
        if r != 1 {
            (*file)._flag |= (if r == 0 { _IOEOF } else { _IOERR }) as i32;
            return EOF;
        }

        return c as i32;
    } else {
        (*file)._cnt = _read((*file)._file, ((*file)._base).cast(), ((*file)._bufsiz) as u32);
        if (*file)._cnt <= 0 {
            (*file)._flag |= (if (*file)._cnt == 0 { _IOEOF } else { _IOERR }) as i32;
            (*file)._cnt = 0;
            return EOF;
        }

        (*file)._cnt -= 1;
        (*file)._ptr = (*file)._base.offset(1);
        c = *((*file)._base as *const u8);
        return c as i32;
    }
}

#[no_mangle]
pub unsafe extern "C" fn fgetc(file: *mut FILE) -> i32 {
    let ret: i32;

    _lock_file(file);
    ret = _fgetc_nolock(file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fgetc_nolock(file: *mut FILE) -> i32 {
    let mut i: *mut u8;
    let mut j: u32;

    if (*file)._cnt > 0 {
        (*file)._cnt -= 1;
        i = (*file)._ptr as *mut u8;
        (*file)._ptr = (*file)._ptr.offset(1);
        j = *i as u32;
    } else {
        j = _filbuf(file) as u32;
    }

    j as i32
}

#[no_mangle]
pub unsafe extern "C" fn _fgetchar() -> i32 {
    fgetc(stdin!())
}

#[no_mangle]
pub unsafe extern "C" fn fgets(s: *mut i8, size: i32, file: *mut FILE) -> *mut i8 {
    let mut cc: i32 = EOF;
    let buf_start: *mut i8 = s;
    let mut s_ptr = s;
    let mut remaining_size = size;

    _lock_file(file);

    while remaining_size > 1 {
        cc = _fgetc_nolock(file);
        if cc == EOF || cc == '\n' as i32 {
            break;
        }
        *s_ptr = cc as i8;
        s_ptr = s_ptr.offset(1);
        remaining_size -= 1;
    }

    if cc == EOF && s_ptr == buf_start {
        _unlock_file(file);
        return core::ptr::null_mut();
    }

    if cc != EOF && remaining_size > 1 {
        *s_ptr = cc as i8;
        s_ptr = s_ptr.offset(1);
    }

    *s_ptr = 0;

    _unlock_file(file);
    buf_start
}

#[no_mangle]
pub unsafe extern "C" fn fgetwc(file: *mut FILE) -> wint_t {
    let ret: wint_t;

    _lock_file(file);
    ret = _fgetwc_nolock(file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fgetwc_nolock(file: *mut FILE) -> wint_t {
    let mut ret: wint_t = 0;
    let mut ch: i32;

    if ioinfo_get_textmode(get_ioinfo_nolock((*file)._file)) != TEXTMODE_ANSI
            || !((*get_ioinfo_nolock((*file)._file)).wxflag & WX_TEXT != 0) {
        let mut p: *mut u8 = core::ptr::addr_of_mut!(ret) as *mut u8;
        while (p as *mut wint_t) < core::ptr::addr_of_mut!(ret).offset(1) {
            ch = _fgetc_nolock(file);
            if ch == EOF {
                ret = WEOF;
                break;
            }
            *p = ch as u8;
            p = p.offset(1);
        }
    } else {
        let mut mbs: [u8; (MB_LEN_MAX) as usize] = [0; (MB_LEN_MAX) as usize];
        let mut len: i32 = 0;

        ch = _fgetc_nolock(file);
        if ch != EOF {
            mbs[0] = ch as u8;
            if isleadbyte((mbs[0]) as i32) != 0 {
                ch = _fgetc_nolock(file);
                if ch != EOF {
                    mbs[1] = ch as u8;
                    len = 2;
                }
            } else {
                len = 1;
            }
        }

        if len == 0 || mbtowc(core::ptr::addr_of_mut!(ret), mbs.as_ptr() as *const i8, (len) as usize) == -1 {
            ret = WEOF;
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _getw(file: *mut FILE) -> i32 {
    let mut i: i32 = 0;
    let mut k: i32;
    let mut j: u32;
    let ch: *mut u8 = &mut i as *mut i32 as *mut u8;

    _lock_file(file);
    j = 0;
    while j < core::mem::size_of::<i32>() as u32 {
        k = _fgetc_nolock(file);
        if k == EOF {
            (*file)._flag |= (_IOEOF) as i32;
            _unlock_file(file);
            return EOF;
        }
        *ch.add(j as usize) = k as u8;
        j += 1;
    }

    _unlock_file(file);
    i
}

#[no_mangle]
pub unsafe extern "C" fn getwc(file: *mut FILE) -> wint_t {
    fgetwc(file)
}

#[no_mangle]
pub unsafe extern "C" fn _fgetwchar() -> wint_t {
    fgetwc(stdin!())
}

#[no_mangle]
pub unsafe extern "C" fn getwchar() -> wint_t {
    _fgetwchar()
}

#[no_mangle]
pub unsafe extern "C" fn fgetws(s: *mut wchar_t, size: i32, file: *mut FILE) -> *mut wchar_t {
    let mut cc: wint_t = WEOF;
    let buf_start: *mut wchar_t = s;
    let mut s = s;
    let mut size = size;

    _lock_file(file);

    while size > 1 {
        cc = _fgetwc_nolock(file);
        if cc == WEOF || cc == '\n' as wint_t {
            break;
        }
        *s = cc as wchar_t;
        s = s.offset(1);
        size -= 1;
    }

    if cc == WEOF && s == buf_start {
        _unlock_file(file);
        return core::ptr::null_mut();
    }

    if cc != WEOF && size > 1 {
        *s = cc as wchar_t;
        s = s.offset(1);
    }

    *s = 0;

    _unlock_file(file);
    buf_start
}

#[no_mangle]
pub unsafe extern "C" fn _flsbuf(c: i32, file: *mut FILE) -> i32 {
    if (*file)._flag & ((MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)) as i32 == 0 {
        msvcrt_alloc_buffer(file);
    }

    if (*file)._flag & (_IOWRT) as i32 == 0 {
        if (*file)._flag & (_IORW) as i32 == 0 {
            (*file)._flag |= (_IOERR) as i32;
            *_errno() = (EBADF) as i32;
            return EOF;
        }
        (*file)._flag |= (_IOWRT) as i32;
    }
    if (*file)._flag & (_IOREAD) as i32 != 0 {
        if (*file)._flag & (_IOEOF) as i32 == 0 {
            (*file)._flag |= (_IOERR) as i32;
            return EOF;
        }
        (*file)._cnt = 0;
        (*file)._ptr = (*file)._base;
        (*file)._flag &= (!(_IOREAD | _IOEOF)) as i32;
    }

    if (*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32 != 0 {
        let mut res = 0;

        if (*file)._cnt <= 0 {
            res = msvcrt_flush_buffer(file);
            if res != 0 {
                return res;
            }
            (*file)._flag |= (_IOWRT) as i32;
            (*file)._cnt = (*file)._bufsiz;
        }
        *(*file)._ptr = (c as u8) as i8;
        (*file)._ptr = (*file)._ptr.offset(1);
        (*file)._cnt -= 1;
        return c & 0xff;
    } else {
        let cc: u8 = c as u8;
        let len: i32;
        (*file)._cnt = 0;
        len = _write((*file)._file, &cc as *const u8 as *const core::ffi::c_void, 1);
        if len == 1 {
            return c & 0xff;
        }
        (*file)._flag |= (_IOERR) as i32;
        return EOF;
    }
}

#[no_mangle]
pub unsafe extern "C" fn fwrite(ptr: *const core::ffi::c_void, size: usize, nmemb: usize, file: *mut FILE) -> usize {
    let ret: usize;

    _lock_file(file);
    ret = _fwrite_nolock(ptr, size, nmemb, file);
    _unlock_file(file);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _fwrite_nolock(ptr: *const core::ffi::c_void, size: usize, nmemb: usize, file: *mut FILE) -> usize {
    let mut wrcnt: usize = size.wrapping_mul(nmemb);
    let mut written: i32 = 0;
    if size == 0 {
        return 0;
    }

    let mut ptr_offset: *const u8 = ptr as *const u8;

    while wrcnt != 0 {
        if (*file)._cnt < 0 {
            (*file)._flag |= (_IOERR) as i32;
            break;
        } else if (*file)._cnt != 0 {
            let pcnt: i32 = if (*file)._cnt > wrcnt as i32 { wrcnt as i32 } else { (*file)._cnt };
            memcpy((*file)._ptr as *mut core::ffi::c_void, ptr_offset as *const core::ffi::c_void, pcnt as usize);
            (*file)._cnt -= pcnt;
            (*file)._ptr = (*file)._ptr.offset(pcnt as isize);
            written += pcnt;
            wrcnt = wrcnt.wrapping_sub(pcnt as usize);
            ptr_offset = ptr_offset.offset(pcnt as isize);
        } else if ((*file)._flag & (MSVCRT__NOBUF) as i32) != 0
            || (((*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32) != 0 && wrcnt >= (*file)._bufsiz as usize)
            || (((*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32) == 0 && wrcnt >= MSVCRT_INTERNAL_BUFSIZ) {
            let pcnt: usize;
            let bufsiz: i32;

            if ((*file)._flag & (MSVCRT__NOBUF) as i32) != 0 {
                bufsiz = 1;
            } else if ((*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32) == 0 {
                bufsiz = (MSVCRT_INTERNAL_BUFSIZ) as i32;
            } else {
                bufsiz = (*file)._bufsiz;
            }

            pcnt = (wrcnt / bufsiz as usize).wrapping_mul(bufsiz as usize);

            if msvcrt_flush_buffer(file) == EOF {
                break;
            }

            if _write((*file)._file, ptr_offset as *const core::ffi::c_void, pcnt as u32) <= 0 {
                (*file)._flag |= (_IOERR) as i32;
                break;
            }
            written = written.wrapping_add(pcnt as i32);
            wrcnt = wrcnt.wrapping_sub(pcnt);
            ptr_offset = ptr_offset.offset(pcnt as isize);
        } else {
            if _flsbuf(*ptr_offset as i32, file) == EOF {
                break;
            }
            written = written.wrapping_add(1);
            wrcnt = wrcnt.wrapping_sub(1);
            ptr_offset = ptr_offset.offset(1 as isize);
        }
    }

    (written as usize).wrapping_div(size)
}

#[no_mangle]
pub unsafe extern "C" fn fputwc(wc: wint_t, file: *mut FILE) -> wint_t {
    let ret: wint_t;

    _lock_file(file);
    ret = _fputwc_nolock(wc, file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fputwc_nolock(wc: wint_t, file: *mut FILE) -> wint_t {
    let mwc: wchar_t = wc;
    let fdinfo: *mut ioinfo;
    let ret: wint_t;

    fdinfo = get_ioinfo_nolock((*file)._file);

    if ((*fdinfo).wxflag & WX_TEXT) != 0 && ioinfo_get_textmode(fdinfo) == TEXTMODE_ANSI {
        let mut buf: [i8; (MB_LEN_MAX) as usize] = [0; (MB_LEN_MAX) as usize];
        let char_len: i32;

        char_len = wctomb(buf.as_mut_ptr(), mwc);
        if char_len != -1 && _fwrite_nolock(buf.as_ptr() as *const core::ffi::c_void, char_len as usize, 1, file) == 1 {
            ret = wc;
        } else {
            ret = WEOF;
        }
    } else if _fwrite_nolock(core::ptr::addr_of!(mwc) as *const core::ffi::c_void, core::mem::size_of::<wchar_t>(), 1, file) == 1 {
        ret = wc;
    } else {
        ret = WEOF;
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _fputwchar(wc: wint_t) -> wint_t {
    fputwc(wc, stdout!())
}

#[no_mangle]
pub unsafe extern "C" fn _wfsopen(path: *const wchar_t, mode: *const wchar_t, share: i32) -> *mut FILE {
    let mut file: *mut FILE;
    let mut open_flags: i32 = 0;
    let mut stream_flags: i32 = 0;
    let mut fd: i32;

    if msvcrt_get_flags(mode, &mut open_flags, &mut stream_flags) == -1 {
        return core::ptr::null_mut();
    }

    LOCK_FILES!();
    fd = _wsopen(path, open_flags, share, _S_IREAD | _S_IWRITE);
    if fd < 0 {
        file = core::ptr::null_mut();
    } else {
        file = msvcrt_alloc_fp();
        if !file.is_null() && msvcrt_init_fp(file, fd, (stream_flags) as u32) != -1 {
            TRACE!(b":fd (%d) mode (%s) FILE* (%p)\n\0".as_ptr() as *const i8, fd, debugstr_w(mode), file);
        } else if !file.is_null() {
            (*file)._flag = 0;
            file = core::ptr::null_mut();
        }
    }

    if fd >= 0 && file.is_null() {
        _close(fd);
    }
    UNLOCK_FILES!();
    file
}

#[no_mangle]
pub unsafe extern "C" fn _fsopen(path: *const i8, mode: *const i8, share: i32) -> *mut FILE {
    let mut pathW: *mut wchar_t = core::ptr::null_mut();
    let mut modeW: *mut wchar_t = core::ptr::null_mut();
    let ret: *mut FILE;

    if !path.is_null() {
        pathW = wstrdupa_utf8(path);
        if pathW.is_null() {
            return core::ptr::null_mut();
        }
    }
    if !mode.is_null() {
        modeW = wstrdupa_utf8(mode);
        if modeW.is_null() {
            free((pathW).cast());
            return core::ptr::null_mut();
        }
    }

    ret = _wfsopen(pathW, modeW, share);

    free((pathW).cast());
    free((modeW).cast());
    ret
}

#[no_mangle]
pub unsafe extern "C" fn fopen(path: *const i8, mode: *const i8) -> *mut FILE {
    _fsopen(path, mode, (_SH_DENYNO) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn fopen_s(
    pFile: *mut *mut FILE,
    filename: *const i8,
    mode: *const i8,
) -> i32 {
    if MSVCRT_CHECK_PMT!(pFile != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(filename != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(mode != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }

    *pFile = fopen(filename, mode);

    if (*pFile).is_null() {
        return *_errno();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _wfopen(path: *const wchar_t, mode: *const wchar_t) -> *mut FILE {
    _wfsopen(path, mode, (_SH_DENYNO) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn _wfopen_s(
    pFile: *mut *mut FILE,
    filename: *const wchar_t,
    mode: *const wchar_t,
) -> i32 {
    if MSVCRT_CHECK_PMT!(pFile as *const core::ffi::c_void != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(filename as *const core::ffi::c_void != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(mode as *const core::ffi::c_void != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }

    *pFile = _wfopen(filename, mode);

    if (*pFile).is_null() {
        return *_errno();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fputc(c: i32, file: *mut FILE) -> i32 {
    let ret: i32;

    _lock_file(file);
    ret = _fputc_nolock(c, file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fputc_nolock(c: i32, file: *mut FILE) -> i32 {
    let res: i32;

    if (*file)._cnt > 0 {
        *(*file)._ptr = (c as u8) as i8;
        (*file)._ptr = (*file)._ptr.offset(1);
        (*file)._cnt -= 1;
        if c == '\n' as i32 {
            res = msvcrt_flush_buffer(file);
            return if res != 0 { res } else { c };
        } else {
            return c & 0xff;
        }
    } else {
        res = _flsbuf(c, file);
        return res;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _fputchar(c: i32) -> i32 {
    fputc(c, core::ptr::addr_of_mut!(MSVCRT__iob[1]))
}

#[no_mangle]
pub unsafe extern "C" fn fread(
    ptr: *mut core::ffi::c_void,
    size: usize,
    nmemb: usize,
    file: *mut FILE,
) -> usize {
    let ret: usize;

    _lock_file(file);
    ret = _fread_nolock(ptr, size, nmemb, file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _fread_nolock(
    ptr: *mut core::ffi::c_void,
    size: usize,
    nmemb: usize,
    file: *mut FILE,
) -> usize {
    let rcnt_total = size.wrapping_mul(nmemb);
    let mut rcnt = rcnt_total;
    let mut read: usize = 0;
    let mut pread: usize = 0;
    let mut ptr = ptr;

    if rcnt_total == 0 {
        return 0;
    }

    if (*file)._cnt > 0 {
        let pcnt = if rcnt > (*file)._cnt as usize {
            (*file)._cnt as usize
        } else {
            rcnt
        };
        memcpy(ptr, (*file)._ptr as *const core::ffi::c_void, pcnt);
        (*file)._cnt -= pcnt as i32;
        (*file)._ptr = (*file)._ptr.add(pcnt);
        read = read.wrapping_add(pcnt);
        rcnt = rcnt.wrapping_sub(pcnt);
        ptr = (ptr as *mut u8).add(pcnt) as *mut core::ffi::c_void;
    } else if (*file)._flag & (_IOREAD) as i32 == 0 {
        if (*file)._flag & (_IORW) as i32 != 0 {
            (*file)._flag |= (_IOREAD) as i32;
        } else {
            return 0;
        }
    }

    if rcnt > 0
        && (*file)._flag & ((MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)) as i32 == 0
    {
        msvcrt_alloc_buffer(file);
    }

    while rcnt > 0 {
        // `i` is REASSIGNED by the clamp below, not shadowed by a second binding, and
        // that is the whole contract of this loop: whatever `i` ends up holding is what
        // the tail credits to `pread`, subtracts from `rcnt`, and advances `ptr` by.
        //
        // The first translation of this made the clamp a separate `actual_i` used only
        // inside the memcpy block, so a buffered read of fewer bytes than the buffer
        // reported the whole BUFFER FILL as bytes delivered: `fread(p, 1, 1, f)` on a
        // six-byte file returned 6, moved the caller's pointer six bytes past one byte of
        // data, and underflowed `rcnt`. It survived the boot only because the callers on
        // that path ask for at least a buffer's worth and never take this branch.
        let mut i: i32;
        if (*file)._cnt == 0
            && rcnt < (*file)._bufsiz as usize
            && ((*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32) != 0
        {
            i = _read((*file)._file, (*file)._base as *mut core::ffi::c_void, (*file)._bufsiz as u32);
            (*file)._ptr = (*file)._base;
            if i != -1 {
                (*file)._cnt = i;
                // The C is `if (i > rcnt) i = rcnt;` with `i` an int against a size_t, so
                // the comparison is unsigned — which is why it has to stay inside the
                // `i != -1` arm, where `i` cannot be negative.
                if i as usize > rcnt {
                    i = rcnt as i32;
                }
            }

            if i > 0 && i < (*file)._cnt {
                (*get_ioinfo_nolock((*file)._file)).wxflag &= !WX_ATEOF;
                (*file)._flag &= (!_IOEOF) as i32;
            }
            if i > 0 {
                memcpy(ptr, (*file)._ptr as *const core::ffi::c_void, i as usize);
                (*file)._cnt -= i;
                (*file)._ptr = (*file)._ptr.add(i as usize);
            }
        } else if rcnt > (INT_MAX) as usize {
            i = _read((*file)._file, ptr, INT_MAX);
        } else if rcnt < (if (*file)._bufsiz != 0 { (*file)._bufsiz } else { (MSVCRT_INTERNAL_BUFSIZ) as i32 }) as usize {
            i = _read((*file)._file, ptr, rcnt as u32);
        } else {
            let bufsize = if (*file)._bufsiz != 0 {
                (*file)._bufsiz as usize
            } else {
                MSVCRT_INTERNAL_BUFSIZ
            };
            i = _read((*file)._file, ptr, (rcnt - rcnt % bufsize) as u32);
        }

        pread = pread.wrapping_add(i as usize);
        rcnt = rcnt.wrapping_sub(i as usize);
        ptr = (ptr as *mut u8).add(i as usize) as *mut core::ffi::c_void;

        if (*get_ioinfo_nolock((*file)._file)).wxflag & WX_ATEOF != 0 {
            (*file)._flag |= (_IOEOF) as i32;
        } else if i == -1 {
            (*file)._flag |= (_IOERR) as i32;
            pread = 0;
            rcnt = 0;
        }
        if i < 1 {
            break;
        }
    }
    read = read.wrapping_add(pread);
    read / size
}

#[no_mangle]
pub unsafe extern "C" fn _wfreopen(path: *const wchar_t, mode: *const wchar_t, file: *mut FILE) -> *mut FILE {
    let mut open_flags: i32 = 0;
    let mut stream_flags: i32 = 0;
    let mut fd: i32;
    let mut result_file: *mut FILE = file;

    LOCK_FILES!();
    if !file.is_null() {
        fclose(file);
        if msvcrt_get_flags(mode, &mut open_flags, &mut stream_flags) == -1 {
            result_file = core::ptr::null_mut();
        } else {
            fd = _wopen(path, open_flags, _S_IREAD | _S_IWRITE);
            if fd < 0 {
                result_file = core::ptr::null_mut();
            } else if msvcrt_init_fp(file, fd, stream_flags as u32) == -1 {
                (*file)._flag = 0;
                result_file = core::ptr::null_mut();
            }
        }
    }
    UNLOCK_FILES!();
    result_file
}

#[no_mangle]
pub unsafe extern "C" fn _wfreopen_s(
    pFile: *mut *mut FILE,
    path: *const wchar_t,
    mode: *const wchar_t,
    file: *mut FILE,
) -> i32 {
    if MSVCRT_CHECK_PMT!(!pFile.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!path.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!mode.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!file.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    *pFile = _wfreopen(path, mode, file);

    if (*pFile).is_null() {
        return *_errno();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn freopen(path: *const i8, mode: *const i8, file: *mut FILE) -> *mut FILE {
    let mut ret: *mut FILE;
    let mut pathW: *mut wchar_t = core::ptr::null_mut();
    let mut modeW: *mut wchar_t = core::ptr::null_mut();

    if !path.is_null() {
        pathW = wstrdupa_utf8(path);
        if pathW.is_null() {
            return core::ptr::null_mut();
        }
    }
    if !mode.is_null() {
        modeW = msvcrt_wstrdupa(mode);
        if modeW.is_null() {
            free((pathW).cast());
            return core::ptr::null_mut();
        }
    }

    ret = _wfreopen(pathW, modeW, file);

    free((pathW).cast());
    free((modeW).cast());
    ret
}

#[no_mangle]
pub unsafe extern "C" fn freopen_s(
    pFile: *mut *mut FILE,
    path: *const i8,
    mode: *const i8,
    file: *mut FILE,
) -> i32 {
    if MSVCRT_CHECK_PMT!(!pFile.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!path.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!mode.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!file.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    *pFile = freopen(path, mode, file);

    if (*pFile).is_null() {
        return *_errno();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn fsetpos(FILE: *mut FILE, pos: *mut fpos_t) -> i32 {
    return _fseeki64(FILE, *pos, (SEEK_SET) as i32);
}

#[no_mangle]
pub unsafe extern "C" fn _ftelli64(FILE: *mut FILE) -> __int64 {
    let ret: __int64;

    _lock_file(FILE);
    ret = _ftelli64_nolock(FILE);
    _unlock_file(FILE);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _ftelli64_nolock(file: *mut FILE) -> __int64 {
    let mut pos: __int64;

    pos = _telli64((*file)._file);
    if pos == -1 {
        return -1;
    }
    if (*file)._flag & ((_IOMYBUF | MSVCRT__USERBUF)) as i32 != 0 {
        if (*file)._flag & (_IOWRT) as i32 != 0 {
            pos += ((*file)._ptr as isize - (*file)._base as isize) as __int64;

            if (*get_ioinfo_nolock((*file)._file)).wxflag & WX_TEXT != 0 {
                let mut p: *mut i8;

                p = (*file)._base;
                while p < (*file)._ptr {
                    if *p == b'\n' as i8 {
                        pos += 1;
                    }
                    p = p.offset(1);
                }
            }
        } else if (*file)._cnt == 0 {
        } else if _lseeki64((*file)._file, 0, (SEEK_END) as i32) == pos {
            let mut i: i32;

            pos -= (*file)._cnt as __int64;
            if (*get_ioinfo_nolock((*file)._file)).wxflag & WX_TEXT != 0 {
                i = 0;
                while i < (*file)._cnt {
                    if *(*file)._ptr.offset(i as isize) == b'\n' as i8 {
                        pos -= 1;
                    }
                    i += 1;
                }
            }
        } else {
            let mut p: *mut i8;

            if _lseeki64((*file)._file, pos, (SEEK_SET) as i32) != pos {
                return -1;
            }

            pos -= (*file)._bufsiz as __int64;
            pos += ((*file)._ptr as isize - (*file)._base as isize) as __int64;

            if (*get_ioinfo_nolock((*file)._file)).wxflag & WX_TEXT != 0 {
                if (*get_ioinfo_nolock((*file)._file)).wxflag & WX_READNL != 0 {
                    pos -= 1;
                }

                p = (*file)._base;
                while p < (*file)._ptr {
                    if *p == b'\n' as i8 {
                        pos += 1;
                    }
                    p = p.offset(1);
                }
            }
        }
    }

    pos
}

#[no_mangle]
pub unsafe extern "C" fn ftell(file: *mut FILE) -> __msvcrt_long {
    (_ftelli64(file)) as i32
}

#[no_mangle]
pub unsafe extern "C" fn fgetpos(FILE: *mut FILE, pos: *mut fpos_t) -> i32 {
    *pos = _ftelli64(FILE);
    if *pos == -1 {
        return -1;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn fputs(s: *const i8, file: *mut FILE) -> i32 {
    let len = strlen(s);
    let ret: i32;

    _lock_file(file);
    ret = if _fwrite_nolock(s as *const core::ffi::c_void, core::mem::size_of::<i8>(), len, file) == len {
        0
    } else {
        EOF
    };
    _unlock_file(file);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn fputws(s: *const wchar_t, file: *mut FILE) -> i32 {
    let len: usize = wcslen(s);
    let tmp_buf: u8;
    let ret: i32;

    _lock_file(file);
    if ((*get_ioinfo_nolock((*file)._file)).wxflag & WX_TEXT) == 0 {
        ret = if _fwrite_nolock(s as *const core::ffi::c_void, core::mem::size_of::<wchar_t>(), len, file) == len {
            0
        } else {
            EOF
        };
        _unlock_file(file);
        return ret;
    }

    tmp_buf = add_std_buffer(file);
    let mut i: usize = 0;
    while i < len {
        if _fputwc_nolock(*s.add(i), file) == WEOF {
            if tmp_buf != 0 {
                remove_std_buffer(file);
            }
            _unlock_file(file);
            return (WEOF) as i32;
        }
        i += 1;
    }

    if tmp_buf != 0 {
        remove_std_buffer(file);
    }
    _unlock_file(file);
    0
}

#[no_mangle]
pub unsafe extern "C" fn getchar() -> i32 {
    fgetc(core::ptr::addr_of_mut!(MSVCRT__iob[0]))
}

#[no_mangle]
pub unsafe extern "C" fn getc(file: *mut FILE) -> i32 {
    fgetc(file)
}

#[no_mangle]
pub unsafe extern "C" fn gets_s(buf: *mut i8, len: usize) -> *mut i8 {
    let buf_start = buf;
    let mut cc: i32;
    let mut buf = buf;
    let mut len = len;

    if MSVCRT_CHECK_PMT!(buf != core::ptr::null_mut()) == 0 { return core::ptr::null_mut(); }
    if MSVCRT_CHECK_PMT!(len != 0) == 0 { return core::ptr::null_mut(); }

    _lock_file(stdin!());
    cc = _fgetc_nolock(stdin!());
    while len != 0 && cc != EOF && cc != '\n' as i32 {
        if cc != '\r' as i32 {
            *buf = cc as i8;
            buf = buf.offset(1);
            len -= 1;
        }
        cc = _fgetc_nolock(stdin!());
    }
    _unlock_file(stdin!());

    if len == 0 {
        *buf_start = 0;
        _invalid_parameter(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), 0, 0);
        return core::ptr::null_mut();
    }

    if cc == EOF && buf_start == buf {
        return core::ptr::null_mut();
    }
    *buf = '\0' as i8;

    buf_start
}

#[no_mangle]
pub unsafe extern "C" fn gets(buf: *mut i8) -> *mut i8 {
    gets_s(buf, (-1i32) as usize)
}

#[no_mangle]
pub unsafe extern "C" fn _getws(buf: *mut wchar_t) -> *mut wchar_t {
    let mut cc: wint_t;
    let ws: *mut wchar_t = buf;
    let mut buf = buf;

    _lock_file(stdin!());
    cc = _fgetwc_nolock(stdin!());
    while cc != WEOF && cc != ('\n' as i32) as u16 {
        if cc != ('\r' as i32) as u16 {
            *buf = cc as wchar_t;
            buf = buf.offset(1);
        }
        cc = _fgetwc_nolock(stdin!());
    }
    _unlock_file(stdin!());

    if cc == WEOF && ws == buf {
        return core::ptr::null_mut();
    }
    *buf = '\0' as wchar_t;

    ws
}

#[no_mangle]
pub unsafe extern "C" fn putc(c: i32, file: *mut FILE) -> i32 {
    fputc(c, file)
}

#[no_mangle]
pub unsafe extern "C" fn putchar(c: i32) -> i32 {
    fputc(c, stdout!())
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const i8) -> i32 {
    let len = strlen(s);
    let ret: i32;

    _lock_file(core::ptr::addr_of_mut!(MSVCRT__iob[1]));
    if _fwrite_nolock(s as *const core::ffi::c_void, core::mem::size_of::<i8>(), len, core::ptr::addr_of_mut!(MSVCRT__iob[1])) != len {
        _unlock_file(core::ptr::addr_of_mut!(MSVCRT__iob[1]));
        return EOF;
    }

    ret = if _fwrite_nolock(b"\n\0".as_ptr() as *const core::ffi::c_void, 1, 1, core::ptr::addr_of_mut!(MSVCRT__iob[1])) == 1 {
        0
    } else {
        EOF
    };
    _unlock_file(core::ptr::addr_of_mut!(MSVCRT__iob[1]));
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _putws(s: *const wchar_t) -> i32 {
    let mut ret: i32;

    _lock_file(stdout!());
    ret = fputws(s, stdout!());
    if ret >= 0 {
        ret = (_fputwc_nolock('\n' as wint_t, stdout!())) as i32;
    }
    _unlock_file(stdout!());
    if ret >= 0 { 0 } else { (WEOF) as i32 }
}

#[no_mangle]
pub unsafe extern "C" fn remove(path: *const i8) -> i32 {
    _unlink(path)
}

#[no_mangle]
pub unsafe extern "C" fn _wremove(path: *const wchar_t) -> i32 {
    _wunlink(path)
}

#[no_mangle]
pub unsafe extern "C" fn rename(oldpath: *const i8, newpath: *const i8) -> i32 {
    let mut oldpathW: *mut wchar_t = core::ptr::null_mut();
    let mut newpathW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !oldpath.is_null() && { oldpathW = wstrdupa_utf8(oldpath); oldpathW.is_null() } {
        return -1;
    }
    if !newpath.is_null() && { newpathW = wstrdupa_utf8(newpath); newpathW.is_null() } {
        free((oldpathW).cast());
        return -1;
    }
    ret = _wrename(oldpathW, newpathW);
    free((oldpathW).cast());
    free((newpathW).cast());
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wrename(oldpath: *const wchar_t, newpath: *const wchar_t) -> i32 {
    if MoveFileExW(oldpath, newpath, MOVEFILE_COPY_ALLOWED) != 0 {
        return 0;
    }
    msvcrt_set_errno((GetLastError()) as i32);
    return -1;
}

#[no_mangle]
pub unsafe extern "C" fn setvbuf(file: *mut FILE, buf: *mut i8, mode: i32, size: usize) -> i32 {
    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return -1;
    }
    if MSVCRT_CHECK_PMT!(mode == (_IONBF) as i32 || mode == (_IOFBF) as i32 || mode == (_IOLBF) as i32) == 0 {
        return -1;
    }
    if MSVCRT_CHECK_PMT!(mode == (_IONBF) as i32 || (size >= 2 && size <= (INT_MAX) as usize)) == 0 {
        return -1;
    }

    _lock_file(file);

    _fflush_nolock(file);
    if (*file)._flag & (_IOMYBUF) as i32 != 0 {
        free((*file)._base as *mut core::ffi::c_void);
    }
    (*file)._flag &= (!(MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)) as i32;
    (*file)._cnt = 0;

    if mode == (_IONBF) as i32 {
        (*file)._flag |= (MSVCRT__NOBUF) as i32;
        (*file)._base = core::ptr::addr_of_mut!((*file)._charbuf) as *mut i8;
        (*file)._ptr = (*file)._base;
        (*file)._bufsiz = 2;
    } else if buf != core::ptr::null_mut() {
        (*file)._base = buf;
        (*file)._ptr = buf;
        (*file)._flag |= (MSVCRT__USERBUF) as i32;
        (*file)._bufsiz = size as i32;
    } else {
        (*file)._base = (malloc(size)).cast();
        (*file)._ptr = (*file)._base;
        if (*file)._base == core::ptr::null_mut() {
            (*file)._bufsiz = 0;
            _unlock_file(file);
            return -1;
        }

        (*file)._flag |= (_IOMYBUF) as i32;
        (*file)._bufsiz = size as i32;
    }
    _unlock_file(file);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn setbuf(file: *mut FILE, buf: *mut i8) {
    setvbuf(file, buf, if !buf.is_null() { (_IOFBF) as i32 } else { (_IONBF) as i32 }, (BUFSIZ) as usize);
}
unsafe extern "C" fn tmpnam_helper(s: PCHAR, size: usize, unique_counter: *mut LONG, tmp_max: i32) -> i32 {
    let mut tmpstr: [i8; 8] = [0; 8];
    let mut p = s;
    let mut digits: i32;

    if MSVCRT_CHECK_PMT!(s != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }

    if size < 3 {
        if size != 0 {
            *s = 0;
        }
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }
    *p = b'\\' as i8;
    p = p.offset(1);
    *p = b's' as i8;
    p = p.offset(1);
    let mut size = size - 2;
    digits = msvcrt_int_to_base32(GetCurrentProcessId() as i32, tmpstr.as_mut_ptr());
    // `(digits + 1) as usize > size`, NOT `digits + 1 > size as i32`. The C compares an
    // `int` against a `size_t`, which the usual arithmetic conversions make an UNSIGNED
    // comparison; casting the other way truncates AND re-signs it. `tmpnam` passes
    // `(size_t)-1`, whose low 32 bits are -3 as an i32 — so this guard fired on every
    // call and `tmpnam` returned NULL with ERANGE, always, in the shipping DLL.
    if (digits + 1) as usize > size {
        *s = 0;
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }
    memcpy(p as *mut core::ffi::c_void, tmpstr.as_ptr() as *const core::ffi::c_void, (digits as usize) * core::mem::size_of_val(&tmpstr[0]));
    p = p.offset(digits as isize);
    *p = b'.' as i8;
    p = p.offset(1);
    size -= (digits as usize) + 1;

    loop {
        loop {
            digits = *unique_counter;
            if digits + 1 >= tmp_max {
                break;
            }
            if InterlockedCompareExchange(unique_counter, digits + 1, digits) == digits {
                break;
            }
        }

        digits = msvcrt_int_to_base32(digits, tmpstr.as_mut_ptr());
        // Same unsigned-comparison rule as tmpnam_helper above; `_wtmpnam` passes (size_t)-1.
    if (digits + 1) as usize > size {
            *s = 0;
            *_errno() = (ERANGE) as i32;
            return (ERANGE) as i32;
        }
        memcpy(p as *mut core::ffi::c_void, tmpstr.as_ptr() as *const core::ffi::c_void, (digits as usize) * core::mem::size_of_val(&tmpstr[0]));
        *p.offset(digits as isize) = 0;

        if GetFileAttributesA(s) == INVALID_FILE_ATTRIBUTES && GetLastError() == ERROR_FILE_NOT_FOUND {
            break;
        }
        if 1 == 0 {
            break;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn tmpnam_s(s: PCHAR, size: usize) -> i32 {
    return tmpnam_helper(s, size, core::ptr::addr_of_mut!(tmpnam_s_unique), (TMP_MAX_S) as i32);
}

#[no_mangle]
pub unsafe extern "C" fn tmpnam(s: PCHAR) -> PCHAR {
    let mut s = s;
    if s.is_null() {
        let data: *mut thread_data_t = msvcrt_get_thread_data();
        if (*data).tmpnam_buffer.is_null() {
            (*data).tmpnam_buffer = malloc((MAX_PATH) as usize) as PCHAR;
        }
        s = (*data).tmpnam_buffer;
    }

    if tmpnam_helper(s, -(1 as isize) as usize, core::ptr::addr_of_mut!(tmpnam_unique), (TMP_MAX) as i32) != 0 {
        core::ptr::null_mut()
    } else {
        s
    }
}
unsafe extern "C" fn wtmpnam_helper(s: *mut wchar_t, size: usize, unique_counter: *mut LONG, tmp_max: i32) -> i32 {
    let mut tmpstr: [wchar_t; 8] = [0; 8];
    let mut p = s;
    let mut digits: i32;

    if MSVCRT_CHECK_PMT!(s != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }

    if size < 3 {
        if size != 0 {
            *s = 0;
        }
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }
    *p = '\\' as wchar_t;
    p = p.offset(1);
    *p = 's' as wchar_t;
    p = p.offset(1);
    let mut size = size - 2;
    digits = msvcrt_int_to_base32_w((GetCurrentProcessId()) as i32, tmpstr.as_mut_ptr());
    if (digits + 1) as usize > size {
        *s = 0;
        *_errno() = (ERANGE) as i32;
        return (ERANGE) as i32;
    }
    memcpy(p as *mut core::ffi::c_void, tmpstr.as_ptr() as *const core::ffi::c_void, (digits as usize) * core::mem::size_of::<wchar_t>());
    p = p.offset(digits as isize);
    *p = '.' as wchar_t;
    p = p.offset(1);
    size -= (digits + 1) as usize;

    loop {
        loop {
            digits = *unique_counter;
            if digits + 1 >= tmp_max {
                break;
            }
            if InterlockedCompareExchange(unique_counter, digits + 1, digits) == digits {
                break;
            }
        }

        digits = msvcrt_int_to_base32_w(digits, tmpstr.as_mut_ptr());
        if (digits + 1) as usize > size {
            *s = 0;
            *_errno() = (ERANGE) as i32;
            return (ERANGE) as i32;
        }
        memcpy(p as *mut core::ffi::c_void, tmpstr.as_ptr() as *const core::ffi::c_void, (digits as usize) * core::mem::size_of::<wchar_t>());
        *p.offset(digits as isize) = 0;

        if GetFileAttributesW(s) == INVALID_FILE_ATTRIBUTES &&
            GetLastError() == ERROR_FILE_NOT_FOUND {
            break;
        }
        if 1 == 0 {
            break;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _wtmpnam_s(s: *mut wchar_t, size: usize) -> i32 {
    wtmpnam_helper(s, size, core::ptr::addr_of_mut!(tmpnam_s_unique), (TMP_MAX_S) as i32)
}

#[no_mangle]
pub unsafe extern "C" fn _wtmpnam(s: *mut wchar_t) -> *mut wchar_t {
    let s = if s.is_null() {
        let data = msvcrt_get_thread_data();
        if (*data).wtmpnam_buffer.is_null() {
            (*data).wtmpnam_buffer = malloc(core::mem::size_of::<[wchar_t; (MAX_PATH) as usize]>()) as *mut wchar_t;
        }
        (*data).wtmpnam_buffer
    } else {
        s
    };

    if wtmpnam_helper(s, (-1i32) as usize, core::ptr::addr_of_mut!(tmpnam_unique), (TMP_MAX) as i32) != 0 {
        core::ptr::null_mut()
    } else {
        s
    }
}

#[no_mangle]
pub unsafe extern "C" fn tmpfile() -> *mut FILE {
    let filename = _tempnam(b",\0".as_ptr() as *const i8, b"t\0".as_ptr() as *const i8);
    let mut fd: i32;
    let mut file: *mut FILE = core::ptr::null_mut();

    LOCK_FILES!();
    fd = _open(filename, (_O_CREAT | _O_BINARY | _O_RDWR | _O_TEMPORARY) as i32,
            _S_IREAD | _S_IWRITE);
    if fd != -1 && { file = msvcrt_alloc_fp(); !file.is_null() }
    {
        if msvcrt_init_fp(file, fd, _IORW) == -1
        {
            (*file)._flag = 0;
            file = core::ptr::null_mut();
        }
        else {
            (*file)._tmpfname = _strdup(filename);
        }
    }

    if fd != -1 && file.is_null() {
        _close(fd);
    }
    free((filename).cast());
    UNLOCK_FILES!();
    file
}

#[no_mangle]
pub unsafe extern "C" fn tmpfile_s(file: *mut *mut FILE) -> i32 {
    if MSVCRT_CHECK_PMT!(!file.is_null()) == 0 {
        return (EINVAL) as i32;
    }

    *file = tmpfile();
    0
}
// `PutsClbk<u8>` (printf.rs:275) spells the buffer `*const u8`; the C is `const char*`.
// Same ABI, and the body immediately casts to c_void, so only the spelling changes.
unsafe extern "C" fn puts_clbk_file_a(file: *mut core::ffi::c_void, len: i32, str: *const u8) -> i32 {
    fwrite(str as *const core::ffi::c_void, core::mem::size_of::<i8>(), len as usize, file as *mut FILE) as i32
}
unsafe extern "C" fn puts_clbk_file_w(file: *mut core::ffi::c_void, len: i32, str: *const wchar_t) -> i32 {
    let mut i: i32;
    let ret: i32;

    _lock_file((file).cast());

    if ((*get_ioinfo_nolock((*(file as *mut FILE))._file)).wxflag & WX_TEXT) == 0 {
        ret = _fwrite_nolock(str as *const core::ffi::c_void, core::mem::size_of::<wchar_t>(), len as usize, file as *mut FILE) as i32;
        _unlock_file((file).cast());
        return ret;
    }

    i = 0;
    while i < len {
        if _fputwc_nolock(*str.offset(i as isize), file as *mut FILE) == WEOF {
            _unlock_file((file).cast());
            return -1;
        }
        i += 1;
    }

    _unlock_file((file).cast());
    return len;
}
unsafe extern "C" fn vfprintf_helper(mut options: u32, file: *mut FILE, format: *const i8,
        locale: _locale_t, mut valist: va_list) -> i32 {
    let mut args_ctx: [printf_arg; (_ARGMAX + 1) as usize] = core::mem::zeroed();
    let tmp_buf: u8;
    let mut ret: i32;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 { return -1; }
    if MSVCRT_CHECK_PMT!(format != core::ptr::null()) == 0 { return -1; }

    if (options & MSVCRT_PRINTF_POSITIONAL_PARAMS) != 0 {
        memset(args_ctx.as_mut_ptr() as *mut core::ffi::c_void, 0, core::mem::size_of_val(&args_ctx));
        ret = create_positional_ctx_a(args_ctx.as_mut_ptr() as *mut c_void, format as *const u8, valist as *mut u8);
        if ret < 0 {
            _invalid_parameter(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
            *_errno() = (EINVAL) as i32;
            return ret;
        } else if ret == 0 {
            options &= !MSVCRT_PRINTF_POSITIONAL_PARAMS;
        }
    }

    _lock_file(file);
    tmp_buf = add_std_buffer(file);
    ret = pf_printf_a(puts_clbk_file_a, file as *mut core::ffi::c_void, format as *const u8, locale as *mut core::ffi::c_void, options,
            if (options & MSVCRT_PRINTF_POSITIONAL_PARAMS) != 0 { arg_clbk_positional as crate::printf::ArgsClbk } else { arg_clbk_valist as crate::printf::ArgsClbk },
            if (options & MSVCRT_PRINTF_POSITIONAL_PARAMS) != 0 { args_ctx.as_mut_ptr() as *mut core::ffi::c_void } else { core::ptr::null_mut() },
            core::ptr::addr_of_mut!(valist) as *mut *mut u8);
    if tmp_buf != 0 { remove_std_buffer(file); }
    _unlock_file(file);

    ret
}
unsafe extern "C" fn vfwprintf_helper(
    options: u32,
    file: *mut FILE,
    format: *const wchar_t,
    locale: _locale_t,
    mut valist: va_list,
) -> i32 {
    let mut args_ctx: [printf_arg; (_ARGMAX + 1) as usize] =
        core::mem::zeroed();
    let tmp_buf: u8;
    let mut ret: i32;
    let mut options = options;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return -1;
    }
    if MSVCRT_CHECK_PMT!(format != core::ptr::null()) == 0 {
        return -1;
    }

    if (options & MSVCRT_PRINTF_POSITIONAL_PARAMS) != 0 {
        memset(
            args_ctx.as_mut_ptr() as *mut core::ffi::c_void,
            0,
            core::mem::size_of_val(&args_ctx),
        );
        ret = create_positional_ctx_w(args_ctx.as_mut_ptr() as *mut c_void, format as *const u16, valist as *mut u8);
        if ret < 0 {
            _invalid_parameter(
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
                0,
                0,
            );
            *_errno() = (EINVAL) as i32;
            return ret;
        } else if ret == 0 {
            options &= !MSVCRT_PRINTF_POSITIONAL_PARAMS;
        }
    }

    _lock_file(file);
    tmp_buf = add_std_buffer(file);
    ret = pf_printf_w(
        puts_clbk_file_w,
        file as *mut core::ffi::c_void,
        format as *const u16,
        locale as *mut core::ffi::c_void,
        options,
        if (options & MSVCRT_PRINTF_POSITIONAL_PARAMS) != 0 {
            arg_clbk_positional as crate::printf::ArgsClbk
        } else {
            arg_clbk_valist as crate::printf::ArgsClbk
        },
        if (options & MSVCRT_PRINTF_POSITIONAL_PARAMS) != 0 {
            args_ctx.as_mut_ptr() as *mut core::ffi::c_void
        } else {
            core::ptr::null_mut()
        },
        core::ptr::addr_of_mut!(valist) as *mut *mut u8,
    );
    if tmp_buf != 0 {
        remove_std_buffer(file);
    }
    _unlock_file(file);

    ret
}

#[no_mangle]
pub unsafe extern "C" fn _vfprintf_s_l(
    file: *mut FILE,
    format: *const i8,
    locale: _locale_t,
    valist: va_list) -> i32 {
    vfprintf_helper(MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER, file, format, locale, valist)
}

#[no_mangle]
pub unsafe extern "C" fn _vfwprintf_s_l(
    file: *mut FILE,
    format: *const wchar_t,
    locale: _locale_t,
    valist: va_list,
) -> i32 {
    vfwprintf_helper(MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER, file, format, locale, valist)
}
#[no_mangle]
pub unsafe extern "C" fn vfprintf(FILE: *mut FILE, format: *const i8, mut valist: va_list) -> i32 {
    return vfprintf_helper(0, FILE, format, core::ptr::null_mut(), valist);
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf_s(
    file: *mut FILE,
    format: *const i8,
    valist: va_list,
) -> i32 {
    _vfprintf_s_l(file, format, core::ptr::null_mut(), valist)
}

#[no_mangle]
pub unsafe extern "C" fn vfwprintf(
    file: *mut FILE,
    format: *const wchar_t,
    mut valist: va_list,
) -> i32 {
    vfwprintf_helper(0, file, format, core::ptr::null_mut(), valist)
}

#[no_mangle]
pub unsafe extern "C" fn vfwprintf_s(
    file: *mut FILE,
    format: *const wchar_t,
    valist: va_list,
) -> i32 {
    _vfwprintf_s_l(file, format, core::ptr::null_mut(), valist)
}

#[no_mangle]
pub unsafe extern "C" fn _vfprintf_l(
    file: *mut FILE,
    format: *const i8,
    locale: _locale_t,
    valist: va_list) -> i32 {
    vfprintf_helper(0, file, format, locale, valist)
}

#[no_mangle]
pub unsafe extern "C" fn _vfwprintf_l(
    file: *mut FILE,
    format: *const wchar_t,
    locale: _locale_t,
    valist: va_list
) -> i32 {
    vfwprintf_helper(0, file, format, locale, valist)
}

#[no_mangle]
pub unsafe extern "C" fn _vfprintf_p_l(
    file: *mut FILE,
    format: *const i8,
    locale: _locale_t,
    valist: va_list) -> i32 {
    vfprintf_helper(
        MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
        file,
        format,
        locale,
        valist,
    )
}

#[no_mangle]
pub unsafe extern "C" fn _vfprintf_p(
    file: *mut FILE,
    format: *const i8,
    valist: va_list,
) -> i32 {
    _vfprintf_p_l(file, format, core::ptr::null_mut(), valist)
}

#[no_mangle]
pub unsafe extern "C" fn _vfwprintf_p_l(
    file: *mut FILE,
    format: *const wchar_t,
    locale: _locale_t,
    valist: va_list
) -> i32 {
    vfwprintf_helper(
        MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
        file,
        format,
        locale,
        valist,
    )
}

#[no_mangle]
pub unsafe extern "C" fn _vfwprintf_p(
    file: *mut FILE,
    format: *const wchar_t,
    valist: va_list,
) -> i32 {
    _vfwprintf_p_l(file, format, core::ptr::null_mut(), valist)
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(format: *const i8, valist: va_list) -> i32 {
    vfprintf(stdout!(), format, valist)
}

#[no_mangle]
pub unsafe extern "C" fn vprintf_s(format: *const i8, valist: va_list) -> i32 {
    vfprintf_s(stdout!(), format, valist)
}

#[no_mangle]
pub unsafe extern "C" fn vwprintf(
    format: *const wchar_t,
    mut valist: va_list,
) -> i32 {
    vfwprintf(stdout!(), format, valist)
}

#[no_mangle]
pub unsafe extern "C" fn vwprintf_s(
    format: *const wchar_t,
    valist: va_list,
) -> i32 {
    vfwprintf_s(MSVCRT__iob.as_mut_ptr().offset(1), format, valist)
}















#[no_mangle]
pub unsafe extern "C" fn ungetc(c: i32, file: *mut FILE) -> i32 {
    let ret: i32;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return EOF;
    }

    _lock_file(file);
    ret = _ungetc_nolock(c, file);
    _unlock_file(file);

    ret
}
#[no_mangle]
pub unsafe extern "C" fn _ungetc_nolock(c: i32, file: *mut FILE) -> i32 {
    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return EOF;
    }

    if c == EOF || ((*file)._flag & (_IOREAD) as i32 == 0 &&
            ((*file)._flag & (_IORW) as i32 == 0 || (*file)._flag & (_IOWRT) as i32 != 0)) {
        return EOF;
    }

    if ((*file)._flag & ((MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)) as i32 == 0
            && msvcrt_alloc_buffer(file) != 0)
            || ((*file)._cnt == 0 && (*file)._ptr == (*file)._base) {
        (*file)._ptr = (*file)._ptr.offset(1);
    }

    if (*file)._ptr > (*file)._base {
        (*file)._ptr = (*file)._ptr.offset(-1);
        if (*file)._flag & (_IOSTRG) as i32 != 0 {
            if *(*file)._ptr != (c as u8) as i8 {
                (*file)._ptr = (*file)._ptr.offset(1);
                return EOF;
            }
        } else {
            *(*file)._ptr = (c as u8) as i8;
        }
        (*file)._cnt += 1;
        (*file)._flag &= (!(_IOERR | _IOEOF)) as i32;
        (*file)._flag |= (_IOREAD) as i32;
        return c;
    }

    return EOF;
}

#[no_mangle]
pub unsafe extern "C" fn ungetwc(wc: wint_t, file: *mut FILE) -> wint_t {
    let ret: wint_t;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return WEOF;
    }

    _lock_file(file);
    ret = _ungetwc_nolock(wc, file);
    _unlock_file(file);

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn _ungetwc_nolock(wc: wint_t, file: *mut FILE) -> wint_t {
    let mwc: wchar_t = wc;

    if MSVCRT_CHECK_PMT!(file != core::ptr::null_mut()) == 0 {
        return WEOF;
    }
    if wc == WEOF {
        return WEOF;
    }

    if ioinfo_get_textmode(get_ioinfo_nolock((*file)._file)) != TEXTMODE_ANSI
        || ((*get_ioinfo_nolock((*file)._file)).wxflag & WX_TEXT) == 0
    {
        let pp: *mut u8 = &mwc as *const wchar_t as *mut u8;
        let mut i: i32 = core::mem::size_of::<wchar_t>() as i32 - 1;
        while i >= 0 {
            if *pp.offset(i as isize) != _ungetc_nolock(*pp.offset(i as isize) as i32, file) as u8 {
                return WEOF;
            }
            i -= 1;
        }
    } else {
        let mut mbs: [i8; MB_LEN_MAX as usize] = [0; MB_LEN_MAX as usize];
        let mut len: i32;

        len = wctomb(mbs.as_mut_ptr(), mwc);
        if len == -1 {
            return WEOF;
        }

        len -= 1;
        while len >= 0 {
            if mbs[len as usize] != _ungetc_nolock(mbs[len as usize] as i32, file) as i8 {
                return WEOF;
            }
            len -= 1;
        }
    }

    return mwc;
}



#[no_mangle]
pub unsafe extern "C" fn _getmaxstdio() -> i32 {
    MSVCRT_max_streams
}

#[no_mangle]
pub unsafe extern "C" fn _setmaxstdio(newmax: i32) -> i32 {
    if newmax < _IOB_ENTRIES as i32
        || newmax > (MSVCRT_MAX_FILES) as i32
        || newmax < MSVCRT_stream_idx
    {
        return -1;
    }

    MSVCRT_max_streams = newmax;
    MSVCRT_max_streams
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll file functions
//  *
//  * Copyright 1996,1998 Marcus Meissner
//  * Copyright 1996 Jukka Iivonen
//  * Copyright 1997,2000 Uwe Bonnes
//  * Copyright 2000 Jon Griffiths
//  * Copyright 2004 Eric Pouech
//  * Copyright 2004 Juan Lang
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
//  *
//  */
//
// #include <direct.h>
// #include <fcntl.h>
// #include <io.h>
// #include <share.h>
// #include <stdarg.h>
// #include <sys/locking.h>
// #include <sys/types.h>
// #include <sys/utime.h>
// #include <time.h>
// #include <limits.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "wincon.h"
// #include "winternl.h"
// #include "winnls.h"
// #include "msvcrt.h"
// #include "mtdll.h"
// #include "wine/asm.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// #undef _fstat
// #undef _fstati64
// #undef _stat
// #undef _stati64
// #undef _wstat
// #undef _wstati64
// #undef _fstat32
// #undef _fstat32i64
// #undef _stat32i64
// #undef _stat32
// #undef _wstat32
// #undef _wstat32i64
// #undef _fstat64i32
// #undef _fstat64
// #undef _stat64
// #undef _stat64i32
// #undef _wstat64i32
// #undef _wstat64
// int __cdecl _wstat64(const wchar_t*, struct _stat64*);
//
// /* for stat mode, permissions apply to all,owner and group */
// #define ALL_S_IREAD  (_S_IREAD  | (_S_IREAD  >> 3) | (_S_IREAD  >> 6))
// #define ALL_S_IWRITE (_S_IWRITE | (_S_IWRITE >> 3) | (_S_IWRITE >> 6))
// #define ALL_S_IEXEC  (_S_IEXEC  | (_S_IEXEC  >> 3) | (_S_IEXEC  >> 6))
//
// /* _access() bit flags FIXME: incomplete */
// #define MSVCRT_W_OK      0x02
// #define MSVCRT_R_OK      0x04
//
// /* values for wxflag in file descriptor */
// #define WX_OPEN           0x01
// #define WX_ATEOF          0x02
// #define WX_READNL         0x04  /* read started with \n */
// #define WX_PIPE           0x08
// #define WX_DONTINHERIT    0x10
// #define WX_APPEND         0x20
// #define WX_TTY            0x40
// #define WX_TEXT           0x80
//
// static char utf8_bom[3] = { 0xef, 0xbb, 0xbf };
// static char utf16_bom[2] = { 0xff, 0xfe };
//
// #define MSVCRT_INTERNAL_BUFSIZ 4096
//
// enum textmode
// {
//     TEXTMODE_ANSI,
//     TEXTMODE_UTF8,
//     TEXTMODE_UTF16LE,
// };
//
//
// #define MSVCRT_MAX_FILES 2048
// #define MSVCRT_FD_BLOCK_SIZE 32
//
// typedef struct {
//     HANDLE              handle;
//     unsigned char       wxflag;
//     char                lookahead[3];
//     int                 exflag;
//     CRITICAL_SECTION    crit;
// #if _MSVCR_VER >= 80
//     char textmode : 7;
//     char unicode : 1;
//     char pipech2[2];
//     __int64 startpos;
//     BOOL utf8translations;
//     char dbcsBuffer[1];
//     BOOL dbcsBufferUsed;
// #endif
// } ioinfo;
//
// /*********************************************************************
//  *		__badioinfo (MSVCRT.@)
//  */
// ioinfo MSVCRT___badioinfo = { INVALID_HANDLE_VALUE, WX_TEXT };
//
// /*********************************************************************
//  *		__pioinfo (MSVCRT.@)
//  * array of pointers to ioinfo arrays [32]
//  */
// ioinfo * MSVCRT___pioinfo[MSVCRT_MAX_FILES/MSVCRT_FD_BLOCK_SIZE] = { 0 };
//
//
// #define EF_UTF8           0x01
// #define EF_UTF16          0x02
// #define EF_CRIT_INIT      0x04
// #define EF_UNK_UNICODE    0x08
//
// static inline BOOL ioinfo_is_crit_init(ioinfo *info)
// {
//     return info->exflag & EF_CRIT_INIT;
// }
//
// static inline void ioinfo_set_crit_init(ioinfo *info)
// {
//     info->exflag |= EF_CRIT_INIT;
// }
//
// static inline enum textmode ioinfo_get_textmode(ioinfo *info)
// {
//     if (info->exflag & EF_UTF8)
//         return TEXTMODE_UTF8;
//     if (info->exflag & EF_UTF16)
//         return TEXTMODE_UTF16LE;
//     return TEXTMODE_ANSI;
// }
//
// static inline void ioinfo_set_textmode(ioinfo *info, enum textmode mode)
// {
//     info->exflag &= EF_CRIT_INIT | EF_UNK_UNICODE;
//     switch (mode)
//     {
//         case TEXTMODE_ANSI:
//             break;
//         case TEXTMODE_UTF8:
//             info->exflag |= EF_UTF8;
//             break;
//         case TEXTMODE_UTF16LE:
//             info->exflag |= EF_UTF16;
//             break;
//     }
// }
//
// static inline void ioinfo_set_unicode(ioinfo *info, BOOL unicode)
// {
//     if (unicode)
//         info->exflag |= EF_UNK_UNICODE;
//     else
//         info->exflag &= ~EF_UNK_UNICODE;
// }
//
// typedef struct {
//     FILE file;
//     CRITICAL_SECTION crit;
// } file_crit;
//
// FILE MSVCRT__iob[_IOB_ENTRIES] = { { 0 } };
//
// static FILE* iob_get_file(int i)
// {
//     return &MSVCRT__iob[i];
// }
//
// static CRITICAL_SECTION* file_get_cs(FILE *f)
// {
//     if (f < iob_get_file(0) || f >= iob_get_file(_IOB_ENTRIES))
//         return &((file_crit*)f)->crit;
//     return NULL;
// }
//
// static file_crit* MSVCRT_fstream[MSVCRT_MAX_FILES/MSVCRT_FD_BLOCK_SIZE];
// static int MSVCRT_max_streams = 512, MSVCRT_stream_idx;
//
// /* INTERNAL: process umask */
// static int MSVCRT_umask = 0;
//
// /* INTERNAL: static data for tmpnam and _wtmpname functions */
// static LONG tmpnam_unique;
// static LONG tmpnam_s_unique;
//
// #define TOUL(x) (ULONGLONG)(x)
// static const ULONGLONG WCEXE = TOUL('e') << 32 | TOUL('x') << 16 | TOUL('e');
// static const ULONGLONG WCBAT = TOUL('b') << 32 | TOUL('a') << 16 | TOUL('t');
// static const ULONGLONG WCCMD = TOUL('c') << 32 | TOUL('m') << 16 | TOUL('d');
// static const ULONGLONG WCCOM = TOUL('c') << 32 | TOUL('o') << 16 | TOUL('m');
//
// /* This critical section protects the MSVCRT_fstreams table
//  * and MSVCRT_stream_idx from race conditions. It also
//  * protects fd critical sections creation code.
//  */
// static CRITICAL_SECTION MSVCRT_file_cs;
// static CRITICAL_SECTION_DEBUG MSVCRT_file_cs_debug =
// {
//     0, 0, &MSVCRT_file_cs,
//     { &MSVCRT_file_cs_debug.ProcessLocksList, &MSVCRT_file_cs_debug.ProcessLocksList },
//       0, 0, { (DWORD_PTR)(__FILE__ ": MSVCRT_file_cs") }
// };
// static CRITICAL_SECTION MSVCRT_file_cs = { &MSVCRT_file_cs_debug, -1, 0, 0, 0, 0 };
// #define LOCK_FILES()    do { EnterCriticalSection(&MSVCRT_file_cs); } while (0)
// #define UNLOCK_FILES()  do { LeaveCriticalSection(&MSVCRT_file_cs); } while (0)
//
// static void msvcrt_stat64_to_stat(const struct _stat64 *buf64, struct _stat *buf)
// {
//     buf->st_dev   = buf64->st_dev;
//     buf->st_ino   = buf64->st_ino;
//     buf->st_mode  = buf64->st_mode;
//     buf->st_nlink = buf64->st_nlink;
//     buf->st_uid   = buf64->st_uid;
//     buf->st_gid   = buf64->st_gid;
//     buf->st_rdev  = buf64->st_rdev;
//     buf->st_size  = buf64->st_size;
//     buf->st_atime = buf64->st_atime;
//     buf->st_mtime = buf64->st_mtime;
//     buf->st_ctime = buf64->st_ctime;
// }
//
// static void msvcrt_stat64_to_stati64(const struct _stat64 *buf64, struct _stati64 *buf)
// {
//     buf->st_dev   = buf64->st_dev;
//     buf->st_ino   = buf64->st_ino;
//     buf->st_mode  = buf64->st_mode;
//     buf->st_nlink = buf64->st_nlink;
//     buf->st_uid   = buf64->st_uid;
//     buf->st_gid   = buf64->st_gid;
//     buf->st_rdev  = buf64->st_rdev;
//     buf->st_size  = buf64->st_size;
//     buf->st_atime = buf64->st_atime;
//     buf->st_mtime = buf64->st_mtime;
//     buf->st_ctime = buf64->st_ctime;
// }
//
// static void msvcrt_stat64_to_stat32(const struct _stat64 *buf64, struct _stat32 *buf)
// {
//     buf->st_dev   = buf64->st_dev;
//     buf->st_ino   = buf64->st_ino;
//     buf->st_mode  = buf64->st_mode;
//     buf->st_nlink = buf64->st_nlink;
//     buf->st_uid   = buf64->st_uid;
//     buf->st_gid   = buf64->st_gid;
//     buf->st_rdev  = buf64->st_rdev;
//     buf->st_size  = buf64->st_size;
//     buf->st_atime = buf64->st_atime;
//     buf->st_mtime = buf64->st_mtime;
//     buf->st_ctime = buf64->st_ctime;
// }
//
// static void msvcrt_stat64_to_stat64i32(const struct _stat64 *buf64, struct _stat64i32 *buf)
// {
//     buf->st_dev   = buf64->st_dev;
//     buf->st_ino   = buf64->st_ino;
//     buf->st_mode  = buf64->st_mode;
//     buf->st_nlink = buf64->st_nlink;
//     buf->st_uid   = buf64->st_uid;
//     buf->st_gid   = buf64->st_gid;
//     buf->st_rdev  = buf64->st_rdev;
//     buf->st_size  = buf64->st_size;
//     buf->st_atime = buf64->st_atime;
//     buf->st_mtime = buf64->st_mtime;
//     buf->st_ctime = buf64->st_ctime;
// }
//
// static void msvcrt_stat64_to_stat32i64(const struct _stat64 *buf64, struct _stat32i64 *buf)
// {
//     buf->st_dev   = buf64->st_dev;
//     buf->st_ino   = buf64->st_ino;
//     buf->st_mode  = buf64->st_mode;
//     buf->st_nlink = buf64->st_nlink;
//     buf->st_uid   = buf64->st_uid;
//     buf->st_gid   = buf64->st_gid;
//     buf->st_rdev  = buf64->st_rdev;
//     buf->st_size  = buf64->st_size;
//     buf->st_atime = buf64->st_atime;
//     buf->st_mtime = buf64->st_mtime;
//     buf->st_ctime = buf64->st_ctime;
// }
//
// static void time_to_filetime( __time64_t time, FILETIME *ft )
// {
//     /* 1601 to 1970 is 369 years plus 89 leap days */
//     static const __int64 secs_1601_to_1970 = ((369 * 365 + 89) * (__int64)86400);
//
//     __int64 ticks = (time + secs_1601_to_1970) * 10000000;
//     ft->dwHighDateTime = ticks >> 32;
//     ft->dwLowDateTime = ticks;
// }
//
// static inline ioinfo* get_ioinfo_nolock(int fd)
// {
//     ioinfo *ret = NULL;
//     if(fd>=0 && fd<MSVCRT_MAX_FILES)
//         ret = MSVCRT___pioinfo[fd/MSVCRT_FD_BLOCK_SIZE];
//     if(!ret)
//         return &MSVCRT___badioinfo;
//
//     return ret + (fd%MSVCRT_FD_BLOCK_SIZE);
// }
//
// static inline void init_ioinfo_cs(ioinfo *info)
// {
//     if(!ioinfo_is_crit_init(info)) {
//         LOCK_FILES();
//         if(!ioinfo_is_crit_init(info)) {
//             InitializeCriticalSection(&info->crit);
//             ioinfo_set_crit_init(info);
//         }
//         UNLOCK_FILES();
//     }
// }
//
// static inline ioinfo* get_ioinfo(int fd)
// {
//     ioinfo *ret = get_ioinfo_nolock(fd);
//     if(ret == &MSVCRT___badioinfo)
//         return ret;
//     init_ioinfo_cs(ret);
//     EnterCriticalSection(&ret->crit);
//     return ret;
// }
//
// static inline BOOL alloc_pioinfo_block(int fd)
// {
//     ioinfo *block;
//     int i;
//
//     if(fd<0 || fd>=MSVCRT_MAX_FILES)
//     {
//         *_errno() = ENFILE;
//         return FALSE;
//     }
//
//     block = calloc(MSVCRT_FD_BLOCK_SIZE, sizeof(ioinfo));
//     if(!block)
//     {
//         WARN(":out of memory!\n");
//         *_errno() = ENOMEM;
//         return FALSE;
//     }
//     for(i=0; i<MSVCRT_FD_BLOCK_SIZE; i++)
//     {
//         block[i].handle = INVALID_HANDLE_VALUE;
//         if (ioinfo_is_crit_init(&block[i]))
//         {
//             /* Initialize crit section on block allocation for _MSVC_VER >= 140,
//              * ioinfo_is_crit_init() is always TRUE. */
//             InitializeCriticalSection(&block[i].crit);
//         }
//     }
//     if(InterlockedCompareExchangePointer((void**)&MSVCRT___pioinfo[fd/MSVCRT_FD_BLOCK_SIZE], block, NULL))
//     {
//         if (ioinfo_is_crit_init(&block[0]))
//         {
//             for(i = 0; i < MSVCRT_FD_BLOCK_SIZE; ++i)
//                 DeleteCriticalSection(&block[i].crit);
//         }
//         free(block);
//     }
//     return TRUE;
// }
//
// static inline ioinfo* get_ioinfo_alloc_fd(int fd)
// {
//     ioinfo *ret;
//
//     ret = get_ioinfo(fd);
//     if(ret != &MSVCRT___badioinfo)
//         return ret;
//
//     if(!alloc_pioinfo_block(fd))
//         return &MSVCRT___badioinfo;
//
//     return get_ioinfo(fd);
// }
//
// static inline ioinfo* get_ioinfo_alloc(int *fd)
// {
//     int i;
//
//     *fd = -1;
//     for(i=0; i<MSVCRT_MAX_FILES; i++)
//     {
//         ioinfo *info = get_ioinfo_nolock(i);
//
//         if(info == &MSVCRT___badioinfo)
//         {
//             if(!alloc_pioinfo_block(i))
//                 return &MSVCRT___badioinfo;
//             info = get_ioinfo_nolock(i);
//         }
//
//         init_ioinfo_cs(info);
//         if(TryEnterCriticalSection(&info->crit))
//         {
//             if(info->handle == INVALID_HANDLE_VALUE)
//             {
//                 *fd = i;
//                 return info;
//             }
//             LeaveCriticalSection(&info->crit);
//         }
//     }
//
//     WARN(":files exhausted!\n");
//     *_errno() = ENFILE;
//     return &MSVCRT___badioinfo;
// }
//
// static inline void release_ioinfo(ioinfo *info)
// {
//     if(info!=&MSVCRT___badioinfo && ioinfo_is_crit_init(info))
//         LeaveCriticalSection(&info->crit);
// }
//
// static inline FILE* msvcrt_get_file(int i)
// {
//     file_crit *ret;
//
//     if(i >= MSVCRT_max_streams)
//         return NULL;
//
//     if(i < _IOB_ENTRIES)
//         return iob_get_file(i);
//
//     ret = MSVCRT_fstream[i/MSVCRT_FD_BLOCK_SIZE];
//     if(!ret) {
//         MSVCRT_fstream[i/MSVCRT_FD_BLOCK_SIZE] = calloc(MSVCRT_FD_BLOCK_SIZE, sizeof(file_crit));
//         if(!MSVCRT_fstream[i/MSVCRT_FD_BLOCK_SIZE]) {
//             ERR("out of memory\n");
//             *_errno() = ENOMEM;
//             return NULL;
//         }
//
//         ret = MSVCRT_fstream[i/MSVCRT_FD_BLOCK_SIZE] + (i%MSVCRT_FD_BLOCK_SIZE);
//     } else
//         ret += i%MSVCRT_FD_BLOCK_SIZE;
//
//     return &ret->file;
// }
//
// /* INTERNAL: free a file entry fd */
// static void msvcrt_free_fd(int fd)
// {
//   ioinfo *fdinfo = get_ioinfo(fd);
//
//   if(fdinfo != &MSVCRT___badioinfo)
//   {
//     fdinfo->handle = INVALID_HANDLE_VALUE;
//     fdinfo->wxflag = 0;
//   }
//   TRACE(":fd (%d) freed\n",fd);
//
//   if (fd < 3)
//   {
//     switch (fd)
//     {
//     case 0:
//         SetStdHandle(STD_INPUT_HANDLE, core::ptr::null_mut());
//         break;
//     case 1:
//         SetStdHandle(STD_OUTPUT_HANDLE, core::ptr::null_mut());
//         break;
//     case 2:
//         SetStdHandle(STD_ERROR_HANDLE, core::ptr::null_mut());
//         break;
//     }
//   }
//   release_ioinfo(fdinfo);
// }
//
// static void msvcrt_set_fd(ioinfo *fdinfo, HANDLE hand, int flag)
// {
//   fdinfo->handle = hand;
//   fdinfo->wxflag = WX_OPEN | (flag & (WX_DONTINHERIT | WX_APPEND | WX_TEXT | WX_PIPE | WX_TTY));
//   fdinfo->lookahead[0] = '\n';
//   fdinfo->lookahead[1] = '\n';
//   fdinfo->lookahead[2] = '\n';
//   ioinfo_set_unicode(fdinfo, FALSE);
//   ioinfo_set_textmode(fdinfo, TEXTMODE_ANSI);
//
//   if (hand != MSVCRT_NO_CONSOLE)
//   {
//     switch (fdinfo-MSVCRT___pioinfo[0])
//     {
//     case 0: SetStdHandle(STD_INPUT_HANDLE,  hand); break;
//     case 1: SetStdHandle(STD_OUTPUT_HANDLE, hand); break;
//     case 2: SetStdHandle(STD_ERROR_HANDLE,  hand); break;
//     }
//   }
// }
//
// /* INTERNAL: Allocate an fd slot from a Win32 HANDLE */
// static int msvcrt_alloc_fd(HANDLE hand, int flag)
// {
//     int fd;
//     ioinfo *info = get_ioinfo_alloc(&fd);
//
//     TRACE(":handle (%p) allocating fd (%d)\n", hand, fd);
//
//     if(info == &MSVCRT___badioinfo)
//         return -1;
//
//     msvcrt_set_fd(info, hand, flag);
//     release_ioinfo(info);
//     return fd;
// }
//
// /* INTERNAL: Allocate a FILE* for an fd slot */
// /* caller must hold the files lock */
// static FILE* msvcrt_alloc_fp(void)
// {
//   int i = 0;
//   FILE *file;
//
//   for (; i < MSVCRT_max_streams; i++)
//   {
//     file = msvcrt_get_file(i);
//     if (!file)
//       return NULL;
//
//     if (file->_flag == 0)
//     {
//       if (i == MSVCRT_stream_idx)
//       {
//           CRITICAL_SECTION *cs = file_get_cs(file);
//           if (cs)
//           {
//               InitializeCriticalSectionEx(cs, 0, RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO);
//               cs->DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": file_crit.crit");
//           }
//           MSVCRT_stream_idx++;
//       }
//       return file;
//     }
//   }
//
//   return NULL;
// }
//
// /* INTERNAL: initialize a FILE* from an open fd */
// static int msvcrt_init_fp(FILE* file, int fd, unsigned stream_flags)
// {
//   TRACE(":fd (%d) allocating FILE*\n",fd);
//   if (!(get_ioinfo_nolock(fd)->wxflag & WX_OPEN))
//   {
//     WARN(":invalid fd %d\n",fd);
//     *__doserrno() = 0;
//     *_errno() = EBADF;
//     return -1;
//   }
//   file->_ptr = file->_base = NULL;
//   file->_cnt = 0;
//   file->_file = fd;
//   file->_flag = stream_flags;
//   file->_tmpfname = NULL;
//
//   TRACE(":got FILE* (%p)\n",file);
//   return 0;
// }
//
// /* INTERNAL: Create an inheritance data block (for spawned process)
//  * The inheritance block is made of:
//  *      00      int     nb of file descriptor (NBFD)
//  *      04      char    file flags (wxflag): repeated for each fd
//  *      4+NBFD  HANDLE  file handle: repeated for each fd
//  */
// BOOL msvcrt_create_io_inherit_block(WORD *size, BYTE **block)
// {
//   int         fd, last_fd;
//   char*       wxflag_ptr;
//   HANDLE*     handle_ptr;
//   ioinfo*     fdinfo;
//
//   for (last_fd=MSVCRT_MAX_FILES-1; last_fd>=0; last_fd--)
//     if (get_ioinfo_nolock(last_fd)->handle != INVALID_HANDLE_VALUE)
//       break;
//   last_fd++;
//
//   *size = sizeof(unsigned) + (sizeof(char) + sizeof(HANDLE)) * last_fd;
//   *block = calloc(1, *size);
//   if (!*block)
//   {
//     *size = 0;
//     return FALSE;
//   }
//   wxflag_ptr = (char*)*block + sizeof(unsigned);
//   handle_ptr = (HANDLE*)(wxflag_ptr + last_fd);
//
//   *(unsigned*)*block = last_fd;
//   for (fd = 0; fd < last_fd; fd++)
//   {
//     /* to be inherited, we need it to be open, and that DONTINHERIT isn't set */
//     fdinfo = get_ioinfo_nolock(fd);
//     if ((fdinfo->wxflag & (WX_OPEN | WX_DONTINHERIT)) == WX_OPEN)
//     {
//       *wxflag_ptr = fdinfo->wxflag;
//       *handle_ptr = fdinfo->handle;
//     }
//     else
//     {
//       *wxflag_ptr = 0;
//       *handle_ptr = INVALID_HANDLE_VALUE;
//     }
//     wxflag_ptr++; handle_ptr++;
//   }
//   return TRUE;
// }
//
// /* INTERNAL: Set up all file descriptors,
//  * as well as default streams (stdin, stderr and stdout)
//  */
// void msvcrt_init_io(void)
// {
//   STARTUPINFOA  si;
//   int           i;
//   ioinfo        *fdinfo;
//
//   GetStartupInfoA(&si);
//   if (si.cbReserved2 >= sizeof(unsigned int) && si.lpReserved2 != NULL)
//   {
//     BYTE*       wxflag_ptr;
//     HANDLE*     handle_ptr;
//     unsigned int count;
//
//     count = *(unsigned*)si.lpReserved2;
//     wxflag_ptr = si.lpReserved2 + sizeof(unsigned);
//     handle_ptr = (HANDLE*)(wxflag_ptr + count);
//
//     count = min(count, (si.cbReserved2 - sizeof(unsigned)) / (sizeof(HANDLE) + 1));
//     count = min(count, MSVCRT_MAX_FILES);
//     for (i = 0; i < count; i++)
//     {
//       if ((*wxflag_ptr & WX_OPEN) && GetFileType(*handle_ptr) != FILE_TYPE_UNKNOWN)
//       {
//         fdinfo = get_ioinfo_alloc_fd(i);
//         if(fdinfo != &MSVCRT___badioinfo)
//             msvcrt_set_fd(fdinfo, *handle_ptr, *wxflag_ptr);
//         release_ioinfo(fdinfo);
//       }
//
//       wxflag_ptr++; handle_ptr++;
//     }
//   }
//
//   fdinfo = get_ioinfo_alloc_fd(STDIN_FILENO);
//   if (!(fdinfo->wxflag & WX_OPEN) || fdinfo->handle == INVALID_HANDLE_VALUE) {
//     HANDLE h = GetStdHandle(STD_INPUT_HANDLE);
//     DWORD flags = WX_OPEN | WX_TEXT;
//     DWORD type = GetFileType(h);
//
//     if (type == FILE_TYPE_UNKNOWN) {
//         h = MSVCRT_NO_CONSOLE;
//         flags |= WX_TTY;
//     } else if ((type & 0xf) == FILE_TYPE_CHAR) {
//         flags |= WX_TTY;
//     } else if ((type & 0xf) == FILE_TYPE_PIPE) {
//         flags |= WX_PIPE;
//     }
//
//     msvcrt_set_fd(fdinfo, h, flags);
//   }
//   release_ioinfo(fdinfo);
//
//   fdinfo = get_ioinfo_alloc_fd(STDOUT_FILENO);
//   if (!(fdinfo->wxflag & WX_OPEN) || fdinfo->handle == INVALID_HANDLE_VALUE) {
//     HANDLE h = GetStdHandle(STD_OUTPUT_HANDLE);
//     DWORD flags = WX_OPEN | WX_TEXT;
//     DWORD type = GetFileType(h);
//
//     if (type == FILE_TYPE_UNKNOWN) {
//         h = MSVCRT_NO_CONSOLE;
//         flags |= WX_TTY;
//     } else if ((type & 0xf) == FILE_TYPE_CHAR) {
//         flags |= WX_TTY;
//     } else if ((type & 0xf) == FILE_TYPE_PIPE) {
//         flags |= WX_PIPE;
//     }
//
//     msvcrt_set_fd(fdinfo, h, flags);
//   }
//   release_ioinfo(fdinfo);
//
//   fdinfo = get_ioinfo_alloc_fd(STDERR_FILENO);
//   if (!(fdinfo->wxflag & WX_OPEN) || fdinfo->handle == INVALID_HANDLE_VALUE) {
//     HANDLE h = GetStdHandle(STD_ERROR_HANDLE);
//     DWORD flags = WX_OPEN | WX_TEXT;
//     DWORD type = GetFileType(h);
//
//     if (type == FILE_TYPE_UNKNOWN) {
//         h = MSVCRT_NO_CONSOLE;
//         flags |= WX_TTY;
//     } else if ((type & 0xf) == FILE_TYPE_CHAR) {
//         flags |= WX_TTY;
//     } else if ((type & 0xf) == FILE_TYPE_PIPE) {
//         flags |= WX_PIPE;
//     }
//
//     msvcrt_set_fd(fdinfo, h, flags);
//   }
//   release_ioinfo(fdinfo);
//
//   TRACE(":handles (%p)(%p)(%p)\n", get_ioinfo_nolock(STDIN_FILENO)->handle,
//         get_ioinfo_nolock(STDOUT_FILENO)->handle,
//         get_ioinfo_nolock(STDERR_FILENO)->handle);
//
//   for (i = 0; i < 3; i++)
//   {
//     FILE *f = iob_get_file(i);
//     CRITICAL_SECTION *cs = file_get_cs(f);
//
//     /* FILE structs for stdin/out/err are static and never deleted */
//     f->_file = get_ioinfo_nolock(i)->handle == MSVCRT_NO_CONSOLE ?
//         MSVCRT_NO_CONSOLE_FD : i;
//     f->_tmpfname = NULL;
//     f->_flag = (i == 0) ? _IOREAD : _IOWRT;
//
//     if (cs)
//     {
//       InitializeCriticalSectionEx(cs, 0, RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO);
//       cs->DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": file_crit.crit");
//     }
//   }
//   MSVCRT_stream_idx = 3;
// }
//
// /* INTERNAL: Flush stdio file buffer */
// static int msvcrt_flush_buffer(FILE* file)
// {
//     int ret = 0;
//
//     if((file->_flag & (_IOREAD|_IOWRT)) == _IOWRT &&
//             file->_flag & (_IOMYBUF|MSVCRT__USERBUF)) {
//         int cnt=file->_ptr-file->_base;
//         if(cnt>0 && _write(file->_file, file->_base, cnt) != cnt) {
//             file->_flag |= _IOERR;
//             ret = EOF;
//         } else if(file->_flag & _IORW) {
//             file->_flag &= ~_IOWRT;
//         }
//     }
//
//     file->_ptr=file->_base;
//     file->_cnt=0;
//     return ret;
// }
//
// /*********************************************************************
//  *		_isatty (MSVCRT.@)
//  */
// int CDECL _isatty(int fd)
// {
//     TRACE(":fd (%d)\n",fd);
//
//     return get_ioinfo_nolock(fd)->wxflag & WX_TTY;
// }
//
// /* INTERNAL: Allocate stdio file buffer */
// static BOOL msvcrt_alloc_buffer(FILE* file)
// {
//     if((file->_file==STDOUT_FILENO || file->_file==STDERR_FILENO)
//             && _isatty(file->_file))
//         return FALSE;
//
//     file->_base = calloc(1, MSVCRT_INTERNAL_BUFSIZ);
//     if(file->_base) {
//         file->_bufsiz = MSVCRT_INTERNAL_BUFSIZ;
//         file->_flag |= _IOMYBUF;
//     } else {
//         file->_base = (char*)(&file->_charbuf);
//         file->_bufsiz = 2;
//         file->_flag |= MSVCRT__NOBUF;
//     }
//     file->_ptr = file->_base;
//     file->_cnt = 0;
//     return TRUE;
// }
//
// /* INTERNAL: Allocate temporary buffer for stdout and stderr */
// static BOOL add_std_buffer(FILE *file)
// {
//     static char buffers[2][BUFSIZ];
//
//     if((file->_file!=STDOUT_FILENO && file->_file!=STDERR_FILENO)
//             || (file->_flag & (MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF))
//             || !_isatty(file->_file))
//         return FALSE;
//
//     file->_ptr = file->_base = buffers[file->_file == STDOUT_FILENO ? 0 : 1];
//     file->_bufsiz = file->_cnt = BUFSIZ;
//     file->_flag |= MSVCRT__USERBUF;
//     return TRUE;
// }
//
// /* INTERNAL: Removes temporary buffer from stdout or stderr */
// /* Only call this function when add_std_buffer returned TRUE */
// static void remove_std_buffer(FILE *file)
// {
//     msvcrt_flush_buffer(file);
//     file->_ptr = file->_base = NULL;
//     file->_bufsiz = file->_cnt = 0;
//     file->_flag &= ~MSVCRT__USERBUF;
// }
//
// /* INTERNAL: Convert integer to base32 string (0-9a-v), 0 becomes "" */
// static int msvcrt_int_to_base32(int num, char *str)
// {
//   char *p;
//   int n = num;
//   int digits = 0;
//
//   while (n != 0)
//   {
//     n >>= 5;
//     digits++;
//   }
//   p = str + digits;
//   *p = 0;
//   while (--p >= str)
//   {
//     *p = (num & 31) + '0';
//     if (*p > '9')
//       *p += ('a' - '0' - 10);
//     num >>= 5;
//   }
//
//   return digits;
// }
//
// /* INTERNAL: wide character version of msvcrt_int_to_base32 */
// static int msvcrt_int_to_base32_w(int num, wchar_t *str)
// {
//     wchar_t *p;
//     int n = num;
//     int digits = 0;
//
//     while (n != 0)
//     {
//         n >>= 5;
//         digits++;
//     }
//     p = str + digits;
//     *p = 0;
//     while (--p >= str)
//     {
//         *p = (num & 31) + '0';
//         if (*p > '9')
//             *p += ('a' - '0' - 10);
//         num >>= 5;
//     }
//
//     return digits;
// }
//
// /*********************************************************************
//  *		__iob_func  (MSVCRT.@)
//  */
// #undef __iob_func
// FILE *  __iob_func(void)
// {
//     return iob_get_file(0);
// }
//
//
// /*********************************************************************
//  *		_access (MSVCRT.@)
//  */
// int CDECL _access(const char *filename, int mode)
// {
//     wchar_t *filenameW = NULL;
//     int ret;
//
//     if (filename && !(filenameW = wstrdupa_utf8(filename))) return -1;
//     ret = _waccess(filenameW, mode);
//     free(filenameW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_access_s (MSVCRT.@)
//  */
// int CDECL _access_s(const char *filename, int mode)
// {
//   if (!MSVCRT_CHECK_PMT(filename != NULL)) return *_errno();
//   if (!MSVCRT_CHECK_PMT((mode & ~(MSVCRT_R_OK | MSVCRT_W_OK)) == 0)) return *_errno();
//
//   if (_access(filename, mode) == -1)
//     return *_errno();
//   return 0;
// }
//
// /*********************************************************************
//  *		_waccess (MSVCRT.@)
//  */
// int CDECL _waccess(const wchar_t *filename, int mode)
// {
//   DWORD attr = GetFileAttributesW(filename);
//
//   TRACE("(%s,%d) %ld\n", debugstr_w(filename), mode, attr);
//
//   if (!filename || attr == INVALID_FILE_ATTRIBUTES)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   if ((attr & FILE_ATTRIBUTE_READONLY) && (mode & MSVCRT_W_OK))
//   {
//     msvcrt_set_errno(ERROR_ACCESS_DENIED);
//     return -1;
//   }
//   return 0;
// }
//
// /*********************************************************************
//  *		_waccess_s (MSVCRT.@)
//  */
// int CDECL _waccess_s(const wchar_t *filename, int mode)
// {
//   if (!MSVCRT_CHECK_PMT(filename != NULL)) return *_errno();
//   if (!MSVCRT_CHECK_PMT((mode & ~(MSVCRT_R_OK | MSVCRT_W_OK)) == 0)) return *_errno();
//
//   if (_waccess(filename, mode) == -1)
//     return *_errno();
//   return 0;
// }
//
// /*********************************************************************
//  *		_chmod (MSVCRT.@)
//  */
// int CDECL _chmod(const char *path, int flags)
// {
//     wchar_t *pathW = NULL;
//     int ret;
//
//     if (path && !(pathW = wstrdupa_utf8(path))) return -1;
//     ret = _wchmod(pathW, flags);
//     free(pathW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wchmod (MSVCRT.@)
//  */
// int CDECL _wchmod(const wchar_t *path, int flags)
// {
//   DWORD oldFlags = GetFileAttributesW(path);
//
//   if (oldFlags != INVALID_FILE_ATTRIBUTES)
//   {
//     DWORD newFlags = (flags & _S_IWRITE)? oldFlags & ~FILE_ATTRIBUTE_READONLY:
//       oldFlags | FILE_ATTRIBUTE_READONLY;
//
//     if (newFlags == oldFlags || SetFileAttributesW(path, newFlags))
//       return 0;
//   }
//   msvcrt_set_errno(GetLastError());
//   return -1;
// }
//
// /*********************************************************************
//  *		_unlink (MSVCRT.@)
//  */
// int CDECL _unlink(const char *path)
// {
//     wchar_t *pathW = NULL;
//     int ret;
//
//     if (path && !(pathW = wstrdupa_utf8(path))) return -1;
//     ret = _wunlink(pathW);
//     free(pathW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wunlink (MSVCRT.@)
//  */
// int CDECL _wunlink(const wchar_t *path)
// {
//   TRACE("(%s)\n", debugstr_w(path));
//   if(DeleteFileW(path))
//     return 0;
//   TRACE("failed (%ld)\n", GetLastError());
//   msvcrt_set_errno(GetLastError());
//   return -1;
// }
//
// /*********************************************************************
//  *		_commit (MSVCRT.@)
//  */
// int CDECL _commit(int fd)
// {
//     ioinfo *info = get_ioinfo(fd);
//     int ret;
//
//     TRACE(":fd (%d) handle (%p)\n", fd, info->handle);
//
//     if (info->handle == INVALID_HANDLE_VALUE)
//         ret = -1;
//     else if (!FlushFileBuffers(info->handle))
//     {
//         if (GetLastError() == ERROR_INVALID_HANDLE)
//         {
//             /* FlushFileBuffers fails for console handles
//              * so we ignore this error.
//              */
//             ret = 0;
//         }
//         else
//         {
//             TRACE(":failed-last error (%ld)\n", GetLastError());
//             msvcrt_set_errno(GetLastError());
//             ret = -1;
//         }
//     }
//     else
//     {
//         TRACE(":ok\n");
//         ret = 0;
//     }
//
//     release_ioinfo(info);
//     return ret;
// }
//
// /* INTERNAL: Flush all stream buffer */
// static int msvcrt_flush_all_buffers(int mask)
// {
//   int i, num_flushed = 0;
//   FILE *file;
//
//   LOCK_FILES();
//   for (i = 0; i < MSVCRT_stream_idx; i++) {
//     file = msvcrt_get_file(i);
//
//     if (file->_flag)
//     {
//       if(file->_flag & mask) {
// 	fflush(file);
//         num_flushed++;
//       }
//     }
//   }
//   UNLOCK_FILES();
//
//   TRACE(":flushed (%d) handles\n",num_flushed);
//   return num_flushed;
// }
//
// /*********************************************************************
//  *		_flushall (MSVCRT.@)
//  */
// int CDECL _flushall(void)
// {
//     return msvcrt_flush_all_buffers(_IOWRT | _IOREAD);
// }
//
// /*********************************************************************
//  *		fflush (MSVCRT.@)
//  */
// int CDECL fflush(FILE* file)
// {
//     int ret;
//
//     if(!file) {
//         msvcrt_flush_all_buffers(_IOWRT);
//         ret = 0;
//     } else {
//         _lock_file(file);
//         ret = _fflush_nolock(file);
//         _unlock_file(file);
//     }
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fflush_nolock (MSVCRT.@)
//  */
// int CDECL _fflush_nolock(FILE* file)
// {
//     int res;
//
//     if(!file) {
//         msvcrt_flush_all_buffers(_IOWRT);
//         return 0;
//     }
//
//     res = msvcrt_flush_buffer(file);
//     if(!res && (file->_flag & MSVCRT__IOCOMMIT))
//         res = _commit(file->_file) ? EOF : 0;
//     return res;
// }
//
// /*********************************************************************
//  *		_close (MSVCRT.@)
//  */
// int CDECL _close(int fd)
// {
//   ioinfo *info = get_ioinfo(fd);
//   int ret;
//
//   TRACE(":fd (%d) handle (%p)\n", fd, info->handle);
//
//   if (fd == MSVCRT_NO_CONSOLE_FD) {
//     *_errno() = EBADF;
//     ret = -1;
//   } else if (!MSVCRT_CHECK_PMT_ERR(info->wxflag & WX_OPEN, EBADF)) {
//     ret = -1;
//   } else if (fd == STDOUT_FILENO &&
//           info->handle == get_ioinfo_nolock(STDERR_FILENO)->handle) {
//     msvcrt_free_fd(fd);
//     ret = 0;
//   } else if (fd == STDERR_FILENO &&
//           info->handle == get_ioinfo_nolock(STDOUT_FILENO)->handle) {
//     msvcrt_free_fd(fd);
//     ret = 0;
//   } else {
//     ret = CloseHandle(info->handle) ? 0 : -1;
//     msvcrt_free_fd(fd);
//     if (ret) {
//       WARN(":failed-last error (%ld)\n", GetLastError());
//       msvcrt_set_errno(GetLastError());
//     }
//   }
//   release_ioinfo(info);
//   return ret;
// }
//
// /*********************************************************************
//  *		_dup2 (MSVCRT.@)
//  * NOTES
//  * MSDN isn't clear on this point, but the remarks for _pipe
//  * indicate file descriptors duplicated with _dup and _dup2 are always
//  * inheritable.
//  */
// int CDECL _dup2(int od, int nd)
// {
//   ioinfo *info_od, *info_nd;
//   int ret;
//
//   TRACE("(od=%d, nd=%d)\n", od, nd);
//
//   if (od < nd)
//   {
//     info_od = get_ioinfo(od);
//     info_nd = get_ioinfo_alloc_fd(nd);
//   }
//   else
//   {
//     info_nd = get_ioinfo_alloc_fd(nd);
//     info_od = get_ioinfo(od);
//   }
//
//   if (info_nd == &MSVCRT___badioinfo)
//   {
//       *_errno() = EBADF;
//       ret = -1;
//   }
//   else if (info_od->wxflag & WX_OPEN)
//   {
//     HANDLE handle;
//
//     if (DuplicateHandle(GetCurrentProcess(), info_od->handle,
//      GetCurrentProcess(), &handle, 0, TRUE, DUPLICATE_SAME_ACCESS))
//     {
//       int wxflag = info_od->wxflag & ~WX_DONTINHERIT;
//
//       if (info_nd->wxflag & WX_OPEN)
//         _close(nd);
//
//       msvcrt_set_fd(info_nd, handle, wxflag);
//       /* _dup2 returns 0, not nd, on success */
//       ret = 0;
//     }
//     else
//     {
//       ret = -1;
//       msvcrt_set_errno(GetLastError());
//     }
//   }
//   else
//   {
//     *_errno() = EBADF;
//     ret = -1;
//   }
//
//   release_ioinfo(info_od);
//   release_ioinfo(info_nd);
//   return ret;
// }
//
// /*********************************************************************
//  *		_dup (MSVCRT.@)
//  */
// int CDECL _dup(int od)
// {
//   int fd, ret;
//   ioinfo *info = get_ioinfo_alloc(&fd);
//
//   if (_dup2(od, fd) == 0)
//     ret = fd;
//   else
//     ret = -1;
//   release_ioinfo(info);
//   return ret;
// }
//
// /*********************************************************************
//  *		_eof (MSVCRT.@)
//  */
// int CDECL _eof(int fd)
// {
//   ioinfo *info = get_ioinfo(fd);
//   DWORD curpos,endpos;
//   LONG hcurpos,hendpos;
//
//   TRACE(":fd (%d) handle (%p)\n", fd, info->handle);
//
//   if (info->handle == INVALID_HANDLE_VALUE)
//   {
//     release_ioinfo(info);
//     return -1;
//   }
//
//   if (info->wxflag & WX_ATEOF)
//   {
//       release_ioinfo(info);
//       return TRUE;
//   }
//
//   /* Otherwise we do it the hard way */
//   hcurpos = hendpos = 0;
//   curpos = SetFilePointer(info->handle, 0, &hcurpos, FILE_CURRENT);
//   endpos = SetFilePointer(info->handle, 0, &hendpos, FILE_END);
//
//   if (curpos == endpos && hcurpos == hendpos)
//   {
//     /* FIXME: shouldn't WX_ATEOF be set here? */
//     release_ioinfo(info);
//     return TRUE;
//   }
//
//   SetFilePointer(info->handle, curpos, &hcurpos, FILE_BEGIN);
//   release_ioinfo(info);
//   return FALSE;
// }
//
// /*********************************************************************
//  *		_fcloseall (MSVCRT.@)
//  */
// int CDECL _fcloseall(void)
// {
//   int num_closed = 0, i;
//   FILE *file;
//
//   LOCK_FILES();
//   for (i = 3; i < MSVCRT_stream_idx; i++) {
//     file = msvcrt_get_file(i);
//
//     if (file->_flag && !fclose(file))
//       num_closed++;
//   }
//   UNLOCK_FILES();
//
//   TRACE(":closed (%d) handles\n",num_closed);
//   return num_closed;
// }
//
// /* free everything on process exit */
// void msvcrt_free_io(void)
// {
//     unsigned int i;
//     int j;
//
//     _flushall();
//     _fcloseall();
//
//     for(i=0; i<ARRAY_SIZE(MSVCRT___pioinfo); i++)
//     {
//         if(!MSVCRT___pioinfo[i])
//             continue;
//
//         for(j=0; j<MSVCRT_FD_BLOCK_SIZE; j++)
//         {
//             if(ioinfo_is_crit_init(&MSVCRT___pioinfo[i][j]))
//                 DeleteCriticalSection(&MSVCRT___pioinfo[i][j].crit);
//         }
//         free(MSVCRT___pioinfo[i]);
//     }
//
//     for(j=0; j<MSVCRT_stream_idx; j++)
//     {
//         FILE *file = msvcrt_get_file(j);
//         CRITICAL_SECTION *cs = file_get_cs(file);
//
//         if(cs)
//         {
//             cs->DebugInfo->Spare[0] = 0;
//             DeleteCriticalSection(cs);
//         }
//     }
//
//     for(i=0; i<ARRAY_SIZE(MSVCRT_fstream); i++)
//         free(MSVCRT_fstream[i]);
// }
//
// /*********************************************************************
//  *		_lseeki64 (MSVCRT.@)
//  */
// __int64 CDECL _lseeki64(int fd, __int64 offset, int whence)
// {
//   ioinfo *info = get_ioinfo(fd);
//   LARGE_INTEGER ofs;
//
//   TRACE(":fd (%d) handle (%p)\n", fd, info->handle);
//
//   if (info->handle == INVALID_HANDLE_VALUE)
//   {
//     *_errno() = EBADF;
//     release_ioinfo(info);
//     return -1;
//   }
//
//   if (whence < 0 || whence > 2)
//   {
//     release_ioinfo(info);
//     *_errno() = EINVAL;
//     return -1;
//   }
//
//   TRACE(":fd (%d) to %#I64x pos %s\n",
//           fd, offset, (whence == SEEK_SET) ? "SEEK_SET" :
//           (whence == SEEK_CUR) ? "SEEK_CUR" :
//           (whence == SEEK_END) ? "SEEK_END" : "UNKNOWN");
//
//   /* The MoleBox protection scheme expects msvcrt to use SetFilePointer only,
//    * so a LARGE_INTEGER offset cannot be passed directly via SetFilePointerEx. */
//   ofs.QuadPart = offset;
//   if ((ofs.u.LowPart = SetFilePointer(info->handle, ofs.u.LowPart, &ofs.u.HighPart, whence)) != INVALID_SET_FILE_POINTER ||
//       GetLastError() == ERROR_SUCCESS)
//   {
//     info->wxflag &= ~WX_ATEOF;
//     /* FIXME: What if we seek _to_ EOF - is EOF set? */
//
//     release_ioinfo(info);
//     return ofs.QuadPart;
//   }
//   release_ioinfo(info);
//   TRACE(":error-last error (%ld)\n", GetLastError());
//   msvcrt_set_errno(GetLastError());
//   return -1;
// }
//
// /*********************************************************************
//  *		_lseek (MSVCRT.@)
//  */
// __msvcrt_long CDECL _lseek(int fd, __msvcrt_long offset, int whence)
// {
//     return _lseeki64(fd, offset, whence);
// }
//
// /*********************************************************************
//  *              _lock_file (MSVCRT.@)
//  */
// void CDECL _lock_file(FILE *file)
// {
//     CRITICAL_SECTION *cs = file_get_cs(file);
//     if (!cs)
//         _lock(_STREAM_LOCKS + (file - iob_get_file(0)));
//     else
//         EnterCriticalSection(cs);
// }
//
// /*********************************************************************
//  *              _unlock_file (MSVCRT.@)
//  */
// void CDECL _unlock_file(FILE *file)
// {
//     CRITICAL_SECTION *cs = file_get_cs(file);
//     if (!cs)
//         _unlock(_STREAM_LOCKS + (file - iob_get_file(0)));
//     else
//         LeaveCriticalSection(cs);
// }
//
// /*********************************************************************
//  *		_locking (MSVCRT.@)
//  *
//  * This is untested; the underlying LockFile doesn't work yet.
//  */
// int CDECL _locking(int fd, int mode, __msvcrt_long nbytes)
// {
//   ioinfo *info = get_ioinfo(fd);
//   BOOL ret;
//   DWORD cur_locn;
//
//   TRACE(":fd (%d) handle (%p)\n", fd, info->handle);
//   if (info->handle == INVALID_HANDLE_VALUE)
//   {
//     release_ioinfo(info);
//     return -1;
//   }
//
//   if (mode < 0 || mode > 4)
//   {
//     release_ioinfo(info);
//     *_errno() = EINVAL;
//     return -1;
//   }
//
//   TRACE(":fd (%d) by %#lx mode %s\n",
//           fd, nbytes, (mode == _LK_UNLCK) ? "_LK_UNLCK" :
//           (mode == _LK_LOCK) ? "_LK_LOCK" :
//           (mode == _LK_NBLCK) ? "_LK_NBLCK" :
//           (mode == _LK_RLCK) ? "_LK_RLCK" :
//           (mode == _LK_NBRLCK) ? "_LK_NBRLCK" :
//           "UNKNOWN");
//
//   if ((cur_locn = SetFilePointer(info->handle, 0, NULL, FILE_CURRENT)) == INVALID_SET_FILE_POINTER)
//   {
//     release_ioinfo(info);
//     FIXME("Seek failed\n");
//     *_errno() = EINVAL; /* FIXME */
//     return -1;
//   }
//   if (mode == _LK_LOCK || mode == _LK_RLCK)
//   {
//     int nretry = 10;
//     ret = 1; /* just to satisfy gcc */
//     while (nretry--)
//     {
//       ret = LockFile(info->handle, cur_locn, 0, nbytes, 0);
//       if (ret) break;
//       Sleep(1);
//     }
//   }
//   else if (mode == _LK_UNLCK)
//     ret = UnlockFile(info->handle, cur_locn, 0, nbytes, 0);
//   else
//     ret = LockFile(info->handle, cur_locn, 0, nbytes, 0);
//   /* FIXME - what about error settings? */
//   release_ioinfo(info);
//   return ret ? 0 : -1;
// }
//
// /*********************************************************************
//  *		_fseeki64 (MSVCRT.@)
//  */
// int CDECL _fseeki64(FILE* file, __int64 offset, int whence)
// {
//     int ret;
//
//     _lock_file(file);
//     ret = _fseeki64_nolock(file, offset, whence);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fseeki64_nolock (MSVCRT.@)
//  */
// int CDECL _fseeki64_nolock(FILE* file, __int64 offset, int whence)
// {
//   int ret;
//
//   if(whence == SEEK_CUR && file->_flag & _IOREAD ) {
//       whence = SEEK_SET;
//       offset += _ftelli64_nolock(file);
//   }
//
//   /* Flush output if needed */
//   msvcrt_flush_buffer(file);
//   /* Reset direction of i/o */
//   if(file->_flag & _IORW) {
//         file->_flag &= ~(_IOREAD|_IOWRT);
//   }
//   /* Clear end of file flag */
//   file->_flag &= ~_IOEOF;
//   ret = (_lseeki64(file->_file,offset,whence) == -1)?-1:0;
//
//   return ret;
// }
//
// /*********************************************************************
//  *		fseek (MSVCRT.@)
//  */
// int CDECL fseek(FILE* file, __msvcrt_long offset, int whence)
// {
//     return _fseeki64( file, offset, whence );
// }
//
// /*********************************************************************
//  *		_fseek_nolock (MSVCRT.@)
//  */
// int CDECL _fseek_nolock(FILE* file, __msvcrt_long offset, int whence)
// {
//     return _fseeki64_nolock( file, offset, whence );
// }
//
// /*********************************************************************
//  *		_chsize_s (MSVCRT.@)
//  */
// int CDECL _chsize_s(int fd, __int64 size)
// {
//     ioinfo *info;
//     __int64 cur, pos;
//     BOOL ret = FALSE;
//
//     TRACE("(fd=%d, size=%#I64x)\n", fd, size);
//
//     if (!MSVCRT_CHECK_PMT(size >= 0)) return EINVAL;
//
//
//     info = get_ioinfo(fd);
//     if (info->handle != INVALID_HANDLE_VALUE)
//     {
//         /* save the current file pointer */
//         cur = _lseeki64(fd, 0, SEEK_CUR);
//         if (cur >= 0)
//         {
//             pos = _lseeki64(fd, size, SEEK_SET);
//             if (pos >= 0)
//             {
//                 ret = SetEndOfFile(info->handle);
//                 if (!ret) msvcrt_set_errno(GetLastError());
//             }
//
//             /* restore the file pointer */
//             _lseeki64(fd, cur, SEEK_SET);
//         }
//     }
//
//     release_ioinfo(info);
//     return ret ? 0 : *_errno();
// }
//
// /*********************************************************************
//  *		_chsize (MSVCRT.@)
//  */
// int CDECL _chsize(int fd, __msvcrt_long size)
// {
//     /* _chsize_s returns errno on failure but _chsize should return -1 */
//     return _chsize_s( fd, size ) == 0 ? 0 : -1;
// }
//
// /*********************************************************************
//  *		clearerr (MSVCRT.@)
//  */
// void CDECL clearerr(FILE* file)
// {
//   TRACE(":file (%p) fd (%d)\n",file,file->_file);
//
//   _lock_file(file);
//   file->_flag &= ~(_IOERR | _IOEOF);
//   _unlock_file(file);
// }
//
// /*********************************************************************
//  *		clearerr_s (MSVCRT.@)
//  */
// int CDECL clearerr_s(FILE* file)
// {
//   TRACE(":file (%p)\n",file);
//
//   if (!MSVCRT_CHECK_PMT(file != NULL)) return EINVAL;
//
//   _lock_file(file);
//   file->_flag &= ~(_IOERR | _IOEOF);
//   _unlock_file(file);
//   return 0;
// }
//
// #if defined(__i386__)
// /* Stack preserving thunk for rewind
//  * needed for the UIO mod for Fallout: New Vegas
//  */
// __ASM_GLOBAL_FUNC(rewind_preserve_stack,
//                   "pushl 4(%esp)\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                   "call "__ASM_NAME("rewind") "\n\t"
//                   "addl $4,%esp\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset -4\n\t")
//                   "ret")
// #endif
//
// /*********************************************************************
//  *		rewind (MSVCRT.@)
//  */
// void CDECL rewind(FILE* file)
// {
//   TRACE(":file (%p) fd (%d)\n",file,file->_file);
//
//   _lock_file(file);
//   _fseek_nolock(file, 0, SEEK_SET);
//   clearerr(file);
//   _unlock_file(file);
// }
//
// static int msvcrt_get_flags(const wchar_t* mode, int *open_flags, int* stream_flags)
// {
//   int plus = wcschr(mode, '+') != NULL;
//
//   TRACE("%s\n", debugstr_w(mode));
//
//   while(*mode == ' ') mode++;
//
//   switch(*mode++)
//   {
//   case 'R': case 'r':
//     *open_flags = plus ? _O_RDWR : _O_RDONLY;
//     *stream_flags = plus ? _IORW : _IOREAD;
//     break;
//   case 'W': case 'w':
//     *open_flags = _O_CREAT | _O_TRUNC | (plus  ? _O_RDWR : _O_WRONLY);
//     *stream_flags = plus ? _IORW : _IOWRT;
//     break;
//   case 'A': case 'a':
//     *open_flags = _O_CREAT | _O_APPEND | (plus  ? _O_RDWR : _O_WRONLY);
//     *stream_flags = plus ? _IORW : _IOWRT;
//     break;
//   default:
//     MSVCRT_INVALID_PMT(0, EINVAL);
//     return -1;
//   }
//
//   *stream_flags |= MSVCRT__commode as i32;
//
//   while (*mode && *mode!=',')
//     switch (*mode++)
//     {
//     case 'B': case 'b':
//       *open_flags |=  _O_BINARY;
//       *open_flags &= ~_O_TEXT;
//       break;
//     case 't':
//       *open_flags |=  _O_TEXT;
//       *open_flags &= ~_O_BINARY;
//       break;
//     case 'D':
//       *open_flags |= _O_TEMPORARY;
//       break;
//     case 'T':
//       *open_flags |= _O_SHORT_LIVED;
//       break;
//     case 'c':
//       *stream_flags |= MSVCRT__IOCOMMIT;
//       break;
//     case 'n':
//       *stream_flags &= ~MSVCRT__IOCOMMIT;
//       break;
//     case 'N':
//       *open_flags |= _O_NOINHERIT;
//       break;
//     case '+':
//     case ' ':
//     case 'a':
//     case 'w':
//       break;
//     case 'S':
//       if (!(*open_flags & _O_RANDOM))
//           *open_flags |= _O_SEQUENTIAL;
//       break;
//     case 'R':
//       if (!(*open_flags & _O_SEQUENTIAL))
//           *open_flags |= _O_RANDOM;
//       break;
//     default:
//       ERR("incorrect mode flag: %c\n", mode[-1]);
//       break;
//     }
//
//   if(*mode == ',')
//   {
//     mode++;
//     while(*mode == ' ') mode++;
//     if(!MSVCRT_CHECK_PMT(!wcsncmp(L"ccs", mode, 3)))
//       return -1;
//     mode += 3;
//     while(*mode == ' ') mode++;
//     if(!MSVCRT_CHECK_PMT(*mode == '='))
//         return -1;
//     mode++;
//     while(*mode == ' ') mode++;
//
//     if(!_wcsnicmp(L"utf-8", mode, 5))
//     {
//       *open_flags |= _O_U8TEXT;
//       mode += 5;
//     }
//     else if(!_wcsnicmp(L"utf-16le", mode, 8))
//     {
//       *open_flags |= _O_U16TEXT;
//       mode += 8;
//     }
//     else if(!_wcsnicmp(L"unicode", mode, 7))
//     {
//       *open_flags |= _O_WTEXT;
//       mode += 7;
//     }
//     else
//     {
//       MSVCRT_INVALID_PMT(0, EINVAL);
//       return -1;
//     }
//
//     while(*mode == ' ') mode++;
//   }
//
//   if(!MSVCRT_CHECK_PMT(*mode == 0))
//     return -1;
//   return 0;
// }
//
// /*********************************************************************
//  *		_fdopen (MSVCRT.@)
//  */
// FILE*  _fdopen(int fd, const char *mode)
// {
//     FILE *ret;
//     wchar_t *modeW = NULL;
//
//     if (mode && !(modeW = msvcrt_wstrdupa(mode))) return NULL;
//
//     ret = _wfdopen(fd, modeW);
//
//     free(modeW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wfdopen (MSVCRT.@)
//  */
// FILE*  _wfdopen(int fd, const wchar_t *mode)
// {
//   int open_flags, stream_flags;
//   FILE* file;
//
//   if (msvcrt_get_flags(mode, &open_flags, &stream_flags) == -1) return NULL;
//
//   LOCK_FILES();
//   if (!(file = msvcrt_alloc_fp()))
//     file = NULL;
//   else if (msvcrt_init_fp(file, fd, stream_flags) == -1)
//   {
//     file->_flag = 0;
//     file = NULL;
//   }
//   else TRACE(":fd (%d) mode (%s) FILE* (%p)\n", fd, debugstr_w(mode), file);
//   UNLOCK_FILES();
//
//   return file;
// }
//
// /*********************************************************************
//  *		_filelength (MSVCRT.@)
//  */
// __msvcrt_long CDECL _filelength(int fd)
// {
//   LONG curPos = _lseek(fd, 0, SEEK_CUR);
//   if (curPos != -1)
//   {
//     LONG endPos = _lseek(fd, 0, SEEK_END);
//     if (endPos != -1)
//     {
//       if (endPos != curPos)
//         _lseek(fd, curPos, SEEK_SET);
//       return endPos;
//     }
//   }
//   return -1;
// }
//
// /*********************************************************************
//  *		_filelengthi64 (MSVCRT.@)
//  */
// __int64 CDECL _filelengthi64(int fd)
// {
//   __int64 curPos = _lseeki64(fd, 0, SEEK_CUR);
//   if (curPos != -1)
//   {
//     __int64 endPos = _lseeki64(fd, 0, SEEK_END);
//     if (endPos != -1)
//     {
//       if (endPos != curPos)
//         _lseeki64(fd, curPos, SEEK_SET);
//       return endPos;
//     }
//   }
//   return -1;
// }
//
// /*********************************************************************
//  *		_fileno (MSVCRT.@)
//  */
// int CDECL _fileno(FILE* file)
// {
//   TRACE(":FILE* (%p) fd (%d)\n",file,file->_file);
//   return file->_file;
// }
//
// /*********************************************************************
//  *		_fstat64 (MSVCRT.@)
//  */
// int CDECL _fstat64(int fd, struct _stat64* buf)
// {
//   ioinfo *info = get_ioinfo(fd);
//   DWORD dw;
//   DWORD type;
//
//   TRACE(":fd (%d) stat (%p)\n", fd, buf);
//   if (info->handle == INVALID_HANDLE_VALUE)
//   {
//     release_ioinfo(info);
//     return -1;
//   }
//
//   if (!buf)
//   {
//     WARN(":failed-NULL buf\n");
//     msvcrt_set_errno(ERROR_INVALID_PARAMETER);
//     release_ioinfo(info);
//     return -1;
//   }
//
//   memset(buf, 0, sizeof(struct _stat64));
//   type = GetFileType(info->handle);
//   if (type == FILE_TYPE_PIPE)
//   {
//     buf->st_dev = buf->st_rdev = fd;
//     buf->st_mode = _S_IFIFO;
//     buf->st_nlink = 1;
//   }
//   else if (type == FILE_TYPE_CHAR)
//   {
//     buf->st_dev = buf->st_rdev = fd;
//     buf->st_mode = _S_IFCHR;
//     buf->st_nlink = 1;
//   }
//   else /* FILE_TYPE_DISK etc. */
//   {
//     FILE_BASIC_INFORMATION basic_info;
//     FILE_STANDARD_INFORMATION std_info;
//     IO_STATUS_BLOCK io;
//     NTSTATUS status;
//
//     if ((status = NtQueryInformationFile( info->handle, &io, &basic_info, sizeof(basic_info), FileBasicInformation )) ||
//         (status = NtQueryInformationFile( info->handle, &io, &std_info, sizeof(std_info), FileStandardInformation )))
//     {
//       WARN(":failed-error %lx\n", status);
//       msvcrt_set_errno(ERROR_INVALID_PARAMETER);
//       release_ioinfo(info);
//       return -1;
//     }
//     buf->st_mode = _S_IFREG | 0444;
//     if (!(basic_info.FileAttributes & FILE_ATTRIBUTE_READONLY))
//       buf->st_mode |= 0222;
//     buf->st_size  = std_info.EndOfFile.QuadPart;
//     RtlTimeToSecondsSince1970((LARGE_INTEGER *)&basic_info.LastAccessTime, &dw);
//     buf->st_atime = dw;
//     RtlTimeToSecondsSince1970((LARGE_INTEGER *)&basic_info.LastWriteTime, &dw);
//     buf->st_mtime = buf->st_ctime = dw;
//     buf->st_nlink = std_info.NumberOfLinks;
//     TRACE(":dwFileAttributes = %#lx, mode set to %#x\n",
//             basic_info.FileAttributes, buf->st_mode);
//   }
//   release_ioinfo(info);
//   return 0;
// }
//
// /*********************************************************************
//  *		_fstati64 (MSVCRT.@)
//  */
// int CDECL _fstati64(int fd, struct _stati64* buf)
// {
//   int ret;
//   struct _stat64 buf64;
//
//   ret = _fstat64(fd, &buf64);
//   if (!ret)
//     msvcrt_stat64_to_stati64(&buf64, buf);
//   return ret;
// }
//
// /*********************************************************************
//  *             _fstat (MSVCRT.@)
//  */
// int CDECL _fstat(int fd, struct _stat* buf)
// { int ret;
//   struct _stat64 buf64;
//
//   ret = _fstat64(fd, &buf64);
//   if (!ret)
//       msvcrt_stat64_to_stat(&buf64, buf);
//   return ret;
// }
//
// /*********************************************************************
//  *		_fstat32 (MSVCR80.@)
//  */
// int CDECL _fstat32(int fd, struct _stat32* buf)
// {
//     int ret;
//     struct _stat64 buf64;
//
//     ret = _fstat64(fd, &buf64);
//     if (!ret)
//         msvcrt_stat64_to_stat32(&buf64, buf);
//     return ret;
// }
//
// /*********************************************************************
//  *		_fstat32i64 (MSVCR80.@)
//  */
// int CDECL _fstat32i64(int fd, struct _stat32i64* buf)
// {
//     int ret;
//     struct _stat64 buf64;
//
//     ret = _fstat64(fd, &buf64);
//     if (!ret)
//         msvcrt_stat64_to_stat32i64(&buf64, buf);
//     return ret;
// }
//
// /*********************************************************************
//  *		_fstat64i32 (MSVCR80.@)
//  */
// int CDECL _fstat64i32(int fd, struct _stat64i32* buf)
// {
//     int ret;
//     struct _stat64 buf64;
//
//     ret = _fstat64(fd, &buf64);
//     if (!ret)
//         msvcrt_stat64_to_stat64i32(&buf64, buf);
//     return ret;
// }
//
// /*********************************************************************
//  *		_futime64 (MSVCRT.@)
//  */
// int CDECL _futime64(int fd, struct __utimbuf64 *t)
// {
//   ioinfo *info = get_ioinfo(fd);
//   FILETIME at, wt;
//
//   if (!t)
//   {
//       time_to_filetime( _time64(NULL), &at );
//       wt = at;
//   }
//   else
//   {
//       time_to_filetime( t->actime, &at );
//       time_to_filetime( t->modtime, &wt );
//   }
//
//   if (!SetFileTime(info->handle, NULL, &at, &wt))
//   {
//     release_ioinfo(info);
//     msvcrt_set_errno(GetLastError());
//     return -1 ;
//   }
//   release_ioinfo(info);
//   return 0;
// }
//
// /*********************************************************************
//  *		_futime32 (MSVCRT.@)
//  */
// int CDECL _futime32(int fd, struct __utimbuf32 *t)
// {
//     if (t)
//     {
//         struct __utimbuf64 t64;
//         t64.actime = t->actime;
//         t64.modtime = t->modtime;
//         return _futime64( fd, &t64 );
//     }
//     else
//         return _futime64( fd, NULL );
// }
//
// /*********************************************************************
//  *		_get_osfhandle (MSVCRT.@)
//  */
// intptr_t CDECL _get_osfhandle(int fd)
// {
//   HANDLE hand = get_ioinfo_nolock(fd)->handle;
//   TRACE(":fd (%d) handle (%p)\n",fd,hand);
//
//   if(hand == INVALID_HANDLE_VALUE)
//       *_errno() = EBADF;
//   return (intptr_t)hand;
// }
//
// /*********************************************************************
//  *		_mktemp_s (MSVCRT.@)
//  */
// int CDECL _mktemp_s(char *pattern, size_t size)
// {
//     DWORD len, wlen, xno, id;
//     wchar_t *pathW;
//
//     if(!MSVCRT_CHECK_PMT(pattern!=NULL))
//         return EINVAL;
//
//     for(len=0; len<size; len++)
//         if(!pattern[len])
//             break;
//     if(!MSVCRT_CHECK_PMT(len!=size && len>=6)) {
//         if(size)
//             pattern[0] = 0;
//         return EINVAL;
//     }
//
//     for(xno=1; xno<=6; xno++)
//         if(!MSVCRT_CHECK_PMT(pattern[len-xno] == 'X'))
//             return EINVAL;
//
//     id = GetCurrentProcessId();
//     for(xno=1; xno<6; xno++) {
//         pattern[len-xno] = id%10 + '0';
//         id /= 10;
//     }
//
//     if(!(pathW = wstrdupa_utf8(pattern))) return *_errno();
//     wlen = wcslen(pathW);
//     for(pathW[wlen-6]='a'; pathW[wlen-6]<='z'; pathW[wlen-6]++) {
//         if(GetFileAttributesW(pathW) == INVALID_FILE_ATTRIBUTES) {
//             pattern[len-6] = pathW[wlen-6];
//             free(pathW);
//             return 0;
//         }
//     }
//     free(pathW);
//
//     pattern[0] = 0;
//     *_errno() = EEXIST;
//     return EEXIST;
// }
//
// /*********************************************************************
//  *		_mktemp (MSVCRT.@)
//  */
// char *  _mktemp(char *pattern)
// {
//   wchar_t *pathW, *p;
//   int numX = 0;
//   char *retVal = pattern;
//   int id;
//   char letter = 'a';
//
//   if(!pattern)
//       return NULL;
//
//   while(*pattern)
//     numX = (*pattern++ == 'X')? numX + 1 : 0;
//   if (numX < 6)
//     return NULL;
//   pattern--;
//   id = GetCurrentProcessId();
//   numX = 6;
//   while(numX--)
//   {
//     int tempNum = id / 10;
//     *pattern-- = id - (tempNum * 10) + '0';
//     id = tempNum;
//   }
//   pattern++;
//   if (!(pathW = wstrdupa_utf8(retVal)))
//     return NULL;
//   p = pathW + wcslen(pathW) - 6;
//   do
//   {
//     *p = letter++;
//     if (GetFileAttributesW(pathW) == INVALID_FILE_ATTRIBUTES)
//     {
//       *pattern = *p;
//       free(pathW);
//       return retVal;
//     }
//   } while(letter <= 'z');
//   free(pathW);
//   return NULL;
// }
//
// /*********************************************************************
//  *		_wmktemp_s (MSVCRT.@)
//  */
// int CDECL _wmktemp_s(wchar_t *pattern, size_t size)
// {
//     DWORD len, xno, id;
//
//     if(!MSVCRT_CHECK_PMT(pattern!=NULL))
//         return EINVAL;
//
//     for(len=0; len<size; len++)
//         if(!pattern[len])
//             break;
//     if(!MSVCRT_CHECK_PMT(len!=size && len>=6)) {
//         if(size)
//             pattern[0] = 0;
//         return EINVAL;
//     }
//
//     for(xno=1; xno<=6; xno++)
//         if(!MSVCRT_CHECK_PMT(pattern[len-xno] == 'X'))
//             return EINVAL;
//
//     id = GetCurrentProcessId();
//     for(xno=1; xno<6; xno++) {
//         pattern[len-xno] = id%10 + '0';
//         id /= 10;
//     }
//
//     for(pattern[len-6]='a'; pattern[len-6]<='z'; pattern[len-6]++) {
//         if(GetFileAttributesW(pattern) == INVALID_FILE_ATTRIBUTES)
//             return 0;
//     }
//
//     pattern[0] = 0;
//     *_errno() = EEXIST;
//     return EEXIST;
// }
//
// /*********************************************************************
//  *		_wmktemp (MSVCRT.@)
//  */
// wchar_t *  _wmktemp(wchar_t *pattern)
// {
//   int numX = 0;
//   wchar_t *retVal = pattern;
//   int id;
//   wchar_t letter = 'a';
//
//   if(!pattern)
//       return NULL;
//
//   while(*pattern)
//     numX = (*pattern++ == 'X')? numX + 1 : 0;
//   if (numX < 6)
//     return NULL;
//   pattern--;
//   id = GetCurrentProcessId();
//   numX = 6;
//   while(numX--)
//   {
//     int tempNum = id / 10;
//     *pattern-- = id - (tempNum * 10) + '0';
//     id = tempNum;
//   }
//   pattern++;
//   do
//   {
//     if (GetFileAttributesW(retVal) == INVALID_FILE_ATTRIBUTES)
//       return retVal;
//     *pattern = letter++;
//   } while(letter != '|');
//   return NULL;
// }
//
// static unsigned split_oflags(unsigned oflags)
// {
//     int wxflags = 0;
//     unsigned unsupp; /* until we support everything */
//
//     if (oflags & _O_APPEND)              wxflags |= WX_APPEND;
//     if (oflags & _O_BINARY)              {/* Nothing to do */}
//     else if (oflags & _O_TEXT)           wxflags |= WX_TEXT;
//     else if (oflags & _O_WTEXT)          wxflags |= WX_TEXT;
//     else if (oflags & _O_U16TEXT)        wxflags |= WX_TEXT;
//     else if (oflags & _O_U8TEXT)         wxflags |= WX_TEXT;
//     else
//     {
//         int fmode;
//         _get_fmode(&fmode);
//         if (!(fmode & _O_BINARY))        wxflags |= WX_TEXT; /* default to TEXT*/
//     }
//     if (oflags & _O_NOINHERIT)           wxflags |= WX_DONTINHERIT;
//
//     if ((unsupp = oflags & ~(_O_BINARY | _O_TEXT | _O_APPEND | _O_TRUNC | _O_EXCL | _O_CREAT |
//                     _O_RDWR | _O_WRONLY | _O_TEMPORARY | _O_NOINHERIT | _O_SEQUENTIAL |
//                     _O_RANDOM | _O_SHORT_LIVED | _O_WTEXT | _O_U16TEXT | _O_U8TEXT)))
//         ERR(":unsupported oflags %#x\n",unsupp);
//
//     return wxflags;
// }
//
// /*********************************************************************
//  *              _pipe (MSVCRT.@)
//  */
// int CDECL _pipe(int *pfds, unsigned int psize, int textmode)
// {
//   int ret = -1;
//   SECURITY_ATTRIBUTES sa;
//   HANDLE readHandle, writeHandle;
//
//   if (!pfds)
//   {
//     *_errno() = EINVAL;
//     return -1;
//   }
//
//   sa.nLength = sizeof(SECURITY_ATTRIBUTES);
//   sa.bInheritHandle = !(textmode & _O_NOINHERIT);
//   sa.lpSecurityDescriptor = NULL;
//   if (CreatePipe(&readHandle, &writeHandle, &sa, psize))
//   {
//     unsigned int wxflags = split_oflags(textmode);
//     int fd;
//
//     fd = msvcrt_alloc_fd(readHandle, wxflags|WX_PIPE);
//     if (fd != -1)
//     {
//       pfds[0] = fd;
//       fd = msvcrt_alloc_fd(writeHandle, wxflags|WX_PIPE);
//       if (fd != -1)
//       {
//         pfds[1] = fd;
//         ret = 0;
//       }
//       else
//       {
//         _close(pfds[0]);
//         CloseHandle(writeHandle);
//         *_errno() = EMFILE;
//       }
//     }
//     else
//     {
//       CloseHandle(readHandle);
//       CloseHandle(writeHandle);
//       *_errno() = EMFILE;
//     }
//   }
//   else
//     msvcrt_set_errno(GetLastError());
//
//   return ret;
// }
//
// static int check_bom(HANDLE h, int oflags, BOOL seek)
// {
//     char bom[sizeof(utf8_bom)];
//     DWORD r;
//
//     if (!ReadFile(h, bom, sizeof(utf8_bom), &r, NULL))
//         return oflags;
//
//     if (r==sizeof(utf8_bom) && !memcmp(bom, utf8_bom, sizeof(utf8_bom))) {
//         oflags = (oflags & ~(_O_WTEXT | _O_U16TEXT)) | _O_U8TEXT;
//     }else if (r>=sizeof(utf16_bom) && !memcmp(bom, utf16_bom, sizeof(utf16_bom))) {
//         if (seek && r>2)
//             SetFilePointer(h, 2, NULL, FILE_BEGIN);
//         oflags = (oflags & ~(_O_WTEXT | _O_U8TEXT)) | _O_U16TEXT;
//     }else if (seek) {
//         SetFilePointer(h, 0, NULL, FILE_BEGIN);
//     }
//
//     return oflags;
// }
//
// /*********************************************************************
//  *              _wsopen_dispatch (UCRTBASE.@)
//  */
// int CDECL _wsopen_dispatch( const wchar_t* path, int oflags, int shflags, int pmode,
//     int *fd, int secure )
// {
//   DWORD access = 0, creation = 0, attrib;
//   SECURITY_ATTRIBUTES sa;
//   DWORD sharing, type;
//   int wxflag;
//   HANDLE hand;
//
//   TRACE("path: (%s) oflags: %#x shflags: %#x pmode: %#x fd*: %p secure: %d\n",
//         debugstr_w(path), oflags, shflags, pmode, fd, secure);
//
//   if (!MSVCRT_CHECK_PMT( fd != NULL )) return EINVAL;
//   *fd = -1;
//   if (!MSVCRT_CHECK_PMT(path != NULL)) return EINVAL;
//
//   wxflag = split_oflags(oflags);
//   switch (oflags & (_O_RDONLY | _O_WRONLY | _O_RDWR))
//   {
//   case _O_RDONLY: access |= GENERIC_READ; break;
//   case _O_WRONLY: access |= GENERIC_WRITE; break;
//   case _O_RDWR:   access |= GENERIC_WRITE | GENERIC_READ; break;
//   }
//
//   if (oflags & _O_CREAT)
//   {
//     if (secure && !MSVCRT_CHECK_PMT(!(pmode & ~(_S_IREAD | _S_IWRITE))))
//       return EINVAL;
//
//     if (oflags & _O_EXCL)
//       creation = CREATE_NEW;
//     else if (oflags & _O_TRUNC)
//       creation = CREATE_ALWAYS;
//     else
//       creation = OPEN_ALWAYS;
//   }
//   else  /* no _O_CREAT */
//   {
//     if (oflags & _O_TRUNC)
//       creation = TRUNCATE_EXISTING;
//     else
//       creation = OPEN_EXISTING;
//   }
//
//   switch( shflags )
//   {
//     case _SH_DENYRW:
//       sharing = 0;
//       break;
//     case _SH_DENYWR:
//       sharing = FILE_SHARE_READ;
//       break;
//     case _SH_DENYRD:
//       sharing = FILE_SHARE_WRITE;
//       break;
//     case _SH_DENYNO:
//       sharing = FILE_SHARE_READ | FILE_SHARE_WRITE;
//       break;
//     default:
//       ERR( "Unhandled shflags %#x\n", shflags );
//       return EINVAL;
//   }
//
//   if (!(pmode & ~MSVCRT_umask & _S_IWRITE))
//       attrib = FILE_ATTRIBUTE_READONLY;
//   else
//       attrib = FILE_ATTRIBUTE_NORMAL;
//
//   if (oflags & _O_TEMPORARY)
//   {
//       attrib |= FILE_FLAG_DELETE_ON_CLOSE;
//       access |= DELETE;
//       sharing |= FILE_SHARE_DELETE;
//   }
//
//   if (oflags & _O_RANDOM)
//       attrib |= FILE_FLAG_RANDOM_ACCESS;
//   if (oflags & _O_SEQUENTIAL)
//       attrib |= FILE_FLAG_SEQUENTIAL_SCAN;
//   if (oflags & _O_SHORT_LIVED)
//       attrib |= FILE_ATTRIBUTE_TEMPORARY;
//
//   sa.nLength              = sizeof( SECURITY_ATTRIBUTES );
//   sa.lpSecurityDescriptor = NULL;
//   sa.bInheritHandle       = !(oflags & _O_NOINHERIT);
//
//   if ((oflags & (_O_WTEXT | _O_U16TEXT | _O_U8TEXT))
//           && (creation==OPEN_ALWAYS || creation==OPEN_EXISTING)
//           && !(access&GENERIC_READ))
//   {
//       hand = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ|FILE_SHARE_WRITE,
//               &sa, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
//       if (hand != INVALID_HANDLE_VALUE)
//       {
//           oflags = check_bom(hand, oflags, FALSE);
//           CloseHandle(hand);
//       }
//   }
//
//   hand = CreateFileW(path, access, sharing, &sa, creation, attrib, 0);
//   if (hand == INVALID_HANDLE_VALUE)  {
//     WARN(":failed-last error (%ld)\n", GetLastError());
//     msvcrt_set_errno(GetLastError());
//     return *_errno();
//   }
//
//   if (oflags & (_O_WTEXT | _O_U16TEXT | _O_U8TEXT))
//   {
//       LARGE_INTEGER size = {{0}};
//
//       if ((access & GENERIC_WRITE) && (creation==OPEN_EXISTING || creation==OPEN_ALWAYS))
//           GetFileSizeEx(hand, &size);
//
//       if ((access & GENERIC_WRITE) && (creation==CREATE_NEW
//                   || creation==CREATE_ALWAYS || creation==TRUNCATE_EXISTING
//                   || ((creation==OPEN_EXISTING || creation==OPEN_ALWAYS) && !size.QuadPart)))
//       {
//           if (oflags & _O_U8TEXT)
//           {
//               DWORD written = 0, tmp;
//
//               while(written!=sizeof(utf8_bom) && WriteFile(hand, (char*)utf8_bom+written,
//                           sizeof(utf8_bom)-written, &tmp, NULL))
//                   written += tmp;
//               if (written != sizeof(utf8_bom)) {
//                   WARN("error writing BOM\n");
//                   CloseHandle(hand);
//                   msvcrt_set_errno(GetLastError());
//                   return *_errno();
//               }
//           }
//           else
//           {
//               DWORD written = 0, tmp;
//
//               while(written!=sizeof(utf16_bom) && WriteFile(hand, (char*)utf16_bom+written,
//                           sizeof(utf16_bom)-written, &tmp, NULL))
//                   written += tmp;
//               if (written != sizeof(utf16_bom))
//               {
//                   WARN("error writing BOM\n");
//                   CloseHandle(hand);
//                   msvcrt_set_errno(GetLastError());
//                   return *_errno();
//               }
//               oflags |= _O_U16TEXT;
//           }
//       }
//       else if (access & GENERIC_READ)
//           oflags = check_bom(hand, oflags, TRUE);
//   }
//
//   type = GetFileType(hand);
//   if (type == FILE_TYPE_CHAR)
//       wxflag |= WX_TTY;
//   else if (type == FILE_TYPE_PIPE)
//       wxflag |= WX_PIPE;
//
//   *fd = msvcrt_alloc_fd(hand, wxflag);
//   if (*fd == -1)
//       return *_errno();
//
//   if (oflags & _O_WTEXT)
//       ioinfo_set_unicode(get_ioinfo_nolock(*fd), TRUE);
//
//   if (oflags & _O_U16TEXT)
//       ioinfo_set_textmode(get_ioinfo_nolock(*fd), TEXTMODE_UTF16LE);
//   else if (oflags & _O_U8TEXT)
//       ioinfo_set_textmode(get_ioinfo_nolock(*fd), TEXTMODE_UTF8);
//
//   TRACE(":fd (%d) handle (%p)\n", *fd, hand);
//   return 0;
// }
//
//
// /*********************************************************************
//  *              _wsopen_s (MSVCRT.@)
//  */
// int CDECL _wsopen_s( int *fd, const wchar_t* path, int oflags, int shflags, int pmode )
// {
//     return _wsopen_dispatch( path, oflags, shflags, pmode, fd, 1 );
// }
//
// /*********************************************************************
//  *              _wsopen (MSVCRT.@)
//  */
// int __cdecl _wsopen( const wchar_t *path, int oflags, int shflags, ... )
// {
//   int pmode;
//   int fd;
//
//   if (oflags & _O_CREAT)
//   {
//     va_list ap;
//
//     va_start(ap, shflags);
//     pmode = va_arg(ap, int);
//     va_end(ap);
//   }
//   else
//     pmode = 0;
//
//   return _wsopen_dispatch(path, oflags, shflags, pmode, &fd, 0) ? -1 : fd;
// }
//
//
// /*********************************************************************
//  *              _sopen_dispatch (UCRTBASE.@)
//  */
// int CDECL _sopen_dispatch( const char *path, int oflags, int shflags,
//     int pmode, int *fd, int secure)
// {
//     wchar_t *pathW = NULL;
//     int ret;
//
//     if (!MSVCRT_CHECK_PMT(fd != NULL))
//         return EINVAL;
//     *fd = -1;
//     if (path && !(pathW = wstrdupa_utf8(path))) return *_errno();
//
//     ret = _wsopen_dispatch(pathW, oflags, shflags, pmode, fd, secure);
//     free(pathW);
//     return ret;
// }
//
// /*********************************************************************
//  *              _sopen_s (MSVCRT.@)
//  */
// int CDECL _sopen_s( int *fd, const char *path, int oflags, int shflags, int pmode )
// {
//     return _sopen_dispatch(path, oflags, shflags, pmode, fd, 1);
// }
//
// /*********************************************************************
//  *              _sopen (MSVCRT.@)
//  */
// int __cdecl _sopen( const char *path, int oflags, int shflags, ... )
// {
//   int pmode;
//   int fd;
//
//   if (oflags & _O_CREAT)
//   {
//     va_list ap;
//
//     va_start(ap, shflags);
//     pmode = va_arg(ap, int);
//     va_end(ap);
//   }
//   else
//     pmode = 0;
//
//   return _sopen_dispatch(path, oflags, shflags, pmode, &fd, 0) ? -1 : fd;
// }
//
// /*********************************************************************
//  *              _open (MSVCRT.@)
//  */
// int __cdecl _open( const char *path, int flags, ... )
// {
//   va_list ap;
//
//   if (flags & _O_CREAT)
//   {
//     int pmode;
//     va_start(ap, flags);
//     pmode = va_arg(ap, int);
//     va_end(ap);
//     return _sopen( path, flags, _SH_DENYNO, pmode );
//   }
//   else
//     return _sopen( path, flags, _SH_DENYNO);
// }
//
// /*********************************************************************
//  *              _wopen (MSVCRT.@)
//  */
// int __cdecl _wopen(const wchar_t *path,int flags,...)
// {
//   va_list ap;
//
//   if (flags & _O_CREAT)
//   {
//     int pmode;
//     va_start(ap, flags);
//     pmode = va_arg(ap, int);
//     va_end(ap);
//     return _wsopen( path, flags, _SH_DENYNO, pmode );
//   }
//   else
//     return _wsopen( path, flags, _SH_DENYNO);
// }
//
// /*********************************************************************
//  *		_creat (MSVCRT.@)
//  */
// int CDECL _creat(const char *path, int pmode)
// {
//   int flags = _O_CREAT | _O_TRUNC | _O_RDWR;
//   return _open(path, flags, pmode);
// }
//
// /*********************************************************************
//  *		_wcreat (MSVCRT.@)
//  */
// int CDECL _wcreat(const wchar_t *path, int pmode)
// {
//   int flags = _O_CREAT | _O_TRUNC | _O_RDWR;
//   return _wopen(path, flags, pmode);
// }
//
// /*********************************************************************
//  *		_open_osfhandle (MSVCRT.@)
//  */
// int CDECL _open_osfhandle(intptr_t handle, int oflags)
// {
//   DWORD flags;
//   int fd;
//
//   /* _O_RDONLY (0) always matches, so set the read flag
//    * MFC's CStdioFile clears O_RDONLY (0)! if it wants to write to the
//    * file, so set the write flag. It also only sets _O_TEXT if it wants
//    * text - it never sets _O_BINARY.
//    */
//   /* don't let split_oflags() decide the mode if no mode is passed */
//   if (!(oflags & (_O_BINARY | _O_TEXT)))
//       oflags |= _O_BINARY;
//
//   flags = GetFileType((HANDLE)handle);
//   if (flags==FILE_TYPE_UNKNOWN && GetLastError()!=NO_ERROR)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//
//   if (flags == FILE_TYPE_CHAR)
//     flags = WX_TTY;
//   else if (flags == FILE_TYPE_PIPE)
//     flags = WX_PIPE;
//   else
//     flags = 0;
//   flags |= split_oflags(oflags);
//
//   fd = msvcrt_alloc_fd((HANDLE)handle, flags);
//   TRACE(":handle (%Iu) fd (%d) flags %#lx\n", handle, fd, flags);
//   return fd;
// }
//
// /*********************************************************************
//  *		_rmtmp (MSVCRT.@)
//  */
// int CDECL _rmtmp(void)
// {
//   int num_removed = 0, i;
//   FILE *file;
//
//   LOCK_FILES();
//   for (i = 3; i < MSVCRT_stream_idx; i++) {
//     file = msvcrt_get_file(i);
//
//     if (file->_tmpfname)
//     {
//       fclose(file);
//       num_removed++;
//     }
//   }
//   UNLOCK_FILES();
//
//   if (num_removed)
//     TRACE(":removed (%d) temp files\n",num_removed);
//   return num_removed;
// }
//
// static inline int get_utf8_char_len(char ch)
// {
//     if((ch&0xf8) == 0xf0)
//         return 4;
//     else if((ch&0xf0) == 0xe0)
//         return 3;
//     else if((ch&0xe0) == 0xc0)
//         return 2;
//     return 1;
// }
//
// /*********************************************************************
//  * (internal) read_utf8
//  */
// static int read_utf8(ioinfo *fdinfo, wchar_t *buf, unsigned int count)
// {
//     HANDLE hand = fdinfo->handle;
//     char min_buf[4], *readbuf, lookahead;
//     DWORD readbuf_size, pos=0, num_read=1, char_len, i, j;
//
//     /* make the buffer big enough to hold at least one character */
//     /* read bytes have to fit to output and lookahead buffers */
//     count /= 2;
//     readbuf_size = count < 4 ? 4 : count;
//     if(readbuf_size<=4 || !(readbuf = malloc(readbuf_size))) {
//         readbuf_size = 4;
//         readbuf = min_buf;
//     }
//
//     if(fdinfo->lookahead[0] != '\n') {
//         readbuf[pos++] = fdinfo->lookahead[0];
//         fdinfo->lookahead[0] = '\n';
//
//         if(fdinfo->lookahead[1] != '\n') {
//             readbuf[pos++] = fdinfo->lookahead[1];
//             fdinfo->lookahead[1] = '\n';
//
//             if(fdinfo->lookahead[2] != '\n') {
//                 readbuf[pos++] = fdinfo->lookahead[2];
//                 fdinfo->lookahead[2] = '\n';
//             }
//         }
//     }
//
//     /* NOTE: this case is broken in native dll, reading
//      *        sometimes fails when small buffer is passed
//      */
//     if(count < 4) {
//         if(!pos && !ReadFile(hand, readbuf, 1, &num_read, NULL)) {
//             if (GetLastError() == ERROR_BROKEN_PIPE) {
//                 fdinfo->wxflag |= WX_ATEOF;
//                 return 0;
//             }else {
//                 msvcrt_set_errno(GetLastError());
//                 if (GetLastError() == ERROR_ACCESS_DENIED)
//                     *_errno() = EBADF;
//                 return -1;
//             }
//         }else if(!num_read) {
//             fdinfo->wxflag |= WX_ATEOF;
//             return 0;
//         }else {
//             pos++;
//         }
//
//         char_len = get_utf8_char_len(readbuf[0]);
//         if(char_len>pos) {
//             if(ReadFile(hand, readbuf+pos, char_len-pos, &num_read, NULL))
//                 pos += num_read;
//         }
//
//         if(readbuf[0] == '\n')
//             fdinfo->wxflag |= WX_READNL;
//         else
//             fdinfo->wxflag &= ~WX_READNL;
//
//         if(readbuf[0] == 0x1a) {
//             fdinfo->wxflag |= WX_ATEOF;
//             return 0;
//         }
//
//         if(readbuf[0] == '\r') {
//             if(!ReadFile(hand, &lookahead, 1, &num_read, NULL) || num_read!=1)
//                 buf[0] = '\r';
//             else if(lookahead == '\n')
//                 buf[0] = '\n';
//             else {
//                 buf[0] = '\r';
//                 if(fdinfo->wxflag & (WX_PIPE | WX_TTY))
//                     fdinfo->lookahead[0] = lookahead;
//                 else
//                     SetFilePointer(fdinfo->handle, -1, NULL, FILE_CURRENT);
//             }
//             return 2;
//         }
//
//         if(!(num_read = MultiByteToWideChar(CP_UTF8, 0, readbuf, pos, buf, count))) {
//             msvcrt_set_errno(GetLastError());
//             return -1;
//         }
//
//         return num_read*2;
//     }
//
//     if(!ReadFile(hand, readbuf+pos, readbuf_size-pos, &num_read, NULL)) {
//         if(pos) {
//             num_read = 0;
//         }else if(GetLastError() == ERROR_BROKEN_PIPE) {
//             fdinfo->wxflag |= WX_ATEOF;
//             if (readbuf != min_buf) free(readbuf);
//             return 0;
//         }else {
//             msvcrt_set_errno(GetLastError());
//             if (GetLastError() == ERROR_ACCESS_DENIED)
//                 *_errno() = EBADF;
//             if (readbuf != min_buf) free(readbuf);
//             return -1;
//         }
//     }else if(!pos && !num_read) {
//         fdinfo->wxflag |= WX_ATEOF;
//         if (readbuf != min_buf) free(readbuf);
//         return 0;
//     }
//
//     pos += num_read;
//     if(readbuf[0] == '\n')
//         fdinfo->wxflag |= WX_READNL;
//     else
//         fdinfo->wxflag &= ~WX_READNL;
//
//     /* Find first byte of last character (may be incomplete) */
//     for(i=pos-1; i>0 && i>pos-4; i--)
//         if((readbuf[i]&0xc0) != 0x80)
//             break;
//     char_len = get_utf8_char_len(readbuf[i]);
//     if(char_len+i <= pos)
//         i += char_len;
//
//     if(fdinfo->wxflag & (WX_PIPE | WX_TTY)) {
//         if(i < pos)
//             fdinfo->lookahead[0] = readbuf[i];
//         if(i+1 < pos)
//             fdinfo->lookahead[1] = readbuf[i+1];
//         if(i+2 < pos)
//             fdinfo->lookahead[2] = readbuf[i+2];
//     }else if(i < pos) {
//         SetFilePointer(fdinfo->handle, i-pos, NULL, FILE_CURRENT);
//     }
//     pos = i;
//
//     for(i=0, j=0; i<pos; i++) {
//         if(readbuf[i] == 0x1a) {
//             fdinfo->wxflag |= WX_ATEOF;
//             break;
//         }
//
//         /* strip '\r' if followed by '\n' */
//         if(readbuf[i] == '\r' && i+1==pos) {
//             if(fdinfo->lookahead[0] != '\n' || !ReadFile(hand, &lookahead, 1, &num_read, NULL) || !num_read) {
//                 readbuf[j++] = '\r';
//             }else if(lookahead == '\n' && j==0) {
//                 readbuf[j++] = '\n';
//             }else {
//                 if(lookahead != '\n')
//                     readbuf[j++] = '\r';
//
//                 if(fdinfo->wxflag & (WX_PIPE | WX_TTY))
//                     fdinfo->lookahead[0] = lookahead;
//                 else
//                     SetFilePointer(fdinfo->handle, -1, NULL, FILE_CURRENT);
//             }
//         }else if(readbuf[i]!='\r' || readbuf[i+1]!='\n') {
//             readbuf[j++] = readbuf[i];
//         }
//     }
//     pos = j;
//
//     if(!(num_read = MultiByteToWideChar(CP_UTF8, 0, readbuf, pos, buf, count))) {
//         msvcrt_set_errno(GetLastError());
//         if (readbuf != min_buf) free(readbuf);
//         return -1;
//     }
//
//     if (readbuf != min_buf) free(readbuf);
//     return num_read*2;
// }
//
// /*********************************************************************
//  * (internal) read_i
//  *
//  * When reading \r as last character in text mode, read() positions
//  * the file pointer on the \r character while getc() goes on to
//  * the following \n
//  */
// static int read_i(int fd, ioinfo *fdinfo, void *buf, unsigned int count)
// {
//     DWORD num_read, utf16;
//     char *bufstart = buf;
//
//     if (count == 0)
//         return 0;
//
//     if (fdinfo->wxflag & WX_ATEOF) {
//         TRACE("already at EOF, returning 0\n");
//         return 0;
//     }
//     /* Don't trace small reads, it gets *very* annoying */
//     if (count > 4)
//         TRACE(":fd (%d) handle (%p) buf (%p) len (%d)\n", fd, fdinfo->handle, buf, count);
//     if (fdinfo->handle == INVALID_HANDLE_VALUE)
//     {
//         *_errno() = EBADF;
//         return -1;
//     }
//
//     utf16 = ioinfo_get_textmode(fdinfo) == TEXTMODE_UTF16LE;
//     if (ioinfo_get_textmode(fdinfo) != TEXTMODE_ANSI && count&1)
//     {
//         *_errno() = EINVAL;
//         return -1;
//     }
//
//     if((fdinfo->wxflag&WX_TEXT) && ioinfo_get_textmode(fdinfo) == TEXTMODE_UTF8)
//         return read_utf8(fdinfo, buf, count);
//
//     if (fdinfo->lookahead[0]!='\n' || ReadFile(fdinfo->handle, bufstart, count, &num_read, NULL))
//     {
//         if (fdinfo->lookahead[0] != '\n')
//         {
//             bufstart[0] = fdinfo->lookahead[0];
//             fdinfo->lookahead[0] = '\n';
//
//             if (utf16)
//             {
//                 bufstart[1] =  fdinfo->lookahead[1];
//                 fdinfo->lookahead[1] = '\n';
//             }
//
//             if(count>1+utf16 && ReadFile(fdinfo->handle, bufstart+1+utf16, count-1-utf16, &num_read, NULL))
//                 num_read += 1+utf16;
//             else
//                 num_read = 1+utf16;
//         }
//
//         if(utf16 && (num_read&1))
//         {
//             /* msvcr90 uses uninitialized value from the buffer in this case */
//             /* msvcrt ignores additional data */
//             ERR("got odd number of bytes in UTF16 mode\n");
//             num_read--;
//         }
//
//         if (count != 0 && num_read == 0)
//         {
//             fdinfo->wxflag |= WX_ATEOF;
//             TRACE(":EOF %s\n",debugstr_an(buf,num_read));
//         }
//         else if (fdinfo->wxflag & WX_TEXT)
//         {
//             DWORD i, j;
//
//             if (bufstart[0]=='\n' && (!utf16 || bufstart[1]==0))
//                 fdinfo->wxflag |= WX_READNL;
//             else
//                 fdinfo->wxflag &= ~WX_READNL;
//
//             for (i=0, j=0; i<num_read; i+=1+utf16)
//             {
//                 /* in text mode, a ctrl-z signals EOF */
//                 if (bufstart[i]==0x1a && (!utf16 || bufstart[i+1]==0))
//                 {
//                     fdinfo->wxflag |= WX_ATEOF;
//                     TRACE(":^Z EOF %s\n",debugstr_an(buf,num_read));
//                     break;
//                 }
//
//                 /* in text mode, strip \r if followed by \n */
//                 if (bufstart[i]=='\r' && (!utf16 || bufstart[i+1]==0) && i+1+utf16==num_read)
//                 {
//                     char lookahead[2];
//                     DWORD len;
//
//                     lookahead[1] = '\n';
//                     if (ReadFile(fdinfo->handle, lookahead, 1+utf16, &len, NULL) && len)
//                     {
//                         if(lookahead[0]=='\n' && (!utf16 || lookahead[1]==0) && j==0)
//                         {
//                             bufstart[j++] = '\n';
//                             if(utf16) bufstart[j++] = 0;
//                         }
//                         else
//                         {
//                             if(lookahead[0]!='\n' || (utf16 && lookahead[1]!=0))
//                             {
//                                 bufstart[j++] = '\r';
//                                 if(utf16) bufstart[j++] = 0;
//                             }
//
//                             if (fdinfo->wxflag & (WX_PIPE | WX_TTY))
//                             {
//                                 if (lookahead[0]=='\n' && (!utf16 || !lookahead[1]))
//                                 {
//                                     bufstart[j++] = '\n';
//                                     if (utf16) bufstart[j++] = 0;
//                                 }
//                                 else
//                                 {
//                                     fdinfo->lookahead[0] = lookahead[0];
//                                     fdinfo->lookahead[1] = lookahead[1];
//                                 }
//                             }
//                             else
//                                 SetFilePointer(fdinfo->handle, -1-utf16, NULL, FILE_CURRENT);
//                         }
//                     }
//                     else
//                     {
//                         bufstart[j++] = '\r';
//                         if(utf16) bufstart[j++] = 0;
//                     }
//                 }
//                 else if((bufstart[i]!='\r' || (utf16 && bufstart[i+1]!=0))
//                         || (bufstart[i+1+utf16]!='\n' || (utf16 && bufstart[i+3]!=0)))
//                 {
//                     bufstart[j++] = bufstart[i];
//                     if(utf16) bufstart[j++] = bufstart[i+1];
//                 }
//             }
//             num_read = j;
//         }
//     }
//     else
//     {
//         if (GetLastError() == ERROR_BROKEN_PIPE)
//         {
//             TRACE(":end-of-pipe\n");
//             fdinfo->wxflag |= WX_ATEOF;
//             return 0;
//         }
//         else
//         {
//             TRACE(":failed-last error (%ld)\n", GetLastError());
//             msvcrt_set_errno(GetLastError());
//             if (GetLastError() == ERROR_ACCESS_DENIED)
//                 *_errno() = EBADF;
//             return -1;
//         }
//     }
//
//     if (count > 4)
//         TRACE("(%lu), %s\n", num_read, debugstr_an(buf, num_read));
//     return num_read;
// }
//
// /*********************************************************************
//  *		_read (MSVCRT.@)
//  */
// int CDECL _read(int fd, void *buf, unsigned int count)
// {
//     ioinfo *info;
//     int num_read;
//
//     if(fd == MSVCRT_NO_CONSOLE_FD) {
//         *_errno() = EBADF;
//         return -1;
//     }
//
//     info = get_ioinfo(fd);
//     num_read = read_i(fd, info, buf, count);
//     release_ioinfo(info);
//     return num_read;
// }
//
// /*********************************************************************
//  *		_setmode (MSVCRT.@)
//  */
// int CDECL _setmode(int fd,int mode)
// {
//     ioinfo *info = get_ioinfo(fd);
//     int ret = info->wxflag & WX_TEXT ? _O_TEXT : _O_BINARY;
//
//     if(ret==_O_TEXT && ioinfo_get_textmode(info) != TEXTMODE_ANSI)
//         ret = _O_WTEXT;
//
//     if(mode!=_O_TEXT && mode!=_O_BINARY && mode!=_O_WTEXT
//                 && mode!=_O_U16TEXT && mode!=_O_U8TEXT) {
//         *_errno() = EINVAL;
//         release_ioinfo(info);
//         return -1;
//     }
//
//     if(info == &MSVCRT___badioinfo) {
//         *_errno() = EBADF;
//         return EOF;
//     }
//
//     if(mode == _O_BINARY) {
//         info->wxflag &= ~WX_TEXT;
//         ioinfo_set_textmode(info, TEXTMODE_ANSI);
//         release_ioinfo(info);
//         return ret;
//     }
//
//     info->wxflag |= WX_TEXT;
//     if(mode == _O_TEXT)
//         ioinfo_set_textmode(info, TEXTMODE_ANSI);
//     else if(mode == _O_U8TEXT)
//         ioinfo_set_textmode(info, TEXTMODE_UTF8);
//     else
//         ioinfo_set_textmode(info, TEXTMODE_UTF16LE);
//
//     release_ioinfo(info);
//     return ret;
// }
//
// /*********************************************************************
//  *		_stat64 (MSVCRT.@)
//  */
// int CDECL _stat64(const char* path, struct _stat64 * buf)
// {
//     wchar_t *pathW = NULL;
//     int ret;
//
//     if (path && !(pathW = wstrdupa_utf8(path))) return -1;
//     ret = _wstat64(pathW, buf);
//     free(pathW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_stati64 (MSVCRT.@)
//  */
// int CDECL _stati64(const char* path, struct _stati64 * buf)
// {
//   int ret;
//   struct _stat64 buf64;
//
//   ret = _stat64(path, &buf64);
//   if (!ret)
//     msvcrt_stat64_to_stati64(&buf64, buf);
//   return ret;
// }
//
// /*********************************************************************
//  *             _stat (MSVCRT.@)
//  */
// int CDECL _stat(const char* path, struct _stat * buf)
// {
//   int ret;
//   struct _stat64 buf64;
//
//   ret = _stat64( path, &buf64);
//   if (!ret)
//       msvcrt_stat64_to_stat(&buf64, buf);
//   return ret;
// }
//
//
// /*********************************************************************
//  *		_wstat64 (MSVCRT.@)
//  */
// int CDECL _wstat64(const wchar_t* path, struct _stat64 * buf)
// {
//   DWORD dw;
//   WIN32_FILE_ATTRIBUTE_DATA hfi;
//   unsigned short mode = ALL_S_IREAD;
//   int plen;
//
//   TRACE(":file (%s) buf(%p)\n", debugstr_w(path), buf);
//
//   plen = wcslen(path);
//   while (plen && path[plen-1]==' ')
//     plen--;
//
//   if (plen==2 && path[1]==':')
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   if (plen>=2 && path[plen-2]!=':' && (path[plen-1]=='\\' || path[plen-1]=='/'))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   if (!GetFileAttributesExW(path, GetFileExInfoStandard, &hfi))
//   {
//       TRACE("failed (%ld)\n", GetLastError());
//       *_errno() = ENOENT;
//       return -1;
//   }
//
//   memset(buf,0,sizeof(struct _stat64));
//
//   /* FIXME: rdev isn't drive num, despite what the docs says-what is it? */
//   if (iswalpha(*path) && path[1] == ':')
//     buf->st_dev = buf->st_rdev = towupper(*path) - 'A'; /* drive num */
//   else
//     buf->st_dev = buf->st_rdev = _getdrive() - 1;
//
//   /* Dir, or regular file? */
//   if (hfi.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
//     mode |= (_S_IFDIR | ALL_S_IEXEC);
//   else
//   {
//     mode |= _S_IFREG;
//     /* executable? */
//     if (plen > 6 && path[plen-4] == '.')  /* shortest exe: "\x.exe" */
//     {
//       ULONGLONG ext = towlower(path[plen-1]) | (towlower(path[plen-2]) << 16) |
//                                ((ULONGLONG)towlower(path[plen-3]) << 32);
//       if (ext == WCEXE || ext == WCBAT || ext == WCCMD || ext == WCCOM)
//         mode |= ALL_S_IEXEC;
//     }
//   }
//
//   if (!(hfi.dwFileAttributes & FILE_ATTRIBUTE_READONLY))
//     mode |= ALL_S_IWRITE;
//
//   buf->st_mode  = mode;
//   buf->st_nlink = 1;
//   buf->st_size  = ((__int64)hfi.nFileSizeHigh << 32) + hfi.nFileSizeLow;
//   RtlTimeToSecondsSince1970((LARGE_INTEGER *)&hfi.ftLastAccessTime, &dw);
//   buf->st_atime = dw;
//   RtlTimeToSecondsSince1970((LARGE_INTEGER *)&hfi.ftLastWriteTime, &dw);
//   buf->st_mtime = buf->st_ctime = dw;
//   TRACE("%d %d %#I64x %I64d %I64d %I64d\n", buf->st_mode, buf->st_nlink,
//           buf->st_size, buf->st_atime, buf->st_mtime, buf->st_ctime);
//   return 0;
// }
//
// /*********************************************************************
//  *		_wstati64 (MSVCRT.@)
//  */
// int CDECL _wstati64(const wchar_t* path, struct _stati64 * buf)
// {
//   int ret;
//   struct _stat64 buf64;
//
//   ret = _wstat64(path, &buf64);
//   if (!ret)
//     msvcrt_stat64_to_stati64(&buf64, buf);
//   return ret;
// }
//
// /*********************************************************************
//  *             _wstat (MSVCRT.@)
//  */
// int CDECL _wstat(const wchar_t* path, struct _stat * buf)
// {
//   int ret;
//   struct _stat64 buf64;
//
//   ret = _wstat64( path, &buf64 );
//   if (!ret) msvcrt_stat64_to_stat(&buf64, buf);
//   return ret;
// }
//
//
// /*********************************************************************
//  *		_tell (MSVCRT.@)
//  */
// __msvcrt_long CDECL _tell(int fd)
// {
//   return _lseek(fd, 0, SEEK_CUR);
// }
//
// /*********************************************************************
//  *		_telli64 (MSVCRT.@)
//  */
// __int64 CDECL _telli64(int fd)
// {
//   return _lseeki64(fd, 0, SEEK_CUR);
// }
//
// /*********************************************************************
//  *		_tempnam (MSVCRT.@)
//  */
// char *  _tempnam(const char *dir, const char *prefix)
// {
//     wchar_t *dirW = NULL, *prefixW = NULL, *retW;
//     char *ret;
//
//     if (dir && !(dirW = wstrdupa_utf8(dir))) return NULL;
//     if (prefix && !(prefixW = wstrdupa_utf8(prefix)))
//     {
//         free(dirW);
//         return NULL;
//     }
//     retW = _wtempnam(dirW, prefixW);
//     free(dirW);
//     free(prefixW);
//     /* TODO: don't do the conversion */
//     ret = astrdupw_utf8(retW);
//     free(retW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wtempnam (MSVCRT.@)
//  */
// wchar_t *  _wtempnam(const wchar_t *dir, const wchar_t *prefix)
// {
//   wchar_t tmpbuf[MAX_PATH];
//   const wchar_t *tmp_dir = _wgetenv(L"TMP");
//
//   if (tmp_dir) dir = tmp_dir;
//
//   TRACE("dir (%s) prefix (%s)\n", debugstr_w(dir), debugstr_w(prefix));
//   /* TODO: use whole prefix */
//   if (GetTempFileNameW(dir,prefix,0,tmpbuf))
//   {
//     TRACE("got name (%s)\n", debugstr_w(tmpbuf));
//     DeleteFileW(tmpbuf);
//     return _wcsdup(tmpbuf);
//   }
//   TRACE("failed (%ld)\n", GetLastError());
//   return NULL;
// }
//
// /*********************************************************************
//  *		_umask (MSVCRT.@)
//  */
// int CDECL _umask(int umask)
// {
//   int old_umask = MSVCRT_umask;
//   TRACE("(%d)\n",umask);
//   MSVCRT_umask = umask;
//   return old_umask;
// }
//
// /*********************************************************************
//  *		_utime64 (MSVCRT.@)
//  */
// int CDECL _utime64(const char* path, struct __utimbuf64 *t)
// {
//   int fd = _open(path, _O_WRONLY | _O_BINARY);
//
//   if (fd > 0)
//   {
//     int retVal = _futime64(fd, t);
//     _close(fd);
//     return retVal;
//   }
//   return -1;
// }
//
// /*********************************************************************
//  *		_utime32 (MSVCRT.@)
//  */
// int CDECL _utime32(const char* path, struct __utimbuf32 *t)
// {
//     if (t)
//     {
//         struct __utimbuf64 t64;
//         t64.actime = t->actime;
//         t64.modtime = t->modtime;
//         return _utime64( path, &t64 );
//     }
//     else
//         return _utime64( path, NULL );
// }
//
// /*********************************************************************
//  *		_wutime64 (MSVCRT.@)
//  */
// int CDECL _wutime64(const wchar_t* path, struct __utimbuf64 *t)
// {
//   int fd = _wopen(path, _O_WRONLY | _O_BINARY);
//
//   if (fd > 0)
//   {
//     int retVal = _futime64(fd, t);
//     _close(fd);
//     return retVal;
//   }
//   return -1;
// }
//
// /*********************************************************************
//  *		_wutime32 (MSVCRT.@)
//  */
// int CDECL _wutime32(const wchar_t* path, struct __utimbuf32 *t)
// {
//     if (t)
//     {
//         struct __utimbuf64 t64;
//         t64.actime = t->actime;
//         t64.modtime = t->modtime;
//         return _wutime64( path, &t64 );
//     }
//     else
//         return _wutime64( path, NULL );
// }
//
// /*********************************************************************
//  *		_write (MSVCRT.@)
//  */
// int CDECL _write(int fd, const void* buf, unsigned int count)
// {
//     ioinfo *info = get_ioinfo(fd);
//     HANDLE hand = info->handle;
//     DWORD num_written, i;
//     BOOL console = FALSE;
//
//     if (hand == INVALID_HANDLE_VALUE || fd == MSVCRT_NO_CONSOLE_FD)
//     {
//         *_errno() = EBADF;
//         release_ioinfo(info);
//         return -1;
//     }
//
//     if (ioinfo_get_textmode(info) != TEXTMODE_ANSI && count&1)
//     {
//         *_errno() = EINVAL;
//         release_ioinfo(info);
//         return -1;
//     }
//
//     /* If appending, go to EOF */
//     if (info->wxflag & WX_APPEND)
//         _lseek(fd, 0, FILE_END);
//
//     if (!(info->wxflag & WX_TEXT))
//     {
//         if (!WriteFile(hand, buf, count, &num_written, NULL)
//                 ||  num_written != count)
//         {
//             TRACE("WriteFile (fd %d, hand %p) failed-last error (%ld)\n", fd,
//                     hand, GetLastError());
//             msvcrt_set_errno(GetLastError());
//             if (GetLastError() == ERROR_ACCESS_DENIED)
//                 *_errno() = EBADF;
//             num_written = -1;
//         }
//
//         release_ioinfo(info);
//         return num_written;
//     }
//
//     if (_isatty(fd)) console = VerifyConsoleIoHandle(hand);
//     for (i = 0; i < count;)
//     {
//         const char *s = buf;
//         char lfbuf[2048];
//         DWORD j = 0;
//
//         if (ioinfo_get_textmode(info) == TEXTMODE_ANSI && console)
//         {
//             char conv[sizeof(lfbuf)];
//             size_t len = 0;
//
//
//             for (; i < count && j < sizeof(conv)-1 &&
//                     len < (sizeof(lfbuf) - 1) / sizeof(WCHAR); i++, j++, len++)
//             {
//                 if (isleadbyte((unsigned char)s[i]))
//                 {
//                     conv[j++] = s[i++];
//
//                     if (i == count)
//                     {
//                         *_errno() = EINVAL;
//                         release_ioinfo(info);
//                         return -1;
//                     }
//                 }
//                 else if (s[i] == '\n')
//                 {
//                     conv[j++] = '\r';
//                     len++;
//                 }
//                 conv[j] = s[i];
//             }
//
//             len = mbstowcs((WCHAR*)lfbuf, conv, len);
//             if (len == -1)
//             {
//                 msvcrt_set_errno(GetLastError());
//                 release_ioinfo(info);
//                 return -1;
//             }
//             j = len * 2;
//         }
//         else if (ioinfo_get_textmode(info) == TEXTMODE_ANSI)
//         {
//             for (j = 0; i < count && j < sizeof(lfbuf)-1; i++, j++)
//             {
//                 if (s[i] == '\n')
//                     lfbuf[j++] = '\r';
//                 lfbuf[j] = s[i];
//             }
//         }
//         else if (ioinfo_get_textmode(info) == TEXTMODE_UTF16LE || console)
//         {
//             for (j = 0; i < count && j < sizeof(lfbuf)-3; i++, j++)
//             {
//                 if (s[i] == '\n' && !s[i+1])
//                 {
//                     lfbuf[j++] = '\r';
//                     lfbuf[j++] = 0;
//                 }
//                 lfbuf[j++] = s[i++];
//                 lfbuf[j] = s[i];
//             }
//         }
//         else
//         {
//             char conv[sizeof(lfbuf)/4];
//
//             for (j = 0; i < count && j < sizeof(conv)-3; i++, j++)
//             {
//                 if (s[i] == '\n' && !s[i+1])
//                 {
//                     conv[j++] = '\r';
//                     conv[j++] = 0;
//                 }
//                 conv[j++] = s[i++];
//                 conv[j] = s[i];
//             }
//
//             j = WideCharToMultiByte(CP_UTF8, 0, (WCHAR*)conv, j/2, lfbuf, sizeof(lfbuf), NULL, NULL);
//             if (!j)
//             {
//                 msvcrt_set_errno(GetLastError());
//                 release_ioinfo(info);
//                 return -1;
//             }
//         }
//
//         if (console)
//         {
//             j = j/2;
//             if (!WriteConsoleW(hand, lfbuf, j, &num_written, NULL))
//                 num_written = -1;
//         }
//         else if (!WriteFile(hand, lfbuf, j, &num_written, NULL))
//         {
//             num_written = -1;
//         }
//
//         if (num_written != j)
//         {
//             TRACE("WriteFile/WriteConsoleW (fd %d, hand %p) failed-last error (%ld)\n", fd,
//                     hand, GetLastError());
//             msvcrt_set_errno(GetLastError());
//             if (GetLastError() == ERROR_ACCESS_DENIED)
//                 *_errno() = EBADF;
//             release_ioinfo(info);
//             return -1;
//         }
//     }
//
//     release_ioinfo(info);
//     return count;
// }
//
// /*********************************************************************
//  *		_putw (MSVCRT.@)
//  */
// int CDECL _putw(int val, FILE* file)
// {
//   int len;
//
//   _lock_file(file);
//   len = _write(file->_file, &val, sizeof(val));
//   if (len == sizeof(val)) {
//     _unlock_file(file);
//     return val;
//   }
//
//   file->_flag |= _IOERR;
//   _unlock_file(file);
//   return EOF;
// }
//
// /*********************************************************************
//  *		fclose (MSVCRT.@)
//  */
// int CDECL fclose(FILE* file)
// {
//   int ret;
//
//   if (!MSVCRT_CHECK_PMT(file != NULL)) return EOF;
//
//   _lock_file(file);
//   ret = _fclose_nolock(file);
//   _unlock_file(file);
//
//   return ret;
// }
//
// /*********************************************************************
//  *		_fclose_nolock (MSVCRT.@)
//  */
// int CDECL _fclose_nolock(FILE* file)
// {
//   int r, flag;
//
//   if (!MSVCRT_CHECK_PMT(file != NULL)) return EOF;
//
//   if(!(file->_flag & (_IOREAD | _IOWRT | _IORW)))
//   {
//       file->_flag = 0;
//       return EOF;
//   }
//
//   flag = file->_flag;
//   free(file->_tmpfname);
//   file->_tmpfname = NULL;
//   /* flush stdio buffers */
//   if(file->_flag & _IOWRT)
//       _fflush_nolock(file);
//   if(file->_flag & _IOMYBUF)
//       free(file->_base);
//
//   r=_close(file->_file);
//   file->_flag = 0;
//
//   return ((r == -1) || (flag & _IOERR) ? EOF : 0);
// }
//
// /*********************************************************************
//  *		feof (MSVCRT.@)
//  */
// int CDECL feof(FILE* file)
// {
//     return file->_flag & _IOEOF;
// }
//
// /*********************************************************************
//  *		ferror (MSVCRT.@)
//  */
// int CDECL ferror(FILE* file)
// {
//     return file->_flag & _IOERR;
// }
//
// /*********************************************************************
//  *		_filbuf (MSVCRT.@)
//  */
// int CDECL _filbuf(FILE* file)
// {
//     unsigned char c;
//
//     if(file->_flag & _IOSTRG)
//         return EOF;
//
//     /* Allocate buffer if needed */
//     if(!(file->_flag & (MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)))
//         msvcrt_alloc_buffer(file);
//
//     if(!(file->_flag & _IOREAD)) {
//         if(file->_flag & _IORW)
//             file->_flag |= _IOREAD;
//         else
//             return EOF;
//     }
//
//     if(!(file->_flag & (_IOMYBUF | MSVCRT__USERBUF))) {
//         int r;
//         if ((r = _read(file->_file,&c,1)) != 1) {
//             file->_flag |= (r == 0) ? _IOEOF : _IOERR;
//             return EOF;
//         }
//
//         return c;
//     } else {
//         file->_cnt = _read(file->_file, file->_base, file->_bufsiz);
//         if(file->_cnt<=0) {
//             file->_flag |= (file->_cnt == 0) ? _IOEOF : _IOERR;
//             file->_cnt = 0;
//             return EOF;
//         }
//
//         file->_cnt--;
//         file->_ptr = file->_base+1;
//         c = *(unsigned char *)file->_base;
//         return c;
//     }
// }
//
// /*********************************************************************
//  *		fgetc (MSVCRT.@)
//  */
// int CDECL fgetc(FILE* file)
// {
//     int ret;
//
//     _lock_file(file);
//     ret = _fgetc_nolock(file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fgetc_nolock (MSVCRT.@)
//  */
// int CDECL _fgetc_nolock(FILE* file)
// {
//   unsigned char *i;
//   unsigned int j;
//
//   if (file->_cnt>0) {
//     file->_cnt--;
//     i = (unsigned char *)file->_ptr++;
//     j = *i;
//   } else
//     j = _filbuf(file);
//
//   return j;
// }
//
// /*********************************************************************
//  *		_fgetchar (MSVCRT.@)
//  */
// int CDECL _fgetchar(void)
// {
//   return fgetc(stdin);
// }
//
// /*********************************************************************
//  *		fgets (MSVCRT.@)
//  */
// char *  fgets(char *s, int size, FILE* file)
// {
//   int    cc = EOF;
//   char * buf_start = s;
//
//   TRACE(":file(%p) fd (%d) str (%p) len (%d)\n",
// 	file,file->_file,s,size);
//
//   _lock_file(file);
//
//   while ((size >1) && (cc = _fgetc_nolock(file)) != EOF && cc != '\n')
//     {
//       *s++ = (char)cc;
//       size --;
//     }
//   if ((cc == EOF) && (s == buf_start)) /* If nothing read, return 0*/
//   {
//     TRACE(":nothing read\n");
//     _unlock_file(file);
//     return NULL;
//   }
//   if ((cc != EOF) && (size > 1))
//     *s++ = cc;
//   *s = '\0';
//   TRACE(":got %s\n", debugstr_a(buf_start));
//   _unlock_file(file);
//   return buf_start;
// }
//
// /*********************************************************************
//  *		fgetwc (MSVCRT.@)
//  */
// wint_t CDECL fgetwc(FILE* file)
// {
//     wint_t ret;
//
//     _lock_file(file);
//     ret = _fgetwc_nolock(file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fgetwc_nolock (MSVCRT.@)
//  */
// wint_t CDECL _fgetwc_nolock(FILE* file)
// {
//     wint_t ret;
//     int ch;
//
//     if(ioinfo_get_textmode(get_ioinfo_nolock(file->_file)) != TEXTMODE_ANSI
//             || !(get_ioinfo_nolock(file->_file)->wxflag & WX_TEXT)) {
//         char *p;
//
//         for(p=(char*)&ret; (wint_t*)p<&ret+1; p++) {
//             ch = _fgetc_nolock(file);
//             if(ch == EOF) {
//                 ret = WEOF;
//                 break;
//             }
//             *p = (char)ch;
//         }
//     }else {
//         char mbs[MB_LEN_MAX];
//         int len = 0;
//
//         ch = _fgetc_nolock(file);
//         if(ch != EOF) {
//             mbs[0] = (char)ch;
//             if(isleadbyte((unsigned char)mbs[0])) {
//                 ch = _fgetc_nolock(file);
//                 if(ch != EOF) {
//                     mbs[1] = (char)ch;
//                     len = 2;
//                 }
//             }else {
//                 len = 1;
//             }
//         }
//
//         if(!len || mbtowc(&ret, mbs, len)==-1)
//             ret = WEOF;
//     }
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_getw (MSVCRT.@)
//  */
// int CDECL _getw(FILE* file)
// {
//   char *ch;
//   int i, k;
//   unsigned int j;
//   ch = (char *)&i;
//
//   _lock_file(file);
//   for (j=0; j<sizeof(int); j++) {
//     k = _fgetc_nolock(file);
//     if (k == EOF) {
//       file->_flag |= _IOEOF;
//       _unlock_file(file);
//       return EOF;
//     }
//     ch[j] = k;
//   }
//
//   _unlock_file(file);
//   return i;
// }
//
// /*********************************************************************
//  *		getwc (MSVCRT.@)
//  */
// wint_t CDECL getwc(FILE* file)
// {
//   return fgetwc(file);
// }
//
// /*********************************************************************
//  *		_fgetwchar (MSVCRT.@)
//  */
// wint_t CDECL _fgetwchar(void)
// {
//   return fgetwc(stdin);
// }
//
// /*********************************************************************
//  *		getwchar (MSVCRT.@)
//  */
// wint_t CDECL getwchar(void)
// {
//   return _fgetwchar();
// }
//
// /*********************************************************************
//  *              fgetws (MSVCRT.@)
//  */
// wchar_t *  fgetws(wchar_t *s, int size, FILE* file)
// {
//   wint_t cc = WEOF;
//   wchar_t * buf_start = s;
//
//   TRACE(":file(%p) fd (%d) str (%p) len (%d)\n",
//         file,file->_file,s,size);
//
//   _lock_file(file);
//
//   while ((size >1) && (cc = _fgetwc_nolock(file)) != WEOF && cc != '\n')
//     {
//       *s++ = cc;
//       size --;
//     }
//   if ((cc == WEOF) && (s == buf_start)) /* If nothing read, return 0*/
//   {
//     TRACE(":nothing read\n");
//     _unlock_file(file);
//     return NULL;
//   }
//   if ((cc != WEOF) && (size > 1))
//     *s++ = cc;
//   *s = 0;
//   TRACE(":got %s\n", debugstr_w(buf_start));
//   _unlock_file(file);
//   return buf_start;
// }
//
// /*********************************************************************
//  *		_flsbuf (MSVCRT.@)
//  */
// int CDECL _flsbuf(int c, FILE* file)
// {
//     /* Flush output buffer */
//     if(!(file->_flag & (MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF))) {
//         msvcrt_alloc_buffer(file);
//     }
//
//     if(!(file->_flag & _IOWRT)) {
//         if(!(file->_flag & _IORW)) {
//             file->_flag |= _IOERR;
//             *_errno() = EBADF;
//             return EOF;
//         }
//         file->_flag |= _IOWRT;
//     }
//     if(file->_flag & _IOREAD) {
//         if(!(file->_flag & _IOEOF)) {
//             file->_flag |= _IOERR;
//             return EOF;
//         }
//         file->_cnt = 0;
//         file->_ptr = file->_base;
//         file->_flag &= ~(_IOREAD | _IOEOF);
//     }
//
//     if(file->_flag & (_IOMYBUF | MSVCRT__USERBUF)) {
//         int res = 0;
//
//         if(file->_cnt <= 0) {
//             res = msvcrt_flush_buffer(file);
//             if(res)
//                 return res;
//             file->_flag |= _IOWRT;
//             file->_cnt=file->_bufsiz;
//         }
//         *file->_ptr++ = c;
//         file->_cnt--;
//         return c&0xff;
//     } else {
//         unsigned char cc=c;
//         int len;
//         /* set _cnt to 0 for unbuffered FILEs */
//         file->_cnt = 0;
//         len = _write(file->_file, &cc, 1);
//         if (len == 1)
//             return c & 0xff;
//         file->_flag |= _IOERR;
//         return EOF;
//     }
// }
//
// /*********************************************************************
//  *		fwrite (MSVCRT.@)
//  */
// size_t CDECL fwrite(const void *ptr, size_t size, size_t nmemb, FILE* file)
// {
//     size_t ret;
//
//     _lock_file(file);
//     ret = _fwrite_nolock(ptr, size, nmemb, file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fwrite_nolock (MSVCRT.@)
//  */
// size_t CDECL _fwrite_nolock(const void *ptr, size_t size, size_t nmemb, FILE* file)
// {
//     size_t wrcnt=size * nmemb;
//     int written = 0;
//     if (size == 0)
//         return 0;
//
//     while(wrcnt) {
//         if(file->_cnt < 0) {
//             WARN("negative file->_cnt value in %p\n", file);
//             file->_flag |= _IOERR;
//             break;
//         } else if(file->_cnt) {
//             int pcnt=(file->_cnt>wrcnt)? wrcnt: file->_cnt;
//             memcpy(file->_ptr, ptr, pcnt);
//             file->_cnt -= pcnt;
//             file->_ptr += pcnt;
//             written += pcnt;
//             wrcnt -= pcnt;
//             ptr = (const char*)ptr + pcnt;
//         } else if((file->_flag & MSVCRT__NOBUF)
//                 || ((file->_flag & (_IOMYBUF | MSVCRT__USERBUF)) && wrcnt >= file->_bufsiz)
//                 || (!(file->_flag & (_IOMYBUF | MSVCRT__USERBUF)) && wrcnt >= MSVCRT_INTERNAL_BUFSIZ)) {
//             size_t pcnt;
//             int bufsiz;
//
//             if(file->_flag & MSVCRT__NOBUF)
//                 bufsiz = 1;
//             else if(!(file->_flag & (_IOMYBUF | MSVCRT__USERBUF)))
//                 bufsiz = MSVCRT_INTERNAL_BUFSIZ;
//             else
//                 bufsiz = file->_bufsiz;
//
//             pcnt = (wrcnt / bufsiz) * bufsiz;
//
//             if(msvcrt_flush_buffer(file) == EOF)
//                 break;
//
//             if(_write(file->_file, ptr, pcnt) <= 0) {
//                 file->_flag |= _IOERR;
//                 break;
//             }
//             written += pcnt;
//             wrcnt -= pcnt;
//             ptr = (const char*)ptr + pcnt;
//         } else {
//             if(_flsbuf(*(const char*)ptr, file) == EOF)
//                 break;
//             written++;
//             wrcnt--;
//             ptr = (const char*)ptr + 1;
//         }
//     }
//
//     return written / size;
// }
//
// /*********************************************************************
//  *		fputwc (MSVCRT.@)
//  */
// wint_t CDECL fputwc(wint_t wc, FILE* file)
// {
//     wint_t ret;
//
//     _lock_file(file);
//     ret = _fputwc_nolock(wc, file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fputwc_nolock (MSVCRT.@)
//  */
// wint_t CDECL _fputwc_nolock(wint_t wc, FILE* file)
// {
//     wchar_t mwc=wc;
//     ioinfo *fdinfo;
//     wint_t ret;
//
//     fdinfo = get_ioinfo_nolock(file->_file);
//
//     if((fdinfo->wxflag&WX_TEXT) && ioinfo_get_textmode(fdinfo) == TEXTMODE_ANSI) {
//         char buf[MB_LEN_MAX];
//         int char_len;
//
//         char_len = wctomb(buf, mwc);
//         if(char_len!=-1 && _fwrite_nolock(buf, char_len, 1, file)==1)
//             ret = wc;
//         else
//             ret = WEOF;
//     }else if(_fwrite_nolock(&mwc, sizeof(mwc), 1, file) == 1) {
//         ret = wc;
//     }else {
//         ret = WEOF;
//     }
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fputwchar (MSVCRT.@)
//  */
// wint_t CDECL _fputwchar(wint_t wc)
// {
//   return fputwc(wc, stdout);
// }
//
// /*********************************************************************
//  *		_wfsopen (MSVCRT.@)
//  */
// FILE *  _wfsopen(const wchar_t *path, const wchar_t *mode, int share)
// {
//   FILE* file;
//   int open_flags, stream_flags, fd;
//
//   TRACE("(%s,%s)\n", debugstr_w(path), debugstr_w(mode));
//
//   /* map mode string to open() flags. "man fopen" for possibilities. */
//   if (msvcrt_get_flags(mode, &open_flags, &stream_flags) == -1)
//       return NULL;
//
//   LOCK_FILES();
//   fd = _wsopen(path, open_flags, share, _S_IREAD | _S_IWRITE);
//   if (fd < 0)
//     file = NULL;
//   else if ((file = msvcrt_alloc_fp()) && msvcrt_init_fp(file, fd, stream_flags)
//    != -1)
//     TRACE(":fd (%d) mode (%s) FILE* (%p)\n", fd, debugstr_w(mode), file);
//   else if (file)
//   {
//     file->_flag = 0;
//     file = NULL;
//   }
//
//   TRACE(":got (%p)\n",file);
//   if (fd >= 0 && !file)
//     _close(fd);
//   UNLOCK_FILES();
//   return file;
// }
//
// /*********************************************************************
//  *		_fsopen (MSVCRT.@)
//  */
// FILE *  _fsopen(const char *path, const char *mode, int share)
// {
//     wchar_t *pathW = NULL, *modeW = NULL;
//     FILE *ret;
//
//     if (path && !(pathW = wstrdupa_utf8(path))) return NULL;
//     if (mode && !(modeW = wstrdupa_utf8(mode)))
//     {
//         free(pathW);
//         return NULL;
//     }
//
//     ret = _wfsopen(pathW, modeW, share);
//
//     free(pathW);
//     free(modeW);
//     return ret;
// }
//
// /*********************************************************************
//  *		fopen (MSVCRT.@)
//  */
// FILE *  fopen(const char *path, const char *mode)
// {
//     return _fsopen( path, mode, _SH_DENYNO );
// }
//
// /*********************************************************************
//  *              fopen_s (MSVCRT.@)
//  */
// int CDECL fopen_s(FILE** pFile,
//         const char *filename, const char *mode)
// {
//     if (!MSVCRT_CHECK_PMT(pFile != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(filename != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mode != NULL)) return EINVAL;
//
//     *pFile = fopen(filename, mode);
//
//     if(!*pFile)
//         return *_errno();
//     return 0;
// }
//
// /*********************************************************************
//  *		_wfopen (MSVCRT.@)
//  */
// FILE *  _wfopen(const wchar_t *path, const wchar_t *mode)
// {
//     return _wfsopen( path, mode, _SH_DENYNO );
// }
//
// /*********************************************************************
//  *		_wfopen_s (MSVCRT.@)
//  */
// int CDECL _wfopen_s(FILE** pFile, const wchar_t *filename,
//         const wchar_t *mode)
// {
//     if (!MSVCRT_CHECK_PMT(pFile != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(filename != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mode != NULL)) return EINVAL;
//
//     *pFile = _wfopen(filename, mode);
//
//     if(!*pFile)
//         return *_errno();
//     return 0;
// }
//
// /*********************************************************************
//  *		fputc (MSVCRT.@)
//  */
// int CDECL fputc(int c, FILE* file)
// {
//     int ret;
//
//     _lock_file(file);
//     ret = _fputc_nolock(c, file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fputc_nolock (MSVCRT.@)
//  */
// int CDECL _fputc_nolock(int c, FILE* file)
// {
//   int res;
//
//   if(file->_cnt>0) {
//     *file->_ptr++=c;
//     file->_cnt--;
//     if (c == '\n')
//     {
//       res = msvcrt_flush_buffer(file);
//       return res ? res : c;
//     }
//     else {
//       return c & 0xff;
//     }
//   } else {
//     res = _flsbuf(c, file);
//     return res;
//   }
// }
//
// /*********************************************************************
//  *		_fputchar (MSVCRT.@)
//  */
// int CDECL _fputchar(int c)
// {
//   return fputc(c, stdout);
// }
//
// /*********************************************************************
//  *		fread (MSVCRT.@)
//  */
// size_t CDECL fread(void *ptr, size_t size, size_t nmemb, FILE* file)
// {
//     size_t ret;
//
//     _lock_file(file);
//     ret = _fread_nolock(ptr, size, nmemb, file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_fread_nolock (MSVCRT.@)
//  */
// size_t CDECL _fread_nolock(void *ptr, size_t size, size_t nmemb, FILE* file)
// {
//   size_t rcnt=size * nmemb;
//   size_t read=0;
//   size_t pread=0;
//
//   if(!rcnt)
// 	return 0;
//
//   /* first buffered data */
//   if(file->_cnt>0) {
// 	int pcnt= (rcnt>file->_cnt)? file->_cnt:rcnt;
// 	memcpy(ptr, file->_ptr, pcnt);
// 	file->_cnt -= pcnt;
// 	file->_ptr += pcnt;
// 	read += pcnt ;
// 	rcnt -= pcnt ;
//         ptr = (char*)ptr + pcnt;
//   } else if(!(file->_flag & _IOREAD )) {
// 	if(file->_flag & _IORW) {
// 		file->_flag |= _IOREAD;
// 	} else {
//         return 0;
//     }
//   }
//
//   if(rcnt>0 && !(file->_flag & (MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF)))
//       msvcrt_alloc_buffer(file);
//
//   while(rcnt>0)
//   {
//     int i;
//     if (!file->_cnt && rcnt<file->_bufsiz && (file->_flag & (_IOMYBUF | MSVCRT__USERBUF))) {
//       i = _read(file->_file, file->_base, file->_bufsiz);
//       file->_ptr = file->_base;
//       if (i != -1) {
//           file->_cnt = i;
//           if (i > rcnt) i = rcnt;
//       }
//       /* If the buffer fill reaches eof but fread wouldn't, clear eof. */
//       if (i > 0 && i < file->_cnt) {
//         get_ioinfo_nolock(file->_file)->wxflag &= ~WX_ATEOF;
//         file->_flag &= ~_IOEOF;
//       }
//       if (i > 0) {
//         memcpy(ptr, file->_ptr, i);
//         file->_cnt -= i;
//         file->_ptr += i;
//       }
//     } else if (rcnt > INT_MAX) {
//       i = _read(file->_file, ptr, INT_MAX);
//     } else if (rcnt < (file->_bufsiz ? file->_bufsiz : MSVCRT_INTERNAL_BUFSIZ)) {
//       i = _read(file->_file, ptr, rcnt);
//     } else {
//       i = _read(file->_file, ptr, rcnt - rcnt % (file->_bufsiz ? file->_bufsiz : MSVCRT_INTERNAL_BUFSIZ));
//     }
//     pread += i;
//     rcnt -= i;
//     ptr = (char *)ptr+i;
//     /* expose feof condition in the flags
//      * MFC tests file->_flag for feof, and doesn't call feof())
//      */
//     if (get_ioinfo_nolock(file->_file)->wxflag & WX_ATEOF)
//         file->_flag |= _IOEOF;
//     else if (i == -1)
//     {
//         file->_flag |= _IOERR;
//         pread = 0;
//         rcnt = 0;
//     }
//     if (i < 1) break;
//   }
//   read+=pread;
//   return read / size;
// }
//
//
// /*********************************************************************
//  *		_wfreopen (MSVCRT.@)
//  *
//  */
// FILE*  _wfreopen(const wchar_t *path, const wchar_t *mode, FILE* file)
// {
//     int open_flags, stream_flags, fd;
//
//     TRACE(":path (%s) mode (%s) file (%p) fd (%d)\n", debugstr_w(path), debugstr_w(mode), file, file ? file->_file : -1);
//
//     LOCK_FILES();
//     if (file)
//     {
//         fclose(file);
//         if (msvcrt_get_flags(mode, &open_flags, &stream_flags) == -1)
//             file = NULL;
//         else if((fd = _wopen(path, open_flags, _S_IREAD | _S_IWRITE)) < 0)
//             file = NULL;
//         else if(msvcrt_init_fp(file, fd, stream_flags) == -1)
//         {
//             file->_flag = 0;
//             file = NULL;
//         }
//     }
//     UNLOCK_FILES();
//     return file;
// }
//
// /*********************************************************************
//  *      _wfreopen_s (MSVCRT.@)
//  */
// int CDECL _wfreopen_s(FILE** pFile,
//         const wchar_t *path, const wchar_t *mode, FILE* file)
// {
//     if (!MSVCRT_CHECK_PMT(pFile != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(path != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mode != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(file != NULL)) return EINVAL;
//
//     *pFile = _wfreopen(path, mode, file);
//
//     if(!*pFile)
//         return *_errno();
//     return 0;
// }
//
// /*********************************************************************
//  *      freopen (MSVCRT.@)
//  *
//  */
// FILE*  freopen(const char *path, const char *mode, FILE* file)
// {
//     FILE *ret;
//     wchar_t *pathW = NULL, *modeW = NULL;
//
//     if (path && !(pathW = wstrdupa_utf8(path))) return NULL;
//     if (mode && !(modeW = msvcrt_wstrdupa(mode)))
//     {
//         free(pathW);
//         return NULL;
//     }
//
//     ret = _wfreopen(pathW, modeW, file);
//
//     free(pathW);
//     free(modeW);
//     return ret;
// }
//
// /*********************************************************************
//  *      freopen_s (MSVCRT.@)
//  */
// errno_t CDECL freopen_s(FILE** pFile,
//         const char *path, const char *mode, FILE* file)
// {
//     if (!MSVCRT_CHECK_PMT(pFile != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(path != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(mode != NULL)) return EINVAL;
//     if (!MSVCRT_CHECK_PMT(file != NULL)) return EINVAL;
//
//     *pFile = freopen(path, mode, file);
//
//     if(!*pFile)
//         return *_errno();
//     return 0;
// }
//
// /*********************************************************************
//  *		fsetpos (MSVCRT.@)
//  */
// int CDECL fsetpos(FILE* file, fpos_t *pos)
// {
//     return _fseeki64(file,*pos,SEEK_SET);
// }
//
// /*********************************************************************
//  *		_ftelli64 (MSVCRT.@)
//  */
// __int64 CDECL _ftelli64(FILE* file)
// {
//     __int64 ret;
//
//     _lock_file(file);
//     ret = _ftelli64_nolock(file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_ftelli64_nolock (MSVCRT.@)
//  */
// __int64 CDECL _ftelli64_nolock(FILE* file)
// {
//     __int64 pos;
//
//     pos = _telli64(file->_file);
//     if(pos == -1)
//         return -1;
//     if(file->_flag & (_IOMYBUF | MSVCRT__USERBUF))  {
//         if(file->_flag & _IOWRT) {
//             pos += file->_ptr - file->_base;
//
//             if(get_ioinfo_nolock(file->_file)->wxflag & WX_TEXT) {
//                 char *p;
//
//                 for(p=file->_base; p<file->_ptr; p++)
//                     if(*p == '\n')
//                         pos++;
//             }
//         } else if(!file->_cnt) { /* nothing to do */
//         } else if(_lseeki64(file->_file, 0, SEEK_END)==pos) {
//             int i;
//
//             pos -= file->_cnt;
//             if(get_ioinfo_nolock(file->_file)->wxflag & WX_TEXT) {
//                 for(i=0; i<file->_cnt; i++)
//                     if(file->_ptr[i] == '\n')
//                         pos--;
//             }
//         } else {
//             char *p;
//
//             if(_lseeki64(file->_file, pos, SEEK_SET) != pos)
//                 return -1;
//
//             pos -= file->_bufsiz;
//             pos += file->_ptr - file->_base;
//
//             if(get_ioinfo_nolock(file->_file)->wxflag & WX_TEXT) {
//                 if(get_ioinfo_nolock(file->_file)->wxflag & WX_READNL)
//                     pos--;
//
//                 for(p=file->_base; p<file->_ptr; p++)
//                     if(*p == '\n')
//                         pos++;
//             }
//         }
//     }
//
//     return pos;
// }
//
// /*********************************************************************
//  *		ftell (MSVCRT.@)
//  */
// __msvcrt_long CDECL ftell(FILE* file)
// {
//   return _ftelli64(file);
// }
//
//
// /*********************************************************************
//  *		fgetpos (MSVCRT.@)
//  */
// int CDECL fgetpos(FILE* file, fpos_t *pos)
// {
//     *pos = _ftelli64(file);
//     if(*pos == -1)
//         return -1;
//     return 0;
// }
//
// /*********************************************************************
//  *		fputs (MSVCRT.@)
//  */
// int CDECL fputs(const char *s, FILE* file)
// {
//     size_t len = strlen(s);
//     int ret;
//
//     _lock_file(file);
//     ret = _fwrite_nolock(s, sizeof(*s), len, file) == len ? 0 : EOF;
//     _unlock_file(file);
//     return ret;
// }
//
// /*********************************************************************
//  *		fputws (MSVCRT.@)
//  */
// int CDECL fputws(const wchar_t *s, FILE* file)
// {
//     size_t i, len = wcslen(s);
//     BOOL tmp_buf;
//     int ret;
//
//     _lock_file(file);
//     if (!(get_ioinfo_nolock(file->_file)->wxflag & WX_TEXT)) {
//         ret = _fwrite_nolock(s,sizeof(*s),len,file) == len ? 0 : EOF;
//         _unlock_file(file);
//         return ret;
//     }
//
//     tmp_buf = add_std_buffer(file);
//     for (i=0; i<len; i++) {
//         if(_fputwc_nolock(s[i], file) == WEOF) {
//             if(tmp_buf) remove_std_buffer(file);
//             _unlock_file(file);
//             return WEOF;
//         }
//     }
//
//     if(tmp_buf) remove_std_buffer(file);
//     _unlock_file(file);
//     return 0;
// }
//
// /*********************************************************************
//  *		getchar (MSVCRT.@)
//  */
// int CDECL getchar(void)
// {
//   return fgetc(stdin);
// }
//
// /*********************************************************************
//  *		getc (MSVCRT.@)
//  */
// int CDECL getc(FILE* file)
// {
//   return fgetc(file);
// }
//
// /*********************************************************************
//  *		gets_s (MSVCR80.@)
//  */
// char *  gets_s(char *buf, size_t len)
// {
//     char *buf_start = buf;
//     int cc;
//
//     if (!MSVCRT_CHECK_PMT(buf != NULL)) return NULL;
//     if (!MSVCRT_CHECK_PMT(len != 0)) return NULL;
//
//     _lock_file(stdin);
//     for(cc = _fgetc_nolock(stdin);
//             len != 0 && cc != EOF && cc != '\n';
//             cc = _fgetc_nolock(stdin))
//     {
//         if (cc != '\r')
//         {
//             *buf++ = (char)cc;
//             len--;
//         }
//     }
//     _unlock_file(stdin);
//
//     if (!len)
//     {
//         *buf_start = 0;
//         _invalid_parameter(NULL, NULL, NULL, 0, 0);
//         return NULL;
//     }
//
//     if ((cc == EOF) && (buf_start == buf))
//     {
//         TRACE(":nothing read\n");
//         return NULL;
//     }
//     *buf = '\0';
//
//     TRACE("got '%s'\n", buf_start);
//     return buf_start;
// }
//
// /*********************************************************************
//  *              gets (MSVCRT.@)
//  */
// char *  gets(char *buf)
// {
//     return gets_s(buf, (-1i32) as usize);
// }
//
// /*********************************************************************
//  *		_getws (MSVCRT.@)
//  */
// wchar_t*  _getws(wchar_t* buf)
// {
//     wint_t cc;
//     wchar_t* ws = buf;
//
//     _lock_file(stdin);
//     for (cc = _fgetwc_nolock(stdin); cc != WEOF && cc != '\n';
//          cc = _fgetwc_nolock(stdin))
//     {
//         if (cc != '\r')
//             *buf++ = (wchar_t)cc;
//     }
//     _unlock_file(stdin);
//
//     if ((cc == WEOF) && (ws == buf))
//     {
//       TRACE(":nothing read\n");
//       return NULL;
//     }
//     *buf = '\0';
//
//     TRACE("got %s\n", debugstr_w(ws));
//     return ws;
// }
//
// /*********************************************************************
//  *		putc (MSVCRT.@)
//  */
// int CDECL putc(int c, FILE* file)
// {
//   return fputc(c, file);
// }
//
// /*********************************************************************
//  *		putchar (MSVCRT.@)
//  */
// int CDECL putchar(int c)
// {
//   return fputc(c, stdout);
// }
//
// /*********************************************************************
//  *		puts (MSVCRT.@)
//  */
// int CDECL puts(const char *s)
// {
//     size_t len = strlen(s);
//     int ret;
//
//     _lock_file(stdout);
//     if(_fwrite_nolock(s, sizeof(*s), len, stdout) != len) {
//         _unlock_file(stdout);
//         return EOF;
//     }
//
//     ret = _fwrite_nolock("\n",1,1,stdout) == 1 ? 0 : EOF;
//     _unlock_file(stdout);
//     return ret;
// }
//
// /*********************************************************************
//  *		_putws (MSVCRT.@)
//  */
// int CDECL _putws(const wchar_t *s)
// {
//     int ret;
//
//     _lock_file(stdout);
//     ret = fputws(s, stdout);
//     if(ret >= 0)
//         ret = _fputwc_nolock('\n', stdout);
//     _unlock_file(stdout);
//     return ret >= 0 ? 0 : WEOF;
// }
//
// /*********************************************************************
//  *		remove (MSVCRT.@)
//  */
// int CDECL remove(const char *path)
// {
//     return _unlink(path);
// }
//
// /*********************************************************************
//  *		_wremove (MSVCRT.@)
//  */
// int CDECL _wremove(const wchar_t *path)
// {
//     return _wunlink(path);
// }
//
// /*********************************************************************
//  *		rename (MSVCRT.@)
//  */
// int CDECL rename(const char *oldpath,const char *newpath)
// {
//     wchar_t *oldpathW = NULL, *newpathW = NULL;
//     int ret;
//
//     if (oldpath && !(oldpathW = wstrdupa_utf8(oldpath))) return -1;
//     if (newpath && !(newpathW = wstrdupa_utf8(newpath)))
//     {
//         free(oldpathW);
//         return -1;
//     }
//     ret = _wrename(oldpathW, newpathW);
//     free(oldpathW);
//     free(newpathW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wrename (MSVCRT.@)
//  */
// int CDECL _wrename(const wchar_t *oldpath,const wchar_t *newpath)
// {
//   TRACE(":from %s to %s\n", debugstr_w(oldpath), debugstr_w(newpath));
//   if (MoveFileExW(oldpath, newpath, MOVEFILE_COPY_ALLOWED))
//     return 0;
//   TRACE(":failed (%ld)\n", GetLastError());
//   msvcrt_set_errno(GetLastError());
//   return -1;
// }
//
// /*********************************************************************
//  *		setvbuf (MSVCRT.@)
//  */
// int CDECL setvbuf(FILE* file, char *buf, int mode, size_t size)
// {
//     if(!MSVCRT_CHECK_PMT(file != NULL)) return -1;
//     if(!MSVCRT_CHECK_PMT(mode==_IONBF || mode==_IOFBF || mode==_IOLBF)) return -1;
//     if(!MSVCRT_CHECK_PMT(mode==_IONBF || (size>=2 && size<=INT_MAX))) return -1;
//
//     _lock_file(file);
//
//     _fflush_nolock(file);
//     if(file->_flag & _IOMYBUF)
//         free(file->_base);
//     file->_flag &= ~(MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF);
//     file->_cnt = 0;
//
//     if(mode == _IONBF) {
//         file->_flag |= MSVCRT__NOBUF;
//         file->_base = file->_ptr = (char*)&file->_charbuf;
//         file->_bufsiz = 2;
//     }else if(buf) {
//         file->_base = file->_ptr = buf;
//         file->_flag |= MSVCRT__USERBUF;
//         file->_bufsiz = size;
//     }else {
//         file->_base = file->_ptr = malloc(size);
//         if(!file->_base) {
//             file->_bufsiz = 0;
//             _unlock_file(file);
//             return -1;
//         }
//
//         file->_flag |= _IOMYBUF;
//         file->_bufsiz = size;
//     }
//     _unlock_file(file);
//     return 0;
// }
//
// /*********************************************************************
//  *		setbuf (MSVCRT.@)
//  */
// void CDECL setbuf(FILE* file, char *buf)
// {
//   setvbuf(file, buf, buf ? _IOFBF : _IONBF, BUFSIZ);
// }
//
// static int tmpnam_helper(char *s, size_t size, LONG *tmpnam_unique, int tmp_max)
// {
//     char tmpstr[8];
//     char *p = s;
//     int digits;
//
//     if (!MSVCRT_CHECK_PMT(s != NULL)) return EINVAL;
//
//     if (size < 3) {
//         if (size) *s = 0;
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//     *p++ = '\\';
//     *p++ = 's';
//     size -= 2;
//     digits = msvcrt_int_to_base32(GetCurrentProcessId(), tmpstr);
//     if (digits+1 > size) {
//         *s = 0;
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//     memcpy(p, tmpstr, digits*sizeof(tmpstr[0]));
//     p += digits;
//     *p++ = '.';
//     size -= digits+1;
//
//     while(1) {
//         while ((digits = *tmpnam_unique)+1 < tmp_max) {
//             if (InterlockedCompareExchange(tmpnam_unique, digits+1, digits) == digits)
//                 break;
//         }
//
//         digits = msvcrt_int_to_base32(digits, tmpstr);
//         if (digits+1 > size) {
//             *s = 0;
//             *_errno() = ERANGE;
//             return ERANGE;
//         }
//         memcpy(p, tmpstr, digits*sizeof(tmpstr[0]));
//         p[digits] = 0;
//
//         if (GetFileAttributesA(s) == INVALID_FILE_ATTRIBUTES &&
//                 GetLastError() == ERROR_FILE_NOT_FOUND)
//             break;
//     }
//     return 0;
// }
//
// int CDECL tmpnam_s(char *s, size_t size)
// {
//     return tmpnam_helper(s, size, &tmpnam_s_unique, TMP_MAX_S);
// }
//
// /*********************************************************************
//  *		tmpnam (MSVCRT.@)
//  */
// char *  tmpnam(char *s)
// {
//   if (!s) {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     if(!data->tmpnam_buffer)
//       data->tmpnam_buffer = malloc(MAX_PATH);
//
//     s = data->tmpnam_buffer;
//   }
//
//   return tmpnam_helper(s, -1, &tmpnam_unique, TMP_MAX) ? NULL : s;
// }
//
// static int wtmpnam_helper(wchar_t *s, size_t size, LONG *tmpnam_unique, int tmp_max)
// {
//     wchar_t tmpstr[8];
//     wchar_t *p = s;
//     int digits;
//
//     if (!MSVCRT_CHECK_PMT(s != NULL)) return EINVAL;
//
//     if (size < 3) {
//         if (size) *s = 0;
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//     *p++ = '\\';
//     *p++ = 's';
//     size -= 2;
//     digits = msvcrt_int_to_base32_w(GetCurrentProcessId(), tmpstr);
//     if (digits+1 > size) {
//         *s = 0;
//         *_errno() = ERANGE;
//         return ERANGE;
//     }
//     memcpy(p, tmpstr, digits*sizeof(tmpstr[0]));
//     p += digits;
//     *p++ = '.';
//     size -= digits+1;
//
//     while(1) {
//         while ((digits = *tmpnam_unique)+1 < tmp_max) {
//             if (InterlockedCompareExchange(tmpnam_unique, digits+1, digits) == digits)
//                 break;
//         }
//
//         digits = msvcrt_int_to_base32_w(digits, tmpstr);
//         if (digits+1 > size) {
//             *s = 0;
//             *_errno() = ERANGE;
//             return ERANGE;
//         }
//         memcpy(p, tmpstr, digits*sizeof(tmpstr[0]));
//         p[digits] = 0;
//
//         if (GetFileAttributesW(s) == INVALID_FILE_ATTRIBUTES &&
//                 GetLastError() == ERROR_FILE_NOT_FOUND)
//             break;
//     }
//     return 0;
// }
//
// /*********************************************************************
//  *              _wtmpnam_s (MSVCRT.@)
//  */
// int CDECL _wtmpnam_s(wchar_t *s, size_t size)
// {
//     return wtmpnam_helper(s, size, &tmpnam_s_unique, TMP_MAX_S);
// }
//
// /*********************************************************************
//  *              _wtmpnam (MSVCRT.@)
//  */
// wchar_t *  _wtmpnam(wchar_t *s)
// {
//     if (!s) {
//         thread_data_t *data = msvcrt_get_thread_data();
//
//         if(!data->wtmpnam_buffer)
//             data->wtmpnam_buffer = malloc(sizeof(wchar_t[MAX_PATH]));
//
//         s = data->wtmpnam_buffer;
//     }
//
//     return wtmpnam_helper(s, -1, &tmpnam_unique, TMP_MAX) ? NULL : s;
// }
//
// /*********************************************************************
//  *		tmpfile (MSVCRT.@)
//  */
// FILE*  tmpfile(void)
// {
//   char *filename = _tempnam(",", "t");
//   int fd;
//   FILE* file = NULL;
//
//   LOCK_FILES();
//   fd = _open(filename, _O_CREAT | _O_BINARY | _O_RDWR | _O_TEMPORARY,
//           _S_IREAD | _S_IWRITE);
//   if (fd != -1 && (file = msvcrt_alloc_fp()))
//   {
//     if (msvcrt_init_fp(file, fd, _IORW) == -1)
//     {
//         file->_flag = 0;
//         file = NULL;
//     }
//     else file->_tmpfname = _strdup(filename);
//   }
//
//   if(fd != -1 && !file)
//       _close(fd);
//   free(filename);
//   UNLOCK_FILES();
//   return file;
// }
//
// /*********************************************************************
//  *      tmpfile_s (MSVCRT.@)
//  */
// int CDECL tmpfile_s(FILE** file)
// {
//     if (!MSVCRT_CHECK_PMT(file != NULL)) return EINVAL;
//
//     *file = tmpfile();
//     return 0;
// }
//
// static int puts_clbk_file_a(void *file, int len, const char *str)
// {
//     return fwrite(str, sizeof(char), len, file);
// }
//
// static int puts_clbk_file_w(void *file, int len, const wchar_t *str)
// {
//     int i, ret;
//
//     _lock_file(file);
//
//     if(!(get_ioinfo_nolock(((FILE*)file)->_file)->wxflag & WX_TEXT)) {
//         ret = _fwrite_nolock(str, sizeof(wchar_t), len, file);
//         _unlock_file(file);
//         return ret;
//     }
//
//     for(i=0; i<len; i++) {
//         if(_fputwc_nolock(str[i], file) == WEOF) {
//             _unlock_file(file);
//             return -1;
//         }
//     }
//
//     _unlock_file(file);
//     return len;
// }
//
// static int vfprintf_helper(DWORD options, FILE* file, const char *format,
//         _locale_t locale, va_list valist)
// {
//     printf_arg args_ctx[_ARGMAX+1];
//     BOOL tmp_buf;
//     int ret;
//
//     if(!MSVCRT_CHECK_PMT( file != NULL )) return -1;
//     if(!MSVCRT_CHECK_PMT( format != NULL )) return -1;
//
//     if(options & MSVCRT_PRINTF_POSITIONAL_PARAMS) {
//         memset(args_ctx, 0, sizeof(args_ctx));
//         ret = create_positional_ctx_a(args_ctx, format, valist);
//         if(ret < 0) {
//             _invalid_parameter(NULL, NULL, NULL, 0, 0);
//             *_errno() = EINVAL;
//             return ret;
//         } else if(!ret)
//             options &= ~MSVCRT_PRINTF_POSITIONAL_PARAMS;
//     }
//
//     _lock_file(file);
//     tmp_buf = add_std_buffer(file);
//     ret = pf_printf_a(puts_clbk_file_a, file, format, locale, options,
//             options & MSVCRT_PRINTF_POSITIONAL_PARAMS ? arg_clbk_positional : arg_clbk_valist,
//             options & MSVCRT_PRINTF_POSITIONAL_PARAMS ? args_ctx : NULL, &valist);
//     if(tmp_buf) remove_std_buffer(file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// static int vfwprintf_helper(DWORD options, FILE* file, const wchar_t *format,
//         _locale_t locale, va_list valist)
// {
//     printf_arg args_ctx[_ARGMAX+1];
//     BOOL tmp_buf;
//     int ret;
//
//     if(!MSVCRT_CHECK_PMT( file != NULL )) return -1;
//     if(!MSVCRT_CHECK_PMT( format != NULL )) return -1;
//
//     if(options & MSVCRT_PRINTF_POSITIONAL_PARAMS) {
//         memset(args_ctx, 0, sizeof(args_ctx));
//         ret = create_positional_ctx_w(args_ctx, format, valist);
//         if(ret < 0) {
//             _invalid_parameter(NULL, NULL, NULL, 0, 0);
//             *_errno() = EINVAL;
//             return ret;
//         } else if(!ret)
//             options &= ~MSVCRT_PRINTF_POSITIONAL_PARAMS;
//     }
//
//     _lock_file(file);
//     tmp_buf = add_std_buffer(file);
//     ret = pf_printf_w(puts_clbk_file_w, file, format, locale, options,
//             options & MSVCRT_PRINTF_POSITIONAL_PARAMS ? arg_clbk_positional : arg_clbk_valist,
//             options & MSVCRT_PRINTF_POSITIONAL_PARAMS ? args_ctx : NULL, &valist);
//     if(tmp_buf) remove_std_buffer(file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *    _vfprintf_s_l (MSVCRT.@)
//  */
// int CDECL _vfprintf_s_l(FILE* file, const char *format,
//         _locale_t locale, va_list valist)
// {
//     return vfprintf_helper(MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER, file, format, locale, valist);
// }
//
// /*********************************************************************
//  *    _vfwprintf_s_l (MSVCRT.@)
//  */
// int CDECL _vfwprintf_s_l(FILE* file, const wchar_t *format,
//         _locale_t locale, va_list valist)
// {
//     return vfwprintf_helper(MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER, file, format, locale, valist);
// }
//
// /*********************************************************************
//  *		vfprintf (MSVCRT.@)
//  */
// int CDECL vfprintf(FILE* file, const char *format, va_list valist)
// {
//     return vfprintf_helper(0, file, format, NULL, valist);
// }
//
// /*********************************************************************
//  *		vfprintf_s (MSVCRT.@)
//  */
// int CDECL vfprintf_s(FILE* file, const char *format, va_list valist)
// {
//     return _vfprintf_s_l(file, format, NULL, valist);
// }
//
// /*********************************************************************
//  *		vfwprintf (MSVCRT.@)
//  */
// int CDECL vfwprintf(FILE* file, const wchar_t *format, va_list valist)
// {
//     return vfwprintf_helper(0, file, format, NULL, valist);
// }
//
// /*********************************************************************
//  *		vfwprintf_s (MSVCRT.@)
//  */
// int CDECL vfwprintf_s(FILE* file, const wchar_t *format, va_list valist)
// {
//     return _vfwprintf_s_l(file, format, NULL, valist);
// }
//
//
// /*********************************************************************
//  *    _vfprintf_l (MSVCRT.@)
//  */
// int CDECL _vfprintf_l(FILE* file, const char *format,
//         _locale_t locale, va_list valist)
// {
//     return vfprintf_helper(0, file, format, locale, valist);
// }
//
// /*********************************************************************
//  *              _vfwprintf_l (MSVCRT.@)
//  */
// int CDECL _vfwprintf_l(FILE* file, const wchar_t *format,
//         _locale_t locale, va_list valist)
// {
//     return vfwprintf_helper(0, file, format, locale, valist);
// }
//
// /*********************************************************************
//  *    _vfprintf_p_l (MSVCRT.@)
//  */
// int CDECL _vfprintf_p_l(FILE* file, const char *format,
//         _locale_t locale, va_list valist)
// {
//     return vfprintf_helper(MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
//             file, format, locale, valist);
// }
//
// /*********************************************************************
//  *    _vfprintf_p (MSVCRT.@)
//  */
// int CDECL _vfprintf_p(FILE* file, const char *format, va_list valist)
// {
//     return _vfprintf_p_l(file, format, NULL, valist);
// }
//
// /*********************************************************************
//  *    _vfwprintf_p_l (MSVCRT.@)
//  */
// int CDECL _vfwprintf_p_l(FILE* file, const wchar_t *format,
//         _locale_t locale, va_list valist)
// {
//     return vfwprintf_helper(MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
//             file, format, locale, valist);
// }
//
// /*********************************************************************
//  *    _vfwprintf_p (MSVCRT.@)
//  */
// int CDECL _vfwprintf_p(FILE* file, const wchar_t *format, va_list valist)
// {
//     return _vfwprintf_p_l(file, format, NULL, valist);
// }
//
// /*********************************************************************
//  *		vprintf (MSVCRT.@)
//  */
// int CDECL vprintf(const char *format, va_list valist)
// {
//   return vfprintf(stdout,format,valist);
// }
//
// /*********************************************************************
//  *		vprintf_s (MSVCRT.@)
//  */
// int CDECL vprintf_s(const char *format, va_list valist)
// {
//   return vfprintf_s(stdout,format,valist);
// }
//
// /*********************************************************************
//  *		vwprintf (MSVCRT.@)
//  */
// int CDECL vwprintf(const wchar_t *format, va_list valist)
// {
//   return vfwprintf(stdout,format,valist);
// }
//
// /*********************************************************************
//  *		vwprintf_s (MSVCRT.@)
//  */
// int CDECL vwprintf_s(const wchar_t *format, va_list valist)
// {
//   return vfwprintf_s(stdout,format,valist);
// }
//
// /*********************************************************************
//  *		fprintf (MSVCRT.@)
//  */
// int __cdecl fprintf(FILE* file, const char *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vfprintf(file, format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		fprintf_s (MSVCRT.@)
//  */
// int __cdecl fprintf_s(FILE* file, const char *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vfprintf_s(file, format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *    _fprintf_l (MSVCRT.@)
//  */
// int __cdecl _fprintf_l(FILE* file, const char *format, _locale_t locale, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, locale);
//     res = _vfprintf_l(file, format, locale, valist);
//     va_end(valist);
//     return res;
// }
//
//
// /*********************************************************************
//  *    _fprintf_p (MSVCRT.@)
//  */
// int __cdecl _fprintf_p(FILE* file, const char *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = _vfprintf_p_l(file, format, NULL, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *    _fprintf_p_l (MSVCRT.@)
//  */
// int __cdecl _fprintf_p_l(FILE* file, const char *format, _locale_t locale, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, locale);
//     res = _vfprintf_p_l(file, format, locale, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *    _fprintf_s_l (MSVCRT.@)
//  */
// int __cdecl _fprintf_s_l(FILE* file, const char *format, _locale_t locale, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, locale);
//     res = _vfprintf_s_l(file, format, locale, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		fwprintf (MSVCRT.@)
//  */
// int __cdecl fwprintf(FILE* file, const wchar_t *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vfwprintf(file, format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		fwprintf_s (MSVCRT.@)
//  */
// int __cdecl fwprintf_s(FILE* file, const wchar_t *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vfwprintf_s(file, format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *              _fwprintf_l (MSVCRT.@)
//  */
// int __cdecl _fwprintf_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, locale);
//     res = _vfwprintf_l(file, format, locale, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *              _fwprintf_p (MSVCRT.@)
//  */
// int __cdecl _fwprintf_p(FILE* file, const wchar_t *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = _vfwprintf_p_l(file, format, NULL, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *              _fwprintf_p_l (MSVCRT.@)
//  */
// int __cdecl _fwprintf_p_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, locale);
//     res = _vfwprintf_p_l(file, format, locale, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *              _fwprintf_s_l (MSVCRT.@)
//  */
// int __cdecl _fwprintf_s_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, locale);
//     res = _vfwprintf_s_l(file, format, locale, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		printf (MSVCRT.@)
//  */
// int __cdecl printf(const char *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vfprintf(stdout, format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		printf_s (MSVCRT.@)
//  */
// int __cdecl printf_s(const char *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vprintf_s(format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		ungetc (MSVCRT.@)
//  */
// int CDECL ungetc(int c, FILE * file)
// {
//     int ret;
//
//     if(!MSVCRT_CHECK_PMT(file != NULL)) return EOF;
//
//     _lock_file(file);
//     ret = _ungetc_nolock(c, file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *		_ungetc_nolock (MSVCRT.@)
//  */
// int CDECL _ungetc_nolock(int c, FILE * file)
// {
//     if(!MSVCRT_CHECK_PMT(file != NULL)) return EOF;
//
//     if (c == EOF || !(file->_flag&_IOREAD ||
//                 (file->_flag&_IORW && !(file->_flag&_IOWRT))))
//         return EOF;
//
//     if((!(file->_flag & (MSVCRT__NOBUF | _IOMYBUF | MSVCRT__USERBUF))
//                 && msvcrt_alloc_buffer(file))
//             || (!file->_cnt && file->_ptr==file->_base))
//         file->_ptr++;
//
//     if(file->_ptr>file->_base) {
//         file->_ptr--;
//         if(file->_flag & _IOSTRG) {
//             if(*file->_ptr != c) {
//                 file->_ptr++;
//                 return EOF;
//             }
//         }else {
//             *file->_ptr = c;
//         }
//         file->_cnt++;
//         file->_flag &= ~(_IOERR | _IOEOF);
//         file->_flag |= _IOREAD;
//         return c;
//     }
//
//     return EOF;
// }
//
// /*********************************************************************
//  *              ungetwc (MSVCRT.@)
//  */
// wint_t CDECL ungetwc(wint_t wc, FILE * file)
// {
//     wint_t ret;
//
//     if(!MSVCRT_CHECK_PMT(file != NULL)) return WEOF;
//
//     _lock_file(file);
//     ret = _ungetwc_nolock(wc, file);
//     _unlock_file(file);
//
//     return ret;
// }
//
// /*********************************************************************
//  *              _ungetwc_nolock (MSVCRT.@)
//  */
// wint_t CDECL _ungetwc_nolock(wint_t wc, FILE * file)
// {
//     wchar_t mwc = wc;
//
//     if(!MSVCRT_CHECK_PMT(file != NULL)) return WEOF;
//     if (wc == WEOF)
//         return WEOF;
//
//     if(ioinfo_get_textmode(get_ioinfo_nolock(file->_file)) != TEXTMODE_ANSI
//             || !(get_ioinfo_nolock(file->_file)->wxflag & WX_TEXT)) {
//         unsigned char * pp = (unsigned char *)&mwc;
//         int i;
//
//         for(i=sizeof(wchar_t)-1;i>=0;i--) {
//             if(pp[i] != _ungetc_nolock(pp[i],file))
//                 return WEOF;
//         }
//     }else {
//         char mbs[MB_LEN_MAX];
//         int len;
//
//         len = wctomb(mbs, mwc);
//         if(len == -1)
//             return WEOF;
//
//         for(len--; len>=0; len--) {
//             if(mbs[len] != _ungetc_nolock(mbs[len], file))
//                 return WEOF;
//         }
//     }
//
//     return mwc;
// }
//
// /*********************************************************************
//  *		wprintf (MSVCRT.@)
//  */
// int __cdecl wprintf(const wchar_t *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vwprintf(format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		wprintf_s (MSVCRT.@)
//  */
// int __cdecl wprintf_s(const wchar_t *format, ...)
// {
//     va_list valist;
//     int res;
//     va_start(valist, format);
//     res = vwprintf_s(format, valist);
//     va_end(valist);
//     return res;
// }
//
// /*********************************************************************
//  *		_getmaxstdio (MSVCRT.@)
//  */
// int CDECL _getmaxstdio(void)
// {
//     return MSVCRT_max_streams;
// }
//
// /*********************************************************************
//  *		_setmaxstdio (MSVCRT.@)
//  */
// int CDECL _setmaxstdio(int newmax)
// {
//     TRACE("%d\n", newmax);
//
//     if(newmax<_IOB_ENTRIES || newmax>MSVCRT_MAX_FILES || newmax<MSVCRT_stream_idx)
//         return -1;
//
//     MSVCRT_max_streams = newmax;
//     return MSVCRT_max_streams;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
