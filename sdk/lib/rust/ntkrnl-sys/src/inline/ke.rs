//! KE subsystem inline functions.
//!
//! Context register access, PCR access, interrupt control, cache flush.

use crate::base::*;
use crate::ddk;

// Type aliases for types used by ke_x.h inlines (not imported via base::*)
type KPRIORITY = LONG; // i32
type KPROCESSOR_MODE = i8; // CCHAR

const EFLAGS_INTERRUPT_MASK: u32 = 0x200;

// ============================================================================
// KeSetContextPc (FORCEINLINE from ke.h per arch)
// ============================================================================

#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeSetContextPc(ctx: *mut ddk::_CONTEXT, pc: ULONG_PTR) {
    (*ctx).Eip = pc as ULONG;
}

#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeSetContextPc(ctx: *mut ddk::_CONTEXT, pc: ULONG_PTR) {
    (*ctx).Rip = pc as u64;
}

#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeSetContextPc(ctx: *mut ddk::_CONTEXT, pc: ULONG_PTR) {
    (*ctx).Pc = pc as u64;
}

// ============================================================================
// KeSetContextReturnRegister (FORCEINLINE from ke.h per arch)
// ============================================================================

#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeSetContextReturnRegister(ctx: *mut ddk::_CONTEXT, val: ULONG_PTR) {
    (*ctx).Eax = val as ULONG;
}

#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeSetContextReturnRegister(ctx: *mut ddk::_CONTEXT, val: ULONG_PTR) {
    (*ctx).Rax = val as u64;
}

#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeSetContextReturnRegister(ctx: *mut ddk::_CONTEXT, val: ULONG_PTR) {
    (*ctx).__bindgen_anon_1.X[0] = val as u64;
}

// ============================================================================
// KeGetContextPc (FORCEINLINE from ke.h per arch)
// ============================================================================

#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeGetContextPc(ctx: *const ddk::_CONTEXT) -> ULONG_PTR {
    (*ctx).Eip as ULONG_PTR
}

#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeGetContextPc(ctx: *const ddk::_CONTEXT) -> ULONG_PTR {
    (*ctx).Rip as ULONG_PTR
}

#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeGetContextPc(ctx: *const ddk::_CONTEXT) -> ULONG_PTR {
    (*ctx).Pc as ULONG_PTR
}

// ============================================================================
// KeGetTrapFrame (FORCEINLINE from ke.h)
// ============================================================================

/// KeGetTrapFrame — compute trap frame location from thread's InitialStack.
///
/// i386:  `InitialStack - sizeof(KTRAP_FRAME) - sizeof(FX_SAVE_AREA)`
/// amd64: `InitialStack - sizeof(KTRAP_FRAME)`
/// arm64: `InitialStack - sizeof(KTRAP_FRAME)`
///
/// NOTE: This is NOT `Thread->TrapFrame` — that field is set dynamically on
/// trap/syscall entry and may be NULL for newly created threads. The C macro
/// computes the static location from the thread's initial stack layout.
#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeGetTrapFrame(thread: ddk::PKTHREAD) -> ddk::PKTRAP_FRAME {
    (((*thread).InitialStack as usize)
        - core::mem::size_of::<ddk::_KTRAP_FRAME>()
        - core::mem::size_of::<ddk::_FX_SAVE_AREA>()) as ddk::PKTRAP_FRAME
}

/// KeGetTrapFrame — amd64/arm64: `InitialStack - sizeof(KTRAP_FRAME)`
#[inline]
#[cfg(not(target_arch = "x86"))]
pub unsafe fn KeGetTrapFrame(thread: ddk::PKTHREAD) -> ddk::PKTRAP_FRAME {
    (((*thread).InitialStack as usize)
        - core::mem::size_of::<ddk::_KTRAP_FRAME>()) as ddk::PKTRAP_FRAME
}

// ============================================================================
// KeGetPcr — read processor control region pointer (per arch)
// ============================================================================

/// KeGetPcr — amd64: read KPCR.Self from GS segment (offset 0x18).
#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeGetPcr() -> ddk::PKPCR {
    let result: *mut ddk::_KPCR;
    core::arch::asm!(
        "mov {}, gs:[0x18]",
        out(reg) result,
        options(nostack, nomem, preserves_flags),
    );
    result
}

/// KeGetPcr — i386: read KPCR.SelfPcr from FS segment (offset 0x1C).
#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeGetPcr() -> ddk::PKPCR {
    let result: u32;
    core::arch::asm!(
        "mov {:e}, dword ptr fs:[0x1C]",
        out(reg) result,
        options(nostack, nomem, preserves_flags),
    );
    result as ddk::PKPCR
}

/// KeGetPcr — arm64: read KPCR from TPIDR_EL1.
#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeGetPcr() -> ddk::PKPCR {
    let result: *mut ddk::_KPCR;
    core::arch::asm!(
        "mrs {}, tpidr_el1",
        out(reg) result,
        options(nostack, nomem, preserves_flags),
    );
    result
}

// ============================================================================
// KeGetCurrentProcessorNumber (FORCEINLINE). UP kernel: constant 0. SMP:
// Prcb->Number via the per-arch PCR — a hardcoded 0 here silently poisons
// the IPI completion protocol / dispatcher remote-IPI branch / freeze CPU
// identity on every CPU but the BSP (smp-bringup M3 audit, item 3).
// ============================================================================

#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KeGetCurrentProcessorNumber() -> ULONG {
    0
}

#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KeGetCurrentProcessorNumber() -> ULONG {
    // KPRCB.Number is CHAR/UCHAR-sized on x86 and USHORT on amd64/arm64 in
    // the NT layouts; the bindgen field is typed per arch — widen to ULONG.
    (*KeGetCurrentPrcb()).Number as ULONG
}

// ============================================================================
// KeDisableInterrupts (FORCEINLINE from ke.h per arch)
// ============================================================================

/// KeDisableInterrupts — i386: pushfd, cli, return IF bit.
#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeDisableInterrupts() -> BOOLEAN {
    let flags: u32;
    core::arch::asm!(
        "pushfd",
        "pop {:e}",
        "cli",
        out(reg) flags,
    );
    if flags & EFLAGS_INTERRUPT_MASK != 0 { 1 } else { 0 }
}

/// KeDisableInterrupts — amd64: pushfq, cli, return IF bit.
#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeDisableInterrupts() -> BOOLEAN {
    let flags: u64;
    core::arch::asm!(
        "pushfq",
        "pop {}",
        "cli",
        out(reg) flags,
    );
    if flags & EFLAGS_INTERRUPT_MASK as u64 != 0 { 1 } else { 0 }
}

/// KeDisableInterrupts — arm64: mrs daif, msr daifset #2, return I bit.
#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeDisableInterrupts() -> BOOLEAN {
    let flags: u64;
    core::arch::asm!(
        "mrs {}, daif",
        "msr daifset, #2",
        out(reg) flags,
        options(nomem, nostack),
    );
    // I bit (bit 7): 0 = enabled, 1 = disabled
    if flags & 0x80 == 0 { 1 } else { 0 }
}

// ============================================================================
// KeRestoreInterrupts (FORCEINLINE from ke.h per arch)
// ============================================================================

#[inline]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn KeRestoreInterrupts(were_enabled: BOOLEAN) {
    if were_enabled != 0 {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeRestoreInterrupts(were_enabled: BOOLEAN) {
    if were_enabled != 0 {
        core::arch::asm!("msr daifclr, #2", options(nomem, nostack));
    }
}

// ============================================================================
// KeSweepICache (FORCEINLINE from ke.h per arch)
// ============================================================================

/// KeSweepICache — x86/x86_64: always sweep entire cache via wbinvd.
#[inline]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn KeSweepICache(_base: PVOID, _size: ULONG_PTR) {
    core::arch::asm!("wbinvd", options(nostack));
}

/// KeSweepICache — arm64: make freshly written code visible to instruction fetch.
///
/// ARM's instruction fetch is not coherent with its data cache: written code sits
/// dirty in the D-cache until cleaned to the point of unification, and the
/// I-cache can hold stale instructions for the same address. A range cleans each
/// line by VA; a NULL base (or zero size) means "the whole cache", which is what
/// NtFlushInstructionCache(NULL, 0) asks for after the loader relocates an image,
/// and needs a set/way clean because there is no range to walk. QEMU models no
/// caches, so only hardware tells a missing clean from a present one.
#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeSweepICache(base: PVOID, size: ULONG_PTR) {
    if !base.is_null() && size > 0 {
        let base_addr = base as usize;
        let mut addr = base_addr & !63usize;
        let end = (base_addr + size + 63) & !63usize;
        while addr < end {
            core::arch::asm!("dc cvau, {}", in(reg) addr, options(nostack));
            addr += 64;
        }
    } else {
        clean_dcache_to_pou_by_set_way();
    }
    core::arch::asm!(
        "dsb ish",
        "ic ialluis",
        "dsb ish",
        "isb",
        options(nostack),
    );
}

/// Clean every data-cache level below the point of unification, by set/way.
/// Set/way operations act on the LOCAL core's caches only; that is exact on a
/// uniprocessor boot and best-effort on MP (the writer is normally this core).
#[cfg(target_arch = "aarch64")]
unsafe fn clean_dcache_to_pou_by_set_way() {
    let clidr: u64;
    core::arch::asm!("mrs {}, clidr_el1", out(reg) clidr, options(nomem, nostack));
    // CLIDR_EL1.LoUU, bits [29:27]: levels below it must be cleaned for I/D coherence.
    let louu = (clidr >> 27) & 0x7;
    for level in 0..louu {
        // Ctype<n>, 3 bits per level: 2 = data only, 3 = split, 4 = unified.
        if (clidr >> (level * 3)) & 0x7 < 2 {
            continue;
        }
        let ccsidr: u64;
        core::arch::asm!(
            "msr csselr_el1, {sel}",
            "isb",
            "mrs {out}, ccsidr_el1",
            sel = in(reg) level << 1,
            out = out(reg) ccsidr,
            options(nostack),
        );
        // CCSIDR_EL1 without FEAT_CCIDX (Cortex-A72): LineSize [2:0] = log2(bytes)-4,
        // Associativity [12:3] = ways-1, NumSets [27:13] = sets-1.
        let line_shift = (ccsidr & 0x7) + 4;
        let ways = ((ccsidr >> 3) & 0x3FF) + 1;
        let sets = ((ccsidr >> 13) & 0x7FFF) + 1;
        // The way number is left-justified in the 32-bit operand.
        let way_shift = ((ways - 1) as u32).leading_zeros() as u64;
        for way in 0..ways {
            for set in 0..sets {
                let op = (way << way_shift) | (set << line_shift) | (level << 1);
                core::arch::asm!("dc csw, {}", in(reg) op, options(nostack));
            }
        }
    }
    core::arch::asm!("dsb ish", options(nostack));
}

// ============================================================================
// KeGetTrapFrameInterruptState (macro from ke.h per arch)
// ============================================================================

/// KeGetTrapFrameInterruptState — x86/x86_64: check EFlags IF bit.
#[inline]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn KeGetTrapFrameInterruptState(tf: *const ddk::_KTRAP_FRAME) -> BOOLEAN {
    if (*tf).EFlags & EFLAGS_INTERRUPT_MASK != 0 { 1 } else { 0 }
}

/// KeGetTrapFrameInterruptState — arm64: check SPSR I bit (bit 7).
#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeGetTrapFrameInterruptState(tf: *const ddk::_KTRAP_FRAME) -> BOOLEAN {
    // I bit (bit 7) in SPSR: 0 = interrupts enabled, 1 = disabled
    if (*tf).Spsr & 0x80 == 0 { 1 } else { 0 }
}

// ============================================================================
// KeGetCurrentPrcb (FORCEINLINE from ndk/{arch}/ketypes.h)
// ============================================================================

/// KeGetCurrentPrcb — i386: KPCR.Prcb pointer field.
#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeGetCurrentPrcb() -> ddk::PKPRCB {
    (*KeGetPcr()).Prcb
}

/// KeGetCurrentPrcb — amd64: KPCR.CurrentPrcb (nested in anonymous union).
#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeGetCurrentPrcb() -> ddk::PKPRCB {
    (*KeGetPcr()).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb
}

/// KeGetCurrentPrcb — arm64: KPCR.CurrentPrcb (nested in anonymous union).
#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeGetCurrentPrcb() -> ddk::PKPRCB {
    (*KeGetPcr()).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb
}

// ============================================================================
// KTHREAD field access helpers (bindgen nesting differs per arch)
// ============================================================================

/// Get current thread as PKTHREAD on all architectures.
/// On amd64, KeGetCurrentThread is FORCEINLINE (not in bindgen), so use mm::KeGetCurrentThread.
#[inline(always)]
unsafe fn current_thread() -> ddk::PKTHREAD {
    #[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
    { KeGetCurrentThread_portable() }
    #[cfg(target_arch = "x86_64")]
    { crate::mm::KeGetCurrentThread() as ddk::PKTHREAD }
}

/// Get pointer to KTHREAD.KernelApcDisable (SHORT).
/// i386: __bindgen_anon_4.__bindgen_anon_1.KernelApcDisable
/// amd64/arm64: __bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.KernelApcDisable
#[inline(always)]
unsafe fn kthread_kernel_apc_disable(thread: ddk::PKTHREAD) -> *mut i16 {
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.__bindgen_anon_1.KernelApcDisable) }
    #[cfg(not(target_arch = "x86"))]
    { core::ptr::addr_of_mut!((*thread).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.KernelApcDisable) }
}

/// Get pointer to KTHREAD.SpecialApcDisable (SHORT).
#[inline(always)]
unsafe fn kthread_special_apc_disable_ptr(thread: ddk::PKTHREAD) -> *mut i16 {
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable) }
    #[cfg(not(target_arch = "x86"))]
    { core::ptr::addr_of_mut!((*thread).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable) }
}

/// Read KTHREAD.SpecialApcDisable (SHORT).
#[inline(always)]
unsafe fn kthread_special_apc_disable(thread: ddk::PKTHREAD) -> i16 {
    *kthread_special_apc_disable_ptr(thread)
}

// ============================================================================
// KeEnterCriticalRegion (macro from ke_x.h)
// ============================================================================

/// KeEnterCriticalRegion — disable kernel APCs.
/// C: `Thread->KernelApcDisable--;`
#[inline]
pub unsafe fn KeEnterCriticalRegion() {
    let thread = current_thread();
    *kthread_kernel_apc_disable(thread) -= 1;
}

// ============================================================================
// KeLeaveCriticalRegion (macro from ke_x.h)
// ============================================================================

/// KeLeaveCriticalRegion — re-enable kernel APCs and check for delivery.
/// C: `Thread->KernelApcDisable++; if (!KernelApcDisable && !IsListEmpty(KernelMode) && !SpecialApcDisable) KiCheckForKernelApcDelivery()`
#[inline]
pub unsafe fn KeLeaveCriticalRegion() {
    let thread = current_thread();
    let p = kthread_kernel_apc_disable(thread);
    *p += 1;

    if *p == 0 {
        // Check if KernelMode APC list is not empty
        let list_head = &(*thread).__bindgen_anon_1.ApcState.ApcListHead[0]
            as *const ddk::_LIST_ENTRY as *mut ddk::_LIST_ENTRY;
        let is_empty = (*list_head).Flink == list_head;
        if !is_empty && kthread_special_apc_disable(thread) == 0 {
            ddk::KiCheckForKernelApcDelivery();
        }
    }
}

// ============================================================================
// KeQueryInterruptTime (macro on amd64/arm64, real function on i386)
// ============================================================================

