use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::externs::FreeLibrary;
use crate::externs::LocalAlloc;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// SE_OBJECT_TYPE enum variants used as match patterns (module-local — see prelude.rs).
const SE_FILE_OBJECT: SE_OBJECT_TYPE = advapi32_sys::_SE_OBJECT_TYPE_SE_FILE_OBJECT;
const SE_REGISTRY_KEY: SE_OBJECT_TYPE = advapi32_sys::_SE_OBJECT_TYPE_SE_REGISTRY_KEY;

extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn LsaClose(ObjectHandle: LSA_HANDLE) -> NTSTATUS;
    fn LsaFreeMemory(Buffer: PVOID) -> NTSTATUS;
    fn LsaLookupPrivilegeName( PolicyHandle: LSA_HANDLE, Value: PLUID, Name: *mut PUNICODE_STRING, ) -> NTSTATUS;
    fn LsaNtStatusToWinError(Status: NTSTATUS) -> ULONG;
    fn LsaOpenPolicy( SystemName: PUNICODE_STRING, ObjectAttributes: *mut _OBJECT_ATTRIBUTES, DesiredAccess: ULONG, PolicyHandle: *mut LSA_HANDLE, ) -> NTSTATUS;
    fn GetLastError() -> DWORD;
    fn InitializeSecurityDescriptor( pSecurityDescriptor: PSECURITY_DESCRIPTOR, dwRevision: DWORD, ) -> i32;
    fn SetSecurityDescriptorControl( pSecurityDescriptor: PSECURITY_DESCRIPTOR, ControlBitsOfInterest: u16, ControlBitsToSet: u16, ) -> i32;
    fn SetSecurityDescriptorDacl( pSecurityDescriptor: PSECURITY_DESCRIPTOR, bDaclPresent: i32, pDacl: PACL, bDaclDefaulted: i32, ) -> i32;
    fn SetSecurityDescriptorGroup( pSecurityDescriptor: PSECURITY_DESCRIPTOR, pGroup: PSID, bGroupDefaulted: i32, ) -> i32;
    fn SetSecurityDescriptorOwner( pSecurityDescriptor: PSECURITY_DESCRIPTOR, pOwner: PSID, bOwnerDefaulted: i32, ) -> i32;
    fn SetSecurityDescriptorSacl( pSecurityDescriptor: PSECURITY_DESCRIPTOR, bSaclPresent: i32, pSacl: PACL, bSaclDefaulted: i32, ) -> i32;
    // Acc{Rewrite,Tree}* are advapi32.h macros for NtMartaStatic.<field>(...) — no
    // real symbol; the call sites below dispatch through the function-pointer table.
    fn LocalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
// Exported: the not-yet-flipped C sec/ac.c references `extern NTMARTA NtMartaStatic`.
#[no_mangle]
pub static mut NtMartaStatic: NTMARTA = unsafe { core::mem::zeroed() };
static mut NtMarta: PNTMARTA = core::ptr::null_mut();

#[repr(C)]
pub struct _INTERNAL_FNPROGRESSW_DATA {
    pub fnProgress: FN_PROGRESSA,
    pub Args: PVOID,
}
pub type INTERNAL_FNPROGRESSW_DATA = _INTERNAL_FNPROGRESSW_DATA;
pub type PINTERNAL_FNPROGRESSW_DATA = *mut _INTERNAL_FNPROGRESSW_DATA;

unsafe fn LoadAndInitializeNtMarta() -> DWORD {
    NtMartaStatic.hDllInstance = LoadLibraryW(
        w!("ntmarta.dll").as_ptr() as LPCWSTR,
    );
    if NtMartaStatic.hDllInstance.is_null() {
        return GetLastError();
    }

    NtMartaStatic.RewriteGetNamedRights = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccRewriteGetNamedRights\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.RewriteGetNamedRights.is_none() {
        return GetLastError();
    }

    NtMartaStatic.RewriteSetNamedRights = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccRewriteSetNamedRights\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.RewriteSetNamedRights.is_none() {
        return GetLastError();
    }

    NtMartaStatic.RewriteGetHandleRights = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccRewriteGetHandleRights\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.RewriteGetHandleRights.is_none() {
        return GetLastError();
    }

    NtMartaStatic.RewriteSetHandleRights = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccRewriteSetHandleRights\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.RewriteSetHandleRights.is_none() {
        return GetLastError();
    }

    NtMartaStatic.RewriteSetEntriesInAcl = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccRewriteSetEntriesInAcl\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.RewriteSetEntriesInAcl.is_none() {
        return GetLastError();
    }

    NtMartaStatic.RewriteGetExplicitEntriesFromAcl = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccRewriteGetExplicitEntriesFromAcl\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.RewriteGetExplicitEntriesFromAcl.is_none() {
        return GetLastError();
    }

    NtMartaStatic.TreeResetNamedSecurityInfo = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccTreeResetNamedSecurityInfo\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.TreeResetNamedSecurityInfo.is_none() {
        return GetLastError();
    }

    NtMartaStatic.GetInheritanceSourceW = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccGetInheritanceSource\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.GetInheritanceSourceW.is_none() {
        return GetLastError();
    }

    NtMartaStatic.FreeIndexArray = core::mem::transmute(GetProcAddress(
        NtMartaStatic.hDllInstance,
        b"AccFreeIndexArray\0".as_ptr() as LPCSTR,
    ));
    if NtMartaStatic.FreeIndexArray.is_none() {
        return GetLastError();
    }

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CheckNtMartaPresent() -> DWORD {
    let mut ErrorCode: DWORD;

    if InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!(NtMarta) as *mut PVOID,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == core::ptr::null_mut()
    {
        ErrorCode = LoadAndInitializeNtMarta();

        if ErrorCode == ERROR_SUCCESS {
            if InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!(NtMarta) as *mut PVOID,
                core::ptr::addr_of_mut!(NtMartaStatic) as PVOID,
                core::ptr::null_mut(),
            ) != core::ptr::null_mut()
            {
                FreeLibrary((NtMartaStatic.hDllInstance as *mut core::ffi::c_void).cast());
            }
        }
    } else {
        ErrorCode = ERROR_SUCCESS;
    }

    ErrorCode
}
#[no_mangle]
pub unsafe extern "C" fn UnloadNtMarta() {
    if !InterlockedExchangePointer(
        core::ptr::addr_of_mut!(NtMarta) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    )
    .is_null()
    {
        FreeLibrary((NtMartaStatic.hDllInstance as *mut core::ffi::c_void).cast());
    }
}

