//! msvcrt-crt-rust — scoped Rust cohab body for msvcrt.dll (the user-mode CRT).
//!
//! Each `#[no_mangle]` here overrides the C copy in `dll/win32/msvcrt/*.c`
//! via lld first-match (`--allow-multiple-definition`, this .a linked before
//! `msvcrt_static`). Only the families we've flipped are defined; everything
//! else stays C. The formatting logic is NOT reimplemented — it delegates to
//! `cnt_rust::core_rs`, the difftest-pinned single source of truth shared with
//! libcntpr. The thin ptr/NUL marshalling below mirrors cnt-rust's generated
//! `cabi` shims verbatim (`cnt-rust/src/cabi/generated.rs`); the eventual
//! zero-duplication mechanism is generator-emitted family scoping (tracked in
//! the lane plan, §8).
//!
//! Contract note: these preserve the CRT "misfeatures" exactly — no radix
//! validation, no buffer-size check, no NULL guard on the narrow variants;
//! sign only for radix==10 (else the value's unsigned bit pattern at the
//! declared width). Verified against msvcrt's `ltoa_helper`.
#![no_std]
// LOAD-BEARING. This crate defines `#[no_mangle] memset`/`memcpy`/`memcmp`/…
// — the very symbols LLVM's loop-idiom pass rewrites store/copy/compare loops
// INTO. Without `no_builtins`, `memset`'s fill loop is recognized and lowered to
// a call to `memset`: an infinite self-call. (The previous body dodged this with
// a per-byte `write_volatile`, which is unvectorizable by construction and cost
// ~8x throughput — it blew the net suite's timeouts and got wave 2 reverted off
// master. `no_builtins` is the right tool: it stops the *recognition* while
// leaving the loop free to be widened/vectorized normally.)
#![no_builtins]
#![allow(clippy::missing_safety_doc)]

use cnt_rust::core_rs::radix::format_radix;
// The string family and the helpers it shares with the rest of this crate live
// in crt-portable-rust, so ucrtbase.dll can link the SAME bodies without
// depending on this crate. See crt-portable-rust/Cargo.toml.
use crt_portable_rust::nz;
pub use crt_portable_rust::{strcat, strcpy, strcspn, strlen, strncat, strncpy, strnlen, strpbrk, strspn};

// ── Wave 1: int↔string radix family ─────────────────────────────────
// Narrow, signed 64-bit.
#[no_mangle]
pub unsafe extern "C" fn _i64toa(value: i64, string: *mut u8, radix: i32) -> *mut u8 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() } else { value as u64 };
    let f = format_radix(mag, radix as u32, neg);
    let mut o = 0usize;
    if f.negative { *string = b'-'; o = 1; }
    let mut i = 0;
    while i < f.ndigits { *string.add(o + i) = f.digits[i]; i += 1; }
    *string.add(o + f.ndigits) = 0;
    string
}

// Wide, signed 64-bit (NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _i64tow(value: i64, string: *mut u16, radix: i32) -> *mut u16 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() } else { value as u64 };
    let f = format_radix(mag, radix as u32, neg);
    if !string.is_null() {
        let mut o = 0usize;
        if f.negative { *string = b'-' as u16; o = 1; }
        let mut i = 0;
        while i < f.ndigits { *string.add(o + i) = f.digits[i] as u16; i += 1; }
        *string.add(o + f.ndigits) = 0;
    }
    string
}

// Narrow, signed 32-bit (_itoa == _ltoa: LP32 width).
#[no_mangle]
pub unsafe extern "C" fn _itoa(value: i32, string: *mut u8, radix: i32) -> *mut u8 {
    _ltoa(value, string, radix)
}

// Wide, signed 32-bit.
#[no_mangle]
pub unsafe extern "C" fn _itow(value: i32, string: *mut u16, radix: i32) -> *mut u16 {
    _ltow(value, string, radix)
}

// Narrow, signed long (32-bit LP32): neg+radix!=10 → u32 reinterpret.
#[no_mangle]
pub unsafe extern "C" fn _ltoa(value: i32, string: *mut u8, radix: i32) -> *mut u8 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() as u64 } else { value as u32 as u64 };
    let f = format_radix(mag, radix as u32, neg);
    let mut o = 0usize;
    if f.negative { *string = b'-'; o = 1; }
    let mut i = 0;
    while i < f.ndigits { *string.add(o + i) = f.digits[i]; i += 1; }
    *string.add(o + f.ndigits) = 0;
    string
}

// Wide, signed long (32-bit LP32; NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _ltow(value: i32, string: *mut u16, radix: i32) -> *mut u16 {
    let neg = radix == 10 && value < 0;
    let mag = if neg { value.unsigned_abs() as u64 } else { value as u32 as u64 };
    let f = format_radix(mag, radix as u32, neg);
    if !string.is_null() {
        let mut o = 0usize;
        if f.negative { *string = b'-' as u16; o = 1; }
        let mut i = 0;
        while i < f.ndigits { *string.add(o + i) = f.digits[i] as u16; i += 1; }
        *string.add(o + f.ndigits) = 0;
    }
    string
}

// Narrow, unsigned 64-bit (pure unsigned, never negative).
#[no_mangle]
pub unsafe extern "C" fn _ui64toa(value: u64, string: *mut u8, radix: i32) -> *mut u8 {
    let f = format_radix(value, radix as u32, false);
    let mut i = 0;
    while i < f.ndigits { *string.add(i) = f.digits[i]; i += 1; }
    *string.add(f.ndigits) = 0;
    string
}

// Wide, unsigned 64-bit (NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _ui64tow(value: u64, string: *mut u16, radix: i32) -> *mut u16 {
    let f = format_radix(value, radix as u32, false);
    if !string.is_null() {
        let mut i = 0;
        while i < f.ndigits { *string.add(i) = f.digits[i] as u16; i += 1; }
        *string.add(f.ndigits) = 0;
    }
    string
}

// Narrow, unsigned long (32-bit).
#[no_mangle]
pub unsafe extern "C" fn _ultoa(value: u32, string: *mut u8, radix: i32) -> *mut u8 {
    let f = format_radix(value as u64, radix as u32, false);
    let mut i = 0;
    while i < f.ndigits { *string.add(i) = f.digits[i]; i += 1; }
    *string.add(f.ndigits) = 0;
    string
}

// Wide, unsigned long (32-bit; NULL dst accepted/returned).
#[no_mangle]
pub unsafe extern "C" fn _ultow(value: u32, string: *mut u16, radix: i32) -> *mut u16 {
    let f = format_radix(value as u64, radix as u32, false);
    if !string.is_null() {
        let mut i = 0;
        while i < f.ndigits { *string.add(i) = f.digits[i] as u16; i += 1; }
        *string.add(f.ndigits) = 0;
    }
    string
}

// ── helpers copied verbatim from cnt-rust `cabi/mod.rs` (pub(crate) there,
//    so not importable). Same code = same semantics the harness graded.
#[allow(dead_code)]
// Wine's ERR/FIXME/WARN/TRACE for the whole crate; `#[macro_use]` must precede the
// modules that call them (macro_rules! is textually scoped).
#[macro_use]
mod winedebug;

mod h {
    // Moved to crt-portable-rust with the string family that needs them there;
    // re-exported so every other family here reaches them unchanged.
    pub(super) use crt_portable_rust::h::{cstr_slice, cstr_slice_bounded, raw_mut_slice};
    #[inline]
    pub(super) unsafe fn wcstr_slice<'a>(p: *const u16) -> &'a [u16] {
        if p.is_null() {
            return &[];
        }
        let mut n = 0usize;
        while *p.add(n) != 0 {
            n += 1;
        }
        core::slice::from_raw_parts(p, n)
    }
    #[inline]
    pub(super) unsafe fn wcstr_slice_bounded<'a>(p: *const u16, cap: usize) -> &'a [u16] {
        if p.is_null() {
            return &[];
        }
        let mut n = 0usize;
        while n < cap && *p.add(n) != 0 {
            n += 1;
        }
        core::slice::from_raw_parts(p, n)
    }
    #[inline]
    pub(super) unsafe fn raw_slice<'a>(p: *const u8, n: usize) -> &'a [u8] {
        if p.is_null() || n == 0 {
            return &[];
        }
        core::slice::from_raw_parts(p, n)
    }
    #[inline]
    pub(super) unsafe fn wraw_mut_slice<'a>(p: *mut u16, n: usize) -> &'a mut [u16] {
        if p.is_null() || n == 0 {
            return &mut [];
        }
        core::slice::from_raw_parts_mut(p, n)
    }
    #[inline]
    pub(super) unsafe fn raw_move(dst: *mut u8, src: *const u8, n: usize) {
        if dst as usize <= src as usize || dst as usize >= src as usize + n {
            let mut i = 0;
            while i < n {
                *dst.add(i) = *src.add(i);
                i += 1;
            }
        } else {
            let mut i = n;
            while i > 0 {
                i -= 1;
                *dst.add(i) = *src.add(i);
            }
        }
    }

    /// `strchr(delim, c) != NULL` for a NON-ZERO `c` — the tokenizers' delimiter
    /// test. Deliberately NOT strchr: the C's `strchr(delim, c)` also matches
    /// delim's own terminator when `c == 0`, and every call site guards that with
    /// a preceding `*str &&`. Spelling the guard into the helper means a future
    /// caller cannot forget it.
    #[inline]
    pub(super) unsafe fn byte_in(delim: *const u8, c: u8) -> bool {
        if c == 0 {
            return false;
        }
        let mut p = delim;
        while *p != 0 {
            if *p == c {
                return true;
            }
            p = p.add(1);
        }
        false
    }

    /// `byte_in`'s wide twin.
    #[inline]
    pub(super) unsafe fn wchar_in(delim: *const u16, c: u16) -> bool {
        if c == 0 {
            return false;
        }
        let mut p = delim;
        while *p != 0 {
            if *p == c {
                return true;
            }
            p = p.add(1);
        }
        false
    }
}

// ── wave 2: the functions msvcrt-difftest graded MATCH vs the GENUINE
//    dll/win32/msvcrt/string.c. Bodies are cnt-rust's own cabi shims
//    VERBATIM (only `crate::`->`cnt_rust::` + helper import repointed),
//    so the harness verdict transfers exactly. Do not hand-edit.
//    NOT flipped (verdict DIVERGE — cnt-rust signed-char vs msvcrt
//    unsigned-char; breaks on bytes >= 0x80):
//      strcmp strncmp _stricmp _strnicmp _memicmp memchr _memccpy _strnset
mod m_atoi64 {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `__int64 _atoi64(const char *nptr)` -> `_strtoi64_l(nptr, NULL, 10, NULL)`
    ///
    /// msvcrt's `_atoi64` is NOT libcntpr's. The kernel one wraps on overflow and
    /// has no error channel; msvcrt's routes straight through `_strtoi64_l`, so it
    /// **saturates** at _I64_MAX/_I64_MIN, sets errno=ERANGE, and fires the
    /// invalid-parameter handler on NULL. It was flipped onto the kernel's
    /// `parse_i64_wrapping` — a VALUE divergence on overflow, not merely an errno
    /// one, and classify missed it because `gen_str` essentially never emits a
    /// 19-digit run. `gen_num_str` now does. (§9.66)
    #[no_mangle]
    pub unsafe extern "C" fn _atoi64(nptr: *const u8) -> i64 {
        crate::strtoi64_contract(nptr, core::ptr::null_mut(), 10, core::ptr::null_mut())
    }

}
pub use m_atoi64::*;

mod m_strlwr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *_strlwr(char *_String)`
    /// preserved misfeature: ASCII-only (literal 'A'..'Z'); NO NULL guard (<string.h>, not precomp) → NULL crashes in C, never fuzzed; signed-char compare is moot for ASCII range
    #[no_mangle]
    pub unsafe extern "C" fn _strlwr(_String: *mut u8) -> *mut u8 {
        let n = cstr_slice(_String as *const u8).len();
        let s = raw_mut_slice(_String, n);
        cnt_rust::core_rs::casemap::lower_ascii(s);
        _String
    }

}
pub use m_strlwr::*;

mod m_strrev {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *_strrev(char *s)`
    /// preserved misfeature: len 0/1 natural no-op (C walks to NUL then swaps inward, b<e guards); NO NULL guard (precomp but while(*e) derefs) → NULL crashes in C, never fuzzed
    #[no_mangle]
    pub unsafe extern "C" fn _strrev(s: *mut u8) -> *mut u8 {
        let __n = cstr_slice(s as *const u8).len();
        let __buf = raw_mut_slice(s, __n);
        // NOTE: cnt-rust's shim calls `core_rs::strmut::reverse`, which is
        // `buf.reverse()`. That pulls core's cross-crate `#[inline]` generic
        // `swap_nonoverlapping`, which under the ORIGINAL build flags was left
        // un-inlined AND un-emitted → undefined symbol at link. An explicit swap
        // loop is semantically identical (both are an in-place reversal) and
        // pulls no generic, so it is kept.
        //
        // (That original note claimed `-C no-prepopulate-passes` "must NOT be
        // dropped". It HAS been dropped — deliberately. It disables rustc's
        // optimization pipeline outright, which compiled memset/memcpy/memcmp to
        // a function call per byte and got wave 2 reverted off master. See the
        // long comment in dll/win32/msvcrt/CMakeLists.txt. Do not re-add it.)
        if __n > 1 {
            let (mut i, mut j) = (0usize, __n - 1);
            while i < j {
                let t = __buf[i];
                __buf[i] = __buf[j];
                __buf[j] = t;
                i += 1;
                j -= 1;
            }
        }
        s
    }

}
pub use m_strrev::*;

mod m_strset {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *_strset(char *szToFill, int szFill)`
    /// preserved misfeature: szFill int truncated to char (low byte); empty → no writes; NO NULL guard → NULL crashes in C, never fuzzed
    #[no_mangle]
    pub unsafe extern "C" fn _strset(szToFill: *mut u8, szFill: i32) -> *mut u8 {
        let __n = cstr_slice(szToFill as *const u8).len();
        let __buf = raw_mut_slice(szToFill, __n);
        cnt_rust::core_rs::strmut::fill(__buf, szFill as u8);
        szToFill
    }

}
pub use m_strset::*;

mod m_strupr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *_strupr(char *_String)`
    /// preserved misfeature: ASCII-only; NULL-guarded: _invalid_parameter(NULL,L"_strupr",..) then return NULL (handler is a relaxed no-op slot under _LIBCNT_, observable result = NULL)
    #[no_mangle]
    pub unsafe extern "C" fn _strupr(_String: *mut u8) -> *mut u8 {
        if _String.is_null() { return core::ptr::null_mut(); }
        let n = cstr_slice(_String as *const u8).len();
        let s = raw_mut_slice(_String, n);
        cnt_rust::core_rs::casemap::upper_ascii(s);
        _String
    }

}
pub use m_strupr::*;

mod matoi {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `int atoi(const char *s)`
    /// preserved misfeature: NO overflow detection: i64 wrapping accumulate, then (int) truncation; leading isspace skipped; stops at first non-digit; NULL→0
    #[no_mangle]
    pub unsafe extern "C" fn atoi(s: *const u8) -> i32 {
        let s_s = cstr_slice(s);
        let (v, _of) = cnt_rust::core_rs::radix::parse_i64_wrapping(s_s);
        v as i32 as i32
    }

}
pub use matoi::*;

mod matol {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `long atol(const char *str)`
    /// preserved misfeature: atol = (long)_atoi64; long is 32-bit (Win i386) → i64 wrap then i32 trunc, no overflow error
    #[no_mangle]
    pub unsafe extern "C" fn atol(str: *const u8) -> i32 {
        let str_s = cstr_slice(str);
        let (v, _of) = cnt_rust::core_rs::radix::parse_i64_wrapping(str_s);
        v as i32 as i32
    }

}
pub use matol::*;

mod mmemcpy {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `void *memcpy(void *dst, const void *src, size_t n)`
    /// preserved misfeature: overlap-SAFE memcpy (textbook libc: overlap is UB). ReactOS memcpy.c handles it like memmove — must replicate or kernel callers relying on it break
    #[no_mangle]
    pub unsafe extern "C" fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void {
        raw_move(dst as *mut u8, src as *const u8, n);
        dst
    }

}
pub use mmemcpy::*;

mod mmemset {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `void* __cdecl memset(void* src, int val, size_t count)`
    /// preserved misfeature: none (textbook; val truncated to u8)
    ///
    /// ⚠️ PERF IS PART OF THE CONTRACT HERE. This is msvcrt.dll's `memset`,
    /// served to every user-mode caller in the system. The original flip was a
    /// per-byte `core::ptr::write_volatile` loop over a *slice* — semantically
    /// perfect (20k fuzz cases MATCH) and ~8x slower than the C, because
    /// `write_volatile` is **unvectorizable by construction**: the compiler may
    /// not widen, merge or elide the stores. That shipped, blew the net suite's
    /// 120s marker timeouts (71s -> 208s) and got wave 2 reverted off master.
    /// Pinned by `msvcrt-difftest --test perf`.
    ///
    /// Three constraints, all load-bearing:
    ///  * **no `write_bytes`/`slice::fill`** — they lower to `llvm.memset`, which
    ///    becomes a call to `memset`: infinite self-recursion in this very fn.
    ///  * **`#![no_builtins]` on the crate** — otherwise LLVM's loop-idiom pass
    ///    recognizes the fill below and turns it back into that same self-call.
    ///  * **raw pointers, not a slice** — the §378 DCE miscompile (LLVM deducing
    ///    `captures(none)` on the slice param and deleting the stores, leaving an
    ///    infinite spin) came in through the *slice* signature. That class is
    ///    toolchain-fixed on LLVM >= 21, but writing through the raw pointer keeps
    ///    us off the path entirely — which is why the volatile crutch can go.
    #[no_mangle]
    pub unsafe extern "C" fn memset(src: *mut core::ffi::c_void, val: i32, count: usize) -> *mut core::ffi::c_void {
        let p = src as *mut u8;
        let b = val as u8;
        let w = u64::from_ne_bytes([b; 8]);
        let mut i = 0usize;
        while i + 8 <= count {
            (p.add(i) as *mut u64).write_unaligned(w);
            i += 8;
        }
        while i < count {
            *p.add(i) = b;
            i += 1;
        }
        src
    }

}
pub use mmemset::*;


mod mstrchr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *strchr(const char *s, int c)`
    /// preserved misfeature: the terminating NUL is a valid target: c==0 returns ptr to the NUL (Some(len)), not NULL; c truncated to char
    ///
    /// ⚠️ Single-pass — a PERF contract (see the note on `strcmp`). The slice
    /// form (`find_char(cstr_slice(s), c)`) scans to the NUL to build the slice
    /// and THEN searches it: two passes where the C's `do { … } while (*str++)`
    /// does one. The perf gate measured that at 2.01x and failed it.
    #[no_mangle]
    pub unsafe extern "C" fn strchr(s: *const u8, c: i32) -> *mut u8 {
        if s.is_null() {
            return core::ptr::null_mut();
        }
        let t = c as u8;
        let mut p = s;
        loop {
            if *p == t {
                return p as *mut u8; // NUL is a valid target when t == 0
            }
            if *p == 0 {
                return core::ptr::null_mut();
            }
            p = p.add(1);
        }
    }

}
pub use mstrchr::*;








mod mstrrchr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *strrchr(const char *s, int c)`
    /// preserved misfeature: last occurrence; c==0 → ptr to terminating NUL (Some(len)); c truncated to char
    #[no_mangle]
    /// Single-pass — see `strchr`. C: `do { if (*str == c) ret = str; } while (*str++);`
    pub unsafe extern "C" fn strrchr(s: *const u8, c: i32) -> *mut u8 {
        if s.is_null() {
            return core::ptr::null_mut();
        }
        let t = c as u8;
        let mut p = s;
        let mut ret = core::ptr::null_mut();
        loop {
            if *p == t {
                ret = p as *mut u8; // NUL is a valid target when t == 0
            }
            if *p == 0 {
                return ret;
            }
            p = p.add(1);
        }
    }

}
pub use mstrrchr::*;


