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


extern "system" {
    // __RPC_USER = stdcall; defined in samrpc.rs (_MIDL_user_free@4).
    fn MIDL_user_free(ptr: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
// External linkage (non-static C globals); #[no_mangle] so the still-C samrpc.c
// (Wave 2) resolves EmptyNtHash/EmptyLmHash/SampResource at final link.
#[no_mangle]
pub static mut EmptyNtHash: ENCRYPTED_NT_OWF_PASSWORD = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut EmptyLmHash: ENCRYPTED_LM_OWF_PASSWORD = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SampResource: RTL_RESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SampProductType: NT_PRODUCT_TYPE = unsafe { core::mem::zeroed() };

unsafe fn SampInitHashes() -> NTSTATUS {
    let mut EmptyNtPassword: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let EmptyLmPassword: [CHAR; 15] = [0; 15];
    let mut Status: NTSTATUS;

    Status = SystemFunction007(
        core::ptr::addr_of_mut!(EmptyNtPassword),
        core::ptr::addr_of_mut!(EmptyNtHash) as LPBYTE,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = SystemFunction006(
        EmptyLmPassword.as_ptr() as LPSTR,
        core::ptr::addr_of_mut!(EmptyLmHash) as LPSTR,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamIConnect(
    _ServerName: PSAMPR_SERVER_NAME,
    ServerHandle: *mut SAMPR_HANDLE,
    mut DesiredAccess: ACCESS_MASK,
    Trusted: BOOLEAN,
) -> NTSTATUS {
    let mut ServerObject: PSAM_DB_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    TRACE!(
        b"SamIConnect(%p %p %lx %ld)\n\0".as_ptr() as LPSTR,
        ServerName,
        ServerHandle,
        DesiredAccess,
        Trusted as i32,
    );

    RtlMapGenericMask(&mut DesiredAccess, pServerMapping);

    Status = SampOpenDbObject(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        w!("SAM").as_ptr() as *mut u16,
        0,
        SamDbServerObject,
        DesiredAccess,
        &mut ServerObject,
    );

    if NT_SUCCESS(Status) {
        (*ServerObject).Trusted = Trusted;
        *ServerHandle = ServerObject as SAMPR_HANDLE;
    }

    TRACE!(
        b"SamIConnect done (Status 0x%08lx)\n\0".as_ptr() as LPSTR,
        Status,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SamIInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS;

    TRACE!(b"SamIInitialize() called\n\0".as_ptr() as *const i8);

    Status = RtlGetNtProductType(core::ptr::addr_of_mut!(SampProductType)) as NTSTATUS;
    if !NT_SUCCESS(Status) {
        SampProductType = NtProductWinNt;
    }

    Status = SampInitHashes();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if SampIsSetupRunning() != 0 {
        Status = SampInitializeRegistry();
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    Status = SampInitializeDisplayCache();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlInitializeResource(core::ptr::addr_of_mut!(SampResource));

    Status = SampInitDatabase();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    SampStartRpcServer();

    Status
}

#[no_mangle]
pub unsafe extern "system" fn SampInitializeRegistry() -> NTSTATUS {
    SampInitializeSAM();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SamIFreeVoid(Ptr: *mut core::ffi::c_void) {
    MIDL_user_free(Ptr);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_ALIAS_INFO_BUFFER(
    Ptr: PSAMPR_ALIAS_INFO_BUFFER,
    InformationClass: ALIAS_INFORMATION_CLASS,
) {
    if Ptr.is_null() {
        return;
    }

    match InformationClass {
        AliasGeneralInformation => {
            if !(*Ptr).General.Name.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.Name.Buffer as *mut core::ffi::c_void);
            }

            if !(*Ptr).General.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.AdminComment.Buffer as *mut core::ffi::c_void);
            }
        }

        AliasNameInformation => {
            if !(*Ptr).Name.Name.Buffer.is_null() {
                MIDL_user_free((*Ptr).Name.Name.Buffer as *mut core::ffi::c_void);
            }
        }

        AliasAdminCommentInformation => {
            if !(*Ptr).AdminComment.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).AdminComment.AdminComment.Buffer as *mut core::ffi::c_void);
            }
        }

        _ => {
            FIXME!("Unsupported information class: %lu\n", InformationClass);
        }
    }

    MIDL_user_free(Ptr as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_DISPLAY_INFO_BUFFER(
    Ptr: PSAMPR_DISPLAY_INFO_BUFFER,
    InformationClass: DOMAIN_DISPLAY_INFORMATION,
) {
    let mut i: ULONG;

    if Ptr.is_null() {
        return;
    }

    match InformationClass {
        DomainDisplayUser => {
            if !(*Ptr).UserInformation.Buffer.is_null() {
                i = 0;
                while i < (*Ptr).UserInformation.EntriesRead {
                    let entry = (*Ptr).UserInformation.Buffer.offset(i as isize);
                    if !(*entry).AccountName.Buffer.is_null() {
                        MIDL_user_free((*entry).AccountName.Buffer as *mut core::ffi::c_void);
                    }
                    if !(*entry).AdminComment.Buffer.is_null() {
                        MIDL_user_free((*entry).AdminComment.Buffer as *mut core::ffi::c_void);
                    }
                    if !(*entry).FullName.Buffer.is_null() {
                        MIDL_user_free((*entry).FullName.Buffer as *mut core::ffi::c_void);
                    }
                    i += 1;
                }

                MIDL_user_free((*Ptr).UserInformation.Buffer as *mut core::ffi::c_void);
            }
        }

        DomainDisplayMachine => {
            if !(*Ptr).MachineInformation.Buffer.is_null() {
                i = 0;
                while i < (*Ptr).MachineInformation.EntriesRead {
                    let entry = (*Ptr).MachineInformation.Buffer.offset(i as isize);
                    if !(*entry).AccountName.Buffer.is_null() {
                        MIDL_user_free((*entry).AccountName.Buffer as *mut core::ffi::c_void);
                    }
                    if !(*entry).AdminComment.Buffer.is_null() {
                        MIDL_user_free((*entry).AdminComment.Buffer as *mut core::ffi::c_void);
                    }
                    i += 1;
                }

                MIDL_user_free((*Ptr).MachineInformation.Buffer as *mut core::ffi::c_void);
            }
        }

        DomainDisplayGroup => {
            if !(*Ptr).GroupInformation.Buffer.is_null() {
                i = 0;
                while i < (*Ptr).GroupInformation.EntriesRead {
                    let entry = (*Ptr).GroupInformation.Buffer.offset(i as isize);
                    if !(*entry).AccountName.Buffer.is_null() {
                        MIDL_user_free((*entry).AccountName.Buffer as *mut core::ffi::c_void);
                    }
                    if !(*entry).AdminComment.Buffer.is_null() {
                        MIDL_user_free((*entry).AdminComment.Buffer as *mut core::ffi::c_void);
                    }
                    i += 1;
                }

                MIDL_user_free((*Ptr).GroupInformation.Buffer as *mut core::ffi::c_void);
            }
        }

        DomainDisplayOemUser => {
            if !(*Ptr).OemUserInformation.Buffer.is_null() {
                i = 0;
                while i < (*Ptr).OemUserInformation.EntriesRead {
                    let entry = (*Ptr).OemUserInformation.Buffer.offset(i as isize);
                    if !(*entry).OemAccountName.Buffer.is_null() {
                        MIDL_user_free((*entry).OemAccountName.Buffer as *mut core::ffi::c_void);
                    }
                    i += 1;
                }

                MIDL_user_free((*Ptr).OemUserInformation.Buffer as *mut core::ffi::c_void);
            }
        }

        DomainDisplayOemGroup => {
            if !(*Ptr).OemGroupInformation.Buffer.is_null() {
                i = 0;
                while i < (*Ptr).OemGroupInformation.EntriesRead {
                    let entry = (*Ptr).OemGroupInformation.Buffer.offset(i as isize);
                    if !(*entry).OemAccountName.Buffer.is_null() {
                        MIDL_user_free((*entry).OemAccountName.Buffer as *mut core::ffi::c_void);
                    }
                    i += 1;
                }

                MIDL_user_free((*Ptr).OemGroupInformation.Buffer as *mut core::ffi::c_void);
            }
        }

        _ => {
            FIXME!("Unsupported information class: %lu\n", InformationClass);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_DOMAIN_INFO_BUFFER(
    Ptr: PSAMPR_DOMAIN_INFO_BUFFER,
    InformationClass: DOMAIN_INFORMATION_CLASS,
) {
    if Ptr.is_null() {
        return;
    }

    match InformationClass {
        DomainPasswordInformation => {}

        DomainGeneralInformation => {
            if !(*Ptr).General.OemInformation.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.OemInformation.Buffer as *mut _);
            }

            if !(*Ptr).General.DomainName.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.DomainName.Buffer as *mut _);
            }

            if !(*Ptr).General.ReplicaSourceNodeName.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.ReplicaSourceNodeName.Buffer as *mut _);
            }
        }

        DomainLogoffInformation => {}

        DomainOemInformation => {
            if !(*Ptr).Oem.OemInformation.Buffer.is_null() {
                MIDL_user_free((*Ptr).Oem.OemInformation.Buffer as *mut _);
            }
        }

        DomainNameInformation => {
            if !(*Ptr).Name.DomainName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Name.DomainName.Buffer as *mut _);
            }
        }

        DomainReplicationInformation => {
            if !(*Ptr).Replication.ReplicaSourceNodeName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Replication.ReplicaSourceNodeName.Buffer as *mut _);
            }
        }

        DomainServerRoleInformation => {}

        DomainModifiedInformation => {}

        DomainStateInformation => {}

        DomainGeneralInformation2 => {
            if !(*Ptr).General2.I1.OemInformation.Buffer.is_null() {
                MIDL_user_free((*Ptr).General2.I1.OemInformation.Buffer as *mut _);
            }

            if !(*Ptr).General2.I1.DomainName.Buffer.is_null() {
                MIDL_user_free((*Ptr).General2.I1.DomainName.Buffer as *mut _);
            }

            if !(*Ptr).General2.I1.ReplicaSourceNodeName.Buffer.is_null() {
                MIDL_user_free((*Ptr).General2.I1.ReplicaSourceNodeName.Buffer as *mut _);
            }
        }

        DomainLockoutInformation => {}

        DomainModifiedInformation2 => {}

        _ => {
            FIXME!("Unsupported information class: %lu\n", InformationClass);
        }
    }

    MIDL_user_free(Ptr as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_ENUMERATION_BUFFER(Ptr: PSAMPR_ENUMERATION_BUFFER) {
    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).Buffer.is_null() {
        let mut i: ULONG = 0;
        while i < (*Ptr).EntriesRead {
            let entry = (*Ptr).Buffer.add(i as usize);
            if !(*entry).Name.Buffer.is_null() {
                MIDL_user_free((*entry).Name.Buffer as *mut core::ffi::c_void);
            }
            i += 1;
        }

        MIDL_user_free((*Ptr).Buffer as *mut core::ffi::c_void);
    }

    MIDL_user_free(Ptr as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_GET_GROUPS_BUFFER(Ptr: PSAMPR_GET_GROUPS_BUFFER) {
    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).Groups.is_null() {
        MIDL_user_free((*Ptr).Groups as *mut core::ffi::c_void);
    }

    MIDL_user_free(Ptr as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_GET_MEMBERS_BUFFER(Ptr: PSAMPR_GET_MEMBERS_BUFFER) {
    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).Members.is_null() {
        MIDL_user_free((*Ptr).Members as *mut _);
    }

    if !(*Ptr).Attributes.is_null() {
        MIDL_user_free((*Ptr).Attributes as *mut _);
    }

    MIDL_user_free(Ptr as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_GROUP_INFO_BUFFER(
    Ptr: PSAMPR_GROUP_INFO_BUFFER,
    InformationClass: GROUP_INFORMATION_CLASS,
) {
    if Ptr.is_null() {
        return;
    }

    match InformationClass {
        GroupGeneralInformation => {
            if !(*Ptr).General.Name.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.Name.Buffer as *mut _);
            }

            if !(*Ptr).General.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.AdminComment.Buffer as *mut _);
            }
        }

        GroupNameInformation => {
            if !(*Ptr).Name.Name.Buffer.is_null() {
                MIDL_user_free((*Ptr).Name.Name.Buffer as *mut _);
            }
        }

        GroupAttributeInformation => {}

        GroupAdminCommentInformation => {
            if !(*Ptr).AdminComment.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).AdminComment.AdminComment.Buffer as *mut _);
            }
        }

        _ => {
            FIXME!(
                "Unsupported information class: %lu\n",
                InformationClass,
            );
        }
    }

    MIDL_user_free(Ptr as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_PSID_ARRAY(Ptr: PSAMPR_PSID_ARRAY) {

    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).Sids.is_null() {
        MIDL_user_free((*Ptr).Sids as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_RETURNED_USTRING_ARRAY(Ptr: PSAMPR_RETURNED_USTRING_ARRAY) {
    let mut i: ULONG;

    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).Element.is_null() {
        i = 0;
        while i < (*Ptr).Count {
            if !(*(*Ptr).Element.offset(i as isize)).Buffer.is_null() {
                MIDL_user_free((*(*Ptr).Element.offset(i as isize)).Buffer as *mut core::ffi::c_void);
            }
            i += 1;
        }

        MIDL_user_free((*Ptr).Element as *mut core::ffi::c_void);
        (*Ptr).Element = core::ptr::null_mut();
        (*Ptr).Count = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_SR_SECURITY_DESCRIPTOR(
    Ptr: PSAMPR_SR_SECURITY_DESCRIPTOR,
) {
    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).SecurityDescriptor.is_null() {
        MIDL_user_free((*Ptr).SecurityDescriptor as *mut core::ffi::c_void);
    }

    MIDL_user_free(Ptr as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_ULONG_ARRAY(Ptr: PSAMPR_ULONG_ARRAY) {
    if Ptr.is_null() {
        return;
    }

    if !(*Ptr).Element.is_null() {
        MIDL_user_free((*Ptr).Element as *mut _);
        (*Ptr).Element = core::ptr::null_mut();
        (*Ptr).Count = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SamIFree_SAMPR_USER_INFO_BUFFER(
    Ptr: PSAMPR_USER_INFO_BUFFER,
    InformationClass: USER_INFORMATION_CLASS,
) {
    if Ptr.is_null() {
        return;
    }

    match InformationClass {
        UserGeneralInformation => {
            if !(*Ptr).General.UserName.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.UserName.Buffer as *mut _);
            }
            if !(*Ptr).General.FullName.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.FullName.Buffer as *mut _);
            }
            if !(*Ptr).General.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.AdminComment.Buffer as *mut _);
            }
            if !(*Ptr).General.UserComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).General.UserComment.Buffer as *mut _);
            }
        }

        UserPreferencesInformation => {
            if !(*Ptr).Preferences.UserComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).Preferences.UserComment.Buffer as *mut _);
            }
            if !(*Ptr).Preferences.Reserved1.Buffer.is_null() {
                MIDL_user_free((*Ptr).Preferences.Reserved1.Buffer as *mut _);
            }
        }

        UserLogonInformation => {
            if !(*Ptr).Logon.UserName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.UserName.Buffer as *mut _);
            }
            if !(*Ptr).Logon.FullName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.FullName.Buffer as *mut _);
            }
            if !(*Ptr).Logon.HomeDirectory.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.HomeDirectory.Buffer as *mut _);
            }
            if !(*Ptr).Logon.HomeDirectoryDrive.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.HomeDirectoryDrive.Buffer as *mut _);
            }
            if !(*Ptr).Logon.ScriptPath.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.ScriptPath.Buffer as *mut _);
            }
            if !(*Ptr).Logon.ProfilePath.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.ProfilePath.Buffer as *mut _);
            }
            if !(*Ptr).Logon.WorkStations.Buffer.is_null() {
                MIDL_user_free((*Ptr).Logon.WorkStations.Buffer as *mut _);
            }
            if !(*Ptr).Logon.LogonHours.LogonHours.is_null() {
                MIDL_user_free((*Ptr).Logon.LogonHours.LogonHours as *mut _);
            }
        }

        UserLogonHoursInformation => {
            if !(*Ptr).LogonHours.LogonHours.LogonHours.is_null() {
                MIDL_user_free((*Ptr).LogonHours.LogonHours.LogonHours as *mut _);
            }
        }

        UserAccountInformation => {
            if !(*Ptr).Account.UserName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.UserName.Buffer as *mut _);
            }
            if !(*Ptr).Account.FullName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.FullName.Buffer as *mut _);
            }
            if !(*Ptr).Account.HomeDirectory.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.HomeDirectory.Buffer as *mut _);
            }
            if !(*Ptr).Account.HomeDirectoryDrive.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.HomeDirectoryDrive.Buffer as *mut _);
            }
            if !(*Ptr).Account.ScriptPath.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.ScriptPath.Buffer as *mut _);
            }
            if !(*Ptr).Account.ProfilePath.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.ProfilePath.Buffer as *mut _);
            }
            if !(*Ptr).Account.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.AdminComment.Buffer as *mut _);
            }
            if !(*Ptr).Account.WorkStations.Buffer.is_null() {
                MIDL_user_free((*Ptr).Account.WorkStations.Buffer as *mut _);
            }
            if !(*Ptr).Account.LogonHours.LogonHours.is_null() {
                MIDL_user_free((*Ptr).Account.LogonHours.LogonHours as *mut _);
            }
        }

        UserNameInformation => {
            if !(*Ptr).Name.UserName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Name.UserName.Buffer as *mut _);
            }
            if !(*Ptr).Name.FullName.Buffer.is_null() {
                MIDL_user_free((*Ptr).Name.FullName.Buffer as *mut _);
            }
        }

        UserAccountNameInformation => {
            if !(*Ptr).AccountName.UserName.Buffer.is_null() {
                MIDL_user_free((*Ptr).AccountName.UserName.Buffer as *mut _);
            }
        }

        UserFullNameInformation => {
            if !(*Ptr).FullName.FullName.Buffer.is_null() {
                MIDL_user_free((*Ptr).FullName.FullName.Buffer as *mut _);
            }
        }

        UserPrimaryGroupInformation => {}

        UserHomeInformation => {
            if !(*Ptr).Home.HomeDirectory.Buffer.is_null() {
                MIDL_user_free((*Ptr).Home.HomeDirectory.Buffer as *mut _);
            }
            if !(*Ptr).Home.HomeDirectoryDrive.Buffer.is_null() {
                MIDL_user_free((*Ptr).Home.HomeDirectoryDrive.Buffer as *mut _);
            }
        }

        UserScriptInformation => {
            if !(*Ptr).Script.ScriptPath.Buffer.is_null() {
                MIDL_user_free((*Ptr).Script.ScriptPath.Buffer as *mut _);
            }
        }

        UserProfileInformation => {
            if !(*Ptr).Profile.ProfilePath.Buffer.is_null() {
                MIDL_user_free((*Ptr).Profile.ProfilePath.Buffer as *mut _);
            }
        }

        UserAdminCommentInformation => {
            if !(*Ptr).AdminComment.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).AdminComment.AdminComment.Buffer as *mut _);
            }
        }

        UserWorkStationsInformation => {
            if !(*Ptr).WorkStations.WorkStations.Buffer.is_null() {
                MIDL_user_free((*Ptr).WorkStations.WorkStations.Buffer as *mut _);
            }
        }

        UserSetPasswordInformation => {
            ERR!("Information class UserSetPasswordInformation cannot be queried!\n");
        }

        UserControlInformation => {}

        UserExpiresInformation => {}

        UserInternal1Information => {}

        UserInternal2Information => {}

        UserParametersInformation => {
            if !(*Ptr).Parameters.Parameters.Buffer.is_null() {
                MIDL_user_free((*Ptr).Parameters.Parameters.Buffer as *mut _);
            }
        }

        UserAllInformation => {
            if !(*Ptr).All.UserName.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.UserName.Buffer as *mut _);
            }
            if !(*Ptr).All.FullName.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.FullName.Buffer as *mut _);
            }
            if !(*Ptr).All.HomeDirectory.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.HomeDirectory.Buffer as *mut _);
            }
            if !(*Ptr).All.HomeDirectoryDrive.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.HomeDirectoryDrive.Buffer as *mut _);
            }
            if !(*Ptr).All.ScriptPath.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.ScriptPath.Buffer as *mut _);
            }
            if !(*Ptr).All.ProfilePath.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.ProfilePath.Buffer as *mut _);
            }
            if !(*Ptr).All.AdminComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.AdminComment.Buffer as *mut _);
            }
            if !(*Ptr).All.WorkStations.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.WorkStations.Buffer as *mut _);
            }
            if !(*Ptr).All.UserComment.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.UserComment.Buffer as *mut _);
            }
            if !(*Ptr).All.Parameters.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.Parameters.Buffer as *mut _);
            }
            if !(*Ptr).All.LmOwfPassword.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.LmOwfPassword.Buffer as *mut _);
            }
            if !(*Ptr).All.NtOwfPassword.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.NtOwfPassword.Buffer as *mut _);
            }
            if !(*Ptr).All.PrivateData.Buffer.is_null() {
                MIDL_user_free((*Ptr).All.PrivateData.Buffer as *mut _);
            }
            if !(*Ptr).All.SecurityDescriptor.SecurityDescriptor.is_null() {
                MIDL_user_free((*Ptr).All.SecurityDescriptor.SecurityDescriptor as *mut _);
            }
            if !(*Ptr).All.LogonHours.LogonHours.is_null() {
                MIDL_user_free((*Ptr).All.LogonHours.LogonHours as *mut _);
            }
        }

        _ => {
            FIXME!("Unsupported information class: %lu\n", InformationClass);
        }
    }

    MIDL_user_free(Ptr as *mut _);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  SAM Server DLL
