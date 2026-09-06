use crate::prelude::*;
// Rust-sibling init fns (same crate) — import for exact signatures.
use crate::lsasrv::database::LsapInitDatabase;
use crate::lsasrv::session::LsapInitLogonSessions;
use crate::lsasrv::notify::LsapInitNotificationList;
use crate::lsasrv::srm::LsapRmInitializeServer;
// Still-C init fns (lookup.c/authpackage.c/lsarpc.c/authport.c) — resolve at link.
extern "C" {
    fn LsapInitSids() -> NTSTATUS;
    fn LsapInitAuthPackages() -> NTSTATUS;
    fn LsarStartRpcServer() -> NTSTATUS;
    fn StartAuthenticationPort() -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// Cross-file global (lsasrv.h extern; lsarpc.c et al. read it) -- export unmangled.
#[no_mangle]
pub static mut LsapProductType: NT_PRODUCT_TYPE = NtProductWinNt;

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
pub unsafe extern "system" fn LsaIFree_LSAPR_ACCOUNT_ENUM_BUFFER(
    Ptr: PLSAPR_ACCOUNT_ENUM_BUFFER,
) {
    if Ptr.is_null() {
        return;
    }
    if !(*Ptr).Information.is_null() {
        let entries_read = (*Ptr).EntriesRead;
        for i in 0..entries_read {
            let sid_ptr = (*((*Ptr).Information.add(i as usize))).Sid;
            MIDL_user_free(sid_ptr as *mut core::ffi::c_void);
        }
        MIDL_user_free((*Ptr).Information as *mut core::ffi::c_void);
    }
    MIDL_user_free(Ptr as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_CR_CIPHER_VALUE(
    Ptr: PLSAPR_CR_CIPHER_VALUE,
) {
    if !Ptr.is_null() {
        MIDL_user_free(Ptr as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_POLICY_INFORMATION(
    InformationClass: POLICY_INFORMATION_CLASS,
    PolicyInformation: PLSAPR_POLICY_INFORMATION,
) {
    if !PolicyInformation.is_null() {
        match InformationClass as u32 {
            2 => {
                if !(*PolicyInformation)
                    .PolicyAuditEventsInfo
                    .EventAuditingOptions
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyAuditEventsInfo
                            .EventAuditingOptions
                            as *mut core::ffi::c_void,
                    );
                }
            }
            3 => {
                if !(*PolicyInformation)
                    .PolicyPrimaryDomInfo
                    .Name
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyPrimaryDomInfo.Name.Buffer
                            as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation).PolicyPrimaryDomInfo.Sid.is_null() {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyPrimaryDomInfo.Sid as *mut core::ffi::c_void,
                    );
                }
            }
            4 => {
                if !(*PolicyInformation)
                    .PolicyPdAccountInfo
                    .Name
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyPdAccountInfo.Name.Buffer
                            as *mut core::ffi::c_void,
                    );
                }
            }
            5 => {
                if !(*PolicyInformation)
                    .PolicyAccountDomainInfo
                    .DomainName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyAccountDomainInfo
                            .DomainName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation).PolicyAccountDomainInfo.Sid.is_null() {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyAccountDomainInfo.Sid
                            as *mut core::ffi::c_void,
                    );
                }
            }
            7 => {
                if !(*PolicyInformation)
                    .PolicyReplicaSourceInfo
                    .ReplicaSource
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyReplicaSourceInfo
                            .ReplicaSource
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation)
                    .PolicyReplicaSourceInfo
                    .ReplicaAccountName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyReplicaSourceInfo
                            .ReplicaAccountName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
            }
            1 | 6 | 8 | 9 | 10 | 11 => {}
            12 => {
                if !(*PolicyInformation)
                    .PolicyDnsDomainInfo
                    .Name
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyDnsDomainInfo.Name.Buffer
                            as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation)
                    .PolicyDnsDomainInfo
                    .DnsDomainName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyDnsDomainInfo
                            .DnsDomainName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation)
                    .PolicyDnsDomainInfo
                    .DnsForestName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyDnsDomainInfo
                            .DnsForestName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation).PolicyDnsDomainInfo.Sid.is_null() {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyDnsDomainInfo.Sid as *mut core::ffi::c_void,
                    );
                }
            }
            13 => {
                if !(*PolicyInformation)
                    .PolicyDnsDomainInfoInt
                    .Name
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyDnsDomainInfoInt.Name.Buffer
                            as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation)
                    .PolicyDnsDomainInfoInt
                    .DnsDomainName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyDnsDomainInfoInt
                            .DnsDomainName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation)
                    .PolicyDnsDomainInfoInt
                    .DnsForestName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyDnsDomainInfoInt
                            .DnsForestName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation).PolicyDnsDomainInfoInt.Sid.is_null() {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyDnsDomainInfoInt.Sid as *mut core::ffi::c_void,
                    );
                }
            }
            14 => {
                if !(*PolicyInformation)
                    .PolicyLocalAccountDomainInfo
                    .DomainName
                    .Buffer
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation)
                            .PolicyLocalAccountDomainInfo
                            .DomainName
                            .Buffer as *mut core::ffi::c_void,
                    );
                }
                if !(*PolicyInformation)
                    .PolicyLocalAccountDomainInfo
                    .Sid
                    .is_null()
                {
                    MIDL_user_free(
                        (*PolicyInformation).PolicyLocalAccountDomainInfo.Sid
                            as *mut core::ffi::c_void,
                    );
                }
            }
            _ => {
                if (*__wine_dbch___default).flags as i32
                    & (1i32 << __WINE_DBCL_ERR as i32)
                    != 0
                {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/lsasrv.c\0"
                            as *const u8 as *const i8,
                        b"LsaIFree_LSAPR_POLICY_INFORMATION\0" as *const u8 as *const i8,
                        147i32,
                        b"Invalid InformationClass: %lu\n\0" as *const u8 as *const i8,
                        InformationClass as u32,
                    );
                }
            }
        }
        MIDL_user_free(PolicyInformation as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_PRIVILEGE_ENUM_BUFFER(
    Ptr: *mut _LSAPR_PRIVILEGE_ENUM_BUFFER,
) {
    if Ptr.is_null() {
        return;
    }
    if !(*Ptr).Privileges.is_null() {
        let mut i: u32 = 0;
        while i < (*Ptr).Entries {
            if !(*(*Ptr).Privileges.add(i as usize)).Name.Buffer.is_null() {
                MIDL_user_free((*(*Ptr).Privileges.add(i as usize)).Name.Buffer as PVOID);
            }
            i = i.wrapping_add(1);
        }
        MIDL_user_free((*Ptr).Privileges as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_PRIVILEGE_SET(
    Ptr: PLSAPR_PRIVILEGE_SET,
) {
    if !Ptr.is_null() {
        MIDL_user_free(Ptr as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_REFERENCED_DOMAIN_LIST(
    Ptr: PLSAPR_REFERENCED_DOMAIN_LIST,
) {
    if Ptr.is_null() {
        return;
    }
    if !(*Ptr).Domains.is_null() {
        let mut i: u32 = 0;
        while i < (*Ptr).Entries {
            let domain = (*Ptr).Domains.offset(i as isize);
            if !(*domain).Name.Buffer.is_null() {
                MIDL_user_free((*domain).Name.Buffer as PVOID);
            }
            if !(*domain).Sid.is_null() {
                MIDL_user_free((*domain).Sid as PVOID);
            }
            i += 1;
        }
        MIDL_user_free((*Ptr).Domains as PVOID);
    }
    MIDL_user_free(Ptr as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_SR_SECURITY_DESCRIPTOR(
    Ptr: *mut _LSAPR_SR_SECURITY_DESCRIPTOR,
) {
    if !Ptr.is_null() {
        if !(*Ptr).SecurityDescriptor.is_null() {
            MIDL_user_free((*Ptr).SecurityDescriptor as *mut core::ffi::c_void);
        }
        MIDL_user_free(Ptr as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_TRANSLATED_NAMES(
    Ptr: PLSAPR_TRANSLATED_NAMES,
) {
    if !Ptr.is_null() {
        if !(*Ptr).Names.is_null() {
            for i in 0..(*Ptr).Entries {
                let buffer = (*(*Ptr).Names.add(i as usize)).Name.Buffer;
                if !buffer.is_null() {
                    MIDL_user_free(buffer as *mut core::ffi::c_void);
                }
            }
            MIDL_user_free((*Ptr).Names as *mut core::ffi::c_void);
        }
        MIDL_user_free(Ptr as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsaIFree_LSAPR_TRANSLATED_SIDS(
    Ptr: PLSAPR_TRANSLATED_SIDS,
) {
    if !Ptr.is_null() {
        if !(*Ptr).Sids.is_null() {
            MIDL_user_free((*Ptr).Sids as *mut core::ffi::c_void);
        }
        MIDL_user_free(Ptr as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LsapInitLsa() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut PrivilegeEnabled: u8 = 0;

    TRACE!(b"LsapInitLsa()\n\0".as_ptr() as *const i8);

    RtlGetNtProductType(core::ptr::addr_of_mut!(LsapProductType));

    LsapInitSids();

    Status = LsapRmInitializeServer();
    if !NT_SUCCESS(Status) {
        ERR!("LsapRmInitializeServer() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    LsapInitDatabase();
    LsapInitLogonSessions();
    LsapInitNotificationList();

    Status = LsapInitAuthPackages();
    if !NT_SUCCESS(Status) {
        ERR!("LsapInitAuthPackages() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    Status = RtlAdjustPrivilege(SE_CREATE_TOKEN_PRIVILEGE as ULONG, 1u8, 0u8, &mut PrivilegeEnabled);
    if !NT_SUCCESS(Status) {
        ERR!("RtlAdjustPrivilege(SE_CREATE_TOKEN_PRIVILEGE) failed, ignoring (Status 0x%08lx)\n", Status);
    }

    Status = StartAuthenticationPort();
    if !NT_SUCCESS(Status) {
        ERR!("StartAuthenticationPort() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    Status = LsarStartRpcServer();
    if !NT_SUCCESS(Status) {
        ERR!("LsarStartRpcServer() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: usize) -> *mut core::ffi::c_void {
    RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, len as SIZE_T)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: PVOID) {
    RtlFreeHeap(RtlGetProcessHeap(), 0, ptr);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/lsasrv.c
//  * PURPOSE:     Main file
//  * COPYRIGHT:   Copyright 2006-2009 Eric Kohl
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "lsasrv.h"
//
// NT_PRODUCT_TYPE LsapProductType = NtProductWinNt;
//
// /* FUNCTIONS ***************************************************************/
//
// VOID
// NTAPI
// LsaIFree_LSAPR_ACCOUNT_ENUM_BUFFER(
//     IN PLSAPR_ACCOUNT_ENUM_BUFFER Ptr)
// {
//     ULONG i;
//
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Information != NULL)
//     {
//         for (i = 0; i < Ptr->EntriesRead; i++)
//             MIDL_user_free(Ptr->Information[i].Sid);
//
//         MIDL_user_free(Ptr->Information);
//     }
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_CR_CIPHER_VALUE(
//     IN PLSAPR_CR_CIPHER_VALUE Ptr)
// {
//     if (Ptr != NULL)
//         MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_POLICY_INFORMATION(IN POLICY_INFORMATION_CLASS InformationClass,
//                                   IN PLSAPR_POLICY_INFORMATION PolicyInformation)
// {
//     if (PolicyInformation != NULL)
//     {
//         switch (InformationClass)
//         {
//             case PolicyAuditLogInformation:      /* 1 */
//                 break;
//
//             case PolicyAuditEventsInformation:   /* 2 */
//                 if (PolicyInformation->PolicyAuditEventsInfo.EventAuditingOptions != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyAuditEventsInfo.EventAuditingOptions);
//                 break;
//
//             case PolicyPrimaryDomainInformation: /* 3 */
//                 if (PolicyInformation->PolicyPrimaryDomInfo.Name.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyPrimaryDomInfo.Name.Buffer);
//
//                 if (PolicyInformation->PolicyPrimaryDomInfo.Sid != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyPrimaryDomInfo.Sid);
//                 break;
//
//             case PolicyPdAccountInformation:     /* 4 */
//                 if (PolicyInformation->PolicyPdAccountInfo.Name.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyPdAccountInfo.Name.Buffer);
//                 break;
//
//             case PolicyAccountDomainInformation: /* 5 */
//                 if (PolicyInformation->PolicyAccountDomainInfo.DomainName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyAccountDomainInfo.DomainName.Buffer);
//
//                 if (PolicyInformation->PolicyAccountDomainInfo.Sid != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyAccountDomainInfo.Sid);
//                 break;
//
//             case PolicyLsaServerRoleInformation: /* 6 */
//                 break;
//
//             case PolicyReplicaSourceInformation: /* 7 */
//                 if (PolicyInformation->PolicyReplicaSourceInfo.ReplicaSource.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyReplicaSourceInfo.ReplicaSource.Buffer);
//
//                 if (PolicyInformation->PolicyReplicaSourceInfo.ReplicaAccountName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyReplicaSourceInfo.ReplicaAccountName.Buffer);
//                 break;
//
//             case PolicyDefaultQuotaInformation:  /* 8 */
//                 break;
//
//             case PolicyModificationInformation:  /* 9 */
//                 break;
//
//             case PolicyAuditFullSetInformation:  /* 10 (0xA) */
//                 break;
//
//             case PolicyAuditFullQueryInformation: /* 11 (0xB) */
//                 break;
//
//             case PolicyDnsDomainInformation:      /* 12 (0xC) */
//                 if (PolicyInformation->PolicyDnsDomainInfo.Name.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfo.Name.Buffer);
//
//                 if (PolicyInformation->PolicyDnsDomainInfo.DnsDomainName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfo.DnsDomainName.Buffer);
//
//                 if (PolicyInformation->PolicyDnsDomainInfo.DnsForestName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfo.DnsForestName.Buffer);
//
//                 if (PolicyInformation->PolicyDnsDomainInfo.Sid != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfo.Sid);
//                 break;
//
//             case PolicyDnsDomainInformationInt:   /* 13 (0xD) */
//                 if (PolicyInformation->PolicyDnsDomainInfoInt.Name.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfoInt.Name.Buffer);
//
//                 if (PolicyInformation->PolicyDnsDomainInfoInt.DnsDomainName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfoInt.DnsDomainName.Buffer);
//
//                 if (PolicyInformation->PolicyDnsDomainInfoInt.DnsForestName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfoInt.DnsForestName.Buffer);
//
//                 if (PolicyInformation->PolicyDnsDomainInfoInt.Sid != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyDnsDomainInfoInt.Sid);
//                 break;
//
//             case PolicyLocalAccountDomainInformation: /* 14 (0xE) */
//                 if (PolicyInformation->PolicyLocalAccountDomainInfo.DomainName.Buffer != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyLocalAccountDomainInfo.DomainName.Buffer);
//
//                 if (PolicyInformation->PolicyLocalAccountDomainInfo.Sid != NULL)
//                     MIDL_user_free(PolicyInformation->PolicyLocalAccountDomainInfo.Sid);
//                 break;
//
//             default:
//                 ERR("Invalid InformationClass: %lu\n", InformationClass);
//                 break;
//         }
//
//         MIDL_user_free(PolicyInformation);
//     }
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_PRIVILEGE_ENUM_BUFFER(
//     IN PLSAPR_PRIVILEGE_ENUM_BUFFER Ptr)
// {
//     ULONG i;
//
//     if (Ptr != NULL)
//     {
//         if (Ptr->Privileges != NULL)
//         {
//             for (i = 0; i < Ptr->Entries; i++)
//             {
//                 if (Ptr->Privileges[i].Name.Buffer != NULL)
//                     MIDL_user_free(Ptr->Privileges[i].Name.Buffer);
//             }
//
//             MIDL_user_free(Ptr->Privileges);
//         }
//     }
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_PRIVILEGE_SET(IN PLSAPR_PRIVILEGE_SET Ptr)
// {
//     if (Ptr != NULL)
//     {
//         MIDL_user_free(Ptr);
//     }
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_REFERENCED_DOMAIN_LIST(
//     IN PLSAPR_REFERENCED_DOMAIN_LIST Ptr)
// {
//     ULONG i;
//
//     if (Ptr != NULL)
//     {
//         if (Ptr->Domains != NULL)
//         {
//             for (i = 0; i < Ptr->Entries; i++)
//             {
//                 if (Ptr->Domains[i].Name.Buffer != NULL)
//                      MIDL_user_free(Ptr->Domains[i].Name.Buffer);
//
//                 if (Ptr->Domains[i].Sid != NULL)
//                     MIDL_user_free(Ptr->Domains[i].Sid);
//             }
//
//             MIDL_user_free(Ptr->Domains);
//         }
//
//         MIDL_user_free(Ptr);
//     }
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_SR_SECURITY_DESCRIPTOR(
//     IN PLSAPR_SR_SECURITY_DESCRIPTOR Ptr)
// {
//     if (Ptr != NULL)
//     {
//         if (Ptr->SecurityDescriptor != NULL)
//             MIDL_user_free(Ptr->SecurityDescriptor);
//
//         MIDL_user_free(Ptr);
//     }
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_TRANSLATED_NAMES(
//     IN PLSAPR_TRANSLATED_NAMES Ptr)
// {
//     ULONG i;
//
//     if (Ptr != NULL)
//     {
//         if (Ptr->Names != NULL)
//         {
//             for (i = 0; i < Ptr->Entries; i++)
//             {
//                 if (Ptr->Names[i].Name.Buffer != NULL)
//                     MIDL_user_free(Ptr->Names[i].Name.Buffer);
//             }
//
//             MIDL_user_free(Ptr->Names);
//         }
//
//         MIDL_user_free(Ptr);
//     }
// }
//
//
// VOID
// NTAPI
// LsaIFree_LSAPR_TRANSLATED_SIDS(
//     IN PLSAPR_TRANSLATED_SIDS Ptr)
// {
//     if (Ptr != NULL)
//     {
//         if (Ptr->Sids != NULL)
//             MIDL_user_free(Ptr->Sids);
//
//         MIDL_user_free(Ptr);
//     }
// }
//
//
// NTSTATUS
// WINAPI
// LsapInitLsa(VOID)
// {
//     NTSTATUS Status;
//     BOOLEAN PrivilegeEnabled;
//
//     TRACE("LsapInitLsa()\n");
//
//     /* Get the product type */
//     RtlGetNtProductType(&LsapProductType);
//
//     /* Initialize the well known SIDs */
//     LsapInitSids();
//
//     /* Initialize the SRM server */
//     Status = LsapRmInitializeServer();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapRmInitializeServer() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Initialize the LSA database */
//     LsapInitDatabase();
//
//     /* Initialize logon sessions */
//     LsapInitLogonSessions();
//
//     /* Initialize the notification list */
//     LsapInitNotificationList();
//
//     /* Initialize registered authentication packages */
//     Status = LsapInitAuthPackages();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsapInitAuthPackages() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Enable the token creation privilege for the rest of our lifetime */
//     Status = RtlAdjustPrivilege(SE_CREATE_TOKEN_PRIVILEGE, TRUE, FALSE, &PrivilegeEnabled);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("RtlAdjustPrivilege(SE_CREATE_TOKEN_PRIVILEGE) failed, ignoring (Status 0x%08lx)\n", Status);
//     }
//
//     /* Start the authentication LPC port thread */
//     Status = StartAuthenticationPort();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("StartAuthenticationPort() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Start the RPC server */
//     Status = LsarStartRpcServer();
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsarStartRpcServer() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// void __RPC_FAR * __RPC_USER MIDL_user_allocate(SIZE_T len)
// {
//     return RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER MIDL_user_free(void __RPC_FAR * ptr)
// {
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ptr);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
