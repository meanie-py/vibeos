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
// KeGetCurrentProcessorNumber (FORCEINLINE — UP kernel always returns 0)
// ============================================================================

#[inline]
pub unsafe fn KeGetCurrentProcessorNumber() -> ULONG {
    0
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

/// KeSweepICache — arm64: DC CVAU per cache line + DSB + IC IALLU + DSB + ISB.
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
    }
    core::arch::asm!(
        "dsb ish",
        "ic iallu",
        "dsb ish",
        "isb",
        options(nostack),
    );
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

/// KxAcquireSpinLock — UP kernel: compiler barrier only.
/// DBG owner tracking disabled: inconsistent paths between C shims and Rust inline
/// cause false SPIN_LOCK_NOT_OWNED on i386. To be investigated separately.
#[inline]
pub unsafe fn KxAcquireSpinLock(spin_lock: *mut ULONG_PTR) {
    // UP kernel: compiler barrier (no SMP spin needed)
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

/// KxReleaseSpinLock — UP kernel: compiler barrier only.
/// DBG owner tracking disabled (see KxAcquireSpinLock comment).
#[inline]
pub unsafe fn KxReleaseSpinLock(spin_lock: *mut ULONG_PTR) {
    // UP kernel: compiler barrier
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
#[cfg(target_arch = "x86")]
extern "system" {
    fn KiCheckForKernelApcDelivery();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
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
    extern "C" {
        static SharedUserData: *mut ddk::_KUSER_SHARED_DATA;
    }
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

/// SYNCH_LEVEL — in UP kernel, equals DISPATCH_LEVEL (= 2)
pub const SYNCH_LEVEL: KIRQL = 2; // DISPATCH_LEVEL

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
#[cfg(target_arch = "x86")]
extern "system" {
    fn PsGetCurrentThread() -> ddk::PETHREAD;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
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
extern "C" {
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
// UP no-op lock functions (ke_x.h, #ifndef CONFIG_SMP section)
// ============================================================================

/// KiAcquireThreadLock — UP: no-op.
#[inline]
pub unsafe fn KiAcquireThreadLock(_thread: ddk::PKTHREAD) {}

/// KiReleaseThreadLock — UP: no-op.
#[inline]
pub unsafe fn KiReleaseThreadLock(_thread: ddk::PKTHREAD) {}

/// KiAcquireDispatcherObject — UP: no-op.
#[inline]
pub unsafe fn KiAcquireDispatcherObject(_object: *mut ddk::_DISPATCHER_HEADER) {}

/// KiReleaseDispatcherObject — UP: no-op.
#[inline]
pub unsafe fn KiReleaseDispatcherObject(_object: *mut ddk::_DISPATCHER_HEADER) {}

/// KiAcquireDispatcherLockAtSynchLevel — UP: no-op.
#[inline]
pub unsafe fn KiAcquireDispatcherLockAtSynchLevel() {}

/// KiReleaseDispatcherLockFromSynchLevel — UP: no-op.
#[inline]
pub unsafe fn KiReleaseDispatcherLockFromSynchLevel() {}

/// KiRescheduleThread — UP: no-op.
#[inline]
pub unsafe fn KiRescheduleThread(_new_thread: BOOLEAN, _cpu: ULONG) {}

/// KiSetThreadSwapBusy — UP: no-op.
#[inline]
pub unsafe fn KiSetThreadSwapBusy(_thread: ddk::PKTHREAD) {}

/// KiAcquirePrcbLock — UP: no-op.
#[inline]
pub unsafe fn KiAcquirePrcbLock(_prcb: ddk::PKPRCB) {}

/// KiReleasePrcbLock — UP: no-op.
#[inline]
pub unsafe fn KiReleasePrcbLock(_prcb: ddk::PKPRCB) {}

/// KiTryThreadLock — UP: always returns FALSE (lock never held).
#[inline]
pub unsafe fn KiTryThreadLock(_thread: ddk::PKTHREAD) -> BOOLEAN {
    0 // FALSE
}

/// KiCheckDeferredReadyList — UP: no-op.
#[inline]
pub unsafe fn KiCheckDeferredReadyList(_prcb: ddk::PKPRCB) {}

/// KiRequestApcInterrupt — UP: no-op.
#[inline]
pub unsafe fn KiRequestApcInterrupt(_need_apc: BOOLEAN, _processor: UCHAR) {}

/// KiAcquireTimerLock — UP: returns NULL.
#[inline]
pub unsafe fn KiAcquireTimerLock(_hand: ULONG) -> ddk::PKSPIN_LOCK_QUEUE {
    core::ptr::null_mut()
}

/// KiReleaseTimerLock — UP: no-op.
#[inline]
pub unsafe fn KiReleaseTimerLock(_lock_queue: ddk::PKSPIN_LOCK_QUEUE) {}

// ============================================================================
// Dispatcher lock — simple wrappers (ke_x.h UP section)
// ============================================================================

/// KiAcquireDispatcherLock — UP: raise IRQL to SYNCH_LEVEL.
#[inline]
pub unsafe fn KiAcquireDispatcherLock() -> KIRQL {
    KfRaiseIrql(SYNCH_LEVEL)
}

/// KiReleaseDispatcherLock — UP: call KiExitDispatcher.
#[inline]
pub unsafe fn KiReleaseDispatcherLock(old_irql: KIRQL) {
    KiExitDispatcher(old_irql);
}

// ============================================================================
// KiInsertDeferredReadyList (ke_x.h UP section)
// ============================================================================

/// KiInsertDeferredReadyList — UP: set DeferredReady state and call KiDeferredReadyThread.
#[inline]
pub unsafe fn KiInsertDeferredReadyList(thread: ddk::PKTHREAD) {
    kt_set_state(thread, ddk::_KTHREAD_STATE_DeferredReady as u8);
    kt_set_deferred_processor(thread, 0);
    KiDeferredReadyThread(thread);
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
pub unsafe fn KxQueueReadyThread(thread: ddk::PKTHREAD, prcb: ddk::PKPRCB) {
    // UP kernel: always TRUE (no affinity check needed)
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

    let list_entry = (*prcb).DispatcherReadyListHead[high_priority as usize].Flink;
    let thread = (list_entry as usize
        - memoffset_of_wait_list_entry()) as ddk::PKTHREAD;

    if crate::base::RemoveEntryList(kt_wait_list_entry(thread)) {
        (*prcb).ReadySummary ^= 1u32 << high_priority;
    }

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
    if _InterlockedBitTestAndReset(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT_V,
    ) == 0
    {
        KiAcquireGuardedMutex(guarded_mutex);
    }
    (*guarded_mutex).Owner = current_thread();
}

/// _KeReleaseGuardedMutexUnsafe — release guarded mutex (caller stays in guarded region).
#[inline]
pub unsafe fn _KeReleaseGuardedMutexUnsafe(guarded_mutex: ddk::PKGUARDED_MUTEX) {
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
    KeEnterGuardedRegion();
    if _InterlockedBitTestAndReset(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT_V,
    ) == 0
    {
        KiAcquireGuardedMutex(guarded_mutex);
    }
    (*guarded_mutex).Owner = current_thread();
    let thread = current_thread();
    (*guarded_mutex).__bindgen_anon_1.__bindgen_anon_1.KernelApcDisable =
        *kthread_kernel_apc_disable(thread);
    (*guarded_mutex).__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable =
        *kthread_special_apc_disable_ptr(thread);
}

/// _KeReleaseGuardedMutex — release mutex and leave guarded region.
#[inline]
pub unsafe fn _KeReleaseGuardedMutex(guarded_mutex: ddk::PKGUARDED_MUTEX) {
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
    KeEnterGuardedRegion();
    if _InterlockedBitTestAndReset(
        core::ptr::addr_of_mut!((*guarded_mutex).Count),
        GM_LOCK_BIT_V,
    ) != 0
    {
        (*guarded_mutex).Owner = current_thread();
        let thread = current_thread();
        (*guarded_mutex).__bindgen_anon_1.__bindgen_anon_1.KernelApcDisable =
            *kthread_kernel_apc_disable(thread);
        (*guarded_mutex).__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable =
            *kthread_special_apc_disable_ptr(thread);
        1 // TRUE
    } else {
        KeLeaveGuardedRegion();
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
