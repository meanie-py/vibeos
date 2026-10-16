
 @ stdcall ArcFilterDprIndicateReceive(ptr ptr ptr long)
 @ stdcall ArcFilterDprIndicateReceiveComplete(ptr)
 @ stdcall EthFilterDprIndicateReceive(ptr ptr ptr ptr long ptr long long)
 @ stdcall EthFilterDprIndicateReceiveComplete(ptr)
 @ stdcall FddiFilterDprIndicateReceive(ptr ptr ptr long ptr long ptr long long)
 @ stdcall FddiFilterDprIndicateReceiveComplete(ptr)
 @ stdcall NDIS_BUFFER_TO_SPAN_PAGES(ptr)
 @ stdcall NdisAcquireReadWriteLock(ptr long ptr)
 @ stdcall NdisAcquireSpinLock(ptr)
 @ stdcall NdisAdjustBufferLength(ptr long)
 @ stdcall NdisAllocateBuffer(ptr ptr ptr ptr long)
 @ stdcall NdisAllocateBufferPool(ptr ptr long)
 @ stdcall NdisAllocateDmaChannel(ptr ptr ptr ptr long)
 @ stdcall NdisAllocateMemory(ptr long long double)
 @ stdcall NdisAllocateFromBlockPool(ptr)
 @ stdcall NdisAllocateMemoryWithTag(ptr long long)
 @ stdcall NdisAllocatePacket(ptr ptr ptr)
 @ stdcall NdisAllocatePacketPool(ptr ptr long long)
 @ stdcall NdisAllocatePacketPoolEx(ptr ptr long long long)
 @ stdcall NdisAllocateSharedMemory(ptr long long ptr ptr)
 @ stdcall NdisAllocateSpinLock(ptr)
 @ stdcall NdisAnsiStringToUnicodeString(ptr ptr)
 @ stdcall NdisBufferLength(ptr)
 @ stdcall NdisBufferVirtualAddress(ptr)
 @ stdcall NdisCancelSendPackets(ptr ptr)
 @ stdcall NdisCancelTimer(ptr ptr)
 @ stdcall NdisClAddParty(ptr ptr ptr ptr)
 @ stdcall NdisClCloseAddressFamily(ptr)
 @ stdcall NdisClCloseCall(ptr ptr ptr long)
 @ stdcall NdisClDeregisterSap(ptr)
 @ stdcall NdisClDropParty(ptr ptr long)
 @ stdcall NdisClGetProtocolVcContextFromTapiCallId(double ptr)
 @ stdcall NdisClIncomingCallComplete(long ptr ptr)
 @ stdcall NdisClMakeCall(ptr ptr ptr ptr)
 @ stdcall NdisClModifyCallQoS(ptr ptr)
 @ stdcall NdisClOpenAddressFamily(ptr ptr ptr ptr long ptr)
 @ stdcall NdisClRegisterSap(ptr ptr ptr ptr)
 @ stdcall NdisCloseAdapter(ptr ptr)
 @ stdcall NdisCloseConfiguration(ptr)
 @ stdcall NdisCloseFile(ptr)
 @ stdcall NdisCmActivateVc(ptr ptr)
 @ stdcall NdisCmAddPartyComplete(long ptr ptr ptr)
 @ stdcall NdisCmCloseAddressFamilyComplete(long ptr)
 @ stdcall NdisCmCloseCallComplete(long ptr ptr)
 @ stdcall NdisCmDeactivateVc(ptr)
 @ stdcall NdisCmDeregisterSapComplete(long ptr)
 @ stdcall NdisCmDispatchCallConnected(ptr)
 @ stdcall NdisCmDispatchIncomingCall(ptr ptr ptr)
 @ stdcall NdisCmDispatchIncomingCallQoSChange(ptr ptr)
 @ stdcall NdisCmDispatchIncomingCloseCall(long ptr ptr long)
 @ stdcall NdisCmDispatchIncomingDropParty(ptr ptr ptr long)
 @ stdcall NdisCmDropPartyComplete(long ptr)
 @ stdcall NdisCmMakeCallComplete(long ptr ptr ptr ptr)
 @ stdcall NdisCmModifyCallQoSComplete(long ptr ptr)
 @ stdcall NdisCmOpenAddressFamilyComplete(long ptr ptr)
 @ stdcall NdisCmRegisterAddressFamily(ptr ptr ptr long)
 @ stdcall NdisCmRegisterSapComplete(long ptr ptr)
 @ stdcall NdisCoAssignInstanceName(ptr ptr ptr)
 @ stdcall NdisCoCreateVc(ptr ptr ptr ptr)
 @ stdcall NdisCoDeleteVc(ptr)
 @ stdcall NdisCoGetTapiCallId(ptr ptr)
 @ stdcall NdisCoRequest(ptr ptr ptr ptr ptr)
 @ stdcall NdisCoRequestComplete(long ptr ptr ptr ptr)
 @ stdcall NdisCoSendPackets(ptr ptr long)
 @ stdcall NdisCompareAnsiString(ptr ptr long)
 @ stdcall NdisCompareUnicodeString(ptr ptr long)
 @ stdcall NdisCompleteBindAdapter(ptr long long)
 @ stdcall NdisCompleteCloseAdapter(ptr long)
 @ stdcall NdisCompleteDmaTransfer(ptr ptr ptr long long long)
 @ stdcall NdisCompleteOpenAdapter(ptr long long)
 @ stdcall NdisCompletePnPEvent(long ptr ptr)
 @ stdcall NdisCompleteQueryStatistics(ptr ptr long)
 @ stdcall NdisCompleteUnbindAdapter(ptr long)
 @ stdcall NdisConvertStringToAtmAddress(ptr ptr ptr)
 @ stdcall NdisCopyBuffer(ptr ptr ptr ptr long long)
 @ stdcall NdisCopyFromPacketToPacket(ptr long long ptr long ptr)
 @ stdcall NdisCopyFromPacketToPacketSafe(ptr long long ptr long ptr long)
 @ stdcall NdisCreateBlockPool(long long long ptr)
 @ stdcall NdisDeregisterAdapter(ptr)
 @ stdcall NdisDeregisterAdapterShutdownHandler(ptr)
 @ stdcall NdisDeregisterMac(ptr ptr)
 @ stdcall NdisDeregisterProtocol(ptr ptr)
 @ stdcall NdisDeregisterTdiCallBack()
 @ stdcall NdisDestroyBlockPool(ptr)
 @ stdcall NdisDprAcquireSpinLock(ptr)
 @ stdcall NdisDprAllocatePacket(ptr ptr ptr)
 @ stdcall NdisDprAllocatePacketNonInterlocked(ptr ptr ptr)
 @ stdcall NdisDprFreePacket(ptr)
 @ stdcall NdisDprFreePacketNonInterlocked(ptr)
 @ stdcall NdisDprReleaseSpinLock(ptr)
 @ stdcall NdisEqualString(ptr ptr long)
 @ stdcall NdisFreeBuffer(ptr)
 @ stdcall NdisFreeBufferPool(ptr)
 @ stdcall NdisFreeDmaChannel(ptr)
 @ stdcall NdisFreeToBlockPool(ptr)
 @ stdcall NdisFreeMemory(ptr long long)
 @ stdcall NdisFreePacket(ptr)
 @ stdcall NdisFreePacketPool(ptr)
 @ stdcall NdisFreeSharedMemory(ptr long long ptr double)
 @ stdcall NdisFreeSpinLock(ptr)
 @ stdcall NdisGeneratePartialCancelId()
 @ stdcall NdisGetBufferPhysicalArraySize(ptr ptr)
 @ stdcall NdisGetCurrentProcessorCounts(ptr ptr ptr)
 @ stdcall NdisGetCurrentProcessorCpuUsage(ptr)
 @ stdcall NdisGetCurrentSystemTime(ptr)
 @ stdcall NdisGetDriverHandle(ptr ptr)
 @ stdcall NdisGetFirstBufferFromPacket(ptr ptr ptr ptr ptr)
 @ stdcall NdisGetFirstBufferFromPacketSafe(ptr ptr ptr ptr ptr long)
 @ stdcall NdisGetPacketCancelId(ptr)
 @ stdcall NdisGetPoolFromPacket(ptr)
 @ stdcall NdisGetReceivedPacket(ptr ptr)
 @ stdcall NdisGetRoutineAddress(ptr)
 @ stdcall NdisGetSharedDataAlignment()
 @ stdcall NdisGetSystemUpTime(ptr)
 @ stdcall NdisGetVersion()
 @ stdcall NdisIMAssociateMiniport(ptr ptr)
 @ stdcall NdisIMCancelInitializeDeviceInstance(ptr ptr)
 @ stdcall NdisIMCopySendCompletePerPacketInfo(ptr ptr)
 @ stdcall NdisIMCopySendPerPacketInfo(ptr ptr)
 @ stdcall NdisIMDeInitializeDeviceInstance(ptr)
 @ stdcall NdisIMDeregisterLayeredMiniport(ptr)
 @ stdcall NdisIMGetBindingContext(ptr)
 @ stdcall NdisIMGetCurrentPacketStack(ptr ptr)
 @ stdcall NdisIMGetDeviceContext(ptr)
 @ stdcall NdisIMInitializeDeviceInstance(ptr ptr)
 @ stdcall NdisIMInitializeDeviceInstanceEx(ptr ptr ptr)
 @ stdcall NdisIMNotifyPnPEvent(ptr ptr)
 @ stdcall NdisImmediateReadPciSlotInformation(ptr long long ptr long)
 @ stdcall NdisImmediateReadPortUchar(ptr long ptr)
 @ stdcall NdisImmediateReadPortUlong(ptr long ptr)
 @ stdcall NdisImmediateReadPortUshort(ptr long ptr)
 @ stdcall NdisImmediateReadSharedMemory(ptr long ptr long)
 @ stdcall NdisImmediateWritePciSlotInformation(ptr long long ptr long)
 @ stdcall NdisImmediateWritePortUchar(ptr long long)
 @ stdcall NdisImmediateWritePortUlong(ptr long long)
 @ stdcall NdisImmediateWritePortUshort(ptr long long)
 @ stdcall NdisImmediateWriteSharedMemory(ptr long ptr long)
 @ stdcall NdisIMQueueMiniportCallback(ptr ptr ptr)
 @ stdcall NdisIMRegisterLayeredMiniport(ptr ptr long ptr)
 @ stdcall NdisIMRevertBack(ptr ptr)
 @ stdcall NdisIMSwitchToMiniport(ptr ptr)
 @ stdcall NdisInitAnsiString(ptr ptr)
 @ stdcall NdisInitializeEvent(ptr)
 @ stdcall NdisInitializeReadWriteLock(ptr)
 @ stdcall NdisInitializeString(ptr ptr)
 @ stdcall NdisInitializeTimer(ptr ptr ptr)
 @ stdcall NdisInitializeWrapper(ptr ptr ptr ptr)
 @ stdcall NdisInitUnicodeString(ptr ptr)
 @ stdcall NdisInterlockedAddLargeInteger(ptr double ptr)
 @ stdcall NdisInterlockedAddUlong(ptr long ptr)
 @ stdcall NdisInterlockedDecrement(ptr)
 @ stdcall NdisInterlockedIncrement(ptr)
 @ stdcall NdisInterlockedInsertHeadList(ptr ptr ptr)
 @ stdcall NdisInterlockedInsertTailList(ptr ptr ptr)
 @ stdcall NdisInterlockedPopEntrySList(ptr ptr)
 @ stdcall NdisInterlockedPushEntrySList(ptr ptr ptr)
 @ stdcall NdisInterlockedRemoveHeadList(ptr ptr)
 @ stdcall NdisMAllocateMapRegisters(ptr long long long long)
 @ stdcall NdisMAllocateSharedMemory(ptr long long ptr ptr)
 @ stdcall NdisMAllocateSharedMemoryAsync(ptr long long ptr)
 @ stdcall NdisMapFile(ptr ptr ptr)
 @ stdcall NdisMapIoSpace(ptr ptr ptr double long)
