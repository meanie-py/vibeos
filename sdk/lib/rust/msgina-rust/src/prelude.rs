//! Crate-wide prelude of the universal symbols the per-file translations expect in
//! scope (NT/Win32 base types, common constants, no-op Wine debug shims). Per-file
//! modules add `use crate::prelude::*;`. Subsystem-specific GINA types live in
//! `msgina_types`; cross-DLL out-calls live in `msgina_externs`.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;

// ---- universal type aliases the translation output uses ----
pub type VOID = core::ffi::c_void;
pub type size_t = usize;
pub use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong};

// wine/debug.h `struct __wine_debug_channel { unsigned char flags; char name[15]; }`.
// The c2rust file-scope preamble emits a per-file `__wine_dbch_<module>` static of
// this type for the WINE_DEFAULT_DEBUG_CHANNEL declaration. The actual ERR/TRACE/WARN
// calls are no-op'd by the post-processor, so these statics are dead — but they must
// still type-check. The layout is fixed in wine/debug.h (name is always 15 bytes).
#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: c_uchar,
    pub name: [c_char; 15],
}

// `enum __wine_debug_class` (wine/debug.h). c2rust expands WINE TRACE/ERR/WARN/FIXME
// into `if (channel.flags & (1<<class)) { ros_dbg_log(class, channel, ...) }`, so the
// anchored bodies reference these symbolically. ros_dbg_log is the libwine logger
// (sdk/lib/libwine/debug.c), resolved at cohab link via msgina's `wine` lib.
// Default channel flags for the c2rust wine-debug preamble (post-processor rewrites
// c2rust's hardcoded `flags: !0` to this). 0 = every trace class off (silent, a single
// flag-test per site — what normal/smoke builds want); the `wine_debug` cargo feature
// flips it to 0xFF (all classes → full ERR/WARN/TRACE/FIXME via libwine ros_dbg_log,
// for debugging the logon path). Gates BOTH the anchored `if (flags & class)` form and
// the rewritten ERR!/TRACE! macros from one knob.
#[cfg(feature = "wine_debug")]
pub const WINE_DBG_DEFAULT_FLAGS: c_uchar = 0xFF;
#[cfg(not(feature = "wine_debug"))]
pub const WINE_DBG_DEFAULT_FLAGS: c_uchar = 0;

pub type __wine_debug_class = c_uint;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;

extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        channel: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        format: *const c_char,
        ...
    ) -> c_int;
}

// ---- universal NT / Win32 constants ----
pub const UNICODE_NULL: u16 = 0;
pub const NULL: PVOID = core::ptr::null_mut();

// NT_SUCCESS(Status) — Status >= 0.
#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- no-op Wine debug shims (TRACE/ERR/WARN/FIXME are DPRINT-class no-ops) ----
#[macro_export]
macro_rules! WINE_TRACE { ($($t:tt)*) => {{}}; }
#[macro_export]
macro_rules! WINE_ERR { ($($t:tt)*) => {{}}; }

