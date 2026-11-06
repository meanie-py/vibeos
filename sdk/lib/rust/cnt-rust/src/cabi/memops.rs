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

// ---------------------------------------------------------------------------------------------
// NUL scans (`cstr_slice` / `wcstr_slice`, so strlen, wcslen and every string function that
// measures its argument first). The count is in ELEMENTS. x86 uses `repne scas`; arm64 reads
// aligned 8-byte words, which never cross a page, and tests each for a zero element with the
// (w - 0x01..) & !w & 0x80.. trick, then finds which element byte by byte. A wide string at an
// odd address never reaches 8-alignment and is scanned a halfword at a time throughout.
// ---------------------------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_len8(p: *const u8) -> usize {
    let mut q = p;
    core::arch::asm!(
        "2:",
        "tst {q}, #7",
        "b.eq 3f",
        "ldrb {t:w}, [{q}]",
        "cbz {t:w}, 6f",
        "add {q}, {q}, #1",
        "b 2b",
        "3:",
        "ldr {t}, [{q}]",
        "sub {u}, {t}, {ones}",
        "bic {u}, {u}, {t}",
        "tst {u}, {highs}",
        "b.ne 4f",
        "add {q}, {q}, #8",
        "b 3b",
        "4:",
        "ldrb {t:w}, [{q}]",
        "cbz {t:w}, 6f",
        "add {q}, {q}, #1",
        "b 4b",
        "6:",
        q = inout(reg) q,
        t = out(reg) _,
        u = out(reg) _,
        ones = in(reg) 0x0101_0101_0101_0101u64,
        highs = in(reg) 0x8080_8080_8080_8080u64,
        options(nostack, readonly),
    );
    q as usize - p as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_len16(p: *const u16) -> usize {
    let mut q = p;
    core::arch::asm!(
        "2:",
        "tst {q}, #7",
        "b.eq 3f",
        "ldrh {t:w}, [{q}]",
        "cbz {t:w}, 6f",
        "add {q}, {q}, #2",
        "b 2b",
        "3:",
        "ldr {t}, [{q}]",
        "sub {u}, {t}, {ones}",
        "bic {u}, {u}, {t}",
        "tst {u}, {highs}",
        "b.ne 4f",
        "add {q}, {q}, #8",
        "b 3b",
        "4:",
        "ldrh {t:w}, [{q}]",
        "cbz {t:w}, 6f",
        "add {q}, {q}, #2",
        "b 4b",
        "6:",
        q = inout(reg) q,
        t = out(reg) _,
        u = out(reg) _,
        ones = in(reg) 0x0001_0001_0001_0001u64,
        highs = in(reg) 0x8000_8000_8000_8000u64,
        options(nostack, readonly),
    );
    (q as usize - p as usize) / 2
}

// rcx counts down from -1 over every element read, the NUL included: !rcx - 1 is the length.
#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn str_len8(p: *const u8) -> usize {
    let c: usize;
    core::arch::asm!("repne scasb", inout("rdi") p => _, inout("rcx") usize::MAX => c, in("al") 0u8, options(nostack, readonly));
    !c - 1
}

#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn str_len16(p: *const u16) -> usize {
    let c: usize;
    core::arch::asm!("repne scasw", inout("rdi") p => _, inout("rcx") usize::MAX => c, in("ax") 0u16, options(nostack, readonly));
    !c - 1
}

#[cfg(target_arch = "x86")]
pub(crate) unsafe fn str_len8(p: *const u8) -> usize {
    let c: usize;
    core::arch::asm!("repne scasb", inout("edi") p => _, inout("ecx") usize::MAX => c, in("al") 0u8, options(nostack, readonly));
    !c - 1
}

#[cfg(target_arch = "x86")]
pub(crate) unsafe fn str_len16(p: *const u16) -> usize {
    let c: usize;
    core::arch::asm!("repne scasw", inout("edi") p => _, inout("ecx") usize::MAX => c, in("ax") 0u16, options(nostack, readonly));
    !c - 1
}