# @ stdcall NdisMatchPdoWithPacket ?
 @ stdcall NdisMCancelTimer(ptr ptr)
 @ stdcall NdisMCloseLog(ptr)
 @ stdcall NdisMCmActivateVc(ptr ptr)
 @ stdcall NdisMCmCreateVc(ptr ptr ptr ptr)
 @ stdcall NdisMCmDeactivateVc(ptr)
 @ stdcall NdisMCmDeleteVc(ptr)
 @ stdcall NdisMCmRegisterAddressFamily(ptr ptr ptr long)
 @ stdcall NdisMCmRequest(ptr ptr ptr ptr)
 @ stdcall NdisMCoActivateVcComplete(long ptr ptr)
 @ stdcall NdisMCoDeactivateVcComplete(long ptr)
 @ stdcall NdisMCoIndicateReceivePacket(ptr ptr long)
 @ stdcall NdisMCoIndicateStatus(ptr ptr long ptr long)
 @ stdcall NdisMCompleteBufferPhysicalMapping(ptr ptr long)
 @ stdcall NdisMCompleteDmaTransfer(ptr ptr ptr long long long)
 @ stdcall NdisMCoReceiveComplete(ptr)
 @ stdcall NdisMCoRequestComplete(long ptr ptr)
 @ stdcall NdisMCoSendComplete(ptr ptr ptr)
 @ stdcall NdisMCreateLog(ptr long ptr)
 @ stdcall NdisMDeregisterAdapterShutdownHandler(ptr)
 @ stdcall NdisMDeregisterDevice(ptr)
 @ stdcall NdisMDeregisterDmaChannel(ptr)
 @ stdcall NdisMDeregisterInterrupt(ptr)
 @ stdcall NdisMDeregisterIoPortRange(ptr long long ptr)
 @ stdcall NdisMFlushLog(ptr)
 @ stdcall NdisMFreeMapRegisters(ptr)
 @ stdcall NdisMFreeSharedMemory(ptr long long ptr double)
 @ stdcall NdisMGetDeviceProperty(ptr ptr ptr ptr ptr ptr)
 @ stdcall NdisMGetDmaAlignment(ptr)
 @ stdcall NdisMIndicateStatus(ptr long ptr long)
 @ stdcall NdisMIndicateStatusComplete(ptr)
 @ stdcall NdisMInitializeScatterGatherDma(ptr long long)
 @ stdcall NdisMInitializeTimer(ptr ptr ptr ptr)
 @ stdcall NdisMMapIoSpace(ptr ptr double long)
 @ stdcall NdisMPciAssignResources(ptr long ptr)
 @ stdcall NdisMPromoteMiniport(ptr)
 @ stdcall NdisMQueryAdapterInstanceName(ptr ptr)
 @ stdcall NdisMQueryAdapterResources(ptr ptr ptr ptr)
 @ stdcall NdisMQueryInformationComplete(ptr long)
 @ stdcall NdisMReadDmaCounter(ptr)
 @ stdcall NdisMRegisterAdapterShutdownHandler(ptr ptr ptr)
 @ stdcall NdisMRegisterDevice(ptr ptr ptr ptr ptr ptr)
 @ stdcall NdisMRegisterDmaChannel(ptr ptr long long ptr long)
 @ stdcall NdisMRegisterInterrupt(ptr ptr long long long long long)
 @ stdcall NdisMRegisterIoPortRange(ptr ptr long long)
 @ stdcall NdisMRegisterMiniport(ptr ptr long)
 @ stdcall NdisMRegisterUnloadHandler(ptr ptr)
 @ stdcall NdisMRemoveMiniport(ptr)
 @ stdcall NdisMResetComplete(ptr long long)
 @ stdcall NdisMSendComplete(ptr ptr long)
 @ stdcall NdisMSendResourcesAvailable(ptr)
 @ stdcall NdisMSetAttributes(ptr ptr long long)
 @ stdcall NdisMSetAttributesEx(ptr ptr long long long)
 @ stdcall NdisMSetInformationComplete(ptr long)
 @ stdcall NdisMSetMiniportSecondary(ptr ptr)
 @ stdcall NdisMSetPeriodicTimer(ptr long)
 @ stdcall NdisMSetTimer(ptr long)
 @ stdcall NdisMSetupDmaTransfer(ptr ptr ptr long long long)
 @ stdcall NdisMSleep(long)
 @ stdcall NdisMStartBufferPhysicalMapping(ptr ptr long long ptr ptr)
 @ stdcall NdisMSynchronizeWithInterrupt(ptr ptr ptr)
 @ stdcall NdisMTransferDataComplete(ptr ptr long long)
 @ stdcall NdisMUnmapIoSpace(ptr ptr long)
 @ stdcall NdisMWanIndicateReceive(ptr ptr ptr ptr long)
 @ stdcall NdisMWanIndicateReceiveComplete(ptr ptr)
 @ stdcall NdisMWanSendComplete(ptr ptr long)
 @ stdcall NdisMWriteLogData(ptr ptr long)
 @ stdcall NdisOpenAdapter(ptr ptr ptr ptr ptr long ptr ptr ptr long ptr)
 @ stdcall NdisOpenConfiguration(ptr ptr ptr)
 @ stdcall NdisOpenConfigurationKeyByIndex(ptr ptr long ptr ptr)
 @ stdcall NdisOpenConfigurationKeyByName(ptr ptr ptr ptr)
 @ stdcall NdisOpenFile(ptr ptr ptr ptr double)
 @ stdcall NdisOpenProtocolConfiguration(ptr ptr ptr)
 @ stdcall NdisOverrideBusNumber(ptr ptr long)
 @ stdcall NdisPacketPoolUsage(ptr)
 @ stdcall NdisPacketSize(long)
 @ stdcall NdisPciAssignResources(ptr ptr ptr long ptr)
 @ stdcall NdisQueryAdapterInstanceName(ptr ptr)
 @ stdcall NdisQueryBindInstanceName(ptr ptr)
 @ stdcall NdisQueryBuffer(ptr ptr ptr)
 @ stdcall NdisQueryBufferOffset(ptr ptr ptr)
 @ stdcall NdisQueryBufferSafe(ptr ptr ptr long)
 @ stdcall NdisQueryMapRegisterCount(long ptr)
 @ stdcall NdisQueryPendingIOCount(ptr ptr)
 @ stdcall NdisReadConfiguration(ptr ptr ptr ptr long)
 @ stdcall NdisReadEisaSlotInformation(ptr ptr ptr ptr)
 @ stdcall NdisReadEisaSlotInformationEx(ptr ptr ptr ptr ptr)
 @ stdcall NdisReadMcaPosInformation(ptr ptr ptr ptr)
 @ stdcall NdisReadNetworkAddress(ptr ptr ptr ptr)
 @ stdcall NdisReadPciSlotInformation(ptr long long ptr long)
 @ stdcall NdisReadPcmciaAttributeMemory(ptr long ptr long)
 @ stdcall NdisReEnumerateProtocolBindings(ptr)
 @ stdcall NdisRegisterAdapter(ptr ptr ptr ptr ptr ptr)
 @ stdcall NdisRegisterAdapterShutdownHandler(ptr ptr ptr)
 @ stdcall NdisRegisterProtocol(ptr ptr ptr long)
 @ stdcall NdisRegisterTdiCallBack(ptr ptr)
 @ stdcall NdisReleaseAdapterResources(ptr)
 @ stdcall NdisReleaseReadWriteLock(ptr ptr)
 @ stdcall NdisReleaseSpinLock(ptr)
 @ stdcall NdisRequest(ptr ptr ptr)
 @ stdcall NdisReset(ptr ptr)
 @ stdcall NdisResetEvent(ptr)
 @ stdcall NdisReturnPackets(ptr long)
 @ stdcall NdisSend(ptr ptr ptr)
 @ stdcall NdisSendPackets(ptr ptr long)
 @ stdcall NdisSetEvent(ptr)
 @ stdcall NdisSetPacketCancelId(ptr ptr)
 @ stdcall NdisSetPacketPoolProtocolId(ptr long)
 @ stdcall NdisSetPacketStatus(ptr long ptr long)
 @ stdcall NdisSetProtocolFilter(ptr ptr ptr ptr long long long ptr)
 @ stdcall NdisSetTimer(ptr long)
 @ stdcall NdisSetTimerEx(ptr long ptr)
 @ stdcall NdisSetupDmaTransfer(ptr ptr ptr long long long)
 @ stdcall NdisSystemProcessorCount()
 @ stdcall NdisTerminateWrapper(ptr ptr)
 @ stdcall NdisTransferData(ptr ptr ptr long long ptr ptr)
 @ stdcall NdisUnchainBufferAtBack(ptr ptr)
 @ stdcall NdisUnchainBufferAtFront(ptr ptr)
 @ stdcall NdisUnicodeStringToAnsiString(ptr ptr)
 @ stdcall NdisUnmapFile(ptr)
 @ stdcall NdisUpcaseUnicodeString(ptr ptr)
 @ stdcall NdisUpdateSharedMemory(ptr long ptr double)
 @ stdcall NdisWaitEvent(ptr long)
 @ stdcall NdisWriteConfiguration(ptr ptr ptr ptr)
 @ cdecl NdisWriteErrorLogEntry(ptr long long)
 @ stdcall NdisWriteEventLogEntry(ptr long long long ptr long ptr)
 @ stdcall NdisWritePciSlotInformation(ptr long long ptr long)
 @ stdcall NdisWritePcmciaAttributeMemory(ptr long ptr long)
 @ stdcall TrFilterDprIndicateReceive(ptr ptr ptr long ptr long long)
 @ stdcall TrFilterDprIndicateReceiveComplete(ptr)
 @ stdcall NdisScheduleWorkItem(ptr)

