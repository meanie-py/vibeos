@ fastcall -arch=arm,arm64 ExAcquireFastMutex(ptr)
@ fastcall -arch=arm,arm64 ExReleaseFastMutex(ptr)
@ fastcall -arch=arm,arm64 ExTryToAcquireFastMutex(ptr)
@ fastcall -arch=i386 ExAcquireFastMutex(ptr) ntoskrnl.ExiAcquireFastMutex
@ fastcall -arch=i386 ExReleaseFastMutex(ptr) ntoskrnl.ExiReleaseFastMutex
@ fastcall -arch=i386 ExTryToAcquireFastMutex(ptr) ntoskrnl.ExiTryToAcquireFastMutex
@ stdcall HalAcquireDisplayOwnership(ptr)
@ stdcall HalAdjustResourceList(ptr)
@ stdcall HalAllProcessorsStarted()
@ stdcall HalAllocateAdapterChannel(ptr ptr long ptr)
@ stdcall HalAllocateCommonBuffer(ptr long ptr long)
@ stdcall HalAllocateCrashDumpRegisters(ptr ptr)
@ stdcall HalAssignSlotResources(ptr ptr ptr ptr long long long ptr)
@ stdcall -arch=i386,arm,arm64 HalBeginSystemInterrupt(long long ptr)
@ stdcall HalCalibratePerformanceCounter(ptr long long)
;@ stdcall -arch=x86_64 HalCallBios()
@ fastcall HalClearSoftwareInterrupt(long)
@ stdcall -arch=arm64 HalClockInterruptArm64(ptr)
@ stdcall -arch=arm64 HalDumpGicArm64()
@ stdcall -arch=arm64 HalSelfTestGicSpiArm64(long)
@ stdcall -arch=arm64 HalpEnterInterruptIrql(long)
@ stdcall -arch=arm64 HalpExitInterruptIrql(long)
@ stdcall HalVibeosDmaAttach(long)
@ stdcall HalVibeosMessageCapacity()
@ stdcall HalVibeosMessageAllocate(long ptr ptr ptr)
@ stdcall HalVibeosMessageAllocateBlock(ptr ptr ptr ptr)
@ stdcall HalVibeosMessageRelease(long)
@ stdcall HalVibeosMessageTarget(long long ptr ptr)
@ stdcall -arch=arm64 HalVibeosQuiesceIrqlTripwire()
@ stdcall HalDisableSystemInterrupt(long long)
@ stdcall HalDisplayString(str)
@ stdcall HalEnableSystemInterrupt(long long long)
@ stdcall -arch=i386,arm,arm64 HalEndSystemInterrupt(long long)
@ stdcall HalFlushCommonBuffer(long long long long long)
@ stdcall HalFreeCommonBuffer(ptr long long long ptr long)
@ stdcall HalGetAdapter(ptr ptr)
@ stdcall HalGetBusData(long long long ptr long)
@ stdcall HalGetBusDataByOffset(long long long ptr long long)
@ stdcall HalGetEnvironmentVariable(str long str)
@ fastcall -arch=arm HalGetInterruptSource()
@ stdcall HalGetInterruptVector(long long long long ptr ptr)
;@ stdcall -arch=x86_64 HalHandleMcheck()
@ stdcall -arch=i386,x86_64 HalHandleNMI(ptr)
@ stdcall HalInitSystem(long ptr)
@ stdcall -arch=arm64 HalInitializeClockArm64(ptr ptr)
@ stdcall HalInitializeProcessor(long ptr)
;@ stdcall -arch=x86_64 HalIsHyperThreadingEnabled()
@ stdcall HalMakeBeep(long)
@ stdcall HalProcessorIdle()
@ stdcall HalQueryDisplayParameters(ptr ptr ptr ptr)
@ stdcall HalQueryRealTimeClock(ptr)
@ stdcall HalReadDmaCounter(ptr)
@ stdcall HalReportResourceUsage()
@ stdcall HalRequestIpi(long)
@ fastcall HalRequestSoftwareInterrupt(long)
@ stdcall HalReturnToFirmware(long)
@ stdcall -arch=x86_64 HalSendNMI(int64)
@ stdcall -arch=x86_64 HalSendSoftwareInterrupt(int64 long)
@ stdcall HalSetBusData(long long long ptr long)
@ stdcall HalSetBusDataByOffset(long long long ptr long long)
@ stdcall HalSetDisplayParameters(long long)
@ stdcall HalSetEnvironmentVariable(str str)
@ stdcall HalSetProfileInterval(long)
@ stdcall HalSetRealTimeClock(ptr)
@ stdcall HalSetTimeIncrement(long)
@ stdcall HalStartNextProcessor(ptr ptr)
@ stdcall HalStartProfileInterrupt(long)
@ stdcall HalStopProfileInterrupt(long)
@ fastcall -arch=arm HalSweepIcache()
@ fastcall -arch=arm HalSweepDcache()
@ fastcall HalSystemVectorDispatchEntry(long long long)
@ stdcall HalTranslateBusAddress(long long long long ptr ptr)
@ stdcall -arch=i386,x86_64 IoAssignDriveLetters(ptr str ptr ptr) HalpAssignDriveLetters
@ stdcall IoFlushAdapterBuffers(ptr ptr ptr ptr long long)
@ stdcall IoFreeAdapterChannel(ptr)
@ stdcall IoFreeMapRegisters(ptr ptr long)
@ stdcall IoMapTransfer(ptr ptr ptr ptr ptr long)
@ stdcall -arch=i386,x86_64 IoReadPartitionTable(ptr long long ptr) HalpReadPartitionTable
@ stdcall -arch=i386,x86_64 IoSetPartitionInformation(ptr long long long) HalpSetPartitionInformation
@ stdcall -arch=i386,x86_64 IoWritePartitionTable(ptr long long long ptr) HalpWritePartitionTable
@ extern KdComPortInUse
@ fastcall -arch=i386,arm,arm64 KeAcquireInStackQueuedSpinLock(ptr ptr)
@ fastcall -arch=i386,arm,arm64 KeAcquireInStackQueuedSpinLockRaiseToSynch(ptr ptr)
@ fastcall -arch=i386,arm,arm64 KeAcquireQueuedSpinLock(ptr)
@ fastcall -arch=i386,arm,arm64 KeAcquireQueuedSpinLockRaiseToSynch(ptr)
@ stdcall -arch=i386,arm,arm64 KeAcquireSpinLock(ptr ptr)
@ fastcall -arch=i386,arm,arm64 KeAcquireSpinLockRaiseToSynch(ptr)
@ stdcall KeFlushWriteBuffer()
@ stdcall -arch=i386,arm,arm64 KeGetCurrentIrql()
@ stdcall -arch=i386,arm,arm64 KeLowerIrql(long)
@ stdcall KeQueryPerformanceCounter(ptr)
@ stdcall -arch=i386,arm,arm64 KeRaiseIrql(long ptr)
@ stdcall -arch=i386,arm,arm64 KeRaiseIrqlToDpcLevel()
@ stdcall -arch=i386,arm,arm64 KeRaiseIrqlToSynchLevel()
@ fastcall -arch=i386,arm,arm64 KeReleaseInStackQueuedSpinLock(ptr)
@ fastcall -arch=i386,arm,arm64 KeReleaseQueuedSpinLock(ptr long)
@ stdcall -arch=i386,arm,arm64 KeReleaseSpinLock(ptr long)
@ stdcall KeStallExecutionProcessor(long)
@ fastcall -arch=i386,arm,arm64 KeTryToAcquireQueuedSpinLock(long ptr)
@ fastcall -arch=i386,arm,arm64 KeTryToAcquireQueuedSpinLockRaiseToSynch(long ptr)
@ fastcall -arch=i386,arm,arm64 KfAcquireSpinLock(ptr)
@ fastcall -arch=i386,arm,arm64 KfLowerIrql(long)
@ fastcall -arch=i386,arm,arm64 KfRaiseIrql(long)
@ fastcall -arch=i386,arm,arm64 KfReleaseSpinLock(ptr long)
@ stdcall -arch=i386,arm,arm64 READ_PORT_BUFFER_UCHAR(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 READ_PORT_BUFFER_ULONG(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 READ_PORT_BUFFER_USHORT(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 READ_PORT_UCHAR(ptr)
@ stdcall -arch=i386,arm,arm64 READ_PORT_ULONG(ptr)
@ stdcall -arch=i386,arm,arm64 READ_PORT_USHORT(ptr)
@ stdcall -arch=i386,arm,arm64 WRITE_PORT_BUFFER_UCHAR(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_PORT_BUFFER_ULONG(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_PORT_BUFFER_USHORT(ptr ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_PORT_UCHAR(ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_PORT_ULONG(ptr long)
@ stdcall -arch=i386,arm,arm64 WRITE_PORT_USHORT(ptr long)

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ HalAcpiGetTableEx # -> ntoskrnl.HalAcpiGetTableEx
@ stub -version=0xA00+ HalAllocateHardwareCounters # -> ntoskrnl.HalAllocateHardwareCounters
@ stub -version=0xA00+ HalBugCheckSystem # -> ntoskrnl.HalBugCheckSystem
@ stub -version=0xA00+ HalConvertDeviceIdtToIrql # -> ntoskrnl.HalConvertDeviceIdtToIrql
@ stub -version=0xA00+ HalDisableInterrupt # -> ntoskrnl.HalDisableInterrupt
@ stub -version=0xA00+ HalDmaAllocateCrashDumpRegistersEx # -> ntoskrnl.HalDmaAllocateCrashDumpRegistersEx
@ stub -version=0xA00+ HalDmaFreeCrashDumpRegistersEx # -> ntoskrnl.HalDmaFreeCrashDumpRegistersEx
@ stub -version=0xA00+ HalEnableInterrupt # -> ntoskrnl.HalEnableInterrupt
@ stub -version=0xA00+ HalEnumerateEnvironmentVariablesEx # -> ntoskrnl.HalEnumerateEnvironmentVariablesEx
@ stub -version=0xA00+ HalEnumerateProcessors # -> ntoskrnl.HalEnumerateProcessors
@ stub -version=0xA00+ HalFreeHardwareCounters # -> ntoskrnl.HalFreeHardwareCounters
@ stub -version=0xA00+ HalGetEnvironmentVariableEx # -> ntoskrnl.HalGetEnvironmentVariableEx
@ stub -version=0xA00+ HalGetInterruptTargetInformation # -> ntoskrnl.HalGetInterruptTargetInformation
@ stub -version=0xA00+ HalGetMemoryCachingRequirements # -> ntoskrnl.HalGetMemoryCachingRequirements
@ stub -version=0xA00+ HalGetMessageRoutingInfo # -> ntoskrnl.HalGetMessageRoutingInfo
@ stub -version=0xA00+ HalGetProcessorIdByNtNumber # -> ntoskrnl.HalGetProcessorIdByNtNumber
@ stub -version=0xA00+ HalGetVectorInput # -> ntoskrnl.HalGetVectorInput
@ stub -version=0xA00+ HalHandleMcheck # -> ntoskrnl.HalHandleMcheck
@ stub -version=0xA00+ HalInitializeBios # -> ntoskrnl.HalInitializeBios
@ stub -version=0xA00+ HalInitializeOnResume # -> ntoskrnl.HalInitializeOnResume
@ stub -version=0xA00+ HalIsHyperThreadingEnabled # -> ntoskrnl.HalIsHyperThreadingEnabled
@ stub -version=0xA00+ HalPerformEndOfInterrupt # -> ntoskrnl.HalPerformEndOfInterrupt
@ stub -version=0xA00+ HalQueryEnvironmentVariableInfoEx # -> ntoskrnl.HalQueryEnvironmentVariableInfoEx
@ stub -version=0xA00+ HalQueryMaximumProcessorCount # -> ntoskrnl.HalQueryMaximumProcessorCount
@ stub -version=0xA00+ HalRegisterDynamicProcessor # -> ntoskrnl.HalRegisterDynamicProcessor
@ stub -version=0xA00+ HalRegisterErrataCallbacks # -> ntoskrnl.HalRegisterErrataCallbacks
@ stub -version=0xA00+ HalRequestClockInterrupt # -> ntoskrnl.HalRequestClockInterrupt
@ stub -version=0xA00+ HalRequestDeferredRecoveryServiceInterrupt # -> ntoskrnl.HalRequestDeferredRecoveryServiceInterrupt
@ stub -version=0xA00+ HalRequestIpiSpecifyVector # -> ntoskrnl.HalRequestIpiSpecifyVector
@ stub -version=0xA00+ HalSetEnvironmentVariableEx # -> ntoskrnl.HalSetEnvironmentVariableEx
@ stub -version=0xA00+ HalStartDynamicProcessor # -> ntoskrnl.HalStartDynamicProcessor
@ stub -version=0xA00+ HalWheaUpdateCmciPolicy # -> ntoskrnl.HalWheaUpdateCmciPolicy
@ stub -version=0xA00+ KdHvComPortInUse # -> ntoskrnl.KdHvComPortInUse
@ stub -version=0xA00+ x86BiosAllocateBuffer # -> ntoskrnl.x86BiosAllocateBuffer
@ stub -version=0xA00+ x86BiosCall # -> ntoskrnl.x86BiosCall
@ stub -version=0xA00+ x86BiosFreeBuffer # -> ntoskrnl.x86BiosFreeBuffer
@ stub -version=0xA00+ x86BiosReadMemory # -> ntoskrnl.x86BiosReadMemory
@ stub -version=0xA00+ x86BiosWriteMemory # -> ntoskrnl.x86BiosWriteMemory
# END win10-surface stubs
