use crate::prelude::*;
// DB object model fns live in the Rust database module (same crate).
use crate::lsasrv::database::{


    LsapOpenDbObject,
    LsapGetObjectAttribute, LsapSetObjectAttribute,
};

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
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

#[no_mangle]
pub unsafe extern "system" fn LsaIOpenPolicyTrusted(PolicyHandle: *mut LSAPR_HANDLE) -> NTSTATUS {
    let mut PolicyObject: PLSA_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"(%p)\n\0".as_ptr() as *const i8,
        PolicyHandle,
    );

    Status = LsapOpenDbObject(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w!("Policy").as_ptr() as *const u16 as LPWSTR,
        _LSA_DB_OBJECT_TYPE_LsaDbPolicyObject,
        POLICY_ALL_ACCESS,
        1u8,
        &raw mut PolicyObject,
    );

    if NT_SUCCESS(Status) {
        *PolicyHandle = PolicyObject as LSAPR_HANDLE;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryAuditLog(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let AuditLogInfo: PPOLICY_AUDIT_LOG_INFO;
    let mut AttributeSize: ULONG = core::mem::size_of::<POLICY_AUDIT_LOG_INFO>() as ULONG;
    let Status: NTSTATUS;

    *PolicyInformation = core::ptr::null_mut();

    AuditLogInfo = MIDL_user_allocate(AttributeSize as usize) as PPOLICY_AUDIT_LOG_INFO;
    if AuditLogInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    const WIDE_POLADTLG: [u16; 9] = [
        0x50, 0x6f, 0x6c, 0x41, 0x64, 0x74, 0x4c, 0x67, 0x0000,
    ];

    Status = LsapGetObjectAttribute(
        PolicyObject,
        WIDE_POLADTLG.as_ptr() as LPWSTR,
        AuditLogInfo as LPVOID,
        &mut AttributeSize,
    );

    if Status >= 0 {
        *PolicyInformation = AuditLogInfo as PLSAPR_POLICY_INFORMATION;
    } else {
        MIDL_user_free(AuditLogInfo as *mut core::ffi::c_void);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryAuditEvents(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let mut AuditData: PLSAP_POLICY_AUDIT_EVENTS_DATA = core::ptr::null_mut();
    let mut p: PLSAPR_POLICY_AUDIT_EVENTS_INFO = core::ptr::null_mut();
    let mut AttributeSize: ULONG;
    let mut Status: NTSTATUS;
    let mut goto_done: bool = false;

    *PolicyInformation = core::ptr::null_mut();

    AttributeSize = 0;
    Status = LsapGetObjectAttribute(
        PolicyObject,
        {
            const POLADTEV: [u16; 9] = [
                0x0050, 0x006F, 0x006C, 0x0041, 0x0064, 0x0074, 0x0045, 0x0076, 0,
            ];
            POLADTEV.as_ptr() as LPWSTR
        },
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(AttributeSize),
    );
    if (Status as i32) < 0 {
        return Status;
    }
    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const i8,
            b"LsarQueryAuditEvents\0".as_ptr() as *const i8,
            88,
            b"Attribute size: %lu\n\0".as_ptr() as *const i8,
            AttributeSize,
        );
    }
    if AttributeSize > 0 {
        AuditData = MIDL_user_allocate(AttributeSize as usize) as PLSAP_POLICY_AUDIT_EVENTS_DATA;
        if AuditData.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        Status = LsapGetObjectAttribute(
            PolicyObject,
            {
                const POLADTEV: [u16; 9] = [
                    0x0050, 0x006F, 0x006C, 0x0041, 0x0064, 0x0074, 0x0045, 0x0076, 0,
                ];
                POLADTEV.as_ptr() as LPWSTR
            },
            AuditData as LPVOID,
            core::ptr::addr_of_mut!(AttributeSize),
        );
        if (Status as i32) < 0 {
            goto_done = true;
        }
    }
    if !goto_done {
        p = MIDL_user_allocate(
            core::mem::size_of::<LSAPR_POLICY_AUDIT_EVENTS_INFO>() as usize,
        ) as PLSAPR_POLICY_AUDIT_EVENTS_INFO;
        if p.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            goto_done = true;
        } else {
            if AuditData.is_null() {
                (*p).AuditingMode = 0u8;
                (*p).MaximumAuditEventCount = 0u32;
                (*p).EventAuditingOptions = core::ptr::null_mut();
            } else {
                (*p).AuditingMode = (*AuditData).AuditingMode;
                (*p).MaximumAuditEventCount = (*AuditData).MaximumAuditEventCount;
                (*p).EventAuditingOptions = MIDL_user_allocate(
                    ((*AuditData).MaximumAuditEventCount as usize)
                        .wrapping_mul(core::mem::size_of::<DWORD>() as usize),
                ) as *mut DWORD;
                if (*p).EventAuditingOptions.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    goto_done = true;
                } else {
                    core::ptr::copy_nonoverlapping(
                        (*AuditData).AuditEvents.as_ptr() as *const DWORD,
                        (*p).EventAuditingOptions,
                        (*AuditData).MaximumAuditEventCount as usize,
                    );
                }
            }
            // On BOTH branches, as the C does (policy.c `*PolicyInformation = p;` after the
            // if/else): with no PolAdtEv data this returned success, a NULL output, and
            // leaked `p`.
            if !goto_done {
                *PolicyInformation = p as PLSAPR_POLICY_INFORMATION;
            }
        }
    }
    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const i8,
            b"LsarQueryAuditEvents\0".as_ptr() as *const i8,
            136,
            b"Status: 0x%lx\n\0".as_ptr() as *const i8,
            Status,
        );
    }
    if (Status as i32) < 0 {
        if !p.is_null() {
            if !(*p).EventAuditingOptions.is_null() {
                MIDL_user_free((*p).EventAuditingOptions as *mut core::ffi::c_void);
            }
            MIDL_user_free(p as *mut core::ffi::c_void);
        }
    }
    if !AuditData.is_null() {
        MIDL_user_free(AuditData as *mut core::ffi::c_void);
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryPrimaryDomain(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let p: PLSAPR_POLICY_PRIMARY_DOM_INFO;
    let domain_name: PUNICODE_STRING;
    let mut attribute_size: ULONG;
    let mut status: NTSTATUS;

    *PolicyInformation = core::ptr::null_mut();

    p = MIDL_user_allocate(core::mem::size_of::<LSAPR_POLICY_PRIMARY_DOM_INFO>()) as PLSAPR_POLICY_PRIMARY_DOM_INFO;
    if p.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = STATUS_SUCCESS;

    'body: {
        // Domain Name
        attribute_size = 0;
        status = LsapGetObjectAttribute(
            PolicyObject,
            w!("PolPrDmN").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            &mut attribute_size,
        );
        if !NT_SUCCESS(status) {
            break 'body;
        }
        if attribute_size > 0 {
            domain_name = MIDL_user_allocate(attribute_size as usize) as PUNICODE_STRING;
            if domain_name.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'body;
            }
            status = LsapGetObjectAttribute(
                PolicyObject,
                w!("PolPrDmN").as_ptr() as LPWSTR,
                domain_name as LPVOID,
                &mut attribute_size,
            );
            if status == STATUS_SUCCESS {
                (*domain_name).Buffer = ((domain_name as usize).wrapping_add((*domain_name).Buffer as usize)) as LPWSTR;
                TRACE!(
                    b"PrimaryDomainName: %wZ\n\0".as_ptr() as *const core::ffi::c_char,
                    domain_name,
                );
                let name_buffer = MIDL_user_allocate((*domain_name).MaximumLength as usize) as LPWSTR;
                if name_buffer.is_null() {
                    MIDL_user_free(domain_name as *mut core::ffi::c_void);
                    status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'body;
                }
                (*p).Name.Buffer = name_buffer;
                (*p).Name.Length = (*domain_name).Length;
                (*p).Name.MaximumLength = (*domain_name).MaximumLength;
                core::ptr::copy_nonoverlapping(
                    (*domain_name).Buffer as *const u8,
                    (*p).Name.Buffer as *mut u8,
                    (*domain_name).MaximumLength as usize,
                );
            }
            MIDL_user_free(domain_name as *mut core::ffi::c_void);
        }

        // Domain SID
        attribute_size = 0;
        status = LsapGetObjectAttribute(
            PolicyObject,
            w!("PolPrDmS").as_ptr() as LPWSTR,
            core::ptr::null_mut(),
            &mut attribute_size,
        );
        if !NT_SUCCESS(status) {
            break 'body;
        }
        if attribute_size > 0 {
            let sid = MIDL_user_allocate(attribute_size as usize);
            if sid.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'body;
            }
            (*p).Sid = sid as PRPC_SID;
            status = LsapGetObjectAttribute(
                PolicyObject,
                w!("PolPrDmS").as_ptr() as LPWSTR,
                sid,
                &mut attribute_size,
            );
        }

        *PolicyInformation = p as PLSAPR_POLICY_INFORMATION;
    }

    if !NT_SUCCESS(status) {
        if !p.is_null() {
            if !(*p).Name.Buffer.is_null() {
                MIDL_user_free((*p).Name.Buffer as *mut core::ffi::c_void);
            }
            if !(*p).Sid.is_null() {
                MIDL_user_free((*p).Sid as *mut core::ffi::c_void);
            }
            MIDL_user_free(p as *mut core::ffi::c_void);
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryPdAccount(
    _PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    *PolicyInformation = core::ptr::null_mut();

    let PdAccountInfo: PLSAPR_POLICY_PD_ACCOUNT_INFO = MIDL_user_allocate(
        core::mem::size_of::<LSAPR_POLICY_PD_ACCOUNT_INFO>() as usize,
    ) as PLSAPR_POLICY_PD_ACCOUNT_INFO;

    if PdAccountInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*PdAccountInfo).Name.Length = 0u16;
    (*PdAccountInfo).Name.MaximumLength = 0u16;
    (*PdAccountInfo).Name.Buffer = core::ptr::null_mut::<WCHAR>();

    *PolicyInformation = PdAccountInfo as PLSAPR_POLICY_INFORMATION;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryAccountDomain(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let p: *mut _LSAPR_POLICY_ACCOUNT_DOM_INFO;
    let DomainName: *mut _UNICODE_STRING;
    let mut AttributeSize: u32 = 0;
    let mut Status: NTSTATUS;

    *PolicyInformation = core::ptr::null_mut();

    p = MIDL_user_allocate(core::mem::size_of::<_LSAPR_POLICY_ACCOUNT_DOM_INFO>()) as *mut _LSAPR_POLICY_ACCOUNT_DOM_INFO;
    if p.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Get Domain Name size
    Status = LsapGetObjectAttribute(
        PolicyObject,
        w!("PolAcDmN").as_ptr() as LPWSTR,
        core::ptr::null_mut(),
        &raw mut AttributeSize,
    );
    if !NT_SUCCESS(Status) {
        // goto Done
        if NT_SUCCESS(Status) {} // dummy break; this is inside 'done block
    } else {
        if AttributeSize > 0 {
            DomainName = MIDL_user_allocate(AttributeSize as usize) as *mut _UNICODE_STRING;
            if DomainName.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                // goto Done (will be handled by outer cleanup)
            } else {
                Status = LsapGetObjectAttribute(
                    PolicyObject,
                    w!("PolAcDmN").as_ptr() as LPWSTR,
                    DomainName as LPVOID,
                    &raw mut AttributeSize,
                );
                if Status == STATUS_SUCCESS {
                    // Adjust self-relative pointer
                    (*DomainName).Buffer = ((DomainName as usize) + ((*DomainName).Buffer as usize)) as PWSTR;

                    TRACE!(b"AccountDomainName: %wZ\n\0".as_ptr() as *const i8, DomainName as *const core::ffi::c_void);

                    (*p).DomainName.Buffer = MIDL_user_allocate((*DomainName).MaximumLength as usize) as LPWSTR;
                    if (*p).DomainName.Buffer.is_null() {
                        MIDL_user_free(DomainName as *mut core::ffi::c_void);
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        // goto Done
                    } else {
                        (*p).DomainName.Length = (*DomainName).Length;
                        (*p).DomainName.MaximumLength = (*DomainName).MaximumLength;
                        core::ptr::copy_nonoverlapping(
                            (*DomainName).Buffer as *const core::ffi::c_void,
                            (*p).DomainName.Buffer as *mut core::ffi::c_void,
                            (*DomainName).MaximumLength as usize,
                        );
                    }
                }
                MIDL_user_free(DomainName as *mut core::ffi::c_void);
            }
        }

        // Domain SID
        if NT_SUCCESS(Status) {
            AttributeSize = 0;
            Status = LsapGetObjectAttribute(
                PolicyObject,
                w!("PolAcDmS").as_ptr() as LPWSTR,
                core::ptr::null_mut(),
                &raw mut AttributeSize,
            );
            if NT_SUCCESS(Status) {
                if AttributeSize > 0 {
                    (*p).Sid = MIDL_user_allocate(AttributeSize as usize) as PRPC_SID;
                    if (*p).Sid.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        // goto Done
                    } else {
                        Status = LsapGetObjectAttribute(
                            PolicyObject,
                            w!("PolAcDmS").as_ptr() as LPWSTR,
                            (*p).Sid as *mut core::ffi::c_void,
                            &raw mut AttributeSize,
                        );
                    }
                }
            }
        }

        if NT_SUCCESS(Status) {
            *PolicyInformation = p as PLSAPR_POLICY_INFORMATION;
        }
    }

    // Done label cleanup
    if !NT_SUCCESS(Status) {
        if !p.is_null() {
            if !(*p).DomainName.Buffer.is_null() {
                MIDL_user_free((*p).DomainName.Buffer as *mut core::ffi::c_void);
            }
            if !(*p).Sid.is_null() {
                MIDL_user_free((*p).Sid as *mut core::ffi::c_void);
            }
            MIDL_user_free(p as *mut core::ffi::c_void);
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryServerRole(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let ServerRoleInfo: PPOLICY_LSA_SERVER_ROLE_INFO;
    let mut AttributeSize: ULONG = 0;
    let mut Status: NTSTATUS;

    *PolicyInformation = core::ptr::null_mut::<_LSAPR_POLICY_INFORMATION>();

    AttributeSize = core::mem::size_of::<_POLICY_LSA_SERVER_ROLE_INFO>() as ULONG;
    ServerRoleInfo = MIDL_user_allocate(AttributeSize as usize) as PPOLICY_LSA_SERVER_ROLE_INFO;
    if ServerRoleInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let wide_str: *const u16 = w!("PolSrvRo").as_ptr() as *const u16;
    Status = LsapGetObjectAttribute(
        PolicyObject,
        wide_str as LPWSTR,
        ServerRoleInfo as LPVOID,
        core::ptr::addr_of_mut!(AttributeSize),
    );
    if Status == STATUS_OBJECT_NAME_NOT_FOUND {
        (*ServerRoleInfo).LsaServerRole = _POLICY_SERVER_ROLE_PolicyServerRolePrimary;
        Status = STATUS_SUCCESS;
    }

    if Status < 0 {
        MIDL_user_free(ServerRoleInfo as *mut core::ffi::c_void);
    } else {
        *PolicyInformation = ServerRoleInfo as PLSAPR_POLICY_INFORMATION;
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryReplicaSource(
    _PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut *mut _LSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const i8,
            b"LsarQueryReplicaSource\0".as_ptr() as *const i8,
            439,
            b"\n\0".as_ptr() as *const i8,
        );
    }
    *PolicyInformation = core::ptr::null_mut::<_LSAPR_POLICY_INFORMATION>();
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryDefaultQuota(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let QuotaInfo: PPOLICY_DEFAULT_QUOTA_INFO;
    let mut AttributeSize: ULONG;
    let Status: NTSTATUS;

    *PolicyInformation = core::ptr::null_mut();

    AttributeSize = core::mem::size_of::<POLICY_DEFAULT_QUOTA_INFO>() as ULONG;
    QuotaInfo = MIDL_user_allocate(AttributeSize as usize) as PPOLICY_DEFAULT_QUOTA_INFO;
    if QuotaInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = LsapGetObjectAttribute(
        PolicyObject,
        [b'D' as u16, b'e' as u16, b'f' as u16, b'Q' as u16, b'u' as u16, b'o' as u16, b't' as u16, b'a' as u16, 0u16].as_ptr() as LPWSTR,
        QuotaInfo as LPVOID,
        &raw mut AttributeSize,
    );

    if Status < 0 {
        MIDL_user_free(QuotaInfo as *mut core::ffi::c_void);
    } else {
        *PolicyInformation = QuotaInfo as PLSAPR_POLICY_INFORMATION;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryModification(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let mut attribute_size = core::mem::size_of::<POLICY_MODIFICATION_INFO>() as ULONG;
    *PolicyInformation = core::ptr::null_mut();
    let info = MIDL_user_allocate(attribute_size as usize) as PPOLICY_MODIFICATION_INFO;
    if info.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let wide_str: &[u16] = &[0x0050, 0x006f, 0x006c, 0x004d, 0x006f, 0x0064, 0x0000];
    let status = LsapGetObjectAttribute(
        PolicyObject,
        wide_str.as_ptr() as LPWSTR,
        info as LPVOID,
        &mut attribute_size,
    );
    if !NT_SUCCESS(status) {
        MIDL_user_free(info as *mut core::ffi::c_void);
    } else {
        *PolicyInformation = info as PLSAPR_POLICY_INFORMATION;
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryAuditFull(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    *PolicyInformation = core::ptr::null_mut::<_LSAPR_POLICY_INFORMATION>();

    let mut attribute_size = core::mem::size_of::<POLICY_AUDIT_FULL_QUERY_INFO>() as u32;
    let audit_full_info =
        MIDL_user_allocate(attribute_size as usize) as *mut _POLICY_AUDIT_FULL_QUERY_INFO;

    if audit_full_info.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let status = LsapGetObjectAttribute(
        PolicyObject,
        w!("PolAdtFl").as_ptr() as *const u16 as LPWSTR,
        audit_full_info as LPVOID,
        core::ptr::addr_of_mut!(attribute_size),
    );

    if status < 0 {
        MIDL_user_free(audit_full_info as *mut core::ffi::c_void);
    } else {
        *PolicyInformation = audit_full_info as PLSAPR_POLICY_INFORMATION;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryDnsDomain(
    PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    let p: *mut _LSAPR_POLICY_DNS_DOMAIN_INFO;
    let mut DomainName: PUNICODE_STRING;
    let mut AttributeSize: u32;
    let mut Status: NTSTATUS;
    let pol_pr_dm_n: *const u16 = core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0P\0r\0D\0m\0N\0\0\0").as_ptr();
    let pol_pr_dm_s: *const u16 = core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0P\0r\0D\0m\0S\0\0\0").as_ptr();
    let pol_dn_dd_n: *const u16 = core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0D\0n\0D\0D\0N\0\0\0").as_ptr();
    let pol_dn_tr_n: *const u16 = core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0D\0n\0T\0r\0N\0\0\0").as_ptr();
    let pol_dn_dm_g: *const u16 = core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0D\0n\0D\0m\0G\0\0\0").as_ptr();

    *PolicyInformation = core::ptr::null_mut();

    p = MIDL_user_allocate(core::mem::size_of::<_LSAPR_POLICY_DNS_DOMAIN_INFO>() as usize) as *mut _LSAPR_POLICY_DNS_DOMAIN_INFO;
    if p.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut cleanup_required = false;
    'main_loop: loop {
        // Primary Domain Name
        AttributeSize = 0;
        Status = LsapGetObjectAttribute(PolicyObject, pol_pr_dm_n as LPWSTR, core::ptr::null_mut(), &mut AttributeSize);
        if Status < 0 {
            cleanup_required = true;
            break 'main_loop;
        }
        if AttributeSize > 0 {
            DomainName = MIDL_user_allocate(AttributeSize as usize) as PUNICODE_STRING;
            if DomainName.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                cleanup_required = true;
                break 'main_loop;
            }
            Status = LsapGetObjectAttribute(PolicyObject, pol_pr_dm_n as LPWSTR, DomainName as LPVOID, &mut AttributeSize);
            if Status == STATUS_SUCCESS {
                (*DomainName).Buffer = (DomainName as usize + (*DomainName).Buffer as usize) as LPWSTR;
                if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0" as *const u8 as *const i8,
                        b"LsarQueryDnsDomain\0" as *const u8 as *const i8,
                        584,
                        b"PrimaryDomainName: %wZ\n\0" as *const u8 as *const i8,
                        DomainName,
                    );
                }
                (*p).Name.Buffer = MIDL_user_allocate((*DomainName).MaximumLength as usize) as LPWSTR;
                if (*p).Name.Buffer.is_null() {
                    MIDL_user_free(DomainName as *mut core::ffi::c_void);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    cleanup_required = true;
                    break 'main_loop;
                }
                (*p).Name.Length = (*DomainName).Length;
                (*p).Name.MaximumLength = (*DomainName).MaximumLength;
                core::ptr::copy_nonoverlapping(
                    (*DomainName).Buffer as *const u8,
                    (*p).Name.Buffer as *mut u8,
                    (*DomainName).MaximumLength as usize,
                );
            }
            MIDL_user_free(DomainName as *mut core::ffi::c_void);
        }

        // Primary Domain SID
        AttributeSize = 0;
        Status = LsapGetObjectAttribute(PolicyObject, pol_pr_dm_s as LPWSTR, core::ptr::null_mut(), &mut AttributeSize);
        if Status < 0 {
            cleanup_required = true;
            break 'main_loop;
        }
        if AttributeSize > 0 {
            (*p).Sid = MIDL_user_allocate(AttributeSize as usize) as PRPC_SID;
            if (*p).Sid.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                cleanup_required = true;
                break 'main_loop;
            }
            LsapGetObjectAttribute(PolicyObject, pol_pr_dm_s as LPWSTR, (*p).Sid as LPVOID, &mut AttributeSize);
        }

        // DNS Domain Name
        AttributeSize = 0;
        Status = LsapGetObjectAttribute(PolicyObject, pol_dn_dd_n as LPWSTR, core::ptr::null_mut(), &mut AttributeSize);
        if Status < 0 {
            cleanup_required = true;
            break 'main_loop;
        }
        if AttributeSize > 0 {
            DomainName = MIDL_user_allocate(AttributeSize as usize) as PUNICODE_STRING;
            if DomainName.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                cleanup_required = true;
                break 'main_loop;
            }
            Status = LsapGetObjectAttribute(PolicyObject, pol_dn_dd_n as LPWSTR, DomainName as LPVOID, &mut AttributeSize);
            if Status == STATUS_SUCCESS {
                (*DomainName).Buffer = (DomainName as usize + (*DomainName).Buffer as usize) as LPWSTR;
                if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0" as *const u8 as *const i8,
                        b"LsarQueryDnsDomain\0" as *const u8 as *const i8,
                        656,
                        b"DNS Domain Name: %wZ\n\0" as *const u8 as *const i8,
                        DomainName,
                    );
                }
                (*p).DnsDomainName.Buffer = MIDL_user_allocate((*DomainName).MaximumLength as usize) as LPWSTR;
                if (*p).DnsDomainName.Buffer.is_null() {
                    MIDL_user_free(DomainName as *mut core::ffi::c_void);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    cleanup_required = true;
                    break 'main_loop;
                }
                (*p).DnsDomainName.Length = (*DomainName).Length;
                (*p).DnsDomainName.MaximumLength = (*DomainName).MaximumLength;
                core::ptr::copy_nonoverlapping(
                    (*DomainName).Buffer as *const u8,
                    (*p).DnsDomainName.Buffer as *mut u8,
                    (*DomainName).MaximumLength as usize,
                );
            }
            MIDL_user_free(DomainName as *mut core::ffi::c_void);
        }

        // DNS Forest Name
        AttributeSize = 0;
        Status = LsapGetObjectAttribute(PolicyObject, pol_dn_tr_n as LPWSTR, core::ptr::null_mut(), &mut AttributeSize);
        if Status < 0 {
            cleanup_required = true;
            break 'main_loop;
        }
        if AttributeSize > 0 {
            DomainName = MIDL_user_allocate(AttributeSize as usize) as PUNICODE_STRING;
            if DomainName.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                cleanup_required = true;
                break 'main_loop;
            }
            Status = LsapGetObjectAttribute(PolicyObject, pol_dn_tr_n as LPWSTR, DomainName as LPVOID, &mut AttributeSize);
            if Status == STATUS_SUCCESS {
                (*DomainName).Buffer = (DomainName as usize + (*DomainName).Buffer as usize) as LPWSTR;
                if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0" as *const u8 as *const i8,
                        b"LsarQueryDnsDomain\0" as *const u8 as *const i8,
                        702,
                        b"DNS Forest Name: %wZ\n\0" as *const u8 as *const i8,
                        DomainName,
                    );
                }
                (*p).DnsForestName.Buffer = MIDL_user_allocate((*DomainName).MaximumLength as usize) as LPWSTR;
                if (*p).DnsForestName.Buffer.is_null() {
                    MIDL_user_free(DomainName as *mut core::ffi::c_void);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    cleanup_required = true;
                    break 'main_loop;
                }
                (*p).DnsForestName.Length = (*DomainName).Length;
                (*p).DnsForestName.MaximumLength = (*DomainName).MaximumLength;
                core::ptr::copy_nonoverlapping(
                    (*DomainName).Buffer as *const u8,
                    (*p).DnsForestName.Buffer as *mut u8,
                    (*DomainName).MaximumLength as usize,
                );
            }
            MIDL_user_free(DomainName as *mut core::ffi::c_void);
        }

        // DNS Domain GUID
        AttributeSize = core::mem::size_of::<GUID>() as u32;
        Status = LsapGetObjectAttribute(PolicyObject, pol_dn_dm_g as LPWSTR, &mut (*p).DomainGuid as *mut _ as LPVOID, &mut AttributeSize);
        if Status < 0 {
            cleanup_required = true;
            break 'main_loop;
        }

        *PolicyInformation = p as PLSAPR_POLICY_INFORMATION;
        break 'main_loop;
    }

    if cleanup_required {
        if !p.is_null() {
            if !(*p).Name.Buffer.is_null() {
                MIDL_user_free((*p).Name.Buffer as *mut core::ffi::c_void);
            }
            if !(*p).DnsDomainName.Buffer.is_null() {
                MIDL_user_free((*p).DnsDomainName.Buffer as *mut core::ffi::c_void);
            }
            if !(*p).DnsForestName.Buffer.is_null() {
                MIDL_user_free((*p).DnsForestName.Buffer as *mut core::ffi::c_void);
            }
            if !(*p).Sid.is_null() {
                MIDL_user_free((*p).Sid as *mut core::ffi::c_void);
            }
            MIDL_user_free(p as *mut core::ffi::c_void);
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryDnsDomainInt(
    _PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0".as_ptr() as PCHAR,
            b"LsarQueryDnsDomainInt\0".as_ptr() as PCHAR,
            762i32,
            b"\n\0".as_ptr() as PCHAR,
        );
    }
    *PolicyInformation = core::ptr::null_mut();
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarQueryLocalAccountDomain(
    _PolicyObject: PLSA_DB_OBJECT,
    PolicyInformation: *mut PLSAPR_POLICY_INFORMATION,
) -> NTSTATUS {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                as *const u8 as *const i8,
            b"LsarQueryLocalAccountDomain\0" as *const u8 as *const i8,
            772i32,
            b"\n\0" as *const u8 as *const i8,
        );
    }
    *PolicyInformation = core::ptr::null_mut::<_LSAPR_POLICY_INFORMATION>();
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetAuditLog(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PPOLICY_AUDIT_LOG_INFO,
) -> NTSTATUS {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const i8,
            b"LsarSetAuditLog\0".as_ptr() as *const i8,
            782i32,
            b"(%p %p)\n\0".as_ptr() as *const i8,
            PolicyObject,
            Info,
        );
    }
    LsapSetObjectAttribute(
        PolicyObject,
        core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0A\0d\0t\0L\0g\0\0\0").as_ptr() as LPWSTR,
        Info as LPVOID,
        core::mem::size_of::<POLICY_AUDIT_LOG_INFO>() as ULONG,
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetAuditEvents(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PLSAPR_POLICY_AUDIT_EVENTS_INFO,
) -> NTSTATUS {
    let AuditData: PLSAP_POLICY_AUDIT_EVENTS_DATA;
    let Status: NTSTATUS;

    let AttributeSize = (core::mem::size_of::<LSAP_POLICY_AUDIT_EVENTS_DATA>()
        + (*Info).MaximumAuditEventCount as usize * core::mem::size_of::<DWORD>())
        as ULONG;

    AuditData = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        AttributeSize as SIZE_T,
    ) as PLSAP_POLICY_AUDIT_EVENTS_DATA;

    if AuditData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*AuditData).AuditingMode = (*Info).AuditingMode;
    (*AuditData).MaximumAuditEventCount = (*Info).MaximumAuditEventCount;

    core::ptr::copy_nonoverlapping(
        (*Info).EventAuditingOptions as *const core::ffi::c_void,
        core::ptr::addr_of_mut!((*AuditData).AuditEvents[0]) as *mut core::ffi::c_void,
        (*Info).MaximumAuditEventCount as usize * core::mem::size_of::<DWORD>(),
    );

    Status = LsapSetObjectAttribute(
        PolicyObject,
        w!("PolAdtEv").as_ptr() as LPWSTR,
        AuditData as LPVOID,
        AttributeSize,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, AuditData as PVOID);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetPrimaryDomain(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PLSAPR_POLICY_PRIMARY_DOM_INFO,
) -> NTSTATUS {
    let Buffer: *mut _UNICODE_STRING;
    let mut Length: ULONG;
    let Ptr: LPWSTR;

    TRACE!(b"(%p %p)\n\0".as_ptr() as *const ::core::ffi::c_char, PolicyObject, Info);

    Length = (core::mem::size_of::<_UNICODE_STRING>() as ULONG)
        .wrapping_add((*Info).Name.MaximumLength as ULONG);

    Buffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        Length as SIZE_T,
    ) as *mut _UNICODE_STRING;

    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Buffer).Length = (*Info).Name.Length;
    (*Buffer).MaximumLength = (*Info).Name.MaximumLength;
    (*Buffer).Buffer = core::mem::size_of::<_UNICODE_STRING>() as LPWSTR as *mut u16;

    Ptr = (Buffer as usize).wrapping_add(core::mem::size_of::<_UNICODE_STRING>() as usize) as LPWSTR;

    core::ptr::copy_nonoverlapping(
        (*Info).Name.Buffer as *const u8,
        Ptr as *mut u8,
        (*Info).Name.MaximumLength as usize,
    );

    static WIDE_POL_PR_DM_N: [u16; 9] = [80, 111, 108, 80, 114, 68, 109, 78, 0];
    let mut Status = LsapSetObjectAttribute(
        PolicyObject,
        &WIDE_POL_PR_DM_N as *const u16 as LPWSTR,
        Buffer as LPVOID,
        Length,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);

    if !NT_SUCCESS(Status) {
        return Status;
    }

    Length = 0;
    if !(*Info).Sid.is_null() {
        Length = RtlLengthSid((*Info).Sid as PSID);
    }

    static WIDE_POL_PR_DM_S: [u16; 9] = [80, 111, 108, 80, 114, 68, 109, 83, 0];
    Status = LsapSetObjectAttribute(
        PolicyObject,
        &WIDE_POL_PR_DM_S as *const u16 as LPWSTR,
        (*Info).Sid as LPVOID,
        Length,
    );

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetAccountDomain(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PLSAPR_POLICY_ACCOUNT_DOM_INFO,
) -> NTSTATUS {
    let mut length;
    let buffer_ptr;

    length = (core::mem::size_of::<_UNICODE_STRING>() as u32)
        .wrapping_add((*Info).DomainName.MaximumLength as u32);

    buffer_ptr = RtlAllocateHeap(RtlGetProcessHeap(), 0, length as SIZE_T);
    if buffer_ptr.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let buffer = buffer_ptr as PUNICODE_STRING;

    (*buffer).Length = (*Info).DomainName.Length;
    (*buffer).MaximumLength = (*Info).DomainName.MaximumLength;
    (*buffer).Buffer = (core::mem::size_of::<_UNICODE_STRING>() as *mut u16) as PWSTR;

    let ptr = (buffer_ptr as usize).wrapping_add(core::mem::size_of::<_UNICODE_STRING>()) as *mut u16;
    core::ptr::copy_nonoverlapping(
        (*Info).DomainName.Buffer as *const u8,
        ptr as *mut u8,
        (*Info).DomainName.MaximumLength as usize,
    );

    let mut status = LsapSetObjectAttribute(
        PolicyObject,
        unsafe { core::ptr::addr_of!(b"P\0o\0l\0A\0c\0D\0m\0N\0\0\0".as_ref()[0]) } as *mut u16,
        buffer as LPVOID,
        length,
    );

    RtlFreeHeap(RtlGetProcessHeap(), 0, buffer_ptr);

    if !NT_SUCCESS(status) {
        return status;
    }

    length = 0;
    if !(*Info).Sid.is_null() {
        length = RtlLengthSid((*Info).Sid as PSID);
    }

    status = LsapSetObjectAttribute(
        PolicyObject,
        unsafe { core::ptr::addr_of!(b"P\0o\0l\0A\0c\0D\0m\0S\0\0\0".as_ref()[0]) } as *mut u16,
        (*Info).Sid as LPBYTE as LPVOID,
        length,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetServerRole(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PPOLICY_LSA_SERVER_ROLE_INFO,
) -> NTSTATUS {
    TRACE!(
        b"(%p %p)\n\0" as *const u8 as *const i8,
        PolicyObject,
        Info,
    );

    LsapSetObjectAttribute(
        PolicyObject,
        core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"P\0o\0l\0S\0r\0v\0R\0o\0\0\0").as_ptr() as LPWSTR,
        Info as LPVOID,
        core::mem::size_of::<POLICY_LSA_SERVER_ROLE_INFO>() as ULONG,
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetReplicaSource(
    _PolicyObject: PLSA_DB_OBJECT,
    _Info: PPOLICY_LSA_REPLICA_SRCE_INFO,
) -> NTSTATUS {
    if ((*__wine_dbch___default).flags as i32 & (1i32 << (__WINE_DBCL_FIXME as i32))) != 0 {
        let _dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let _dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            _dbcl,
            _dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as PCHAR,
            b"LsarSetReplicaSource\0".as_ptr() as PCHAR,
            939i32,
            b"\n\0".as_ptr() as PCHAR,
        );
    }
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetDefaultQuota(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PPOLICY_DEFAULT_QUOTA_INFO,
) -> NTSTATUS {
    if ((*__wine_dbch___default).flags as core::ffi::c_int)
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/lsasrv/policy.c\0".as_ptr() as *const core::ffi::c_char,
            b"LsarSetDefaultQuota\0".as_ptr() as *const core::ffi::c_char,
            948 as core::ffi::c_int,
            b"(%p %p)\n\0".as_ptr() as *const core::ffi::c_char,
            PolicyObject as *const _,
            Info as *const _,
        );
    }

    let def_quota: [u16; 9] = [
        0x44, 0x65, 0x66, 0x51, 0x75, 0x6f, 0x74, 0x61, 0x00,
    ];
    LsapSetObjectAttribute(
        PolicyObject,
        def_quota.as_ptr() as LPWSTR,
        Info as LPVOID,
        core::mem::size_of::<POLICY_DEFAULT_QUOTA_INFO>() as ULONG,
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetModification(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PPOLICY_MODIFICATION_INFO,
) -> NTSTATUS {
    if (*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const i8,
            b"LsarSetModification\0".as_ptr() as *const i8,
            961i32,
            b"(%p %p)\n\0".as_ptr() as *const i8,
            PolicyObject,
            Info,
        );
    }

    const POL_MOD_WIDE: [u16; 7] = [
        0x0050, 0x006F, 0x006C, 0x004D, 0x006F, 0x0064, 0x0000,
    ];

    LsapSetObjectAttribute(
        PolicyObject,
        POL_MOD_WIDE.as_ptr() as LPWSTR,
        Info as *mut core::ffi::c_void,
        core::mem::size_of::<POLICY_MODIFICATION_INFO>() as u32,
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetAuditFull(
    PolicyObject: PLSA_DB_OBJECT,
    Info: PPOLICY_AUDIT_FULL_QUERY_INFO,
) -> NTSTATUS {
    let AuditFullInfo: PPOLICY_AUDIT_FULL_QUERY_INFO;
    let mut AttributeSize: ULONG = core::mem::size_of::<POLICY_AUDIT_FULL_QUERY_INFO>() as ULONG;
    let mut Status: NTSTATUS;

    // TRACE
    TRACE!(
        b"(%p %p)\n\0".as_ptr() as *const i8,
        PolicyObject as *mut _,
        Info as *mut _,
    );

    AuditFullInfo = MIDL_user_allocate(AttributeSize as usize) as PPOLICY_AUDIT_FULL_QUERY_INFO;
    if AuditFullInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    const WSTR_POLADTFL: [u16; 9] = [80, 111, 108, 65, 100, 116, 70, 108, 0]; // L"PolAdtFl"
    Status = LsapGetObjectAttribute(
        PolicyObject,
        WSTR_POLADTFL.as_ptr() as LPWSTR,
        AuditFullInfo as LPVOID,
        &raw mut AttributeSize as *mut ULONG,
    );

    if NT_SUCCESS(Status) {
        (*AuditFullInfo).ShutDownOnFull = (*Info).ShutDownOnFull;
        Status = LsapSetObjectAttribute(
            PolicyObject,
            WSTR_POLADTFL.as_ptr() as LPWSTR,
            AuditFullInfo as LPVOID,
            AttributeSize,
        );
    }

    if !AuditFullInfo.is_null() {
        MIDL_user_free(AuditFullInfo as *mut core::ffi::c_void);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetDnsDomain(
    _PolicyObject: PLSA_DB_OBJECT,
    _Info: PLSAPR_POLICY_DNS_DOMAIN_INFO,
) -> NTSTATUS {
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int)
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int)
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_FIXME,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LsarSetDnsDomain\0" as *const u8 as *const ::core::ffi::c_char,
            1011 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetDnsDomainInt(
    _PolicyObject: PLSA_DB_OBJECT,
    _Info: PLSAPR_POLICY_DNS_DOMAIN_INFO,
) -> NTSTATUS {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 << __WINE_DBCL_FIXME as ::core::ffi::c_int)
        != 0
    {
        let dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            dbcl,
            dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"LsarSetDnsDomainInt\0".as_ptr() as *const ::core::ffi::c_char,
            1020,
            b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn LsarSetLocalAccountDomain(
    _PolicyObject: PLSA_DB_OBJECT,
    _Info: PLSAPR_POLICY_ACCOUNT_DOM_INFO,
) -> NTSTATUS {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_FIXME as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/policy.c\0"
                .as_ptr() as *const i8,
            b"LsarSetLocalAccountDomain\0".as_ptr() as *const i8,
            1029i32,
            b"\n\0".as_ptr() as *const i8,
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/policy.c
//  * PURPOSE:     Policy object routines
//  * COPYRIGHT:   Copyright 2011 Eric Kohl
//  */
//
// #include "lsasrv.h"
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// WINAPI
// LsaIOpenPolicyTrusted(OUT LSAPR_HANDLE *PolicyHandle)
// {
//     PLSA_DB_OBJECT PolicyObject;
//     NTSTATUS Status;
//
//     TRACE("(%p)\n", PolicyHandle);
//
//     Status = LsapOpenDbObject(NULL,
//                               NULL,
//                               L"Policy",
//                               LsaDbPolicyObject,
//                               POLICY_ALL_ACCESS,
//                               TRUE,
//                               &PolicyObject);
//
//     if (NT_SUCCESS(Status))
//         *PolicyHandle = (LSAPR_HANDLE)PolicyObject;
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryAuditLog(PLSA_DB_OBJECT PolicyObject,
//                   PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PPOLICY_AUDIT_LOG_INFO AuditLogInfo = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     AttributeSize = sizeof(POLICY_AUDIT_LOG_INFO);
//     AuditLogInfo = MIDL_user_allocate(AttributeSize);
//     if (AuditLogInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAdtLg",
//                                     AuditLogInfo,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         MIDL_user_free(AuditLogInfo);
//     }
//     else
//     {
//         *PolicyInformation = (PLSAPR_POLICY_INFORMATION)AuditLogInfo;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryAuditEvents(PLSA_DB_OBJECT PolicyObject,
//                      PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PLSAP_POLICY_AUDIT_EVENTS_DATA AuditData = NULL;
//     PLSAPR_POLICY_AUDIT_EVENTS_INFO p = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     *PolicyInformation = NULL;
//
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAdtEv",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     TRACE("Attribute size: %lu\n", AttributeSize);
//     if (AttributeSize > 0)
//     {
//         AuditData = MIDL_user_allocate(AttributeSize);
//         if (AuditData == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolAdtEv",
//                                         AuditData,
//                                         &AttributeSize);
//         if (!NT_SUCCESS(Status))
//             goto done;
//     }
//
//     p = MIDL_user_allocate(sizeof(LSAPR_POLICY_AUDIT_EVENTS_INFO));
//     if (p == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     if (AuditData == NULL)
//     {
//         p->AuditingMode = FALSE;
//         p->MaximumAuditEventCount = 0;
//         p->EventAuditingOptions = NULL;
//     }
//     else
//     {
//         p->AuditingMode = AuditData->AuditingMode;
//         p->MaximumAuditEventCount = AuditData->MaximumAuditEventCount;
//
//         p->EventAuditingOptions = MIDL_user_allocate(AuditData->MaximumAuditEventCount * sizeof(DWORD));
//         if (p->EventAuditingOptions == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         memcpy(p->EventAuditingOptions,
//                &(AuditData->AuditEvents[0]),
//                AuditData->MaximumAuditEventCount * sizeof(DWORD));
//     }
//
//     *PolicyInformation = (PLSAPR_POLICY_INFORMATION)p;
//
// done:
//     TRACE("Status: 0x%lx\n", Status);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (p != NULL)
//         {
//             if (p->EventAuditingOptions != NULL)
//                 MIDL_user_free(p->EventAuditingOptions);
//
//             MIDL_user_free(p);
//         }
//     }
//
//     if (AuditData != NULL)
//         MIDL_user_free(AuditData);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryPrimaryDomain(PLSA_DB_OBJECT PolicyObject,
//                        PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PLSAPR_POLICY_PRIMARY_DOM_INFO p = NULL;
//     PUNICODE_STRING DomainName;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     p = MIDL_user_allocate(sizeof(LSAPR_POLICY_PRIMARY_DOM_INFO));
//     if (p == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Domain Name */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolPrDmN",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Done;
//     }
//
//     if (AttributeSize > 0)
//     {
//         DomainName = MIDL_user_allocate(AttributeSize);
//         if (DomainName == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolPrDmN",
//                                         DomainName,
//                                         &AttributeSize);
//         if (Status == STATUS_SUCCESS)
//         {
//             DomainName->Buffer = (LPWSTR)((ULONG_PTR)DomainName + (ULONG_PTR)DomainName->Buffer);
//
//             TRACE("PrimaryDomainName: %wZ\n", DomainName);
//
//             p->Name.Buffer = MIDL_user_allocate(DomainName->MaximumLength);
//             if (p->Name.Buffer == NULL)
//             {
//                 MIDL_user_free(DomainName);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Done;
//             }
//
//             p->Name.Length = DomainName->Length;
//             p->Name.MaximumLength = DomainName->MaximumLength;
//             memcpy(p->Name.Buffer,
//                    DomainName->Buffer,
//                    DomainName->MaximumLength);
//         }
//
//         MIDL_user_free(DomainName);
//     }
//
//     /* Domain SID */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolPrDmS",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Done;
//     }
//
//     if (AttributeSize > 0)
//     {
//         p->Sid = MIDL_user_allocate(AttributeSize);
//         if (p->Sid == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolPrDmS",
//                                         p->Sid,
//                                         &AttributeSize);
//     }
//
//     *PolicyInformation = (PLSAPR_POLICY_INFORMATION)p;
//
// Done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (p != NULL)
//         {
//             if (p->Name.Buffer)
//                 MIDL_user_free(p->Name.Buffer);
//
//             if (p->Sid)
//                 MIDL_user_free(p->Sid);
//
//             MIDL_user_free(p);
//         }
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryPdAccount(PLSA_DB_OBJECT PolicyObject,
//                    PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PLSAPR_POLICY_PD_ACCOUNT_INFO PdAccountInfo = NULL;
//
//     *PolicyInformation = NULL;
//
//     PdAccountInfo = MIDL_user_allocate(sizeof(LSAPR_POLICY_PD_ACCOUNT_INFO));
//     if (PdAccountInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     PdAccountInfo->Name.Length = 0;
//     PdAccountInfo->Name.MaximumLength = 0;
//     PdAccountInfo->Name.Buffer = NULL;
//
//     *PolicyInformation = (PLSAPR_POLICY_INFORMATION)PdAccountInfo;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsarQueryAccountDomain(PLSA_DB_OBJECT PolicyObject,
//                        PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PLSAPR_POLICY_ACCOUNT_DOM_INFO p = NULL;
//     PUNICODE_STRING DomainName;
//     ULONG AttributeSize = 0;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     p = MIDL_user_allocate(sizeof(LSAPR_POLICY_ACCOUNT_DOM_INFO));
//     if (p == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Domain Name */
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAcDmN",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Done;
//     }
//
//     if (AttributeSize > 0)
//     {
//         DomainName = MIDL_user_allocate(AttributeSize);
//         if (DomainName == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolAcDmN",
//                                         DomainName,
//                                         &AttributeSize);
//         if (Status == STATUS_SUCCESS)
//         {
//             DomainName->Buffer = (LPWSTR)((ULONG_PTR)DomainName + (ULONG_PTR)DomainName->Buffer);
//
//             TRACE("AccountDomainName: %wZ\n", DomainName);
//
//             p->DomainName.Buffer = MIDL_user_allocate(DomainName->MaximumLength);
//             if (p->DomainName.Buffer == NULL)
//             {
//                 MIDL_user_free(DomainName);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Done;
//             }
//
//             p->DomainName.Length = DomainName->Length;
//             p->DomainName.MaximumLength = DomainName->MaximumLength;
//             memcpy(p->DomainName.Buffer,
//                    DomainName->Buffer,
//                    DomainName->MaximumLength);
//         }
//
//         MIDL_user_free(DomainName);
//     }
//
//     /* Domain SID */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAcDmS",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Done;
//     }
//
//     if (AttributeSize > 0)
//     {
//         p->Sid = MIDL_user_allocate(AttributeSize);
//         if (p->Sid == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolAcDmS",
//                                         p->Sid,
//                                         &AttributeSize);
//     }
//
//     *PolicyInformation = (PLSAPR_POLICY_INFORMATION)p;
//
// Done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (p)
//         {
//             if (p->DomainName.Buffer)
//                 MIDL_user_free(p->DomainName.Buffer);
//
//             if (p->Sid)
//                 MIDL_user_free(p->Sid);
//
//             MIDL_user_free(p);
//         }
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryServerRole(PLSA_DB_OBJECT PolicyObject,
//                     PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PPOLICY_LSA_SERVER_ROLE_INFO ServerRoleInfo = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     AttributeSize = sizeof(POLICY_LSA_SERVER_ROLE_INFO);
//     ServerRoleInfo = MIDL_user_allocate(AttributeSize);
//     if (ServerRoleInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolSrvRo",
//                                     ServerRoleInfo,
//                                     &AttributeSize);
//     if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         ServerRoleInfo->LsaServerRole = PolicyServerRolePrimary;
//         Status = STATUS_SUCCESS;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         MIDL_user_free(ServerRoleInfo);
//     }
//     else
//     {
//         *PolicyInformation = (PLSAPR_POLICY_INFORMATION)ServerRoleInfo;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryReplicaSource(PLSA_DB_OBJECT PolicyObject,
//                        PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     FIXME("\n");
//     *PolicyInformation = NULL;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarQueryDefaultQuota(PLSA_DB_OBJECT PolicyObject,
//                       PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PPOLICY_DEFAULT_QUOTA_INFO QuotaInfo = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     AttributeSize = sizeof(POLICY_DEFAULT_QUOTA_INFO);
//     QuotaInfo = MIDL_user_allocate(AttributeSize);
//     if (QuotaInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"DefQuota",
//                                     QuotaInfo,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         MIDL_user_free(QuotaInfo);
//     }
//     else
//     {
//         *PolicyInformation = (PLSAPR_POLICY_INFORMATION)QuotaInfo;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryModification(PLSA_DB_OBJECT PolicyObject,
//                       PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PPOLICY_MODIFICATION_INFO Info = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     AttributeSize = sizeof(POLICY_MODIFICATION_INFO);
//     Info = MIDL_user_allocate(AttributeSize);
//     if (Info == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolMod",
//                                     Info,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         MIDL_user_free(Info);
//     }
//     else
//     {
//         *PolicyInformation = (PLSAPR_POLICY_INFORMATION)Info;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryAuditFull(PLSA_DB_OBJECT PolicyObject,
//                    PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PPOLICY_AUDIT_FULL_QUERY_INFO AuditFullInfo = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     AttributeSize = sizeof(POLICY_AUDIT_FULL_QUERY_INFO);
//     AuditFullInfo = MIDL_user_allocate(AttributeSize);
//     if (AuditFullInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAdtFl",
//                                     AuditFullInfo,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         MIDL_user_free(AuditFullInfo);
//     }
//     else
//     {
//         *PolicyInformation = (PLSAPR_POLICY_INFORMATION)AuditFullInfo;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryDnsDomain(PLSA_DB_OBJECT PolicyObject,
//                    PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     PLSAPR_POLICY_DNS_DOMAIN_INFO p = NULL;
//     PUNICODE_STRING DomainName;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     *PolicyInformation = NULL;
//
//     p = MIDL_user_allocate(sizeof(LSAPR_POLICY_DNS_DOMAIN_INFO));
//     if (p == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Primary Domain Name */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolPrDmN",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         goto done;
//     }
//
//     if (AttributeSize > 0)
//     {
//         DomainName = MIDL_user_allocate(AttributeSize);
//         if (DomainName == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolPrDmN",
//                                         DomainName,
//                                         &AttributeSize);
//         if (Status == STATUS_SUCCESS)
//         {
//             DomainName->Buffer = (LPWSTR)((ULONG_PTR)DomainName + (ULONG_PTR)DomainName->Buffer);
//
//             TRACE("PrimaryDomainName: %wZ\n", DomainName);
//
//             p->Name.Buffer = MIDL_user_allocate(DomainName->MaximumLength);
//             if (p->Name.Buffer == NULL)
//             {
//                 MIDL_user_free(DomainName);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             p->Name.Length = DomainName->Length;
//             p->Name.MaximumLength = DomainName->MaximumLength;
//             memcpy(p->Name.Buffer,
//                    DomainName->Buffer,
//                    DomainName->MaximumLength);
//         }
//
//         MIDL_user_free(DomainName);
//     }
//
//     /* Primary Domain SID */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolPrDmS",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//     {
//         goto done;
//     }
//
//     if (AttributeSize > 0)
//     {
//         p->Sid = MIDL_user_allocate(AttributeSize);
//         if (p->Sid == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolPrDmS",
//                                         p->Sid,
//                                         &AttributeSize);
//     }
//
//     /* DNS Domain Name */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolDnDDN",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (AttributeSize > 0)
//     {
//         DomainName = MIDL_user_allocate(AttributeSize);
//         if (DomainName == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolDnDDN",
//                                         DomainName,
//                                         &AttributeSize);
//         if (Status == STATUS_SUCCESS)
//         {
//             DomainName->Buffer = (LPWSTR)((ULONG_PTR)DomainName + (ULONG_PTR)DomainName->Buffer);
//
//             TRACE("DNS Domain Name: %wZ\n", DomainName);
//
//             p->DnsDomainName.Buffer = MIDL_user_allocate(DomainName->MaximumLength);
//             if (p->DnsDomainName.Buffer == NULL)
//             {
//                 MIDL_user_free(DomainName);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             p->DnsDomainName.Length = DomainName->Length;
//             p->DnsDomainName.MaximumLength = DomainName->MaximumLength;
//             memcpy(p->DnsDomainName.Buffer,
//                    DomainName->Buffer,
//                    DomainName->MaximumLength);
//         }
//
//         MIDL_user_free(DomainName);
//     }
//
//     /* DNS Forest Name */
//     AttributeSize = 0;
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolDnTrN",
//                                     NULL,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     if (AttributeSize > 0)
//     {
//         DomainName = MIDL_user_allocate(AttributeSize);
//         if (DomainName == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         Status = LsapGetObjectAttribute(PolicyObject,
//                                         L"PolDnTrN",
//                                         DomainName,
//                                         &AttributeSize);
//         if (Status == STATUS_SUCCESS)
//         {
//             DomainName->Buffer = (LPWSTR)((ULONG_PTR)DomainName + (ULONG_PTR)DomainName->Buffer);
//
//             TRACE("DNS Forest Name: %wZ\n", DomainName);
//
//             p->DnsForestName.Buffer = MIDL_user_allocate(DomainName->MaximumLength);
//             if (p->DnsForestName.Buffer == NULL)
//             {
//                 MIDL_user_free(DomainName);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto done;
//             }
//
//             p->DnsForestName.Length = DomainName->Length;
//             p->DnsForestName.MaximumLength = DomainName->MaximumLength;
//             memcpy(p->DnsForestName.Buffer,
//                    DomainName->Buffer,
//                    DomainName->MaximumLength);
//         }
//
//         MIDL_user_free(DomainName);
//     }
//
//     /* DNS Domain GUID */
//     AttributeSize = sizeof(GUID);
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolDnDmG",
//                                     &(p->DomainGuid),
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     *PolicyInformation = (PLSAPR_POLICY_INFORMATION)p;
//
// done:
//     if (!NT_SUCCESS(Status))
//     {
//         if (p)
//         {
//             if (p->Name.Buffer)
//                 MIDL_user_free(p->Name.Buffer);
//
//             if (p->DnsDomainName.Buffer)
//                 MIDL_user_free(p->DnsDomainName.Buffer);
//
//             if (p->DnsForestName.Buffer)
//                 MIDL_user_free(p->DnsForestName.Buffer);
//
//             if (p->Sid)
//                 MIDL_user_free(p->Sid);
//
//             MIDL_user_free(p);
//         }
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarQueryDnsDomainInt(PLSA_DB_OBJECT PolicyObject,
//                       PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     FIXME("\n");
//     *PolicyInformation = NULL;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarQueryLocalAccountDomain(PLSA_DB_OBJECT PolicyObject,
//                             PLSAPR_POLICY_INFORMATION *PolicyInformation)
// {
//     FIXME("\n");
//     *PolicyInformation = NULL;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarSetAuditLog(PLSA_DB_OBJECT PolicyObject,
//                 PPOLICY_AUDIT_LOG_INFO Info)
// {
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     return LsapSetObjectAttribute(PolicyObject,
//                                   L"PolAdtLg",
//                                   Info,
//                                   sizeof(POLICY_AUDIT_LOG_INFO));
// }
//
//
// NTSTATUS
// LsarSetAuditEvents(PLSA_DB_OBJECT PolicyObject,
//                    PLSAPR_POLICY_AUDIT_EVENTS_INFO Info)
// {
//     PLSAP_POLICY_AUDIT_EVENTS_DATA AuditData = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     AttributeSize = sizeof(LSAP_POLICY_AUDIT_EVENTS_DATA) +
//                     Info->MaximumAuditEventCount * sizeof(DWORD);
//
//     AuditData = RtlAllocateHeap(RtlGetProcessHeap(),
//                                 HEAP_ZERO_MEMORY,
//                                 AttributeSize);
//     if (AuditData == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     AuditData->AuditingMode = Info->AuditingMode;
//     AuditData->MaximumAuditEventCount = Info->MaximumAuditEventCount;
//
//     memcpy(&(AuditData->AuditEvents[0]),
//            Info->EventAuditingOptions,
//            Info->MaximumAuditEventCount * sizeof(DWORD));
//
//     Status = LsapSetObjectAttribute(PolicyObject,
//                                     L"PolAdtEv",
//                                     AuditData,
//                                     AttributeSize);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, AuditData);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarSetPrimaryDomain(PLSA_DB_OBJECT PolicyObject,
//                      PLSAPR_POLICY_PRIMARY_DOM_INFO Info)
// {
//     PUNICODE_STRING Buffer;
//     ULONG Length = 0;
//     NTSTATUS Status;
//     LPWSTR Ptr;
//
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     Length = sizeof(UNICODE_STRING) + Info->Name.MaximumLength;
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                              0,
//                              Length);
//     if (Buffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Buffer->Length = Info->Name.Length;
//     Buffer->MaximumLength = Info->Name.MaximumLength;
//     Buffer->Buffer = (LPWSTR)sizeof(UNICODE_STRING);
//     Ptr = (LPWSTR)((ULONG_PTR)Buffer + sizeof(UNICODE_STRING));
//     memcpy(Ptr, Info->Name.Buffer, Info->Name.MaximumLength);
//
//     Status = LsapSetObjectAttribute(PolicyObject,
//                                     L"PolPrDmN",
//                                     Buffer,
//                                     Length);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Length = 0;
//     if (Info->Sid != NULL)
//         Length = RtlLengthSid(Info->Sid);
//
//     Status = LsapSetObjectAttribute(PolicyObject,
//                                     L"PolPrDmS",
//                                     (LPBYTE)Info->Sid,
//                                     Length);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarSetAccountDomain(PLSA_DB_OBJECT PolicyObject,
//                      PLSAPR_POLICY_ACCOUNT_DOM_INFO Info)
// {
//     PUNICODE_STRING Buffer;
//     ULONG Length = 0;
//     NTSTATUS Status;
//     LPWSTR Ptr;
//
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     Length = sizeof(UNICODE_STRING) + Info->DomainName.MaximumLength;
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                              0,
//                              Length);
//     if (Buffer == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Buffer->Length = Info->DomainName.Length;
//     Buffer->MaximumLength = Info->DomainName.MaximumLength;
//     Buffer->Buffer = (LPWSTR)sizeof(UNICODE_STRING);
//     Ptr = (LPWSTR)((ULONG_PTR)Buffer + sizeof(UNICODE_STRING));
//     memcpy(Ptr, Info->DomainName.Buffer, Info->DomainName.MaximumLength);
//
//     Status = LsapSetObjectAttribute(PolicyObject,
//                                     L"PolAcDmN",
//                                     Buffer,
//                                     Length);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     Length = 0;
//     if (Info->Sid != NULL)
//         Length = RtlLengthSid(Info->Sid);
//
//     Status = LsapSetObjectAttribute(PolicyObject,
//                                     L"PolAcDmS",
//                                     (LPBYTE)Info->Sid,
//                                     Length);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarSetServerRole(PLSA_DB_OBJECT PolicyObject,
//                   PPOLICY_LSA_SERVER_ROLE_INFO Info)
// {
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     return LsapSetObjectAttribute(PolicyObject,
//                                   L"PolSrvRo",
//                                   Info,
//                                   sizeof(POLICY_LSA_SERVER_ROLE_INFO));
// }
//
//
// NTSTATUS
// LsarSetReplicaSource(PLSA_DB_OBJECT PolicyObject,
//                      PPOLICY_LSA_REPLICA_SRCE_INFO Info)
// {
//     FIXME("\n");
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarSetDefaultQuota(PLSA_DB_OBJECT PolicyObject,
//                     PPOLICY_DEFAULT_QUOTA_INFO Info)
// {
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     return LsapSetObjectAttribute(PolicyObject,
//                                   L"DefQuota",
//                                   Info,
//                                   sizeof(POLICY_DEFAULT_QUOTA_INFO));
// }
//
//
// NTSTATUS
// LsarSetModification(PLSA_DB_OBJECT PolicyObject,
//                     PPOLICY_MODIFICATION_INFO Info)
// {
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     return LsapSetObjectAttribute(PolicyObject,
//                                   L"PolMod",
//                                   Info,
//                                   sizeof(POLICY_MODIFICATION_INFO));
// }
//
//
// NTSTATUS
// LsarSetAuditFull(PLSA_DB_OBJECT PolicyObject,
//                  PPOLICY_AUDIT_FULL_QUERY_INFO Info)
// {
//     PPOLICY_AUDIT_FULL_QUERY_INFO AuditFullInfo = NULL;
//     ULONG AttributeSize;
//     NTSTATUS Status;
//
//     TRACE("(%p %p)\n", PolicyObject, Info);
//
//     AttributeSize = sizeof(POLICY_AUDIT_FULL_QUERY_INFO);
//     AuditFullInfo = MIDL_user_allocate(AttributeSize);
//     if (AuditFullInfo == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = LsapGetObjectAttribute(PolicyObject,
//                                     L"PolAdtFl",
//                                     AuditFullInfo,
//                                     &AttributeSize);
//     if (!NT_SUCCESS(Status))
//         goto done;
//
//     AuditFullInfo->ShutDownOnFull = Info->ShutDownOnFull;
//
//     Status = LsapSetObjectAttribute(PolicyObject,
//                                     L"PolAdtFl",
//                                     AuditFullInfo,
//                                     AttributeSize);
//
// done:
//     if (AuditFullInfo != NULL)
//         MIDL_user_free(AuditFullInfo);
//
//     return Status;
// }
//
//
// NTSTATUS
// LsarSetDnsDomain(PLSA_DB_OBJECT PolicyObject,
//                  PLSAPR_POLICY_DNS_DOMAIN_INFO Info)
// {
//     FIXME("\n");
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarSetDnsDomainInt(PLSA_DB_OBJECT PolicyObject,
//                     PLSAPR_POLICY_DNS_DOMAIN_INFO Info)
// {
//     FIXME("\n");
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// LsarSetLocalAccountDomain(PLSA_DB_OBJECT PolicyObject,
//                           PLSAPR_POLICY_ACCOUNT_DOM_INFO Info)
// {
//     FIXME("\n");
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
