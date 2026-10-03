//! KE subsystem shared definitions — extern declarations, constants, and type re-exports.
//!
//! All ke/ files should `use super::ke_defs::*;` to get these.

// ============================================================================
// Extern "system" functions used across ke/ files
// ============================================================================
extern "system" {
    pub fn KeRaiseIrqlToSynchLevel() -> u8;

    // Misc kernel functions
    pub fn ExFreePool(P: *mut core::ffi::c_void);
}
extern "C" {
    // IRQL management (provided by ke_rust_shims.c or HAL)
    pub fn KeRaiseIrql(NewIrql: u8, OldIrql: *mut u8);
    pub fn KeLowerIrql(NewIrql: u8);
}
extern "system" {
    // Critical/Guarded regions (provided by ke_rust_shims.c)
    pub fn KeEnterCriticalRegion();
    pub fn KeLeaveCriticalRegion();
}
// KfRaiseIrql is the raw HAL export: FASTCALL on i386 (@KfRaiseIrql@4 —
// a plain extern "C" decl links as _KfRaiseIrql and dies at link time the
// moment a caller is compiled in), cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KfRaiseIrql(NewIrql: u8) -> u8;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KfRaiseIrql(NewIrql: u8) -> u8;
}
extern "C" {
    // Dispatcher lock functions (provided by ex_rust_shims.c or ke_rust_shims.c)
    pub fn KiAcquireDispatcherLock() -> u8;
    pub fn KiReleaseDispatcherLock(OldIrql: u8);
    pub fn KiAcquireDispatcherLockAtSynchLevel();
    pub fn KiReleaseDispatcherLockFromSynchLevel();

    // Thread lock functions (provided by ex_rust_shims.c)
    pub fn KiAcquireThreadLock(Thread: *mut ntkrnl_sys::ddk::_KTHREAD);
    pub fn KiReleaseThreadLock(Thread: *mut ntkrnl_sys::ddk::_KTHREAD);

    // Spinlock helpers (provided by ex_rust_shims.c or ke_rust_shims.c)
    // Takes PKSPIN_LOCK but declared as *mut c_void for portability
    pub fn KxAcquireSpinLock(SpinLock: *mut core::ffi::c_void);
    pub fn KxReleaseSpinLock(SpinLock: *mut core::ffi::c_void);

    // Interlocked operations (provided by ke_rust_shims.c or compiler builtins)
    pub fn InterlockedBitTestAndSet(Base: *mut i32, Bit: i32) -> u8;
    pub fn InterlockedBitTestAndReset(Base: *mut i32, Bit: i32) -> u8;

    // Timer/Wait helpers (provided by ke_rust_shims.c)
    pub fn YieldProcessor();
}
// KiWaitTest, KiUnwaitThread, KiUnlinkThread are FASTCALL on i386, extern "C" on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KiWaitTest(Object: *mut core::ffi::c_void, Increment: i32);
    pub fn KiUnwaitThread(Thread: *mut ntkrnl_sys::ddk::_KTHREAD, WaitStatus: ntkrnl_sys::base::LONG_PTR, Increment: i32);
    pub fn KiUnlinkThread(Thread: *mut ntkrnl_sys::ddk::_KTHREAD, WaitStatus: ntkrnl_sys::base::LONG_PTR);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KiWaitTest(Object: *mut core::ffi::c_void, Increment: i32);
    pub fn KiUnwaitThread(Thread: *mut ntkrnl_sys::ddk::_KTHREAD, WaitStatus: ntkrnl_sys::base::LONG_PTR, Increment: i32);
    pub fn KiUnlinkThread(Thread: *mut ntkrnl_sys::ddk::_KTHREAD, WaitStatus: ntkrnl_sys::base::LONG_PTR);
}
extern "C" {
}
extern "system" {
    pub fn KdReportProcessorChange() -> u32;

    // Ke intrinsics provided by ke_rust_shims.c
    pub fn _rdmsr(Register: u32) -> u64;
    pub fn _wrmsr(Register: u32, Value: u64);
}
extern "system" {
    // KeGetCurrentThread — FORCEINLINE on amd64, provided by PsGetCurrentThread shim
    // Must be "system" (stdcall on i386) to match the shim export's calling convention.
    pub fn PsGetCurrentThread() -> *mut ntkrnl_sys::ddk::_KTHREAD;
}
// §9.56 mechanism hunt: on SMP, bring the INLINE KxQueueReadyThread into scope
// (instead of the extern-C shim symbol) so its #[track_caller] reports the TRUE
// caller line. Going through the extern-C wrapper flattened every in-crate call
// site to shim_exports.rs:2069, defeating the line-based discriminator. UP keeps
// the extern symbol path unchanged.
#[cfg(feature = "smp")]
pub use ntkrnl_sys::inline::ke::KxQueueReadyThread;
#[cfg(not(feature = "smp"))]
extern "C" {
    pub fn KxQueueReadyThread(Thread: *mut ntkrnl_sys::ddk::_KTHREAD, Prcb: *mut ntkrnl_sys::ddk::_KPRCB);
}