/// KeQueryInterruptTime — read InterruptTime from SharedUserData.
/// On i386 this is also a real function, but the inline version works everywhere.
#[inline]
pub unsafe fn KeQueryInterruptTime() -> u64 {
    let shared_interrupt_time =
        (crate::mm::KI_USER_SHARED_DATA + 0x8) as *const u64;
    core::ptr::read_volatile(shared_interrupt_time)
}

// ============================================================================
// KeGetContextSwitches (macro from internal/ke.h, per-arch)
// ============================================================================

/// KeGetContextSwitches — read context switch count from KPRCB.
/// i386: CONTAINING_RECORD(Prcb, KIPCR, PrcbData)->ContextSwitches (in NtTib union).
/// amd64/arm64: Prcb->KeContextSwitches.
#[inline]
pub unsafe fn KeGetContextSwitches(prcb: ddk::PKPRCB) -> ULONG {
    #[cfg(target_arch = "x86")]
    {
        // On i386: CONTAINING_RECORD(Prcb, KIPCR, PrcbData)
        // PrcbData is the last field of KIPCR, so Prcb ptr IS &KIPCR.PrcbData
        let kipcr = crate::inline::common::CONTAINING_RECORD_typed::<ddk::_KIPCR>(
            prcb as PVOID,
            core::mem::offset_of!(ddk::_KIPCR, PrcbData),
        );
        (*kipcr).__bindgen_anon_1.__bindgen_anon_1.ContextSwitches
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*prcb).KeContextSwitches
    }
}

// ============================================================================
// KxAcquireSpinLock / KxReleaseSpinLock (FORCEINLINE from spinlock.h)
// UP kernel with DBG=1: compiler barrier + owner tracking.
// The DBG checks store KeGetCurrentThread()|1 in the lock on acquire and
// validate it on release, catching mismatched acquire/release pairs.
// ============================================================================

const SPIN_LOCK_ALREADY_OWNED: ULONG = 0xF;
const SPIN_LOCK_NOT_OWNED: ULONG = 0x10;

// cswitchdep P2 spinlock taps (cswitchdep-design.md): Kx is the choke point
// every ntoskrnl spinlock funnel delegates to, so a per-CPU held-counter tap
// here covers the whole kernel link in one place. Link-name externs into
// ntoskrnl-rust's cswitchdep module — identical plumbing to the lockdep
// pushlock taps in inline/ex.rs. Zero cost when the feature is off.
#[cfg(feature = "cswitchdep")]
extern "C" {
    fn VibeosCswitchdepSpinAcquire(lock: usize);
    fn VibeosCswitchdepSpinRelease(lock: usize);
}

#[cfg(feature = "cswitchdep")]
#[inline]
pub(crate) unsafe fn cswdep_spin_acquire(lock: *mut ULONG_PTR) {
    VibeosCswitchdepSpinAcquire(lock as usize);
}
#[cfg(not(feature = "cswitchdep"))]
#[inline(always)]
pub(crate) unsafe fn cswdep_spin_acquire(_lock: *mut ULONG_PTR) {}

#[cfg(feature = "cswitchdep")]
#[inline]
pub(crate) unsafe fn cswdep_spin_release(lock: *mut ULONG_PTR) {
    VibeosCswitchdepSpinRelease(lock as usize);
}
#[cfg(not(feature = "cswitchdep"))]
#[inline(always)]
pub(crate) unsafe fn cswdep_spin_release(_lock: *mut ULONG_PTR) {}

/// KxAcquireSpinLock — UP kernel: compiler barrier only. SMP kernel
/// (feature = "smp"): the spinlock.h CONFIG_SMP body — try-acquire via
/// InterlockedBitTestAndSet(bit 0), on contention spin-read until free.
///
/// Under `feature = "dbg"` it also stamps the owner, mirroring
/// `KI_TRACK_SPINLOCK_OWNER` in ntoskrnl/include/internal/spinlock.h. That
/// stamp is OBSERVABLE BEHAVIOUR, not a diagnostic: `KeTestSpinLock` reads it,
/// and so does every DDK caller that inspects a held lock. The C and Rust
/// halves of this funnel MUST agree — a lock stamped by one and not cleared by
/// the other stays dirty forever. Of the two `KeBugCheckEx` validations that
/// upstream pairs with the stamp (`KI_VALIDATE_...`, compiled out here), the
/// release-side SPIN_LOCK_NOT_OWNED one stays OFF on both sides — it is what
/// broke i386 boot when this was first enabled, and finding the mismatched
/// acquire/release pairs is still separate work. The acquire-side
/// SPIN_LOCK_ALREADY_OWNED one is now ON for `smp + dbg`; see the body for why
/// that subset cannot false-positive. It reads the stamp without writing one,
/// so it creates no C-vs-Rust asymmetry and needs no mirror in spinlock.h.
#[inline]
pub unsafe fn KxAcquireSpinLock(spin_lock: *mut ULONG_PTR) {
    cswdep_spin_acquire(spin_lock);
    // Upstream's SPIN_LOCK_ALREADY_OWNED pre-check, which this tree compiles out:
    // `_USE_DUMMY_PSEH=1` is defined globally (sdk/cmake/gcc.cmake), so
    // KI_VALIDATE_SPINLOCK_OWNER is 0 — the stamp below is written and NOTHING
    // ever reads it back. Restored here for the one case where it cannot be a
    // false positive: on an MP kernel, finding our OWN thread stamped in a lock
    // we are about to spin on is a guaranteed deadlock, because the only thread
    // that can clear it is the one that is about to start spinning. The
    // alternative is not a surviving boot, it is an unbounded silent spin at
    // IRQL >= DISPATCH_LEVEL that takes the clock, every timer, and the whole
    // machine down with it — which is precisely the arm64 `--smp 1` stall where
    // a 30s SCM pipe wait sat for 185s and every subsystem went quiet at once.
    //
    // Deliberately paired with `smp`: on the UP kernel the spin below is
    // compiled away entirely, so the same recursive acquire is harmless there
    // and bugchecking on it would invent a failure the UP boot does not have.
    // Deliberately NOT accompanied by upstream's release-side SPIN_LOCK_NOT_OWNED
    // check — that one fires on every benign mismatched acquire/release pair and
    // is what broke i386 boot when validation was first switched on. This takes
    // only the subset that is provably already a hang.
    //
    // This once cited KeTryToAcquireSpinLockAtDpcLevel (ntoskrnl-rust
    // ke/spinlock.rs) as precedent for "carried this check without incident".
    // It was not one: that copy was gated on cargo's `debug_assertions`, which
    // the release profile leaves off, so it had never executed. Both are now on
    // `feature = "dbg"` and genuinely share the rationale above.
    #[cfg(all(feature = "smp", feature = "dbg"))]
    if core::ptr::read_volatile(spin_lock) == (current_thread() as ULONG_PTR | 1) {
        ddk::KeBugCheckEx(SPIN_LOCK_ALREADY_OWNED, spin_lock as ULONG_PTR as _, 0, 0, 0);
    }
    #[cfg(feature = "smp")]
    while InterlockedBitTestAndSet(spin_lock as *mut i32, 0) != 0 {
        // Locked — spin read-only (cache-friendly) until it's released.
        while core::ptr::read_volatile(spin_lock) & 1 != 0 {
            core::hint::spin_loop();
        }
    }
    // Compiler barrier (KeMemoryBarrierWithoutFence); under smp the
    // interlocked op above is the hardware fence.
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    #[cfg(feature = "dbg")]
    {
        *spin_lock = current_thread() as ULONG_PTR | 1;
    }
}

/// KxReleaseSpinLock — UP kernel: compiler barrier only. SMP kernel:
/// spinlock.h's `InterlockedAnd(64)(SpinLock, 0)` — full-barrier atomic clear.
/// Under `feature = "dbg"` the clear also happens on UP, because that is where
/// the owner stamp is undone (see KxAcquireSpinLock).
#[inline]
pub unsafe fn KxReleaseSpinLock(spin_lock: *mut ULONG_PTR) {
    cswdep_spin_release(spin_lock);
    #[cfg(any(feature = "smp", feature = "dbg"))]
    {
        core::sync::atomic::AtomicUsize::from_ptr(spin_lock as *mut usize)
            .swap(0, core::sync::atomic::Ordering::SeqCst);
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

// ============================================================================
// KeGetContextReturnRegister (FORCEINLINE from ke.h per arch)
// ============================================================================

#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeGetContextReturnRegister(ctx: *mut ddk::_CONTEXT) -> ULONG_PTR {
    (*ctx).Eax as ULONG_PTR
}

#[inline]
#[cfg(target_arch = "x86_64")]
pub unsafe fn KeGetContextReturnRegister(ctx: *mut ddk::_CONTEXT) -> ULONG_PTR {
    (*ctx).Rax as ULONG_PTR
}

#[inline]
#[cfg(target_arch = "aarch64")]
pub unsafe fn KeGetContextReturnRegister(ctx: *mut ddk::_CONTEXT) -> ULONG_PTR {
    (*ctx).__bindgen_anon_1.X[0] as ULONG_PTR
}

// ============================================================================
// KeGetExceptionFrame (macro from ke.h per arch)
// ============================================================================

#[inline]
#[cfg(target_arch = "x86")]
pub unsafe fn KeGetExceptionFrame(_thread: ddk::PKTHREAD) -> ddk::PKEXCEPTION_FRAME {
    // i386 has no exception frame
    core::ptr::null_mut()
}

#[inline]
#[cfg(not(target_arch = "x86"))]
pub unsafe fn KeGetExceptionFrame(thread: ddk::PKTHREAD) -> ddk::PKEXCEPTION_FRAME {
    let trap_frame = KeGetTrapFrame(thread);
    (trap_frame as usize - core::mem::size_of::<ddk::_KEXCEPTION_FRAME>())
        as ddk::PKEXCEPTION_FRAME
}

// ============================================================================
// KeEnterGuardedRegion / KeLeaveGuardedRegion (macro from ke_x.h)
// ============================================================================

// KiCheckForKernelApcDelivery is stdcall on i386, cdecl on others
extern "system" {
    fn KiCheckForKernelApcDelivery();
}

/// KeEnterGuardedRegionThread — disable special APCs by decrementing SpecialApcDisable.
#[inline]
pub unsafe fn KeEnterGuardedRegionThread(thread: ddk::PKTHREAD) {
    let p = kthread_special_apc_disable_ptr(thread);
    *p -= 1;
}

/// KeLeaveGuardedRegionThread — re-enable special APCs and check for pending kernel APCs.
#[inline]
pub unsafe fn KeLeaveGuardedRegionThread(thread: ddk::PKTHREAD) {
    let p = kthread_special_apc_disable_ptr(thread);
    *p += 1;
    if *p == 0 {
        let list_head = &(*thread).__bindgen_anon_1.ApcState.ApcListHead[0]
            as *const ddk::_LIST_ENTRY as *mut ddk::_LIST_ENTRY;
        let is_empty = (*list_head).Flink == list_head;
        if !is_empty {
            KiCheckForKernelApcDelivery();
        }
    }
}

/// KeEnterGuardedRegion — get current thread and enter guarded region.
#[inline]
pub unsafe fn KeEnterGuardedRegion() {
    let thread = crate::mm::KeGetCurrentThread() as ddk::PKTHREAD;
    KeEnterGuardedRegionThread(thread);
}

/// KeLeaveGuardedRegion — get current thread and leave guarded region.
#[inline]
pub unsafe fn KeLeaveGuardedRegion() {
    let thread = crate::mm::KeGetCurrentThread() as ddk::PKTHREAD;
    KeLeaveGuardedRegionThread(thread);
}

// ============================================================================
// KeInitializeSpinLock
// ============================================================================

/// KeInitializeSpinLock — set spin lock to zero.
#[inline]
pub unsafe fn KeInitializeSpinLock(spin_lock: *mut ULONG_PTR) {
    *spin_lock = 0;
}

// ============================================================================
// NtCurrentTeb (per-arch)
// ============================================================================

/// NtCurrentTeb — get current TEB pointer (per-arch).
#[inline]
pub unsafe fn NtCurrentTeb() -> PVOID {
    #[cfg(target_arch = "x86")]
    {
        let teb: PVOID;
        core::arch::asm!("mov {}, dword ptr fs:[0x18]", out(reg) teb);
        teb
    }
    #[cfg(target_arch = "x86_64")]
    {
        let teb: PVOID;
        core::arch::asm!("mov {}, qword ptr gs:[0x30]", out(reg) teb);
        teb
    }
    #[cfg(target_arch = "aarch64")]
    {
        let teb: PVOID;
        // Windows ARM64 uses x18 as the platform register that carries the
        // current TEB pointer. TPIDR_EL0 is not the kernel's source of truth
        // for NtCurrentTeb and can hold unrelated values during thread startup.
        core::arch::asm!("mov {}, x18", out(reg) teb, options(nomem, nostack, preserves_flags));
        teb
    }
}

// ============================================================================
// KeQuerySystemTime (macro on amd64/arm64, real function on i386)
// ============================================================================

/// KeQuerySystemTime — read system time from SharedUserData.
/// Only needed on amd64/arm64 where it's a macro.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn KeQuerySystemTime(current_time: *mut ddk::_LARGE_INTEGER) {
    // `SharedUserData` is a MACRO, `((KUSER_SHARED_DATA *const)KI_USER_SHARED_DATA)` -- a
    // fixed address, not a symbol. An `extern static` of that name linked only inside
    // ntoskrnl; atapi.sys, the first driver to call this, failed with an undefined symbol.
    #[allow(non_snake_case)]
    let SharedUserData = crate::mm::KI_USER_SHARED_DATA as *const ddk::_KUSER_SHARED_DATA;
    // SharedUserData is updated asynchronously by the kernel clock tick; these
    // reads must remain actual memory loads for spin-wait callers.
    loop {
        let high1 = core::ptr::addr_of!((*SharedUserData).SystemTime.High1Time).read_volatile();
        let low = core::ptr::addr_of!((*SharedUserData).SystemTime.LowPart).read_volatile();
        let high2 = core::ptr::addr_of!((*SharedUserData).SystemTime.High2Time).read_volatile();
        if high1 == high2 {
            (*current_time).QuadPart = ((high1 as i64) << 32) | (low as u32 as i64);
            break;
        }
    }
}

// ============================================================================
// KeGetCurrentIrql / KeLowerIrql / KeRaiseIrql (amd64 FORCEINLINE: read/write CR8)
// These are only needed on amd64 where they're FORCEINLINE.
// On i386/arm64, HAL provides the real symbols.
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeGetCurrentIrql_amd64() -> KIRQL {
    let cr8: u64;
    core::arch::asm!("mov {}, cr8", out(reg) cr8);
    cr8 as KIRQL
}

// ============================================================================
// Ki386PerfEnd (stub — no-op)
// ============================================================================

/// Ki386PerfEnd — performance counter end (stub, no-op on ReactOS).
#[inline]
pub unsafe fn Ki386PerfEnd() {
    // no-op
}

// ============================================================================
// ExRaiseStatus (calls RtlRaiseStatus)
// ============================================================================

extern "system" {
    fn RtlRaiseStatus(status: NTSTATUS) -> !;
}

/// ExRaiseStatus — raise an exception with the given NTSTATUS.
#[inline]
pub unsafe fn ExRaiseStatus(status: NTSTATUS) -> ! {
    RtlRaiseStatus(status)
}

// ============================================================================
// PspDumpThreads (debug stub — no-op)
// ============================================================================

/// PspDumpThreads — debug-only thread dump (stub).
#[inline]
pub unsafe fn PspDumpThreads(_include_system: BOOLEAN) {
    // no-op stub
}

// ============================================================================
// amd64 KeLowerIrql / KfRaiseIrql (FORCEINLINE, write/read CR8)
// ============================================================================