#[no_mangle]
pub unsafe extern "system" fn ImpersonateAnonymousToken(ThreadHandle: HANDLE) -> BOOL {
    let Status: NTSTATUS = NtImpersonateAnonymousToken(ThreadHandle);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ImpersonateLoggedOnUser(hToken: HANDLE) -> BOOL {
    let mut Qos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut NewToken: HANDLE;
    let mut Type: TOKEN_TYPE = core::mem::zeroed();
    let mut ReturnLength: ULONG = 0;
    let Duplicated: BOOL;
    let mut Status: NTSTATUS;

    Status = NtQueryInformationToken(
        hToken,
        TokenType,
        core::ptr::addr_of_mut!(Type) as *mut _,
        core::mem::size_of::<TOKEN_TYPE>() as ULONG,
        &mut ReturnLength,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    if Type == TokenPrimary {
        Qos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG;
        Qos.ImpersonationLevel = SecurityImpersonation;
        Qos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as u8;
        Qos.EffectiveOnly = 0;

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::addr_of_mut!(Qos) as *mut _;

        NewToken = core::ptr::null_mut();
        Status = NtDuplicateToken(
            hToken,
            TOKEN_IMPERSONATE | TOKEN_QUERY,
            &mut ObjectAttributes,
            0,
            TokenImpersonation,
            &mut NewToken,
        );
        if !NT_SUCCESS(Status) {
            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }

        Duplicated = 1;
    } else {
        NewToken = hToken;
        Duplicated = 0;
    }

    Status = NtSetInformationThread(
        -2isize as HANDLE,
        ThreadImpersonationToken,
        core::ptr::addr_of_mut!(NewToken) as *mut _,
        core::mem::size_of::<HANDLE>() as ULONG,
    );

    if Duplicated != 0 {
        NtClose(NewToken);
    }

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetUserNameA(
    lpszName: LPSTR,
    lpSize: LPDWORD,
) -> BOOL {
    let mut NameW: _UNICODE_STRING = core::mem::zeroed();
    let mut NameA: ANSI_STRING = core::mem::zeroed();
    let Ret: BOOL;

    NameW.MaximumLength = ((*lpSize) * core::mem::size_of::<u16>() as u32) as u16;
    NameW.Buffer = LocalAlloc(LMEM_FIXED, (NameW.MaximumLength) as SIZE_T) as LPWSTR;
    if NameW.Buffer.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    NameA.Length = 0;
    NameA.MaximumLength = if (*lpSize) < 0xFFFF {
        (*lpSize) as u16
    } else {
        0xFFFF
    };
    NameA.Buffer = lpszName;

    Ret = GetUserNameW(NameW.Buffer, lpSize);
    if Ret != 0 {
        NameW.Length = ((*lpSize - 1) * core::mem::size_of::<u16>() as u32) as u16;
        RtlUnicodeStringToAnsiString(&mut NameA, &mut NameW, 0);

        *lpSize = NameA.Length as u32 + 1;
    }

    LocalFree(NameW.Buffer as *mut core::ffi::c_void);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetUserNameW(
    lpszName: LPWSTR,
    lpSize: LPDWORD,
) -> BOOL {
    let mut hToken: HANDLE = INVALID_HANDLE_VALUE as HANDLE;
    let mut tu_len: DWORD = 0;
    let mut tu_buf: *mut core::ffi::c_char;
    let token_user: *mut TOKEN_USER;
    let mut an_len: DWORD;
    let mut snu: SID_NAME_USE = SidTypeUser;
    let mut domain_name: *mut WCHAR;
    let mut dn_len: DWORD;

    if OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 0, &mut hToken) == 0 {
        let dwLastError = GetLastError();
        if dwLastError != ERROR_NO_TOKEN && dwLastError != ERROR_NO_IMPERSONATION_TOKEN {
            return 0;
        }

        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut hToken) == 0 {
            return 0;
        }
    }

    tu_buf = LocalAlloc(LMEM_FIXED, 36) as *mut core::ffi::c_char;
    if tu_buf.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        CloseHandle(hToken);
        return 0;
    }

    if GetTokenInformation(hToken, TokenUser, tu_buf as PVOID, 36, &mut tu_len) == 0 || tu_len > 36 {
        LocalFree(tu_buf as HLOCAL);
        tu_buf = LocalAlloc(LMEM_FIXED, (tu_len) as SIZE_T) as *mut core::ffi::c_char;
        if tu_buf.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            CloseHandle(hToken);
            return 0;
        }

        if GetTokenInformation(hToken, TokenUser, tu_buf as PVOID, tu_len, &mut tu_len) == 0 {
            LocalFree(tu_buf as HLOCAL);
            CloseHandle(hToken);
            return 0;
        }
    }

    CloseHandle(hToken);
    token_user = tu_buf as *mut TOKEN_USER;

    an_len = *lpSize;
    dn_len = 32;
    domain_name = LocalAlloc(LMEM_FIXED, ((dn_len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
    if domain_name.is_null() {
        LocalFree(tu_buf as HLOCAL);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    if LookupAccountSidW(
        core::ptr::null_mut(),
        (*token_user).User.Sid,
        lpszName,
        &mut an_len,
        domain_name,
        &mut dn_len,
        &mut snu,
    ) == 0
        || dn_len > 32
    {
        if dn_len > 32 {
            LocalFree(domain_name as HLOCAL);
            domain_name = LocalAlloc(LMEM_FIXED, ((dn_len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as *mut WCHAR;
            if domain_name.is_null() {
                LocalFree(tu_buf as HLOCAL);
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return 0;
            }
        }

        an_len = *lpSize;
        if LookupAccountSidW(
            core::ptr::null_mut(),
            (*token_user).User.Sid,
            lpszName,
            &mut an_len,
            domain_name,
            &mut dn_len,
            &mut snu,
        ) == 0
        {
            LocalFree(domain_name as HLOCAL);
            LocalFree(tu_buf as HLOCAL);
            *lpSize = an_len;
            return 0;
        }
    }

    LocalFree(domain_name as HLOCAL);
    LocalFree(tu_buf as HLOCAL);
    *lpSize = an_len + 1;
    1
}

#[no_mangle]
pub unsafe extern "system" fn LookupAccountSidA(
    lpSystemName: LPCSTR,
    lpSid: PSID,
    lpName: LPSTR,
    cchName: LPDWORD,
    lpReferencedDomainName: LPSTR,
    cchReferencedDomainName: LPDWORD,
    peUse: PSID_NAME_USE,
) -> BOOL {
    let mut NameW: _UNICODE_STRING = core::mem::zeroed();
    let mut ReferencedDomainNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut SystemNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut NameBuffer: LPWSTR;
    let mut ReferencedDomainNameBuffer: LPWSTR;
    let dwName: DWORD;
    let dwReferencedDomainName: DWORD;
    let Ret: BOOL;

    dwName = *cchName;
    dwReferencedDomainName = *cchReferencedDomainName;

    if dwName > 0 {
        NameBuffer = LocalAlloc(LMEM_FIXED, (dwName as usize * core::mem::size_of::<u16>()) as _) as LPWSTR;
        if NameBuffer.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return 0;
        }
    } else {
        NameBuffer = core::ptr::null_mut();
    }

    if dwReferencedDomainName > 0 {
        ReferencedDomainNameBuffer = LocalAlloc(LMEM_FIXED, (dwReferencedDomainName as usize * core::mem::size_of::<u16>()) as _) as LPWSTR;
        if ReferencedDomainNameBuffer.is_null() {
            if dwName > 0 {
                LocalFree(NameBuffer as _);
            }
            SetLastError(ERROR_OUTOFMEMORY);
            return 0;
        }
    } else {
        ReferencedDomainNameBuffer = core::ptr::null_mut();
    }

    if !lpSystemName.is_null() {
        let mut SystemNameA: ANSI_STRING = core::mem::zeroed();
        RtlInitAnsiString(&mut SystemNameA, lpSystemName as _);
        RtlAnsiStringToUnicodeString(&mut SystemNameW, &mut SystemNameA, 1);
    } else {
        SystemNameW.Buffer = core::ptr::null_mut();
    }

    Ret = LookupAccountSidW(
        SystemNameW.Buffer,
        lpSid,
        NameBuffer,
        cchName,
        ReferencedDomainNameBuffer,
        cchReferencedDomainName,
        peUse,
    );

    if Ret != 0 {
        if !lpName.is_null() {
            let mut NameA: ANSI_STRING = core::mem::zeroed();
            NameA.Length = 0;
            NameA.MaximumLength = if dwName <= 0xFFFF { dwName as u16 } else { 0xFFFF };
            NameA.Buffer = lpName as _;

            RtlInitUnicodeString(&mut NameW, NameBuffer);
            RtlUnicodeStringToAnsiString(&mut NameA, &mut NameW, 0);
            *NameA.Buffer.add(NameA.Length as usize) = b'\0' as _;
        }

        if !lpReferencedDomainName.is_null() {
            let mut ReferencedDomainNameA: ANSI_STRING = core::mem::zeroed();
            ReferencedDomainNameA.Length = 0;
            ReferencedDomainNameA.MaximumLength = if dwReferencedDomainName <= 0xFFFF {
                dwReferencedDomainName as u16
            } else {
                0xFFFF
            };
            ReferencedDomainNameA.Buffer = lpReferencedDomainName as _;

            RtlInitUnicodeString(&mut ReferencedDomainNameW, ReferencedDomainNameBuffer);
            RtlUnicodeStringToAnsiString(&mut ReferencedDomainNameA, &mut ReferencedDomainNameW, 0);
            *ReferencedDomainNameA.Buffer.add(ReferencedDomainNameA.Length as usize) = b'\0' as _;
        }
    }

    if !SystemNameW.Buffer.is_null() {
        RtlFreeUnicodeString(&mut SystemNameW);
    }

    if !NameBuffer.is_null() {
        LocalFree(NameBuffer as _);
    }

    if !ReferencedDomainNameBuffer.is_null() {
        LocalFree(ReferencedDomainNameBuffer as _);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn LookupAccountSidW(
    pSystemName: LPCWSTR,
    pSid: PSID,
    pAccountName: LPWSTR,
    pdwAccountName: LPDWORD,
    pDomainName: LPWSTR,
    pdwDomainName: LPDWORD,
    peUse: PSID_NAME_USE,
) -> BOOL {
    let mut SystemName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ReferencedDomain: PLSA_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut TranslatedName: PLSA_TRANSLATED_NAME = core::ptr::null_mut();
    let mut ret: BOOL;
    let dwAccountName: DWORD;
    let dwDomainName: DWORD;

    RtlInitUnicodeString(&mut SystemName, pSystemName);
    Status = LsaOpenPolicy(
        &mut SystemName,
        &mut ObjectAttributes,
        POLICY_LOOKUP_NAMES,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    let mut pSidLocal = pSid;
    Status = LsaLookupSids(
        PolicyHandle,
        1,
        &mut pSidLocal,
        &mut ReferencedDomain,
        &mut TranslatedName,
    );

    LsaClose(PolicyHandle);

    if !NT_SUCCESS(Status) || Status == STATUS_SOME_NOT_MAPPED {
        SetLastError(LsaNtStatusToWinError(Status));
        ret = 0;
    } else {
        ret = 1;

        dwAccountName = ((*TranslatedName).Name.Length as DWORD) / (core::mem::size_of::<u16>() as DWORD);
        if !ReferencedDomain.is_null() && (*ReferencedDomain).Entries > 0 {
            dwDomainName = ((*(*ReferencedDomain).Domains.offset(0)).Name.Length as DWORD)
                / (core::mem::size_of::<u16>() as DWORD);
        } else {
            dwDomainName = 0;
        }

        if *pdwAccountName <= dwAccountName || *pdwDomainName <= dwDomainName {
            *pdwAccountName = dwAccountName + 1;
            *pdwDomainName = dwDomainName + 1;
            ret = 0;
        } else {
            if dwAccountName != 0 {
                RtlCopyMemory(
                    pAccountName as *mut core::ffi::c_void,
                    (*TranslatedName).Name.Buffer as *const core::ffi::c_void,
                    (dwAccountName as usize) * core::mem::size_of::<u16>(),
                );
            }
            *pAccountName.offset(dwAccountName as isize) = 0;

            if dwDomainName != 0 {
                RtlCopyMemory(
                    pDomainName as *mut core::ffi::c_void,
                    (*(*ReferencedDomain).Domains.offset(0)).Name.Buffer as *const core::ffi::c_void,
                    (dwDomainName as usize) * core::mem::size_of::<u16>(),
                );
            }
            *pDomainName.offset(dwDomainName as isize) = 0;

            *pdwAccountName = dwAccountName;
            *pdwDomainName = dwDomainName;

            if !peUse.is_null() {
                *peUse = (*TranslatedName).Use;
            }
        }

        if ret == 0 {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
        }
    }

    if !ReferencedDomain.is_null() {
        LsaFreeMemory(ReferencedDomain as PVOID);
    }

    if !TranslatedName.is_null() {
        LsaFreeMemory(TranslatedName as PVOID);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn LookupAccountNameW(
    lpSystemName: LPCWSTR,
    lpAccountName: LPCWSTR,
    Sid: PSID,
    cbSid: LPDWORD,
    ReferencedDomainName: LPWSTR,
    cchReferencedDomainName: LPDWORD,
    peUse: PSID_NAME_USE,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SystemName: _UNICODE_STRING = core::mem::zeroed();
    let mut AccountName: _UNICODE_STRING = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut ReferencedDomains: PLSA_REFERENCED_DOMAIN_LIST = core::ptr::null_mut();
    let mut TranslatedSid: PLSA_TRANSLATED_SID = core::ptr::null_mut();
    let pDomainSid: PSID;
    let dwDomainNameLength: DWORD;
    let dwSidLength: DWORD;
    let nSubAuthorities: UCHAR;
    let mut bResult: BOOL;
    let Status: NTSTATUS;

    RtlInitUnicodeString(&mut SystemName, lpSystemName);

    Status = LsaOpenPolicy(
        if !lpSystemName.is_null() {
            &mut SystemName
        } else {
            core::ptr::null_mut()
        },
        &mut ObjectAttributes,
        POLICY_LOOKUP_NAMES,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    RtlInitUnicodeString(&mut AccountName, lpAccountName);

    let Status = LsaLookupNames(
        PolicyHandle,
        1,
        core::ptr::addr_of_mut!(AccountName) as PLSA_UNICODE_STRING,
        &mut ReferencedDomains,
        &mut TranslatedSid,
    );

    LsaClose(PolicyHandle);

    if !NT_SUCCESS(Status) || Status == STATUS_SOME_NOT_MAPPED {
        SetLastError(LsaNtStatusToWinError(Status));
        bResult = 0;
    } else {
        pDomainSid = (*(*ReferencedDomains)
            .Domains
            .offset((*TranslatedSid).DomainIndex as isize))
        .Sid;
        nSubAuthorities = *GetSidSubAuthorityCount(pDomainSid);
        dwSidLength = GetSidLengthRequired(nSubAuthorities + 1);

        dwDomainNameLength =
            (*(*ReferencedDomains).Domains).Name.Length as DWORD / core::mem::size_of::<u16>() as DWORD;

        if *cbSid < dwSidLength || *cchReferencedDomainName < dwDomainNameLength + 1 {
            *cbSid = dwSidLength;
            *cchReferencedDomainName = dwDomainNameLength + 1;

            bResult = 0;
        } else {
            CopySid(*cbSid, Sid, pDomainSid);
            *GetSidSubAuthorityCount(Sid) = nSubAuthorities + 1;
            *GetSidSubAuthority(Sid, nSubAuthorities as DWORD) = (*TranslatedSid).RelativeId;

            RtlCopyMemory(
                ReferencedDomainName as *mut core::ffi::c_void,
                (*(*ReferencedDomains).Domains).Name.Buffer as *const core::ffi::c_void,
                (dwDomainNameLength as usize) * core::mem::size_of::<u16>(),
            );
            *ReferencedDomainName.offset(dwDomainNameLength as isize) = 0u16;

            *cchReferencedDomainName = dwDomainNameLength;

            *peUse = (*TranslatedSid).Use;

            bResult = 1;
        }

        if bResult == 0 {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
        }
    }

    if !ReferencedDomains.is_null() {
        LsaFreeMemory(ReferencedDomains as *mut core::ffi::c_void);
    }

    if !TranslatedSid.is_null() {
        LsaFreeMemory(TranslatedSid as *mut core::ffi::c_void);
    }

    bResult
}

#[no_mangle]
pub unsafe extern "system" fn LookupPrivilegeValueA(
    lpSystemName: LPCSTR,
    lpName: LPCSTR,
    lpLuid: PLUID,
) -> BOOL {
    let mut SystemName: _UNICODE_STRING = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let Result: BOOL;

    /* Remote system? */
    if !lpSystemName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(SystemName),
            lpSystemName as LPSTR,
        );
    } else {
        SystemName.Buffer = core::ptr::null_mut();
    }

    /* Check the privilege name is not NULL */
    if lpName.is_null() {
        SetLastError(ERROR_NO_SUCH_PRIVILEGE);
        return 0;
    }

    RtlCreateUnicodeStringFromAsciiz(
        core::ptr::addr_of_mut!(Name),
        lpName as LPSTR,
    );

    Result = LookupPrivilegeValueW(SystemName.Buffer, Name.Buffer, lpLuid);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(Name));

    /* Remote system? */
    if !SystemName.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(SystemName));
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn LookupPrivilegeValueW(
    lpSystemName: LPCWSTR,
    lpPrivilegeName: LPCWSTR,
    lpLuid: PLUID,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SystemName: _UNICODE_STRING = core::mem::zeroed();
    let mut PrivilegeName: _UNICODE_STRING = core::mem::zeroed();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        b"%S,%S,%p\n\0".as_ptr() as LPCSTR,
        lpSystemName,
        lpPrivilegeName,
        lpLuid,
    );

    RtlInitUnicodeString(&mut SystemName, lpSystemName);

    Status = LsaOpenPolicy(
        if !lpSystemName.is_null() {
            &mut SystemName
        } else {
            core::ptr::null_mut()
        },
        &mut ObjectAttributes,
        POLICY_LOOKUP_NAMES,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    RtlInitUnicodeString(&mut PrivilegeName, lpPrivilegeName);

    Status = LsaLookupPrivilegeValue(PolicyHandle, core::ptr::addr_of_mut!(PrivilegeName) as PLSA_UNICODE_STRING, lpLuid);

    LsaClose(PolicyHandle);

    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn LookupPrivilegeNameW(
    lpSystemName: LPCWSTR,
    lpLuid: PLUID,
    lpName: LPWSTR,
    cchName: LPDWORD,
) -> BOOL {

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SystemName: _UNICODE_STRING = core::mem::zeroed();
    let mut PrivilegeName: PUNICODE_STRING = core::ptr::null_mut();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut SystemName, lpSystemName);

    Status = LsaOpenPolicy(
        if !lpSystemName.is_null() {
            &mut SystemName
        } else {
            core::ptr::null_mut()
        },
        &mut ObjectAttributes,
        POLICY_LOOKUP_NAMES,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    Status = LsaLookupPrivilegeName(PolicyHandle, lpLuid, &mut PrivilegeName);
    if NT_SUCCESS(Status) {
        if (*PrivilegeName).Length as usize + core::mem::size_of::<u16>()
            > (*cchName) as usize * core::mem::size_of::<u16>()
        {
            Status = STATUS_BUFFER_TOO_SMALL;

            *cchName = (((*PrivilegeName).Length as usize + core::mem::size_of::<u16>())
                / core::mem::size_of::<u16>()) as DWORD;
        } else {
            core::ptr::copy(
                (*PrivilegeName).Buffer as *const u8,
                lpName as *mut u8,
                (*PrivilegeName).Length as usize,
            );
            *lpName.add((*PrivilegeName).Length as usize / core::mem::size_of::<u16>()) = 0;

            *cchName = ((*PrivilegeName).Length as usize / core::mem::size_of::<u16>()) as DWORD;
        }

        LsaFreeMemory((*PrivilegeName).Buffer as PVOID);
        LsaFreeMemory(PrivilegeName as PVOID);
    }

    LsaClose(PolicyHandle);

    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn LookupPrivilegeDisplayNameW(
    lpSystemName: LPCWSTR,
    lpName: LPCWSTR,
    lpDisplayName: LPWSTR,
    cchDisplayName: LPDWORD,
    lpLanguageId: LPDWORD,
) -> BOOL {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SystemName: _UNICODE_STRING = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut DisplayName: PUNICODE_STRING = core::ptr::null_mut();
    let mut PolicyHandle: LSA_HANDLE = core::ptr::null_mut();
    let mut LanguageId: USHORT = 0;
    let mut Status: NTSTATUS;

    TRACE!(
        b"%S,%S,%p,%p,%p\n\0".as_ptr() as LPCSTR,
        lpSystemName,
        lpName,
        lpDisplayName,
        cchDisplayName,
        lpLanguageId,
    );

    RtlInitUnicodeString(&mut SystemName, lpSystemName);
    RtlInitUnicodeString(&mut Name, lpName);

    Status = LsaOpenPolicy(
        if !lpSystemName.is_null() {
            &mut SystemName
        } else {
            core::ptr::null_mut()
        },
        &mut ObjectAttributes,
        POLICY_LOOKUP_NAMES,
        &mut PolicyHandle,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    Status = LsaLookupPrivilegeDisplayName(PolicyHandle, core::ptr::addr_of_mut!(Name) as PLSA_UNICODE_STRING, core::ptr::addr_of_mut!(DisplayName) as *mut PLSA_UNICODE_STRING, &mut LanguageId);
    if NT_SUCCESS(Status) {
        *lpLanguageId = LanguageId as DWORD;
        if (*DisplayName).Length as usize + core::mem::size_of::<u16>()
            > (*cchDisplayName as usize) * core::mem::size_of::<u16>()
        {
            Status = STATUS_BUFFER_TOO_SMALL;

            *cchDisplayName = (((*DisplayName).Length as usize + core::mem::size_of::<u16>())
                / core::mem::size_of::<u16>()) as DWORD;
        } else {
            core::ptr::copy(
                (*DisplayName).Buffer as *const u8,
                lpDisplayName as *mut u8,
                (*DisplayName).Length as usize,
            );
            *lpDisplayName.add((*DisplayName).Length as usize / core::mem::size_of::<u16>()) = 0;

            *cchDisplayName = ((*DisplayName).Length as usize / core::mem::size_of::<u16>()) as DWORD;
        }

        LsaFreeMemory((*DisplayName).Buffer as PVOID);
        LsaFreeMemory(DisplayName as PVOID);
    }

    LsaClose(PolicyHandle);

    if !NT_SUCCESS(Status) {
        SetLastError(LsaNtStatusToWinError(Status));
        return 0;
    }

    1
}

unsafe fn pGetSecurityInfoCheck(
    SecurityInfo: SECURITY_INFORMATION,
    ppsidOwner: *mut PSID,
    ppsidGroup: *mut PSID,
    ppDacl: *mut PACL,
    ppSacl: *mut PACL,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    if (SecurityInfo
        & (OWNER_SECURITY_INFORMATION
            | GROUP_SECURITY_INFORMATION
            | DACL_SECURITY_INFORMATION
            | SACL_SECURITY_INFORMATION))
        != 0
        && ppSecurityDescriptor.is_null()
    {
        ERROR_INVALID_PARAMETER
    } else {
        if (SecurityInfo & OWNER_SECURITY_INFORMATION) != 0 && !ppsidOwner.is_null() {
            *ppsidOwner = core::ptr::null_mut();
        }
        if (SecurityInfo & GROUP_SECURITY_INFORMATION) != 0 && !ppsidGroup.is_null() {
            *ppsidGroup = core::ptr::null_mut();
        }
        if (SecurityInfo & DACL_SECURITY_INFORMATION) != 0 && !ppDacl.is_null() {
            *ppDacl = core::ptr::null_mut();
        }
        if (SecurityInfo & SACL_SECURITY_INFORMATION) != 0 && !ppSacl.is_null() {
            *ppSacl = core::ptr::null_mut();
        }

        if (SecurityInfo
            & (OWNER_SECURITY_INFORMATION
                | GROUP_SECURITY_INFORMATION
                | DACL_SECURITY_INFORMATION
                | SACL_SECURITY_INFORMATION))
            != 0
        {
            *ppSecurityDescriptor = core::ptr::null_mut();
        }

        ERROR_SUCCESS
    }
}

unsafe fn pSetSecurityInfoCheck(
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    SecurityInfo: SECURITY_INFORMATION,
    psidOwner: PSID,
    psidGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
) -> DWORD {

    if InitializeSecurityDescriptor(pSecurityDescriptor, SECURITY_DESCRIPTOR_REVISION as DWORD)
        == 0
    {
        return GetLastError();
    }

    if SecurityInfo & OWNER_SECURITY_INFORMATION != 0 {
        if RtlValidSid(psidOwner) != 0 {
            if SetSecurityDescriptorOwner(pSecurityDescriptor, psidOwner, 0) == 0 {
                return GetLastError();
            }
        } else {
            return ERROR_INVALID_PARAMETER;
        }
    }

    if SecurityInfo & GROUP_SECURITY_INFORMATION != 0 {
        if RtlValidSid(psidGroup) != 0 {
            if SetSecurityDescriptorGroup(pSecurityDescriptor, psidGroup, 0) == 0 {
                return GetLastError();
            }
        } else {
            return ERROR_INVALID_PARAMETER;
        }
    }

    if SecurityInfo & DACL_SECURITY_INFORMATION != 0 {
        let mut protect_dacl = false;
        if pDacl != core::ptr::null_mut() {
            if SetSecurityDescriptorDacl(pSecurityDescriptor, 1, pDacl, 0) != 0 {
                if SecurityInfo & PROTECTED_DACL_SECURITY_INFORMATION != 0 {
                    protect_dacl = true;
                }
            } else {
                return GetLastError();
            }
        } else {
            protect_dacl = true;
        }

        if protect_dacl {
            if SetSecurityDescriptorControl(
                pSecurityDescriptor,
                SE_DACL_PROTECTED as u16,
                SE_DACL_PROTECTED as u16,
            ) == 0
            {
                return GetLastError();
            }
        }
    }

    if SecurityInfo & SACL_SECURITY_INFORMATION != 0 {
        let mut protect_sacl = false;
        if pSacl != core::ptr::null_mut() {
            if SetSecurityDescriptorSacl(pSecurityDescriptor, 1, pSacl, 0) != 0 {
                if SecurityInfo & PROTECTED_SACL_SECURITY_INFORMATION != 0 {
                    protect_sacl = true;
                }
            } else {
                return GetLastError();
            }
        } else {
            protect_sacl = true;
        }

        if protect_sacl {
            if SetSecurityDescriptorControl(
                pSecurityDescriptor,
                SE_SACL_PROTECTED as u16,
                SE_SACL_PROTECTED as u16,
            ) == 0
            {
                return GetLastError();
            }
        }
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedSecurityInfoW(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    ppsidOwner: *mut PSID,
    ppsidGroup: *mut PSID,
    ppDacl: *mut PACL,
    ppSacl: *mut PACL,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut ErrorCode: DWORD;

    if !pObjectName.is_null() {
        ErrorCode = CheckNtMartaPresent();
        if ErrorCode == ERROR_SUCCESS {
            ErrorCode = pGetSecurityInfoCheck(
                SecurityInfo,
                ppsidOwner,
                ppsidGroup,
                ppDacl,
                ppSacl,
                ppSecurityDescriptor,
            );

            if ErrorCode == ERROR_SUCCESS {
                ErrorCode = (NtMartaStatic.RewriteGetNamedRights.unwrap())(
                    pObjectName,
                    ObjectType,
                    SecurityInfo,
                    ppsidOwner,
                    ppsidGroup,
                    // PACL differs by crate type-identity (ntdll_sys vs advapi32_sys),
                    // layout-identical — cast to the NTMARTA fn's expected pointer type.
                    ppDacl as _,
                    ppSacl as _,
                    ppSecurityDescriptor,
                );
            }
        }
    } else {
        ErrorCode = ERROR_INVALID_PARAMETER;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn SetNamedSecurityInfoW(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    psidOwner: PSID,
    psidGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
) -> DWORD {
    let mut ErrorCode: DWORD;

    if pObjectName != core::ptr::null_mut() {
        ErrorCode = CheckNtMartaPresent();
        if ErrorCode == ERROR_SUCCESS {
            let mut SecurityDescriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();

            ErrorCode = pSetSecurityInfoCheck(
                core::ptr::addr_of_mut!(SecurityDescriptor) as PSECURITY_DESCRIPTOR,
                SecurityInfo,
                psidOwner,
                psidGroup,
                pDacl,
                pSacl,
            );

            if ErrorCode == ERROR_SUCCESS {
                ErrorCode = (NtMartaStatic.RewriteSetNamedRights.unwrap())(
                    pObjectName,
                    ObjectType,
                    SecurityInfo,
                    core::ptr::addr_of_mut!(SecurityDescriptor) as PSECURITY_DESCRIPTOR,
                );
            }
        }
    } else {
        ErrorCode = ERROR_INVALID_PARAMETER;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn GetSecurityInfo(
    handle: HANDLE,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    ppsidOwner: *mut PSID,
    ppsidGroup: *mut PSID,
    ppDacl: *mut PACL,
    ppSacl: *mut PACL,
    ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> DWORD {
    let mut ErrorCode: DWORD;

    if handle != core::ptr::null_mut() {
        ErrorCode = CheckNtMartaPresent();
        if ErrorCode == ERROR_SUCCESS {
            ErrorCode = pGetSecurityInfoCheck(
                SecurityInfo,
                ppsidOwner,
                ppsidGroup,
                ppDacl,
                ppSacl,
                ppSecurityDescriptor,
            );

            if ErrorCode == ERROR_SUCCESS {
                ErrorCode = (NtMartaStatic.RewriteGetHandleRights.unwrap())(
                    handle,
                    ObjectType,
                    SecurityInfo,
                    ppsidOwner,
                    ppsidGroup,
                    ppDacl as _,
                    ppSacl as _,
                    ppSecurityDescriptor,
                );
            }
        }
    } else {
        ErrorCode = ERROR_INVALID_HANDLE;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn SetSecurityInfo(
    handle: HANDLE,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    psidOwner: PSID,
    psidGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
) -> DWORD {
    let mut ErrorCode: DWORD;

    if handle != core::ptr::null_mut() {
        ErrorCode = CheckNtMartaPresent();
        if ErrorCode == ERROR_SUCCESS {
            let mut SecurityDescriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();

            ErrorCode = pSetSecurityInfoCheck(
                core::ptr::addr_of_mut!(SecurityDescriptor) as PSECURITY_DESCRIPTOR,
                SecurityInfo,
                psidOwner,
                psidGroup,
                pDacl,
                pSacl,
            );

            if ErrorCode == ERROR_SUCCESS {
                ErrorCode = (NtMartaStatic.RewriteSetHandleRights.unwrap())(
                    handle,
                    ObjectType,
                    SecurityInfo,
                    core::ptr::addr_of_mut!(SecurityDescriptor) as PSECURITY_DESCRIPTOR,
                );
            }
        }
    } else {
        ErrorCode = ERROR_INVALID_HANDLE;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn CreatePrivateObjectSecurity(
    ParentDescriptor: PSECURITY_DESCRIPTOR,
    CreatorDescriptor: PSECURITY_DESCRIPTOR,
    NewDescriptor: *mut PSECURITY_DESCRIPTOR,
    IsDirectoryObject: i32,
    Token: HANDLE,
    GenericMapping: PGENERIC_MAPPING,
) -> i32 {
    let Status = RtlNewSecurityObject(
        ParentDescriptor,
        CreatorDescriptor,
        NewDescriptor,
        IsDirectoryObject as u8,
        Token,
        GenericMapping,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CreatePrivateObjectSecurityEx(
    ParentDescriptor: PSECURITY_DESCRIPTOR,
    CreatorDescriptor: PSECURITY_DESCRIPTOR,
    NewDescriptor: *mut PSECURITY_DESCRIPTOR,
    ObjectType: *mut GUID,
    IsContainerObject: BOOL,
    AutoInheritFlags: ULONG,
    Token: HANDLE,
    GenericMapping: PGENERIC_MAPPING,
) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn CreatePrivateObjectSecurityWithMultipleInheritance(
    ParentDescriptor: PSECURITY_DESCRIPTOR,
    CreatorDescriptor: PSECURITY_DESCRIPTOR,
    NewDescriptor: *mut PSECURITY_DESCRIPTOR,
    ObjectTypes: *mut *mut GUID,
    GuidCount: u32,
    IsContainerObject: i32,
    AutoInheritFlags: u32,
    Token: HANDLE,
    GenericMapping: PGENERIC_MAPPING,
) -> i32 {
    CreatePrivateObjectSecurity(
        ParentDescriptor,
        CreatorDescriptor,
        NewDescriptor,
        IsContainerObject,
        Token,
        GenericMapping,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DestroyPrivateObjectSecurity(
    ObjectDescriptor: *mut PSECURITY_DESCRIPTOR,
) -> BOOL {

    let Status = RtlDeleteSecurityObject(ObjectDescriptor);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetPrivateObjectSecurity(
    ObjectDescriptor: PSECURITY_DESCRIPTOR,
    SecurityInformation: SECURITY_INFORMATION,
    ResultantDescriptor: PSECURITY_DESCRIPTOR,
    DescriptorLength: DWORD,
    ReturnLength: PDWORD,
) -> BOOL {
    let Status = RtlQuerySecurityObject(
        ObjectDescriptor,
        SecurityInformation,
        ResultantDescriptor,
        DescriptorLength,
        ReturnLength,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetPrivateObjectSecurity(
    SecurityInformation: SECURITY_INFORMATION,
    ModificationDescriptor: PSECURITY_DESCRIPTOR,
    ObjectsSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    GenericMapping: PGENERIC_MAPPING,
    Token: HANDLE,
) -> BOOL {

    let Status = RtlSetSecurityObject(
        SecurityInformation,
        ModificationDescriptor,
        ObjectsSecurityDescriptor,
        GenericMapping,
        Token,
    );
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn TreeResetNamedSecurityInfoW(
    pObjectName: LPWSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    pOwner: PSID,
    pGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
    KeepExplicit: BOOL,
    fnProgress: FN_PROGRESSW,
    ProgressInvokeSetting: PROG_INVOKE_SETTING,
    Args: PVOID,
) -> DWORD {
    let mut ErrorCode: DWORD;

    if pObjectName != core::ptr::null_mut() {
        ErrorCode = CheckNtMartaPresent();
        if ErrorCode == ERROR_SUCCESS {
            match ObjectType {
                SE_FILE_OBJECT | SE_REGISTRY_KEY => {
                    if ((SecurityInfo & DACL_SECURITY_INFORMATION) != 0
                        && (SecurityInfo
                            & (PROTECTED_DACL_SECURITY_INFORMATION
                                | UNPROTECTED_DACL_SECURITY_INFORMATION))
                            == (PROTECTED_DACL_SECURITY_INFORMATION
                                | UNPROTECTED_DACL_SECURITY_INFORMATION))
                        || ((SecurityInfo & SACL_SECURITY_INFORMATION) != 0
                            && (SecurityInfo
                                & (PROTECTED_SACL_SECURITY_INFORMATION
                                    | UNPROTECTED_SACL_SECURITY_INFORMATION))
                                == (PROTECTED_SACL_SECURITY_INFORMATION
                                    | UNPROTECTED_SACL_SECURITY_INFORMATION))
                    {
                        ErrorCode = ERROR_INVALID_PARAMETER;
                    } else {
                        ErrorCode = (NtMartaStatic.TreeResetNamedSecurityInfo.unwrap())(
                            pObjectName,
                            ObjectType,
                            SecurityInfo,
                            pOwner,
                            pGroup,
                            pDacl as _,
                            pSacl as _,
                            KeepExplicit,
                            fnProgress,
                            ProgressInvokeSetting,
                            Args,
                        );
                    }
                }
                _ => {
                    ErrorCode = ERROR_INVALID_PARAMETER;
                }
            }
        }
    } else {
        ErrorCode = ERROR_INVALID_PARAMETER;
    }

    ErrorCode
}
// A FN_PROGRESSW: TreeResetNamedSecurityInfoA hands it to the W call, so it is WINAPI.
unsafe extern "system" fn InternalfnProgressW(
    pObjectName: LPWSTR,
    Status: DWORD,
    pInvokeSetting: PPROG_INVOKE_SETTING,
    Args: PVOID,
    SecuritySet: BOOL,
) {
    let pifnProgressData = Args as PINTERNAL_FNPROGRESSW_DATA;

    let ObjectNameSize = WideCharToMultiByte(
        CP_ACP,
        0,
        pObjectName,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if ObjectNameSize > 0 {
        let pObjectNameA = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ObjectNameSize as SIZE_T,
        ) as LPSTR;
        if !pObjectNameA.is_null() {
            *pObjectNameA = 0;
            WideCharToMultiByte(
                CP_ACP,
                0,
                pObjectName,
                -1,
                pObjectNameA,
                ObjectNameSize,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            ((*pifnProgressData).fnProgress.unwrap())(
                pObjectNameA,
                Status,
                pInvokeSetting,
                (*pifnProgressData).Args,
                SecuritySet,
            );

            RtlFreeHeap(RtlGetProcessHeap(), 0, pObjectNameA as PVOID);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn TreeResetNamedSecurityInfoA(
    pObjectName: LPSTR,
    ObjectType: SE_OBJECT_TYPE,
    SecurityInfo: SECURITY_INFORMATION,
    pOwner: PSID,
    pGroup: PSID,
    pDacl: PACL,
    pSacl: PACL,
    KeepExplicit: BOOL,
    fnProgress: FN_PROGRESSA,
    ProgressInvokeSetting: PROG_INVOKE_SETTING,
    Args: PVOID,
) -> DWORD {
    // advapi32 builds UNICODE at _WIN32_WINNT 0x502, so accctrl.h defines HAS_FN_PROGRESSW
    // and the C's live arm is this thunk to the W version. The translation took the
    // `#ifndef HAS_FN_PROGRESSW` stub arm and returned ERROR_CALL_NOT_IMPLEMENTED.
    let mut ifnProgressData: INTERNAL_FNPROGRESSW_DATA = core::mem::zeroed();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(ObjectName), pObjectName) == 0 {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    ifnProgressData.fnProgress = fnProgress;
    ifnProgressData.Args = Args;

    let Ret = TreeResetNamedSecurityInfoW(
        ObjectName.Buffer,
        ObjectType,
        SecurityInfo,
        pOwner,
        pGroup,
        pDacl,
        pSacl,
        KeepExplicit,
        if fnProgress.is_some() { Some(InternalfnProgressW) } else { None },
        ProgressInvokeSetting,
        core::ptr::addr_of_mut!(ifnProgressData) as PVOID,
    );

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ObjectName));

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * WINE COPYRIGHT:
//  * Copyright 1999, 2000 Juergen Schmied <juergen.schmied@debitel.net>
//  * Copyright 2003 CodeWeavers Inc. (Ulrich Czekalla)
//  * Copyright 2006 Robert Reif
//  *
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/advapi32/sec/misc.c
//  * PURPOSE:         Miscellaneous security functions (some ported from Wine)
//  */
//
// #include <advapi32.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /* Interface to ntmarta.dll ***************************************************/
//
// NTMARTA NtMartaStatic = { 0 };
// static PNTMARTA NtMarta = NULL;
//
// #define FindNtMartaProc(Name)                                                  \
//     NtMartaStatic.Name = (PVOID)GetProcAddress(NtMartaStatic.hDllInstance,     \
//                                                "Acc" # Name );                 \
//     if (NtMartaStatic.Name == NULL)                                            \
//     {                                                                          \
//         return GetLastError();                                                 \
//     }
//
//
// static DWORD
// LoadAndInitializeNtMarta(VOID)
// {
//     /* this code may be executed simultaneously by multiple threads in case they're
//        trying to initialize the interface at the same time, but that's no problem
//        because the pointers returned by GetProcAddress will be the same. However,
//        only one of the threads will change the NtMarta pointer to the NtMartaStatic
//        structure, the others threads will detect that there were other threads
//        initializing the structure faster and will release the reference to the
//        DLL */
//
//     NtMartaStatic.hDllInstance = LoadLibraryW(L"ntmarta.dll");
//     if (NtMartaStatic.hDllInstance == NULL)
//     {
//         return GetLastError();
//     }
//
//     FindNtMartaProc(RewriteGetNamedRights);
//     FindNtMartaProc(RewriteSetNamedRights);
//     FindNtMartaProc(RewriteGetHandleRights);
//     FindNtMartaProc(RewriteSetHandleRights);
//     FindNtMartaProc(RewriteSetEntriesInAcl);
//     FindNtMartaProc(RewriteGetExplicitEntriesFromAcl);
//     FindNtMartaProc(TreeResetNamedSecurityInfo);
//     FindNtMartaProc(GetInheritanceSource);
//     FindNtMartaProc(FreeIndexArray);
//
//     return ERROR_SUCCESS;
// }
//
//
// DWORD
// CheckNtMartaPresent(VOID)
// {
//     DWORD ErrorCode;
//
//     if (InterlockedCompareExchangePointer((PVOID)&NtMarta,
//                                           NULL,
//                                           NULL) == NULL)
//     {
//         /* we're the first one trying to use ntmarta, initialize it and change
//            the pointer after initialization */
//         ErrorCode = LoadAndInitializeNtMarta();
//
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             /* try change the NtMarta pointer */
//             if (InterlockedCompareExchangePointer((PVOID)&NtMarta,
//                                                   &NtMartaStatic,
//                                                   NULL) != NULL)
//             {
//                 /* another thread initialized ntmarta in the meanwhile, release
//                    the reference of the dll loaded. */
//                 FreeLibrary(NtMartaStatic.hDllInstance as *mut core::ffi::c_void);
//             }
//         }
//         else
//         {
//             ERR("Failed to initialize ntmarta.dll! Error: 0x%x\n", ErrorCode);
//         }
//     }
//     else
//     {
//         /* ntmarta was already initialized */
//         ErrorCode = ERROR_SUCCESS;
//     }
//
//     return ErrorCode;
// }
//
//
// VOID
// UnloadNtMarta(VOID)
// {
//     if (InterlockedExchangePointer((PVOID)&NtMarta,
//                                    NULL) != NULL)
//     {
//         FreeLibrary(NtMartaStatic.hDllInstance as *mut core::ffi::c_void);
//     }
// }
//
//
// /******************************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ImpersonateAnonymousToken(IN HANDLE ThreadHandle)
// {
//     NTSTATUS Status;
//
//     Status = NtImpersonateAnonymousToken(ThreadHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ImpersonateLoggedOnUser(HANDLE hToken)
// {
//     SECURITY_QUALITY_OF_SERVICE Qos;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE NewToken;
//     TOKEN_TYPE Type;
//     ULONG ReturnLength;
//     BOOL Duplicated;
//     NTSTATUS Status;
//
//     /* Get the token type */
//     Status = NtQueryInformationToken(hToken,
//                                      TokenType,
//                                      &Type,
//                                      sizeof(TOKEN_TYPE),
//                                      &ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     if (Type == TokenPrimary)
//     {
//         /* Create a duplicate impersonation token */
//         Qos.Length = sizeof(SECURITY_QUALITY_OF_SERVICE);
//         Qos.ImpersonationLevel = SecurityImpersonation;
//         Qos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING;
//         Qos.EffectiveOnly = FALSE;
//
//         ObjectAttributes.Length = sizeof(OBJECT_ATTRIBUTES);
//         ObjectAttributes.RootDirectory = NULL;
//         ObjectAttributes.ObjectName = NULL;
//         ObjectAttributes.Attributes = 0;
//         ObjectAttributes.SecurityDescriptor = NULL;
//         ObjectAttributes.SecurityQualityOfService = &Qos;
//
//         Status = NtDuplicateToken(hToken,
//                                   TOKEN_IMPERSONATE | TOKEN_QUERY,
//                                   &ObjectAttributes,
//                                   FALSE,
//                                   TokenImpersonation,
//                                   &NewToken);
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("NtDuplicateToken failed: Status %08x\n", Status);
//             SetLastError(RtlNtStatusToDosError(Status));
//             return FALSE;
//         }
//
//         Duplicated = TRUE;
//     }
//     else
//     {
//         /* User the original impersonation token */
//         NewToken = hToken;
//         Duplicated = FALSE;
//     }
//
//     /* Impersonate the the current thread */
//     Status = NtSetInformationThread(NtCurrentThread(),
//                                     ThreadImpersonationToken,
//                                     &NewToken,
//                                     sizeof(HANDLE));
//
//     if (Duplicated != FALSE)
//     {
//         NtClose(NewToken);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtSetInformationThread failed: Status %08x\n", Status);
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * GetUserNameA [ADVAPI32.@]
//  *
//  * Get the current user name.
//  *
//  * PARAMS
//  *  lpszName [O]   Destination for the user name.
//  *  lpSize   [I/O] Size of lpszName.
//  *
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// GetUserNameA(LPSTR lpszName,
//              LPDWORD lpSize)
// {
//     UNICODE_STRING NameW;
//     ANSI_STRING NameA;
//     BOOL Ret;
//
//     /* apparently Win doesn't check whether lpSize is valid at all! */
//
//     NameW.MaximumLength = (*lpSize) * sizeof(WCHAR);
//     NameW.Buffer = LocalAlloc(LMEM_FIXED, NameW.MaximumLength);
//     if(NameW.Buffer == NULL)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     NameA.Length = 0;
//     NameA.MaximumLength = ((*lpSize) < 0xFFFF ? (USHORT)(*lpSize) : 0xFFFF);
//     NameA.Buffer = lpszName;
//
//     Ret = GetUserNameW(NameW.Buffer,
//                        lpSize);
//     if(Ret)
//     {
//         NameW.Length = (*lpSize - 1) * sizeof(WCHAR);
//         RtlUnicodeStringToAnsiString(&NameA, &NameW, FALSE);
//
//         *lpSize = NameA.Length + 1;
//     }
//
//     LocalFree(NameW.Buffer);
//
//     return Ret;
// }
//
// /******************************************************************************
//  * GetUserNameW [ADVAPI32.@]
//  *
//  * See GetUserNameA.
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// GetUserNameW(LPWSTR lpszName,
//              LPDWORD lpSize)
// {
//     HANDLE hToken = INVALID_HANDLE_VALUE;
//     DWORD tu_len = 0;
//     char* tu_buf = NULL;
//     TOKEN_USER* token_user = NULL;
//     DWORD an_len = 0;
//     SID_NAME_USE snu = SidTypeUser;
//     WCHAR* domain_name = NULL;
//     DWORD dn_len = 0;
//
//     if (!OpenThreadToken (GetCurrentThread(), TOKEN_QUERY, FALSE, &hToken))
//     {
//         DWORD dwLastError = GetLastError();
//         if (dwLastError != ERROR_NO_TOKEN
//             && dwLastError != ERROR_NO_IMPERSONATION_TOKEN)
//         {
//             /* don't call SetLastError(),
//                as OpenThreadToken() ought to have set one */
//             return FALSE;
//         }
//
//         if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken))
//         {
//             /* don't call SetLastError(),
//                as OpenProcessToken() ought to have set one */
//             return FALSE;
//         }
//     }
//
//     tu_buf = LocalAlloc(LMEM_FIXED, 36);
//     if (!tu_buf)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         CloseHandle(hToken);
//         return FALSE;
//     }
//
//     if (!GetTokenInformation(hToken, TokenUser, tu_buf, 36, &tu_len) || tu_len > 36)
//     {
//         LocalFree(tu_buf);
//         tu_buf = LocalAlloc(LMEM_FIXED, tu_len);
//         if (!tu_buf)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             CloseHandle(hToken);
//             return FALSE;
//         }
//
//         if (!GetTokenInformation(hToken, TokenUser, tu_buf, tu_len, &tu_len))
//         {
//             /* don't call SetLastError(),
//                as GetTokenInformation() ought to have set one */
//             LocalFree(tu_buf);
//             CloseHandle(hToken);
//             return FALSE;
//         }
//     }
//
//     CloseHandle(hToken);
//     token_user = (TOKEN_USER*)tu_buf;
//
//     an_len = *lpSize;
//     dn_len = 32;
//     domain_name = LocalAlloc(LMEM_FIXED, dn_len * sizeof(WCHAR));
//     if (!domain_name)
//     {
//         LocalFree(tu_buf);
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     if (!LookupAccountSidW(NULL, token_user->User.Sid, lpszName, &an_len, domain_name, &dn_len, &snu)
//         || dn_len > 32)
//     {
//         if (dn_len > 32)
//         {
//             LocalFree(domain_name);
//             domain_name = LocalAlloc(LMEM_FIXED, dn_len * sizeof(WCHAR));
//             if (!domain_name)
//             {
//                 LocalFree(tu_buf);
//                 SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                 return FALSE;
//             }
//         }
//
//         an_len = *lpSize;
//         if (!LookupAccountSidW(NULL, token_user->User.Sid, lpszName, &an_len, domain_name, &dn_len, &snu))
//         {
//             /* don't call SetLastError(),
//                as LookupAccountSid() ought to have set one */
//             LocalFree(domain_name);
//             LocalFree(tu_buf);
//             *lpSize = an_len;
//             return FALSE;
//         }
//     }
//
//     LocalFree(domain_name);
//     LocalFree(tu_buf);
//     *lpSize = an_len + 1;
//     return TRUE;
// }
//
//
// /******************************************************************************
//  * LookupAccountSidA [ADVAPI32.@]
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupAccountSidA(LPCSTR lpSystemName,
//                   PSID lpSid,
//                   LPSTR lpName,
//                   LPDWORD cchName,
//                   LPSTR lpReferencedDomainName,
//                   LPDWORD cchReferencedDomainName,
//                   PSID_NAME_USE peUse)
// {
//     UNICODE_STRING NameW, ReferencedDomainNameW, SystemNameW;
//     LPWSTR NameBuffer = NULL;
//     LPWSTR ReferencedDomainNameBuffer = NULL;
//     DWORD dwName, dwReferencedDomainName;
//     BOOL Ret;
//
//     /*
//      * save the buffer sizes the caller passed to us, as they may get modified and
//      * we require the original values when converting back to ansi
//      */
//     dwName = *cchName;
//     dwReferencedDomainName = *cchReferencedDomainName;
//
//     /* allocate buffers for the unicode strings to receive */
//     if (dwName > 0)
//     {
//         NameBuffer = LocalAlloc(LMEM_FIXED, dwName * sizeof(WCHAR));
//         if (NameBuffer == NULL)
//         {
//             SetLastError(ERROR_OUTOFMEMORY);
//             return FALSE;
//         }
//     }
//     else
//         NameBuffer = NULL;
//
//     if (dwReferencedDomainName > 0)
//     {
//         ReferencedDomainNameBuffer = LocalAlloc(LMEM_FIXED, dwReferencedDomainName * sizeof(WCHAR));
//         if (ReferencedDomainNameBuffer == NULL)
//         {
//             if (dwName > 0)
//             {
//                 LocalFree(NameBuffer);
//             }
//
//             SetLastError(ERROR_OUTOFMEMORY);
//             return FALSE;
//         }
//     }
//     else
//         ReferencedDomainNameBuffer = NULL;
//
//
//     /* convert the system name to unicode - if present */
//     if (lpSystemName != NULL)
//     {
//         ANSI_STRING SystemNameA;
//
//         RtlInitAnsiString(&SystemNameA, lpSystemName);
//         RtlAnsiStringToUnicodeString(&SystemNameW, &SystemNameA, TRUE);
//     }
//     else
//         SystemNameW.Buffer = NULL;
//
//     /* it's time to call the unicode version */
//     Ret = LookupAccountSidW(SystemNameW.Buffer,
//                             lpSid,
//                             NameBuffer,
//                             cchName,
//                             ReferencedDomainNameBuffer,
//                             cchReferencedDomainName,
//                             peUse);
//     if (Ret)
//     {
//         /*
//          * convert unicode strings back to ansi, don't forget that we can't convert
//          * more than 0xFFFF (USHORT) characters! Also don't forget to explicitly
//          * terminate the converted string, the Rtl functions don't do that!
//          */
//         if (lpName != NULL)
//         {
//             ANSI_STRING NameA;
//
//             NameA.Length = 0;
//             NameA.MaximumLength = ((dwName <= 0xFFFF) ? (USHORT)dwName : 0xFFFF);
//             NameA.Buffer = lpName;
//
//             RtlInitUnicodeString(&NameW, NameBuffer);
//             RtlUnicodeStringToAnsiString(&NameA, &NameW, FALSE);
//             NameA.Buffer[NameA.Length] = '\0';
//         }
//
//         if (lpReferencedDomainName != NULL)
//         {
//             ANSI_STRING ReferencedDomainNameA;
//
//             ReferencedDomainNameA.Length = 0;
//             ReferencedDomainNameA.MaximumLength = ((dwReferencedDomainName <= 0xFFFF) ?
//                                                    (USHORT)dwReferencedDomainName : 0xFFFF);
//             ReferencedDomainNameA.Buffer = lpReferencedDomainName;
//
//             RtlInitUnicodeString(&ReferencedDomainNameW, ReferencedDomainNameBuffer);
//             RtlUnicodeStringToAnsiString(&ReferencedDomainNameA, &ReferencedDomainNameW, FALSE);
//             ReferencedDomainNameA.Buffer[ReferencedDomainNameA.Length] = '\0';
//         }
//     }
//
//     /* free previously allocated buffers */
//     if (SystemNameW.Buffer != NULL)
//     {
//         RtlFreeUnicodeString(&SystemNameW);
//     }
//
//     if (NameBuffer != NULL)
//     {
//         LocalFree(NameBuffer);
//     }
//
//     if (ReferencedDomainNameBuffer != NULL)
//     {
//         LocalFree(ReferencedDomainNameBuffer);
//     }
//
//     return Ret;
// }
//
//
// /******************************************************************************
//  * LookupAccountSidW [ADVAPI32.@]
//  *
//  * @implemented
//  */
// BOOL WINAPI
// LookupAccountSidW(LPCWSTR pSystemName,
//                   PSID pSid,
//                   LPWSTR pAccountName,
//                   LPDWORD pdwAccountName,
//                   LPWSTR pDomainName,
//                   LPDWORD pdwDomainName,
//                   PSID_NAME_USE peUse)
// {
//     LSA_UNICODE_STRING SystemName;
//     LSA_OBJECT_ATTRIBUTES ObjectAttributes = {0};
//     LSA_HANDLE PolicyHandle = NULL;
//     NTSTATUS Status;
//     PLSA_REFERENCED_DOMAIN_LIST ReferencedDomain = NULL;
//     PLSA_TRANSLATED_NAME TranslatedName = NULL;
//     BOOL ret;
//     DWORD dwAccountName, dwDomainName;
//
//     RtlInitUnicodeString(&SystemName, pSystemName);
//     Status = LsaOpenPolicy(&SystemName, &ObjectAttributes, POLICY_LOOKUP_NAMES, &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     Status = LsaLookupSids(PolicyHandle, 1, &pSid, &ReferencedDomain, &TranslatedName);
//
//     LsaClose(PolicyHandle);
//
//     if (!NT_SUCCESS(Status) || Status == STATUS_SOME_NOT_MAPPED)
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         ret = FALSE;
//     }
//     else
//     {
//         ret = TRUE;
//
//         dwAccountName = TranslatedName->Name.Length / sizeof(WCHAR);
//         if (ReferencedDomain && ReferencedDomain->Entries > 0)
//             dwDomainName = ReferencedDomain->Domains[0].Name.Length / sizeof(WCHAR);
//         else
//             dwDomainName = 0;
//
//         if (*pdwAccountName <= dwAccountName || *pdwDomainName <= dwDomainName)
//         {
//             /* One or two buffers are insufficient, add up a char for NULL termination */
//             *pdwAccountName = dwAccountName + 1;
//             *pdwDomainName = dwDomainName + 1;
//             ret = FALSE;
//         }
//         else
//         {
//             /* Lengths are sufficient, copy the data */
//             if (dwAccountName)
//                 RtlCopyMemory(pAccountName, TranslatedName->Name.Buffer, dwAccountName * sizeof(WCHAR));
//             pAccountName[dwAccountName] = L'\0';
//
//             if (dwDomainName)
//                 RtlCopyMemory(pDomainName, ReferencedDomain->Domains[0].Name.Buffer, dwDomainName * sizeof(WCHAR));
//             pDomainName[dwDomainName] = L'\0';
//
//             *pdwAccountName = dwAccountName;
//             *pdwDomainName = dwDomainName;
//
//             if (peUse)
//                 *peUse = TranslatedName->Use;
//         }
//
//         if (!ret)
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//     }
//
//     if (ReferencedDomain)
//         LsaFreeMemory(ReferencedDomain);
//
//     if (TranslatedName)
//         LsaFreeMemory(TranslatedName);
//
//     return ret;
// }
//
// /******************************************************************************
//  * LookupAccountNameW [ADVAPI32.@]
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupAccountNameW(LPCWSTR lpSystemName,
//                    LPCWSTR lpAccountName,
//                    PSID Sid,
//                    LPDWORD cbSid,
//                    LPWSTR ReferencedDomainName,
//                    LPDWORD cchReferencedDomainName,
//                    PSID_NAME_USE peUse)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes = {0};
//     UNICODE_STRING SystemName;
//     UNICODE_STRING AccountName;
//     LSA_HANDLE PolicyHandle = NULL;
//     PLSA_REFERENCED_DOMAIN_LIST ReferencedDomains = NULL;
//     PLSA_TRANSLATED_SID TranslatedSid = NULL;
//     PSID pDomainSid;
//     DWORD dwDomainNameLength;
//     DWORD dwSidLength;
//     UCHAR nSubAuthorities;
//     BOOL bResult;
//     NTSTATUS Status;
//
//     TRACE("%s %s %p %p %p %p %p\n", lpSystemName, lpAccountName,
//           Sid, cbSid, ReferencedDomainName, cchReferencedDomainName, peUse);
//
//     RtlInitUnicodeString(&SystemName,
//                          lpSystemName);
//
//     Status = LsaOpenPolicy(lpSystemName ? &SystemName : NULL,
//                            &ObjectAttributes,
//                            POLICY_LOOKUP_NAMES,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&AccountName,
//                          lpAccountName);
//
//     Status = LsaLookupNames(PolicyHandle,
//                             1,
//                             &AccountName,
//                             &ReferencedDomains,
//                             &TranslatedSid);
//
//     LsaClose(PolicyHandle);
//
//     if (!NT_SUCCESS(Status) || Status == STATUS_SOME_NOT_MAPPED)
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         bResult = FALSE;
//     }
//     else
//     {
//         pDomainSid = ReferencedDomains->Domains[TranslatedSid->DomainIndex].Sid;
//         nSubAuthorities = *GetSidSubAuthorityCount(pDomainSid);
//         dwSidLength = GetSidLengthRequired(nSubAuthorities + 1);
//
//         dwDomainNameLength = ReferencedDomains->Domains->Name.Length / sizeof(WCHAR);
//
//         if (*cbSid < dwSidLength ||
//             *cchReferencedDomainName < dwDomainNameLength + 1)
//         {
//             *cbSid = dwSidLength;
//             *cchReferencedDomainName = dwDomainNameLength + 1;
//
//             bResult = FALSE;
//         }
//         else
//         {
//             CopySid(*cbSid, Sid, pDomainSid);
//             *GetSidSubAuthorityCount(Sid) = nSubAuthorities + 1;
//             *GetSidSubAuthority(Sid, (DWORD)nSubAuthorities) = TranslatedSid->RelativeId;
//
//             RtlCopyMemory(ReferencedDomainName, ReferencedDomains->Domains->Name.Buffer, dwDomainNameLength * sizeof(WCHAR));
//             ReferencedDomainName[dwDomainNameLength] = L'\0';
//
//             *cchReferencedDomainName = dwDomainNameLength;
//
//             *peUse = TranslatedSid->Use;
//
//             bResult = TRUE;
//         }
//
//         if (bResult == FALSE)
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//     }
//
//     if (ReferencedDomains != NULL)
//         LsaFreeMemory(ReferencedDomains);
//
//     if (TranslatedSid != NULL)
//         LsaFreeMemory(TranslatedSid);
//
//     return bResult;
// }
//
//
// /**********************************************************************
//  * LookupPrivilegeValueA				EXPORTED
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupPrivilegeValueA(LPCSTR lpSystemName,
//                       LPCSTR lpName,
//                       PLUID lpLuid)
// {
//     UNICODE_STRING SystemName;
//     UNICODE_STRING Name;
//     BOOL Result;
//
//     /* Remote system? */
//     if (lpSystemName != NULL)
//     {
//         RtlCreateUnicodeStringFromAsciiz(&SystemName,
//                                          (LPSTR)lpSystemName);
//     }
//     else
//         SystemName.Buffer = NULL;
//
//     /* Check the privilege name is not NULL */
//     if (lpName == NULL)
//     {
//         SetLastError(ERROR_NO_SUCH_PRIVILEGE);
//         return FALSE;
//     }
//
//     RtlCreateUnicodeStringFromAsciiz(&Name,
//                                      (LPSTR)lpName);
//
//     Result = LookupPrivilegeValueW(SystemName.Buffer,
//                                    Name.Buffer,
//                                    lpLuid);
//
//     RtlFreeUnicodeString(&Name);
//
//     /* Remote system? */
//     if (SystemName.Buffer != NULL)
//     {
//         RtlFreeUnicodeString(&SystemName);
//     }
//
//     return Result;
// }
//
//
// /**********************************************************************
//  * LookupPrivilegeValueW
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupPrivilegeValueW(LPCWSTR lpSystemName,
//                       LPCWSTR lpPrivilegeName,
//                       PLUID lpLuid)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes = {0};
//     UNICODE_STRING SystemName;
//     UNICODE_STRING PrivilegeName;
//     LSA_HANDLE PolicyHandle = NULL;
//     NTSTATUS Status;
//
//     TRACE("%S,%S,%p\n", lpSystemName, lpPrivilegeName, lpLuid);
//
//     RtlInitUnicodeString(&SystemName,
//                          lpSystemName);
//
//     Status = LsaOpenPolicy(lpSystemName ? &SystemName : NULL,
//                            &ObjectAttributes,
//                            POLICY_LOOKUP_NAMES,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     RtlInitUnicodeString(&PrivilegeName,
//                          lpPrivilegeName);
//
//     Status = LsaLookupPrivilegeValue(PolicyHandle,
//                                      &PrivilegeName,
//                                      lpLuid);
//
//     LsaClose(PolicyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**********************************************************************
//  * LookupPrivilegeNameW				EXPORTED
//  *
//  * @implemented
//  */
// BOOL
// WINAPI
// LookupPrivilegeNameW(LPCWSTR lpSystemName,
//                      PLUID lpLuid,
//                      LPWSTR lpName,
//                      LPDWORD cchName)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes = {0};
//     UNICODE_STRING SystemName;
//     PUNICODE_STRING PrivilegeName = NULL;
//     LSA_HANDLE PolicyHandle = NULL;
//     NTSTATUS Status;
//
//     TRACE("%S,%p,%p,%p\n", lpSystemName, lpLuid, lpName, cchName);
//
//     RtlInitUnicodeString(&SystemName,
//                          lpSystemName);
//
//     Status = LsaOpenPolicy(lpSystemName ? &SystemName : NULL,
//                            &ObjectAttributes,
//                            POLICY_LOOKUP_NAMES,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     Status = LsaLookupPrivilegeName(PolicyHandle,
//                                     lpLuid,
//                                     &PrivilegeName);
//     if (NT_SUCCESS(Status))
//     {
//         if (PrivilegeName->Length + sizeof(WCHAR) > *cchName * sizeof(WCHAR))
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//
//             *cchName = (PrivilegeName->Length + sizeof(WCHAR)) / sizeof(WCHAR);
//         }
//         else
//         {
//             RtlMoveMemory(lpName,
//                           PrivilegeName->Buffer,
//                           PrivilegeName->Length);
//             lpName[PrivilegeName->Length / sizeof(WCHAR)] = 0;
//
//             *cchName = PrivilegeName->Length / sizeof(WCHAR);
//         }
//
//         LsaFreeMemory(PrivilegeName->Buffer);
//         LsaFreeMemory(PrivilegeName);
//     }
//
//     LsaClose(PolicyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /**********************************************************************
//  * LookupPrivilegeDisplayNameW			EXPORTED
//  *
//  * @unimplemented
//  */
// BOOL
// WINAPI
// LookupPrivilegeDisplayNameW(LPCWSTR lpSystemName,
//                             LPCWSTR lpName,
//                             LPWSTR lpDisplayName,
//                             LPDWORD cchDisplayName,
//                             LPDWORD lpLanguageId)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes = {0};
//     UNICODE_STRING SystemName, Name;
//     PUNICODE_STRING DisplayName;
//     LSA_HANDLE PolicyHandle = NULL;
//     USHORT LanguageId;
//     NTSTATUS Status;
//
//     TRACE("%S,%S,%p,%p,%p\n", lpSystemName, lpName, lpDisplayName, cchDisplayName, lpLanguageId);
//
//     RtlInitUnicodeString(&SystemName, lpSystemName);
//     RtlInitUnicodeString(&Name, lpName);
//
//     Status = LsaOpenPolicy(lpSystemName ? &SystemName : NULL,
//                            &ObjectAttributes,
//                            POLICY_LOOKUP_NAMES,
//                            &PolicyHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     Status = LsaLookupPrivilegeDisplayName(PolicyHandle, &Name, &DisplayName, &LanguageId);
//     if (NT_SUCCESS(Status))
//     {
//         *lpLanguageId = LanguageId;
//         if (DisplayName->Length + sizeof(WCHAR) > *cchDisplayName * sizeof(WCHAR))
//         {
//             Status = STATUS_BUFFER_TOO_SMALL;
//
//             *cchDisplayName = (DisplayName->Length + sizeof(WCHAR)) / sizeof(WCHAR);
//         }
//         else
//         {
//             RtlMoveMemory(lpDisplayName,
//                           DisplayName->Buffer,
//                           DisplayName->Length);
//             lpDisplayName[DisplayName->Length / sizeof(WCHAR)] = 0;
//
//             *cchDisplayName = DisplayName->Length / sizeof(WCHAR);
//         }
//
//         LsaFreeMemory(DisplayName->Buffer);
//         LsaFreeMemory(DisplayName);
//     }
//
//     LsaClose(PolicyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(LsaNtStatusToWinError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static DWORD
// pGetSecurityInfoCheck(SECURITY_INFORMATION SecurityInfo,
//                       PSID *ppsidOwner,
//                       PSID *ppsidGroup,
//                       PACL *ppDacl,
//                       PACL *ppSacl,
//                       PSECURITY_DESCRIPTOR* ppSecurityDescriptor)
// {
//     if ((SecurityInfo & (OWNER_SECURITY_INFORMATION |
//                          GROUP_SECURITY_INFORMATION |
//                          DACL_SECURITY_INFORMATION |
//                          SACL_SECURITY_INFORMATION)) &&
//         ppSecurityDescriptor == NULL)
//     {
//         /* if one of the SIDs or ACLs are present, the security descriptor
//            most not be NULL */
//         return ERROR_INVALID_PARAMETER;
//     }
//     else
//     {
//         /* reset the pointers unless they're ignored */
//         if ((SecurityInfo & OWNER_SECURITY_INFORMATION) &&
//             ppsidOwner != NULL)
//         {
//             *ppsidOwner = NULL;
//         }
//         if ((SecurityInfo & GROUP_SECURITY_INFORMATION) &&
//             ppsidGroup != NULL)
//         {
//             *ppsidGroup = NULL;
//         }
//         if ((SecurityInfo & DACL_SECURITY_INFORMATION) &&
//             ppDacl != NULL)
//         {
//             *ppDacl = NULL;
//         }
//         if ((SecurityInfo & SACL_SECURITY_INFORMATION) &&
//             ppSacl != NULL)
//         {
//             *ppSacl = NULL;
//         }
//
//         if (SecurityInfo & (OWNER_SECURITY_INFORMATION |
//                             GROUP_SECURITY_INFORMATION |
//                             DACL_SECURITY_INFORMATION |
//                             SACL_SECURITY_INFORMATION))
//         {
//             *ppSecurityDescriptor = NULL;
//         }
//
//         return ERROR_SUCCESS;
//     }
// }
//
//
// static DWORD
// pSetSecurityInfoCheck(PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                       SECURITY_INFORMATION SecurityInfo,
//                       PSID psidOwner,
//                       PSID psidGroup,
//                       PACL pDacl,
//                       PACL pSacl)
// {
//     /* initialize a security descriptor on the stack */
//     if (!InitializeSecurityDescriptor(pSecurityDescriptor,
//                                       SECURITY_DESCRIPTOR_REVISION))
//     {
//         return GetLastError();
//     }
//
//     if (SecurityInfo & OWNER_SECURITY_INFORMATION)
//     {
//         if (RtlValidSid(psidOwner))
//         {
//             if (!SetSecurityDescriptorOwner(pSecurityDescriptor,
//                                             psidOwner,
//                                             FALSE))
//             {
//                 return GetLastError();
//             }
//         }
//         else
//         {
//             return ERROR_INVALID_PARAMETER;
//         }
//     }
//
//     if (SecurityInfo & GROUP_SECURITY_INFORMATION)
//     {
//         if (RtlValidSid(psidGroup))
//         {
//             if (!SetSecurityDescriptorGroup(pSecurityDescriptor,
//                                             psidGroup,
//                                             FALSE))
//             {
//                 return GetLastError();
//             }
//         }
//         else
//         {
//             return ERROR_INVALID_PARAMETER;
//         }
//     }
//
//     if (SecurityInfo & DACL_SECURITY_INFORMATION)
//     {
//         if (pDacl != NULL)
//         {
//             if (SetSecurityDescriptorDacl(pSecurityDescriptor,
//                                           TRUE,
//                                           pDacl,
//                                           FALSE))
//             {
//                 /* check if the DACL needs to be protected from being
//                    modified by inheritable ACEs */
//                 if (SecurityInfo & PROTECTED_DACL_SECURITY_INFORMATION)
//                 {
//                     goto ProtectDacl;
//                 }
//             }
//             else
//             {
//                 return GetLastError();
//             }
//         }
//         else
//         {
// ProtectDacl:
//             /* protect the DACL from being modified by inheritable ACEs */
//             if (!SetSecurityDescriptorControl(pSecurityDescriptor,
//                                               SE_DACL_PROTECTED,
//                                               SE_DACL_PROTECTED))
//             {
//                 return GetLastError();
//             }
//         }
//     }
//
//     if (SecurityInfo & SACL_SECURITY_INFORMATION)
//     {
//         if (pSacl != NULL)
//         {
//             if (SetSecurityDescriptorSacl(pSecurityDescriptor,
//                                           TRUE,
//                                           pSacl,
//                                           FALSE))
//             {
//                 /* check if the SACL needs to be protected from being
//                    modified by inheritable ACEs */
//                 if (SecurityInfo & PROTECTED_SACL_SECURITY_INFORMATION)
//                 {
//                     goto ProtectSacl;
//                 }
//             }
//             else
//             {
//                 return GetLastError();
//             }
//         }
//         else
//         {
// ProtectSacl:
//             /* protect the SACL from being modified by inheritable ACEs */
//             if (!SetSecurityDescriptorControl(pSecurityDescriptor,
//                                               SE_SACL_PROTECTED,
//                                               SE_SACL_PROTECTED))
//             {
//                 return GetLastError();
//             }
//         }
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /**********************************************************************
//  * GetNamedSecurityInfoW			EXPORTED
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// GetNamedSecurityInfoW(LPWSTR pObjectName,
//                       SE_OBJECT_TYPE ObjectType,
//                       SECURITY_INFORMATION SecurityInfo,
//                       PSID *ppsidOwner,
//                       PSID *ppsidGroup,
//                       PACL *ppDacl,
//                       PACL *ppSacl,
//                       PSECURITY_DESCRIPTOR *ppSecurityDescriptor)
// {
//     DWORD ErrorCode;
//
//     if (pObjectName != NULL)
//     {
//         ErrorCode = CheckNtMartaPresent();
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             ErrorCode = pGetSecurityInfoCheck(SecurityInfo,
//                                               ppsidOwner,
//                                               ppsidGroup,
//                                               ppDacl,
//                                               ppSacl,
//                                               ppSecurityDescriptor);
//
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* call the MARTA provider */
//                 ErrorCode = AccRewriteGetNamedRights(pObjectName,
//                                                      ObjectType,
//                                                      SecurityInfo,
//                                                      ppsidOwner,
//                                                      ppsidGroup,
//                                                      ppDacl,
//                                                      ppSacl,
//                                                      ppSecurityDescriptor);
//             }
//         }
//     }
//     else
//         ErrorCode = ERROR_INVALID_PARAMETER;
//
//     return ErrorCode;
// }
//
// /**********************************************************************
//  * SetNamedSecurityInfoW			EXPORTED
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// SetNamedSecurityInfoW(LPWSTR pObjectName,
//                       SE_OBJECT_TYPE ObjectType,
//                       SECURITY_INFORMATION SecurityInfo,
//                       PSID psidOwner,
//                       PSID psidGroup,
//                       PACL pDacl,
//                       PACL pSacl)
// {
//     DWORD ErrorCode;
//
//     if (pObjectName != NULL)
//     {
//         ErrorCode = CheckNtMartaPresent();
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             SECURITY_DESCRIPTOR SecurityDescriptor;
//
//             ErrorCode = pSetSecurityInfoCheck(&SecurityDescriptor,
//                                               SecurityInfo,
//                                               psidOwner,
//                                               psidGroup,
//                                               pDacl,
//                                               pSacl);
//
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* call the MARTA provider */
//                 ErrorCode = AccRewriteSetNamedRights(pObjectName,
//                                                      ObjectType,
//                                                      SecurityInfo,
//                                                      &SecurityDescriptor);
//             }
//         }
//     }
//     else
//         ErrorCode = ERROR_INVALID_PARAMETER;
//
//     return ErrorCode;
// }
//
// /**********************************************************************
//  * GetSecurityInfo				EXPORTED
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// GetSecurityInfo(HANDLE handle,
//                 SE_OBJECT_TYPE ObjectType,
//                 SECURITY_INFORMATION SecurityInfo,
//                 PSID *ppsidOwner,
//                 PSID *ppsidGroup,
//                 PACL *ppDacl,
//                 PACL *ppSacl,
//                 PSECURITY_DESCRIPTOR *ppSecurityDescriptor)
// {
//     DWORD ErrorCode;
//
//     if (handle != NULL)
//     {
//         ErrorCode = CheckNtMartaPresent();
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             ErrorCode = pGetSecurityInfoCheck(SecurityInfo,
//                                               ppsidOwner,
//                                               ppsidGroup,
//                                               ppDacl,
//                                               ppSacl,
//                                               ppSecurityDescriptor);
//
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* call the MARTA provider */
//                 ErrorCode = AccRewriteGetHandleRights(handle,
//                                                       ObjectType,
//                                                       SecurityInfo,
//                                                       ppsidOwner,
//                                                       ppsidGroup,
//                                                       ppDacl,
//                                                       ppSacl,
//                                                       ppSecurityDescriptor);
//             }
//         }
//     }
//     else
//         ErrorCode = ERROR_INVALID_HANDLE;
//
//     return ErrorCode;
// }
//
//
// /**********************************************************************
//  * SetSecurityInfo				EXPORTED
//  *
//  * @implemented
//  */
// DWORD
// WINAPI
// SetSecurityInfo(HANDLE handle,
//                 SE_OBJECT_TYPE ObjectType,
//                 SECURITY_INFORMATION SecurityInfo,
//                 PSID psidOwner,
//                 PSID psidGroup,
//                 PACL pDacl,
//                 PACL pSacl)
// {
//     DWORD ErrorCode;
//
//     if (handle != NULL)
//     {
//         ErrorCode = CheckNtMartaPresent();
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             SECURITY_DESCRIPTOR SecurityDescriptor;
//
//             ErrorCode = pSetSecurityInfoCheck(&SecurityDescriptor,
//                                               SecurityInfo,
//                                               psidOwner,
//                                               psidGroup,
//                                               pDacl,
//                                               pSacl);
//
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* call the MARTA provider */
//                 ErrorCode = AccRewriteSetHandleRights(handle,
//                                                       ObjectType,
//                                                       SecurityInfo,
//                                                       &SecurityDescriptor);
//             }
//         }
//     }
//     else
//         ErrorCode = ERROR_INVALID_HANDLE;
//
//     return ErrorCode;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CreatePrivateObjectSecurity(PSECURITY_DESCRIPTOR ParentDescriptor,
//                             PSECURITY_DESCRIPTOR CreatorDescriptor,
//                             PSECURITY_DESCRIPTOR *NewDescriptor,
//                             BOOL IsDirectoryObject,
//                             HANDLE Token,
//                             PGENERIC_MAPPING GenericMapping)
// {
//     NTSTATUS Status;
//
//     Status = RtlNewSecurityObject(ParentDescriptor,
//                                   CreatorDescriptor,
//                                   NewDescriptor,
//                                   IsDirectoryObject,
//                                   Token,
//                                   GenericMapping);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// CreatePrivateObjectSecurityEx(PSECURITY_DESCRIPTOR ParentDescriptor,
//                               PSECURITY_DESCRIPTOR CreatorDescriptor,
//                               PSECURITY_DESCRIPTOR* NewDescriptor,
//                               GUID* ObjectType,
//                               BOOL IsContainerObject,
//                               ULONG AutoInheritFlags,
//                               HANDLE Token,
//                               PGENERIC_MAPPING GenericMapping)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// CreatePrivateObjectSecurityWithMultipleInheritance(PSECURITY_DESCRIPTOR ParentDescriptor,
//                                                    PSECURITY_DESCRIPTOR CreatorDescriptor,
//                                                    PSECURITY_DESCRIPTOR* NewDescriptor,
//                                                    GUID** ObjectTypes,
//                                                    ULONG GuidCount,
//                                                    BOOL IsContainerObject,
//                                                    ULONG AutoInheritFlags,
//                                                    HANDLE Token,
//                                                    PGENERIC_MAPPING GenericMapping)
// {
//     FIXME("%s() semi-stub\n", __FUNCTION__);
//     return CreatePrivateObjectSecurity(ParentDescriptor, CreatorDescriptor, NewDescriptor, IsContainerObject, Token, GenericMapping);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DestroyPrivateObjectSecurity(PSECURITY_DESCRIPTOR *ObjectDescriptor)
// {
//     NTSTATUS Status;
//
//     Status = RtlDeleteSecurityObject(ObjectDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetPrivateObjectSecurity(IN PSECURITY_DESCRIPTOR ObjectDescriptor,
//                          IN SECURITY_INFORMATION SecurityInformation,
//                          OUT PSECURITY_DESCRIPTOR ResultantDescriptor OPTIONAL,
//                          IN DWORD DescriptorLength,
//                          OUT PDWORD ReturnLength)
// {
//     NTSTATUS Status;
//
//     /* Call RTL */
//     Status = RtlQuerySecurityObject(ObjectDescriptor,
//                                     SecurityInformation,
//                                     ResultantDescriptor,
//                                     DescriptorLength,
//                                     ReturnLength);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetPrivateObjectSecurity(SECURITY_INFORMATION SecurityInformation,
//                          PSECURITY_DESCRIPTOR ModificationDescriptor,
//                          PSECURITY_DESCRIPTOR *ObjectsSecurityDescriptor,
//                          PGENERIC_MAPPING GenericMapping,
//                          HANDLE Token)
// {
//     NTSTATUS Status;
//
//     Status = RtlSetSecurityObject(SecurityInformation,
//                                   ModificationDescriptor,
//                                   ObjectsSecurityDescriptor,
//                                   GenericMapping,
//                                   Token);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// TreeResetNamedSecurityInfoW(LPWSTR pObjectName,
//                             SE_OBJECT_TYPE ObjectType,
//                             SECURITY_INFORMATION SecurityInfo,
//                             PSID pOwner,
//                             PSID pGroup,
//                             PACL pDacl,
//                             PACL pSacl,
//                             BOOL KeepExplicit,
//                             FN_PROGRESSW fnProgress,
//                             PROG_INVOKE_SETTING ProgressInvokeSetting,
//                             PVOID Args)
// {
//     DWORD ErrorCode;
//
//     if (pObjectName != NULL)
//     {
//         ErrorCode = CheckNtMartaPresent();
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             switch (ObjectType)
//             {
//                 case SE_FILE_OBJECT:
//                 case SE_REGISTRY_KEY:
//                 {
//                     /* check the SecurityInfo flags for sanity (both, the protected
//                        and unprotected dacl/sacl flag must not be passed together) */
//                     if (((SecurityInfo & DACL_SECURITY_INFORMATION) &&
//                          (SecurityInfo & (PROTECTED_DACL_SECURITY_INFORMATION | UNPROTECTED_DACL_SECURITY_INFORMATION)) ==
//                              (PROTECTED_DACL_SECURITY_INFORMATION | UNPROTECTED_DACL_SECURITY_INFORMATION))
//
//                         ||
//
//                         ((SecurityInfo & SACL_SECURITY_INFORMATION) &&
//                          (SecurityInfo & (PROTECTED_SACL_SECURITY_INFORMATION | UNPROTECTED_SACL_SECURITY_INFORMATION)) ==
//                              (PROTECTED_SACL_SECURITY_INFORMATION | UNPROTECTED_SACL_SECURITY_INFORMATION)))
//                     {
//                         ErrorCode = ERROR_INVALID_PARAMETER;
//                         break;
//                     }
//
//                     /* call the MARTA provider */
//                     ErrorCode = AccTreeResetNamedSecurityInfo(pObjectName,
//                                                               ObjectType,
//                                                               SecurityInfo,
//                                                               pOwner,
//                                                               pGroup,
//                                                               pDacl,
//                                                               pSacl,
//                                                               KeepExplicit,
//                                                               fnProgress,
//                                                               ProgressInvokeSetting,
//                                                               Args);
//                     break;
//                 }
//
//                 default:
//                     /* object type not supported */
//                     ErrorCode = ERROR_INVALID_PARAMETER;
//                     break;
//             }
//         }
//     }
//     else
//         ErrorCode = ERROR_INVALID_PARAMETER;
//
//     return ErrorCode;
// }
//
// #ifdef HAS_FN_PROGRESSW
//
// typedef struct _INTERNAL_FNPROGRESSW_DATA
// {
//     FN_PROGRESSA fnProgress;
//     PVOID Args;
// } INTERNAL_FNPROGRESSW_DATA, *PINTERNAL_FNPROGRESSW_DATA;
//
// static VOID WINAPI
// InternalfnProgressW(LPWSTR pObjectName,
//                     DWORD Status,
//                     PPROG_INVOKE_SETTING pInvokeSetting,
//                     PVOID Args,
//                     BOOL SecuritySet)
// {
//     PINTERNAL_FNPROGRESSW_DATA pifnProgressData = (PINTERNAL_FNPROGRESSW_DATA)Args;
//     INT ObjectNameSize;
//     LPSTR pObjectNameA;
//
//     ObjectNameSize = WideCharToMultiByte(CP_ACP,
//                                          0,
//                                          pObjectName,
//                                          -1,
//                                          NULL,
//                                          0,
//                                          NULL,
//                                          NULL);
//
//     if (ObjectNameSize > 0)
//     {
//         pObjectNameA = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        0,
//                                        ObjectNameSize);
//         if (pObjectNameA != NULL)
//         {
//             pObjectNameA[0] = '\0';
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 pObjectName,
//                                 -1,
//                                 pObjectNameA,
//                                 ObjectNameSize,
//                                 NULL,
//                                 NULL);
//
//             pifnProgressData->fnProgress((LPWSTR)pObjectNameA, /* FIXME: wrong cast!! */
//                                          Status,
//                                          pInvokeSetting,
//                                          pifnProgressData->Args,
//                                          SecuritySet);
//
//             RtlFreeHeap(RtlGetProcessHeap(),
//                         0,
//                         pObjectNameA);
//         }
//     }
// }
// #endif
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// TreeResetNamedSecurityInfoA(LPSTR pObjectName,
//                             SE_OBJECT_TYPE ObjectType,
//                             SECURITY_INFORMATION SecurityInfo,
//                             PSID pOwner,
//                             PSID pGroup,
//                             PACL pDacl,
//                             PACL pSacl,
//                             BOOL KeepExplicit,
//                             FN_PROGRESSA fnProgress,
//                             PROG_INVOKE_SETTING ProgressInvokeSetting,
//                             PVOID Args)
// {
// #ifndef HAS_FN_PROGRESSW
//     /* That's all this function does, at least up to w2k3... Even MS was too
//        lazy to implement it... */
//     return ERROR_CALL_NOT_IMPLEMENTED;
// #else
//     INTERNAL_FNPROGRESSW_DATA ifnProgressData;
//     UNICODE_STRING ObjectName;
//     DWORD Ret;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ObjectName, pObjectName))
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     ifnProgressData.fnProgress = fnProgress;
//     ifnProgressData.Args = Args;
//
//     Ret = TreeResetNamedSecurityInfoW(ObjectName.Buffer,
//                                       ObjectType,
//                                       SecurityInfo,
//                                       pOwner,
//                                       pGroup,
//                                       pDacl,
//                                       pSacl,
//                                       KeepExplicit,
//                                       (fnProgress != NULL ? InternalfnProgressW : NULL),
//                                       ProgressInvokeSetting,
//                                       &ifnProgressData);
//
//     RtlFreeUnicodeString(&ObjectName);
//
//     return Ret;
// #endif
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