mod mstrstr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *strstr(const char *hay, const char *needle)`
    /// preserved misfeature: empty needle → returns hay (Some(0)), not NULL
    #[no_mangle]
    pub unsafe extern "C" fn strstr(hay: *const u8, needle: *const u8) -> *mut u8 {
        let hay_s = cstr_slice(hay);
        let needle_s = cstr_slice(needle);
        match cnt_rust::core_rs::string::find_sub(hay_s, needle_s) {
            Some(i) => hay.add(i) as *mut u8,
            None => core::ptr::null_mut(),
        }
    }

}
pub use mstrstr::*;

mod mstrtol {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `long strtol(const char *nptr, char **endptr, int base)` -> `_strtol_l(.., NULL)`
    ///
    /// Parses as i64 through msvcrt's engine, then clamps to the 32-bit Windows
    /// `long` with ERANGE (string.c `_strtol_l`). NOT the BSD strtol: that is a
    /// different function, and using it here is what shipped this without its
    /// errno contract, and writing `*endptr` on a rejected base (§9.66).
    #[no_mangle]
    pub unsafe extern "C" fn strtol(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> i32 {
        let ret = crate::strtoi64_contract(nptr, endptr, base, core::ptr::null_mut());
        if ret > i32::MAX as i64 {
            *crate::_errno() = crate::ERANGE;
            i32::MAX
        } else if ret < i32::MIN as i64 {
            *crate::_errno() = crate::ERANGE;
            i32::MIN
        } else {
            ret as i32
        }
    }

}
pub use mstrtol::*;

mod mstrtoul {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `unsigned long strtoul(const char* nptr, char** end, int base)` -> `_strtoul_l(.., NULL)`
    ///
    /// preserved misfeature: `_strtoul_l` parses with the **signed** `_strtoi64_l`
    /// and then clamps ASYMMETRICALLY — and its underflow arm returns **1**:
    ///
    ///     if(ret > ULONG_MAX)              { ret = ULONG_MAX; errno = ERANGE; }
    ///     else if(ret < -(__int64)ULONG_MAX) { ret = 1;       errno = ERANGE; }
    ///
    /// Not 0, not ULONG_MAX — one. Bug-for-bug with Microsoft; no reimplementation
    /// would ever derive it. Anything in range falls through and is truncated to
    /// 32 bits, which is how `strtoul("-1")` yields 0xFFFFFFFF.
    #[no_mangle]
    pub unsafe extern "C" fn strtoul(nptr: *const u8, end: *mut *mut u8, base: i32) -> u32 {
        const ULONG_MAX: i64 = 4294967295;
        let ret = crate::strtoi64_contract(nptr, end, base, core::ptr::null_mut());
        let v = if ret > ULONG_MAX {
            *crate::_errno() = crate::ERANGE;
            ULONG_MAX
        } else if ret < -ULONG_MAX {
            *crate::_errno() = crate::ERANGE;
            1
        } else {
            ret
        };
        v as u32
    }

}
pub use mstrtoul::*;


// ── Wave 3: the unsigned-char contract family ───────────────────────
//
// These six were the `msvcrt-difftest` DIVERGE/SIGN-ONLY set. All of them
// were blocked by ONE root cause: cnt-rust's `core_rs` speaks libcntpr's
// **signed-char** convention, msvcrt speaks **unsigned-char**. They agree on
// ASCII and flip sign on any byte >= 0x80.
//
// The fix is a *contract adapter*, not a reimplementation: `Elem::udiff` +
// `string::cmp_u`/`ncmp_u` give cnt-rust's identical loops the unsigned
// element difference, and the marshalling below mirrors cnt-rust's proven
// `cabi` shims. The kernel keeps `Elem::diff`; both contracts stay pinned by
// their own harness (`cnt-difftest` / `msvcrt-difftest`).
//
// NOT flipped, deliberately: `_stricmp` / `_strnicmp` / `_memicmp`. Fixing
// their signedness makes the harness report MATCH, but that is a harness
// BLIND SPOT — msvcrt's bodies consult `get_locinfo()->lc_handle[LC_CTYPE]`
// and fold through `_tolower_l`, i.e. they are **locale-dependent**, while
// cnt-rust's fold is locale-blind ASCII. They coincide only in the "C" locale
// — which is exactly what the harness pins. Correct under the default locale,
// wrong the moment an app calls `setlocale()`. They belong to Phase D (locale)
// and must NOT be flipped on a green MATCH alone. See the lane plan.

/// ⚠️ **NULL is an EMPTY STRING here, not a fault. This is load-bearing.**
///
/// Every wrapper in this crate used to reach its input through `cstr_slice` /
/// `wcstr_slice`, which NULL-guard (`if p.is_null() { return &[] }`). So the
/// shipped, gate-green behaviour of `strcmp`/`strcpy`/`strcat`/`strspn`/… has
/// always been *NULL behaves like ""* — more forgiving than the C, which
/// dereferences and dies.
///
/// The single-pass rewrites (the perf fix — see `strcmp`) walk raw pointers and
/// so silently DROPPED that tolerance. Nothing caught it: the differential fuzz
/// generates real strings and never passes NULL, so it reported 45/45 MATCH
/// while `msvcrt.dll` had started faulting on inputs it used to survive. The net
/// suite went 15/15 -> 13/15 and an A/B control against the previous commit is
/// what proved it (field manual §9.64).
///
/// Redirecting NULL to a static NUL restores the old semantics exactly — an
/// empty string compares/copies/spans exactly as `&[]` did — for one predictable
/// branch. Do NOT "simplify" these away to match the C.
static EMPTY_W: [u16; 1] = [0];
#[inline]
fn wnz(p: *const u16) -> *const u16 {
    if p.is_null() {
        EMPTY_W.as_ptr()
    } else {
        p
    }
}

/// `cstr_copy`'s wide twin.
///
/// # Safety
/// `dst` must have room for `wcslen(src) + 1` elements.
#[inline]
unsafe fn wcstr_copy(mut dst: *mut u16, mut src: *const u16) {
    loop {
        let w = *src;
        *dst = w;
        if w == 0 {
            return;
        }
        dst = dst.add(1);
        src = src.add(1);
    }
}

#[inline]
fn clamp(d: i32) -> i32 {
    if d > 0 {
        1
    } else if d < 0 {
        -1
    } else {
        0
    }
}

/// msvcrt `strcmp`: unsigned-char compare, **clamped** to -1/0/1
/// (`if ((unsigned char)*str1 > (unsigned char)*str2) return 1; ...`).
///
/// ⚠️ **Single-pass, on purpose — this is a PERF contract.** The obvious
/// spelling, `clamp(cmp_u(cstr_slice(a), cstr_slice(b)))`, is THREE passes:
/// `cstr_slice` scans `a` to its NUL, then scans `b`, and only then does `cmp_u`
/// walk them both. The C is ONE pass and never computes a length at all.
///
/// A slice-based core *cannot* express that — building a `&[u8]` requires
/// knowing the length, and knowing the length requires exactly the scan the C
/// skips. So the compare family is transcribed directly here rather than routed
/// through cnt-rust. Same call `memcmp` below already makes (hand-written
/// word-wise, because cnt's byte-zip `mem::cmp` measured 10.3x off the C).
/// Measured: the 3-pass form ran 3.8-4.8x slower than the C on the wide twin.
#[no_mangle]
pub unsafe extern "C" fn strcmp(a: *const u8, b: *const u8) -> i32 {
    let (mut p, mut q) = (crate::nz(a), crate::nz(b));
    while *p != 0 && *p == *q {
        p = p.add(1);
        q = q.add(1);
    }
    // unsigned compare — the msvcrt contract (libcntpr's `char` is signed).
    match (*p).cmp(&*q) {
        core::cmp::Ordering::Greater => 1,
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
    }
}

/// msvcrt `strncmp`: unsigned-char compare — but the return contract is
/// **arch-dependent**, and this is the one place in the flipped set where it
/// is. msvcrt.dll builds with `_MSVCR_VER=0`, so string.c's
/// `#if defined(_WIN64) || defined(_UCRT) || _MSVCR_VER == 70 || ... >= 110`
/// reduces to plain `_WIN64`:
///   * amd64 / arm64 (`_WIN64`) → **clamped** -1/0/1;
///   * i386                     → the **raw** unsigned difference.
///
/// The differential harness cannot see this: its reference TU is compiled for
/// the *host* (x86_64 linux, where `_WIN64` is undefined), so it grades the
/// i386 arm on every arch. The 64-bit arm below is therefore derived by
/// reading the C, not by fuzzing it — the harness's `msvcrt=-17` (an
/// unclamped value) is the proof it took the `#else` branch.
#[no_mangle]
pub unsafe extern "C" fn strncmp(a: *const u8, b: *const u8, len: usize) -> i32 {
    let d = strncmp_raw(a, b, len);
    #[cfg(target_pointer_width = "64")]
    {
        clamp(d)
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        d
    }
}

/// The i386 arm of [`strncmp`] — the raw unsigned difference — exposed so the
/// differential harness can grade it directly.
///
/// It is also the *basis* of the `_WIN64` arm: both C branches run after the
/// identical `while` loop and then compare the **same two bytes**, so
/// `_WIN64` is exactly `sign(#else)`. That identity is what lets one host
/// reference compile grade both shipped contracts (the harness models the
/// 64-bit C branch as `clamp(ref_strncmp(..))`).
///
/// Single-pass for the same PERF reason as [`strcmp`] — the bounded slice form
/// scanned both inputs before comparing anything.
#[inline]
pub unsafe fn strncmp_raw(a: *const u8, b: *const u8, len: usize) -> i32 {
    if len == 0 {
        return 0;
    }
    // C: `while (--len && *str1 && *str1 == *str2) { str1++; str2++; }`
    // (pre-decrement: the loop body runs at most len-1 times, and the compare
    // below then reads the len'th pair.)
    let (mut p, mut q, mut k) = (crate::nz(a), crate::nz(b), len);
    loop {
        k -= 1;
        if k == 0 || *p == 0 || *p != *q {
            break;
        }
        p = p.add(1);
        q = q.add(1);
    }
    (*p as i32) - (*q as i32)
}

/// msvcrt `memcmp`: unsigned compare, **clamped** to -1/0/1
/// (`memcmp_bytes`: `return *p1 > *p2 ? 1 : -1;`) — the lone SIGN-ONLY verdict.
///
/// ⚠️ **Word-wise on purpose — this is a perf contract, not just a value one.**
/// Delegating to `cnt_rust::core_rs::mem::cmp` is semantically exact but is a
/// byte-at-a-time `zip` loop: measured **10.3x slower** than the C, which
/// compares 64-bit blocks (`memcmp_blocks`). `memcmp` is a system-wide CRT
/// primitive, so that is a regression even though every value matches — the same
/// class that got wave 2's `memset` reverted off master. Pinned by
/// `msvcrt-difftest --test perf` (>2x slower than the C = FAIL).
///
/// The first differing byte is located from the XOR of the two words:
/// little-endian, so the LOWEST set bit is the lowest differing byte index.
/// All three targets are LE; asserted below rather than assumed.
#[no_mangle]
pub unsafe extern "C" fn memcmp(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    n: usize,
) -> i32 {
    const _: () = assert!(cfg!(target_endian = "little"), "byte-index-from-XOR is LE-only");
    let (a, b) = (s1 as *const u8, s2 as *const u8);
    let mut i = 0usize;
    while i + 8 <= n {
        let x = (a.add(i) as *const u64).read_unaligned();
        let y = (b.add(i) as *const u64).read_unaligned();
        if x != y {
            let j = i + ((x ^ y).trailing_zeros() / 8) as usize;
            return if *a.add(j) > *b.add(j) { 1 } else { -1 };
        }
        i += 8;
    }
    while i < n {
        let (xa, yb) = (*a.add(i), *b.add(i));
        if xa != yb {
            return if xa > yb { 1 } else { -1 };
        }
        i += 1;
    }
    0
}

/// msvcrt `memchr`: matches `*p == (unsigned char)c`. cnt-rust's `mem::chr`
/// matches `(b as i8 as i32) == c` (libcntpr's signed-char `memchr.c`), so a
/// byte 0x90 matches only `c == -112` there. No core change is needed — the
/// two are exactly reconciled by handing `chr` the sign-extended form of the
/// truncated byte, which is what msvcrt's `(unsigned char)c` cast means.
#[no_mangle]
pub unsafe extern "C" fn memchr(
    s: *const core::ffi::c_void,
    c: i32,
    n: usize,
) -> *mut core::ffi::c_void {
    let hay = h::raw_slice(s as *const u8, n);
    match cnt_rust::core_rs::mem::chr(hay, (c as u8) as i8 as i32) {
        Some(i) => (s as *mut u8).add(i) as *mut core::ffi::c_void,
        None => core::ptr::null_mut(),
    }
}

/// msvcrt `_memccpy`:
/// `while (n--) if ((*d++ = *s++) == (unsigned char)c) return d; return NULL;`
///
/// A genuinely *different algorithm* from cnt-rust's `mem::ccpy`, not just a
/// different element contract, so it is ported here rather than bent into the
/// kernel core: libcntpr's `memccpy.c` also stops at a NUL byte (returning
/// NULL) and compares `c` signed, whereas msvcrt copies straight through NULs
/// and compares unsigned. Copies the byte BEFORE testing it; returns the
/// pointer just PAST the matched byte.
#[no_mangle]
pub unsafe extern "C" fn _memccpy(
    to: *mut core::ffi::c_void,
    from: *const core::ffi::c_void,
    c: i32,
    count: usize,
) -> *mut core::ffi::c_void {
    let d = to as *mut u8;
    let s = from as *const u8;
    let needle = c as u8;
    let mut i = 0usize;
    while i < count {
        let t = *s.add(i);
        *d.add(i) = t;
        i += 1;
        if t == needle {
            return d.add(i) as *mut core::ffi::c_void;
        }
    }
    core::ptr::null_mut()
}

/// msvcrt `_strnset`:
/// `if (len > 0 && str) while (*str && len--) *str++ = value; return str;`
///
/// The *buffer* effect is identical to cnt-rust's (the harness proved it:
/// same bytes, `ret+0` vs `ret+11`) — msvcrt returns the **advanced** pointer,
/// one past the last byte written, not the start of the string. That is very
/// likely a Wine bug (MS documents "a pointer to the altered string"), but this
/// lane's invariant is that a flip is a *pure refactor of the C it replaces*,
/// so it is reproduced exactly rather than quietly "fixed" here.
///
/// ⚠️ **The scan MUST be bounded by `len`.** The C's `while (*str && len--)`
/// reads at most `len` bytes; an *unbounded* `strlen`-style scan here reads off
/// the end of a buffer that an `n`-suffixed API is explicitly allowed to leave
/// unterminated — and `_strnset` IS reached that way, via `mbcs.c`'s
/// `u__strnset()` → `_mbsnset`/`_mbsnbset`. The over-read produces the *same*
/// bytes (the fill stays bounded either way), so it is invisible to a
/// value-comparing fuzz — it only shows up when the scan walks into an unmapped
/// page and faults. That is exactly the bug this function shipped with for one
/// gate cycle: msvcrt-difftest reported MATCH across 20k cases because `gen_str`
/// always NUL-terminates. Detector: the guard-page test in
/// `msvcrt-difftest/tests/classify.rs` (`oob_guard`), which places the buffer
/// flush against a PROT_NONE page so an over-read segfaults.
#[no_mangle]
pub unsafe extern "C" fn _strnset(str_: *mut u8, value: i32, len: usize) -> *mut u8 {
    if str_.is_null() || len == 0 {
        return str_;
    }
    // bounded: stops at the first NUL *or* at `len`, whichever comes first.
    let take = h::cstr_slice_bounded(str_ as *const u8, len).len();
    cnt_rust::core_rs::strmut::fill(core::slice::from_raw_parts_mut(str_, take), value as u8);
    str_.add(take)
}

// ─────────────────────────────────────────────────────────────────────────
// Wave 4: the wide-string family (dll/win32/msvcrt/wcs.c)
//
// WHY THIS WAVE IS STRUCTURALLY SAFER THAN WAVE 3. Wave 3's whole hazard was
// the signed-char contract: libcntpr compares `char` as SIGNED (`Elem::diff for
// u8` is `(a as i8) - (b as i8)`), msvcrt compares UNSIGNED. They agree on
// ASCII and flip sign on bytes >= 0x80, which is why `cmp_u`/`ncmp_u` had to be
// added to the core. `wchar_t` is unsigned BY DEFINITION and `Elem::diff for
// u16` is already `(a as i32) - (b as i32)` — so that entire class cannot recur
// here. The plain generic core IS the msvcrt contract for u16, and these
// wrappers use `cmp`/`ncmp`, not the `_u` variants.
//
// TWO TRAPS THAT DO NOT TRANSFER FROM THE NARROW FAMILY:
//
//  1. `_strnset` returns the ADVANCED pointer (a Wine quirk, reproduced
//     exactly). `_wcsnset` returns the START (`wchar_t *ret = str;`). Same name
//     shape, DIFFERENT contract — do not copy the narrow wrapper.
//  2. `wcsncmp` has NO `_WIN64` branch, unlike `strncmp`: it returns the raw
//     `*str1 - *str2` difference on every arch. So there is nothing to clamp,
//     and only ONE contract to grade.
//
// The locale-dependent wide set (`_wcsicmp`, `_wcslwr`, `_wcsupr`, the whole
// `isw*` ctype block, the NLS `wcstombs` family) is DEFERRED, exactly like wave
// 3's `_stricmp`/`_strnicmp`/`_memicmp` trio: the harness pins the "C" locale,
// so it would report MATCH — and that is a blind spot, not a license to flip.
// See field manual §9.60.
// ─────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn wcslen(s: *const u16) -> usize {
    h::wcstr_slice(s).len()
}

/// u16 is already the unsigned contract, so the *element* compare is plain `cmp`
/// (never `cmp_u`). But the RESULT is clamped:
///
/// ```c
/// while (*str1 && (*str1 == *str2)) { str1++; str2++; }
/// if (*str1 < *str2) return -1;
/// if (*str1 > *str2) return 1;
/// return 0;
/// ```
///
/// Note the asymmetry with `wcsncmp` directly below, which returns the RAW
/// difference. Same family, same file, different contract — and unlike
/// `strncmp`'s, this clamp is unconditional (there is no `#if defined(_WIN64)`
/// here at all). Caught by msvcrt-difftest: raw gave -63 where msvcrt gives -1.
///
/// Single-pass for the same PERF reason as [`strcmp`]: the slice form scanned
/// BOTH strings for their length before comparing a single element, and the
/// perf gate measured it at 3.8-4.8x the C.
#[no_mangle]
pub unsafe extern "C" fn wcscmp(a: *const u16, b: *const u16) -> i32 {
    let (mut p, mut q) = (wnz(a), wnz(b));
    while *p != 0 && *p == *q {
        p = p.add(1);
        q = q.add(1);
    }
    match (*p).cmp(&*q) {
        core::cmp::Ordering::Greater => 1,
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
    }
}

/// Unlike `strncmp`, msvcrt's `wcsncmp` has **no `_WIN64` branch** — and unlike
/// `wcscmp` directly above, it does not clamp either. It returns the raw
/// `*str1 - *str2` difference on every arch.
///
/// Single-pass, like the rest of the compare family — see [`strcmp`].
#[no_mangle]
pub unsafe extern "C" fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32 {
    if n == 0 {
        return 0;
    }
    // C: `while (--n && *str1 && (*str1 == *str2)) { str1++; str2++; }`
    let (mut p, mut q, mut k) = (wnz(a), wnz(b), n);
    loop {
        k -= 1;
        if k == 0 || *p == 0 || *p != *q {
            break;
        }
        p = p.add(1);
        q = q.add(1);
    }
    (*p as i32) - (*q as i32)
}

/// The terminating NUL is a VALID target: msvcrt's `do { ... } while (*str++)`
/// means `wcschr(s, 0)` returns the terminator, NOT null. cnt's `find_char`
/// already models exactly that (`if target == ZERO { Some(s.len()) }`).
/// Single-pass, like the narrow `strchr` — see the PERF note there.
#[no_mangle]
pub unsafe extern "C" fn wcschr(s: *const u16, c: u16) -> *mut u16 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    let mut p = s;
    loop {
        if *p == c {
            return p as *mut u16; // NUL is a valid target when c == 0
        }
        if *p == 0 {
            return core::ptr::null_mut();
        }
        p = p.add(1);
    }
}

