//! i386 fault-tolerant kernel copy — a Linux-extable-style fault-fixup primitive.
//!
//! This is the i386 arm of `KmProtectedCopyMemory` (the amd64/arm64 arms are the
//! native-SEH `__try` shim in `ntoskrnl/kmsafecopy.c`). It closes the honest gap
//! that file and `ntkrnl_sys::probe::fault_tolerant_copy` both documented:
//!
//! > "The arch-neutral close for BOTH i386 and arm64 is a Linux-extable-style
//! >  fault-fixup primitive (a table keyed by faulting PC, consulted in
//! >  KiPageFault) — it needs no working SEH at all."
//!
//! **Why not SEH here.** i386 frame-based `__try/__except` only lowers under the
//! clang `-windows-msvc` env, and even with it the kernel-mode catch does not work
//! (conformance-ratchet-quest.md, 2026-07-17 spike; task #18). This primitive
//! sidesteps that entirely: no exception dispatch, no registration chain, no
//! unwinder. The page-fault handler recognises ONE instruction address and
//! redirects `Eip` to a fixup label. That is the whole mechanism.
//!
//! **Why not an `MmIsAddressValid` pre-check.** It cannot distinguish "invalid"
//! from "valid but paged out", so on the I/O completion hot path it would
//! spuriously fail legitimate I/O under memory pressure. The extable approach has
//! the same semantics as the amd64 SEH path: the fault goes through
//! `MmAccessFault` FIRST, so a valid-but-nonresident page is simply paged in and
//! the copy proceeds — only a genuinely invalid address reaches the fixup.
//!
//! **The faulting instruction is `rep movsb`, deliberately.** It is a single
//! restartable instruction, so the extable has exactly one entry and the key is
//! an exact `Eip` match rather than a range. (It is also not the byte-at-a-time
//! loop `kmsafecopy.c` warns about: the CPU moves wide chunks internally, and
//! QEMU's TCG lowers it in bulk too.) A partially-completed `rep` leaves
//! esi/edi/ecx advanced, which is fine — the contract is a boolean, not a count.

use ntkrnl_sys::ddk::*;

// The copy primitive plus its fixup landing pad. `_KmProtectedCopyMemory` is
// cdecl (`extern "C"`), matching the declaration in ntkrnl-sys probe.rs and the
// C prototype the other two arches export from kmsafecopy.c.
//
// Stack on entry:  [esp]=ret [esp+4]=Dst [esp+8]=Src [esp+12]=Len
// After the two pushes: +8=ret +12=Dst +16=Src +20=Len
core::arch::global_asm!(
    ".text",
    ".global _KmProtectedCopyMemory",
    "_KmProtectedCopyMemory:",
    "push edi",
    "push esi",
    "mov edi, [esp+12]", // Dst
    "mov esi, [esp+16]", // Src
    "mov ecx, [esp+20]", // Len
    "cld",
    // THE faulting instruction. Its address is the extable key; a page fault
    // here reports Eip pointing AT this instruction (rep-prefixed string ops are
    // restartable), so the lookup is an exact compare.
    ".global _KmpProtectedCopyFaultPc",
    "_KmpProtectedCopyFaultPc:",
    "rep movsb",
    "mov eax, 1",
    "jmp 2f",
    // Fixup landing pad: KiTrap0EHandler redirects Eip here when the copy
    // faults. Returns 0 -> the Rust caller maps it to STATUS_ACCESS_VIOLATION.
    ".global _KmpProtectedCopyFixup",
    "_KmpProtectedCopyFixup:",
    "xor eax, eax",
    "2:",
    "pop esi",
    "pop edi",
    "ret",
);

// The atomic twin: `KmProtectedInterlockedAdd(Addend, Value, Result)`, cdecl,
// same ABI as the native-SEH version in kmsafecopy.c. A read-modify-write on user
// memory cannot be expressed as a copy (capture -> mutate -> publish is not
// atomic), so the `lock xadd` itself is the second extable key. A fault on it is
// precise -- nothing was written -- and reports Eip at the lock prefix, which is
// the label.
//
// Stack on entry:  [esp]=ret [esp+4]=Addend [esp+8]=Value [esp+12]=Result
core::arch::global_asm!(
    ".text",
    ".global _KmProtectedInterlockedAdd",
    "_KmProtectedInterlockedAdd:",
    "mov ecx, [esp+4]", // Addend
    "mov eax, [esp+8]", // Value
    "mov edx, eax",
    ".global _KmpProtectedXaddFaultPc",
    "_KmpProtectedXaddFaultPc:",
    "lock xadd [ecx], eax", // eax = old value
    "add eax, edx",         // new value, as InterlockedAdd returns it
    "mov ecx, [esp+12]",    // Result is kernel memory; only the xadd is protected
    "mov [ecx], eax",
    "mov eax, 1",
    "ret",
    ".global _KmpProtectedXaddFixup",
    "_KmpProtectedXaddFixup:",
    "xor eax, eax",
    "ret",
);

