use crate::prelude::*;
use core::mem::size_of;

// Cross-module siblings (same crate) — import via path so the ABI is whatever
// the definition declares (extern "C"); declaring them extern here would risk an
// i386 stdcall/cdecl decoration mismatch.
use crate::lsasrv::registry::{LsapRegDeleteSubKey, LsapRegDeleteKey, LsapRegEnumerateSubKey};
use crate::lsasrv::security::LsapCreatePolicySd;

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


// Genuinely external imports (advapi32/kernel32/ntdll; WINAPI/NTAPI = stdcall on
// i386 → extern "system" resolves per-arch).
extern "system" {
    fn ConvertSidToStringSidW(Sid: PSID, StringSid: *mut *mut u16) -> i32;
    fn LocalFree(hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn RtlpNtOpenKey(KeyHandle: *mut HANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES, Unused: ULONG) -> NTSTATUS;
    fn RtlpNtQueryValueKey(KeyHandle: HANDLE, KeyValueType: *mut ULONG, KeyValue: *mut core::ffi::c_void, KeyValueLength: *mut ULONG, Unused: ULONG) -> NTSTATUS;
    fn RtlpNtSetValueKey(KeyHandle: HANDLE, KeyValueType: ULONG, KeyValue: *mut core::ffi::c_void, KeyValueLength: ULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// File-private in C (`static HANDLE SecurityKeyHandle`) — NOT exported.
pub static mut SecurityKeyHandle: HANDLE = core::ptr::null_mut();
// Cross-file globals (declared `extern` in lsasrv.h) consumed by the still-C TUs
// (policy.c, authpackage.c, lookup.c) — must export unmangled, hence #[no_mangle].
#[no_mangle]
pub static mut NullSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 0] };
#[no_mangle]
pub static mut WorldSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 1] };
#[no_mangle]
pub static mut LocalSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 2] };
#[no_mangle]
pub static mut CreatorSidAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 3] };
#[no_mangle]
pub static mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0, 0, 0, 0, 0, 5] };
#[no_mangle]
pub static mut BuiltinDomainSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut AccountDomainSid: PSID = core::ptr::null_mut();
#[no_mangle]
pub static mut BuiltinDomainName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
#[no_mangle]
pub static mut AccountDomainName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };

unsafe fn LsapOpenServiceKey() -> NTSTATUS {
    let mut key_name: _UNICODE_STRING = core::mem::zeroed();
    // L"\\Registry\\Machine\\SECURITY"
    const WIDE_STR: [u16; 27] = [
        0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72,
        0x79, 0x5C, 0x4D, 0x61, 0x63, 0x68, 0x69, 0x6E,
        0x65, 0x5C, 0x53, 0x45, 0x43, 0x55, 0x52, 0x49,
        0x54, 0x59, 0x00,
    ];
    RtlInitUnicodeString(&raw mut key_name, WIDE_STR.as_ptr() as *const u16);

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = core::ptr::null_mut();
    object_attributes.ObjectName = &key_name as *const _ as *mut _; // PUNICODE_STRING
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    let status = RtlpNtOpenKey(
        &raw mut SecurityKeyHandle,
        KEY_READ | KEY_CREATE_SUB_KEY | KEY_ENUMERATE_SUB_KEYS,
        &raw mut object_attributes,
        0,
    );

    status
}

unsafe fn LsapIsDatabaseInstalled() -> u8 {
    let mut key_name: _UNICODE_STRING = core::mem::zeroed();
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut key_handle: HANDLE = core::ptr::null_mut();
    let status: NTSTATUS;

    let key_name_buf: [u16; 7] = [0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0079, 0x0000];
    RtlInitUnicodeString(&mut key_name, key_name_buf.as_ptr());

    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = SecurityKeyHandle;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
    object_attributes.ObjectName = &mut key_name;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    status = RtlpNtOpenKey(&mut key_handle, KEY_READ, &mut object_attributes, 0);
    if !NT_SUCCESS(status) {
        return 0u8; // FALSE
    }

    NtClose(key_handle);

    1u8 // TRUE
}

unsafe fn LsapCreateDatabaseKeys() -> NTSTATUS {
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut key_name: _UNICODE_STRING = core::mem::zeroed();
    let mut policy_key_handle: HANDLE = core::ptr::null_mut();
    let mut accounts_key_handle: HANDLE = core::ptr::null_mut();
    let mut domains_key_handle: HANDLE = core::ptr::null_mut();
    let mut secrets_key_handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    TRACE!(b"LsapInstallDatabase()\n\0".as_ptr() as *const u8);

    let p_oa = core::ptr::addr_of_mut!(object_attributes);

    'done: loop {
        // Create 'Policy' key
        let policy_name: &[u16] = &[0x50, 0x6f, 0x6c, 0x69, 0x63, 0x79, 0]; // "Policy"
        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), policy_name.as_ptr());

        (*p_oa).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*p_oa).RootDirectory = SecurityKeyHandle;
        (*p_oa).Attributes = OBJ_CASE_INSENSITIVE;
        (*p_oa).ObjectName = core::ptr::addr_of_mut!(key_name);
        (*p_oa).SecurityDescriptor = core::ptr::null_mut();
        (*p_oa).SecurityQualityOfService = core::ptr::null_mut();

        status = NtCreateKey(
            core::ptr::addr_of_mut!(policy_key_handle),
            KEY_ALL_ACCESS,
            p_oa,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            ERR!("Failed to create the 'Policy' key (Status: 0x%08lx)\n", status);
            break 'done;
        }

        // Create 'Accounts' key
        let accounts_name: &[u16] = &[0x41, 0x63, 0x63, 0x6f, 0x75, 0x6e, 0x74, 0x73, 0]; // "Accounts"
        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), accounts_name.as_ptr());

        (*p_oa).RootDirectory = policy_key_handle;
        status = NtCreateKey(
            core::ptr::addr_of_mut!(accounts_key_handle),
            KEY_ALL_ACCESS,
            p_oa,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            ERR!("Failed to create the 'Accounts' key (Status: 0x%08lx)\n", status);
            break 'done;
        }

        // Create 'Domains' key
        let domains_name: &[u16] = &[0x44, 0x6f, 0x6d, 0x61, 0x69, 0x6e, 0x73, 0]; // "Domains"
        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), domains_name.as_ptr());

        (*p_oa).RootDirectory = policy_key_handle;
        status = NtCreateKey(
            core::ptr::addr_of_mut!(domains_key_handle),
            KEY_ALL_ACCESS,
            p_oa,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            ERR!("Failed to create the 'Domains' key (Status: 0x%08lx)\n", status);
            break 'done;
        }

        // Create 'Secrets' key
        let secrets_name: &[u16] = &[0x53, 0x65, 0x63, 0x72, 0x65, 0x74, 0x73, 0]; // "Secrets"
        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), secrets_name.as_ptr());

        (*p_oa).RootDirectory = policy_key_handle;
        status = NtCreateKey(
            core::ptr::addr_of_mut!(secrets_key_handle),
            KEY_ALL_ACCESS,
            p_oa,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            ERR!("Failed to create the 'Secrets' key (Status: 0x%08lx)\n", status);
            break 'done;
        }

        break; // All keys created successfully
    }

    // Cleanup
    if secrets_key_handle != core::ptr::null_mut() {
        NtClose(secrets_key_handle);
    }
    if domains_key_handle != core::ptr::null_mut() {
        NtClose(domains_key_handle);
    }
    if accounts_key_handle != core::ptr::null_mut() {
        NtClose(accounts_key_handle);
    }
    if policy_key_handle != core::ptr::null_mut() {
        NtClose(policy_key_handle);
    }

    TRACE!(b"LsapInstallDatabase() done (Status: 0x%08lx)\n\0".as_ptr() as *const u8, status);

    status
}

