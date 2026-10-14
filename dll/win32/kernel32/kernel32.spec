@ stdcall -arch=arm64 AcquireSRWLockExclusive(ptr) ntdll_vista.RtlAcquireSRWLockExclusive
@ stdcall -version=0x600+ AcquireSRWLockShared(ptr) ntdll.RtlAcquireSRWLockShared
@ stdcall ActivateActCtx(ptr ptr)
@ stdcall AddAtomA(str)
@ stdcall AddAtomW(wstr)
@ stdcall AddConsoleAliasA(str str str) ;check
@ stdcall AddConsoleAliasW(wstr wstr wstr) ;check
@ stdcall AddLocalAlternateComputerNameA(str ptr)
@ stdcall AddLocalAlternateComputerNameW(wstr ptr)
@ stdcall AddRefActCtx(ptr)
@ stdcall -stub -version=0x600+ AddSIDToBoundaryDescriptor(ptr ptr)
@ stdcall -stub -version=0x600+ AddSecureMemoryCacheCallback(ptr)
@ stdcall AddVectoredContinueHandler(long ptr) ntdll.RtlAddVectoredContinueHandler
@ stdcall AddVectoredExceptionHandler(long ptr) ntdll.RtlAddVectoredExceptionHandler
@ stdcall -stub -version=0x600+ AdjustCalendarDate(ptr long long)
@ stdcall AllocConsole()
@ stdcall AllocateUserPhysicalPages(long ptr ptr)
@ stdcall -stub -version=0x600+ AllocateUserPhysicalPagesNuma(ptr ptr ptr long)
@ stdcall -version=0x600+ ApplicationRecoveryFinished(long)
@ stdcall -version=0x600+ ApplicationRecoveryInProgress(ptr)
@ stdcall AreFileApisANSI()
@ stdcall AssignProcessToJobObject(ptr ptr)
@ stdcall AttachConsole(long)
@ stdcall BackupRead(ptr ptr long ptr long long ptr)
@ stdcall BackupSeek(ptr long long ptr ptr ptr)
@ stdcall BackupWrite(ptr ptr long ptr long long ptr)
@ stdcall BaseCheckAppcompatCache(wstr ptr wstr ptr)
@ stdcall BaseCheckRunApp(long ptr long long long long long long long long)
@ stdcall BaseCleanupAppcompatCacheSupport(ptr)
@ stdcall BaseDumpAppcompatCache()
@ stdcall BaseFlushAppcompatCache()
@ stub -version=0x600+ BaseGenerateAppCompatData
@ stdcall BaseInitAppcompatCacheSupport()
@ stdcall BaseIsAppcompatInfrastructureDisabled() IsShimInfrastructureDisabled
@ stdcall -version=0x501-0x502 BaseProcessInitPostImport()
@ stdcall -version=0x600+ BaseProcessInitPostImport() # HACK: This export is dynamicaly imported by ntdll
# BaseProcessStart / BaseThreadStart: XP x64's exports, here on every arch, as
# BaseInitializeContext's process and thread starts under names another process
# can resolve. wowx64 recognises an x86-64 guest's new-process and new-thread
# CONTEXTs by them and translates each into the native kernel32's (a native
# arm64 child's initial thread; CreateRemoteThread between an x86-64 guest and
# a native or 32-bit process, in either direction).
@ stdcall BaseProcessStart(ptr) BaseProcessStartup
@ stdcall BaseQueryModuleData(str str ptr ptr ptr) ;check
@ stub -version=0x600+ BaseThreadInitThunk
@ stdcall BaseThreadStart(ptr ptr) BaseThreadStartup
@ stdcall BaseUpdateAppcompatCache(long long long)
@ stdcall BasepCheckBadapp(long ptr long long long long long long long)
@ stdcall BasepCheckWinSaferRestrictions(long long long long long long)
@ stdcall BasepFreeAppCompatData(ptr ptr)
@ stdcall Beep(long long)
@ stdcall BeginUpdateResourceA(str long)
@ stdcall BeginUpdateResourceW(wstr long)
@ stdcall BindIoCompletionCallback(long ptr long)
@ stdcall BuildCommDCBA(str ptr)
@ stdcall BuildCommDCBAndTimeoutsA(str ptr ptr)
@ stdcall BuildCommDCBAndTimeoutsW(wstr ptr ptr)
@ stdcall BuildCommDCBW(wstr ptr)
@ stdcall CallNamedPipeA(str ptr long ptr long ptr long)
@ stdcall CallNamedPipeW(wstr ptr long ptr long ptr long)
@ stdcall -stub -version=0x600+ CallbackMayRunLong(ptr)
@ stdcall CancelDeviceWakeupRequest(long)
@ stdcall CancelIo(long)
@ stdcall -stub -version=0x600+ CancelIoEx(ptr ptr)
@ stdcall -stub -version=0x600+ CancelSynchronousIo(ptr)
@ stdcall -version=0x600+ CancelThreadpoolIo(ptr) ntdll.TpCancelAsyncIoOperation
@ stdcall CancelTimerQueueTimer(long long)
@ stdcall CancelWaitableTimer(long)
@ stdcall ChangeTimerQueueTimer(ptr ptr long long)
@ stdcall -stub -version=0x600+ CheckElevation(ptr ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ CheckElevationEnabled(ptr)
@ stub -version=0x600+ CheckForReadOnlyResource
@ stdcall CheckNameLegalDOS8Dot3A(str str long long long)
@ stdcall CheckNameLegalDOS8Dot3W(wstr str long long long)
@ stdcall CheckRemoteDebuggerPresent(long ptr)
@ stdcall ClearCommBreak(long)
@ stdcall ClearCommError(long ptr ptr)
@ stdcall CloseConsoleHandle(long)
@ stdcall CloseHandle(long)
@ stdcall -stub -version=0x600+ ClosePrivateNamespace(ptr long)
@ stdcall CloseProfileUserMapping()
@ stdcall -version=0x600+ CloseThreadpool(ptr) ntdll.TpReleasePool
@ stdcall -version=0x600+ CloseThreadpoolCleanupGroup(ptr) ntdll.TpReleaseCleanupGroup
@ stdcall -version=0x600+ CloseThreadpoolCleanupGroupMembers(ptr long ptr) ntdll.TpReleaseCleanupGroupMembers
@ stdcall -version=0x600+ CloseThreadpoolIo(ptr) ntdll.TpReleaseIoCompletion
@ stdcall -version=0x600+ CloseThreadpoolTimer(ptr) ntdll.TpReleaseTimer
@ stdcall -version=0x600+ CloseThreadpoolWait(ptr) ntdll.TpReleaseWait
@ stdcall -version=0x600+ CloseThreadpoolWork(ptr) ntdll.TpReleaseWork
@ stdcall CmdBatNotification(long)
@ stdcall CommConfigDialogA(str long ptr)
@ stdcall CommConfigDialogW(wstr long ptr)
@ stdcall -stub -version=0x600+ CompareCalendarDates(ptr ptr ptr)
@ stdcall CompareFileTime(ptr ptr)
@ stdcall CompareStringA(long long str long str long)
@ stdcall CompareStringEx(wstr long wstr long wstr long ptr ptr ptr)
@ stdcall -version=0x600+ CompareStringOrdinal(wstr long wstr long long)
@ stdcall CompareStringW(long long wstr long wstr long)
@ stdcall ConnectNamedPipe(long ptr)
# Upstream-sync 2026-09: genuinely new upstream export (client/console/init.c), not yet
# ported. Internal East-Asian-IME startup helper for winsrv.dll on Server2003/XP64/
# Vista x86_64 only; -arch=x86_64 means no i386 decoration concern from leaving this a
# bare stub. Depends on GetConsoleIMECommandLine (also unported) plus CreateProcessW /
# WaitForSingleObject / TerminateProcess, already in kernel32-rust -- porting is real
# work for a dead legacy code path nothing in this tree calls. Left as a stub rather
# than silently reverting/hiding it.
@ stub -version=0x502-0x600 -arch=x86_64 ConsoleIMERoutine
@ stdcall ConsoleMenuControl(long long long)
@ stdcall ContinueDebugEvent(long long long)
@ stdcall -stub -version=0x600+ ConvertCalDateTimeToSystemTime(ptr ptr)
@ stdcall ConvertDefaultLocale(long)
@ stdcall ConvertFiberToThread()
@ stdcall -stub -version=0x600+ ConvertNLSDayOfWeekToWin32DayOfWeek(long)
@ stdcall -stub -version=0x600+ ConvertSystemTimeToCalDateTime(ptr long ptr)
@ stdcall ConvertThreadToFiber(ptr)
@ stdcall ConvertThreadToFiberEx(ptr long)
@ stdcall CopyFileA(str str long)
@ stdcall CopyFileExA(str str ptr ptr ptr long)
@ stdcall CopyFileExW(wstr wstr ptr ptr ptr long)
@ stdcall -stub -version=0x600+ CopyFileTransactedA(str str ptr ptr ptr long ptr)
@ stdcall -stub -version=0x600+ CopyFileTransactedW(wstr wstr ptr ptr ptr long ptr)
@ stdcall CopyFileW(wstr wstr long)
@ stdcall CopyLZFile(long long) LZCopy
@ stdcall CreateActCtxA(ptr)
@ stdcall CreateActCtxW(ptr)
@ stdcall -stub -version=0x600+ CreateBoundaryDescriptorA(str long)
@ stdcall -stub -version=0x600+ CreateBoundaryDescriptorW(wstr long)
@ stdcall CreateConsoleScreenBuffer(long long ptr long ptr)
@ stdcall CreateDirectoryA(str ptr)
@ stdcall CreateDirectoryExA(str str ptr)
@ stdcall CreateDirectoryExW(wstr wstr ptr)
@ stdcall -stub -version=0x600+ CreateDirectoryTransactedA(str str ptr ptr)
@ stdcall -stub -version=0x600+ CreateDirectoryTransactedW(wstr wstr ptr ptr)
@ stdcall CreateDirectoryW(wstr ptr)
@ stdcall CreateEventA(ptr long long str)
@ stdcall CreateEventExA(ptr str long long)
@ stdcall CreateEventExW(ptr wstr long long)
@ stdcall CreateEventW(ptr long long wstr)
@ stdcall CreateFiber(long ptr ptr)
@ stdcall CreateFiberEx(long long long ptr ptr)
@ stdcall CreateFileA(str long long ptr long long long)
@ stdcall CreateFileMappingA(long ptr long long long str)
@ stdcall -stub -version=0x600+ CreateFileMappingNumaA(ptr ptr long long long str long)
@ stdcall -stub -version=0x600+ CreateFileMappingNumaW(ptr ptr long long long wstr long)
@ stdcall CreateFileMappingW(long ptr long long long wstr)
@ stdcall -stub -version=0x600+ CreateFileTransactedA(str long long ptr long long ptr ptr ptr ptr)
@ stdcall -stub -version=0x600+ CreateFileTransactedW(wstr long long ptr long long ptr ptr ptr ptr)
@ stdcall CreateFileW(wstr long long ptr long long long)
@ stdcall CreateHardLinkA(str str ptr)
@ stdcall -stub -version=0x600+ CreateHardLinkTransactedA(str str ptr ptr)
@ stdcall -stub -version=0x600+ CreateHardLinkTransactedW(wstr wstr ptr ptr)
@ stdcall CreateHardLinkW(wstr wstr ptr)
@ stdcall CreateIoCompletionPort(long long long long)
@ stdcall CreateJobObjectA(ptr str)
@ stdcall CreateJobObjectW(ptr wstr)
@ stdcall CreateJobSet(long ptr long)
@ stdcall CreateMailslotA(ptr long long ptr)
@ stdcall CreateMailslotW(ptr long long ptr)
@ stdcall CreateMemoryResourceNotification(long)
@ stdcall CreateMutexA(ptr long str)
@ stdcall CreateMutexExA(ptr str long long)
@ stdcall CreateMutexExW(ptr wstr long long)
@ stdcall CreateMutexW(ptr long wstr)
@ stdcall CreateNamedPipeA(str long long long long long long ptr)
@ stdcall CreateNamedPipeW(wstr long long long long long long ptr)
@ stdcall -version=0x501-0x502 CreateNlsSecurityDescriptor(ptr long long)
@ stdcall CreatePipe(ptr ptr ptr long)
@ stdcall -stub -version=0x600+ CreatePrivateNamespaceA(ptr ptr str)
@ stdcall -stub -version=0x600+ CreatePrivateNamespaceW(ptr ptr wstr)
@ stdcall CreateProcessA(str str ptr ptr long long ptr str ptr ptr)
@ stdcall CreateProcessInternalA(ptr str str ptr ptr long long ptr str ptr ptr long)
@ stdcall CreateProcessInternalW(ptr wstr wstr ptr ptr long long ptr wstr ptr ptr long)
@ stdcall CreateProcessW(wstr wstr ptr ptr long long ptr wstr ptr ptr)
@ stdcall CreateRemoteThread(long ptr long ptr long long ptr)
@ stdcall CreateSemaphoreA(ptr long long str)
@ stdcall CreateSemaphoreExA(ptr long long str long long)
@ stdcall CreateSemaphoreExW(ptr long long wstr long long)
@ stdcall CreateSemaphoreW(ptr long long wstr)
@ stdcall -i386 CreateSocketHandle()
@ stdcall -version=0x600+ CreateSymbolicLinkA(str str long)
@ stdcall -stub -version=0x600+ CreateSymbolicLinkTransactedA(str str long ptr)
@ stdcall -stub -version=0x600+ CreateSymbolicLinkTransactedW(wstr wstr long ptr)
@ stdcall -arch=arm64 CreateSymbolicLinkW(wstr wstr long) kernel32_vista.CreateSymbolicLinkW
@ stdcall CreateTapePartition(long long long long)
@ stdcall CreateThread(ptr long ptr long long ptr)
@ stdcall -version=0x600+ CreateThreadpool(ptr)
@ stdcall -version=0x600+ CreateThreadpoolCleanupGroup()
@ stdcall -version=0x600+ CreateThreadpoolIo(ptr ptr ptr ptr)
@ stdcall -version=0x600+ CreateThreadpoolTimer(ptr ptr ptr)
@ stdcall -version=0x600+ CreateThreadpoolWait(ptr ptr ptr)
@ stdcall -version=0x600+ CreateThreadpoolWork(ptr ptr ptr)
@ stdcall CreateTimerQueue()
@ stdcall CreateTimerQueueTimer(ptr long ptr ptr long long long)
@ stdcall CreateToolhelp32Snapshot(long long)
@ stdcall CreateWaitableTimerA(ptr long str)
@ stdcall CreateWaitableTimerExA(ptr str long long)
@ stdcall CreateWaitableTimerExW(ptr wstr long long)
@ stdcall CreateWaitableTimerW(ptr long wstr)
;@ stdcall -arch=x86_64 CtrlRoutine()
@ stdcall DeactivateActCtx(long ptr)
@ stdcall DebugActiveProcess(long)
@ stdcall DebugActiveProcessStop(long)
@ stdcall DebugBreak() ntdll.DbgBreakPoint
@ stdcall DebugBreakProcess(long)
@ stdcall DebugSetProcessKillOnExit(long)
@ stdcall DecodePointer(ptr) ntdll.RtlDecodePointer
@ stdcall DecodeSystemPointer(ptr) ntdll.RtlDecodeSystemPointer
@ stdcall DefineDosDeviceA(long str str)
@ stdcall DefineDosDeviceW(long wstr wstr)
@ stdcall DelayLoadFailureHook(str str)
@ stdcall DeleteAtom(long)
@ stub -version=0x600+ DeleteBoundaryDescriptor
@ stdcall DeleteCriticalSection(ptr) ntdll.RtlDeleteCriticalSection
@ stdcall DeleteFiber(ptr)
@ stdcall DeleteFileA(str)
@ stub -version=0x600+ DeleteFileTransactedA
@ stub -version=0x600+ DeleteFileTransactedW
@ stdcall DeleteFileW(wstr)
@ stdcall DeleteProcThreadAttributeList(ptr)
@ stdcall DeleteTimerQueue(long)
@ stdcall DeleteTimerQueueEx(long long)
@ stdcall DeleteTimerQueueTimer(long long long)
@ stdcall DeleteVolumeMountPointA(str) ;check
@ stdcall DeleteVolumeMountPointW(wstr) ;check
@ stdcall DeviceIoControl(long long ptr long ptr long ptr ptr)
@ stdcall DisableThreadLibraryCalls(ptr)
@ stub -version=0x600+ DisassociateCurrentThreadFromCallback
@ stdcall DisconnectNamedPipe(long)
@ stdcall DnsHostnameToComputerNameA(str ptr ptr)
@ stdcall DnsHostnameToComputerNameW(wstr ptr ptr)
@ stdcall DosDateTimeToFileTime(long long ptr)
@ stdcall DosPathToSessionPathA(long str str)
@ stdcall DosPathToSessionPathW(long wstr wstr)
@ stdcall DuplicateConsoleHandle(long long long long)
@ stdcall DuplicateHandle(long long long ptr long long long)
@ stdcall EncodePointer(ptr) ntdll.RtlEncodePointer
@ stdcall EncodeSystemPointer(ptr) ntdll.RtlEncodeSystemPointer
@ stdcall EndUpdateResourceA(long long)
@ stdcall EndUpdateResourceW(long long)
@ stdcall EnterCriticalSection(ptr) ntdll.RtlEnterCriticalSection
@ stdcall EnumCalendarInfoA(ptr long long long)
@ stdcall EnumCalendarInfoExA(ptr long long long)
@ stdcall EnumCalendarInfoExEx(ptr wstr long wstr long long)
@ stdcall EnumCalendarInfoExW(ptr long long long)
@ stdcall EnumCalendarInfoW(ptr long long long)
@ stdcall EnumDateFormatsA(ptr long long)
@ stdcall EnumDateFormatsExA(ptr long long)
@ stdcall EnumDateFormatsExEx(ptr wstr long long)
@ stdcall EnumDateFormatsExW(ptr long long)
@ stdcall EnumDateFormatsW(ptr long long)
@ stdcall EnumLanguageGroupLocalesA(ptr long long ptr)
@ stdcall EnumLanguageGroupLocalesW(ptr long long ptr)
@ stdcall EnumResourceLanguagesA(long str str ptr long)
@ stub -version=0x600+ EnumResourceLanguagesExA
@ stub -version=0x600+ EnumResourceLanguagesExW
@ stdcall EnumResourceLanguagesW(long wstr wstr ptr long)
@ stdcall EnumResourceNamesA(long str ptr long)
@ stub -version=0x600+ EnumResourceNamesExA
@ stub -version=0x600+ EnumResourceNamesExW
@ stdcall EnumResourceNamesW(long wstr ptr long)
@ stdcall EnumResourceTypesA(long ptr long)
@ stub -version=0x600+ EnumResourceTypesExA
@ stub -version=0x600+ EnumResourceTypesExW
@ stdcall EnumResourceTypesW(long ptr long)
@ stdcall EnumSystemCodePagesA(ptr long)
@ stdcall EnumSystemCodePagesW(ptr long)
@ stdcall EnumSystemFirmwareTables(long ptr long)
@ stdcall EnumSystemGeoID(long long ptr)
@ stdcall EnumSystemLanguageGroupsA(ptr long ptr)
@ stdcall EnumSystemLanguageGroupsW(ptr long ptr)
@ stdcall EnumSystemLocalesA(ptr long)
@ stdcall -version=0x600+ EnumSystemLocalesEx(ptr long long ptr)
@ stdcall EnumSystemLocalesW(ptr long)
@ stdcall EnumTimeFormatsA(ptr long long)
@ stdcall EnumTimeFormatsEx(ptr wstr long long)
@ stdcall EnumTimeFormatsW(ptr long long)
@ stdcall EnumUILanguagesA(ptr long long)
@ stdcall EnumUILanguagesW(ptr long long)
@ stdcall EnumerateLocalComputerNamesA(ptr long str ptr)
@ stdcall EnumerateLocalComputerNamesW(ptr long wstr ptr)
@ stdcall EraseTape(ptr long long)
@ stdcall EscapeCommFunction(long long)
@ stdcall ExitProcess(long) ; FIXME: ntdll.RtlExitUserProcess
@ stdcall ExitThread(long) ; FIXME: ntdll.RtlExitUserThread
@ stdcall ExitVDM(long long)
@ stdcall ExpandEnvironmentStringsA(str ptr long)
@ stdcall ExpandEnvironmentStringsW(wstr ptr long)
@ stdcall ExpungeConsoleCommandHistoryA(long)
@ stdcall ExpungeConsoleCommandHistoryW(long)
@ stdcall FatalAppExitA(long str)
@ stdcall FatalAppExitW(long wstr)
@ stdcall FatalExit(long)
@ stdcall FileTimeToDosDateTime(ptr ptr ptr)
@ stdcall FileTimeToLocalFileTime(ptr ptr)
@ stdcall FileTimeToSystemTime(ptr ptr)
@ stdcall FillConsoleOutputAttribute(long long long long ptr)
@ stdcall FillConsoleOutputCharacterA(long long long long ptr)
@ stdcall FillConsoleOutputCharacterW(long long long long ptr)
@ stdcall FindActCtxSectionGuid(long ptr long ptr ptr)
@ stdcall FindActCtxSectionStringA(long ptr long str ptr)
@ stdcall FindActCtxSectionStringW(long ptr long wstr ptr)
@ stdcall FindAtomA(str)
@ stdcall FindAtomW(wstr)
@ stdcall FindClose(long)
@ stdcall FindCloseChangeNotification(long)
@ stdcall FindFirstChangeNotificationA(str long long)
@ stdcall FindFirstChangeNotificationW(wstr long long)
@ stdcall FindFirstFileA(str ptr)
@ stdcall FindFirstFileExA(str long ptr long ptr long)
@ stdcall FindFirstFileExW(wstr long ptr long ptr long)
@ stub -version=0x600+ FindFirstFileNameTransactedW
@ stub -version=0x600+ FindFirstFileNameW
@ stub -version=0x600+ FindFirstFileTransactedA
@ stub -version=0x600+ FindFirstFileTransactedW
@ stdcall FindFirstFileW(wstr ptr)
@ stub -version=0x600+ FindFirstStreamTransactedW
@ stdcall FindFirstStreamW(wstr ptr ptr long)
@ stdcall FindFirstVolumeA(ptr long)
@ stdcall FindFirstVolumeMountPointA(str ptr long)
@ stdcall FindFirstVolumeMountPointW(wstr ptr long)
@ stdcall FindFirstVolumeW(ptr long)
@ stub -version=0x600+ FindNLSString
@ stub -version=0x600+ FindNLSStringEx
@ stdcall FindNextChangeNotification(long)
@ stdcall FindNextFileA(long ptr)
@ stub -version=0x600+ FindNextFileNameW
@ stdcall FindNextFileW(long ptr)
@ stdcall FindNextStreamW(ptr ptr)
@ stdcall FindNextVolumeA(long ptr long)
@ stdcall FindNextVolumeMountPointA(long str long)
@ stdcall FindNextVolumeMountPointW(long wstr long)
@ stdcall FindNextVolumeW(long ptr long)
@ stdcall FindResourceA(long str str)
@ stdcall FindResourceExA(long str str long)
@ stdcall FindResourceExW(long wstr wstr long)
@ stdcall FindResourceW(long wstr wstr)
@ stdcall FindVolumeClose(ptr)
@ stdcall FindVolumeMountPointClose(ptr)
@ stdcall FlsAlloc(ptr)
@ stdcall FlsFree(long)
@ stdcall FlsGetValue(long)
@ stdcall FlsSetValue(long ptr)
@ stdcall FlushConsoleInputBuffer(long)
@ stdcall FlushFileBuffers(long)
@ stdcall FlushInstructionCache(long long long)
@ stub -version=0x600+ FlushProcessWriteBuffers
@ stdcall FlushViewOfFile(ptr long)
@ stdcall FoldStringA(long str long ptr long)
@ stdcall FoldStringW(long wstr long ptr long)
@ stdcall FormatMessageA(long ptr long long ptr long ptr)
@ stdcall FormatMessageW(long ptr long long ptr long ptr)
@ stdcall FreeConsole()
@ stdcall FreeEnvironmentStringsA(ptr)
@ stdcall FreeEnvironmentStringsW(ptr)
@ stdcall FreeLibrary(long)
@ stdcall FreeLibraryAndExitThread(long long)
@ stub -version=0x600+ FreeLibraryWhenCallbackReturns
@ stdcall FreeResource(long)
@ stdcall FreeUserPhysicalPages(long long long)
@ stdcall GenerateConsoleCtrlEvent(long long)
@ stdcall GetACP()
@ stdcall GetActiveProcessorCount(long)
@ stdcall GetActiveProcessorGroupCount()
@ stdcall -version=0x600+ GetApplicationRecoveryCallback(ptr ptr ptr ptr ptr)
@ stub -version=0x600+ GetApplicationRestartSettings
@ stdcall GetAtomNameA(long ptr long)
@ stdcall GetAtomNameW(long ptr long)
@ stdcall GetBinaryType(str ptr) GetBinaryTypeA
@ stdcall GetBinaryTypeA(str ptr)
@ stdcall GetBinaryTypeW(wstr ptr)
@ stdcall -version=0x501-0x600 GetCPFileNameFromRegistry(long wstr long)
@ stdcall GetCPInfo(long ptr)
@ stdcall GetCPInfoExA(long long ptr)
@ stdcall GetCPInfoExW(long long ptr)
@ stub -version=0x600+ GetCalendarDateFormat
@ stub -version=0x600+ GetCalendarDateFormatEx
@ stub -version=0x600+ GetCalendarDaysInMonth
@ stub -version=0x600+ GetCalendarDifferenceInDays
@ stdcall GetCalendarInfoA(long long long ptr long ptr)
@ stdcall GetCalendarInfoEx(wstr long wstr long wstr long ptr)
@ stdcall GetCalendarInfoW(long long long ptr long ptr)
@ stub -version=0x600+ GetCalendarMonthsInYear
@ stub -version=0x600+ GetCalendarSupportedDateRange
@ stub -version=0x600+ GetCalendarWeekNumber
@ stdcall GetComPlusPackageInstallStatus()
@ stdcall GetCommConfig(long ptr long)
@ stdcall GetCommMask(long ptr)
@ stdcall GetCommModemStatus(long ptr)
@ stdcall GetCommProperties(long ptr)
@ stdcall GetCommState(long ptr)
@ stdcall GetCommTimeouts(long ptr)
@ stdcall GetCommandLineA()
@ stdcall GetCommandLineW()
@ stdcall GetCompressedFileSizeA(long ptr)
@ stub -version=0x600+ GetCompressedFileSizeTransactedA
@ stub -version=0x600+ GetCompressedFileSizeTransactedW
@ stdcall GetCompressedFileSizeW(long ptr)
@ stdcall GetComputerNameA(ptr ptr)
@ stdcall GetComputerNameExA(long ptr ptr)
@ stdcall GetComputerNameExW(long ptr ptr)
@ stdcall GetComputerNameW(ptr ptr)
@ stdcall GetConsoleAliasA(str str long str)
@ stdcall GetConsoleAliasExesA(str long)
@ stdcall GetConsoleAliasExesLengthA()
@ stdcall GetConsoleAliasExesLengthW()
@ stdcall GetConsoleAliasExesW(wstr long)
@ stdcall GetConsoleAliasW(wstr ptr long wstr)
@ stdcall GetConsoleAliasesA(str long str)
@ stdcall GetConsoleAliasesLengthA(str)
@ stdcall GetConsoleAliasesLengthW(wstr)
@ stdcall GetConsoleAliasesW(wstr long wstr)
@ stdcall GetConsoleCP()
@ stdcall GetConsoleCharType(long long ptr)
@ stdcall GetConsoleCommandHistoryA(long long long)
@ stdcall GetConsoleCommandHistoryLengthA(long)
@ stdcall GetConsoleCommandHistoryLengthW(long)
@ stdcall GetConsoleCommandHistoryW(long long long)
@ stdcall GetConsoleCursorInfo(long ptr)
@ stdcall GetConsoleCursorMode(long ptr ptr)
@ stdcall GetConsoleDisplayMode(ptr)
@ stdcall GetConsoleFontInfo(long long long ptr)
@ stdcall GetConsoleFontSize(long long)
@ stdcall GetConsoleHardwareState(long long ptr)
@ stdcall -version=0x600+ GetConsoleHistoryInfo(ptr)
@ stdcall GetConsoleInputExeNameA(long ptr)
@ stdcall GetConsoleInputExeNameW(long ptr)
@ stdcall GetConsoleInputWaitHandle()
@ stdcall GetConsoleKeyboardLayoutNameA(ptr)
@ stdcall GetConsoleKeyboardLayoutNameW(ptr)
@ stdcall GetConsoleMode(long ptr)
@ stdcall GetConsoleNlsMode(long ptr)
@ stdcall -version=0x600+ GetConsoleOriginalTitleA(ptr long)
@ stdcall -version=0x600+ GetConsoleOriginalTitleW(ptr long)
@ stdcall GetConsoleOutputCP()
@ stdcall GetConsoleProcessList(ptr long) ; missing in XP SP3
@ stdcall GetConsoleScreenBufferInfo(long ptr)
@ stdcall -version=0x600+ GetConsoleScreenBufferInfoEx(ptr ptr)
@ stdcall GetConsoleSelectionInfo(ptr)
@ stdcall GetConsoleTitleA(ptr long)
@ stdcall GetConsoleTitleW(ptr long)
@ stdcall GetConsoleWindow()
@ stdcall GetCurrencyFormatA(long long str ptr str long)
@ stdcall GetCurrencyFormatEx(wstr long wstr ptr wstr long)
@ stdcall GetCurrencyFormatW(long long wstr ptr wstr long)
@ stdcall GetCurrentActCtx(ptr)
@ stdcall GetCurrentConsoleFont(long long ptr)
@ stdcall -version=0x600+ GetCurrentConsoleFontEx(ptr long ptr)
@ stdcall GetCurrentDirectoryA(long ptr)
@ stdcall GetCurrentDirectoryW(long ptr)
@ stdcall GetCurrentPackageId(ptr ptr)
@ stdcall -norelay GetCurrentProcess()
@ stdcall -norelay GetCurrentProcessId()
@ stdcall GetCurrentProcessorNumber() ntdll.RtlGetCurrentProcessorNumber
@ stdcall GetCurrentProcessorNumberEx(ptr) ntdll.RtlGetCurrentProcessorNumberEx
@ stdcall -norelay GetCurrentThread()
@ stdcall -norelay GetCurrentThreadId()
@ stdcall GetDateFormatA(long long ptr str ptr long)
@ stdcall GetDateFormatEx(wstr long ptr wstr wstr long wstr)
@ stdcall GetDateFormatW(long long ptr wstr ptr long)
@ stdcall GetDefaultCommConfigA(str ptr long)
@ stdcall GetDefaultCommConfigW(wstr ptr long)
@ stdcall -version=0x501-0x502 GetDefaultSortkeySize(ptr)
@ stdcall GetDevicePowerState(long ptr)
@ stdcall GetDiskFreeSpaceA(str ptr ptr ptr ptr)
@ stdcall GetDiskFreeSpaceExA(str ptr ptr ptr)
@ stdcall GetDiskFreeSpaceExW(wstr ptr ptr ptr)
@ stdcall GetDiskFreeSpaceW(wstr ptr ptr ptr ptr)
@ stdcall GetDllDirectoryA(long ptr)
@ stdcall GetDllDirectoryW(long ptr)
@ stdcall GetDriveTypeA(str)
@ stdcall GetDriveTypeW(wstr)
@ stub -version=0x600+ GetDurationFormat
@ stub -version=0x600+ GetDurationFormatEx
@ stub -version=0x600+ GetDynamicTimeZoneInformation
@ stdcall GetEnvironmentStrings()
@ stdcall GetEnvironmentStringsA() GetEnvironmentStrings
@ stdcall GetEnvironmentStringsW()
@ stdcall GetEnvironmentVariableA(str ptr long)
@ stdcall GetEnvironmentVariableW(wstr ptr long)
@ stdcall GetErrorMode()
@ stdcall GetExitCodeProcess(long ptr)
@ stdcall GetExitCodeThread(long ptr)
@ stdcall GetExpandedNameA(str ptr)
@ stdcall GetExpandedNameW(wstr ptr)
@ stdcall GetFileAttributesA(str)
@ stdcall GetFileAttributesExA(str long ptr)
@ stdcall GetFileAttributesExW(wstr long ptr)
@ stub -version=0x600+ GetFileAttributesTransactedA
@ stub -version=0x600+ GetFileAttributesTransactedW
@ stdcall GetFileAttributesW(wstr)
@ stdcall -version=0x600+ GetFileBandwidthReservation(ptr ptr ptr ptr ptr ptr)
@ stdcall GetFileInformationByHandle(long ptr)
@ stdcall -arch=arm64 GetFileInformationByHandleEx(ptr long ptr long) kernel32_vista.GetFileInformationByHandleEx
@ stdcall -version=0x600+ GetFileMUIInfo(long wstr ptr ptr)
@ stdcall -version=0x600+ GetFileMUIPath(long wstr wstr ptr wstr ptr ptr)
@ stdcall GetFileSize(long ptr)
@ stdcall GetFileSizeEx(long ptr)
@ stdcall GetFileTime(long ptr ptr ptr)
@ stdcall GetFileType(long)
@ stdcall -version=0x600+ GetFinalPathNameByHandleA(ptr str long long)
@ stdcall -arch=arm64 GetFinalPathNameByHandleW(ptr wstr long long) kernel32_vista.GetFinalPathNameByHandleW
@ stdcall GetFirmwareEnvironmentVariableA(str str ptr long)
@ stdcall -version=0x602+ GetFirmwareEnvironmentVariableExA(str str ptr long long)
@ stdcall -version=0x602+ GetFirmwareEnvironmentVariableExW(wstr wstr ptr long long)
@ stdcall GetFirmwareEnvironmentVariableW(wstr wstr ptr long)
@ stdcall -version=0x602+ GetFirmwareType(ptr)
@ stdcall GetFullPathNameA(str long ptr ptr)
@ stub -version=0x600+ GetFullPathNameTransactedA
@ stub -version=0x600+ GetFullPathNameTransactedW
@ stdcall GetFullPathNameW(wstr long ptr ptr)
@ stdcall GetGeoInfoA(long long ptr long long)
@ stdcall GetGeoInfoW(long long ptr long long)
@ stdcall -i386 GetHandleContext(long) ; missing on x64
@ stdcall GetHandleInformation(long ptr)
@ stdcall GetLargePageMinimum()
@ stdcall GetLargestConsoleWindowSize(long)
@ stdcall GetLastError() ntdll.RtlGetLastWin32Error
@ stdcall -version=0x500-0x502 GetLinguistLangSize(ptr)
@ stdcall GetLocalTime(ptr)
@ stdcall GetLocaleInfoA(long long ptr long)
@ stdcall -version=0x600+ GetLocaleInfoEx(wstr long ptr long)
@ stdcall GetLocaleInfoW(long long ptr long)
@ stdcall -version=0x600+ IsValidLocaleName(wstr)
@ stdcall GetLogicalDriveStringsA(long ptr)
@ stdcall GetLogicalDriveStringsW(long ptr)
@ stdcall GetLogicalDrives()
@ stdcall GetLogicalProcessorInformation(ptr ptr)
@ stdcall GetLogicalProcessorInformationEx(long ptr ptr)
@ stdcall GetLongPathNameA(str long long)
@ stub -version=0x600+ GetLongPathNameTransactedA
@ stub -version=0x600+ GetLongPathNameTransactedW
@ stdcall GetLongPathNameW(wstr long long)
@ stdcall GetMailslotInfo(long ptr ptr ptr ptr)
@ stdcall GetMaximumProcessorCount(long)
@ stdcall GetModuleFileNameA(long ptr long)
@ stdcall GetModuleFileNameW(long ptr long)
@ stdcall GetModuleHandleA(str)
@ stdcall GetModuleHandleExA(long ptr ptr)
@ stdcall GetModuleHandleExW(long ptr ptr)
@ stdcall GetModuleHandleW(wstr)
@ stdcall GetNLSVersion(long long ptr)
@ stdcall GetNLSVersionEx(long wstr ptr)
@ stub -version=0x600+ GetNamedPipeAttribute
@ stub -version=0x600+ GetNamedPipeClientComputerNameA
@ stub -version=0x600+ GetNamedPipeClientComputerNameW
@ stdcall -version=0x600+ GetNamedPipeClientProcessId(ptr ptr)
@ stub -version=0x600+ GetNamedPipeClientSessionId
@ stdcall GetNamedPipeHandleStateA(long ptr ptr ptr ptr str long)
@ stdcall GetNamedPipeHandleStateW(long ptr ptr ptr ptr wstr long)
@ stdcall GetNamedPipeInfo(long ptr ptr ptr ptr)
@ stub -version=0x600+ GetNamedPipeServerProcessId
@ stub -version=0x600+ GetNamedPipeServerSessionId
@ stdcall GetNativeSystemInfo(ptr)
@ stdcall GetNextVDMCommand(long)
@ stdcall -version=0x500-0x502 GetNlsSectionName(long long long str str long)
@ stdcall GetNumaAvailableMemoryNode(long ptr)
@ stdcall GetNumaHighestNodeNumber(ptr)
@ stdcall -stub -version=0x601+ GetNumaNodeProcessorMaskEx(long ptr)
@ stdcall GetNumaNodeProcessorMask(long ptr)
@ stdcall GetNumaProcessorNode(long ptr)
@ stub -version=0x600+ GetNumaProximityNode
@ stdcall GetNumberFormatA(long long str ptr ptr long)
@ stdcall GetNumberFormatEx(wstr long wstr ptr wstr long)
@ stdcall GetNumberFormatW(long long wstr ptr ptr long)
@ stdcall GetNumberOfConsoleFonts()
@ stdcall GetNumberOfConsoleInputEvents(long ptr)
@ stdcall GetNumberOfConsoleMouseButtons(ptr)
@ stdcall GetOEMCP()
@ stdcall GetOverlappedResult(long ptr ptr long)
@ stdcall -stub -version=0x600+ GetPhysicallyInstalledSystemMemory(ptr)
@ stdcall GetPriorityClass(long)
@ stdcall GetPrivateProfileIntA(str str long str)
@ stdcall GetPrivateProfileIntW(wstr wstr long wstr)
@ stdcall GetPrivateProfileSectionA(str ptr long str)
@ stdcall GetPrivateProfileSectionNamesA(ptr long str)
@ stdcall GetPrivateProfileSectionNamesW(ptr long wstr)
@ stdcall GetPrivateProfileSectionW(wstr ptr long wstr)
@ stdcall GetPrivateProfileStringA(str str str ptr long str)
@ stdcall GetPrivateProfileStringW(wstr wstr wstr ptr long wstr)
@ stdcall GetPrivateProfileStructA(str str ptr long str)
@ stdcall GetPrivateProfileStructW(wstr wstr ptr long wstr)
@ stdcall GetProcAddress(long str)
@ stdcall GetProcessAffinityMask(long ptr ptr)
@ stdcall -stub -version=0x601+ GetProcessGroupAffinity(ptr ptr ptr)
@ stdcall GetProcessDEPPolicy(ptr ptr ptr)
@ stdcall GetProcessHandleCount(long ptr)
@ stdcall -norelay GetProcessHeap()
@ stdcall GetProcessHeaps(long ptr)
@ stdcall GetProcessId(long)
@ stdcall GetProcessIdOfThread(ptr)
@ stdcall GetProcessIoCounters(long ptr)
@ stdcall GetProcessPriorityBoost(long ptr)
@ stdcall GetProcessShutdownParameters(ptr ptr)
@ stdcall GetProcessTimes(long ptr ptr ptr ptr)
@ stdcall GetProcessVersion(long)
@ stdcall GetProcessWorkingSetSize(long ptr ptr)
@ stdcall GetProcessWorkingSetSizeEx(long ptr ptr long)
@ stdcall -version=0x600+ GetProductInfo(long long long long ptr) ntdll.RtlGetProductInfo
@ stdcall GetProfileIntA(str str long)
@ stdcall GetProfileIntW(wstr wstr long)
@ stdcall GetProfileSectionA(str ptr long)
@ stdcall GetProfileSectionW(wstr ptr long)
@ stdcall GetProfileStringA(str str str ptr long)
@ stdcall GetProfileStringW(wstr wstr wstr ptr long)
@ stdcall GetQueuedCompletionStatus(long ptr ptr ptr long)
@ stub -version=0x600+ GetQueuedCompletionStatusEx
@ stdcall GetShortPathNameA(str ptr long)
@ stdcall GetShortPathNameW(wstr ptr long)
@ stdcall GetStartupInfoA(ptr)
@ stdcall GetStartupInfoW(ptr)
@ stdcall GetStdHandle(long)
@ stub -version=0x600+ GetStringScripts
@ stdcall GetStringTypeA(long long str long ptr)
@ stdcall GetStringTypeExA(long long str long ptr)
@ stdcall GetStringTypeExW(long long wstr long ptr)
@ stdcall GetStringTypeW(long wstr long ptr)
@ stdcall GetSystemDEPPolicy()
@ stdcall GetSystemDefaultLCID()
@ stdcall GetSystemDefaultLangID()
@ stdcall -version=0x600+ GetSystemDefaultLocaleName(ptr long)
@ stdcall GetSystemDefaultUILanguage()
@ stdcall GetSystemDirectoryA(ptr long)
@ stdcall GetSystemDirectoryW(ptr long)
@ stdcall GetSystemFileCacheSize(ptr ptr ptr)
@ stdcall GetSystemFirmwareTable(long long ptr long)
@ stdcall GetSystemInfo(ptr)
@ stdcall GetSystemPowerStatus(ptr)
@ stdcall -version=0x600+ GetSystemPreferredUILanguages(long ptr wstr ptr)
@ stdcall GetSystemRegistryQuota(ptr ptr)
@ stdcall GetSystemTime(ptr)
@ stdcall GetSystemTimeAdjustment(ptr ptr ptr)
@ stdcall GetSystemTimeAsFileTime(ptr)
@ stdcall -version=0x602+ GetSystemTimePreciseAsFileTime(ptr)
@ stdcall GetSystemTimes(ptr ptr ptr)
@ stdcall GetSystemWindowsDirectoryA(ptr long)
@ stdcall GetSystemWindowsDirectoryW(ptr long)
@ stdcall GetSystemWow64DirectoryA(ptr long)
@ stdcall GetSystemWow64DirectoryW(ptr long)
@ stdcall GetTapeParameters(ptr long ptr ptr)
@ stdcall GetTapePosition(ptr long ptr ptr ptr)
@ stdcall GetTapeStatus(ptr)
@ stdcall GetTempFileNameA(str str long ptr)
@ stdcall GetTempFileNameW(wstr wstr long ptr)
@ stdcall GetTempPathA(long ptr)
@ stdcall GetTempPathW(long ptr)
@ stdcall GetThreadContext(long ptr)
@ stdcall GetThreadErrorMode()
@ stdcall GetThreadGroupAffinity(ptr ptr)
@ stdcall GetThreadIOPendingFlag(long ptr)
@ stdcall GetThreadId(ptr)
@ stdcall GetThreadLocale()
@ stdcall -version=0x600+ GetThreadPreferredUILanguages(long ptr wstr ptr)
@ stdcall GetThreadPriority(long)
@ stdcall GetThreadPriorityBoost(long ptr)
@ stdcall GetThreadSelectorEntry(long long ptr)
@ stdcall GetThreadTimes(long ptr ptr ptr ptr)
@ stdcall -version=0x600+ GetThreadUILanguage()
@ stdcall GetTickCount()
@ stdcall -version=0x600+ -ret64 GetTickCount64()
@ stdcall GetTimeFormatA(long long ptr str ptr long)
@ stdcall GetTimeFormatEx(wstr long ptr wstr wstr long)
@ stdcall GetTimeFormatW(long long ptr wstr ptr long)
@ stdcall GetTimeZoneInformation(ptr)
@ stdcall -stub -version=0x600+ GetTimeZoneInformationForYear(long ptr ptr)
@ stdcall -version=0x600+ GetUILanguageInfo(long wstr wstr ptr ptr)
@ stdcall GetUserDefaultLCID()
@ stdcall GetUserDefaultLangID()
@ stdcall -version=0x600+ GetUserDefaultLocaleName(wstr long)
@ stdcall GetUserDefaultUILanguage()
@ stdcall GetUserGeoID(long)
@ stdcall -version=0x600+ GetUserPreferredUILanguages(long ptr wstr ptr)
@ stdcall GetVDMCurrentDirectories(long long)
@ stdcall GetVersion()
@ stdcall GetVersionExA(ptr)
@ stdcall GetVersionExW(ptr)
@ stdcall GetVolumeInformationA(str ptr long ptr ptr ptr ptr long)
@ stub -version=0x600+ GetVolumeInformationByHandleW
@ stdcall GetVolumeInformationW(wstr ptr long ptr ptr ptr ptr long)
@ stdcall GetVolumeNameForVolumeMountPointA(str ptr long)
@ stdcall GetVolumeNameForVolumeMountPointW(wstr ptr long)
@ stdcall GetVolumePathNameA(str ptr long)
@ stdcall GetVolumePathNameW(wstr ptr long)
@ stdcall GetVolumePathNamesForVolumeNameA(str str long ptr)
@ stdcall GetVolumePathNamesForVolumeNameW(wstr wstr long ptr)
@ stdcall GetWindowsDirectoryA(ptr long)
@ stdcall GetWindowsDirectoryW(ptr long)
@ stdcall GetWriteWatch(long ptr long ptr ptr ptr)
@ stdcall GlobalAddAtomA(str)
@ stdcall GlobalAddAtomW(wstr)
@ stdcall GlobalAlloc(long long)
@ stdcall GlobalCompact(long)
@ stdcall GlobalDeleteAtom(long)
@ stdcall GlobalFindAtomA(str)
@ stdcall GlobalFindAtomW(wstr)
@ stdcall GlobalFix(long)
@ stdcall GlobalFlags(long)
@ stdcall GlobalFree(long)
@ stdcall GlobalGetAtomNameA(long ptr long)
@ stdcall GlobalGetAtomNameW(long ptr long)
@ stdcall GlobalHandle(ptr)
@ stdcall GlobalLock(long)
@ stdcall GlobalMemoryStatus(ptr)
@ stdcall GlobalMemoryStatusEx(ptr)
@ stdcall GlobalReAlloc(long long long)
@ stdcall GlobalSize(long)
@ stdcall GlobalUnWire(long)
@ stdcall GlobalUnfix(long)
@ stdcall GlobalUnlock(long)
@ stdcall GlobalWire(long)
@ stdcall Heap32First(ptr long long)
@ stdcall Heap32ListFirst(long ptr)
@ stdcall Heap32ListNext(long ptr)
@ stdcall Heap32Next(ptr)
@ stdcall HeapAlloc(long long long) ntdll.RtlAllocateHeap
@ stdcall HeapCompact(long long)
@ stdcall HeapCreate(long long long)
@ stdcall -version=0x351-0x502 HeapCreateTagsW(ptr long wstr wstr)
@ stdcall HeapDestroy(long)
@ stdcall -version=0x351-0x502 HeapExtend(long long ptr long)
@ stdcall HeapFree(long long long) ntdll.RtlFreeHeap
@ stdcall HeapLock(long)
@ stdcall HeapQueryInformation(long long ptr long ptr)
@ stdcall -version=0x351-0x502 HeapQueryTagW(long long long long ptr)
@ stdcall HeapReAlloc(long long ptr long) ntdll.RtlReAllocateHeap
@ stdcall HeapSetInformation(ptr long ptr long)
@ stdcall HeapSize(long long ptr) ntdll.RtlSizeHeap
@ stdcall HeapSummary(long long ptr)
@ stdcall HeapUnlock(long)
@ stdcall -version=0x351-0x502 HeapUsage(long long long long ptr)
@ stdcall HeapValidate(long long ptr)
@ stdcall HeapWalk(long ptr)
@ stdcall -stub -version=0x600+ IdnToAscii(long wstr long ptr long)
@ stdcall -stub -version=0x600+ IdnToNameprepUnicode(long wstr long ptr long)
@ stdcall -stub -version=0x600+ IdnToUnicode(long wstr long ptr long)
@ stdcall InitAtomTable(long)
@ stdcall -version=0x600+ InitOnceBeginInitialize(ptr long ptr ptr)
@ stdcall -version=0x600+ InitOnceComplete(ptr long ptr)
@ stdcall -version=0x600+ InitOnceExecuteOnce(ptr ptr ptr ptr)
@ stdcall -version=0x600+ InitOnceInitialize(ptr) ntdll.RtlRunOnceInitialize
@ stdcall -version=0x600+ InitializeConditionVariable(ptr) ntdll.RtlInitializeConditionVariable
@ stdcall InitializeCriticalSection(ptr)
@ stdcall InitializeCriticalSectionAndSpinCount(ptr long)
@ stdcall -version=0x600+ InitializeCriticalSectionEx(ptr long long)
@ stdcall InitializeProcThreadAttributeList(ptr long long ptr)
@ stdcall InitializeSListHead(ptr) ntdll.RtlInitializeSListHead
@ stdcall -version=0x600+ InitializeSRWLock(ptr) ntdll.RtlInitializeSRWLock
@ stdcall -arch=i386 -ret64 InterlockedCompareExchange64(ptr double double) ntdll.RtlInterlockedCompareExchange64
@ stdcall -arch=i386 InterlockedCompareExchange(ptr long long)
@ stdcall -arch=i386 InterlockedDecrement(ptr)
@ stdcall -arch=i386 InterlockedExchange(ptr long)
@ stdcall -arch=i386 InterlockedExchangeAdd(ptr long)
@ stdcall InterlockedFlushSList(ptr) ntdll.RtlInterlockedFlushSList
@ stdcall -arch=i386 InterlockedIncrement(ptr)
@ stdcall InterlockedPopEntrySList(ptr) ntdll.RtlInterlockedPopEntrySList
@ stdcall InterlockedPushEntrySList(ptr ptr) ntdll.RtlInterlockedPushEntrySList
@ fastcall -version=0x600+ InterlockedPushListSList(ptr ptr ptr long) ntdll.RtlInterlockedPushListSList
@ stdcall InvalidateConsoleDIBits(long long)
@ stdcall IsBadCodePtr(ptr)
@ stdcall IsBadHugeReadPtr(ptr long)
@ stdcall IsBadHugeWritePtr(ptr long)
@ stdcall IsBadReadPtr(ptr long)
@ stdcall IsBadStringPtrA(ptr long)
@ stdcall IsBadStringPtrW(ptr long)
@ stdcall IsBadWritePtr(ptr long)
@ stub -version=0x600+ IsCalendarLeapDay
@ stub -version=0x600+ IsCalendarLeapMonth
@ stub -version=0x600+ IsCalendarLeapYear
@ stdcall IsDBCSLeadByte(long)
@ stdcall IsDBCSLeadByteEx(long long)
@ stdcall IsDebuggerPresent()
@ stdcall IsNLSDefinedString(long long ptr long long)
@ stdcall -stub -version=0x600+ IsNormalizedString(long wstr long)
@ stdcall IsProcessInJob(long long ptr)
@ stdcall IsProcessorFeaturePresent(long)
@ stdcall IsSystemResumeAutomatic()
@ stdcall IsThreadAFiber()
@ stdcall IsThreadpoolTimerSet(ptr) ntdll.TpIsTimerSet
@ stdcall IsTimeZoneRedirectionEnabled()
@ stub -version=0x600+ IsValidCalDateTime
@ stdcall IsValidCodePage(long)
@ stdcall IsValidLanguageGroup(long long)
@ stdcall IsValidLocale(long long)
@ stdcall -version=0x501-0x502 IsValidUILanguage(long)
@ stdcall IsWow64Process(ptr ptr)
@ stdcall K32EmptyWorkingSet(long) EmptyWorkingSet
@ stdcall K32EnumDeviceDrivers(ptr long ptr) EnumDeviceDrivers
@ stdcall K32EnumPageFilesA(ptr ptr) EnumPageFilesA
@ stdcall K32EnumPageFilesW(ptr ptr) EnumPageFilesW
@ stdcall K32EnumProcessModules(long ptr long ptr) EnumProcessModules
@ stdcall -stub -version=0x601+ K32EnumProcessModulesEx(long ptr long ptr long); EnumProcessModulesEx
@ stdcall K32EnumProcesses(ptr long ptr) EnumProcesses
@ stdcall K32GetDeviceDriverBaseNameA(ptr ptr long) GetDeviceDriverBaseNameA
@ stdcall K32GetDeviceDriverBaseNameW(ptr ptr long) GetDeviceDriverBaseNameW
@ stdcall K32GetDeviceDriverFileNameA(ptr ptr long) GetDeviceDriverFileNameA
@ stdcall K32GetDeviceDriverFileNameW(ptr ptr long) GetDeviceDriverFileNameW
@ stdcall K32GetMappedFileNameA(long ptr ptr long) GetMappedFileNameA
@ stdcall K32GetMappedFileNameW(long ptr ptr long) GetMappedFileNameW
@ stdcall K32GetModuleBaseNameA(long long ptr long) GetModuleBaseNameA
@ stdcall K32GetModuleBaseNameW(long long ptr long) GetModuleBaseNameW
@ stdcall K32GetModuleFileNameExA(long long ptr long) GetModuleFileNameExA
@ stdcall K32GetModuleFileNameExW(long long ptr long) GetModuleFileNameExW
@ stdcall K32GetModuleInformation(long long ptr long) GetModuleInformation
@ stdcall K32GetPerformanceInfo(ptr long) GetPerformanceInfo
@ stdcall K32GetProcessImageFileNameA(long ptr long) GetProcessImageFileNameA
@ stdcall K32GetProcessImageFileNameW(long ptr long) GetProcessImageFileNameW
@ stdcall K32GetProcessMemoryInfo(long ptr long) GetProcessMemoryInfo
@ stdcall K32GetWsChanges(long ptr long) GetWsChanges
@ stdcall -stub -version=0x601+ K32GetWsChangesEx(long ptr ptr); GetWsChangesEx
@ stdcall K32InitializeProcessForWsWatch(long) InitializeProcessForWsWatch
@ stdcall K32QueryWorkingSet(long ptr long) QueryWorkingSet
@ stdcall K32QueryWorkingSetEx(long ptr long) QueryWorkingSetEx
@ stdcall -version=0x600+ LCIDToLocaleName(long wstr long long)
@ stdcall LCMapStringA(long long str long ptr long)
@ stdcall LCMapStringEx(long long wstr long ptr long ptr ptr long)
@ stdcall LCMapStringW(long long wstr long ptr long)
@ stdcall LZClose(long)
@ stdcall LZCloseFile(long)
@ stdcall LZCopy(long long)
@ stdcall LZCreateFileW(ptr long long long ptr)
@ stdcall LZDone()
@ stdcall LZInit(long)
@ stdcall LZOpenFileA(str ptr long)
@ stdcall LZOpenFileW(wstr ptr long)
@ stdcall LZRead(long str long)
@ stdcall LZSeek(long long long)
@ stdcall LZStart()
@ stdcall LeaveCriticalSection(ptr) ntdll.RtlLeaveCriticalSection
@ stub -version=0x600+ LeaveCriticalSectionWhenCallbackReturns
@ stdcall LoadLibraryA(str)
@ stdcall LoadLibraryExA(str long long)
@ stdcall LoadLibraryExW(wstr long long)
@ stdcall LoadLibraryW(wstr)
@ stdcall LoadModule(str ptr)
@ stdcall LoadResource(long long)
@ stub -version=0x600+ LoadStringBaseExW
@ stub -version=0x600+ LoadStringBaseW
@ stdcall LocalAlloc(long long)
@ stdcall LocalCompact(long)
@ stdcall LocalFileTimeToFileTime(ptr ptr)
@ stdcall LocalFlags(long)
@ stdcall LocalFree(long)
@ stdcall LocalHandle(ptr)
@ stdcall LocalLock(long)
@ stdcall LocalReAlloc(long long long)
@ stdcall LocalShrink(long long)
@ stdcall LocalSize(long)
@ stdcall LocalUnlock(long)
@ stdcall -version=0x600+ LocaleNameToLCID(wstr long)
@ stdcall LockFile(long long long long long)
@ stdcall LockFileEx(long long long long long ptr)
@ stdcall LockResource(long)
@ stdcall MapUserPhysicalPages(ptr long ptr)
@ stdcall MapUserPhysicalPagesScatter(ptr long ptr)
@ stdcall MapViewOfFile(long long long long long)
@ stdcall MapViewOfFileEx(long long long long long ptr)
@ stub -version=0x600+ MapViewOfFileExNuma
@ stdcall Module32First(long ptr)
@ stdcall Module32FirstW(long ptr)
@ stdcall Module32Next(long ptr)
@ stdcall Module32NextW(long ptr)
@ stdcall MoveFileA(str str)
@ stdcall MoveFileExA(str str long)
@ stdcall MoveFileExW(wstr wstr long)
@ stub -version=0x600+ MoveFileTransactedA
@ stub -version=0x600+ MoveFileTransactedW
@ stdcall MoveFileW(wstr wstr)
@ stdcall MoveFileWithProgressA(str str ptr ptr long)
@ stdcall MoveFileWithProgressW(wstr wstr ptr ptr long)
@ stdcall MulDiv(long long long)
@ stdcall MultiByteToWideChar(long long str long ptr long)
@ stdcall NeedCurrentDirectoryForExePathA(str)
@ stdcall NeedCurrentDirectoryForExePathW(wstr)
@ stub -version=0x600+ NlsCheckPolicy
@ stdcall -version=0x500-0x600 NlsConvertIntegerToString(long long long wstr long)
@ stub -version=0x600+ NlsEventDataDescCreate
@ stdcall NlsGetCacheUpdateCount()
@ stdcall -version=0x500-0x502 NlsResetProcessLocale()
@ stub -version=0x600+ NlsUpdateLocale
@ stub -version=0x600+ NlsUpdateSystemLocale
@ stub -version=0x600+ NlsWriteEtwEvent
@ stdcall -stub -version=0x600+ NormalizeString(long wstr long ptr long)
@ stdcall -stub -version=0x600+ NotifyUILanguageChange(long wstr wstr long ptr)
@ stdcall OpenConsoleW(wstr long long long)
@ stdcall -version=0x500-0x502 OpenDataFile(long long)
@ stdcall OpenEventA(long long str)
@ stdcall OpenEventW(long long wstr)
@ stdcall OpenFile(str ptr long)
@ stdcall -version=0x600+ OpenFileById(ptr ptr long long ptr long)
@ stdcall OpenFileMappingA(long long str)
@ stdcall OpenFileMappingW(long long wstr)
@ stdcall OpenJobObjectA(long long str)
@ stdcall OpenJobObjectW(long long wstr)
@ stdcall OpenMutexA(long long str)
@ stdcall OpenMutexW(long long wstr)
@ stub -version=0x600+ OpenPrivateNamespaceA
@ stub -version=0x600+ OpenPrivateNamespaceW
@ stdcall OpenProcess(long long long)
@ stdcall OpenProfileUserMapping()
@ stdcall OpenSemaphoreA(long long str)
@ stdcall OpenSemaphoreW(long long wstr)
@ stdcall OpenThread(long long long)
@ stdcall OpenWaitableTimerA(long long str)
@ stdcall OpenWaitableTimerW(long long wstr)
@ stdcall OutputDebugStringA(str)
@ stdcall OutputDebugStringW(wstr)
@ stdcall PeekConsoleInputA(ptr ptr long ptr)
@ stdcall PeekConsoleInputW(ptr ptr long ptr)
@ stdcall PeekNamedPipe(long ptr long ptr ptr ptr)
@ stdcall PostQueuedCompletionStatus(long long ptr ptr)
@ stdcall PrepareTape(ptr long long)
@ stdcall PrivCopyFileExW(wstr wstr ptr ptr long long)
@ stdcall PrivMoveFileIdentityW(long long long)
@ stdcall Process32First(ptr ptr)
@ stdcall Process32FirstW(ptr ptr)
@ stdcall Process32Next(ptr ptr)
@ stdcall Process32NextW(ptr ptr)
@ stdcall ProcessIdToSessionId(long ptr)
@ stdcall PulseEvent(long)
@ stdcall PurgeComm(long long)
@ stub -version=0x600+ QueryActCtxSettingsW
@ stdcall QueryActCtxW(long ptr ptr long ptr long ptr)
@ stdcall QueryDepthSList(ptr) ntdll.RtlQueryDepthSList
@ stdcall QueryDosDeviceA(str ptr long)
@ stdcall QueryDosDeviceW(wstr ptr long)
@ stdcall -version=0x600+ QueryFullProcessImageNameA(ptr long ptr ptr)
@ stdcall -version=0x600+ QueryFullProcessImageNameW(ptr long ptr ptr)
@ stub -version=0x600+ QueryIdleProcessorCycleTime
@ stdcall QueryInformationJobObject(long long ptr long ptr)
@ stdcall QueryMemoryResourceNotification(ptr ptr)
@ stdcall QueryPerformanceCounter(ptr)
@ stdcall QueryPerformanceFrequency(ptr)
@ stub -version=0x600+ QueryProcessAffinityUpdateMode
@ stub -version=0x600+ QueryProcessCycleTime
@ stub -version=0x600+ QueryThreadCycleTime
@ stdcall QueueUserAPC(ptr long long)
@ stdcall QueueUserWorkItem(ptr ptr long)
@ stdcall -norelay RaiseException(long long long ptr)
@ stdcall RaiseFailFastException(ptr ptr long)
@ stdcall ReOpenFile(ptr long long long)
@ stdcall ReadConsoleA(long ptr long ptr ptr)
@ stdcall ReadConsoleInputA(long ptr long ptr)
@ stdcall ReadConsoleInputExA(long ptr long ptr long)
@ stdcall ReadConsoleInputExW(long ptr long ptr long)
@ stdcall ReadConsoleInputW(long ptr long ptr)
@ stdcall ReadConsoleOutputA(long ptr long long ptr)
@ stdcall ReadConsoleOutputAttribute(long ptr long long ptr)
@ stdcall ReadConsoleOutputCharacterA(long ptr long long ptr)
@ stdcall ReadConsoleOutputCharacterW(long ptr long long ptr)
@ stdcall ReadConsoleOutputW(long ptr long long ptr)
@ stdcall ReadConsoleW(long ptr long ptr ptr)
@ stdcall ReadDirectoryChangesW(long ptr long long long ptr ptr ptr)
@ stdcall ReadFile(long ptr long ptr ptr)
@ stdcall ReadFileEx(long ptr long ptr ptr)
@ stdcall ReadFileScatter(long ptr long ptr ptr)
@ stdcall ReadProcessMemory(long ptr ptr long ptr)
@ stdcall -version=0x600+ RegisterApplicationRecoveryCallback(ptr ptr long long)
@ stdcall -version=0x600+ RegisterApplicationRestart(wstr long)
@ stdcall RegisterConsoleIME(ptr ptr)
@ stdcall RegisterConsoleOS2(long)
@ stdcall RegisterConsoleVDM(long long long long long long long long long long long)
@ stdcall RegisterWaitForInputIdle(ptr)
@ stdcall RegisterWaitForSingleObject(ptr long ptr ptr long long)
@ stdcall RegisterWaitForSingleObjectEx(long ptr ptr long long)
@ stdcall RegisterWowBaseHandlers(long)
@ stdcall RegisterWowExec(long)
@ stdcall ReleaseActCtx(ptr)
@ stdcall ReleaseMutex(long)
@ stub -version=0x600+ ReleaseMutexWhenCallbackReturns
@ stdcall -arch=arm64 ReleaseSRWLockExclusive(ptr) ntdll_vista.RtlReleaseSRWLockExclusive
@ stdcall -version=0x600+ ReleaseSRWLockShared(ptr) ntdll.RtlReleaseSRWLockShared
@ stdcall ReleaseSemaphore(long long ptr)
@ stub -version=0x600+ ReleaseSemaphoreWhenCallbackReturns
@ stdcall RemoveDirectoryA(str)
@ stub -version=0x600+ RemoveDirectoryTransactedA
@ stub -version=0x600+ RemoveDirectoryTransactedW
@ stdcall RemoveDirectoryW(wstr)
@ stdcall RemoveLocalAlternateComputerNameA(str long)
@ stdcall RemoveLocalAlternateComputerNameW(wstr long)
@ stub -version=0x600+ RemoveSecureMemoryCacheCallback
@ stdcall RemoveVectoredContinueHandler(ptr) ntdll.RtlRemoveVectoredContinueHandler
@ stdcall RemoveVectoredExceptionHandler(ptr) ntdll.RtlRemoveVectoredExceptionHandler
@ stdcall ReplaceFile(wstr wstr wstr long ptr ptr) ReplaceFileW
@ stdcall ReplaceFileA(str str str long ptr ptr)
@ stdcall ReplaceFileW(wstr wstr wstr long ptr ptr)
@ stub -version=0x600+ ReplacePartitionUnit
@ stdcall RequestDeviceWakeup(long)
@ stdcall RequestWakeupLatency(long)
@ stdcall ResetEvent(long)
@ stdcall ResetWriteWatch(ptr long)
@ stdcall RestoreLastError(long) ntdll.RtlRestoreLastWin32Error
@ stdcall ResumeThread(long)
@ stdcall -arch=x86_64,arm64 RtlAddFunctionTable(ptr long long) ntdll.RtlAddFunctionTable
@ stdcall -register RtlCaptureContext(ptr) ntdll.RtlCaptureContext
@ stdcall RtlCaptureStackBackTrace(long long ptr ptr) ntdll.RtlCaptureStackBackTrace
@ stdcall -arch=x86_64,arm64 RtlCompareMemory(ptr ptr ptr) ntdll.RtlCompareMemory
@ stdcall -arch=x86_64,arm64 RtlCopyMemory(ptr ptr ptr) ntdll.memcpy
@ stdcall -arch=x86_64,arm64 RtlDeleteFunctionTable(ptr) ntdll.RtlDeleteFunctionTable
@ stdcall RtlFillMemory(ptr long long) ntdll.RtlFillMemory
@ stdcall -arch=x86_64,arm64 RtlInstallFunctionTableCallback(double double long ptr ptr ptr) ntdll.RtlInstallFunctionTableCallback
@ stdcall -arch=x86_64,arm64 RtlLookupFunctionEntry(ptr ptr ptr) ntdll.RtlLookupFunctionEntry
@ stdcall RtlMoveMemory(ptr ptr long) ntdll.RtlMoveMemory
@ stdcall -arch=x86_64,arm64 RtlPcToFileHeader(ptr ptr) ntdll.RtlPcToFileHeader
@ stdcall -arch=x86_64,arm64 RtlRaiseException(ptr) ntdll.RtlRaiseException
@ stdcall -arch=x86_64,arm64 RtlRestoreContext(ptr ptr) ntdll.RtlRestoreContext
@ stdcall RtlUnwind(ptr ptr ptr ptr) ntdll.RtlUnwind
@ stdcall -arch=x86_64,arm64 RtlUnwindEx(ptr ptr ptr ptr ptr ptr) ntdll.RtlUnwindEx
@ stdcall -arch=x86_64,arm64 RtlVirtualUnwind(long int64 int64 ptr ptr ptr ptr ptr) ntdll.RtlVirtualUnwind
@ stdcall RtlZeroMemory(ptr long) ntdll.RtlZeroMemory
@ stdcall ScrollConsoleScreenBufferA(long ptr ptr ptr ptr)
@ stdcall ScrollConsoleScreenBufferW(long ptr ptr ptr ptr)
@ stdcall SearchPathA(str str str long ptr ptr)
@ stdcall SearchPathW(wstr wstr wstr long ptr ptr)
@ stdcall -version=0x500-0x502 SetCPGlobal(long)
@ stdcall SetCalendarInfoA(long long long str)
@ stdcall SetCalendarInfoW(long long long wstr)
@ stdcall SetClientTimeZoneInformation(ptr)
@ stdcall SetComPlusPackageInstallStatus(ptr)
@ stdcall SetCommBreak(long)
@ stdcall SetCommConfig(long ptr long)
@ stdcall SetCommMask(long ptr)
@ stdcall SetCommState(long ptr)
@ stdcall SetCommTimeouts(long ptr)
@ stdcall SetComputerNameA(str)
@ stdcall SetComputerNameExA(long str)
@ stdcall SetComputerNameExW(long wstr)
@ stdcall SetComputerNameW(wstr)
@ stdcall SetConsoleActiveScreenBuffer(long)
@ stdcall SetConsoleCP(long)
@ stdcall -version=0x351-0x502 SetConsoleCommandHistoryMode(long)
@ stdcall SetConsoleCtrlHandler(ptr long)
@ stdcall SetConsoleCursor(long long)
@ stdcall SetConsoleCursorInfo(long ptr)
@ stdcall SetConsoleCursorMode(long long long)
@ stdcall SetConsoleCursorPosition(long long)
@ stdcall SetConsoleDisplayMode(long long ptr)
@ stdcall SetConsoleFont(long long)
@ stdcall SetConsoleHardwareState(long long long)
@ stdcall -version=0x600+ SetConsoleHistoryInfo(ptr)
@ stdcall SetConsoleIcon(ptr)
@ stdcall SetConsoleInputExeNameA(ptr)
@ stdcall SetConsoleInputExeNameW(ptr)
@ stdcall SetConsoleKeyShortcuts(long long long long)
@ stdcall SetConsoleLocalEUDC(long long long long)
@ stdcall SetConsoleMaximumWindowSize(long long)
@ stdcall SetConsoleMenuClose(long)
@ stdcall SetConsoleMode(long long)
@ stdcall SetConsoleNlsMode(long long)
@ stdcall SetConsoleNumberOfCommandsA(long long)
@ stdcall SetConsoleNumberOfCommandsW(long long)
@ stdcall SetConsoleOS2OemFormat(long)
@ stdcall SetConsoleOutputCP(long)
@ stdcall SetConsolePalette(long long long)
@ stdcall -version=0x600+ SetConsoleScreenBufferInfoEx(ptr ptr)
@ stdcall SetConsoleScreenBufferSize(long long)
@ stdcall SetConsoleTextAttribute(long long)
@ stdcall SetConsoleTitleA(str)
@ stdcall SetConsoleTitleW(wstr)
@ stdcall SetConsoleWindowInfo(long long ptr)
@ stdcall SetCriticalSectionSpinCount(ptr long) ntdll.RtlSetCriticalSectionSpinCount
@ stub -version=0x600+ SetCurrentConsoleFontEx
@ stdcall SetCurrentDirectoryA(str)
@ stdcall SetCurrentDirectoryW(wstr)
@ stdcall SetDefaultCommConfigA(str ptr long)
@ stdcall SetDefaultCommConfigW(wstr ptr long)
@ stdcall SetDllDirectoryA(str)
@ stdcall SetDllDirectoryW(wstr)
@ stub -version=0x600+ SetDynamicTimeZoneInformation
@ stdcall SetEndOfFile(long)
@ stdcall SetEnvironmentStringsA(ptr)
@ stdcall SetEnvironmentStringsW(ptr)
@ stdcall SetEnvironmentVariableA(str str)
@ stdcall SetEnvironmentVariableW(wstr wstr)
@ stdcall SetErrorMode(long)
@ stdcall SetEvent(long)
@ stub -version=0x600+ SetEventWhenCallbackReturns
@ stdcall SetFileApisToANSI()
@ stdcall SetFileApisToOEM()
@ stdcall SetFileAttributesA(str long)
@ stub -version=0x600+ SetFileAttributesTransactedA
@ stub -version=0x600+ SetFileAttributesTransactedW
@ stdcall SetFileAttributesW(wstr long)
@ stdcall -version=0x600+ SetFileBandwidthReservation(ptr long long long ptr ptr)
@ stdcall SetFileCompletionNotificationModes(ptr long)
@ stub -arch=arm64 SetFileInformationByHandle
@ stub -version=0x600+ SetFileIoOverlappedRange
@ stdcall SetFilePointer(long long ptr long)
@ stdcall SetFilePointerEx(long double ptr long)
@ stdcall SetFileShortNameA(long str)
@ stdcall SetFileShortNameW(long wstr)
@ stdcall SetFileTime(long ptr ptr ptr)
@ stdcall SetFileValidData(long double)
@ stdcall SetFirmwareEnvironmentVariableA(str str ptr long)
@ stdcall -version=0x602+ SetFirmwareEnvironmentVariableExA(str str ptr long long)
@ stdcall -version=0x602+ SetFirmwareEnvironmentVariableExW(str str ptr long long)
@ stdcall SetFirmwareEnvironmentVariableW(wstr wstr ptr long)
@ stdcall -i386 SetHandleContext(long long)
@ stdcall SetHandleCount(long)
@ stdcall SetHandleInformation(long long long)
@ stdcall SetInformationJobObject(long long ptr long)
@ stdcall SetLastConsoleEventActive() ; missing in XP SP3
@ stdcall SetLastError(long) ntdll.RtlSetLastWin32Error
@ stdcall SetLocalPrimaryComputerNameA(long long) ; missing in XP SP3
@ stdcall SetLocalPrimaryComputerNameW(long long) ; missing in XP SP3
@ stdcall SetLocalTime(ptr)
@ stdcall SetLocaleInfoA(long long str)
@ stdcall SetLocaleInfoW(long long wstr)
@ stdcall SetMailslotInfo(long long)
@ stdcall SetMessageWaitingIndicator(ptr long)
@ stub -version=0x600+ SetNamedPipeAttribute
@ stdcall SetNamedPipeHandleState(long ptr ptr ptr)
@ stdcall SetPriorityClass(long long)
@ stdcall SetProcessAffinityMask(long long)
@ stub -version=0x600+ SetProcessAffinityUpdateMode
@ stdcall SetProcessDEPPolicy(long)
@ stdcall SetProcessPriorityBoost(long long)
@ stdcall SetProcessShutdownParameters(long long)
@ stdcall SetProcessWorkingSetSize(long long long)
@ stdcall SetProcessWorkingSetSizeEx(long long long long)
@ stdcall SetSearchPathMode(long)
@ stdcall SetStdHandle(long long)
@ stub -version=0x600+ SetStdHandleEx
@ stdcall SetSystemFileCacheSize(long long long)
@ stdcall SetSystemPowerState(long long)
@ stdcall SetSystemTime(ptr)
@ stdcall SetSystemTimeAdjustment(long long)
@ stdcall SetTapeParameters(ptr long ptr)
@ stdcall SetTapePosition(ptr long long long long long)
@ stdcall SetTermsrvAppInstallMode(long)
@ stdcall SetThreadAffinityMask(long long)
@ stdcall SetThreadContext(long ptr)
@ stdcall -version=0xA00+ SetThreadDescription(ptr wstr)
@ stdcall SetThreadErrorMode(long ptr)
@ stdcall SetThreadExecutionState(long)
@ stdcall SetThreadGroupAffinity(ptr ptr ptr)
@ stdcall SetThreadIdealProcessor(long long)
@ stdcall SetThreadLocale(long)
@ stdcall -version=0x600+ SetThreadPreferredUILanguages(long wstr ptr)
@ stdcall SetThreadPriority(long long)
@ stdcall SetThreadPriorityBoost(long long)
@ stdcall SetThreadStackGuarantee(ptr)
@ stdcall SetThreadUILanguage(long)
@ stdcall -version=0x600+ SetThreadpoolThreadMaximum(ptr long) ntdll.TpSetPoolMaxThreads
@ stdcall -version=0x600+ SetThreadpoolThreadMinimum(ptr long) ntdll.TpSetPoolMinThreads
@ stdcall -version=0x600+ SetThreadpoolTimer(ptr ptr long long) ntdll.TpSetTimer
@ stdcall -version=0x600+ SetThreadpoolWait(ptr long ptr) ntdll.TpSetWait
@ stdcall SetTimeZoneInformation(ptr)
@ stdcall SetTimerQueueTimer(long ptr ptr long long long)
@ stdcall SetUnhandledExceptionFilter(ptr)
@ stdcall SetUserGeoID(long)
@ stdcall SetVDMCurrentDirectories(long long)
@ stdcall SetVolumeLabelA(str str)
@ stdcall SetVolumeLabelW(wstr wstr)
@ stdcall SetVolumeMountPointA(str str)
@ stdcall SetVolumeMountPointW(wstr wstr)
@ stdcall SetWaitableTimer(long ptr long ptr ptr long)
@ stdcall SetupComm(long long long)
@ stdcall ShowConsoleCursor(long long)
@ stdcall SignalObjectAndWait(long long long long)
@ stdcall SizeofResource(long long)
@ stdcall Sleep(long)
@ stdcall -version=0x600+ SleepConditionVariableCS(ptr ptr long)
@ stdcall -version=0x600+ SleepConditionVariableSRW(ptr ptr long long)
@ stdcall SleepEx(long long)
@ stdcall -version=0x600+ StartThreadpoolIo(ptr) ntdll.TpStartAsyncIoOperation
@ stdcall -version=0x600+ SubmitThreadpoolWork(ptr) ntdll.TpPostWork
@ stdcall SuspendThread(long)
@ stdcall SwitchToFiber(ptr)
@ stdcall SwitchToThread()
@ stdcall SystemTimeToFileTime(ptr ptr)
@ stdcall SystemTimeToTzSpecificLocalTime(ptr ptr ptr)
@ stdcall TerminateJobObject(ptr long)
@ stdcall TerminateProcess(ptr long)
@ stdcall TerminateThread(ptr long)
@ stdcall TermsrvAppInstallMode()
@ stdcall Thread32First(long ptr)
@ stdcall Thread32Next(long ptr)
@ stdcall TlsAlloc()
@ stdcall TlsFree(long)
@ stdcall -norelay TlsGetValue(long)
@ stdcall -norelay TlsSetValue(long ptr)
@ stdcall Toolhelp32ReadProcessMemory(long ptr ptr long ptr)
@ stdcall TransactNamedPipe(long ptr long ptr long ptr ptr)
@ stdcall TransmitCommChar(long long)
@ stdcall -arch=arm64 TryAcquireSRWLockExclusive(ptr) ntdll.RtlTryAcquireSRWLockExclusive
@ stdcall -version=0x601+ TryAcquireSRWLockShared(ptr) ntdll.RtlTryAcquireSRWLockShared
@ stdcall TryEnterCriticalSection(ptr) ntdll.RtlTryEnterCriticalSection
@ stdcall -version=0x600+ TrySubmitThreadpoolCallback(ptr ptr ptr)
@ stdcall TzSpecificLocalTimeToSystemTime(ptr ptr ptr)
@ stdcall UTRegister(long str str str ptr ptr ptr)
@ stdcall UTUnRegister(long)
@ stdcall UnhandledExceptionFilter(ptr)
@ stdcall UnlockFile(long long long long long)
@ stdcall UnlockFileEx(long long long long ptr)
@ stdcall UnmapViewOfFile(ptr)
@ stub -version=0x600+ UnregisterApplicationRecoveryCallback
@ stub -version=0x600+ UnregisterApplicationRestart
@ stdcall UnregisterConsoleIME()
@ stdcall UnregisterWait(long)
@ stdcall UnregisterWaitEx(long long)
@ stub -version=0x600+ UpdateCalendarDayOfWeek
@ stdcall UpdateProcThreadAttribute(ptr long ptr ptr ptr ptr ptr)
@ stdcall UpdateResourceA(long str str long ptr long)
@ stdcall UpdateResourceW(long wstr wstr long ptr long)
@ stdcall VDMConsoleOperation(long long)
@ stdcall VDMOperationStarted(long)
@ stdcall -version=0x500-0x502 ValidateLCType(long long ptr ptr)
@ stdcall -version=0x500-0x502 ValidateLocale(long)
@ stdcall VerLanguageNameA(long str long)
@ stdcall VerLanguageNameW(long wstr long)
@ stdcall -ret64 VerSetConditionMask(long long long long) ntdll.VerSetConditionMask
@ stdcall VerifyConsoleIoHandle(long)
@ stub -version=0x600+ VerifyScripts
@ stdcall VerifyVersionInfoA(long long double)
@ stdcall VerifyVersionInfoW(long long double)
@ stdcall VirtualAlloc(ptr long long long)
@ stdcall VirtualAllocEx(long ptr long long long)
@ stub -version=0x600+ VirtualAllocExNuma
@ stdcall VirtualFree(ptr long long)
@ stdcall VirtualFreeEx(long ptr long long)
@ stdcall VirtualLock(ptr long)
@ stdcall VirtualProtect(ptr long long ptr)
@ stdcall VirtualProtectEx(long ptr long long ptr)
@ stdcall VirtualQuery(ptr ptr long)
@ stdcall VirtualQueryEx(long ptr ptr long)
@ stdcall VirtualUnlock(ptr long)
@ stdcall WTSGetActiveConsoleSessionId()
@ stdcall WaitCommEvent(long ptr ptr)
@ stdcall WaitForDebugEvent(ptr long)
@ stdcall WaitForMultipleObjects(long ptr long long)
@ stdcall WaitForMultipleObjectsEx(long ptr long long long)
@ stdcall WaitForSingleObject(long long)
@ stdcall WaitForSingleObjectEx(long long long)
@ stdcall -version=0x600+ WaitForThreadpoolIoCallbacks(ptr long) ntdll.TpWaitForIoCompletion
@ stdcall -version=0x600+ WaitForThreadpoolTimerCallbacks(ptr long) ntdll.TpWaitForTimer
@ stdcall -version=0x600+ WaitForThreadpoolWaitCallbacks(ptr long) ntdll.TpWaitForWait
@ stdcall -version=0x600+ WaitForThreadpoolWorkCallbacks(ptr long) ntdll.TpWaitForWork
@ stdcall WaitNamedPipeA(str long)
@ stdcall WaitNamedPipeW(wstr long)
@ stdcall -version=0x600+ WakeAllConditionVariable(ptr) ntdll.RtlWakeAllConditionVariable
@ stdcall -version=0x600+ WakeConditionVariable(ptr) ntdll.RtlWakeConditionVariable
@ stub -version=0x600+ WerGetFlags
@ stub -version=0x600+ WerRegisterFile
@ stub -version=0x600+ WerRegisterMemoryBlock
@ stub -version=0x600+ WerSetFlags
@ stub -version=0x600+ WerUnregisterFile
@ stub -version=0x600+ WerUnregisterMemoryBlock
@ stub -version=0x600+ WerpCleanupMessageMapping
@ stub -version=0x600+ WerpInitiateRemoteRecovery
@ stub -version=0x600+ WerpNotifyLoadStringResource
@ stub -version=0x600+ WerpNotifyLoadStringResourceEx
@ stub -version=0x600+ WerpNotifyUseStringResource
@ stub -version=0x600+ WerpStringLookup
@ stdcall WideCharToMultiByte(long long wstr long ptr long ptr ptr)
@ stdcall WinExec(str long)
@ stdcall Wow64DisableWow64FsRedirection(ptr)
@ stdcall Wow64EnableWow64FsRedirection(long)
@ stub -version=0x600+ Wow64GetThreadContext
@ stdcall Wow64RevertWow64FsRedirection(ptr)
@ stub -version=0x600+ Wow64SetThreadContext
@ stub -version=0x600+ Wow64SuspendThread
@ stdcall WriteConsoleA(long ptr long ptr ptr)
@ stdcall WriteConsoleInputA(long ptr long ptr)
@ stdcall WriteConsoleInputVDMA(long long long long)
@ stdcall WriteConsoleInputVDMW(long long long long)
@ stdcall WriteConsoleInputW(long ptr long ptr)
@ stdcall WriteConsoleOutputA(long ptr long long ptr)
@ stdcall WriteConsoleOutputAttribute(long ptr long long ptr)
@ stdcall WriteConsoleOutputCharacterA(long ptr long long ptr)
@ stdcall WriteConsoleOutputCharacterW(long ptr long long ptr)
@ stdcall WriteConsoleOutputW(long ptr long long ptr)
@ stdcall WriteConsoleW(long ptr long ptr ptr)
@ stdcall WriteFile(long ptr long ptr ptr)
@ stdcall WriteFileEx(long ptr long ptr ptr)
@ stdcall WriteFileGather(long ptr long ptr ptr)
@ stdcall WritePrivateProfileSectionA(str str str)
@ stdcall WritePrivateProfileSectionW(wstr wstr wstr)
@ stdcall WritePrivateProfileStringA(str str str str)
@ stdcall WritePrivateProfileStringW(wstr wstr wstr wstr)
@ stdcall WritePrivateProfileStructA(str str ptr long str)
@ stdcall WritePrivateProfileStructW(wstr wstr ptr long wstr)
@ stdcall WriteProcessMemory(long ptr ptr long ptr)
@ stdcall WriteProfileSectionA(str str)
@ stdcall WriteProfileSectionW(str str)
@ stdcall WriteProfileStringA(str str str)
@ stdcall WriteProfileStringW(wstr wstr wstr)
@ stdcall WriteTapemark(ptr long long long)
@ stdcall ZombifyActCtx(ptr)
@ stdcall -arch=x86_64,arm64 __C_specific_handler() ntdll.__C_specific_handler
@ stdcall -arch=x86_64,arm64 __chkstk() ntdll.__chkstk
;@ stdcall -arch=x86_64 __misaligned_access() ntdll.__misaligned_access
@ stdcall _hread(long ptr long)
@ stdcall _hwrite(long ptr long)
@ stdcall _lclose(long)
@ stdcall _lcreat(str long)
@ stdcall _llseek(long long long)
@ stdcall -arch=x86_64 _local_unwind() ntdll._local_unwind
@ stdcall _lopen(str long)
@ stdcall _lread(long ptr long) _hread
@ stdcall _lwrite(long ptr long) _hwrite
@ stdcall lstrcat(str str) lstrcatA
@ stdcall lstrcatA(str str)
@ stdcall lstrcatW(wstr wstr)
@ stdcall lstrcmp(str str) lstrcmpA
@ stdcall lstrcmpA(str str)
@ stdcall lstrcmpW(wstr wstr)
@ stdcall lstrcmpi(str str) lstrcmpiA
@ stdcall lstrcmpiA(str str)
@ stdcall lstrcmpiW(wstr wstr)
@ stdcall lstrcpy(ptr str) lstrcpyA
@ stdcall lstrcpyA(ptr str)
@ stdcall lstrcpyW(ptr wstr)
@ stdcall lstrcpyn(ptr str long) lstrcpynA
@ stdcall lstrcpynA(ptr str long)
@ stdcall lstrcpynW(ptr wstr long)
@ stdcall lstrlen(str) lstrlenA
@ stdcall lstrlenA(str)
@ stdcall lstrlenW(wstr)
;@ stdcall -arch=x86_64 uaw_lstrcmpW(wstr wstr)
;@ stdcall -arch=x86_64 uaw_lstrcmpiW(wstr wstr)
;@ stdcall -arch=x86_64 uaw_lstrlenW(wstr)
;@ stdcall -arch=x86_64 uaw_wcschr(wstr long)
;@ stdcall -arch=x86_64 uaw_wcscpy(ptr wstr)
;@ stdcall -arch=x86_64 uaw_wcsicmp(wstr wstr)
;@ stdcall -arch=x86_64 uaw_wcslen(wstr)
;@ stdcall -arch=x86_64 uaw_wcsrchr(wstr long)

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ ActivateActCtxWorker
@ stub -version=0xA00+ ActivatePackageVirtualizationContext
@ stub -version=0xA00+ AddDllDirectory # -> api-ms-win-core-libraryloader-l1-1-0.AddDllDirectory
@ stub -version=0xA00+ AddIntegrityLabelToBoundaryDescriptor
@ stub -version=0xA00+ AddRefActCtxWorker
@ stub -version=0xA00+ AddResourceAttributeAce
@ stub -version=0xA00+ AddScopedPolicyIDAce
@ stub -version=0xA00+ AllocConsoleWithOptions
@ stub -version=0xA00+ AppPolicyGetClrCompat # -> kernelbase.AppPolicyGetClrCompat
@ stub -version=0xA00+ AppPolicyGetCreateFileAccess # -> kernelbase.AppPolicyGetCreateFileAccess
@ stub -version=0xA00+ AppPolicyGetLifecycleManagement # -> kernelbase.AppPolicyGetLifecycleManagement
@ stub -version=0xA00+ AppPolicyGetMediaFoundationCodecLoading # -> kernelbase.AppPolicyGetMediaFoundationCodecLoading
@ stub -version=0xA00+ AppPolicyGetProcessTerminationMethod # -> kernelbase.AppPolicyGetProcessTerminationMethod
@ stub -version=0xA00+ AppPolicyGetShowDeveloperDiagnostic # -> kernelbase.AppPolicyGetShowDeveloperDiagnostic
@ stub -version=0xA00+ AppPolicyGetThreadInitializationType # -> kernelbase.AppPolicyGetThreadInitializationType
@ stub -version=0xA00+ AppPolicyGetWindowingModel # -> kernelbase.AppPolicyGetWindowingModel
@ stub -version=0xA00+ AppXGetOSMaxVersionTested # -> kernelbase.AppXGetOSMaxVersionTested
@ stub -version=0xA00+ AreShortNamesEnabled
@ stub -version=0xA00+ BackupReadEx
@ stub -version=0xA00+ BackupWriteEx
@ stub -version=0xA00+ BaseCheckAppcompatCacheEx
@ stub -version=0xA00+ BaseCheckAppcompatCacheExWorker
@ stub -version=0xA00+ BaseCheckAppcompatCacheWorker
@ stub -version=0xA00+ BaseCheckElevation
@ stub -version=0xA00+ BaseCleanupAppcompatCacheSupportWorker
@ stub -version=0xA00+ BaseDestroyVDMEnvironment
@ stub -version=0xA00+ BaseDllReadWriteIniFile
@ stub -version=0xA00+ BaseDumpAppcompatCacheWorker
@ stub -version=0xA00+ BaseElevationPostProcessing
@ stub -version=0xA00+ BaseFlushAppcompatCacheWorker
@ stub -version=0xA00+ BaseFormatObjectAttributes
@ stub -version=0xA00+ BaseFormatTimeOut
@ stub -version=0xA00+ BaseFreeAppCompatDataForProcessWorker
@ stub -version=0xA00+ BaseGetNamedObjectDirectory
@ stub -version=0xA00+ BaseInitAppcompatCacheSupportWorker
@ stub -version=0xA00+ BaseIsAppcompatInfrastructureDisabledWorker
@ stub -version=0xA00+ BaseIsDosApplication
@ stub -version=0xA00+ BaseMarkProcessTokenForInstallers
@ stub -version=0xA00+ BaseReadAppCompatDataForProcessWorker
@ stub -version=0xA00+ BaseSetLastNTError
@ stub -version=0xA00+ BaseUpdateAppcompatCacheWorker
@ stub -version=0xA00+ BaseUpdateVDMEntry
@ stub -version=0xA00+ BaseWriteErrorElevationRequiredEvent
@ stub -version=0xA00+ Basep8BitStringToDynamicUnicodeString
@ stub -version=0xA00+ BasepAllocateActivationContextActivationBlock
@ stub -version=0xA00+ BasepAnsiStringToDynamicUnicodeString
@ stub -version=0xA00+ BasepAppContainerEnvironmentExtension
@ stub -version=0xA00+ BasepAppXExtension
@ stub -version=0xA00+ BasepBrokerSandboxedCreateProcess
@ stub -version=0xA00+ BasepCheckAppCompat
@ stub -version=0xA00+ BasepCheckPplSupport
@ stub -version=0xA00+ BasepCheckWebBladeHashes
@ stub -version=0xA00+ BasepConstructSxsCreateProcessMessage
@ stub -version=0xA00+ BasepCopyEncryption
@ stub -version=0xA00+ BasepCreateProcessAsUserInsideSecurityEnvironment
@ stub -version=0xA00+ BasepFinishPackageActivation
@ stub -version=0xA00+ BasepFinishPackageActivationForSxS
@ stub -version=0xA00+ BasepFreeActivationContextActivationBlock
@ stub -version=0xA00+ BasepFreeActivationTokenInfo
@ stub -version=0xA00+ BasepGetAppCompatData
@ stub -version=0xA00+ BasepGetComputerNameFromNtPath
@ stub -version=0xA00+ BasepGetExeArchType
@ stub -version=0xA00+ BasepGetPackageActivationTokenForFilePath
@ stub -version=0xA00+ BasepGetPackageActivationTokenForFilePath2
@ stub -version=0xA00+ BasepGetPackageActivationTokenForSxS
@ stub -version=0xA00+ BasepGetPackageActivationTokenForSxS2
@ stub -version=0xA00+ BasepGetPackageActivationTokenForSxS3
@ stub -version=0xA00+ BasepGetPackagedAppInfoForFile
@ stub -version=0xA00+ BasepInitAppCompatData
@ stub -version=0xA00+ BasepIsProcessAllowed
@ stub -version=0xA00+ BasepMapModuleHandle
@ stub -version=0xA00+ BasepNotifyLoadStringResource
@ stub -version=0xA00+ BasepPostSuccessAppXExtension
@ stub -version=0xA00+ BasepProcessInvalidImage
@ stub -version=0xA00+ BasepQueryAppCompat
@ stub -version=0xA00+ BasepQueryModuleChpeSettings
@ stub -version=0xA00+ BasepReleaseAppXContext
@ stub -version=0xA00+ BasepReleasePackagedAppInfo
@ stub -version=0xA00+ BasepReleaseSxsCreateProcessUtilityStruct
@ stub -version=0xA00+ BasepReportFault
@ stub -version=0xA00+ BasepSandboxEngineCreateProcessHooks_IsSandboxedLaunchSignaled
@ stub -version=0xA00+ BasepSandboxEngineCreateProcessHooks_SetMsixPackageSignal
@ stub -version=0xA00+ BasepSetFileEncryptionCompression
@ stub -version=0xA00+ BuildIoRingCancelRequest # -> api-ms-win-core-ioring-l1-1-0.BuildIoRingCancelRequest
@ stub -version=0xA00+ BuildIoRingFlushFile # -> api-ms-win-core-ioring-l1-1-1.BuildIoRingFlushFile
@ stub -version=0xA00+ BuildIoRingReadFile # -> api-ms-win-core-ioring-l1-1-0.BuildIoRingReadFile
@ stub -version=0xA00+ BuildIoRingReadFileScatter # -> api-ms-win-core-ioring-l1-1-2.BuildIoRingReadFileScatter
@ stub -version=0xA00+ BuildIoRingRegisterBuffers # -> api-ms-win-core-ioring-l1-1-0.BuildIoRingRegisterBuffers
@ stub -version=0xA00+ BuildIoRingRegisterFileHandles # -> api-ms-win-core-ioring-l1-1-0.BuildIoRingRegisterFileHandles
@ stub -version=0xA00+ BuildIoRingWriteFile # -> api-ms-win-core-ioring-l1-1-1.BuildIoRingWriteFile
@ stub -version=0xA00+ BuildIoRingWriteFileGather # -> api-ms-win-core-ioring-l1-1-2.BuildIoRingWriteFileGather
@ stub -version=0xA00+ CeipIsOptedIn # -> kernelbase.CeipIsOptedIn
@ stub -version=0xA00+ CheckAllowDecryptedRemoteDestinationPolicy
@ stub -version=0xA00+ CheckForReadOnlyResourceFilter
@ stub -version=0xA00+ CheckTokenCapability
@ stub -version=0xA00+ CheckTokenMembershipEx
@ stub -version=0xA00+ CloseIoRing # -> api-ms-win-core-ioring-l1-1-0.CloseIoRing
@ stub -version=0xA00+ ClosePackageInfo # -> kernelbase.ClosePackageInfo
@ stub -version=0xA00+ ClosePseudoConsole
@ stub -version=0xA00+ CloseState # -> kernelbase.CloseState
@ stub -version=0xA00+ CopyContext
@ stub -version=0xA00+ CopyFile2
@ stub -version=0xA00+ CreateActCtxWWorker
@ stub -version=0xA00+ CreateDirectory2A
@ stub -version=0xA00+ CreateDirectory2W
@ stub -version=0xA00+ CreateEnclave # -> api-ms-win-core-enclave-l1-1-0.CreateEnclave
@ stub -version=0xA00+ CreateFile2
@ stub -version=0xA00+ CreateFile3
@ stub -version=0xA00+ CreateFileMappingFromApp # -> api-ms-win-core-memory-l1-1-1.CreateFileMappingFromApp
@ stub -version=0xA00+ CreateIoRing # -> api-ms-win-core-ioring-l1-1-0.CreateIoRing
@ stub -version=0xA00+ CreatePackageVirtualizationContext
@ stub -version=0xA00+ CreateProcessAsUserA
@ stub -version=0xA00+ CreateProcessAsUserW
@ stub -version=0xA00+ CreatePseudoConsole
@ stub -version=0xA00+ CreateRemoteThreadEx # -> api-ms-win-core-processthreads-l1-1-0.CreateRemoteThreadEx
@ stub -arch=win64 -version=0xA00+ CreateUmsCompletionList
@ stub -arch=win64 -version=0xA00+ CreateUmsThreadContext
@ stub -version=0xA00+ CtrlRoutine # -> kernelbase.CtrlRoutine
@ stub -version=0xA00+ DeactivateActCtxWorker
@ stub -version=0xA00+ DeactivatePackageVirtualizationContext
@ stub -version=0xA00+ DeleteFile2A
@ stub -version=0xA00+ DeleteFile2W
@ stub -version=0xA00+ DeleteSynchronizationBarrier
@ stub -arch=win64 -version=0xA00+ DeleteUmsCompletionList
@ stub -arch=win64 -version=0xA00+ DeleteUmsThreadContext
@ stub -arch=win64 -version=0xA00+ DequeueUmsCompletionListItems
@ stub -version=0xA00+ DisableThreadProfiling
@ stub -version=0xA00+ DiscardVirtualMemory # -> api-ms-win-core-memory-l1-1-2.DiscardVirtualMemory
@ stub -version=0xA00+ DnsHostnameToComputerNameExW
@ stub -version=0xA00+ DuplicateEncryptionInfoFileExt
@ stub -version=0xA00+ DuplicatePackageVirtualizationContext
@ stub -version=0xA00+ EnableProcessOptionalXStateFeatures
@ stub -version=0xA00+ EnableThreadProfiling
@ stub -version=0xA00+ EnterSynchronizationBarrier
@ stub -arch=win64 -version=0xA00+ EnterUmsSchedulingMode
@ stub -version=0xA00+ EnumSystemGeoNames
@ stub -arch=win64 -version=0xA00+ ExecuteUmsThread
@ stub -version=0xA00+ FindActCtxSectionGuidWorker
@ stub -version=0xA00+ FindActCtxSectionStringWWorker
@ stub -version=0xA00+ FindPackagesByPackageFamily # -> kernelbase.FindPackagesByPackageFamily
@ stub -version=0xA00+ FindStringOrdinal
@ stub -version=0xA00+ FlsGetValue2 # -> kernelbase.FlsGetValue2
@ stub -version=0xA00+ FormatApplicationUserModelId # -> kernelbase.FormatApplicationUserModelId
@ stub -version=0xA00+ FreeMemoryJobObject
@ stub -version=0xA00+ GetAppContainerAce
@ stub -version=0xA00+ GetAppContainerNamedObjectPath
@ stub -version=0xA00+ GetApplicationRecoveryCallbackWorker
@ stub -version=0xA00+ GetApplicationRestartSettingsWorker
@ stub -version=0xA00+ GetApplicationUserModelId # -> kernelbase.GetApplicationUserModelId
@ stub -version=0xA00+ GetCachedSigningLevel
@ stub -version=0xA00+ GetCurrentActCtxWorker
@ stub -version=0xA00+ GetCurrentApplicationUserModelId # -> kernelbase.GetCurrentApplicationUserModelId
@ stub -version=0xA00+ GetCurrentPackageFamilyName # -> kernelbase.GetCurrentPackageFamilyName
@ stub -version=0xA00+ GetCurrentPackageFullName # -> kernelbase.GetCurrentPackageFullName
@ stub -version=0xA00+ GetCurrentPackageInfo # -> kernelbase.GetCurrentPackageInfo
@ stub -version=0xA00+ GetCurrentPackagePath # -> kernelbase.GetCurrentPackagePath
@ stub -version=0xA00+ GetCurrentPackageVirtualizationContext
@ stub -version=0xA00+ GetCurrentThreadStackLimits # -> api-ms-win-core-processthreads-l1-1-0.GetCurrentThreadStackLimits
@ stub -arch=win64 -version=0xA00+ GetCurrentUmsThread
@ stub -version=0xA00+ GetDateFormatAWorker
@ stub -version=0xA00+ GetDateFormatWWorker
@ stub -version=0xA00+ GetDiskSpaceInformationA # -> api-ms-win-core-file-l1-2-3.GetDiskSpaceInformationA
@ stub -version=0xA00+ GetDiskSpaceInformationW # -> api-ms-win-core-file-l1-2-3.GetDiskSpaceInformationW
@ stub -version=0xA00+ GetEnabledXStateFeatures
@ stub -version=0xA00+ GetEncryptedFileVersionExt
@ stub -version=0xA00+ GetEraNameCountedString
@ stub -version=0xA00+ GetFileInformationByName
@ stub -version=0xA00+ GetGeoInfoEx
@ stub -version=0xA00+ GetIoRingInfo # -> api-ms-win-core-ioring-l1-1-0.GetIoRingInfo
@ stub -version=0xA00+ GetMachineTypeAttributes # -> api-ms-win-core-processthreads-l1-1-7.GetMachineTypeAttributes
@ stub -version=0xA00+ GetMaximumProcessorGroupCount
@ stub -version=0xA00+ GetMemoryErrorHandlingCapabilities
@ stub -arch=win64 -version=0xA00+ GetNextUmsListItem
@ stub -version=0xA00+ GetNumaAvailableMemoryNodeEx
@ stub -version=0xA00+ GetNumaNodeNumberFromHandle
@ stub -version=0xA00+ GetNumaNodeProcessorMask2 # -> api-ms-win-core-systemtopology-l1-1-2.GetNumaNodeProcessorMask2
@ stub -version=0xA00+ GetNumaProcessorNodeEx
@ stub -version=0xA00+ GetNumaProximityNodeEx
@ stub -version=0xA00+ GetOverlappedResultEx # -> api-ms-win-core-io-l1-1-1.GetOverlappedResultEx
@ stub -version=0xA00+ GetPackageApplicationIds # -> kernelbase.GetPackageApplicationIds
@ stub -version=0xA00+ GetPackageFamilyName # -> kernelbase.GetPackageFamilyName
@ stub -version=0xA00+ GetPackageFullName # -> kernelbase.GetPackageFullName
@ stub -version=0xA00+ GetPackageId # -> kernelbase.GetPackageId
@ stub -version=0xA00+ GetPackageInfo # -> kernelbase.GetPackageInfo
@ stub -version=0xA00+ GetPackagePath # -> kernelbase.GetPackagePath
@ stub -version=0xA00+ GetPackagePathByFullName # -> kernelbase.GetPackagePathByFullName
@ stub -version=0xA00+ GetPackagesByPackageFamily # -> kernelbase.GetPackagesByPackageFamily
@ stub -version=0xA00+ GetProcessDefaultCpuSetMasks # -> api-ms-win-core-processthreads-l1-1-6.GetProcessDefaultCpuSetMasks
@ stub -version=0xA00+ GetProcessDefaultCpuSets # -> api-ms-win-core-processthreads-l1-1-3.GetProcessDefaultCpuSets
@ stub -version=0xA00+ GetProcessInformation
@ stub -version=0xA00+ GetProcessMitigationPolicy # -> api-ms-win-core-processthreads-l1-1-1.GetProcessMitigationPolicy
@ stub -version=0xA00+ GetProcessPreferredUILanguages
@ stub -version=0xA00+ GetProcessesInVirtualizationContext
@ stub -version=0xA00+ GetProcessorSystemCycleTime # -> api-ms-win-core-sysinfo-l1-2-2.GetProcessorSystemCycleTime
@ stub -version=0xA00+ GetStagedPackagePathByFullName # -> kernelbase.GetStagedPackagePathByFullName
@ stub -version=0xA00+ GetStateFolder # -> kernelbase.GetStateFolder
@ stub -version=0xA00+ GetSystemAppDataKey # -> kernelbase.GetSystemAppDataKey
@ stub -version=0xA00+ GetSystemCpuSetInformation # -> api-ms-win-core-processthreads-l1-1-3.GetSystemCpuSetInformation
@ stub -version=0xA00+ GetTempPath2A
@ stub -version=0xA00+ GetTempPath2W
@ stub -version=0xA00+ GetThreadEnabledXStateFeatures
@ stub -version=0xA00+ GetThreadIdealProcessorEx
@ stub -version=0xA00+ GetThreadInformation
@ stub -version=0xA00+ GetThreadSelectedCpuSetMasks # -> api-ms-win-core-processthreads-l1-1-6.GetThreadSelectedCpuSetMasks
@ stub -version=0xA00+ GetThreadSelectedCpuSets # -> api-ms-win-core-processthreads-l1-1-3.GetThreadSelectedCpuSets
@ stub -version=0xA00+ GetTimeFormatAWorker
@ stub -version=0xA00+ GetTimeFormatWWorker
@ stub -arch=win64 -version=0xA00+ GetUmsCompletionListEvent
@ stub -arch=win64 -version=0xA00+ GetUmsSystemThreadInformation
@ stub -version=0xA00+ GetUserDefaultGeoName
@ stub -version=0xA00+ GetXStateFeaturesMask
@ stub -version=0xA00+ GlobalAddAtomExA
@ stub -version=0xA00+ GlobalAddAtomExW
@ stub -version=0xA00+ InitializeContext
@ stub -version=0xA00+ InitializeContext2
@ stub -version=0xA00+ InitializeEnclave # -> api-ms-win-core-enclave-l1-1-0.InitializeEnclave
@ stub -version=0xA00+ InitializeSynchronizationBarrier
@ stub -version=0xA00+ InstallELAMCertificateInfo # -> api-ms-win-core-sysinfo-l1-2-1.InstallELAMCertificateInfo
@ stub -version=0xA00+ InterlockedPushListSListEx # -> NTDLL.RtlInterlockedPushListSListEx
@ stub -version=0xA00+ IsEnclaveTypeSupported # -> api-ms-win-core-enclave-l1-1-0.IsEnclaveTypeSupported
@ stub -version=0xA00+ IsIoRingOpSupported # -> api-ms-win-core-ioring-l1-1-0.IsIoRingOpSupported
@ stub -version=0xA00+ IsNativeVhdBoot
@ stub -version=0xA00+ IsProcessCritical # -> api-ms-win-core-processthreads-l1-1-2.IsProcessCritical
@ stub -version=0xA00+ IsUserCetAvailableInEnvironment # -> api-ms-win-core-sysinfo-l1-2-6.IsUserCetAvailableInEnvironment
@ stub -version=0xA00+ IsValidNLSVersion
@ stub -version=0xA00+ IsWow64GuestMachineSupported # -> api-ms-win-core-wow64-l1-1-2.IsWow64GuestMachineSupported
@ stub -version=0xA00+ IsWow64Process2 # -> api-ms-win-core-wow64-l1-1-1.IsWow64Process2
@ stub -version=0xA00+ LoadAppInitDlls
@ stub -version=0xA00+ LoadEnclaveData # -> api-ms-win-core-enclave-l1-1-0.LoadEnclaveData
@ stub -version=0xA00+ LoadPackagedLibrary
@ stub -version=0xA00+ LocalFileTimeToLocalSystemTime # -> api-ms-win-core-timezone-l1-1-1.LocalFileTimeToLocalSystemTime
@ stub -version=0xA00+ LocalSystemTimeToLocalFileTime # -> api-ms-win-core-timezone-l1-1-1.LocalSystemTimeToLocalFileTime
@ stub -version=0xA00+ LocateXStateFeature
@ stub -version=0xA00+ LogUnexpectedCodepath # -> NTDLL.RtlLogUnexpectedCodepath
@ stub -version=0xA00+ MapViewOfFileFromApp # -> api-ms-win-core-memory-l1-1-1.MapViewOfFileFromApp
@ stub -version=0xA00+ NotifyMountMgr
@ stub -version=0xA00+ NtVdm64CreateProcessInternalW
@ stub -version=0xA00+ OOBEComplete
@ stub -version=0xA00+ OfferVirtualMemory # -> api-ms-win-core-memory-l1-1-2.OfferVirtualMemory
@ stub -version=0xA00+ OpenConsoleWStub
@ stub -version=0xA00+ OpenPackageInfoByFullName # -> kernelbase.OpenPackageInfoByFullName
@ stub -version=0xA00+ OpenProcessToken # -> api-ms-win-core-processthreads-l1-1-0.OpenProcessToken
@ stub -version=0xA00+ OpenState # -> kernelbase.OpenState
@ stub -version=0xA00+ OpenStateExplicit # -> kernelbase.OpenStateExplicit
@ stub -version=0xA00+ OpenThreadToken # -> api-ms-win-core-processthreads-l1-1-0.OpenThreadToken
@ stub -version=0xA00+ PackageFamilyNameFromFullName # -> kernelbase.PackageFamilyNameFromFullName
@ stub -version=0xA00+ PackageFamilyNameFromId # -> kernelbase.PackageFamilyNameFromId
@ stub -version=0xA00+ PackageFullNameFromId # -> kernelbase.PackageFullNameFromId
@ stub -version=0xA00+ PackageIdFromFullName # -> kernelbase.PackageIdFromFullName
@ stub -version=0xA00+ PackageNameAndPublisherIdFromFamilyName # -> kernelbase.PackageNameAndPublisherIdFromFamilyName
@ stub -version=0xA00+ ParseApplicationUserModelId # -> kernelbase.ParseApplicationUserModelId
@ stub -version=0xA00+ PopIoRingCompletion # -> api-ms-win-core-ioring-l1-1-0.PopIoRingCompletion
@ stub -version=0xA00+ PowerClearRequest
@ stub -version=0xA00+ PowerCreateRequest
@ stub -version=0xA00+ PowerSetRequest
@ stub -version=0xA00+ PrefetchVirtualMemory # -> api-ms-win-core-memory-l1-1-1.PrefetchVirtualMemory
@ stub -version=0xA00+ PssCaptureSnapshot
@ stub -version=0xA00+ PssDuplicateSnapshot
@ stub -version=0xA00+ PssFreeSnapshot
@ stub -version=0xA00+ PssQuerySnapshot
@ stub -version=0xA00+ PssWalkMarkerCreate
@ stub -version=0xA00+ PssWalkMarkerFree
@ stub -version=0xA00+ PssWalkMarkerGetPosition
@ stub -version=0xA00+ PssWalkMarkerRewind
@ stub -version=0xA00+ PssWalkMarkerSeek
@ stub -version=0xA00+ PssWalkMarkerSeekToBeginning
@ stub -version=0xA00+ PssWalkMarkerSetPosition
@ stub -version=0xA00+ PssWalkMarkerTell
@ stub -version=0xA00+ PssWalkSnapshot
@ stub -version=0xA00+ QueryActCtxSettingsWWorker
@ stub -version=0xA00+ QueryActCtxWWorker
@ stub -version=0xA00+ QueryIdleProcessorCycleTimeEx
@ stub -version=0xA00+ QueryIoRateControlInformationJobObject
@ stub -version=0xA00+ QueryIoRingCapabilities # -> api-ms-win-core-ioring-l1-1-0.QueryIoRingCapabilities
@ stub -version=0xA00+ QueryProtectedPolicy # -> api-ms-win-core-processthreads-l1-1-2.QueryProtectedPolicy
@ stub -version=0xA00+ QueryThreadProfiling
@ stub -version=0xA00+ QueryThreadpoolStackInformation
@ stub -arch=win64 -version=0xA00+ QueryUmsThreadInformation
@ stub -version=0xA00+ QueryUnbiasedInterruptTime
@ stub -version=0xA00+ QueueUserAPC2 # -> api-ms-win-core-processthreads-l1-1-5.QueueUserAPC2
@ stub -version=0xA00+ QuirkGetData2Worker
@ stub -version=0xA00+ QuirkGetDataWorker
@ stub -version=0xA00+ QuirkIsEnabled2Worker
@ stub -version=0xA00+ QuirkIsEnabled3Worker
@ stub -version=0xA00+ QuirkIsEnabledForPackage2Worker
@ stub -version=0xA00+ QuirkIsEnabledForPackage3Worker
@ stub -version=0xA00+ QuirkIsEnabledForPackage4Worker
@ stub -version=0xA00+ QuirkIsEnabledForPackageWorker
@ stub -version=0xA00+ QuirkIsEnabledForProcessWorker
@ stub -version=0xA00+ QuirkIsEnabledWorker
@ stub -version=0xA00+ RaiseInvalid16BitExeError
@ stub -version=0xA00+ ReadDirectoryChangesExW
@ stub -version=0xA00+ ReadThreadProfilingData
@ stub -version=0xA00+ ReclaimVirtualMemory # -> api-ms-win-core-memory-l1-1-2.ReclaimVirtualMemory
@ stub -version=0xA00+ RegCloseKey
@ stub -version=0xA00+ RegCopyTreeW
@ stub -version=0xA00+ RegCreateKeyExA
@ stub -version=0xA00+ RegCreateKeyExW
@ stub -version=0xA00+ RegDeleteKeyExA
@ stub -version=0xA00+ RegDeleteKeyExW
@ stub -version=0xA00+ RegDeleteTreeA
@ stub -version=0xA00+ RegDeleteTreeW
@ stub -version=0xA00+ RegDeleteValueA
@ stub -version=0xA00+ RegDeleteValueW
@ stub -version=0xA00+ RegDisablePredefinedCacheEx
@ stub -version=0xA00+ RegEnumKeyExA
@ stub -version=0xA00+ RegEnumKeyExW
@ stub -version=0xA00+ RegEnumValueA
@ stub -version=0xA00+ RegEnumValueW
@ stub -version=0xA00+ RegFlushKey
@ stub -version=0xA00+ RegGetKeySecurity
@ stub -version=0xA00+ RegGetValueA
@ stub -version=0xA00+ RegGetValueW
@ stub -version=0xA00+ RegLoadKeyA
@ stub -version=0xA00+ RegLoadKeyW
@ stub -version=0xA00+ RegLoadMUIStringA
@ stub -version=0xA00+ RegLoadMUIStringW
@ stub -version=0xA00+ RegNotifyChangeKeyValue
@ stub -version=0xA00+ RegOpenCurrentUser
@ stub -version=0xA00+ RegOpenKeyExA
@ stub -version=0xA00+ RegOpenKeyExW
@ stub -version=0xA00+ RegOpenUserClassesRoot
@ stub -version=0xA00+ RegQueryInfoKeyA
@ stub -version=0xA00+ RegQueryInfoKeyW
@ stub -version=0xA00+ RegQueryValueExA
@ stub -version=0xA00+ RegQueryValueExW
@ stub -version=0xA00+ RegRestoreKeyA
@ stub -version=0xA00+ RegRestoreKeyW
@ stub -version=0xA00+ RegSaveKeyExA
@ stub -version=0xA00+ RegSaveKeyExW
@ stub -version=0xA00+ RegSetKeySecurity
@ stub -version=0xA00+ RegSetValueExA
@ stub -version=0xA00+ RegSetValueExW
@ stub -version=0xA00+ RegUnLoadKeyA
@ stub -version=0xA00+ RegUnLoadKeyW
@ stub -version=0xA00+ RegisterBadMemoryNotification
@ stub -version=0xA00+ RegisterWaitUntilOOBECompleted
@ stub -version=0xA00+ ReleaseActCtxWorker
@ stub -version=0xA00+ ReleasePackageVirtualizationContext
@ stub -version=0xA00+ ReleasePseudoConsole
@ stub -version=0xA00+ RemoveDirectory2A
@ stub -version=0xA00+ RemoveDirectory2W
@ stub -version=0xA00+ RemoveDllDirectory # -> api-ms-win-core-libraryloader-l1-1-0.RemoveDllDirectory
@ stub -version=0xA00+ ResizePseudoConsole
@ stub -version=0xA00+ ResolveDelayLoadedAPI # -> NTDLL.LdrResolveDelayLoadedAPI
@ stub -version=0xA00+ ResolveDelayLoadsFromDll # -> NTDLL.LdrResolveDelayLoadsFromDll
@ stub -version=0xA00+ ResolveLocaleName
@ stub -arch=win64 -version=0xA00+ RtlIsEcCode # -> NTDLL.RtlIsEcCode
@ stub -arch=win64 -version=0xA00+ RtlVirtualUnwind2
@ stub -version=0xA00+ SetCachedSigningLevel
@ stub -version=0xA00+ SetComputerNameEx2W
@ stub -version=0xA00+ SetDefaultDllDirectories # -> api-ms-win-core-libraryloader-l1-1-0.SetDefaultDllDirectories
@ stub -version=0xA00+ SetIoRateControlInformationJobObject
@ stub -version=0xA00+ SetIoRingCompletionEvent # -> api-ms-win-core-ioring-l1-1-0.SetIoRingCompletionEvent
@ stub -version=0xA00+ SetProcessDefaultCpuSetMasks # -> api-ms-win-core-processthreads-l1-1-6.SetProcessDefaultCpuSetMasks
@ stub -version=0xA00+ SetProcessDefaultCpuSets # -> api-ms-win-core-processthreads-l1-1-3.SetProcessDefaultCpuSets
@ stub -version=0xA00+ SetProcessDynamicEHContinuationTargets # -> api-ms-win-core-processthreads-l1-1-4.SetProcessDynamicEHContinuationTargets
@ stub -version=0xA00+ SetProcessDynamicEnforcedCetCompatibleRanges # -> api-ms-win-core-processthreads-l1-1-6.SetProcessDynamicEnforcedCetCompatibleRanges
@ stub -version=0xA00+ SetProcessInformation
@ stub -version=0xA00+ SetProcessMitigationPolicy # -> api-ms-win-core-processthreads-l1-1-1.SetProcessMitigationPolicy
@ stub -version=0xA00+ SetProcessPreferredUILanguages
@ stub -version=0xA00+ SetProtectedPolicy # -> api-ms-win-core-processthreads-l1-1-2.SetProtectedPolicy
@ stub -version=0xA00+ SetThreadIdealProcessorEx
@ stub -version=0xA00+ SetThreadInformation
@ stub -version=0xA00+ SetThreadSelectedCpuSetMasks # -> api-ms-win-core-processthreads-l1-1-6.SetThreadSelectedCpuSetMasks
@ stub -version=0xA00+ SetThreadSelectedCpuSets # -> api-ms-win-core-processthreads-l1-1-3.SetThreadSelectedCpuSets
@ stub -version=0xA00+ SetThreadToken # -> api-ms-win-core-processthreads-l1-1-0.SetThreadToken
@ stub -version=0xA00+ SetThreadpoolStackInformation
@ stub -version=0xA00+ SetThreadpoolTimerEx # -> NTDLL.TpSetTimerEx
@ stub -version=0xA00+ SetThreadpoolWaitEx # -> NTDLL.TpSetWaitEx
@ stub -arch=win64 -version=0xA00+ SetUmsThreadInformation
@ stub -version=0xA00+ SetUserGeoName
@ stub -version=0xA00+ SetVolumeMountPointWStub
@ stub -version=0xA00+ SetWaitableTimerEx # -> api-ms-win-core-synch-l1-1-0.SetWaitableTimerEx
@ stub -version=0xA00+ SetXStateFeaturesMask
@ stub -version=0xA00+ SortCloseHandle
@ stub -version=0xA00+ SortGetHandle
@ stub -version=0xA00+ SubmitIoRing # -> api-ms-win-core-ioring-l1-1-0.SubmitIoRing
@ stub -version=0xA00+ SystemTimeToTzSpecificLocalTimeEx # -> api-ms-win-core-timezone-l1-1-0.SystemTimeToTzSpecificLocalTimeEx
@ stub -version=0xA00+ TermsrvConvertSysRootToUserDir
@ stub -version=0xA00+ TermsrvCreateRegEntry
@ stub -version=0xA00+ TermsrvDeleteKey
@ stub -version=0xA00+ TermsrvDeleteValue
@ stub -version=0xA00+ TermsrvGetPreSetValue
@ stub -version=0xA00+ TermsrvGetWindowsDirectoryA
@ stub -version=0xA00+ TermsrvGetWindowsDirectoryW
@ stub -version=0xA00+ TermsrvOpenRegEntry
@ stub -version=0xA00+ TermsrvOpenUserClasses
@ stub -version=0xA00+ TermsrvRestoreKey
@ stub -version=0xA00+ TermsrvSetKeySecurity
@ stub -version=0xA00+ TermsrvSetValueKey
@ stub -version=0xA00+ TermsrvSyncUserIniFileExt
@ stub -version=0xA00+ TlsGetValue2 # -> kernelbase.TlsGetValue2
@ stub -version=0xA00+ TzSpecificLocalTimeToSystemTimeEx # -> api-ms-win-core-timezone-l1-1-0.TzSpecificLocalTimeToSystemTimeEx
@ stub -arch=win64 -version=0xA00+ UmsThreadYield
@ stub -version=0xA00+ UnmapViewOfFileEx # -> api-ms-win-core-memory-l1-1-1.UnmapViewOfFileEx
@ stub -version=0xA00+ UnregisterBadMemoryNotification
@ stub -version=0xA00+ UnregisterWaitUntilOOBECompleted
@ stub -version=0xA00+ WaitForDebugEventEx # -> api-ms-win-core-debug-l1-1-2.WaitForDebugEventEx
@ stub -version=0xA00+ WerGetFlagsWorker
@ stub -version=0xA00+ WerRegisterAdditionalProcess
@ stub -version=0xA00+ WerRegisterAppLocalDump
@ stub -version=0xA00+ WerRegisterCustomMetadata
@ stub -version=0xA00+ WerRegisterExcludedMemoryBlock
@ stub -version=0xA00+ WerRegisterFileWorker
@ stub -version=0xA00+ WerRegisterMemoryBlockWorker
@ stub -version=0xA00+ WerRegisterRuntimeExceptionModule
@ stub -version=0xA00+ WerRegisterRuntimeExceptionModuleWorker
@ stub -version=0xA00+ WerSetFlagsWorker
@ stub -version=0xA00+ WerUnregisterAdditionalProcess
@ stub -version=0xA00+ WerUnregisterAppLocalDump
@ stub -version=0xA00+ WerUnregisterCustomMetadata
@ stub -version=0xA00+ WerUnregisterExcludedMemoryBlock
@ stub -version=0xA00+ WerUnregisterFileWorker
@ stub -version=0xA00+ WerUnregisterMemoryBlockWorker
@ stub -version=0xA00+ WerUnregisterRuntimeExceptionModule
@ stub -version=0xA00+ WerUnregisterRuntimeExceptionModuleWorker
@ stub -version=0xA00+ WerpGetDebugger
@ stub -version=0xA00+ WerpLaunchAeDebug
@ stub -version=0xA00+ WerpNotifyLoadStringResourceWorker
@ stub -version=0xA00+ WerpNotifyUseStringResourceWorker
@ stub -version=0xA00+ Wow64GetThreadSelectorEntry
@ stub -version=0xA00+ ZombifyActCtxWorker
@ stub -arch=win64 -version=0xA00+ __misaligned_access # -> NTDLL.__misaligned_access
@ stub -version=0xA00+ timeBeginPeriod
@ stub -version=0xA00+ timeEndPeriod
@ stub -version=0xA00+ timeGetDevCaps
@ stub -version=0xA00+ timeGetSystemTime
@ stub -version=0xA00+ timeGetTime
@ stub -arch=win64 -version=0xA00+ uaw_lstrcmpW
@ stub -arch=win64 -version=0xA00+ uaw_lstrcmpiW
@ stub -arch=win64 -version=0xA00+ uaw_lstrlenW
@ stub -arch=win64 -version=0xA00+ uaw_wcschr
@ stub -arch=win64 -version=0xA00+ uaw_wcscpy
@ stub -arch=win64 -version=0xA00+ uaw_wcsicmp
@ stub -arch=win64 -version=0xA00+ uaw_wcslen
@ stub -arch=win64 -version=0xA00+ uaw_wcsrchr
# Data exports (need an `extern`, not a code stub):
#   Wow64Transition
# END win10-surface stubs

# BEGIN vista forwards
# Generated by tools/win10_surface.py forwards; do not edit by hand. Exports Windows'
# kernel32.dll has and this build would not, forwarded to the implementation in kernel32_vista.
# Each switches off where this spec's own row for the name takes over.
@ stdcall -arch=i386,x86_64 AcquireSRWLockExclusive(ptr) kernel32_vista.AcquireSRWLockExclusive
@ stdcall -version=0x400-0x5ff AcquireSRWLockShared(ptr) kernel32_vista.AcquireSRWLockShared
@ stdcall -version=0x400-0x5ff ApplicationRecoveryFinished(long) kernel32_vista.ApplicationRecoveryFinished
@ stdcall -version=0x400-0x5ff ApplicationRecoveryInProgress(ptr) kernel32_vista.ApplicationRecoveryInProgress
@ stdcall -version=0x400-0x5ff CancelThreadpoolIo(ptr) kernel32_vista.CancelThreadpoolIo
@ stdcall -version=0x400-0x5ff CloseThreadpool(ptr) kernel32_vista.CloseThreadpool
@ stdcall -version=0x400-0x5ff CloseThreadpoolCleanupGroup(ptr) kernel32_vista.CloseThreadpoolCleanupGroup
@ stdcall -version=0x400-0x5ff CloseThreadpoolCleanupGroupMembers(ptr long ptr) kernel32_vista.CloseThreadpoolCleanupGroupMembers
@ stdcall -version=0x400-0x5ff CloseThreadpoolIo(ptr) kernel32_vista.CloseThreadpoolIo
@ stdcall -version=0x400-0x5ff CloseThreadpoolTimer(ptr) kernel32_vista.CloseThreadpoolTimer
@ stdcall -version=0x400-0x5ff CloseThreadpoolWait(ptr) kernel32_vista.CloseThreadpoolWait
@ stdcall -version=0x400-0x5ff CloseThreadpoolWork(ptr) kernel32_vista.CloseThreadpoolWork
@ stdcall -version=0x400-0x5ff CompareStringOrdinal(wstr long wstr long long) kernel32_vista.CompareStringOrdinal
@ stdcall -version=0x400-0x5ff CreateSymbolicLinkA(str str long) kernel32_vista.CreateSymbolicLinkA
@ stdcall -arch=i386,x86_64 CreateSymbolicLinkW(wstr wstr long) kernel32_vista.CreateSymbolicLinkW
@ stdcall -version=0x400-0x5ff CreateThreadpool(ptr) kernel32_vista.CreateThreadpool
@ stdcall -version=0x400-0x5ff CreateThreadpoolCleanupGroup() kernel32_vista.CreateThreadpoolCleanupGroup
@ stdcall -version=0x400-0x5ff CreateThreadpoolIo(ptr ptr ptr ptr) kernel32_vista.CreateThreadpoolIo
@ stdcall -version=0x400-0x5ff CreateThreadpoolTimer(ptr ptr ptr) kernel32_vista.CreateThreadpoolTimer
@ stdcall -version=0x400-0x5ff CreateThreadpoolWait(ptr ptr ptr) kernel32_vista.CreateThreadpoolWait
@ stdcall -version=0x400-0x5ff CreateThreadpoolWork(ptr ptr ptr) kernel32_vista.CreateThreadpoolWork
@ stdcall -version=0x400-0x5ff EnumSystemLocalesEx(ptr long long ptr) kernel32_vista.EnumSystemLocalesEx
@ stdcall -version=0x400-0x5ff FreeLibraryWhenCallbackReturns(ptr ptr) kernel32_vista.FreeLibraryWhenCallbackReturns
@ stdcall -version=0x400-0x5ff GetApplicationRecoveryCallback(ptr ptr ptr ptr ptr) kernel32_vista.GetApplicationRecoveryCallback
@ stdcall -version=0x400-0x5ff GetFileBandwidthReservation(ptr ptr ptr ptr ptr ptr) kernel32_vista.GetFileBandwidthReservation
@ stdcall -arch=i386,x86_64 GetFileInformationByHandleEx(ptr long ptr long) kernel32_vista.GetFileInformationByHandleEx
@ stdcall -version=0x400-0x5ff GetFileMUIInfo(long wstr ptr ptr) kernel32_vista.GetFileMUIInfo
@ stdcall -version=0x400-0x5ff GetFileMUIPath(long wstr wstr ptr wstr ptr ptr) kernel32_vista.GetFileMUIPath
@ stdcall -version=0x400-0x5ff GetFinalPathNameByHandleA(ptr str long long) kernel32_vista.GetFinalPathNameByHandleA
@ stdcall -arch=i386,x86_64 GetFinalPathNameByHandleW(ptr wstr long long) kernel32_vista.GetFinalPathNameByHandleW
@ stdcall -version=0x400-0x601 GetFirmwareEnvironmentVariableExA(str str ptr long long) kernel32_vista.GetFirmwareEnvironmentVariableExA
@ stdcall -version=0x400-0x601 GetFirmwareEnvironmentVariableExW(wstr wstr ptr long long) kernel32_vista.GetFirmwareEnvironmentVariableExW
@ stdcall -version=0x400-0x601 GetFirmwareType(ptr) kernel32_vista.GetFirmwareType
@ stdcall -version=0x400-0x5ff GetLocaleInfoEx(wstr long ptr long) kernel32_vista.GetLocaleInfoEx
@ stdcall -version=0x400-0x5ff GetNamedPipeClientProcessId(ptr ptr) kernel32_vista.GetNamedPipeClientProcessId
@ stdcall -version=0x400-0x5ff GetSystemDefaultLocaleName(ptr long) kernel32_vista.GetSystemDefaultLocaleName
@ stdcall -version=0x400-0x5ff GetSystemPreferredUILanguages(long ptr wstr ptr) kernel32_vista.GetSystemPreferredUILanguages
@ stdcall -version=0x400-0x601 GetSystemTimePreciseAsFileTime(ptr) kernel32_vista.GetSystemTimePreciseAsFileTime
@ stdcall GetThreadDescription(ptr ptr) kernel32_vista.GetThreadDescription
@ stdcall -version=0x400-0x5ff GetThreadPreferredUILanguages(long ptr wstr ptr) kernel32_vista.GetThreadPreferredUILanguages
@ stdcall -version=0x400-0x5ff GetThreadUILanguage() kernel32_vista.GetThreadUILanguage
@ stdcall -version=0x400-0x5ff -ret64 GetTickCount64() kernel32_vista.GetTickCount64
@ stdcall -version=0x400-0x5ff GetUILanguageInfo(long wstr wstr ptr ptr) kernel32_vista.GetUILanguageInfo
@ stdcall -version=0x400-0x5ff GetUserDefaultLocaleName(wstr long) kernel32_vista.GetUserDefaultLocaleName
@ stdcall -version=0x400-0x5ff GetUserPreferredUILanguages(long ptr wstr ptr) kernel32_vista.GetUserPreferredUILanguages
@ stdcall -version=0x400-0x5ff InitOnceBeginInitialize(ptr long ptr ptr) kernel32_vista.InitOnceBeginInitialize
@ stdcall -version=0x400-0x5ff InitOnceComplete(ptr long ptr) kernel32_vista.InitOnceComplete
@ stdcall -version=0x400-0x5ff InitOnceExecuteOnce(ptr ptr ptr ptr) kernel32_vista.InitOnceExecuteOnce
@ stdcall -version=0x400-0x5ff InitOnceInitialize(ptr) kernel32_vista.InitOnceInitialize
@ stdcall -version=0x400-0x5ff InitializeConditionVariable(ptr) kernel32_vista.InitializeConditionVariable
@ stdcall -version=0x400-0x5ff InitializeCriticalSectionEx(ptr long long) kernel32_vista.InitializeCriticalSectionEx
@ stdcall -version=0x400-0x5ff InitializeSRWLock(ptr) kernel32_vista.InitializeSRWLock
@ stdcall -version=0x400-0x5ff IsValidLocaleName(wstr) kernel32_vista.IsValidLocaleName
@ stdcall -version=0x400-0x5ff LCIDToLocaleName(long wstr long long) kernel32_vista.LCIDToLocaleName
@ stdcall -version=0x400-0x5ff LocaleNameToLCID(wstr long) kernel32_vista.LocaleNameToLCID
@ stdcall -version=0x400-0x5ff OpenFileById(ptr ptr long long ptr long) kernel32_vista.OpenFileById
@ stdcall -version=0x400-0x5ff QueryFullProcessImageNameA(ptr long ptr ptr) kernel32_vista.QueryFullProcessImageNameA
@ stdcall -version=0x400-0x5ff QueryFullProcessImageNameW(ptr long ptr ptr) kernel32_vista.QueryFullProcessImageNameW
@ stdcall -version=0x400-0x5ff RegisterApplicationRecoveryCallback(ptr ptr long long) kernel32_vista.RegisterApplicationRecoveryCallback
@ stdcall -version=0x400-0x5ff RegisterApplicationRestart(wstr long) kernel32_vista.RegisterApplicationRestart
@ stdcall -arch=i386,x86_64 ReleaseSRWLockExclusive(ptr) kernel32_vista.ReleaseSRWLockExclusive
@ stdcall -version=0x400-0x5ff ReleaseSRWLockShared(ptr) kernel32_vista.ReleaseSRWLockShared
@ stdcall -version=0x400-0x5ff SetFileBandwidthReservation(ptr long long long ptr ptr) kernel32_vista.SetFileBandwidthReservation
@ stdcall -version=0x400-0x601 SetFirmwareEnvironmentVariableExA(str str ptr long long) kernel32_vista.SetFirmwareEnvironmentVariableExA
@ stdcall -version=0x400-0x601 SetFirmwareEnvironmentVariableExW(str str ptr long long) kernel32_vista.SetFirmwareEnvironmentVariableExW
@ stdcall -version=0x400-0x9ff SetThreadDescription(ptr wstr) kernel32_vista.SetThreadDescription
@ stdcall -version=0x400-0x5ff SetThreadPreferredUILanguages(long wstr ptr) kernel32_vista.SetThreadPreferredUILanguages
@ stdcall -version=0x400-0x5ff SetThreadpoolThreadMaximum(ptr long) kernel32_vista.SetThreadpoolThreadMaximum
@ stdcall -version=0x400-0x5ff SetThreadpoolThreadMinimum(ptr long) kernel32_vista.SetThreadpoolThreadMinimum
@ stdcall -version=0x400-0x5ff SetThreadpoolTimer(ptr ptr long long) kernel32_vista.SetThreadpoolTimer
@ stdcall -version=0x400-0x5ff SetThreadpoolWait(ptr long ptr) kernel32_vista.SetThreadpoolWait
@ stdcall -version=0x400-0x5ff SleepConditionVariableCS(ptr ptr long) kernel32_vista.SleepConditionVariableCS
@ stdcall -version=0x400-0x5ff SleepConditionVariableSRW(ptr ptr long long) kernel32_vista.SleepConditionVariableSRW
@ stdcall -version=0x400-0x5ff StartThreadpoolIo(ptr) kernel32_vista.StartThreadpoolIo
@ stdcall -version=0x400-0x5ff SubmitThreadpoolWork(ptr) kernel32_vista.SubmitThreadpoolWork
@ stdcall -arch=i386,x86_64 TryAcquireSRWLockExclusive(ptr) kernel32_vista.TryAcquireSRWLockExclusive
@ stdcall -version=0x400-0x600 TryAcquireSRWLockShared(ptr) kernel32_vista.TryAcquireSRWLockShared
@ stdcall -version=0x400-0x5ff TrySubmitThreadpoolCallback(ptr ptr ptr) kernel32_vista.TrySubmitThreadpoolCallback
@ stdcall -version=0x400-0x5ff WaitForThreadpoolIoCallbacks(ptr long) kernel32_vista.WaitForThreadpoolIoCallbacks
@ stdcall -version=0x400-0x5ff WaitForThreadpoolTimerCallbacks(ptr long) kernel32_vista.WaitForThreadpoolTimerCallbacks
@ stdcall -version=0x400-0x5ff WaitForThreadpoolWaitCallbacks(ptr long) kernel32_vista.WaitForThreadpoolWaitCallbacks
@ stdcall -version=0x400-0x5ff WaitForThreadpoolWorkCallbacks(ptr long) kernel32_vista.WaitForThreadpoolWorkCallbacks
@ stdcall -version=0x400-0x5ff WakeAllConditionVariable(ptr) kernel32_vista.WakeAllConditionVariable
@ stdcall -version=0x400-0x5ff WakeConditionVariable(ptr) kernel32_vista.WakeConditionVariable
# END vista forwards
