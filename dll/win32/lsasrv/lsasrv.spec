@ stub DsRolerDcAsDc
@ stub DsRolerDcAsReplica
@ stub DsRolerDemoteDc
@ stub DsRolerGetDcOperationProgress
@ stub DsRolerGetDcOperationResults
@ stub LsaIAddNameToLogonSession
@ stdcall LsaIAllocateHeap(long) LsapAllocateHeap
@ stdcall LsaIAllocateHeapZero(long) LsapAllocateHeapZero
@ stub LsaIAuditAccountLogon
@ stub LsaIAuditAccountLogonEx
@ stub LsaIAuditKdcEvent
@ stub LsaIAuditKerberosLogon
@ stub LsaIAuditLogonUsingExplicitCreds
@ stub LsaIAuditNotifyPackageLoad
@ stub LsaIAuditPasswordAccessEvent
@ stub LsaIAuditSamEvent
@ stub LsaICallPackage
@ stub LsaICallPackageEx
@ stub LsaICallPackagePassthrough
@ stub LsaICancelNotification
@ stub LsaIChangeSecretCipherKey
@ stub LsaICryptProtectData
@ stub LsaICryptUnprotectData
@ stub LsaIDsNotifiedObjectChange
@ stub LsaIEnumerateSecrets
@ stub LsaIEventNotify
@ stub LsaIFilterSids
@ stub LsaIForestTrustFindMatch
@ stub LsaIFreeForestTrustInfo
@ stdcall LsaIFreeHeap(ptr) LsapFreeHeap
@ stub LsaIFreeReturnBuffer
@ stub LsaIFree_LSAI_PRIVATE_DATA #DATA
@ stub LsaIFree_LSAI_SECRET_ENUM_BUFFER
@ stdcall LsaIFree_LSAPR_ACCOUNT_ENUM_BUFFER(ptr)
@ stdcall LsaIFree_LSAPR_CR_CIPHER_VALUE(ptr)
@ stub LsaIFree_LSAPR_POLICY_DOMAIN_INFORMATION
@ stdcall LsaIFree_LSAPR_POLICY_INFORMATION(long ptr)
@ stdcall LsaIFree_LSAPR_PRIVILEGE_ENUM_BUFFER(ptr)
@ stdcall LsaIFree_LSAPR_PRIVILEGE_SET(ptr)
@ stdcall LsaIFree_LSAPR_REFERENCED_DOMAIN_LIST(ptr)
@ stdcall LsaIFree_LSAPR_SR_SECURITY_DESCRIPTOR(ptr)
@ stdcall LsaIFree_LSAPR_TRANSLATED_NAMES(ptr)
@ stdcall LsaIFree_LSAPR_TRANSLATED_SIDS(ptr)
@ stub LsaIFree_LSAPR_TRUSTED_DOMAIN_INFO
@ stub LsaIFree_LSAPR_TRUSTED_ENUM_BUFFER
@ stub LsaIFree_LSAPR_TRUSTED_ENUM_BUFFER_EX
@ stub LsaIFree_LSAPR_TRUST_INFORMATION
@ stub LsaIFree_LSAPR_UNICODE_STRING
@ stub LsaIFree_LSAPR_UNICODE_STRING_BUFFER
@ stub LsaIFree_LSAP_SITENAME_INFO
@ stub LsaIFree_LSAP_SITE_INFO
@ stub LsaIFree_LSAP_SUBNET_INFO
@ stub LsaIFree_LSAP_UPN_SUFFIXES
@ stub LsaIFree_LSA_FOREST_TRUST_COLLISION_INFORMATION
@ stub LsaIFree_LSA_FOREST_TRUST_INFORMATION
@ stub LsaIGetBootOption
@ stub LsaIGetCallInfo
@ stub LsaIGetForestTrustInformation
@ stub LsaIGetLogonGuid
@ stub LsaIGetNbAndDnsDomainNames
@ stub LsaIGetPrivateData
@ stub LsaIGetSerialNumberPolicy
@ stub LsaIGetSerialNumberPolicy2
@ stub LsaIGetSiteName
@ stub LsaIHealthCheck
@ stub LsaIImpersonateClient
@ stub LsaIInitializeWellKnownSids
@ stub LsaIIsClassIdLsaClass
@ stub LsaIIsDsPaused
@ stub LsaIKerberosRegisterTrustNotification
@ stub LsaILookupWellKnownName
@ stub LsaINotifyChangeNotification
@ stub LsaINotifyNetlogonParametersChangeW
@ stub LsaINotifyPasswordChanged
@ stdcall LsaIOpenPolicyTrusted(ptr)
@ stub LsaIQueryForestTrustInfo
@ stub LsaIQueryInformationPolicyTrusted
@ stub LsaIQuerySiteInfo
@ stub LsaIQuerySubnetInfo
@ stub LsaIQueryUpnSuffixes
@ stub LsaIRegisterNotification
@ stub LsaIRegisterPolicyChangeNotificationCallback
@ stub LsaISafeMode
@ stub LsaISamIndicatedDsStarted
@ stub LsaISetBootOption
@ stub LsaISetClientDnsHostName
@ stub LsaISetLogonGuidInLogonSession
@ stub LsaISetPrivateData
@ stub LsaISetSerialNumberPolicy
@ stub LsaISetTimesSecret
@ stub LsaISetupWasRun
@ stub LsaITestCall
@ stub LsaIUnregisterAllPolicyChangeNotificationCallback
@ stub LsaIUnregisterPolicyChangeNotificationCallback
@ stub LsaIUpdateForestTrustInformation
@ stub LsaIWriteAuditEvent
@ stub LsapAuOpenSam
@ stub LsapCheckBootMode
@ stub LsapDsDebugInitialize
@ stub LsapDsInitializeDsStateInfo
@ stub LsapDsInitializePromoteInterface
@ stdcall LsapInitLsa()
@ stdcall LsarAddPrivilegesToAccount(ptr ptr)
@ stdcall LsarClose(ptr)
@ stdcall LsarCreateAccount(ptr ptr long ptr)
@ stdcall LsarCreateSecret(ptr ptr long ptr)
@ stdcall LsarCreateTrustedDomain(ptr ptr long ptr)
@ stdcall LsarCreateTrustedDomainEx(ptr ptr ptr long ptr)
@ stdcall LsarDelete(ptr)
@ stdcall LsarEnumerateAccounts(ptr ptr ptr long)
@ stdcall LsarEnumeratePrivileges(ptr ptr ptr long)
@ stdcall LsarEnumeratePrivilegesAccount(ptr ptr)
@ stdcall LsarEnumerateTrustedDomains(ptr ptr ptr long)
@ stdcall LsarEnumerateTrustedDomainsEx(ptr long ptr long)
@ stdcall LsarGetQuotasForAccount(ptr ptr)
@ stdcall LsarGetSystemAccessAccount(ptr ptr)
@ stdcall LsarLookupNames(ptr long ptr ptr ptr long ptr)
@ stdcall LsarLookupPrivilegeDisplayName(ptr ptr long long ptr ptr)
@ stdcall LsarLookupPrivilegeName(ptr ptr ptr)
@ stdcall LsarLookupPrivilegeValue(ptr ptr ptr)
@ stdcall LsarLookupSids(ptr ptr ptr ptr long ptr)
@ stdcall LsarLookupSids2(ptr ptr ptr ptr long long long long)
@ stdcall LsarOpenAccount(ptr ptr long ptr)
@ stdcall LsarOpenPolicy(ptr ptr long ptr)
@ stdcall LsarOpenPolicySce(ptr ptr long ptr)
@ stdcall LsarOpenSecret(ptr ptr long ptr)
@ stdcall LsarOpenTrustedDomain(ptr ptr long ptr)
@ stdcall LsarOpenTrustedDomainByName(ptr ptr long ptr)
@ stdcall LsarQueryDomainInformationPolicy(ptr long ptr)
@ stdcall LsarQueryForestTrustInformation(ptr ptr long ptr)
@ stdcall LsarQueryInfoTrustedDomain(ptr long ptr)
@ stdcall LsarQueryInformationPolicy(ptr long ptr)
@ stdcall LsarQuerySecret(ptr ptr ptr ptr ptr)
@ stdcall LsarQuerySecurityObject(ptr long ptr)
@ stdcall LsarQueryTrustedDomainInfo(ptr ptr long ptr)
@ stdcall LsarQueryTrustedDomainInfoByName(ptr ptr long ptr)
@ stdcall LsarRemovePrivilegesFromAccount(ptr long ptr)
@ stdcall LsarSetDomainInformationPolicy(ptr long long)
@ stdcall LsarSetForestTrustInformation(ptr ptr long ptr long ptr)
@ stdcall LsarSetInformationPolicy(ptr long ptr)
@ stdcall LsarSetInformationTrustedDomain(ptr long ptr)
@ stdcall LsarSetQuotasForAccount(ptr ptr)
@ stdcall LsarSetSecret(ptr ptr ptr)
@ stdcall LsarSetSecurityObject(ptr long ptr)
@ stdcall LsarSetSystemAccessAccount(ptr long)
@ stdcall LsarSetTrustedDomainInfoByName(ptr ptr long long)
@ stdcall ServiceInit()

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ InitializeLsaExtension
@ stub -version=0xA00+ IsTraceLevelEnabled
@ stub -version=0xA00+ LsaDbLookupSidChainRequest
@ stub -version=0xA00+ LsaIAddNamesToLogonSession
@ stub -version=0xA00+ LsaIAdjustTokenObjectIntegrity
@ stub -version=0xA00+ LsaIAdtAuditingEnabledByCategory
@ stub -version=0xA00+ LsaIAdtAuditingEnabledBySubCategory
@ stub -version=0xA00+ LsaIAllowProtectedCredLogon
@ stub -version=0xA00+ LsaIAuditInitializeParametersAndWriteEvent
@ stub -version=0xA00+ LsaIAuditLogonEx
@ stub -version=0xA00+ LsaIAuditReplay
@ stub -version=0xA00+ LsaICheckProtectedUserByTokenInfo
@ stub -version=0xA00+ LsaICheckRestrictedMode
@ stub -version=0xA00+ LsaIClearOldSyskey
@ stub -version=0xA00+ LsaIContextToHandleNoRef
@ stub -version=0xA00+ LsaICopyToTokenInfoFromHandle
@ stub -version=0xA00+ LsaICryptProtectDataEx
@ stub -version=0xA00+ LsaICryptUnprotectDataEx
@ stub -version=0xA00+ LsaIDereferenceCredHandle
@ stub -version=0xA00+ LsaIDereferenceCtxtHandle
@ stub -version=0xA00+ LsaIDeriveCredentialKey
@ stub -version=0xA00+ LsaIEfsAcceptSmartcardCredentials
@ stub -version=0xA00+ LsaIEqualLogonProcessName
@ stub -version=0xA00+ LsaIEqualSupplementalTokenInfo
@ stub -version=0xA00+ LsaIEventWritePackageNoCredential
@ stub -version=0xA00+ LsaIEventWritePackageNotCacheLogonUser
@ stub -version=0xA00+ LsaIExtractTargetInfo
@ stub -version=0xA00+ LsaIFilterInboundNamespace
@ stub -version=0xA00+ LsaIFilterNamespace
@ stub -version=0xA00+ LsaIFlushIdentityCacheForSid
@ stub -version=0xA00+ LsaIFreeFilterInboundNamespaceResult
@ stub -version=0xA00+ LsaIFreeSupplementalTokenInfo
@ stub -version=0xA00+ LsaIGetCcgClient
@ stub -version=0xA00+ LsaIGetClientOsInfo
@ stub -version=0xA00+ LsaIGetNameFromLuid
@ stub -version=0xA00+ LsaIGetNego2Package
@ stub -version=0xA00+ LsaIGetNetworkInfo
@ stub -version=0xA00+ LsaIGetRemoteCredGuardLogonBuffer
@ stub -version=0xA00+ LsaIGetRemoteCredGuardSupplementalCreds
@ stub -version=0xA00+ LsaIGetSupplementalTokenInfo
@ stub -version=0xA00+ LsaIGetTokenInformationForLocalUser
@ stub -version=0xA00+ LsaIInitializeNetlogonFuncPtrs
@ stub -version=0xA00+ LsaIIsContainerized
@ stub -version=0xA00+ LsaIIsDomainWithinForest
@ stub -version=0xA00+ LsaIIsInEmulatedDomainJoinMode
@ stub -version=0xA00+ LsaIIsLastInteractiveLogonInfoEnabled
@ stub -version=0xA00+ LsaIIsLocalHost
@ stub -version=0xA00+ LsaIIsMachineSecureByDefault
@ stub -version=0xA00+ LsaIIsSuppressChannelBindingInfo
@ stub -version=0xA00+ LsaIIsTargetPrivate
@ stub -version=0xA00+ LsaIIsTrustedDomainsEnabled
@ stub -version=0xA00+ LsaIIsUserMSA
@ stub -version=0xA00+ LsaILookupUserAccountType
@ stub -version=0xA00+ LsaIModifyPerformanceCounter
@ stub -version=0xA00+ LsaINoConnectedUserPolicy
@ stub -version=0xA00+ LsaINoMoreWin2KDomain
@ stub -version=0xA00+ LsaINotifyGCStatusChange
@ stub -version=0xA00+ LsaINotifyNewPassword
@ stub -version=0xA00+ LsaIQueryForestTrustInformation
@ stub -version=0xA00+ LsaIQueryPackageAttrInLogonSession
@ stub -version=0xA00+ LsaIReferenceCredHandle
@ stub -version=0xA00+ LsaIReferenceCtxtHandle
@ stub -version=0xA00+ LsaIRegisterLogonSessionCallback
@ stub -version=0xA00+ LsaIRenewCertificate
@ stub -version=0xA00+ LsaIReplicateClientObject
@ stub -version=0xA00+ LsaIRetrieveCurrentUserSid
@ stub -version=0xA00+ LsaISanitizeSAMName
@ stub -version=0xA00+ LsaISetLogonInfo
@ stub -version=0xA00+ LsaISetNewSyskey
@ stub -version=0xA00+ LsaISetPackageAttrInLogonSession
@ stub -version=0xA00+ LsaISetSupplementalTokenInfo
@ stub -version=0xA00+ LsaISetTokenDacl
@ stub -version=0xA00+ LsaISetUserFlags
@ stub -version=0xA00+ LsaITransformAuthorizationData
@ stub -version=0xA00+ LsaIUnregisterLogonSessionCallback
@ stub -version=0xA00+ LsaIUpdateKerbMaxTokenSize
@ stub -version=0xA00+ LsaIUpdateLogonSession
@ stub -version=0xA00+ LsaIValidateTargetInfo
@ stub -version=0xA00+ LsaIVerifyCachability
@ stub -version=0xA00+ LsaIVerifyCachabilityEx
@ stub -version=0xA00+ LsaIWasLogonNotifiedOfProfileLoad
@ stub -version=0xA00+ LsaIWriteKdcAuthenticationEvent
@ stub -version=0xA00+ LsaLookupPerfCounterAddAmount
@ stub -version=0xA00+ LsaLookupPerfCounterAddLargeAmount
@ stub -version=0xA00+ LsaLookupPerfCounterDecrementCount
@ stub -version=0xA00+ LsaLookupPerfCounterDecrementLargeCount
@ stub -version=0xA00+ LsaLookupPerfCounterIncrementCount
@ stub -version=0xA00+ LsaLookupPerfCounterIncrementLargeCount
@ stub -version=0xA00+ LsapAdtGetCallerProcessInfo
@ stub -version=0xA00+ LsapAdtWriteLog
@ stub -version=0xA00+ LsapAllocateLsaHeap
@ stub -version=0xA00+ LsapAllocatePrivateHeap
@ stub -version=0xA00+ LsapAuditFailed
@ stub -version=0xA00+ LsapBuildPrivilegeAuditString
@ stub -version=0xA00+ LsapCloseHandle
@ stub -version=0xA00+ LsapCompareDomainNames
@ stub -version=0xA00+ LsapCrServerGetSessionKey
@ stub -version=0xA00+ LsapCrServerGetSessionKeySafe
@ stub -version=0xA00+ LsapDbAcquireLockEx
@ stub -version=0xA00+ LsapDbApplyTransaction
@ stub -version=0xA00+ LsapDbBuildObjectCaches
@ stub -version=0xA00+ LsapDbCloseHandle
@ stub -version=0xA00+ LsapDbCloseObject
@ stub -version=0xA00+ LsapDbCopyUnicodeAttribute
@ stub -version=0xA00+ LsapDbCopyUnicodeAttributeNoAlloc
@ stub -version=0xA00+ LsapDbCreateObject
@ stub -version=0xA00+ LsapDbDeleteAttributesObject
@ stub -version=0xA00+ LsapDbDeleteObject
@ stub -version=0xA00+ LsapDbDereferenceHandle
@ stub -version=0xA00+ LsapDbDereferenceObject
@ stub -version=0xA00+ LsapDbEnumerateSids
@ stub -version=0xA00+ LsapDbEnumerateTrustedDomainsEx
@ stub -version=0xA00+ LsapDbExpAcquireReadLockTrustedDomainList
@ stub -version=0xA00+ LsapDbExpAcquireWriteLockTrustedDomainList
@ stub -version=0xA00+ LsapDbExpConvertReadLockTrustedDomainListToExclusive
@ stub -version=0xA00+ LsapDbExpConvertWriteLockTrustedDomainListToShared
@ stub -version=0xA00+ LsapDbExpIsCacheBuilding
@ stub -version=0xA00+ LsapDbExpIsCacheValid
@ stub -version=0xA00+ LsapDbExpIsLockedTrustedDomainList
@ stub -version=0xA00+ LsapDbExpMakeCacheBuilding
@ stub -version=0xA00+ LsapDbExpMakeCacheInvalid
@ stub -version=0xA00+ LsapDbExpMakeCacheValid
@ stub -version=0xA00+ LsapDbExpReleaseLockTrustedDomainList
@ stub -version=0xA00+ LsapDbFreeAttributes
@ stub -version=0xA00+ LsapDbFreeTrustedDomainsEx
@ stub -version=0xA00+ LsapDbGetDbObjectTypeName
@ stub -version=0xA00+ LsapDbGetDbPolicyHandle
@ stub -version=0xA00+ LsapDbGetSecretType
@ stub -version=0xA00+ LsapDbInitializeAttribute
@ stub -version=0xA00+ LsapDbIsStatusConnectionFailure
@ stub -version=0xA00+ LsapDbLookupAddListReferencedDomains
@ stub -version=0xA00+ LsapDbLookupCreateListReferencedDomains
@ stub -version=0xA00+ LsapDbLookupGetDomainInfo
@ stub -version=0xA00+ LsapDbLookupListReferencedDomains
@ stub -version=0xA00+ LsapDbLookupMergeDisjointReferencedDomains
@ stub -version=0xA00+ LsapDbLookupNameChainRequest
@ stub -version=0xA00+ LsapDbLookupNamesInPrimaryDomain
@ stub -version=0xA00+ LsapDbLookupSidsInPrimaryDomain
@ stub -version=0xA00+ LsapDbMakeGuidAttribute
@ stub -version=0xA00+ LsapDbMakeSidAttribute
@ stub -version=0xA00+ LsapDbMakeUnicodeAttribute
@ stub -version=0xA00+ LsapDbOpenObject
@ stub -version=0xA00+ LsapDbQueryInformationPolicy
@ stub -version=0xA00+ LsapDbReadAttribute
@ stub -version=0xA00+ LsapDbReadAttributesObject
@ stub -version=0xA00+ LsapDbReferenceObject
@ stub -version=0xA00+ LsapDbReleaseLockEx
@ stub -version=0xA00+ LsapDbSecretIsMachineAcc
@ stub -version=0xA00+ LsapDbSidToLogicalNameObject
@ stub -version=0xA00+ LsapDbSlowEnumerateTrustedDomains
@ stub -version=0xA00+ LsapDbUpdateCountCompUnmappedNames
@ stub -version=0xA00+ LsapDbVerifyHandle
@ stub -version=0xA00+ LsapDbVerifyInfoQueryTrustedDomain
@ stub -version=0xA00+ LsapDbVerifyInfoSetTrustedDomain
@ stub -version=0xA00+ LsapDbWriteAttributesObject
@ stub -version=0xA00+ LsapDomainRenameHandlerForLogonSessions
@ stub -version=0xA00+ LsapDsUnitializeDsStateInfo
@ stub -version=0xA00+ LsapDssetupInitializeGetPrimaryDomainInformationOpState
@ stub -version=0xA00+ LsapDuplicateSid
@ stub -version=0xA00+ LsapDuplicateString
@ stub -version=0xA00+ LsapFreeLsaHeap
@ stub -version=0xA00+ LsapFreePrivateHeap
@ stub -version=0xA00+ LsapFreeString
@ stub -version=0xA00+ LsapGetAccountDomainHandle
@ stub -version=0xA00+ LsapGetCapeNamesForCap
@ stub -version=0xA00+ LsapGetGlobalRestrictAnonymous
@ stub -version=0xA00+ LsapGetHourlyLogLevel
@ stub -version=0xA00+ LsapGetLogonSessionAccountInfoEx
@ stub -version=0xA00+ LsapGetLookupRestrictIsolatedNameLevel
@ stub -version=0xA00+ LsapGetPolicyHandle
@ stub -version=0xA00+ LsapGetWellKnownSid
@ stub -version=0xA00+ LsapInitializeLsaDb
@ stub -version=0xA00+ LsapIsBuiltinDomain
@ stub -version=0xA00+ LsapIsSamOpened
@ stub -version=0xA00+ LsapOpenSam
@ stub -version=0xA00+ LsapQueryClientInfo
@ stub -version=0xA00+ LsapRemoveTrailingDot
@ stub -version=0xA00+ LsapRpcCopySid
@ stub -version=0xA00+ LsapRpcCopyUnicodeString
@ stub -version=0xA00+ LsapRtlValidateControllerTrustedDomain
@ stub -version=0xA00+ LsapRtlValidateControllerTrustedDomainByHandle
@ stub -version=0xA00+ LsapSetErrorInfo
@ stub -version=0xA00+ LsapSidListSize
@ stub -version=0xA00+ LsapTraceEvent
@ stub -version=0xA00+ LsapTraceEventWithData
@ stub -version=0xA00+ LsapTruncateUnicodeString
@ stub -version=0xA00+ LsarDeleteObject
@ stub -version=0xA00+ LsarRetrievePrivateData
@ stub -version=0xA00+ LsarStorePrivateData
@ stub -version=0xA00+ QueryLsaInterface
@ stub -version=0xA00+ TracePrint
@ stub -version=0xA00+ TracePrintCallerInformation
@ stub -version=0xA00+ _fgs__LSAPR_TRUSTED_DOMAIN_FULL_INFORMATION2
@ stub -version=0xA00+ _fgs__LSAPR_TRUSTED_DOMAIN_INFORMATION_EX2
@ stub -version=0xA00+ _fgs__LSAPR_TRUSTED_ENUM_BUFFER
@ stub -version=0xA00+ _fgs__LSAPR_TRUSTED_ENUM_BUFFER_EX
@ stub -version=0xA00+ _fgs__LSAPR_TRUST_INFORMATION
@ stub -version=0xA00+ _fgu__LSAPR_TRUSTED_DOMAIN_INFO
# END win10-surface stubs
