@ stdcall CcCanIWrite(ptr long long long)
@ stdcall CcCopyRead(ptr ptr long long ptr ptr)
@ stdcall CcCopyWrite(ptr ptr long long ptr)
@ stdcall CcDeferWrite(ptr ptr ptr ptr long long)
@ stdcall CcFastCopyRead(ptr long long long ptr ptr)
@ stdcall CcFastCopyWrite(ptr long long ptr)
@ extern CcFastMdlReadWait
@ extern CcFastReadNotPossible
@ extern CcFastReadWait
@ stdcall CcFlushCache(ptr ptr long ptr)
@ stdcall CcGetDirtyPages(ptr ptr ptr ptr)
@ stdcall CcGetFileObjectFromBcb(ptr)
@ stdcall CcGetFileObjectFromSectionPtrs(ptr)
@ stdcall CcGetFlushedValidData(ptr long)
@ stdcall CcGetLsnForFileObject(ptr ptr)
@ stdcall CcInitializeCacheMap(ptr ptr long ptr ptr)
@ stdcall CcIsThereDirtyData(ptr)
@ stdcall CcMapData(ptr ptr long long ptr ptr)
@ stdcall CcMdlRead(ptr ptr long ptr ptr)
@ stdcall CcMdlReadComplete(ptr ptr)
@ stdcall CcMdlWriteAbort(ptr ptr)
@ stdcall CcMdlWriteComplete(ptr ptr ptr)
@ stdcall CcPinMappedData(ptr ptr long long ptr)
@ stdcall CcPinRead(ptr ptr long long ptr ptr)
@ stdcall CcPrepareMdlWrite(ptr ptr long ptr ptr)
@ stdcall CcPreparePinWrite(ptr ptr long long long ptr ptr)
@ stdcall CcPurgeCacheSection(ptr ptr long long)
@ stdcall CcRemapBcb(ptr)
@ stdcall CcRepinBcb(ptr)
@ stdcall CcScheduleReadAhead(ptr ptr long)
@ stdcall CcSetAdditionalCacheAttributes(ptr long long)
@ stdcall CcSetBcbOwnerPointer(ptr ptr)
@ stdcall CcSetDirtyPageThreshold(ptr long)
@ stdcall CcSetDirtyPinnedData(ptr ptr)
@ stdcall CcSetFileSizes(ptr ptr)
@ stdcall CcSetLogHandleForFile(ptr ptr ptr)
@ stdcall CcSetReadAheadGranularity(ptr long)
@ stdcall CcUninitializeCacheMap(ptr ptr ptr)
@ stdcall CcUnpinData(ptr)
@ stdcall CcUnpinDataForThread(ptr ptr)
@ stdcall CcUnpinRepinnedBcb(ptr long ptr)
@ stdcall CcWaitForCurrentLazyWriterActivity()
@ stdcall CcZeroData(ptr ptr ptr long)
@ stdcall CmRegisterCallback(ptr ptr ptr)
@ stdcall CmUnRegisterCallback(long long)
@ stdcall DbgBreakPoint()
@ stdcall DbgBreakPointWithStatus(long)
@ stdcall DbgCommandString(ptr ptr)
@ stdcall DbgLoadImageSymbols(ptr ptr long)
@ varargs DbgPrint(str)
@ varargs DbgPrintEx(long long str)
@ varargs DbgPrintReturnControlC(str)
@ stdcall DbgPrompt(str ptr long)
@ stdcall DbgQueryDebugFilterState(long long)
@ stdcall DbgSetDebugFilterState(long long long)
@ stdcall EtwActivityIdControl(long ptr)
@ stdcall EtwEventEnabled(int64 ptr)
@ stdcall EtwProviderEnabled(int64 long int64)
@ stdcall EtwRegister(ptr ptr ptr ptr)
@ stdcall EtwSetInformation(int64 long ptr long)
@ stdcall EtwUnregister(int64)
@ stdcall EtwWrite(int64 ptr ptr long ptr)
@ stdcall EtwWriteEx(int64 ptr int64 long ptr ptr long ptr)
@ stdcall EtwWriteString(int64 long int64 ptr ptr)
@ stdcall EtwWriteTransfer(int64 ptr ptr ptr long ptr)
@ stdcall -arch=x86_64 ExAcquireFastMutex(ptr)
@ fastcall ExAcquireFastMutexUnsafe(ptr)
@ stdcall ExAcquireResourceExclusiveLite(ptr long)
@ stdcall ExAcquireResourceSharedLite(ptr long)
@ fastcall ExAcquireRundownProtection(ptr) ExfAcquireRundownProtection
@ fastcall ExAcquireRundownProtectionCacheAware(ptr) ExfAcquireRundownProtectionCacheAware
@ fastcall ExAcquireRundownProtectionCacheAwareEx(ptr long) ExfAcquireRundownProtectionCacheAwareEx
@ fastcall ExAcquireRundownProtectionEx(ptr long) ExfAcquireRundownProtectionEx
@ stdcall ExAcquireSharedStarveExclusive(ptr long)
@ stdcall ExAcquireSharedWaitForExclusive(ptr long)
@ stdcall ExAllocateCacheAwareRundownProtection(long long)
@ stdcall ExAllocateFromPagedLookasideList(ptr) ExiAllocateFromPagedLookasideList
@ stdcall ExAllocatePool(long long)
@ stdcall ExAllocatePool2(int64 long long)
@ stdcall ExAllocatePool3(int64 long long ptr long)
@ stdcall ExAllocatePoolWithQuota(long long)
@ stdcall ExAllocatePoolWithQuotaTag(long long long)
@ stdcall ExAllocatePoolWithTag(long long long)
@ stdcall ExAllocatePoolWithTagPriority(long long long long)
@ stdcall ExConvertExclusiveToSharedLite(ptr)
@ stdcall ExCreateCallback(ptr ptr long long)
@ stdcall ExDeleteNPagedLookasideList(ptr)
@ stdcall ExDeletePagedLookasideList(ptr)
@ stdcall ExDeleteResourceLite(ptr)
@ extern ExDesktopObjectType
@ stdcall ExDisableResourceBoostLite(ptr)
@ fastcall ExEnterCriticalRegionAndAcquireFastMutexUnsafe(ptr)
@ stdcall ExEnterCriticalRegionAndAcquireResourceExclusive(ptr)
@ stdcall ExEnterCriticalRegionAndAcquireResourceShared(ptr)
@ stdcall ExEnterCriticalRegionAndAcquireSharedWaitForExclusive(ptr)
@ stdcall ExEnumHandleTable(ptr ptr ptr ptr)
@ extern ExEventObjectType
@ stdcall ExExtendZone(ptr ptr long)
@ stdcall ExFreeCacheAwareRundownProtection(ptr)
@ stdcall ExFreePool(ptr)
@ stdcall ExFreePool2(ptr long ptr long)
@ stdcall ExFreePoolWithTag(ptr long)
@ stdcall ExFreeToPagedLookasideList(ptr ptr) ExiFreeToPagedLookasideList
@ stdcall ExGetCurrentProcessorCounts(ptr ptr ptr)
@ stdcall ExGetCurrentProcessorCpuUsage(ptr)
@ stdcall ExGetExclusiveWaiterCount(ptr)
@ stdcall ExGetPreviousMode()
@ stdcall ExGetSharedWaiterCount(ptr)
@ stdcall ExInitializeNPagedLookasideList(ptr ptr ptr long long long long)
@ stdcall ExInitializePagedLookasideList(ptr ptr ptr long long long long)
@ stdcall ExInitializeResourceLite(ptr)
@ fastcall ExInitializeRundownProtection(ptr) ExfInitializeRundownProtection
@ stdcall ExInitializeRundownProtectionCacheAware(ptr long)
@ stdcall ExInitializeZone(ptr long ptr long)
@ stdcall ExInterlockedAddLargeInteger(ptr long long ptr)
@ fastcall -arch=i386 ExInterlockedAddLargeStatistic(ptr long)
@ stdcall ExInterlockedAddUlong(ptr long ptr)
@ fastcall -arch=i386 ExInterlockedCompareExchange64(ptr ptr ptr ptr)
@ stdcall -arch=i386 ExInterlockedDecrementLong(ptr ptr)
@ stdcall -arch=i386 ExInterlockedExchangeUlong(ptr long ptr)
@ stdcall ExInterlockedExtendZone(ptr ptr long ptr)
@ fastcall -arch=i386 ExInterlockedFlushSList(ptr)
@ stdcall -arch=i386 ExInterlockedIncrementLong(ptr ptr)
@ stdcall ExInterlockedInsertHeadList(ptr ptr ptr)
@ stdcall ExInterlockedInsertTailList(ptr ptr ptr)
@ stdcall ExInterlockedPopEntryList(ptr ptr)
@ fastcall -arch=i386 ExInterlockedPopEntrySList(ptr ptr)
@ stdcall ExInterlockedPushEntryList(ptr ptr ptr)
@ fastcall -arch=i386 ExInterlockedPushEntrySList(ptr ptr ptr)
@ stdcall ExInterlockedRemoveHeadList(ptr ptr)
@ stdcall ExIsProcessorFeaturePresent(long)
@ stdcall ExIsResourceAcquiredExclusiveLite(ptr)
@ stdcall ExIsResourceAcquiredSharedLite(ptr)
@ stdcall ExLocalTimeToSystemTime(ptr ptr)
@ stdcall ExNotifyCallback(ptr ptr ptr)
@ stdcall -arch=x86_64,arm,arm64 ExQueryDepthSList(ptr) RtlQueryDepthSList
@ stdcall ExQueryPoolBlockSize(ptr ptr)
@ stdcall ExQueueWorkItem(ptr long)
@ stdcall ExRaiseAccessViolation()
@ stdcall ExRaiseDatatypeMisalignment()
@ stdcall ExRaiseException(ptr) RtlRaiseException
@ stdcall ExRaiseHardError(long long long ptr long ptr)
@ stdcall ExRaiseStatus(long)
@ fastcall ExReInitializeRundownProtection(ptr) ExfReInitializeRundownProtection
@ fastcall ExReInitializeRundownProtectionCacheAware(ptr) ExfReInitializeRundownProtectionCacheAware
@ stdcall ExRegisterCallback(ptr ptr ptr)
@ stdcall ExReinitializeResourceLite(ptr)
@ stdcall -arch=x86_64 ExReleaseFastMutex(ptr)
@ fastcall ExReleaseFastMutexUnsafe(ptr)
@ fastcall ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(ptr)
@ fastcall ExReleaseResourceAndLeaveCriticalRegion(ptr)
@ stdcall ExReleaseResourceForThreadLite(ptr long)
@ fastcall ExReleaseResourceLite(ptr)
@ fastcall ExReleaseRundownProtection(ptr) ExfReleaseRundownProtection
@ fastcall ExReleaseRundownProtectionCacheAware(ptr) ExfReleaseRundownProtectionCacheAware
@ fastcall ExReleaseRundownProtectionCacheAwareEx(ptr long) ExfReleaseRundownProtectionCacheAwareEx
@ fastcall ExReleaseRundownProtectionEx(ptr long) ExfReleaseRundownProtectionEx
@ fastcall ExRundownCompleted(ptr) ExfRundownCompleted
@ fastcall ExRundownCompletedCacheAware(ptr) ExfRundownCompletedCacheAware
@ extern ExSemaphoreObjectType
@ stdcall ExSetResourceOwnerPointer(ptr ptr)
@ stdcall ExSetTimerResolution(long long)
@ stdcall ExSizeOfRundownProtectionCacheAware()
@ stdcall ExSystemExceptionFilter()
@ stdcall ExSystemTimeToLocalTime(ptr ptr)
@ stdcall -arch=x86_64 ExTryToAcquireFastMutex(ptr)
@ stdcall ExUnregisterCallback(ptr)
@ stdcall ExUuidCreate(ptr)
@ stdcall ExVerifySuite(long)
@ fastcall ExWaitForRundownProtectionRelease(ptr) ExfWaitForRundownProtectionRelease
@ fastcall ExWaitForRundownProtectionReleaseCacheAware(ptr) ExfWaitForRundownProtectionReleaseCacheAware
@ extern ExWindowStationObjectType
@ fastcall ExfAcquirePushLockExclusive(ptr)
@ fastcall ExfAcquirePushLockShared(ptr)
@ fastcall -arch=i386 ExfInterlockedAddUlong(ptr long ptr)
@ fastcall -arch=i386 ExfInterlockedCompareExchange64(ptr ptr ptr)
@ fastcall -arch=i386 ExfInterlockedInsertHeadList(ptr ptr ptr)
@ fastcall -arch=i386 ExfInterlockedInsertTailList(ptr ptr ptr)
@ fastcall -arch=i386 ExfInterlockedPopEntryList(ptr ptr)
@ fastcall -arch=i386 ExfInterlockedPushEntryList(ptr ptr ptr)
@ fastcall -arch=i386 ExfInterlockedRemoveHeadList(ptr ptr)
@ fastcall ExfReleasePushLock(ptr)
@ fastcall ExfReleasePushLockExclusive(ptr)
@ fastcall ExfReleasePushLockShared(ptr)
@ fastcall ExfTryToWakePushLock(ptr)
@ fastcall ExfUnblockPushLock(ptr ptr)
@ stdcall -arch=x86_64,arm,arm64 ExpInterlockedFlushSList(ptr) RtlInterlockedFlushSList
@ stdcall -arch=x86_64,arm,arm64 ExpInterlockedPopEntrySList(ptr ptr) RtlInterlockedPopEntrySList
@ stdcall -arch=x86_64,arm,arm64 ExpInterlockedPushEntrySList(ptr ptr) RtlInterlockedPushEntrySList
@ fastcall -arch=i386 Exfi386InterlockedDecrementLong(ptr)
@ fastcall -arch=i386 Exfi386InterlockedExchangeUlong(ptr long)
@ fastcall -arch=i386 Exfi386InterlockedIncrementLong(ptr)
@ stdcall -arch=i386 Exi386InterlockedDecrementLong(ptr)
@ stdcall -arch=i386 Exi386InterlockedExchangeUlong(ptr long)
@ stdcall -arch=i386 Exi386InterlockedIncrementLong(ptr)
@ fastcall -arch=i386 ExiAcquireFastMutex(ptr) ExAcquireFastMutex
@ fastcall -arch=i386 ExiReleaseFastMutex(ptr) ExReleaseFastMutex
@ fastcall -arch=i386 ExiTryToAcquireFastMutex(ptr) ExTryToAcquireFastMutex
@ stdcall FsRtlAcquireFileExclusive(ptr)
@ stdcall FsRtlAddBaseMcbEntry(ptr long long long long long long)
@ stdcall FsRtlAddLargeMcbEntry(ptr long long long long long long)
@ stdcall FsRtlAddMcbEntry(ptr long long long)
@ stdcall FsRtlAddToTunnelCache(ptr long long ptr ptr long long ptr)
@ stdcall FsRtlAllocateFileLock(ptr ptr)
@ stdcall FsRtlAllocatePool(long long)
@ stdcall FsRtlAllocatePoolWithQuota(long long)
@ stdcall FsRtlAllocatePoolWithQuotaTag(long long long)
@ stdcall FsRtlAllocatePoolWithTag(long long long)
@ stdcall FsRtlAllocateResource()
@ stdcall FsRtlAreNamesEqual(ptr ptr long wstr)
@ stdcall FsRtlBalanceReads(ptr)
@ stdcall FsRtlCheckLockForReadAccess(ptr ptr)
@ stdcall FsRtlCheckLockForWriteAccess(ptr ptr)
@ stdcall FsRtlCheckOplock(ptr ptr ptr ptr ptr)
@ stdcall FsRtlCopyRead(ptr ptr long long long ptr ptr ptr)
@ stdcall FsRtlCopyWrite(ptr ptr long long long ptr ptr ptr)
@ stdcall FsRtlCreateSectionForDataScan(ptr ptr ptr ptr long ptr ptr long long long)
@ stdcall FsRtlCurrentBatchOplock(ptr)
@ stdcall FsRtlDeleteKeyFromTunnelCache(ptr long long)
@ stdcall FsRtlDeleteTunnelCache(ptr)
@ stdcall FsRtlDeregisterUncProvider(ptr)
@ stdcall FsRtlDissectDbcs(long ptr ptr ptr)
@ stdcall FsRtlDissectName(long ptr ptr ptr)
@ stdcall FsRtlDoesDbcsContainWildCards(ptr)
@ stdcall FsRtlDoesNameContainWildCards(ptr)
@ stdcall FsRtlFastCheckLockForRead(ptr ptr ptr long ptr ptr)
@ stdcall FsRtlFastCheckLockForWrite(ptr ptr ptr long ptr ptr)
@ stdcall FsRtlFastUnlockAll(ptr ptr ptr ptr)
@ stdcall FsRtlFastUnlockAllByKey(ptr ptr ptr long ptr)
@ stdcall FsRtlFastUnlockSingle(ptr ptr ptr ptr ptr long ptr long)
@ stdcall FsRtlFindInTunnelCache(ptr long long ptr ptr ptr ptr ptr)
@ stdcall FsRtlFreeFileLock(ptr)
@ stdcall FsRtlGetFileSize(ptr ptr)
@ stdcall FsRtlGetNextBaseMcbEntry(ptr long ptr ptr ptr)
@ stdcall FsRtlGetNextFileLock(ptr long)
@ stdcall FsRtlGetNextLargeMcbEntry(ptr long ptr ptr ptr)
@ stdcall FsRtlGetNextMcbEntry(ptr long ptr ptr ptr)
@ stdcall FsRtlIncrementCcFastReadNoWait()
@ stdcall FsRtlIncrementCcFastReadNotPossible()
@ stdcall FsRtlIncrementCcFastReadResourceMiss()
@ stdcall FsRtlIncrementCcFastReadWait()
@ stdcall FsRtlInitializeBaseMcb(ptr ptr)
@ stdcall FsRtlInitializeFileLock(ptr ptr ptr)
@ stdcall FsRtlInitializeLargeMcb(ptr long)
@ stdcall FsRtlInitializeMcb(ptr long)
@ stdcall FsRtlInitializeOplock(ptr)
@ stdcall FsRtlInitializeTunnelCache(ptr)
@ stdcall FsRtlInsertPerFileObjectContext(ptr ptr)
@ stdcall FsRtlInsertPerStreamContext(ptr ptr)
@ stdcall FsRtlIsDbcsInExpression(ptr ptr)
@ stdcall FsRtlIsFatDbcsLegal(long ptr long long long)
@ stdcall FsRtlIsHpfsDbcsLegal(long ptr long long long)
@ stdcall FsRtlIsNameInExpression(ptr ptr long wstr)
@ stdcall FsRtlIsNtstatusExpected(long)
@ stdcall FsRtlIsPagingFile(ptr)
@ stdcall FsRtlIsTotalDeviceFailure(ptr)
@ extern FsRtlLegalAnsiCharacterArray
@ stdcall FsRtlLookupBaseMcbEntry(ptr long long ptr ptr ptr ptr ptr)
@ stdcall FsRtlLookupLargeMcbEntry(ptr long long ptr ptr ptr ptr ptr)
@ stdcall FsRtlLookupLastBaseMcbEntry(ptr ptr ptr)
@ stdcall FsRtlLookupLastBaseMcbEntryAndIndex(ptr ptr ptr ptr)
@ stdcall FsRtlLookupLastLargeMcbEntry(ptr ptr ptr)
@ stdcall FsRtlLookupLastLargeMcbEntryAndIndex(ptr ptr ptr ptr)
@ stdcall FsRtlLookupLastMcbEntry(ptr ptr ptr)
@ stdcall FsRtlLookupMcbEntry(ptr long ptr ptr ptr)
@ stdcall FsRtlLookupPerFileObjectContext(ptr ptr ptr)
@ stdcall FsRtlLookupPerStreamContextInternal(ptr ptr ptr)
@ stdcall FsRtlMdlRead(ptr ptr long long ptr ptr)
@ stdcall FsRtlMdlReadComplete(ptr ptr)
@ stdcall FsRtlMdlReadCompleteDev(ptr ptr ptr)
@ stdcall FsRtlMdlReadDev(ptr ptr long long ptr ptr ptr)
@ stdcall FsRtlMdlWriteComplete(ptr ptr ptr)
@ stdcall FsRtlMdlWriteCompleteDev(ptr ptr ptr ptr)
@ stdcall FsRtlNormalizeNtstatus(long long)
@ stdcall FsRtlNotifyChangeDirectory(ptr ptr ptr ptr long long ptr)
@ stdcall FsRtlNotifyCleanup(ptr ptr ptr)
@ stdcall FsRtlNotifyFilterChangeDirectory(ptr ptr ptr ptr long long long ptr ptr ptr ptr)
@ stdcall FsRtlNotifyFilterReportChange(ptr ptr ptr long ptr ptr long long ptr ptr)
@ stdcall FsRtlNotifyFullChangeDirectory(ptr ptr ptr ptr long long long ptr ptr ptr)
@ stdcall FsRtlNotifyFullReportChange(ptr ptr ptr long ptr ptr long long ptr)
@ stdcall FsRtlNotifyInitializeSync(ptr)
@ stdcall FsRtlNotifyReportChange(ptr ptr ptr ptr long)
@ stdcall FsRtlNotifyUninitializeSync(ptr)
@ stdcall FsRtlNotifyVolumeEvent(ptr long)
@ stdcall FsRtlNumberOfRunsInBaseMcb(ptr)
@ stdcall FsRtlNumberOfRunsInLargeMcb(ptr)
@ stdcall FsRtlNumberOfRunsInMcb(ptr)
@ stdcall FsRtlOplockFsctrl(ptr ptr long)
@ stdcall FsRtlOplockIsFastIoPossible(ptr)
@ stdcall FsRtlPostPagingFileStackOverflow(ptr ptr ptr)
@ stdcall FsRtlPostStackOverflow(ptr ptr ptr)
@ stdcall FsRtlPrepareMdlWrite(ptr ptr long long ptr ptr)
@ stdcall FsRtlPrepareMdlWriteDev(ptr ptr long long ptr ptr ptr)
@ stdcall FsRtlPrivateLock(ptr ptr ptr ptr ptr long long long ptr ptr ptr long)
@ stdcall FsRtlProcessFileLock(ptr ptr ptr)
@ stdcall FsRtlRegisterFileSystemFilterCallbacks(ptr ptr)
@ stdcall FsRtlRegisterUncProvider(ptr ptr long)
@ stdcall FsRtlReleaseFile(ptr)
@ stdcall FsRtlRemoveBaseMcbEntry(ptr long long long long)
@ stdcall FsRtlRemoveLargeMcbEntry(ptr long long long long)
@ stdcall FsRtlRemoveMcbEntry(ptr long long)
@ stdcall FsRtlRemovePerFileObjectContext(ptr ptr ptr)
@ stdcall FsRtlRemovePerStreamContext(ptr ptr ptr)
@ stdcall FsRtlResetBaseMcb(ptr)
@ stdcall FsRtlResetLargeMcb(ptr long)
@ stdcall FsRtlSplitBaseMcb(ptr long long long long)
@ stdcall FsRtlSplitLargeMcb(ptr long long long long)
@ stdcall FsRtlSyncVolumes(long long long)
@ stdcall FsRtlTeardownPerStreamContexts(ptr)
@ stdcall FsRtlTruncateBaseMcb(ptr long long)
@ stdcall FsRtlTruncateLargeMcb(ptr long long)
@ stdcall FsRtlTruncateMcb(ptr long)
@ stdcall FsRtlUninitializeBaseMcb(ptr)
@ stdcall FsRtlUninitializeFileLock(ptr)
@ stdcall FsRtlUninitializeLargeMcb(ptr)
@ stdcall FsRtlUninitializeMcb(ptr)
@ stdcall FsRtlUninitializeOplock(ptr)
@ extern HalDispatchTable
@ fastcall HalExamineMBR(ptr long long ptr)
@ extern HalPrivateDispatchTable
@ stdcall HeadlessDispatch(long ptr long ptr ptr)
@ stdcall InbvAcquireDisplayOwnership()
@ stdcall InbvCheckDisplayOwnership()
@ stdcall InbvDisplayString(str)
@ stdcall InbvEnableBootDriver(long)
@ stdcall InbvEnableDisplayString(long)
@ stdcall InbvInstallDisplayStringFilter(ptr)
@ stdcall InbvIsBootDriverInstalled()
@ stdcall InbvNotifyDisplayOwnershipLost(ptr)
@ stdcall InbvResetDisplay()
@ stdcall InbvSetScrollRegion(long long long long)
@ stdcall InbvSetTextColor(long)
@ stdcall InbvSolidColorFill(long long long long long)
@ extern InitSafeBootMode
@ fastcall -arch=i386,arm InterlockedCompareExchange(ptr long long)
@ fastcall -arch=i386,arm InterlockedDecrement(ptr)
@ fastcall -arch=i386,arm InterlockedExchange(ptr long)
@ fastcall -arch=i386,arm InterlockedExchangeAdd(ptr long)
@ fastcall -arch=i386,arm InterlockedIncrement(ptr)
@ fastcall -arch=i386 InterlockedPopEntrySList(ptr)
@ fastcall -arch=i386 InterlockedPushEntrySList(ptr ptr)
@ stdcall -arch=arm InterlockedPopEntrySList(ptr) RtlInterlockedPopEntrySList
@ stdcall -arch=arm InterlockedPushEntrySList(ptr ptr) RtlInterlockedPushEntrySList
@ stdcall -arch=x86_64,arm64 InitializeSListHead(ptr) RtlInitializeSListHead
@ stdcall IoAcquireCancelSpinLock(ptr)
@ stdcall IoAcquireRemoveLockEx(ptr ptr str long long)
@ stdcall IoAcquireVpbSpinLock(ptr)
@ extern IoAdapterObjectType
@ stdcall IoAllocateAdapterChannel(ptr ptr long ptr ptr)
@ stdcall IoAllocateController(ptr ptr ptr ptr)
@ stdcall IoAllocateDriverObjectExtension(ptr ptr long ptr)
@ stdcall IoAllocateErrorLogEntry(ptr long)
@ stdcall IoAllocateIrp(long long)
@ stdcall IoAllocateMdl(ptr long long long ptr)
@ stdcall IoAllocateWorkItem(ptr)
@ fastcall IoAssignDriveLetters(ptr ptr ptr ptr)
@ stdcall IoAssignResources(ptr ptr ptr ptr ptr ptr)
@ stdcall IoAttachDevice(ptr ptr ptr)
@ stdcall IoAttachDeviceByPointer(ptr ptr)
@ stdcall IoAttachDeviceToDeviceStack(ptr ptr)
@ stdcall IoAttachDeviceToDeviceStackSafe(ptr ptr ptr)
@ stdcall IoBuildAsynchronousFsdRequest(long ptr ptr long ptr ptr)
@ stdcall IoBuildDeviceIoControlRequest(long ptr ptr long ptr long long ptr ptr)
@ stdcall IoBuildPartialMdl(ptr ptr ptr long)
@ stdcall IoBuildSynchronousFsdRequest(long ptr ptr long ptr ptr ptr)
@ stdcall IoCallDriver(ptr ptr)
@ stdcall IoCancelFileOpen(ptr ptr)
@ stdcall IoCancelIrp(ptr)
@ stdcall IoCheckDesiredAccess(ptr long)
@ stdcall IoCheckEaBufferValidity(ptr long ptr)
@ stdcall IoCheckFunctionAccess(long long long long ptr ptr)
@ stdcall IoCheckQuerySetFileInformation(long long long)
@ stdcall IoCheckQuerySetVolumeInformation(long long long)
@ stdcall IoCheckQuotaBufferValidity(ptr long ptr)
@ stdcall IoCheckShareAccess(long long ptr ptr long)
@ stdcall IoCompleteRequest(ptr long)
@ stdcall IoConnectInterrupt(ptr ptr ptr ptr long long long long long long long)
@ stdcall IoConnectInterruptEx(ptr)
@ stdcall IoCreateController(long)
@ stdcall IoCreateDevice(ptr long ptr long long long ptr)
@ stdcall IoCreateDisk(ptr ptr)
@ stdcall IoCreateDriver(ptr ptr)
@ stdcall IoCreateFile(ptr long ptr ptr ptr long long long long ptr long long ptr long)
@ stdcall IoCreateFileSpecifyDeviceObjectHint(ptr long ptr ptr ptr long long long long ptr long long ptr long ptr)
@ stdcall IoCreateNotificationEvent(ptr ptr)
@ stdcall IoCreateStreamFileObject(ptr ptr)
@ stdcall IoCreateStreamFileObjectEx(ptr ptr ptr)
@ stdcall IoCreateStreamFileObjectLite(ptr ptr)
@ stdcall IoCreateSymbolicLink(ptr ptr)
@ stdcall IoCreateSynchronizationEvent(ptr ptr)
@ stdcall IoCreateUnprotectedSymbolicLink(ptr ptr)
@ stdcall IoCsqInitialize(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall IoCsqInitializeEx(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall IoCsqInsertIrp(ptr ptr ptr)
@ stdcall IoCsqInsertIrpEx(ptr ptr ptr ptr)
@ stdcall IoCsqRemoveIrp(ptr ptr)
@ stdcall IoCsqRemoveNextIrp(ptr ptr)
@ stdcall IoDeleteController(ptr)
@ stdcall IoDeleteDevice(ptr)
@ stdcall IoDeleteDriver(ptr)
@ stdcall IoDeleteSymbolicLink(ptr)
@ stdcall IoDetachDevice(ptr)
@ extern IoDeviceHandlerObjectSize
@ extern IoDeviceHandlerObjectType
@ extern IoDeviceObjectType
@ stdcall IoDisconnectInterrupt(ptr)
@ stdcall IoDisconnectInterruptEx(ptr)
@ extern IoDriverObjectType
@ stdcall IoEnqueueIrp(ptr)
@ stdcall IoEnumerateDeviceObjectList(ptr ptr long ptr)
@ stdcall IoEnumerateRegisteredFiltersList(ptr long ptr)
@ stdcall IoFastQueryNetworkAttributes(ptr long long ptr ptr)
@ extern IoFileObjectType
@ stdcall IoForwardAndCatchIrp(ptr ptr) IoForwardIrpSynchronously
@ stdcall IoForwardIrpSynchronously(ptr ptr)
@ stdcall IoFreeController(ptr)
@ stdcall IoFreeErrorLogEntry(ptr)
@ stdcall IoFreeIrp(ptr)
@ stdcall IoFreeMdl(ptr)
@ stdcall IoFreeWorkItem(ptr)
@ stdcall IoGetAttachedDevice(ptr)
@ stdcall IoGetAttachedDeviceReference(ptr)
@ stdcall IoGetBaseFileSystemDeviceObject(ptr)
@ stdcall IoGetBootDiskInformation(ptr long)
@ stdcall IoGetConfigurationInformation()
@ stdcall IoGetCurrentProcess()
@ stdcall IoGetDeviceAttachmentBaseRef(ptr)
@ stdcall IoGetDeviceInterfaceAlias(ptr ptr ptr)
@ stdcall IoGetDeviceInterfaces(ptr ptr long ptr)
@ stdcall IoGetDeviceObjectPointer(ptr long ptr ptr)
@ stdcall IoGetDeviceProperty(ptr long long ptr ptr)
@ stdcall IoGetDeviceToVerify(ptr)
@ stdcall IoGetDiskDeviceObject(ptr ptr)
@ stdcall IoGetDmaAdapter(ptr ptr ptr)
@ stdcall IoGetDriverObjectExtension(ptr ptr)
@ stdcall IoGetFileObjectGenericMapping()
@ stdcall IoGetInitialStack()
@ stdcall IoGetLowerDeviceObject(ptr)
@ fastcall IoGetPagingIoPriority(ptr)
@ stdcall IoGetRelatedDeviceObject(ptr)
@ stdcall IoGetRequestorProcess(ptr)
@ stdcall IoGetRequestorProcessId(ptr)
@ stdcall IoGetRequestorSessionId(ptr ptr)
@ stdcall IoGetStackLimits(ptr ptr)
@ stdcall IoGetTopLevelIrp()
@ stdcall IoInitializeIrp(ptr long long)
@ stdcall IoInitializeRemoveLockEx(ptr long long long long)
@ stdcall IoInitializeTimer(ptr ptr ptr)
@ stdcall IoInvalidateDeviceRelations(ptr long)
@ stdcall IoInvalidateDeviceState(ptr)
@ stdcall -arch=x86_64,arm64 IoIs32bitProcess(ptr)
@ stdcall IoIsFileOriginRemote(ptr)
@ stdcall IoIsOperationSynchronous(ptr)
@ stdcall IoIsSystemThread(ptr)
@ stdcall IoIsValidNameGraftingBuffer(ptr ptr)
@ stdcall IoIsWdmVersionAvailable(long long)
@ stdcall IoMakeAssociatedIrp(ptr long)
@ stdcall IoOpenDeviceInterfaceRegistryKey(ptr long ptr)
@ stdcall IoOpenDeviceRegistryKey(ptr long long ptr)
@ stdcall IoPageRead(ptr ptr ptr ptr ptr)
@ stdcall IoPnPDeliverServicePowerNotification(long long long long)
@ stdcall IoQueryDeviceDescription(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall IoQueryFileDosDeviceName(ptr ptr)
@ stdcall IoQueryFileInformation(ptr long long ptr ptr)
@ stdcall IoQueryVolumeInformation(ptr long long ptr ptr)
@ stdcall IoQueueThreadIrp(ptr)
@ stdcall IoQueueWorkItem(ptr ptr long ptr)
@ stdcall IoRaiseHardError(ptr ptr ptr)
@ stdcall IoRaiseInformationalHardError(long ptr ptr)
@ stdcall IoReadDiskSignature(ptr long ptr)
@ extern IoReadOperationCount
@ fastcall IoReadPartitionTable(ptr long long ptr)
@ stdcall IoReadPartitionTableEx(ptr ptr)
@ extern IoReadTransferCount
@ stdcall IoRegisterBootDriverReinitialization(ptr ptr ptr)
@ stdcall IoRegisterDeviceInterface(ptr ptr ptr ptr)
@ stdcall IoRegisterDriverReinitialization(ptr ptr ptr)
@ stdcall IoRegisterFileSystem(ptr)
@ stdcall IoRegisterFsRegistrationChange(ptr ptr)
@ stdcall IoRegisterLastChanceShutdownNotification(ptr)
@ stdcall IoRegisterPlugPlayNotification(long long ptr ptr ptr ptr ptr)
@ stdcall IoRegisterShutdownNotification(ptr)
@ stdcall IoReleaseCancelSpinLock(long)
@ stdcall IoReleaseRemoveLockAndWaitEx(ptr ptr long)
@ stdcall IoReleaseRemoveLockEx(ptr ptr long)
@ stdcall IoReleaseVpbSpinLock(long)
@ stdcall IoRemoveShareAccess(ptr ptr)
@ stdcall IoReportDetectedDevice(ptr long long long ptr ptr long ptr)
@ stdcall IoReportHalResourceUsage(ptr ptr ptr long)
@ stdcall IoReportResourceForDetection(ptr ptr long ptr ptr long ptr)
@ stdcall IoReportResourceUsage(ptr ptr ptr long ptr ptr long long ptr)
@ stdcall IoReportTargetDeviceChange(ptr ptr)
@ stdcall IoReportTargetDeviceChangeAsynchronous(ptr ptr ptr ptr)
@ stdcall IoRequestDeviceEject(ptr)
@ stdcall IoReuseIrp(ptr long)
@ stdcall IoSetCompletionRoutineEx(ptr ptr ptr ptr long long long)
@ stdcall IoSetDeviceInterfaceState(ptr long)
@ stdcall IoSetDeviceToVerify(ptr ptr)
@ stdcall IoSetFileOrigin(ptr long)
@ stdcall IoSetHardErrorOrVerifyDevice(ptr ptr)
@ stdcall IoSetInformation(ptr ptr long ptr)
@ stdcall IoSetIoCompletion(ptr ptr ptr long ptr long)
@ fastcall IoSetPartitionInformation(ptr long long long)
@ stdcall IoSetPartitionInformationEx(ptr long ptr)
@ stdcall IoSetShareAccess(long long ptr ptr)
@ stdcall IoSetStartIoAttributes(ptr long long)
@ stdcall IoSetSystemPartition(ptr)
@ stdcall IoSetThreadHardErrorMode(long)
@ stdcall IoSetTopLevelIrp(ptr)
@ stdcall IoStartNextPacket(ptr long)
@ stdcall IoStartNextPacketByKey(ptr long long)
@ stdcall IoStartPacket(ptr ptr ptr ptr)
@ stdcall IoStartTimer(ptr)
@ extern IoStatisticsLock
@ stdcall IoStopTimer(ptr)
@ stdcall IoSynchronousInvalidateDeviceRelations(ptr long)
@ stdcall IoSynchronousPageWrite(ptr ptr ptr ptr ptr)
@ stdcall IoThreadToProcess(ptr)
@ stdcall IoTranslateBusAddress(long long long long ptr ptr)
@ stdcall IoUnregisterFileSystem(ptr)
@ stdcall IoUnregisterFsRegistrationChange(ptr ptr)
@ stdcall IoUnregisterPlugPlayNotification(ptr)
@ stdcall IoUnregisterShutdownNotification(ptr)
@ stdcall IoUpdateShareAccess(ptr ptr)
@ stdcall IoValidateDeviceIoControlAccess(ptr long)
@ stdcall IoVerifyPartitionTable(ptr long)
@ stdcall IoVerifyVolume(ptr long)
@ stdcall IoVolumeDeviceToDosName(ptr ptr)
@ stdcall IoWMIAllocateInstanceIds(ptr long ptr)
@ stdcall IoWMIDeviceObjectToInstanceName(ptr ptr ptr)
@ stdcall -arch=x86_64,arm64 IoWMIDeviceObjectToProviderId(ptr)
@ stdcall IoWMIExecuteMethod(ptr ptr long long ptr ptr)
@ stdcall IoWMIHandleToInstanceName(ptr ptr ptr)
@ stdcall IoWMIOpenBlock(ptr long ptr)
@ stdcall IoWMIQueryAllData(ptr ptr ptr)
@ stdcall IoWMIQueryAllDataMultiple(ptr long ptr ptr)
@ stdcall IoWMIQuerySingleInstance(ptr ptr ptr ptr)
@ stdcall IoWMIQuerySingleInstanceMultiple(ptr ptr long ptr ptr)
@ stdcall IoWMIRegistrationControl(ptr long)
@ stdcall IoWMISetNotificationCallback(ptr ptr ptr)
@ stdcall IoWMISetSingleInstance(ptr ptr long long ptr)
@ stdcall IoWMISetSingleItem(ptr ptr long long long ptr)
@ stdcall IoWMISuggestInstanceName(ptr ptr long ptr)
@ stdcall IoWMIWriteEvent(ptr)
@ stdcall IoWriteErrorLogEntry(ptr)
@ extern IoWriteOperationCount
@ fastcall IoWritePartitionTable(ptr long long long ptr)
@ stdcall IoWritePartitionTableEx(ptr ptr)
@ extern IoWriteTransferCount
@ fastcall IofCallDriver(ptr ptr)
@ fastcall IofCompleteRequest(ptr long)
@ stdcall KdChangeOption(long long ptr long ptr ptr)
@ extern KdDebuggerEnabled
@ extern KdDebuggerNotPresent
@ stdcall KdDisableDebugger()
@ stdcall KdEnableDebugger()
@ extern KdEnteredDebugger
@ stdcall KdPollBreakIn()
@ stdcall KdPowerTransition(long)
@ stdcall KdRefreshDebuggerNotPresent()
@ stdcall KdSystemDebugControl(long ptr long ptr long ptr long)
@ stdcall -arch=i386 Ke386CallBios(long ptr)
@ stdcall -arch=i386 Ke386IoSetAccessProcess(ptr long)
@ stdcall -arch=i386 Ke386QueryIoAccessMap(long ptr)
@ stdcall -arch=i386 Ke386SetIoAccessMap(long ptr)
@ fastcall KeAcquireGuardedMutex(ptr)
@ fastcall KeAcquireGuardedMutexUnsafe(ptr)
@ cdecl -arch=x86_64 KeAcquireInStackQueuedSpinLock(ptr ptr)
@ fastcall KeAcquireInStackQueuedSpinLockAtDpcLevel(ptr ptr)
@ fastcall KeAcquireInStackQueuedSpinLockForDpc(ptr ptr)
@ cdecl -arch=x86_64 KeAcquireInStackQueuedSpinLockRaiseToSynch(ptr ptr)
@ stdcall KeAcquireInterruptSpinLock(ptr)
@ cdecl -arch=x86_64 KeAcquireQueuedSpinLock(long)
@ cdecl -arch=x86_64 KeAcquireQueuedSpinLockRaiseToSynch(long)
@ stdcall KeAcquireSpinLockAtDpcLevel(ptr)
@ fastcall KeAcquireSpinLockForDpc(ptr)
@ stdcall -arch=x86_64,arm64 KeAcquireSpinLockRaiseToDpc(ptr)
@ stdcall -arch=x86_64,arm64 KeAcquireSpinLockRaiseToSynch(ptr)
@ stdcall KeAddSystemServiceTable(ptr ptr long ptr long)
@ stdcall KeAreAllApcsDisabled()
@ stdcall KeAreApcsDisabled()
@ stdcall KeAttachProcess(ptr)
@ stdcall KeBugCheck(long)
@ stdcall KeBugCheckEx(long ptr ptr ptr ptr)
@ stdcall KeCancelTimer(ptr)
@ stdcall KeCapturePersistentThreadState(ptr long long long long long ptr)
@ stdcall KeClearEvent(ptr)
@ stdcall KeConnectInterrupt(ptr)
@ stdcall KeDelayExecutionThread(long long ptr)
@ stdcall KeDeregisterBugCheckCallback(ptr)
@ stdcall KeDeregisterBugCheckReasonCallback(ptr)
@ stdcall KeDeregisterNmiCallback(ptr)
@ stdcall KeDetachProcess()
@ stdcall KeDisconnectInterrupt(ptr)
@ stdcall KeEnterCriticalRegion() _KeEnterCriticalRegion
@ stdcall KeEnterGuardedRegion() _KeEnterGuardedRegion
@ stdcall KeEnterKernelDebugger()
;@ stdcall -arch=x86_64 KeExpandKernelStackAndCallout(ptr ptr double)
@ stdcall KeFindConfigurationEntry(ptr long long ptr)
@ stdcall KeFindConfigurationNextEntry(ptr long long ptr ptr)
@ stdcall KeFlushEntireTb(long long)
@ stdcall -arch=arm KeFlushIoBuffers(ptr long long)
@ stdcall KeFlushQueuedDpcs()
@ stdcall KeGenericCallDpc(ptr ptr)
@ stdcall -arch=x86_64 KeGetCurrentIrql()
@ stdcall KeGetCurrentProcessorNumberEx(ptr)
@ stdcall KeGetCurrentThread()
@ stdcall KeGetPreviousMode()
@ stdcall KeGetRecommendedSharedDataAlignment()
@ stdcall -arch=i386 KeI386AbiosCall(long ptr ptr long)
@ stdcall -arch=i386 KeI386AllocateGdtSelectors(ptr long)
@ stdcall -arch=i386 KeI386Call16BitCStyleFunction(long long ptr long)
@ stdcall -arch=i386 KeI386Call16BitFunction(ptr)
@ stdcall -arch=i386 KeI386FlatToGdtSelector(long long long)
@ stdcall -arch=i386 KeI386GetLid(long long long ptr ptr)
@ extern -arch=i386 KeI386MachineType
@ stdcall -arch=i386 KeI386ReleaseGdtSelectors(ptr long)
@ stdcall -arch=i386 KeI386ReleaseLid(long ptr)
@ stdcall -arch=i386 KeI386SetGdtSelector(long ptr)
@ stdcall KeInitializeApc(ptr ptr long ptr ptr ptr long ptr)
@ stdcall KeInitializeCrashDumpHeader(long long ptr long ptr)
@ stdcall KeInitializeDeviceQueue(ptr)
@ stdcall KeInitializeDpc(ptr ptr ptr)
@ stdcall KeInitializeEvent(ptr long long)
@ fastcall KeInitializeGuardedMutex(ptr)
@ stdcall KeInitializeInterrupt(ptr ptr ptr ptr long long long long long long long)
@ stdcall KeInitializeMutant(ptr long)
@ stdcall KeInitializeMutex(ptr long)
@ stdcall KeInitializeQueue(ptr long)
@ stdcall KeInitializeSemaphore(ptr long long)
@ stdcall -arch=i386,arm KeInitializeSpinLock(ptr) _KeInitializeSpinLock
@ stdcall -arch=x86_64 KeInitializeSpinLock(ptr)
@ stdcall KeInitializeThreadedDpc(ptr ptr ptr)
@ stdcall KeInitializeTimer(ptr)
@ stdcall KeInitializeTimerEx(ptr long)
@ stdcall KeInsertByKeyDeviceQueue(ptr ptr long)
@ stdcall KeInsertDeviceQueue(ptr ptr)
@ stdcall KeInsertHeadQueue(ptr ptr)
@ stdcall KeInsertQueue(ptr ptr)
@ stdcall KeInsertQueueApc(ptr ptr ptr long)
@ stdcall KeInsertQueueDpc(ptr ptr ptr)
@ stdcall KeInvalidateAllCaches()
@ stdcall KeIpiGenericCall(ptr ptr)
@ stdcall KeIsAttachedProcess()
@ stdcall -arch=i386,arm KeIsExecutingDpc()
@ stdcall KeIsWaitListEmpty(ptr)
;@ cdecl -arch=x86_64 KeLastBranchMSR()
@ stdcall KeLeaveCriticalRegion() _KeLeaveCriticalRegion
@ stdcall KeLeaveGuardedRegion() _KeLeaveGuardedRegion
@ extern KeLoaderBlock
@ cdecl -arch=x86_64 -private KeLowerIrql(long) KxLowerIrql
@ extern KeNumberProcessors
@ stdcall -arch=i386,arm KeProfileInterrupt(ptr)
@ stdcall KeProfileInterruptWithSource(ptr long)
@ stdcall KePulseEvent(ptr long long)
@ stdcall KeQueryActiveProcessors()
@ stdcall -arch=i386,arm KeQueryInterruptTime()
;@ cdecl -arch=x86_64 KeQueryMultiThreadProcessorSet
;@ cdecl -arch=x86_64 KeQueryPrcbAddress
@ stdcall KeQueryPriorityThread(ptr)
@ stdcall KeQueryRuntimeThread(ptr ptr)
@ stdcall -arch=i386,arm KeQuerySystemTime(ptr)
@ stdcall -arch=i386,arm KeQueryTickCount(ptr)
@ stdcall KeQueryTimeIncrement()
@ cdecl -arch=x86_64 -private KeRaiseIrqlToDpcLevel() KxRaiseIrqlToDpcLevel
@ cdecl -arch=x86_64 KeRaiseIrqlToSynchLevel()
@ stdcall KeRaiseUserException(long)
@ stdcall KeReadStateEvent(ptr)
@ stdcall KeReadStateMutant(ptr)
@ stdcall KeReadStateMutex(ptr) KeReadStateMutant
@ stdcall KeReadStateQueue(ptr)
@ stdcall KeReadStateSemaphore(ptr)
@ stdcall KeReadStateTimer(ptr)
@ stdcall KeRegisterBugCheckCallback(ptr ptr ptr long ptr)
@ stdcall KeRegisterBugCheckReasonCallback(ptr ptr ptr ptr)
@ stdcall KeRegisterNmiCallback(ptr ptr)
@ fastcall KeReleaseGuardedMutex(ptr)
@ fastcall KeReleaseGuardedMutexUnsafe(ptr)
@ cdecl -arch=x86_64 KeReleaseInStackQueuedSpinLock(ptr)
@ fastcall KeReleaseInStackQueuedSpinLockForDpc(ptr)
@ fastcall KeReleaseInStackQueuedSpinLockFromDpcLevel(ptr)
@ stdcall KeReleaseInterruptSpinLock(ptr long)
@ stdcall KeReleaseMutant(ptr long long long)
@ stdcall KeReleaseMutex(ptr long)
@ cdecl -arch=x86_64 KeReleaseQueuedSpinLock(long long)
@ stdcall KeReleaseSemaphore(ptr long long long)
@ stdcall -arch=x86_64 KeReleaseSpinLock(ptr long)
@ fastcall KeReleaseSpinLockForDpc(ptr long)
@ stdcall KeReleaseSpinLockFromDpcLevel(ptr)
@ stdcall KeRemoveByKeyDeviceQueue(ptr long)
@ stdcall KeRemoveByKeyDeviceQueueIfBusy(ptr long)
@ stdcall KeRemoveDeviceQueue(ptr)
@ stdcall KeRemoveEntryDeviceQueue(ptr ptr)
@ stdcall KeRemoveQueue(ptr long ptr)
@ stdcall KeRemoveQueueEx(ptr long long ptr ptr long)
@ stdcall KeRemoveQueueDpc(ptr)
@ stdcall KeRemoveSystemServiceTable(long)
@ stdcall KeResetEvent(ptr)
@ stdcall -arch=i386 KeRestoreFloatingPointState(ptr)
@ stdcall -arch=x86_64 KeRestoreFloatingPointState(ptr) KxRestoreFloatingPointState
@ stdcall KeRevertToUserAffinityThread()
@ stdcall KeRundownQueue(ptr)
@ stdcall -arch=i386 KeSaveFloatingPointState(ptr)
@ stdcall -arch=x86_64 KeSaveFloatingPointState(ptr) KxSaveFloatingPointState
@ cdecl KeSaveStateForHibernate(ptr)
@ extern KeServiceDescriptorTable
@ stdcall KeSetAffinityThread(ptr long)
@ stdcall KeSetBasePriorityThread(ptr long)
@ stdcall KeSetDmaIoCoherency(long)
@ stdcall KeSetEvent(ptr long long)
@ stdcall KeSetEventBoostPriority(ptr ptr)
@ stdcall KeSetIdealProcessorThread(ptr long)
@ stdcall KeSetImportanceDpc(ptr long)
@ stdcall KeSetKernelStackSwapEnable(long)
@ stdcall KeSetPriorityThread(ptr long)
@ stdcall KeSetProfileIrql(long)
@ stdcall KeSetSystemAffinityThread(long)
@ stdcall KeSetTargetProcessorDpc(ptr long)
@ stdcall KeSetTimeIncrement(long long)
@ stdcall KeSetTimer(ptr long long ptr)
@ stdcall KeSetTimerEx(ptr long long long ptr)
@ stdcall KeSignalCallDpcDone(ptr)
@ stdcall KeSignalCallDpcSynchronize(ptr)
@ stdcall KeStackAttachProcess(ptr ptr)
@ stdcall KeSynchronizeExecution(ptr ptr ptr)
@ stdcall KeTerminateThread(long)
@ fastcall KeTestSpinLock(ptr)
@ extern -arch=i386,arm,x86_64 KeTickCount
@ fastcall KeTryToAcquireGuardedMutex(ptr)
@ cdecl -arch=x86_64 KeTryToAcquireQueuedSpinLock(long long)
@ cdecl -arch=x86_64 KeTryToAcquireQueuedSpinLockRaiseToSynch(long long)
@ fastcall KeTryToAcquireSpinLockAtDpcLevel(ptr)
@ stdcall KeUnstackDetachProcess(ptr)
@ stdcall KeUpdateRunTime(ptr long)
@ fastcall KeUpdateSystemTime(ptr long long)
@ stdcall KeUserModeCallback(long ptr long ptr ptr)
@ stdcall KeWaitForMultipleObjects(long ptr long long long long ptr ptr)
@ stdcall KeWaitForMutexObject(ptr long long long ptr) KeWaitForSingleObject
@ stdcall KeWaitForSingleObject(ptr long long long ptr)
@ fastcall -arch=i386,arm KefAcquireSpinLockAtDpcLevel(ptr)
@ fastcall -arch=i386,arm KefReleaseSpinLockFromDpcLevel(ptr)
@ stdcall -arch=i386 Kei386EoiHelper()
@ cdecl -arch=x86_64 -private KfRaiseIrql(long) KxRaiseIrql
@ fastcall -arch=i386 KiEoiHelper(ptr) #ReactOS-Specific
@ fastcall -arch=i386,arm KiAcquireSpinLock(ptr)
@ extern KiBugCheckData
@ stdcall KiCheckForKernelApcDelivery()
@ fastcall -arch=i386 KiCheckForSListAddress(ptr)
@ stdcall -arch=i386 KiCoprocessorError()
;@ cdecl -arch=x86_64 KiCpuId()
@ stdcall -arch=i386,arm,arm64 KiDeliverApc(long ptr ptr)
@ stdcall -arch=i386 KiDispatchInterrupt()
@ stdcall -arch=arm64 KiDispatchInterruptArm64(long)
@ stdcall -arch=arm64 KiDispatchPreemptArm64()
@ stdcall -arch=arm64 KiDispatchSoftwareIntArm64()
@ extern -arch=i386,arm,x86_64 KiEnableTimerWatchdog
@ stdcall -arch=i386,arm,arm64 KiIpiServiceRoutine(ptr ptr)
@ fastcall -arch=i386,arm KiReleaseSpinLock(ptr)
@ cdecl -arch=i386,arm KiUnexpectedInterrupt()
@ stdcall -arch=i386 Kii386SpinOnSpinLock(ptr long)
# vibeos-internal, not an NT API: the fault-tolerant kernel copy (returns 1 on
# success, 0 if the copy faulted). Exported because win32k.sys is a separate PE
# and needs it for _MmCopy{From,To}Caller — and it CANNOT carry its own copy on
# i386, where the primitive is a fault-fixup whose exception table lives in the
# kernel's KiTrap0EHandler and keys on a link-time PC (field manual §9.122).
# amd64/arm64: the native-SEH shim in ntoskrnl/kmsafecopy.c. i386: the extable
# primitive in ntoskrnl-rust ke/i386/safecopy.rs.
# NOT `-private`: spec2def skips private exports when generating the IMPORT LIB
# (spec2def.c:750), so win32k.sys would fail to link against it.
@ cdecl KmProtectedCopyMemory(ptr ptr long)
@ cdecl KmProtectedInterlockedAdd(ptr long ptr)
# vibeos-internal: MmProbeAndLockPages returning its status instead of raising it.
# Without kernel SEH a raise from the NT export is a bugcheck, so every Rust caller
# uses this; it is exported for the drivers that are separate PEs (mup, ks, afd,
# cdfs, cdrom, win32k). Not `-private`, for the same import-lib reason as above.
@ stdcall MmTryProbeAndLockPages(ptr long long)
@ stdcall -arch=win64 RtlArmFeatureUsageProviderFlushNotification()
@ stdcall RtlNotifyFeatureUsage(ptr)
@ stdcall RtlQueryFeatureConfiguration(long long ptr ptr)
@ stdcall -ret64 RtlQueryFeatureConfigurationChangeStamp()
@ stdcall -arch=win64 RtlRecordFeatureUsage()
@ stdcall RtlRegisterFeatureConfigurationChangeNotification(ptr ptr ptr ptr)
@ stdcall -arch=win64 RtlRegisterFeatureUsageProvider()
@ cdecl RtlSehTryExcept(ptr ptr ptr ptr ptr)
@ cdecl RtlSehTryFinally(ptr ptr ptr ptr)
@ stdcall LdrAccessResource(ptr ptr ptr ptr)
@ stdcall LdrEnumResources(ptr ptr long ptr ptr)
@ stdcall LdrFindResourceDirectory_U(ptr ptr long ptr)
@ stdcall LdrFindResource_U(ptr ptr long ptr)
@ extern LpcPortObjectType
@ stdcall LpcRequestPort(ptr ptr)
@ stdcall LpcRequestWaitReplyPort(ptr ptr ptr)
@ stdcall LsaCallAuthenticationPackage(long long ptr long ptr ptr ptr)
@ stdcall LsaDeregisterLogonProcess(long)
@ stdcall LsaFreeReturnBuffer(ptr)
@ stdcall LsaLogonUser(long ptr long long ptr long ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall LsaLookupAuthenticationPackage(long ptr ptr)
@ stdcall LsaRegisterLogonProcess(ptr ptr ptr)
@ extern Mm64BitPhysicalAddress
@ stdcall MmAddPhysicalMemory(ptr ptr)
@ stdcall MmAddVerifierThunks(ptr long)
@ stdcall MmAdjustWorkingSetSize(long long long long)
@ stdcall MmAdvanceMdl(ptr long)
@ stdcall MmAllocateContiguousMemory(long long long)
@ stdcall MmAllocateContiguousMemorySpecifyCache(long long long long long long long long)
@ stdcall MmAllocateMappingAddress(long long)
@ stdcall MmAllocateNonCachedMemory(long)
@ stdcall MmAllocatePagesForMdl(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall MmAllocatePagesForMdlEx(long long long long long long long long long)
@ stdcall MmBuildMdlForNonPagedPool(ptr)
@ stdcall MmCanFileBeTruncated(ptr ptr)
@ stdcall MmCommitSessionMappedView(ptr ptr)
@ stdcall MmCreateMdl(ptr ptr long)
@ stdcall MmCreateMirror()
@ stdcall MmCreateSection(ptr long ptr ptr long long ptr ptr)
@ stdcall MmDisableModifiedWriteOfSection(long)
@ stdcall MmDoesFileHaveUserWritableReferences(ptr)
@ stdcall MmFlushImageSection(ptr long)
@ stdcall MmForceSectionClosed(ptr long)
@ stdcall MmFreeContiguousMemory(ptr)
@ stdcall MmFreeContiguousMemorySpecifyCache(ptr long long)
@ stdcall MmFreeMappingAddress(ptr long)
@ stdcall MmFreeNonCachedMemory(ptr long)
@ stdcall MmFreePagesFromMdl(ptr)
@ stdcall MmGetPhysicalAddress(ptr)
@ stdcall MmGetPhysicalMemoryRanges()
@ stdcall MmGetSystemRoutineAddress(ptr)
@ stdcall MmGetVirtualForPhysical(long long)
@ stdcall MmGrowKernelStack(ptr)
@ extern MmHighestUserAddress
@ stdcall MmIsAddressValid(ptr)
@ stdcall MmIsDriverVerifying(ptr)
@ stdcall MmIsIoSpaceActive(long long ptr)
@ stdcall MmIsNonPagedSystemAddressValid(ptr)
@ stdcall MmIsRecursiveIoFault()
@ stdcall MmIsThisAnNtAsSystem()
@ stdcall MmIsVerifierEnabled(ptr)
@ stdcall MmLockPagableDataSection(ptr) MmLockPageableDataSection
@ stdcall MmLockPagableImageSection(ptr) MmLockPageableDataSection
@ stdcall MmLockPagableSectionByHandle(ptr) MmLockPageableSectionByHandle
@ stdcall MmMapIoSpace(long long long long)
@ stdcall MmMapIoSpaceEx(int64 long long)
@ stdcall MmMapLockedPages(ptr long)
@ stdcall MmMapLockedPagesSpecifyCache(ptr long long ptr long long)
@ stdcall MmMapLockedPagesWithReservedMapping(ptr long ptr long)
@ stdcall MmMapMemoryDumpMdl(ptr)
@ stdcall MmMapUserAddressesToPage(ptr long ptr)
@ stdcall MmMapVideoDisplay(long long long long)
@ stdcall MmMapViewInSessionSpace(ptr ptr ptr)
@ stdcall MmMapViewInSystemSpace(ptr ptr ptr)
@ stdcall MmMapViewOfSection(ptr ptr ptr long long ptr ptr long long long)
@ stdcall MmMarkPhysicalMemoryAsBad(ptr ptr)
@ stdcall MmMarkPhysicalMemoryAsGood(ptr ptr)
@ stdcall MmPageEntireDriver(ptr)
@ stdcall MmPrefetchPages(long ptr)
@ stdcall MmProbeAndLockPages(ptr long long)
@ stdcall MmProbeAndLockProcessPages(ptr ptr long long)
@ stdcall MmProbeAndLockSelectedPages(ptr ptr long long)
@ stdcall MmProtectMdlSystemAddress(ptr long)
@ stdcall MmQuerySystemSize()
@ stdcall MmRemovePhysicalMemory(ptr ptr)
@ stdcall MmResetDriverPaging(ptr)
@ extern MmSectionObjectType
@ stdcall MmSecureVirtualMemory(ptr long long)
@ stdcall MmSetAddressRangeModified(ptr long)
@ stdcall MmSetBankedSection(long long long long long long)
@ stdcall MmSizeOfMdl(ptr long)
@ extern MmSystemRangeStart
@ stdcall MmTrimAllSystemPagableMemory(long) MmTrimAllSystemPageableMemory
@ stdcall MmUnlockPagableImageSection(ptr) MmUnlockPageableImageSection
@ stdcall MmUnlockPages(ptr)
@ stdcall MmUnmapIoSpace(ptr long)
@ stdcall MmUnmapLockedPages(ptr ptr)
@ stdcall MmUnmapReservedMapping(ptr long ptr)
@ stdcall MmUnmapVideoDisplay(ptr long)
@ stdcall MmUnmapViewInSessionSpace(ptr)
@ stdcall MmUnmapViewInSystemSpace(ptr)
@ stdcall MmUnmapViewOfSection(ptr ptr)
@ stdcall MmUnsecureVirtualMemory(ptr)
@ extern MmUserProbeAddress
@ extern NlsAnsiCodePage
@ extern NlsLeadByteInfo
@ extern NlsMbCodePageTag
@ extern NlsMbOemCodePageTag
@ extern NlsOemCodePage
@ extern NlsOemLeadByteInfo
@ stdcall NtAddAtom(wstr long ptr)
@ stdcall NtAdjustPrivilegesToken(ptr long ptr long ptr ptr)
@ stdcall NtAllocateLocallyUniqueId(ptr)
@ stdcall NtAllocateUuids(ptr ptr ptr ptr)
@ stdcall NtAllocateVirtualMemory(ptr ptr long ptr long long)
@ extern NtBuildNumber
@ stdcall NtClose(ptr)
@ stdcall NtConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtCreateEvent(ptr long ptr long long)
@ stdcall NtCreateFile(ptr long ptr ptr ptr long long long long ptr long)
@ stdcall NtCreateSection(ptr long ptr ptr long long ptr)
@ stdcall NtDeleteAtom(ptr)
@ stdcall NtDeleteFile(ptr)
@ stdcall NtDeviceIoControlFile(ptr ptr ptr ptr ptr long ptr long ptr long)
@ stdcall NtDuplicateObject(ptr ptr ptr ptr long long long)
@ stdcall NtDuplicateToken(ptr long ptr long long ptr)
@ stdcall NtFindAtom(wstr long ptr)
@ stdcall NtFreeVirtualMemory(ptr ptr ptr long)
@ stdcall NtFsControlFile(ptr ptr ptr ptr ptr long ptr long ptr long)
@ extern NtGlobalFlag
@ stdcall NtLockFile(ptr ptr ptr ptr ptr ptr ptr long long long)
@ stdcall NtMakePermanentObject(ptr)
@ stdcall NtMapViewOfSection(ptr ptr ptr long long ptr ptr long long long)
@ stdcall NtNotifyChangeDirectoryFile(ptr ptr ptr ptr ptr ptr long long long)
@ stdcall NtOpenFile(ptr long ptr ptr long long)
@ stdcall NtOpenProcess(ptr long ptr ptr)
@ stdcall NtOpenProcessToken(ptr long ptr)
@ stdcall NtOpenProcessTokenEx(ptr long long ptr)
@ stdcall NtOpenThread(ptr long ptr ptr)
@ stdcall NtOpenThreadToken(ptr long long ptr)
@ stdcall NtOpenThreadTokenEx(ptr long long long ptr)
@ stdcall NtQueryDirectoryFile(ptr ptr ptr ptr ptr ptr long long long ptr long)
@ stdcall NtQueryEaFile(ptr ptr ptr long long ptr long ptr long)
@ stdcall NtQueryInformationAtom(long long ptr long ptr)
@ stdcall NtQueryInformationFile(ptr ptr ptr long long)
@ stdcall NtQueryInformationProcess(ptr long ptr long ptr)
@ stdcall NtQueryInformationThread(ptr long ptr long ptr)
@ stdcall NtQueryInformationToken(ptr long ptr long ptr)
@ stdcall NtQueryQuotaInformationFile(ptr ptr ptr long long ptr long ptr long)
@ stdcall NtQuerySecurityObject(ptr long ptr long ptr)
@ stdcall NtQuerySystemInformation(long ptr long ptr)
@ stdcall NtQueryVolumeInformationFile(ptr ptr ptr long long)
@ stdcall NtReadFile(ptr ptr ptr ptr ptr ptr long ptr ptr)
@ stdcall NtRequestPort(ptr ptr)
@ stdcall NtRequestWaitReplyPort(ptr ptr ptr)
@ stdcall NtSetEaFile(ptr ptr ptr long)
@ stdcall NtSetEvent(ptr ptr)
@ stdcall NtSetInformationFile(ptr ptr ptr long long)
@ stdcall NtSetInformationProcess(ptr long ptr long)
@ stdcall NtSetInformationThread(ptr long ptr long)
@ stdcall NtSetQuotaInformationFile(ptr ptr ptr long)
@ stdcall NtSetSecurityObject(ptr long ptr)
@ stdcall NtSetVolumeInformationFile(ptr ptr ptr long long)
@ stdcall NtShutdownSystem(long)
@ stdcall NtTraceEvent(long long long ptr)
@ stdcall NtUnlockFile(ptr ptr ptr ptr long)
@ stdcall NtVdmControl(long ptr)
@ stdcall NtWaitForSingleObject(ptr long ptr)
@ stdcall NtWriteFile(ptr ptr ptr ptr ptr ptr long ptr ptr)
@ stdcall ObAssignSecurity(ptr ptr ptr ptr)
@ stdcall ObCheckCreateObjectAccess(ptr long ptr ptr long long ptr)
@ stdcall ObCheckObjectAccess(ptr ptr long long ptr)
@ stdcall ObCloseHandle(ptr long)
@ stdcall ObCreateObject(long ptr ptr long ptr long long long ptr)
@ stdcall ObCreateObjectType(ptr ptr ptr ptr)
@ stdcall ObDeleteCapturedInsertInfo(ptr)
@ stdcall ObDereferenceObject(ptr)
@ stdcall ObDereferenceSecurityDescriptor(ptr long)
@ stdcall ObFindHandleForObject(ptr ptr ptr ptr ptr)
@ stdcall ObGetObjectSecurity(ptr ptr ptr)
@ stdcall ObInsertObject(ptr ptr long long ptr ptr)
@ stdcall ObLogSecurityDescriptor(ptr ptr long)
@ stdcall ObMakeTemporaryObject(ptr)
@ stdcall ObOpenObjectByName(ptr ptr long ptr long ptr ptr)
@ stdcall ObOpenObjectByPointer(ptr long ptr long ptr long ptr)
@ stdcall ObQueryNameString(ptr ptr long ptr)
@ stdcall ObQueryObjectAuditingByHandle(ptr ptr)
@ stdcall ObReferenceObjectByHandle(ptr long ptr long ptr ptr)
@ stdcall ObReferenceObjectByName(ptr long ptr long ptr long ptr ptr)
@ stdcall ObReferenceObjectByPointer(ptr long ptr long)
@ stdcall ObReferenceSecurityDescriptor(ptr long)
@ stdcall ObReleaseObjectSecurity(ptr long)
@ stdcall ObSetHandleAttributes(ptr ptr long)
@ stdcall ObSetSecurityDescriptorInfo(ptr ptr ptr ptr long ptr)
@ stdcall ObSetSecurityObjectByPointer(ptr long ptr)
@ fastcall ObfDereferenceObject(ptr)
@ fastcall ObfReferenceObject(ptr)
@ stdcall PfxFindPrefix(ptr ptr)
@ stdcall PfxInitialize(ptr)
@ stdcall PfxInsertPrefix(ptr ptr ptr)
@ stdcall PfxRemovePrefix(ptr ptr)
@ stdcall PoCallDriver(ptr ptr)
@ stdcall PoCancelDeviceNotify(ptr)
@ stdcall -version=0x602+ PoFxActivateComponent(ptr long long)
@ stdcall -version=0x602+ PoFxCompleteDevicePowerNotRequired(ptr)
@ stdcall -version=0x602+ PoFxCompleteIdleCondition(ptr long)
@ stdcall -version=0x602+ PoFxCompleteIdleState(ptr long)
@ stdcall -version=0x602+ PoFxIdleComponent(ptr ptr long)
@ stdcall -version=0x602+ PoFxRegisterDevice(ptr ptr ptr)
@ stdcall -version=0x602+ PoFxReportDevicePoweredOn(ptr)
@ stdcall -version=0x602+ PoFxSetDeviceIdleTimeout(ptr int64)
@ stdcall -version=0x602+ PoFxStartDevicePowerManagement(ptr)
@ stdcall -version=0x602+ PoFxUnregisterDevice(ptr)
@ stdcall PoQueueShutdownWorkItem(ptr)
@ stdcall PoRegisterDeviceForIdleDetection(ptr long long long)
@ stdcall PoRegisterDeviceNotify(ptr long long long ptr ptr)
@ stdcall PoRegisterSystemState(ptr long)
@ stdcall PoRequestPowerIrp(ptr long long ptr ptr ptr)
@ stdcall PoRequestShutdownEvent(ptr)
@ stdcall PoSetHiberRange(ptr long ptr long long)
@ stdcall PoSetPowerState(ptr long long)
@ stdcall PoSetSystemState(long)
@ stdcall PoShutdownBugCheck(long long ptr ptr ptr ptr)
@ stdcall PoStartNextPowerIrp(ptr)
@ stdcall PoUnregisterSystemState(ptr)
@ stdcall ProbeForRead(ptr long long)
@ stdcall ProbeForWrite(ptr long long)
@ stdcall PsAssignImpersonationToken(ptr ptr)
@ stdcall PsChargePoolQuota(ptr long long)
@ stdcall PsChargeProcessNonPagedPoolQuota(ptr long)
@ stdcall PsChargeProcessPagedPoolQuota(ptr long)
@ stdcall PsChargeProcessPoolQuota(ptr long long)
@ stdcall PsCreateSystemProcess(ptr long ptr)
@ stdcall PsCreateSystemThread(ptr long ptr ptr ptr ptr ptr)
@ stdcall PsDereferenceImpersonationToken(ptr) PsDereferencePrimaryToken
@ stdcall PsDereferencePrimaryToken(ptr)
@ stdcall PsDisableImpersonation(ptr ptr)
@ stdcall PsEstablishWin32Callouts(ptr)
@ stdcall PsGetContextThread(ptr ptr long)
@ stdcall PsGetCurrentProcess() IoGetCurrentProcess
@ stdcall PsGetCurrentProcessId()
@ stdcall PsGetCurrentProcessSessionId()
@ stdcall PsGetCurrentProcessWin32Process()
;@ cdecl -arch=x86_64 PsGetCurrentProcessWow64Process()
@ stdcall PsGetCurrentThread() KeGetCurrentThread
@ stdcall PsGetCurrentThreadId()
@ stdcall PsGetCurrentThreadPreviousMode()
@ stdcall PsGetCurrentThreadProcess()
@ stdcall PsGetCurrentThreadProcessId()
@ stdcall PsGetCurrentThreadStackBase()
@ stdcall PsGetCurrentThreadStackLimit()
@ stdcall PsGetCurrentThreadTeb()
@ stdcall PsGetCurrentThreadWin32Thread()
@ stdcall PsGetCurrentThreadWin32ThreadAndEnterCriticalRegion(ptr)
@ stdcall PsGetJobLock(ptr)
@ stdcall PsGetJobSessionId(ptr)
@ stdcall PsGetJobUIRestrictionsClass(ptr)
@ stdcall PsGetProcessCreateTimeQuadPart(ptr)
@ stdcall PsGetProcessDebugPort(ptr)
@ stdcall PsGetProcessExitProcessCalled(ptr)
@ stdcall PsGetProcessExitStatus(ptr)
@ stdcall PsGetProcessExitTime()
@ stdcall PsGetProcessId(ptr)
@ stdcall PsGetProcessImageFileName(ptr)
@ stdcall PsGetProcessInheritedFromUniqueProcessId(ptr)
@ stdcall PsGetProcessJob(ptr)
@ stdcall PsGetProcessPeb(ptr)
@ stdcall PsGetProcessPriorityClass(ptr)
@ stdcall PsGetProcessSectionBaseAddress(ptr)
@ stdcall PsGetProcessSecurityPort(ptr)
@ stdcall PsGetProcessSessionId(ptr)
@ stdcall PsGetProcessSessionIdEx(ptr)
@ stdcall PsGetProcessWin32Process(ptr)
@ stdcall PsGetProcessWin32WindowStation(ptr)
;@ cdecl -arch=x86_64 PsGetProcessWow64Process()
@ stdcall PsGetThreadFreezeCount(ptr)
@ stdcall PsGetThreadHardErrorsAreDisabled(ptr)
@ stdcall PsGetThreadId(ptr)
@ stdcall PsGetThreadProcess(ptr)
@ stdcall PsGetThreadProcessId(ptr)
@ stdcall PsGetThreadSessionId(ptr)
@ stdcall PsGetThreadTeb(ptr)
@ stdcall PsGetThreadWin32Thread(ptr)
@ stdcall PsGetVersion(ptr ptr ptr ptr)
@ stdcall PsImpersonateClient(ptr ptr long long long)
@ extern PsInitialSystemProcess
@ stdcall PsIsProcessBeingDebugged(ptr)
@ stdcall PsIsSystemProcess(ptr)
@ stdcall PsIsSystemThread(ptr)
@ stdcall PsIsThreadImpersonating(ptr)
@ stdcall PsIsThreadTerminating(ptr)
@ extern PsJobType
@ stdcall PsLookupProcessByProcessId(ptr ptr)
@ stdcall PsLookupProcessThreadByCid(ptr ptr ptr)
@ stdcall PsLookupThreadByThreadId(ptr ptr)
@ extern PsProcessType
@ stdcall PsReferenceImpersonationToken(ptr ptr ptr ptr)
@ stdcall PsReferencePrimaryToken(ptr)
@ stdcall PsRemoveCreateThreadNotifyRoutine(ptr)
@ stdcall PsRemoveLoadImageNotifyRoutine(ptr)
@ stdcall PsRestoreImpersonation(ptr ptr)
@ stdcall PsReturnPoolQuota(ptr long long)
@ stdcall PsReturnProcessNonPagedPoolQuota(ptr long)
@ stdcall PsReturnProcessPagedPoolQuota(ptr long)
@ stdcall PsRevertThreadToSelf(ptr)
@ stdcall PsRevertToSelf()
@ stdcall PsSetContextThread(ptr ptr long)
@ stdcall PsSetCreateProcessNotifyRoutine(ptr long)
@ stdcall PsSetCreateThreadNotifyRoutine(ptr)
@ stdcall PsSetJobUIRestrictionsClass(ptr long)
@ stdcall PsSetLegoNotifyRoutine(ptr)
@ stdcall PsSetLoadImageNotifyRoutine(ptr)
@ stdcall PsSetProcessPriorityByClass(ptr ptr)
@ stdcall PsSetProcessPriorityClass(ptr long)
@ stdcall PsSetProcessSecurityPort(ptr ptr)
@ stdcall PsSetProcessWin32Process(ptr ptr ptr)
@ stdcall PsSetProcessWindowStation(ptr ptr)
@ stdcall PsSetThreadHardErrorsAreDisabled(ptr long)
@ stdcall PsSetThreadWin32Thread(ptr ptr ptr)
@ stdcall PsTerminateSystemThread(long)
@ extern PsThreadType
@ stdcall PsWrapApcWow64Thread(ptr ptr)
@ stdcall -arch=i386,arm,arm64 READ_REGISTER_BUFFER_UCHAR(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 READ_REGISTER_BUFFER_ULONG(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 READ_REGISTER_BUFFER_USHORT(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 READ_REGISTER_UCHAR(ptr)
@ stdcall -arch=i386,arm,arm64 READ_REGISTER_ULONG(ptr)
@ stdcall -arch=i386,arm,arm64 READ_REGISTER_USHORT(ptr)
@ stdcall RtlAbsoluteToSelfRelativeSD(ptr ptr ptr)
@ stdcall RtlAddAccessAllowedAce(ptr long long ptr)
@ stdcall RtlAddAccessAllowedAceEx(ptr long long long ptr)
@ stdcall RtlAddAce(ptr long long ptr long)
@ stdcall RtlAddAtomToAtomTable(ptr wstr ptr)
@ stdcall RtlAddRange(ptr long long long long long long ptr ptr)
@ stdcall RtlAllocateHeap(ptr long long)
@ stdcall RtlAnsiCharToUnicodeChar(ptr)
@ stdcall RtlAnsiStringToUnicodeSize(ptr) RtlxAnsiStringToUnicodeSize
@ stdcall RtlAnsiStringToUnicodeString(ptr ptr long)
@ stdcall RtlAppendAsciizToString(ptr str)
@ stdcall RtlAppendStringToString(ptr ptr)
@ stdcall RtlAppendUnicodeStringToString(ptr ptr)
@ stdcall RtlAppendUnicodeToString(ptr wstr)
@ stdcall RtlAreAllAccessesGranted(long long)
@ stdcall RtlAreAnyAccessesGranted(long long)
@ stdcall RtlAreBitsClear(ptr long long)
@ stdcall RtlAreBitsSet(ptr long long)
@ stdcall RtlAssert(str str long str)
@ stdcall RtlCaptureContext(ptr)
@ stdcall RtlCaptureStackBackTrace(long long ptr ptr)
@ stdcall RtlCharToInteger(str long ptr)
@ stdcall RtlCheckRegistryKey(long wstr)
@ stdcall RtlClearAllBits(ptr)
@ stdcall RtlClearBit(ptr long)
@ stdcall RtlClearBits(ptr long long)
@ stdcall RtlCmDecodeMemIoResource(ptr ptr)
@ stdcall RtlCmEncodeMemIoResource(ptr long int64 int64)
@ stdcall RtlCompareMemory(ptr ptr long)
@ stdcall RtlCompareMemoryUlong(ptr long long)
@ stdcall RtlCompareString(ptr ptr long)
@ stdcall RtlCompareUnicodeString(ptr ptr long)
@ stdcall RtlCompressBuffer(long ptr long ptr long long ptr ptr)
@ stdcall RtlCompressChunks(ptr long ptr long ptr long ptr)
@ stdcall RtlConvertLongToLargeInteger(long)
@ stdcall RtlConvertSidToUnicodeString(ptr ptr long)
@ stdcall RtlConvertUlongToLargeInteger(long)
@ stdcall RtlCopyLuid(ptr ptr)
@ stdcall -arch=x86_64 RtlCopyMemory(ptr ptr int64) memmove
@ stdcall -arch=x86_64 RtlCopyMemoryNonTemporal(ptr ptr int64) memmove
@ stdcall RtlCopyRangeList(ptr ptr)
@ stdcall RtlCopySid(long ptr ptr)
@ stdcall RtlCopyString(ptr ptr)
@ stdcall RtlCopyUnicodeString(ptr ptr)
@ stdcall RtlCreateAcl(ptr long long)
@ stdcall RtlCreateAtomTable(long ptr)
@ stdcall RtlCreateHeap(long ptr long long ptr ptr)
@ stdcall RtlCreateRegistryKey(long wstr)
@ stdcall RtlCreateSecurityDescriptor(ptr long)
@ stdcall RtlCreateSystemVolumeInformationFolder(ptr)
@ stdcall RtlCreateUnicodeString(ptr wstr)
@ stdcall RtlCustomCPToUnicodeN(ptr wstr long ptr ptr long)
@ stdcall RtlDecompressBuffer(long ptr long ptr long ptr)
@ stdcall RtlDecompressChunks(ptr long ptr long ptr long ptr)
@ stdcall RtlDecompressFragment(long ptr long ptr long long ptr ptr)
@ stdcall RtlDelete(ptr)
@ stdcall RtlDeleteAce(ptr long)
@ stdcall RtlDeleteAtomFromAtomTable(ptr ptr)
@ stdcall RtlDeleteElementGenericTable(ptr ptr)
@ stdcall RtlDeleteElementGenericTableAvl(ptr ptr)
@ stdcall RtlDeleteNoSplay(ptr ptr)
@ stdcall RtlDeleteOwnersRanges(ptr ptr)
@ stdcall RtlDeleteRange(ptr long long long long ptr)
@ stdcall RtlDeleteRegistryValue(long wstr wstr)
@ stdcall RtlDescribeChunk(long ptr ptr ptr ptr)
@ stdcall RtlDestroyAtomTable(ptr)
@ stdcall RtlDestroyHeap(ptr)
@ stdcall RtlDowncaseUnicodeString(ptr ptr long)
@ stdcall RtlEmptyAtomTable(ptr long)
@ stdcall -arch=win32 RtlEnlargedIntegerMultiply(long long)
@ stdcall -arch=win32 RtlEnlargedUnsignedDivide(long long long ptr)
@ stdcall -arch=win32 RtlEnlargedUnsignedMultiply(long long)
@ stdcall RtlEnumerateGenericTable(ptr long)
@ stdcall RtlEnumerateGenericTableAvl(ptr long)
@ stdcall RtlEnumerateGenericTableLikeADirectory(ptr ptr ptr long ptr ptr ptr)
@ stdcall RtlEnumerateGenericTableWithoutSplaying(ptr ptr)
@ stdcall RtlEnumerateGenericTableWithoutSplayingAvl(ptr ptr)
@ stdcall RtlEqualLuid(ptr ptr)
@ stdcall RtlEqualSid(ptr ptr)
@ stdcall RtlEqualString(ptr ptr long)
@ stdcall RtlEqualUnicodeString(ptr ptr long)
@ stdcall -arch=win32 RtlExtendedIntegerMultiply(long long long)
@ stdcall -arch=win32,arm64 RtlExtendedLargeIntegerDivide(long long long ptr)
@ stdcall -arch=win32 RtlExtendedMagicDivide(long long long long long)
@ stdcall RtlFillMemory(ptr long long)
@ stdcall -arch=i386,arm,arm64 RtlFillMemoryUlong(ptr long long)
@ stdcall RtlFindClearBits(ptr long long)
@ stdcall RtlFindClearBitsAndSet(ptr long long)
@ stdcall RtlFindClearRuns(ptr ptr long long)
@ stdcall RtlFindFirstRunClear(ptr ptr)
@ stdcall RtlFindLastBackwardRunClear(ptr long ptr)
@ stdcall RtlFindLeastSignificantBit(long long)
@ stdcall RtlFindLongestRunClear(ptr ptr)
@ stdcall RtlFindMessage(ptr long long long ptr)
@ stdcall RtlFindMostSignificantBit(long long)
@ stdcall RtlFindNextForwardRunClear(ptr long ptr)
@ stdcall RtlFindRange(ptr int64 int64 int64 int64 long long ptr ptr ptr)
@ stdcall RtlFindSetBits(ptr long long)
@ stdcall RtlFindSetBitsAndClear(ptr long long)
@ stdcall RtlFindUnicodePrefix(ptr ptr long)
@ stdcall RtlFormatCurrentUserKeyPath(ptr)
@ stdcall RtlFreeAnsiString(ptr)
@ stdcall RtlFreeHeap(ptr long ptr)
@ stdcall RtlFreeOemString(ptr)
@ stdcall RtlFreeRangeList(ptr)
@ stdcall RtlFreeUnicodeString(ptr)
@ stdcall RtlGUIDFromString(ptr ptr)
@ stdcall RtlGenerate8dot3Name(ptr ptr long ptr)
@ stdcall RtlGetAce(ptr long ptr)
@ stdcall RtlGetCallersAddress(ptr ptr)
@ stdcall RtlGetCompressionWorkSpaceSize(long ptr ptr)
@ stdcall RtlGetDaclSecurityDescriptor(ptr ptr ptr ptr)
@ stdcall RtlGetDefaultCodePage(ptr ptr)
@ stdcall RtlGetElementGenericTable(ptr long)
@ stdcall RtlGetElementGenericTableAvl(ptr long)
@ stdcall RtlGetFirstRange(ptr ptr ptr)
@ stdcall RtlGetLastRange(ptr ptr ptr)
@ stdcall RtlGetGroupSecurityDescriptor(ptr ptr ptr)
@ stdcall RtlGetNextRange(ptr ptr long)
@ stdcall RtlGetNtGlobalFlags()
@ stdcall RtlGetOwnerSecurityDescriptor(ptr ptr ptr)
@ stdcall RtlGetSaclSecurityDescriptor(ptr ptr ptr ptr)
@ stdcall RtlGetSetBootStatusData(ptr long long ptr long long)
@ stdcall RtlGetVersion(ptr)
@ stdcall RtlHashUnicodeString(ptr long long ptr)
@ stdcall RtlImageDirectoryEntryToData(ptr long long ptr)
@ stdcall RtlImageNtHeader(ptr)
@ stdcall RtlInitAnsiString(ptr str)
@ stdcall RtlInitAnsiStringEx(ptr str)
@ stdcall RtlInitCodePageTable(ptr ptr)
@ stdcall RtlInitString(ptr str)
@ stdcall RtlInitUnicodeString(ptr wstr)
@ stdcall RtlInitUnicodeStringEx(ptr wstr)
@ stdcall RtlInitializeBitMap(ptr ptr long)
@ stdcall RtlInitializeGenericTable(ptr ptr ptr ptr ptr)
@ stdcall RtlInitializeGenericTableAvl(ptr ptr ptr ptr ptr)
@ stdcall RtlInitializeRangeList(ptr)
@ stdcall RtlInitializeSid(ptr ptr long)
@ stdcall RtlInitializeUnicodePrefix(ptr)
@ stdcall RtlInsertElementGenericTable(ptr ptr long ptr)
@ stdcall RtlInsertElementGenericTableAvl(ptr ptr long ptr)
@ stdcall RtlInsertElementGenericTableFull(ptr ptr long ptr ptr long)
@ stdcall RtlInsertElementGenericTableFullAvl(ptr ptr long ptr ptr ptr)
@ stdcall RtlInsertUnicodePrefix(ptr ptr ptr)
@ stdcall RtlInt64ToUnicodeString(long long long ptr)
@ stdcall RtlIntegerToChar(long long long ptr)
@ stdcall RtlIntegerToUnicode(long long long ptr)
@ stdcall RtlIntegerToUnicodeString(long long ptr)
@ stdcall RtlInvertRangeList(ptr ptr)
@ stdcall RtlInvertRangeListEx(ptr ptr long ptr ptr)
@ stdcall RtlIoDecodeMemIoResource(ptr ptr ptr ptr)
@ stdcall RtlIoEncodeMemIoResource(ptr long int64 int64 int64 int64)
@ stdcall RtlIpv4AddressToStringA(ptr ptr)
@ stdcall RtlIpv4AddressToStringExA(ptr long ptr ptr)
@ stdcall RtlIpv4AddressToStringExW(ptr long ptr ptr)
@ stdcall RtlIpv4AddressToStringW(ptr ptr)
@ stdcall RtlIpv4StringToAddressA(str long ptr ptr)
@ stdcall RtlIpv4StringToAddressExA(str long ptr ptr)
@ stdcall RtlIpv4StringToAddressExW(wstr long ptr ptr)
@ stdcall RtlIpv4StringToAddressW(wstr long ptr ptr)
@ stdcall RtlIpv6AddressToStringA(ptr ptr)
@ stdcall RtlIpv6AddressToStringExA(ptr long long ptr ptr)
@ stdcall RtlIpv6AddressToStringExW(ptr long long ptr ptr)
@ stdcall RtlIpv6AddressToStringW(ptr ptr)
@ stdcall RtlIpv6StringToAddressA(str ptr ptr)
@ stdcall RtlIpv6StringToAddressExA(str ptr ptr ptr)
@ stdcall RtlIpv6StringToAddressExW(wstr ptr ptr ptr)
@ stdcall RtlIpv6StringToAddressW(wstr ptr ptr)
@ stdcall RtlIsGenericTableEmpty(ptr)
@ stdcall RtlIsGenericTableEmptyAvl(ptr)
@ stdcall RtlIsNameLegalDOS8Dot3(ptr ptr ptr)
@ stdcall RtlIsRangeAvailable(ptr long long long long long long ptr ptr ptr)
@ stdcall RtlIsValidOemCharacter(ptr)
@ stdcall -arch=win32 RtlLargeIntegerAdd(long long long long)
@ stdcall -arch=win32 RtlLargeIntegerArithmeticShift(long long long)
@ stdcall -arch=win32 RtlLargeIntegerDivide(long long long long ptr)
@ stdcall -arch=win32 RtlLargeIntegerNegate(long long)
@ stdcall -arch=win32 RtlLargeIntegerShiftLeft(long long long)
@ stdcall -arch=win32 RtlLargeIntegerShiftRight(long long long)
@ stdcall -arch=win32 RtlLargeIntegerSubtract(long long long long)
@ stdcall RtlLengthRequiredSid(long)
@ stdcall RtlLengthSecurityDescriptor(ptr)
@ stdcall RtlLengthSid(ptr)
@ stdcall RtlLockBootStatusData(ptr)
@ stdcall RtlLookupAtomInAtomTable(ptr wstr ptr)
@ stdcall RtlLookupElementGenericTable(ptr ptr)
@ stdcall RtlLookupElementGenericTableAvl(ptr ptr)
@ stdcall RtlLookupElementGenericTableFull(ptr ptr ptr ptr)
@ stdcall RtlLookupElementGenericTableFullAvl(ptr ptr ptr ptr)
@ cdecl -arch=x86_64,arm64 RtlLookupFunctionEntry(double ptr ptr)
@ stdcall RtlMapGenericMask(ptr ptr)
@ stdcall RtlMapSecurityErrorToNtStatus(long)
@ stdcall RtlMergeRangeLists(ptr ptr ptr long)
@ stdcall RtlMoveMemory(ptr ptr long)
@ stdcall RtlMultiByteToUnicodeN(ptr long ptr str long)
@ stdcall RtlMultiByteToUnicodeSize(ptr str long)
@ stdcall RtlNextUnicodePrefix(ptr long)
@ stdcall RtlNtStatusToDosError(long)
@ stdcall RtlNtStatusToDosErrorNoTeb(long)
@ stdcall RtlNumberGenericTableElements(ptr)
@ stdcall RtlNumberGenericTableElementsAvl(ptr)
@ stdcall RtlNumberOfClearBits(ptr)
@ stdcall RtlNumberOfSetBits(ptr)
@ stdcall RtlOemStringToCountedUnicodeString(ptr ptr long)
@ stdcall RtlOemStringToUnicodeSize(ptr) RtlxOemStringToUnicodeSize
@ stdcall RtlOemStringToUnicodeString(ptr ptr long)
@ stdcall RtlOemToUnicodeN(wstr long ptr ptr long)
@ cdecl -arch=x86_64 RtlPcToFileHeader(ptr ptr)
@ stdcall RtlPinAtomInAtomTable(ptr ptr)
@ fastcall RtlPrefetchMemoryNonTemporal(ptr long)
@ stdcall RtlPrefixString(ptr ptr long)
@ stdcall RtlPrefixUnicodeString(ptr ptr long)
@ stdcall RtlQueryAtomInAtomTable(ptr long ptr ptr ptr ptr)
@ stdcall RtlQueryRegistryValues(long wstr ptr ptr ptr)
@ stdcall RtlQueryTimeZoneInformation(ptr)
@ stdcall RtlRaiseException(ptr)
@ stdcall RtlRandom(ptr)
@ stdcall RtlRandomEx(ptr)
@ stdcall RtlRealPredecessor(ptr)
@ stdcall RtlRealSuccessor(ptr)
@ stdcall RtlRemoveUnicodePrefix(ptr ptr)
@ stdcall RtlReserveChunk(long ptr ptr ptr long)
@ cdecl -arch=x86_64,arm64 RtlRestoreContext(ptr ptr)
@ stdcall RtlSecondsSince1970ToTime(long ptr)
@ stdcall RtlSecondsSince1980ToTime(long ptr)
@ stdcall RtlSelfRelativeToAbsoluteSD(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall RtlSelfRelativeToAbsoluteSD2(ptr long)
@ stdcall RtlSetAllBits(ptr)
@ stdcall RtlSetBit(ptr long)
@ stdcall RtlSetBits(ptr long long)
@ stdcall RtlSetDaclSecurityDescriptor(ptr long ptr long)
@ stdcall RtlSetGroupSecurityDescriptor(ptr ptr long)
@ stdcall RtlSetOwnerSecurityDescriptor(ptr ptr long)
@ stdcall RtlSetSaclSecurityDescriptor(ptr long ptr long)
@ stdcall RtlSetTimeZoneInformation(ptr)
@ stdcall RtlSizeHeap(ptr long ptr)
@ stdcall RtlSplay(ptr)
@ stdcall RtlStringFromGUID(ptr ptr)
@ stdcall RtlSubAuthorityCountSid(ptr)
@ stdcall RtlSubAuthoritySid(ptr long)
@ stdcall RtlSubtreePredecessor(ptr)
@ stdcall RtlSubtreeSuccessor(ptr)
@ stdcall RtlTestBit(ptr long)
@ stdcall RtlTimeFieldsToTime(ptr ptr)
@ stdcall RtlTimeToElapsedTimeFields(ptr ptr)
@ stdcall RtlTimeToSecondsSince1970(ptr ptr)
@ stdcall RtlTimeToSecondsSince1980(ptr ptr)
@ stdcall RtlTimeToTimeFields(ptr ptr)
@ stdcall RtlTraceDatabaseAdd(ptr long ptr ptr)
@ stdcall RtlTraceDatabaseCreate(long ptr long long ptr)
@ stdcall RtlTraceDatabaseDestroy(ptr)
@ stdcall RtlTraceDatabaseEnumerate(ptr ptr ptr)
@ stdcall RtlTraceDatabaseFind(ptr long ptr ptr)
@ stdcall RtlTraceDatabaseLock(ptr)
@ stdcall RtlTraceDatabaseUnlock(ptr)
@ stdcall RtlTraceDatabaseValidate(ptr)
@ stdcall RtlUnregisterFeatureConfigurationChangeNotification(ptr)
@ stdcall -arch=win64 RtlUnregisterFeatureUsageProvider()
@ stdcall RtlUTF8ToUnicodeN(ptr long ptr str long)
@ fastcall -arch=i386,arm RtlUlongByteSwap(long)
@ fastcall -arch=i386,arm RtlUlonglongByteSwap(long long)
@ stdcall RtlUnicodeStringToAnsiSize(ptr) RtlxUnicodeStringToAnsiSize
@ stdcall RtlUnicodeStringToAnsiString(ptr ptr long)
@ stdcall RtlUnicodeStringToCountedOemString(ptr ptr long)
@ stdcall RtlUnicodeStringToInteger(ptr long ptr)
@ stdcall RtlUnicodeStringToOemSize(ptr) RtlxUnicodeStringToOemSize
@ stdcall RtlUnicodeStringToOemString(ptr ptr long)
@ stdcall RtlUnicodeToCustomCPN(ptr ptr long ptr wstr long)
@ stdcall RtlUnicodeToMultiByteN(ptr long ptr wstr long)
@ stdcall RtlUnicodeToMultiByteSize(ptr wstr long)
@ stdcall RtlUnicodeToOemN(ptr long ptr wstr long)
@ stdcall RtlUnicodeToUTF8N(ptr long ptr wstr long)
@ stdcall RtlUnlockBootStatusData(ptr)
@ stdcall RtlUnwind(ptr ptr ptr ptr)
@ stdcall -arch=x86_64,arm,arm64 RtlUnwindEx(ptr ptr ptr ptr ptr ptr)
@ stdcall RtlUpcaseUnicodeChar(long)
@ stdcall RtlUpcaseUnicodeString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeStringToAnsiString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeStringToCountedOemString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeStringToOemString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeToCustomCPN(ptr ptr long ptr wstr long)
@ stdcall RtlUpcaseUnicodeToMultiByteN(ptr long ptr wstr long)
@ stdcall RtlUpcaseUnicodeToOemN(ptr long ptr wstr long)
@ stdcall RtlUpperChar(long)
@ stdcall RtlUpperString(ptr ptr)
@ fastcall -arch=i386,arm RtlUshortByteSwap(long)
@ stdcall RtlValidRelativeSecurityDescriptor(ptr long long)
@ stdcall RtlValidSecurityDescriptor(ptr)
@ stdcall RtlValidSid(ptr)
@ stdcall RtlVerifyVersionInfo(ptr long long long)
@ stdcall -arch=x86_64,arm,arm64 RtlVirtualUnwind(long int64 int64 ptr ptr ptr ptr ptr)
@ stdcall RtlVolumeDeviceToDosName(ptr ptr) IoVolumeDeviceToDosName
@ stdcall RtlWalkFrameChain(ptr long long)
@ stdcall RtlWriteRegistryValue(long wstr wstr long ptr long)
@ stdcall RtlZeroHeap(ptr long)
@ stdcall RtlZeroMemory(ptr long)
@ stdcall RtlxAnsiStringToUnicodeSize(ptr)
@ stdcall RtlxOemStringToUnicodeSize(ptr)
@ stdcall RtlxUnicodeStringToAnsiSize(ptr)
@ stdcall RtlxUnicodeStringToOemSize(ptr)
@ stdcall SeAccessCheck(ptr ptr ptr long long ptr ptr long ptr ptr)
@ stdcall SeAppendPrivileges(ptr ptr)
@ stdcall SeAssignSecurity(ptr ptr ptr long ptr ptr ptr)
@ stdcall SeAssignSecurityEx(ptr ptr ptr ptr long long ptr ptr ptr)
@ stdcall SeAuditHardLinkCreation(ptr ptr long)
@ stdcall SeAuditingFileEvents(long ptr)
@ stdcall SeAuditingFileEventsWithContext(long ptr ptr)
@ stdcall SeAuditingFileOrGlobalEvents(long ptr ptr)
@ stdcall SeAuditingHardLinkEvents(long ptr)
@ stdcall SeAuditingHardLinkEventsWithContext(long ptr ptr)
@ stdcall SeCaptureSecurityDescriptor(ptr long long long ptr)
@ stdcall SeCaptureSubjectContext(ptr)
@ stdcall SeCloseObjectAuditAlarm(ptr ptr long)
@ stdcall SeCreateAccessState(ptr ptr long ptr)
@ stdcall SeCreateClientSecurity(ptr ptr long ptr)
@ stdcall SeCreateClientSecurityFromSubjectContext(ptr ptr long ptr)
@ stdcall SeDeassignSecurity(ptr)
@ stdcall SeDeleteAccessState(ptr)
@ stdcall SeDeleteObjectAuditAlarm(ptr ptr)
@ extern SeExports
@ stdcall SeFilterToken(ptr long ptr ptr ptr ptr)
@ stdcall SeFreePrivileges(ptr)
@ stdcall SeImpersonateClient(ptr ptr)
@ stdcall SeImpersonateClientEx(ptr ptr)
@ stdcall SeLockSubjectContext(ptr)
@ stdcall SeMarkLogonSessionForTerminationNotification(ptr)
@ stdcall SeOpenObjectAuditAlarm(ptr ptr ptr ptr ptr long long long ptr)
@ stdcall SeOpenObjectForDeleteAuditAlarm(ptr ptr ptr ptr ptr long long long ptr)
@ stdcall SePrivilegeCheck(ptr ptr long)
@ stdcall SePrivilegeObjectAuditAlarm(ptr ptr long ptr long long)
@ extern SePublicDefaultDacl
@ stdcall SeQueryAuthenticationIdToken(ptr ptr)
@ stdcall SeQueryInformationToken(ptr long ptr)
@ stdcall SeQuerySecurityDescriptorInfo(ptr ptr ptr ptr)
@ stdcall SeQuerySessionIdToken(ptr ptr)
@ stdcall SeRegisterLogonSessionTerminatedRoutine(ptr)
@ stdcall SeReleaseSecurityDescriptor(ptr long long)
@ stdcall SeReleaseSubjectContext(ptr)
@ stdcall SeReportSecurityEvent(long ptr ptr ptr)
@ stdcall SeSetAccessStateGenericMapping(ptr ptr)
@ stdcall SeSetAuditParameter(ptr long long ptr)
@ stdcall SeSetSecurityDescriptorInfo(ptr ptr ptr ptr long ptr)
@ stdcall SeSetSecurityDescriptorInfoEx(ptr ptr ptr ptr long long ptr)
@ stdcall SeSinglePrivilegeCheck(long long long)
@ extern SeSystemDefaultDacl
@ stdcall SeTokenImpersonationLevel(ptr)
@ stdcall SeTokenIsAdmin(ptr)
@ stdcall SeTokenIsRestricted(ptr)
@ extern SeTokenObjectType
@ stdcall SeTokenType(ptr)
@ stdcall SeUnlockSubjectContext(ptr)
@ stdcall SeUnregisterLogonSessionTerminatedRoutine(ptr)
@ stdcall SeValidSecurityDescriptor(long ptr)
@ stdcall VerSetConditionMask(long long long long)
@ stdcall VibeTripwireHasToken(ptr long)
@ cdecl VfFailDeviceNode(ptr long long long ptr ptr ptr)
@ cdecl VfFailDriver(long long long ptr ptr ptr)
@ cdecl VfFailSystemBIOS(long long long ptr ptr ptr)
@ stdcall VfIsVerificationEnabled(long ptr)
@ stdcall -arch=i386,arm,arm64 WRITE_REGISTER_BUFFER_UCHAR(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_REGISTER_BUFFER_ULONG(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_REGISTER_BUFFER_USHORT(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_REGISTER_UCHAR(ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_REGISTER_ULONG(ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_REGISTER_USHORT(ptr long)
@ stdcall WmiFlushTrace(ptr)
@ fastcall WmiGetClock(long ptr)
@ stdcall WmiQueryTrace(ptr)
@ stdcall WmiQueryTraceInformation(long ptr long ptr ptr)
@ stdcall WmiStartTrace(ptr)
@ stdcall WmiStopTrace(ptr)
@ fastcall WmiTraceFastEvent(ptr)
@ varargs WmiTraceMessage(int64 long ptr long)
@ stdcall WmiTraceMessageVa(int64 long ptr long ptr)
@ stdcall WmiUpdateTrace(ptr)
@ stdcall XIPDispatch(long ptr long)
@ stdcall ZwAccessCheckAndAuditAlarm(ptr ptr ptr ptr ptr long ptr long ptr ptr ptr)
@ stdcall ZwAddBootEntry(ptr long)
@ stdcall ZwAddDriverEntry(ptr long)
@ stdcall ZwAdjustPrivilegesToken(ptr long ptr long ptr ptr)
@ stdcall ZwAlertThread(ptr)
@ stdcall ZwAllocateVirtualMemory(ptr ptr long ptr long long)
@ stdcall ZwAssignProcessToJobObject(ptr ptr)
@ stdcall ZwCancelIoFile(ptr ptr)
@ stdcall ZwCancelTimer(ptr ptr)
@ stdcall ZwClearEvent(ptr)
@ stdcall ZwClose(ptr)
@ stdcall ZwCloseObjectAuditAlarm(ptr ptr long)
@ stdcall ZwConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall ZwCreateDirectoryObject(ptr long ptr)
@ stdcall ZwCreateEvent(ptr long ptr long long)
@ stdcall ZwCreateFile(ptr long ptr ptr ptr long long long long ptr long)
@ stdcall ZwCreateJobObject(ptr long ptr)
@ stdcall ZwCreateKey(ptr long ptr long ptr long ptr)
@ stdcall ZwCreateSection(ptr long ptr ptr long long ptr)
@ stdcall ZwCreateSymbolicLinkObject(ptr long ptr ptr)
@ stdcall ZwCreateTimer(ptr long ptr long)
@ stdcall ZwDeleteBootEntry(long)
@ stdcall ZwDeleteDriverEntry(long)
@ stdcall ZwDeleteFile(ptr)
@ stdcall ZwDeleteKey(ptr)
@ stdcall ZwDeleteValueKey(ptr ptr)
@ stdcall ZwDeviceIoControlFile(ptr ptr ptr ptr ptr long ptr long ptr long)
@ stdcall ZwDisplayString(ptr)
@ stdcall ZwDuplicateObject(ptr ptr ptr ptr long long long)
@ stdcall ZwDuplicateToken(ptr long ptr long long ptr)
@ stdcall ZwEnumerateBootEntries(ptr ptr)
@ stdcall ZwEnumerateDriverEntries(ptr ptr)
@ stdcall ZwEnumerateKey(ptr long long ptr long ptr)
@ stdcall ZwEnumerateValueKey(ptr long long ptr long ptr)
@ stdcall ZwFlushInstructionCache(ptr ptr long)
@ stdcall ZwFlushKey(ptr)
@ stdcall ZwFlushVirtualMemory(ptr ptr ptr ptr)
@ stdcall ZwFreeVirtualMemory(ptr ptr ptr long)
@ stdcall ZwFsControlFile(ptr ptr ptr ptr ptr long ptr long ptr long)
@ stdcall ZwInitiatePowerAction(long long long long)
@ stdcall ZwIsProcessInJob(ptr ptr)
@ stdcall ZwLoadDriver(ptr)
@ stdcall ZwLoadKey(ptr ptr)
@ stdcall ZwMakeTemporaryObject(ptr)
@ stdcall ZwMapViewOfSection(ptr ptr ptr long long ptr ptr long long long)
@ stdcall ZwModifyBootEntry(ptr)
@ stdcall ZwModifyDriverEntry(ptr)
@ stdcall ZwNotifyChangeKey(ptr ptr ptr ptr ptr long long ptr long long)
@ stdcall ZwOpenDirectoryObject(ptr long ptr)
@ stdcall ZwOpenEvent(ptr long ptr)
@ stdcall ZwOpenFile(ptr long ptr ptr long long)
@ stdcall ZwOpenJobObject(ptr long ptr)
@ stdcall ZwOpenKey(ptr long ptr)
@ stdcall ZwOpenProcess(ptr long ptr ptr)
@ stdcall ZwOpenProcessToken(ptr long ptr)
@ stdcall ZwOpenProcessTokenEx(ptr long long ptr)
@ stdcall ZwOpenSection(ptr long ptr)
@ stdcall ZwOpenSymbolicLinkObject(ptr long ptr)
@ stdcall ZwOpenThread(ptr long ptr ptr)
@ stdcall ZwOpenThreadToken(ptr long long ptr)
@ stdcall ZwOpenThreadTokenEx(ptr long long long ptr)
@ stdcall ZwOpenTimer(ptr long ptr)
@ stdcall ZwPowerInformation(long ptr long ptr long)
@ stdcall ZwPulseEvent(ptr ptr)
@ stdcall ZwQueryBootEntryOrder(ptr ptr)
@ stdcall ZwQueryBootOptions(ptr ptr)
@ stdcall ZwQueryDefaultLocale(long ptr)
@ stdcall ZwQueryDefaultUILanguage(ptr)
@ stdcall ZwQueryDirectoryFile(ptr ptr ptr ptr ptr ptr long long long ptr long)
@ stdcall ZwQueryDirectoryObject(ptr ptr long long long ptr ptr)
@ stdcall ZwQueryDriverEntryOrder(ptr ptr)
@ stdcall ZwQueryEaFile(ptr ptr ptr long long ptr long ptr long)
@ stdcall ZwQueryFullAttributesFile(ptr ptr)
@ stdcall ZwQueryInformationFile(ptr ptr ptr long long)
@ stdcall ZwQueryInformationJobObject(ptr long ptr long ptr)
@ stdcall ZwQueryInformationProcess(ptr long ptr long ptr)
@ stdcall ZwQueryInformationThread(ptr long ptr long ptr)
@ stdcall ZwQueryInformationToken(ptr long long long ptr)
@ stdcall ZwQueryInstallUILanguage(ptr)
@ stdcall ZwQueryKey(ptr long ptr long ptr)
@ stdcall ZwQueryObject(ptr long ptr long ptr)
@ stdcall ZwQuerySection(ptr long ptr long ptr)
@ stdcall ZwQuerySecurityObject(ptr long ptr long ptr)
@ stdcall ZwQuerySymbolicLinkObject(ptr ptr ptr)
@ stdcall ZwQuerySystemInformation(long ptr long ptr)
@ stdcall ZwQueryValueKey(ptr ptr long ptr long ptr)
@ stdcall ZwQueryVolumeInformationFile(ptr ptr ptr long long)
@ stdcall ZwReadFile(ptr ptr ptr ptr ptr ptr long ptr ptr)
@ stdcall ZwReplaceKey(ptr ptr ptr)
@ stdcall ZwRequestWaitReplyPort(ptr ptr ptr)
@ stdcall ZwResetEvent(ptr ptr)
@ stdcall ZwRestoreKey(ptr ptr long)
@ stdcall ZwSaveKey(ptr ptr)
@ stdcall ZwSaveKeyEx(ptr ptr long)
@ stdcall ZwSecureConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall ZwSetBootEntryOrder(ptr ptr)
@ stdcall ZwSetBootOptions(ptr long)
@ stdcall ZwSetDefaultLocale(long long)
@ stdcall ZwSetDefaultUILanguage(long)
@ stdcall ZwSetDriverEntryOrder(ptr ptr)
@ stdcall ZwSetEaFile(ptr ptr ptr long)
@ stdcall ZwSetEvent(ptr ptr)
@ stdcall ZwSetInformationFile(ptr ptr ptr long long)
@ stdcall ZwSetInformationJobObject(ptr long ptr long)
@ stdcall ZwSetInformationObject(ptr long ptr long)
@ stdcall ZwSetInformationProcess(ptr long ptr long)
@ stdcall ZwSetInformationThread(ptr long ptr long)
@ stdcall ZwSetSecurityObject(ptr long ptr)
@ stdcall ZwSetSystemInformation(long ptr long)
@ stdcall ZwSetSystemTime(ptr ptr)
@ stdcall ZwSetTimer(ptr ptr ptr ptr long long ptr)
@ stdcall ZwSetValueKey(ptr ptr long long ptr long)
@ stdcall ZwSetVolumeInformationFile(ptr ptr ptr long long)
@ stdcall ZwTerminateJobObject(ptr long)
@ stdcall ZwTerminateProcess(ptr long)
@ stdcall ZwTranslateFilePath(ptr long ptr long)
@ stdcall ZwUnloadDriver(ptr)
@ stdcall ZwUnloadKey(ptr)
@ stdcall ZwUnmapViewOfSection(ptr ptr)
@ stdcall ZwWaitForMultipleObjects(long ptr long long ptr)
@ stdcall ZwWaitForSingleObject(ptr long ptr)
@ stdcall ZwWriteFile(ptr ptr ptr ptr ptr ptr long ptr ptr)
@ stdcall ZwYieldExecution()
@ cdecl -arch=x86_64,arm,arm64 __C_specific_handler(ptr long ptr ptr)
@ cdecl -arch=arm __jump_unwind()
@ cdecl -arch=x86_64 __chkstk()
;@ cdecl -arch=x86_64 __misaligned_access()
@ cdecl -arch=i386 _CIcos()
@ cdecl -arch=i386 _CIsin()
@ cdecl -arch=i386 _CIsqrt()
@ cdecl -arch=i386,arm _abnormal_termination()
@ cdecl -arch=i386 _alldiv()
@ cdecl -arch=i386 _alldvrm()
@ cdecl -arch=i386 _allmul()
@ cdecl -arch=i386 _alloca_probe()
@ cdecl -arch=i386 _allrem()
@ cdecl -arch=i386 _allshl()
@ cdecl -arch=i386 _allshr()
@ cdecl -arch=i386 _aulldiv()
@ cdecl -arch=i386 _aulldvrm()
@ cdecl -arch=i386 _aullrem()
@ cdecl -arch=i386 _aullshr()
@ cdecl -arch=i386,arm _except_handler2()
@ cdecl -arch=i386,arm _except_handler3()
@ cdecl -arch=i386,arm _global_unwind2()
@ cdecl _itoa()
@ cdecl _itow()
@ cdecl -arch=i386,arm _local_unwind2()
@ cdecl -arch=x86_64 _local_unwind()
@ cdecl _purecall()
@ cdecl -arch=x86_64,arm _setjmp(ptr ptr)
@ cdecl -arch=x86_64,arm _setjmpex(ptr ptr)
@ cdecl _snprintf()
@ cdecl _snwprintf()
@ cdecl _stricmp()
@ cdecl _strlwr()
@ cdecl _strnicmp()
@ cdecl _strnset()
@ cdecl _strrev()
@ cdecl _strset()
@ cdecl _strupr()
@ cdecl -version=0x400-0x502 -impsym _swprintf() swprintf # Compatibility with pre NT6
@ cdecl _vsnprintf()
@ cdecl _vsnwprintf()
@ cdecl _wcsicmp()
@ cdecl _wcslwr()
@ cdecl _wcsnicmp()
@ cdecl _wcsnset()
@ cdecl _wcsrev()
@ cdecl _wcsupr()
@ cdecl atoi()
@ cdecl atol()
@ cdecl isdigit()
@ cdecl islower()
@ cdecl isprint()
@ cdecl isspace()
@ cdecl isupper()
@ cdecl isxdigit()
@ cdecl -arch=x86_64,arm longjmp(ptr long)
@ cdecl mbstowcs()
@ cdecl mbtowc()
@ cdecl memchr()
@ cdecl -arch=x86_64 memcmp()
@ cdecl memcpy()
@ cdecl memmove()
@ cdecl memset()
@ cdecl qsort()
@ cdecl rand()
@ varargs sprintf(ptr str)
@ cdecl srand()
@ cdecl strcat()
@ cdecl strchr()
@ cdecl strcmp()
@ cdecl strcpy()
@ cdecl strlen()
@ cdecl strncat()
@ cdecl strncmp()
@ cdecl strncpy()
@ cdecl strrchr()
@ cdecl strspn()
@ cdecl strstr()
@ cdecl swprintf()
@ cdecl tolower()
@ cdecl toupper() toupper_nt_mb
@ cdecl towlower()
@ cdecl towupper()
@ stdcall vDbgPrintEx(long long str ptr)
@ stdcall vDbgPrintExWithPrefix(str long long str ptr)
@ cdecl vsprintf(ptr str ptr)
@ cdecl wcscat()
@ cdecl wcschr()
@ cdecl wcscmp()
@ cdecl wcscpy()
@ cdecl wcscspn()
@ cdecl wcslen()
@ cdecl wcsncat()
@ cdecl wcsncmp()
@ cdecl wcsncpy()
@ cdecl wcsrchr()
@ cdecl wcsspn()
@ cdecl wcsstr()
@ cdecl wcstombs()
@ cdecl wctomb()

# FIXME: check if this is correct
@ stdcall -arch=arm __dtoi64()
@ stdcall -arch=arm __dtou64()
@ stdcall -arch=arm __i64tod()
@ stdcall -arch=arm __u64tod()
@ stdcall -arch=arm __rt_sdiv()
@ stdcall -arch=arm __rt_sdiv64()
@ stdcall -arch=arm __rt_udiv()
@ stdcall -arch=arm __rt_udiv64()
@ stdcall -arch=arm __rt_srsh()

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ AlpcCreateSecurityContext
@ stub -version=0xA00+ AlpcGetHeaderSize
@ stub -version=0xA00+ AlpcGetMessageAttribute
@ stub -version=0xA00+ AlpcInitializeMessageAttribute
@ stub -version=0xA00+ BgkDisplayCharacter
@ stub -version=0xA00+ BgkGetConsoleState
@ stub -version=0xA00+ BgkGetCursorState
@ stub -version=0xA00+ BgkSetCursor
@ stub -version=0xA00+ CarCopyRuleViolationDetails
@ stub -version=0xA00+ CarCreateRuleViolationDetails
@ stub -version=0xA00+ CarDeleteRuleViolationDetails
@ stub -version=0xA00+ CarDeregisterRuleClassConfiguration
@ stub -version=0xA00+ CarDeregisterRuleOverride
@ stub -version=0xA00+ CarInitializeRuleViolationDetails
@ stub -version=0xA00+ CarQueryReportAction
@ stub -version=0xA00+ CarQueryReportActionForTriage
@ stub -version=0xA00+ CarRegisterDefaultRuleClassConfiguration
@ stub -version=0xA00+ CarRegisterRuleClassConfiguration
@ stub -version=0xA00+ CarRegisterRuleOverride
@ stub -version=0xA00+ CarRegisterRuleOverrideAllContexts
@ stub -version=0xA00+ CarRegisterRuleOverridesAllContexts
@ stub -version=0xA00+ CarReportDifPluginRuleViolation
@ stub -version=0xA00+ CarSetCustomIdInRuleOverride
@ stub -version=0xA00+ CarSetCustomRuleIdRange
@ stub -version=0xA00+ CcAddDirtyPagesToExternalCache
@ stub -version=0xA00+ CcAsyncCopyRead
@ stub -version=0xA00+ CcCoherencyFlushAndPurgeCache
@ stub -version=0xA00+ CcCopyReadEx
@ stub -version=0xA00+ CcCopyWriteEx
@ stub -version=0xA00+ CcCopyWriteWontFlush
@ stub -version=0xA00+ CcDeductDirtyPagesFromExternalCache
@ stub -version=0xA00+ CcErrorCallbackRoutine
@ stub -version=0xA00+ CcFlushCacheToLsn
@ stub -version=0xA00+ CcGetCachedDirtyPageCountForFile
@ stub -version=0xA00+ CcGetFileObjectFromSectionPtrsRef
@ stub -version=0xA00+ CcGetNumberOfMappedPages
@ stub -version=0xA00+ CcInitializeCacheMapEx
@ stub -version=0xA00+ CcInitializeCacheMapEx2
@ stub -version=0xA00+ CcIsCacheManagerCallbackNeeded
@ stub -version=0xA00+ CcIsThereDirtyDataEx
@ stub -version=0xA00+ CcIsThereDirtyLoggedPages
@ stub -version=0xA00+ CcRegisterExternalCache
@ stub -version=0xA00+ CcScheduleReadAheadEx
@ stub -version=0xA00+ CcSetAdditionalCacheAttributesEx
@ stub -version=0xA00+ CcSetFileSizesEx
@ stub -version=0xA00+ CcSetLogHandleForFileEx
@ stub -version=0xA00+ CcSetLoggedDataThreshold
@ stub -version=0xA00+ CcSetParallelFlushFile
@ stub -version=0xA00+ CcSetReadAheadGranularityEx
@ stub -version=0xA00+ CcTestControl
@ stub -version=0xA00+ CcUnmapFileOffsetFromSystemCache
@ stub -version=0xA00+ CcUnregisterExternalCache
@ stub -version=0xA00+ CcZeroDataOnDisk
@ stub -version=0xA00+ CmCallbackGetKeyObjectID
@ stub -version=0xA00+ CmCallbackGetKeyObjectIDEx
@ stub -version=0xA00+ CmCallbackReleaseKeyObjectIDEx
@ stub -version=0xA00+ CmGetBoundTransaction
@ stub -version=0xA00+ CmGetCallbackVersion
@ stub -version=0xA00+ CmRegisterCallbackEx
@ stub -version=0xA00+ CmRegisterMachineHiveLoadedNotification
@ stub -version=0xA00+ CmSetCallbackObjectContext
@ stub -version=0xA00+ CmUnregisterMachineHiveLoadedNotification
@ stub -version=0xA00+ CsanRead16NoCheck
@ stub -version=0xA00+ CsanRead64NoCheck
@ stub -version=0xA00+ CsanRead8NoCheck
@ stub -version=0xA00+ CsanReadNoCheck
@ stub -version=0xA00+ CsanWrite16NoCheck
@ stub -version=0xA00+ CsanWrite64NoCheck
@ stub -version=0xA00+ CsanWrite8NoCheck
@ stub -version=0xA00+ CsanWriteNoCheck
@ stub -version=0xA00+ DbgSetDebugPrintCallback
@ stub -version=0xA00+ DbgkLkmdRegisterCallback
@ stub -version=0xA00+ DbgkLkmdUnregisterCallback
@ stub -version=0xA00+ DbgkWerCaptureLiveKernelDump
@ stub -version=0xA00+ DbgkWerCaptureLiveKernelDump2
@ stub -version=0xA00+ DifEnumeratePluginData
@ stub -version=0xA00+ DifFindThreadContextData
@ stub -version=0xA00+ DifGetPluginPerDriverData
@ stub -version=0xA00+ DifObjTrkInsertItem
@ stub -version=0xA00+ DifObjTrkQeuryInvokeDeleteRange
@ stub -version=0xA00+ DifObjTrkRemoveItem
@ stub -version=0xA00+ DifPluginSimplePerfControl
@ stub -version=0xA00+ DifPopThreadContextData
@ stub -version=0xA00+ DifPushThreadContextData
@ stub -version=0xA00+ DifRegisterClassDriverPlugin
@ stub -version=0xA00+ DifRegisterObjectTracking
@ stub -version=0xA00+ DifRegisterPlugin
@ stub -version=0xA00+ DifUtilDbgPrint
@ stub -version=0xA00+ EmClientQueryRuleState
@ stub -version=0xA00+ EmClientRuleDeregisterNotification
@ stub -version=0xA00+ EmClientRuleEvaluate
@ stub -version=0xA00+ EmClientRuleRegisterNotification
@ stub -version=0xA00+ EmProviderDeregister
@ stub -version=0xA00+ EmProviderDeregisterEntry
@ stub -version=0xA00+ EmProviderRegister
@ stub -version=0xA00+ EmProviderRegisterEntry
@ stub -version=0xA00+ EmpProviderRegister
@ stub -version=0xA00+ EtwEnableTrace
@ stub -version=0xA00+ EtwRegisterClassicProvider
@ stub -version=0xA00+ EtwSendTraceBuffer
@ stub -version=0xA00+ EtwTelemetryCoverageReport
@ stub -version=0xA00+ EtwWriteEndScenario
@ stub -version=0xA00+ EtwWriteStartScenario
@ stub -version=0xA00+ EtwpDisableStackWalkApc
@ stub -version=0xA00+ EtwpReenableStackWalkApc
@ stub -version=0xA00+ ExAccessByte
@ stub -version=0xA00+ ExAcquireAutoExpandPushLockExclusive
@ stub -version=0xA00+ ExAcquireAutoExpandPushLockShared
@ stub -version=0xA00+ ExAcquireCacheAwarePushLockExclusive
@ stub -version=0xA00+ ExAcquireCacheAwarePushLockExclusiveEx
@ stub -version=0xA00+ ExAcquireCacheAwarePushLockSharedEx
@ stub -version=0xA00+ ExAcquireFastResourceExclusive
@ stub -version=0xA00+ ExAcquireFastResourceShared
@ stub -version=0xA00+ ExAcquireFastResourceSharedStarveExclusive
@ stub -version=0xA00+ ExAcquireFastResourceWithFlags
@ stub -version=0xA00+ ExAcquirePushLockExclusiveEx
@ stub -version=0xA00+ ExAcquirePushLockSharedEx
@ stub -version=0xA00+ ExAcquireSpinLockExclusive
@ stub -version=0xA00+ ExAcquireSpinLockExclusiveAtDpcLevel
@ stub -version=0xA00+ ExAcquireSpinLockShared
@ stub -version=0xA00+ ExAcquireSpinLockSharedAtDpcLevel
@ stub -version=0xA00+ ExAllocateAutoExpandPushLock
@ stub -version=0xA00+ ExAllocateCacheAwarePushLock
@ stub -version=0xA00+ ExAllocateFromLookasideListEx
@ stub -version=0xA00+ ExAllocateFromNPagedLookasideList
@ stub -version=0xA00+ ExAllocateTimer
@ stub -version=0xA00+ ExBlockOnAddressPushLock
@ stub -version=0xA00+ ExBlockPushLock
@ stub -version=0xA00+ ExCancelDpcEventWait
@ stub -version=0xA00+ ExCancelTimer
@ stub -version=0xA00+ ExCleanupAutoExpandPushLock
@ stub -version=0xA00+ ExCleanupRundownProtectionCacheAware
@ stub -version=0xA00+ ExConvertFastResourceExclusiveToShared
@ stub -version=0xA00+ ExConvertPushLockExclusiveToShared
@ stub -version=0xA00+ ExCreateDpcEvent
@ stub -version=0xA00+ ExCreatePool
@ stub -version=0xA00+ ExDeleteDpcEvent
@ stub -version=0xA00+ ExDeleteFastResource
@ stub -version=0xA00+ ExDeleteLookasideListEx
@ stub -version=0xA00+ ExDeleteTimer
@ stub -version=0xA00+ ExDestroyPool
@ stub -version=0xA00+ ExDisownFastResource
@ stub -version=0xA00+ ExEnterPriorityRegionAndAcquireResourceExclusive
@ stub -version=0xA00+ ExEnterPriorityRegionAndAcquireResourceShared
@ stub -version=0xA00+ ExEnumerateSystemFirmwareTables
@ stub -version=0xA00+ ExFetchLicenseData
@ stub -version=0xA00+ ExFlushLookasideListEx
@ stub -version=0xA00+ ExFreeAutoExpandPushLock
@ stub -version=0xA00+ ExFreeCacheAwarePushLock
@ stub -version=0xA00+ ExFreeToLookasideListEx
@ stub -version=0xA00+ ExFreeToNPagedLookasideList
@ stub -version=0xA00+ ExGetFirmwareEnvironmentVariable
@ stub -version=0xA00+ ExGetFirmwareType
@ stub -version=0xA00+ ExGetLicenseTamperState
@ stub -version=0xA00+ ExGetPrmInterface
@ stub -version=0xA00+ ExGetSystemFirmwareTable
@ stub -version=0xA00+ ExInitializeAutoExpandPushLock
@ stub -version=0xA00+ ExInitializeDeviceAts
@ stub -version=0xA00+ ExInitializeFastOwnerEntry
@ stub -version=0xA00+ ExInitializeFastResource
@ stub -version=0xA00+ ExInitializeFastResource2
@ stub -version=0xA00+ ExInitializeFastResourceAcquired
@ stub -version=0xA00+ ExInitializeLookasideListEx
@ stub -version=0xA00+ ExInitializePushLock
@ stub -version=0xA00+ ExInitializeResourceLite2
@ stub -version=0xA00+ ExInitializeRundownProtectionCacheAwareEx
@ stub -version=0xA00+ ExIsFastResourceContended
@ stub -version=0xA00+ ExIsFastResourceHeld
@ stub -version=0xA00+ ExIsFastResourceHeldExclusive
@ stub -version=0xA00+ ExIsManufacturingModeEnabled
@ stub -version=0xA00+ ExIsSoftBoot
@ stub -version=0xA00+ ExMoveFastResourceOwnershipWithFlags
@ stub -version=0xA00+ ExNotifyBootDeviceRemoval
@ stub -version=0xA00+ ExQueryFastCacheDevLicense
@ stub -version=0xA00+ ExQueryTimerResolution
@ stub -version=0xA00+ ExQueryWnfStateData
@ stub -version=0xA00+ ExQueueDpcEventWait
@ stub -version=0xA00+ ExRcuFreePool
@ stub -version=0xA00+ ExRealTimeIsUniversal
@ stub -version=0xA00+ ExRegisterBootDevice
@ stub -version=0xA00+ ExRegisterExtension
@ stub -version=0xA00+ ExReinitializeFastResource
@ stub -version=0xA00+ ExReleaseAutoExpandPushLockExclusive
@ stub -version=0xA00+ ExReleaseAutoExpandPushLockShared
@ stub -version=0xA00+ ExReleaseCacheAwarePushLockExclusive
@ stub -version=0xA00+ ExReleaseCacheAwarePushLockExclusiveEx
@ stub -version=0xA00+ ExReleaseCacheAwarePushLockSharedEx
@ stub -version=0xA00+ ExReleaseDisownedFastResource
@ stub -version=0xA00+ ExReleaseDisownedFastResourceExclusive
@ stub -version=0xA00+ ExReleaseDisownedFastResourceShared
@ stub -version=0xA00+ ExReleaseFastResource
@ stub -version=0xA00+ ExReleaseFastResourceExclusive
@ stub -version=0xA00+ ExReleaseFastResourceShared
@ stub -version=0xA00+ ExReleasePushLockEx
@ stub -version=0xA00+ ExReleasePushLockExclusiveEx
@ stub -version=0xA00+ ExReleasePushLockSharedEx
@ stub -version=0xA00+ ExReleaseResourceAndLeavePriorityRegion
@ stub -version=0xA00+ ExReleaseSpinLockExclusive
@ stub -version=0xA00+ ExReleaseSpinLockExclusiveFromDpcLevel
@ stub -version=0xA00+ ExReleaseSpinLockShared
@ stub -version=0xA00+ ExReleaseSpinLockSharedFromDpcLevel
@ stub -version=0xA00+ ExSecurePoolUpdate
@ stub -version=0xA00+ ExSecurePoolValidate
@ stub -version=0xA00+ ExSetFirmwareEnvironmentVariable
@ stub -version=0xA00+ ExSetLicenseTamperState
@ stub -version=0xA00+ ExSetResourceOwnerPointerEx
@ stub -version=0xA00+ ExSetTimer
@ stub -version=0xA00+ ExShareAddressSpaceWithDevice
@ stub -version=0xA00+ ExShareSystemAddressSpaceWithDevice
@ stub -version=0xA00+ ExSizeOfAutoExpandPushLock
@ stub -version=0xA00+ ExStopSharingAddressSpaceWithDevice
@ stub -version=0xA00+ ExStopSharingSystemAddressSpaceWithDevice
@ stub -version=0xA00+ ExSubscribeWnfStateChange
@ stub -version=0xA00+ ExSvmBeginDeviceReset
@ stub -version=0xA00+ ExSvmFinalizeDeviceReset
@ stub -version=0xA00+ ExTimedWaitForUnblockPushLock
@ stub -version=0xA00+ ExTryAcquireAutoExpandPushLockExclusive
@ stub -version=0xA00+ ExTryAcquireAutoExpandPushLockShared
@ stub -version=0xA00+ ExTryAcquireCacheAwarePushLockExclusiveEx
@ stub -version=0xA00+ ExTryAcquireCacheAwarePushLockSharedEx
@ stub -version=0xA00+ ExTryAcquirePushLockExclusiveEx
@ stub -version=0xA00+ ExTryAcquirePushLockSharedEx
@ stub -version=0xA00+ ExTryAcquireSpinLockExclusiveAtDpcLevel
@ stub -version=0xA00+ ExTryAcquireSpinLockSharedAtDpcLevel
@ stub -version=0xA00+ ExTryConvertPushLockSharedToExclusiveEx
@ stub -version=0xA00+ ExTryConvertSharedSpinLockExclusive
@ stub -version=0xA00+ ExTryQueueWorkItem
@ stub -version=0xA00+ ExTryToAcquireResourceExclusiveLite
@ stub -version=0xA00+ ExTryToConvertFastResourceSharedToExclusive
@ stub -version=0xA00+ ExUnblockOnAddressPushLockEx
@ stub -version=0xA00+ ExUnblockPushLockEx
@ stub -version=0xA00+ ExUnregisterExtension
@ stub -version=0xA00+ ExUnsubscribeWnfStateChange
@ stub -version=0xA00+ ExUpdateLicenseData
@ stub -version=0xA00+ ExWaitForUnblockPushLock
@ stub -version=0xA00+ ExfTryAcquirePushLockShared
@ stub -version=0xA00+ FirstEntrySList
@ stub -version=0xA00+ FsRtlAcknowledgeEcp
@ stub -version=0xA00+ FsRtlAcquireEofLock
@ stub -version=0xA00+ FsRtlAcquireHeaderMutex
@ stub -version=0xA00+ FsRtlAddBaseMcbEntryEx
@ stub -version=0xA00+ FsRtlAddToTunnelCacheEx
@ stub -version=0xA00+ FsRtlAllocateAePushLock
@ stub -version=0xA00+ FsRtlAllocateExtraCreateParameter
@ stub -version=0xA00+ FsRtlAllocateExtraCreateParameterFromLookasideList
@ stub -version=0xA00+ FsRtlAllocateExtraCreateParameterList
@ stub -version=0xA00+ FsRtlAreThereCurrentOrInProgressFileLocks
@ stub -version=0xA00+ FsRtlAreThereWaitingFileLocks
@ stub -version=0xA00+ FsRtlAreVolumeStartupApplicationsComplete
@ stub -version=0xA00+ FsRtlCancellableWaitForMultipleObjects
@ stub -version=0xA00+ FsRtlCancellableWaitForSingleObject
@ stub -version=0xA00+ FsRtlChangeBackingFileObject
@ stub -version=0xA00+ FsRtlCheckFileSystemFilterCallbacksRegistered
@ stub -version=0xA00+ FsRtlCheckLockForOplockRequest
@ stub -version=0xA00+ FsRtlCheckOplockEx
@ stub -version=0xA00+ FsRtlCheckOplockEx2
@ stub -version=0xA00+ FsRtlCheckOplockForFsFilterCallback
@ stub -version=0xA00+ FsRtlCheckUpperOplock
@ stub -version=0xA00+ FsRtlCurrentOplock
@ stub -version=0xA00+ FsRtlCurrentOplockH
@ stub -version=0xA00+ FsRtlDedupChangeInit
@ stub -version=0xA00+ FsRtlDedupChangeLogOverwriteOrFree
@ stub -version=0xA00+ FsRtlDedupChangeLogWrite
@ stub -version=0xA00+ FsRtlDedupChangeUninit
@ stub -version=0xA00+ FsRtlDeleteExtraCreateParameterLookasideList
@ stub -version=0xA00+ FsRtlDisallowLegacyFilterOnDevice
@ stub -version=0xA00+ FsRtlDismountComplete
@ stub -version=0xA00+ FsRtlFindExtraCreateParameter
@ stub -version=0xA00+ FsRtlFindInTunnelCacheEx
@ stub -version=0xA00+ FsRtlFreeAePushLock
@ stub -version=0xA00+ FsRtlFreeExtraCreateParameter
@ stub -version=0xA00+ FsRtlFreeExtraCreateParameterList
@ stub -version=0xA00+ FsRtlGetCurrentProcessLoaderList
@ stub -version=0xA00+ FsRtlGetDirectImageOriginalBase
@ stub -version=0xA00+ FsRtlGetEcpListFromIrp
@ stub -version=0xA00+ FsRtlGetFileExtents
@ stub -version=0xA00+ FsRtlGetFileNameInformation
@ stub -version=0xA00+ FsRtlGetIoAtEof
@ stub -version=0xA00+ FsRtlGetNextExtraCreateParameter
@ stub -version=0xA00+ FsRtlGetSectorSizeInformation
@ stub -version=0xA00+ FsRtlGetSupportedFeatures
@ stub -version=0xA00+ FsRtlGetVirtualDiskNestingLevel
@ stub -version=0xA00+ FsRtlHeatInit
@ stub -version=0xA00+ FsRtlHeatLogIo
@ stub -version=0xA00+ FsRtlHeatLogTierMove
@ stub -version=0xA00+ FsRtlHeatUninit
@ stub -version=0xA00+ FsRtlIncrementCcFastMdlReadWait
@ stub -version=0xA00+ FsRtlInitExtraCreateParameterLookasideList
@ stub -version=0xA00+ FsRtlInitializeBaseMcbEx
@ stub -version=0xA00+ FsRtlInitializeEofLock
@ stub -version=0xA00+ FsRtlInitializeExtraCreateParameter
@ stub -version=0xA00+ FsRtlInitializeExtraCreateParameterList
@ stub -version=0xA00+ FsRtlInsertExtraCreateParameter
@ stub -version=0xA00+ FsRtlInsertPerFileContext
@ stub -version=0xA00+ FsRtlInsertPerFileContextWithReserve
@ stub -version=0xA00+ FsRtlIs32BitProcess
@ stub -version=0xA00+ FsRtlIsDaxVolume
@ stub -version=0xA00+ FsRtlIsEcpAcknowledged
@ stub -version=0xA00+ FsRtlIsEcpFromUserMode
@ stub -version=0xA00+ FsRtlIsExtentDangling
@ stub -version=0xA00+ FsRtlIsMobileOS
@ stub -version=0xA00+ FsRtlIsNameInUnUpcasedExpression
@ stub -version=0xA00+ FsRtlIsNonEmptyDirectoryReparsePointAllowed
@ stub -version=0xA00+ FsRtlIsSystemPagingFile
@ stub -version=0xA00+ FsRtlIssueDeviceIoControl
@ stub -version=0xA00+ FsRtlKernelFsControlFile
@ stub -version=0xA00+ FsRtlLogCcFlushError
@ stub -version=0xA00+ FsRtlLookupPerFileContext
@ stub -version=0xA00+ FsRtlMdlReadEx
@ stub -version=0xA00+ FsRtlMupGetProviderIdFromName
@ stub -version=0xA00+ FsRtlMupGetProviderInfoFromFileObject
@ stub -version=0xA00+ FsRtlNotifyCleanupAll
@ stub -version=0xA00+ FsRtlNotifyFilterChangeDirectoryLite
@ stub -version=0xA00+ FsRtlNotifyFilterReportChangeLite
@ stub -version=0xA00+ FsRtlNotifyFilterReportChangeLiteEx
@ stub -version=0xA00+ FsRtlNotifyVolumeEventEx
@ stub -version=0xA00+ FsRtlOpenFileSystemRegistryKeyFromFsGuid
@ stub -version=0xA00+ FsRtlOplockBreakH
@ stub -version=0xA00+ FsRtlOplockBreakH2
@ stub -version=0xA00+ FsRtlOplockBreakToNone
@ stub -version=0xA00+ FsRtlOplockBreakToNoneEx
@ stub -version=0xA00+ FsRtlOplockFsctrlEx
@ stub -version=0xA00+ FsRtlOplockGetAnyBreakOwnerProcess
@ stub -version=0xA00+ FsRtlOplockIsSharedRequest
@ stub -version=0xA00+ FsRtlOplockKeysEqual
@ stub -version=0xA00+ FsRtlPrepareMdlWriteEx
@ stub -version=0xA00+ FsRtlPrepareToReuseEcp
@ stub -version=0xA00+ FsRtlQueryCachedVdl
@ stub -version=0xA00+ FsRtlQueryInformationFile
@ stub -version=0xA00+ FsRtlQueryKernelEaFile
@ stub -version=0xA00+ FsRtlQueryMaximumVirtualDiskNestingLevel
@ stub -version=0xA00+ FsRtlRegisterFltMgrCalls
@ stub -version=0xA00+ FsRtlRegisterMupCalls
@ stub -version=0xA00+ FsRtlRegisterUncProviderEx
@ stub -version=0xA00+ FsRtlRegisterUncProviderEx2
@ stub -version=0xA00+ FsRtlReleaseEofLock
@ stub -version=0xA00+ FsRtlReleaseFileNameInformation
@ stub -version=0xA00+ FsRtlReleaseHeaderMutex
@ stub -version=0xA00+ FsRtlRemoveDotsFromPath
@ stub -version=0xA00+ FsRtlRemoveExtraCreateParameter
@ stub -version=0xA00+ FsRtlRemovePerFileContext
@ stub -version=0xA00+ FsRtlRemovePerFileContextWithReserve
@ stub -version=0xA00+ FsRtlSendModernAppTermination
@ stub -version=0xA00+ FsRtlSetDriverBacking
@ stub -version=0xA00+ FsRtlSetEcpListIntoIrp
@ stub -version=0xA00+ FsRtlSetKernelEaFile
@ stub -version=0xA00+ FsRtlTeardownPerFileContexts
@ stub -version=0xA00+ FsRtlTryToAcquireHeaderMutex
@ stub -version=0xA00+ FsRtlUpdateDiskCounters
@ stub -version=0xA00+ FsRtlUpperOplockFsctrl
@ stub -version=0xA00+ FsRtlValidateReparsePointBuffer
@ stub -version=0xA00+ FsRtlVolumeDeviceToCorrelationId
@ stub -version=0xA00+ HalAcpiGetTableEx
@ stub -version=0xA00+ HalAcquireDisplayOwnership
@ stub -version=0xA00+ HalAdjustResourceList
@ stub -version=0xA00+ HalAllProcessorsStarted
@ stub -version=0xA00+ HalAllocateAdapterChannel
@ stub -version=0xA00+ HalAllocateCommonBuffer
@ stub -version=0xA00+ HalAllocateCrashDumpRegisters
@ stub -version=0xA00+ HalAllocateHardwareCounters
@ stub -version=0xA00+ HalAssignSlotResources
@ stub -version=0xA00+ HalBugCheckSystem
@ stub -version=0xA00+ HalCalibratePerformanceCounter
@ stub -version=0xA00+ HalClearSoftwareInterrupt
@ stub -version=0xA00+ HalConvertDeviceIdtToIrql
@ stub -version=0xA00+ HalDisableInterrupt
@ stub -version=0xA00+ HalDisplayString
@ stub -version=0xA00+ HalDmaAllocateCrashDumpRegistersEx
@ stub -version=0xA00+ HalDmaFreeCrashDumpRegistersEx
@ stub -version=0xA00+ HalEnableInterrupt
@ stub -version=0xA00+ HalEnumerateEnvironmentVariablesEx
@ stub -version=0xA00+ HalEnumerateProcessors
@ stub -version=0xA00+ HalFlushCommonBuffer
@ stub -version=0xA00+ HalFreeCommonBuffer
@ stub -version=0xA00+ HalFreeHardwareCounters
@ stub -version=0xA00+ HalGetAdapter
@ stub -version=0xA00+ HalGetBusData
@ stub -version=0xA00+ HalGetBusDataByOffset
@ stub -version=0xA00+ HalGetEnvironmentVariable
@ stub -version=0xA00+ HalGetEnvironmentVariableEx
@ stub -version=0xA00+ HalGetInterruptTargetInformation
@ stub -version=0xA00+ HalGetInterruptVector
@ stub -version=0xA00+ HalGetMemoryCachingRequirements
@ stub -version=0xA00+ HalGetMessageRoutingInfo
@ stub -version=0xA00+ HalGetProcessorIdByNtNumber
@ stub -version=0xA00+ HalGetVectorInput
@ stub -version=0xA00+ HalHandleMcheck
@ stub -version=0xA00+ HalHandleNMI
@ stub -version=0xA00+ HalInitSystem
@ stub -version=0xA00+ HalInitializeBios
@ stub -version=0xA00+ HalInitializeOnResume
@ stub -version=0xA00+ HalInitializeProcessor
@ stub -version=0xA00+ HalIsHyperThreadingEnabled
@ stub -version=0xA00+ HalMakeBeep
@ stub -version=0xA00+ HalPerformEndOfInterrupt
@ stub -version=0xA00+ HalProcessorIdle
@ stub -version=0xA00+ HalQueryDisplayParameters
@ stub -version=0xA00+ HalQueryEnvironmentVariableInfoEx
@ stub -version=0xA00+ HalQueryMaximumProcessorCount
@ stub -version=0xA00+ HalQueryRealTimeClock
@ stub -version=0xA00+ HalReadDmaCounter
@ stub -version=0xA00+ HalRegisterDynamicProcessor
@ stub -version=0xA00+ HalRegisterErrataCallbacks
@ stub -version=0xA00+ HalReportResourceUsage
@ stub -version=0xA00+ HalRequestClockInterrupt
@ stub -version=0xA00+ HalRequestDeferredRecoveryServiceInterrupt
@ stub -version=0xA00+ HalRequestIpi
@ stub -version=0xA00+ HalRequestIpiSpecifyVector
@ stub -version=0xA00+ HalRequestSoftwareInterrupt
@ stub -version=0xA00+ HalReturnToFirmware
@ stub -version=0xA00+ HalSendNMI
@ stub -version=0xA00+ HalSendSoftwareInterrupt
@ stub -version=0xA00+ HalSetBusData
@ stub -version=0xA00+ HalSetBusDataByOffset
@ stub -version=0xA00+ HalSetDisplayParameters
@ stub -version=0xA00+ HalSetEnvironmentVariable
@ stub -version=0xA00+ HalSetEnvironmentVariableEx
@ stub -version=0xA00+ HalSetProfileInterval
@ stub -version=0xA00+ HalSetRealTimeClock
@ stub -version=0xA00+ HalStartDynamicProcessor
@ stub -version=0xA00+ HalStartNextProcessor
@ stub -version=0xA00+ HalStartProfileInterrupt
@ stub -version=0xA00+ HalStopProfileInterrupt
@ stub -version=0xA00+ HalSystemVectorDispatchEntry
@ stub -version=0xA00+ HalTranslateBusAddress
@ stub -version=0xA00+ HalWheaUpdateCmciPolicy
@ stub -version=0xA00+ HviGetHardwareFeatures
@ stub -version=0xA00+ HviGetHypervisorFeatures
@ stub -version=0xA00+ HviIsAnyHypervisorPresent
@ stub -version=0xA00+ HviIsHypervisorVendorMicrosoft
@ stub -version=0xA00+ HvlGetApicIdFromLpIndex
@ stub -version=0xA00+ HvlGetLpIndexFromApicId
@ stub -version=0xA00+ HvlGetLpIndexFromProcessorIndex
@ stub -version=0xA00+ HvlInvokeFastExtendedHypercall
@ stub -version=0xA00+ HvlInvokeHypercall
@ stub -version=0xA00+ HvlIsSchedulerAssistAvailable
@ stub -version=0xA00+ HvlLpReadMultipleMsr
@ stub -version=0xA00+ HvlLpWriteMultipleMsr
@ stub -version=0xA00+ HvlMapDmaRanges
@ stub -version=0xA00+ HvlPerformEndOfInterrupt
@ stub -version=0xA00+ HvlQueryActiveHypervisorProcessorCount
@ stub -version=0xA00+ HvlQueryActiveProcessors
@ stub -version=0xA00+ HvlQueryConnection
@ stub -version=0xA00+ HvlQueryHypervisorProcessorNodeNumber
@ stub -version=0xA00+ HvlQueryNumaDistance
@ stub -version=0xA00+ HvlQueryProcessorTopology
@ stub -version=0xA00+ HvlQueryProcessorTopologyCount
@ stub -version=0xA00+ HvlQueryProcessorTopologyEx
@ stub -version=0xA00+ HvlQueryProcessorTopologyHighestId
@ stub -version=0xA00+ HvlQueryStartedProcessors
@ stub -version=0xA00+ HvlReadPerformanceStateCounters
@ stub -version=0xA00+ HvlRegisterInterruptCallback
@ stub -version=0xA00+ HvlRegisterWheaErrorNotification
@ stub -version=0xA00+ HvlSchedulerAssistAcknowledgeEvents
@ stub -version=0xA00+ HvlUnmapDmaRanges
@ stub -version=0xA00+ HvlUnregisterInterruptCallback
@ stub -version=0xA00+ HvlUnregisterWheaErrorNotification
@ stub -version=0xA00+ HvlUpdatePerformanceStateCountersForLp
@ stub -version=0xA00+ InbvNotifyDisplayOwnershipChange
@ stub -version=0xA00+ InbvSetVirtualFrameBuffer
@ stub -version=0xA00+ InterlockedPushListSList
@ stub -version=0xA00+ IoAcquireKsrPersistentMemory
@ stub -version=0xA00+ IoAcquireKsrPersistentMemoryEx
@ stub -version=0xA00+ IoAddBugcheckTriageThread
@ stub -version=0xA00+ IoAdjustStackSizeForRedirection
@ stub -version=0xA00+ IoAllocateIrpEx
@ stub -version=0xA00+ IoAllocateMiniCompletionPacket
@ stub -version=0xA00+ IoAllocateSfioStreamIdentifier
@ stub -version=0xA00+ IoApplyPriorityInfoThread
@ stub -version=0xA00+ IoBoostThreadIo
@ stub -version=0xA00+ IoCancelMiniCompletionPacket
@ stub -version=0xA00+ IoCheckFileObjectOpenedAsCopyDestination
@ stub -version=0xA00+ IoCheckFileObjectOpenedAsCopySource
@ stub -version=0xA00+ IoCheckLinkShareAccess
@ stub -version=0xA00+ IoCheckRedirectionTrustLevel
@ stub -version=0xA00+ IoCheckShareAccessEx
@ stub -version=0xA00+ IoCleanupIrp
@ stub -version=0xA00+ IoClearActivityIdThread
@ stub -version=0xA00+ IoClearAdapterCryptoEngineExtension
@ stub -version=0xA00+ IoClearFsTrackOffsetState
@ stub -version=0xA00+ IoClearIrpExtraCreateParameter
@ stub -version=0xA00+ IoComputeRedirectionTrustLevel
@ stub -version=0xA00+ IoConvertFileHandleToKernelHandle
@ stub -version=0xA00+ IoCopyDeviceObjectHint
@ stub -version=0xA00+ IoCreateArcName
@ stub -version=0xA00+ IoCreateDeviceSecure
@ stub -version=0xA00+ IoCreateDriverProxyExtension
@ stub -version=0xA00+ IoCreateFileEx
@ stub -version=0xA00+ IoCreateStreamFileObjectEx2
@ stub -version=0xA00+ IoCreateSymbolicLink2
@ stub -version=0xA00+ IoCreateSystemThread
@ stub -version=0xA00+ IoDecrementKeepAliveCount
@ stub -version=0xA00+ IoDriverProxyCreateHotSwappableWorkerThread
@ stub -version=0xA00+ IoDuplicateDependency
@ stub -version=0xA00+ IoEnumerateKsrPersistentMemoryEx
@ stub -version=0xA00+ IoFlushAdapterBuffers
@ stub -version=0xA00+ IoFreeAdapterChannel
@ stub -version=0xA00+ IoFreeKsrPersistentMemory
@ stub -version=0xA00+ IoFreeMapRegisters
@ stub -version=0xA00+ IoFreeMiniCompletionPacket
@ stub -version=0xA00+ IoFreeSfioStreamIdentifier
@ stub -version=0xA00+ IoGetActivityIdIrp
@ stub -version=0xA00+ IoGetActivityIdThread
@ stub -version=0xA00+ IoGetAdapterCryptoEngineExtension
@ stub -version=0xA00+ IoGetAffinityInterrupt
@ stub -version=0xA00+ IoGetBootDiskInformationLite
@ stub -version=0xA00+ IoGetContainerInformation
@ stub -version=0xA00+ IoGetCopyInformationExtension
@ stub -version=0xA00+ IoGetDeviceDirectory
@ stub -version=0xA00+ IoGetDeviceInterfacePropertyData
@ stub -version=0xA00+ IoGetDeviceNumaNode
@ stub -version=0xA00+ IoGetDevicePropertyData
@ stub -version=0xA00+ IoGetDriverDirectory
@ stub -version=0xA00+ IoGetDriverProxyEndpointWrapper
@ stub -version=0xA00+ IoGetDriverProxyFeatures
@ stub -version=0xA00+ IoGetFsTrackOffsetState
@ stub -version=0xA00+ IoGetFsZeroingOffset
@ stub -version=0xA00+ IoGetGenericIrpExtension
@ stub -version=0xA00+ IoGetInitiatorProcess
@ stub -version=0xA00+ IoGetIoAttributionHandle
@ stub -version=0xA00+ IoGetIoPriorityHint
@ stub -version=0xA00+ IoGetIommuInterface
@ stub -version=0xA00+ IoGetIommuInterfaceEx
@ stub -version=0xA00+ IoGetIrpExtraCreateParameter
@ stub -version=0xA00+ IoGetKsrPersistentMemoryBuffer
@ stub -version=0xA00+ IoGetOplockKeyContext
@ stub -version=0xA00+ IoGetOplockKeyContextEx
@ stub -version=0xA00+ IoGetSfioStreamIdentifier
@ stub -version=0xA00+ IoGetShadowFileInformation
@ stub -version=0xA00+ IoGetSilo
@ stub -version=0xA00+ IoGetSiloParameters
@ stub -version=0xA00+ IoGetSymlinkSupportInformation
@ stub -version=0xA00+ IoGetTransactionParameterBlock
@ stub -version=0xA00+ IoIncrementKeepAliveCount
@ stub -version=0xA00+ IoInitializeIrpEx
@ stub -version=0xA00+ IoInitializeMiniCompletionPacket
@ stub -version=0xA00+ IoInitializeWorkItem
@ stub -version=0xA00+ IoIrpHasFsTrackOffsetExtensionType
@ stub -version=0xA00+ IoIsActivityTracingEnabled
@ stub -version=0xA00+ IoIsFileObjectIgnoringSharing
@ stub -version=0xA00+ IoIsInitiator32bitProcess
@ stub -version=0xA00+ IoIsValidIrpStatus
@ stub -version=0xA00+ IoLoadCrashDumpDriver
@ stub -version=0xA00+ IoMakeAssociatedIrpEx
@ stub -version=0xA00+ IoMapKsrPersistentMemoryEx
@ stub -version=0xA00+ IoMapTransfer
@ stub -version=0xA00+ IoOpenDriverRegistryKey
@ stub -version=0xA00+ IoPropagateActivityIdToThread
@ stub -version=0xA00+ IoPropagateIrpExtension
@ stub -version=0xA00+ IoPropagateIrpExtensionEx
@ stub -version=0xA00+ IoQueryFullDriverPath
@ stub -version=0xA00+ IoQueryInformationByName
@ stub -version=0xA00+ IoQueryInterface
@ stub -version=0xA00+ IoQueryKsrPersistentMemorySize
@ stub -version=0xA00+ IoQueryKsrPersistentMemorySizeEx
@ stub -version=0xA00+ IoQueueWorkItemEx
@ stub -version=0xA00+ IoQueueWorkItemToNode
@ stub -version=0xA00+ IoRecordIoAttribution
@ stub -version=0xA00+ IoRegisterBootDriverCallback
@ stub -version=0xA00+ IoRegisterContainerNotification
@ stub -version=0xA00+ IoRegisterDriverProxyEndpoints
@ stub -version=0xA00+ IoRegisterFsRegistrationChangeMountAware
@ stub -version=0xA00+ IoRegisterIoTracking
@ stub -version=0xA00+ IoRegisterPriorityCallback
@ stub -version=0xA00+ IoRemoveIoCompletion
@ stub -version=0xA00+ IoRemoveLinkShareAccess
@ stub -version=0xA00+ IoRemoveLinkShareAccessEx
@ stub -version=0xA00+ IoReplaceFileObjectName
@ stub -version=0xA00+ IoReplacePartitionUnit
@ stub -version=0xA00+ IoReportInterruptActive
@ stub -version=0xA00+ IoReportInterruptInactive
@ stub -version=0xA00+ IoReportRootDevice
@ stub -version=0xA00+ IoRequestDeviceEjectEx
@ stub -version=0xA00+ IoRequestDeviceRemovalForReset
@ stub -version=0xA00+ IoReserveDependency
@ stub -version=0xA00+ IoReserveKsrPersistentMemory
@ stub -version=0xA00+ IoReserveKsrPersistentMemoryEx
@ stub -version=0xA00+ IoResolveDependency
@ stub -version=0xA00+ IoRetrievePriorityInfo
@ stub -version=0xA00+ IoSetActivityIdIrp
@ stub -version=0xA00+ IoSetActivityIdThread
@ stub -version=0xA00+ IoSetAdapterCryptoEngineExtension
@ stub -version=0xA00+ IoSetDependency
@ stub -version=0xA00+ IoSetDeviceInterfacePropertyData
@ stub -version=0xA00+ IoSetDevicePropertyData
@ stub -version=0xA00+ IoSetFileObjectIgnoreSharing
@ stub -version=0xA00+ IoSetFsTrackOffsetState
@ stub -version=0xA00+ IoSetFsZeroingOffset
@ stub -version=0xA00+ IoSetFsZeroingOffsetRequired
@ stub -version=0xA00+ IoSetGenericIrpExtension
@ stub -version=0xA00+ IoSetIoAttributionIrp
@ stub -version=0xA00+ IoSetIoCompletionEx
@ stub -version=0xA00+ IoSetIoCompletionEx3
@ stub -version=0xA00+ IoSetIoPriorityHint
@ stub -version=0xA00+ IoSetIoPriorityHintIntoFileObject
@ stub -version=0xA00+ IoSetIoPriorityHintIntoThread
@ stub -version=0xA00+ IoSetIrpExtraCreateParameter
@ stub -version=0xA00+ IoSetLinkShareAccess
@ stub -version=0xA00+ IoSetMasterIrpStatus
@ stub -version=0xA00+ IoSetShadowFileInformation
@ stub -version=0xA00+ IoSetShareAccessEx
@ stub -version=0xA00+ IoSizeOfIrpEx
@ stub -version=0xA00+ IoSizeofGenericIrpExtension
@ stub -version=0xA00+ IoSizeofWorkItem
@ stub -version=0xA00+ IoSteerInterrupt
@ stub -version=0xA00+ IoSynchronousCallDriver
@ stub -version=0xA00+ IoTestDependency
@ stub -version=0xA00+ IoTransferActivityId
@ stub -version=0xA00+ IoTryQueueWorkItem
@ stub -version=0xA00+ IoUninitializeWorkItem
@ stub -version=0xA00+ IoUnregisterBootDriverCallback
@ stub -version=0xA00+ IoUnregisterContainerNotification
@ stub -version=0xA00+ IoUnregisterIoTracking
@ stub -version=0xA00+ IoUnregisterPlugPlayNotificationEx
@ stub -version=0xA00+ IoUnregisterPriorityCallback
@ stub -version=0xA00+ IoUpdateLinkShareAccess
@ stub -version=0xA00+ IoUpdateLinkShareAccessEx
@ stub -version=0xA00+ IoVolumeDeviceNameToGuid
@ stub -version=0xA00+ IoVolumeDeviceNameToGuidPath
@ stub -version=0xA00+ IoVolumeDeviceToGuid
@ stub -version=0xA00+ IoVolumeDeviceToGuidPath
@ stub -version=0xA00+ IoWithinStackLimits
@ stub -version=0xA00+ IoWriteKsrPersistentMemory
@ stub -version=0xA00+ IofGetDriverProxyWrapperFromEndpoint
@ stub -version=0xA00+ KasanMarkAddressInvalid
@ stub -version=0xA00+ KasanMarkAddressRedZone
@ stub -version=0xA00+ KasanMarkAddressValid
@ stub -version=0xA00+ KasanTrackAddress
@ stub -version=0xA00+ KasanValidateAddress
@ stub -version=0xA00+ KdAcquireDebuggerLock
@ stub -version=0xA00+ KdDeregisterPowerHandler
@ stub -version=0xA00+ KdGetDebugDevice
@ stub -version=0xA00+ KdLogDbgPrint
@ stub -version=0xA00+ KdPowerTransitionEx
@ stub -version=0xA00+ KdRegisterPowerHandler
@ stub -version=0xA00+ KdReleaseDebuggerLock
@ stub -version=0xA00+ KdSetEventLoggingPresent
@ stub -version=0xA00+ KeAddGroupAffinityEx
@ stub -version=0xA00+ KeAddProcessorAffinityEx
@ stub -version=0xA00+ KeAddProcessorGroupAffinity
@ stub -version=0xA00+ KeAddTriageDumpDataBlock
@ stub -version=0xA00+ KeAlertThread
@ stub -version=0xA00+ KeAllocateCalloutStack
@ stub -version=0xA00+ KeAllocateCalloutStackEx
@ stub -version=0xA00+ KeAllocateProcessorProfileStructures
@ stub -version=0xA00+ KeAndAffinityEx
@ stub -version=0xA00+ KeAndAffinityEx2
@ stub -version=0xA00+ KeAndGroupAffinityEx
@ stub -version=0xA00+ KeCancelTimer2
@ stub -version=0xA00+ KeCheckProcessorAffinityEx
@ stub -version=0xA00+ KeCheckProcessorGroupAffinity
@ stub -version=0xA00+ KeClockInterruptNotify
@ stub -version=0xA00+ KeComplementAffinityEx
@ stub -version=0xA00+ KeComplementAffinityEx2
@ stub -version=0xA00+ KeConnectInterruptForHal
@ stub -version=0xA00+ KeConvertAuxiliaryCounterToPerformanceCounter
@ stub -version=0xA00+ KeConvertPerformanceCounterToAuxiliaryCounter
@ stub -version=0xA00+ KeCopyAffinityEx
@ stub -version=0xA00+ KeCopyAffinityEx2
@ stub -version=0xA00+ KeCountSetBitsAffinityEx
@ stub -version=0xA00+ KeCountSetBitsGroupAffinity
@ stub -version=0xA00+ KeDeregisterBoundCallback
@ stub -version=0xA00+ KeDeregisterProcessorChangeCallback
@ stub -version=0xA00+ KeDispatchSecondaryInterrupt
@ stub -version=0xA00+ KeEnumerateNextProcessor
@ stub -version=0xA00+ KeExpandKernelStackAndCallout
@ stub -version=0xA00+ KeExpandKernelStackAndCalloutEx
@ stub -version=0xA00+ KeFindFirstSetLeftAffinityEx
@ stub -version=0xA00+ KeFindFirstSetLeftGroupAffinity
@ stub -version=0xA00+ KeFindFirstSetRightAffinityEx
@ stub -version=0xA00+ KeFindFirstSetRightGroupAffinity
@ stub -version=0xA00+ KeFirstGroupAffinityEx
@ stub -version=0xA00+ KeFlushCurrentTbImmediately
@ stub -version=0xA00+ KeFlushWriteBuffer
@ stub -version=0xA00+ KeForceEnableNx
@ stub -version=0xA00+ KeFreeCalloutStack
@ stub -version=0xA00+ KeGetClockOwner
@ stub -version=0xA00+ KeGetClockTimerResolution
@ stub -version=0xA00+ KeGetCurrentNodeNumber
@ stub -version=0xA00+ KeGetEffectiveIrql
@ stub -version=0xA00+ KeGetNextClockTickDuration
@ stub -version=0xA00+ KeGetProcessorIndexFromNumber
@ stub -version=0xA00+ KeGetProcessorNumberFromIndex
@ stub -version=0xA00+ KeGetXSaveFeatureFlags
@ stub -version=0xA00+ KeHwPolicyLocateResource
@ stub -version=0xA00+ KeInitializeAffinityEx
@ stub -version=0xA00+ KeInitializeAffinityEx2
@ stub -version=0xA00+ KeInitializeEnumerationContext
@ stub -version=0xA00+ KeInitializeEnumerationContextFromAffinity
@ stub -version=0xA00+ KeInitializeEnumerationContextFromGroup
@ stub -version=0xA00+ KeInitializeSecondaryInterruptServices
@ stub -version=0xA00+ KeInitializeTimer2
@ stub -version=0xA00+ KeInitializeTriageDumpDataArray
@ stub -version=0xA00+ KeInterlockedClearProcessorAffinityEx
@ stub -version=0xA00+ KeInterlockedSetProcessorAffinityEx
@ stub -version=0xA00+ KeInvalidateRangeAllCaches
@ stub -version=0xA00+ KeInvalidateRangeAllCachesNoIpi
@ stub -version=0xA00+ KeIsEmptyAffinityEx
@ stub -version=0xA00+ KeIsEqualAffinityEx
@ stub -version=0xA00+ KeIsSingleGroupAffinityEx
@ stub -version=0xA00+ KeIsSubsetAffinityEx
@ stub -version=0xA00+ KeLoadMTRR
@ stub -version=0xA00+ KeNotifyProcessorFreezeSupported
@ stub -version=0xA00+ KeOrAffinityEx
@ stub -version=0xA00+ KeOrAffinityEx2
@ stub -version=0xA00+ KePrepareToDispatchVirtualProcessor
@ stub -version=0xA00+ KeProcessorGroupAffinity
@ stub -version=0xA00+ KeQueryActiveGroupCount
@ stub -version=0xA00+ KeQueryActiveProcessorAffinity
@ stub -version=0xA00+ KeQueryActiveProcessorAffinity2
@ stub -version=0xA00+ KeQueryActiveProcessorCount
@ stub -version=0xA00+ KeQueryActiveProcessorCountEx
@ stub -version=0xA00+ KeQueryAuxiliaryCounterFrequency
@ stub -version=0xA00+ KeQueryDpcWatchdogInformation
@ stub -version=0xA00+ KeQueryEffectivePriorityThread
@ stub -version=0xA00+ KeQueryGroupAffinity
@ stub -version=0xA00+ KeQueryGroupAffinityEx
@ stub -version=0xA00+ KeQueryHardwareCounterConfiguration
@ stub -version=0xA00+ KeQueryHeteroCpuPolicyThread
@ stub -version=0xA00+ KeQueryHighestNodeNumber
@ stub -version=0xA00+ KeQueryInterruptPartitionCount
@ stub -version=0xA00+ KeQueryInterruptPartitionInformation
@ stub -version=0xA00+ KeQueryInterruptTimePrecise
@ stub -version=0xA00+ KeQueryLogicalProcessorRelationship
@ stub -version=0xA00+ KeQueryMaximumGroupCount
@ stub -version=0xA00+ KeQueryMaximumProcessorCount
@ stub -version=0xA00+ KeQueryMaximumProcessorCountEx
@ stub -version=0xA00+ KeQueryNodeActiveAffinity
@ stub -version=0xA00+ KeQueryNodeActiveAffinity2
@ stub -version=0xA00+ KeQueryNodeActiveProcessorCount
@ stub -version=0xA00+ KeQueryNodeMaximumProcessorCount
@ stub -version=0xA00+ KeQueryPerformanceCounter
@ stub -version=0xA00+ KeQueryPrcbAddress
@ stub -version=0xA00+ KeQuerySystemCpuPartitionAffinity
@ stub -version=0xA00+ KeQuerySystemTimePrecise
@ stub -version=0xA00+ KeQueryTotalCycleTimeThread
@ stub -version=0xA00+ KeQueryTypeEvent
@ stub -version=0xA00+ KeQueryUnbiasedInterruptTime
@ stub -version=0xA00+ KeQueryUnbiasedInterruptTimePrecise
@ stub -version=0xA00+ KeRcuReadLock
@ stub -version=0xA00+ KeRcuReadUnlock
@ stub -version=0xA00+ KeRcuSynchronize
@ stub -version=0xA00+ KeReenterRetpolinedCode
@ stub -version=0xA00+ KeRegisterBoundCallback
@ stub -version=0xA00+ KeRegisterProcessorChangeCallback
@ stub -version=0xA00+ KeReinitializeAffinityEx
@ stub -version=0xA00+ KeRemoveGroupAffinityEx
@ stub -version=0xA00+ KeRemoveProcessorAffinityEx
@ stub -version=0xA00+ KeRemoveProcessorGroupAffinity
@ stub -version=0xA00+ KeRemoveQueueApc
@ stub -version=0xA00+ KeRemoveQueueDpcEx
@ stub -version=0xA00+ KeReportCacheIncoherentDevice
@ stub -version=0xA00+ KeRestoreExtendedProcessorState
@ stub -version=0xA00+ KeRevertToUserAffinityThreadEx
@ stub -version=0xA00+ KeRevertToUserGroupAffinityThread
@ stub -version=0xA00+ KeSaveExtendedProcessorState
@ stub -version=0xA00+ KeSetActualBasePriorityThread
@ stub -version=0xA00+ KeSetCoalescableTimer
@ stub -version=0xA00+ KeSetHardwareCounterConfiguration
@ stub -version=0xA00+ KeSetHeteroCpuPolicyThread
@ stub -version=0xA00+ KeSetLastBranchRecordInUse
@ stub -version=0xA00+ KeSetSelectedCpuSetsThread
@ stub -version=0xA00+ KeSetSystemAffinityThreadEx
@ stub -version=0xA00+ KeSetSystemGroupAffinityThread
@ stub -version=0xA00+ KeSetTargetProcessorDpcEx
@ stub -version=0xA00+ KeSetTimer2
@ stub -version=0xA00+ KeShouldYieldProcessor
@ stub -version=0xA00+ KeSizeOfAffinityEx
@ stub -version=0xA00+ KeSrcuAllocate
@ stub -version=0xA00+ KeSrcuFree
@ stub -version=0xA00+ KeSrcuReadLock
@ stub -version=0xA00+ KeSrcuReadUnlock
@ stub -version=0xA00+ KeSrcuSynchronize
@ stub -version=0xA00+ KeStallExecutionProcessor
@ stub -version=0xA00+ KeStallWhileFrozen
@ stub -version=0xA00+ KeStartDynamicProcessor
@ stub -version=0xA00+ KeSubtractAffinityEx
@ stub -version=0xA00+ KeSubtractAffinityEx2
@ stub -version=0xA00+ KeSweepLocalCaches
@ stub -version=0xA00+ KeSynchronizeTimeToQpc
@ stub -version=0xA00+ KeSystemFullyCacheCoherent
@ stub -version=0xA00+ KeTestAlertThread
@ stub -version=0xA00+ KeTryToAcquireInStackQueuedSpinLockAtDpcLevel
@ stub -version=0xA00+ KeUpdateThreadTag
@ stub -version=0xA00+ KeWriteProtectPAT
@ stub -version=0xA00+ KitLogFeatureUsage
@ stub -version=0xA00+ KseQueryDeviceData
@ stub -version=0xA00+ KseQueryDeviceDataList
@ stub -version=0xA00+ KseQueryDeviceFlags
@ stub -version=0xA00+ KseRegisterShim
@ stub -version=0xA00+ KseRegisterShimEx
@ stub -version=0xA00+ KseSetDeviceFlags
@ stub -version=0xA00+ KseUnregisterShim
@ stub -version=0xA00+ LdrFindResourceEx_U
@ stub -version=0xA00+ LdrResFindResource
@ stub -version=0xA00+ LdrResFindResourceDirectory
@ stub -version=0xA00+ LdrResSearchResource
@ stub -version=0xA00+ LpcReplyWaitReplyPort
@ stub -version=0xA00+ LpcRequestWaitReplyPortEx
@ stub -version=0xA00+ LpcSendWaitReceivePort
@ stub -version=0xA00+ MmAddVerifierSpecialThunks
@ stub -version=0xA00+ MmAllocateContiguousMemoryEx
@ stub -version=0xA00+ MmAllocateContiguousMemorySpecifyCacheNode
@ stub -version=0xA00+ MmAllocateContiguousNodeMemory
@ stub -version=0xA00+ MmAllocateMappingAddressEx
@ stub -version=0xA00+ MmAllocateMdlForIoSpace
@ stub -version=0xA00+ MmAllocateMemoryRanges
@ stub -version=0xA00+ MmAllocateNodePagesForMdlEx
@ stub -version=0xA00+ MmAllocatePartitionNodePagesForMdlEx
@ stub -version=0xA00+ MmAreMdlPagesCached
@ stub -version=0xA00+ MmChangeImageProtection
@ stub -version=0xA00+ MmConfigureGraphicsPtes
@ stub -version=0xA00+ MmCopyMemory
@ stub -version=0xA00+ MmCopyVirtualMemory
@ stub -version=0xA00+ MmForceSectionClosedEx
@ stub -version=0xA00+ MmFreeMemoryRanges
@ stub -version=0xA00+ MmFreePagesFromMdlEx
@ stub -version=0xA00+ MmGetCacheAttribute
@ stub -version=0xA00+ MmGetCacheAttributeEx
@ stub -version=0xA00+ MmGetMaximumFileSectionSize
@ stub -version=0xA00+ MmGetPageBadStatus
@ stub -version=0xA00+ MmGetPhysicalMemoryRangesEx
@ stub -version=0xA00+ MmGetPhysicalMemoryRangesEx2
@ stub -version=0xA00+ MmGetSectionInformation
@ stub -version=0xA00+ MmIsDriverSuspectForVerifier
@ stub -version=0xA00+ MmIsDriverVerifyingByAddress
@ stub -version=0xA00+ MmIsFileSectionActive
@ stub -version=0xA00+ MmLoadSystemImage
@ stub -version=0xA00+ MmLockPreChargedPagedPool
@ stub -version=0xA00+ MmMapMdl
@ stub -version=0xA00+ MmMapMemoryDumpMdlEx
@ stub -version=0xA00+ MmMapViewInSessionSpaceEx
@ stub -version=0xA00+ MmMapViewInSystemSpaceEx
@ stub -version=0xA00+ MmMdlPageContentsState
@ stub -version=0xA00+ MmMdlPagesAreZero
@ stub -version=0xA00+ MmObtainChargesToLockPagedPool
@ stub -version=0xA00+ MmPrefetchVirtualAddresses
@ stub -version=0xA00+ MmProtectDriverSection
@ stub -version=0xA00+ MmQueryMemoryRanges
@ stub -version=0xA00+ MmReturnChargesToLockPagedPool
@ stub -version=0xA00+ MmRotatePhysicalView
@ stub -version=0xA00+ MmSecureVirtualMemoryEx
@ stub -version=0xA00+ MmSetGraphicsPtes
@ stub -version=0xA00+ MmSetPermanentCacheAttribute
@ stub -version=0xA00+ MmUnloadSystemImage
@ stub -version=0xA00+ MmUnlockPreChargedPagedPool
@ stub -version=0xA00+ NtCommitComplete
@ stub -version=0xA00+ NtCommitEnlistment
@ stub -version=0xA00+ NtCommitTransaction
@ stub -version=0xA00+ NtCompareSigningLevels
@ stub -version=0xA00+ NtCopyFileChunk
@ stub -version=0xA00+ NtCreateCrossVmEvent
@ stub -version=0xA00+ NtCreateEnlistment
@ stub -version=0xA00+ NtCreateResourceManager
@ stub -version=0xA00+ NtCreateTransaction
@ stub -version=0xA00+ NtCreateTransactionManager
@ stub -version=0xA00+ NtEnumerateSystemEnvironmentValuesEx
@ stub -version=0xA00+ NtEnumerateTransactionObject
@ stub -version=0xA00+ NtFreezeTransactions
@ stub -version=0xA00+ NtGetEnvironmentVariableEx
@ stub -version=0xA00+ NtGetNotificationResourceManager
@ stub -version=0xA00+ NtImageInfo
@ stub -version=0xA00+ NtNotifyChangeDirectoryFileEx
@ stub -version=0xA00+ NtOpenEnlistment
@ stub -version=0xA00+ NtOpenResourceManager
@ stub -version=0xA00+ NtOpenTransaction
@ stub -version=0xA00+ NtOpenTransactionManager
@ stub -version=0xA00+ NtPrePrepareComplete
@ stub -version=0xA00+ NtPrePrepareEnlistment
@ stub -version=0xA00+ NtPrepareComplete
@ stub -version=0xA00+ NtPrepareEnlistment
@ stub -version=0xA00+ NtPropagationComplete
@ stub -version=0xA00+ NtPropagationFailed
@ stub -version=0xA00+ NtQueryDirectoryFileEx
@ stub -version=0xA00+ NtQueryEnvironmentVariableInfoEx
@ stub -version=0xA00+ NtQueryInformationByName
@ stub -version=0xA00+ NtQueryInformationEnlistment
@ stub -version=0xA00+ NtQueryInformationResourceManager
@ stub -version=0xA00+ NtQueryInformationTransaction
@ stub -version=0xA00+ NtQueryInformationTransactionManager
@ stub -version=0xA00+ NtQuerySecurityAttributesToken
@ stub -version=0xA00+ NtQuerySystemInformationEx
@ stub -version=0xA00+ NtReadFileScatter
@ stub -version=0xA00+ NtReadOnlyEnlistment
@ stub -version=0xA00+ NtRecoverEnlistment
@ stub -version=0xA00+ NtRecoverResourceManager
@ stub -version=0xA00+ NtRecoverTransactionManager
@ stub -version=0xA00+ NtRollbackComplete
@ stub -version=0xA00+ NtRollbackEnlistment
@ stub -version=0xA00+ NtRollbackTransaction
@ stub -version=0xA00+ NtSetCachedSigningLevel
@ stub -version=0xA00+ NtSetInformationEnlistment
@ stub -version=0xA00+ NtSetInformationResourceManager
@ stub -version=0xA00+ NtSetInformationToken
@ stub -version=0xA00+ NtSetInformationTransaction
@ stub -version=0xA00+ NtSetInformationVirtualMemory
@ stub -version=0xA00+ NtThawTransactions
@ stub -version=0xA00+ NtTraceControl
@ stub -version=0xA00+ NtWriteFileGather
@ stub -version=0xA00+ ObCreateObjectTypeEx
@ stub -version=0xA00+ ObDereferenceObjectDeferDelete
@ stub -version=0xA00+ ObDereferenceObjectDeferDeleteWithTag
@ stub -version=0xA00+ ObDuplicateObject
@ stub -version=0xA00+ ObGetFilterVersion
@ stub -version=0xA00+ ObGetObjectType
@ stub -version=0xA00+ ObIsDosDeviceLocallyMapped
@ stub -version=0xA00+ ObIsKernelHandle
@ stub -version=0xA00+ ObOpenObjectByNameEx
@ stub -version=0xA00+ ObOpenObjectByPointerWithTag
@ stub -version=0xA00+ ObQueryNameInfo
@ stub -version=0xA00+ ObReferenceObjectByHandleWithTag
@ stub -version=0xA00+ ObReferenceObjectByPointerWithTag
@ stub -version=0xA00+ ObReferenceObjectSafe
@ stub -version=0xA00+ ObReferenceObjectSafeWithTag
@ stub -version=0xA00+ ObRegisterCallbacks
@ stub -version=0xA00+ ObUnRegisterCallbacks
@ stub -version=0xA00+ ObWaitForMultipleObjects
@ stub -version=0xA00+ ObWaitForSingleObject
@ stub -version=0xA00+ ObfDereferenceObjectWithTag
@ stub -version=0xA00+ ObfReferenceObjectWithTag
@ stub -version=0xA00+ PcwAddInstance
@ stub -version=0xA00+ PcwCloseInstance
@ stub -version=0xA00+ PcwCreateInstance
@ stub -version=0xA00+ PcwRegister
@ stub -version=0xA00+ PcwUnregister
@ stub -version=0xA00+ PfFileInfoNotify
@ stub -version=0xA00+ PoClearPowerRequest
@ stub -version=0xA00+ PoCpuIdledSinceLastCallImprecise
@ stub -version=0xA00+ PoCreatePowerLimitRequest
@ stub -version=0xA00+ PoCreatePowerRequest
@ stub -version=0xA00+ PoCreateThermalRequest
@ stub -version=0xA00+ PoDeletePowerLimitRequest
@ stub -version=0xA00+ PoDeletePowerRequest
@ stub -version=0xA00+ PoDeleteThermalRequest
@ stub -version=0xA00+ PoDirectedDripsClearDeviceFlags
@ stub -version=0xA00+ PoDirectedDripsSetDeviceFlags
@ stub -version=0xA00+ PoDisableSleepStates
@ stub -version=0xA00+ PoEndDeviceBusy
@ stub -version=0xA00+ PoEnergyEstimationEnabled
@ stub -version=0xA00+ PoFxAddComponentRelation
@ stub -version=0xA00+ PoFxAddDeviceRelation
@ stub -version=0xA00+ PoFxCompleteDirectedPowerDown
@ stub -version=0xA00+ PoFxEnableDStateReporting
@ stub -version=0xA00+ PoFxIssueComponentPerfStateChange
@ stub -version=0xA00+ PoFxIssueComponentPerfStateChangeMultiple
@ stub -version=0xA00+ PoFxNotifySurprisePowerOn
@ stub -version=0xA00+ PoFxPowerControl
@ stub -version=0xA00+ PoFxPowerOnCrashdumpDevice
@ stub -version=0xA00+ PoFxProcessorNotification
@ stub -version=0xA00+ PoFxQueryCurrentComponentPerfState
@ stub -version=0xA00+ PoFxRegisterComponentPerfStates
@ stub -version=0xA00+ PoFxRegisterCoreDevice
@ stub -version=0xA00+ PoFxRegisterCrashdumpDevice
@ stub -version=0xA00+ PoFxRegisterDripsWatchdogCallback
@ stub -version=0xA00+ PoFxRegisterPlugin
@ stub -version=0xA00+ PoFxRegisterPluginEx
@ stub -version=0xA00+ PoFxRegisterPrimaryDevice
@ stub -version=0xA00+ PoFxRemoveComponentRelation
@ stub -version=0xA00+ PoFxRemoveDeviceRelation
@ stub -version=0xA00+ PoFxSetComponentLatency
@ stub -version=0xA00+ PoFxSetComponentResidency
@ stub -version=0xA00+ PoFxSetComponentWake
@ stub -version=0xA00+ PoFxSetTargetDripsDevicePowerState
@ stub -version=0xA00+ PoGetProcessorIdleAccounting
@ stub -version=0xA00+ PoGetSystemWake
@ stub -version=0xA00+ PoGetThermalRequestSupport
@ stub -version=0xA00+ PoInitiateProcessorWake
@ stub -version=0xA00+ PoLatencySensitivityHint
@ stub -version=0xA00+ PoNotifyMediaBuffering
@ stub -version=0xA00+ PoNotifyVSyncChange
@ stub -version=0xA00+ PoQueryPowerLimitAttributes
@ stub -version=0xA00+ PoQueryPowerLimitValue
@ stub -version=0xA00+ PoQueryWatchdogTime
@ stub -version=0xA00+ PoReenableSleepStates
@ stub -version=0xA00+ PoRegisterCoalescingCallback
@ stub -version=0xA00+ PoRegisterForEffectivePowerModeNotifications
@ stub -version=0xA00+ PoRegisterPowerSettingCallback
@ stub -version=0xA00+ PoRegisterSstNotificationHandler
@ stub -version=0xA00+ PoSetDeviceBusyEx
@ stub -version=0xA00+ PoSetFixedWakeSource
@ stub -version=0xA00+ PoSetPowerButtonHoldState
@ stub -version=0xA00+ PoSetPowerLimitValue
@ stub -version=0xA00+ PoSetPowerRequest
@ stub -version=0xA00+ PoSetSystemWake
@ stub -version=0xA00+ PoSetSystemWakeDevice
@ stub -version=0xA00+ PoSetThermalActiveCooling
@ stub -version=0xA00+ PoSetThermalPassiveCooling
@ stub -version=0xA00+ PoSetUserPresent
@ stub -version=0xA00+ PoStartDeviceBusy
@ stub -version=0xA00+ PoUnregisterCoalescingCallback
@ stub -version=0xA00+ PoUnregisterFromEffectivePowerModeNotifications
@ stub -version=0xA00+ PoUnregisterPowerSettingCallback
@ stub -version=0xA00+ PoUnregisterSstNotificationHandler
@ stub -version=0xA00+ PoUserShutdownCancelled
@ stub -version=0xA00+ PoUserShutdownInitiated
@ stub -version=0xA00+ PsAcquireProcessExitSynchronization
@ stub -version=0xA00+ PsAcquireSiloHardReference
@ stub -version=0xA00+ PsAdjustWin32kPriorityFloor
@ stub -version=0xA00+ PsAllocSiloContextSlot
@ stub -version=0xA00+ PsAllocateAffinityToken
@ stub -version=0xA00+ PsAssignProcessToJobObject
@ stub -version=0xA00+ PsAttachSiloToCurrentThread
@ stub -version=0xA00+ PsChargeProcessWakeCounter
@ stub -version=0xA00+ PsCheckProcessFileSigningLevel
@ stub -version=0xA00+ PsCreateSiloContext
@ stub -version=0xA00+ PsCreateSystemThreadEx
@ stub -version=0xA00+ PsDereferenceKernelStack
@ stub -version=0xA00+ PsDereferenceSiloContext
@ stub -version=0xA00+ PsDetachSiloFromCurrentThread
@ stub -version=0xA00+ PsEnterPriorityRegion
@ stub -version=0xA00+ PsFreeAffinityToken
@ stub -version=0xA00+ PsFreeSiloContextSlot
@ stub -version=0xA00+ PsGetCurrentProcessWow64Process
@ stub -version=0xA00+ PsGetCurrentServerSilo
@ stub -version=0xA00+ PsGetCurrentServerSiloName
@ stub -version=0xA00+ PsGetCurrentSilo
@ stub -version=0xA00+ PsGetEffectiveContainerId
@ stub -version=0xA00+ PsGetEffectiveServerSilo
@ stub -version=0xA00+ PsGetHostSilo
@ stub -version=0xA00+ PsGetJobProperty
@ stub -version=0xA00+ PsGetJobServerSilo
@ stub -version=0xA00+ PsGetJobSilo
@ stub -version=0xA00+ PsGetParentSilo
@ stub -version=0xA00+ PsGetPermanentSiloContext
@ stub -version=0xA00+ PsGetProcessActiveThreadCount
@ stub -version=0xA00+ PsGetProcessCommonJob
@ stub -version=0xA00+ PsGetProcessDxgProcess
@ stub -version=0xA00+ PsGetProcessMachine
@ stub -version=0xA00+ PsGetProcessProtection
@ stub -version=0xA00+ PsGetProcessSequenceNumber
@ stub -version=0xA00+ PsGetProcessServerSilo
@ stub -version=0xA00+ PsGetProcessSignatureLevel
@ stub -version=0xA00+ PsGetProcessSilo
@ stub -version=0xA00+ PsGetProcessStartKey
@ stub -version=0xA00+ PsGetProcessWow64Process
@ stub -version=0xA00+ PsGetServerSiloServiceSessionId
@ stub -version=0xA00+ PsGetSiloContainerId
@ stub -version=0xA00+ PsGetSiloContext
@ stub -version=0xA00+ PsGetSiloIdentifier
@ stub -version=0xA00+ PsGetSiloMonitorContextSlot
@ stub -version=0xA00+ PsGetThreadCreateTime
@ stub -version=0xA00+ PsGetThreadExitStatus
@ stub -version=0xA00+ PsGetThreadProperty
@ stub -version=0xA00+ PsGetThreadServerSilo
@ stub -version=0xA00+ PsGetWin32KFilterSet
@ stub -version=0xA00+ PsInsertPermanentSiloContext
@ stub -version=0xA00+ PsInsertSiloContext
@ stub -version=0xA00+ PsIsComponentEnabled
@ stub -version=0xA00+ PsIsCurrentThreadInServerSilo
@ stub -version=0xA00+ PsIsCurrentThreadPrefetching
@ stub -version=0xA00+ PsIsDiskCountersEnabled
@ stub -version=0xA00+ PsIsHostSilo
@ stub -version=0xA00+ PsIsProcessCommitRelinquished
@ stub -version=0xA00+ PsIsProcessInAppSilo
@ stub -version=0xA00+ PsIsProtectedProcess
@ stub -version=0xA00+ PsIsProtectedProcessLight
@ stub -version=0xA00+ PsIsWin32KFilterAuditEnabled
@ stub -version=0xA00+ PsIsWin32KFilterAuditEnabledForProcess
@ stub -version=0xA00+ PsIsWin32KFilterEnabled
@ stub -version=0xA00+ PsIsWin32KFilterEnabledForProcess
@ stub -version=0xA00+ PsLeavePriorityRegion
@ stub -version=0xA00+ PsMakeSiloContextPermanent
@ stub -version=0xA00+ PsQueryCurrentApiSetSchema
@ stub -version=0xA00+ PsQueryProcessAttributesByToken
@ stub -version=0xA00+ PsQueryProcessAvailableCpus
@ stub -version=0xA00+ PsQueryProcessAvailableCpusCount
@ stub -version=0xA00+ PsQueryProcessCommandLine
@ stub -version=0xA00+ PsQueryProcessExceptionFlags
@ stub -version=0xA00+ PsQuerySyscallProviderInformation
@ stub -version=0xA00+ PsQuerySystemAvailableCpus
@ stub -version=0xA00+ PsQuerySystemAvailableCpusCount
@ stub -version=0xA00+ PsQueryTotalCycleTimeProcess
@ stub -version=0xA00+ PsReferenceKernelStack
@ stub -version=0xA00+ PsReferenceProcessFilePointer
@ stub -version=0xA00+ PsReferenceSiloContext
@ stub -version=0xA00+ PsRegisterAltSystemCallHandler
@ stub -version=0xA00+ PsRegisterPicoProvider
@ stub -version=0xA00+ PsRegisterProcessAvailableCpusChangeNotification
@ stub -version=0xA00+ PsRegisterSiloMonitor
@ stub -version=0xA00+ PsRegisterSyscallProvider
@ stub -version=0xA00+ PsRegisterSystemAvailableCpusChangeNotification
@ stub -version=0xA00+ PsReleaseProcessExitSynchronization
@ stub -version=0xA00+ PsReleaseProcessWakeCounter
@ stub -version=0xA00+ PsReleaseSiloHardReference
@ stub -version=0xA00+ PsRemoveSiloContext
@ stub -version=0xA00+ PsReplaceSiloContext
@ stub -version=0xA00+ PsResumeProcess
@ stub -version=0xA00+ PsRevertToUserMultipleGroupAffinityThread
@ stub -version=0xA00+ PsSetCreateProcessNotifyRoutineEx
@ stub -version=0xA00+ PsSetCreateProcessNotifyRoutineEx2
@ stub -version=0xA00+ PsSetCreateThreadNotifyRoutineEx
@ stub -version=0xA00+ PsSetCurrentThreadPrefetching
@ stub -version=0xA00+ PsSetJobProperty
@ stub -version=0xA00+ PsSetLoadImageNotifyRoutineEx
@ stub -version=0xA00+ PsSetProcessDxgProcess
@ stub -version=0xA00+ PsSetProcessFaultInformation
@ stub -version=0xA00+ PsSetProcessesWindowState
@ stub -version=0xA00+ PsSetSystemMultipleGroupAffinityThread
@ stub -version=0xA00+ PsSetThreadProperty
@ stub -version=0xA00+ PsStartSiloMonitor
@ stub -version=0xA00+ PsSuspendProcess
@ stub -version=0xA00+ PsTerminateServerSilo
@ stub -version=0xA00+ PsTlsAlloc
@ stub -version=0xA00+ PsTlsFree
@ stub -version=0xA00+ PsTlsGetValue
@ stub -version=0xA00+ PsTlsSetValue
@ stub -version=0xA00+ PsUnEstablishWin32Callouts
@ stub -version=0xA00+ PsUnregisterAvailableCpusChangeNotification
@ stub -version=0xA00+ PsUnregisterSiloMonitor
@ stub -version=0xA00+ PsUnregisterSyscallProvider
@ stub -version=0xA00+ PsUpdateComponentPower
@ stub -version=0xA00+ PsUpdateDiskCounters
@ stub -version=0xA00+ PsUpdateNetworkCounters
@ stub -version=0xA00+ PsWow64GetProcessMachine
@ stub -version=0xA00+ PsWow64IsMachineSupported
@ stub -version=0xA00+ RtlAddAccessAllowedObjectAce
@ stub -version=0xA00+ RtlAddAccessDeniedAceEx
@ stub -version=0xA00+ RtlAddAccessDeniedObjectAce
@ stub -version=0xA00+ RtlAddAccessFilterAce
@ stub -version=0xA00+ RtlAddAtomToAtomTableEx
@ stub -version=0xA00+ RtlAddAuditAccessAceEx
@ stub -version=0xA00+ RtlAddAuditAccessObjectAce
@ stub -version=0xA00+ RtlAddMandatoryAce
@ stub -version=0xA00+ RtlAddProcessTrustLabelAce
@ stub -version=0xA00+ RtlAddResourceAttributeAce
@ stub -version=0xA00+ RtlAreBitsClearEx
@ stub -version=0xA00+ RtlAreBitsSetEx
@ stub -version=0xA00+ RtlAvlInsertNodeEx
@ stub -version=0xA00+ RtlAvlRemoveNode
@ stub -version=0xA00+ RtlCapabilityCheck
@ stub -version=0xA00+ RtlCapabilityCheckForSingleSessionSku
@ stub -version=0xA00+ RtlCheckPortableOperatingSystem
@ stub -version=0xA00+ RtlCheckSystemBootStatusIntegrity
@ stub -version=0xA00+ RtlCheckTokenCapability
@ stub -version=0xA00+ RtlCheckTokenMembership
@ stub -version=0xA00+ RtlCheckTokenMembershipEx
@ stub -version=0xA00+ RtlClearAllBitsEx
@ stub -version=0xA00+ RtlClearBitEx
@ stub -version=0xA00+ RtlClearBitsEx
@ stub -version=0xA00+ RtlCompareAltitudes
@ stub -version=0xA00+ RtlCompareExchangePointerMapping
@ stub -version=0xA00+ RtlCompareExchangePropertyStore
@ stub -version=0xA00+ RtlCompareUnicodeStrings
@ stub -version=0xA00+ RtlComputeCrc32
@ stub -version=0xA00+ RtlConstructCrossVmEventPath
@ stub -version=0xA00+ RtlConstructCrossVmMutexPath
@ stub -version=0xA00+ RtlContractHashTable
@ stub -version=0xA00+ RtlConvertHostPerfCounterToPerfCounter
@ stub -version=0xA00+ RtlCopyBitMap
@ stub -version=0xA00+ RtlCopyBitMapEx
@ stub -version=0xA00+ RtlCopyContext
@ stub -version=0xA00+ RtlCopyExtendedContext
@ stub -version=0xA00+ RtlCopyLuidAndAttributesArray
@ stub -version=0xA00+ RtlCopySidAndAttributesArray
@ stub -version=0xA00+ RtlCrc32
@ stub -version=0xA00+ RtlCrc64
@ stub -version=0xA00+ RtlCreateAtomTableEx
@ stub -version=0xA00+ RtlCreateHashTable
@ stub -version=0xA00+ RtlCreateHashTableEx
@ stub -version=0xA00+ RtlCreateUnicodeStringFromAsciiz
@ stub -version=0xA00+ RtlCreateUserThread
@ stub -version=0xA00+ RtlCultureNameToLCID
@ stub -version=0xA00+ RtlDecompressBufferEx
@ stub -version=0xA00+ RtlDecompressBufferEx2
@ stub -version=0xA00+ RtlDecompressFragmentEx
@ stub -version=0xA00+ RtlDeleteElementGenericTableAvlEx
@ stub -version=0xA00+ RtlDeleteHashTable
@ stub -version=0xA00+ RtlDeriveCapabilitySidsFromName
@ stub -version=0xA00+ RtlDowncaseUnicodeChar
@ stub -version=0xA00+ RtlDrainNonVolatileFlush
@ stub -version=0xA00+ RtlDuplicateUnicodeString
@ stub -version=0xA00+ RtlEndEnumerationHashTable
@ stub -version=0xA00+ RtlEndStrongEnumerationHashTable
@ stub -version=0xA00+ RtlEndWeakEnumerationHashTable
@ stub -version=0xA00+ RtlEnumerateEntryHashTable
@ stub -version=0xA00+ RtlEqualWnfChangeStamps
@ stub -version=0xA00+ RtlEthernetAddressToStringA
@ stub -version=0xA00+ RtlEthernetAddressToStringW
@ stub -version=0xA00+ RtlEthernetStringToAddressA
@ stub -version=0xA00+ RtlEthernetStringToAddressW
@ stub -version=0xA00+ RtlExpandHashTable
@ stub -version=0xA00+ RtlExtendCorrelationVector
@ stub -version=0xA00+ RtlExtractBitMap
@ stub -version=0xA00+ RtlExtractBitMapEx
@ stub -version=0xA00+ RtlFillMemoryNonTemporal
@ stub -version=0xA00+ RtlFillNonVolatileMemory
@ stub -version=0xA00+ RtlFindAceByType
@ stub -version=0xA00+ RtlFindClearBitsAndSetEx
@ stub -version=0xA00+ RtlFindClearBitsEx
@ stub -version=0xA00+ RtlFindClosestEncodableLength
@ stub -version=0xA00+ RtlFindExportedRoutineByName
@ stub -version=0xA00+ RtlFindNextForwardRunClearCapped
@ stub -version=0xA00+ RtlFindNextForwardRunClearEx
@ stub -version=0xA00+ RtlFindNextForwardRunSet
@ stub -version=0xA00+ RtlFindNextForwardRunSetEx
@ stub -version=0xA00+ RtlFindSetBitsAndClearEx
@ stub -version=0xA00+ RtlFindSetBitsEx
@ stub -version=0xA00+ RtlFindUnicodeSubstring
@ stub -version=0xA00+ RtlFirstFreeAce
@ stub -version=0xA00+ RtlFlushFeatureUsage
@ stub -version=0xA00+ RtlFlushNonVolatileMemory
@ stub -version=0xA00+ RtlFlushNonVolatileMemoryRanges
@ stub -version=0xA00+ RtlFormatMessage
@ stub -version=0xA00+ RtlFreeNonVolatileToken
@ stub -version=0xA00+ RtlFreeUTF8String
@ stub -version=0xA00+ RtlGenerateClass5Guid
@ stub -version=0xA00+ RtlGetAcesBufferSize
@ stub -version=0xA00+ RtlGetActiveConsoleId
@ stub -version=0xA00+ RtlGetAppContainerNamedObjectPath
@ stub -version=0xA00+ RtlGetAppContainerParent
@ stub -version=0xA00+ RtlGetAppContainerSidType
@ stub -version=0xA00+ RtlGetConsoleSessionForegroundProcessId
@ stub -version=0xA00+ RtlGetControlSecurityDescriptor
@ stub -version=0xA00+ RtlGetCurrentServiceSessionId
@ stub -version=0xA00+ RtlGetEnabledExtendedAndSupervisorFeatures
@ stub -version=0xA00+ RtlGetEnabledExtendedFeatures
@ stub -version=0xA00+ RtlGetExtendedContextLength
@ stub -version=0xA00+ RtlGetIntegerAtom
@ stub -version=0xA00+ RtlGetMultiTimePrecise
@ stub -version=0xA00+ RtlGetNextEntryHashTable
@ stub -version=0xA00+ RtlGetNonVolatileToken
@ stub -version=0xA00+ RtlGetNtProductType
@ stub -version=0xA00+ RtlGetNtSystemRoot
@ stub -version=0xA00+ RtlGetPersistedStateLocation
@ stub -version=0xA00+ RtlGetProductInfo
@ stub -version=0xA00+ RtlGetSessionProperties
@ stub -version=0xA00+ RtlGetSuiteMask
@ stub -version=0xA00+ RtlGetSystemBootStatus
@ stub -version=0xA00+ RtlGetSystemBootStatusEx
@ stub -version=0xA00+ RtlGetSystemGlobalData
@ stub -version=0xA00+ RtlGetThreadLangIdByIndex
@ stub -version=0xA00+ RtlGetTokenNamedObjectPath
@ stub -version=0xA00+ RtlIdnToAscii
@ stub -version=0xA00+ RtlIdnToNameprepUnicode
@ stub -version=0xA00+ RtlIdnToUnicode
@ stub -version=0xA00+ RtlImageNtHeaderEx
@ stub -version=0xA00+ RtlIncrementCorrelationVector
@ stub -version=0xA00+ RtlInitEnumerationHashTable
@ stub -version=0xA00+ RtlInitStringEx
@ stub -version=0xA00+ RtlInitStrongEnumerationHashTable
@ stub -version=0xA00+ RtlInitUTF8String
@ stub -version=0xA00+ RtlInitUTF8StringEx
@ stub -version=0xA00+ RtlInitWeakEnumerationHashTable
@ stub -version=0xA00+ RtlInitializeBitMapEx
@ stub -version=0xA00+ RtlInitializeCorrelationVector
@ stub -version=0xA00+ RtlInitializeExtendedContext
@ stub -version=0xA00+ RtlInitializeSidEx
@ stub -version=0xA00+ RtlInsertEntryHashTable
@ stub -version=0xA00+ RtlInterlockedClearBitRun
@ stub -version=0xA00+ RtlInterlockedClearBitRunEx
@ stub -version=0xA00+ RtlInterlockedSetBitRun
@ stub -version=0xA00+ RtlInterlockedSetBitRunEx
@ stub -version=0xA00+ RtlInterlockedSetClearRun
@ stub -version=0xA00+ RtlIntersectBitMaps
@ stub -version=0xA00+ RtlIntersectBitMapsEx
@ stub -version=0xA00+ RtlIsApiSetImplemented
@ stub -version=0xA00+ RtlIsCloudFilesPlaceholder
@ stub -version=0xA00+ RtlIsElevatedRid
@ stub -version=0xA00+ RtlIsFunctionalityAvailable
@ stub -version=0xA00+ RtlIsMultiSessionSku
@ stub -version=0xA00+ RtlIsMultiUsersInSessionSku
@ stub -version=0xA00+ RtlIsNonEmptyDirectoryReparsePointAllowed
@ stub -version=0xA00+ RtlIsNormalizedString
@ stub -version=0xA00+ RtlIsNtDdiVersionAvailable
@ stub -version=0xA00+ RtlIsPartialPlaceholder
@ stub -version=0xA00+ RtlIsPartialPlaceholderFileHandle
@ stub -version=0xA00+ RtlIsPartialPlaceholderFileInfo
@ stub -version=0xA00+ RtlIsProcessorFeaturePresent
@ stub -version=0xA00+ RtlIsSandboxedToken
@ stub -version=0xA00+ RtlIsServicePackVersionInstalled
@ stub -version=0xA00+ RtlIsStateSeparationEnabled
@ stub -version=0xA00+ RtlIsUntrustedObject
@ stub -version=0xA00+ RtlIsZeroMemory
@ stub -version=0xA00+ RtlLCIDToCultureName
@ stub -version=0xA00+ RtlLargeIntegerToChar
@ stub -version=0xA00+ RtlLoadString
@ stub -version=0xA00+ RtlLocalTimeToSystemTime
@ stub -version=0xA00+ RtlLocateSupervisorFeature
@ stub -version=0xA00+ RtlLogUnexpectedCodepath
@ stub -version=0xA00+ RtlLookupEntryHashTable
@ stub -version=0xA00+ RtlLookupFirstMatchingElementGenericTableAvl
@ stub -version=0xA00+ RtlMergeBitMaps
@ stub -version=0xA00+ RtlMergeBitMapsEx
@ stub -version=0xA00+ RtlNormalizeSecurityDescriptor
@ stub -version=0xA00+ RtlNormalizeString
@ stub -version=0xA00+ RtlNumberOfClearBitsEx
@ stub -version=0xA00+ RtlNumberOfClearBitsInRange
@ stub -version=0xA00+ RtlNumberOfSetBitsEx
@ stub -version=0xA00+ RtlNumberOfSetBitsInRange
@ stub -version=0xA00+ RtlNumberOfSetBitsInRangeEx
@ stub -version=0xA00+ RtlNumberOfSetBitsUlongPtr
@ stub -version=0xA00+ RtlOpenCurrentUser
@ stub -version=0xA00+ RtlOpenImageFileOptionsKey
@ stub -version=0xA00+ RtlOsDeploymentState
@ stub -version=0xA00+ RtlOwnerAcesPresent
@ stub -version=0xA00+ RtlPcToFileName
@ stub -version=0xA00+ RtlPcToFilePath
@ stub -version=0xA00+ RtlQueryAllFeatureConfigurations
@ stub -version=0xA00+ RtlQueryAllInternalFeatureConfigurations
@ stub -version=0xA00+ RtlQueryAllInternalRuntimeFeatureConfigurations
@ stub -version=0xA00+ RtlQueryDynamicTimeZoneInformation
@ stub -version=0xA00+ RtlQueryElevationFlags
@ stub -version=0xA00+ RtlQueryImageFileKeyOption
@ stub -version=0xA00+ RtlQueryInformationAcl
@ stub -version=0xA00+ RtlQueryInternalFeatureConfiguration
@ stub -version=0xA00+ RtlQueryModuleInformation
@ stub -version=0xA00+ RtlQueryPackageClaims
@ stub -version=0xA00+ RtlQueryPackageIdentity
@ stub -version=0xA00+ RtlQueryPackageIdentityEx
@ stub -version=0xA00+ RtlQueryPointerMapping
@ stub -version=0xA00+ RtlQueryProcessPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlQueryPropertyStore
@ stub -version=0xA00+ RtlQueryRegistryValueWithFallback
@ stub -version=0xA00+ RtlQueryRegistryValuesEx
@ stub -version=0xA00+ RtlQueryThreadPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlQueryValidationRunlevel
@ stub -version=0xA00+ RtlRaiseCustomSystemEventTrigger
@ stub -version=0xA00+ RtlRaiseStatus
@ stub -version=0xA00+ RtlRbInsertNodeEx
@ stub -version=0xA00+ RtlRbRemoveNode
@ stub -version=0xA00+ RtlRbReplaceNode
@ stub -version=0xA00+ RtlRemoveEntryHashTable
@ stub -version=0xA00+ RtlRemovePointerMapping
@ stub -version=0xA00+ RtlRemovePropertyStore
@ stub -version=0xA00+ RtlReplaceSidInSd
@ stub -version=0xA00+ RtlRestoreSystemBootStatusDefaults
@ stub -version=0xA00+ RtlRunOnceBeginInitialize
@ stub -version=0xA00+ RtlRunOnceComplete
@ stub -version=0xA00+ RtlRunOnceExecuteOnce
@ stub -version=0xA00+ RtlRunOnceInitialize
@ stub -version=0xA00+ RtlSetActiveConsoleId
@ stub -version=0xA00+ RtlSetAllBitsEx
@ stub -version=0xA00+ RtlSetBitEx
@ stub -version=0xA00+ RtlSetBitsEx
@ stub -version=0xA00+ RtlSetConsoleSessionForegroundProcessId
@ stub -version=0xA00+ RtlSetControlSecurityDescriptor
@ stub -version=0xA00+ RtlSetDynamicTimeZoneInformation
@ stub -version=0xA00+ RtlSetPortableOperatingSystem
@ stub -version=0xA00+ RtlSetProcessPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlSetSystemBootStatus
@ stub -version=0xA00+ RtlSetSystemBootStatusEx
@ stub -version=0xA00+ RtlSetSystemGlobalData
@ stub -version=0xA00+ RtlSetThreadPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlShiftLeftBitMap
@ stub -version=0xA00+ RtlShiftLeftBitMapEx
@ stub -version=0xA00+ RtlSidHashInitialize
@ stub -version=0xA00+ RtlSidHashLookup
@ stub -version=0xA00+ RtlStringFromGUIDEx
@ stub -version=0xA00+ RtlStronglyEnumerateEntryHashTable
@ stub -version=0xA00+ RtlSuffixUnicodeString
@ stub -version=0xA00+ RtlSystemTimeToLocalTime
@ stub -version=0xA00+ RtlTestBitEx
@ stub -version=0xA00+ RtlUTF8StringToUnicodeString
@ stub -version=0xA00+ RtlUdiv128
@ stub -version=0xA00+ RtlUnicodeStringToInt64
@ stub -version=0xA00+ RtlUnicodeStringToUTF8String
@ stub -version=0xA00+ RtlUnsignedMultiplyHigh
@ stub -version=0xA00+ RtlValidAcl
@ stub -version=0xA00+ RtlValidateCorrelationVector
@ stub -version=0xA00+ RtlValidateUnicodeString
@ stub -version=0xA00+ RtlVirtualUnwind2
@ stub -version=0xA00+ RtlWeaklyEnumerateEntryHashTable
@ stub -version=0xA00+ RtlWriteNonVolatileMemory
@ stub -version=0xA00+ SeAccessCheckEx
@ stub -version=0xA00+ SeAccessCheckFromState
@ stub -version=0xA00+ SeAccessCheckFromStateEx
@ stub -version=0xA00+ SeAccessCheckWithHint
@ stub -version=0xA00+ SeAdjustAccessStateForAccessConstraints
@ stub -version=0xA00+ SeAdjustAccessStateForTrustLabel
@ stub -version=0xA00+ SeAdjustObjectSecurity
@ stub -version=0xA00+ SeAuditFipsCryptoSelftests
@ stub -version=0xA00+ SeAuditHardLinkCreationWithTransaction
@ stub -version=0xA00+ SeAuditTransactionStateChange
@ stub -version=0xA00+ SeAuditingAnyFileEventsWithContext
@ stub -version=0xA00+ SeAuditingAnyFileEventsWithContextEx
@ stub -version=0xA00+ SeAuditingFileEventsWithContextEx
@ stub -version=0xA00+ SeAuditingWithTokenForSubcategory
@ stub -version=0xA00+ SeCaptureSubjectContextEx
@ stub -version=0xA00+ SeCheckForCriticalAceRemoval
@ stub -version=0xA00+ SeCloseObjectAuditAlarmForNonObObject
@ stub -version=0xA00+ SeCompareSigningLevels
@ stub -version=0xA00+ SeComputeAutoInheritByObjectType
@ stub -version=0xA00+ SeConvertSecurityDescriptorToStringSecurityDescriptor
@ stub -version=0xA00+ SeConvertSidToStringSid
@ stub -version=0xA00+ SeConvertStringSecurityDescriptorToSecurityDescriptor
@ stub -version=0xA00+ SeConvertStringSidToSid
@ stub -version=0xA00+ SeCreateAccessStateEx
@ stub -version=0xA00+ SeCreateAndRegisterAccessCheckDebugContext
@ stub -version=0xA00+ SeCreateClientSecurityEx
@ stub -version=0xA00+ SeCreateClientSecurityFromSubjectContextEx
@ stub -version=0xA00+ SeDeleteClientSecurity
@ stub -version=0xA00+ SeDeleteObjectAuditAlarmWithTransaction
@ stub -version=0xA00+ SeEtwWriteKMCveEvent
@ stub -version=0xA00+ SeExamineSacl
@ stub -version=0xA00+ SeGetCachedSigningLevel
@ stub -version=0xA00+ SeGetLinkedToken
@ stub -version=0xA00+ SeGetLogonSessionToken
@ stub -version=0xA00+ SeIsParentOfChildAppContainer
@ stub -version=0xA00+ SeLocateProcessImageName
@ stub -version=0xA00+ SeMarkLogonSessionForTerminationNotificationEx
@ stub -version=0xA00+ SeOpenObjectAuditAlarmForNonObObject
@ stub -version=0xA00+ SeOpenObjectAuditAlarmWithTransaction
@ stub -version=0xA00+ SeOpenObjectForDeleteAuditAlarmWithTransaction
@ stub -version=0xA00+ SeQuerySecureBootPlatformManifest
@ stub -version=0xA00+ SeQuerySecureBootPolicyValue
@ stub -version=0xA00+ SeQuerySecurityAttributesToken
@ stub -version=0xA00+ SeQuerySecurityAttributesTokenAccessInformation
@ stub -version=0xA00+ SeQueryServerSiloToken
@ stub -version=0xA00+ SeQuerySessionIdTokenEx
@ stub -version=0xA00+ SeRegisterImageVerificationCallback
@ stub -version=0xA00+ SeRegisterLogonSessionTerminatedRoutineEx
@ stub -version=0xA00+ SeReportSecurityEventWithSubCategory
@ stub -version=0xA00+ SeSecurityAttributePresent
@ stub -version=0xA00+ SeSetSecurityAttributesToken
@ stub -version=0xA00+ SeSetSecurityAttributesTokenEx
@ stub -version=0xA00+ SeSetSessionIdTokenWithLinked
@ stub -version=0xA00+ SeShouldCheckForAccessRightsFromParent
@ stub -version=0xA00+ SeSrpAccessCheck
@ stub -version=0xA00+ SeTokenFromAccessInformation
@ stub -version=0xA00+ SeTokenIsWriteRestricted
@ stub -version=0xA00+ SeUnRegisterAndFreeAccessCheckDebugContext
@ stub -version=0xA00+ SeUnregisterImageVerificationCallback
@ stub -version=0xA00+ SeUnregisterLogonSessionTerminatedRoutineEx
@ stub -version=0xA00+ SkAcquirePushLockExclusive
@ stub -version=0xA00+ SkAllocatePool
@ stub -version=0xA00+ SkFreePool
@ stub -version=0xA00+ SkInitializePushLock
@ stub -version=0xA00+ SkIsSecureKernel
@ stub -version=0xA00+ SkQuerySecureKernelInformation
@ stub -version=0xA00+ SkReleasePushLockExclusive
@ stub -version=0xA00+ TmCancelPropagationRequest
@ stub -version=0xA00+ TmCommitComplete
@ stub -version=0xA00+ TmCommitEnlistment
@ stub -version=0xA00+ TmCommitTransaction
@ stub -version=0xA00+ TmCreateEnlistment
@ stub -version=0xA00+ TmCurrentTransaction
@ stub -version=0xA00+ TmDereferenceEnlistmentKey
@ stub -version=0xA00+ TmEnableCallbacks
@ stub -version=0xA00+ TmEndPropagationRequest
@ stub -version=0xA00+ TmFreezeTransactions
@ stub -version=0xA00+ TmGetTransactionId
@ stub -version=0xA00+ TmInitSystem
@ stub -version=0xA00+ TmInitSystemPhase2
@ stub -version=0xA00+ TmInitializeTransactionManager
@ stub -version=0xA00+ TmIsKTMCommitCoordinator
@ stub -version=0xA00+ TmIsTransactionActive
@ stub -version=0xA00+ TmPrePrepareComplete
@ stub -version=0xA00+ TmPrePrepareEnlistment
@ stub -version=0xA00+ TmPrepareComplete
@ stub -version=0xA00+ TmPrepareEnlistment
@ stub -version=0xA00+ TmPropagationComplete
@ stub -version=0xA00+ TmPropagationFailed
@ stub -version=0xA00+ TmReadOnlyEnlistment
@ stub -version=0xA00+ TmRecoverEnlistment
@ stub -version=0xA00+ TmRecoverResourceManager
@ stub -version=0xA00+ TmRecoverTransactionManager
@ stub -version=0xA00+ TmReferenceEnlistmentKey
@ stub -version=0xA00+ TmRenameTransactionManager
@ stub -version=0xA00+ TmRequestOutcomeEnlistment
@ stub -version=0xA00+ TmRollbackComplete
@ stub -version=0xA00+ TmRollbackEnlistment
@ stub -version=0xA00+ TmRollbackTransaction
@ stub -version=0xA00+ TmSetCurrentTransaction
@ stub -version=0xA00+ TmSinglePhaseReject
@ stub -version=0xA00+ TmThawTransactions
@ stub -version=0xA00+ TtmNotifyDeviceArrival
@ stub -version=0xA00+ TtmNotifyDeviceDeparture
@ stub -version=0xA00+ TtmNotifyDeviceInput
@ stub -version=0xA00+ VfInsertContext
@ stub -version=0xA00+ VfQueryDeviceContext
@ stub -version=0xA00+ VfQueryDispatchTable
@ stub -version=0xA00+ VfQueryDriverContext
@ stub -version=0xA00+ VfQueryIrpContext
@ stub -version=0xA00+ VfRemoveContext
@ stub -version=0xA00+ VslCreateSecureSection
@ stub -version=0xA00+ VslDeleteSecureSection
@ stub -version=0xA00+ VslExchangeEntropy
@ stub -version=0xA00+ VslGetSecurePciDeviceAlternateFunctionNumberForVtl0Dma
@ stub -version=0xA00+ VslGetSecurePciDeviceBootConfiguration
@ stub -version=0xA00+ VslGetSecurePciEnabled
@ stub -version=0xA00+ VslQuerySecureDevice
@ stub -version=0xA00+ VslRetrieveMailbox
@ stub -version=0xA00+ WheaAddErrorSource
@ stub -version=0xA00+ WheaAddErrorSourceDeviceDriver
@ stub -version=0xA00+ WheaAddErrorSourceDeviceDriverV1
@ stub -version=0xA00+ WheaAddHwErrorReportSectionDeviceDriver
@ stub -version=0xA00+ WheaAttemptClearPoison
@ stub -version=0xA00+ WheaAttemptPhysicalPageOffline
@ stub -version=0xA00+ WheaAttemptRowOffline
@ stub -version=0xA00+ WheaConfigureErrorSource
@ stub -version=0xA00+ WheaCreateHwErrorReportDeviceDriver
@ stub -version=0xA00+ WheaDeferredRecoveryService
@ stub -version=0xA00+ WheaEnterCriticalState
@ stub -version=0xA00+ WheaErrorSourceGetState
@ stub -version=0xA00+ WheaExitCriticalState
@ stub -version=0xA00+ WheaGetCurrentProcessName
@ stub -version=0xA00+ WheaGetErrorSource
@ stub -version=0xA00+ WheaGetErrorSourceInfo
@ stub -version=0xA00+ WheaGetNotifyAllOfflinesPolicy
@ stub -version=0xA00+ WheaHighIrqlLogSelEventHandlerRegister
@ stub -version=0xA00+ WheaHighIrqlLogSelEventHandlerUnregister
@ stub -version=0xA00+ WheaHwErrorReportAbandonDeviceDriver
@ stub -version=0xA00+ WheaHwErrorReportGetLogDataBufferDeviceDriver
@ stub -version=0xA00+ WheaHwErrorReportMarkAsCriticalDeviceDriver
@ stub -version=0xA00+ WheaHwErrorReportSetFatalSeverityDeviceDriver
@ stub -version=0xA00+ WheaHwErrorReportSetSectionNameDeviceDriver
@ stub -version=0xA00+ WheaHwErrorReportSetSeverityDeviceDriver
@ stub -version=0xA00+ WheaHwErrorReportSubmitDeviceDriver
@ stub -version=0xA00+ WheaInitializeDeferredRecoveryObject
@ stub -version=0xA00+ WheaInitializeRecordHeader
@ stub -version=0xA00+ WheaIsAltContextAllocPossible
@ stub -version=0xA00+ WheaIsCriticalState
@ stub -version=0xA00+ WheaIsLogSelHandlerInitialized
@ stub -version=0xA00+ WheaLogInternalEvent
@ stub -version=0xA00+ WheaPrmTranslateDimmAddress
@ stub -version=0xA00+ WheaPrmTranslatePhysicalAddress
@ stub -version=0xA00+ WheaProcessWaitingETWEvents
@ stub -version=0xA00+ WheaRecoveryBugCheck
@ stub -version=0xA00+ WheaRegisterErrorSourceOverride
@ stub -version=0xA00+ WheaRegisterInUsePageOfflineNotification
@ stub -version=0xA00+ WheaRemoveErrorSource
@ stub -version=0xA00+ WheaRemoveErrorSourceDeviceDriver
@ stub -version=0xA00+ WheaReportFatalHwErrorDeviceDriverEx
@ stub -version=0xA00+ WheaReportHwError
@ stub -version=0xA00+ WheaReportHwErrorDeviceDriver
@ stub -version=0xA00+ WheaReportHwErrorDeviceDriverEx
@ stub -version=0xA00+ WheaRequestDeferredRecovery
@ stub -version=0xA00+ WheaSignalHandlerOverride
@ stub -version=0xA00+ WheaTerminateProcess
@ stub -version=0xA00+ WheaUnconfigureErrorSource
@ stub -version=0xA00+ WheaUnregisterErrorSourceOverride
@ stub -version=0xA00+ WheaUnregisterInUsePageOfflineNotification
@ stub -version=0xA00+ ZwAllocateLocallyUniqueId
@ stub -version=0xA00+ ZwAllocateReserveObject
@ stub -version=0xA00+ ZwAllocateVirtualMemoryEx
@ stub -version=0xA00+ ZwAlpcAcceptConnectPort
@ stub -version=0xA00+ ZwAlpcCancelMessage
@ stub -version=0xA00+ ZwAlpcConnectPort
@ stub -version=0xA00+ ZwAlpcConnectPortEx
@ stub -version=0xA00+ ZwAlpcCreatePort
@ stub -version=0xA00+ ZwAlpcCreatePortSection
@ stub -version=0xA00+ ZwAlpcCreateResourceReserve
@ stub -version=0xA00+ ZwAlpcCreateSectionView
@ stub -version=0xA00+ ZwAlpcCreateSecurityContext
@ stub -version=0xA00+ ZwAlpcDeletePortSection
@ stub -version=0xA00+ ZwAlpcDeleteResourceReserve
@ stub -version=0xA00+ ZwAlpcDeleteSectionView
@ stub -version=0xA00+ ZwAlpcDeleteSecurityContext
@ stub -version=0xA00+ ZwAlpcDisconnectPort
@ stub -version=0xA00+ ZwAlpcOpenSenderProcess
@ stub -version=0xA00+ ZwAlpcOpenSenderThread
@ stub -version=0xA00+ ZwAlpcQueryInformation
@ stub -version=0xA00+ ZwAlpcQueryInformationMessage
@ stub -version=0xA00+ ZwAlpcSendWaitReceivePort
@ stub -version=0xA00+ ZwAlpcSetInformation
@ stub -version=0xA00+ ZwAssociateWaitCompletionPacket
@ stub -version=0xA00+ ZwCancelIoFileEx
@ stub -version=0xA00+ ZwCancelWaitCompletionPacket
@ stub -version=0xA00+ ZwCommitComplete
@ stub -version=0xA00+ ZwCommitEnlistment
@ stub -version=0xA00+ ZwCommitRegistryTransaction
@ stub -version=0xA00+ ZwCommitTransaction
@ stub -version=0xA00+ ZwCompareTokens
@ stub -version=0xA00+ ZwCreateCpuPartition
@ stub -version=0xA00+ ZwCreateCrossVmEvent
@ stub -version=0xA00+ ZwCreateEnlistment
@ stub -version=0xA00+ ZwCreateIoCompletion
@ stub -version=0xA00+ ZwCreateKeyTransacted
@ stub -version=0xA00+ ZwCreatePartition
@ stub -version=0xA00+ ZwCreateProcessEx
@ stub -version=0xA00+ ZwCreateProfileEx
@ stub -version=0xA00+ ZwCreateRegistryTransaction
@ stub -version=0xA00+ ZwCreateResourceManager
@ stub -version=0xA00+ ZwCreateSectionEx
@ stub -version=0xA00+ ZwCreateSemaphore
@ stub -version=0xA00+ ZwCreateTransaction
@ stub -version=0xA00+ ZwCreateTransactionManager
@ stub -version=0xA00+ ZwCreateWaitCompletionPacket
@ stub -version=0xA00+ ZwCreateWnfStateName
@ stub -version=0xA00+ ZwDeleteWnfStateData
@ stub -version=0xA00+ ZwDeleteWnfStateName
@ stub -version=0xA00+ ZwEnumerateTransactionObject
@ stub -version=0xA00+ ZwFlushBuffersFile
@ stub -version=0xA00+ ZwFlushBuffersFileEx
@ stub -version=0xA00+ ZwGetCachedSigningLevel
@ stub -version=0xA00+ ZwGetNextProcess
@ stub -version=0xA00+ ZwGetNextThread
@ stub -version=0xA00+ ZwGetNotificationResourceManager
@ stub -version=0xA00+ ZwGetWriteWatch
@ stub -version=0xA00+ ZwImpersonateAnonymousToken
@ stub -version=0xA00+ ZwLoadKeyEx
@ stub -version=0xA00+ ZwLockFile
@ stub -version=0xA00+ ZwLockProductActivationKeys
@ stub -version=0xA00+ ZwLockVirtualMemory
@ stub -version=0xA00+ ZwManagePartition
@ stub -version=0xA00+ ZwMapViewOfSectionEx
@ stub -version=0xA00+ ZwNotifyChangeDirectoryFile
@ stub -version=0xA00+ ZwNotifyChangeDirectoryFileEx
@ stub -version=0xA00+ ZwNotifyChangeSession
@ stub -version=0xA00+ ZwOpenCpuPartition
@ stub -version=0xA00+ ZwOpenEnlistment
@ stub -version=0xA00+ ZwOpenKeyEx
@ stub -version=0xA00+ ZwOpenKeyTransacted
@ stub -version=0xA00+ ZwOpenKeyTransactedEx
@ stub -version=0xA00+ ZwOpenPartition
@ stub -version=0xA00+ ZwOpenRegistryTransaction
@ stub -version=0xA00+ ZwOpenResourceManager
@ stub -version=0xA00+ ZwOpenSession
@ stub -version=0xA00+ ZwOpenTransaction
@ stub -version=0xA00+ ZwOpenTransactionManager
@ stub -version=0xA00+ ZwPrePrepareComplete
@ stub -version=0xA00+ ZwPrePrepareEnlistment
@ stub -version=0xA00+ ZwPrepareComplete
@ stub -version=0xA00+ ZwPrepareEnlistment
@ stub -version=0xA00+ ZwPropagationComplete
@ stub -version=0xA00+ ZwPropagationFailed
@ stub -version=0xA00+ ZwProtectVirtualMemory
@ stub -version=0xA00+ ZwQueryDirectoryFileEx
@ stub -version=0xA00+ ZwQueryInformationByName
@ stub -version=0xA00+ ZwQueryInformationCpuPartition
@ stub -version=0xA00+ ZwQueryInformationEnlistment
@ stub -version=0xA00+ ZwQueryInformationResourceManager
@ stub -version=0xA00+ ZwQueryInformationTransaction
@ stub -version=0xA00+ ZwQueryInformationTransactionManager
@ stub -version=0xA00+ ZwQueryIntervalProfile
@ stub -version=0xA00+ ZwQueryLicenseValue
@ stub -version=0xA00+ ZwQueryQuotaInformationFile
@ stub -version=0xA00+ ZwQuerySecurityAttributesToken
@ stub -version=0xA00+ ZwQuerySecurityPolicy
@ stub -version=0xA00+ ZwQuerySystemEnvironmentValueEx
@ stub -version=0xA00+ ZwQuerySystemInformationEx
@ stub -version=0xA00+ ZwQueryTimerResolution
@ stub -version=0xA00+ ZwQueryVirtualMemory
@ stub -version=0xA00+ ZwQueryWnfStateData
@ stub -version=0xA00+ ZwQueryWnfStateNameInformation
@ stub -version=0xA00+ ZwReadOnlyEnlistment
@ stub -version=0xA00+ ZwRecoverEnlistment
@ stub -version=0xA00+ ZwRecoverResourceManager
@ stub -version=0xA00+ ZwRecoverTransactionManager
@ stub -version=0xA00+ ZwReleaseSemaphore
@ stub -version=0xA00+ ZwRemoveIoCompletion
@ stub -version=0xA00+ ZwRemoveIoCompletionEx
@ stub -version=0xA00+ ZwRenameKey
@ stub -version=0xA00+ ZwRequestPort
@ stub -version=0xA00+ ZwResetWriteWatch
@ stub -version=0xA00+ ZwResumeThread
@ stub -version=0xA00+ ZwRollbackComplete
@ stub -version=0xA00+ ZwRollbackEnlistment
@ stub -version=0xA00+ ZwRollbackRegistryTransaction
@ stub -version=0xA00+ ZwRollbackTransaction
@ stub -version=0xA00+ ZwSetCachedSigningLevel
@ stub -version=0xA00+ ZwSetInformationCpuPartition
@ stub -version=0xA00+ ZwSetInformationEnlistment
@ stub -version=0xA00+ ZwSetInformationKey
@ stub -version=0xA00+ ZwSetInformationResourceManager
@ stub -version=0xA00+ ZwSetInformationToken
@ stub -version=0xA00+ ZwSetInformationTransaction
@ stub -version=0xA00+ ZwSetInformationVirtualMemory
@ stub -version=0xA00+ ZwSetIntervalProfile
@ stub -version=0xA00+ ZwSetIoCompletion
@ stub -version=0xA00+ ZwSetIoCompletionEx
@ stub -version=0xA00+ ZwSetQuotaInformationFile
@ stub -version=0xA00+ ZwSetSystemEnvironmentValueEx
@ stub -version=0xA00+ ZwSetTimerEx
@ stub -version=0xA00+ ZwSetTimerResolution
@ stub -version=0xA00+ ZwStartProfile
@ stub -version=0xA00+ ZwStopProfile
@ stub -version=0xA00+ ZwSuspendThread
@ stub -version=0xA00+ ZwSystemDebugControl
@ stub -version=0xA00+ ZwTraceControl
@ stub -version=0xA00+ ZwTraceEvent
@ stub -version=0xA00+ ZwUnloadKey2
@ stub -version=0xA00+ ZwUnloadKeyEx
@ stub -version=0xA00+ ZwUnlockFile
@ stub -version=0xA00+ ZwUnlockVirtualMemory
@ stub -version=0xA00+ ZwUpdateWnfStateData
@ stub -version=0xA00+ __asan_alloca_poison
@ stub -version=0xA00+ __asan_allocas_unpoison
@ stub -version=0xA00+ __asan_load1
@ stub -version=0xA00+ __asan_load16
@ stub -version=0xA00+ __asan_load16_volatile
@ stub -version=0xA00+ __asan_load1_volatile
@ stub -version=0xA00+ __asan_load2
@ stub -version=0xA00+ __asan_load2_volatile
@ stub -version=0xA00+ __asan_load4
@ stub -version=0xA00+ __asan_load4_volatile
@ stub -version=0xA00+ __asan_load8
@ stub -version=0xA00+ __asan_load8_volatile
@ stub -version=0xA00+ __asan_loadN
@ stub -version=0xA00+ __asan_loadN_volatile
@ stub -version=0xA00+ __asan_memcpy
@ stub -version=0xA00+ __asan_memmove
@ stub -version=0xA00+ __asan_memset
@ stub -version=0xA00+ __asan_report_load1
@ stub -version=0xA00+ __asan_report_load16
@ stub -version=0xA00+ __asan_report_load2
@ stub -version=0xA00+ __asan_report_load4
@ stub -version=0xA00+ __asan_report_load8
@ stub -version=0xA00+ __asan_report_loadN
@ stub -version=0xA00+ __asan_report_store1
@ stub -version=0xA00+ __asan_report_store16
@ stub -version=0xA00+ __asan_report_store2
@ stub -version=0xA00+ __asan_report_store4
@ stub -version=0xA00+ __asan_report_store8
@ stub -version=0xA00+ __asan_report_storeN
@ stub -version=0xA00+ __asan_set_shadow_00
@ stub -version=0xA00+ __asan_set_shadow_f8
@ stub -version=0xA00+ __asan_store1
@ stub -version=0xA00+ __asan_store16
@ stub -version=0xA00+ __asan_store16_volatile
@ stub -version=0xA00+ __asan_store1_volatile
@ stub -version=0xA00+ __asan_store2
@ stub -version=0xA00+ __asan_store2_volatile
@ stub -version=0xA00+ __asan_store4
@ stub -version=0xA00+ __asan_store4_volatile
@ stub -version=0xA00+ __asan_store8
@ stub -version=0xA00+ __asan_store8_volatile
@ stub -version=0xA00+ __asan_storeN
@ stub -version=0xA00+ __asan_storeN_volatile
@ stub -version=0xA00+ __asan_wrap_memchr
@ stub -version=0xA00+ __asan_wrap_memcmp
@ stub -version=0xA00+ __asan_wrap_strcat
@ stub -version=0xA00+ __asan_wrap_strcmp
@ stub -version=0xA00+ __asan_wrap_strcpy
@ stub -version=0xA00+ __asan_wrap_strlen
@ stub -version=0xA00+ __asan_wrap_strncmp
@ stub -version=0xA00+ __asan_wrap_strncpy
@ stub -version=0xA00+ __asan_wrap_wcslen
@ stub -version=0xA00+ __misaligned_access
@ stub -version=0xA00+ _atoi64
@ stub -version=0xA00+ _finite
@ stub -version=0xA00+ _i64toa_s
@ stub -version=0xA00+ _i64tow_s
@ stub -version=0xA00+ _itoa_s
@ stub -version=0xA00+ _itow_s
@ stub -version=0xA00+ _ltoa_s
@ stub -version=0xA00+ _ltow_s
@ stub -version=0xA00+ _makepath_s
@ stub -version=0xA00+ _snprintf_s
@ stub -version=0xA00+ _snscanf_s
@ stub -version=0xA00+ _snwprintf_s
@ stub -version=0xA00+ _snwscanf_s
@ stub -version=0xA00+ _splitpath_s
@ stub -version=0xA00+ _strnset_s
@ stub -version=0xA00+ _strset_s
@ stub -version=0xA00+ _strtoui64
@ stub -version=0xA00+ _ui64toa_s
@ stub -version=0xA00+ _ui64tow_s
@ stub -version=0xA00+ _ultoa_s
@ stub -version=0xA00+ _ultow_s
@ stub -version=0xA00+ _vsnprintf_s
@ stub -version=0xA00+ _vsnwprintf_s
@ stub -version=0xA00+ _vswprintf
@ stub -version=0xA00+ _wcslwr_s
@ stub -version=0xA00+ _wcsnset_s
@ stub -version=0xA00+ _wcsset_s
@ stub -version=0xA00+ _wmakepath_s
@ stub -version=0xA00+ _wsplitpath_s
@ stub -version=0xA00+ _wtoi
@ stub -version=0xA00+ _wtol
@ stub -version=0xA00+ bsearch
@ stub -version=0xA00+ bsearch_s
@ stub -version=0xA00+ iswalnum
@ stub -version=0xA00+ iswdigit
@ stub -version=0xA00+ iswspace
@ stub -version=0xA00+ memcpy_s
@ stub -version=0xA00+ memmove_s
@ stub -version=0xA00+ qsort_s
@ stub -version=0xA00+ sprintf_s
@ stub -version=0xA00+ sqrt
@ stub -version=0xA00+ sqrtf
@ stub -version=0xA00+ sscanf_s
@ stub -version=0xA00+ strcat_s
@ stub -version=0xA00+ strcpy_s
@ stub -version=0xA00+ strncat_s
@ stub -version=0xA00+ strncpy_s
@ stub -version=0xA00+ strnlen
@ stub -version=0xA00+ strtok_s
@ stub -version=0xA00+ swprintf_s
@ stub -version=0xA00+ swscanf_s
@ stub -version=0xA00+ vsprintf_s
@ stub -version=0xA00+ vswprintf_s
@ stub -version=0xA00+ wcscat_s
@ stub -version=0xA00+ wcscpy_s
@ stub -version=0xA00+ wcsncat_s
@ stub -version=0xA00+ wcsncpy_s
@ stub -version=0xA00+ wcsnlen
@ stub -version=0xA00+ wcstoul
@ stub -version=0xA00+ x86BiosAllocateBuffer
@ stub -version=0xA00+ x86BiosCall
@ stub -version=0xA00+ x86BiosFreeBuffer
@ stub -version=0xA00+ x86BiosReadMemory
@ stub -version=0xA00+ x86BiosWriteMemory
# Data exports (need an `extern`, not a code stub):
#   CmKeyObjectType
#   ExActivationObjectType
#   ExCompositionObjectType
#   ExCoreMessagingObjectType
#   ExRawInputManagerObjectType
#   ExTimerObjectType
#   IoCompletionObjectType
#   KdComPortInUse
#   KdEventLoggingEnabled
#   KdHvComPortInUse
#   KeDynamicPartitioningSupported
#   KeLastBranchMSR
#   MmBadPointer
#   NtBuildGUID
#   NtBuildLab
#   POGOBuffer
#   PsLoadedModuleList
#   PsLoadedModuleResource
#   PsPartitionType
#   PsSiloContextNonPagedType
#   PsSiloContextPagedType
#   PsUILanguageComitted
#   SeILSigningPolicyPtr
#   SeSystemDefaultSd
#   TmEnlistmentObjectType
#   TmResourceManagerObjectType
#   TmTransactionManagerObjectType
#   TmTransactionObjectType
#   psMUITest
# END win10-surface stubs
