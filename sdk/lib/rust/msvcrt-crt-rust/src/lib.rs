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

    /// `__int64 _atoi64(const char *nptr)`
    /// preserved misfeature: NULL→0; i64 wrapping accumulate, NO overflow error; no (int) trunc (unlike atoi)
    #[no_mangle]
    pub unsafe extern "C" fn _atoi64(nptr: *const u8) -> i64 {
        let nptr_s = cstr_slice(nptr);
        let (v, _of) = cnt_rust::core_rs::radix::parse_i64_wrapping(nptr_s);
        v as i64
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
        // `swap_nonoverlapping`, and `-C no-prepopulate-passes` (the §378
        // miscompile guard we must NOT drop — it's why memset didn't get DCE'd
        // into a spin) leaves it un-inlined AND un-emitted → undefined symbol at
        // link. An explicit swap loop is semantically identical to `reverse()`
        // (both are an in-place byte reversal) and pulls no generic.
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
    pub unsafe extern "C" fn strcat(s: *mut u8, append: *const u8) -> *mut u8 {
        let dl = cstr_slice(s as *const u8).len();
        let a = cstr_slice(append);
        let m = a.len();
        let tail = raw_mut_slice(s.add(dl), m + 1);
        cnt_rust::core_rs::string::cat(tail, a);
        s
    }

}
pub use mstrcat::*;

mod mstrchr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *strchr(const char *s, int c)`
    /// preserved misfeature: the terminating NUL is a valid target: c==0 returns ptr to the NUL (Some(len)), not NULL; c truncated to char
    #[no_mangle]
    pub unsafe extern "C" fn strchr(s: *const u8, c: i32) -> *mut u8 {
        let s_s = cstr_slice(s);
        match cnt_rust::core_rs::string::find_char(s_s, c as i32) {
            Some(i) => s.add(i) as *mut u8,
            None => core::ptr::null_mut(),
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
    pub unsafe extern "C" fn strcpy(to: *mut u8, from: *const u8) -> *mut u8 {
        let s = cstr_slice(from);
        let m = s.len();
        let d = raw_mut_slice(to, m + 1);
        cnt_rust::core_rs::string::cpy(d, s);
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
    pub unsafe extern "C" fn strcspn(s: *const u8, reject: *const u8) -> usize {
        let s_s = cstr_slice(s);
        let reject_s = cstr_slice(reject);
        cnt_rust::core_rs::string::span_out(s_s, reject_s) as usize
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
    pub unsafe extern "C" fn strpbrk(s1: *const u8, s2: *const u8) -> *mut u8 {
        let s1_s = cstr_slice(s1);
        let s2_s = cstr_slice(s2);
        match cnt_rust::core_rs::string::find_any(s1_s, s2_s) {
            Some(i) => s1.add(i) as *mut u8,
            None => core::ptr::null_mut(),
        }
    }

}
pub use mstrpbrk::*;

mod mstrrchr {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `char *strrchr(const char *s, int c)`
    /// preserved misfeature: last occurrence; c==0 → ptr to terminating NUL (Some(len)); c truncated to char
    #[no_mangle]
    pub unsafe extern "C" fn strrchr(s: *const u8, c: i32) -> *mut u8 {
        let s_s = cstr_slice(s);
        match cnt_rust::core_rs::string::find_last_char(s_s, c as i32) {
            Some(i) => s.add(i) as *mut u8,
            None => core::ptr::null_mut(),
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
    pub unsafe extern "C" fn strspn(s: *const u8, accept: *const u8) -> usize {
        let s_s = cstr_slice(s);
        let accept_s = cstr_slice(accept);
        cnt_rust::core_rs::string::span_in(s_s, accept_s) as usize
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

    /// `long strtol(const char *nptr, char **endptr, int base)`
    /// preserved misfeature: BSD signed-long clamp: overflow → LONG_MIN/LONG_MAX (32-bit Win long); neg non-overflow = 0u32.wrapping_sub(acc); cutoff = neg?(ULONG)LONG_MAX+1:LONG_MAX; base 0 ⇒ 0x→16/0→8/else 10; no base validation; _LIBCNT_ no errno
    #[no_mangle]
    pub unsafe extern "C" fn strtol(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> i32 {
        let nptr_s = cstr_slice(nptr);
        let r = cnt_rust::core_rs::radix::parse_long_bsd(nptr_s, base);
        if !endptr.is_null() {
            *endptr = nptr.add(r.end) as *mut _;
        }
        r.value as i32
    }

}
pub use mstrtol::*;

mod mstrtoul {
    #[allow(unused_imports)]
    use super::h::{cstr_slice, cstr_slice_bounded, raw_move, raw_mut_slice, raw_slice, wcstr_slice, wcstr_slice_bounded, wraw_mut_slice};

    /// `unsigned long strtoul(const char* nptr, char** end, int base)`
    /// preserved misfeature: delegates to _strtoul_l — same asymmetric clamp incl. neg>ULONG_MAX→1
    #[no_mangle]
    pub unsafe extern "C" fn strtoul(nptr: *const u8, end: *mut *mut u8, base: i32) -> u32 {
        let nptr_s = cstr_slice(nptr);
        let r = cnt_rust::core_rs::radix::parse_i64_clamped(nptr_s, base);
        if r.base_ok && !end.is_null() {
            *end = nptr.add(r.end) as *mut u8;
        }
        let v = if r.value > 4294967295i64 {
            4294967295i64
        } else if r.value < -4294967295i64 {
            1i64
        } else { r.value };
        v as u32 as u32
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
#[no_mangle]
pub unsafe extern "C" fn strcmp(a: *const u8, b: *const u8) -> i32 {
    clamp(cnt_rust::core_rs::string::cmp_u(
        h::cstr_slice(a),
        h::cstr_slice(b),
    ))
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
#[inline]
pub unsafe fn strncmp_raw(a: *const u8, b: *const u8, len: usize) -> i32 {
    if len == 0 {
        return 0;
    }
    cnt_rust::core_rs::string::ncmp_u(
        h::cstr_slice_bounded(a, len),
        h::cstr_slice_bounded(b, len),
        len,
    )
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

// NOTE: the no_std runtime bits (`#[panic_handler]` + `rust_eh_personality`)
// deliberately do NOT live here — they are in the `msvcrt-crt-cargo` staticlib
// umbrella. This crate is a plain rlib so that `msvcrt-difftest` can link it
// and fuzz THESE wrappers (the code that actually ships) against the genuine
// msvcrt C. A `staticlib` crate-type would force a `#[panic_handler]` into the
// same rustc invocation that emits the rlib, and that becomes a duplicate
// `panic_impl` lang item inside the harness's std test binary.
