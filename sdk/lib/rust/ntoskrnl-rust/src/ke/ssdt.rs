// AUTO-GENERATED from ntoskrnl/include/sysfuncs.h by tools/gen_ssdt.py.
// Do not edit by hand — re-run the generator if sysfuncs.h changes.
//
// Materializes MainSSDT (function pointer table), MainSSPT (per-syscall
// argument byte counts), and MainNumberOfSysCalls in Rust. Replaces the
// macro-expansion of <internal/napi.h> inside ke_rust_shims.c so the shim
// can drop its <ntoskrnl.h> include.
//
// MainSSDT entries are taken as `<name> as usize` so the ABI of the
// underlying Rust impl (NTAPI = stdcall on i386, cdecl on amd64/arm64)
// is opaque here — KiFastCallEntry asm does the real call. The forward
// declarations below are sized with `usize` arguments so on i386 the
// stdcall name decoration (_Name@N) matches the real impl.
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]
#![allow(unused_imports)]

/// SSDT entry — a syscall implementation. The arg count + ABI is
/// captured by the per-syscall extern decl below; here we erase to
/// `*const ()` because Rust forbids `fn() as fn()` casts between
/// different signatures and forbids `fn() as integer` in const
/// init, but allows `fn() as *const ()`. On every supported arch a
/// fn pointer is the same size as `ULONG_PTR`, so consumers re-cast
/// `addr_of_mut!(MainSSDT)` to `*mut ULONG_PTR` to match the
/// historical C `ULONG_PTR MainSSDT[]` signature without changing
/// the bytes.
pub type SSDTEntry = *const ();

