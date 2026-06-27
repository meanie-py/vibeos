use crate::prelude::*;
use core::ffi::c_void;

// Enum-variant bare-name consts (kept module-local — see prelude.rs note).
// SECURITY_LOGON_TYPE (ntsecapi.h).
const Interactive: SECURITY_LOGON_TYPE = advapi32_sys::_SECURITY_LOGON_TYPE_Interactive;
const Network: SECURITY_LOGON_TYPE = advapi32_sys::_SECURITY_LOGON_TYPE_Network;
const Batch: SECURITY_LOGON_TYPE = advapi32_sys::_SECURITY_LOGON_TYPE_Batch;
const Service: SECURITY_LOGON_TYPE = advapi32_sys::_SECURITY_LOGON_TYPE_Service;
// MSV1_0_LOGON_SUBMIT_TYPE (ntsecapi.h).
const MsV1_0InteractiveLogon: advapi32_sys::MSV1_0_LOGON_SUBMIT_TYPE =
    advapi32_sys::_MSV1_0_LOGON_SUBMIT_TYPE_MsV1_0InteractiveLogon;

// All WINAPI advapi32/kernel32 exports — __stdcall on i386 → extern "system"
// (emits `_Name@N`; extern "C" would emit undecorated `_Name` and fail to link).
extern "system" {
    fn AddAccessAllowedAce(pAcl: PACL, dwAceRevision: u32, AccessMask: u32, pSid: PSID) -> i32;
    fn AllocateAndInitializeSid( pIdentifierAuthority: *mut SID_IDENTIFIER_AUTHORITY, nSubAuthorityCount: u8, nSubAuthority0: u32, nSubAuthority1: u32, nSubAuthority2: u32, nSubAuthority3: u32, nSubAuthority4: u32, nSubAuthority5: u32, nSubAuthority6: u32, nSubAuthority7: u32, pSid: *mut PSID, ) -> i32;
    fn FreeSid(pSid: PSID) -> PVOID;
    fn GetLastError() -> u32;
    fn GetLengthSid(pSid: PSID) -> u32;
    fn InitializeAcl(pAcl: PACL, nAclLength: u32, dwAclRevision: u32) -> i32;
    fn InitializeSecurityDescriptor( pSecurityDescriptor: PSECURITY_DESCRIPTOR, dwRevision: u32, ) -> i32;
    fn MakeSelfRelativeSD( pAbsoluteSecurityDescriptor: PSECURITY_DESCRIPTOR, pSelfRelativeSecurityDescriptor: PSECURITY_DESCRIPTOR, lpdwBufferLength: LPDWORD, ) -> i32;
    fn SetSecurityDescriptorDacl( pSecurityDescriptor: PSECURITY_DESCRIPTOR, bDaclPresent: i32, pDacl: PACL, bDaclDefaulted: i32, ) -> i32;
    fn SetSecurityDescriptorGroup( pSecurityDescriptor: PSECURITY_DESCRIPTOR, pGroup: PSID, bGroupDefaulted: i32, ) -> i32;
    fn SetSecurityDescriptorOwner( pSecurityDescriptor: PSECURITY_DESCRIPTOR, pOwner: PSID, bOwnerDefaulted: i32, ) -> i32;
}

// ---- file-scope preamble (translated) ----
pub static AdvapiTokenSourceName: [CHAR; 9] = [b'A' as CHAR, b'd' as CHAR, b'v' as CHAR, b'a' as CHAR, b'p' as CHAR, b'i' as CHAR, b' ' as CHAR, b' ' as CHAR, 0];

pub static mut LsaHandle: HANDLE = core::ptr::null_mut();
pub static mut AuthenticationPackage: ULONG = 0;

unsafe fn OpenLogonLsaHandle() -> NTSTATUS {
    let mut LogonProcessName: LSA_STRING = core::mem::zeroed();
    let mut PackageName: LSA_STRING = core::mem::zeroed();
    let mut SecurityMode: LSA_OPERATIONAL_MODE = 0;
    let mut Status: NTSTATUS;

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(LogonProcessName) as PANSI_STRING,
        b"User32LogonProcess\0".as_ptr() as PCSZ,
    );

    Status = LsaRegisterLogonProcess(
        core::ptr::addr_of_mut!(LogonProcessName),
        core::ptr::addr_of_mut!(LsaHandle),
        core::ptr::addr_of_mut!(SecurityMode),
    );

    'done: loop {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(PackageName) as PANSI_STRING,
            b"MICROSOFT_AUTHENTICATION_PACKAGE_V1_0\0".as_ptr() as PCSZ,
        );

        Status = LsaLookupAuthenticationPackage(
            LsaHandle,
            core::ptr::addr_of_mut!(PackageName),
            core::ptr::addr_of_mut!(AuthenticationPackage),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if LsaHandle != core::ptr::null_mut() {
            Status = LsaDeregisterLogonProcess(LsaHandle);
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CloseLogonLsaHandle() -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if LsaHandle != core::ptr::null_mut() {
        Status = LsaDeregisterLogonProcess(LsaHandle);
        if !NT_SUCCESS(Status) {
            TRACE!(b"LsaDeregisterLogonProcess failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
        }
    }

    Status
}

unsafe fn CreateDefaultProcessSecurityCommon(
    TokenHandle: HANDLE,
    Sd: *mut PSECURITY_DESCRIPTOR,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Success: BOOL = 0;
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut OwnerOfToken: PTOKEN_OWNER = core::ptr::null_mut();
    let mut PrimaryGroupOfToken: PTOKEN_PRIMARY_GROUP = core::ptr::null_mut();
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut DaclSize: ULONG;
    let mut TokenOwnerSize: ULONG = 0;
    let mut PrimaryGroupSize: ULONG = 0;
    let mut RelativeSDSize: ULONG = 0;
    let mut OwnerSid: PSID = core::ptr::null_mut();
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut PrimaryGroupSid: PSID;
    let mut RelativeSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };

    'quit: loop {
        Status = NtQueryInformationToken(
            TokenHandle,
            TokenOwner,
            core::ptr::null_mut(),
            0,
            &mut TokenOwnerSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            return 0;
        }

        OwnerOfToken = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            TokenOwnerSize as SIZE_T,
        ) as PTOKEN_OWNER;
        if OwnerOfToken.is_null() {
            return 0;
        }

        Status = NtQueryInformationToken(
            TokenHandle,
            TokenOwner,
            OwnerOfToken as PVOID,
            TokenOwnerSize,
            &mut TokenOwnerSize,
        );
        if !NT_SUCCESS(Status) {
            Success = 0;
            break 'quit;
        }

        Status = NtQueryInformationToken(
            TokenHandle,
            TokenPrimaryGroup,
            core::ptr::null_mut(),
            0,
            &mut PrimaryGroupSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            Success = 0;
            break 'quit;
        }

        PrimaryGroupOfToken = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            PrimaryGroupSize as SIZE_T,
        ) as PTOKEN_PRIMARY_GROUP;
        if PrimaryGroupOfToken.is_null() {
            Success = 0;
            break 'quit;
        }

        Status = NtQueryInformationToken(
            TokenHandle,
            TokenPrimaryGroup,
            PrimaryGroupOfToken as PVOID,
            PrimaryGroupSize,
            &mut PrimaryGroupSize,
        );
        if !NT_SUCCESS(Status) {
            Success = 0;
            break 'quit;
        }

        if AllocateAndInitializeSid(
            &mut NtAuthority,
            1,
            SECURITY_LOCAL_SYSTEM_RID,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut SystemSid,
        ) == 0
        {
            Success = 0;
            break 'quit;
        }

        OwnerSid = (*OwnerOfToken).Owner;
        PrimaryGroupSid = (*PrimaryGroupOfToken).PrimaryGroup;

        DaclSize = core::mem::size_of::<ACL>() as u32
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32
            + GetLengthSid(OwnerSid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32
            + GetLengthSid(SystemSid);

        Dacl = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            DaclSize as SIZE_T,
        ) as PACL;
        if Dacl.is_null() {
            Success = 0;
            break 'quit;
        }

        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            Success = 0;
            break 'quit;
        }

        if AddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, OwnerSid) == 0 {
            Success = 0;
            break 'quit;
        }

        if AddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, SystemSid) == 0 {
            Success = 0;
            break 'quit;
        }

        if InitializeSecurityDescriptor(
            &mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR_REVISION,
        ) == 0
        {
            Success = 0;
            break 'quit;
        }

        if SetSecurityDescriptorDacl(
            &mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR,
            1,
            Dacl,
            0,
        ) == 0
        {
            Success = 0;
            break 'quit;
        }

        if SetSecurityDescriptorOwner(
            &mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR,
            OwnerSid,
            0,
        ) == 0
        {
            Success = 0;
            break 'quit;
        }

        if SetSecurityDescriptorGroup(
            &mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR,
            PrimaryGroupSid,
            0,
        ) == 0
        {
            Success = 0;
            break 'quit;
        }

        if MakeSelfRelativeSD(
            &mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            &mut RelativeSDSize,
        ) != 0
            || GetLastError() != ERROR_INSUFFICIENT_BUFFER
        {
            Success = 0;
            break 'quit;
        }

        RelativeSD = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            RelativeSDSize as SIZE_T,
        ) as PSECURITY_DESCRIPTOR;
        if RelativeSD.is_null() {
            Success = 0;
            break 'quit;
        }

        if MakeSelfRelativeSD(
            &mut AbsoluteSd as *mut _ as PSECURITY_DESCRIPTOR,
            RelativeSD,
            &mut RelativeSDSize,
        ) == 0
        {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSD);
            Success = 0;
            break 'quit;
        }

        *Sd = RelativeSD;
        Success = 1;
        break 'quit;
    }

    if !OwnerOfToken.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, OwnerOfToken as PVOID);
    }

    if !PrimaryGroupOfToken.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, PrimaryGroupOfToken as PVOID);
    }

    if !SystemSid.is_null() {
        FreeSid(SystemSid);
    }

    if !Dacl.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl as PVOID);
    }

    Success
}

