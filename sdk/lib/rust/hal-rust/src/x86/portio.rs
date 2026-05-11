//! Port-I/O wrappers exposed by the kernel as `READ_PORT_UCHAR` /
//! `WRITE_PORT_UCHAR` and friends. NTAPI exports — kernel + driver
//! callers expect these as ordinary functions, not intrinsics.
//!
//! 12 functions: 6 read + 6 write, scalar (UCHAR/USHORT/ULONG) +
//! buffer (REP INS/OUTS) variants. All translate to a single x86
//! `in`/`out` or `rep ins`/`rep outs` instruction.
//!
//! Translated from `hal/halx86/generic/portio.c`. **i386 only** —
//! amd64 has these as inline intrinsics in the kernel headers, no
//! HAL export needed (gated `if(ARCH STREQUAL "i386")` in
//! generic.cmake).

#![cfg(target_arch = "x86")]

use ntkrnl_sys::hal_ddk::{PUCHAR, PULONG, PUSHORT, UCHAR, ULONG, USHORT};

// ── Scalar reads ────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR {
    let v: UCHAR;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") Port as u16, out("al") v,
            options(nostack, preserves_flags),
        );
    }
    v
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_USHORT(Port: PUSHORT) -> USHORT {
    let v: USHORT;
    unsafe {
        core::arch::asm!(
            "in ax, dx",
            in("dx") Port as u16, out("ax") v,
            options(nostack, preserves_flags),
        );
    }
    v
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_ULONG(Port: PULONG) -> ULONG {
    let v: ULONG;
    unsafe {
        core::arch::asm!(
            "in eax, dx",
            in("dx") Port as u16, out("eax") v,
            options(nostack, preserves_flags),
        );
    }
    v
}

// ── Scalar writes ───────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") Port as u16, in("al") Value,
            options(nostack, preserves_flags),
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_USHORT(Port: PUSHORT, Value: USHORT) {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") Port as u16, in("ax") Value,
            options(nostack, preserves_flags),
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_ULONG(Port: PULONG, Value: ULONG) {
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") Port as u16, in("eax") Value,
            options(nostack, preserves_flags),
        );
    }
}

// ── Buffer reads (REP INS) ──────────────────────────────────────────────────
//
// REP INSB/INSW/INSD reads ECX units from port DX into [EDI]. After
// the rep: EDI advances by Count*sizeof(unit), ECX is 0. DF must be
// clear (forward); we use `cld` to ensure that.
//
// EDI is reserved by LLVM on i386 (same class of constraint as RBX
// on amd64 for CMPXCHG16B). Standard workaround: xchg the buffer
// pointer into EDI, do the rep, xchg it back. Caller doesn't care
// about the post-rep EDI value, so the second xchg's effect on the
// scratch register is "discarded" via `=> _`.

macro_rules! rep_in {
    ($insn:literal, $port:expr, $buf:expr, $count:expr) => {
        unsafe {
            core::arch::asm!(
                "xchg edi, {buf}",
                concat!("cld; rep ", $insn, "; xchg edi, {buf}"),
                buf = inout(reg) $buf => _,
                in("dx") $port as u16,
                inout("ecx") $count => _,
                options(nostack, preserves_flags),
            );
        }
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_BUFFER_UCHAR(
    Port: PUCHAR,
    Buffer: PUCHAR,
    Count: ULONG,
) {
    rep_in!("insb", Port, Buffer, Count);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_BUFFER_USHORT(
    Port: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    rep_in!("insw", Port, Buffer, Count);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_BUFFER_ULONG(
    Port: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    rep_in!("insd", Port, Buffer, Count);
}

// ── Buffer writes (REP OUTS) ────────────────────────────────────────────────
//
// REP OUTSB/OUTSW/OUTSD writes ECX units from [ESI] to port DX. ESI
// is reserved by LLVM on i386, same xchg-trick as EDI above.

macro_rules! rep_out {
    ($insn:literal, $port:expr, $buf:expr, $count:expr) => {
        unsafe {
            core::arch::asm!(
                "xchg esi, {buf}",
                concat!("cld; rep ", $insn, "; xchg esi, {buf}"),
                buf = inout(reg) $buf => _,
                in("dx") $port as u16,
                inout("ecx") $count => _,
                options(nostack, preserves_flags),
            );
        }
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_BUFFER_UCHAR(
    Port: PUCHAR,
    Buffer: PUCHAR,
    Count: ULONG,
) {
    rep_out!("outsb", Port, Buffer, Count);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_BUFFER_USHORT(
    Port: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    rep_out!("outsw", Port, Buffer, Count);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_BUFFER_ULONG(
    Port: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    rep_out!("outsd", Port, Buffer, Count);
}