extern "C" {
    /// Address of the `rep movsb` above — an extable key.
    static KmpProtectedCopyFaultPc: u8;
    /// Address of the failure landing pad.
    static KmpProtectedCopyFixup: u8;
    /// Address of the `lock xadd` — the other extable key.
    static KmpProtectedXaddFaultPc: u8;
    static KmpProtectedXaddFixup: u8;
}

/// The exception table lookup, consulted by `KiTrap0EHandler` on a kernel-mode
/// fault that `MmAccessFault` declined.
///
/// Returns `true` (and rewrites `TrapFrame->Eip` to the fixup pad) when the
/// faulting PC is our protected copy, meaning the caller should resume the guest
/// instead of raising an exception. Returns `false` for every other PC, so a
/// genuine kernel bug still bugchecks exactly as before — this must never
/// swallow a fault it does not own.
#[inline]
pub unsafe fn KiFixupProtectedCopy(TrapFrame: PKTRAP_FRAME) -> bool {
    let eip = (*TrapFrame).Eip;
    let fixup = if eip == core::ptr::addr_of!(KmpProtectedCopyFaultPc) as ULONG {
        core::ptr::addr_of!(KmpProtectedCopyFixup)
    } else if eip == core::ptr::addr_of!(KmpProtectedXaddFaultPc) as ULONG {
        core::ptr::addr_of!(KmpProtectedXaddFixup)
    } else {
        return false;
    };
    (*TrapFrame).Eip = fixup as ULONG;
    true
}

/// Per-boot tripwire, the i386 twin of `KmSafeCopySelfTest` in kmsafecopy.c
/// (which is not compiled on i386 — its `__try` needs the msvc env).
///
/// Both assertions are load-bearing, for the reason that file spells out: a
/// primitive that "never faults" because it silently copies nothing would pass a
/// bad-pointer-only test. So assert that a VALID copy still succeeds AND that an
/// INVALID one is caught. Address 0x1 is unmapped on i386 and below
/// MmUserProbeAddress, so a range-only probe would wave it through — exactly the
/// hostile-pointer shape that produced the memcpy+0x40 cluster.
#[no_mangle]
pub unsafe extern "C" fn KmSafeCopySelfTest() -> i32 {
    extern "C" {
        fn KmProtectedCopyMemory(
            dst: *mut core::ffi::c_void,
            src: *const core::ffi::c_void,
            len: usize,
        ) -> i32;
        fn DbgPrint(fmt: *const u8, ...) -> ULONG;
    }

    let src: [u8; 8] = *b"vibeos!\0";
    let mut dst: [u8; 8] = [0; 8];

    let good = KmProtectedCopyMemory(
        dst.as_mut_ptr() as *mut core::ffi::c_void,
        src.as_ptr() as *const core::ffi::c_void,
        src.len(),
    );
    let bad = KmProtectedCopyMemory(
        dst.as_mut_ptr() as *mut core::ffi::c_void,
        0x1 as *const core::ffi::c_void,
        src.len(),
    );

    // A valid copy must also have actually MOVED the bytes — the "silently
    // copies nothing" failure mode the C file warns about.
    let moved = (dst[0] == b'v' && dst[6] == b'!') as i32;
    let ok = (good == 1 && bad == 0 && moved == 1) as i32;

    DbgPrint(
        b"[KMSAFECOPY] fault-tolerant copy: valid=%d moved=%d invalid_caught=%d (%s)\n\0"
            .as_ptr(),
        good,
        moved,
        (bad == 0) as i32,
        if ok != 0 {
            b"PASS\0".as_ptr()
        } else {
            b"FAIL\0".as_ptr()
        },
    );

    (ok != 0 && KmSafeAtomicSelfTest() != 0) as i32
}

/// The atomic's twin of the copy test above, printing the same line the
/// kmsafecopy.c version does. `landed` is the assertion a copy does not need:
/// the add must be visible IN MEMORY, not merely returned.
unsafe fn KmSafeAtomicSelfTest() -> i32 {
    extern "C" {
        fn KmProtectedInterlockedAdd(addend: *mut i32, value: i32, result: *mut i32) -> i32;
        fn DbgPrint(fmt: *const u8, ...) -> ULONG;
    }

    let mut counter: i32 = 41;
    let mut result: i32 = 0;
    let mut junk: i32 = 0;

    let good = KmProtectedInterlockedAdd(core::ptr::addr_of_mut!(counter), 1, &mut result);
    let landed = (core::ptr::read_volatile(core::ptr::addr_of!(counter)) == 42 && result == 42) as i32;
    let bad = KmProtectedInterlockedAdd(0x1 as *mut i32, 1, &mut junk);

    let ok = (good == 1 && landed == 1 && bad == 0) as i32;
    DbgPrint(
        b"[KMSAFECOPY] fault-tolerant interlocked add: valid=%d landed=%d invalid_caught=%d (%s)\n\0"
            .as_ptr(),
        good,
        landed,
        (bad == 0) as i32,
        if ok != 0 {
            b"PASS\0".as_ptr()
        } else {
            b"FAIL\0".as_ptr()
        },
    );
    ok
}