unsafe fn InsertProcessSecurityCommon(
    ProcessHandle: HANDLE,
    ThreadHandle: HANDLE,
    ProcessSecurity: PSECURITY_DESCRIPTOR,
    ThreadSecurity: PSECURITY_DESCRIPTOR,
) -> BOOL {
    if SetKernelObjectSecurity(
        ProcessHandle,
        DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
        ProcessSecurity,
    ) == 0
    {
        return 0;
    }

    if SetKernelObjectSecurity(
        ThreadHandle,
        DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
        ThreadSecurity,
    ) == 0
    {
        return 0;
    }

    1
}

unsafe fn InsertTokenToProcessCommon(
    ImpersonateAsSelf: BOOL,
    ProcessHandle: HANDLE,
    ThreadHandle: HANDLE,
    DuplicatedTokenHandle: HANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut AccessToken: PROCESS_ACCESS_TOKEN = core::mem::zeroed();
    let mut PrivilegeSet: BOOLEAN;
    let HavePrivilege: BOOLEAN;

    PrivilegeSet = 0u8;

    if ImpersonateAsSelf != 0 {
        Status = RtlImpersonateSelf(SecurityImpersonation);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    Status = RtlAdjustPrivilege(
        SE_ASSIGNPRIMARYTOKEN_PRIVILEGE as _,
        1u8,
        1u8,
        &mut PrivilegeSet,
    );
    HavePrivilege = if NT_SUCCESS(Status) { 1u8 } else { 0u8 };

    AccessToken.Token = DuplicatedTokenHandle;
    AccessToken.Thread = ThreadHandle;

    Status = NtSetInformationProcess(
        ProcessHandle,
        ProcessAccessToken,
        &mut AccessToken as *mut _ as LPVOID,
        core::mem::size_of::<PROCESS_ACCESS_TOKEN>() as u32,
    );

    if HavePrivilege != 0 {
        RtlAdjustPrivilege(
            SE_ASSIGNPRIMARYTOKEN_PRIVILEGE as _,
            PrivilegeSet,
            1u8,
            &mut PrivilegeSet,
        );
    }

    if ImpersonateAsSelf != 0 {
        RevertToSelf();
    }

    if !NT_SUCCESS(Status) {
        return Status;
    }

    STATUS_SUCCESS
}

unsafe fn CreateProcessAsUserCommon(
    hToken: HANDLE,
    dwCreationFlags: DWORD,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let StatusOnExit: NTSTATUS;
    let mut Success: BOOL;
    let mut Type: TOKEN_TYPE = 0;
    let mut ReturnLength: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DefaultSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let ProcessSd: PSECURITY_DESCRIPTOR;
    let ThreadSd: PSECURITY_DESCRIPTOR;
    let mut hTokenDup: HANDLE = core::ptr::null_mut();
    let mut OriginalImpersonationToken: HANDLE = core::ptr::null_mut();
    let mut NullToken: HANDLE = core::ptr::null_mut();

    'quit: loop {
        if hToken != core::ptr::null_mut() {
            Status = NtQueryInformationToken(
                hToken,
                TokenType,
                core::ptr::addr_of_mut!(Type) as PVOID,
                core::mem::size_of::<TOKEN_TYPE>() as ULONG,
                &mut ReturnLength,
            );
            if !NT_SUCCESS(Status) {
                Success = 0;
                break 'quit;
            }

            if Type != TokenPrimary {
                Status = STATUS_BAD_TOKEN_TYPE;
                Success = 0;
                break 'quit;
            }

            Status = NtOpenThreadToken(
                -2isize as HANDLE,
                TOKEN_QUERY | TOKEN_IMPERSONATE,
                1,
                &mut OriginalImpersonationToken,
            );
            if !NT_SUCCESS(Status) {
                OriginalImpersonationToken = core::ptr::null_mut();
                if Status != STATUS_NO_TOKEN {
                    Success = 0;
                    break 'quit;
                }
            } else {
                Status = NtSetInformationThread(
                    -2isize as HANDLE,
                    ThreadImpersonationToken,
                    core::ptr::addr_of_mut!(NullToken) as PVOID,
                    core::mem::size_of::<HANDLE>() as ULONG,
                );
                if !NT_SUCCESS(Status) {
                    Success = 0;
                    break 'quit;
                }
            }

            if CreateDefaultProcessSecurityCommon(hToken, &mut DefaultSd) == 0 {
                Success = 0;
                break 'quit;
            }

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.Attributes = 0;
            ObjectAttributes.ObjectName = core::ptr::null_mut();
            ObjectAttributes.SecurityDescriptor = DefaultSd;
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtDuplicateToken(
                hToken,
                0,
                &mut ObjectAttributes,
                0,
                TokenPrimary,
                &mut hTokenDup,
            );
            if !NT_SUCCESS(Status) {
                Success = 0;
                break 'quit;
            }

            Status = InsertTokenToProcessCommon(
                1,
                (*lpProcessInformation).hProcess,
                (*lpProcessInformation).hThread,
                hTokenDup,
            );
            if !NT_SUCCESS(Status) {
                Status = NtSetInformationThread(
                    -2isize as HANDLE,
                    ThreadImpersonationToken,
                    core::ptr::addr_of_mut!(OriginalImpersonationToken) as PVOID,
                    core::mem::size_of::<HANDLE>() as ULONG,
                );
                if !NT_SUCCESS(Status) {
                    NtClose(hTokenDup);
                    Success = 0;
                    break 'quit;
                }

                Status = InsertTokenToProcessCommon(
                    0,
                    (*lpProcessInformation).hProcess,
                    (*lpProcessInformation).hThread,
                    hTokenDup,
                );
                if !NT_SUCCESS(Status) {
                    NtClose(hTokenDup);
                    Success = 0;
                    break 'quit;
                }

                Status = NtSetInformationThread(
                    -2isize as HANDLE,
                    ThreadImpersonationToken,
                    core::ptr::addr_of_mut!(NullToken) as PVOID,
                    core::mem::size_of::<HANDLE>() as ULONG,
                );
                if !NT_SUCCESS(Status) {
                    NtClose(hTokenDup);
                    Success = 0;
                    break 'quit;
                }
            }

            if !lpProcessAttributes.is_null() && !(*lpProcessAttributes).lpSecurityDescriptor.is_null() {
                ProcessSd = (*lpProcessAttributes).lpSecurityDescriptor;
            } else {
                ProcessSd = DefaultSd;
            }

            if !lpThreadAttributes.is_null() && !(*lpThreadAttributes).lpSecurityDescriptor.is_null() {
                ThreadSd = (*lpThreadAttributes).lpSecurityDescriptor;
            } else {
                ThreadSd = DefaultSd;
            }

            if InsertProcessSecurityCommon(
                (*lpProcessInformation).hProcess,
                (*lpProcessInformation).hThread,
                ProcessSd,
                ThreadSd,
            ) == 0
            {
                NtClose(hTokenDup);
                Success = 0;
                break 'quit;
            }

            NtClose(hTokenDup);
            Success = 1;
        }

        Success = 1;
        break 'quit;
    }

    if OriginalImpersonationToken != core::ptr::null_mut() {
        StatusOnExit = NtSetInformationThread(
            -2isize as HANDLE,
            ThreadImpersonationToken,
            core::ptr::addr_of_mut!(OriginalImpersonationToken) as PVOID,
            core::mem::size_of::<HANDLE>() as ULONG,
        );
        let _ = StatusOnExit;

        NtClose(OriginalImpersonationToken);
    }

    if !NT_SUCCESS(Status) {
        TerminateProcess((*lpProcessInformation).hProcess, Status as UINT);
        SetLastError(RtlNtStatusToDosError(Status));
    }

    if (dwCreationFlags & CREATE_SUSPENDED) == 0 {
        ResumeThread((*lpProcessInformation).hThread);
    }

    if DefaultSd != core::ptr::null_mut() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DefaultSd);
    }

    Success
}

