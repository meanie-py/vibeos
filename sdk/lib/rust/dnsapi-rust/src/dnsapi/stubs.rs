use crate::prelude::*;
use core::ffi::c_void;


// ---- DnsAddRecordSet_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsAddRecordSet_A() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsAddRecordSet_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsAddRecordSet_UTF8() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsAddRecordSet_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsAddRecordSet_W() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsAllocateRecord ----
#[no_mangle]
pub unsafe extern "system" fn DnsAllocateRecord() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsApiHeapReset ----
#[no_mangle]
pub unsafe extern "system" fn DnsApiHeapReset() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsAsyncRegisterHostAddrs_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsAsyncRegisterHostAddrs_A() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsAsyncRegisterHostAddrs_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsAsyncRegisterHostAddrs_UTF8() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsAsyncRegisterHostAddrs_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsAsyncRegisterHostAddrs_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsAsyncRegisterInit ----
#[no_mangle]
pub unsafe extern "system" fn DnsAsyncRegisterInit() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsAsyncRegisterTerm ----
#[no_mangle]
pub unsafe extern "system" fn DnsAsyncRegisterTerm() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsCheckNameCollision_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsCheckNameCollision_A() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsCheckNameCollision_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsCheckNameCollision_UTF8() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsCheckNameCollision_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsCheckNameCollision_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsCopyStringEx ----
#[no_mangle]
pub unsafe extern "system" fn DnsCopyStringEx() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsCreateReverseNameStringForIpAddress ----
#[no_mangle]
pub unsafe extern "system" fn DnsCreateReverseNameStringForIpAddress() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsCreateStandardDnsNameCopy ----
#[no_mangle]
pub unsafe extern "system" fn DnsCreateStandardDnsNameCopy() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsCreateStringCopy ----
#[no_mangle]
pub unsafe extern "system" fn DnsCreateStringCopy() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsDhcpSrvRegisterHostName_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsDhcpSrvRegisterHostName_W() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsDhcpSrvRegisterInit ----
#[no_mangle]
pub unsafe extern "system" fn DnsDhcpSrvRegisterInit() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsDhcpSrvRegisterTerm ----
#[no_mangle]
pub unsafe extern "system" fn DnsDhcpSrvRegisterTerm() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsDisableAdapterDomainNameRegistration ----
#[no_mangle]
pub unsafe extern "system" fn DnsDisableAdapterDomainNameRegistration() -> DWORD
{
    return ERROR_OUTOFMEMORY;
}

// ---- DnsDisableBNodeResolverThread ----
#[no_mangle]
pub unsafe extern "system" fn DnsDisableBNodeResolverThread() -> DNS_STATUS
{
    ERROR_OUTOFMEMORY
}

// ---- DnsDisableDynamicRegistration ----
#[no_mangle]
pub unsafe extern "system" fn DnsDisableDynamicRegistration() -> DNS_STATUS {
    return ERROR_OUTOFMEMORY;
}

// ---- DnsDowncaseDnsNameLabel ----
#[no_mangle]
pub unsafe extern "system" fn DnsDowncaseDnsNameLabel() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsEnableAdapterDomainNameRegistration ----
#[no_mangle]
pub unsafe extern "system" fn DnsEnableAdapterDomainNameRegistration() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsEnableBNodeResolverThread ----
#[no_mangle]
pub unsafe extern "system" fn DnsEnableBNodeResolverThread() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsEnableDynamicRegistration ----
#[no_mangle]
pub unsafe extern "system" fn DnsEnableDynamicRegistration() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsExtractRecordsFromMessage_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsExtractRecordsFromMessage_UTF8(
    pDnsBuffer: PDNS_MESSAGE_BUFFER,
    wMessageLength: WORD,
    ppRecord: *mut PDNS_RECORD,
) -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsExtractRecordsFromMessage_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsExtractRecordsFromMessage_W(
    pDnsBuffer: PDNS_MESSAGE_BUFFER,
    wMessageLength: WORD,
    ppRecord: *mut PDNS_RECORD,
) -> DNS_STATUS {
    // UNIMPLEMENTED
    ERROR_OUTOFMEMORY
}

// ---- DnsFindAuthoritativeZone ----
#[no_mangle]
pub unsafe extern "system" fn DnsFindAuthoritativeZone() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsFreeAdapterInformation ----
#[no_mangle]
pub unsafe extern "system" fn DnsFreeAdapterInformation() -> DNS_STATUS {
    // UNIMPLEMENTED
    ERROR_OUTOFMEMORY
}

// ---- DnsFreeNetworkInformation ----
#[no_mangle]
pub unsafe extern "system" fn DnsFreeNetworkInformation() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsFreeSearchInformation ----
#[no_mangle]
pub unsafe extern "system" fn DnsFreeSearchInformation() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetBufferLengthForStringCopy ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetBufferLengthForStringCopy() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetDnsServerList ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetDnsServerList() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetDomainName ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetDomainName() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetHostName_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetHostName_A() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetHostName_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetHostName_UTF8() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetHostName_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetHostName_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetIpAddressInfoList ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetIpAddressInfoList() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetIpAddressList ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetIpAddressList() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetLastServerUpdateIP ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetLastServerUpdateIP() -> DNS_STATUS
{
    ERROR_OUTOFMEMORY
}

