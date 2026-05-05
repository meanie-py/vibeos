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
    }
    #[used]
    static REFS: [unsafe extern "C" fn(); 14] = [
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
        fn ObpGetHandleObject();
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
    static REFS: [unsafe extern "C" fn(); 292] = [
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
        ObReferenceObject, ObpAcquireDirectoryLockExclusive,
        ObpAcquireDirectoryLockShared, ObpAcquireLookupContextLock,
        ObpAcquireObjectLock, ObpAcquireObjectLockShared,
        ObpAllocateObjectCreateInfoBuffer, ObpCalloutEnd, ObpCalloutStart,
        ObpDereferenceNameInfo, ObpEnterObjectTypeMutex, ObpFreeCapturedAttributes,
        ObpFreeObjectCreateInformation, ObpGetHandleObject, ObpInitializeLookupContext,
        ObpLeaveObjectTypeMutex, ObpReferenceNameInfo, ObpReleaseDirectoryLock,
        ObpReleaseLookupContext, ObpReleaseLookupContextObject,
        ObpReleaseObjectCreateInformation, ObpReleaseObjectLock,
        ObpSelectObjectLockSlot, ObpValidateAttributes, PAGE_ROUND_DOWN, PFN_FROM_SSE,
        PopEntryList, ProbeAndCaptureUnicodeString, ProbeForReadLargeInteger,
        ProbeForReadUchar, ProbeForReadUshort, ProbeForWriteBoolean,
        ProbeForWriteHandle, ProbeForWriteIoStatusBlock, ProbeForWriteLangId,
        ProbeForWritePointer, ProbeForWriteSize_t, ProbeForWriteUlong,
        PsGetCurrentProcess, PspClearCrossThreadFlag,
        PspClearProcessFlag, PspLockProcessSecurityExclusive,
        PspLockProcessSecurityShared, PspLockThreadSecurityExclusive,
        PspLockThreadSecurityShared, PspPrioritySeparationFromMask,
        PspQuantumLengthFromMask, PspQuantumTypeFromMask,
        PspRunCreateProcessNotifyRoutines, PspRunCreateThreadNotifyRoutines,
        PspRunLegoRoutine, PspRunLoadImageNotifyRoutines, PspSetCrossThreadFlag,
        PspSetProcessFlag, PspUnlockProcessSecurityExclusive,
        PspUnlockProcessSecurityShared, PspUnlockThreadSecurityExclusive,
        PspUnlockThreadSecurityShared, RMAP_IS_SEGMENT, ReleaseCapturedUnicodeString,
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
