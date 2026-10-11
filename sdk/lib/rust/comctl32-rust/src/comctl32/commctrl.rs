use crate::comctl32::toolbar::{TOOLBAR_Register, TOOLBAR_Unregister, TOOLBARv6_Register, TOOLBARv6_Unregister};
use crate::comctl32::animate::ANIMATE_Register;
use crate::comctl32::animate::ANIMATE_Unregister;
use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::comboex::COMBOEX_Register;
use crate::comctl32::comboex::COMBOEX_Unregister;
use crate::comctl32::comctl32undoc::Free;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
crate::wine_default_debug_channel!(__wine_dbch_commctrl);
pub static RUSTCONV_WSTR_33A2A8F3E59F: [u16; 8] = [0x005Cu16, 0x0077, 0x0069, 0x006E, 0x0073, 0x0078, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_35F38A602832: [u16; 12] = [
    0x005Cu16, 0x006D, 0x0061, 0x006E, 0x0069, 0x0066, 0x0065, 0x0073, 0x0074, 0x0073,
    0x005C, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const TOOLBARCLASSNAMEW: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"T\0o\0o\0l\0b\0a\0r\0W\0i\0n\0d\0o\0w\x003\x002\0\0\0",
    )
};

pub const TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

static mut COMCTL32_wSubclass: LPWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut COMCTL32_hModule: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut COMCTL32_uiLang: LANGID =
    ((0 as ::core::ffi::c_int as USHORT as ::core::ffi::c_int) << 10 as ::core::ffi::c_int
        | 0 as ::core::ffi::c_int as USHORT as ::core::ffi::c_int) as LANGID;

#[no_mangle]
pub static mut COMCTL32_hPattern55AABrush: HBRUSH =
    ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__;

#[no_mangle]
pub static mut comctl32_color: COMCTL32_SysColor = COMCTL32_SysColor {
    clrBtnHighlight: 0,
    clrBtnShadow: 0,
    clrBtnText: 0,
    clrBtnFace: 0,
    clrHighlight: 0,
    clrHighlightText: 0,
    clrHotTrackingColor: 0,
    clr3dHilight: 0,
    clr3dShadow: 0,
    clr3dDkShadow: 0,
    clr3dFace: 0,
    clrWindow: 0,
    clrWindowText: 0,
    clrGrayText: 0,
    clrActiveCaption: 0,
    clrInfoBk: 0,
    clrInfoText: 0,
};

static mut COMCTL32_hPattern55AABitmap: HBITMAP =
    ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

static mut wPattern55AA: [WORD; 8] = [
    0x5555 as ::core::ffi::c_int as WORD,
    0xaaaa as ::core::ffi::c_int as WORD,
    0x5555 as ::core::ffi::c_int as WORD,
    0xaaaa as ::core::ffi::c_int as WORD,
    0x5555 as ::core::ffi::c_int as WORD,
    0xaaaa as ::core::ffi::c_int as WORD,
    0x5555 as ::core::ffi::c_int as WORD,
    0xaaaa as ::core::ffi::c_int as WORD,
];

static mut strCC32SubclassInfo: [WCHAR; 17] = [
    'C' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    '3' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// The C builds both names from `ARCH`, a per-target #define (x86/amd64/arm64). The
// translation went through the preprocessor once, under the arm64 profile, and baked
// "arm64_" into both arrays: i386 and amd64 then asked for arm64_*.manifest, CreateActCtxW
// failed with ERROR_FILE_NOT_FOUND, and neither the v5 nor the v6 activation context existed.
#[cfg(target_arch = "x86")]
const ARCH: &str = "x86";
#[cfg(target_arch = "x86_64")]
const ARCH: &str = "amd64";
#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";
const MANIFEST_MID: &str = "_microsoft.windows.common-controls_6595b64144ccf1df_";
const MANIFEST_V6: &str = "6.0.2600.2982_none_deadbeef.manifest";
const MANIFEST_V5: &str = "5.82.2600.2982_none_deadbeef.manifest";

/// `ARCH L"_" NAME L"_" PUBLIC_KEY L"_" <version> L"_none_deadbeef.manifest"`, NUL-terminated.
const fn manifest_name<const N: usize>(version: &str) -> [WCHAR; N] {
    let mut out = [0 as WCHAR; N];
    let parts = [ARCH.as_bytes(), MANIFEST_MID.as_bytes(), version.as_bytes()];
    let (mut at, mut p) = (0, 0);
    while p < parts.len() {
        let mut i = 0;
        while i < parts[p].len() {
            out[at] = parts[p][i] as WCHAR;
            at += 1;
            i += 1;
        }
        p += 1;
    }
    core::assert!(at + 1 == N);
    out
}

static manifest_filename: [WCHAR; ARCH.len() + MANIFEST_MID.len() + MANIFEST_V6.len() + 1] =
    manifest_name(MANIFEST_V6);
static manifest_filename_v5: [WCHAR; ARCH.len() + MANIFEST_MID.len() + MANIFEST_V5.len() + 1] =
    manifest_name(MANIFEST_V5);

// #define NO_SHLWAPI_STREAM (no Rust declaration needed)


// MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL) == (SUBLANG_NEUTRAL << 10) | LANG_NEUTRAL == 0

// ---- __REACTOS__ block ----
// ARCH is L"amd64" on x86_64, L"x86" on i386, L"arm64" on aarch64 (wide-literal macro, expanded by the token below)

// ---- !__REACTOS__ block ----
// #define OCM__BASE (WM_USER+0x1c00)  (WM_USER == 0x0400)
pub const OCM__BASE: u32 = 0x2000;

unsafe extern "C" fn GetManifestPath(mut create: BOOL, mut bV6: BOOL) -> *mut WCHAR {
    let mut pwszBuf: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut hres: HRESULT = 0;
    pwszBuf = HeapAlloc(
        GetProcessHeap(),
        0,
        (MAX_PATH as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    if pwszBuf.is_null() {
        return core::ptr::null_mut::<WCHAR>();
    }
    GetWindowsDirectoryW(pwszBuf, MAX_PATH);
    hres = StringCchCatW(pwszBuf, (MAX_PATH) as _, RUSTCONV_WSTR_33A2A8F3E59F.as_ptr());
    if hres < 0 {
        return core::ptr::null_mut::<WCHAR>();
    }
    if create != 0 {
        CreateDirectoryW(pwszBuf, core::ptr::null_mut());
    }
    hres = StringCchCatW(pwszBuf, (MAX_PATH) as _, RUSTCONV_WSTR_35F38A602832.as_ptr());
    if hres < 0 {
        return core::ptr::null_mut::<WCHAR>();
    }
    if create != 0 {
        CreateDirectoryW(pwszBuf, core::ptr::null_mut());
    }

    hres = StringCchCatW(
        pwszBuf,
        (MAX_PATH) as _,
        if bV6 != 0 {
            core::ptr::addr_of!(manifest_filename) as *const u16
        } else {
            core::ptr::addr_of!(manifest_filename_v5) as *const u16
        },
    );
    if hres < 0 {
        return core::ptr::null_mut::<WCHAR>();
    }

    return pwszBuf;
}

unsafe extern "C" fn CreateComctl32ActCtx(bV6: BOOL) -> HANDLE {
    let mut ret: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut pwstrSource: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut ActCtx: ACTCTXW = core::mem::zeroed();
    ActCtx.cbSize = core::mem::size_of::<ACTCTX>() as ULONG;
    pwstrSource = GetManifestPath(FALSE, bV6);
    if pwstrSource.is_null() {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"CreateComctl32ActCtx\0" as *const u8 as *const core::ffi::c_char,
                157 as core::ffi::c_int,
                b"GetManifestPath failed! bV6=%d\n\0" as *const u8
                    as *const core::ffi::c_char,
                bV6,
            );
        }
        return INVALID_HANDLE_VALUE;
    }
    ActCtx.lpSource = pwstrSource as LPCWSTR;
    ret = CreateActCtxW(&raw mut ActCtx as PCACTCTXW);
    if ret == INVALID_HANDLE_VALUE {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"CreateComctl32ActCtx\0" as *const u8 as *const core::ffi::c_char,
                163 as core::ffi::c_int,
                b"CreateActCtxW failed! bV6=%d err=%lu src=%S\n\0" as *const u8
                    as *const core::ffi::c_char,
                bV6,
                GetLastError(),
                pwstrSource,
            );
        }
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, pwstrSource as PVOID);
    return ret;
}

unsafe extern "C" fn RegisterControls(bV6: BOOL) {
    ANIMATE_Register();
    COMBOEX_Register();
    DATETIME_Register();
    FLATSB_Register();
    HEADER_Register();
    HOTKEY_Register();
    IPADDRESS_Register();
    LISTVIEW_Register();
    MONTHCAL_Register();
    NATIVEFONT_Register();
    PAGER_Register();
    PROGRESS_Register();
    REBAR_Register();
    STATUS_Register();
    SYSLINK_Register();
    TAB_Register();
    TOOLTIPS_Register();
    TRACKBAR_Register();
    TREEVIEW_Register();
    UPDOWN_Register();

    if bV6 == 0 {
        TOOLBAR_Register();
    } else {
        BUTTON_Register();
        COMBO_Register();
        COMBOLBOX_Register();
        EDIT_Register();
        LISTBOX_Register();
        STATIC_Register();

        TOOLBARv6_Register();
    }
}

