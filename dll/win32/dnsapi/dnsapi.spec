@ stub BreakRecordsIntoBlob
@ stub CombineRecordsInBlob
@ stdcall DnsAcquireContextHandle_A(long ptr ptr)
@ stdcall DnsAcquireContextHandle_UTF8(long ptr ptr)
@ stdcall DnsAcquireContextHandle_W(long ptr ptr)
@ stdcall DnsAddRecordSet_A()
@ stdcall DnsAddRecordSet_UTF8()
@ stdcall DnsAddRecordSet_W()
@ stdcall DnsAllocateRecord()
@ stdcall DnsApiAlloc(long)
@ stdcall DnsApiHeapReset()
@ stdcall DnsApiFree(ptr)
@ stub DnsApiRealloc
@ stub DnsApiSetDebugGlobals
@ stdcall DnsAsyncRegisterHostAddrs_A()
@ stdcall DnsAsyncRegisterHostAddrs_UTF8()
@ stdcall DnsAsyncRegisterHostAddrs_W()
@ stdcall DnsAsyncRegisterInit()
@ stdcall DnsAsyncRegisterTerm()
@ stdcall DnsCheckNameCollision_A()
@ stdcall DnsCheckNameCollision_UTF8()
@ stdcall DnsCheckNameCollision_W()
@ stdcall DnsCopyStringEx()
@ stdcall DnsCreateReverseNameStringForIpAddress()
@ stdcall DnsCreateStandardDnsNameCopy()
@ stdcall DnsCreateStringCopy()
@ stdcall DnsDhcpSrvRegisterHostName_W()
@ stdcall DnsDhcpSrvRegisterInit()
@ stdcall DnsDhcpSrvRegisterTerm()
@ stdcall DnsDisableAdapterDomainNameRegistration()
@ stdcall DnsDisableBNodeResolverThread()
@ stdcall DnsDisableDynamicRegistration()
@ stdcall DnsDowncaseDnsNameLabel()
@ stdcall DnsEnableAdapterDomainNameRegistration()
@ stdcall DnsEnableBNodeResolverThread()
@ stdcall DnsEnableDynamicRegistration()
@ stdcall DnsExtractRecordsFromMessage_UTF8(ptr long ptr)
@ stdcall DnsExtractRecordsFromMessage_W(ptr long ptr)
@ stdcall DnsFindAuthoritativeZone()
@ stdcall DnsFlushResolverCache()
@ stdcall DnsFlushResolverCacheEntry_A(str)
@ stdcall DnsFlushResolverCacheEntry_UTF8(str)
@ stdcall DnsFlushResolverCacheEntry_W(wstr)
@ stdcall DnsFree(ptr long)
@ stdcall DnsFreeAdapterInformation()
@ stub DnsFreeConfigStructure
@ stdcall DnsFreeNetworkInformation()
@ stdcall DnsFreeSearchInformation()
@ stdcall DnsGetBufferLengthForStringCopy()
@ stdcall DnsGetCacheDataTable(ptr)
@ stdcall DnsGetDnsServerList()
@ stdcall DnsGetDomainName()
@ stdcall DnsGetHostName_A()
@ stdcall DnsGetHostName_UTF8()
@ stdcall DnsGetHostName_W()
@ stdcall DnsGetIpAddressInfoList()
@ stdcall DnsGetIpAddressList()
@ stdcall DnsGetLastServerUpdateIP()
@ stdcall DnsGetMaxNumberOfAddressesToRegister()
@ stdcall DnsGetNetworkInformation()
@ stdcall DnsGetPrimaryDomainName_A()
@ stdcall DnsGetPrimaryDomainName_UTF8()
@ stdcall DnsGetPrimaryDomainName_W()
@ stdcall DnsGetSearchInformation()
@ stub DnsGlobals
@ stdcall DnsIpv6AddressToString()
@ stdcall DnsIpv6StringToAddress()
@ stdcall DnsIsAdapterDomainNameRegistrationEnabled()
@ stdcall DnsIsAMailboxType()
@ stdcall DnsIsDynamicRegistrationEnabled()
@ stdcall DnsIsStatusRcode()
@ stdcall DnsIsStringCountValidForTextType()
@ stdcall DnsMapRcodeToStatus()
@ stdcall DnsModifyRecordSet_A()
@ stdcall DnsModifyRecordSet_UTF8()
@ stdcall DnsModifyRecordSet_W()
@ stdcall DnsModifyRecordsInSet_A(ptr ptr long ptr ptr ptr)
@ stdcall DnsModifyRecordsInSet_UTF8(ptr ptr long ptr ptr ptr)
@ stdcall DnsModifyRecordsInSet_W(ptr ptr long ptr ptr ptr)
@ stdcall DnsNameCompareEx_A()
@ stdcall DnsNameCompareEx_UTF8()
@ stdcall DnsNameCompareEx_W()
@ stdcall DnsNameCompare_A(str str)
@ stdcall DnsNameCompare_W(wstr wstr)
@ stdcall DnsNameCopy()
@ stdcall DnsNameCopyAllocate()
@ stdcall DnsNotifyResolver()
@ stub DnsNotifyResolverClusterIp
@ stub DnsNotifyResolverEx
@ stdcall DnsQueryConfig(long long wstr ptr ptr ptr)
@ stdcall DnsQueryEx()
@ stdcall DnsQuery_A(str long long ptr ptr ptr)
@ stdcall DnsQuery_UTF8(str long long ptr ptr ptr)
@ stdcall DnsQuery_W(wstr long long ptr ptr ptr)
@ stdcall DnsRecordBuild_UTF8()
@ stdcall DnsRecordBuild_W()
@ stdcall DnsRecordCompare(ptr ptr)
@ stdcall DnsRecordCopyEx(ptr long long)
@ stdcall DnsRecordListFree(ptr long)
@ stdcall DnsRecordSetCompare(ptr ptr ptr ptr)
@ stdcall DnsRecordSetCopyEx(ptr long long)
@ stdcall DnsRecordSetDetach(ptr)
@ stdcall DnsRecordStringForType()
@ stdcall DnsRecordStringForWritableType()
@ stdcall DnsRecordTypeForName()
@ stub DnsRegisterClusterAddress
@ stdcall DnsRelationalCompare_UTF8()
@ stdcall DnsRelationalCompare_W()
@ stdcall DnsReleaseContextHandle(ptr)
@ stdcall DnsRemoveRegistrations()
@ stdcall DnsReplaceRecordSetA(ptr long ptr ptr ptr)
@ stdcall DnsReplaceRecordSet_A()
@ stdcall DnsReplaceRecordSetUTF8(ptr long ptr ptr ptr)
@ stdcall DnsReplaceRecordSet_UTF8()
@ stdcall DnsReplaceRecordSetW(ptr long ptr ptr ptr)
@ stdcall DnsReplaceRecordSet_W()
@ stdcall DnsServiceNotificationDeregister_A()
@ stdcall DnsServiceNotificationDeregister_UTF8()
@ stdcall DnsServiceNotificationDeregister_W()
@ stdcall DnsServiceNotificationRegister_A()
@ stdcall DnsServiceNotificationRegister_UTF8()
@ stdcall DnsServiceNotificationRegister_W()
@ stdcall DnsSetMaxNumberOfAddressesToRegister()
@ stdcall DnsStatusString()
@ stdcall DnsStringCopyAllocateEx()
@ stdcall DnsUnicodeToUtf8()
@ stdcall DnsUpdate()
@ stdcall DnsUpdateTest_A()
@ stdcall DnsUpdateTest_UTF8()
@ stdcall DnsUpdateTest_W()
@ stdcall DnsUtf8ToUnicode()
@ stdcall DnsValidateName_A(str long)
@ stdcall DnsValidateName_UTF8(str long)
@ stdcall DnsValidateName_W(wstr long)
@ stdcall DnsValidateUtf8Byte()
@ stdcall DnsWinsRecordFlagForString()
@ stdcall DnsWinsRecordFlagString()
@ stdcall DnsWriteQuestionToBuffer_UTF8(ptr ptr str long long long)
@ stdcall DnsWriteQuestionToBuffer_W(ptr ptr wstr long long long)
@ stdcall DnsWriteReverseNameStringForIpAddress()
@ stub Dns_AddRecordsToMessage
@ stub Dns_AllocateMsgBuf
@ stub Dns_BuildPacket
@ stub Dns_CacheSocketCleanup
@ stub Dns_CacheSocketInit
@ stub Dns_CleanupWinsock
@ stub Dns_CloseConnection
@ stub Dns_CloseHostFile
@ stub Dns_CloseSocket
@ stub Dns_CreateMulticastSocket
@ stub Dns_CreateSocket
@ stub Dns_CreateSocketEx
@ stub Dns_FindAuthoritativeZoneLib
@ stub Dns_GetIpAddresses
@ stub Dns_GetLocalIpAddressArray
@ stub Dns_GetRandomXid
@ stub Dns_InitializeMsgRemoteSockaddr
@ stub Dns_InitializeWinsock
@ stub Dns_InitQueryTimeouts
@ stub Dns_OpenHostFile
@ stub Dns_OpenTcpConnectionAndSend
@ stub Dns_ParseMessage
@ stub Dns_ParsePacketRecord
@ stub Dns_PingAdapterServers
@ stub Dns_ReadHostFileLine
@ stub Dns_ReadPacketName
@ stub Dns_ReadPacketNameAllocate
@ stub Dns_ReadRecordStructureFromPacket
@ stub Dns_RecvTcp
@ stub Dns_ResetNetworkInfo
@ stub Dns_SendAndRecvUdp
@ stub Dns_SendEx
@ stub Dns_SetRecordDatalength
@ stub Dns_SkipPacketName
@ stub Dns_SkipToRecord
@ stub Dns_UpdateLib
@ stub Dns_UpdateLibEx
@ stub Dns_WriteDottedNameToPacket
@ stub Dns_WriteQuestionToMessage
@ stub Dns_WriteRecordStructureToPacketEx
@ stdcall GetCurrentTimeInSeconds()
@ stub GetRecordsForLocalName
@ stub NetInfo_Build
@ stub NetInfo_Clean
@ stub NetInfo_Copy
@ stub NetInfo_Free
@ stub NetInfo_IsForUpdate
@ stub NetInfo_ResetServerPriorities
@ stub QueryDirectEx
@ stdcall Query_Main(wstr long long ptr)
@ stub Reg_ReadGlobalsEx

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ AdaptiveTimeout_ClearInterfaceSpecificConfiguration
@ stub -version=0xA00+ AdaptiveTimeout_ResetAdaptiveTimeout
@ stub -version=0xA00+ AddRefQueryBlobEx
@ stub -version=0xA00+ Coalesce_UpdateNetVersion
@ stub -version=0xA00+ CreateSendBlob
@ stub -version=0xA00+ DeRefQueryBlobEx
@ stub -version=0xA00+ DelaySortDAServerlist
@ stub -version=0xA00+ DnsAddrArrayAddAddr
@ stub -version=0xA00+ DnsAddrArrayAddIp4
@ stub -version=0xA00+ DnsAddrArrayCreate
@ stub -version=0xA00+ DnsAddrBuildFromDnsRecord
@ stub -version=0xA00+ DnsApiAllocZero
@ stub -version=0xA00+ DnsApiOnNetworkChange
@ stub -version=0xA00+ DnsAsyncRegisterHostAddrs
@ stub -version=0xA00+ DnsBuildDotHost
@ stub -version=0xA00+ DnsCancelQuery
@ stub -version=0xA00+ DnsCancelQueryRaw
@ stub -version=0xA00+ DnsCheckNrptRuleIntegrity
@ stub -version=0xA00+ DnsCheckNrptRules
@ stub -version=0xA00+ DnsCleanupTcpConnections
@ stub -version=0xA00+ DnsConnectionDeletePolicyEntries
@ stub -version=0xA00+ DnsConnectionDeletePolicyEntriesByAppSid
@ stub -version=0xA00+ DnsConnectionDeletePolicyEntriesPrivate
@ stub -version=0xA00+ DnsConnectionDeleteProxyInfo
@ stub -version=0xA00+ DnsConnectionFreeNameList
@ stub -version=0xA00+ DnsConnectionFreeProxyInfo
@ stub -version=0xA00+ DnsConnectionFreeProxyInfoEx
@ stub -version=0xA00+ DnsConnectionFreeProxyList
@ stub -version=0xA00+ DnsConnectionGetHandleForHostUrlPrivate
@ stub -version=0xA00+ DnsConnectionGetNameList
@ stub -version=0xA00+ DnsConnectionGetProxyInfo
@ stub -version=0xA00+ DnsConnectionGetProxyInfoForHostUrl
@ stub -version=0xA00+ DnsConnectionGetProxyInfoForHostUrlEx
@ stub -version=0xA00+ DnsConnectionGetProxyList
@ stub -version=0xA00+ DnsConnectionSetPolicyEntries
@ stub -version=0xA00+ DnsConnectionSetPolicyEntriesPrivate
@ stub -version=0xA00+ DnsConnectionSetProxyInfo
@ stub -version=0xA00+ DnsConnectionUpdateIfIndexTable
@ stub -version=0xA00+ DnsDeRegisterLocal
@ stub -version=0xA00+ DnsDeleteSettings
@ stub -version=0xA00+ DnsDeleteWellKnownServerProperty
@ stub -version=0xA00+ DnsDhcpRegisterAddrs
@ stub -version=0xA00+ DnsDhcpRegisterHostAddrs
@ stub -version=0xA00+ DnsDhcpRegisterInit
@ stub -version=0xA00+ DnsDhcpRegisterTerm
@ stub -version=0xA00+ DnsDhcpRemoveRegistrations
@ stub -version=0xA00+ DnsDhcpSrvRegisterHostAddr
@ stub -version=0xA00+ DnsDhcpSrvRegisterHostAddrEx
@ stub -version=0xA00+ DnsDhcpSrvRegisterHostName
@ stub -version=0xA00+ DnsDhcpSrvRegisterHostNameEx
@ stub -version=0xA00+ DnsDhcpSrvRegisterInitEx
@ stub -version=0xA00+ DnsDhcpSrvRegisterInitialize
@ stub -version=0xA00+ DnsDisableIdnEncoding
@ stub -version=0xA00+ DnsFreeAdaptersInfo
@ stub -version=0xA00+ DnsFreeCustomServers
@ stub -version=0xA00+ DnsFreeNetworkServers
@ stub -version=0xA00+ DnsFreeNrptRule
@ stub -version=0xA00+ DnsFreeNrptRuleNamesList
@ stub -version=0xA00+ DnsFreePolicyConfig
@ stub -version=0xA00+ DnsFreeProxyName
@ stub -version=0xA00+ DnsFreeWellKnownServerProperty
@ stub -version=0xA00+ DnsFreeWellKnownServerPropertyEx
@ stub -version=0xA00+ DnsFreeWellKnownServers
@ stub -version=0xA00+ DnsFreeZtClientCertConfig
@ stub -version=0xA00+ DnsFreeZtConfig
@ stub -version=0xA00+ DnsFreeZtRules
@ stub -version=0xA00+ DnsFreeZtSettings
@ stub -version=0xA00+ DnsFreeZtTrustedCa
@ stub -version=0xA00+ DnsFreeZtTrustedServers
@ stub -version=0xA00+ DnsGetAdaptersInfo
@ stub -version=0xA00+ DnsGetApplicationIdentifier
@ stub -version=0xA00+ DnsGetApplicationSettings
@ stub -version=0xA00+ DnsGetCacheDataTableEx
@ stub -version=0xA00+ DnsGetCacheRecords
@ stub -version=0xA00+ DnsGetDdrInfo
@ stub -version=0xA00+ DnsGetDdrInfoCancel
@ stub -version=0xA00+ DnsGetDdrInfoInternal
@ stub -version=0xA00+ DnsGetInterfaceSettings
@ stub -version=0xA00+ DnsGetInterfaceSettingsInProc
@ stub -version=0xA00+ DnsGetLastFailedUpdateInfo
@ stub -version=0xA00+ DnsGetNetworkServers
@ stub -version=0xA00+ DnsGetNrptRule
@ stub -version=0xA00+ DnsGetNrptRuleNamesList
@ stub -version=0xA00+ DnsGetPolicyTableInfo
@ stub -version=0xA00+ DnsGetPolicyTableInfoPrivate
@ stub -version=0xA00+ DnsGetProxyInfoPrivate
@ stub -version=0xA00+ DnsGetProxyInformation
@ stub -version=0xA00+ DnsGetProxyInformationEx
@ stub -version=0xA00+ DnsGetQueryRetryTimeouts
@ stub -version=0xA00+ DnsGetSettings
@ stub -version=0xA00+ DnsGetWellKnownServerProperty
@ stub -version=0xA00+ DnsGetWellKnownServers
@ stub -version=0xA00+ DnsGetZtClientCertConfig
@ stub -version=0xA00+ DnsGetZtConfig
@ stub -version=0xA00+ DnsGetZtFilterIds
@ stub -version=0xA00+ DnsGetZtRules
@ stub -version=0xA00+ DnsGetZtSettings
@ stub -version=0xA00+ DnsGetZtTrustedCa
@ stub -version=0xA00+ DnsGetZtTrustedServers
@ stub -version=0xA00+ DnsIp6AddressToReverseNameW
@ stub -version=0xA00+ DnsIp6ReverseNameToAddressW
@ stub -version=0xA00+ DnsIsFlatRecord
@ stub -version=0xA00+ DnsIsNSECType
@ stub -version=0xA00+ DnsIsZtEnabled
@ stub -version=0xA00+ DnsLogEvent
@ stub -version=0xA00+ DnsNameCompare_UTF8
@ stub -version=0xA00+ DnsNetworkInfo_CreateFromFAZ
@ stub -version=0xA00+ DnsNetworkInformation_CreateFromFAZ
@ stub -version=0xA00+ DnsNotifyProxyConfigChangePrivate
@ stub -version=0xA00+ DnsQueryConfigAllocEx
@ stub -version=0xA00+ DnsQueryConfigDword
@ stub -version=0xA00+ DnsQueryExA
@ stub -version=0xA00+ DnsQueryExUTF8
@ stub -version=0xA00+ DnsQueryExW
@ stub -version=0xA00+ DnsQueryRaw
@ stub -version=0xA00+ DnsQueryRawResultFree
@ stub -version=0xA00+ DnsRecordListUnmapV4MappedAAAAInPlace
@ stub -version=0xA00+ DnsRegisterLocal
@ stub -version=0xA00+ DnsRemoveNrptRule
@ stub -version=0xA00+ DnsResetQueryRetryTimeouts
@ stub -version=0xA00+ DnsResolverOp
@ stub -version=0xA00+ DnsResolverQueryHvsi
@ stub -version=0xA00+ DnsScreenLocalAddrsForRegistration
@ stub -version=0xA00+ DnsServiceBrowse
@ stub -version=0xA00+ DnsServiceBrowseCancel
@ stub -version=0xA00+ DnsServiceConstructInstance
@ stub -version=0xA00+ DnsServiceCopyInstance
@ stub -version=0xA00+ DnsServiceDeRegister
@ stub -version=0xA00+ DnsServiceFreeInstance
@ stub -version=0xA00+ DnsServiceRegister
@ stub -version=0xA00+ DnsServiceRegisterCancel
@ stub -version=0xA00+ DnsServiceResolve
@ stub -version=0xA00+ DnsServiceResolveCancel
@ stub -version=0xA00+ DnsSetApplicationSettings
@ stub -version=0xA00+ DnsSetConfigDword
@ stub -version=0xA00+ DnsSetConfigValue
@ stub -version=0xA00+ DnsSetInterfaceSettings
@ stub -version=0xA00+ DnsSetNrptRule
@ stub -version=0xA00+ DnsSetNrptRules
@ stub -version=0xA00+ DnsSetQueryRetryTimeouts
@ stub -version=0xA00+ DnsSetSettings
@ stub -version=0xA00+ DnsSetWellKnownServerProperty
@ stub -version=0xA00+ DnsSetZtSettings
@ stub -version=0xA00+ DnsSplitDotHost
@ stub -version=0xA00+ DnsStartMulticastQuery
@ stub -version=0xA00+ DnsStopMulticastQuery
@ stub -version=0xA00+ DnsStringToAddress
@ stub -version=0xA00+ DnsTraceServerConfig
@ stub -version=0xA00+ DnsUpdateCaptivePortalPresence
@ stub -version=0xA00+ DnsUpdateMachinePresence
@ stub -version=0xA00+ DnsValidateNameOrIp_TempW
@ stub -version=0xA00+ DnsValidateServerArray_A
@ stub -version=0xA00+ DnsValidateServerArray_W
@ stub -version=0xA00+ DnsValidateServerStatus
@ stub -version=0xA00+ DnsValidateServer_A
@ stub -version=0xA00+ DnsValidateServer_W
@ stub -version=0xA00+ DnsWriteReverseNameStringForIpAddressW
@ stub -version=0xA00+ DnsZtHelperGetSettings
@ stub -version=0xA00+ DnsZtHelperProcessClientCerts
@ stub -version=0xA00+ DnsZtHelperPromoteConfig
@ stub -version=0xA00+ DnsZtNotifyDnscache
@ stub -version=0xA00+ Dns_CacheServiceCleanup
@ stub -version=0xA00+ Dns_CacheServiceInit
@ stub -version=0xA00+ Dns_CacheServiceInitEx
@ stub -version=0xA00+ Dns_CacheServiceStopIssued
@ stub -version=0xA00+ Dns_ExtractRecordsFromMessage
@ stub -version=0xA00+ Dns_FreeMsgBuf
@ stub -version=0xA00+ Dns_InitializeMsgBuf
@ stub -version=0xA00+ Dns_SetRecordsScopeId
@ stub -version=0xA00+ Dns_SetRecordsSection
@ stub -version=0xA00+ Dns_SetRecordsTtl
@ stub -version=0xA00+ ExtraInfo_Init
@ stub -version=0xA00+ Faz_AreServerListsInSameNameSpace
@ stub -version=0xA00+ FlushDnsPolicyUnreachableStatus
@ stub -version=0xA00+ HostsFile_Close
@ stub -version=0xA00+ HostsFile_Open
@ stub -version=0xA00+ HostsFile_ReadLine
@ stub -version=0xA00+ IpHelp_IsAddrOnLink
@ stub -version=0xA00+ Local_GetRecordsForLocalName
@ stub -version=0xA00+ Local_GetRecordsForLocalNameEx
@ stub -version=0xA00+ NetInfo_BuildEx
@ stub -version=0xA00+ NetInfo_CopyNetworkIndex
@ stub -version=0xA00+ NetInfo_CreatePerNetworkNetinfo
@ stub -version=0xA00+ NetInfo_GetAdapterByAddress
@ stub -version=0xA00+ NetInfo_GetAdapterByInterfaceIndex
@ stub -version=0xA00+ NetInfo_GetAdapterByName
@ stub -version=0xA00+ NetInfo_IsAddrConfig
@ stub -version=0xA00+ NetInfo_IsTcpipConfigChange
@ stub -version=0xA00+ NetInfo_UpdateDnsInterfaceConfigChange
@ stub -version=0xA00+ NetInfo_UpdateNetworkProperties
@ stub -version=0xA00+ NetInfo_UpdateServerReachability
@ stub -version=0xA00+ Query_Cancel
@ stub -version=0xA00+ Query_InProcSetCompletion
@ stub -version=0xA00+ Reg_FreeUpdateInfo
@ stub -version=0xA00+ Reg_GetValueEx
@ stub -version=0xA00+ Reg_ReadUpdateInfo
@ stub -version=0xA00+ Security_ContextListTimeout
@ stub -version=0xA00+ Send_AndRecvUdpWithParam
@ stub -version=0xA00+ Send_MessagePrivate
@ stub -version=0xA00+ Send_MessagePrivateEx
@ stub -version=0xA00+ Send_OpenTcpConnectionAndSend
@ stub -version=0xA00+ Socket_CacheCleanup
@ stub -version=0xA00+ Socket_CacheInit
@ stub -version=0xA00+ Socket_CleanupWinsock
@ stub -version=0xA00+ Socket_ClearMessageSockets
@ stub -version=0xA00+ Socket_CloseEx
@ stub -version=0xA00+ Socket_CloseMessageSockets
@ stub -version=0xA00+ Socket_Create
@ stub -version=0xA00+ Socket_CreateMulticast
@ stub -version=0xA00+ Socket_InitWinsock
@ stub -version=0xA00+ Socket_JoinMulticast
@ stub -version=0xA00+ Socket_RecvFrom
@ stub -version=0xA00+ Socket_SetMulticastInterface
@ stub -version=0xA00+ Socket_SetMulticastLoopBack
@ stub -version=0xA00+ Socket_SetTtl
@ stub -version=0xA00+ Socket_TcpListen
@ stub -version=0xA00+ ThreadPool_QueueWork
@ stub -version=0xA00+ Trace_Reset
@ stub -version=0xA00+ Update_ReplaceAddressRecordsW
@ stub -version=0xA00+ Util_IsIp6Running
@ stub -version=0xA00+ Util_IsRunningOnXboxOne
@ stub -version=0xA00+ WriteDnsNrptRulesToRegistry
# END win10-surface stubs