// Arch-specific extern declarations
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
extern "system" {
    pub fn __cpuidex(CPUInfo: *mut i32, Function: i32, SubLeaf: i32);
}
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
extern "C" {
    pub fn __writemsr(Register: u32, Value: u64);
}

#[cfg(target_arch = "x86_64")]
extern "system" {
    pub fn InterlockedBitTestAndSet64(Base: *mut i64, Bit: i64) -> u8;
    pub fn InterlockedAnd64(Destination: *mut i64, Value: i64) -> i64;
    pub fn _xsetbv(Register: u32, Value: u64);
    pub fn _rdtsc() -> u64;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    pub fn __writecr3(Value: u64);
}

#[cfg(target_arch = "x86")]
extern "system" {
    pub fn ExAllocatePool(PoolType: u32, NumberOfBytes: u32) -> *mut core::ffi::c_void;
    pub fn KeIpiGenericCall(
        BroadcastFunction: Option<unsafe extern "system" fn(usize) -> usize>,
        Context: usize,
    ) -> usize;
    pub fn KeTrapFrameToContext(
        TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
        ExceptionFrame: *mut core::ffi::c_void,
        Context: *mut ntkrnl_sys::ddk::_CONTEXT,
    );
    pub fn RtlCaptureContext(ContextRecord: *mut ntkrnl_sys::ddk::_CONTEXT);
    pub fn Ke386SetLocalDescriptorTable(Selector: u16);
    pub fn Ke386SetGs(Selector: u16);
    pub fn KiComputeIopmOffset(SetMember: u32) -> u16;
}
#[cfg(target_arch = "x86")]
extern "C" {
    pub fn KeGetPcr() -> *mut ntkrnl_sys::ddk::_KIPCR;
    pub fn KeInvalidateTlbEntry(Address: *mut core::ffi::c_void);
}

// ============================================================================
// Re-exported globals from defining modules (NOT extern — avoids lto=false splits)
// ============================================================================
// Timer-related globals (from dpc.rs and timerobj.rs)
pub use super::dpc::{KiTimerExpireDpc, KiMaximumDpcQueueDepth, KiMinimumDpcRate,
    KiAdjustDpcThreshold, KeThreadDpcEnable, KiGenericCallDpcMutex};
pub use super::timerobj::KiTimerTableListHead;

// Profile globals (from profobj.rs)
pub use super::profobj::{KiProfileLock, KiProfileListHead, KiProfileSourceListHead};

// Swap/process globals (from procobj.rs)
pub use super::procobj::{KiSwapEvent, KiProcessInSwapListHead, KiProcessOutSwapListHead,
    KiStackInSwapListHead, KeServiceDescriptorTable, KeServiceDescriptorTableShadow};

// Processor globals (from processor.rs)
pub use super::processor::KeActiveProcessors;

// MainSSDT / MainSSPT live in `super::ssdt` (generated from sysfuncs.h);
// re-export here so existing consumers (`use ke_defs::*`) keep working.
pub use super::ssdt::{MainSSDT, MainSSPT};
extern "system" {
    pub fn KeQuerySystemTime(CurrentTime: *mut i64);
}
extern "system" {
    pub fn KeEnterGuardedRegion();
}
extern "system" {
    pub fn KeLeaveGuardedRegion();
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KiSignalTimer(Timer: *mut ntkrnl_sys::ddk::_KTIMER);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KiSignalTimer(Timer: *mut ntkrnl_sys::ddk::_KTIMER);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KiIpiSignalPacketDone(SignalDone: *mut core::ffi::c_void);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KiIpiSignalPacketDone(SignalDone: *mut core::ffi::c_void);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KiRecordDr7(Dr7: *mut u32, DrMask: *mut u32) -> u8;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KiRecordDr7(Dr7: *mut u32, DrMask: *mut u32) -> u8;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KiUpdateDr7(Dr7: u32) -> u32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KiUpdateDr7(Dr7: u32) -> u32;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KiGetMachineBootPointers(
        Gdt: *mut *mut core::ffi::c_void,
        Idt: *mut *mut core::ffi::c_void,
        Tss: *mut *mut core::ffi::c_void,
        Pcr: *mut *mut core::ffi::c_void,
    );
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KiGetMachineBootPointers(
        Gdt: *mut *mut core::ffi::c_void,
        Idt: *mut *mut core::ffi::c_void,
        Tss: *mut *mut core::ffi::c_void,
        Pcr: *mut *mut core::ffi::c_void,
    );
}

// ============================================================================
// Constants used across ke/ files
// ============================================================================

// Timer table size
pub const TIMER_TABLE_SIZE: usize = 512;
pub const LOCK_QUEUE_TIMER_TABLE_LOCKS: usize = 16;

// SSDT constants
pub const NUMBER_OF_SYSCALLS: u32 = 0x1000;
pub const SSDT_MAX_ENTRIES: u32 = 2;

// Priority
pub const MAXIMUM_PRIORITY: u32 = 32;

