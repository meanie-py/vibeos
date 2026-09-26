//! Umbrella staticlib that combines all Rust kernel components into a single
//! archive for linking into ntoskrnl. Each component is an rlib dependency;
//! this crate provides the single panic_handler and pulls in all symbols.

#![no_std]

extern crate ke_inlines;
extern crate ke_macros;
// Re-export ntoskrnl-rust (ob subsystem) so its #[no_mangle] symbols are linked in.
extern crate ntoskrnl_rust;
// lpc subsystem, extracted from ntoskrnl-rust into its own crate (crate-split
// PoC). extern crate forces its #[no_mangle] Nt*/Lpcp* symbols into the archive
// (a Cargo dep alone would drop them) — same pattern as cmboot_rust below.
extern crate lpc_rust;
// fsrtl subsystem, extracted from ntoskrnl-rust (crate-split). extern crate forces
// its #[no_mangle] FsRtl* symbols into the archive — same pattern as lpc_rust.
extern crate fsrtl_rust;
// dbgk + wmi subsystems, extracted from ntoskrnl-rust (crate-split).
extern crate dbgk_rust;
extern crate wmi_rust;
// cc (Cache Controller), extracted from ntoskrnl-rust (crate-split Phase 1b).
// extern crate forces its #[no_mangle] Cc* symbols + _cc_global_write_barrier
// into the archive — same pattern as lpc_rust/fsrtl_rust.
extern crate cc_rust;
// fstub (I/O disk-stub / partition + HAL dispatch tables) + se (Security Reference
// Monitor), extracted from ntoskrnl-rust (crate-split Phase 1b). extern crate forces
// their #[no_mangle] symbols + per-crate write barriers into the archive.
extern crate fstub_rust;
extern crate se_rust;
// pnpmgr (Plug-and-Play manager), extracted from ntoskrnl-rust (crate-split Phase 1b).
extern crate pnpmgr_rust;
// iomgr (I/O Manager), the LARGEST subsystem, extracted from ntoskrnl-rust (crate-split
// Phase 1b). extern crate forces its 344 #[no_mangle] Io*/Iop* symbols + the 7 Io
// object-type singletons + _iomgr_global_write_barrier into the archive.
extern crate iomgr_rust;
// kdbg (integrated kernel debugger KDBG), extracted from ntoskrnl-rust (crate-split
// Phase 1b). x86-ONLY — the crate is empty on arm64 (crate-level #![cfg]), and this
// extern crate is cfg-gated out there so arm64 never references it. extern crate
// forces its #[no_mangle] Kdb*/Kdp* symbols + _kdbg_global_write_barrier into the
// archive (a Cargo dep alone would drop them).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern crate kdbg_rust;
// config (Configuration Manager / registry), extracted from ntoskrnl-rust (crate-split
// Phase 1b). extern crate forces its #[no_mangle] Cm*/Cmp* symbols + the runtime
// CmControlVector assembler + _config_global_write_barrier into the archive.
extern crate config_rust;
// ex (Executive + pool), the largest subsystem, extracted from ntoskrnl-rust (crate-split Phase 1b).
extern crate ex_rust;
// ob (Object Manager), extracted from ntoskrnl-rust (crate-split Phase 1b).
extern crate ob_rust;
// ps (Process/Thread Manager), extracted from ntoskrnl-rust (crate-split Phase 1b).
extern crate ps_rust;
// kd (kernel-debug I/O), kd64 (KD protocol stack), vf (Driver Verifier), rtlk
// (kernel RTL glue), extracted from ntoskrnl-rust (crate-split Phase 1b).
extern crate kd_rust;
extern crate kd64_rust;
extern crate vf_rust;
extern crate rtlk_rust;
// po (Power Manager), inbv (boot video), vdm (16-bit/VDM), extracted from
// ntoskrnl-rust (crate-split Phase 1b). extern crate pulls their #[no_mangle]
// symbols + per-crate barriers into the archive. vdm is i386-only (empty rlib
// elsewhere), so its extern crate is cfg-gated to match.
extern crate po_rust;
extern crate inbv_rust;
#[cfg(target_arch = "x86")]
extern crate vdm_rust;
// cmboot.c is the shared kernel+bootloader CM boot-init unit, moved out of
// ntoskrnl-rust into its own crate (cmboot-rust) so the loaders can link the
// SAME translation. Pull its #[no_mangle] CmpFind*/CmpAddDriverToList symbols
// into libkernel_rust.a here (extern crate is required for a staticlib umbrella;
// a Cargo dep alone would compile the rlib but drop its symbols).
extern crate cmboot_rust;
// Re-export ntprintf so its FFI symbols (ntprintf_scan_arg_types,
// ntprintf_format_a) are linked in for the C variadic trampolines.
extern crate ntprintf;
// Re-export ntprintf_w (UTF-16 sibling) so its FFI symbols
// (ntprintf_w_scan_arg_types, ntprintf_w_format) are linked in for the
// wide variadic trampolines (RtlStringCbPrintfW, swprintf).
extern crate ntprintf_w;

// Defensive symbol pull — make sure rustc/lld don't DCE the ntprintf FFI
// entries. Same pattern as `_force_kdbg_symbols`.
mod _force_ntprintf_symbols {
    extern "C" {
        fn ntprintf_scan_arg_types(fmt: *const u8, out: *mut u8, max: usize) -> i32;
        fn ntprintf_format_a(dst: *mut u8, cap: usize, fmt: *const u8,
                             slots: *const u64, n: usize) -> usize;
    }
    #[used]
    static REF_SCAN: unsafe extern "C" fn(*const u8, *mut u8, usize) -> i32 =
        ntprintf_scan_arg_types;
    #[used]
    static REF_FORMAT: unsafe extern "C" fn(*mut u8, usize, *const u8, *const u64, usize) -> usize =
        ntprintf_format_a;
}

// Defensive symbol pull for the cm-inlines KCB-lock Rust impls. cm-inlines
// now exports the bare `Cmp*` names directly (no `_rs_` prefix, no C
// trampoline) — Rust callers in `config/cmkcbncb.rs` resolve via the
// bindgen-emitted `extern "C" { fn Cmp*(...) }` decls.
//
// 2026-05-07: extended to include all of cm_rust_shims.c (shared KCB locks,
// NCB locks, HHIVE function-pointer dispatch, empty-stub assertions, trivial
// returns, CmpIsKeyValueSmall). The C file is gone from the build.
mod _force_cm_inlines_symbols {
    use core::ffi::c_void;
    extern "C" {
        fn CmpAcquireKcbLockExclusive(kcb: *mut c_void);
        fn CmpAcquireKcbLockExclusiveByIndex(index: u32);
        fn CmpAcquireKcbLockExclusiveByKey(conv_key: u32);
        fn CmpReleaseKcbLock(kcb: *mut c_void);
        fn CmpReleaseKcbLockByIndex(index: u32);
        fn CmpReleaseKcbLockByKey(conv_key: u32);
        fn CmpTryToConvertKcbSharedToExclusive(kcb: *mut c_void) -> u8;
        fn CmpConvertKcbSharedToExclusive(kcb: *mut c_void);
        // 2026-05-07 additions (former cm_rust_shims.c):
        fn CmpAcquireKcbLockShared(kcb: *mut c_void);
        fn CmpAcquireKcbLockSharedByIndex(index: u32);
        fn CmpAcquireNcbLockExclusiveByKey(conv_key: u32);
        fn CmpReleaseNcbLockByKey(conv_key: u32);
        fn CmpIsKcbLockedExclusive(kcb: *mut c_void) -> u8;
        fn CmpIsKeyValueSmall(real_length: *mut u32, length: u32) -> u8;
        fn HvGetCell(hive: *mut c_void, cell: u32) -> *mut c_void;
        fn HvReleaseCell(hive: *mut c_void, cell: u32);
        fn HvGetCellType(cell: u32) -> u32;
        fn CMP_ASSERT_REGISTRY_LOCK();
        fn CMP_ASSERT_FLUSH_LOCK(h: *mut c_void);
        fn CMP_ASSERT_KCB_LOCK(k: *mut c_void);
        fn CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK();
        fn CMP_ASSERT_REGISTRY_LOCK_OR_LOADING(h: *mut c_void);
        fn ASSERT_VALUE_BIG(h: *mut c_void, l: u32);
        fn CMTRACE(filter: u32, format: *const i8, arg1: *const i8, arg2: *mut c_void);
        fn IS_NULL_TERMINATED(s: *mut u16, l: u32) -> u8;
        fn LANGIDFROMLCID(lcid: u32) -> u32;
        fn PtrToUlong(p: *mut c_void) -> u32;
        fn GET_HASH_INDEX(conv_key: u32) -> u32;
    }
    #[used]
    static REFS_PTR: [unsafe extern "C" fn(*mut c_void); 6] = [
        CmpAcquireKcbLockExclusive,
        CmpReleaseKcbLock,
        CmpConvertKcbSharedToExclusive,
        CmpAcquireKcbLockShared,
        CMP_ASSERT_FLUSH_LOCK,
        CMP_ASSERT_KCB_LOCK,
    ];
    #[used]
    static REFS_INDEX: [unsafe extern "C" fn(u32); 6] = [
        CmpAcquireKcbLockExclusiveByIndex,
        CmpAcquireKcbLockExclusiveByKey,
        CmpReleaseKcbLockByIndex,
        CmpReleaseKcbLockByKey,
        CmpAcquireKcbLockSharedByIndex,
        CmpAcquireNcbLockExclusiveByKey,
    ];
    #[used]
    static REF_TRY: unsafe extern "C" fn(*mut c_void) -> u8 =
        CmpTryToConvertKcbSharedToExclusive;
    #[used]
    static REFS_NOARG: [unsafe extern "C" fn(); 2] = [
        CMP_ASSERT_REGISTRY_LOCK,
        CMP_ASSERT_EXCLUSIVE_REGISTRY_LOCK,
    ];
    #[used]
    static REF_HV_GET: unsafe extern "C" fn(*mut c_void, u32) -> *mut c_void = HvGetCell;
    #[used]
    static REF_HV_RELEASE: unsafe extern "C" fn(*mut c_void, u32) = HvReleaseCell;
    #[used]
    static REF_HV_TYPE: unsafe extern "C" fn(u32) -> u32 = HvGetCellType;
    #[used]
    static REF_KCB_LOCKED: unsafe extern "C" fn(*mut c_void) -> u8 = CmpIsKcbLockedExclusive;
    #[used]
    static REF_KEYVAL_SMALL: unsafe extern "C" fn(*mut u32, u32) -> u8 = CmpIsKeyValueSmall;
    #[used]
    static REF_RELEASE_NCB: unsafe extern "C" fn(u32) = CmpReleaseNcbLockByKey;
    #[used]
    static REF_LANGID: unsafe extern "C" fn(u32) -> u32 = LANGIDFROMLCID;
    #[used]
    static REF_GET_HASH: unsafe extern "C" fn(u32) -> u32 = GET_HASH_INDEX;
    #[used]
    static REF_PTR_TO_ULONG: unsafe extern "C" fn(*mut c_void) -> u32 = PtrToUlong;
    #[used]
    static REF_IS_NULL_TERM: unsafe extern "C" fn(*mut u16, u32) -> u8 = IS_NULL_TERMINATED;
    #[used]
    static REF_REG_LOCK_LOAD: unsafe extern "C" fn(*mut c_void) = CMP_ASSERT_REGISTRY_LOCK_OR_LOADING;
    #[used]
    static REF_VAL_BIG: unsafe extern "C" fn(*mut c_void, u32) = ASSERT_VALUE_BIG;
    #[used]
    static REF_CMTRACE: unsafe extern "C" fn(u32, *const i8, *const i8, *mut c_void) = CMTRACE;
}

