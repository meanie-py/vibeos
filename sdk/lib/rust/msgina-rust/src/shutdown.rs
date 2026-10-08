use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use crate::resources::*;
use crate::msgina_consts::*;
use crate::strsafe::*;
use crate::msgina::hDllInstance;
use crate::utils::{ReadRegDwordValue, ReadRegSzValue, RegOpenLoggedOnHKCU, TestTokenPrivilege, DuplicateString};
use core::ffi::c_void;

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


// All cross-DLL out-calls used here come from crate::msgina_externs (registry,
// gdi32, GetKeyState) — local re-declarations removed to avoid ABI/type drift.

// wine debug-channel statics (WINE_DEFAULT_DEBUG_CHANNEL(msgina) preamble).
static mut __wine_dbch_msgina: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"msgina\0\0\0\0\0\0\0\0\0",
        ),
    }
};
static mut __wine_dbch___default: *mut __wine_debug_channel =
    &raw const __wine_dbch_msgina as *mut __wine_debug_channel;

// ---- file-scope preamble (translated) ----
pub const FONT_POINT_SIZE: i32 = 13;
pub const DARK_GREY_COLOR: u32 = 0x00F4F4F4u32;
pub const LIGHT_GREY_COLOR: u32 = 0x00262626u32;
pub const CX_BITMAP: u32 = 33;
pub const CY_BITMAP: u32 = 33;
pub const NUMBER_OF_BUTTONS: usize = 4;
pub const BUTTON_SHUTDOWN: u32 = 0;
pub const BUTTON_SHUTDOWN_PRESSED: u32 = 33;
pub const BUTTON_SHUTDOWN_FOCUSED: u32 = 66;
pub const BUTTON_REBOOT: u32 = 99;
pub const BUTTON_REBOOT_PRESSED: u32 = 132;
pub const BUTTON_REBOOT_FOCUSED: u32 = 165;
pub const BUTTON_SLEEP: u32 = 198;
pub const BUTTON_SLEEP_PRESSED: u32 = 231;
pub const BUTTON_SLEEP_FOCUSED: u32 = 264;
pub const BUTTON_SLEEP_DISABLED: u32 = 297;
pub const SHUTDOWN_BUTTON_HOT: u32 = 0;
pub const REBOOT_BUTTON_HOT: u32 = 1;
pub const SLEEP_BUTTON_HOT: u32 = 2;
pub const HIBERNATE_BUTTON_HOT: u32 = 3;

#[repr(C)]
pub struct _SHUTDOWN_DLG_CONTEXT {
    pub pgContext: PGINA_CONTEXT,
    pub hBitmap: HBITMAP,
    pub hImageStrip: HBITMAP,
    pub ShutdownDialogId: DWORD,
    pub ShutdownOptions: DWORD,
    pub hBrush: HBRUSH,
    pub hfFont: HFONT,
    pub bCloseDlg: BOOL,
    pub bIsSleepButtonReplaced: BOOL,
    pub bReasonUI: BOOL,
    pub bFriendlyUI: BOOL,
    pub bIsButtonHot: [BOOL; NUMBER_OF_BUTTONS],
    pub bTimer: BOOL,
    pub iTimer: UINT_PTR,
    pub OldButtonProc: WNDPROC,
}
pub type SHUTDOWN_DLG_CONTEXT = _SHUTDOWN_DLG_CONTEXT;
pub type PSHUTDOWN_DLG_CONTEXT = *mut SHUTDOWN_DLG_CONTEXT;
#[no_mangle]
pub unsafe extern "C" fn GetPolicyDWORDValue(
    hKeyCurrentUser: HKEY,
    pPolicyKey: PCWSTR,
    ValueName: PCWSTR,
    pValue: PDWORD,
) -> LONG {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwValue: u32 = 0;
    let mut rc: LONG;

    *pValue = 0;

    rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, pPolicyKey, 0, KEY_QUERY_VALUE, &mut hKey) as i32;
    if rc == ERROR_SUCCESS.try_into().unwrap() {
        rc = ReadRegDwordValue(hKey, ValueName, &mut dwValue) as i32;
        RegCloseKey(hKey);
        if rc == ERROR_SUCCESS.try_into().unwrap() {
            *pValue = dwValue;
            return ERROR_SUCCESS.try_into().unwrap();
        }
    }

    let use_hkey = if hKeyCurrentUser != core::ptr::null_mut() {
        hKeyCurrentUser
    } else {
        HKEY_CURRENT_USER
    };
    rc = RegOpenKeyExW(use_hkey, pPolicyKey, 0, KEY_QUERY_VALUE, &mut hKey) as i32;
    if rc == ERROR_SUCCESS.try_into().unwrap() {
        rc = ReadRegDwordValue(hKey, ValueName, &mut dwValue) as i32;
        RegCloseKey(hKey);
        if rc == ERROR_SUCCESS.try_into().unwrap() {
            *pValue = dwValue;
            return ERROR_SUCCESS.try_into().unwrap();
        }
    }

    ERROR_NOT_FOUND.try_into().unwrap()
}

