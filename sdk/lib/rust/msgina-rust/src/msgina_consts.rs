//! Win32 / NT / LSA constants used by the msgina translation, with values taken
//! verbatim from the psdk/ndk headers (winuser.h, wingdi.h, winreg.h, winbase.h,
//! commctrl.h, winnls.h, ntstatus.h, ntsecapi.h, dll/win32/msgina/msgina.h).
//! Body modules add `use crate::msgina_consts::*;`.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::prelude::*;

// ---- registry roots (winreg.h: ((HKEY)(LONG_PTR)(LONG)0x800000xx)) ----
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// ---- winuser.h / wingdi.h ----
pub const CW_USEDEFAULT: c_int = 0x8000_0000u32 as c_int; // ((int)0x80000000)
pub const TRANSPARENT: c_int = 1;
pub const HOLLOW_BRUSH: c_int = 5;
pub const FW_MEDIUM: c_int = 500;
pub const LOGPIXELSY: c_int = 90;
pub const HWND_TOP: HWND = core::ptr::null_mut(); // ((HWND)0)
// IDC_HAND == MAKEINTRESOURCE(32649) → a PWSTR resource id.
pub const IDC_HAND: PWSTR = 32649u16 as usize as PWSTR;

// ---- winbase.h ----
pub const STD_OUTPUT_HANDLE: DWORD = 0xffff_fff5; // (DWORD)(0xfffffff5) == -11

// ---- winnls.h / ntbasedef.h ----
pub const LOCALE_USER_DEFAULT: DWORD = 0x0400; // MAKELCID(LANG_USER_DEFAULT, SORT_DEFAULT)
pub const DATE_SHORTDATE: DWORD = 1;

// ---- winerror.h (HRESULT) ----
pub const S_OK: LONG = 0;

// ---- commctrl.h tooltips ----
// TOOLTIPS_CLASSW L"tooltips_class32" — a NUL-terminated wide string literal.
pub static TOOLTIPS_CLASSW: [u16; 17] = [
    b't' as u16, b'o' as u16, b'o' as u16, b'l' as u16, b't' as u16, b'i' as u16, b'p' as u16,
    b's' as u16, b'_' as u16, b'c' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16,
    b'3' as u16, b'2' as u16, 0,
];
pub const TTS_ALWAYSTIP: DWORD = 0x01;
pub const TTS_BALLOON: DWORD = 0x40;
pub const TTF_IDISHWND: UINT = 0x01;
pub const TTF_SUBCLASS: UINT = 0x10;
pub const TTI_NONE: c_int = 0;
const WM_USER: UINT = 0x0400; // 1024
pub const TTM_ADDTOOLW: UINT = WM_USER + 50;
pub const TTM_SETMAXTIPWIDTH: UINT = WM_USER + 24;
pub const TTM_SETTITLEW: UINT = WM_USER + 33;

// ---- WLX shutdown-state bits (dll/win32/msgina/msgina.h) ----
pub const WLX_SHUTDOWN_STATE_LOGOFF: DWORD = 0x01;
pub const WLX_SHUTDOWN_STATE_POWER_OFF: DWORD = 0x02;
pub const WLX_SHUTDOWN_STATE_REBOOT: DWORD = 0x04;
pub const WLX_SHUTDOWN_STATE_SLEEP: DWORD = 0x10;
pub const WLX_SHUTDOWN_STATE_SLEEP2: DWORD = 0x20;
pub const WLX_SHUTDOWN_STATE_HIBERNATE: DWORD = 0x40;
pub const WLX_SHUTDOWN_STATE_DISCONNECT: DWORD = 0x80;

// ---- NTSTATUS logon-failure codes (ntstatus.h) ----
pub const STATUS_ACCOUNT_RESTRICTION: NTSTATUS = 0xC000006Eu32 as NTSTATUS;
pub const STATUS_INVALID_LOGON_HOURS: NTSTATUS = 0xC000006Fu32 as NTSTATUS;
pub const STATUS_INVALID_WORKSTATION: NTSTATUS = 0xC0000070u32 as NTSTATUS;
pub const STATUS_PASSWORD_EXPIRED: NTSTATUS = 0xC0000071u32 as NTSTATUS;
pub const STATUS_ACCOUNT_DISABLED: NTSTATUS = 0xC0000072u32 as NTSTATUS;
pub const STATUS_ACCOUNT_EXPIRED: NTSTATUS = 0xC0000193u32 as NTSTATUS;
pub const STATUS_PASSWORD_MUST_CHANGE: NTSTATUS = 0xC0000224u32 as NTSTATUS;
pub const STATUS_ACCOUNT_LOCKED_OUT: NTSTATUS = 0xC0000234u32 as NTSTATUS;

// ---- LSA / MSV (ntsecapi.h) ----
pub const POLICY_GET_PRIVATE_INFORMATION: ULONG = 4;
// SECURITY_LOGON_TYPE::Interactive == 2 (passed to LsaLogonUser as the type arg).
pub const Interactive: ULONG = 2;
// MSV1_0_LOGON_SUBMIT_TYPE::MsV1_0InteractiveLogon == 2.
pub const MsV1_0InteractiveLogon: ULONG = 2;
// MSV1_0_PROTOCOL_MESSAGE_TYPE::MsV1_0ChangePassword == 5 (0-based: ...,MsV1_0ReLogonUsers=4,MsV1_0ChangePassword=5).
pub const MsV1_0ChangePassword: ULONG = 5;