// DPC importance
pub const DPC_NORMAL: u32 = 0;

// IPI freeze states
// IPI request values (ndk/<arch>/ketypes.h). NB the ipi.rs translation uses
// them as BIT POSITIONS in IpiFrozen (`1 << IPI_x`), faithfully mirroring the
// C's InterlockedBitTestAndReset(&Prcb->IpiFrozen, IPI_x).
pub const IPI_APC: u32 = 1;
pub const IPI_DPC: u32 = 2;
pub const IPI_FREEZE: u32 = 4;
pub const IPI_SYNCH_REQUEST: u32 = 16;
pub const IPI_FROZEN_FLAG_ACTIVE: u32 = 0x20;
pub const IPI_FROZEN_STATE_RUNNING: u32 = 0;
pub const IPI_FROZEN_STATE_TARGET_FREEZE: u32 = 0x5;
pub const IPI_FROZEN_STATE_FROZEN: u32 = 2;
pub const IPI_FROZEN_STATE_THAW: u32 = 3;
pub const IPI_FROZEN_STATE_OWNER: u32 = 4;

// IRQ levels
// HIGH_LEVEL and IPI_LEVEL are in ntkrnl_sys::constants (cfg-gated per arch)
// UP kernel: SYNCH_LEVEL == DISPATCH_LEVEL; SMP: IPI_LEVEL - 2 on ALL THREE
// arches — every ndk header has the same `#ifndef CONFIG_SMP` pair (i386
// asm.h; amd64/arm64 ketypes.h). arm64 read 2 here until 2026-07-25 on the
// premise that its ndk had no SMP variant; it does (ketypes.h:29), and kmtest
// asks the running kernel (KmtIsMultiProcessorBuild) which one to expect.
// 12 lands in the gap the arm64 IRQL↔GIC map already leaves free: above every
// DIRQL (max 11) and below CLOCK_LEVEL(13), so a SYNCH section masks devices
// but never the scheduler tick (hal-rust arm64/clock.rs gic_priority).
#[cfg(not(feature = "smp"))]
pub const SYNCH_LEVEL: u8 = 2;
#[cfg(all(feature = "smp", target_arch = "x86"))]
pub const SYNCH_LEVEL: u8 = 0x1D - 2; // 27
#[cfg(all(feature = "smp", target_arch = "x86_64"))]
pub const SYNCH_LEVEL: u8 = 14 - 2; // 12
#[cfg(all(feature = "smp", target_arch = "aarch64"))]
pub const SYNCH_LEVEL: u8 = 14 - 2; // 12

// Exported for ke_rust_shims.c's amd64 KeRaiseIrqlToSynchLevel CR8 shim (the
// only out-of-line provider of that export on amd64) so the C side raises to
// the kernel-flavor level without needing CONFIG_SMP at its compile.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub static KiSynchIrql: u8 = SYNCH_LEVEL;

// Bugcheck codes
pub const IRQL_NOT_GREATER_OR_EQUAL: u32 = 0x0000_0009;
pub const SPIN_LOCK_ALREADY_OWNED: u32 = 0x0000_000F;
pub const TRAP_CAUSE_UNKNOWN: u32 = 0x0000_0012;
pub const MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED: u32 = 0x0000_003E;
pub const INVALID_PROCESS_ATTACH_ATTEMPT: u32 = 0x0000_0005;
pub const INVALID_PROCESS_DETACH_ATTEMPT: u32 = 0x0000_0006;
pub const UNSUPPORTED_PROCESSOR: u32 = 0x0000_005D;
pub const UNEXPECTED_KERNEL_MODE_TRAP: u32 = 0x0000_007F;
pub const ATTEMPTED_SWITCH_FROM_DPC: u32 = 0x0000_00B8;
pub const MISMATCHED_HAL: u32 = 0x0000_0079;
pub const NMI_HARDWARE_FAILURE: u32 = 0x0000_0080;
pub const KERNEL_SECURITY_CHECK_FAILURE: u32 = 0x0000_0139;