//  *  Copyright (C) 2005 Eric Kohl
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include "samsrv.h"
//
// #include <samsrv/samsrv.h>
//
// /* GLOBALS *******************************************************************/
//
// ENCRYPTED_NT_OWF_PASSWORD EmptyNtHash;
// ENCRYPTED_LM_OWF_PASSWORD EmptyLmHash;
// RTL_RESOURCE SampResource;
// NT_PRODUCT_TYPE SampProductType;
//
//
// /* FUNCTIONS *****************************************************************/
//
// static
// NTSTATUS
// SampInitHashes(VOID)
// {
//     UNICODE_STRING EmptyNtPassword = {0, 0, NULL};
//     CHAR EmptyLmPassword[15] = {0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,};
//     NTSTATUS Status;
//
//     /* Calculate the NT hash value of the empty password */
//     Status = SystemFunction007(&EmptyNtPassword,
//                                (LPBYTE)&EmptyNtHash);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Calculation of the empty NT hash failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Calculate the LM hash value of the empty password */
//     Status = SystemFunction006(EmptyLmPassword,
//                                (LPSTR)&EmptyLmHash);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Calculation of the empty LM hash failed (Status 0x%08lx)\n", Status);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamIConnect(IN PSAMPR_SERVER_NAME ServerName,
//             OUT SAMPR_HANDLE *ServerHandle,
//             IN ACCESS_MASK DesiredAccess,
//             IN BOOLEAN Trusted)
// {
//     PSAM_DB_OBJECT ServerObject;
//     NTSTATUS Status;
//
//     TRACE("SamIConnect(%p %p %lx %ld)\n",
//           ServerName, ServerHandle, DesiredAccess, Trusted);
//
//     /* Map generic access rights */
//     RtlMapGenericMask(&DesiredAccess,
//                       pServerMapping);
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
//         ServerObject->Trusted = Trusted;
//         *ServerHandle = (SAMPR_HANDLE)ServerObject;
//     }
//
//     TRACE("SamIConnect done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SamIInitialize(VOID)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("SamIInitialize() called\n");
//
//     Status = RtlGetNtProductType(&SampProductType);
//     if (!NT_SUCCESS(Status))
//         SampProductType = NtProductWinNt;
//
//     Status = SampInitHashes();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (SampIsSetupRunning())
//     {
//         Status = SampInitializeRegistry();
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     Status = SampInitializeDisplayCache();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RtlInitializeResource(&SampResource);
//
//     /* Initialize the SAM database */
//     Status = SampInitDatabase();
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Start the RPC server */
//     SampStartRpcServer();
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// SampInitializeRegistry(VOID)
// {
//     TRACE("SampInitializeRegistry() called\n");
//
//     SampInitializeSAM();
//
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// NTAPI
// SamIFreeVoid(PVOID Ptr)
// {
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_ALIAS_INFO_BUFFER(
//     PSAMPR_ALIAS_INFO_BUFFER Ptr,
//     ALIAS_INFORMATION_CLASS InformationClass)
// {
//     if (Ptr == NULL)
//         return;
//
//     switch (InformationClass)
//     {
//         case AliasGeneralInformation:
//             if (Ptr->General.Name.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.Name.Buffer);
//
//             if (Ptr->General.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.AdminComment.Buffer);
//             break;
//
//         case AliasNameInformation:
//             if (Ptr->Name.Name.Buffer != NULL)
//                 MIDL_user_free(Ptr->Name.Name.Buffer);
//             break;
//
//         case AliasAdminCommentInformation:
//             if (Ptr->AdminComment.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->AdminComment.AdminComment.Buffer);
//             break;
//
//         default:
//             FIXME("Unsupported information class: %lu\n", InformationClass);
//             break;
//     }
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_DISPLAY_INFO_BUFFER(
//     PSAMPR_DISPLAY_INFO_BUFFER Ptr,
//     DOMAIN_DISPLAY_INFORMATION InformationClass)
// {
//     ULONG i;
//
//     if (Ptr == NULL)
//         return;
//
//     switch (InformationClass)
//     {
//         case DomainDisplayUser:
//             if (Ptr->UserInformation.Buffer != NULL)
//             {
//                 for (i = 0; i < Ptr->UserInformation.EntriesRead; i++)
//                 {
//                     if (Ptr->UserInformation.Buffer[i].AccountName.Buffer != NULL)
//                         MIDL_user_free(Ptr->UserInformation.Buffer[i].AccountName.Buffer);
//
//                     if (Ptr->UserInformation.Buffer[i].AdminComment.Buffer != NULL)
//                         MIDL_user_free(Ptr->UserInformation.Buffer[i].AdminComment.Buffer);
//
//                     if (Ptr->UserInformation.Buffer[i].FullName.Buffer != NULL)
//                         MIDL_user_free(Ptr->UserInformation.Buffer[i].FullName.Buffer);
//                 }
//
//                 MIDL_user_free(Ptr->UserInformation.Buffer);
//             }
//             break;
//
//         case DomainDisplayMachine:
//             if (Ptr->MachineInformation.Buffer != NULL)
//             {
//                 for (i = 0; i < Ptr->MachineInformation.EntriesRead; i++)
//                 {
//                     if (Ptr->MachineInformation.Buffer[i].AccountName.Buffer != NULL)
//                         MIDL_user_free(Ptr->MachineInformation.Buffer[i].AccountName.Buffer);
//
//                     if (Ptr->MachineInformation.Buffer[i].AdminComment.Buffer != NULL)
//                         MIDL_user_free(Ptr->MachineInformation.Buffer[i].AdminComment.Buffer);
//                 }
//
//                 MIDL_user_free(Ptr->MachineInformation.Buffer);
//             }
//             break;
//
//         case DomainDisplayGroup:
//             if (Ptr->GroupInformation.Buffer != NULL)
//             {
//                 for (i = 0; i < Ptr->GroupInformation.EntriesRead; i++)
//                 {
//                     if (Ptr->GroupInformation.Buffer[i].AccountName.Buffer != NULL)
//                         MIDL_user_free(Ptr->GroupInformation.Buffer[i].AccountName.Buffer);
//
//                     if (Ptr->GroupInformation.Buffer[i].AdminComment.Buffer != NULL)
//                         MIDL_user_free(Ptr->GroupInformation.Buffer[i].AdminComment.Buffer);
//                 }
//
//                 MIDL_user_free(Ptr->GroupInformation.Buffer);
//             }
//             break;
//
//         case DomainDisplayOemUser:
//             if (Ptr->OemUserInformation.Buffer != NULL)
//             {
//                 for (i = 0; i < Ptr->OemUserInformation.EntriesRead; i++)
//                 {
//                     if (Ptr->OemUserInformation.Buffer[i].OemAccountName.Buffer != NULL)
//                         MIDL_user_free(Ptr->OemUserInformation.Buffer[i].OemAccountName.Buffer);
//                 }
//
//                 MIDL_user_free(Ptr->OemUserInformation.Buffer);
//             }
//             break;
//
//         case DomainDisplayOemGroup:
//             if (Ptr->OemGroupInformation.Buffer != NULL)
//             {
//                 for (i = 0; i < Ptr->OemGroupInformation.EntriesRead; i++)
//                 {
//                     if (Ptr->OemGroupInformation.Buffer[i].OemAccountName.Buffer != NULL)
//                         MIDL_user_free(Ptr->OemGroupInformation.Buffer[i].OemAccountName.Buffer);
//                 }
//
//                 MIDL_user_free(Ptr->OemGroupInformation.Buffer);
//             }
//             break;
//
//         default:
//             FIXME("Unsupported information class: %lu\n", InformationClass);
//             break;
//     }
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_DOMAIN_INFO_BUFFER(
//     PSAMPR_DOMAIN_INFO_BUFFER Ptr,
//     DOMAIN_INFORMATION_CLASS InformationClass)
// {
//     if (Ptr == NULL)
//         return;
//
//     switch (InformationClass)
//     {
//         case DomainPasswordInformation:
//             break;
//
//         case DomainGeneralInformation:
//             if (Ptr->General.OemInformation.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.OemInformation.Buffer);
//
//             if (Ptr->General.DomainName.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.DomainName.Buffer);
//
//             if (Ptr->General.ReplicaSourceNodeName.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.ReplicaSourceNodeName.Buffer);
//             break;
//
//         case DomainLogoffInformation:
//             break;
//
//         case DomainOemInformation:
//             if (Ptr->Oem.OemInformation.Buffer != NULL)
//                 MIDL_user_free(Ptr->Oem.OemInformation.Buffer);
//             break;
//
//         case DomainNameInformation:
//             if (Ptr->Name.DomainName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Name.DomainName.Buffer);
//             break;
//
//         case DomainReplicationInformation:
//             if (Ptr->Replication.ReplicaSourceNodeName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Replication.ReplicaSourceNodeName.Buffer);
//             break;
//
//         case DomainServerRoleInformation:
//             break;
//
//         case DomainModifiedInformation:
//             break;
//
//         case DomainStateInformation:
//             break;
//
//         case DomainGeneralInformation2:
//             if (Ptr->General2.I1.OemInformation.Buffer != NULL)
//                 MIDL_user_free(Ptr->General2.I1.OemInformation.Buffer);
//
//             if (Ptr->General2.I1.DomainName.Buffer != NULL)
//                 MIDL_user_free(Ptr->General2.I1.DomainName.Buffer);
//
//             if (Ptr->General2.I1.ReplicaSourceNodeName.Buffer != NULL)
//                 MIDL_user_free(Ptr->General2.I1.ReplicaSourceNodeName.Buffer);
//             break;
//
//         case DomainLockoutInformation:
//             break;
//
//         case DomainModifiedInformation2:
//             break;
//
//         default:
//             FIXME("Unsupported information class: %lu\n", InformationClass);
//             break;
//     }
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_ENUMERATION_BUFFER(PSAMPR_ENUMERATION_BUFFER Ptr)
// {
//     ULONG i;
//
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Buffer != NULL)
//     {
//         for (i = 0; i < Ptr->EntriesRead; i++)
//         {
//             if (Ptr->Buffer[i].Name.Buffer != NULL)
//                 MIDL_user_free(Ptr->Buffer[i].Name.Buffer);
//         }
//
//         MIDL_user_free(Ptr->Buffer);
//     }
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_GET_GROUPS_BUFFER(PSAMPR_GET_GROUPS_BUFFER Ptr)
// {
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Groups != NULL)
//         MIDL_user_free(Ptr->Groups);
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_GET_MEMBERS_BUFFER(PSAMPR_GET_MEMBERS_BUFFER Ptr)
// {
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Members != NULL)
//         MIDL_user_free(Ptr->Members);
//
//     if (Ptr->Attributes != NULL)
//         MIDL_user_free(Ptr->Attributes);
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_GROUP_INFO_BUFFER(
//     PSAMPR_GROUP_INFO_BUFFER Ptr,
//     GROUP_INFORMATION_CLASS InformationClass)
// {
//     if (Ptr == NULL)
//         return;
//
//     switch (InformationClass)
//     {
//         case GroupGeneralInformation:
//             if (Ptr->General.Name.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.Name.Buffer);
//
//             if (Ptr->General.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.AdminComment.Buffer);
//             break;
//
//         case GroupNameInformation:
//             if (Ptr->Name.Name.Buffer != NULL)
//                 MIDL_user_free(Ptr->Name.Name.Buffer);
//             break;
//
//         case GroupAttributeInformation:
//             break;
//
//         case GroupAdminCommentInformation:
//             if (Ptr->AdminComment.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->AdminComment.AdminComment.Buffer);
//             break;
//
//         default:
//             FIXME("Unsupported information class: %lu\n", InformationClass);
//             break;
//     }
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_PSID_ARRAY(PSAMPR_PSID_ARRAY Ptr)
// {
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Sids != NULL)
//     {
//         MIDL_user_free(Ptr->Sids);
//     }
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_RETURNED_USTRING_ARRAY(PSAMPR_RETURNED_USTRING_ARRAY Ptr)
// {
//     ULONG i;
//
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Element != NULL)
//     {
//         for (i = 0; i < Ptr->Count; i++)
//         {
//             if (Ptr->Element[i].Buffer != NULL)
//                 MIDL_user_free(Ptr->Element[i].Buffer);
//         }
//
//         MIDL_user_free(Ptr->Element);
//         Ptr->Element = NULL;
//         Ptr->Count = 0;
//     }
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_SR_SECURITY_DESCRIPTOR(PSAMPR_SR_SECURITY_DESCRIPTOR Ptr)
// {
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->SecurityDescriptor != NULL)
//         MIDL_user_free(Ptr->SecurityDescriptor);
//
//     MIDL_user_free(Ptr);
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_ULONG_ARRAY(PSAMPR_ULONG_ARRAY Ptr)
// {
//     if (Ptr == NULL)
//         return;
//
//     if (Ptr->Element != NULL)
//     {
//         MIDL_user_free(Ptr->Element);
//         Ptr->Element = NULL;
//         Ptr->Count = 0;
//     }
// }
//
//
// VOID
// NTAPI
// SamIFree_SAMPR_USER_INFO_BUFFER(PSAMPR_USER_INFO_BUFFER Ptr,
//                                 USER_INFORMATION_CLASS InformationClass)
// {
//     if (Ptr == NULL)
//         return;
//
//     switch (InformationClass)
//     {
//         case UserGeneralInformation:
//             if (Ptr->General.UserName.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.UserName.Buffer);
//
//             if (Ptr->General.FullName.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.FullName.Buffer);
//
//             if (Ptr->General.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.AdminComment.Buffer);
//
//             if (Ptr->General.UserComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->General.UserComment.Buffer);
//             break;
//
//         case UserPreferencesInformation:
//             if (Ptr->Preferences.UserComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->Preferences.UserComment.Buffer);
//
//             if (Ptr->Preferences.Reserved1.Buffer != NULL)
//                 MIDL_user_free(Ptr->Preferences.Reserved1.Buffer);
//             break;
//
//         case UserLogonInformation:
//             if (Ptr->Logon.UserName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.UserName.Buffer);
//
//             if (Ptr->Logon.FullName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.FullName.Buffer);
//
//             if (Ptr->Logon.HomeDirectory.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.HomeDirectory.Buffer);
//
//             if (Ptr->Logon.HomeDirectoryDrive.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.HomeDirectoryDrive.Buffer);
//
//             if (Ptr->Logon.ScriptPath.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.ScriptPath.Buffer);
//
//             if (Ptr->Logon.ProfilePath.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.ProfilePath.Buffer);
//
//             if (Ptr->Logon.WorkStations.Buffer != NULL)
//                 MIDL_user_free(Ptr->Logon.WorkStations.Buffer);
//
//             if (Ptr->Logon.LogonHours.LogonHours != NULL)
//                 MIDL_user_free(Ptr->Logon.LogonHours.LogonHours);
//             break;
//
//         case UserLogonHoursInformation:
//             if (Ptr->LogonHours.LogonHours.LogonHours != NULL)
//                 MIDL_user_free(Ptr->LogonHours.LogonHours.LogonHours);
//             break;
//
//         case UserAccountInformation:
//             if (Ptr->Account.UserName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.UserName.Buffer);
//
//             if (Ptr->Account.FullName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.FullName.Buffer);
//
//             if (Ptr->Account.HomeDirectory.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.HomeDirectory.Buffer);
//
//             if (Ptr->Account.HomeDirectoryDrive.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.HomeDirectoryDrive.Buffer);
//
//             if (Ptr->Account.ScriptPath.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.ScriptPath.Buffer);
//
//             if (Ptr->Account.ProfilePath.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.ProfilePath.Buffer);
//
//             if (Ptr->Account.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.AdminComment.Buffer);
//
//             if (Ptr->Account.WorkStations.Buffer != NULL)
//                 MIDL_user_free(Ptr->Account.WorkStations.Buffer);
//
//             if (Ptr->Account.LogonHours.LogonHours != NULL)
//                 MIDL_user_free(Ptr->Account.LogonHours.LogonHours);
//             break;
//
//         case UserNameInformation:
//             if (Ptr->Name.UserName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Name.UserName.Buffer);
//
//             if (Ptr->Name.FullName.Buffer != NULL)
//                 MIDL_user_free(Ptr->Name.FullName.Buffer);
//             break;
//
//         case UserAccountNameInformation:
//             if (Ptr->AccountName.UserName.Buffer != NULL)
//                 MIDL_user_free(Ptr->AccountName.UserName.Buffer);
//             break;
//
//         case UserFullNameInformation:
//             if (Ptr->FullName.FullName.Buffer != NULL)
//                 MIDL_user_free(Ptr->FullName.FullName.Buffer);
//             break;
//
//         case UserPrimaryGroupInformation:
//             break;
//
//         case UserHomeInformation:
//             if (Ptr->Home.HomeDirectory.Buffer != NULL)
//                 MIDL_user_free(Ptr->Home.HomeDirectory.Buffer);
//
//             if (Ptr->Home.HomeDirectoryDrive.Buffer != NULL)
//                 MIDL_user_free(Ptr->Home.HomeDirectoryDrive.Buffer);
//             break;
//
//         case UserScriptInformation:
//             if (Ptr->Script.ScriptPath.Buffer != NULL)
//                 MIDL_user_free(Ptr->Script.ScriptPath.Buffer);
//             break;
//
//         case UserProfileInformation:
//             if (Ptr->Profile.ProfilePath.Buffer != NULL)
//                 MIDL_user_free(Ptr->Profile.ProfilePath.Buffer);
//             break;
//
//         case UserAdminCommentInformation:
//             if (Ptr->AdminComment.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->AdminComment.AdminComment.Buffer);
//             break;
//
//         case UserWorkStationsInformation:
//             if (Ptr->WorkStations.WorkStations.Buffer != NULL)
//                 MIDL_user_free(Ptr->WorkStations.WorkStations.Buffer);
//             break;
//
//         case UserSetPasswordInformation:
//             ERR("Information class UserSetPasswordInformation cannot be queried!\n");
//             break;
//
//         case UserControlInformation:
//             break;
//
//         case UserExpiresInformation:
//             break;
//
//         case UserInternal1Information:
//             break;
//
//         case UserInternal2Information:
//             break;
//
//         case UserParametersInformation:
//             if (Ptr->Parameters.Parameters.Buffer != NULL)
//                 MIDL_user_free(Ptr->Parameters.Parameters.Buffer);
//             break;
//
//         case UserAllInformation:
//             if (Ptr->All.UserName.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.UserName.Buffer);
//
//             if (Ptr->All.FullName.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.FullName.Buffer);
//
//             if (Ptr->All.HomeDirectory.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.HomeDirectory.Buffer);
//
//             if (Ptr->All.HomeDirectoryDrive.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.HomeDirectoryDrive.Buffer);
//
//             if (Ptr->All.ScriptPath.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.ScriptPath.Buffer);
//
//             if (Ptr->All.ProfilePath.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.ProfilePath.Buffer);
//
//             if (Ptr->All.AdminComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.AdminComment.Buffer);
//
//             if (Ptr->All.WorkStations.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.WorkStations.Buffer);
//
//             if (Ptr->All.UserComment.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.UserComment.Buffer);
//
//             if (Ptr->All.Parameters.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.Parameters.Buffer);
//
//             if (Ptr->All.LmOwfPassword.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.LmOwfPassword.Buffer);
//
//             if (Ptr->All.NtOwfPassword.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.NtOwfPassword.Buffer);
//
//             if (Ptr->All.PrivateData.Buffer != NULL)
//                 MIDL_user_free(Ptr->All.PrivateData.Buffer);
//
//             if (Ptr->All.SecurityDescriptor.SecurityDescriptor != NULL)
//                 MIDL_user_free(Ptr->All.SecurityDescriptor.SecurityDescriptor);
//
//             if (Ptr->All.LogonHours.LogonHours != NULL)
//                 MIDL_user_free(Ptr->All.LogonHours.LogonHours);
//             break;
//
//         default:
//             FIXME("Unsupported information class: %lu\n", InformationClass);
//             break;
//     }
//
//     MIDL_user_free(Ptr);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
