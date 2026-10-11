use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ServiceLogonSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut InteractiveSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut NetworkSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut LocalSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut WorldSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut NullSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut LocalServiceSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut NetworkServiceSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut BuiltinDomainSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut LocalSystemSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasUsersSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasAdminsSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AnonymousLogonSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AuthenticatedUserSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasGuestsSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasSystemOpsSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasAccountOpsSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasPowerUsersSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasPrintOpsSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut AliasBackupOpsSid: PSID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut SidData: [SID_DATA; 12] = unsafe {
    [
        _SID_DATA {
            Sid: &raw const NullSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_NULL_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const WorldSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    1 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_WORLD_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const LocalSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    2 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_LOCAL_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const NetworkSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_NETWORK_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const InteractiveSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_INTERACTIVE_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const ServiceLogonSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_SERVICE_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const LocalSystemSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_LOCAL_SYSTEM_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const LocalServiceSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_LOCAL_SERVICE_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const NetworkServiceSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_NETWORK_SERVICE_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const BuiltinDomainSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_BUILTIN_DOMAIN_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const AuthenticatedUserSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_AUTHENTICATED_USER_RID as DWORD,
        },
        _SID_DATA {
            Sid: &raw const AnonymousLogonSid as *mut PSID,
            Authority: _SID_IDENTIFIER_AUTHORITY {
                Value: [
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    0 as ::core::ffi::c_int as BYTE,
                    5 as ::core::ffi::c_int as BYTE,
                ],
            },
            SubAuthority: SECURITY_ANONYMOUS_LOGON_RID as DWORD,
        },
    ]
};

#[no_mangle]
pub static mut DomainSidData: [DOMAIN_SID_DATA; 8] = unsafe {
    [
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasAdminsSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_ADMINS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasUsersSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_USERS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasGuestsSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_GUESTS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasPowerUsersSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_POWER_USERS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasAccountOpsSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_ACCOUNT_OPS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasSystemOpsSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_SYSTEM_OPS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasPrintOpsSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_PRINT_OPS as DWORD,
        },
        _DOMAIN_SID_DATA {
            Sid: &raw const AliasBackupOpsSid as *mut PSID,
            SubAuthority: DOMAIN_ALIAS_RID_BACKUP_OPS as DWORD,
        },
    ]
};

#[no_mangle]
pub static mut g_pSvchostSharedGlobals: PSVCHOST_GLOBAL_DATA =
    ::core::ptr::null::<_SVCHOST_GLOBAL_DATA>() as *mut _SVCHOST_GLOBAL_DATA;

#[no_mangle]
pub static mut g_SvchostInitFlag: DWORD = 0;

#[no_mangle]
pub static mut g_hHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn MemInit(Heap: HANDLE) {
    g_hHeap = Heap;
}