unsafe extern "C" fn UnregisterControls(bV6: BOOL) {
    ANIMATE_Unregister();
    COMBOEX_Unregister();
    DATETIME_Unregister();
    FLATSB_Unregister();
    HEADER_Unregister();
    HOTKEY_Unregister();
    IPADDRESS_Unregister();
    LISTVIEW_Unregister();
    MONTHCAL_Unregister();
    NATIVEFONT_Unregister();
    PAGER_Unregister();
    PROGRESS_Unregister();
    REBAR_Unregister();
    STATUS_Unregister();
    SYSLINK_Unregister();
    TAB_Unregister();
    TOOLTIPS_Unregister();
    TRACKBAR_Unregister();
    TREEVIEW_Unregister();
    UPDOWN_Unregister();

    if bV6 == 0 {
        TOOLBAR_Unregister();
    } else {
        BUTTON_Unregister();
        COMBO_Unregister();
        COMBOLBOX_Unregister();
        EDIT_Unregister();
        LISTBOX_Unregister();
        STATIC_Unregister();
        TOOLBARv6_Unregister();
    }
}

unsafe extern "C" fn InitializeClasses() {
    let mut hActCtx5: HANDLE = core::ptr::null_mut();
    let mut hActCtx6: HANDLE = core::ptr::null_mut();
    let mut activated: BOOL = 0;
    let mut ulCookie: ULONG_PTR = 0;

    /* like comctl32 5.82+ register all the common control classes */
    /* Register the classes once no matter what */
    hActCtx5 = CreateComctl32ActCtx(FALSE);
    activated = if hActCtx5 != INVALID_HANDLE_VALUE {
        ActivateActCtx(hActCtx5, core::ptr::addr_of_mut!(ulCookie))
    } else {
        FALSE
    };
    RegisterControls(FALSE);      /* Register the classes pretending to be v5 */
    if activated != 0 {
        DeactivateActCtx(0, ulCookie);
    }

    hActCtx6 = CreateComctl32ActCtx(TRUE);
    if hActCtx6 != INVALID_HANDLE_VALUE {
        activated = ActivateActCtx(hActCtx6, core::ptr::addr_of_mut!(ulCookie));
        RegisterControls(TRUE);      /* Register the classes pretending to be v6 */
        if activated != 0 {
            DeactivateActCtx(0, ulCookie);
        }

        /* Initialize the themed controls only when the v6 manifest is present */
        THEMING_Initialize(hActCtx5, hActCtx6);
    }
}

