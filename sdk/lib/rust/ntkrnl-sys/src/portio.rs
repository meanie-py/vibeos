//! HAL port I/O and register (MMIO) access: READ/WRITE_PORT_* and READ/WRITE_REGISTER_*.
//!
//! Lifted from storport-rust's glue.rs, the first of several drivers to carry its own copy
//! (i8042prt and bootvid have others). On i386 and arm64 these are hal.dll exports; on amd64
//! the HAL exports none of them (MSVC treats them as intrinsics), so they are inline asm and
//! volatile accesses there. The new ATA stack (pciidex/atapi/legide) is the first user that
//! imports them from here instead of re-porting them.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use crate::base::*;

// ============================================================================
// PORT I/O and REGISTER (MMIO) access
//
// On i386 and arm64 these are HAL exports — resolved at link time via the
// extern "system" blocks below.
//
// On amd64 (x86_64) the HAL does NOT export them as symbols; they are
// compiler intrinsics in MSVC.  We provide inline-asm / volatile-ptr
// equivalents that are faithful to the DDK semantics.
// ============================================================================

// ---------------------------------------------------------------------------
// PORT I/O — HAL extern (i386 / arm64)
// ---------------------------------------------------------------------------
#[cfg(not(target_arch = "x86_64"))]
extern "system" {
    pub fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR;
    pub fn READ_PORT_USHORT(Port: PUSHORT) -> USHORT;
    pub fn READ_PORT_ULONG(Port: PULONG) -> ULONG;
    pub fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR);
    pub fn WRITE_PORT_USHORT(Port: PUSHORT, Value: USHORT);
    pub fn WRITE_PORT_ULONG(Port: PULONG, Value: ULONG);
    pub fn READ_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG);
    pub fn READ_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG);
    pub fn READ_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG);
    pub fn WRITE_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG);
    pub fn WRITE_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG);
    pub fn WRITE_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG);
}

// ---------------------------------------------------------------------------
// PORT I/O — inline asm (amd64 only)
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") (Port as usize as u16), options(nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_USHORT(Port: PUSHORT) -> USHORT {
    let v: u16;
    core::arch::asm!("in ax, dx", out("ax") v, in("dx") (Port as usize as u16), options(nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_ULONG(Port: PULONG) -> ULONG {
    let v: u32;
    core::arch::asm!("in eax, dx", out("eax") v, in("dx") (Port as usize as u16), options(nostack, preserves_flags));
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR) {
    core::arch::asm!("out dx, al", in("dx") (Port as usize as u16), in("al") Value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_USHORT(Port: PUSHORT, Value: USHORT) {
    core::arch::asm!("out dx, ax", in("dx") (Port as usize as u16), in("ax") Value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_ULONG(Port: PULONG, Value: ULONG) {
    core::arch::asm!("out dx, eax", in("dx") (Port as usize as u16), in("eax") Value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), READ_PORT_UCHAR(Port));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), READ_PORT_USHORT(Port));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), READ_PORT_ULONG(Port));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_BUFFER_UCHAR(Port: PUCHAR, Buffer: PUCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        WRITE_PORT_UCHAR(Port, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_BUFFER_USHORT(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG) {
    for i in 0..Count as isize {
        WRITE_PORT_USHORT(Port, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_PORT_BUFFER_ULONG(Port: PULONG, Buffer: PULONG, Count: ULONG) {
    for i in 0..Count as isize {
        WRITE_PORT_ULONG(Port, core::ptr::read(Buffer.offset(i)));
    }
}

// ---------------------------------------------------------------------------
// REGISTER (MMIO) — HAL extern (i386 / arm64)
// ---------------------------------------------------------------------------
#[cfg(not(target_arch = "x86_64"))]
extern "system" {
    pub fn READ_REGISTER_UCHAR(Register: *mut UCHAR) -> UCHAR;
    pub fn READ_REGISTER_USHORT(Register: *mut USHORT) -> USHORT;
    pub fn READ_REGISTER_ULONG(Register: *mut ULONG) -> ULONG;
    pub fn READ_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG);
    pub fn READ_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG);
    pub fn READ_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG);
    pub fn WRITE_REGISTER_UCHAR(Register: *mut UCHAR, Value: UCHAR);
    pub fn WRITE_REGISTER_USHORT(Register: *mut USHORT, Value: USHORT);
    pub fn WRITE_REGISTER_ULONG(Register: *mut ULONG, Value: ULONG);
    pub fn WRITE_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG);
    pub fn WRITE_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG);
    pub fn WRITE_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG);
}

// ---------------------------------------------------------------------------
// REGISTER (MMIO) — volatile pointer (amd64 only)
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_UCHAR(Register: *mut UCHAR) -> UCHAR {
    core::ptr::read_volatile(Register)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_USHORT(Register: *mut USHORT) -> USHORT {
    core::ptr::read_volatile(Register)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_ULONG(Register: *mut ULONG) -> ULONG {
    core::ptr::read_volatile(Register)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_UCHAR(Register: *mut UCHAR, Value: UCHAR) {
    core::ptr::write_volatile(Register, Value)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_USHORT(Register: *mut USHORT, Value: USHORT) {
    core::ptr::write_volatile(Register, Value)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_ULONG(Register: *mut ULONG, Value: ULONG) {
    core::ptr::write_volatile(Register, Value)
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), core::ptr::read_volatile(Register));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), core::ptr::read_volatile(Register));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn READ_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write(Buffer.offset(i), core::ptr::read_volatile(Register));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_BUFFER_UCHAR(Register: *mut UCHAR, Buffer: *mut UCHAR, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write_volatile(Register, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_BUFFER_USHORT(Register: *mut USHORT, Buffer: *mut USHORT, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write_volatile(Register, core::ptr::read(Buffer.offset(i)));
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn WRITE_REGISTER_BUFFER_ULONG(Register: *mut ULONG, Buffer: *mut ULONG, Count: ULONG) {
    for i in 0..Count as isize {
        core::ptr::write_volatile(Register, core::ptr::read(Buffer.offset(i)));
    }
}