// ---------------------------------------------------------------------------------------------
// Compare and search scans (strcmp, strncmp, _stricmp, _strnicmp, strchr and their wide twins).
// The `core_rs::string` bodies are the contract, but compiled here (no LLVM passes) a slice loop
// cost 7.5 ns an element against 0.25 for the C: 10x at 8 characters, 30x at 256
// (`cnt-difftest --test perf`). So the scan is asm and returns WHERE it stopped, and the result
// is worked out once, in Rust, from the two elements there.
//
// `*_mismatch*(a, b, n)` returns the first index below `n` at which the elements differ or are
// both NUL, or `n`. The `_ci` forms compare after folding ASCII A..Z to a..z, and nothing else
// (the libcntpr NT `_stricmp_nt.c` / `_wcsicmp_nt.c` rule, wide included). `*_chr*(s, c)` returns
// the index of the first element equal to `c` or NUL. Every access is one element at its own
// alignment, so these never read past the terminator or the count.
// ---------------------------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_mismatch8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{p}]",
            "ldrb {u:w}, [{q}]",
            "cmp {t:w}, {u:w}",
            "b.ne 9f",
            "cbz {t:w}, 9f",
            "add {p}, {p}, #1",
            "add {q}, {q}, #1",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_mismatch16(a: *const u16, b: *const u16, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrh {t:w}, [{p}]",
            "ldrh {u:w}, [{q}]",
            "cmp {t:w}, {u:w}",
            "b.ne 9f",
            "cbz {t:w}, 9f",
            "add {p}, {p}, #2",
            "add {q}, {q}, #2",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    (p as usize - a as usize) / 2
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_mismatch_ci8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{p}]",
            "ldrb {u:w}, [{q}]",
            "sub {v:w}, {t:w}, #65",
            "cmp {v:w}, #25",
            "add {v:w}, {t:w}, #32",
            "csel {t:w}, {v:w}, {t:w}, ls",
            "sub {v:w}, {u:w}, #65",
            "cmp {v:w}, #25",
            "add {v:w}, {u:w}, #32",
            "csel {u:w}, {v:w}, {u:w}, ls",
            "cmp {t:w}, {u:w}",
            "b.ne 9f",
            "cbz {t:w}, 9f",
            "add {p}, {p}, #1",
            "add {q}, {q}, #1",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            v = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_mismatch_ci16(a: *const u16, b: *const u16, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrh {t:w}, [{p}]",
            "ldrh {u:w}, [{q}]",
            "sub {v:w}, {t:w}, #65",
            "cmp {v:w}, #25",
            "add {v:w}, {t:w}, #32",
            "csel {t:w}, {v:w}, {t:w}, ls",
            "sub {v:w}, {u:w}, #65",
            "cmp {v:w}, #25",
            "add {v:w}, {u:w}, #32",
            "csel {u:w}, {v:w}, {u:w}, ls",
            "cmp {t:w}, {u:w}",
            "b.ne 9f",
            "cbz {t:w}, 9f",
            "add {p}, {p}, #2",
            "add {q}, {q}, #2",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            v = out(reg) _,
            options(nostack, readonly),
        );
    }
    (p as usize - a as usize) / 2
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_chr8(s: *const u8, c: u8) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "ldrb {t:w}, [{p}]",
        "cmp {t:w}, {c:w}",
        "b.eq 9f",
        "cbz {t:w}, 9f",
        "add {p}, {p}, #1",
        "b 2b",
        "9:",
        p = inout(reg) p,
        c = in(reg) c as u32,
        t = out(reg) _,
        options(nostack, readonly),
    );
    p as usize - s as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_chr16(s: *const u16, c: u16) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "ldrh {t:w}, [{p}]",
        "cmp {t:w}, {c:w}",
        "b.eq 9f",
        "cbz {t:w}, 9f",
        "add {p}, {p}, #2",
        "b 2b",
        "9:",
        p = inout(reg) p,
        c = in(reg) c as u32,
        t = out(reg) _,
        options(nostack, readonly),
    );
    (p as usize - s as usize) / 2
}

