//! The bulk loops under `memset`, `memcpy` and `memmove` (`raw_fill`, `raw_move`), in inline
//! assembly. Hand-written; `tools/gen_cnt.py`'s template declares the module.
//!
//! Two reasons this is asm and not Rust:
//!   * cnt-rust is built with `-C no-prepopulate-passes` (libcntpr.cmake, a miscompile guard),
//!     so Rust here gets NO LLVM optimisation: the old byte loops cost ~10 instructions a byte,
//!     and every RtlZeroMemory / RtlCopyMemory in the kernel and ntdll paid it. wowx64's
//!     1088-byte stack zeroing was 45 us of every x86-64 guest syscall under TCG, eight times
//!     the syscall itself.
//!   * asm is opaque, so LLVM can neither delete the stores (the dead-store elimination that once
//!     turned memset into a spin, see gen_cnt.py) nor fold the loop back into a call to the
//!     function it implements.
//!
//! arm64 touches memory only with ACCESSES ALIGNED TO THEIR SIZE: these run on device memory
//! too (MMIO, a framebuffer), where an unaligned access faults whatever SCTLR.A says. So the
//! wide path needs the destination and source to agree modulo 8; otherwise it is bytes. x86
//! has no such rule, and `rep movsb` / `rep stosb` are the fast path on every CPU with ERMSB.
//! The direction flag is clear at every x86 call boundary, and the backward copy restores it.
//!
//! Every function is top level, one cfg-guarded copy per arch, and calls only others in this
//! file by bare name: the memmove difftest contract (tools/rustconv/data/difftest/memmove.json)
//! lifts them into its harness item by item, which cannot follow a module path or a macro.

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_fill(mut d: *mut u8, v: u8, mut n: usize) {
    while n != 0 && (d as usize) & 15 != 0 {
        core::arch::asm!("strb {v:w}, [{d}], #1", d = inout(reg) d, v = in(reg) v as u32, options(nostack, preserves_flags));
        n -= 1;
    }
    let blocks = n / 16;
    if blocks != 0 {
        let pat = (v as u64).wrapping_mul(0x0101_0101_0101_0101);
        core::arch::asm!(
            "2:",
            "stp {p}, {p}, [{d}], #16",
            "subs {c}, {c}, #1",
            "b.ne 2b",
            d = inout(reg) d,
            c = inout(reg) blocks => _,
            p = in(reg) pat,
            options(nostack),
        );
        n %= 16;
    }
    while n != 0 {
        core::arch::asm!("strb {v:w}, [{d}], #1", d = inout(reg) d, v = in(reg) v as u32, options(nostack, preserves_flags));
        n -= 1;
    }
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_bytes_forward(d: *mut u8, s: *const u8, n: usize) {
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{s}], #1",
            "strb {t:w}, [{d}], #1",
            "subs {c}, {c}, #1",
            "b.ne 2b",
            d = inout(reg) d => _,
            s = inout(reg) s => _,
            c = inout(reg) n => _,
            t = out(reg) _,
            options(nostack),
        );
    }
}

/// Copies the `n` bytes that END just below `d` and `s` (both one past the end).
#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_bytes_backward(d: *mut u8, s: *const u8, n: usize) {
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{s}, #-1]!",
            "strb {t:w}, [{d}, #-1]!",
            "subs {c}, {c}, #1",
            "b.ne 2b",
            d = inout(reg) d => _,
            s = inout(reg) s => _,
            c = inout(reg) n => _,
            t = out(reg) _,
            options(nostack),
        );
    }
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_copy_forward(d: *mut u8, s: *const u8, n: usize) {
    if (d as usize ^ s as usize) & 7 != 0 || n < 32 {
        return mem_bytes_forward(d, s, n);
    }
    let head = (8 - (d as usize & 7)) & 7;
    mem_bytes_forward(d, s, head);
    let (d, s, n) = (d.add(head), s.add(head), n - head);
    // n >= 25 here, so at least one pair.
    let pairs = n / 16;
    core::arch::asm!(
        "2:",
        "ldp {a}, {b}, [{s}], #16",
        "stp {a}, {b}, [{d}], #16",
        "subs {c}, {c}, #1",
        "b.ne 2b",
        d = inout(reg) d => _,
        s = inout(reg) s => _,
        c = inout(reg) pairs => _,
        a = out(reg) _,
        b = out(reg) _,
        options(nostack),
    );
    let done = pairs * 16;
    mem_bytes_forward(d.add(done), s.add(done), n - done);
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_copy_backward(d: *mut u8, s: *const u8, n: usize) {
    let (de, se) = (d.add(n), s.add(n));
    if (d as usize ^ s as usize) & 7 != 0 || n < 32 {
        return mem_bytes_backward(de, se, n);
    }
    let tail = de as usize & 7;
    mem_bytes_backward(de, se, tail);
    let (de, se, n) = (de.sub(tail), se.sub(tail), n - tail);
    let pairs = n / 16;
    core::arch::asm!(
        "2:",
        "ldp {a}, {b}, [{s}, #-16]!",
        "stp {a}, {b}, [{d}, #-16]!",
        "subs {c}, {c}, #1",
        "b.ne 2b",
        d = inout(reg) de => _,
        s = inout(reg) se => _,
        c = inout(reg) pairs => _,
        a = out(reg) _,
        b = out(reg) _,
        options(nostack),
    );
    let done = pairs * 16;
    mem_bytes_backward(de.sub(done), se.sub(done), n - done);
}

#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn mem_fill(d: *mut u8, v: u8, n: usize) {
    core::arch::asm!("rep stosb", inout("rdi") d => _, inout("rcx") n => _, in("al") v, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn mem_copy_forward(d: *mut u8, s: *const u8, n: usize) {
    core::arch::asm!("rep movsb", inout("rdi") d => _, inout("rsi") s => _, inout("rcx") n => _, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn mem_copy_backward(d: *mut u8, s: *const u8, n: usize) {
    if n != 0 {
        core::arch::asm!(
            "std",
            "rep movsb",
            "cld",
            inout("rdi") d.add(n - 1) => _,
            inout("rsi") s.add(n - 1) => _,
            inout("rcx") n => _,
            options(nostack),
        );
    }
}

#[cfg(target_arch = "x86")]
pub(crate) unsafe fn mem_fill(d: *mut u8, v: u8, n: usize) {
    core::arch::asm!("rep stosb", inout("edi") d => _, inout("ecx") n => _, in("al") v, options(nostack, preserves_flags));
}

// i386 LLVM keeps esi for itself (its base pointer), so it cannot be an operand: the source
// comes in any register and is swapped into esi around the string op.
#[cfg(target_arch = "x86")]
pub(crate) unsafe fn mem_copy_forward(d: *mut u8, s: *const u8, n: usize) {
    core::arch::asm!(
        "xchg {s}, esi",
        "rep movsb",
        "mov esi, {s}",
        s = inout(reg) s => _,
        inout("edi") d => _,
        inout("ecx") n => _,
        options(nostack, preserves_flags),
    );
}

#[cfg(target_arch = "x86")]
pub(crate) unsafe fn mem_copy_backward(d: *mut u8, s: *const u8, n: usize) {
    if n != 0 {
        core::arch::asm!(
            "xchg {s}, esi",
            "std",
            "rep movsb",
            "cld",
            "mov esi, {s}",
            s = inout(reg) s.add(n - 1) => _,
            inout("edi") d.add(n - 1) => _,
            inout("ecx") n => _,
            options(nostack),
        );
    }
}