// Status codes
pub const STATUS_ABANDONED: i32 = 0x0000_0080u32 as i32;
pub const STATUS_USER_APC: i32 = 0x0000_00C0u32 as i32;
pub const STATUS_KERNEL_APC: i32 = 0x0000_0100u32 as i32;
pub const STATUS_SEMAPHORE_LIMIT_EXCEEDED: i32 = 0xC0000047_u32 as i32;
pub const STATUS_MUTANT_NOT_OWNED: i32 = 0xC0000046_u32 as i32;
pub const STATUS_INVALID_SYSTEM_SERVICE: i32 = 0xC000_001Cu32 as i32;
pub const STATUS_PRIVILEGED_INSTRUCTION: i32 = 0xC000_0096u32 as i32;
pub const STATUS_NO_CALLBACK_ACTIVE: i32 = 0xC0000258_u32 as i32;
pub const STATUS_CALLBACK_POP_STACK: i32 = 0xC0000423_u32 as i32;
pub const STATUS_FLOAT_DIVIDE_BY_ZERO: i32 = 0xC000_008Eu32 as i32;
pub const STATUS_FLOAT_INEXACT_RESULT: i32 = 0xC000_008Fu32 as i32;
pub const STATUS_FLOAT_INVALID_OPERATION: i32 = 0xC000_0090u32 as i32;
pub const STATUS_FLOAT_OVERFLOW: i32 = 0xC000_0091u32 as i32;
pub const STATUS_FLOAT_UNDERFLOW: i32 = 0xC000_0093u32 as i32;
pub const STATUS_FLOAT_MULTIPLE_TRAPS: i32 = 0xC00002B5_u32 as i32;
pub const STATUS_FLOAT_STACK_CHECK: i32 = 0xC000_0092u32 as i32;
pub const STATUS_INTEGER_DIVIDE_BY_ZERO: i32 = 0xC000_0094u32 as i32;
pub const STATUS_INTEGER_OVERFLOW: i32 = 0xC000_0095u32 as i32;
pub const STATUS_GUARD_PAGE_VIOLATION: i32 = 0x8000_0001u32 as i32;
pub const STATUS_IN_PAGE_ERROR: i32 = 0xC000_0006u32 as i32;
pub const STATUS_ARRAY_BOUNDS_EXCEEDED: i32 = 0xC000_008Cu32 as i32;
pub const STATUS_STACK_OVERFLOW: i32 = 0xC000_00FDu32 as i32;
pub const STATUS_STACK_BUFFER_OVERRUN: i32 = 0xC000_0409u32 as i32;
pub const STATUS_INVALID_LOCK_SEQUENCE: i32 = 0xC000_001Eu32 as i32;
pub const STATUS_INVALID_LDT_DESCRIPTOR: i32 = 0xC000011A_u32 as i32;
pub const STATUS_MUTANT_LIMIT_EXCEEDED: i32 = 0xC000_0191u32 as i32;
pub const STATUS_NO_YIELD_PERFORMED: i32 = 0x4000_0024u32 as i32;
// STATUS_ACCESS_VIOLATION and STATUS_UNSUCCESSFUL are in ntkrnl_sys::constants
pub const KMODE_EXCEPTION_NOT_HANDLED: u32 = 0x0000_001E;
// 0x8E, NOT 0x1E — KMODE_EXCEPTION_NOT_HANDLED (above) is the 0x1E one. Both
// names were declared 0x1E here; the second was invented (§9.113).
pub const KERNEL_MODE_EXCEPTION_NOT_HANDLED: u32 = 0x0000_008E;

// KF_ feature flags — from ndk/i386/ketypes.h. The low bits agree with
// ndk/amd64/ketypes.h but the Win 6.x+ bits do NOT (KF_XSTATE, KF_SSSE3/SSE4_x,
// KF_DTS, KF_RDRAND move), so the arch-dependent ones used here are cfg-split.
pub const KF_V86_VIS: u32 = 0x0000_0001;
pub const KF_RDTSC: u32 = 0x0000_0002;
pub const KF_CR4: u32 = 0x0000_0004;
pub const KF_CMOV: u32 = 0x0000_0008;
pub const KF_GLOBAL_PAGE: u32 = 0x0000_0010;
pub const KF_LARGE_PAGE: u32 = 0x0000_0020;
pub const KF_MTRR: u32 = 0x0000_0040;
pub const KF_CMPXCHG8B: u32 = 0x0000_0080;
pub const KF_MMX: u32 = 0x0000_0100;
pub const KF_WORKING_PTE: u32 = 0x0000_0200;
pub const KF_PAT: u32 = 0x0000_0400;
pub const KF_FXSR: u32 = 0x0000_0800;
pub const KF_FAST_SYSCALL: u32 = 0x0000_1000;
pub const KF_XMMI: u32 = 0x0000_2000;
pub const KF_3DNOW: u32 = 0x0000_4000;
pub const KF_AMDK6MTRR: u32 = 0x0000_8000;
pub const KF_XMMI64: u32 = 0x0001_0000;
pub const KF_DTS: u32 = 0x0002_0000;
pub const KF_SSE3: u32 = 0x0008_0000;
// 64-bit feature bits: ndk/i386/ketypes.h:66-68 / ndk/amd64/ketypes.h:72-74.
#[cfg(target_arch = "x86")]
pub const KF_SSSE3: u64 = 0x0000_0040_0000_0000;
#[cfg(target_arch = "x86")]
pub const KF_SSE4_1: u64 = 0x0000_0080_0000_0000;
#[cfg(target_arch = "x86")]
pub const KF_SSE4_2: u64 = 0x0000_0100_0000_0000;
#[cfg(not(target_arch = "x86"))]
pub const KF_SSSE3: u64 = 0x0000_8000_0000_0000;
#[cfg(not(target_arch = "x86"))]
pub const KF_SSE4_1: u64 = 0x0001_0000_0000_0000;
#[cfg(not(target_arch = "x86"))]
pub const KF_SSE4_2: u64 = 0x0002_0000_0000_0000;
// ndk/i386/ketypes.h:53 = 0x400000; ndk/amd64/ketypes.h:55 = 0x800000.
#[cfg(target_arch = "x86")]
pub const KF_XSTATE: u32 = 0x0040_0000;
#[cfg(not(target_arch = "x86"))]
pub const KF_XSTATE: u32 = 0x0080_0000;
pub const KF_RDRAND: u32 = 0x0200_0000;
pub const KF_NX_BIT: u32 = 0x2000_0000;
pub const KF_NX_DISABLED: u32 = 0x4000_0000;
pub const KF_NX_ENABLED: u32 = 0x8000_0000;