/// KeLowerIrql — amd64: write CR8 to lower IRQL.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeLowerIrql_amd64(new_irql: KIRQL) {
    core::arch::asm!("mov cr8, {}", in(reg) new_irql as u64);
}

/// KfRaiseIrql — amd64: read CR8 (old IRQL), write CR8 (new IRQL).
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KfRaiseIrql_amd64(new_irql: KIRQL) -> KIRQL {
    let old: u64;
    core::arch::asm!(
        "mov {}, cr8",
        "mov cr8, {}",
        out(reg) old,
        in(reg) new_irql as u64,
    );
    old as KIRQL
}

// ============================================================================
// Constants needed by ke_x.h FORCEINLINE functions
// ============================================================================

/// TIMER_TABLE_SIZE — size of the timer table (sdk/include/xdk/ketypes.h)
pub const TIMER_TABLE_SIZE: u32 = 512;

/// TIMER_WAIT_BLOCK — index of timer wait block in thread's WaitBlock array
pub const TIMER_WAIT_BLOCK: usize = 3;

/// SYNCH_LEVEL — UP kernel: DISPATCH_LEVEL (= 2). SMP kernel: IPI_LEVEL - 2.
/// Every arch's ndk header carries the same `#ifndef CONFIG_SMP` pair (i386
/// asm.h; amd64/arm64 ketypes.h), so all three vary — an earlier note here
/// claiming arm64 "has no SMP variant" was simply wrong, and the 2 it justified
/// disagreed with `ndk/arm64/ketypes.h:29` on the MP kernel.
#[cfg(not(feature = "smp"))]
pub const SYNCH_LEVEL: KIRQL = 2; // DISPATCH_LEVEL
#[cfg(all(feature = "smp", target_arch = "x86"))]
pub const SYNCH_LEVEL: KIRQL = 0x1D - 2; // IPI_LEVEL(0x1D) - 2 = 27
#[cfg(all(feature = "smp", target_arch = "x86_64"))]
pub const SYNCH_LEVEL: KIRQL = 14 - 2; // IPI_LEVEL(14) - 2 = 12
#[cfg(all(feature = "smp", target_arch = "aarch64"))]
pub const SYNCH_LEVEL: KIRQL = 14 - 2; // IPI_LEVEL(14) - 2 = 12

/// TIMER_OR_EVENT_TYPE — mask for dispatcher header type check
pub const TIMER_OR_EVENT_TYPE: i32 = 0x7;

/// GM_LOCK_BIT — guarded mutex lock bit (bit 0)
pub const GM_LOCK_BIT: i32 = 0x1;
/// GM_LOCK_BIT_V — bit number for InterlockedBitTestAndReset
pub const GM_LOCK_BIT_V: i32 = 0x0;
/// GM_LOCK_WAITER_WOKEN — bit 1, waiter has been woken
pub const GM_LOCK_WAITER_WOKEN: i32 = 0x2;
/// GM_LOCK_WAITER_INC — increment per waiter (bit 2)
pub const GM_LOCK_WAITER_INC: i32 = 0x4;

// ============================================================================
// Extern declarations for globals NOT in bindgen
// ============================================================================

extern "C" {
    /// KeMaximumIncrement — maximum timer increment (100ns units).
    pub static KeMaximumIncrement: ULONG;

    /// KiTimerTableListHead — global timer table (KTIMER_TABLE_ENTRY[TIMER_TABLE_SIZE]).
    pub static mut KiTimerTableListHead: [ddk::_KTIMER_TABLE_ENTRY; TIMER_TABLE_SIZE as usize];

    /// KeTickCount — system tick count (KSYSTEM_TIME).
    pub static KeTickCount: ddk::_KSYSTEM_TIME;

    /// KiNmiCallbackListLock — spin lock for NMI callback list.
    pub static mut KiNmiCallbackListLock: ULONG_PTR;
}

// ============================================================================
// Extern declarations for functions used by ke_x.h inlines
// ============================================================================

// PsGetCurrentThread is stdcall on i386, cdecl on others
extern "system" {
    fn PsGetCurrentThread() -> ddk::PETHREAD;
}

// Ki*/Ke* internal functions are fastcall on i386, cdecl on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiExitDispatcher(OldIrql: KIRQL);
    fn KiDeferredReadyThread(Thread: ddk::PKTHREAD);
    fn KiInsertTimerTable(Timer: ddk::PKTIMER, Hand: ULONG) -> BOOLEAN;
    fn KiCompleteTimer(Timer: ddk::PKTIMER, LockQueue: ddk::PKSPIN_LOCK_QUEUE);
    fn KiUnwaitThread(Thread: ddk::PKTHREAD, WaitStatus: LONG_PTR, Increment: KPRIORITY);
    fn KiAcquireGuardedMutex(GuardedMutex: ddk::PKGUARDED_MUTEX);
    fn KeInitializeGate(Gate: ddk::PKGATE);
    fn KeSignalGateBoostPriority(Gate: ddk::PKGATE);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiExitDispatcher(OldIrql: KIRQL);
    fn KiDeferredReadyThread(Thread: ddk::PKTHREAD);
    fn KiInsertTimerTable(Timer: ddk::PKTIMER, Hand: ULONG) -> BOOLEAN;
    fn KiCompleteTimer(Timer: ddk::PKTIMER, LockQueue: ddk::PKSPIN_LOCK_QUEUE);
    fn KiUnwaitThread(Thread: ddk::PKTHREAD, WaitStatus: LONG_PTR, Increment: KPRIORITY);
    fn KiAcquireGuardedMutex(GuardedMutex: ddk::PKGUARDED_MUTEX);
    fn KeInitializeGate(Gate: ddk::PKGATE);
    fn KeSignalGateBoostPriority(Gate: ddk::PKGATE);
}

// KfRaiseIrql is fastcall on i386, cdecl on amd64/arm64
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}

// InStackQueuedSpinLock functions are fastcall on i386, cdecl on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
        SpinLock: *mut ULONG_PTR,
        LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLockAtDpcLevel(
        SpinLock: *mut ULONG_PTR,
        LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLock(
        SpinLock: *mut ULONG_PTR,
        LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
    );
    fn KeReleaseInStackQueuedSpinLock(LockHandle: ddk::PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLockFromDpcLevel(LockHandle: ddk::PKLOCK_QUEUE_HANDLE);
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
        SpinLock: *mut ULONG_PTR,
        LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLockAtDpcLevel(
        SpinLock: *mut ULONG_PTR,
        LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLock(
        SpinLock: *mut ULONG_PTR,
        LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
    );
    fn KeReleaseInStackQueuedSpinLock(LockHandle: ddk::PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLockFromDpcLevel(LockHandle: ddk::PKLOCK_QUEUE_HANDLE);
}

// ============================================================================
// Helper accessors for bindgen anonymous unions
// These abstract the deeply nested __bindgen_anon_N paths that differ per arch.
// ============================================================================

/// DISPATCHER_HEADER.Hand (all arches: same nesting).
#[inline]
unsafe fn dh_get_hand(header: *const ddk::_DISPATCHER_HEADER) -> u8 {
    (*header).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Hand
}
#[inline]
unsafe fn dh_set_hand(header: *mut ddk::_DISPATCHER_HEADER, val: u8) {
    (*header).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Hand = val;
}

/// DISPATCHER_HEADER.Type (all arches: same nesting).
#[inline]
unsafe fn dh_get_type(header: *const ddk::_DISPATCHER_HEADER) -> u8 {
    (*header).__bindgen_anon_1.__bindgen_anon_1.Type
}

/// DISPATCHER_HEADER.Absolute bitfield (all arches: 5-level nesting).
#[inline]
unsafe fn dh_set_absolute(header: *mut ddk::_DISPATCHER_HEADER, val: u8) {
    (*header).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
        .__bindgen_anon_1.__bindgen_anon_1.set_Absolute(val);
}

/// DISPATCHER_HEADER.Inserted field (all arches: 3-level nesting).
#[inline]
unsafe fn dh_set_inserted(header: *mut ddk::_DISPATCHER_HEADER, val: u8) {
    (*header).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = val;
}

/// KTHREAD.State reader (mirror of kt_set_state).
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn kt_state_read(thread: ddk::PKTHREAD) -> u8 {
    #[cfg(target_arch = "x86")]
    { (*thread).State }
    #[cfg(not(target_arch = "x86"))]
    { (*thread).__bindgen_anon_5.__bindgen_anon_6.State }
}

/// KTHREAD.State — i386: direct field; amd64/arm64: nested.
#[inline]
unsafe fn kt_set_state(thread: ddk::PKTHREAD, val: u8) {
    #[cfg(target_arch = "x86")]
    { (*thread).State = val; }
    #[cfg(not(target_arch = "x86"))]
    { (*thread).__bindgen_anon_5.__bindgen_anon_6.State = val; }
}

/// KTHREAD.DeferredProcessor — all arches: nested in __bindgen_anon_1.
#[inline]
unsafe fn kt_set_deferred_processor(thread: ddk::PKTHREAD, val: u8) {
    (*thread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor = val;
}

/// KTHREAD.Timer pointer — i386: __bindgen_anon_5; amd64/arm64: __bindgen_anon_4.
#[inline]
unsafe fn kt_timer(thread: ddk::PKTHREAD) -> *mut ddk::_KTIMER {
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*thread).__bindgen_anon_5.Timer) }
    #[cfg(not(target_arch = "x86"))]
    { core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.Timer) }
}

/// KTHREAD.WaitBlock[index] pointer — i386: __bindgen_anon_7; amd64/arm64: __bindgen_anon_5.
#[inline]
unsafe fn kt_wait_block(thread: ddk::PKTHREAD, index: usize) -> *mut ddk::_KWAIT_BLOCK {
    #[cfg(target_arch = "x86")]
    { (*thread).__bindgen_anon_7.WaitBlock.as_mut_ptr().add(index) }
    #[cfg(not(target_arch = "x86"))]
    { (*thread).__bindgen_anon_5.WaitBlock.as_mut_ptr().add(index) }
}

/// KTHREAD.WaitBlockList setter — all arches: __bindgen_anon_2.
#[inline]
unsafe fn kt_set_wait_block_list(thread: ddk::PKTHREAD, val: *mut ddk::_KWAIT_BLOCK) {
    (*thread).__bindgen_anon_2.WaitBlockList = val;
}

/// KTHREAD.WaitListEntry pointer — all arches: __bindgen_anon_3.
#[inline]
unsafe fn kt_wait_list_entry(thread: ddk::PKTHREAD) -> *mut ddk::_LIST_ENTRY {
    core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.WaitListEntry)
}

/// KTHREAD.WaitMode setter — i386: direct; amd64/arm64: nested.
#[inline]
unsafe fn kt_set_wait_mode(thread: ddk::PKTHREAD, val: i8) {
    #[cfg(target_arch = "x86")]
    { (*thread).WaitMode = val; }
    #[cfg(not(target_arch = "x86"))]
    { (*thread).__bindgen_anon_5.__bindgen_anon_6.WaitMode = val; }
}

/// KTHREAD.WaitTime setter — i386: direct; amd64/arm64: nested.
#[inline]
unsafe fn kt_set_wait_time(thread: ddk::PKTHREAD, val: u32) {
    #[cfg(target_arch = "x86")]
    { (*thread).WaitTime = val; }
    #[cfg(not(target_arch = "x86"))]
    { (*thread).__bindgen_anon_5.__bindgen_anon_7.WaitTime = val; }
}

/// KPRCB.DpcThreadActive — i386/amd64: direct UCHAR; arm64: bitfield getter.
#[inline]
unsafe fn prcb_dpc_thread_active(prcb: ddk::PKPRCB) -> bool {
    #[cfg(target_arch = "aarch64")]
    { (*prcb).__bindgen_anon_2.__bindgen_anon_2.DpcThreadActive() != 0 }
    #[cfg(not(target_arch = "aarch64"))]
    { (*prcb).DpcThreadActive != 0 }
}

/// KTRAP_FRAME.TrapFrame getter — i386: Edx; amd64: direct; arm64: nested.
#[inline]
unsafe fn tf_get_linked(trap_frame: ddk::PKTRAP_FRAME) -> ddk::PKTRAP_FRAME {
    #[cfg(target_arch = "x86")]
    { (*trap_frame).Edx as ddk::PKTRAP_FRAME }
    #[cfg(target_arch = "x86_64")]
    { (*trap_frame).TrapFrame as ddk::PKTRAP_FRAME }
    #[cfg(target_arch = "aarch64")]
    { (*trap_frame).__bindgen_anon_2.__bindgen_anon_1.TrapFrame as ddk::PKTRAP_FRAME }
}

/// Portable KeGetCurrentThread — works on all arches.
#[inline]
unsafe fn KeGetCurrentThread_portable() -> ddk::PKTHREAD {
    PsGetCurrentThread() as ddk::PKTHREAD
}

/// Offset of WaitListEntry within _KTHREAD (it's in __bindgen_anon_3 union at offset 0).
#[inline]
fn memoffset_of_wait_list_entry() -> usize {
    core::mem::offset_of!(ddk::_KTHREAD, __bindgen_anon_3)
}

// ============================================================================
// ke_x.h lock functions — UP (#ifndef CONFIG_SMP) no-ops vs SMP (feature =
// "smp") real bodies, translated from ke_x.h lines 305-603. The SMP simple
// spinlocks (thread/PRCB/dispatcher-object) mirror the C's 32-bit
// InterlockedExchange/And on the low DWORD of the (pointer-sized) lock field.
// ============================================================================

#[cfg(feature = "smp")]
const KOBJECT_LOCK_BIT: i32 = 0x80; // ndk/ketypes.h
#[cfg(feature = "smp")]
const LOCK_QUEUE_TIMER_LOCK_SHIFT: ULONG = 4; // xdk/ketypes.h
#[cfg(feature = "smp")]
const LOCK_QUEUE_TIMER_TABLE_LOCKS: ULONG = 1 << (8 - LOCK_QUEUE_TIMER_LOCK_SHIFT);
#[cfg(feature = "smp")]
const LOCK_QUEUE_DISPATCHER_LOCK: usize = 0; // LockQueueDispatcherLock
#[cfg(feature = "smp")]
const LOCK_QUEUE_TIMER_TABLE_LOCK: usize = 17; // LockQueueTimerTableLock
#[cfg(feature = "smp")]
const IPI_APC: ULONG = 1; // ndk/<arch>/ketypes.h
#[cfg(feature = "smp")]
const IPI_DPC: ULONG = 2;

/// SMP simple-spinlock acquire on a lock word, using 32-bit interlocked ops on
/// the low DWORD exactly like the C (`InterlockedExchange((PLONG)&Lock, 1)`).
#[cfg(feature = "smp")]
#[inline]
unsafe fn smp_simple_lock_acquire(lock: *mut i32) {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(lock as *const AtomicI32);
    loop {
        if atomic.swap(1, Ordering::SeqCst) == 0 {
            break;
        }
        while atomic.load(Ordering::Relaxed) != 0 {
            core::hint::spin_loop();
        }
    }
}

/// SMP simple-spinlock release (`InterlockedAnd((PLONG)&Lock, 0)`).
#[cfg(feature = "smp")]
#[inline]
unsafe fn smp_simple_lock_release(lock: *mut i32) {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(lock as *const AtomicI32)).swap(0, Ordering::SeqCst);
}

/// KiAcquireThreadLock — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiAcquireThreadLock(_thread: ddk::PKTHREAD) {}

/// KiAcquireThreadLock — SMP: spin on Thread->ThreadLock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiAcquireThreadLock(thread: ddk::PKTHREAD) {
    smp_simple_lock_acquire(core::ptr::addr_of_mut!((*thread).ThreadLock) as *mut i32);
}