/// Same NUL-is-a-valid-target rule as `wcschr`; same single-pass contract.
#[no_mangle]
pub unsafe extern "C" fn wcsrchr(s: *const u16, c: u16) -> *mut u16 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    let mut p = s;
    let mut ret = core::ptr::null_mut();
    loop {
        if *p == c {
            ret = p as *mut u16;
        }
        if *p == 0 {
            return ret;
        }
        p = p.add(1);
    }
}

/// Is `c` one of the (non-NUL) elements of the wide set `set`?
///
/// Naive nested scan — deliberately, because that is exactly what the C does
/// (`wcschr(accept, *ptr)` per element). The narrow family can afford a 256-bit
/// `ByteSet`; the u16 space cannot be tabulated that cheaply, and msvcrt does
/// not try.
#[inline]
unsafe fn wset_has(set: *const u16, c: u16) -> bool {
    let mut p = wnz(set);
    while *p != 0 {
        if *p == c {
            return true;
        }
        p = p.add(1);
    }
    false
}

/// Single-pass — see the narrow `strcat`. C: `while ((*p++ = *src++));`
#[no_mangle]
pub unsafe extern "C" fn wcscpy(to: *mut u16, from: *const u16) -> *mut u16 {
    wcstr_copy(to, wnz(from));
    to
}

/// Single-pass. C: `wcscpy(dst + wcslen(dst), src); return dst;`
#[no_mangle]
pub unsafe extern "C" fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16 {
    if dst.is_null() {
        return dst;
    }
    let mut d = dst;
    while *d != 0 {
        d = d.add(1);
    }
    wcstr_copy(d, wnz(src));
    dst
}

/// ⚠️ An EMPTY HAYSTACK yields NULL — **even for an empty needle** — and that is
/// NOT what the narrow `strstr` does.
///
/// msvcrt's two implementations genuinely disagree on this edge:
///
/// ```c
/// char* strstr(h, n)  { needle_len = strlen(n);
///                       if (!needle_len) return (char*)h;   /* "" -> "" */
///                       ... }
/// wchar_t* wcsstr(s, sub) { while (*s) { ... }              /* never entered */
///                           return NULL; }                  /* L"" -> NULL  */
/// ```
///
/// So `strstr("", "")` is `""` but `wcsstr(L"", L"")` is `NULL`. cnt's `find_sub`
/// follows the textbook rule (empty needle matches at 0), so the empty-haystack
/// case has to be reproduced here. Caught by msvcrt-difftest.
#[no_mangle]
pub unsafe extern "C" fn wcsstr(hay: *const u16, sub: *const u16) -> *mut u16 {
    let h_s = h::wcstr_slice(hay);
    if h_s.is_empty() {
        return core::ptr::null_mut();
    }
    match cnt_rust::core_rs::string::find_sub(h_s, h::wcstr_slice(sub)) {
        Some(i) => hay.add(i) as *mut u16,
        None => core::ptr::null_mut(),
    }
}

/// C: `for (ptr = str; *ptr; ptr++) if (!wcschr(accept, *ptr)) break;`
#[no_mangle]
pub unsafe extern "C" fn wcsspn(s: *const u16, accept: *const u16) -> usize {
    let mut p = wnz(s);
    let mut n = 0usize;
    while *p != 0 && wset_has(accept, *p) {
        p = p.add(1);
        n += 1;
    }
    n
}

/// C: `for (ptr = str; *ptr; ptr++) if (wcschr(reject, *ptr)) break;`
#[no_mangle]
pub unsafe extern "C" fn wcscspn(s: *const u16, reject: *const u16) -> usize {
    let mut p = wnz(s);
    let mut n = 0usize;
    while *p != 0 && !wset_has(reject, *p) {
        p = p.add(1);
        n += 1;
    }
    n
}

/// C: `for (; *str; str++) if (wcschr(accept, *str)) return (wchar_t*)str;`
#[no_mangle]
pub unsafe extern "C" fn wcspbrk(s: *const u16, accept: *const u16) -> *mut u16 {
    let mut p = wnz(s);
    while *p != 0 {
        if wset_has(accept, *p) {
            return p as *mut u16;
        }
        p = p.add(1);
    }
    core::ptr::null_mut()
}

/// An explicit swap loop rather than `slice::reverse()` — same reason the
/// narrow `_strrev` uses one: `reverse()` pulls core's cross-crate `#[inline]`
/// generic `swap_nonoverlapping`, and this crate is `#![no_builtins]`, so it is
/// not worth betting the link on which generics get instantiated. The loop is
/// semantically identical and pulls nothing.
#[no_mangle]
pub unsafe extern "C" fn _wcsrev(s: *mut u16) -> *mut u16 {
    let n = h::wcstr_slice(s as *const u16).len();
    let buf = h::wraw_mut_slice(s, n);
    if n > 1 {
        let (mut i, mut j) = (0usize, n - 1);
        while i < j {
            let t = buf[i];
            buf[i] = buf[j];
            buf[j] = t;
            i += 1;
            j -= 1;
        }
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn _wcsset(s: *mut u16, c: u16) -> *mut u16 {
    let n = h::wcstr_slice(s as *const u16).len();
    cnt_rust::core_rs::strmut::fill(h::wraw_mut_slice(s, n), c);
    s
}

/// msvcrt: `while ((n-- > 0) && *str) *str++ = c; return ret;`
///
/// ⚠️ **The scan MUST be bounded by `n`** — the C reads at most `n` chars. This
/// is the same shape that shipped an out-of-bounds read in the narrow
/// `_strnset` (field manual §9.61): computing the length with an *unbounded*
/// `wcstr_slice(..).len()` and then `min(_, n)` yields byte-identical VALUES
/// (the fill stays bounded either way), so a value-comparing fuzz reports MATCH
/// while the *read* walks off the end of a buffer an `n`-suffixed API is
/// explicitly allowed to leave unterminated. `oob_guard` is what catches it.
///
/// Returns the **start** pointer — NOT the advanced one. (`_strnset` returns
/// the advanced pointer; its wide twin does not.)
#[no_mangle]
pub unsafe extern "C" fn _wcsnset(s: *mut u16, c: u16, n: usize) -> *mut u16 {
    if s.is_null() || n == 0 {
        return s;
    }
    let take = h::wcstr_slice_bounded(s as *const u16, n).len();
    cnt_rust::core_rs::strmut::fill(h::wraw_mut_slice(s, take), c);
    s
}

// NOTE: the no_std runtime bits (`#[panic_handler]` + `rust_eh_personality`)
// deliberately do NOT live here — they are in the `msvcrt-crt-cargo` staticlib
// umbrella. This crate is a plain rlib so that `msvcrt-difftest` can link it
// and fuzz THESE wrappers (the code that actually ships) against the genuine
// msvcrt C. A `staticlib` crate-type would force a `#[panic_handler]` into the
// same rustc invocation that emits the rlib, and that becomes a duplicate
// `panic_impl` lang item inside the harness's std test binary.

// ─────────────────────────────────────────────────────────────────────────
// Wave 5 PROBE (not a flip — deliberately NOT `#[no_mangle]`).
//
// Exposed so msvcrt-difftest can grade cnt-rust's float parser against the
// genuine msvcrt `strtod` BEFORE anything overrides the C. If it diverges,
// nothing has shipped.
//
// The suspicion going in (from cnt-rust's own doc comment): these are two
// DIFFERENT ALGORITHMS from two different lineages —
//   cnt-rust  = ReactOS libcntpr's DJ Delorie parser: a naive `r = r*10 + d`
//               accumulator, then `r *= 10^e`. Explicitly "never IEEE
//               correctly-rounded".
//   msvcrt    = Wine's `fpnum_parse` + `bnum.h`, a base-10 BIGNUM — correctly
//               rounded.
// If that is right they cannot agree on hard-to-round inputs, and the
// "assemble from cnt-rust" thesis does not reach this family.
// ─────────────────────────────────────────────────────────────────────────

/// Probe: cnt-rust's `strtod` core, marshalled to the C signature.
///
/// **HOST-ONLY (`cfg(target_os = "linux")`).** This exists solely so
/// msvcrt-difftest can grade the float parser; it is never part of msvcrt.dll.
/// It must not be compiled for the Windows targets, because i386 msvcrt builds
/// with `-C target-feature=-sse,-sse2` (the ReactOS baseline) and an `f64`
/// return needs SSE for the ABI:
///
///   error: this function definition uses SIMD vector type `f64` which (with
///   the chosen ABI) requires the `sse` target feature, which is not enabled
///
/// # Safety
/// `s` must be a valid NUL-terminated C string; `end` may be NULL.
#[cfg(target_os = "linux")]
pub unsafe fn strtod_probe(s: *const u8, end: *mut *mut u8) -> f64 {
    let sl = h::cstr_slice(s);
    let (v, consumed) = cnt_rust::core_rs::strtod::strtod_bytes(sl);
    if !end.is_null() {
        *end = (s as *mut u8).add(consumed);
    }
    v
}

// ─────────────────────────────────────────────────────────────────────────
// Wave 5a: the plumbing-free leftovers of string.c.
// ─────────────────────────────────────────────────────────────────────────

/// msvcrt `_swab`: swap adjacent byte pairs. `len` is a BYTE count and an odd
/// trailing byte is simply not copied (`len >>= 1` after the `len > 1` guard) —
/// a misfeature, reproduced.
///
/// Note `(unsigned)len >> 1`: a NEGATIVE len is guarded out by `len > 1` first,
/// so the unsigned shift never sees one.
#[no_mangle]
pub unsafe extern "C" fn _swab(src: *const u8, dst: *mut u8, len: i32) {
    if len > 1 {
        let mut n = (len as u32) >> 1;
        let (mut s, mut d) = (src, dst);
        while n > 0 {
            let s0 = *s;
            let s1 = *s.add(1);
            *d = s1;
            *d.add(1) = s0;
            d = d.add(2);
            s = s.add(2);
            n -= 1;
        }
    }
}

/// msvcrt `__strncnt`: count up to `size` non-NUL bytes.
///
/// ⚠️ TWO THINGS. (1) The body is `#if _MSVCR_VER >= 140`-gated and we build with
/// `_MSVCR_VER=0`, so the shipped branch is `while (size-- && *str++)` — the
/// bound is tested BEFORE the load, so it reads at most `size` bytes. The >=140
/// branch (`*str++ && size--`) reads one MORE. Grading the wrong arm would be
/// the §9.60 `_WIN64` blind spot all over again.
/// (2) It is `n`-suffixed and bounded, so it gets an `oob_guard` row — the §9.61
/// class (`_strnset`'s unbounded scan) is exactly this shape.
#[no_mangle]
pub unsafe extern "C" fn __strncnt(str_: *const u8, size: usize) -> usize {
    let p = nz(str_);
    let mut ret = 0usize;
    let mut i = 0usize;
    while i < size && *p.add(i) != 0 {
        ret += 1;
        i += 1;
    }
    ret
}

// ─────────────────────────────────────────────────────────────────────────
// Wave 5b: the `_s` (secure-CRT) string family.
//
// ⚠️ THE CONTRACT IS NOT JUST THE BUFFER. Every `_s` function has FOUR
// observable effects, and a flip that gets only the first is silently wrong:
//
//   1. the destination bytes
//   2. the RETURN CODE (0 / EINVAL / ERANGE / STRUNCATE)
//   3. **errno**, set via `MSVCRT_CHECK_PMT` / `MSVCRT_INVALID_PMT`
//   4. **the invalid-parameter handler firing** (apps install one; it can abort)
//
// msvcrt-difftest grades all four (see `subjectimpl.c`, which gives the SUBJECT
// its own errno storage so it can be asserted independently of the reference's).
//
// The `_s` family is Wine-derived and its edge cases are NOT the ISO ones — e.g.
// `strcat_s` truncates dst to "" on ERANGE, and `strncat_s` returns EINVAL (not
// ERANGE) when dst has no terminator within `elem`. Reproduced exactly.
// ─────────────────────────────────────────────────────────────────────────

const EINVAL: i32 = 22;
const ERANGE: i32 = 34;
const STRUNCATE: i32 = 80;
/// `((size_t)-1)` — the caller's "truncate rather than fail" sentinel.
const TRUNCATE: usize = usize::MAX;

// `_errno` and `_invalid_parameter` used to be declared `extern "C"` here, back when
// errno.c was still C. This crate DEFINES both now (msvcrt/errno.rs), and an `extern`
// declaration for a symbol your own crate defines is its own bug class: it links, so
// nothing complains, while the declaration and the definition are free to disagree about
// the signature. Import the items instead — then rustc checks them.
//
// `_errno` is msvcrt's per-thread errno slot; inside msvcrt.dll and inside the harness it
// is the same Rust body, reading `msvcrt_get_thread_data()`. `_invalid_parameter`'s default
// behaviour is "no handler installed → return", but an app CAN install one that aborts, so
// the call itself is part of the contract and must actually happen.
use crate::msvcrt::errno::_errno;

/// `_invalid_parameter` follows its feature: this crate's body when it is on (every build
/// that produces msvcrt.dll), and an import when it is off — the host harness DEFINES the
/// plain name as its call counter. Either way the call itself must happen, because it is
/// the observable the `_s` contract's `pmt` effect is measured from.
#[cfg(feature = "invalid-parameter")]
use crate::msvcrt::errno::_invalid_parameter;
#[cfg(not(feature = "invalid-parameter"))]
extern "C" {
    fn _invalid_parameter(
        expr: *const u16,
        func: *const u16,
        file: *const u16,
        line: u32,
        arg: usize,
    );
}

/// `MSVCRT_INVALID_PMT(x, err)`: set errno, then invoke the handler.
#[inline]
unsafe fn invalid_pmt(err: i32) {
    *_errno() = err;
    _invalid_parameter(
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        0,
        0,
    );
}

/// `MSVCRT_CHECK_PMT(cond)`: on failure set errno=EINVAL + fire the handler.
/// Returns the condition so call sites read like the C.
#[inline]
unsafe fn check_pmt(cond: bool) -> bool {
    if !cond {
        invalid_pmt(EINVAL);
    }
    cond
}

// ── the msvcrt integer-parse CONTRACT (wave 6a, §9.66) ────────────────────
//
// cnt-rust's `parse_i64_clamped` already IS msvcrt's `_strtoi64_l` *engine*,
// faithfully — same base auto-detect, same saturating accumulate, same
// `got_digit ? nptr : p` endptr rule. What it deliberately does NOT carry is
// msvcrt's *contract*, because the kernel CRT (`_LIBCNT_`) has neither an errno
// nor an invalid-parameter handler:
//
//   * front door — CHECK_PMT(nptr != NULL), (base == 0 || base >= 2), (base <= 36)
//       -> return 0, errno = EINVAL, FIRE the handler,
//          and leave `*endptr` ALONE (the C returns before it writes it)
//   * overflow  -> clamp AND errno = ERANGE
//
// That contract belongs HERE, at the msvcrt boundary, and the engine stays
// kernel-pure. This split is the thesis of the whole lane: cnt-rust supplies the
// algorithm, msvcrt-crt-rust supplies msvcrt's semantics. Bolting msvcrt's
// contract onto a *foreign* engine is the mistake the float probe caught — and
// it is what `strtol` was doing here, on BSD's `parse_long_bsd`, which has no
// errno concept at all.
//
// LOCALE CAVEAT (known, bounded, tracked): the C skips whitespace with
// `_isspace_l(c, NULL)` and folds case with `_tolower_l(c, NULL)` — the thread's
// CURRENT locale. We implement the "C" locale (ASCII), which is what msvcrt.dll
// starts in and what the overwhelming majority of callers ever see. classify
// fuzzes the full 0x00-0xFF byte range against the genuine C to prove C-locale
// equivalence. Under a non-C LC_CTYPE that marks extra bytes as space, this WILL
// diverge. That is Phase D (locale) and is recorded in the plan — it is not new
// debt: the already-shipped `strtol`/`strtoul` carry exactly the same caveat.

/// `_strtoi64_l`: msvcrt's front door + engine + errno.
/// Shared by `strtol`, `strtoul`, `_atoi64` (and `_strtoi64` once flipped).
#[inline]
unsafe fn strtoi64_contract(
    nptr: *const u8,
    endptr: *mut *mut u8,
    base: i32,
    locale: *mut core::ffi::c_void,
) -> i64 {
    if !check_pmt(!nptr.is_null()) {
        return 0;
    }
    if !check_pmt(base == 0 || base >= 2) {
        return 0;
    }
    if !check_pmt(base <= 36) {
        return 0;
    }
    // The C returns above WITHOUT reaching `if(endptr) *endptr = (char*)nptr;`,
    // so on a rejected front door the caller's endptr keeps whatever it held.
    // Writing it "helpfully" here is itself a divergence — classify caught the
    // old `strtol` doing exactly that.

    // ── WAVE 10: this is now a TRANSCRIPTION, not a delegation. ────────────────
    //
    // It used to be `cnt_rust::core_rs::radix::parse_i64_clamped(...)` + a bolted-on
    // ERANGE. That produced the right *value* and the wrong *side effects*, and the
    // harness said MATCH for three waves — because refimpl.c stubbed `_tolower_l` as a
    // plain ASCII fold, so the ORACLE had no side effects either. The stub was not
    // neutral scaffolding; it was an oracle, and it was our own model of msvcrt rather
    // than msvcrt. The moment ctype.c became a real reference TU, five shipped parsers
    // and `_memicmp` went DIVERGE.
    //
    // What they were missing: msvcrt folds each digit with `_tolower_l(*nptr, locale)`
    // — passing a RAW SIGNED `char`, with no `(unsigned char)` cast, unlike the
    // `_isspace_l` two lines above it. So any byte >= 0x80 sign-extends NEGATIVE, fails
    // `_tolower_l`'s `(unsigned)c < 256` test, falls into its `c >= 256` NLS arm, and
    // **sets errno = EILSEQ**. The parse then stops (the folded byte is not a digit), so
    // the VALUE is unaffected and only the errno betrays it.
    //
    // That is msvcrt's behaviour, quirk and all. The only way to be sure of reproducing
    // it is to call the same functions at the same points — which is the lane's thesis
    // restated: cnt-rust supplies the ALGORITHM, msvcrt-crt-rust supplies msvcrt's
    // SEMANTICS. A side effect is not something you can bolt onto a foreign engine.
    // ONE get_locinfo() for the whole parse — see `towlower_li`. The C resolves it per
    // character (it hands `_isspace_l`/`_tolower_l` a NULL locale), which in the real DLL
    // is three Win32 calls PER DIGIT. `hot_path` measured 513 lookups parsing a 512-digit
    // number. Idempotent, so hoisting changes nothing observable.
    let li = locinfo_of(locale);
    let p = nptr;
    let mut s = nptr;
    let mut base = base;
    let mut negative = false;
    let mut got_digit = false;
    let mut ret: i64 = 0;

    // `(unsigned char)*nptr` — the cast IS here in the C.
    while isctype_li(*s as i32, sys::_SPACE as i32, li) != 0 {
        s = s.add(1);
    }

    if *s == b'-' {
        negative = true;
        s = s.add(1);
    } else if *s == b'+' {
        s = s.add(1);
    }

    // `_tolower_l(*(nptr+1), locale)` — NO cast. `*s as i8 as i32` reproduces the
    // sign-extension exactly; `*s as i32` would not, and would lose the EILSEQ.
    if (base == 0 || base == 16)
        && *s == b'0'
        && tolower_li(*s.add(1) as i8 as i32, li) == b'x' as i32
    {
        base = 16;
        s = s.add(2);
    }

    if base == 0 {
        base = if *s == b'0' { 8 } else { 10 };
    }

    while *s != 0 {
        // `char cur = _tolower_l(*nptr, locale);` — the int result is truncated back
        // into a SIGNED char, so a high-bit fold stays negative and fails both digit
        // tests below. Reproduce the truncation (`as i8`), not just the call.
        let cur = tolower_li(*s as i8 as i32, li) as i8 as i32;
        let mut v: i32;

        if cur >= b'0' as i32 && cur <= b'9' as i32 {
            if cur >= b'0' as i32 + base {
                break;
            }
            v = cur - b'0' as i32;
        } else {
            if cur < b'a' as i32 || cur >= b'a' as i32 + base - 10 {
                break;
            }
            v = cur - b'a' as i32 + 10;
        }
        got_digit = true;

        if negative {
            v = -v;
        }
        s = s.add(1);

        let b = base as i64;
        let vv = v as i64;
        if !negative && (ret > i64::MAX / b || ret.wrapping_mul(b) > i64::MAX - vv) {
            ret = i64::MAX;
            *_errno() = ERANGE;
        } else if negative && (ret < i64::MIN / b || ret.wrapping_mul(b) < i64::MIN - vv) {
            ret = i64::MIN;
            *_errno() = ERANGE;
        } else {
            ret = ret.wrapping_mul(b).wrapping_add(vv);
        }
    }

    if !endptr.is_null() {
        // `got_digit ? nptr : p` — on a no-digit parse the endptr rewinds to the
        // ORIGINAL pointer, not to where the whitespace scan stopped.
        *endptr = (if got_digit { s } else { p }) as *mut u8;
    }
    ret
}

/// `_strtoui64_l` — msvcrt's UNSIGNED engine. Not `strtoi64_contract` with a cast:
/// it accumulates in `unsigned __int64`, saturates at `UI64_MAX`, and negates the
/// unsigned accumulator at the end (`return negative ? -ret : ret;`, which is where
/// `strtoul("-1")` gets its `ret = 1` underflow quirk from).
///
/// And note the asymmetry with the signed twin, which is real and deliberate-looking
/// enough that it must be preserved:
///
///     _strtoi64_l:   v = cur   - '0';    // the FOLDED char
///     _strtoui64_l:  v = *nptr - '0';    // the RAW char
///
/// Identical for ASCII digits; not identical for anything else.
#[inline]
unsafe fn strtoui64_contract(
    nptr: *const u8,
    endptr: *mut *mut u8,
    base: i32,
    locale: *mut core::ffi::c_void,
) -> u64 {
    if !check_pmt(!nptr.is_null()) {
        return 0;
    }
    if !check_pmt(base == 0 || base >= 2) {
        return 0;
    }
    if !check_pmt(base <= 36) {
        return 0;
    }

    let li = locinfo_of(locale); // ONCE — see strtoi64_contract
    let p = nptr;
    let mut s = nptr;
    let mut base = base;
    let mut negative = false;
    let mut got_digit = false;
    let mut ret: u64 = 0;

    while isctype_li(*s as i32, sys::_SPACE as i32, li) != 0 {
        s = s.add(1);
    }

    if *s == b'-' {
        negative = true;
        s = s.add(1);
    } else if *s == b'+' {
        s = s.add(1);
    }

    if (base == 0 || base == 16)
        && *s == b'0'
        && tolower_li(*s.add(1) as i8 as i32, li) == b'x' as i32
    {
        base = 16;
        s = s.add(2);
    }

    if base == 0 {
        base = if *s == b'0' { 8 } else { 10 };
    }

    while *s != 0 {
        let raw = *s;
        let cur = tolower_li(raw as i8 as i32, li) as i8 as i32;
        let v: u64;

        if cur >= b'0' as i32 && cur <= b'9' as i32 {
            if cur >= b'0' as i32 + base {
                break;
            }
            v = (raw as i32 - b'0' as i32) as u64; // RAW, per the asymmetry above
        } else {
            if cur < b'a' as i32 || cur >= b'a' as i32 + base - 10 {
                break;
            }
            v = (cur - b'a' as i32 + 10) as u64;
        }
        got_digit = true;
        s = s.add(1);

        let b = base as u64;
        if ret > u64::MAX / b || ret.wrapping_mul(b) > u64::MAX - v {
            ret = u64::MAX;
            *_errno() = ERANGE;
        } else {
            ret = ret.wrapping_mul(b).wrapping_add(v);
        }
    }

    if !endptr.is_null() {
        *endptr = (if got_digit { s } else { p }) as *mut u8;
    }
    if negative {
        ret.wrapping_neg()
    } else {
        ret
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcpy_s(dst: *mut u8, elem: usize, src: *const u8) -> i32 {
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem != 0) {
        return EINVAL;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }
    let mut i = 0usize;
    while i < elem {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            return 0;
        }
        i += 1;
    }
    invalid_pmt(ERANGE);
    *dst = 0; // dst[0], NOT dst[elem] — see the C's comment
    ERANGE
}

#[no_mangle]
pub unsafe extern "C" fn strcat_s(dst: *mut u8, elem: usize, src: *const u8) -> i32 {
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem != 0) {
        return EINVAL;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }
    let mut i = 0usize;
    while i < elem {
        if *dst.add(i) == 0 {
            let mut j = 0usize;
            while j + i < elem {
                let c = *src.add(j);
                *dst.add(j + i) = c;
                if c == 0 {
                    return 0;
                }
                j += 1;
            }
            break; // fell out of the inner loop -> too small
        }
        i += 1;
    }
    invalid_pmt(ERANGE);
    *dst = 0;
    ERANGE
}

#[no_mangle]
pub unsafe extern "C" fn strncpy_s(
    dst: *mut u8,
    elem: usize,
    src: *const u8,
    count: usize,
) -> i32 {
    let truncate = count == TRUNCATE;
    if count == 0 {
        if !dst.is_null() && elem != 0 {
            *dst = 0;
        }
        return 0;
    }
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem != 0) {
        return EINVAL;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }
    let (mut p, mut s) = (dst, src);
    let (mut e, mut c) = (elem, count);
    while e != 0 && c != 0 && *s != 0 {
        *p = *s;
        p = p.add(1);
        s = s.add(1);
        e -= 1;
        c -= 1;
    }
    if e == 0 && truncate {
        *p.sub(1) = 0;
        return STRUNCATE;
    }
    if e == 0 {
        *dst = 0;
        return ERANGE;
    }
    *p = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn strncat_s(
    dst: *mut u8,
    elem: usize,
    src: *const u8,
    count: usize,
) -> i32 {
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem != 0) {
        return EINVAL;
    }
    if count == 0 {
        return 0;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }

    let mut i = 0usize;
    while i < elem && *dst.add(i) != 0 {
        i += 1;
    }
    if i == elem {
        // dst has no terminator inside elem: EINVAL, NOT ERANGE (a Wine quirk).
        invalid_pmt(EINVAL);
        *dst = 0;
        return EINVAL;
    }

    let mut j = 0usize;
    while j + i < elem {
        if count == TRUNCATE && j + i == elem - 1 {
            *dst.add(j + i) = 0;
            return STRUNCATE;
        }
        if j == count {
            *dst.add(j + i) = 0;
            return 0;
        }
        let c = *src.add(j);
        *dst.add(j + i) = c;
        if c == 0 {
            *dst.add(j + i) = 0;
            return 0;
        }
        j += 1;
    }

    invalid_pmt(ERANGE);
    *dst = 0;
    ERANGE
}