extern "system" {
    pub fn NtAcceptConnectPort(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheck(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheckAndAuditAlarm(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheckByType(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheckByTypeAndAuditAlarm(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheckByTypeResultList(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheckByTypeResultListAndAuditAlarm(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAccessCheckByTypeResultListAndAuditAlarmByHandle(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAddAtom(_: usize, _: usize, _: usize);
    pub fn NtAddBootEntry(_: usize, _: usize);
    pub fn NtAddDriverEntry(_: usize, _: usize);
    pub fn NtAdjustGroupsToken(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAdjustPrivilegesToken(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtAlertResumeThread(_: usize, _: usize);
    pub fn NtAlertThread(_: usize);
    pub fn NtAllocateLocallyUniqueId(_: usize);
    pub fn NtAllocateUserPhysicalPages(_: usize, _: usize, _: usize);
    pub fn NtAllocateUuids(_: usize, _: usize, _: usize, _: usize);
    pub fn NtAllocateVirtualMemory(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtApphelpCacheControl(_: usize, _: usize);
    pub fn NtAreMappedFilesTheSame(_: usize, _: usize);
    pub fn NtAssignProcessToJobObject(_: usize, _: usize);
    pub fn NtCallbackReturn(_: usize, _: usize, _: usize);
    pub fn NtCancelDeviceWakeupRequest(_: usize);
    pub fn NtCancelIoFile(_: usize, _: usize);
    pub fn NtCancelTimer(_: usize, _: usize);
    pub fn NtClearEvent(_: usize);
    pub fn NtClose(_: usize);
    pub fn NtCloseObjectAuditAlarm(_: usize, _: usize, _: usize);
    pub fn NtCompactKeys(_: usize, _: usize);
    pub fn NtCompareTokens(_: usize, _: usize, _: usize);
    pub fn NtCompleteConnectPort(_: usize);
    pub fn NtCompressKey(_: usize);
    pub fn NtConnectPort(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtContinue(_: usize, _: usize);
    pub fn NtCreateDebugObject(_: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateDirectoryObject(_: usize, _: usize, _: usize);
    pub fn NtCreateEvent(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateEventPair(_: usize, _: usize, _: usize);
    pub fn NtCreateFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateIoCompletion(_: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateJobObject(_: usize, _: usize, _: usize);
    pub fn NtCreateJobSet(_: usize, _: usize, _: usize);
    pub fn NtCreateKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateMailslotFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateMutant(_: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateNamedPipeFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreatePagingFile(_: usize, _: usize, _: usize, _: usize);
    pub fn NtCreatePort(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateProcess(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateProcessEx(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateProfile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateSection(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateSemaphore(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateSymbolicLinkObject(_: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateThread(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateTimer(_: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateToken(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCreateWaitablePort(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtDebugActiveProcess(_: usize, _: usize);
    pub fn NtDebugContinue(_: usize, _: usize, _: usize);
    pub fn NtDelayExecution(_: usize, _: usize);
    pub fn NtDeleteAtom(_: usize);
    pub fn NtDeleteBootEntry(_: usize);
    pub fn NtDeleteDriverEntry(_: usize);
    pub fn NtDeleteFile(_: usize);
    pub fn NtDeleteKey(_: usize);
    pub fn NtDeleteObjectAuditAlarm(_: usize, _: usize, _: usize);
    pub fn NtDeleteValueKey(_: usize, _: usize);
    pub fn NtDeviceIoControlFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtDisplayString(_: usize);
    pub fn NtDuplicateObject(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtDuplicateToken(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtEnumerateBootEntries(_: usize, _: usize);
    pub fn NtEnumerateDriverEntries(_: usize, _: usize);
    pub fn NtEnumerateKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtEnumerateSystemEnvironmentValuesEx(_: usize, _: usize, _: usize);
    pub fn NtEnumerateValueKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtExtendSection(_: usize, _: usize);
    pub fn NtFilterToken(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtFindAtom(_: usize, _: usize, _: usize);
    pub fn NtFlushBuffersFile(_: usize, _: usize);
    pub fn NtFlushInstructionCache(_: usize, _: usize, _: usize);
    pub fn NtFlushKey(_: usize);
    pub fn NtFlushVirtualMemory(_: usize, _: usize, _: usize, _: usize);
    pub fn NtFlushWriteBuffer();
    pub fn NtFreeUserPhysicalPages(_: usize, _: usize, _: usize);
    pub fn NtFreeVirtualMemory(_: usize, _: usize, _: usize, _: usize);
    pub fn NtFsControlFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtGetContextThread(_: usize, _: usize);
    pub fn NtGetDevicePowerState(_: usize, _: usize);
    pub fn NtGetPlugPlayEvent(_: usize, _: usize, _: usize, _: usize);
    pub fn NtGetWriteWatch(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtImpersonateAnonymousToken(_: usize);
    pub fn NtImpersonateClientOfPort(_: usize, _: usize);
    pub fn NtImpersonateThread(_: usize, _: usize, _: usize);
    pub fn NtInitializeRegistry(_: usize);
    pub fn NtInitiatePowerAction(_: usize, _: usize, _: usize, _: usize);
    pub fn NtIsProcessInJob(_: usize, _: usize);
    pub fn NtIsSystemResumeAutomatic();
    pub fn NtListenPort(_: usize, _: usize);
    pub fn NtLoadDriver(_: usize);
    pub fn NtLoadKey(_: usize, _: usize);
    pub fn NtLoadKey2(_: usize, _: usize, _: usize);
    pub fn NtLoadKeyEx(_: usize, _: usize, _: usize, _: usize);
    pub fn NtLockFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtLockProductActivationKeys(_: usize, _: usize);
    pub fn NtLockRegistryKey(_: usize);
    pub fn NtLockVirtualMemory(_: usize, _: usize, _: usize, _: usize);
    pub fn NtMakePermanentObject(_: usize);
    pub fn NtMakeTemporaryObject(_: usize);
    pub fn NtMapUserPhysicalPages(_: usize, _: usize, _: usize);
    pub fn NtMapUserPhysicalPagesScatter(_: usize, _: usize, _: usize);
    pub fn NtMapViewOfSection(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtModifyBootEntry(_: usize);
    pub fn NtModifyDriverEntry(_: usize);
    pub fn NtNotifyChangeDirectoryFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtNotifyChangeKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtNotifyChangeMultipleKeys(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenDirectoryObject(_: usize, _: usize, _: usize);
    pub fn NtOpenEvent(_: usize, _: usize, _: usize);
    pub fn NtOpenEventPair(_: usize, _: usize, _: usize);
    pub fn NtOpenFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenIoCompletion(_: usize, _: usize, _: usize);
    pub fn NtOpenJobObject(_: usize, _: usize, _: usize);
    pub fn NtOpenKey(_: usize, _: usize, _: usize);
    pub fn NtOpenMutant(_: usize, _: usize, _: usize);
    pub fn NtOpenObjectAuditAlarm(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenProcess(_: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenProcessToken(_: usize, _: usize, _: usize);
    pub fn NtOpenProcessTokenEx(_: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenSection(_: usize, _: usize, _: usize);
    pub fn NtOpenSemaphore(_: usize, _: usize, _: usize);
    pub fn NtOpenSymbolicLinkObject(_: usize, _: usize, _: usize);
    pub fn NtOpenThread(_: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenThreadToken(_: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenThreadTokenEx(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenTimer(_: usize, _: usize, _: usize);
    pub fn NtPlugPlayControl(_: usize, _: usize, _: usize);
    pub fn NtPowerInformation(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtPrivilegeCheck(_: usize, _: usize, _: usize);
    pub fn NtPrivilegeObjectAuditAlarm(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtPrivilegedServiceAuditAlarm(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtProtectVirtualMemory(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtPulseEvent(_: usize, _: usize);
    pub fn NtQueryAttributesFile(_: usize, _: usize);
    pub fn NtQueryBootEntryOrder(_: usize, _: usize);
    pub fn NtQueryBootOptions(_: usize, _: usize);
    pub fn NtQueryDebugFilterState(_: usize, _: usize);
    pub fn NtQueryDefaultLocale(_: usize, _: usize);
    pub fn NtQueryDefaultUILanguage(_: usize);
    pub fn NtQueryDirectoryFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryDirectoryObject(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryDriverEntryOrder(_: usize, _: usize);
    pub fn NtQueryEaFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryEvent(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryFullAttributesFile(_: usize, _: usize);
    pub fn NtQueryInformationAtom(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInformationFile(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInformationJobObject(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInformationPort(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInformationProcess(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInformationThread(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInformationToken(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryInstallUILanguage(_: usize);
    pub fn NtQueryIntervalProfile(_: usize, _: usize);
    pub fn NtQueryIoCompletion(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryKey(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryMultipleValueKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryMutant(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryObject(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryOpenSubKeys(_: usize, _: usize);
    pub fn NtQueryOpenSubKeysEx(_: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryPerformanceCounter(_: usize, _: usize);
    pub fn NtQueryQuotaInformationFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySection(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySecurityObject(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySemaphore(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySymbolicLinkObject(_: usize, _: usize, _: usize);
    pub fn NtQuerySystemEnvironmentValue(_: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySystemEnvironmentValueEx(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySystemInformation(_: usize, _: usize, _: usize, _: usize);
    pub fn NtQuerySystemTime(_: usize);
    pub fn NtQueryTimer(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryTimerResolution(_: usize, _: usize, _: usize);
    pub fn NtQueryValueKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryVirtualMemory(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryVolumeInformationFile(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtQueueApcThread(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtRaiseException(_: usize, _: usize, _: usize);
    pub fn NtRaiseHardError(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtReadFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtReadFileScatter(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtReadRequestData(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtReadVirtualMemory(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtRegisterThreadTerminatePort(_: usize);
    pub fn NtReleaseMutant(_: usize, _: usize);
    pub fn NtReleaseSemaphore(_: usize, _: usize, _: usize);
    pub fn NtRemoveIoCompletion(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtRemoveProcessDebug(_: usize, _: usize);
    pub fn NtRenameKey(_: usize, _: usize);
    pub fn NtReplaceKey(_: usize, _: usize, _: usize);
    pub fn NtReplyPort(_: usize, _: usize);
    pub fn NtReplyWaitReceivePort(_: usize, _: usize, _: usize, _: usize);
    pub fn NtReplyWaitReceivePortEx(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtReplyWaitReplyPort(_: usize, _: usize);
    pub fn NtRequestDeviceWakeup(_: usize);
    pub fn NtRequestPort(_: usize, _: usize);
    pub fn NtRequestWaitReplyPort(_: usize, _: usize, _: usize);
    pub fn NtRequestWakeupLatency(_: usize);
    pub fn NtResetEvent(_: usize, _: usize);
    pub fn NtResetWriteWatch(_: usize, _: usize, _: usize);
    pub fn NtRestoreKey(_: usize, _: usize, _: usize);
    pub fn NtResumeProcess(_: usize);
    pub fn NtResumeThread(_: usize, _: usize);
    pub fn NtSaveKey(_: usize, _: usize);
    pub fn NtSaveKeyEx(_: usize, _: usize, _: usize);
    pub fn NtSaveMergedKeys(_: usize, _: usize, _: usize);
    pub fn NtSecureConnectPort(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetBootEntryOrder(_: usize, _: usize);
    pub fn NtSetBootOptions(_: usize, _: usize);
    pub fn NtSetContextThread(_: usize, _: usize);
    pub fn NtSetDebugFilterState(_: usize, _: usize, _: usize);
    pub fn NtSetDefaultHardErrorPort(_: usize);
    pub fn NtSetDefaultLocale(_: usize, _: usize);
    pub fn NtSetDefaultUILanguage(_: usize);
    pub fn NtSetDriverEntryOrder(_: usize, _: usize);
    pub fn NtSetEaFile(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetEvent(_: usize, _: usize);
    pub fn NtSetEventBoostPriority(_: usize);
    pub fn NtSetHighEventPair(_: usize);
    pub fn NtSetHighWaitLowEventPair(_: usize);
    pub fn NtSetInformationDebugObject(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationFile(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationJobObject(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationKey(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationObject(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationProcess(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationThread(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetInformationToken(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetIntervalProfile(_: usize, _: usize);
    pub fn NtSetIoCompletion(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetLdtEntries(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetLowEventPair(_: usize);
    pub fn NtSetLowWaitHighEventPair(_: usize);
    pub fn NtSetQuotaInformationFile(_: usize, _: usize, _: usize, _: usize);
    pub fn NtSetSecurityObject(_: usize, _: usize, _: usize);
    pub fn NtSetSystemEnvironmentValue(_: usize, _: usize);
    pub fn NtSetSystemEnvironmentValueEx(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetSystemInformation(_: usize, _: usize, _: usize);
    pub fn NtSetSystemPowerState(_: usize, _: usize, _: usize);
    pub fn NtSetSystemTime(_: usize, _: usize);
    pub fn NtSetThreadExecutionState(_: usize, _: usize);
    pub fn NtSetTimer(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetTimerResolution(_: usize, _: usize, _: usize);
    pub fn NtSetUuidSeed(_: usize);
    pub fn NtSetValueKey(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtSetVolumeInformationFile(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtShutdownSystem(_: usize);
    pub fn NtSignalAndWaitForSingleObject(_: usize, _: usize, _: usize, _: usize);
    pub fn NtStartProfile(_: usize);
    pub fn NtStopProfile(_: usize);
    pub fn NtSuspendProcess(_: usize);
    pub fn NtSuspendThread(_: usize, _: usize);
    pub fn NtSystemDebugControl(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtTerminateJobObject(_: usize, _: usize);
    pub fn NtTerminateProcess(_: usize, _: usize);
    pub fn NtTerminateThread(_: usize, _: usize);
    pub fn NtTestAlert();
    pub fn NtTraceEvent(_: usize, _: usize, _: usize, _: usize);
    pub fn NtTranslateFilePath(_: usize, _: usize, _: usize, _: usize);
    pub fn NtUnloadDriver(_: usize);
    pub fn NtUnloadKey(_: usize);
    pub fn NtUnloadKey2(_: usize, _: usize);
    pub fn NtUnloadKeyEx(_: usize, _: usize);
    pub fn NtUnlockFile(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtUnlockVirtualMemory(_: usize, _: usize, _: usize, _: usize);
    pub fn NtUnmapViewOfSection(_: usize, _: usize);
    pub fn NtVdmControl(_: usize, _: usize);
    pub fn NtWaitForDebugEvent(_: usize, _: usize, _: usize, _: usize);
    pub fn NtWaitForMultipleObjects(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtWaitForSingleObject(_: usize, _: usize, _: usize);
    pub fn NtWaitHighEventPair(_: usize);
    pub fn NtWaitLowEventPair(_: usize);
    pub fn NtWriteFile(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtWriteFileGather(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtWriteRequestData(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtWriteVirtualMemory(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtYieldExecution();
    pub fn NtCreateKeyedEvent(_: usize, _: usize, _: usize, _: usize);
    pub fn NtOpenKeyedEvent(_: usize, _: usize, _: usize);
    pub fn NtReleaseKeyedEvent(_: usize, _: usize, _: usize, _: usize);
    pub fn NtWaitForKeyedEvent(_: usize, _: usize, _: usize, _: usize);
    pub fn NtQueryPortInformationProcess();
    pub fn NtGetCurrentProcessorNumber();
    pub fn NtWaitForMultipleObjects32(_: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtCancelIoFileEx(_: usize, _: usize, _: usize);
    pub fn NtCancelSynchronousIoFile(_: usize, _: usize, _: usize);
    pub fn NtRemoveIoCompletionEx(_: usize, _: usize, _: usize, _: usize, _: usize, _: usize);
    pub fn NtFlushProcessWriteBuffers();
    pub fn NtCompareObjects(_: usize, _: usize);
}

// `static mut` (not `static`) so consumers can `addr_of_mut!` to match
// the original C `ULONG_PTR MainSSDT[]` declaration.
#[no_mangle]
pub static mut MainSSDT: [SSDTEntry; 301] = [
    NtAcceptConnectPort as SSDTEntry,
    NtAccessCheck as SSDTEntry,
    NtAccessCheckAndAuditAlarm as SSDTEntry,
    NtAccessCheckByType as SSDTEntry,
    NtAccessCheckByTypeAndAuditAlarm as SSDTEntry,
    NtAccessCheckByTypeResultList as SSDTEntry,
    NtAccessCheckByTypeResultListAndAuditAlarm as SSDTEntry,
    NtAccessCheckByTypeResultListAndAuditAlarmByHandle as SSDTEntry,
    NtAddAtom as SSDTEntry,
    NtAddBootEntry as SSDTEntry,
    NtAddDriverEntry as SSDTEntry,
    NtAdjustGroupsToken as SSDTEntry,
    NtAdjustPrivilegesToken as SSDTEntry,
    NtAlertResumeThread as SSDTEntry,
    NtAlertThread as SSDTEntry,
    NtAllocateLocallyUniqueId as SSDTEntry,
    NtAllocateUserPhysicalPages as SSDTEntry,
    NtAllocateUuids as SSDTEntry,
    NtAllocateVirtualMemory as SSDTEntry,
    NtApphelpCacheControl as SSDTEntry,
    NtAreMappedFilesTheSame as SSDTEntry,
    NtAssignProcessToJobObject as SSDTEntry,
    NtCallbackReturn as SSDTEntry,
    NtCancelDeviceWakeupRequest as SSDTEntry,
    NtCancelIoFile as SSDTEntry,
    NtCancelTimer as SSDTEntry,
    NtClearEvent as SSDTEntry,
    NtClose as SSDTEntry,
    NtCloseObjectAuditAlarm as SSDTEntry,
    NtCompactKeys as SSDTEntry,
    NtCompareTokens as SSDTEntry,
    NtCompleteConnectPort as SSDTEntry,
    NtCompressKey as SSDTEntry,
    NtConnectPort as SSDTEntry,
    NtContinue as SSDTEntry,
    NtCreateDebugObject as SSDTEntry,
    NtCreateDirectoryObject as SSDTEntry,
    NtCreateEvent as SSDTEntry,
    NtCreateEventPair as SSDTEntry,
    NtCreateFile as SSDTEntry,
    NtCreateIoCompletion as SSDTEntry,
    NtCreateJobObject as SSDTEntry,
    NtCreateJobSet as SSDTEntry,
    NtCreateKey as SSDTEntry,
    NtCreateMailslotFile as SSDTEntry,
    NtCreateMutant as SSDTEntry,
    NtCreateNamedPipeFile as SSDTEntry,
    NtCreatePagingFile as SSDTEntry,
    NtCreatePort as SSDTEntry,
    NtCreateProcess as SSDTEntry,
    NtCreateProcessEx as SSDTEntry,
    NtCreateProfile as SSDTEntry,
    NtCreateSection as SSDTEntry,
    NtCreateSemaphore as SSDTEntry,
    NtCreateSymbolicLinkObject as SSDTEntry,
    NtCreateThread as SSDTEntry,
    NtCreateTimer as SSDTEntry,
    NtCreateToken as SSDTEntry,
    NtCreateWaitablePort as SSDTEntry,
    NtDebugActiveProcess as SSDTEntry,
    NtDebugContinue as SSDTEntry,
    NtDelayExecution as SSDTEntry,
    NtDeleteAtom as SSDTEntry,
    NtDeleteBootEntry as SSDTEntry,
    NtDeleteDriverEntry as SSDTEntry,
    NtDeleteFile as SSDTEntry,
    NtDeleteKey as SSDTEntry,
    NtDeleteObjectAuditAlarm as SSDTEntry,
    NtDeleteValueKey as SSDTEntry,
    NtDeviceIoControlFile as SSDTEntry,
    NtDisplayString as SSDTEntry,
    NtDuplicateObject as SSDTEntry,
    NtDuplicateToken as SSDTEntry,
    NtEnumerateBootEntries as SSDTEntry,
    NtEnumerateDriverEntries as SSDTEntry,
    NtEnumerateKey as SSDTEntry,
    NtEnumerateSystemEnvironmentValuesEx as SSDTEntry,
    NtEnumerateValueKey as SSDTEntry,
    NtExtendSection as SSDTEntry,
    NtFilterToken as SSDTEntry,
    NtFindAtom as SSDTEntry,
    NtFlushBuffersFile as SSDTEntry,
    NtFlushInstructionCache as SSDTEntry,
    NtFlushKey as SSDTEntry,
    NtFlushVirtualMemory as SSDTEntry,
    NtFlushWriteBuffer as SSDTEntry,
    NtFreeUserPhysicalPages as SSDTEntry,
    NtFreeVirtualMemory as SSDTEntry,
    NtFsControlFile as SSDTEntry,
    NtGetContextThread as SSDTEntry,
    NtGetDevicePowerState as SSDTEntry,
    NtGetPlugPlayEvent as SSDTEntry,
    NtGetWriteWatch as SSDTEntry,
    NtImpersonateAnonymousToken as SSDTEntry,
    NtImpersonateClientOfPort as SSDTEntry,
    NtImpersonateThread as SSDTEntry,
    NtInitializeRegistry as SSDTEntry,
    NtInitiatePowerAction as SSDTEntry,
    NtIsProcessInJob as SSDTEntry,
    NtIsSystemResumeAutomatic as SSDTEntry,
    NtListenPort as SSDTEntry,
    NtLoadDriver as SSDTEntry,
    NtLoadKey as SSDTEntry,
    NtLoadKey2 as SSDTEntry,
    NtLoadKeyEx as SSDTEntry,
    NtLockFile as SSDTEntry,
    NtLockProductActivationKeys as SSDTEntry,
    NtLockRegistryKey as SSDTEntry,
    NtLockVirtualMemory as SSDTEntry,
    NtMakePermanentObject as SSDTEntry,
    NtMakeTemporaryObject as SSDTEntry,
    NtMapUserPhysicalPages as SSDTEntry,
    NtMapUserPhysicalPagesScatter as SSDTEntry,
    NtMapViewOfSection as SSDTEntry,
    NtModifyBootEntry as SSDTEntry,
    NtModifyDriverEntry as SSDTEntry,
    NtNotifyChangeDirectoryFile as SSDTEntry,
    NtNotifyChangeKey as SSDTEntry,
    NtNotifyChangeMultipleKeys as SSDTEntry,
    NtOpenDirectoryObject as SSDTEntry,
    NtOpenEvent as SSDTEntry,
    NtOpenEventPair as SSDTEntry,
    NtOpenFile as SSDTEntry,
    NtOpenIoCompletion as SSDTEntry,
    NtOpenJobObject as SSDTEntry,
    NtOpenKey as SSDTEntry,
    NtOpenMutant as SSDTEntry,
    NtOpenObjectAuditAlarm as SSDTEntry,
    NtOpenProcess as SSDTEntry,
    NtOpenProcessToken as SSDTEntry,
    NtOpenProcessTokenEx as SSDTEntry,
    NtOpenSection as SSDTEntry,
    NtOpenSemaphore as SSDTEntry,
    NtOpenSymbolicLinkObject as SSDTEntry,
    NtOpenThread as SSDTEntry,
    NtOpenThreadToken as SSDTEntry,
    NtOpenThreadTokenEx as SSDTEntry,
    NtOpenTimer as SSDTEntry,
    NtPlugPlayControl as SSDTEntry,
    NtPowerInformation as SSDTEntry,
    NtPrivilegeCheck as SSDTEntry,
    NtPrivilegeObjectAuditAlarm as SSDTEntry,
    NtPrivilegedServiceAuditAlarm as SSDTEntry,
    NtProtectVirtualMemory as SSDTEntry,
    NtPulseEvent as SSDTEntry,
    NtQueryAttributesFile as SSDTEntry,
    NtQueryBootEntryOrder as SSDTEntry,
    NtQueryBootOptions as SSDTEntry,
    NtQueryDebugFilterState as SSDTEntry,
    NtQueryDefaultLocale as SSDTEntry,
    NtQueryDefaultUILanguage as SSDTEntry,
    NtQueryDirectoryFile as SSDTEntry,
    NtQueryDirectoryObject as SSDTEntry,
    NtQueryDriverEntryOrder as SSDTEntry,
    NtQueryEaFile as SSDTEntry,
    NtQueryEvent as SSDTEntry,
    NtQueryFullAttributesFile as SSDTEntry,
    NtQueryInformationAtom as SSDTEntry,
    NtQueryInformationFile as SSDTEntry,
    NtQueryInformationJobObject as SSDTEntry,
    NtQueryInformationPort as SSDTEntry,
    NtQueryInformationProcess as SSDTEntry,
    NtQueryInformationThread as SSDTEntry,
    NtQueryInformationToken as SSDTEntry,
    NtQueryInstallUILanguage as SSDTEntry,
    NtQueryIntervalProfile as SSDTEntry,
    NtQueryIoCompletion as SSDTEntry,
    NtQueryKey as SSDTEntry,
    NtQueryMultipleValueKey as SSDTEntry,
    NtQueryMutant as SSDTEntry,
    NtQueryObject as SSDTEntry,
    NtQueryOpenSubKeys as SSDTEntry,
    NtQueryOpenSubKeysEx as SSDTEntry,
    NtQueryPerformanceCounter as SSDTEntry,
    NtQueryQuotaInformationFile as SSDTEntry,
    NtQuerySection as SSDTEntry,
    NtQuerySecurityObject as SSDTEntry,
    NtQuerySemaphore as SSDTEntry,
    NtQuerySymbolicLinkObject as SSDTEntry,
    NtQuerySystemEnvironmentValue as SSDTEntry,
    NtQuerySystemEnvironmentValueEx as SSDTEntry,
    NtQuerySystemInformation as SSDTEntry,
    NtQuerySystemTime as SSDTEntry,
    NtQueryTimer as SSDTEntry,
    NtQueryTimerResolution as SSDTEntry,
    NtQueryValueKey as SSDTEntry,
    NtQueryVirtualMemory as SSDTEntry,
    NtQueryVolumeInformationFile as SSDTEntry,
    NtQueueApcThread as SSDTEntry,
    NtRaiseException as SSDTEntry,
    NtRaiseHardError as SSDTEntry,
    NtReadFile as SSDTEntry,
    NtReadFileScatter as SSDTEntry,
    NtReadRequestData as SSDTEntry,
    NtReadVirtualMemory as SSDTEntry,
    NtRegisterThreadTerminatePort as SSDTEntry,
    NtReleaseMutant as SSDTEntry,
    NtReleaseSemaphore as SSDTEntry,
    NtRemoveIoCompletion as SSDTEntry,
    NtRemoveProcessDebug as SSDTEntry,
    NtRenameKey as SSDTEntry,
    NtReplaceKey as SSDTEntry,
    NtReplyPort as SSDTEntry,
    NtReplyWaitReceivePort as SSDTEntry,
    NtReplyWaitReceivePortEx as SSDTEntry,
    NtReplyWaitReplyPort as SSDTEntry,
    NtRequestDeviceWakeup as SSDTEntry,
    NtRequestPort as SSDTEntry,
    NtRequestWaitReplyPort as SSDTEntry,
    NtRequestWakeupLatency as SSDTEntry,
    NtResetEvent as SSDTEntry,
    NtResetWriteWatch as SSDTEntry,
    NtRestoreKey as SSDTEntry,
    NtResumeProcess as SSDTEntry,
    NtResumeThread as SSDTEntry,
    NtSaveKey as SSDTEntry,
    NtSaveKeyEx as SSDTEntry,
    NtSaveMergedKeys as SSDTEntry,
    NtSecureConnectPort as SSDTEntry,
    NtSetBootEntryOrder as SSDTEntry,
    NtSetBootOptions as SSDTEntry,
    NtSetContextThread as SSDTEntry,
    NtSetDebugFilterState as SSDTEntry,
    NtSetDefaultHardErrorPort as SSDTEntry,
    NtSetDefaultLocale as SSDTEntry,
    NtSetDefaultUILanguage as SSDTEntry,
    NtSetDriverEntryOrder as SSDTEntry,
    NtSetEaFile as SSDTEntry,
    NtSetEvent as SSDTEntry,
    NtSetEventBoostPriority as SSDTEntry,
    NtSetHighEventPair as SSDTEntry,
    NtSetHighWaitLowEventPair as SSDTEntry,
    NtSetInformationDebugObject as SSDTEntry,
    NtSetInformationFile as SSDTEntry,
    NtSetInformationJobObject as SSDTEntry,
    NtSetInformationKey as SSDTEntry,
    NtSetInformationObject as SSDTEntry,
    NtSetInformationProcess as SSDTEntry,
    NtSetInformationThread as SSDTEntry,
    NtSetInformationToken as SSDTEntry,
    NtSetIntervalProfile as SSDTEntry,
    NtSetIoCompletion as SSDTEntry,
    NtSetLdtEntries as SSDTEntry,
    NtSetLowEventPair as SSDTEntry,
    NtSetLowWaitHighEventPair as SSDTEntry,
    NtSetQuotaInformationFile as SSDTEntry,
    NtSetSecurityObject as SSDTEntry,
    NtSetSystemEnvironmentValue as SSDTEntry,
    NtSetSystemEnvironmentValueEx as SSDTEntry,
    NtSetSystemInformation as SSDTEntry,
    NtSetSystemPowerState as SSDTEntry,
    NtSetSystemTime as SSDTEntry,
    NtSetThreadExecutionState as SSDTEntry,
    NtSetTimer as SSDTEntry,
    NtSetTimerResolution as SSDTEntry,
    NtSetUuidSeed as SSDTEntry,
    NtSetValueKey as SSDTEntry,
    NtSetVolumeInformationFile as SSDTEntry,
    NtShutdownSystem as SSDTEntry,
    NtSignalAndWaitForSingleObject as SSDTEntry,
    NtStartProfile as SSDTEntry,
    NtStopProfile as SSDTEntry,
    NtSuspendProcess as SSDTEntry,
    NtSuspendThread as SSDTEntry,
    NtSystemDebugControl as SSDTEntry,
    NtTerminateJobObject as SSDTEntry,
    NtTerminateProcess as SSDTEntry,
    NtTerminateThread as SSDTEntry,
    NtTestAlert as SSDTEntry,
    NtTraceEvent as SSDTEntry,
    NtTranslateFilePath as SSDTEntry,
    NtUnloadDriver as SSDTEntry,
    NtUnloadKey as SSDTEntry,
    NtUnloadKey2 as SSDTEntry,
    NtUnloadKeyEx as SSDTEntry,
    NtUnlockFile as SSDTEntry,
    NtUnlockVirtualMemory as SSDTEntry,
    NtUnmapViewOfSection as SSDTEntry,
    NtVdmControl as SSDTEntry,
    NtWaitForDebugEvent as SSDTEntry,
    NtWaitForMultipleObjects as SSDTEntry,
    NtWaitForSingleObject as SSDTEntry,
    NtWaitHighEventPair as SSDTEntry,
    NtWaitLowEventPair as SSDTEntry,
    NtWriteFile as SSDTEntry,
    NtWriteFileGather as SSDTEntry,
    NtWriteRequestData as SSDTEntry,
    NtWriteVirtualMemory as SSDTEntry,
    NtYieldExecution as SSDTEntry,
    NtCreateKeyedEvent as SSDTEntry,
    NtOpenKeyedEvent as SSDTEntry,
    NtReleaseKeyedEvent as SSDTEntry,
    NtWaitForKeyedEvent as SSDTEntry,
    NtQueryPortInformationProcess as SSDTEntry,
    NtGetCurrentProcessorNumber as SSDTEntry,
    NtWaitForMultipleObjects32 as SSDTEntry,
    NtCancelIoFileEx as SSDTEntry,
    NtCancelSynchronousIoFile as SSDTEntry,
    NtRemoveIoCompletionEx as SSDTEntry,
    NtFlushProcessWriteBuffers as SSDTEntry,
    NtCompareObjects as SSDTEntry,
];

// `argcount * sizeof(void*)` per entry — pointer size differs per arch
// so the multiplier is computed at const-eval time.
#[no_mangle]
pub static mut MainSSPT: [u8; 301] = [
    (6 * core::mem::size_of::<usize>()) as u8,
    (8 * core::mem::size_of::<usize>()) as u8,
    (11 * core::mem::size_of::<usize>()) as u8,
    (11 * core::mem::size_of::<usize>()) as u8,
    (16 * core::mem::size_of::<usize>()) as u8,
    (11 * core::mem::size_of::<usize>()) as u8,
    (16 * core::mem::size_of::<usize>()) as u8,
    (17 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (8 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (11 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (7 * core::mem::size_of::<usize>()) as u8,
    (8 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (14 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (8 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (7 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (8 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (13 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (10 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (7 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (10 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (7 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (10 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (10 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (10 * core::mem::size_of::<usize>()) as u8,
    (12 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (12 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (11 * core::mem::size_of::<usize>()) as u8,
    (7 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (7 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (1 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (9 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (4 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (5 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (3 * core::mem::size_of::<usize>()) as u8,
    (6 * core::mem::size_of::<usize>()) as u8,
    (0 * core::mem::size_of::<usize>()) as u8,
    (2 * core::mem::size_of::<usize>()) as u8,
];

#[no_mangle]
pub static mut MainNumberOfSysCalls: u32 = 301;
