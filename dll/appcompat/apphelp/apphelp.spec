@ stdcall AllowPermLayer(wstr)
@ stub ApphelpCheckExe
@ stdcall ApphelpCheckInstallShieldPackage(ptr wstr)
@ stub ApphelpCheckMsiPackage
@ stub ApphelpCheckRunApp
@ stdcall ApphelpCheckRunAppEx(ptr ptr ptr wstr ptr long ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall ApphelpCheckShellObject(ptr long ptr)
@ stub ApphelpCreateAppcompatData
@ stub ApphelpFixMsiPackage
@ stub ApphelpFixMsiPackageExe
@ stub ApphelpFreeFileAttributes
@ stub ApphelpGetFileAttributes
@ stub ApphelpGetMsiProperties
@ stub ApphelpGetNTVDMInfo
@ stub ApphelpParseModuleData
@ stub ApphelpQueryModuleData
@ stub ApphelpQueryModuleDataEx
@ stub ApphelpUpdateCacheEntry
@ stub GetPermLayers
@ stub SdbAddLayerTagRefToQuery
@ stub SdbApphelpNotify
@ stub SdbApphelpNotifyExSdbApphelpNotifyEx
@ stdcall SdbBeginWriteListTag(ptr long)
@ stub SdbBuildCompatEnvVariables
@ stub SdbCloseApphelpInformation
@ stdcall SdbCloseDatabase(ptr)
@ stdcall SdbCloseDatabaseWrite(ptr)
@ stub SdbCloseLocalDatabase
@ stub SdbCommitIndexes
@ stdcall SdbCreateDatabase(wstr long)
@ stub SdbCreateHelpCenterURL
@ stub SdbCreateMsiTransformFile
@ stub SdbDeclareIndex
@ stub SdbDumpSearchPathPartCaches
@ stub SdbEnumMsiTransforms
@ stdcall SdbEndWriteListTag(ptr long)
@ stub SdbEscapeApphelpURL
@ stub SdbFindFirstDWORDIndexedTag
@ stub SdbFindFirstMsiPackage
@ stub SdbFindFirstMsiPackage_Str
@ stdcall SdbFindFirstNamedTag(ptr long long long wstr)
@ stub SdbFindFirstStringIndexedTag
@ stdcall SdbFindFirstTag(ptr long long)
@ stub SdbFindFirstTagRef
@ stub SdbFindNextDWORDIndexedTag
@ stub SdbFindNextMsiPackage
@ stub SdbFindNextStringIndexedTag
@ stdcall SdbFindNextTag(ptr long long)
@ stub SdbFindNextTagRef
@ stdcall SdbFreeDatabaseInformation(ptr)
@ stdcall SdbFreeFileAttributes(ptr)
@ stub SdbFreeFileInfo
@ stub SdbFreeFlagInfo
@ stdcall SdbGetAppCompatDataSize(ptr)
@ stdcall SdbGetAppPatchDir(ptr wstr long)
@ stdcall SdbGetBinaryTagData(ptr long)
@ stdcall SdbGetDatabaseID(ptr ptr)
@ stdcall SdbGetDatabaseInformation(ptr ptr)
@ stdcall SdbGetDatabaseInformationByName(wstr ptr)
@ stub SdbGetDatabaseMatch
@ stdcall SdbGetDatabaseVersion(wstr ptr ptr)
@ stub SdbGetDllPath
@ stub SdbGetEntryFlags
@ stdcall SdbGetFileAttributes(wstr ptr ptr)
@ stub SdbGetFileImageType
@ stub SdbGetFileImageTypeEx
@ stub SdbGetFileInfo
@ stdcall SdbGetFirstChild(ptr long)
@ stub SdbGetIndex
@ stub SdbGetItemFromItemRef
@ stub SdbGetLayerName
@ stdcall SdbGetLayerTagRef(ptr wstr)
@ stub SdbGetLocalPDB
@ stdcall SdbGetMatchingExe(ptr wstr wstr wstr long ptr)
@ stub SdbGetMsiPackageInformation
@ stub SdbGetNamedLayer
@ stdcall SdbGetNextChild(ptr long long)
@ stub SdbGetNthUserSdb
@ stdcall SdbGetPermLayerKeys(wstr wstr ptr long)
@ stub SdbGetShowDebugInfoOption
@ stub SdbGetShowDebugInfoOptionValue
@ stdcall SdbGetStandardDatabaseGUID(long ptr)
@ stdcall SdbGetStringTagPtr(ptr long)
@ stdcall SdbGetTagDataSize(ptr long)
@ stdcall SdbGetTagFromTagID(ptr long)
@ stub SdbGrabMatchingInfo
@ stub SdbGrabMatchingInfoEx
@ stdcall SdbGUIDFromString(wstr ptr)
@ stdcall SdbGUIDToString(ptr wstr long)
@ stdcall SdbInitDatabase(long wstr)
@ stub SdbInitDatabaseEx
@ stdcall SdbIsNullGUID(ptr)
@ stub SdbIsStandardDatabase
@ stub SdbIsTagrefFromLocalDB
@ stub SdbIsTagrefFromMainDB
@ stub SdbLoadString
@ stdcall SdbMakeIndexKeyFromString(wstr)
@ stub SdbOpenApphelpDetailsDatabase
@ stub SdbOpenApphelpDetailsDatabaseSP
@ stub SdbOpenApphelpInformation
@ stub SdbOpenApphelpInformationByID
@ stub SdbOpenApphelpResourceFile
@ stdcall SdbOpenDatabase(wstr long)
@ stub SdbOpenDbFromGuid
@ stub SdbOpenLocalDatabase
@ stdcall SdbPackAppCompatData(ptr ptr ptr ptr)
@ stdcall SdbUnpackAppCompatData(ptr wstr ptr ptr)
@ stub SdbQueryApphelpInformation
@ stub SdbQueryBlockUpgrade
@ stub SdbQueryContext
@ stdcall SdbQueryData(ptr long wstr ptr ptr ptr)
@ stdcall SdbQueryDataEx(ptr long wstr ptr ptr ptr ptr)
@ stdcall SdbQueryDataExTagID(ptr long wstr ptr ptr ptr ptr)
@ stub SdbQueryFlagInfo
@ stub SdbQueryName
@ stub SdbQueryReinstallUpgrade
@ stub SdbReadApphelpData
@ stub SdbReadApphelpDetailsData
@ stdcall SdbReadBinaryTag(ptr long ptr long)
@ stub SdbReadBYTETag
@ stdcall SdbReadDWORDTag(ptr long long)
@ stub SdbReadDWORDTagRef
@ stub SdbReadEntryInformation
@ stub SdbReadMsiTransformInfo
@ stub SdbReadPatchBits
@ stdcall SdbReadQWORDTag(ptr long int64)
@ stub SdbReadQWORDTagRef
@ stdcall SdbReadStringTag(ptr long wstr long)
@ stub SdbReadStringTagRef
@ stdcall SdbReadWORDTag(ptr long long)
@ stub SdbReadWORDTagRef
@ stdcall SdbRegisterDatabase(wstr long)
@ stdcall SdbReleaseDatabase(ptr)
@ stub SdbReleaseMatchingExe
@ stub SdbResolveDatabase
@ stub SdbSetApphelpDebugParameters
@ stub SdbSetEntryFlags
@ stub SdbSetImageType
@ stdcall SdbSetPermLayerKeys(wstr wstr long)
@ stub SdbShowApphelpDialog
@ stub SdbShowApphelpFromQuery
@ stub SdbStartIndexing
@ stub SdbStopIndexing
@ stub SdbStringDuplicate
@ stub SdbStringReplace
@ stub SdbStringReplaceArray
@ stdcall SdbTagIDToTagRef(ptr ptr long ptr)
@ stdcall SdbTagRefToTagID(ptr long ptr ptr)
@ stdcall SdbTagToString(long)
@ stdcall SdbUnregisterDatabase(ptr)
@ stdcall SdbWriteBinaryTag(ptr long ptr long)
@ stdcall SdbWriteBinaryTagFromFile(ptr long wstr)
@ stub SdbWriteBYTETag
@ stdcall SdbWriteDWORDTag(ptr long long)
@ stdcall SdbWriteNULLTag(ptr long)
@ stdcall SdbWriteQWORDTag(ptr long int64)
@ stdcall SdbWriteStringRefTag(ptr long long)
@ stdcall SdbWriteStringTag(ptr long wstr)
@ stub SdbWriteStringTagDirect
@ stdcall SdbWriteWORDTag(ptr long long)
@ stdcall SE_DllLoaded(ptr)
@ stdcall SE_DllUnloaded(ptr)
@ stdcall SE_DynamicShim(wstr ptr ptr str ptr)
@ stub SE_DynamicUnshim
@ stdcall SE_InstallAfterInit(ptr ptr)
@ stdcall SE_InstallBeforeInit(ptr ptr)
@ stdcall SE_IsShimDll(ptr)
@ stdcall SE_ProcessDying()
@ stub SE_GetHookAPIs
@ stub SE_GetMaxShimCount
@ stub SE_GetProcAddressLoad
@ stub SE_GetShimCount
@ stub SE_LdrEntryRemoved
@ stub SetPermLayers
@ varargs ShimDbgPrint(long str str)
@ stdcall ShimDumpCache(ptr ptr wstr long)
@ stdcall ShimFlushCache(ptr ptr wstr long)
@ stdcall SetPermLayerState(wstr wstr long long long)
@ stdcall SdbRegisterDatabaseEx(wstr long ptr)

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ ApphelpCheckAPO
@ stub -version=0xA00+ ApphelpCheckIME
@ stub -version=0xA00+ ApphelpCheckModule
@ stub -version=0xA00+ ApphelpChpeModSettingsFromQueryResult
@ stub -version=0xA00+ ApphelpGetShimDebugLevel
@ stub -version=0xA00+ ApphelpIsPortMonAllowed
@ stub -version=0xA00+ ApphelpNotifyPcaOfProblem
@ stub -version=0xA00+ ApphelpQueryModSettingsAlloc
@ stub -version=0xA00+ ApphelpShowDialog
@ stub -arch=win32 -version=0xA00+ DWM8And16Bit_ChangeDisplaySettingsExW_CallOut
@ stub -arch=win32 -version=0xA00+ DWM8And16Bit_DirectDrawCreateEx_CallOut
@ stub -arch=win32 -version=0xA00+ DWM8And16Bit_DirectDrawCreate_CallOut
@ stub -arch=win32 -version=0xA00+ DWM8And16Bit_EnumDisplaySettingsExW_CallOut
@ stub -arch=win32 -version=0xA00+ DWM8And16Bit_IsShimApplied_CallOut
@ stub -arch=win32 -version=0xA00+ DWM8And16Bit_RestoreDisplayMode_CallOut
@ stub -version=0xA00+ SE_AddHookset
@ stub -version=0xA00+ SE_CALLBACK_AddHook
@ stub -version=0xA00+ SE_CALLBACK_Lookup
@ stub -version=0xA00+ SE_COM_AddHook
@ stub -version=0xA00+ SE_COM_AddServer
@ stub -version=0xA00+ SE_COM_HookInterface
@ stub -version=0xA00+ SE_COM_HookObject
@ stub -version=0xA00+ SE_COM_Lookup
@ stub -version=0xA00+ SE_GetProcAddressForCaller
@ stub -version=0xA00+ SE_GetProcAddressIgnoreIncExc
@ stub -version=0xA00+ SE_GetShimId
@ stub -version=0xA00+ SE_InitializeEngine
@ stub -version=0xA00+ SE_LdrResolveDllName
@ stub -version=0xA00+ SE_LookupAddress
@ stub -version=0xA00+ SE_LookupCaller
@ stub -version=0xA00+ SE_ShimDPF
@ stub -version=0xA00+ SE_ShimDllLoaded
@ stub -version=0xA00+ SE_WINRT_AddHook
@ stub -version=0xA00+ SE_WINRT_HookObject
@ stub -version=0xA00+ SdbApphelpNotifyEx
@ stub -version=0xA00+ SdbApphelpNotifyEx2
@ stub -version=0xA00+ SdbDeletePermLayerKeys
@ stub -version=0xA00+ SdbFindCustomActionForPackage
@ stub -version=0xA00+ SdbFindFirstGUIDIndexedTag
@ stub -version=0xA00+ SdbFindMsiPackageByID
@ stub -version=0xA00+ SdbFindNextGUIDIndexedTag
@ stub -version=0xA00+ SdbFormatAttribute
@ stub -version=0xA00+ SdbGetDatabaseGUID
@ stub -version=0xA00+ SdbGetImageType
@ stub -version=0xA00+ SdbGetPDBFromGUID
@ stub -version=0xA00+ SdbGetPathCustomSdb
@ stub -version=0xA00+ SdbGetPathSystemSdb
@ stub -version=0xA00+ SdbIsDbRuntimePlatformSupportedOnHost
@ stub -version=0xA00+ SdbIsTagrefFromMergeStubDB
@ stub -version=0xA00+ SdbQueryFlagMask
@ stub -version=0xA00+ SdbReadBYTETagRef
@ stub -version=0xA00+ SdbUnpackQueryResult
@ stub -version=0xA00+ SetPermLayerStateEx
# END win10-surface stubs
