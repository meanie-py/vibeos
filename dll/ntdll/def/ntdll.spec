@ stdcall -version=0x502 PropertyLengthAsVariant(ptr long long long)
@ stdcall -version=0x502 RtlConvertPropertyToVariant(ptr long ptr ptr)
@ stdcall -version=0x502 RtlConvertVariantToProperty(ptr long ptr ptr ptr long ptr)
@ fastcall -arch=i386 RtlActivateActivationContextUnsafeFast(ptr ptr)
@ fastcall -arch=i386 RtlDeactivateActivationContextUnsafeFast(ptr)
@ fastcall -arch=i386 RtlInterlockedPushListSList(ptr ptr ptr long)
@ fastcall -arch=i386 RtlUlongByteSwap(long)
@ fastcall -arch=i386 -ret64 RtlUlonglongByteSwap(double)
@ fastcall -arch=i386 RtlUshortByteSwap(long)
@ stdcall -arch=i386 ExpInterlockedPopEntrySListEnd()
@ stdcall -arch=i386 ExpInterlockedPopEntrySListFault()
@ stdcall -arch=i386 ExpInterlockedPopEntrySListResume()
@ stdcall -version=0x600+ A_SHAFinal(ptr ptr)
@ stdcall -version=0x600+ A_SHAInit(ptr)
@ stdcall -version=0x600+ A_SHAUpdate(ptr ptr long)
@ stdcall -stub -version=0x600+ AlpcAdjustCompletionListConcurrencyCount(ptr long)
@ stdcall -stub -version=0x600+ AlpcFreeCompletionListMessage(ptr ptr)
@ stdcall -stub -version=0x600+ AlpcGetCompletionListLastMessageInformation(ptr ptr ptr)
@ stdcall -stub -version=0x600+ AlpcGetCompletionListMessageAttributes(ptr ptr)
@ stdcall -stub -version=0x600+ AlpcGetHeaderSize(long)
@ stdcall -stub -version=0x600+ AlpcGetMessageAttribute(ptr long)
@ stdcall -stub -version=0x600+ AlpcGetMessageFromCompletionList(ptr ptr)
@ stdcall -stub -version=0x600+ AlpcGetOutstandingCompletionListMessageCount(ptr)
@ stdcall -stub -version=0x600+ AlpcInitializeMessageAttribute(long ptr long ptr)
@ stdcall -stub -version=0x600+ AlpcMaxAllowedMessageLength()
@ stdcall -stub -version=0x600+ AlpcRegisterCompletionList(ptr ptr long long long)
@ stdcall -stub -version=0x600+ AlpcRegisterCompletionListWorkerThread(ptr)
@ stdcall -stub -version=0x600+ AlpcUnregisterCompletionList(ptr)
@ stdcall -stub -version=0x600+ AlpcUnregisterCompletionListWorkerThread(ptr)
@ stdcall CsrAllocateCaptureBuffer(long long)
@ stdcall CsrAllocateMessagePointer(ptr long ptr)
@ stdcall CsrCaptureMessageBuffer(ptr ptr long ptr)
@ stdcall CsrCaptureMessageMultiUnicodeStringsInPlace(ptr long ptr)
@ stdcall CsrCaptureMessageString(ptr str long long ptr)
@ stdcall CsrCaptureTimeout(long ptr)
@ stdcall CsrClientCallServer(ptr ptr long long)
@ stdcall CsrClientConnectToServer(str long ptr ptr ptr)
@ stdcall CsrFreeCaptureBuffer(ptr)
@ stdcall CsrGetProcessId()
@ stdcall CsrIdentifyAlertableThread()
@ stdcall -version=0x502+ CsrNewThread() # Should be only 0x502, but we need it for now
@ stdcall -version=0x502 CsrProbeForRead(ptr long long)
@ stdcall -version=0x502 CsrProbeForWrite(ptr long long)
@ stdcall CsrSetPriorityClass(ptr ptr)
@ stdcall -stub -version=0x600+ CsrVerifyRegion(ptr long)
@ stdcall DbgBreakPoint()
@ varargs DbgPrint(str)
@ varargs DbgPrintEx(long long str)
@ varargs DbgPrintReturnControlC(str)
@ stdcall DbgPrompt(ptr ptr long)
@ stdcall DbgQueryDebugFilterState(long long)
@ stdcall DbgSetDebugFilterState(long long long)
@ stdcall DbgUiConnectToDbg()
@ stdcall DbgUiContinue(ptr long)
@ stdcall DbgUiConvertStateChangeStructure(ptr ptr)
@ stdcall DbgUiDebugActiveProcess(ptr)
@ stdcall DbgUiGetThreadDebugObject()
@ stdcall DbgUiIssueRemoteBreakin(ptr)
@ stdcall DbgUiRemoteBreakin()
@ stdcall DbgUiSetThreadDebugObject(ptr)
@ stdcall DbgUiStopDebugging(ptr)
@ stdcall DbgUiWaitStateChange(ptr ptr)
@ stdcall DbgUserBreakPoint()
@ stdcall -version=0x502 EtwControlTraceA(double str ptr long)
@ stdcall -version=0x502 EtwControlTraceW(double wstr ptr long)
@ stdcall -stub EtwCreateTraceInstanceId(ptr ptr)
@ stdcall -stub -version=0x600+ EtwDeliverDataBlock(long)
@ stdcall -version=0x502 EtwEnableTrace(long long long ptr double)
@ stdcall -stub -version=0x600+ EtwEnumerateProcessRegGuids(ptr long ptr)
@ stdcall -stub -version=0x502 EtwEnumerateTraceGuids(ptr long ptr)
@ stdcall -stub -version=0x600+ EtwEventActivityIdControl(long ptr)
@ stdcall EtwEventEnabled(int64 ptr)
@ stdcall -stub -version=0x600+ EtwEventProviderEnabled(long long long)
@ stdcall EtwEventRegister(ptr ptr ptr ptr)
@ stdcall EtwEventSetInformation(int64 long ptr long)
@ stdcall EtwEventUnregister(int64)
@ stdcall EtwEventWrite(int64 ptr long ptr)
@ stdcall -stub -version=0x600+ EtwEventWriteEndScenario(long ptr long long)
@ stdcall -stub -version=0x600+ EtwEventWriteFull(long long long long long long long)
@ stdcall -stub -version=0x600+ EtwEventWriteStartScenario(long ptr long long)
@ stdcall EtwEventWriteString(int64 long int64 ptr)
@ stdcall EtwEventWriteTransfer(int64 ptr ptr ptr long ptr)
@ stdcall EtwEventWriteEx(int64 ptr int64 long ptr ptr long ptr)
@ stdcall -version=0x502 EtwFlushTraceA(double str ptr)
@ stdcall -version=0x502 EtwFlushTraceW(double wstr ptr)
@ stdcall EtwGetTraceEnableFlags(double)
@ stdcall EtwGetTraceEnableLevel(double)
@ stdcall EtwGetTraceLoggerHandle(ptr)
@ stdcall -stub -version=0x600+ EtwLogTraceEvent(long long)
@ stdcall -stub -version=0x600+ EtwNotificationRegister(ptr long long long ptr)
@ stdcall -stub -version=0x502 EtwNotificationRegistrationA(ptr long ptr long long)
@ stdcall -stub -version=0x502 EtwNotificationRegistrationW(ptr long ptr long long)
@ stdcall -stub -version=0x600+ EtwNotificationUnregister(long ptr)
@ stdcall -stub -version=0x600+ EtwProcessPrivateLoggerRequest(ptr)
@ stdcall -version=0x502 EtwQueryAllTracesA(ptr long ptr)
@ stdcall -version=0x502 EtwQueryAllTracesW(ptr long ptr)
@ stdcall -version=0x502 EtwQueryTraceA(double str ptr)
@ stdcall -version=0x502 EtwQueryTraceW(double wstr ptr)
@ stdcall -stub -version=0x502 EtwReceiveNotificationsA(long long long long)
@ stdcall -stub -version=0x502 EtwReceiveNotificationsW(long long long long)
@ stdcall -stub -version=0x600+ EtwRegister(ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ EtwRegisterSecurityProvider()
@ stdcall EtwRegisterTraceGuidsA(ptr ptr ptr long ptr str str ptr)
@ stdcall EtwRegisterTraceGuidsW(ptr ptr ptr long ptr wstr wstr ptr)
@ stdcall -stub -version=0x600+ EtwReplyNotification(long)
@ stdcall -stub -version=0x600+ EtwSendNotification(long long ptr long long)
@ stdcall -stub -version=0x600+ EtwSetMark(long long long)
@ stdcall -version=0x502 EtwStartTraceA(ptr str ptr)
@ stdcall -version=0x502 EtwStartTraceW(ptr wstr ptr)
@ stdcall -version=0x502 EtwStopTraceA(double str ptr)
@ stdcall -version=0x502 EtwStopTraceW(double wstr ptr)
@ stdcall -version=0x502 EtwTraceEvent(double ptr)
@ stdcall -stub EtwTraceEventInstance(double ptr ptr ptr)
@ varargs EtwTraceMessage(int64 long ptr long)
@ stdcall -stub EtwTraceMessageVa(int64 long ptr long ptr)
@ stdcall -stub -version=0x600+ EtwUnregister(int64)
@ stdcall EtwUnregisterTraceGuids(double)
@ stdcall -version=0x502 EtwUpdateTraceA(double str ptr)
@ stdcall -version=0x502 EtwUpdateTraceW(double wstr ptr)
@ stdcall -stub -version=0x600+ EtwWrite(int64 ptr ptr long ptr)
@ stdcall -stub -version=0x600+ EtwWriteUMSecurityEvent(ptr long long long)
@ stdcall -stub -version=0x600+ EtwpCreateEtwThread(long long)
@ stdcall -stub -version=0x600+ EtwpGetCpuSpeed(ptr)
@ stdcall -stub -version=0x502 EtwpGetTraceBuffer(long long long long)
@ stdcall -stub -version=0x600+ EtwpNotificationThread() ; doesn't exist in win11
@ stdcall -stub -version=0x502 EtwpSetHWConfigFunction(ptr long)
@ stdcall -arch=x86_64 ExpInterlockedPopEntrySListEnd()
@ stub -version=0x600+ -arch=x86_64 ExpInterlockedPopEntrySListEnd8
@ stdcall -arch=x86_64 ExpInterlockedPopEntrySListFault()
@ stub -version=0x600+ -arch=x86_64 ExpInterlockedPopEntrySListFault8
@ stdcall -arch=x86_64 ExpInterlockedPopEntrySListResume()
@ stub -version=0x600+ -arch=x86_64 ExpInterlockedPopEntrySListResume8
@ stdcall -arch=i386 KiFastSystemCall()
@ stdcall -arch=i386 KiFastSystemCallRet()
@ stdcall -arch=i386 KiIntSystemCall()
@ stdcall KiRaiseUserExceptionDispatcher()
@ stdcall KiUserApcDispatcher(ptr ptr ptr ptr)
@ stdcall KiUserCallbackDispatcher(ptr ptr long) ; CHECKME
@ stdcall KiUserExceptionDispatcher(ptr ptr)
@ stdcall -version=0x502 LdrAccessOutOfProcessResource(ptr ptr ptr ptr ptr)
@ stdcall LdrAccessResource(ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ LdrAddLoadAsDataTable(ptr wstr long ptr)
@ stdcall LdrAddRefDll(long ptr)
@ stdcall -version=0x502 LdrAlternateResourcesEnabled()
@ stdcall -version=0x502 LdrCreateOutOfProcessImage(long ptr ptr ptr)
@ stdcall -version=0x502 LdrDestroyOutOfProcessImage(ptr)
@ stdcall LdrDisableThreadCalloutsForDll(ptr)
@ stdcall LdrEnumResources(ptr ptr long ptr ptr)
@ stdcall LdrEnumerateLoadedModules(long ptr ptr)
@ stdcall -version=0x501-0x502 LdrFindCreateProcessManifest(long ptr ptr long ptr)
@ stdcall LdrFindEntryForAddress(ptr ptr)
@ stdcall LdrFindResourceDirectory_U(ptr ptr long ptr)
@ stdcall -stub LdrFindResourceEx_U(ptr ptr ptr ptr ptr) ; 5.1 and higher
@ stdcall LdrFindResource_U(ptr ptr long ptr)
@ stdcall LdrFlushAlternateResourceModules()
@ stdcall -arch=arm64,x86_64 LdrGetDllFullName(ptr ptr)
@ extern -arch=arm64,x86_64 LdrSystemDllInitBlock
@ stdcall LdrGetDllHandle(wstr ptr ptr ptr)
@ stdcall LdrGetDllHandleEx(long wstr ptr ptr ptr)
@ stdcall -stub -version=0x600+ LdrGetFailureData()
@ stdcall -stub -version=0x600+ LdrGetFileNameFromLoadAsDataTable(ptr ptr)
@ stdcall -stub -version=0x600+ -arch=x86_64 LdrGetKnownDllSectionHandle(wstr long ptr)
@ stdcall LdrGetProcedureAddress(ptr ptr long ptr)
@ stdcall LdrGetProcedureAddressEx(ptr ptr long ptr long)
@ stdcall -stub LdrHotPatchRoutine(ptr)
@ stdcall LdrInitShimEngineDynamic(ptr)
@ stdcall LdrInitializeThunk(long long long long)
@ stdcall LdrLoadAlternateResourceModule(ptr ptr)
@ stdcall -stub -version=0x600+ LdrLoadAlternateResourceModuleEx(long long ptr ptr long)
@ stdcall LdrLoadDll(wstr long ptr ptr)
@ stdcall LdrLockLoaderLock(long ptr ptr)
@ stdcall LdrOpenImageFileOptionsKey(ptr long ptr) ; 5.2 SP1 and higher
@ stdcall -stub -version=0x600+ -arch=x86_64 LdrProcessInitializationComplete()
@ stdcall LdrProcessRelocationBlock(ptr long ptr long)
@ stdcall LdrQueryImageFileExecutionOptions(ptr str long ptr long ptr)
@ stdcall LdrQueryImageFileExecutionOptionsEx(ptr ptr long ptr long ptr long)
@ stdcall LdrQueryImageFileKeyOption(ptr ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ LdrQueryModuleServiceTags(ptr ptr ptr)
@ stdcall LdrQueryProcessModuleInformation(ptr long ptr)
@ stdcall -version=0x600+ LdrRegisterDllNotification(long ptr ptr ptr)
@ stdcall -stub -version=0x600+ LdrRemoveLoadAsDataTable(ptr ptr ptr long)
@ stdcall -stub -version=0x600+ LdrResFindResource(ptr long long long long long long long long)
@ stdcall -stub -version=0x600+ LdrResFindResourceDirectory(ptr long long long long long long)
@ stdcall -stub -version=0x600+ LdrResRelease(ptr ptr long long)
@ stdcall -stub -version=0x600+ LdrResSearchResource(wstr wstr long long long ptr long long)
@ stdcall LdrSetAppCompatDllRedirectionCallback(long ptr ptr)
@ stdcall LdrSetDllManifestProber(ptr)
@ stdcall -stub -version=0x600+ LdrSetMUICacheType(long)
@ stdcall LdrShutdownProcess()
@ stdcall LdrShutdownThread()
@ stdcall LdrUnloadAlternateResourceModule(ptr)
@ stdcall -stub -version=0x600+ LdrUnloadAlternateResourceModuleEx(long long)
@ stdcall LdrUnloadDll(ptr)
@ stdcall LdrUnlockLoaderLock(long ptr)
@ stdcall -version=0x600+ LdrUnregisterDllNotification(ptr)
@ stdcall LdrVerifyImageMatchesChecksum(ptr long long long)
@ stdcall -stub -version=0x600+ LdrVerifyImageMatchesChecksumEx(ptr ptr)
@ stdcall -stub -version=0x600+ LdrpResGetMappingSize(long ptr long long)
@ stdcall -stub -version=0x600+ LdrpResGetRCConfig(long long ptr long long) ; doesn't exist on win11
@ stdcall -stub -version=0x600+ LdrpResGetResourceDirectory(long long long ptr ptr)
@ stdcall -version=0x600+ MD4Final(ptr)
@ stdcall -version=0x600+ MD4Init(ptr)
@ stdcall -version=0x600+ MD4Update(ptr ptr long)
@ stdcall -version=0x600+ MD5Final(ptr)
@ stdcall -version=0x600+ MD5Init(ptr)
@ stdcall -version=0x600+ MD5Update(ptr ptr long)
@ extern NlsAnsiCodePage
@ extern NlsMbCodePageTag
@ extern NlsMbOemCodePageTag
@ stdcall NtAcceptConnectPort(ptr long ptr long long ptr)
@ stdcall NtAccessCheck(ptr long long ptr ptr ptr ptr ptr)
@ stdcall NtAccessCheckAndAuditAlarm(ptr long ptr ptr ptr long ptr long ptr ptr ptr)
@ stdcall NtAccessCheckByType(ptr ptr ptr long ptr long ptr ptr long ptr ptr)
@ stdcall NtAccessCheckByTypeAndAuditAlarm(ptr ptr ptr ptr ptr ptr long long long ptr long ptr long ptr ptr ptr)
@ stdcall NtAccessCheckByTypeResultList(ptr ptr ptr long ptr long ptr ptr long ptr ptr)
@ stdcall NtAccessCheckByTypeResultListAndAuditAlarm(ptr ptr ptr ptr ptr ptr long long long ptr long ptr long ptr ptr ptr)
@ stdcall NtAccessCheckByTypeResultListAndAuditAlarmByHandle(ptr ptr ptr ptr ptr ptr ptr long long long ptr long ptr long ptr ptr ptr)
@ stdcall -stub -version=0x600+ NtAcquireCMFViewOwnership(ptr ptr long) ; doesn't exist on Win11
@ stdcall NtAddAtom(ptr long ptr)
@ stdcall NtAddBootEntry(ptr long)
@ stdcall NtAddDriverEntry(ptr long) ; 5.2 and higher
@ stdcall NtAdjustGroupsToken(long long ptr long ptr ptr)
@ stdcall NtAdjustPrivilegesToken(long long long long long long)
@ stdcall NtAlertResumeThread(long ptr)
@ stdcall NtAlertThread(long)
@ stdcall NtAllocateLocallyUniqueId(ptr)
@ stdcall NtAllocateUserPhysicalPages(ptr ptr ptr)
@ stdcall NtAllocateUuids(ptr ptr ptr ptr)
@ stdcall NtAllocateVirtualMemory(long ptr ptr ptr long long)
@ stdcall -arch=arm64,x86_64 NtAllocateVirtualMemoryEx(long ptr ptr long long ptr long)
@ stdcall -stub -version=0x600+ NtAlpcAcceptConnectPort(long long long long long long long long long)
@ stdcall -stub -version=0x600+ NtAlpcCancelMessage(long long long)
@ stdcall -stub -version=0x600+ NtAlpcConnectPort(long long long long long long long long long long long)
@ stdcall -stub -version=0x600+ NtAlpcCreatePort(long long long)
@ stdcall -stub -version=0x600+ NtAlpcCreatePortSection(long long long long ptr ptr)
@ stdcall -stub -version=0x600+ NtAlpcCreateResourceReserve(long long long ptr)
@ stdcall -stub -version=0x600+ NtAlpcCreateSectionView(long long long)
@ stdcall -stub -version=0x600+ NtAlpcCreateSecurityContext(long long ptr)
@ stdcall -stub -version=0x600+ NtAlpcDeletePortSection(long long long long ptr ptr)
@ stdcall -stub -version=0x600+ NtAlpcDeleteResourceReserve(long long long ptr)
@ stdcall -stub -version=0x600+ NtAlpcDeleteSectionView(long long long ptr)
@ stdcall -stub -version=0x600+ NtAlpcDeleteSecurityContext(long long long)
@ stdcall -stub -version=0x600+ NtAlpcDisconnectPort(long long)
@ stdcall -stub -version=0x600+ NtAlpcImpersonateClientOfPort(long long long)
@ stub -version=0x600+ NtAlpcOpenSenderProcess
@ stub -version=0x600+ NtAlpcOpenSenderThread
@ stdcall -stub -version=0x600+ NtAlpcQueryInformation(long long long long long)
@ stdcall -stub -version=0x600+ NtAlpcQueryInformationMessage(long long long long long long)
@ stdcall -stub -version=0x600+ NtAlpcRevokeSecurityContext(long long long)
@ stdcall -stub -version=0x600+ NtAlpcSendWaitReceivePort(long long long long long long long long)
@ stdcall -stub -version=0x600+ NtAlpcSetInformation(long long long long)
@ stdcall NtApphelpCacheControl(long ptr)
@ stdcall NtAreMappedFilesTheSame(ptr ptr)
@ stdcall NtAssignProcessToJobObject(long long)
@ stdcall NtCallbackReturn(ptr long long)
@ stdcall NtCancelDeviceWakeupRequest(ptr)
@ stdcall NtCancelIoFile(long ptr)
@ stdcall NtCancelIoFileEx(long ptr ptr)
@ stdcall NtCancelSynchronousIoFile(long ptr ptr)
@ stdcall NtCancelTimer(long ptr)
@ stdcall NtClearEvent(long)
@ stdcall NtClose(long)
@ stdcall NtCloseObjectAuditAlarm(ptr ptr long)
@ stdcall -stub -version=0x600+ NtCommitComplete(ptr ptr)
@ stdcall -stub -version=0x600+ NtCommitEnlistment(ptr ptr)
@ stdcall NtCommitTransaction(ptr long)
@ stdcall NtCompactKeys(long ptr)
@ stdcall NtCompareObjects(ptr ptr)
@ stdcall NtCompareTokens(ptr ptr ptr)
@ stdcall NtCompleteConnectPort(ptr)
@ stdcall NtCompressKey(ptr)
@ stdcall NtConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtContinue(ptr long)
@ stdcall NtCreateDebugObject(ptr long ptr long)
@ stdcall NtCreateDirectoryObject(long long long)
@ stdcall -stub -version=0x600+ NtCreateEnlistment(ptr long ptr ptr ptr long long ptr)
@ stdcall NtCreateEvent(long long long long long)
@ stdcall NtCreateEventPair(ptr long ptr)
@ stdcall NtCreateFile(ptr long ptr ptr long long long ptr long long ptr)
@ stdcall NtCreateIoCompletion(ptr long ptr long)
@ stdcall NtCreateJobObject(ptr long ptr)
@ stdcall NtCreateJobSet(long ptr long)
@ stdcall NtCreateKey(ptr long ptr long ptr long long)
@ stdcall -stub -version=0x600+ NtCreateKeyTransacted(ptr long ptr long ptr long long ptr)
@ stdcall NtCreateKeyedEvent(ptr long ptr long)
@ stdcall NtCreateMailslotFile(long long long long long long long long)
@ stdcall NtCreateMutant(ptr long ptr long)
@ stdcall NtCreateNamedPipeFile(ptr long ptr ptr long long long long long long long long long ptr)
@ stdcall NtCreatePagingFile(ptr ptr ptr long)
@ stdcall NtCreatePort(ptr ptr long long ptr)
@ stdcall -stub -version=0x600+ NtCreatePrivateNamespace(ptr long ptr ptr)
@ stdcall NtCreateProcess(ptr long ptr ptr long ptr ptr ptr)
@ stdcall NtCreateProcessEx(ptr long ptr ptr long ptr ptr ptr long)
@ stdcall NtCreateProfile(ptr ptr ptr long long ptr long long long) ; CHECKME
@ stdcall -stub -version=0x600+ NtCreateResourceManager(ptr long ptr ptr ptr long wstr)
@ stdcall NtCreateSection(ptr long ptr ptr long long ptr)
@ stdcall NtCreateSemaphore(ptr long ptr long long)
@ stdcall NtCreateSymbolicLinkObject(ptr long ptr ptr)
@ stdcall NtCreateThread(ptr long ptr ptr ptr ptr ptr long)
@ stub -version=0x600+ NtCreateThreadEx
@ stdcall NtCreateTimer(ptr long ptr long)
@ stdcall NtCreateToken(ptr long ptr long ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtCreateTransaction(ptr long ptr ptr ptr long long long ptr ptr)
@ stdcall -stub -version=0x600+ NtCreateTransactionManager(ptr long ptr wstr long long)
@ stdcall -stub -version=0x600+ NtCreateUserProcess(ptr ptr long long ptr ptr long long ptr long long)
@ stdcall NtCreateWaitablePort(ptr ptr long long long)
@ stdcall -stub -version=0x600+ NtCreateWorkerFactory(ptr long long long long long long long long long)
@ stdcall -arch=win32 NtCurrentTeb() _NtCurrentTeb
@ stdcall NtDebugActiveProcess(ptr ptr)
@ stdcall NtDebugContinue(ptr ptr long)
@ stdcall NtDelayExecution(long ptr)
@ stdcall NtDeleteAtom(long)
@ stdcall NtDeleteBootEntry(long)
@ stdcall NtDeleteDriverEntry(long)
@ stdcall NtDeleteFile(ptr)
@ stdcall NtDeleteKey(long)
@ stdcall NtDeleteObjectAuditAlarm(ptr ptr long)
@ stdcall -stub -version=0x600+ NtDeletePrivateNamespace(long)
@ stdcall NtDeleteValueKey(long ptr)
@ stdcall NtDeviceIoControlFile(long long long long long long long long long long)
@ stdcall NtDisplayString(ptr)
@ stdcall NtDuplicateObject(long long long ptr long long long)
@ stdcall NtDuplicateToken(long long long long long long)
@ stdcall NtEnumerateBootEntries(ptr ptr)
@ stdcall NtEnumerateDriverEntries(ptr ptr)
@ stdcall NtEnumerateKey (long long long long long long)
@ stdcall NtEnumerateSystemEnvironmentValuesEx(long ptr long)
@ stub -version=0x600+ NtEnumerateTransactionObject
@ stdcall NtEnumerateValueKey(long long long long long long)
@ stdcall NtExtendSection(ptr ptr)
@ stdcall NtFilterToken(ptr long ptr ptr ptr ptr)
@ stdcall NtFindAtom(ptr long ptr)
@ stdcall NtFlushBuffersFile(long ptr)
@ stdcall -stub -version=0x600+ NtFlushInstallUILanguage(long long)
@ stdcall NtFlushInstructionCache(long ptr long)
@ stdcall NtFlushKey(long)
@ stdcall NtFlushProcessWriteBuffers()
@ stdcall NtFlushVirtualMemory(ptr ptr ptr ptr)
@ stdcall NtFlushWriteBuffer()
@ stdcall NtFreeUserPhysicalPages(ptr ptr ptr)
@ stdcall NtFreeVirtualMemory(long ptr ptr long)
@ stdcall -stub -version=0x600+ NtFreezeRegistry(long)
@ stdcall -stub -version=0x600+ NtFreezeTransactions(ptr ptr)
@ stdcall NtFsControlFile(long long long long long long long long long long)
@ stdcall NtGetContextThread(long ptr)
@ stdcall NtGetCurrentProcessorNumber() ; 5.2 and higher
@ stdcall -version=0xA00+ NtGetCurrentProcessorNumberEx(ptr)
@ stdcall NtGetDevicePowerState(ptr ptr)
@ stdcall -stub -version=0x600+ NtGetMUIRegistryInfo(long ptr ptr)
@ stdcall -stub -version=0x600+ NtGetNextProcess(long long long long ptr)
@ stdcall NtGetNextThread(ptr ptr long long long ptr)
@ stdcall -stub -version=0x600+ NtGetNlsSectionPtr(long long long ptr ptr)
@ stdcall -stub -version=0x600+ NtGetNotificationResourceManager(ptr ptr long ptr ptr long long)
@ stdcall NtGetPlugPlayEvent(long long ptr long)
@ stdcall NtGetTickCount() RtlGetTickCount
@ stdcall NtGetWriteWatch(long long ptr long ptr ptr ptr)
@ stdcall NtImpersonateAnonymousToken(ptr)
@ stdcall NtImpersonateClientOfPort(ptr ptr)
@ stdcall NtImpersonateThread(ptr ptr ptr)
@ stdcall -stub -version=0x600+ NtInitializeNlsFiles(ptr ptr)
@ stdcall NtInitializeRegistry(long)
@ stdcall NtInitiatePowerAction (long long long long)
@ stdcall NtIsProcessInJob(long long)
@ stdcall NtIsSystemResumeAutomatic()
@ stdcall -stub -version=0x600+ NtIsUILanguageComitted()
@ stdcall NtListenPort(ptr ptr)
@ stdcall NtLoadDriver(ptr)
@ stdcall NtLoadKey2(ptr ptr long)
@ stdcall NtLoadKey(ptr ptr)
@ stdcall NtLoadKeyEx(ptr ptr long ptr)
@ stdcall NtLockFile(long long ptr ptr ptr ptr ptr ptr long long)
@ stdcall NtLockProductActivationKeys(ptr ptr)
@ stdcall NtLockRegistryKey(ptr)
@ stdcall NtLockVirtualMemory(long ptr ptr long)
@ stdcall NtMakePermanentObject(ptr)
@ stdcall NtMakeTemporaryObject(long)
@ stdcall -stub -version=0x600+ NtMapCMFModule(long long ptr ptr ptr ptr)
@ stdcall NtMapUserPhysicalPages(ptr ptr ptr)
@ stdcall NtMapUserPhysicalPagesScatter(ptr ptr ptr)
@ stdcall NtMapViewOfSection(long long ptr long long ptr ptr long long long)
@ stdcall NtModifyBootEntry(ptr)
@ stdcall NtModifyDriverEntry(ptr)
@ stdcall NtNotifyChangeDirectoryFile(long long ptr ptr ptr ptr long long long)
@ stdcall NtNotifyChangeKey(long long ptr ptr ptr long long ptr long long)
@ stdcall NtNotifyChangeMultipleKeys(ptr long ptr ptr ptr ptr ptr long long ptr long long)
@ stdcall NtOpenDirectoryObject(long long long)
@ stdcall -stub -version=0x600+ NtOpenEnlistment(ptr long ptr ptr ptr)
@ stdcall NtOpenEvent(long long long)
@ stdcall NtOpenEventPair(ptr long ptr)
@ stdcall NtOpenFile(ptr long ptr ptr long long)
@ stdcall NtOpenIoCompletion(ptr long ptr)
@ stdcall NtOpenJobObject(ptr long ptr)
@ stdcall NtOpenKey(ptr long ptr)
@ stdcall NtOpenKeyEx(ptr long ptr long)
@ stdcall -stub -version=0x600+ NtOpenKeyTransacted(ptr long ptr ptr)
@ stdcall NtOpenKeyedEvent(ptr long ptr)
@ stdcall NtOpenMutant(ptr long ptr)
@ stdcall NtOpenObjectAuditAlarm(ptr ptr ptr ptr ptr ptr long long ptr long long ptr)
@ stdcall -stub -version=0x600+ NtOpenPrivateNamespace(ptr long ptr ptr)
@ stdcall NtOpenProcess(ptr long ptr ptr)
@ stdcall NtOpenProcessToken(long long ptr)
@ stdcall NtOpenProcessTokenEx(long long long ptr)
@ stdcall -stub -version=0x600+ NtOpenResourceManager(ptr long ptr ptr ptr)
@ stdcall NtOpenSection(ptr long ptr)
@ stdcall NtOpenSemaphore(long long ptr)
@ stdcall -stub -version=0x600+ NtOpenSession(ptr long ptr)
@ stdcall NtOpenSymbolicLinkObject (ptr long ptr)
@ stdcall NtOpenThread(ptr long ptr ptr)
@ stdcall NtOpenThreadToken(long long long ptr)
@ stdcall NtOpenThreadTokenEx(long long long long ptr)
@ stdcall NtOpenTimer(ptr long ptr)
@ stdcall -stub -version=0x600+ NtOpenTransaction(ptr long ptr ptr ptr)
@ stdcall -stub -version=0x600+ NtOpenTransactionManager(ptr long ptr ptr ptr long)
@ stdcall NtPlugPlayControl(ptr ptr long)
@ stdcall NtPowerInformation(long ptr long ptr long)
@ stdcall -stub -version=0x600+ NtPrePrepareComplete(ptr ptr)
@ stdcall -stub -version=0x600+ NtPrePrepareEnlistment(ptr ptr)
@ stdcall -stub -version=0x600+ NtPrepareComplete(ptr ptr)
@ stdcall -stub -version=0x600+ NtPrepareEnlistment(ptr ptr)
@ stdcall NtPrivilegeCheck(ptr ptr ptr)
@ stdcall NtPrivilegeObjectAuditAlarm(ptr ptr ptr long ptr long)
@ stdcall NtPrivilegedServiceAuditAlarm(ptr ptr ptr ptr long)
@ stdcall -stub -version=0x600+ NtPropagationComplete(ptr long long ptr)
@ stdcall -stub -version=0x600+ NtPropagationFailed(ptr long long)
@ stdcall NtProtectVirtualMemory(long ptr ptr long ptr)
@ stdcall NtPulseEvent(long ptr)
@ stdcall NtQueryAttributesFile(ptr ptr)
@ stdcall NtQueryBootEntryOrder(ptr ptr)
@ stdcall NtQueryBootOptions(ptr ptr)
@ stdcall NtQueryDebugFilterState(long long)
@ stdcall NtQueryDefaultLocale(long ptr)
@ stdcall NtQueryDefaultUILanguage(ptr)
@ stdcall NtQueryDirectoryFile(long long ptr ptr ptr ptr long long long ptr long)
@ stdcall NtQueryDirectoryObject(long ptr long long long ptr ptr)
@ stdcall NtQueryDriverEntryOrder(ptr ptr)
@ stdcall NtQueryEaFile(long ptr ptr long long ptr long ptr long)
@ stdcall NtQueryEvent(long long ptr long ptr)
@ stdcall NtQueryFullAttributesFile(ptr ptr)
@ stdcall NtQueryInformationAtom(long long ptr long ptr)
@ stdcall -stub -version=0x600+ NtQueryInformationEnlistment(ptr long ptr long ptr)
@ stdcall NtQueryInformationFile(ptr ptr ptr long long)
@ stdcall NtQueryInformationJobObject(ptr long ptr long ptr)
@ stdcall NtQueryInformationPort(ptr long ptr long ptr)
@ stdcall NtQueryInformationProcess(ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ NtQueryInformationResourceManager(ptr long ptr long ptr)
@ stdcall NtQueryInformationThread(ptr long ptr long ptr)
@ stdcall NtQueryInformationToken(ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ NtQueryInformationTransaction(ptr long ptr ptr ptr)
@ stdcall -stub -version=0x600+ NtQueryInformationTransactionManager(ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ NtQueryInformationWorkerFactory(ptr long ptr long ptr)
@ stdcall NtQueryInstallUILanguage(ptr)
@ stdcall NtQueryIntervalProfile(long ptr)
@ stdcall NtQueryIoCompletion(long long ptr long ptr)
@ stdcall NtQueryKey (long long ptr long ptr)
@ stdcall -stub -version=0x600+ NtQueryLicenseValue(wstr ptr ptr long ptr)
@ stdcall NtQueryMultipleValueKey(long ptr long ptr long ptr)
@ stdcall NtQueryMutant(long long ptr long ptr)
@ stdcall NtQueryObject(long long long long long)
@ stdcall NtQueryOpenSubKeys(ptr ptr)
@ stdcall NtQueryOpenSubKeysEx(ptr long ptr ptr)
@ stdcall NtQueryPerformanceCounter(ptr ptr)
@ stdcall NtQueryPortInformationProcess()
@ stdcall NtQueryQuotaInformationFile(ptr ptr ptr long long ptr long ptr long)
@ stdcall NtQuerySection (long long long long long)
@ stdcall NtQuerySecurityObject (long long long long long)
@ stdcall NtQuerySemaphore (long long ptr long ptr)
@ stdcall NtQuerySymbolicLinkObject(long ptr ptr)
@ stdcall NtQuerySystemEnvironmentValue(ptr ptr long ptr)
@ stdcall NtQuerySystemEnvironmentValueEx(ptr ptr ptr ptr ptr)
@ stdcall NtQuerySystemInformation(long long long long)
@ stdcall -stub -version=0x601+ NtQuerySystemInformationEx(long ptr long ptr long ptr)
@ stdcall NtQuerySystemTime(ptr)
@ stdcall NtQueryTimer(ptr long ptr long ptr)
@ stdcall NtQueryTimerResolution(long long long)
@ stdcall NtQueryValueKey(long long long long long long)
@ stdcall NtQueryVirtualMemory(long ptr long ptr long ptr)
@ stdcall NtQueryVolumeInformationFile(long ptr ptr long long)
@ stdcall NtQueueApcThread(long ptr long long long)
@ stdcall NtRaiseException(ptr ptr long)
@ stdcall NtRaiseHardError(long long long ptr long ptr)
@ stdcall NtReadFile(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall NtReadFileScatter(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall -stub -version=0x600+ NtReadOnlyEnlistment(ptr ptr)
@ stdcall NtReadRequestData(ptr ptr long ptr long ptr)
@ stdcall NtReadVirtualMemory(long ptr ptr long ptr)
@ stdcall -stub -version=0x600+ NtRecoverEnlistment(ptr ptr)
@ stdcall -stub -version=0x600+ NtRecoverResourceManager(ptr)
@ stdcall -stub -version=0x600+ NtRecoverTransactionManager(ptr)
@ stdcall -stub -version=0x600+ NtRegisterProtocolAddressInformation(ptr ptr long ptr long)
@ stdcall NtRegisterThreadTerminatePort(ptr)
@ stdcall -stub -version=0x600+ NtReleaseCMFViewOwnership()
@ stdcall NtReleaseKeyedEvent(ptr ptr long ptr)
@ stdcall NtReleaseMutant(long ptr)
@ stdcall NtReleaseSemaphore(long long ptr)
@ stdcall -stub -version=0x600+ NtReleaseWorkerFactoryWorker(ptr)
@ stdcall NtRemoveIoCompletion(ptr ptr ptr ptr ptr)
@ stdcall NtRemoveIoCompletionEx(ptr ptr long ptr ptr long)
@ stdcall NtRemoveProcessDebug(ptr ptr)
@ stdcall NtRenameKey(ptr ptr)
@ stdcall -stub -version=0x600+ NtRenameTransactionManager(ptr ptr)
@ stdcall NtReplaceKey(ptr long ptr)
@ stdcall -stub -version=0x600+ NtReplacePartitionUnit(wstr wstr long)
@ stdcall NtReplyPort(ptr ptr)
@ stdcall NtReplyWaitReceivePort(ptr ptr ptr ptr)
@ stdcall NtReplyWaitReceivePortEx(ptr ptr ptr ptr ptr)
@ stdcall NtReplyWaitReplyPort(ptr ptr)
@ stdcall NtRequestDeviceWakeup(ptr)
@ stdcall NtRequestPort(ptr ptr)
@ stdcall NtRequestWaitReplyPort(ptr ptr ptr)
@ stdcall NtRequestWakeupLatency(long)
@ stdcall NtResetEvent(long ptr)
@ stdcall NtResetWriteWatch(long ptr long)
@ stdcall NtRestoreKey(long long long)
@ stdcall NtResumeProcess(ptr)
@ stdcall NtResumeThread(long long)
@ stdcall -stub -version=0x600+ NtRollbackComplete(ptr ptr)
@ stdcall -stub -version=0x600+ NtRollbackEnlistment(ptr long ptr long)
@ stdcall NtRollbackTransaction(ptr long)
@ stdcall -stub -version=0x600+ NtRollforwardTransactionManager(ptr ptr)
@ stdcall NtSaveKey(long long)
@ stdcall NtSaveKeyEx(ptr ptr long)
@ stdcall NtSaveMergedKeys(ptr ptr ptr)
@ stdcall NtSecureConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall NtSetBootEntryOrder(ptr ptr)
@ stdcall NtSetBootOptions(ptr long)
@ stdcall NtSetContextThread(long ptr)
@ stdcall NtSetDebugFilterState(long long long)
@ stdcall NtSetDefaultHardErrorPort(ptr)
@ stdcall NtSetDefaultLocale(long long)
@ stdcall NtSetDefaultUILanguage(long)
@ stdcall NtSetDriverEntryOrder(ptr ptr)
@ stdcall NtSetEaFile(long ptr ptr long)
@ stdcall NtSetEvent(long long)
@ stdcall NtSetEventBoostPriority(ptr)
@ stdcall NtSetHighEventPair(ptr)
@ stdcall NtSetHighWaitLowEventPair(ptr)
@ stdcall NtSetInformationDebugObject(ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ NtSetInformationEnlistment(ptr long ptr long)
@ stdcall NtSetInformationFile(ptr ptr ptr long long)
@ stdcall NtSetInformationJobObject(ptr long ptr long)
@ stdcall NtSetInformationKey(ptr long ptr long)
@ stdcall NtSetInformationObject(ptr long ptr long)
@ stdcall NtSetInformationProcess(ptr long ptr long)
@ stdcall -stub -version=0x600+ NtSetInformationResourceManager(ptr long ptr long)
@ stdcall NtSetInformationThread(ptr long ptr long)
@ stdcall NtSetInformationToken(ptr long ptr long)
@ stdcall -stub -version=0x600+ NtSetInformationTransaction(ptr long ptr long)
@ stdcall -stub -version=0x600+ NtSetInformationTransactionManager(ptr long ptr long)
@ stdcall -stub -version=0x600+ NtSetInformationWorkerFactory(ptr long ptr long)
@ stdcall NtSetIntervalProfile(long long)
@ stdcall NtSetIoCompletion(ptr long ptr long long)
@ stdcall NtSetLdtEntries(long int64 long int64)
@ stdcall NtSetLowEventPair(ptr)
@ stdcall NtSetLowWaitHighEventPair(ptr)
@ stdcall NtSetQuotaInformationFile(ptr ptr ptr long)
@ stdcall NtSetSecurityObject(long long ptr)
@ stdcall NtSetSystemEnvironmentValue(ptr ptr)
@ stdcall NtSetSystemEnvironmentValueEx(ptr ptr ptr ptr ptr)
@ stdcall NtSetSystemInformation(long ptr long)
@ stdcall NtSetSystemPowerState(long long long)
@ stdcall NtSetSystemTime(ptr ptr)
@ stdcall NtSetThreadExecutionState(long ptr)
@ stdcall NtSetTimer(long ptr ptr ptr long long ptr)
@ stdcall NtSetTimerResolution(long long ptr)
@ stdcall NtSetUuidSeed(ptr)
@ stdcall NtSetValueKey(long long long long long long)
@ stdcall NtSetVolumeInformationFile(long ptr ptr long long)
@ stdcall NtShutdownSystem(long)
@ stdcall -stub -version=0x600+ NtShutdownWorkerFactory(ptr ptr)
@ stdcall NtSignalAndWaitForSingleObject(long long long ptr)
@ stdcall -stub -version=0x600+ NtSinglePhaseReject(ptr ptr)
@ stdcall NtStartProfile(ptr)
@ stdcall NtStopProfile(ptr)
@ stdcall NtSuspendProcess(ptr)
@ stdcall NtSuspendThread(long ptr)
@ stdcall NtSystemDebugControl(long ptr long ptr long ptr)
@ stdcall NtTerminateJobObject(ptr long)
@ stdcall NtTerminateProcess(ptr long)
@ stdcall NtTerminateThread(ptr long)
@ stdcall NtTestAlert()
@ stdcall -stub -version=0x600+ NtThawRegistry()
@ stdcall -stub -version=0x600+ NtThawTransactions()
@ stdcall -stub -version=0x600+ NtTraceControl(long ptr long ptr long long)
@ stdcall NtTraceEvent(long long long ptr)
@ stdcall NtTranslateFilePath(ptr long ptr long)
@ stdcall NtUnloadDriver(ptr)
@ stdcall NtUnloadKey2(ptr long)
@ stdcall NtUnloadKey(long)
@ stdcall NtUnloadKeyEx(ptr ptr)
@ stdcall NtUnlockFile(long ptr ptr ptr ptr)
@ stdcall NtUnlockVirtualMemory(long ptr ptr long)
@ stdcall NtUnmapViewOfSection(long ptr)
@ stdcall NtVdmControl(long ptr)
@ stdcall NtWaitForDebugEvent(ptr long ptr ptr)
@ stdcall NtWaitForKeyedEvent(ptr ptr long ptr)
@ stdcall NtWaitForMultipleObjects32(long ptr long long ptr)
@ stdcall NtWaitForMultipleObjects(long ptr long long ptr)
@ stdcall NtWaitForSingleObject(long long long)
@ stub -version=0x600+ NtWaitForWorkViaWorkerFactory
@ stdcall NtWaitHighEventPair(ptr)
@ stdcall NtWaitLowEventPair(ptr)
@ stdcall -stub -version=0x600+ NtWorkerFactoryWorkerReady(long)
@ stdcall NtWriteFile(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall NtWriteFileGather(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall NtWriteRequestData(ptr ptr long ptr long ptr)
@ stdcall NtWriteVirtualMemory(long ptr ptr long ptr)
@ stdcall NtYieldExecution()
@ stdcall -stub -version=0x600+ NtdllDefWindowProc_A(long long long long)
@ stdcall -stub -version=0x600+ NtdllDefWindowProc_W(long long long long)
@ stdcall -stub -version=0x600+ NtdllDialogWndProc_A(long long long long)
@ stdcall -stub -version=0x600+ NtdllDialogWndProc_W(long long long long)
@ stdcall PfxFindPrefix(ptr ptr)
@ stdcall PfxInitialize(ptr)
@ stdcall PfxInsertPrefix(ptr ptr ptr)
@ stdcall PfxRemovePrefix(ptr ptr)
@ stdcall RtlAbortRXact(ptr)
@ stdcall RtlAbsoluteToSelfRelativeSD(ptr ptr ptr)
@ stdcall RtlAcquirePebLock()
@ stdcall RtlAcquirePrivilege(ptr long long ptr)
@ stdcall RtlAcquireResourceExclusive(ptr long)
@ stdcall RtlAcquireResourceShared(ptr long)
@ stdcall -version=0x600+ RtlAcquireSRWLockExclusive(ptr)
@ stdcall -arch=arm64 RtlAcquireSRWLockExclusive(ptr)
@ stdcall -version=0x600+ RtlAcquireSRWLockShared(ptr)
@ stdcall -arch=arm64 RtlAcquireSRWLockShared(ptr)
@ stdcall RtlActivateActivationContext(long ptr ptr)
@ stdcall RtlActivateActivationContextEx(long ptr ptr ptr)
@ stdcall -arch=x86_64,arm,arm64 RtlActivateActivationContextUnsafeFast(ptr ptr)
@ stdcall RtlAddAccessAllowedAce(ptr long long ptr)
@ stdcall RtlAddAccessAllowedAceEx(ptr long long long ptr)
@ stdcall RtlAddAccessAllowedObjectAce(ptr long long long ptr ptr ptr)
@ stdcall RtlAddAccessDeniedAce(ptr long long ptr)
@ stdcall RtlAddAccessDeniedAceEx(ptr long long long ptr)
@ stdcall RtlAddAccessDeniedObjectAce(ptr long long long ptr ptr ptr)
@ stdcall RtlAddAce(ptr long long ptr long)
@ stdcall RtlAddActionToRXact(ptr long ptr long ptr long)
@ stdcall RtlAddAtomToAtomTable(ptr wstr ptr)
@ stdcall RtlAddAttributeActionToRXact(ptr long ptr ptr ptr long ptr long)
@ stdcall RtlAddAuditAccessAce(ptr long long ptr long long)
@ stdcall RtlAddAuditAccessAceEx(ptr long long long ptr long long)
@ stdcall RtlAddAuditAccessObjectAce(ptr long long long ptr ptr ptr long long)
@ stdcall -stub RtlAddCompoundAce(ptr long long long ptr ptr)
@ stdcall -arch=x86_64,arm64 RtlAddFunctionTable(ptr long long)
@ stdcall -stub -version=0x600+ RtlAddMandatoryAce(ptr long long long long ptr) ; not present in Win11
@ stdcall RtlAddRefActivationContext(ptr)
@ stdcall RtlAddRefMemoryStream(ptr)
@ stdcall -stub -version=0x600+ RtlAddSIDToBoundaryDescriptor(ptr ptr) ; not present in Win11
@ stdcall RtlAddVectoredContinueHandler(long ptr)
@ stdcall RtlAddVectoredExceptionHandler(long ptr)
@ stdcall -stub RtlAddressInSectionTable(ptr ptr long)
@ stdcall RtlAdjustPrivilege(long long long ptr)
@ stdcall RtlAllocateActivationContextStack(ptr) ; CHECKME
@ stdcall RtlAllocateAndInitializeSid(ptr long long long long long long long long long ptr)
@ stdcall RtlAllocateHandle(ptr ptr)
@ stdcall RtlAllocateHeap(ptr long ptr)
@ stdcall -stub -version=0x600+ RtlAllocateMemoryBlockLookaside(ptr long ptr) ; not present in Win11
@ stdcall -stub -version=0x600+ RtlAllocateMemoryZone(long long ptr) ; not present in Win11
@ stdcall RtlAnsiCharToUnicodeChar(ptr)
@ stdcall RtlAnsiStringToUnicodeSize(ptr) RtlxAnsiStringToUnicodeSize
@ stdcall RtlAnsiStringToUnicodeString(ptr ptr long)
@ stdcall RtlAppendAsciizToString(ptr str)
@ stdcall -stub RtlAppendPathElement(ptr ptr ptr)
@ stdcall RtlAppendStringToString(ptr ptr)
@ stdcall RtlAppendUnicodeStringToString(ptr ptr)
@ stdcall RtlAppendUnicodeToString(ptr wstr)
@ stdcall RtlApplicationVerifierStop(ptr str ptr str ptr str ptr str ptr str)
@ stdcall RtlApplyRXact(ptr)
@ stdcall RtlApplyRXactNoFlush(ptr)
@ stdcall RtlAreAllAccessesGranted(long long)
@ stdcall RtlAreAnyAccessesGranted(long long)
@ stdcall RtlAreBitsClear(ptr long long)
@ stdcall RtlAreBitsSet(ptr long long)
@ stdcall RtlAssert(ptr ptr long ptr)
@ stdcall -stub -version=0x600+ RtlBarrier(ptr long)
@ stdcall -stub -version=0x600+ RtlBarrierForDelete(long long)
@ stdcall RtlCancelTimer(ptr ptr)
@ stdcall -register RtlCaptureContext(ptr)
@ stdcall RtlCaptureStackBackTrace(long long ptr ptr)
@ stdcall -stub -arch=i386 RtlCaptureStackContext(ptr ptr ptr)
@ stdcall RtlCharToInteger(ptr long ptr)
@ stdcall RtlCheckForOrphanedCriticalSections(ptr)
@ stdcall -stub -version=0x502 RtlCheckProcessParameters(ptr ptr ptr ptr)
@ stdcall RtlCheckRegistryKey(long ptr)
@ stdcall -stub -version=0x600+ RtlCleanUpTEBLangLists()
@ stdcall RtlClearAllBits(ptr)
@ stdcall RtlClearBits(ptr long long)
@ stdcall RtlCloneMemoryStream(ptr ptr)
@ stdcall -stub -version=0x600+ RtlCloneUserProcess(long long long long long)
@ stdcall -ret64 -version=0x600+ RtlCmDecodeMemIoResource(ptr ptr)
@ stdcall -version=0x600+ RtlCmEncodeMemIoResource(ptr long int64 int64)
@ stdcall -stub -version=0x600+ RtlCommitDebugInfo(ptr long)
@ stdcall RtlCommitMemoryStream(ptr long)
@ stdcall RtlCompactHeap(long long)
@ stdcall -stub -version=0x600+ RtlCompareAltitudes(wstr wstr)
@ stdcall RtlCompareMemory(ptr ptr long)
@ stdcall RtlCompareMemoryUlong(ptr long long)
@ stdcall RtlCompareString(ptr ptr long)
@ stdcall RtlCompareUnicodeString (ptr ptr long)
@ stdcall RtlCompareUnicodeStrings(wstr long wstr long long)
@ stdcall -stub -version=0x600+ -arch=x86_64 RtlCompleteProcessCloning(long)
@ stdcall RtlCompressBuffer(long ptr long ptr long long ptr ptr)
@ stdcall RtlComputeCrc32(long ptr long)
@ stdcall RtlComputeImportTableHash(ptr ptr long)
@ stdcall RtlComputePrivatizedDllName_U(ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlConnectToSm(ptr ptr long ptr)
@ stdcall RtlConsoleMultiByteToUnicodeN(ptr long ptr ptr long ptr)
@ stdcall RtlConvertExclusiveToShared(ptr)
@ stdcall -version=0x600+ RtlConvertLCIDToString(long long long ptr long)
@ stdcall -arch=win32 -ret64 RtlConvertLongToLargeInteger(long)
@ stdcall RtlConvertSharedToExclusive(ptr)
@ stdcall RtlConvertSidToUnicodeString(ptr ptr long)
@ stdcall RtlConvertToAutoInheritSecurityObject(ptr ptr ptr ptr long ptr)
@ stdcall RtlConvertUiListToApiList(ptr ptr long)
@ stdcall -arch=win32 -ret64 RtlConvertUlongToLargeInteger(long)
@ stdcall RtlCopyLuid(ptr ptr)
@ stdcall RtlCopyLuidAndAttributesArray(long ptr ptr)
@ stdcall RtlCopyMappedMemory(ptr ptr long)
@ cdecl -arch=x86_64 RtlCopyMemory(ptr ptr long) memmove
@ stdcall -stub -version=0x600+ -arch=x86_64 RtlCopyMemoryNonTemporal(ptr ptr long)
@ stdcall RtlCopyMemoryStreamTo(ptr ptr int64 ptr ptr)
@ stdcall RtlCopyOutOfProcessMemoryStreamTo(ptr ptr int64 ptr ptr) RtlCopyMemoryStreamTo
@ stdcall RtlCopySecurityDescriptor(ptr ptr)
@ stdcall RtlCopySid(long ptr ptr)
@ stdcall RtlCopySidAndAttributesArray(long ptr long ptr ptr ptr ptr)
@ stdcall RtlCopyString(ptr ptr)
@ stdcall RtlCopyUnicodeString(ptr ptr)
@ stdcall RtlCreateAcl(ptr long long)
@ stdcall RtlCreateActivationContext(long ptr long ptr ptr ptr)
@ stdcall RtlCreateAndSetSD(ptr long ptr ptr ptr)
@ stdcall RtlCreateAtomTable(long ptr)
@ stdcall RtlCreateBootStatusDataFile()
@ stdcall -stub -version=0x600+ RtlCreateBoundaryDescriptor(ptr long)
@ stdcall RtlCreateEnvironment(long ptr)
@ stdcall -stub -version=0x600+ RtlCreateEnvironmentEx(ptr ptr long)
@ stdcall RtlCreateHeap(long ptr long long ptr ptr)
@ stdcall -stub -version=0x600+ RtlCreateMemoryBlockLookaside(ptr long long long long)
@ stdcall -stub -version=0x600+ RtlCreateMemoryZone(ptr long long)
@ stdcall RtlCreateProcessParameters(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlCreateProcessParametersEx(ptr wstr ptr wstr wstr ptr wstr wstr ptr ptr long)
@ stdcall RtlCreateQueryDebugBuffer(long long)
@ stdcall RtlCreateRegistryKey(long wstr)
@ stdcall RtlCreateSecurityDescriptor(ptr long)
@ stdcall RtlCreateServiceSid(ptr ptr ptr) # Exists in Windows 2003 SP 2
@ stdcall RtlCreateSystemVolumeInformationFolder(ptr)
@ stdcall RtlCreateTagHeap(ptr long wstr wstr)
@ stdcall RtlCreateTimer(ptr ptr ptr ptr long long long)
@ stdcall RtlCreateTimerQueue(ptr)
@ stdcall RtlCreateUnicodeString(ptr wstr)
@ stdcall RtlCreateUnicodeStringFromAsciiz(ptr str)
@ stdcall RtlCreateUserProcess(ptr long ptr ptr ptr ptr long ptr ptr ptr)
@ stdcall RtlCreateUserSecurityObject(ptr long ptr ptr long ptr ptr)
@ stdcall -stub -version=0x600+ RtlCreateUserStack(long long long long long ptr)
@ stdcall RtlCreateUserThread(long ptr long ptr long long ptr ptr ptr ptr)
@ stdcall RtlCultureNameToLCID(ptr ptr)
@ stdcall RtlCustomCPToUnicodeN(ptr wstr long ptr str long)
@ stdcall RtlCutoverTimeToSystemTime(ptr ptr ptr long)
@ stdcall -stub -version=0x600+ RtlDeCommitDebugInfo(long long long) ; doesn't exist in win11
@ stdcall RtlDeNormalizeProcessParams(ptr)
@ stdcall RtlDeactivateActivationContext(long long)
@ stdcall -arch=x86_64,arm,arm64 RtlDeactivateActivationContextUnsafeFast(ptr)
@ stdcall -stub RtlDebugPrintTimes()
@ stdcall RtlDecodePointer(ptr)
@ stdcall RtlDecodeSystemPointer(ptr)
@ stdcall RtlDecompressBuffer(long ptr long ptr long ptr)
@ stdcall RtlDecompressFragment(long ptr long ptr long long ptr ptr)
@ stdcall RtlDefaultNpAcl(ptr)
@ stdcall RtlDelete(ptr)
@ stdcall RtlDeleteAce(ptr long)
@ stdcall RtlDeleteAtomFromAtomTable(ptr long)
@ stdcall -stub -version=0x600+ RtlDeleteBarrier(long)
@ stdcall -stub -version=0x600+ RtlDeleteBoundaryDescriptor(ptr)
@ stdcall RtlDeleteCriticalSection(ptr)
@ stdcall RtlDeleteElementGenericTable(ptr ptr)
@ stdcall RtlDeleteElementGenericTableAvl(ptr ptr)
@ cdecl -arch=x86_64,arm64 RtlDeleteFunctionTable(ptr)
@ stdcall RtlDeleteNoSplay(ptr ptr)
@ stdcall RtlDeleteRegistryValue(long ptr ptr)
@ stdcall RtlDeleteResource(ptr)
@ stdcall RtlDeleteSecurityObject(ptr)
@ stdcall RtlDeleteTimer(ptr ptr ptr)
@ stdcall RtlDeleteTimerQueue(ptr)
@ stdcall RtlDeleteTimerQueueEx(ptr ptr)
@ stdcall -stub -version=0x600+ RtlDeregisterSecureMemoryCacheCallback(ptr)
@ stdcall RtlDeregisterWait(ptr)
@ stdcall RtlDeregisterWaitEx(ptr ptr)
@ stdcall RtlDestroyAtomTable(ptr)
@ stdcall RtlDestroyEnvironment(ptr)
@ stdcall RtlDestroyHandleTable(ptr)
@ stdcall RtlDestroyHeap(long)
@ stdcall -stub -version=0x600+ RtlDestroyMemoryBlockLookaside(long)
@ stdcall -stub -version=0x600+ RtlDestroyMemoryZone(long)
@ stdcall RtlDestroyProcessParameters(ptr)
@ stdcall RtlDestroyQueryDebugBuffer(ptr)
@ stdcall RtlDetermineDosPathNameType_U(wstr)
@ stdcall RtlDllShutdownInProgress()
@ stdcall RtlDnsHostNameToComputerName(ptr ptr long)
@ stdcall RtlDoesFileExists_U(wstr)
@ stdcall RtlDosApplyFileIsolationRedirection_Ustr(long ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall RtlDosPathNameToNtPathName_U(wstr ptr ptr ptr)
@ stdcall RtlDosPathNameToNtPathName_U_WithStatus(wstr ptr ptr ptr) ; 5.2 SP1, and higher
@ stdcall RtlDosPathNameToRelativeNtPathName_U(ptr ptr ptr ptr)
@ stdcall RtlDosPathNameToRelativeNtPathName_U_WithStatus(wstr ptr ptr ptr)
@ stdcall RtlDosSearchPath_U(wstr wstr wstr long ptr ptr)
@ stdcall RtlDosSearchPath_Ustr(long ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall RtlDowncaseUnicodeChar(long)
@ stdcall RtlDowncaseUnicodeString(ptr ptr long)
@ stdcall RtlDumpResource(ptr)
@ stdcall RtlDuplicateUnicodeString(long ptr ptr)
@ stdcall RtlEmptyAtomTable(ptr long)
@ stdcall -stub RtlEnableEarlyCriticalSectionEventCreation()
@ stdcall RtlEncodePointer(ptr)
@ stdcall RtlEncodeSystemPointer(ptr)
@ stdcall -arch=win32 -ret64 RtlEnlargedIntegerMultiply(long long)
@ stdcall -arch=win32 RtlEnlargedUnsignedDivide(double long ptr)
@ stdcall -arch=win32 -ret64 RtlEnlargedUnsignedMultiply(long long)
@ stdcall RtlEnterCriticalSection(ptr)
@ stdcall RtlEnumProcessHeaps(ptr ptr)
@ stdcall RtlEnumerateGenericTable(ptr long)
@ stdcall RtlEnumerateGenericTableAvl(ptr long)
@ stdcall RtlEnumerateGenericTableLikeADirectory(ptr ptr ptr long ptr ptr ptr)
@ stdcall RtlEnumerateGenericTableWithoutSplaying(ptr ptr)
@ stdcall RtlEnumerateGenericTableWithoutSplayingAvl(ptr ptr)
@ stdcall RtlEqualComputerName(ptr ptr)
@ stdcall RtlEqualDomainName(ptr ptr)
@ stdcall RtlEqualLuid(ptr ptr)
@ stdcall RtlEqualPrefixSid(ptr ptr)
@ stdcall RtlEqualSid(long long)
@ stdcall RtlEqualString(ptr ptr long)
@ stdcall RtlEqualUnicodeString(ptr ptr long)
@ stdcall RtlEraseUnicodeString(ptr)
@ stdcall -stub -version=0x600+ RtlExitUserProcess(long)
@ stdcall RtlExitUserThread(long)
@ stdcall RtlExpandEnvironmentStrings(long ptr long ptr long ptr)
@ stdcall RtlExpandEnvironmentStrings_U(ptr ptr ptr ptr)
@ stdcall -version=0x502 RtlExtendHeap(ptr long ptr ptr)
@ stdcall -stub -version=0x600+ RtlExtendMemoryBlockLookaside(long)
@ stdcall -stub -version=0x600+ RtlExtendMemoryZone(long long)
@ stdcall -arch=win32 -ret64 RtlExtendedIntegerMultiply(double long)
@ stdcall -arch=win32,arm64 -ret64 RtlExtendedLargeIntegerDivide(double long ptr)
@ stdcall -arch=win32 -ret64 RtlExtendedMagicDivide(double double long)
@ stdcall RtlFillMemory(ptr long long)
@ stdcall -arch=i386,arm,arm64 RtlFillMemoryUlong(ptr long long)
@ stdcall RtlFinalReleaseOutOfProcessMemoryStream(ptr)
@ stdcall -stub -version=0x600+ RtlFindAceByType(long long ptr)
@ stdcall RtlFindActivationContextSectionGuid(long ptr long ptr ptr)
@ stdcall RtlFindActivationContextSectionString(long ptr long ptr ptr)
@ stdcall RtlFindCharInUnicodeString(long ptr ptr ptr)
@ stdcall RtlFindClearBits(ptr long long)
@ stdcall RtlFindClearBitsAndSet(ptr long long)
@ stdcall RtlFindClearRuns(ptr ptr long long)
@ stdcall -version=0x600+ RtlFindClosestEncodableLength(int64 ptr)
@ stdcall RtlFindLastBackwardRunClear(ptr long ptr)
@ stdcall RtlFindLeastSignificantBit(double)
@ stdcall RtlFindLongestRunClear(ptr long)
@ stdcall RtlFindMessage(long long long long ptr)
@ stdcall RtlFindMostSignificantBit(double)
@ stdcall RtlFindNextForwardRunClear(ptr long ptr)
@ stdcall RtlFindSetBits(ptr long long)
@ stdcall RtlFindSetBitsAndClear(ptr long long)
@ stdcall RtlFirstEntrySList(ptr)
@ stdcall RtlFirstFreeAce(ptr ptr)
@ stdcall -stub -version=0x600+ RtlFlsAlloc(long long long long)
@ stdcall -stub -version=0x600+ RtlFlsFree(long)
@ stdcall RtlFlushSecureMemoryCache(ptr ptr)
@ stdcall RtlFormatCurrentUserKeyPath(ptr)
@ stdcall RtlFormatMessage(ptr long long long long ptr ptr long ptr)
@ stdcall RtlFormatMessageEx(ptr long long long long ptr ptr long ptr long)
@ stdcall RtlFreeActivationContextStack(ptr)
@ stdcall RtlFreeAnsiString(long)
@ stdcall RtlFreeHandle(ptr ptr)
@ stdcall RtlFreeHeap(long long long)
@ stdcall -stub -version=0x600+ RtlFreeMemoryBlockLookaside(long long)
@ stdcall RtlFreeOemString(ptr)
@ stdcall RtlFreeSid(long)
@ stdcall RtlFreeThreadActivationContextStack()
@ stdcall RtlFreeUnicodeString(ptr)
@ stdcall -stub -version=0x600+ RtlFreeUserStack(long)
@ stdcall -version=0x502 RtlFreeUserThreadStack(ptr ptr)
@ stdcall RtlGUIDFromString(ptr ptr)
@ stdcall RtlGenerate8dot3Name(ptr ptr long ptr)
@ stdcall RtlGetAce(ptr long ptr)
@ stdcall RtlGetActiveActivationContext(ptr)
@ stdcall RtlGetCallersAddress(ptr ptr)
@ stdcall RtlGetCompressionWorkSpaceSize(long ptr ptr)
@ stdcall RtlGetControlSecurityDescriptor(ptr ptr ptr)
@ stdcall RtlGetCriticalSectionRecursionCount(ptr)
@ stdcall RtlGetCurrentDirectory_U(long ptr)
@ stdcall RtlGetCurrentPeb()
@ stdcall RtlGetCurrentProcessorNumber() ; 5.2 SP1 and higher
@ stdcall RtlGetCurrentProcessorNumberEx(ptr)
@ stdcall RtlGetCurrentTransaction()
@ stdcall RtlGetDaclSecurityDescriptor(ptr ptr ptr ptr)
@ stdcall RtlGetElementGenericTable(ptr long)
@ stdcall RtlGetElementGenericTableAvl(ptr long)
@ stdcall -stub -version=0x600+ RtlGetFileMUIPath(long long ptr ptr long long ptr)
@ stdcall RtlGetFrame()
@ stdcall RtlGetFullPathName_U(wstr long ptr ptr)
@ stdcall RtlGetFullPathName_UstrEx(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall -arch=x86_64 RtlGetFunctionTableListHead()
@ stdcall RtlGetGroupSecurityDescriptor(ptr ptr ptr)
@ stub -version=0x600+ RtlGetIntegerAtom
@ stdcall RtlGetLastNtStatus()
@ stdcall RtlGetLastWin32Error()
@ stdcall RtlGetLengthWithoutLastFullDosOrNtPathElement(long ptr ptr)
; Yes, Microsoft really misspelled this one!
@ stdcall RtlGetLengthWithoutTrailingPathSeperators(long ptr ptr) RtlGetLengthWithoutTrailingPathSeparators
@ stdcall RtlGetLongestNtPathLength()
@ stdcall RtlGetNativeSystemInformation(long long long long) NtQuerySystemInformation
@ stdcall RtlGetNtGlobalFlags()
@ stdcall RtlGetNtProductType(ptr)
@ stdcall RtlGetNtVersionNumbers(ptr ptr ptr)
@ stdcall RtlGetOwnerSecurityDescriptor(ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlGetParentLocaleName(wstr long long long)
@ stdcall RtlGetProcessHeaps(long ptr)
@ stdcall RtlGetProductInfo(long long long long ptr)
@ stdcall RtlGetSaclSecurityDescriptor(ptr ptr ptr ptr)
@ stdcall RtlGetSecurityDescriptorRMControl(ptr ptr)
@ stdcall RtlGetSetBootStatusData(ptr long long ptr long long)
@ stdcall -stub -version=0x600+ RtlGetSystemPreferredUILanguages(long ptr ptr ptr ptr)
@ stdcall RtlGetThreadErrorMode()
@ stdcall -stub -version=0x600+ RtlGetThreadLangIdByIndex(long long ptr ptr)
@ stdcall -stub -version=0x600+ RtlGetThreadPreferredUILanguages(long long ptr ptr)
@ stdcall -stub -version=0x600+ RtlGetUILanguageInfo(long ptr long ptr ptr)
@ stdcall RtlGetUnloadEventTrace()
@ stdcall -stub -version=0x600+ RtlGetUnloadEventTraceEx(ptr ptr ptr)
@ stdcall RtlGetUserInfoHeap(ptr long ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlGetUserPreferredUILanguages(long ptr ptr ptr ptr)
@ stdcall RtlGetVersion(ptr)
@ stdcall RtlHashUnicodeString(ptr long long ptr)
@ stdcall -stub -version=0x600+ RtlHeapTrkInitialize(ptr)
@ stdcall RtlIdentifierAuthoritySid(ptr)
@ stdcall -stub -version=0x600+ RtlIdnToAscii(long long long long long)
@ stdcall -stub -version=0x600+ RtlIdnToNameprepUnicode(long long long long long)
@ stdcall -stub -version=0x600+ RtlIdnToUnicode(long long long long long)
@ stdcall RtlImageDirectoryEntryToData(ptr long long ptr)
@ stdcall RtlImageNtHeader(long)
@ stdcall RtlImageNtHeaderEx(long ptr double ptr)
@ stdcall RtlImageRvaToSection(ptr long long)
@ stdcall RtlImageRvaToVa(ptr long long ptr)
@ stdcall RtlImpersonateSelf(long)
@ stdcall -stub -version=0x600+ RtlImpersonateSelfEx(long long ptr)
@ stdcall RtlInitAnsiString(ptr str)
@ stdcall RtlInitAnsiStringEx(ptr str)
@ stdcall -stub -version=0x600+ RtlInitBarrier(long long)
@ stdcall RtlInitCodePageTable(ptr ptr)
@ stdcall RtlInitMemoryStream(ptr)
@ stdcall RtlInitNlsTables(ptr ptr ptr ptr)
@ stdcall RtlInitOutOfProcessMemoryStream(ptr)
@ stdcall RtlInitString(ptr str)
@ stdcall RtlInitUnicodeString(ptr wstr)
@ stdcall RtlInitUnicodeStringEx(ptr wstr)
@ stdcall -stub RtlInitializeAtomPackage(ptr)
@ stdcall RtlInitializeBitMap(ptr long long)
@ stdcall -version=0x600+ RtlInitializeConditionVariable(ptr)
@ stdcall -arch=arm64 RtlInitializeConditionVariable(ptr)
@ stdcall RtlInitializeContext(ptr ptr ptr ptr ptr)
@ stdcall RtlInitializeCriticalSection(ptr)
@ stdcall RtlInitializeCriticalSectionAndSpinCount(ptr long)
@ stdcall RtlInitializeCriticalSectionEx(ptr long long)
@ stdcall -stub -version=0x600+ -arch=i386 RtlInitializeExceptionChain(ptr)
@ stdcall RtlInitializeGenericTable(ptr ptr ptr ptr ptr)
@ stdcall RtlInitializeGenericTableAvl(ptr ptr ptr ptr ptr)
@ stdcall RtlInitializeHandleTable(long long ptr)
@ stdcall -stub -version=0x600+ RtlInitializeNtUserPfn(wstr long ptr long wstr ptr)
@ stdcall RtlInitializeRXact(ptr long ptr)
@ stdcall RtlInitializeResource(ptr)
@ stdcall RtlInitializeSListHead(ptr)
@ stdcall -version=0x600+ RtlInitializeSRWLock(ptr)
@ stdcall -arch=arm64 RtlInitializeSRWLock(ptr)
@ stdcall RtlInitializeSid(ptr ptr long)
@ stdcall RtlInsertElementGenericTable(ptr ptr long ptr)
@ stdcall RtlInsertElementGenericTableAvl(ptr ptr long ptr)
@ stdcall RtlInsertElementGenericTableFull(ptr ptr long ptr ptr long)
@ stdcall RtlInsertElementGenericTableFullAvl(ptr ptr long ptr ptr long)
@ stdcall -arch=x86_64,arm64 RtlInstallFunctionTableCallback(double double long ptr ptr ptr)
@ stdcall RtlInt64ToUnicodeString(double long ptr)
@ stdcall RtlIntegerToChar(long long long ptr)
@ stdcall RtlIntegerToUnicodeString(long long ptr)
@ stdcall -arch=win32 -ret64 RtlInterlockedCompareExchange64(ptr double double)
@ stdcall RtlInterlockedFlushSList(ptr)
@ stdcall RtlInterlockedPopEntrySList(ptr)
@ stdcall RtlInterlockedPushEntrySList(ptr ptr)
@ stdcall -arch=x86_64 RtlInterlockedPushListSList(ptr ptr ptr long)
@ stdcall -version=0x602+ -arch=i386 RtlInterlockedPushListSListEx(ptr ptr ptr long)
@ stdcall -version=0x602+ -arch=!i386 RtlInterlockedPushListSListEx(ptr ptr ptr long) RtlInterlockedPushListSList
@ stdcall -ret64 -version=0x600+ RtlIoDecodeMemIoResource(ptr ptr ptr ptr)
@ stdcall -version=0x600+ RtlIoEncodeMemIoResource(ptr long int64 int64 int64 int64)
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
@ stdcall RtlIsActivationContextActive(ptr)
@ stdcall RtlIsCriticalSectionLocked(ptr)
@ stdcall RtlIsCriticalSectionLockedByThread(ptr)
@ stdcall -stub -version=0x600+ RtlIsCurrentThreadAttachExempt()
@ stdcall RtlIsDosDeviceName_U(wstr)
@ stdcall RtlIsGenericTableEmpty(ptr)
@ stdcall RtlIsGenericTableEmptyAvl(ptr)
@ stdcall -version=0x601+ RtlIsNameInExpression(ptr ptr long ptr)
@ stdcall RtlIsNameLegalDOS8Dot3(ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlIsNormalizedString(long ptr long ptr)
@ stdcall RtlIsProcessorFeaturePresent(long)
@ stdcall RtlIsTextUnicode(ptr long ptr)
@ stdcall RtlIsThreadWithinLoaderCallout()
@ stdcall RtlIsValidHandle(ptr ptr)
@ stdcall RtlIsValidIndexHandle(ptr long ptr)
@ stdcall -version=0x600+ RtlIsValidLocaleName(wstr long)
@ stdcall RtlLCIDToCultureName(long ptr)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerAdd(double double)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerArithmeticShift(double long)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerDivide(double double ptr)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerNegate(double)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerShiftLeft(double long)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerShiftRight(double long)
@ stdcall -arch=win32 -ret64 RtlLargeIntegerSubtract(double double)
@ stdcall RtlLargeIntegerToChar(ptr long long ptr)
@ stdcall -version=0x600+ RtlLcidToLocaleName(long ptr long long)
@ stdcall RtlLeaveCriticalSection(ptr)
@ stdcall RtlLengthRequiredSid(long)
@ stdcall RtlLengthSecurityDescriptor(ptr)
@ stdcall RtlLengthSid(ptr)
@ stdcall RtlLocalTimeToSystemTime(ptr ptr)
@ stdcall -version=0x600+ RtlLocaleNameToLcid(wstr ptr long)
@ stdcall RtlLockBootStatusData(ptr)
@ stdcall -stub -version=0x600+ RtlLockCurrentThread()
@ stdcall RtlLockHeap(long)
@ stdcall -stub -version=0x600+ RtlLockMemoryBlockLookaside(long)
@ stdcall RtlLockMemoryStreamRegion(ptr int64 int64 long)
@ stdcall -stub -version=0x600+ RtlLockMemoryZone(long)
@ stdcall -stub -version=0x600+ RtlLockModuleSection(long)
@ stdcall -stub RtlLogStackBackTrace()
@ stdcall RtlLookupAtomInAtomTable(ptr wstr ptr)
@ stdcall RtlLookupElementGenericTable(ptr ptr)
@ stdcall RtlLookupElementGenericTableAvl(ptr ptr)
@ stdcall RtlLookupElementGenericTableFull(ptr ptr ptr long)
@ stdcall RtlLookupElementGenericTableFullAvl(ptr ptr ptr long)
@ stdcall -arch=x86_64,arm64 RtlLookupFunctionEntry(long ptr ptr)
@ stdcall -arch=x86_64 RtlLookupFunctionTable(int64 ptr ptr)
@ stdcall RtlMakeSelfRelativeSD(ptr ptr ptr)
@ stdcall RtlMapGenericMask(long ptr)
@ stdcall RtlMapSecurityErrorToNtStatus(long)
@ stdcall RtlMoveMemory(ptr ptr long)
@ stdcall RtlMultiAppendUnicodeStringBuffer(ptr long ptr)
@ stdcall RtlMultiByteToUnicodeN(ptr long ptr ptr long)
@ stdcall RtlMultiByteToUnicodeSize(ptr str long)
@ stdcall RtlMultipleAllocateHeap(ptr long ptr long ptr)
@ stdcall RtlMultipleFreeHeap(ptr long long ptr)
@ stdcall RtlNewInstanceSecurityObject(long long ptr ptr ptr ptr ptr long ptr ptr)
@ stdcall RtlNewSecurityGrantedAccess(long ptr ptr ptr ptr ptr)
@ stdcall RtlNewSecurityObject(ptr ptr ptr long ptr ptr)
@ stdcall RtlNewSecurityObjectEx(ptr ptr ptr ptr long long ptr ptr)
@ stdcall RtlNewSecurityObjectWithMultipleInheritance(ptr ptr ptr ptr long long long ptr ptr)
@ stdcall RtlNormalizeProcessParams(ptr)
@ stdcall -stub -version=0x600+ RtlNormalizeString(long long long long ptr)
@ stdcall RtlNtPathNameToDosPathName(long ptr ptr ptr) ; CHECKME (last arg)
@ stdcall RtlNtStatusToDosError(long)
@ stdcall RtlNtStatusToDosErrorNoTeb(long)
@ stub -version=0x600+ -arch=x86_64 RtlNtdllName
@ stdcall RtlNumberGenericTableElements(ptr)
@ stdcall RtlNumberGenericTableElementsAvl(ptr)
@ stdcall RtlNumberOfClearBits(ptr)
@ stdcall RtlNumberOfSetBits(ptr)
@ stdcall -stub -version=0x600+ RtlNumberOfSetBitsUlongPtr(long)
@ stdcall RtlOemStringToUnicodeSize(ptr) RtlxOemStringToUnicodeSize
@ stdcall RtlOemStringToUnicodeString(ptr ptr long)
@ stdcall RtlOemToUnicodeN(ptr long ptr ptr long)
@ stdcall RtlOpenCurrentUser(long ptr)
@ stdcall -stub -version=0x600+ RtlOwnerAcesPresent(long)
@ stdcall RtlPcToFileHeader(ptr ptr)
@ stdcall RtlPinAtomInAtomTable(ptr long)
@ stdcall RtlPopFrame(ptr)
@ stdcall RtlPrefixString(ptr ptr long)
@ stdcall RtlPrefixUnicodeString(ptr ptr long)
@ stdcall -stub -version=0x600+ -arch=x86_64 RtlPrepareForProcessCloning()
@ stdcall -stub -version=0x600+ RtlProcessFlsData(long long)
@ stdcall RtlProtectHeap(ptr long)
@ stdcall RtlPushFrame(ptr)
@ stdcall RtlQueryActivationContextApplicationSettings(long ptr wstr wstr ptr ptr ptr)
@ stdcall RtlQueryAtomInAtomTable(ptr long ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlQueryCriticalSectionOwner(ptr)
@ stdcall RtlQueryDepthSList(ptr)
@ stdcall -stub -version=0x600+ RtlQueryDynamicTimeZoneInformation(ptr)
@ stdcall -stub -version=0x600+ RtlQueryElevationFlags(ptr)
@ stdcall RtlQueryEnvironmentVariable(ptr ptr long ptr long ptr)
@ stdcall RtlQueryEnvironmentVariable_U(ptr ptr ptr)
@ stdcall RtlQueryHeapInformation(long long ptr long ptr)
@ stdcall RtlQueryInformationAcl(ptr ptr long long)
@ stdcall RtlQueryInformationActivationContext(long long ptr long ptr long ptr)
@ stdcall RtlQueryInformationActiveActivationContext(long ptr long ptr)
@ stdcall RtlQueryInterfaceMemoryStream(ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlQueryModuleInformation(ptr long ptr)
@ stdcall -stub RtlQueryProcessBackTraceInformation(ptr)
@ stdcall RtlQueryProcessDebugInformation(long long ptr)
@ stdcall RtlQueryProcessHeapInformation(ptr)
@ stdcall -stub RtlQueryProcessLockInformation(ptr)
@ stdcall RtlQueryRegistryValues(long ptr ptr ptr ptr)
@ stdcall RtlQuerySecurityObject(ptr long ptr long ptr)
@ stdcall RtlQueryTagHeap(ptr long long long ptr)
@ stdcall RtlQueryTimeZoneInformation(ptr)
@ stdcall -arch=i386,x86_64 RtlQueueApcWow64Thread(ptr ptr ptr ptr ptr)
@ stdcall RtlQueueWorkItem(ptr ptr long)
@ stdcall -register RtlRaiseException(ptr)
@ stdcall RtlRaiseStatus(long)
@ stdcall RtlRandom(ptr)
@ stdcall RtlRandomEx(ptr)
@ stdcall RtlReAllocateHeap(long long ptr long)
@ stdcall RtlReadMemoryStream(ptr ptr long ptr)
@ stdcall RtlReadOutOfProcessMemoryStream(ptr ptr long ptr)
@ stdcall RtlRealPredecessor(ptr)
@ stdcall RtlRealSuccessor(ptr)
@ stdcall RtlRegisterSecureMemoryCacheCallback(ptr)
@ stdcall -stub -version=0x600+ RtlRegisterThreadWithCsrss()
@ stdcall RtlRegisterWait(ptr ptr ptr ptr long long)
@ stdcall RtlReleaseActivationContext(ptr)
@ stdcall RtlReleaseMemoryStream(ptr)
@ stdcall RtlReleasePebLock()
@ stdcall RtlReleasePrivilege(ptr)
@ stdcall RtlReleaseRelativeName(ptr)
@ stdcall RtlReleaseResource(ptr)
@ stdcall -version=0x600+ RtlReleaseSRWLockExclusive(ptr)
@ stdcall -arch=arm64 RtlReleaseSRWLockExclusive(ptr)
@ stdcall -version=0x600+ RtlReleaseSRWLockShared(ptr)
@ stdcall -arch=arm64 RtlReleaseSRWLockShared(ptr)
@ stdcall RtlRemoteCall(ptr ptr ptr long ptr long long)
@ stdcall RtlRemovePrivileges(ptr ptr long)
@ stdcall RtlRemoveVectoredContinueHandler(ptr)
@ stdcall RtlRemoveVectoredExceptionHandler(ptr)
@ stdcall -stub -version=0x600+ RtlReportException(long long long)
@ stdcall -stub -version=0x600+ RtlResetMemoryBlockLookaside(long)
@ stdcall -stub -version=0x600+ RtlResetMemoryZone(long)
@ stdcall RtlResetRtlTranslations(ptr)
@ stdcall -arch=x86_64,arm64 RtlRestoreContext(ptr ptr)
@ stdcall RtlRestoreLastWin32Error(long) RtlSetLastWin32Error
@ stdcall -stub -version=0x600+ RtlRetrieveNtUserPfn(ptr ptr ptr)
@ stdcall RtlRevertMemoryStream(ptr)
@ stdcall RtlRunDecodeUnicodeString(long ptr)
@ stdcall RtlRunEncodeUnicodeString(long ptr)
@ stdcall -version=0x600+ RtlRunOnceBeginInitialize(ptr long ptr)
@ stdcall -version=0x600+ RtlRunOnceComplete(ptr long ptr)
@ stdcall -version=0x600+ RtlRunOnceExecuteOnce(ptr ptr ptr ptr)
@ stdcall -arch=arm64 RtlRunOnceExecuteOnce(ptr ptr ptr ptr)
@ stdcall -version=0x600+ RtlRunOnceInitialize(ptr)
@ stdcall RtlSecondsSince1970ToTime(long ptr)
@ stdcall RtlSecondsSince1980ToTime(long ptr)
@ stdcall RtlSeekMemoryStream(ptr int64 long ptr)
@ stdcall RtlSelfRelativeToAbsoluteSD2(ptr ptr)
@ stdcall RtlSelfRelativeToAbsoluteSD(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ cdecl RtlSehTryExcept(ptr ptr ptr ptr ptr)
@ cdecl RtlSehTryFinally(ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ RtlSendMsgToSm(ptr ptr)
@ stdcall RtlSetAllBits(ptr)
@ stdcall RtlSetAttributesSecurityDescriptor(ptr long ptr)
@ stdcall RtlSetBits(ptr long long)
@ stdcall RtlSetControlSecurityDescriptor(ptr long long)
@ stdcall RtlSetCriticalSectionSpinCount(ptr long)
@ stdcall RtlSetCurrentDirectory_U(ptr)
@ stdcall RtlSetCurrentEnvironment(wstr ptr)
@ stdcall RtlSetCurrentTransaction(ptr)
@ stdcall RtlSetDaclSecurityDescriptor(ptr long ptr long)
@ stdcall -stub -version=0x600+ RtlSetDynamicTimeZoneInformation(long)
@ stdcall RtlSetEnvironmentStrings(wstr long)
@ stdcall -stub -version=0x600+ RtlSetEnvironmentVar(ptr ptr long ptr long)
@ stdcall RtlSetEnvironmentVariable(ptr ptr ptr)
@ stdcall RtlSetGroupSecurityDescriptor(ptr ptr long)
@ stdcall RtlSetHeapInformation(ptr long ptr ptr)
@ stdcall RtlSetInformationAcl(ptr ptr long long)
@ stdcall RtlSetIoCompletionCallback(long ptr long)
@ stdcall RtlSetLastWin32Error(long)
@ stdcall RtlSetLastWin32ErrorAndNtStatusFromNtStatus(long)
@ stdcall RtlSetMemoryStreamSize(ptr int64)
@ stdcall RtlSetOwnerSecurityDescriptor(ptr ptr long)
@ stdcall -stub -version=0x600+ RtlSetProcessDebugInformation(ptr long ptr)
@ cdecl RtlSetProcessIsCritical(long ptr long)
@ stdcall RtlSetSaclSecurityDescriptor(ptr long ptr long)
@ stdcall RtlSetSecurityDescriptorRMControl(ptr ptr)
@ stdcall RtlSetSecurityObject(long ptr ptr ptr ptr)
@ stdcall RtlSetSecurityObjectEx(long ptr ptr long ptr ptr)
@ stdcall RtlSetThreadErrorMode(long ptr)
@ cdecl RtlSetThreadIsCritical(long ptr long)
@ stdcall RtlSetThreadPoolStartFunc(ptr ptr)
@ stdcall -stub -version=0x600+ RtlSetThreadPreferredUILanguages(long ptr ptr)
@ stdcall RtlSetTimeZoneInformation(ptr)
@ stdcall RtlSetTimer(ptr ptr ptr ptr long long long)
@ stdcall RtlSetUnhandledExceptionFilter(ptr)
@ stdcall -stub -version=0x502 RtlSetUnicodeCallouts(ptr)
@ stdcall RtlSetUserFlagsHeap(ptr long ptr long long)
@ stdcall RtlSetUserValueHeap(ptr long ptr ptr)
@ stdcall -stub -version=0x600+ RtlSidDominates(long long ptr)
@ stdcall -stub -version=0x600+ RtlSidEqualLevel(long long ptr)
@ stdcall -stub -version=0x600+ RtlSidHashInitialize(ptr long ptr)
@ stdcall -stub -version=0x600+ RtlSidHashLookup(long ptr)
@ stdcall -stub -version=0x600+ RtlSidIsHigherLevel(long long ptr)
@ stdcall RtlSizeHeap(long long ptr)
@ stdcall -version=0x600+ RtlSleepConditionVariableCS(ptr ptr ptr)
@ stdcall -version=0x600+ RtlSleepConditionVariableSRW(ptr ptr ptr long)
@ stdcall -arch=arm64 RtlSleepConditionVariableSRW(ptr ptr ptr long)
@ stdcall RtlSplay(ptr)
@ stdcall RtlStartRXact(ptr)
@ stdcall RtlStatMemoryStream(ptr ptr long)
@ stdcall RtlStringFromGUID(ptr ptr)
@ stdcall RtlSubAuthorityCountSid(ptr)
@ stdcall RtlSubAuthoritySid(ptr long)
@ stdcall RtlSubtreePredecessor(ptr)
@ stdcall RtlSubtreeSuccessor(ptr)
@ stdcall RtlSystemTimeToLocalTime(ptr ptr)
@ stdcall RtlTestBit(ptr long)
@ stdcall RtlTimeFieldsToTime(ptr ptr)
@ stdcall RtlTimeToElapsedTimeFields(long long)
@ stdcall RtlTimeToSecondsSince1970(ptr ptr)
@ stdcall RtlTimeToSecondsSince1980(ptr ptr)
@ stdcall RtlTimeToTimeFields (long long)
@ stdcall RtlTraceDatabaseAdd(ptr long ptr ptr)
@ stdcall RtlTraceDatabaseCreate(long ptr long long ptr)
@ stdcall RtlTraceDatabaseDestroy(ptr)
@ stdcall RtlTraceDatabaseEnumerate(ptr ptr ptr)
@ stdcall RtlTraceDatabaseFind(ptr long ptr ptr)
@ stdcall RtlTraceDatabaseLock(ptr)
@ stdcall RtlTraceDatabaseUnlock(ptr)
@ stdcall RtlTraceDatabaseValidate(ptr)
@ stdcall -stub -version=0x600+ RtlTryAcquirePebLock()
@ stdcall RtlTryAcquireSRWLockExclusive(ptr)
@ stdcall RtlTryAcquireSRWLockShared(ptr)
@ stdcall RtlTryEnterCriticalSection(ptr)
@ stdcall RtlUnhandledExceptionFilter2(ptr long)
@ stdcall RtlUnhandledExceptionFilter(ptr)
@ stdcall RtlUnicodeStringToAnsiSize(ptr) RtlxUnicodeStringToAnsiSize
@ stdcall RtlUnicodeStringToAnsiString(ptr ptr long)
@ stdcall RtlUnicodeStringToCountedOemString(ptr ptr long)
@ stdcall RtlUnicodeStringToInteger(ptr long ptr)
@ stdcall RtlUnicodeStringToOemSize(ptr) RtlxUnicodeStringToOemSize
@ stdcall RtlUnicodeStringToOemString(ptr ptr long)
@ stdcall RtlUnicodeToCustomCPN(ptr ptr long ptr wstr long)
@ stdcall RtlUnicodeToMultiByteN(ptr long ptr ptr long)
@ stdcall RtlUnicodeToMultiByteSize(ptr ptr long)
@ stdcall RtlUnicodeToOemN(ptr long ptr ptr long)
@ stdcall RtlUnicodeToUTF8N(ptr long ptr ptr long)
@ stdcall RtlUTF8ToUnicodeN(ptr long ptr ptr long)
@ stdcall RtlUniform(ptr)
@ stdcall RtlUnlockBootStatusData(ptr)
@ stdcall -stub -version=0x600+ RtlUnlockCurrentThread()
@ stdcall RtlUnlockHeap(long)
@ stdcall -stub -version=0x600+ RtlUnlockMemoryBlockLookaside(long)
@ stdcall RtlUnlockMemoryStreamRegion(ptr int64 int64 long)
@ stdcall -stub -version=0x600+ RtlUnlockMemoryZone(long)
@ stdcall -stub -version=0x600+ RtlUnlockModuleSection(long)
@ stdcall -register RtlUnwind(ptr ptr ptr ptr)
@ stdcall -arch=x86_64,arm64 RtlUnwindEx(ptr ptr ptr ptr ptr ptr)
@ stdcall RtlUpcaseUnicodeChar(long)
@ stdcall RtlUpcaseUnicodeString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeStringToAnsiString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeStringToCountedOemString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeStringToOemString(ptr ptr long)
@ stdcall RtlUpcaseUnicodeToCustomCPN(ptr ptr long ptr wstr long)
@ stdcall RtlUpcaseUnicodeToMultiByteN(ptr long ptr ptr long)
@ stdcall RtlUpcaseUnicodeToOemN(ptr long ptr ptr long)
@ stdcall -stub -version=0x600+ RtlUpdateClonedCriticalSection(long)
@ stdcall -stub -version=0x600+ RtlUpdateClonedSRWLock(ptr long)
@ stdcall RtlUpdateTimer(ptr ptr long long)
@ stdcall RtlUpperChar(long)
@ stdcall RtlUpperString(ptr ptr)
@ stdcall -version=0x502 RtlUsageHeap(ptr long ptr)
@ stdcall -stub -version=0x600+ RtlUserThreadStart(long long)
@ stdcall RtlValidAcl(ptr)
@ stdcall RtlValidRelativeSecurityDescriptor(ptr long long)
@ stdcall RtlValidSecurityDescriptor(ptr)
@ stdcall RtlValidSid(ptr)
@ stdcall RtlValidateHeap(long long ptr)
@ stdcall RtlValidateProcessHeaps()
@ stdcall RtlValidateUnicodeString(long ptr)
@ stdcall RtlVerifyVersionInfo(ptr long double)
@ stdcall -arch=x86_64,arm64 RtlVirtualUnwind(long int64 int64 ptr ptr ptr ptr ptr)
@ stdcall -version=0x600+ RtlWaitOnAddress(ptr ptr long ptr)
@ stdcall -arch=arm64 RtlWaitOnAddress(ptr ptr long ptr)
@ stdcall -version=0x600+ RtlWakeAddressAll(ptr)
@ stdcall -arch=arm64 RtlWakeAddressAll(ptr)
@ stdcall -version=0x600+ RtlWakeAddressSingle(ptr)
@ stdcall -arch=arm64 RtlWakeAddressSingle(ptr)
@ stdcall -version=0x600+ RtlWakeAllConditionVariable(ptr)
@ stdcall -arch=arm64 RtlWakeAllConditionVariable(ptr)
@ stdcall -version=0x600+ RtlWakeConditionVariable(ptr)
@ stdcall -arch=arm64 RtlWakeConditionVariable(ptr)
@ stdcall RtlWalkFrameChain(ptr long long)
@ stdcall RtlWalkHeap(long ptr)
@ stdcall -stub -version=0x600+ RtlWerpReportException(long long ptr long long ptr)
@ stdcall -stub -version=0x600+ RtlWow64CallFunction64()
@ stdcall -arch=arm64,x86_64 RtlQueryPerformanceCounter(ptr)
@ stdcall -arch=arm64,x86_64 RtlQueryPerformanceFrequency(ptr)
@ stdcall -arch=arm64,x86_64 RtlWow64GetCurrentCpuArea(ptr ptr ptr)
@ stdcall -arch=arm64,x86_64 RtlWow64GetThreadContext(ptr ptr)
@ stdcall -arch=arm64,x86_64 RtlWow64SetThreadContext(ptr ptr)
@ stdcall -arch=arm64,x86_64 RtlWow64SuspendThread(ptr ptr)
# Wine/Win10 ntdll exports these CRT helpers; the FEX CPU layers' mini-CRT
# imports them (wow64-plan Phase B — our loader poison-snaps missing imports to
# 0xffbadd11, so absence means a PC-alignment crash on first errno write, not
# a load failure). ntdll's OWN (ntdll-rust crtexports.rs; atan2/tan the linked
# CRT math, tan below), NOT forwarders to msvcrt: resolving a forwarder loads
# msvcrt, and msvcrt's imports load kernel32, into every CPU-layer process.
@ cdecl -arch=arm64 _errno()
@ cdecl -arch=arm64 atan2(double double)
@ stdcall RtlWow64EnableFsRedirection(long)
@ stdcall RtlWow64EnableFsRedirectionEx(long ptr)
# WoW64 syscall dispatch slot (i386 only): every user Nt* stub calls through
# this pointer (syscalls.inc STUBCODE_U). Native boots: static fallback to the
# SharedUserData->SystemCall path. Under WoW64 the host wow64.dll patches the
# guest ntdll32's slot to the CPU layer's BOP bridge (Win10-parity export).
@ extern -arch=i386 Wow64Transition
# The win32k callback hook slot (dispatch/mod.rs, KiUserCallbackDispatcher).
# wowx64.dll fills it in an x86-64 guest process, whose callback table is the
# guest user32's x86-64 code. Data export; arm64 only.
@ extern -arch=arm64 WowX64UserCallbackHook
# Its user-APC twin (dispatch.S, KiUserApcDispatcher): an APC routine there
# may be the guest's x86-64 code. Data export; arm64 only.
@ extern -arch=arm64 WowX64UserApcHook
# E5: the KiUserExceptionDispatcher hook slot (dispatch.S). wow64.dll writes its
# guest-SEH dispatch handler here so hardware faults in 32-bit guest code reach
# the guest's own exception dispatcher. Data export; amd64 native WoW64 only.
@ extern -arch=x86_64 Wow64PrepareForException
# Real RtlWow64Get/SetThreadContext/SuspendThread for x86_64 live with the
# arm64 lines above (wow64shim.rs, Phase E) — the old -stub duplicates are gone.
@ stdcall -stub -version=0x600+ -arch=x86_64 RtlWow64LogMessageInEventLogger(long long long)
@ stdcall RtlWriteMemoryStream(ptr ptr long ptr)
@ stdcall RtlWriteRegistryValue(long ptr ptr long ptr long)
@ stdcall RtlZeroHeap(ptr long)
@ stdcall RtlZeroMemory(ptr long)
@ stdcall RtlZombifyActivationContext(ptr)
@ stdcall RtlpApplyLengthFunction(long long ptr ptr)
@ stdcall -stub -version=0x600+ RtlpCheckDynamicTimeZoneInformation(ptr long)
@ stdcall -stub -version=0x600+ RtlpCleanupRegistryKeys()
@ stdcall -stub -version=0x600+ RtlpConvertCultureNamesToLCIDs(wstr ptr)
@ stdcall -stub -version=0x600+ RtlpConvertLCIDsToCultureNames(wstr ptr)
@ stdcall -stub -version=0x600+ RtlpCreateProcessRegistryInfo(ptr)
@ stdcall RtlpEnsureBufferSize(long ptr long)
@ stdcall -stub -version=0x600+ RtlpGetLCIDFromLangInfoNode(long long ptr)
@ stdcall -stub -version=0x600+ RtlpGetNameFromLangInfoNode(long long long)
@ stdcall -stub -version=0x600+ RtlpGetSystemDefaultUILanguage(ptr long)
@ stdcall -stub -version=0x600+ RtlpGetUserOrMachineUILanguage4NLS(long long ptr)
@ stdcall -stub -version=0x600+ RtlpInitializeLangRegistryInfo(ptr)
@ stdcall -stub -version=0x600+ RtlpIsQualifiedLanguage(long ptr long)
@ stdcall -stub -version=0x600+ RtlpLoadMachineUIByPolicy(ptr long ptr)
@ stdcall -stub -version=0x600+ RtlpLoadUserUIByPolicy(ptr long ptr)
@ stdcall -stub -version=0x600+ RtlpMuiFreeLangRegistryInfo(long)
@ stdcall -stub -version=0x600+ RtlpMuiRegCreateRegistryInfo()
@ stdcall -stub -version=0x600+ RtlpMuiRegFreeRegistryInfo(long long)
@ stdcall -stub -version=0x600+ RtlpMuiRegLoadRegistryInfo(long long)
@ stdcall RtlpNotOwnerCriticalSection(ptr)
@ stdcall RtlpNtCreateKey(ptr long ptr long ptr ptr)
@ stdcall RtlpNtEnumerateSubKey(ptr ptr long long)
@ stdcall RtlpNtMakeTemporaryKey(ptr)
@ stdcall RtlpNtOpenKey(ptr long ptr long)
@ stdcall RtlpNtQueryValueKey(ptr ptr ptr ptr long)
@ stdcall RtlpNtSetValueKey(ptr long ptr long)
@ stdcall -stub -version=0x600+ RtlpQueryDefaultUILanguage(ptr long)
@ stdcall -stub -version=0x600+ -arch=x86_64 RtlpQueryProcessDebugInformationFromWow64(long ptr)
@ stdcall -stub -version=0x600+ RtlpRefreshCachedUILanguage(wstr long)
@ stdcall -stub -version=0x600+ RtlpSetInstallLanguage(long ptr)
@ stdcall -stub -version=0x600+ RtlpSetPreferredUILanguages(long ptr ptr) ; x64 only?
@ stdcall -stub -arch=i386 -version=0x600+ RtlpSetUserPreferredUILanguages(long ptr ptr) ; seems to be i386 only
@ stdcall RtlpUnWaitCriticalSection(ptr)
@ stdcall -stub -version=0x600+ RtlpVerifyAndCommitUILanguageSettings(long)
@ stdcall RtlpWaitForCriticalSection(ptr)
@ stdcall RtlxAnsiStringToUnicodeSize(ptr)
@ stdcall RtlxOemStringToUnicodeSize(ptr)
@ stdcall RtlxUnicodeStringToAnsiSize(ptr)
@ stdcall RtlxUnicodeStringToOemSize(ptr)
@ stdcall -stub -version=0x600+ ShipAssert(long long)
@ stdcall -stub -version=0x600+ ShipAssertGetBufferInfo(ptr ptr)
@ stdcall -stub -version=0x600+ ShipAssertMsgA(long long)
@ stdcall -stub -version=0x600+ ShipAssertMsgW(long long)
@ stdcall -stub -version=0x600+ TpAllocAlpcCompletion(ptr ptr ptr ptr ptr)
@ stdcall -version=0x600+ TpAllocCleanupGroup(ptr)
@ stdcall -version=0x600+ TpAllocIoCompletion(ptr ptr ptr ptr ptr)
@ stdcall -version=0x600+ TpAllocPool(ptr ptr)
@ stdcall -version=0x600+ TpAllocTimer(ptr ptr ptr ptr)
@ stdcall -version=0x600+ TpAllocWait(ptr ptr ptr ptr)
@ stdcall -version=0x600+ TpAllocWork(ptr ptr ptr ptr)
@ stdcall -version=0x600+ TpCallbackLeaveCriticalSectionOnCompletion(ptr ptr)
@ stdcall -version=0x600+ TpCallbackMayRunLong(ptr)
@ stdcall -version=0x600+ TpCallbackReleaseMutexOnCompletion(ptr ptr)
@ stdcall -version=0x600+ TpCallbackReleaseSemaphoreOnCompletion(ptr ptr long)
@ stdcall -version=0x600+ TpCallbackSetEventOnCompletion(ptr ptr)
@ stdcall -version=0x600+ TpCallbackUnloadDllOnCompletion(ptr ptr)
@ stdcall -version=0x600+ TpCancelAsyncIoOperation(ptr)
@ stdcall -stub -version=0x600+ TpCaptureCaller(long)
@ stdcall -stub -version=0x600+ TpCheckTerminateWorker(ptr)
@ stdcall -stub -version=0x600+ TpDbgDumpHeapUsage(long ptr long)
@ stdcall -stub -version=0x600+ TpDbgSetLogRoutine()
@ stdcall -version=0x600+ TpDisassociateCallback(ptr)
@ stdcall -version=0x600+ TpIsTimerSet(ptr)
@ stdcall -version=0x600+ TpPostWork(ptr)
@ stdcall -version=0x601+ TpQueryPoolStackInformation(ptr ptr)
@ stdcall -stub -version=0x600+ TpReleaseAlpcCompletion(ptr)
@ stdcall -version=0x600+ TpReleaseCleanupGroup(ptr)
@ stdcall -version=0x600+ TpReleaseCleanupGroupMembers(ptr long ptr)
@ stdcall -version=0x600+ TpReleaseIoCompletion(ptr)
@ stdcall -version=0x600+ TpReleasePool(ptr)
@ stdcall -version=0x600+ TpReleaseTimer(ptr)
@ stdcall -version=0x600+ TpReleaseWait(ptr)
@ stdcall -version=0x600+ TpReleaseWork(ptr)
@ stdcall -version=0x600+ TpSetPoolMaxThreads(ptr long)
@ stdcall -version=0x600+ TpSetPoolMinThreads(ptr long)
@ stdcall -version=0x601+ TpSetPoolStackInformation(ptr ptr)
@ stdcall -version=0x600+ TpSetTimer(ptr ptr long long)
@ stdcall -version=0x600+ TpSetWait(ptr long ptr)
@ stdcall -version=0x600+ TpSimpleTryPost(ptr ptr ptr)
@ stdcall -version=0x600+ TpStartAsyncIoOperation(ptr)
@ stdcall -stub -version=0x600+ TpWaitForAlpcCompletion(ptr)
@ stdcall -version=0x600+ TpWaitForIoCompletion(ptr long)
@ stdcall -version=0x600+ TpWaitForTimer(ptr long)
@ stdcall -version=0x600+ TpWaitForWait(ptr long)
@ stdcall -version=0x600+ TpWaitForWork(ptr long)
@ stdcall -ret64 VerSetConditionMask(double long long)
@ stdcall -stub -version=0x600+ WerCheckEventEscalation(long ptr) ; doesn't exist in win11
@ stdcall -stub -version=0x600+ WerReportSQMEvent(long long long)
@ stdcall -stub -version=0x600+ WerReportWatsonEvent(long long long long) ; doesn't exist in win11
@ stdcall -stub -version=0x600+ WinSqmAddToStream(ptr long long long) ; stub on Win11?
@ stdcall -stub -version=0x600+ WinSqmEndSession(ptr) ; stub on Win11?
@ stdcall -stub -version=0x600+ WinSqmEventEnabled(long ptr) ; stub on Win11?
@ stdcall -stub -version=0x600+ WinSqmEventWrite(long long long) ; stub on Win11?
@ stdcall -stub -version=0x600+ WinSqmIsOptedIn() ; stub on Win11?
@ stdcall -stub -version=0x600+ WinSqmSetString(ptr long ptr) ; stub on Win11?
@ stdcall -stub -version=0x600+ WinSqmStartSession(ptr) ; stub on Win11?
@ stdcall ZwAcceptConnectPort(ptr long ptr long long ptr)
@ stdcall ZwAccessCheck(ptr long long ptr ptr ptr ptr ptr)
@ stdcall ZwAccessCheckAndAuditAlarm(ptr long ptr ptr ptr long ptr long ptr ptr ptr)
@ stdcall ZwAccessCheckByType(ptr ptr ptr long ptr long ptr ptr long ptr ptr)
@ stdcall ZwAccessCheckByTypeAndAuditAlarm(ptr ptr ptr ptr ptr ptr long long long ptr long ptr long ptr ptr ptr)
@ stdcall ZwAccessCheckByTypeResultList(ptr ptr ptr long ptr long ptr ptr long ptr ptr)
@ stdcall ZwAccessCheckByTypeResultListAndAuditAlarm(ptr ptr ptr ptr ptr ptr long long long ptr long ptr long ptr ptr ptr)
@ stdcall ZwAccessCheckByTypeResultListAndAuditAlarmByHandle(ptr ptr ptr ptr ptr ptr ptr long long long ptr long ptr long ptr ptr ptr)
@ stub -version=0x600+ ZwAcquireCMFViewOwnership
@ stdcall ZwAddAtom(ptr long ptr)
@ stdcall ZwAddBootEntry(ptr long)
@ stdcall ZwAddDriverEntry(ptr long)
@ stdcall ZwAdjustGroupsToken(long long long long long long)
@ stdcall ZwAdjustPrivilegesToken(long long long long long long)
@ stdcall ZwAlertResumeThread(long ptr)
@ stdcall ZwAlertThread(long)
@ stdcall ZwAllocateLocallyUniqueId(ptr)
@ stdcall ZwAllocateUserPhysicalPages(ptr ptr ptr)
@ stdcall ZwAllocateUuids(ptr ptr ptr ptr)
@ stdcall ZwAllocateVirtualMemory(long ptr ptr ptr long long)
@ stub -version=0x600+ ZwAlpcAcceptConnectPort
@ stub -version=0x600+ ZwAlpcCancelMessage
@ stub -version=0x600+ ZwAlpcConnectPort
@ stub -version=0x600+ ZwAlpcCreatePort
@ stub -version=0x600+ ZwAlpcCreatePortSection
@ stub -version=0x600+ ZwAlpcCreateResourceReserve
@ stub -version=0x600+ ZwAlpcCreateSectionView
@ stub -version=0x600+ ZwAlpcCreateSecurityContext
@ stub -version=0x600+ ZwAlpcDeletePortSection
@ stub -version=0x600+ ZwAlpcDeleteResourceReserve
@ stub -version=0x600+ ZwAlpcDeleteSectionView
@ stub -version=0x600+ ZwAlpcDeleteSecurityContext
@ stub -version=0x600+ ZwAlpcDisconnectPort
@ stub -version=0x600+ ZwAlpcImpersonateClientOfPort
@ stub -version=0x600+ ZwAlpcOpenSenderProcess
@ stub -version=0x600+ ZwAlpcOpenSenderThread
@ stub -version=0x600+ ZwAlpcQueryInformation
@ stub -version=0x600+ ZwAlpcQueryInformationMessage
@ stub -version=0x600+ ZwAlpcRevokeSecurityContext
@ stub -version=0x600+ ZwAlpcSendWaitReceivePort
@ stub -version=0x600+ ZwAlpcSetInformation
@ stdcall ZwApphelpCacheControl(long ptr)
@ stdcall ZwAreMappedFilesTheSame(ptr ptr)
@ stdcall ZwAssignProcessToJobObject(long long)
@ stdcall ZwCallbackReturn(ptr long long)
@ stdcall ZwCancelDeviceWakeupRequest(ptr)
@ stdcall ZwCancelIoFile(long ptr)
@ stdcall ZwCancelIoFileEx(long ptr ptr)
@ stdcall ZwCancelSynchronousIoFile(long ptr ptr)
@ stdcall ZwCancelTimer(long ptr)
@ stdcall ZwClearEvent(long)
@ stdcall ZwClose(long)
@ stdcall ZwCloseObjectAuditAlarm(ptr ptr long)
@ stdcall -stub -version=0x600+ ZwCommitComplete(ptr ptr)
@ stdcall -stub -version=0x600+ ZwCommitEnlistment(ptr ptr)
@ stdcall ZwCommitTransaction(ptr long) NtCommitTransaction
@ stdcall ZwCompactKeys(long ptr)
@ stdcall ZwCompareObjects(ptr ptr)
@ stdcall ZwCompareTokens(ptr ptr ptr)
@ stdcall ZwCompleteConnectPort(ptr)
@ stdcall ZwCompressKey(ptr)
@ stdcall ZwConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall ZwContinue(ptr long)
@ stdcall ZwCreateDebugObject(ptr long ptr long)
@ stdcall ZwCreateDirectoryObject(long long long)
@ stdcall -stub -version=0x600+ ZwCreateEnlistment(ptr long ptr ptr ptr long long ptr)
@ stdcall ZwCreateEvent(long long long long long)
@ stdcall ZwCreateEventPair(ptr long ptr)
@ stdcall ZwCreateFile(ptr long ptr ptr long long long ptr long long ptr)
@ stdcall ZwCreateIoCompletion(ptr long ptr long)
@ stdcall ZwCreateJobObject(ptr long ptr)
@ stdcall ZwCreateJobSet(long ptr long)
@ stdcall ZwCreateKey(ptr long ptr long ptr long long)
@ stdcall -stub -version=0x600+ ZwCreateKeyTransacted(ptr long ptr long ptr long ptr ptr)
@ stdcall ZwCreateKeyedEvent(ptr long ptr long)
@ stdcall ZwCreateMailslotFile(long long long long long long long long)
@ stdcall ZwCreateMutant(ptr long ptr long)
@ stdcall ZwCreateNamedPipeFile(ptr long ptr ptr long long long long long long long long long ptr)
@ stdcall ZwCreatePagingFile(ptr ptr ptr long)
@ stdcall ZwCreatePort(ptr ptr long long long)
@ stdcall ZwCreateProcess(ptr long ptr ptr long ptr ptr ptr)
@ stdcall ZwCreateProcessEx(ptr long ptr ptr long ptr ptr ptr long)
@ stdcall ZwCreateProfile(ptr ptr ptr long long ptr long long long) ; CHECKME
@ stdcall -stub -version=0x600+ ZwCreateResourceManager(ptr long ptr ptr ptr long wstr)
@ stdcall ZwCreateSection(ptr long ptr ptr long long long)
@ stdcall ZwCreateSemaphore(ptr long ptr long long)
@ stdcall ZwCreateSymbolicLinkObject(ptr long ptr ptr)
@ stdcall ZwCreateThread(ptr long ptr ptr ptr ptr ptr long)
@ stdcall -stub -version=0x600+ ZwCreateThreadEx(ptr long ptr ptr ptr ptr long long long long ptr)
@ stdcall ZwCreateTimer(ptr long ptr long)
@ stdcall ZwCreateToken(ptr long ptr long ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall ZwCreateTransaction(ptr long ptr ptr ptr long long long ptr ptr) NtCreateTransaction
@ stdcall -stub -version=0x600+ ZwCreateTransactionManager(ptr long ptr ptr long long)
@ stdcall -stub -version=0x600+ ZwCreateUserProcess(ptr ptr long long ptr ptr long long ptr ptr ptr)
@ stdcall ZwCreateWaitablePort(ptr ptr long long long)
@ stdcall -stub -version=0x600+ ZwCreateWorkerFactory(ptr long ptr ptr ptr ptr ptr long long long)
@ stdcall ZwDebugActiveProcess(ptr ptr)
@ stdcall ZwDebugContinue(ptr ptr long)
@ stdcall ZwDelayExecution(long ptr)
@ stdcall ZwDeleteAtom(long)
@ stdcall ZwDeleteBootEntry(long)
@ stdcall ZwDeleteDriverEntry(long)
@ stdcall ZwDeleteFile(ptr)
@ stdcall ZwDeleteKey(long)
@ stdcall ZwDeleteObjectAuditAlarm(ptr ptr long)
@ stdcall -stub -version=0x600+ ZwDeletePrivateNamespace(ptr)
@ stdcall ZwDeleteValueKey(long ptr)
@ stdcall ZwDeviceIoControlFile(long long long long long long long long long long)
@ stdcall ZwDisplayString(ptr)
@ stdcall ZwDuplicateObject(long long long ptr long long long)
@ stdcall ZwDuplicateToken(long long long long long long)
@ stdcall ZwEnumerateBootEntries(ptr ptr)
@ stdcall ZwEnumerateDriverEntries(ptr ptr)
@ stdcall ZwEnumerateKey(long long long ptr long ptr)
@ stdcall ZwEnumerateSystemEnvironmentValuesEx(long ptr long)
@ stdcall -stub -version=0x600+ ZwEnumerateTransactionObject(ptr long ptr long ptr)
@ stdcall ZwEnumerateValueKey(long long long ptr long ptr)
@ stdcall ZwExtendSection(ptr ptr)
@ stdcall ZwFilterToken(ptr long ptr ptr ptr ptr)
@ stdcall ZwFindAtom(ptr long ptr)
@ stdcall ZwFlushBuffersFile(long ptr)
@ stdcall -stub -version=0x600+ ZwFlushInstallUILanguage(long long)
@ stdcall ZwFlushInstructionCache(long ptr long)
@ stdcall ZwFlushKey(long)
@ stdcall ZwFlushProcessWriteBuffers()
@ stdcall ZwFlushVirtualMemory(ptr ptr ptr ptr)
@ stdcall ZwFlushWriteBuffer()
@ stdcall ZwFreeUserPhysicalPages(ptr ptr ptr)
@ stdcall ZwFreeVirtualMemory(long ptr ptr long)
@ stdcall -stub -version=0x600+ ZwFreezeRegistry(long)
@ stdcall -stub -version=0x600+ ZwFreezeTransactions(ptr ptr)
@ stdcall ZwFsControlFile(long long long long long long long long long long)
@ stdcall ZwGetContextThread(long ptr)
@ stdcall ZwGetCurrentProcessorNumber()
@ stdcall -version=0xA00+ ZwGetCurrentProcessorNumberEx(ptr)
@ stdcall ZwGetDevicePowerState(ptr ptr)
@ stdcall -stub -version=0x600+ ZwGetMUIRegistryInfo(long ptr ptr)
@ stdcall -stub -version=0x600+ ZwGetNextProcess(ptr long long long ptr)
@ stdcall ZwGetNextThread(ptr ptr long long long ptr) NtGetNextThread
@ stdcall -stub -version=0x600+ ZwGetNlsSectionPtr(long long ptr ptr ptr)
@ stdcall -stub -version=0x600+ ZwGetNotificationResourceManager(ptr ptr long ptr ptr long ptr)
@ stdcall ZwGetPlugPlayEvent(long long ptr long)
@ stdcall ZwGetWriteWatch(long long ptr long ptr ptr ptr)
@ stdcall ZwImpersonateAnonymousToken(ptr)
@ stdcall ZwImpersonateClientOfPort(ptr ptr)
@ stdcall ZwImpersonateThread(ptr ptr ptr)
@ stdcall -stub -version=0x600+ ZwInitializeNlsFiles(ptr ptr ptr ptr)
@ stdcall ZwInitializeRegistry(long)
@ stdcall ZwInitiatePowerAction(long long long long)
@ stdcall ZwIsProcessInJob(long long)
@ stdcall ZwIsSystemResumeAutomatic()
@ stdcall -stub -version=0x600+ ZwIsUILanguageComitted()
@ stdcall ZwListenPort(ptr ptr)
@ stdcall ZwLoadDriver(ptr)
@ stdcall ZwLoadKey2(ptr ptr long)
@ stdcall ZwLoadKey(ptr ptr)
@ stdcall ZwLoadKeyEx(ptr ptr long ptr)
@ stdcall ZwLockFile(long long ptr ptr ptr ptr ptr ptr long long)
@ stdcall ZwLockProductActivationKeys(ptr ptr)
@ stdcall ZwLockRegistryKey(ptr)
@ stdcall ZwLockVirtualMemory(long ptr ptr long)
@ stdcall ZwMakePermanentObject(ptr)
@ stdcall ZwMakeTemporaryObject(long)
@ stdcall -stub -version=0x600+ ZwMapCMFModule(long long ptr ptr ptr)
@ stdcall ZwMapUserPhysicalPages(ptr ptr ptr)
@ stdcall ZwMapUserPhysicalPagesScatter(ptr ptr ptr)
@ stdcall ZwMapViewOfSection(long long ptr long long ptr ptr long long long)
@ stdcall ZwModifyBootEntry(ptr)
@ stdcall ZwModifyDriverEntry(ptr)
@ stdcall ZwNotifyChangeDirectoryFile(long long ptr ptr ptr ptr long long long)
@ stdcall ZwNotifyChangeKey(long long ptr ptr ptr long long ptr long long)
@ stdcall ZwNotifyChangeMultipleKeys(ptr long ptr ptr ptr ptr ptr long long ptr long long)
@ stdcall ZwOpenDirectoryObject(long long long)
@ stdcall -stub -version=0x600+ ZwOpenEnlistment(ptr long ptr ptr ptr)
@ stdcall ZwOpenEvent(long long long)
@ stdcall ZwOpenEventPair(ptr long ptr)
@ stdcall ZwOpenFile(ptr long ptr ptr long long)
@ stdcall ZwOpenIoCompletion(ptr long ptr)
@ stdcall ZwOpenJobObject(ptr long ptr)
@ stdcall ZwOpenKey(ptr long ptr)
@ stdcall -stub -version=0x600+ ZwOpenKeyTransacted(ptr long ptr ptr)
@ stdcall ZwOpenKeyedEvent(ptr long ptr)
@ stdcall ZwOpenMutant(ptr long ptr)
@ stdcall ZwOpenObjectAuditAlarm(ptr ptr ptr ptr ptr ptr long long ptr long long ptr)
@ stdcall -stub -version=0x600+ ZwOpenPrivateNamespace(ptr long ptr ptr)
@ stdcall ZwOpenProcess(ptr long ptr ptr)
@ stdcall ZwOpenProcessToken(long long ptr)
@ stdcall ZwOpenProcessTokenEx(long long long ptr)
@ stdcall -stub -version=0x600+ ZwOpenResourceManager(ptr long ptr ptr ptr)
@ stdcall ZwOpenSection(ptr long ptr)
@ stdcall ZwOpenSemaphore(long long ptr)
@ stdcall -stub -version=0x600+ ZwOpenSession(ptr long ptr)
@ stdcall ZwOpenSymbolicLinkObject (ptr long ptr)
@ stdcall ZwOpenThread(ptr long ptr ptr)
@ stdcall ZwOpenThreadToken(long long long ptr)
@ stdcall ZwOpenThreadTokenEx(long long long long ptr)
@ stdcall ZwOpenTimer(ptr long ptr)
@ stdcall -stub -version=0x600+ ZwOpenTransaction(ptr long ptr ptr ptr)
@ stdcall -stub -version=0x600+ ZwOpenTransactionManager(ptr long ptr wstr ptr long)
@ stdcall ZwPlugPlayControl(ptr ptr long)
@ stdcall ZwPowerInformation(long ptr long ptr long)
@ stdcall -stub -version=0x600+ ZwPrePrepareComplete(ptr ptr)
@ stdcall -stub -version=0x600+ ZwPrePrepareEnlistment(ptr ptr)
@ stdcall -stub -version=0x600+ ZwPrepareComplete(ptr ptr)
@ stdcall -stub -version=0x600+ ZwPrepareEnlistment(ptr ptr)
@ stdcall ZwPrivilegeCheck(ptr ptr ptr)
@ stdcall ZwPrivilegeObjectAuditAlarm(ptr ptr ptr long ptr long)
@ stdcall ZwPrivilegedServiceAuditAlarm(ptr ptr ptr ptr long)
@ stdcall -stub -version=0x600+ ZwPropagationComplete(ptr long long ptr)
@ stdcall -stub -version=0x600+ ZwPropagationFailed(ptr long long)
@ stdcall ZwProtectVirtualMemory(long ptr ptr long ptr)
@ stdcall ZwPulseEvent(long ptr)
@ stdcall ZwQueryAttributesFile(ptr ptr)
@ stdcall ZwQueryBootEntryOrder(ptr ptr)
@ stdcall ZwQueryBootOptions(ptr ptr)
@ stdcall ZwQueryDebugFilterState(long long)
@ stdcall ZwQueryDefaultLocale(long ptr)
@ stdcall ZwQueryDefaultUILanguage(ptr)
@ stdcall ZwQueryDirectoryFile(long long ptr ptr ptr ptr long long long ptr long)
@ stdcall ZwQueryDirectoryObject(long ptr long long long ptr ptr)
@ stdcall ZwQueryDriverEntryOrder(ptr ptr)
@ stdcall ZwQueryEaFile(long ptr ptr long long ptr long ptr long)
@ stdcall ZwQueryEvent(long long ptr long ptr)
@ stdcall ZwQueryFullAttributesFile(ptr ptr)
@ stdcall ZwQueryInformationAtom(long long ptr long ptr)
@ stdcall -stub -version=0x600+ ZwQueryInformationEnlistment(ptr long ptr long ptr)
@ stdcall ZwQueryInformationFile(long ptr ptr long long)
@ stdcall ZwQueryInformationJobObject(long long ptr long ptr)
@ stdcall ZwQueryInformationPort(ptr long ptr long ptr)
@ stdcall ZwQueryInformationProcess(long long ptr long ptr)
@ stdcall -stub -version=0x600+ ZwQueryInformationResourceManager(ptr long ptr long ptr)
@ stdcall ZwQueryInformationThread(long long ptr long ptr)
@ stdcall ZwQueryInformationToken(long long ptr long ptr)
@ stdcall -stub -version=0x600+ ZwQueryInformationTransaction(ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ ZwQueryInformationTransactionManager(ptr)
@ stdcall -stub -version=0x600+ ZwQueryInformationWorkerFactory(ptr long ptr long ptr)
@ stdcall ZwQueryInstallUILanguage(ptr)
@ stdcall ZwQueryIntervalProfile(long ptr)
@ stdcall ZwQueryIoCompletion(long long ptr long ptr)
@ stdcall ZwQueryKey(long long ptr long ptr)
@ stdcall -stub -version=0x600+ ZwQueryLicenseValue(ptr ptr ptr long ptr)
@ stdcall ZwQueryMultipleValueKey(long ptr long ptr long ptr)
@ stdcall ZwQueryMutant(long long ptr long ptr)
@ stdcall ZwQueryObject(long long long long long)
@ stdcall ZwQueryOpenSubKeys(ptr ptr)
@ stdcall ZwQueryOpenSubKeysEx(ptr long ptr ptr)
@ stdcall ZwQueryPerformanceCounter (long long)
@ stdcall ZwQueryPortInformationProcess()
@ stdcall ZwQueryQuotaInformationFile(ptr ptr ptr long long ptr long ptr long)
@ stdcall ZwQuerySection (long long long long long)
@ stdcall ZwQuerySecurityObject (long long long long long)
@ stdcall ZwQuerySemaphore (long long long long long)
@ stdcall ZwQuerySymbolicLinkObject(long ptr ptr)
@ stdcall ZwQuerySystemEnvironmentValue(ptr ptr long ptr)
@ stdcall ZwQuerySystemEnvironmentValueEx(ptr ptr ptr ptr ptr)
@ stdcall ZwQuerySystemInformation(long long long long)
@ stdcall ZwQuerySystemTime(ptr)
@ stdcall ZwQueryTimer(ptr long ptr long ptr)
@ stdcall ZwQueryTimerResolution(long long long)
@ stdcall ZwQueryValueKey(long ptr long ptr long ptr)
@ stdcall ZwQueryVirtualMemory(long ptr long ptr long ptr)
@ stdcall ZwQueryVolumeInformationFile(long ptr ptr long long)
@ stdcall ZwQueueApcThread(long ptr long long long)
@ stdcall ZwRaiseException(ptr ptr long)
@ stdcall ZwRaiseHardError(long long long ptr long ptr)
@ stdcall ZwReadFile(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall ZwReadFileScatter(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall -stub -version=0x600+ ZwReadOnlyEnlistment(ptr ptr)
@ stdcall ZwReadRequestData(ptr ptr long ptr long ptr)
@ stdcall ZwReadVirtualMemory(long ptr ptr long ptr)
@ stdcall -stub -version=0x600+ ZwRecoverEnlistment(ptr ptr)
@ stdcall -stub -version=0x600+ ZwRecoverResourceManager(ptr)
@ stdcall -stub -version=0x600+ ZwRecoverTransactionManager(ptr)
@ stdcall -stub -version=0x600+ ZwRegisterProtocolAddressInformation(ptr ptr long ptr long)
@ stdcall ZwRegisterThreadTerminatePort(ptr)
@ stdcall -stub -version=0x600+ ZwReleaseCMFViewOwnership()
@ stdcall ZwReleaseKeyedEvent(ptr ptr long ptr)
@ stdcall ZwReleaseMutant(long ptr)
@ stdcall ZwReleaseSemaphore(long long ptr)
@ stdcall -stub -version=0x600+ ZwReleaseWorkerFactoryWorker(ptr)
@ stdcall ZwRemoveIoCompletion(ptr ptr ptr ptr ptr)
@ stdcall ZwRemoveIoCompletionEx(ptr ptr long ptr ptr long)
@ stdcall ZwRemoveProcessDebug(ptr ptr)
@ stdcall ZwRenameKey(ptr ptr)
@ stdcall -stub -version=0x600+ ZwRenameTransactionManager(wstr ptr)
@ stdcall ZwReplaceKey(ptr long ptr)
@ stdcall -stub -version=0x600+ ZwReplacePartitionUnit(wstr wstr long)
@ stdcall ZwReplyPort(ptr ptr)
@ stdcall ZwReplyWaitReceivePort(ptr ptr ptr ptr)
@ stdcall ZwReplyWaitReceivePortEx(ptr ptr ptr ptr ptr)
@ stdcall ZwReplyWaitReplyPort(ptr ptr)
@ stdcall ZwRequestDeviceWakeup(ptr)
@ stdcall ZwRequestPort(ptr ptr)
@ stdcall ZwRequestWaitReplyPort(ptr ptr ptr)
@ stdcall ZwRequestWakeupLatency(long)
@ stdcall ZwResetEvent(long ptr)
@ stdcall ZwResetWriteWatch(long ptr long)
@ stdcall ZwRestoreKey(long long long)
@ stdcall ZwResumeProcess(ptr)
@ stdcall ZwResumeThread(long long)
@ stdcall -stub -version=0x600+ ZwRollbackComplete(ptr ptr)
@ stdcall -stub -version=0x600+ ZwRollbackEnlistment(ptr ptr)
@ stdcall ZwRollbackTransaction(ptr long) NtRollbackTransaction
@ stdcall -stub -version=0x600+ ZwRollforwardTransactionManager(ptr ptr)
@ stdcall ZwSaveKey(long long)
@ stdcall ZwSaveKeyEx(ptr ptr long)
@ stdcall ZwSaveMergedKeys(ptr ptr ptr)
@ stdcall ZwSecureConnectPort(ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall ZwSetBootEntryOrder(ptr ptr)
@ stdcall ZwSetBootOptions(ptr long)
@ stdcall ZwSetContextThread(long ptr)
@ stdcall ZwSetDebugFilterState(long long long)
@ stdcall ZwSetDefaultHardErrorPort(ptr)
@ stdcall ZwSetDefaultLocale(long long)
@ stdcall ZwSetDefaultUILanguage(long)
@ stdcall ZwSetDriverEntryOrder(ptr ptr)
@ stdcall ZwSetEaFile(long ptr ptr long)
@ stdcall ZwSetEvent(long long)
@ stdcall ZwSetEventBoostPriority(ptr)
@ stdcall ZwSetHighEventPair(ptr)
@ stdcall ZwSetHighWaitLowEventPair(ptr)
@ stdcall ZwSetInformationDebugObject(ptr long ptr long ptr)
@ stdcall -stub -version=0x600+ ZwSetInformationEnlistment(ptr long ptr long)
@ stdcall ZwSetInformationFile(long long long long long)
@ stdcall ZwSetInformationJobObject(long long ptr long)
@ stdcall ZwSetInformationKey(long long ptr long)
@ stdcall ZwSetInformationObject(long long ptr long)
@ stdcall ZwSetInformationProcess(long long long long)
@ stdcall -stub -version=0x600+ ZwSetInformationResourceManager(ptr long ptr long)
@ stdcall ZwSetInformationThread(long long ptr long)
@ stdcall ZwSetInformationToken(long long ptr long)
@ stdcall -stub -version=0x600+ ZwSetInformationTransaction(ptr long ptr long)
@ stdcall -stub -version=0x600+ ZwSetInformationTransactionManager(ptr long ptr long)
@ stdcall -stub -version=0x600+ ZwSetInformationWorkerFactory(ptr long ptr long)
@ stdcall ZwSetIntervalProfile(long long)
@ stdcall ZwSetIoCompletion(ptr long ptr long long)
@ stdcall ZwSetLdtEntries(long int64 long int64)
@ stdcall ZwSetLowEventPair(ptr)
@ stdcall ZwSetLowWaitHighEventPair(ptr)
@ stdcall ZwSetQuotaInformationFile(ptr ptr ptr long)
@ stdcall ZwSetSecurityObject(long long ptr)
@ stdcall ZwSetSystemEnvironmentValue(ptr ptr)
@ stdcall ZwSetSystemEnvironmentValueEx(ptr ptr ptr ptr ptr)
@ stdcall ZwSetSystemInformation(long ptr long)
@ stdcall ZwSetSystemPowerState(long long long)
@ stdcall ZwSetSystemTime(ptr ptr)
@ stdcall ZwSetThreadExecutionState(long ptr)
@ stdcall ZwSetTimer(long ptr ptr ptr long long ptr)
@ stdcall ZwSetTimerResolution(long long ptr)
@ stdcall ZwSetUuidSeed(ptr)
@ stdcall ZwSetValueKey(long long long long long long)
@ stdcall ZwSetVolumeInformationFile(long ptr ptr long long)
@ stdcall ZwShutdownSystem(long)
@ stdcall -stub -version=0x600+ ZwShutdownWorkerFactory(ptr ptr)
@ stdcall ZwSignalAndWaitForSingleObject(long long long ptr)
@ stdcall -stub -version=0x600+ ZwSinglePhaseReject(ptr ptr)
@ stdcall ZwStartProfile(ptr)
@ stdcall ZwStopProfile(ptr)
@ stdcall ZwSuspendProcess(ptr)
@ stdcall ZwSuspendThread(long ptr)
@ stdcall ZwSystemDebugControl(long ptr long ptr long ptr)
@ stdcall ZwTerminateJobObject(ptr long)
@ stdcall ZwTerminateProcess(ptr long)
@ stdcall ZwTerminateThread(ptr long)
@ stdcall ZwTestAlert()
@ stdcall -stub -version=0x600+ ZwThawRegistry()
@ stdcall -stub -version=0x600+ ZwThawTransactions()
@ stdcall -stub -version=0x600+ ZwTraceControl(long ptr long ptr long long)
@ stdcall ZwTraceEvent(long long long ptr)
@ stdcall ZwTranslateFilePath(ptr long ptr long)
@ stdcall ZwUnloadDriver(ptr)
@ stdcall ZwUnloadKey2(ptr long)
@ stdcall ZwUnloadKey(long)
@ stdcall ZwUnloadKeyEx(ptr ptr)
@ stdcall ZwUnlockFile(long ptr ptr ptr ptr)
@ stdcall ZwUnlockVirtualMemory(long ptr ptr long)
@ stdcall ZwUnmapViewOfSection(long ptr)
@ stdcall ZwVdmControl(long ptr)
@ stdcall ZwWaitForDebugEvent(ptr long ptr ptr)
@ stdcall ZwWaitForKeyedEvent(ptr ptr long ptr)
@ stdcall ZwWaitForMultipleObjects32(long ptr long long ptr)
@ stdcall ZwWaitForMultipleObjects(long ptr long long ptr)
@ stdcall ZwWaitForSingleObject(long long long)
@ stdcall -stub -version=0x600+ ZwWaitForWorkViaWorkerFactory(ptr ptr long ptr ptr)
@ stdcall ZwWaitHighEventPair(ptr)
@ stdcall ZwWaitLowEventPair(ptr)
@ stdcall -stub -version=0x600+ ZwWorkerFactoryWorkerReady(ptr)
@ stdcall ZwWriteFile(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall ZwWriteFileGather(long long ptr ptr ptr ptr long ptr ptr)
@ stdcall ZwWriteRequestData(ptr ptr long ptr long ptr)
@ stdcall ZwWriteVirtualMemory(long ptr ptr long ptr)
@ stdcall ZwYieldExecution()
@ cdecl -arch=i386 _CIcos()
@ cdecl -arch=i386 _CIlog()
@ cdecl -arch=i386 _CIpow()
@ cdecl -arch=i386 _CIsin()
@ cdecl -arch=i386 _CIsqrt()
@ cdecl -arch=x86_64,arm,arm64 __C_specific_handler(ptr long ptr ptr)
@ cdecl -arch=x86_64,arm,arm64 __chkstk()
@ cdecl __isascii(long)
@ cdecl __iscsym(long)
@ cdecl __iscsymf(long)
@ cdecl -arch=arm __jump_unwind()
@ cdecl -stub -version=0x600+ -arch=x86_64 __misaligned_access()
@ cdecl __toascii(long)
@ cdecl -arch=i386 -ret64 _alldiv(double double)
@ cdecl -arch=i386 _alldvrm()
@ cdecl -arch=i386 -ret64 _allmul(double double)
@ cdecl -arch=i386 -norelay -private _alloca_probe()
@ cdecl -arch=i386 -private _alloca_probe_16()
@ stub -version=0x600+ -arch=i386 _alloca_probe_8
@ cdecl -arch=i386 -ret64 _allrem(double double)
@ cdecl -arch=i386 _allshl()
@ cdecl -arch=i386 _allshr()
@ cdecl -ret64 _atoi64(str)
@ cdecl -arch=i386 -ret64 _aulldiv(double double)
@ cdecl -arch=i386 _aulldvrm()
@ cdecl -arch=i386 -ret64 _aullrem(double double)
@ cdecl -arch=i386 _aullshr()
@ extern -arch=i386 _chkstk
@ cdecl -arch=i386,x86_64,arm _fltused()
@ cdecl -arch=i386 -ret64 _ftol()
@ cdecl _i64toa(double ptr long)
@ cdecl _i64tow(double ptr long)
@ cdecl _itoa(long ptr long)
@ cdecl _itow(long ptr long)
@ cdecl _lfind(ptr ptr ptr long ptr)
@ cdecl -arch=x86_64 _local_unwind()
@ cdecl _ltoa(long ptr long)
@ cdecl _ltow(long ptr long)
@ cdecl _memccpy(ptr ptr long long)
@ cdecl _memicmp(str str long)
@ cdecl -arch=x86_64,arm _setjmp(ptr ptr)
@ cdecl -arch=x86_64,arm _setjmpex(ptr ptr)
@ varargs _snprintf(ptr long str)
@ varargs _snwprintf(ptr long wstr)
@ cdecl _splitpath(str ptr ptr ptr ptr)
@ cdecl _strcmpi(str str) _stricmp
@ cdecl _stricmp(str str)
@ cdecl _strlwr(str)
@ cdecl _strnicmp(str str long)
@ cdecl _strupr(str)
@ cdecl -version=0x400-0x502 -impsym _swprintf() swprintf # Compatibility for pre NT6
@ cdecl -version=0x600+ _swprintf(ptr str)
@ cdecl -version=0x502 _tolower(long)
@ cdecl -version=0x502 _toupper(long)
@ cdecl _ui64toa(double ptr long)
@ cdecl _ui64tow(double ptr long)
@ cdecl _ultoa(long ptr long)
@ cdecl _ultow(long ptr long)
@ cdecl _vscwprintf(wstr ptr)
@ cdecl _vsnprintf(ptr long str ptr)
@ cdecl _vsnwprintf(ptr long wstr ptr)
@ cdecl -stub -version=0x600+ _vswprintf(ptr wstr ptr)
@ cdecl _wcsicmp(wstr wstr)
@ cdecl _wcslwr(wstr)
@ cdecl _wcsnicmp(wstr wstr long)
@ cdecl _wcstoui64(wstr ptr long)
@ cdecl _wcsupr(wstr)
@ cdecl _wtoi(wstr)
@ cdecl _wtoi64(wstr)
@ cdecl _wtol(wstr)
@ cdecl abs(long)
@ cdecl -arch=i386,x86_64 atan(double)
@ cdecl atoi(str)
@ cdecl atol(str)
@ cdecl bsearch(ptr ptr long long ptr)
@ cdecl ceil(double)
@ cdecl cos(double)
@ cdecl fabs(double)
@ cdecl floor(double)
@ cdecl isalnum(long)
@ cdecl isalpha(long)
@ cdecl iscntrl(long)
@ cdecl isdigit(long)
@ cdecl isgraph(long)
@ cdecl islower(long)
@ cdecl isprint(long)
@ cdecl ispunct(long)
@ cdecl isspace(long)
@ cdecl isupper(long)
@ cdecl iswalpha(long)
@ cdecl iswctype(long long)
@ cdecl iswdigit(long)
@ cdecl iswlower(long)
@ cdecl iswspace(long)
@ cdecl iswxdigit(long)
@ cdecl isxdigit(long)
@ cdecl labs(long)
@ cdecl -arch=i386,x86_64 log(double)
@ cdecl -arch=x86_64 longjmp(ptr)
@ cdecl mbstowcs(ptr str long)
@ cdecl memchr(ptr long long)
@ cdecl memcmp(ptr ptr long)
@ cdecl memcpy(ptr ptr long) memmove
@ cdecl memmove(ptr ptr long)
@ cdecl memset(ptr long long)
@ cdecl -arch=i386,x86_64,arm pow(double double)
@ cdecl qsort(ptr long long ptr)
@ cdecl sin(double)
@ varargs sprintf(ptr str)
@ cdecl -arch=i386,x86_64 sqrt(double)
@ varargs sscanf(str str)
@ cdecl strcat(str str)
@ cdecl strchr(str long)
@ cdecl strcmp(str str)
@ cdecl strcpy(ptr str)
@ cdecl strcspn(str str)
@ cdecl strlen(str)
@ cdecl strncat(str str long)
@ cdecl strncmp(str str long)
@ cdecl strncpy(ptr str long)
@ cdecl strpbrk(str str)
@ cdecl strrchr(str long)
@ cdecl strspn(str str)
@ cdecl strstr(str str)
@ cdecl strtol(str ptr long)
@ cdecl strtoul(str ptr long)
@ varargs swprintf(ptr wstr) _swprintf # Non-conforming swprintf
@ cdecl tan(double)
@ cdecl tolower(long)
@ cdecl toupper(long) toupper_nt_mb
@ cdecl towlower(long)
@ cdecl towupper(long)
@ stdcall vDbgPrintEx(long long str ptr)
@ stdcall vDbgPrintExWithPrefix(str long long str ptr)
@ cdecl vsprintf(ptr str ptr)
@ cdecl wcscat(wstr wstr)
@ cdecl wcschr(wstr long)
@ cdecl wcscmp(wstr wstr)
@ cdecl wcscpy(ptr wstr)
@ cdecl wcscspn(wstr wstr)
@ cdecl wcslen(wstr)
@ cdecl wcsncat(wstr wstr long)
@ cdecl wcsncmp(wstr wstr long)
@ cdecl wcsncpy(ptr wstr long)
@ cdecl wcspbrk(wstr wstr)
@ cdecl wcsrchr(wstr long)
@ cdecl wcsspn(wstr wstr)
@ cdecl wcsstr(wstr wstr)
@ cdecl wcstol(wstr ptr long)
@ cdecl wcstombs(ptr ptr long)
@ cdecl wcstoul(wstr ptr long)

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
@ stub -version=0xA00+ AlpcRundownCompletionList
@ stub -version=0xA00+ ApiSetGetImplementationHost
@ stub -version=0xA00+ ApiSetQueryApiSetPresence
@ stub -version=0xA00+ ApiSetQueryApiSetPresenceEx
@ stub -version=0xA00+ ApiSetQuerySchema
@ stub -version=0xA00+ DbgUiConvertStateChangeStructureEx
@ stub -version=0xA00+ EtwCheckCoverage
@ stub -version=0xA00+ EtwEventWriteNoRegistration
@ stub -version=0xA00+ EvtIntReportAuthzEventAndSourceAsync
@ stub -version=0xA00+ EvtIntReportEventAndSourceAsync
@ stub -version=0xA00+ LdrAddDllDirectory
@ stub -version=0xA00+ LdrAppxHandleIntegrityFailure
@ stub -version=0xA00+ LdrCallEnclave
@ stub -version=0xA00+ LdrControlFlowGuardEnforced
@ stub -version=0xA00+ LdrCreateEnclave
@ stub -version=0xA00+ LdrDeleteEnclave
@ stub -version=0xA00+ LdrFastFailInLoaderCallout
@ stub -version=0xA00+ LdrGetDllDirectory
@ stub -version=0xA00+ LdrGetDllHandleByMapping
@ stub -version=0xA00+ LdrGetDllHandleByName
@ stub -version=0xA00+ LdrGetDllPath
@ stub -version=0xA00+ LdrGetProcedureAddressForCaller
@ stub -version=0xA00+ LdrHotPatchNotify
@ stub -version=0xA00+ LdrInitializeEnclave
@ stub -version=0xA00+ LdrIsModuleSxsRedirected
@ stub -version=0xA00+ LdrLoadEnclaveModule
@ stub -version=0xA00+ LdrProcessRelocationBlockEx
@ stub -version=0xA00+ LdrQueryOptionalDelayLoadedAPI
@ stub -version=0xA00+ LdrRemoveDllDirectory
@ stub -version=0xA00+ LdrResGetRCConfig
@ stub -version=0xA00+ LdrResolveDelayLoadedAPI
@ stub -version=0xA00+ LdrResolveDelayLoadsFromDll
@ stub -version=0xA00+ LdrRscIsTypeExist
@ stub -version=0xA00+ LdrSetDefaultDllDirectories
@ stub -version=0xA00+ LdrSetDllDirectory
@ stub -version=0xA00+ LdrSetImplicitPathOptions
@ stub -version=0xA00+ LdrStandardizeSystemPath
@ stub -version=0xA00+ LdrUpdatePackageSearchPath
@ stub -version=0xA00+ MicrosoftTelemetryAssertTriggeredUM
@ stub -version=0xA00+ NtAcquireCrossVmMutant
@ stub -version=0xA00+ NtAcquireProcessActivityReference
@ stub -version=0xA00+ NtAddAtomEx
@ stub -version=0xA00+ NtAdjustTokenClaimsAndDeviceGroups
@ stub -version=0xA00+ NtAlertMultipleThreadByThreadId
@ stub -version=0xA00+ NtAlertThreadByThreadId
@ stub -version=0xA00+ NtAlertThreadByThreadIdEx
@ stub -version=0xA00+ NtAllocateReserveObject
@ stub -version=0xA00+ NtAllocateUserPhysicalPagesEx
@ stub -version=0xA00+ NtAlpcConnectPortEx
@ stub -version=0xA00+ NtAlpcImpersonateClientContainerOfPort
@ stub -version=0xA00+ NtAssociateWaitCompletionPacket
@ stub -version=0xA00+ NtCallEnclave
@ stub -version=0xA00+ NtCancelTimer2
@ stub -version=0xA00+ NtCancelWaitCompletionPacket
@ stub -version=0xA00+ NtChangeProcessState
@ stub -version=0xA00+ NtChangeThreadState
@ stub -version=0xA00+ NtCommitRegistryTransaction
@ stub -version=0xA00+ NtCompareSigningLevels
@ stub -version=0xA00+ NtContinueEx
@ stub -version=0xA00+ NtConvertBetweenAuxiliaryCounterAndPerformanceCounter
@ stub -version=0xA00+ NtCopyFileChunk
@ stub -version=0xA00+ NtCreateCpuPartition
@ stub -version=0xA00+ NtCreateCrossVmEvent
@ stub -version=0xA00+ NtCreateCrossVmMutant
@ stub -version=0xA00+ NtCreateDirectoryObjectEx
@ stub -version=0xA00+ NtCreateEnclave
@ stub -version=0xA00+ NtCreateIRTimer
@ stub -version=0xA00+ NtCreateIoRing
@ stub -version=0xA00+ NtCreateLowBoxToken
@ stub -version=0xA00+ NtCreatePartition
@ stub -version=0xA00+ NtCreateProcessStateChange
@ stub -version=0xA00+ NtCreateProfileEx
@ stub -version=0xA00+ NtCreateRegistryTransaction
@ stub -version=0xA00+ NtCreateSectionEx
@ stub -version=0xA00+ NtCreateThreadStateChange
@ stub -version=0xA00+ NtCreateTimer2
@ stub -version=0xA00+ NtCreateTokenEx
@ stub -version=0xA00+ NtCreateWaitCompletionPacket
@ stub -version=0xA00+ NtCreateWnfStateName
@ stub -version=0xA00+ NtDeleteWnfStateData
@ stub -version=0xA00+ NtDeleteWnfStateName
@ stub -version=0xA00+ NtDirectGraphicsCall
@ stub -version=0xA00+ NtDisableLastKnownGood
@ stub -version=0xA00+ NtDrawText
@ stub -version=0xA00+ NtEnableLastKnownGood
@ stub -version=0xA00+ NtFilterBootOption
@ stub -version=0xA00+ NtFilterTokenEx
@ stub -version=0xA00+ NtFlushBuffersFileEx
@ stub -version=0xA00+ NtGetCachedSigningLevel
@ stub -version=0xA00+ NtGetCompleteWnfStateSubscription
@ stub -version=0xA00+ NtInitializeEnclave
@ stub -version=0xA00+ NtLoadEnclaveData
@ stub -version=0xA00+ NtLoadKey3
@ stub -version=0xA00+ NtManageHotPatch
@ stub -version=0xA00+ NtManagePartition
@ stub -version=0xA00+ NtMapViewOfSectionEx
@ stub -version=0xA00+ NtNotifyChangeDirectoryFileEx
@ stub -version=0xA00+ NtNotifyChangeSession
@ stub -version=0xA00+ NtOpenCpuPartition
@ stub -version=0xA00+ NtOpenKeyTransactedEx
@ stub -version=0xA00+ NtOpenPartition
@ stub -version=0xA00+ NtOpenRegistryTransaction
@ stub -version=0xA00+ NtPssCaptureVaSpaceBulk
@ stub -version=0xA00+ NtQueryAuxiliaryCounterFrequency
@ stub -version=0xA00+ NtQueryDirectoryFileEx
@ stub -version=0xA00+ NtQueryInformationByName
@ stub -version=0xA00+ NtQueryInformationCpuPartition
@ stub -version=0xA00+ NtQueryIoRingCapabilities
@ stub -version=0xA00+ NtQuerySecurityAttributesToken
@ stub -version=0xA00+ NtQuerySecurityPolicy
@ stub -version=0xA00+ NtQueryWnfStateData
@ stub -version=0xA00+ NtQueryWnfStateNameInformation
@ stub -version=0xA00+ NtQueueApcThreadEx
@ stub -version=0xA00+ NtQueueApcThreadEx2
@ stub -version=0xA00+ NtReadVirtualMemoryEx
@ stub -version=0xA00+ NtRevertContainerImpersonation
@ stub -version=0xA00+ NtRollbackRegistryTransaction
@ stub -version=0xA00+ NtSerializeBoot
@ stub -version=0xA00+ NtSetCachedSigningLevel
@ stub -version=0xA00+ NtSetCachedSigningLevel2
@ stub -version=0xA00+ NtSetEventEx
@ stub -version=0xA00+ NtSetIRTimer
@ stub -version=0xA00+ NtSetInformationCpuPartition
@ stub -version=0xA00+ NtSetInformationIoRing
@ stub -version=0xA00+ NtSetInformationSymbolicLink
@ stub -version=0xA00+ NtSetInformationVirtualMemory
@ stub -version=0xA00+ NtSetIoCompletionEx
@ stub -version=0xA00+ NtSetTimer2
@ stub -version=0xA00+ NtSetTimerEx
@ stub -version=0xA00+ NtSetWnfProcessNotificationEvent
@ stub -version=0xA00+ NtSubmitIoRing
@ stub -version=0xA00+ NtSubscribeWnfStateChange
@ stub -version=0xA00+ NtTerminateEnclave
@ stub -version=0xA00+ NtUmsThreadYield
@ stub -version=0xA00+ NtUnmapViewOfSectionEx
@ stub -version=0xA00+ NtUnsubscribeWnfStateChange
@ stub -version=0xA00+ NtUpdateWnfStateData
@ stub -version=0xA00+ NtWaitForAlertByThreadId
@ stub -arch=win32 -version=0xA00+ NtWow64AllocateVirtualMemory64
@ stub -arch=win32 -version=0xA00+ NtWow64CallFunction64
@ stub -arch=win32 -version=0xA00+ NtWow64CsrAllocateCaptureBuffer
@ stub -arch=win32 -version=0xA00+ NtWow64CsrAllocateMessagePointer
@ stub -arch=win32 -version=0xA00+ NtWow64CsrCaptureMessageBuffer
@ stub -arch=win32 -version=0xA00+ NtWow64CsrCaptureMessageString
@ stub -arch=win32 -version=0xA00+ NtWow64CsrClientCallServer
@ stub -arch=win32 -version=0xA00+ NtWow64CsrClientConnectToServer
@ stub -arch=win32 -version=0xA00+ NtWow64CsrFreeCaptureBuffer
@ stub -arch=win32 -version=0xA00+ NtWow64CsrGetProcessId
@ stub -arch=win32 -version=0xA00+ NtWow64CsrIdentifyAlertableThread
@ stub -arch=win32 -version=0xA00+ NtWow64CsrVerifyRegion
@ stub -arch=win32 -version=0xA00+ NtWow64DebuggerCall
@ stub -arch=win32 -version=0xA00+ NtWow64GetCurrentProcessorNumber
@ stub -arch=win32 -version=0xA00+ NtWow64GetCurrentProcessorNumberEx
@ stub -arch=win32 -version=0xA00+ NtWow64GetNativeSystemInformation
@ stub -arch=win32 -version=0xA00+ NtWow64IsProcessorFeaturePresent
@ stub -arch=win32 -version=0xA00+ NtWow64QueryInformationProcess64
@ stub -arch=win32 -version=0xA00+ NtWow64ReadVirtualMemory64
@ stub -arch=win32 -version=0xA00+ NtWow64WriteVirtualMemory64
@ stub -version=0xA00+ PssNtCaptureSnapshot
@ stub -version=0xA00+ PssNtDuplicateSnapshot
@ stub -version=0xA00+ PssNtFreeRemoteSnapshot
@ stub -version=0xA00+ PssNtFreeSnapshot
@ stub -version=0xA00+ PssNtFreeWalkMarker
@ stub -version=0xA00+ PssNtQuerySnapshot
@ stub -version=0xA00+ PssNtValidateDescriptor
@ stub -version=0xA00+ PssNtWalkSnapshot
@ stub -version=0xA00+ RtlAcquireReleaseSRWLockExclusive
@ stub -version=0xA00+ RtlAddAccessFilterAce
@ stub -arch=win64 -version=0xA00+ RtlAddGrowableFunctionTable
@ stub -version=0xA00+ RtlAddIntegrityLabelToBoundaryDescriptor
@ stub -version=0xA00+ RtlAddProcessTrustLabelAce
@ stub -version=0xA00+ RtlAddResourceAttributeAce
@ stub -version=0xA00+ RtlAddScopedPolicyIDAce
@ stub -version=0xA00+ RtlAllocateAndInitializeSidEx
@ stub -version=0xA00+ RtlAllocateWnfSerializationGroup
@ stub -version=0xA00+ RtlAppxIsFileOwnedByTrustedInstaller
@ stub -arch=win64 -version=0xA00+ RtlAreBitsClearEx
@ stub -version=0xA00+ RtlAreLongPathsEnabled
@ stub -version=0xA00+ RtlAvlInsertNodeEx
@ stub -version=0xA00+ RtlAvlRemoveNode
@ stub -arch=win64 -version=0xA00+ RtlCallEnclave
@ stub -arch=win64 -version=0xA00+ RtlCallEnclaveReturn
@ stub -version=0xA00+ RtlCanonicalizeDomainName
@ stub -version=0xA00+ RtlCapabilityCheck
@ stub -version=0xA00+ RtlCapabilityCheckForSingleSessionSku
@ stub -arch=win64 -version=0xA00+ RtlCaptureContext2
@ stub -version=0xA00+ RtlCheckBootStatusIntegrity
@ stub -version=0xA00+ RtlCheckPortableOperatingSystem
@ stub -version=0xA00+ RtlCheckSandboxedToken
@ stub -version=0xA00+ RtlCheckSystemBootStatusIntegrity
@ stub -version=0xA00+ RtlCheckTokenCapability
@ stub -version=0xA00+ RtlCheckTokenMembership
@ stub -version=0xA00+ RtlCheckTokenMembershipEx
@ stub -arch=win64 -version=0xA00+ RtlClearAllBitsEx
@ stub -version=0xA00+ RtlClearBit
@ stub -arch=win64 -version=0xA00+ RtlClearBitEx
@ stub -arch=win64 -version=0xA00+ RtlClearBitsEx
@ stub -version=0xA00+ RtlClearThreadWorkOnBehalfTicket
@ stub -version=0xA00+ RtlCompareExchangePointerMapping
@ stub -version=0xA00+ RtlCompareExchangePropertyStore
@ stub -version=0xA00+ RtlConstructCrossVmEventPath
@ stub -version=0xA00+ RtlConstructCrossVmMutexPath
@ stub -version=0xA00+ RtlContractHashTable
@ stub -version=0xA00+ RtlConvertDeviceFamilyInfoToString
@ stub -version=0xA00+ RtlConvertHostPerfCounterToPerfCounter
@ stub -version=0xA00+ RtlConvertSRWLockExclusiveToShared
@ stub -version=0xA00+ RtlCopyBitMap
@ stub -version=0xA00+ RtlCopyContext
@ stub -version=0xA00+ RtlCopyExtendedContext
@ stub -version=0xA00+ RtlCrc32
@ stub -version=0xA00+ RtlCrc64
@ stub -version=0xA00+ RtlCreateHashTable
@ stub -version=0xA00+ RtlCreateHashTableEx
@ stub -version=0xA00+ RtlCreateProcessParametersWithTemplate
@ stub -version=0xA00+ RtlCreateProcessReflection
@ stub -arch=win64 -version=0xA00+ RtlCreateUmsCompletionList
@ stub -arch=win64 -version=0xA00+ RtlCreateUmsThreadContext
@ stub -arch=win64 -version=0xA00+ RtlCreateUserFiberShadowStack
@ stub -version=0xA00+ RtlCreateUserProcessEx
@ stub -version=0xA00+ RtlCreateVirtualAccountSid
@ stub -version=0xA00+ RtlDecodeRemotePointer
@ stub -version=0xA00+ RtlDecompressBufferEx
@ stub -version=0xA00+ RtlDelayExecution
@ stub -version=0xA00+ RtlDeleteElementGenericTableAvlEx
@ stub -arch=win64 -version=0xA00+ RtlDeleteGrowableFunctionTable
@ stub -version=0xA00+ RtlDeleteHashTable
@ stub -arch=win64 -version=0xA00+ RtlDeleteUmsCompletionList
@ stub -arch=win64 -version=0xA00+ RtlDeleteUmsThreadContext
@ stub -arch=win64 -version=0xA00+ RtlDequeueUmsCompletionListItems
@ stub -version=0xA00+ RtlDeriveCapabilitySidsFromName
@ stub -version=0xA00+ RtlDetectHeapLeaks
@ stub -version=0xA00+ RtlDisableThreadProfiling
@ stub -version=0xA00+ RtlDisownModuleHeapAllocation
@ stub -arch=win32 -version=0xA00+ RtlDispatchAPC
@ stub -version=0xA00+ RtlDoesNameContainWildCards
@ stub -version=0xA00+ RtlDosLongPathNameToNtPathName_U_WithStatus
@ stub -version=0xA00+ RtlDosLongPathNameToRelativeNtPathName_U_WithStatus
@ stub -arch=win64 -version=0xA00+ RtlDrainNonVolatileFlush
@ stub -version=0xA00+ RtlEnableThreadProfiling
@ stub -arch=win64 -version=0xA00+ RtlEnclaveCallDispatch
@ stub -arch=win64 -version=0xA00+ RtlEnclaveCallDispatchReturn
@ stub -version=0xA00+ RtlEncodeRemotePointer
@ stub -version=0xA00+ RtlEndEnumerationHashTable
@ stub -version=0xA00+ RtlEndStrongEnumerationHashTable
@ stub -version=0xA00+ RtlEndWeakEnumerationHashTable
@ stub -arch=win64 -version=0xA00+ RtlEnterUmsSchedulingMode
@ stub -version=0xA00+ RtlEnumerateEntryHashTable
@ stub -version=0xA00+ RtlEqualWnfChangeStamps
@ stub -version=0xA00+ RtlEthernetAddressToStringA
@ stub -version=0xA00+ RtlEthernetAddressToStringW
@ stub -version=0xA00+ RtlEthernetStringToAddressA
@ stub -version=0xA00+ RtlEthernetStringToAddressW
@ stub -arch=win64 -version=0xA00+ RtlExecuteUmsThread
@ stub -version=0xA00+ RtlExpandHashTable
@ stub -version=0xA00+ RtlExtendCorrelationVector
@ stub -version=0xA00+ RtlExtractBitMap
@ stub -arch=win64 -version=0xA00+ RtlFillMemoryNonTemporal
@ stub -arch=win32 -version=0xA00+ RtlFillMemoryUlonglong
@ stub -arch=win64 -version=0xA00+ RtlFillNonVolatileMemory
@ stub -arch=win64 -version=0xA00+ RtlFindClearBitsAndSetEx
@ stub -arch=win64 -version=0xA00+ RtlFindClearBitsEx
@ stub -version=0xA00+ RtlFindExportedRoutineByName
@ stub -arch=win64 -version=0xA00+ RtlFindSetBitsAndClearEx
@ stub -arch=win64 -version=0xA00+ RtlFindSetBitsEx
@ stub -version=0xA00+ RtlFindUnicodeSubstring
@ stub -version=0xA00+ RtlFlsAllocEx
@ stub -version=0xA00+ RtlFlsGetValue
@ stub -version=0xA00+ RtlFlsGetValue2
@ stub -version=0xA00+ RtlFlsSetValue
@ stub -version=0xA00+ RtlFlushHeaps
@ stub -arch=win64 -version=0xA00+ RtlFlushNonVolatileMemory
@ stub -arch=win64 -version=0xA00+ RtlFlushNonVolatileMemoryRanges
@ stub -arch=win64 -version=0xA00+ RtlFreeNonVolatileToken
@ stub -version=0xA00+ RtlFreeUTF8String
@ stub -arch=win64 -version=0xA00+ RtlFreeUserFiberShadowStack
@ stub -version=0xA00+ RtlGetAcesBufferSize
@ stub -version=0xA00+ RtlGetActiveConsoleId
@ stub -version=0xA00+ RtlGetAppContainerNamedObjectPath
@ stub -version=0xA00+ RtlGetAppContainerParent
@ stub -version=0xA00+ RtlGetAppContainerSidType
@ stub -version=0xA00+ RtlGetConsoleSessionForegroundProcessId
@ stub -version=0xA00+ RtlGetCurrentServiceSessionId
@ stub -version=0xA00+ RtlGetCurrentThreadPrimaryGroup
@ stub -arch=win64 -version=0xA00+ RtlGetCurrentUmsThread
@ stub -version=0xA00+ RtlGetDeviceFamilyInfoEnum
@ stub -version=0xA00+ RtlGetEnabledExtendedFeatures
@ stub -version=0xA00+ RtlGetExePath
@ stub -version=0xA00+ RtlGetExtendedContextLength
@ stub -version=0xA00+ RtlGetExtendedContextLength2
@ stub -version=0xA00+ RtlGetExtendedFeaturesMask
@ stub -version=0xA00+ RtlGetFeatureToggleConfiguration
@ stub -version=0xA00+ RtlGetFeatureTogglesChangeToken
@ stub -version=0xA00+ RtlGetFullPathName_UEx
@ stub -version=0xA00+ RtlGetImageFileMachines
@ stub -version=0xA00+ RtlGetInterruptTimePrecise
@ stub -version=0xA00+ RtlGetLocaleFileMappingAddress
@ stub -version=0xA00+ RtlGetMultiTimePrecise
@ stub -version=0xA00+ RtlGetNextEntryHashTable
@ stub -arch=win64 -version=0xA00+ RtlGetNextUmsListItem
@ stub -arch=win64 -version=0xA00+ RtlGetNonVolatileToken
@ stub -version=0xA00+ RtlGetNtSystemRoot
@ stub -version=0xA00+ RtlGetPersistedStateLocation
@ stub -version=0xA00+ RtlGetProcessPreferredUILanguages
@ stub -version=0xA00+ RtlGetReturnAddressHijackTarget
@ stub -version=0xA00+ RtlGetSearchPath
@ stub -version=0xA00+ RtlGetSessionProperties
@ stub -version=0xA00+ RtlGetSuiteMask
@ stub -version=0xA00+ RtlGetSystemBootStatus
@ stub -version=0xA00+ RtlGetSystemBootStatusEx
@ stub -version=0xA00+ RtlGetSystemGlobalData
@ stub -version=0xA00+ RtlGetSystemTimeAndBias
@ stub -version=0xA00+ RtlGetSystemTimePrecise
@ stub -version=0xA00+ RtlGetThreadWorkOnBehalfTicket
@ stub -version=0xA00+ RtlGetTokenNamedObjectPath
@ stub -arch=win64 -version=0xA00+ RtlGetUmsCompletionListEvent
@ stub -arch=win64 -version=0xA00+ RtlGrowFunctionTable
@ stub -version=0xA00+ RtlGuardCheckLongJumpTarget
@ stub -version=0xA00+ RtlIncrementCorrelationVector
@ stub -version=0xA00+ RtlInitEnumerationHashTable
@ stub -version=0xA00+ RtlInitStringEx
@ stub -version=0xA00+ RtlInitStrongEnumerationHashTable
@ stub -version=0xA00+ RtlInitUTF8String
@ stub -version=0xA00+ RtlInitUTF8StringEx
@ stub -version=0xA00+ RtlInitWeakEnumerationHashTable
@ stub -arch=win64 -version=0xA00+ RtlInitializeBitMapEx
@ stub -version=0xA00+ RtlInitializeCorrelationVector
@ stub -version=0xA00+ RtlInitializeExtendedContext
@ stub -version=0xA00+ RtlInitializeExtendedContext2
@ stub -version=0xA00+ RtlInitializeSidEx
@ stub -version=0xA00+ RtlInsertEntryHashTable
@ stub -version=0xA00+ RtlInterlockedClearBitRun
@ stub -arch=win64 -version=0xA00+ RtlInterlockedClearBitRunEx
@ stub -version=0xA00+ RtlInterlockedSetBitRun
@ stub -version=0xA00+ RtlIsApiSetImplemented
@ stub -version=0xA00+ RtlIsCapabilitySid
@ stub -version=0xA00+ RtlIsCloudFilesPlaceholder
@ stub -version=0xA00+ RtlIsCurrentProcess
@ stub -version=0xA00+ RtlIsCurrentThread
@ stub -arch=win64 -version=0xA00+ RtlIsEcCode
@ stub -version=0xA00+ RtlIsElevatedRid
@ stub -version=0xA00+ RtlIsEnclaveFeaturePresent
@ stub -version=0xA00+ RtlIsFeatureEnabledForEnterprise
@ stub -version=0xA00+ RtlIsMultiSessionSku
@ stub -version=0xA00+ RtlIsMultiUsersInSessionSku
@ stub -version=0xA00+ RtlIsNameInUnUpcasedExpression
@ stub -version=0xA00+ RtlIsNonEmptyDirectoryReparsePointAllowed
@ stub -version=0xA00+ RtlIsPackageSid
@ stub -version=0xA00+ RtlIsParentOfChildAppContainer
@ stub -version=0xA00+ RtlIsPartialPlaceholder
@ stub -version=0xA00+ RtlIsPartialPlaceholderFileHandle
@ stub -version=0xA00+ RtlIsPartialPlaceholderFileInfo
@ stub -version=0xA00+ RtlIsStateSeparationEnabled
@ stub -version=0xA00+ RtlIsUntrustedObject
@ stub -version=0xA00+ RtlIsValidProcessTrustLabelSid
@ stub -version=0xA00+ RtlIsZeroMemory
@ stub -version=0xA00+ RtlKnownExceptionFilter
@ stub -arch=win64 -version=0xA00+ RtlLengthCurrentClearRunBackwardEx
@ stub -arch=win64 -version=0xA00+ RtlLengthCurrentClearRunForwardEx
@ stub -version=0xA00+ RtlLengthSidAsUnicodeString
@ stub -version=0xA00+ RtlLoadString
@ stub -version=0xA00+ RtlLocateExtendedFeature
@ stub -version=0xA00+ RtlLocateExtendedFeature2
@ stub -version=0xA00+ RtlLocateLegacyContext
@ stub -version=0xA00+ RtlLogUnexpectedCodepath
@ stub -version=0xA00+ RtlLookupEntryHashTable
@ stub -version=0xA00+ RtlLookupFirstMatchingElementGenericTableAvl
@ stub -version=0xA00+ RtlNormalizeSecurityDescriptor
@ stub -version=0xA00+ RtlNotifyFeatureToggleUsage
@ stub -version=0xA00+ RtlNotifyFeatureUsage
@ stub -arch=win64 -version=0xA00+ RtlNumberOfClearBitsEx
@ stub -version=0xA00+ RtlNumberOfClearBitsInRange
@ stub -arch=win64 -version=0xA00+ RtlNumberOfSetBitsEx
@ stub -version=0xA00+ RtlNumberOfSetBitsInRange
@ stub -arch=win64 -version=0xA00+ RtlOpenCrossProcessEmulatorWorkConnection
@ stub -version=0xA00+ RtlOsDeploymentState
@ stub -version=0xA00+ RtlOverwriteFeatureConfigurationBuffer
@ stub -version=0xA00+ RtlPublishWnfStateData
@ stub -version=0xA00+ RtlQueryAllFeatureConfigurations
@ stub -version=0xA00+ RtlQueryAllInternalFeatureConfigurations
@ stub -version=0xA00+ RtlQueryAllInternalRuntimeFeatureConfigurations
@ stub -version=0xA00+ RtlQueryFeatureConfiguration
@ stub -version=0xA00+ RtlQueryFeatureConfigurationChangeStamp
@ stub -version=0xA00+ RtlQueryFeatureUsageNotificationSubscriptions
@ stub -version=0xA00+ RtlQueryImageMitigationPolicy
@ stub -version=0xA00+ RtlQueryInternalFeatureConfiguration
@ stub -version=0xA00+ RtlQueryPackageClaims
@ stub -version=0xA00+ RtlQueryPackageIdentity
@ stub -version=0xA00+ RtlQueryPackageIdentityEx
@ stub -version=0xA00+ RtlQueryPointerMapping
@ stub -version=0xA00+ RtlQueryProcessPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlQueryPropertyStore
@ stub -version=0xA00+ RtlQueryProtectedPolicy
@ stub -version=0xA00+ RtlQueryRegistryValueWithFallback
@ stub -version=0xA00+ RtlQueryRegistryValuesEx
@ stub -version=0xA00+ RtlQueryResourcePolicy
@ stub -version=0xA00+ RtlQueryThreadPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlQueryThreadProfiling
@ stub -version=0xA00+ RtlQueryTokenHostIdAsUlong64
@ stub -arch=win64 -version=0xA00+ RtlQueryUmsThreadInformation
@ stub -version=0xA00+ RtlQueryUnbiasedInterruptTime
@ stub -version=0xA00+ RtlQueryValidationRunlevel
@ stub -version=0xA00+ RtlQueryWnfMetaNotification
@ stub -version=0xA00+ RtlQueryWnfStateData
@ stub -version=0xA00+ RtlQueryWnfStateDataWithExplicitScope
@ stub -version=0xA00+ RtlRaiseCustomSystemEventTrigger
@ stub -arch=win64 -version=0xA00+ RtlRaiseExceptionForReturnAddressHijack
@ stub -arch=win64 -version=0xA00+ RtlRaiseNoncontinuableException
@ stub -version=0xA00+ RtlRbInsertNodeEx
@ stub -version=0xA00+ RtlRbRemoveNode
@ stub -version=0xA00+ RtlRcuAllocate
@ stub -version=0xA00+ RtlRcuFree
@ stub -version=0xA00+ RtlRcuReadLock
@ stub -version=0xA00+ RtlRcuReadUnlock
@ stub -version=0xA00+ RtlRcuSynchronize
@ stub -version=0xA00+ RtlReadThreadProfilingData
@ stub -version=0xA00+ RtlRegisterFeatureConfigurationChangeNotification
@ stub -version=0xA00+ RtlRegisterForWnfMetaNotification
@ stub -version=0xA00+ RtlReleasePath
@ stub -version=0xA00+ RtlRemoveEntryHashTable
@ stub -version=0xA00+ RtlRemovePointerMapping
@ stub -version=0xA00+ RtlRemovePropertyStore
@ stub -version=0xA00+ RtlReplaceSidInSd
@ stub -version=0xA00+ RtlReplaceSystemDirectoryInPath
@ stub -version=0xA00+ RtlReportExceptionEx
@ stub -version=0xA00+ RtlReportSilentProcessExit
@ stub -version=0xA00+ RtlReportSqmEscalation
@ stub -version=0xA00+ RtlResetNtUserPfn
@ stub -version=0xA00+ RtlRestoreBootStatusDefaults
@ stub -version=0xA00+ RtlRestoreSystemBootStatusDefaults
@ stub -version=0xA00+ RtlRestoreThreadPreferredUILanguages
@ stub -arch=win64 -version=0xA00+ RtlSetAllBitsEx
@ stub -version=0xA00+ RtlSetBit
@ stub -arch=win64 -version=0xA00+ RtlSetBitEx
@ stub -arch=win64 -version=0xA00+ RtlSetBitsEx
@ stub -version=0xA00+ RtlSetExtendedFeaturesMask
@ stub -version=0xA00+ RtlSetFeatureConfigurations
@ stub -version=0xA00+ RtlSetImageMitigationPolicy
@ stub -version=0xA00+ RtlSetPortableOperatingSystem
@ stub -version=0xA00+ RtlSetProcessPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlSetProcessPreferredUILanguages
@ stub -version=0xA00+ RtlSetProtectedPolicy
@ stub -version=0xA00+ RtlSetProxiedProcessId
@ stub -version=0xA00+ RtlSetSearchPathMode
@ stub -version=0xA00+ RtlSetSystemBootStatus
@ stub -version=0xA00+ RtlSetSystemBootStatusEx
@ stub -version=0xA00+ RtlSetThreadPlaceholderCompatibilityMode
@ stub -version=0xA00+ RtlSetThreadPreferredUILanguages2
@ stub -version=0xA00+ RtlSetThreadSubProcessTag
@ stub -version=0xA00+ RtlSetThreadWorkOnBehalfTicket
@ stub -arch=win64 -version=0xA00+ RtlSetUmsThreadInformation
@ stub -arch=win32 -version=0xA00+ RtlSetUserCallbackExceptionFilter
@ stub -version=0xA00+ RtlSidDominatesForTrust
@ stub -version=0xA00+ RtlStringFromGUIDEx
@ stub -version=0xA00+ RtlStronglyEnumerateEntryHashTable
@ stub -version=0xA00+ RtlSubscribeForFeatureUsageNotification
@ stub -version=0xA00+ RtlSubscribeWnfStateChangeNotification
@ stub -version=0xA00+ RtlSwitchedVVI
@ stub -version=0xA00+ RtlTestAndPublishWnfStateData
@ stub -arch=win64 -version=0xA00+ RtlTestBitEx
@ stub -version=0xA00+ RtlTestProtectedAccess
@ stub -version=0xA00+ RtlTlsAlloc
@ stub -version=0xA00+ RtlTlsFree
@ stub -version=0xA00+ RtlTlsSetValue
@ stub -version=0xA00+ RtlTryConvertSRWLockSharedToExclusiveOrRelease
@ stub -version=0xA00+ RtlUTF8StringToUnicodeString
@ stub -version=0xA00+ RtlUdiv128
@ stub -arch=win64 -version=0xA00+ RtlUmsThreadYield
@ stub -version=0xA00+ RtlUnicodeStringToUTF8String
@ stub -version=0xA00+ RtlUnregisterFeatureConfigurationChangeNotification
@ stub -version=0xA00+ RtlUnsubscribeFromFeatureUsageNotifications
@ stub -version=0xA00+ RtlUnsubscribeWnfNotificationWaitForCompletion
@ stub -version=0xA00+ RtlUnsubscribeWnfNotificationWithCompletionCallback
@ stub -version=0xA00+ RtlUnsubscribeWnfStateChangeNotification
@ stub -version=0xA00+ RtlUserFiberStart
@ stub -version=0xA00+ RtlValidProcessProtection
@ stub -version=0xA00+ RtlValidateCorrelationVector
@ stub -version=0xA00+ RtlValidateUserCallTarget
@ stub -arch=win64 -version=0xA00+ RtlVirtualUnwind2
@ stub -version=0xA00+ RtlWaitForWnfMetaNotification
@ stub -version=0xA00+ RtlWakeAddressAllNoFence
@ stub -version=0xA00+ RtlWakeAddressSingleNoFence
@ stub -version=0xA00+ RtlWeaklyEnumerateEntryHashTable
@ stub -version=0xA00+ RtlWnfDllUnloadCallback
@ stub -arch=win64 -version=0xA00+ RtlWow64ChangeProcessState
@ stub -arch=win64 -version=0xA00+ RtlWow64ChangeThreadState
@ stub -arch=win64 -version=0xA00+ RtlWow64GetCpuAreaEnabledFeatures
@ stub -arch=win64 -version=0xA00+ RtlWow64GetCpuAreaInfo
@ stub -version=0xA00+ RtlWow64GetCurrentMachine
@ stub -version=0xA00+ RtlWow64GetEquivalentMachineCHPE
@ stub -version=0xA00+ RtlWow64GetProcessMachines
@ stub -version=0xA00+ RtlWow64GetSharedInfoProcess
@ stub -arch=win64 -version=0xA00+ RtlWow64GetThreadSelectorEntry
@ stub -version=0xA00+ RtlWow64IsWowGuestMachineSupported
@ stub -arch=win64 -version=0xA00+ RtlWow64PopAllCrossProcessWorkFromWorkList
@ stub -arch=win64 -version=0xA00+ RtlWow64PopCrossProcessWorkFromFreeList
@ stub -arch=win64 -version=0xA00+ RtlWow64PushCrossProcessWorkOntoFreeList
@ stub -arch=win64 -version=0xA00+ RtlWow64PushCrossProcessWorkOntoWorkList
@ stub -arch=win64 -version=0xA00+ RtlWow64RequestCrossProcessHeavyFlush
@ stub -arch=win64 -version=0xA00+ RtlWow64SuspendProcess
@ stub -arch=win64 -version=0xA00+ RtlWriteNonVolatileMemory
@ stub -version=0xA00+ RtlXRestore
@ stub -version=0xA00+ RtlXSave
@ stub -version=0xA00+ RtlpConvertAbsoluteToRelativeSecurityAttribute
@ stub -version=0xA00+ RtlpConvertRelativeToAbsoluteSecurityAttribute
@ stub -version=0xA00+ RtlpGetDeviceFamilyInfoEnum
@ stub -version=0xA00+ RtlpMergeSecurityAttributeInformation
@ stub -version=0xA00+ RtlpQueryProcessDebugInformationRemote
@ stub -version=0xA00+ RtlpTimeFieldsToTime
@ stub -version=0xA00+ RtlpTimeToTimeFields
@ stub -arch=win64 -version=0xA00+ RtlpWow64CtxFromAmd64
@ stub -arch=win64 -version=0xA00+ RtlpWow64GetContextOnAmd64
@ stub -arch=win64 -version=0xA00+ RtlpWow64SetContextOnAmd64
@ stub -version=0xA00+ SbExecuteProcedure
@ stub -version=0xA00+ SbSelectProcedure
@ stub -version=0xA00+ TpAllocAlpcCompletionEx
@ stub -version=0xA00+ TpAllocJobNotification
@ stub -version=0xA00+ TpAlpcRegisterCompletionList
@ stub -version=0xA00+ TpAlpcUnregisterCompletionList
@ stub -version=0xA00+ TpCallbackDetectedUnrecoverableError
@ stub -version=0xA00+ TpCallbackIndependent
@ stub -version=0xA00+ TpCallbackSendAlpcMessageOnCompletion
@ stub -version=0xA00+ TpCallbackSendPendingAlpcMessage
@ stub -version=0xA00+ TpDisablePoolCallbackChecks
@ stub -version=0xA00+ TpReleaseJobNotification
@ stub -version=0xA00+ TpSetDefaultPoolMaxThreads
@ stub -version=0xA00+ TpSetDefaultPoolStackInformation
@ stub -version=0xA00+ TpSetPoolMaxThreadsSoftLimit
@ stub -version=0xA00+ TpSetPoolThreadBasePriority
@ stub -version=0xA00+ TpSetPoolThreadCpuSets
@ stub -version=0xA00+ TpSetPoolWorkerThreadIdleTimeout
@ stub -version=0xA00+ TpSetTimerEx
@ stub -version=0xA00+ TpSetWaitEx
@ stub -version=0xA00+ TpTimerOutstandingCallbackCount
@ stub -version=0xA00+ TpTrimPools
@ stub -version=0xA00+ TpWaitForJobNotification
@ stub -version=0xA00+ TpWorkOnBehalfClearTicket
@ stub -version=0xA00+ TpWorkOnBehalfSetTicket
@ stub -version=0xA00+ WerReportExceptionWorker
@ stub -version=0xA00+ WinSqmAddToAverageDWORD
@ stub -version=0xA00+ WinSqmAddToStreamEx
@ stub -version=0xA00+ WinSqmCheckEscalationAddToStreamEx
@ stub -version=0xA00+ WinSqmCheckEscalationSetDWORD
@ stub -version=0xA00+ WinSqmCheckEscalationSetDWORD64
@ stub -version=0xA00+ WinSqmCheckEscalationSetString
@ stub -version=0xA00+ WinSqmCommonDatapointDelete
@ stub -version=0xA00+ WinSqmCommonDatapointSetDWORD
@ stub -version=0xA00+ WinSqmCommonDatapointSetDWORD64
@ stub -version=0xA00+ WinSqmCommonDatapointSetStreamEx
@ stub -version=0xA00+ WinSqmCommonDatapointSetString
@ stub -version=0xA00+ WinSqmGetEscalationRuleStatus
@ stub -version=0xA00+ WinSqmGetInstrumentationProperty
@ stub -version=0xA00+ WinSqmIncrementDWORD
@ stub -version=0xA00+ WinSqmIsOptedInEx
@ stub -version=0xA00+ WinSqmIsSessionDisabled
@ stub -version=0xA00+ WinSqmSetDWORD
@ stub -version=0xA00+ WinSqmSetDWORD64
@ stub -version=0xA00+ WinSqmSetEscalationInfo
@ stub -version=0xA00+ WinSqmSetIfMaxDWORD
@ stub -version=0xA00+ WinSqmSetIfMinDWORD
@ stub -version=0xA00+ WinSqmStartSessionForPartner
@ stub -version=0xA00+ WinSqmStartSqmOptinListener
@ stub -version=0xA00+ ZwAcquireCrossVmMutant
@ stub -version=0xA00+ ZwAcquireProcessActivityReference
@ stub -version=0xA00+ ZwAddAtomEx
@ stub -version=0xA00+ ZwAdjustTokenClaimsAndDeviceGroups
@ stub -version=0xA00+ ZwAlertMultipleThreadByThreadId
@ stub -version=0xA00+ ZwAlertThreadByThreadId
@ stub -version=0xA00+ ZwAlertThreadByThreadIdEx
@ stub -version=0xA00+ ZwAllocateReserveObject
@ stub -version=0xA00+ ZwAllocateUserPhysicalPagesEx
@ stub -version=0xA00+ ZwAllocateVirtualMemoryEx
@ stub -version=0xA00+ ZwAlpcConnectPortEx
@ stub -version=0xA00+ ZwAlpcImpersonateClientContainerOfPort
@ stub -version=0xA00+ ZwAssociateWaitCompletionPacket
@ stub -version=0xA00+ ZwCallEnclave
@ stub -version=0xA00+ ZwCancelTimer2
@ stub -version=0xA00+ ZwCancelWaitCompletionPacket
@ stub -version=0xA00+ ZwChangeProcessState
@ stub -version=0xA00+ ZwChangeThreadState
@ stub -version=0xA00+ ZwCommitRegistryTransaction
@ stub -version=0xA00+ ZwCompareSigningLevels
@ stub -version=0xA00+ ZwContinueEx
@ stub -version=0xA00+ ZwConvertBetweenAuxiliaryCounterAndPerformanceCounter
@ stub -version=0xA00+ ZwCopyFileChunk
@ stub -version=0xA00+ ZwCreateCpuPartition
@ stub -version=0xA00+ ZwCreateCrossVmEvent
@ stub -version=0xA00+ ZwCreateCrossVmMutant
@ stub -version=0xA00+ ZwCreateDirectoryObjectEx
@ stub -version=0xA00+ ZwCreateEnclave
@ stub -version=0xA00+ ZwCreateIRTimer
@ stub -version=0xA00+ ZwCreateIoRing
@ stub -version=0xA00+ ZwCreateLowBoxToken
@ stub -version=0xA00+ ZwCreatePartition
@ stub -version=0xA00+ ZwCreatePrivateNamespace
@ stub -version=0xA00+ ZwCreateProcessStateChange
@ stub -version=0xA00+ ZwCreateProfileEx
@ stub -version=0xA00+ ZwCreateRegistryTransaction
@ stub -version=0xA00+ ZwCreateSectionEx
@ stub -version=0xA00+ ZwCreateThreadStateChange
@ stub -version=0xA00+ ZwCreateTimer2
@ stub -version=0xA00+ ZwCreateTokenEx
@ stub -version=0xA00+ ZwCreateWaitCompletionPacket
@ stub -version=0xA00+ ZwCreateWnfStateName
@ stub -version=0xA00+ ZwDeleteWnfStateData
@ stub -version=0xA00+ ZwDeleteWnfStateName
@ stub -version=0xA00+ ZwDirectGraphicsCall
@ stub -version=0xA00+ ZwDisableLastKnownGood
@ stub -version=0xA00+ ZwDrawText
@ stub -version=0xA00+ ZwEnableLastKnownGood
@ stub -version=0xA00+ ZwFilterBootOption
@ stub -version=0xA00+ ZwFilterTokenEx
@ stub -version=0xA00+ ZwFlushBuffersFileEx
@ stub -version=0xA00+ ZwGetCachedSigningLevel
@ stub -version=0xA00+ ZwGetCompleteWnfStateSubscription
@ stub -version=0xA00+ ZwInitializeEnclave
@ stub -version=0xA00+ ZwLoadEnclaveData
@ stub -version=0xA00+ ZwLoadKey3
@ stub -version=0xA00+ ZwManageHotPatch
@ stub -version=0xA00+ ZwManagePartition
@ stub -version=0xA00+ ZwMapViewOfSectionEx
@ stub -version=0xA00+ ZwNotifyChangeDirectoryFileEx
@ stub -version=0xA00+ ZwNotifyChangeSession
@ stub -version=0xA00+ ZwOpenCpuPartition
@ stub -version=0xA00+ ZwOpenKeyEx
@ stub -version=0xA00+ ZwOpenKeyTransactedEx
@ stub -version=0xA00+ ZwOpenPartition
@ stub -version=0xA00+ ZwOpenRegistryTransaction
@ stub -version=0xA00+ ZwPssCaptureVaSpaceBulk
@ stub -version=0xA00+ ZwQueryAuxiliaryCounterFrequency
@ stub -version=0xA00+ ZwQueryDirectoryFileEx
@ stub -version=0xA00+ ZwQueryInformationByName
@ stub -version=0xA00+ ZwQueryInformationCpuPartition
@ stub -version=0xA00+ ZwQueryIoRingCapabilities
@ stub -version=0xA00+ ZwQuerySecurityAttributesToken
@ stub -version=0xA00+ ZwQuerySecurityPolicy
@ stub -version=0xA00+ ZwQuerySystemInformationEx
@ stub -version=0xA00+ ZwQueryWnfStateData
@ stub -version=0xA00+ ZwQueryWnfStateNameInformation
@ stub -version=0xA00+ ZwQueueApcThreadEx
@ stub -version=0xA00+ ZwQueueApcThreadEx2
@ stub -version=0xA00+ ZwReadVirtualMemoryEx
@ stub -version=0xA00+ ZwRevertContainerImpersonation
@ stub -version=0xA00+ ZwRollbackRegistryTransaction
@ stub -version=0xA00+ ZwSerializeBoot
@ stub -version=0xA00+ ZwSetCachedSigningLevel
@ stub -version=0xA00+ ZwSetCachedSigningLevel2
@ stub -version=0xA00+ ZwSetEventEx
@ stub -version=0xA00+ ZwSetIRTimer
@ stub -version=0xA00+ ZwSetInformationCpuPartition
@ stub -version=0xA00+ ZwSetInformationIoRing
@ stub -version=0xA00+ ZwSetInformationSymbolicLink
@ stub -version=0xA00+ ZwSetInformationVirtualMemory
@ stub -version=0xA00+ ZwSetIoCompletionEx
@ stub -version=0xA00+ ZwSetTimer2
@ stub -version=0xA00+ ZwSetTimerEx
@ stub -version=0xA00+ ZwSetWnfProcessNotificationEvent
@ stub -version=0xA00+ ZwSubmitIoRing
@ stub -version=0xA00+ ZwSubscribeWnfStateChange
@ stub -version=0xA00+ ZwTerminateEnclave
@ stub -version=0xA00+ ZwUmsThreadYield
@ stub -version=0xA00+ ZwUnmapViewOfSectionEx
@ stub -version=0xA00+ ZwUnsubscribeWnfStateChange
@ stub -version=0xA00+ ZwUpdateWnfStateData
@ stub -version=0xA00+ ZwWaitForAlertByThreadId
@ stub -arch=win32 -version=0xA00+ ZwWow64AllocateVirtualMemory64
@ stub -arch=win32 -version=0xA00+ ZwWow64CallFunction64
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrAllocateCaptureBuffer
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrAllocateMessagePointer
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrCaptureMessageBuffer
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrCaptureMessageString
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrClientCallServer
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrClientConnectToServer
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrFreeCaptureBuffer
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrGetProcessId
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrIdentifyAlertableThread
@ stub -arch=win32 -version=0xA00+ ZwWow64CsrVerifyRegion
@ stub -arch=win32 -version=0xA00+ ZwWow64DebuggerCall
@ stub -arch=win32 -version=0xA00+ ZwWow64GetCurrentProcessorNumber
@ stub -arch=win32 -version=0xA00+ ZwWow64GetCurrentProcessorNumberEx
@ stub -arch=win32 -version=0xA00+ ZwWow64GetNativeSystemInformation
@ stub -arch=win32 -version=0xA00+ ZwWow64IsProcessorFeaturePresent
@ stub -arch=win32 -version=0xA00+ ZwWow64QueryInformationProcess64
@ stub -arch=win32 -version=0xA00+ ZwWow64ReadVirtualMemory64
@ stub -arch=win32 -version=0xA00+ ZwWow64WriteVirtualMemory64
@ stub -arch=win32 -version=0xA00+ _except_handler4_common
@ stub -arch=win32 -version=0xA00+ _ftol2
@ stub -arch=win32 -version=0xA00+ _ftol2_sse
@ stub -version=0xA00+ _i64toa_s
@ stub -version=0xA00+ _i64tow_s
@ stub -version=0xA00+ _itoa_s
@ stub -version=0xA00+ _itow_s
@ stub -arch=win32 -version=0xA00+ _libm_sse2_cos_precise
@ stub -arch=win32 -version=0xA00+ _libm_sse2_sin_precise
@ stub -arch=win32 -version=0xA00+ _libm_sse2_sqrt_precise
@ stub -arch=win32 -version=0xA00+ _local_unwind4
@ stub -version=0xA00+ _ltoa_s
@ stub -version=0xA00+ _ltow_s
@ stub -version=0xA00+ _makepath_s
@ stub -version=0xA00+ _snprintf_s
@ stub -version=0xA00+ _snscanf_s
@ stub -version=0xA00+ _snwprintf_s
@ stub -version=0xA00+ _snwscanf_s
@ stub -version=0xA00+ _splitpath_s
@ stub -version=0xA00+ _strlwr_s
@ stub -version=0xA00+ _strnset_s
@ stub -version=0xA00+ _strset_s
@ stub -version=0xA00+ _strupr_s
@ stub -version=0xA00+ _ui64toa_s
@ stub -version=0xA00+ _ui64tow_s
@ stub -version=0xA00+ _ultoa_s
@ stub -version=0xA00+ _ultow_s
@ stub -version=0xA00+ _vscprintf
@ stub -version=0xA00+ _vsnprintf_s
@ stub -version=0xA00+ _vsnwprintf_s
@ stub -version=0xA00+ _wcslwr_s
@ stub -version=0xA00+ _wcsnset_s
@ stub -version=0xA00+ _wcsset_s
@ stub -version=0xA00+ _wcstoi64
@ stub -version=0xA00+ _wcsupr_s
@ stub -version=0xA00+ _wmakepath_s
@ stub -version=0xA00+ _wsplitpath_s
@ stub -version=0xA00+ bsearch_s
@ stub -version=0xA00+ iswalnum
@ stub -version=0xA00+ iswascii
@ stub -version=0xA00+ iswgraph
@ stub -version=0xA00+ iswprint
@ stub -version=0xA00+ memcpy_s
@ stub -version=0xA00+ memmove_s
@ stub -version=0xA00+ qsort_s
@ stub -version=0xA00+ sprintf_s
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
@ stub -version=0xA00+ wcstok_s
# Data exports (need an `extern`, not a code stub):
#   KiUserInvertedFunctionTable
#   LdrParentInterlockedPopEntrySList
#   LdrParentRtlInitializeNtUserPfn
#   LdrParentRtlResetNtUserPfn
#   LdrParentRtlRetrieveNtUserPfn
#   LdrpChildNtdll
#   RtlpFreezeTimeBias
#   RtlpScpCfgNtdllExports
# END win10-surface stubs

# BEGIN vista forwards
# Generated by tools/win10_surface.py forwards; do not edit by hand. Exports Windows'
# ntdll.dll has and this build would not, forwarded to the implementation in ntdll_vista.
# Each switches off where this spec's own row for the name takes over.
@ stdcall -version=0x400-0x5ff LdrRegisterDllNotification(long ptr ptr ptr) ntdll_vista.LdrRegisterDllNotification
@ stdcall -version=0x400-0x5ff LdrUnregisterDllNotification(ptr) ntdll_vista.LdrUnregisterDllNotification
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlAcquireSRWLockExclusive(ptr) ntdll_vista.RtlAcquireSRWLockExclusive
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlAcquireSRWLockShared(ptr) ntdll_vista.RtlAcquireSRWLockShared
@ stdcall -version=0x400-0x5ff RtlConnectToSm(ptr ptr long ptr) ntdll_vista.RtlConnectToSm
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlInitializeConditionVariable(ptr) ntdll_vista.RtlInitializeConditionVariable
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlInitializeSRWLock(ptr) ntdll_vista.RtlInitializeSRWLock
@ stdcall -version=0x400-0x5ff RtlLcidToLocaleName(long ptr long long) ntdll_vista.RtlLcidToLocaleName
@ stdcall -version=0x400-0x5ff RtlLocaleNameToLcid(wstr ptr long) ntdll_vista.RtlLocaleNameToLcid
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlReleaseSRWLockExclusive(ptr) ntdll_vista.RtlReleaseSRWLockExclusive
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlReleaseSRWLockShared(ptr) ntdll_vista.RtlReleaseSRWLockShared
@ stdcall -version=0x400-0x5ff RtlRunOnceBeginInitialize(ptr long ptr) ntdll_vista.RtlRunOnceBeginInitialize
@ stdcall -version=0x400-0x5ff RtlRunOnceComplete(ptr long ptr) ntdll_vista.RtlRunOnceComplete
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlRunOnceExecuteOnce(ptr ptr ptr ptr) ntdll_vista.RtlRunOnceExecuteOnce
@ stdcall -version=0x400-0x5ff RtlRunOnceInitialize(ptr) ntdll_vista.RtlRunOnceInitialize
@ stdcall -version=0x400-0x5ff RtlSendMsgToSm(ptr ptr) ntdll_vista.RtlSendMsgToSm
@ stdcall -version=0x400-0x5ff RtlSleepConditionVariableCS(ptr ptr ptr) ntdll_vista.RtlSleepConditionVariableCS
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlSleepConditionVariableSRW(ptr ptr ptr long) ntdll_vista.RtlSleepConditionVariableSRW
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlWaitOnAddress(ptr ptr long ptr) ntdll_vista.RtlWaitOnAddress
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlWakeAddressAll(ptr) ntdll_vista.RtlWakeAddressAll
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlWakeAddressSingle(ptr) ntdll_vista.RtlWakeAddressSingle
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlWakeAllConditionVariable(ptr) ntdll_vista.RtlWakeAllConditionVariable
@ stdcall -arch=i386,x86_64 -version=0x400-0x5ff RtlWakeConditionVariable(ptr) ntdll_vista.RtlWakeConditionVariable
@ stdcall -version=0x400-0x5ff TpAllocCleanupGroup(ptr) ntdll_vista.TpAllocCleanupGroup
@ stdcall -version=0x400-0x5ff TpAllocIoCompletion(ptr ptr ptr ptr ptr) ntdll_vista.TpAllocIoCompletion
@ stdcall -version=0x400-0x5ff TpAllocPool(ptr ptr) ntdll_vista.TpAllocPool
@ stdcall -version=0x400-0x5ff TpAllocTimer(ptr ptr ptr ptr) ntdll_vista.TpAllocTimer
@ stdcall -version=0x400-0x5ff TpAllocWait(ptr ptr ptr ptr) ntdll_vista.TpAllocWait
@ stdcall -version=0x400-0x5ff TpAllocWork(ptr ptr ptr ptr) ntdll_vista.TpAllocWork
@ stdcall -version=0x400-0x5ff TpCallbackLeaveCriticalSectionOnCompletion(ptr ptr) ntdll_vista.TpCallbackLeaveCriticalSectionOnCompletion
@ stdcall -version=0x400-0x5ff TpCallbackMayRunLong(ptr) ntdll_vista.TpCallbackMayRunLong
@ stdcall -version=0x400-0x5ff TpCallbackReleaseMutexOnCompletion(ptr ptr) ntdll_vista.TpCallbackReleaseMutexOnCompletion
@ stdcall -version=0x400-0x5ff TpCallbackReleaseSemaphoreOnCompletion(ptr ptr long) ntdll_vista.TpCallbackReleaseSemaphoreOnCompletion
@ stdcall -version=0x400-0x5ff TpCallbackSetEventOnCompletion(ptr ptr) ntdll_vista.TpCallbackSetEventOnCompletion
@ stdcall -version=0x400-0x5ff TpCallbackUnloadDllOnCompletion(ptr ptr) ntdll_vista.TpCallbackUnloadDllOnCompletion
@ stdcall -version=0x400-0x5ff TpCancelAsyncIoOperation(ptr) ntdll_vista.TpCancelAsyncIoOperation
@ stdcall -version=0x400-0x5ff TpDisassociateCallback(ptr) ntdll_vista.TpDisassociateCallback
@ stdcall -version=0x400-0x5ff TpIsTimerSet(ptr) ntdll_vista.TpIsTimerSet
@ stdcall -version=0x400-0x5ff TpPostWork(ptr) ntdll_vista.TpPostWork
@ stdcall -version=0x400-0x600 TpQueryPoolStackInformation(ptr ptr) ntdll_vista.TpQueryPoolStackInformation
@ stdcall -version=0x400-0x5ff TpReleaseCleanupGroup(ptr) ntdll_vista.TpReleaseCleanupGroup
@ stdcall -version=0x400-0x5ff TpReleaseCleanupGroupMembers(ptr long ptr) ntdll_vista.TpReleaseCleanupGroupMembers
@ stdcall -version=0x400-0x5ff TpReleaseIoCompletion(ptr) ntdll_vista.TpReleaseIoCompletion
@ stdcall -version=0x400-0x5ff TpReleasePool(ptr) ntdll_vista.TpReleasePool
@ stdcall -version=0x400-0x5ff TpReleaseTimer(ptr) ntdll_vista.TpReleaseTimer
@ stdcall -version=0x400-0x5ff TpReleaseWait(ptr) ntdll_vista.TpReleaseWait
@ stdcall -version=0x400-0x5ff TpReleaseWork(ptr) ntdll_vista.TpReleaseWork
@ stdcall -version=0x400-0x5ff TpSetPoolMaxThreads(ptr long) ntdll_vista.TpSetPoolMaxThreads
@ stdcall -version=0x400-0x5ff TpSetPoolMinThreads(ptr long) ntdll_vista.TpSetPoolMinThreads
@ stdcall -version=0x400-0x600 TpSetPoolStackInformation(ptr ptr) ntdll_vista.TpSetPoolStackInformation
@ stdcall -version=0x400-0x5ff TpSetTimer(ptr ptr long long) ntdll_vista.TpSetTimer
@ stdcall -version=0x400-0x5ff TpSetWait(ptr long ptr) ntdll_vista.TpSetWait
@ stdcall -version=0x400-0x5ff TpSimpleTryPost(ptr ptr ptr) ntdll_vista.TpSimpleTryPost
@ stdcall -version=0x400-0x5ff TpStartAsyncIoOperation(ptr) ntdll_vista.TpStartAsyncIoOperation
@ stdcall -version=0x400-0x5ff TpWaitForIoCompletion(ptr long) ntdll_vista.TpWaitForIoCompletion
@ stdcall -version=0x400-0x5ff TpWaitForTimer(ptr long) ntdll_vista.TpWaitForTimer
@ stdcall -version=0x400-0x5ff TpWaitForWait(ptr long) ntdll_vista.TpWaitForWait
@ stdcall -version=0x400-0x5ff TpWaitForWork(ptr long) ntdll_vista.TpWaitForWork
# END vista forwards
