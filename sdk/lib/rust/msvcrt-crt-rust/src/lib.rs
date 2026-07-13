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
mod h {
    #[inline]
    pub(super) unsafe fn cstr_slice<'a>(p: *const u8) -> &'a [u8] {
        if p.is_null() {
            return &[];
        }
        let mut n = 0usize;
        while *p.add(n) != 0 {
            n += 1;
            // rustc-1.92 host (linux) loop-idiom recognizes this NUL-scan as
            // `strlen`; inlined into the #[no_mangle] `strlen` shim it rewrites
            // to a self-call that folds to `return 0` (cnt-difftest diff_strlen
            // under --release). black_box breaks the match. cfg'd to the host so
            // the windows-gnu KERNEL codegen (idiom never fires there) is
            // byte-for-byte unchanged.
            #[cfg(target_os = "linux")]
            {
                n = core::hint::black_box(n);
            }
        }
        core::slice::from_raw_parts(p, n)
    }
    #[inline]
    pub(super) unsafe fn cstr_slice_bounded<'a>(p: *const u8, cap: usize) -> &'a [u8] {
        if p.is_null() {
            return &[];
        }
        let mut n = 0usize;
        while n < cap && *p.add(n) != 0 {
            n += 1;
            // see cstr_slice: defeat the host loop-idiom strlen miscompile.
            #[cfg(target_os = "linux")]
            {
                n = core::hint::black_box(n);
            }
        }
        core::slice::from_raw_parts(p, n)
    }
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
    pub(super) unsafe fn raw_mut_slice<'a>(p: *mut u8, n: usize) -> &'a mut [u8] {
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
        crate::strtoi64_contract(nptr, core::ptr::null_mut(), 10)
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

mod mstrcat {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char* strcat(char *s, const char *append)`
    /// preserved misfeature: no bounds check (tcscat.h); appends through+incl. NUL at the existing terminator
    #[no_mangle]
    /// Single-pass copy — a PERF contract (see `strcmp`). The slice form scanned
    /// dst, then scanned src, THEN copied: three passes. The C is two —
    /// `strcpy(dst + strlen(dst), src)`, whose copy loop finds src's NUL as it
    /// goes (`while ((*p++ = *src++));`). Measured 2.5x before this.
    pub unsafe extern "C" fn strcat(s: *mut u8, append: *const u8) -> *mut u8 {
        if s.is_null() {
            return s;
        }
        let mut d = s;
        while *d != 0 {
            d = d.add(1);
        }
        crate::cstr_copy(d, crate::nz(append));
        s
    }

}
pub use mstrcat::*;

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

mod mstrcpy {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char* strcpy(char *to, const char *from)`
    /// preserved misfeature: no bounds check (tcscpy.h trusts the dst buffer); copies through and incl. the terminating NUL
    #[no_mangle]
    /// Single-pass — see `strcat`. C: `while ((*p++ = *src++));`
    pub unsafe extern "C" fn strcpy(to: *mut u8, from: *const u8) -> *mut u8 {
        crate::cstr_copy(to, crate::nz(from));
        to
    }

}
pub use mstrcpy::*;

mod mstrcspn {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `size_t strcspn(const char *s, const char *reject)`
    /// preserved misfeature: complement of strspn (_x inverted); empty reject → strlen(s)
    #[no_mangle]
    /// Single-pass over `s`. This mirrors the C most directly of all of them —
    /// msvcrt's `strcspn` already builds a `BOOL rejects[256]` table and then
    /// walks the subject once.
    pub unsafe extern "C" fn strcspn(s: *const u8, reject: *const u8) -> usize {
        let set = crate::ByteSet::of(reject);
        let mut p = crate::nz(s);
        let mut n = 0usize;
        while *p != 0 && !set.has(*p) {
            p = p.add(1);
            n += 1;
        }
        n
    }

}
pub use mstrcspn::*;

mod mstrlen {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `size_t strlen(const char *s)`
    /// preserved misfeature: strlen(NULL) == 0 (textbook libc: UB) — tcslen.h explicit NULL guard
    #[no_mangle]
    pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
        let s_s = cstr_slice(s);
        cnt_rust::core_rs::string::len_of(s_s) as usize
    }

}
pub use mstrlen::*;

mod mstrncat {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char* strncat(char *dst, const char *src, size_t n)`
    /// preserved misfeature: n==0 writes nothing at all (no terminator) and returns dst unchanged (tcsncat.h); otherwise always terminates after ≤n appended
    #[no_mangle]
    pub unsafe extern "C" fn strncat(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        if n == 0 { return dst; }
        let dl = cstr_slice(dst as *const u8).len();
        let sc = cstr_slice_bounded(src, n);
        let m = if sc.len() < n { sc.len() } else { n };
        let tail = raw_mut_slice(dst.add(dl), m + 1);
        cnt_rust::core_rs::string::ncat(tail, sc, n);
        dst
    }

}
pub use mstrncat::*;

mod mstrncpy {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *strncpy(char *dst, const char *src, size_t n)`
    /// preserved misfeature: no NUL-termination when src len >= n (the classic strncpy footgun) — tcsncpy.h
    #[no_mangle]
    pub unsafe extern "C" fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        let dst_s = raw_mut_slice(dst, n);
        let src_s = cstr_slice_bounded(src, n);
        let _ = cnt_rust::core_rs::string::ncpy(dst_s, src_s);
        dst
    }

}
pub use mstrncpy::*;

