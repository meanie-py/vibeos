use crate::prelude::*;
use crate::lsasrv::utils::{LsapLoadStringEx, LsapGetResourceStringLengthEx};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"lsasrv\0\0\0\0\0\0\0\0\0"),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

static mut WellKnownPrivileges: [PRIVILEGE_DATA; 29] = unsafe {
    [
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_CREATE_TOKEN_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 46], &[::core::ffi::c_ushort; 23]>(b"S\0e\0C\0r\0e\0a\0t\0e\0T\0o\0k\0e\0n\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_CREATE_TOKEN_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_ASSIGNPRIMARYTOKEN_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 60], &[::core::ffi::c_ushort; 30]>(b"S\0e\0A\0s\0s\0i\0g\0n\0P\0r\0i\0m\0a\0r\0y\0T\0o\0k\0e\0n\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_ASSIGNPRIMARYTOKEN_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_LOCK_MEMORY_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 44], &[::core::ffi::c_ushort; 22]>(b"S\0e\0L\0o\0c\0k\0M\0e\0m\0o\0r\0y\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_LOCK_MEMORY_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_INCREASE_QUOTA_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 50], &[::core::ffi::c_ushort; 25]>(b"S\0e\0I\0n\0c\0r\0e\0a\0s\0e\0Q\0u\0o\0t\0a\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_INCREASE_QUOTA_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_MACHINE_ACCOUNT_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 52], &[::core::ffi::c_ushort; 26]>(b"S\0e\0M\0a\0c\0h\0i\0n\0e\0A\0c\0c\0o\0u\0n\0t\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_MACHINE_ACCOUNT_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_TCB_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 30], &[::core::ffi::c_ushort; 15]>(b"S\0e\0T\0c\0b\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_TCB_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_SECURITY_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 40], &[::core::ffi::c_ushort; 20]>(b"S\0e\0S\0e\0c\0u\0r\0i\0t\0y\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_SECURITY_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_TAKE_OWNERSHIP_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 50], &[::core::ffi::c_ushort; 25]>(b"S\0e\0T\0a\0k\0e\0O\0w\0n\0e\0r\0s\0h\0i\0p\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_TAKE_OWNERSHIP_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_LOAD_DRIVER_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 44], &[::core::ffi::c_ushort; 22]>(b"S\0e\0L\0o\0a\0d\0D\0r\0i\0v\0e\0r\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_LOAD_DRIVER_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_SYSTEM_PROFILE_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 50], &[::core::ffi::c_ushort; 25]>(b"S\0e\0S\0y\0s\0t\0e\0m\0P\0r\0o\0f\0i\0l\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_SYSTEM_PROFILE_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_SYSTEMTIME_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 44], &[::core::ffi::c_ushort; 22]>(b"S\0e\0S\0y\0s\0t\0e\0m\0t\0i\0m\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_SYSTEMTIME_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_PROF_SINGLE_PROCESS_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 64], &[::core::ffi::c_ushort; 32]>(b"S\0e\0P\0r\0o\0f\0i\0l\0e\0S\0i\0n\0g\0l\0e\0P\0r\0o\0c\0e\0s\0s\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_PROF_SINGLE_PROCESS_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_INC_BASE_PRIORITY_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 64], &[::core::ffi::c_ushort; 32]>(b"S\0e\0I\0n\0c\0r\0e\0a\0s\0e\0B\0a\0s\0e\0P\0r\0i\0o\0r\0i\0t\0y\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_INC_BASE_PRIORITY_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_CREATE_PAGEFILE_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 52], &[::core::ffi::c_ushort; 26]>(b"S\0e\0C\0r\0e\0a\0t\0e\0P\0a\0g\0e\0f\0i\0l\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_CREATE_PAGEFILE_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_CREATE_PERMANENT_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 54], &[::core::ffi::c_ushort; 27]>(b"S\0e\0C\0r\0e\0a\0t\0e\0P\0e\0r\0m\0a\0n\0e\0n\0t\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_CREATE_PERMANENT_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_BACKUP_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 36], &[::core::ffi::c_ushort; 18]>(b"S\0e\0B\0a\0c\0k\0u\0p\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_BACKUP_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_RESTORE_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 38], &[::core::ffi::c_ushort; 19]>(b"S\0e\0R\0e\0s\0t\0o\0r\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_RESTORE_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_SHUTDOWN_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 40], &[::core::ffi::c_ushort; 20]>(b"S\0e\0S\0h\0u\0t\0d\0o\0w\0n\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_SHUTDOWN_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_DEBUG_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 34], &[::core::ffi::c_ushort; 17]>(b"S\0e\0D\0e\0b\0u\0g\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_DEBUG_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_AUDIT_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 34], &[::core::ffi::c_ushort; 17]>(b"S\0e\0A\0u\0d\0i\0t\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_AUDIT_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_SYSTEM_ENVIRONMENT_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 58], &[::core::ffi::c_ushort; 29]>(b"S\0e\0S\0y\0s\0t\0e\0m\0E\0n\0v\0i\0r\0o\0n\0m\0e\0n\0t\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_SYSTEM_ENVIRONMENT_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_CHANGE_NOTIFY_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 48], &[::core::ffi::c_ushort; 24]>(b"S\0e\0C\0h\0a\0n\0g\0e\0N\0o\0t\0i\0f\0y\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_CHANGE_NOTIFY_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_REMOTE_SHUTDOWN_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 52], &[::core::ffi::c_ushort; 26]>(b"S\0e\0R\0e\0m\0o\0t\0e\0S\0h\0u\0t\0d\0o\0w\0n\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_REMOTE_SHUTDOWN_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_UNDOCK_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 36], &[::core::ffi::c_ushort; 18]>(b"S\0e\0U\0n\0d\0o\0c\0k\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_UNDOCK_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_SYNC_AGENT_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 42], &[::core::ffi::c_ushort; 21]>(b"S\0e\0S\0y\0n\0c\0A\0g\0e\0n\0t\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_SYNC_AGENT_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_ENABLE_DELEGATION_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 56], &[::core::ffi::c_ushort; 28]>(b"S\0e\0E\0n\0a\0b\0l\0e\0D\0e\0l\0e\0g\0a\0t\0i\0o\0n\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_ENABLE_DELEGATION_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_MANAGE_VOLUME_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 48], &[::core::ffi::c_ushort; 24]>(b"S\0e\0M\0a\0n\0a\0g\0e\0V\0o\0l\0u\0m\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_MANAGE_VOLUME_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_IMPERSONATE_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 46], &[::core::ffi::c_ushort; 23]>(b"S\0e\0I\0m\0p\0e\0r\0s\0o\0n\0a\0t\0e\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_IMPERSONATE_PRIVILEGE,
        },
        PRIVILEGE_DATA {
            Luid: _LUID {
                LowPart: SE_CREATE_GLOBAL_PRIVILEGE as DWORD,
                HighPart: 0 as LONG,
            },
            Name: ::core::mem::transmute::<&[u8; 48], &[::core::ffi::c_ushort; 24]>(b"S\0e\0C\0r\0e\0a\0t\0e\0G\0l\0o\0b\0a\0l\0P\0r\0i\0v\0i\0l\0e\0g\0e\0\0\0")
                .as_ptr() as LPCWSTR,
            DisplayNameId: IDS_CREATE_GLOBAL_PRIVILEGE,
        },
    ]
};