// ---- DnsGetMaxNumberOfAddressesToRegister ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetMaxNumberOfAddressesToRegister() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetNetworkInformation ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetNetworkInformation() -> DNS_STATUS {
    return ERROR_OUTOFMEMORY;
}

// ---- DnsGetPrimaryDomainName_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetPrimaryDomainName_A() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetPrimaryDomainName_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetPrimaryDomainName_UTF8() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetPrimaryDomainName_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetPrimaryDomainName_W() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsGetSearchInformation ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetSearchInformation() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsIpv6AddressToString ----
#[no_mangle]
pub unsafe extern "system" fn DnsIpv6AddressToString() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsIpv6StringToAddress ----
#[no_mangle]
pub unsafe extern "system" fn DnsIpv6StringToAddress() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsIsAMailboxType ----
#[no_mangle]
pub unsafe extern "system" fn DnsIsAMailboxType() -> u32 {
    return ERROR_OUTOFMEMORY as u32;
}

// ---- DnsIsAdapterDomainNameRegistrationEnabled ----
#[no_mangle]
pub unsafe extern "system" fn DnsIsAdapterDomainNameRegistrationEnabled() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsIsDynamicRegistrationEnabled ----
#[no_mangle]
pub unsafe extern "system" fn DnsIsDynamicRegistrationEnabled() -> DNS_STATUS {
    return ERROR_OUTOFMEMORY;
}

// ---- DnsIsStatusRcode ----
#[no_mangle]
pub unsafe extern "system" fn DnsIsStatusRcode() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsIsStringCountValidForTextType ----
#[no_mangle]
pub unsafe extern "system" fn DnsIsStringCountValidForTextType() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsMapRcodeToStatus ----
#[no_mangle]
pub unsafe extern "system" fn DnsMapRcodeToStatus() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsModifyRecordSet_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsModifyRecordSet_A() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsModifyRecordSet_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsModifyRecordSet_UTF8() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsModifyRecordSet_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsModifyRecordSet_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsModifyRecordsInSet_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsModifyRecordsInSet_A(
    pAddRecords: PDNS_RECORD,
    pDeleteRecords: PDNS_RECORD,
    Options: DWORD,
    hContext: HANDLE,
    pExtraList: PIP4_ARRAY,
    pReserved: PVOID,
) -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsModifyRecordsInSet_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsModifyRecordsInSet_UTF8(
    pAddRecords: PDNS_RECORD,
    pDeleteRecords: PDNS_RECORD,
    Options: DWORD,
    hContext: HANDLE,
    pExtraList: PIP4_ARRAY,
    pReserved: PVOID,
) -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsModifyRecordsInSet_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsModifyRecordsInSet_W(
    pAddRecords: PDNS_RECORD,
    pDeleteRecords: PDNS_RECORD,
    Options: DWORD,
    hContext: HANDLE,
    pExtraList: PIP4_ARRAY,
    pReserved: PVOID,
) -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsNameCompareEx_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCompareEx_A() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsNameCompareEx_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCompareEx_UTF8() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsNameCompareEx_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCompareEx_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsNameCopy ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCopy() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsNameCopyAllocate ----
#[no_mangle]
pub unsafe extern "system" fn DnsNameCopyAllocate() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsQueryConfig ----
#[no_mangle]
pub unsafe extern "system" fn DnsQueryConfig(
    Config: DNS_CONFIG_TYPE,
    Flag: DWORD,
    pwsAdapterName: PWSTR,
    pReserved: PVOID,
    pBuffer: PVOID,
    pBufferLength: PDWORD,
) -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsQueryEx ----
#[no_mangle]
pub unsafe extern "system" fn DnsQueryEx() -> i32 {
    ERROR_OUTOFMEMORY as i32
}

// ---- DnsRecordBuild_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordBuild_UTF8() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsRecordBuild_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordBuild_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsRecordStringForType ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordStringForType() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsRecordStringForWritableType ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordStringForWritableType() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsRecordTypeForName ----
#[no_mangle]
pub unsafe extern "system" fn DnsRecordTypeForName() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsRelationalCompare_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsRelationalCompare_UTF8() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsRelationalCompare_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsRelationalCompare_W() -> u32 {
    return ERROR_OUTOFMEMORY;
}