// Defensive symbol pull for the ke-inlines APC/Process/DeviceQueue lock
// Rust impls. C-side `ke_rust_shims.c` trampolines call these by name;
// without `[used]` rustc/lld may strip them from the staticlib. Same
// no-arg-extern-decl trick as `_force_shim_symbols` — the array element
// type is a *symbol reference*, not a real signature.
mod _force_ke_inlines_symbols {
    extern "C" {
        fn _rs_KiAcquireApcLockRaiseToSynch();
        fn _rs_KiAcquireApcLockAtSynchLevel();
        fn _rs_KiAcquireApcLockRaiseToDpc();
        fn _rs_KiReleaseApcLock();
        fn _rs_KiReleaseApcLockFromSynchLevel();
        fn _rs_KiReleaseProcessLockFromSynchLevel();
        fn _rs_KiAcquireDeviceQueueLock();
        fn _rs_KiReleaseDeviceQueueLock();
        fn _rs_KiAcquireDispatcherObject();
        fn _rs_KiReleaseDispatcherObject();
        fn _rs_KiAcquireDispatcherLock();
        fn _rs_KiReleaseDispatcherLock();
        fn _rs_KiAcquireDispatcherLockAtSynchLevel();
        fn _rs_KiReleaseDispatcherLockFromSynchLevel();
        fn _rs_KiAcquireThreadLock();
        fn _rs_KiReleaseThreadLock();
        fn _rs_KiTryThreadLock();
        fn _rs_KiAcquirePrcbLock();
        fn _rs_KiReleasePrcbLock();
        fn _rs_KiInsertDeferredReadyList();
        fn _rs_KiRescheduleThread();
        fn _rs_KiSetThreadSwapBusy();
        fn _rs_KiCheckDeferredReadyList();
        fn _rs_KiRequestApcInterrupt();
        fn _rs_KiAcquireTimerLock();
        fn _rs_KiReleaseTimerLock();
        fn _rs_KiComputeTimerTableIndex();
        fn _rs_KxAcquireSpinLock();
        fn _rs_KxReleaseSpinLock();
        fn _rs_KiAcquireNmiListLock();
        fn _rs_KiReleaseNmiListLock();
        fn _rs_KiCheckThreadStackSwap();
        fn _rs_KiCheckAlertability();
        fn _rs_KiSelectReadyThread();
        fn _rs_KiComputeNewPriority();
        fn _rs_KxUnwaitThread();
        fn _rs_KxUnwaitThreadForEvent();
        fn _rs_KxQueueReadyThread();
        fn _rs__KeInitializeGuardedMutex();
        fn _rs__KeAcquireGuardedMutexUnsafe();
        fn _rs__KeReleaseGuardedMutexUnsafe();
        fn _rs__KeAcquireGuardedMutex();
        fn _rs__KeReleaseGuardedMutex();
        fn _rs__KeTryToAcquireGuardedMutex();
        fn _rs_KiRecalculateDueTime();
        fn _rs_KxInsertTimer();
        fn _rs_KiComputeDueTime();
        fn _rs_KxRemoveTreeTimer();
        fn _rs_KxSetTimerForThreadWait();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 49] = [
        _rs_KiAcquireApcLockRaiseToSynch,
        _rs_KiAcquireApcLockAtSynchLevel,
        _rs_KiAcquireApcLockRaiseToDpc,
        _rs_KiReleaseApcLock,
        _rs_KiReleaseApcLockFromSynchLevel,
        _rs_KiReleaseProcessLockFromSynchLevel,
        _rs_KiAcquireDeviceQueueLock,
        _rs_KiReleaseDeviceQueueLock,
        _rs_KiAcquireDispatcherObject,
        _rs_KiReleaseDispatcherObject,
        _rs_KiAcquireDispatcherLock,
        _rs_KiReleaseDispatcherLock,
        _rs_KiAcquireDispatcherLockAtSynchLevel,
        _rs_KiReleaseDispatcherLockFromSynchLevel,
        _rs_KiAcquireThreadLock,
        _rs_KiReleaseThreadLock,
        _rs_KiTryThreadLock,
        _rs_KiAcquirePrcbLock,
        _rs_KiReleasePrcbLock,
        _rs_KiInsertDeferredReadyList,
        _rs_KiRescheduleThread,
        _rs_KiSetThreadSwapBusy,
        _rs_KiCheckDeferredReadyList,
        _rs_KiRequestApcInterrupt,
        _rs_KiAcquireTimerLock,
        _rs_KiReleaseTimerLock,
        _rs_KiComputeTimerTableIndex,
        _rs_KxAcquireSpinLock,
        _rs_KxReleaseSpinLock,
        _rs_KiAcquireNmiListLock,
        _rs_KiReleaseNmiListLock,
        _rs_KiCheckThreadStackSwap,
        _rs_KiCheckAlertability,
        _rs_KiSelectReadyThread,
        _rs_KiComputeNewPriority,
        _rs_KxUnwaitThread,
        _rs_KxUnwaitThreadForEvent,
        _rs_KxQueueReadyThread,
        _rs__KeInitializeGuardedMutex,
        _rs__KeAcquireGuardedMutexUnsafe,
        _rs__KeReleaseGuardedMutexUnsafe,
        _rs__KeAcquireGuardedMutex,
        _rs__KeReleaseGuardedMutex,
        _rs__KeTryToAcquireGuardedMutex,
        _rs_KiRecalculateDueTime,
        _rs_KxInsertTimer,
        _rs_KiComputeDueTime,
        _rs_KxRemoveTreeTimer,
        _rs_KxSetTimerForThreadWait,
    ];
}