mod mstrnlen {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `size_t strnlen(const char *str, size_t maxlen)`
    /// preserved misfeature: NULL→0 (tcsnlen.h guard); never reads past maxlen; result = min(strlen,maxlen)
    #[no_mangle]
    pub unsafe extern "C" fn strnlen(str: *const u8, maxlen: usize) -> usize {
        let str_s = cstr_slice_bounded(str, maxlen);
        cnt_rust::core_rs::string::len_of(str_s) as usize
    }

}
pub use mstrnlen::*;

mod mstrpbrk {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char* strpbrk(const char *s1, const char *s2)`
    /// preserved misfeature: first s1 char in s2 else NULL; empty s2→NULL. strpbrk.c's 3+-char path bitmap is sized by BIT_SIZE=sizeof(unsigned long)*8 (LP32=32 on target). Core models the correct first-in-set; diff fuzzes set≤2 only (the strchr fast paths — host-correct; 3+ UBs `1<<>=32` on LP64 host, unfuzzable, same caveat class as itoa long-family / strtoul ULONG_MAX). set size doesn't change find_any's code path so ≤2 fully pins it.
    #[no_mangle]
    /// Single-pass over `s1`. C:
    /// `for (; *str; str++) if (strchr(accept, *str)) return (char*)str; return NULL;`
    pub unsafe extern "C" fn strpbrk(s1: *const u8, s2: *const u8) -> *mut u8 {
        let set = crate::ByteSet::of(s2);
        let mut p = crate::nz(s1);
        while *p != 0 {
            if set.has(*p) {
                return p as *mut u8;
            }
            p = p.add(1);
        }
        core::ptr::null_mut()
    }

}
pub use mstrpbrk::*;

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

mod mstrspn {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `size_t strspn(const char *s, const char *accept)`
    /// preserved misfeature: bitmap over unsigned-char (0..255); empty accept → 0; NUL terminates both (never in set)
    #[no_mangle]
    /// Single-pass over `s` (see the `ByteSet` note). C:
    /// `for (ptr = str; *ptr; ptr++) if (!strchr(accept, *ptr)) break; return ptr - str;`
    pub unsafe extern "C" fn strspn(s: *const u8, accept: *const u8) -> usize {
        let set = crate::ByteSet::of(accept);
        let mut p = crate::nz(s);
        let mut n = 0usize;
        while *p != 0 && set.has(*p) {
            p = p.add(1);
            n += 1;
        }
        n
    }

}
pub use mstrspn::*;

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
        let ret = crate::strtoi64_contract(nptr, endptr, base);
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
        let ret = crate::strtoi64_contract(nptr, end, base);
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
static EMPTY_A: [u8; 1] = [0];
static EMPTY_W: [u16; 1] = [0];

#[inline]
fn nz(p: *const u8) -> *const u8 {
    if p.is_null() {
        EMPTY_A.as_ptr()
    } else {
        p
    }
}

#[inline]
fn wnz(p: *const u16) -> *const u16 {
    if p.is_null() {
        EMPTY_W.as_ptr()
    } else {
        p
    }
}

/// A 256-bit membership set over the bytes of a C string, built in ONE pass.
///
/// This is what msvcrt's own `strcspn` does (`BOOL rejects[256]` + `memset`), and
/// it is why the set-based searchers can be single-pass over the *subject*: the
/// slice-based core forced a `cstr_slice` scan of the subject up front, and the
/// perf gate showed that whole family paying ~2x for it.
///
/// The set's terminating NUL is NOT a member — which is exactly right, because
/// every caller below only tests bytes it has already proven non-NUL. (The C
/// gets this for free: it calls `strchr(accept, *ptr)` with `*ptr != 0`.)
struct ByteSet([u64; 4]);

impl ByteSet {
    #[inline]
    unsafe fn of(mut p: *const u8) -> Self {
        let mut w = [0u64; 4];
        if !p.is_null() {
            while *p != 0 {
                let b = *p as usize;
                w[b >> 6] |= 1u64 << (b & 63);
                p = p.add(1);
            }
        }
        ByteSet(w)
    }
    #[inline]
    fn has(&self, b: u8) -> bool {
        let b = b as usize;
        self.0[b >> 6] & (1u64 << (b & 63)) != 0
    }
}

/// `while ((*dst++ = *src++));` — copy `src`'s bytes AND its terminating NUL, in
/// ONE pass. The C never computes `strlen(src)` and neither do we.
///
/// (The slice-based core cannot express this: `cpy(&mut [u8], &[u8])` needs both
/// lengths up front, so it costs a scan of `src` before a single byte moves.
/// That is the whole reason the copy/compare/search families are transcribed
/// here rather than routed through cnt-rust — see the note on `strcmp`.)
///
/// # Safety
/// `dst` must have room for `strlen(src) + 1` bytes — the same contract the C
/// `strcpy` places on its caller.
#[inline]
unsafe fn cstr_copy(mut dst: *mut u8, mut src: *const u8) {
    loop {
        let b = *src;
        *dst = b;
        if b == 0 {
            return;
        }
        dst = dst.add(1);
        src = src.add(1);
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

extern "C" {
    /// msvcrt's own per-thread errno slot. Inside msvcrt.dll this is the real C
    /// one; in the harness it is `subjectimpl.c`'s (deliberately separate from
    /// the reference's, so each side's errno can be asserted independently).
    fn _errno() -> *mut i32;
    /// msvcrt's invalid-parameter handler. Default behaviour is "no handler
    /// installed → return", but an app CAN install one that aborts, so the call
    /// itself is part of the contract and must actually happen.
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

/// `_strtoi64_l` with the "C" locale: msvcrt's front door + engine + errno.
/// Shared by `strtol`, `strtoul`, `_atoi64` (and `_strtoi64` once flipped).
#[inline]
unsafe fn strtoi64_contract(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> i64 {
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

    let r = cnt_rust::core_rs::radix::parse_i64_clamped(h::cstr_slice(nptr), base);
    if !endptr.is_null() {
        *endptr = nptr.add(r.end) as *mut u8;
    }
    if r.overflowed {
        *_errno() = ERANGE;
    }
    r.value
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
    strtoi64_contract(nptr, endptr, base)
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
pub unsafe extern "C" fn _strtoui64(nptr: *const u8, endptr: *mut *mut u8, mut base: i32) -> u64 {
    if !check_pmt(!nptr.is_null()) {
        return 0;
    }
    if !check_pmt(base == 0 || base >= 2) {
        return 0;
    }
    if !check_pmt(base <= 36) {
        return 0;
    }
    let p = nptr;
    let mut q = nptr;
    let mut negative = false;
    let mut got_digit = false;
    let mut ret: u64 = 0;

    while matches!(*q, b' ' | 0x09..=0x0d) {
        q = q.add(1);
    }
    if *q == b'-' {
        negative = true;
        q = q.add(1);
    } else if *q == b'+' {
        q = q.add(1);
    }
    if (base == 0 || base == 16) && *q == b'0' && (*q.add(1)).to_ascii_lowercase() == b'x' {
        base = 16;
        q = q.add(2);
    }
    if base == 0 {
        base = if *q == b'0' { 8 } else { 10 };
    }
    let bm = base as u64;
    while *q != 0 {
        let cur = (*q).to_ascii_lowercase();
        let v: u64 = if cur.is_ascii_digit() {
            if cur >= b'0' + base as u8 {
                break;
            }
            (cur - b'0') as u64
        } else {
            if cur < b'a' || cur >= b'a' + (base - 10) as u8 {
                break;
            }
            (cur - b'a') as u64 + 10
        };
        got_digit = true;
        q = q.add(1);
        // Same short-circuit safety as the signed engine: the second disjunct is
        // only reached once `ret <= u64::MAX / bm`, so `ret * bm` cannot overflow.
        if ret > u64::MAX / bm || ret * bm > u64::MAX - v {
            ret = u64::MAX;
            *_errno() = ERANGE;
        } else {
            ret = ret * bm + v;
        }
    }
    if !endptr.is_null() {
        *endptr = (if got_digit { q } else { p }) as *mut u8;
    }
    if negative {
        0u64.wrapping_sub(ret)
    } else {
        ret
    }
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

/// `_strnicmp_l(s1, s2, count, NULL)` — the C-locale path.
#[inline]
unsafe fn strnicmp_core(s1: *const u8, s2: *const u8, mut count: usize) -> i32 {
    if count == 0 {
        return 0; // before the front door, per the C
    }
    // msvcrt.dll builds `_MSVCR_VER == 0`, which takes the NARROW check — no
    // `count <= INT_MAX` arm (that one is _MSVCR_VER >= 80).
    if !check_pmt(!s1.is_null() && !s2.is_null()) {
        return NLSCMPERROR;
    }
    // `char` is SIGNED: the C reads into an `int`, folds on the sign-extended
    // value, and only casts to `unsigned char` in the RETURN. Both halves matter.
    let (mut p, mut q) = (s1 as *const i8, s2 as *const i8);
    let (mut c1, mut c2);
    loop {
        // The C reads `c1 -= 'A' - 'a';` — and 'A'-'a' is NEGATIVE (-32), so it
        // ADDS 32, folding uppercase DOWN to lowercase. Transcribing the `-=`
        // literally as `c1 -= 32` maps 'I' to ')' and classify convicts it
        // instantly. Spelled out here so nobody "fixes" it back.
        c1 = *p as i32;
        p = p.add(1);
        if c1 >= b'A' as i32 && c1 <= b'Z' as i32 {
            c1 += 32; // c1 -= ('A' - 'a')
        }
        c2 = *q as i32;
        q = q.add(1);
        if c2 >= b'A' as i32 && c2 <= b'Z' as i32 {
            c2 += 32;
        }
        count -= 1; // C: `while(--count && c1 && c1==c2)`
        if count == 0 || c1 == 0 || c1 != c2 {
            break;
        }
    }
    (c1 as u8) as i32 - (c2 as u8) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _strnicmp(s1: *const u8, s2: *const u8, count: usize) -> i32 {
    strnicmp_core(s1, s2, count)
}

#[no_mangle]
pub unsafe extern "C" fn _stricmp(s1: *const u8, s2: *const u8) -> i32 {
    strnicmp_core(s1, s2, i32::MAX as usize) // the C passes INT_MAX
}

/// `_memicmp_l(s1, s2, len, NULL)`.
///
/// Note how this differs from `_strnicmp`: it folds through `_tolower_l`, which
/// in the C locale passes non-ASCII (including NEGATIVE chars) straight through,
/// and it returns that difference RAW — no `(unsigned char)` cast. So on high
/// bytes the two functions return different numbers. They are not the same
/// comparison with a different bound.
#[no_mangle]
pub unsafe extern "C" fn _memicmp(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    mut len: usize,
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
    let (mut p, mut q) = (s1 as *const i8, s2 as *const i8);
    let mut ret = 0i32;
    while len != 0 {
        len -= 1;
        ret = c_lower(*p as i32) - c_lower(*q as i32);
        if ret != 0 {
            break;
        }
        p = p.add(1);
        q = q.add(1);
    }
    ret
}

/// `_strlwr_s_l` / `_strupr_s_l` with the C locale.
///
/// The handler NEVER fires (hand-rolled `*_errno() = EINVAL`, the `_ultoa_s`
/// shape). "No terminator inside len" is its own EINVAL and truncates dst to "".
#[inline]
unsafe fn strcase_s_core(str_: *mut u8, len: usize, upper: bool) -> i32 {
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
    let mut p = str_;
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
    0
}

#[no_mangle]
pub unsafe extern "C" fn _strlwr_s(str_: *mut u8, len: usize) -> i32 {
    strcase_s_core(str_, len, false)
}

#[no_mangle]
pub unsafe extern "C" fn _strupr_s(str_: *mut u8, len: usize) -> i32 {
    strcase_s_core(str_, len, true)
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
    let (mut p, mut q) = (s1, s2);
    let (mut c1, mut c2);
    loop {
        c1 = cw_lower(*p);
        p = p.add(1);
        c2 = cw_lower(*q);
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
    let (mut p, mut q) = (s1, s2);
    let (mut c1, mut c2);
    loop {
        c1 = cw_lower(*p);
        p = p.add(1);
        c2 = cw_lower(*q);
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
    let mut left = n;
    let mut ptr = str_;
    while left != 0 {
        left -= 1;
        if *ptr == 0 {
            return 0;
        }
        *ptr = if upper { cw_upper(*ptr) } else { cw_lower(*ptr) };
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

extern "C" {
    /// msvcrt's wide ctype table — a POINTER to 256 WORDs of mask bits.
    /// In msvcrt.dll this is the real table; in the harness, subjectimpl.c's.
    /// (Data, so no calling convention — `extern "C"` is right here.)
    static MSVCRT__pwctype: *const u16;
}

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
