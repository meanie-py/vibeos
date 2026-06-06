//! svcctl.idl + winsvc.h RPC marshalling types used by rpcserver.rs.
//! Mirrors of the generated svcctl_s.h typedefs + the public winsvc.h service
//! structs (repr(C), layout-exact for the RPC wire format).

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::types::SERVICE_STATUS;
use winsrv_sys::winsrv::{DWORD, LPSTR, LPWSTR, PVOID, PWSTR, UCHAR};

// --- svcctl.idl scalar typedefs ---
pub type SVCCTL_HANDLEA = LPSTR;
pub type SVCCTL_HANDLEW = LPWSTR;
pub type SC_RPC_HANDLE = PVOID;
pub type LPSC_RPC_HANDLE = *mut SC_RPC_HANDLE;
pub type RPC_SERVICE_STATUS_HANDLE = usize; // ULONG_PTR
pub type SC_NOTIFY_RPC_HANDLE = PVOID;
pub type LPSC_NOTIFY_RPC_HANDLE = *mut SC_NOTIFY_RPC_HANDLE;

pub type BOUNDED_DWORD_4K = DWORD;
pub type LPBOUNDED_DWORD_4K = *mut BOUNDED_DWORD_4K;
pub type BOUNDED_DWORD_8K = DWORD;
pub type LPBOUNDED_DWORD_8K = *mut BOUNDED_DWORD_8K;
pub type BOUNDED_DWORD_256K = DWORD;
pub type LPBOUNDED_DWORD_256K = *mut BOUNDED_DWORD_256K;

// --- winsvc.h access-rights / state / config-level constants ---
pub const SC_MANAGER_CONNECT: DWORD = 1;
pub const SC_MANAGER_CREATE_SERVICE: DWORD = 2;
pub const SC_MANAGER_ENUMERATE_SERVICE: DWORD = 4;
pub const SC_MANAGER_LOCK: DWORD = 8;
pub const SC_MANAGER_QUERY_LOCK_STATUS: DWORD = 16;
pub const SC_MANAGER_MODIFY_BOOT_CONFIG: DWORD = 32;
pub const SC_MANAGER_ALL_ACCESS: DWORD = 0xf003f;

pub const SERVICE_ACCEPT_PAUSE_CONTINUE: DWORD = 2;
pub const SERVICE_ACCEPT_PARAMCHANGE: DWORD = 8;
pub const SERVICE_ACCEPT_NETBINDCHANGE: DWORD = 16;
pub const SERVICE_ACCEPT_HARDWAREPROFILECHANGE: DWORD = 32;
pub const SERVICE_ACCEPT_POWEREVENT: DWORD = 64;
pub const SERVICE_ACCEPT_SESSIONCHANGE: DWORD = 128;

pub const SERVICE_ACTIVE: DWORD = 1;
pub const SERVICE_INACTIVE: DWORD = 2;
pub const SERVICE_STATE_ALL: DWORD = 3;

pub const SERVICE_CONFIG_DESCRIPTION: DWORD = 1;
pub const SERVICE_CONFIG_FAILURE_ACTIONS: DWORD = 2;
pub const SERVICE_CONFIG_DELAYED_AUTO_START_INFO: DWORD = 3;
pub const SERVICE_CONFIG_FAILURE_ACTIONS_FLAG: DWORD = 4;
pub const SERVICE_CONFIG_SERVICE_SID_INFO: DWORD = 5;
pub const SERVICE_CONFIG_REQUIRED_PRIVILEGES_INFO: DWORD = 6;
pub const SERVICE_CONFIG_PRESHUTDOWN_INFO: DWORD = 7;

// --- winsvc.h enums (C enum = c_int) ---
pub type SC_STATUS_TYPE = core::ffi::c_int;
pub const SC_STATUS_PROCESS_INFO: SC_STATUS_TYPE = 0;
pub type SC_ENUM_TYPE = core::ffi::c_int;
pub const SC_ENUM_PROCESS_INFO: SC_ENUM_TYPE = 0;
pub type TAG_INFO_LEVEL = core::ffi::c_int;
pub const TagInfoLevelNameFromTag: TAG_INFO_LEVEL = 1;
pub type SC_ACTION_TYPE = core::ffi::c_int;
pub const SC_ACTION_NONE: SC_ACTION_TYPE = 0;
pub const SC_ACTION_RESTART: SC_ACTION_TYPE = 1;
pub const SC_ACTION_REBOOT: SC_ACTION_TYPE = 2;
pub const SC_ACTION_RUN_COMMAND: SC_ACTION_TYPE = 3;

