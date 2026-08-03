//! The single import surface the whole-C-file translations in `src/msvcrt/` compile against.
//!
//! Everything here is transcribed from a C declaration in the translated TU's own include
//! closure — never inferred from a call site. That distinction is the whole point: a wrong
//! parameter type in an `extern` block is not a build error. It links, it runs, and it reads
//! the stack one slot off forever. `cargo check` cannot see it, the difftest can only see it
//! if the fuzzer happens to hit the divergent input, and the C header states it for free. So
//! each block below names the header it came from, and a signature that is not in a header
//! does not go in.
//!
//! Three things deliberately do NOT appear:
//!
//! * **`InterlockedIncrement`** — an `__INTRIN_INLINE` in `vcruntime/mingw32/intrin_x86.h`,
//!   not an import. Declaring it `extern` yields an undefined symbol at link (the mistake
//!   mainline just fixed for `__debugbreak`, `1c3884c8349`). It is a Rust `fn` over
//!   `core::sync::atomic` below.
//! * **The 19 CRT entry points this crate already defines** (`memcpy`, `strlen`, `strcmp`, …)
//!   are still declared here as `extern`. They live in private `mod`s in `lib.rs`, so they
//!   are unreachable by path — but they are `#[no_mangle]`, so the extern declaration binds
//!   to this crate's own definition at link. That is the intended cohab behaviour: `mbcs.rs`
//!   calls the same `strlen` msvcrt.dll exports.
//! * **Anything bindgen already provides.** `msvcrt-sys` carries the state structs and 183
//!   constants (`EINVAL`, `LCMAP_*`, `CT_CTYPE*`, `_MBC_*`, …); re-declaring one here would
//!   fork it from the header snapshot the layout guardians check.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use core::ffi::{c_char, c_int, c_uint, c_void};
pub use msvcrt_sys::*;

// ── types bindgen did not reach ────────────────────────────────────────────────────────────
// `size_t` is `unsigned int`/`unsigned __int64` per arch, which IS `usize`. bindgen renders
// every USE of it as `usize` and emits no alias, so the translated signatures that kept the C
// spelling have nothing to resolve against.
pub type size_t = usize;
pub type WORD = u16;
pub type uintptr_t = usize;

// ── constants bindgen would not fold ───────────────────────────────────────────────────────
// Each is a macro, so it has no symbol for bindgen to snapshot; the value is the header's.
/// `minwindef.h`: `#define FALSE 0`. Typed `u8` because the two uses are `BOOL`-shaped
/// locals that the C assigns from `_ismbblead*`, which the translation carries as `u8`.
pub const FALSE: u8 = 0;
/// `mbstring.h`: `#define _NLSCMPERROR ((unsigned int)0x7fffffff)`. Every use in `mbcs.c`
/// is a `return` from an `int` function, where the C conversion is value-preserving.
pub const _NLSCMPERROR: c_int = 0x7fff_ffff;
/// `include/stdlib.h`: `#define _TRUNCATE ((size_t)-1)`.
pub const _TRUNCATE: size_t = size_t::MAX;
/// `mtdll.h`: `#define _MB_CP_LOCK 25`.
pub const _MB_CP_LOCK: c_int = 25;
/// `include/wctype.h`: `#define WEOF (wint_t)(0xFFFF)`.
pub const WEOF: wint_t = 0xFFFF;

/// `include/mbctype.h:51` — re-typed, and this shadows the island deliberately.
///
/// The four `_MB_CP_*` macros are one `#define` block and all four are `int` in C, which is
/// why `_setmbcp`'s `switch(cp)` accepts them together. bindgen types a macro from its VALUE,
/// so the three negative ones come back `i32` and this one — being 0 — comes back `u32`,
/// splitting the family. A `u32` constant cannot be a `match` arm against an `int` scrutinee,
/// and a cast is not allowed in a pattern, so the arm is unwritable as generated.
///
/// Fixed here rather than at the island: bindgen's `--default-macro-constant-type=signed`
/// would re-type all 176 non-negative constants at once, which is a much larger claim than
/// the one header line that is actually wrong.
pub const _MB_CP_SBCS: c_int = 0;

// ── macros that must stay macros ───────────────────────────────────────────────────────────
// A C debug/parameter-check macro lowered to an out-of-line `fn` is an optimization barrier
// that uncloaks latent ABI mismatches into `ud2` (field manual §9.65). These also must not
// evaluate what the C does not: `MSVCRT_INVALID_PMT` drops its first argument entirely.

