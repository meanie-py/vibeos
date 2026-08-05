use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn GetAccountDomainSid(
    Sid: *mut PRPC_SID
) -> NTSTATUS {
    let mut PolicyHandle: LSAPR_HANDLE = core::ptr::null_mut();
    let mut PolicyInfo: PLSAPR_POLICY_INFORMATION = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = LsaIOpenPolicyTrusted(&mut PolicyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = LsarQueryInformationPolicy(PolicyHandle,
                                     PolicyAccountDomainInformation,
                                     &mut PolicyInfo);
    if !NT_SUCCESS(Status) {
        LsarClose(&mut PolicyHandle);
        return Status;
    }

    Length = RtlLengthSid((*PolicyInfo).PolicyAccountDomainInfo.Sid);

    *Sid = (RtlAllocateHeap(RtlGetProcessHeap(), 0, ((Length) as u64) as u32)).cast();
    if (*Sid).is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        if !PolicyInfo.is_null() {
            LsaIFree_LSAPR_POLICY_INFORMATION(PolicyAccountDomainInformation,
                                   PolicyInfo);
        }
        if !PolicyHandle.is_null() {
            LsarClose(&mut PolicyHandle);
        }
        return Status;
    }

    memcpy((*Sid).cast(), (*PolicyInfo).PolicyAccountDomainInfo.Sid, (Length) as usize);

    if !PolicyInfo.is_null() {
        LsaIFree_LSAPR_POLICY_INFORMATION(PolicyAccountDomainInformation,
                               PolicyInfo);
    }

    if !PolicyHandle.is_null() {
        LsarClose(&mut PolicyHandle);
    }

    Status
}
unsafe extern "C" fn MsvpCheckPassword(
    UserPwdData: PLSA_SAM_PWD_DATA,
    UserInfo: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut UserNtPassword: ENCRYPTED_NT_OWF_PASSWORD = core::mem::zeroed();
    let mut UserLmPassword: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut UserLmPasswordPresent: BOOLEAN = (FALSE) as u8;
    let mut UserNtPasswordPresent: BOOLEAN = (FALSE) as u8;
    let mut LmPwdString: OEM_STRING = core::mem::zeroed();
    let mut LmPwdBuffer: [CHAR; 15] = [0; 15];
    let mut Status: NTSTATUS;

    LmPwdString.Length = 15;
    LmPwdString.MaximumLength = 15;
    LmPwdString.Buffer = LmPwdBuffer.as_mut_ptr();
    ZeroMemory(LmPwdString.Buffer as *mut core::ffi::c_void, LmPwdString.MaximumLength as usize);

    Status = RtlUpcaseUnicodeStringToOemString(
        &mut LmPwdString,
        (*UserPwdData).PlainPwd,
        (FALSE) as u8,
    );
    if NT_SUCCESS(Status) {
        Status = SystemFunction006(
            LmPwdString.Buffer,
            &mut UserLmPassword as *mut _ as LPSTR,
        );
        if NT_SUCCESS(Status) {
            UserLmPasswordPresent = (TRUE) as u8;
        }
    }

    Status = SystemFunction007(
        (*UserPwdData).PlainPwd,
        &mut UserNtPassword as *mut _ as LPBYTE,
    );
    if NT_SUCCESS(Status) {
        UserNtPasswordPresent = (TRUE) as u8;
    }

    Status = STATUS_WRONG_PASSWORD;

    'done: loop {
        if (*UserInfo).All.NtPasswordPresent == (FALSE) as u8
            && (*UserInfo).All.LmPasswordPresent == (FALSE) as u8
        {
            Status = STATUS_SUCCESS;
            break 'done;
        }

        if UserNtPasswordPresent != 0 && (*UserInfo).All.NtPasswordPresent != 0 {
            if RtlEqualMemory(
                &UserNtPassword as *const _ as *const core::ffi::c_void,
                (*UserInfo).All.NtOwfPassword.Buffer as *const core::ffi::c_void,
                core::mem::size_of_val(&ENCRYPTED_NT_OWF_PASSWORD),
            ) != 0
            {
                Status = STATUS_SUCCESS;
                break 'done;
            }
        }

        if UserLmPasswordPresent != 0 && (*UserInfo).All.LmPasswordPresent != 0 {
            if RtlEqualMemory(
                &UserLmPassword as *const _ as *const core::ffi::c_void,
                (*UserInfo).All.LmOwfPassword.Buffer as *const core::ffi::c_void,
                core::mem::size_of_val(&ENCRYPTED_LM_OWF_PASSWORD),
            ) != 0
            {
                Status = STATUS_SUCCESS;
                break 'done;
            }
        }

        break 'done;
    }

    Status
}
unsafe extern "C" fn MsvpCheckLogonHours(
    LogonHours: PSAMPR_LOGON_HOURS,
    LogonTime: LARGE_INTEGER,
) -> bool {
    true
}
unsafe extern "C" fn MsvpCheckWorkstations(
    WorkStations: *mut RPC_UNICODE_STRING,
    ComputerName: PWSTR,
) -> bool {
    let mut pStart: PWSTR;
    let mut pEnd: PWSTR;
    let mut bFound: bool = false;

    if (*WorkStations).Length == 0 || (*WorkStations).Buffer.is_null() {
        return true;
    }

    pStart = (*WorkStations).Buffer;
    loop {
        pEnd = wcschr(pStart, ',' as u16);
        if !pEnd.is_null() {
            *pEnd = UNICODE_NULL;
        }

        if _wcsicmp(ComputerName, pStart) == 0 {
            bFound = true;
            if !pEnd.is_null() {
                *pEnd = ',' as u16;
            }
            break;
        }

        if pEnd.is_null() {
            break;
        }

        *pEnd = ',' as u16;
        pStart = pEnd.offset(1);
    }

    return bFound;
}
unsafe extern "C" fn SamValidateNormalUser(
    UserName: PUNICODE_STRING,
    PwdData: PLSA_SAM_PWD_DATA,
    ComputerName: PUNICODE_STRING,
    AccountDomainSidPtr: *mut PRPC_SID,
    UserHandlePtr: *mut SAMPR_HANDLE,
    UserInfoPtr: *mut PSAMPR_USER_INFO_BUFFER,
    SubStatus: PNTSTATUS,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut AccountDomainSid: PRPC_SID;
    let mut Names: [RPC_UNICODE_STRING; 1] = core::mem::zeroed();
    let mut UserHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut RelativeIds: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut Use: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut UserInfo: PSAMPR_USER_INFO_BUFFER = core::ptr::null_mut();
    let mut LogonTime: LARGE_INTEGER = core::mem::zeroed();

    NtQuerySystemTime(&mut LogonTime);

    Status = GetAccountDomainSid(&mut AccountDomainSid);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: loop {
        Status = SamIConnect(
            core::ptr::null_mut(),
            &mut ServerHandle,
            SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN,
            (TRUE) as u8,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_LOOKUP,
            AccountDomainSid,
            &mut DomainHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Names[0].Length = (*UserName).Length;
        Names[0].MaximumLength = (*UserName).MaximumLength;
        Names[0].Buffer = (*UserName).Buffer;

        Status = SamrLookupNamesInDomain(
            DomainHandle,
            1,
            Names.as_mut_ptr(),
            &mut RelativeIds,
            &mut Use,
        );
        if !NT_SUCCESS(Status) {
            Status = STATUS_NO_SUCH_USER;
            break 'done;
        }

        if (*Use.Element.offset(0)) != SidTypeUser {
            Status = STATUS_NO_SUCH_USER;
            break 'done;
        }

        Status = SamrOpenUser(
            DomainHandle,
            USER_READ_GENERAL | USER_READ_LOGON |
            USER_READ_ACCOUNT | USER_READ_PREFERENCES,
            *RelativeIds.Element.offset(0),
            &mut UserHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SamrQueryInformationUser(UserHandle, UserAllInformation, &mut UserInfo);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if ((*UserInfo).All.UserAccountControl & USER_PASSWORD_NOT_REQUIRED) == 0 {
            Status = MsvpCheckPassword(PwdData, UserInfo);
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if *RelativeIds.Element.offset(0) != DOMAIN_USER_RID_ADMIN {
            if (*UserInfo).All.UserAccountControl & USER_ACCOUNT_DISABLED != 0 {
                *SubStatus = STATUS_ACCOUNT_DISABLED;
                Status = STATUS_ACCOUNT_RESTRICTION;
                break 'done;
            }

            if (*UserInfo).All.UserAccountControl & USER_ACCOUNT_AUTO_LOCKED != 0 {
                *SubStatus = STATUS_ACCOUNT_LOCKED_OUT;
                Status = STATUS_ACCOUNT_RESTRICTION;
                break 'done;
            }

            if (*LogonTime.QuadPart) >= *((*UserInfo).All.AccountExpires.QuadPart as *const UINT64) {
                *SubStatus = STATUS_ACCOUNT_EXPIRED;
                Status = STATUS_ACCOUNT_RESTRICTION;
                break 'done;
            }

            if (*LogonTime.QuadPart) >= *((*UserInfo).All.PasswordMustChange.QuadPart as *const UINT64) {
                if *((*UserInfo).All.PasswordLastSet.QuadPart as *const UINT64) == 0 {
                    *SubStatus = STATUS_PASSWORD_MUST_CHANGE;
                } else {
                    *SubStatus = STATUS_PASSWORD_EXPIRED;
                }
                Status = STATUS_ACCOUNT_RESTRICTION;
                break 'done;
            }

            if !MsvpCheckLogonHours(&mut (*UserInfo).All.LogonHours, LogonTime) {
                *SubStatus = STATUS_INVALID_LOGON_HOURS;
                Status = STATUS_ACCOUNT_RESTRICTION;
                break 'done;
            }

            if !MsvpCheckWorkstations(&mut (*UserInfo).All.WorkStations, (*ComputerName).Buffer) {
                *SubStatus = STATUS_INVALID_WORKSTATION;
                Status = STATUS_ACCOUNT_RESTRICTION;
                break 'done;
            }
        }

        break 'done;
    }

    if NT_SUCCESS(Status) {
        *UserHandlePtr = UserHandle;
        *AccountDomainSidPtr = AccountDomainSid;
        *UserInfoPtr = UserInfo;
    } else {
        if !AccountDomainSid.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid as *mut _);
        }

        if !UserHandle.is_null() {
            SamrCloseHandle(&mut UserHandle);
        }

        SamIFree_SAMPR_USER_INFO_BUFFER(UserInfo, UserAllInformation);
    }

    SamIFree_SAMPR_ULONG_ARRAY(&mut RelativeIds);
    SamIFree_SAMPR_ULONG_ARRAY(&mut Use);

    if !DomainHandle.is_null() {
        SamrCloseHandle(&mut DomainHandle);
    }

    if !ServerHandle.is_null() {
        SamrCloseHandle(&mut ServerHandle);
    }

    Status
}
unsafe extern "C" fn GetNtAuthorityDomainSid(
     Sid: *mut PRPC_SID) -> NTSTATUS {
    let NtAuthority: SID_IDENTIFIER_AUTHORITY = SECURITY_NT_AUTHORITY;
    let mut Length: ULONG = 0;

    Length = RtlLengthRequiredSid(0);
    *Sid = RtlAllocateHeap(RtlGetProcessHeap(), 0, ((Length) as u64) as u32) as PRPC_SID;
    if (*Sid).is_null()
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlInitializeSid((*Sid).cast(), core::ptr::addr_of!(NtAuthority) as *mut SID_IDENTIFIER_AUTHORITY, 0);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn SamValidateUser(
    LogonType: SECURITY_LOGON_TYPE,
    LogonUserName: PUNICODE_STRING,
    LogonDomain: PUNICODE_STRING,
    LogonPwdData: PLSA_SAM_PWD_DATA,
    ComputerName: PUNICODE_STRING,
    SpecialAccount: PBOOL,
    AccountDomainSidPtr: *mut PRPC_SID,
    UserHandlePtr: *mut SAMPR_HANDLE,
    UserInfoPtr: *mut PSAMPR_USER_INFO_BUFFER,
    SubStatus: PNTSTATUS,
) -> NTSTATUS {
    static NtAuthorityU: _UNICODE_STRING = RTL_CONSTANT_STRING(b"N\0T\0 \0A\0U\0T\0H\0O\0R\0I\0T\0Y\0\0\0");
    static LocalServiceU: _UNICODE_STRING = RTL_CONSTANT_STRING(b"L\0o\0c\0a\0l\0S\0e\0r\0v\0i\0c\0e\0\0\0");
    static NetworkServiceU: _UNICODE_STRING = RTL_CONSTANT_STRING(b"N\0e\0t\0w\0o\0r\0k\0S\0e\0r\0v\0i\0c\0e\0\0\0");

    let mut Status: NTSTATUS = STATUS_SUCCESS;

    *SpecialAccount = FALSE;
    *UserInfoPtr = core::ptr::null_mut();
    *SubStatus = STATUS_SUCCESS;

    if RtlEqualUnicodeString(LogonDomain, &NtAuthorityU as *const _ as *mut _, (TRUE) as u8) != 0 {
        *SpecialAccount = TRUE;

        Status = GetNtAuthorityDomainSid(AccountDomainSidPtr);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if RtlEqualUnicodeString(LogonUserName, &LocalServiceU as *const _ as *mut _, (TRUE) as u8) != 0 {
            if LogonType != Service {
                return STATUS_LOGON_FAILURE;
            }

            *UserInfoPtr = (RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<SAMPR_USER_ALL_INFORMATION>()) as u32,
            ) as *mut SAMPR_USER_ALL_INFORMATION).cast();
            if (*UserInfoPtr).is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (**UserInfoPtr).All.UserId = SECURITY_LOCAL_SERVICE_RID;
            (**UserInfoPtr).All.PrimaryGroupId = SECURITY_LOCAL_SERVICE_RID;
        } else if RtlEqualUnicodeString(LogonUserName, &NetworkServiceU as *const _ as *mut _, (TRUE) as u8) != 0 {
            if LogonType != Service {
                return STATUS_LOGON_FAILURE;
            }

            *UserInfoPtr = (RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<SAMPR_USER_ALL_INFORMATION>()) as u32,
            ) as *mut SAMPR_USER_ALL_INFORMATION).cast();
            if (*UserInfoPtr).is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (**UserInfoPtr).All.UserId = SECURITY_NETWORK_SERVICE_RID;
            (**UserInfoPtr).All.PrimaryGroupId = SECURITY_NETWORK_SERVICE_RID;
        } else {
            return STATUS_NO_SUCH_USER;
        }
    } else {
        Status = SamValidateNormalUser(
            LogonUserName,
            LogonPwdData,
            ComputerName,
            AccountDomainSidPtr,
            UserHandlePtr,
            UserInfoPtr,
            SubStatus,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Authentication Package DLL
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security Account Manager (SAM) related functions
//  * COPYRIGHT:   Copyright 2013 Eric Kohl <eric.kohl@reactos.org>
//  */
//
// #include "precomp.h"
//
// #include "wine/debug.h"
// WINE_DEFAULT_DEBUG_CHANNEL(msv1_0_sam);
//
//
// static
// NTSTATUS
// GetAccountDomainSid(
//      PRPC_SID *Sid)
// {
//     LSAPR_HANDLE PolicyHandle = NULL;
//     PLSAPR_POLICY_INFORMATION PolicyInfo = NULL;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Status = LsaIOpenPolicyTrusted(&PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("LsaIOpenPolicyTrusted() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = LsarQueryInformationPolicy(PolicyHandle,
//                                         PolicyAccountDomainInformation,
//                                         &PolicyInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("LsarQueryInformationPolicy() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Length = RtlLengthSid(PolicyInfo->PolicyAccountDomainInfo.Sid);
//
//     *Sid = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length);
//     if (*Sid == NULL)
//     {
//         ERR("Failed to allocate SID\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     memcpy(*Sid, PolicyInfo->PolicyAccountDomainInfo.Sid, Length);
//
// done:
//     if (PolicyInfo != NULL)
//         LsaIFree_LSAPR_POLICY_INFORMATION(PolicyAccountDomainInformation,
//                                           PolicyInfo);
//
//     if (PolicyHandle != NULL)
//         LsarClose(&PolicyHandle);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// MsvpCheckPassword(
//      PLSA_SAM_PWD_DATA UserPwdData,
//      PSAMPR_USER_INFO_BUFFER UserInfo)
// {
//     ENCRYPTED_NT_OWF_PASSWORD UserNtPassword;
//     ENCRYPTED_LM_OWF_PASSWORD UserLmPassword;
//     BOOLEAN UserLmPasswordPresent = FALSE;
//     BOOLEAN UserNtPasswordPresent = FALSE;
//     OEM_STRING LmPwdString;
//     CHAR LmPwdBuffer[15];
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n", UserPwdData, UserInfo);
//
//     /* Calculate the LM password and hash for the users password */
//     LmPwdString.Length = 15;
//     LmPwdString.MaximumLength = 15;
//     LmPwdString.Buffer = LmPwdBuffer;
//     ZeroMemory(LmPwdString.Buffer, LmPwdString.MaximumLength);
//
//     Status = RtlUpcaseUnicodeStringToOemString(&LmPwdString,
//                                                UserPwdData->PlainPwd,
//                                                FALSE);
//     if (NT_SUCCESS(Status))
//     {
//         /* Calculate the LM hash value of the users password */
//         Status = SystemFunction006(LmPwdString.Buffer,
//                                    (LPSTR)&UserLmPassword);
//         if (NT_SUCCESS(Status))
//         {
//             UserLmPasswordPresent = TRUE;
//         }
//     }
//
//     /* Calculate the NT hash of the users password */
//     Status = SystemFunction007(UserPwdData->PlainPwd,
//                                (LPBYTE)&UserNtPassword);
//     if (NT_SUCCESS(Status))
//     {
//         UserNtPasswordPresent = TRUE;
//     }
//
//     Status = STATUS_WRONG_PASSWORD;
//
//     /* Succeed, if no password has been set */
//     if (UserInfo->All.NtPasswordPresent == FALSE &&
//         UserInfo->All.LmPasswordPresent == FALSE)
//     {
//         TRACE("No password check!\n");
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     /* Succeed, if NT password matches */
//     if (UserNtPasswordPresent && UserInfo->All.NtPasswordPresent)
//     {
//         TRACE("Check NT password hashes:\n");
//         if (RtlEqualMemory(&UserNtPassword,
//                            UserInfo->All.NtOwfPassword.Buffer,
//                            sizeof(ENCRYPTED_NT_OWF_PASSWORD)))
//         {
//             TRACE("  success!\n");
//             Status = STATUS_SUCCESS;
//             goto done;
//         }
//
//         TRACE("  failed!\n");
//     }
//
//     /* Succeed, if LM password matches */
//     if (UserLmPasswordPresent && UserInfo->All.LmPasswordPresent)
//     {
//         TRACE("Check LM password hashes:\n");
//         if (RtlEqualMemory(&UserLmPassword,
//                            UserInfo->All.LmOwfPassword.Buffer,
//                            sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//         {
//             TRACE("  success!\n");
//             Status = STATUS_SUCCESS;
//             goto done;
//         }
//         TRACE("  failed!\n");
//     }
//
// done:
//     return Status;
// }
//
//
// static
// bool
// MsvpCheckLogonHours(
//      PSAMPR_LOGON_HOURS LogonHours,
//      LARGE_INTEGER LogonTime)
// {
//     return true;
// }
//
//
// static
// bool
// MsvpCheckWorkstations(
//      PRPC_UNICODE_STRING WorkStations,
//      PWSTR ComputerName)
// {
//     PWSTR pStart, pEnd;
//     bool bFound = false;
//
//     TRACE("MsvpCheckWorkstations(%p %S)\n", WorkStations, ComputerName);
//
//     if (WorkStations->Length == 0 || WorkStations->Buffer == NULL)
//     {
//         TRACE("No workstations!\n");
//         return true;
//     }
//
//     TRACE("Workstations: %wZ\n", WorkStations);
//
//     pStart = WorkStations->Buffer;
//     for (;;)
//     {
//         pEnd = wcschr(pStart, ',');
//         if (pEnd != NULL)
//             *pEnd = UNICODE_NULL;
//
//         TRACE("Comparing '%S' and '%S'\n", ComputerName, pStart);
//         if (_wcsicmp(ComputerName, pStart) == 0)
//         {
//             bFound = true;
//             if (pEnd != NULL)
//                 *pEnd = ',';
//             break;
//         }
//
//         if (pEnd == NULL)
//             break;
//
//         *pEnd = ',';
//         pStart = pEnd + 1;
//     }
//
//     TRACE("Found allowed workstation: %s\n", (bFound) ? "Yes" : "No");
//
//     return bFound;
// }
//
//
// static
// NTSTATUS
// SamValidateNormalUser(
//      PUNICODE_STRING UserName,
//      PLSA_SAM_PWD_DATA PwdData,
//      PUNICODE_STRING ComputerName,
//      PRPC_SID* AccountDomainSidPtr,
//      SAMPR_HANDLE* UserHandlePtr,
//      PSAMPR_USER_INFO_BUFFER* UserInfoPtr,
//      PNTSTATUS SubStatus)
// {
//     NTSTATUS Status;
//     SAMPR_HANDLE ServerHandle = NULL;
//     SAMPR_HANDLE DomainHandle = NULL;
//     PRPC_SID AccountDomainSid;
//     RPC_UNICODE_STRING Names[1];
//     SAMPR_HANDLE UserHandle = NULL;
//     SAMPR_ULONG_ARRAY RelativeIds = {0, NULL};
//     SAMPR_ULONG_ARRAY Use = {0, NULL};
//     PSAMPR_USER_INFO_BUFFER UserInfo = NULL;
//     LARGE_INTEGER LogonTime;
//
//     /* Get the logon time */
//     NtQuerySystemTime(&LogonTime);
//
//     /* Get the account domain SID */
//     Status = GetAccountDomainSid(&AccountDomainSid);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("GetAccountDomainSid() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Connect to the SAM server */
//     Status = SamIConnect(NULL, &ServerHandle, SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN, TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamIConnect() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Open the account domain */
//     Status = SamrOpenDomain(ServerHandle, DOMAIN_LOOKUP, AccountDomainSid, &DomainHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamrOpenDomain failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Names[0].Length = UserName->Length;
//     Names[0].MaximumLength = UserName->MaximumLength;
//     Names[0].Buffer = UserName->Buffer;
//
//     /* Try to get the RID for the user name */
//     Status = SamrLookupNamesInDomain(DomainHandle, 1, Names, &RelativeIds, &Use);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamrLookupNamesInDomain failed (Status %08lx)\n", Status);
//         Status = STATUS_NO_SUCH_USER;
//         // FIXME: Try without domain?
//         goto done;
//     }
//
//     /* Fail, if it is not a user account */
//     if (Use.Element[0] != SidTypeUser)
//     {
//         ERR("Account is not a user account!\n");
//         Status = STATUS_NO_SUCH_USER;
//         goto done;
//     }
//
//     /* Open the user object */
//     Status = SamrOpenUser(DomainHandle,
//                           USER_READ_GENERAL | USER_READ_LOGON |
//                           USER_READ_ACCOUNT | USER_READ_PREFERENCES, /* FIXME */
//                           RelativeIds.Element[0],
//                           &UserHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamrOpenUser failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SamrQueryInformationUser(UserHandle, UserAllInformation, &UserInfo);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SamrQueryInformationUser failed (Status %08lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("UserName: %wZ\n", &UserInfo->All.UserName);
//
//     /* Check the password */
//     if ((UserInfo->All.UserAccountControl & USER_PASSWORD_NOT_REQUIRED) == 0)
//     {
//         Status = MsvpCheckPassword(PwdData, UserInfo);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("MsvpCheckPassword failed (Status %08lx)\n", Status);
//             goto done;
//         }
//     }
//
//     /* Check account restrictions for non-administrator accounts */
//     if (RelativeIds.Element[0] != DOMAIN_USER_RID_ADMIN)
//     {
//         /* Check if the account has been disabled */
//         if (UserInfo->All.UserAccountControl & USER_ACCOUNT_DISABLED)
//         {
//             ERR("Account disabled!\n");
//             *SubStatus = STATUS_ACCOUNT_DISABLED;
//             Status = STATUS_ACCOUNT_RESTRICTION;
//             goto done;
//         }
//
//         /* Check if the account has been locked */
//         if (UserInfo->All.UserAccountControl & USER_ACCOUNT_AUTO_LOCKED)
//         {
//             ERR("Account locked!\n");
//             *SubStatus = STATUS_ACCOUNT_LOCKED_OUT;
//             Status = STATUS_ACCOUNT_RESTRICTION;
//             goto done;
//         }
//
//         /* Check if the account expired */
//         if (LogonTime.QuadPart >= *(UINT64*)&UserInfo->All.AccountExpires)
//         {
//             ERR("Account expired!\n");
//             *SubStatus = STATUS_ACCOUNT_EXPIRED;
//             Status = STATUS_ACCOUNT_RESTRICTION;
//             goto done;
//         }
//
//         /* Check if the password expired */
//         if (LogonTime.QuadPart >= *(UINT64*)&UserInfo->All.PasswordMustChange)
//         {
//             ERR("Password expired!\n");
//             if (*(UINT64*)&UserInfo->All.PasswordLastSet == 0)
//                 *SubStatus = STATUS_PASSWORD_MUST_CHANGE;
//             else
//                 *SubStatus = STATUS_PASSWORD_EXPIRED;
//
//             Status = STATUS_ACCOUNT_RESTRICTION;
//             goto done;
//         }
//
//         /* Check logon hours */
//         if (!MsvpCheckLogonHours(&UserInfo->All.LogonHours, LogonTime))
//         {
//             ERR("Invalid logon hours!\n");
//             *SubStatus = STATUS_INVALID_LOGON_HOURS;
//             Status = STATUS_ACCOUNT_RESTRICTION;
//             goto done;
//         }
//
//         /* Check workstations */
//         if (!MsvpCheckWorkstations(&UserInfo->All.WorkStations, ComputerName->Buffer))
//         {
//             ERR("Invalid workstation!\n");
//             *SubStatus = STATUS_INVALID_WORKSTATION;
//             Status = STATUS_ACCOUNT_RESTRICTION;
//             goto done;
//         }
//     }
// done:
//     if (NT_SUCCESS(Status))
//     {
//         *UserHandlePtr = UserHandle;
//         *AccountDomainSidPtr = AccountDomainSid;
//         *UserInfoPtr = UserInfo;
//     }
//     else
//     {
//         if (AccountDomainSid != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, AccountDomainSid);
//
//         if (UserHandle != NULL)
//             SamrCloseHandle(&UserHandle);
//
//         SamIFree_SAMPR_USER_INFO_BUFFER(UserInfo,
//                                         UserAllInformation);
//     }
//
//     SamIFree_SAMPR_ULONG_ARRAY(&RelativeIds);
//     SamIFree_SAMPR_ULONG_ARRAY(&Use);
//
//     if (DomainHandle != NULL)
//         SamrCloseHandle(&DomainHandle);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// GetNtAuthorityDomainSid(
//      PRPC_SID *Sid)
// {
//     SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     ULONG Length = 0;
//
//     Length = RtlLengthRequiredSid(0);
//     *Sid = RtlAllocateHeap(RtlGetProcessHeap(), 0, Length);
//     if (*Sid == NULL)
//     {
//         ERR("Failed to allocate SID\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlInitializeSid(*Sid,&NtAuthority, 0);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// SamValidateUser(
//      SECURITY_LOGON_TYPE LogonType,
//      PUNICODE_STRING LogonUserName,
//      PUNICODE_STRING LogonDomain,
//      PLSA_SAM_PWD_DATA LogonPwdData,
//      PUNICODE_STRING ComputerName,
//      PBOOL SpecialAccount,
//      PRPC_SID* AccountDomainSidPtr,
//      SAMPR_HANDLE* UserHandlePtr,
//      PSAMPR_USER_INFO_BUFFER* UserInfoPtr,
//      PNTSTATUS SubStatus)
// {
//     static const UNICODE_STRING NtAuthorityU = RTL_CONSTANT_STRING(L"NT AUTHORITY");
//     static const UNICODE_STRING LocalServiceU = RTL_CONSTANT_STRING(L"LocalService");
//     static const UNICODE_STRING NetworkServiceU = RTL_CONSTANT_STRING(L"NetworkService");
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     *SpecialAccount = FALSE;
//     *UserInfoPtr = NULL;
//     *SubStatus = STATUS_SUCCESS;
//
//     /* Check for special accounts */
//     // FIXME: Windows does not do this that way!! (msv1_0 does not contain these hardcoded values)
//     if (RtlEqualUnicodeString(LogonDomain, &NtAuthorityU, TRUE))
//     {
//         *SpecialAccount = TRUE;
//
//         /* Get the authority domain SID */
//         Status = GetNtAuthorityDomainSid(AccountDomainSidPtr);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("GetNtAuthorityDomainSid() failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         if (RtlEqualUnicodeString(LogonUserName, &LocalServiceU, TRUE))
//         {
//             TRACE("SpecialAccount: LocalService\n");
//
//             if (LogonType != Service)
//                 return STATUS_LOGON_FAILURE;
//
//             *UserInfoPtr = RtlAllocateHeap(RtlGetProcessHeap(),
//                                            HEAP_ZERO_MEMORY,
//                                            sizeof(SAMPR_USER_ALL_INFORMATION));
//             if (*UserInfoPtr == NULL)
//                 return STATUS_INSUFFICIENT_RESOURCES;
//
//             (*UserInfoPtr)->All.UserId = SECURITY_LOCAL_SERVICE_RID;
//             (*UserInfoPtr)->All.PrimaryGroupId = SECURITY_LOCAL_SERVICE_RID;
//         }
//         else if (RtlEqualUnicodeString(LogonUserName, &NetworkServiceU, TRUE))
//         {
//             TRACE("SpecialAccount: NetworkService\n");
//
//             if (LogonType != Service)
//                 return STATUS_LOGON_FAILURE;
//
//             *UserInfoPtr = RtlAllocateHeap(RtlGetProcessHeap(),
//                                            HEAP_ZERO_MEMORY,
//                                            sizeof(SAMPR_USER_ALL_INFORMATION));
//             if (*UserInfoPtr == NULL)
//                 return STATUS_INSUFFICIENT_RESOURCES;
//
//             (*UserInfoPtr)->All.UserId = SECURITY_NETWORK_SERVICE_RID;
//             (*UserInfoPtr)->All.PrimaryGroupId = SECURITY_NETWORK_SERVICE_RID;
//         }
//         else
//         {
//             return STATUS_NO_SUCH_USER;
//         }
//     }
//     else
//     {
//         TRACE("NormalAccount\n");
//         Status = SamValidateNormalUser(LogonUserName,
//                                        LogonPwdData,
//                                        ComputerName,
//                                        AccountDomainSidPtr,
//                                        UserHandlePtr,
//                                        UserInfoPtr,
//                                        SubStatus);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("SamValidateNormalUser() failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
