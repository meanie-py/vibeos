//! Umbrella staticlib that combines all Rust kernel components into a single
//! archive for linking into ntoskrnl. Each component is an rlib dependency;
//! this crate provides the single panic_handler and pulls in all symbols.

#![no_std]

// Re-export pool so its #[no_mangle] symbols are linked in.
extern crate reactos_pool;
// Re-export ntoskrnl-rust (ob subsystem) so its #[no_mangle] symbols are linked in.
extern crate ntoskrnl_rust;
// Re-export ntprintf so its FFI symbols (ntprintf_scan_arg_types,
// ntprintf_format_a) are linked in for the C variadic trampolines.
extern crate ntprintf;
// Re-export ntprintf_w (UTF-16 sibling) so its FFI symbols
// (ntprintf_w_scan_arg_types, ntprintf_w_format) are linked in for the
// wide variadic trampolines (RtlStringCbPrintfW, swprintf).
extern crate ntprintf_w;
// Re-export cm-inlines so its `_rs_Cmp{Acquire,Release,…}KcbLock*` impls
// are linked in for the C-side `cm_rust_shims.c` outlines that delegate
// to them. Isolated crate to dodge cross-CGU captures(none) cascades —
// see docs/engineering/shim-elimination.md § "Phase 1" for context.
extern crate cm_inlines;
// Same pattern, second batch: ke-inlines holds the `_rs_Ki{Acquire,Release}*`
// impls for the in-stack-queued-spinlock FORCEINLINEs from `ke_x.h`. C-side
// trampolines in `ke_rust_shims.c` delegate to them.
extern crate ke_inlines;
// Same pattern, third batch: ob-inlines and ps-inlines hold the wrappers
// for `ob_x.h` and `ps_x.h` FORCEINLINEs that previously lived in
// `ntoskrnl-rust::shim_exports` — extracted into sibling rlibs to defuse the
// cross-CGU captures(none) contagion risk that killed the original cm Pilot 1.
extern crate ob_inlines;
extern crate ps_inlines;
// Same pattern, fourth batch: se-inlines holds the Se*/Sep* helper wrappers
// (security-descriptor accessors, token locks, SeQuerySubjectContextToken,
// SeLengthSid, SeDeleteClientSecurity) that previously lived in
// `ntoskrnl-rust::shim_exports`. Sibling rlib for the same captures(none)
// containment reason as ob/ps-inlines.
extern crate se_inlines;
// Same pattern, fifth batch: common-inlines holds 34 cross-subsystem helper
// wrappers (Probe* family, splay-link, list-ops, RTL string helpers,
// interlocked, FsRtl region, init helpers, PsGetCurrentThread) extracted
// from `ntoskrnl-rust::shim_exports`. Sibling rlib for the same
// captures(none) containment reason — and because the wrapper count there
// is highest, defusing it with the rlib boundary has the highest expected
// payoff.
extern crate common_inlines;
// Same pattern, sixth batch — first MM batch. mm-pte-class-inlines holds
// the read-only PTE classification FORCEINLINEs from `miarm.h`
// (MiIsMappedPte, MiIsPageWriteable/Dirty, MiIsUserPde/Pte,
// MiIsPdeForAddressValid (64-bit only), MiDetermineUserGlobalPteMask).
// C trampolines in `mm_rust_shims.c` collapse to one-line `_rs_Mi*`
// delegations.
extern crate mm_pte_class_inlines;
// M2 sibling: PTE bitfield-write FORCEINLINEs (MiMake{Dirty,Clean,Accessed,
// Owner,Write}Page, MiPage{DisableCache,WriteThrough,WriteCombined}).
extern crate mm_pte_mut_inlines;
// M3 sibling: mm utility helpers (Interlocked{Increment,ExchangeAdd}UL,
// MmSetDirtyPage, ProbeForWrite{Size_t,IoStatusBlock}, ExAllocatePoolZero).
extern crate mm_util_inlines;

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