unsafe extern "C" fn UninitializeClasses() {
    let mut hActCtx5: HANDLE;
    let mut hActCtx6: HANDLE;
    let mut activated: BOOL;
    let mut ulCookie: ULONG_PTR = 0;

    hActCtx5 = CreateComctl32ActCtx(FALSE);
    activated = if hActCtx5 != INVALID_HANDLE_VALUE {
        ActivateActCtx(hActCtx5, core::ptr::addr_of_mut!(ulCookie))
    } else {
        FALSE
    };
    UnregisterControls(FALSE);
    if activated != 0 {
        DeactivateActCtx(0, ulCookie);
    }

    hActCtx6 = CreateComctl32ActCtx(TRUE);
    if hActCtx6 != INVALID_HANDLE_VALUE {
        activated = ActivateActCtx(hActCtx6, core::ptr::addr_of_mut!(ulCookie));
        THEMING_Uninitialize();
        UnregisterControls(TRUE);
        if activated != 0 {
            DeactivateActCtx(0, ulCookie);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClassNameW(className: LPCWSTR) -> BOOLEAN {
    InitializeClasses();
    return (TRUE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    'sw: {
        if fdwReason == DLL_PROCESS_ATTACH {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);

            COMCTL32_hModule = hinstDLL as HMODULE;

            /* add global subclassing atom (used by 'tooltip' and 'updown') */
            COMCTL32_wSubclass = GlobalAddAtomW(core::ptr::addr_of!(strCC32SubclassInfo) as LPCWSTR) as DWORD_PTR as LPWSTR;

            /* create local pattern brush */
            COMCTL32_hPattern55AABitmap = CreateBitmap(
                8 as INT,
                8 as INT,
                1 as UINT,
                1 as UINT,
                core::ptr::addr_of!(wPattern55AA) as *const core::ffi::c_void,
            );
            COMCTL32_hPattern55AABrush = CreatePatternBrush(COMCTL32_hPattern55AABitmap);

            /* Get all the colors at DLL load */
            COMCTL32_RefreshSysColors();

            // C: `#ifndef __REACTOS__ .. #else` -- only the __REACTOS__ arm is compiled.
                InitializeClasses();


            break 'sw;
        } else if fdwReason == DLL_PROCESS_DETACH {
            if !lpvReserved.is_null() {
                break 'sw;
            }
            // C: `#ifndef __REACTOS__ .. #else` -- only the __REACTOS__ arm is compiled.
            UninitializeClasses();

            /* delete local pattern brush */
            DeleteObject(COMCTL32_hPattern55AABrush as HGDIOBJ);
            DeleteObject(COMCTL32_hPattern55AABitmap as HGDIOBJ);

            /* delete global subclassing atom */
            GlobalDeleteAtom(LOWORD!(COMCTL32_wSubclass as DWORD_PTR) as ATOM);

            break 'sw;
        }
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn MenuHelp(
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    hMainMenu: HMENU,
    hInst: HINSTANCE,
    hwndStatus: HWND,
    lpwIDs: *mut UINT,
) {
    let mut uMenuID: UINT = 0 as UINT;

    if IsWindow(hwndStatus) == 0 {
        return;
    }

    if uMsg == WM_MENUSELECT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MenuHelp\0" as *const u8 as *const ::core::ffi::c_char,
                495 as ::core::ffi::c_int,
                b"WM_MENUSELECT wParam=0x%lX lParam=0x%lX\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                wParam,
                lParam,
            );
        }

        if (HIWORD!(wParam) == 0xFFFF) && (lParam == 0) {
            /* menu was closed */
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/commctrl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"MenuHelp\0" as *const u8 as *const ::core::ffi::c_char,
                    499 as ::core::ffi::c_int,
                    b"menu was closed!\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            SendMessageW(
                hwndStatus,
                SB_SIMPLE as UINT,
                FALSE as WPARAM,
                0 as LPARAM,
            );
        } else {
            /* menu item was selected */
            if (HIWORD!(wParam) as UINT) & MF_POPUP != 0 {
                uMenuID = *lpwIDs.offset(1 as isize);
            } else {
                uMenuID = LOWORD!(wParam) as UINT;
            }

            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/commctrl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"MenuHelp\0" as *const u8 as *const ::core::ffi::c_char,
                    508 as ::core::ffi::c_int,
                    b"uMenuID = %u\n\0" as *const u8 as *const ::core::ffi::c_char,
                    uMenuID,
                );
            }

            if uMenuID != 0 {
                let mut szText: [WCHAR; 256] = [0; 256];

                if LoadStringW(
                    hInst,
                    uMenuID,
                    &raw mut szText as LPWSTR,
                    ARRAY_SIZE!(szText) as ::core::ffi::c_int,
                ) == 0
                {
                    szText[0 as usize] = '\0' as i32 as WCHAR;
                }

                SendMessageW(
                    hwndStatus,
                    SB_SETTEXTW as UINT,
                    (255 | SBT_NOBORDERS) as WPARAM,
                    &raw mut szText as *mut WCHAR as LPARAM,
                );
                SendMessageW(
                    hwndStatus,
                    SB_SIMPLE as UINT,
                    TRUE as WPARAM,
                    0 as LPARAM,
                );
            }
        }
    } else if uMsg == WM_COMMAND {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MenuHelp\0" as *const u8 as *const ::core::ffi::c_char,
                525 as ::core::ffi::c_int,
                b"WM_COMMAND wParam=0x%lX lParam=0x%lX\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                wParam,
                lParam,
            );
        }
        /* WM_COMMAND is not invalid since it is documented
         * in the windows api reference. So don't output
         * any FIXME for WM_COMMAND
         */
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MenuHelp\0" as *const u8 as *const ::core::ffi::c_char,
                530 as ::core::ffi::c_int,
                b"We don't care about the WM_COMMAND\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_4: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_4,
                __dbch_4,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MenuHelp\0" as *const u8 as *const ::core::ffi::c_char,
                534 as ::core::ffi::c_int,
                b"Invalid Message 0x%x!\n\0" as *const u8 as *const ::core::ffi::c_char,
                uMsg,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ShowHideMenuCtl(hwnd: HWND, uFlags: UINT_PTR, lpInfo: LPINT) -> BOOL {
    let mut lpMenuId: LPINT = core::ptr::null_mut();

    if lpInfo.is_null() {
        return FALSE;
    }

    if *lpInfo.offset(0 as isize) == 0 || *lpInfo.offset(1 as isize) == 0 {
        return FALSE;
    }

    /* search for control */
    lpMenuId = lpInfo.offset(2 as isize);
    while *lpMenuId as UINT_PTR != uFlags {
        lpMenuId = lpMenuId.offset(2 as isize);
    }

    if (GetMenuState(
        *lpInfo.offset(1 as isize) as DWORD_PTR as HMENU,
        uFlags as UINT,
        MF_BYCOMMAND as UINT,
    ) & (MFS_CHECKED as UINT)) != 0
    {
        /* uncheck menu item */
        CheckMenuItem(
            *lpInfo.offset(0 as isize) as DWORD_PTR as HMENU,
            *lpMenuId as UINT,
            (MF_BYCOMMAND | MF_UNCHECKED) as UINT,
        );

        /* hide control */
        lpMenuId = lpMenuId.offset(1);
        SetWindowPos(
            GetDlgItem(hwnd, *lpMenuId),
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_HIDEWINDOW,
        );
    } else {
        /* check menu item */
        CheckMenuItem(
            *lpInfo.offset(0 as isize) as DWORD_PTR as HMENU,
            *lpMenuId as UINT,
            (MF_BYCOMMAND | MF_CHECKED) as UINT,
        );

        /* show control */
        lpMenuId = lpMenuId.offset(1);
        SetWindowPos(
            GetDlgItem(hwnd, *lpMenuId),
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_SHOWWINDOW,
        );
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetEffectiveClientRect(hwnd: HWND, lpRect: LPRECT, lpInfo: *const INT) {
    let mut rcCtrl: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut lpRun: *const INT = core::ptr::null::<INT>();
    let mut hwndCtrl: HWND = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/commctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetEffectiveClientRect\0" as *const u8 as *const ::core::ffi::c_char,
            640 as ::core::ffi::c_int,
            b"(%p %p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            lpRect,
            lpInfo,
        );
    }
    GetClientRect(hwnd, lpRect);
    lpRun = lpInfo;

    loop {
        lpRun = lpRun.offset(2 as isize);
        if *lpRun == 0 {
            return;
        }
        lpRun = lpRun.offset(1);
        hwndCtrl = GetDlgItem(hwnd, *lpRun);
        if ((GetWindowLongW(hwndCtrl, GWL_STYLE) as u32) & WS_VISIBLE) != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/commctrl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"GetEffectiveClientRect\0" as *const u8 as *const ::core::ffi::c_char,
                    652 as ::core::ffi::c_int,
                    b"control id 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
                    *lpRun,
                );
            }
            GetWindowRect(hwndCtrl, core::ptr::addr_of_mut!(rcCtrl));
            MapWindowPoints(
                core::ptr::null_mut(),
                hwnd,
                core::ptr::addr_of_mut!(rcCtrl) as LPPOINT,
                2 as UINT,
            );
            SubtractRect(
                lpRect,
                lpRect as LPCRECT,
                core::ptr::addr_of_mut!(rcCtrl) as LPCRECT,
            );
        }
        lpRun = lpRun.offset(1);
        if !(*lpRun != 0) {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DrawStatusTextW(mut hdc: HDC, mut lprc: LPCRECT, mut text: LPCWSTR, mut style: UINT) {
    let mut r: RECT = *lprc;
    let mut border: UINT = BDR_SUNKENOUTER as UINT;
    let mut oldbkcolor: COLORREF = 0;

    if (style & (SBT_POPOUT as UINT)) != 0 {
        border = BDR_RAISEDOUTER as UINT;
    } else if (style & (SBT_NOBORDERS as UINT)) != 0 {
        border = 0 as UINT;
    }

    oldbkcolor = SetBkColor(hdc, comctl32_color.clrBtnFace);
    DrawEdge(
        hdc,
        core::ptr::addr_of_mut!(r),
        border,
        (BF_RECT as UINT) | (BF_ADJUST as UINT),
    );
    if false {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(r),
            border,
            (BF_MIDDLE as UINT) | (BF_RECT as UINT) | (BF_ADJUST as UINT),
        );
    }

    /* now draw text */
    if !text.is_null() {
        let mut oldbkmode: ::core::ffi::c_int = SetBkMode(hdc, TRANSPARENT as ::core::ffi::c_int);
        let mut oldtextcolor: COLORREF = 0;
        let mut align: UINT = DT_LEFT as UINT;
        let mut strCnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

        oldtextcolor = SetTextColor(hdc, comctl32_color.clrBtnText);
        if (style & (SBT_RTLREADING as UINT)) != 0 {
            FIXME!(b"Unsupported RTL style!\n\0".as_ptr() as *const ::core::ffi::c_char);
        }
        r.left += 3 as ::core::ffi::c_int;
        loop {
            if *text == ('\t' as u16) {
                if strCnt != 0 {
                    DrawTextW(
                        hdc,
                        text.offset(-(strCnt as isize)),
                        strCnt,
                        core::ptr::addr_of_mut!(r),
                        align
                            | (DT_VCENTER as UINT)
                            | (DT_SINGLELINE as UINT)
                            | (DT_NOPREFIX as UINT),
                    );
                    strCnt = 0 as ::core::ffi::c_int;
                }
                if align == (DT_RIGHT as UINT) {
                    break;
                }
                align = (if align == (DT_LEFT as UINT) {
                    DT_CENTER
                } else {
                    DT_RIGHT
                }) as UINT;
            } else {
                strCnt += 1;
            }
            let fresh15 = text;
            text = text.offset(1);
            if !(*fresh15 != 0) {
                break;
            }
        }

        if strCnt != 0 {
            DrawTextW(
                hdc,
                text.offset(-(strCnt as isize)),
                -(1 as ::core::ffi::c_int),
                core::ptr::addr_of_mut!(r),
                align
                    | (DT_VCENTER as UINT)
                    | (DT_SINGLELINE as UINT)
                    | (DT_NOPREFIX as UINT),
            );
        }
        SetBkMode(hdc, oldbkmode);
        SetTextColor(hdc, oldtextcolor);
    }

    SetBkColor(hdc, oldbkcolor);
}