# NDIS 6 miniport surface (sdk/lib/rust/ndis-rust/src/ndis/ndis6.rs)
@ stdcall NdisAcquireRWLockRead(ptr ptr long)
@ stdcall NdisAcquireRWLockWrite(ptr ptr long)
@ stdcall NdisAllocateIoWorkItem(ptr)
@ stdcall NdisAllocateMdl(ptr ptr long)
@ stdcall NdisAllocateMemoryWithTagPriority(ptr long long long)
@ stdcall NdisAllocateNetBuffer(ptr ptr long ptr)
@ stdcall NdisAllocateNetBufferAndNetBufferList(ptr long long ptr long ptr)
@ stdcall NdisAllocateNetBufferList(ptr long long)
@ stdcall NdisAllocateNetBufferListPool(ptr ptr)
@ stdcall NdisAllocateNetBufferPool(ptr ptr)
@ stdcall NdisAllocateRWLock(ptr)
@ stdcall NdisAllocateTimerObject(ptr ptr ptr)
@ stdcall NdisCancelTimerObject(ptr)
@ stdcall NdisDeregisterDeviceEx(ptr)
@ stdcall NdisFreeIoWorkItem(ptr)
@ stdcall NdisFreeMdl(ptr)
@ stdcall NdisFreeNetBuffer(ptr)
@ stdcall NdisFreeNetBufferList(ptr)
@ stdcall NdisFreeNetBufferListPool(ptr)
@ stdcall NdisFreeNetBufferPool(ptr)
@ stdcall NdisFreeRWLock(ptr)
@ stdcall NdisFreeTimerObject(ptr)
@ stdcall NdisGetDeviceReservedExtension(ptr)
@ stdcall NdisMAllocateNetBufferSGList(ptr ptr ptr long ptr long)
@ stdcall NdisMDeregisterInterruptEx(ptr)
@ stdcall NdisMDeregisterMiniportDriver(ptr)
@ stdcall NdisMDeregisterScatterGatherDma(ptr)
@ stdcall NdisMFreeNetBufferSGList(ptr ptr ptr)
@ stdcall NdisMGetBusData(ptr long long ptr long)
@ stdcall NdisMIdleNotificationComplete(ptr)
@ stdcall NdisMIdleNotificationConfirm(ptr long)
@ stdcall NdisMIndicateReceiveNetBufferLists(ptr ptr long long long)
@ stdcall NdisMIndicateStatusEx(ptr ptr)
@ stdcall NdisMNetPnPEvent(ptr ptr)
@ stdcall NdisMOidRequestComplete(ptr ptr long)
@ stdcall NdisMPauseComplete(ptr)
@ stdcall NdisMRegisterInterruptEx(ptr ptr ptr ptr)
@ stdcall NdisMRegisterMiniportDriver(ptr ptr ptr ptr ptr)
@ stdcall NdisMRegisterWdiMiniportDriver(ptr ptr ptr ptr ptr ptr)
@ stdcall NdisMDeregisterWdiMiniportDriver(ptr)
@ stdcall NdisMAllocatePort(ptr ptr)
@ stdcall NdisMFreePort(ptr long)
@ stdcall NdisMRegisterScatterGatherDma(ptr ptr ptr)
@ stdcall NdisMRestartComplete(ptr long)
@ stdcall NdisMSendNetBufferListsComplete(ptr ptr long)
@ stdcall NdisMSetBusData(ptr long long ptr long)
@ stdcall NdisMSetMiniportAttributes(ptr ptr)
@ stdcall NdisMSynchronizeWithInterruptEx(ptr long ptr ptr)
@ stdcall NdisOpenConfigurationEx(ptr ptr)
@ stdcall NdisQueueIoWorkItem(ptr ptr ptr)
@ stdcall NdisRegisterDeviceEx(ptr ptr ptr ptr)
@ stdcall NdisReleaseRWLock(ptr ptr)
@ stdcall NdisSetOptionalHandlers(ptr ptr)
@ stdcall NdisSetTimerObject(ptr int64 long ptr)

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ KLoaderDereferenceModule
@ stub -version=0xA00+ KLoaderQueryDispatchTable
@ stub -version=0xA00+ KLoaderReferenceModule
@ stub -version=0xA00+ KLoaderRegisterModule
@ stub -version=0xA00+ NdisActiveGroupCount
@ stub -version=0xA00+ NdisAdjustNetBufferCurrentMdl
@ stub -version=0xA00+ NdisAdvanceNetBufferDataStart
@ stub -version=0xA00+ NdisAdvanceNetBufferListDataStart
@ stub -version=0xA00+ NdisAllocateCloneNetBufferList
@ stub -version=0xA00+ NdisAllocateCloneOidRequest
@ stub -version=0xA00+ NdisAllocateFragmentNetBufferList
@ stub -version=0xA00+ NdisAllocateGenericObject
@ stub -version=0xA00+ NdisAllocateNetBufferListContext
@ stub -version=0xA00+ NdisAllocateNetBufferMdlAndData
@ stub -version=0xA00+ NdisAllocateReassembledNetBufferList
@ stub -version=0xA00+ NdisAllocateRefCount
@ stub -version=0xA00+ NdisBuildScatterGatherList
@ stub -version=0xA00+ NdisCancelDirectOidRequest
@ stub -version=0xA00+ NdisCancelOidRequest
@ stub -version=0xA00+ NdisCancelSendNetBufferLists
@ stub -version=0xA00+ NdisClNotifyCloseAddressFamilyComplete
@ stub -version=0xA00+ NdisClOpenAddressFamilyEx
@ stub -version=0xA00+ NdisCloseAdapterEx
@ stub -version=0xA00+ NdisCloseNDKAdapter
@ stub -version=0xA00+ NdisCmNotifyCloseAddressFamily
@ stub -version=0xA00+ NdisCmRegisterAddressFamilyEx
@ stub -version=0xA00+ NdisCoOidRequest
@ stub -version=0xA00+ NdisCoOidRequestComplete
@ stub -version=0xA00+ NdisCoSendNetBufferLists
@ stub -version=0xA00+ NdisCompleteBindAdapterEx
@ stub -version=0xA00+ NdisCompleteNetPnPEvent
@ stub -version=0xA00+ NdisCompleteUnbindAdapterEx
@ stub -version=0xA00+ NdisConvertNdisStatusToNtStatus
@ stub -version=0xA00+ NdisConvertNtStatusToNdisStatus
@ stub -version=0xA00+ NdisCopyFromNetBufferToNetBuffer
@ stub -version=0xA00+ NdisCopyReceiveNetBufferListInfo
@ stub -version=0xA00+ NdisCopySendNetBufferListInfo
@ stub -version=0xA00+ NdisCurrentGroupAndProcessor
@ stub -version=0xA00+ NdisCurrentProcessorIndex
@ stub -version=0xA00+ NdisDereferenceWithTag
@ stub -version=0xA00+ NdisDeregisterPoll
@ stub -version=0xA00+ NdisDeregisterProtocolDriver
@ stub -version=0xA00+ NdisDirectOidRequest
@ stub -version=0xA00+ NdisDllInitialize
@ stub -version=0xA00+ NdisDprAcquireReadWriteLock
@ stub -version=0xA00+ NdisDprReleaseReadWriteLock
@ stub -version=0xA00+ NdisEnumerateFilterModules
@ stub -version=0xA00+ NdisFCancelDirectOidRequest
@ stub -version=0xA00+ NdisFCancelOidRequest
@ stub -version=0xA00+ NdisFCancelSendNetBufferLists
@ stub -version=0xA00+ NdisFDeregisterFilterDriver
@ stub -version=0xA00+ NdisFDevicePnPEventNotify
@ stub -version=0xA00+ NdisFDirectOidRequest
@ stub -version=0xA00+ NdisFDirectOidRequestComplete
@ stub -version=0xA00+ NdisFGetOptionalSwitchHandlers
@ stub -version=0xA00+ NdisFIndicateReceiveNetBufferLists
@ stub -version=0xA00+ NdisFIndicateStatus
@ stub -version=0xA00+ NdisFNetPnPEvent
@ stub -version=0xA00+ NdisFOidRequest
@ stub -version=0xA00+ NdisFOidRequestComplete
@ stub -version=0xA00+ NdisFPauseComplete
@ stub -version=0xA00+ NdisFRegisterFilterDriver
@ stub -version=0xA00+ NdisFRestartComplete
@ stub -version=0xA00+ NdisFRestartFilter
@ stub -version=0xA00+ NdisFRetryAttach
@ stub -version=0xA00+ NdisFReturnNetBufferLists
@ stub -version=0xA00+ NdisFSendNetBufferLists
@ stub -version=0xA00+ NdisFSendNetBufferListsComplete
@ stub -version=0xA00+ NdisFSetAttributes
@ stub -version=0xA00+ NdisFSynchronousOidRequest
@ stub -version=0xA00+ NdisFreeCloneNetBufferList
@ stub -version=0xA00+ NdisFreeCloneOidRequest
@ stub -version=0xA00+ NdisFreeFragmentNetBufferList
@ stub -version=0xA00+ NdisFreeGenericObject
@ stub -version=0xA00+ NdisFreeMemoryWithTag
@ stub -version=0xA00+ NdisFreeMemoryWithTagPriority
@ stub -version=0xA00+ NdisFreeNetBufferListContext
@ stub -version=0xA00+ NdisFreeReassembledNetBufferList
@ stub -version=0xA00+ NdisFreeRefCount
@ stub -version=0xA00+ NdisFreeScatterGatherList
@ stub -version=0xA00+ NdisGetAndReferenceCompartmentJobObject
@ stub -version=0xA00+ NdisGetDataBuffer
@ stub -version=0xA00+ NdisGetHypervisorInfo
@ stub -version=0xA00+ NdisGetJobObjectCompartmentId
@ stub -version=0xA00+ NdisGetNetBufferListProtocolId
@ stub -version=0xA00+ NdisGetPMProtocolOffloadType
@ stub -version=0xA00+ NdisGetPacketFromNetBufferList
@ stub -version=0xA00+ NdisGetPoolFromNetBuffer
@ stub -version=0xA00+ NdisGetPoolFromNetBufferList
@ stub -version=0xA00+ NdisGetProcessObjectCompartmentId
@ stub -version=0xA00+ NdisGetProcessorInformation
@ stub -version=0xA00+ NdisGetProcessorInformationEx
@ stub -version=0xA00+ NdisGetRefCount
@ stub -version=0xA00+ NdisGetRssProcessorInformation
@ stub -version=0xA00+ NdisGetSessionCompartmentId
@ stub -version=0xA00+ NdisGetSessionToCompartmentMappingEpochAndZero
@ stub -version=0xA00+ NdisGetSystemUpTimeEx
@ stub -version=0xA00+ NdisGetThreadObjectCompartmentId
@ stub -version=0xA00+ NdisGetThreadObjectCompartmentScope
@ stub -version=0xA00+ NdisGroupActiveProcessorCount
@ stub -version=0xA00+ NdisGroupActiveProcessorMask
@ stub -version=0xA00+ NdisGroupMaxProcessorCount
@ stub -version=0xA00+ NdisIMVBusDeviceAdd
@ stub -version=0xA00+ NdisIMVBusDeviceRemove
@ stub -version=0xA00+ NdisIfAddIfStackEntry
@ stub -version=0xA00+ NdisIfAllocateNetLuidIndex
@ stub -version=0xA00+ NdisIfAllocateNetLuidIndexEx
@ stub -version=0xA00+ NdisIfDeleteIfStackEntry
@ stub -version=0xA00+ NdisIfDeregisterInterface
@ stub -version=0xA00+ NdisIfDeregisterProvider
@ stub -version=0xA00+ NdisIfFreeNetLuidIndex
@ stub -version=0xA00+ NdisIfGetInterfaceIndexFromNetLuid
@ stub -version=0xA00+ NdisIfGetNetLuidFromInterfaceIndex
@ stub -version=0xA00+ NdisIfQueryBindingIfIndex
@ stub -version=0xA00+ NdisIfRegisterInterface
@ stub -version=0xA00+ NdisIfRegisterProvider
@ stub -version=0xA00+ NdisIfUpdateInterface
@ stub -version=0xA00+ NdisInitiateOffload
@ stub -version=0xA00+ NdisInterlockedAddLargeInterger # -> NTOSKRNL.ExInterlockedAddLargeInteger
@ stub -version=0xA00+ NdisInterlockedPopEntryList
@ stub -version=0xA00+ NdisInterlockedPushEntryList
@ stub -version=0xA00+ NdisInvalidateOffload
@ stub -version=0xA00+ NdisIsStatusIndicationCloneable
@ stub -version=0xA00+ NdisLWMDeregisterMiniportDriver
@ stub -version=0xA00+ NdisLWMInitializeNetworkInterface
@ stub -version=0xA00+ NdisLWMRegisterMiniportDriver
@ stub -version=0xA00+ NdisLWMStartNetworkInterface
@ stub -version=0xA00+ NdisLWMUninitializeNetworkInterface
@ stub -version=0xA00+ NdisMAllocateSharedMemoryAsyncEx
@ stub -version=0xA00+ NdisMCmOidRequest
@ stub -version=0xA00+ NdisMCmRegisterAddressFamilyEx
@ stub -version=0xA00+ NdisMCoIndicateReceiveNetBufferLists
@ stub -version=0xA00+ NdisMCoIndicateStatusEx
@ stub -version=0xA00+ NdisMCoOidRequestComplete
@ stub -version=0xA00+ NdisMCoSendNetBufferListsComplete
@ stub -version=0xA00+ NdisMConfigMSIXTableEntry
@ stub -version=0xA00+ NdisMDirectOidRequestComplete
@ stub -version=0xA00+ NdisMEnableVirtualization
@ stub -version=0xA00+ NdisMGetMiniportInitAttributes
@ stub -version=0xA00+ NdisMGetOffloadHandlers
@ stub -version=0xA00+ NdisMGetVirtualDeviceLocation
@ stub -version=0xA00+ NdisMGetVirtualFunctionBusData
@ stub -version=0xA00+ NdisMGetVirtualFunctionLocation
@ stub -version=0xA00+ NdisMIdleNotificationCompleteEx
@ stub -version=0xA00+ NdisMInitiateOffloadComplete
@ stub -version=0xA00+ NdisMInvalidateConfigBlock
@ stub -version=0xA00+ NdisMInvalidateOffloadComplete
@ stub -version=0xA00+ NdisMOffloadEventIndicate
@ stub -version=0xA00+ NdisMQueryOffloadStateComplete
@ stub -version=0xA00+ NdisMQueryProbedBars
@ stub -version=0xA00+ NdisMQueueDpc
@ stub -version=0xA00+ NdisMQueueDpcEx
@ stub -version=0xA00+ NdisMReadConfigBlock
@ stub -version=0xA00+ NdisMReenumerateFailedAdapter
@ stub -version=0xA00+ NdisMRequestDpc
@ stub -version=0xA00+ NdisMResetMiniport
@ stub -version=0xA00+ NdisMSetInterfaceCompartment
@ stub -version=0xA00+ NdisMSetVirtualFunctionBusData
@ stub -version=0xA00+ NdisMTerminateOffloadComplete
@ stub -version=0xA00+ NdisMTriggerPDDrainNotification
@ stub -version=0xA00+ NdisMUpdateOffloadComplete
@ stub -version=0xA00+ NdisMWriteConfigBlock
@ stub -version=0xA00+ NdisMatchPdoWithPacket
@ stub -version=0xA00+ NdisMaxGroupCount
@ stub -version=0xA00+ NdisMonitorReceiveDrop
@ stub -version=0xA00+ NdisNblTrackerDeregisterComponent
@ stub -version=0xA00+ NdisNblTrackerQueryNblCurrentOwner
@ stub -version=0xA00+ NdisNblTrackerRecordEvent
@ stub -version=0xA00+ NdisNblTrackerRegisterComponent
@ stub -version=0xA00+ NdisNblTrackerTransferOwnership
@ stub -version=0xA00+ NdisOffloadTcpDisconnect
@ stub -version=0xA00+ NdisOffloadTcpForward
@ stub -version=0xA00+ NdisOffloadTcpReceive
@ stub -version=0xA00+ NdisOffloadTcpReceiveReturn
@ stub -version=0xA00+ NdisOffloadTcpSend
@ stub -version=0xA00+ NdisOidRequest
@ stub -version=0xA00+ NdisOpenAdapterEx
@ stub -version=0xA00+ NdisOpenNDKAdapter
@ stub -version=0xA00+ NdisPDStartup
@ stub -version=0xA00+ NdisProcessorIndexToNumber
@ stub -version=0xA00+ NdisProcessorNumberToIndex
@ stub -version=0xA00+ NdisQueryDiagnosticSetting
@ stub -version=0xA00+ NdisQueryNetBufferPhysicalCount
@ stub -version=0xA00+ NdisQueryOffloadState
@ stub -version=0xA00+ NdisReferenceWithTag
@ stub -version=0xA00+ NdisRegisterPoll
@ stub -version=0xA00+ NdisRegisterProtocolDriver
@ stub -version=0xA00+ NdisRegisterUserThreadExitCallback
@ stub -version=0xA00+ NdisReleaseNicActive
@ stub -version=0xA00+ NdisRequestEx
@ stub -version=0xA00+ NdisRequestPoll
@ stub -version=0xA00+ NdisRetreatNetBufferDataStart
@ stub -version=0xA00+ NdisRetreatNetBufferListDataStart
@ stub -version=0xA00+ NdisReturnNetBufferLists
@ stub -version=0xA00+ NdisSendNetBufferLists
@ stub -version=0xA00+ NdisSetAoAcOptions
@ stub -version=0xA00+ NdisSetCoalescableTimerObject
@ stub -version=0xA00+ NdisSetPeriodicTimer
@ stub -version=0xA00+ NdisSetPollAffinity
@ stub -version=0xA00+ NdisSetSessionCompartmentId
@ stub -version=0xA00+ NdisSetThreadObjectCompartmentId
@ stub -version=0xA00+ NdisSetThreadObjectCompartmentScope
@ stub -version=0xA00+ NdisSynchronousOidRequest
@ stub -version=0xA00+ NdisSystemActiveProcessorCount
@ stub -version=0xA00+ NdisTerminateOffload
@ stub -version=0xA00+ NdisTestRWLockHeldByCurrentProcessorRead
@ stub -version=0xA00+ NdisTestRWLockHeldByCurrentProcessorWrite
@ stub -version=0xA00+ NdisTryAcquireNicActive
@ stub -version=0xA00+ NdisTryAcquireRWLockRead
@ stub -version=0xA00+ NdisTryAcquireRWLockWrite
@ stub -version=0xA00+ NdisTryPromoteRWLockFromReadToWrite
@ stub -version=0xA00+ NdisUnbindAdapter
@ stub -version=0xA00+ NdisUnregisterUserThreadExitCallback
@ stub -version=0xA00+ NdisUpdateOffload
@ stub -version=0xA00+ NdisWdfAsyncPowerReferenceCompleteNotification
@ stub -version=0xA00+ NdisWdfChangeSingleInstance
@ stub -version=0xA00+ NdisWdfCloseIrpHandler
@ stub -version=0xA00+ NdisWdfCreateIrpHandler
@ stub -version=0xA00+ NdisWdfDeregisterCx
@ stub -version=0xA00+ NdisWdfDeviceControlIrpHandler
@ stub -version=0xA00+ NdisWdfDeviceInternalControlIrpHandler
@ stub -version=0xA00+ NdisWdfExecuteMethod
@ stub -version=0xA00+ NdisWdfGenerateFdoNameIndex
@ stub -version=0xA00+ NdisWdfGetAdapterContextFromAdapterHandle
@ stub -version=0xA00+ NdisWdfGetGuidToOidMap
@ stub -version=0xA00+ NdisWdfMiniportDataPathPause
@ stub -version=0xA00+ NdisWdfMiniportDataPathStart
@ stub -version=0xA00+ NdisWdfMiniportDereference
@ stub -version=0xA00+ NdisWdfMiniportSetPower
@ stub -version=0xA00+ NdisWdfMiniportStarted
@ stub -version=0xA00+ NdisWdfMiniportTryReference
@ stub -version=0xA00+ NdisWdfNotifyWmiAdapterArrival
@ stub -version=0xA00+ NdisWdfPnPAddDevice
@ stub -version=0xA00+ NdisWdfPnpPowerEventHandler
@ stub -version=0xA00+ NdisWdfQueryAllData
@ stub -version=0xA00+ NdisWdfQuerySingleInstance
@ stub -version=0xA00+ NdisWdfReadConfiguration
@ stub -version=0xA00+ NdisWdfRegisterCx
@ stub -version=0xA00+ NdisWdfRegisterMiniportDriver
@ stub -version=0xA00+ NetDmaDeregisterProvider
@ stub -version=0xA00+ NetDmaGetVersion
@ stub -version=0xA00+ NetDmaInterruptDpc
@ stub -version=0xA00+ NetDmaIsr
@ stub -version=0xA00+ NetDmaPnPEventNotify
@ stub -version=0xA00+ NetDmaProviderStart
@ stub -version=0xA00+ NetDmaProviderStop
@ stub -version=0xA00+ NetDmaRegisterProvider
# END win10-surface stubs