/// `msvcrt.h:376` — `#define MSVCRT_INVALID_PMT(x,err) (*_errno() = (err), _invalid_parameter(NULL, NULL, NULL, 0, 0))`
///
/// The message argument is NOT in the expansion, so the C never evaluates it and neither does
/// this. The `let _ =` would be wrong: it would evaluate an expression the C discards.
#[macro_export]
macro_rules! MSVCRT_INVALID_PMT {
    ($x:expr, $err:expr $(,)?) => {{
        *$crate::prelude::_errno() = $err;
        $crate::prelude::_invalid_parameter(
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            0,
            0,
        );
    }};
}

/// `msvcrt.h:377` — `#define MSVCRT_CHECK_PMT_ERR(x,err) ((x) || (MSVCRT_INVALID_PMT(0,(err)), FALSE))`
///
/// Yields the C `int` 1/0 that `||` produces, because the translated call sites compare it
/// with `== 0` exactly as the C compares it with `!`.
#[macro_export]
macro_rules! MSVCRT_CHECK_PMT_ERR {
    ($x:expr, $err:expr $(,)?) => {
        if $x {
            1i32
        } else {
            $crate::MSVCRT_INVALID_PMT!(0, $err);
            0i32
        }
    };
}

/// `msvcrt.h:378` — `#define MSVCRT_CHECK_PMT(x) MSVCRT_CHECK_PMT_ERR((x), EINVAL)`
#[macro_export]
macro_rules! MSVCRT_CHECK_PMT {
    ($x:expr $(,)?) => {
        $crate::MSVCRT_CHECK_PMT_ERR!($x, ($crate::prelude::EINVAL) as i32)
    };
}

/// `minwindef.h:191` — `#define LOBYTE(w) ((BYTE)((DWORD_PTR)(w) & 0xff))`
///
/// Yields `u32`, not `u8`: the C result is a `BYTE` that integer-promotes to `int` in every
/// use, and `mbcs.c` relies on that promotion (`(HIBYTE(c) - 0x21)/2 + 0x81) << 8` would
/// wrap at 8 bits otherwise). Masking to `0xff` first makes the `DWORD_PTR` width — 32-bit
/// on i386, 64-bit on amd64/arm64 — unobservable, so one spelling is right on all three.
#[macro_export]
macro_rules! LOBYTE {
    ($w:expr $(,)?) => {
        (($w as u32) & 0xff)
    };
}

/// `minwindef.h:192` — `#define HIBYTE(w) ((BYTE)(((DWORD_PTR)(w) >> 8) & 0xff))`
#[macro_export]
macro_rules! HIBYTE {
    ($w:expr $(,)?) => {
        ((($w as u32) >> 8) & 0xff)
    };
}

/// `minwindef.h:183` — `#define min(a, b) (((a) < (b)) ? (a) : (b))`
#[macro_export]
macro_rules! min {
    ($a:expr, $b:expr $(,)?) => {
        if $a < $b {
            $a
        } else {
            $b
        }
    };
}

pub use crate::{HIBYTE, LOBYTE, MSVCRT_CHECK_PMT, MSVCRT_CHECK_PMT_ERR, MSVCRT_INVALID_PMT, min};