#[no_mangle]
pub unsafe extern "system" fn CreateProcessAsUserA(
    hToken: HANDLE,
    lpApplicationName: LPCSTR,
    lpCommandLine: LPSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCSTR,
    lpStartupInfo: LPSTARTUPINFOA,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    if CreateProcessA(
        lpApplicationName,
        lpCommandLine,
        lpProcessAttributes,
        lpThreadAttributes,
        bInheritHandles,
        dwCreationFlags | CREATE_SUSPENDED,
        lpEnvironment,
        lpCurrentDirectory,
        lpStartupInfo,
        lpProcessInformation,
    ) == 0
    {
        return 0;
    }

    CreateProcessAsUserCommon(
        hToken,
        dwCreationFlags,
        lpProcessAttributes,
        lpThreadAttributes,
        lpProcessInformation,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CreateProcessAsUserW(
    hToken: HANDLE,
    lpApplicationName: LPCWSTR,
    lpCommandLine: LPWSTR,
    lpProcessAttributes: LPSECURITY_ATTRIBUTES,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    bInheritHandles: BOOL,
    dwCreationFlags: DWORD,
    lpEnvironment: LPVOID,
    lpCurrentDirectory: LPCWSTR,
    lpStartupInfo: LPSTARTUPINFOW,
    lpProcessInformation: LPPROCESS_INFORMATION,
) -> BOOL {
    if CreateProcessW(
        lpApplicationName,
        lpCommandLine,
        lpProcessAttributes,
        lpThreadAttributes,
        bInheritHandles,
        dwCreationFlags | CREATE_SUSPENDED,
        lpEnvironment,
        lpCurrentDirectory,
        lpStartupInfo,
        lpProcessInformation,
    ) == 0
    {
        return 0;
    }

    CreateProcessAsUserCommon(
        hToken,
        dwCreationFlags,
        lpProcessAttributes,
        lpThreadAttributes,
        lpProcessInformation,
    )
}

#[no_mangle]
pub unsafe extern "system" fn LogonUserA(
    lpszUsername: LPSTR,
    lpszDomain: LPSTR,
    lpszPassword: LPSTR,
    dwLogonType: DWORD,
    dwLogonProvider: DWORD,
    phToken: PHANDLE,
) -> BOOL {
    LogonUserExA(
        lpszUsername,
        lpszDomain,
        lpszPassword,
        dwLogonType,
        dwLogonProvider,
        phToken,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn LogonUserExA(
    lpszUsername: LPSTR,
    lpszDomain: LPSTR,
    lpszPassword: LPSTR,
    dwLogonType: DWORD,
    dwLogonProvider: DWORD,
    phToken: PHANDLE,
    ppLogonSid: *mut PSID,
    ppProfileBuffer: *mut PVOID,
    pdwProfileLength: LPDWORD,
    pQuotaLimits: PQUOTA_LIMITS,
) -> BOOL {
    let mut UserName: _UNICODE_STRING = core::mem::zeroed();
    let mut Domain: _UNICODE_STRING = core::mem::zeroed();
    let mut Password: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL = 0;

    UserName.Buffer = core::ptr::null_mut();
    Domain.Buffer = core::ptr::null_mut();
    Password.Buffer = core::ptr::null_mut();

    'username_done: {
        if RtlCreateUnicodeStringFromAsciiz(&mut UserName, lpszUsername as *const _) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'username_done;
        }

        'domain_done: {
            if RtlCreateUnicodeStringFromAsciiz(&mut Domain, lpszDomain as *const _) == 0 {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'domain_done;
            }

            'password_done: {
                if RtlCreateUnicodeStringFromAsciiz(&mut Password, lpszPassword as *const _) == 0 {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'password_done;
                }

                ret = LogonUserExW(
                    UserName.Buffer,
                    Domain.Buffer,
                    Password.Buffer,
                    dwLogonType,
                    dwLogonProvider,
                    phToken,
                    ppLogonSid,
                    ppProfileBuffer,
                    pdwProfileLength,
                    pQuotaLimits,
                );

                if !Password.Buffer.is_null() {
                    RtlFreeUnicodeString(&mut Password);
                }
            }

            if !Domain.Buffer.is_null() {
                RtlFreeUnicodeString(&mut Domain);
            }
        }

        if !UserName.Buffer.is_null() {
            RtlFreeUnicodeString(&mut UserName);
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn LogonUserW(
    lpszUsername: LPWSTR,
    lpszDomain: LPWSTR,
    lpszPassword: LPWSTR,
    dwLogonType: DWORD,
    dwLogonProvider: DWORD,
    phToken: PHANDLE,
) -> BOOL {
    LogonUserExW(
        lpszUsername,
        lpszDomain,
        lpszPassword,
        dwLogonType,
        dwLogonProvider,
        phToken,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn LogonUserExW(
    lpszUsername: LPWSTR,
    lpszDomain: LPWSTR,
    lpszPassword: LPWSTR,
    dwLogonType: DWORD,
    dwLogonProvider: DWORD,
    phToken: PHANDLE,
    ppLogonSid: *mut PSID,
    ppProfileBuffer: *mut PVOID,
    pdwProfileLength: LPDWORD,
    pQuotaLimits: PQUOTA_LIMITS,
) -> BOOL {
    let mut LocalAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, SECURITY_LOCAL_SID_AUTHORITY.Value[5]],
    };
    let mut SystemAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, SECURITY_NT_AUTHORITY.Value[5]],
    };
    let mut LogonSid: PSID = core::ptr::null_mut();
    let mut LocalSid: PSID = core::ptr::null_mut();
    let mut OriginName: LSA_STRING = core::mem::zeroed();
    let mut DomainName: _UNICODE_STRING = core::mem::zeroed();
    let mut UserName: _UNICODE_STRING = core::mem::zeroed();
    let mut Password: _UNICODE_STRING = core::mem::zeroed();
    let mut AuthInfo: *mut MSV1_0_INTERACTIVE_LOGON = core::ptr::null_mut();
    let mut AuthInfoLength: ULONG;
    let mut Ptr: ULONG_PTR;
    let mut TokenSrc: TOKEN_SOURCE = core::mem::zeroed();
    let mut TokenGrps: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut ProfileBuffer: *mut MSV1_0_INTERACTIVE_PROFILE = core::ptr::null_mut();
    let mut ProfileBufferLength: ULONG = 0;
    let mut Luid: LUID = LUID { LowPart: 0, HighPart: 0 };
    let mut LogonId: LUID = LUID { LowPart: 0, HighPart: 0 };
    let mut TokenHandle: HANDLE = core::ptr::null_mut();
    let mut QuotaLimits: QUOTA_LIMITS = core::mem::zeroed();
    let LogonType: SECURITY_LOGON_TYPE;
    let mut SubStatus: NTSTATUS = STATUS_SUCCESS;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (ppProfileBuffer != core::ptr::null_mut() && pdwProfileLength == core::ptr::null_mut())
        || (ppProfileBuffer == core::ptr::null_mut() && pdwProfileLength != core::ptr::null_mut())
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if ppProfileBuffer != core::ptr::null_mut() && pdwProfileLength != core::ptr::null_mut() {
        *ppProfileBuffer = core::ptr::null_mut();
        *pdwProfileLength = 0;
    }

    if phToken != core::ptr::null_mut() {
        *phToken = core::ptr::null_mut();
    }

    'done: loop {
        match dwLogonType {
            LOGON32_LOGON_INTERACTIVE => {
                LogonType = Interactive;
            }
            LOGON32_LOGON_NETWORK => {
                LogonType = Network;
            }
            LOGON32_LOGON_BATCH => {
                LogonType = Batch;
            }
            LOGON32_LOGON_SERVICE => {
                LogonType = Service;
            }
            _ => {
                Status = STATUS_INVALID_PARAMETER;
                break 'done;
            }
        }

        if LsaHandle == core::ptr::null_mut() {
            Status = OpenLogonLsaHandle();
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(OriginName) as PANSI_STRING,
            b"Advapi32 Logon\0".as_ptr() as LPCSTR,
        );

        RtlInitUnicodeString(&mut DomainName, lpszDomain);
        RtlInitUnicodeString(&mut UserName, lpszUsername);
        RtlInitUnicodeString(&mut Password, lpszPassword);

        AuthInfoLength = core::mem::size_of::<MSV1_0_INTERACTIVE_LOGON>() as ULONG
            + DomainName.MaximumLength as ULONG
            + UserName.MaximumLength as ULONG
            + Password.MaximumLength as ULONG;

        AuthInfo = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, AuthInfoLength as SIZE_T)
            as *mut MSV1_0_INTERACTIVE_LOGON;
        if AuthInfo == core::ptr::null_mut() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*AuthInfo).MessageType = MsV1_0InteractiveLogon;

        Ptr = AuthInfo as ULONG_PTR + core::mem::size_of::<MSV1_0_INTERACTIVE_LOGON>() as ULONG_PTR;

        (*AuthInfo).LogonDomainName.Length = DomainName.Length;
        (*AuthInfo).LogonDomainName.MaximumLength = DomainName.MaximumLength;
        (*AuthInfo).LogonDomainName.Buffer = if DomainName.Buffer == core::ptr::null_mut() {
            core::ptr::null_mut()
        } else {
            Ptr as PWCHAR
        };
        if DomainName.MaximumLength > 0 {
            RtlCopyMemory(
                (*AuthInfo).LogonDomainName.Buffer as PVOID,
                DomainName.Buffer as PVOID,
                DomainName.MaximumLength as usize,
            );
            Ptr += DomainName.MaximumLength as ULONG_PTR;
        }

        (*AuthInfo).UserName.Length = UserName.Length;
        (*AuthInfo).UserName.MaximumLength = UserName.MaximumLength;
        (*AuthInfo).UserName.Buffer = Ptr as PWCHAR;
        if UserName.MaximumLength > 0 {
            RtlCopyMemory(
                (*AuthInfo).UserName.Buffer as PVOID,
                UserName.Buffer as PVOID,
                UserName.MaximumLength as usize,
            );
        }

        Ptr += UserName.MaximumLength as ULONG_PTR;

        (*AuthInfo).Password.Length = Password.Length;
        (*AuthInfo).Password.MaximumLength = Password.MaximumLength;
        (*AuthInfo).Password.Buffer = Ptr as PWCHAR;
        if Password.MaximumLength > 0 {
            RtlCopyMemory(
                (*AuthInfo).Password.Buffer as PVOID,
                Password.Buffer as PVOID,
                Password.MaximumLength as usize,
            );
        }

        AllocateLocallyUniqueId(&mut LogonId);
        Status = RtlAllocateAndInitializeSid(
            &mut SystemAuthority,
            SECURITY_LOGON_IDS_RID_COUNT as u8,
            SECURITY_LOGON_IDS_RID,
            LogonId.HighPart as u32,
            LogonId.LowPart,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            &mut LogonSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = RtlAllocateAndInitializeSid(
            &mut LocalAuthority,
            1,
            SECURITY_LOCAL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            SECURITY_NULL_RID,
            &mut LocalSid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        TokenGrps = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (core::mem::size_of::<TOKEN_GROUPS>()
                + ((2 - ANYSIZE_ARRAY as usize) * core::mem::size_of::<SID_AND_ATTRIBUTES>())) as SIZE_T,
        ) as PTOKEN_GROUPS;
        if TokenGrps == core::ptr::null_mut() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*TokenGrps).GroupCount = 2;
        (*TokenGrps).Groups.as_mut_ptr().add(0).write(SID_AND_ATTRIBUTES {
            Sid: LogonSid,
            Attributes: SE_GROUP_MANDATORY
                | SE_GROUP_ENABLED
                | SE_GROUP_ENABLED_BY_DEFAULT
                | SE_GROUP_LOGON_ID,
        });
        (*TokenGrps).Groups.as_mut_ptr().add(1).write(SID_AND_ATTRIBUTES {
            Sid: LocalSid,
            Attributes: SE_GROUP_MANDATORY | SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT,
        });

        RtlCopyMemory(
            TokenSrc.SourceName.as_mut_ptr() as PVOID,
            AdvapiTokenSourceName.as_ptr() as PVOID,
            core::mem::size_of_val(&TokenSrc.SourceName),
        );
        AllocateLocallyUniqueId(&mut TokenSrc.SourceIdentifier);

        Status = LsaLogonUser(
            LsaHandle,
            &mut OriginName,
            LogonType,
            AuthenticationPackage,
            AuthInfo as PVOID,
            AuthInfoLength,
            TokenGrps,
            &mut TokenSrc,
            core::ptr::addr_of_mut!(ProfileBuffer) as *mut PVOID,
            &mut ProfileBufferLength,
            &mut Luid,
            &mut TokenHandle,
            &mut QuotaLimits,
            &mut SubStatus,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if phToken != core::ptr::null_mut() {
            *phToken = TokenHandle;
        }

        break 'done;
    }

    if ProfileBuffer != core::ptr::null_mut() {
        LsaFreeReturnBuffer(ProfileBuffer as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if TokenHandle != core::ptr::null_mut() {
            CloseHandle(TokenHandle);
        }
    }

    if TokenGrps != core::ptr::null_mut() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, TokenGrps as PVOID);
    }

    if LocalSid != core::ptr::null_mut() {
        RtlFreeSid(LocalSid);
    }

    if LogonSid != core::ptr::null_mut() {
        RtlFreeSid(LogonSid);
    }

    if AuthInfo != core::ptr::null_mut() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AuthInfo as PVOID);
    }

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS system libraries
//  * FILE:        lib/advapi32/misc/logon.c
//  * PURPOSE:     Logon functions
//  * PROGRAMMER:  Eric Kohl
//  */
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /* GLOBALS *****************************************************************/
//
// static const CHAR AdvapiTokenSourceName[] = "Advapi  ";
// C_ASSERT(sizeof(AdvapiTokenSourceName) == RTL_FIELD_SIZE(TOKEN_SOURCE, SourceName) + 1);
//
// HANDLE LsaHandle = NULL;
// ULONG AuthenticationPackage = 0;
//
// /* FUNCTIONS ***************************************************************/
//
// static
// NTSTATUS
// OpenLogonLsaHandle(VOID)
// {
//     LSA_STRING LogonProcessName;
//     LSA_STRING PackageName;
//     LSA_OPERATIONAL_MODE SecurityMode = 0;
//     NTSTATUS Status;
//
//     RtlInitAnsiString((PANSI_STRING)&LogonProcessName,
//                       "User32LogonProcess");
//
//     Status = LsaRegisterLogonProcess(&LogonProcessName,
//                                      &LsaHandle,
//                                      &SecurityMode);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("LsaRegisterLogonProcess failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     RtlInitAnsiString((PANSI_STRING)&PackageName,
//                       MSV1_0_PACKAGE_NAME);
//
//     Status = LsaLookupAuthenticationPackage(LsaHandle,
//                                             &PackageName,
//                                             &AuthenticationPackage);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("LsaLookupAuthenticationPackage failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     TRACE("AuthenticationPackage: 0x%08lx\n", AuthenticationPackage);
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (LsaHandle != NULL)
//         {
//             Status = LsaDeregisterLogonProcess(LsaHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 TRACE("LsaDeregisterLogonProcess failed (Status 0x%08lx)\n", Status);
//             }
//         }
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// CloseLogonLsaHandle(VOID)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (LsaHandle != NULL)
//     {
//         Status = LsaDeregisterLogonProcess(LsaHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("LsaDeregisterLogonProcess failed (Status 0x%08lx)\n", Status);
//         }
//     }
//
//     return Status;
// }
//
//
// /**
//  * @brief
//  * Creates a default security descriptor that is going
//  * to be used by both the newly created process and thread
//  * by a call to CreateProcessAsUserA/W. This descriptor also
//  * serves for the newly duplicated token object that is going
//  * to be set for the token which acts as the main user.
//  *
//  * @param[in] TokenHandle
//  * A handle to a token. The function will use this token to
//  * query security details such as the owner and primary group
//  * associated with the security context of this token. The
//  * obtained information will then be assigned to the security
//  * descriptor.
//  *
//  * @param[out] Sd
//  * A pointer to an allocated security descriptor that is given
//  * to the caller.
//  *
//  * @return
//  * Return TRUE if the security descriptor has been successfully
//  * created, FALSE otherwise.
//  *
//  * @remarks
//  * When a process is created on behald of the user's security context
//  * this user will be the owner and responsible for that process. Whatever
//  * objects created or stuff done within the process space is at the
//  * discretion of the user, that is, further objects created are in
//  * charge by the user himself as is the owner of the process.
//  *
//  * !!!NOTE!!! -- On Windows the security descriptor is created by using
//  * CreatePrivateObjectSecurity(Ex) API call. Whilst the way the security
//  * descriptor is created in our end is not wrong per se, this function
//  * serves a placeholder until CreatePrivateObjectSecurity is implemented.
//  */
// static
// BOOL
// CreateDefaultProcessSecurityCommon(
//      HANDLE TokenHandle,
//      PSECURITY_DESCRIPTOR *Sd)
// {
//     NTSTATUS Status;
//     BOOL Success;
//     PACL Dacl;
//     PTOKEN_OWNER OwnerOfToken;
//     PTOKEN_PRIMARY_GROUP PrimaryGroupOfToken;
//     SECURITY_DESCRIPTOR AbsoluteSd;
//     ULONG DaclSize, TokenOwnerSize, PrimaryGroupSize, RelativeSDSize = 0;
//     PSID OwnerSid = NULL, SystemSid = NULL, PrimaryGroupSid = NULL;
//     PSECURITY_DESCRIPTOR RelativeSD = NULL;
//     static SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//
//     /*
//      * Since we do not know how much space
//      * is needed to allocate the buffer to
//      * hold the token owner, first we must
//      * query the exact size.
//      */
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenOwner,
//                                      NULL,
//                                      0,
//                                      &TokenOwnerSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Unexpected status code returned, must be STATUS_BUFFER_TOO_SMALL (Status 0x%08lx)\n", Status);
//         return FALSE;
//     }
//
//     /* We have the required space size, allocate the buffer now */
//     OwnerOfToken = RtlAllocateHeap(RtlGetProcessHeap(),
//                                    HEAP_ZERO_MEMORY,
//                                    TokenOwnerSize);
//     if (OwnerOfToken == NULL)
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to allocate buffer for token owner\n");
//         return FALSE;
//     }
//
//     /* Now query the token owner */
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenOwner,
//                                      OwnerOfToken,
//                                      TokenOwnerSize,
//                                      &TokenOwnerSize);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to query the token owner (Status 0x%08lx)\n", Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Do the same process but for the primary group now */
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenPrimaryGroup,
//                                      NULL,
//                                      0,
//                                      &PrimaryGroupSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Unexpected status code returned, must be STATUS_BUFFER_TOO_SMALL (Status 0x%08lx)\n", Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Allocate the buffer */
//     PrimaryGroupOfToken = RtlAllocateHeap(RtlGetProcessHeap(),
//                                           HEAP_ZERO_MEMORY,
//                                           PrimaryGroupSize);
//     if (PrimaryGroupOfToken == NULL)
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to allocate buffer for primary group token\n");
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Query the primary group now */
//     Status = NtQueryInformationToken(TokenHandle,
//                                      TokenPrimaryGroup,
//                                      PrimaryGroupOfToken,
//                                      PrimaryGroupSize,
//                                      &PrimaryGroupSize);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to query the token owner (Status 0x%08lx)\n", Status);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Create the SYSTEM SID */
//     if (!AllocateAndInitializeSid(&NtAuthority,
//                                   1,
//                                   SECURITY_LOCAL_SYSTEM_RID,
//                                   0, 0, 0, 0, 0, 0, 0,
//                                   &SystemSid))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to create Local System SID (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Cache the token owner and primary group SID */
//     OwnerSid = OwnerOfToken->Owner;
//     PrimaryGroupSid = PrimaryGroupOfToken->PrimaryGroup;
//
//     /* Set up the DACL size */
//     DaclSize = sizeof(ACL) +
//                sizeof(ACCESS_ALLOWED_ACE) + GetLengthSid(OwnerSid) +
//                sizeof(ACCESS_ALLOWED_ACE) + GetLengthSid(SystemSid);
//
//     /* Allocate buffer for the DACL */
//     Dacl = RtlAllocateHeap(RtlGetProcessHeap(),
//                            HEAP_ZERO_MEMORY,
//                            DaclSize);
//     if (Dacl == NULL)
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to allocate buffer for DACL\n");
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Initialize the DACL */
//     if (!InitializeAcl(Dacl, DaclSize, ACL_REVISION))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to initialize DACL (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Give full powers to the owner */
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              GENERIC_ALL,
//                              OwnerSid))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to set up ACE for owner (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Give full powers to SYSTEM as well */
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              GENERIC_ALL,
//                              SystemSid))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to set up ACE for SYSTEM (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Initialize the descriptor in absolute format */
//     if (!InitializeSecurityDescriptor(&AbsoluteSd, SECURITY_DESCRIPTOR_REVISION))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to initialize absolute security descriptor (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Set the DACL to the security descriptor */
//     if (!SetSecurityDescriptorDacl(&AbsoluteSd, TRUE, Dacl, FALSE))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to set up DACL to absolute security descriptor (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Set the owner for this descriptor */
//     if (!SetSecurityDescriptorOwner(&AbsoluteSd, OwnerSid, FALSE))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to set up owner to absolute security descriptor (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Set the primary group for this descriptor */
//     if (!SetSecurityDescriptorGroup(&AbsoluteSd, PrimaryGroupSid, FALSE))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to set up group to absolute security descriptor (error %lu)\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /*
//      * Determine the exact size space of the absolute
//      * descriptor so that we can allocate a buffer
//      * to hold the descriptor in a converted self
//      * relative format.
//      */
//     if (MakeSelfRelativeSD(&AbsoluteSd, NULL, &RelativeSDSize) ||
//         (GetLastError() != ERROR_INSUFFICIENT_BUFFER))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): error %lu, expected ERROR_INSUFFICIENT_BUFFER\n", GetLastError());
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Allocate the buffer */
//     RelativeSD = RtlAllocateHeap(RtlGetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  RelativeSDSize);
//     if (RelativeSD == NULL)
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to allocate relative SD\n");
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Convert to a self relative format now */
//     if (!MakeSelfRelativeSD(&AbsoluteSd, RelativeSD, &RelativeSDSize))
//     {
//         ERR("CreateDefaultProcessSecurityCommon(): Failed to allocate relative SD (error %lu)\n", GetLastError());
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSD);
//         Success = FALSE;
//         goto Quit;
//     }
//
//     /* Success, give the descriptor to the caller */
//     *Sd = RelativeSD;
//     Success = TRUE;
//
// Quit:
//     /* Free all the stuff we have allocated */
//     if (OwnerOfToken != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, OwnerOfToken);
//
//     if (PrimaryGroupOfToken != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PrimaryGroupOfToken);
//
//     if (SystemSid != NULL)
//         FreeSid(SystemSid);
//
//     if (Dacl != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Dacl);
//
//     return Success;
// }
//
//
// /**
//  * @brief
//  * Changes the object security information of a process
//  * and thread that belongs to the process with new security
//  * data, basically by replacing the previous security descriptor
//  * with a new one.
//  *
//  * @param[in] ProcessHandle
//  * A handle to a valid process of which security information is
//  * to be changed by setting up a new security descriptor.
//  *
//  * @param[in] ThreadHandle
//  * A handle to a valid thread of which security information is
//  * to be changed by setting up a new security descriptor.
//  *
//  * @param[in] ProcessSecurity
//  * A pointer to a security descriptor that is for the process.
//  *
//  * @param[in] ThreadSecurity
//  * A pointer to a security descriptor that is for the thread.
//  *
//  * @return
//  * Return TRUE if new security information has been set, FALSE
//  * otherwise.
//  */
// static
// BOOL
// InsertProcessSecurityCommon(
//      HANDLE ProcessHandle,
//      HANDLE ThreadHandle,
//      PSECURITY_DESCRIPTOR ProcessSecurity,
//      PSECURITY_DESCRIPTOR ThreadSecurity)
// {
//     /* Set new security data for the process */
//     if (!SetKernelObjectSecurity(ProcessHandle,
//                                  DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
//                                  ProcessSecurity))
//     {
//         ERR("InsertProcessSecurityCommon(): Failed to set security for process (error %lu)\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Set new security data for the thread */
//     if (!SetKernelObjectSecurity(ThreadHandle,
//                                  DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
//                                  ThreadSecurity))
//     {
//         ERR("InsertProcessSecurityCommon(): Failed to set security for thread (error %lu)\n", GetLastError());
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /**
//  * @brief
//  * Sets a primary token to the newly created process.
//  * The primary token that gets assigned to is a token
//  * whose security context is associated with the logged
//  * in user. For futher documentation information, see
//  * Remarks.
//  *
//  * @param[in] ImpersonateAsSelf
//  * If set to TRUE, the function will act on behalf of
//  * the calling process by impersonating its security context.
//  * Generally the caller will disable impersonation and attempt
//  * to act on behalf of the said main process as a first tentative
//  * to acquire the needed privilege in order to assign a token
//  * to the process. If set to FALSE, the function won't act on behalf
//  * of the calling process.
//  *
//  * @param[in] ProcessHandle
//  * A handle to the newly created process. The function will use it
//  * as a mean to assign the primary token to this process.
//  *
//  * @param[in] ThreadHandle
//  * A handle to the newly and primary created thread associated with
//  * the process.
//  *
//  * @param[in] DuplicatedTokenHandle
//  * A handle to a duplicated access token. This token represents as a primary
//  * one, initially duplicated in form as a primary type from an impersonation
//  * type.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if token assignment to process succeeded, otherwise
//  * a failure NTSTATUS code is returned. A potential failure status code is
//  * STATUS_ACCESS_DENIED which means the caller doesn't have enough rights
//  * to grant access for primary token assignment to process.
//  *
//  * @remarks
//  * This function acts like an internal helper for CreateProcessAsUserCommon (and as
//  * such for CreateProcessAsUserW/A as well) as once a process is created, the
//  * function is tasked to assign the security context of the logged in user to
//  * that process. However, the rate of success of inserting the token into the
//  * process ultimately depends on the caller.
//  *
//  * The caller will either succeed or fail at acquiring SE_ASSIGNPRIMARYTOKEN_PRIVILEGE
//  * privilege depending on the security context of the user. If it's allowed, the caller
//  * would generally acquire such privilege immediately but if not, the caller will attempt
//  * to do a second try.
//  */
// static
// NTSTATUS
// InsertTokenToProcessCommon(
//      BOOL ImpersonateAsSelf,
//      HANDLE ProcessHandle,
//      HANDLE ThreadHandle,
//      HANDLE DuplicatedTokenHandle)
// {
//     NTSTATUS Status;
//     PROCESS_ACCESS_TOKEN AccessToken;
//     BOOLEAN PrivilegeSet;
//     BOOLEAN HavePrivilege;
//
//     /*
//      * Assume the SE_ASSIGNPRIMARYTOKEN_PRIVILEGE
//      * privilege hasn't been set.
//      */
//     PrivilegeSet = FALSE;
//
//     /*
//      * The caller asked that we must impersonate as
//      * ourselves, that is, we'll be going to impersonate
//      * the security context of the calling process. If
//      * self impersonation fails then the caller has
//      * to do a "rinse and repeat" approach.
//      */
//     if (ImpersonateAsSelf)
//     {
//         Status = RtlImpersonateSelf(SecurityImpersonation);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("RtlImpersonateSelf(SecurityImpersonation) failed, Status 0x%08x\n", Status);
//             return Status;
//         }
//     }
//
//     /*
//      * Attempt to acquire the process primary token assignment privilege
//      * in case we actually need it.
//      * The call will either succeed or fail when the caller has (or has not)
//      * enough rights.
//      * The last situation may not be dramatic for us. Indeed it may happen
//      * that the user-provided token is a restricted version of the caller's
//      * primary token (aka. a "child" token), or both tokens inherit (i.e. are
//      * children, and are together "siblings") from a common parent token.
//      * In this case the NT kernel allows us to assign the token to the child
//      * process without the need for the assignment privilege, which is fine.
//      * On the contrary, if the user-provided token is completely arbitrary,
//      * then the NT kernel will enforce the presence of the assignment privilege:
//      * because we failed (by assumption) to assign the privilege, the process
//      * token assignment will fail as required. It is then the job of the
//      * caller to manually acquire the necessary privileges.
//      */
//     Status = RtlAdjustPrivilege(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE,
//                                 TRUE, TRUE, &PrivilegeSet);
//     HavePrivilege = NT_SUCCESS(Status);
//     if (!HavePrivilege)
//     {
//         ERR("RtlAdjustPrivilege(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE) failed, Status 0x%08lx, "
//             "attempting to continue without it...\n", Status);
//     }
//
//     /*
//      * Assign the duplicated token and thread
//      * handle to the structure so that we'll
//      * use it to assign the primary token
//      * to process.
//      */
//     AccessToken.Token = DuplicatedTokenHandle;
//     AccessToken.Thread = ThreadHandle;
//
//     /* Set the new process token */
//     Status = NtSetInformationProcess(ProcessHandle,
//                                      ProcessAccessToken,
//                                      (PVOID)&AccessToken,
//                                      sizeof(AccessToken));
//
//     /* Restore the privilege */
//     if (HavePrivilege)
//     {
//         RtlAdjustPrivilege(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE,
//                            PrivilegeSet, TRUE, &PrivilegeSet);
//     }
//
//     /*
//      * Check again if the caller wanted to impersonate
//      * as self. If that is the case we must revert this
//      * impersonation back.
//      */
//     if (ImpersonateAsSelf)
//     {
//         RevertToSelf();
//     }
//
//     /*
//      * Finally, check if we actually succeeded on assigning
//      * a primary token to the process. If we failed, oh well,
//      * asta la vista baby e arrivederci. The caller has to do
//      * a rinse and repeat approach.
//      */
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to assign primary token to the process (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Internal function that serves as a helper for
//  * CreateProcessAsUserW/A routines on creating
//  * a process within the context of the logged in
//  * user.
//  *
//  * @param[in] hToken
//  * A handle to an access token that is associated
//  * with the logged in user. If the caller does not
//  * submit a token, the helper will immediately quit
//  * and return success, and the newly created process
//  * will be created upon using the default security
//  * context.
//  *
//  * @param[in] dwCreationFlags
//  * Bit masks containing the creation process flags.
//  * The function uses this parameter to determine
//  * if the process wasn't created in a suspended way
//  * and if not the function will resume the main thread.
//  *
//  * @param[in] lpProcessAttributes
//  * A pointer to process attributes. This function uses
//  * this parameter to gather the security descriptor,
//  * if ever present. If it is, this descriptor takes
//  * precedence over the default one when setting
//  * new security information to the process.
//  *
//  * @param[in] lpThreadAttributes
//  * A pointer to thread attributes. This function uses
//  * this parameter to gather the security descriptor,
//  * if ever present. If it is, this descriptor takes
//  * precedence over the default one when setting
//  * new security information to the thread.
//  *
//  * @param[in,out] lpProcessInformation
//  * A pointer to a structure that contains process creation
//  * information data. Such pointer contains the process
//  * and thread handles and whatnot.
//  *
//  * @return
//  * Returns TRUE if the helper has successfully assigned
//  * the newly created process the user's security context
//  * to that process, otherwise FALSE is returned.
//  *
//  * @remarks
//  * In order for the helper function to assign the primary
//  * token to the process, it has to do a "rinse and repeat"
//  * approach. That is, the helper will stop the impersonation
//  * and attempt to assign the token to process by acting
//  * on behalf of the main process' security context. If that
//  * fails, the function will do a second attempt by doing this
//  * but with impersonation enabled instead.
//  */
// static
// BOOL
// CreateProcessAsUserCommon(
//      HANDLE hToken,
//      DWORD dwCreationFlags,
//      LPSECURITY_ATTRIBUTES lpProcessAttributes,
//      LPSECURITY_ATTRIBUTES lpThreadAttributes,
//      LPPROCESS_INFORMATION lpProcessInformation)
// {
//     NTSTATUS Status = STATUS_SUCCESS, StatusOnExit;
//     BOOL Success;
//     TOKEN_TYPE Type;
//     ULONG ReturnLength;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PSECURITY_DESCRIPTOR DefaultSd = NULL, ProcessSd, ThreadSd;
//     HANDLE hTokenDup = NULL;
//     HANDLE OriginalImpersonationToken = NULL;
//     HANDLE NullToken = NULL;
//
//     if (hToken != NULL)
//     {
//         /* Check whether the user-provided token is a primary token */
//         // GetTokenInformation();
//         Status = NtQueryInformationToken(hToken,
//                                          TokenType,
//                                          &Type,
//                                          sizeof(Type),
//                                          &ReturnLength);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtQueryInformationToken() failed, Status 0x%08x\n", Status);
//             Success = FALSE;
//             goto Quit;
//         }
//
//         if (Type != TokenPrimary)
//         {
//             ERR("Wrong token type for token 0x%p, expected TokenPrimary, got %ld\n", hToken, Type);
//             Status = STATUS_BAD_TOKEN_TYPE;
//             Success = FALSE;
//             goto Quit;
//         }
//
//         /*
//          * Open the original token of the calling thread
//          * and halt the impersonation for the moment
//          * being. The opened thread token will be cached
//          * so that we will restore it back when we're done.
//          */
//         Status = NtOpenThreadToken(NtCurrentThread(),
//                                    TOKEN_QUERY | TOKEN_IMPERSONATE,
//                                    TRUE,
//                                    &OriginalImpersonationToken);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed? Does this thread have a token at least? */
//             OriginalImpersonationToken = NULL;
//             if (Status != STATUS_NO_TOKEN)
//             {
//                 /*
//                  * OK so this thread has a token but we
//                  * could not open it for whatever reason.
//                  * Bail out then.
//                  */
//                 ERR("Failed to open thread token with 0x%08lx\n", Status);
//                 Success = FALSE;
//                 goto Quit;
//             }
//         }
//         else
//         {
//             /* We succeeded, stop the impersonation for now */
//             Status = NtSetInformationThread(NtCurrentThread(),
//                                             ThreadImpersonationToken,
//                                             &NullToken,
//                                             sizeof(NullToken));
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("Failed to stop impersonation with 0x%08lx\n", Status);
//                 Success = FALSE;
//                 goto Quit;
//             }
//         }
//
//         /*
//          * Create a security descriptor that will be common for the
//          * newly created process on behalf of the context user.
//          */
//         if (!CreateDefaultProcessSecurityCommon(hToken, &DefaultSd))
//         {
//             ERR("Failed to create common security descriptor for the token for new process\n");
//             Success = FALSE;
//             goto Quit;
//         }
//
//         /*
//          * Duplicate the token for this new process. This token
//          * object will get a default security descriptor that we
//          * have created ourselves in ADVAPI32.
//          */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    NULL,
//                                    0,
//                                    NULL,
//                                    DefaultSd);
//         Status = NtDuplicateToken(hToken,
//                                   0,
//                                   &ObjectAttributes,
//                                   FALSE,
//                                   TokenPrimary,
//                                   &hTokenDup);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtDuplicateToken() failed, Status 0x%08x\n", Status);
//             Success = FALSE;
//             goto Quit;
//         }
//
//         /*
//          * Now it's time to set the primary token into
//          * the process. On the first try, do it by
//          * impersonating the security context of the
//          * calling process (impersonate as self).
//          */
//         Status = InsertTokenToProcessCommon(TRUE,
//                                             lpProcessInformation->hProcess,
//                                             lpProcessInformation->hThread,
//                                             hTokenDup);
//         if (!NT_SUCCESS(Status))
//         {
//             /*
//              * OK, we failed. Our second (and last try) is to not
//              * impersonate as self but instead we will try by setting
//              * the original impersonation (thread) token and set the
//              * primary token to the process through this way. This is
//              * what we call -- the "rinse and repeat" approach.
//              */
//             Status = NtSetInformationThread(NtCurrentThread(),
//                                             ThreadImpersonationToken,
//                                             &OriginalImpersonationToken,
//                                             sizeof(OriginalImpersonationToken));
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("Failed to restore impersonation token for setting process token, Status 0x%08lx\n", Status);
//                 NtClose(hTokenDup);
//                 Success = FALSE;
//                 goto Quit;
//             }
//
//             /* Retry again */
//             Status = InsertTokenToProcessCommon(FALSE,
//                                                 lpProcessInformation->hProcess,
//                                                 lpProcessInformation->hThread,
//                                                 hTokenDup);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Even the second try failed, bail out... */
//                 ERR("Failed to insert the primary token into process, Status 0x%08lx\n", Status);
//                 NtClose(hTokenDup);
//                 Success = FALSE;
//                 goto Quit;
//             }
//
//             /* All good, now stop impersonation */
//             Status = NtSetInformationThread(NtCurrentThread(),
//                                             ThreadImpersonationToken,
//                                             &NullToken,
//                                             sizeof(NullToken));
//             if (!NT_SUCCESS(Status))
//             {
//                 ERR("Failed to unset impersonationg token after setting process token, Status 0x%08lx\n", Status);
//                 NtClose(hTokenDup);
//                 Success = FALSE;
//                 goto Quit;
//             }
//         }
//
//         /*
//          * FIXME: As we have successfully set up a primary token to
//          * the newly created process, we must set up as well a definite
//          * limit of quota charges for this process on the context of
//          * this user.
//          */
//
//         /*
//          * As we have successfully set the token into the process now
//          * it is time that we set up new security information for both
//          * the process and its thread as well, that is, these securable
//          * objects will grant a security descriptor. The security descriptors
//          * provided by the caller take precedence so we should use theirs
//          * if possible in this case. Otherwise both the process and thread
//          * will receive the default security descriptor that we have created
//          * ourselves.
//          *
//          * BEAR IN MIND!!! AT THE MOMENT when these securable objects get new
//          * security information, the process (and the thread) can't be opened
//          * by the creator anymore as the new owner will take in charge of
//          * the process and future objects that are going to be created within
//          * the process. For further information in regard of the documentation
//          * see https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessasuserw.
//          */
//         if (lpProcessAttributes && lpProcessAttributes->lpSecurityDescriptor)
//         {
//             ProcessSd = lpProcessAttributes->lpSecurityDescriptor;
//         }
//         else
//         {
//             ProcessSd = DefaultSd;
//         }
//
//         if (lpThreadAttributes && lpThreadAttributes->lpSecurityDescriptor)
//         {
//             ThreadSd = lpThreadAttributes->lpSecurityDescriptor;
//         }
//         else
//         {
//             ThreadSd = DefaultSd;
//         }
//
//         /* Set new security info to the process and thread now */
//         if (!InsertProcessSecurityCommon(lpProcessInformation->hProcess,
//                                          lpProcessInformation->hThread,
//                                          ProcessSd,
//                                          ThreadSd))
//         {
//             ERR("Failed to set new security information for process and thread\n");
//             NtClose(hTokenDup);
//             Success = FALSE;
//             goto Quit;
//         }
//
//         /* Close the duplicated token */
//         NtClose(hTokenDup);
//         Success = TRUE;
//     }
//
//     /*
//      * If the caller did not supply a token then just declare
//      * ourselves as job done. The newly created process will use
//      * the default security context at this point anyway.
//      */
//     TRACE("No token supplied, the process will use default security context\n");
//     Success = TRUE;
//
// Quit:
//     /*
//      * If we successfully opened the thread token before
//      * and stopped the impersonation then we have to assign
//      * its original token back and close that token we have
//      * referenced it.
//      */
//     if (OriginalImpersonationToken != NULL)
//     {
//         StatusOnExit = NtSetInformationThread(NtCurrentThread(),
//                                               ThreadImpersonationToken,
//                                               &OriginalImpersonationToken,
//                                               sizeof(OriginalImpersonationToken));
//
//         /*
//          * We really must assert ourselves that we successfully
//          * set the original token back, otherwise if we fail
//          * then something is seriously going wrong....
//          * The status code is cached in a separate status
//          * variable because we would not want to tamper
//          * with the original status code that could have been
//          * returned by someone else above in this function code.
//          */
//         ASSERT(NT_SUCCESS(StatusOnExit));
//
//         /* De-reference it */
//         NtClose(OriginalImpersonationToken);
//     }
//
//     /* Terminate the process and set the last error status */
//     if (!NT_SUCCESS(Status))
//     {
//         TerminateProcess(lpProcessInformation->hProcess, Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//     }
//
//     /* Resume the main thread */
//     if (!(dwCreationFlags & CREATE_SUSPENDED))
//     {
//         ResumeThread(lpProcessInformation->hThread);
//     }
//
//     /* Free the security descriptor from memory */
//     if (DefaultSd != NULL)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DefaultSd);
//     }
//
//     return Success;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// CreateProcessAsUserA(
//      HANDLE hToken,
//      LPCSTR lpApplicationName,
//      LPSTR lpCommandLine,
//      LPSECURITY_ATTRIBUTES lpProcessAttributes,
//      LPSECURITY_ATTRIBUTES lpThreadAttributes,
//      BOOL bInheritHandles,
//      DWORD dwCreationFlags,
//      LPVOID lpEnvironment,
//      LPCSTR lpCurrentDirectory,
//      LPSTARTUPINFOA lpStartupInfo,
//      LPPROCESS_INFORMATION lpProcessInformation)
// {
//     TRACE("%p %s %s %p %p %d 0x%08x %p %s %p %p\n", hToken, debugstr_a(lpApplicationName),
//         debugstr_a(lpCommandLine), lpProcessAttributes, lpThreadAttributes, bInheritHandles,
//         dwCreationFlags, lpEnvironment, debugstr_a(lpCurrentDirectory), lpStartupInfo, lpProcessInformation);
//
//     /* Create the process with a suspended main thread */
//     if (!CreateProcessA(lpApplicationName,
//                         lpCommandLine,
//                         lpProcessAttributes,
//                         lpThreadAttributes,
//                         bInheritHandles,
//                         dwCreationFlags | CREATE_SUSPENDED,
//                         lpEnvironment,
//                         lpCurrentDirectory,
//                         lpStartupInfo,
//                         lpProcessInformation))
//     {
//         ERR("CreateProcessA failed, last error: %d\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Call the helper function */
//     return CreateProcessAsUserCommon(hToken,
//                                      dwCreationFlags,
//                                      lpProcessAttributes,
//                                      lpThreadAttributes,
//                                      lpProcessInformation);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// CreateProcessAsUserW(
//      HANDLE hToken,
//      LPCWSTR lpApplicationName,
//      LPWSTR lpCommandLine,
//      LPSECURITY_ATTRIBUTES lpProcessAttributes,
//      LPSECURITY_ATTRIBUTES lpThreadAttributes,
//      BOOL bInheritHandles,
//      DWORD dwCreationFlags,
//      LPVOID lpEnvironment,
//      LPCWSTR lpCurrentDirectory,
//      LPSTARTUPINFOW lpStartupInfo,
//      LPPROCESS_INFORMATION lpProcessInformation)
// {
//     TRACE("%p %s %s %p %p %d 0x%08x %p %s %p %p\n", hToken, debugstr_w(lpApplicationName),
//         debugstr_w(lpCommandLine), lpProcessAttributes, lpThreadAttributes, bInheritHandles,
//         dwCreationFlags, lpEnvironment, debugstr_w(lpCurrentDirectory), lpStartupInfo, lpProcessInformation);
//
//     /* Create the process with a suspended main thread */
//     if (!CreateProcessW(lpApplicationName,
//                         lpCommandLine,
//                         lpProcessAttributes,
//                         lpThreadAttributes,
//                         bInheritHandles,
//                         dwCreationFlags | CREATE_SUSPENDED,
//                         lpEnvironment,
//                         lpCurrentDirectory,
//                         lpStartupInfo,
//                         lpProcessInformation))
//     {
//         ERR("CreateProcessW failed, last error: %d\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Call the helper function */
//     return CreateProcessAsUserCommon(hToken,
//                                      dwCreationFlags,
//                                      lpProcessAttributes,
//                                      lpThreadAttributes,
//                                      lpProcessInformation);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LogonUserA(
//      LPSTR lpszUsername,
//      LPSTR lpszDomain,
//      LPSTR lpszPassword,
//      DWORD dwLogonType,
//      DWORD dwLogonProvider,
//      PHANDLE phToken)
// {
//     return LogonUserExA(lpszUsername,
//                         lpszDomain,
//                         lpszPassword,
//                         dwLogonType,
//                         dwLogonProvider,
//                         phToken,
//                         NULL,
//                         NULL,
//                         NULL,
//                         NULL);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LogonUserExA(
//      LPSTR lpszUsername,
//      LPSTR lpszDomain,
//      LPSTR lpszPassword,
//      DWORD dwLogonType,
//      DWORD dwLogonProvider,
//      PHANDLE phToken,
//      PSID *ppLogonSid,
//      PVOID *ppProfileBuffer,
//      LPDWORD pdwProfileLength,
//      PQUOTA_LIMITS pQuotaLimits)
// {
//     UNICODE_STRING UserName;
//     UNICODE_STRING Domain;
//     UNICODE_STRING Password;
//     BOOL ret = FALSE;
//
//     UserName.Buffer = NULL;
//     Domain.Buffer = NULL;
//     Password.Buffer = NULL;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&UserName, lpszUsername))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto UsernameDone;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Domain, lpszDomain))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto DomainDone;
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Password, lpszPassword))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto PasswordDone;
//     }
//
//     ret = LogonUserExW(UserName.Buffer,
//                        Domain.Buffer,
//                        Password.Buffer,
//                        dwLogonType,
//                        dwLogonProvider,
//                        phToken,
//                        ppLogonSid,
//                        ppProfileBuffer,
//                        pdwProfileLength,
//                        pQuotaLimits);
//
//     if (Password.Buffer != NULL)
//         RtlFreeUnicodeString(&Password);
//
// PasswordDone:
//     if (Domain.Buffer != NULL)
//         RtlFreeUnicodeString(&Domain);
//
// DomainDone:
//     if (UserName.Buffer != NULL)
//         RtlFreeUnicodeString(&UserName);
//
// UsernameDone:
//     return ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LogonUserW(
//      LPWSTR lpszUsername,
//      LPWSTR lpszDomain,
//      LPWSTR lpszPassword,
//      DWORD dwLogonType,
//      DWORD dwLogonProvider,
//      PHANDLE phToken)
// {
//     return LogonUserExW(lpszUsername,
//                         lpszDomain,
//                         lpszPassword,
//                         dwLogonType,
//                         dwLogonProvider,
//                         phToken,
//                         NULL,
//                         NULL,
//                         NULL,
//                         NULL);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// LogonUserExW(
//      LPWSTR lpszUsername,
//      LPWSTR lpszDomain,
//      LPWSTR lpszPassword,
//      DWORD dwLogonType,
//      DWORD dwLogonProvider,
//      PHANDLE phToken,
//      PSID *ppLogonSid,
//      PVOID *ppProfileBuffer,
//      LPDWORD pdwProfileLength,
//      PQUOTA_LIMITS pQuotaLimits)
// {
//     SID_IDENTIFIER_AUTHORITY LocalAuthority = {SECURITY_LOCAL_SID_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY SystemAuthority = {SECURITY_NT_AUTHORITY};
//     PSID LogonSid = NULL;
//     PSID LocalSid = NULL;
//     LSA_STRING OriginName;
//     UNICODE_STRING DomainName;
//     UNICODE_STRING UserName;
//     UNICODE_STRING Password;
//     PMSV1_0_INTERACTIVE_LOGON AuthInfo = NULL;
//     ULONG AuthInfoLength;
//     ULONG_PTR Ptr;
//     TOKEN_SOURCE TokenSrc;
//     PTOKEN_GROUPS TokenGrps = NULL;
//     PMSV1_0_INTERACTIVE_PROFILE ProfileBuffer = NULL;
//     ULONG ProfileBufferLength = 0;
//     LUID Luid = {0, 0};
//     LUID LogonId = {0, 0};
//     HANDLE TokenHandle = NULL;
//     QUOTA_LIMITS QuotaLimits;
//     SECURITY_LOGON_TYPE LogonType;
//     NTSTATUS SubStatus = STATUS_SUCCESS;
//     NTSTATUS Status;
//
//     if ((ppProfileBuffer != NULL && pdwProfileLength == NULL) ||
//         (ppProfileBuffer == NULL && pdwProfileLength != NULL))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (ppProfileBuffer != NULL && pdwProfileLength != NULL)
//     {
//         *ppProfileBuffer = NULL;
//         *pdwProfileLength = 0;
//     }
//
//     if (phToken != NULL)
//         *phToken = NULL;
//
//     switch (dwLogonType)
//     {
//         case LOGON32_LOGON_INTERACTIVE:
//             LogonType = Interactive;
//             break;
//
//         case LOGON32_LOGON_NETWORK:
//             LogonType = Network;
//             break;
//
//         case LOGON32_LOGON_BATCH:
//             LogonType = Batch;
//             break;
//
//         case LOGON32_LOGON_SERVICE:
//             LogonType = Service;
//             break;
//
//        default:
//             ERR("Invalid logon type: %ul\n", dwLogonType);
//             Status = STATUS_INVALID_PARAMETER;
//             goto done;
//     }
//
//     if (LsaHandle == NULL)
//     {
//         Status = OpenLogonLsaHandle();
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     RtlInitAnsiString((PANSI_STRING)&OriginName,
//                       "Advapi32 Logon");
//
//     RtlInitUnicodeString(&DomainName,
//                          lpszDomain);
//
//     RtlInitUnicodeString(&UserName,
//                          lpszUsername);
//
//     RtlInitUnicodeString(&Password,
//                          lpszPassword);
//
//     AuthInfoLength = sizeof(MSV1_0_INTERACTIVE_LOGON)+
//                      DomainName.MaximumLength +
//                      UserName.MaximumLength +
//                      Password.MaximumLength;
//
//     AuthInfo = RtlAllocateHeap(RtlGetProcessHeap(),
//                                HEAP_ZERO_MEMORY,
//                                AuthInfoLength);
//     if (AuthInfo == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     AuthInfo->MessageType = MsV1_0InteractiveLogon;
//
//     Ptr = (ULONG_PTR)AuthInfo + sizeof(MSV1_0_INTERACTIVE_LOGON);
//
//     AuthInfo->LogonDomainName.Length = DomainName.Length;
//     AuthInfo->LogonDomainName.MaximumLength = DomainName.MaximumLength;
//     AuthInfo->LogonDomainName.Buffer = (DomainName.Buffer == NULL) ? NULL : (PWCHAR)Ptr;
//     if (DomainName.MaximumLength > 0)
//     {
//         RtlCopyMemory(AuthInfo->LogonDomainName.Buffer,
//                       DomainName.Buffer,
//                       DomainName.MaximumLength);
//
//         Ptr += DomainName.MaximumLength;
//     }
//
//     AuthInfo->UserName.Length = UserName.Length;
//     AuthInfo->UserName.MaximumLength = UserName.MaximumLength;
//     AuthInfo->UserName.Buffer = (PWCHAR)Ptr;
//     if (UserName.MaximumLength > 0)
//         RtlCopyMemory(AuthInfo->UserName.Buffer,
//                       UserName.Buffer,
//                       UserName.MaximumLength);
//
//     Ptr += UserName.MaximumLength;
//
//     AuthInfo->Password.Length = Password.Length;
//     AuthInfo->Password.MaximumLength = Password.MaximumLength;
//     AuthInfo->Password.Buffer = (PWCHAR)Ptr;
//     if (Password.MaximumLength > 0)
//         RtlCopyMemory(AuthInfo->Password.Buffer,
//                       Password.Buffer,
//                       Password.MaximumLength);
//
//     /* Create the Logon SID */
//     AllocateLocallyUniqueId(&LogonId);
//     Status = RtlAllocateAndInitializeSid(&SystemAuthority,
//                                          SECURITY_LOGON_IDS_RID_COUNT,
//                                          SECURITY_LOGON_IDS_RID,
//                                          LogonId.HighPart,
//                                          LogonId.LowPart,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          &LogonSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Create the Local SID */
//     Status = RtlAllocateAndInitializeSid(&LocalAuthority,
//                                          1,
//                                          SECURITY_LOCAL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          SECURITY_NULL_RID,
//                                          &LocalSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate and set the token groups */
//     TokenGrps = RtlAllocateHeap(RtlGetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   sizeof(TOKEN_GROUPS) + ((2 - ANYSIZE_ARRAY) * sizeof(SID_AND_ATTRIBUTES)));
//     if (TokenGrps == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     TokenGrps->GroupCount = 2;
//     TokenGrps->Groups[0].Sid = LogonSid;
//     TokenGrps->Groups[0].Attributes = SE_GROUP_MANDATORY | SE_GROUP_ENABLED |
//                                         SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_LOGON_ID;
//     TokenGrps->Groups[1].Sid = LocalSid;
//     TokenGrps->Groups[1].Attributes = SE_GROUP_MANDATORY | SE_GROUP_ENABLED |
//                                         SE_GROUP_ENABLED_BY_DEFAULT;
//
//     /* Set the token source */
//     RtlCopyMemory(TokenSrc.SourceName,
//                   AdvapiTokenSourceName,
//                   sizeof(TokenSrc.SourceName));
//     AllocateLocallyUniqueId(&TokenSrc.SourceIdentifier);
//
//     Status = LsaLogonUser(LsaHandle,
//                           &OriginName,
//                           LogonType,
//                           AuthenticationPackage,
//                           (PVOID)AuthInfo,
//                           AuthInfoLength,
//                           TokenGrps,
//                           &TokenSrc,
//                           (PVOID*)&ProfileBuffer,
//                           &ProfileBufferLength,
//                           &Luid,
//                           &TokenHandle,
//                           &QuotaLimits,
//                           &SubStatus);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsaLogonUser failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     if (ProfileBuffer != NULL)
//     {
//         TRACE("ProfileBuffer: %p\n", ProfileBuffer);
//         TRACE("MessageType: %u\n", ProfileBuffer->MessageType);
//
//         TRACE("FullName: %p\n", ProfileBuffer->FullName.Buffer);
//         TRACE("FullName: %S\n", ProfileBuffer->FullName.Buffer);
//
//         TRACE("LogonServer: %p\n", ProfileBuffer->LogonServer.Buffer);
//         TRACE("LogonServer: %S\n", ProfileBuffer->LogonServer.Buffer);
//     }
//
//     TRACE("Luid: 0x%08lx%08lx\n", Luid.HighPart, Luid.LowPart);
//
//     if (TokenHandle != NULL)
//     {
//         TRACE("TokenHandle: %p\n", TokenHandle);
//     }
//
//     if (phToken != NULL)
//         *phToken = TokenHandle;
//
//     /* FIXME: return ppLogonSid and pQuotaLimits */
//
// done:
//     if (ProfileBuffer != NULL)
//         LsaFreeReturnBuffer(ProfileBuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (TokenHandle != NULL)
//             CloseHandle(TokenHandle);
//     }
//
//     if (TokenGrps != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, TokenGrps);
//
//     if (LocalSid != NULL)
//         RtlFreeSid(LocalSid);
//
//     if (LogonSid != NULL)
//         RtlFreeSid(LogonSid);
//
//     if (AuthInfo != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AuthInfo);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