// ---- DnsRemoveRegistrations ----
#[no_mangle]
pub unsafe extern "system" fn DnsRemoveRegistrations() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsReplaceRecordSetA ----
#[no_mangle]
pub unsafe extern "system" fn DnsReplaceRecordSetA(
    pNewSet: PDNS_RECORD,
    Options: DWORD,
    hContext: HANDLE,
    pExtraInfo: PIP4_ARRAY,
    pReserved: PVOID,
) -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsReplaceRecordSetUTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsReplaceRecordSetUTF8(
    _pNewSet: PDNS_RECORD,
    _Options: DWORD,
    _hContext: HANDLE,
    _pExtraInfo: PIP4_ARRAY,
    _pReserved: PVOID,
) -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsReplaceRecordSetW ----
#[no_mangle]
pub unsafe extern "system" fn DnsReplaceRecordSetW(
    pNewSet: PDNS_RECORD,
    Options: DWORD,
    hContext: HANDLE,
    pExtraInfo: PIP4_ARRAY,
    pReserved: PVOID,
) -> DNS_STATUS {
    return ERROR_OUTOFMEMORY;
}

// ---- DnsReplaceRecordSet_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsReplaceRecordSet_A() -> u32 {
    return ERROR_OUTOFMEMORY;
}

// ---- DnsReplaceRecordSet_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsReplaceRecordSet_UTF8() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsReplaceRecordSet_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsReplaceRecordSet_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsServiceNotificationDeregister_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsServiceNotificationDeregister_A() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsServiceNotificationDeregister_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsServiceNotificationDeregister_UTF8() -> u32 {
    // UNIMPLEMENTED;
    ERROR_OUTOFMEMORY
}

// ---- DnsServiceNotificationDeregister_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsServiceNotificationDeregister_W() -> DWORD
{
    ERROR_OUTOFMEMORY
}

// ---- DnsServiceNotificationRegister_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsServiceNotificationRegister_A() -> DNS_STATUS
{
    ERROR_OUTOFMEMORY
}

// ---- DnsServiceNotificationRegister_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsServiceNotificationRegister_UTF8() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsServiceNotificationRegister_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsServiceNotificationRegister_W() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsSetMaxNumberOfAddressesToRegister ----
#[no_mangle]
pub unsafe extern "system" fn DnsSetMaxNumberOfAddressesToRegister() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsStatusString ----
#[no_mangle]
pub unsafe extern "system" fn DnsStatusString() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsStringCopyAllocateEx ----
#[no_mangle]
pub unsafe extern "system" fn DnsStringCopyAllocateEx() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsUnicodeToUtf8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsUnicodeToUtf8() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsUpdate ----
#[no_mangle]
pub unsafe extern "system" fn DnsUpdate() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsUpdateTest_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsUpdateTest_A() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsUpdateTest_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsUpdateTest_UTF8() -> DWORD {
    ERROR_OUTOFMEMORY
}

// ---- DnsUpdateTest_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsUpdateTest_W() -> u32 {
    ERROR_OUTOFMEMORY
}

// ---- DnsUtf8ToUnicode ----
#[no_mangle]
pub unsafe extern "system" fn DnsUtf8ToUnicode() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- DnsValidateUtf8Byte ----
#[no_mangle]
pub unsafe extern "system" fn DnsValidateUtf8Byte() -> DNS_STATUS {
    ERROR_OUTOFMEMORY
}

// ---- tail stubs dropped by the assembler's 2-digit glob (files 101-105) + DnsWinsRecordFlagForString
//      (which DeepSeek never emitted at all). All are UNIMPLEMENTED stubs in stubs.c. Arg shapes
//      follow dnsapi.spec (its WORD args are declared `long` → 4-byte, so the i386 stdcall @N
//      decoration matches the spec2def .def: the WriteQuestionToBuffer pair is @24, the rest @0). ----
#[no_mangle]
pub unsafe extern "system" fn DnsWinsRecordFlagForString() -> DNS_STATUS {
    ERROR_OUTOFMEMORY as DNS_STATUS
}
#[no_mangle]
pub unsafe extern "system" fn DnsWinsRecordFlagString() -> DNS_STATUS {
    ERROR_OUTOFMEMORY as DNS_STATUS
}
#[no_mangle]
pub unsafe extern "system" fn DnsWriteQuestionToBuffer_UTF8(
    _pDnsBuffer: PVOID,
    _pdwBufferSize: LPDWORD,
    _pszName: LPSTR,
    _wType: u32,
    _Xid: u32,
    _fRecursionDesired: u32,
) -> BOOL {
    FALSE
}
#[no_mangle]
pub unsafe extern "system" fn DnsWriteQuestionToBuffer_W(
    _pDnsBuffer: PVOID,
    _pdwBufferSize: LPDWORD,
    _pszName: LPWSTR,
    _wType: u32,
    _Xid: u32,
    _fRecursionDesired: u32,
) -> BOOL {
    ERROR_OUTOFMEMORY as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn DnsWriteReverseNameStringForIpAddress() -> DNS_STATUS {
    ERROR_OUTOFMEMORY as DNS_STATUS
}
#[no_mangle]
pub unsafe extern "system" fn DnsNotifyResolver() -> DNS_STATUS {
    ERROR_OUTOFMEMORY as DNS_STATUS
}