// XSTATE constants
pub const XSTATE_LEGACY_FLOATING_POINT: u32 = 0;
pub const XSTATE_LEGACY_SSE: u32 = 1;
pub const MAXIMUM_XSTATE_FEATURES: u32 = 64;
// xdk/ketypes.h:1296-1338. ALLOWED = LEGACY|AVX|MPX|AVX512|IPT|PASID|CET_U|
// AMX_TILE_CONFIG|AMX_TILE_DATA|LWP on _AMD64_ (no PASID/AMX on _X86_).
#[cfg(target_arch = "x86")]
pub const XSTATE_MASK_ALLOWED: u64 = 0x4000_0000_0000_09FF;
#[cfg(not(target_arch = "x86"))]
pub const XSTATE_MASK_ALLOWED: u64 = 0x4000_0000_0006_0DFF;
pub const XSTATE_MASK_PERSISTENT: u64 = 0x4000_0000_0000_0010; // MPX_BNDCSR | LWP
pub const XSTATE_MASK_USER_VISIBLE_SUPERVISOR: u64 = 0x0000_0800; // CET_U
pub const XSTATE_MASK_LARGE_FEATURES: u64 = 0x0004_0000; // AMX_TILE_DATA
pub const XSTATE_MASK_CET_U: u64 = 0x0000_0800;
pub const XSTATE_MASK_CET_S: u64 = 0x0000_1000;
pub const XSTATE_MASK_IPT: u64 = 0x0000_0100;
pub const XSTATE_MASK_PASID: u64 = 0x0000_0400;

// Lock queue numbers (from sdk/include/xdk/ketypes.h)
// NOTE: LockQueueNonPagedPoolLock (6) and LockQueueMmNonPagedPoolLock (15)
// are defined in ntkrnl_sys::base and imported via `use ntkrnl_sys::base::*`.
pub const LockQueueDispatcherLock: u32 = 0;
pub const LockQueueExpansionLock: u32 = 1;
pub const LockQueuePfnLock: u32 = 2;
pub const LockQueueSystemSpaceLock: u32 = 3;
pub const LockQueueVacbLock: u32 = 4;
pub const LockQueueMasterLock: u32 = 5;
pub const LockQueueIoCancelLock: u32 = 7;
pub const LockQueueWorkQueueLock: u32 = 8;
pub const LockQueueIoVpbLock: u32 = 9;
pub const LockQueueIoDatabaseLock: u32 = 10;
pub const LockQueueIoCompletionLock: u32 = 11;
pub const LockQueueNtfsStructLock: u32 = 12;
pub const LockQueueAfdWorkQueueLock: u32 = 13;
pub const LockQueueBcbLock: u32 = 14;
pub const LockQueueUnusedSpare16: u32 = 16;
pub const LockQueueTimerTableLock: u32 = 17;

// Process states
pub const ProcessInMemory: u32 = 0;

// Thread states
pub const Running: u32 = 2;

// Dispatcher object types (KOBJECTS enum values)
pub const SemaphoreObject: u32 = 5;

// Pool tags
pub const TAG_KERNEL: u32 = 0x2020654B;

// Processor modes — KernelMode and UserMode are in ntkrnl_sys::constants

// Missing type aliases
// NOTE: ULONG64 is provided by ntkrnl_sys::mm (or ntkrnl_sys::ddk on arm64/i386).
// KCONTINUE_STATUS and Continue* constants are provided by ntkrnl_sys::globals.
// Do NOT re-export them here to avoid ambiguity with glob imports.
pub type KPRIORITY = i32;
pub type CCHAR = i8;
pub type ULARGE_INTEGER = ntkrnl_sys::ddk::_ULARGE_INTEGER;
pub type FAST_MUTEX = ntkrnl_sys::ddk::_FAST_MUTEX;
pub type CONTEXT = ntkrnl_sys::ddk::_CONTEXT;
pub type HANDLE = ntkrnl_sys::base::PVOID;
pub type KPROCESSOR_MODE = i8;

// CONTEXT flags (i386-specific)
#[cfg(target_arch = "x86")]
pub const CONTEXT_FULL: u32 = 0x0001_0007;
#[cfg(target_arch = "x86")]
pub const CONTEXT_DEBUG_REGISTERS: u32 = 0x0001_0010;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_FULL: u32 = 0x0010_000B;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_DEBUG_REGISTERS: u32 = 0x0010_0010;