// Defensive symbol pull for the cm-inlines KCB-lock Rust impls. The C-side
// `cm_rust_shims.c` outlines call these by name; without this `[used]` block
// rustc/lld may strip the rlib's contribution from the final staticlib.
mod _force_cm_inlines_symbols {
    use core::ffi::c_void;
    extern "C" {
        fn _rs_CmpAcquireKcbLockExclusive(kcb: *mut c_void);
        fn _rs_CmpAcquireKcbLockExclusiveByIndex(index: u32);
        fn _rs_CmpAcquireKcbLockExclusiveByKey(conv_key: u32);
        fn _rs_CmpReleaseKcbLock(kcb: *mut c_void);
        fn _rs_CmpReleaseKcbLockByIndex(index: u32);
        fn _rs_CmpReleaseKcbLockByKey(conv_key: u32);
        fn _rs_CmpTryToConvertKcbSharedToExclusive(kcb: *mut c_void) -> u8;
        fn _rs_CmpConvertKcbSharedToExclusive(kcb: *mut c_void);
    }
    #[used]
    static REFS_PTR: [unsafe extern "C" fn(*mut c_void); 4] = [
        _rs_CmpAcquireKcbLockExclusive,
        _rs_CmpReleaseKcbLock,
        _rs_CmpConvertKcbSharedToExclusive,
        // _rs_CmpTryToConvertKcbSharedToExclusive returns u8 — typed below
        _rs_CmpReleaseKcbLock, // padding to keep the array uniform
    ];
    #[used]
    static REFS_INDEX: [unsafe extern "C" fn(u32); 4] = [
        _rs_CmpAcquireKcbLockExclusiveByIndex,
        _rs_CmpAcquireKcbLockExclusiveByKey,
        _rs_CmpReleaseKcbLockByIndex,
        _rs_CmpReleaseKcbLockByKey,
    ];
    #[used]
    static REF_TRY: unsafe extern "C" fn(*mut c_void) -> u8 =
        _rs_CmpTryToConvertKcbSharedToExclusive;
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
        fn SeDeleteClientSecurity();
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
    #[used]
    static REFS: [unsafe extern "C" fn(); 10] = [
        SeDeleteClientSecurity,
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
        fn InsertTailList();
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
        fn ProbeForWriteUlong();
        fn PsGetCurrentThread();
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
    #[used]
    static REFS: [unsafe extern "C" fn(); 34] = [
        CcBugCheck,
        DefaultQueryInfoBufferCheck,
        DefaultSetInfoBufferCheck,
        ExInitializeResource,
        ExceptionRecord32To64,
        FsRtlEnterFileSystem,
        FsRtlExitFileSystem,
        InitializeObjectAttributes,
        InsertTailList,
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
        ProbeForWriteUlong,
        PsGetCurrentThread,
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
    #[used]
    static REFS: [unsafe extern "C" fn(); 6] = [
        _rs_InterlockedIncrementUL,
        _rs_InterlockedExchangeAddUL,
        _rs_MmSetDirtyPage,
        _rs_ProbeForWriteSize_t,
        _rs_ProbeForWriteIoStatusBlock,
        _rs_ExAllocatePoolZero,
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
    }
    #[used]
    static KDBG_REFS: [unsafe extern "C" fn(u32, *mut *mut i8) -> u8; 4] = [
        ExpKdbgExtPool,
        ExpKdbgExtPoolUsed,
        ExpKdbgExtPoolFind,
        ExpKdbgExtIrpFind,
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
        fn ExAllocateFromPagedLookasideList();
        fn ExCompareSwapFastReference();
        fn ExConvertPushLockSharedToExclusive();
        fn ExFreeToNPagedLookasideList();
        fn ExFreeToPagedLookasideList();
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
        fn IoGetCurrentProcess();
        fn IS_SWAP_FROM_SSE();
        fn InitializeObjectAttributes();
        fn InsertTailList();
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
        fn KeInitializeSpinLock();
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
        fn LpcpAllocateFromPortZone();
        fn LpcpCompleteWait();
        fn LpcpConnectWait();
        fn LpcpGetDataInfoFromMessage();
        fn LpcpGetMessageFromThread();
        fn LpcpGetPortFromThread();
        fn LpcpReceiveWait();
        fn LpcpReplyWait();
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
        fn ProbeForWriteUlong();
        fn PsGetCurrentProcess();
        fn PspClearCrossThreadFlag();
        // Other Psp* wrappers extracted into the ps-inlines rlib (see
        // _force_ps_inlines_symbols above).
        fn RMAP_IS_SEGMENT();
        fn ReleaseCapturedUnicodeString();
        fn RtlAcquireSRWLockExclusive();
        fn RtlAcquireSRWLockShared();
        fn RtlInitEmptyAnsiString();
        fn RtlInitEmptyUnicodeString();
        fn RtlInitializeSplayLinks();
        fn RtlInsertAsLeftChild();
        fn RtlInsertAsRightChild();
        fn RtlIsLeftChild();
        fn RtlLongLongAdd();
        fn RtlParent();
        fn RtlReleaseSRWLockExclusive();
        fn RtlReleaseSRWLockShared();
        fn RtlStringCbCopyA();
        fn RtlUnicodeStringToOemSize();
        fn SWAPENTRY_FROM_SSE();
        fn SeDeleteClientSecurity();
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
    #[used]
    static REFS: [unsafe extern "C" fn(); 252] = [
        ARM64_UART_TRACE, ARM64_UART_TRACE_HEX,
        CONTAINING_RECORD, CcBugCheck, DefaultQueryInfoBufferCheck,
        DefaultSetInfoBufferCheck, ExAcquireFastReference, ExAcquirePushLockExclusive,
        ExAcquirePushLockShared, ExAllocateFromNPagedLookasideList,
        ExAllocateFromPagedLookasideList, ExCompareSwapFastReference,
        ExConvertPushLockSharedToExclusive, ExFreeToNPagedLookasideList,
        ExFreeToPagedLookasideList, ExGetCountFastReference, ExGetObjectFastReference,
        ExGetRunRefForGivenProcessor, ExInitializeFastMutex, ExInitializeFastReference,
        ExInitializeResource, ExInitializeWorkItem,
        ExInsertFastReference, ExReleaseFastReference, ExReleasePushLock,
        ExReleasePushLockExclusive, ExReleasePushLockShared, ExSwapFastReference,
        ExWaitOnPushLock, ExceptionRecord32To64, FsRtlEnterFileSystem,
        FsRtlExitFileSystem, HalEndOfBoot, HalSetWakeEnable,
        IS_SWAP_FROM_SSE, InitializeObjectAttributes, InsertTailList,
        InterlockedBitTestAndReset, InterlockedBitTestAndSet,
        InterlockedCompareExchangePointer, InterlockedDecrementUL,
        InterlockedExchangeAdd64, InterlockedExchangeAddUL, InterlockedIncrementUL,
        IoGetCurrentIrpStackLocation, IoGetDevObjExtension, IoGetNextIrpStackLocation,
        IoMarkIrpPending, IoSetCancelRoutine, IoSetCompletionRoutine,
        IoGetCurrentProcess,
        IoSetNextIrpStackLocation, IoSkipCurrentIrpStackLocation, KeDisableInterrupts,
        KeEnterGuardedRegionThread,
        KeFlushProcessTb, KeGetContextPc, KeGetContextReturnRegister,
        KeGetContextSwitches, KeGetCurrentPrcb, KeGetCurrentProcessorNumber,
        KeGetExceptionFrame, KeGetTrapFrame, KeGetTrapFrameInterruptState,
        KeInitializeSpinLock, KeInvalidateTlbEntry,
        KeLeaveGuardedRegionThread,
        KeRestoreInterrupts, KeSetContextPc, KeSetContextReturnRegister, KeSweepICache,
        Ki386PerfEnd, KiAcquireApcLockAtSynchLevel, KiAcquireApcLockRaiseToDpc,
        KiAcquireApcLockRaiseToSynch, KiAcquireDeviceQueueLock,
        KiAcquireDispatcherLock, KiAcquireDispatcherLockAtSynchLevel,
        KiAcquireDispatcherObject, KiAcquireNmiListLock, KiAcquirePrcbLock,
        KiAcquireProcessLockRaiseToSynch, KiAcquireThreadLock, KiAcquireTimerLock,
        KiAddThreadToWaitList, KiCheckAlertability, KiCheckDeferredReadyList,
        KiCheckThreadStackSwap, KiComputeDueTime, KiComputeNewPriority,
        KiComputeTimerTableIndex, KiGetLinkedTrapFrame, KiGetUserModeStackAddress,
        KiInsertDeferredReadyList, KiRecalculateDueTime, KiReleaseApcLock,
        KiReleaseApcLockFromSynchLevel, KiReleaseDeviceQueueLock,
        KiReleaseDispatcherLock, KiReleaseDispatcherLockFromSynchLevel,
        KiReleaseDispatcherObject, KiReleaseNmiListLock, KiReleasePrcbLock,
        KiReleaseProcessLock, KiReleaseProcessLockFromSynchLevel, KiReleaseThreadLock,
        KiReleaseTimerLock, KiRemoveEntryTimer, KiRequestApcInterrupt,
        KiRescheduleThread, KiRundownThread, KiSatisfyMutantWait,
        KiSatisfyNonMutantWait, KiSatisfyObjectWait, KiSelectReadyThread,
        KiSetThreadSwapBusy, KiTryThreadLock, KxAcquireSpinLock, KxDelayThreadWait,
        KxInsertTimer, KxMultiThreadWait, KxQueueReadyThread, KxQueueThreadWait,
        KxReleaseSpinLock, KxRemoveTreeTimer, KxSetTimerForThreadWait,
        KxSingleThreadWait, KxUnwaitThread, KxUnwaitThreadForEvent, KxUnwaitThread_impl,
        LpcpAllocateFromPortZone, LpcpCompleteWait, LpcpConnectWait,
        LpcpGetDataInfoFromMessage, LpcpGetMessageFromThread, LpcpGetPortFromThread,
        LpcpReceiveWait, LpcpReplyWait, LpcpSetPortToThread, MI_MAKE_HARDWARE_PTE,
        MI_MAKE_HARDWARE_PTE_KERNEL, MI_MAKE_HARDWARE_PTE_USER, MI_UPDATE_VALID_PTE,
        MM_ANY_WS_LOCK_HELD, MM_IS_WAIT_PTE, MiAcquireExpansionLock,
        MiConvertSharedWorkingSetLockToExclusive, MiDetermineUserGlobalPteMask,
        MiDropLockCount, MiIncrementPageTableReferences,
        MiInterlockedCompareExchangePte, MiInterlockedExchangePte, MiIsMappedPte,
        MiIsMemoryTypeFree, MiIsMemoryTypeInvisible, MiIsPageDirty, MiIsPageWriteable,
        MiIsRosSectionObject, MiIsUserPde, MiIsUserPte, MiLockProcessWorkingSet,
        MiLockProcessWorkingSetForFault, MiLockProcessWorkingSetShared,
        MiLockProcessWorkingSetUnsafe, MiLockWorkingSet, MiLockWorkingSetShared,
        MiMakeAccessedPage, MiMakeCleanPage, MiMakeDirtyPage, MiMakeOwnerPage,
        MiMakeWritePage, MiPageDisableCache, MiPageWriteCombined, MiPageWriteThrough,
        MiReferenceProbedPageAndBumpLockCount, MiReferenceUnusedPageAndBumpLockCount,
        MiReferenceUsedPageAndBumpLockCount, MiReleaseExpansionLock,
        MiRemoveZeroPageSafe, MiUnlockProcessWorkingSet,
        MiUnlockProcessWorkingSetForFault, MiUnlockProcessWorkingSetShared,
        MiUnlockProcessWorkingSetUnsafe, MiUnlockWorkingSet, MiUnlockWorkingSetShared,
        MmDereferenceSegment, MmGetAddressSpaceOwner, MmGetCurrentAddressSpace,
        MmGetKernelAddressSpace, MmGetMdlPfnArray, MmGetSystemAddressForMdl,
        MmGetSystemAddressForMdlSafe, MmInitializeMdl, MmLockAddressSpace,
        MmSetDirtyPage, MmUnlockAddressSpace, NtCurrentProcess, NtCurrentTeb,
        ObReferenceObject, ObpGetHandleObject, PAGE_ROUND_DOWN, PFN_FROM_SSE,
        PopEntryList, ProbeAndCaptureUnicodeString, ProbeForReadLargeInteger,
        ProbeForReadUchar, ProbeForReadUshort, ProbeForWriteBoolean,
        ProbeForWriteHandle, ProbeForWriteIoStatusBlock, ProbeForWriteLangId,
        ProbeForWritePointer, ProbeForWriteSize_t, ProbeForWriteUlong,
        PsGetCurrentProcess, PspClearCrossThreadFlag,
        RMAP_IS_SEGMENT, ReleaseCapturedUnicodeString,
        RtlAcquireSRWLockExclusive, RtlAcquireSRWLockShared, RtlInitEmptyAnsiString,
        RtlInitEmptyUnicodeString, RtlInitializeSplayLinks, RtlInsertAsLeftChild,
        RtlInsertAsRightChild, RtlIsLeftChild, RtlLongLongAdd, RtlParent,
        RtlReleaseSRWLockExclusive, RtlReleaseSRWLockShared, RtlStringCbCopyA,
        RtlUnicodeStringToOemSize, SWAPENTRY_FROM_SSE, SeDeleteClientSecurity,
        SeLengthSid, SeQuerySubjectContextToken, SepAcquireTokenLockExclusive,
        SepAcquireTokenLockShared, SepGetDaclFromDescriptor, SepGetGroupFromDescriptor,
        SepGetOwnerFromDescriptor, SepGetSaclFromDescriptor, SepReleaseTokenLock,
        UpdateTotalCommittedPages, YieldProcessor, _ExAcquireFastMutex,
        _ExAcquireFastMutexUnsafe, _ExEnterCriticalRegionAndAcquireFastMutexUnsafe,
        _ExReleaseFastMutex, _ExReleaseFastMutexUnsafe,
        _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion, _ExTryToAcquireFastMutex,
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
            fn KeQuerySystemTime();
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
        extern "system" {
            fn IopSendEject();
            fn PipDeviceActionWorker();
            fn IopStartRamdisk();
        }
        #[used]
        static REFS_CDECL: [unsafe extern "C" fn(); 15] = [
            ExAcquireRundownProtection, ExInitializeRundownProtection,
            ExReleaseRundownProtection, ExRundownCompleted,
            ExWaitForRundownProtectionRelease, KeQueryInterruptTime,
            KeQuerySystemTime, MiIsPdeForAddressValid,
            IopCancelRemoveChildDevices, IopCancelRemoveDeviceRelations,
            IopPrepareDeviceForRemoval, IopQueryRemoveChildDevices,
            IopQueryRemoveDeviceRelations, IopSendRemoveChildDevices,
            IopSendRemoveDeviceRelations,
        ];
        #[used]
        static REFS_STDCALL: [unsafe extern "system" fn(); 3] = [
            IopSendEject, PipDeviceActionWorker, IopStartRamdisk,
        ];
    }

    #[cfg(target_arch = "x86")]
    mod x86 {
        extern "C" {
            fn KeLowerIrql();
            fn KeMemoryBarrier();
            fn KeRaiseIrql();
            fn KiGetGdtEntry();
            fn KiSetGdtDescriptorBase();
            fn MiSynchronizeSystemPde();
            fn RtlFillMemoryUlong();
        }
        #[used]
        static REFS: [unsafe extern "C" fn(); 7] = [
            KeLowerIrql, KeMemoryBarrier, KeRaiseIrql,
            KiGetGdtEntry, KiSetGdtDescriptorBase,
            MiSynchronizeSystemPde, RtlFillMemoryUlong,
        ];
    }

    #[cfg(target_arch = "x86_64")]
    mod x64 {
        extern "C" {
            fn KeGetCurrentIrql();
            fn KeGetPcr();
            fn KeLowerIrql();
            fn KeMemoryBarrier();
            fn KeQueryInterruptHandler();
            fn KeRaiseIrql();
            fn KeRaiseIrqlToDpcLevel();
            fn KeRegisterInterruptHandler();
            fn KfLowerIrql();
            fn KfRaiseIrql();
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
        #[used]
        static REFS: [unsafe extern "C" fn(); 40] = [
            KeGetCurrentIrql, KeGetPcr, KeLowerIrql, KeMemoryBarrier,
            KeQueryInterruptHandler, KeRaiseIrql, KeRaiseIrqlToDpcLevel,
            KeRegisterInterruptHandler, KfLowerIrql, KfRaiseIrql,
            KiGetGdtDescriptorBase, KiGetGdtEntry, KiInitGdtEntry, KiRestoreXState,
            KiSaveXState, KiSendEOI, KiSetGdtDescriptorBase, KiSetGdtDescriptorLimit,
            RtlFillMemoryUlong, __lgdt, __lidt, __readcr0, __readcr2, __readcr3,
            __readcr4, __readcr8, __readdr, __readmsr, __sgdt, __sidt, __sldt, __str,
            __swapgs, __wbinvd, __writecr0, __writecr3, __writecr4, __writecr8,
            __writedr, __writemsr,
        ];
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Output panic location via debugcon port 0xe9 for QEMU diagnostics
    // (x86-only — ARM64 has no port I/O)
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        // Output "PANIC:" marker
        for &b in b"PANIC:" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }

        // Output file:line from PanicInfo if available
        if let Some(loc) = info.location() {
            let file = loc.file().as_bytes();
            let mut i = 0usize;
            while i < file.len() {
                let b = *file.as_ptr().add(i);
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
                i += 1;
            }
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b':');

            // Output line number as decimal digits
            let line = loc.line();
            if line == 0 {
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'0');
            } else {
                // Convert to decimal in a fixed buffer using raw pointer ops
                let mut buf = [0u8; 10];
                let mut n = line;
                let mut len = 0u32;
                while n > 0 {
                    *buf.as_mut_ptr().add(len as usize) = b'0' + (n % 10) as u8;
                    n /= 10;
                    len += 1;
                }
                // Output in reverse (most significant digit first)
                let mut j = len;
                while j > 0 {
                    j -= 1;
                    let b = *buf.as_ptr().add(j as usize);
                    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
                }
            }
        }
        core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'\n');
    }
    let _ = info;
    loop {}
}

// At opt-level=0, the compiler emits .eh_frame data that references this symbol.
// Since we use panic=abort, unwinding never occurs, but we still need the symbol.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