// x86 and x86-64 share text: `{r}` is the pointer-width register on each, and five operands fit
// i386's free set (eax, ebx, ecx, edx, edi; LLVM keeps esi and ebp). The case fold runs only
// once the raw elements differ.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_mismatch8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, byte ptr [{p}]",
            "movzx {u:e}, byte ptr [{q}]",
            "cmp {t:e}, {u:e}",
            "jne 9f",
            "test {t:e}, {t:e}",
            "jz 9f",
            "add {p}, 1",
            "add {q}, 1",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_mismatch16(a: *const u16, b: *const u16, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, word ptr [{p}]",
            "movzx {u:e}, word ptr [{q}]",
            "cmp {t:e}, {u:e}",
            "jne 9f",
            "test {t:e}, {t:e}",
            "jz 9f",
            "add {p}, 2",
            "add {q}, 2",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    (p as usize - a as usize) / 2
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_mismatch_ci8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, byte ptr [{p}]",
            "movzx {u:e}, byte ptr [{q}]",
            "cmp {t:e}, {u:e}",
            "je 6f",
            "cmp {t:e}, 65",
            "jb 3f",
            "cmp {t:e}, 90",
            "ja 3f",
            "add {t:e}, 32",
            "3:",
            "cmp {u:e}, 65",
            "jb 4f",
            "cmp {u:e}, 90",
            "ja 4f",
            "add {u:e}, 32",
            "4:",
            "cmp {t:e}, {u:e}",
            "jne 9f",
            "6:",
            "test {t:e}, {t:e}",
            "jz 9f",
            "add {p}, 1",
            "add {q}, 1",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_mismatch_ci16(a: *const u16, b: *const u16, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, word ptr [{p}]",
            "movzx {u:e}, word ptr [{q}]",
            "cmp {t:e}, {u:e}",
            "je 6f",
            "cmp {t:e}, 65",
            "jb 3f",
            "cmp {t:e}, 90",
            "ja 3f",
            "add {t:e}, 32",
            "3:",
            "cmp {u:e}, 65",
            "jb 4f",
            "cmp {u:e}, 90",
            "ja 4f",
            "add {u:e}, 32",
            "4:",
            "cmp {t:e}, {u:e}",
            "jne 9f",
            "6:",
            "test {t:e}, {t:e}",
            "jz 9f",
            "add {p}, 2",
            "add {q}, 2",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    (p as usize - a as usize) / 2
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_chr8(s: *const u8, c: u8) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "movzx {t:e}, byte ptr [{p}]",
        "cmp {t:e}, {c:e}",
        "je 9f",
        "test {t:e}, {t:e}",
        "jz 9f",
        "add {p}, 1",
        "jmp 2b",
        "9:",
        p = inout(reg) p,
        c = in(reg) c as u32 as usize,
        t = out(reg) _,
        options(nostack, readonly),
    );
    p as usize - s as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_chr16(s: *const u16, c: u16) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "movzx {t:e}, word ptr [{p}]",
        "cmp {t:e}, {c:e}",
        "je 9f",
        "test {t:e}, {t:e}",
        "jz 9f",
        "add {p}, 2",
        "jmp 2b",
        "9:",
        p = inout(reg) p,
        c = in(reg) c as u32 as usize,
        t = out(reg) _,
        options(nostack, readonly),
    );
    (p as usize - s as usize) / 2
}

// The shims' bodies (manifest `fast_path`). NULL reads as the empty string, as `cstr_slice` has
// it; the result is the `core_rs::string` / `wstring` body's, worked out at the stopping index.

const NUL8: &u8 = &0;
const NUL16: &u16 = &0;

fn fold(c: i32) -> i32 {
    if (65..=90).contains(&c) {
        c + 32
    } else {
        c
    }
}

pub(crate) unsafe fn fast_strcmp(a: *const u8, b: *const u8) -> i32 {
    fast_strncmp(a, b, usize::MAX)
}

pub(crate) unsafe fn fast_strncmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let a = if a.is_null() { NUL8 as *const u8 } else { a };
    let b = if b.is_null() { NUL8 as *const u8 } else { b };
    let i = str_mismatch8(a, b, n);
    if i == n {
        return 0;
    }
    (*a.add(i) as i8 as i32) - (*b.add(i) as i8 as i32)
}

pub(crate) unsafe fn fast_wcscmp(a: *const u16, b: *const u16) -> i32 {
    fast_wcsncmp(a, b, usize::MAX)
}

pub(crate) unsafe fn fast_wcsncmp(a: *const u16, b: *const u16, n: usize) -> i32 {
    let a = if a.is_null() { NUL16 as *const u16 } else { a };
    let b = if b.is_null() { NUL16 as *const u16 } else { b };
    let i = str_mismatch16(a, b, n);
    if i == n {
        return 0;
    }
    (*a.add(i) as i32) - (*b.add(i) as i32)
}

pub(crate) unsafe fn fast_stricmp(a: *const u8, b: *const u8) -> i32 {
    fast_strnicmp(a, b, usize::MAX)
}

pub(crate) unsafe fn fast_strnicmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let a = if a.is_null() { NUL8 as *const u8 } else { a };
    let b = if b.is_null() { NUL8 as *const u8 } else { b };
    let i = str_mismatch_ci8(a, b, n);
    if i == n {
        return 0;
    }
    // A fold moves only A..Z, which are positive as signed char, so folding the signed value
    // is the C's `chr1 - chr2` after its own fold.
    fold(*a.add(i) as i8 as i32) - fold(*b.add(i) as i8 as i32)
}

pub(crate) unsafe fn fast_wcsicmp(a: *const u16, b: *const u16) -> i32 {
    fast_wcsnicmp(a, b, usize::MAX)
}

pub(crate) unsafe fn fast_wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32 {
    let a = if a.is_null() { NUL16 as *const u16 } else { a };
    let b = if b.is_null() { NUL16 as *const u16 } else { b };
    let i = str_mismatch_ci16(a, b, n);
    if i == n {
        return 0;
    }
    fold(*a.add(i) as i32) - fold(*b.add(i) as i32)
}

/// `c` is cut to `char` first (tcschr.h's `_TCHAR cc = c`); c == 0 finds the terminator. NULL
/// searches nothing and returns NULL either way (the slice shim's `s.add(0)`).
pub(crate) unsafe fn fast_strchr(s: *const u8, c: i32) -> *mut u8 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    let i = str_chr8(s, c as u8);
    if *s.add(i) == c as u8 {
        s.add(i) as *mut u8
    } else {
        core::ptr::null_mut()
    }
}