// GDT selectors (i386)
#[cfg(target_arch = "x86")]
pub const KGDT_LDT: u16 = 0x0048;
#[cfg(target_arch = "x86")]
pub const IO_ACCESS_MAP_NONE: u16 = 0;

// KWAIT_REASON enum values (with underscore prefix as bindgen generates them)
pub const _KWAIT_REASON_WrQuantumEnd: u32 = 30;
pub const _KWAIT_REASON_WrQueue: u32 = 15;
pub const _KWAIT_REASON_WrMutex: u32 = 5;
pub const _KWAIT_REASON_WrGuardedMutex: u32 = 35;
pub const _KWAIT_REASON_WrYieldExecution: u32 = 36;

// High importance for DPC
pub const HighImportance: u32 = 2;

// Bugcheck code
pub const NO_PAGES_AVAILABLE: u32 = 0x0000_004D;

// KeGetCurrentThread wrapper — calls PsGetCurrentThread
#[inline(always)]
pub unsafe fn KeGetCurrentThread() -> *mut ntkrnl_sys::ddk::_KTHREAD {
    PsGetCurrentThread()
}

// ============================================================================
// KTHREAD field accessors — architecture-dependent union nesting
// ============================================================================
//
// KTHREAD has different layouts on i386 vs amd64/arm64.
// For arch-specific files (ke/i386/, ke/amd64/, ke/arm64/), use direct
// bindgen union paths. For generic ke/ files, use cfg-gated code.
//
// Quick reference for sed-based fixups:
//
// Generic files (need cfg):
//   .ApcState       → all arches: .__bindgen_anon_1.ApcState
//   .WaitListEntry  → all arches: .__bindgen_anon_3.WaitListEntry
//   .NextProcessor  → all arches: .__bindgen_anon_1.__bindgen_anon_1.NextProcessor
//
// i386-only (direct): .State, .NpxState, .WaitIrql, .WaitMode, .WaitTime,
//   .ContextSwitches, .UserTime
//
// amd64/arm64 paths:
//   .State/.NpxState/.WaitIrql/.WaitMode → .__bindgen_anon_5.__bindgen_anon_6.Field
//   .Quantum → .__bindgen_anon_7.__bindgen_anon_1.Quantum
//   .QuantumReset → .__bindgen_anon_7.__bindgen_anon_2.QuantumReset
//   .PreviousMode → .__bindgen_anon_5.__bindgen_anon_2.PreviousMode
//   .WaitTime → .__bindgen_anon_5.__bindgen_anon_7.WaitTime
//   .ContextSwitches → .__bindgen_anon_5.__bindgen_anon_5.ContextSwitches
//   .SpecialApcDisable → .__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable
//   .CombinedApcDisable → .__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable
//   .KernelTime → .__bindgen_anon_7.__bindgen_anon_3.KernelTime
//   .UserTime → .__bindgen_anon_7.__bindgen_anon_6.UserTime
//   .WaitBlock → .__bindgen_anon_5.WaitBlock
//   .Timer → .__bindgen_anon_4.Timer
//
// i386 paths (different union numbers):
//   .Quantum → .__bindgen_anon_9.__bindgen_anon_1.Quantum
//   .QuantumReset → .__bindgen_anon_9.__bindgen_anon_2.QuantumReset
//   .PreviousMode → .__bindgen_anon_7.__bindgen_anon_2.PreviousMode
//   .SpecialApcDisable → .__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable
//   .CombinedApcDisable → .__bindgen_anon_4.__bindgen_anon_1.CombinedApcDisable
//   .KernelTime → .__bindgen_anon_9.__bindgen_anon_3.KernelTime
//   .WaitBlock → .__bindgen_anon_7.WaitBlock
//   .Timer → .__bindgen_anon_5.Timer

// CONTAINING_RECORD macro — convert a list entry pointer to a containing struct pointer
#[macro_export]
macro_rules! CONTAINING_RECORD {
    ($ptr:expr, $type:ty, $($field:tt).+) => {
        ($ptr as *const u8).sub(core::mem::offset_of!($type, $($field).+)) as *mut $type
    };
}

// ---- Unsafe accessor functions for KTHREAD fields ----
// These return mutable references to fields through the correct union path.
use ntkrnl_sys::ddk::_KTHREAD;
use ntkrnl_sys::ddk::_KAPC_STATE;
use ntkrnl_sys::ddk::_LIST_ENTRY;
use ntkrnl_sys::ddk::_KWAIT_BLOCK;
use ntkrnl_sys::ddk::_KTIMER;

#[inline(always)]
pub unsafe fn kt_ApcState(t: *mut _KTHREAD) -> &'static mut _KAPC_STATE {
    &mut (*t).__bindgen_anon_1.ApcState
}

#[inline(always)]
pub unsafe fn kt_WaitListEntry(t: *mut _KTHREAD) -> &'static mut _LIST_ENTRY {
    &mut (*t).__bindgen_anon_3.WaitListEntry
}