static mut WellKnownRights: [RIGHT_DATA; 10] = unsafe {
    [
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_INTERACTIVE_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 48], &[::core::ffi::c_ushort; 24]>(b"S\0e\0I\0n\0t\0e\0r\0a\0c\0t\0i\0v\0e\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_NETWORK_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 40], &[::core::ffi::c_ushort; 20]>(b"S\0e\0N\0e\0t\0w\0o\0r\0k\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_BATCH_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 36], &[::core::ffi::c_ushort; 18]>(b"S\0e\0B\0a\0t\0c\0h\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_SERVICE_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 40], &[::core::ffi::c_ushort; 20]>(b"S\0e\0S\0e\0r\0v\0i\0c\0e\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_DENY_INTERACTIVE_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 56], &[::core::ffi::c_ushort; 28]>(b"S\0e\0D\0e\0n\0y\0I\0n\0t\0e\0r\0a\0c\0t\0i\0v\0e\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_DENY_NETWORK_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 48], &[::core::ffi::c_ushort; 24]>(b"S\0e\0D\0e\0n\0y\0N\0e\0t\0w\0o\0r\0k\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_DENY_BATCH_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 44], &[::core::ffi::c_ushort; 22]>(b"S\0e\0D\0e\0n\0y\0B\0a\0t\0c\0h\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_DENY_SERVICE_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 48], &[::core::ffi::c_ushort; 24]>(b"S\0e\0D\0e\0n\0y\0S\0e\0r\0v\0i\0c\0e\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_REMOTE_INTERACTIVE_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 60], &[::core::ffi::c_ushort; 30]>(b"S\0e\0R\0e\0m\0o\0t\0e\0I\0n\0t\0e\0r\0a\0c\0t\0i\0v\0e\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
        RIGHT_DATA {
            Flag: SECURITY_ACCESS_DENY_REMOTE_INTERACTIVE_LOGON as ULONG,
            Name: ::core::mem::transmute::<&[u8; 68], &[::core::ffi::c_ushort; 34]>(b"S\0e\0D\0e\0n\0y\0R\0e\0m\0o\0t\0e\0I\0n\0t\0e\0r\0a\0c\0t\0i\0v\0e\0L\0o\0g\0o\0n\0R\0i\0g\0h\0t\0\0\0")
                .as_ptr() as LPCWSTR,
        },
    ]
};

#[repr(C)]
pub struct PRIVILEGE_DATA {
    pub Luid: LUID,
    pub Name: *const u16,
    pub DisplayNameId: i32,
}

