@ stdcall BRUSHOBJ_hGetColorTransform(ptr)
@ stdcall BRUSHOBJ_pvAllocRbrush(ptr long)
@ stdcall BRUSHOBJ_pvGetRbrush(ptr)
@ stdcall BRUSHOBJ_ulGetBrushColor(ptr)
@ stdcall CLIPOBJ_bEnum(ptr long ptr)
@ stdcall CLIPOBJ_cEnumStart(ptr long long long long)
@ stdcall CLIPOBJ_ppoGetPath(ptr)
@ stdcall EngAcquireSemaphore(ptr)
@ stdcall EngAllocMem(long long long)
@ stdcall EngAllocPrivateUserMem(ptr ptr long)
@ stdcall EngAllocSectionMem(ptr long ptr long)
@ stdcall EngAllocUserMem(ptr long)
@ stdcall EngAlphaBlend(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall EngAssociateSurface(ptr ptr long)
@ stdcall EngBitBlt(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall EngBugCheckEx(long ptr ptr ptr ptr) NTOSKRNL.KeBugCheckEx
@ stdcall EngCheckAbort(ptr)
@ stdcall EngClearEvent(ptr)
@ stdcall EngComputeGlyphSet(long long long)
@ stdcall EngControlSprites(ptr long)
@ stdcall EngCopyBits(ptr ptr ptr ptr ptr ptr)
@ stdcall EngCreateBitmap(long long long long long ptr)
@ stdcall EngCreateClip()
@ stdcall EngCreateDeviceBitmap(ptr long long long)
@ stdcall EngCreateDeviceSurface(ptr long long long)
@ stdcall EngCreateDriverObj(ptr ptr ptr)
@ stdcall EngCreateEvent(ptr)
@ stdcall EngCreatePalette(long long long long long long)
@ stdcall EngCreatePath()
@ stdcall EngCreateSemaphore()
@ stdcall EngCreateWnd(ptr ptr ptr long long)
@ stdcall EngDebugBreak() NTOSKRNL.DbgBreakPoint
@ stdcall EngDebugPrint(str str ptr)
@ stdcall EngDeleteClip(ptr)
@ stdcall EngDeleteDriverObj(ptr long long)
@ stdcall EngDeleteEvent(ptr)
@ stdcall EngDeleteFile(ptr)
@ stdcall EngDeletePalette(ptr)
@ stdcall EngDeletePath(ptr)
@ stdcall EngDeleteSafeSemaphore(ptr)
@ stdcall EngDeleteSemaphore(ptr)
@ stdcall EngDeleteSurface(ptr)
@ stdcall EngDeleteWnd(ptr)
@ stdcall EngDeviceIoControl(ptr long ptr long ptr long ptr)
@ stdcall EngDitherColor(ptr long long long)
@ stdcall EngDxIoctl(long ptr long)
@ stdcall EngEnumForms(ptr long ptr long ptr ptr)
@ stdcall EngEraseSurface(ptr ptr long)
@ stdcall EngFileIoControl(ptr long ptr ptr ptr ptr ptr)
@ stdcall EngFileWrite(ptr ptr ptr ptr)
@ stdcall EngFillPath(ptr ptr ptr ptr ptr long long)
@ stdcall EngFindImageProcAddress(ptr ptr)
@ stdcall EngFindResource(ptr long long ptr)
@ stdcall EngFntCacheAlloc(long long)
@ stdcall EngFntCacheFault(long long)
@ stdcall EngFntCacheLookUp(long ptr)
@ stdcall EngFreeMem(ptr)
@ stdcall EngFreeModule(ptr)
@ stdcall EngFreePrivateUserMem(ptr ptr)
@ stdcall EngFreeSectionMem(ptr ptr)
@ stdcall EngFreeUserMem(ptr)
@ stdcall EngGetCurrentCodePage(ptr ptr)
@ stdcall EngGetCurrentProcessId() NTOSKRNL.PsGetCurrentProcessId
@ stdcall EngGetCurrentThreadId() NTOSKRNL.PsGetCurrentThreadId
@ stdcall EngGetDriverName(ptr)
@ stdcall EngGetFileChangeTime(ptr ptr)
@ stdcall EngGetFilePath(ptr ptr)
@ stdcall EngGetForm(ptr ptr long ptr long ptr)
@ stdcall EngGetLastError()
@ stdcall EngGetPrinter(ptr long ptr long ptr)
@ stdcall EngGetPrinterData(ptr ptr ptr ptr long ptr)
@ stdcall EngGetPrinterDataFileName(ptr)
@ stdcall EngGetPrinterDriver(ptr ptr long ptr long ptr)
@ stdcall EngGetProcessHandle()
@ stdcall EngGetTickCount()
@ stdcall EngGetType1FontList(ptr ptr long ptr ptr ptr)
@ stdcall EngGradientFill(ptr ptr ptr ptr long ptr long ptr ptr long)
@ stdcall EngHangNotification(ptr ptr)
@ stdcall EngInitializeSafeSemaphore(ptr)
@ stdcall EngIsSemaphoreOwned(ptr)
@ stdcall EngIsSemaphoreOwnedByCurrentThread(ptr)
@ stdcall EngLineTo(ptr ptr ptr long long long long ptr long)
@ stdcall EngLoadImage(ptr)
@ stdcall EngLoadModule(ptr)
@ stdcall EngLoadModuleForWrite(ptr long)
@ stdcall EngLockDirectDrawSurface(ptr)
@ stdcall EngLockDriverObj(ptr)
@ stdcall EngLockSurface(ptr)
@ stdcall EngLpkInstalled()
@ stdcall EngMapEvent(ptr ptr ptr ptr ptr)
@ stdcall EngMapFile(ptr long ptr)
@ stdcall EngMapFontFile(ptr ptr ptr)
@ stdcall EngMapFontFileFD(ptr ptr ptr)
@ stdcall EngMapModule(ptr ptr)
@ stdcall EngMapSection(ptr long ptr ptr)
@ stdcall EngMarkBandingSurface(ptr)
@ stdcall EngModifySurface(ptr ptr long long ptr ptr long ptr)
@ stdcall EngMovePointer(ptr long long ptr)
@ stdcall EngMulDiv(long long long)
@ stdcall EngMultiByteToUnicodeN(ptr long ptr ptr long) NTOSKRNL.RtlMultiByteToUnicodeN
@ stdcall EngMultiByteToWideChar(long ptr long ptr long)
@ stdcall EngNineGrid(ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall EngPaint(ptr ptr ptr ptr long)
@ stdcall EngPlgBlt(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall EngProbeForRead(ptr long long) NTOSKRNL.ProbeForRead
@ stdcall EngProbeForReadAndWrite(ptr long long) NTOSKRNL.ProbeForWrite
@ stdcall EngQueryDeviceAttribute(ptr long ptr long ptr long)
@ stdcall EngQueryFileTimeStamp(ptr)
@ stdcall EngQueryLocalTime(ptr)
@ stdcall EngQueryPalette(ptr ptr long ptr)
@ stdcall EngQueryPerformanceCounter(ptr)
@ stdcall EngQueryPerformanceFrequency(ptr)
@ stdcall EngQuerySystemAttribute(long ptr)
@ stdcall EngReadStateEvent(ptr)
@ stdcall EngReleaseSemaphore(ptr)
@ stdcall EngRestoreFloatingPointState(ptr)
@ stdcall EngSaveFloatingPointState(ptr long)
@ stdcall EngSecureMem(ptr long)
@ stdcall EngSetEvent(ptr)
@ stdcall EngSetLastError(long)
@ stdcall EngSetPointerShape(ptr ptr ptr ptr long long long long ptr long)
@ stdcall EngSetPointerTag(ptr ptr ptr ptr long)
@ stdcall EngSetPrinterData(ptr ptr long ptr long)
@ stdcall EngSort(ptr long long ptr)
@ stdcall EngStretchBlt(ptr ptr ptr ptr ptr long long ptr ptr ptr long)
@ stdcall EngStretchBltROP(ptr ptr ptr ptr ptr long long ptr ptr ptr long ptr long)
@ stdcall EngStrokeAndFillPath(ptr ptr ptr ptr ptr ptr ptr ptr long long)
@ stdcall EngStrokePath(ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall EngTextOut(ptr ptr ptr ptr ptr ptr ptr ptr ptr long)
@ stdcall EngTransparentBlt(ptr ptr ptr ptr ptr ptr long long)
@ stdcall EngUnicodeToMultiByteN(ptr long ptr ptr long) NTOSKRNL.RtlUnicodeToMultiByteN
@ stdcall EngUnloadImage(ptr)
@ stdcall EngUnlockDirectDrawSurface(ptr)
@ stdcall EngUnlockDriverObj(ptr)
@ stdcall EngUnlockSurface(ptr)
@ stdcall EngUnmapEvent(ptr)
@ stdcall EngUnmapFile(ptr)
@ stdcall EngUnmapFontFile(ptr)
@ stdcall EngUnmapFontFileFD(ptr)
@ stdcall EngUnsecureMem(ptr)
@ stdcall EngWaitForSingleObject(ptr ptr)
@ stdcall EngWideCharToMultiByte(long ptr long ptr long)
@ stdcall EngWritePrinter(ptr ptr long ptr)
@ stdcall -arch=i386 FLOATOBJ_Add(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_AddFloat(ptr long)
@ stdcall -arch=i386 FLOATOBJ_AddFloatObj(ptr ptr) FLOATOBJ_Add
@ stdcall -arch=i386 FLOATOBJ_AddLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_Div(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_DivFloat(ptr long)
@ stdcall -arch=i386 FLOATOBJ_DivFloatObj(ptr ptr) FLOATOBJ_Div
@ stdcall -arch=i386 FLOATOBJ_DivLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_Equal(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_EqualLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_GetFloat(ptr)
@ stdcall -arch=i386 FLOATOBJ_GetLong(ptr)
@ stdcall -arch=i386 FLOATOBJ_GreaterThan(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_GreaterThanLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_LessThan(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_LessThanLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_Mul(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_MulFloat(ptr long)
@ stdcall -arch=i386 FLOATOBJ_MulFloatObj(ptr ptr) FLOATOBJ_Mul
@ stdcall -arch=i386 FLOATOBJ_MulLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_Neg(ptr)
@ stdcall -arch=i386 FLOATOBJ_SetFloat(ptr long)
@ stdcall -arch=i386 FLOATOBJ_SetLong(ptr long)
@ stdcall -arch=i386 FLOATOBJ_Sub(ptr ptr)
@ stdcall -arch=i386 FLOATOBJ_SubFloat(ptr long)
@ stdcall -arch=i386 FLOATOBJ_SubFloatObj(ptr ptr) FLOATOBJ_Sub
@ stdcall -arch=i386 FLOATOBJ_SubLong(ptr long)
@ stdcall FONTOBJ_cGetAllGlyphHandles(ptr ptr)
@ stdcall FONTOBJ_cGetGlyphs(ptr long long ptr ptr)
@ stdcall FONTOBJ_pQueryGlyphAttrs(ptr long)
@ stdcall FONTOBJ_pfdg(ptr)
@ stdcall FONTOBJ_pifi(ptr)
@ stdcall FONTOBJ_pjOpenTypeTablePointer(ptr long ptr)
@ stdcall FONTOBJ_pvTrueTypeFontFile(ptr ptr)
@ stdcall FONTOBJ_pwszFontFilePaths(ptr ptr)
@ stdcall FONTOBJ_pxoGetXform(ptr)
@ stdcall FONTOBJ_vGetInfo(ptr long ptr)
@ stdcall HT_ComputeRGBGammaTable(long long long long long ptr)
@ stdcall HT_Get8BPPFormatPalette(ptr long long long)
@ stdcall HT_Get8BPPMaskPalette(ptr long long long long long)
@ stdcall HeapVidMemAllocAligned(ptr long long ptr ptr)
@ stdcall PALOBJ_cGetColors(ptr long long ptr)
@ stdcall PATHOBJ_bCloseFigure(ptr)
@ stdcall PATHOBJ_bEnum(ptr ptr)
@ stdcall PATHOBJ_bEnumClipLines(ptr long ptr)
@ stdcall PATHOBJ_bMoveTo(ptr long long)
@ stdcall PATHOBJ_bPolyBezierTo(ptr ptr long)
@ stdcall PATHOBJ_bPolyLineTo(ptr ptr long)
@ stdcall PATHOBJ_vEnumStart(ptr)
@ stdcall PATHOBJ_vEnumStartClipLines(ptr ptr ptr ptr)
@ stdcall PATHOBJ_vGetBounds(ptr ptr)
@ stdcall RtlAnsiCharToUnicodeChar(ptr) NTOSKRNL.RtlAnsiCharToUnicodeChar
@ stdcall -arch=x86_64,arm RtlCaptureContext(ptr) NTOSKRNL.RtlCaptureContext
@ stdcall -arch=x86_64,arm RtlCopyMemory(ptr ptr int64) NTOSKRNL.RtlCopyMemory
@ stdcall -arch=x86_64,arm RtlCopyMemoryNonTemporal(ptr ptr int64) NTOSKRNL.RtlCopyMemoryNonTemporal
@ stdcall -arch=x86_64,arm RtlFillMemory(ptr long long) NTOSKRNL.RtlFillMemory
@ cdecl -arch=x86_64,arm RtlLookupFunctionEntry(double ptr ptr) NTOSKRNL.RtlLookupFunctionEntry
@ stdcall -arch=x86_64,arm RtlMoveMemory(ptr ptr long) NTOSKRNL.RtlMoveMemory
@ stdcall RtlMultiByteToUnicodeN(ptr long ptr ptr long) NTOSKRNL.RtlMultiByteToUnicodeN
@ cdecl -arch=x86_64,arm RtlPcToFileHeader(ptr ptr) NTOSKRNL.RtlPcToFileHeader
@ stdcall RtlRaiseException(ptr) NTOSKRNL.RtlRaiseException
@ cdecl -arch=x86_64,arm RtlRestoreContext(ptr ptr) NTOSKRNL.RtlRestoreContext
@ stdcall RtlUnicodeToMultiByteN(ptr long ptr ptr long) NTOSKRNL.RtlUnicodeToMultiByteN
@ stdcall RtlUnicodeToMultiByteSize(ptr ptr long) NTOSKRNL.RtlUnicodeToMultiByteSize
@ stdcall RtlUnwind(ptr ptr ptr ptr) NTOSKRNL.RtlUnwind
@ stdcall -arch=x86_64,arm RtlUnwindEx(ptr ptr ptr ptr ptr ptr) NTOSKRNL.RtlUnwindEx
@ stdcall RtlUpcaseUnicodeChar(long) NTOSKRNL.RtlUpcaseUnicodeChar
@ stdcall RtlUpcaseUnicodeToMultiByteN(ptr long ptr ptr long) NTOSKRNL.RtlUpcaseUnicodeToMultiByteN
@ stdcall -arch=x86_64,arm RtlVirtualUnwind(long int64 int64 ptr ptr ptr ptr ptr) NTOSKRNL.RtlVirtualUnwind
@ stdcall -arch=x86_64,arm RtlZeroMemory(ptr long) NTOSKRNL.RtlZeroMemory
@ stdcall STROBJ_bEnum(ptr ptr ptr)
@ stdcall STROBJ_bEnumPositionsOnly(ptr ptr ptr)
@ stdcall STROBJ_bGetAdvanceWidths(ptr long long ptr)
@ stdcall STROBJ_dwGetCodePage(ptr)
@ stdcall STROBJ_fxBreakExtra(ptr)
@ stdcall STROBJ_fxCharacterExtra(ptr)
@ stdcall STROBJ_vEnumStart(ptr)
@ stdcall VidMemFree(ptr ptr)
@ stdcall WNDOBJ_bEnum(ptr long ptr)
@ stdcall WNDOBJ_cEnumStart(ptr long long long)
@ stdcall WNDOBJ_vSetConsumer(ptr ptr)
@ stdcall XFORMOBJ_bApplyXform(ptr long long ptr ptr) EXFORMOBJ_bApplyXform
@ stdcall XFORMOBJ_iGetFloatObjXform(ptr ptr) EXFORMOBJ_iGetFloatObjXform
@ stdcall XFORMOBJ_iGetXform(ptr ptr) EXFORMOBJ_iGetXform
@ stdcall XLATEOBJ_cGetPalette(ptr long long ptr)
@ stdcall XLATEOBJ_hGetColorTransform(ptr)
@ stdcall XLATEOBJ_iXlate(ptr long)
@ stdcall XLATEOBJ_piVector(ptr)
@ cdecl -arch=x86_64,arm __C_specific_handler(ptr long ptr ptr) NTOSKRNL.__C_specific_handler
@ cdecl -arch=x86_64,arm __chkstk(ptr long ptr ptr) NTOSKRNL.__chkstk
@ cdecl -arch=arm __jump_unwind() NTOSKRNL.__jump_unwind
@ cdecl -arch=i386 _abnormal_termination() NTOSKRNL._abnormal_termination
@ cdecl -arch=i386 _except_handler2() NTOSKRNL._except_handler2
@ cdecl -arch=i386 _global_unwind2() NTOSKRNL._global_unwind2
@ cdecl _itoa() NTOSKRNL._itoa
@ cdecl _itow() NTOSKRNL._itow
@ cdecl -arch=i386 _local_unwind2() NTOSKRNL._local_unwind2
@ cdecl -arch=x86_64 _local_unwind() NTOSKRNL._local_unwind
@ cdecl -arch=x86_64,arm _setjmp(ptr ptr) NTOSKRNL._setjmp
@ cdecl -arch=x86_64,arm _setjmpex(ptr ptr) NTOSKRNL._setjmpex
@ cdecl -arch=x86_64,arm longjmp(ptr long) NTOSKRNL.longjmp
@ cdecl -arch=x86_64,arm memcmp() NTOSKRNL.memcmp
@ cdecl -arch=x86_64,arm memcpy() NTOSKRNL.memcpy
@ cdecl -arch=x86_64,arm memmove() NTOSKRNL.memmove
@ cdecl -arch=x86_64,arm memset() NTOSKRNL.memset

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ CddEngCombineRgn
@ stub -version=0xA00+ CddEngCopyRgn
@ stub -version=0xA00+ CddEngCreateRectRgn
@ stub -version=0xA00+ CddEngDeleteRgn
@ stub -version=0xA00+ CddEngEqualRgn
@ stub -version=0xA00+ CddEngGetRgnBox
@ stub -version=0xA00+ CddEngGetRgnData
@ stub -version=0xA00+ CddEngOffsetRgn
@ stub -version=0xA00+ CddEngRectInRgn
@ stub -version=0xA00+ CddEngSetRectRgn
@ stub -version=0xA00+ DxDdCleanupDxGraphics
@ stub -version=0xA00+ DxDdGetDxgkWin32kInterface
@ stub -version=0xA00+ DxDdIsTearDownLddmSpriteDisabled
@ stub -version=0xA00+ DxDdProcessCallout
@ stub -version=0xA00+ DxDdThreadCallout
@ stub -version=0xA00+ DxGetGpuUsageStatistics
@ stub -version=0xA00+ DxGetProcessInterferenceCount
@ stub -version=0xA00+ DxNotifyProcessFreezeCallout
@ stub -version=0xA00+ DxNotifyProcessThawCallout
@ stub -version=0xA00+ EngAcquireFastMutex
@ stub -version=0xA00+ EngAcquireSemaphoreNoWait
@ stub -version=0xA00+ EngAcquireSemaphoreShared
@ stub -version=0xA00+ EngAcquireSemaphoreSharedNoWait
@ stub -version=0xA00+ EngCTGetCurrentGamma
@ stub -version=0xA00+ EngCTGetGammaTable
@ stub -version=0xA00+ EngCreateFastMutex
@ stub -version=0xA00+ EngCreateRedirectionDeviceBitmap
@ stub -version=0xA00+ EngDeleteFastMutex
@ stub -version=0xA00+ EngIsCddDeviceBitmap
@ stub -version=0xA00+ EngIsSemaphoreSharedByCurrentThread
@ stub -version=0xA00+ EngQueryW32kCddInterface
@ stub -version=0xA00+ EngReleaseFastMutex
@ stub -version=0xA00+ EngUpdateDeviceSurface
@ stub -version=0xA00+ InitializeGraphicsInfrastructure
@ stub -version=0xA00+ IsSysEntryApiExtSupported
@ stub -version=0xA00+ IsWin32kJobNotifyCallOutPresent
@ stub -version=0xA00+ KfGetJobIdentity
@ stub -version=0xA00+ KfGetJobProvidingUILimit
@ stub -version=0xA00+ KfGetProcessAntiTamperIdentity
@ stub -version=0xA00+ KfGetProcessAntiTamperUIPolicy
@ stub -version=0xA00+ KfGetProcessEffectiveUILimits
@ stub -version=0xA00+ KfIsProcessDeepFrozen
@ stub -version=0xA00+ KfReportLearningModeViolation
@ stub -version=0xA00+ KfSetJobUILimits
@ stub -version=0xA00+ NtUserInitialize
@ stub -version=0xA00+ SysEntryGetDispatchTableValues
@ stub -version=0xA00+ SysEntryGetDispatchTableValuesFilter
@ stub -version=0xA00+ SysEntryGetW32pArgumentTable
@ stub -version=0xA00+ SysEntryGetW32pArgumentTableFilter
@ stub -version=0xA00+ SysEntryGetW32pServiceLimit
@ stub -version=0xA00+ SysEntryGetW32pServiceLimitFilter
@ stub -version=0xA00+ SysEntryGetW32pServiceTable
@ stub -version=0xA00+ SysEntryGetW32pServiceTableFilter
@ stub -version=0xA00+ SysEntryPsEstablishWin32Callouts
@ stub -version=0xA00+ SysEntryPsUnEstablishWin32Callouts
@ stub -version=0xA00+ SysEntryRegisterWin32kBaseDriverUnload
@ stub -version=0xA00+ SysEntrySMgrNotifySessionChange
@ stub -version=0xA00+ SysEntryWin32kBlackBoxLogFailure
@ stub -version=0xA00+ W32GetDCompSessionState
@ stub -version=0xA00+ W32GetDebugState
@ stub -version=0xA00+ W32GetHeapSessionState
@ stub -version=0xA00+ W32GetInputMonitorSessionState
@ stub -version=0xA00+ W32GetMaxSessionCount
@ stub -version=0xA00+ W32GetReferencedSessionProcessWithTag
@ stub -version=0xA00+ W32GetSessionState
@ stub -version=0xA00+ W32GetSiloSessionState
@ stub -version=0xA00+ W32GetUserCoreMessagingSessionState
@ stub -version=0xA00+ W32GetUserGdiSessionState
@ stub -version=0xA00+ W32GetUserInputMgrSessionState
@ stub -version=0xA00+ W32GetUserSessionState
@ stub -version=0xA00+ W32GetWin32kApiSetTable
@ stub -version=0xA00+ W32GetWin32kBaseApiSetTable
@ stub -version=0xA00+ W32GetWin32kFullApiSetTable
@ stub -version=0xA00+ W32GetWin32kSysSessionState
@ stub -version=0xA00+ W32PageEntireDriver
@ stub -version=0xA00+ W32RegisterSessionProcess
@ stub -version=0xA00+ W32UnregisterSessionProcess
@ stub -version=0xA00+ Win32kAsyncProcessFreezeThawSupportIsActive
@ stub -version=0xA00+ Win32kGetSupportedExportsVersion
@ stub -version=0xA00+ Win32kJobAddProcessNotify
@ stub -version=0xA00+ Win32kJobTerminateNotify
@ stub -version=0xA00+ Win32kJobUpdateUIRestrictionsNotify
@ stub -version=0xA00+ Win32kKernelExportsSet
@ stub -version=0xA00+ Win32kProcessFreezeChangeNotify
@ stub -version=0xA00+ Win32kSystemTableAdd
@ stub -version=0xA00+ __win32kstub_NtBindCompositionSurface
@ stub -version=0xA00+ __win32kstub_NtCloseCompositionInputSink
@ stub -version=0xA00+ __win32kstub_NtCompositionSetDropTarget
@ stub -version=0xA00+ __win32kstub_NtCompositorNotifyExitWindows
@ stub -version=0xA00+ __win32kstub_NtConfigureInputSpace
@ stub -version=0xA00+ __win32kstub_NtConfirmCompositionSurfaceIndependentFlipEntry
@ stub -version=0xA00+ __win32kstub_NtCreateCompositionInputSink
@ stub -version=0xA00+ __win32kstub_NtCreateCompositionSurfaceHandle
@ stub -version=0xA00+ __win32kstub_NtCreateImplicitCompositionInputSink
@ stub -version=0xA00+ __win32kstub_NtDCompositionAddCrossDeviceVisualChild
@ stub -version=0xA00+ __win32kstub_NtDCompositionBeginFrame
@ stub -version=0xA00+ __win32kstub_NtDCompositionBoostCompositorClock
@ stub -version=0xA00+ __win32kstub_NtDCompositionCommitChannel
@ stub -version=0xA00+ __win32kstub_NtDCompositionCommitSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtDCompositionConfirmFrame
@ stub -version=0xA00+ __win32kstub_NtDCompositionConnectPipe
@ stub -version=0xA00+ __win32kstub_NtDCompositionCreateAndBindSharedSection
@ stub -version=0xA00+ __win32kstub_NtDCompositionCreateBufferCollection
@ stub -version=0xA00+ __win32kstub_NtDCompositionCreateChannel
@ stub -version=0xA00+ __win32kstub_NtDCompositionCreateConnection
@ stub -version=0xA00+ __win32kstub_NtDCompositionCreateSharedResourceHandle
@ stub -version=0xA00+ __win32kstub_NtDCompositionCreateSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtDCompositionDestroyChannel
@ stub -version=0xA00+ __win32kstub_NtDCompositionDestroyConnection
@ stub -version=0xA00+ __win32kstub_NtDCompositionDuplicateHandleToProcess
@ stub -version=0xA00+ __win32kstub_NtDCompositionDuplicateSwapchainHandleToDwm
@ stub -version=0xA00+ __win32kstub_NtDCompositionEnableMMCSS
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetBatchId
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetChannels
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetConnectionBatch
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetDeletedResources
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetFrameId
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetFrameIdFromBatchId
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetFrameLegacyTokens
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetFrameStatistics
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetFrameSurfaceUpdates
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetStatistics
@ stub -version=0xA00+ __win32kstub_NtDCompositionGetTargetStatistics
@ stub -version=0xA00+ __win32kstub_NtDCompositionNotifySuperWetInkWork
@ stub -version=0xA00+ __win32kstub_NtDCompositionProcessChannelBatchBuffer
@ stub -version=0xA00+ __win32kstub_NtDCompositionRegisterThumbnailVisual
@ stub -version=0xA00+ __win32kstub_NtDCompositionRegisterVirtualDesktopVisual
@ stub -version=0xA00+ __win32kstub_NtDCompositionReleaseAllResources
@ stub -version=0xA00+ __win32kstub_NtDCompositionRemoveCrossDeviceVisualChild
@ stub -version=0xA00+ __win32kstub_NtDCompositionSendDwmLpcMessage
@ stub -version=0xA00+ __win32kstub_NtDCompositionSetBlurredWallpaperSurface
@ stub -version=0xA00+ __win32kstub_NtDCompositionSetChannelCommitCompletionEvent
@ stub -version=0xA00+ __win32kstub_NtDCompositionSetChannelConnectionId
@ stub -version=0xA00+ __win32kstub_NtDCompositionSetChildRootVisual
@ stub -version=0xA00+ __win32kstub_NtDCompositionSetDebugCounter
@ stub -version=0xA00+ __win32kstub_NtDCompositionSuspendAnimations
@ stub -version=0xA00+ __win32kstub_NtDCompositionSyncWait
@ stub -version=0xA00+ __win32kstub_NtDCompositionSynchronize
@ stub -version=0xA00+ __win32kstub_NtDCompositionTelemetrySetApplicationId
@ stub -version=0xA00+ __win32kstub_NtDCompositionUpdatePointerCapture
@ stub -version=0xA00+ __win32kstub_NtDCompositionWaitForChannel
@ stub -version=0xA00+ __win32kstub_NtDCompositionWaitForCompositorClock
@ stub -version=0xA00+ __win32kstub_NtDesktopCaptureBits
@ stub -version=0xA00+ __win32kstub_NtDuplicateCompositionInputSink
@ stub -version=0xA00+ __win32kstub_NtDxgkAcquireExclusiveEngineAccess
@ stub -version=0xA00+ __win32kstub_NtDxgkCancelPresents
@ stub -version=0xA00+ __win32kstub_NtDxgkCheckEngineAccess
@ stub -version=0xA00+ __win32kstub_NtDxgkCheckSinglePlaneForMultiPlaneOverlaySupport
@ stub -version=0xA00+ __win32kstub_NtDxgkConnectDoorbell
@ stub -version=0xA00+ __win32kstub_NtDxgkCreateDoorbell
@ stub -version=0xA00+ __win32kstub_NtDxgkCreateNativeFence
@ stub -version=0xA00+ __win32kstub_NtDxgkCreateTrackedWorkload
@ stub -version=0xA00+ __win32kstub_NtDxgkDDisplayEnum2
@ stub -version=0xA00+ __win32kstub_NtDxgkDestroyDoorbell
@ stub -version=0xA00+ __win32kstub_NtDxgkDestroyTrackedWorkload
@ stub -version=0xA00+ __win32kstub_NtDxgkDisableProcessDebugBlobCollection
@ stub -version=0xA00+ __win32kstub_NtDxgkDispMgrOperation
@ stub -version=0xA00+ __win32kstub_NtDxgkDisplayMuxSwitchExecute
@ stub -version=0xA00+ __win32kstub_NtDxgkDisplayMuxSwitchFinish
@ stub -version=0xA00+ __win32kstub_NtDxgkDisplayMuxSwitchPrepare
@ stub -version=0xA00+ __win32kstub_NtDxgkDisplayPortOperation
@ stub -version=0xA00+ __win32kstub_NtDxgkDuplicateHandle
@ stub -version=0xA00+ __win32kstub_NtDxgkEnableProcessDebugBlobCollection
@ stub -version=0xA00+ __win32kstub_NtDxgkEnumAdapters3
@ stub -version=0xA00+ __win32kstub_NtDxgkEnumProcesses
@ stub -version=0xA00+ __win32kstub_NtDxgkGetAvailableTrackedWorkloadIndex
@ stub -version=0xA00+ __win32kstub_NtDxgkGetNativeFenceLogDetail
@ stub -version=0xA00+ __win32kstub_NtDxgkGetProcessList
@ stub -version=0xA00+ __win32kstub_NtDxgkGetProperties
@ stub -version=0xA00+ __win32kstub_NtDxgkGetTrackedWorkloadStatistics
@ stub -version=0xA00+ __win32kstub_NtDxgkIsFeatureEnabled
@ stub -version=0xA00+ __win32kstub_NtDxgkMapProcessDebugBlob
@ stub -version=0xA00+ __win32kstub_NtDxgkNotifyWorkSubmission
@ stub -version=0xA00+ __win32kstub_NtDxgkOpenNativeFenceFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtDxgkOutputDuplPresentToHwQueue
@ stub -version=0xA00+ __win32kstub_NtDxgkPinResources
@ stub -version=0xA00+ __win32kstub_NtDxgkRegisterVailProcess
@ stub -version=0xA00+ __win32kstub_NtDxgkResetTrackedWorkloadStatistics
@ stub -version=0xA00+ __win32kstub_NtDxgkSetProperties
@ stub -version=0xA00+ __win32kstub_NtDxgkSubmitPresentBltToHwQueue
@ stub -version=0xA00+ __win32kstub_NtDxgkSubmitPresentToHwQueue
@ stub -version=0xA00+ __win32kstub_NtDxgkUnmapProcessDebugBlob
@ stub -version=0xA00+ __win32kstub_NtDxgkUnpinResources
@ stub -version=0xA00+ __win32kstub_NtDxgkUpdateTrackedWorkload
@ stub -version=0xA00+ __win32kstub_NtDxgkVailConnect
@ stub -version=0xA00+ __win32kstub_NtDxgkVailDisconnect
@ stub -version=0xA00+ __win32kstub_NtDxgkVailPromoteCompositionSurface
@ stub -version=0xA00+ __win32kstub_NtEnableOneCoreTransformMode
@ stub -version=0xA00+ __win32kstub_NtFlipObjectAddContent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectAddPoolBuffer
@ stub -version=0xA00+ __win32kstub_NtFlipObjectConsumerAcquirePresent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectConsumerAdjustUsageReference
@ stub -version=0xA00+ __win32kstub_NtFlipObjectConsumerBeginProcessPresent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectConsumerEndProcessPresent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectConsumerPostMessage
@ stub -version=0xA00+ __win32kstub_NtFlipObjectConsumerQueryBufferInfo
@ stub -version=0xA00+ __win32kstub_NtFlipObjectCreate
@ stub -version=0xA00+ __win32kstub_NtFlipObjectDisconnectEndpoint
@ stub -version=0xA00+ __win32kstub_NtFlipObjectEnablePresentStatisticsType
@ stub -version=0xA00+ __win32kstub_NtFlipObjectOpen
@ stub -version=0xA00+ __win32kstub_NtFlipObjectPresentCancel
@ stub -version=0xA00+ __win32kstub_NtFlipObjectQueryBufferAvailableEvent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectQueryEndpointConnected
@ stub -version=0xA00+ __win32kstub_NtFlipObjectQueryLostEvent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectQueryNextMessageToProducer
@ stub -version=0xA00+ __win32kstub_NtFlipObjectReadNextMessageToProducer
@ stub -version=0xA00+ __win32kstub_NtFlipObjectRemoveContent
@ stub -version=0xA00+ __win32kstub_NtFlipObjectRemovePoolBuffer
@ stub -version=0xA00+ __win32kstub_NtFlipObjectSetContent
@ stub -version=0xA00+ __win32kstub_NtGdiAbortDoc
@ stub -version=0xA00+ __win32kstub_NtGdiAbortPath
@ stub -version=0xA00+ __win32kstub_NtGdiAddEmbFontToDC
@ stub -version=0xA00+ __win32kstub_NtGdiAddFontMemResourceEx
@ stub -version=0xA00+ __win32kstub_NtGdiAddFontResourceW
@ stub -version=0xA00+ __win32kstub_NtGdiAddInitialFonts
@ stub -version=0xA00+ __win32kstub_NtGdiAddRemoteFontToDC
@ stub -version=0xA00+ __win32kstub_NtGdiAddRemoteMMInstanceToDC
@ stub -version=0xA00+ __win32kstub_NtGdiAlphaBlend
@ stub -version=0xA00+ __win32kstub_NtGdiAngleArc
@ stub -version=0xA00+ __win32kstub_NtGdiAnyLinkedFonts
@ stub -version=0xA00+ __win32kstub_NtGdiArcInternal
@ stub -version=0xA00+ __win32kstub_NtGdiBRUSHOBJ_DeleteRbrush
@ stub -version=0xA00+ __win32kstub_NtGdiBRUSHOBJ_hGetColorTransform
@ stub -version=0xA00+ __win32kstub_NtGdiBRUSHOBJ_pvAllocRbrush
@ stub -version=0xA00+ __win32kstub_NtGdiBRUSHOBJ_pvGetRbrush
@ stub -version=0xA00+ __win32kstub_NtGdiBRUSHOBJ_ulGetBrushColor
@ stub -version=0xA00+ __win32kstub_NtGdiBeginGdiRendering
@ stub -version=0xA00+ __win32kstub_NtGdiBeginPath
@ stub -version=0xA00+ __win32kstub_NtGdiBitBlt
@ stub -version=0xA00+ __win32kstub_NtGdiCLIPOBJ_bEnum
@ stub -version=0xA00+ __win32kstub_NtGdiCLIPOBJ_cEnumStart
@ stub -version=0xA00+ __win32kstub_NtGdiCLIPOBJ_ppoGetPath
@ stub -version=0xA00+ __win32kstub_NtGdiCancelDC
@ stub -version=0xA00+ __win32kstub_NtGdiChangeGhostFont
@ stub -version=0xA00+ __win32kstub_NtGdiCheckBitmapBits
@ stub -version=0xA00+ __win32kstub_NtGdiClearBitmapAttributes
@ stub -version=0xA00+ __win32kstub_NtGdiClearBrushAttributes
@ stub -version=0xA00+ __win32kstub_NtGdiCloseFigure
@ stub -version=0xA00+ __win32kstub_NtGdiColorCorrectPalette
@ stub -version=0xA00+ __win32kstub_NtGdiCombineRgn
@ stub -version=0xA00+ __win32kstub_NtGdiCombineTransform
@ stub -version=0xA00+ __win32kstub_NtGdiComputeXformCoefficients
@ stub -version=0xA00+ __win32kstub_NtGdiConfigureOPMProtectedOutput
@ stub -version=0xA00+ __win32kstub_NtGdiConvertMetafileRect
@ stub -version=0xA00+ __win32kstub_NtGdiCreateBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiCreateBitmapFromDxSurface
@ stub -version=0xA00+ __win32kstub_NtGdiCreateBitmapFromDxSurface2
@ stub -version=0xA00+ __win32kstub_NtGdiCreateClientObj
@ stub -version=0xA00+ __win32kstub_NtGdiCreateColorSpace
@ stub -version=0xA00+ __win32kstub_NtGdiCreateColorTransform
@ stub -version=0xA00+ __win32kstub_NtGdiCreateCompatibleBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiCreateCompatibleDC
@ stub -version=0xA00+ __win32kstub_NtGdiCreateDIBBrush
@ stub -version=0xA00+ __win32kstub_NtGdiCreateDIBSection
@ stub -version=0xA00+ __win32kstub_NtGdiCreateDIBitmapInternal
@ stub -version=0xA00+ __win32kstub_NtGdiCreateEllipticRgn
@ stub -version=0xA00+ __win32kstub_NtGdiCreateHalftonePalette
@ stub -version=0xA00+ __win32kstub_NtGdiCreateHatchBrushInternal
@ stub -version=0xA00+ __win32kstub_NtGdiCreateMetafileDC
@ stub -version=0xA00+ __win32kstub_NtGdiCreateOPMProtectedOutput
@ stub -version=0xA00+ __win32kstub_NtGdiCreateOPMProtectedOutputs
@ stub -version=0xA00+ __win32kstub_NtGdiCreatePaletteInternal
@ stub -version=0xA00+ __win32kstub_NtGdiCreatePatternBrushInternal
@ stub -version=0xA00+ __win32kstub_NtGdiCreatePen
@ stub -version=0xA00+ __win32kstub_NtGdiCreateRectRgn
@ stub -version=0xA00+ __win32kstub_NtGdiCreateRoundRectRgn
@ stub -version=0xA00+ __win32kstub_NtGdiCreateServerMetaFile
@ stub -version=0xA00+ __win32kstub_NtGdiCreateSessionMappedDIBSection
@ stub -version=0xA00+ __win32kstub_NtGdiCreateSolidBrush
@ stub -version=0xA00+ __win32kstub_NtGdiDDCCIGetCapabilitiesString
@ stub -version=0xA00+ __win32kstub_NtGdiDDCCIGetCapabilitiesStringLength
@ stub -version=0xA00+ __win32kstub_NtGdiDDCCIGetTimingReport
@ stub -version=0xA00+ __win32kstub_NtGdiDDCCIGetVCPFeature
@ stub -version=0xA00+ __win32kstub_NtGdiDDCCISaveCurrentSettings
@ stub -version=0xA00+ __win32kstub_NtGdiDDCCISetVCPFeature
@ stub -version=0xA00+ __win32kstub_NtGdiDdCreateFullscreenSprite
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIAbandonSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIAcquireKeyedMutex
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIAcquireKeyedMutex2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIAcquireSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIAddSurfaceToSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIAdjustFullscreenGamma
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICacheHybridQueryValue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIChangeVideoMemoryReservation
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckExclusiveOwnership
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckMonitorPowerState
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckMultiPlaneOverlaySupport
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckMultiPlaneOverlaySupport2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckMultiPlaneOverlaySupport3
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckOcclusion
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckSharedResourceAccess
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICheckVidPnExclusiveOwnership
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICloseAdapter
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIConfigureSharedResource
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateAllocation
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateBundleObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateContext
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateContextVirtual
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateDCFromMemory
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateDevice
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateHwContext
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateHwQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateKeyedMutex
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateKeyedMutex2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateOutputDupl
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateOverlay
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreatePagingQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateProtectedSession
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDICreateSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDDisplayEnum
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyAllocation
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyAllocation2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyContext
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyDCFromMemory
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyDevice
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyHwContext
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyHwQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyKeyedMutex
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyOutputDupl
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyOverlay
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyPagingQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroyProtectedSession
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDestroySynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDispMgrCreate
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDispMgrSourceOperation
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIDispMgrTargetOperation
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIEnumAdapters
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIEnumAdapters2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIEscape
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIEvict
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIExtractBundleObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIFlipOverlay
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIFlushHeapTransitions
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIFreeGpuVirtualAddress
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetAllocationPriority
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetCachedHybridQueryValue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetContextInProcessSchedulingPriority
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetContextSchedulingPriority
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetDWMVerticalBlankEvent
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetDeviceState
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetDisplayModeList
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetMemoryBudgetTarget
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetMultiPlaneOverlayCaps
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetMultisampleMethodList
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetOverlayState
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetPostCompositionCaps
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetPresentHistory
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetPresentQueueEvent
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetProcessDeviceRemovalSupport
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetProcessSchedulingPriorityBand
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetProcessSchedulingPriorityClass
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetResourcePresentPrivateDriverData
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetRuntimeData
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetScanLine
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetSetSwapChainMetadata
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetSharedPrimaryHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetSharedResourceAdapterLuid
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetSharedResourceAdapterLuidFlipManager
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetSwapChainSurfacePhysicalAddress
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIGetYieldPercentage
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIInvalidateActiveVidPn
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIInvalidateCache
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDILock
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDILock2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIMakeResident
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIMapGpuVirtualAddress
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIMarkDeviceAsError
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDINetDispGetNextChunkInfo
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDINetDispQueryMiracastDisplayDeviceStatus
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDINetDispQueryMiracastDisplayDeviceSupport
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDINetDispStartMiracastDisplayDevice
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDINetDispStopMiracastDisplayDevice
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOfferAllocations
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenAdapterFromDeviceName
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenAdapterFromHdc
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenAdapterFromLuid
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenBundleObjectNtHandleFromName
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenKeyedMutex
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenKeyedMutex2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenKeyedMutexFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenNtHandleFromName
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenProtectedSessionFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenResource
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenResourceFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenSyncObjectFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenSyncObjectFromNtHandle2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenSyncObjectNtHandleFromName
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOpenSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOutputDuplGetFrameInfo
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOutputDuplGetMetaData
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOutputDuplGetPointerShapeData
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOutputDuplPresent
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIOutputDuplReleaseFrame
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIPollDisplayChildren
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIPresent
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIPresentMultiPlaneOverlay
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIPresentMultiPlaneOverlay2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIPresentMultiPlaneOverlay3
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIPresentRedirected
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryAdapterInfo
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryAllocationResidency
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryClockCalibration
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryFSEBlock
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryProcessOfferInfo
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryProtectedSessionInfoFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryProtectedSessionStatus
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryRemoteVidPnSourceFromGdiDisplayName
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryResourceInfo
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryResourceInfoFromNtHandle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryStatistics
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryVidPnExclusiveOwnership
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIQueryVideoMemoryInfo
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReclaimAllocations
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReclaimAllocations2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReleaseKeyedMutex
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReleaseKeyedMutex2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReleaseProcessVidPnSourceOwners
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReleaseSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIRemoveSurfaceFromSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIRender
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIReserveGpuVirtualAddress
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetAllocationPriority
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetContextInProcessSchedulingPriority
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetContextSchedulingPriority
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetDisplayMode
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetDodIndirectSwapchain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetFSEBlock
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetGammaRamp
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetHwProtectionTeardownRecovery
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetMemoryBudgetTarget
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetMonitorColorSpaceTransform
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetProcessDeviceRemovalSupport
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetProcessSchedulingPriorityBand
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetProcessSchedulingPriorityClass
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetQueuedLimit
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetStablePowerState
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetStereoEnabled
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetSyncRefreshCountWaitTarget
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetVidPnSourceHwProtection
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetVidPnSourceOwner
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISetYieldPercentage
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIShareObjects
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISharedPrimaryLockNotification
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISharedPrimaryUnLockNotification
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISignalSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISignalSynchronizationObjectFromCpu
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISignalSynchronizationObjectFromGpu
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISignalSynchronizationObjectFromGpu2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISubmitCommand
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISubmitCommandToHwQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISubmitSignalSyncObjectsToHwQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDISubmitWaitForSyncObjectsToHwQueue
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDITrimProcessCommitment
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIUnOrderedPresentSwapChain
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIUnlock
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIUnlock2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIUpdateAllocationProperty
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIUpdateGpuVirtualAddress
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIUpdateOverlay
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIWaitForIdle
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIWaitForSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIWaitForSynchronizationObjectFromCpu
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIWaitForSynchronizationObjectFromGpu
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIWaitForVerticalBlankEvent
@ stub -version=0xA00+ __win32kstub_NtGdiDdDDIWaitForVerticalBlankEvent2
@ stub -version=0xA00+ __win32kstub_NtGdiDdDestroyFullscreenSprite
@ stub -version=0xA00+ __win32kstub_NtGdiDdNotifyFullscreenSpriteUpdate
@ stub -version=0xA00+ __win32kstub_NtGdiDdQueryVisRgnUniqueness
@ stub -version=0xA00+ __win32kstub_NtGdiDeleteClientObj
@ stub -version=0xA00+ __win32kstub_NtGdiDeleteColorSpace
@ stub -version=0xA00+ __win32kstub_NtGdiDeleteColorTransform
@ stub -version=0xA00+ __win32kstub_NtGdiDeleteObjectApp
@ stub -version=0xA00+ __win32kstub_NtGdiDescribePixelFormat
@ stub -version=0xA00+ __win32kstub_NtGdiDestroyOPMProtectedOutput
@ stub -version=0xA00+ __win32kstub_NtGdiDestroyPhysicalMonitor
@ stub -version=0xA00+ __win32kstub_NtGdiDisableUMPDSandboxing
@ stub -version=0xA00+ __win32kstub_NtGdiDoBanding
@ stub -version=0xA00+ __win32kstub_NtGdiDoPalette
@ stub -version=0xA00+ __win32kstub_NtGdiDrawEscape
@ stub -version=0xA00+ __win32kstub_NtGdiDrawStream
@ stub -version=0xA00+ __win32kstub_NtGdiDwmCreatedBitmapRemotingOutput
@ stub -version=0xA00+ __win32kstub_NtGdiEllipse
@ stub -version=0xA00+ __win32kstub_NtGdiEnableEudc
@ stub -version=0xA00+ __win32kstub_NtGdiEndDoc
@ stub -version=0xA00+ __win32kstub_NtGdiEndGdiRendering
@ stub -version=0xA00+ __win32kstub_NtGdiEndPage
@ stub -version=0xA00+ __win32kstub_NtGdiEndPath
@ stub -version=0xA00+ __win32kstub_NtGdiEngAlphaBlend
@ stub -version=0xA00+ __win32kstub_NtGdiEngAssociateSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEngBitBlt
@ stub -version=0xA00+ __win32kstub_NtGdiEngCheckAbort
@ stub -version=0xA00+ __win32kstub_NtGdiEngComputeGlyphSet
@ stub -version=0xA00+ __win32kstub_NtGdiEngCopyBits
@ stub -version=0xA00+ __win32kstub_NtGdiEngCreateBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiEngCreateClip
@ stub -version=0xA00+ __win32kstub_NtGdiEngCreateDeviceBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiEngCreateDeviceSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEngCreatePalette
@ stub -version=0xA00+ __win32kstub_NtGdiEngDeleteClip
@ stub -version=0xA00+ __win32kstub_NtGdiEngDeletePalette
@ stub -version=0xA00+ __win32kstub_NtGdiEngDeletePath
@ stub -version=0xA00+ __win32kstub_NtGdiEngDeleteSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEngEraseSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEngFillPath
@ stub -version=0xA00+ __win32kstub_NtGdiEngGradientFill
@ stub -version=0xA00+ __win32kstub_NtGdiEngLineTo
@ stub -version=0xA00+ __win32kstub_NtGdiEngLockSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEngMarkBandingSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEngPaint
@ stub -version=0xA00+ __win32kstub_NtGdiEngPlgBlt
@ stub -version=0xA00+ __win32kstub_NtGdiEngStretchBlt
@ stub -version=0xA00+ __win32kstub_NtGdiEngStretchBltROP
@ stub -version=0xA00+ __win32kstub_NtGdiEngStrokeAndFillPath
@ stub -version=0xA00+ __win32kstub_NtGdiEngStrokePath
@ stub -version=0xA00+ __win32kstub_NtGdiEngTextOut
@ stub -version=0xA00+ __win32kstub_NtGdiEngTransparentBlt
@ stub -version=0xA00+ __win32kstub_NtGdiEngUnlockSurface
@ stub -version=0xA00+ __win32kstub_NtGdiEnsureDpiDepDefaultGuiFontForPlateau
@ stub -version=0xA00+ __win32kstub_NtGdiEnumFonts
@ stub -version=0xA00+ __win32kstub_NtGdiEnumObjects
@ stub -version=0xA00+ __win32kstub_NtGdiEqualRgn
@ stub -version=0xA00+ __win32kstub_NtGdiEudcLoadUnloadLink
@ stub -version=0xA00+ __win32kstub_NtGdiExcludeClipRect
@ stub -version=0xA00+ __win32kstub_NtGdiExtCreatePen
@ stub -version=0xA00+ __win32kstub_NtGdiExtCreateRegion
@ stub -version=0xA00+ __win32kstub_NtGdiExtEscape
@ stub -version=0xA00+ __win32kstub_NtGdiExtFloodFill
@ stub -version=0xA00+ __win32kstub_NtGdiExtGetObjectW
@ stub -version=0xA00+ __win32kstub_NtGdiExtSelectClipRgn
@ stub -version=0xA00+ __win32kstub_NtGdiExtTextOutW
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_cGetAllGlyphHandles
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_cGetGlyphs
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_pQueryGlyphAttrs
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_pfdg
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_pifi
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_pvTrueTypeFontFile
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_pxoGetXform
@ stub -version=0xA00+ __win32kstub_NtGdiFONTOBJ_vGetInfo
@ stub -version=0xA00+ __win32kstub_NtGdiFillPath
@ stub -version=0xA00+ __win32kstub_NtGdiFillRgn
@ stub -version=0xA00+ __win32kstub_NtGdiFlattenPath
@ stub -version=0xA00+ __win32kstub_NtGdiFlush
@ stub -version=0xA00+ __win32kstub_NtGdiFontIsLinked
@ stub -version=0xA00+ __win32kstub_NtGdiForceUFIMapping
@ stub -version=0xA00+ __win32kstub_NtGdiFrameRgn
@ stub -version=0xA00+ __win32kstub_NtGdiFullscreenControl
@ stub -version=0xA00+ __win32kstub_NtGdiGetAndSetDCDword
@ stub -version=0xA00+ __win32kstub_NtGdiGetAppClipBox
@ stub -version=0xA00+ __win32kstub_NtGdiGetBitmapBits
@ stub -version=0xA00+ __win32kstub_NtGdiGetBitmapDimension
@ stub -version=0xA00+ __win32kstub_NtGdiGetBitmapDpiScaleValue
@ stub -version=0xA00+ __win32kstub_NtGdiGetBoundsRect
@ stub -version=0xA00+ __win32kstub_NtGdiGetCOPPCompatibleOPMInformation
@ stub -version=0xA00+ __win32kstub_NtGdiGetCertificate
@ stub -version=0xA00+ __win32kstub_NtGdiGetCertificateByHandle
@ stub -version=0xA00+ __win32kstub_NtGdiGetCertificateSize
@ stub -version=0xA00+ __win32kstub_NtGdiGetCertificateSizeByHandle
@ stub -version=0xA00+ __win32kstub_NtGdiGetCharABCWidthsW
@ stub -version=0xA00+ __win32kstub_NtGdiGetCharSet
@ stub -version=0xA00+ __win32kstub_NtGdiGetCharWidthInfo
@ stub -version=0xA00+ __win32kstub_NtGdiGetCharWidthW
@ stub -version=0xA00+ __win32kstub_NtGdiGetCharacterPlacementW
@ stub -version=0xA00+ __win32kstub_NtGdiGetColorAdjustment
@ stub -version=0xA00+ __win32kstub_NtGdiGetColorSpaceforBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiGetCurrentDpiInfo
@ stub -version=0xA00+ __win32kstub_NtGdiGetDCDpiScaleValue
@ stub -version=0xA00+ __win32kstub_NtGdiGetDCDword
@ stub -version=0xA00+ __win32kstub_NtGdiGetDCObject
@ stub -version=0xA00+ __win32kstub_NtGdiGetDCPoint
@ stub -version=0xA00+ __win32kstub_NtGdiGetDCforBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiGetDIBitsInternal
@ stub -version=0xA00+ __win32kstub_NtGdiGetDeviceCaps
@ stub -version=0xA00+ __win32kstub_NtGdiGetDeviceCapsAll
@ stub -version=0xA00+ __win32kstub_NtGdiGetDeviceWidth
@ stub -version=0xA00+ __win32kstub_NtGdiGetDhpdev
@ stub -version=0xA00+ __win32kstub_NtGdiGetETM
@ stub -version=0xA00+ __win32kstub_NtGdiGetEmbUFI
@ stub -version=0xA00+ __win32kstub_NtGdiGetEmbedFonts
@ stub -version=0xA00+ __win32kstub_NtGdiGetEntry
@ stub -version=0xA00+ __win32kstub_NtGdiGetEudcTimeStampEx
@ stub -version=0xA00+ __win32kstub_NtGdiGetFontData
@ stub -version=0xA00+ __win32kstub_NtGdiGetFontFileData
@ stub -version=0xA00+ __win32kstub_NtGdiGetFontFileInfo
@ stub -version=0xA00+ __win32kstub_NtGdiGetFontResourceInfoInternalW
@ stub -version=0xA00+ __win32kstub_NtGdiGetFontUnicodeRanges
@ stub -version=0xA00+ __win32kstub_NtGdiGetGlyphIndicesW
@ stub -version=0xA00+ __win32kstub_NtGdiGetGlyphIndicesWInternal
@ stub -version=0xA00+ __win32kstub_NtGdiGetGlyphOutline
@ stub -version=0xA00+ __win32kstub_NtGdiGetKerningPairs
@ stub -version=0xA00+ __win32kstub_NtGdiGetLinkedUFIs
@ stub -version=0xA00+ __win32kstub_NtGdiGetMiterLimit
@ stub -version=0xA00+ __win32kstub_NtGdiGetMonitorID
@ stub -version=0xA00+ __win32kstub_NtGdiGetNearestColor
@ stub -version=0xA00+ __win32kstub_NtGdiGetNearestPaletteIndex
@ stub -version=0xA00+ __win32kstub_NtGdiGetNumberOfPhysicalMonitors
@ stub -version=0xA00+ __win32kstub_NtGdiGetOPMInformation
@ stub -version=0xA00+ __win32kstub_NtGdiGetOPMRandomNumber
@ stub -version=0xA00+ __win32kstub_NtGdiGetObjectBitmapHandle
@ stub -version=0xA00+ __win32kstub_NtGdiGetOutlineTextMetricsInternalW
@ stub -version=0xA00+ __win32kstub_NtGdiGetPath
@ stub -version=0xA00+ __win32kstub_NtGdiGetPerBandInfo
@ stub -version=0xA00+ __win32kstub_NtGdiGetPhysicalMonitorDescription
@ stub -version=0xA00+ __win32kstub_NtGdiGetPhysicalMonitorFromTarget
@ stub -version=0xA00+ __win32kstub_NtGdiGetPhysicalMonitors
@ stub -version=0xA00+ __win32kstub_NtGdiGetPixel
@ stub -version=0xA00+ __win32kstub_NtGdiGetProcessSessionFonts
@ stub -version=0xA00+ __win32kstub_NtGdiGetPublicFontTableChangeCookie
@ stub -version=0xA00+ __win32kstub_NtGdiGetRandomRgn
@ stub -version=0xA00+ __win32kstub_NtGdiGetRasterizerCaps
@ stub -version=0xA00+ __win32kstub_NtGdiGetRealizationInfo
@ stub -version=0xA00+ __win32kstub_NtGdiGetRegionData
@ stub -version=0xA00+ __win32kstub_NtGdiGetRgnBox
@ stub -version=0xA00+ __win32kstub_NtGdiGetServerMetaFileBits
@ stub -version=0xA00+ __win32kstub_NtGdiGetSpoolMessage
@ stub -version=0xA00+ __win32kstub_NtGdiGetStats
@ stub -version=0xA00+ __win32kstub_NtGdiGetStringBitmapW
@ stub -version=0xA00+ __win32kstub_NtGdiGetSuggestedOPMProtectedOutputArraySize
@ stub -version=0xA00+ __win32kstub_NtGdiGetSystemPaletteUse
@ stub -version=0xA00+ __win32kstub_NtGdiGetTextCharsetInfo
@ stub -version=0xA00+ __win32kstub_NtGdiGetTextExtent
@ stub -version=0xA00+ __win32kstub_NtGdiGetTextExtentExW
@ stub -version=0xA00+ __win32kstub_NtGdiGetTextFaceW
@ stub -version=0xA00+ __win32kstub_NtGdiGetTextMetricsW
@ stub -version=0xA00+ __win32kstub_NtGdiGetTransform
@ stub -version=0xA00+ __win32kstub_NtGdiGetUFI
@ stub -version=0xA00+ __win32kstub_NtGdiGetUFIPathname
@ stub -version=0xA00+ __win32kstub_NtGdiGetWidthTable
@ stub -version=0xA00+ __win32kstub_NtGdiGradientFill
@ stub -version=0xA00+ __win32kstub_NtGdiHLSurfGetInformation
@ stub -version=0xA00+ __win32kstub_NtGdiHLSurfSetInformation
@ stub -version=0xA00+ __win32kstub_NtGdiHT_Get8BPPFormatPalette
@ stub -version=0xA00+ __win32kstub_NtGdiHT_Get8BPPMaskPalette
@ stub -version=0xA00+ __win32kstub_NtGdiHfontCreate
@ stub -version=0xA00+ __win32kstub_NtGdiIcmBrushInfo
@ stub -version=0xA00+ __win32kstub_NtGdiInit
@ stub -version=0xA00+ __win32kstub_NtGdiInit2
@ stub -version=0xA00+ __win32kstub_NtGdiInitSpool
@ stub -version=0xA00+ __win32kstub_NtGdiIntersectClipRect
@ stub -version=0xA00+ __win32kstub_NtGdiInvertRgn
@ stub -version=0xA00+ __win32kstub_NtGdiLineTo
@ stub -version=0xA00+ __win32kstub_NtGdiMakeFontDir
@ stub -version=0xA00+ __win32kstub_NtGdiMakeInfoDC
@ stub -version=0xA00+ __win32kstub_NtGdiMakeObjectUnXferable
@ stub -version=0xA00+ __win32kstub_NtGdiMakeObjectXferable
@ stub -version=0xA00+ __win32kstub_NtGdiMaskBlt
@ stub -version=0xA00+ __win32kstub_NtGdiMirrorWindowOrg
@ stub -version=0xA00+ __win32kstub_NtGdiModifyWorldTransform
@ stub -version=0xA00+ __win32kstub_NtGdiMonoBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiMoveTo
@ stub -version=0xA00+ __win32kstub_NtGdiOffsetClipRgn
@ stub -version=0xA00+ __win32kstub_NtGdiOffsetRgn
@ stub -version=0xA00+ __win32kstub_NtGdiOpenDCW
@ stub -version=0xA00+ __win32kstub_NtGdiPATHOBJ_bEnum
@ stub -version=0xA00+ __win32kstub_NtGdiPATHOBJ_bEnumClipLines
@ stub -version=0xA00+ __win32kstub_NtGdiPATHOBJ_vEnumStart
@ stub -version=0xA00+ __win32kstub_NtGdiPATHOBJ_vEnumStartClipLines
@ stub -version=0xA00+ __win32kstub_NtGdiPATHOBJ_vGetBounds
@ stub -version=0xA00+ __win32kstub_NtGdiPatBlt
@ stub -version=0xA00+ __win32kstub_NtGdiPathToRegion
@ stub -version=0xA00+ __win32kstub_NtGdiPlgBlt
@ stub -version=0xA00+ __win32kstub_NtGdiPolyDraw
@ stub -version=0xA00+ __win32kstub_NtGdiPolyPatBlt
@ stub -version=0xA00+ __win32kstub_NtGdiPolyPolyDraw
@ stub -version=0xA00+ __win32kstub_NtGdiPolyTextOutW
@ stub -version=0xA00+ __win32kstub_NtGdiPtInRegion
@ stub -version=0xA00+ __win32kstub_NtGdiPtVisible
@ stub -version=0xA00+ __win32kstub_NtGdiQueryFontAssocInfo
@ stub -version=0xA00+ __win32kstub_NtGdiQueryFonts
@ stub -version=0xA00+ __win32kstub_NtGdiRectInRegion
@ stub -version=0xA00+ __win32kstub_NtGdiRectVisible
@ stub -version=0xA00+ __win32kstub_NtGdiRectangle
@ stub -version=0xA00+ __win32kstub_NtGdiRemoveFontMemResourceEx
@ stub -version=0xA00+ __win32kstub_NtGdiRemoveFontResourceW
@ stub -version=0xA00+ __win32kstub_NtGdiRemoveMergeFont
@ stub -version=0xA00+ __win32kstub_NtGdiResetDC
@ stub -version=0xA00+ __win32kstub_NtGdiResizePalette
@ stub -version=0xA00+ __win32kstub_NtGdiRestoreDC
@ stub -version=0xA00+ __win32kstub_NtGdiRoundRect
@ stub -version=0xA00+ __win32kstub_NtGdiSTROBJ_bEnum
@ stub -version=0xA00+ __win32kstub_NtGdiSTROBJ_bEnumPositionsOnly
@ stub -version=0xA00+ __win32kstub_NtGdiSTROBJ_bGetAdvanceWidths
@ stub -version=0xA00+ __win32kstub_NtGdiSTROBJ_dwGetCodePage
@ stub -version=0xA00+ __win32kstub_NtGdiSTROBJ_vEnumStart
@ stub -version=0xA00+ __win32kstub_NtGdiSaveDC
@ stub -version=0xA00+ __win32kstub_NtGdiScaleRgn
@ stub -version=0xA00+ __win32kstub_NtGdiScaleValues
@ stub -version=0xA00+ __win32kstub_NtGdiScaleViewportExtEx
@ stub -version=0xA00+ __win32kstub_NtGdiScaleWindowExtEx
@ stub -version=0xA00+ __win32kstub_NtGdiSelectBitmap
@ stub -version=0xA00+ __win32kstub_NtGdiSelectBrush
@ stub -version=0xA00+ __win32kstub_NtGdiSelectClipPath
@ stub -version=0xA00+ __win32kstub_NtGdiSelectFont
@ stub -version=0xA00+ __win32kstub_NtGdiSelectPen
@ stub -version=0xA00+ __win32kstub_NtGdiSetBitmapAttributes
@ stub -version=0xA00+ __win32kstub_NtGdiSetBitmapBits
@ stub -version=0xA00+ __win32kstub_NtGdiSetBitmapDimension
@ stub -version=0xA00+ __win32kstub_NtGdiSetBoundsRect
@ stub -version=0xA00+ __win32kstub_NtGdiSetBrushAttributes
@ stub -version=0xA00+ __win32kstub_NtGdiSetBrushOrg
@ stub -version=0xA00+ __win32kstub_NtGdiSetColorAdjustment
@ stub -version=0xA00+ __win32kstub_NtGdiSetColorSpace
@ stub -version=0xA00+ __win32kstub_NtGdiSetDIBitsToDeviceInternal
@ stub -version=0xA00+ __win32kstub_NtGdiSetFontEnumeration
@ stub -version=0xA00+ __win32kstub_NtGdiSetFontXform
@ stub -version=0xA00+ __win32kstub_NtGdiSetIcmMode
@ stub -version=0xA00+ __win32kstub_NtGdiSetLayout
@ stub -version=0xA00+ __win32kstub_NtGdiSetLinkedUFIs
@ stub -version=0xA00+ __win32kstub_NtGdiSetMagicColors
@ stub -version=0xA00+ __win32kstub_NtGdiSetMetaRgn
@ stub -version=0xA00+ __win32kstub_NtGdiSetMiterLimit
@ stub -version=0xA00+ __win32kstub_NtGdiSetOPMSigningKeyAndSequenceNumbers
@ stub -version=0xA00+ __win32kstub_NtGdiSetPUMPDOBJ
@ stub -version=0xA00+ __win32kstub_NtGdiSetPixel
@ stub -version=0xA00+ __win32kstub_NtGdiSetPixelFormat
@ stub -version=0xA00+ __win32kstub_NtGdiSetRectRgn
@ stub -version=0xA00+ __win32kstub_NtGdiSetSizeDevice
@ stub -version=0xA00+ __win32kstub_NtGdiSetSystemPaletteUse
@ stub -version=0xA00+ __win32kstub_NtGdiSetTextJustification
@ stub -version=0xA00+ __win32kstub_NtGdiSetUMPDSandboxState
@ stub -version=0xA00+ __win32kstub_NtGdiSetVirtualResolution
@ stub -version=0xA00+ __win32kstub_NtGdiStartDoc
@ stub -version=0xA00+ __win32kstub_NtGdiStartPage
@ stub -version=0xA00+ __win32kstub_NtGdiStretchBlt
@ stub -version=0xA00+ __win32kstub_NtGdiStretchDIBitsInternal
@ stub -version=0xA00+ __win32kstub_NtGdiStrokeAndFillPath
@ stub -version=0xA00+ __win32kstub_NtGdiStrokePath
@ stub -version=0xA00+ __win32kstub_NtGdiSwapBuffers
@ stub -version=0xA00+ __win32kstub_NtGdiTransformPoints
@ stub -version=0xA00+ __win32kstub_NtGdiTransparentBlt
@ stub -version=0xA00+ __win32kstub_NtGdiUMPDEngFreeUserMem
@ stub -version=0xA00+ __win32kstub_NtGdiUnloadPrinterDriver
@ stub -version=0xA00+ __win32kstub_NtGdiUnmapMemFont
@ stub -version=0xA00+ __win32kstub_NtGdiUnrealizeObject
@ stub -version=0xA00+ __win32kstub_NtGdiUpdateColors
@ stub -version=0xA00+ __win32kstub_NtGdiUpdateTransform
@ stub -version=0xA00+ __win32kstub_NtGdiWaitForTextReady
@ stub -version=0xA00+ __win32kstub_NtGdiWidenPath
@ stub -version=0xA00+ __win32kstub_NtGdiXFORMOBJ_bApplyXform
@ stub -version=0xA00+ __win32kstub_NtGdiXFORMOBJ_iGetXform
@ stub -version=0xA00+ __win32kstub_NtGdiXLATEOBJ_cGetPalette
@ stub -version=0xA00+ __win32kstub_NtGdiXLATEOBJ_hGetColorTransform
@ stub -version=0xA00+ __win32kstub_NtGdiXLATEOBJ_iXlate
@ stub -version=0xA00+ __win32kstub_NtHWCursorUpdatePointer
@ stub -version=0xA00+ __win32kstub_NtInputSpaceRegionFromPoint
@ stub -version=0xA00+ __win32kstub_NtIsOneCoreTransformMode
@ stub -version=0xA00+ __win32kstub_NtKSTInitialize
@ stub -version=0xA00+ __win32kstub_NtKSTWait
@ stub -version=0xA00+ __win32kstub_NtMITAccessibilityTimerNotification
@ stub -version=0xA00+ __win32kstub_NtMITActivateInputProcessing
@ stub -version=0xA00+ __win32kstub_NtMITConfigureVirtualTouchpad
@ stub -version=0xA00+ __win32kstub_NtMITCoreMsgKOpenConnectionTo
@ stub -version=0xA00+ __win32kstub_NtMITDeactivateInputProcessing
@ stub -version=0xA00+ __win32kstub_NtMITDisableMouseIntercept
@ stub -version=0xA00+ __win32kstub_NtMITDispatchCompletion
@ stub -version=0xA00+ __win32kstub_NtMITEnableMouseIntercept
@ stub -version=0xA00+ __win32kstub_NtMITGetCursorUpdateHandle
@ stub -version=0xA00+ __win32kstub_NtMITInitMinuserThread
@ stub -version=0xA00+ __win32kstub_NtMITMinuserSetInputTransformOffset
@ stub -version=0xA00+ __win32kstub_NtMITMinuserWindowDestroyed
@ stub -version=0xA00+ __win32kstub_NtMITPostMouseInputMessage
@ stub -version=0xA00+ __win32kstub_NtMITPostThreadEventMessage
@ stub -version=0xA00+ __win32kstub_NtMITPostWindowEventMessage
@ stub -version=0xA00+ __win32kstub_NtMITPrepareReceiveInputMessage
@ stub -version=0xA00+ __win32kstub_NtMITPrepareSendInputMessage
@ stub -version=0xA00+ __win32kstub_NtMITProcessDelegateCapturedPointers
@ stub -version=0xA00+ __win32kstub_NtMITSetInputCallbacks
@ stub -version=0xA00+ __win32kstub_NtMITSetInputDelegationMode
@ stub -version=0xA00+ __win32kstub_NtMITSetInputObservationState
@ stub -version=0xA00+ __win32kstub_NtMITSetKeyboardInputRoutingPolicy
@ stub -version=0xA00+ __win32kstub_NtMITSetKeyboardOverriderState
@ stub -version=0xA00+ __win32kstub_NtMITSetLastInputRecipient
@ stub -version=0xA00+ __win32kstub_NtMITSynthesizeKeyboardInput
@ stub -version=0xA00+ __win32kstub_NtMITSynthesizeMouseInput
@ stub -version=0xA00+ __win32kstub_NtMITSynthesizeTouchInput
@ stub -version=0xA00+ __win32kstub_NtMITUninitMinuserThread
@ stub -version=0xA00+ __win32kstub_NtMITUpdateInputGlobals
@ stub -version=0xA00+ __win32kstub_NtMapVisualRelativePoints
@ stub -version=0xA00+ __win32kstub_NtMinGetInputTransform
@ stub -version=0xA00+ __win32kstub_NtMinInteropCoreMessagingWithInput
@ stub -version=0xA00+ __win32kstub_NtMinQPeekForInput
@ stub -version=0xA00+ __win32kstub_NtMinQSuspendInputProcessing
@ stub -version=0xA00+ __win32kstub_NtMinQUpdateWakeMask
@ stub -version=0xA00+ __win32kstub_NtModerncoreBeginLayoutUpdate
@ stub -version=0xA00+ __win32kstub_NtModerncoreCreateDCompositionHwndTarget
@ stub -version=0xA00+ __win32kstub_NtModerncoreCreateGDIHwndTarget
@ stub -version=0xA00+ __win32kstub_NtModerncoreDestroyDCompositionHwndTarget
@ stub -version=0xA00+ __win32kstub_NtModerncoreDestroyGDIHwndTarget
@ stub -version=0xA00+ __win32kstub_NtModerncoreEnableResizeLayoutSynchronization
@ stub -version=0xA00+ __win32kstub_NtModerncoreGetNavigationWindowVisual
@ stub -version=0xA00+ __win32kstub_NtModerncoreGetResizeDCompositionSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtModerncoreGetWindowContentVisual
@ stub -version=0xA00+ __win32kstub_NtModerncoreIdleTimerThread
@ stub -version=0xA00+ __win32kstub_NtModerncoreIsResizeLayoutSynchronizationEnabled
@ stub -version=0xA00+ __win32kstub_NtModerncoreProcessConnect
@ stub -version=0xA00+ __win32kstub_NtModerncoreRegisterEnhancedNavigationWindowHandle
@ stub -version=0xA00+ __win32kstub_NtModerncoreRegisterNavigationWindowHandle
@ stub -version=0xA00+ __win32kstub_NtModerncoreSetNavigationServiceSid
@ stub -version=0xA00+ __win32kstub_NtModerncoreUnregisterNavigationWindowHandle
@ stub -version=0xA00+ __win32kstub_NtNotifyPresentToCompositionSurface
@ stub -version=0xA00+ __win32kstub_NtOpenCompositionSurfaceDirtyRegion
@ stub -version=0xA00+ __win32kstub_NtOpenCompositionSurfaceRealizationInfo
@ stub -version=0xA00+ __win32kstub_NtOpenCompositionSurfaceSectionInfo
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionInputIsImplicit
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionInputQueueAndTransform
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionInputSink
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionInputSinkLuid
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionInputSinkViewId
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionSurfaceBinding
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionSurfaceFrameRate
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionSurfaceHDRMetaData
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionSurfaceRenderingRealization
@ stub -version=0xA00+ __win32kstub_NtQueryCompositionSurfaceStatistics
@ stub -version=0xA00+ __win32kstub_NtRIMAddInputObserver
@ stub -version=0xA00+ __win32kstub_NtRIMAreSiblingDevices
@ stub -version=0xA00+ __win32kstub_NtRIMDeviceIoControl
@ stub -version=0xA00+ __win32kstub_NtRIMEnableMonitorMappingForDevice
@ stub -version=0xA00+ __win32kstub_NtRIMFreeInputBuffer
@ stub -version=0xA00+ __win32kstub_NtRIMGetDevicePreparsedData
@ stub -version=0xA00+ __win32kstub_NtRIMGetDevicePreparsedDataLockfree
@ stub -version=0xA00+ __win32kstub_NtRIMGetDeviceProperties
@ stub -version=0xA00+ __win32kstub_NtRIMGetDevicePropertiesLockfree
@ stub -version=0xA00+ __win32kstub_NtRIMGetPhysicalDeviceRect
@ stub -version=0xA00+ __win32kstub_NtRIMGetSourceProcessId
@ stub -version=0xA00+ __win32kstub_NtRIMObserveNextInput
@ stub -version=0xA00+ __win32kstub_NtRIMOnAsyncPnpWorkNotification
@ stub -version=0xA00+ __win32kstub_NtRIMOnPnpNotification
@ stub -version=0xA00+ __win32kstub_NtRIMOnTimerNotification
@ stub -version=0xA00+ __win32kstub_NtRIMQueryDevicePath
@ stub -version=0xA00+ __win32kstub_NtRIMReadInput
@ stub -version=0xA00+ __win32kstub_NtRIMRegisterForInputEx
@ stub -version=0xA00+ __win32kstub_NtRIMRemoveInputObserver
@ stub -version=0xA00+ __win32kstub_NtRIMSetDeadzoneRotation
@ stub -version=0xA00+ __win32kstub_NtRIMSetExtendedDeviceProperty
@ stub -version=0xA00+ __win32kstub_NtRIMSetTestModeStatus
@ stub -version=0xA00+ __win32kstub_NtRIMUnregisterForInput
@ stub -version=0xA00+ __win32kstub_NtRIMUpdateInputObserverRegistration
@ stub -version=0xA00+ __win32kstub_NtSetCompositionSurfaceAnalogExclusive
@ stub -version=0xA00+ __win32kstub_NtSetCompositionSurfaceBufferUsage
@ stub -version=0xA00+ __win32kstub_NtSetCompositionSurfaceDirectFlipState
@ stub -version=0xA00+ __win32kstub_NtSetCompositionSurfaceIndependentFlipInfo
@ stub -version=0xA00+ __win32kstub_NtSetCompositionSurfaceStatistics
@ stub -version=0xA00+ __win32kstub_NtSetCursorInputSpace
@ stub -version=0xA00+ __win32kstub_NtSetPointerDeviceInputSpace
@ stub -version=0xA00+ __win32kstub_NtSetShellCursorState
@ stub -version=0xA00+ __win32kstub_NtTokenManagerConfirmOutstandingAnalogToken
@ stub -version=0xA00+ __win32kstub_NtTokenManagerCreateCompositionTokenHandle
@ stub -version=0xA00+ __win32kstub_NtTokenManagerCreateFlipObjectReturnTokenHandle
@ stub -version=0xA00+ __win32kstub_NtTokenManagerCreateFlipObjectTokenHandle
@ stub -version=0xA00+ __win32kstub_NtTokenManagerGetAnalogExclusiveSurfaceUpdates
@ stub -version=0xA00+ __win32kstub_NtTokenManagerGetAnalogExclusiveTokenEvent
@ stub -version=0xA00+ __win32kstub_NtTokenManagerOpenSectionAndEvents
@ stub -version=0xA00+ __win32kstub_NtTokenManagerThread
@ stub -version=0xA00+ __win32kstub_NtUnBindCompositionSurface
@ stub -version=0xA00+ __win32kstub_NtUpdateInputSinkTransforms
@ stub -version=0xA00+ __win32kstub_NtUserAcquireIAMKey
@ stub -version=0xA00+ __win32kstub_NtUserAcquireInteractiveControlBackgroundAccess
@ stub -version=0xA00+ __win32kstub_NtUserActivateKeyboardLayout
@ stub -version=0xA00+ __win32kstub_NtUserAddClipboardFormatListener
@ stub -version=0xA00+ __win32kstub_NtUserAddVisualIdentifier
@ stub -version=0xA00+ __win32kstub_NtUserAllowSetForegroundWindow
@ stub -version=0xA00+ __win32kstub_NtUserAlterWindowStyle
@ stub -version=0xA00+ __win32kstub_NtUserApplyWindowAction
@ stub -version=0xA00+ __win32kstub_NtUserArrangeIconicWindows
@ stub -version=0xA00+ __win32kstub_NtUserAssociateInputContext
@ stub -version=0xA00+ __win32kstub_NtUserAttachThreadInput
@ stub -version=0xA00+ __win32kstub_NtUserAutoPromoteMouseInPointer
@ stub -version=0xA00+ __win32kstub_NtUserAutoRotateScreen
@ stub -version=0xA00+ __win32kstub_NtUserBeginDeferWindowPos
@ stub -version=0xA00+ __win32kstub_NtUserBeginLayoutUpdate
@ stub -version=0xA00+ __win32kstub_NtUserBeginPaint
@ stub -version=0xA00+ __win32kstub_NtUserBitBltSysBmp
@ stub -version=0xA00+ __win32kstub_NtUserBlockInput
@ stub -version=0xA00+ __win32kstub_NtUserBroadcastImeShowStatusChange
@ stub -version=0xA00+ __win32kstub_NtUserBroadcastThemeChangeEvent
@ stub -version=0xA00+ __win32kstub_NtUserBuildHimcList
@ stub -version=0xA00+ __win32kstub_NtUserBuildHwndList
@ stub -version=0xA00+ __win32kstub_NtUserBuildNameList
@ stub -version=0xA00+ __win32kstub_NtUserBuildPropList
@ stub -version=0xA00+ __win32kstub_NtUserCalcMenuBar
@ stub -version=0xA00+ __win32kstub_NtUserCalculatePopupWindowPosition
@ stub -version=0xA00+ __win32kstub_NtUserCallMsgFilter
@ stub -version=0xA00+ __win32kstub_NtUserCallNextHookEx
@ stub -version=0xA00+ __win32kstub_NtUserCanCurrentThreadChangeForeground
@ stub -version=0xA00+ __win32kstub_NtUserCancelQueueEventCompletionPacket
@ stub -version=0xA00+ __win32kstub_NtUserChangeClipboardChain
@ stub -version=0xA00+ __win32kstub_NtUserChangeDisplaySettings
@ stub -version=0xA00+ __win32kstub_NtUserChangeWindowMessageFilter
@ stub -version=0xA00+ __win32kstub_NtUserChangeWindowMessageFilterEx
@ stub -version=0xA00+ __win32kstub_NtUserCheckImeShowStatusInThread
@ stub -version=0xA00+ __win32kstub_NtUserCheckMenuItem
@ stub -version=0xA00+ __win32kstub_NtUserCheckProcessForClipboardAccess
@ stub -version=0xA00+ __win32kstub_NtUserCheckProcessSession
@ stub -version=0xA00+ __win32kstub_NtUserCheckWindowThreadDesktop
@ stub -version=0xA00+ __win32kstub_NtUserChildWindowFromPointEx
@ stub -version=0xA00+ __win32kstub_NtUserCitSetInfo
@ stub -version=0xA00+ __win32kstub_NtUserClearForeground
@ stub -version=0xA00+ __win32kstub_NtUserClearWakeMask
@ stub -version=0xA00+ __win32kstub_NtUserClearWindowState
@ stub -version=0xA00+ __win32kstub_NtUserClipCursor
@ stub -version=0xA00+ __win32kstub_NtUserCloseClipboard
@ stub -version=0xA00+ __win32kstub_NtUserCloseDesktop
@ stub -version=0xA00+ __win32kstub_NtUserCloseWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserCompositionInputSinkLuidFromPoint
@ stub -version=0xA00+ __win32kstub_NtUserCompositionInputSinkViewInstanceIdFromPoint
@ stub -version=0xA00+ __win32kstub_NtUserConfigureActivationObject
@ stub -version=0xA00+ __win32kstub_NtUserConfirmResizeCommit
@ stub -version=0xA00+ __win32kstub_NtUserConsoleControl
@ stub -version=0xA00+ __win32kstub_NtUserConvertMemHandle
@ stub -version=0xA00+ __win32kstub_NtUserConvertPrimaryPointerToMouseDrag
@ stub -version=0xA00+ __win32kstub_NtUserConvertToInterceptWindow
@ stub -version=0xA00+ __win32kstub_NtUserCopyAcceleratorTable
@ stub -version=0xA00+ __win32kstub_NtUserCountClipboardFormats
@ stub -version=0xA00+ __win32kstub_NtUserCreateAcceleratorTable
@ stub -version=0xA00+ __win32kstub_NtUserCreateActivationObject
@ stub -version=0xA00+ __win32kstub_NtUserCreateBaseWindow
@ stub -version=0xA00+ __win32kstub_NtUserCreateCaret
@ stub -version=0xA00+ __win32kstub_NtUserCreateDCompositionHwndTarget
@ stub -version=0xA00+ __win32kstub_NtUserCreateDesktopEx
@ stub -version=0xA00+ __win32kstub_NtUserCreateEmptyCursorObject
@ stub -version=0xA00+ __win32kstub_NtUserCreateInputContext
@ stub -version=0xA00+ __win32kstub_NtUserCreateLayoutSyncForHwnd
@ stub -version=0xA00+ __win32kstub_NtUserCreateLocalMemHandle
@ stub -version=0xA00+ __win32kstub_NtUserCreateMenu
@ stub -version=0xA00+ __win32kstub_NtUserCreatePalmRejectionDelayZone
@ stub -version=0xA00+ __win32kstub_NtUserCreatePopupMenu
@ stub -version=0xA00+ __win32kstub_NtUserCreateSyntheticPointerDevice2
@ stub -version=0xA00+ __win32kstub_NtUserCreateSystemThreads
@ stub -version=0xA00+ __win32kstub_NtUserCreateWindowEx
@ stub -version=0xA00+ __win32kstub_NtUserCreateWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserCreateWindowStationEx
@ stub -version=0xA00+ __win32kstub_NtUserCsDdeUninitialize
@ stub -version=0xA00+ __win32kstub_NtUserCtxDisplayIOCtl
@ stub -version=0xA00+ __win32kstub_NtUserDWP_GetEnabledPopupOffset
@ stub -version=0xA00+ __win32kstub_NtUserDdeInitialize
@ stub -version=0xA00+ __win32kstub_NtUserDefSetText
@ stub -version=0xA00+ __win32kstub_NtUserDeferWindowDpiChanges
@ stub -version=0xA00+ __win32kstub_NtUserDeferWindowPosAndBand
@ stub -version=0xA00+ __win32kstub_NtUserDeferredDesktopRotation
@ stub -version=0xA00+ __win32kstub_NtUserDelegateCapturePointers
@ stub -version=0xA00+ __win32kstub_NtUserDelegateInput
@ stub -version=0xA00+ __win32kstub_NtUserDeleteMenu
@ stub -version=0xA00+ __win32kstub_NtUserDeregisterShellHookWindow
@ stub -version=0xA00+ __win32kstub_NtUserDestroyAcceleratorTable
@ stub -version=0xA00+ __win32kstub_NtUserDestroyActivationObject
@ stub -version=0xA00+ __win32kstub_NtUserDestroyCaret
@ stub -version=0xA00+ __win32kstub_NtUserDestroyCursor
@ stub -version=0xA00+ __win32kstub_NtUserDestroyDCompositionHwndTarget
@ stub -version=0xA00+ __win32kstub_NtUserDestroyInputContext
@ stub -version=0xA00+ __win32kstub_NtUserDestroyMenu
@ stub -version=0xA00+ __win32kstub_NtUserDestroyPalmRejectionDelayZone
@ stub -version=0xA00+ __win32kstub_NtUserDestroyWindow
@ stub -version=0xA00+ __win32kstub_NtUserDisableImmersiveOwner
@ stub -version=0xA00+ __win32kstub_NtUserDisableProcessWindowFiltering
@ stub -version=0xA00+ __win32kstub_NtUserDisableProcessWindowsGhosting
@ stub -version=0xA00+ __win32kstub_NtUserDisableThreadIme
@ stub -version=0xA00+ __win32kstub_NtUserDiscardPointerFrameMessages
@ stub -version=0xA00+ __win32kstub_NtUserDispatchMessage
@ stub -version=0xA00+ __win32kstub_NtUserDisplayConfigGetDeviceInfo
@ stub -version=0xA00+ __win32kstub_NtUserDisplayConfigSetDeviceInfo
@ stub -version=0xA00+ __win32kstub_NtUserDoInitMessagePumpHook
@ stub -version=0xA00+ __win32kstub_NtUserDoMenuOperation
@ stub -version=0xA00+ __win32kstub_NtUserDoSoundConnect
@ stub -version=0xA00+ __win32kstub_NtUserDoSoundDisconnect
@ stub -version=0xA00+ __win32kstub_NtUserDoUninitMessagePumpHook
@ stub -version=0xA00+ __win32kstub_NtUserDownlevelTouchpad
@ stub -version=0xA00+ __win32kstub_NtUserDragDetect
@ stub -version=0xA00+ __win32kstub_NtUserDragObject
@ stub -version=0xA00+ __win32kstub_NtUserDrainThreadCoreMessagingCompletions
@ stub -version=0xA00+ __win32kstub_NtUserDrainThreadCoreMessagingCompletions2
@ stub -version=0xA00+ __win32kstub_NtUserDrawAnimatedRects
@ stub -version=0xA00+ __win32kstub_NtUserDrawCaption
@ stub -version=0xA00+ __win32kstub_NtUserDrawCaptionTemp
@ stub -version=0xA00+ __win32kstub_NtUserDrawIconEx
@ stub -version=0xA00+ __win32kstub_NtUserDrawMenuBar
@ stub -version=0xA00+ __win32kstub_NtUserDrawMenuBarTemp
@ stub -version=0xA00+ __win32kstub_NtUserDwmGetRemoteSessionOcclusionEvent
@ stub -version=0xA00+ __win32kstub_NtUserDwmGetRemoteSessionOcclusionState
@ stub -version=0xA00+ __win32kstub_NtUserDwmKernelShutdown
@ stub -version=0xA00+ __win32kstub_NtUserDwmKernelStartup
@ stub -version=0xA00+ __win32kstub_NtUserDwmLockScreenUpdates
@ stub -version=0xA00+ __win32kstub_NtUserDwmValidateWindow
@ stub -version=0xA00+ __win32kstub_NtUserDwmWindowNotificationsEnabled
@ stub -version=0xA00+ __win32kstub_NtUserEmptyClipboard
@ stub -version=0xA00+ __win32kstub_NtUserEnableChildWindowDpiMessage
@ stub -version=0xA00+ __win32kstub_NtUserEnableIAMAccess
@ stub -version=0xA00+ __win32kstub_NtUserEnableMenuItem
@ stub -version=0xA00+ __win32kstub_NtUserEnableModernAppWindowKeyboardIntercept
@ stub -version=0xA00+ __win32kstub_NtUserEnableMouseInPointer
@ stub -version=0xA00+ __win32kstub_NtUserEnableMouseInPointerForThread
@ stub -version=0xA00+ __win32kstub_NtUserEnableMouseInPointerForWindow
@ stub -version=0xA00+ __win32kstub_NtUserEnableMouseInputForCursorSuppression
@ stub -version=0xA00+ __win32kstub_NtUserEnableNonClientDpiScaling
@ stub -version=0xA00+ __win32kstub_NtUserEnableResizeLayoutSynchronization
@ stub -version=0xA00+ __win32kstub_NtUserEnableScrollBar
@ stub -version=0xA00+ __win32kstub_NtUserEnableSessionForMMCSS
@ stub -version=0xA00+ __win32kstub_NtUserEnableShellWindowManagementBehavior
@ stub -version=0xA00+ __win32kstub_NtUserEnableSoftwareCursorForScreenCapture
@ stub -version=0xA00+ __win32kstub_NtUserEnableSynchronizedLayout
@ stub -version=0xA00+ __win32kstub_NtUserEnableTouchPad
@ stub -version=0xA00+ __win32kstub_NtUserEnableWindow
@ stub -version=0xA00+ __win32kstub_NtUserEnableWindowGDIScaledDpiMessage
@ stub -version=0xA00+ __win32kstub_NtUserEnableWindowResizeOptimization
@ stub -version=0xA00+ __win32kstub_NtUserEnableWindowShellWindowManagementBehavior
@ stub -version=0xA00+ __win32kstub_NtUserEndDeferWindowPosEx
@ stub -version=0xA00+ __win32kstub_NtUserEndMenu
@ stub -version=0xA00+ __win32kstub_NtUserEndPaint
@ stub -version=0xA00+ __win32kstub_NtUserEnsureDpiDepSysMetCacheForPlateau
@ stub -version=0xA00+ __win32kstub_NtUserEnsureDpiMetricsForDpi
@ stub -version=0xA00+ __win32kstub_NtUserEnsureDpiServerInfoForDpi
@ stub -version=0xA00+ __win32kstub_NtUserEnsureOemBitmapInfoForDpi
@ stub -version=0xA00+ __win32kstub_NtUserEnterMoveSizeLoop
@ stub -version=0xA00+ __win32kstub_NtUserEnumClipboardFormats
@ stub -version=0xA00+ __win32kstub_NtUserEnumDisplayDevices
@ stub -version=0xA00+ __win32kstub_NtUserEnumDisplayMonitors
@ stub -version=0xA00+ __win32kstub_NtUserEnumDisplaySettings
@ stub -version=0xA00+ __win32kstub_NtUserEvent
@ stub -version=0xA00+ __win32kstub_NtUserExcludeUpdateRgn
@ stub -version=0xA00+ __win32kstub_NtUserFillWindow
@ stub -version=0xA00+ __win32kstub_NtUserFindExistingCursorIcon
@ stub -version=0xA00+ __win32kstub_NtUserFindWindowEx
@ stub -version=0xA00+ __win32kstub_NtUserFlashWindowEx
@ stub -version=0xA00+ __win32kstub_NtUserForceEnableNumpadTranslation
@ stub -version=0xA00+ __win32kstub_NtUserForceWindowToDpiForTest
@ stub -version=0xA00+ __win32kstub_NtUserFrostCrashedWindow
@ stub -version=0xA00+ __win32kstub_NtUserFunctionalizeDisplayConfig
@ stub -version=0xA00+ __win32kstub_NtUserGetActiveProcessesDpis
@ stub -version=0xA00+ __win32kstub_NtUserGetAncestor
@ stub -version=0xA00+ __win32kstub_NtUserGetAppImeLevel
@ stub -version=0xA00+ __win32kstub_NtUserGetAsyncKeyState
@ stub -version=0xA00+ __win32kstub_NtUserGetAtomName
@ stub -version=0xA00+ __win32kstub_NtUserGetAutoRotationState
@ stub -version=0xA00+ __win32kstub_NtUserGetCIMSSM
@ stub -version=0xA00+ __win32kstub_NtUserGetCPD
@ stub -version=0xA00+ __win32kstub_NtUserGetCaretBlinkTime
@ stub -version=0xA00+ __win32kstub_NtUserGetCaretPos
@ stub -version=0xA00+ __win32kstub_NtUserGetClassIcoCur
@ stub -version=0xA00+ __win32kstub_NtUserGetClassInfoEx
@ stub -version=0xA00+ __win32kstub_NtUserGetClassName
@ stub -version=0xA00+ __win32kstub_NtUserGetClipCursor
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardAccessToken
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardData
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardFormatName
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardMetadata
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardOwner
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardSequenceNumber
@ stub -version=0xA00+ __win32kstub_NtUserGetClipboardViewer
@ stub -version=0xA00+ __win32kstub_NtUserGetComboBoxInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetControlBrush
@ stub -version=0xA00+ __win32kstub_NtUserGetControlColor
@ stub -version=0xA00+ __win32kstub_NtUserGetCurrentDpiInfoForWindow
@ stub -version=0xA00+ __win32kstub_NtUserGetCurrentInputMessageSource
@ stub -version=0xA00+ __win32kstub_NtUserGetCursor
@ stub -version=0xA00+ __win32kstub_NtUserGetCursorFrameInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetCursorInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetCursorPos
@ stub -version=0xA00+ __win32kstub_NtUserGetDC
@ stub -version=0xA00+ __win32kstub_NtUserGetDCEx
@ stub -version=0xA00+ __win32kstub_NtUserGetDCompositionHwndBitmap
@ stub -version=0xA00+ __win32kstub_NtUserGetDManipHookInitFunction
@ stub -version=0xA00+ __win32kstub_NtUserGetDesktopID
@ stub -version=0xA00+ __win32kstub_NtUserGetDesktopVisualTransform
@ stub -version=0xA00+ __win32kstub_NtUserGetDeviceChangeInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetDisplayAutoRotationPreferences
@ stub -version=0xA00+ __win32kstub_NtUserGetDisplayAutoRotationPreferencesByProcessId
@ stub -version=0xA00+ __win32kstub_NtUserGetDisplayConfigBufferSizes
@ stub -version=0xA00+ __win32kstub_NtUserGetDoubleClickTime
@ stub -version=0xA00+ __win32kstub_NtUserGetDpiForCurrentProcess
@ stub -version=0xA00+ __win32kstub_NtUserGetDpiForMonitor
@ stub -version=0xA00+ __win32kstub_NtUserGetDwmCursorShape
@ stub -version=0xA00+ __win32kstub_NtUserGetExtendedPointerDeviceProperty
@ stub -version=0xA00+ __win32kstub_NtUserGetForegroundWindow
@ stub -version=0xA00+ __win32kstub_NtUserGetGUIThreadInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetGestureConfig
@ stub -version=0xA00+ __win32kstub_NtUserGetGestureExtArgs
@ stub -version=0xA00+ __win32kstub_NtUserGetGestureInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetGuiResources
@ stub -version=0xA00+ __win32kstub_NtUserGetHDevName
@ stub -version=0xA00+ __win32kstub_NtUserGetHimetricScaleFactorFromPixelLocation
@ stub -version=0xA00+ __win32kstub_NtUserGetIMEShowStatus
@ stub -version=0xA00+ __win32kstub_NtUserGetIconInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetIconSize
@ stub -version=0xA00+ __win32kstub_NtUserGetImeHotKey
@ stub -version=0xA00+ __win32kstub_NtUserGetImeInfoEx
@ stub -version=0xA00+ __win32kstub_NtUserGetInputContainerId
@ stub -version=0xA00+ __win32kstub_NtUserGetInputDesktop
@ stub -version=0xA00+ __win32kstub_NtUserGetInputEvent
@ stub -version=0xA00+ __win32kstub_NtUserGetInputLocaleInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetInteractiveControlDeviceInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetInteractiveControlInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetInteractiveCtrlSupportedWaveforms
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyNameText
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyState
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyboardInputThreadId
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyboardLayout
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyboardLayoutList
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyboardLayoutName
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyboardState
@ stub -version=0xA00+ __win32kstub_NtUserGetKeyboardType
@ stub -version=0xA00+ __win32kstub_NtUserGetLatestInputMessageData
@ stub -version=0xA00+ __win32kstub_NtUserGetLayeredWindowAttributes
@ stub -version=0xA00+ __win32kstub_NtUserGetListBoxInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetMenuBarInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetMenuIndex
@ stub -version=0xA00+ __win32kstub_NtUserGetMenuItemRect
@ stub -version=0xA00+ __win32kstub_NtUserGetMessage
@ stub -version=0xA00+ __win32kstub_NtUserGetMessagePos
@ stub -version=0xA00+ __win32kstub_NtUserGetMinuserIdForBaseWindow
@ stub -version=0xA00+ __win32kstub_NtUserGetModernAppWindow
@ stub -version=0xA00+ __win32kstub_NtUserGetMouseMovePointsEx
@ stub -version=0xA00+ __win32kstub_NtUserGetObjectInformation
@ stub -version=0xA00+ __win32kstub_NtUserGetOemBitmapSize
@ stub -version=0xA00+ __win32kstub_NtUserGetOpenClipboardWindow
@ stub -version=0xA00+ __win32kstub_NtUserGetOwnerTransformedMonitorRect
@ stub -version=0xA00+ __win32kstub_NtUserGetPhysicalDeviceRect
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerCursorId
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDevice
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDeviceCursors
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDeviceInputSpace
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDeviceOrientation
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDeviceProperties
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDeviceRects
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerDevices
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerFrameTimes
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerIdForPromotion
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerInfoList
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerInputTransform
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerProprietaryId
@ stub -version=0xA00+ __win32kstub_NtUserGetPointerType
@ stub -version=0xA00+ __win32kstub_NtUserGetPriorityClipboardFormat
@ stub -version=0xA00+ __win32kstub_NtUserGetProcessDefaultLayout
@ stub -version=0xA00+ __win32kstub_NtUserGetProcessDpiAwarenessContext
@ stub -version=0xA00+ __win32kstub_NtUserGetProcessUIContextInformation
@ stub -version=0xA00+ __win32kstub_NtUserGetProcessWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserGetProp
@ stub -version=0xA00+ __win32kstub_NtUserGetProp2
@ stub -version=0xA00+ __win32kstub_NtUserGetQueueIocp
@ stub -version=0xA00+ __win32kstub_NtUserGetQueueStatus
@ stub -version=0xA00+ __win32kstub_NtUserGetQueueStatusReadonly
@ stub -version=0xA00+ __win32kstub_NtUserGetRawInputBuffer
@ stub -version=0xA00+ __win32kstub_NtUserGetRawInputData
@ stub -version=0xA00+ __win32kstub_NtUserGetRawInputDeviceInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetRawInputDeviceList
@ stub -version=0xA00+ __win32kstub_NtUserGetRawPointerDeviceData
@ stub -version=0xA00+ __win32kstub_NtUserGetRegisteredRawInputDevices
@ stub -version=0xA00+ __win32kstub_NtUserGetRequiredCursorSizes
@ stub -version=0xA00+ __win32kstub_NtUserGetResizeDCompositionSynchronizationObject
@ stub -version=0xA00+ __win32kstub_NtUserGetScrollBarInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetSendMessageReceiver
@ stub -version=0xA00+ __win32kstub_NtUserGetSharedWindowData
@ stub -version=0xA00+ __win32kstub_NtUserGetSysMenuOffset
@ stub -version=0xA00+ __win32kstub_NtUserGetSystemContentRects
@ stub -version=0xA00+ __win32kstub_NtUserGetSystemDpiForProcess
@ stub -version=0xA00+ __win32kstub_NtUserGetSystemMenu
@ stub -version=0xA00+ __win32kstub_NtUserGetThreadDesktop
@ stub -version=0xA00+ __win32kstub_NtUserGetThreadState
@ stub -version=0xA00+ __win32kstub_NtUserGetTitleBarInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetTopLevelWindow
@ stub -version=0xA00+ __win32kstub_NtUserGetTouchInputInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetTouchValidationStatus
@ stub -version=0xA00+ __win32kstub_NtUserGetUniformSpaceMapping
@ stub -version=0xA00+ __win32kstub_NtUserGetUnpredictedMessagePos
@ stub -version=0xA00+ __win32kstub_NtUserGetUpdateRect
@ stub -version=0xA00+ __win32kstub_NtUserGetUpdateRgn
@ stub -version=0xA00+ __win32kstub_NtUserGetUpdatedClipboardFormats
@ stub -version=0xA00+ __win32kstub_NtUserGetWOWClass
@ stub -version=0xA00+ __win32kstub_NtUserGetWinStationInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowBand
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowCompositionAttribute
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowCompositionInfo
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowContextHelpId
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowDC
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowDisplayAffinity
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowFeedbackSetting
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowMinimizeRect
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowPlacement
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowProcessHandle
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowRgnEx
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowThreadProcessId
@ stub -version=0xA00+ __win32kstub_NtUserGetWindowTrackInfoAsync
@ stub -version=0xA00+ __win32kstub_NtUserGhostWindowFromHungWindow
@ stub -version=0xA00+ __win32kstub_NtUserHandleDelegatedInput
@ stub -version=0xA00+ __win32kstub_NtUserHandleSystemThreadCreationFailure
@ stub -version=0xA00+ __win32kstub_NtUserHardErrorControl
@ stub -version=0xA00+ __win32kstub_NtUserHideCaret
@ stub -version=0xA00+ __win32kstub_NtUserHideCursorNoCapture
@ stub -version=0xA00+ __win32kstub_NtUserHidePointerContactVisualization
@ stub -version=0xA00+ __win32kstub_NtUserHiliteMenuItem
@ stub -version=0xA00+ __win32kstub_NtUserHungWindowFromGhostWindow
@ stub -version=0xA00+ __win32kstub_NtUserHwndQueryRedirectionInfo
@ stub -version=0xA00+ __win32kstub_NtUserHwndSetRedirectionInfo
@ stub -version=0xA00+ __win32kstub_NtUserImpersonateDdeClientWindow
@ stub -version=0xA00+ __win32kstub_NtUserInheritWindowMonitor
@ stub -version=0xA00+ __win32kstub_NtUserInitAnsiOem
@ stub -version=0xA00+ __win32kstub_NtUserInitThreadCoreMessagingIocp
@ stub -version=0xA00+ __win32kstub_NtUserInitThreadCoreMessagingIocp2
@ stub -version=0xA00+ __win32kstub_NtUserInitialize
@ stub -version=0xA00+ __win32kstub_NtUserInitializeClientPfnArrays
@ stub -version=0xA00+ __win32kstub_NtUserInitializeGenericHidInjection
@ stub -version=0xA00+ __win32kstub_NtUserInitializeInputDeviceInjection
@ stub -version=0xA00+ __win32kstub_NtUserInitializeTouchInjection
@ stub -version=0xA00+ __win32kstub_NtUserInitializeUserModeMenus
@ stub -version=0xA00+ __win32kstub_NtUserInjectDeviceInput
@ stub -version=0xA00+ __win32kstub_NtUserInjectGenericHidInput
@ stub -version=0xA00+ __win32kstub_NtUserInjectGesture
@ stub -version=0xA00+ __win32kstub_NtUserInjectKeyboardInput
@ stub -version=0xA00+ __win32kstub_NtUserInjectMouseInput
@ stub -version=0xA00+ __win32kstub_NtUserInjectPointerInput
@ stub -version=0xA00+ __win32kstub_NtUserInjectTouchInput
@ stub -version=0xA00+ __win32kstub_NtUserInjectTouchpadAction
@ stub -version=0xA00+ __win32kstub_NtUserInteractiveControlQueryUsage
@ stub -version=0xA00+ __win32kstub_NtUserInternalGetWindowIcon
@ stub -version=0xA00+ __win32kstub_NtUserInternalGetWindowText
@ stub -version=0xA00+ __win32kstub_NtUserInternalToUnicode
@ stub -version=0xA00+ __win32kstub_NtUserInvalidateRect
@ stub -version=0xA00+ __win32kstub_NtUserInvalidateRgn
@ stub -version=0xA00+ __win32kstub_NtUserIsChildWindowDpiMessageEnabled
@ stub -version=0xA00+ __win32kstub_NtUserIsClipboardFormatAvailable
@ stub -version=0xA00+ __win32kstub_NtUserIsInterceptWindow
@ stub -version=0xA00+ __win32kstub_NtUserIsMouseInPointerEnabled
@ stub -version=0xA00+ __win32kstub_NtUserIsMouseInputEnabled
@ stub -version=0xA00+ __win32kstub_NtUserIsNonClientDpiScalingEnabled
@ stub -version=0xA00+ __win32kstub_NtUserIsResizeLayoutSynchronizationEnabled
@ stub -version=0xA00+ __win32kstub_NtUserIsTopLevelWindow
@ stub -version=0xA00+ __win32kstub_NtUserIsTouchWindow
@ stub -version=0xA00+ __win32kstub_NtUserIsUtilitySession
@ stub -version=0xA00+ __win32kstub_NtUserIsWindowBroadcastingDpiToChildren
@ stub -version=0xA00+ __win32kstub_NtUserIsWindowDisplayChangeSuppressed
@ stub -version=0xA00+ __win32kstub_NtUserIsWindowGDIScaledDpiMessageEnabled
@ stub -version=0xA00+ __win32kstub_NtUserKillSystemTimer
@ stub -version=0xA00+ __win32kstub_NtUserKillTimer
@ stub -version=0xA00+ __win32kstub_NtUserLW_LoadFonts
@ stub -version=0xA00+ __win32kstub_NtUserLayoutCompleted
@ stub -version=0xA00+ __win32kstub_NtUserLinkDpiCursor
@ stub -version=0xA00+ __win32kstub_NtUserLoadCursorsAndIcons
@ stub -version=0xA00+ __win32kstub_NtUserLoadKeyboardLayoutEx
@ stub -version=0xA00+ __win32kstub_NtUserLoadUserApiHook
@ stub -version=0xA00+ __win32kstub_NtUserLockCursor
@ stub -version=0xA00+ __win32kstub_NtUserLockSetForegroundWindow
@ stub -version=0xA00+ __win32kstub_NtUserLockWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserLockWindowUpdate
@ stub -version=0xA00+ __win32kstub_NtUserLockWorkStation
@ stub -version=0xA00+ __win32kstub_NtUserLogicalToPerMonitorDPIPhysicalPoint
@ stub -version=0xA00+ __win32kstub_NtUserLogicalToPhysicalDpiPointForWindow
@ stub -version=0xA00+ __win32kstub_NtUserLogicalToPhysicalPoint
@ stub -version=0xA00+ __win32kstub_NtUserMNDragLeave
@ stub -version=0xA00+ __win32kstub_NtUserMNDragOver
@ stub -version=0xA00+ __win32kstub_NtUserMagControl
@ stub -version=0xA00+ __win32kstub_NtUserMagGetContextInformation
@ stub -version=0xA00+ __win32kstub_NtUserMagSetContextInformation
@ stub -version=0xA00+ __win32kstub_NtUserMapDesktopObject
@ stub -version=0xA00+ __win32kstub_NtUserMapPointsByVisualIdentifier
@ stub -version=0xA00+ __win32kstub_NtUserMapVirtualKeyEx
@ stub -version=0xA00+ __win32kstub_NtUserMarkWindowForRawMouse
@ stub -version=0xA00+ __win32kstub_NtUserMenuItemFromPoint
@ stub -version=0xA00+ __win32kstub_NtUserMessageBeep
@ stub -version=0xA00+ __win32kstub_NtUserMessageCall
@ stub -version=0xA00+ __win32kstub_NtUserMinInitialize
@ stub -version=0xA00+ __win32kstub_NtUserMinMaximize
@ stub -version=0xA00+ __win32kstub_NtUserModifyUserStartupInfoFlags
@ stub -version=0xA00+ __win32kstub_NtUserModifyWindowTouchCapability
@ stub -version=0xA00+ __win32kstub_NtUserMoveWindow
@ stub -version=0xA00+ __win32kstub_NtUserMsgWaitForMultipleObjectsEx
@ stub -version=0xA00+ __win32kstub_NtUserNavigateFocus
@ stub -version=0xA00+ __win32kstub_NtUserNlsKbdSendIMENotification
@ stub -version=0xA00+ __win32kstub_NtUserNotifyIMEStatus
@ stub -version=0xA00+ __win32kstub_NtUserNotifyOverlayWindow
@ stub -version=0xA00+ __win32kstub_NtUserNotifyProcessCreate
@ stub -version=0xA00+ __win32kstub_NtUserNotifyWinEvent
@ stub -version=0xA00+ __win32kstub_NtUserOpenClipboard
@ stub -version=0xA00+ __win32kstub_NtUserOpenDesktop
@ stub -version=0xA00+ __win32kstub_NtUserOpenInputDesktop
@ stub -version=0xA00+ __win32kstub_NtUserOpenThreadDesktop
@ stub -version=0xA00+ __win32kstub_NtUserOpenWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserPaintDesktop
@ stub -version=0xA00+ __win32kstub_NtUserPaintMenuBar
@ stub -version=0xA00+ __win32kstub_NtUserPaintMonitor
@ stub -version=0xA00+ __win32kstub_NtUserPeekMessage
@ stub -version=0xA00+ __win32kstub_NtUserPerMonitorDPIPhysicalToLogicalPoint
@ stub -version=0xA00+ __win32kstub_NtUserPhysicalToLogicalDpiPointForWindow
@ stub -version=0xA00+ __win32kstub_NtUserPhysicalToLogicalPoint
@ stub -version=0xA00+ __win32kstub_NtUserPlayEventSound
@ stub -version=0xA00+ __win32kstub_NtUserPostKeyboardInputMessage
@ stub -version=0xA00+ __win32kstub_NtUserPostMessage
@ stub -version=0xA00+ __win32kstub_NtUserPostQuitMessage
@ stub -version=0xA00+ __win32kstub_NtUserPostThreadMessage
@ stub -version=0xA00+ __win32kstub_NtUserPrepareForLogoff
@ stub -version=0xA00+ __win32kstub_NtUserPrintWindow
@ stub -version=0xA00+ __win32kstub_NtUserProcessConnect
@ stub -version=0xA00+ __win32kstub_NtUserProcessInkFeedbackCommand
@ stub -version=0xA00+ __win32kstub_NtUserPromoteMouseInPointer
@ stub -version=0xA00+ __win32kstub_NtUserPromotePointer
@ stub -version=0xA00+ __win32kstub_NtUserQueryBSDRWindow
@ stub -version=0xA00+ __win32kstub_NtUserQueryDisplayConfig
@ stub -version=0xA00+ __win32kstub_NtUserQueryInformationThread
@ stub -version=0xA00+ __win32kstub_NtUserQueryInputContext
@ stub -version=0xA00+ __win32kstub_NtUserQueryWindow
@ stub -version=0xA00+ __win32kstub_NtUserRaiseLowerShellWindow
@ stub -version=0xA00+ __win32kstub_NtUserRealChildWindowFromPoint
@ stub -version=0xA00+ __win32kstub_NtUserRealInternalGetMessage
@ stub -version=0xA00+ __win32kstub_NtUserRealWaitMessageEx
@ stub -version=0xA00+ __win32kstub_NtUserRealizePalette
@ stub -version=0xA00+ __win32kstub_NtUserReassociateQueueEventCompletionPacket
@ stub -version=0xA00+ __win32kstub_NtUserRedrawFrame
@ stub -version=0xA00+ __win32kstub_NtUserRedrawFrameAndHook
@ stub -version=0xA00+ __win32kstub_NtUserRedrawTitle
@ stub -version=0xA00+ __win32kstub_NtUserRedrawWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterBSDRWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterClassExWOW
@ stub -version=0xA00+ __win32kstub_NtUserRegisterCloakedNotification
@ stub -version=0xA00+ __win32kstub_NtUserRegisterCoreMessagingEndPoint
@ stub -version=0xA00+ __win32kstub_NtUserRegisterDManipHook
@ stub -version=0xA00+ __win32kstub_NtUserRegisterEdgy
@ stub -version=0xA00+ __win32kstub_NtUserRegisterErrorReportingDialog
@ stub -version=0xA00+ __win32kstub_NtUserRegisterForCustomDockTargets
@ stub -version=0xA00+ __win32kstub_NtUserRegisterForTooltipDismissNotification
@ stub -version=0xA00+ __win32kstub_NtUserRegisterGhostWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterHotKey
@ stub -version=0xA00+ __win32kstub_NtUserRegisterLPK
@ stub -version=0xA00+ __win32kstub_NtUserRegisterLogonProcess
@ stub -version=0xA00+ __win32kstub_NtUserRegisterManipulationThread
@ stub -version=0xA00+ __win32kstub_NtUserRegisterPointerDeviceNotifications
@ stub -version=0xA00+ __win32kstub_NtUserRegisterPointerInputTarget
@ stub -version=0xA00+ __win32kstub_NtUserRegisterRawInputDevices
@ stub -version=0xA00+ __win32kstub_NtUserRegisterServicesProcess
@ stub -version=0xA00+ __win32kstub_NtUserRegisterSessionPort
@ stub -version=0xA00+ __win32kstub_NtUserRegisterShellHookWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterSiblingFrostWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterSystemThread
@ stub -version=0xA00+ __win32kstub_NtUserRegisterTasklist
@ stub -version=0xA00+ __win32kstub_NtUserRegisterTouchHitTestingWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterTouchPadCapable
@ stub -version=0xA00+ __win32kstub_NtUserRegisterTouchpadCapableWindow
@ stub -version=0xA00+ __win32kstub_NtUserRegisterUserApiHook
@ stub -version=0xA00+ __win32kstub_NtUserRegisterWindowArrangementCallout
@ stub -version=0xA00+ __win32kstub_NtUserRegisterWindowMessage
@ stub -version=0xA00+ __win32kstub_NtUserReleaseCapture
@ stub -version=0xA00+ __win32kstub_NtUserReleaseDC
@ stub -version=0xA00+ __win32kstub_NtUserReleaseDwmHitTestWaiters
@ stub -version=0xA00+ __win32kstub_NtUserRemoteConnect
@ stub -version=0xA00+ __win32kstub_NtUserRemoteConnectState
@ stub -version=0xA00+ __win32kstub_NtUserRemoteConsoleShadowStop
@ stub -version=0xA00+ __win32kstub_NtUserRemoteDisconnect
@ stub -version=0xA00+ __win32kstub_NtUserRemoteNotify
@ stub -version=0xA00+ __win32kstub_NtUserRemotePassthruDisable
@ stub -version=0xA00+ __win32kstub_NtUserRemotePassthruEnable
@ stub -version=0xA00+ __win32kstub_NtUserRemoteReconnect
@ stub -version=0xA00+ __win32kstub_NtUserRemoteRedrawRectangle
@ stub -version=0xA00+ __win32kstub_NtUserRemoteRedrawScreen
@ stub -version=0xA00+ __win32kstub_NtUserRemoteShadowCleanup
@ stub -version=0xA00+ __win32kstub_NtUserRemoteShadowSetup
@ stub -version=0xA00+ __win32kstub_NtUserRemoteShadowStart
@ stub -version=0xA00+ __win32kstub_NtUserRemoteShadowStop
@ stub -version=0xA00+ __win32kstub_NtUserRemoteStopScreenUpdates
@ stub -version=0xA00+ __win32kstub_NtUserRemoteThinwireStats
@ stub -version=0xA00+ __win32kstub_NtUserRemoveClipboardFormatListener
@ stub -version=0xA00+ __win32kstub_NtUserRemoveInjectionDevice
@ stub -version=0xA00+ __win32kstub_NtUserRemoveMenu
@ stub -version=0xA00+ __win32kstub_NtUserRemoveProp
@ stub -version=0xA00+ __win32kstub_NtUserRemoveQueueCompletion
@ stub -version=0xA00+ __win32kstub_NtUserRemoveVisualIdentifier
@ stub -version=0xA00+ __win32kstub_NtUserReplyMessage
@ stub -version=0xA00+ __win32kstub_NtUserReportInertia
@ stub -version=0xA00+ __win32kstub_NtUserRequestLowLatencyProfile
@ stub -version=0xA00+ __win32kstub_NtUserResetDblClk
@ stub -version=0xA00+ __win32kstub_NtUserResolveDesktopForWOW
@ stub -version=0xA00+ __win32kstub_NtUserRestoreWindowDpiChanges
@ stub -version=0xA00+ __win32kstub_NtUserSBGetParms
@ stub -version=0xA00+ __win32kstub_NtUserScaleSystemMetricForDPIWithoutCache
@ stub -version=0xA00+ __win32kstub_NtUserScheduleDispatchNotification
@ stub -version=0xA00+ __win32kstub_NtUserScrollDC
@ stub -version=0xA00+ __win32kstub_NtUserScrollWindowEx
@ stub -version=0xA00+ __win32kstub_NtUserSelectPalette
@ stub -version=0xA00+ __win32kstub_NtUserSendEventMessage
@ stub -version=0xA00+ __win32kstub_NtUserSendInput
@ stub -version=0xA00+ __win32kstub_NtUserSendInteractiveControlHapticsReport
@ stub -version=0xA00+ __win32kstub_NtUserSetActivationFilter
@ stub -version=0xA00+ __win32kstub_NtUserSetActiveProcessForMonitor
@ stub -version=0xA00+ __win32kstub_NtUserSetActiveWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetAdditionalForegroundBoostProcesses
@ stub -version=0xA00+ __win32kstub_NtUserSetAdditionalPowerThrottlingProcess
@ stub -version=0xA00+ __win32kstub_NtUserSetAppImeLevel
@ stub -version=0xA00+ __win32kstub_NtUserSetAutoRotation
@ stub -version=0xA00+ __win32kstub_NtUserSetBridgeWindowChild
@ stub -version=0xA00+ __win32kstub_NtUserSetBrokeredForeground
@ stub -version=0xA00+ __win32kstub_NtUserSetCalibrationData
@ stub -version=0xA00+ __win32kstub_NtUserSetCancelRotationDelayHintWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetCapture
@ stub -version=0xA00+ __win32kstub_NtUserSetCaretBlinkTime
@ stub -version=0xA00+ __win32kstub_NtUserSetCaretPos
@ stub -version=0xA00+ __win32kstub_NtUserSetChildWindowNoActivate
@ stub -version=0xA00+ __win32kstub_NtUserSetClassLong
@ stub -version=0xA00+ __win32kstub_NtUserSetClassLongPtr
@ stub -version=0xA00+ __win32kstub_NtUserSetClassWord
@ stub -version=0xA00+ __win32kstub_NtUserSetClipboardData
@ stub -version=0xA00+ __win32kstub_NtUserSetClipboardViewer
@ stub -version=0xA00+ __win32kstub_NtUserSetCoreWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetCoreWindowPartner
@ stub -version=0xA00+ __win32kstub_NtUserSetCoveredWindowStates
@ stub -version=0xA00+ __win32kstub_NtUserSetCursor
@ stub -version=0xA00+ __win32kstub_NtUserSetCursorIconData
@ stub -version=0xA00+ __win32kstub_NtUserSetCursorIconDataEx
@ stub -version=0xA00+ __win32kstub_NtUserSetCursorPos
@ stub -version=0xA00+ __win32kstub_NtUserSetDesktopColorTransform
@ stub -version=0xA00+ __win32kstub_NtUserSetDesktopVisualInputSink
@ stub -version=0xA00+ __win32kstub_NtUserSetDialogControlDpiChangeBehavior
@ stub -version=0xA00+ __win32kstub_NtUserSetDialogPointer
@ stub -version=0xA00+ __win32kstub_NtUserSetDialogSystemMenu
@ stub -version=0xA00+ __win32kstub_NtUserSetDisplayAutoRotationPreferences
@ stub -version=0xA00+ __win32kstub_NtUserSetDisplayConfig
@ stub -version=0xA00+ __win32kstub_NtUserSetDisplayMapping
@ stub -version=0xA00+ __win32kstub_NtUserSetDoubleClickTime
@ stub -version=0xA00+ __win32kstub_NtUserSetDpiForWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetFeatureReportResponse
@ stub -version=0xA00+ __win32kstub_NtUserSetFocus
@ stub -version=0xA00+ __win32kstub_NtUserSetForegroundRedirectionForActivationObject
@ stub -version=0xA00+ __win32kstub_NtUserSetForegroundWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetForegroundWindowForApplication
@ stub -version=0xA00+ __win32kstub_NtUserSetFullscreenMagnifierOffsetsDWMUpdated
@ stub -version=0xA00+ __win32kstub_NtUserSetGUISystemInformation
@ stub -version=0xA00+ __win32kstub_NtUserSetGestureConfig
@ stub -version=0xA00+ __win32kstub_NtUserSetImeHotKey
@ stub -version=0xA00+ __win32kstub_NtUserSetImeInfoEx
@ stub -version=0xA00+ __win32kstub_NtUserSetImeOwnerWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetInformationThread
@ stub -version=0xA00+ __win32kstub_NtUserSetInputServiceState
@ stub -version=0xA00+ __win32kstub_NtUserSetInteractiveControlFocus
@ stub -version=0xA00+ __win32kstub_NtUserSetInteractiveCtrlRotationAngle
@ stub -version=0xA00+ __win32kstub_NtUserSetInternalWindowPos
@ stub -version=0xA00+ __win32kstub_NtUserSetJobUILimits
@ stub -version=0xA00+ __win32kstub_NtUserSetKeyboardState
@ stub -version=0xA00+ __win32kstub_NtUserSetLayeredWindowAttributes
@ stub -version=0xA00+ __win32kstub_NtUserSetMagnificationDesktopMagnifierOffsetsDWMUpdated
@ stub -version=0xA00+ __win32kstub_NtUserSetManipulationInputTarget
@ stub -version=0xA00+ __win32kstub_NtUserSetMaxTouchpadSensitivity
@ stub -version=0xA00+ __win32kstub_NtUserSetMenu
@ stub -version=0xA00+ __win32kstub_NtUserSetMenuContextHelpId
@ stub -version=0xA00+ __win32kstub_NtUserSetMenuDefaultItem
@ stub -version=0xA00+ __win32kstub_NtUserSetMenuFlagRtoL
@ stub -version=0xA00+ __win32kstub_NtUserSetMessageExtraInfo
@ stub -version=0xA00+ __win32kstub_NtUserSetMirrorRendering
@ stub -version=0xA00+ __win32kstub_NtUserSetModernAppWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetMonitorWorkArea
@ stub -version=0xA00+ __win32kstub_NtUserSetMsgBox
@ stub -version=0xA00+ __win32kstub_NtUserSetObjectInformation
@ stub -version=0xA00+ __win32kstub_NtUserSetParent
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessDefaultLayout
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessDpiAwarenessContext
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessInteractionFlags
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessLaunchForegroundPolicy
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessMousewheelRoutingMode
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessRestrictionExemption
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessUIAccessZorder
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessWin32Capabilities
@ stub -version=0xA00+ __win32kstub_NtUserSetProcessWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserSetProgmanWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetProp
@ stub -version=0xA00+ __win32kstub_NtUserSetProp2
@ stub -version=0xA00+ __win32kstub_NtUserSetScrollInfo
@ stub -version=0xA00+ __win32kstub_NtUserSetSensorPresence
@ stub -version=0xA00+ __win32kstub_NtUserSetSharedWindowData
@ stub -version=0xA00+ __win32kstub_NtUserSetShellChangeNotifyHWND
@ stub -version=0xA00+ __win32kstub_NtUserSetShellSpecialWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetShellWindowEx
@ stub -version=0xA00+ __win32kstub_NtUserSetSysColors
@ stub -version=0xA00+ __win32kstub_NtUserSetSysMenu
@ stub -version=0xA00+ __win32kstub_NtUserSetSystemContentRects
@ stub -version=0xA00+ __win32kstub_NtUserSetSystemCursor
@ stub -version=0xA00+ __win32kstub_NtUserSetSystemMenu
@ stub -version=0xA00+ __win32kstub_NtUserSetSystemTimer
@ stub -version=0xA00+ __win32kstub_NtUserSetTSFEventState
@ stub -version=0xA00+ __win32kstub_NtUserSetTargetForResourceBrokering
@ stub -version=0xA00+ __win32kstub_NtUserSetTaskmanWindow
@ stub -version=0xA00+ __win32kstub_NtUserSetThreadDesktop
@ stub -version=0xA00+ __win32kstub_NtUserSetThreadInputBlocked
@ stub -version=0xA00+ __win32kstub_NtUserSetThreadLayoutHandles
@ stub -version=0xA00+ __win32kstub_NtUserSetThreadQueueMergeSetting
@ stub -version=0xA00+ __win32kstub_NtUserSetThreadState
@ stub -version=0xA00+ __win32kstub_NtUserSetTimer
@ stub -version=0xA00+ __win32kstub_NtUserSetUserObjectCapability
@ stub -version=0xA00+ __win32kstub_NtUserSetVisible
@ stub -version=0xA00+ __win32kstub_NtUserSetWaitForQueueAttach
@ stub -version=0xA00+ __win32kstub_NtUserSetWatermarkStrings
@ stub -version=0xA00+ __win32kstub_NtUserSetWinEventHook
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowBand
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowCompositionAttribute
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowCompositionTransition
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowContextHelpId
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowDisplayAffinity
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowFNID
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowFeedbackSetting
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowLong
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowLongPtr
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowMessageCapability
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowPlacement
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowPos
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowRgn
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowRgnEx
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowShowState
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowState
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowStationUser
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowWord
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowsHookAW
@ stub -version=0xA00+ __win32kstub_NtUserSetWindowsHookEx
@ stub -version=0xA00+ __win32kstub_NtUserShellForegroundBoostProcess
@ stub -version=0xA00+ __win32kstub_NtUserShellHandwritingDelegateInput
@ stub -version=0xA00+ __win32kstub_NtUserShellHandwritingHandleDelegatedInput
@ stub -version=0xA00+ __win32kstub_NtUserShellHandwritingUndelegateInput
@ stub -version=0xA00+ __win32kstub_NtUserShellMigrateWindow
@ stub -version=0xA00+ __win32kstub_NtUserShellRegisterHotKey
@ stub -version=0xA00+ __win32kstub_NtUserShellSetWindowPos
@ stub -version=0xA00+ __win32kstub_NtUserShowCaret
@ stub -version=0xA00+ __win32kstub_NtUserShowCursor
@ stub -version=0xA00+ __win32kstub_NtUserShowOwnedPopups
@ stub -version=0xA00+ __win32kstub_NtUserShowScrollBar
@ stub -version=0xA00+ __win32kstub_NtUserShowStartGlass
@ stub -version=0xA00+ __win32kstub_NtUserShowSystemCursor
@ stub -version=0xA00+ __win32kstub_NtUserShowWindow
@ stub -version=0xA00+ __win32kstub_NtUserShowWindowAsync
@ stub -version=0xA00+ __win32kstub_NtUserShutdownBlockReasonCreate
@ stub -version=0xA00+ __win32kstub_NtUserShutdownBlockReasonQuery
@ stub -version=0xA00+ __win32kstub_NtUserShutdownReasonDestroy
@ stub -version=0xA00+ __win32kstub_NtUserSignalRedirectionStartComplete
@ stub -version=0xA00+ __win32kstub_NtUserSlicerControl
@ stub -version=0xA00+ __win32kstub_NtUserSoundSentry
@ stub -version=0xA00+ __win32kstub_NtUserStopAndEndInertia
@ stub -version=0xA00+ __win32kstub_NtUserSuppressWindowDisplayChange
@ stub -version=0xA00+ __win32kstub_NtUserSwapMouseButton
@ stub -version=0xA00+ __win32kstub_NtUserSwitchDesktop
@ stub -version=0xA00+ __win32kstub_NtUserSwitchToThisWindow
@ stub -version=0xA00+ __win32kstub_NtUserSystemParametersInfo
@ stub -version=0xA00+ __win32kstub_NtUserSystemParametersInfoForDpi
@ stub -version=0xA00+ __win32kstub_NtUserTestForInteractiveUser
@ stub -version=0xA00+ __win32kstub_NtUserThreadMessageQueueAttached
@ stub -version=0xA00+ __win32kstub_NtUserThunkedMenuInfo
@ stub -version=0xA00+ __win32kstub_NtUserThunkedMenuItemInfo
@ stub -version=0xA00+ __win32kstub_NtUserToUnicodeEx
@ stub -version=0xA00+ __win32kstub_NtUserTraceLoggingSendMixedModeTelemetry
@ stub -version=0xA00+ __win32kstub_NtUserTrackMouseEvent
@ stub -version=0xA00+ __win32kstub_NtUserTrackPopupMenuEx
@ stub -version=0xA00+ __win32kstub_NtUserTransformPoint
@ stub -version=0xA00+ __win32kstub_NtUserTransformRect
@ stub -version=0xA00+ __win32kstub_NtUserTranslateAccelerator
@ stub -version=0xA00+ __win32kstub_NtUserTranslateMessage
@ stub -version=0xA00+ __win32kstub_NtUserUndelegateInput
@ stub -version=0xA00+ __win32kstub_NtUserUnhookWinEvent
@ stub -version=0xA00+ __win32kstub_NtUserUnhookWindowsHook
@ stub -version=0xA00+ __win32kstub_NtUserUnhookWindowsHookEx
@ stub -version=0xA00+ __win32kstub_NtUserUnloadKeyboardLayout
@ stub -version=0xA00+ __win32kstub_NtUserUnlockWindowStation
@ stub -version=0xA00+ __win32kstub_NtUserUnregisterClass
@ stub -version=0xA00+ __win32kstub_NtUserUnregisterHotKey
@ stub -version=0xA00+ __win32kstub_NtUserUnregisterSessionPort
@ stub -version=0xA00+ __win32kstub_NtUserUnregisterUserApiHook
@ stub -version=0xA00+ __win32kstub_NtUserUpdateClientRect
@ stub -version=0xA00+ __win32kstub_NtUserUpdateDefaultDesktopThumbnail
@ stub -version=0xA00+ __win32kstub_NtUserUpdateInputContext
@ stub -version=0xA00+ __win32kstub_NtUserUpdateInstance
@ stub -version=0xA00+ __win32kstub_NtUserUpdateLayeredWindow
@ stub -version=0xA00+ __win32kstub_NtUserUpdatePerUserImmEnabling
@ stub -version=0xA00+ __win32kstub_NtUserUpdatePerUserSystemParameters
@ stub -version=0xA00+ __win32kstub_NtUserUpdateWindow
@ stub -version=0xA00+ __win32kstub_NtUserUpdateWindowInputSinkHints
@ stub -version=0xA00+ __win32kstub_NtUserUpdateWindowTrackingInfo
@ stub -version=0xA00+ __win32kstub_NtUserUpdateWindows
@ stub -version=0xA00+ __win32kstub_NtUserUserHandleGrantAccess
@ stub -version=0xA00+ __win32kstub_NtUserUserPowerCalloutWorker
@ stub -version=0xA00+ __win32kstub_NtUserValidateHandleSecure
@ stub -version=0xA00+ __win32kstub_NtUserValidateRect
@ stub -version=0xA00+ __win32kstub_NtUserValidateRgn
@ stub -version=0xA00+ __win32kstub_NtUserValidateTimerCallback
@ stub -version=0xA00+ __win32kstub_NtUserVkKeyScanEx
@ stub -version=0xA00+ __win32kstub_NtUserWaitAvailableMessageEx
@ stub -version=0xA00+ __win32kstub_NtUserWaitForInputIdle
@ stub -version=0xA00+ __win32kstub_NtUserWaitForRedirectionStartComplete
@ stub -version=0xA00+ __win32kstub_NtUserWaitMessage
@ stub -version=0xA00+ __win32kstub_NtUserWakeRITForShutdown
@ stub -version=0xA00+ __win32kstub_NtUserWindowFromDC
@ stub -version=0xA00+ __win32kstub_NtUserWindowFromPhysicalPoint
@ stub -version=0xA00+ __win32kstub_NtUserWindowFromPoint
@ stub -version=0xA00+ __win32kstub_NtUserZapActiveAndFocus
@ stub -version=0xA00+ __win32kstub_NtValidateCompositionSurfaceHandle
@ stub -version=0xA00+ __win32kstub_NtVisualCaptureBits
# Data exports (need an `extern`, not a code stub):
#   W32pArgumentTable
#   W32pArgumentTableFilter
#   W32pServiceLimit
#   W32pServiceLimitFilter
#   W32pServiceTable
#   W32pServiceTableFilter
#   ext_ms_win_moderncore_win32k_base_sysentry_l1_table
#   ext_ms_win_moderncore_win32k_base_sysentry_l1_table_size
#   gSessionApiSetHostRefCount
#   gSessionApiSetHostRefCountLock
# END win10-surface stubs