#[inline(always)]
pub unsafe fn kt_State(t: *mut _KTHREAD) -> &'static mut u8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).State }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_6.State }
}

#[inline(always)]
pub unsafe fn kt_NpxState(t: *mut _KTHREAD) -> &'static mut u8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).NpxState }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_6.NpxState }
}

#[inline(always)]
pub unsafe fn kt_WaitIrql(t: *mut _KTHREAD) -> &'static mut u8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).WaitIrql }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_6.WaitIrql }
}

#[inline(always)]
pub unsafe fn kt_WaitMode(t: *mut _KTHREAD) -> &'static mut i8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).WaitMode }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_6.WaitMode }
}

#[inline(always)]
pub unsafe fn kt_Quantum(t: *mut _KTHREAD) -> &'static mut i8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_9.__bindgen_anon_1.Quantum }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_1.Quantum }
}

#[inline(always)]
pub unsafe fn kt_QuantumReset(t: *mut _KTHREAD) -> &'static mut u8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_9.__bindgen_anon_2.QuantumReset }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_2.QuantumReset }
}

#[inline(always)]
pub unsafe fn kt_PreviousMode(t: *mut _KTHREAD) -> &'static mut i8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_2.PreviousMode }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_2.PreviousMode }
}

#[inline(always)]
pub unsafe fn kt_WaitTime(t: *mut _KTHREAD) -> &'static mut u32 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).WaitTime }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_7.WaitTime }
}

#[inline(always)]
pub unsafe fn kt_ContextSwitches(t: *mut _KTHREAD) -> &'static mut u32 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).ContextSwitches }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_5.ContextSwitches }
}

#[inline(always)]
pub unsafe fn kt_SpecialApcDisable(t: *mut _KTHREAD) -> &'static mut i16 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable }
}

#[inline(always)]
pub unsafe fn kt_CombinedApcDisable(t: *mut _KTHREAD) -> &'static mut u32 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_4.CombinedApcDisable }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.CombinedApcDisable }
}

#[inline(always)]
pub unsafe fn kt_KernelTime(t: *mut _KTHREAD) -> &'static mut u32 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_9.__bindgen_anon_3.KernelTime }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_3.KernelTime }
}

#[inline(always)]
pub unsafe fn kt_UserTime(t: *mut _KTHREAD) -> &'static mut u32 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).UserTime }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_6.UserTime }
}

#[inline(always)]
pub unsafe fn kt_WaitBlock(t: *mut _KTHREAD) -> &'static mut [_KWAIT_BLOCK; 4] {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_7.WaitBlock }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.WaitBlock }
}

#[inline(always)]
pub unsafe fn kt_Timer(t: *mut _KTHREAD) -> &'static mut _KTIMER {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_5.Timer }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_4.Timer }
}

#[inline(always)]
pub unsafe fn kt_SavedApcState(t: *mut _KTHREAD) -> &'static mut _KAPC_STATE {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_8.SavedApcState }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_6.SavedApcState }
}

#[inline(always)]
pub unsafe fn kt_LargeStack(t: *mut _KTHREAD) -> &'static mut u8 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_4.LargeStack }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_4.LargeStack }
}

// --- Additional KTHREAD field accessors for apc.rs ---

use ntkrnl_sys::ddk::_KGATE;

#[inline(always)]
pub unsafe fn kt_KernelApcDisable(t: *mut _KTHREAD) -> &'static mut i16 {
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_4.__bindgen_anon_1.KernelApcDisable }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.KernelApcDisable }
}

#[inline(always)]
pub unsafe fn kt_GateObject(t: *mut _KTHREAD) -> *mut _KGATE {
    // All arches: __bindgen_anon_2.GateObject
    (*t).__bindgen_anon_2.GateObject
}

#[inline(always)]
pub unsafe fn kt_NextProcessor(t: *mut _KTHREAD) -> &'static mut u8 {
    // All arches: __bindgen_anon_1.__bindgen_anon_1.NextProcessor
    &mut (*t).__bindgen_anon_1.__bindgen_anon_1.NextProcessor
}

#[inline(always)]
pub unsafe fn kt_DeferredProcessor(t: *mut _KTHREAD) -> &'static mut u8 {
    // All arches: __bindgen_anon_1.__bindgen_anon_1.DeferredProcessor
    &mut (*t).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor
}

/// Read DisableBoost bitfield from KTHREAD (returns ULONG, 0 or 1).
#[inline(always)]
pub unsafe fn kt_DisableBoost(t: *const _KTHREAD) -> u32 {
    #[cfg(target_arch = "x86")]
    { (*t).__bindgen_anon_6.__bindgen_anon_1.DisableBoost() }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.DisableBoost() }
}

// --- Additional KTHREAD field accessors for thrdobj.rs ---

use ntkrnl_sys::ddk::_KSEMAPHORE;
use ntkrnl_sys::ddk::_KAPC;