#[repr(C)]
pub struct RIGHT_DATA {
    pub Flag: u32,
    pub Name: *const u16,
}
#[no_mangle]
pub unsafe extern "C" fn LsarpLookupPrivilegeName(
    Value: PLUID,
    Name: *mut PRPC_UNICODE_STRING,
) -> NTSTATUS {
    if (*Value).HighPart != 0
        || ((*Value).LowPart < SE_MIN_WELL_KNOWN_PRIVILEGE as u32
            || (*Value).LowPart > SE_MAX_WELL_KNOWN_PRIVILEGE as u32)
    {
        return STATUS_NO_SUCH_PRIVILEGE;
    }

    for priv_data in (*&raw const WellKnownPrivileges).iter() {
        if (*Value).LowPart == priv_data.Luid.LowPart
            && (*Value).HighPart == priv_data.Luid.HighPart
        {
            let name_buffer = MIDL_user_allocate(
                core::mem::size_of::<RPC_UNICODE_STRING>(),
            ) as PRPC_UNICODE_STRING;
            if name_buffer.is_null() {
                return STATUS_NO_MEMORY;
            }

            let src_name = priv_data.Name;
            let len = wcslen(src_name);
            let length = (len as usize)
                .wrapping_mul(core::mem::size_of::<u16>()) as u16;
            (*name_buffer).Length = length;
            let max_length = length.wrapping_add(core::mem::size_of::<u16>() as u16);
            (*name_buffer).MaximumLength = max_length;

            let buffer =
                MIDL_user_allocate(max_length as usize) as *mut u16;
            if buffer.is_null() {
                MIDL_user_free(name_buffer as *mut core::ffi::c_void);
                return STATUS_NO_MEMORY;
            }
            (*name_buffer).Buffer = buffer;

            wcscpy((*name_buffer).Buffer, src_name);

            *Name = name_buffer;
            return STATUS_SUCCESS;
        }
    }

    STATUS_NO_SUCH_PRIVILEGE
}
#[no_mangle]
pub unsafe extern "C" fn LsarpLookupPrivilegeDisplayName(
    Name: PRPC_UNICODE_STRING,
    ClientLanguage: USHORT,
    ClientSystemDefaultLanguage: USHORT,
    DisplayName: *mut PRPC_UNICODE_STRING,
    LanguageReturned: *mut USHORT,
) -> NTSTATUS {
    let DisplayNameBuffer: PRPC_UNICODE_STRING;
    let hInstance: HINSTANCE;
    let mut Index: ULONG;
    let mut nLength: UINT;

    TRACE!(
        b"LsarpLookupPrivilegeDisplayName(%p 0x%04hu 0x%04hu %p %p)\n\0".as_ptr() as *const i8,
        Name,
        ClientLanguage,
        ClientSystemDefaultLanguage,
        DisplayName,
        LanguageReturned,
    );

    if (*Name).Length == 0 || (*Name).Buffer.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let module_name: [u16; 12] = [
        'l' as u16, 's' as u16, 'a' as u16, 's' as u16,
        'r' as u16, 'v' as u16, '.' as u16, 'd' as u16,
        'l' as u16, 'l' as u16, 0u16, 0u16,
    ];
    hInstance = GetModuleHandleW(module_name.as_ptr()) as HINSTANCE;

    Index = 0;
    while (Index as usize) < (*&raw const WellKnownPrivileges).len() {
        if _wcsicmp((*Name).Buffer, WellKnownPrivileges[Index as usize].Name) == 0 {
            TRACE!(
                b"Index: %u\n\0".as_ptr() as *const i8,
                Index,
            );

            nLength = (LsapGetResourceStringLengthEx(
                hInstance,
                (IDS_CREATE_TOKEN_PRIVILEGE as u32).wrapping_add(Index),
                ClientLanguage,
            )) as UINT;

            if nLength != 0 {
                DisplayNameBuffer = MIDL_user_allocate(
                    core::mem::size_of::<RPC_UNICODE_STRING>(),
                ) as PRPC_UNICODE_STRING;
                if DisplayNameBuffer.is_null() {
                    return STATUS_NO_MEMORY;
                }

                (*DisplayNameBuffer).Length = (nLength as usize)
                    .wrapping_mul(core::mem::size_of::<u16>()) as USHORT;
                (*DisplayNameBuffer).MaximumLength = (*DisplayNameBuffer)
                    .Length
                    .wrapping_add(core::mem::size_of::<u16>() as USHORT);
                (*DisplayNameBuffer).Buffer = MIDL_user_allocate(
                    (*DisplayNameBuffer).MaximumLength as usize,
                ) as LPWSTR;
                if (*DisplayNameBuffer).Buffer.is_null() {
                    MIDL_user_free(DisplayNameBuffer as *mut core::ffi::c_void);
                    return STATUS_NO_MEMORY;
                }

                LsapLoadStringEx(
                    hInstance,
                    (IDS_CREATE_TOKEN_PRIVILEGE as u32).wrapping_add(Index),
                    ClientLanguage,
                    (*DisplayNameBuffer).Buffer,
                    nLength as i32,
                );

                *DisplayName = DisplayNameBuffer;
                *LanguageReturned = ClientLanguage;
            } else {
                nLength = (LsapGetResourceStringLengthEx(
                    hInstance,
                    (IDS_CREATE_TOKEN_PRIVILEGE as u32).wrapping_add(Index),
                    ClientSystemDefaultLanguage,
                )) as UINT;

                if nLength != 0 {
                    DisplayNameBuffer = MIDL_user_allocate(
                        core::mem::size_of::<RPC_UNICODE_STRING>(),
                    ) as PRPC_UNICODE_STRING;
                    if DisplayNameBuffer.is_null() {
                        return STATUS_NO_MEMORY;
                    }

                    (*DisplayNameBuffer).Length = (nLength as usize)
                        .wrapping_mul(core::mem::size_of::<u16>()) as USHORT;
                    (*DisplayNameBuffer).MaximumLength = (*DisplayNameBuffer)
                        .Length
                        .wrapping_add(core::mem::size_of::<u16>() as USHORT);
                    (*DisplayNameBuffer).Buffer = MIDL_user_allocate(
                        (*DisplayNameBuffer).MaximumLength as usize,
                    ) as LPWSTR;
                    if (*DisplayNameBuffer).Buffer.is_null() {
                        MIDL_user_free(DisplayNameBuffer as *mut core::ffi::c_void);
                        return STATUS_NO_MEMORY;
                    }

                    LsapLoadStringEx(
                        hInstance,
                        (IDS_CREATE_TOKEN_PRIVILEGE as u32).wrapping_add(Index),
                        ClientSystemDefaultLanguage,
                        (*DisplayNameBuffer).Buffer,
                        nLength as i32,
                    );

                    *DisplayName = DisplayNameBuffer;
                    *LanguageReturned = ClientSystemDefaultLanguage;
                } else {
                    return STATUS_INVALID_PARAMETER;
                }
            }

            return STATUS_SUCCESS;
        }

        Index = Index.wrapping_add(1);
    }

    return STATUS_NO_SUCH_PRIVILEGE;
}
#[no_mangle]
pub unsafe extern "C" fn LsarpLookupPrivilegeValue(Name: *mut _RPC_UNICODE_STRING) -> *mut _LUID {
    if (*Name).Length == 0 || (*Name).Buffer.is_null() {
        return core::ptr::null_mut::<_LUID>();
    }
    let mut Priv: u32 = 0;
    while (Priv as usize) < (*&raw const WellKnownPrivileges).len() {
        if _wcsicmp(
            (*Name).Buffer as *const u16,
            WellKnownPrivileges[Priv as usize].Name as *const u16,
        ) == 0
        {
            return core::ptr::addr_of!(
                (*(&raw const WellKnownPrivileges).cast::<PRIVILEGE_DATA>().add(Priv as usize)).Luid
            ) as *mut _LUID ;
        }
        Priv = Priv.wrapping_add(1);
    }
    core::ptr::null_mut::<_LUID>()
}
#[no_mangle]
pub unsafe extern "C" fn LsarpEnumeratePrivileges(
    EnumerationContext: *mut DWORD,
    EnumerationBuffer: PLSAPR_PRIVILEGE_ENUM_BUFFER,
    PreferedMaximumLength: DWORD,
) -> NTSTATUS {

    let mut Privileges: PLSAPR_POLICY_PRIVILEGE_DEF = core::ptr::null_mut::<LSAPR_POLICY_PRIVILEGE_DEF>();
    let mut EnumIndex: u32;
    let mut EnumCount: u32 = 0;
    let mut RequiredLength: u32 = 0;
    let mut i: u32;
    let mut MoreEntries: BOOLEAN = 0u8;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    EnumIndex = *EnumerationContext as u32;
    const WELL_KNOWN_COUNT: usize = 29;
    while (EnumIndex as usize) < WELL_KNOWN_COUNT {
        TRACE!("EnumIndex: %lu\n", EnumIndex);
        TRACE!("Privilege Name: %S\n", WellKnownPrivileges[EnumIndex as usize].Name);
        TRACE!("Name Length: %lu\n", wcslen(WellKnownPrivileges[EnumIndex as usize].Name));
        let name_len = wcslen(WellKnownPrivileges[EnumIndex as usize].Name) as u32;
        let needed = RequiredLength
            .wrapping_add(name_len.wrapping_mul(core::mem::size_of::<WCHAR>() as u32))
            .wrapping_add(core::mem::size_of::<WCHAR>() as u32)
            .wrapping_add(core::mem::size_of::<LSAPR_POLICY_PRIVILEGE_DEF>() as u32);
        if needed > PreferedMaximumLength {
            MoreEntries = 1u8;
            break;
        }
        RequiredLength = needed;
        EnumCount = EnumCount.wrapping_add(1);
        EnumIndex = EnumIndex.wrapping_add(1);
    }

    TRACE!("EnumCount: %lu\n", EnumCount);
    TRACE!("RequiredLength: %lu\n", RequiredLength);

    if EnumCount == 0 {
        // goto done
    } else {
        Privileges = MIDL_user_allocate(
            (EnumCount as usize).wrapping_mul(core::mem::size_of::<LSAPR_POLICY_PRIVILEGE_DEF>())
        ) as PLSAPR_POLICY_PRIVILEGE_DEF;
        if Privileges.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            // goto done
        } else {
            EnumIndex = *EnumerationContext as u32;
            i = 0;
            while i < EnumCount {
                (*Privileges.offset(i as isize)).LocalValue = ::core::mem::transmute(WellKnownPrivileges[EnumIndex as usize].Luid);
                (*Privileges.offset(i as isize)).Name.Length = (wcslen(
                    WellKnownPrivileges[EnumIndex as usize].Name,
                ) as u16)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as u16);
                (*Privileges.offset(i as isize)).Name.MaximumLength =
                    (*Privileges.offset(i as isize))
                        .Name
                        .Length
                        .wrapping_add(core::mem::size_of::<WCHAR>() as u16);
                let buffer = MIDL_user_allocate(
                    (*Privileges.offset(i as isize)).Name.MaximumLength as usize,
                ) as LPWSTR;
                (*Privileges.offset(i as isize)).Name.Buffer = buffer;
                if buffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }
                core::ptr::copy_nonoverlapping(
                    WellKnownPrivileges[EnumIndex as usize].Name as *const u8,
                    buffer as *mut u8,
                    (*Privileges.offset(i as isize)).Name.Length as usize,
                );
                i = i.wrapping_add(1);
                EnumIndex = EnumIndex.wrapping_add(1);
            }
        }
    }

    // done:
    if Status >= 0 {
        (*EnumerationBuffer).Entries = EnumCount as u32;
        (*EnumerationBuffer).Privileges = Privileges;
        *EnumerationContext = (*EnumerationContext).wrapping_add(EnumCount);
    } else {
        if !Privileges.is_null() {
            i = 0;
            while i < EnumCount {
                if !(*Privileges.offset(i as isize)).Name.Buffer.is_null() {
                    MIDL_user_free(
                        (*Privileges.offset(i as isize)).Name.Buffer as *mut core::ffi::c_void,
                    );
                }
                i = i.wrapping_add(1);
            }
            MIDL_user_free(Privileges as *mut core::ffi::c_void);
        }
    }

    if Status == STATUS_SUCCESS && MoreEntries != 0 {
        Status = STATUS_MORE_ENTRIES;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupAccountRightName(
    RightValue: ULONG,
    Name: *mut PRPC_UNICODE_STRING,
) -> NTSTATUS {
    let NameBuffer: PRPC_UNICODE_STRING;
    for right_data in (*&raw const WellKnownRights).iter() {
        if right_data.Flag == RightValue {
            NameBuffer = MIDL_user_allocate(
                core::mem::size_of::<RPC_UNICODE_STRING>(),
            ) as PRPC_UNICODE_STRING;
            if NameBuffer.is_null() {
                return STATUS_NO_MEMORY;
            }
            (*NameBuffer).Length = (wcslen(right_data.Name as *const wchar_t) as USHORT as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as USHORT;
            (*NameBuffer).MaximumLength = ((*NameBuffer).Length as usize)
                .wrapping_add(core::mem::size_of::<WCHAR>()) as USHORT;
            (*NameBuffer).Buffer = MIDL_user_allocate((*NameBuffer).MaximumLength as usize)
                as LPWSTR;
            if (*NameBuffer).Buffer.is_null() {
                MIDL_user_free(NameBuffer as *mut core::ffi::c_void);
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            wcscpy(
                (*NameBuffer).Buffer as *mut wchar_t,
                right_data.Name as *const wchar_t,
            );
            *Name = NameBuffer;
            return STATUS_SUCCESS;
        }
    }
    return STATUS_NO_SUCH_PRIVILEGE;
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupAccountRightValue(Name: *mut _UNICODE_STRING) -> ACCESS_MASK {
    if (*Name).Length == 0 || (*Name).Buffer.is_null() {
        return 0;
    }
    for i in 0..(*&raw const WellKnownRights).len() {
        if _wcsicmp(
            (*Name).Buffer as *const wchar_t,
            WellKnownRights[i].Name as *const wchar_t,
        ) == 0
        {
            return WellKnownRights[i].Flag;
        }
    }
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Local Security Authority (LSA) Server
//  * FILE:            reactos/dll/win32/lsasrv/privileges.c
//  * PURPOSE:         Privilege lookup functions
//  *
//  * PROGRAMMERS:     Eric Kohl <eric.kohl@reactos.org>
//  */
//
// #include "lsasrv.h"
// #include "resources.h"
//
// typedef struct
// {
//     LUID Luid;
//     LPCWSTR Name;
//     INT DisplayNameId;
// } PRIVILEGE_DATA;
//
// typedef struct
// {
//     ULONG Flag;
//     LPCWSTR Name;
// } RIGHT_DATA;
//
//
// /* GLOBALS *****************************************************************/
//
// static const PRIVILEGE_DATA WellKnownPrivileges[] =
// {
//     {{SE_CREATE_TOKEN_PRIVILEGE, 0}, SE_CREATE_TOKEN_NAME, IDS_CREATE_TOKEN_PRIVILEGE},
//     {{SE_ASSIGNPRIMARYTOKEN_PRIVILEGE, 0}, SE_ASSIGNPRIMARYTOKEN_NAME, IDS_ASSIGNPRIMARYTOKEN_PRIVILEGE},
//     {{SE_LOCK_MEMORY_PRIVILEGE, 0}, SE_LOCK_MEMORY_NAME, IDS_LOCK_MEMORY_PRIVILEGE},
//     {{SE_INCREASE_QUOTA_PRIVILEGE, 0}, SE_INCREASE_QUOTA_NAME, IDS_INCREASE_QUOTA_PRIVILEGE},
//     {{SE_MACHINE_ACCOUNT_PRIVILEGE, 0}, SE_MACHINE_ACCOUNT_NAME, IDS_MACHINE_ACCOUNT_PRIVILEGE},
//     {{SE_TCB_PRIVILEGE, 0}, SE_TCB_NAME, IDS_TCB_PRIVILEGE},
//     {{SE_SECURITY_PRIVILEGE, 0}, SE_SECURITY_NAME, IDS_SECURITY_PRIVILEGE},
//     {{SE_TAKE_OWNERSHIP_PRIVILEGE, 0}, SE_TAKE_OWNERSHIP_NAME, IDS_TAKE_OWNERSHIP_PRIVILEGE},
//     {{SE_LOAD_DRIVER_PRIVILEGE, 0}, SE_LOAD_DRIVER_NAME, IDS_LOAD_DRIVER_PRIVILEGE},
//     {{SE_SYSTEM_PROFILE_PRIVILEGE, 0}, SE_SYSTEM_PROFILE_NAME, IDS_SYSTEM_PROFILE_PRIVILEGE},
//     {{SE_SYSTEMTIME_PRIVILEGE, 0}, SE_SYSTEMTIME_NAME, IDS_SYSTEMTIME_PRIVILEGE},
//     {{SE_PROF_SINGLE_PROCESS_PRIVILEGE, 0}, SE_PROF_SINGLE_PROCESS_NAME, IDS_PROF_SINGLE_PROCESS_PRIVILEGE},
//     {{SE_INC_BASE_PRIORITY_PRIVILEGE, 0}, SE_INC_BASE_PRIORITY_NAME, IDS_INC_BASE_PRIORITY_PRIVILEGE},
//     {{SE_CREATE_PAGEFILE_PRIVILEGE, 0}, SE_CREATE_PAGEFILE_NAME, IDS_CREATE_PAGEFILE_PRIVILEGE},
//     {{SE_CREATE_PERMANENT_PRIVILEGE, 0}, SE_CREATE_PERMANENT_NAME, IDS_CREATE_PERMANENT_PRIVILEGE},
//     {{SE_BACKUP_PRIVILEGE, 0}, SE_BACKUP_NAME, IDS_BACKUP_PRIVILEGE},
//     {{SE_RESTORE_PRIVILEGE, 0}, SE_RESTORE_NAME, IDS_RESTORE_PRIVILEGE},
//     {{SE_SHUTDOWN_PRIVILEGE, 0}, SE_SHUTDOWN_NAME, IDS_SHUTDOWN_PRIVILEGE},
//     {{SE_DEBUG_PRIVILEGE, 0}, SE_DEBUG_NAME, IDS_DEBUG_PRIVILEGE},
//     {{SE_AUDIT_PRIVILEGE, 0}, SE_AUDIT_NAME, IDS_AUDIT_PRIVILEGE},
//     {{SE_SYSTEM_ENVIRONMENT_PRIVILEGE, 0}, SE_SYSTEM_ENVIRONMENT_NAME, IDS_SYSTEM_ENVIRONMENT_PRIVILEGE},
//     {{SE_CHANGE_NOTIFY_PRIVILEGE, 0}, SE_CHANGE_NOTIFY_NAME, IDS_CHANGE_NOTIFY_PRIVILEGE},
//     {{SE_REMOTE_SHUTDOWN_PRIVILEGE, 0}, SE_REMOTE_SHUTDOWN_NAME, IDS_REMOTE_SHUTDOWN_PRIVILEGE},
//     {{SE_UNDOCK_PRIVILEGE, 0}, SE_UNDOCK_NAME, IDS_UNDOCK_PRIVILEGE},
//     {{SE_SYNC_AGENT_PRIVILEGE, 0}, SE_SYNC_AGENT_NAME, IDS_SYNC_AGENT_PRIVILEGE},
//     {{SE_ENABLE_DELEGATION_PRIVILEGE, 0}, SE_ENABLE_DELEGATION_NAME, IDS_ENABLE_DELEGATION_PRIVILEGE},
//     {{SE_MANAGE_VOLUME_PRIVILEGE, 0}, SE_MANAGE_VOLUME_NAME, IDS_MANAGE_VOLUME_PRIVILEGE},
//     {{SE_IMPERSONATE_PRIVILEGE, 0}, SE_IMPERSONATE_NAME, IDS_IMPERSONATE_PRIVILEGE},
//     {{SE_CREATE_GLOBAL_PRIVILEGE, 0}, SE_CREATE_GLOBAL_NAME, IDS_CREATE_GLOBAL_PRIVILEGE}
// };
//
// static const RIGHT_DATA WellKnownRights[] =
// {
//     {SECURITY_ACCESS_INTERACTIVE_LOGON, SE_INTERACTIVE_LOGON_NAME},
//     {SECURITY_ACCESS_NETWORK_LOGON, SE_NETWORK_LOGON_NAME},
//     {SECURITY_ACCESS_BATCH_LOGON, SE_BATCH_LOGON_NAME},
//     {SECURITY_ACCESS_SERVICE_LOGON, SE_SERVICE_LOGON_NAME},
//     {SECURITY_ACCESS_DENY_INTERACTIVE_LOGON, SE_DENY_INTERACTIVE_LOGON_NAME},
//     {SECURITY_ACCESS_DENY_NETWORK_LOGON, SE_DENY_NETWORK_LOGON_NAME},
//     {SECURITY_ACCESS_DENY_BATCH_LOGON, SE_DENY_BATCH_LOGON_NAME},
//     {SECURITY_ACCESS_DENY_SERVICE_LOGON, SE_DENY_SERVICE_LOGON_NAME},
//     {SECURITY_ACCESS_REMOTE_INTERACTIVE_LOGON, SE_REMOTE_INTERACTIVE_LOGON_NAME},
//     {SECURITY_ACCESS_DENY_REMOTE_INTERACTIVE_LOGON, SE_DENY_REMOTE_INTERACTIVE_LOGON_NAME}
// };
//
//
// /* FUNCTIONS ***************************************************************/
//
// NTSTATUS
// LsarpLookupPrivilegeName(PLUID Value,
//                          PRPC_UNICODE_STRING *Name)
// {
//     PRPC_UNICODE_STRING NameBuffer;
//     ULONG Priv;
//
//     if (Value->HighPart != 0 ||
//         (Value->LowPart < SE_MIN_WELL_KNOWN_PRIVILEGE ||
//          Value->LowPart > SE_MAX_WELL_KNOWN_PRIVILEGE))
//     {
//         return STATUS_NO_SUCH_PRIVILEGE;
//     }
//
//     for (Priv = 0; Priv < ARRAYSIZE(WellKnownPrivileges); Priv++)
//     {
//         if (Value->LowPart == WellKnownPrivileges[Priv].Luid.LowPart &&
//             Value->HighPart == WellKnownPrivileges[Priv].Luid.HighPart)
//         {
//             NameBuffer = MIDL_user_allocate(sizeof(RPC_UNICODE_STRING));
//             if (NameBuffer == NULL)
//                 return STATUS_NO_MEMORY;
//
//             NameBuffer->Length = (USHORT)wcslen(WellKnownPrivileges[Priv].Name) * sizeof(WCHAR);
//             NameBuffer->MaximumLength = NameBuffer->Length + sizeof(WCHAR);
//
//             NameBuffer->Buffer = MIDL_user_allocate(NameBuffer->MaximumLength);
//             if (NameBuffer->Buffer == NULL)
//             {
//                 MIDL_user_free(NameBuffer);
//                 return STATUS_NO_MEMORY;
//             }
//
//             wcscpy(NameBuffer->Buffer, WellKnownPrivileges[Priv].Name);
//
//             *Name = NameBuffer;
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     return STATUS_NO_SUCH_PRIVILEGE;
// }
//
//
// NTSTATUS
// LsarpLookupPrivilegeDisplayName(PRPC_UNICODE_STRING Name,
//                                 USHORT ClientLanguage,
//                                 USHORT ClientSystemDefaultLanguage,
//                                 PRPC_UNICODE_STRING *DisplayName,
//                                 USHORT *LanguageReturned)
// {
//     PRPC_UNICODE_STRING DisplayNameBuffer;
//     HINSTANCE hInstance;
//     ULONG Index;
//     UINT nLength;
//
//     TRACE("LsarpLookupPrivilegeDisplayName(%p 0x%04hu 0x%04hu %p %p)\n",
//           Name, ClientLanguage, ClientSystemDefaultLanguage, DisplayName, LanguageReturned);
//
//     if (Name->Length == 0 || Name->Buffer == NULL)
//         return STATUS_INVALID_PARAMETER;
//
//     hInstance = GetModuleHandleW(L"lsasrv.dll");
//
//     for (Index = 0; Index < ARRAYSIZE(WellKnownPrivileges); Index++)
//     {
//         if (_wcsicmp(Name->Buffer, WellKnownPrivileges[Index].Name) == 0)
//         {
//             TRACE("Index: %u\n", Index);
//             nLength = (LsapGetResourceStringLengthEx(hInstance,
//                                                     IDS_CREATE_TOKEN_PRIVILEGE + Index,
//                                                     ClientLanguage)) as UINT;
//             if (nLength != 0)
//             {
//                 DisplayNameBuffer = MIDL_user_allocate(sizeof(RPC_UNICODE_STRING));
//                 if (DisplayNameBuffer == NULL)
//                     return STATUS_NO_MEMORY;
//
//                 DisplayNameBuffer->Length = nLength * sizeof(WCHAR);
//                 DisplayNameBuffer->MaximumLength = DisplayNameBuffer->Length + sizeof(WCHAR);
//
//                 DisplayNameBuffer->Buffer = MIDL_user_allocate(DisplayNameBuffer->MaximumLength);
//                 if (DisplayNameBuffer->Buffer == NULL)
//                 {
//                     MIDL_user_free(DisplayNameBuffer);
//                     return STATUS_NO_MEMORY;
//                 }
//
//                 LsapLoadStringEx(hInstance,
//                                  IDS_CREATE_TOKEN_PRIVILEGE + Index,
//                                  ClientLanguage,
//                                  DisplayNameBuffer->Buffer,
//                                  nLength);
//
//                 *DisplayName = DisplayNameBuffer;
//                 *LanguageReturned = ClientLanguage;
//             }
//             else
//             {
//                 nLength = (LsapGetResourceStringLengthEx(hInstance,
//                                                         IDS_CREATE_TOKEN_PRIVILEGE + Index,
//                                                         ClientSystemDefaultLanguage)) as UINT;
//                 if (nLength != 0)
//                 {
//                     DisplayNameBuffer = MIDL_user_allocate(sizeof(RPC_UNICODE_STRING));
//                     if (DisplayNameBuffer == NULL)
//                         return STATUS_NO_MEMORY;
//
//                     DisplayNameBuffer->Length = nLength * sizeof(WCHAR);
//                     DisplayNameBuffer->MaximumLength = DisplayNameBuffer->Length + sizeof(WCHAR);
//
//                     DisplayNameBuffer->Buffer = MIDL_user_allocate(DisplayNameBuffer->MaximumLength);
//                     if (DisplayNameBuffer->Buffer == NULL)
//                     {
//                         MIDL_user_free(DisplayNameBuffer);
//                         return STATUS_NO_MEMORY;
//                     }
//
//                     LsapLoadStringEx(hInstance,
//                                      IDS_CREATE_TOKEN_PRIVILEGE + Index,
//                                      ClientSystemDefaultLanguage,
//                                      DisplayNameBuffer->Buffer,
//                                      nLength);
//
//                     *DisplayName = DisplayNameBuffer;
//                     *LanguageReturned = ClientSystemDefaultLanguage;
//                 }
//                 else
//                 {
//                     return STATUS_INVALID_PARAMETER;
//                 }
//             }
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     return STATUS_NO_SUCH_PRIVILEGE;
// }
//
//
// PLUID
// LsarpLookupPrivilegeValue(
//     IN PRPC_UNICODE_STRING Name)
// {
//     ULONG Priv;
//
//     if (Name->Length == 0 || Name->Buffer == NULL)
//         return NULL;
//
//     for (Priv = 0; Priv < ARRAYSIZE(WellKnownPrivileges); Priv++)
//     {
//         if (_wcsicmp(Name->Buffer, WellKnownPrivileges[Priv].Name) == 0)
//             return (PLUID)&(WellKnownPrivileges[Priv].Luid);
//     }
//
//     return NULL;
// }
//
//
// NTSTATUS
// LsarpEnumeratePrivileges(DWORD *EnumerationContext,
//                          PLSAPR_PRIVILEGE_ENUM_BUFFER EnumerationBuffer,
//                          DWORD PreferedMaximumLength)
// {
//     PLSAPR_POLICY_PRIVILEGE_DEF Privileges = NULL;
//     ULONG EnumIndex;
//     ULONG EnumCount = 0;
//     ULONG RequiredLength = 0;
//     ULONG i;
//     BOOLEAN MoreEntries = FALSE;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     EnumIndex = *EnumerationContext;
//
//     for (; EnumIndex < ARRAYSIZE(WellKnownPrivileges); EnumIndex++)
//     {
//         TRACE("EnumIndex: %lu\n", EnumIndex);
//         TRACE("Privilege Name: %S\n", WellKnownPrivileges[EnumIndex].Name);
//         TRACE("Name Length: %lu\n", wcslen(WellKnownPrivileges[EnumIndex].Name));
//
//         if ((RequiredLength +
//              wcslen(WellKnownPrivileges[EnumIndex].Name) * sizeof(WCHAR) +
//              sizeof(UNICODE_NULL) +
//              sizeof(LSAPR_POLICY_PRIVILEGE_DEF)) > PreferedMaximumLength)
//         {
//             MoreEntries = TRUE;
//             break;
//         }
//
//         RequiredLength += (wcslen(WellKnownPrivileges[EnumIndex].Name) * sizeof(WCHAR) +
//                            sizeof(UNICODE_NULL) + sizeof(LSAPR_POLICY_PRIVILEGE_DEF));
//         EnumCount++;
//     }
//
//     TRACE("EnumCount: %lu\n", EnumCount);
//     TRACE("RequiredLength: %lu\n", RequiredLength);
//
//     if (EnumCount == 0)
//         goto done;
//
//     Privileges = MIDL_user_allocate(EnumCount * sizeof(LSAPR_POLICY_PRIVILEGE_DEF));
//     if (Privileges == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     EnumIndex = *EnumerationContext;
//
//     for (i = 0; i < EnumCount; i++, EnumIndex++)
//     {
//         Privileges[i].LocalValue = WellKnownPrivileges[EnumIndex].Luid;
//
//         Privileges[i].Name.Length = (USHORT)wcslen(WellKnownPrivileges[EnumIndex].Name) * sizeof(WCHAR);
//         Privileges[i].Name.MaximumLength = (USHORT)Privileges[i].Name.Length + sizeof(UNICODE_NULL);
//
//         Privileges[i].Name.Buffer = MIDL_user_allocate(Privileges[i].Name.MaximumLength);
//         if (Privileges[i].Name.Buffer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto done;
//         }
//
//         memcpy(Privileges[i].Name.Buffer,
//                WellKnownPrivileges[EnumIndex].Name,
//                Privileges[i].Name.Length);
//     }
//
// done:
//     if (NT_SUCCESS(Status))
//     {
//         EnumerationBuffer->Entries = EnumCount;
//         EnumerationBuffer->Privileges = Privileges;
//         *EnumerationContext += EnumCount;
//     }
//     else
//     {
//         if (Privileges != NULL)
//         {
//             for (i = 0; i < EnumCount; i++)
//             {
//                 if (Privileges[i].Name.Buffer != NULL)
//                     MIDL_user_free(Privileges[i].Name.Buffer);
//             }
//
//             MIDL_user_free(Privileges);
//         }
//     }
//
//     if ((Status == STATUS_SUCCESS) && (MoreEntries != FALSE))
//         Status = STATUS_MORE_ENTRIES;
//
//     return Status;
// }
//
//
// NTSTATUS
// LsapLookupAccountRightName(ULONG RightValue,
//                            PRPC_UNICODE_STRING *Name)
// {
//     PRPC_UNICODE_STRING NameBuffer;
//     ULONG i;
//
//     for (i = 0; i < ARRAYSIZE(WellKnownRights); i++)
//     {
//         if (WellKnownRights[i].Flag == RightValue)
//         {
//             NameBuffer = MIDL_user_allocate(sizeof(RPC_UNICODE_STRING));
//             if (NameBuffer == NULL)
//                 return STATUS_NO_MEMORY;
//
//             NameBuffer->Length = (USHORT)wcslen(WellKnownRights[i].Name) * sizeof(WCHAR);
//             NameBuffer->MaximumLength = NameBuffer->Length + sizeof(WCHAR);
//
//             NameBuffer->Buffer = MIDL_user_allocate(NameBuffer->MaximumLength);
//             if (NameBuffer->Buffer == NULL)
//             {
//                 MIDL_user_free(NameBuffer);
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             wcscpy(NameBuffer->Buffer, WellKnownRights[i].Name);
//
//             *Name = NameBuffer;
//
//             return STATUS_SUCCESS;
//         }
//     }
//
//     return STATUS_NO_SUCH_PRIVILEGE;
// }
//
//
// ACCESS_MASK
// LsapLookupAccountRightValue(
//     IN PRPC_UNICODE_STRING Name)
// {
//     ULONG i;
//
//     if (Name->Length == 0 || Name->Buffer == NULL)
//         return 0;
//
//     for (i = 0; i < ARRAYSIZE(WellKnownRights); i++)
//     {
//         if (_wcsicmp(Name->Buffer, WellKnownRights[i].Name) == 0)
//             return WellKnownRights[i].Flag;
//     }
//
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