/// KiReleaseThreadLock — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiReleaseThreadLock(_thread: ddk::PKTHREAD) {}

/// KiReleaseThreadLock — SMP: release Thread->ThreadLock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiReleaseThreadLock(thread: ddk::PKTHREAD) {
    smp_simple_lock_release(core::ptr::addr_of_mut!((*thread).ThreadLock) as *mut i32);
}

/// KiAcquireDispatcherObject — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiAcquireDispatcherObject(_object: *mut ddk::_DISPATCHER_HEADER) {}

/// KiAcquireDispatcherObject — SMP: CAS-set KOBJECT_LOCK_BIT in Header.Lock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiAcquireDispatcherObject(object: *mut ddk::_DISPATCHER_HEADER) {
    use core::sync::atomic::{AtomicI32, Ordering};
    let lock = &*(core::ptr::addr_of_mut!((*object).__bindgen_anon_1.Lock) as *const AtomicI32);
    loop {
        let old = loop {
            let v = lock.load(Ordering::Relaxed);
            if v & KOBJECT_LOCK_BIT == 0 {
                break v;
            }
            core::hint::spin_loop();
        };
        if lock
            .compare_exchange(old, old | KOBJECT_LOCK_BIT, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
}

/// KiReleaseDispatcherObject — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiReleaseDispatcherObject(_object: *mut ddk::_DISPATCHER_HEADER) {}

/// KiReleaseDispatcherObject — SMP: InterlockedAnd(&Header.Lock, ~KOBJECT_LOCK_BIT).
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiReleaseDispatcherObject(object: *mut ddk::_DISPATCHER_HEADER) {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(core::ptr::addr_of_mut!((*object).__bindgen_anon_1.Lock) as *const AtomicI32))
        .fetch_and(!KOBJECT_LOCK_BIT, Ordering::SeqCst);
}

/// KiAcquireDispatcherLockAtSynchLevel — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiAcquireDispatcherLockAtSynchLevel() {}

/// KiAcquireDispatcherLockAtSynchLevel — SMP: queued dispatcher lock at DPC level.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiAcquireDispatcherLockAtSynchLevel() {
    ddk::KeAcquireQueuedSpinLockAtDpcLevel(core::ptr::addr_of_mut!(
        (*KeGetCurrentPrcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK]
    ));
}

/// KiReleaseDispatcherLockFromSynchLevel — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiReleaseDispatcherLockFromSynchLevel() {}

/// KiReleaseDispatcherLockFromSynchLevel — SMP: release queued dispatcher lock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiReleaseDispatcherLockFromSynchLevel() {
    ddk::KeReleaseQueuedSpinLockFromDpcLevel(core::ptr::addr_of_mut!(
        (*KeGetCurrentPrcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK]
    ));
}

/// KiRescheduleThread — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiRescheduleThread(_new_thread: BOOLEAN, _cpu: ULONG) {}

/// KiRescheduleThread — SMP: IPI the target CPU if a new thread needs it.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiRescheduleThread(new_thread: BOOLEAN, cpu: ULONG) {
    if new_thread != 0 && (*KeGetCurrentPrcb()).Number as ULONG != cpu {
        ddk::KiIpiSend(1 << cpu, IPI_DPC);
    }
}

/// KiSetThreadSwapBusy — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiSetThreadSwapBusy(_thread: ddk::PKTHREAD) {}

/// KiSetThreadSwapBusy — SMP: mark the thread as being swapped.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiSetThreadSwapBusy(thread: ddk::PKTHREAD) {
    (*thread).SwapBusy = 1;
}

/// KiAcquirePrcbLock — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiAcquirePrcbLock(_prcb: ddk::PKPRCB) {}

/// KiAcquirePrcbLock — SMP: spin on Prcb->PrcbLock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiAcquirePrcbLock(prcb: ddk::PKPRCB) {
    smp_simple_lock_acquire(core::ptr::addr_of_mut!((*prcb).PrcbLock) as *mut i32);
}

/// KiReleasePrcbLock — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiReleasePrcbLock(_prcb: ddk::PKPRCB) {}

/// KiReleasePrcbLock — SMP: release Prcb->PrcbLock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiReleasePrcbLock(prcb: ddk::PKPRCB) {
    smp_simple_lock_release(core::ptr::addr_of_mut!((*prcb).PrcbLock) as *mut i32);
}

/// KiTryThreadLock — UP: always returns FALSE (lock never held).
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiTryThreadLock(_thread: ddk::PKTHREAD) -> BOOLEAN {
    0 // FALSE
}

/// KiTryThreadLock — SMP: try-ACQUIRE Thread->ThreadLock. Returns FALSE(0) on
/// success (lock now held by us), TRUE(1) on failure (someone else holds it) —
/// the caller contract is `if KiTryThreadLock(t) { retry }` (gate.rs signal).
///
/// §9.56 ROOT-CAUSE FIX (2026-07-13): the previous shape — faithfully copied
/// from ReactOS ke_x.h:533 — returned FALSE ("success") WITHOUT ACQUIRING when
/// the lock read free (`if (!Thread->ThreadLock) return FALSE;`), an upstream
/// ReactOS SMP bug that was dead code there (ReactOS never shipped SMP) and
/// came alive at -smp 2 here: KeSignalGateBoostPriority proceeded lockless on
/// the common path, popping + readying a gate waiter mid-commit → early/double
/// ready → two CPUs InsertTailList the same KTHREAD WaitListEntry union into
/// two lists concurrently → the DispatcherReadyListHead tears (site 0x11/0x22
/// forensics: self-looped / cross-linked just-inserted entry). NT's primitive
/// is KeTryToAcquireSpinLockAtDpcLevel — a real interlocked try-acquire.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiTryThreadLock(thread: ddk::PKTHREAD) -> BOOLEAN {
    use core::sync::atomic::{AtomicI32, Ordering};
    let lock = core::ptr::addr_of_mut!((*thread).ThreadLock) as *mut i32;
    // Fast-fail without dirtying the cache line if visibly held.
    if core::ptr::read_volatile(lock) != 0 {
        return 1; // TRUE = failed, caller retries
    }
    // Actually acquire: old != 0 means we lost the race — failure.
    ((&*(lock as *const AtomicI32)).swap(1, Ordering::SeqCst) != 0) as BOOLEAN
}

/// KiCheckDeferredReadyList — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiCheckDeferredReadyList(_prcb: ddk::PKPRCB) {}

/// KiCheckDeferredReadyList — SMP: drain the PRCB deferred-ready list.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiCheckDeferredReadyList(prcb: ddk::PKPRCB) {
    if !(*prcb).DeferredReadyListHead.Next.is_null() {
        ddk::KiProcessDeferredReadyList(prcb);
    }
}

/// KiRequestApcInterrupt — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiRequestApcInterrupt(_need_apc: BOOLEAN, _processor: UCHAR) {}

/// KiRequestApcInterrupt — SMP: IPI the target CPU (or soft-int self).
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiRequestApcInterrupt(need_apc: BOOLEAN, processor: UCHAR) {
    if need_apc != 0 {
        if (*KeGetCurrentPrcb()).Number as UCHAR != processor {
            ddk::KiIpiSend(1 << processor, IPI_APC);
        } else {
            ddk::HalRequestSoftwareInterrupt(APC_LEVEL);
        }
    }
}

/// KiAcquireTimerLock — UP: returns NULL.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiAcquireTimerLock(_hand: ULONG) -> ddk::PKSPIN_LOCK_QUEUE {
    core::ptr::null_mut()
}

/// KiAcquireTimerLock — SMP: acquire the hashed timer-table queued lock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiAcquireTimerLock(hand: ULONG) -> ddk::PKSPIN_LOCK_QUEUE {
    let lock_index =
        ((hand >> LOCK_QUEUE_TIMER_LOCK_SHIFT) & (LOCK_QUEUE_TIMER_TABLE_LOCKS - 1)) as usize;
    let lock_queue = core::ptr::addr_of_mut!(
        (*KeGetCurrentPrcb()).LockQueue[LOCK_QUEUE_TIMER_TABLE_LOCK + lock_index]
    );
    ddk::KeAcquireQueuedSpinLockAtDpcLevel(lock_queue);
    lock_queue
}

/// KiReleaseTimerLock — UP: no-op.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiReleaseTimerLock(_lock_queue: ddk::PKSPIN_LOCK_QUEUE) {}

/// KiReleaseTimerLock — SMP: release the timer-table queued lock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiReleaseTimerLock(lock_queue: ddk::PKSPIN_LOCK_QUEUE) {
    ddk::KeReleaseQueuedSpinLockFromDpcLevel(lock_queue);
}

// ============================================================================
// Dispatcher lock — simple wrappers (ke_x.h UP vs SMP)
// ============================================================================

/// KiAcquireDispatcherLock — UP: raise IRQL to SYNCH_LEVEL.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiAcquireDispatcherLock() -> KIRQL {
    KfRaiseIrql(SYNCH_LEVEL)
}

/// KiAcquireDispatcherLock — SMP: raise to synch + take the queued dispatcher lock.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiAcquireDispatcherLock() -> KIRQL {
    ddk::KeAcquireQueuedSpinLockRaiseToSynch(
        LOCK_QUEUE_DISPATCHER_LOCK as ddk::KSPIN_LOCK_QUEUE_NUMBER,
    )
}

/// KiReleaseDispatcherLock — UP: call KiExitDispatcher.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiReleaseDispatcherLock(old_irql: KIRQL) {
    KiExitDispatcher(old_irql);
}

/// KiReleaseDispatcherLock — SMP: drop the queued lock, then exit the dispatcher.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiReleaseDispatcherLock(old_irql: KIRQL) {
    ddk::KeReleaseQueuedSpinLockFromDpcLevel(core::ptr::addr_of_mut!(
        (*KeGetCurrentPrcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK]
    ));
    KiExitDispatcher(old_irql);
}

// ============================================================================
// KiInsertDeferredReadyList (ke_x.h UP vs SMP)
// ============================================================================

/// KiInsertDeferredReadyList — UP: set DeferredReady state and call KiDeferredReadyThread.
#[cfg(not(feature = "smp"))]
#[inline]
pub unsafe fn KiInsertDeferredReadyList(thread: ddk::PKTHREAD) {
    kt_set_state(thread, ddk::_KTHREAD_STATE_DeferredReady as u8);
    kt_set_deferred_processor(thread, 0);
    KiDeferredReadyThread(thread);
}

/// §9.56 double-DEFER oracle (0x59). The 0x58 linkage check is blind to a
/// double-defer: BOTH pushes happen before either drain insert, so the thread
/// is unlinked at both checks — yet one KTHREAD SwapListEntry.Next on two
/// deferred SLISTs cross-routes the chains, and the two drain inserts then
/// tear the ready list (the sequential self-loop face). Catch it AT PUSH TIME
/// with an in-flight side table: mark on push, unmark at the drain pop.
/// Lossy under collision pressure (8-probe open addressing) — a tripwire,
/// not a lock.
/// §9.56 oracle arming (2026-07-14): `true` = the 15 scheduler-protocol
/// tripwires (tags 0x54–0x63) BUGCHECK at the catch; `false` = print-only
/// ([RQ*] lines stay, cheap, always-on). De-armed after the fix set
/// (gate trylock + timer-block init + in-flight dedupe + idle NextThread
/// lock v2) took the amplified burst to 29/32 with zero oracle hits.
/// Corruption FINGERPRINT bugchecks (torn ready-list links, null queue
/// entries, ReadySummary inconsistency) are NOT gated — real corruption
/// still stops the machine. Re-arm by flipping this for a debug burst.
pub const RQ_ORACLE_ARMED: bool = false;

#[cfg(feature = "smp")]
static RQ_INFLIGHT: [core::sync::atomic::AtomicUsize; 512] =
    [const { core::sync::atomic::AtomicUsize::new(0) }; 512];

/// Waker identity per in-flight slot: the caller line of the defer that
/// marked it (Requeue sites use 9000+site sentinels). Read back at the
/// 0x5A/0x5D catch points as `deferred_by` — names the mystery waker.
#[cfg(feature = "smp")]
static RQ_WAKER_LINE: [core::sync::atomic::AtomicU32; 512] =
    [const { core::sync::atomic::AtomicU32::new(0) }; 512];

/// Look up who deferred this thread (0 = not marked / unknown).
#[cfg(feature = "smp")]
pub unsafe fn rq_inflight_waker(thread: ddk::PKTHREAD) -> u32 {
    use core::sync::atomic::Ordering::SeqCst;
    let t = thread as usize;
    for i in 0..8 {
        let idx = rq_inflight_slot(t, i);
        if RQ_INFLIGHT[idx].load(SeqCst) == t {
            return RQ_WAKER_LINE[idx].load(SeqCst);
        }
    }
    0
}

/// Public stamping entry for out-of-crate direct-defer sites.
#[cfg(feature = "smp")]
pub unsafe fn rq_waker_stamp_pub(thread: ddk::PKTHREAD, line: u32) {
    rq_waker_stamp(thread, line);
}

#[cfg(feature = "smp")]
unsafe fn rq_waker_stamp(thread: ddk::PKTHREAD, line: u32) {
    use core::sync::atomic::Ordering::SeqCst;
    let t = thread as usize;
    for i in 0..8 {
        let idx = rq_inflight_slot(t, i);
        if RQ_INFLIGHT[idx].load(SeqCst) == t {
            RQ_WAKER_LINE[idx].store(line, SeqCst);
            return;
        }
    }
}

#[cfg(feature = "smp")]
#[inline]
fn rq_inflight_slot(t: usize, i: usize) -> usize {
    ((t >> 7).wrapping_add(i)) & 511
}

/// Returns true if the thread was ALREADY marked in-flight (double-defer).
#[cfg(feature = "smp")]
pub unsafe fn rq_inflight_mark(thread: ddk::PKTHREAD) -> bool {
    use core::sync::atomic::Ordering::SeqCst;
    let t = thread as usize;
    // Full scan for an existing entry BEFORE inserting. Inserting at the first
    // empty slot mid-scan creates DUPLICATE entries when an earlier-probe slot
    // frees up between the original mark and a legal re-mark (Requeue): unmark
    // then clears one twin and the stale twin fires a false 0x59 on the next
    // legal wake with the old stamp. (2026-07-13: this was the 11/32 flood —
    // all victims were boost-requeued ExWorker threads.)
    let mut empty: Option<usize> = None;
    for i in 0..8 {
        let idx = rq_inflight_slot(t, i);
        let cur = RQ_INFLIGHT[idx].load(SeqCst);
        if cur == t {
            return true;
        }
        if cur == 0 && empty.is_none() {
            empty = Some(idx);
        }
    }
    if let Some(idx) = empty {
        if RQ_INFLIGHT[idx].compare_exchange(0, t, SeqCst, SeqCst).is_ok() {
            return false;
        }
        // Lost the slot race — one bounded retry over the whole window.
        for i in 0..8 {
            let idx = rq_inflight_slot(t, i);
            let cur = RQ_INFLIGHT[idx].load(SeqCst);
            if cur == t {
                return true;
            }
            if cur == 0 && RQ_INFLIGHT[idx].compare_exchange(0, t, SeqCst, SeqCst).is_ok() {
                return false;
            }
        }
    }
    false // table pressure — stay lossy rather than false-positive
}

#[cfg(feature = "smp")]
pub unsafe fn rq_inflight_unmark(thread: ddk::PKTHREAD) {
    use core::sync::atomic::Ordering::SeqCst;
    let t = thread as usize;
    // Clear ALL matches (no early return): defensive against duplicate twins
    // from the pre-fix mark() — see the duplicate-entry note in mark().
    for i in 0..8 {
        let slot = &RQ_INFLIGHT[rq_inflight_slot(t, i)];
        if slot.load(SeqCst) == t {
            slot.store(0, SeqCst);
        }
    }
}