#[inline(always)]
pub unsafe fn kt_ApcQueueable(t: *mut _KTHREAD) -> &'static mut u8 {
    // All arches: __bindgen_anon_1.__bindgen_anon_1.ApcQueueable
    &mut (*t).__bindgen_anon_1.__bindgen_anon_1.ApcQueueable
}

#[inline(always)]
pub unsafe fn kt_AdjustReason(t: *mut _KTHREAD) -> &'static mut u8 {
    // All arches: __bindgen_anon_1.__bindgen_anon_1.AdjustReason
    &mut (*t).__bindgen_anon_1.__bindgen_anon_1.AdjustReason
}

#[inline(always)]
pub unsafe fn kt_AdjustIncrement(t: *mut _KTHREAD) -> &'static mut i8 {
    // All arches: __bindgen_anon_1.__bindgen_anon_1.AdjustIncrement
    &mut (*t).__bindgen_anon_1.__bindgen_anon_1.AdjustIncrement
}

#[inline(always)]
pub unsafe fn kt_SuspendCount(t: *mut _KTHREAD) -> &'static mut i8 {
    // i386: direct field SuspendCount
    // amd64/arm64: __bindgen_anon_6.__bindgen_anon_1.SuspendCount
    #[cfg(target_arch = "x86")]
    { &mut (*t).SuspendCount }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_6.__bindgen_anon_1.SuspendCount }
}

#[inline(always)]
pub unsafe fn kt_FreezeCount(t: *mut _KTHREAD) -> &'static mut i8 {
    // i386: __bindgen_anon_8.__bindgen_anon_1.FreezeCount
    // amd64/arm64: __bindgen_anon_6.__bindgen_anon_1.FreezeCount
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_8.__bindgen_anon_1.FreezeCount }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_6.__bindgen_anon_1.FreezeCount }
}

#[inline(always)]
pub unsafe fn kt_UserIdealProcessor(t: *mut _KTHREAD) -> &'static mut u8 {
    // i386: direct field UserIdealProcessor
    // amd64/arm64: __bindgen_anon_6.__bindgen_anon_1.UserIdealProcessor
    #[cfg(target_arch = "x86")]
    { &mut (*t).UserIdealProcessor }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_6.__bindgen_anon_1.UserIdealProcessor }
}

#[inline(always)]
pub unsafe fn kt_SuspendApc(t: *mut _KTHREAD) -> &'static mut _KAPC {
    // i386: __bindgen_anon_9.SuspendApc
    // amd64/arm64: __bindgen_anon_7.SuspendApc
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_9.SuspendApc }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_7.SuspendApc }
}

#[inline(always)]
pub unsafe fn kt_SuspendSemaphore(t: *mut _KTHREAD) -> &'static mut _KSEMAPHORE {
    // i386: __bindgen_anon_10.SuspendSemaphore
    // amd64/arm64: __bindgen_anon_8.SuspendSemaphore
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_10.SuspendSemaphore }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_8.SuspendSemaphore }
}

#[inline(always)]
pub unsafe fn kt_SystemAffinityActive(t: *mut _KTHREAD) -> &'static mut u8 {
    // i386: __bindgen_anon_7.__bindgen_anon_1.SystemAffinityActive
    // amd64/arm64: __bindgen_anon_5.__bindgen_anon_1.SystemAffinityActive
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_7.__bindgen_anon_1.SystemAffinityActive }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_5.__bindgen_anon_1.SystemAffinityActive }
}

#[inline(always)]
pub unsafe fn kt_ThreadFlags(t: *mut _KTHREAD) -> &'static mut i32 {
    // i386: __bindgen_anon_6.ThreadFlags
    // amd64/arm64: __bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.ThreadFlags (same as LONG)
    #[cfg(target_arch = "x86")]
    { &mut (*t).__bindgen_anon_6.ThreadFlags }
    #[cfg(not(target_arch = "x86"))]
    { &mut (*t).__bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.ThreadFlags }
}

/// DisableBoost is a bitfield on _KTHREAD. Returns the ThreadFlags union.
/// Use set_DisableBoost(val) / DisableBoost() on the bitfield struct.
#[inline(always)]
pub unsafe fn kt_ThreadFlags_bitfield(t: *mut _KTHREAD) -> *mut core::ffi::c_void {
    // i386: __bindgen_anon_6.__bindgen_anon_1
    // amd64/arm64: __bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1
    #[cfg(target_arch = "x86")]
    { core::ptr::addr_of_mut!((*t).__bindgen_anon_6.__bindgen_anon_1) as *mut core::ffi::c_void }
    #[cfg(not(target_arch = "x86"))]
    { core::ptr::addr_of_mut!((*t).__bindgen_anon_4.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1) as *mut core::ffi::c_void }
}

/// KPROCESS.DisableBoost is a bitfield in __bindgen_anon_1.__bindgen_anon_1
#[inline(always)]
pub unsafe fn kp_ProcessFlags(p: *mut ntkrnl_sys::ddk::_KPROCESS) -> &'static mut i32 {
    &mut (*p).__bindgen_anon_1.ProcessFlags
}
