# Up until Windows 2000 these APIs have hardcoded ordinals.
# Keep it that way for compatibility.
1 stub CDLGetLongPathNameA
2 stub CDLGetLongPathNameW
# IsJITInProgress has a hardcoded ordinal on WinME and Windows 2000
#3 stub IsJITInProgress

@ stub AsyncGetClassBits
@ stdcall AsyncInstallDistributionUnit(wstr wstr wstr long long wstr ptr ptr long)
@ stdcall BindAsyncMoniker(ptr long ptr ptr ptr)
@ stdcall CoGetClassObjectFromURL(ptr wstr long long wstr ptr long ptr ptr ptr)
@ stub CoInstall
@ stdcall CoInternetCombineUrl(wstr wstr long ptr long ptr long)
@ stdcall CoInternetCombineUrlEx(ptr wstr long ptr long)
@ stdcall CoInternetCompareUrl(wstr wstr long)
@ stdcall CoInternetCombineIUri(ptr ptr long ptr long)
@ stdcall CoInternetCreateSecurityManager(ptr ptr long)
@ stdcall CoInternetCreateZoneManager(ptr ptr long)
@ stub CoInternetGetProtocolFlags
@ stdcall CoInternetGetSecurityUrl(wstr ptr long long)
@ stdcall CoInternetGetSecurityUrlEx(ptr ptr long long)
@ stdcall CoInternetGetSession(long ptr long)
@ stdcall CoInternetIsFeatureEnabled(long long)
@ stdcall CoInternetIsFeatureEnabledForUrl(long long wstr ptr)
@ stdcall CoInternetIsFeatureZoneElevationEnabled(wstr wstr ptr long)
@ stdcall CoInternetParseUrl(wstr long long ptr long ptr long)
@ stdcall CoInternetParseIUri(ptr long long ptr long ptr long)
@ stdcall CoInternetQueryInfo(wstr long long ptr long ptr long)
@ stdcall CoInternetSetFeatureEnabled(long long long)
@ stdcall CompareSecurityIds(ptr long ptr long long)
@ stdcall CopyBindInfo(ptr ptr)
@ stdcall CopyStgMedium(ptr ptr)
@ stdcall CreateAsyncBindCtx(long ptr ptr ptr)
@ stdcall CreateAsyncBindCtxEx(ptr long ptr ptr ptr long)
@ stdcall CreateFormatEnumerator(long ptr ptr)
@ stdcall CreateIUriBuilder(ptr long long ptr)
@ stdcall CreateUri(wstr long long ptr)
@ stdcall CreateUriWithFragment(wstr wstr long long ptr)
@ stdcall CreateURLMoniker(ptr wstr ptr)
@ stdcall CreateURLMonikerEx(ptr wstr ptr long)
@ stdcall CreateURLMonikerEx2(ptr ptr ptr long)
@ stdcall -private DllCanUnloadNow()
@ stdcall -private DllGetClassObject(ptr ptr ptr)
@ stdcall -private DllInstall(long wstr)
@ stdcall -private DllRegisterServer()
@ stdcall -private DllRegisterServerEx()
@ stdcall -private DllUnregisterServer()
@ stdcall Extract(ptr str)
@ stdcall FaultInIEFeature(long ptr ptr long)
@ stub FindMediaType
@ stub FindMediaTypeClass
@ stdcall FindMimeFromData(ptr wstr ptr long wstr long ptr long)
@ stdcall GetClassFileOrMime(ptr wstr ptr long wstr long ptr)
@ stub GetClassURL
@ stub GetComponentIDFromCLSSPEC
@ stdcall GetIUriPriv(ptr ptr)
@ stub GetMarkOfTheWeb
@ stdcall GetSoftwareUpdateInfo(wstr ptr)
@ stub HlinkGoBack
@ stub HlinkGoForward
@ stub HlinkNavigateMoniker
@ stdcall HlinkNavigateString(ptr wstr)
@ stdcall HlinkSimpleNavigateToMoniker(ptr wstr wstr ptr ptr ptr long long)
@ stdcall HlinkSimpleNavigateToString(wstr wstr wstr ptr ptr ptr long long)
@ stub IEInstallScope
@ stdcall IsAsyncMoniker(ptr)
@ stdcall IsLoggingEnabledA(str)
@ stdcall IsLoggingEnabledW(wstr)
@ stdcall IsValidURL(ptr wstr long)
@ stdcall MkParseDisplayNameEx(ptr wstr ptr ptr)
@ stdcall ObtainUserAgentString(long str ptr)
@ stub PrivateCoInstall
@ stdcall RegisterBindStatusCallback(ptr ptr ptr long)
@ stdcall RegisterFormatEnumerator(ptr ptr long)
@ stub RegisterMediaTypeClass
@ stdcall RegisterMediaTypes(long ptr ptr)
@ stdcall ReleaseBindInfo(ptr)
@ stdcall RevokeBindStatusCallback(ptr ptr)
@ stdcall RevokeFormatEnumerator(ptr ptr)
@ stub SetSoftwareUpdateAdvertisementState
@ stdcall ShouldShowIntranetWarningSecband(long)
@ stub URLDownloadA
@ stdcall URLDownloadToCacheFileA(ptr str str long long ptr)
@ stdcall URLDownloadToCacheFileW(ptr wstr wstr long long ptr)
@ stdcall URLDownloadToFileA(ptr str str long ptr)
@ stdcall URLDownloadToFileW(ptr wstr wstr long ptr)
@ stub URLDownloadW
@ stdcall URLOpenBlockingStreamA(ptr str ptr long ptr)
@ stdcall URLOpenBlockingStreamW(ptr wstr ptr long ptr)
@ stub URLOpenPullStreamA
@ stdcall URLOpenPullStreamW(ptr wstr long ptr)
@ stdcall URLOpenStreamA(ptr str long ptr)
@ stdcall URLOpenStreamW(ptr wstr long ptr)
@ stub UrlMkBuildVersion
@ stdcall UrlMkGetSessionOption(long ptr long ptr long)
@ stdcall UrlMkSetSessionOption(long ptr long long)
@ stub WriteHitLogging
@ stub ZonesReInit