/// ⚠️ Note the FINAL failure here sets **EINVAL**, not ERANGE — and unlike its
/// siblings it calls `_invalid_parameter` BEFORE `*_errno() = EINVAL` (the C
/// spells the macro out by hand rather than using MSVCRT_INVALID_PMT). The
/// observable effects are the same set; reproduced in the C's order anyway.
#[no_mangle]
pub unsafe extern "C" fn _strnset_s(str_: *mut u8, size: usize, c: i32, count: usize) -> i32 {
    if str_.is_null() && size == 0 && count == 0 {
        return 0;
    }
    if !check_pmt(!str_.is_null()) {
        return EINVAL;
    }
    if !check_pmt(size > 0) {
        return EINVAL;
    }
    let mut i = 0usize;
    while i < size - 1 && i < count {
        if *str_.add(i) == 0 {
            return 0;
        }
        *str_.add(i) = c as u8;
        i += 1;
    }
    while i < size {
        if *str_.add(i) == 0 {
            return 0;
        }
        i += 1;
    }
    *str_ = 0;
    _invalid_parameter(
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        0,
        0,
    );
    *_errno() = EINVAL;
    EINVAL
}

// ═════════════════════════════════════════════════════════════════════════════
// WAVE 6b — the radix `_s` family (10 fns), and why it needed FOUR writers
// ═════════════════════════════════════════════════════════════════════════════
//
// These ten look like one function with ten signatures. They are not. msvcrt
// implements them with FOUR mutually-inconsistent error contracts, and the only
// way to know that is to read all ten bodies — or to have a differential oracle
// tell you. Both were done.
//
//   group A  _ltoa_s _ltow_s _itoa_s _itow_s      (32-bit signed, via ltoa_helper)
//   group B  _i64toa_s _i64tow_s                  (64-bit signed)
//     front door: MSVCRT_CHECK_PMT -> errno=EINVAL + handler FIRES
//     too small : ERANGE, and the buffer gets the digits written BACKWARDS
//
//   group C  _ui64toa_s _ui64tow_s                (64-bit unsigned)
//     front door: MSVCRT_CHECK_PMT -> errno=EINVAL + handler FIRES
//     too small : *** EINVAL, not ERANGE *** — and the buffer is left UNTOUCHED
//
//   group D  _ultoa_s _ultow_s                    (32-bit unsigned)
//     front door: HAND-ROLLED `*_errno() = EINVAL` — the handler does NOT FIRE
//     too small : ERANGE + reversed residue, handler still does NOT fire
//
// Group D is the reason wave 6a had to start grading effect 4. `_ultoa_s` and
// `_ltoa_s` are byte-identical, return-code-identical AND errno-identical on a
// bad radix. The ONLY thing that distinguishes them is whether the invalid-
// parameter handler fired — which an app can install and which can abort().
// A three-effect oracle calls them equal.
//
// THE REVERSED RESIDUE. On a too-small buffer, groups A/B/D do not simply fail:
//
//     for (pos = buffer + 31, i = 0; i < size; i++)   /* buffer+31 == LAST digit */
//         *p++ = *pos--;                              /* ...so this walks BACKWARDS */
//     str[0] = '\0';
//
// The caller's buffer is filled with the digits in REVERSE, and only then NUL'd
// at [0]. A clean reimplementation (`str[0] = 0; return ERANGE;`) passes every
// string-level test and silently differs in every byte past the first. Nobody
// would write this on purpose; it is bug-for-bug with Microsoft. Reproduced.

/// Groups A+B — signed, `MSVCRT_CHECK_PMT` front door, ERANGE + reversed residue.
///
/// `mag`/`negative` are decided by the CALLER because the width matters: the C
/// assigns the signed value into an unsigned of ITS OWN width, so for a negative
/// value with radix != 10, `_ltoa_s` reinterprets as u32 and `_i64toa_s` as u64.
/// Collapsing that here would silently sign-extend.
#[inline]
unsafe fn radix_s_signed(mag: u64, negative: bool, str: *mut u8, size: usize, radix: i32) -> i32 {
    if !check_pmt(!str.is_null()) {
        return EINVAL;
    }
    if !check_pmt(size > 0) {
        return EINVAL;
    }
    if !check_pmt((2..=36).contains(&radix)) {
        *str = 0;
        return EINVAL;
    }
    let f = cnt_rust::core_rs::radix::format_radix(mag, radix as u32, negative);
    let len = f.ndigits + f.negative as usize + 1; // digits + sign + NUL
    if len > size {
        // The C skips the sign slot, shrinks `size` by it, then copies the digit
        // buffer BACKWARDS from its last digit. `size <= ndigits + neg` holds here
        // (that is what `len > size` means), so the reads stay inside the digits.
        let o = f.negative as usize;
        let n = size - o;
        let mut i = 0;
        while i < n && i < f.ndigits {
            *str.add(o + i) = f.digits[f.ndigits - 1 - i];
            i += 1;
        }
        *str = 0;
        invalid_pmt(ERANGE); // MSVCRT_INVALID_PMT: errno = ERANGE, handler fires
        return ERANGE;
    }
    let mut o = 0usize;
    if f.negative {
        *str = b'-';
        o = 1;
    }
    let mut i = 0;
    while i < f.ndigits {
        *str.add(o + i) = f.digits[i];
        i += 1;
    }
    *str.add(o + f.ndigits) = 0;
    0
}

/// Groups A+B, wide. Identical contract; the digits are ASCII so they widen 1:1.
#[inline]
unsafe fn radix_s_signed_w(mag: u64, negative: bool, str: *mut u16, size: usize, radix: i32) -> i32 {
    if !check_pmt(!str.is_null()) {
        return EINVAL;
    }
    if !check_pmt(size > 0) {
        return EINVAL;
    }
    if !check_pmt((2..=36).contains(&radix)) {
        *str = 0;
        return EINVAL;
    }
    let f = cnt_rust::core_rs::radix::format_radix(mag, radix as u32, negative);
    let len = f.ndigits + f.negative as usize + 1;
    if len > size {
        let o = f.negative as usize;
        let n = size - o;
        let mut i = 0;
        while i < n && i < f.ndigits {
            *str.add(o + i) = f.digits[f.ndigits - 1 - i] as u16;
            i += 1;
        }
        *str = 0;
        invalid_pmt(ERANGE);
        return ERANGE;
    }
    let mut o = 0usize;
    if f.negative {
        *str = b'-' as u16;
        o = 1;
    }
    let mut i = 0;
    while i < f.ndigits {
        *str.add(o + i) = f.digits[i] as u16;
        i += 1;
    }
    *str.add(o + f.ndigits) = 0;
    0
}

/// Group C — `_ui64toa_s`/`_ui64tow_s`. The odd one out: a too-small buffer is
/// **EINVAL, not ERANGE**, and the buffer is left completely alone (no residue,
/// not even a NUL). Handler still fires.
#[inline]
unsafe fn radix_s_u64(value: u64, str: *mut u8, size: usize, radix: i32) -> i32 {
    if !check_pmt(!str.is_null()) {
        return EINVAL;
    }
    if !check_pmt(size > 0) {
        return EINVAL;
    }
    if !check_pmt((2..=36).contains(&radix)) {
        *str = 0;
        return EINVAL;
    }
    let f = cnt_rust::core_rs::radix::format_radix(value, radix as u32, false);
    if f.ndigits + 1 > size {
        invalid_pmt(EINVAL); // MSVCRT_INVALID_PMT(..., EINVAL) — NOT ERANGE
        return EINVAL;
    }
    let mut i = 0;
    while i < f.ndigits {
        *str.add(i) = f.digits[i];
        i += 1;
    }
    *str.add(f.ndigits) = 0;
    0
}

/// Group C, wide.
#[inline]
unsafe fn radix_s_u64_w(value: u64, str: *mut u16, size: usize, radix: i32) -> i32 {
    if !check_pmt(!str.is_null()) {
        return EINVAL;
    }
    if !check_pmt(size > 0) {
        return EINVAL;
    }
    if !check_pmt((2..=36).contains(&radix)) {
        *str = 0;
        return EINVAL;
    }
    let f = cnt_rust::core_rs::radix::format_radix(value, radix as u32, false);
    if f.ndigits + 1 > size {
        invalid_pmt(EINVAL);
        return EINVAL;
    }
    let mut i = 0;
    while i < f.ndigits {
        *str.add(i) = f.digits[i] as u16;
        i += 1;
    }
    *str.add(f.ndigits) = 0;
    0
}

/// Group D — `_ultoa_s`/`_ultow_s`. Same *observable buffer and return code* as
/// group A, but its front door is hand-rolled (`*_errno() = EINVAL`) rather than
/// MSVCRT_CHECK_PMT, so **the invalid-parameter handler never fires**. That one
/// bit is the entire difference, and it is why the oracle has to grade it.
#[inline]
unsafe fn radix_s_ulong(value: u32, str: *mut u8, size: usize, radix: i32) -> i32 {
    if str.is_null() || size == 0 || !(2..=36).contains(&radix) {
        if !str.is_null() && size != 0 {
            *str = 0;
        }
        *_errno() = EINVAL; // errno only — deliberately NOT invalid_pmt()
        return EINVAL;
    }
    let f = cnt_rust::core_rs::radix::format_radix(value as u64, radix as u32, false);
    let len = f.ndigits + 1;
    if len > size {
        let mut i = 0;
        while i < size && i < f.ndigits {
            *str.add(i) = f.digits[f.ndigits - 1 - i];
            i += 1;
        }
        *str = 0;
        *_errno() = ERANGE; // again: errno only, no handler
        return ERANGE;
    }
    let mut i = 0;
    while i < f.ndigits {
        *str.add(i) = f.digits[i];
        i += 1;
    }
    *str.add(f.ndigits) = 0;
    0
}

/// Group D, wide.
#[inline]
unsafe fn radix_s_ulong_w(value: u32, str: *mut u16, size: usize, radix: i32) -> i32 {
    if str.is_null() || size == 0 || !(2..=36).contains(&radix) {
        if !str.is_null() && size != 0 {
            *str = 0;
        }
        *_errno() = EINVAL;
        return EINVAL;
    }
    let f = cnt_rust::core_rs::radix::format_radix(value as u64, radix as u32, false);
    let len = f.ndigits + 1;
    if len > size {
        let mut i = 0;
        while i < size && i < f.ndigits {
            *str.add(i) = f.digits[f.ndigits - 1 - i] as u16;
            i += 1;
        }
        *str = 0;
        *_errno() = ERANGE;
        return ERANGE;
    }
    let mut i = 0;
    while i < f.ndigits {
        *str.add(i) = f.digits[i] as u16;
        i += 1;
    }
    *str.add(f.ndigits) = 0;
    0
}

// ── the ten exports ──────────────────────────────────────────────────────────
// `negative` is `value < 0 && radix == 10` — for any other radix the C
// reinterprets the value's bit pattern as unsigned OF ITS OWN WIDTH, which is
// why _ltoa_s goes through u32 and _i64toa_s through u64.

#[inline]
fn mag32(value: i32, radix: i32) -> (u64, bool) {
    if value < 0 && radix == 10 {
        (value.unsigned_abs() as u64, true)
    } else {
        (value as u32 as u64, false)
    }
}
#[inline]
fn mag64(value: i64, radix: i32) -> (u64, bool) {
    if value < 0 && radix == 10 {
        (value.unsigned_abs(), true)
    } else {
        (value as u64, false)
    }
}

