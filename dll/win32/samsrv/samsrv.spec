@ stub SamIAccountRestrictions
@ stub SamIAddDSNameToAlias
@ stub SamIAddDSNameToGroup
@ stub SamIAmIGC
@ stub SamIChangePasswordForeignUser
@ stub SamIChangePasswordForeignUser2
@ stdcall SamIConnect(ptr ptr long long)
@ stub SamICreateAccountByRid
@ stub SamIDemote
@ stub SamIDemoteUndo
@ stub SamIDoFSMORoleChange
@ stub SamIDsCreateObjectInDomain
@ stub SamIDsSetObjectInformation
@ stub SamIEnumerateAccountRids
@ stub SamIEnumerateInterdomainTrustAccountsForUpgrade
@ stub SamIFloatingSingleMasterOpEx
@ stub SamIFreeSidAndAttributesList
@ stub SamIFreeSidArray
@ stdcall SamIFreeVoid(ptr)
@ stdcall SamIFree_SAMPR_ALIAS_INFO_BUFFER(ptr long)
@ stdcall SamIFree_SAMPR_DISPLAY_INFO_BUFFER(ptr long)
@ stdcall SamIFree_SAMPR_DOMAIN_INFO_BUFFER(ptr long)
@ stdcall SamIFree_SAMPR_ENUMERATION_BUFFER(ptr)
@ stdcall SamIFree_SAMPR_GET_GROUPS_BUFFER(ptr)
@ stdcall SamIFree_SAMPR_GET_MEMBERS_BUFFER(ptr)
@ stdcall SamIFree_SAMPR_GROUP_INFO_BUFFER(ptr long)
@ stdcall SamIFree_SAMPR_PSID_ARRAY(ptr)
@ stdcall SamIFree_SAMPR_RETURNED_USTRING_ARRAY(ptr)
@ stdcall SamIFree_SAMPR_SR_SECURITY_DESCRIPTOR(ptr)
@ stdcall SamIFree_SAMPR_ULONG_ARRAY(ptr)
@ stdcall SamIFree_SAMPR_USER_INFO_BUFFER(ptr long)
@ stub SamIFree_UserInternal6Information
@ stub SamIGCLookupNames
@ stub SamIGCLookupSids
@ stub SamIGetAliasMembership
@ stub SamIGetBootKeyInformation
@ stub SamIGetDefaultAdministratorName
@ stub SamIGetFixedAttributes
@ stub SamIGetInterdomainTrustAccountPasswordsForUpgrade
@ stub SamIGetPrivateData
@ stub SamIGetResourceGroupMembershipsTransitive
@ stub SamIGetSerialNumberDomain
@ stub SamIGetUserLogonInformation
@ stub SamIGetUserLogonInformation2
@ stub SamIGetUserLogonInformationEx
@ stub SamIImpersonateNullSession
@ stub SamIIncrementPerformanceCounter
@ stdcall SamIInitialize()
@ stub SamIIsDownlevelDcUpgrade
@ stub SamIIsExtendedSidMode
@ stub SamIIsRebootAfterPromotion
@ stub SamIIsSetupInProgress
@ stub SamILoadDownlevelDatabase
@ stub SamILoopbackConnect
@ stub SamIMixedDomain
@ stub SamIMixedDomain2
@ stub SamINT4UpgradeInProgress
@ stub SamINetLogonPing
@ stub SamINotifyDelta
@ stub SamINotifyRoleChange
@ stub SamINotifyServerDelta
@ stub SamIOpenAccount
@ stub SamIOpenUserByAlternateId
@ stub SamIPromote
@ stub SamIPromoteUndo
@ stub SamIQueryServerRole
@ stub SamIQueryServerRole2
@ stub SamIRemoveDSNameFromAlias
@ stub SamIRemoveDSNameFromGroup
@ stub SamIRelaceDownlevelDatabase
@ stub SamIResetBadPwdCountOnPdc
@ stub SamIRetrievePrimaryCredentials
@ stub SamIRevertNullSession
@ stub SamISameSite
@ stub SamISetAuditingInformation
@ stub SamISetMixedDomainFlag
@ stub SamISetPasswordForeignUser
@ stub SamISetPasswordForeignUser2
@ stub SamISetPasswordInfoOnPdc
@ stub SamISetPrivateData
@ stub SamISetSerialNumberDomain
@ stub SamIStorePrimaryCredentials
@ stub SamIUPNFromUserHandle
@ stub SamIUnLoadDownlevelDatabase
@ stub SamIUpdateLogonStatistics
@ stub SampAbortSingleLoopbackTask
@ stub SampAccountControlToFlags
@ stub SampAcquireSamLockExclusive
@ stub SampAcquireWriteLock
@ stub SampCommitBufferedWrites
@ stub SampConvertNt4SdToNt5Sd
@ stub SampDsChangePasswordUser
@ stub SampFlagsToAccountControl
@ stub SampGetDefaultSecurityDescriptorForClass
@ stub SampGetSerialNumberDomain2
@ stdcall SampInitializeRegistry()
@ stub SampInitializeSdConversion
@ stub SampInvalidateDomainCache
@ stub SampInvalidateRidRange
@ stub SampNetLogonNotificationRequired
@ stub SampNotifyReplicatedInChange
@ stub SampProcessSingleLoopbackTask
@ stub SampReleaseSamLockExclusive
@ stub SampReleaseWriteLock
@ stub SampRtlConvertUlongToUnicodeString
@ stub SampSetSerialNumberDomain2
@ stub SampUsingDsData
@ stub SampWriteGroupType
@ stdcall SamrAddMemberToAlias(ptr ptr)
@ stdcall SamrAddMemberToGroup(ptr long long)
@ stdcall SamrAddMultipleMembersToAlias(ptr ptr)
@ stdcall SamrChangePasswordUser(ptr long ptr ptr long ptr ptr long ptr long ptr)
@ stdcall SamrCloseHandle(ptr)
@ stdcall SamrConnect(ptr ptr long)
@ stdcall SamrCreateAliasInDomain(ptr ptr long ptr ptr)
@ stdcall SamrCreateGroupInDomain(ptr ptr long ptr ptr)
@ stdcall SamrCreateUser2InDomain(ptr ptr long long ptr ptr ptr)
@ stdcall SamrCreateUserInDomain(ptr ptr long ptr ptr)
@ stdcall SamrDeleteAlias(ptr)
@ stdcall SamrDeleteGroup(ptr)
@ stdcall SamrDeleteUser(ptr)
@ stdcall SamrEnumerateAliasesInDomain(ptr ptr ptr long ptr)
@ stdcall SamrEnumerateDomainsInSamServer(ptr ptr ptr long ptr)
@ stdcall SamrEnumerateGroupsInDomain(ptr ptr ptr long ptr)
@ stdcall SamrEnumerateUsersInDomain(ptr ptr long ptr long ptr)
@ stdcall SamrGetAliasMembership(ptr ptr ptr)
@ stdcall SamrGetGroupsForUser(ptr ptr)
@ stdcall SamrGetMembersInAlias(ptr ptr)
@ stdcall SamrGetMembersInGroup(ptr ptr)
@ stdcall SamrGetUserDomainPasswordInformation(ptr ptr)
@ stdcall SamrLookupDomainInSamServer(ptr ptr ptr)
@ stdcall SamrLookupIdsInDomain(ptr long ptr ptr ptr)
@ stdcall SamrLookupNamesInDomain(ptr long ptr ptr ptr)
@ stdcall SamrOpenAlias(ptr long long ptr)
@ stdcall SamrOpenDomain(ptr long ptr ptr)
@ stdcall SamrOpenGroup(ptr long long ptr)
@ stdcall SamrOpenUser(ptr long long ptr)
@ stdcall SamrQueryDisplayInformation(ptr long long long long ptr ptr ptr)
@ stdcall SamrQueryInformationAlias(ptr long ptr)
@ stdcall SamrQueryInformationDomain(ptr long ptr)
@ stdcall SamrQueryInformationGroup(ptr long ptr)
@ stdcall SamrQueryInformationUser(ptr long ptr)
@ stdcall SamrQuerySecurityObject(ptr long ptr)
@ stdcall SamrRemoveMemberFromAlias(ptr ptr)
@ stdcall SamrRemoveMemberFromForeignDomain(ptr ptr)
@ stdcall SamrRemoveMemberFromGroup(ptr long)
@ stdcall SamrRemoveMultipleMembersFromAlias(ptr ptr)
@ stdcall SamrRidToSid(ptr long ptr)
@ stdcall SamrSetInformationAlias(ptr long ptr)
@ stdcall SamrSetInformationDomain(ptr long ptr)
@ stdcall SamrSetInformationGroup(ptr long ptr)
@ stdcall SamrSetInformationUser(ptr long ptr)
@ stdcall SamrSetMemberAttributesOfGroup(ptr long long)
@ stdcall SamrSetSecurityObject(ptr long ptr)
@ stdcall SamrShutdownSamServer(ptr)
@ stdcall SamrTestPrivateFunctionsDomain(ptr)
@ stdcall SamrTestPrivateFunctionsUser(ptr)
@ stdcall SamrUnicodeChangePasswordUser2(ptr ptr ptr ptr ptr long ptr ptr)
; EOF

