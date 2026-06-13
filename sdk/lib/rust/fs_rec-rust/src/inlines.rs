//! inlines.rs — FORCEINLINE IRP-stack walkers the LLM rendered as calls. These are
//! ntddk.h header inlines (Irp->Tail.Overlay.CurrentStackLocation arithmetic), NOT
//! ntoskrnl link exports — the bindgen `ddk` versions are phantom externs that fail at
//! link. Re-export the real inline impls from ntkrnl_sys::inline::tier1.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation};
use ntkrnl_sys::ddk::LARGE_INTEGER;

// RtlExtendedLargeIntegerDivide — ntoskrnl exports it on i386 only (64/32 division
// helper); on amd64/arm64 it is a header inline over native division. Provide a faithful
// inline so the symbol resolves on every arch. Dividend is an unsigned sector/byte count.
#[inline(always)]
pub unsafe fn RtlExtendedLargeIntegerDivide(
    Dividend: LARGE_INTEGER,
    Divisor: u32,
    Remainder: *mut u32,
) -> LARGE_INTEGER {
    let dv = Dividend.QuadPart as u64;
    let q = dv / Divisor as u64;
    let r = dv % Divisor as u64;
    if !Remainder.is_null() {
        *Remainder = r as u32;
    }
    let mut out: LARGE_INTEGER = core::mem::zeroed();
    out.QuadPart = q as i64;
    out
}

// fs_rec.h non-standard rounding macros: ROUND_DOWN(n,align)=n & ~(align-1);
// ROUND_UP(n,align)=ROUND_DOWN(n+align-1, align). Both operate on ULONG (u32).
#[inline(always)]
pub fn ROUND_DOWN(n: u32, align: u32) -> u32 {
    n & !(align - 1)
}
#[inline(always)]
pub fn ROUND_UP(n: u32, align: u32) -> u32 {
    ROUND_DOWN(n.wrapping_add(align - 1), align)
}

// strncmp — C lib byte compare (udfs.rs sniffs fixed-length UDF volume-descriptor
// identifiers). Not a kernel export; implement faithfully. Returns 0 on equal prefix.
#[inline(always)]
pub unsafe fn strncmp(a: *const i8, b: *const i8, n: u64) -> i32 {
    let mut i: usize = 0;
    while i < n as usize {
        let ca = *a.add(i) as u8;
        let cb = *b.add(i) as u8;
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