// Defensive pull for ke-macros (Phase 2). Pilot batch (4 satisfy/add macros)
// + wait-setup batch (4 Kx*ThreadWait macros) + trap-frame/intrinsic batch
// (5 fns: YieldProcessor, KeGetTrapFramePc, KiRundownThread,
// KiGetLinkedTrapFrame, KiGetUserModeStackAddress) + HAL-dispatch batch
// (3 fns: HalQuerySystemInformation, HalSetSystemInformation,
// HalVectorToIDTEntry). Same `[used]` pattern as `_force_ke_inlines_symbols`.
mod _force_ke_macros_symbols {
    extern "C" {
        fn _rs_KiSatisfyMutantWait();
        fn _rs_KiSatisfyNonMutantWait();
        fn _rs_KiSatisfyObjectWait();
        fn _rs_KiAddThreadToWaitList();
        fn _rs_KxSingleThreadWait();
        fn _rs_KxDelayThreadWait();
        fn _rs_KxMultiThreadWait();
        fn _rs_KxQueueThreadWait();
        fn _rs_YieldProcessor();
        // _rs_KeGetTrapFramePc retired 2026-05-08; bare `KeGetTrapFramePc`
        // is now an `extern "system"` export in ke-macros, force-pulled by
        // existing kernel callers.
        fn _rs_KiRundownThread();
        fn _rs_KiGetLinkedTrapFrame();
        fn _rs_KiGetUserModeStackAddress();
        // HAL bridges promoted to bare-name `extern "system"` exports
        // 2026-05-08; existing C/Rust callers force-pull them.
        // 2026-05-07 ke_rust_shims.c additions:
        #[cfg(target_arch = "x86")]
        fn _rs_Ke386GetGlobalDescriptorTable();
        #[cfg(target_arch = "x86")]
        fn _rs_Ke386GetLocalDescriptorTable();
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        fn _rs_KeGdiFlushUserBatch_call();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 12] = [
        _rs_KiSatisfyMutantWait,
        _rs_KiSatisfyNonMutantWait,
        _rs_KiSatisfyObjectWait,
        _rs_KiAddThreadToWaitList,
        _rs_KxSingleThreadWait,
        _rs_KxDelayThreadWait,
        _rs_KxMultiThreadWait,
        _rs_KxQueueThreadWait,
        _rs_YieldProcessor,
        _rs_KiRundownThread,
        _rs_KiGetLinkedTrapFrame,
        _rs_KiGetUserModeStackAddress,
    ];
    // 2026-05-07: kdbg cdecl wrappers translated from kdbg_rust_shims.c.
    // Both arches need them — kdb_cli.rs's `KdbpCmdGdtLdtIdt` is exported
    // unconditionally and references the descriptor-table + TR helpers.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    extern "C" {
        fn KeGetContextFrameRegister();
        fn KeSetContextFrameRegister();
        fn KeGetTrapFrameFrameRegister();
        fn KeGetTrapFrameStackRegister();
        fn Ke386GetGlobalDescriptorTable();
        fn Ke386GetLocalDescriptorTable();
        fn Ke386GetTr();
    }
    // 2026-05-08: `KiExitSystemCallDebugChecks` translated from
    // `ke_rust_shims.c` `_saved_*` outline to a direct Rust impl in
    // `ke-macros` (i386-only — FORCEINLINE in `trap_x.h`). No force-pull
    // needed: the existing caller in `ke/i386/traphdlr.rs` already
    // references the bare stdcall name (`_KiExitSystemCallDebugChecks@8`).
    #[cfg(target_arch = "x86")]
    #[used]
    static REFS_I386: [unsafe extern "C" fn(); 10] = [
        _rs_Ke386GetGlobalDescriptorTable,
        _rs_Ke386GetLocalDescriptorTable,
        _rs_KeGdiFlushUserBatch_call,
        Ke386GetGlobalDescriptorTable,
        Ke386GetLocalDescriptorTable,
        Ke386GetTr,
        KeGetContextFrameRegister,
        KeSetContextFrameRegister,
        KeGetTrapFrameFrameRegister,
        KeGetTrapFrameStackRegister,
    ];
    #[cfg(target_arch = "x86_64")]
    #[used]
    static REFS_AMD64: [unsafe extern "C" fn(); 8] = [
        _rs_KeGdiFlushUserBatch_call,
        Ke386GetGlobalDescriptorTable,
        Ke386GetLocalDescriptorTable,
        Ke386GetTr,
        KeGetContextFrameRegister,
        KeSetContextFrameRegister,
        KeGetTrapFrameFrameRegister,
        KeGetTrapFrameStackRegister,
    ];

    // 2026-05-07: ARM64 global_asm! stubs translated from ntoskrnl/ke/arm64/{boot,stubs_asm}.S.
    // global_asm! symbols don't get pulled into the staticlib unless something
    // references them; force-pull to keep them in libkernel_rust.a.
    #[cfg(target_arch = "aarch64")]
    extern "C" {
        fn KiSwitchToBootStack();
        fn _enable();
        fn _disable();
    }
    #[cfg(target_arch = "aarch64")]
    #[used]
    static REFS_ARM64: [unsafe extern "C" fn(); 3] = [
        KiSwitchToBootStack,
        _enable,
        _disable,
    ];
}

// Defensive pull for mm-avl: AVL tree algorithms + 5 accessors.
// All 8 are bare-name exports (no C trampoline) — the duplicate-insertion
// guard that historically lived in the C `RtlpInsertAvlTreeNode` shim is
// now an early-return inside the rlib's `RtlpInsertAvlTreeNode` body.
mod _force_mm_avl_symbols {
    extern "C" {
    }
    unsafe extern "system" {
        fn RtlRightChildAvl();
        fn RtlLeftChildAvl();
        fn RtlParentAvl();
        fn RtlIsRightChildAvl();
        fn RtlIsLeftChildAvl();
        fn RtlpFindAvlTableNodeOrParent();
        fn RtlpInsertAvlTreeNode();
        fn RtlpDeleteAvlTreeNode();
    }
    #[used]
    static REFS: [unsafe extern "system" fn(); 8] = [
        RtlRightChildAvl,
        RtlLeftChildAvl,
        RtlParentAvl,
        RtlIsRightChildAvl,
        RtlIsLeftChildAvl,
        RtlpFindAvlTableNodeOrParent,
        RtlpInsertAvlTreeNode,
        RtlpDeleteAvlTreeNode,
    ];
}

// Defensive pull for mm-macros (mm.h / mmtypes.h / miarm.h #define macros).
mod _force_mm_macros_symbols {
    extern "C" {
        fn _rs_RMAP_IS_SEGMENT();
        fn _rs_PAGE_ROUND_DOWN();
        fn _rs_PFN_FROM_SSE();
        fn _rs_MM_IS_WAIT_PTE();
        fn _rs_MI_MAKE_HARDWARE_PTE_KERNEL();
        fn _rs_MI_MAKE_HARDWARE_PTE_USER();
        fn _rs_MI_MAKE_HARDWARE_PTE();
        fn _rs_MI_UPDATE_VALID_PTE();
        fn _rs_MmInitializeMdl();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 9] = [
        _rs_RMAP_IS_SEGMENT,
        _rs_PAGE_ROUND_DOWN,
        _rs_PFN_FROM_SSE,
        _rs_MM_IS_WAIT_PTE,
        _rs_MI_MAKE_HARDWARE_PTE_KERNEL,
        _rs_MI_MAKE_HARDWARE_PTE_USER,
        _rs_MI_MAKE_HARDWARE_PTE,
        _rs_MI_UPDATE_VALID_PTE,
        _rs_MmInitializeMdl,
    ];
}

// Defensive pull for io-inlines (io_x.h / iofuncs.h FORCEINLINE bodies).
// Same `[used]` pattern as `_force_ke_macros_symbols`.
//
// io-inlines exports the `_shim`-suffixed symbols directly (no C trampoline);
// the Rust callers in iomgr/{file,iofunc,irp,volume,rawfs}.rs declare these
// via `#[link_name = "X_shim"]`.
mod _force_io_inlines_symbols {
    extern "C" {
        fn IoCopyCurrentIrpStackLocationToNext_shim();
        fn HalInitPnpDriver_shim();
        fn CmiGetNextResourceDescriptor();
    }
    unsafe extern "system" {
        fn IopValidateOpenPacket_shim();
        fn IoGetDevObjExtension_shim();
        fn IopQueueIrpToThread_shim();
        fn IopUnQueueIrpFromThread_shim();
        fn IopLockFileObject_shim();
        fn IopUnlockFileObject_shim();
        fn IopUpdateOperationCount_shim();
        fn IopUpdateTransferCount_shim();
        fn IopIsValidPhysicalDeviceObject();
    }
    #[used]
    static REFS: [unsafe extern "system" fn(); 9] = [
        IopValidateOpenPacket_shim,
        IoGetDevObjExtension_shim,
        IopQueueIrpToThread_shim,
        IopUnQueueIrpFromThread_shim,
        IopLockFileObject_shim,
        IopUnlockFileObject_shim,
        IopUpdateOperationCount_shim,
        IopUpdateTransferCount_shim,
        IopIsValidPhysicalDeviceObject,
    ];
    #[used]
    static REFS_C: [unsafe extern "C" fn(); 3] = [
        IoCopyCurrentIrpStackLocationToNext_shim,
        HalInitPnpDriver_shim,
        CmiGetNextResourceDescriptor,
    ];
}

