@ stdcall AddIPAddress(long long long ptr ptr)
@ stub AllocateAndGetArpEntTableFromStack
@ stdcall AllocateAndGetIfTableFromStack(ptr long long long)
@ stdcall AllocateAndGetIpAddrTableFromStack(ptr long long long)
@ stdcall AllocateAndGetIpForwardTableFromStack(ptr long long long)
@ stdcall AllocateAndGetIpNetTableFromStack(ptr long long long)
@ stdcall AllocateAndGetTcpExTable2FromStack(ptr long long long long long)
@ stdcall AllocateAndGetTcpExTableFromStack(ptr long long long long)
@ stdcall AllocateAndGetTcpTableFromStack(ptr long long long)
@ stdcall AllocateAndGetUdpExTable2FromStack(ptr long long long long long)
@ stdcall AllocateAndGetUdpExTableFromStack(ptr long long long long)
@ stdcall AllocateAndGetUdpTableFromStack(ptr long long long)
@ stdcall CancelIPChangeNotify(ptr)
@ stub CancelSecurityHealthChangeNotify
@ stdcall -stub -version=0x600+ ConvertGuidToStringA(ptr ptr long)
@ stdcall -stub -version=0x600+ ConvertGuidToStringW(ptr ptr long)
@ stdcall -stub -version=0x600+ ConvertInterfaceAliasToLuid(wstr ptr)
@ stdcall -stub -version=0x600+ ConvertInterfaceGuidToLuid(ptr ptr)
@ stdcall ConvertInterfaceIndexToLuid(long ptr)
@ stdcall -stub -version=0x600+ ConvertInterfaceLuidToAlias(ptr ptr long)
@ stdcall -stub -version=0x600+ ConvertInterfaceLuidToGuid(ptr ptr)
@ stdcall -stub -version=0x600+ ConvertInterfaceLuidToIndex(ptr ptr)
@ stdcall -stub -version=0x600+ ConvertInterfaceLuidToNameA(ptr ptr long)
@ stdcall -stub -version=0x600+ ConvertInterfaceLuidToNameW(ptr ptr long)
@ stdcall -stub -version=0x600+ ConvertInterfaceNameToLuidA(str ptr)
@ stdcall -stub -version=0x600+ ConvertInterfaceNameToLuidW(wstr ptr)
@ stdcall -stub -version=0x600+ ConvertLengthToIpv4Mask(long ptr)
@ stdcall -stub -version=0x600+ ConvertStringToGuidW(wstr ptr)
@ stdcall CreateIpForwardEntry(ptr)
@ stdcall CreateIpNetEntry(ptr)
@ stdcall CreateProxyArpEntry(long long long)
@ stdcall DeleteIPAddress(long)
@ stdcall DeleteIpForwardEntry(ptr)
@ stdcall DeleteIpNetEntry(ptr)
@ stdcall DeleteProxyArpEntry(long long long)
@ stdcall DisableMediaSense(ptr ptr)
@ stdcall EnableRouter(ptr ptr)
@ stdcall FlushIpNetTable(long)
@ stub FlushIpNetTableFromStack
@ stdcall FreeMibTable(ptr)
@ stdcall GetAdapterIndex(wstr ptr)
@ stdcall GetAdapterOrderMap()
@ stdcall GetAdaptersAddresses(long long ptr ptr ptr)
@ stdcall GetAdaptersInfo(ptr ptr)
@ stdcall GetBestInterface(long ptr)
@ stdcall GetBestInterfaceEx(ptr ptr)
@ stub GetBestInterfaceFromStack
@ stdcall GetBestRoute(long long long)
@ stub GetBestRouteFromStack
@ stdcall -stub -version=0x600+ GetCurrentThreadCompartmentId()
@ stdcall GetExtendedTcpTable(ptr ptr long long long long)
@ stdcall GetExtendedUdpTable(ptr ptr long long long long)
@ stdcall GetFriendlyIfIndex(long)
@ stdcall GetIcmpStatistics(ptr)
@ stdcall GetIcmpStatisticsEx(ptr long)
@ stub GetIcmpStatsFromStack
@ stub GetIcmpStatsFromStackEx
@ stdcall GetIfEntry(ptr)
@ stdcall -stub -version=0x600+ GetIfEntry2(ptr)
@ stdcall -stub -version=0xA00+ GetIfEntry2Ex(long ptr)
@ stub GetIfEntryFromStack
@ stdcall GetIfTable(ptr ptr long)
@ stdcall -stub -version=0x600+ GetIfTable2(ptr)
@ stdcall -stub -version=0x600+ GetIfTable2Ex(long ptr)
@ stub GetIfTableFromStack
@ stub GetIgmpList
@ stdcall GetInterfaceInfo(ptr ptr)
@ stdcall GetIpAddrTable(ptr ptr long)
@ stub GetIpAddrTableFromStack
@ stdcall GetIpErrorString(long ptr ptr)
@ stdcall GetIpForwardTable(ptr ptr long)
@ stdcall -stub -version=0x600+ GetIpForwardTable2(long ptr)
@ stub GetIpForwardTableFromStack
@ stdcall GetIpNetTable(ptr ptr long)
@ stdcall -stub -version=0x600+ GetIpNetTable2(long ptr)
@ stub GetIpNetTableFromStack
@ stdcall GetIpStatistics(ptr)
@ stdcall GetIpStatisticsEx(ptr long)
@ stub GetIpStatsFromStack
@ stub GetIpStatsFromStackEx
@ stdcall GetNetworkParams(ptr ptr)
@ stdcall GetNumberOfInterfaces(ptr)
@ stub GetOwnerModuleFromTcp6Entry
@ stdcall GetOwnerModuleFromTcpEntry(ptr long ptr ptr)
@ stub GetOwnerModuleFromUdp6Entry
@ stdcall GetOwnerModuleFromUdpEntry(ptr long ptr ptr)
@ stdcall GetPerAdapterInfo(long ptr ptr)
@ stdcall GetRTTAndHopCount(long ptr long ptr)
@ stub GetTcpExTable2FromStack
@ stdcall GetTcpStatistics(ptr)
@ stdcall GetTcpStatisticsEx(ptr long)
@ stub GetTcpStatsFromStack
@ stub GetTcpStatsFromStackEx
@ stdcall GetTcpTable(ptr ptr long)
@ stub GetTcpTableFromStack
@ stub GetUdpExTable2FromStack
@ stdcall GetUdpStatistics(ptr)
@ stdcall GetUdpStatisticsEx(ptr long)
@ stub GetUdpStatsFromStack
@ stub  GetUdpStatsFromStackEx
@ stdcall GetUdpTable(ptr ptr long)
@ stub GetUdpTableFromStack
@ stdcall -stub -version=0x600+ GetUnicastIpAddressEntry(ptr)
@ stdcall -stub -version=0x600+ GetUnicastIpAddressTable(long ptr)
@ stdcall GetUniDirectionalAdapterInfo(ptr ptr)
@ stdcall Icmp6CreateFile()
@ stdcall Icmp6ParseReplies(ptr long)
@ stdcall Icmp6SendEcho2(ptr ptr ptr ptr ptr ptr ptr long ptr ptr long long)
@ stdcall IcmpCloseHandle(ptr)
@ stdcall IcmpCreateFile()
@ stdcall IcmpParseReplies(ptr long)
@ stdcall IcmpSendEcho2(ptr ptr ptr ptr long ptr long ptr ptr long long)
@ stdcall IcmpSendEcho(ptr long ptr long ptr ptr long long)
@ stdcall -stub -version=0x600+ if_indextoname(long ptr)
@ stdcall -stub -version=0x600+ if_nametoindex(str)
@ stub InternalCreateIpForwardEntry
@ stub InternalCreateIpNetEntry
@ stub InternalDeleteIpForwardEntry
@ stub InternalDeleteIpNetEntry
@ stub InternalGetIfTable
@ stub InternalGetIpAddrTable
@ stub InternalGetIpForwardTable
@ stub InternalGetIpNetTable
@ stub InternalGetTcpTable
@ stub InternalGetUdpTable
@ stub InternalSetIfEntry
@ stub InternalSetIpForwardEntry
@ stub InternalSetIpNetEntry
@ stub InternalSetIpStats
@ stub InternalSetTcpEntry
@ stdcall IpReleaseAddress(ptr)
@ stdcall IpRenewAddress(ptr)
@ stub IsLocalAddress
@ stub NTPTimeToNTFileTime
@ stub NTTimeToNTPTime
# Ported to iphlpapi-rust by the 2026-09-20 upstream sync (see iphlpapi_main.rs).
# Upstream implemented it after our fork and their new netsh/ifmon call it.
@ stdcall NhGetGuidFromInterfaceName(wstr ptr long long)
@ stdcall NhGetInterfaceNameFromDeviceGuid(ptr ptr ptr long long)
@ stdcall NhGetInterfaceNameFromGuid(ptr ptr ptr long long)
@ stdcall NhpAllocateAndGetInterfaceInfoFromStack(ptr ptr long ptr long)
@ stub NhpGetInterfaceIndexFromStack
@ stdcall NotifyAddrChange(ptr ptr)
@ stdcall NotifyRouteChange(ptr ptr)
@ stub NotifyRouteChangeEx
@ stub NotifySecurityHealthChange
@ stdcall _PfAddFiltersToInterface@24()
@ stdcall _PfAddGlobalFilterToInterface@8()
@ stdcall _PfBindInterfaceToIPAddress@12()
@ stdcall _PfBindInterfaceToIndex@16()
@ stdcall _PfCreateInterface@24()
@ stdcall _PfDeleteInterface@4()
@ stdcall _PfDeleteLog@0()
@ stdcall _PfGetInterfaceStatistics@16()
@ stdcall _PfMakeLog@4()
@ stdcall _PfRebindFilters@8()
@ stdcall _PfRemoveFilterHandles@12()
@ stdcall _PfRemoveFiltersFromInterface@20()
@ stdcall _PfRemoveGlobalFilterFromInterface@8()
@ stdcall _PfSetLogBuffer@28()
@ stdcall _PfTestPacket@20()
@ stdcall _PfUnBindInterface@4()
@ stdcall RestoreMediaSense(ptr ptr)
@ stdcall SendARP(long long ptr ptr)
@ stub SetAdapterIpAddress
@ stub SetBlockRoutes
@ stdcall SetIfEntry(ptr)
@ stub SetIfEntryToStack
@ stdcall SetIpForwardEntry(ptr)
@ stdcall SetIpForwardEntryToStack(ptr)
@ stub SetIpMultihopRouteEntryToStack
@ stdcall SetIpNetEntry(ptr)
@ stub SetIpNetEntryToStack
@ stub SetIpRouteEntryToStack
@ stdcall SetIpStatistics(ptr)
@ stub SetIpStatsToStack
@ stdcall SetIpTTL(long)
@ stub SetProxyArpEntryToStack
@ stub SetRouteWithRef
@ stdcall SetTcpEntry(ptr)
@ stub SetTcpEntryToStack
@ stdcall UnenableRouter(ptr ptr)
@ stub do_echo_rep
@ stub do_echo_req
@ stub register_icmp

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ AllocateAndGetInterfaceInfoFromStack
@ stub -version=0xA00+ CancelIfTimestampConfigChange
@ stub -version=0xA00+ CancelMibChangeNotify2
@ stub -version=0xA00+ CaptureInterfaceHardwareCrossTimestamp
@ stub -version=0xA00+ CloseCompartment
@ stub -version=0xA00+ CloseGetIPPhysicalInterfaceForDestination
@ stub -version=0xA00+ ConvertCompartmentGuidToId
@ stub -version=0xA00+ ConvertCompartmentIdToGuid
@ stub -version=0xA00+ ConvertInterfacePhysicalAddressToLuid
@ stub -version=0xA00+ ConvertIpv4MaskToLength
@ stub -version=0xA00+ ConvertRemoteInterfaceAliasToLuid
@ stub -version=0xA00+ ConvertRemoteInterfaceGuidToLuid
@ stub -version=0xA00+ ConvertRemoteInterfaceIndexToLuid
@ stub -version=0xA00+ ConvertRemoteInterfaceLuidToAlias
@ stub -version=0xA00+ ConvertRemoteInterfaceLuidToGuid
@ stub -version=0xA00+ ConvertRemoteInterfaceLuidToIndex
@ stub -version=0xA00+ ConvertStringToGuidA
@ stub -version=0xA00+ ConvertStringToInterfacePhysicalAddress
@ stub -version=0xA00+ CreateAnycastIpAddressEntry
@ stub -version=0xA00+ CreateCompartment
@ stub -version=0xA00+ CreateFlVirtualInterface
@ stub -version=0xA00+ CreateIpForwardEntry2
@ stub -version=0xA00+ CreateIpNetEntry2
@ stub -version=0xA00+ CreatePersistentTcpPortReservation
@ stub -version=0xA00+ CreatePersistentUdpPortReservation
@ stub -version=0xA00+ CreateSortedAddressPairs
@ stub -version=0xA00+ CreateUnicastIpAddressEntry
@ stub -version=0xA00+ DeleteAnycastIpAddressEntry
@ stub -version=0xA00+ DeleteCompartment
@ stub -version=0xA00+ DeleteFlVirtualInterface
@ stub -version=0xA00+ DeleteIpForwardEntry2
@ stub -version=0xA00+ DeleteIpNetEntry2
@ stub -version=0xA00+ DeletePersistentTcpPortReservation
@ stub -version=0xA00+ DeletePersistentUdpPortReservation
@ stub -version=0xA00+ DeleteUnicastIpAddressEntry
@ stub -version=0xA00+ FlushIpNetTable2
@ stub -version=0xA00+ FlushIpPathTable
@ stub -version=0xA00+ FreeDnsSettings
@ stub -version=0xA00+ FreeInterfaceDnsSettings
@ stub -version=0xA00+ GetAnycastIpAddressEntry
@ stub -version=0xA00+ GetAnycastIpAddressTable
@ stub -version=0xA00+ GetBestRoute2
@ stub -version=0xA00+ GetCurrentThreadCompartmentScope
@ stub -version=0xA00+ GetDefaultCompartmentId
@ stub -version=0xA00+ GetDnsSettings
@ stub -version=0xA00+ GetFlVirtualInterface
@ stub -version=0xA00+ GetFlVirtualInterfaceTable
@ stub -version=0xA00+ GetIfStackTable
@ stub -version=0xA00+ GetInterfaceActiveTimestampCapabilities
@ stub -version=0xA00+ GetInterfaceCompartmentId
@ stub -version=0xA00+ GetInterfaceCurrentTimestampCapabilities
@ stub -version=0xA00+ GetInterfaceDnsSettings
@ stub -version=0xA00+ GetInterfaceHardwareTimestampCapabilities
@ stub -version=0xA00+ GetInterfaceSupportedTimestampCapabilities
@ stub -version=0xA00+ GetInvertedIfStackTable
@ stub -version=0xA00+ GetIpForwardEntry2
@ stub -version=0xA00+ GetIpInterfaceEntry
@ stub -version=0xA00+ GetIpInterfaceTable
@ stub -version=0xA00+ GetIpNetEntry2
@ stub -version=0xA00+ GetIpNetworkConnectionBandwidthEstimates
@ stub -version=0xA00+ GetIpPathEntry
@ stub -version=0xA00+ GetIpPathTable
@ stub -version=0xA00+ GetJobCompartmentId
@ stub -version=0xA00+ GetMulticastIpAddressEntry
@ stub -version=0xA00+ GetMulticastIpAddressTable
@ stub -version=0xA00+ GetNetworkConnectivityHint
@ stub -version=0xA00+ GetNetworkConnectivityHintForInterface
@ stub -version=0xA00+ GetNetworkInformation
@ stub -version=0xA00+ GetOwnerModuleFromPidAndInfo
@ stub -version=0xA00+ GetPerTcp6ConnectionEStats
@ stub -version=0xA00+ GetPerTcp6ConnectionStats
@ stub -version=0xA00+ GetPerTcpConnectionEStats
@ stub -version=0xA00+ GetPerTcpConnectionStats
@ stub -version=0xA00+ GetSessionCompartmentId
@ stub -version=0xA00+ GetTcp6Table
@ stub -version=0xA00+ GetTcp6Table2
@ stub -version=0xA00+ GetTcpStatisticsEx2
@ stub -version=0xA00+ GetTcpTable2
@ stub -version=0xA00+ GetTeredoPort
@ stub -version=0xA00+ GetUdp6Table
@ stub -version=0xA00+ GetUdpStatisticsEx2
@ stub -version=0xA00+ GetWPAOACSupportLevel
@ stub -version=0xA00+ IcmpSendEcho2Ex
@ stub -version=0xA00+ InitializeCompartmentEntry
@ stub -version=0xA00+ InitializeFlVirtualInterfaceEntry
@ stub -version=0xA00+ InitializeIpForwardEntry
@ stub -version=0xA00+ InitializeIpInterfaceEntry
@ stub -version=0xA00+ InitializeUnicastIpAddressEntry
@ stub -version=0xA00+ InternalCleanupPersistentStore
@ stub -version=0xA00+ InternalCreateAnycastIpAddressEntry
@ stub -version=0xA00+ InternalCreateIpForwardEntry2
@ stub -version=0xA00+ InternalCreateIpNetEntry2
@ stub -version=0xA00+ InternalCreateOrRefIpForwardEntry2
@ stub -version=0xA00+ InternalCreateUnicastIpAddressEntry
@ stub -version=0xA00+ InternalDeleteAnycastIpAddressEntry
@ stub -version=0xA00+ InternalDeleteIpForwardEntry2
@ stub -version=0xA00+ InternalDeleteIpNetEntry2
@ stub -version=0xA00+ InternalDeleteUnicastIpAddressEntry
@ stub -version=0xA00+ InternalFindInterfaceByAddress
@ stub -version=0xA00+ InternalGetAnycastIpAddressEntry
@ stub -version=0xA00+ InternalGetAnycastIpAddressTable
@ stub -version=0xA00+ InternalGetBoundTcp6EndpointTable
@ stub -version=0xA00+ InternalGetBoundTcpEndpointTable
@ stub -version=0xA00+ InternalGetForwardIpTable2
@ stub -version=0xA00+ InternalGetIPPhysicalInterfaceForDestination
@ stub -version=0xA00+ InternalGetIfEntry2
@ stub -version=0xA00+ InternalGetIfTable2
@ stub -version=0xA00+ InternalGetIpForwardEntry2
@ stub -version=0xA00+ InternalGetIpInterfaceEntry
@ stub -version=0xA00+ InternalGetIpInterfaceTable
@ stub -version=0xA00+ InternalGetIpNetEntry2
@ stub -version=0xA00+ InternalGetIpNetTable2
@ stub -version=0xA00+ InternalGetMulticastIpAddressEntry
@ stub -version=0xA00+ InternalGetMulticastIpAddressTable
@ stub -version=0xA00+ InternalGetRtcSlotInformation
@ stub -version=0xA00+ InternalGetTcp6Table2
@ stub -version=0xA00+ InternalGetTcp6TableWithOwnerModule
@ stub -version=0xA00+ InternalGetTcp6TableWithOwnerPid
@ stub -version=0xA00+ InternalGetTcpDynamicPortRange
@ stub -version=0xA00+ InternalGetTcpTable2
@ stub -version=0xA00+ InternalGetTcpTableEx
@ stub -version=0xA00+ InternalGetTcpTableWithOwnerModule
@ stub -version=0xA00+ InternalGetTcpTableWithOwnerPid
@ stub -version=0xA00+ InternalGetTunnelPhysicalAdapter
@ stub -version=0xA00+ InternalGetUdp6Table2
@ stub -version=0xA00+ InternalGetUdp6TableWithOwnerModule
@ stub -version=0xA00+ InternalGetUdp6TableWithOwnerPid
@ stub -version=0xA00+ InternalGetUdpDynamicPortRange
@ stub -version=0xA00+ InternalGetUdpTable2
@ stub -version=0xA00+ InternalGetUdpTableEx
@ stub -version=0xA00+ InternalGetUdpTableWithOwnerModule
@ stub -version=0xA00+ InternalGetUdpTableWithOwnerPid
@ stub -version=0xA00+ InternalGetUnicastIpAddressEntry
@ stub -version=0xA00+ InternalGetUnicastIpAddressTable
@ stub -version=0xA00+ InternalIcmpCreateFileEx
@ stub -version=0xA00+ InternalSetIpForwardEntry2
@ stub -version=0xA00+ InternalSetIpInterfaceEntry
@ stub -version=0xA00+ InternalSetIpNetEntry2
@ stub -version=0xA00+ InternalSetTcpDynamicPortRange
@ stub -version=0xA00+ InternalSetTeredoPort
@ stub -version=0xA00+ InternalSetUdpDynamicPortRange
@ stub -version=0xA00+ InternalSetUnicastIpAddressEntry
@ stub -version=0xA00+ LookupPersistentTcpPortReservation
@ stub -version=0xA00+ LookupPersistentUdpPortReservation
@ stub -version=0xA00+ NhGetInterfaceDescriptionFromGuid
@ stub -version=0xA00+ NotifyCompartmentChange
@ stub -version=0xA00+ NotifyIfTimestampConfigChange
@ stub -version=0xA00+ NotifyIpInterfaceChange
@ stub -version=0xA00+ NotifyNetworkConnectivityHintChange
@ stub -version=0xA00+ NotifyRouteChange2
@ stub -version=0xA00+ NotifyStableUnicastIpAddressTable
@ stub -version=0xA00+ NotifyTeredoPortChange
@ stub -version=0xA00+ NotifyUnicastIpAddressChange
@ stub -version=0xA00+ OpenCompartment
@ stub -version=0xA00+ ParseNetworkString
@ stub -arch=win64 -version=0xA00+ PfAddFiltersToInterface
@ stub -arch=win64 -version=0xA00+ PfAddGlobalFilterToInterface
@ stub -arch=win64 -version=0xA00+ PfBindInterfaceToIPAddress
@ stub -arch=win64 -version=0xA00+ PfBindInterfaceToIndex
@ stub -arch=win64 -version=0xA00+ PfCreateInterface
@ stub -arch=win64 -version=0xA00+ PfDeleteInterface
@ stub -arch=win64 -version=0xA00+ PfDeleteLog
@ stub -arch=win64 -version=0xA00+ PfGetInterfaceStatistics
@ stub -arch=win64 -version=0xA00+ PfMakeLog
@ stub -arch=win64 -version=0xA00+ PfRebindFilters
@ stub -arch=win64 -version=0xA00+ PfRemoveFilterHandles
@ stub -arch=win64 -version=0xA00+ PfRemoveFiltersFromInterface
@ stub -arch=win64 -version=0xA00+ PfRemoveGlobalFilterFromInterface
@ stub -arch=win64 -version=0xA00+ PfSetLogBuffer
@ stub -arch=win64 -version=0xA00+ PfTestPacket
@ stub -arch=win64 -version=0xA00+ PfUnBindInterface
@ stub -version=0xA00+ RegisterInterfaceTimestampConfigChange
@ stub -version=0xA00+ ResolveIpNetEntry2
@ stub -version=0xA00+ ResolveNeighbor
@ stub -version=0xA00+ SetCurrentThreadCompartmentId
@ stub -version=0xA00+ SetCurrentThreadCompartmentScope
@ stub -version=0xA00+ SetDnsSettings
@ stub -version=0xA00+ SetFlVirtualInterface
@ stub -version=0xA00+ SetInterfaceDnsSettings
@ stub -version=0xA00+ SetIpForwardEntry2
@ stub -version=0xA00+ SetIpInterfaceEntry
@ stub -version=0xA00+ SetIpNetEntry2
@ stub -version=0xA00+ SetIpStatisticsEx
@ stub -version=0xA00+ SetJobCompartmentId
@ stub -version=0xA00+ SetNetworkInformation
@ stub -version=0xA00+ SetPerTcp6ConnectionEStats
@ stub -version=0xA00+ SetPerTcp6ConnectionStats
@ stub -version=0xA00+ SetPerTcpConnectionEStats
@ stub -version=0xA00+ SetPerTcpConnectionStats
@ stub -version=0xA00+ SetSessionCompartmentId
@ stub -version=0xA00+ SetUnicastIpAddressEntry
@ stub -version=0xA00+ UnregisterInterfaceTimestampConfigChange
# END win10-surface stubs