pub(crate) unsafe fn fast_wcschr(s: *const u16, c: u16) -> *mut u16 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    let i = str_chr16(s, c);
    if *s.add(i) == c {
        s.add(i) as *mut u16
    } else {
        core::ptr::null_mut()
    }
}

// ---------------------------------------------------------------------------------------------
// The rest of the search family (strrchr, strstr, strspn, strcspn, strpbrk, memchr and the wide
// twins; strnlen/wcsnlen through the bounded-slice helpers). Same reason as above: the slice
// bodies ran 5-86x the C (`cnt-difftest --test perf`, wcsstr at 256 the worst).
//
// `mem_chr*(p, c, n)` returns the first index below `n` whose element is `c`, or `n`.
// `str_rchr*(s, c)` returns a pointer to the last `c` up to and including the terminator, or
// NULL, so `c == 0` lands on the terminator itself, as strrchr wants. The span pair works on a 256-byte map
// that `span_map*` fills with 1 for every member of a NUL-terminated set below 256 and 0
// elsewhere; `span_scan*(s, map, go)` returns the first index whose map byte is not `go`, and
// the wide scan also stops at any element of 256 or more, which the map cannot answer.
// ---------------------------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_chr8(s: *const u8, c: u8, n: usize) -> usize {
    let mut p = s;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{p}]",
            "cmp {t:w}, {c:w}",
            "b.eq 9f",
            "add {p}, {p}, #1",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            n = inout(reg) n => _,
            c = in(reg) c as u32,
            t = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - s as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_chr16(s: *const u16, c: u16, n: usize) -> usize {
    let mut p = s;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrh {t:w}, [{p}]",
            "cmp {t:w}, {c:w}",
            "b.eq 9f",
            "add {p}, {p}, #2",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            n = inout(reg) n => _,
            c = in(reg) c as u32,
            t = out(reg) _,
            options(nostack, readonly),
        );
    }
    (p as usize - s as usize) / 2
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_rchr8(s: *const u8, c: u8) -> *const u8 {
    let mut last: *const u8 = core::ptr::null();
    core::arch::asm!(
        "2:",
        "ldrb {t:w}, [{p}]",
        "cmp {t:w}, {c:w}",
        "csel {l}, {p}, {l}, eq",
        "add {p}, {p}, #1",
        "cbnz {t:w}, 2b",
        p = inout(reg) s => _,
        l = inout(reg) last,
        c = in(reg) c as u32,
        t = out(reg) _,
        options(nostack, readonly),
    );
    last
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_rchr16(s: *const u16, c: u16) -> *const u16 {
    let mut last: *const u16 = core::ptr::null();
    core::arch::asm!(
        "2:",
        "ldrh {t:w}, [{p}]",
        "cmp {t:w}, {c:w}",
        "csel {l}, {p}, {l}, eq",
        "add {p}, {p}, #2",
        "cbnz {t:w}, 2b",
        p = inout(reg) s => _,
        l = inout(reg) last,
        c = in(reg) c as u32,
        t = out(reg) _,
        options(nostack, readonly),
    );
    last
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn span_map8(set: *const u8, map: *mut u8) {
    core::arch::asm!(
        "add {t}, {m}, #256",
        "5:",
        "stp xzr, xzr, [{t}, #-16]!",
        "cmp {t}, {m}",
        "b.ne 5b",
        "mov {u:w}, #1",
        "3:",
        "ldrb {t:w}, [{q}], #1",
        "cbz {t:w}, 4f",
        "strb {u:w}, [{m}, {t}]",
        "b 3b",
        "4:",
        q = inout(reg) set => _,
        m = in(reg) map,
        t = out(reg) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn span_map16(set: *const u16, map: *mut u8) {
    core::arch::asm!(
        "add {t}, {m}, #256",
        "5:",
        "stp xzr, xzr, [{t}, #-16]!",
        "cmp {t}, {m}",
        "b.ne 5b",
        "mov {u:w}, #1",
        "3:",
        "ldrh {t:w}, [{q}], #2",
        "cbz {t:w}, 4f",
        "cmp {t:w}, #255",
        "b.hi 3b",
        "strb {u:w}, [{m}, {t}]",
        "b 3b",
        "4:",
        q = inout(reg) set => _,
        m = in(reg) map,
        t = out(reg) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn span_scan8(s: *const u8, map: *const u8, go: u32) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "ldrb {t:w}, [{p}]",
        "ldrb {t:w}, [{m}, {t}]",
        "cmp {t:w}, {g:w}",
        "b.ne 9f",
        "add {p}, {p}, #1",
        "b 2b",
        "9:",
        p = inout(reg) p,
        m = in(reg) map,
        g = in(reg) go,
        t = out(reg) _,
        options(nostack, readonly),
    );
    p as usize - s as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn span_scan16(s: *const u16, map: *const u8, go: u32) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "ldrh {t:w}, [{p}]",
        "cmp {t:w}, #255",
        "b.hi 9f",
        "ldrb {t:w}, [{m}, {t}]",
        "cmp {t:w}, {g:w}",
        "b.ne 9f",
        "add {p}, {p}, #2",
        "b 2b",
        "9:",
        p = inout(reg) p,
        m = in(reg) map,
        g = in(reg) go,
        t = out(reg) _,
        options(nostack, readonly),
    );
    (p as usize - s as usize) / 2
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn mem_chr8(s: *const u8, c: u8, n: usize) -> usize {
    let mut p = s;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, byte ptr [{p}]",
            "cmp {t:e}, {c:e}",
            "je 9f",
            "add {p}, 1",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            n = inout(reg) n => _,
            c = in(reg) c as u32 as usize,
            t = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - s as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn mem_chr16(s: *const u16, c: u16, n: usize) -> usize {
    let mut p = s;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, word ptr [{p}]",
            "cmp {t:e}, {c:e}",
            "je 9f",
            "add {p}, 2",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            n = inout(reg) n => _,
            c = in(reg) c as u32 as usize,
            t = out(reg) _,
            options(nostack, readonly),
        );
    }
    (p as usize - s as usize) / 2
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_rchr8(s: *const u8, c: u8) -> *const u8 {
    let mut last: *const u8 = core::ptr::null();
    core::arch::asm!(
        "2:",
        "movzx {t:e}, byte ptr [{p}]",
        "cmp {t:e}, {c:e}",
        "cmove {l}, {p}",
        "add {p}, 1",
        "test {t:e}, {t:e}",
        "jnz 2b",
        p = inout(reg) s => _,
        l = inout(reg) last,
        c = in(reg) c as u32 as usize,
        t = out(reg) _,
        options(nostack, readonly),
    );
    last
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_rchr16(s: *const u16, c: u16) -> *const u16 {
    let mut last: *const u16 = core::ptr::null();
    core::arch::asm!(
        "2:",
        "movzx {t:e}, word ptr [{p}]",
        "cmp {t:e}, {c:e}",
        "cmove {l}, {p}",
        "add {p}, 2",
        "test {t:e}, {t:e}",
        "jnz 2b",
        p = inout(reg) s => _,
        l = inout(reg) last,
        c = in(reg) c as u32 as usize,
        t = out(reg) _,
        options(nostack, readonly),
    );
    last
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn span_map8(set: *const u8, map: *mut u8) {
    core::arch::asm!(
        "mov {t}, 252",
        "5:",
        "mov dword ptr [{m} + {t}], 0",
        "sub {t}, 4",
        "jns 5b",
        "3:",
        "movzx {t:e}, byte ptr [{q}]",
        "test {t:e}, {t:e}",
        "jz 4f",
        "mov byte ptr [{m} + {t}], 1",
        "add {q}, 1",
        "jmp 3b",
        "4:",
        q = inout(reg) set => _,
        m = in(reg) map,
        t = out(reg) _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn span_map16(set: *const u16, map: *mut u8) {
    core::arch::asm!(
        "mov {t}, 252",
        "5:",
        "mov dword ptr [{m} + {t}], 0",
        "sub {t}, 4",
        "jns 5b",
        "3:",
        "movzx {t:e}, word ptr [{q}]",
        "add {q}, 2",
        "test {t:e}, {t:e}",
        "jz 4f",
        "cmp {t:e}, 255",
        "ja 3b",
        "mov byte ptr [{m} + {t}], 1",
        "jmp 3b",
        "4:",
        q = inout(reg) set => _,
        m = in(reg) map,
        t = out(reg) _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn span_scan8(s: *const u8, map: *const u8, go: u32) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "movzx {t:e}, byte ptr [{p}]",
        "movzx {t:e}, byte ptr [{m} + {t}]",
        "cmp {t:e}, {g:e}",
        "jne 9f",
        "add {p}, 1",
        "jmp 2b",
        "9:",
        p = inout(reg) p,
        m = in(reg) map,
        g = in(reg) go as usize,
        t = out(reg) _,
        options(nostack, readonly),
    );
    p as usize - s as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn span_scan16(s: *const u16, map: *const u8, go: u32) -> usize {
    let mut p = s;
    core::arch::asm!(
        "2:",
        "movzx {t:e}, word ptr [{p}]",
        "cmp {t:e}, 255",
        "ja 9f",
        "movzx {t:e}, byte ptr [{m} + {t}]",
        "cmp {t:e}, {g:e}",
        "jne 9f",
        "add {p}, 2",
        "jmp 2b",
        "9:",
        p = inout(reg) p,
        m = in(reg) map,
        g = in(reg) go as usize,
        t = out(reg) _,
        options(nostack, readonly),
    );
    (p as usize - s as usize) / 2
}

/// strrchr: `c` cut to `char`; 0 finds the terminator (the scan's last hit); NULL finds nothing (the slice shim's
/// `s.add(i)` on a NULL base is NULL whatever `i` is).
pub(crate) unsafe fn fast_strrchr(s: *const u8, c: i32) -> *mut u8 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    str_rchr8(s, c as u8) as *mut u8
}

pub(crate) unsafe fn fast_wcsrchr(s: *const u16, c: u16) -> *mut u16 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    str_rchr16(s, c) as *mut u16
}

/// strstr: candidates by `str_chr8` on the needle's first byte, each confirmed by
/// `str_mismatch8` over the rest (which can only reach the rest's length on a match, since a
/// haystack NUL inside it differs). Empty or NULL needle → the haystack (`find_sub`'s Some(0)).
pub(crate) unsafe fn fast_strstr(hay: *const u8, needle: *const u8) -> *mut u8 {
    if needle.is_null() || *needle == 0 {
        return hay as *mut u8;
    }
    if hay.is_null() {
        return core::ptr::null_mut();
    }
    let rest = needle.add(1);
    let rlen = str_len8(rest);
    let mut p = hay;
    loop {
        p = p.add(str_chr8(p, *needle));
        if *p == 0 {
            return core::ptr::null_mut();
        }
        if str_mismatch8(p.add(1), rest, rlen) == rlen {
            return p as *mut u8;
        }
        p = p.add(1);
    }
}

/// wcsstr: as strstr, except an empty or NULL needle → NULL (`find_sub_strict`).
pub(crate) unsafe fn fast_wcsstr(hay: *const u16, needle: *const u16) -> *mut u16 {
    if needle.is_null() || *needle == 0 || hay.is_null() {
        return core::ptr::null_mut();
    }
    let rest = needle.add(1);
    let rlen = str_len16(rest);
    let mut p = hay;
    loop {
        p = p.add(str_chr16(p, *needle));
        if *p == 0 {
            return core::ptr::null_mut();
        }
        if str_mismatch16(p.add(1), rest, rlen) == rlen {
            return p as *mut u16;
        }
        p = p.add(1);
    }
}

/// The leading run of `s` whose membership in `set` is `want_in`. NUL stops both forms: it is
/// never a member (spn stops), and cspn marks it a member.
unsafe fn span8(s: *const u8, set: *const u8, want_in: bool) -> usize {
    let s = if s.is_null() { NUL8 as *const u8 } else { s };
    let set = if set.is_null() {
        NUL8 as *const u8
    } else {
        set
    };
    let mut map = core::mem::MaybeUninit::<[u8; 256]>::uninit();
    let m = map.as_mut_ptr() as *mut u8;
    span_map8(set, m);
    if !want_in {
        *m = 1;
    }
    span_scan8(s, m, want_in as u32)
}

/// Wide: the map answers below 256; an element at or above it is looked up in the set itself
/// (the C's own linear membership test), and the scan resumes past it if it does not stop.
unsafe fn span16(s: *const u16, set: *const u16, want_in: bool) -> usize {
    let s = if s.is_null() { NUL16 as *const u16 } else { s };
    let set = if set.is_null() {
        NUL16 as *const u16
    } else {
        set
    };
    let mut map = core::mem::MaybeUninit::<[u8; 256]>::uninit();
    let m = map.as_mut_ptr() as *mut u8;
    span_map16(set, m);
    if !want_in {
        *m = 1;
    }
    let mut i = 0usize;
    loop {
        i += span_scan16(s.add(i), m, want_in as u32);
        let c = *s.add(i);
        if c < 256 {
            return i;
        }
        let member = *set.add(str_chr16(set, c)) == c;
        if member != want_in {
            return i;
        }
        i += 1;
    }
}

pub(crate) unsafe fn fast_strspn(s: *const u8, accept: *const u8) -> usize {
    span8(s, accept, true)
}

pub(crate) unsafe fn fast_strcspn(s: *const u8, reject: *const u8) -> usize {
    span8(s, reject, false)
}

pub(crate) unsafe fn fast_wcsspn(s: *const u16, accept: *const u16) -> usize {
    span16(s, accept, true)
}

pub(crate) unsafe fn fast_wcscspn(s: *const u16, reject: *const u16) -> usize {
    span16(s, reject, false)
}

/// strpbrk: where strcspn stops, unless that is the terminator. Empty set → NULL.
pub(crate) unsafe fn fast_strpbrk(s1: *const u8, s2: *const u8) -> *mut u8 {
    if s1.is_null() {
        return core::ptr::null_mut();
    }
    let i = span8(s1, s2, false);
    if *s1.add(i) == 0 {
        core::ptr::null_mut()
    } else {
        s1.add(i) as *mut u8
    }
}

/// memchr compares each byte as signed char against the int (`mem::chr`), so a `c` outside
/// -128..=127 matches nothing; inside it, the byte to find is `c as u8`. n == 0 or NULL → NULL.
pub(crate) unsafe fn fast_memchr(
    s: *const core::ffi::c_void,
    c: i32,
    n: usize,
) -> *mut core::ffi::c_void {
    if s.is_null() || n == 0 || !(-128..=127).contains(&c) {
        return core::ptr::null_mut();
    }
    let p = s as *const u8;
    let i = mem_chr8(p, c as u8, n);
    if i == n {
        core::ptr::null_mut()
    } else {
        p.add(i) as *mut core::ffi::c_void
    }
}

// ---------------------------------------------------------------------------------------------
// memcmp, _memicmp and the ASCII case maps (_strlwr, _strupr, _wcslwr, _wcsupr): 15-32x the C at
// 256 as slice loops (`cnt-difftest --test perf`).
//
// `mem_mismatch*(a, b, n)` is `str_mismatch*` without the NUL stop: the first index below `n`
// whose bytes differ (after the A..Z fold, for `_ci`), or `n`. The fold loops rewrite every
// element of a NUL-terminated string in place that lies in one 26-letter range, by +-32.
// ---------------------------------------------------------------------------------------------

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_mismatch8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{p}]",
            "ldrb {u:w}, [{q}]",
            "cmp {t:w}, {u:w}",
            "b.ne 9f",
            "add {p}, {p}, #1",
            "add {q}, {q}, #1",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn mem_mismatch_ci8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "ldrb {t:w}, [{p}]",
            "ldrb {u:w}, [{q}]",
            "sub {v:w}, {t:w}, #65",
            "cmp {v:w}, #25",
            "add {v:w}, {t:w}, #32",
            "csel {t:w}, {v:w}, {t:w}, ls",
            "sub {v:w}, {u:w}, #65",
            "cmp {v:w}, #25",
            "add {v:w}, {u:w}, #32",
            "csel {u:w}, {v:w}, {u:w}, ls",
            "cmp {t:w}, {u:w}",
            "b.ne 9f",
            "add {p}, {p}, #1",
            "add {q}, {q}, #1",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            v = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn mem_mismatch8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, byte ptr [{p}]",
            "cmp {t:l}, byte ptr [{q}]",
            "jne 9f",
            "add {p}, 1",
            "add {q}, 1",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg_abcd) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn mem_mismatch_ci8(a: *const u8, b: *const u8, n: usize) -> usize {
    let mut p = a;
    if n != 0 {
        core::arch::asm!(
            "2:",
            "movzx {t:e}, byte ptr [{p}]",
            "movzx {u:e}, byte ptr [{q}]",
            "cmp {t:e}, {u:e}",
            "je 6f",
            "cmp {t:e}, 65",
            "jb 3f",
            "cmp {t:e}, 90",
            "ja 3f",
            "add {t:e}, 32",
            "3:",
            "cmp {u:e}, 65",
            "jb 4f",
            "cmp {u:e}, 90",
            "ja 4f",
            "add {u:e}, 32",
            "4:",
            "cmp {t:e}, {u:e}",
            "jne 9f",
            "6:",
            "add {p}, 1",
            "add {q}, 1",
            "sub {n}, 1",
            "jnz 2b",
            "9:",
            p = inout(reg) p,
            q = inout(reg) b => _,
            n = inout(reg) n => _,
            t = out(reg) _,
            u = out(reg) _,
            options(nostack, readonly),
        );
    }
    p as usize - a as usize
}

// The fold loops are written out, not stamped by a macro: rustconv difftest contracts select
// helpers by item name, and a macro invocation has none. `lo` is the range's first letter.

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_lower8(s: *mut u8) {
    core::arch::asm!(
        "2:",
        "ldrb {t:w}, [{p}]",
        "cbz {t:w}, 9f",
        "sub {u:w}, {t:w}, #65",
        "cmp {u:w}, #25",
        "b.hi 3f",
        "add {t:w}, {t:w}, #32",
        "strb {t:w}, [{p}]",
        "3:",
        "add {p}, {p}, #1",
        "b 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_lower8(s: *mut u8) {
    core::arch::asm!(
        "2:",
        "movzx {t:e}, byte ptr [{p}]",
        "test {t:e}, {t:e}",
        "jz 9f",
        "mov {u:e}, {t:e}",
        "sub {u:e}, 65",
        "cmp {u:e}, 25",
        "ja 3f",
        "add {t:e}, 32",
        "mov byte ptr [{p}], {t:l}",
        "3:",
        "add {p}, 1",
        "jmp 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg_abcd) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_upper8(s: *mut u8) {
    core::arch::asm!(
        "2:",
        "ldrb {t:w}, [{p}]",
        "cbz {t:w}, 9f",
        "sub {u:w}, {t:w}, #97",
        "cmp {u:w}, #25",
        "b.hi 3f",
        "sub {t:w}, {t:w}, #32",
        "strb {t:w}, [{p}]",
        "3:",
        "add {p}, {p}, #1",
        "b 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_upper8(s: *mut u8) {
    core::arch::asm!(
        "2:",
        "movzx {t:e}, byte ptr [{p}]",
        "test {t:e}, {t:e}",
        "jz 9f",
        "mov {u:e}, {t:e}",
        "sub {u:e}, 97",
        "cmp {u:e}, 25",
        "ja 3f",
        "sub {t:e}, 32",
        "mov byte ptr [{p}], {t:l}",
        "3:",
        "add {p}, 1",
        "jmp 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg_abcd) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_lower16(s: *mut u16) {
    core::arch::asm!(
        "2:",
        "ldrh {t:w}, [{p}]",
        "cbz {t:w}, 9f",
        "sub {u:w}, {t:w}, #65",
        "cmp {u:w}, #25",
        "b.hi 3f",
        "add {t:w}, {t:w}, #32",
        "strh {t:w}, [{p}]",
        "3:",
        "add {p}, {p}, #2",
        "b 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_lower16(s: *mut u16) {
    core::arch::asm!(
        "2:",
        "movzx {t:e}, word ptr [{p}]",
        "test {t:e}, {t:e}",
        "jz 9f",
        "mov {u:e}, {t:e}",
        "sub {u:e}, 65",
        "cmp {u:e}, 25",
        "ja 3f",
        "add {t:e}, 32",
        "mov word ptr [{p}], {t:x}",
        "3:",
        "add {p}, 2",
        "jmp 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg_abcd) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(target_arch = "aarch64")]
pub(crate) unsafe fn str_upper16(s: *mut u16) {
    core::arch::asm!(
        "2:",
        "ldrh {t:w}, [{p}]",
        "cbz {t:w}, 9f",
        "sub {u:w}, {t:w}, #97",
        "cmp {u:w}, #25",
        "b.hi 3f",
        "sub {t:w}, {t:w}, #32",
        "strh {t:w}, [{p}]",
        "3:",
        "add {p}, {p}, #2",
        "b 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg) _,
        u = out(reg) _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn str_upper16(s: *mut u16) {
    core::arch::asm!(
        "2:",
        "movzx {t:e}, word ptr [{p}]",
        "test {t:e}, {t:e}",
        "jz 9f",
        "mov {u:e}, {t:e}",
        "sub {u:e}, 97",
        "cmp {u:e}, 25",
        "ja 3f",
        "sub {t:e}, 32",
        "mov word ptr [{p}], {t:x}",
        "3:",
        "add {p}, 2",
        "jmp 2b",
        "9:",
        p = inout(reg) s => _,
        t = out(reg_abcd) _,
        u = out(reg) _,
        options(nostack),
    );
}

/// memcmp: the unsigned byte difference at the first mismatch; n == 0 or a NULL side → 0 (the
/// slice shim zipped an empty slice).
pub(crate) unsafe fn fast_memcmp(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    n: usize,
) -> i32 {
    if s1.is_null() || s2.is_null() || n == 0 {
        return 0;
    }
    let (a, b) = (s1 as *const u8, s2 as *const u8);
    let i = mem_mismatch8(a, b, n);
    if i == n {
        return 0;
    }
    *a.add(i) as i32 - *b.add(i) as i32
}

/// _memicmp: compares folded, returns the RAW unsigned difference (`mem::icmp`); a NULL side
/// → _NLSCMPERROR when n != 0.
pub(crate) unsafe fn fast_memicmp(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    n: usize,
) -> i32 {
    if s1.is_null() || s2.is_null() {
        return if n != 0 { 0x7fff_ffff } else { 0 };
    }
    let (a, b) = (s1 as *const u8, s2 as *const u8);
    let i = mem_mismatch_ci8(a, b, n);
    if i == n {
        return 0;
    }
    *a.add(i) as i32 - *b.add(i) as i32
}

/// The case maps: ASCII only, in place, return the argument. Every shim returned NULL for NULL,
/// guarded or not (the slice of NULL is empty, and the unguarded ones return `_String`).
pub(crate) unsafe fn fast_strlwr(s: *mut u8) -> *mut u8 {
    if !s.is_null() {
        str_lower8(s);
    }
    s
}

pub(crate) unsafe fn fast_strupr(s: *mut u8) -> *mut u8 {
    if !s.is_null() {
        str_upper8(s);
    }
    s
}

pub(crate) unsafe fn fast_wcslwr(s: *mut u16) -> *mut u16 {
    if !s.is_null() {
        str_lower16(s);
    }
    s
}

pub(crate) unsafe fn fast_wcsupr(s: *mut u16) -> *mut u16 {
    if !s.is_null() {
        str_upper16(s);
    }
    s
}