// ---- WLX GINA dispatch ABI: curated re-export from winlogon-sys (NOT a glob —
// globbing winlogon_sys::nt would collide with the ntdll_sys::nt glob on shared
// base types like HANDLE/BOOL). Every WLX_/PWLX_/_WLX_ symbol the GINA code uses.
pub use winlogon_sys::nt::{
    PWLX_ASSIGN_SHELL_PROTECTION,
    PWLX_CHANGE_PASSWORD_NOTIFY,
    PWLX_CHANGE_PASSWORD_NOTIFY_EX,
    PWLX_CLIENT_CREDENTIALS_INFO_V1_0,
    PWLX_CLIENT_CREDENTIALS_INFO_V2_0,
    PWLX_CLOSE_USER_DESKTOP,
    PWLX_CONSOLESWITCH_CREDENTIALS_INFO_V1_0,
    PWLX_CREATE_USER_DESKTOP,
    PWLX_DESKTOP,
    PWLX_DIALOG_BOX,
    PWLX_DIALOG_BOX_INDIRECT,
    PWLX_DIALOG_BOX_INDIRECT_PARAM,
    PWLX_DIALOG_BOX_PARAM,
    PWLX_DISCONNECT,
    PWLX_DISPATCH_VERSION_1_0,
    PWLX_DISPATCH_VERSION_1_1,
    PWLX_DISPATCH_VERSION_1_2,
    PWLX_DISPATCH_VERSION_1_3,
    PWLX_DISPATCH_VERSION_1_4,
    PWLX_GET_OPTION,
    PWLX_GET_SOURCE_DESKTOP,
    PWLX_MESSAGE_BOX,
    PWLX_MPR_NOTIFY_INFO,
    PWLX_NOTIFICATION_INFO,
    PWLX_PROFILE_V1_0,
    PWLX_PROFILE_V2_0,
    PWLX_QUERY_CLIENT_CREDENTIALS,
    PWLX_QUERY_CONSOLESWITCH_CREDENTIALS,
    PWLX_QUERY_IC_CREDENTIALS,
    PWLX_QUERY_TERMINAL_SERVICES_DATA,
    PWLX_QUERY_TS_LOGON_CREDENTIALS,
    PWLX_SAS_NOTIFY,
    PWLX_SC_NOTIFICATION_INFO,
    PWLX_SET_CONTEXT_POINTER,
    PWLX_SET_OPTION,
    PWLX_SET_RETURN_DESKTOP,
    PWLX_SET_TIMEOUT,
    PWLX_SWITCH_DESKTOP_TO_USER,
    PWLX_SWITCH_DESKTOP_TO_WINLOGON,
    PWLX_TERMINAL_SERVICES_DATA,
    PWLX_USE_CTRL_ALT_DEL,
    PWLX_WIN31_MIGRATE,
    WLX_CLIENT_CREDENTIALS_INFO_V1_0,
    WLX_CLIENT_CREDENTIALS_INFO_V2_0,
    WLX_CONSOLESWITCHCREDENTIAL_TYPE_V1_0,
    WLX_CONSOLESWITCH_CREDENTIALS_INFO_V1_0,
    WLX_CREATE_INSTANCE_ONLY,
    WLX_CREATE_USER,
    WLX_CREDENTIAL_TYPE_V1_0,
    WLX_CREDENTIAL_TYPE_V2_0,
    WLX_CURRENT_VERSION,
    WLX_DESKTOP,
    WLX_DESKTOP_HANDLE,
    WLX_DESKTOP_NAME,
    WLX_DIRECTORY_LENGTH,
    WLX_DISPATCH_VERSION_1_0,
    WLX_DISPATCH_VERSION_1_1,
    WLX_DISPATCH_VERSION_1_2,
    WLX_DISPATCH_VERSION_1_3,
    WLX_DISPATCH_VERSION_1_4,
    WLX_DLG_INPUT_TIMEOUT,
    WLX_DLG_SAS,
    WLX_DLG_SCREEN_SAVER_TIMEOUT,
    WLX_DLG_USER_LOGOFF,
    WLX_LOGON_OPT_NO_PROFILE,
    WLX_MPR_NOTIFY_INFO,
    WLX_NOTIFICATION_INFO,
    WLX_OPTION_CONTEXT_POINTER,
    WLX_OPTION_DISPATCH_TABLE_SIZE,
    WLX_OPTION_FORCE_LOGOFF_TIME,
    WLX_OPTION_IGNORE_AUTO_LOGON,
    WLX_OPTION_NO_SWITCH_ON_SAS,
    WLX_OPTION_SMART_CARD_INFO,
    WLX_OPTION_SMART_CARD_PRESENT,
    WLX_OPTION_USE_CTRL_ALT_DEL,
    WLX_OPTION_USE_SMART_CARD,
    WLX_PROFILE_TYPE_V1_0,
    WLX_PROFILE_TYPE_V2_0,
    WLX_PROFILE_V1_0,
    WLX_PROFILE_V2_0,
    WLX_SAS_ACTION_DELAYED_FORCE_LOGOFF,
    WLX_SAS_ACTION_FORCE_LOGOFF,
    WLX_SAS_ACTION_LOCK_WKSTA,
    WLX_SAS_ACTION_LOGOFF,
    WLX_SAS_ACTION_LOGON,
    WLX_SAS_ACTION_NONE,
    WLX_SAS_ACTION_PWD_CHANGED,
    WLX_SAS_ACTION_RECONNECTED,
    WLX_SAS_ACTION_SHUTDOWN,
    WLX_SAS_ACTION_SHUTDOWN_HIBERNATE,
    WLX_SAS_ACTION_SHUTDOWN_POWER_OFF,
    WLX_SAS_ACTION_SHUTDOWN_REBOOT,
    WLX_SAS_ACTION_SHUTDOWN_SLEEP,
    WLX_SAS_ACTION_SHUTDOWN_SLEEP2,
    WLX_SAS_ACTION_SWITCH_CONSOLE,
    WLX_SAS_ACTION_TASKLIST,
    WLX_SAS_ACTION_UNLOCK_WKSTA,
    WLX_SAS_TYPE_AUTHENTICATED,
    WLX_SAS_TYPE_CTRL_ALT_DEL,
    WLX_SAS_TYPE_MAX_MSFT_VALUE,
    WLX_SAS_TYPE_SCRNSVR_ACTIVITY,
    WLX_SAS_TYPE_SCRNSVR_TIMEOUT,
    WLX_SAS_TYPE_SC_FIRST_READER_ARRIVED,
    WLX_SAS_TYPE_SC_INSERT,
    WLX_SAS_TYPE_SC_LAST_READER_REMOVED,
    WLX_SAS_TYPE_SC_REMOVE,
    WLX_SAS_TYPE_SWITCHUSER,
    WLX_SAS_TYPE_TIMEOUT,
    WLX_SAS_TYPE_USER_LOGOFF,
    WLX_SC_NOTIFICATION_INFO,
    WLX_TERMINAL_SERVICES_DATA,
    WLX_VERSION_1_0,
    WLX_VERSION_1_1,
    WLX_VERSION_1_2,
    WLX_VERSION_1_3,
    WLX_VERSION_1_4,
    WLX_WM_SAS,
};

// strsafe.h's VARIADIC printf entry points, declared at the crate root (see lib.rs for why they
// stopped being macros and what that macro was doing wrong). Re-exported here so the per-file
// `use crate::prelude::*` reaches them exactly as the macros' textual scope used to.
pub use crate::{StringCbPrintfW, StringCchPrintfW};
