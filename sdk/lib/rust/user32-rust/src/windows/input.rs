use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
// ntdll Rtl* exports + RtlInitEmptyUnicodeString inline live in crate::internal.
use crate::internal::{RtlAllocateHeap, RtlFreeHeap, RtlGetProcessHeap, RtlInitEmptyUnicodeString, RtlInitUnicodeString, LOWORD, HIWORD, MAKELONG};
use crate::ntwrapper::{NtUserxEnableWindow, NtUserxGetKeyboardLayout, NtUserxGetKeyboardType, NtUserxSwapMouseButton};
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


// `Input` was a bindgen mis-resolution; the real type is `INPUT` (= tagINPUT) from
// win32k_sys::win32k::*, already in scope. No import needed (keybd_event/mouse_event
// shadow the name with a local `Input` variable of type INPUT anyway).

// ── undocuser.h SETIMEHOTKEY_ACTION enum (not in win32k-sys bindgen surface) ──
const SETIMEHOTKEY_DELETE: DWORD = 1;
const SETIMEHOTKEY_ADD: DWORD = 2;
const SETIMEHOTKEY_INITIALIZE: DWORD = 3;

// ── imm32_undoc.h IS_IME_HKL macro ──
const IME_MASK: ULONG_PTR = 0xE0000000;
#[inline]
unsafe fn IS_IME_HKL(hKL: HKL) -> BOOL {
    (((hKL as ULONG_PTR) & 0xF0000000) == IME_MASK) as BOOL
}

// ── ntbasedef.h MAKELANGID / LANGIDFROMLCID macros ──
#[inline]
const fn MAKELANGID(p: u32, s: u32) -> LANGID {
    (((s as u16) << 10) | (p as u16)) as LANGID
}
#[inline]
fn LANGIDFROMLCID(lcid: LCID) -> LANGID {
    lcid as u16 as LANGID
}

// ── basetsd.h UlongToHandle ──
#[inline]
fn UlongToHandle(ul: ULONG) -> HANDLE {
    ul as usize as HANDLE
}

// ── winreg.h predefined HKEYs + REGSAM (winnt.h) ──
const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
type REGSAM = u32;

// ── winnls.h codepages / fileapi.h sentinel ──
const CP_ACP: UINT = 0;
const CP_OEMCP: UINT = 1;
const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;

const ENGLISH_US: WORD = MAKELANGID(LANG_ENGLISH as u32, SUBLANG_ENGLISH_US as u32) as WORD;

// CRT string helpers (msvcrt) — not user32 exports.
extern "C" {
    fn wcsspn(s: *const u16, accept: *const u16) -> usize;
    fn wcscspn(s: *const u16, reject: *const u16) -> usize;
    fn wcslen(s: *const u16) -> usize;
    fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> ULONG;
    fn _ultow(value: u32, str_: LPWSTR, radix: i32) -> LPWSTR;
}

// strsafe.h StringCch*/Cb* are header inlines with no exported symbol. Use wsprintfW
// (user32 __cdecl variadic export, like messagebox.rs/dialog.rs) for the printf cases
// and a local bounded wide-concat for StringCbCatW.
extern "C" {
    fn wsprintfW(buf: LPWSTR, fmt: LPCWSTR, ...) -> INT;
}
#[inline]
unsafe fn StringCbCatW(pszDest: LPWSTR, cbDest: usize, pszSrc: LPCWSTR) -> i32 {
    let cch_dest = cbDest / core::mem::size_of::<u16>();
    let mut d = 0usize;
    while d + 1 < cch_dest && *pszDest.add(d) != 0 {
        d += 1;
    }
    let mut s = 0usize;
    while d + 1 < cch_dest && *pszSrc.add(s) != 0 {
        *pszDest.add(d) = *pszSrc.add(s);
        d += 1;
        s += 1;
    }
    if cch_dest > 0 {
        *pszDest.add(d) = 0;
    }
    0
}

// advapi32 (registry) — extern "system" (stdcall on i386).
extern "system" {
    fn RegCloseKey(hKey: HKEY) -> LONG;
    fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: u32, lpSecurityAttributes: *mut core::ffi::c_void, phkResult: *mut HKEY, lpdwDisposition: *mut DWORD, ) -> LONG;
    fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LONG;
    fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const u8, cbData: DWORD, ) -> LONG;
    fn RegOpenKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: u32, phkResult: *mut HKEY, ) -> LONG;
    fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: *mut DWORD, lpReserved: *mut DWORD, lpClass: LPWSTR, lpcchClass: *mut DWORD, lpftLastWriteTime: *mut FILETIME, ) -> LONG;
    fn RegOpenKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: *mut HKEY) -> i32;
    fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: *mut u32, lpType: *mut u32, lpData: LPBYTE, lpcbData: *mut u32, ) -> i32;
}

// kernel32 — extern "system" (stdcall on i386).
extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree( hHeap: *mut core::ffi::c_void, dwFlags: DWORD, lpMem: *mut core::ffi::c_void, ) -> core::ffi::c_int;
    fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: core::ffi::c_int, lpMultiByteStr: LPSTR, cbMultiByte: core::ffi::c_int, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut core::ffi::c_int, ) -> core::ffi::c_int;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: PCSTR, cbMultiByte: core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: core::ffi::c_int, ) -> core::ffi::c_int;
    fn SystemParametersInfoW(uiAction: u32, uiParam: u32, pvParam: *mut core::ffi::c_void, fWinIni: u32) -> i32;
    fn SetLastError(dwErrCode: DWORD);
    fn GetOEMCP() -> UINT;
    fn GetUserDefaultLCID() -> LCID;
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD;
    fn IsDBCSLeadByte(TestChar: BYTE) -> BOOL;
}

// NOTE: VkKeyScanW / MapVirtualKeyW / LoadKeyboardLayoutW / GetKeyboardLayout etc.
// are all #[no_mangle] exports defined in THIS file — called directly, no extern
// decl needed (an extern block would collide with the definition).

// ntdll export not in crate::internal / win32k-sys surface.
extern "system" {
    fn RtlUnicodeStringToInteger(String: PCUNICODE_STRING, Base: ULONG, Value: PULONG) -> NTSTATUS;
}