// --- winsvc.h structs ---
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_STATUS_PROCESS {
    pub dwServiceType: DWORD,
    pub dwCurrentState: DWORD,
    pub dwControlsAccepted: DWORD,
    pub dwWin32ExitCode: DWORD,
    pub dwServiceSpecificExitCode: DWORD,
    pub dwCheckPoint: DWORD,
    pub dwWaitHint: DWORD,
    pub dwProcessId: DWORD,
    pub dwServiceFlags: DWORD,
}
pub type LPSERVICE_STATUS_PROCESS = *mut SERVICE_STATUS_PROCESS;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUM_SERVICE_STATUSA {
    pub lpServiceName: LPSTR,
    pub lpDisplayName: LPSTR,
    pub ServiceStatus: SERVICE_STATUS,
}
pub type LPENUM_SERVICE_STATUSA = *mut ENUM_SERVICE_STATUSA;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUM_SERVICE_STATUSW {
    pub lpServiceName: LPWSTR,
    pub lpDisplayName: LPWSTR,
    pub ServiceStatus: SERVICE_STATUS,
}
pub type LPENUM_SERVICE_STATUSW = *mut ENUM_SERVICE_STATUSW;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUM_SERVICE_STATUS_PROCESSA {
    pub lpServiceName: LPSTR,
    pub lpDisplayName: LPSTR,
    pub ServiceStatusProcess: SERVICE_STATUS_PROCESS,
}
pub type LPENUM_SERVICE_STATUS_PROCESSA = *mut ENUM_SERVICE_STATUS_PROCESSA;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ENUM_SERVICE_STATUS_PROCESSW {
    pub lpServiceName: LPWSTR,
    pub lpDisplayName: LPWSTR,
    pub ServiceStatusProcess: SERVICE_STATUS_PROCESS,
}
pub type LPENUM_SERVICE_STATUS_PROCESSW = *mut ENUM_SERVICE_STATUS_PROCESSW;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QUERY_SERVICE_CONFIGA {
    pub dwServiceType: DWORD,
    pub dwStartType: DWORD,
    pub dwErrorControl: DWORD,
    pub lpBinaryPathName: LPSTR,
    pub lpLoadOrderGroup: LPSTR,
    pub dwTagId: DWORD,
    pub lpDependencies: LPSTR,
    pub lpServiceStartName: LPSTR,
    pub lpDisplayName: LPSTR,
}
pub type LPQUERY_SERVICE_CONFIGA = *mut QUERY_SERVICE_CONFIGA;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct QUERY_SERVICE_CONFIGW {
    pub dwServiceType: DWORD,
    pub dwStartType: DWORD,
    pub dwErrorControl: DWORD,
    pub lpBinaryPathName: LPWSTR,
    pub lpLoadOrderGroup: LPWSTR,
    pub dwTagId: DWORD,
    pub lpDependencies: LPWSTR,
    pub lpServiceStartName: LPWSTR,
    pub lpDisplayName: LPWSTR,
}
pub type LPQUERY_SERVICE_CONFIGW = *mut QUERY_SERVICE_CONFIGW;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_DESCRIPTIONA {
    pub lpDescription: LPSTR,
}
pub type LPSERVICE_DESCRIPTIONA = *mut SERVICE_DESCRIPTIONA;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_DESCRIPTIONW {
    pub lpDescription: LPWSTR,
}
pub type LPSERVICE_DESCRIPTIONW = *mut SERVICE_DESCRIPTIONW;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SC_ACTION {
    pub Type: SC_ACTION_TYPE,
    pub Delay: DWORD,
}
pub type LPSC_ACTION = *mut SC_ACTION;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_FAILURE_ACTIONSA {
    pub dwResetPeriod: DWORD,
    pub lpRebootMsg: LPSTR,
    pub lpCommand: LPSTR,
    pub cActions: DWORD,
    pub lpsaActions: *mut SC_ACTION,
}
pub type LPSERVICE_FAILURE_ACTIONSA = *mut SERVICE_FAILURE_ACTIONSA;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_FAILURE_ACTIONSW {
    pub dwResetPeriod: DWORD,
    pub lpRebootMsg: LPWSTR,
    pub lpCommand: LPWSTR,
    pub cActions: DWORD,
    pub lpsaActions: *mut SC_ACTION,
}
pub type LPSERVICE_FAILURE_ACTIONSW = *mut SERVICE_FAILURE_ACTIONSW;