// ── the CRT (cdecl) ────────────────────────────────────────────────────────────────────────
// From `dll/win32/msvcrt/include/{string,stdlib,ctype,corecrt_malloc,corecrt_wstring,
// corecrt_wctype,stdio}.h` and `dll/win32/msvcrt/{msvcrt,mtdll}.h`, verbatim.
extern "C" {
    // include/stdlib.h:130
    pub fn _errno() -> *mut c_int;
    // include/stdlib.h:264
    pub fn _invalid_parameter(
        expr: *const wchar_t,
        func: *const wchar_t,
        file: *const wchar_t,
        line: c_uint,
        arg: uintptr_t,
    );

    // include/corecrt_malloc.h:31 / :46
    pub fn malloc(size: size_t) -> *mut c_void;
    pub fn free(ptr: *mut c_void);

    // include/corecrt_wstring.h — NEVER hand-write these in Rust; LLVM folds a byte-copy
    // loop back into a call to the function itself (AGENTS.md). This crate defines them
    // under `#![no_builtins]`; here we merely import the symbol.
    pub fn memset(dst: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: size_t) -> c_int;

    // include/stdlib.h:191 — time.c's `_tzset` reads the TZ variable through it.
    pub fn getenv(name: *const c_char) -> *mut c_char;

    // include/string.h
    pub fn strlen(s: *const c_char) -> size_t;
    pub fn strnlen(s: *const c_char, n: size_t) -> size_t;
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn strstr(hay: *const c_char, needle: *const c_char) -> *mut c_char;
    pub fn strpbrk(s: *const c_char, accept: *const c_char) -> *mut c_char;
    pub fn strncmp(a: *const c_char, b: *const c_char, n: size_t) -> c_int;
    pub fn strncat(dst: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char;
    pub fn strtok_s(s: *mut c_char, delim: *const c_char, ctx: *mut *mut c_char) -> *mut c_char;
    pub fn _stricmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn _strnicmp(a: *const c_char, b: *const c_char, n: size_t) -> c_int;
    pub fn _strncoll_l(a: *const c_char, b: *const c_char, n: size_t, l: _locale_t) -> c_int;
    pub fn _strnicoll_l(a: *const c_char, b: *const c_char, n: size_t, l: _locale_t) -> c_int;
    pub fn _strnset(s: *mut c_char, c: c_int, n: size_t) -> *mut c_char;
    pub fn _strrev(s: *mut c_char) -> *mut c_char;
    pub fn _strset(s: *mut c_char, c: c_int) -> *mut c_char;

    // include/stdio.h — declaring and CALLING a C-variadic is stable Rust; only *defining*
    // one needs nightly.
    pub fn sprintf(buf: *mut c_char, format: *const c_char, ...) -> c_int;
    pub fn _snprintf(buf: *mut c_char, count: size_t, format: *const c_char, ...) -> c_int;

    // include/ctype.h:33/:35/:44, include/corecrt_wctype.h
    pub fn _tolower_l(c: c_int, l: _locale_t) -> c_int;
    pub fn _toupper_l(c: c_int, l: _locale_t) -> c_int;
    pub fn _isprint_l(c: c_int, l: _locale_t) -> c_int;
    pub fn isleadbyte(c: c_int) -> c_int;
    pub fn _isleadbyte_l(c: c_int, l: _locale_t) -> c_int;
    // `u32`, NOT `wint_t` — and that is not a style choice. This crate DEFINES all
    // nine (the `isw!` macro in lib.rs), and it defines them taking `u32`, because
    // that is what `_iswctype_l` takes. `wint_t` is `unsigned short`, so declaring
    // them here as `wint_t` made every call through this block disagree with the
    // definition it resolves to: the caller writes 16 bits and the callee reads 32,
    // with the upper half undefined per the Windows x64 ABI. It has been benign only
    // because rustc happens to zero-extend the argument. `fpnum.rs` had already
    // noticed and routed around it with a private `u32` shim; `scan_extern_arity`
    // cannot see it because the ARITY matches. Declaration now matches definition.
    pub fn _iswalnum_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswalpha_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswdigit_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswgraph_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswlower_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswprint_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswpunct_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswspace_l(c: u32, l: _locale_t) -> c_int;
    pub fn _iswupper_l(c: u32, l: _locale_t) -> c_int;

    // mtdll.h:44/:45
    pub fn _lock(locknum: c_int);
    pub fn _unlock(locknum: c_int);

    // msvcrt.h — msvcrt's own internals, all cdecl.
    pub fn free_mbcinfo(mbcinfo: pthreadmbcinfo);
    pub fn locale_to_sname(
        locale: *const c_char,
        sname: *mut u16,
        sname_match: *mut BOOL,
        buf: *mut WCHAR,
    ) -> BOOL;
    pub fn __crtLCMapStringA(
        lcid: LCID,
        mapflags: DWORD,
        src: *const c_char,
        srclen: c_int,
        dst: *mut c_char,
        dstlen: c_int,
        codepage: c_uint,
        xflag: c_int,
    ) -> c_int;

    // msvcrt.h:229 — `extern _locale_t MSVCRT_locale;`
    pub static mut MSVCRT_locale: _locale_t;
}

// ── Win32 (stdcall on i386, cdecl elsewhere — that is exactly `extern "system"`) ────────────
// From `sdk/include/psdk/winnls.h`. Getting the convention wrong here is the project's #1
// historical i386 crash cause, so it is `system`, never `C`.
extern "system" {
    // winnls.h:1087
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: *const c_char,
        cbMultiByte: c_int,
        lpWideCharStr: *mut u16,
        cchWideChar: c_int,
    ) -> c_int;
    // winnls.h — `int WINAPI WideCharToMultiByte(UINT,DWORD,LPCWSTR,int,LPSTR,int,LPCSTR,LPBOOL);`
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: *const u16,
        cchWideChar: c_int,
        lpMultiByteStr: *mut c_char,
        cbMultiByte: c_int,
        lpDefaultChar: *const c_char,
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
    // winnls.h:1079
    pub fn LCMapStringW(
        Locale: LCID,
        dwMapFlags: DWORD,
        lpSrcStr: *const u16,
        cchSrc: c_int,
        lpDestStr: *mut u16,
        cchDest: c_int,
    ) -> c_int;
    // winnls.h:972 — returns BOOL, and `lpCharType` is LPWORD.
    pub fn GetStringTypeW(
        dwInfoType: DWORD,
        lpSrcStr: *const u16,
        cchSrc: c_int,
        lpCharType: *mut WORD,
    ) -> BOOL;
    // winnls.h:758
    pub fn CompareStringA(
        Locale: LCID,
        dwCmpFlags: DWORD,
        lpString1: *const c_char,
        cchCount1: c_int,
        lpString2: *const c_char,
        cchCount2: c_int,
    ) -> c_int;
    // winnls.h — `BOOL WINAPI GetCPInfo(_In_ UINT, _Out_ LPCPINFO);`
    pub fn GetCPInfo(CodePage: UINT, lpCPInfo: *mut CPINFO) -> BOOL;
    pub fn GetACP() -> UINT;
    pub fn GetOEMCP() -> UINT;
    // winnls.h — `LCID WINAPI LocaleNameToLCID(_In_ LPCWSTR, _In_ DWORD);`
    pub fn LocaleNameToLCID(lpName: *const u16, dwFlags: DWORD) -> LCID;
}

