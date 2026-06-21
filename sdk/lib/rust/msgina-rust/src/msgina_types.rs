//! GINA-specific types for msgina-rust (PGINA_CONTEXT, the GINA_UI dispatch table,
//! the PFGINA_* callback typedefs, and the ntsecapi LSA_STRING/LSA_OPERATIONAL_MODE
//! types) that are private to msgina.h / not surfaced by ntdll-sys. Layouts are taken
//! verbatim from dll/win32/msgina/msgina.h and sdk/include/.../ntsecapi.h.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::prelude::*;

// ---- ntsecapi LSA client types (not in ntdll-sys) ----
// LSA_STRING is an alias of the ANSI STRING; LSA_OPERATIONAL_MODE is a ULONG.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LSA_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PCHAR,
}
pub type PLSA_STRING = *mut LSA_STRING;
pub type LSA_OPERATIONAL_MODE = ULONG;
pub type PLSA_OPERATIONAL_MODE = *mut ULONG;

// ---- GINA context (msgina.h) ----
#[repr(C)]
pub struct GINA_CONTEXT {
    pub hWlx: HANDLE,
    pub station: LPWSTR,
    pub pWlxFuncs: PWLX_DISPATCH_VERSION_1_3,
    pub hDllInstance: HANDLE,
    pub hStatusWindow: HWND,
    pub LsaHandle: HANDLE,
    pub AuthenticationPackage: ULONG,
    pub bDisableCAD: BOOL,
    pub bAutoAdminLogon: BOOL,
    pub bDontDisplayLastUserName: BOOL,
    pub bShutdownWithoutLogon: BOOL,
    pub bIgnoreShiftOverride: BOOL,
    pub nShutdownAction: ULONG,
    pub UserName: [WCHAR; 256],
    pub DomainName: [WCHAR; 256],
    pub Password: [WCHAR; 256],
    pub LogonTime: SYSTEMTIME,
    pub UserToken: HANDLE,
    pub pAuthenticationId: PLUID,
    pub pdwOptions: PDWORD,
    pub pMprNotifyInfo: PWLX_MPR_NOTIFY_INFO,
    pub pProfile: *mut PVOID,
}
pub type PGINA_CONTEXT = *mut GINA_CONTEXT;
// c2rust emits the bindgen-style tag alias too.
pub type _GINA_CONTEXT = GINA_CONTEXT;

// ---- GINA UI dispatch table (msgina.h) ----
// The PFGINA_* callbacks are plain (cdecl) C function pointers in msgina.h.
pub type PFGINA_INITIALIZE = Option<unsafe extern "C" fn(PGINA_CONTEXT) -> BOOL>;
pub type PFGINA_DISPLAYSTATUSMESSAGE =
    Option<unsafe extern "C" fn(PGINA_CONTEXT, HDESK, DWORD, PWSTR, PWSTR) -> BOOL>;
pub type PFGINA_REMOVESTATUSMESSAGE = Option<unsafe extern "C" fn(PGINA_CONTEXT) -> BOOL>;
pub type PFGINA_DISPLAYSASNOTICE = Option<unsafe extern "C" fn(PGINA_CONTEXT)>;
pub type PFGINA_LOGGEDONSAS = Option<unsafe extern "C" fn(PGINA_CONTEXT, DWORD) -> INT>;
pub type PFGINA_LOGGEDOUTSAS = Option<unsafe extern "C" fn(PGINA_CONTEXT) -> INT>;
pub type PFGINA_LOCKEDSAS = Option<unsafe extern "C" fn(PGINA_CONTEXT) -> INT>;
pub type PFGINA_DISPLAYLOCKEDNOTICE = Option<unsafe extern "C" fn(PGINA_CONTEXT)>;

#[repr(C)]
pub struct GINA_UI {
    pub Initialize: PFGINA_INITIALIZE,
    pub DisplayStatusMessage: PFGINA_DISPLAYSTATUSMESSAGE,
    pub RemoveStatusMessage: PFGINA_REMOVESTATUSMESSAGE,
    pub DisplaySASNotice: PFGINA_DISPLAYSASNOTICE,
    pub LoggedOnSAS: PFGINA_LOGGEDONSAS,
    pub LoggedOutSAS: PFGINA_LOGGEDOUTSAS,
    pub LockedSAS: PFGINA_LOCKEDSAS,
    pub DisplayLockedNotice: PFGINA_DISPLAYLOCKEDNOTICE,
}
pub type PGINA_UI = *mut GINA_UI;
pub type _GINA_UI = GINA_UI;
