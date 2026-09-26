//! crt-portable-rust — CRT bodies with one contract across msvcrt.dll and
//! ucrtbase.dll. See Cargo.toml for the two conditions a body must meet to live
//! here, and why the `#[no_mangle]` definitions (not shims) are what moved.
//!
//! The helpers are unconditional: `msvcrt-crt-rust` uses them for families that
//! have NOT moved (strcmp, the wide twins, ...). The bodies are feature-gated so
//! each consumer takes over exactly the families it declares.
#![no_std]
// LOAD-BEARING, for the same reason as in msvcrt-crt-rust: these loops are the
// idioms LLVM rewrites INTO calls to the very functions defined here.
#![no_builtins]
#![allow(clippy::missing_safety_doc)]

pub mod h {
    #[inline]
    pub unsafe fn cstr_slice<'a>(p: *const u8) -> &'a [u8] {
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
    pub unsafe fn cstr_slice_bounded<'a>(p: *const u8, cap: usize) -> &'a [u8] {
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
    pub unsafe fn raw_mut_slice<'a>(p: *mut u8, n: usize) -> &'a mut [u8] {
        if p.is_null() || n == 0 {
            return &mut [];
        }
        core::slice::from_raw_parts_mut(p, n)
    }
}

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


#[inline]
pub fn nz(p: *const u8) -> *const u8 {
    if p.is_null() {
        EMPTY_A.as_ptr()
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
pub struct ByteSet([u64; 4]);

impl ByteSet {
    #[inline]
    pub unsafe fn of(mut p: *const u8) -> Self {
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
    pub fn has(&self, b: u8) -> bool {
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
pub unsafe fn cstr_copy(mut dst: *mut u8, mut src: *const u8) {
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

#[cfg(feature = "string")]
mod string {
mod mstrcat {
    #[allow(unused_imports)]
    use crate::h::*;

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

mod mstrcpy {
    #[allow(unused_imports)]
    use crate::h::*;

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
    use crate::h::*;

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
    use crate::h::*;

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
    use crate::h::*;

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
    use crate::h::*;

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
    use crate::h::*;

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
    use crate::h::*;

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

mod mstrspn {
    #[allow(unused_imports)]
    use crate::h::*;

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

}
#[cfg(feature = "string")]
pub use string::*;