// --- svcctl.idl SC_RPC_CONFIG_INFO (tagged union; all variants are pointers) ---
#[repr(C)]
#[derive(Clone, Copy)]
pub union SC_RPC_CONFIG_INFOW_u {
    pub psd: LPSERVICE_DESCRIPTIONW,
    pub psfa: LPSERVICE_FAILURE_ACTIONSW,
    pub psda: PVOID,
    pub psfaf: PVOID,
    pub pssid: PVOID,
    pub psrp: PVOID,
    pub psps: PVOID,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SC_RPC_CONFIG_INFOW {
    pub dwInfoLevel: DWORD,
    pub __bindgen_anon_1: SC_RPC_CONFIG_INFOW_u,
}
pub type LPSC_RPC_CONFIG_INFOW = *mut SC_RPC_CONFIG_INFOW;

#[repr(C)]
#[derive(Clone, Copy)]
pub union SC_RPC_CONFIG_INFOA_u {
    pub psd: LPSERVICE_DESCRIPTIONA,
    pub psfa: LPSERVICE_FAILURE_ACTIONSA,
    pub psda: PVOID,
    pub psfaf: PVOID,
    pub pssid: PVOID,
    pub psrp: PVOID,
    pub psps: PVOID,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SC_RPC_CONFIG_INFOA {
    pub dwInfoLevel: DWORD,
    pub __bindgen_anon_1: SC_RPC_CONFIG_INFOA_u,
}
pub type LPSC_RPC_CONFIG_INFOA = *mut SC_RPC_CONFIG_INFOA;

// --- svcctl.idl notify params ---
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_NOTIFY_STATUS_CHANGE_PARAMS_1 {
    pub ullThreadId: u64,
    pub dwNotifyMask: DWORD,
    pub CallbackAddressArray: [UCHAR; 16],
    pub CallbackParamAddressArray: [UCHAR; 16],
    pub ServiceStatus: SERVICE_STATUS_PROCESS,
    pub dwNotificationStatus: DWORD,
    pub dwSequence: DWORD,
}
pub type PSERVICE_NOTIFY_STATUS_CHANGE_PARAMS_1 = *mut SERVICE_NOTIFY_STATUS_CHANGE_PARAMS_1;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SERVICE_NOTIFY_STATUS_CHANGE_PARAMS_2 {
    pub ullThreadId: u64,
    pub dwNotifyMask: DWORD,
    pub CallbackAddressArray: [UCHAR; 16],
    pub CallbackParamAddressArray: [UCHAR; 16],
    pub ServiceStatus: SERVICE_STATUS_PROCESS,
    pub dwNotificationStatus: DWORD,
    pub dwSequence: DWORD,
    pub dwNotificationTriggered: DWORD,
    pub pszServiceNames: PWSTR,
}
pub type PSERVICE_NOTIFY_STATUS_CHANGE_PARAMS_2 = *mut SERVICE_NOTIFY_STATUS_CHANGE_PARAMS_2;

#[repr(C)]
#[derive(Clone, Copy)]
pub union SC_RPC_NOTIFY_PARAMS_u {
    pub pStatusChangeParam1: PSERVICE_NOTIFY_STATUS_CHANGE_PARAMS_1,
    pub pStatusChangeParam2: PSERVICE_NOTIFY_STATUS_CHANGE_PARAMS_2,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SC_RPC_NOTIFY_PARAMS {
    pub dwInfoLevel: DWORD,
    pub __bindgen_anon_1: SC_RPC_NOTIFY_PARAMS_u,
}
pub type LPSC_RPC_NOTIFY_PARAMS = *mut SC_RPC_NOTIFY_PARAMS;

#[repr(C)]
pub struct SC_RPC_NOTIFY_PARAMS_LIST {
    pub cElements: BOUNDED_DWORD_4K,
    pub NotifyParamsArray: [SC_RPC_NOTIFY_PARAMS; 1],
}
pub type PSC_RPC_NOTIFY_PARAMS_LIST = *mut SC_RPC_NOTIFY_PARAMS_LIST;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct STRING_PTRSA {
    pub StringPtr: LPSTR,
}
pub type PSTRING_PTRSA = *mut STRING_PTRSA;
pub type LPSTRING_PTRSA = *mut STRING_PTRSA;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct STRING_PTRSW {
    pub StringPtr: LPWSTR,
}
pub type PSTRING_PTRSW = *mut STRING_PTRSW;
pub type LPSTRING_PTRSW = *mut STRING_PTRSW;
