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
