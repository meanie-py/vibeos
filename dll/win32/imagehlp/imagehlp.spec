@ stdcall BindImage(str str str)
@ stdcall BindImageEx(long str str str ptr)
@ stdcall CheckSumMappedFile(ptr long ptr ptr)
@ stdcall -import EnumerateLoadedModules64(long ptr ptr)
@ stdcall -import EnumerateLoadedModules(long ptr ptr)
@ stdcall -import FindDebugInfoFile(str str ptr)
@ stdcall -import FindDebugInfoFileEx(str str ptr ptr ptr)
@ stdcall -import FindExecutableImage(str str str)
@ stdcall -import FindExecutableImageEx(str str ptr ptr ptr)
@ stub FindFileInPath
@ stub FindFileInSearchPath
@ stdcall GetImageConfigInformation(ptr ptr)
@ stdcall GetImageUnusedHeaderBytes(ptr ptr)
@ stdcall -import GetTimestampForLoadedLibrary(long)
@ stdcall ImageAddCertificate(long ptr ptr)
@ stdcall -import ImageDirectoryEntryToData(ptr long long ptr)
@ stdcall -import ImageDirectoryEntryToDataEx(ptr long long ptr ptr)
@ stdcall ImageEnumerateCertificates(long long ptr ptr long)
@ stdcall ImageGetCertificateData(long long ptr ptr)
@ stdcall ImageGetCertificateHeader(long long ptr)
@ stdcall ImageGetDigestStream(long long ptr long)
@ stdcall ImageLoad(str str)
@ stdcall -import ImageNtHeader(ptr) RtlImageNtHeader
@ stdcall ImageRemoveCertificate(long long)
@ stdcall -import ImageRvaToSection(ptr ptr long) RtlImageRvaToSection
@ stdcall -import ImageRvaToVa(ptr ptr long ptr) RtlImageRvaToVa
@ stdcall ImageUnload(ptr)
@ stdcall -import ImagehlpApiVersion()
@ stdcall -import ImagehlpApiVersionEx(ptr)
@ stdcall -import MakeSureDirectoryPathExists(str)
@ stdcall MapAndLoad(str str ptr long long)
@ stdcall -import -arch=win32 MapDebugInformation(long str str long)
@ stdcall MapFileAndCheckSumA(str ptr ptr)
@ stdcall MapFileAndCheckSumW(wstr ptr ptr)
@ stub  MarkImageAsRunFromSwap
@ stub ReBaseImage64
@ stdcall ReBaseImage(str str long long long long ptr ptr ptr ptr long)
@ stdcall RemovePrivateCvSymbolic(ptr ptr ptr)
@ stub RemovePrivateCvSymbolicEx
@ stdcall RemoveRelocations(ptr)
@ stdcall -import SearchTreeForFile(str str ptr)
@ stdcall SetImageConfigInformation(ptr ptr)
@ stdcall SplitSymbols(str str str long)
@ stdcall -import StackWalk64(long long long ptr ptr ptr ptr ptr ptr)
@ stdcall -import StackWalk(long long long ptr ptr ptr ptr ptr ptr)
@ stdcall -import SymCleanup(long)
@ stdcall SymEnumSourceFiles(ptr int64 str ptr ptr) SymEnumSourceFiles
@ stub SymEnumSym
@ stdcall -import SymEnumSymbols(ptr int64 str ptr ptr)
@ stdcall -import SymEnumTypes(ptr int64 ptr ptr)
@ stdcall -import SymEnumerateModules64(long ptr ptr)
@ stdcall -import SymEnumerateModules(long ptr ptr)
@ stdcall -import SymEnumerateSymbols64(long int64 ptr ptr)
@ stdcall -import SymEnumerateSymbols(long long ptr ptr)
@ stub SymEnumerateSymbolsW64
@ stub SymEnumerateSymbolsW
@ stdcall -import SymFindFileInPath(long str str ptr long long long ptr ptr ptr)
@ stdcall -import SymFromAddr(ptr int64 ptr ptr)
@ stdcall -import SymFromName(long str ptr)
@ stdcall -import SymFunctionTableAccess64(long int64)
@ stdcall -import SymFunctionTableAccess(long long)
@ stdcall -import SymGetLineFromAddr64(long int64 ptr ptr)
@ stdcall -import SymGetLineFromAddr(long long ptr ptr)
@ stub SymGetLineFromName64
@ stub SymGetLineFromName
@ stdcall -import SymGetLineNext64(long ptr)
@ stdcall -import SymGetLineNext(long ptr)
@ stdcall -import SymGetLinePrev64(long ptr)
@ stdcall -import SymGetLinePrev(long ptr)
@ stdcall -import SymGetModuleBase64(long int64)
@ stdcall -import SymGetModuleBase(long long)
@ stdcall -import SymGetModuleInfo64(long int64 ptr)
@ stdcall -import SymGetModuleInfo(long long ptr)
@ stdcall -import SymGetModuleInfoW64(long int64 ptr)
@ stdcall -import SymGetModuleInfoW(long long ptr)
@ stdcall -import SymGetOptions()
@ stdcall -import SymGetSearchPath(long ptr long)
@ stdcall -import SymGetSymFromAddr64(long int64 ptr ptr)
@ stdcall -import SymGetSymFromAddr(long long ptr ptr)
@ stdcall -import SymGetSymFromName64(long str ptr)
@ stdcall -import SymGetSymFromName(long str ptr)
@ stdcall -import SymGetSymNext64(long ptr)
@ stdcall -import SymGetSymNext(long ptr)
@ stdcall -import SymGetSymPrev64(long ptr)
@ stdcall -import SymGetSymPrev(long ptr)
@ stdcall -import SymGetTypeFromName(ptr int64 str ptr)
@ stdcall -import SymGetTypeInfo(ptr int64 long long ptr)
@ stdcall -import SymInitialize(long str long)
@ stdcall -import SymLoadModule64(long long str str int64 long)
@ stdcall -import SymLoadModule(long long str str long long)
@ stdcall -import SymMatchFileName(str str ptr ptr)
@ stdcall -import SymMatchString(str str long)
@ stdcall -import SymRegisterCallback64(long ptr int64)
@ stdcall -import SymRegisterCallback(long ptr ptr)
@ stdcall -import SymRegisterFunctionEntryCallback64(ptr ptr int64)
@ stdcall -import SymRegisterFunctionEntryCallback(ptr ptr ptr)
@ stdcall -import SymSetContext(long ptr ptr)
@ stdcall -import SymSetOptions(long)
@ stdcall -import SymSetSearchPath(long str)
@ stdcall -import SymUnDName64(ptr str long)
@ stdcall -import SymUnDName(ptr str long)
@ stdcall -import SymUnloadModule64(long int64)
@ stdcall -import SymUnloadModule(long long)
@ stdcall TouchFileTimes(long ptr)
@ stdcall -import UnDecorateSymbolName(str str long long)
@ stdcall UnMapAndLoad(ptr)
@ stdcall -import -arch=win32 UnmapDebugInformation(ptr)
@ stdcall UpdateDebugInfoFile(str str str ptr)
@ stdcall UpdateDebugInfoFileEx(str str str ptr long)

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ EnumerateLoadedModulesEx
@ stub -version=0xA00+ EnumerateLoadedModulesExW
@ stub -version=0xA00+ EnumerateLoadedModulesW64
@ stub -version=0xA00+ GetSymLoadError
@ stub -version=0xA00+ ImageGetCertificateDataEx
@ stub -version=0xA00+ IsBufferCleanOfInvalidMarkers
@ stub -version=0xA00+ RemoveInvalidModuleList
@ stub -version=0xA00+ ReportSymbolLoadSummary
@ stub -version=0xA00+ SetCheckUserInterruptShared
@ stub -version=0xA00+ SetSymLoadError
@ stub -version=0xA00+ StackWalk2
@ stub -version=0xA00+ StackWalkEx
@ stub -version=0xA00+ SymAddrIncludeInlineTrace
@ stub -version=0xA00+ SymAllocDiaString
@ stub -version=0xA00+ SymCompareInlineTrace
@ stub -version=0xA00+ SymEnumSymbolsEx
@ stub -version=0xA00+ SymEnumSymbolsExW
@ stub -version=0xA00+ SymEnumSymbolsForAddr
@ stub -version=0xA00+ SymEnumTypesByName
@ stub -version=0xA00+ SymEnumTypesByNameW
@ stub -version=0xA00+ SymEnumTypesW
@ stub -version=0xA00+ SymFindFileInPathW
@ stub -version=0xA00+ SymFreeDiaString
@ stub -version=0xA00+ SymFromInlineContext
@ stub -version=0xA00+ SymFromInlineContextW
@ stub -version=0xA00+ SymFunctionTableAccess64AccessRoutines
@ stub -version=0xA00+ SymGetDiaSession
@ stub -version=0xA00+ SymGetDiaSource
@ stub -version=0xA00+ SymGetExtendedOption
@ stub -version=0xA00+ SymGetLineFromInlineContext
@ stub -version=0xA00+ SymGetLineFromInlineContextW
@ stub -version=0xA00+ SymGetParentWindow
@ stub -version=0xA00+ SymGetSourceFileChecksumW
@ stub -version=0xA00+ SymGetSourceFileFromTokenByTokenName
@ stub -version=0xA00+ SymGetSourceFileFromTokenByTokenNameW
@ stub -version=0xA00+ SymGetSourceFileFromTokenW
@ stub -version=0xA00+ SymGetSourceFileTokenByTokenName
@ stub -version=0xA00+ SymGetSourceFileTokenByTokenNameW
@ stub -version=0xA00+ SymGetSourceFileTokenW
@ stub -version=0xA00+ SymGetSourceVarFromTokenW
@ stub -version=0xA00+ SymGetSymbolFile
@ stub -version=0xA00+ SymGetSymbolFileW
@ stub -version=0xA00+ SymGetTypeFromNameW
@ stub -version=0xA00+ SymGetTypeInfoEx
@ stub -version=0xA00+ SymMatchFileNameW
@ stub -version=0xA00+ SymMatchStringA
@ stub -version=0xA00+ SymMatchStringW
@ stub -version=0xA00+ SymQueryInlineTrace
@ stub -version=0xA00+ SymRegisterGetSourcePathPartCallback
@ stub -version=0xA00+ SymRegisterSourceFileUrlListCallback
@ stub -version=0xA00+ SymSetDiaSession
@ stub -version=0xA00+ SymSetExtendedOption
@ stub -version=0xA00+ SymSetScopeFromAddr
@ stub -version=0xA00+ SymSetScopeFromIndex
@ stub -version=0xA00+ SymSetScopeFromInlineContext
@ stub -version=0xA00+ SymSetServiceManager
@ stub -version=0xA00+ SymSrvGetFileIndexString
@ stub -version=0xA00+ SymSrvGetFileIndexStringW
@ stub -version=0xA00+ SymSrvGetFileIndexes
@ stub -version=0xA00+ SymSrvGetFileIndexesW
# END win10-surface stubs