// ── intrinsics ─────────────────────────────────────────────────────────────────────────────

/// `InterlockedIncrement` is `_InterlockedIncrement`, an `__INTRIN_INLINE` in
/// `sdk/include/vcruntime/mingw32/intrin_x86.h` — a compiler intrinsic, **not** an imported
/// symbol. An `extern` declaration would compile and then fail to link.
///
/// `SeqCst` is the faithful lowering: the Win32 Interlocked family is documented as a full
/// barrier, and it is what the intrinsic emits (`lock xadd` on x86, `ldaxr`/`stlxr` on arm64).
/// Returns the INCREMENTED value, as the intrinsic does — `fetch_add` returns the previous.
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut c_int) -> c_int {
    use core::sync::atomic::{AtomicI32, Ordering};
    (*(addend as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}

// ══ file.c — the stream layer ══════════════════════════════════════════════════════════════
//
// Everything below is transcribed from a header in file.c's own include closure, per this
// file's rule. The bulk of the vocabulary (FILE, _O_*, _IO*, _S_I*, the Win32 file API and
// ERROR_* families) is DERIVED by bindgen in msvcrt-sys and deliberately absent here.

/// MSVC's fixed-width 64-bit type (`vcruntime.h`). Not `long long`-by-arch: it is 64 bits on
/// every target, which is why the C spells the `_*i64` entry points with it.
pub type __int64 = i64;

// ── msvcrt.h's internal `FILE._flag` bits ─────────────────────────────────────────────────
// These are macros, so bindgen has no symbol to snapshot.
//
// `MSVCRT__NOBUF` is version-gated and the two arms DISAGREE: `0x0400` under
// `_MSVCR_VER >= 140`, `_IONBF` (0x0004) under the `#else` this build takes (msvcrt.h:47/51).
// Spelled in terms of the island's `_IONBF` rather than as a literal, so it cannot drift from
// the header the way a transcribed `0x0400` silently would — the same trap `_IOMYBUF` sets
// (0x40 vs 0x08), which is why neither is written as a number here.
pub const MSVCRT__NOBUF: u32 = _IONBF;
/// `msvcrt.h:336` — outside the version gate.
pub const MSVCRT__USERBUF: u32 = 0x0100;
/// `msvcrt.h:337` — outside the version gate.
pub const MSVCRT__IOCOMMIT: u32 = 0x4000;

/// `msvcrt.h:327` — `#define MSVCRT_NO_CONSOLE_FD (-2)`.
pub const MSVCRT_NO_CONSOLE_FD: c_int = -2;
/// `msvcrt.h:328` — `#define MSVCRT_NO_CONSOLE ((HANDLE)MSVCRT_NO_CONSOLE_FD)`.
pub const MSVCRT_NO_CONSOLE: HANDLE = MSVCRT_NO_CONSOLE_FD as isize as HANDLE;

/// `winbase.h:294` — `#define INVALID_HANDLE_VALUE (HANDLE)(-1)`. A pointer cast, so bindgen
/// emits no var for it; file.c compares fd-table handles against it constantly.
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

// `GetLastError` is NOT declared here. It is WINAPI, so a hand-written `extern "C"` block
// links on amd64/arm64 and fails only on i386 — which is exactly what it did. It comes from
// the msvcrt-sys island now, where the convention is derived from winbase.h instead of
// asserted here. Re-glob'd via `pub use msvcrt_sys::*` above.
extern "C" {
    /// `msvcrt.h:237` — `void msvcrt_set_errno(int);`. Lives in errno.c, which is still C.
    /// Genuinely cdecl — a CRT-internal helper, not a WINAPI import — so this block is right.
    pub fn msvcrt_set_errno(err: c_int);
}

// ── msvcrt.h's UTF-8 <-> wide helpers ─────────────────────────────────────────────────────
//
// `static inline` in msvcrt.h (lines 422-464), so there is NO symbol to import — an `extern`
// here would be an undefined reference at link, the mistake this file's header warns about
// for `InterlockedIncrement`. Carried as Rust fns instead, transcribed from that header with
// the `_MSVCR_VER >= 140` arm collapsed:
//
//   * `get_aw_cp()` is a CONSTANT `CP_ACP` at _MSVCR_VER=0 — it consults the locale only in
//     the >=140 arm, which this build does not compile. Writing what the NAME suggests (a
//     locale-sensitive codepage) would be wrong in the quiet direction.
//   * the narrow->wide direction passes `MB_PRECOMPOSED`, not 0.
#[inline]
fn get_aw_cp() -> UINT {
    CP_ACP
}

/// `msvcrt.h:440` — `static inline wchar_t* wstrdupa_utf8(const char *str)`.
#[inline]
pub unsafe fn wstrdupa_utf8(str_: *const c_char) -> *mut wchar_t {
    let len = MultiByteToWideChar(get_aw_cp(), MB_PRECOMPOSED, str_, -1, core::ptr::null_mut(), 0);
    if len == 0 {
        return core::ptr::null_mut();
    }
    let wstr = malloc(len as size_t * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if wstr.is_null() {
        return core::ptr::null_mut();
    }
    MultiByteToWideChar(get_aw_cp(), MB_PRECOMPOSED, str_, -1, wstr, len);
    wstr
}

/// `msvcrt.h:452` — `static inline char* astrdupw_utf8(const wchar_t *wstr)`.
#[inline]
pub unsafe fn astrdupw_utf8(wstr: *const wchar_t) -> *mut c_char {
    let len = WideCharToMultiByte(
        get_aw_cp(),
        0,
        wstr,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    if len == 0 {
        return core::ptr::null_mut();
    }
    let str_ = malloc(len as size_t) as *mut c_char;
    if str_.is_null() {
        return core::ptr::null_mut();
    }
    WideCharToMultiByte(
        get_aw_cp(),
        0,
        wstr,
        -1,
        str_,
        len,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    str_
}

/// A `va_list` cursor, BY VALUE.
///
/// On all three of our Windows targets a `va_list` is a plain `char*` walking the flat
/// argument-slot area, which is exactly why a fixed-arity function that *receives* an
/// already-started cursor is ordinary stable Rust and needs no `c_variadic`. This is the
/// same type `printf_shells.rs` uses; see its module header for the full reasoning,
/// including why each shell takes its own `let mut ap = valist` copy before handing
/// `&mut ap` down (the C parameter IS a private copy, so a callee advancing the cursor
/// must not disturb its caller's).
///
/// NOT `*mut __va_list_tag`. That spelling is the SysV/Linux model, where a `va_list` is
/// `__va_list_tag[1]` and an extra indirection deep; c2rust emits it because it anchors on
/// the host. It is wrong for our targets and the type does not exist in the island — the
/// translation of file.c arrived with both spellings mixed, and they are normalised here.
pub type va_list = *mut u8;
