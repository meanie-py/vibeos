//! GINA-specific types for msgina-rust (PGINA_CONTEXT, the GINA_UI dispatch table,
//! the PFGINA_* callback typedefs, and the ntsecapi LSA_STRING/LSA_OPERATIONAL_MODE
//! types) that are private to msgina.h / not surfaced by ntdll-sys. Layouts are taken
//! verbatim from dll/win32/msgina/msgina.h and sdk/include/.../ntsecapi.h.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::prelude::*;

// ---- ntsecapi LSA client types ----
// LSA_STRING is, per ntsecapi.h, a typedef of the ANSI STRING (_STRING). Aliasing it
// to ntdll-sys's `_STRING` keeps it the SAME Rust type as PANSI_STRING/PSTRING, so the
// translation's `as PANSI_STRING` casts and the LSA externs' PLSA_STRING params unify.
pub type LSA_STRING = _STRING;
pub type PLSA_STRING = *mut _STRING;
pub type LSA_OPERATIONAL_MODE = ULONG;
pub type PLSA_OPERATIONAL_MODE = *mut ULONG;

// LSA_UNICODE_STRING is an alias of UNICODE_STRING; LSA_HANDLE is a PVOID (ntsecapi.h).
pub type LSA_UNICODE_STRING = _UNICODE_STRING;
pub type PLSA_UNICODE_STRING = *mut _UNICODE_STRING;
pub type LSA_HANDLE = HANDLE;
pub type PLSA_HANDLE = *mut HANDLE;

// ---- ntsecapi.h MSV1_0 message structs (field-exact) ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSV1_0_INTERACTIVE_LOGON {
    pub MessageType: ULONG, // MSV1_0_LOGON_SUBMIT_TYPE (C enum == int/ULONG)
    pub LogonDomainName: _UNICODE_STRING,
    pub UserName: _UNICODE_STRING,
    pub Password: _UNICODE_STRING,
}
pub type PMSV1_0_INTERACTIVE_LOGON = *mut MSV1_0_INTERACTIVE_LOGON;
pub type _MSV1_0_INTERACTIVE_LOGON = MSV1_0_INTERACTIVE_LOGON;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSV1_0_INTERACTIVE_PROFILE {
    pub MessageType: ULONG, // MSV1_0_PROFILE_BUFFER_TYPE
    pub LogonCount: USHORT,
    pub BadPasswordCount: USHORT,
    pub LogonTime: LARGE_INTEGER,
    pub LogoffTime: LARGE_INTEGER,
    pub KickOffTime: LARGE_INTEGER,
    pub PasswordLastSet: LARGE_INTEGER,
    pub PasswordCanChange: LARGE_INTEGER,
    pub PasswordMustChange: LARGE_INTEGER,
    pub LogonScript: _UNICODE_STRING,
    pub HomeDirectory: _UNICODE_STRING,
    pub FullName: _UNICODE_STRING,
    pub ProfilePath: _UNICODE_STRING,
    pub HomeDirectoryDrive: _UNICODE_STRING,
    pub LogonServer: _UNICODE_STRING,
    pub UserFlags: ULONG,
}
pub type PMSV1_0_INTERACTIVE_PROFILE = *mut MSV1_0_INTERACTIVE_PROFILE;
pub type _MSV1_0_INTERACTIVE_PROFILE = MSV1_0_INTERACTIVE_PROFILE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DOMAIN_PASSWORD_INFORMATION {
    pub MinPasswordLength: USHORT,
    pub PasswordHistoryLength: USHORT,
    pub PasswordProperties: ULONG,
    pub MaxPasswordAge: LARGE_INTEGER,
    pub MinPasswordAge: LARGE_INTEGER,
}
pub type PDOMAIN_PASSWORD_INFORMATION = *mut DOMAIN_PASSWORD_INFORMATION;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSV1_0_CHANGEPASSWORD_REQUEST {
    pub MessageType: ULONG, // MSV1_0_PROTOCOL_MESSAGE_TYPE
    pub DomainName: _UNICODE_STRING,
    pub AccountName: _UNICODE_STRING,
    pub OldPassword: _UNICODE_STRING,
    pub NewPassword: _UNICODE_STRING,
    pub Impersonating: BOOLEAN,
}
pub type PMSV1_0_CHANGEPASSWORD_REQUEST = *mut MSV1_0_CHANGEPASSWORD_REQUEST;
pub type _MSV1_0_CHANGEPASSWORD_REQUEST = MSV1_0_CHANGEPASSWORD_REQUEST;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSV1_0_CHANGEPASSWORD_RESPONSE {
    pub MessageType: ULONG, // MSV1_0_PROTOCOL_MESSAGE_TYPE
    pub PasswordInfoValid: BOOLEAN,
    pub DomainPasswordInfo: DOMAIN_PASSWORD_INFORMATION,
}
pub type PMSV1_0_CHANGEPASSWORD_RESPONSE = *mut MSV1_0_CHANGEPASSWORD_RESPONSE;
pub type _MSV1_0_CHANGEPASSWORD_RESPONSE = MSV1_0_CHANGEPASSWORD_RESPONSE;

// ---- BITMAP / WLX_PROFILE_V2_0 re-exports from winlogon-sys (tag forms) ----
pub use winlogon_sys::nt::{tagBITMAP, BITMAP, _WLX_PROFILE_V2_0};

// ---- commctrl.h TTTOOLINFOW (tagTOOLINFOW) — field-exact ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TTTOOLINFOW {
    pub cbSize: UINT,
    pub uFlags: UINT,
    pub hwnd: HWND,
    pub uId: UINT_PTR,
    pub rect: RECT,
    pub hinst: HINSTANCE,
    pub lpszText: LPWSTR,
    pub lParam: LPARAM,
    pub lpReserved: *mut core::ffi::c_void,
}
pub type PTOOLINFOW = *mut TTTOOLINFOW;
pub type LPTTTOOLINFOW = *mut TTTOOLINFOW;

// ---- GINA context (msgina.h) ----
#[repr(C)]
pub struct GINA_CONTEXT {
    pub hWlx: HANDLE,
    pub station: LPWSTR,
    pub pWlxFuncs: PWLX_DISPATCH_VERSION_1_3,
    // C type is HANDLE; it always holds the HINSTANCE module handle (assigned from the
    // `hDllInstance` global) and is passed to HINSTANCE-typed Win32 APIs. Typed HINSTANCE
    // here so Rust's distinct pointer types line up (HANDLE/HINSTANCE are both void* in C).
    pub hDllInstance: HINSTANCE,
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