// Defensive pull for ob-inlines (ob_x.h FORCEINLINE wrappers, extracted from
// ntoskrnl-rust::shim_exports into a sibling rlib to defuse the cross-CGU
// captures(none) contagion that killed the original cm Pilot 1).
mod _force_ob_inlines_symbols {
    extern "C" {
        fn ObpAcquireDirectoryLockExclusive();
        fn ObpAcquireDirectoryLockShared();
        fn ObpAcquireLookupContextLock();
        fn ObpAcquireObjectLock();
        fn ObpAcquireObjectLockShared();
        fn ObpAllocateObjectCreateInfoBuffer();
        fn ObpCalloutEnd();
        fn ObpCalloutStart();
        fn ObpDereferenceNameInfo();
        fn ObpEnterObjectTypeMutex();
        fn ObpFreeCapturedAttributes();
        fn ObpFreeObjectCreateInformation();
        fn ObpInitializeLookupContext();
        fn ObpLeaveObjectTypeMutex();
        fn ObpReferenceNameInfo();
        fn ObpReleaseDirectoryLock();
        fn ObpReleaseLookupContext();
        fn ObpReleaseLookupContextObject();
        fn ObpReleaseObjectCreateInformation();
        fn ObpReleaseObjectLock();
        fn ObpSelectObjectLockSlot();
        fn ObpValidateAttributes();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 22] = [
        ObpAcquireDirectoryLockExclusive,
        ObpAcquireDirectoryLockShared,
        ObpAcquireLookupContextLock,
        ObpAcquireObjectLock,
        ObpAcquireObjectLockShared,
        ObpAllocateObjectCreateInfoBuffer,
        ObpCalloutEnd,
        ObpCalloutStart,
        ObpDereferenceNameInfo,
        ObpEnterObjectTypeMutex,
        ObpFreeCapturedAttributes,
        ObpFreeObjectCreateInformation,
        ObpInitializeLookupContext,
        ObpLeaveObjectTypeMutex,
        ObpReferenceNameInfo,
        ObpReleaseDirectoryLock,
        ObpReleaseLookupContext,
        ObpReleaseLookupContextObject,
        ObpReleaseObjectCreateInformation,
        ObpReleaseObjectLock,
        ObpSelectObjectLockSlot,
        ObpValidateAttributes,
    ];
}

// Defensive pull for ps-inlines (ps_x.h FORCEINLINE wrappers, extracted from
// ntoskrnl-rust::shim_exports — sibling rlib pattern).
mod _force_ps_inlines_symbols {
    extern "C" {
        fn PspClearProcessFlag();
        fn PspLockProcessSecurityExclusive();
        fn PspLockProcessSecurityShared();
        fn PspLockThreadSecurityExclusive();
        fn PspLockThreadSecurityShared();
        fn PspPrioritySeparationFromMask();
        fn PspQuantumLengthFromMask();
        fn PspQuantumTypeFromMask();
        fn PspRunCreateProcessNotifyRoutines();
        fn PspRunCreateThreadNotifyRoutines();
        fn PspRunLegoRoutine();
        fn PspRunLoadImageNotifyRoutines();
        fn PspSetCrossThreadFlag();
        fn PspSetProcessFlag();
        fn PspUnlockProcessSecurityExclusive();
        fn PspUnlockProcessSecurityShared();
        fn PspUnlockThreadSecurityExclusive();
        fn PspUnlockThreadSecurityShared();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 18] = [
        PspClearProcessFlag,
        PspLockProcessSecurityExclusive,
        PspLockProcessSecurityShared,
        PspLockThreadSecurityExclusive,
        PspLockThreadSecurityShared,
        PspPrioritySeparationFromMask,
        PspQuantumLengthFromMask,
        PspQuantumTypeFromMask,
        PspRunCreateProcessNotifyRoutines,
        PspRunCreateThreadNotifyRoutines,
        PspRunLegoRoutine,
        PspRunLoadImageNotifyRoutines,
        PspSetCrossThreadFlag,
        PspSetProcessFlag,
        PspUnlockProcessSecurityExclusive,
        PspUnlockProcessSecurityShared,
        PspUnlockThreadSecurityExclusive,
        PspUnlockThreadSecurityShared,
    ];
}

// Defensive pull for se-inlines (Se*/Sep* wrappers, extracted from
// ntoskrnl-rust::shim_exports — sibling rlib pattern).
mod _force_se_inlines_symbols {
    extern "C" {
        fn SeLengthSid();
        fn SeQuerySubjectContextToken();
        fn SepAcquireTokenLockExclusive();
        fn SepAcquireTokenLockShared();
        fn SepGetDaclFromDescriptor();
        fn SepGetGroupFromDescriptor();
        fn SepGetOwnerFromDescriptor();
        fn SepGetSaclFromDescriptor();
        fn SepReleaseTokenLock();
    }
    unsafe extern "system" {
        fn SeDeleteClientSecurity();
    }
    #[used]
    static REFS: [unsafe extern "system" fn(); 1] = [
        SeDeleteClientSecurity,
    ];
    #[used]
    static REFS_C: [unsafe extern "C" fn(); 9] = [
        SeLengthSid,
        SeQuerySubjectContextToken,
        SepAcquireTokenLockExclusive,
        SepAcquireTokenLockShared,
        SepGetDaclFromDescriptor,
        SepGetGroupFromDescriptor,
        SepGetOwnerFromDescriptor,
        SepGetSaclFromDescriptor,
        SepReleaseTokenLock,
    ];
}

// Defensive pull for common-inlines (cross-subsystem wrappers extracted from
// ntoskrnl-rust::shim_exports — sibling rlib pattern). 34 functions; one
// uses extern "system" (PsGetCurrentThread) but the REFS array stores the
// raw fn pointer either way.
mod _force_common_inlines_symbols {
    extern "C" {
        fn CcBugCheck();
        fn DefaultQueryInfoBufferCheck();
        fn DefaultSetInfoBufferCheck();
        fn ExInitializeResource();
        fn ExceptionRecord32To64();
        fn FsRtlEnterFileSystem();
        fn FsRtlExitFileSystem();
        fn InitializeObjectAttributes();
        fn InterlockedCompareExchangePointer();
        fn InterlockedDecrementUL();
        fn InterlockedExchangeAdd64();
        fn PopEntryList();
        fn ProbeAndCaptureUnicodeString();
        fn ProbeForReadLargeInteger();
        fn ProbeForReadUchar();
        fn ProbeForReadUshort();
        fn ProbeForWriteBoolean();
        fn ProbeForWriteHandle();
        fn ProbeForWriteLangId();
        fn ProbeForWritePointer();
        fn PspClearCrossThreadFlag();
        fn ReleaseCapturedUnicodeString();
        fn RtlInitEmptyAnsiString();
        fn RtlInitEmptyUnicodeString();
        fn RtlInitializeSplayLinks();
        fn RtlInsertAsLeftChild();
        fn RtlInsertAsRightChild();
        fn RtlIsLeftChild();
        fn RtlLongLongAdd();
        fn RtlParent();
        fn RtlStringCbCopyA();
    }
    unsafe extern "system" {
        fn InsertTailList();
        fn ProbeForWriteUlong();
        fn PsGetCurrentThread();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 31] = [
        CcBugCheck,
        DefaultQueryInfoBufferCheck,
        DefaultSetInfoBufferCheck,
        ExInitializeResource,
        ExceptionRecord32To64,
        FsRtlEnterFileSystem,
        FsRtlExitFileSystem,
        InitializeObjectAttributes,
        InterlockedCompareExchangePointer,
        InterlockedDecrementUL,
        InterlockedExchangeAdd64,
        PopEntryList,
        ProbeAndCaptureUnicodeString,
        ProbeForReadLargeInteger,
        ProbeForReadUchar,
        ProbeForReadUshort,
        ProbeForWriteBoolean,
        ProbeForWriteHandle,
        ProbeForWriteLangId,
        ProbeForWritePointer,
        PspClearCrossThreadFlag,
        ReleaseCapturedUnicodeString,
        RtlInitEmptyAnsiString,
        RtlInitEmptyUnicodeString,
        RtlInitializeSplayLinks,
        RtlInsertAsLeftChild,
        RtlInsertAsRightChild,
        RtlIsLeftChild,
        RtlLongLongAdd,
        RtlParent,
        RtlStringCbCopyA,
    ];
    #[used]
    static REFS_SYS: [unsafe extern "system" fn(); 3] = [
        InsertTailList,
        ProbeForWriteUlong,
        PsGetCurrentThread,
    ];
}

// Defensive pull for mm-pte-class-inlines (Phase 1 batch M1). Six wrappers
// on all arches plus one (_rs_MiIsPdeForAddressValid) on amd64/arm64 only.
mod _force_mm_pte_class_inlines_symbols {
    extern "C" {
        fn _rs_MiIsMappedPte();
        fn _rs_MiIsPageWriteable();
        fn _rs_MiIsPageDirty();
        fn _rs_MiIsUserPde();
        fn _rs_MiIsUserPte();
        fn _rs_MiDetermineUserGlobalPteMask();
        #[cfg(not(target_arch = "x86"))]
        fn _rs_MiIsPdeForAddressValid();
    }