unsafe fn LsapCreateRandomDomainSid(sid: *mut PSID) -> NTSTATUS {
    let mut system_time: _LARGE_INTEGER = core::mem::zeroed();
    NtQuerySystemTime(core::ptr::addr_of_mut!(system_time));
    let seed_ptr: *mut u32 = core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(system_time)).__bindgen_anon_1.LowPart);

    RtlAllocateAndInitializeSid(
        core::ptr::addr_of_mut!(NtAuthority),
        4u8,
        SECURITY_NT_NON_UNIQUE,
        RtlUniform(seed_ptr),
        RtlUniform(seed_ptr),
        RtlUniform(seed_ptr),
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        SECURITY_NULL_RID,
        sid,
    )
}

unsafe fn LsapCreateDatabaseObjects() -> NTSTATUS {
    let AuditEventsInfo: *mut _LSAP_POLICY_AUDIT_EVENTS_DATA;
    let mut QuotaInfo: _POLICY_DEFAULT_QUOTA_INFO = core::mem::zeroed();
    let mut ModificationInfo: _POLICY_MODIFICATION_INFO = core::mem::zeroed();
    let AuditFullInfo: _POLICY_AUDIT_FULL_QUERY_INFO = _POLICY_AUDIT_FULL_QUERY_INFO {
        ShutDownOnFull: 0u8,
        LogIsFull: 0u8,
    };
    let mut AuditLogInfo: _POLICY_AUDIT_LOG_INFO = core::mem::zeroed();
    let mut DnsDomainGuid: GUID = core::mem::zeroed();
    let mut PolicyObject: *mut _LSA_DB_OBJECT = core::ptr::null_mut();
    // Local shadows the global `AccountDomainSid` in C; renamed (Rust `let`
    // cannot shadow a static).
    let mut LocalAccountDomainSid: PSID = core::ptr::null_mut();
    let mut PolicySd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut PolicySdSize: u32 = 0;
    let mut i: u32;
    let mut Status: NTSTATUS;

    QuotaInfo.QuotaLimits.PagedPoolLimit = 0x2000000;
    QuotaInfo.QuotaLimits.NonPagedPoolLimit = 0x100000;
    QuotaInfo.QuotaLimits.MinimumWorkingSetSize = 0x10000;
    QuotaInfo.QuotaLimits.MaximumWorkingSetSize = 0xF000000;
    QuotaInfo.QuotaLimits.PagefileLimit = 0;
    (*((&mut QuotaInfo.QuotaLimits.TimeLimit) as *mut _ as *mut _LARGE_INTEGER)).QuadPart = 0;

    AuditLogInfo.AuditLogPercentFull = 0;
    AuditLogInfo.MaximumLogSize = 0;
    (*((&mut AuditLogInfo.AuditRetentionPeriod) as *mut _ as *mut _LARGE_INTEGER)).QuadPart = 0;
    AuditLogInfo.AuditLogFullShutdownInProgress = 0;
    (*((&mut AuditLogInfo.TimeToShutdown) as *mut _ as *mut _LARGE_INTEGER)).QuadPart = 0;
    AuditLogInfo.NextAuditRecordId = 0;

    AuditEventsInfo = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<_LSAP_POLICY_AUDIT_EVENTS_DATA>() as SIZE_T,
    ) as *mut _LSAP_POLICY_AUDIT_EVENTS_DATA;
    if AuditEventsInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*AuditEventsInfo).AuditingMode = 0u8;
    (*AuditEventsInfo).MaximumAuditEventCount = POLICY_AUDIT_EVENT_TYPE_COUNT;
    i = 0;
    while i < POLICY_AUDIT_EVENT_TYPE_COUNT {
        (*AuditEventsInfo).AuditEvents[i as usize] = 0;
        i += 1;
    }

    core::ptr::write_bytes(&mut DnsDomainGuid, 0, 1);

    (*((&mut ModificationInfo.ModifiedId) as *mut _ as *mut _LARGE_INTEGER)).QuadPart = 0;
    NtQuerySystemTime(core::ptr::addr_of_mut!(ModificationInfo.DatabaseCreationTime) as *mut LARGE_INTEGER);

    'done: loop {
    Status = LsapCreateRandomDomainSid(&mut LocalAccountDomainSid);
    if !NT_SUCCESS(Status) {
        break 'done;
    }

    Status = LsapCreatePolicySd(&mut PolicySd, &mut PolicySdSize);
    if !NT_SUCCESS(Status) {
        break 'done;
    }

    Status = LsapOpenDbObject(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w!("Policy").as_ptr() as *mut u16,
        _LSA_DB_OBJECT_TYPE_LsaDbPolicyObject,
        0,
        1u8,
        &mut PolicyObject,
    );
    if !NT_SUCCESS(Status) {
        break 'done;
    }

    LsapSetObjectAttribute(PolicyObject, w!("PolPrDmN").as_ptr() as *mut u16, core::ptr::null_mut(), 0);
    LsapSetObjectAttribute(PolicyObject, w!("PolPrDmS").as_ptr() as *mut u16, core::ptr::null_mut(), 0);
    LsapSetObjectAttribute(PolicyObject, w!("PolAcDmN").as_ptr() as *mut u16, core::ptr::null_mut(), 0);
    LsapSetObjectAttribute(
        PolicyObject,
        w!("PolAcDmS").as_ptr() as *mut u16,
        LocalAccountDomainSid,
        RtlLengthSid(LocalAccountDomainSid),
    );
    LsapSetObjectAttribute(
        PolicyObject,
        w!("DefQuota").as_ptr() as *mut u16,
        &QuotaInfo as *const _ as *mut core::ffi::c_void,
        core::mem::size_of::<_POLICY_DEFAULT_QUOTA_INFO>() as u32,
    );
    LsapSetObjectAttribute(
        PolicyObject,
        w!("PolMod").as_ptr() as *mut u16,
        &ModificationInfo as *const _ as *mut core::ffi::c_void,
        core::mem::size_of::<_POLICY_MODIFICATION_INFO>() as u32,
    );
    LsapSetObjectAttribute(
        PolicyObject,
        w!("PolAdtFl").as_ptr() as *mut u16,
        &AuditFullInfo as *const _ as *mut core::ffi::c_void,
        core::mem::size_of::<_POLICY_AUDIT_FULL_QUERY_INFO>() as u32,
    );
    LsapSetObjectAttribute(
        PolicyObject,
        w!("PolAdtLg").as_ptr() as *mut u16,
        &AuditLogInfo as *const _ as *mut core::ffi::c_void,
        core::mem::size_of::<_POLICY_AUDIT_LOG_INFO>() as u32,
    );
    LsapSetObjectAttribute(
        PolicyObject,
        w!("PolAdtEv").as_ptr() as *mut u16,
        AuditEventsInfo as *mut core::ffi::c_void,
        core::mem::size_of::<_LSAP_POLICY_AUDIT_EVENTS_DATA>() as u32,
    );
    LsapSetObjectAttribute(PolicyObject, w!("PolDnDDN").as_ptr() as *mut u16, core::ptr::null_mut(), 0);
    LsapSetObjectAttribute(PolicyObject, w!("PolDnTrN").as_ptr() as *mut u16, core::ptr::null_mut(), 0);
    LsapSetObjectAttribute(
        PolicyObject,
        w!("PolDnDmG").as_ptr() as *mut u16,
        &DnsDomainGuid as *const _ as *mut core::ffi::c_void,
        core::mem::size_of::<GUID>() as u32,
    );
    LsapSetObjectAttribute(
        PolicyObject,
        w!("SecDesc").as_ptr() as *mut u16,
        PolicySd,
        PolicySdSize,
    );

    break 'done;
    } // 'done

    if !AuditEventsInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, AuditEventsInfo as *mut core::ffi::c_void);
    }
    if !PolicyObject.is_null() {
        LsapCloseDbObject(PolicyObject);
    }
    if !LocalAccountDomainSid.is_null() {
        RtlFreeSid(LocalAccountDomainSid);
    }
    if !PolicySd.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, PolicySd as *mut core::ffi::c_void);
    }

    Status
}

