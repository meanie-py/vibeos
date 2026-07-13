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
/// what proved it (field manual §9.63).
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