    #[cfg(target_arch = "x86")]
    #[used]
    static REFS: [unsafe extern "C" fn(); 6] = [
        _rs_MiIsMappedPte,
        _rs_MiIsPageWriteable,
        _rs_MiIsPageDirty,
        _rs_MiIsUserPde,
        _rs_MiIsUserPte,
        _rs_MiDetermineUserGlobalPteMask,
    ];

    #[cfg(not(target_arch = "x86"))]
    #[used]
    static REFS: [unsafe extern "C" fn(); 7] = [
        _rs_MiIsMappedPte,
        _rs_MiIsPageWriteable,
        _rs_MiIsPageDirty,
        _rs_MiIsUserPde,
        _rs_MiIsUserPte,
        _rs_MiDetermineUserGlobalPteMask,
        _rs_MiIsPdeForAddressValid,
    ];
}

// Defensive pull for mm-util-inlines (Phase 1 batch M3). Six utility
// wrappers — interlocked, dirty-page setter, two probes, pool-zero alloc.
mod _force_mm_util_inlines_symbols {
    extern "C" {
        fn _rs_InterlockedIncrementUL();
        fn _rs_InterlockedExchangeAddUL();
        fn _rs_MmSetDirtyPage();
        fn _rs_ProbeForWriteSize_t();
        fn _rs_ProbeForWriteIoStatusBlock();
        fn _rs_ExAllocatePoolZero();
    }
    unsafe extern "system" {
        fn ExAllocatePoolZero();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 6] = [
        _rs_InterlockedIncrementUL,
        _rs_InterlockedExchangeAddUL,
        _rs_MmSetDirtyPage,
        _rs_ProbeForWriteSize_t,
        _rs_ProbeForWriteIoStatusBlock,
        _rs_ExAllocatePoolZero,
    ];
    #[used]
    static REFS_SYS: [unsafe extern "system" fn(); 1] = [
        ExAllocatePoolZero,
    ];
}

// Defensive pull for mm-pte-mut-inlines (Phase 1 batch M2). Eight bitfield
// writes against MMPTE — uniform shape across arches at the rlib layer
// (per-arch divergence handled inside `inline::mm`).
mod _force_mm_pte_mut_inlines_symbols {
    extern "C" {
        fn _rs_MiMakeDirtyPage();
        fn _rs_MiMakeCleanPage();
        fn _rs_MiMakeAccessedPage();
        fn _rs_MiMakeOwnerPage();
        fn _rs_MiMakeWritePage();
        fn _rs_MiPageDisableCache();
        fn _rs_MiPageWriteThrough();
        fn _rs_MiPageWriteCombined();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 8] = [
        _rs_MiMakeDirtyPage,
        _rs_MiMakeCleanPage,
        _rs_MiMakeAccessedPage,
        _rs_MiMakeOwnerPage,
        _rs_MiMakeWritePage,
        _rs_MiPageDisableCache,
        _rs_MiPageWriteThrough,
        _rs_MiPageWriteCombined,
    ];
}

// Defensive pull for mm-ws-lock-inlines (Phase 1 batch M4). 13 working-set
// lock wrappers — uniform shape (no arch divergence at the rlib layer).
mod _force_mm_ws_lock_inlines_symbols {
    extern "C" {
        fn _rs_MiLockWorkingSet();
        fn _rs_MiUnlockWorkingSet();
        fn _rs_MiLockWorkingSetShared();
        fn _rs_MiUnlockWorkingSetShared();
        fn _rs_MiLockProcessWorkingSet();
        fn _rs_MiUnlockProcessWorkingSet();
        fn _rs_MiLockProcessWorkingSetUnsafe();
        fn _rs_MiUnlockProcessWorkingSetUnsafe();
        fn _rs_MiLockProcessWorkingSetShared();
        fn _rs_MiUnlockProcessWorkingSetShared();
        fn _rs_MiLockProcessWorkingSetForFault();
        fn _rs_MiUnlockProcessWorkingSetForFault();
        fn _rs_MM_ANY_WS_LOCK_HELD();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 13] = [
        _rs_MiLockWorkingSet,
        _rs_MiUnlockWorkingSet,
        _rs_MiLockWorkingSetShared,
        _rs_MiUnlockWorkingSetShared,
        _rs_MiLockProcessWorkingSet,
        _rs_MiUnlockProcessWorkingSet,
        _rs_MiLockProcessWorkingSetUnsafe,
        _rs_MiUnlockProcessWorkingSetUnsafe,
        _rs_MiLockProcessWorkingSetShared,
        _rs_MiUnlockProcessWorkingSetShared,
        _rs_MiLockProcessWorkingSetForFault,
        _rs_MiUnlockProcessWorkingSetForFault,
        _rs_MM_ANY_WS_LOCK_HELD,
    ];
}

// Defensive pull for mm-addr-space-inlines (Phase 1 batch M5). Five
// wrappers — no arch divergence. The MmGetCurrent/Kernel/AddressSpaceOwner
// trio plus the AddressCreationLock guarded mutex pair.
mod _force_mm_addr_space_inlines_symbols {
    extern "C" {
        fn _rs_MmGetCurrentAddressSpace();
        fn _rs_MmGetKernelAddressSpace();
        fn _rs_MmGetAddressSpaceOwner();
        fn _rs_MmLockAddressSpace();
        fn _rs_MmUnlockAddressSpace();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 5] = [
        _rs_MmGetCurrentAddressSpace,
        _rs_MmGetKernelAddressSpace,
        _rs_MmGetAddressSpaceOwner,
        _rs_MmLockAddressSpace,
        _rs_MmUnlockAddressSpace,
    ];
}

// Defensive pull for mm-pte-atomic-inlines (Phase 1 batch M6). Two PTE
// atomic-update wrappers backed by core::sync::atomic.
mod _force_mm_pte_atomic_inlines_symbols {
    extern "C" {
        fn _rs_MiInterlockedCompareExchangePte();
        fn _rs_MiInterlockedExchangePte();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 2] = [
        _rs_MiInterlockedCompareExchangePte,
        _rs_MiInterlockedExchangePte,
    ];
}

// Defensive pull for mm-lock-inlines (Phase 1 batch M7). Expansion +
// PFN locks. The C trampolines retain MiExpansionLockOwner tracking
// (debug-only) and the ARM64 IRQL UART trace; the lock primitives
// themselves are in this rlib.
mod _force_mm_lock_inlines_symbols {
    extern "C" {
        fn _rs_MiAcquireExpansionLock();
        fn _rs_MiReleaseExpansionLock();
        fn _rs_MiAcquirePfnLock();
        fn _rs_MiReleasePfnLock();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 4] = [
        _rs_MiAcquireExpansionLock,
        _rs_MiReleaseExpansionLock,
        _rs_MiAcquirePfnLock,
        _rs_MiReleasePfnLock,
    ];
}

// Same defensive pull for the wide-format ntprintf_w FFI entries.
mod _force_ntprintf_w_symbols {
    extern "C" {
        fn ntprintf_w_scan_arg_types(fmt: *const u16, out: *mut u8, max: usize) -> i32;
        fn ntprintf_w_format(dst: *mut u16, cap_chars: usize, fmt: *const u16,
                             slots: *const u64, n: usize) -> usize;
    }
    #[used]
    static REF_SCAN_W: unsafe extern "C" fn(*const u16, *mut u8, usize) -> i32 =
        ntprintf_w_scan_arg_types;
    #[used]
    static REF_FORMAT_W: unsafe extern "C" fn(*mut u16, usize, *const u16, *const u64, usize) -> usize =
        ntprintf_w_format;
}

// Force inclusion of KDBG extension symbols (called from C kdb_cli.c, not from Rust).
// Without this, rustc dead-code elimination strips them from the staticlib.
mod _force_kdbg_symbols {
    extern "C" {
        fn ExpKdbgExtPool(argc: u32, argv: *mut *mut i8) -> u8;
        fn ExpKdbgExtPoolUsed(argc: u32, argv: *mut *mut i8) -> u8;
        fn ExpKdbgExtPoolFind(argc: u32, argv: *mut *mut i8) -> u8;
        fn ExpKdbgExtIrpFind(argc: u32, argv: *mut *mut i8) -> u8;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        fn ExpKdbgExtHandle(argc: u32, argv: *mut *mut i8) -> u8;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        fn ExpKdbgExtFileCache(argc: u32, argv: *mut *mut i8) -> u8;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        fn ExpKdbgExtDefWrites(argc: u32, argv: *mut *mut i8) -> u8;
    }
    #[used]
    static KDBG_REFS: [unsafe extern "C" fn(u32, *mut *mut i8) -> u8; 4] = [
        ExpKdbgExtPool,
        ExpKdbgExtPoolUsed,
        ExpKdbgExtPoolFind,
        ExpKdbgExtIrpFind,
    ];
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[used]
    static KDBG_REFS_X86: [unsafe extern "C" fn(u32, *mut *mut i8) -> u8; 3] = [
        ExpKdbgExtHandle,
        ExpKdbgExtFileCache,
        ExpKdbgExtDefWrites,
    ];
}