unsafe fn LsapUpdateDatabase() -> i32 {
    STATUS_SUCCESS
}

unsafe fn LsapGetDomainInfo() -> i32 {
    // Wide strings
    const BUILTIN_WSTR: [u16; 8] = [
        0x0042, 0x0055, 0x0049, 0x004C, 0x0054, 0x0049, 0x004E, 0x0000,
    ];
    const POLICY_WSTR: [u16; 7] = [
        0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0079, 0x0000,
    ];
    const POLACDMS_WSTR: [u16; 9] = [
        0x0050, 0x006F, 0x006C, 0x0041, 0x0063, 0x0044, 0x006D, 0x0053, 0x0000,
    ];
    const POLACDMN_WSTR: [u16; 9] = [
        0x0050, 0x006F, 0x006C, 0x0041, 0x0063, 0x0044, 0x006D, 0x004E, 0x0000,
    ];

    let mut PolicyObject: *mut _LSA_DB_OBJECT = core::ptr::null_mut();
    let mut DomainName: *mut _UNICODE_STRING = core::ptr::null_mut();
    let mut SidString: *mut u16 = core::ptr::null_mut();
    let mut AttributeSize: u32;

    let status = 'done: {
        // Build-in domain SID
        let mut status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            1,
            SECURITY_BUILTIN_DOMAIN_RID,
            0, 0, 0, 0, 0, 0, 0,
            &raw mut BuiltinDomainSid,
        );
        if !NT_SUCCESS(status) {
            break 'done status;
        }

        BuiltinDomainName.Length = 14;
        BuiltinDomainName.MaximumLength = 16;
        BuiltinDomainName.Buffer = BUILTIN_WSTR.as_ptr() as *mut u16;

        // Open Policy object
        status = LsapOpenDbObject(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            POLICY_WSTR.as_ptr() as *mut u16,
            LsaDbPolicyObject,
            0,
            1u8,
            &mut PolicyObject,
        );
        if !NT_SUCCESS(status) {
            break 'done status;
        }

        // Get account domain SID
        AttributeSize = 0;
        status = LsapGetObjectAttribute(
            PolicyObject,
            POLACDMS_WSTR.as_ptr() as *mut u16,
            core::ptr::null_mut(),
            &mut AttributeSize,
        );
        if !NT_SUCCESS(status) {
            break 'done status;
        }

        if AttributeSize > 0 {
            AccountDomainSid = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                AttributeSize as SIZE_T,
            ) as PSID;
            if AccountDomainSid.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done status;
            }

            status = LsapGetObjectAttribute(
                PolicyObject,
                POLACDMS_WSTR.as_ptr() as *mut u16,
                AccountDomainSid as *mut core::ffi::c_void,
                &mut AttributeSize,
            );
            if !NT_SUCCESS(status) {
                break 'done status;
            }
        }

        // Get account domain name
        AttributeSize = 0;
        status = LsapGetObjectAttribute(
            PolicyObject,
            POLACDMN_WSTR.as_ptr() as *mut u16,
            core::ptr::null_mut(),
            &mut AttributeSize,
        );
        if !NT_SUCCESS(status) {
            break 'done status;
        }

        if AttributeSize > 0 {
            DomainName = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                AttributeSize as SIZE_T,
            ) as *mut _UNICODE_STRING;
            if DomainName.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done status;
            }

            status = LsapGetObjectAttribute(
                PolicyObject,
                POLACDMN_WSTR.as_ptr() as *mut u16,
                DomainName as *mut core::ffi::c_void,
                &mut AttributeSize,
            );
            if !NT_SUCCESS(status) {
                break 'done status;
            }

            // Adjust Buffer pointer (offset -> absolute)
            let buffer_offset = (*DomainName).Buffer as usize;
            (*DomainName).Buffer = ((DomainName as usize) + buffer_offset) as *mut u16;

            AccountDomainName.Length = (*DomainName).Length;
            AccountDomainName.MaximumLength = (*DomainName).Length + 2;
            AccountDomainName.Buffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                AccountDomainName.MaximumLength as SIZE_T,
            ) as *mut u16;
            if AccountDomainName.Buffer.is_null() {
                ERR!(
                    "Failed to allocate the account domain name buffer\n",
                );
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done status;
            }

            core::ptr::copy_nonoverlapping(
                (*DomainName).Buffer as *const u8,
                AccountDomainName.Buffer as *mut u8,
                (*DomainName).Length as usize,
            );
        }

        // Trace SIDs and names
        ConvertSidToStringSidW(BuiltinDomainSid, &mut SidString);
        TRACE!(b"Builtin Domain SID: %S\n\0".as_ptr() as *const i8, SidString);
        LocalFree(SidString as *mut core::ffi::c_void);
        SidString = core::ptr::null_mut();

        TRACE!(
            b"Builtin Domain Name: %wZ\n\0".as_ptr() as *const i8,
            &BuiltinDomainName as *const _UNICODE_STRING,
        );

        ConvertSidToStringSidW(AccountDomainSid, &mut SidString);
        TRACE!(b"Account Domain SID: %S\n\0".as_ptr() as *const i8, SidString);
        LocalFree(SidString as *mut core::ffi::c_void);

        TRACE!(
            b"Account Domain Name: %wZ\n\0".as_ptr() as *const i8,
            &AccountDomainName as *const _UNICODE_STRING,
        );

        status // return whatever status we have (should be success)
    };

    // Cleanup
    if !DomainName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, DomainName as *mut core::ffi::c_void);
    }
    if !PolicyObject.is_null() {
        LsapCloseDbObject(PolicyObject);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn LsapInitDatabase() -> i32 {
    let mut status: i32;

    TRACE!(b"LsapInitDatabase()\n\0".as_ptr() as *const u8);
    status = LsapOpenServiceKey();
    if !NT_SUCCESS(status) {
        ERR!("Failed to open the service key (Status: 0x%08lx)\n", status as u32);
        return status;
    }

    if LsapIsDatabaseInstalled() == 0u8 {
        status = LsapCreateDatabaseKeys();
        if !NT_SUCCESS(status) {
            ERR!("Failed to create the LSA database keys (Status: 0x%08lx)\n", status as u32);
            return status;
        }

        status = LsapCreateDatabaseObjects();
        if !NT_SUCCESS(status) {
            ERR!("Failed to create the LSA database objects (Status: 0x%08lx)\n", status as u32);
            return status;
        }
    } else {
        status = LsapUpdateDatabase();
        if !NT_SUCCESS(status) {
            ERR!("Failed to update the LSA database (Status: 0x%08lx)\n", status as u32);
            return status;
        }
    }

    status = LsapGetDomainInfo();
    if !NT_SUCCESS(status) {
        ERR!("Failed to get the domain information (Status: 0x%08lx)\n", status as u32);
        return status;
    }

    TRACE!(b"LsapInitDatabase() done\n\0".as_ptr() as *const u8);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapCreateDbObject(
    ParentObject: *mut LSA_DB_OBJECT,
    ContainerName: *mut u16,
    ObjectName: *mut u16,
    ObjectType: LSA_DB_OBJECT_TYPE,
    DesiredAccess: ACCESS_MASK,
    Trusted: u8,
    DbObject: *mut *mut LSA_DB_OBJECT,
) -> NTSTATUS {
    if DbObject.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let parent_key_handle: HANDLE;
    if ParentObject.is_null() {
        parent_key_handle = SecurityKeyHandle;
    } else {
        parent_key_handle = (*ParentObject).KeyHandle;
    }

    let mut container_key_handle: HANDLE = core::ptr::null_mut();
    let mut object_key_handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    if !ContainerName.is_null() {
        // Open the container key
        let mut key_name: _UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut key_name, ContainerName);

        let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        (*core::ptr::addr_of_mut!(object_attributes)).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*core::ptr::addr_of_mut!(object_attributes)).RootDirectory = parent_key_handle;
        (*core::ptr::addr_of_mut!(object_attributes)).Attributes = OBJ_CASE_INSENSITIVE;
        (*core::ptr::addr_of_mut!(object_attributes)).ObjectName = &mut key_name;
        (*core::ptr::addr_of_mut!(object_attributes)).SecurityDescriptor = core::ptr::null_mut();
        (*core::ptr::addr_of_mut!(object_attributes)).SecurityQualityOfService = core::ptr::null_mut();

        status = NtOpenKey(&mut container_key_handle, KEY_ALL_ACCESS, &mut object_attributes);
        if !NT_SUCCESS(status) {
            return status;
        }

        // Open the object key
        let mut key_name: _UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut key_name, ObjectName);

        let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        (*core::ptr::addr_of_mut!(object_attributes)).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*core::ptr::addr_of_mut!(object_attributes)).RootDirectory = container_key_handle;
        (*core::ptr::addr_of_mut!(object_attributes)).Attributes = OBJ_CASE_INSENSITIVE;
        (*core::ptr::addr_of_mut!(object_attributes)).ObjectName = &mut key_name;
        (*core::ptr::addr_of_mut!(object_attributes)).SecurityDescriptor = core::ptr::null_mut();
        (*core::ptr::addr_of_mut!(object_attributes)).SecurityQualityOfService = core::ptr::null_mut();

        status = NtCreateKey(
            &mut object_key_handle,
            KEY_ALL_ACCESS,
            &mut object_attributes,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );

        NtClose(container_key_handle);

        if !NT_SUCCESS(status) {
            return status;
        }
    } else {
        let mut key_name: _UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut key_name, ObjectName);

        let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        (*core::ptr::addr_of_mut!(object_attributes)).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*core::ptr::addr_of_mut!(object_attributes)).RootDirectory = parent_key_handle;
        (*core::ptr::addr_of_mut!(object_attributes)).Attributes = OBJ_CASE_INSENSITIVE;
        (*core::ptr::addr_of_mut!(object_attributes)).ObjectName = &mut key_name;
        (*core::ptr::addr_of_mut!(object_attributes)).SecurityDescriptor = core::ptr::null_mut();
        (*core::ptr::addr_of_mut!(object_attributes)).SecurityQualityOfService = core::ptr::null_mut();

        status = NtCreateKey(
            &mut object_key_handle,
            KEY_ALL_ACCESS,
            &mut object_attributes,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    let new_object = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<LSA_DB_OBJECT>() as SIZE_T,
    ) as *mut LSA_DB_OBJECT;
    if new_object.is_null() {
        NtClose(object_key_handle);
        return STATUS_NO_MEMORY;
    }

    (*new_object).Signature = LSAP_DB_SIGNATURE;
    (*new_object).RefCount = 1;
    (*new_object).ObjectType = ObjectType;
    (*new_object).Access = DesiredAccess;
    (*new_object).KeyHandle = object_key_handle;
    (*new_object).ParentObject = ParentObject;
    (*new_object).Trusted = Trusted;

    if !ParentObject.is_null() {
        (*ParentObject).RefCount += 1;
    }

    *DbObject = new_object;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapOpenDbObject(
    parent_object: *mut _LSA_DB_OBJECT,
    container_name: *mut u16,
    object_name: *mut u16,
    object_type: _LSA_DB_OBJECT_TYPE,
    desired_access: ACCESS_MASK,
    trusted: u8,
    db_object: *mut *mut _LSA_DB_OBJECT,
) -> NTSTATUS {
    if db_object.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let parent_key_handle: HANDLE = if parent_object.is_null() {
        SecurityKeyHandle
    } else {
        (*parent_object).KeyHandle
    };

    let mut container_key_handle: HANDLE = core::ptr::null_mut();
    let mut object_key_handle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    let mut key_name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut object_attributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    if !container_name.is_null() {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), container_name);

        {
            let attr_ptr = core::ptr::addr_of_mut!(object_attributes);
            (*attr_ptr).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            (*attr_ptr).RootDirectory = parent_key_handle;
            (*attr_ptr).Attributes = OBJ_CASE_INSENSITIVE;
            (*attr_ptr).ObjectName = core::ptr::addr_of_mut!(key_name);
            (*attr_ptr).SecurityDescriptor = core::ptr::null_mut();
            (*attr_ptr).SecurityQualityOfService = core::ptr::null_mut();
        }

        status = NtOpenKey(
            core::ptr::addr_of_mut!(container_key_handle),
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
        );
        if !NT_SUCCESS(status) {
            return status;
        }

        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), object_name);

        let attr_ptr = core::ptr::addr_of_mut!(object_attributes);
        (*attr_ptr).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*attr_ptr).RootDirectory = container_key_handle;
        (*attr_ptr).Attributes = OBJ_CASE_INSENSITIVE;
        (*attr_ptr).ObjectName = core::ptr::addr_of_mut!(key_name);
        (*attr_ptr).SecurityDescriptor = core::ptr::null_mut();
        (*attr_ptr).SecurityQualityOfService = core::ptr::null_mut();

        status = NtOpenKey(
            core::ptr::addr_of_mut!(object_key_handle),
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
        );

        NtClose(container_key_handle);

        if !NT_SUCCESS(status) {
            return status;
        }
    } else {
        RtlInitUnicodeString(core::ptr::addr_of_mut!(key_name), object_name);

        let attr_ptr = core::ptr::addr_of_mut!(object_attributes);
        (*attr_ptr).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        (*attr_ptr).RootDirectory = parent_key_handle;
        (*attr_ptr).Attributes = OBJ_CASE_INSENSITIVE;
        (*attr_ptr).ObjectName = core::ptr::addr_of_mut!(key_name);
        (*attr_ptr).SecurityDescriptor = core::ptr::null_mut();
        (*attr_ptr).SecurityQualityOfService = core::ptr::null_mut();

        status = NtOpenKey(
            core::ptr::addr_of_mut!(object_key_handle),
            KEY_ALL_ACCESS,
            core::ptr::addr_of_mut!(object_attributes),
        );
        if !NT_SUCCESS(status) {
            return status;
        }
    }

    let new_object: *mut _LSA_DB_OBJECT = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        core::mem::size_of::<_LSA_DB_OBJECT>() as SIZE_T,
    ) as *mut _LSA_DB_OBJECT;
    if new_object.is_null() {
        NtClose(object_key_handle);
        return STATUS_NO_MEMORY;
    }

    (*new_object).Signature = LSAP_DB_SIGNATURE;
    (*new_object).RefCount = 1;
    (*new_object).ObjectType = object_type;
    (*new_object).Access = desired_access;
    (*new_object).KeyHandle = object_key_handle;
    (*new_object).ParentObject = parent_object;
    (*new_object).Trusted = trusted;

    if !parent_object.is_null() {
        (*parent_object).RefCount += 1;
    }

    *db_object = new_object;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapValidateDbObject(
    Handle: PLSA_DB_OBJECT,
    ObjectType: LSA_DB_OBJECT_TYPE,
    DesiredAccess: ACCESS_MASK,
    DbObject: *mut PLSA_DB_OBJECT,
) -> NTSTATUS {
    let mut bValid: u8 = 0; // FALSE

    // DUMMY_PSEH: execute TRY body directly, omit exception handler
    if (*Handle).Signature == LSAP_DB_SIGNATURE {
        if (ObjectType == LsaDbIgnoreObject) || ((*Handle).ObjectType == ObjectType) {
            bValid = 1; // TRUE
        }
    }

    if bValid == 0 {
        return STATUS_INVALID_HANDLE;
    }

    if DesiredAccess != 0 {
        if ((*Handle).Access & DesiredAccess) != DesiredAccess {
            ERR!(
                "LsapValidateDbObject access check failed %08lx  %08lx\n",
                (*Handle).Access,
                DesiredAccess,
            );
            return STATUS_ACCESS_DENIED;
        }
    }

    if !DbObject.is_null() {
        *DbObject = Handle;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapCloseDbObject(DbObject: PLSA_DB_OBJECT) -> NTSTATUS {
    let mut ParentObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    (*DbObject).RefCount -= 1;

    if (*DbObject).RefCount > 0 {
        return STATUS_SUCCESS;
    }

    if (*DbObject).KeyHandle != core::ptr::null_mut() {
        NtClose((*DbObject).KeyHandle);
    }

    if (*DbObject).ParentObject != core::ptr::null_mut() {
        ParentObject = (*DbObject).ParentObject;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject as LPVOID);

    if ParentObject != core::ptr::null_mut() {
        (*ParentObject).RefCount -= 1;

        if (*ParentObject).RefCount == 0 {
            Status = LsapCloseDbObject(ParentObject);
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapDeleteDbObject(DbObject: PLSA_DB_OBJECT) -> NTSTATUS {
    let mut parent_object: PLSA_DB_OBJECT = core::ptr::null_mut();
    let mut key_name: [u16; 64] = [0; 64];
    let mut index: u32;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    (*DbObject).RefCount -= 1;

    if (*DbObject).RefCount > 0 {
        return STATUS_SUCCESS;
    }

    if !(*DbObject).KeyHandle.is_null() {
        index = 0;
        loop {
            status = LsapRegEnumerateSubKey(
                (*DbObject).KeyHandle,
                index,
                core::mem::size_of::<[u16; 64]>() as u32,
                key_name.as_mut_ptr(),
            );
            if !NT_SUCCESS(status) {
                break;
            }
            status = LsapRegDeleteSubKey(
                (*DbObject).KeyHandle,
                key_name.as_ptr(),
            );
            if !NT_SUCCESS(status) {
                break;
            }
            index += 1;
        }

        if status == STATUS_NO_MORE_ENTRIES {
            status = STATUS_SUCCESS;
        }

        LsapRegDeleteKey((*DbObject).KeyHandle);
        NtClose((*DbObject).KeyHandle);
    }

    if !(*DbObject).ParentObject.is_null() {
        parent_object = (*DbObject).ParentObject;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject as *mut c_void);

    if !parent_object.is_null() {
        (*parent_object).RefCount -= 1;
        if (*parent_object).RefCount == 0 {
            status = LsapCloseDbObject(parent_object);
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn LsapSetObjectAttribute(
    DbObject: *mut _LSA_DB_OBJECT,
    AttributeName: *mut u16,
    AttributeData: *mut core::ffi::c_void,
    AttributeSize: u32,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut AttributeKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut KeyName, AttributeName);

    ObjectAttributes.Length = size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = (*DbObject).KeyHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateKey(
        &mut AttributeKey as *mut _,
        KEY_SET_VALUE,
        &mut ObjectAttributes as *mut _,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        ERR!(
            "NtCreateKey failed for '%S' with status 0x%lx\n",
            AttributeName,
            Status,
        );
        return Status;
    }

    Status = RtlpNtSetValueKey(
        AttributeKey,
        REG_NONE,
        AttributeData,
        AttributeSize,
    );

    NtClose(AttributeKey);

    if Status < 0 {
        ERR!(
            "RtlpNtSetValueKey failed for '%S' with status 0x%lx\n",
            AttributeName,
            Status,
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapGetObjectAttribute(
    DbObject: PLSA_DB_OBJECT,
    AttributeName: LPWSTR,
    AttributeData: LPVOID,
    AttributeSize: PULONG,
) -> NTSTATUS {
    let mut key_name: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut key_name, AttributeName);

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    object_attributes.RootDirectory = (*DbObject).KeyHandle;
    object_attributes.ObjectName = &mut key_name;
    object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
    object_attributes.SecurityDescriptor = core::ptr::null_mut();
    object_attributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut attribute_key: HANDLE = core::ptr::null_mut();
    let mut status = NtOpenKey(
        &mut attribute_key,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(object_attributes),
    );
    if !NT_SUCCESS(status) {
        return status;
    }

    let mut value_size: u32 = *AttributeSize;
    status = RtlpNtQueryValueKey(
        attribute_key,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut value_size,
        0,
    );
    if !NT_SUCCESS(status) && status != STATUS_BUFFER_OVERFLOW {
        NtClose(attribute_key);
        return status;
    }

    if AttributeData.is_null() || *AttributeSize == 0 {
        *AttributeSize = value_size;
        NtClose(attribute_key);
        return STATUS_SUCCESS;
    } else if *AttributeSize < value_size {
        *AttributeSize = value_size;
        NtClose(attribute_key);
        return STATUS_BUFFER_OVERFLOW;
    }

    status = RtlpNtQueryValueKey(
        attribute_key,
        core::ptr::null_mut(),
        AttributeData,
        &mut value_size,
        0,
    );
    if NT_SUCCESS(status) {
        *AttributeSize = value_size;
    }

    NtClose(attribute_key);
    status
}
#[no_mangle]
pub unsafe extern "C" fn LsapDeleteObjectAttribute(DbObject: *mut LSA_DB_OBJECT, AttributeName: *mut u16) -> i32 {
    LsapRegDeleteSubKey((*DbObject).KeyHandle, AttributeName)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/database.c
//  * PURPOSE:     LSA object database
//  * COPYRIGHT:   Copyright 2011 Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// #include <pseh/pseh2.h>
//
// /* GLOBALS *****************************************************************/
//
// static HANDLE SecurityKeyHandle = NULL;
//
// SID_IDENTIFIER_AUTHORITY NullSidAuthority    = {SECURITY_NULL_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY WorldSidAuthority   = {SECURITY_WORLD_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY LocalSidAuthority   = {SECURITY_LOCAL_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY CreatorSidAuthority = {SECURITY_CREATOR_SID_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY NtAuthority         = {SECURITY_NT_AUTHORITY};
//
// PSID BuiltinDomainSid = NULL;
// PSID AccountDomainSid = NULL;
// UNICODE_STRING BuiltinDomainName = {0, 0, NULL};
// UNICODE_STRING AccountDomainName = {0, 0, NULL};
//
//
// /* FUNCTIONS ***************************************************************/
//
// static NTSTATUS
// LsapOpenServiceKey(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\SECURITY");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = RtlpNtOpenKey(&SecurityKeyHandle,
//                            KEY_READ | KEY_CREATE_SUB_KEY | KEY_ENUMERATE_SUB_KEYS,
//                            &ObjectAttributes,
//                            0);
//
//     return Status;
// }
//
//
// static BOOLEAN
// LsapIsDatabaseInstalled(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&KeyName,
//                          L"Policy");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                SecurityKeyHandle,
//                                NULL);
//
//     Status = RtlpNtOpenKey(&KeyHandle,
//                            KEY_READ,
//                            &ObjectAttributes,
//                            0);
//     if (!NT_SUCCESS(Status))
//         return FALSE;
//
//     NtClose(KeyHandle);
//
//     return TRUE;
// }
//
//
// static NTSTATUS
// LsapCreateDatabaseKeys(VOID)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE PolicyKeyHandle = NULL;
//     HANDLE AccountsKeyHandle = NULL;
//     HANDLE DomainsKeyHandle = NULL;
//     HANDLE SecretsKeyHandle = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsapInstallDatabase()\n");
//
//     /* Create the 'Policy' key */
//     RtlInitUnicodeString(&KeyName,
//                          L"Policy");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                SecurityKeyHandle,
//                                NULL);
//
//     Status = NtCreateKey(&PolicyKeyHandle,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to create the 'Policy' key (Status: 0x%08lx)\n", Status);
//         goto Done;
//     }
//
//     /* Create the 'Accounts' key */
//     RtlInitUnicodeString(&KeyName,
//                          L"Accounts");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                PolicyKeyHandle,
//                                NULL);
//
//     Status = NtCreateKey(&AccountsKeyHandle,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to create the 'Accounts' key (Status: 0x%08lx)\n", Status);
//         goto Done;
//     }
//
//     /* Create the 'Domains' key */
//     RtlInitUnicodeString(&KeyName,
//                          L"Domains");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                PolicyKeyHandle,
//                                NULL);
//
//     Status = NtCreateKey(&DomainsKeyHandle,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to create the 'Domains' key (Status: 0x%08lx)\n", Status);
//         goto Done;
//     }
//
//     /* Create the 'Secrets' key */
//     RtlInitUnicodeString(&KeyName,
//                          L"Secrets");
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                PolicyKeyHandle,
//                                NULL);
//
//     Status = NtCreateKey(&SecretsKeyHandle,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          0,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to create the 'Secrets' key (Status: 0x%08lx)\n", Status);
//         goto Done;
//     }
//
// Done:
//     if (SecretsKeyHandle != NULL)
//         NtClose(SecretsKeyHandle);
//
//     if (DomainsKeyHandle != NULL)
//         NtClose(DomainsKeyHandle);
//
//     if (AccountsKeyHandle != NULL)
//         NtClose(AccountsKeyHandle);
//
//     if (PolicyKeyHandle != NULL)
//         NtClose(PolicyKeyHandle);
//
//     TRACE("LsapInstallDatabase() done (Status: 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// static NTSTATUS
// LsapCreateRandomDomainSid(OUT PSID *Sid)
// {
//     LARGE_INTEGER SystemTime;
//     PULONG Seed;
//
//     NtQuerySystemTime(&SystemTime);
//     Seed = &SystemTime.u.LowPart;
//
//     return RtlAllocateAndInitializeSid(&NtAuthority,
//                                        4,
//                                        SECURITY_NT_NON_UNIQUE,
//                                        RtlUniform(Seed),
//                                        RtlUniform(Seed),
//                                        RtlUniform(Seed),
//                                        SECURITY_NULL_RID,
//                                        SECURITY_NULL_RID,
//                                        SECURITY_NULL_RID,
//                                        SECURITY_NULL_RID,
//                                        Sid);
// }
//
//
// static NTSTATUS
// LsapCreateDatabaseObjects(VOID)
// {
//     PLSAP_POLICY_AUDIT_EVENTS_DATA AuditEventsInfo = NULL;
//     POLICY_DEFAULT_QUOTA_INFO QuotaInfo;
//     POLICY_MODIFICATION_INFO ModificationInfo;
//     POLICY_AUDIT_FULL_QUERY_INFO AuditFullInfo = {FALSE, FALSE};
//     POLICY_AUDIT_LOG_INFO AuditLogInfo;
//     GUID DnsDomainGuid;
//     PLSA_DB_OBJECT PolicyObject = NULL;
//     PSID AccountDomainSid = NULL;
//     PSECURITY_DESCRIPTOR PolicySd = NULL;
//     ULONG PolicySdSize = 0;
//     ULONG i;
//     NTSTATUS Status;
//
//     /* Initialize the default quota limits */
//     QuotaInfo.QuotaLimits.PagedPoolLimit = 0x2000000;
//     QuotaInfo.QuotaLimits.NonPagedPoolLimit = 0x100000;
//     QuotaInfo.QuotaLimits.MinimumWorkingSetSize = 0x10000;
//     QuotaInfo.QuotaLimits.MaximumWorkingSetSize = 0xF000000;
//     QuotaInfo.QuotaLimits.PagefileLimit = 0;
//     QuotaInfo.QuotaLimits.TimeLimit.QuadPart = 0;
//
//     /* Initialize the audit log attribute */
//     AuditLogInfo.AuditLogPercentFull = 0;
//     AuditLogInfo.MaximumLogSize = 0;                    // DWORD
//     AuditLogInfo.AuditRetentionPeriod.QuadPart = 0;     // LARGE_INTEGER
//     AuditLogInfo.AuditLogFullShutdownInProgress = 0;    // BYTE
//     AuditLogInfo.TimeToShutdown.QuadPart = 0;           // LARGE_INTEGER
//     AuditLogInfo.NextAuditRecordId = 0;                 // DWORD
//
//     /* Initialize the Audit Events attribute */
//     AuditEventsInfo = RtlAllocateHeap(RtlGetProcessHeap(),
//                                       HEAP_ZERO_MEMORY,
//                                       sizeof(LSAP_POLICY_AUDIT_EVENTS_DATA));
//     if (AuditEventsInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     AuditEventsInfo->AuditingMode = FALSE;
//     AuditEventsInfo->MaximumAuditEventCount = POLICY_AUDIT_EVENT_TYPE_COUNT;
//     for (i = 0; i < POLICY_AUDIT_EVENT_TYPE_COUNT; i++)
//         AuditEventsInfo->AuditEvents[i] = 0;
//
//     /* Initialize the DNS Domain GUID attribute */
//     RtlZeroMemory(&DnsDomainGuid, sizeof(DnsDomainGuid));
//
//     /* Initialize the modification attribute */
//     ModificationInfo.ModifiedId.QuadPart = 0;
//     NtQuerySystemTime(&ModificationInfo.DatabaseCreationTime);
//
//     /* Create a random domain SID */
//     Status = LsapCreateRandomDomainSid(&AccountDomainSid);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     Status = LsapCreatePolicySd(&PolicySd, &PolicySdSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Open the 'Policy' object */
//     Status = LsapOpenDbObject(NULL,
//                               NULL,
//                               L"Policy",
//                               LsaDbPolicyObject,
//                               0,
//                               TRUE,
//                               &PolicyObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Set the Primary Domain Name attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolPrDmN",
//                            NULL,
//                            0);
//
//     /* Set the Primary Domain SID attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolPrDmS",
//                            NULL,
//                            0);
//
//     /* Set the Account Domain Name attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolAcDmN",
//                            NULL,
//                            0);
//
//     /* Set the Account Domain SID attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolAcDmS",
//                            AccountDomainSid,
//                            RtlLengthSid(AccountDomainSid));
//
//     /* Set the default quota limits attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"DefQuota",
//                            &QuotaInfo,
//                            sizeof(QuotaInfo));
//
//     /* Set the modification attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolMod",
//                            &ModificationInfo,
//                            sizeof(ModificationInfo));
//
//     /* Set the audit full attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolAdtFl",
//                            &AuditFullInfo,
//                            sizeof(AuditFullInfo));
//
//     /* Set the audit log attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolAdtLg",
//                            &AuditLogInfo,
//                            sizeof(AuditLogInfo));
//
//     /* Set the audit events attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolAdtEv",
//                            AuditEventsInfo,
//                            sizeof(*AuditEventsInfo));
//
//     /* Set the DNS Domain Name attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolDnDDN",
//                            NULL,
//                            0);
//
//     /* Set the DNS Forest Name attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolDnTrN",
//                            NULL,
//                            0);
//
//     /* Set the DNS Domain GUID attribute */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"PolDnDmG",
//                            &DnsDomainGuid,
//                            sizeof(DnsDomainGuid));
//
//     /* Set the Security Descriptor */
//     LsapSetObjectAttribute(PolicyObject,
//                            L"SecDesc",
//                            PolicySd,
//                            PolicySdSize);
//
// done:
//     if (AuditEventsInfo != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, AuditEventsInfo);
//
//     if (PolicyObject != NULL)
//         LsapCloseDbObject(PolicyObject);
//
//     if (AccountDomainSid != NULL)
//         RtlFreeSid(AccountDomainSid);
//
//     if (PolicySd != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, PolicySd);
//
//     return Status;
// }
//
//
// static NTSTATUS
// LsapUpdateDatabase(VOID)
// {
//     return STATUS_SUCCESS;
// }
//
//
// static NTSTATUS
// LsapGetDomainInfo(VOID)
// {
//     PLSA_DB_OBJECT PolicyObject = NULL;
//     PUNICODE_STRING DomainName = NULL;
//     ULONG AttributeSize;
//     LPWSTR SidString = NULL;
//     NTSTATUS Status;
//
//     /* Get the built-in domain SID and name */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &BuiltinDomainSid);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /**/
//     RtlInitUnicodeString(&BuiltinDomainName,
//                          L"BUILTIN");
//
//     /* Open the 'Policy' object */
//     Status = LsapOpenDbObject(NULL,
//                               NULL,
//                               L"Policy",
//                               LsaDbPolicyObject,
//                               0,
//                               TRUE,
//                               &PolicyObject);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     /* Get the account domain SID */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAcDmS",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (AttributeSize > 0)
//     {
//         AccountDomainSid = RtlAllocateHeap(RtlGetProcessHeap(),
//                                            HEAP_ZERO_MEMORY,
//                                            AttributeSize);
//         if (AccountDomainSid == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolAcDmS",
//                                         AccountDomainSid,
//                                         &AttributeSize);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     /* Get the account domain name */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAcDmN",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (AttributeSize > 0)
//     {
//         DomainName = RtlAllocateHeap(RtlGetProcessHeap(),
//                                      HEAP_ZERO_MEMORY,
//                                      AttributeSize);
//         if (DomainName == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolAcDmN",
//                                         DomainName,
//                                         &AttributeSize);
//         if (!NT_SUCCESS(Status))
//             goto done;
//
//         DomainName->Buffer = (LPWSTR)((ULONG_PTR)DomainName + (ULONG_PTR)DomainName->Buffer);
//
//         AccountDomainName.Length = DomainName->Length;
//         AccountDomainName.MaximumLength = DomainName->Length + sizeof(WCHAR);
//         AccountDomainName.Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                    HEAP_ZERO_MEMORY,
//                                                    AccountDomainName.MaximumLength);
//         if (AccountDomainName.Buffer == NULL)
//         {
//             ERR("Failed to allocate the account domain name buffer\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         RtlCopyMemory(AccountDomainName.Buffer,
//                       DomainName->Buffer,
//                       DomainName->Length);
//     }
//
//     ConvertSidToStringSidW(BuiltinDomainSid, &SidString);
//     TRACE("Builtin Domain SID: %S\n", SidString);
//     LocalFree(SidString);
//     SidString = NULL;
//
//     TRACE("Builtin Domain Name: %wZ\n", &BuiltinDomainName);
//
//     ConvertSidToStringSidW(AccountDomainSid, &SidString);
//     TRACE("Account Domain SID: %S\n", SidString);
//     LocalFree(SidString);
//     SidString = NULL;
//
//     TRACE("Account Domain Name: %wZ\n", &AccountDomainName);
//
// done:
//     if (DomainName != NULL)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DomainName);
//
//     if (PolicyObject != NULL)
//         LsapCloseDbObject(PolicyObject);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapInitDatabase(VOID)
// {
//     NTSTATUS Status;
//
//     TRACE("LsapInitDatabase()\n");
//
//     Status = LsapOpenServiceKey();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to open the service key (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     if (!LsapIsDatabaseInstalled())
//     {
//         Status = LsapCreateDatabaseKeys();
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("Failed to create the LSA database keys (Status: 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         Status = LsapCreateDatabaseObjects();
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("Failed to create the LSA database objects (Status: 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//     else
//     {
//         Status = LsapUpdateDatabase();
//         if (!NT_SUCCESS(Status))
//         {
//             ERR("Failed to update the LSA database (Status: 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//
//     Status = LsapGetDomainInfo();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to get the domain information (Status: 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     TRACE("LsapInitDatabase() done\n");
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsapCreateDbObject(IN PLSA_DB_OBJECT ParentObject,
//                    IN LPWSTR ContainerName,
//                    IN LPWSTR ObjectName,
//                    IN LSA_DB_OBJECT_TYPE ObjectType,
//                    IN ACCESS_MASK DesiredAccess,
//                    IN BOOLEAN Trusted,
//                    OUT PLSA_DB_OBJECT *DbObject)
// {
//     PLSA_DB_OBJECT NewObject;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE ParentKeyHandle;
//     HANDLE ContainerKeyHandle = NULL;
//     HANDLE ObjectKeyHandle = NULL;
//     NTSTATUS Status;
//
//     if (DbObject == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     if (ParentObject == NULL)
//         ParentKeyHandle = SecurityKeyHandle;
//     else
//         ParentKeyHandle = ParentObject->KeyHandle;
//
//     if (ContainerName != NULL)
//     {
//         /* Open the container key */
//         RtlInitUnicodeString(&KeyName,
//                              ContainerName);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ParentKeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&ContainerKeyHandle,
//                            KEY_ALL_ACCESS,
//                            &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* Open the object key */
//         RtlInitUnicodeString(&KeyName,
//                              ObjectName);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ContainerKeyHandle,
//                                    NULL);
//
//         Status = NtCreateKey(&ObjectKeyHandle,
//                              KEY_ALL_ACCESS,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              0,
//                              NULL);
//
//         NtClose(ContainerKeyHandle);
//
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//     else
//     {
//         RtlInitUnicodeString(&KeyName,
//                              ObjectName);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ParentKeyHandle,
//                                    NULL);
//
//         Status = NtCreateKey(&ObjectKeyHandle,
//                              KEY_ALL_ACCESS,
//                              &ObjectAttributes,
//                              0,
//                              NULL,
//                              0,
//                              NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     NewObject = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 0,
//                                 sizeof(LSA_DB_OBJECT));
//     if (NewObject == NULL)
//     {
//         NtClose(ObjectKeyHandle);
//         return STATUS_NO_MEMORY;
//     }
//
//     NewObject->Signature = LSAP_DB_SIGNATURE;
//     NewObject->RefCount = 1;
//     NewObject->ObjectType = ObjectType;
//     NewObject->Access = DesiredAccess;
//     NewObject->KeyHandle = ObjectKeyHandle;
//     NewObject->ParentObject = ParentObject;
//     NewObject->Trusted = Trusted;
//
//     if (ParentObject != NULL)
//         ParentObject->RefCount++;
//
//     *DbObject = NewObject;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsapOpenDbObject(IN PLSA_DB_OBJECT ParentObject,
//                  IN LPWSTR ContainerName,
//                  IN LPWSTR ObjectName,
//                  IN LSA_DB_OBJECT_TYPE ObjectType,
//                  IN ACCESS_MASK DesiredAccess,
//                  IN BOOLEAN Trusted,
//                  OUT PLSA_DB_OBJECT *DbObject)
// {
//     PLSA_DB_OBJECT NewObject;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE ParentKeyHandle;
//     HANDLE ContainerKeyHandle = NULL;
//     HANDLE ObjectKeyHandle = NULL;
//     NTSTATUS Status;
//
//     if (DbObject == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     if (ParentObject == NULL)
//         ParentKeyHandle = SecurityKeyHandle;
//     else
//         ParentKeyHandle = ParentObject->KeyHandle;
//
//     if (ContainerName != NULL)
//     {
//         /* Open the container key */
//         RtlInitUnicodeString(&KeyName,
//                              ContainerName);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ParentKeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&ContainerKeyHandle,
//                            KEY_ALL_ACCESS,
//                            &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* Open the object key */
//         RtlInitUnicodeString(&KeyName,
//                              ObjectName);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ContainerKeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&ObjectKeyHandle,
//                            KEY_ALL_ACCESS,
//                            &ObjectAttributes);
//
//         NtClose(ContainerKeyHandle);
//
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//     else
//     {
//         /* Open the object key */
//         RtlInitUnicodeString(&KeyName,
//                              ObjectName);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    ParentKeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&ObjectKeyHandle,
//                            KEY_ALL_ACCESS,
//                            &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//     }
//
//     NewObject = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 0,
//                                 sizeof(LSA_DB_OBJECT));
//     if (NewObject == NULL)
//     {
//         NtClose(ObjectKeyHandle);
//         return STATUS_NO_MEMORY;
//     }
//
//     NewObject->Signature = LSAP_DB_SIGNATURE;
//     NewObject->RefCount = 1;
//     NewObject->ObjectType = ObjectType;
//     NewObject->Access = DesiredAccess;
//     NewObject->KeyHandle = ObjectKeyHandle;
//     NewObject->ParentObject = ParentObject;
//     NewObject->Trusted = Trusted;
//
//     if (ParentObject != NULL)
//         ParentObject->RefCount++;
//
//     *DbObject = NewObject;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsapValidateDbObject(LSAPR_HANDLE Handle,
//                      LSA_DB_OBJECT_TYPE ObjectType,
//                      ACCESS_MASK DesiredAccess,
//                      PLSA_DB_OBJECT *DbObject)
// {
//     PLSA_DB_OBJECT LocalObject = (PLSA_DB_OBJECT)Handle;
//     BOOLEAN bValid = FALSE;
//
//     _SEH2_TRY
//     {
//         if (LocalObject->Signature == LSAP_DB_SIGNATURE)
//         {
//             if ((ObjectType == LsaDbIgnoreObject) ||
//                 (LocalObject->ObjectType == ObjectType))
//                 bValid = TRUE;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         bValid = FALSE;
//     }
//     _SEH2_END;
//
//     if (bValid == FALSE)
//         return STATUS_INVALID_HANDLE;
//
//     if (DesiredAccess != 0)
//     {
//         /* Check for granted access rights */
//         if ((LocalObject->Access & DesiredAccess) != DesiredAccess)
//         {
//             ERR("LsapValidateDbObject access check failed %08lx  %08lx\n",
//                 LocalObject->Access, DesiredAccess);
//             return STATUS_ACCESS_DENIED;
//         }
//     }
//
//     if (DbObject != NULL)
//         *DbObject = LocalObject;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsapCloseDbObject(PLSA_DB_OBJECT DbObject)
// {
//     PLSA_DB_OBJECT ParentObject = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DbObject->RefCount--;
//
//     if (DbObject->RefCount > 0)
//         return STATUS_SUCCESS;
//
//     if (DbObject->KeyHandle != NULL)
//         NtClose(DbObject->KeyHandle);
//
//     if (DbObject->ParentObject != NULL)
//         ParentObject = DbObject->ParentObject;
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject);
//
//     if (ParentObject != NULL)
//     {
//         ParentObject->RefCount--;
//
//         if (ParentObject->RefCount == 0)
//             Status = LsapCloseDbObject(ParentObject);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapDeleteDbObject(IN PLSA_DB_OBJECT DbObject)
// {
//     PLSA_DB_OBJECT ParentObject = NULL;
//     WCHAR KeyName[64];
//     ULONG Index;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DbObject->RefCount--;
//
//     if (DbObject->RefCount > 0)
//         return STATUS_SUCCESS;
//
//     if (DbObject->KeyHandle != NULL)
//     {
//         Index = 0;
//
//         while (TRUE)
//         {
//             Status = LsapRegEnumerateSubKey(DbObject->KeyHandle,
//                                             Index,
//                                             sizeof(KeyName),
//                                             KeyName);
//             if (!NT_SUCCESS(Status))
//                 break;
//
//             TRACE("Index: %lu\n", Index);
//             TRACE("Key name: %S\n", KeyName);
//
//             Status = LsapRegDeleteSubKey(DbObject->KeyHandle,
//                                          KeyName);
//             if (!NT_SUCCESS(Status))
//                 break;
//         }
//
//         if (Status == STATUS_NO_MORE_ENTRIES)
//             Status = STATUS_SUCCESS;
//
//         LsapRegDeleteKey(DbObject->KeyHandle);
//
//         NtClose(DbObject->KeyHandle);
//     }
//
//     if (DbObject->ParentObject != NULL)
//         ParentObject = DbObject->ParentObject;
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, DbObject);
//
//     if (ParentObject != NULL)
//     {
//         ParentObject->RefCount--;
//
//         if (ParentObject->RefCount == 0)
//             Status = LsapCloseDbObject(ParentObject);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapSetObjectAttribute(PLSA_DB_OBJECT DbObject,
//                        LPWSTR AttributeName,
//                        LPVOID AttributeData,
//                        ULONG AttributeSize)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE AttributeKey;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&KeyName,
//                          AttributeName);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                DbObject->KeyHandle,
//                                NULL);
//
//     Status = NtCreateKey(&AttributeKey,
//                          KEY_SET_VALUE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtCreateKey failed for '%S' with status 0x%lx\n",
//             AttributeName, Status);
//         return Status;
//     }
//
//     Status = RtlpNtSetValueKey(AttributeKey,
//                                REG_NONE,
//                                AttributeData,
//                                AttributeSize);
//
//     NtClose(AttributeKey);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("RtlpNtSetValueKey failed for '%S' with status 0x%lx\n",
//             AttributeName, Status);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapGetObjectAttribute(PLSA_DB_OBJECT DbObject,
//                        LPWSTR AttributeName,
//                        LPVOID AttributeData,
//                        PULONG AttributeSize)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE AttributeKey;
//     ULONG ValueSize;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&KeyName,
//                          AttributeName);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                DbObject->KeyHandle,
//                                NULL);
//
//     Status = NtOpenKey(&AttributeKey,
//                        KEY_QUERY_VALUE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     ValueSize = *AttributeSize;
//     Status = RtlpNtQueryValueKey(AttributeKey,
//                                  NULL,
//                                  NULL,
//                                  &ValueSize,
//                                  0);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         goto Done;
//     }
//
//     if (AttributeData == NULL || *AttributeSize == 0)
//     {
//         *AttributeSize = ValueSize;
//         Status = STATUS_SUCCESS;
//         goto Done;
//     }
//     else if (*AttributeSize < ValueSize)
//     {
//         *AttributeSize = ValueSize;
//         Status = STATUS_BUFFER_OVERFLOW;
//         goto Done;
//     }
//
//     Status = RtlpNtQueryValueKey(AttributeKey,
//                                  NULL,
//                                  AttributeData,
//                                  &ValueSize,
//                                  0);
//     if (NT_SUCCESS(Status))
//     {
//         *AttributeSize = ValueSize;
//     }
//
// Done:
//     NtClose(AttributeKey);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapDeleteObjectAttribute(PLSA_DB_OBJECT DbObject,
//                           LPWSTR AttributeName)
// {
//     return LsapRegDeleteSubKey(DbObject->KeyHandle,
//                                AttributeName);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