/// §9.56 requeue variant — for the LEGAL re-defer sites that reposition a
/// thread already in ready-flight (KiSetPriorityThread / affinity moves /
/// standby displacement: remove-or-displace then re-defer). Skips the 0x59
/// in-flight collision check (the thread is SUPPOSED to still be marked) and
/// keeps the mark live — it is still in flight until promoted to Running.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn KiInsertDeferredReadyListRequeue(thread: ddk::PKTHREAD) {
    let prcb = KeGetCurrentPrcb();
    let _ = rq_inflight_mark(thread); // ensure marked; collision expected/legal
    rq_waker_stamp(thread, 9000);
    kt_set_state(thread, ddk::_KTHREAD_STATE_DeferredReady as u8);
    kt_set_deferred_processor(thread, (*prcb).Number as u8);
    (*thread).__bindgen_anon_3.SwapListEntry.Next = (*prcb).DeferredReadyListHead.Next;
    (*prcb).DeferredReadyListHead.Next =
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.SwapListEntry);
}

/// KiInsertDeferredReadyList — SMP: push onto this CPU's deferred-ready list.
#[cfg(feature = "smp")]
#[inline]
#[track_caller]
pub unsafe fn KiInsertDeferredReadyList(thread: ddk::PKTHREAD) {
    let prcb = KeGetCurrentPrcb();

    // §9.56 double-defer oracle (0x59) — LIFECYCLE v2 (2026-07-13): the mark
    // now lives from WAKE (this push) to PROMOTION-to-Running (unmark at the
    // 6 promote sites), not to drain-pop. v1 (unmark at pop) was blind to the
    // observed sequential double: wake→pop→[2nd wake lands here]→insert→…
    // A collision here = a second wake of a thread already in ready-flight —
    // the true §9.56 first-illegal event; track_caller names the waker.
    let __waker_line = core::panic::Location::caller().line() as u32;
    // Read the STALE stamp first: on collision it names the waker whose mark
    // never got a promote-unmark — i.e. either the missing promotion site or
    // the true first wake of a double-wake pair. (mark() never touches the
    // stamp array, so order vs mark() is cosmetic; explicit is clearer.)
    let __stale_by = rq_inflight_waker(thread);
    if rq_inflight_mark(thread) {
        let caller_line = core::panic::Location::caller().line() as u64;
        crate::driver_ddk::DbgPrint(
            b"[RQDEFER2] DOUBLE-DEFER thr=%p state=%u waitreason=%u waitstatus=%p defproc=%u caller_line=%u stale_by=%u\n\0".as_ptr() as *const i8,
            thread,
            kt_state_read(thread) as u32,
            (*thread).WaitReason as u32,
            (*thread).WaitStatus as usize as *const u8,
            (*thread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor as u32,
            caller_line as u32,
            __stale_by,
        );
        if RQ_ORACLE_ARMED {
        ddk::KeBugCheckEx(
            crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
            thread as usize as _,
            ((0x59u64 << 40) | (caller_line << 8) | kt_state_read(thread) as u64) as _,
            __stale_by as u64 as _,
            caller_line as _,
        );
        }
    }

    // §9.56 hypothesis-B ROOT: deferring a thread that is CurrentThread (running)
    // anywhere is the FIRST illegal readying (it sets State=DeferredReady + pushes to
    // the deferred list; the drain later links it into the ready list while it still
    // runs). #[track_caller] propagates (this fn is #[track_caller]) so the reported
    // line is the READIER that called KiInsertDeferredReadyList on a running thread.
    ki_check_not_running(thread, core::panic::Location::caller().line() as u64);

    // §9.56 tripwire (0x53) — THE UNION-ALIAS TEAR.
    // SwapListEntry.Next below shares memory with WaitListEntry.Flink (KTHREAD
    // __bindgen_anon_3 is a union — same as NT). So deferring a thread that is
    // STILL LINKED in some Prcb's DispatcherReadyListHead CLOBBERS that thread's
    // ready-list Flink, tearing a list we do NOT hold the lock for — which is
    // exactly how site 0x11 fires while its own PRCB lock IS held. NT permits the
    // aliasing only because a Ready thread is always REMOVED from its ready list
    // before being re-readied; that invariant is what we test here.
    //
    // NOTE: `State == Ready` alone is NOT the invariant — the legitimate
    // remove-then-defer paths (KiSetPriorityThread / KiUpdateEffectiveAffinity)
    // call RemoveEntryList first and only *then* defer, so State is still Ready
    // while the thread is already UNLINKED. (A state-only check false-positived on
    // every boot — that's how we learned this.) Test ACTUAL LINKAGE instead: a
    // node still in a doubly-linked list has neighbours that point back at it.
    // After a correct RemoveEntryList the stale Flink/Blink point at old
    // neighbours which now point to each other, so the back-check fails => quiet.
    // §9.56 (2026-07-13): the state gate above was the 0x53 oracle's HOLE — the
    // gate signal path (KeSignalGateBoostPriority) sets State=DeferredReady
    // ITSELF before calling KiReadyThread, so a still-LINKED gate-woken victim
    // arrived here with state 7 and sailed past a Ready-only check (that's how
    // the elimination round wrongly "refuted" still-linked). Run the linkage
    // check for ALL states (tag 0x58). Legit paths stay quiet: a correctly
    // removed node's stale neighbours no longer back-point at it.
    {
        let entry = kt_wait_list_entry(thread);
        let flink = (*entry).Flink;
        let blink = (*entry).Blink;
        let still_linked = !flink.is_null()
            && !blink.is_null()
            && flink != entry // self-loop (InitializeListHead) == not in a list
            && (*flink).Blink == entry
            && (*blink).Flink == entry;
        if still_linked {
            let caller_line = core::panic::Location::caller().line() as u64;
            crate::driver_ddk::DbgPrint(
                b"[RQDOUBLE] deferring a STILL-LINKED thread thr=%p state=%u waitreason=%u waitstatus=%p defproc=%u caller_line=%u\n\0".as_ptr() as *const i8,
                thread,
                kt_state_read(thread) as u32,
                (*thread).WaitReason as u32,
                (*thread).WaitStatus as usize as *const u8,
                (*thread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor as u32,
                caller_line as u32,
            );
            if RQ_ORACLE_ARMED {
            ddk::KeBugCheckEx(
                crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
                thread as usize as _,
                ((0x58u64 << 40) | (caller_line << 8) | kt_state_read(thread) as u64) as _,
                prcb as usize as _,
                caller_line as _,
            );
            }
        }
    }

    rq_waker_stamp(thread, __waker_line);
    kt_set_state(thread, ddk::_KTHREAD_STATE_DeferredReady as u8);
    kt_set_deferred_processor(thread, (*prcb).Number as u8);
    // PushEntryList(&Prcb->DeferredReadyListHead, &Thread->SwapListEntry)
    // (SwapListEntry shares the __bindgen_anon_3 union with WaitListEntry)
    (*thread).__bindgen_anon_3.SwapListEntry.Next = (*prcb).DeferredReadyListHead.Next;
    (*prcb).DeferredReadyListHead.Next =
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.SwapListEntry);
}

// ============================================================================
// APC lock functions (ke_x.h — call InStackQueuedSpinLock variants)
// ============================================================================

/// KiAcquireApcLockRaiseToSynch — acquire thread's APC queue lock, raising to SYNCH_LEVEL.
#[inline]
pub unsafe fn KiAcquireApcLockRaiseToSynch(
    thread: ddk::PKTHREAD,
    handle: ddk::PKLOCK_QUEUE_HANDLE,
) {
    KeAcquireInStackQueuedSpinLockRaiseToSynch(
        core::ptr::addr_of_mut!((*thread).ApcQueueLock) as *mut ULONG_PTR,
        handle,
    );
}

/// KiAcquireApcLockAtSynchLevel — acquire thread's APC queue lock at SYNCH_LEVEL.
#[inline]
pub unsafe fn KiAcquireApcLockAtSynchLevel(
    thread: ddk::PKTHREAD,
    handle: ddk::PKLOCK_QUEUE_HANDLE,
) {
    KeAcquireInStackQueuedSpinLockAtDpcLevel(
        core::ptr::addr_of_mut!((*thread).ApcQueueLock) as *mut ULONG_PTR,
        handle,
    );
}

/// KiAcquireApcLockRaiseToDpc — acquire thread's APC queue lock, raising to DPC level.
#[inline]
pub unsafe fn KiAcquireApcLockRaiseToDpc(
    thread: ddk::PKTHREAD,
    handle: ddk::PKLOCK_QUEUE_HANDLE,
) {
    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*thread).ApcQueueLock) as *mut ULONG_PTR,
        handle,
    );
}

/// KiReleaseApcLock — release APC queue lock.
#[inline]
pub unsafe fn KiReleaseApcLock(handle: ddk::PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLock(handle);
}

/// KiReleaseApcLockFromSynchLevel — release APC queue lock from SYNCH_LEVEL.
#[inline]
pub unsafe fn KiReleaseApcLockFromSynchLevel(handle: ddk::PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLockFromDpcLevel(handle);
}

// ============================================================================
// Process lock — partial (rest in mm_rust_shims)
// ============================================================================

/// KiReleaseProcessLockFromSynchLevel — release process lock from SYNCH_LEVEL.
#[inline]
pub unsafe fn KiReleaseProcessLockFromSynchLevel(handle: ddk::PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLockFromDpcLevel(handle);
}

// ============================================================================
// Device queue lock (ke_x.h)
// ============================================================================

/// KiAcquireDeviceQueueLock — acquire device queue lock (check DpcThreadActive).
#[inline]
pub unsafe fn KiAcquireDeviceQueueLock(
    device_queue: ddk::PKDEVICE_QUEUE,
    device_lock: ddk::PKLOCK_QUEUE_HANDLE,
) {
    let prcb = KeGetCurrentPrcb();
    if prcb_dpc_thread_active(prcb) {
        KeAcquireInStackQueuedSpinLock(
            core::ptr::addr_of_mut!((*device_queue).Lock) as *mut ULONG_PTR,
            device_lock,
        );
    } else {
        KeAcquireInStackQueuedSpinLockAtDpcLevel(
            core::ptr::addr_of_mut!((*device_queue).Lock) as *mut ULONG_PTR,
            device_lock,
        );
    }
}

/// KiReleaseDeviceQueueLock — release device queue lock.
#[inline]
pub unsafe fn KiReleaseDeviceQueueLock(device_lock: ddk::PKLOCK_QUEUE_HANDLE) {
    let prcb = KeGetCurrentPrcb();
    if prcb_dpc_thread_active(prcb) {
        KeReleaseInStackQueuedSpinLock(device_lock);
    } else {
        KeReleaseInStackQueuedSpinLockFromDpcLevel(device_lock);
    }
}

// ============================================================================
// NMI list lock (ke_x.h)
// ============================================================================

/// KiAcquireNmiListLock — acquire NMI callback list spin lock.
#[inline]
pub unsafe fn KiAcquireNmiListLock(old_irql: *mut KIRQL) {
    crate::ke::KeAcquireSpinLock(
        core::ptr::addr_of_mut!(KiNmiCallbackListLock),
        old_irql,
    );
}

/// KiReleaseNmiListLock — release NMI callback list spin lock.
#[inline]
pub unsafe fn KiReleaseNmiListLock(old_irql: KIRQL) {
    crate::ke::KeReleaseSpinLock(
        core::ptr::addr_of_mut!(KiNmiCallbackListLock),
        old_irql,
    );
}

// ============================================================================
// Timer/wait helpers (ke_x.h)
// ============================================================================

/// KiRecalculateDueTime — recalculate due time for relative timers.
#[inline]
pub unsafe fn KiRecalculateDueTime(
    original_due_time: *mut ddk::_LARGE_INTEGER,
    due_time: *mut ddk::_LARGE_INTEGER,
    new_due_time: *mut ddk::_LARGE_INTEGER,
) -> *mut ddk::_LARGE_INTEGER {
    if (*original_due_time).QuadPart >= 0 {
        return original_due_time;
    }
    (*new_due_time).QuadPart = KeQueryInterruptTime() as i64;
    (*new_due_time).QuadPart -= (*due_time).QuadPart;
    new_due_time
}

/// KiCheckThreadStackSwap — check if thread stack should be swapped.
#[inline]
pub unsafe fn KiCheckThreadStackSwap(thread: ddk::PKTHREAD, wait_mode: KPROCESSOR_MODE) -> BOOLEAN {
    if wait_mode != 0 /* KernelMode */
        && (*thread).EnableStackSwap != 0
        && (*thread).Priority >= (crate::constants::LOW_REALTIME_PRIORITY as i8 + 9)
    {
        1 // TRUE
    } else {
        0 // FALSE
    }
}

/// KiCheckAlertability — check thread alertability and pending APCs.
#[inline]
pub unsafe fn KiCheckAlertability(
    thread: ddk::PKTHREAD,
    alertable: BOOLEAN,
    wait_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    if alertable != 0 {
        if (*thread).Alerted[wait_mode as usize] != 0 {
            (*thread).Alerted[wait_mode as usize] = 0;
            return 0x00000101u32 as NTSTATUS; // STATUS_ALERTED
        }
        if wait_mode != 0 /* KernelMode */ {
            let user_apc_list = &(*thread).__bindgen_anon_1.ApcState.ApcListHead[1]
                as *const ddk::_LIST_ENTRY;
            if !crate::base::IsListEmpty(user_apc_list as *const _ as *mut _) {
                (*thread).__bindgen_anon_1.ApcState.UserApcPending = 1;
                return 0x000000C0u32 as NTSTATUS; // STATUS_USER_APC
            }
        }
        if (*thread).Alerted[0] != 0 {
            (*thread).Alerted[0] = 0;
            return 0x00000101u32 as NTSTATUS; // STATUS_ALERTED
        }
    } else if wait_mode != 0 /* KernelMode */
        && (*thread).__bindgen_anon_1.ApcState.UserApcPending != 0
    {
        return 0x000000C0u32 as NTSTATUS; // STATUS_USER_APC
    }
    0 // STATUS_WAIT_0
}

/// KiComputeTimerTableIndex — compute hash index into timer table.
#[inline]
pub unsafe fn KiComputeTimerTableIndex(due_time: u64) -> ULONG {
    ((due_time / KeMaximumIncrement as u64) & (TIMER_TABLE_SIZE as u64 - 1)) as ULONG
}

/// KiRemoveEntryTimer — remove timer from its timer table entry.
#[inline]
pub unsafe fn KiRemoveEntryTimer(timer: ddk::PKTIMER) {
    let hand = (dh_get_hand(&(*timer).Header) as usize) & (TIMER_TABLE_SIZE as usize - 1);
    if crate::base::RemoveEntryList(core::ptr::addr_of_mut!((*timer).TimerListEntry)) {
        let table_entry = &mut KiTimerTableListHead[hand];
        if table_entry.Entry.Flink
            == core::ptr::addr_of_mut!(table_entry.Entry) as *mut ddk::_LIST_ENTRY
        {
            table_entry.Time.u.HighPart = -1i32 as u32;
        }
    }
}

/// KxInsertTimer — insert timer into timer table (ke_x.h).
#[inline]
pub unsafe fn KxInsertTimer(timer: ddk::PKTIMER, hand: ULONG) {
    let lock_queue = KiAcquireTimerLock(hand);
    KiReleaseDispatcherLockFromSynchLevel();
    if KiInsertTimerTable(timer, hand) != 0 {
        KiCompleteTimer(timer, lock_queue);
    } else {
        KiReleaseTimerLock(lock_queue);
    }
}

