use crate::prelude::*;

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


// ---- file-scope preamble (translated) ----
pub static mut NtSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};

pub static mut ServerMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: SAM_SERVER_READ,
    GenericWrite: SAM_SERVER_WRITE,
    GenericExecute: SAM_SERVER_EXECUTE,
    GenericAll: SAM_SERVER_ALL_ACCESS,
};

pub static mut DomainMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: DOMAIN_READ,
    GenericWrite: DOMAIN_WRITE,
    GenericExecute: DOMAIN_EXECUTE,
    GenericAll: DOMAIN_ALL_ACCESS,
};

pub static mut AliasMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: ALIAS_READ,
    GenericWrite: ALIAS_WRITE,
    GenericExecute: ALIAS_EXECUTE,
    GenericAll: ALIAS_ALL_ACCESS,
};

pub static mut GroupMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: GROUP_READ,
    GenericWrite: GROUP_WRITE,
    GenericExecute: GROUP_EXECUTE,
    GenericAll: GROUP_ALL_ACCESS,
};

pub static mut UserMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: USER_READ,
    GenericWrite: USER_WRITE,
    GenericExecute: USER_EXECUTE,
    GenericAll: USER_ALL_ACCESS,
};

pub static mut pServerMapping: PGENERIC_MAPPING = unsafe { core::ptr::addr_of_mut!(ServerMapping) };

unsafe fn SampAddRelativeTimeToTime(AbsoluteTime: LARGE_INTEGER, RelativeTime: LARGE_INTEGER) -> LARGE_INTEGER {
    let mut NewTime: LARGE_INTEGER = core::mem::zeroed();

    NewTime.QuadPart = AbsoluteTime.QuadPart - RelativeTime.QuadPart;

    if NewTime.QuadPart < 0 {
        NewTime.QuadPart = 0;
    }

    NewTime
}
#[no_mangle]
pub unsafe extern "C" fn SampStartRpcServer() {
    let mut Status: RPC_STATUS;

    Status = RpcServerUseProtseqEpW(
        w!("ncacn_np").as_ptr() as LPWSTR,
        RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
        w!("\\pipe\\samr").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
    );
    if Status != RPC_S_OK {
        return;
    }

    Status = RpcServerRegisterIf(samr_v1_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut());
    if Status != RPC_S_OK {
        return;
    }

    Status = RpcServerListen(1, 20, 1);
    if Status != RPC_S_OK {
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn midl_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[no_mangle]
pub unsafe extern "C" fn midl_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
// The MIDL runtime/stub references the UPPERCASE names (C `#define
// MIDL_user_allocate midl_user_allocate` in rpcndr.h; no preprocessor in the
// Rust sam-stub). __RPC_USER = stdcall on i386 → extern "system" (_MIDL_user_*@N).
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
// __RPC_USER (= __stdcall on i386) context-handle rundown → extern "system".
#[no_mangle]
pub unsafe extern "system" fn SAMPR_HANDLE_rundown(hHandle: SAMPR_HANDLE) {
    let _ = hHandle;
}

#[no_mangle]
pub unsafe extern "system" fn SamrConnect(
    ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    let mut InRevisionInfo: SAMPR_REVISION_INFO = core::mem::zeroed();
    let mut OutRevisionInfo: SAMPR_REVISION_INFO = core::mem::zeroed();
    let mut OutVersion: ULONG = 0;

    TRACE!(
        b"SamrConnect(%p %p %lx)\n\0".as_ptr() as *const core::ffi::c_char,
        ServerName,
        ServerHandle,
        DesiredAccess,
    );

    InRevisionInfo.V1.Revision = 0;
    InRevisionInfo.V1.SupportedFeatures = 0;

    SamrConnect5(
        ServerName,
        DesiredAccess,
        1,
        &mut InRevisionInfo,
        &mut OutVersion,
        &mut OutRevisionInfo,
        ServerHandle,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SamrCloseHandle(SamHandle: *mut SAMPR_HANDLE) -> NTSTATUS {
    let mut DbObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    Status = SampValidateDbObject(
        *SamHandle,
        SamDbIgnoreObject,
        0,
        core::ptr::addr_of_mut!(DbObject),
    );
    if Status == STATUS_SUCCESS {
        Status = SampCloseDbObject(DbObject);
        *SamHandle = core::ptr::null_mut();
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetSecurityObject(
    ObjectHandle: SAMPR_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: PSAMPR_SR_SECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut DbObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK = 0;
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut RelativeSdSize: ULONG = 0;
    let TokenHandle: HANDLE = core::ptr::null_mut();
    let Mapping: PGENERIC_MAPPING;
    let mut Status: NTSTATUS;

    if SecurityDescriptor.is_null()
        || (*SecurityDescriptor).SecurityDescriptor.is_null()
        || RtlValidSecurityDescriptor(
            (*SecurityDescriptor).SecurityDescriptor as PSECURITY_DESCRIPTOR,
        ) == 0
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    if SecurityInformation == 0
        || (SecurityInformation
            & !(OWNER_SECURITY_INFORMATION
                | GROUP_SECURITY_INFORMATION
                | DACL_SECURITY_INFORMATION
                | SACL_SECURITY_INFORMATION))
            != 0
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }

    if SecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= WRITE_DAC;
    }

    if SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION) != 0 {
        DesiredAccess |= WRITE_OWNER;
    }

    if (SecurityInformation & OWNER_SECURITY_INFORMATION != 0)
        && (*(SecurityDescriptor as PISECURITY_DESCRIPTOR)).Owner.is_null()
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    if (SecurityInformation & GROUP_SECURITY_INFORMATION != 0)
        && (*(SecurityDescriptor as PISECURITY_DESCRIPTOR)).Group.is_null()
    {
        return ERROR_INVALID_PARAMETER as NTSTATUS;
    }

    Status = SampValidateDbObject(
        ObjectHandle,
        SamDbIgnoreObject,
        DesiredAccess,
        &mut DbObject,
    );
    if !NT_SUCCESS(Status) {
        if !TokenHandle.is_null() {
            NtClose(TokenHandle);
        }
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
        return Status;
    }

    match (*DbObject).ObjectType {
        SamDbServerObject => Mapping = &raw const ServerMapping as *const _ as PGENERIC_MAPPING,
        SamDbDomainObject => Mapping = &raw const DomainMapping as *const _ as PGENERIC_MAPPING,
        SamDbAliasObject => Mapping = &raw const AliasMapping as *const _ as PGENERIC_MAPPING,
        SamDbGroupObject => Mapping = &raw const GroupMapping as *const _ as PGENERIC_MAPPING,
        SamDbUserObject => Mapping = &raw const UserMapping as *const _ as PGENERIC_MAPPING,
        _ => return STATUS_INVALID_HANDLE,
    }

    Status = SampGetObjectAttribute(
        DbObject,
        w!("SecDesc").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut RelativeSdSize,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(), 0, RelativeSdSize as SIZE_T);
    if RelativeSd.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttribute(
        DbObject,
        w!("SecDesc").as_ptr() as *mut u16,
        core::ptr::null_mut(),
        RelativeSd,
        &mut RelativeSdSize,
    );
    if !NT_SUCCESS(Status) {
        if !TokenHandle.is_null() {
            NtClose(TokenHandle);
        }
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
        return Status;
    }

    Status = RtlSetSecurityObject(
        SecurityInformation,
        (*SecurityDescriptor).SecurityDescriptor as PSECURITY_DESCRIPTOR,
        &mut RelativeSd,
        Mapping,
        TokenHandle,
    );
    if !NT_SUCCESS(Status) {
        if !TokenHandle.is_null() {
            NtClose(TokenHandle);
        }
        if !RelativeSd.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
        }
        return Status;
    }

    Status = SampSetObjectAttribute(
        DbObject,
        w!("SecDesc").as_ptr() as *mut u16,
        REG_BINARY,
        RelativeSd,
        RtlLengthSecurityDescriptor(RelativeSd),
    );

    if !TokenHandle.is_null() {
        NtClose(TokenHandle);
    }

    if !RelativeSd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQuerySecurityObject(
    ObjectHandle: SAMPR_HANDLE,
    SecurityInformation: SECURITY_INFORMATION,
    SecurityDescriptor: *mut PSAMPR_SR_SECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut SamObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let SdData: PSAMPR_SR_SECURITY_DESCRIPTOR;
    let mut RelativeSd: PISECURITY_DESCRIPTOR_RELATIVE = core::ptr::null_mut();
    let mut ResultSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut DesiredAccess: ACCESS_MASK = 0;
    let mut RelativeSdSize: ULONG = 0;
    let mut ResultSdSize: ULONG = 0;
    let mut Status: NTSTATUS;

    *SecurityDescriptor = core::ptr::null_mut();

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    if SecurityInformation
        & (DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION)
        != 0
    {
        DesiredAccess |= READ_CONTROL;
    }

    if SecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }

    'done: {
        /* Validate the server handle */
        Status = SampValidateDbObject(
            ObjectHandle,
            SamDbIgnoreObject,
            DesiredAccess,
            &mut SamObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Get the size of the SD */
        Status = SampGetObjectAttribute(
            SamObject,
            w!("SecDesc").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW {
            break 'done;
        }

        /* Allocate a buffer for the SD */
        RelativeSd = midl_user_allocate(RelativeSdSize as SIZE_T) as PISECURITY_DESCRIPTOR_RELATIVE;
        if RelativeSd.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        /* Get the SD */
        Status = SampGetObjectAttribute(
            SamObject,
            w!("SecDesc").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            RelativeSd as *mut core::ffi::c_void,
            &mut RelativeSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Invalidate the SD information that was not requested */
        if SecurityInformation & OWNER_SECURITY_INFORMATION == 0 {
            (*RelativeSd).Owner = 0;
        }

        if SecurityInformation & GROUP_SECURITY_INFORMATION == 0 {
            (*RelativeSd).Group = 0;
        }

        if SecurityInformation & DACL_SECURITY_INFORMATION == 0 {
            (*RelativeSd).Control &= !(SE_DACL_PRESENT as u16);
        }

        if SecurityInformation & SACL_SECURITY_INFORMATION == 0 {
            (*RelativeSd).Control &= !(SE_SACL_PRESENT as u16);
        }

        /* Calculate the required SD size */
        Status = RtlMakeSelfRelativeSD(
            RelativeSd as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            &mut ResultSdSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'done;
        }

        /* Allocate a buffer for the new SD */
        ResultSd = MIDL_user_allocate(ResultSdSize as SIZE_T) as PSECURITY_DESCRIPTOR;
        if ResultSd.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        /* Build the new SD */
        Status = RtlMakeSelfRelativeSD(
            RelativeSd as PSECURITY_DESCRIPTOR,
            ResultSd,
            &mut ResultSdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Allocate the SD data buffer */
        SdData = midl_user_allocate(
            core::mem::size_of::<SAMPR_SR_SECURITY_DESCRIPTOR>() as SIZE_T,
        ) as PSAMPR_SR_SECURITY_DESCRIPTOR;
        if SdData.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        /* Fill the SD data buffer and return it to the caller */
        (*SdData).Length = ResultSdSize;
        (*SdData).SecurityDescriptor = ResultSd as PBYTE;

        *SecurityDescriptor = SdData;
    }

    RtlReleaseResource(&raw mut SampResource);

    if !NT_SUCCESS(Status) {
        if !ResultSd.is_null() {
            MIDL_user_free(ResultSd as *mut core::ffi::c_void);
        }
    }

    if !RelativeSd.is_null() {
        MIDL_user_free(RelativeSd as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrShutdownSamServer(ServerHandle: SAMPR_HANDLE) -> NTSTATUS {
    let mut ServerObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    Status = SampValidateDbObject(
        ServerHandle,
        SamDbServerObject,
        SAM_SERVER_SHUTDOWN,
        core::ptr::addr_of_mut!(ServerObject),
    );

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    if !NT_SUCCESS(Status) {
        return Status;
    }

    RpcMgmtStopServerListening(core::ptr::null_mut());

    let _ = SampShutdownDisplayCache();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SamrLookupDomainInSamServer(
    ServerHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    DomainId: *mut PRPC_SID,
) -> NTSTATUS {
    let mut ServerObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut DomainsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut DomainKeyHandle: HANDLE = core::ptr::null_mut();
    let mut DomainKeyName: [WCHAR; 64] = [0; 64];
    let mut Index: ULONG;
    let mut DomainNameString: [WCHAR; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
        [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
    let mut DomainName: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Found: BOOL = 0;
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            ServerHandle,
            SamDbServerObject,
            SAM_SERVER_LOOKUP_DOMAIN,
            &mut ServerObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *DomainId = core::ptr::null_mut();

        Status = SampRegOpenKey(
            (*ServerObject).KeyHandle,
            L!("Domains"),
            KEY_READ,
            &mut DomainsKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Index = 0;
        while Found == 0 {
            Status = SampRegEnumerateSubKey(
                DomainsKeyHandle,
                Index,
                64,
                DomainKeyName.as_mut_ptr(),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_NO_SUCH_DOMAIN;
                }
                break;
            }

            Status = SampRegOpenKey(
                DomainsKeyHandle,
                DomainKeyName.as_mut_ptr(),
                KEY_READ,
                &mut DomainKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Length = (MAX_COMPUTERNAME_LENGTH + 1) * core::mem::size_of::<WCHAR>() as u32;
                Status = SampRegQueryValue(
                    DomainKeyHandle,
                    L!("Name"),
                    core::ptr::null_mut(),
                    DomainNameString.as_mut_ptr() as *mut core::ffi::c_void,
                    &mut Length,
                );
                if NT_SUCCESS(Status) {
                    RtlInitUnicodeString(&mut DomainName, DomainNameString.as_ptr());
                    if RtlEqualUnicodeString(&DomainName, Name as PUNICODE_STRING, 1u8) != 0 {
                        Found = 1;

                        Status = SampRegQueryValue(
                            DomainKeyHandle,
                            L!("SID"),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            &mut Length,
                        );
                        if NT_SUCCESS(Status) {
                            *DomainId = midl_user_allocate(Length as SIZE_T) as PRPC_SID;

                            SampRegQueryValue(
                                DomainKeyHandle,
                                L!("SID"),
                                core::ptr::null_mut(),
                                *DomainId as *mut core::ffi::c_void,
                                &mut Length,
                            );

                            Status = STATUS_SUCCESS;
                            break;
                        }
                    }
                }

                SampRegCloseKey(&mut DomainKeyHandle);
            }

            Index += 1;
        }
    }

    SampRegCloseKey(&mut DomainKeyHandle);
    SampRegCloseKey(&mut DomainsKeyHandle);

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrEnumerateDomainsInSamServer(
    ServerHandle: SAMPR_HANDLE,
    EnumerationContext: *mut core::ffi::c_ulong,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: ULONG,
    CountReturned: PULONG,
) -> NTSTATUS {
    let mut ServerObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut DomainKeyName: [u16; 64] = [0; 64];
    let mut DomainsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut DomainKeyHandle: HANDLE = core::ptr::null_mut();
    let mut EnumIndex: ULONG;
    let mut EnumCount: ULONG;
    let mut RequiredLength: ULONG;
    let mut DataLength: ULONG;
    let mut i: ULONG;
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            ServerHandle,
            SamDbServerObject,
            SAM_SERVER_ENUMERATE_DOMAINS,
            core::ptr::addr_of_mut!(ServerObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            (*ServerObject).KeyHandle,
            w!("Domains").as_ptr() as LPCWSTR,
            KEY_READ,
            core::ptr::addr_of_mut!(DomainsKeyHandle),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumIndex = *EnumerationContext;
        EnumCount = 0;
        RequiredLength = 0;

        loop {
            Status = SampRegEnumerateSubKey(
                DomainsKeyHandle,
                EnumIndex,
                (64 * core::mem::size_of::<u16>()) as ULONG,
                DomainKeyName.as_mut_ptr(),
            );
            if !NT_SUCCESS(Status) {
                break;
            }

            Status = SampRegOpenKey(
                DomainsKeyHandle,
                DomainKeyName.as_ptr(),
                KEY_READ,
                core::ptr::addr_of_mut!(DomainKeyHandle),
            );
            if NT_SUCCESS(Status) {
                DataLength = 0;
                Status = SampRegQueryValue(
                    DomainKeyHandle,
                    w!("Name").as_ptr() as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(DataLength),
                );
                if NT_SUCCESS(Status) {
                    if (RequiredLength + DataLength + core::mem::size_of::<_UNICODE_STRING>() as ULONG) > PreferedMaximumLength {
                        break;
                    }

                    RequiredLength += DataLength + core::mem::size_of::<_UNICODE_STRING>() as ULONG;
                    EnumCount += 1;
                }

                SampRegCloseKey(core::ptr::addr_of_mut!(DomainKeyHandle));
            }

            EnumIndex += 1;
        }

        EnumBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ENUMERATION_BUFFER>() as SIZE_T) as PSAMPR_ENUMERATION_BUFFER;
        if EnumBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*EnumBuffer).EntriesRead = EnumCount;
        (*EnumBuffer).Buffer = midl_user_allocate((EnumCount as usize * core::mem::size_of::<SAMPR_RID_ENUMERATION>()) as SIZE_T) as *mut SAMPR_RID_ENUMERATION;
        if (*EnumBuffer).Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        EnumIndex = *EnumerationContext;
        i = 0;
        while i < EnumCount {
            Status = SampRegEnumerateSubKey(
                DomainsKeyHandle,
                EnumIndex,
                (64 * core::mem::size_of::<u16>()) as ULONG,
                DomainKeyName.as_mut_ptr(),
            );
            if !NT_SUCCESS(Status) {
                break;
            }

            Status = SampRegOpenKey(
                DomainsKeyHandle,
                DomainKeyName.as_ptr(),
                KEY_READ,
                core::ptr::addr_of_mut!(DomainKeyHandle),
            );
            if NT_SUCCESS(Status) {
                DataLength = 0;
                Status = SampRegQueryValue(
                    DomainKeyHandle,
                    w!("Name").as_ptr() as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(DataLength),
                );
                if NT_SUCCESS(Status) {
                    (*(*EnumBuffer).Buffer.add(i as usize)).RelativeId = 0;
                    (*(*EnumBuffer).Buffer.add(i as usize)).Name.Length = (DataLength as USHORT) - core::mem::size_of::<u16>() as USHORT;
                    (*(*EnumBuffer).Buffer.add(i as usize)).Name.MaximumLength = DataLength as USHORT;
                    (*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer = midl_user_allocate(DataLength as SIZE_T) as *mut u16;
                    if (*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer.is_null() {
                        SampRegCloseKey(core::ptr::addr_of_mut!(DomainKeyHandle));
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        break 'done;
                    }

                    Status = SampRegQueryValue(
                        DomainKeyHandle,
                        w!("Name").as_ptr() as LPCWSTR,
                        core::ptr::null_mut(),
                        (*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer as PVOID,
                        core::ptr::addr_of_mut!(DataLength),
                    );
                }

                SampRegCloseKey(core::ptr::addr_of_mut!(DomainKeyHandle));

                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }

            i += 1;
            EnumIndex += 1;
        }

        if NT_SUCCESS(Status) {
            *EnumerationContext += EnumCount;
            *Buffer = EnumBuffer;
            *CountReturned = EnumCount;
        }

        break 'done;
    }

    SampRegCloseKey(core::ptr::addr_of_mut!(DomainKeyHandle));
    SampRegCloseKey(core::ptr::addr_of_mut!(DomainsKeyHandle));

    if !NT_SUCCESS(Status) {
        *EnumerationContext = 0;
        *Buffer = core::ptr::null_mut();
        *CountReturned = 0;

        if !EnumBuffer.is_null() {
            if !(*EnumBuffer).Buffer.is_null() {
                if (*EnumBuffer).EntriesRead != 0 {
                    i = 0;
                    while i < (*EnumBuffer).EntriesRead {
                        if !(*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer.is_null() {
                            midl_user_free((*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer as *mut core::ffi::c_void);
                        }
                        i += 1;
                    }
                }

                midl_user_free((*EnumBuffer).Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(EnumBuffer as *mut core::ffi::c_void);
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrOpenDomain(
    ServerHandle: SAMPR_HANDLE,
    mut DesiredAccess: ACCESS_MASK,
    DomainId: PRPC_SID,
    DomainHandle: *mut SAMPR_HANDLE,
) -> NTSTATUS {
    let mut ServerObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrOpenDomain(%p %lx %p %p)\n\0".as_ptr() as *const i8,
        ServerHandle,
        DesiredAccess,
        DomainId,
        DomainHandle,
    );

    /* Map generic access rights */
    RtlMapGenericMask(
        core::ptr::addr_of_mut!(DesiredAccess),
        core::ptr::addr_of_mut!(DomainMapping),
    );

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    /* Validate the server handle */
    Status = SampValidateDbObject(
        ServerHandle,
        SamDbServerObject,
        SAM_SERVER_LOOKUP_DOMAIN,
        core::ptr::addr_of_mut!(ServerObject),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Validate the Domain SID */
    if ((*DomainId).Revision as u32 != SID_REVISION)
        || ((*DomainId).SubAuthorityCount as u32 > SID_MAX_SUB_AUTHORITIES)
        || (memcmp(
            core::ptr::addr_of!((*DomainId).IdentifierAuthority) as *const core::ffi::c_void,
            core::ptr::addr_of!(NtSidAuthority) as *const core::ffi::c_void,
            core::mem::size_of::<SID_IDENTIFIER_AUTHORITY>(),
        ) != 0)
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Open the domain object */
    if ((*DomainId).SubAuthorityCount == 1)
        && (*(*DomainId).SubAuthority.as_ptr().offset(0) == SECURITY_BUILTIN_DOMAIN_RID as u32)
    {
        /* Builtin domain object */
        TRACE!(b"Opening the builtin domain object.\n\0".as_ptr() as *const i8);

        Status = SampOpenDbObject(
            ServerObject,
            wstr!("Domains"),
            wstr!("Builtin"),
            0,
            SamDbDomainObject,
            DesiredAccess,
            core::ptr::addr_of_mut!(DomainObject),
        );
    } else if ((*DomainId).SubAuthorityCount == 4)
        && (*(*DomainId).SubAuthority.as_ptr().offset(0) == SECURITY_NT_NON_UNIQUE as u32)
    {
        /* Account domain object */
        TRACE!(b"Opening the account domain object.\n\0".as_ptr() as *const i8);

        /* FIXME: Check the account domain sub authorities!!! */

        Status = SampOpenDbObject(
            ServerObject,
            wstr!("Domains"),
            wstr!("Account"),
            0,
            SamDbDomainObject,
            DesiredAccess,
            core::ptr::addr_of_mut!(DomainObject),
        );
    } else {
        /* No valid domain SID */
        Status = STATUS_INVALID_PARAMETER;
    }

    if NT_SUCCESS(Status) {
        *DomainHandle = DomainObject as SAMPR_HANDLE;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    TRACE!(b"SamrOpenDomain done (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);

    Status
}

unsafe fn SampQueryDomainPassword(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Password.MinPasswordLength = FixedData.MinPasswordLength;
        (*InfoBuffer).Password.PasswordHistoryLength = FixedData.PasswordHistoryLength;
        (*InfoBuffer).Password.PasswordProperties = FixedData.PasswordProperties;
        (*InfoBuffer).Password.MaxPasswordAge.LowPart = FixedData.MaxPasswordAge.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Password.MaxPasswordAge.HighPart = FixedData.MaxPasswordAge.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Password.MinPasswordAge.LowPart = FixedData.MinPasswordAge.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Password.MinPasswordAge.HighPart = FixedData.MinPasswordAge.__bindgen_anon_1.HighPart;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampGetNumberOfAccounts(
    DomainObject: PSAM_DB_OBJECT,
    AccountType: LPCWSTR,
    Count: PULONG,
) -> NTSTATUS {
    let mut AccountKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    *Count = 0;

    Status = SampRegOpenKey(
        (*DomainObject).KeyHandle,
        AccountType,
        KEY_READ,
        &mut AccountKeyHandle,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'done: {
        Status = SampRegOpenKey(
            AccountKeyHandle,
            w!("Names").as_ptr() as LPCWSTR,
            KEY_READ,
            &mut NamesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegQueryKeyInfo(NamesKeyHandle, core::ptr::null_mut(), Count);
    }

    SampRegCloseKey(&mut NamesKeyHandle);
    SampRegCloseKey(&mut AccountKeyHandle);

    Status
}

unsafe fn SampQueryDomainGeneral(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(
        core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T,
    ) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        wstr!("F"),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).General.ForceLogoff.LowPart = FixedData.ForceLogoff.__bindgen_anon_1.LowPart;
        (*InfoBuffer).General.ForceLogoff.HighPart = FixedData.ForceLogoff.__bindgen_anon_1.HighPart;
        (*InfoBuffer).General.DomainModifiedCount.LowPart =
            FixedData.DomainModifiedCount.__bindgen_anon_1.LowPart;
        (*InfoBuffer).General.DomainModifiedCount.HighPart =
            FixedData.DomainModifiedCount.__bindgen_anon_1.HighPart;
        (*InfoBuffer).General.DomainServerState = FixedData.DomainServerState;
        (*InfoBuffer).General.DomainServerRole = FixedData.DomainServerRole;
        (*InfoBuffer).General.UasCompatibilityRequired = FixedData.UasCompatibilityRequired;

        Status = SampGetObjectAttributeString(
            DomainObject,
            wstr!("OemInformation"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.OemInformation),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            DomainObject,
            wstr!("Name"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.DomainName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            DomainObject,
            wstr!("ReplicaSourceNodeName"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.ReplicaSourceNodeName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetNumberOfAccounts(
            DomainObject,
            wide!("Users").as_ptr(),
            core::ptr::addr_of_mut!((*InfoBuffer).General.UserCount),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetNumberOfAccounts(
            DomainObject,
            wide!("Groups").as_ptr(),
            core::ptr::addr_of_mut!((*InfoBuffer).General.GroupCount),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetNumberOfAccounts(
            DomainObject,
            wide!("Aliases").as_ptr(),
            core::ptr::addr_of_mut!((*InfoBuffer).General.AliasCount),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).General.OemInformation.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.OemInformation.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.DomainName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.DomainName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.ReplicaSourceNodeName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.ReplicaSourceNodeName.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryDomainLogoff(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as *mut core::ffi::c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Logoff.ForceLogoff.LowPart =
            FixedData.ForceLogoff.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Logoff.ForceLogoff.HighPart =
            FixedData.ForceLogoff.__bindgen_anon_1.HighPart;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryDomainOem(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        DomainObject,
        w!("OemInformation").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).Oem.OemInformation),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Oem.OemInformation.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Oem.OemInformation.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryDomainName(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = SampGetObjectAttributeString(
            DomainObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Name.DomainName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Name.DomainName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Name.DomainName.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryDomainReplication(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        DomainObject,
        w!("ReplicaSourceNodeName").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).Replication.ReplicaSourceNodeName),
    );

    if NT_SUCCESS(Status) {
        *Buffer = InfoBuffer;
    } else {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Replication.ReplicaSourceNodeName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Replication.ReplicaSourceNodeName.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryDomainServerRole(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        w!("F").as_ptr() as *const u16 as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Role.DomainServerRole = FixedData.DomainServerRole;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryDomainModified(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        widestring!("F"),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Modified.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Modified.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Modified.CreationTime.LowPart = FixedData.CreationTime.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Modified.CreationTime.HighPart = FixedData.CreationTime.__bindgen_anon_1.HighPart;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryDomainState(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(Length),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).State.DomainServerState = FixedData.DomainServerState;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryDomainGeneral2(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;

    'done: loop {
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).General2.I1.ForceLogoff.LowPart = FixedData.ForceLogoff.__bindgen_anon_1.LowPart;
        (*InfoBuffer).General2.I1.ForceLogoff.HighPart = FixedData.ForceLogoff.__bindgen_anon_1.HighPart;
        (*InfoBuffer).General2.I1.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.__bindgen_anon_1.LowPart;
        (*InfoBuffer).General2.I1.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.__bindgen_anon_1.HighPart;
        (*InfoBuffer).General2.I1.DomainServerState = FixedData.DomainServerState;
        (*InfoBuffer).General2.I1.DomainServerRole = FixedData.DomainServerRole;
        (*InfoBuffer).General2.I1.UasCompatibilityRequired = FixedData.UasCompatibilityRequired;

        (*InfoBuffer).General2.LockoutDuration = core::mem::transmute(FixedData.LockoutDuration);
        (*InfoBuffer).General2.LockoutObservationWindow = core::mem::transmute(FixedData.LockoutObservationWindow);
        (*InfoBuffer).General2.LockoutThreshold = FixedData.LockoutThreshold;

        Status = SampGetObjectAttributeString(
            DomainObject,
            w!("OemInformation").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General2.I1.OemInformation),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            DomainObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General2.I1.DomainName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            DomainObject,
            w!("ReplicaSourceNodeName").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General2.I1.ReplicaSourceNodeName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetNumberOfAccounts(
            DomainObject,
            w!("Users").as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General2.I1.UserCount),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetNumberOfAccounts(
            DomainObject,
            w!("Groups").as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General2.I1.GroupCount),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetNumberOfAccounts(
            DomainObject,
            w!("Aliases").as_ptr() as LPCWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General2.I1.AliasCount),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).General2.I1.OemInformation.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General2.I1.OemInformation.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General2.I1.DomainName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General2.I1.DomainName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General2.I1.ReplicaSourceNodeName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General2.I1.ReplicaSourceNodeName.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryDomainLockout(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        windows_wide!("F") as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Lockout.LockoutDuration = core::mem::transmute(FixedData.LockoutDuration);
        (*InfoBuffer).Lockout.LockoutObservationWindow = core::mem::transmute(FixedData.LockoutObservationWindow);
        (*InfoBuffer).Lockout.LockoutThreshold = FixedData.LockoutThreshold;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryDomainModified2(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_DOMAIN_INFO_BUFFER;
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_DOMAIN_INFO_BUFFER>() as SIZE_T) as PSAMPR_DOMAIN_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        w!("F").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Modified2.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Modified2.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Modified2.CreationTime.LowPart = FixedData.CreationTime.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Modified2.CreationTime.HighPart = FixedData.CreationTime.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Modified2.ModifiedCountAtLastPromotion.LowPart = FixedData.ModifiedCountAtLastPromotion.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Modified2.ModifiedCountAtLastPromotion.HighPart = FixedData.ModifiedCountAtLastPromotion.__bindgen_anon_1.HighPart;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryInformationDomain(
    DomainHandle: SAMPR_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    SamrQueryInformationDomain2(DomainHandle, DomainInformationClass, Buffer)
}

unsafe fn SampSetDomainPassword(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        widestring!("F"),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.MinPasswordLength = (*Buffer).Password.MinPasswordLength;
        FixedData.PasswordHistoryLength = (*Buffer).Password.PasswordHistoryLength;
        FixedData.PasswordProperties = (*Buffer).Password.PasswordProperties;
        FixedData.MaxPasswordAge.__bindgen_anon_1.LowPart = (*Buffer).Password.MaxPasswordAge.LowPart;
        FixedData.MaxPasswordAge.__bindgen_anon_1.HighPart = (*Buffer).Password.MaxPasswordAge.HighPart;
        FixedData.MinPasswordAge.__bindgen_anon_1.LowPart = (*Buffer).Password.MinPasswordAge.LowPart;
        FixedData.MinPasswordAge.__bindgen_anon_1.HighPart = (*Buffer).Password.MinPasswordAge.HighPart;

        Status = SampSetObjectAttribute(
            DomainObject,
            widestring!("F"),
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
    }

    Status
}

unsafe fn SampSetDomainLogoff(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        w!("F").as_ptr() as *const u16 as LPWSTR,
        core::ptr::null_mut(),
        &mut FixedData as *mut _ as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.ForceLogoff.__bindgen_anon_1.LowPart = (*Buffer).Logoff.ForceLogoff.LowPart;
        FixedData.ForceLogoff.__bindgen_anon_1.HighPart = (*Buffer).Logoff.ForceLogoff.HighPart;

        Status = SampSetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            REG_BINARY,
            &mut FixedData as *mut _ as PVOID,
            Length,
        );
    }

    Status
}

unsafe fn SampSetDomainServerRole(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        w!("F").as_ptr() as *const u16 as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.DomainServerRole = (*Buffer).Role.DomainServerRole;

        return SampSetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
    }

    Status
}

unsafe fn SampSetDomainState(DomainObject: PSAM_DB_OBJECT,
                           Buffer: PSAMPR_DOMAIN_INFO_BUFFER) -> NTSTATUS {
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(DomainObject,
                                    widestr!("F"),
                                    core::ptr::null_mut(),
                                    core::ptr::addr_of_mut!(FixedData) as PVOID,
                                    core::ptr::addr_of_mut!(Length));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    FixedData.DomainServerState = (*Buffer).State.DomainServerState;

    Status = SampSetObjectAttribute(DomainObject,
                                    widestr!("F"),
                                    REG_BINARY,
                                    core::ptr::addr_of_mut!(FixedData) as PVOID,
                                    Length);

    Status
}

unsafe fn SampSetDomainLockout(
    DomainObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        DomainObject,
        widestr!("F"),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.LockoutDuration = core::mem::transmute((*Buffer).Lockout.LockoutDuration);
        FixedData.LockoutObservationWindow = core::mem::transmute((*Buffer).Lockout.LockoutObservationWindow);
        FixedData.LockoutThreshold = (*Buffer).Lockout.LockoutThreshold;

        Status = SampSetObjectAttribute(
            DomainObject,
            widestr!("F"),
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetInformationDomain(
    DomainHandle: SAMPR_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    DomainInformation: PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let DesiredAccess: ACCESS_MASK;
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrSetInformationDomain(%p %lu %p)\n\0".as_ptr() as *const i8,
        DomainHandle,
        DomainInformationClass,
        DomainInformation,
    );

    match DomainInformationClass {
        DomainPasswordInformation | DomainLockoutInformation => {
            DesiredAccess = DOMAIN_WRITE_PASSWORD_PARAMS;
        }
        DomainLogoffInformation | DomainOemInformation | DomainNameInformation => {
            DesiredAccess = DOMAIN_WRITE_OTHER_PARAMETERS;
        }
        DomainReplicationInformation | DomainServerRoleInformation | DomainStateInformation => {
            DesiredAccess = DOMAIN_ADMINISTER_SERVER;
        }
        _ => {
            return STATUS_INVALID_INFO_CLASS;
        }
    }

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DesiredAccess,
            core::ptr::addr_of_mut!(DomainObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        match DomainInformationClass {
            DomainPasswordInformation => {
                Status = SampSetDomainPassword(DomainObject, DomainInformation);
            }
            DomainLogoffInformation => {
                Status = SampSetDomainLogoff(DomainObject, DomainInformation);
            }
            DomainOemInformation => {
                Status = SampSetObjectAttributeString(
                    DomainObject,
                    wstr!("OemInformation") as LPWSTR,
                    core::ptr::addr_of_mut!((*DomainInformation).Oem.OemInformation),
                );
            }
            DomainNameInformation => {
                Status = SampSetObjectAttributeString(
                    DomainObject,
                    wstr!("Name") as LPWSTR,
                    core::ptr::addr_of_mut!((*DomainInformation).Name.DomainName),
                );
            }
            DomainReplicationInformation => {
                Status = SampSetObjectAttributeString(
                    DomainObject,
                    wstr!("ReplicaSourceNodeName") as LPWSTR,
                    core::ptr::addr_of_mut!((*DomainInformation).Replication.ReplicaSourceNodeName),
                );
            }
            DomainServerRoleInformation => {
                Status = SampSetDomainServerRole(DomainObject, DomainInformation);
            }
            DomainStateInformation => {
                Status = SampSetDomainState(DomainObject, DomainInformation);
            }
            DomainLockoutInformation => {
                Status = SampSetDomainLockout(DomainObject, DomainInformation);
            }
            _ => {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrCreateGroupInDomain(
    DomainHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    mut DesiredAccess: ACCESS_MASK,
    GroupHandle: *mut SAMPR_HANDLE,
    RelativeId: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    let mut FixedDomainData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut FixedGroupData: SAM_GROUP_FIXED_DATA = core::mem::zeroed();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut ulSize: ULONG;
    let ulRid: ULONG;
    let mut szRid: [u16; 9] = [0; 9];
    let mut Status: NTSTATUS;

    RtlMapGenericMask(&mut DesiredAccess, &raw mut GroupMapping);

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_CREATE_GROUP,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountName(Name, 256);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountNameInDomain(DomainObject, (*Name).Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateGroupSD(&mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulSize = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            wstr!("F"),
            core::ptr::null_mut(),
            &mut FixedDomainData as *mut _ as PVOID,
            &mut ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulRid = FixedDomainData.NextRid;
        FixedDomainData.NextRid += 1;

        Status = SampSetObjectAttribute(
            DomainObject,
            wstr!("F"),
            REG_BINARY,
            &mut FixedDomainData as *mut _ as PVOID,
            ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        swprintf(
            szRid.as_mut_ptr(),
            wstr!("%08lX"),
            ulRid,
        );

        Status = SampCreateDbObject(
            DomainObject,
            wstr!("Groups"),
            szRid.as_ptr() as *mut u16,
            ulRid,
            SamDbGroupObject,
            DesiredAccess,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetAccountNameInDomain(
            DomainObject,
            wstr!("Groups"),
            (*Name).Buffer,
            ulRid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        memset(
            &mut FixedGroupData as *mut _ as PVOID,
            0,
            core::mem::size_of::<SAM_GROUP_FIXED_DATA>(),
        );
        FixedGroupData.Version = 1;
        FixedGroupData.GroupId = ulRid;

        Status = SampSetObjectAttribute(
            GroupObject,
            wstr!("F"),
            REG_BINARY,
            &mut FixedGroupData as *mut _ as LPVOID,
            core::mem::size_of::<SAM_GROUP_FIXED_DATA>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            GroupObject,
            wstr!("Name"),
            Name,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            GroupObject,
            wstr!("AdminComment"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            GroupObject,
            wstr!("SecDesc"),
            REG_BINARY,
            Sd,
            SdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if NT_SUCCESS(Status) {
            *GroupHandle = GroupObject as SAMPR_HANDLE;
            *RelativeId = ulRid;
        }

        break 'done;
    }

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrEnumerateGroupsInDomain(
    DomainHandle: SAMPR_HANDLE,
    EnumerationContext: *mut core::ffi::c_ulong,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: core::ffi::c_ulong,
    CountReturned: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut GroupsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut GroupName: [WCHAR; 64] = [0; 64];
    let mut EnumIndex: ULONG;
    let mut EnumCount: ULONG = 0;
    let mut RequiredLength: ULONG = 0;
    let mut NameLength: ULONG;
    let mut DataLength: ULONG;
    let mut Rid: ULONG = 0;
    let mut i: ULONG;
    let mut MoreEntries: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LIST_ACCOUNTS,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            (*DomainObject).KeyHandle,
            w!("Groups").as_ptr() as LPCWSTR,
            KEY_READ,
            &mut GroupsKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            GroupsKeyHandle,
            w!("Names").as_ptr() as LPCWSTR,
            KEY_READ,
            &mut NamesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumIndex = *EnumerationContext;

        loop {
            NameLength = (64 * core::mem::size_of::<WCHAR>()) as ULONG;
            Status = SampRegEnumerateValue(
                NamesKeyHandle,
                EnumIndex,
                GroupName.as_mut_ptr(),
                &mut NameLength,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            if (RequiredLength as usize
                + NameLength as usize
                + core::mem::size_of::<WCHAR>()
                + core::mem::size_of::<SAMPR_RID_ENUMERATION>())
                > PreferedMaximumLength as usize
            {
                MoreEntries = 1u8;
                break;
            }

            RequiredLength += NameLength
                + core::mem::size_of::<WCHAR>() as ULONG
                + core::mem::size_of::<SAMPR_RID_ENUMERATION>() as ULONG;
            EnumCount += 1;

            EnumIndex += 1;
        }

        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ENUMERATION_BUFFER>() as SIZE_T)
            as PSAMPR_ENUMERATION_BUFFER;
        if EnumBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*EnumBuffer).EntriesRead = EnumCount;
        if EnumCount == 0 {
            Status = STATUS_NO_MORE_ENTRIES;
            break 'done;
        }

        (*EnumBuffer).Buffer = midl_user_allocate(
            (EnumCount as usize * core::mem::size_of::<SAMPR_RID_ENUMERATION>()) as SIZE_T,
        ) as *mut SAMPR_RID_ENUMERATION;
        if (*EnumBuffer).Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        EnumIndex = *EnumerationContext;
        i = 0;
        while i < EnumCount {
            NameLength = (64 * core::mem::size_of::<WCHAR>()) as ULONG;
            DataLength = core::mem::size_of::<ULONG>() as ULONG;
            Status = SampRegEnumerateValue(
                NamesKeyHandle,
                EnumIndex,
                GroupName.as_mut_ptr(),
                &mut NameLength,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Rid) as PVOID,
                &mut DataLength,
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            (*(*EnumBuffer).Buffer.add(i as usize)).RelativeId = Rid;

            (*(*EnumBuffer).Buffer.add(i as usize)).Name.Length = NameLength as USHORT;
            (*(*EnumBuffer).Buffer.add(i as usize)).Name.MaximumLength =
                (NameLength + core::mem::size_of::<WCHAR>() as ULONG) as USHORT;

            i += 1;
            EnumIndex += 1;
        }
    }

    if NT_SUCCESS(Status) {
        *EnumerationContext += EnumCount;
        *Buffer = EnumBuffer;
        *CountReturned = EnumCount;
    } else {
        *EnumerationContext = 0;
        *Buffer = core::ptr::null_mut();
        *CountReturned = 0;

        if !EnumBuffer.is_null() {
            if !(*EnumBuffer).Buffer.is_null() {
                if (*EnumBuffer).EntriesRead != 0 {
                    i = 0;
                    while i < (*EnumBuffer).EntriesRead {
                        if !(*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer.is_null() {
                            midl_user_free(
                                (*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer
                                    as *mut core::ffi::c_void,
                            );
                        }
                        i += 1;
                    }
                }

                midl_user_free((*EnumBuffer).Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(EnumBuffer as *mut core::ffi::c_void);
        }
    }

    SampRegCloseKey(&mut NamesKeyHandle);
    SampRegCloseKey(&mut GroupsKeyHandle);

    if Status == STATUS_SUCCESS && MoreEntries != 0 {
        Status = STATUS_MORE_ENTRIES;
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrCreateUserInDomain(
    DomainHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    mut DesiredAccess: ACCESS_MASK,
    UserHandle: *mut SAMPR_HANDLE,
    RelativeId: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    let mut FixedDomainData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut FixedUserData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut GroupMembership: GROUP_MEMBERSHIP = core::mem::zeroed();
    let mut LogonHours: [UCHAR; 23] = [0; 23];
    let mut ulSize: ULONG;
    let ulRid: ULONG;
    let mut szRid: [WCHAR; 9] = [0; 9];
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut UserSid: PSID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if Name.is_null()
        || (*Name).Length == 0
        || (*Name).Buffer.is_null()
        || UserHandle.is_null()
        || RelativeId.is_null()
    {
        return STATUS_INVALID_PARAMETER;
    }

    RtlMapGenericMask(&mut DesiredAccess, &raw mut UserMapping);

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_CREATE_USER,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountName(Name, 20);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountNameInDomain(DomainObject, (*Name).Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulSize = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            wstr!("F"),
            core::ptr::null_mut(),
            &mut FixedDomainData as *mut _ as PVOID,
            &mut ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulRid = FixedDomainData.NextRid;
        FixedDomainData.NextRid += 1;

        Status = SampCreateAccountSid(DomainObject, ulRid, &mut UserSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateUserSD(UserSid, &mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            DomainObject,
            wstr!("F"),
            REG_BINARY,
            &mut FixedDomainData as *mut _ as PVOID,
            ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        swprintf(
            szRid.as_mut_ptr(),
            wstr!("%08lX"),
            ulRid,
        );

        Status = SampCreateDbObject(
            DomainObject,
            wstr!("Users"),
            szRid.as_ptr() as *mut u16,
            ulRid,
            SamDbUserObject,
            DesiredAccess,
            &mut UserObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetAccountNameInDomain(
            DomainObject,
            wstr!("Users"),
            (*Name).Buffer,
            ulRid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        memset(
            &mut FixedUserData as *mut _ as PVOID,
            0,
            core::mem::size_of::<SAM_USER_FIXED_DATA>(),
        );
        FixedUserData.Version = 1;
        FixedUserData.Reserved = 0;
        FixedUserData.LastLogon.QuadPart = 0;
        FixedUserData.LastLogoff.QuadPart = 0;
        FixedUserData.PasswordLastSet.QuadPart = 0;
        FixedUserData.AccountExpires.QuadPart = MAXLONGLONG as _;
        FixedUserData.LastBadPasswordTime.QuadPart = 0;
        FixedUserData.UserId = ulRid;
        FixedUserData.PrimaryGroupId = DOMAIN_GROUP_RID_USERS;
        FixedUserData.UserAccountControl =
            USER_ACCOUNT_DISABLED | USER_PASSWORD_NOT_REQUIRED | USER_NORMAL_ACCOUNT;
        FixedUserData.CountryCode = 0;
        FixedUserData.CodePage = 0;
        FixedUserData.BadPasswordCount = 0;
        FixedUserData.LogonCount = 0;
        FixedUserData.AdminCount = 0;
        FixedUserData.OperatorCount = 0;

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("F"),
            REG_BINARY,
            &mut FixedUserData as *mut _ as LPVOID,
            core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("Name"),
            Name,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("FullName"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("HomeDirectory"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("HomeDirectoryDrive"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("ScriptPath"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("ProfilePath"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("AdminComment"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("UserComment"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("WorkStations"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("Parameters"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (LogonHours.as_mut_ptr() as *mut USHORT).write_unaligned(168);
        memset(&mut LogonHours[2] as *mut _ as PVOID, 0xff, 21);

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("LogonHours"),
            REG_BINARY,
            LogonHours.as_mut_ptr() as PVOID,
            core::mem::size_of::<[UCHAR; 23]>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        GroupMembership.RelativeId = DOMAIN_GROUP_RID_USERS;
        GroupMembership.Attributes =
            SE_GROUP_MANDATORY | SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT;

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("Groups"),
            REG_BINARY,
            &mut GroupMembership as *mut _ as PVOID,
            core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("LMPwd"),
            REG_BINARY,
            &raw const EmptyLmHash as *const _ as PVOID,
            core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("NTPwd"),
            REG_BINARY,
            &raw const EmptyNtHash as *const _ as PVOID,
            core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("LMPwdHistory"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("NTPwdHistory"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("PrivateData"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("SecDesc"),
            REG_BINARY,
            Sd,
            SdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if NT_SUCCESS(Status) {
            *UserHandle = UserObject as SAMPR_HANDLE;
            *RelativeId = ulRid;
        }
    }

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    if !UserSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrEnumerateUsersInDomain(
    DomainHandle: SAMPR_HANDLE,
    EnumerationContext: *mut core::ffi::c_ulong,
    _UserAccountControl: core::ffi::c_ulong,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: core::ffi::c_ulong,
    CountReturned: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UsersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut UserName: [WCHAR; 64] = [0; 64];
    let mut EnumIndex: ULONG;
    let mut EnumCount: ULONG = 0;
    let mut RequiredLength: ULONG = 0;
    let mut NameLength: ULONG;
    let mut DataLength: ULONG;
    let mut Rid: ULONG = 0;
    let mut i: ULONG;
    let mut MoreEntries: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    Status = SampValidateDbObject(
        DomainHandle,
        SamDbDomainObject,
        DOMAIN_LIST_ACCOUNTS,
        core::ptr::addr_of_mut!(DomainObject),
    );

    'done: loop {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            (*DomainObject).KeyHandle,
            wstr!("Users"),
            KEY_READ,
            core::ptr::addr_of_mut!(UsersKeyHandle),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            UsersKeyHandle,
            wstr!("Names"),
            KEY_READ,
            core::ptr::addr_of_mut!(NamesKeyHandle),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumIndex = *EnumerationContext;

        loop {
            NameLength = (64 * core::mem::size_of::<WCHAR>()) as ULONG;
            Status = SampRegEnumerateValue(
                NamesKeyHandle,
                EnumIndex,
                UserName.as_mut_ptr(),
                core::ptr::addr_of_mut!(NameLength),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            if (RequiredLength as usize
                + NameLength as usize
                + core::mem::size_of::<u16>()
                + core::mem::size_of::<SAMPR_RID_ENUMERATION>())
                > PreferedMaximumLength as usize
            {
                MoreEntries = 1u8;
                break;
            }

            RequiredLength += NameLength
                + core::mem::size_of::<u16>() as ULONG
                + core::mem::size_of::<SAMPR_RID_ENUMERATION>() as ULONG;
            EnumCount += 1;

            EnumIndex += 1;
        }

        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ENUMERATION_BUFFER>() as SIZE_T)
            as PSAMPR_ENUMERATION_BUFFER;
        if EnumBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*EnumBuffer).EntriesRead = EnumCount;
        if EnumCount == 0 {
            Status = STATUS_NO_MORE_ENTRIES;
            break 'done;
        }

        (*EnumBuffer).Buffer = midl_user_allocate(
            (EnumCount as usize * core::mem::size_of::<SAMPR_RID_ENUMERATION>()) as SIZE_T,
        ) as *mut SAMPR_RID_ENUMERATION;
        if (*EnumBuffer).Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        EnumIndex = *EnumerationContext;
        i = 0;
        while i < EnumCount {
            NameLength = (64 * core::mem::size_of::<WCHAR>()) as ULONG;
            DataLength = core::mem::size_of::<ULONG>() as ULONG;
            Status = SampRegEnumerateValue(
                NamesKeyHandle,
                EnumIndex,
                UserName.as_mut_ptr(),
                core::ptr::addr_of_mut!(NameLength),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Rid) as PVOID,
                core::ptr::addr_of_mut!(DataLength),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            (*(*EnumBuffer).Buffer.add(i as usize)).RelativeId = Rid;

            (*(*EnumBuffer).Buffer.add(i as usize)).Name.Length = NameLength as USHORT;
            (*(*EnumBuffer).Buffer.add(i as usize)).Name.MaximumLength =
                (NameLength + core::mem::size_of::<u16>() as ULONG) as USHORT;

            i += 1;
            EnumIndex += 1;
        }

        break 'done;
    }

    if NT_SUCCESS(Status) {
        *EnumerationContext += EnumCount;
        *Buffer = EnumBuffer;
        *CountReturned = EnumCount;
    } else {
        *EnumerationContext = 0;
        *Buffer = core::ptr::null_mut();
        *CountReturned = 0;

        if !EnumBuffer.is_null() {
            if !(*EnumBuffer).Buffer.is_null() {
                if (*EnumBuffer).EntriesRead != 0 {
                    i = 0;
                    while i < (*EnumBuffer).EntriesRead {
                        if !(*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer.is_null() {
                            midl_user_free(
                                (*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer
                                    as *mut core::ffi::c_void,
                            );
                        }
                        i += 1;
                    }
                }

                midl_user_free((*EnumBuffer).Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(EnumBuffer as *mut core::ffi::c_void);
        }
    }

    SampRegCloseKey(core::ptr::addr_of_mut!(NamesKeyHandle));
    SampRegCloseKey(core::ptr::addr_of_mut!(UsersKeyHandle));

    if Status == STATUS_SUCCESS && MoreEntries != 0 {
        Status = STATUS_MORE_ENTRIES;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrCreateAliasInDomain(
    DomainHandle: SAMPR_HANDLE,
    AccountName: PRPC_UNICODE_STRING,
    mut DesiredAccess: ACCESS_MASK,
    AliasHandle: *mut SAMPR_HANDLE,
    RelativeId: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    let mut FixedDomainData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut ulSize: ULONG;
    let ulRid: ULONG;
    let mut szRid: [u16; 9] = [0; 9];
    let mut Status: NTSTATUS;

    RtlMapGenericMask(&mut DesiredAccess, &raw mut AliasMapping);

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_CREATE_ALIAS,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountName(AccountName, 256);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountNameInDomain(DomainObject, (*AccountName).Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateAliasSD(&mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulSize = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            &mut FixedDomainData as *mut _ as PVOID,
            &mut ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulRid = FixedDomainData.NextRid;
        FixedDomainData.NextRid += 1;

        Status = SampSetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as LPWSTR,
            REG_BINARY,
            &mut FixedDomainData as *mut _ as PVOID,
            ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        swprintf(
            szRid.as_mut_ptr(),
            w!("%08lX").as_ptr() as LPWSTR,
            ulRid,
        );

        Status = SampCreateDbObject(
            DomainObject,
            w!("Aliases").as_ptr() as LPWSTR,
            szRid.as_mut_ptr(),
            ulRid,
            SamDbAliasObject,
            DesiredAccess,
            &mut AliasObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetAccountNameInDomain(
            DomainObject,
            w!("Aliases").as_ptr() as LPWSTR,
            (*AccountName).Buffer,
            ulRid,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            AliasObject,
            w!("Name").as_ptr() as LPWSTR,
            AccountName,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            AliasObject,
            w!("Description").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            AliasObject,
            w!("SecDesc").as_ptr() as LPWSTR,
            REG_BINARY,
            Sd,
            SdSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if NT_SUCCESS(Status) {
            *AliasHandle = AliasObject as SAMPR_HANDLE;
            *RelativeId = ulRid;
        }

        break 'done;
    }

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrEnumerateAliasesInDomain(
    DomainHandle: SAMPR_HANDLE,
    EnumerationContext: *mut core::ffi::c_ulong,
    Buffer: *mut PSAMPR_ENUMERATION_BUFFER,
    PreferedMaximumLength: core::ffi::c_ulong,
    CountReturned: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    let mut EnumBuffer: PSAMPR_ENUMERATION_BUFFER = core::ptr::null_mut();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut AliasesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut AliasName: [WCHAR; 64] = [0; 64];
    let mut EnumIndex: ULONG;
    let mut EnumCount: ULONG = 0;
    let mut RequiredLength: ULONG = 0;
    let mut NameLength: ULONG;
    let mut DataLength: ULONG;
    let mut Rid: ULONG = 0;
    let mut i: ULONG;
    let mut MoreEntries: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LIST_ACCOUNTS,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            (*DomainObject).KeyHandle,
            wstr!("Aliases"),
            KEY_READ,
            &mut AliasesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            AliasesKeyHandle,
            wstr!("Names"),
            KEY_READ,
            &mut NamesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumIndex = *EnumerationContext;

        loop {
            NameLength = (64 * core::mem::size_of::<WCHAR>()) as ULONG;
            Status = SampRegEnumerateValue(
                NamesKeyHandle,
                EnumIndex,
                AliasName.as_mut_ptr(),
                &mut NameLength,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            if (RequiredLength as usize
                + NameLength as usize
                + core::mem::size_of::<u16>()
                + core::mem::size_of::<SAMPR_RID_ENUMERATION>())
                > PreferedMaximumLength as usize
            {
                MoreEntries = 1u8;
                break;
            }

            RequiredLength += NameLength
                + core::mem::size_of::<u16>() as ULONG
                + core::mem::size_of::<SAMPR_RID_ENUMERATION>() as ULONG;
            EnumCount += 1;

            EnumIndex += 1;
        }

        if !NT_SUCCESS(Status) {
            break 'done;
        }

        EnumBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ENUMERATION_BUFFER>() as SIZE_T)
            as PSAMPR_ENUMERATION_BUFFER;
        if EnumBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*EnumBuffer).EntriesRead = EnumCount;
        if EnumCount == 0 {
            Status = STATUS_NO_MORE_ENTRIES;
            break 'done;
        }

        (*EnumBuffer).Buffer = midl_user_allocate(
            (EnumCount as usize * core::mem::size_of::<SAMPR_RID_ENUMERATION>()) as SIZE_T,
        ) as PSAMPR_RID_ENUMERATION;
        if (*EnumBuffer).Buffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        EnumIndex = *EnumerationContext;
        i = 0;
        while i < EnumCount {
            NameLength = (64 * core::mem::size_of::<WCHAR>()) as ULONG;
            DataLength = core::mem::size_of::<ULONG>() as ULONG;
            Status = SampRegEnumerateValue(
                NamesKeyHandle,
                EnumIndex,
                AliasName.as_mut_ptr(),
                &mut NameLength,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Rid) as PVOID,
                &mut DataLength,
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_NO_MORE_ENTRIES {
                    Status = STATUS_SUCCESS;
                }
                break;
            }

            (*(*EnumBuffer).Buffer.add(i as usize)).RelativeId = Rid;
            (*(*EnumBuffer).Buffer.add(i as usize)).Name.Length = NameLength as USHORT;
            (*(*EnumBuffer).Buffer.add(i as usize)).Name.MaximumLength =
                (NameLength + core::mem::size_of::<u16>() as ULONG) as USHORT;

            i += 1;
            EnumIndex += 1;
        }

        break 'done;
    }

    if NT_SUCCESS(Status) {
        *EnumerationContext += EnumCount;
        *Buffer = EnumBuffer;
        *CountReturned = EnumCount;
    } else {
        *EnumerationContext = 0;
        *Buffer = core::ptr::null_mut();
        *CountReturned = 0;

        if !EnumBuffer.is_null() {
            if !(*EnumBuffer).Buffer.is_null() {
                if (*EnumBuffer).EntriesRead != 0 {
                    i = 0;
                    while i < (*EnumBuffer).EntriesRead {
                        if !(*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer.is_null() {
                            midl_user_free(
                                (*(*EnumBuffer).Buffer.add(i as usize)).Name.Buffer
                                    as *mut core::ffi::c_void,
                            );
                        }
                        i += 1;
                    }
                }

                midl_user_free((*EnumBuffer).Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(EnumBuffer as *mut core::ffi::c_void);
        }
    }

    SampRegCloseKey(&mut NamesKeyHandle);
    SampRegCloseKey(&mut AliasesKeyHandle);

    if Status == STATUS_SUCCESS && MoreEntries != 0 {
        Status = STATUS_MORE_ENTRIES;
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetAliasMembership(
    DomainHandle: SAMPR_HANDLE,
    SidArray: PSAMPR_PSID_ARRAY,
    Membership: PSAMPR_ULONG_ARRAY,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut AliasesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MemberKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MemberSidString: LPWSTR = core::ptr::null_mut();
    let mut RidArray: PULONG = core::ptr::null_mut();
    let mut MaxSidCount: ULONG = 0;
    let mut ValueCount: ULONG = 0;
    let mut DataLength: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut RidIndex: ULONG;
    let mut Status: NTSTATUS;
    let mut NameBuffer: [WCHAR; 9] = [0; 9];

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_GET_ALIAS_MEMBERSHIP,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            (*DomainObject).KeyHandle,
            w!("Aliases").as_ptr() as LPCWSTR,
            KEY_READ,
            &mut AliasesKeyHandle,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRegOpenKey(
            AliasesKeyHandle,
            w!("Members").as_ptr() as LPCWSTR,
            KEY_READ,
            &mut MembersKeyHandle,
        );

        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            Status = STATUS_SUCCESS;
            break 'done;
        }

        if !NT_SUCCESS(Status) {
            break 'done;
        }

        i = 0;
        while i < (*SidArray).Count {
            ConvertSidToStringSid((*(*SidArray).Sids.offset(i as isize)).SidPointer as PSID, &mut MemberSidString);

            Status = SampRegOpenKey(
                MembersKeyHandle,
                MemberSidString,
                KEY_READ,
                &mut MemberKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegQueryKeyInfo(MemberKeyHandle, core::ptr::null_mut(), &mut ValueCount);
                if NT_SUCCESS(Status) {
                    MaxSidCount += ValueCount;
                }

                SampRegCloseKey(&mut MemberKeyHandle);
            }

            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }

            LocalFree(MemberSidString as *mut core::ffi::c_void);

            i += 1;
        }

        if MaxSidCount == 0 {
            Status = STATUS_SUCCESS;
            break 'done;
        }

        RidArray = midl_user_allocate(
            (MaxSidCount as usize * core::mem::size_of::<ULONG>()) as SIZE_T,
        ) as PULONG;
        if RidArray.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        RidIndex = 0;
        i = 0;
        while i < (*SidArray).Count {
            ConvertSidToStringSid((*(*SidArray).Sids.offset(i as isize)).SidPointer as PSID, &mut MemberSidString);

            Status = SampRegOpenKey(
                MembersKeyHandle,
                MemberSidString,
                KEY_READ,
                &mut MemberKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegQueryKeyInfo(MemberKeyHandle, core::ptr::null_mut(), &mut ValueCount);
                if NT_SUCCESS(Status) {
                    j = 0;
                    while j < ValueCount {
                        DataLength = 9 * core::mem::size_of::<WCHAR>() as ULONG;
                        Status = SampRegEnumerateValue(
                            MemberKeyHandle,
                            j,
                            NameBuffer.as_mut_ptr(),
                            &mut DataLength,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        );
                        if NT_SUCCESS(Status) {
                            *RidArray.offset(RidIndex as isize) =
                                wcstoul(NameBuffer.as_ptr(), core::ptr::null_mut(), 16);
                            RidIndex += 1;
                        }
                        j += 1;
                    }
                }

                SampRegCloseKey(&mut MemberKeyHandle);
            }

            if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                Status = STATUS_SUCCESS;
            }

            LocalFree(MemberSidString as *mut core::ffi::c_void);

            i += 1;
        }
    }

    SampRegCloseKey(&mut MembersKeyHandle);
    SampRegCloseKey(&mut AliasesKeyHandle);

    if NT_SUCCESS(Status) {
        (*Membership).Count = MaxSidCount;
        (*Membership).Element = RidArray;
    } else if !RidArray.is_null() {
        midl_user_free(RidArray as *mut core::ffi::c_void);
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrLookupNamesInDomain(
    DomainHandle: SAMPR_HANDLE,
    Count: ULONG,
    Names: *mut RPC_UNICODE_STRING,
    RelativeIds: PSAMPR_ULONG_ARRAY,
    Use: PSAMPR_ULONG_ARRAY,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut AccountsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut NamesKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MappedCount: ULONG = 0;
    let mut DataLength: ULONG;
    let mut i: ULONG;
    let mut RelativeId: ULONG;
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    Status = SampValidateDbObject(
        DomainHandle,
        SamDbDomainObject,
        DOMAIN_LOOKUP,
        &mut DomainObject,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*RelativeIds).Count = 0;
        (*Use).Count = 0;

        if Count == 0 {
            Status = STATUS_SUCCESS;
            break 'done;
        }

        (*RelativeIds).Element = midl_user_allocate(
            (Count as usize * core::mem::size_of::<ULONG>()) as SIZE_T,
        ) as *mut ULONG;
        if (*RelativeIds).Element.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*Use).Element = midl_user_allocate(
            (Count as usize * core::mem::size_of::<ULONG>()) as SIZE_T,
        ) as *mut ULONG;
        if (*Use).Element.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*RelativeIds).Count = Count;
        (*Use).Count = Count;

        i = 0;
        while i < Count {
            RelativeId = 0;

            Status = SampRegOpenKey(
                (*DomainObject).KeyHandle,
                wstr!("Aliases"),
                KEY_READ,
                &mut AccountsKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AccountsKeyHandle,
                    wstr!("Names"),
                    KEY_READ,
                    &mut NamesKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    DataLength = core::mem::size_of::<ULONG>() as ULONG;
                    Status = SampRegQueryValue(
                        NamesKeyHandle,
                        (*Names.add(i as usize)).Buffer,
                        core::ptr::null_mut(),
                        &mut RelativeId as *mut ULONG as *mut core::ffi::c_void,
                        &mut DataLength,
                    );

                    SampRegCloseKey(&mut NamesKeyHandle);
                }

                SampRegCloseKey(&mut AccountsKeyHandle);
            }

            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break;
            }

            if NT_SUCCESS(Status) && RelativeId != 0 {
                *(*RelativeIds).Element.add(i as usize) = RelativeId;
                *(*Use).Element.add(i as usize) = SidTypeAlias as ULONG;
                MappedCount += 1;
                i += 1;
                continue;
            }

            Status = SampRegOpenKey(
                (*DomainObject).KeyHandle,
                wstr!("Groups"),
                KEY_READ,
                &mut AccountsKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AccountsKeyHandle,
                    wstr!("Names"),
                    KEY_READ,
                    &mut NamesKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    DataLength = core::mem::size_of::<ULONG>() as ULONG;
                    Status = SampRegQueryValue(
                        NamesKeyHandle,
                        (*Names.add(i as usize)).Buffer,
                        core::ptr::null_mut(),
                        &mut RelativeId as *mut ULONG as *mut core::ffi::c_void,
                        &mut DataLength,
                    );

                    SampRegCloseKey(&mut NamesKeyHandle);
                }

                SampRegCloseKey(&mut AccountsKeyHandle);
            }

            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break;
            }

            if NT_SUCCESS(Status) && RelativeId != 0 {
                *(*RelativeIds).Element.add(i as usize) = RelativeId;
                *(*Use).Element.add(i as usize) = SidTypeGroup as ULONG;
                MappedCount += 1;
                i += 1;
                continue;
            }

            Status = SampRegOpenKey(
                (*DomainObject).KeyHandle,
                wstr!("Users"),
                KEY_READ,
                &mut AccountsKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AccountsKeyHandle,
                    wstr!("Names"),
                    KEY_READ,
                    &mut NamesKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    DataLength = core::mem::size_of::<ULONG>() as ULONG;
                    Status = SampRegQueryValue(
                        NamesKeyHandle,
                        (*Names.add(i as usize)).Buffer,
                        core::ptr::null_mut(),
                        &mut RelativeId as *mut ULONG as *mut core::ffi::c_void,
                        &mut DataLength,
                    );

                    SampRegCloseKey(&mut NamesKeyHandle);
                }

                SampRegCloseKey(&mut AccountsKeyHandle);
            }

            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break;
            }

            if NT_SUCCESS(Status) && RelativeId != 0 {
                *(*RelativeIds).Element.add(i as usize) = RelativeId;
                *(*Use).Element.add(i as usize) = SidTypeUser as ULONG;
                MappedCount += 1;
                i += 1;
                continue;
            }

            *(*RelativeIds).Element.add(i as usize) = 0;
            *(*Use).Element.add(i as usize) = SidTypeUnknown as ULONG;

            i += 1;
        }
    }

    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
        Status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(Status) {
        if MappedCount == 0 {
            Status = STATUS_NONE_MAPPED;
        } else if MappedCount < Count {
            Status = STATUS_SOME_NOT_MAPPED;
        }
    } else {
        if !(*RelativeIds).Element.is_null() {
            midl_user_free((*RelativeIds).Element as *mut core::ffi::c_void);
            (*RelativeIds).Element = core::ptr::null_mut();
        }

        (*RelativeIds).Count = 0;

        if !(*Use).Element.is_null() {
            midl_user_free((*Use).Element as *mut core::ffi::c_void);
            (*Use).Element = core::ptr::null_mut();
        }

        (*Use).Count = 0;
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrLookupIdsInDomain(
    DomainHandle: SAMPR_HANDLE,
    Count: ULONG,
    RelativeIds: *mut ULONG,
    Names: PSAMPR_RETURNED_USTRING_ARRAY,
    Use: PSAMPR_ULONG_ARRAY,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut RidString: [WCHAR; 9] = [0; 9];
    let mut AccountsKeyHandle: HANDLE = core::ptr::null_mut();
    let mut AccountKeyHandle: HANDLE = core::ptr::null_mut();
    let mut MappedCount: ULONG = 0;
    let mut DataLength: ULONG;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrLookupIdsInDomain(%p %lu %p %p %p)\n\0".as_ptr() as *const i8,
        DomainHandle,
        Count,
        RelativeIds,
        Names,
        Use,
    );

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LOOKUP,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        (*Names).Count = 0;
        (*Use).Count = 0;

        if Count == 0 {
            Status = STATUS_SUCCESS;
            break 'done;
        }

        (*Names).Element = midl_user_allocate(
            (Count as usize * core::mem::size_of::<RPC_UNICODE_STRING>()) as SIZE_T,
        ) as *mut RPC_UNICODE_STRING;
        if (*Names).Element.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*Use).Element = midl_user_allocate(
            (Count as usize * core::mem::size_of::<ULONG>()) as SIZE_T,
        ) as *mut ULONG;
        if (*Use).Element.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*Names).Count = Count;
        (*Use).Count = Count;

        i = 0;
        while i < Count {
            TRACE!(b"RID: %lu\n\0".as_ptr() as *const i8, *RelativeIds.add(i as usize));

            // C: swprintf(RidString, L"%08lX", RelativeIds[i]) — the previous two
            // attempts here were an ANSI-as-wide literal and an UNTERMINATED [u16; 5]
            swprintf(
                RidString.as_mut_ptr(),
                w!("%08lX").as_ptr(),
                *RelativeIds.add(i as usize),
            );

            let elem = (*Names).Element.add(i as usize);

            /* Lookup aliases */
            let aliases: [u16; 8] = [0x41, 0x6c, 0x69, 0x61, 0x73, 0x65, 0x73, 0];
            Status = SampRegOpenKey(
                (*DomainObject).KeyHandle,
                aliases.as_ptr(),
                KEY_READ,
                &mut AccountsKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AccountsKeyHandle,
                    RidString.as_ptr(),
                    KEY_READ,
                    &mut AccountKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    let name_val: [u16; 5] = [0x4e, 0x61, 0x6d, 0x65, 0];
                    DataLength = 0;
                    Status = SampRegQueryValue(
                        AccountKeyHandle,
                        name_val.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        &mut DataLength,
                    );
                    if NT_SUCCESS(Status) {
                        (*elem).Buffer = midl_user_allocate(DataLength as SIZE_T) as *mut u16;
                        if (*elem).Buffer.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        }

                        if NT_SUCCESS(Status) {
                            (*elem).MaximumLength = DataLength as USHORT;
                            (*elem).Length =
                                (DataLength - core::mem::size_of::<WCHAR>() as ULONG) as USHORT;

                            Status = SampRegQueryValue(
                                AccountKeyHandle,
                                name_val.as_ptr(),
                                core::ptr::null_mut(),
                                (*elem).Buffer as *mut core::ffi::c_void,
                                &mut DataLength,
                            );
                        }
                    }

                    SampRegCloseKey(&mut AccountKeyHandle);
                }

                SampRegCloseKey(&mut AccountsKeyHandle);
            }

            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break;
            }

            if NT_SUCCESS(Status) && !(*elem).Buffer.is_null() {
                TRACE!(b"Name: %S\n\0".as_ptr() as *const i8, (*elem).Buffer);
                *(*Use).Element.add(i as usize) = SidTypeAlias as ULONG;
                MappedCount += 1;
                i += 1;
                continue;
            }

            /* Lookup groups */
            let groups: [u16; 7] = [0x47, 0x72, 0x6f, 0x75, 0x70, 0x73, 0];
            Status = SampRegOpenKey(
                (*DomainObject).KeyHandle,
                groups.as_ptr(),
                KEY_READ,
                &mut AccountsKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AccountsKeyHandle,
                    RidString.as_ptr(),
                    KEY_READ,
                    &mut AccountKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    let name_val: [u16; 5] = [0x4e, 0x61, 0x6d, 0x65, 0];
                    DataLength = 0;
                    Status = SampRegQueryValue(
                        AccountKeyHandle,
                        name_val.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        &mut DataLength,
                    );
                    if NT_SUCCESS(Status) {
                        (*elem).Buffer = midl_user_allocate(DataLength as SIZE_T) as *mut u16;
                        if (*elem).Buffer.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        }

                        if NT_SUCCESS(Status) {
                            (*elem).MaximumLength = DataLength as USHORT;
                            (*elem).Length =
                                (DataLength - core::mem::size_of::<WCHAR>() as ULONG) as USHORT;

                            Status = SampRegQueryValue(
                                AccountKeyHandle,
                                name_val.as_ptr(),
                                core::ptr::null_mut(),
                                (*elem).Buffer as *mut core::ffi::c_void,
                                &mut DataLength,
                            );
                        }
                    }

                    SampRegCloseKey(&mut AccountKeyHandle);
                }

                SampRegCloseKey(&mut AccountsKeyHandle);
            }

            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break;
            }

            if NT_SUCCESS(Status) && !(*elem).Buffer.is_null() {
                TRACE!(b"Name: %S\n\0".as_ptr() as *const i8, (*elem).Buffer);
                *(*Use).Element.add(i as usize) = SidTypeGroup as ULONG;
                MappedCount += 1;
                i += 1;
                continue;
            }

            /* Lookup users */
            let users: [u16; 6] = [0x55, 0x73, 0x65, 0x72, 0x73, 0];
            Status = SampRegOpenKey(
                (*DomainObject).KeyHandle,
                users.as_ptr(),
                KEY_READ,
                &mut AccountsKeyHandle,
            );
            if NT_SUCCESS(Status) {
                Status = SampRegOpenKey(
                    AccountsKeyHandle,
                    RidString.as_ptr(),
                    KEY_READ,
                    &mut AccountKeyHandle,
                );
                if NT_SUCCESS(Status) {
                    let name_val: [u16; 5] = [0x4e, 0x61, 0x6d, 0x65, 0];
                    DataLength = 0;
                    Status = SampRegQueryValue(
                        AccountKeyHandle,
                        name_val.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        &mut DataLength,
                    );
                    if NT_SUCCESS(Status) {
                        TRACE!(b"DataLength: %lu\n\0".as_ptr() as *const i8, DataLength);

                        (*elem).Buffer = midl_user_allocate(DataLength as SIZE_T) as *mut u16;
                        if (*elem).Buffer.is_null() {
                            Status = STATUS_INSUFFICIENT_RESOURCES;
                        }

                        if NT_SUCCESS(Status) {
                            (*elem).MaximumLength = DataLength as USHORT;
                            (*elem).Length =
                                (DataLength - core::mem::size_of::<WCHAR>() as ULONG) as USHORT;

                            Status = SampRegQueryValue(
                                AccountKeyHandle,
                                name_val.as_ptr(),
                                core::ptr::null_mut(),
                                (*elem).Buffer as *mut core::ffi::c_void,
                                &mut DataLength,
                            );
                        }
                    }

                    SampRegCloseKey(&mut AccountKeyHandle);
                }

                SampRegCloseKey(&mut AccountsKeyHandle);
            }

            if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
                break;
            }

            if NT_SUCCESS(Status) && !(*elem).Buffer.is_null() {
                TRACE!(b"Name: %S\n\0".as_ptr() as *const i8, (*elem).Buffer);
                *(*Use).Element.add(i as usize) = SidTypeUser as ULONG;
                MappedCount += 1;
                i += 1;
                continue;
            }

            *(*Use).Element.add(i as usize) = SidTypeUnknown as ULONG;

            i += 1;
        }

        break 'done;
    }

    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
        Status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(Status) {
        if MappedCount == 0 {
            Status = STATUS_NONE_MAPPED;
        } else if MappedCount < Count {
            Status = STATUS_SOME_NOT_MAPPED;
        }
    } else {
        if !(*Names).Element.is_null() {
            i = 0;
            while i < Count {
                let elem = (*Names).Element.add(i as usize);
                if !(*elem).Buffer.is_null() {
                    midl_user_free((*elem).Buffer as *mut core::ffi::c_void);
                }
                i += 1;
            }

            midl_user_free((*Names).Element as *mut core::ffi::c_void);
            (*Names).Element = core::ptr::null_mut();
        }

        (*Names).Count = 0;

        if !(*Use).Element.is_null() {
            midl_user_free((*Use).Element as *mut core::ffi::c_void);
            (*Use).Element = core::ptr::null_mut();
        }

        (*Use).Count = 0;
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrOpenGroup(
    DomainHandle: SAMPR_HANDLE,
    mut DesiredAccess: ACCESS_MASK,
    GroupId: core::ffi::c_ulong,
    GroupHandle: *mut SAMPR_HANDLE,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut szRid: [u16; 9] = [0; 9];
    let mut Status: NTSTATUS;

    RtlMapGenericMask(&mut DesiredAccess, core::ptr::addr_of_mut!(GroupMapping));

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LOOKUP,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        swprintf(
            szRid.as_mut_ptr(),
            w!("%08lX").as_ptr(),
            GroupId,
        );

        Status = SampOpenDbObject(
            DomainObject,
            w!("Groups").as_ptr() as *const u16 as *mut u16,
            szRid.as_mut_ptr(),
            GroupId,
            SamDbGroupObject,
            DesiredAccess,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *GroupHandle = GroupObject as SAMPR_HANDLE;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}
unsafe fn SampQueryGroupGeneral(
    GroupObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_GROUP_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_GROUP_INFO_BUFFER;
    let mut FixedData: SAM_GROUP_FIXED_DATA = core::mem::zeroed();
    let mut MembersLength: ULONG = 0;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_GROUP_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_GROUP_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: loop {
        Status = SampGetObjectAttributeString(
            GroupObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General.Name),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            GroupObject,
            w!("AdminComment").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General.AdminComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Length = core::mem::size_of::<SAM_GROUP_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            GroupObject,
            w!("F").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        (*InfoBuffer).General.Attributes = FixedData.Attributes;

        Status = SampGetObjectAttribute(
            GroupObject,
            w!("Members").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut MembersLength,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            (*InfoBuffer).General.MemberCount = 0;
            Status = STATUS_SUCCESS;
        } else {
            (*InfoBuffer).General.MemberCount =
                MembersLength / core::mem::size_of::<ULONG>() as ULONG;
        }

        *Buffer = InfoBuffer;

        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).General.Name.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.Name.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.AdminComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryGroupName(GroupObject: PSAM_DB_OBJECT,
                            Buffer: *mut PSAMPR_GROUP_INFO_BUFFER) -> NTSTATUS {
    let InfoBuffer: PSAMPR_GROUP_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_GROUP_INFO_BUFFER>() as SIZE_T) as PSAMPR_GROUP_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(GroupObject,
                                          w!("Name").as_ptr() as LPWSTR,
                                          core::ptr::addr_of_mut!((*InfoBuffer).Name.Name));
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Name.Name.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Name.Name.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryGroupAttribute(
    GroupObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_GROUP_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_GROUP_INFO_BUFFER;
    let mut FixedData: SAM_GROUP_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_GROUP_INFO_BUFFER>() as SIZE_T) as PSAMPR_GROUP_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_GROUP_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        GroupObject,
        w!("F").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );

    'done: {
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        (*InfoBuffer).Attribute.Attributes = FixedData.Attributes;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryGroupAdminComment(
    GroupObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_GROUP_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_GROUP_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_GROUP_INFO_BUFFER>() as SIZE_T) as PSAMPR_GROUP_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        GroupObject,
        w!("AdminComment").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).AdminComment.AdminComment),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).AdminComment.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).AdminComment.AdminComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryInformationGroup(
    GroupHandle: SAMPR_HANDLE,
    GroupInformationClass: GROUP_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_GROUP_INFO_BUFFER,
) -> NTSTATUS {
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            GroupHandle,
            SamDbGroupObject,
            GROUP_READ_INFORMATION,
            core::ptr::addr_of_mut!(GroupObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        match GroupInformationClass {
            GroupGeneralInformation => {
                Status = SampQueryGroupGeneral(GroupObject, Buffer);
            }
            GroupNameInformation => {
                Status = SampQueryGroupName(GroupObject, Buffer);
            }
            GroupAttributeInformation => {
                Status = SampQueryGroupAttribute(GroupObject, Buffer);
            }
            GroupAdminCommentInformation => {
                Status = SampQueryGroupAdminComment(GroupObject, Buffer);
            }
            _ => {
                Status = STATUS_INVALID_INFO_CLASS;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

unsafe fn SampSetGroupName(GroupObject: PSAM_DB_OBJECT,
                           Buffer: PSAMPR_GROUP_INFO_BUFFER) -> NTSTATUS {
    let mut OldGroupName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut NewGroupName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    'done: {
        Status = SampGetObjectAttributeString(GroupObject,
                                              wchar_str!("Name"),
                                              core::ptr::addr_of_mut!(OldGroupName) as PRPC_UNICODE_STRING);
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampGetObjectAttributeString failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        Status = SampCheckAccountName(core::ptr::addr_of_mut!((*Buffer).Name.Name), 256);
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampCheckAccountName failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            return Status;
        }

        NewGroupName.Length = (*Buffer).Name.Name.Length;
        NewGroupName.MaximumLength = (*Buffer).Name.Name.MaximumLength;
        NewGroupName.Buffer = (*Buffer).Name.Name.Buffer;

        if RtlEqualUnicodeString(core::ptr::addr_of_mut!(OldGroupName),
                                 core::ptr::addr_of_mut!(NewGroupName), 1u8) == 0 {
            Status = SampCheckAccountNameInDomain((*GroupObject).ParentObject,
                                                  NewGroupName.Buffer);
            if !NT_SUCCESS(Status) {
                TRACE!(b"Group name '%S' already exists in domain (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                      NewGroupName.Buffer, Status);
                break 'done;
            }
        }

        Status = SampSetAccountNameInDomain((*GroupObject).ParentObject,
                                            wchar_str!("Groups"),
                                            NewGroupName.Buffer,
                                            (*GroupObject).RelativeId);
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampSetAccountNameInDomain failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        Status = SampRemoveAccountNameFromDomain((*GroupObject).ParentObject,
                                                 wchar_str!("Groups"),
                                                 OldGroupName.Buffer);
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampRemoveAccountNameFromDomain failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        Status = SampSetObjectAttributeString(GroupObject,
                                              wchar_str!("Name"),
                                              core::ptr::addr_of_mut!(NewGroupName) as PRPC_UNICODE_STRING);
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampSetObjectAttribute failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
        }
    }

    if !OldGroupName.Buffer.is_null() {
        midl_user_free(OldGroupName.Buffer as *mut core::ffi::c_void);
    }

    Status
}

unsafe fn SampSetGroupAttribute(GroupObject: PSAM_DB_OBJECT,
                                  Buffer: PSAMPR_GROUP_INFO_BUFFER) -> NTSTATUS {
    let mut FixedData: SAM_GROUP_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_GROUP_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(GroupObject,
                                    w!("F").as_ptr() as *const u16 as LPWSTR,
                                    core::ptr::null_mut(),
                                    core::ptr::addr_of_mut!(FixedData) as PVOID,
                                    core::ptr::addr_of_mut!(Length));

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.Attributes = (*Buffer).Attribute.Attributes;

        Status = SampSetObjectAttribute(GroupObject,
                                        w!("F").as_ptr() as *const u16 as LPWSTR,
                                        REG_BINARY,
                                        core::ptr::addr_of_mut!(FixedData) as PVOID,
                                        Length);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetInformationGroup(
    GroupHandle: SAMPR_HANDLE,
    GroupInformationClass: GROUP_INFORMATION_CLASS,
    Buffer: PSAMPR_GROUP_INFO_BUFFER,
) -> NTSTATUS {
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrSetInformationGroup(%p %lu %p)\n\0".as_ptr() as *const i8,
        GroupHandle,
        GroupInformationClass,
        Buffer,
    );

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            GroupHandle,
            SamDbGroupObject,
            GROUP_WRITE_ACCOUNT,
            core::ptr::addr_of_mut!(GroupObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        match GroupInformationClass {
            GroupNameInformation => {
                Status = SampSetGroupName(GroupObject, Buffer);
            }
            GroupAttributeInformation => {
                Status = SampSetGroupAttribute(GroupObject, Buffer);
            }
            GroupAdminCommentInformation => {
                Status = SampSetObjectAttributeString(
                    GroupObject,
                    w!("AdminComment").as_ptr() as LPWSTR,
                    core::ptr::addr_of_mut!((*Buffer).AdminComment.AdminComment),
                );
            }
            _ => {
                Status = STATUS_INVALID_INFO_CLASS;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrAddMemberToGroup(
    GroupHandle: SAMPR_HANDLE,
    MemberId: core::ffi::c_ulong,
    Attributes: core::ffi::c_ulong,
) -> NTSTATUS {
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            GroupHandle,
            SamDbGroupObject,
            GROUP_ADD_MEMBER,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampOpenUserObject(
            (*GroupObject).ParentObject,
            MemberId,
            0,
            &mut UserObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampAddGroupMembershipToUser(
            UserObject,
            (*GroupObject).RelativeId,
            Attributes,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampAddMemberToGroup(GroupObject, MemberId);
        if !NT_SUCCESS(Status) {}
    }

    if !UserObject.is_null() {
        SampCloseDbObject(UserObject);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrDeleteGroup(GroupHandle: *mut SAMPR_HANDLE) -> NTSTATUS {
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            *GroupHandle,
            SamDbGroupObject,
            DELETE,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if (*GroupObject).RelativeId < 1000 {
            Status = STATUS_SPECIAL_ACCOUNT;
            break 'done;
        }

        SampGetObjectAttribute(
            GroupObject,
            w!("Members").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );

        if Length != 0 {
            Status = STATUS_MEMBER_IN_GROUP;
            break 'done;
        }

        Status = SampDeleteAccountDbObject(GroupObject);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *GroupHandle = core::ptr::null_mut();
        break 'done;
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrRemoveMemberFromGroup(
    GroupHandle: SAMPR_HANDLE,
    MemberId: core::ffi::c_ulong,
) -> NTSTATUS {
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        /* Validate the group handle */
        Status = SampValidateDbObject(
            GroupHandle,
            SamDbGroupObject,
            GROUP_REMOVE_MEMBER,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Open the user object in the same domain */
        Status = SampOpenUserObject(
            (*GroupObject).ParentObject,
            MemberId,
            0,
            &mut UserObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Remove group membership from the user object */
        Status = SampRemoveGroupMembershipFromUser(UserObject, (*GroupObject).RelativeId);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Remove the member from the group object */
        Status = SampRemoveMemberFromGroup(GroupObject, MemberId);
        if !NT_SUCCESS(Status) {}
    }

    if !UserObject.is_null() {
        SampCloseDbObject(UserObject);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetMembersInGroup(
    GroupHandle: SAMPR_HANDLE,
    Members: *mut PSAMPR_GET_MEMBERS_BUFFER,
) -> NTSTATUS {
    let mut MembersBuffer: PSAMPR_GET_MEMBERS_BUFFER = core::ptr::null_mut();
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    TRACE!(b"SamrGetMembersInGroup(%p %p)\n\0".as_ptr() as *const i8, GroupHandle, Members);

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            GroupHandle,
            SamDbGroupObject,
            GROUP_LIST_MEMBERS,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        MembersBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_GET_MEMBERS_BUFFER>() as SIZE_T) as PSAMPR_GET_MEMBERS_BUFFER;
        if MembersBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        SampGetObjectAttribute(
            GroupObject,
            ['M' as u16, 'e' as u16, 'm' as u16, 'b' as u16, 'e' as u16, 'r' as u16, 's' as u16, 0].as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );

        if Length == 0 {
            (*MembersBuffer).MemberCount = 0;
            (*MembersBuffer).Members = core::ptr::null_mut();
            (*MembersBuffer).Attributes = core::ptr::null_mut();

            *Members = MembersBuffer;

            Status = STATUS_SUCCESS;
            break 'done;
        }

        (*MembersBuffer).Members = midl_user_allocate(Length as SIZE_T) as *mut ULONG;
        if (*MembersBuffer).Members.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        (*MembersBuffer).Attributes = midl_user_allocate(Length as SIZE_T) as *mut ULONG;
        if (*MembersBuffer).Attributes.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            GroupObject,
            ['M' as u16, 'e' as u16, 'm' as u16, 'b' as u16, 'e' as u16, 'r' as u16, 's' as u16, 0].as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            (*MembersBuffer).Members as *mut core::ffi::c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampGetObjectAttributes() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        (*MembersBuffer).MemberCount = Length / core::mem::size_of::<ULONG>() as ULONG;

        i = 0;
        while i < (*MembersBuffer).MemberCount {
            Status = SampGetUserGroupAttributes(
                (*GroupObject).ParentObject,
                *(*MembersBuffer).Members.add(i as usize),
                (*GroupObject).RelativeId,
                (*MembersBuffer).Attributes.add(i as usize),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"SampGetUserGroupAttributes() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
            i += 1;
        }

        *Members = MembersBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !MembersBuffer.is_null() {
            if !(*MembersBuffer).Members.is_null() {
                midl_user_free((*MembersBuffer).Members as *mut core::ffi::c_void);
            }

            if !(*MembersBuffer).Attributes.is_null() {
                midl_user_free((*MembersBuffer).Attributes as *mut core::ffi::c_void);
            }

            midl_user_free(MembersBuffer as *mut core::ffi::c_void);
        }
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetMemberAttributesOfGroup(
    GroupHandle: SAMPR_HANDLE,
    MemberId: ::core::ffi::c_ulong,
    Attributes: ::core::ffi::c_ulong,
) -> NTSTATUS {
    let mut GroupObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrSetMemberAttributesOfGroup(%p %lu %lx)\n\0".as_ptr() as *const i8,
        GroupHandle,
        MemberId,
        Attributes,
    );

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            GroupHandle,
            SamDbGroupObject,
            GROUP_ADD_MEMBER,
            &mut GroupObject,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SampValidateDbObject failed with status 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        Status = SampSetUserGroupAttributes(
            (*GroupObject).ParentObject,
            MemberId,
            (*GroupObject).RelativeId,
            Attributes,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SampSetUserGroupAttributes failed with status 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
        }
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrOpenAlias(
    DomainHandle: SAMPR_HANDLE,
    mut DesiredAccess: ACCESS_MASK,
    AliasId: ULONG,
    AliasHandle: *mut SAMPR_HANDLE,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut szRid: [WCHAR; 9] = [0; 9];
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrOpenAlias(%p %lx %lx %p)\n\0".as_ptr() as *const i8,
        DomainHandle,
        DesiredAccess,
        AliasId,
        AliasHandle,
    );

    RtlMapGenericMask(
        core::ptr::addr_of_mut!(DesiredAccess),
        core::ptr::addr_of_mut!(AliasMapping),
    );

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LOOKUP,
            core::ptr::addr_of_mut!(DomainObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        swprintf(
            szRid.as_mut_ptr(),
            w!("%08lX").as_ptr() as *const u16,
            AliasId,
        );

        Status = SampOpenDbObject(
            DomainObject,
            w!("Aliases").as_ptr() as *mut u16,
            szRid.as_mut_ptr(),
            AliasId,
            SamDbAliasObject,
            DesiredAccess,
            core::ptr::addr_of_mut!(AliasObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        *AliasHandle = AliasObject as SAMPR_HANDLE;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}
unsafe fn SampQueryAliasGeneral(
    AliasObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_ALIAS_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_ALIAS_INFO_BUFFER;
    let mut MembersKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ALIAS_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_ALIAS_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = SampGetObjectAttributeString(
            AliasObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General.Name),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            AliasObject,
            w!("Description").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).General.AdminComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampRegOpenKey(
            (*AliasObject).KeyHandle,
            w!("Members").as_ptr() as LPWSTR,
            KEY_READ,
            &mut MembersKeyHandle,
        );
        if NT_SUCCESS(Status) {
            Status = SampRegQueryKeyInfo(
                MembersKeyHandle,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*InfoBuffer).General.MemberCount),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
                break 'done;
            }
        } else if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            (*InfoBuffer).General.MemberCount = 0;
            Status = STATUS_SUCCESS;
        } else {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    SampRegCloseKey(&mut MembersKeyHandle);

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).General.Name.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.Name.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.AdminComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryAliasName(
    AliasObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_ALIAS_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_ALIAS_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ALIAS_INFO_BUFFER>() as SIZE_T) as PSAMPR_ALIAS_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = SampGetObjectAttributeString(
            AliasObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Name.Name),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Name.Name.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Name.Name.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryAliasAdminComment(
    AliasObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_ALIAS_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_ALIAS_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_ALIAS_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_ALIAS_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = SampGetObjectAttributeString(
            AliasObject,
            w!("Description").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).AdminComment.AdminComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).AdminComment.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).AdminComment.AdminComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryInformationAlias(
    AliasHandle: SAMPR_HANDLE,
    AliasInformationClass: ALIAS_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_ALIAS_INFO_BUFFER,
) -> NTSTATUS {
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrQueryInformationAlias(%p %lu %p)\n\0".as_ptr() as *const i8,
        AliasHandle,
        AliasInformationClass,
        Buffer,
    );

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            AliasHandle,
            SamDbAliasObject,
            ALIAS_READ_INFORMATION,
            core::ptr::addr_of_mut!(AliasObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        match AliasInformationClass {
            AliasGeneralInformation => {
                Status = SampQueryAliasGeneral(AliasObject, Buffer);
            }
            AliasNameInformation => {
                Status = SampQueryAliasName(AliasObject, Buffer);
            }
            AliasAdminCommentInformation => {
                Status = SampQueryAliasAdminComment(AliasObject, Buffer);
            }
            _ => {
                Status = STATUS_INVALID_INFO_CLASS;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

unsafe fn SampSetAliasName(AliasObject: PSAM_DB_OBJECT,
                            Buffer: PSAMPR_ALIAS_INFO_BUFFER) -> NTSTATUS {
    let mut OldAliasName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut NewAliasName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    'done: {
        Status = SampGetObjectAttributeString(
            AliasObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!(OldAliasName) as PRPC_UNICODE_STRING);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountName(core::ptr::addr_of_mut!((*Buffer).Name.Name) as *mut _, 256);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        NewAliasName.Length = (*Buffer).Name.Name.Length;
        NewAliasName.MaximumLength = (*Buffer).Name.Name.MaximumLength;
        NewAliasName.Buffer = (*Buffer).Name.Name.Buffer;

        if RtlEqualUnicodeString(core::ptr::addr_of_mut!(OldAliasName),
                                 core::ptr::addr_of_mut!(NewAliasName),
                                 1u8) == 0 {
            Status = SampCheckAccountNameInDomain((*AliasObject).ParentObject,
                                                  NewAliasName.Buffer);
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        Status = SampSetAccountNameInDomain((*AliasObject).ParentObject,
                                            w!("Aliases").as_ptr() as LPWSTR,
                                            NewAliasName.Buffer,
                                            (*AliasObject).RelativeId);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRemoveAccountNameFromDomain((*AliasObject).ParentObject,
                                                 w!("Aliases").as_ptr() as LPWSTR,
                                                 OldAliasName.Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            AliasObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!(NewAliasName) as PRPC_UNICODE_STRING);
    }

    if !OldAliasName.Buffer.is_null() {
        midl_user_free(OldAliasName.Buffer as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetInformationAlias(
    AliasHandle: SAMPR_HANDLE,
    AliasInformationClass: ALIAS_INFORMATION_CLASS,
    Buffer: PSAMPR_ALIAS_INFO_BUFFER,
) -> NTSTATUS {
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrSetInformationAlias(%p %lu %p)\n\0".as_ptr() as *const i8,
        AliasHandle,
        AliasInformationClass,
        Buffer,
    );

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            AliasHandle,
            SamDbAliasObject,
            ALIAS_WRITE_ACCOUNT,
            &mut AliasObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        match AliasInformationClass {
            AliasNameInformation => {
                Status = SampSetAliasName(AliasObject, Buffer);
            }
            AliasAdminCommentInformation => {
                Status = SampSetObjectAttributeString(
                    AliasObject,
                    w!("Description").as_ptr() as LPWSTR,
                    core::ptr::addr_of_mut!((*Buffer).AdminComment.AdminComment),
                );
            }
            _ => {
                Status = STATUS_INVALID_INFO_CLASS;
            }
        }
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrDeleteAlias(AliasHandle: *mut SAMPR_HANDLE) -> NTSTATUS {
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            *AliasHandle,
            SamDbAliasObject,
            DELETE,
            core::ptr::addr_of_mut!(AliasObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if (*AliasObject).RelativeId < 1000 {
            Status = STATUS_SPECIAL_ACCOUNT;
            break 'done;
        }

        Status = SampRemoveAllMembersFromAlias(AliasObject);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampDeleteAccountDbObject(AliasObject);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *AliasHandle = core::ptr::null_mut();
        break 'done;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrAddMemberToAlias(
    AliasHandle: SAMPR_HANDLE,
    MemberId: PRPC_SID,
) -> NTSTATUS {
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    Status = SampValidateDbObject(
        AliasHandle,
        SamDbAliasObject,
        ALIAS_ADD_MEMBER,
        core::ptr::addr_of_mut!(AliasObject),
    );
    if !NT_SUCCESS(Status) {
        RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));
        return Status;
    }

    Status = SampAddMemberToAlias(AliasObject, MemberId);

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrRemoveMemberFromAlias(
    AliasHandle: SAMPR_HANDLE,
    MemberId: PRPC_SID,
) -> NTSTATUS {
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            AliasHandle,
            SamDbAliasObject,
            ALIAS_REMOVE_MEMBER,
            core::ptr::addr_of_mut!(AliasObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRemoveMemberFromAlias(AliasObject, MemberId);
        if !NT_SUCCESS(Status) {}
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetMembersInAlias(
    AliasHandle: SAMPR_HANDLE,
    Members: PSAMPR_PSID_ARRAY_OUT,
) -> NTSTATUS {
    let mut AliasObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut MemberArray: PSAMPR_SID_INFORMATION = core::ptr::null_mut();
    let mut MemberCount: ULONG = 0;
    let _Index: ULONG;
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrGetMembersInAlias(%p %p %p)\n\0".as_ptr() as *const core::ffi::c_char,
        AliasHandle,
        Members,
    );

    RtlAcquireResourceShared(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            AliasHandle,
            SamDbAliasObject,
            ALIAS_LIST_MEMBERS,
            &mut AliasObject,
        );
        if !NT_SUCCESS(Status) {
            ERR!(
                "failed with status 0x%08lx\n",
                Status,
            );
            break 'done;
        }

        Status = SampGetMembersInAlias(AliasObject, &mut MemberCount, &mut MemberArray);

        if NT_SUCCESS(Status) {
            (*Members).Count = MemberCount;
            (*Members).Sids = MemberArray;
        }
    }

    if !NT_SUCCESS(Status) {
        if !MemberArray.is_null() {
            let mut Index: ULONG = 0;
            while Index < MemberCount {
                if !(*MemberArray.add(Index as usize)).SidPointer.is_null() {
                    midl_user_free((*MemberArray.add(Index as usize)).SidPointer as *mut core::ffi::c_void);
                }
                Index += 1;
            }

            midl_user_free(MemberArray as *mut core::ffi::c_void);
        }
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrOpenUser(
    DomainHandle: SAMPR_HANDLE,
    mut DesiredAccess: ACCESS_MASK,
    UserId: core::ffi::c_ulong,
    UserHandle: *mut SAMPR_HANDLE,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut szRid: [u16; 9] = [0; 9];
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrOpenUser(%p %lx %lx %p)\n\0".as_ptr() as *const i8,
        DomainHandle,
        DesiredAccess,
        UserId,
        UserHandle,
    );

    RtlMapGenericMask(
        core::ptr::addr_of_mut!(DesiredAccess),
        core::ptr::addr_of_mut!(UserMapping),
    );

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LOOKUP,
            core::ptr::addr_of_mut!(DomainObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        swprintf(
            szRid.as_mut_ptr(),
            ['%' as u16, '0' as u16, '8' as u16, 'l' as u16, 'X' as u16, 0u16].as_ptr(),
            UserId,
        );

        Status = SampOpenDbObject(
            DomainObject,
            ['U' as u16, 's' as u16, 'e' as u16, 'r' as u16, 's' as u16, 0u16].as_ptr() as *mut u16,
            szRid.as_ptr() as *mut u16,
            UserId,
            SamDbUserObject,
            DesiredAccess,
            core::ptr::addr_of_mut!(UserObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        *UserHandle = UserObject as SAMPR_HANDLE;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrDeleteUser(UserHandle: *mut SAMPR_HANDLE) -> NTSTATUS {
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            *UserHandle,
            SamDbUserObject,
            DELETE,
            core::ptr::addr_of_mut!(UserObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if (*UserObject).RelativeId < 1000 {
            Status = STATUS_SPECIAL_ACCOUNT;
            break 'done;
        }

        Status = SampRemoveUserFromAllGroups(UserObject);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRemoveUserFromAllAliases(UserObject);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampDeleteAccountDbObject(UserObject);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *UserHandle = core::ptr::null_mut();
        break 'done;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

unsafe fn SampQueryUserGeneral(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F"),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).General.PrimaryGroupId = FixedData.PrimaryGroupId;

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("Name"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.UserName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("FullName"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.FullName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("AdminComment"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.AdminComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("UserComment"),
            core::ptr::addr_of_mut!((*InfoBuffer).General.UserComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).General.UserName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.UserName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.FullName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.FullName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.AdminComment.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).General.UserComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).General.UserComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserPreferences(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Preferences.CountryCode = FixedData.CountryCode;
        (*InfoBuffer).Preferences.CodePage = FixedData.CodePage;

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("UserComment") as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Preferences.UserComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Preferences.UserComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Preferences.UserComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserLogon(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut DomainFixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let PasswordCanChange: LARGE_INTEGER;
    let PasswordMustChange: LARGE_INTEGER;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            (*UserObject).ParentObject,
            wstr!("F"),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(DomainFixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F"),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Logon.UserId = FixedData.UserId;
        (*InfoBuffer).Logon.PrimaryGroupId = FixedData.PrimaryGroupId;
        (*InfoBuffer).Logon.LastLogon.LowPart = FixedData.LastLogon.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Logon.LastLogon.HighPart = FixedData.LastLogon.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Logon.LastLogoff.LowPart = FixedData.LastLogoff.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Logon.LastLogoff.HighPart = FixedData.LastLogoff.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Logon.PasswordLastSet.LowPart = FixedData.PasswordLastSet.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Logon.PasswordLastSet.HighPart = FixedData.PasswordLastSet.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Logon.BadPasswordCount = FixedData.BadPasswordCount;
        (*InfoBuffer).Logon.LogonCount = FixedData.LogonCount;
        (*InfoBuffer).Logon.UserAccountControl = FixedData.UserAccountControl;

        PasswordCanChange = SampAddRelativeTimeToTime(FixedData.PasswordLastSet, DomainFixedData.MinPasswordAge);
        (*InfoBuffer).Logon.PasswordCanChange.LowPart = PasswordCanChange.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Logon.PasswordCanChange.HighPart = PasswordCanChange.__bindgen_anon_1.HighPart;

        PasswordMustChange = SampAddRelativeTimeToTime(FixedData.PasswordLastSet, DomainFixedData.MaxPasswordAge);
        (*InfoBuffer).Logon.PasswordMustChange.LowPart = PasswordMustChange.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Logon.PasswordMustChange.HighPart = PasswordMustChange.__bindgen_anon_1.HighPart;

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("Name"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.UserName),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("FullName"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.FullName),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("HomeDirectory"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.HomeDirectory),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("HomeDirectoryDrive"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.HomeDirectoryDrive),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("ScriptPath"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.ScriptPath),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("ProfilePath"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.ProfilePath),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            wstr!("WorkStations"),
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.WorkStations),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampGetLogonHoursAttribute(
            UserObject,
            core::ptr::addr_of_mut!((*InfoBuffer).Logon.LogonHours),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Logon.UserName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.UserName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.FullName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.FullName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.HomeDirectory.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.HomeDirectory.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.HomeDirectoryDrive.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.HomeDirectoryDrive.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.ScriptPath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.ScriptPath.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.ProfilePath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.ProfilePath.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.WorkStations.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Logon.WorkStations.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Logon.LogonHours.LogonHours.is_null() {
                midl_user_free((*InfoBuffer).Logon.LogonHours.LogonHours as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserAccount(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        UserObject,
        w!("F").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        &mut Length,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Account.UserId = FixedData.UserId;
        (*InfoBuffer).Account.PrimaryGroupId = FixedData.PrimaryGroupId;
        (*InfoBuffer).Account.LastLogon.LowPart = FixedData.LastLogon.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Account.LastLogon.HighPart = FixedData.LastLogon.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Account.LastLogoff.LowPart = FixedData.LastLogoff.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Account.LastLogoff.HighPart = FixedData.LastLogoff.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Account.PasswordLastSet.LowPart = FixedData.PasswordLastSet.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Account.PasswordLastSet.HighPart = FixedData.PasswordLastSet.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Account.AccountExpires.LowPart = FixedData.AccountExpires.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Account.AccountExpires.HighPart = FixedData.AccountExpires.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Account.BadPasswordCount = FixedData.BadPasswordCount;
        (*InfoBuffer).Account.LogonCount = FixedData.LogonCount;
        (*InfoBuffer).Account.UserAccountControl = FixedData.UserAccountControl;

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.UserName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("FullName").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.FullName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("HomeDirectory").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.HomeDirectory),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("HomeDirectoryDrive").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.HomeDirectoryDrive),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("ScriptPath").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.ScriptPath),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("ProfilePath").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.ProfilePath),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("AdminComment").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.AdminComment),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("WorkStations").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.WorkStations),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        Status = SampGetLogonHoursAttribute(
            UserObject,
            core::ptr::addr_of_mut!((*InfoBuffer).Account.LogonHours),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Account.UserName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.UserName.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.FullName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.FullName.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.HomeDirectory.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.HomeDirectory.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.HomeDirectoryDrive.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.HomeDirectoryDrive.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.ScriptPath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.ScriptPath.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.ProfilePath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.ProfilePath.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.AdminComment.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.WorkStations.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Account.WorkStations.Buffer as *mut core::ffi::c_void);
            }
            if !(*InfoBuffer).Account.LogonHours.LogonHours.is_null() {
                midl_user_free((*InfoBuffer).Account.LogonHours.LogonHours as *mut core::ffi::c_void);
            }
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryUserLogonHours(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut *mut SAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: *mut SAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as *mut SAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetLogonHoursAttribute(
        UserObject,
        core::ptr::addr_of_mut!((*InfoBuffer).LogonHours.LogonHours),
    );
    if NT_SUCCESS(Status) {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).LogonHours.LogonHours.LogonHours.is_null() {
                midl_user_free((*InfoBuffer).LogonHours.LogonHours.LogonHours as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserName(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: loop {
        Status = SampGetObjectAttributeString(
            UserObject,
            w!("Name").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Name.UserName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("FullName").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Name.FullName),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
        break 'done;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Name.UserName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Name.UserName.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Name.FullName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Name.FullName.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserAccountName(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("Name").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).AccountName.UserName),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).AccountName.UserName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).AccountName.UserName.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryUserFullName(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("FullName").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).FullName.FullName),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).FullName.FullName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).FullName.FullName.Buffer as *mut core::ffi::c_void);
            }
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserPrimaryGroup(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        UserObject,
        w!("F").as_ptr() as *const u16 as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as *mut core::ffi::c_void,
        &mut Length,
    );

    if NT_SUCCESS(Status) {
        (*InfoBuffer).PrimaryGroup.PrimaryGroupId = FixedData.PrimaryGroupId;
        *Buffer = InfoBuffer;
    } else {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserHome(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = SampGetObjectAttributeString(
            UserObject,
            w!("HomeDirectory").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Home.HomeDirectory),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        Status = SampGetObjectAttributeString(
            UserObject,
            w!("HomeDirectoryDrive").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*InfoBuffer).Home.HomeDirectoryDrive),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
            break 'done;
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Home.HomeDirectory.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Home.HomeDirectory.Buffer as *mut core::ffi::c_void);
            }

            if !(*InfoBuffer).Home.HomeDirectoryDrive.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Home.HomeDirectoryDrive.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserScript(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("ScriptPath").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).Script.ScriptPath),
    );
    if NT_SUCCESS(Status) {
        *Buffer = InfoBuffer;
    } else {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Script.ScriptPath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Script.ScriptPath.Buffer as *mut core::ffi::c_void);
            }
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserProfile(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("ProfilePath").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).Profile.ProfilePath),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Profile.ProfilePath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Profile.ProfilePath.Buffer as *mut core::ffi::c_void);
            }
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryUserAdminComment(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("AdminComment").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).AdminComment.AdminComment),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).AdminComment.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).AdminComment.AdminComment.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryUserWorkStations(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("WorkStations").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).WorkStations.WorkStations),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr(), Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).WorkStations.WorkStations.Buffer.is_null() {
                midl_user_free((*InfoBuffer).WorkStations.WorkStations.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserControl(UserObject: PSAM_DB_OBJECT, Buffer: *mut PSAMPR_USER_INFO_BUFFER) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(UserObject,
                                    w!("F").as_ptr() as *const u16 as LPWSTR,
                                    core::ptr::null_mut(),
                                    core::ptr::addr_of_mut!(FixedData) as PVOID,
                                    core::ptr::addr_of_mut!(Length));
    if NT_SUCCESS(Status) {
        (*InfoBuffer).Control.UserAccountControl = FixedData.UserAccountControl;
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserExpires(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Expires.AccountExpires.LowPart = FixedData.AccountExpires.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Expires.AccountExpires.HighPart = FixedData.AccountExpires.__bindgen_anon_1.HighPart;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserInternal1(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut Length: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*UserObject).Trusted == 0 {
        return STATUS_INVALID_INFO_CLASS;
    }

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*InfoBuffer).Internal1.LmPasswordPresent = 0;
    (*InfoBuffer).Internal1.NtPasswordPresent = 0;

    'done: {
        /* Get the NT password */
        Length = 0;
        SampGetObjectAttribute(
            UserObject,
            wstr!("NTPwd") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );

        if Length as usize == core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() {
            Status = SampGetObjectAttribute(
                UserObject,
                wstr!("NTPwd") as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*InfoBuffer).Internal1.EncryptedNtOwfPassword) as PVOID,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            if memcmp(
                core::ptr::addr_of!((*InfoBuffer).Internal1.EncryptedNtOwfPassword) as *const core::ffi::c_void,
                core::ptr::addr_of!(EmptyNtHash) as *const core::ffi::c_void,
                core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>(),
            ) != 0
            {
                (*InfoBuffer).Internal1.NtPasswordPresent = 1;
            }
        }

        /* Get the LM password */
        Length = 0;
        SampGetObjectAttribute(
            UserObject,
            wstr!("LMPwd") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );

        if Length as usize == core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() {
            Status = SampGetObjectAttribute(
                UserObject,
                wstr!("LMPwd") as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*InfoBuffer).Internal1.EncryptedLmOwfPassword) as PVOID,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            if memcmp(
                core::ptr::addr_of!((*InfoBuffer).Internal1.EncryptedLmOwfPassword) as *const core::ffi::c_void,
                core::ptr::addr_of!(EmptyLmHash) as *const core::ffi::c_void,
                core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
            ) != 0
            {
                (*InfoBuffer).Internal1.LmPasswordPresent = 1;
            }
        }

        (*InfoBuffer).Internal1.PasswordExpired = 0;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserInternal2(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T) as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*InfoBuffer).Internal2.Flags = 0;
        (*InfoBuffer).Internal2.LastLogon.LowPart = FixedData.LastLogon.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Internal2.LastLogon.HighPart = FixedData.LastLogon.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Internal2.LastLogoff.LowPart = FixedData.LastLogoff.__bindgen_anon_1.LowPart;
        (*InfoBuffer).Internal2.LastLogoff.HighPart = FixedData.LastLogoff.__bindgen_anon_1.HighPart;
        (*InfoBuffer).Internal2.BadPasswordCount = FixedData.BadPasswordCount;
        (*InfoBuffer).Internal2.LogonCount = FixedData.LogonCount;

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}
unsafe fn SampQueryUserParameters(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        w!("Parameters").as_ptr() as LPWSTR,
        core::ptr::addr_of_mut!((*InfoBuffer).Parameters.Parameters),
    );
    if !NT_SUCCESS(Status) {
        TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const core::ffi::c_char, Status);
    } else {
        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).Parameters.Parameters.Buffer.is_null() {
                midl_user_free((*InfoBuffer).Parameters.Parameters.Buffer as *mut core::ffi::c_void);
            }

            midl_user_free(InfoBuffer as *mut core::ffi::c_void);
        }
    }

    Status
}

unsafe fn SampQueryUserAll(
    UserObject: PSAM_DB_OBJECT,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let InfoBuffer: PSAMPR_USER_INFO_BUFFER;
    let mut DomainFixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let PasswordCanChange: LARGE_INTEGER;
    let PasswordMustChange: LARGE_INTEGER;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    *Buffer = core::ptr::null_mut();

    InfoBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_USER_INFO_BUFFER>() as SIZE_T)
        as PSAMPR_USER_INFO_BUFFER;
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        /* Get the fixed size domain data */
        Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            (*UserObject).ParentObject,
            wchar!("F"),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(DomainFixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Get the fixed size user data */
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wchar!("F"),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Set the fields to be returned */
        if (*UserObject).Trusted != 0 {
            (*InfoBuffer).All.WhichFields = USER_ALL_READ_GENERAL_MASK
                | USER_ALL_READ_LOGON_MASK
                | USER_ALL_READ_ACCOUNT_MASK
                | USER_ALL_READ_PREFERENCES_MASK
                | USER_ALL_READ_TRUSTED_MASK;
        } else {
            (*InfoBuffer).All.WhichFields = 0;

            if (*UserObject).Access & USER_READ_GENERAL != 0 {
                (*InfoBuffer).All.WhichFields |= USER_ALL_READ_GENERAL_MASK;
            }

            if (*UserObject).Access & USER_READ_LOGON != 0 {
                (*InfoBuffer).All.WhichFields |= USER_ALL_READ_LOGON_MASK;
            }

            if (*UserObject).Access & USER_READ_ACCOUNT != 0 {
                (*InfoBuffer).All.WhichFields |= USER_ALL_READ_ACCOUNT_MASK;
            }

            if (*UserObject).Access & USER_READ_PREFERENCES != 0 {
                (*InfoBuffer).All.WhichFields |= USER_ALL_READ_PREFERENCES_MASK;
            }
        }

        /* Fail, if no fields are to be returned */
        if (*InfoBuffer).All.WhichFields == 0 {
            Status = STATUS_ACCESS_DENIED;
            break 'done;
        }

        /* Get the UserName attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_USERNAME != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("Name"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.UserName),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the FullName attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_FULLNAME != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("FullName"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.FullName),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the UserId attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_USERID != 0 {
            (*InfoBuffer).All.UserId = FixedData.UserId;
        }

        /* Get the PrimaryGroupId attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_PRIMARYGROUPID != 0 {
            (*InfoBuffer).All.PrimaryGroupId = FixedData.PrimaryGroupId;
        }

        /* Get the AdminComment attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_ADMINCOMMENT != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("AdminComment"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.AdminComment),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the UserComment attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_USERCOMMENT != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("UserComment"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.UserComment),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the HomeDirectory attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_HOMEDIRECTORY != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("HomeDirectory"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.HomeDirectory),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the HomeDirectoryDrive attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_HOMEDIRECTORYDRIVE != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("HomeDirectoryDrive"),
                core::ptr::addr_of_mut!((*InfoBuffer).Home.HomeDirectoryDrive),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the ScriptPath attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_SCRIPTPATH != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("ScriptPath"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.ScriptPath),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the ProfilePath attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_PROFILEPATH != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("ProfilePath"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.ProfilePath),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the WorkStations attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_WORKSTATIONS != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("WorkStations"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.WorkStations),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the LastLogon attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_LASTLOGON != 0 {
            (*InfoBuffer).All.LastLogon.LowPart = FixedData.LastLogon.__bindgen_anon_1.LowPart;
            (*InfoBuffer).All.LastLogon.HighPart = FixedData.LastLogon.__bindgen_anon_1.HighPart;
        }

        /* Get the LastLogoff attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_LASTLOGOFF != 0 {
            (*InfoBuffer).All.LastLogoff.LowPart = FixedData.LastLogoff.__bindgen_anon_1.LowPart;
            (*InfoBuffer).All.LastLogoff.HighPart = FixedData.LastLogoff.__bindgen_anon_1.HighPart;
        }

        /* Get the LogonHours attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_LOGONHOURS != 0 {
            Status = SampGetLogonHoursAttribute(
                UserObject,
                core::ptr::addr_of_mut!((*InfoBuffer).All.LogonHours),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the BadPasswordCount attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_BADPASSWORDCOUNT != 0 {
            (*InfoBuffer).All.BadPasswordCount = FixedData.BadPasswordCount;
        }

        /* Get the LogonCount attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_LOGONCOUNT != 0 {
            (*InfoBuffer).All.LogonCount = FixedData.LogonCount;
        }

        /* Get the PasswordCanChange attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_PASSWORDCANCHANGE != 0 {
            PasswordCanChange = SampAddRelativeTimeToTime(
                FixedData.PasswordLastSet,
                DomainFixedData.MinPasswordAge,
            );
            (*InfoBuffer).All.PasswordCanChange.LowPart = PasswordCanChange.__bindgen_anon_1.LowPart;
            (*InfoBuffer).All.PasswordCanChange.HighPart = PasswordCanChange.__bindgen_anon_1.HighPart;
        }

        /* Get the PasswordMustChange attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_PASSWORDMUSTCHANGE != 0 {
            PasswordMustChange = SampAddRelativeTimeToTime(
                FixedData.PasswordLastSet,
                DomainFixedData.MaxPasswordAge,
            );
            (*InfoBuffer).All.PasswordMustChange.LowPart = PasswordMustChange.__bindgen_anon_1.LowPart;
            (*InfoBuffer).All.PasswordMustChange.HighPart = PasswordMustChange.__bindgen_anon_1.HighPart;
        }

        /* Get the PasswordLastSet attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_PASSWORDLASTSET != 0 {
            (*InfoBuffer).All.PasswordLastSet.LowPart = FixedData.PasswordLastSet.__bindgen_anon_1.LowPart;
            (*InfoBuffer).All.PasswordLastSet.HighPart = FixedData.PasswordLastSet.__bindgen_anon_1.HighPart;
        }

        /* Get the AccountExpires attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_ACCOUNTEXPIRES != 0 {
            (*InfoBuffer).All.AccountExpires.LowPart = FixedData.AccountExpires.__bindgen_anon_1.LowPart;
            (*InfoBuffer).All.AccountExpires.HighPart = FixedData.AccountExpires.__bindgen_anon_1.HighPart;
        }

        /* Get the UserAccountControl attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_USERACCOUNTCONTROL != 0 {
            (*InfoBuffer).All.UserAccountControl = FixedData.UserAccountControl;
        }

        /* Get the Parameters attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_PARAMETERS != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("Parameters"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.Parameters),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        /* Get the CountryCode attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_COUNTRYCODE != 0 {
            (*InfoBuffer).All.CountryCode = FixedData.CountryCode;
        }

        /* Get the CodePage attribute */
        if (*InfoBuffer).All.WhichFields & USER_ALL_CODEPAGE != 0 {
            (*InfoBuffer).All.CodePage = FixedData.CodePage;
        }

        /* Get the LmPassword and NtPassword attributes */
        if (*InfoBuffer).All.WhichFields & (USER_ALL_NTPASSWORDPRESENT | USER_ALL_LMPASSWORDPRESENT)
            != 0
        {
            (*InfoBuffer).All.LmPasswordPresent = 0u8;
            (*InfoBuffer).All.NtPasswordPresent = 0u8;

            /* Get the NT password */
            Length = 0;
            SampGetObjectAttribute(
                UserObject,
                wchar!("NTPwd"),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut Length,
            );

            if Length == core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG {
                (*InfoBuffer).All.NtOwfPassword.Buffer = midl_user_allocate(
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as SIZE_T,
                ) as *mut _;
                if (*InfoBuffer).All.NtOwfPassword.Buffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'done;
                }

                (*InfoBuffer).All.NtOwfPassword.Length =
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as u16;
                (*InfoBuffer).All.NtOwfPassword.MaximumLength =
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as u16;

                Status = SampGetObjectAttribute(
                    UserObject,
                    wchar!("NTPwd"),
                    core::ptr::null_mut(),
                    (*InfoBuffer).All.NtOwfPassword.Buffer as PVOID,
                    &mut Length,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }

                if memcmp(
                    (*InfoBuffer).All.NtOwfPassword.Buffer as *const _,
                    core::ptr::addr_of!(EmptyNtHash) as *const _,
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>(),
                ) != 0
                {
                    (*InfoBuffer).All.NtPasswordPresent = 1u8;
                }
            }

            /* Get the LM password */
            Length = 0;
            SampGetObjectAttribute(
                UserObject,
                wchar!("LMPwd"),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut Length,
            );

            if Length == core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG {
                (*InfoBuffer).All.LmOwfPassword.Buffer = midl_user_allocate(
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as SIZE_T,
                ) as *mut _;
                if (*InfoBuffer).All.LmOwfPassword.Buffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'done;
                }

                (*InfoBuffer).All.LmOwfPassword.Length =
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as u16;
                (*InfoBuffer).All.LmOwfPassword.MaximumLength =
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as u16;

                Status = SampGetObjectAttribute(
                    UserObject,
                    wchar!("LMPwd"),
                    core::ptr::null_mut(),
                    (*InfoBuffer).All.LmOwfPassword.Buffer as PVOID,
                    &mut Length,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }

                if memcmp(
                    (*InfoBuffer).All.LmOwfPassword.Buffer as *const _,
                    core::ptr::addr_of!(EmptyLmHash) as *const _,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                ) != 0
                {
                    (*InfoBuffer).All.LmPasswordPresent = 1u8;
                }
            }
        }

        if (*InfoBuffer).All.WhichFields & USER_ALL_PRIVATEDATA != 0 {
            Status = SampGetObjectAttributeString(
                UserObject,
                wchar!("PrivateData"),
                core::ptr::addr_of_mut!((*InfoBuffer).All.PrivateData),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(b"Status 0x%08lx\n\0".as_ptr() as *const i8, Status);
                break 'done;
            }
        }

        if (*InfoBuffer).All.WhichFields & USER_ALL_PASSWORDEXPIRED != 0 {
            /* FIXME */
        }

        if (*InfoBuffer).All.WhichFields & USER_ALL_SECURITYDESCRIPTOR != 0 {
            Length = 0;
            SampGetObjectAttribute(
                UserObject,
                wchar!("SecDesc"),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut Length,
            );

            if Length > 0 {
                (*InfoBuffer).All.SecurityDescriptor.SecurityDescriptor =
                    midl_user_allocate(Length as SIZE_T) as *mut _;
                if (*InfoBuffer)
                    .All
                    .SecurityDescriptor
                    .SecurityDescriptor
                    .is_null()
                {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'done;
                }

                (*InfoBuffer).All.SecurityDescriptor.Length = Length;

                Status = SampGetObjectAttribute(
                    UserObject,
                    wchar!("SecDesc"),
                    core::ptr::null_mut(),
                    (*InfoBuffer).All.SecurityDescriptor.SecurityDescriptor as PVOID,
                    &mut Length,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }
        }

        *Buffer = InfoBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !InfoBuffer.is_null() {
            if !(*InfoBuffer).All.UserName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.UserName.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.FullName.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.FullName.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.AdminComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.AdminComment.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.UserComment.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.UserComment.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.HomeDirectory.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.HomeDirectory.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.HomeDirectoryDrive.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.HomeDirectoryDrive.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.ScriptPath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.ScriptPath.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.ProfilePath.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.ProfilePath.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.WorkStations.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.WorkStations.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.LogonHours.LogonHours.is_null() {
                midl_user_free((*InfoBuffer).All.LogonHours.LogonHours as *mut _);
            }

            if !(*InfoBuffer).All.Parameters.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.Parameters.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.LmOwfPassword.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.LmOwfPassword.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.NtOwfPassword.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.NtOwfPassword.Buffer as *mut _);
            }

            if !(*InfoBuffer).All.PrivateData.Buffer.is_null() {
                midl_user_free((*InfoBuffer).All.PrivateData.Buffer as *mut _);
            }

            if !(*InfoBuffer)
                .All
                .SecurityDescriptor
                .SecurityDescriptor
                .is_null()
            {
                midl_user_free((*InfoBuffer).All.SecurityDescriptor.SecurityDescriptor as *mut _);
            }

            midl_user_free(InfoBuffer as *mut _);
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryInformationUser(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    SamrQueryInformationUser2(UserHandle, UserInformationClass, Buffer)
}

unsafe fn SampSetUserName(
    UserObject: PSAM_DB_OBJECT,
    NewUserName: PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let mut OldUserName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS;

    Status = SampCheckAccountName(NewUserName, 20);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = SampGetObjectAttributeString(
        UserObject,
        widestr_name().as_ptr() as *mut u16,
        core::ptr::addr_of_mut!(OldUserName) as PRPC_UNICODE_STRING,
    );
    if !NT_SUCCESS(Status) {
        break_done(&OldUserName);
        return Status;
    }

    'done: {
        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(OldUserName),
            NewUserName as PCUNICODE_STRING,
            1u8,
        ) == 0
        {
            Status = SampCheckAccountNameInDomain(
                (*UserObject).ParentObject,
                (*NewUserName).Buffer,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        Status = SampSetAccountNameInDomain(
            (*UserObject).ParentObject,
            widestr_users().as_ptr(),
            (*NewUserName).Buffer,
            (*UserObject).RelativeId,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampRemoveAccountNameFromDomain(
            (*UserObject).ParentObject,
            widestr_users().as_ptr(),
            OldUserName.Buffer,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            widestr_name().as_ptr() as *mut u16,
            NewUserName,
        );
    }

    if OldUserName.Buffer != core::ptr::null_mut() {
        midl_user_free(OldUserName.Buffer as *mut core::ffi::c_void);
    }

    Status
}

unsafe fn break_done(OldUserName: &_UNICODE_STRING) {
    if OldUserName.Buffer != core::ptr::null_mut() {
        midl_user_free(OldUserName.Buffer as *mut core::ffi::c_void);
    }
}

fn widestr_name() -> [u16; 5] {
    [b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0]
}

fn widestr_users() -> [u16; 6] {
    [
        b'U' as u16,
        b's' as u16,
        b'e' as u16,
        b'r' as u16,
        b's' as u16,
        0,
    ]
}

unsafe fn SampSetUserGeneral(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.PrimaryGroupId = (*Buffer).General.PrimaryGroupId;

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetUserName(
            UserObject,
            core::ptr::addr_of_mut!((*Buffer).General.UserName),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("FullName") as LPWSTR,
            core::ptr::addr_of_mut!((*Buffer).General.FullName),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("AdminComment") as LPWSTR,
            core::ptr::addr_of_mut!((*Buffer).General.AdminComment),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wstr!("UserComment") as LPWSTR,
            core::ptr::addr_of_mut!((*Buffer).General.UserComment),
        );
    }

    Status
}
unsafe fn SampSetUserPreferences(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        UserObject,
        w!("F").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.CountryCode = (*Buffer).Preferences.CountryCode;
        FixedData.CodePage = (*Buffer).Preferences.CodePage;

        Status = SampSetObjectAttribute(
            UserObject,
            w!("F").as_ptr() as LPWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            w!("UserComment").as_ptr() as LPWSTR,
            core::ptr::addr_of_mut!((*Buffer).Preferences.UserComment),
        );
    }

    Status
}

unsafe fn SampSetUserPrimaryGroup(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(
        UserObject,
        w!("F").as_ptr() as *const u16 as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(FixedData) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.PrimaryGroupId = (*Buffer).PrimaryGroup.PrimaryGroupId;

        Status = SampSetObjectAttribute(
            UserObject,
            w!("F").as_ptr() as *const u16 as LPWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
    }

    Status
}

unsafe fn SampSetUserControl(UserObject: PSAM_DB_OBJECT,
                            Buffer: PSAMPR_USER_INFO_BUFFER) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
    Status = SampGetObjectAttribute(UserObject,
                                    wstr!("F") as LPWSTR,
                                    core::ptr::null_mut(),
                                    &mut FixedData as *mut _ as PVOID,
                                    &mut Length);

    'done: loop {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.UserAccountControl = (*Buffer).Control.UserAccountControl;

        Status = SampSetObjectAttribute(UserObject,
                                        wstr!("F") as LPWSTR,
                                        REG_BINARY,
                                        &mut FixedData as *mut _ as PVOID,
                                        Length);

        break 'done;
    }

    Status
}

unsafe fn SampSetUserExpires(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedData.AccountExpires.__bindgen_anon_1.LowPart = (*Buffer).Expires.AccountExpires.LowPart;
        FixedData.AccountExpires.__bindgen_anon_1.HighPart = (*Buffer).Expires.AccountExpires.HighPart;

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
    }

    Status
}

unsafe fn SampSetUserInternal1(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    /* FIXME: Decrypt NT password */
    /* FIXME: Decrypt LM password */

    'done: loop {
        Status = SampSetUserPassword(
            UserObject,
            core::ptr::addr_of_mut!((*Buffer).Internal1.EncryptedNtOwfPassword),
            (*Buffer).Internal1.NtPasswordPresent,
            core::ptr::addr_of_mut!((*Buffer).Internal1.EncryptedLmOwfPassword),
            (*Buffer).Internal1.LmPasswordPresent,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            w!("F").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            core::ptr::addr_of_mut!(Length),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if (*Buffer).Internal1.PasswordExpired != 0 {
            FixedData.PasswordLastSet.u.LowPart = 0;
            FixedData.PasswordLastSet.u.HighPart = 0;
        } else {
            Status = NtQuerySystemTime(core::ptr::addr_of_mut!(FixedData.PasswordLastSet));
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        Status = SampSetObjectAttribute(
            UserObject,
            w!("F").as_ptr() as LPWSTR,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );

        break 'done;
    }

    Status
}

unsafe fn SampSetUserInternal2(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as *mut u16,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if ((*Buffer).Internal2.Flags & USER_LOGON_SUCCESS) != 0
            && ((*Buffer).Internal2.Flags & !USER_LOGON_SUCCESS) == 0
        {
            Status = NtQuerySystemTime(&mut FixedData.LastLogon);
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            FixedData.LogonCount += 1;
            FixedData.BadPasswordCount = 0;
        }

        if ((*Buffer).Internal2.Flags & USER_LOGON_BAD_PASSWORD) != 0
            && ((*Buffer).Internal2.Flags & !USER_LOGON_BAD_PASSWORD) == 0
        {
            Status = NtQuerySystemTime(&mut FixedData.LastBadPasswordTime);
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            FixedData.BadPasswordCount += 1;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wstr!("F") as *mut u16,
            REG_BINARY,
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            Length,
        );
    }

    Status
}

unsafe fn SampSetUserAll(
    UserObject: PSAM_DB_OBJECT,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut FixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let WhichFields: ULONG;
    let mut NtPassword: PENCRYPTED_NT_OWF_PASSWORD = core::ptr::null_mut();
    let mut LmPassword: PENCRYPTED_LM_OWF_PASSWORD = core::ptr::null_mut();
    let mut NtPasswordPresent: BOOLEAN = 0u8;
    let mut LmPasswordPresent: BOOLEAN = 0u8;
    let mut WriteFixedData: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    WhichFields = (*Buffer).All.WhichFields;

    'done: {
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FixedData) as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if WhichFields & USER_ALL_USERNAME != 0 {
            Status = SampSetUserName(UserObject, core::ptr::addr_of_mut!((*Buffer).All.UserName));
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_FULLNAME != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("FullName") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.FullName),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_ADMINCOMMENT != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("AdminComment") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.AdminComment),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_USERCOMMENT != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("UserComment") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.UserComment),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_HOMEDIRECTORY != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("HomeDirectory") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.HomeDirectory),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_HOMEDIRECTORYDRIVE != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("HomeDirectoryDrive") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.HomeDirectoryDrive),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_SCRIPTPATH != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("ScriptPath") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.ScriptPath),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_PROFILEPATH != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("ProfilePath") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.ProfilePath),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_WORKSTATIONS != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("WorkStations") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.WorkStations),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_PARAMETERS != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("Parameters") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.Parameters),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_LOGONHOURS != 0 {
            Status = SampSetLogonHoursAttribute(
                UserObject,
                core::ptr::addr_of_mut!((*Buffer).All.LogonHours),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_PRIMARYGROUPID != 0 {
            FixedData.PrimaryGroupId = (*Buffer).All.PrimaryGroupId;
            WriteFixedData = 1u8;
        }

        if WhichFields & USER_ALL_ACCOUNTEXPIRES != 0 {
            FixedData.AccountExpires.__bindgen_anon_1.LowPart = (*Buffer).All.AccountExpires.LowPart;
            FixedData.AccountExpires.__bindgen_anon_1.HighPart = (*Buffer).All.AccountExpires.HighPart;
            WriteFixedData = 1u8;
        }

        if WhichFields & USER_ALL_USERACCOUNTCONTROL != 0 {
            FixedData.UserAccountControl = (*Buffer).All.UserAccountControl;
            WriteFixedData = 1u8;
        }

        if WhichFields & USER_ALL_COUNTRYCODE != 0 {
            FixedData.CountryCode = (*Buffer).All.CountryCode;
            WriteFixedData = 1u8;
        }

        if WhichFields & USER_ALL_CODEPAGE != 0 {
            FixedData.CodePage = (*Buffer).All.CodePage;
            WriteFixedData = 1u8;
        }

        if WhichFields & (USER_ALL_NTPASSWORDPRESENT | USER_ALL_LMPASSWORDPRESENT) != 0 {
            if WhichFields & USER_ALL_NTPASSWORDPRESENT != 0 {
                NtPassword = (*Buffer).All.NtOwfPassword.Buffer as PENCRYPTED_NT_OWF_PASSWORD;
                NtPasswordPresent = (*Buffer).All.NtPasswordPresent;
            }

            if WhichFields & USER_ALL_LMPASSWORDPRESENT != 0 {
                LmPassword = (*Buffer).All.LmOwfPassword.Buffer as PENCRYPTED_LM_OWF_PASSWORD;
                LmPasswordPresent = (*Buffer).All.LmPasswordPresent;
            }

            Status = SampSetUserPassword(
                UserObject,
                NtPassword,
                NtPasswordPresent,
                LmPassword,
                LmPasswordPresent,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = NtQuerySystemTime(
                core::ptr::addr_of_mut!(FixedData.PasswordLastSet) as *mut LARGE_INTEGER,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            WriteFixedData = 1u8;
        }

        if WhichFields & USER_ALL_PRIVATEDATA != 0 {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("PrivateData") as LPWSTR,
                core::ptr::addr_of_mut!((*Buffer).All.PrivateData),
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        if WhichFields & USER_ALL_PASSWORDEXPIRED != 0 {
            if (*Buffer).All.PasswordExpired != 0 {
                FixedData.PasswordLastSet.__bindgen_anon_1.LowPart = 0;
                FixedData.PasswordLastSet.__bindgen_anon_1.HighPart = 0;
            } else {
                Status = NtQuerySystemTime(
                    core::ptr::addr_of_mut!(FixedData.PasswordLastSet) as *mut LARGE_INTEGER,
                );
                if !NT_SUCCESS(Status) {
                    break 'done;
                }
            }

            WriteFixedData = 1u8;
        }

        if WhichFields & USER_ALL_SECURITYDESCRIPTOR != 0 {
            Status = SampSetObjectAttribute(
                UserObject,
                wstr!("SecDesc") as LPWSTR,
                REG_BINARY,
                (*Buffer).All.SecurityDescriptor.SecurityDescriptor as PVOID,
                (*Buffer).All.SecurityDescriptor.Length,
            );
        }

        if WriteFixedData != 0u8 {
            Status = SampSetObjectAttribute(
                UserObject,
                wstr!("F") as LPWSTR,
                REG_BINARY,
                core::ptr::addr_of_mut!(FixedData) as PVOID,
                Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetInformationUser(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    TRACE!(
        b"SamrSetInformationUser(%p %lu %p)\n\0".as_ptr() as *const i8,
        UserHandle,
        UserInformationClass,
        Buffer,
    );

    SamrSetInformationUser2(UserHandle, UserInformationClass, Buffer)
}

#[no_mangle]
pub unsafe extern "system" fn SamrChangePasswordUser(
    UserHandle: SAMPR_HANDLE,
    LmPresent: ::core::ffi::c_uchar,
    OldLmEncryptedWithNewLm: PENCRYPTED_LM_OWF_PASSWORD,
    NewLmEncryptedWithOldLm: PENCRYPTED_LM_OWF_PASSWORD,
    NtPresent: ::core::ffi::c_uchar,
    OldNtEncryptedWithNewNt: PENCRYPTED_NT_OWF_PASSWORD,
    NewNtEncryptedWithOldNt: PENCRYPTED_NT_OWF_PASSWORD,
    NtCrossEncryptionPresent: ::core::ffi::c_uchar,
    NewNtEncryptedWithNewLm: PENCRYPTED_NT_OWF_PASSWORD,
    LmCrossEncryptionPresent: ::core::ffi::c_uchar,
    NewLmEncryptedWithNewNt: PENCRYPTED_LM_OWF_PASSWORD,
) -> NTSTATUS {
    let mut StoredLmPassword: ENCRYPTED_LM_OWF_PASSWORD = core::mem::zeroed();
    let mut StoredNtPassword: ENCRYPTED_NT_OWF_PASSWORD = core::mem::zeroed();
    let mut OldLmPassword: LM_OWF_PASSWORD = core::mem::zeroed();
    let mut NewLmPassword: LM_OWF_PASSWORD = core::mem::zeroed();
    let mut OldNtPassword: NT_OWF_PASSWORD = core::mem::zeroed();
    let mut NewNtPassword: NT_OWF_PASSWORD = core::mem::zeroed();
    let mut StoredLmPresent: BOOLEAN = 0u8;
    let mut StoredNtPresent: BOOLEAN = 0u8;
    let mut StoredLmEmpty: BOOLEAN = 1u8;
    let mut StoredNtEmpty: BOOLEAN = 1u8;
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG;
    let mut UserFixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut DomainFixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        /* Validate the user handle */
        Status = SampValidateDbObject(
            UserHandle,
            SamDbUserObject,
            USER_CHANGE_PASSWORD,
            &mut UserObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Get the current time */
        Status = NtQuerySystemTime(&mut SystemTime);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Retrieve the LM password */
        Length = core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("LMPwd") as LPWSTR,
            core::ptr::null_mut(),
            &mut StoredLmPassword as *mut _ as PVOID,
            &mut Length,
        );
        if NT_SUCCESS(Status) {
            if Length == core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>() as ULONG {
                StoredLmPresent = 1u8;
                if RtlEqualMemory(
                    &StoredLmPassword as *const _ as PVOID,
                    core::ptr::addr_of!(EmptyLmHash) as PVOID,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                ) == 0
                {
                    StoredLmEmpty = 0u8;
                }
            }
        }

        /* Retrieve the NT password */
        Length = core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("NTPwd") as LPWSTR,
            core::ptr::null_mut(),
            &mut StoredNtPassword as *mut _ as PVOID,
            &mut Length,
        );
        if NT_SUCCESS(Status) {
            if Length == core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>() as ULONG {
                StoredNtPresent = 1u8;
                if RtlEqualMemory(
                    &StoredNtPassword as *const _ as PVOID,
                    core::ptr::addr_of!(EmptyNtHash) as PVOID,
                    core::mem::size_of::<ENCRYPTED_NT_OWF_PASSWORD>(),
                ) == 0
                {
                    StoredNtEmpty = 0u8;
                }
            }
        }

        /* Retrieve the fixed size user data */
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            wstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            &mut UserFixedData as *mut _ as PVOID,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Check if we can change the password at this time */
        if StoredLmEmpty == 0u8 || StoredNtEmpty == 0u8 {
            /* Get fixed domain data */
            Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
            Status = SampGetObjectAttribute(
                (*UserObject).ParentObject,
                wstr!("F") as LPWSTR,
                core::ptr::null_mut(),
                &mut DomainFixedData as *mut _ as PVOID,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            if DomainFixedData.MinPasswordAge.QuadPart > 0 {
                if SystemTime.QuadPart
                    < (UserFixedData.PasswordLastSet.QuadPart
                        + DomainFixedData.MinPasswordAge.QuadPart)
                {
                    Status = STATUS_ACCOUNT_RESTRICTION;
                    break 'done;
                }
            }
        }

        /* Decrypt the LM passwords, if present */
        if LmPresent != 0 {
            Status = SystemFunction013(
                NewLmEncryptedWithOldLm as *const BYTE,
                &StoredLmPassword as *const _ as *const BYTE,
                &mut NewLmPassword as *mut _ as LPBYTE,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SystemFunction013(
                OldLmEncryptedWithNewLm as *const BYTE,
                &NewLmPassword as *const _ as *const BYTE,
                &mut OldLmPassword as *mut _ as LPBYTE,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        /* Decrypt the NT passwords, if present */
        if NtPresent != 0 {
            Status = SystemFunction013(
                NewNtEncryptedWithOldNt as *const BYTE,
                &StoredNtPassword as *const _ as *const BYTE,
                &mut NewNtPassword as *mut _ as LPBYTE,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }

            Status = SystemFunction013(
                OldNtEncryptedWithNewNt as *const BYTE,
                &NewNtPassword as *const _ as *const BYTE,
                &mut OldNtPassword as *mut _ as LPBYTE,
            );
            if !NT_SUCCESS(Status) {
                break 'done;
            }
        }

        /* Check if the old passwords match the stored ones */
        if NtPresent != 0 {
            if LmPresent != 0 {
                if RtlEqualMemory(
                    &StoredLmPassword as *const _ as PVOID,
                    &OldLmPassword as *const _ as PVOID,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                ) == 0
                {
                    Status = STATUS_WRONG_PASSWORD;
                } else {
                    if RtlEqualMemory(
                        &StoredNtPassword as *const _ as PVOID,
                        &OldNtPassword as *const _ as PVOID,
                        core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                    ) == 0
                    {
                        Status = STATUS_WRONG_PASSWORD;
                    }
                }
            } else {
                if RtlEqualMemory(
                    &StoredNtPassword as *const _ as PVOID,
                    &OldNtPassword as *const _ as PVOID,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                ) == 0
                {
                    Status = STATUS_WRONG_PASSWORD;
                }
            }
        } else {
            if LmPresent != 0 {
                if RtlEqualMemory(
                    &StoredLmPassword as *const _ as PVOID,
                    &OldLmPassword as *const _ as PVOID,
                    core::mem::size_of::<ENCRYPTED_LM_OWF_PASSWORD>(),
                ) == 0
                {
                    Status = STATUS_WRONG_PASSWORD;
                }
            } else {
                Status = STATUS_INVALID_PARAMETER;
            }
        }

        /* Store the new password hashes */
        if NT_SUCCESS(Status) {
            Status = SampSetUserPassword(
                UserObject,
                &mut NewNtPassword,
                NtPresent,
                &mut NewLmPassword,
                LmPresent,
            );
            if NT_SUCCESS(Status) {
                /* Update PasswordLastSet */
                UserFixedData.PasswordLastSet.QuadPart = SystemTime.QuadPart;

                /* Set the fixed size user data */
                Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
                Status = SampSetObjectAttribute(
                    UserObject,
                    wstr!("F") as LPWSTR,
                    REG_BINARY,
                    &mut UserFixedData as *mut _ as PVOID,
                    Length,
                );
            }
        }

        if Status == STATUS_WRONG_PASSWORD {
            /* Update BadPasswordCount and LastBadPasswordTime */
            UserFixedData.BadPasswordCount += 1;
            UserFixedData.LastBadPasswordTime.QuadPart = SystemTime.QuadPart;

            /* Set the fixed size user data */
            Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
            Status = SampSetObjectAttribute(
                UserObject,
                wstr!("F") as LPWSTR,
                REG_BINARY,
                &mut UserFixedData as *mut _ as PVOID,
                Length,
            );
        }
    }

    let _ = StoredLmPresent;
    let _ = StoredNtPresent;
    let _ = NtCrossEncryptionPresent;
    let _ = NewNtEncryptedWithNewLm;
    let _ = LmCrossEncryptionPresent;
    let _ = NewLmEncryptedWithNewNt;

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetGroupsForUser(
    UserHandle: SAMPR_HANDLE,
    Groups: *mut PSAMPR_GET_GROUPS_BUFFER,
) -> NTSTATUS {
    let mut GroupsBuffer: PSAMPR_GET_GROUPS_BUFFER = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    TRACE!(b"SamrGetGroupsForUser(%p %p)\n\0".as_ptr() as *const i8, UserHandle, Groups);

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: {
        Status = SampValidateDbObject(
            UserHandle,
            SamDbUserObject,
            USER_LIST_GROUPS,
            core::ptr::addr_of_mut!(UserObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampValidateDbObject failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        GroupsBuffer = midl_user_allocate(core::mem::size_of::<SAMPR_GET_GROUPS_BUFFER>() as SIZE_T) as PSAMPR_GET_GROUPS_BUFFER;
        if GroupsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        SampGetObjectAttribute(
            UserObject,
            w!("Groups").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(Length),
        );

        if Length == 0 {
            (*GroupsBuffer).MembershipCount = 0;
            (*GroupsBuffer).Groups = core::ptr::null_mut();

            *Groups = GroupsBuffer;

            Status = STATUS_SUCCESS;
            break 'done;
        }

        (*GroupsBuffer).Groups = midl_user_allocate(Length as SIZE_T) as *mut GROUP_MEMBERSHIP;
        if (*GroupsBuffer).Groups.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        Status = SampGetObjectAttribute(
            UserObject,
            w!("Groups").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            (*GroupsBuffer).Groups as PVOID,
            core::ptr::addr_of_mut!(Length),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(b"SampGetObjectAttribute failed with status 0x%08lx\n\0".as_ptr() as *const i8, Status);
            break 'done;
        }

        (*GroupsBuffer).MembershipCount = Length / core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG;

        *Groups = GroupsBuffer;
    }

    if !NT_SUCCESS(Status) {
        if !GroupsBuffer.is_null() {
            if !(*GroupsBuffer).Groups.is_null() {
                midl_user_free((*GroupsBuffer).Groups as *mut core::ffi::c_void);
            }

            midl_user_free(GroupsBuffer as *mut core::ffi::c_void);
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryDisplayInformation(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Index: ::core::ffi::c_ulong,
    EntryCount: ::core::ffi::c_ulong,
    PreferredMaximumLength: ::core::ffi::c_ulong,
    TotalAvailable: *mut ::core::ffi::c_ulong,
    TotalReturned: *mut ::core::ffi::c_ulong,
    Buffer: PSAMPR_DISPLAY_INFO_BUFFER,
) -> NTSTATUS {
    SamrQueryDisplayInformation3(
        DomainHandle,
        DisplayInformationClass,
        Index,
        EntryCount,
        PreferredMaximumLength,
        TotalAvailable,
        TotalReturned,
        Buffer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetDisplayEnumerationIndex(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Prefix: PRPC_UNICODE_STRING,
    Index: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    SamrGetDisplayEnumerationIndex2(
        DomainHandle,
        DisplayInformationClass,
        Prefix,
        Index,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SamrTestPrivateFunctionsDomain(_DomainHandle: SAMPR_HANDLE) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrTestPrivateFunctionsUser(_UserHandle: SAMPR_HANDLE) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetUserDomainPasswordInformation(
    UserHandle: SAMPR_HANDLE,
    PasswordInformation: PUSER_DOMAIN_PASSWORD_INFORMATION,
) -> NTSTATUS {
    let mut DomainFixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut UserFixedData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrGetUserDomainPasswordInformation(%p %p)\n\0".as_ptr() as *const i8,
        UserHandle,
        PasswordInformation,
    );

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: loop {
        /* Validate the user handle */
        Status = SampValidateDbObject(
            UserHandle,
            SamDbUserObject,
            0,
            core::ptr::addr_of_mut!(UserObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SampValidateDbObject failed with status 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        /* Validate the domain object */
        Status = SampValidateDbObject(
            (*UserObject).ParentObject as SAMPR_HANDLE,
            SamDbDomainObject,
            DOMAIN_READ_PASSWORD_PARAMETERS,
            core::ptr::addr_of_mut!(DomainObject),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SampValidateDbObject failed with status 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        /* Get fixed user data */
        Length = core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            UserObject,
            bwstr!("F") as LPWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(UserFixedData) as LPVOID,
            core::ptr::addr_of_mut!(Length),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SampGetObjectAttribute failed with status 0x%08lx\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        if ((*UserObject).RelativeId == DOMAIN_USER_RID_KRBTGT)
            || (UserFixedData.UserAccountControl
                & (USER_INTERDOMAIN_TRUST_ACCOUNT
                    | USER_WORKSTATION_TRUST_ACCOUNT
                    | USER_SERVER_TRUST_ACCOUNT))
                != 0
        {
            (*PasswordInformation).MinPasswordLength = 0;
            (*PasswordInformation).PasswordProperties = 0;
        } else {
            /* Get fixed domain data */
            Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
            Status = SampGetObjectAttribute(
                DomainObject,
                bwstr!("F") as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(DomainFixedData) as LPVOID,
                core::ptr::addr_of_mut!(Length),
            );
            if !NT_SUCCESS(Status) {
                TRACE!(
                    b"SampGetObjectAttribute failed with status 0x%08lx\n\0".as_ptr() as *const i8,
                    Status,
                );
                break 'done;
            }

            (*PasswordInformation).MinPasswordLength = DomainFixedData.MinPasswordLength;
            (*PasswordInformation).PasswordProperties = DomainFixedData.PasswordProperties;
        }

        break 'done;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SamrRemoveMemberFromForeignDomain(
    DomainHandle: SAMPR_HANDLE,
    MemberSid: PRPC_SID,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Rid: ULONG = 0;
    let mut Status: NTSTATUS;

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        /* Validate the domain object */
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LOOKUP,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Retrieve the RID from the MemberSID */
        Status = SampGetRidFromSid(MemberSid as PSID, &mut Rid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        /* Fail, if the RID represents a special account */
        if Rid < 1000 {
            Status = STATUS_SPECIAL_ACCOUNT;
            break 'done;
        }

        /* Remove the member from all aliases in the domain */
        Status = SampRemoveMemberFromAllAliases(DomainObject, MemberSid);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryInformationDomain2(
    DomainHandle: SAMPR_HANDLE,
    DomainInformationClass: DOMAIN_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_DOMAIN_INFO_BUFFER,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let DesiredAccess: ACCESS_MASK;
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrQueryInformationDomain2(%p %lu %p)\n\0".as_ptr(),
        DomainHandle,
        DomainInformationClass,
        Buffer,
    );

    match DomainInformationClass {
        DOMAIN_INFORMATION_CLASS_DomainPasswordInformation
        | DOMAIN_INFORMATION_CLASS_DomainLockoutInformation => {
            DesiredAccess = DOMAIN_READ_PASSWORD_PARAMETERS;
        }
        DOMAIN_INFORMATION_CLASS_DomainGeneralInformation
        | DOMAIN_INFORMATION_CLASS_DomainLogoffInformation
        | DOMAIN_INFORMATION_CLASS_DomainOemInformation
        | DOMAIN_INFORMATION_CLASS_DomainNameInformation
        | DOMAIN_INFORMATION_CLASS_DomainReplicationInformation
        | DOMAIN_INFORMATION_CLASS_DomainServerRoleInformation
        | DOMAIN_INFORMATION_CLASS_DomainModifiedInformation
        | DOMAIN_INFORMATION_CLASS_DomainStateInformation
        | DOMAIN_INFORMATION_CLASS_DomainModifiedInformation2 => {
            DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS;
        }
        DOMAIN_INFORMATION_CLASS_DomainGeneralInformation2 => {
            DesiredAccess = DOMAIN_READ_PASSWORD_PARAMETERS | DOMAIN_READ_OTHER_PARAMETERS;
        }
        _ => {
            return STATUS_INVALID_INFO_CLASS;
        }
    }

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    Status = SampValidateDbObject(
        DomainHandle,
        SamDbDomainObject,
        DesiredAccess,
        core::ptr::addr_of_mut!(DomainObject),
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        match DomainInformationClass {
            DOMAIN_INFORMATION_CLASS_DomainPasswordInformation => {
                Status = SampQueryDomainPassword(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainGeneralInformation => {
                Status = SampQueryDomainGeneral(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainLogoffInformation => {
                Status = SampQueryDomainLogoff(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainOemInformation => {
                Status = SampQueryDomainOem(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainNameInformation => {
                Status = SampQueryDomainName(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainReplicationInformation => {
                Status = SampQueryDomainReplication(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainServerRoleInformation => {
                Status = SampQueryDomainServerRole(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainModifiedInformation => {
                Status = SampQueryDomainModified(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainStateInformation => {
                Status = SampQueryDomainState(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainGeneralInformation2 => {
                Status = SampQueryDomainGeneral2(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainLockoutInformation => {
                Status = SampQueryDomainLockout(DomainObject, Buffer);
            }
            DOMAIN_INFORMATION_CLASS_DomainModifiedInformation2 => {
                Status = SampQueryDomainModified2(DomainObject, Buffer);
            }
            _ => {
                Status = STATUS_NOT_IMPLEMENTED;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryInformationUser2(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: *mut PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let DesiredAccess: ACCESS_MASK;
    let mut Status: NTSTATUS;

    TRACE!(
        b"SamrQueryInformationUser2(%p %lu %p)\n\0".as_ptr() as *const i8,
        UserHandle,
        UserInformationClass,
        Buffer,
    );

    match UserInformationClass {
        UserGeneralInformation
        | UserNameInformation
        | UserAccountNameInformation
        | UserFullNameInformation
        | UserPrimaryGroupInformation
        | UserAdminCommentInformation => {
            DesiredAccess = USER_READ_GENERAL;
        }

        UserLogonHoursInformation
        | UserHomeInformation
        | UserScriptInformation
        | UserProfileInformation
        | UserWorkStationsInformation => {
            DesiredAccess = USER_READ_LOGON;
        }

        UserControlInformation | UserExpiresInformation | UserParametersInformation => {
            DesiredAccess = USER_READ_ACCOUNT;
        }

        UserPreferencesInformation => {
            DesiredAccess = USER_READ_GENERAL | USER_READ_PREFERENCES;
        }

        UserLogonInformation | UserAccountInformation => {
            DesiredAccess = USER_READ_GENERAL
                | USER_READ_PREFERENCES
                | USER_READ_LOGON
                | USER_READ_ACCOUNT;
        }

        UserInternal1Information | UserInternal2Information | UserAllInformation => {
            DesiredAccess = 0;
        }

        _ => return STATUS_INVALID_INFO_CLASS,
    }

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    Status = SampValidateDbObject(
        UserHandle,
        SamDbUserObject,
        DesiredAccess,
        &mut UserObject,
    );
    if !NT_SUCCESS(Status) {
        TRACE!(
            b"failed with status 0x%08lx\n\0".as_ptr() as *const i8,
            Status,
        );
    } else {
        match UserInformationClass {
            UserGeneralInformation => {
                Status = SampQueryUserGeneral(UserObject, Buffer);
            }
            UserPreferencesInformation => {
                Status = SampQueryUserPreferences(UserObject, Buffer);
            }
            UserLogonInformation => {
                Status = SampQueryUserLogon(UserObject, Buffer);
            }
            UserLogonHoursInformation => {
                Status = SampQueryUserLogonHours(UserObject, Buffer);
            }
            UserAccountInformation => {
                Status = SampQueryUserAccount(UserObject, Buffer);
            }
            UserNameInformation => {
                Status = SampQueryUserName(UserObject, Buffer);
            }
            UserAccountNameInformation => {
                Status = SampQueryUserAccountName(UserObject, Buffer);
            }
            UserFullNameInformation => {
                Status = SampQueryUserFullName(UserObject, Buffer);
            }
            UserPrimaryGroupInformation => {
                Status = SampQueryUserPrimaryGroup(UserObject, Buffer);
            }
            UserHomeInformation => {
                Status = SampQueryUserHome(UserObject, Buffer);
            }
            UserScriptInformation => {
                Status = SampQueryUserScript(UserObject, Buffer);
            }
            UserProfileInformation => {
                Status = SampQueryUserProfile(UserObject, Buffer);
            }
            UserAdminCommentInformation => {
                Status = SampQueryUserAdminComment(UserObject, Buffer);
            }
            UserWorkStationsInformation => {
                Status = SampQueryUserWorkStations(UserObject, Buffer);
            }
            UserControlInformation => {
                Status = SampQueryUserControl(UserObject, Buffer);
            }
            UserExpiresInformation => {
                Status = SampQueryUserExpires(UserObject, Buffer);
            }
            UserInternal1Information => {
                Status = SampQueryUserInternal1(UserObject, Buffer);
            }
            UserInternal2Information => {
                Status = SampQueryUserInternal2(UserObject, Buffer);
            }
            UserParametersInformation => {
                Status = SampQueryUserParameters(UserObject, Buffer);
            }
            UserAllInformation => {
                Status = SampQueryUserAll(UserObject, Buffer);
            }
            _ => {
                Status = STATUS_INVALID_INFO_CLASS;
            }
        }
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryDisplayInformation2(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    Index: ::core::ffi::c_ulong,
    EntryCount: ::core::ffi::c_ulong,
    PreferredMaximumLength: ::core::ffi::c_ulong,
    TotalAvailable: *mut ::core::ffi::c_ulong,
    TotalReturned: *mut ::core::ffi::c_ulong,
    Buffer: PSAMPR_DISPLAY_INFO_BUFFER,
) -> NTSTATUS {
    SamrQueryDisplayInformation3(
        DomainHandle,
        DisplayInformationClass,
        Index,
        EntryCount,
        PreferredMaximumLength,
        TotalAvailable,
        TotalReturned,
        Buffer,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetDisplayEnumerationIndex2(
    _DomainHandle: SAMPR_HANDLE,
    _DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    _Prefix: PRPC_UNICODE_STRING,
    _Index: *mut core::ffi::c_ulong,
) -> NTSTATUS {
    TRACE!(
        b"SamrGetDisplayEnumerationIndex2(%p %lu %p %p)\n\0".as_ptr() as *const i8,
        DomainHandle,
        DisplayInformationClass,
        Prefix,
        Index,
    );

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrCreateUser2InDomain(
    DomainHandle: SAMPR_HANDLE,
    Name: PRPC_UNICODE_STRING,
    AccountType: ::core::ffi::c_ulong,
    mut DesiredAccess: ACCESS_MASK,
    UserHandle: *mut SAMPR_HANDLE,
    GrantedAccess: *mut ::core::ffi::c_ulong,
    RelativeId: *mut ::core::ffi::c_ulong,
) -> NTSTATUS {
    let mut FixedDomainData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut FixedUserData: SAM_USER_FIXED_DATA = core::mem::zeroed();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut GroupMembership: GROUP_MEMBERSHIP = core::mem::zeroed();
    let mut LogonHours: [UCHAR; 23] = [0; 23];
    let mut ulSize: ULONG;
    let ulRid: ULONG;
    let mut szRid: [WCHAR; 9] = [0; 9];
    let mut Sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SdSize: ULONG = 0;
    let mut UserSid: PSID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if Name.is_null()
        || (*Name).Length == 0
        || (*Name).Buffer.is_null()
        || UserHandle.is_null()
        || RelativeId.is_null()
    {
        return STATUS_INVALID_PARAMETER;
    }

    if AccountType != USER_NORMAL_ACCOUNT
        && AccountType != USER_WORKSTATION_TRUST_ACCOUNT
        && AccountType != USER_INTERDOMAIN_TRUST_ACCOUNT
        && AccountType != USER_SERVER_TRUST_ACCOUNT
        && AccountType != USER_TEMP_DUPLICATE_ACCOUNT
    {
        return STATUS_INVALID_PARAMETER;
    }

    RtlMapGenericMask(&mut DesiredAccess, &raw mut UserMapping);

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    'done: {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_CREATE_USER,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountName(Name, 20);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCheckAccountNameInDomain(DomainObject, (*Name).Buffer);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulSize = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            wsz!("F"),
            core::ptr::null_mut(),
            &mut FixedDomainData as *mut _ as PVOID,
            &mut ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        ulRid = FixedDomainData.NextRid;
        FixedDomainData.NextRid += 1;

        Status = SampCreateAccountSid(DomainObject, ulRid, &mut UserSid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampCreateUserSD(UserSid, &mut Sd, &mut SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            DomainObject,
            wsz!("F"),
            REG_BINARY,
            &mut FixedDomainData as *mut _ as PVOID,
            ulSize,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        swprintf(szRid.as_mut_ptr(), wsz!("%08lX"), ulRid);

        Status = SampCreateDbObject(
            DomainObject,
            wsz!("Users"),
            szRid.as_mut_ptr(),
            ulRid,
            SamDbUserObject,
            DesiredAccess,
            &mut UserObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetAccountNameInDomain(DomainObject, wsz!("Users"), (*Name).Buffer, ulRid);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        FixedUserData.Version = 1;
        FixedUserData.Reserved = 0;
        FixedUserData.LastLogon.QuadPart = 0;
        FixedUserData.LastLogoff.QuadPart = 0;
        FixedUserData.PasswordLastSet.QuadPart = 0;
        FixedUserData.AccountExpires.__bindgen_anon_1.LowPart = MAXULONG;
        FixedUserData.AccountExpires.__bindgen_anon_1.HighPart = MAXLONG as LONG;
        FixedUserData.LastBadPasswordTime.QuadPart = 0;
        FixedUserData.UserId = ulRid;
        FixedUserData.PrimaryGroupId = DOMAIN_GROUP_RID_USERS;
        FixedUserData.UserAccountControl =
            USER_ACCOUNT_DISABLED | USER_PASSWORD_NOT_REQUIRED | AccountType;
        FixedUserData.CountryCode = 0;
        FixedUserData.CodePage = 0;
        FixedUserData.BadPasswordCount = 0;
        FixedUserData.LogonCount = 0;
        FixedUserData.AdminCount = 0;
        FixedUserData.OperatorCount = 0;

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("F"),
            REG_BINARY,
            &mut FixedUserData as *mut _ as LPVOID,
            core::mem::size_of::<SAM_USER_FIXED_DATA>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(UserObject, wsz!("Name"), Name);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(UserObject, wsz!("FullName"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status =
            SampSetObjectAttributeString(UserObject, wsz!("HomeDirectory"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(
            UserObject,
            wsz!("HomeDirectoryDrive"),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(UserObject, wsz!("ScriptPath"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status =
            SampSetObjectAttributeString(UserObject, wsz!("ProfilePath"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status =
            SampSetObjectAttributeString(UserObject, wsz!("AdminComment"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status =
            SampSetObjectAttributeString(UserObject, wsz!("UserComment"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status =
            SampSetObjectAttributeString(UserObject, wsz!("WorkStations"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttributeString(UserObject, wsz!("Parameters"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (LogonHours.as_mut_ptr() as PUSHORT).write_unaligned(168);
        memset(
            &mut LogonHours[2] as *mut _ as *mut ::core::ffi::c_void,
            0xff,
            21,
        );

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("LogonHours"),
            REG_BINARY,
            &mut LogonHours as *mut _ as PVOID,
            core::mem::size_of::<[UCHAR; 23]>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        GroupMembership.RelativeId = DOMAIN_GROUP_RID_USERS;
        GroupMembership.Attributes =
            SE_GROUP_MANDATORY | SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT;

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("Groups"),
            REG_BINARY,
            &mut GroupMembership as *mut _ as PVOID,
            core::mem::size_of::<GROUP_MEMBERSHIP>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("LMPwd"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("NTPwd"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("LMPwdHistory"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(
            UserObject,
            wsz!("NTPwdHistory"),
            REG_BINARY,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status =
            SampSetObjectAttributeString(UserObject, wsz!("PrivateData"), core::ptr::null_mut());
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampSetObjectAttribute(UserObject, wsz!("SecDesc"), REG_BINARY, Sd, SdSize);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        if NT_SUCCESS(Status) {
            *UserHandle = UserObject as SAMPR_HANDLE;
            *RelativeId = ulRid;
            *GrantedAccess = (*UserObject).Access;
        }
    }

    if !Sd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
    }

    if !UserSid.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrQueryDisplayInformation3(
    DomainHandle: SAMPR_HANDLE,
    DisplayInformationClass: DOMAIN_DISPLAY_INFORMATION,
    _Index: ::core::ffi::c_ulong,
    _EntryCount: ::core::ffi::c_ulong,
    _PreferredMaximumLength: ::core::ffi::c_ulong,
    _TotalAvailable: *mut ::core::ffi::c_ulong,
    _TotalReturned: *mut ::core::ffi::c_ulong,
    _Buffer: PSAMPR_DISPLAY_INFO_BUFFER,
) -> NTSTATUS {
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    'done: loop {
        Status = SampValidateDbObject(
            DomainHandle,
            SamDbDomainObject,
            DOMAIN_LIST_ACCOUNTS,
            core::ptr::addr_of_mut!(DomainObject),
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampFillDisplayCache(DomainObject, DisplayInformationClass);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        break 'done;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrAddMultipleMembersToAlias(
    AliasHandle: SAMPR_HANDLE,
    MembersBuffer: PSAMPR_PSID_ARRAY,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    TRACE!(
        b"SamrAddMultipleMembersToAlias(%p %p)\n\0".as_ptr() as *const i8,
        AliasHandle,
        MembersBuffer,
    );

    let mut i: ULONG = 0;
    while i < (*MembersBuffer).Count {
        Status = SamrAddMemberToAlias(
            AliasHandle,
            (*(*MembersBuffer).Sids.offset(i as isize)).SidPointer as PRPC_SID,
        );

        if Status == STATUS_MEMBER_IN_ALIAS {
            Status = STATUS_SUCCESS;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        i += 1;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrRemoveMultipleMembersFromAlias(
    AliasHandle: SAMPR_HANDLE,
    MembersBuffer: PSAMPR_PSID_ARRAY,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    TRACE!(
        b"SamrRemoveMultipleMembersFromAlias(%p %p)\n\0".as_ptr() as *const i8,
        AliasHandle,
        MembersBuffer,
    );

    let mut i: ULONG = 0;
    while i < (*MembersBuffer).Count {
        Status = SamrRemoveMemberFromAlias(
            AliasHandle,
            (*(*MembersBuffer).Sids.offset(i as isize)).SidPointer as PRPC_SID,
        );

        if Status == STATUS_MEMBER_IN_ALIAS {
            Status = STATUS_SUCCESS;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        i += 1;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrOemChangePasswordUser2(
    _BindingHandle: handle_t,
    _ServerName: PRPC_STRING,
    _UserName: PRPC_STRING,
    _NewPasswordEncryptedWithOldLm: PSAMPR_ENCRYPTED_USER_PASSWORD,
    _OldLmOwfPasswordEncryptedWithNewLm: PENCRYPTED_LM_OWF_PASSWORD,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrUnicodeChangePasswordUser2(
    _BindingHandle: handle_t,
    _ServerName: PRPC_UNICODE_STRING,
    _UserName: PRPC_UNICODE_STRING,
    _NewPasswordEncryptedWithOldNt: PSAMPR_ENCRYPTED_USER_PASSWORD,
    _OldNtOwfPasswordEncryptedWithNewNt: PENCRYPTED_NT_OWF_PASSWORD,
    _LmPresent: ::core::ffi::c_uchar,
    _NewPasswordEncryptedWithOldLm: PSAMPR_ENCRYPTED_USER_PASSWORD,
    _OldLmOwfPasswordEncryptedWithNewNt: PENCRYPTED_LM_OWF_PASSWORD,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrGetDomainPasswordInformation(
    _BindingHandle: handle_t,
    _Unused: PRPC_UNICODE_STRING,
    PasswordInformation: PUSER_DOMAIN_PASSWORD_INFORMATION,
) -> NTSTATUS {
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut DomainObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let mut FixedData: SAM_DOMAIN_FIXED_DATA = core::mem::zeroed();
    let mut Length: ULONG;
    let mut Status: NTSTATUS;

    Status = SamrConnect(
        core::ptr::null_mut(),
        &mut ServerHandle,
        SAM_SERVER_LOOKUP_DOMAIN,
    );

    'done: {
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Status = SampOpenDbObject(
            ServerHandle as PSAM_DB_OBJECT,
            w!("Domains").as_ptr() as *mut u16,
            w!("Account").as_ptr() as *mut u16,
            0,
            SamDbDomainObject,
            DOMAIN_READ_PASSWORD_PARAMETERS,
            &mut DomainObject,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        Length = core::mem::size_of::<SAM_DOMAIN_FIXED_DATA>() as ULONG;
        Status = SampGetObjectAttribute(
            DomainObject,
            w!("F").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            &mut FixedData as *mut _ as *mut core::ffi::c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        (*PasswordInformation).MinPasswordLength = FixedData.MinPasswordLength;
        (*PasswordInformation).PasswordProperties = FixedData.PasswordProperties;
    }

    if !DomainObject.is_null() {
        SampCloseDbObject(DomainObject);
    }

    if !ServerHandle.is_null() {
        SamrCloseHandle(&mut ServerHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrConnect2(
    ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    DesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    let mut InRevisionInfo: SAMPR_REVISION_INFO = core::mem::zeroed();
    let mut OutRevisionInfo: SAMPR_REVISION_INFO = core::mem::zeroed();
    let mut OutVersion: ULONG = 0;

    TRACE!(
        b"SamrConnect2(%p %p %lx)\n\0".as_ptr() as *const core::ffi::c_char,
        ServerName,
        ServerHandle,
        DesiredAccess,
    );

    InRevisionInfo.V1.Revision = 1;
    InRevisionInfo.V1.SupportedFeatures = 0;

    SamrConnect5(
        ServerName,
        DesiredAccess,
        1,
        &mut InRevisionInfo,
        &mut OutVersion,
        &mut OutRevisionInfo,
        ServerHandle,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetInformationUser2(
    UserHandle: SAMPR_HANDLE,
    UserInformationClass: USER_INFORMATION_CLASS,
    Buffer: PSAMPR_USER_INFO_BUFFER,
) -> NTSTATUS {
    let mut UserObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let DesiredAccess: ACCESS_MASK;
    let mut Status: NTSTATUS;

    match UserInformationClass {
        UserLogonHoursInformation
        | UserNameInformation
        | UserAccountNameInformation
        | UserFullNameInformation
        | UserPrimaryGroupInformation
        | UserHomeInformation
        | UserScriptInformation
        | UserProfileInformation
        | UserAdminCommentInformation
        | UserWorkStationsInformation
        | UserControlInformation
        | UserExpiresInformation
        | UserParametersInformation => {
            DesiredAccess = USER_WRITE_ACCOUNT;
        }
        UserGeneralInformation => {
            DesiredAccess = USER_WRITE_ACCOUNT | USER_WRITE_PREFERENCES;
        }
        UserPreferencesInformation => {
            DesiredAccess = USER_WRITE_PREFERENCES;
        }
        UserSetPasswordInformation | UserInternal1Information => {
            DesiredAccess = USER_FORCE_PASSWORD_CHANGE;
        }
        UserAllInformation | UserInternal2Information => {
            DesiredAccess = 0;
        }
        _ => {
            return STATUS_INVALID_INFO_CLASS;
        }
    }

    RtlAcquireResourceExclusive(&raw mut SampResource, 1u8);

    Status = SampValidateDbObject(
        UserHandle,
        SamDbUserObject,
        DesiredAccess,
        &mut UserObject,
    );
    if !NT_SUCCESS(Status) {
        RtlReleaseResource(&raw mut SampResource);
        return Status;
    }

    match UserInformationClass {
        UserGeneralInformation => {
            Status = SampSetUserGeneral(UserObject, Buffer);
        }
        UserPreferencesInformation => {
            Status = SampSetUserPreferences(UserObject, Buffer);
        }
        UserLogonHoursInformation => {
            Status = SampSetLogonHoursAttribute(
                UserObject,
                core::ptr::addr_of_mut!((*Buffer).LogonHours.LogonHours),
            );
        }
        UserNameInformation => {
            Status = SampSetUserName(
                UserObject,
                core::ptr::addr_of_mut!((*Buffer).Name.UserName),
            );
            if NT_SUCCESS(Status) {
                Status = SampSetObjectAttributeString(
                    UserObject,
                    wstr!("FullName"),
                    core::ptr::addr_of_mut!((*Buffer).Name.FullName),
                );
            }
        }
        UserAccountNameInformation => {
            Status = SampSetUserName(
                UserObject,
                core::ptr::addr_of_mut!((*Buffer).AccountName.UserName),
            );
        }
        UserFullNameInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("FullName"),
                core::ptr::addr_of_mut!((*Buffer).FullName.FullName),
            );
        }
        UserPrimaryGroupInformation => {
            Status = SampSetUserPrimaryGroup(UserObject, Buffer);
        }
        UserHomeInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("HomeDirectory"),
                core::ptr::addr_of_mut!((*Buffer).Home.HomeDirectory),
            );
            if NT_SUCCESS(Status) {
                Status = SampSetObjectAttributeString(
                    UserObject,
                    wstr!("HomeDirectoryDrive"),
                    core::ptr::addr_of_mut!((*Buffer).Home.HomeDirectoryDrive),
                );
            }
        }
        UserScriptInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("ScriptPath"),
                core::ptr::addr_of_mut!((*Buffer).Script.ScriptPath),
            );
        }
        UserProfileInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("ProfilePath"),
                core::ptr::addr_of_mut!((*Buffer).Profile.ProfilePath),
            );
        }
        UserAdminCommentInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("AdminComment"),
                core::ptr::addr_of_mut!((*Buffer).AdminComment.AdminComment),
            );
        }
        UserWorkStationsInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("WorkStations"),
                core::ptr::addr_of_mut!((*Buffer).WorkStations.WorkStations),
            );
        }
        UserSetPasswordInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("Password"),
                core::ptr::addr_of_mut!((*Buffer).SetPassword.Password),
            );
        }
        UserControlInformation => {
            Status = SampSetUserControl(UserObject, Buffer);
        }
        UserExpiresInformation => {
            Status = SampSetUserExpires(UserObject, Buffer);
        }
        UserInternal1Information => {
            Status = SampSetUserInternal1(UserObject, Buffer);
        }
        UserInternal2Information => {
            Status = SampSetUserInternal2(UserObject, Buffer);
        }
        UserParametersInformation => {
            Status = SampSetObjectAttributeString(
                UserObject,
                wstr!("Parameters"),
                core::ptr::addr_of_mut!((*Buffer).Parameters.Parameters),
            );
        }
        UserAllInformation => {
            Status = SampSetUserAll(UserObject, Buffer);
        }
        _ => {
            Status = STATUS_INVALID_INFO_CLASS;
        }
    }

    RtlReleaseResource(&raw mut SampResource);

    Status
}
#[no_mangle]
pub unsafe extern "system" fn SamrSetBootKeyInformation(_BindingHandle: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn SamrGetBootKeyInformation(_BindingHandle: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn SamrConnect3(_BindingHandle: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrConnect4(
    ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    _ClientRevision: core::ffi::c_ulong,
    DesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    let mut InRevisionInfo: SAMPR_REVISION_INFO = core::mem::zeroed();
    let mut OutRevisionInfo: SAMPR_REVISION_INFO = core::mem::zeroed();
    let mut OutVersion: ULONG = 0;

    TRACE!(
        b"SamrConnect4(%p %p %lu 0x%lx)\n\0".as_ptr() as *const core::ffi::c_char,
        ServerName,
        ServerHandle,
        ClientRevision,
        DesiredAccess,
    );

    InRevisionInfo.V1.Revision = 2;
    InRevisionInfo.V1.SupportedFeatures = 0;

    SamrConnect5(
        ServerName,
        DesiredAccess,
        1,
        &mut InRevisionInfo,
        &mut OutVersion,
        &mut OutRevisionInfo,
        ServerHandle,
    )
}
#[no_mangle]
pub unsafe extern "system" fn SamrUnicodeChangePasswordUser3(_BindingHandle: handle_t) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrConnect5(
    _ServerName: PSAMPR_SERVER_NAME,
    mut DesiredAccess: ACCESS_MASK,
    InVersion: core::ffi::c_ulong,
    _InRevisionInfo: *mut SAMPR_REVISION_INFO,
    OutVersion: *mut core::ffi::c_ulong,
    OutRevisionInfo: *mut SAMPR_REVISION_INFO,
    ServerHandle: *mut SAMPR_HANDLE,
) -> NTSTATUS {
    let mut ServerObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"SamrConnect5(%p 0x%lx %lu %p %p %p %p)\n\0".as_ptr() as *const i8,
        ServerName,
        DesiredAccess,
        InVersion,
        InRevisionInfo,
        OutVersion,
        OutRevisionInfo,
        ServerHandle,
    );

    if InVersion != 1 {
        return STATUS_NOT_SUPPORTED;
    }

    RtlAcquireResourceShared(core::ptr::addr_of_mut!(SampResource), 1u8);

    RtlMapGenericMask(
        core::ptr::addr_of_mut!(DesiredAccess),
        core::ptr::addr_of_mut!(ServerMapping),
    );

    Status = SampOpenDbObject(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        ['S' as u16, 'A' as u16, 'M' as u16, 0].as_ptr() as *mut u16,
        0,
        SamDbServerObject,
        DesiredAccess,
        core::ptr::addr_of_mut!(ServerObject),
    );
    if NT_SUCCESS(Status) {
        *OutVersion = 1;

        (*OutRevisionInfo).V1.Revision = 3;
        (*OutRevisionInfo).V1.SupportedFeatures = 0;

        *ServerHandle = ServerObject as SAMPR_HANDLE;
    }

    RtlReleaseResource(core::ptr::addr_of_mut!(SampResource));

    TRACE!(
        b"SamrConnect5 done (Status 0x%08lx)\n\0".as_ptr() as *const i8,
        Status,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamrRidToSid(
    _ObjectHandle: SAMPR_HANDLE,
    _Rid: ::core::ffi::c_ulong,
    _Sid: *mut PRPC_SID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrSetDSRMPassword(
    _BindingHandle: handle_t,
    _Unused: PRPC_UNICODE_STRING,
    _UserId: core::ffi::c_ulong,
    _EncryptedNtOwfPassword: PENCRYPTED_NT_OWF_PASSWORD,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SamrValidatePassword(
    _Handle: handle_t,
    _ValidationType: PASSWORD_POLICY_VALIDATION_TYPE,
    _InputArg: PSAM_VALIDATE_INPUT_ARG,
    _OutputArg: *mut PSAM_VALIDATE_OUTPUT_ARG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Security Account Manager (SAM) Server
//  * FILE:            reactos/dll/win32/samsrv/samrpc.c
//  * PURPOSE:         RPC interface functions
//  *
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// #include "samsrv.h"
//
// /* GLOBALS *******************************************************************/
//
// static SID_IDENTIFIER_AUTHORITY NtSidAuthority = {SECURITY_NT_AUTHORITY};
//
// static GENERIC_MAPPING ServerMapping =
// {
//     SAM_SERVER_READ,
//     SAM_SERVER_WRITE,
//     SAM_SERVER_EXECUTE,
//     SAM_SERVER_ALL_ACCESS
// };
//
// static GENERIC_MAPPING DomainMapping =
// {
//     DOMAIN_READ,
//     DOMAIN_WRITE,
//     DOMAIN_EXECUTE,
//     DOMAIN_ALL_ACCESS
// };
//
// static GENERIC_MAPPING AliasMapping =
// {
//     ALIAS_READ,
//     ALIAS_WRITE,
//     ALIAS_EXECUTE,
//     ALIAS_ALL_ACCESS
// };
//
// static GENERIC_MAPPING GroupMapping =
// {
//     GROUP_READ,
//     GROUP_WRITE,
//     GROUP_EXECUTE,
//     GROUP_ALL_ACCESS
// };
//
// static GENERIC_MAPPING UserMapping =
// {
//     USER_READ,
//     USER_WRITE,
//     USER_EXECUTE,
//     USER_ALL_ACCESS
// };
//
// PGENERIC_MAPPING pServerMapping = &ServerMapping;
//
//
// /* FUNCTIONS *****************************************************************/
//
// static
// LARGE_INTEGER
// SampAddRelativeTimeToTime(IN LARGE_INTEGER AbsoluteTime,
//                           IN LARGE_INTEGER RelativeTime)
// {
//     LARGE_INTEGER NewTime;
//
//     NewTime.QuadPart = AbsoluteTime.QuadPart - RelativeTime.QuadPart;
//
//     if (NewTime.QuadPart < 0)
//         NewTime.QuadPart = 0;
//
//     return NewTime;
// }
//
//
// VOID
// SampStartRpcServer(VOID)
// {
//     RPC_STATUS Status;
//
//     TRACE("SampStartRpcServer() called\n");
//
//     Status = RpcServerUseProtseqEpW(L"ncacn_np",
//                                     RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
//                                     L"\\pipe\\samr",
//                                     NULL);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return;
//     }
//
//     Status = RpcServerRegisterIf(samr_v1_0_s_ifspec,
//                                  NULL,
//                                  NULL);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return;
//     }
//
//     Status = RpcServerListen(1, 20, TRUE);
//     if (Status != RPC_S_OK)
//     {
//         WARN("RpcServerListen() failed (Status %lx)\n", Status);
//         return;
//     }
//
//     TRACE("SampStartRpcServer() done\n");
// }
//
//
// void __RPC_FAR * __RPC_USER midl_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER midl_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// void __RPC_USER SAMPR_HANDLE_rundown(SAMPR_HANDLE hHandle)
// {
//     FIXME("SAMPR_HANDLE_rundown(%p)\n", hHandle);
// }
//
//
// /* Function 0 */
// NTSTATUS
// NTAPI
// SamrConnect(IN PSAMPR_SERVER_NAME ServerName,
//             OUT SAMPR_HANDLE *ServerHandle,
//             IN ACCESS_MASK DesiredAccess)
// {
//     SAMPR_REVISION_INFO InRevisionInfo, OutRevisionInfo;
//     ULONG OutVersion;
//
//     TRACE("SamrConnect(%p %p %lx)\n",
//           ServerName, ServerHandle, DesiredAccess);
//
//     InRevisionInfo.V1.Revision = 0;
//     InRevisionInfo.V1.SupportedFeatures = 0;
//
//     return SamrConnect5(ServerName,
//                         DesiredAccess,
//                         1,
//                         &InRevisionInfo,
//                         &OutVersion,
//                         &OutRevisionInfo,
//                         ServerHandle);
// }
//
//
// /* Function 1 */
// NTSTATUS
// NTAPI
// SamrCloseHandle(IN OUT SAMPR_HANDLE *SamHandle)
// {
//     PSAM_DB_OBJECT DbObject;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("SamrCloseHandle(%p)\n", SamHandle);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     Status = SampValidateDbObject(*SamHandle,
//                                   SamDbIgnoreObject,
//                                   0,
//                                   &DbObject);
//     if (Status == STATUS_SUCCESS)
//     {
//         Status = SampCloseDbObject(DbObject);
//         *SamHandle = NULL;
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("SamrCloseHandle done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /* Function 2 */
// NTSTATUS
// NTAPI
// SamrSetSecurityObject(IN SAMPR_HANDLE ObjectHandle,
//                       IN SECURITY_INFORMATION SecurityInformation,
//                       IN PSAMPR_SR_SECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     PSAM_DB_OBJECT DbObject = NULL;
//     ACCESS_MASK DesiredAccess = 0;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     ULONG RelativeSdSize = 0;
//     HANDLE TokenHandle = NULL;
//     PGENERIC_MAPPING Mapping;
//     NTSTATUS Status;
//
//     TRACE("SamrSetSecurityObject(%p %lx %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     if ((SecurityDescriptor == NULL) ||
//         (SecurityDescriptor->SecurityDescriptor == NULL) ||
//         !RtlValidSecurityDescriptor((PSECURITY_DESCRIPTOR)SecurityDescriptor->SecurityDescriptor))
//         return ERROR_INVALID_PARAMETER;
//
//     if (SecurityInformation == 0 ||
//         SecurityInformation & ~(OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION
//         | DACL_SECURITY_INFORMATION | SACL_SECURITY_INFORMATION))
//         return ERROR_INVALID_PARAMETER;
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//         DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//
//     if (SecurityInformation & DACL_SECURITY_INFORMATION)
//         DesiredAccess |= WRITE_DAC;
//
//     if (SecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION))
//         DesiredAccess |= WRITE_OWNER;
//
//     if ((SecurityInformation & OWNER_SECURITY_INFORMATION) &&
//         (((PISECURITY_DESCRIPTOR)SecurityDescriptor)->Owner == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     if ((SecurityInformation & GROUP_SECURITY_INFORMATION) &&
//         (((PISECURITY_DESCRIPTOR)SecurityDescriptor)->Group == NULL))
//         return ERROR_INVALID_PARAMETER;
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(ObjectHandle,
//                                   SamDbIgnoreObject,
//                                   DesiredAccess,
//                                   &DbObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the mapping for the object type */
//     switch (DbObject->ObjectType)
//     {
//         case SamDbServerObject:
//             Mapping = &ServerMapping;
//             break;
//
//         case SamDbDomainObject:
//             Mapping = &DomainMapping;
//             break;
//
//         case SamDbAliasObject:
//             Mapping = &AliasMapping;
//             break;
//
//         case SamDbGroupObject:
//             Mapping = &GroupMapping;
//             break;
//
//         case SamDbUserObject:
//             Mapping = &UserMapping;
//             break;
//
//         default:
//             return STATUS_INVALID_HANDLE;
//     }
//
//     /* Get the size of the SD */
//     Status = SampGetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     NULL,
//                                     NULL,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Allocate a buffer for the SD */
//     RelativeSd = RtlAllocateHeap(RtlGetProcessHeap(), 0, RelativeSdSize);
//     if (RelativeSd == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the SD */
//     Status = SampGetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     NULL,
//                                     RelativeSd,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Build the new security descriptor */
//     Status = RtlSetSecurityObject(SecurityInformation,
//                                   (PSECURITY_DESCRIPTOR)SecurityDescriptor->SecurityDescriptor,
//                                   &RelativeSd,
//                                   Mapping,
//                                   TokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("RtlSetSecurityObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Set the modified SD */
//     Status = SampSetObjectAttribute(DbObject,
//                                     L"SecDesc",
//                                     REG_BINARY,
//                                     RelativeSd,
//                                     RtlLengthSecurityDescriptor(RelativeSd));
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (TokenHandle != NULL)
//         NtClose(TokenHandle);
//
//     if (RelativeSd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSd);
//
//     return Status;
// }
//
//
// /* Function 3 */
// NTSTATUS
// NTAPI
// SamrQuerySecurityObject(IN SAMPR_HANDLE ObjectHandle,
//                         IN SECURITY_INFORMATION SecurityInformation,
//                         OUT PSAMPR_SR_SECURITY_DESCRIPTOR *SecurityDescriptor)
// {
//     PSAM_DB_OBJECT SamObject;
//     PSAMPR_SR_SECURITY_DESCRIPTOR SdData = NULL;
//     PISECURITY_DESCRIPTOR_RELATIVE RelativeSd = NULL;
//     PSECURITY_DESCRIPTOR ResultSd = NULL;
//     ACCESS_MASK DesiredAccess = 0;
//     ULONG RelativeSdSize = 0;
//     ULONG ResultSdSize = 0;
//     NTSTATUS Status;
//
//     TRACE("SamrQuerySecurityObject(%p %lx %p)\n",
//           ObjectHandle, SecurityInformation, SecurityDescriptor);
//
//     *SecurityDescriptor = NULL;
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     if (SecurityInformation & (DACL_SECURITY_INFORMATION |
//                                OWNER_SECURITY_INFORMATION |
//                                GROUP_SECURITY_INFORMATION))
//         DesiredAccess |= READ_CONTROL;
//
//     if (SecurityInformation & SACL_SECURITY_INFORMATION)
//         DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(ObjectHandle,
//                                   SamDbIgnoreObject,
//                                   DesiredAccess,
//                                   &SamObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the size of the SD */
//     Status = SampGetObjectAttribute(SamObject,
//                                     L"SecDesc",
//                                     NULL,
//                                     NULL,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Allocate a buffer for the SD */
//     RelativeSd = midl_user_allocate(RelativeSdSize);
//     if (RelativeSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Get the SD */
//     Status = SampGetObjectAttribute(SamObject,
//                                     L"SecDesc",
//                                     NULL,
//                                     RelativeSd,
//                                     &RelativeSdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Invalidate the SD information that was not requested */
//     if (!(SecurityInformation & OWNER_SECURITY_INFORMATION))
//         RelativeSd->Owner = 0;
//
//     if (!(SecurityInformation & GROUP_SECURITY_INFORMATION))
//         RelativeSd->Group = 0;
//
//     if (!(SecurityInformation & DACL_SECURITY_INFORMATION))
//         RelativeSd->Control &= ~SE_DACL_PRESENT;
//
//     if (!(SecurityInformation & SACL_SECURITY_INFORMATION))
//         RelativeSd->Control &= ~SE_SACL_PRESENT;
//
//     /* Calculate the required SD size */
//     Status = RtlMakeSelfRelativeSD(RelativeSd,
//                                    NULL,
//                                    &ResultSdSize);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//         goto done;
//
//     /* Allocate a buffer for the new SD */
//     ResultSd = MIDL_user_allocate(ResultSdSize);
//     if (ResultSd == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Build the new SD */
//     Status = RtlMakeSelfRelativeSD(RelativeSd,
//                                    ResultSd,
//                                    &ResultSdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Allocate the SD data buffer */
//     SdData = midl_user_allocate(sizeof(SAMPR_SR_SECURITY_DESCRIPTOR));
//     if (SdData == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Fill the SD data buffer and return it to the caller */
//     SdData->Length = ResultSdSize;
//     SdData->SecurityDescriptor = (PBYTE)ResultSd;
//
//     *SecurityDescriptor = SdData;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (ResultSd != NULL)
//             MIDL_user_free(ResultSd);
//     }
//
//     if (RelativeSd != NULL)
//         MIDL_user_free(RelativeSd);
//
//     return Status;
// }
//
//
// /* Function 4 */
// NTSTATUS
// NTAPI
// SamrShutdownSamServer(IN SAMPR_HANDLE ServerHandle)
// {
//     PSAM_DB_OBJECT ServerObject;
//     NTSTATUS Status;
//
//     TRACE("SamrShutdownSamServer(%p)\n",
//           ServerHandle);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(ServerHandle,
//                                   SamDbServerObject,
//                                   SAM_SERVER_SHUTDOWN,
//                                   &ServerObject);
//
//     RtlReleaseResource(&SampResource);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Shut the server down */
//     RpcMgmtStopServerListening(0);
//
//     Status = SampShutdownDisplayCache();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampShutdownDisplayCache() failed (Status 0x%08lx)\n", Status);
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 5 */
// NTSTATUS
// NTAPI
// SamrLookupDomainInSamServer(IN SAMPR_HANDLE ServerHandle,
//                             IN PRPC_UNICODE_STRING Name,
//                             OUT PRPC_SID *DomainId)
// {
//     PSAM_DB_OBJECT ServerObject;
//     HANDLE DomainsKeyHandle = NULL;
//     HANDLE DomainKeyHandle = NULL;
//     WCHAR DomainKeyName[64];
//     ULONG Index;
//     WCHAR DomainNameString[MAX_COMPUTERNAME_LENGTH + 1];
//     UNICODE_STRING DomainName;
//     ULONG Length;
//     BOOL Found = FALSE;
//     NTSTATUS Status;
//
//     TRACE("SamrLookupDomainInSamServer(%p %p %p)\n",
//           ServerHandle, Name, DomainId);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(ServerHandle,
//                                   SamDbServerObject,
//                                   SAM_SERVER_LOOKUP_DOMAIN,
//                                   &ServerObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *DomainId = NULL;
//
//     Status = SampRegOpenKey(ServerObject->KeyHandle,
//                             L"Domains",
//                             KEY_READ,
//                             &DomainsKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Index = 0;
//     while (Found == FALSE)
//     {
//         Status = SampRegEnumerateSubKey(DomainsKeyHandle,
//                                         Index,
//                                         64,
//                                         DomainKeyName);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_NO_SUCH_DOMAIN;
//             break;
//         }
//
//         TRACE("Domain key name: %S\n", DomainKeyName);
//
//         Status = SampRegOpenKey(DomainsKeyHandle,
//                                 DomainKeyName,
//                                 KEY_READ,
//                                 &DomainKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Length = (MAX_COMPUTERNAME_LENGTH + 1) * sizeof(WCHAR);
//             Status = SampRegQueryValue(DomainKeyHandle,
//                                        L"Name",
//                                        NULL,
//                                        (PVOID)&DomainNameString,
//                                        &Length);
//             if (NT_SUCCESS(Status))
//             {
//                 TRACE("Domain name: %S\n", DomainNameString);
//
//                 RtlInitUnicodeString(&DomainName,
//                                      DomainNameString);
//                 if (RtlEqualUnicodeString(&DomainName, (PUNICODE_STRING)Name, TRUE))
//                 {
//                    TRACE("Found it!\n");
//                    Found = TRUE;
//
//                    Status = SampRegQueryValue(DomainKeyHandle,
//                                               L"SID",
//                                               NULL,
//                                               NULL,
//                                               &Length);
//                    if (NT_SUCCESS(Status))
//                    {
//                        *DomainId = midl_user_allocate(Length);
//
//                        SampRegQueryValue(DomainKeyHandle,
//                                          L"SID",
//                                          NULL,
//                                          (PVOID)*DomainId,
//                                          &Length);
//
//                        Status = STATUS_SUCCESS;
//                        break;
//                    }
//                 }
//             }
//
//             SampRegCloseKey(&DomainKeyHandle);
//         }
//
//         Index++;
//     }
//
// done:
//     SampRegCloseKey(&DomainKeyHandle);
//     SampRegCloseKey(&DomainsKeyHandle);
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 6 */
// NTSTATUS
// NTAPI
// SamrEnumerateDomainsInSamServer(IN SAMPR_HANDLE ServerHandle,
//                                 IN OUT unsigned long *EnumerationContext,
//                                 OUT PSAMPR_ENUMERATION_BUFFER *Buffer,
//                                 IN ULONG PreferedMaximumLength,
//                                 OUT PULONG CountReturned)
// {
//     PSAM_DB_OBJECT ServerObject;
//     WCHAR DomainKeyName[64];
//     HANDLE DomainsKeyHandle = NULL;
//     HANDLE DomainKeyHandle = NULL;
//     ULONG EnumIndex;
//     ULONG EnumCount;
//     ULONG RequiredLength;
//     ULONG DataLength;
//     ULONG i;
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamrEnumerateDomainsInSamServer(%p %p %p %lu %p)\n",
//           ServerHandle, EnumerationContext, Buffer, PreferedMaximumLength,
//           CountReturned);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(ServerHandle,
//                                   SamDbServerObject,
//                                   SAM_SERVER_ENUMERATE_DOMAINS,
//                                   &ServerObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(ServerObject->KeyHandle,
//                             L"Domains",
//                             KEY_READ,
//                             &DomainsKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     EnumIndex = *EnumerationContext;
//     EnumCount = 0;
//     RequiredLength = 0;
//
//     while (TRUE)
//     {
//         Status = SampRegEnumerateSubKey(DomainsKeyHandle,
//                                         EnumIndex,
//                                         64 * sizeof(WCHAR),
//                                         DomainKeyName);
//         if (!NT_SUCCESS(Status))
//             break;
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Domain key name: %S\n", DomainKeyName);
//
//         Status = SampRegOpenKey(DomainsKeyHandle,
//                                 DomainKeyName,
//                                 KEY_READ,
//                                 &DomainKeyHandle);
//         TRACE("SampRegOpenKey returned %08lX\n", Status);
//         if (NT_SUCCESS(Status))
//         {
//             DataLength = 0;
//             Status = SampRegQueryValue(DomainKeyHandle,
//                                        L"Name",
//                                        NULL,
//                                        NULL,
//                                        &DataLength);
//             TRACE("SampRegQueryValue returned %08lX\n", Status);
//             if (NT_SUCCESS(Status))
//             {
//                 TRACE("Data length: %lu\n", DataLength);
//
//                 if ((RequiredLength + DataLength + sizeof(UNICODE_STRING)) > PreferedMaximumLength)
//                     break;
//
//                 RequiredLength += (DataLength + sizeof(UNICODE_STRING));
//                 EnumCount++;
//             }
//
//             SampRegCloseKey(&DomainKeyHandle);
//         }
//
//         EnumIndex++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//     TRACE("RequiredLength: %lu\n", RequiredLength);
//
//     EnumBuffer = midl_user_allocate(sizeof(SAMPR_ENUMERATION_BUFFER));
//     if (EnumBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumBuffer->EntriesRead = EnumCount;
//     EnumBuffer->Buffer = midl_user_allocate(EnumCount * sizeof(SAMPR_RID_ENUMERATION));
//     if (EnumBuffer->Buffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumIndex = *EnumerationContext;
//     for (i = 0; i < EnumCount; i++, EnumIndex++)
//     {
//         Status = SampRegEnumerateSubKey(DomainsKeyHandle,
//                                         EnumIndex,
//                                         64 * sizeof(WCHAR),
//                                         DomainKeyName);
//         if (!NT_SUCCESS(Status))
//             break;
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Domain key name: %S\n", DomainKeyName);
//
//         Status = SampRegOpenKey(DomainsKeyHandle,
//                                 DomainKeyName,
//                                 KEY_READ,
//                                 &DomainKeyHandle);
//         TRACE("SampRegOpenKey returned %08lX\n", Status);
//         if (NT_SUCCESS(Status))
//         {
//             DataLength = 0;
//             Status = SampRegQueryValue(DomainKeyHandle,
//                                        L"Name",
//                                        NULL,
//                                        NULL,
//                                        &DataLength);
//             TRACE("SampRegQueryValue returned %08lX\n", Status);
//             if (NT_SUCCESS(Status))
//             {
//                 EnumBuffer->Buffer[i].RelativeId = 0;
//                 EnumBuffer->Buffer[i].Name.Length = (USHORT)DataLength - sizeof(WCHAR);
//                 EnumBuffer->Buffer[i].Name.MaximumLength = (USHORT)DataLength;
//                 EnumBuffer->Buffer[i].Name.Buffer = midl_user_allocate(DataLength);
//                 if (EnumBuffer->Buffer[i].Name.Buffer == NULL)
//                 {
//                     SampRegCloseKey(&DomainKeyHandle);
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto done;
//                 }
//
//                 Status = SampRegQueryValue(DomainKeyHandle,
//                                            L"Name",
//                                            NULL,
//                                            EnumBuffer->Buffer[i].Name.Buffer,
//                                            &DataLength);
//                 TRACE("SampRegQueryValue returned %08lX\n", Status);
//                 if (NT_SUCCESS(Status))
//                 {
//                     TRACE("Domain name: %S\n", EnumBuffer->Buffer[i].Name.Buffer);
//                 }
//             }
//
//             SampRegCloseKey(&DomainKeyHandle);
//
//             if (!NT_SUCCESS(Status))
//                 goto done;
//         }
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *EnumerationContext += EnumCount;
//         *Buffer = EnumBuffer;
//         *CountReturned = EnumCount;
//     }
//
// done:
//     SampRegCloseKey(&DomainKeyHandle);
//     SampRegCloseKey(&DomainsKeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         *EnumerationContext = 0;
//         *Buffer = NULL;
//         *CountReturned = 0;
//
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 if (EnumBuffer->EntriesRead != 0)
//                 {
//                     for (i = 0; i < EnumBuffer->EntriesRead; i++)
//                     {
//                         if (EnumBuffer->Buffer[i].Name.Buffer != NULL)
//                             midl_user_free(EnumBuffer->Buffer[i].Name.Buffer);
//                     }
//                 }
//
//                 midl_user_free(EnumBuffer->Buffer);
//             }
//
//             midl_user_free(EnumBuffer);
//         }
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 7 */
// NTSTATUS
// NTAPI
// SamrOpenDomain(IN SAMPR_HANDLE ServerHandle,
//                IN ACCESS_MASK DesiredAccess,
//                IN PRPC_SID DomainId,
//                OUT SAMPR_HANDLE *DomainHandle)
// {
//     PSAM_DB_OBJECT ServerObject;
//     PSAM_DB_OBJECT DomainObject;
//     NTSTATUS Status;
//
//     TRACE("SamrOpenDomain(%p %lx %p %p)\n",
//           ServerHandle, DesiredAccess, DomainId, DomainHandle);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &DomainMapping);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(ServerHandle,
//                                   SamDbServerObject,
//                                   SAM_SERVER_LOOKUP_DOMAIN,
//                                   &ServerObject);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Validate the Domain SID */
//     if ((DomainId->Revision != SID_REVISION) ||
//         (DomainId->SubAuthorityCount > SID_MAX_SUB_AUTHORITIES) ||
//         (memcmp(&DomainId->IdentifierAuthority, &NtSidAuthority, sizeof(SID_IDENTIFIER_AUTHORITY)) != 0))
//         return STATUS_INVALID_PARAMETER;
//
//     /* Open the domain object */
//     if ((DomainId->SubAuthorityCount == 1) &&
//         (DomainId->SubAuthority[0] == SECURITY_BUILTIN_DOMAIN_RID))
//     {
//         /* Builtin domain object */
//         TRACE("Opening the builtin domain object.\n");
//
//         Status = SampOpenDbObject(ServerObject,
//                                   L"Domains",
//                                   L"Builtin",
//                                   0,
//                                   SamDbDomainObject,
//                                   DesiredAccess,
//                                   &DomainObject);
//     }
//     else if ((DomainId->SubAuthorityCount == 4) &&
//              (DomainId->SubAuthority[0] == SECURITY_NT_NON_UNIQUE))
//     {
//         /* Account domain object */
//         TRACE("Opening the account domain object.\n");
//
//         /* FIXME: Check the account domain sub authorities!!! */
//
//         Status = SampOpenDbObject(ServerObject,
//                                   L"Domains",
//                                   L"Account",
//                                   0,
//                                   SamDbDomainObject,
//                                   DesiredAccess,
//                                   &DomainObject);
//     }
//     else
//     {
//         /* No valid domain SID */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     if (NT_SUCCESS(Status))
//         *DomainHandle = (SAMPR_HANDLE)DomainObject;
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("SamrOpenDomain done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainPassword(PSAM_DB_OBJECT DomainObject,
//                         PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Password.MinPasswordLength = FixedData.MinPasswordLength;
//     InfoBuffer->Password.PasswordHistoryLength = FixedData.PasswordHistoryLength;
//     InfoBuffer->Password.PasswordProperties = FixedData.PasswordProperties;
//     InfoBuffer->Password.MaxPasswordAge.LowPart = FixedData.MaxPasswordAge.LowPart;
//     InfoBuffer->Password.MaxPasswordAge.HighPart = FixedData.MaxPasswordAge.HighPart;
//     InfoBuffer->Password.MinPasswordAge.LowPart = FixedData.MinPasswordAge.LowPart;
//     InfoBuffer->Password.MinPasswordAge.HighPart = FixedData.MinPasswordAge.HighPart;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampGetNumberOfAccounts(PSAM_DB_OBJECT DomainObject,
//                         LPCWSTR AccountType,
//                         PULONG Count)
// {
//     HANDLE AccountKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     NTSTATUS Status;
//
//     *Count = 0;
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             AccountType,
//                             KEY_READ,
//                             &AccountKeyHandle);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Status = SampRegOpenKey(AccountKeyHandle,
//                             L"Names",
//                             KEY_READ,
//                             &NamesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegQueryKeyInfo(NamesKeyHandle,
//                                  NULL,
//                                  Count);
//
// done:
//     SampRegCloseKey(&NamesKeyHandle);
//     SampRegCloseKey(&AccountKeyHandle);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainGeneral(PSAM_DB_OBJECT DomainObject,
//                        PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->General.ForceLogoff.LowPart = FixedData.ForceLogoff.LowPart;
//     InfoBuffer->General.ForceLogoff.HighPart = FixedData.ForceLogoff.HighPart;
//     InfoBuffer->General.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.LowPart;
//     InfoBuffer->General.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.HighPart;
//     InfoBuffer->General.DomainServerState = FixedData.DomainServerState;
//     InfoBuffer->General.DomainServerRole = FixedData.DomainServerRole;
//     InfoBuffer->General.UasCompatibilityRequired = FixedData.UasCompatibilityRequired;
//
//     /* Get the OemInformation string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"OemInformation",
//                                           &InfoBuffer->General.OemInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"Name",
//                                           &InfoBuffer->General.DomainName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the ReplicaSourceNodeName string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"ReplicaSourceNodeName",
//                                           &InfoBuffer->General.ReplicaSourceNodeName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of Users in the Domain */
//     Status = SampGetNumberOfAccounts(DomainObject,
//                                      L"Users",
//                                      &InfoBuffer->General.UserCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of Groups in the Domain */
//     Status = SampGetNumberOfAccounts(DomainObject,
//                                      L"Groups",
//                                      &InfoBuffer->General.GroupCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of Aliases in the Domain */
//     Status = SampGetNumberOfAccounts(DomainObject,
//                                      L"Aliases",
//                                      &InfoBuffer->General.AliasCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->General.OemInformation.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.OemInformation.Buffer);
//
//             if (InfoBuffer->General.DomainName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.DomainName.Buffer);
//
//             if (InfoBuffer->General.ReplicaSourceNodeName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.ReplicaSourceNodeName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainLogoff(PSAM_DB_OBJECT DomainObject,
//                       PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Logoff.ForceLogoff.LowPart = FixedData.ForceLogoff.LowPart;
//     InfoBuffer->Logoff.ForceLogoff.HighPart = FixedData.ForceLogoff.HighPart;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainOem(PSAM_DB_OBJECT DomainObject,
//                    PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the OemInformation string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"OemInformation",
//                                           &InfoBuffer->Oem.OemInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Oem.OemInformation.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Oem.OemInformation.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainName(PSAM_DB_OBJECT DomainObject,
//                     PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"Name",
//                                           &InfoBuffer->Name.DomainName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Name.DomainName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Name.DomainName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainReplication(PSAM_DB_OBJECT DomainObject,
//                            PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the ReplicaSourceNodeName string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"ReplicaSourceNodeName",
//                                           &InfoBuffer->Replication.ReplicaSourceNodeName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Replication.ReplicaSourceNodeName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Replication.ReplicaSourceNodeName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainServerRole(PSAM_DB_OBJECT DomainObject,
//                           PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Role.DomainServerRole = FixedData.DomainServerRole;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainModified(PSAM_DB_OBJECT DomainObject,
//                         PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Modified.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.LowPart;
//     InfoBuffer->Modified.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.HighPart;
//     InfoBuffer->Modified.CreationTime.LowPart = FixedData.CreationTime.LowPart;
//     InfoBuffer->Modified.CreationTime.HighPart = FixedData.CreationTime.HighPart;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainState(PSAM_DB_OBJECT DomainObject,
//                      PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->State.DomainServerState = FixedData.DomainServerState;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainGeneral2(PSAM_DB_OBJECT DomainObject,
//                         PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->General2.I1.ForceLogoff.LowPart = FixedData.ForceLogoff.LowPart;
//     InfoBuffer->General2.I1.ForceLogoff.HighPart = FixedData.ForceLogoff.HighPart;
//     InfoBuffer->General2.I1.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.LowPart;
//     InfoBuffer->General2.I1.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.HighPart;
//     InfoBuffer->General2.I1.DomainServerState = FixedData.DomainServerState;
//     InfoBuffer->General2.I1.DomainServerRole = FixedData.DomainServerRole;
//     InfoBuffer->General2.I1.UasCompatibilityRequired = FixedData.UasCompatibilityRequired;
//
//     InfoBuffer->General2.LockoutDuration = FixedData.LockoutDuration;
//     InfoBuffer->General2.LockoutObservationWindow = FixedData.LockoutObservationWindow;
//     InfoBuffer->General2.LockoutThreshold = FixedData.LockoutThreshold;
//
//     /* Get the OemInformation string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"OemInformation",
//                                           &InfoBuffer->General2.I1.OemInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"Name",
//                                           &InfoBuffer->General2.I1.DomainName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the ReplicaSourceNodeName string */
//     Status = SampGetObjectAttributeString(DomainObject,
//                                           L"ReplicaSourceNodeName",
//                                           &InfoBuffer->General2.I1.ReplicaSourceNodeName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of Users in the Domain */
//     Status = SampGetNumberOfAccounts(DomainObject,
//                                      L"Users",
//                                      &InfoBuffer->General2.I1.UserCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of Groups in the Domain */
//     Status = SampGetNumberOfAccounts(DomainObject,
//                                      L"Groups",
//                                      &InfoBuffer->General2.I1.GroupCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the number of Aliases in the Domain */
//     Status = SampGetNumberOfAccounts(DomainObject,
//                                      L"Aliases",
//                                      &InfoBuffer->General2.I1.AliasCount);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->General2.I1.OemInformation.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General2.I1.OemInformation.Buffer);
//
//             if (InfoBuffer->General2.I1.DomainName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General2.I1.DomainName.Buffer);
//
//             if (InfoBuffer->General2.I1.ReplicaSourceNodeName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General2.I1.ReplicaSourceNodeName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainLockout(PSAM_DB_OBJECT DomainObject,
//                        PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Lockout.LockoutDuration = FixedData.LockoutDuration;
//     InfoBuffer->Lockout.LockoutObservationWindow = FixedData.LockoutObservationWindow;
//     InfoBuffer->Lockout.LockoutThreshold = FixedData.LockoutThreshold;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryDomainModified2(PSAM_DB_OBJECT DomainObject,
//                         PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAMPR_DOMAIN_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_DOMAIN_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Modified2.DomainModifiedCount.LowPart = FixedData.DomainModifiedCount.LowPart;
//     InfoBuffer->Modified2.DomainModifiedCount.HighPart = FixedData.DomainModifiedCount.HighPart;
//     InfoBuffer->Modified2.CreationTime.LowPart = FixedData.CreationTime.LowPart;
//     InfoBuffer->Modified2.CreationTime.HighPart = FixedData.CreationTime.HighPart;
//     InfoBuffer->Modified2.ModifiedCountAtLastPromotion.LowPart = FixedData.ModifiedCountAtLastPromotion.LowPart;
//     InfoBuffer->Modified2.ModifiedCountAtLastPromotion.HighPart = FixedData.ModifiedCountAtLastPromotion.HighPart;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// /* Function 8 */
// NTSTATUS
// NTAPI
// SamrQueryInformationDomain(IN SAMPR_HANDLE DomainHandle,
//                            IN DOMAIN_INFORMATION_CLASS DomainInformationClass,
//                            OUT PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     TRACE("SamrQueryInformationDomain(%p %lu %p)\n",
//           DomainHandle, DomainInformationClass, Buffer);
//
//     return SamrQueryInformationDomain2(DomainHandle,
//                                        DomainInformationClass,
//                                        Buffer);
// }
//
//
// static NTSTATUS
// SampSetDomainPassword(PSAM_DB_OBJECT DomainObject,
//                       PSAMPR_DOMAIN_INFO_BUFFER Buffer)
// {
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.MinPasswordLength = Buffer->Password.MinPasswordLength;
//     FixedData.PasswordHistoryLength = Buffer->Password.PasswordHistoryLength;
//     FixedData.PasswordProperties = Buffer->Password.PasswordProperties;
//     FixedData.MaxPasswordAge.LowPart = Buffer->Password.MaxPasswordAge.LowPart;
//     FixedData.MaxPasswordAge.HighPart = Buffer->Password.MaxPasswordAge.HighPart;
//     FixedData.MinPasswordAge.LowPart = Buffer->Password.MinPasswordAge.LowPart;
//     FixedData.MinPasswordAge.HighPart = Buffer->Password.MinPasswordAge.HighPart;
//
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetDomainLogoff(PSAM_DB_OBJECT DomainObject,
//                     PSAMPR_DOMAIN_INFO_BUFFER Buffer)
// {
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.ForceLogoff.LowPart = Buffer->Logoff.ForceLogoff.LowPart;
//     FixedData.ForceLogoff.HighPart = Buffer->Logoff.ForceLogoff.HighPart;
//
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetDomainServerRole(PSAM_DB_OBJECT DomainObject,
//                         PSAMPR_DOMAIN_INFO_BUFFER Buffer)
// {
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.DomainServerRole = Buffer->Role.DomainServerRole;
//
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetDomainState(PSAM_DB_OBJECT DomainObject,
//                    PSAMPR_DOMAIN_INFO_BUFFER Buffer)
// {
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.DomainServerState = Buffer->State.DomainServerState;
//
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetDomainLockout(PSAM_DB_OBJECT DomainObject,
//                      PSAMPR_DOMAIN_INFO_BUFFER Buffer)
// {
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.LockoutDuration = Buffer->Lockout.LockoutDuration;
//     FixedData.LockoutObservationWindow = Buffer->Lockout.LockoutObservationWindow;
//     FixedData.LockoutThreshold = Buffer->Lockout.LockoutThreshold;
//
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// /* Function 9 */
// NTSTATUS
// NTAPI
// SamrSetInformationDomain(IN SAMPR_HANDLE DomainHandle,
//                          IN DOMAIN_INFORMATION_CLASS DomainInformationClass,
//                          IN PSAMPR_DOMAIN_INFO_BUFFER DomainInformation)
// {
//     PSAM_DB_OBJECT DomainObject;
//     ACCESS_MASK DesiredAccess;
//     NTSTATUS Status;
//
//     TRACE("SamrSetInformationDomain(%p %lu %p)\n",
//           DomainHandle, DomainInformationClass, DomainInformation);
//
//     switch (DomainInformationClass)
//     {
//         case DomainPasswordInformation:
//         case DomainLockoutInformation:
//             DesiredAccess = DOMAIN_WRITE_PASSWORD_PARAMS;
//             break;
//
//         case DomainLogoffInformation:
//         case DomainOemInformation:
//         case DomainNameInformation:
//             DesiredAccess = DOMAIN_WRITE_OTHER_PARAMETERS;
//             break;
//
//         case DomainReplicationInformation:
//         case DomainServerRoleInformation:
//         case DomainStateInformation:
//             DesiredAccess = DOMAIN_ADMINISTER_SERVER;
//             break;
//
//         default:
//             return STATUS_INVALID_INFO_CLASS;
//     }
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DesiredAccess,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     switch (DomainInformationClass)
//     {
//         case DomainPasswordInformation:
//             Status = SampSetDomainPassword(DomainObject,
//                                            DomainInformation);
//             break;
//
//         case DomainLogoffInformation:
//             Status = SampSetDomainLogoff(DomainObject,
//                                          DomainInformation);
//             break;
//
//         case DomainOemInformation:
//             Status = SampSetObjectAttributeString(DomainObject,
//                                                   L"OemInformation",
//                                                   &DomainInformation->Oem.OemInformation);
//             break;
//
//         case DomainNameInformation:
//             Status = SampSetObjectAttributeString(DomainObject,
//                                                   L"Name",
//                                                   &DomainInformation->Name.DomainName);
//             break;
//
//         case DomainReplicationInformation:
//             Status = SampSetObjectAttributeString(DomainObject,
//                                                   L"ReplicaSourceNodeName",
//                                                   &DomainInformation->Replication.ReplicaSourceNodeName);
//             break;
//
//         case DomainServerRoleInformation:
//             Status = SampSetDomainServerRole(DomainObject,
//                                              DomainInformation);
//             break;
//
//         case DomainStateInformation:
//             Status = SampSetDomainState(DomainObject,
//                                         DomainInformation);
//             break;
//
//         case DomainLockoutInformation:
//             Status = SampSetDomainLockout(DomainObject,
//                                           DomainInformation);
//             break;
//
//         default:
//             Status = STATUS_NOT_IMPLEMENTED;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 10 */
// NTSTATUS
// NTAPI
// SamrCreateGroupInDomain(IN SAMPR_HANDLE DomainHandle,
//                         IN PRPC_UNICODE_STRING Name,
//                         IN ACCESS_MASK DesiredAccess,
//                         OUT SAMPR_HANDLE *GroupHandle,
//                         OUT unsigned long *RelativeId)
// {
//     SAM_DOMAIN_FIXED_DATA FixedDomainData;
//     SAM_GROUP_FIXED_DATA FixedGroupData;
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT GroupObject;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     ULONG ulSize;
//     ULONG ulRid;
//     WCHAR szRid[9];
//     NTSTATUS Status;
//
//     TRACE("SamrCreateGroupInDomain(%p %p %lx %p %p)\n",
//           DomainHandle, Name, DesiredAccess, GroupHandle, RelativeId);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &GroupMapping);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_CREATE_GROUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Check the group account name */
//     Status = SampCheckAccountName(Name, 256);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check if the group name already exists in the domain */
//     Status = SampCheckAccountNameInDomain(DomainObject,
//                                           Name->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Group name \'%S\' already exists in domain (Status 0x%08lx)\n",
//               Name->Buffer, Status);
//         goto done;
//     }
//
//     /* Create the security descriptor */
//     Status = SampCreateGroupSD(&Sd,
//                                &SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCreateGroupSD failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Get the fixed domain attributes */
//     ulSize = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedDomainData,
//                                     &ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Increment the NextRid attribute */
//     ulRid = FixedDomainData.NextRid;
//     FixedDomainData.NextRid++;
//
//     /* Store the fixed domain attributes */
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedDomainData,
//                                     ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     TRACE("RID: %lx\n", ulRid);
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", ulRid);
//
//     /* Create the group object */
//     Status = SampCreateDbObject(DomainObject,
//                                 L"Groups",
//                                 szRid,
//                                 ulRid,
//                                 SamDbGroupObject,
//                                 DesiredAccess,
//                                 &GroupObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Add the account name of the user object */
//     Status = SampSetAccountNameInDomain(DomainObject,
//                                         L"Groups",
//                                         Name->Buffer,
//                                         ulRid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Initialize fixed user data */
//     memset(&FixedGroupData, 0, sizeof(SAM_GROUP_FIXED_DATA));
//     FixedGroupData.Version = 1;
//     FixedGroupData.GroupId = ulRid;
//
//     /* Set fixed user data attribute */
//     Status = SampSetObjectAttribute(GroupObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     (LPVOID)&FixedGroupData,
//                                     sizeof(SAM_GROUP_FIXED_DATA));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Name attribute */
//     Status = SampSetObjectAttributeString(GroupObject,
//                                           L"Name",
//                                           Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the AdminComment attribute */
//     Status = SampSetObjectAttributeString(GroupObject,
//                                           L"AdminComment",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the SecDesc attribute*/
//     Status = SampSetObjectAttribute(GroupObject,
//                                     L"SecDesc",
//                                     REG_BINARY,
//                                     Sd,
//                                     SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *GroupHandle = (SAMPR_HANDLE)GroupObject;
//         *RelativeId = ulRid;
//     }
//
// done:
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("returns with status 0x%08lx\n", Status);
//
//     return Status;
// }
//
//
// /* Function 11 */
// NTSTATUS
// NTAPI
// SamrEnumerateGroupsInDomain(IN SAMPR_HANDLE DomainHandle,
//                             IN OUT unsigned long *EnumerationContext,
//                             OUT PSAMPR_ENUMERATION_BUFFER *Buffer,
//                             IN unsigned long PreferedMaximumLength,
//                             OUT unsigned long *CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     PSAM_DB_OBJECT DomainObject;
//     HANDLE GroupsKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     WCHAR GroupName[64];
//     ULONG EnumIndex;
//     ULONG EnumCount = 0;
//     ULONG RequiredLength = 0;
//     ULONG NameLength;
//     ULONG DataLength;
//     ULONG Rid;
//     ULONG i;
//     BOOLEAN MoreEntries = FALSE;
//     NTSTATUS Status;
//
//     TRACE("SamrEnumerateUsersInDomain(%p %p %p %lu %p)\n",
//           DomainHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LIST_ACCOUNTS,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Groups",
//                             KEY_READ,
//                             &GroupsKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(GroupsKeyHandle,
//                             L"Names",
//                             KEY_READ,
//                             &NamesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     TRACE("Part 1\n");
//
//     EnumIndex = *EnumerationContext;
//
//     while (TRUE)
//     {
//         NameLength = 64 * sizeof(WCHAR);
//         Status = SampRegEnumerateValue(NamesKeyHandle,
//                                        EnumIndex,
//                                        GroupName,
//                                        &NameLength,
//                                        NULL,
//                                        NULL,
//                                        NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Group name: %S\n", GroupName);
//         TRACE("Name length: %lu\n", NameLength);
//
//         if ((RequiredLength + NameLength + sizeof(UNICODE_NULL) + sizeof(SAMPR_RID_ENUMERATION)) > PreferedMaximumLength)
//         {
//             MoreEntries = TRUE;
//             break;
//         }
//
//         RequiredLength += (NameLength + sizeof(UNICODE_NULL) + sizeof(SAMPR_RID_ENUMERATION));
//         EnumCount++;
//
//         EnumIndex++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//     TRACE("RequiredLength: %lu\n", RequiredLength);
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     EnumBuffer = midl_user_allocate(sizeof(SAMPR_ENUMERATION_BUFFER));
//     if (EnumBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumBuffer->EntriesRead = EnumCount;
//     if (EnumCount == 0)
//     {
//         Status = STATUS_NO_MORE_ENTRIES;
//         goto done;
//     }
//
//     EnumBuffer->Buffer = midl_user_allocate(EnumCount * sizeof(SAMPR_RID_ENUMERATION));
//     if (EnumBuffer->Buffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     TRACE("Part 2\n");
//
//     EnumIndex = *EnumerationContext;
//     for (i = 0; i < EnumCount; i++, EnumIndex++)
//     {
//         NameLength = 64 * sizeof(WCHAR);
//         DataLength = sizeof(ULONG);
//         Status = SampRegEnumerateValue(NamesKeyHandle,
//                                        EnumIndex,
//                                        GroupName,
//                                        &NameLength,
//                                        NULL,
//                                        &Rid,
//                                        &DataLength);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Group name: %S\n", GroupName);
//         TRACE("Name length: %lu\n", NameLength);
//         TRACE("RID: %lu\n", Rid);
//
//         EnumBuffer->Buffer[i].RelativeId = Rid;
//
//         EnumBuffer->Buffer[i].Name.Length = (USHORT)NameLength;
//         EnumBuffer->Buffer[i].Name.MaximumLength = (USHORT)(NameLength + sizeof(UNICODE_NULL));
//
// /* FIXME: Disabled because of bugs in widl and rpcrt4 */
//     }
//
// done:
//     if (NT_SUCCESS(Status))
//     {
//         *EnumerationContext += EnumCount;
//         *Buffer = EnumBuffer;
//         *CountReturned = EnumCount;
//     }
//     else
//     {
//         *EnumerationContext = 0;
//         *Buffer = NULL;
//         *CountReturned = 0;
//
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 if (EnumBuffer->EntriesRead != 0)
//                 {
//                     for (i = 0; i < EnumBuffer->EntriesRead; i++)
//                     {
//                         if (EnumBuffer->Buffer[i].Name.Buffer != NULL)
//                             midl_user_free(EnumBuffer->Buffer[i].Name.Buffer);
//                     }
//                 }
//
//                 midl_user_free(EnumBuffer->Buffer);
//             }
//
//             midl_user_free(EnumBuffer);
//         }
//     }
//
//     SampRegCloseKey(&NamesKeyHandle);
//     SampRegCloseKey(&GroupsKeyHandle);
//
//     if ((Status == STATUS_SUCCESS) && (MoreEntries != FALSE))
//         Status = STATUS_MORE_ENTRIES;
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 12 */
// NTSTATUS
// NTAPI
// SamrCreateUserInDomain(IN SAMPR_HANDLE DomainHandle,
//                        IN PRPC_UNICODE_STRING Name,
//                        IN ACCESS_MASK DesiredAccess,
//                        OUT SAMPR_HANDLE *UserHandle,
//                        OUT unsigned long *RelativeId)
// {
//     SAM_DOMAIN_FIXED_DATA FixedDomainData;
//     SAM_USER_FIXED_DATA FixedUserData;
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT UserObject;
//     GROUP_MEMBERSHIP GroupMembership;
//     UCHAR LogonHours[23];
//     ULONG ulSize;
//     ULONG ulRid;
//     WCHAR szRid[9];
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     PSID UserSid = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamrCreateUserInDomain(%p %p %lx %p %p)\n",
//           DomainHandle, Name, DesiredAccess, UserHandle, RelativeId);
//
//     if (Name == NULL ||
//         Name->Length == 0 ||
//         Name->Buffer == NULL ||
//         UserHandle == NULL ||
//         RelativeId == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &UserMapping);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_CREATE_USER,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Check the user account name */
//     Status = SampCheckAccountName(Name, 20);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check if the user name already exists in the domain */
//     Status = SampCheckAccountNameInDomain(DomainObject,
//                                           Name->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("User name \'%S\' already exists in domain (Status 0x%08lx)\n",
//               Name->Buffer, Status);
//         goto done;
//     }
//
//     /* Get the fixed domain attributes */
//     ulSize = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedDomainData,
//                                     &ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Increment the NextRid attribute */
//     ulRid = FixedDomainData.NextRid;
//     FixedDomainData.NextRid++;
//
//     TRACE("RID: %lx\n", ulRid);
//
//     /* Create the user SID */
//     Status = SampCreateAccountSid(DomainObject,
//                                   ulRid,
//                                   &UserSid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCreateAccountSid failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Create the security descriptor */
//     Status = SampCreateUserSD(UserSid,
//                               &Sd,
//                               &SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCreateUserSD failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Store the fixed domain attributes */
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedDomainData,
//                                     ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", ulRid);
//
//     /* Create the user object */
//     Status = SampCreateDbObject(DomainObject,
//                                 L"Users",
//                                 szRid,
//                                 ulRid,
//                                 SamDbUserObject,
//                                 DesiredAccess,
//                                 &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Add the account name for the user object */
//     Status = SampSetAccountNameInDomain(DomainObject,
//                                         L"Users",
//                                         Name->Buffer,
//                                         ulRid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Initialize fixed user data */
//     memset(&FixedUserData, 0, sizeof(SAM_USER_FIXED_DATA));
//     FixedUserData.Version = 1;
//     FixedUserData.Reserved = 0;
//     FixedUserData.LastLogon.QuadPart = 0;
//     FixedUserData.LastLogoff.QuadPart = 0;
//     FixedUserData.PasswordLastSet.QuadPart = 0;
//     FixedUserData.AccountExpires.QuadPart = MAXLONGLONG;
//     FixedUserData.LastBadPasswordTime.QuadPart = 0;
//     FixedUserData.UserId = ulRid;
//     FixedUserData.PrimaryGroupId = DOMAIN_GROUP_RID_USERS;
//     FixedUserData.UserAccountControl = USER_ACCOUNT_DISABLED |
//                                        USER_PASSWORD_NOT_REQUIRED |
//                                        USER_NORMAL_ACCOUNT;
//     FixedUserData.CountryCode = 0;
//     FixedUserData.CodePage = 0;
//     FixedUserData.BadPasswordCount = 0;
//     FixedUserData.LogonCount = 0;
//     FixedUserData.AdminCount = 0;
//     FixedUserData.OperatorCount = 0;
//
//     /* Set fixed user data attribute */
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     (LPVOID)&FixedUserData,
//                                     sizeof(SAM_USER_FIXED_DATA));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Name attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the FullName attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the HomeDirectory attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"HomeDirectory",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the HomeDirectoryDrive attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"HomeDirectoryDrive",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the ScriptPath attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"ScriptPath",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the ProfilePath attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"ProfilePath",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the AdminComment attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"AdminComment",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the UserComment attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"UserComment",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the WorkStations attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"WorkStations",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Parameters attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"Parameters",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set LogonHours attribute*/
//     *((PUSHORT)LogonHours) = 168;
//     memset(&(LogonHours[2]), 0xff, 21);
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LogonHours",
//                                     REG_BINARY,
//                                     &LogonHours,
//                                     sizeof(LogonHours));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set Groups attribute*/
//     GroupMembership.RelativeId = DOMAIN_GROUP_RID_USERS;
//     GroupMembership.Attributes = SE_GROUP_MANDATORY |
//                                  SE_GROUP_ENABLED |
//                                  SE_GROUP_ENABLED_BY_DEFAULT;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     REG_BINARY,
//                                     &GroupMembership,
//                                     sizeof(GROUP_MEMBERSHIP));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set LMPwd attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LMPwd",
//                                     REG_BINARY,
//                                     &EmptyLmHash,
//                                     sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set NTPwd attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"NTPwd",
//                                     REG_BINARY,
//                                     &EmptyNtHash,
//                                     sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set LMPwdHistory attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LMPwdHistory",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set NTPwdHistory attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"NTPwdHistory",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the PrivateData attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"PrivateData",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the SecDesc attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"SecDesc",
//                                     REG_BINARY,
//                                     Sd,
//                                     SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *UserHandle = (SAMPR_HANDLE)UserObject;
//         *RelativeId = ulRid;
//     }
//
// done:
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     if (UserSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("returns with status 0x%08lx\n", Status);
//
//     return Status;
// }
//
//
// /* Function 13 */
// NTSTATUS
// NTAPI
// SamrEnumerateUsersInDomain(IN SAMPR_HANDLE DomainHandle,
//                            IN OUT unsigned long *EnumerationContext,
//                            IN unsigned long UserAccountControl,
//                            OUT PSAMPR_ENUMERATION_BUFFER *Buffer,
//                            IN unsigned long PreferedMaximumLength,
//                            OUT unsigned long *CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     PSAM_DB_OBJECT DomainObject;
//     HANDLE UsersKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     WCHAR UserName[64];
//     ULONG EnumIndex;
//     ULONG EnumCount = 0;
//     ULONG RequiredLength = 0;
//     ULONG NameLength;
//     ULONG DataLength;
//     ULONG Rid;
//     ULONG i;
//     BOOLEAN MoreEntries = FALSE;
//     NTSTATUS Status;
//
//     TRACE("SamrEnumerateUsersInDomain(%p %p %lx %p %lu %p)\n",
//           DomainHandle, EnumerationContext, UserAccountControl, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LIST_ACCOUNTS,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Users",
//                             KEY_READ,
//                             &UsersKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(UsersKeyHandle,
//                             L"Names",
//                             KEY_READ,
//                             &NamesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     TRACE("Part 1\n");
//
//     EnumIndex = *EnumerationContext;
//
//     while (TRUE)
//     {
//         NameLength = 64 * sizeof(WCHAR);
//         Status = SampRegEnumerateValue(NamesKeyHandle,
//                                        EnumIndex,
//                                        UserName,
//                                        &NameLength,
//                                        NULL,
//                                        NULL,
//                                        NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("User name: %S\n", UserName);
//         TRACE("Name length: %lu\n", NameLength);
//
//         if ((RequiredLength + NameLength + sizeof(UNICODE_NULL) + sizeof(SAMPR_RID_ENUMERATION)) > PreferedMaximumLength)
//         {
//             MoreEntries = TRUE;
//             break;
//         }
//
//         RequiredLength += (NameLength + sizeof(UNICODE_NULL) + sizeof(SAMPR_RID_ENUMERATION));
//         EnumCount++;
//
//         EnumIndex++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//     TRACE("RequiredLength: %lu\n", RequiredLength);
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     EnumBuffer = midl_user_allocate(sizeof(SAMPR_ENUMERATION_BUFFER));
//     if (EnumBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumBuffer->EntriesRead = EnumCount;
//     if (EnumCount == 0)
//     {
//         Status = STATUS_NO_MORE_ENTRIES;
//         goto done;
//     }
//
//     EnumBuffer->Buffer = midl_user_allocate(EnumCount * sizeof(SAMPR_RID_ENUMERATION));
//     if (EnumBuffer->Buffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     TRACE("Part 2\n");
//
//     EnumIndex = *EnumerationContext;
//     for (i = 0; i < EnumCount; i++, EnumIndex++)
//     {
//         NameLength = 64 * sizeof(WCHAR);
//         DataLength = sizeof(ULONG);
//         Status = SampRegEnumerateValue(NamesKeyHandle,
//                                        EnumIndex,
//                                        UserName,
//                                        &NameLength,
//                                        NULL,
//                                        &Rid,
//                                        &DataLength);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("User name: %S\n", UserName);
//         TRACE("Name length: %lu\n", NameLength);
//         TRACE("RID: %lu\n", Rid);
//
//         EnumBuffer->Buffer[i].RelativeId = Rid;
//
//         EnumBuffer->Buffer[i].Name.Length = (USHORT)NameLength;
//         EnumBuffer->Buffer[i].Name.MaximumLength = (USHORT)(NameLength + sizeof(UNICODE_NULL));
//
// /* FIXME: Disabled because of bugs in widl and rpcrt4 */
//     }
//
// done:
//     if (NT_SUCCESS(Status))
//     {
//         *EnumerationContext += EnumCount;
//         *Buffer = EnumBuffer;
//         *CountReturned = EnumCount;
//     }
//     else
//     {
//         *EnumerationContext = 0;
//         *Buffer = NULL;
//         *CountReturned = 0;
//
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 if (EnumBuffer->EntriesRead != 0)
//                 {
//                     for (i = 0; i < EnumBuffer->EntriesRead; i++)
//                     {
//                         if (EnumBuffer->Buffer[i].Name.Buffer != NULL)
//                             midl_user_free(EnumBuffer->Buffer[i].Name.Buffer);
//                     }
//                 }
//
//                 midl_user_free(EnumBuffer->Buffer);
//             }
//
//             midl_user_free(EnumBuffer);
//         }
//     }
//
//     SampRegCloseKey(&NamesKeyHandle);
//     SampRegCloseKey(&UsersKeyHandle);
//
//     if ((Status == STATUS_SUCCESS) && (MoreEntries != FALSE))
//         Status = STATUS_MORE_ENTRIES;
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 14 */
// NTSTATUS
// NTAPI
// SamrCreateAliasInDomain(IN SAMPR_HANDLE DomainHandle,
//                         IN PRPC_UNICODE_STRING AccountName,
//                         IN ACCESS_MASK DesiredAccess,
//                         OUT SAMPR_HANDLE *AliasHandle,
//                         OUT unsigned long *RelativeId)
// {
//     SAM_DOMAIN_FIXED_DATA FixedDomainData;
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT AliasObject;
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     ULONG ulSize;
//     ULONG ulRid;
//     WCHAR szRid[9];
//     NTSTATUS Status;
//
//     TRACE("SamrCreateAliasInDomain(%p %p %lx %p %p)\n",
//           DomainHandle, AccountName, DesiredAccess, AliasHandle, RelativeId);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &AliasMapping);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_CREATE_ALIAS,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Check the alias account name */
//     Status = SampCheckAccountName(AccountName, 256);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check if the alias name already exists in the domain */
//     Status = SampCheckAccountNameInDomain(DomainObject,
//                                           AccountName->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Alias name \'%S\' already exists in domain (Status 0x%08lx)\n",
//               AccountName->Buffer, Status);
//         goto done;
//     }
//
//     /* Create the security descriptor */
//     Status = SampCreateAliasSD(&Sd,
//                                &SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCreateAliasSD failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Get the fixed domain attributes */
//     ulSize = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedDomainData,
//                                     &ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Increment the NextRid attribute */
//     ulRid = FixedDomainData.NextRid;
//     FixedDomainData.NextRid++;
//
//     /* Store the fixed domain attributes */
//     Status = SampSetObjectAttribute(DomainObject,
//                            L"F",
//                            REG_BINARY,
//                            &FixedDomainData,
//                            ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     TRACE("RID: %lx\n", ulRid);
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", ulRid);
//
//     /* Create the alias object */
//     Status = SampCreateDbObject(DomainObject,
//                                 L"Aliases",
//                                 szRid,
//                                 ulRid,
//                                 SamDbAliasObject,
//                                 DesiredAccess,
//                                 &AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Add the account name for the alias object */
//     Status = SampSetAccountNameInDomain(DomainObject,
//                                         L"Aliases",
//                                         AccountName->Buffer,
//                                         ulRid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Name attribute */
//     Status = SampSetObjectAttributeString(AliasObject,
//                                           L"Name",
//                                           AccountName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Description attribute */
//     Status = SampSetObjectAttributeString(AliasObject,
//                                           L"Description",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the SecDesc attribute*/
//     Status = SampSetObjectAttribute(AliasObject,
//                                     L"SecDesc",
//                                     REG_BINARY,
//                                     Sd,
//                                     SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *AliasHandle = (SAMPR_HANDLE)AliasObject;
//         *RelativeId = ulRid;
//     }
//
// done:
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("returns with status 0x%08lx\n", Status);
//
//     return Status;
// }
//
//
// /* Function 15 */
// NTSTATUS
// NTAPI
// SamrEnumerateAliasesInDomain(IN SAMPR_HANDLE DomainHandle,
//                              IN OUT unsigned long *EnumerationContext,
//                              OUT PSAMPR_ENUMERATION_BUFFER *Buffer,
//                              IN unsigned long PreferedMaximumLength,
//                              OUT unsigned long *CountReturned)
// {
//     PSAMPR_ENUMERATION_BUFFER EnumBuffer = NULL;
//     PSAM_DB_OBJECT DomainObject;
//     HANDLE AliasesKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     WCHAR AliasName[64];
//     ULONG EnumIndex;
//     ULONG EnumCount = 0;
//     ULONG RequiredLength = 0;
//     ULONG NameLength;
//     ULONG DataLength;
//     ULONG Rid;
//     ULONG i;
//     BOOLEAN MoreEntries = FALSE;
//     NTSTATUS Status;
//
//     TRACE("SamrEnumerateAliasesInDomain(%p %p %p %lu %p)\n",
//           DomainHandle, EnumerationContext, Buffer,
//           PreferedMaximumLength, CountReturned);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LIST_ACCOUNTS,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Aliases",
//                             KEY_READ,
//                             &AliasesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(AliasesKeyHandle,
//                             L"Names",
//                             KEY_READ,
//                             &NamesKeyHandle);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     TRACE("Part 1\n");
//
//     EnumIndex = *EnumerationContext;
//
//     while (TRUE)
//     {
//         NameLength = 64 * sizeof(WCHAR);
//         Status = SampRegEnumerateValue(NamesKeyHandle,
//                                        EnumIndex,
//                                        AliasName,
//                                        &NameLength,
//                                        NULL,
//                                        NULL,
//                                        NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Alias name: %S\n", AliasName);
//         TRACE("Name length: %lu\n", NameLength);
//
//         if ((RequiredLength + NameLength + sizeof(UNICODE_NULL) + sizeof(SAMPR_RID_ENUMERATION)) > PreferedMaximumLength)
//         {
//             MoreEntries = TRUE;
//             break;
//         }
//
//         RequiredLength += (NameLength + sizeof(UNICODE_NULL) + sizeof(SAMPR_RID_ENUMERATION));
//         EnumCount++;
//
//         EnumIndex++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//     TRACE("RequiredLength: %lu\n", RequiredLength);
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     EnumBuffer = midl_user_allocate(sizeof(SAMPR_ENUMERATION_BUFFER));
//     if (EnumBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumBuffer->EntriesRead = EnumCount;
//     if (EnumCount == 0)
//     {
//         Status = STATUS_NO_MORE_ENTRIES;
//         goto done;
//     }
//
//     EnumBuffer->Buffer = midl_user_allocate(EnumCount * sizeof(SAMPR_RID_ENUMERATION));
//     if (EnumBuffer->Buffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     TRACE("Part 2\n");
//
//     EnumIndex = *EnumerationContext;
//     for (i = 0; i < EnumCount; i++, EnumIndex++)
//     {
//         NameLength = 64 * sizeof(WCHAR);
//         DataLength = sizeof(ULONG);
//         Status = SampRegEnumerateValue(NamesKeyHandle,
//                                        EnumIndex,
//                                        AliasName,
//                                        &NameLength,
//                                        NULL,
//                                        &Rid,
//                                        &DataLength);
//         if (!NT_SUCCESS(Status))
//         {
//             if (Status == STATUS_NO_MORE_ENTRIES)
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Alias name: %S\n", AliasName);
//         TRACE("Name length: %lu\n", NameLength);
//         TRACE("RID: %lu\n", Rid);
//
//         EnumBuffer->Buffer[i].RelativeId = Rid;
//
//         EnumBuffer->Buffer[i].Name.Length = (USHORT)NameLength;
//         EnumBuffer->Buffer[i].Name.MaximumLength = (USHORT)(NameLength + sizeof(UNICODE_NULL));
//
// /* FIXME: Disabled because of bugs in widl and rpcrt4 */
//     }
//
// done:
//     if (NT_SUCCESS(Status))
//     {
//         *EnumerationContext += EnumCount;
//         *Buffer = EnumBuffer;
//         *CountReturned = EnumCount;
//     }
//     else
//     {
//         *EnumerationContext = 0;
//         *Buffer = NULL;
//         *CountReturned = 0;
//
//         if (EnumBuffer != NULL)
//         {
//             if (EnumBuffer->Buffer != NULL)
//             {
//                 if (EnumBuffer->EntriesRead != 0)
//                 {
//                     for (i = 0; i < EnumBuffer->EntriesRead; i++)
//                     {
//                         if (EnumBuffer->Buffer[i].Name.Buffer != NULL)
//                             midl_user_free(EnumBuffer->Buffer[i].Name.Buffer);
//                     }
//                 }
//
//                 midl_user_free(EnumBuffer->Buffer);
//             }
//
//             midl_user_free(EnumBuffer);
//         }
//     }
//
//     SampRegCloseKey(&NamesKeyHandle);
//     SampRegCloseKey(&AliasesKeyHandle);
//
//     if ((Status == STATUS_SUCCESS) && (MoreEntries != FALSE))
//         Status = STATUS_MORE_ENTRIES;
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 16 */
// NTSTATUS
// NTAPI
// SamrGetAliasMembership(IN SAMPR_HANDLE DomainHandle,
//                        IN PSAMPR_PSID_ARRAY SidArray,
//                        OUT PSAMPR_ULONG_ARRAY Membership)
// {
//     PSAM_DB_OBJECT DomainObject;
//     HANDLE AliasesKeyHandle = NULL;
//     HANDLE MembersKeyHandle = NULL;
//     HANDLE MemberKeyHandle = NULL;
//     LPWSTR MemberSidString = NULL;
//     PULONG RidArray = NULL;
//     ULONG MaxSidCount = 0;
//     ULONG ValueCount;
//     ULONG DataLength;
//     ULONG i, j;
//     ULONG RidIndex;
//     NTSTATUS Status;
//     WCHAR NameBuffer[9];
//
//     TRACE("SamrGetAliasMembership(%p %p %p)\n",
//           DomainHandle, SidArray, Membership);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_GET_ALIAS_MEMBERSHIP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(DomainObject->KeyHandle,
//                             L"Aliases",
//                             KEY_READ,
//                             &AliasesKeyHandle);
//     TRACE("SampRegOpenKey returned %08lX\n", Status);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampRegOpenKey(AliasesKeyHandle,
//                             L"Members",
//                             KEY_READ,
//                             &MembersKeyHandle);
//     TRACE("SampRegOpenKey returned %08lX\n", Status);
//
//     if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     for (i = 0; i < SidArray->Count; i++)
//     {
//         ConvertSidToStringSid(SidArray->Sids[i].SidPointer, &MemberSidString);
// TRACE("Open %S\n", MemberSidString);
//
//         Status = SampRegOpenKey(MembersKeyHandle,
//                                 MemberSidString,
//                                 KEY_READ,
//                                 &MemberKeyHandle);
//         TRACE("SampRegOpenKey returned %08lX\n", Status);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegQueryKeyInfo(MemberKeyHandle,
//                                          NULL,
//                                          &ValueCount);
//             if (NT_SUCCESS(Status))
//             {
//                 TRACE("Found %lu values\n", ValueCount);
//                 MaxSidCount += ValueCount;
//             }
//
//             SampRegCloseKey(&MemberKeyHandle);
//         }
//
//         if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//             Status = STATUS_SUCCESS;
//
//         LocalFree(MemberSidString);
//     }
//
//     if (MaxSidCount == 0)
//     {
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     TRACE("Maximum sid count: %lu\n", MaxSidCount);
//     RidArray = midl_user_allocate(MaxSidCount * sizeof(ULONG));
//     if (RidArray == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     RidIndex = 0;
//     for (i = 0; i < SidArray->Count; i++)
//     {
//         ConvertSidToStringSid(SidArray->Sids[i].SidPointer, &MemberSidString);
// TRACE("Open %S\n", MemberSidString);
//
//         Status = SampRegOpenKey(MembersKeyHandle,
//                                 MemberSidString,
//                                 KEY_READ,
//                                 &MemberKeyHandle);
//         TRACE("SampRegOpenKey returned %08lX\n", Status);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegQueryKeyInfo(MemberKeyHandle,
//                                          NULL,
//                                          &ValueCount);
//             if (NT_SUCCESS(Status))
//             {
//                 TRACE("Found %lu values\n", ValueCount);
//
//                 for (j = 0; j < ValueCount; j++)
//                 {
//                     DataLength = 9 * sizeof(WCHAR);
//                     Status = SampRegEnumerateValue(MemberKeyHandle,
//                                                    j,
//                                                    NameBuffer,
//                                                    &DataLength,
//                                                    NULL,
//                                                    NULL,
//                                                    NULL);
//                     if (NT_SUCCESS(Status))
//                     {
//                         /* FIXME: Do not return each RID more than once. */
//                         RidArray[RidIndex] = wcstoul(NameBuffer, NULL, 16);
//                         RidIndex++;
//                     }
//                 }
//             }
//
//             SampRegCloseKey(&MemberKeyHandle);
//         }
//
//         if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//             Status = STATUS_SUCCESS;
//
//         LocalFree(MemberSidString);
//     }
//
// done:
//     SampRegCloseKey(&MembersKeyHandle);
//     SampRegCloseKey(&AliasesKeyHandle);
//
//     if (NT_SUCCESS(Status))
//     {
//         Membership->Count = MaxSidCount;
//         Membership->Element = RidArray;
//     }
//     else
//     {
//         if (RidArray != NULL)
//             midl_user_free(RidArray);
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 17 */
// NTSTATUS
// NTAPI
// SamrLookupNamesInDomain(IN SAMPR_HANDLE DomainHandle,
//                         IN ULONG Count,
//                         IN RPC_UNICODE_STRING Names[],
//                         OUT PSAMPR_ULONG_ARRAY RelativeIds,
//                         OUT PSAMPR_ULONG_ARRAY Use)
// {
//     PSAM_DB_OBJECT DomainObject;
//     HANDLE AccountsKeyHandle = NULL;
//     HANDLE NamesKeyHandle = NULL;
//     ULONG MappedCount = 0;
//     ULONG DataLength;
//     ULONG i;
//     ULONG RelativeId;
//     NTSTATUS Status;
//
//     TRACE("SamrLookupNamesInDomain(%p %lu %p %p %p)\n",
//           DomainHandle, Count, Names, RelativeIds, Use);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LOOKUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     RelativeIds->Count = 0;
//     Use->Count = 0;
//
//     if (Count == 0)
//     {
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     /* Allocate the relative IDs array */
//     RelativeIds->Element = midl_user_allocate(Count * sizeof(ULONG));
//     if (RelativeIds->Element == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Allocate the use array */
//     Use->Element = midl_user_allocate(Count * sizeof(ULONG));
//     if (Use->Element == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     RelativeIds->Count = Count;
//     Use->Count = Count;
//
//     for (i = 0; i < Count; i++)
//     {
//         TRACE("Name: %S\n", Names[i].Buffer);
//
//         RelativeId = 0;
//
//         /* Lookup aliases */
//         Status = SampRegOpenKey(DomainObject->KeyHandle,
//                                 L"Aliases",
//                                 KEY_READ,
//                                 &AccountsKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegOpenKey(AccountsKeyHandle,
//                                     L"Names",
//                                     KEY_READ,
//                                     &NamesKeyHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = sizeof(ULONG);
//                 Status = SampRegQueryValue(NamesKeyHandle,
//                                            Names[i].Buffer,
//                                            NULL,
//                                            &RelativeId,
//                                            &DataLength);
//
//                 SampRegCloseKey(&NamesKeyHandle);
//             }
//
//             SampRegCloseKey(&AccountsKeyHandle);
//         }
//
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             break;
//
//         /* Return alias account */
//         if (NT_SUCCESS(Status) && RelativeId != 0)
//         {
//             TRACE("Rid: %lu\n", RelativeId);
//             RelativeIds->Element[i] = RelativeId;
//             Use->Element[i] = SidTypeAlias;
//             MappedCount++;
//             continue;
//         }
//
//         /* Lookup groups */
//         Status = SampRegOpenKey(DomainObject->KeyHandle,
//                                 L"Groups",
//                                 KEY_READ,
//                                 &AccountsKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegOpenKey(AccountsKeyHandle,
//                                     L"Names",
//                                     KEY_READ,
//                                     &NamesKeyHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = sizeof(ULONG);
//                 Status = SampRegQueryValue(NamesKeyHandle,
//                                            Names[i].Buffer,
//                                            NULL,
//                                            &RelativeId,
//                                            &DataLength);
//
//                 SampRegCloseKey(&NamesKeyHandle);
//             }
//
//             SampRegCloseKey(&AccountsKeyHandle);
//         }
//
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             break;
//
//         /* Return group account */
//         if (NT_SUCCESS(Status) && RelativeId != 0)
//         {
//             TRACE("Rid: %lu\n", RelativeId);
//             RelativeIds->Element[i] = RelativeId;
//             Use->Element[i] = SidTypeGroup;
//             MappedCount++;
//             continue;
//         }
//
//         /* Lookup users */
//         Status = SampRegOpenKey(DomainObject->KeyHandle,
//                                 L"Users",
//                                 KEY_READ,
//                                 &AccountsKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegOpenKey(AccountsKeyHandle,
//                                     L"Names",
//                                     KEY_READ,
//                                     &NamesKeyHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = sizeof(ULONG);
//                 Status = SampRegQueryValue(NamesKeyHandle,
//                                            Names[i].Buffer,
//                                            NULL,
//                                            &RelativeId,
//                                            &DataLength);
//
//                 SampRegCloseKey(&NamesKeyHandle);
//             }
//
//             SampRegCloseKey(&AccountsKeyHandle);
//         }
//
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             break;
//
//         /* Return user account */
//         if (NT_SUCCESS(Status) && RelativeId != 0)
//         {
//             TRACE("Rid: %lu\n", RelativeId);
//             RelativeIds->Element[i] = RelativeId;
//             Use->Element[i] = SidTypeUser;
//             MappedCount++;
//             continue;
//         }
//
//         /* Return unknown account */
//         RelativeIds->Element[i] = 0;
//         Use->Element[i] = SidTypeUnknown;
//     }
//
// done:
//     if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//         Status = STATUS_SUCCESS;
//
//     if (NT_SUCCESS(Status))
//     {
//         if (MappedCount == 0)
//             Status = STATUS_NONE_MAPPED;
//         else if (MappedCount < Count)
//             Status = STATUS_SOME_NOT_MAPPED;
//     }
//     else
//     {
//         if (RelativeIds->Element != NULL)
//         {
//             midl_user_free(RelativeIds->Element);
//             RelativeIds->Element = NULL;
//         }
//
//         RelativeIds->Count = 0;
//
//         if (Use->Element != NULL)
//         {
//             midl_user_free(Use->Element);
//             Use->Element = NULL;
//         }
//
//         Use->Count = 0;
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("Returned Status %lx\n", Status);
//
//     return Status;
// }
//
//
// /* Function 18 */
// NTSTATUS
// NTAPI
// SamrLookupIdsInDomain(IN SAMPR_HANDLE DomainHandle,
//                       IN ULONG Count,
//                       IN ULONG *RelativeIds,
//                       OUT PSAMPR_RETURNED_USTRING_ARRAY Names,
//                       OUT PSAMPR_ULONG_ARRAY Use)
// {
//     PSAM_DB_OBJECT DomainObject;
//     WCHAR RidString[9];
//     HANDLE AccountsKeyHandle = NULL;
//     HANDLE AccountKeyHandle = NULL;
//     ULONG MappedCount = 0;
//     ULONG DataLength;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("SamrLookupIdsInDomain(%p %lu %p %p %p)\n",
//           DomainHandle, Count, RelativeIds, Names, Use);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LOOKUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Names->Count = 0;
//     Use->Count = 0;
//
//     if (Count == 0)
//     {
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     /* Allocate the names array */
//     Names->Element = midl_user_allocate(Count * sizeof(*Names->Element));
//     if (Names->Element == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Allocate the use array */
//     Use->Element = midl_user_allocate(Count * sizeof(*Use->Element));
//     if (Use->Element == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Names->Count = Count;
//     Use->Count = Count;
//
//     for (i = 0; i < Count; i++)
//     {
//         TRACE("RID: %lu\n", RelativeIds[i]);
//
//         swprintf(RidString, L"%08lx", RelativeIds[i]);
//
//         /* Lookup aliases */
//         Status = SampRegOpenKey(DomainObject->KeyHandle,
//                                 L"Aliases",
//                                 KEY_READ,
//                                 &AccountsKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegOpenKey(AccountsKeyHandle,
//                                     RidString,
//                                     KEY_READ,
//                                     &AccountKeyHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = 0;
//                 Status = SampRegQueryValue(AccountKeyHandle,
//                                            L"Name",
//                                            NULL,
//                                            NULL,
//                                            &DataLength);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Names->Element[i].Buffer = midl_user_allocate(DataLength);
//                     if (Names->Element[i].Buffer == NULL)
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//
//                     if (NT_SUCCESS(Status))
//                     {
//                         Names->Element[i].MaximumLength = (USHORT)DataLength;
//                         Names->Element[i].Length = (USHORT)(DataLength - sizeof(WCHAR));
//
//                         Status = SampRegQueryValue(AccountKeyHandle,
//                                                    L"Name",
//                                                    NULL,
//                                                    Names->Element[i].Buffer,
//                                                    &DataLength);
//                     }
//                 }
//
//                 SampRegCloseKey(&AccountKeyHandle);
//             }
//
//             SampRegCloseKey(&AccountsKeyHandle);
//         }
//
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             break;
//
//         /* Return alias account */
//         if (NT_SUCCESS(Status) && Names->Element[i].Buffer != NULL)
//         {
//             TRACE("Name: %S\n", Names->Element[i].Buffer);
//             Use->Element[i] = SidTypeAlias;
//             MappedCount++;
//             continue;
//         }
//
//         /* Lookup groups */
//         Status = SampRegOpenKey(DomainObject->KeyHandle,
//                                 L"Groups",
//                                 KEY_READ,
//                                 &AccountsKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegOpenKey(AccountsKeyHandle,
//                                     RidString,
//                                     KEY_READ,
//                                     &AccountKeyHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = 0;
//                 Status = SampRegQueryValue(AccountKeyHandle,
//                                            L"Name",
//                                            NULL,
//                                            NULL,
//                                            &DataLength);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Names->Element[i].Buffer = midl_user_allocate(DataLength);
//                     if (Names->Element[i].Buffer == NULL)
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//
//                     if (NT_SUCCESS(Status))
//                     {
//                         Names->Element[i].MaximumLength = (USHORT)DataLength;
//                         Names->Element[i].Length = (USHORT)(DataLength - sizeof(WCHAR));
//
//                         Status = SampRegQueryValue(AccountKeyHandle,
//                                                    L"Name",
//                                                    NULL,
//                                                    Names->Element[i].Buffer,
//                                                    &DataLength);
//                     }
//                 }
//
//                 SampRegCloseKey(&AccountKeyHandle);
//             }
//
//             SampRegCloseKey(&AccountsKeyHandle);
//         }
//
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             break;
//
//         /* Return group account */
//         if (NT_SUCCESS(Status) && Names->Element[i].Buffer != NULL)
//         {
//             TRACE("Name: %S\n", Names->Element[i].Buffer);
//             Use->Element[i] = SidTypeGroup;
//             MappedCount++;
//             continue;
//         }
//
//         /* Lookup users */
//         Status = SampRegOpenKey(DomainObject->KeyHandle,
//                                 L"Users",
//                                 KEY_READ,
//                                 &AccountsKeyHandle);
//         if (NT_SUCCESS(Status))
//         {
//             Status = SampRegOpenKey(AccountsKeyHandle,
//                                     RidString,
//                                     KEY_READ,
//                                     &AccountKeyHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DataLength = 0;
//                 Status = SampRegQueryValue(AccountKeyHandle,
//                                            L"Name",
//                                            NULL,
//                                            NULL,
//                                            &DataLength);
//                 if (NT_SUCCESS(Status))
//                 {
//                     TRACE("DataLength: %lu\n", DataLength);
//
//                     Names->Element[i].Buffer = midl_user_allocate(DataLength);
//                     if (Names->Element[i].Buffer == NULL)
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//
//                     if (NT_SUCCESS(Status))
//                     {
//                         Names->Element[i].MaximumLength = (USHORT)DataLength;
//                         Names->Element[i].Length = (USHORT)(DataLength - sizeof(WCHAR));
//
//                         Status = SampRegQueryValue(AccountKeyHandle,
//                                                    L"Name",
//                                                    NULL,
//                                                    Names->Element[i].Buffer,
//                                                    &DataLength);
//                     }
//                 }
//
//                 SampRegCloseKey(&AccountKeyHandle);
//             }
//
//             SampRegCloseKey(&AccountsKeyHandle);
//         }
//
//         if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//             break;
//
//         /* Return user account */
//         if (NT_SUCCESS(Status) && Names->Element[i].Buffer != NULL)
//         {
//             TRACE("Name: %S\n", Names->Element[i].Buffer);
//             Use->Element[i] = SidTypeUser;
//             MappedCount++;
//             continue;
//         }
//
//         /* Return unknown account */
//         Use->Element[i] = SidTypeUnknown;
//     }
//
// done:
//     if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//         Status = STATUS_SUCCESS;
//
//     if (NT_SUCCESS(Status))
//     {
//         if (MappedCount == 0)
//             Status = STATUS_NONE_MAPPED;
//         else if (MappedCount < Count)
//             Status = STATUS_SOME_NOT_MAPPED;
//     }
//     else
//     {
//         if (Names->Element != NULL)
//         {
//             for (i = 0; i < Count; i++)
//             {
//                 if (Names->Element[i].Buffer != NULL)
//                     midl_user_free(Names->Element[i].Buffer);
//             }
//
//             midl_user_free(Names->Element);
//             Names->Element = NULL;
//         }
//
//         Names->Count = 0;
//
//         if (Use->Element != NULL)
//         {
//             midl_user_free(Use->Element);
//             Use->Element = NULL;
//         }
//
//         Use->Count = 0;
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 19 */
// NTSTATUS
// NTAPI
// SamrOpenGroup(IN SAMPR_HANDLE DomainHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN unsigned long GroupId,
//               OUT SAMPR_HANDLE *GroupHandle)
// {
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT GroupObject;
//     WCHAR szRid[9];
//     NTSTATUS Status;
//
//     TRACE("SamrOpenGroup(%p %lx %lx %p)\n",
//           DomainHandle, DesiredAccess, GroupId, GroupHandle);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &GroupMapping);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LOOKUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", GroupId);
//
//     /* Create the group object */
//     Status = SampOpenDbObject(DomainObject,
//                               L"Groups",
//                               szRid,
//                               GroupId,
//                               SamDbGroupObject,
//                               DesiredAccess,
//                               &GroupObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *GroupHandle = (SAMPR_HANDLE)GroupObject;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryGroupGeneral(PSAM_DB_OBJECT GroupObject,
//                       PSAMPR_GROUP_INFO_BUFFER *Buffer)
// {
//     PSAMPR_GROUP_INFO_BUFFER InfoBuffer = NULL;
//     SAM_GROUP_FIXED_DATA FixedData;
//     ULONG MembersLength = 0;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_GROUP_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = SampGetObjectAttributeString(GroupObject,
//                                           L"Name",
//                                           &InfoBuffer->General.Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampGetObjectAttributeString(GroupObject,
//                                           L"AdminComment",
//                                           &InfoBuffer->General.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Length = sizeof(SAM_GROUP_FIXED_DATA);
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     InfoBuffer->General.Attributes = FixedData.Attributes;
//
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"Members",
//                                     NULL,
//                                     NULL,
//                                     &MembersLength);
//     if (!NT_SUCCESS(Status) && Status != STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         InfoBuffer->General.MemberCount = 0;
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         InfoBuffer->General.MemberCount = MembersLength / sizeof(ULONG);
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->General.Name.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.Name.Buffer);
//
//             if (InfoBuffer->General.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.AdminComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryGroupName(PSAM_DB_OBJECT GroupObject,
//                    PSAMPR_GROUP_INFO_BUFFER *Buffer)
// {
//     PSAMPR_GROUP_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_GROUP_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = SampGetObjectAttributeString(GroupObject,
//                                           L"Name",
//                                           &InfoBuffer->Name.Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Name.Name.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Name.Name.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryGroupAttribute(PSAM_DB_OBJECT GroupObject,
//                         PSAMPR_GROUP_INFO_BUFFER *Buffer)
// {
//     PSAMPR_GROUP_INFO_BUFFER InfoBuffer = NULL;
//     SAM_GROUP_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_GROUP_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_GROUP_FIXED_DATA);
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     InfoBuffer->Attribute.Attributes = FixedData.Attributes;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryGroupAdminComment(PSAM_DB_OBJECT GroupObject,
//                            PSAMPR_GROUP_INFO_BUFFER *Buffer)
// {
//     PSAMPR_GROUP_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_GROUP_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = SampGetObjectAttributeString(GroupObject,
//                                           L"AdminComment",
//                                           &InfoBuffer->AdminComment.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->AdminComment.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->AdminComment.AdminComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// /* Function 20 */
// NTSTATUS
// NTAPI
// SamrQueryInformationGroup(IN SAMPR_HANDLE GroupHandle,
//                           IN GROUP_INFORMATION_CLASS GroupInformationClass,
//                           OUT PSAMPR_GROUP_INFO_BUFFER *Buffer)
// {
//     PSAM_DB_OBJECT GroupObject;
//     NTSTATUS Status;
//
//     TRACE("SamrQueryInformationGroup(%p %lu %p)\n",
//           GroupHandle, GroupInformationClass, Buffer);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(GroupHandle,
//                                   SamDbGroupObject,
//                                   GROUP_READ_INFORMATION,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     switch (GroupInformationClass)
//     {
//         case GroupGeneralInformation:
//             Status = SampQueryGroupGeneral(GroupObject,
//                                            Buffer);
//             break;
//
//         case GroupNameInformation:
//             Status = SampQueryGroupName(GroupObject,
//                                         Buffer);
//             break;
//
//         case GroupAttributeInformation:
//             Status = SampQueryGroupAttribute(GroupObject,
//                                              Buffer);
//             break;
//
//         case GroupAdminCommentInformation:
//             Status = SampQueryGroupAdminComment(GroupObject,
//                                                 Buffer);
//             break;
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetGroupName(PSAM_DB_OBJECT GroupObject,
//                  PSAMPR_GROUP_INFO_BUFFER Buffer)
// {
//     UNICODE_STRING OldGroupName = {0, 0, NULL};
//     UNICODE_STRING NewGroupName;
//     NTSTATUS Status;
//
//     Status = SampGetObjectAttributeString(GroupObject,
//                                           L"Name",
//                                           (PRPC_UNICODE_STRING)&OldGroupName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttributeString failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check the new account name */
//     Status = SampCheckAccountName(&Buffer->Name.Name, 256);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     NewGroupName.Length = Buffer->Name.Name.Length;
//     NewGroupName.MaximumLength = Buffer->Name.Name.MaximumLength;
//     NewGroupName.Buffer = Buffer->Name.Name.Buffer;
//
//     if (!RtlEqualUnicodeString(&OldGroupName, &NewGroupName, TRUE))
//     {
//         Status = SampCheckAccountNameInDomain(GroupObject->ParentObject,
//                                               NewGroupName.Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Group name \'%S\' already exists in domain (Status 0x%08lx)\n",
//                   NewGroupName.Buffer, Status);
//             goto done;
//         }
//     }
//
//     Status = SampSetAccountNameInDomain(GroupObject->ParentObject,
//                                         L"Groups",
//                                         NewGroupName.Buffer,
//                                         GroupObject->RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetAccountNameInDomain failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampRemoveAccountNameFromDomain(GroupObject->ParentObject,
//                                              L"Groups",
//                                              OldGroupName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveAccountNameFromDomain failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampSetObjectAttributeString(GroupObject,
//                                           L"Name",
//                                           (PRPC_UNICODE_STRING)&NewGroupName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (OldGroupName.Buffer != NULL)
//         midl_user_free(OldGroupName.Buffer);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetGroupAttribute(PSAM_DB_OBJECT GroupObject,
//                       PSAMPR_GROUP_INFO_BUFFER Buffer)
// {
//     SAM_GROUP_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_GROUP_FIXED_DATA);
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.Attributes = Buffer->Attribute.Attributes;
//
//     Status = SampSetObjectAttribute(GroupObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// /* Function 21 */
// NTSTATUS
// NTAPI
// SamrSetInformationGroup(IN SAMPR_HANDLE GroupHandle,
//                         IN GROUP_INFORMATION_CLASS GroupInformationClass,
//                         IN PSAMPR_GROUP_INFO_BUFFER Buffer)
// {
//     PSAM_DB_OBJECT GroupObject;
//     NTSTATUS Status;
//
//     TRACE("SamrSetInformationGroup(%p %lu %p)\n",
//           GroupHandle, GroupInformationClass, Buffer);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(GroupHandle,
//                                   SamDbGroupObject,
//                                   GROUP_WRITE_ACCOUNT,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     switch (GroupInformationClass)
//     {
//         case GroupNameInformation:
//             Status = SampSetGroupName(GroupObject,
//                                       Buffer);
//             break;
//
//         case GroupAttributeInformation:
//             Status = SampSetGroupAttribute(GroupObject,
//                                            Buffer);
//             break;
//
//         case GroupAdminCommentInformation:
//             Status = SampSetObjectAttributeString(GroupObject,
//                                                   L"AdminComment",
//                                                   &Buffer->AdminComment.AdminComment);
//             break;
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 22 */
// NTSTATUS
// NTAPI
// SamrAddMemberToGroup(IN SAMPR_HANDLE GroupHandle,
//                      IN unsigned long MemberId,
//                      IN unsigned long Attributes)
// {
//     PSAM_DB_OBJECT GroupObject;
//     PSAM_DB_OBJECT UserObject = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamrAddMemberToGroup(%p %lu %lx)\n",
//           GroupHandle, MemberId, Attributes);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(GroupHandle,
//                                   SamDbGroupObject,
//                                   GROUP_ADD_MEMBER,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Open the user object in the same domain */
//     Status = SampOpenUserObject(GroupObject->ParentObject,
//                                 MemberId,
//                                 0,
//                                 &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampOpenUserObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Add group membership to the user object */
//     Status = SampAddGroupMembershipToUser(UserObject,
//                                           GroupObject->RelativeId,
//                                           Attributes);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampAddGroupMembershipToUser() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Add the member to the group object */
//     Status = SampAddMemberToGroup(GroupObject,
//                                   MemberId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampAddMemberToGroup() failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (UserObject)
//         SampCloseDbObject(UserObject);
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 23 */
// NTSTATUS
// NTAPI
// SamrDeleteGroup(IN OUT SAMPR_HANDLE *GroupHandle)
// {
//     PSAM_DB_OBJECT GroupObject;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     TRACE("SamrDeleteGroup(%p)\n", GroupHandle);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(*GroupHandle,
//                                   SamDbGroupObject,
//                                   DELETE,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Fail, if the group is built-in */
//     if (GroupObject->RelativeId < 1000)
//     {
//         TRACE("You can not delete a special account!\n");
//         Status = STATUS_SPECIAL_ACCOUNT;
//         goto done;
//     }
//
//     /* Get the length of the Members attribute */
//     SampGetObjectAttribute(GroupObject,
//                            L"Members",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     /* Fail, if the group has members */
//     if (Length != 0)
//     {
//         TRACE("There are still members in the group!\n");
//         Status = STATUS_MEMBER_IN_GROUP;
//         goto done;
//     }
//
//     /* FIXME: Remove the group from all aliases */
//
//     /* Delete the group from the database */
//     Status = SampDeleteAccountDbObject(GroupObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampDeleteAccountDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Invalidate the handle */
//     *GroupHandle = NULL;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 24 */
// NTSTATUS
// NTAPI
// SamrRemoveMemberFromGroup(IN SAMPR_HANDLE GroupHandle,
//                           IN unsigned long MemberId)
// {
//     PSAM_DB_OBJECT GroupObject;
//     PSAM_DB_OBJECT UserObject = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamrRemoveMemberFromGroup(%p %lu)\n",
//           GroupHandle, MemberId);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(GroupHandle,
//                                   SamDbGroupObject,
//                                   GROUP_REMOVE_MEMBER,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Open the user object in the same domain */
//     Status = SampOpenUserObject(GroupObject->ParentObject,
//                                 MemberId,
//                                 0,
//                                 &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampOpenUserObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Remove group membership from the user object */
//     Status = SampRemoveGroupMembershipFromUser(UserObject,
//                                                GroupObject->RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampAddGroupMembershipToUser() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Remove the member from the group object */
//     Status = SampRemoveMemberFromGroup(GroupObject,
//                                        MemberId);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampRemoveMemberFromGroup() failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (UserObject)
//         SampCloseDbObject(UserObject);
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 25 */
// NTSTATUS
// NTAPI
// SamrGetMembersInGroup(IN SAMPR_HANDLE GroupHandle,
//                       OUT PSAMPR_GET_MEMBERS_BUFFER *Members)
// {
//     PSAMPR_GET_MEMBERS_BUFFER MembersBuffer = NULL;
//     PSAM_DB_OBJECT GroupObject;
//     ULONG Length = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE("SamrGetMembersInGroup(%p %p)\n",
//           GroupHandle, Members);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(GroupHandle,
//                                   SamDbGroupObject,
//                                   GROUP_LIST_MEMBERS,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     MembersBuffer = midl_user_allocate(sizeof(SAMPR_GET_MEMBERS_BUFFER));
//     if (MembersBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     SampGetObjectAttribute(GroupObject,
//                            L"Members",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == 0)
//     {
//         MembersBuffer->MemberCount = 0;
//         MembersBuffer->Members = NULL;
//         MembersBuffer->Attributes = NULL;
//
//         *Members = MembersBuffer;
//
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     MembersBuffer->Members = midl_user_allocate(Length);
//     if (MembersBuffer->Members == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     MembersBuffer->Attributes = midl_user_allocate(Length);
//     if (MembersBuffer->Attributes == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     Status = SampGetObjectAttribute(GroupObject,
//                                     L"Members",
//                                     NULL,
//                                     MembersBuffer->Members,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttributes() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     MembersBuffer->MemberCount = Length / sizeof(ULONG);
//
//     for (i = 0; i < MembersBuffer->MemberCount; i++)
//     {
//         Status = SampGetUserGroupAttributes(GroupObject->ParentObject,
//                                             MembersBuffer->Members[i],
//                                             GroupObject->RelativeId,
//                                             &(MembersBuffer->Attributes[i]));
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SampGetUserGroupAttributes() failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//     }
//
//     *Members = MembersBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (MembersBuffer != NULL)
//         {
//             if (MembersBuffer->Members != NULL)
//                 midl_user_free(MembersBuffer->Members);
//
//             if (MembersBuffer->Attributes != NULL)
//                 midl_user_free(MembersBuffer->Attributes);
//
//             midl_user_free(MembersBuffer);
//         }
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 26 */
// NTSTATUS
// NTAPI
// SamrSetMemberAttributesOfGroup(IN SAMPR_HANDLE GroupHandle,
//                                IN unsigned long MemberId,
//                                IN unsigned long Attributes)
// {
//     PSAM_DB_OBJECT GroupObject;
//     NTSTATUS Status;
//
//     TRACE("SamrSetMemberAttributesOfGroup(%p %lu %lx)\n",
//           GroupHandle, MemberId, Attributes);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the group handle */
//     Status = SampValidateDbObject(GroupHandle,
//                                   SamDbGroupObject,
//                                   GROUP_ADD_MEMBER,
//                                   &GroupObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampSetUserGroupAttributes(GroupObject->ParentObject,
//                                         MemberId,
//                                         GroupObject->RelativeId,
//                                         Attributes);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetUserGroupAttributes failed with status 0x%08lx\n", Status);
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 27 */
// NTSTATUS
// NTAPI
// SamrOpenAlias(IN SAMPR_HANDLE DomainHandle,
//               IN ACCESS_MASK DesiredAccess,
//               IN ULONG AliasId,
//               OUT SAMPR_HANDLE *AliasHandle)
// {
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT AliasObject;
//     WCHAR szRid[9];
//     NTSTATUS Status;
//
//     TRACE("SamrOpenAlias(%p %lx %lx %p)\n",
//           DomainHandle, DesiredAccess, AliasId, AliasHandle);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &AliasMapping);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LOOKUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", AliasId);
//
//     /* Create the alias object */
//     Status = SampOpenDbObject(DomainObject,
//                               L"Aliases",
//                               szRid,
//                               AliasId,
//                               SamDbAliasObject,
//                               DesiredAccess,
//                               &AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *AliasHandle = (SAMPR_HANDLE)AliasObject;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryAliasGeneral(PSAM_DB_OBJECT AliasObject,
//                       PSAMPR_ALIAS_INFO_BUFFER *Buffer)
// {
//     PSAMPR_ALIAS_INFO_BUFFER InfoBuffer = NULL;
//     HANDLE MembersKeyHandle = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_ALIAS_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = SampGetObjectAttributeString(AliasObject,
//                                           L"Name",
//                                           &InfoBuffer->General.Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampGetObjectAttributeString(AliasObject,
//                                           L"Description",
//                                           &InfoBuffer->General.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Open the Members subkey */
//     Status = SampRegOpenKey(AliasObject->KeyHandle,
//                             L"Members",
//                             KEY_READ,
//                             &MembersKeyHandle);
//     if (NT_SUCCESS(Status))
//     {
//         /* Retrieve the number of members of the alias */
//         Status = SampRegQueryKeyInfo(MembersKeyHandle,
//                                      NULL,
//                                      &InfoBuffer->General.MemberCount);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//     else if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         InfoBuffer->General.MemberCount = 0;
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     SampRegCloseKey(&MembersKeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->General.Name.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.Name.Buffer);
//
//             if (InfoBuffer->General.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.AdminComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryAliasName(PSAM_DB_OBJECT AliasObject,
//                    PSAMPR_ALIAS_INFO_BUFFER *Buffer)
// {
//     PSAMPR_ALIAS_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_ALIAS_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = SampGetObjectAttributeString(AliasObject,
//                                           L"Name",
//                                           &InfoBuffer->Name.Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Name.Name.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Name.Name.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryAliasAdminComment(PSAM_DB_OBJECT AliasObject,
//                            PSAMPR_ALIAS_INFO_BUFFER *Buffer)
// {
//     PSAMPR_ALIAS_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_ALIAS_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = SampGetObjectAttributeString(AliasObject,
//                                           L"Description",
//                                           &InfoBuffer->AdminComment.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->AdminComment.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->AdminComment.AdminComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// /* Function 28 */
// NTSTATUS
// NTAPI
// SamrQueryInformationAlias(IN SAMPR_HANDLE AliasHandle,
//                           IN ALIAS_INFORMATION_CLASS AliasInformationClass,
//                           OUT PSAMPR_ALIAS_INFO_BUFFER *Buffer)
// {
//     PSAM_DB_OBJECT AliasObject;
//     NTSTATUS Status;
//
//     TRACE("SamrQueryInformationAlias(%p %lu %p)\n",
//           AliasHandle, AliasInformationClass, Buffer);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the alias handle */
//     Status = SampValidateDbObject(AliasHandle,
//                                   SamDbAliasObject,
//                                   ALIAS_READ_INFORMATION,
//                                   &AliasObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     switch (AliasInformationClass)
//     {
//         case AliasGeneralInformation:
//             Status = SampQueryAliasGeneral(AliasObject,
//                                            Buffer);
//             break;
//
//         case AliasNameInformation:
//             Status = SampQueryAliasName(AliasObject,
//                                         Buffer);
//             break;
//
//         case AliasAdminCommentInformation:
//             Status = SampQueryAliasAdminComment(AliasObject,
//                                                 Buffer);
//             break;
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetAliasName(PSAM_DB_OBJECT AliasObject,
//                  PSAMPR_ALIAS_INFO_BUFFER Buffer)
// {
//     UNICODE_STRING OldAliasName = {0, 0, NULL};
//     UNICODE_STRING NewAliasName;
//     NTSTATUS Status;
//
//     Status = SampGetObjectAttributeString(AliasObject,
//                                           L"Name",
//                                           (PRPC_UNICODE_STRING)&OldAliasName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttributeString failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check the new account name */
//     Status = SampCheckAccountName(&Buffer->Name.Name, 256);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     NewAliasName.Length = Buffer->Name.Name.Length;
//     NewAliasName.MaximumLength = Buffer->Name.Name.MaximumLength;
//     NewAliasName.Buffer = Buffer->Name.Name.Buffer;
//
//     if (!RtlEqualUnicodeString(&OldAliasName, &NewAliasName, TRUE))
//     {
//         Status = SampCheckAccountNameInDomain(AliasObject->ParentObject,
//                                               NewAliasName.Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Alias name \'%S\' already exists in domain (Status 0x%08lx)\n",
//                   NewAliasName.Buffer, Status);
//             goto done;
//         }
//     }
//
//     Status = SampSetAccountNameInDomain(AliasObject->ParentObject,
//                                         L"Aliases",
//                                         NewAliasName.Buffer,
//                                         AliasObject->RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetAccountNameInDomain failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampRemoveAccountNameFromDomain(AliasObject->ParentObject,
//                                              L"Aliases",
//                                              OldAliasName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveAccountNameFromDomain failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampSetObjectAttributeString(AliasObject,
//                                           L"Name",
//                                           (PRPC_UNICODE_STRING)&NewAliasName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (OldAliasName.Buffer != NULL)
//         midl_user_free(OldAliasName.Buffer);
//
//     return Status;
// }
//
//
// /* Function 29 */
// NTSTATUS
// NTAPI
// SamrSetInformationAlias(IN SAMPR_HANDLE AliasHandle,
//                         IN ALIAS_INFORMATION_CLASS AliasInformationClass,
//                         IN PSAMPR_ALIAS_INFO_BUFFER Buffer)
// {
//     PSAM_DB_OBJECT AliasObject;
//     NTSTATUS Status;
//
//     TRACE("SamrSetInformationAlias(%p %lu %p)\n",
//           AliasHandle, AliasInformationClass, Buffer);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the alias handle */
//     Status = SampValidateDbObject(AliasHandle,
//                                   SamDbAliasObject,
//                                   ALIAS_WRITE_ACCOUNT,
//                                   &AliasObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     switch (AliasInformationClass)
//     {
//         case AliasNameInformation:
//             Status = SampSetAliasName(AliasObject,
//                                       Buffer);
//             break;
//
//         case AliasAdminCommentInformation:
//             Status = SampSetObjectAttributeString(AliasObject,
//                                                   L"Description",
//                                                   &Buffer->AdminComment.AdminComment);
//             break;
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//             break;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 30 */
// NTSTATUS
// NTAPI
// SamrDeleteAlias(IN OUT SAMPR_HANDLE *AliasHandle)
// {
//     PSAM_DB_OBJECT AliasObject;
//     NTSTATUS Status;
//
//     TRACE("SamrDeleteAlias(%p)\n", AliasHandle);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the alias handle */
//     Status = SampValidateDbObject(*AliasHandle,
//                                   SamDbAliasObject,
//                                   DELETE,
//                                   &AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Fail, if the alias is built-in */
//     if (AliasObject->RelativeId < 1000)
//     {
//         TRACE("You can not delete a special account!\n");
//         Status = STATUS_SPECIAL_ACCOUNT;
//         goto done;
//     }
//
//     /* Remove all members from the alias */
//     Status = SampRemoveAllMembersFromAlias(AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveAllMembersFromAlias() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Delete the alias from the database */
//     Status = SampDeleteAccountDbObject(AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampDeleteAccountDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Invalidate the handle */
//     *AliasHandle = NULL;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 31 */
// NTSTATUS
// NTAPI
// SamrAddMemberToAlias(IN SAMPR_HANDLE AliasHandle,
//                      IN PRPC_SID MemberId)
// {
//     PSAM_DB_OBJECT AliasObject;
//     NTSTATUS Status;
//
//     TRACE("SamrAddMemberToAlias(%p %p)\n",
//           AliasHandle, MemberId);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the alias handle */
//     Status = SampValidateDbObject(AliasHandle,
//                                   SamDbAliasObject,
//                                   ALIAS_ADD_MEMBER,
//                                   &AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampAddMemberToAlias(AliasObject,
//                                   MemberId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 32 */
// NTSTATUS
// NTAPI
// SamrRemoveMemberFromAlias(IN SAMPR_HANDLE AliasHandle,
//                           IN PRPC_SID MemberId)
// {
//     PSAM_DB_OBJECT AliasObject;
//     NTSTATUS Status;
//
//     TRACE("SamrRemoveMemberFromAlias(%p %p)\n",
//           AliasHandle, MemberId);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the alias handle */
//     Status = SampValidateDbObject(AliasHandle,
//                                   SamDbAliasObject,
//                                   ALIAS_REMOVE_MEMBER,
//                                   &AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampRemoveMemberFromAlias(AliasObject,
//                                        MemberId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 33 */
// NTSTATUS
// NTAPI
// SamrGetMembersInAlias(IN SAMPR_HANDLE AliasHandle,
//                       OUT PSAMPR_PSID_ARRAY_OUT Members)
// {
//     PSAM_DB_OBJECT AliasObject;
//     PSAMPR_SID_INFORMATION MemberArray = NULL;
//     ULONG MemberCount = 0;
//     ULONG Index;
//     NTSTATUS Status;
//
//     TRACE("SamrGetMembersInAlias(%p %p %p)\n",
//           AliasHandle, Members);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the alias handle */
//     Status = SampValidateDbObject(AliasHandle,
//                                   SamDbAliasObject,
//                                   ALIAS_LIST_MEMBERS,
//                                   &AliasObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     Status = SampGetMembersInAlias(AliasObject,
//                                    &MemberCount,
//                                    &MemberArray);
//
//     /* Return the number of members and the member array */
//     if (NT_SUCCESS(Status))
//     {
//         Members->Count = MemberCount;
//         Members->Sids = MemberArray;
//     }
//
// done:
//     /* Clean up the members array and the SID buffers if something failed */
//     if (!NT_SUCCESS(Status))
//     {
//         if (MemberArray != NULL)
//         {
//             for (Index = 0; Index < MemberCount; Index++)
//             {
//                 if (MemberArray[Index].SidPointer != NULL)
//                     midl_user_free(MemberArray[Index].SidPointer);
//             }
//
//             midl_user_free(MemberArray);
//         }
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 34 */
// NTSTATUS
// NTAPI
// SamrOpenUser(IN SAMPR_HANDLE DomainHandle,
//              IN ACCESS_MASK DesiredAccess,
//              IN unsigned long UserId,
//              OUT SAMPR_HANDLE *UserHandle)
// {
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT UserObject;
//     WCHAR szRid[9];
//     NTSTATUS Status;
//
//     TRACE("SamrOpenUser(%p %lx %lx %p)\n",
//           DomainHandle, DesiredAccess, UserId, UserHandle);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &UserMapping);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LOOKUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", UserId);
//
//     /* Create the user object */
//     Status = SampOpenDbObject(DomainObject,
//                               L"Users",
//                               szRid,
//                               UserId,
//                               SamDbUserObject,
//                               DesiredAccess,
//                               &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *UserHandle = (SAMPR_HANDLE)UserObject;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 35 */
// NTSTATUS
// NTAPI
// SamrDeleteUser(IN OUT SAMPR_HANDLE *UserHandle)
// {
//     PSAM_DB_OBJECT UserObject;
//     NTSTATUS Status;
//
//     TRACE("SamrDeleteUser(%p)\n", UserHandle);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the user handle */
//     Status = SampValidateDbObject(*UserHandle,
//                                   SamDbUserObject,
//                                   DELETE,
//                                   &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Fail, if the user is built-in */
//     if (UserObject->RelativeId < 1000)
//     {
//         TRACE("You can not delete a special account!\n");
//         Status = STATUS_SPECIAL_ACCOUNT;
//         goto done;
//     }
//
//     /* Remove the user from all groups */
//     Status = SampRemoveUserFromAllGroups(UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveUserFromAllGroups() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Remove the user from all aliases */
//     Status = SampRemoveUserFromAllAliases(UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveUserFromAllAliases() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Delete the user from the database */
//     Status = SampDeleteAccountDbObject(UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampDeleteAccountDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Invalidate the handle */
//     *UserHandle = NULL;
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserGeneral(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->General.PrimaryGroupId = FixedData.PrimaryGroupId;
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           &InfoBuffer->General.UserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the FullName string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           &InfoBuffer->General.FullName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the AdminComment string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"AdminComment",
//                                           &InfoBuffer->General.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the UserComment string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"UserComment",
//                                           &InfoBuffer->General.UserComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->General.UserName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.UserName.Buffer);
//
//             if (InfoBuffer->General.FullName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.FullName.Buffer);
//
//             if (InfoBuffer->General.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.AdminComment.Buffer);
//
//             if (InfoBuffer->General.UserComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->General.UserComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserPreferences(PSAM_DB_OBJECT UserObject,
//                          PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Preferences.CountryCode = FixedData.CountryCode;
//     InfoBuffer->Preferences.CodePage = FixedData.CodePage;
//
//     /* Get the UserComment string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"UserComment",
//                                           &InfoBuffer->Preferences.UserComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Preferences.UserComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Preferences.UserComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserLogon(PSAM_DB_OBJECT UserObject,
//                    PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA DomainFixedData;
//     SAM_USER_FIXED_DATA FixedData;
//     LARGE_INTEGER PasswordCanChange;
//     LARGE_INTEGER PasswordMustChange;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the fixed size domain data */
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject->ParentObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&DomainFixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the fixed size user data */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Logon.UserId = FixedData.UserId;
//     InfoBuffer->Logon.PrimaryGroupId = FixedData.PrimaryGroupId;
//     InfoBuffer->Logon.LastLogon.LowPart = FixedData.LastLogon.LowPart;
//     InfoBuffer->Logon.LastLogon.HighPart = FixedData.LastLogon.HighPart;
//     InfoBuffer->Logon.LastLogoff.LowPart = FixedData.LastLogoff.LowPart;
//     InfoBuffer->Logon.LastLogoff.HighPart = FixedData.LastLogoff.HighPart;
//     InfoBuffer->Logon.PasswordLastSet.LowPart = FixedData.PasswordLastSet.LowPart;
//     InfoBuffer->Logon.PasswordLastSet.HighPart = FixedData.PasswordLastSet.HighPart;
//     InfoBuffer->Logon.BadPasswordCount = FixedData.BadPasswordCount;
//     InfoBuffer->Logon.LogonCount = FixedData.LogonCount;
//     InfoBuffer->Logon.UserAccountControl = FixedData.UserAccountControl;
//
//     PasswordCanChange = SampAddRelativeTimeToTime(FixedData.PasswordLastSet,
//                                                   DomainFixedData.MinPasswordAge);
//     InfoBuffer->Logon.PasswordCanChange.LowPart = PasswordCanChange.LowPart;
//     InfoBuffer->Logon.PasswordCanChange.HighPart = PasswordCanChange.HighPart;
//
//     PasswordMustChange = SampAddRelativeTimeToTime(FixedData.PasswordLastSet,
//                                                    DomainFixedData.MaxPasswordAge);
//     InfoBuffer->Logon.PasswordMustChange.LowPart = PasswordMustChange.LowPart;
//     InfoBuffer->Logon.PasswordMustChange.HighPart = PasswordMustChange.HighPart;
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           &InfoBuffer->Logon.UserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the FullName string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           &InfoBuffer->Logon.FullName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the HomeDirectory string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"HomeDirectory",
//                                           &InfoBuffer->Logon.HomeDirectory);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the HomeDirectoryDrive string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"HomeDirectoryDrive",
//                                           &InfoBuffer->Logon.HomeDirectoryDrive);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the ScriptPath string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"ScriptPath",
//                                           &InfoBuffer->Logon.ScriptPath);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the ProfilePath string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"ProfilePath",
//                                           &InfoBuffer->Logon.ProfilePath);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the WorkStations string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"WorkStations",
//                                           &InfoBuffer->Logon.WorkStations);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the LogonHours attribute */
//     Status = SampGetLogonHoursAttribute(UserObject,
//                                        &InfoBuffer->Logon.LogonHours);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Logon.UserName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.UserName.Buffer);
//
//             if (InfoBuffer->Logon.FullName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.FullName.Buffer);
//
//             if (InfoBuffer->Logon.HomeDirectory.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.HomeDirectory.Buffer);
//
//             if (InfoBuffer->Logon.HomeDirectoryDrive.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.HomeDirectoryDrive.Buffer);
//
//             if (InfoBuffer->Logon.ScriptPath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.ScriptPath.Buffer);
//
//             if (InfoBuffer->Logon.ProfilePath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.ProfilePath.Buffer);
//
//             if (InfoBuffer->Logon.WorkStations.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Logon.WorkStations.Buffer);
//
//             if (InfoBuffer->Logon.LogonHours.LogonHours != NULL)
//                 midl_user_free(InfoBuffer->Logon.LogonHours.LogonHours);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserAccount(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Account.UserId = FixedData.UserId;
//     InfoBuffer->Account.PrimaryGroupId = FixedData.PrimaryGroupId;
//     InfoBuffer->Account.LastLogon.LowPart = FixedData.LastLogon.LowPart;
//     InfoBuffer->Account.LastLogon.HighPart = FixedData.LastLogon.HighPart;
//     InfoBuffer->Account.LastLogoff.LowPart = FixedData.LastLogoff.LowPart;
//     InfoBuffer->Account.LastLogoff.HighPart = FixedData.LastLogoff.HighPart;
//     InfoBuffer->Account.PasswordLastSet.LowPart = FixedData.PasswordLastSet.LowPart;
//     InfoBuffer->Account.PasswordLastSet.HighPart = FixedData.PasswordLastSet.HighPart;
//     InfoBuffer->Account.AccountExpires.LowPart = FixedData.AccountExpires.LowPart;
//     InfoBuffer->Account.AccountExpires.HighPart = FixedData.AccountExpires.HighPart;
//     InfoBuffer->Account.BadPasswordCount = FixedData.BadPasswordCount;
//     InfoBuffer->Account.LogonCount = FixedData.LogonCount;
//     InfoBuffer->Account.UserAccountControl = FixedData.UserAccountControl;
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           &InfoBuffer->Account.UserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the FullName string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           &InfoBuffer->Account.FullName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the HomeDirectory string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"HomeDirectory",
//                                           &InfoBuffer->Account.HomeDirectory);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the HomeDirectoryDrive string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"HomeDirectoryDrive",
//                                           &InfoBuffer->Account.HomeDirectoryDrive);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the ScriptPath string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"ScriptPath",
//                                           &InfoBuffer->Account.ScriptPath);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the ProfilePath string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"ProfilePath",
//                                           &InfoBuffer->Account.ProfilePath);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the AdminComment string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"AdminComment",
//                                           &InfoBuffer->Account.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the WorkStations string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"WorkStations",
//                                           &InfoBuffer->Account.WorkStations);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the LogonHours attribute */
//     Status = SampGetLogonHoursAttribute(UserObject,
//                                        &InfoBuffer->Account.LogonHours);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Account.UserName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.UserName.Buffer);
//
//             if (InfoBuffer->Account.FullName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.FullName.Buffer);
//
//             if (InfoBuffer->Account.HomeDirectory.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.HomeDirectory.Buffer);
//
//             if (InfoBuffer->Account.HomeDirectoryDrive.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.HomeDirectoryDrive.Buffer);
//
//             if (InfoBuffer->Account.ScriptPath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.ScriptPath.Buffer);
//
//             if (InfoBuffer->Account.ProfilePath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.ProfilePath.Buffer);
//
//             if (InfoBuffer->Account.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.AdminComment.Buffer);
//
//             if (InfoBuffer->Account.WorkStations.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Account.WorkStations.Buffer);
//
//             if (InfoBuffer->Account.LogonHours.LogonHours != NULL)
//                 midl_user_free(InfoBuffer->Account.LogonHours.LogonHours);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserLogonHours(PSAM_DB_OBJECT UserObject,
//                         PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n", UserObject, Buffer);
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//     {
//         TRACE("Failed to allocate InfoBuffer!\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Status = SampGetLogonHoursAttribute(UserObject,
//                                         &InfoBuffer->LogonHours.LogonHours);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetLogonHoursAttribute failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->LogonHours.LogonHours.LogonHours != NULL)
//                 midl_user_free(InfoBuffer->LogonHours.LogonHours.LogonHours);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserName(PSAM_DB_OBJECT UserObject,
//                   PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           &InfoBuffer->Name.UserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the FullName string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           &InfoBuffer->Name.FullName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Name.UserName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Name.UserName.Buffer);
//
//             if (InfoBuffer->Name.FullName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Name.FullName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserAccountName(PSAM_DB_OBJECT UserObject,
//                          PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the Name string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           &InfoBuffer->AccountName.UserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->AccountName.UserName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->AccountName.UserName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserFullName(PSAM_DB_OBJECT UserObject,
//                       PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the FullName string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           &InfoBuffer->FullName.FullName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->FullName.FullName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->FullName.FullName.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserPrimaryGroup(PSAM_DB_OBJECT UserObject,
//                           PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->PrimaryGroup.PrimaryGroupId = FixedData.PrimaryGroupId;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserHome(PSAM_DB_OBJECT UserObject,
//                   PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the HomeDirectory string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"HomeDirectory",
//                                           &InfoBuffer->Home.HomeDirectory);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the HomeDirectoryDrive string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"HomeDirectoryDrive",
//                                           &InfoBuffer->Home.HomeDirectoryDrive);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Home.HomeDirectory.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Home.HomeDirectory.Buffer);
//
//             if (InfoBuffer->Home.HomeDirectoryDrive.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Home.HomeDirectoryDrive.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserScript(PSAM_DB_OBJECT UserObject,
//                     PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the ScriptPath string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"ScriptPath",
//                                           &InfoBuffer->Script.ScriptPath);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Script.ScriptPath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Script.ScriptPath.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserProfile(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the ProfilePath string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"ProfilePath",
//                                           &InfoBuffer->Profile.ProfilePath);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Profile.ProfilePath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Profile.ProfilePath.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserAdminComment(PSAM_DB_OBJECT UserObject,
//                           PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the AdminComment string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"AdminComment",
//                                           &InfoBuffer->AdminComment.AdminComment);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->AdminComment.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->AdminComment.AdminComment.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserWorkStations(PSAM_DB_OBJECT UserObject,
//                           PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the WorkStations string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"WorkStations",
//                                           &InfoBuffer->WorkStations.WorkStations);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->WorkStations.WorkStations.Buffer != NULL)
//                 midl_user_free(InfoBuffer->WorkStations.WorkStations.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserControl(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Control.UserAccountControl = FixedData.UserAccountControl;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserExpires(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Expires.AccountExpires.LowPart = FixedData.AccountExpires.LowPart;
//     InfoBuffer->Expires.AccountExpires.HighPart = FixedData.AccountExpires.HighPart;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserInternal1(PSAM_DB_OBJECT UserObject,
//                        PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     ULONG Length = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Fail, if the caller is not a trusted caller */
//     if (UserObject->Trusted == FALSE)
//         return STATUS_INVALID_INFO_CLASS;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     InfoBuffer->Internal1.LmPasswordPresent = FALSE;
//     InfoBuffer->Internal1.NtPasswordPresent = FALSE;
//
//     /* Get the NT password */
//     Length = 0;
//     SampGetObjectAttribute(UserObject,
//                            L"NTPwd",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == sizeof(ENCRYPTED_NT_OWF_PASSWORD))
//     {
//         Status = SampGetObjectAttribute(UserObject,
//                                         L"NTPwd",
//                                         NULL,
//                                         (PVOID)&InfoBuffer->Internal1.EncryptedNtOwfPassword,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         if (memcmp(&InfoBuffer->Internal1.EncryptedNtOwfPassword,
//                    &EmptyNtHash,
//                    sizeof(ENCRYPTED_NT_OWF_PASSWORD)))
//             InfoBuffer->Internal1.NtPasswordPresent = TRUE;
//     }
//
//
//     /* Get the LM password */
//     Length = 0;
//     SampGetObjectAttribute(UserObject,
//                            L"LMPwd",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     if (Length == sizeof(ENCRYPTED_LM_OWF_PASSWORD))
//     {
//         Status = SampGetObjectAttribute(UserObject,
//                                         L"LMPwd",
//                                         NULL,
//                                         (PVOID)&InfoBuffer->Internal1.EncryptedLmOwfPassword,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         if (memcmp(&InfoBuffer->Internal1.EncryptedLmOwfPassword,
//                    &EmptyLmHash,
//                    sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//             InfoBuffer->Internal1.LmPasswordPresent = TRUE;
//     }
//
//     InfoBuffer->Internal1.PasswordExpired = FALSE;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// SampQueryUserInternal2(PSAM_DB_OBJECT UserObject,
//                        PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     InfoBuffer->Internal2.Flags = 0;
//     InfoBuffer->Internal2.LastLogon.LowPart = FixedData.LastLogon.LowPart;
//     InfoBuffer->Internal2.LastLogon.HighPart = FixedData.LastLogon.HighPart;
//     InfoBuffer->Internal2.LastLogoff.LowPart = FixedData.LastLogoff.LowPart;
//     InfoBuffer->Internal2.LastLogoff.HighPart = FixedData.LastLogoff.HighPart;
//     InfoBuffer->Internal2.BadPasswordCount = FixedData.BadPasswordCount;
//     InfoBuffer->Internal2.LogonCount = FixedData.LogonCount;
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserParameters(PSAM_DB_OBJECT UserObject,
//                         PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the Parameters string */
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Parameters",
//                                           &InfoBuffer->Parameters.Parameters);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("Status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->Parameters.Parameters.Buffer != NULL)
//                 midl_user_free(InfoBuffer->Parameters.Parameters.Buffer);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampQueryUserAll(PSAM_DB_OBJECT UserObject,
//                  PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAMPR_USER_INFO_BUFFER InfoBuffer = NULL;
//     SAM_DOMAIN_FIXED_DATA DomainFixedData;
//     SAM_USER_FIXED_DATA FixedData;
//     LARGE_INTEGER PasswordCanChange;
//     LARGE_INTEGER PasswordMustChange;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     *Buffer = NULL;
//
//     InfoBuffer = midl_user_allocate(sizeof(SAMPR_USER_INFO_BUFFER));
//     if (InfoBuffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Get the fixed size domain data */
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject->ParentObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&DomainFixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the fixed size user data */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the fields to be returned */
//     if (UserObject->Trusted)
//     {
//         InfoBuffer->All.WhichFields = USER_ALL_READ_GENERAL_MASK |
//                                       USER_ALL_READ_LOGON_MASK |
//                                       USER_ALL_READ_ACCOUNT_MASK |
//                                       USER_ALL_READ_PREFERENCES_MASK |
//                                       USER_ALL_READ_TRUSTED_MASK;
//     }
//     else
//     {
//         InfoBuffer->All.WhichFields = 0;
//
//         if (UserObject->Access & USER_READ_GENERAL)
//             InfoBuffer->All.WhichFields |= USER_ALL_READ_GENERAL_MASK;
//
//         if (UserObject->Access & USER_READ_LOGON)
//             InfoBuffer->All.WhichFields |= USER_ALL_READ_LOGON_MASK;
//
//         if (UserObject->Access & USER_READ_ACCOUNT)
//             InfoBuffer->All.WhichFields |= USER_ALL_READ_ACCOUNT_MASK;
//
//         if (UserObject->Access & USER_READ_PREFERENCES)
//             InfoBuffer->All.WhichFields |= USER_ALL_READ_PREFERENCES_MASK;
//     }
//
//     /* Fail, if no fields are to be returned */
//     if (InfoBuffer->All.WhichFields == 0)
//     {
//         Status = STATUS_ACCESS_DENIED;
//         goto done;
//     }
//
//     /* Get the UserName attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_USERNAME)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"Name",
//                                               &InfoBuffer->All.UserName);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the FullName attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_FULLNAME)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"FullName",
//                                               &InfoBuffer->All.FullName);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the UserId attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_USERID)
//     {
//         InfoBuffer->All.UserId = FixedData.UserId;
//     }
//
//     /* Get the PrimaryGroupId attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_PRIMARYGROUPID)
//     {
//         InfoBuffer->All.PrimaryGroupId = FixedData.PrimaryGroupId;
//     }
//
//     /* Get the AdminComment attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_ADMINCOMMENT)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"AdminComment",
//                                               &InfoBuffer->All.AdminComment);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the UserComment attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_USERCOMMENT)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"UserComment",
//                                               &InfoBuffer->All.UserComment);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the HomeDirectory attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_HOMEDIRECTORY)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"HomeDirectory",
//                                               &InfoBuffer->All.HomeDirectory);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the HomeDirectoryDrive attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_HOMEDIRECTORYDRIVE)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"HomeDirectoryDrive",
//                                               &InfoBuffer->Home.HomeDirectoryDrive);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the ScriptPath attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_SCRIPTPATH)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"ScriptPath",
//                                               &InfoBuffer->All.ScriptPath);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the ProfilePath attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_PROFILEPATH)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"ProfilePath",
//                                               &InfoBuffer->All.ProfilePath);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the WorkStations attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_WORKSTATIONS)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"WorkStations",
//                                               &InfoBuffer->All.WorkStations);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the LastLogon attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_LASTLOGON)
//     {
//         InfoBuffer->All.LastLogon.LowPart = FixedData.LastLogon.LowPart;
//         InfoBuffer->All.LastLogon.HighPart = FixedData.LastLogon.HighPart;
//     }
//
//     /* Get the LastLogoff attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_LASTLOGOFF)
//     {
//         InfoBuffer->All.LastLogoff.LowPart = FixedData.LastLogoff.LowPart;
//         InfoBuffer->All.LastLogoff.HighPart = FixedData.LastLogoff.HighPart;
//     }
//
//     /* Get the LogonHours attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_LOGONHOURS)
//     {
//         Status = SampGetLogonHoursAttribute(UserObject,
//                                             &InfoBuffer->All.LogonHours);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the BadPasswordCount attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_BADPASSWORDCOUNT)
//     {
//         InfoBuffer->All.BadPasswordCount = FixedData.BadPasswordCount;
//     }
//
//     /* Get the LogonCount attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_LOGONCOUNT)
//     {
//         InfoBuffer->All.LogonCount = FixedData.LogonCount;
//     }
//
//     /* Get the PasswordCanChange attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_PASSWORDCANCHANGE)
//     {
//         PasswordCanChange = SampAddRelativeTimeToTime(FixedData.PasswordLastSet,
//                                                       DomainFixedData.MinPasswordAge);
//         InfoBuffer->All.PasswordCanChange.LowPart = PasswordCanChange.LowPart;
//         InfoBuffer->All.PasswordCanChange.HighPart = PasswordCanChange.HighPart;
//     }
//
//     /* Get the PasswordMustChange attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_PASSWORDMUSTCHANGE)
//     {
//         PasswordMustChange = SampAddRelativeTimeToTime(FixedData.PasswordLastSet,
//                                                        DomainFixedData.MaxPasswordAge);
//         InfoBuffer->All.PasswordMustChange.LowPart = PasswordMustChange.LowPart;
//         InfoBuffer->All.PasswordMustChange.HighPart = PasswordMustChange.HighPart;
//     }
//
//     /* Get the PasswordLastSet attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_PASSWORDLASTSET)
//     {
//         InfoBuffer->All.PasswordLastSet.LowPart = FixedData.PasswordLastSet.LowPart;
//         InfoBuffer->All.PasswordLastSet.HighPart = FixedData.PasswordLastSet.HighPart;
//     }
//
//     /* Get the AccountExpires attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_ACCOUNTEXPIRES)
//     {
//         InfoBuffer->All.AccountExpires.LowPart = FixedData.AccountExpires.LowPart;
//         InfoBuffer->All.AccountExpires.HighPart = FixedData.AccountExpires.HighPart;
//     }
//
//     /* Get the UserAccountControl attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_USERACCOUNTCONTROL)
//     {
//         InfoBuffer->All.UserAccountControl = FixedData.UserAccountControl;
//     }
//
//     /* Get the Parameters attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_PARAMETERS)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"Parameters",
//                                               &InfoBuffer->All.Parameters);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     /* Get the CountryCode attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_COUNTRYCODE)
//     {
//         InfoBuffer->All.CountryCode = FixedData.CountryCode;
//     }
//
//     /* Get the CodePage attribute */
//     if (InfoBuffer->All.WhichFields & USER_ALL_CODEPAGE)
//     {
//         InfoBuffer->All.CodePage = FixedData.CodePage;
//     }
//
//     /* Get the LmPassword and NtPassword attributes */
//     if (InfoBuffer->All.WhichFields & (USER_ALL_NTPASSWORDPRESENT | USER_ALL_LMPASSWORDPRESENT))
//     {
//         InfoBuffer->All.LmPasswordPresent = FALSE;
//         InfoBuffer->All.NtPasswordPresent = FALSE;
//
//         /* Get the NT password */
//         Length = 0;
//         SampGetObjectAttribute(UserObject,
//                                L"NTPwd",
//                                NULL,
//                                NULL,
//                                &Length);
//
//         if (Length == sizeof(ENCRYPTED_NT_OWF_PASSWORD))
//         {
//             InfoBuffer->All.NtOwfPassword.Buffer = midl_user_allocate(sizeof(ENCRYPTED_NT_OWF_PASSWORD));
//             if (InfoBuffer->All.NtOwfPassword.Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             InfoBuffer->All.NtOwfPassword.Length = sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//             InfoBuffer->All.NtOwfPassword.MaximumLength = sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//
//             Status = SampGetObjectAttribute(UserObject,
//                                             L"NTPwd",
//                                             NULL,
//                                             (PVOID)InfoBuffer->All.NtOwfPassword.Buffer,
//                                             &Length);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             if (memcmp(InfoBuffer->All.NtOwfPassword.Buffer,
//                        &EmptyNtHash,
//                        sizeof(ENCRYPTED_NT_OWF_PASSWORD)))
//                 InfoBuffer->All.NtPasswordPresent = TRUE;
//         }
//
//         /* Get the LM password */
//         Length = 0;
//         SampGetObjectAttribute(UserObject,
//                                L"LMPwd",
//                                NULL,
//                                NULL,
//                                &Length);
//
//         if (Length == sizeof(ENCRYPTED_LM_OWF_PASSWORD))
//         {
//             InfoBuffer->All.LmOwfPassword.Buffer = midl_user_allocate(sizeof(ENCRYPTED_LM_OWF_PASSWORD));
//             if (InfoBuffer->All.LmOwfPassword.Buffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             InfoBuffer->All.LmOwfPassword.Length = sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//             InfoBuffer->All.LmOwfPassword.MaximumLength = sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//
//             Status = SampGetObjectAttribute(UserObject,
//                                             L"LMPwd",
//                                             NULL,
//                                             (PVOID)InfoBuffer->All.LmOwfPassword.Buffer,
//                                             &Length);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//
//             if (memcmp(InfoBuffer->All.LmOwfPassword.Buffer,
//                        &EmptyLmHash,
//                        sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//                 InfoBuffer->All.LmPasswordPresent = TRUE;
//         }
//     }
//
//     if (InfoBuffer->All.WhichFields & USER_ALL_PRIVATEDATA)
//     {
//         Status = SampGetObjectAttributeString(UserObject,
//                                               L"PrivateData",
//                                               &InfoBuffer->All.PrivateData);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("Status 0x%08lx\n", Status);
//             goto done;
//         }
//     }
//
//     if (InfoBuffer->All.WhichFields & USER_ALL_PASSWORDEXPIRED)
//     {
//         /* FIXME */
//     }
//
//     if (InfoBuffer->All.WhichFields & USER_ALL_SECURITYDESCRIPTOR)
//     {
//         Length = 0;
//         SampGetObjectAttribute(UserObject,
//                                L"SecDesc",
//                                NULL,
//                                NULL,
//                                &Length);
//
//         if (Length > 0)
//         {
//             InfoBuffer->All.SecurityDescriptor.SecurityDescriptor = midl_user_allocate(Length);
//             if (InfoBuffer->All.SecurityDescriptor.SecurityDescriptor == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             InfoBuffer->All.SecurityDescriptor.Length = Length;
//
//             Status = SampGetObjectAttribute(UserObject,
//                                             L"SecDesc",
//                                             NULL,
//                                             (PVOID)InfoBuffer->All.SecurityDescriptor.SecurityDescriptor,
//                                             &Length);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//         }
//     }
//
//     *Buffer = InfoBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (InfoBuffer != NULL)
//         {
//             if (InfoBuffer->All.UserName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.UserName.Buffer);
//
//             if (InfoBuffer->All.FullName.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.FullName.Buffer);
//
//             if (InfoBuffer->All.AdminComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.AdminComment.Buffer);
//
//             if (InfoBuffer->All.UserComment.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.UserComment.Buffer);
//
//             if (InfoBuffer->All.HomeDirectory.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.HomeDirectory.Buffer);
//
//             if (InfoBuffer->All.HomeDirectoryDrive.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.HomeDirectoryDrive.Buffer);
//
//             if (InfoBuffer->All.ScriptPath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.ScriptPath.Buffer);
//
//             if (InfoBuffer->All.ProfilePath.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.ProfilePath.Buffer);
//
//             if (InfoBuffer->All.WorkStations.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.WorkStations.Buffer);
//
//             if (InfoBuffer->All.LogonHours.LogonHours != NULL)
//                 midl_user_free(InfoBuffer->All.LogonHours.LogonHours);
//
//             if (InfoBuffer->All.Parameters.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.Parameters.Buffer);
//
//             if (InfoBuffer->All.LmOwfPassword.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.LmOwfPassword.Buffer);
//
//             if (InfoBuffer->All.NtOwfPassword.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.NtOwfPassword.Buffer);
//
//             if (InfoBuffer->All.PrivateData.Buffer != NULL)
//                 midl_user_free(InfoBuffer->All.PrivateData.Buffer);
//
//             if (InfoBuffer->All.SecurityDescriptor.SecurityDescriptor != NULL)
//                 midl_user_free(InfoBuffer->All.SecurityDescriptor.SecurityDescriptor);
//
//             midl_user_free(InfoBuffer);
//         }
//     }
//
//     return Status;
// }
//
//
// /* Function 36 */
// NTSTATUS
// NTAPI
// SamrQueryInformationUser(IN SAMPR_HANDLE UserHandle,
//                          IN USER_INFORMATION_CLASS UserInformationClass,
//                          OUT PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     TRACE("SamrQueryInformationUser(%p %lu %p)\n",
//           UserHandle, UserInformationClass, Buffer);
//
//     return SamrQueryInformationUser2(UserHandle,
//                                      UserInformationClass,
//                                      Buffer);
// }
//
//
// static NTSTATUS
// SampSetUserName(PSAM_DB_OBJECT UserObject,
//                 PRPC_UNICODE_STRING NewUserName)
// {
//     UNICODE_STRING OldUserName = {0, 0, NULL};
//     NTSTATUS Status;
//
//     /* Check the account name */
//     Status = SampCheckAccountName(NewUserName, 20);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     Status = SampGetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           (PRPC_UNICODE_STRING)&OldUserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttributeString failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     if (!RtlEqualUnicodeString(&OldUserName, (PCUNICODE_STRING)NewUserName, TRUE))
//     {
//         Status = SampCheckAccountNameInDomain(UserObject->ParentObject,
//                                               NewUserName->Buffer);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("User name \'%S\' already exists in domain (Status 0x%08lx)\n",
//                   NewUserName->Buffer, Status);
//             goto done;
//         }
//     }
//
//     Status = SampSetAccountNameInDomain(UserObject->ParentObject,
//                                         L"Users",
//                                         NewUserName->Buffer,
//                                         UserObject->RelativeId);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetAccountNameInDomain failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampRemoveAccountNameFromDomain(UserObject->ParentObject,
//                                              L"Users",
//                                              OldUserName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveAccountNameFromDomain failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           NewUserName);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampSetObjectAttribute failed (Status 0x%08lx)\n", Status);
//     }
//
// done:
//     if (OldUserName.Buffer != NULL)
//         midl_user_free(OldUserName.Buffer);
//
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserGeneral(PSAM_DB_OBJECT UserObject,
//                    PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.PrimaryGroupId = Buffer->General.PrimaryGroupId;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetUserName(UserObject,
//                              &Buffer->General.UserName);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           &Buffer->General.FullName);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"AdminComment",
//                                           &Buffer->General.AdminComment);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"UserComment",
//                                           &Buffer->General.UserComment);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserPreferences(PSAM_DB_OBJECT UserObject,
//                        PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.CountryCode = Buffer->Preferences.CountryCode;
//     FixedData.CodePage = Buffer->Preferences.CodePage;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"UserComment",
//                                           &Buffer->Preferences.UserComment);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserPrimaryGroup(PSAM_DB_OBJECT UserObject,
//                         PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.PrimaryGroupId = Buffer->PrimaryGroup.PrimaryGroupId;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserControl(PSAM_DB_OBJECT UserObject,
//                    PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.UserAccountControl = Buffer->Control.UserAccountControl;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserExpires(PSAM_DB_OBJECT UserObject,
//                    PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     FixedData.AccountExpires.LowPart = Buffer->Expires.AccountExpires.LowPart;
//     FixedData.AccountExpires.HighPart = Buffer->Expires.AccountExpires.HighPart;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserInternal1(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* FIXME: Decrypt NT password */
//     /* FIXME: Decrypt LM password */
//
//     Status = SampSetUserPassword(UserObject,
//                                  &Buffer->Internal1.EncryptedNtOwfPassword,
//                                  Buffer->Internal1.NtPasswordPresent,
//                                  &Buffer->Internal1.EncryptedLmOwfPassword,
//                                  Buffer->Internal1.LmPasswordPresent);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the fixed user attributes */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (Buffer->Internal1.PasswordExpired)
//     {
//         /* The password was last set ages ago */
//         FixedData.PasswordLastSet.LowPart = 0;
//         FixedData.PasswordLastSet.HighPart = 0;
//     }
//     else
//     {
//         /* The password was last set right now */
//         Status = NtQuerySystemTime(&FixedData.PasswordLastSet);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     /* Set the fixed user attributes */
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserInternal2(PSAM_DB_OBJECT UserObject,
//                      PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Get the fixed user attributes */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if ((Buffer->Internal2.Flags & USER_LOGON_SUCCESS) &&
//         ((Buffer->Internal2.Flags & ~USER_LOGON_SUCCESS) == 0))
//     {
//         /* Update the LastLogon time */
//         Status = NtQuerySystemTime(&FixedData.LastLogon);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         FixedData.LogonCount++;
//         FixedData.BadPasswordCount = 0;
//     }
//
//     if ((Buffer->Internal2.Flags & USER_LOGON_BAD_PASSWORD) &&
//         ((Buffer->Internal2.Flags & ~USER_LOGON_BAD_PASSWORD) == 0))
//     {
//         /* Update the LastBadPasswordTime */
//         Status = NtQuerySystemTime(&FixedData.LastBadPasswordTime);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         FixedData.BadPasswordCount++;
//     }
//
//     /* Set the fixed user attributes */
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedData,
//                                     Length);
//
// done:
//     return Status;
// }
//
//
// static NTSTATUS
// SampSetUserAll(PSAM_DB_OBJECT UserObject,
//                PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     SAM_USER_FIXED_DATA FixedData;
//     ULONG Length = 0;
//     ULONG WhichFields;
//     PENCRYPTED_NT_OWF_PASSWORD NtPassword = NULL;
//     PENCRYPTED_LM_OWF_PASSWORD LmPassword = NULL;
//     BOOLEAN NtPasswordPresent = FALSE;
//     BOOLEAN LmPasswordPresent = FALSE;
//     BOOLEAN WriteFixedData = FALSE;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     WhichFields = Buffer->All.WhichFields;
//
//     /* Get the fixed size attributes */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (WhichFields & USER_ALL_USERNAME)
//     {
//         Status = SampSetUserName(UserObject,
//                                  &Buffer->All.UserName);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_FULLNAME)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"FullName",
//                                               &Buffer->All.FullName);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_ADMINCOMMENT)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"AdminComment",
//                                               &Buffer->All.AdminComment);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_USERCOMMENT)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"UserComment",
//                                               &Buffer->All.UserComment);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_HOMEDIRECTORY)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"HomeDirectory",
//                                               &Buffer->All.HomeDirectory);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_HOMEDIRECTORYDRIVE)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"HomeDirectoryDrive",
//                                               &Buffer->All.HomeDirectoryDrive);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_SCRIPTPATH)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"ScriptPath",
//                                               &Buffer->All.ScriptPath);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_PROFILEPATH)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"ProfilePath",
//                                               &Buffer->All.ProfilePath);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_WORKSTATIONS)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"WorkStations",
//                                               &Buffer->All.WorkStations);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_PARAMETERS)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"Parameters",
//                                               &Buffer->All.Parameters);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_LOGONHOURS)
//     {
//         Status = SampSetLogonHoursAttribute(UserObject,
//                                            &Buffer->All.LogonHours);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_PRIMARYGROUPID)
//     {
//         FixedData.PrimaryGroupId = Buffer->All.PrimaryGroupId;
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & USER_ALL_ACCOUNTEXPIRES)
//     {
//         FixedData.AccountExpires.LowPart = Buffer->All.AccountExpires.LowPart;
//         FixedData.AccountExpires.HighPart = Buffer->All.AccountExpires.HighPart;
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & USER_ALL_USERACCOUNTCONTROL)
//     {
//         FixedData.UserAccountControl = Buffer->All.UserAccountControl;
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & USER_ALL_COUNTRYCODE)
//     {
//         FixedData.CountryCode = Buffer->All.CountryCode;
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & USER_ALL_CODEPAGE)
//     {
//         FixedData.CodePage = Buffer->All.CodePage;
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & (USER_ALL_NTPASSWORDPRESENT |
//                        USER_ALL_LMPASSWORDPRESENT))
//     {
//         if (WhichFields & USER_ALL_NTPASSWORDPRESENT)
//         {
//             NtPassword = (PENCRYPTED_NT_OWF_PASSWORD)Buffer->All.NtOwfPassword.Buffer;
//             NtPasswordPresent = Buffer->All.NtPasswordPresent;
//         }
//
//         if (WhichFields & USER_ALL_LMPASSWORDPRESENT)
//         {
//             LmPassword = (PENCRYPTED_LM_OWF_PASSWORD)Buffer->All.LmOwfPassword.Buffer;
//             LmPasswordPresent = Buffer->All.LmPasswordPresent;
//         }
//
//         Status = SampSetUserPassword(UserObject,
//                                      NtPassword,
//                                      NtPasswordPresent,
//                                      LmPassword,
//                                      LmPasswordPresent);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         /* The password has just been set */
//         Status = NtQuerySystemTime(&FixedData.PasswordLastSet);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & USER_ALL_PRIVATEDATA)
//     {
//         Status = SampSetObjectAttributeString(UserObject,
//                                               L"PrivateData",
//                                               &Buffer->All.PrivateData);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     if (WhichFields & USER_ALL_PASSWORDEXPIRED)
//     {
//         if (Buffer->All.PasswordExpired)
//         {
//             /* The password was last set ages ago */
//             FixedData.PasswordLastSet.LowPart = 0;
//             FixedData.PasswordLastSet.HighPart = 0;
//         }
//         else
//         {
//             /* The password was last set right now */
//             Status = NtQuerySystemTime(&FixedData.PasswordLastSet);
//             if (!NT_SUCCESS(Status))
//                 goto done;
//         }
//
//         WriteFixedData = TRUE;
//     }
//
//     if (WhichFields & USER_ALL_SECURITYDESCRIPTOR)
//     {
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"SecDesc",
//                                         REG_BINARY,
//                                         Buffer->All.SecurityDescriptor.SecurityDescriptor,
//                                         Buffer->All.SecurityDescriptor.Length);
//     }
//
//     if (WriteFixedData != FALSE)
//     {
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"F",
//                                         REG_BINARY,
//                                         &FixedData,
//                                         Length);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
// done:
//     return Status;
// }
//
//
// /* Function 37 */
// NTSTATUS
// NTAPI
// SamrSetInformationUser(IN SAMPR_HANDLE UserHandle,
//                        IN USER_INFORMATION_CLASS UserInformationClass,
//                        IN PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     TRACE("SamrSetInformationUser(%p %lu %p)\n",
//           UserHandle, UserInformationClass, Buffer);
//
//     return SamrSetInformationUser2(UserHandle,
//                                    UserInformationClass,
//                                    Buffer);
// }
//
//
// /* Function 38 */
// NTSTATUS
// NTAPI
// SamrChangePasswordUser(IN SAMPR_HANDLE UserHandle,
//                        IN unsigned char LmPresent,
//                        IN PENCRYPTED_LM_OWF_PASSWORD OldLmEncryptedWithNewLm,
//                        IN PENCRYPTED_LM_OWF_PASSWORD NewLmEncryptedWithOldLm,
//                        IN unsigned char NtPresent,
//                        IN PENCRYPTED_NT_OWF_PASSWORD OldNtEncryptedWithNewNt,
//                        IN PENCRYPTED_NT_OWF_PASSWORD NewNtEncryptedWithOldNt,
//                        IN unsigned char NtCrossEncryptionPresent,
//                        IN PENCRYPTED_NT_OWF_PASSWORD NewNtEncryptedWithNewLm,
//                        IN unsigned char LmCrossEncryptionPresent,
//                        IN PENCRYPTED_LM_OWF_PASSWORD NewLmEncryptedWithNewNt)
// {
//     ENCRYPTED_LM_OWF_PASSWORD StoredLmPassword;
//     ENCRYPTED_NT_OWF_PASSWORD StoredNtPassword;
//     LM_OWF_PASSWORD OldLmPassword;
//     LM_OWF_PASSWORD NewLmPassword;
//     NT_OWF_PASSWORD OldNtPassword;
//     NT_OWF_PASSWORD NewNtPassword;
//     BOOLEAN StoredLmPresent = FALSE;
//     BOOLEAN StoredNtPresent = FALSE;
//     BOOLEAN StoredLmEmpty = TRUE;
//     BOOLEAN StoredNtEmpty = TRUE;
//     PSAM_DB_OBJECT UserObject;
//     ULONG Length;
//     SAM_USER_FIXED_DATA UserFixedData;
//     SAM_DOMAIN_FIXED_DATA DomainFixedData;
//     LARGE_INTEGER SystemTime;
//     NTSTATUS Status;
//
//     DBG_UNREFERENCED_LOCAL_VARIABLE(StoredLmPresent);
//     DBG_UNREFERENCED_LOCAL_VARIABLE(StoredNtPresent);
//     DBG_UNREFERENCED_LOCAL_VARIABLE(StoredLmEmpty);
//
//     TRACE("SamrChangePasswordUser(%p %u %p %p %u %p %p %u %p %u %p)\n",
//           UserHandle, LmPresent, OldLmEncryptedWithNewLm, NewLmEncryptedWithOldLm,
//           NtPresent, OldNtEncryptedWithNewNt, NewNtEncryptedWithOldNt, NtCrossEncryptionPresent,
//           NewNtEncryptedWithNewLm, LmCrossEncryptionPresent, NewLmEncryptedWithNewNt);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the user handle */
//     Status = SampValidateDbObject(UserHandle,
//                                   SamDbUserObject,
//                                   USER_CHANGE_PASSWORD,
//                                   &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get the current time */
//     Status = NtQuerySystemTime(&SystemTime);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("NtQuerySystemTime failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Retrieve the LM password */
//     Length = sizeof(ENCRYPTED_LM_OWF_PASSWORD);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"LMPwd",
//                                     NULL,
//                                     &StoredLmPassword,
//                                     &Length);
//     if (NT_SUCCESS(Status))
//     {
//         if (Length == sizeof(ENCRYPTED_LM_OWF_PASSWORD))
//         {
//             StoredLmPresent = TRUE;
//             if (!RtlEqualMemory(&StoredLmPassword,
//                                 &EmptyLmHash,
//                                 sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//                 StoredLmEmpty = FALSE;
//         }
//     }
//
//     /* Retrieve the NT password */
//     Length = sizeof(ENCRYPTED_NT_OWF_PASSWORD);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"NTPwd",
//                                     NULL,
//                                     &StoredNtPassword,
//                                     &Length);
//     if (NT_SUCCESS(Status))
//     {
//         if (Length == sizeof(ENCRYPTED_NT_OWF_PASSWORD))
//         {
//             StoredNtPresent = TRUE;
//             if (!RtlEqualMemory(&StoredNtPassword,
//                                 &EmptyNtHash,
//                                 sizeof(ENCRYPTED_NT_OWF_PASSWORD)))
//                 StoredNtEmpty = FALSE;
//         }
//     }
//
//     /* Retrieve the fixed size user data */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     &UserFixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttribute failed to retrieve the fixed user data (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check if we can change the password at this time */
//     if ((StoredLmEmpty == FALSE) || (StoredNtEmpty == FALSE))
//     {
//         /* Get fixed domain data */
//         Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//         Status = SampGetObjectAttribute(UserObject->ParentObject,
//                                         L"F",
//                                         NULL,
//                                         &DomainFixedData,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SampGetObjectAttribute failed to retrieve the fixed domain data (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         if (DomainFixedData.MinPasswordAge.QuadPart > 0)
//         {
//             if (SystemTime.QuadPart < (UserFixedData.PasswordLastSet.QuadPart + DomainFixedData.MinPasswordAge.QuadPart))
//             {
//                 Status = STATUS_ACCOUNT_RESTRICTION;
//                 goto done;
//             }
//         }
//     }
//
//     /* Decrypt the LM passwords, if present */
//     if (LmPresent)
//     {
//         Status = SystemFunction013((const BYTE *)NewLmEncryptedWithOldLm,
//                                    (const BYTE *)&StoredLmPassword,
//                                    (LPBYTE)&NewLmPassword);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction013 failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         Status = SystemFunction013((const BYTE *)OldLmEncryptedWithNewLm,
//                                    (const BYTE *)&NewLmPassword,
//                                    (LPBYTE)&OldLmPassword);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction013 failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//     }
//
//     /* Decrypt the NT passwords, if present */
//     if (NtPresent)
//     {
//         Status = SystemFunction013((const BYTE *)NewNtEncryptedWithOldNt,
//                                    (const BYTE *)&StoredNtPassword,
//                                    (LPBYTE)&NewNtPassword);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction013 failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//
//         Status = SystemFunction013((const BYTE *)OldNtEncryptedWithNewNt,
//                                    (const BYTE *)&NewNtPassword,
//                                    (LPBYTE)&OldNtPassword);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SystemFunction013 failed (Status 0x%08lx)\n", Status);
//             goto done;
//         }
//     }
//
//     /* Check if the old passwords match the stored ones */
//     if (NtPresent)
//     {
//         if (LmPresent)
//         {
//             if (!RtlEqualMemory(&StoredLmPassword,
//                                 &OldLmPassword,
//                                 sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//             {
//                 TRACE("Old LM Password does not match!\n");
//                 Status = STATUS_WRONG_PASSWORD;
//             }
//             else
//             {
//                 if (!RtlEqualMemory(&StoredNtPassword,
//                                     &OldNtPassword,
//                                     sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//                 {
//                     TRACE("Old NT Password does not match!\n");
//                     Status = STATUS_WRONG_PASSWORD;
//                 }
//             }
//         }
//         else
//         {
//             if (!RtlEqualMemory(&StoredNtPassword,
//                                 &OldNtPassword,
//                                 sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//             {
//                 TRACE("Old NT Password does not match!\n");
//                 Status = STATUS_WRONG_PASSWORD;
//             }
//         }
//     }
//     else
//     {
//         if (LmPresent)
//         {
//             if (!RtlEqualMemory(&StoredLmPassword,
//                                 &OldLmPassword,
//                                 sizeof(ENCRYPTED_LM_OWF_PASSWORD)))
//             {
//                 TRACE("Old LM Password does not match!\n");
//                 Status = STATUS_WRONG_PASSWORD;
//             }
//         }
//         else
//         {
//             Status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     /* Store the new password hashes */
//     if (NT_SUCCESS(Status))
//     {
//         Status = SampSetUserPassword(UserObject,
//                                      &NewNtPassword,
//                                      NtPresent,
//                                      &NewLmPassword,
//                                      LmPresent);
//         if (NT_SUCCESS(Status))
//         {
//             /* Update PasswordLastSet */
//             UserFixedData.PasswordLastSet.QuadPart = SystemTime.QuadPart;
//
//             /* Set the fixed size user data */
//             Length = sizeof(SAM_USER_FIXED_DATA);
//             Status = SampSetObjectAttribute(UserObject,
//                                             L"F",
//                                             REG_BINARY,
//                                             &UserFixedData,
//                                             Length);
//         }
//     }
//
//     if (Status == STATUS_WRONG_PASSWORD)
//     {
//         /* Update BadPasswordCount and LastBadPasswordTime */
//         UserFixedData.BadPasswordCount++;
//         UserFixedData.LastBadPasswordTime.QuadPart = SystemTime.QuadPart;
//
//         /* Set the fixed size user data */
//         Length = sizeof(SAM_USER_FIXED_DATA);
//         Status = SampSetObjectAttribute(UserObject,
//                                         L"F",
//                                         REG_BINARY,
//                                         &UserFixedData,
//                                         Length);
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 39 */
// NTSTATUS
// NTAPI
// SamrGetGroupsForUser(IN SAMPR_HANDLE UserHandle,
//                      OUT PSAMPR_GET_GROUPS_BUFFER *Groups)
// {
//     PSAMPR_GET_GROUPS_BUFFER GroupsBuffer = NULL;
//     PSAM_DB_OBJECT UserObject;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     TRACE("SamrGetGroupsForUser(%p %p)\n",
//           UserHandle, Groups);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the user handle */
//     Status = SampValidateDbObject(UserHandle,
//                                   SamDbUserObject,
//                                   USER_LIST_GROUPS,
//                                   &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Allocate the groups buffer */
//     GroupsBuffer = midl_user_allocate(sizeof(SAMPR_GET_GROUPS_BUFFER));
//     if (GroupsBuffer == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /*
//      * Get the size of the Groups attribute.
//      * Do not check the status code because in case of an error
//      * Length will be 0. And that is all we need.
//      */
//     SampGetObjectAttribute(UserObject,
//                            L"Groups",
//                            NULL,
//                            NULL,
//                            &Length);
//
//     /* If there is no Groups attribute, return a groups buffer without an array */
//     if (Length == 0)
//     {
//         GroupsBuffer->MembershipCount = 0;
//         GroupsBuffer->Groups = NULL;
//
//         *Groups = GroupsBuffer;
//
//         Status = STATUS_SUCCESS;
//         goto done;
//     }
//
//     /* Allocate a buffer for the Groups attribute */
//     GroupsBuffer->Groups = midl_user_allocate(Length);
//     if (GroupsBuffer->Groups == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Retrieve the Grous attribute */
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     NULL,
//                                     GroupsBuffer->Groups,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttribute failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Calculate the membership count */
//     GroupsBuffer->MembershipCount = Length / sizeof(GROUP_MEMBERSHIP);
//
//     /* Return the groups buffer to the caller */
//     *Groups = GroupsBuffer;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (GroupsBuffer != NULL)
//         {
//             if (GroupsBuffer->Groups != NULL)
//                 midl_user_free(GroupsBuffer->Groups);
//
//             midl_user_free(GroupsBuffer);
//         }
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 40 */
// NTSTATUS
// NTAPI
// SamrQueryDisplayInformation(IN SAMPR_HANDLE DomainHandle,
//                             IN DOMAIN_DISPLAY_INFORMATION DisplayInformationClass,
//                             IN unsigned long Index,
//                             IN unsigned long EntryCount,
//                             IN unsigned long PreferredMaximumLength,
//                             OUT unsigned long *TotalAvailable,
//                             OUT unsigned long *TotalReturned,
//                             OUT PSAMPR_DISPLAY_INFO_BUFFER Buffer)
// {
//     TRACE("SamrQueryDisplayInformation(%p %lu %lu %lu %lu %p %p %p)\n",
//           DomainHandle, DisplayInformationClass, Index,
//           EntryCount, PreferredMaximumLength, TotalAvailable,
//           TotalReturned, Buffer);
//
//     return SamrQueryDisplayInformation3(DomainHandle,
//                                         DisplayInformationClass,
//                                         Index,
//                                         EntryCount,
//                                         PreferredMaximumLength,
//                                         TotalAvailable,
//                                         TotalReturned,
//                                         Buffer);
// }
//
//
// /* Function 41 */
// NTSTATUS
// NTAPI
// SamrGetDisplayEnumerationIndex(IN SAMPR_HANDLE DomainHandle,
//                                IN DOMAIN_DISPLAY_INFORMATION DisplayInformationClass,
//                                IN PRPC_UNICODE_STRING Prefix,
//                                OUT unsigned long *Index)
// {
//     TRACE("SamrGetDisplayEnumerationIndex(%p %lu %p %p)\n",
//           DomainHandle, DisplayInformationClass, Prefix, Index);
//
//     return SamrGetDisplayEnumerationIndex2(DomainHandle,
//                                            DisplayInformationClass,
//                                            Prefix,
//                                            Index);
// }
//
//
// /* Function 42 */
// NTSTATUS
// NTAPI
// SamrTestPrivateFunctionsDomain(IN SAMPR_HANDLE DomainHandle)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 43 */
// NTSTATUS
// NTAPI
// SamrTestPrivateFunctionsUser(IN SAMPR_HANDLE UserHandle)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 44 */
// NTSTATUS
// NTAPI
// SamrGetUserDomainPasswordInformation(IN SAMPR_HANDLE UserHandle,
//                                      OUT PUSER_DOMAIN_PASSWORD_INFORMATION PasswordInformation)
// {
//     SAM_DOMAIN_FIXED_DATA DomainFixedData;
//     SAM_USER_FIXED_DATA UserFixedData;
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT UserObject;
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     TRACE("SamrGetUserDomainPasswordInformation(%p %p)\n",
//           UserHandle, PasswordInformation);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the user handle */
//     Status = SampValidateDbObject(UserHandle,
//                                   SamDbUserObject,
//                                   0,
//                                   &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Validate the domain object */
//     Status = SampValidateDbObject((SAMPR_HANDLE)UserObject->ParentObject,
//                                   SamDbDomainObject,
//                                   DOMAIN_READ_PASSWORD_PARAMETERS,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Get fixed user data */
//     Length = sizeof(SAM_USER_FIXED_DATA);
//     Status = SampGetObjectAttribute(UserObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&UserFixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttribute failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if ((UserObject->RelativeId == DOMAIN_USER_RID_KRBTGT) ||
//         (UserFixedData.UserAccountControl & (USER_INTERDOMAIN_TRUST_ACCOUNT |
//                                              USER_WORKSTATION_TRUST_ACCOUNT |
//                                              USER_SERVER_TRUST_ACCOUNT)))
//     {
//         PasswordInformation->MinPasswordLength = 0;
//         PasswordInformation->PasswordProperties = 0;
//     }
//     else
//     {
//         /* Get fixed domain data */
//         Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//         Status = SampGetObjectAttribute(DomainObject,
//                                         L"F",
//                                         NULL,
//                                         (PVOID)&DomainFixedData,
//                                         &Length);
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("SampGetObjectAttribute failed with status 0x%08lx\n", Status);
//             goto done;
//         }
//
//         PasswordInformation->MinPasswordLength = DomainFixedData.MinPasswordLength;
//         PasswordInformation->PasswordProperties = DomainFixedData.PasswordProperties;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return STATUS_SUCCESS;
// }
//
//
// /* Function 45 */
// NTSTATUS
// NTAPI
// SamrRemoveMemberFromForeignDomain(IN SAMPR_HANDLE DomainHandle,
//                                   IN PRPC_SID MemberSid)
// {
//     PSAM_DB_OBJECT DomainObject;
//     ULONG Rid = 0;
//     NTSTATUS Status;
//
//     TRACE("SamrRemoveMemberFromForeignDomain(%p %p)\n",
//           DomainHandle, MemberSid);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the domain object */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LOOKUP,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampValidateDbObject failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Retrieve the RID from the MemberSID */
//     Status = SampGetRidFromSid((PSID)MemberSid,
//                                &Rid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetRidFromSid failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Fail, if the RID represents a special account */
//     if (Rid < 1000)
//     {
//         TRACE("Cannot remove a special account (RID: %lu)\n", Rid);
//         Status = STATUS_SPECIAL_ACCOUNT;
//         goto done;
//     }
//
//     /* Remove the member from all aliases in the domain */
//     Status = SampRemoveMemberFromAllAliases(DomainObject,
//                                             MemberSid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampRemoveMemberFromAllAliases failed with status 0x%08lx\n", Status);
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 46 */
// NTSTATUS
// NTAPI
// SamrQueryInformationDomain2(IN SAMPR_HANDLE DomainHandle,
//                             IN DOMAIN_INFORMATION_CLASS DomainInformationClass,
//                             OUT PSAMPR_DOMAIN_INFO_BUFFER *Buffer)
// {
//     PSAM_DB_OBJECT DomainObject;
//     ACCESS_MASK DesiredAccess;
//     NTSTATUS Status;
//
//     TRACE("SamrQueryInformationDomain2(%p %lu %p)\n",
//           DomainHandle, DomainInformationClass, Buffer);
//
//     switch (DomainInformationClass)
//     {
//         case DomainPasswordInformation:
//         case DomainLockoutInformation:
//             DesiredAccess = DOMAIN_READ_PASSWORD_PARAMETERS;
//             break;
//
//         case DomainGeneralInformation:
//         case DomainLogoffInformation:
//         case DomainOemInformation:
//         case DomainNameInformation:
//         case DomainReplicationInformation:
//         case DomainServerRoleInformation:
//         case DomainModifiedInformation:
//         case DomainStateInformation:
//         case DomainModifiedInformation2:
//             DesiredAccess = DOMAIN_READ_OTHER_PARAMETERS;
//             break;
//
//         case DomainGeneralInformation2:
//             DesiredAccess = DOMAIN_READ_PASSWORD_PARAMETERS |
//                             DOMAIN_READ_OTHER_PARAMETERS;
//             break;
//
//         default:
//             return STATUS_INVALID_INFO_CLASS;
//     }
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the server handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DesiredAccess,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     switch (DomainInformationClass)
//     {
//         case DomainPasswordInformation:
//             Status = SampQueryDomainPassword(DomainObject,
//                                              Buffer);
//             break;
//
//         case DomainGeneralInformation:
//             Status = SampQueryDomainGeneral(DomainObject,
//                                             Buffer);
//             break;
//
//         case DomainLogoffInformation:
//             Status = SampQueryDomainLogoff(DomainObject,
//                                            Buffer);
//             break;
//
//         case DomainOemInformation:
//             Status = SampQueryDomainOem(DomainObject,
//                                         Buffer);
//             break;
//
//         case DomainNameInformation:
//             Status = SampQueryDomainName(DomainObject,
//                                          Buffer);
//             break;
//
//         case DomainReplicationInformation:
//             Status = SampQueryDomainReplication(DomainObject,
//                                                 Buffer);
//             break;
//
//         case DomainServerRoleInformation:
//             Status = SampQueryDomainServerRole(DomainObject,
//                                                Buffer);
//             break;
//
//         case DomainModifiedInformation:
//             Status = SampQueryDomainModified(DomainObject,
//                                              Buffer);
//             break;
//
//         case DomainStateInformation:
//             Status = SampQueryDomainState(DomainObject,
//                                           Buffer);
//             break;
//
//         case DomainGeneralInformation2:
//             Status = SampQueryDomainGeneral2(DomainObject,
//                                              Buffer);
//             break;
//
//         case DomainLockoutInformation:
//             Status = SampQueryDomainLockout(DomainObject,
//                                             Buffer);
//             break;
//
//         case DomainModifiedInformation2:
//             Status = SampQueryDomainModified2(DomainObject,
//                                               Buffer);
//             break;
//
//         default:
//             Status = STATUS_NOT_IMPLEMENTED;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 47 */
// NTSTATUS
// NTAPI
// SamrQueryInformationUser2(IN SAMPR_HANDLE UserHandle,
//                           IN USER_INFORMATION_CLASS UserInformationClass,
//                           OUT PSAMPR_USER_INFO_BUFFER *Buffer)
// {
//     PSAM_DB_OBJECT UserObject;
//     ACCESS_MASK DesiredAccess;
//     NTSTATUS Status;
//
//     TRACE("SamrQueryInformationUser2(%p %lu %p)\n",
//           UserHandle, UserInformationClass, Buffer);
//
//     switch (UserInformationClass)
//     {
//         case UserGeneralInformation:
//         case UserNameInformation:
//         case UserAccountNameInformation:
//         case UserFullNameInformation:
//         case UserPrimaryGroupInformation:
//         case UserAdminCommentInformation:
//             DesiredAccess = USER_READ_GENERAL;
//             break;
//
//         case UserLogonHoursInformation:
//         case UserHomeInformation:
//         case UserScriptInformation:
//         case UserProfileInformation:
//         case UserWorkStationsInformation:
//             DesiredAccess = USER_READ_LOGON;
//             break;
//
//         case UserControlInformation:
//         case UserExpiresInformation:
//         case UserParametersInformation:
//             DesiredAccess = USER_READ_ACCOUNT;
//             break;
//
//         case UserPreferencesInformation:
//             DesiredAccess = USER_READ_GENERAL |
//                             USER_READ_PREFERENCES;
//             break;
//
//         case UserLogonInformation:
//         case UserAccountInformation:
//             DesiredAccess = USER_READ_GENERAL |
//                             USER_READ_PREFERENCES |
//                             USER_READ_LOGON |
//                             USER_READ_ACCOUNT;
//             break;
//
//         case UserInternal1Information:
//         case UserInternal2Information:
//         case UserAllInformation:
//             DesiredAccess = 0;
//             break;
//
//         default:
//             return STATUS_INVALID_INFO_CLASS;
//     }
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(UserHandle,
//                                   SamDbUserObject,
//                                   DesiredAccess,
//                                   &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     switch (UserInformationClass)
//     {
//         case UserGeneralInformation:
//             Status = SampQueryUserGeneral(UserObject,
//                                           Buffer);
//             break;
//
//         case UserPreferencesInformation:
//             Status = SampQueryUserPreferences(UserObject,
//                                               Buffer);
//             break;
//
//         case UserLogonInformation:
//             Status = SampQueryUserLogon(UserObject,
//                                         Buffer);
//             break;
//
//         case UserLogonHoursInformation:
//             Status = SampQueryUserLogonHours(UserObject,
//                                              Buffer);
//             break;
//
//         case UserAccountInformation:
//             Status = SampQueryUserAccount(UserObject,
//                                           Buffer);
//             break;
//
//         case UserNameInformation:
//             Status = SampQueryUserName(UserObject,
//                                        Buffer);
//             break;
//
//         case UserAccountNameInformation:
//             Status = SampQueryUserAccountName(UserObject,
//                                               Buffer);
//             break;
//
//         case UserFullNameInformation:
//             Status = SampQueryUserFullName(UserObject,
//                                            Buffer);
//             break;
//
//         case UserPrimaryGroupInformation:
//             Status = SampQueryUserPrimaryGroup(UserObject,
//                                                Buffer);
//             break;
//
//         case UserHomeInformation:
//             Status = SampQueryUserHome(UserObject,
//                                        Buffer);
//             break;
//
//         case UserScriptInformation:
//             Status = SampQueryUserScript(UserObject,
//                                          Buffer);
//             break;
//
//         case UserProfileInformation:
//             Status = SampQueryUserProfile(UserObject,
//                                           Buffer);
//             break;
//
//         case UserAdminCommentInformation:
//             Status = SampQueryUserAdminComment(UserObject,
//                                                Buffer);
//             break;
//
//         case UserWorkStationsInformation:
//             Status = SampQueryUserWorkStations(UserObject,
//                                                Buffer);
//             break;
//
//         case UserControlInformation:
//             Status = SampQueryUserControl(UserObject,
//                                           Buffer);
//             break;
//
//         case UserExpiresInformation:
//             Status = SampQueryUserExpires(UserObject,
//                                           Buffer);
//             break;
//
//         case UserInternal1Information:
//             Status = SampQueryUserInternal1(UserObject,
//                                             Buffer);
//             break;
//
//         case UserInternal2Information:
//             Status = SampQueryUserInternal2(UserObject,
//                                             Buffer);
//             break;
//
//         case UserParametersInformation:
//             Status = SampQueryUserParameters(UserObject,
//                                              Buffer);
//             break;
//
//         case UserAllInformation:
//             Status = SampQueryUserAll(UserObject,
//                                       Buffer);
//             break;
//
// //        case UserInternal4Information:
// //        case UserInternal5Information:
// //        case UserInternal4InformationNew:
// //        case UserInternal5InformationNew:
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 48 */
// NTSTATUS
// NTAPI
// SamrQueryDisplayInformation2(IN SAMPR_HANDLE DomainHandle,
//                              IN DOMAIN_DISPLAY_INFORMATION DisplayInformationClass,
//                              IN unsigned long Index,
//                              IN unsigned long EntryCount,
//                              IN unsigned long PreferredMaximumLength,
//                              OUT unsigned long *TotalAvailable,
//                              OUT unsigned long *TotalReturned,
//                              OUT PSAMPR_DISPLAY_INFO_BUFFER Buffer)
// {
//     TRACE("SamrQueryDisplayInformation2(%p %lu %lu %lu %lu %p %p %p)\n",
//           DomainHandle, DisplayInformationClass, Index,
//           EntryCount, PreferredMaximumLength, TotalAvailable,
//           TotalReturned, Buffer);
//
//     return SamrQueryDisplayInformation3(DomainHandle,
//                                         DisplayInformationClass,
//                                         Index,
//                                         EntryCount,
//                                         PreferredMaximumLength,
//                                         TotalAvailable,
//                                         TotalReturned,
//                                         Buffer);
// }
//
//
// /* Function 49 */
// NTSTATUS
// NTAPI
// SamrGetDisplayEnumerationIndex2(IN SAMPR_HANDLE DomainHandle,
//                                 IN DOMAIN_DISPLAY_INFORMATION DisplayInformationClass,
//                                 IN PRPC_UNICODE_STRING Prefix,
//                                 OUT unsigned long *Index)
// {
//     TRACE("SamrGetDisplayEnumerationIndex2(%p %lu %p %p)\n",
//           DomainHandle, DisplayInformationClass, Prefix, Index);
//
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 50 */
// NTSTATUS
// NTAPI
// SamrCreateUser2InDomain(IN SAMPR_HANDLE DomainHandle,
//                         IN PRPC_UNICODE_STRING Name,
//                         IN unsigned long AccountType,
//                         IN ACCESS_MASK DesiredAccess,
//                         OUT SAMPR_HANDLE *UserHandle,
//                         OUT unsigned long *GrantedAccess,
//                         OUT unsigned long *RelativeId)
// {
//     SAM_DOMAIN_FIXED_DATA FixedDomainData;
//     SAM_USER_FIXED_DATA FixedUserData;
//     PSAM_DB_OBJECT DomainObject;
//     PSAM_DB_OBJECT UserObject;
//     GROUP_MEMBERSHIP GroupMembership;
//     UCHAR LogonHours[23];
//     ULONG ulSize;
//     ULONG ulRid;
//     WCHAR szRid[9];
//     PSECURITY_DESCRIPTOR Sd = NULL;
//     ULONG SdSize = 0;
//     PSID UserSid = NULL;
//     NTSTATUS Status;
//
//     TRACE("SamrCreateUserInDomain(%p %p %lx %p %p)\n",
//           DomainHandle, Name, DesiredAccess, UserHandle, RelativeId);
//
//     if (Name == NULL ||
//         Name->Length == 0 ||
//         Name->Buffer == NULL ||
//         UserHandle == NULL ||
//         RelativeId == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Check for valid account type */
//     if (AccountType != USER_NORMAL_ACCOUNT &&
//         AccountType != USER_WORKSTATION_TRUST_ACCOUNT &&
//         AccountType != USER_INTERDOMAIN_TRUST_ACCOUNT &&
//         AccountType != USER_SERVER_TRUST_ACCOUNT &&
//         AccountType != USER_TEMP_DUPLICATE_ACCOUNT)
//         return STATUS_INVALID_PARAMETER;
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &UserMapping);
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_CREATE_USER,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Check the user account name */
//     Status = SampCheckAccountName(Name, 20);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCheckAccountName failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Check if the user name already exists in the domain */
//     Status = SampCheckAccountNameInDomain(DomainObject,
//                                           Name->Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("User name \'%S\' already exists in domain (Status 0x%08lx)\n",
//               Name->Buffer, Status);
//         goto done;
//     }
//
//     /* Get the fixed domain attributes */
//     ulSize = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     (PVOID)&FixedDomainData,
//                                     &ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Increment the NextRid attribute */
//     ulRid = FixedDomainData.NextRid;
//     FixedDomainData.NextRid++;
//
//     TRACE("RID: %lx\n", ulRid);
//
//     /* Create the user SID */
//     Status = SampCreateAccountSid(DomainObject,
//                                   ulRid,
//                                   &UserSid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCreateAccountSid failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Create the security descriptor */
//     Status = SampCreateUserSD(UserSid,
//                               &Sd,
//                               &SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampCreateUserSD failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Store the fixed domain attributes */
//     Status = SampSetObjectAttribute(DomainObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     &FixedDomainData,
//                                     ulSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Convert the RID into a string (hex) */
//     swprintf(szRid, L"%08lX", ulRid);
//
//     /* Create the user object */
//     Status = SampCreateDbObject(DomainObject,
//                                 L"Users",
//                                 szRid,
//                                 ulRid,
//                                 SamDbUserObject,
//                                 DesiredAccess,
//                                 &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Add the account name for the user object */
//     Status = SampSetAccountNameInDomain(DomainObject,
//                                         L"Users",
//                                         Name->Buffer,
//                                         ulRid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Initialize fixed user data */
//     FixedUserData.Version = 1;
//     FixedUserData.Reserved = 0;
//     FixedUserData.LastLogon.QuadPart = 0;
//     FixedUserData.LastLogoff.QuadPart = 0;
//     FixedUserData.PasswordLastSet.QuadPart = 0;
//     FixedUserData.AccountExpires.LowPart = MAXULONG;
//     FixedUserData.AccountExpires.HighPart = MAXLONG;
//     FixedUserData.LastBadPasswordTime.QuadPart = 0;
//     FixedUserData.UserId = ulRid;
//     FixedUserData.PrimaryGroupId = DOMAIN_GROUP_RID_USERS;
//     FixedUserData.UserAccountControl = USER_ACCOUNT_DISABLED |
//                                        USER_PASSWORD_NOT_REQUIRED |
//                                        AccountType;
//     FixedUserData.CountryCode = 0;
//     FixedUserData.CodePage = 0;
//     FixedUserData.BadPasswordCount = 0;
//     FixedUserData.LogonCount = 0;
//     FixedUserData.AdminCount = 0;
//     FixedUserData.OperatorCount = 0;
//
//     /* Set fixed user data attribute */
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"F",
//                                     REG_BINARY,
//                                     (LPVOID)&FixedUserData,
//                                     sizeof(SAM_USER_FIXED_DATA));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Name attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"Name",
//                                           Name);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the FullName attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"FullName",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the HomeDirectory attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"HomeDirectory",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the HomeDirectoryDrive attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"HomeDirectoryDrive",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the ScriptPath attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"ScriptPath",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the ProfilePath attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"ProfilePath",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the AdminComment attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"AdminComment",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the UserComment attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"UserComment",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the WorkStations attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"WorkStations",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the Parameters attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"Parameters",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set LogonHours attribute*/
//     *((PUSHORT)LogonHours) = 168;
//     memset(&(LogonHours[2]), 0xff, 21);
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LogonHours",
//                                     REG_BINARY,
//                                     &LogonHours,
//                                     sizeof(LogonHours));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set Groups attribute*/
//     GroupMembership.RelativeId = DOMAIN_GROUP_RID_USERS;
//     GroupMembership.Attributes = SE_GROUP_MANDATORY |
//                                  SE_GROUP_ENABLED |
//                                  SE_GROUP_ENABLED_BY_DEFAULT;
//
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"Groups",
//                                     REG_BINARY,
//                                     &GroupMembership,
//                                     sizeof(GROUP_MEMBERSHIP));
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set LMPwd attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LMPwd",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set NTPwd attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"NTPwd",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set LMPwdHistory attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"LMPwdHistory",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set NTPwdHistory attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"NTPwdHistory",
//                                     REG_BINARY,
//                                     NULL,
//                                     0);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the PrivateData attribute */
//     Status = SampSetObjectAttributeString(UserObject,
//                                           L"PrivateData",
//                                           NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     /* Set the SecDesc attribute*/
//     Status = SampSetObjectAttribute(UserObject,
//                                     L"SecDesc",
//                                     REG_BINARY,
//                                     Sd,
//                                     SdSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *UserHandle = (SAMPR_HANDLE)UserObject;
//         *RelativeId = ulRid;
//         *GrantedAccess = UserObject->Access;
//     }
//
// done:
//     if (Sd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Sd);
//
//     if (UserSid != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, UserSid);
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("returns with status 0x%08lx\n", Status);
//
//     return Status;
// }
//
//
// /* Function 51 */
// NTSTATUS
// NTAPI
// SamrQueryDisplayInformation3(IN SAMPR_HANDLE DomainHandle,
//                              IN DOMAIN_DISPLAY_INFORMATION DisplayInformationClass,
//                              IN unsigned long Index,
//                              IN unsigned long EntryCount,
//                              IN unsigned long PreferredMaximumLength,
//                              OUT unsigned long *TotalAvailable,
//                              OUT unsigned long *TotalReturned,
//                              OUT PSAMPR_DISPLAY_INFO_BUFFER Buffer)
// {
//     PSAM_DB_OBJECT DomainObject;
//     NTSTATUS Status;
//
//     FIXME("SamrQueryDisplayInformation3(%p %lu %lu %lu %lu %p %p %p)\n",
//           DomainHandle, DisplayInformationClass, Index,
//           EntryCount, PreferredMaximumLength, TotalAvailable,
//           TotalReturned, Buffer);
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(DomainHandle,
//                                   SamDbDomainObject,
//                                   DOMAIN_LIST_ACCOUNTS,
//                                   &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampValidateDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampFillDisplayCache(DomainObject,
//                                   DisplayInformationClass);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("SampFillDisplayCache() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     TRACE("returns with status 0x%08lx\n", Status);
//
//     RtlReleaseResource(&SampResource);
//
// //    return Status;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 52 */
// NTSTATUS
// NTAPI
// SamrAddMultipleMembersToAlias(IN SAMPR_HANDLE AliasHandle,
//                               IN PSAMPR_PSID_ARRAY MembersBuffer)
// {
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("SamrAddMultipleMembersToAlias(%p %p)\n",
//           AliasHandle, MembersBuffer);
//
//     for (i = 0; i < MembersBuffer->Count; i++)
//     {
//         Status = SamrAddMemberToAlias(AliasHandle,
//                                       ((PSID *)MembersBuffer->Sids)[i]);
//
//         if (Status == STATUS_MEMBER_IN_ALIAS)
//             Status = STATUS_SUCCESS;
//
//         if (!NT_SUCCESS(Status))
//             break;
//     }
//
//     return Status;
// }
//
//
// /* Function 53 */
// NTSTATUS
// NTAPI
// SamrRemoveMultipleMembersFromAlias(IN SAMPR_HANDLE AliasHandle,
//                                    IN PSAMPR_PSID_ARRAY MembersBuffer)
// {
//     ULONG i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("SamrRemoveMultipleMembersFromAlias(%p %p)\n",
//           AliasHandle, MembersBuffer);
//
//     for (i = 0; i < MembersBuffer->Count; i++)
//     {
//         Status = SamrRemoveMemberFromAlias(AliasHandle,
//                                            ((PSID *)MembersBuffer->Sids)[i]);
//
//         if (Status == STATUS_MEMBER_IN_ALIAS)
//             Status = STATUS_SUCCESS;
//
//         if (!NT_SUCCESS(Status))
//             break;
//     }
//
//     return Status;
// }
//
//
// /* Function 54 */
// NTSTATUS
// NTAPI
// SamrOemChangePasswordUser2(IN handle_t BindingHandle,
//                            IN PRPC_STRING ServerName,
//                            IN PRPC_STRING UserName,
//                            IN PSAMPR_ENCRYPTED_USER_PASSWORD NewPasswordEncryptedWithOldLm,
//                            IN PENCRYPTED_LM_OWF_PASSWORD OldLmOwfPasswordEncryptedWithNewLm)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* Function 55 */
// NTSTATUS
// NTAPI
// SamrUnicodeChangePasswordUser2(IN handle_t BindingHandle,
//                                IN PRPC_UNICODE_STRING ServerName,
//                                IN PRPC_UNICODE_STRING UserName,
//                                IN PSAMPR_ENCRYPTED_USER_PASSWORD NewPasswordEncryptedWithOldNt,
//                                IN PENCRYPTED_NT_OWF_PASSWORD OldNtOwfPasswordEncryptedWithNewNt,
//                                IN unsigned char LmPresent,
//                                IN PSAMPR_ENCRYPTED_USER_PASSWORD NewPasswordEncryptedWithOldLm,
//                                IN PENCRYPTED_LM_OWF_PASSWORD OldLmOwfPasswordEncryptedWithNewNt)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 56 */
// NTSTATUS
// NTAPI
// SamrGetDomainPasswordInformation(IN handle_t BindingHandle,
//                                  IN PRPC_UNICODE_STRING Unused,
//                                  OUT PUSER_DOMAIN_PASSWORD_INFORMATION PasswordInformation)
// {
//     SAMPR_HANDLE ServerHandle = NULL;
//     PSAM_DB_OBJECT DomainObject = NULL;
//     SAM_DOMAIN_FIXED_DATA FixedData;
//     ULONG Length;
//     NTSTATUS Status;
//
//     TRACE("SamrGetDomainPasswordInformation(%p %p %p)\n",
//           BindingHandle, Unused, PasswordInformation);
//
//     Status = SamrConnect(NULL,
//                          &ServerHandle,
//                          SAM_SERVER_LOOKUP_DOMAIN);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SamrConnect() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Status = SampOpenDbObject((PSAM_DB_OBJECT)ServerHandle,
//                               L"Domains",
//                               L"Account",
//                               0,
//                               SamDbDomainObject,
//                               DOMAIN_READ_PASSWORD_PARAMETERS,
//                               &DomainObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampOpenDbObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Length = sizeof(SAM_DOMAIN_FIXED_DATA);
//     Status = SampGetObjectAttribute(DomainObject,
//                                     L"F",
//                                     NULL,
//                                     &FixedData,
//                                     &Length);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("SampGetObjectAttribute() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     PasswordInformation->MinPasswordLength = FixedData.MinPasswordLength;
//     PasswordInformation->PasswordProperties = FixedData.PasswordProperties;
//
// done:
//     if (DomainObject != NULL)
//         SampCloseDbObject(DomainObject);
//
//     if (ServerHandle != NULL)
//         SamrCloseHandle(&ServerHandle);
//
//     return Status;
// }
//
//
// /* Function 57 */
// NTSTATUS
// NTAPI
// SamrConnect2(IN PSAMPR_SERVER_NAME ServerName,
//              OUT SAMPR_HANDLE *ServerHandle,
//              IN ACCESS_MASK DesiredAccess)
// {
//     SAMPR_REVISION_INFO InRevisionInfo, OutRevisionInfo;
//     ULONG OutVersion;
//
//     TRACE("SamrConnect2(%p %p %lx)\n",
//           ServerName, ServerHandle, DesiredAccess);
//
//     InRevisionInfo.V1.Revision = 1;
//     InRevisionInfo.V1.SupportedFeatures = 0;
//
//     return SamrConnect5(ServerName,
//                         DesiredAccess,
//                         1,
//                         &InRevisionInfo,
//                         &OutVersion,
//                         &OutRevisionInfo,
//                         ServerHandle);
// }
//
//
// /* Function 58 */
// NTSTATUS
// NTAPI
// SamrSetInformationUser2(IN SAMPR_HANDLE UserHandle,
//                         IN USER_INFORMATION_CLASS UserInformationClass,
//                         IN PSAMPR_USER_INFO_BUFFER Buffer)
// {
//     PSAM_DB_OBJECT UserObject;
//     ACCESS_MASK DesiredAccess;
//     NTSTATUS Status;
//
//     TRACE("SamrSetInformationUser2(%p %lu %p)\n",
//           UserHandle, UserInformationClass, Buffer);
//
//     switch (UserInformationClass)
//     {
//         case UserLogonHoursInformation:
//         case UserNameInformation:
//         case UserAccountNameInformation:
//         case UserFullNameInformation:
//         case UserPrimaryGroupInformation:
//         case UserHomeInformation:
//         case UserScriptInformation:
//         case UserProfileInformation:
//         case UserAdminCommentInformation:
//         case UserWorkStationsInformation:
//         case UserControlInformation:
//         case UserExpiresInformation:
//         case UserParametersInformation:
//             DesiredAccess = USER_WRITE_ACCOUNT;
//             break;
//
//         case UserGeneralInformation:
//             DesiredAccess = USER_WRITE_ACCOUNT |
//                             USER_WRITE_PREFERENCES;
//             break;
//
//         case UserPreferencesInformation:
//             DesiredAccess = USER_WRITE_PREFERENCES;
//             break;
//
//         case UserSetPasswordInformation:
//         case UserInternal1Information:
//             DesiredAccess = USER_FORCE_PASSWORD_CHANGE;
//             break;
//
//         case UserAllInformation:
//         case UserInternal2Information:
//             DesiredAccess = 0; /* FIXME */
//             break;
//
//         default:
//             return STATUS_INVALID_INFO_CLASS;
//     }
//
//     RtlAcquireResourceExclusive(&SampResource,
//                                 TRUE);
//
//     /* Validate the domain handle */
//     Status = SampValidateDbObject(UserHandle,
//                                   SamDbUserObject,
//                                   DesiredAccess,
//                                   &UserObject);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("failed with status 0x%08lx\n", Status);
//         goto done;
//     }
//
//     switch (UserInformationClass)
//     {
//         case UserGeneralInformation:
//             Status = SampSetUserGeneral(UserObject,
//                                         Buffer);
//             break;
//
//         case UserPreferencesInformation:
//             Status = SampSetUserPreferences(UserObject,
//                                             Buffer);
//             break;
//
//         case UserLogonHoursInformation:
//             Status = SampSetLogonHoursAttribute(UserObject,
//                                                &Buffer->LogonHours.LogonHours);
//             break;
//
//         case UserNameInformation:
//             Status = SampSetUserName(UserObject,
//                                      &Buffer->Name.UserName);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"FullName",
//                                                   &Buffer->Name.FullName);
//             break;
//
//         case UserAccountNameInformation:
//             Status = SampSetUserName(UserObject,
//                                      &Buffer->AccountName.UserName);
//             break;
//
//         case UserFullNameInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"FullName",
//                                                   &Buffer->FullName.FullName);
//             break;
//
//         case UserPrimaryGroupInformation:
//             Status = SampSetUserPrimaryGroup(UserObject,
//                                              Buffer);
//             break;
//
//         case UserHomeInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"HomeDirectory",
//                                                   &Buffer->Home.HomeDirectory);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"HomeDirectoryDrive",
//                                                   &Buffer->Home.HomeDirectoryDrive);
//             break;
//
//         case UserScriptInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"ScriptPath",
//                                                   &Buffer->Script.ScriptPath);
//             break;
//
//         case UserProfileInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"ProfilePath",
//                                                   &Buffer->Profile.ProfilePath);
//             break;
//
//         case UserAdminCommentInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"AdminComment",
//                                                   &Buffer->AdminComment.AdminComment);
//             break;
//
//         case UserWorkStationsInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"WorkStations",
//                                                   &Buffer->WorkStations.WorkStations);
//             break;
//
//         case UserSetPasswordInformation:
//             TRACE("Password: %S\n", Buffer->SetPassword.Password.Buffer);
//             TRACE("PasswordExpired: %d\n", Buffer->SetPassword.PasswordExpired);
//
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"Password",
//                                                   &Buffer->SetPassword.Password);
//             break;
//
//         case UserControlInformation:
//             Status = SampSetUserControl(UserObject,
//                                         Buffer);
//             break;
//
//         case UserExpiresInformation:
//             Status = SampSetUserExpires(UserObject,
//                                         Buffer);
//             break;
//
//         case UserInternal1Information:
//             Status = SampSetUserInternal1(UserObject,
//                                           Buffer);
//             break;
//
//         case UserInternal2Information:
//             Status = SampSetUserInternal2(UserObject,
//                                           Buffer);
//             break;
//
//         case UserParametersInformation:
//             Status = SampSetObjectAttributeString(UserObject,
//                                                   L"Parameters",
//                                                   &Buffer->Parameters.Parameters);
//             break;
//
//         case UserAllInformation:
//             Status = SampSetUserAll(UserObject,
//                                     Buffer);
//             break;
//
// //        case UserInternal4Information:
// //        case UserInternal5Information:
// //        case UserInternal4InformationNew:
// //        case UserInternal5InformationNew:
//
//         default:
//             Status = STATUS_INVALID_INFO_CLASS;
//     }
//
// done:
//     RtlReleaseResource(&SampResource);
//
//     return Status;
// }
//
//
// /* Function 59 */
// NTSTATUS
// NTAPI
// SamrSetBootKeyInformation(IN handle_t BindingHandle) /* FIXME */
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* Function 60 */
// NTSTATUS
// NTAPI
// SamrGetBootKeyInformation(IN handle_t BindingHandle) /* FIXME */
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* Function 61 */
// NTSTATUS
// NTAPI
// SamrConnect3(IN handle_t BindingHandle) /* FIXME */
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 62 */
// NTSTATUS
// NTAPI
// SamrConnect4(IN PSAMPR_SERVER_NAME ServerName,
//              OUT SAMPR_HANDLE *ServerHandle,
//              IN unsigned long ClientRevision,
//              IN ACCESS_MASK DesiredAccess)
// {
//     SAMPR_REVISION_INFO InRevisionInfo, OutRevisionInfo;
//     ULONG OutVersion;
//
//     TRACE("SamrConnect4(%p %p %lu 0x%lx)\n",
//           ServerName, ServerHandle, ClientRevision, DesiredAccess);
//
//     InRevisionInfo.V1.Revision = 2;
//     InRevisionInfo.V1.SupportedFeatures = 0;
//
//     return SamrConnect5(ServerName,
//                         DesiredAccess,
//                         1,
//                         &InRevisionInfo,
//                         &OutVersion,
//                         &OutRevisionInfo,
//                         ServerHandle);
// }
//
//
// /* Function 63 */
// NTSTATUS
// NTAPI
// SamrUnicodeChangePasswordUser3(IN handle_t BindingHandle) /* FIXME */
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* Function 64 */
// NTSTATUS
// NTAPI
// SamrConnect5(IN PSAMPR_SERVER_NAME ServerName,
//              IN ACCESS_MASK DesiredAccess,
//              IN unsigned long InVersion,
//              IN SAMPR_REVISION_INFO *InRevisionInfo,
//              OUT unsigned long *OutVersion,
//              OUT SAMPR_REVISION_INFO *OutRevisionInfo,
//              OUT SAMPR_HANDLE *ServerHandle)
// {
//     PSAM_DB_OBJECT ServerObject;
//     NTSTATUS Status;
//
//     TRACE("SamrConnect5(%p 0x%lx %lu %p %p %p %p)\n",
//           ServerName, DesiredAccess, InVersion, InRevisionInfo,
//           OutVersion, OutRevisionInfo, ServerHandle);
//
//     if (InVersion != 1)
//         return STATUS_NOT_SUPPORTED;
//
//     RtlAcquireResourceShared(&SampResource,
//                              TRUE);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       &ServerMapping);
//
//     /* Open the Server Object */
//     Status = SampOpenDbObject(NULL,
//                               NULL,
//                               L"SAM",
//                               0,
//                               SamDbServerObject,
//                               DesiredAccess,
//                               &ServerObject);
//     if (NT_SUCCESS(Status))
//     {
//         *OutVersion = 1;
//
//         OutRevisionInfo->V1.Revision = 3;
//         OutRevisionInfo->V1.SupportedFeatures = 0;
//
//         *ServerHandle = (SAMPR_HANDLE)ServerObject;
//     }
//
//     RtlReleaseResource(&SampResource);
//
//     TRACE("SamrConnect5 done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /* Function 65 */
// NTSTATUS
// NTAPI
// SamrRidToSid(IN SAMPR_HANDLE ObjectHandle,
//              IN unsigned long Rid,
//              OUT PRPC_SID *Sid)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* Function 66 */
// NTSTATUS
// NTAPI
// SamrSetDSRMPassword(IN handle_t BindingHandle,
//                     IN PRPC_UNICODE_STRING Unused,
//                     IN unsigned long UserId,
//                     IN PENCRYPTED_NT_OWF_PASSWORD EncryptedNtOwfPassword)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* Function 67 */
// NTSTATUS
// NTAPI
// SamrValidatePassword(IN handle_t Handle,
//                      IN PASSWORD_POLICY_VALIDATION_TYPE ValidationType,
//                      IN PSAM_VALIDATE_INPUT_ARG InputArg,
//                      OUT PSAM_VALIDATE_OUTPUT_ARG *OutputArg)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
