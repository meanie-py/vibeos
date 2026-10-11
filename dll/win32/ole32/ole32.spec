# CoVrfCheckThreadState
# CoVrfGetThreadState
# CoVrfReleaseThreadState
# PropVariantChangeType
@ stdcall BindMoniker(ptr long ptr ptr)
@ stdcall CLIPFORMAT_UserFree(ptr ptr)
@ stdcall CLIPFORMAT_UserMarshal(ptr ptr ptr)
@ stdcall CLIPFORMAT_UserSize(ptr long ptr)
@ stdcall CLIPFORMAT_UserUnmarshal(ptr ptr ptr)
# CLSIDFromOle1Class
@ stdcall CLSIDFromProgID(wstr ptr)
@ stdcall CLSIDFromProgIDEx(wstr ptr)
@ stdcall CLSIDFromString(wstr ptr)
@ stdcall CoAddRefServerProcess()
@ stdcall CoAllowSetForegroundWindow(ptr ptr)
@ stdcall CoBuildVersion()
@ stdcall -stub CoCancelCall(long long)
@ stdcall CoCopyProxy(ptr ptr)
@ stdcall CoCreateFreeThreadedMarshaler(ptr ptr)
@ stdcall CoCreateGuid(ptr)
@ stdcall CoCreateInstance(ptr ptr long ptr ptr)
@ stdcall CoCreateInstanceEx(ptr ptr long ptr long ptr)
@ stdcall CoCreateInstanceFromApp(ptr ptr long ptr long ptr)
# CoCreateObjectInContext
# CoDeactivateObject
@ stdcall CoDecrementMTAUsage(ptr)
@ stdcall CoDisableCallCancellation(ptr)
@ stdcall -stub -version=0x600+ CoDisconnectContext(long)
@ stdcall CoDisconnectObject(ptr long)
@ stdcall CoDosDateTimeToFileTime(long long ptr) kernel32.DosDateTimeToFileTime
@ stdcall CoEnableCallCancellation(ptr)
@ stdcall CoFileTimeNow(ptr)
@ stdcall CoFileTimeToDosDateTime(ptr ptr ptr) kernel32.FileTimeToDosDateTime
@ stdcall CoFreeAllLibraries()
@ stdcall CoFreeLibrary(long)
@ stdcall CoFreeUnusedLibraries()
@ stdcall CoFreeUnusedLibrariesEx(long long)
@ stdcall CoGetActivationState(int128 long ptr)
# CoGetApartmentID
@ stdcall CoGetApartmentType(ptr ptr)
@ stdcall CoGetCallContext(ptr ptr)
@ stdcall CoGetCallState(long ptr)
@ stdcall CoGetCallerTID(ptr)
@ stdcall -stub CoGetCancelObject(long ptr ptr)
@ stdcall CoGetClassObject(ptr long ptr ptr ptr)
# CoGetClassVersion
# CoGetComCatalog
@ stdcall CoGetContextToken(ptr)
@ stdcall CoGetCurrentLogicalThreadId(ptr)
@ stdcall CoGetCurrentProcess()
@ stdcall CoGetDefaultContext(long ptr ptr)
@ stdcall CoGetInstanceFromFile(ptr ptr ptr long long wstr long ptr)
@ stdcall CoGetInstanceFromIStorage(ptr ptr ptr long ptr long ptr)
@ stdcall -stub CoGetInterceptor(ptr ptr ptr ptr)
@ stdcall -stub CoGetInterceptorFromTypeInfo(ptr ptr ptr ptr ptr)
@ stdcall CoGetInterfaceAndReleaseStream(ptr ptr ptr)
@ stdcall CoGetMalloc(long ptr)
@ stdcall CoGetMarshalSizeMax(ptr ptr ptr long ptr long)
# CoGetModuleType
@ stdcall CoGetObject(wstr ptr ptr ptr)
@ stdcall CoGetObjectContext(ptr ptr)
@ stdcall CoGetPSClsid(ptr ptr)
# CoGetProcessIdentifier
@ stdcall CoGetStandardMarshal(ptr ptr long ptr long ptr)
@ stdcall CoGetState(ptr)
@ stdcall -stub CoGetStdMarshalEx(ptr long ptr)
# CoGetSystemSecurityPermissions
@ stdcall CoGetTreatAsClass(ptr ptr)
@ stdcall CoIncrementMTAUsage(ptr)
@ stdcall CoImpersonateClient()
@ stdcall CoInitialize(ptr)
@ stdcall CoInitializeEx(ptr long)
@ stdcall CoInitializeSecurity(ptr long ptr ptr long long ptr long ptr)
@ stdcall CoInitializeWOW(long long)
# CoInstall
# CoInvalidateRemoteMachineBindings
@ stdcall CoIsHandlerConnected(ptr)
@ stdcall CoIsOle1Class (ptr)
@ stdcall CoLoadLibrary(wstr long)
@ stdcall CoLockObjectExternal(ptr long long)
@ stdcall CoMarshalHresult(ptr long)
@ stdcall CoMarshalInterThreadInterfaceInStream(ptr ptr ptr)
@ stdcall CoMarshalInterface(ptr ptr ptr long ptr long)
# CoPopServiceDomain
# CoPushServiceDomain
@ stub CoQueryAuthenticationServices
@ stdcall CoQueryClientBlanket(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall CoQueryProxyBlanket(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stub CoQueryReleaseObject
# CoReactivateObject
@ stdcall CoRegisterChannelHook(ptr ptr)
@ stdcall CoRegisterClassObject(ptr ptr long long ptr)
@ stdcall CoRegisterInitializeSpy(ptr ptr)
@ stdcall CoRegisterMallocSpy (ptr)
@ stdcall CoRegisterMessageFilter(ptr ptr)
@ stdcall CoRegisterPSClsid(ptr ptr)
@ stdcall CoRegisterSurrogate(ptr)
@ stdcall CoRegisterSurrogateEx(ptr ptr)
@ stdcall CoReleaseMarshalData(ptr)
@ stdcall CoReleaseServerProcess()
@ stdcall CoResumeClassObjects()
# CoRetireServer
@ stdcall CoRevertToSelf()
@ stdcall CoRevokeClassObject(long)
@ stdcall CoRevokeInitializeSpy(int64)
@ stdcall CoRevokeMallocSpy()
# CoSetCancelObject
@ stdcall CoSetProxyBlanket(ptr long long ptr long long ptr long)
@ stdcall CoSetState(ptr)
@ stdcall CoSuspendClassObjects()
@ stdcall CoSwitchCallContext(ptr ptr)
@ stdcall CoTaskMemAlloc(long)
@ stdcall CoTaskMemFree(ptr)
@ stdcall CoTaskMemRealloc(ptr long)
# CoTestCancel
@ stdcall CoTreatAsClass(ptr ptr)
@ stdcall CoUninitialize()
@ stub CoUnloadingWOW
@ stdcall CoUnmarshalHresult(ptr ptr)
@ stdcall CoUnmarshalInterface(ptr ptr ptr)
@ stdcall CoWaitForMultipleHandles(long long long ptr ptr)
# ComPs_NdrDllCanUnloadNow
# ComPs_NdrDllGetClassObject
# ComPs_NdrDllRegisterProxy
# ComPs_NdrDllUnregisterProxy
@ stdcall CreateAntiMoniker(ptr)
@ stdcall CreateBindCtx(long ptr)
@ stdcall CreateClassMoniker(ptr ptr)
@ stdcall CreateDataAdviseHolder(ptr)
@ stdcall CreateDataCache(ptr ptr ptr ptr)
@ stdcall CreateErrorInfo(ptr)
@ stdcall CreateFileMoniker(wstr ptr)
@ stdcall CreateGenericComposite(ptr ptr ptr)
@ stdcall CreateILockBytesOnHGlobal(ptr long ptr)
@ stdcall CreateItemMoniker(wstr wstr ptr)
@ stdcall CreateObjrefMoniker(ptr ptr)
@ stdcall CreateOleAdviseHolder(ptr)
@ stdcall CreatePointerMoniker(ptr ptr)
# CreateStdProgressIndicator
@ stdcall CreateStreamOnHGlobal(ptr long ptr)
# DcomChannelSetHResult
@ stdcall DllDebugObjectRPCHook(long ptr)
@ stdcall DllGetClassObject(ptr ptr ptr)
@ stub DllGetClassObjectWOW
@ stdcall -private DllRegisterServer()
@ stdcall DoDragDrop(ptr ptr long ptr)
@ stub EnableHookObject
@ stdcall FmtIdToPropStgName(ptr wstr)
@ stdcall FreePropVariantArray(long ptr)
@ stdcall GetClassFile(wstr ptr)
@ stdcall GetConvertStg(ptr)
@ stub GetDocumentBitStg
@ stdcall GetErrorInfo(long ptr)
@ stdcall GetHGlobalFromILockBytes(ptr ptr)
@ stdcall GetHGlobalFromStream(ptr ptr)
@ stub GetHookInterface
@ stdcall GetRunningObjectTable(long ptr)
@ stdcall HACCEL_UserFree(ptr ptr)
@ stdcall HACCEL_UserMarshal(ptr ptr ptr)
@ stdcall HACCEL_UserSize(ptr long ptr)
@ stdcall HACCEL_UserUnmarshal(ptr ptr ptr)
@ stdcall HBITMAP_UserFree(ptr ptr)
@ stdcall HBITMAP_UserMarshal(ptr ptr ptr)
@ stdcall HBITMAP_UserSize(ptr long ptr)
@ stdcall HBITMAP_UserUnmarshal(ptr ptr ptr)
@ stdcall HBRUSH_UserFree(ptr ptr)
@ stdcall HBRUSH_UserMarshal(ptr ptr ptr)
@ stdcall HBRUSH_UserSize(ptr long ptr)
@ stdcall HBRUSH_UserUnmarshal(ptr ptr ptr)
@ stdcall HDC_UserFree(ptr ptr)
@ stdcall HDC_UserMarshal(ptr ptr ptr)
@ stdcall HDC_UserSize(ptr long ptr)
@ stdcall HDC_UserUnmarshal(ptr ptr ptr)
@ stdcall HENHMETAFILE_UserFree(ptr ptr)
@ stdcall HENHMETAFILE_UserMarshal(ptr ptr ptr)
@ stdcall HENHMETAFILE_UserSize(ptr long ptr)
@ stdcall HENHMETAFILE_UserUnmarshal(ptr ptr ptr)
@ stdcall HGLOBAL_UserFree(ptr ptr)
@ stdcall HGLOBAL_UserMarshal(ptr ptr ptr)
@ stdcall HGLOBAL_UserSize(ptr long ptr)
@ stdcall HGLOBAL_UserUnmarshal(ptr ptr ptr)
@ stdcall HICON_UserFree(ptr ptr)
@ stdcall HICON_UserMarshal(ptr ptr ptr)
@ stdcall HICON_UserSize(ptr long ptr)
@ stdcall HICON_UserUnmarshal(ptr ptr ptr)
@ stdcall HMENU_UserFree(ptr ptr)
@ stdcall HMENU_UserMarshal(ptr ptr ptr)
@ stdcall HMENU_UserSize(ptr long ptr)
@ stdcall HMENU_UserUnmarshal(ptr ptr ptr)
@ stdcall HMETAFILEPICT_UserFree(ptr ptr)
@ stdcall HMETAFILEPICT_UserMarshal(ptr ptr ptr)
@ stdcall HMETAFILEPICT_UserSize(ptr long ptr)
@ stdcall HMETAFILEPICT_UserUnmarshal(ptr ptr ptr)
@ stdcall HMETAFILE_UserFree(ptr ptr)
@ stdcall HMETAFILE_UserMarshal(ptr ptr ptr)
@ stdcall HMETAFILE_UserSize(ptr long ptr)
@ stdcall HMETAFILE_UserUnmarshal(ptr ptr ptr)
@ stdcall HPALETTE_UserFree(ptr ptr)
@ stdcall HPALETTE_UserMarshal(ptr ptr ptr)
@ stdcall HPALETTE_UserSize(ptr long ptr)
@ stdcall HPALETTE_UserUnmarshal(ptr ptr ptr)
@ stdcall HWND_UserFree(ptr ptr)
@ stdcall HWND_UserMarshal(ptr ptr ptr)
@ stdcall HWND_UserSize(ptr long ptr)
@ stdcall HWND_UserUnmarshal(ptr ptr ptr)
# HkOleRegisterObject
@ stdcall IIDFromString(wstr ptr)
@ stdcall IsAccelerator(long long ptr long)
@ stdcall IsEqualGUID(ptr ptr)
@ stub IsValidIid
@ stdcall IsValidInterface(ptr)
@ stub IsValidPtrIn
@ stub IsValidPtrOut
@ stdcall MkParseDisplayName(ptr ptr ptr ptr)
@ stdcall MonikerCommonPrefixWith(ptr ptr ptr)
@ stub MonikerRelativePathTo
@ stdcall OleBuildVersion()
@ stdcall OleConvertIStorageToOLESTREAM(ptr ptr)
@ stdcall OleConvertIStorageToOLESTREAMEx(ptr long long long long ptr ptr)
@ stdcall OleConvertOLESTREAMToIStorage(ptr ptr ptr)
@ stub OleConvertOLESTREAMToIStorageEx
@ stdcall OleCreate(ptr ptr long ptr ptr ptr ptr)
@ stdcall OleCreateDefaultHandler(ptr ptr ptr ptr)
@ stdcall OleCreateEmbeddingHelper(ptr ptr long ptr ptr ptr)
@ stub OleCreateEx
@ stdcall OleCreateFromData(ptr ptr long ptr ptr ptr ptr)
@ stdcall OleCreateFromDataEx(ptr ptr long long long ptr ptr ptr ptr ptr ptr ptr)
@ stdcall OleCreateFromFile(ptr wstr ptr long ptr ptr ptr ptr)
@ stdcall OleCreateFromFileEx(ptr wstr ptr long long long ptr ptr ptr ptr ptr ptr ptr)
@ stdcall OleCreateLink(ptr ptr long ptr ptr ptr ptr)
@ stub OleCreateLinkEx
@ stdcall OleCreateLinkFromData(ptr ptr long ptr ptr ptr ptr)
@ stub OleCreateLinkFromDataEx
@ stdcall OleCreateLinkToFile(ptr ptr long ptr ptr ptr ptr)
@ stub OleCreateLinkToFileEx
@ stdcall OleCreateMenuDescriptor(long ptr)
@ stdcall OleCreateStaticFromData(ptr ptr long ptr ptr ptr ptr)
@ stdcall OleDestroyMenuDescriptor(long)
@ stdcall OleDoAutoConvert(ptr ptr)
@ stdcall OleDraw(ptr long long ptr)
@ stdcall OleDuplicateData(long long long)
@ stdcall OleFlushClipboard()
@ stdcall OleGetAutoConvert(ptr ptr)
@ stdcall OleGetClipboard(ptr)
@ stdcall OleGetIconOfClass(ptr ptr long)
@ stdcall OleGetIconOfFile(wstr long)
@ stdcall OleInitialize(ptr)
@ stdcall OleInitializeWOW(long long)
@ stdcall OleIsCurrentClipboard(ptr)
@ stdcall OleIsRunning(ptr)
@ stdcall OleLoad(ptr ptr ptr ptr)
@ stdcall OleLoadFromStream(ptr ptr ptr)
@ stdcall OleLockRunning(ptr long long)
@ stdcall OleMetafilePictFromIconAndLabel(long ptr ptr long)
@ stdcall OleNoteObjectVisible(ptr long)
@ stdcall OleQueryCreateFromData(ptr)
@ stdcall OleQueryLinkFromData(ptr)
@ stdcall OleRegEnumFormatEtc(ptr long ptr)
@ stdcall OleRegEnumVerbs(long ptr)
@ stdcall OleRegGetMiscStatus(ptr long ptr)
@ stdcall OleRegGetUserType(long long ptr)
@ stdcall OleRun(ptr)
@ stdcall OleSave(ptr ptr long)
@ stdcall OleSaveToStream(ptr ptr)
@ stdcall OleSetAutoConvert(ptr ptr)
@ stdcall OleSetClipboard(ptr)
@ stdcall OleSetContainedObject(ptr long)
@ stdcall OleSetMenuDescriptor(long long long ptr ptr)
@ stdcall OleTranslateAccelerator(ptr ptr ptr)
@ stdcall OleUninitialize()
@ stub OpenOrCreateStream
@ stdcall ProgIDFromCLSID(ptr ptr)
@ stdcall PropStgNameToFmtId(wstr ptr)
@ stdcall PropSysAllocString(wstr)
@ stdcall PropSysFreeString(wstr)
@ stdcall PropVariantClear(ptr)
@ stdcall PropVariantCopy(ptr ptr)
@ stdcall ReadClassStg(ptr ptr)
@ stdcall ReadClassStm(ptr ptr)
@ stdcall ReadFmtUserTypeStg(ptr ptr ptr)
@ stub ReadOleStg
@ stub ReadStringStream
@ stdcall RegisterDragDrop(long ptr)
@ stdcall ReleaseStgMedium(ptr)
@ stdcall RevokeDragDrop(long)
@ stdcall SNB_UserFree(ptr ptr)
@ stdcall SNB_UserMarshal(ptr ptr ptr)
@ stdcall SNB_UserSize(ptr long ptr)
@ stdcall SNB_UserUnmarshal(ptr ptr ptr)
@ stdcall STGMEDIUM_UserFree(ptr ptr)
@ stdcall STGMEDIUM_UserMarshal(ptr ptr ptr)
@ stdcall STGMEDIUM_UserSize(ptr long ptr)
@ stdcall STGMEDIUM_UserUnmarshal(ptr ptr ptr)
@ stdcall SetConvertStg(ptr long)
@ stub SetDocumentBitStg
@ stdcall SetErrorInfo(long ptr)
@ stdcall StgConvertPropertyToVariant(ptr long ptr ptr)
@ stdcall StgConvertVariantToProperty(ptr long ptr ptr long long ptr)
@ stdcall StgCreateDocfile(wstr long long ptr)
@ stdcall StgCreateDocfileOnILockBytes(ptr long long ptr)
@ stdcall StgCreatePropSetStg(ptr long ptr)
@ stdcall StgCreatePropStg(ptr ptr ptr long long ptr)
@ stdcall StgCreateStorageEx(wstr long long long ptr ptr ptr ptr)
@ stub StgGetIFillLockBytesOnFile
@ stub StgGetIFillLockBytesOnILockBytes
@ stdcall StgIsStorageFile(wstr)
@ stdcall StgIsStorageILockBytes(ptr)
@ stub StgOpenAsyncDocfileOnIFillLockBytes
@ stdcall StgOpenPropStg(ptr ptr long long ptr)
@ stdcall StgOpenStorage(wstr ptr long ptr long ptr)
@ stdcall StgOpenStorageEx(wstr long long long ptr ptr ptr ptr)
# StgOpenStorageOnHandle
@ stdcall StgOpenStorageOnILockBytes(ptr ptr long long long ptr)
# StgPropertyLengthAsVariant
@ stdcall StgSetTimes(wstr ptr ptr ptr )
@ stdcall StringFromCLSID(ptr ptr)
@ stdcall StringFromGUID2(ptr ptr long)
@ stdcall StringFromIID(ptr ptr) StringFromCLSID
@ stub UpdateDCOMSettings
@ stub UtConvertDvtd16toDvtd32
@ stub UtConvertDvtd32toDvtd16
@ stub UtGetDvtd16Info
@ stub UtGetDvtd32Info
@ stdcall WdtpInterfacePointer_UserFree(ptr)
@ stdcall WdtpInterfacePointer_UserMarshal(ptr long ptr ptr ptr)
@ stdcall WdtpInterfacePointer_UserSize(ptr long ptr long ptr)
@ stdcall WdtpInterfacePointer_UserUnmarshal(ptr ptr ptr ptr)
@ stdcall WriteClassStg(ptr ptr)
@ stdcall WriteClassStm(ptr ptr)
@ stdcall WriteFmtUserTypeStg(ptr long ptr)
@ stub WriteOleStg
@ stub WriteStringStream

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -arch=win64 -version=0xA00+ CLIPFORMAT_UserFree64 # -> COMBASE.CLIPFORMAT_UserFree64
@ stub -version=0xA00+ CLIPFORMAT_UserFreeExt
@ stub -arch=win64 -version=0xA00+ CLIPFORMAT_UserMarshal64 # -> COMBASE.CLIPFORMAT_UserMarshal64
@ stub -version=0xA00+ CLIPFORMAT_UserMarshalExt
@ stub -arch=win64 -version=0xA00+ CLIPFORMAT_UserSize64 # -> COMBASE.CLIPFORMAT_UserSize64
@ stub -version=0xA00+ CLIPFORMAT_UserSizeExt
@ stub -arch=win64 -version=0xA00+ CLIPFORMAT_UserUnmarshal64 # -> COMBASE.CLIPFORMAT_UserUnmarshal64
@ stub -version=0xA00+ CLIPFORMAT_UserUnmarshalExt
@ stub -version=0xA00+ CLSIDFromOle1Class # -> api-ms-win-core-com-private-l1-1-1.CLSIDFromOle1Class
@ stub -version=0xA00+ CStdAsyncStubBuffer2_Connect # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer2_Connect
@ stub -version=0xA00+ CStdAsyncStubBuffer2_Disconnect # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer2_Disconnect
@ stub -version=0xA00+ CStdAsyncStubBuffer2_Release # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer2_Release
@ stub -version=0xA00+ CStdAsyncStubBuffer_AddRef # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer_AddRef
@ stub -version=0xA00+ CStdAsyncStubBuffer_Connect # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer_Connect
@ stub -version=0xA00+ CStdAsyncStubBuffer_Disconnect # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer_Disconnect
@ stub -version=0xA00+ CStdAsyncStubBuffer_Invoke # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer_Invoke
@ stub -version=0xA00+ CStdAsyncStubBuffer_QueryInterface # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer_QueryInterface
@ stub -version=0xA00+ CStdAsyncStubBuffer_Release # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdAsyncStubBuffer_Release
@ stub -version=0xA00+ CStdStubBuffer2_Connect # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdStubBuffer2_Connect
@ stub -version=0xA00+ CStdStubBuffer2_CountRefs # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdStubBuffer2_CountRefs
@ stub -version=0xA00+ CStdStubBuffer2_Disconnect # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdStubBuffer2_Disconnect
@ stub -version=0xA00+ CStdStubBuffer2_QueryInterface # -> api-ms-win-core-com-midlproxystub-l1-1-0.CStdStubBuffer2_QueryInterface
@ stub -version=0xA00+ CheckInitDde
@ stub -version=0xA00+ CleanROTForApartment
@ stub -arch=win32 -version=0xA00+ ClearQITableEntry
@ stub -version=0xA00+ ClipboardProcessUninitialize
@ stub -version=0xA00+ CoAicGetTokenForCOM
@ stub -version=0xA00+ CoAllowUnmarshalerCLSID # -> api-ms-win-core-com-l1-1-0.CoAllowUnmarshalerCLSID
@ stub -version=0xA00+ CoCheckElevationEnabled
@ stub -version=0xA00+ CoCreateObjectInContext # -> api-ms-win-core-com-private-l1-1-1.CoCreateObjectInContext
@ stub -version=0xA00+ CoDeactivateObject # -> api-ms-win-core-com-private-l1-1-1.CoDeactivateObject
@ stub -version=0xA00+ CoDecodeProxy # -> api-ms-win-core-com-l1-1-0.CoDecodeProxy
@ stub -version=0xA00+ CoGetApartmentID # -> api-ms-win-core-com-private-l1-1-1.CoGetApartmentID
@ stub -version=0xA00+ CoGetClassVersion # -> api-ms-win-core-com-private-l1-1-1.CoGetClassVersion
@ stub -version=0xA00+ CoGetComCatalog # -> api-ms-win-core-com-private-l1-1-1.GetCatalogHelper
@ stub -version=0xA00+ CoGetInterceptorForOle32
@ stub -version=0xA00+ CoGetModuleType # -> api-ms-win-core-com-private-l1-1-1.CoGetModuleType
@ stub -version=0xA00+ CoGetProcessIdentifier # -> api-ms-win-core-com-private-l1-1-1.CoGetProcessIdentifier
@ stub -version=0xA00+ CoGetSystemSecurityPermissions # -> api-ms-win-core-com-private-l1-1-1.CoGetSystemSecurityPermissions
@ stub -version=0xA00+ CoGetSystemWow64DirectoryW
@ stub -version=0xA00+ CoHandlePriorityEventsFromMessagePump # -> COMBASE.#111
@ stub -version=0xA00+ CoInstall
@ stub -version=0xA00+ CoInvalidateRemoteMachineBindings # -> api-ms-win-core-com-l1-1-0.CoInvalidateRemoteMachineBindings
@ stub -version=0xA00+ CoPopServiceDomain # -> api-ms-win-core-com-private-l1-1-1.CoPopServiceDomain
@ stub -version=0xA00+ CoPushServiceDomain # -> api-ms-win-core-com-private-l1-1-1.CoPushServiceDomain
@ stub -version=0xA00+ CoReactivateObject # -> api-ms-win-core-com-private-l1-1-1.CoReactivateObject
@ stub -version=0xA00+ CoRegisterActivationFilter # -> api-ms-win-core-com-l1-1-0.CoRegisterActivationFilter
@ stub -version=0xA00+ CoRetireServer # -> api-ms-win-core-com-private-l1-1-1.CoRetireServer
@ stub -version=0xA00+ CoSetCancelObject # -> api-ms-win-core-com-l1-1-0.CoSetCancelObject
@ stub -version=0xA00+ CoSetMessageDispatcher # -> COMBASE.#110
@ stub -version=0xA00+ CoTestCancel # -> api-ms-win-core-com-l1-1-0.CoTestCancel
@ stub -version=0xA00+ CoTryGetInterceptorForAppModelVirtualization
@ stub -version=0xA00+ CoVrfCheckThreadState # -> api-ms-win-core-com-private-l1-1-1.CoVrfCheckThreadState
@ stub -version=0xA00+ CoVrfGetThreadState # -> api-ms-win-core-com-private-l1-1-1.CoVrfGetThreadState
@ stub -version=0xA00+ CoVrfReleaseThreadState # -> api-ms-win-core-com-private-l1-1-1.CoVrfReleaseThreadState
@ stub -version=0xA00+ CoWaitForMultipleObjects # -> api-ms-win-core-com-l1-1-0.CoWaitForMultipleObjects
@ stub -version=0xA00+ ComPs_NdrDllCanUnloadNow
@ stub -version=0xA00+ ComPs_NdrDllGetClassObject
@ stub -version=0xA00+ ComPs_NdrDllRegisterProxy
@ stub -version=0xA00+ ComPs_NdrDllUnregisterProxy
@ stub -version=0xA00+ CreateStdProgressIndicator
@ stub -version=0xA00+ DcomChannelSetHResult # -> COMBASE.DcomChannelSetHResult
@ stub -version=0xA00+ DdeBindToObject
@ stub -version=0xA00+ DeletePatternAndExtensionTables
@ stub -version=0xA00+ DestroyRunningObjectTable
@ stub -version=0xA00+ DragDropSetFDT
@ stub -version=0xA00+ FindExt
@ stub -version=0xA00+ GetActiveObjectExt
@ stub -version=0xA00+ GetObjectFromRotByPath
@ stub -arch=win64 -version=0xA00+ HACCEL_UserFree64 # -> COMBASE.HACCEL_UserFree64
@ stub -arch=win64 -version=0xA00+ HACCEL_UserMarshal64 # -> COMBASE.HACCEL_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HACCEL_UserSize64 # -> COMBASE.HACCEL_UserSize64
@ stub -arch=win64 -version=0xA00+ HACCEL_UserUnmarshal64 # -> COMBASE.HACCEL_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HBITMAP_UserFree64 # -> COMBASE.HBITMAP_UserFree64
@ stub -arch=win64 -version=0xA00+ HBITMAP_UserMarshal64 # -> COMBASE.HBITMAP_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HBITMAP_UserSize64 # -> COMBASE.HBITMAP_UserSize64
@ stub -arch=win64 -version=0xA00+ HBITMAP_UserUnmarshal64 # -> COMBASE.HBITMAP_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HBRUSH_UserFree64 # -> COMBASE.HBRUSH_UserFree64
@ stub -arch=win64 -version=0xA00+ HBRUSH_UserMarshal64 # -> COMBASE.HBRUSH_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HBRUSH_UserSize64 # -> COMBASE.HBRUSH_UserSize64
@ stub -arch=win64 -version=0xA00+ HBRUSH_UserUnmarshal64 # -> COMBASE.HBRUSH_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HDC_UserFree64 # -> COMBASE.HDC_UserFree64
@ stub -arch=win64 -version=0xA00+ HDC_UserMarshal64 # -> COMBASE.HDC_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HDC_UserSize64 # -> COMBASE.HDC_UserSize64
@ stub -arch=win64 -version=0xA00+ HDC_UserUnmarshal64 # -> COMBASE.HDC_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HENHMETAFILE_UserFree64
@ stub -arch=win64 -version=0xA00+ HENHMETAFILE_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HENHMETAFILE_UserSize64
@ stub -arch=win64 -version=0xA00+ HENHMETAFILE_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HGLOBAL_UserFree64 # -> COMBASE.HGLOBAL_UserFree64
@ stub -arch=win64 -version=0xA00+ HGLOBAL_UserMarshal64 # -> COMBASE.HGLOBAL_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HGLOBAL_UserSize64 # -> COMBASE.HGLOBAL_UserSize64
@ stub -arch=win64 -version=0xA00+ HGLOBAL_UserUnmarshal64 # -> COMBASE.HGLOBAL_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HICON_UserFree64 # -> COMBASE.HICON_UserFree64
@ stub -arch=win64 -version=0xA00+ HICON_UserMarshal64 # -> COMBASE.HICON_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HICON_UserSize64 # -> COMBASE.HICON_UserSize64
@ stub -arch=win64 -version=0xA00+ HICON_UserUnmarshal64 # -> COMBASE.HICON_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HMENU_UserFree64 # -> COMBASE.HMENU_UserFree64
@ stub -arch=win64 -version=0xA00+ HMENU_UserMarshal64 # -> COMBASE.HMENU_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HMENU_UserSize64 # -> COMBASE.HMENU_UserSize64
@ stub -arch=win64 -version=0xA00+ HMENU_UserUnmarshal64 # -> COMBASE.HMENU_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HMETAFILEPICT_UserFree64
@ stub -arch=win64 -version=0xA00+ HMETAFILEPICT_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HMETAFILEPICT_UserSize64
@ stub -arch=win64 -version=0xA00+ HMETAFILEPICT_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HMETAFILE_UserFree64
@ stub -arch=win64 -version=0xA00+ HMETAFILE_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HMETAFILE_UserSize64
@ stub -arch=win64 -version=0xA00+ HMETAFILE_UserUnmarshal64
@ stub -version=0xA00+ HMONITOR_UserFree # -> COMBASE.HMONITOR_UserFree
@ stub -arch=win64 -version=0xA00+ HMONITOR_UserFree64 # -> COMBASE.HMONITOR_UserFree64
@ stub -version=0xA00+ HMONITOR_UserMarshal # -> COMBASE.HMONITOR_UserMarshal
@ stub -arch=win64 -version=0xA00+ HMONITOR_UserMarshal64 # -> COMBASE.HMONITOR_UserMarshal64
@ stub -version=0xA00+ HMONITOR_UserSize # -> COMBASE.HMONITOR_UserSize
@ stub -arch=win64 -version=0xA00+ HMONITOR_UserSize64 # -> COMBASE.HMONITOR_UserSize64
@ stub -version=0xA00+ HMONITOR_UserUnmarshal # -> COMBASE.HMONITOR_UserUnmarshal
@ stub -arch=win64 -version=0xA00+ HMONITOR_UserUnmarshal64 # -> COMBASE.HMONITOR_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HPALETTE_UserFree64 # -> COMBASE.HPALETTE_UserFree64
@ stub -version=0xA00+ HPALETTE_UserFreeExt
@ stub -arch=win64 -version=0xA00+ HPALETTE_UserMarshal64 # -> COMBASE.HPALETTE_UserMarshal64
@ stub -version=0xA00+ HPALETTE_UserMarshalExt
@ stub -arch=win64 -version=0xA00+ HPALETTE_UserSize64 # -> COMBASE.HPALETTE_UserSize64
@ stub -version=0xA00+ HPALETTE_UserSizeExt
@ stub -arch=win64 -version=0xA00+ HPALETTE_UserUnmarshal64 # -> COMBASE.HPALETTE_UserUnmarshal64
@ stub -version=0xA00+ HPALETTE_UserUnmarshalExt
@ stub -version=0xA00+ HRGN_UserFree # -> COMBASE.HRGN_UserFree
@ stub -version=0xA00+ HRGN_UserMarshal # -> COMBASE.HRGN_UserMarshal
@ stub -version=0xA00+ HRGN_UserSize # -> COMBASE.HRGN_UserSize
@ stub -version=0xA00+ HRGN_UserUnmarshal # -> COMBASE.HRGN_UserUnmarshal
@ stub -arch=win64 -version=0xA00+ HWND_UserFree64 # -> COMBASE.HWND_UserFree64
@ stub -arch=win64 -version=0xA00+ HWND_UserFree64Ext
@ stub -version=0xA00+ HWND_UserFreeExt
@ stub -arch=win64 -version=0xA00+ HWND_UserMarshal64 # -> COMBASE.HWND_UserMarshal64
@ stub -arch=win64 -version=0xA00+ HWND_UserMarshal64Ext
@ stub -version=0xA00+ HWND_UserMarshalExt
@ stub -arch=win64 -version=0xA00+ HWND_UserSize64 # -> COMBASE.HWND_UserSize64
@ stub -arch=win64 -version=0xA00+ HWND_UserSize64Ext
@ stub -version=0xA00+ HWND_UserSizeExt
@ stub -arch=win64 -version=0xA00+ HWND_UserUnmarshal64 # -> COMBASE.HWND_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ HWND_UserUnmarshal64Ext
@ stub -version=0xA00+ HWND_UserUnmarshalExt
@ stub -version=0xA00+ HkOleRegisterObject # -> api-ms-win-core-com-private-l1-1-1.HkOleRegisterObject
@ stub -version=0xA00+ IsRoInitializeASTAAllowedInDesktop
@ stub -version=0xA00+ MonikerLoadTypeLib
@ stub -version=0xA00+ NdrOleInitializeExtension # -> api-ms-win-core-com-private-l1-1-1.NdrOleInitializeExtension
@ stub -version=0xA00+ NdrProxyForwardingFunction10 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction10
@ stub -version=0xA00+ NdrProxyForwardingFunction11 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction11
@ stub -version=0xA00+ NdrProxyForwardingFunction12 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction12
@ stub -version=0xA00+ NdrProxyForwardingFunction13 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction13
@ stub -version=0xA00+ NdrProxyForwardingFunction14 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction14
@ stub -version=0xA00+ NdrProxyForwardingFunction15 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction15
@ stub -version=0xA00+ NdrProxyForwardingFunction16 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction16
@ stub -version=0xA00+ NdrProxyForwardingFunction17 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction17
@ stub -version=0xA00+ NdrProxyForwardingFunction18 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction18
@ stub -version=0xA00+ NdrProxyForwardingFunction19 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction19
@ stub -version=0xA00+ NdrProxyForwardingFunction20 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction20
@ stub -version=0xA00+ NdrProxyForwardingFunction21 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction21
@ stub -version=0xA00+ NdrProxyForwardingFunction22 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction22
@ stub -version=0xA00+ NdrProxyForwardingFunction23 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction23
@ stub -version=0xA00+ NdrProxyForwardingFunction24 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction24
@ stub -version=0xA00+ NdrProxyForwardingFunction25 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction25
@ stub -version=0xA00+ NdrProxyForwardingFunction26 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction26
@ stub -version=0xA00+ NdrProxyForwardingFunction27 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction27
@ stub -version=0xA00+ NdrProxyForwardingFunction28 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction28
@ stub -version=0xA00+ NdrProxyForwardingFunction29 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction29
@ stub -version=0xA00+ NdrProxyForwardingFunction3 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction3
@ stub -version=0xA00+ NdrProxyForwardingFunction30 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction30
@ stub -version=0xA00+ NdrProxyForwardingFunction31 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction31
@ stub -version=0xA00+ NdrProxyForwardingFunction32 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction32
@ stub -version=0xA00+ NdrProxyForwardingFunction4 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction4
@ stub -version=0xA00+ NdrProxyForwardingFunction5 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction5
@ stub -version=0xA00+ NdrProxyForwardingFunction6 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction6
@ stub -version=0xA00+ NdrProxyForwardingFunction7 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction7
@ stub -version=0xA00+ NdrProxyForwardingFunction8 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction8
@ stub -version=0xA00+ NdrProxyForwardingFunction9 # -> api-ms-win-core-com-midlproxystub-l1-1-0.NdrProxyForwardingFunction9
@ stub -version=0xA00+ ObjectStublessClient10 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient10
@ stub -version=0xA00+ ObjectStublessClient11 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient11
@ stub -version=0xA00+ ObjectStublessClient12 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient12
@ stub -version=0xA00+ ObjectStublessClient13 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient13
@ stub -version=0xA00+ ObjectStublessClient14 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient14
@ stub -version=0xA00+ ObjectStublessClient15 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient15
@ stub -version=0xA00+ ObjectStublessClient16 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient16
@ stub -version=0xA00+ ObjectStublessClient17 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient17
@ stub -version=0xA00+ ObjectStublessClient18 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient18
@ stub -version=0xA00+ ObjectStublessClient19 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient19
@ stub -version=0xA00+ ObjectStublessClient20 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient20
@ stub -version=0xA00+ ObjectStublessClient21 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient21
@ stub -version=0xA00+ ObjectStublessClient22 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient22
@ stub -version=0xA00+ ObjectStublessClient23 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient23
@ stub -version=0xA00+ ObjectStublessClient24 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient24
@ stub -version=0xA00+ ObjectStublessClient25 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient25
@ stub -version=0xA00+ ObjectStublessClient26 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient26
@ stub -version=0xA00+ ObjectStublessClient27 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient27
@ stub -version=0xA00+ ObjectStublessClient28 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient28
@ stub -version=0xA00+ ObjectStublessClient29 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient29
@ stub -version=0xA00+ ObjectStublessClient3 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient3
@ stub -version=0xA00+ ObjectStublessClient30 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient30
@ stub -version=0xA00+ ObjectStublessClient31 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient31
@ stub -version=0xA00+ ObjectStublessClient32 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient32
@ stub -version=0xA00+ ObjectStublessClient4 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient4
@ stub -version=0xA00+ ObjectStublessClient5 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient5
@ stub -version=0xA00+ ObjectStublessClient6 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient6
@ stub -version=0xA00+ ObjectStublessClient7 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient7
@ stub -version=0xA00+ ObjectStublessClient8 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient8
@ stub -version=0xA00+ ObjectStublessClient9 # -> api-ms-win-core-com-midlproxystub-l1-1-0.ObjectStublessClient9
@ stub -version=0xA00+ Ole32DllGetClassObject
@ stub -version=0xA00+ OleConvertOLESTREAMToIStorage2
@ stub -version=0xA00+ OleConvertOLESTREAMToIStorageEx2
@ stub -version=0xA00+ OleCreateFontIndirectExt
@ stub -version=0xA00+ OleCreatePictureIndirectExt
@ stub -version=0xA00+ OleCreatePropertyFrameIndirectExt
@ stub -version=0xA00+ OleGetClipboardWithEnterpriseInfo
@ stub -version=0xA00+ OleGetPackageClipboardOwner
@ stub -version=0xA00+ OleIconToCursorExt
@ stub -version=0xA00+ OleLoadPictureExt
@ stub -version=0xA00+ OleLoadPictureFileExt
@ stub -version=0xA00+ OleLoadPicturePathExt
@ stub -version=0xA00+ OleReleaseEnumVerbCache
@ stub -version=0xA00+ OleSavePictureFileExt
@ stub -version=0xA00+ OleTranslateColorExt
@ stub -version=0xA00+ PropVariantChangeType
@ stub -version=0xA00+ ReadStorageProperties
@ stub -version=0xA00+ RegisterActiveObjectExt
@ stub -version=0xA00+ RevokeActiveObjectExt
@ stub -version=0xA00+ RoGetAgileReference # -> api-ms-win-core-com-l1-1-1.RoGetAgileReference
@ stub -arch=win64 -version=0xA00+ SNB_UserFree64
@ stub -arch=win64 -version=0xA00+ SNB_UserMarshal64
@ stub -arch=win64 -version=0xA00+ SNB_UserSize64
@ stub -arch=win64 -version=0xA00+ SNB_UserUnmarshal64
@ stub -arch=win64 -version=0xA00+ STGMEDIUM_UserFree64
@ stub -version=0xA00+ STGMEDIUM_UserFreeExt
@ stub -arch=win64 -version=0xA00+ STGMEDIUM_UserMarshal64
@ stub -version=0xA00+ STGMEDIUM_UserMarshalExt
@ stub -arch=win64 -version=0xA00+ STGMEDIUM_UserSize64
@ stub -version=0xA00+ STGMEDIUM_UserSizeExt
@ stub -arch=win64 -version=0xA00+ STGMEDIUM_UserUnmarshal64
@ stub -version=0xA00+ STGMEDIUM_UserUnmarshalExt
@ stub -version=0xA00+ SetOleautModule
@ stub -version=0xA00+ SetWOWThunkGlobalPtr
@ stub -version=0xA00+ StdTypesGetClassObject
@ stub -version=0xA00+ StdTypesRegisterServer
@ stub -version=0xA00+ StgOpenStorageOnHandle
@ stub -version=0xA00+ StgPropertyLengthAsVariant
@ stub -arch=win32 -version=0xA00+ StoreQITableEntry
@ stub -version=0xA00+ UpdateProcessTracing # -> api-ms-win-core-com-private-l1-1-1.UpdateProcessTracing
@ stub -arch=win64 -version=0xA00+ WdtpInterfacePointer_UserFree64 # -> COMBASE.WdtpInterfacePointer_UserFree64
@ stub -arch=win64 -version=0xA00+ WdtpInterfacePointer_UserMarshal64 # -> COMBASE.WdtpInterfacePointer_UserMarshal64
@ stub -arch=win64 -version=0xA00+ WdtpInterfacePointer_UserSize64 # -> COMBASE.WdtpInterfacePointer_UserSize64
@ stub -arch=win64 -version=0xA00+ WdtpInterfacePointer_UserUnmarshal64 # -> COMBASE.WdtpInterfacePointer_UserUnmarshal64
@ stub -version=0xA00+ WriteStorageProperties
# END win10-surface stubs