unsafe fn GetShutdownReasonUI() -> i32 {
    let mut dwValue: u32;
    let mut dwSize: u32;
    let mut hKey: HKEY;
    let mut lRet: i32;

    const HKLM: HKEY = 0x80000002usize as HKEY;
    let policy_key: &[u16] = &[
        0x53, 0x6f, 0x66, 0x74, 0x77, 0x61, 0x72, 0x65, 0x5c, 0x50, 0x6f, 0x6c, 0x69, 0x63, 0x69,
        0x65, 0x73, 0x5c, 0x4d, 0x69, 0x63, 0x72, 0x6f, 0x73, 0x6f, 0x66, 0x74, 0x5c, 0x57, 0x69,
        0x6e, 0x64, 0x6f, 0x77, 0x73, 0x20, 0x4e, 0x54, 0x5c, 0x52, 0x65, 0x6c, 0x69, 0x61, 0x62,
        0x69, 0x6c, 0x69, 0x74, 0x79, 0x0,
    ];
    let machine_key: &[u16] = &[
        0x53, 0x6f, 0x66, 0x74, 0x77, 0x61, 0x72, 0x65, 0x5c, 0x4d, 0x69, 0x63, 0x72, 0x6f, 0x73,
        0x6f, 0x66, 0x74, 0x5c, 0x57, 0x69, 0x6e, 0x64, 0x6f, 0x77, 0x73, 0x5c, 0x43, 0x75, 0x72,
        0x72, 0x65, 0x6e, 0x74, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x5c, 0x52, 0x65, 0x6c,
        0x69, 0x61, 0x62, 0x69, 0x6c, 0x69, 0x74, 0x79, 0x0,
    ];
    let value_name: &[u16] = &[
        0x53, 0x68, 0x75, 0x74, 0x64, 0x6f, 0x77, 0x6e, 0x52, 0x65, 0x61, 0x73, 0x6f, 0x6e, 0x55,
        0x49, 0x0,
    ];

    // Query the policy value
    hKey = core::ptr::null_mut();
    lRet = RegOpenKeyExW(
        HKLM,
        policy_key.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as i32;
    if lRet == ERROR_SUCCESS.try_into().unwrap() {
        dwValue = 0;
        dwSize = core::mem::size_of::<u32>() as u32;
        RegQueryValueExW(
            hKey,
            value_name.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwValue as *mut u32 as LPBYTE,
            &mut dwSize,
        );
        RegCloseKey(hKey);
        return if dwValue != 0 { 1 } else { 0 };
    }

    // Query the machine value
    hKey = core::ptr::null_mut();
    lRet = RegOpenKeyExW(
        HKLM,
        machine_key.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as i32;
    if lRet == ERROR_SUCCESS.try_into().unwrap() {
        dwValue = 0;
        dwSize = core::mem::size_of::<u32>() as u32;
        RegQueryValueExW(
            hKey,
            value_name.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwValue as *mut u32 as LPBYTE,
            &mut dwSize,
        );
        RegCloseKey(hKey);
        return if dwValue != 0 { 1 } else { 0 };
    }

    // Return the default value
    let mut VersionInfo: OSVERSIONINFOEX = core::mem::zeroed();
    VersionInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEX>() as u32;
    if GetVersionEx!(&mut VersionInfo as *mut OSVERSIONINFOEX as *mut core::ffi::c_void) == 0 {
        return 0;
    }
    0
}

unsafe fn IsFriendlyUIActive() -> i32 {
    const HKLM: usize = 0x80000002;
    const KEY_QUERY_VALUE: u32 = 0x0001;
    const REG_DWORD: u32 = 4;

    const SYS_PATH: [u16; 41] = [
        0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C,
        0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074,
        0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
        0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E,
        0x0074, 0x0072, 0x006F, 0x006C, 0x005C, 0x0057, 0x0069,
        0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0000,
    ];

    const ENFORCE: [u16; 24] = [
        0x0045, 0x006E, 0x0066, 0x006F, 0x0072, 0x0063, 0x0065,
        0x0046, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C,
        0x0079, 0x0053, 0x0068, 0x0075, 0x0074, 0x0064, 0x006F,
        0x0077, 0x006E, 0x0000,
    ];

    const CSD_VER: [u16; 11] = [
        0x0043, 0x0053, 0x0044, 0x0056, 0x0065, 0x0072, 0x0073,
        0x0069, 0x006F, 0x006E, 0x0000,
    ];

    const WINLOGON_PATH: [u16; 54] = [
        0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052,
        0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F,
        0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069,
        0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E,
        0x0054, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
        0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069,
        0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E, 0x006C,
        0x006F, 0x0067, 0x006F, 0x006E, 0x0000,
    ];

    const LOGON_TYPE: [u16; 10] = [
        0x004C, 0x006F, 0x0067, 0x006F, 0x006E, 0x0054, 0x0079,
        0x0070, 0x0065, 0x0000,
    ];

    let mut dw_type: u32 = 0;
    let mut dw_value: u32;
    let mut dw_size: u32;
    let mut h_key: HKEY = core::ptr::null_mut();
    let mut l_ret: i32;

    l_ret = RegOpenKeyExW(
        HKLM as HKEY,
        SYS_PATH.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut h_key,
    ) as i32;
    if l_ret != 0 {
        return 0;
    }

    dw_value = 0;
    dw_size = core::mem::size_of::<u32>() as u32;
    l_ret = RegQueryValueExW(
        h_key,
        ENFORCE.as_ptr(),
        core::ptr::null_mut(),
        &mut dw_type,
        &mut dw_value as *mut u32 as *mut u8,
        &mut dw_size,
    ) as i32;

    if l_ret == 0 && dw_type == REG_DWORD && dw_value == 1 {
        RegCloseKey(h_key);
        return 1;
    }

    dw_value = 0;
    dw_size = core::mem::size_of::<u32>() as u32;
    l_ret = RegQueryValueExW(
        h_key,
        CSD_VER.as_ptr(),
        core::ptr::null_mut(),
        &mut dw_type,
        &mut dw_value as *mut u32 as *mut u8,
        &mut dw_size,
    ) as i32;
    RegCloseKey(h_key);

    if l_ret != 0 || dw_type != REG_DWORD || dw_value != 0x300 {
        return 0;
    }

    l_ret = RegOpenKeyExW(
        HKLM as HKEY,
        WINLOGON_PATH.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut h_key,
    ) as i32;
    if l_ret != 0 {
        return 0;
    }

    dw_value = 0;
    dw_size = core::mem::size_of::<u32>() as u32;
    l_ret = RegQueryValueExW(
        h_key,
        LOGON_TYPE.as_ptr(),
        core::ptr::null_mut(),
        &mut dw_type,
        &mut dw_value as *mut u32 as *mut u8,
        &mut dw_size,
    ) as i32;
    RegCloseKey(h_key);

    if l_ret != 0 || dw_type != REG_DWORD {
        return 0;
    }

    (dw_value != 0) as i32
}

unsafe fn IsDomainMember() -> BOOL {
    // UNIMPLEMENTED
    0i32
}

unsafe fn IsNetwareActive() -> BOOLEAN {
    UNIMPLEMENTED!();
    return 0u8;
}

unsafe fn IsShowHibernateButtonActive() -> i32 {
    let reg_path: [u16; 52] = [
        b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16,
        b'W' as u16, b'A' as u16, b'R' as u16, b'E' as u16,
        b'\\' as u16, b'P' as u16, b'o' as u16, b'l' as u16,
        b'i' as u16, b'c' as u16, b'i' as u16, b'e' as u16,
        b's' as u16, b'\\' as u16, b'M' as u16, b'i' as u16,
        b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16,
        b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16,
        b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16,
        b'e' as u16, b'm' as u16, b'\\' as u16, b'S' as u16,
        b'h' as u16, b'u' as u16, b't' as u16, b'd' as u16,
        b'o' as u16, b'w' as u16, b'n' as u16, 0u16,
    ];

    let value_name: [u16; 20] = [
        b'S' as u16, b'h' as u16, b'o' as u16, b'w' as u16,
        b'H' as u16, b'i' as u16, b'b' as u16, b'e' as u16,
        b'r' as u16, b'n' as u16, b'a' as u16, b't' as u16,
        b'e' as u16, b'B' as u16, b'u' as u16, b't' as u16,
        b't' as u16, b'o' as u16, b'n' as u16, 0u16,
    ];

    let mut hKey: HKEY = core::ptr::null_mut();
    let lRet: i32 = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        reg_path.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey as *mut HKEY,
    ).try_into().unwrap();

    if lRet as u32 == ERROR_SUCCESS {
        let mut dwValue: u32 = 0;
        let mut dwSize: u32 = core::mem::size_of::<u32>() as u32;

        let lRet2: i32 = RegQueryValueExW(
            hKey,
            value_name.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwValue as *mut u32 as LPBYTE,
            &mut dwSize as *mut u32,
        ).try_into().unwrap();

        RegCloseKey(hKey);

        if lRet2 as u32 != ERROR_SUCCESS {
            return 0; // FALSE
        }
        return (dwValue != 0) as i32;
    }

    0 // FALSE
}

unsafe fn ForceFriendlyUI() -> u8 {

    static KEY_PATH1: [u16; 58] = [
        b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16, b'E' as u16,
        b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16,
        b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16,
        b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16,
        b'\\' as u16,
        b'P' as u16, b'o' as u16, b'l' as u16, b'i' as u16, b'c' as u16, b'i' as u16, b'e' as u16, b's' as u16,
        b'\\' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        0,
    ];

    static KEY_PATH2: [u16; 54] = [
        b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16, b'E' as u16,
        b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16,
        b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16, b'T' as u16,
        b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16,
        b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'l' as u16, b'o' as u16, b'g' as u16, b'o' as u16, b'n' as u16,
        0,
    ];

    static VALUE_NAME: [u16; 16] = [
        b'F' as u16, b'o' as u16, b'r' as u16, b'c' as u16, b'e' as u16,
        b'F' as u16, b'r' as u16, b'i' as u16, b'e' as u16, b'n' as u16, b'd' as u16, b'l' as u16, b'y' as u16,
        b'U' as u16, b'I' as u16,
        0,
    ];

    let mut dwType: DWORD = 0;
    let mut dwValue: DWORD;
    let mut dwSize: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lRet: LONG;

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        KEY_PATH1.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as i32;
    if lRet == ERROR_SUCCESS.try_into().unwrap() {
        dwValue = 0;
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lRet = RegQueryValueExW(
            hKey,
            VALUE_NAME.as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            &mut dwType,
            (&mut dwValue as *mut DWORD) as LPBYTE,
            &mut dwSize,
        ) as i32;
        RegCloseKey(hKey);
        if lRet == ERROR_SUCCESS.try_into().unwrap() && dwType == REG_DWORD {
            return if dwValue != 0 { 1u8 } else { 0u8 };
        }
    }

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        KEY_PATH2.as_ptr() as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as i32;
    if lRet == ERROR_SUCCESS.try_into().unwrap() {
        dwValue = 0;
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lRet = RegQueryValueExW(
            hKey,
            VALUE_NAME.as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            &mut dwType,
            (&mut dwValue as *mut DWORD) as LPBYTE,
            &mut dwSize,
        ) as i32;
        RegCloseKey(hKey);
        if lRet == ERROR_SUCCESS.try_into().unwrap() && dwType == REG_DWORD {
            return if dwValue != 0 { 1u8 } else { 0u8 };
        }
    }

    0u8
}

unsafe fn DrawIconOnOwnerDrawnButtons(
    pdis: *mut DRAWITEMSTRUCT,
    pContext: *mut SHUTDOWN_DLG_CONTEXT,
) -> BOOL {

    const CX_BITMAP: i32 = 33;
    const CY_BITMAP: i32 = 33;
    const BUTTON_SHUTDOWN: i32 = 0;
    const BUTTON_SHUTDOWN_PRESSED: i32 = BUTTON_SHUTDOWN + CY_BITMAP;
    const BUTTON_SHUTDOWN_FOCUSED: i32 = BUTTON_SHUTDOWN_PRESSED + CY_BITMAP;
    const BUTTON_REBOOT: i32 = BUTTON_SHUTDOWN_FOCUSED + CY_BITMAP;
    const BUTTON_REBOOT_PRESSED: i32 = BUTTON_REBOOT + CY_BITMAP;
    const BUTTON_REBOOT_FOCUSED: i32 = BUTTON_REBOOT_PRESSED + CY_BITMAP;
    const BUTTON_SLEEP: i32 = BUTTON_REBOOT_FOCUSED + CY_BITMAP;
    const BUTTON_SLEEP_PRESSED: i32 = BUTTON_SLEEP + CY_BITMAP;
    const BUTTON_SLEEP_FOCUSED: i32 = BUTTON_SLEEP_PRESSED + CY_BITMAP;
    const BUTTON_SLEEP_DISABLED: i32 = BUTTON_SLEEP_FOCUSED + CY_BITMAP;
    const SRCCOPY: u32 = 0xCC0020;

    let hdcMem = CreateCompatibleDC((*pdis).hDC);
    let hbmOld = SelectObject(hdcMem, (*pContext).hImageStrip as *mut core::ffi::c_void);
    let rect = (*pdis).rcItem;

    let mut y = 0i32;

    match (*pdis).CtlID as i32 {
        IDC_BUTTON_SHUTDOWN => {
            match (*pdis).itemAction {
                ODA_DRAWENTIRE | ODA_FOCUS | ODA_SELECT => {
                    if (*pdis).itemState & ODS_SELECTED != 0 {
                        y = BUTTON_SHUTDOWN_PRESSED;
                    } else if (*pContext).bIsButtonHot[0] != 0 || (*pdis).itemState & ODS_FOCUS != 0 {
                        y = BUTTON_SHUTDOWN_FOCUSED;
                    } else {
                        y = BUTTON_SHUTDOWN;
                    }
                }
                _ => {}
            }
        }
        IDC_BUTTON_REBOOT => {
            match (*pdis).itemAction {
                ODA_DRAWENTIRE | ODA_FOCUS | ODA_SELECT => {
                    if (*pdis).itemState & ODS_SELECTED != 0 {
                        y = BUTTON_REBOOT_PRESSED;
                    } else if (*pContext).bIsButtonHot[1] != 0 || (*pdis).itemState & ODS_FOCUS != 0 {
                        y = BUTTON_REBOOT_FOCUSED;
                    } else {
                        y = BUTTON_REBOOT;
                    }
                }
                _ => {}
            }
        }
        IDC_BUTTON_HIBERNATE | IDC_BUTTON_SLEEP => {
            match (*pdis).itemAction {
                ODA_DRAWENTIRE | ODA_FOCUS | ODA_SELECT => {
                    if (*pdis).itemState & ODS_DISABLED != 0 {
                        y = BUTTON_SLEEP_DISABLED;
                    } else if (*pdis).itemState & ODS_SELECTED != 0 {
                        y = BUTTON_SLEEP_PRESSED;
                    } else if 
                        ((*pdis).CtlID == IDC_BUTTON_SLEEP.try_into().unwrap() && (*pContext).bIsButtonHot[2] != 0)
                        || ((*pdis).CtlID == IDC_BUTTON_HIBERNATE.try_into().unwrap() && (*pContext).bIsButtonHot[3] != 0)
                        || ((*pdis).itemState & ODS_FOCUS != 0)
                    {
                        y = BUTTON_SLEEP_FOCUSED;
                    } else {
                        y = BUTTON_SLEEP;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    let bRet = BitBlt(
        (*pdis).hDC,
        (rect.right - rect.left - CX_BITMAP) / 2,
        (rect.bottom - rect.top - CY_BITMAP) / 2,
        CX_BITMAP,
        CY_BITMAP,
        hdcMem,
        0,
        y,
        SRCCOPY,
    );

    SelectObject(hdcMem, hbmOld);
    DeleteDC(hdcMem);

    bRet
}

#[no_mangle]
pub unsafe extern "system" fn ShellIsFriendlyUIActive() -> BOOL {
    let b_active = IsFriendlyUIActive();

    if (IsDomainMember() != 0 || IsNetwareActive() != 0) && ForceFriendlyUI() == 0 {
        return 0;
    }

    b_active
}

unsafe fn OwnerDrawButtonSubclass(
    hButton: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pContext =
        GetWindowLongPtrW(GetParent(hButton), GWLP_USERDATA) as *mut SHUTDOWN_DLG_CONTEXT;
    let buttonID = GetDlgCtrlID(hButton);

    match uMsg {
        WM_MOUSEMOVE => {
            let mut pt = POINT {
                x: GET_X_LPARAM!(lParam) as i32,
                y: GET_Y_LPARAM!(lParam) as i32,
            };
            if GetCapture() != hButton {
                SetCapture(hButton);
                if buttonID == IDC_BUTTON_SHUTDOWN {
                    (*pContext).bIsButtonHot[SHUTDOWN_BUTTON_HOT as usize] = 1;
                } else if buttonID == IDC_BUTTON_REBOOT {
                    (*pContext).bIsButtonHot[REBOOT_BUTTON_HOT as usize] = 1;
                } else if buttonID == IDC_BUTTON_SLEEP {
                    (*pContext).bIsButtonHot[SLEEP_BUTTON_HOT as usize] = 1;
                } else if buttonID == IDC_BUTTON_HIBERNATE {
                    (*pContext).bIsButtonHot[HIBERNATE_BUTTON_HOT as usize] = 1;
                }
                SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_HAND));
            }

            ClientToScreen(hButton, &mut pt as *mut POINT);
            let hwndTarget = WindowFromPoint(pt);

            if hwndTarget != hButton {
                ReleaseCapture();
                if buttonID == IDC_BUTTON_SHUTDOWN {
                    (*pContext).bIsButtonHot[SHUTDOWN_BUTTON_HOT as usize] = 0;
                } else if buttonID == IDC_BUTTON_REBOOT {
                    (*pContext).bIsButtonHot[REBOOT_BUTTON_HOT as usize] = 0;
                } else if buttonID == IDC_BUTTON_SLEEP {
                    (*pContext).bIsButtonHot[SLEEP_BUTTON_HOT as usize] = 0;
                } else if buttonID == IDC_BUTTON_HIBERNATE {
                    (*pContext).bIsButtonHot[HIBERNATE_BUTTON_HOT as usize] = 0;
                }
            }
            InvalidateRect(hButton, core::ptr::null_mut(), 0i32);
        }
        WM_SETFOCUS => {
            SendMessageW(GetParent(hButton), DM_SETDEFID, (buttonID as WPARAM).try_into().unwrap(), (0 as LPARAM).try_into().unwrap());
        }
        WM_KILLFOCUS => {
            SendMessageW(GetParent(hButton), DM_SETDEFID, (IDCANCEL as WPARAM).try_into().unwrap(), (0 as LPARAM).try_into().unwrap());
        }
        _ => {}
    }

    CallWindowProcW((*pContext).OldButtonProc, hButton, uMsg, wParam, lParam)
}
#[no_mangle]
pub unsafe extern "C" fn CreateToolTipForButtons(
    controlID: i32,
    detailID: i32,
    hDlg: HWND,
    titleID: i32,
    hInst: HINSTANCE,
) {
    let hwndTool: HWND = GetDlgItem(hDlg, controlID);
    let mut tool: TTTOOLINFOW = core::mem::zeroed();
    let mut szBuffer: [u16; 256] = [0; 256];

    tool.cbSize = core::mem::size_of::<TTTOOLINFOW>() as u32;
    tool.hwnd = hDlg;
    tool.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
    tool.uId = hwndTool as UINT_PTR;

    let hwndTip: HWND = CreateWindowExW(
        0,
        TOOLTIPS_CLASSW.as_ptr(),
        core::ptr::null_mut(),
        WS_POPUP | TTS_ALWAYSTIP | TTS_BALLOON,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hDlg,
        core::ptr::null_mut(),
        hInst,
        core::ptr::null_mut(),
    );

    LoadStringW(hInst, (detailID as u32).try_into().unwrap(), szBuffer.as_mut_ptr(), 256);
    tool.lpszText = szBuffer.as_mut_ptr();
    SendMessageW(hwndTip, TTM_ADDTOOLW, 0, &tool as *const _ as isize);

    LoadStringW(hInst, (titleID as u32).try_into().unwrap(), szBuffer.as_mut_ptr(), 256);
    SendMessageW(hwndTip, TTM_SETTITLEW, TTI_NONE as usize, szBuffer.as_mut_ptr() as isize);

    SendMessageW(hwndTip, TTM_SETMAXTIPWIDTH, 0, 250);
}
#[no_mangle]
pub unsafe extern "C" fn EndFriendlyDialog(hDlg: HWND, pContext: *mut SHUTDOWN_DLG_CONTEXT) {
    if (*pContext).bTimer != 0i32 {
        KillTimer(hDlg, (*pContext).iTimer);
    }

    DeleteObject((*pContext).hBitmap as *mut core::ffi::c_void);
    DeleteObject((*pContext).hBrush as *mut core::ffi::c_void);
    DeleteObject((*pContext).hImageStrip as *mut core::ffi::c_void);
    DeleteObject((*pContext).hfFont as *mut core::ffi::c_void);

    for i in 0..NUMBER_OF_BUTTONS {
        SetWindowLongPtrW(GetDlgItem(hDlg, IDC_BUTTON_SHUTDOWN + i as i32),
                          GWLP_WNDPROC,
                          (*pContext).OldButtonProc.map_or(0isize, |f| f as usize as isize));
    }
}
#[no_mangle]
pub unsafe extern "C" fn ChangeRequiredButton(
    hDlg: HWND,
    pContext: *mut SHUTDOWN_DLG_CONTEXT,
) {
    let mut destID = IDC_BUTTON_SLEEP;
    let mut targetedID = IDC_BUTTON_HIBERNATE;
    let hwndDest: HWND;
    let hwndTarget: HWND;
    let mut rect: RECT = core::mem::zeroed();
    let mut szBuffer: [u16; 30] = [0; 30];

    if (*pContext).bIsSleepButtonReplaced != 0 {
        destID = IDC_BUTTON_HIBERNATE;
        targetedID = IDC_BUTTON_SLEEP;
    }

    hwndDest = GetDlgItem(hDlg, destID);
    hwndTarget = GetDlgItem(hDlg, targetedID);

    GetWindowRect(hwndDest, &mut rect);
    MapWindowPoints(
        core::ptr::null_mut(),
        hDlg,
        &mut rect as *mut RECT as *mut POINT,
        2,
    );

    SetWindowPos(
        hwndTarget,
        HWND_TOP,
        rect.left,
        rect.top,
        0,
        0,
        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
    );

    EnableWindow(hwndDest, 0);
    ShowWindow(hwndDest, SW_HIDE.try_into().unwrap());
    EnableWindow(hwndTarget, 1);
    ShowWindow(hwndTarget, SW_SHOW.try_into().unwrap());
    SetFocus(hwndTarget);

    if (*pContext).bIsSleepButtonReplaced == 0 {
        LoadStringW(
            (*(*pContext).pgContext).hDllInstance,
            IDS_SHUTDOWN_HIBERNATE,
            szBuffer.as_mut_ptr(),
            30,
        );
        SetDlgItemTextW(hDlg, IDC_SLEEP_STATIC, szBuffer.as_ptr());
    } else {
        LoadStringW(
            (*(*pContext).pgContext).hDllInstance,
            IDS_SHUTDOWN_SLEEP,
            szBuffer.as_mut_ptr(),
            30,
        );
        SetDlgItemTextW(hDlg, IDC_SLEEP_STATIC, szBuffer.as_ptr());
    }

    InvalidateRect(hDlg, core::ptr::null(), 0);
}
#[no_mangle]
pub unsafe extern "C" fn OnTimer(
    hDlg: HWND,
    pContext: *mut SHUTDOWN_DLG_CONTEXT,
) {
    const VK_SHIFT: i32 = 0x10;

    let key_state = GetKeyState(VK_SHIFT);
    let replace_button: i32 = if (key_state as i32 & 0x8000) != 0 { 1 } else { 0 };

    if replace_button != 0 && (*pContext).bIsSleepButtonReplaced == 0 {
        ChangeRequiredButton(hDlg, pContext);
        (*pContext).bIsSleepButtonReplaced = 1;
    } else if replace_button == 0 && (*pContext).bIsSleepButtonReplaced != 0 {
        ChangeRequiredButton(hDlg, pContext);
        (*pContext).bIsSleepButtonReplaced = 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn LoadShutdownSelState(hKeyCurrentUser: HKEY) -> DWORD {
    let mut dwValue: DWORD = 0;
    let mut dwTemp: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let subkey: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer").as_ptr() as PCWSTR;
    let value_name: PCWSTR = w!("Shutdown Setting").as_ptr() as PCWSTR;
    let mut lRet: DWORD;

    lRet = RegOpenKeyExW(hKeyCurrentUser, subkey, 0, KEY_QUERY_VALUE, &mut hKey);
    if lRet != ERROR_SUCCESS {
        return dwValue;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lRet = RegQueryValueExW(hKey, value_name, core::ptr::null_mut(), core::ptr::null_mut(), &mut dwTemp as *mut DWORD as LPBYTE, &mut dwSize);
    RegCloseKey(hKey);

    if lRet == ERROR_SUCCESS {
        match dwTemp {
            WLX_SHUTDOWN_STATE_LOGOFF => dwValue = WLX_SAS_ACTION_LOGOFF,
            WLX_SHUTDOWN_STATE_POWER_OFF => dwValue = WLX_SAS_ACTION_SHUTDOWN_POWER_OFF,
            WLX_SHUTDOWN_STATE_REBOOT => dwValue = WLX_SAS_ACTION_SHUTDOWN_REBOOT,
            // 0x08
            WLX_SHUTDOWN_STATE_SLEEP => dwValue = WLX_SAS_ACTION_SHUTDOWN_SLEEP,
            WLX_SHUTDOWN_STATE_SLEEP2 => dwValue = WLX_SAS_ACTION_SHUTDOWN_SLEEP2,
            WLX_SHUTDOWN_STATE_HIBERNATE => dwValue = WLX_SAS_ACTION_SHUTDOWN_HIBERNATE,
            _ => {}
        }
    }

    dwValue
}
#[no_mangle]
pub unsafe extern "C" fn SaveShutdownSelState(
    hKeyCurrentUser: HKEY,
    ShutdownCode: DWORD,
) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;
    const SUBKEY: [u16; 51] = [
        'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '\\' as u16,
        'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16,
        'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16,
        'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16,
        'E' as u16, 'x' as u16, 'p' as u16, 'l' as u16, 'o' as u16, 'r' as u16, 'e' as u16, 'r' as u16, 0u16,
    ];
    let lRet = RegCreateKeyExW(
        hKeyCurrentUser,
        SUBKEY.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    );
    if lRet != ERROR_SUCCESS {
        return;
    }

    match ShutdownCode {
        WLX_SAS_ACTION_LOGOFF => dwValue = WLX_SHUTDOWN_STATE_LOGOFF,
        WLX_SAS_ACTION_SHUTDOWN_POWER_OFF => dwValue = WLX_SHUTDOWN_STATE_POWER_OFF,
        WLX_SAS_ACTION_SHUTDOWN_REBOOT => dwValue = WLX_SHUTDOWN_STATE_REBOOT,
        WLX_SAS_ACTION_SHUTDOWN_SLEEP => dwValue = WLX_SHUTDOWN_STATE_SLEEP,
        WLX_SAS_ACTION_SHUTDOWN_HIBERNATE => dwValue = WLX_SHUTDOWN_STATE_HIBERNATE,
        _ => {}
    }

    const SHUTDOWN_SETTING: [u16; 17] = [
        'S' as u16, 'h' as u16, 'u' as u16, 't' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'n' as u16,
        ' ' as u16, 'S' as u16, 'e' as u16, 't' as u16, 't' as u16, 'i' as u16, 'n' as u16, 'g' as u16,
        0u16,
    ];

    RegSetValueExW(
        hKey,
        SHUTDOWN_SETTING.as_ptr(),
        0,
        REG_DWORD,
        &dwValue as *const _ as *const u8,
        core::mem::size_of::<DWORD>() as u32,
    );
    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn GetAllowedShutdownOptions(
    hKeyCurrentUser: HKEY,
    hUserToken: HANDLE,
) -> DWORD {
    let mut options: DWORD = 0;
    let mut dw_policy_value: DWORD = 0;
    let mut status: NTSTATUS;
    let mut power_caps: SYSTEM_POWER_CAPABILITIES = core::mem::zeroed();

    let policy_key = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer");
    let value_name = w!("NoLogoff");
    GetPolicyDWORDValue(hKeyCurrentUser, policy_key.as_ptr(), value_name.as_ptr(), &mut dw_policy_value);
    if dw_policy_value == 0 {
        options |= WLX_SHUTDOWN_STATE_LOGOFF;
    }

    if TestTokenPrivilege(hUserToken, SE_SHUTDOWN_PRIVILEGE) == 0 {
        return options;
    }

    options |= WLX_SHUTDOWN_STATE_POWER_OFF | WLX_SHUTDOWN_STATE_REBOOT;

    status = NtPowerInformation(
        SystemPowerCapabilities,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(power_caps) as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_POWER_CAPABILITIES>() as u32,
    );

    if !NT_SUCCESS(status) {
        ERR!(
            b"NtPowerInformation(SystemPowerCapabilities) failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            status as core::ffi::c_ulong,
        );
    } else {
        if IS_PWR_SUSPEND_ALLOWED!(&power_caps) != 0 {
            options |= WLX_SHUTDOWN_STATE_SLEEP;
        }
        if IS_PWR_HIBERNATE_ALLOWED!(&power_caps) != 0 {
            options |= WLX_SHUTDOWN_STATE_HIBERNATE;
        }
    }

    options
}

unsafe fn UpdateShutdownDesc(
    h_dlg: HWND,
    p_context: PSHUTDOWN_DLG_CONTEXT,
) {
    let hwnd_list = GetDlgItem(h_dlg, IDC_SHUTDOWN_ACTION);
    let mut shutdown_code = SendMessageW(hwnd_list, CB_GETCURSEL, 0, 0);
    if shutdown_code != CB_ERR.into() {
        shutdown_code = SendMessageW(hwnd_list, CB_GETITEMDATA, shutdown_code as usize, 0);
    }
    if shutdown_code == CB_ERR.into() {
        return;
    }

    let desc_id = match shutdown_code as u32 {
        WLX_SAS_ACTION_LOGOFF => IDS_SHUTDOWN_LOGOFF_DESC,
        WLX_SAS_ACTION_SHUTDOWN_POWER_OFF => IDS_SHUTDOWN_SHUTDOWN_DESC,
        WLX_SAS_ACTION_SHUTDOWN_REBOOT => IDS_SHUTDOWN_RESTART_DESC,
        WLX_SAS_ACTION_SHUTDOWN_SLEEP => IDS_SHUTDOWN_SLEEP_DESC,
        WLX_SAS_ACTION_SHUTDOWN_HIBERNATE => IDS_SHUTDOWN_HIBERNATE_DESC,
        _ => 0,
    };

    let mut sz_buffer: [u16; 256] = [0; 256];
    let h_dll_instance = (*(*p_context).pgContext).hDllInstance;
    LoadStringW(h_dll_instance, desc_id, sz_buffer.as_mut_ptr(), sz_buffer.len() as i32);
    SetDlgItemTextW(h_dlg, IDC_SHUTDOWN_DESCRIPTION, sz_buffer.as_ptr());

    if (*p_context).bReasonUI != 0 {
        let b_not_logoff = (shutdown_code != (WLX_SAS_ACTION_LOGOFF as isize).try_into().unwrap()) as i32;
        EnableWindow(GetDlgItem(h_dlg, IDC_REASON_PLANNED), b_not_logoff);
        EnableWindow(GetDlgItem(h_dlg, IDC_REASON_LIST), b_not_logoff);
        EnableWindow(GetDlgItem(h_dlg, IDC_REASON_COMMENT), b_not_logoff);
    }
}

unsafe fn ShutdownOnFriendlyInit(
    hDlg: HWND,
    pContext: PSHUTDOWN_DLG_CONTEXT,
) {
    let pgContext = (*pContext).pgContext;
    let bCanSuspend: u8 = if ((*pContext).ShutdownOptions & (WLX_SHUTDOWN_STATE_SLEEP | WLX_SHUTDOWN_STATE_SLEEP2)) != 0 { 1 } else { 0 };
    let bCanHibernate: u8 = if ((*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_HIBERNATE) != 0 { 1 } else { 0 };

    let hdc = GetDC(hDlg);
    let lfHeight = -MulDiv(
        FONT_POINT_SIZE,
        GetDeviceCaps(hdc, LOGPIXELSY),
        72,
    );
    ReleaseDC(hDlg, hdc);
    const MS_SHELL_DLG: [u16; 13] = [
        b'M' as u16, b'S' as u16, b' ' as u16, b'S' as u16, b'h' as u16,
        b'e' as u16, b'l' as u16, b'l' as u16, b' ' as u16, b'D' as u16,
        b'l' as u16, b'g' as u16, 0u16,
    ];
    (*pContext).hfFont = CreateFontW(
        lfHeight,
        0,
        0,
        0,
        FW_MEDIUM,
        0u32,
        0u32,
        0u32,
        0u32,
        0u32,
        0u32,
        0u32,
        0u32,
        MS_SHELL_DLG.as_ptr(),
    );
    SendDlgItemMessageW(
        hDlg,
        IDC_TURN_OFF_STATIC,
        WM_SETFONT,
        ((*pContext).hfFont as WPARAM).try_into().unwrap(),
        1isize,
    );

    (*pContext).hBrush = CreateSolidBrush(DARK_GREY_COLOR);

    (*pContext).hImageStrip = LoadBitmapW(
        (*pgContext).hDllInstance,
        IDB_IMAGE_STRIP as LPCWSTR,
    );

    (*pContext).bIsSleepButtonReplaced = 0;
    (*pContext).bTimer = 0;

    EnableWindow(
        GetDlgItem(hDlg, IDC_BUTTON_HIBERNATE),
        bCanHibernate as i32,
    );
    EnableWindow(
        GetDlgItem(hDlg, IDC_BUTTON_SLEEP),
        bCanSuspend as i32,
    );

    let old_proc: WNDPROC = core::mem::transmute(GetWindowLongPtrW(
        GetDlgItem(hDlg, IDC_BUTTON_HIBERNATE),
        GWLP_WNDPROC as i32,
    ) as isize);
    (*pContext).OldButtonProc = old_proc;

    for i in 0..NUMBER_OF_BUTTONS {
        (*pContext).bIsButtonHot[i as usize] = 0;
        SetWindowLongPtrW(
            GetDlgItem(hDlg, IDC_BUTTON_SHUTDOWN + i as i32),
            GWLP_WNDPROC as i32,
            core::mem::transmute::<unsafe fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR, isize>(
                OwnerDrawButtonSubclass,
            ),
        );
        CreateToolTipForButtons(
            IDC_BUTTON_SHUTDOWN + i as i32,
            IDS_SHUTDOWN_SHUTDOWN_DESC + i as i32,
            hDlg,
            IDS_SHUTDOWN_SHUTDOWN + i as i32,
            (*pgContext).hDllInstance,
        );
    }

    if (*pContext).ShutdownDialogId == IDD_SHUTDOWN_FANCY.try_into().unwrap() && bCanSuspend != 0 {
        (*pContext).iTimer = SetTimer(hDlg, 0, 50, core::ptr::null_mut());
        (*pContext).bTimer = 1;
    }
}

unsafe fn ShutdownOnInit(
    hDlg: HWND,
    pContext: PSHUTDOWN_DLG_CONTEXT,
) {
    let pgContext = (*pContext).pgContext;
    let hwndList: HWND;
    let mut idx: LRESULT;
    let count: LRESULT;
    let mut i: LRESULT;
    let mut szBuffer: [u16; 256] = [0; 256];
    let mut szBuffer2: [u16; 256] = [0; 256];

    if (*pContext).bFriendlyUI != 0i32 {
        ShutdownOnFriendlyInit(hDlg, pContext);
        return;
    }

    hwndList = GetDlgItem(hDlg, IDC_SHUTDOWN_ACTION);

    /* Clear the content before it's used */
    SendMessageW(hwndList, CB_RESETCONTENT, 0, 0);

    /* Log off */
    if ((*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_LOGOFF) != 0 {
        LoadStringW(
            (*pgContext).hDllInstance,
            IDS_SHUTDOWN_LOGOFF,
            szBuffer.as_mut_ptr(),
            szBuffer.len() as i32,
        );
        // `_countof(szBuffer2)` in the C. `.as_ptr()` on UserName is C's array-to-pointer decay,
        // which Rust does not do for you — passing the `[WCHAR; 256]` by value put 512 bytes of
        // name text in the varargs area and `%ws` read the first of them as an address.
        StringCchPrintfW!(
            szBuffer2.as_mut_ptr(),
            szBuffer2.len(),
            szBuffer.as_ptr(),
            (*pgContext).UserName.as_ptr(),
        );
        idx = SendMessageW(
            hwndList,
            CB_ADDSTRING,
            0,
            (szBuffer2.as_ptr() as LPARAM).try_into().unwrap(),
        );
        if idx != CB_ERR.into() {
            SendMessageW(hwndList, CB_SETITEMDATA, idx.try_into().unwrap(), WLX_SAS_ACTION_LOGOFF.try_into().unwrap());
        }
    }

    /* Shut down */
    if ((*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_POWER_OFF) != 0 {
        LoadStringW(
            (*pgContext).hDllInstance,
            IDS_SHUTDOWN_SHUTDOWN,
            szBuffer.as_mut_ptr(),
            szBuffer.len() as i32,
        );
        idx = SendMessageW(
            hwndList,
            CB_ADDSTRING,
            0,
            (szBuffer.as_ptr() as LPARAM).try_into().unwrap(),
        );
        if idx != CB_ERR.into() {
            SendMessageW(hwndList, CB_SETITEMDATA, idx.try_into().unwrap(), WLX_SAS_ACTION_SHUTDOWN_POWER_OFF.try_into().unwrap());
        }
    }

    /* Restart */
    if ((*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_REBOOT) != 0 {
        LoadStringW(
            (*pgContext).hDllInstance,
            IDS_SHUTDOWN_RESTART,
            szBuffer.as_mut_ptr(),
            szBuffer.len() as i32,
        );
        idx = SendMessageW(
            hwndList,
            CB_ADDSTRING,
            0,
            (szBuffer.as_ptr() as LPARAM).try_into().unwrap(),
        );
        if idx != CB_ERR.into() {
            SendMessageW(hwndList, CB_SETITEMDATA, idx.try_into().unwrap(), WLX_SAS_ACTION_SHUTDOWN_REBOOT.try_into().unwrap());
        }
    }

    /* Sleep */
    if ((*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_SLEEP) != 0 {
        LoadStringW(
            (*pgContext).hDllInstance,
            IDS_SHUTDOWN_SLEEP,
            szBuffer.as_mut_ptr(),
            szBuffer.len() as i32,
        );
        idx = SendMessageW(
            hwndList,
            CB_ADDSTRING,
            0,
            (szBuffer.as_ptr() as LPARAM).try_into().unwrap(),
        );
        if idx != CB_ERR.into() {
            SendMessageW(hwndList, CB_SETITEMDATA, idx.try_into().unwrap(), WLX_SAS_ACTION_SHUTDOWN_SLEEP.try_into().unwrap());
        }
    }

    /* Hibernate */
    if ((*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_HIBERNATE) != 0 {
        LoadStringW(
            (*pgContext).hDllInstance,
            IDS_SHUTDOWN_HIBERNATE,
            szBuffer.as_mut_ptr(),
            szBuffer.len() as i32,
        );
        idx = SendMessageW(
            hwndList,
            CB_ADDSTRING,
            0,
            (szBuffer.as_ptr() as LPARAM).try_into().unwrap(),
        );
        if idx != CB_ERR.into() {
            SendMessageW(hwndList, CB_SETITEMDATA, idx.try_into().unwrap(), WLX_SAS_ACTION_SHUTDOWN_HIBERNATE.try_into().unwrap());
        }
    }

    /* Set the default selection */
    count = SendMessageW(hwndList, CB_GETCOUNT, 0, 0);
    i = 0;
    while i < count {
        if SendMessageW(hwndList, CB_GETITEMDATA, i.try_into().unwrap(), 0) == (*pgContext).nShutdownAction as LPARAM {
            SendMessageW(hwndList, CB_SETCURSEL, i.try_into().unwrap(), 0);
            break;
        }
        i += 1;
    }

    /* If no item was selected (nShutdownAction with value 0 or invalid,
     * or an action that is currently unavailable), set the selection to
     * the first list item. */
    if count != 0 && (SendMessageW(hwndList, CB_GETCURSEL, 0, 0) == CB_ERR.into()) {
        SendMessageW(hwndList, CB_SETCURSEL, 0, 0);
    }

    /* Update the choice description based on the current selection */
    UpdateShutdownDesc(hDlg, pContext);
}

unsafe fn ShutdownOnOk(hDlg: HWND, pgContext: *mut _GINA_CONTEXT) {
    let hwndList = GetDlgItem(hDlg, IDC_SHUTDOWN_ACTION as i32);
    let mut lresult = SendMessageW(hwndList, CB_GETCURSEL, 0, 0);
    if lresult != (CB_ERR as isize).try_into().unwrap() {
        let mut shutdown_code = lresult as u32;
        lresult = SendMessageW(hwndList, CB_GETITEMDATA, shutdown_code as usize, 0);
        if lresult != (CB_ERR as isize).try_into().unwrap() {
            shutdown_code = lresult as u32;
            (*pgContext).nShutdownAction = shutdown_code;
        }
    }
}
unsafe extern "system" fn ShutdownDialogProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pContext = GetWindowLongPtrW(hDlg, GWLP_USERDATA) as *mut SHUTDOWN_DLG_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pContext = lParam as *mut SHUTDOWN_DLG_CONTEXT;
            SetWindowLongPtrW(hDlg, GWLP_USERDATA, pContext as isize);
            ShutdownOnInit(hDlg, pContext);
            return 1 as INT_PTR;
        }

        WM_DESTROY => {
            if (*pContext).bFriendlyUI != 0 {
                EndFriendlyDialog(hDlg, pContext);
            }
            return 1 as INT_PTR;
        }

        WM_ACTIVATE => {
            if u32::from((wParam as u32 & 0xFFFF) as u16) == WA_INACTIVE {
                if (*pContext).bCloseDlg == 0 {
                    (*pContext).bCloseDlg = 1i32;
                    EndDialog(hDlg, IDCANCEL as isize);
                }
            }
            return 0 as INT_PTR;
        }

        WM_CLOSE => {
            (*pContext).bCloseDlg = 1i32;
            EndDialog(hDlg, IDCANCEL as isize);
            return 1 as INT_PTR;
        }

        WM_COMMAND => {
            let loword = (wParam as u32 & 0xFFFF) as u32;
            match loword as i32 {
                IDC_BUTTON_SHUTDOWN
                | IDC_BUTTON_REBOOT
                | IDC_BUTTON_SLEEP
                | IDC_BUTTON_HIBERNATE => {
                    let button = loword;
                    let pgContext = (*pContext).pgContext;
                    (*pgContext).nShutdownAction = 0;

                    if button == IDC_BUTTON_SHUTDOWN.try_into().unwrap() {
                        if (*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_POWER_OFF != 0 {
                            (*pgContext).nShutdownAction = WLX_SAS_ACTION_SHUTDOWN_POWER_OFF;
                        }
                    } else if button == IDC_BUTTON_REBOOT.try_into().unwrap() {
                        if (*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_REBOOT != 0 {
                            (*pgContext).nShutdownAction = WLX_SAS_ACTION_SHUTDOWN_REBOOT;
                        }
                    } else if button == IDC_BUTTON_SLEEP.try_into().unwrap() {
                        if (*pContext).ShutdownOptions
                            & (WLX_SHUTDOWN_STATE_SLEEP | WLX_SHUTDOWN_STATE_SLEEP2)
                            != 0
                        {
                            if (*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_SLEEP2 != 0 {
                                (*pgContext).nShutdownAction =
                                    WLX_SAS_ACTION_SHUTDOWN_SLEEP2;
                            } else {
                                (*pgContext).nShutdownAction =
                                    WLX_SAS_ACTION_SHUTDOWN_SLEEP;
                            }
                        }
                    } else if button == IDC_BUTTON_HIBERNATE.try_into().unwrap() {
                        if (*pContext).ShutdownOptions & WLX_SHUTDOWN_STATE_HIBERNATE != 0 {
                            (*pgContext).nShutdownAction =
                                WLX_SAS_ACTION_SHUTDOWN_HIBERNATE;
                        }
                    }

                    if (*pgContext).nShutdownAction == 0 {
                        // break out of inner match; execution continues after inner match,
                        // which returns 1 as INT_PTR below.
                    } else {
                        (*pContext).bCloseDlg = 1i32;
                        EndDialog(hDlg, IDOK as isize);
                    }
                }

                x if x == IDOK as i32 => {
                    ShutdownOnOk(hDlg, (*pContext).pgContext);
                    // Fall through
                    (*pContext).bCloseDlg = 1i32;
                    EndDialog(hDlg, (wParam as u32 & 0xFFFF) as isize);
                }

                x if x == IDCANCEL as i32 || x == IDHELP as i32 => {
                    (*pContext).bCloseDlg = 1i32;
                    EndDialog(hDlg, (wParam as u32 & 0xFFFF) as isize);
                }

                IDC_SHUTDOWN_ACTION => {
                    UpdateShutdownDesc(hDlg, pContext);
                }

                _ => {}
            }
            return 1 as INT_PTR;
        }

        WM_CTLCOLORSTATIC => {
            let hdcStatic = wParam as HDC;
            let staticId = GetWindowLongPtrW(lParam as HWND, GWL_ID) as u32;

            match staticId as i32 {
                IDC_TURN_OFF_STATIC => {
                    SetTextColor(hdcStatic, DARK_GREY_COLOR);
                    SetBkMode(hdcStatic, TRANSPARENT);
                    return GetStockObject(HOLLOW_BRUSH) as INT_PTR;
                }
                IDC_HIBERNATE_STATIC
                | IDC_SHUTDOWN_STATIC
                | IDC_SLEEP_STATIC
                | IDC_RESTART_STATIC => {
                    SetTextColor(hdcStatic, LIGHT_GREY_COLOR);
                    SetBkMode(hdcStatic, TRANSPARENT);
                    return (*pContext).hBrush as INT_PTR;
                }
                _ => {}
            }
            return 0 as INT_PTR;
        }

        WM_DRAWITEM => {
            let pdis = lParam as *mut DRAWITEMSTRUCT;
            match (*pdis).CtlID as i32 {
                IDC_BUTTON_SHUTDOWN
                | IDC_BUTTON_REBOOT
                | IDC_BUTTON_SLEEP
                | IDC_BUTTON_HIBERNATE => {
                    return DrawIconOnOwnerDrawnButtons(pdis, pContext) as INT_PTR;
                }
                _ => {}
            }
            return 1 as INT_PTR;
        }

        WM_TIMER => {
            OnTimer(hDlg, pContext);
            return 1 as INT_PTR;
        }

        _ => return 0 as INT_PTR,
    }
}
#[no_mangle]
pub unsafe extern "C" fn ShutdownDialog(
    hwndDlg: HWND,
    ShutdownOptions: DWORD,
    pgContext: PGINA_CONTEXT,
) -> INT_PTR {
    let mut ret: INT_PTR;
    let mut context: SHUTDOWN_DLG_CONTEXT = core::mem::zeroed();

    if ShutdownOptions == 0 {
        return IDCANCEL as INT_PTR;
    }

    context.pgContext = pgContext;
    context.ShutdownOptions = ShutdownOptions;
    context.ShutdownDialogId = IDD_SHUTDOWN as u32;
    context.bCloseDlg = 0;
    context.bReasonUI = GetShutdownReasonUI();
    context.bFriendlyUI = ShellIsFriendlyUIActive();

    if !(*pgContext).hWlx.is_null() && !(*pgContext).pWlxFuncs.is_null() && context.bFriendlyUI == 0 {
        let resource_id = if context.bReasonUI != 0 { IDD_SHUTDOWN_REASON } else { IDD_SHUTDOWN };
        ret = (*(*pgContext).pWlxFuncs).WlxDialogBoxParam.unwrap()(
            (*pgContext).hWlx,
            (*pgContext).hDllInstance as *mut core::ffi::c_void,
            MAKEINTRESOURCEW!(resource_id) as *mut u16,
            hwndDlg as winlogon_sys::nt::HWND,
            core::mem::transmute::<*const (), winlogon_sys::nt::DLGPROC>(
                ShutdownDialogProc as *const (),
            ),
            &mut context as *mut _ as winlogon_sys::nt::LPARAM,
        ) as INT_PTR;
    } else {
        if context.bFriendlyUI != 0 {
            if IsShowHibernateButtonActive() != 0 {
                context.ShutdownDialogId = IDD_SHUTDOWN_FANCY_LONG as u32;
            } else {
                context.ShutdownDialogId = IDD_SHUTDOWN_FANCY as u32;
            }
        }
        let resource_id = if context.bReasonUI != 0 { IDD_SHUTDOWN_REASON } else { context.ShutdownDialogId.try_into().unwrap() };
        ret = DialogBoxParamW(
            (*pgContext).hDllInstance,
            MAKEINTRESOURCEW!(resource_id),
            hwndDlg,
            Some(ShutdownDialogProc),
            &mut context as *mut _ as LPARAM,
        );
    }

    // TODO: Handle shutdown reasons.

    ret
}

#[no_mangle]
pub unsafe extern "system" fn ShellShutdownDialog(
    hWndParent: HWND,
    pUserName: PCWSTR,
    dwExcludeOptions: DWORD,
) -> DWORD {
    let mut dlgValue: INT_PTR;
    let mut ShutdownOptions: DWORD;
    let mut gContext: GINA_CONTEXT = core::mem::zeroed();

    ShutdownOptions = GetAllowedShutdownOptions(HKEY_CURRENT_USER, core::ptr::null_mut());
    ShutdownOptions &= !dwExcludeOptions;

    gContext.hDllInstance = hDllInstance;
    gContext.nShutdownAction = LoadShutdownSelState(HKEY_CURRENT_USER);
    if gContext.nShutdownAction == 0 {
        gContext.nShutdownAction = WLX_SAS_ACTION_SHUTDOWN_POWER_OFF;
    }

    if !pUserName.is_null() {
        let size = core::mem::size_of_val(&gContext.UserName);
        StringCbCopyW(gContext.UserName.as_mut_ptr(), size, pUserName);
    } else {
        let mut hKey: HKEY = core::ptr::null_mut();
        let key_path: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer").as_ptr() as PCWSTR;
        let lRet: LONG = RegOpenKeyExW(HKEY_CURRENT_USER, key_path, 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKey)).try_into().unwrap();
        if lRet == ERROR_SUCCESS.try_into().unwrap() {
            let mut dwType: DWORD = 0;
            let mut cbData: DWORD = core::mem::size_of_val(&gContext.UserName) as DWORD;
            let logon_name: PCWSTR = w!("Logon User Name").as_ptr() as PCWSTR;
            let lRet2 = RegQueryValueExW(
                hKey,
                logon_name,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwType),
                gContext.UserName.as_mut_ptr() as *mut u8,
                core::ptr::addr_of_mut!(cbData),
            );
            RegCloseKey(hKey);

            if lRet2 != ERROR_SUCCESS {
                cbData = 0;
            }
            let index = (cbData as usize) / core::mem::size_of::<WCHAR>();
            gContext.UserName[index] = 0;
        } else {
            let element_count = core::mem::size_of_val(&gContext.UserName) / core::mem::size_of::<WCHAR>();
            let mut BufferSize: DWORD = element_count as DWORD;
            GetUserNameW(gContext.UserName.as_mut_ptr(), core::ptr::addr_of_mut!(BufferSize));
        }
    }

    dlgValue = ShutdownDialog(hWndParent, ShutdownOptions, core::ptr::addr_of_mut!(gContext));

    if dlgValue == IDOK as INT_PTR {
        SaveShutdownSelState(HKEY_CURRENT_USER, gContext.nShutdownAction);

        match gContext.nShutdownAction {
            WLX_SAS_ACTION_LOGOFF => return WLX_SHUTDOWN_STATE_LOGOFF,
            WLX_SAS_ACTION_SHUTDOWN_POWER_OFF => return WLX_SHUTDOWN_STATE_POWER_OFF,
            WLX_SAS_ACTION_SHUTDOWN_REBOOT => return WLX_SHUTDOWN_STATE_REBOOT,
            WLX_SAS_ACTION_SHUTDOWN_SLEEP => return WLX_SHUTDOWN_STATE_SLEEP,
            WLX_SAS_ACTION_SHUTDOWN_SLEEP2 => return WLX_SHUTDOWN_STATE_SLEEP2,
            WLX_SAS_ACTION_SHUTDOWN_HIBERNATE => return WLX_SHUTDOWN_STATE_HIBERNATE,
            _ => (),
        }
    } else if dlgValue == IDHELP as INT_PTR {
        FIXME!(b"Help is not implemented yet.\n".as_ptr() as PCHAR);
        let msg: PCWSTR = w!("Help is not implemented yet.").as_ptr() as PCWSTR;
        let caption: PCWSTR = w!("Message").as_ptr() as PCWSTR;
        MessageBoxW(hWndParent, msg, caption, MB_OK | MB_ICONEXCLAMATION);
    } else if dlgValue == -1 {
        ERR!(b"Failed to create dialog\n".as_ptr() as PCHAR);
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn ShellTurnOffDialog(
    hWnd: HWND,
) -> DWORD {
    ShellShutdownDialog(hWnd, core::ptr::null_mut(), 0)
}