# BEGIN win10-surface stubs
# Generated by tools/win10_surface.py from Windows 10.0.26100.9457; do not edit by hand.
# Names that build exports and this spec does not declare. Dormant below
# DLL_EXPORT_VERSION 0xA00; give a name a real line above and re-run to drop it.
@ stub -version=0xA00+ RtlDeleteElementGenericTable2
@ stub -version=0xA00+ RtlInitializeGenericTable2
@ stub -version=0xA00+ RtlInsertElementGenericTable2
@ stub -version=0xA00+ RtlLookupElementGenericTable2
@ stub -version=0xA00+ SAM_MIDL_user_allocate
@ stub -version=0xA00+ SAM_MIDL_user_free
@ stub -version=0xA00+ SamDsExtAlloc
@ stub -version=0xA00+ SamDsExtFree
@ stub -version=0xA00+ SamIClaimIsValid
@ stub -version=0xA00+ SamIConvertSecurityAttributesToClaimsBlob
@ stub -version=0xA00+ SamICopyCurrentDomainAccountSettings
@ stub -version=0xA00+ SamICreateKrbTgt
@ stub -version=0xA00+ SamIDecodeClaimsBlob
@ stub -version=0xA00+ SamIDecodeClaimsBlobIntoClaimsSet
@ stub -version=0xA00+ SamIDecodeClaimsBlobToAuthz
@ stub -version=0xA00+ SamIFreeAuthzSecurityAttributesInfo
@ stub -version=0xA00+ SamIFreeClaimsBlob
@ stub -version=0xA00+ SamIFreeDecodedClaimsSet
@ stub -version=0xA00+ SamIFreeLookupNamesInfo
@ stub -version=0xA00+ SamIFreeLookupSidsInfo
@ stub -version=0xA00+ SamIFreeOidList
@ stub -version=0xA00+ SamIFreeRealmList
@ stub -version=0xA00+ SamIFreeSecurityAttributesInfo
@ stub -version=0xA00+ SamIGetConfigurationOidList
@ stub -version=0xA00+ SamIGetResourceGroupMembershipsTransitive2
@ stub -version=0xA00+ SamIGetUserLogonInformation3
@ stub -version=0xA00+ SamIHandleObjectUpdate
@ stub -version=0xA00+ SamILookupNamesBySid
@ stub -version=0xA00+ SamILookupNamesInDomain
@ stub -version=0xA00+ SamILookupSidsByName
@ stub -version=0xA00+ SamILoopbackConnectEx
@ stub -version=0xA00+ SamIPurgeSecrets
@ stub -version=0xA00+ SamIQueryAccountSecretsCachability
@ stub -version=0xA00+ SamIQueryCapabilities
@ stub -version=0xA00+ SamIQueryRealmList
@ stub -version=0xA00+ SamIRandomizeStoredPassword
@ stub -version=0xA00+ SamIRandomizeStoredPasswordWithoutExpirationCheck
@ stub -version=0xA00+ SamIReplaceDownlevelDatabase
@ stub -version=0xA00+ SamIReplicateAccountData
@ stub -version=0xA00+ SamIRetrieveMultiplePrimaryCredentials
@ stub -version=0xA00+ SamIRetrieveNGCKeyCredential
@ stub -version=0xA00+ SamIScorePassword
@ stub -version=0xA00+ SamISetMachinePassword
@ stub -version=0xA00+ SamISetPasswordForeignUser3
@ stub -version=0xA00+ SamISetPasswordInfoOnDc
@ stub -version=0xA00+ SamITransformClaims
@ stub -version=0xA00+ SamIUninitialize
@ stub -version=0xA00+ SamIValidateAccountName
@ stub -version=0xA00+ SamIValidateNewAccountName
@ stub -version=0xA00+ SampAcquireReadLock
@ stub -version=0xA00+ SampAddAccountToGroupMembers
@ stub -version=0xA00+ SampAddAccountsAndApplyMemberships
@ stub -version=0xA00+ SampAddDeltaTime
@ stub -version=0xA00+ SampAddNonLocalDomainRelativeMemberships
@ stub -version=0xA00+ SampAddSameDomainMemberToGlobalOrUniversalGroup
@ stub -version=0xA00+ SampAddUserToGroup
@ stub -version=0xA00+ SampAlInvalidateAliasInformation
@ stub -version=0xA00+ SampAllocateNextCurrentRidFromIndex
@ stub -version=0xA00+ SampApplyDomainUpdatesForAllDomains
@ stub -version=0xA00+ SampAssignPrimaryGroup
@ stub -version=0xA00+ SampAuditAccountEnableDisableChange
@ stub -version=0xA00+ SampAuditAccountNameChange
@ stub -version=0xA00+ SampAuditAnyEvent
@ stub -version=0xA00+ SampAuditGroupTypeChange
@ stub -version=0xA00+ SampAuditSidHistory
@ stub -version=0xA00+ SampBuildDsNameFromSid
@ stub -version=0xA00+ SampBuildSamProtection
@ stub -version=0xA00+ SampCalculateLmAndNtOwfPasswords
@ stub -version=0xA00+ SampChangeAliasAccountName
@ stub -version=0xA00+ SampChangeGroupAccountName
@ stub -version=0xA00+ SampChangeUserAccountName
@ stub -version=0xA00+ SampCheckForAccountLockout
@ stub -version=0xA00+ SampCheckGroupTypeBits
@ stub -version=0xA00+ SampCheckSidType
@ stub -version=0xA00+ SampCompareDisplayStrings
@ stub -version=0xA00+ SampComputePasswordExpired
@ stub -version=0xA00+ SampConnect
@ stub -version=0xA00+ SampConvertUiListToApiList
@ stub -version=0xA00+ SampCopyUserSupplementalCredentialsForDCPromo
@ stub -version=0xA00+ SampCreateAccountContext2
@ stub -version=0xA00+ SampCreateAliasInDomain
@ stub -version=0xA00+ SampCreateContextEx
@ stub -version=0xA00+ SampCreateDefaultUPN
@ stub -version=0xA00+ SampCreateFullSid
@ stub -version=0xA00+ SampCreateGroupInDomain
@ stub -version=0xA00+ SampCreateUserInDomain
@ stub -version=0xA00+ SampCurrentThreadOwnsLock
@ stub -version=0xA00+ SampDeReferenceContext
@ stub -version=0xA00+ SampDecrementActiveThreads
@ stub -version=0xA00+ SampDecryptCredentialData
@ stub -version=0xA00+ SampDeleteContext
@ stub -version=0xA00+ SampDeleteDsDirsToDeleteKey
@ stub -version=0xA00+ SampDeleteKeyForPostBootPromote
@ stub -version=0xA00+ SampDeltaChangeNotify
@ stub -version=0xA00+ SampDsConvertReadAttrBlock
@ stub -version=0xA00+ SampDsGetPrimaryDomainStart
@ stub -version=0xA00+ SampDsInitializeSingleDomain
@ stub -version=0xA00+ SampDsIsRunning
@ stub -version=0xA00+ SampDsMakeAttrBlock
@ stub -version=0xA00+ SampDsSetBuiltinDomainPolicy
@ stub -version=0xA00+ SampDsSetDomainPolicy
@ stub -version=0xA00+ SampDsSetPasswordUser
@ stub -version=0xA00+ SampDsUpdateContextAttributes
@ stub -version=0xA00+ SampDuplicateGroupInfo
@ stub -version=0xA00+ SampDuplicateMachineInfo
@ stub -version=0xA00+ SampDuplicateOemGroupInfo
@ stub -version=0xA00+ SampDuplicateOemUserInfo
@ stub -version=0xA00+ SampDuplicateUnicodeString
@ stub -version=0xA00+ SampDuplicateUserInfo
@ stub -version=0xA00+ SampEncryptCredentialData
@ stub -version=0xA00+ SampExamineSid
@ stub -version=0xA00+ SampExtendDefinedDomains
@ stub -version=0xA00+ SampFlagsToAccountControlEx
@ stub -version=0xA00+ SampFreeGroupInfo
@ stub -version=0xA00+ SampFreeMachineInfo
@ stub -version=0xA00+ SampFreeOemGroupInfo
@ stub -version=0xA00+ SampFreeOemUserInfo
@ stub -version=0xA00+ SampFreeUnicodeString
@ stub -version=0xA00+ SampFreeUserInfo
@ stub -version=0xA00+ SampGenerateRandomPassword
@ stub -version=0xA00+ SampGetAccessAttribute
@ stub -version=0xA00+ SampGetAccountDomainInfo
@ stub -version=0xA00+ SampGetBehaviorVersion
@ stub -version=0xA00+ SampGetCurrentOwnerAndPrimaryGroup
@ stub -version=0xA00+ SampGetDisableOutboundRSO
@ stub -version=0xA00+ SampGetDisableRSOOnPDCForward
@ stub -version=0xA00+ SampGetDisableResetBadPwdCountForward
@ stub -version=0xA00+ SampGetDisableSingleObjectRepl
@ stub -version=0xA00+ SampGetDnsDomainNameFromIndex
@ stub -version=0xA00+ SampGetDomainContextFromIndex
@ stub -version=0xA00+ SampGetDomainObjectFromAccountContext
@ stub -version=0xA00+ SampGetDomainObjectFromIndex
@ stub -version=0xA00+ SampGetDomainServerRoleFromIndex
@ stub -version=0xA00+ SampGetDomainSidFromAccountContext
@ stub -version=0xA00+ SampGetDomainSidFromIndex
@ stub -version=0xA00+ SampGetDomainSidListForSam
@ stub -version=0xA00+ SampGetDomainUpgradeTasks
@ stub -version=0xA00+ SampGetDownLevelDomainControllersPresent
@ stub -version=0xA00+ SampGetExtendedAttribute
@ stub -version=0xA00+ SampGetExternalNameFromIndex
@ stub -version=0xA00+ SampGetFixedAttributes
@ stub -version=0xA00+ SampGetHasNeverTime
@ stub -version=0xA00+ SampGetIgnoreGCFailures
@ stub -version=0xA00+ SampGetLogLevel
@ stub -version=0xA00+ SampGetNT4UpgradeInProgress
@ stub -version=0xA00+ SampGetNewAccountSecurityNt4
@ stub -version=0xA00+ SampGetNextUnmodifiedRidFromIndex
@ stub -version=0xA00+ SampGetNoGcLogonEnforceKerberosIpCheck
@ stub -version=0xA00+ SampGetNoGcLogonEnforceNTLMCheck
@ stub -version=0xA00+ SampGetObjectSD
@ stub -version=0xA00+ SampGetObjectTypeNameFromIndex
@ stub -version=0xA00+ SampGetPasswordMustChangeWithUF_UAC
@ stub -version=0xA00+ SampGetReverseMembershipTransitive
@ stub -version=0xA00+ SampGetSamSubsystemName
@ stub -version=0xA00+ SampGetServerObjectName
@ stub -version=0xA00+ SampGetSidArrayAttribute
@ stub -version=0xA00+ SampGetSidAttribute
@ stub -version=0xA00+ SampGetSuccessAccountAuditingEnabled
@ stub -version=0xA00+ SampGetUlongArrayAttribute
@ stub -version=0xA00+ SampGetUnicodeStringAttribute
@ stub -version=0xA00+ SampGetUserAccountControlComputed
@ stub -version=0xA00+ SampGetUserAccountSettings
@ stub -version=0xA00+ SampGetWillNeverTime
@ stub -version=0xA00+ SampImpersonateClient
@ stub -version=0xA00+ SampIncreaseBadPwdCountLoopback
@ stub -version=0xA00+ SampIncrementActiveThreads
@ stub -version=0xA00+ SampIncrementNetlogonChangeLogSerialNumber
@ stub -version=0xA00+ SampIsAccountBuiltIn
@ stub -version=0xA00+ SampIsAuditingEnabled
@ stub -version=0xA00+ SampIsBuiltinDomain
@ stub -version=0xA00+ SampIsDomainHosted
@ stub -version=0xA00+ SampIsInternetAccount
@ stub -version=0xA00+ SampIsServiceRunning
@ stub -version=0xA00+ SampIsSetupInProgress
@ stub -version=0xA00+ SampLogPrint
@ stub -version=0xA00+ SampLookupContext
@ stub -version=0xA00+ SampMarkPerAttributeInvalidFromWhichFields
@ stub -version=0xA00+ SampNeedUserAccountSettingsDuringQuery
@ stub -version=0xA00+ SampNotifyAuditChange
@ stub -version=0xA00+ SampPasswordChangeNotify
@ stub -version=0xA00+ SampPasswordChangeNotifyWorker
@ stub -version=0xA00+ SampPositionOfHighestBit
@ stub -version=0xA00+ SampQueryCapabilities
@ stub -version=0xA00+ SampQueryDSRMAccountName
@ stub -version=0xA00+ SampQueryInformationUserInternal
@ stub -version=0xA00+ SampQueryPolicyValue
@ stub -version=0xA00+ SampQueryResourceString
@ stub -version=0xA00+ SampQueryUserSupplementalCredentialsRegistry
@ stub -version=0xA00+ SampReadExtendedAttributes
@ stub -version=0xA00+ SampRecordSystemSchemaVerisonInRegistry
@ stub -version=0xA00+ SampReferenceContext
@ stub -version=0xA00+ SampRegObjToDsObj
@ stub -version=0xA00+ SampReleaseReadLock
@ stub -version=0xA00+ SampRemoveAccountFromGroupMembers
@ stub -version=0xA00+ SampRemoveSameDomainMemberFromGlobalOrUniversalGroup
@ stub -version=0xA00+ SampRemoveUserFromGroup
@ stub -version=0xA00+ SampRenameKrbtgtAccount
@ stub -version=0xA00+ SampReplaceUserLogonHours
@ stub -version=0xA00+ SampReplaceUserV1aFixed
@ stub -version=0xA00+ SampRetrieveGroupV1Fixed
@ stub -version=0xA00+ SampRetrieveMultipleCredentials
@ stub -version=0xA00+ SampRetrieveUserPasswords
@ stub -version=0xA00+ SampRetrieveUserV1aFixed
@ stub -version=0xA00+ SampRevertToSelf
@ stub -version=0xA00+ SampRtlWellKnownPrivilegeCheck
@ stub -version=0xA00+ SampSetAccessAttribute
@ stub -version=0xA00+ SampSetAdminPassword
@ stub -version=0xA00+ SampSetAttributeAccess
@ stub -version=0xA00+ SampSetCheckpointInfo
@ stub -version=0xA00+ SampSetComputerObjectDsName
@ stub -version=0xA00+ SampSetDSRMPasswordWorker
@ stub -version=0xA00+ SampSetErrorInfo
@ stub -version=0xA00+ SampSetExtendedAttributeAccess
@ stub -version=0xA00+ SampSetFixedAttributes
@ stub -version=0xA00+ SampSetGlobalDsSids
@ stub -version=0xA00+ SampSetPassword
@ stub -version=0xA00+ SampSetPasswordInfoOnPdcByHandle
@ stub -version=0xA00+ SampSetPasswordInfoOnPdcByIndex
@ stub -version=0xA00+ SampSetTransactionDomain
@ stub -version=0xA00+ SampSetTransactionWithinDomain
@ stub -version=0xA00+ SampSetUnicodeStringAttribute
@ stub -version=0xA00+ SampSetUserAccountControl
@ stub -version=0xA00+ SampSplitSid
@ stub -version=0xA00+ SampStoreObjectAttributes
@ stub -version=0xA00+ SampStringFromGuid
@ stub -version=0xA00+ SampTraceEvent
@ stub -version=0xA00+ SampUpdateAccountDisabledFlag
@ stub -version=0xA00+ SampUpdateComputedUserAccountControlBits
@ stub -version=0xA00+ SampUpdateMixedModeAndFindDomain
@ stub -version=0xA00+ SampUpdatePerformanceCounters
@ stub -version=0xA00+ SampUpgradeUserParmsActual
@ stub -version=0xA00+ SampValidateDomainCacheCallback
@ stub -version=0xA00+ SampValidateDomainControllerCreation
@ stub -version=0xA00+ SampValidatePwdSettingAttempt
@ stub -version=0xA00+ SampValidateRegAttributes
@ stub -version=0xA00+ SampWriteEventLog
@ stub -version=0xA00+ SamrEnumerateUsersInDomain2
@ stub -version=0xA00+ SamrLookupNamesInDomain2
@ stub -version=0xA00+ SamrQueryInformationUser2
@ stub -version=0xA00+ SamrValidatePassword
# END win10-surface stubs