/// KiComputeDueTime — compute absolute due time for a timer (ke_x.h).
#[inline]
pub unsafe fn KiComputeDueTime(
    timer: ddk::PKTIMER,
    due_time: ddk::_LARGE_INTEGER,
    hand: *mut ULONG,
) -> BOOLEAN {
    dh_set_absolute(&mut (*timer).Header,0);

    if due_time.QuadPart >= 0 {
        // Absolute time — convert to relative
        let mut system_time: ddk::_LARGE_INTEGER = core::mem::zeroed();
        #[cfg(target_arch = "x86")]
        { ddk::KeQuerySystemTime(&mut system_time); }
        #[cfg(not(target_arch = "x86"))]
        { KeQuerySystemTime(&mut system_time); }

        let diff = system_time.QuadPart - due_time.QuadPart;
        dh_set_absolute(&mut (*timer).Header,1);
        if diff >= 0 {
            // Already expired
            (*timer).Header.SignalState = 1;
            dh_set_hand(&mut (*timer).Header, 0);
            (*timer).DueTime.QuadPart = 0;
            *hand = 0;
            return 0; // FALSE
        }
        // Use diff as negative relative time
        let interrupt_time = KeQueryInterruptTime();
        (*timer).DueTime.QuadPart = (interrupt_time as i64 - diff) as u64;
    } else {
        // Relative time
        let interrupt_time = KeQueryInterruptTime();
        (*timer).DueTime.QuadPart = (interrupt_time as i64 - due_time.QuadPart) as u64;
    }

    *hand = KiComputeTimerTableIndex((*timer).DueTime.QuadPart as u64);
    dh_set_hand(&mut (*timer).Header, *hand as u8);
    dh_set_inserted(&mut (*timer).Header,1);
    1 // TRUE
}

/// KxRemoveTreeTimer — remove timer from timer table with lock (ke_x.h).
#[inline]
pub unsafe fn KxRemoveTreeTimer(timer: ddk::PKTIMER) {
    let hand = dh_get_hand(&(*timer).Header) as u32 & (TIMER_TABLE_SIZE - 1);
    let lock_queue = KiAcquireTimerLock(hand);
    dh_set_inserted(&mut (*timer).Header,0);
    if crate::base::RemoveEntryList(core::ptr::addr_of_mut!((*timer).TimerListEntry)) {
        let timer_entry = &mut KiTimerTableListHead[hand as usize];
        if crate::base::IsListEmpty(core::ptr::addr_of_mut!(timer_entry.Entry)) {
            timer_entry.Time.u.HighPart = -1i32 as u32;
        }
    }
    KiReleaseTimerLock(lock_queue);
}

/// KxSetTimerForThreadWait — set up timer for thread wait (ke_x.h).
#[inline]
pub unsafe fn KxSetTimerForThreadWait(
    timer: ddk::PKTIMER,
    interval: ddk::_LARGE_INTEGER,
    hand: *mut ULONG,
) {
    dh_set_absolute(&mut (*timer).Header,0);

    if interval.QuadPart >= 0 {
        // Absolute time
        let mut system_time: ddk::_LARGE_INTEGER = core::mem::zeroed();
        #[cfg(target_arch = "x86")]
        { ddk::KeQuerySystemTime(&mut system_time); }
        #[cfg(not(target_arch = "x86"))]
        { KeQuerySystemTime(&mut system_time); }

        let diff = system_time.QuadPart - interval.QuadPart;
        dh_set_absolute(&mut (*timer).Header,1);
        if diff >= 0 {
            (*timer).DueTime.QuadPart = 0;
            *hand = 0;
            dh_set_hand(&mut (*timer).Header, 0);
            return;
        }
        // Use diff as negative relative time
        let interrupt_time = KeQueryInterruptTime();
        let due = interrupt_time as i64 - diff;
        (*timer).DueTime.QuadPart = due as u64;
        *hand = KiComputeTimerTableIndex(due as u64);
        dh_set_hand(&mut (*timer).Header, *hand as u8);
    } else {
        // Relative time
        let interrupt_time = KeQueryInterruptTime();
        let due = interrupt_time as i64 - interval.QuadPart;
        (*timer).DueTime.QuadPart = due as u64;
        *hand = KiComputeTimerTableIndex(due as u64);
        dh_set_hand(&mut (*timer).Header, *hand as u8);
    }
}

// ============================================================================
// Unwait functions (ke_x.h)
// ============================================================================

/// KxUnwaitThread — unwait all threads on dispatcher object's wait list (ke_x.h).
#[inline]
pub unsafe fn KxUnwaitThread(
    object: *mut ddk::_DISPATCHER_HEADER,
    increment: KPRIORITY,
) {
    let wait_list = core::ptr::addr_of_mut!((*object).WaitListHead);
    let mut wait_entry = (*wait_list).Flink;
    loop {
        if wait_entry == wait_list as *mut ddk::_LIST_ENTRY {
            break;
        }
        let wait_block = crate::inline::common::CONTAINING_RECORD_typed::<ddk::_KWAIT_BLOCK>(
            wait_entry as PVOID,
            core::mem::offset_of!(ddk::_KWAIT_BLOCK, WaitListEntry),
        );
        let wait_thread = (*wait_block).Thread;
        let wait_key = if (*wait_block).WaitType as u32 == ddk::_WAIT_TYPE_WaitAny {
            (*wait_block).WaitKey as LONG_PTR
        } else {
            0x00000100 as LONG_PTR // STATUS_KERNEL_APC
        };
        KiUnwaitThread(wait_thread, wait_key, increment);
        wait_entry = (*wait_list).Flink;
    }
}

/// KxUnwaitThreadForEvent — unwait threads for event object (ke_x.h).
#[inline]
pub unsafe fn KxUnwaitThreadForEvent(
    event: ddk::PKEVENT,
    increment: KPRIORITY,
) {
    let wait_list = core::ptr::addr_of_mut!((*event).Header.WaitListHead);
    let mut wait_entry = (*wait_list).Flink;
    loop {
        if wait_entry == wait_list as *mut ddk::_LIST_ENTRY {
            break;
        }
        let wait_block = crate::inline::common::CONTAINING_RECORD_typed::<ddk::_KWAIT_BLOCK>(
            wait_entry as PVOID,
            core::mem::offset_of!(ddk::_KWAIT_BLOCK, WaitListEntry),
        );
        let wait_thread = (*wait_block).Thread;
        if (*wait_block).WaitType as u32 == ddk::_WAIT_TYPE_WaitAny {
            (*event).Header.SignalState = 0;
            KiUnwaitThread(wait_thread, (*wait_block).WaitKey as LONG_PTR, increment);
            break;
        }
        KiUnwaitThread(wait_thread, 0x00000100 as LONG_PTR, increment);
        wait_entry = (*wait_list).Flink;
    }
}

// ============================================================================
// Ready queue functions (ke_x.h)
// ============================================================================

/// BitScanForward — find the lowest set bit; FALSE (index untouched) when `mask` is 0.
#[inline]
pub unsafe fn BitScanForward(index: *mut ULONG, mask: ULONG) -> BOOLEAN {
    if mask == 0 {
        return 0; // FALSE
    }
    *index = mask.trailing_zeros();
    1 // TRUE
}

/// BitScanReverse — find highest set bit (needed by KiSelectReadyThread).
#[inline]
pub unsafe fn BitScanReverse(index: *mut ULONG, mask: ULONG) -> BOOLEAN {
    if mask == 0 {
        return 0; // FALSE
    }
    *index = 31 - mask.leading_zeros();
    1 // TRUE
}

/// KxQueueReadyThread — queue thread to ready list (ke_x.h).
#[inline]
/// §9.56 ready-list integrity post-check. Called by each ready-list WRITER
/// after its modify, still holding the PRCB lock. If a non-empty list's
/// back-links are torn, the writer that LEFT it torn bugchecks in the act
/// (P2 high byte = call-site marker), instead of the corruption surfacing
/// later in KiSelectReadyThread's reader-side check. feature=smp only.
/// §9.56 promotion-point discriminator. Call right before a thread is promoted to
/// Running (NextThread -> CurrentThread). If it is STILL genuinely linked in a ready
/// list at that moment, then whoever SET it as NextThread left it linked — the bug is
/// upstream of the swap, in a set-NextThread path (NOT a cross-CPU queue-while-running
/// race). Silent unless the invariant is already broken. `site` is a small tag so we
/// know which swap function observed it.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn ki_check_promote_not_linked(thread: ddk::PKTHREAD, site: u64) {
    if thread.is_null() { return; }
    let entry = kt_wait_list_entry(thread);
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    let linked = !flink.is_null()
        && !blink.is_null()
        && flink != entry
        && (*flink).Blink == entry
        && (*blink).Flink == entry;
    if linked {
        crate::driver_ddk::DbgPrint(
            b"[RQPROMOTE] site=%x thr=%p state=%u F=%p B=%p - NextThread linked at promotion => set-NextThread bug upstream\n\0".as_ptr() as *const i8,
            site as u32,
            thread,
            kt_state_read(thread) as u32,
            flink,
            blink,
        );
        if RQ_ORACLE_ARMED {
        ddk::KeBugCheckEx(
            crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
            thread as usize as _,
            ((0x55u64 << 40) | site) as _,
            kt_state_read(thread) as _,
            flink as usize as _,
        );
        }
    }
}

/// §9.56 hypothesis-B catcher. A thread is being queued to a ready list — but if it
/// is CURRENTLY the CurrentThread of ANY cpu, it is a RUNNING thread and queuing it is
/// the bug (the first, illegal queue; `0x54` only catches the second). The legitimate
/// dispatch/yield/quantum paths all dethrone the outgoing thread (Prcb->CurrentThread =
/// NewThread) BEFORE queuing it, so this is quiet on them and fires only on a path that
/// readies a thread that is still running somewhere. `#[track_caller]` names the queuer.
#[cfg(feature = "smp")]
#[inline]
pub unsafe fn ki_check_not_running(thread: ddk::PKTHREAD, caller_line: u64) {
    if thread.is_null() { return; }
    let nproc = (crate::globals::KeNumberProcessors as usize).min(64);
    for c in 0..nproc {
        let p = *(core::ptr::addr_of!(crate::globals::KiProcessorBlock)
            as *const *mut ddk::_KPRCB).add(c);
        if p.is_null() { continue; }
        if (*p).CurrentThread == thread {
            // §9.56 paradox splitter: is the global dispatcher lock HELD right now?
            // The readier SHOULD hold it (serializing vs the wait-commit). Held(1)
            // yet the race happened => hypothesis (4) early-release/handoff bug.
            // Not-held(0) => hypothesis (2) this readier bypassed the dispatcher lock.
            let disp_lock_ptr =
                (*KeGetCurrentPrcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK].Lock as *const usize;
            let disp_held = if disp_lock_ptr.is_null() { 0xFFu32 }
                else { (core::ptr::read_volatile(disp_lock_ptr) & 1) as u32 };
            crate::driver_ddk::DbgPrint(
                b"[RQRUNNING] queuing a RUNNING thread thr=%p state=%u runningOnCPU=%u caller_line=%u dispLockHeld=%u\n\0".as_ptr() as *const i8,
                thread,
                kt_state_read(thread) as u32,
                c as u32,
                caller_line as u32,
                disp_held,
            );
            // §9.56 (2026-07-13): DOWNGRADED from bugcheck to print-only. This is
            // NOT the bug — it is NT's LEGAL handoff window: the wait-commit
            // releases the dispatcher lock BEFORE KiSwapThread dethrones (C
            // wait.c:567/859, and KxInsertTimer embeds the release — our Rust is
            // faithful), so a cross-CPU readier may legally link a
            // still-CurrentThread thread; the run-side SwapBusy spin in
            // ctxswitch.S is what makes that safe. The print stays as the
            // breadcrumb for the DOWNSTREAM hazard this window enables: the
            // self-swap (0x57 tripwire in KiSwapThreadImpl).
        }
    }
}

#[cfg(feature = "smp")]
#[inline]
pub unsafe fn ki_rq_integrity(prcb: ddk::PKPRCB, priority: u32, site: u64) {
    if (*prcb).ReadySummary & (1u32 << priority) == 0 {
        return; // not flagged occupied — nothing to check
    }
    let head =
        core::ptr::addr_of_mut!((*prcb).DispatcherReadyListHead[priority as usize])
            as ddk::PLIST_ENTRY;
    let flink = (*head).Flink;
    if flink == head {
        return; // legitimately empty (summary may trail a removal); reader-side owns bit/empty
    }
    // first node must back-link to head, and head's back-neighbour must forward to head
    let blink = (*head).Blink;
    let flink_bad = (*flink).Blink != head;
    let blink_bad = (*blink).Flink != head;
    if flink_bad || blink_bad {
        // §9.56 FORENSICS: the ready list is torn under a correctly-held PRCB lock,
        // so the tear is almost certainly a VICTIM of a wild write. The VALUE now
        // sitting in the corrupted link is the writer's fingerprint — dump it (and
        // the raw LIST_ENTRY words + a couple of neighbour nodes) so a torn Flink
        // that turns out to be e.g. a KTHREAD field / pool tag / small int / poison
        // names the culprit. Print BEFORE the bugcheck (still under the lock; this
        // is not mid-swap, so DbgPrint is safe here — unlike KiSwapContextResume).
        let bad_ptr = if flink_bad { (*flink).Blink } else { (*blink).Flink };
        crate::driver_ddk::DbgPrint(
            b"[RQTEAR] site=%x pri=%u prcb=%p summary=%x head=%p F=%p B=%p | flink.Blink=%p(bad=%u) blink.Flink=%p(bad=%u) BADPTR=%p | head[F,B]=%p,%p flink[F,B]=%p,%p blink[F,B]=%p,%p\n\0".as_ptr() as *const i8,
            site as u32,
            priority,
            prcb,
            (*prcb).ReadySummary,
            head,
            flink,
            blink,
            (*flink).Blink,
            flink_bad as u32,
            (*blink).Flink,
            blink_bad as u32,
            bad_ptr,
            (*head).Flink,
            (*head).Blink,
            (*flink).Flink,
            (*flink).Blink,
            (*blink).Flink,
            (*blink).Blink,
        );
        ddk::KeBugCheckEx(
            crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
            prcb as usize as _,
            ((site << 8) | priority as u64) as _,
            bad_ptr as usize as _, // the corrupted link value (the fingerprint)
            flink as usize as _,
        );
    }
}

