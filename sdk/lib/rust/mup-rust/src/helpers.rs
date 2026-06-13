//! helpers.rs — the handful of header inlines / CRT shims mup.c uses that are NOT
//! ntoskrnl link exports: CONTAINING_RECORD (offset arithmetic), RtlMoveMemory (memmove),
//! and the wide-char CRT helpers wcschr / _wcsnicmp. Everything else (list ops,
//! interlocked, Io stack walkers, ExInitializeWorkItem, KeQuerySystemTime, FsRtl enter/
//! exit, ProbeForRead, InitializeObjectAttributes) comes from ntkrnl-sys inline modules.
#![allow(non_snake_case, dead_code)]

use ntkrnl_sys::base::{PVOID, SIZE_T};

/// wsz — const ASCII &[u8] → NUL-terminated UTF-16 `[u16; N]`. All of mup's wide string
/// literals are ASCII; N must be (char count + 1) for the trailing NUL. A too-small N is
/// a compile error (const index OOB), so the size is checked at build time.
pub const fn wsz<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}

/// CONTAINING_RECORD(addr, Type, field): recover the base of the containing struct from
/// a pointer to one of its fields. Self-consistent via core::mem::offset_of.
#[macro_export]
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $T:ty, $field:ident) => {
        (($addr as *const u8).sub(core::mem::offset_of!($T, $field)) as *mut $T)
    };
}

/// RtlMoveMemory — memmove (overlap-safe), byte granular. ntoskrnl exports it as an
/// intrinsic alias; provide a faithful inline so we don't depend on the export name.
#[inline(always)]
pub unsafe fn RtlMoveMemory(Destination: PVOID, Source: PVOID, Length: SIZE_T) {
    core::ptr::copy(Source as *const u8, Destination as *mut u8, Length as usize);
}

/// wcschr — find the first occurrence of `c` in the null-terminated wide string `s`.
/// Returns a pointer into `s`, or null. (NUL itself matches, per the CRT contract.)
#[inline]
pub unsafe fn wcschr(s: *const u16, c: u16) -> *mut u16 {
    let mut p = s;
    loop {
        let ch = *p;
        if ch == c {
            return p as *mut u16;
        }
        if ch == 0 {
            return core::ptr::null_mut();
        }
        p = p.add(1);
    }
}

/// wcslen — length (in wide chars) of a null-terminated wide string.
#[inline]
pub unsafe fn wcslen(s: *const u16) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// _wcsnicmp — case-insensitive (ASCII fold) compare of up to `n` wide chars. mup only
/// tests for equality (== 0), but we return the full <0/0/>0 ordering to be faithful.
#[inline]
pub unsafe fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let mut ca = *a.add(i);
        let mut cb = *b.add(i);
        if (b'A' as u16..=b'Z' as u16).contains(&ca) {
            ca += 32;
        }
        if (b'A' as u16..=b'Z' as u16).contains(&cb) {
            cb += 32;
        }
        if ca != cb {
            return ca as i32 - cb as i32;
        }
        if ca == 0 {
            break;
        }
        i += 1;
    }
    0
}
