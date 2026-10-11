
@ stdcall LogonIdFromWinStationNameA(ptr ptr ptr)
@ stdcall LogonIdFromWinStationNameW(ptr ptr ptr)
@ stdcall RemoteAssistancePrepareSystemRestore(ptr)
@ stdcall ServerGetInternetConnectorStatus(ptr ptr ptr)
@ stdcall ServerLicensingClose(ptr)
@ stdcall ServerLicensingDeactivateCurrentPolicy(ptr)
@ stdcall ServerLicensingFreePolicyInformation(ptr)
@ stdcall ServerLicensingGetAvailablePolicyIds(ptr ptr ptr)
@ stdcall ServerLicensingGetPolicy(ptr ptr)
@ stdcall ServerLicensingGetPolicyInformationA(ptr ptr ptr ptr)
@ stdcall ServerLicensingGetPolicyInformationW(ptr ptr ptr ptr)
@ stdcall ServerLicensingLoadPolicy(ptr ptr)
@ stdcall ServerLicensingOpenA(ptr)
@ stdcall ServerLicensingOpenW(ptr)
@ stdcall ServerLicensingSetPolicy(ptr ptr ptr)
@ stdcall ServerLicensingUnloadPolicy(ptr ptr)
@ stdcall ServerQueryInetConnectorInformationA(ptr ptr ptr ptr)
@ stdcall ServerQueryInetConnectorInformationW(ptr ptr ptr ptr)
@ stdcall ServerSetInternetConnectorStatus(ptr ptr ptr)
@ stdcall WinStationActivateLicense(ptr ptr ptr ptr)
@ stdcall WinStationAutoReconnect(ptr)
@ stdcall WinStationBroadcastSystemMessage(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationCanLogonProceed(ptr ptr ptr)
@ stdcall WinStationCheckAccess(ptr ptr ptr)
@ stdcall WinStationCheckLoopBack(ptr ptr ptr ptr)
@ stdcall WinStationCloseServer(ptr ptr ptr ptr)
@ stdcall WinStationConnectA(ptr)
@ stdcall WinStationConnectCallback(ptr ptr ptr ptr ptr)
@ stdcall WinStationConnectEx(ptr ptr)
@ stdcall WinStationConnectW(ptr ptr ptr ptr ptr)
@ stdcall WinStationDisconnect(ptr ptr ptr)
@ stdcall WinStationDynVirtualChanRead(ptr ptr ptr ptr ptr)
@ stdcall WinStationDynVirtualChanWrite(ptr ptr ptr ptr)
@ stdcall WinStationEnumerateA(ptr ptr ptr)
@ stdcall WinStationEnumerateLicenses(ptr ptr ptr)
@ stdcall WinStationEnumerateProcesses(ptr ptr)
@ stdcall WinStationEnumerateW(ptr ptr ptr)
@ stdcall WinStationEnumerate_IndexedA(ptr ptr ptr ptr ptr)
@ stdcall WinStationEnumerate_IndexedW(ptr ptr ptr ptr ptr)
@ stdcall WinStationFreeGAPMemory(ptr ptr ptr)
@ stdcall WinStationFreeMemory(ptr)
@ stdcall WinStationFreeUserCertificates(ptr)
@ stdcall WinStationFreeUserCredentials(ptr)
@ stdcall WinStationGenerateLicense(ptr ptr ptr ptr)
@ stdcall WinStationGetAllProcesses(ptr ptr ptr ptr)
@ stdcall WinStationGetConnectionProperty(ptr ptr ptr)
@ stdcall WinStationGetInitialApplication(ptr ptr ptr ptr ptr)
@ stdcall WinStationGetLanAdapterNameA(ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationGetLanAdapterNameW(ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationGetLoggedOnCount(ptr ptr)
@ stdcall WinStationGetMachinePolicy(ptr ptr)
@ stdcall WinStationGetProcessSid(ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationGetTermSrvCountersValue(ptr ptr ptr)
@ stdcall WinStationGetUserCertificates(ptr)
@ stdcall WinStationGetUserCredentials(ptr)
@ stdcall WinStationGetUserProfile(ptr ptr ptr ptr)
@ stdcall WinStationInstallLicense(ptr ptr ptr)
@ stdcall WinStationIsHelpAssistantSession(ptr ptr)
@ stdcall WinStationIsSessionPermitted()
@ stdcall WinStationNameFromLogonIdA(ptr ptr ptr)
@ stdcall WinStationNameFromLogonIdW(ptr ptr ptr)
@ stdcall WinStationNtsdDebug(ptr ptr ptr ptr ptr)
@ stdcall WinStationOpenServerA(ptr)
@ stdcall WinStationOpenServerW(ptr)
@ stdcall WinStationQueryAllowConcurrentConnections()
@ stdcall WinStationQueryEnforcementCore(ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationQueryInformationA(ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationQueryInformationW(ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationQueryLicense(ptr ptr ptr)
@ stdcall WinStationQueryLogonCredentialsW(ptr)
@ stdcall WinStationQueryUpdateRequired(ptr ptr)
@ stdcall WinStationRedirectErrorMessage(ptr ptr)
@ stdcall WinStationRegisterConsoleNotification(ptr ptr ptr)
@ stdcall WinStationRegisterConsoleNotificationEx(ptr ptr ptr ptr)
@ stdcall WinStationRegisterNotificationEvent(ptr ptr ptr ptr)
@ stdcall WinStationRemoveLicense(ptr ptr ptr)
@ stdcall WinStationRenameA(ptr ptr ptr)
@ stdcall WinStationRenameW(ptr ptr ptr)
@ stdcall WinStationRequestSessionsList(ptr ptr ptr)
@ stdcall WinStationReset(ptr ptr ptr)
@ stdcall WinStationSendMessageA(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationSendMessageW(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationSendWindowMessage(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall WinStationServerPing(ptr)
@ stdcall WinStationSetInformationA(ptr ptr ptr ptr ptr)
@ stdcall WinStationSetInformationW(ptr ptr ptr ptr ptr)
@ stdcall WinStationSetPoolCount(ptr ptr ptr)
@ stdcall WinStationShadow(ptr ptr ptr ptr ptr)
@ stdcall WinStationShadowStop(ptr ptr ptr)
@ stdcall WinStationShutdownSystem(ptr ptr)
@ stdcall WinStationSwitchToServicesSession()
@ stdcall WinStationSystemShutdownStarted()
@ stdcall WinStationSystemShutdownWait(ptr ptr)
@ stdcall WinStationTerminateProcess(ptr ptr ptr)
@ stdcall WinStationUnRegisterConsoleNotification(ptr ptr)
@ stdcall WinStationUnRegisterNotificationEvent(ptr)
@ stdcall WinStationUserLoginAccessCheck(ptr ptr ptr ptr)
@ stdcall WinStationVirtualOpen(ptr ptr ptr)
@ stdcall WinStationVirtualOpenEx(ptr ptr ptr ptr)
@ stdcall WinStationWaitSystemEvent(ptr ptr ptr)
@ stdcall _NWLogonQueryAdmin(ptr ptr ptr)
@ stdcall _NWLogonSetAdmin(ptr ptr ptr)
@ stdcall _WinStationAnnoyancePopup(ptr ptr)
@ stdcall _WinStationBeepOpen(ptr ptr ptr)
@ stdcall _WinStationBreakPoint(ptr ptr ptr)
@ stdcall _WinStationCallback(ptr ptr ptr)
@ stdcall _WinStationCheckForApplicationName(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall _WinStationFUSCanRemoteUserDisconnect(ptr ptr ptr)
@ stdcall _WinStationGetApplicationInfo(ptr ptr ptr ptr)
@ stdcall _WinStationNotifyDisconnectPipe()
@ stdcall _WinStationNotifyLogoff()
@ stdcall _WinStationNotifyLogon(ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall _WinStationNotifyNewSession(ptr ptr)
@ stdcall _WinStationOpenSessionDirectory(ptr ptr)
@ stdcall _WinStationReInitializeSecurity(ptr)
@ stdcall _WinStationReadRegistry(ptr)
@ stdcall _WinStationSessionInitialized()
@ stdcall _WinStationShadowTarget(ptr ptr ptr ptr ptr ptr ptr ptr ptr ptr)
@ stdcall _WinStationShadowTargetSetup(ptr ptr)
@ stdcall _WinStationUpdateClientCachedCredentials(ptr ptr ptr ptr ptr ptr ptr)
@ stdcall _WinStationUpdateSettings(ptr ptr ptr)
@ stdcall _WinStationUpdateUserConfig(ptr)
@ stdcall _WinStationWaitForConnect()


# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ ServerLicensingGetAadInfo
@ stub -version=0xA00+ ServerLicensingSetAadInfo
@ stub -version=0xA00+ WTSRegisterSessionNotificationEx
@ stub -version=0xA00+ WTSUnRegisterSessionNotificationEx
@ stub -version=0xA00+ WinStationActiveSessionExists
@ stub -version=0xA00+ WinStationConnectAndLockDesktop
@ stub -version=0xA00+ WinStationConsumeCacheSession
@ stub -version=0xA00+ WinStationCreateAgentSessionTransport
@ stub -version=0xA00+ WinStationCreateChildSessionTransport
@ stub -version=0xA00+ WinStationEnableChildSessions
@ stub -version=0xA00+ WinStationEnumerateContainerSessions
@ stub -version=0xA00+ WinStationEnumerateExW
@ stub -version=0xA00+ WinStationFreeConsoleNotification
@ stub -version=0xA00+ WinStationFreeEXECENVDATAEX
@ stub -version=0xA00+ WinStationFreePropertyValue
@ stub -version=0xA00+ WinStationFreeSessionNotification
@ stub -version=0xA00+ WinStationFreeUserSessionInfo
@ stub -version=0xA00+ WinStationGetAllSessionsEx
@ stub -version=0xA00+ WinStationGetAllSessionsW
@ stub -version=0xA00+ WinStationGetAllUserSessions
@ stub -version=0xA00+ WinStationGetChildSessionId
@ stub -version=0xA00+ WinStationGetCurrentSessionCapabilities
@ stub -version=0xA00+ WinStationGetCurrentSessionConnectionProperty
@ stub -version=0xA00+ WinStationGetCurrentSessionTerminalName
@ stub -version=0xA00+ WinStationGetDeviceId
@ stub -version=0xA00+ WinStationGetLastWinlogonNotification
@ stub -version=0xA00+ WinStationGetParentSessionId
@ stub -version=0xA00+ WinStationGetRedirectAuthInfo
@ stub -version=0xA00+ WinStationGetRestrictedLogonInfo
@ stub -version=0xA00+ WinStationGetSessionIds
@ stub -version=0xA00+ WinStationGetSpecialSessionFlags
@ stub -version=0xA00+ WinStationIsBoundToCacheTerminal
@ stub -version=0xA00+ WinStationIsChildSessionsEnabled
@ stub -version=0xA00+ WinStationIsCurrentSessionRemoteable
@ stub -version=0xA00+ WinStationIsSessionRemoteable
@ stub -version=0xA00+ WinStationNegotiateSession
@ stub -version=0xA00+ WinStationOpenServerExA
@ stub -version=0xA00+ WinStationOpenServerExW
@ stub -version=0xA00+ WinStationPreCreateGlassReplacementSession
@ stub -version=0xA00+ WinStationPreCreateGlassReplacementSessionEx
@ stub -version=0xA00+ WinStationQueryCurrentSessionInformation
@ stub -version=0xA00+ WinStationQuerySessionVirtualIP
@ stub -version=0xA00+ WinStationRcmShadow2
@ stub -version=0xA00+ WinStationRedirectLogonBeginPainting
@ stub -version=0xA00+ WinStationRedirectLogonError
@ stub -version=0xA00+ WinStationRedirectLogonMessage
@ stub -version=0xA00+ WinStationRedirectLogonStatus
@ stub -version=0xA00+ WinStationRedirectShellExecute
@ stub -version=0xA00+ WinStationRegisterConsoleNotificationEx2
@ stub -version=0xA00+ WinStationRegisterCurrentSessionNotificationEvent
@ stub -version=0xA00+ WinStationRegisterSessionNotification
@ stub -version=0xA00+ WinStationRegisterSessionNotificationEx
@ stub -version=0xA00+ WinStationReportLoggedOnCompleted
@ stub -version=0xA00+ WinStationReportUIResult
@ stub -version=0xA00+ WinStationRevertFromServicesSession
@ stub -version=0xA00+ WinStationSetAutologonPassword
@ stub -version=0xA00+ WinStationSetLastWinlogonNotification
@ stub -version=0xA00+ WinStationSetRenderHint
@ stub -version=0xA00+ WinStationShadowAccessCheck
@ stub -version=0xA00+ WinStationShadowStop2
@ stub -version=0xA00+ WinStationTerminateGlassReplacementSession
@ stub -version=0xA00+ WinStationUnRegisterSessionNotification
@ stub -version=0xA00+ WinStationVerify
@ stub -version=0xA00+ _WinStationShadowTarget2
@ stub -version=0xA00+ _WinStationWaitForConnectEx
# END win10-surface stubs