// Force inclusion of shim export symbols (called from C, not from Rust).
// Without this, rustc dead-code elimination strips them from the staticlib.
mod _force_shim_symbols {
    extern "C" {
        fn ARM64_UART_TRACE();
        fn ARM64_UART_TRACE_HEX();
        fn CONTAINING_RECORD();
        fn CcBugCheck();
        fn DefaultQueryInfoBufferCheck();
        fn DefaultSetInfoBufferCheck();
        fn ExAcquireFastReference();
        fn ExAcquirePushLockExclusive();
        fn ExAcquirePushLockShared();
        fn ExAllocateFromNPagedLookasideList();
        fn ExCompareSwapFastReference();
        fn ExConvertPushLockSharedToExclusive();
        fn ExFreeToNPagedLookasideList();
        fn ExGetCountFastReference();
        fn ExGetObjectFastReference();
        fn ExGetRunRefForGivenProcessor();
        fn ExInitializeFastMutex();
        fn ExInitializeFastReference();
        fn ExInitializeResource();
        fn ExInitializeWorkItem();
        fn ExInsertFastReference();
        fn ExReleaseFastReference();
        fn ExReleasePushLock();
        fn ExReleasePushLockExclusive();
        fn ExReleasePushLockShared();
        fn ExSwapFastReference();
        fn ExWaitOnPushLock();
        fn ExceptionRecord32To64();
        fn FsRtlEnterFileSystem();
        fn FsRtlExitFileSystem();
        fn HalEndOfBoot();
        fn HalSetWakeEnable();
        fn IS_SWAP_FROM_SSE();
        fn InitializeObjectAttributes();
        fn InterlockedBitTestAndReset();
        fn InterlockedBitTestAndSet();
        fn InterlockedCompareExchangePointer();
        fn InterlockedDecrementUL();
        fn InterlockedExchangeAdd64();
        fn InterlockedExchangeAddUL();
        fn InterlockedIncrementUL();
        fn IoGetCurrentIrpStackLocation();
        fn IoGetDevObjExtension();
        fn IoGetNextIrpStackLocation();
        fn IoMarkIrpPending();
        fn IoSetCancelRoutine();
        fn IoSetCompletionRoutine();
        fn IoSetNextIrpStackLocation();
        fn IoSkipCurrentIrpStackLocation();
        fn KeDisableInterrupts();
        fn KeEnterGuardedRegionThread();
        fn KeFlushProcessTb();
        fn KeGetContextPc();
        fn KeGetContextReturnRegister();
        fn KeGetContextSwitches();
        fn KeGetCurrentPrcb();
        fn KeGetCurrentProcessorNumber();
        fn KeGetExceptionFrame();
        fn KeGetTrapFrame();
        fn KeGetTrapFrameInterruptState();
        fn KeInvalidateTlbEntry();
        fn KeLeaveGuardedRegionThread();
        fn KeRestoreInterrupts();
        fn KeSetContextPc();
        fn KeSetContextReturnRegister();
        fn KeSweepICache();
        fn Ki386PerfEnd();
        fn KiAcquireApcLockAtSynchLevel();
        fn KiAcquireApcLockRaiseToDpc();
        fn KiAcquireApcLockRaiseToSynch();
        fn KiAcquireDeviceQueueLock();
        fn KiAcquireDispatcherLock();
        fn KiAcquireDispatcherLockAtSynchLevel();
        fn KiAcquireDispatcherObject();
        fn KiAcquireNmiListLock();
        fn KiAcquirePrcbLock();
        fn KiAcquireProcessLockRaiseToSynch();
        fn KiAcquireThreadLock();
        fn KiAcquireTimerLock();
        fn KiAddThreadToWaitList();
        fn KiCheckAlertability();
        fn KiCheckDeferredReadyList();
        fn KiCheckThreadStackSwap();
        fn KiComputeDueTime();
        fn KiComputeNewPriority();
        fn KiComputeTimerTableIndex();
        fn KiGetLinkedTrapFrame();
        fn KiGetUserModeStackAddress();
        fn KiInsertDeferredReadyList();
        fn KiRecalculateDueTime();
        fn KiReleaseApcLock();
        fn KiReleaseApcLockFromSynchLevel();
        fn KiReleaseDeviceQueueLock();
        fn KiReleaseDispatcherLock();
        fn KiReleaseDispatcherLockFromSynchLevel();
        fn KiReleaseDispatcherObject();
        fn KiReleaseNmiListLock();
        fn KiReleasePrcbLock();
        fn KiReleaseProcessLock();
        fn KiReleaseProcessLockFromSynchLevel();
        fn KiReleaseThreadLock();
        fn KiReleaseTimerLock();
        fn KiRemoveEntryTimer();
        fn KiRequestApcInterrupt();
        fn KiRescheduleThread();
        fn KiRundownThread();
        fn KiSatisfyMutantWait();
        fn KiSatisfyNonMutantWait();
        fn KiSatisfyObjectWait();
        fn KiSelectReadyThread();
        fn KiSetThreadSwapBusy();
        fn KiTryThreadLock();
        fn KxAcquireSpinLock();
        fn KxDelayThreadWait();
        fn KxInsertTimer();
        fn KxMultiThreadWait();
        fn KxQueueReadyThread();
        fn KxQueueThreadWait();
        fn KxReleaseSpinLock();
        fn KxRemoveTreeTimer();
        fn KxSetTimerForThreadWait();
        fn KxSingleThreadWait();
        fn KxUnwaitThread();
        fn KxUnwaitThreadForEvent();
        fn KxUnwaitThread_impl();
        fn LpcpGetDataInfoFromMessage();
        fn LpcpGetMessageFromThread();
        fn LpcpGetPortFromThread();
        fn LpcpReceiveWait();
        fn LpcpSetPortToThread();
        fn MI_MAKE_HARDWARE_PTE();
        fn MI_MAKE_HARDWARE_PTE_KERNEL();
        fn MI_MAKE_HARDWARE_PTE_USER();
        fn MI_UPDATE_VALID_PTE();
        fn MM_ANY_WS_LOCK_HELD();
        fn MM_IS_WAIT_PTE();
        fn MiAcquireExpansionLock();
        fn MiConvertSharedWorkingSetLockToExclusive();
        fn MiDetermineUserGlobalPteMask();
        fn MiDropLockCount();
        fn MiIncrementPageTableReferences();
        fn MiInterlockedCompareExchangePte();
        fn MiInterlockedExchangePte();
        fn MiIsMappedPte();
        fn MiIsMemoryTypeFree();
        fn MiIsMemoryTypeInvisible();
        fn MiIsPageDirty();
        fn MiIsPageWriteable();
        fn MiIsRosSectionObject();
        fn MiIsUserPde();
        fn MiIsUserPte();
        fn MiLockProcessWorkingSet();
        fn MiLockProcessWorkingSetForFault();
        fn MiLockProcessWorkingSetShared();
        fn MiLockProcessWorkingSetUnsafe();
        fn MiLockWorkingSet();
        fn MiLockWorkingSetShared();
        fn MiMakeAccessedPage();
        fn MiMakeCleanPage();
        fn MiMakeDirtyPage();
        fn MiMakeOwnerPage();
        fn MiMakeWritePage();
        fn MiPageDisableCache();
        fn MiPageWriteCombined();
        fn MiPageWriteThrough();
        fn MiReferenceProbedPageAndBumpLockCount();
        fn MiReferenceUnusedPageAndBumpLockCount();
        fn MiReferenceUsedPageAndBumpLockCount();
        fn MiReleaseExpansionLock();
        fn MiRemoveZeroPageSafe();
        fn MiUnlockProcessWorkingSet();
        fn MiUnlockProcessWorkingSetForFault();
        fn MiUnlockProcessWorkingSetShared();
        fn MiUnlockProcessWorkingSetUnsafe();
        fn MiUnlockWorkingSet();
        fn MiUnlockWorkingSetShared();
        fn MmDereferenceSegment();
        fn MmGetAddressSpaceOwner();
        fn MmGetCurrentAddressSpace();
        fn MmGetKernelAddressSpace();
        fn MmGetMdlPfnArray();
        fn MmGetSystemAddressForMdl();
        fn MmGetSystemAddressForMdlSafe();
        fn MmInitializeMdl();
        fn MmLockAddressSpace();
        fn MmSetDirtyPage();
        fn MmUnlockAddressSpace();
        fn NtCurrentProcess();
        fn NtCurrentTeb();
        fn ObReferenceObject();
        fn ObpGetHandleObject();
        // Other Obp* wrappers extracted into the ob-inlines rlib (see
        // _force_ob_inlines_symbols above).
        fn PAGE_ROUND_DOWN();
        fn PFN_FROM_SSE();
        fn PopEntryList();
        fn ProbeAndCaptureUnicodeString();
        fn ProbeForReadLargeInteger();
        fn ProbeForReadUchar();
        fn ProbeForReadUshort();
        fn ProbeForWriteBoolean();
        fn ProbeForWriteHandle();
        fn ProbeForWriteIoStatusBlock();
        fn ProbeForWriteLangId();
        fn ProbeForWritePointer();
        fn ProbeForWriteSize_t();
        fn PspClearCrossThreadFlag();
        // Other Psp* wrappers extracted into the ps-inlines rlib (see
        // _force_ps_inlines_symbols above).
        fn RMAP_IS_SEGMENT();
        fn ReleaseCapturedUnicodeString();
        fn RtlInitEmptyAnsiString();
        fn RtlInitEmptyUnicodeString();
        fn RtlInitializeSplayLinks();
        fn RtlInsertAsLeftChild();
        fn RtlInsertAsRightChild();
        fn RtlIsLeftChild();
        fn RtlLongLongAdd();
        fn RtlParent();
        fn RtlStringCbCopyA();
        fn SWAPENTRY_FROM_SSE();
        fn SeLengthSid();
        fn SeQuerySubjectContextToken();
        fn SepAcquireTokenLockExclusive();
        fn SepAcquireTokenLockShared();
        fn SepGetDaclFromDescriptor();
        fn SepGetGroupFromDescriptor();
        fn SepGetOwnerFromDescriptor();
        fn SepGetSaclFromDescriptor();
        fn SepReleaseTokenLock();
        fn UpdateTotalCommittedPages();
        fn YieldProcessor();
        fn _ExAcquireFastMutex();
        fn _ExAcquireFastMutexUnsafe();
        fn _ExEnterCriticalRegionAndAcquireFastMutexUnsafe();
        fn _ExReleaseFastMutex();
        fn _ExReleaseFastMutexUnsafe();
        fn _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion();
        fn _ExTryToAcquireFastMutex();
    }
    unsafe extern "system" {
        fn ExAllocateFromPagedLookasideList();
        fn ExFreeToPagedLookasideList();
        fn IoGetCurrentProcess();
        fn InsertTailList();
        fn KeInitializeSpinLock();
        fn LpcpAllocateFromPortZone();
        fn LpcpCompleteWait();
        fn LpcpConnectWait();
        fn LpcpReplyWait();
        fn ProbeForWriteUlong();
        fn PsGetCurrentProcess();
        fn RtlUnicodeStringToOemSize();
        fn SeDeleteClientSecurity();
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 235] = [
        ARM64_UART_TRACE,
        ARM64_UART_TRACE_HEX,
        CONTAINING_RECORD,
        CcBugCheck,
        DefaultQueryInfoBufferCheck,
        DefaultSetInfoBufferCheck,
        ExAcquireFastReference,
        ExAcquirePushLockExclusive,
        ExAcquirePushLockShared,
        ExAllocateFromNPagedLookasideList,
        ExCompareSwapFastReference,
        ExConvertPushLockSharedToExclusive,
        ExFreeToNPagedLookasideList,
        ExGetCountFastReference,
        ExGetObjectFastReference,
        ExGetRunRefForGivenProcessor,
        ExInitializeFastMutex,
        ExInitializeFastReference,
        ExInitializeResource,
        ExInitializeWorkItem,
        ExInsertFastReference,
        ExReleaseFastReference,
        ExReleasePushLock,
        ExReleasePushLockExclusive,
        ExReleasePushLockShared,
        ExSwapFastReference,
        ExWaitOnPushLock,
        ExceptionRecord32To64,
        FsRtlEnterFileSystem,
        FsRtlExitFileSystem,
        HalEndOfBoot,
        HalSetWakeEnable,
        IS_SWAP_FROM_SSE,
        InitializeObjectAttributes,
        InterlockedBitTestAndReset,
        InterlockedBitTestAndSet,
        InterlockedCompareExchangePointer,
        InterlockedDecrementUL,
        InterlockedExchangeAdd64,
        InterlockedExchangeAddUL,
        InterlockedIncrementUL,
        IoGetCurrentIrpStackLocation,
        IoGetDevObjExtension,
        IoGetNextIrpStackLocation,
        IoMarkIrpPending,
        IoSetCancelRoutine,
        IoSetCompletionRoutine,
        IoSetNextIrpStackLocation,
        IoSkipCurrentIrpStackLocation,
        KeDisableInterrupts,
        KeEnterGuardedRegionThread,
        KeFlushProcessTb,
        KeGetContextPc,
        KeGetContextReturnRegister,
        KeGetContextSwitches,
        KeGetCurrentPrcb,
        KeGetCurrentProcessorNumber,
        KeGetExceptionFrame,
        KeGetTrapFrame,
        KeGetTrapFrameInterruptState,
        KeInvalidateTlbEntry,
        KeLeaveGuardedRegionThread,
        KeRestoreInterrupts,
        KeSetContextPc,
        KeSetContextReturnRegister,
        KeSweepICache,
        Ki386PerfEnd,
        KiAcquireApcLockAtSynchLevel,
        KiAcquireApcLockRaiseToDpc,
        KiAcquireApcLockRaiseToSynch,
        KiAcquireDeviceQueueLock,
        KiAcquireDispatcherLock,
        KiAcquireDispatcherLockAtSynchLevel,
        KiAcquireDispatcherObject,
        KiAcquireNmiListLock,
        KiAcquirePrcbLock,
        KiAcquireProcessLockRaiseToSynch,
        KiAcquireThreadLock,
        KiAcquireTimerLock,
        KiAddThreadToWaitList,
        KiCheckAlertability,
        KiCheckDeferredReadyList,
        KiCheckThreadStackSwap,
        KiComputeDueTime,
        KiComputeNewPriority,
        KiComputeTimerTableIndex,
        KiGetLinkedTrapFrame,
        KiGetUserModeStackAddress,
        KiInsertDeferredReadyList,
        KiRecalculateDueTime,
        KiReleaseApcLock,
        KiReleaseApcLockFromSynchLevel,
        KiReleaseDeviceQueueLock,
        KiReleaseDispatcherLock,
        KiReleaseDispatcherLockFromSynchLevel,
        KiReleaseDispatcherObject,
        KiReleaseNmiListLock,
        KiReleasePrcbLock,
        KiReleaseProcessLock,
        KiReleaseProcessLockFromSynchLevel,
        KiReleaseThreadLock,
        KiReleaseTimerLock,
        KiRemoveEntryTimer,
        KiRequestApcInterrupt,
        KiRescheduleThread,
        KiRundownThread,
        KiSatisfyMutantWait,
        KiSatisfyNonMutantWait,
        KiSatisfyObjectWait,
        KiSelectReadyThread,
        KiSetThreadSwapBusy,
        KiTryThreadLock,
        KxAcquireSpinLock,
        KxDelayThreadWait,
        KxInsertTimer,
        KxMultiThreadWait,
        KxQueueReadyThread,
        KxQueueThreadWait,
        KxReleaseSpinLock,
        KxRemoveTreeTimer,
        KxSetTimerForThreadWait,
        KxSingleThreadWait,
        KxUnwaitThread,
        KxUnwaitThreadForEvent,
        KxUnwaitThread_impl,
        LpcpGetDataInfoFromMessage,
        LpcpGetMessageFromThread,
        LpcpGetPortFromThread,
        LpcpReceiveWait,
        LpcpSetPortToThread,
        MI_MAKE_HARDWARE_PTE,
        MI_MAKE_HARDWARE_PTE_KERNEL,
        MI_MAKE_HARDWARE_PTE_USER,
        MI_UPDATE_VALID_PTE,
        MM_ANY_WS_LOCK_HELD,
        MM_IS_WAIT_PTE,
        MiAcquireExpansionLock,
        MiConvertSharedWorkingSetLockToExclusive,
        MiDetermineUserGlobalPteMask,
        MiDropLockCount,
        MiIncrementPageTableReferences,
        MiInterlockedCompareExchangePte,
        MiInterlockedExchangePte,
        MiIsMappedPte,
        MiIsMemoryTypeFree,
        MiIsMemoryTypeInvisible,
        MiIsPageDirty,
        MiIsPageWriteable,
        MiIsRosSectionObject,
        MiIsUserPde,
        MiIsUserPte,
        MiLockProcessWorkingSet,
        MiLockProcessWorkingSetForFault,
        MiLockProcessWorkingSetShared,
        MiLockProcessWorkingSetUnsafe,
        MiLockWorkingSet,
        MiLockWorkingSetShared,
        MiMakeAccessedPage,
        MiMakeCleanPage,
        MiMakeDirtyPage,
        MiMakeOwnerPage,
        MiMakeWritePage,
        MiPageDisableCache,
        MiPageWriteCombined,
        MiPageWriteThrough,
        MiReferenceProbedPageAndBumpLockCount,
        MiReferenceUnusedPageAndBumpLockCount,
        MiReferenceUsedPageAndBumpLockCount,
        MiReleaseExpansionLock,
        MiRemoveZeroPageSafe,
        MiUnlockProcessWorkingSet,
        MiUnlockProcessWorkingSetForFault,
        MiUnlockProcessWorkingSetShared,
        MiUnlockProcessWorkingSetUnsafe,
        MiUnlockWorkingSet,
        MiUnlockWorkingSetShared,
        MmDereferenceSegment,
        MmGetAddressSpaceOwner,
        MmGetCurrentAddressSpace,
        MmGetKernelAddressSpace,
        MmGetMdlPfnArray,
        MmGetSystemAddressForMdl,
        MmGetSystemAddressForMdlSafe,
        MmInitializeMdl,
        MmLockAddressSpace,
        MmSetDirtyPage,
        MmUnlockAddressSpace,
        NtCurrentProcess,
        NtCurrentTeb,
        ObReferenceObject,
        ObpGetHandleObject,
        PAGE_ROUND_DOWN,
        PFN_FROM_SSE,
        PopEntryList,
        ProbeAndCaptureUnicodeString,
        ProbeForReadLargeInteger,
        ProbeForReadUchar,
        ProbeForReadUshort,
        ProbeForWriteBoolean,
        ProbeForWriteHandle,
        ProbeForWriteIoStatusBlock,
        ProbeForWriteLangId,
        ProbeForWritePointer,
        ProbeForWriteSize_t,
        PspClearCrossThreadFlag,
        RMAP_IS_SEGMENT,
        ReleaseCapturedUnicodeString,
        RtlInitEmptyAnsiString,
        RtlInitEmptyUnicodeString,
        RtlInitializeSplayLinks,
        RtlInsertAsLeftChild,
        RtlInsertAsRightChild,
        RtlIsLeftChild,
        RtlLongLongAdd,
        RtlParent,
        RtlStringCbCopyA,
        SWAPENTRY_FROM_SSE,
        SeLengthSid,
        SeQuerySubjectContextToken,
        SepAcquireTokenLockExclusive,
        SepAcquireTokenLockShared,
        SepGetDaclFromDescriptor,
        SepGetGroupFromDescriptor,
        SepGetOwnerFromDescriptor,
        SepGetSaclFromDescriptor,
        SepReleaseTokenLock,
        UpdateTotalCommittedPages,
        YieldProcessor,
        _ExAcquireFastMutex,
        _ExAcquireFastMutexUnsafe,
        _ExEnterCriticalRegionAndAcquireFastMutexUnsafe,
        _ExReleaseFastMutex,
        _ExReleaseFastMutexUnsafe,
        _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion,
        _ExTryToAcquireFastMutex,
    ];
    #[used]
    static REFS_SYS: [unsafe extern "system" fn(); 13] = [
        ExAllocateFromPagedLookasideList,
        ExFreeToPagedLookasideList,
        InsertTailList,
        IoGetCurrentProcess,
        KeInitializeSpinLock,
        LpcpAllocateFromPortZone,
        LpcpCompleteWait,
        LpcpConnectWait,
        LpcpReplyWait,
        ProbeForWriteUlong,
        PsGetCurrentProcess,
        RtlUnicodeStringToOemSize,
        SeDeleteClientSecurity,
    ];