108 stdcall @() IsInternetESCEnabledLocal
111 stdcall @(wstr) IsProtectedModeURL
328 stdcall @(ptr ptr) propsys.VariantCompare
329 stdcall @(ptr ptr) propsys.VariantToGUID
331 stdcall @(ptr long ptr) propsys.InitPropVariantFromBuffer
335 stdcall @(ptr long ptr) propsys.InitVariantFromBuffer
350 stdcall @(ptr ptr) propsys.PropVariantToGUID
362 stdcall @(ptr ptr) propsys.InitVariantFromGUIDAsString
363 stdcall @(long long ptr) propsys.InitVariantFromResource
387 stdcall @(ptr long) propsys.VariantToUInt32WithDefault
410 stdcall @(long long) LogSqmBits
414 stdcall @(long long) LogSqmIncrement
423 stdcall @(long long long long) LogSqmUXCommandOffsetInternal
444 stdcall @(long long long) MapUriToBrowserEmulationState
445 stdcall -noname MapBrowserEmulationModeToUserAgent(ptr ptr)
446 stdcall @(long) CoInternetGetBrowserProfile
455 stdcall @() FlushUrlmonZonesCache

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ CAuthenticateHostUI_CreateInstance
@ stub -version=0xA00+ CORPolicyProvider
@ stub -version=0xA00+ CoInternetCanonicalizeIUri
@ stub -version=0xA00+ CoInternetFeatureSettingsChanged
@ stub -version=0xA00+ CoInternetGetMobileBrowserAppCompatMode
@ stub -version=0xA00+ CoInternetGetMobileBrowserForceDesktopMode
@ stub -version=0xA00+ CoInternetIsFeatureEnabledForIUri
@ stub -version=0xA00+ CoInternetSetMobileBrowserAppCompatMode
@ stub -version=0xA00+ CoInternetSetMobileBrowserForceDesktopMode
@ stub -version=0xA00+ CompatFlagsFromClsid
@ stub -version=0xA00+ CreateUriFromMultiByteString
@ stub -version=0xA00+ CreateUriPriv
@ stub -version=0xA00+ FileBearsMarkOfTheWeb
@ stub -version=0xA00+ GetAddSitesFileUrl
@ stub -version=0xA00+ GetIDNFlagsForUri
@ stub -version=0xA00+ GetIUriPriv2
@ stub -version=0xA00+ GetLabelsFromNamedHost
@ stub -version=0xA00+ GetPortFromUrlScheme
@ stub -version=0xA00+ GetPropertyFromName
@ stub -version=0xA00+ GetPropertyName
@ stub -version=0xA00+ GetUrlmonThreadNotificationHwnd
@ stub -version=0xA00+ GetZoneFromAlternateDataStreamEx
@ stub -version=0xA00+ IECompatLogCSSFix
@ stub -version=0xA00+ IEGetUserPrivateNamespaceName
@ stub -version=0xA00+ IntlPercentEncodeNormalize
@ stub -version=0xA00+ IsDWORDProperty
@ stub -version=0xA00+ IsIntranetAvailable
@ stub -version=0xA00+ IsJITInProgress
@ stub -version=0xA00+ IsStringProperty
@ stub -version=0xA00+ QueryAssociations
@ stub -version=0xA00+ QueryClsidAssociation
@ stub -version=0xA00+ RegisterWebPlatformPermanentSecurityManager
@ stub -version=0xA00+ RestrictHTTP2
@ stub -version=0xA00+ SetAccessForIEAppContainer
@ stub -version=0xA00+ ShouldDisplayPunycodeForUri
@ stub -version=0xA00+ ShowTrustAlertDialog
@ stub -version=0xA00+ UnregisterWebPlatformPermanentSecurityManager
@ stub -version=0xA00+ UrlmonCleanupCurrentThread
# END win10-surface stubs