#[cfg_attr(feature = "smp", track_caller)]
pub unsafe fn KxQueueReadyThread(thread: ddk::PKTHREAD, prcb: ddk::PKPRCB) {
    // §9.56 hypothesis-B: catch the FIRST, illegal queue — a thread that is still
    // CurrentThread (running) somewhere. Legit callers dethrone first, so quiet.
    #[cfg(feature = "smp")]
    ki_check_not_running(thread, core::panic::Location::caller().line() as u64);
    // §9.56 tripwire 0x5D — queueing a DEFERRED-PARKED thread. Legit inputs
    // here are Running (preempt re-queue of the outgoing thread) or Standby
    // (displaced NextThread). State==DeferredReady means the thread is (or is
    // about to be) parked on a deferred SLIST: the InsertTail below overwrites
    // SwapListEntry.Next (union) and reroutes that SLIST into this ready
    // list's interior — the §9.56 chain-corruption first-write. track_caller
    // names the queuer.
    #[cfg(feature = "smp")]
    {
        if kt_state_read(thread) == ddk::_KTHREAD_STATE_DeferredReady as u8 {
            let caller_line = core::panic::Location::caller().line() as u64;
            crate::driver_ddk::DbgPrint(
                b"[RQ5D] KxQueueReadyThread on DEFERRED thr=%p defproc=%u reason=%u wstat=%p caller_line=%u deferred_by=%u\n\0".as_ptr() as *const i8,
                thread,
                (*thread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor as u32,
                (*thread).WaitReason as u32,
                (*thread).WaitStatus as usize as *const u8,
                caller_line as u32,
                rq_inflight_waker(thread),
            );
            if RQ_ORACLE_ARMED {
            ddk::KeBugCheckEx(
                crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
                thread as usize as _,
                ((0x5Du64 << 40) | (caller_line << 8)
                    | (*thread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor as u64) as _,
                prcb as usize as _,
                caller_line as _,
            );
            }
        }
    }
    // §9.56 entry diagnostics (holding the PRCB lock):
    #[cfg(feature = "smp")]
    {
        let priority_c = (*thread).Priority as u32;
        // (0x52) DOUBLE-QUEUE: this thread is already Ready — queuing it again
        // re-inserts an entry that's still linked, tearing the list.
        // (0x54) ALREADY-LINKED — the precise predicate.
        // Inserting a node that is STILL LINKED into a ready list corrupts both
        // that list and this one. The old 0x52 guard tested `State == Ready`, but
        // STATE IS THE WRONG PREDICATE: a path that changes State without
        // unlinking sails past it, and a burst proved it (0x52 silent while site
        // 0x11 still tore the list right after a legit insert, PRCB lock held).
        // Test LINKAGE directly instead — a node still in a doubly-linked list has
        // neighbours that point back at it. After a correct RemoveEntryList the
        // stale Flink/Blink point at old neighbours that now point to each other,
        // so this is quiet on the legitimate paths.
        let entry = kt_wait_list_entry(thread);
        let flink = (*entry).Flink;
        let blink = (*entry).Blink;
        let already_linked = !flink.is_null()
            && !blink.is_null()
            && flink != entry // self-loop (InitializeListHead) == not in a list
            && (*flink).Blink == entry
            && (*blink).Flink == entry;
        if already_linked {
            let caller_line = core::panic::Location::caller().line() as u64;
            // §9.56 FORENSICS: `thread` is genuinely doubly-linked in SOME ready
            // list while being (re)queued. NT's dispatch/quantum paths queue
            // Prcb->CurrentThread WITHOUT a linkage check (verified: our
            // KiDpcInterruptHandler is an exact copy of NT KiDispatchInterrupt),
            // so the corruption is UPSTREAM — something left this thread linked.
            // Identify WHICH cpu's DispatcherReadyListHead[pri] it is linked into
            // by scanning KiProcessorBlock, and dump its scheduling identity so
            // the upstream writer can be traced. (Under the PRCB lock; not
            // mid-swap; DbgPrint is safe here.)
            let mut owner_cpu: i64 = -1;
            let mut owner_pri: i64 = -1;
            let nproc = crate::globals::KeNumberProcessors as usize;
            for c in 0..nproc.min(64) {
                let p = *(core::ptr::addr_of!(crate::globals::KiProcessorBlock)
                    as *const *mut ddk::_KPRCB).add(c);
                if p.is_null() { continue; }
                for pr in 0..32usize {
                    let h = core::ptr::addr_of_mut!((*p).DispatcherReadyListHead[pr]) as usize;
                    // flink/blink point AT a list node; if this thread's forward
                    // neighbour IS the list head, that names the owning list.
                    if flink as usize == h || blink as usize == h {
                        owner_cpu = c as i64;
                        owner_pri = pr as i64;
                    }
                }
            }
            crate::driver_ddk::DbgPrint(
                b"[RQLINKED] thr=%p site=%u pri=%u state=%u | F=%p B=%p ownerCPU=%d ownerPRI=%d | thisPrcb=%p thisCPU=%u\n\0".as_ptr() as *const i8,
                thread,
                caller_line as u32,
                priority_c,
                kt_state_read(thread) as u32,
                flink,
                blink,
                owner_cpu as i32,
                owner_pri as i32,
                prcb,
                (*prcb).Number as u32,
            );
            if RQ_ORACLE_ARMED {
            ddk::KeBugCheckEx(
                crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
                thread as usize as _,
                ((0x54u64 << 40) | (caller_line << 8) | priority_c as u64) as _,
                kt_state_read(thread) as _, // the state that slipped past 0x52
                caller_line as _,
            );
            }
        }
        // §9.56 (2026-07-13): affinity-defer branch MOVED BELOW the 0x5D/0x54
        // guards — it used to sit above them, letting an already-linked or
        // deferred OldThread with mismatched affinity bypass both and reach
        // the drain unchecked (the deferred_by=0 0x5A face).
        if (*thread).Affinity & (*prcb).SetMember == 0 {
            // §9.56 [RQAFF]: this branch is a DIRECT KiDeferredReadyThread
            // caller (no in-flight mark => the deferred_by=0 face). Print the
            // affinity evidence: a zero/corrupt Affinity routes EVERY requeue
            // here and then KiSelectNextProcessor(trailing_zeros(0)=64) walks
            // KiProcessorBlock[64] OUT OF BOUNDS (0x63 guards that).
            crate::driver_ddk::DbgPrint(
                b"[RQAFF] affinity-defer thr=%p affinity=%p setmember=%p state=%u reason=%u\n\0".as_ptr() as *const i8,
                thread,
                (*thread).Affinity as usize as *const u8,
                (*prcb).SetMember as usize as *const u8,
                kt_state_read(thread) as u32,
                (*thread).WaitReason as u32,
            );
            if (*thread).Affinity == 0 {
                if RQ_ORACLE_ARMED {
                ddk::KeBugCheckEx(
                    crate::bugcodes::INVALID_WORK_QUEUE_ITEM,
                    thread as usize as _,
                    (0x63u64 << 40) as _,
                    (*prcb).SetMember as _,
                    0,
                );
                }
            }
            let _ = rq_inflight_mark(thread);
            rq_waker_stamp(thread, 9100); // 9100 = affinity-defer direct path
            kt_set_state(thread, ddk::_KTHREAD_STATE_DeferredReady as u8);
            kt_set_deferred_processor(thread, (*prcb).Number as u8);
            KiReleasePrcbLock(prcb);
            KiDeferredReadyThread(thread);
            return;
        }
        // (0x51) list ALREADY torn on entry — a prior/concurrent corruptor,
        // not this insert.
        ki_rq_integrity(prcb, priority_c, 0x51);
    }
    kt_set_state(thread, ddk::_KTHREAD_STATE_Ready as u8);
    let priority = (*thread).Priority as usize;
    let preempted = (*thread).Preempted != 0;
    (*thread).Preempted = 0;
    kt_set_wait_time(thread, KeTickCount.LowPart);

    let ready_list =
        core::ptr::addr_of_mut!((*prcb).DispatcherReadyListHead[priority]);
    if preempted {
        crate::base::InsertHeadList(ready_list, kt_wait_list_entry(thread));
    } else {
        crate::base::InsertTailList(ready_list, kt_wait_list_entry(thread));
    }
    (*prcb).ReadySummary |= 1u32 << priority;
    #[cfg(feature = "smp")]
    ki_rq_integrity(prcb, priority as u32, 0x11); // site 0x11 = KxQueueReadyThread
    KiReleasePrcbLock(prcb);
}

/// KiSelectReadyThread — select highest priority ready thread (ke_x.h).
#[inline]
pub unsafe fn KiSelectReadyThread(priority: KPRIORITY, prcb: ddk::PKPRCB) -> ddk::PKTHREAD {
    let priority_set = (*prcb).ReadySummary >> priority;
    if priority_set == 0 {
        return core::ptr::null_mut();
    }
    let mut high_priority: ULONG = 0;
    BitScanReverse(&mut high_priority, priority_set);
    high_priority += priority as ULONG;

    let list_head =
        core::ptr::addr_of_mut!((*prcb).DispatcherReadyListHead[high_priority as usize]);
    let list_entry = (*list_head).Flink;

    // §9.56 tripwire (SMP ready-queue race hunt): ReadySummary said this
    // priority is occupied, so its list MUST be non-empty. If it's empty
    // (Flink == head) or its first node's back-link is broken
    // (first->Blink != head), the ReadySummary/list pair is INCONSISTENT —
    // a concurrent unserialized manipulation corrupted this Prcb's ready
    // list. Catch it HERE, at the read, with the state that localizes the
    // corruptor, instead of faulting downstream in RemoveEntryList (the
    // 0x1E IN_PAGE_ERROR face). Both derefs below are safe: list_head lives
    // in the PRCB, and the empty check gates the Blink deref so we never
    // walk a self-referential-head-as-node. Release-mode (debug_asserts at
    // the writer sites are compiled out).
    #[cfg(feature = "smp")]
    {
        if list_entry == list_head as ddk::PLIST_ENTRY
            || (*list_entry).Blink != list_head as ddk::PLIST_ENTRY
        {
            ddk::KeBugCheckEx(
                crate::bugcodes::INVALID_WORK_QUEUE_ITEM, // marker: SMP ready-list/ReadySummary inconsistency (§9.56)
                prcb as usize as _,
                high_priority as _,
                (*prcb).ReadySummary as _,
                list_entry as usize as _,
            );
        }
    }

    let thread = (list_entry as usize
        - memoffset_of_wait_list_entry()) as ddk::PKTHREAD;

    if crate::base::RemoveEntryList(kt_wait_list_entry(thread)) {
        (*prcb).ReadySummary ^= 1u32 << high_priority;
    }
    #[cfg(feature = "smp")]
    ki_rq_integrity(prcb, high_priority, 0x33); // site 0x33 = KiSelectReadyThread post-remove

    thread
}

/// KiComputeNewPriority — compute new priority after boost decay (ke_x.h).
#[inline]
pub unsafe fn KiComputeNewPriority(thread: ddk::PKTHREAD, adjustment: i8) -> i8 {
    let mut priority = (*thread).Priority;
    if priority < crate::constants::LOW_REALTIME_PRIORITY as i8 {
        priority -= (*thread).PriorityDecrement as i8 + adjustment;
        if priority < (*thread).BasePriority {
            priority = (*thread).BasePriority;
        }
        (*thread).PriorityDecrement = 0;
    }
    priority
}

// ============================================================================
// Guarded mutex functions (ke_x.h)
// ============================================================================

/// _KeInitializeGuardedMutex — initialize a guarded mutex.
#[inline]
pub unsafe fn _KeInitializeGuardedMutex(guarded_mutex: ddk::PKGUARDED_MUTEX) {
    (*guarded_mutex).Count = GM_LOCK_BIT;
    (*guarded_mutex).Owner = core::ptr::null_mut();
    (*guarded_mutex).Contention = 0;
    KeInitializeGate(core::ptr::addr_of_mut!((*guarded_mutex).Gate));
}

/// _KeAcquireGuardedMutexUnsafe — acquire guarded mutex (caller already in guarded region).
#[inline]
pub unsafe fn _KeAcquireGuardedMutexUnsafe(guarded_mutex: ddk::PKGUARDED_MUTEX) {
    _KeAcquireGuardedMutexUnsafeSited(guarded_mutex, 0)
}

/// Sited variant: the gmutex.rs export wrappers pass their return address so
/// lockdep names the true acquiring caller (sync-instrumentation §4).
#[inline]
pub unsafe fn _KeAcquireGuardedMutexUnsafeSited(guarded_mutex: ddk::PKGUARDED_MUTEX, site: usize) {
    if _InterlockedBitTestAndReset(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT_V,
    ) == 0
    {
        KiAcquireGuardedMutex(guarded_mutex);
    }
    (*guarded_mutex).Owner = current_thread();
    super::ex::lockdep_acquire(guarded_mutex as _, 3, site); // held now — gmutex
}

/// _KeReleaseGuardedMutexUnsafe — release guarded mutex (caller stays in guarded region).
#[inline]
pub unsafe fn _KeReleaseGuardedMutexUnsafe(guarded_mutex: ddk::PKGUARDED_MUTEX) {
    super::ex::lockdep_release(guarded_mutex as _); // pop before we drop it
    (*guarded_mutex).Owner = core::ptr::null_mut();

    let old_value = _InterlockedExchangeAdd(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT,
    );

    if (old_value != 0) && (old_value & GM_LOCK_WAITER_WOKEN == 0) {
        let expected = old_value + GM_LOCK_BIT;
        let new_value = expected + GM_LOCK_WAITER_WOKEN - GM_LOCK_WAITER_INC;
        // CAS loop: try to set WAITER_WOKEN and decrement WAITER_INC
        if _InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*guarded_mutex).Count),
            new_value,
            expected,
        ) == expected
        {
            KeSignalGateBoostPriority(core::ptr::addr_of_mut!((*guarded_mutex).Gate));
        }
    }
}

/// _KeAcquireGuardedMutex — enter guarded region and acquire mutex.
#[inline]
pub unsafe fn _KeAcquireGuardedMutex(guarded_mutex: ddk::PKGUARDED_MUTEX) {
    _KeAcquireGuardedMutexSited(guarded_mutex, 0)
}

/// Sited variant — see _KeAcquireGuardedMutexUnsafeSited.
#[inline]
pub unsafe fn _KeAcquireGuardedMutexSited(guarded_mutex: ddk::PKGUARDED_MUTEX, site: usize) {
    KeEnterGuardedRegion();
    if _InterlockedBitTestAndReset(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT_V,
    ) == 0
    {
        KiAcquireGuardedMutex(guarded_mutex);
    }
    (*guarded_mutex).Owner = current_thread();
    /* SpecialApcDisable ONLY (ke_x.h:1626). KernelApcDisable belongs to
     * KFAST_MUTEX, not KGUARDED_MUTEX: a guarded region disables special APCs,
     * and the field is where KeReleaseGuardedMutex's ASSERT reads the depth
     * back from. Writing it here made KeGuardedMutex report 87 assertion
     * failures per arch — the test pre-fills the mutex with 0x55 and checks
     * that this field is never touched. */
    (*guarded_mutex).__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable =
        *kthread_special_apc_disable_ptr(current_thread());
    super::ex::lockdep_acquire(guarded_mutex as _, 3, site); // held now — gmutex
}

/// _KeReleaseGuardedMutex — release mutex and leave guarded region.
#[inline]
pub unsafe fn _KeReleaseGuardedMutex(guarded_mutex: ddk::PKGUARDED_MUTEX) {
    super::ex::lockdep_release(guarded_mutex as _); // pop before we drop it
    (*guarded_mutex).Owner = core::ptr::null_mut();

    let old_value = _InterlockedExchangeAdd(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT,
    );

    if (old_value != 0) && (old_value & GM_LOCK_WAITER_WOKEN == 0) {
        let expected = old_value + GM_LOCK_BIT;
        let new_value = expected + GM_LOCK_WAITER_WOKEN - GM_LOCK_WAITER_INC;
        if _InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*guarded_mutex).Count),
            new_value,
            expected,
        ) == expected
        {
            KeSignalGateBoostPriority(core::ptr::addr_of_mut!((*guarded_mutex).Gate));
        }
    }

    KeLeaveGuardedRegion();
}

/// _KeTryToAcquireGuardedMutex — try to acquire guarded mutex.
#[inline]
pub unsafe fn _KeTryToAcquireGuardedMutex(guarded_mutex: ddk::PKGUARDED_MUTEX) -> BOOLEAN {
    _KeTryToAcquireGuardedMutexSited(guarded_mutex, 0)
}