#[no_mangle]
pub unsafe extern "system" fn DrawStatusTextA(
    hdc: HDC,
    lprc: LPCRECT,
    text: LPCSTR,
    style: UINT,
) {
    let mut len: INT = 0;
    let mut textW: LPWSTR = core::ptr::null_mut();

    if !text.is_null() {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            text,
            -1,
            core::ptr::null_mut(),
            0,
        ) as INT;
        if len != 0 {
            textW = Alloc(
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ) as LPWSTR;
            if !textW.is_null() {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    text,
                    -1,
                    textW,
                    len,
                );
            }
        }
    }
    DrawStatusTextW(hdc, lprc, textW as LPCWSTR, style);
    Free(textW as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn CreateStatusWindowA(style: LONG, text: LPCSTR, parent: HWND, wid: UINT) -> HWND {
    return CreateWindowA!(STATUSCLASSNAMEA.as_ptr() as LPCSTR, text, style as DWORD,
                         CW_USEDEFAULT, CW_USEDEFAULT,
                         CW_USEDEFAULT, CW_USEDEFAULT,
                         parent, wid as DWORD_PTR as HMENU, core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CreateStatusWindowW(style: LONG, text: LPCWSTR, parent: HWND, wid: UINT) -> HWND {
    return CreateWindowW!(STATUSCLASSNAMEW.as_ptr(), text, style as DWORD,
        CW_USEDEFAULT, CW_USEDEFAULT,
        CW_USEDEFAULT, CW_USEDEFAULT,
        parent, wid as DWORD_PTR as HMENU, core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn CreateUpDownControl(style: DWORD, x: INT, y: INT, cx: INT, cy: INT, parent: HWND, id: INT, inst: HINSTANCE, buddy: HWND, maxVal: INT, minVal: INT, curVal: INT) -> HWND {
    let hUD: HWND = CreateWindowW!(UPDOWN_CLASSW.as_ptr(), core::ptr::null(), style, x, y, cx, cy, parent, id as DWORD_PTR as HMENU, inst, core::ptr::null_mut());
    if !hUD.is_null() {
        SendMessageW(hUD, UDM_SETBUDDY, buddy as WPARAM, 0);
        SendMessageW(hUD, UDM_SETRANGE, 0, (MAKELONG!(maxVal, minVal)) as LPARAM);
        SendMessageW(hUD, UDM_SETPOS, 0, (MAKELONG!(curVal, 0)) as LPARAM);
    }
    return hUD;
}

#[no_mangle]
pub unsafe extern "system" fn InitCommonControls() {}

#[no_mangle]
pub unsafe extern "system" fn InitCommonControlsEx(lpInitCtrls: *const INITCOMMONCONTROLSEX) -> BOOL {
    if lpInitCtrls.is_null() || (*lpInitCtrls).dwSize as usize != core::mem::size_of::<INITCOMMONCONTROLSEX>() {
        return FALSE;
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn CreateToolbarEx(
    hwnd: HWND,
    style: DWORD,
    wID: UINT,
    nBitmaps: INT,
    hBMInst: HINSTANCE,
    wBMID: UINT_PTR,
    lpButtons: LPCTBBUTTON,
    iNumButtons: INT,
    mut dxButton: INT,
    mut dyButton: INT,
    mut dxBitmap: INT,
    mut dyBitmap: INT,
    uStructSize: UINT,
) -> HWND {
    let mut hwndTB: HWND;

    hwndTB = CreateWindowExW(
        0,
        (TOOLBARCLASSNAMEW).as_ptr(),
        core::ptr::null(),
        style | WS_CHILD,
        0,
        0,
        100,
        30,
        hwnd,
        wID as DWORD_PTR as HMENU,
        COMCTL32_hModule,
        core::ptr::null_mut(),
    );
    if !hwndTB.is_null() {
        let mut tbab: TBADDBITMAP = tagTBADDBITMAP {
            hInst: core::ptr::null_mut(),
            nID: 0,
        };

        SendMessageW(hwndTB, TB_BUTTONSTRUCTSIZE, uStructSize as WPARAM, 0 as LPARAM);

        /* set bitmap and button size */
        /*If CreateToolbarEx receives 0, windows sets default values*/
        if dxBitmap < 0 {
            dxBitmap = 16;
        }
        if dyBitmap < 0 {
            dyBitmap = 16;
        }
        if dxBitmap == 0 || dyBitmap == 0 {
            dyBitmap = 16;
            dxBitmap = dyBitmap;
        }
        SendMessageW(
            hwndTB,
            TB_SETBITMAPSIZE,
            0 as WPARAM,
            MAKELPARAM!(dxBitmap, dyBitmap),
        );

        if dxButton < 0 {
            dxButton = dxBitmap;
        }
        if dyButton < 0 {
            dyButton = dyBitmap;
        }
        /* TB_SETBUTTONSIZE -> TB_SETBITMAPSIZE bug introduced for Windows compatibility */
        if dxButton != 0 && dyButton != 0 {
            SendMessageW(
                hwndTB,
                TB_SETBITMAPSIZE,
                0 as WPARAM,
                MAKELPARAM!(dxButton, dyButton),
            );
        }

        /* add bitmaps */
        if nBitmaps > 0 || hBMInst == HINST_COMMCTRL {
            tbab.hInst = hBMInst;
            tbab.nID = wBMID;

            SendMessageW(
                hwndTB,
                TB_ADDBITMAP,
                nBitmaps as WPARAM,
                &mut tbab as *mut TBADDBITMAP as LPARAM,
            );
        }
        /* add buttons */
        if iNumButtons > 0 {
            SendMessageW(
                hwndTB,
                TB_ADDBUTTONSW,
                iNumButtons as WPARAM,
                lpButtons as LPARAM,
            );
        }
    }

    return hwndTB;
}

#[no_mangle]
pub unsafe extern "system" fn CreateMappedBitmap(
    hInstance: HINSTANCE,
    idBitmap: INT_PTR,
    wFlags: UINT,
    lpColorMap: LPCOLORMAP,
    iNumMaps: INT,
) -> HBITMAP {
    let mut hglb: HGLOBAL = core::ptr::null_mut();
    let mut hRsrc: HRSRC = core::ptr::null_mut();
    let mut lpBitmap: *const BITMAPINFOHEADER = core::ptr::null();
    let mut lpBitmapInfo: LPBITMAPINFOHEADER = core::ptr::null_mut();
    let mut nSize: UINT = 0;
    let mut nColorTableSize: UINT = 0;
    let mut iColor: UINT = 0;
    let mut pColorTable: *mut RGBQUAD = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut iMaps: INT = 0;
    let mut nWidth: INT = 0;
    let mut nHeight: INT = 0;
    let mut hdcScreen: HDC = core::ptr::null_mut();
    let mut hbm: HBITMAP = core::ptr::null_mut();
    let mut sysColorMap: LPCOLORMAP = core::ptr::null_mut();
    let mut cRef: COLORREF = 0;
    let mut internalColorMap: [COLORMAP; 4] = [
        COLORMAP {
            from: 0x000000 as COLORREF,
            to: 0 as COLORREF,
        },
        COLORMAP {
            from: 0x808080 as COLORREF,
            to: 0 as COLORREF,
        },
        COLORMAP {
            from: 0xC0C0C0 as COLORREF,
            to: 0 as COLORREF,
        },
        COLORMAP {
            from: 0xFFFFFF as COLORREF,
            to: 0 as COLORREF,
        },
    ];

    /* initialize pointer to colortable and default color table */
    if !lpColorMap.is_null() {
        iMaps = iNumMaps;
        sysColorMap = lpColorMap;
    } else {
        internalColorMap[0].to =
            GetSysColor((COLOR_BTNTEXT) as i32) as COLORREF;
        internalColorMap[1].to =
            GetSysColor((COLOR_BTNSHADOW) as i32) as COLORREF;
        internalColorMap[2].to =
            GetSysColor((COLOR_BTNFACE) as i32) as COLORREF;
        internalColorMap[3].to =
            GetSysColor((COLOR_BTNHIGHLIGHT) as i32) as COLORREF;
        iMaps = 4 as INT;
        sysColorMap = core::ptr::addr_of_mut!(internalColorMap) as *mut COLORMAP as LPCOLORMAP;
    }

    hRsrc = FindResourceW(
        hInstance,
        idBitmap as LPWSTR,
        RT_BITMAP as LPWSTR,
    );
    if hRsrc == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }
    hglb = LoadResource(hInstance, hRsrc);
    if hglb == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }
    lpBitmap = LockResource(hglb) as *const BITMAPINFOHEADER;
    if lpBitmap.is_null() {
        return core::ptr::null_mut();
    }

    if (*lpBitmap).biSize as usize >= core::mem::size_of::<BITMAPINFOHEADER>()
        && (*lpBitmap).biClrUsed != 0
    {
        nColorTableSize = (*lpBitmap).biClrUsed as UINT;
    } else if ((*lpBitmap).biBitCount as INT) <= 8 {
        nColorTableSize =
            ((1 as INT) << (*lpBitmap).biBitCount) as UINT;
    } else {
        nColorTableSize = 0 as UINT;
    }
    nSize = (*lpBitmap).biSize as UINT;
    if nSize as usize == core::mem::size_of::<BITMAPINFOHEADER>()
        && (*lpBitmap).biCompression == BI_BITFIELDS as DWORD
    {
        nSize = nSize.wrapping_add(
            (3 as UINT).wrapping_mul(core::mem::size_of::<DWORD>() as UINT),
        );
    }
    nSize = nSize.wrapping_add(
        nColorTableSize.wrapping_mul(core::mem::size_of::<RGBQUAD>() as UINT),
    );
    lpBitmapInfo = GlobalAlloc(GMEM_FIXED as UINT, nSize as SIZE_T)
        as LPBITMAPINFOHEADER;
    if lpBitmapInfo.is_null() {
        return core::ptr::null_mut();
    }
    RtlMoveMemory!(
        lpBitmapInfo as *mut core::ffi::c_void,
        lpBitmap as *const core::ffi::c_void,
        nSize as usize,
    );

    pColorTable = (lpBitmapInfo as LPBYTE).offset((*lpBitmapInfo).biSize as isize) as *mut RGBQUAD;

    iColor = 0 as UINT;
    while iColor < nColorTableSize {
        i = 0 as INT;
        while i < iMaps {
            cRef = RGB!(
                (*pColorTable.offset(iColor as isize)).rgbRed,
                (*pColorTable.offset(iColor as isize)).rgbGreen,
                (*pColorTable.offset(iColor as isize)).rgbBlue,
            );
            if cRef == (*sysColorMap.offset(i as isize)).from {
                (*pColorTable.offset(iColor as isize)).rgbBlue =
                    GetBValue!((*sysColorMap.offset(i as isize)).to);
                (*pColorTable.offset(iColor as isize)).rgbGreen =
                    GetGValue!((*sysColorMap.offset(i as isize)).to);
                (*pColorTable.offset(iColor as isize)).rgbRed =
                    GetRValue!((*sysColorMap.offset(i as isize)).to);
                break;
            } else {
                i += 1;
            }
        }
        iColor = iColor.wrapping_add(1);
    }
    nWidth = (*lpBitmapInfo).biWidth as INT;
    nHeight = (*lpBitmapInfo).biHeight as INT;
    hdcScreen = GetDC(core::ptr::null_mut());
    hbm = CreateCompatibleBitmap(hdcScreen, nWidth, nHeight);
    if !hbm.is_null() {
        let hdcDst: HDC = CreateCompatibleDC(hdcScreen);
        let hbmOld: HBITMAP =
            SelectObject(hdcDst, hbm as HGDIOBJ) as HBITMAP;
        let lpBits: *const BYTE = (lpBitmap as *const BYTE).offset(nSize as isize);
        StretchDIBits(
            hdcDst,
            0 as INT,
            0 as INT,
            nWidth,
            nHeight,
            0 as INT,
            0 as INT,
            nWidth,
            nHeight,
            lpBits as *const core::ffi::c_void,
            lpBitmapInfo as LPBITMAPINFO as *const BITMAPINFO,
            DIB_RGB_COLORS as UINT,
            SRCCOPY as DWORD,
        );
        SelectObject(hdcDst, hbmOld as HGDIOBJ);
        DeleteDC(hdcDst);
    }
    ReleaseDC(core::ptr::null_mut(), hdcScreen);
    GlobalFree(lpBitmapInfo as HGLOBAL);
    FreeResource(hglb);

    return hbm;
}

#[no_mangle]
pub unsafe extern "system" fn CreateToolbar(hwnd: HWND, style: DWORD, wID: UINT, nBitmaps: INT, hBMInst: HINSTANCE, wBMID: UINT, lpButtons: LPCTBBUTTON, iNumButtons: INT) -> HWND {
    return CreateToolbarEx(hwnd, style | (CCS_NODIVIDER as DWORD), wID, nBitmaps,
                            hBMInst, wBMID as UINT_PTR, lpButtons,
                            iNumButtons, (0) as INT, (0) as INT, (0) as INT, (0) as INT, CCSIZEOF_STRUCT!(TBBUTTON, dwData) as UINT);
}

#[no_mangle]
pub unsafe extern "system" fn DllGetVersion(pdvi: *mut DLLVERSIONINFO) -> HRESULT {
    if (*pdvi).cbSize as usize != ::core::mem::size_of::<DLLVERSIONINFO>() as usize {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DllGetVersion\0" as *const u8 as *const ::core::ffi::c_char,
                1166 as ::core::ffi::c_int,
                b"wrong DLLVERSIONINFO size from app\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return E_INVALIDARG;
    }
    (*pdvi).dwMajorVersion = COMCTL32_VERSION as DWORD;
    (*pdvi).dwMinorVersion = COMCTL32_VERSION_MINOR as DWORD;
    (*pdvi).dwBuildNumber = 2919 as DWORD;
    (*pdvi).dwPlatformID = 6304 as DWORD;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/commctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DllGetVersion\0" as *const u8 as *const ::core::ffi::c_char,
            1177 as ::core::ffi::c_int,
            b"%u.%u.%u.%u\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*pdvi).dwMajorVersion,
            (*pdvi).dwMinorVersion,
            (*pdvi).dwBuildNumber,
            (*pdvi).dwPlatformID,
        );
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn DllInstall(bInstall: BOOL, cmdline: LPCWSTR) -> HRESULT {
    // TRACE("(%u, %s): stub\n", bInstall, debugstr_w(cmdline));
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn _TrackMouseEvent(ptme: *mut TRACKMOUSEEVENT) -> BOOL {
    TrackMouseEvent(ptme)
}

#[no_mangle]
pub unsafe extern "system" fn GetMUILanguage() -> LANGID {
    return COMCTL32_uiLang;
}

#[no_mangle]
pub unsafe extern "system" fn InitMUILanguage(uiLang: LANGID) {
    COMCTL32_uiLang = uiLang;
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn SetWindowSubclass(
    mut hWnd: HWND,
    mut pfnSubclass: SUBCLASSPROC,
    mut uIDSubclass: UINT_PTR,
    mut dwRef: DWORD_PTR,
) -> BOOL {
    let mut stack: LPSUBCLASS_INFO = core::ptr::null_mut::<SUBCLASS_INFO>();
    let mut proc_0: LPSUBCLASSPROCS = core::ptr::null_mut::<_SUBCLASSPROCS>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/commctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetWindowSubclass\0" as *const u8 as *const ::core::ffi::c_char,
            1282 as ::core::ffi::c_int,
            b"(%p, %p, %lx, %lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hWnd,
            pfnSubclass,
            uIDSubclass,
            dwRef,
        );
    }
    if hWnd.is_null() || pfnSubclass.is_none() {
        return FALSE;
    }
    stack = GetPropW(hWnd, COMCTL32_wSubclass as LPCWSTR) as LPSUBCLASS_INFO;
    if stack.is_null() {
        stack = Alloc(core::mem::size_of::<SUBCLASS_INFO>() as DWORD)
            as LPSUBCLASS_INFO;
        if stack.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/commctrl.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"SetWindowSubclass\0" as *const u8 as *const ::core::ffi::c_char,
                    1298 as ::core::ffi::c_int,
                    b"Failed to allocate our Subclassing stack\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return FALSE;
        }
        SetPropW(hWnd, COMCTL32_wSubclass as LPCWSTR, stack as HANDLE);
        if IsWindowUnicode(hWnd) != 0 {
            (*stack).origproc = ::core::mem::transmute::<isize, WNDPROC>(
                SetWindowLongPtrW(
                    hWnd,
                    GWLP_WNDPROC,
                    ::core::mem::transmute::<
                        Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
                        DWORD_PTR,
                    >(Some(
                        COMCTL32_SubclassProc,
                    )) as LONG_PTR,
                ) as isize,
            );
        } else {
            (*stack).origproc = ::core::mem::transmute::<isize, WNDPROC>(
                SetWindowLongPtrA(
                    hWnd,
                    GWLP_WNDPROC,
                    ::core::mem::transmute::<
                        Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
                        DWORD_PTR,
                    >(Some(
                        COMCTL32_SubclassProc,
                    )) as LONG_PTR,
                ) as isize,
            );
        }
    } else {
        proc_0 = (*stack).SubclassProcs as LPSUBCLASSPROCS;
        while !proc_0.is_null() {
            if (*proc_0).id == uIDSubclass && (*proc_0).subproc == pfnSubclass {
                (*proc_0).ref_ = dwRef;
                return TRUE;
            }
            proc_0 = (*proc_0).next as LPSUBCLASSPROCS;
        }
    }
    proc_0 = Alloc(core::mem::size_of::<SUBCLASSPROCS>() as DWORD)
        as LPSUBCLASSPROCS;
    if proc_0.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SetWindowSubclass\0" as *const u8 as *const ::core::ffi::c_char,
                1327 as ::core::ffi::c_int,
                b"Failed to allocate subclass entry in stack\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        if IsWindowUnicode(hWnd) != 0 {
            SetWindowLongPtrW(
                hWnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*stack).origproc) as LONG_PTR,
            );
        } else {
            SetWindowLongPtrA(
                hWnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*stack).origproc) as LONG_PTR,
            );
        }
        Free(stack as LPVOID);
        RemovePropW(hWnd, COMCTL32_wSubclass as LPCWSTR);
        return FALSE;
    }
    (*proc_0).subproc = pfnSubclass;
    (*proc_0).ref_ = dwRef;
    (*proc_0).id = uIDSubclass;
    (*proc_0).next = (*stack).SubclassProcs as *mut _SUBCLASSPROCS;
    (*stack).SubclassProcs = proc_0 as *mut SUBCLASSPROCS;
    return TRUE;
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn GetWindowSubclass(
    mut hWnd: HWND,
    mut pfnSubclass: SUBCLASSPROC,
    mut uID: UINT_PTR,
    mut pdwRef: *mut DWORD_PTR,
) -> BOOL {
    let mut stack: *const SUBCLASS_INFO = ::core::ptr::null::<SUBCLASS_INFO>();
    let mut proc_0: *const SUBCLASSPROCS = ::core::ptr::null::<SUBCLASSPROCS>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/commctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetWindowSubclass\0" as *const u8 as *const ::core::ffi::c_char,
            1369 as ::core::ffi::c_int,
            b"(%p, %p, %lx, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hWnd,
            pfnSubclass,
            uID,
            pdwRef,
        );
    }
    stack = GetPropW(hWnd, COMCTL32_wSubclass as LPCWSTR) as *const SUBCLASS_INFO;
    if stack.is_null() {
        return FALSE;
    }
    proc_0 = (*stack).SubclassProcs;
    while !proc_0.is_null() {
        if (*proc_0).id == uID && (*proc_0).subproc == pfnSubclass {
            *pdwRef = (*proc_0).ref_;
            return TRUE;
        }
        proc_0 = (*proc_0).next;
    }
    return FALSE;
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn RemoveWindowSubclass(hWnd: HWND, pfnSubclass: SUBCLASSPROC, uID: UINT_PTR) -> BOOL {
    let mut stack: LPSUBCLASS_INFO = core::ptr::null_mut();
    let mut prevproc: LPSUBCLASSPROCS = core::ptr::null_mut();
    let mut proc_0: LPSUBCLASSPROCS;
    let mut ret: BOOL = FALSE;

    stack = GetPropW(hWnd, COMCTL32_wSubclass as LPCWSTR) as LPSUBCLASS_INFO;
    if stack.is_null() {
        return FALSE;
    }
    proc_0 = (*stack).SubclassProcs as LPSUBCLASSPROCS;
    while !proc_0.is_null() {
        if (*proc_0).id == uID && (*proc_0).subproc == pfnSubclass {
            if prevproc.is_null() {
                (*stack).SubclassProcs = (*proc_0).next as *mut SUBCLASSPROCS;
            } else {
                (*prevproc).next = (*proc_0).next;
            }
            if (*stack).stackpos == proc_0 {
                (*stack).stackpos = (*(*stack).stackpos).next as *mut SUBCLASSPROCS;
            }
            Free(proc_0 as LPVOID);
            ret = TRUE;
            break;
        }
        prevproc = proc_0;
        proc_0 = (*proc_0).next as LPSUBCLASSPROCS;
    }

    if (*stack).SubclassProcs.is_null() && (*stack).running == 0 {
        if IsWindowUnicode(hWnd) != 0 {
            SetWindowLongPtrW(
                hWnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*stack).origproc) as LONG_PTR,
            );
        } else {
            SetWindowLongPtrA(
                hWnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*stack).origproc) as LONG_PTR,
            );
        }
        Free(stack as LPVOID);
        RemovePropW(hWnd, COMCTL32_wSubclass as LPCWSTR);
    }

    return ret;
}
unsafe extern "system" fn COMCTL32_SubclassProc(
    mut hWnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut stack: LPSUBCLASS_INFO = ::core::ptr::null_mut::<SUBCLASS_INFO>();
    let mut proc_0: LPSUBCLASSPROCS = ::core::ptr::null_mut::<_SUBCLASSPROCS>();
    let mut ret: LRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/commctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"COMCTL32_SubclassProc\0" as *const u8 as *const ::core::ffi::c_char,
            1466 as ::core::ffi::c_int,
            b"(%p, 0x%08x, 0x%08lx, 0x%08lx)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hWnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    stack = GetPropW(hWnd, COMCTL32_wSubclass as LPCWSTR) as LPSUBCLASS_INFO;
    if stack.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMCTL32_SubclassProc\0" as *const u8 as *const ::core::ffi::c_char,
                1470 as ::core::ffi::c_int,
                b"Our sub classing stack got erased for %p!! Nothing we can do\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                hWnd,
            );
        }
        return 0;
    }
    proc_0 = (*stack).stackpos as LPSUBCLASSPROCS;
    (*stack).stackpos = (*stack).SubclassProcs;
    (*stack).running += 1;
    ret = DefSubclassProc(hWnd, uMsg, wParam, lParam);
    (*stack).running -= 1;
    (*stack).stackpos = proc_0 as *mut SUBCLASSPROCS;
    if (*stack).SubclassProcs.is_null() && (*stack).running == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/commctrl.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"COMCTL32_SubclassProc\0" as *const u8 as *const ::core::ffi::c_char,
                1483 as ::core::ffi::c_int,
                b"Last Subclass removed, cleaning up\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        if IsWindowUnicode(hWnd) != 0 {
            SetWindowLongPtrW(
                hWnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*stack).origproc) as LONG_PTR,
            );
        } else {
            SetWindowLongPtrA(
                hWnd,
                GWLP_WNDPROC,
                ::core::mem::transmute::<WNDPROC, DWORD_PTR>((*stack).origproc) as LONG_PTR,
            );
        }
        Free(stack as LPVOID);
        RemovePropW(hWnd, COMCTL32_wSubclass as LPCWSTR);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn DefSubclassProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut stack: LPSUBCLASS_INFO = core::ptr::null_mut::<SUBCLASS_INFO>();
    let mut ret: LRESULT = 0;

    /* retrieve our little stack from the Properties */
    stack = GetPropW(hWnd, COMCTL32_wSubclass as LPCWSTR) as LPSUBCLASS_INFO;
    if stack.is_null() {
        return 0;
    }

    /* If we are at the end of stack then we have to call the original
     * window procedure */
    if (*stack).stackpos.is_null() {
        if IsWindowUnicode(hWnd) != 0 {
            ret = CallWindowProcW((*stack).origproc, hWnd, uMsg, wParam, lParam);
        } else {
            ret = CallWindowProcA((*stack).origproc, hWnd, uMsg, wParam, lParam);
        }
    } else {
        let mut proc_0: *const SUBCLASSPROCS = (*stack).stackpos;
        (*stack).stackpos = (*(*stack).stackpos).next as *mut SUBCLASSPROCS;
        /* call the Subclass procedure from the stack */
        ret = (*proc_0).subproc.unwrap_unchecked()(
            hWnd,
            uMsg,
            wParam,
            lParam,
            (*proc_0).id,
            (*proc_0).ref_,
        );
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn COMCTL32_CreateToolTip(hwndOwner: HWND) -> HWND {
    let mut hwndToolTip: HWND = core::ptr::null_mut();

    hwndToolTip = CreateWindowExW(
        0 as DWORD,
        (TOOLTIPS_CLASSW).as_ptr() as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        WS_POPUP as DWORD,
        CW_USEDEFAULT as i32,
        CW_USEDEFAULT as i32,
        CW_USEDEFAULT as i32,
        CW_USEDEFAULT as i32,
        hwndOwner,
        0 as HMENU,
        0 as HINSTANCE,
        0 as LPVOID,
    );

    /* Send NM_TOOLTIPSCREATED notification */
    if !hwndToolTip.is_null() {
        let mut nmttc: NMTOOLTIPSCREATED = core::mem::zeroed();
        /* true owner can be different if hwndOwner is a child window */
        let mut hwndTrueOwner: HWND = GetWindow(hwndToolTip, GW_OWNER as UINT);
        nmttc.hdr.hwndFrom = hwndTrueOwner;
        nmttc.hdr.idFrom = GetWindowLongPtrW(hwndTrueOwner, GWLP_ID) as UINT_PTR;
        nmttc.hdr.code = NM_TOOLTIPSCREATED as UINT;
        nmttc.hwndToolTips = hwndToolTip;

        SendMessageW(
            GetParent(hwndTrueOwner),
            WM_NOTIFY as UINT,
            GetWindowLongPtrW(hwndTrueOwner, GWLP_ID) as WPARAM,
            &raw mut nmttc as LPARAM,
        );
    }

    return hwndToolTip;
}

#[no_mangle]
pub unsafe extern "C" fn COMCTL32_RefreshSysColors() {
    comctl32_color.clrBtnHighlight = GetSysColor((COLOR_BTNHIGHLIGHT) as i32) as COLORREF;
    comctl32_color.clrBtnShadow = GetSysColor((COLOR_BTNSHADOW) as i32) as COLORREF;
    comctl32_color.clrBtnText = GetSysColor((COLOR_BTNTEXT) as i32) as COLORREF;
    comctl32_color.clrBtnFace = GetSysColor((COLOR_BTNFACE) as i32) as COLORREF;
    comctl32_color.clrHighlight = GetSysColor((COLOR_HIGHLIGHT) as i32) as COLORREF;
    comctl32_color.clrHighlightText = GetSysColor((COLOR_HIGHLIGHTTEXT) as i32) as COLORREF;
    comctl32_color.clrHotTrackingColor = GetSysColor((COLOR_HOTLIGHT) as i32) as COLORREF;
    comctl32_color.clr3dHilight = GetSysColor((COLOR_3DHILIGHT) as i32) as COLORREF;
    comctl32_color.clr3dShadow = GetSysColor((COLOR_3DSHADOW) as i32) as COLORREF;
    comctl32_color.clr3dDkShadow = GetSysColor((COLOR_3DDKSHADOW) as i32) as COLORREF;
    comctl32_color.clr3dFace = GetSysColor((COLOR_3DFACE) as i32) as COLORREF;
    comctl32_color.clrWindow = GetSysColor((COLOR_WINDOW) as i32) as COLORREF;
    comctl32_color.clrWindowText = GetSysColor((COLOR_WINDOWTEXT) as i32) as COLORREF;
    comctl32_color.clrGrayText = GetSysColor((COLOR_GRAYTEXT) as i32) as COLORREF;
    comctl32_color.clrActiveCaption = GetSysColor((COLOR_ACTIVECAPTION) as i32) as COLORREF;
    comctl32_color.clrInfoBk = GetSysColor((COLOR_INFOBK) as i32) as COLORREF;
    comctl32_color.clrInfoText = GetSysColor((COLOR_INFOTEXT) as i32) as COLORREF;
}
#[no_mangle]
pub unsafe extern "C" fn COMCTL32_DrawInsertMark(hDC: HDC, lpRect: *const RECT, clrInsertMark: COLORREF, bHorizontal: BOOL) {
    let hPen: HPEN = CreatePen((PS_SOLID as i32), 1, clrInsertMark);
    let mut hOldPen: HPEN;
    static mut adwPolyPoints: [DWORD; 3] = [4, 4, 4];
    let lCentre: LONG = if bHorizontal != 0 {
        (*lpRect).top + ((*lpRect).bottom - (*lpRect).top) / 2
    } else {
        (*lpRect).left + ((*lpRect).right - (*lpRect).left) / 2
    };
    let l1: LONG = if bHorizontal != 0 { (*lpRect).left } else { (*lpRect).top };
    let l2: LONG = if bHorizontal != 0 { (*lpRect).right } else { (*lpRect).bottom };
    let aptInsertMark: [POINT; 12] = [
        /* top (V) or left (H) arrow */
        POINT { x: lCentre, y: l1 + 2 },
        POINT { x: lCentre - 2, y: l1 },
        POINT { x: lCentre + 3, y: l1 },
        POINT { x: lCentre + 1, y: l1 + 2 },
        /* middle line */
        POINT { x: lCentre, y: l2 - 2 },
        POINT { x: lCentre, y: l1 - 1 },
        POINT { x: lCentre + 1, y: l1 - 1 },
        POINT { x: lCentre + 1, y: l2 - 2 },
        /* bottom (V) or right (H) arrow */
        POINT { x: lCentre, y: l2 - 3 },
        POINT { x: lCentre - 2, y: l2 - 1 },
        POINT { x: lCentre + 3, y: l2 - 1 },
        POINT { x: lCentre + 1, y: l2 - 3 },
    ];
    hOldPen = SelectObject(hDC, hPen as HGDIOBJ) as HPEN;
    PolyPolyline(
        hDC,
        core::ptr::addr_of!(aptInsertMark) as *const POINT,
        core::ptr::addr_of!(adwPolyPoints) as *const DWORD,
        ARRAY_SIZE!(adwPolyPoints),
    );
    SelectObject(hDC, hOldPen as HGDIOBJ);
    DeleteObject(hPen as HGDIOBJ);
}
#[no_mangle]
pub unsafe extern "C" fn COMCTL32_EnsureBitmapSize(
    pBitmap: *mut HBITMAP,
    cxMinWidth: i32,
    cyMinHeight: i32,
    crBackground: COLORREF,
) {
    let mut cxNew: i32;
    let mut cyNew: i32;
    let mut bmp: BITMAP = core::mem::zeroed();
    let mut hNewBitmap: HBITMAP;
    let mut hNewDCBitmap: HBITMAP;
    let mut hOldDCBitmap: HBITMAP;
    let mut hNewDCBrush: HBRUSH;
    let mut hdcNew: HDC;
    let mut hdcOld: HDC;

    if GetObjectW(
        *pBitmap as HANDLE,
        core::mem::size_of::<BITMAP>() as i32,
        core::ptr::addr_of_mut!(bmp) as LPVOID,
    ) == 0
    {
        return;
    }
    cxNew = if cxMinWidth > bmp.bmWidth { cxMinWidth } else { bmp.bmWidth as i32 };
    cyNew = if cyMinHeight > bmp.bmHeight { cyMinHeight } else { bmp.bmHeight as i32 };
    if cxNew == bmp.bmWidth && cyNew == bmp.bmHeight {
        return;
    }

    hdcNew = CreateCompatibleDC(core::ptr::null_mut());
    hNewBitmap = CreateBitmap(
        cxNew as INT,
        cyNew as INT,
        bmp.bmPlanes as UINT,
        bmp.bmBitsPixel as UINT,
        core::ptr::null(),
    );
    hNewDCBitmap = SelectObject((hdcNew as HGDIOBJ).cast(), hNewBitmap as HGDIOBJ) as HBITMAP;
    hNewDCBrush = SelectObject(
        (hdcNew as HGDIOBJ).cast(),
        CreateSolidBrush(crBackground) as HGDIOBJ,
    ) as HBRUSH;

    hdcOld = CreateCompatibleDC(core::ptr::null_mut());
    hOldDCBitmap = SelectObject((hdcOld as HGDIOBJ).cast(), *pBitmap as HGDIOBJ) as HBITMAP;

    BitBlt(
        hdcNew,
        0 as INT,
        0 as INT,
        bmp.bmWidth as INT,
        bmp.bmHeight as INT,
        hdcOld,
        0 as INT,
        0 as INT,
        SRCCOPY as DWORD,
    );
    if bmp.bmWidth < cxMinWidth {
        PatBlt(
            hdcNew,
            bmp.bmWidth as i32,
            0 as i32,
            cxNew,
            bmp.bmHeight as i32,
            PATCOPY as DWORD,
        );
    }
    if bmp.bmHeight < cyMinHeight {
        PatBlt(
            hdcNew,
            0 as i32,
            bmp.bmHeight as i32,
            bmp.bmWidth as i32,
            cyNew,
            PATCOPY as DWORD,
        );
    }
    if bmp.bmWidth < cxMinWidth && bmp.bmHeight < cyMinHeight {
        PatBlt(
            hdcNew,
            bmp.bmWidth as i32,
            bmp.bmHeight as i32,
            cxNew,
            cyNew,
            PATCOPY as DWORD,
        );
    }

    SelectObject((hdcNew as HGDIOBJ).cast(), hNewDCBitmap as HGDIOBJ);
    DeleteObject(SelectObject((hdcNew as HGDIOBJ).cast(), hNewDCBrush as HGDIOBJ));
    DeleteDC(hdcNew);
    SelectObject((hdcOld as HGDIOBJ).cast(), hOldDCBitmap as HGDIOBJ);
    DeleteDC(hdcOld);

    DeleteObject(*pBitmap as HGDIOBJ);
    *pBitmap = hNewBitmap;
    return;
}
#[no_mangle]
pub unsafe extern "C" fn COMCTL32_GetFontMetrics(mut hFont: HFONT, mut ptm: *mut TEXTMETRICW) {
    let mut hdc: HDC = GetDC(core::ptr::null_mut::<HWND__>());
    let mut hOldFont: HFONT = core::ptr::null_mut::<HFONT__>();
    hOldFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
    GetTextMetricsW(hdc, ptm);
    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC(core::ptr::null_mut::<HWND__>(), hdc);
}
#[no_mangle]
pub unsafe extern "C" fn COMCTL32_IsReflectedMessage(uMsg: UINT) -> BOOL {
    if uMsg == OCM__BASE + WM_COMMAND
        || uMsg == OCM__BASE + WM_CTLCOLORBTN
        || uMsg == OCM__BASE + WM_CTLCOLOREDIT
        || uMsg == OCM__BASE + WM_CTLCOLORDLG
        || uMsg == OCM__BASE + WM_CTLCOLORLISTBOX
        || uMsg == OCM__BASE + WM_CTLCOLORMSGBOX
        || uMsg == OCM__BASE + WM_CTLCOLORSCROLLBAR
        || uMsg == OCM__BASE + WM_CTLCOLORSTATIC
        || uMsg == OCM__BASE + WM_DRAWITEM
        || uMsg == OCM__BASE + WM_MEASUREITEM
        || uMsg == OCM__BASE + WM_DELETEITEM
        || uMsg == OCM__BASE + WM_VKEYTOITEM
        || uMsg == OCM__BASE + WM_CHARTOITEM
        || uMsg == OCM__BASE + WM_COMPAREITEM
        || uMsg == OCM__BASE + WM_HSCROLL
        || uMsg == OCM__BASE + WM_VSCROLL
        || uMsg == OCM__BASE + WM_PARENTNOTIFY
        || uMsg == OCM__BASE + WM_NOTIFY
    {
        return TRUE;
    }
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn MirrorIcon(phicon1: *mut HICON, phicon2: *mut HICON) -> BOOL {
    // FIXME("(%p, %p): stub\n", phicon1, phicon2);
    FALSE
}

unsafe extern "C" fn IsDelimiter(c: WCHAR) -> BOOL {
    match c {
        0x2F | 0x5C | 0x2E | 0x20 => return TRUE, // '/' '\\' '.' ' '
        _ => {}
    }
    return FALSE;
}

unsafe extern "system" fn PathWordBreakProc(lpch: LPCWSTR, mut ichCurrent: i32, cch: i32, code: i32) -> i32 {
    if (code) as u32 == WB_ISDELIMITER {
        return IsDelimiter(*lpch.offset(ichCurrent as isize)) as i32;
    } else {
        let mut dir: i32 = if (code) as u32 == WB_LEFT {
            -1
        } else {
            1
        };
        while 0 <= ichCurrent && ichCurrent < cch {
            if IsDelimiter(*lpch.offset(ichCurrent as isize)) != 0 {
                return ichCurrent;
            }
            ichCurrent += dir;
        }
    }
    return ichCurrent;
}

#[no_mangle]
pub unsafe extern "system" fn SetPathWordBreakProc(hwnd: HWND, bSet: BOOL) -> LRESULT {
    return SendMessageW(hwnd, EM_SETWORDBREAKPROC, 0,
        (if bSet != 0 { PathWordBreakProc as usize as LPARAM } else { 0 }));
}

#[no_mangle]
pub unsafe extern "system" fn DrawShadowText(
    hdc: HDC,
    pszText: LPCWSTR,
    cch: UINT,
    prc: *mut RECT,
    dwFlags: DWORD,
    crText: COLORREF,
    crShadow: COLORREF,
    mut ixOffset: c_int,
    mut iyOffset: c_int,
) -> c_int {
    let mut crOldText: COLORREF;
    let mut rcText: RECT = core::mem::zeroed();
    let mut iRet: INT;
    let mut x: INT;
    let mut y: INT;
    let mut x2: INT;
    let mut y2: INT;
    let mut pBits: *mut BYTE = core::ptr::null_mut();
    let mut hbm: HBITMAP;
    let mut hbmOld: HBITMAP;
    let mut bi: BITMAPINFO = core::mem::zeroed();
    let mut hdcMem: HDC;
    let mut hOldFont: HFONT;
    let mut bf: BLENDFUNCTION = core::mem::zeroed();

    /* Create 32 bit DIB section for the shadow */
    ZeroMemory!(core::ptr::addr_of_mut!(bi) as *mut c_void, core::mem::size_of::<BITMAPINFO>());
    bi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    bi.bmiHeader.biWidth = ((*prc).right - (*prc).left + 4) as LONG;
    bi.bmiHeader.biHeight = ((*prc).bottom - (*prc).top + 5) as LONG; // bottom-up DIB
    bi.bmiHeader.biPlanes = 1 as WORD;
    bi.bmiHeader.biBitCount = 32 as WORD;
    bi.bmiHeader.biCompression = BI_RGB as DWORD;
    hbm = CreateDIBSection(
        hdc,
        core::ptr::addr_of_mut!(bi),
        DIB_RGB_COLORS as UINT,
        core::ptr::addr_of_mut!(pBits) as *mut *mut c_void,
        core::ptr::null_mut(),
        0 as DWORD,
    );
    if hbm.is_null() {
        return 0;
    }

    /* Create memory device context for new DIB section and select it */
    hdcMem = CreateCompatibleDC(hdc);
    if hdcMem.is_null() {
        DeleteObject(hbm as HGDIOBJ);
        return 0;
    }

    hbmOld = SelectObject(hdcMem, hbm as HGDIOBJ) as HBITMAP;

    /* Draw text on our helper bitmap */
    hOldFont = SelectObject(
        hdcMem,
        GetCurrentObject(hdc, OBJ_FONT as UINT),
    ) as HFONT;
    SetTextColor(
        hdcMem,
        RGB!(16, 16, 16),
    );
    SetBkColor(
        hdcMem,
        RGB!(0, 0, 0),
    );
    SetBkMode(hdcMem, (TRANSPARENT as i32));
    SetRect(
        core::ptr::addr_of_mut!(rcText),
        0,
        0,
        (*prc).right - (*prc).left,
        (*prc).bottom - (*prc).top,
    );
    DrawTextW(
        hdcMem,
        pszText,
        cch as c_int,
        core::ptr::addr_of_mut!(rcText),
        dwFlags as UINT,
    );
    SelectObject(hdcMem, hOldFont as HGDIOBJ);

    /* Flush GDI so data pointed by pBits is valid */
    GdiFlush();

    /* Set alpha of pixels (forget about colors for now. They will be changed in next loop).
       We copy text image 4*5 times and each time alpha is added */
    x = 0 as INT;
    while x < bi.bmiHeader.biWidth {
        y = 0 as INT;
        while y < bi.bmiHeader.biHeight {
            let pDest: *mut BYTE = pBits.offset(
                ((y * bi.bmiHeader.biWidth + x) * 4 as LONG) as isize,
            );
            let mut Alpha: UINT = 0 as UINT;

            x2 = x - 4 as INT + 1 as INT;
            while x2 <= x {
                y2 = y;
                while y2 < y + 5 as INT {
                    if x2 >= 0 as INT
                        && x2 < bi.bmiHeader.biWidth
                        && y2 >= 0 as INT
                        && y2 < bi.bmiHeader.biHeight
                    {
                        let pSrc: *mut BYTE = pBits.offset(
                            ((y2 * bi.bmiHeader.biWidth + x2) * 4 as LONG)
                                as isize,
                        );
                        Alpha = Alpha.wrapping_add(
                            *pSrc.offset(0 as isize) as UINT,
                        );
                    }
                    y2 += 1;
                }
                x2 += 1;
            }

            if Alpha > 255 as UINT {
                Alpha = 255 as UINT;
            }
            *pDest.offset(3 as isize) = Alpha as BYTE;
            y += 1;
        }
        x += 1;
    }

    /* Now set the color of each pixel to shadow color * alpha (see GdiAlphaBlend) */
    x = 0 as INT;
    while x < bi.bmiHeader.biWidth {
        y = 0 as INT;
        while y < bi.bmiHeader.biHeight {
            let pDest: *mut BYTE = pBits.offset(
                ((y * bi.bmiHeader.biWidth + x) * 4 as LONG) as isize,
            );
            *pDest.offset(0 as isize) = (GetBValue!(crShadow) as c_int
                * *pDest.offset(3 as isize) as c_int
                / 255 as c_int) as BYTE;
            *pDest.offset(1 as isize) = (GetGValue!(crShadow) as c_int
                * *pDest.offset(3 as isize) as c_int
                / 255 as c_int) as BYTE;
            *pDest.offset(2 as isize) = (GetRValue!(crShadow) as c_int
                * *pDest.offset(3 as isize) as c_int
                / 255 as c_int) as BYTE;
            y += 1;
        }
        x += 1;
    }

    /* Fix ixOffset of the shadow (tested on Win) */
    ixOffset -= 3 as c_int;
    iyOffset -= 3 as c_int;

    /* Alpha blend helper image to destination DC */
    bf.BlendOp = AC_SRC_OVER as BYTE;
    bf.BlendFlags = 0 as BYTE;
    bf.SourceConstantAlpha = 255 as BYTE;
    bf.AlphaFormat = AC_SRC_ALPHA as BYTE;
    GdiAlphaBlend(
        hdc,
        (*prc).left + ixOffset,
        (*prc).top + iyOffset,
        bi.bmiHeader.biWidth,
        bi.bmiHeader.biHeight,
        hdcMem,
        0 as c_int,
        0 as c_int,
        bi.bmiHeader.biWidth,
        bi.bmiHeader.biHeight,
        bf,
    );

    /* Delete the helper bitmap */
    SelectObject(hdcMem, hbmOld as HGDIOBJ);
    DeleteObject(hbm as HGDIOBJ);
    DeleteDC(hdcMem);

    /* Finally draw the text over shadow */
    crOldText = SetTextColor(hdc, crText);
    SetBkMode(hdc, (TRANSPARENT as UINT) as i32);
    iRet = DrawTextW(
        hdc,
        pszText,
        cch as c_int,
        prc,
        dwFlags as UINT,
    ) as INT;
    SetTextColor(hdc, crOldText);

    return iRet as c_int;
}

#[no_mangle]
pub unsafe extern "system" fn LoadIconWithScaleDown(hinst: HINSTANCE, name: *const WCHAR, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, icon: *mut HICON) -> HRESULT {
    *icon = core::ptr::null_mut();

    if name.is_null() {
        return E_INVALIDARG as HRESULT;
    }

    *icon = LoadImageW(hinst, name as LPCWSTR, IMAGE_ICON as UINT, cx, cy,
        (if !hinst.is_null() || IS_INTRESOURCE!(name) { 0 } else { LR_LOADFROMFILE }) as UINT) as HICON;
    if (*icon).is_null() {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    return S_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn LoadIconMetric(
    hinst: HINSTANCE,
    name: *const WCHAR,
    size: ::core::ffi::c_int,
    icon: *mut HICON,
) -> HRESULT {
    let mut cx: ::core::ffi::c_int = 0;
    let mut cy: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/commctrl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LoadIconMetric\0" as *const u8 as *const ::core::ffi::c_char,
            1996 as ::core::ffi::c_int,
            b"(%p, %s, %d, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hinst,
            debugstr_w(name),
            size,
            icon,
        );
    }
    if (size) as u32 == LIM_SMALL {
        cx = GetSystemMetrics((SM_CXSMICON) as i32);
        cy = GetSystemMetrics((SM_CYSMICON) as i32);
    } else if (size) as u32 == LIM_LARGE {
        cx = GetSystemMetrics((SM_CXICON) as i32);
        cy = GetSystemMetrics((SM_CYICON) as i32);
    } else {
        *icon = ::core::ptr::null_mut::<HICON__>();
        return E_INVALIDARG;
    }
    return LoadIconWithScaleDown(hinst, name, cx, cy, icon);
}