    #[cfg(not(target_arch = "x86"))]
    mod not_x86 {
        extern "C" {
            fn ExAcquireRundownProtection();
            fn ExInitializeRundownProtection();
            fn ExReleaseRundownProtection();
            fn ExRundownCompleted();
            fn ExWaitForRundownProtectionRelease();
            fn KeQueryInterruptTime();
            fn MiIsPdeForAddressValid();
            // devaction.rs PnP functions called from C shims
            fn IopCancelRemoveChildDevices();
            fn IopCancelRemoveDeviceRelations();
            fn IopPrepareDeviceForRemoval();
            fn IopQueryRemoveChildDevices();
            fn IopQueryRemoveDeviceRelations();
            fn IopSendRemoveChildDevices();
            fn IopSendRemoveDeviceRelations();
        }
        unsafe extern "system" {
            fn KeQuerySystemTime();
        }
        extern "system" {
            fn IopSendEject();
            fn PipDeviceActionWorker();
            fn IopStartRamdisk();
        }
        #[used]
        static REFS_CDECL: [unsafe extern "C" fn(); 14] = [
            ExAcquireRundownProtection,
            ExInitializeRundownProtection,
            ExReleaseRundownProtection,
            ExRundownCompleted,
            ExWaitForRundownProtectionRelease,
            KeQueryInterruptTime,
            MiIsPdeForAddressValid,
            IopCancelRemoveChildDevices,
            IopCancelRemoveDeviceRelations,
            IopPrepareDeviceForRemoval,
            IopQueryRemoveChildDevices,
            IopQueryRemoveDeviceRelations,
            IopSendRemoveChildDevices,
            IopSendRemoveDeviceRelations,
        ];
        #[used]
        static REFS_CDECL_SYS: [unsafe extern "system" fn(); 1] = [
            KeQuerySystemTime,
        ];
        #[used]
        static REFS_STDCALL: [unsafe extern "system" fn(); 3] = [
            IopSendEject,
            PipDeviceActionWorker,
            IopStartRamdisk,
        ];
    }