/// Sited variant — see _KeAcquireGuardedMutexUnsafeSited.
#[inline]
pub unsafe fn _KeTryToAcquireGuardedMutexSited(
    guarded_mutex: ddk::PKGUARDED_MUTEX,
    site: usize,
) -> BOOLEAN {
    KeEnterGuardedRegion();
    if _InterlockedBitTestAndReset(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT_V,
    ) != 0
    {
        (*guarded_mutex).Owner = current_thread();
        /* SpecialApcDisable ONLY — see _KeAcquireGuardedMutexSited (ke_x.h:1694). */
        (*guarded_mutex).__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable =
            *kthread_special_apc_disable_ptr(current_thread());
        // held now — but via TRY: held-set only, no order edges (can't park).
        super::ex::lockdep_acquire_try(guarded_mutex as _, 3, site);
        1 // TRUE
    } else {
        KeLeaveGuardedRegion();
        core::hint::spin_loop(); // C: YieldProcessor() (ke_x.h:1686)
        0 // FALSE
    }
}

// ============================================================================
// Interlocked helpers (used by guarded mutex)
// ============================================================================

/// _InterlockedBitTestAndReset — atomically test and reset a bit.
#[inline]
pub unsafe fn _InterlockedBitTestAndReset(base: *mut i32, bit: i32) -> BOOLEAN {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(base as *const AtomicI32);
    let mask = 1i32 << bit;
    let old = atomic.fetch_and(!mask, Ordering::SeqCst);
    if old & mask != 0 { 1 } else { 0 }
}

/// _InterlockedExchangeAdd — atomically add and return old value.
#[inline]
pub unsafe fn _InterlockedExchangeAdd(target: *mut i32, value: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.fetch_add(value, Ordering::SeqCst)
}

/// _InterlockedExchange — atomically store and return the old value.
#[inline]
pub unsafe fn _InterlockedExchange(target: *mut i32, value: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    atomic.swap(value, Ordering::SeqCst)
}

/// _InterlockedCompareExchange — atomically compare and exchange.
#[inline]
pub unsafe fn _InterlockedCompareExchange(target: *mut i32, exchange: i32, comperand: i32) -> i32 {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(target as *const AtomicI32);
    match atomic.compare_exchange(comperand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

// ============================================================================
// Macro wrappers (ke_x.h macros → Rust inline functions)
// ============================================================================

/// KiSatisfyMutantWait — satisfy a mutant wait (ke_x.h macro).
#[inline]
pub unsafe fn KiSatisfyMutantWait(object: *mut ddk::_KMUTANT, thread: ddk::PKTHREAD) {
    (*object).Header.SignalState -= 1;
    if (*object).Header.SignalState == 0 {
        (*object).OwnerThread = thread;
        *kthread_kernel_apc_disable(thread) -= (*object).ApcDisable as i16;
        if (*object).Abandoned != 0 {
            (*object).Abandoned = 0;
            (*thread).WaitStatus = 0x00000080i32 as _; // STATUS_ABANDONED
        }
        crate::base::InsertHeadList(
            (*thread).MutantListHead.Blink as *mut ddk::_LIST_ENTRY,
            core::ptr::addr_of_mut!((*object).MutantListEntry),
        );
    }
}

/// KiSatisfyNonMutantWait — satisfy a non-mutant wait (ke_x.h macro).
#[inline]
pub unsafe fn KiSatisfyNonMutantWait(object: *mut ddk::_KMUTANT) {
    if (dh_get_type(&(*object).Header) as i32 & TIMER_OR_EVENT_TYPE)
        == ddk::_KOBJECTS_EventSynchronizationObject as i32
    {
        (*object).Header.SignalState = 0;
    } else if dh_get_type(&(*object).Header) as i32 == ddk::_KOBJECTS_SemaphoreObject as i32 {
        (*object).Header.SignalState -= 1;
    }
}

/// KiSatisfyObjectWait — dispatch to mutant or non-mutant wait satisfaction.
#[inline]
pub unsafe fn KiSatisfyObjectWait(object: *mut ddk::_KMUTANT, thread: ddk::PKTHREAD) {
    if dh_get_type(&(*object).Header) as i32 == ddk::_KOBJECTS_MutantObject as i32 {
        KiSatisfyMutantWait(object, thread);
    } else {
        KiSatisfyNonMutantWait(object);
    }
}

/// KiAddThreadToWaitList — add thread to PRCB wait list if swappable (ke_x.h macro).
#[inline]
pub unsafe fn KiAddThreadToWaitList(thread: ddk::PKTHREAD, swappable: BOOLEAN) {
    if swappable != 0 {
        let prcb = KeGetCurrentPrcb();
        crate::base::InsertTailList(
            core::ptr::addr_of_mut!((*prcb).WaitListHead),
            kt_wait_list_entry(thread),
        );
    }
}

// ============================================================================
// Statement macro wrappers (ke_x.h statement macros)
// ============================================================================

/// KxSingleThreadWait — set up single-object wait (ke_x.h statement macro).
#[inline]
pub unsafe fn KxSingleThreadWait(
    object: PVOID,
    timeout: *mut ddk::_LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    wait_reason: ddk::_KWAIT_REASON,
    alertable: BOOLEAN,
    swappable: *mut BOOLEAN,
    due_time: *mut ddk::_LARGE_INTEGER,
    hand: *mut ULONG,
) {
    let thread = KeGetCurrentThread_portable();
    let wait_block = kt_wait_block(thread, 0);
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, wait_block);
    (*wait_block).WaitKey = 0; // STATUS_SUCCESS
    (*wait_block).Object = object;
    (*wait_block).WaitType = ddk::_WAIT_TYPE_WaitAny as u8;
    (*thread).WaitStatus = 0;

    if !timeout.is_null() {
        KxSetTimerForThreadWait(timer, *timeout, hand);
        (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
        (*wait_block).NextWaitBlock = timer_block;
        (*timer_block).NextWaitBlock = wait_block;
        // Defensive BOTH-ENDS init (parity with the C ke_x.h vibeos patch):
        // the original macro relied on TimerBlock->WaitListEntry staying
        // pre-linked from thread init, but it can be observed self-referencing
        // (KiTimerExpiration -> KxUnwaitThread spin) or stale — and a stale
        // RemoveEntryList in KiUnlinkThread writes through dangling pointers
        // (§9.56 tear class). Idempotent regardless of prior state.
        (*timer_block).WaitListEntry.Flink =
            core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
        (*timer_block).WaitListEntry.Blink =
            core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
        (*timer).Header.WaitListHead.Flink =
            core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
        (*timer).Header.WaitListHead.Blink =
            core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
    } else {
        (*wait_block).NextWaitBlock = wait_block;
    }

    (*thread).Alertable = alertable;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = wait_reason as u8;
    (*kt_wait_list_entry(thread)).Flink = core::ptr::null_mut();
    *swappable = KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, KeTickCount.LowPart);
}

/// KxDelayThreadWait — set up delay execution wait (ke_x.h statement macro).
#[inline]
pub unsafe fn KxDelayThreadWait(
    interval: *mut ddk::_LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    alertable: BOOLEAN,
    swappable: *mut BOOLEAN,
    due_time: *mut ddk::_LARGE_INTEGER,
    hand: *mut ULONG,
) {
    let thread = KeGetCurrentThread_portable();
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, timer_block);
    (*timer_block).NextWaitBlock = timer_block;
    KxSetTimerForThreadWait(timer, *interval, hand);
    (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
    // Defensive BOTH-ENDS init — see KxSingleThreadWait note (§9.56 tear class).
    (*timer_block).WaitListEntry.Flink =
        core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
    (*timer_block).WaitListEntry.Blink =
        core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
    (*timer).Header.WaitListHead.Flink =
        core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
    (*timer).Header.WaitListHead.Blink =
        core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
    // The C macro (ke_x.h KxDelayThreadWait) sets Thread->Alertable from the
    // caller's Alertable local. This was mistranslated as (wait_mode==UserMode),
    // which made every non-alertable user Sleep() alertable (spurious early
    // STATUS_USER_APC wakes) and every alertable KernelMode delay non-alertable
    // (KeAlertThread/user-APC wakeups test Thread->Alertable → lost wakeup).
    (*thread).Alertable = alertable;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = ddk::_KWAIT_REASON_DelayExecution as u8;
    (*kt_wait_list_entry(thread)).Flink = core::ptr::null_mut();
    *swappable = KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, KeTickCount.LowPart);
}

/// KxMultiThreadWait — set up multi-object wait (ke_x.h statement macro).
#[inline]
pub unsafe fn KxMultiThreadWait(
    wait_block_array: *mut ddk::_KWAIT_BLOCK,
    objects: *mut PVOID,
    count: ULONG,
    timeout: *mut ddk::_LARGE_INTEGER,
    wait_type: ddk::_WAIT_TYPE,
    wait_mode: KPROCESSOR_MODE,
    wait_reason: ddk::_KWAIT_REASON,
    alertable: BOOLEAN,
    swappable: *mut BOOLEAN,
    due_time: *mut ddk::_LARGE_INTEGER,
    hand: *mut ULONG,
) {
    let thread = KeGetCurrentThread_portable();
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, wait_block_array);
    for i in 0..count as usize {
        let wb = wait_block_array.add(i);
        (*wb).Object = *objects.add(i);
        (*wb).WaitKey = i as u16;
        (*wb).WaitType = wait_type as u8;
        (*wb).Thread = thread;
        (*wb).NextWaitBlock = wait_block_array.add(i + 1);
    }

    (*thread).WaitStatus = 0;
    if !timeout.is_null() {
        KxSetTimerForThreadWait(timer, *timeout, hand);
        (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
        (*wait_block_array.add(count as usize - 1)).NextWaitBlock = timer_block;
        (*timer_block).NextWaitBlock = wait_block_array;
        // Match the original ReactOS macro KxMultiThreadWait
        // (ntoskrnl/include/internal/ke_x.h): InitializeListHead at the end
        // of the timeout setup. KeWaitForMultipleObjects loops over the
        // WaitBlock chain (which now includes timer_block) and calls
        // InsertTailList for each — INCLUDING timer_block — which properly
        // inserts it into Timer.WaitListHead. Setting Header.WaitListHead.
        // Flink/Blink to &timer_block.WaitListEntry here would corrupt
        // timer_block on the subsequent InsertTailList (Entry.Blink =
        // OldBlink = &timer_block → self-reference) and cause the
        // KiTimerExpiration → KxUnwaitThread → KiUnwaitThread spin loop.
        InitializeListHead(
            core::ptr::addr_of_mut!((*timer).Header.WaitListHead) as *mut LIST_ENTRY,
        );
    } else {
        (*wait_block_array.add(count as usize - 1)).NextWaitBlock = wait_block_array;
    }

    (*thread).Alertable = alertable;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = wait_reason as u8;
    (*kt_wait_list_entry(thread)).Flink = core::ptr::null_mut();
    *swappable = KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, KeTickCount.LowPart);
}

/// KxQueueThreadWait — set up queue wait (ke_x.h statement macro).
#[inline]
pub unsafe fn KxQueueThreadWait(
    queue: PVOID,
    timeout: *mut ddk::_LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    swappable: *mut BOOLEAN,
    due_time: *mut ddk::_LARGE_INTEGER,
    hand: *mut ULONG,
) {
    let thread = KeGetCurrentThread_portable();
    let wait_block = kt_wait_block(thread, 0);
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, wait_block);
    (*wait_block).WaitKey = 0; // STATUS_SUCCESS
    (*wait_block).Object = queue;
    (*wait_block).WaitType = ddk::_WAIT_TYPE_WaitAny as u8;
    (*wait_block).Thread = thread;
    (*thread).WaitStatus = 0;

    if !timeout.is_null() {
        KxSetTimerForThreadWait(timer, *timeout, hand);
        (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
        (*wait_block).NextWaitBlock = timer_block;
        (*timer_block).NextWaitBlock = wait_block;
        // Defensive BOTH-ENDS init (parity with the C ke_x.h vibeos patch):
        // the original macro relied on TimerBlock->WaitListEntry staying
        // pre-linked from thread init, but it can be observed self-referencing
        // (KiTimerExpiration -> KxUnwaitThread spin) or stale — and a stale
        // RemoveEntryList in KiUnlinkThread writes through dangling pointers
        // (§9.56 tear class). Idempotent regardless of prior state.
        (*timer_block).WaitListEntry.Flink =
            core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
        (*timer_block).WaitListEntry.Blink =
            core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
        (*timer).Header.WaitListHead.Flink =
            core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
        (*timer).Header.WaitListHead.Blink =
            core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
    } else {
        (*wait_block).NextWaitBlock = wait_block;
    }

    (*thread).Alertable = 0; // FALSE
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = ddk::_KWAIT_REASON_WrQueue as u8;
    (*kt_wait_list_entry(thread)).Flink = core::ptr::null_mut();
    *swappable = KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, KeTickCount.LowPart);
}

// ============================================================================
// Utility functions
// ============================================================================

/// YieldProcessor — architecture-dependent yield hint.
///
/// NOTE: `nomem` intentionally absent. With it, LLVM treats retry loops as
/// having no memory side effects → infinite loop without side effects is UB
/// → LLVM eliminates the loop entirely.
#[inline]
pub unsafe fn YieldProcessor() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!("pause", options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!("yield", options(nostack, preserves_flags));
    }
}

/// KeGetTrapFramePc — get PC from trap frame (per arch).
#[inline]
pub unsafe fn KeGetTrapFramePc(trap_frame: ddk::PKTRAP_FRAME) -> ULONG_PTR {
    #[cfg(target_arch = "x86")]
    { (*trap_frame).Eip as ULONG_PTR }
    #[cfg(target_arch = "x86_64")]
    { (*trap_frame).Rip as ULONG_PTR }
    #[cfg(target_arch = "aarch64")]
    { (*trap_frame).Pc as ULONG_PTR }
}

/// KiRundownThread — clean up FPU state on thread exit.
#[inline]
pub unsafe fn KiRundownThread(thread: ddk::PKTHREAD) {
    #[cfg(target_arch = "x86")]
    {
        let prcb = KeGetCurrentPrcb();
        if (*prcb).NpxThread == thread {
            (*prcb).NpxThread = core::ptr::null_mut();
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = thread;
    }
}

/// KiGetLinkedTrapFrame — get previous trap frame.
#[inline]
pub unsafe fn KiGetLinkedTrapFrame(trap_frame: ddk::PKTRAP_FRAME) -> ddk::PKTRAP_FRAME {
    tf_get_linked(trap_frame)
}

/// KiGetUserModeStackAddress — get pointer to user-mode stack in current thread's trap frame.
#[inline]
pub unsafe fn KiGetUserModeStackAddress() -> *mut ULONG_PTR {
    let thread = PsGetCurrentThread();
    let trap_frame = (*(thread as ddk::PKTHREAD)).TrapFrame;
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*trap_frame).HardwareEsp) as *mut ULONG_PTR }
    #[cfg(target_arch = "x86_64")]
    { core::ptr::addr_of_mut!((*trap_frame).Rsp) as *mut ULONG_PTR }
    #[cfg(target_arch = "aarch64")]
    { core::ptr::addr_of_mut!((*trap_frame).Sp) as *mut ULONG_PTR }
}

/// InterlockedBitTestAndSet — atomically test and set a bit (cdecl, all arches).
#[inline]
pub unsafe fn InterlockedBitTestAndSet(base: *mut i32, bit: i32) -> BOOLEAN {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(base as *const AtomicI32);
    let mask = 1i32 << bit;
    let old = atomic.fetch_or(mask, Ordering::SeqCst);
    if old & mask != 0 { 1 } else { 0 }
}

/// InterlockedBitTestAndReset — atomically test and reset a bit (cdecl, all arches).
#[inline]
pub unsafe fn InterlockedBitTestAndReset(base: *mut i32, bit: i32) -> BOOLEAN {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(base as *const AtomicI32);
    let mask = 1i32 << bit;
    let old = atomic.fetch_and(!mask, Ordering::SeqCst);
    if old & mask != 0 { 1 } else { 0 }
}