#[no_mangle]
pub unsafe extern "system" fn MemFree(lpMem: LPVOID) -> BOOL {
    return HeapFree(g_hHeap, 0 as DWORD, lpMem as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn MemAlloc(dwFlags: DWORD, dwBytes: DWORD) -> PVOID {
    /* Allocate memory from the heap */
    return HeapAlloc(g_hHeap, dwFlags, dwBytes as SIZE_T);
}

#[no_mangle]
pub unsafe extern "system" fn SvchostBuildSharedGlobals() {
    if !g_pSvchostSharedGlobals.is_null() {
        RtlAssert(
            b"g_pSvchostSharedGlobals == NULL\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"base/services/svchost/globals.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            93 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    /* Is RPC initialized? */
    if (g_SvchostInitFlag & SVCHOST_RPC_INIT_COMPLETE) == 0 {
        /* Nope, go initialize it */
        if !NT_SUCCESS!(RpcpInitRpcServer()) {
            return;
        }

        /* This is now done */
        g_SvchostInitFlag |= SVCHOST_RPC_INIT_COMPLETE;
    }

    /* Is NetBIOS initialized? */
    if (g_SvchostInitFlag & SVCHOST_NBT_INIT_COMPLETE) == 0 {
        /* Nope, set it up */
        SvcNetBiosInit();

        /* This is now done */
        g_SvchostInitFlag |= SVCHOST_NBT_INIT_COMPLETE;
    }

    /* Do we have the global SIDs initialized? */
    if (g_SvchostInitFlag & SVCHOST_SID_INIT_COMPLETE) == 0 {
        /* Create the SIDs we'll export in the global structure */
        if !NT_SUCCESS!(ScCreateWellKnownSids()) {
            return;
        }

        /* This is now done */
        g_SvchostInitFlag |= SVCHOST_SID_INIT_COMPLETE;
    }

    /* Allocate memory for the globals */
    g_pSvchostSharedGlobals = MemAlloc(
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<_SVCHOST_GLOBAL_DATA>() as DWORD,
    ) as PSVCHOST_GLOBAL_DATA;
    if g_pSvchostSharedGlobals.is_null() {
        return;
    }

    /* Write the pointers to the SIDs */
    (*g_pSvchostSharedGlobals).NullSid = NullSid;
    (*g_pSvchostSharedGlobals).WorldSid = WorldSid;
    (*g_pSvchostSharedGlobals).LocalSid = LocalSid;
    (*g_pSvchostSharedGlobals).NetworkSid = NetworkSid;
    (*g_pSvchostSharedGlobals).LocalSystemSid = LocalSystemSid;
    (*g_pSvchostSharedGlobals).LocalServiceSid = LocalServiceSid;
    (*g_pSvchostSharedGlobals).NetworkServiceSid = NetworkServiceSid;
    (*g_pSvchostSharedGlobals).BuiltinDomainSid = BuiltinDomainSid;
    (*g_pSvchostSharedGlobals).AuthenticatedUserSid = AuthenticatedUserSid;
    (*g_pSvchostSharedGlobals).AnonymousLogonSid = AnonymousLogonSid;
    (*g_pSvchostSharedGlobals).AliasAdminsSid = AliasAdminsSid;
    (*g_pSvchostSharedGlobals).AliasUsersSid = AliasUsersSid;
    (*g_pSvchostSharedGlobals).AliasGuestsSid = AliasGuestsSid;
    (*g_pSvchostSharedGlobals).AliasPowerUsersSid = AliasPowerUsersSid;
    (*g_pSvchostSharedGlobals).AliasAccountOpsSid = AliasAccountOpsSid;
    (*g_pSvchostSharedGlobals).AliasSystemOpsSid = AliasSystemOpsSid;
    (*g_pSvchostSharedGlobals).AliasPrintOpsSid = AliasPrintOpsSid;
    (*g_pSvchostSharedGlobals).AliasBackupOpsSid = AliasBackupOpsSid;

    /* Write the pointers to the callbacks */
    (*g_pSvchostSharedGlobals).StartRpcServer = Some(RpcpStartRpcServer);
    (*g_pSvchostSharedGlobals).StopRpcServer = Some(RpcpStopRpcServer);
    (*g_pSvchostSharedGlobals).StopRpcServerEx = Some(RpcpStopRpcServerEx);
    (*g_pSvchostSharedGlobals).NetBiosOpen = Some(SvcNetBiosOpen);
    (*g_pSvchostSharedGlobals).NetBiosClose = Some(SvcNetBiosClose);
    (*g_pSvchostSharedGlobals).NetBiosReset = Some(SvcNetBiosReset);
    (*g_pSvchostSharedGlobals).RegisterStopCallback = Some(SvcRegisterStopCallback);
}

#[no_mangle]
pub unsafe extern "system" fn SvchostCharLowerW(lpSrcStr: LPCWSTR) {
    let mut cchSrc: DWORD;

    /* If there's nothing to do, bail out */
    if lpSrcStr.is_null() {
        return;
    }

    /* Get the length of the input string */
    cchSrc = wcslen(lpSrcStr as *const u16) as DWORD;

    /* Call the locale API to lower-case it */
    if LCMapStringW(
        LANG_USER_DEFAULT,
        LCMAP_LOWERCASE,
        lpSrcStr,
        (cchSrc + 1) as i32,
        lpSrcStr as LPWSTR,
        (cchSrc + 1) as i32,
    ) == FALSE as i32
    {
        DBG_ERR!("SvchostCharLowerW failed for %ws\n",
            lpSrcStr,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn ScDomainIdToSid(
    SourceSid: PSID,
    DomainId: ULONG,
    DestinationSid: *mut PSID,
) -> NTSTATUS {
    let sidCount: ULONG;
    let sidLength: ULONG;
    let mut status: NTSTATUS;

    /* Get the length of the SID based onthe number of subauthorities */
    sidCount = (*RtlSubAuthorityCountSid(SourceSid)) as ULONG;
    sidLength = RtlLengthRequiredSid(sidCount.wrapping_add(1));

    /* Allocate it */
    *DestinationSid = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        sidLength as SIZE_T,
    ) as PSID;
    if !(*DestinationSid).is_null() {
        /* Make a copy of it */
        status = RtlCopySid(sidLength, *DestinationSid, SourceSid);
        if NT_SUCCESS!(status) {
            /* Increase the subauthority count */
            let Count = RtlSubAuthorityCountSid(*DestinationSid);
            *Count = (*Count).wrapping_add(1);

            /* And add the specific domain RID we're creating */
            *RtlSubAuthoritySid(*DestinationSid, sidCount) = DomainId;

            /* Everything worked */
            status = STATUS_SUCCESS;
        } else {
            /* The SID copy failed, so free the SID we just allocated */
            RtlFreeHeap(
                RtlGetProcessHeap!(),
                0,
                *DestinationSid,
            );
        }
    } else {
        /* No space for the SID, bail out */
        status = STATUS_NO_MEMORY;
    }

    /* Return back to the caller */
    return status;
}
#[no_mangle]
pub unsafe extern "system" fn ScAllocateAndInitializeSid(
    Sid: *mut PSID,
    IdentifierAuthority: PSID_IDENTIFIER_AUTHORITY,
    SubAuthorityCount: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* Allocate room for the SID */
    *Sid = RtlAllocateHeap(
        RtlGetProcessHeap!(),
        0,
        RtlLengthRequiredSid(SubAuthorityCount) as SIZE_T,
    ) as PSID;
    if !(*Sid).is_null() {
        /* Initialize it, we're done */
        RtlInitializeSid(*Sid, IdentifierAuthority, SubAuthorityCount as UCHAR);
        Status = STATUS_SUCCESS;
    } else {
        /* No memory, we'll fail */
        Status = STATUS_NO_MEMORY;
    }

    /* Return what happened */
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn ScCreateWellKnownSids() -> NTSTATUS {
    let mut i: ULONG;
    let mut Status: NTSTATUS = 0;
    i = 0;
    while i < RTL_NUMBER_OF!(SidData) as ULONG {
        Status = ScAllocateAndInitializeSid(
            SidData[i as usize].Sid,
            core::ptr::addr_of_mut!(SidData[i as usize].Authority),
            1,
        );
        if !NT_SUCCESS!(Status) {
            DBG_ERR!("ScAllocateAndInitializeSid failed for %u\n",
                i,
            );
            break;
        }
        *RtlSubAuthoritySid(*SidData[i as usize].Sid, 0) =
            SidData[i as usize].SubAuthority as ULONG;
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < RTL_NUMBER_OF!(DomainSidData) as ULONG {
        Status = ScDomainIdToSid(
            BuiltinDomainSid,
            DomainSidData[i as usize].SubAuthority as ULONG,
            DomainSidData[i as usize].Sid,
        );
        if !NT_SUCCESS!(Status) {
            DBG_ERR!("ScDomainIdToSid failed for %u\n",
                i,
            );
            break;
        }
        i = i.wrapping_add(1);
    }
    return if i == RTL_NUMBER_OF!(DomainSidData) as ULONG {
        STATUS_SUCCESS
    } else {
        STATUS_NO_MEMORY
    };
}