#[no_mangle]
pub unsafe extern "C" fn _ltoa_s(value: i32, str: *mut u8, size: usize, radix: i32) -> i32 {
    let (m, n) = mag32(value, radix);
    radix_s_signed(m, n, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _itoa_s(value: i32, str: *mut u8, size: usize, radix: i32) -> i32 {
    _ltoa_s(value, str, size, radix) // the C literally does this
}
#[no_mangle]
pub unsafe extern "C" fn _ltow_s(value: i32, str: *mut u16, size: usize, radix: i32) -> i32 {
    let (m, n) = mag32(value, radix);
    radix_s_signed_w(m, n, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _itow_s(value: i32, str: *mut u16, size: usize, radix: i32) -> i32 {
    _ltow_s(value, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _i64toa_s(value: i64, str: *mut u8, size: usize, radix: i32) -> i32 {
    let (m, n) = mag64(value, radix);
    radix_s_signed(m, n, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _i64tow_s(value: i64, str: *mut u16, size: usize, radix: i32) -> i32 {
    let (m, n) = mag64(value, radix);
    radix_s_signed_w(m, n, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _ui64toa_s(value: u64, str: *mut u8, size: usize, radix: i32) -> i32 {
    radix_s_u64(value, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _ui64tow_s(value: u64, str: *mut u16, size: usize, radix: i32) -> i32 {
    radix_s_u64_w(value, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _ultoa_s(value: u32, str: *mut u8, size: usize, radix: i32) -> i32 {
    radix_s_ulong(value, str, size, radix)
}
#[no_mangle]
pub unsafe extern "C" fn _ultow_s(value: u32, str: *mut u16, size: usize, radix: i32) -> i32 {
    radix_s_ulong_w(value, str, size, radix)
}

// ── the msvcrt integer-parse engine, now exported (wave 6b) ──────────────────
// These ARE `strtoi64_contract` — `_strtoi64` is literally
// `_strtoi64_l(nptr, endptr, base, NULL)`. Flipping them was blocked until 6a
// gave the contract layer an errno and a handler to talk to.

#[no_mangle]
pub unsafe extern "C" fn _strtoi64(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> i64 {
    strtoi64_contract(nptr, endptr, base, core::ptr::null_mut())
}

// ═════════════════════════════════════════════════════════════════════════════
// WAVE 6c — the wide `_s` string family, the tokenizers, memmove, _strtoui64
// ═════════════════════════════════════════════════════════════════════════════
//
// The wide `_s` functions are NOT the narrow ones with a wider char. Their
// handler behaviour differs from each other and from their narrow twins:
//
//   wcscpy_s    handler fires on EVERY error        (CHECK_PMT / CHECK_PMT_ERR)
//   wcsncpy_s   handler fires on the FRONT DOOR ONLY — its ERANGE and STRUNCATE
//               arms are plain `return`s, silent
//   wcscat_s    handler NEVER fires — every check is hand-rolled `if (!dst ...)`
//   wcsncat_s   handler fires on everything, including a dst with no terminator
//               inside `elem` (its own EINVAL, distinct from the ERANGE overflow)
//
// Assuming wide == narrow is how this lane already got bitten twice (wcscmp
// CLAMPS where wcsncmp returns a raw diff; wcstol consults the locale where
// strtol does not). Every one of these was read, not inferred.

/// `wcscpy_s` — CHECK_PMT front door, then a whole-length check: ERANGE (with
/// dst truncated to "") if the source does not fit, else a straight memmove.
#[no_mangle]
pub unsafe extern "C" fn wcscpy_s(dst: *mut u16, elem: usize, src: *const u16) -> i32 {
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem != 0) {
        return EINVAL;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }
    let size = h::wcstr_slice(src).len() + 1; // wcslen(src) + 1
    if size > elem {
        *dst = 0;
        invalid_pmt(ERANGE); // MSVCRT_CHECK_PMT_ERR(.., ERANGE)
        return ERANGE;
    }
    let mut i = 0;
    while i < size {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    0
}

/// `wcsncpy_s` — note `count == 0` returns 0 *before* the front door (and still
/// NULs dst), and note the `_TRUNCATE` arm writes `*(p-1) = 0`, i.e. it
/// OVERWRITES the last character it just copied. Its ERANGE/STRUNCATE arms do
/// NOT fire the handler.
#[no_mangle]
pub unsafe extern "C" fn wcsncpy_s(dst: *mut u16, elem: usize, src: *const u16, count: usize) -> i32 {
    let truncate = count == TRUNCATE;
    if count == 0 {
        if !dst.is_null() && elem != 0 {
            *dst = 0;
        }
        return 0; // no handler, no errno
    }
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem != 0) {
        return EINVAL;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }
    let (mut e, mut c) = (elem, count);
    let (mut p, mut s) = (dst, src);
    while e != 0 && c != 0 && *s != 0 {
        *p = *s;
        p = p.add(1);
        s = s.add(1);
        e -= 1;
        c -= 1;
    }
    if e == 0 && truncate {
        *p.sub(1) = 0; // clobbers the last copied char — deliberate, per the C
        return STRUNCATE; // silent: no handler
    } else if e == 0 {
        *dst = 0;
        return ERANGE; // silent: no handler
    }
    *p = 0;
    0
}

/// `wcscat_s` — the one that NEVER fires the invalid-parameter handler. Every
/// check is hand-rolled, so errno is not set either: it just returns the code.
/// Also note it scans for dst's terminator only WITHIN `elem`, and on overflow
/// truncates dst to "" and returns ERANGE.
#[no_mangle]
pub unsafe extern "C" fn wcscat_s(dst: *mut u16, elem: usize, src: *const u16) -> i32 {
    if dst.is_null() || elem == 0 {
        return EINVAL; // no handler, and NOT even errno — bare return
    }
    if src.is_null() {
        *dst = 0;
        return EINVAL;
    }
    let mut i = 0usize;
    while i < elem && *dst.add(i) != 0 {
        i += 1;
    }
    let mut s = src;
    while i < elem {
        let c = *s;
        s = s.add(1);
        *dst.add(i) = c;
        i += 1;
        if c == 0 {
            return 0;
        }
    }
    *dst = 0;
    ERANGE
}

/// `wcsncat_s` — the fully-instrumented one: handler fires on the front door,
/// on a dst with no terminator inside `elem` (EINVAL), and on overflow (ERANGE).
#[no_mangle]
pub unsafe extern "C" fn wcsncat_s(dst: *mut u16, elem: usize, src: *const u16, count: usize) -> i32 {
    if !check_pmt(!dst.is_null()) {
        return EINVAL;
    }
    if !check_pmt(elem > 0) {
        return EINVAL;
    }
    if count == 0 {
        return 0;
    }
    if !check_pmt(!src.is_null()) {
        *dst = 0;
        return EINVAL;
    }
    let mut i = 0usize;
    while i < elem {
        if *dst.add(i) == 0 {
            break;
        }
        i += 1;
    }
    if i == elem {
        // dst[elem] is not NUL-terminated — its OWN EINVAL, with the handler.
        invalid_pmt(EINVAL);
        *dst = 0;
        return EINVAL;
    }
    let mut j = 0usize;
    while j + i < elem {
        if count == TRUNCATE && j + i == elem - 1 {
            *dst.add(j + i) = 0;
            return STRUNCATE;
        }
        let c = if j == count { 0 } else { *src.add(j) };
        if j == count {
            *dst.add(j + i) = 0;
            return 0;
        }
        *dst.add(j + i) = c;
        if c == 0 {
            *dst.add(j + i) = 0;
            return 0;
        }
        j += 1;
    }
    invalid_pmt(ERANGE);
    *dst = 0;
    ERANGE
}

/// `_wcsnset_s` — the all-NULL early-out (`!str && !size && !count` -> 0) comes
/// BEFORE the front door, and the failure arm fires the handler and sets errno
/// by hand rather than through MSVCRT_INVALID_PMT.
#[no_mangle]
pub unsafe extern "C" fn _wcsnset_s(str_: *mut u16, size: usize, c: u16, count: usize) -> i32 {
    if str_.is_null() && size == 0 && count == 0 {
        return 0;
    }
    if !check_pmt(!str_.is_null()) {
        return EINVAL;
    }
    if !check_pmt(size > 0) {
        return EINVAL;
    }
    let mut i = 0usize;
    while i < size - 1 && i < count {
        if *str_.add(i) == 0 {
            return 0;
        }
        *str_.add(i) = c;
        i += 1;
    }
    while i < size {
        if *str_.add(i) == 0 {
            return 0;
        }
        i += 1;
    }
    *str_ = 0;
    invalid_pmt(EINVAL);
    EINVAL
}

/// `_wcsset_s` — fills until the terminator or until `n` runs out; running out
/// means dst was never terminated inside n -> "" + EINVAL + handler.
#[no_mangle]
pub unsafe extern "C" fn _wcsset_s(str_: *mut u16, n: usize, c: u16) -> i32 {
    if !check_pmt(!str_.is_null()) {
        return EINVAL;
    }
    if !check_pmt(n != 0) {
        return EINVAL;
    }
    let mut p = str_;
    let mut left = n;
    // C: `while(*p && --n) *p++ = c;`  — the decrement happens in the condition,
    // so it runs at most n-1 times and `left == 0` means "no terminator in n".
    while *p != 0 {
        left -= 1;
        if left == 0 {
            break;
        }
        *p = c;
        p = p.add(1);
    }
    if left == 0 {
        *str_ = 0;
        invalid_pmt(EINVAL);
        return EINVAL;
    }
    0
}

/// `wcsncpy` — copies at most n, then ZERO-PADS the remainder. Does NOT
/// guarantee a terminator when src is longer than n.
#[no_mangle]
pub unsafe extern "C" fn wcsncpy(s1: *mut u16, s2: *const u16, n: usize) -> *mut u16 {
    let mut i = 0usize;
    while i < n {
        let c = *s2.add(i);
        *s1.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    while i < n {
        *s1.add(i) = 0;
        i += 1;
    }
    s1
}

/// `wcsnlen` — bounded length; never reads past `maxlen`.
#[no_mangle]
pub unsafe extern "C" fn wcsnlen(s: *const u16, maxlen: usize) -> usize {
    let mut i = 0usize;
    while i < maxlen {
        if *s.add(i) == 0 {
            break;
        }
        i += 1;
    }
    i
}

// `memmove` is NOT flipped, and deliberately so.
//
// It has no ORACLE. refstub/msvcrt.h:185 routes `memmove` into the private
// `msvcrt_ref_*` namespace — that namespace is what guarantees the reference TU
// can never bind to the code under test — so the compiled reference does not
// define an msvcrt `memmove` to compare against. Exposing one would mean
// un-namespacing it, weakening the harness's central guarantee to test a single
// function.
//
// The lane's rule is that nothing flips without a differential proof, and that
// rule does not get suspended because a flip looks easy. (The body would be
// three lines — `h::raw_move` is already overlap-correct and already ships under
// `memcpy`.) Owed: give the refstub a way to expose a namespaced symbol as a
// ref_* oracle without opening the namespace to the subject.

/// `strtok_s` — the re-entrant tokenizer. Flippable (unlike `strtok`) because the
/// caller owns the context: `strtok` reaches into msvcrt's `thread_data_t`, and
/// getting at that field from Rust would mean hardcoding an offset into a struct
/// we do not own — the magic-offset class (`scan_magic_offsets`). Deferred until
/// there is a real TLS story.
#[no_mangle]
pub unsafe extern "C" fn strtok_s(
    mut str_: *mut u8,
    delim: *const u8,
    ctx: *mut *mut u8,
) -> *mut u8 {
    if !check_pmt(!delim.is_null()) {
        return core::ptr::null_mut();
    }
    if !check_pmt(!ctx.is_null()) {
        return core::ptr::null_mut();
    }
    if !check_pmt(!str_.is_null() || !(*ctx).is_null()) {
        return core::ptr::null_mut();
    }
    if str_.is_null() {
        str_ = *ctx;
    }
    // `*str &&` guards each strchr: strchr(delim, 0) would match delim's own NUL.
    while *str_ != 0 && h::byte_in(delim, *str_) {
        str_ = str_.add(1);
    }
    if *str_ == 0 {
        *ctx = str_;
        return core::ptr::null_mut();
    }
    *ctx = str_.add(1);
    while **ctx != 0 && !h::byte_in(delim, **ctx) {
        *ctx = (*ctx).add(1);
    }
    if **ctx != 0 {
        **ctx = 0;
        *ctx = (*ctx).add(1);
    }
    str_
}

/// `wcstok_s` — the wide twin. Same contract; structured slightly differently in
/// the C but observably identical (it assigns `*next_token` unconditionally).
#[no_mangle]
pub unsafe extern "C" fn wcstok_s(
    mut str_: *mut u16,
    delim: *const u16,
    next_token: *mut *mut u16,
) -> *mut u16 {
    if !check_pmt(!delim.is_null()) {
        return core::ptr::null_mut();
    }
    if !check_pmt(!next_token.is_null()) {
        return core::ptr::null_mut();
    }
    if !check_pmt(!str_.is_null() || !(*next_token).is_null()) {
        return core::ptr::null_mut();
    }
    if str_.is_null() {
        str_ = *next_token;
    }
    while *str_ != 0 && h::wchar_in(delim, *str_) {
        str_ = str_.add(1);
    }
    let ret = if *str_ == 0 {
        core::ptr::null_mut()
    } else {
        let r = str_;
        str_ = str_.add(1);
        while *str_ != 0 && !h::wchar_in(delim, *str_) {
            str_ = str_.add(1);
        }
        if *str_ != 0 {
            *str_ = 0;
            str_ = str_.add(1);
        }
        r
    };
    *next_token = str_;
    ret
}

/// `_strtoui64` -> `_strtoui64_l(nptr, endptr, base, NULL)`.
///
/// A SEPARATE function from `_strtoi64_l`, not a cast of it: it accumulates
/// unsigned, saturates at _UI64_MAX, and then applies `negative ? -ret : ret` —
/// so `_strtoui64("-1")` is 0xFFFF_FFFF_FFFF_FFFF, not an error. (cnt-rust's
/// generic `parse_u64_clamped_w` is close but returns the ADVANCED endptr when no
/// digit was parsed, where msvcrt rewinds to the original — so this is translated
/// here rather than borrowed.)
#[no_mangle]
pub unsafe extern "C" fn _strtoui64(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> u64 {
    // WAVE 10: was a hand-rolled ASCII parser (`to_ascii_lowercase`, a literal
    // `matches!(*q, b' ' | 0x09..=0x0d)` whitespace set). Both are models of msvcrt
    // rather than msvcrt: the real engine skips whitespace with `_isspace_l` and folds
    // digits with `_tolower_l`, reading the LOCALE's tables — and, on a high-bit byte,
    // setting errno = EILSEQ. Delegate to the transcription.
    strtoui64_contract(nptr, endptr, base, core::ptr::null_mut())
}

// ═════════════════════════════════════════════════════════════════════════════
// WAVE 7 — the case-insensitive family
// ═════════════════════════════════════════════════════════════════════════════
//
// LOCALE (checked first, per wave 6's lesson — do not assume). Each of these
// consults `locinfo->lc_handle[LC_CTYPE]` and takes an INLINE ASCII fast path
// when it is unset. That is the "C" locale, which is what msvcrt.dll starts in.
// classify fuzzes the full 0x00-0xFF byte range against the genuine C to prove
// equivalence there. This is the SAME bounded caveat the already-shipped
// `_strlwr`/`_strupr` carry (they are `_strlwr_s_l(str, -1, NULL)`), so wave 7
// introduces no new debt. A non-C LC_CTYPE is Phase D.
//
// THREE contracts, again all different:
//
//   _stricmp/_strnicmp   CHECK_PMT(s1 && s2) -> _NLSCMPERROR + handler.
//                        `count == 0` returns 0 BEFORE the front door.
//                        Folds on the SIGNED char, returns the (unsigned char) diff.
//   _memicmp             hand-rolled NULL check, but STILL fires the handler --
//                        and only when len != 0 (`len ? _NLSCMPERROR : 0`).
//                        Folds via _tolower_l, which passes non-ASCII THROUGH,
//                        so its result is the SIGNED difference. Not the same as
//                        _strnicmp's. Reproduced separately.
//   _strlwr_s/_strupr_s  hand-rolled `*_errno() = EINVAL` -- handler NEVER fires
//                        (the `_ultoa_s` shape). Unterminated-within-len is its
//                        own EINVAL, and it truncates dst to "".

/// `_NLSCMPERROR` (crt/string.h) — what the icmp family returns on a bad pointer.
const NLSCMPERROR: i32 = 2147483647;

/// The C locale's `_tolower_l`: fold A-Z, pass EVERYTHING else through unchanged
/// (including negative `char`s — msvcrt hands this raw signed chars).
#[inline]
fn c_lower(c: i32) -> i32 {
    if c >= b'A' as i32 && c <= b'Z' as i32 {
        c + 32
    } else {
        c
    }
}

/// `_strnicmp_l`.
#[inline]
unsafe fn strnicmp_core(
    s1: *const u8,
    s2: *const u8,
    mut count: usize,
    locale: *mut core::ffi::c_void,
) -> i32 {
    if count == 0 {
        return 0; // before the front door, per the C
    }
    if !check_pmt(!s1.is_null() && !s2.is_null()) {
        return NLSCMPERROR;
    }

    let li = locinfo_of(locale);
    let (mut p, mut q) = (s1 as *const i8, s2 as *const i8);
    let (mut c1, mut c2);

    // ── path 1: the "C" locale (lc_handle[LC_CTYPE] == 0) ────────────────
    //
    // An INLINE ASCII fold on the SIGNED char, and note the return: the operands are
    // re-read as UNSIGNED. `c1 -= 'A' - 'a'` is a SUBTRACTION of a NEGATIVE constant
    // (-32), i.e. an ADD — transcribing it literally as `c1 -= 32` is how wave 7 first
    // got 'I'(73) -> ')'(41), and classify convicted it with cnt=-131.
    if (*li).lc_handle[sys::LC_CTYPE] == 0 {
        loop {
            c1 = *p as i32;
            p = p.add(1);
            if c1 >= b'A' as i32 && c1 <= b'Z' as i32 {
                c1 += 32;
            }
            c2 = *q as i32;
            q = q.add(1);
            if c2 >= b'A' as i32 && c2 <= b'Z' as i32 {
                c2 += 32;
            }
            count -= 1;
            if count == 0 || c1 == 0 || c1 != c2 {
                break;
            }
        }
        return (c1 as u8) as i32 - (c2 as u8) as i32;
    }

    // ── path 2: a REAL locale ────────────────────────────────────────────
    //
    // Folds through `_tolower_l` — WITH the `(unsigned char)` cast, so unlike the int
    // parsers this one never reaches the EILSEQ arm. And it returns the RAW difference,
    // not the unsigned-re-read one. Two paths, two return conventions.
    loop {
        c1 = tolower_li((*p as u8) as i32, li); // li resolved ONCE, above
        p = p.add(1);
        c2 = tolower_li((*q as u8) as i32, li);
        q = q.add(1);
        count -= 1;
        if count == 0 || c1 == 0 || c1 != c2 {
            break;
        }
    }
    c1 - c2
}

#[no_mangle]
pub unsafe extern "C" fn _strnicmp(s1: *const u8, s2: *const u8, count: usize) -> i32 {
    strnicmp_core(s1, s2, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _stricmp(s1: *const u8, s2: *const u8) -> i32 {
    // the C passes INT_MAX
    strnicmp_core(s1, s2, i32::MAX as usize, core::ptr::null_mut())
}

/// `_memicmp_l`.
///
/// Note how this differs from `_strnicmp`: it folds through `_tolower_l`, which
/// in the C locale passes non-ASCII (including NEGATIVE chars) straight through,
/// and it returns that difference RAW — no `(unsigned char)` cast. So on high
/// bytes the two functions return different numbers. They are not the same
/// comparison with a different bound.
#[inline]
unsafe fn memicmp_core(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    mut len: usize,
    locale: *mut core::ffi::c_void,
) -> i32 {
    // `_MSVCR_VER == 0` -> this guard IS compiled in. It is hand-rolled rather
    // than CHECK_PMT, but it still fires the handler — and only when len != 0.
    if s1.is_null() || s2.is_null() {
        if len != 0 {
            invalid_pmt(EINVAL);
            return NLSCMPERROR;
        }
        return 0;
    }
    // Resolve the locale ONCE, not twice per byte. The C does it per byte (it passes a
    // NULL `locale` straight into `_tolower_l`), and in the real DLL that is THREE Win32
    // calls — GetLastError + TlsGetValue + SetLastError — per fold. `get_locinfo()` is
    // idempotent, so hoisting is behaviourally identical and strictly better; msvcrt's own
    // hot comparer (`_wcsicmp_l`) hoists too. Caught by `hot_path` (1024 lookups for a
    // 512-byte compare).
    let li = locinfo_of(locale);
    let (mut p, mut q) = (s1 as *const i8, s2 as *const i8);
    let mut ret = 0i32;
    while len != 0 {
        len -= 1;
        // WAVE 10. This was `c_lower(*p as i32)` — an ASCII fold — and classify said
        // MATCH, because refimpl.c's `_tolower_l` stub was an ASCII fold too. The C is:
        //
        //     const char *s1 = v1;                                  // SIGNED char
        //     ret = _tolower_l(*s1, locale) - _tolower_l(*s2, locale);
        //
        // No `(unsigned char)` cast. A byte >= 0x80 therefore arrives at `_tolower_l`
        // as a NEGATIVE int, misses its `(unsigned)c < 256` table path, and takes the
        // NLS arm — which sets errno = EILSEQ and returns the LCMapString result. So
        // `_memicmp` on high-bit bytes differs from an ASCII fold in BOTH its return
        // value and its errno. Call the real thing; do not re-model it.
        ret = tolower_li(*p as i32, li) - tolower_li(*q as i32, li);
        if ret != 0 {
            break;
        }
        p = p.add(1);
        q = q.add(1);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _memicmp(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    len: usize,
) -> i32 {
    memicmp_core(s1, s2, len, core::ptr::null_mut())
}

/// `_strlwr_s_l` / `_strupr_s_l`.
///
/// The handler NEVER fires (hand-rolled `*_errno() = EINVAL`, the `_ultoa_s`
/// shape). "No terminator inside len" is its own EINVAL and truncates dst to "".
#[inline]
unsafe fn strcase_s_core(
    str_: *mut u8,
    len: usize,
    upper: bool,
    locale: *mut core::ffi::c_void,
) -> i32 {
    if str_.is_null() || len == 0 {
        *_errno() = EINVAL; // errno only — no handler
        return EINVAL;
    }
    let mut n = len;
    let mut ptr = str_;
    while n != 0 && *ptr != 0 {
        n -= 1;
        ptr = ptr.add(1);
    }
    if n == 0 {
        *str_ = 0;
        *_errno() = EINVAL; // errno only — no handler
        return EINVAL;
    }
    // WAVE 11: `_strlwr_s_l` / `_strupr_s_l` carry their OWN lc_handle[LC_CTYPE] branch —
    // an inline ASCII fold in the "C" locale, `_tolower_l((unsigned char)*str, locale)`
    // otherwise. We shipped only the first half (wave 7), which is correct in the "C"
    // locale and wrong the moment setlocale is called. Both halves now.
    let li = locinfo_of(locale);
    let mut p = str_;

    if (*li).lc_handle[sys::LC_CTYPE] == 0 {
        while *p != 0 {
            let c = *p;
            if upper {
                if c.is_ascii_lowercase() {
                    *p = c - 32;
                }
            } else if c.is_ascii_uppercase() {
                *p = c + 32;
            }
            p = p.add(1);
        }
        return 0;
    }

    // The cast IS in the C here (`_tolower_l((unsigned char)*str, locale)`), so this path
    // — unlike the int parsers — never reaches the EILSEQ arm.
    while *p != 0 {
        let c = (*p) as i32;
        *p = if upper {
            toupper_li(c, li) as u8   // li resolved ONCE, above
        } else {
            tolower_li(c, li) as u8
        };
        p = p.add(1);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _strlwr_s(str_: *mut u8, len: usize) -> i32 {
    strcase_s_core(str_, len, false, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _strupr_s(str_: *mut u8, len: usize) -> i32 {
    strcase_s_core(str_, len, true, core::ptr::null_mut())
}

// ═════════════════════════════════════════════════════════════════════════════
// WAVE 8 — the WIDE case-insensitive family
// ═════════════════════════════════════════════════════════════════════════════
//
// The wide mirror of wave 7, and it is NOT a mirror. Three differences, each of
// which a "same thing but u16" transcription would have gotten wrong:
//
//  1. NO C-locale fast path in the comparer. `_strnicmp_l` folds INLINE when
//     `lc_handle[LC_CTYPE]` is unset; `_wcsicmp_l` always calls `_towlower_l`.
//     That is fine only because `_towlower_l` has the fast path INSIDE it
//     (`if(c >= 'A' && c <= 'Z') return c + 'a' - 'A';` — checked, not assumed).
//
//  2. The comparers return the RAW u16 difference — `return c1 - c2;` with no
//     clamp. `wcscmp` (wave 4) CLAMPS. Same file, same shape, different contract.
//
//  3. The folders MUTATE BEFORE THEY FAIL. `_strlwr_s_l` scans for the terminator
//     FIRST and errors without touching the buffer; `_wcslwr_s_l` folds as it
//     scans, so on a string with no terminator inside `n` the caller's buffer
//     comes back with its first `n` chars ALREADY FOLDED and then `str[0] = 0`.
//     Its front door also zeroes `*str` where the narrow one does not.
//     classify compares the full buffer, so this is graded, not hoped for.
//
// The C's own comment on the failure arm is worth keeping:
//     /* MSDN claims that the function should return and set errno to
//      * ERANGE, which doesn't seem to be true based on the tests. */
// It returns EINVAL. Bug-for-bug.

/// The C locale's `_towlower_l` (wcs.c) — ASCII fold, everything else through.
#[inline]
fn cw_lower(c: u16) -> u16 {
    if c >= b'A' as u16 && c <= b'Z' as u16 {
        c + 32 // c + 'a' - 'A'
    } else {
        c
    }
}
#[inline]
fn cw_upper(c: u16) -> u16 {
    if c >= b'a' as u16 && c <= b'z' as u16 {
        c - 32
    } else {
        c
    }
}

/// `_wcsnicmp_l(s1, s2, n, NULL)` — C locale. Returns the RAW diff (no clamp).
#[inline]
unsafe fn wcsnicmp_core(s1: *const u16, s2: *const u16, mut n: usize) -> i32 {
    if n == 0 {
        return 0; // before the front door
    }
    // C: `if(!CHECK_PMT(str1) || !CHECK_PMT(str2))` — the || SHORT-CIRCUITS, so a
    // NULL str1 fires the handler exactly ONCE and never evaluates the str2 check.
    if !check_pmt(!s1.is_null()) || !check_pmt(!s2.is_null()) {
        return NLSCMPERROR;
    }
    let li = sys::get_locinfo(); // ONCE, not per char — see towlower_li
    let (mut p, mut q) = (s1, s2);
    let (mut c1, mut c2);
    // WAVE 11. Was `cw_lower` — an inline ASCII fold, so this was locale-INDEPENDENT.
    // `_wcsnicmp_l` has NO lc_handle branch of its own: it folds through `_towlower_l`
    // unconditionally and inherits that function's locale-correctness. Do the same, and
    // the whole wide compare becomes locale-correct for free.
    loop {
        c1 = towlower_li(*p, li);
        p = p.add(1);
        c2 = towlower_li(*q, li);
        q = q.add(1);
        n -= 1; // C: `while(--n && c1 && (c1 == c2))`
        if n == 0 || c1 == 0 || c1 != c2 {
            break;
        }
    }
    c1 as i32 - c2 as i32 // RAW — wcscmp clamps, this does not
}

#[no_mangle]
pub unsafe extern "C" fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32 {
    wcsnicmp_core(s1, s2, n)
}

/// `_wcsicmp_l(s1, s2, NULL)`. Unlike `_wcsnicmp` there is no count, so no
/// `!n` early-out: the front door is the first thing that runs.
#[no_mangle]
pub unsafe extern "C" fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32 {
    if !check_pmt(!s1.is_null()) || !check_pmt(!s2.is_null()) {
        return NLSCMPERROR;
    }
    // The C resolves the locale ONCE (`get_current_locale_noalloc(&tmp)`) and passes it
    // down. Doing it per character costs a TLS lookup per char — a 3x boot slowdown.
    let li = sys::get_locinfo();
    let (mut p, mut q) = (s1, s2);
    let (mut c1, mut c2);
    loop {
        // WAVE 11: was `cw_lower` — an inline ASCII fold, i.e. locale-INDEPENDENT.
        // `_wcsicmp_l` folds through `_towlower_l`, which carries the lc_handle branch.
        // Caught by the [NLS] row the moment the harness's locale tables stopped being
        // ASCII: cnt=94 vs msvcrt=126.
        c1 = towlower_li(*p, li);
        p = p.add(1);
        c2 = towlower_li(*q, li);
        q = q.add(1);
        if c1 == 0 || c1 != c2 {
            break;
        }
    }
    c1 as i32 - c2 as i32
}

/// `_wcslwr_s_l` / `_wcsupr_s_l` with the C locale.
///
/// Folds AS it scans — so a string with no terminator inside `n` comes back with
/// its first `n` chars folded and `str[0] = 0`. The narrow twin does not do this.
/// Handler never fires (hand-rolled errno).
#[inline]
unsafe fn wcscase_s_core(str_: *mut u16, n: usize, upper: bool) -> i32 {
    if str_.is_null() || n == 0 {
        if !str_.is_null() {
            *str_ = 0; // the narrow `_strlwr_s_l` does NOT do this
        }
        *_errno() = EINVAL; // errno only — no handler
        return EINVAL;
    }
    let li = sys::get_locinfo(); // ONCE, not per char — see towlower_li
    let mut left = n;
    let mut ptr = str_;
    while left != 0 {
        left -= 1;
        if *ptr == 0 {
            return 0;
        }
        // WAVE 11: was cw_upper/cw_lower (ASCII). `_wcslwr_s_l` folds through
        // `_towlower_l`, which carries the lc_handle branch.
        *ptr = if upper {
            towupper_li(*ptr, li)
        } else {
            towlower_li(*ptr, li)
        };
        ptr = ptr.add(1);
    }
    // No terminator within n. MSDN says ERANGE; msvcrt returns EINVAL (Wine
    // verified this against the real DLL — see the comment in wcs.c).
    *str_ = 0;
    *_errno() = EINVAL;
    EINVAL
}

#[no_mangle]
pub unsafe extern "C" fn _wcslwr_s(str_: *mut u16, n: usize) -> i32 {
    wcscase_s_core(str_, n, false)
}
#[no_mangle]
pub unsafe extern "C" fn _wcsupr_s(str_: *mut u16, n: usize) -> i32 {
    wcscase_s_core(str_, n, true)
}

/// `_wcslwr(str)` == `_wcslwr_s_l(str, -1, NULL)`; `_wcsupr` likewise via
/// `_wcsupr_l`. `n = (size_t)-1` means the terminator is always found, so the
/// EINVAL arm is unreachable — but the errno/return path is inherited, and a NULL
/// str still takes the front door.
#[no_mangle]
pub unsafe extern "C" fn _wcslwr(str_: *mut u16) -> *mut u16 {
    wcscase_s_core(str_, usize::MAX, false);
    str_
}
#[no_mangle]
pub unsafe extern "C" fn _wcsupr(str_: *mut u16) -> *mut u16 {
    wcscase_s_core(str_, usize::MAX, true);
    str_
}

// ═════════════════════════════════════════════════════════════════════════════
// WAVE 9 — the wide ctype family (`isw*`), 26 functions
// ═════════════════════════════════════════════════════════════════════════════
//
// These look locale-bound and are NOT. `_iswctype_l` takes a `_locale_t` and
// then **never uses it**:
//
//     INT CDECL _iswctype_l( wchar_t wc, wctype_t type, _locale_t locale )
//     {
//         if (wc == WEOF)  return 0;
//         if (wc < 256)    return MSVCRT__pwctype[wc] & type;
//         if (!GetStringTypeW(CT_CTYPE1, &wc, 1, &ct)) return 0;
//         return ct & type;                          /* `locale` unused */
//     }
//
// So there is no `locinfo->...` field to reach into — no magic-offset problem —
// and the whole family (including every `_l` variant) is flippable.
//
// Better still: reading `MSVCRT__pwctype` and calling `GetStringTypeW` means the
// flip is locale-CORRECT, not merely C-locale-correct. In the DLL that global is
// msvcrt's own table (repopulated by setlocale) and GetStringTypeW is kernel32's
// real Unicode classifier. This carries NONE of the caveat the parse and
// case-folding families carry — it inherits msvcrt's locale state instead of
// modelling it.
//
// `towupper`/`towlower` are NOT here: `_towlower_l` reads
// `locinfo->lc_handle[LC_CTYPE]` and calls LCMapStringW, and that field IS a
// struct offset we don't own. They stay C until Phase D.

/// msvcrt's wide ctype table — a POINTER to 256 WORDs of mask bits.
///
/// In msvcrt.dll this is now `crate::msvcrt::ctype`'s own definition (`MSVCRT__wctype + 1`,
/// generated and proven against ctype.c.obj). In the host harness the `ctype-tables`
/// feature is off and subjectimpl.c owns the symbol, so it stays an `extern` there.
/// (Data, so no calling convention — `extern "C"` is right for the `extern` arm.)
#[cfg(not(feature = "ctype-tables"))]
extern "C" {
    static MSVCRT__pwctype: *const u16;
}
#[cfg(feature = "ctype-tables")]
use crate::msvcrt::ctype::MSVCRT__pwctype;

// GetStringTypeW is `WINAPI` — i.e. **stdcall on i386** (`_GetStringTypeW@16`) and
// cdecl on amd64/arm64. `extern "system"` is the one that resolves per-arch;
// `extern "C"` hardcodes cdecl and silently links on amd64/arm64 while failing i386
// with `_GetStringTypeW: i386 cdecl — missing symbol / wrong ABI`. This is the
// exact trap AGENTS.md calls out, and it cost an i386 gate.
//
// Worth noting HOW it was caught: `cargo check --target i686-pc-windows-gnu` passed
// clean, because `check` does not LINK. Three green cross-checks proved nothing
// about this. The gate's ABI verifier is what named it.
extern "system" {
    /// kernel32's Unicode classifier — the `wc >= 256` path.
    fn GetStringTypeW(kind: u32, src: *const u16, count: i32, chartype: *mut u16) -> i32;
}

// ctype.h mask bits.
const CT_UPPER: u16 = 0x0001;
const CT_LOWER: u16 = 0x0002;
const CT_DIGIT: u16 = 0x0004;
const CT_SPACE: u16 = 0x0008;
const CT_PUNCT: u16 = 0x0010;
const CT_CONTROL: u16 = 0x0020;
const CT_BLANK: u16 = 0x0040;
const CT_HEX: u16 = 0x0080;
const CT_ALPHA: u16 = 0x0100 | CT_UPPER | CT_LOWER;
/// `CT_CTYPE1` — GetStringTypeW's "character type 1" request.
const CT_CTYPE1: u32 = 1;
/// `WEOF` — the wide end-of-file sentinel, classified as nothing.
const WEOF: u32 = 0xFFFF;

/// `_iswctype_l(wc, type, locale)` — `locale` is accepted and ignored, exactly as
/// in the C. Everything else in this section is a mask applied through here.
#[no_mangle]
pub unsafe extern "C" fn _iswctype_l(wc: u32, ty: u16, _locale: *mut core::ffi::c_void) -> i32 {
    if wc == WEOF {
        return 0;
    }
    if wc < 256 {
        return ((*MSVCRT__pwctype.add(wc as usize)) & ty) as i32;
    }
    let c = wc as u16;
    let mut ct: u16 = 0;
    if GetStringTypeW(CT_CTYPE1, &c, 1, &mut ct) == 0 {
        return 0; // the C ERR()s and returns 0
    }
    (ct & ty) as i32
}

#[no_mangle]
pub unsafe extern "C" fn iswctype(wc: u32, ty: u16) -> i32 {
    _iswctype_l(wc, ty, core::ptr::null_mut())
}

/// Each `isw*` is one mask through `_iswctype_l`. The masks are transcribed from
/// the C, not reasoned about — `iswgraph` and `iswprint` differ only by _BLANK,
/// and `iswblank` is not a mask at all (see below).
macro_rules! isw {
    ($l:ident, $plain:ident, $mask:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $l(wc: u32, locale: *mut core::ffi::c_void) -> i32 {
            _iswctype_l(wc, $mask, locale)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(wc: u32) -> i32 {
            _iswctype_l(wc, $mask, core::ptr::null_mut())
        }
    };
}

isw!(_iswalnum_l,  iswalnum,  CT_ALPHA | CT_DIGIT);
isw!(_iswalpha_l,  iswalpha,  CT_ALPHA);
isw!(_iswcntrl_l,  iswcntrl,  CT_CONTROL);
isw!(_iswdigit_l,  iswdigit,  CT_DIGIT);
isw!(_iswgraph_l,  iswgraph,  CT_ALPHA | CT_DIGIT | CT_PUNCT);
isw!(_iswlower_l,  iswlower,  CT_LOWER);
isw!(_iswprint_l,  iswprint,  CT_ALPHA | CT_BLANK | CT_DIGIT | CT_PUNCT);
isw!(_iswpunct_l,  iswpunct,  CT_PUNCT);
isw!(_iswspace_l,  iswspace,  CT_SPACE);
isw!(_iswupper_l,  iswupper,  CT_UPPER);
isw!(_iswxdigit_l, iswxdigit, CT_HEX);

/// `_iswblank_l` is the one that is NOT just a mask:
///
///     return wc == '\t' || _iswctype_l( wc, _BLANK, locale );
///
/// TWO things hide in that line. The explicit TAB is load-bearing — a bare _BLANK
/// mask would still match TAB in the C-locale table, but not necessarily under a
/// locale whose table drops it. And C's `||` **NORMALISES to 0/1**, so this is the
/// only member of the family that returns a BOOLEAN. Every other `isw*` returns
/// the raw masked bits (`iswspace(' ')` is 8, not 1).
///
/// Returning the raw bit here gave `iswblank(' ') == 64` against msvcrt's `1`.
/// The exhaustive sweep caught it on the first character it could.
#[no_mangle]
pub unsafe extern "C" fn _iswblank_l(wc: u32, locale: *mut core::ffi::c_void) -> i32 {
    if wc == b'\t' as u32 {
        return 1;
    }
    (_iswctype_l(wc, CT_BLANK, locale) != 0) as i32 // the `||` normalises
}
#[no_mangle]
pub unsafe extern "C" fn iswblank(wc: u32) -> i32 {
    _iswblank_l(wc, core::ptr::null_mut())
}

/// `iswalpha_l` — msvcrt exports this WITHOUT the leading underscore too (an
/// alias of `_iswalpha_l`); it is a distinct export ordinal, so it needs its own
/// symbol.
#[no_mangle]
pub unsafe extern "C" fn iswalpha_l(wc: u32, locale: *mut core::ffi::c_void) -> i32 {
    _iswctype_l(wc, CT_ALPHA, locale)
}

// ═══════════════════════════════════════════════════════════════════════════
// WAVE 10 — the locale-table family.  The door Phase D was waiting on.
//
// The lane's plan recorded the whole locale family as BLOCKED, on the grounds that the
// harness pins the "C" locale (§9.60) and would therefore report MATCH for a `toupper`
// that is only C-locale-correct.  That is true — of a locale-INDEPENDENT implementation.
// It is not true of a faithful one, and the distinction is the whole of wave 10:
//
//     int CDECL _toupper_l(int c, _locale_t locale) {
//         ...
//         if((unsigned)c < 256) {
//             if(locinfo->pctype[c] & _LEADBYTE) return c;
//             return locinfo->pcumap[c];          // <-- the LOCALE'S OWN table
//         }
//
// A Rust body that reads `locinfo->pcumap[c]` — the same table — inherits the C's
// locale-correctness BY CONSTRUCTION: the locale-dependence lives in the DATA, setlocale
// repopulates it, and both bodies follow it.  The blind spot forbids hardcoding ASCII.
// It never forbade reading the real table.  We just had no way to reach the table
// without hand-rolling struct offsets — which is the bug class that cost this repo the
// KTHREAD.Teb hunt and the TEB.GdiBatchCount-at-0x74 read that killed explorer (§9.41).
//
// `msvcrt-sys` is that way: bindgen + `offset_of!`, with the layout asserted on two
// sides by two compilers on every build (rustc via layout_<arch>.rs; clang, with
// msvcrt's REAL flags, via dll/win32/msvcrt/rust_layout_assert.c).
//
// ── THE TRAP IN THIS FAMILY, and it bit us ─────────────────────────────────
//
// `pctype` is `MSVCRT__ctype + 1`.  The +1 is load-bearing: `pctype[-1]` is a real,
// addressable slot (the EOF entry), which is exactly why `_isctype_l`'s domain test is
// `c >= -1`.  Index it with an UNSIGNED c and you read at 0xFFFFFFFF.  Every index here
// is a SIGNED `offset()`, never `add()`.
//
// And the domain test `if((unsigned)c < 256)` is FALSE for a negative c.  msvcrt's own
// callers pass raw *signed* `char`s (string.c's `_tolower_l(*nptr, locale)` — no cast,
// while the `_isspace_l` two lines above it DOES cast), so a byte >= 0x80 sign-extends
// negative, misses the table path entirely, and lands in the `c >= 256` NLS arm —
// which sets errno = EILSEQ.  That side effect is REAL msvcrt behaviour, it is why
// six already-shipped functions were divergent the moment the oracle got honest, and
// it is reproduced faithfully below.  Do not "fix" it into a clean unsigned cast: we
// are matching msvcrt, not improving it.
// ═══════════════════════════════════════════════════════════════════════════

use msvcrt_sys as sys;

const EILSEQ: i32 = 42; // msvcrt's value. The HOST's is 84 — see refstub/msvcrt.h.

const LCMAP_LOWERCASE: u32 = 0x0000_0100;
const LCMAP_UPPERCASE: u32 = 0x0000_0200;
// CT_CTYPE1 + MSVCRT__pwctype already declared by wave 9 (the wide ctype family).

extern "C" {
    /// msvcrt's `initial_locale` (locale.c) — TRUE until the first `setlocale`.
    /// `toupper`/`tolower` branch on it to take a hardcoded-ASCII fast path.
    static initial_locale: i32;


}

extern "system" {
    /// kernel32 — and therefore **WINAPI, i.e. stdcall on i386** (`_GetStringTypeExA@20`),
    /// cdecl on amd64/arm64. `extern "system"` resolves to the right one per arch;
    /// `extern "C"` would hardcode cdecl, link fine on amd64+arm64, and fail ONLY on
    /// i386 — which is precisely the bug wave 9 shipped on `GetStringTypeW` and only
    /// caught at the gate, because `cargo check` does not LINK and so all three
    /// "green" cross-target checks proved nothing.
    fn GetStringTypeExA(
        lcid: u32,
        info_type: u32,
        src: *const u8,
        cchsrc: i32,
        chartype: *mut u16,
    ) -> i32;
}

/// `locale ? locale->locinfo : get_locinfo()` — the two-line preamble every `_l`
/// function in ctype.c opens with.
#[inline]
unsafe fn locinfo_of(locale: *mut core::ffi::c_void) -> *mut sys::threadlocaleinfostruct {
    if locale.is_null() {
        sys::get_locinfo()
    } else {
        (*(locale as *mut sys::localeinfo_struct)).locinfo
    }
}

/// `locinfo->pctype[c]`, with the SIGNED index the `+1` bias requires.
#[inline]
unsafe fn pctype_at(li: *mut sys::threadlocaleinfostruct, c: i32) -> u32 {
    *(*li).pctype.offset(c as isize) as u32
}

/// `_isctype_l`, on an ALREADY-RESOLVED locinfo. See `towlower_li` for why the callers
/// below must hoist this out of their per-character loops.
#[inline]
unsafe fn isctype_li(c: i32, ty: i32, li: *mut sys::threadlocaleinfostruct) -> i32 {
    // `c >= -1 && c <= 255` — and pctype[-1] is a real slot, hence offset(), not add().
    if (-1..=255).contains(&c) {
        return (pctype_at(li, c) as i32) & ty;
    }

    // Multibyte locales only. mb_cur_max == 1 in the "C" locale, so this is dead there.
    if (*li).mb_cur_max != 1 && c > 0 {
        let mut convert = [0u8; 3];
        let mut n = 0usize;
        if pctype_at(li, ((c as u32) >> 8) as i32) & (sys::_LEADBYTE) != 0 {
            convert[n] = ((c as u32) >> 8) as u8;
            n += 1;
        }
        convert[n] = (c & 0xff) as u8;
        n += 1;
        convert[n] = 0;

        let mut type_info: u16 = 0;
        let cch = if convert[1] != 0 { 2 } else { 1 };
        if GetStringTypeExA(
            (*li).lc_handle[sys::LC_CTYPE],
            CT_CTYPE1,
            convert.as_ptr(),
            cch,
            &mut type_info,
        ) != 0
        {
            return (type_info as i32) & ty;
        }
    }
    0
}

/// `_isctype_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _isctype_l(c: i32, ty: i32, locale: *mut core::ffi::c_void) -> i32 {
    isctype_li(c, ty, locinfo_of(locale))
}

/// `_isctype (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _isctype(c: i32, ty: i32) -> i32 {
    _isctype_l(c, ty, core::ptr::null_mut())
}

/// The `is*` family: every one is a one-line `_isctype_l` with a fixed mask, and the
/// plain form is the `_l` form with a NULL locale. Exactly as ctype.c has it.
macro_rules! isc {
    ($l:ident, $plain:ident, $mask:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $l(c: i32, locale: *mut core::ffi::c_void) -> i32 {
            _isctype_l(c, $mask, locale)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(c: i32) -> i32 {
            _isctype_l(c, $mask, core::ptr::null_mut())
        }
    };
}

isc!(_isalnum_l, isalnum, (sys::_ALPHA | sys::_DIGIT) as i32);
isc!(_isalpha_l, isalpha, sys::_ALPHA as i32);
isc!(_iscntrl_l, iscntrl, sys::_CONTROL as i32);
isc!(_isdigit_l, isdigit, sys::_DIGIT as i32);
isc!(_isgraph_l, isgraph, (sys::_ALPHA | sys::_DIGIT | sys::_PUNCT) as i32);
isc!(_islower_l, islower, sys::_LOWER as i32);
isc!(
    _isprint_l,
    isprint,
    (sys::_ALPHA | sys::_BLANK | sys::_DIGIT | sys::_PUNCT) as i32
);
isc!(_isspace_l, isspace, sys::_SPACE as i32);
isc!(_isupper_l, isupper, sys::_UPPER as i32);
isc!(_isxdigit_l, isxdigit, sys::_HEX as i32);
isc!(_isleadbyte_l, isleadbyte, sys::_LEADBYTE as i32);

/// `ispunct (MSVCRT.@)` and `_ispunct_l (MSVCR80.@)`.
///
/// The `isc!` macro above is not used here because the pair is asymmetric in the DLL:
/// only `ispunct` is in `msvcrt.spec`, so only it is an EXPORT of msvcrt.dll. But
/// `_ispunct_l` is still compiled — `llvm-nm --defined-only ctype.c.obj` lists
/// `T __ispunct_l` — and an earlier note here read "msvcrt exports NO `_ispunct_l`, only
/// the plain form. (Checked against ctype.c's banner list; do not 'complete the set'.)"
/// The banner list is the DLL export list; it is not the object's symbol table, and it is
/// the object that decides whether the C body stays live. This one symbol was the whole
/// reason ctype.c.obj was still being pulled into the link.
#[no_mangle]
pub unsafe extern "C" fn ispunct(c: i32) -> i32 {
    _isctype_l(c, sys::_PUNCT as i32, core::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn _ispunct_l(c: i32, locale: *mut core::ffi::c_void) -> i32 {
    _isctype_l(c, sys::_PUNCT as i32, locale)
}

/// `_isblank_l` / `isblank` — the odd ones out, and they are odd in DIFFERENT ways.
///
/// Both special-case `\t` ahead of the table (it is `_CONTROL|_SPACE` in the ctype
/// table, but not `_BLANK`). What they then RETURN is not the same:
///
///     int CDECL _isblank_l(int c, _locale_t locale) {
///         if (c == '\t') return _BLANK;                    // the MASK -- 64
///         ...
///     int CDECL isblank(int c) {
///         return c == '\t' || _isctype( c, _BLANK );       // `||` NORMALISES -- 1
///     }
///
/// C's `||` yields 0/1, so the plain form is the only member of the whole narrow ctype
/// family that returns a BOOLEAN rather than the raw masked bits. Its `_l` sibling,
/// three lines up in the same file, returns the mask.
///
/// This is the identical trap wave 9 hit on the WIDE twin (`iswblank`), and it was
/// walked into again here — caught both times only because the sweep is exhaustive:
/// the divergence exists at exactly one input, `c == '\t'`.
#[no_mangle]
pub unsafe extern "C" fn _isblank_l(c: i32, locale: *mut core::ffi::c_void) -> i32 {
    if c == b'\t' as i32 {
        return sys::_BLANK as i32; // the MASK
    }
    _isctype_l(c, sys::_BLANK as i32, locale)
}
#[no_mangle]
pub unsafe extern "C" fn isblank(c: i32) -> i32 {
    // `||` — a normalised 0/1, NOT the mask. Do not "make this consistent".
    (c == b'\t' as i32 || _isctype_l(c, sys::_BLANK as i32, core::ptr::null_mut()) != 0) as i32
}

/// `__isascii` / `__toascii` / `__iscsym` / `__iscsymf` — pure bit tests, no locale.
#[no_mangle]
pub unsafe extern "C" fn __isascii(c: i32) -> i32 {
    ((c as u32) < 0x80) as i32
}
#[no_mangle]
pub unsafe extern "C" fn __toascii(c: i32) -> i32 {
    ((c as u32) & 0x7f) as i32
}
#[no_mangle]
pub unsafe extern "C" fn iswascii(c: i32) -> i32 {
    ((c as u32) < 0x80) as i32
}
#[no_mangle]
pub unsafe extern "C" fn __iscsym(c: i32) -> i32 {
    (c == b'_' as i32 || isalnum(c) != 0) as i32
}
#[no_mangle]
pub unsafe extern "C" fn __iscsymf(c: i32) -> i32 {
    (c == b'_' as i32 || isalpha(c) != 0) as i32
}
#[no_mangle]
pub unsafe extern "C" fn __iswcsym(c: u32) -> i32 {
    (c == b'_' as u32 || iswalnum(c) != 0) as i32
}
#[no_mangle]
pub unsafe extern "C" fn __iswcsymf(c: u32) -> i32 {
    (c == b'_' as u32 || iswalpha(c) != 0) as i32
}

/// `__pctype_func` / `__p__pctype` — the ctype table accessors.
#[no_mangle]
pub unsafe extern "C" fn __pctype_func() -> *const u16 {
    (*sys::get_locinfo()).pctype
}
#[no_mangle]
pub unsafe extern "C" fn __p__pctype() -> *mut *mut u16 {
    core::ptr::addr_of_mut!(MSVCRT__pctype)
}
#[no_mangle]
pub unsafe extern "C" fn __pwctype_func() -> *const u16 {
    MSVCRT__pwctype
}
#[no_mangle]
pub unsafe extern "C" fn __p__pwctype() -> *mut *mut u16 {
    // Wave 9 declares MSVCRT__pwctype as `static ... : *const u16` (it only ever READ
    // the table). msvcrt's `__p__pwctype` hands out the address of the pointer itself,
    // which C spells `unsigned short**`. Cast at the boundary; the C owns the storage.
    core::ptr::addr_of!(MSVCRT__pwctype) as *mut *mut u16
}

extern "C" {
    static mut MSVCRT__pctype: *mut u16;
}

/// The `c >= 256` arm shared by `_toupper_l` and `_tolower_l`, verbatim.
///
/// This is where the EILSEQ comes from. It is NOT dead code: msvcrt's own callers hand
/// these functions raw signed `char`s, so any byte >= 0x80 arrives here as a negative
/// int (`(unsigned)c < 256` is false) and takes the `else` branch below.
#[inline]
unsafe fn case_map_wide(c: i32, li: *mut sys::threadlocaleinfostruct, mapflag: u32) -> i32 {
    let mut s = [0u8; 2];
    let mut n = 0usize;

    if pctype_at(li, ((c >> 8) & 255) as i32) & sys::_LEADBYTE != 0 {
        s[n] = ((c >> 8) & 255) as u8;
        n += 1;
    } else {
        *_errno() = EILSEQ;
        s[1] = 0;
    }
    s[n] = (c & 255) as u8;
    n += 1;

    let mut ret = [0u8; 2];
    match __crtLCMapStringA(
        (*li).lc_handle[sys::LC_CTYPE],
        mapflag,
        s.as_ptr() as *const i8,
        n as i32,
        ret.as_mut_ptr() as *mut i8,
        2,
        (*li).lc_codepage,
        0,
    ) {
        0 => c,
        1 => ret[0] as i32,
        _ => ret[0] as i32 + ((ret[1] as i32) << 8),
    }
}

/// The narrow case-mappers, on an ALREADY-RESOLVED locinfo — see `towlower_li` for why
/// per-character `get_locinfo()` is a 3x boot slowdown rather than a style nit.
#[inline]
unsafe fn toupper_li(c: i32, li: *mut sys::threadlocaleinfostruct) -> i32 {
    if (c as u32) < 256 {
        if pctype_at(li, c) & sys::_LEADBYTE != 0 {
            return c;
        }
        return *(*li).pcumap.offset(c as isize) as i32;
    }
    case_map_wide(c, li, LCMAP_UPPERCASE)
}
#[inline]
unsafe fn tolower_li(c: i32, li: *mut sys::threadlocaleinfostruct) -> i32 {
    if (c as u32) < 256 {
        if pctype_at(li, c) & sys::_LEADBYTE != 0 {
            return c;
        }
        return *(*li).pclmap.offset(c as isize) as i32;
    }
    case_map_wide(c, li, LCMAP_LOWERCASE)
}

/// `_toupper_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _toupper_l(c: i32, locale: *mut core::ffi::c_void) -> i32 {
    let li = locinfo_of(locale);
    if (c as u32) < 256 {
        if pctype_at(li, c) & sys::_LEADBYTE != 0 {
            return c;
        }
        return *(*li).pcumap.offset(c as isize) as i32;
    }
    case_map_wide(c, li, LCMAP_UPPERCASE)
}

/// `_tolower_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _tolower_l(c: i32, locale: *mut core::ffi::c_void) -> i32 {
    let li = locinfo_of(locale);
    if (c as u32) < 256 {
        if pctype_at(li, c) & sys::_LEADBYTE != 0 {
            return c;
        }
        return *(*li).pclmap.offset(c as isize) as i32;
    }
    case_map_wide(c, li, LCMAP_LOWERCASE)
}

/// `toupper` / `tolower (MSVCRT.@)`
///
/// NOT simply `_toupper_l(c, NULL)`. msvcrt short-circuits on the global
/// `initial_locale` ("setlocale has never been called"), taking a hardcoded ASCII fold.
/// Keep the branch: it is what the C does, and dropping it would also cost a
/// `get_locinfo()` call on the hottest path in the CRT (the perf gate is 2x).
#[no_mangle]
pub unsafe extern "C" fn toupper(c: i32) -> i32 {
    if initial_locale != 0 {
        return if c >= b'a' as i32 && c <= b'z' as i32 {
            c - b'a' as i32 + b'A' as i32
        } else {
            c
        };
    }
    _toupper_l(c, core::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn tolower(c: i32) -> i32 {
    if initial_locale != 0 {
        return if c >= b'A' as i32 && c <= b'Z' as i32 {
            c - b'A' as i32 + b'a' as i32
        } else {
            c
        };
    }
    _tolower_l(c, core::ptr::null_mut())
}

/// `_toupper` / `_tolower (MSVCRT.@)` — msvcrt's `/* sic */`.
///
/// UNCONDITIONAL arithmetic. No range check, no table, no locale. `_toupper('0')` is
/// 0x10. This is not an oversight in our transcription — it is what the C says, comment
/// and all, and it is the documented behaviour of these two exports. A "reasonable"
/// implementation here would be a bug.
#[no_mangle]
pub unsafe extern "C" fn _toupper(c: i32) -> i32 {
    c - 0x20
}
#[no_mangle]
pub unsafe extern "C" fn _tolower(c: i32) -> i32 {
    c + 0x20
}

// ── wave 10: the thread_data_t tokenizers ──────────────────────────────────
//
// REFUSED in wave 6c, for a good reason: they keep their cursor in
// `msvcrt_get_thread_data()->strtok_next`, and reaching that field meant hand-rolling a
// numeric struct offset — the `scan_magic_offsets` bug class. `msvcrt-sys` removes the
// obstacle: `offset_of!`, machine-generated, double-guarded.

/// `strtok (MSVCRT.@)`
///
/// NOT `strtok_s(str, delim, &data->strtok_next)`, however much it looks like it should
/// be. The two have completely different error contracts, three lines apart in the C:
///
///     strtok_s: if (!MSVCRT_CHECK_PMT(delim != NULL)) return NULL;      // errno + HANDLER
///               if (!MSVCRT_CHECK_PMT(ctx   != NULL)) return NULL;
///               if (!MSVCRT_CHECK_PMT(str != NULL || *ctx != NULL)) return NULL;
///     strtok:   if (!str) if (!(str = data->strtok_next)) return NULL;  // silence
///
/// A `strtok(NULL, d)` on a fresh thread returns NULL *quietly*. Delegating to
/// `strtok_s` would fire the invalid-parameter handler instead — observable effect #4,
/// which classify grades, and which an app can hook with `_set_invalid_parameter_handler`
/// and turn into an abort(). Transcribe the real one.
#[no_mangle]
pub unsafe extern "C" fn strtok(str_: *mut u8, delim: *const u8) -> *mut u8 {
    let data = sys::msvcrt_get_thread_data();
    let mut s = str_;

    if s.is_null() {
        s = (*data).strtok_next as *mut u8;
        if s.is_null() {
            return core::ptr::null_mut();
        }
    }

    // `*str &&` guards each strchr — strchr(delim, 0) would match delim's own NUL.
    while *s != 0 && h::byte_in(delim, *s) {
        s = s.add(1);
    }
    if *s == 0 {
        (*data).strtok_next = s as *mut core::ffi::c_char;
        return core::ptr::null_mut();
    }

    let ret = s;
    s = s.add(1);
    while *s != 0 && !h::byte_in(delim, *s) {
        s = s.add(1);
    }
    if *s != 0 {
        *s = 0;
        s = s.add(1);
    }
    (*data).strtok_next = s as *mut core::ffi::c_char;
    ret
}

/// `wcstok (MSVCRT.@)`
///
/// At `_MSVCR_VER == 0` this is the TWO-argument form, and it really is just its `_s`
/// sibling plus the per-thread cursor — unlike `strtok`, which is hand-written. (The
/// three-argument `wcstok` only exists at `_MSVCR_VER >= 140`; do not port that one.)
#[no_mangle]
pub unsafe extern "C" fn wcstok(str_: *mut u16, delim: *const u16) -> *mut u16 {
    let data = sys::msvcrt_get_thread_data();
    wcstok_s(
        str_,
        delim,
        core::ptr::addr_of_mut!((*data).wcstok_next) as *mut *mut u16,
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// WAVE 11 — path 2. Closing the lane's oldest blind spot (§9.60).
//
// Waves 7 and 8 shipped locale-INDEPENDENT bodies for a family of locale-DEPENDENT
// functions. They passed `classify` — and they had to, because the harness pinned the
// "C" locale, and in the "C" locale msvcrt takes an inline ASCII path:
//
//     int __cdecl _strnicmp_l(const char *s1, const char *s2, size_t count, _locale_t l) {
//         ...
//         if(!locinfo->lc_handle[LC_CTYPE]) {          // <-- path 1: the "C" locale
//             do { if ((c1 = *s1++) >= 'A' && c1 <= 'Z') c1 -= 'A' - 'a'; ... } while(...);
//             return (unsigned char)c1 - (unsigned char)c2;
//         }
//         do {                                          // <-- path 2: a REAL locale
//             c1 = _tolower_l((unsigned char)*s1++, locale);
//             c2 = _tolower_l((unsigned char)*s2++, locale);
//         } while(--count && c1 && c1==c2);
//         return c1-c2;                                 //     ...and a DIFFERENT return!
//     }
//
// We implemented path 1 only. Correct in the "C" locale; wrong the moment `setlocale` is
// called. That is not a bug the old harness could ever have caught — which is the actual
// lesson: **a green from an unexercised branch is not a green.** So wave 11 makes the
// harness exercise it (`subj_set_nls`/`ref_set_nls` plant a real `lc_handle[LC_CTYPE]`,
// and both sides reach the SAME shared NLS stub) and only then writes the branch.
//
// Note the two paths do not even agree on their return convention: path 1 returns
// `(unsigned char)c1 - (unsigned char)c2`, path 2 the raw `c1 - c2`.
// ═══════════════════════════════════════════════════════════════════════════

extern "system" {
    /// kernel32 — WINAPI, so **stdcall on i386**. `extern "C"` here would link on
    /// amd64/arm64 and fail only on i386 (the wave-9 `GetStringTypeW` bug).
    fn LCMapStringW(
        lcid: u32,
        mapflags: u32,
        src: *const u16,
        srclen: i32,
        dst: *mut u16,
        dstlen: i32,
    ) -> i32;
}

/// `_towlower_l (MSVCRT.@)` — the wide case-mapper the whole wide family funnels through.
///
/// `wint_t` is **16-bit** in msvcrt (hence `WEOF == 0xFFFF`), not the host's 32-bit.
///
/// ## Resolve the locale ONCE per CALL, never once per CHARACTER
///
/// The callers below (`_wcsicmp`, `_wcsnicmp`, the folders) run this per character. If
/// each of those iterations re-enters `get_locinfo()` — a TLS lookup — the function goes
/// from "a compare" to "a compare plus a TLS lookup per byte", and `_wcsicmp` is on the
/// boot's hot path (registry key comparisons). It cost a **3x boot slowdown**, which timed
/// out the kernel TDI self-test and turned the gate red. The C does not do this:
///
///     INT CDECL _wcsicmp_l(const wchar_t *s1, const wchar_t *s2, _locale_t locale) {
///         if(!locale) locale = get_current_locale_noalloc(&tmp);   // ONCE
///         do { c1 = _towlower_l(*str1++, locale); ... }            // ...then passed down
///
/// So the fold takes an ALREADY-RESOLVED locinfo, and the `_l` entry points below are thin
/// wrappers that resolve it once. Neither `classify` nor `oob_guard` can see this — only
/// `perf` and the smoke tests can, and `perf` was not benching this family (§9.63: "a perf
/// gate only defends what it BENCHES"). It is benched now.
#[inline]
unsafe fn towlower_li(c: u16, li: *mut sys::threadlocaleinfostruct) -> u16 {
    if (*li).lc_handle[sys::LC_CTYPE] == 0 {
        if c >= b'A' as u16 && c <= b'Z' as u16 {
            return c + (b'a' - b'A') as u16;
        }
        return c;
    }
    let mut ret: u16 = 0;
    if LCMapStringW(
        (*li).lc_handle[sys::LC_CTYPE],
        LCMAP_LOWERCASE,
        &c,
        1,
        &mut ret,
        1,
    ) == 0
    {
        return c; // NLS refused -> unchanged. Not an error, and no errno.
    }
    ret
}

/// `_towlower_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _towlower_l(c: u16, locale: *mut core::ffi::c_void) -> u16 {
    towlower_li(c, locinfo_of(locale))
}

#[inline]
unsafe fn towupper_li(c: u16, li: *mut sys::threadlocaleinfostruct) -> u16 {
    if (*li).lc_handle[sys::LC_CTYPE] == 0 {
        if c >= b'a' as u16 && c <= b'z' as u16 {
            return c - (b'a' - b'A') as u16;
        }
        return c;
    }
    let mut ret: u16 = 0;
    if LCMapStringW(
        (*li).lc_handle[sys::LC_CTYPE],
        LCMAP_UPPERCASE,
        &c,
        1,
        &mut ret,
        1,
    ) == 0
    {
        return c;
    }
    ret
}

/// `_towupper_l (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _towupper_l(c: u16, locale: *mut core::ffi::c_void) -> u16 {
    towupper_li(c, locinfo_of(locale))
}

/// `towlower` / `towupper (MSVCRT.@)` — plain forms. Unlike the narrow `tolower`, these
/// do NOT short-circuit on `initial_locale`; they are straight `_l` delegations.
#[no_mangle]
pub unsafe extern "C" fn towlower(c: u16) -> u16 {
    _towlower_l(c, core::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn towupper(c: u16) -> u16 {
    _towupper_l(c, core::ptr::null_mut())
}

// ═══════════════════════════════════════════════════════════════════════════
// WAVE 12 — the collation family. Unblocked by wave 11, and ONLY by wave 11.
//
// Every one of these branches on `lc_handle[LC_COLLATE]`:
//
//     int CDECL _strcoll_l(const char *s1, const char *s2, _locale_t locale) {
//         if(!locinfo->lc_handle[LC_COLLATE])
//             return strcmp(str1, str2);                       // path 1: the "C" locale
//         return CompareStringA(locinfo->lc_handle[LC_COLLATE], SORT_STRINGSORT,
//                   str1, -1, str2, -1) - CSTR_EQUAL;          // path 2
//     }
//
// Before wave 11 the harness could only ever see path 1, so flipping these would have
// been §9.60 all over again — a green from an unexercised branch. The NLS mode plants a
// real `lc_handle[LC_COLLATE]` AND makes the collation stub's answer differ from
// `strcmp`'s, so path 2 is genuinely under test (§9.75: exercising a branch proves
// nothing unless it can produce a DIFFERENT answer). Negative-controlled.
//
// Both sides reach the SAME `CompareStringA`/`LCMapStringA` — un-namespaced in wave 11,
// because kernel32 is the ENVIRONMENT, never code under test. So what is graded is the
// Rust's logic around the call: the flags, the lengths, and the `- CSTR_EQUAL` rebase.
// Windows' real collation ORDER is out of scope, correctly.
// ═══════════════════════════════════════════════════════════════════════════

const NORM_IGNORECASE: u32 = 0x0000_0001;
const SORT_STRINGSORT: u32 = 0x0000_1000;
const LCMAP_SORTKEY: u32 = 0x0000_0400;
const CSTR_EQUAL: i32 = 2;
/// The `size_t`-returning transforms use `INT_MAX` as their error value — 0x7FFFFFFF,
/// NOT `usize::MAX`.
const XFRM_ERR: usize = 0x7FFF_FFFF;

// `CompareStringA` comes from the prelude, declared from winnls.h. Keeping a second,
// differently-spelled declaration here is what `clashing_extern_declarations` exists to catch.
use crate::prelude::{CompareStringA, __crtLCMapStringA};

extern "system" {
    fn CompareStringW(lcid: u32, f: u32, s1: *const u16, l1: i32, s2: *const u16, l2: i32) -> i32;
    fn LCMapStringA(lcid: u32, f: u32, src: *const u8, sl: i32, dst: *mut u8, dl: i32) -> i32;
}

/// The `LC_COLLATE` handle — 0 in the "C" locale.
#[inline]
unsafe fn collate_handle(locale: *mut core::ffi::c_void) -> u32 {
    (*locinfo_of(locale)).lc_handle[sys::LC_COLLATE]
}

// ── narrow ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _strcoll_l(s1: *const u8, s2: *const u8, l: *mut core::ffi::c_void) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        return strcmp(s1, s2);
    }
    CompareStringA(h, SORT_STRINGSORT, s1 as *const i8, -1, s2 as *const i8, -1) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn strcoll(s1: *const u8, s2: *const u8) -> i32 {
    _strcoll_l(s1, s2, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _stricoll_l(s1: *const u8, s2: *const u8, l: *mut core::ffi::c_void) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        return _stricmp(s1, s2);
    }
    CompareStringA(h, NORM_IGNORECASE, s1 as *const i8, -1, s2 as *const i8, -1) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn _stricoll(s1: *const u8, s2: *const u8) -> i32 {
    _stricoll_l(s1, s2, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _strncoll_l(
    s1: *const u8,
    s2: *const u8,
    count: usize,
    l: *mut core::ffi::c_void,
) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        return strncmp(s1, s2, count);
    }
    // NOTE the lengths: `strnlen(str, count)`, not -1. A bounded collate tells
    // CompareStringA exactly how many bytes to look at.
    CompareStringA(
        h,
        SORT_STRINGSORT,
        s1 as *const i8,
        strnlen(s1, count) as i32,
        s2 as *const i8,
        strnlen(s2, count) as i32,
    ) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn _strncoll(s1: *const u8, s2: *const u8, count: usize) -> i32 {
    _strncoll_l(s1, s2, count, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _strnicoll_l(
    s1: *const u8,
    s2: *const u8,
    count: usize,
    l: *mut core::ffi::c_void,
) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        return _strnicmp(s1, s2, count);
    }
    CompareStringA(
        h,
        NORM_IGNORECASE,
        s1 as *const i8,
        strnlen(s1, count) as i32,
        s2 as *const i8,
        strnlen(s2, count) as i32,
    ) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn _strnicoll(s1: *const u8, s2: *const u8, count: usize) -> i32 {
    _strnicoll_l(s1, s2, count, core::ptr::null_mut())
}

/// `_strxfrm_l (MSVCRT.@)` — the sort-key transform. Five distinct returns.
#[no_mangle]
pub unsafe extern "C" fn _strxfrm_l(
    dest: *mut u8,
    src: *const u8,
    len: usize,
    l: *mut core::ffi::c_void,
) -> usize {
    if !check_pmt(!src.is_null()) {
        return XFRM_ERR;
    }
    if !check_pmt(!dest.is_null() || len == 0) {
        return XFRM_ERR;
    }
    let mut len = len;
    if len > XFRM_ERR {
        len = XFRM_ERR; // the C FIXMEs and clamps to INT_MAX
    }

    let h = collate_handle(l);
    if h == 0 {
        strncpy(dest, src, len);
        return strlen(src);
    }

    let ret = LCMapStringA(h, LCMAP_SORTKEY, src, -1, core::ptr::null_mut(), 0);
    if ret == 0 {
        if len != 0 {
            *dest = 0;
        }
        *_errno() = EILSEQ; // <-- MISSED IT THE FIRST TIME. See the note on _wcsxfrm_l.
        return XFRM_ERR;
    }
    let ret = ret as usize;
    if len == 0 {
        return ret - 1;
    }
    if ret > len {
        *dest = 0;
        *_errno() = ERANGE; // <-- and this one
        return ret - 1;
    }
    (LCMapStringA(h, LCMAP_SORTKEY, src, -1, dest, len as i32) - 1) as usize
}
#[no_mangle]
pub unsafe extern "C" fn strxfrm(dest: *mut u8, src: *const u8, len: usize) -> usize {
    _strxfrm_l(dest, src, len, core::ptr::null_mut())
}

// ── wide ──────────────────────────────────────────────────────────────────

/// `_wcscoll_l` / `wcscoll (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcscoll_l(
    s1: *const u16,
    s2: *const u16,
    l: *mut core::ffi::c_void,
) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        return wcscmp(s1, s2);
    }
    CompareStringW(h, SORT_STRINGSORT, s1, -1, s2, -1) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn wcscoll(s1: *const u16, s2: *const u16) -> i32 {
    _wcscoll_l(s1, s2, core::ptr::null_mut())
}

/// `_wcsicoll_l` / `_wcsicoll (MSVCRT.@)`
///
/// Path 1 is an INLINE ASCII fold and returns the RAW `c1 - c2` — it does NOT delegate to
/// `_wcsicmp` (which folds through `_towlower_l`). Same family, different code.
#[no_mangle]
pub unsafe extern "C" fn _wcsicoll_l(
    s1: *const u16,
    s2: *const u16,
    l: *mut core::ffi::c_void,
) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        let (mut p, mut q) = (s1, s2);
        let (mut c1, mut c2);
        loop {
            c1 = *p;
            p = p.add(1);
            if c1 >= b'A' as u16 && c1 <= b'Z' as u16 {
                c1 += 32;
            }
            c2 = *q;
            q = q.add(1);
            if c2 >= b'A' as u16 && c2 <= b'Z' as u16 {
                c2 += 32;
            }
            if c1 == 0 || c1 != c2 {
                break;
            }
        }
        return c1 as i32 - c2 as i32;
    }
    CompareStringW(h, NORM_IGNORECASE, s1, -1, s2, -1) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn _wcsicoll(s1: *const u16, s2: *const u16) -> i32 {
    _wcsicoll_l(s1, s2, core::ptr::null_mut())
}

/// `_wcsncoll_l` / `_wcsncoll (MSVCRT.@)`
#[no_mangle]
pub unsafe extern "C" fn _wcsncoll_l(
    s1: *const u16,
    s2: *const u16,
    count: usize,
    l: *mut core::ffi::c_void,
) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        return wcsncmp(s1, s2, count);
    }
    CompareStringW(
        h,
        SORT_STRINGSORT,
        s1,
        wcsnlen(s1, count) as i32,
        s2,
        wcsnlen(s2, count) as i32,
    ) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn _wcsncoll(s1: *const u16, s2: *const u16, count: usize) -> i32 {
    _wcsncoll_l(s1, s2, count, core::ptr::null_mut())
}

/// `_wcsnicoll_l` / `_wcsnicoll (MSVCRT.@)`
///
/// Path 1 tests `if (!count) return 0;` FIRST, then does its own ASCII fold.
#[no_mangle]
pub unsafe extern "C" fn _wcsnicoll_l(
    s1: *const u16,
    s2: *const u16,
    mut count: usize,
    l: *mut core::ffi::c_void,
) -> i32 {
    let h = collate_handle(l);
    if h == 0 {
        if count == 0 {
            return 0;
        }
        let (mut p, mut q) = (s1, s2);
        let (mut c1, mut c2);
        loop {
            c1 = *p;
            p = p.add(1);
            if c1 >= b'A' as u16 && c1 <= b'Z' as u16 {
                c1 += 32;
            }
            c2 = *q;
            q = q.add(1);
            if c2 >= b'A' as u16 && c2 <= b'Z' as u16 {
                c2 += 32;
            }
            count -= 1;
            if count == 0 || c1 == 0 || c1 != c2 {
                break;
            }
        }
        return c1 as i32 - c2 as i32;
    }
    CompareStringW(
        h,
        NORM_IGNORECASE,
        s1,
        wcsnlen(s1, count) as i32,
        s2,
        wcsnlen(s2, count) as i32,
    ) - CSTR_EQUAL
}
#[no_mangle]
pub unsafe extern "C" fn _wcsnicoll(s1: *const u16, s2: *const u16, count: usize) -> i32 {
    _wcsnicoll_l(s1, s2, count, core::ptr::null_mut())
}

/// `_wcsxfrm_l` / `wcsxfrm (MSVCRT.@)`
///
/// The subtle one. `LCMapStringW(LCMAP_SORTKEY, ...)` writes a **byte** sort key into a
/// `wchar_t*` buffer, and the C then expands it in place — **walking BACKWARDS**, so the
/// widening never clobbers a byte it has not read yet:
///
///     ret = LCMapStringW(..., dest, len) - 1;
///     for(i=ret; i>=0; i--)
///         dest[i] = ((unsigned char*)dest)[i];
///
/// Transcribe the direction, not just the effect.
#[no_mangle]
pub unsafe extern "C" fn _wcsxfrm_l(
    dest: *mut u16,
    src: *const u16,
    len: usize,
    l: *mut core::ffi::c_void,
) -> usize {
    if !check_pmt(!src.is_null()) {
        return XFRM_ERR;
    }
    if !check_pmt(!dest.is_null() || len == 0) {
        return XFRM_ERR;
    }
    let mut len = len;
    if len > XFRM_ERR {
        len = XFRM_ERR;
    }

    let h = collate_handle(l);
    if h == 0 {
        wcsncpy(dest, src, len);
        return wcslen(src);
    }

    // ── THE TWO ERRNO SETS, AND HOW THEY WERE ALMOST MISSED ──────────────
    //
    // I read this function through `sed ... | grep -vE "^\s*\*|^/\*"` to strip the
    // comment lines — and that pattern ALSO eats every statement beginning with a
    // dereference. `        *_errno() = ERANGE;` starts with whitespace-then-star. So the
    // C I read had no errno in it, and the first version of this flip had none either.
    // classify convicted it (`ret=17` on both sides, `errno=ERANGE` on msvcrt's).
    //
    // This is the SECOND time the same filter has eaten load-bearing C (`*p++ = *src++;`
    // in wave 6c). NEVER read C through a comment-stripping grep. Read it unfiltered.
    let ret = LCMapStringW(h, LCMAP_SORTKEY, src, -1, core::ptr::null_mut(), 0);
    if ret == 0 {
        if len != 0 {
            *dest = 0;
        }
        *_errno() = EILSEQ;
        return XFRM_ERR;
    }
    let ret = ret as usize;
    if len == 0 {
        return ret - 1;
    }
    if ret > len {
        *dest = 0;
        *_errno() = ERANGE;
        return ret - 1;
    }

    let n = (LCMapStringW(h, LCMAP_SORTKEY, src, -1, dest, len as i32) - 1) as isize;
    // BACKWARDS — byte i must be read before wide slot i is written over it.
    let bytes = dest as *const u8;
    let mut i = n;
    while i >= 0 {
        let b = *bytes.offset(i);
        *dest.offset(i) = b as u16;
        i -= 1;
    }
    n as usize
}
#[no_mangle]
pub unsafe extern "C" fn wcsxfrm(dest: *mut u16, src: *const u16, len: usize) -> usize {
    _wcsxfrm_l(dest, src, len, core::ptr::null_mut())
}

// ═══════════════════════════════════════════════════════════════════════════
// WAVE 13 — the float PARSER (msvcrt's own bignum, ported).
//
// The first module in this crate that is a genuine TRANSLATION rather than an
// assembly: cnt-rust's float parser is a different algorithm from a different
// lineage and disagrees with msvcrt by double-digit ULPs (wave 5's probe, which
// stays in `classify` as a DO-NOT-FLIP record). So `strtod` here is not a wrapper
// around someone else's parser — it IS msvcrt's parser, transcribed.
//
// ── i386 works too — via x87 ST(0), the way cnt-rust already does it ──
//
// i386 msvcrt is `-march=pentium` (pre-SSE) and the i686 user target carries
// `-C target-feature=-sse,-sse2`. The KEY fact (learned from cnt-rust/src/cabi/strtod.rs
// and core_rs/floatcvt.rs, which shipped this painstakingly): an `extern "C" fn -> f64`
// returns via the x87 `ST(0)` register, needs NO SSE, and IS the i386 C ABI msvcrt's
// callers expect — so it compiles on no-SSE i386. What hard-errors on no-SSE i386 is a
// **Rust-ABI** `-> f64` (that one wants xmm) and the SSE-pulling `f64`/`f32` std methods
// (`is_infinite`, `is_finite`, `abs`). fpnum.rs therefore keeps every f64-returning
// helper `extern "C"` and uses bit-based finite/inf predicates — see its `no_sse` notes.
// ═══════════════════════════════════════════════════════════════════════════
mod bnum;
mod fpnum;
// Wave 14b — msvcrt's printf ENGINE (printf.h transcription; shares bnum with
// the wave-13 parser). `pub` so msvcrt-difftest can drive it directly against
// the genuine pf_printf_a (the pfflat_* oracle; 142 narrow + 30 wide rows
// byte-identical). The six engine symbols (pf_printf_a/_w, arg_clbk_valist/
// _positional, create_positional_ctx_a/_w) are #[no_mangle]: wcs.c's ~50
// C printf shells (sprintf, _snwprintf, _vscprintf_p_l, …) are trivial
// va_start wrappers calling the engine BY NAME, so whole-archive first-match
// flips every printf-family export's machinery to Rust while the shells stay
// C until a later varargs-shell wave.
pub mod printf;

// Wave 14c — the shells themselves. Flipping the engine's six symbols swung
// file.c's stream family but NOT wcs.c's own shells: clang had already
// IPA-specialized the same-TU engine into them, and intra-TU inlining never
// consults the linker (symbol replacement flips a TU's CALLERS, not its
// HOUSEMATES). These are the 31 `va_list`-taking entry points, transcribed;
// the `...` ones are C va_start trampolines in msvcrt's rust_varargs_shim.c
// because stable Rust cannot DEFINE a variadic. `pub` for the difftest.
pub mod printf_shells;

// Wave 16 — the last of wcs.c: the wide integer parsers and their clamps, the
// case folders and comparers, and the whole wchar->multibyte family. Its own
// module rather than more lib.rs because the wcstombs/wctomb group shares two
// file-static helpers (`wcsrtombs_l`, `wcsrtombs_s_l`) that nothing else may see,
// exactly as they are file-static in wcs.c.
pub mod wcs_rest;

// The import surface every whole-C-file translation compiles against. It MOVED to
// `crt-shared-rust` with cpp.c/except.c/undname.c, because that crate is linked into
// ucrtbase.dll where this one is not, and it cannot depend on msvcrt's floor to supply
// it. Re-exported rather than re-declared so `crate::prelude::*` keeps resolving here —
// one copy, two crates, which is the rule for a header the tree ports per module.
pub use crt_shared_rust::prelude;

// Wave 15 — whole-C-file rustconv translations (mbcs.c first). See src/msvcrt/mod.rs.
mod msvcrt;

// Wave 16 — string.c's remaining `_l` exports (plus `_strdup`, `_atoflt`, `I10_OUTPUT`).
pub mod string_l;

// Wave 16 — the scanf ENGINE. `dll/win32/msvcrt/scanf.h` is a template `#include`d 16
// times from scanf.c under every WIDE_SCANF/SECURE/STRING/STRING_LEN/CONSOLE
// combination; this is one generic engine over those same three axes, plus the 16
// `v*scanf*_l` entry points the C keeps `static` and the four `__stdio_common_*` doors.
// `%f` goes through wave 13's `fpnum_parse`/`fpnum_double`, the same bignum `strtod`
// uses. The 40 `...` entry points cannot be Rust (no `va_start` on stable) and live in
// `dll/win32/msvcrt/rust_scanf_shim.c`. `pub` for the difftest.
pub mod scanf;
// The C `...` entry points (were the rust_*_shim.c files), as va_entry! thunks.
mod va_entries;