/// `_countof(arr)` — element count of a fixed-size array reference.
#[inline]
fn _countof<T, const N: usize>(_a: &[T; N]) -> usize {
    N
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct tagIMEHOTKEYENTRY {
    pub dwHotKeyId: DWORD,
    pub uVirtualKey: UINT,
    pub uModifiers: UINT,
    pub hKL: HKL,
}
pub type IMEHOTKEYENTRY = tagIMEHOTKEYENTRY;
pub type PIMEHOTKEYENTRY = *mut tagIMEHOTKEYENTRY;

pub static mut DefaultHotKeyTableJ: [IMEHOTKEYENTRY; 1] = [
    IMEHOTKEYENTRY {
        dwHotKeyId: IME_JHOTKEY_CLOSE_OPEN as DWORD,
        uVirtualKey: VK_KANJI as UINT,
        uModifiers: MOD_IGNORE_ALL_MODIFIER as UINT,
        hKL: core::ptr::null_mut(),
    },
];

pub static mut DefaultHotKeyTableT: [IMEHOTKEYENTRY; 2] = [
    IMEHOTKEYENTRY {
        dwHotKeyId: IME_THOTKEY_IME_NONIME_TOGGLE as DWORD,
        uVirtualKey: VK_SPACE as UINT,
        uModifiers: (MOD_LEFT | MOD_RIGHT | MOD_CONTROL) as UINT,
        hKL: core::ptr::null_mut(),
    },
    IMEHOTKEYENTRY {
        dwHotKeyId: IME_THOTKEY_SHAPE_TOGGLE as DWORD,
        uVirtualKey: VK_SPACE as UINT,
        uModifiers: (MOD_LEFT | MOD_RIGHT | MOD_SHIFT) as UINT,
        hKL: core::ptr::null_mut(),
    },
];

pub static mut DefaultHotKeyTableC: [IMEHOTKEYENTRY; 2] = [
    IMEHOTKEYENTRY {
        dwHotKeyId: IME_CHOTKEY_IME_NONIME_TOGGLE as DWORD,
        uVirtualKey: VK_SPACE as UINT,
        uModifiers: (MOD_LEFT | MOD_RIGHT | MOD_CONTROL) as UINT,
        hKL: core::ptr::null_mut(),
    },
    IMEHOTKEYENTRY {
        dwHotKeyId: IME_CHOTKEY_SHAPE_TOGGLE as DWORD,
        uVirtualKey: VK_SPACE as UINT,
        uModifiers: (MOD_LEFT | MOD_RIGHT | MOD_SHIFT) as UINT,
        hKL: core::ptr::null_mut(),
    },
];

pub const FE_JAPANESE: u32 = 1 << 0;
pub const FE_CHINESE_TRADITIONAL: u32 = 1 << 1;
pub const FE_CHINESE_SIMPLIFIED: u32 = 1 << 2;
pub const FE_KOREAN: u32 = 1 << 3;


#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetFeKeyboardFlags(LangID: LANGID, pbFlags: PBYTE) {
    IntSetFeKeyboardFlags_impl(LangID, pbFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetFeKeyboardFlags(LangID: LANGID, pbFlags: PBYTE) {
    IntSetFeKeyboardFlags_impl(LangID, pbFlags)
}

#[inline]
unsafe fn IntSetFeKeyboardFlags_impl(LangID: LANGID, pbFlags: PBYTE) {
    const FE_JAPANESE: u8 = 1 << 0;
    const FE_CHINESE_TRADITIONAL: u8 = 1 << 1;
    const FE_CHINESE_SIMPLIFIED: u8 = 1 << 2;
    const FE_KOREAN: u8 = 1 << 3;

    let make = |lang: u32, sub: u32| -> LANGID { ((sub << 10) | lang) as LANGID };

    if LangID == make(LANG_JAPANESE as u32, SUBLANG_DEFAULT as u32) {
        *pbFlags |= FE_JAPANESE;
    } else if LangID == make(LANG_CHINESE as u32, SUBLANG_CHINESE_TRADITIONAL as u32)
        || LangID == make(LANG_CHINESE as u32, SUBLANG_CHINESE_HONGKONG as u32)
    {
        *pbFlags |= FE_CHINESE_TRADITIONAL;
    } else if LangID == make(LANG_CHINESE as u32, SUBLANG_CHINESE_SIMPLIFIED as u32)
        || LangID == make(LANG_CHINESE as u32, SUBLANG_CHINESE_SINGAPORE as u32)
    {
        *pbFlags |= FE_CHINESE_SIMPLIFIED;
    } else if LangID == make(LANG_KOREAN as u32, SUBLANG_KOREAN as u32) {
        *pbFlags |= FE_KOREAN;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CliReadRegistryValue(hKey: HANDLE, pszName: LPCWSTR) -> DWORD {
    CliReadRegistryValueImpl(hKey, pszName)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CliReadRegistryValue(hKey: HANDLE, pszName: LPCWSTR) -> DWORD {
    CliReadRegistryValueImpl(hKey, pszName)
}

unsafe fn CliReadRegistryValueImpl(hKey: HANDLE, pszName: LPCWSTR) -> DWORD {
    let mut dwValue: DWORD = 0;
    let mut cbValue: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    let error = RegQueryValueExW(
        hKey as HKEY,
        pszName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwValue) as LPBYTE,
        core::ptr::addr_of_mut!(cbValue),
    );

    if error != ERROR_SUCCESS as LONG || cbValue < core::mem::size_of::<DWORD>() as DWORD {
        return 0;
    }

    dwValue
}

#[no_mangle]
pub unsafe extern "system" fn CliImmSetHotKeyWorker(
    dwHotKeyId: DWORD,
    uModifiers: UINT,
    uVirtualKey: UINT,
    hKL: HKL,
    dwAction: DWORD,
) -> BOOL {
    const MOD_ALL_MODS: UINT = MOD_ALT | MOD_CONTROL | MOD_SHIFT | MOD_WIN;

    'failure: {
        if dwAction == SETIMEHOTKEY_ADD {
            if IME_HOTKEY_DSWITCH_FIRST <= dwHotKeyId && dwHotKeyId <= IME_HOTKEY_DSWITCH_LAST {
                if hKL.is_null() {
                    break 'failure;
                }
            } else {
                if !hKL.is_null() {
                    break 'failure;
                }

                if IME_KHOTKEY_SHAPE_TOGGLE <= dwHotKeyId
                    && dwHotKeyId < IME_THOTKEY_IME_NONIME_TOGGLE
                {
                    break 'failure;
                }
            }

            if (uModifiers & MOD_ALL_MODS) != 0 && (uModifiers & (MOD_LEFT | MOD_RIGHT)) == 0 {
                break 'failure;
            }
        }

        return NtUserSetImeHotKey(dwHotKeyId, uModifiers, uVirtualKey, hKL, dwAction);
    }

    SetLastError(ERROR_INVALID_PARAMETER);
    FALSE as BOOL
}

// Cross-TU export: the surviving logon.c calls plain-cdecl `_IntLoadPreloadKeyboardLayouts`.
#[no_mangle]
pub unsafe extern "C" fn IntLoadPreloadKeyboardLayouts() {

    const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;

    let mut nNumber: UINT;
    let mut uFlags: UINT;
    let mut cbValue: DWORD;
    let mut dwType: DWORD = 0;
    let mut szNumber: [u16; 32] = [0; 32];
    let mut szValue: [u16; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];
    let mut hPreloadKey: HKEY = core::ptr::null_mut();
    let mut bOK: BOOL = 0;
    let mut hKL: HKL;
    let mut hDefaultKL: HKL = core::ptr::null_mut();

    let preload: [u16; 25] = [
        b'K' as u16, b'e' as u16, b'y' as u16, b'b' as u16, b'o' as u16, b'a' as u16, b'r' as u16,
        b'd' as u16, b' ' as u16, b'L' as u16, b'a' as u16, b'y' as u16, b'o' as u16, b'u' as u16,
        b't' as u16, b'\\' as u16, b'P' as u16, b'r' as u16, b'e' as u16, b'l' as u16, b'o' as u16,
        b'a' as u16, b'd' as u16, 0, 0,
    ];

    if RegOpenKeyW(HKEY_CURRENT_USER, preload.as_ptr(), &mut hPreloadKey) != ERROR_SUCCESS as i32 {
        return;
    }

    nNumber = 1;
    while nNumber <= 1000 {
        _ultow(nNumber, szNumber.as_mut_ptr(), 10);

        cbValue = core::mem::size_of_val(&szValue) as DWORD;
        if RegQueryValueExW(
            hPreloadKey,
            szNumber.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            szValue.as_mut_ptr() as LPBYTE,
            &mut cbValue,
        ) != ERROR_SUCCESS as i32
        {
            break;
        }

        if dwType != REG_SZ {
            nNumber += 1;
            continue;
        }

        if nNumber == 1 {
            uFlags = KLF_SUBSTITUTE_OK | KLF_ACTIVATE | KLF_RESET;
        } else {
            uFlags = KLF_SUBSTITUTE_OK | KLF_NOTELLSHELL | KLF_REPLACELANG;
        }

        hKL = LoadKeyboardLayoutW(szValue.as_ptr(), uFlags);
        if !hKL.is_null() {
            bOK = 1;
            if nNumber == 1 {
                hDefaultKL = hKL;
            }
        }

        nNumber += 1;
    }

    RegCloseKey(hPreloadKey);

    if !hDefaultKL.is_null() {
        SystemParametersInfoW(
            SPI_SETDEFAULTINPUTLANG,
            0,
            &mut hDefaultKL as *mut HKL as *mut core::ffi::c_void,
            0,
        );
    }

    if bOK == 0 {
        let fallback: [u16; 9] = [
            b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16, b'0' as u16, b'4' as u16,
            b'0' as u16, b'9' as u16, 0,
        ];
        LoadKeyboardLayoutW(fallback.as_ptr(), KLF_SUBSTITUTE_OK | KLF_ACTIVATE | KLF_RESET);
    }
}

unsafe extern "system" fn CliSaveImeHotKey(
    dwID: DWORD,
    uModifiers: UINT,
    uVirtualKey: UINT,
    hKL: HKL,
    bDelete: BOOL,
) -> BOOL {

    const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;

    let mut szName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut error: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut bRevertOnFailure: BOOL = 0;

    let fmt: &[u16] = &[
        0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061,
        0x006E, 0x0065, 0x006C, 0x005C, 0x0049, 0x006E, 0x0070, 0x0075, 0x0074, 0x0020,
        0x004D, 0x0065, 0x0074, 0x0068, 0x006F, 0x0064, 0x005C, 0x0048, 0x006F, 0x0074,
        0x0020, 0x004B, 0x0065, 0x0079, 0x0073, 0x005C, 0x0025, 0x0030, 0x0038, 0x006C,
        0x0058, 0x0000,
    ];

    wsprintfW(
        szName.as_mut_ptr(),
        fmt.as_ptr(),
        dwID,
    );

    if bDelete != 0 {
        error = RegDeleteKeyW(HKEY_CURRENT_USER, szName.as_ptr());
        return (error == ERROR_SUCCESS as LONG) as BOOL;
    }

    let vk_name: &[u16] = &[
        0x0056, 0x0069, 0x0072, 0x0074, 0x0075, 0x0061, 0x006C, 0x0020, 0x004B, 0x0065,
        0x0079, 0x0000,
    ];
    let km_name: &[u16] = &[
        0x004B, 0x0065, 0x0079, 0x0020, 0x004D, 0x006F, 0x0064, 0x0069, 0x0066, 0x0069,
        0x0065, 0x0072, 0x0073, 0x0000,
    ];
    let ti_name: &[u16] = &[
        0x0054, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0020, 0x0049, 0x004D, 0x0045,
        0x0000,
    ];

    error = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        szName.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    );
    if error == ERROR_SUCCESS as LONG {
        bRevertOnFailure = 1;

        error = RegSetValueExW(
            hKey,
            vk_name.as_ptr(),
            0,
            REG_BINARY,
            core::ptr::addr_of!(uVirtualKey) as *const u8,
            core::mem::size_of::<UINT>() as DWORD,
        );
        if error == ERROR_SUCCESS as LONG {
            error = RegSetValueExW(
                hKey,
                km_name.as_ptr(),
                0,
                REG_BINARY,
                core::ptr::addr_of!(uModifiers) as *const u8,
                core::mem::size_of::<UINT>() as DWORD,
            );
            if error == ERROR_SUCCESS as LONG {
                error = RegSetValueExW(
                    hKey,
                    ti_name.as_ptr(),
                    0,
                    REG_BINARY,
                    core::ptr::addr_of!(hKL) as *const u8,
                    core::mem::size_of::<HKL>() as DWORD,
                );
                if error == ERROR_SUCCESS as LONG {
                    ret = 1;
                    bRevertOnFailure = 0;
                }
            }
        }
        RegCloseKey(hKey);
    }

    if bRevertOnFailure != 0 {
        CliSaveImeHotKey(dwID, uVirtualKey, uModifiers, hKL, 1);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn CliImmSetHotKey(
    dwID: DWORD,
    uModifiers: UINT,
    uVirtualKey: UINT,
    hKL: HKL,
) -> BOOL {
    let mut ret: BOOL;

    if uVirtualKey == 0 {
        ret = CliSaveImeHotKey(dwID, uModifiers, uVirtualKey, hKL, 1);
        if ret != 0 {
            CliImmSetHotKeyWorker(dwID, uModifiers, uVirtualKey, hKL, SETIMEHOTKEY_DELETE);
        }
        return ret;
    }

    ret = CliImmSetHotKeyWorker(dwID, uModifiers, uVirtualKey, hKL, SETIMEHOTKEY_ADD);
    if ret != 0 {
        ret = CliSaveImeHotKey(dwID, uModifiers, uVirtualKey, hKL, 0);
        if ret == 0 {
            CliImmSetHotKeyWorker(dwID, uModifiers, uVirtualKey, hKL, SETIMEHOTKEY_DELETE);
        }
    }

    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CliSetSingleHotKey(pszSubKey: LPCWSTR, hKey: HANDLE) -> BOOL {
    CliSetSingleHotKeyImpl(pszSubKey, hKey)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CliSetSingleHotKey(pszSubKey: LPCWSTR, hKey: HANDLE) -> BOOL {
    CliSetSingleHotKeyImpl(pszSubKey, hKey)
}

unsafe fn CliSetSingleHotKeyImpl(pszSubKey: LPCWSTR, hKey: HANDLE) -> BOOL {
    let mut error: LONG;
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut dwHotKeyId: DWORD = 0;
    let mut uModifiers: UINT;
    let mut uVirtualKey: UINT;
    let hKL: HKL;
    let mut ustrName: _UNICODE_STRING = core::mem::zeroed();

    error = RegOpenKeyExW(
        hKey as HKEY,
        pszSubKey,
        0,
        KEY_READ,
        &mut hSubKey,
    );
    if error != ERROR_SUCCESS as LONG {
        return 0;
    }

    RtlInitUnicodeString(&mut ustrName, pszSubKey);
    RtlUnicodeStringToInteger(&mut ustrName, 16, &mut dwHotKeyId);

    uModifiers = CliReadRegistryValue(hSubKey as HANDLE, w!("Key Modifiers").as_ptr() as LPCWSTR);
    hKL = CliReadRegistryValue(hSubKey as HANDLE, w!("Target IME").as_ptr() as LPCWSTR) as ULONG_PTR as HKL;
    uVirtualKey = CliReadRegistryValue(hSubKey as HANDLE, w!("Virtual Key").as_ptr() as LPCWSTR);

    RegCloseKey(hSubKey);

    CliImmSetHotKeyWorker(dwHotKeyId, uModifiers, uVirtualKey, hKL, SETIMEHOTKEY_ADD)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CliGetImeHotKeysFromRegistry() -> BOOL {
    CliGetImeHotKeysFromRegistryImpl()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CliGetImeHotKeysFromRegistry() -> BOOL {
    CliGetImeHotKeysFromRegistryImpl()
}

unsafe fn CliGetImeHotKeysFromRegistryImpl() -> BOOL {

    let mut hKey: HKEY = core::ptr::null_mut();
    let error: LONG;
    let mut ret: BOOL = 0;
    let mut cchKeyName: DWORD;
    let mut szKeyName: [WCHAR; 16] = [0; 16];

    let subkey: [u16; 38] = [
        0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061, 0x006E,
        0x0065, 0x006C, 0x005C, 0x0049, 0x006E, 0x0070, 0x0075, 0x0074, 0x0020, 0x004D, 0x0065,
        0x0074, 0x0068, 0x006F, 0x0064, 0x005C, 0x0048, 0x006F, 0x0074, 0x0020, 0x004B, 0x0065,
        0x0079, 0x0073, 0x0000, 0x0000, 0x0000,
    ];

    error = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        subkey.as_ptr(),
        0,
        KEY_READ as REGSAM,
        &mut hKey,
    );
    if error != ERROR_SUCCESS as LONG {
        return ret;
    }

    let mut dwIndex: DWORD = 0;
    while dwIndex < 1000 {
        cchKeyName = szKeyName.len() as DWORD;
        let error = RegEnumKeyExW(
            hKey,
            dwIndex,
            szKeyName.as_mut_ptr(),
            &mut cchKeyName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if error != ERROR_SUCCESS as LONG {
            break;
        }

        szKeyName[szKeyName.len() - 1] = 0;

        if CliSetSingleHotKey(szKeyName.as_ptr(), hKey as HANDLE) != 0 {
            ret = 1;
        }

        dwIndex += 1;
    }

    RegCloseKey(hKey);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CliGetPreloadKeyboardLayouts(pbFlags: PBYTE) {
    let mut szValueName: [u16; 33] = [0; 33];
    let mut szValue: [u16; 16] = [0; 16];
    let mut ustrValue: _UNICODE_STRING = core::mem::zeroed();
    let mut dwKL: DWORD = 0;
    let mut cbValue: DWORD;
    let mut dwType: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    let error = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Keyboard Layout\\Preload").as_ptr() as LPCWSTR,
        0,
        KEY_READ,
        &mut hKey,
    );
    if error != ERROR_SUCCESS as LONG {
        return;
    }

    let mut iNumber: UINT = 1;
    while iNumber < 1000 {
        _ultow(iNumber, szValueName.as_mut_ptr(), 10);

        cbValue = core::mem::size_of_val(&szValue) as DWORD;
        let error = RegQueryValueExW(
            hKey,
            szValueName.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            szValue.as_mut_ptr() as LPBYTE,
            &mut cbValue,
        );
        if error != ERROR_SUCCESS as LONG {
            break;
        }

        if dwType != REG_SZ {
            iNumber += 1;
            continue;
        }

        szValue[szValue.len() - 1] = 0;

        RtlInitUnicodeString(&mut ustrValue, szValue.as_ptr());
        RtlUnicodeStringToInteger(&ustrValue, 16, &mut dwKL);

        IntSetFeKeyboardFlags(LOWORD(dwKL), pbFlags);

        iNumber += 1;
    }

    RegCloseKey(hKey);
}

#[no_mangle]
pub unsafe extern "system" fn CliSetDefaultImeHotKeys(
    mut pEntries: PIMEHOTKEYENTRY,
    mut nCount: UINT,
    bCheck: BOOL,
) {
    let mut uVirtualKey: UINT = 0;
    let mut uModifiers: UINT = 0;
    let mut hKL: HKL = core::ptr::null_mut();

    while {
        let old = nCount;
        nCount = nCount.wrapping_sub(1);
        old > 0
    } {
        if bCheck == 0
            || NtUserGetImeHotKey(
                (*pEntries).dwHotKeyId,
                &mut uModifiers,
                &mut uVirtualKey,
                &mut hKL,
            ) == 0
        {
            CliImmSetHotKeyWorker(
                (*pEntries).dwHotKeyId,
                (*pEntries).uModifiers,
                (*pEntries).uVirtualKey,
                (*pEntries).hKL,
                SETIMEHOTKEY_ADD as DWORD,
            );
        }
        pEntries = pEntries.add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CliImmInitializeHotKeys(dwAction: DWORD, hKL: HKL) {
    let nCount: UINT;
    let pList: LPHKL;
    let mut iIndex: UINT;
    let LangID: LANGID;
    let mut bFlags: BYTE = 0;
    let bCheck: BOOL;

    NtUserSetImeHotKey(0, 0, 0, core::ptr::null_mut(), SETIMEHOTKEY_INITIALIZE);

    bCheck = CliGetImeHotKeysFromRegistry();

    if dwAction == SETIMEHOTKEY_INITIALIZE {
        LangID = LANGIDFROMLCID(GetUserDefaultLCID()) as LANGID;
        IntSetFeKeyboardFlags(LangID, core::ptr::addr_of_mut!(bFlags));

        CliGetPreloadKeyboardLayouts(core::ptr::addr_of_mut!(bFlags));
    } else {
        nCount = NtUserGetKeyboardLayoutList(0, core::ptr::null_mut());
        if nCount == 0 {
            return;
        }

        pList = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            ((nCount as usize) * core::mem::size_of::<HKL>()) as SIZE_T,
        ) as LPHKL;
        if pList.is_null() {
            return;
        }

        NtUserGetKeyboardLayoutList(nCount as ULONG, pList);

        iIndex = 0;
        while iIndex < nCount {
            let LangID2 = LOWORD(*pList.add(iIndex as usize) as usize as DWORD) as LANGID;
            IntSetFeKeyboardFlags(LangID2, core::ptr::addr_of_mut!(bFlags));
            iIndex += 1;
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, pList as PVOID);
    }

    if (bFlags as u32 & FE_JAPANESE) != 0 {
        CliSetDefaultImeHotKeys(
            (*(&raw mut DefaultHotKeyTableJ)).as_mut_ptr(),
            (*(&raw const DefaultHotKeyTableJ)).len() as UINT,
            bCheck,
        );
    }

    if (bFlags as u32 & FE_CHINESE_TRADITIONAL) != 0 {
        CliSetDefaultImeHotKeys(
            (*(&raw mut DefaultHotKeyTableT)).as_mut_ptr(),
            (*(&raw const DefaultHotKeyTableT)).len() as UINT,
            bCheck,
        );
    }

    if (bFlags as u32 & FE_CHINESE_SIMPLIFIED) != 0 {
        CliSetDefaultImeHotKeys(
            (*(&raw mut DefaultHotKeyTableC)).as_mut_ptr(),
            (*(&raw const DefaultHotKeyTableC)).len() as UINT,
            bCheck,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn DragDetect(hWnd: HWND, pt: POINT) -> BOOL {
    // NtUserDragDetect's bindgen sig spells the point as POINTL; POINT/POINTL are
    // layout-identical ReactOS aliases.
    NtUserDragDetect(hWnd, core::mem::transmute::<POINT, POINTL>(pt))
}

#[no_mangle]
pub unsafe extern "system" fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL {
    NtUserxEnableWindow(hWnd, bEnable)
}

#[no_mangle]
pub unsafe extern "system" fn GetAsyncKeyState(vKey: core::ffi::c_int) -> SHORT {
    if vKey < 0 || vKey > 256 {
        return 0;
    }
    NtUserGetAsyncKeyState(vKey as INT) as SHORT
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyboardLayout(idThread: DWORD) -> HKL {
    NtUserxGetKeyboardLayout(idThread)
}

#[no_mangle]
pub unsafe extern "system" fn GetKBCodePage() -> UINT {
    GetOEMCP()
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyNameTextA(
    lParam: LONG,
    lpString: LPSTR,
    nSize: core::ffi::c_int,
) -> core::ffi::c_int {

    const CP_ACP: UINT = 0;

    let pwszBuf: LPWSTR;
    let cchBuf: UINT;
    let iRet: core::ffi::c_int;
    let mut defChar: core::ffi::c_int = 0;

    pwszBuf = HeapAlloc(
        GetProcessHeap(),
        0,
        ((nSize as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if pwszBuf.is_null() {
        return 0;
    }

    cchBuf = NtUserGetKeyNameText(lParam, pwszBuf, nSize) as UINT;

    iRet = WideCharToMultiByte(
        CP_ACP,
        0,
        pwszBuf,
        cchBuf as core::ffi::c_int,
        lpString,
        nSize,
        b".\0".as_ptr() as LPCSTR,
        &mut defChar,
    );
    // §2.5: volatile NUL-terminator store through the caller's output buffer.
    core::ptr::write_volatile(lpString.offset(iRet as isize), 0);
    HeapFree(GetProcessHeap(), 0, pwszBuf as *mut core::ffi::c_void);

    iRet
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyNameTextW(
    lParam: LONG,
    lpString: LPWSTR,
    nSize: core::ffi::c_int,
) -> core::ffi::c_int {
    NtUserGetKeyNameText(lParam, lpString, nSize) as core::ffi::c_int
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyState(nVirtKey: core::ffi::c_int) -> i16 {
    NtUserGetKeyState(nVirtKey as INT) as i16
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyboardLayoutNameA(pwszKLID: LPSTR) -> BOOL {
    let mut buf: [WCHAR; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];

    if GetKeyboardLayoutNameW(buf.as_mut_ptr()) == 0 {
        return 0;
    }

    if WideCharToMultiByte(
        CP_ACP,
        0,
        buf.as_ptr(),
        -1,
        pwszKLID,
        KL_NAMELENGTH as i32,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyboardLayoutNameW(pwszKLID: LPWSTR) -> BOOL {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlInitEmptyUnicodeString(
        &mut Name,
        pwszKLID,
        (KL_NAMELENGTH as usize * core::mem::size_of::<u16>()) as u16,
    );

    NtUserGetKeyboardLayoutName(&mut Name)
}

#[no_mangle]
pub unsafe extern "system" fn GetKeyboardType(nTypeFlag: core::ffi::c_int) -> core::ffi::c_int {
    NtUserxGetKeyboardType(nTypeFlag)
}

#[no_mangle]
pub unsafe extern "system" fn GetLastInputInfo(plii: PLASTINPUTINFO) -> i32 {
    if (*plii).cbSize as usize != core::mem::size_of_val(&*plii) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let dwTime = if !gpsi.is_null() {
        crate::wow64::psi::last_rit_event_tick()
    } else {
        0
    };
    // §2.5: volatile store through the output pointer (defeat captures(none)).
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*plii).dwTime), dwTime);

    (!gpsi.is_null()) as i32
}

#[no_mangle]
pub unsafe extern "system" fn LoadKeyboardLayoutA(
    pszKLID: LPCSTR,
    Flags: UINT,
) -> HKL {
    let mut wszKLID: [u16; 16] = [0; 16];

    if MultiByteToWideChar(
        CP_ACP,
        0,
        pszKLID,
        -1,
        wszKLID.as_mut_ptr(),
        (core::mem::size_of::<[u16; 16]>() / core::mem::size_of::<u16>()) as i32,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    LoadKeyboardLayoutW(wszKLID.as_ptr(), Flags)
}

unsafe fn IsValidKLID(pwszKLID: LPCWSTR) -> i32 {
    let accept: [u16; 23] = [
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
        0x41, 0x42, 0x43, 0x44, 0x45, 0x46,
        0x61, 0x62, 0x63, 0x64, 0x65, 0x66,
        0x00,
    ];
    ((!pwszKLID.is_null())
        && (wcsspn(pwszKLID, accept.as_ptr()) == (KL_NAMELENGTH - 1) as usize)) as i32
}

unsafe fn GetSystemLibraryPath(pszPath: LPWSTR, cchPath: INT, pszFileName: LPCWSTR) {
    let mut szSysDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    GetSystemDirectoryW(szSysDir.as_mut_ptr(), szSysDir.len() as u32);
    /* StringCchPrintfW(pszPath, cchPath, ...): the output is bounded by cchPath and
     * always NUL-terminated (truncated if it does not fit). wsprintfW alone knows no
     * destination size, and the file name here comes from the registry, so format
     * into wsprintfW's own 1024-char ceiling and copy at most cchPath - 1 back. */
    if cchPath <= 0 {
        return;
    }
    let mut szTemp: [u16; 1025] = [0; 1025];
    wsprintfW(
        szTemp.as_mut_ptr(),
        w!("%s\\%s").as_ptr() as LPCWSTR,
        szSysDir.as_ptr(),
        pszFileName,
    );
    let mut i = 0usize;
    while i + 1 < cchPath as usize && szTemp[i] != 0 {
        *pszPath.add(i) = szTemp[i];
        i += 1;
    }
    *pszPath.add(i) = 0;
}

#[no_mangle]
pub unsafe extern "system" fn IntLoadKeyboardLayout(
    hklUnload: HKL,
    mut pwszKLID: LPCWSTR,
    wLangID: LANGID,
    Flags: UINT,
    unknown5: BOOL,
) -> HKL {
    let mut dwKLID: DWORD;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let dwHKL: DWORD;
    let mut ustrKLID: _UNICODE_STRING = core::mem::zeroed();
    let mut wszRegKey: [WCHAR; 256] = [0; 256];
    {
        let init: &[u16] = &[
            0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
            0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
            0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
            0x006F, 0x006C, 0x005C, 0x004B, 0x0065, 0x0079, 0x0062, 0x006F, 0x0061, 0x0072,
            0x0064, 0x0020, 0x004C, 0x0061, 0x0079, 0x006F, 0x0075, 0x0074, 0x0073, 0x005C,
        ];
        core::ptr::copy_nonoverlapping(init.as_ptr(), wszRegKey.as_mut_ptr(), init.len());
    }
    let mut wszLayoutId: [WCHAR; 10] = [0; 10];
    let mut wszNewKLID: [WCHAR; KL_NAMELENGTH as usize] = [0; KL_NAMELENGTH as usize];
    let mut szImeFileName: [WCHAR; 80] = [0; 80];
    let hNewKL: HKL;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut bIsIME: BOOL;
    let wLow: WORD;
    let mut wHigh: WORD;

    if IsValidKLID(pwszKLID) == 0 {
        return UlongToHandle(MAKELONG(ENGLISH_US, ENGLISH_US) as ULONG) as HKL;
    }

    dwKLID = wcstoul(pwszKLID, core::ptr::null_mut(), 16);
    bIsIME = IS_IME_HKL(UlongToHandle(dwKLID) as HKL);

    wLow = LOWORD(dwKLID);
    wHigh = HIWORD(dwKLID);

    if (Flags & KLF_SUBSTITUTE_OK) != 0 {
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Keyboard Layout\\Substitutes").as_ptr() as LPCWSTR,
            0,
            KEY_READ,
            &mut hKey,
        ) == ERROR_SUCCESS as i32
        {
            dwSize = core::mem::size_of_val(&wszNewKLID) as DWORD;
            if RegQueryValueExW(
                hKey,
                pwszKLID,
                core::ptr::null_mut(),
                &mut dwType,
                wszNewKLID.as_mut_ptr() as LPBYTE,
                &mut dwSize,
            ) == ERROR_SUCCESS as i32
                && dwType == REG_SZ
            {
                pwszKLID = wszNewKLID.as_ptr();
                dwKLID = wcstoul(pwszKLID, core::ptr::null_mut(), 16);
                wHigh = LOWORD(dwKLID);
            }

            RegCloseKey(hKey);
        }
    }

    StringCbCatW(
        wszRegKey.as_mut_ptr(),
        core::mem::size_of_val(&wszRegKey),
        pwszKLID,
    );

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wszRegKey.as_ptr(),
        0,
        KEY_READ,
        &mut hKey,
    ) == ERROR_SUCCESS as i32
    {
        dwSize = core::mem::size_of_val(&wszLayoutId) as DWORD;
        if RegQueryValueExW(
            hKey,
            w!("Layout Id").as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            &mut dwType,
            wszLayoutId.as_mut_ptr() as LPBYTE,
            &mut dwSize,
        ) == ERROR_SUCCESS as i32
            && dwType == REG_SZ
        {
            wHigh = (0xF000 | wcstoul(wszLayoutId.as_ptr(), core::ptr::null_mut(), 16)) as WORD;
        }

        if bIsIME != 0 {
            dwSize = core::mem::size_of_val(&szImeFileName) as DWORD;
            if RegQueryValueExW(
                hKey,
                w!("IME File").as_ptr() as LPCWSTR,
                core::ptr::null_mut(),
                &mut dwType,
                szImeFileName.as_mut_ptr() as LPBYTE,
                &mut dwSize,
            ) != ERROR_SUCCESS as i32
            {
                bIsIME = 0;
                wHigh = 0;
            } else {
                let mut szPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
                szImeFileName[_countof(&szImeFileName) - 1] = 0;
                GetSystemLibraryPath(
                    szPath.as_mut_ptr(),
                    _countof(&szPath) as INT,
                    szImeFileName.as_ptr(),
                );

                if dwType != REG_SZ
                    || szImeFileName[0] == 0
                    || wcscspn(szImeFileName.as_ptr(), w!(":\\/").as_ptr() as LPCWSTR)
                        != wcslen(szImeFileName.as_ptr())
                    || GetFileAttributesW(szPath.as_ptr()) == INVALID_FILE_ATTRIBUTES
                {
                    bIsIME = 0;
                    wHigh = 0;
                }
            }
        }

        RegCloseKey(hKey);
    } else {
        return core::ptr::null_mut();
    }

    if wHigh == 0 {
        wHigh = wLow;
    }

    dwHKL = MAKELONG(wLow, wHigh) as DWORD;

    RtlInitUnicodeString(&mut ustrKLID, pwszKLID);
    hNewKL = NtUserLoadKeyboardLayoutEx(
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        hklUnload,
        &mut ustrKLID,
        dwHKL,
        Flags,
    );
    CliImmInitializeHotKeys(SETIMEHOTKEY_ADD, hNewKL);
    hNewKL
}

#[no_mangle]
pub unsafe extern "system" fn LoadKeyboardLayoutW(
    pwszKLID: LPCWSTR,
    Flags: UINT,
) -> HKL {
    IntLoadKeyboardLayout(core::ptr::null_mut(), pwszKLID, 0, Flags, 0)
}

#[no_mangle]
pub unsafe extern "system" fn LoadKeyboardLayoutEx(
    hklUnload: HKL,
    pwszKLID: LPCWSTR,
    Flags: UINT,
) -> HKL {
    if hklUnload.is_null() {
        return core::ptr::null_mut();
    }
    IntLoadKeyboardLayout(hklUnload, pwszKLID, 0, Flags, 0)
}

#[no_mangle]
pub unsafe extern "system" fn UnloadKeyboardLayout(hKL: HKL) -> BOOL {
    if NtUserUnloadKeyboardLayout(hKL) == 0 {
        return 0;
    }

    CliImmInitializeHotKeys(SETIMEHOTKEY_DELETE as DWORD, hKL);
    1
}

#[no_mangle]
pub unsafe extern "system" fn MapVirtualKeyA(uCode: UINT, uMapType: UINT) -> UINT {
    MapVirtualKeyExA(uCode, uMapType, GetKeyboardLayout(0))
}

#[no_mangle]
pub unsafe extern "system" fn MapVirtualKeyExA(uCode: UINT, uMapType: UINT, dwhkl: HKL) -> UINT {
    MapVirtualKeyExW(uCode, uMapType, dwhkl)
}

#[no_mangle]
pub unsafe extern "system" fn MapVirtualKeyExW(uCode: UINT, uMapType: UINT, dwhkl: HKL) -> UINT {
    NtUserMapVirtualKeyEx(uCode, uMapType, 0, dwhkl)
}

#[no_mangle]
pub unsafe extern "system" fn MapVirtualKeyW(uCode: UINT, uMapType: UINT) -> UINT {
    MapVirtualKeyExW(uCode, uMapType, GetKeyboardLayout(0))
}

#[no_mangle]
pub unsafe extern "system" fn OemKeyScan(wOemChar: WORD) -> DWORD {
    let mut p: WCHAR = 0;
    let Vk: SHORT;
    let Scan: UINT;

    MultiByteToWideChar(
        CP_OEMCP,
        0,
        core::ptr::addr_of!(wOemChar) as PCSTR,
        1,
        &mut p,
        1,
    );
    Vk = VkKeyScanW(p);
    Scan = MapVirtualKeyW((Vk as UINT) & 0x00ff, 0);
    if Scan == 0 {
        return -1i32 as DWORD;
    }
    ((((Vk as UINT) & 0xff00) << 8) | Scan) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn SetDoubleClickTime(uInterval: UINT) -> BOOL {
    NtUserSystemParametersInfo(
        SPI_SETDOUBLECLICKTIME,
        uInterval,
        core::ptr::null_mut(),
        0,
    ) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SwapMouseButton(fSwap: BOOL) -> BOOL {
    NtUserxSwapMouseButton(fSwap)
}

#[no_mangle]
pub unsafe extern "system" fn ToAscii(
    uVirtKey: UINT,
    uScanCode: UINT,
    lpKeyState: *const BYTE,
    lpChar: LPWORD,
    uFlags: UINT,
) -> core::ffi::c_int {
    ToAsciiEx(uVirtKey, uScanCode, lpKeyState, lpChar, uFlags, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn ToAsciiEx(
    uVirtKey: UINT,
    uScanCode: UINT,
    lpKeyState: *const BYTE,
    lpChar: LPWORD,
    uFlags: UINT,
    dwhkl: HKL,
) -> core::ffi::c_int {
    let mut UniChars: [WCHAR; 2] = [0; 2];
    let Ret: core::ffi::c_int;
    let CharCount: core::ffi::c_int;

    Ret = ToUnicodeEx(
        uVirtKey,
        uScanCode,
        lpKeyState,
        UniChars.as_mut_ptr(),
        2,
        uFlags,
        dwhkl,
    );
    CharCount = if Ret < 0 { 1 } else { Ret };
    WideCharToMultiByte(
        CP_ACP,
        0,
        UniChars.as_ptr(),
        CharCount,
        lpChar as LPSTR,
        2,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn ToUnicode(
    wVirtKey: UINT,
    wScanCode: UINT,
    lpKeyState: *const BYTE,
    pwszBuff: LPWSTR,
    cchBuff: core::ffi::c_int,
    wFlags: UINT,
) -> core::ffi::c_int {
    ToUnicodeEx(
        wVirtKey,
        wScanCode,
        lpKeyState,
        pwszBuff,
        cchBuff,
        wFlags,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn ToUnicodeEx(
    wVirtKey: UINT,
    wScanCode: UINT,
    lpKeyState: *const BYTE,
    pwszBuff: LPWSTR,
    cchBuff: core::ffi::c_int,
    wFlags: UINT,
    dwhkl: HKL,
) -> core::ffi::c_int {
    NtUserToUnicodeEx(
        wVirtKey,
        wScanCode,
        lpKeyState as PBYTE,
        pwszBuff,
        cchBuff,
        wFlags,
        dwhkl,
    )
}

#[no_mangle]
pub unsafe extern "system" fn VkKeyScanA(ch: CHAR) -> SHORT {
    let mut wChar: WCHAR = 0;

    if IsDBCSLeadByte(ch as u8) != 0 {
        return -1;
    }

    MultiByteToWideChar(CP_ACP, 0, core::ptr::addr_of!(ch) as LPCSTR, 1, core::ptr::addr_of_mut!(wChar), 1);
    VkKeyScanW(wChar)
}

#[no_mangle]
pub unsafe extern "system" fn VkKeyScanExA(ch: CHAR, dwhkl: HKL) -> SHORT {
    let mut wChar: WCHAR = 0;

    if IsDBCSLeadByte(ch as u8) != 0 {
        return -1;
    }

    MultiByteToWideChar(CP_ACP, 0, &ch as *const CHAR, 1, &mut wChar, 1);
    VkKeyScanExW(wChar, dwhkl)
}

#[no_mangle]
pub unsafe extern "system" fn VkKeyScanExW(ch: WCHAR, dwhkl: HKL) -> SHORT {
    NtUserVkKeyScanEx(ch, dwhkl, TRUE as BOOL) as SHORT
}

#[no_mangle]
pub unsafe extern "system" fn VkKeyScanW(ch: WCHAR) -> SHORT {
    NtUserVkKeyScanEx(ch, core::ptr::null_mut(), FALSE as BOOL) as SHORT
}

#[no_mangle]
pub unsafe extern "system" fn keybd_event(
    bVk: BYTE,
    bScan: BYTE,
    dwFlags: DWORD,
    dwExtraInfo: ULONG_PTR,
) {
    let mut Input: INPUT = core::mem::zeroed();

    Input.type_ = INPUT_KEYBOARD;
    Input.__bindgen_anon_1.ki.wVk = bVk as WORD;
    Input.__bindgen_anon_1.ki.wScan = bScan as WORD;
    Input.__bindgen_anon_1.ki.dwFlags = dwFlags;
    Input.__bindgen_anon_1.ki.time = 0;
    Input.__bindgen_anon_1.ki.dwExtraInfo = dwExtraInfo;

    NtUserSendInput(1, &mut Input, core::mem::size_of::<INPUT>() as i32);
}

#[no_mangle]
pub unsafe extern "system" fn mouse_event(
    dwFlags: DWORD,
    dx: DWORD,
    dy: DWORD,
    dwData: DWORD,
    dwExtraInfo: ULONG_PTR,
) {
    let mut Input: INPUT = core::mem::zeroed();

    Input.type_ = INPUT_MOUSE;
    Input.__bindgen_anon_1.mi.dx = dx as _;
    Input.__bindgen_anon_1.mi.dy = dy as _;
    Input.__bindgen_anon_1.mi.mouseData = dwData;
    Input.__bindgen_anon_1.mi.dwFlags = dwFlags;
    Input.__bindgen_anon_1.mi.time = 0;
    Input.__bindgen_anon_1.mi.dwExtraInfo = dwExtraInfo;

    NtUserSendInput(1, &mut Input, core::mem::size_of::<INPUT>() as _);
}
