//! Cross-platform atomic primitives for SList operations.
//!
//! Provides a unified interface across three architectures:
//! - **aarch64**: LDAXP/STLXP (128-bit exclusive monitor)
//! - **x86_64**: volatile reads + LOCK CMPXCHG16B (128-bit CAS)
//! - **x86**: volatile reads + LOCK CMPXCHG8B (64-bit CAS)

use core::arch::asm;

/// Atomically load the SList header.
///
/// Returns `(alignment, region)` — the two halves of the header.
/// On 32-bit, `region` is always 0 (header is only 8 bytes).
///
/// # Safety
/// `addr` must point to a valid, properly aligned SLIST_HEADER.
#[inline(always)]
pub unsafe fn load(addr: *const u8) -> (u64, u64) {
    #[cfg(target_arch = "aarch64")]
    {
        let lo: u64;
        let hi: u64;
        unsafe {
            asm!(
                "ldaxp {lo}, {hi}, [{addr}]",
                addr = in(reg) addr,
                lo = out(reg) lo,
                hi = out(reg) hi,
                options(nostack, preserves_flags),
            );
        }
        (lo, hi)
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Volatile reads — the atomicity is provided by CMPXCHG16B in try_store.
        // We read Region first, then Alignment, matching the original asm order.
        let region = unsafe { core::ptr::read_volatile((addr as *const u64).add(1)) };
        let alignment = unsafe { core::ptr::read_volatile(addr as *const u64) };
        (alignment, region)
    }

    #[cfg(target_arch = "x86")]
    {
        // 32-bit: header is 8 bytes (two u32s). Read as two volatile u32 reads.
        let lo = unsafe { core::ptr::read_volatile(addr as *const u32) };
        let hi = unsafe { core::ptr::read_volatile((addr as *const u32).add(1)) };
        // Pack into alignment as: Next[31:0] | Depth+Sequence[63:32]
        let alignment = (lo as u64) | ((hi as u64) << 32);
        (alignment, 0)
    }
}

/// Attempt to atomically store new header values.
///
/// On aarch64: Uses STLXP (ignores `old` — relies on exclusive monitor from `load`).
/// On x86_64: Uses LOCK CMPXCHG16B (compares against `old`).
/// On x86: Uses LOCK CMPXCHG8B (compares against `old`, only uses first element).
///
/// Returns `true` if the store succeeded.
///
/// # Safety
/// `addr` must point to a valid, properly aligned SLIST_HEADER.
/// Must be preceded by a `load` on the same address.
#[inline(always)]
pub unsafe fn try_store(
    addr: *mut u8,
    old: (u64, u64),
    new: (u64, u64),
) -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        let _ = old; // ARM64 uses exclusive monitor, not CAS
        let status: u32;
        unsafe {
            asm!(
                "stlxp {status:w}, {lo}, {hi}, [{addr}]",
                addr = in(reg) addr,
                lo = in(reg) new.0,
                hi = in(reg) new.1,
                status = out(reg) status,
                options(nostack, preserves_flags),
            );
        }
        status == 0
    }

    #[cfg(target_arch = "x86_64")]
    {
        // LOCK CMPXCHG16B: compare RDX:RAX with [addr], if equal store RCX:RBX.
        // LLVM reserves rbx, so we use xchg to swap it in/out.
        let success: u8;
        unsafe {
            asm!(
                "xchg {new_lo}, rbx",
                "lock cmpxchg16b [{addr}]",
                "xchg {new_lo}, rbx",
                "setz {success}",
                addr = in(reg) addr,
                new_lo = inout(reg) new.0 => _,
                inout("rax") old.0 => _,
                inout("rdx") old.1 => _,
                in("rcx") new.1,
                success = out(reg_byte) success,
                options(nostack),
            );
        }
        success != 0
    }

    #[cfg(target_arch = "x86")]
    {
        // LOCK CMPXCHG8B: compare EDX:EAX with [addr], if equal store ECX:EBX.
        // ebx may be reserved (PIC), so we use xchg to swap it in/out.
        // xchg does not affect flags, so sete reads the ZF from cmpxchg8b.
        //
        // `addr` is PINNED to edi. On x86 (unlike x86_64's rbx) ebx is an ordinary
        // allocatable `reg`, and with `addr = in(reg)` LLVM put the HEADER pointer in ebx:
        // the first xchg then replaced it with new_lo -- the entry being pushed -- and
        // cmpxchg8b compared against the ENTRY. Push had just written entry->Next = old
        // head, so it "succeeded" into the entry (list unchanged) when entry+4 happened to
        // equal the header's high dword, and spun forever otherwise; flush CAS'd at
        // address 0. That shipped in ntdll.dll and ks.sys (ucrtbase's
        // __std_type_info_name hung in it). With addr out of ebx's reach, new_lo is the
        // only operand that can land there, and then the xchg is a harmless no-op.
        let old_lo = old.0 as u32;
        let old_hi = (old.0 >> 32) as u32;
        let new_lo = new.0 as u32;
        let new_hi = (new.0 >> 32) as u32;
        let zf: u32;
        unsafe {
            asm!(
                "xchg {new_lo}, ebx",
                "lock cmpxchg8b [edi]",
                "xchg {new_lo}, ebx",
                "sete al",
                "movzx eax, al",
                in("edi") addr,
                new_lo = inout(reg) new_lo => _,
                inout("eax") old_lo => zf,
                inout("edx") old_hi => _,
                in("ecx") new_hi,
                options(nostack),
            );
        }
        zf != 0
    }
}

/// Cancel an exclusive load without storing.
///
/// On aarch64: CLREX to release the exclusive monitor.
/// On x86/x86_64: No-op (no exclusive monitor to release).
#[inline(always)]
#[allow(dead_code)]
pub unsafe fn cancel() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        asm!("clrex", options(nostack, preserves_flags));
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        // No-op: x86 CAS doesn't need cancellation
    }
}