    #[cfg(target_arch = "x86")]
    mod x86 {
        extern "C" {
            fn KeMemoryBarrier();
            fn KiGetGdtEntry();
            fn KiSetGdtDescriptorBase();
            fn MiSynchronizeSystemPde();
        }
        unsafe extern "system" {
            fn KeLowerIrql();
            fn KeRaiseIrql();
            fn RtlFillMemoryUlong();
        }
        #[used]
        static REFS: [unsafe extern "system" fn(); 3] = [
            KeLowerIrql,
            KeRaiseIrql,
            RtlFillMemoryUlong,
        ];
        #[used]
        static REFS_C: [unsafe extern "C" fn(); 4] = [
            KeMemoryBarrier,
            KiGetGdtEntry,
            KiSetGdtDescriptorBase,
            MiSynchronizeSystemPde,
        ];
    }

    #[cfg(target_arch = "x86_64")]
    mod x64 {
        extern "C" {
            fn KeGetPcr();
            fn KeMemoryBarrier();
            fn KeQueryInterruptHandler();
            fn KeRegisterInterruptHandler();
            fn KiGetGdtDescriptorBase();
            fn KiGetGdtEntry();
            fn KiInitGdtEntry();
            fn KiRestoreXState();
            fn KiSaveXState();
            fn KiSendEOI();
            fn KiSetGdtDescriptorBase();
            fn KiSetGdtDescriptorLimit();
            fn RtlFillMemoryUlong();
            fn __lgdt();
            fn __lidt();
            fn __readcr0();
            fn __readcr2();
            fn __readcr3();
            fn __readcr4();
            fn __readcr8();
            fn __readdr();
            fn __readmsr();
            fn __sgdt();
            fn __sidt();
            fn __sldt();
            fn __str();
            fn __swapgs();
            fn __wbinvd();
            fn __writecr0();
            fn __writecr3();
            fn __writecr4();
            fn __writecr8();
            fn __writedr();
            fn __writemsr();
        }
        unsafe extern "system" {
            fn KeGetCurrentIrql();
            fn KeLowerIrql();
            fn KeRaiseIrql();
            fn KeRaiseIrqlToDpcLevel();
            fn KfLowerIrql();
            fn KfRaiseIrql();
        }
        #[used]
        static REFS: [unsafe extern "system" fn(); 6] = [
            KeGetCurrentIrql,
            KeLowerIrql,
            KeRaiseIrql,
            KeRaiseIrqlToDpcLevel,
            KfLowerIrql,
            KfRaiseIrql,
        ];
        #[used]
        static REFS_C: [unsafe extern "C" fn(); 34] = [
            KeGetPcr,
            KeMemoryBarrier,
            KeQueryInterruptHandler,
            KeRegisterInterruptHandler,
            KiGetGdtDescriptorBase,
            KiGetGdtEntry,
            KiInitGdtEntry,
            KiRestoreXState,
            KiSaveXState,
            KiSendEOI,
            KiSetGdtDescriptorBase,
            KiSetGdtDescriptorLimit,
            RtlFillMemoryUlong,
            __lgdt,
            __lidt,
            __readcr0,
            __readcr2,
            __readcr3,
            __readcr4,
            __readcr8,
            __readdr,
            __readmsr,
            __sgdt,
            __sidt,
            __sldt,
            __str,
            __swapgs,
            __wbinvd,
            __writecr0,
            __writecr3,
            __writecr4,
            __writecr8,
            __writedr,
            __writemsr,
        ];
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Shared body: message + file:line + raw-PC backtrace to debugcon +
    // DbgPrint, then KeBugCheckEx(RUST_PANIC 0xE0525354). NB in the
    // cargo-linked kernel the *-cargo main.rs forwarder is the one that
    // actually runs (--allow-multiple-definition first-match) — both route
    // through the same panic-common body.
    panic_common::panic_common("ntoskrnl", info)
}

// At opt-level=0, the compiler emits .eh_frame data that references this symbol.
// Since we use panic=abort, unwinding never occurs, but we still need the symbol.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
