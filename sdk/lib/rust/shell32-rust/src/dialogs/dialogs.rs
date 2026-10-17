// Deterministic UTF-16 literals from the selected C++ helper closure.
use crate::prelude::*;
// One declaration per external symbol: island.rs.
use crate::island::AdjustTokenPrivileges;
use crate::island::CloseHandle;
use crate::island::GetCurrentProcess;
use crate::island::GetSystemMetrics;
use crate::island::GetWindowLongW;
use crate::island::OpenProcessToken;
use crate::island::SetWindowLongW;
pub static CPP_COM_WSTR_1D125463AF58: [WCHAR; 11] = [0x006D, 0x0073, 0x0067, 0x0069, 0x006E, 0x0061, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_24B5C33A15AA: [WCHAR; 11] = [0x0043, 0x0053, 0x0044, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_28276425D458: [WCHAR; 2] = [0x002D, 0x0000];
pub static CPP_COM_WSTR_2B4C92AAD354: [WCHAR; 3] = [0x005C, 0x0031, 0x0000];
pub static CPP_COM_WSTR_342C9756B5E3: [WCHAR; 58] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0052, 0x0075, 0x006E, 0x004D, 0x0052, 0x0055, 0x0000];
pub static CPP_COM_WSTR_491507152B61: [WCHAR; 13] = [0x004D, 0x0053, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0044, 0x006C, 0x0067, 0x0000];
pub static CPP_COM_WSTR_5E6CBD2A6368: [WCHAR; 20] = [0x0053, 0x0065, 0x0053, 0x0068, 0x0075, 0x0074, 0x0064, 0x006F, 0x0077, 0x006E, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006C, 0x0065, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_731D7EC48EEF: [WCHAR; 3] = [0x0020, 0x0009, 0x0000];
pub static CPP_COM_WSTR_7ABCE230ED71: [WCHAR; 5] = [0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_9E50FE27125E: [WCHAR; 9] = [0x0063, 0x006F, 0x006D, 0x0064, 0x006C, 0x0067, 0x0033, 0x0032, 0x0000];
pub static CPP_COM_WSTR_AA2E5A31D547: [WCHAR; 54] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E, 0x006C, 0x006F, 0x0067, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_BB5DB58F6712: [WCHAR; 22] = [0x0045, 0x006E, 0x0066, 0x006F, 0x0072, 0x0063, 0x0065, 0x0046, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x004C, 0x006F, 0x0067, 0x006F, 0x0066, 0x0066, 0x0000];
pub static CPP_COM_WSTR_BBAEC67C90FF: [WCHAR; 8] = [0x004D, 0x0052, 0x0055, 0x004C, 0x0069, 0x0073, 0x0074, 0x0000];
pub static CPP_COM_WSTR_BC5705039DA5: [WCHAR; 10] = [0x004C, 0x006F, 0x0067, 0x006F, 0x006E, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static CPP_COM_WSTR_F76C7D8D0DBB: [WCHAR; 41] = [0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0000];
// The island's `TOOLTIPS_CLASSW` (island_i386.rs etc) is bindgen output for
// commctrl.h's `#define TOOLTIPS_CLASSW L"tooltips_class32"`: bindgen renders wide
// string macros as raw ANSI byte arrays (`&[u8; 17]`), losing the `L` prefix, so the
// constant is NOT actually UTF-16 despite the name. A `CreateWindowExW` call needs a
// real `*const u16`, so this TU carries its own deterministic UTF-16 literal instead
// of the mistyped island constant (same convention as the `CPP_COM_WSTR_*` set above).
pub static DIALOGS_WSTR_TOOLTIPS_CLASS32: [WCHAR; 17] = [0x0074, 0x006F, 0x006F, 0x006C, 0x0074, 0x0069, 0x0070, 0x0073, 0x005F, 0x0063, 0x006C, 0x0061, 0x0073, 0x0073, 0x0033, 0x0032, 0x0000];

use core::ffi::c_void;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else. Adding
// `shellcom::*` / `types::*` / `win32::*` alongside it does not add anything the prelude
// lacks -- it re-imports the same names by a second path, and every one of them becomes
// ambiguous at the use site. See prelude.rs for why this is "instead of", not "as well as".

extern "system" {
    fn LookupPrivilegeValueW(lpSystemName: *const u16, lpName: *const u16, lpLuid: *mut LUID) -> i32;
}

// `ExtractIconExW` is shell32's own export, already translated -- import the Rust definition
// rather than re-declaring it as an extern, so the signature is checked rather than asserted.
use crate::shell32::iconcache::ExtractIconExW;

extern "system" {
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
}

// i386 has no `*W`-suffixed LongPtr entry point -- the 32-bit ABI is the plain `Long` one.
// ARCH-SPLIT (copy_to_menu.rs precedent).
#[inline]
unsafe fn GetWindowLongPtr(hwnd: HWND, index: INT) -> LONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        GetWindowLongW(hwnd, index) as LONG_PTR
    }
    #[cfg(not(target_arch = "x86"))]
    {
        GetWindowLongPtrW(hwnd, index)
    }
}
#[inline]
unsafe fn SetWindowLongPtr(hwnd: HWND, index: INT, value: LONG_PTR) -> LONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        SetWindowLongW(hwnd, index, value as LONG) as LONG_PTR
    }
    #[cfg(not(target_arch = "x86"))]
    {
        SetWindowLongPtrW(hwnd, index, value)
    }
}

// dialogs.cpp:179-184. These are macros over a RUNTIME query, not constants -- the user can
// change the icon metric while the dialog is open. Lowering them to `const` would freeze
// whatever the value happened to be at build time, so they stay macros here too.
macro_rules! CX_ICON {
    () => {
        GetSystemMetrics(SM_CXICON as INT)
    };
}
macro_rules! CY_ICON {
    () => {
        GetSystemMetrics(SM_CYICON as INT)
    };
}
macro_rules! CX_ITEM {
    () => {
        CX_ICON!() + 4
    };
}
macro_rules! CY_ITEM {
    () => {
        CY_ICON!() + 12
    };
}

// winuser.h `#define MAKEWPARAM(l, h) ((WPARAM)(DWORD)MAKELONG(l, h))`. The island carries
// MAKELONG but not this wrapper.
macro_rules! MAKEWPARAM {
    ($l:expr, $h:expr) => {
        (MAKELONG!($l as DWORD, $h as DWORD) as DWORD) as WPARAM
    };
}

// Not in the bindgen island; spelled here rather than as bare literals at the use sites.
pub const RT_GROUP_ICON: LPCWSTR = 14 as LPCWSTR;
pub const IDC_HAND: LPCWSTR = 32649 as LPCWSTR;
pub const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT;

// bindgen spells a callback's ABI as the RESOLVED convention for the target, so the same
// pointer is `extern "stdcall"` on i686 and `extern "C"` on amd64/arm64. Our definitions say
// `extern "system"`, which IS that convention on each of those targets -- but Rust keeps the
// two spellings as distinct nominal types on fn pointers, so a plain `as` cast is rejected as
// non-primitive and a transmute has to name whichever one the target actually uses. Hardcoding
// `stdcall` compiles on i386 and fails everywhere else.
// These forked per arch to mirror what bindgen wrote (stdcall on i386, C on the 64-bit pair)
// so a translated `system` fn could be TRANSMUTED into the island's slot type. The slots are
// spelled `system` now, which is the same ABI under one portable name, so a single alias
// matches on all three arches — and the transmutes below are identity conversions kept only
// where the surrounding call still names the alias.
type BindgenDlgProc = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;

type BindgenEnumResNameProc = unsafe extern "system" fn(HMODULE, LPCWSTR, LPWSTR, LONG_PTR) -> BOOL;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct RUNFILEDLGPARAMS {
    pub hwndOwner: HWND,
    pub hIcon: HICON,
    pub lpstrDirectory: LPCWSTR,
    pub lpstrTitle: LPCWSTR,
    pub lpstrDescription: LPCWSTR,
    pub uFlags: UINT,
    pub bCoInited: BOOL,
}

#[repr(C)]
pub struct LOGOFF_DLG_CONTEXT {
    pub bFriendlyUI: BOOL,
    pub bIsButtonHot: [BOOL; 2],
    pub hImageStrip: HBITMAP,
    pub hBrush: HBRUSH,
    pub hfFont: HFONT,
    pub bCloseDlg: BOOL,
    pub OldButtonProc: WNDPROC,
}

pub type PLOGOFF_DLG_CONTEXT = *mut LOGOFF_DLG_CONTEXT;

pub type LPFNOFN = Option<unsafe extern "system" fn(*mut OPENFILENAMEW) -> BOOL>;

#[repr(C)]
pub struct PICK_ICON_CONTEXT {
    pub hLibrary: HMODULE,
    pub hDlgCtrl: HWND,
    pub szPath: [WCHAR; MAX_PATH as usize],
    pub Index: INT,
    pub nIcons: INT,
    pub phIcons: *mut HICON,
}

pub type PPICK_ICON_CONTEXT = *mut PICK_ICON_CONTEXT;

pub type tShellDimScreen = Option<unsafe extern "system" fn(*mut *mut IUnknown, *mut HWND) -> HRESULT>;

pub const IS_PRODUCT_VERSION_WORKSTATION: u32 = 0x300;
pub const FRIENDLY_LOGOFF_IS_NOT_ENFORCED: u32 = 0x0;
pub const FONT_POINT_SIZE: u32 = 13;
pub const DARK_GREY_COLOR: u32 = 16053492;
pub const LIGHT_GREY_COLOR: u32 = 2500134;
pub const CX_BITMAP: u32 = 33;
pub const CY_BITMAP: u32 = 33;
// dialogs.cpp:1106 `#define NUMBER_OF_BUTTONS 2`. Prefixed because the NAME is not unique
// across crates: msgina-rust/src/shutdown.rs carries its own, worth 4. Both are faithful to
// their own source, so the collision is in the naming, not in either value.
pub const SHELL_LOGOFF_NUMBER_OF_BUTTONS: u32 = 2;
pub const BUTTON_SWITCH_USER: u32 = 0;
pub const BUTTON_SWITCH_USER_PRESSED: u32 = 33;
pub const BUTTON_SWITCH_USER_FOCUSED: u32 = 66;
pub const BUTTON_LOG_OFF: u32 = 99;
pub const BUTTON_LOG_OFF_PRESSED: u32 = 132;
pub const BUTTON_LOG_OFF_FOCUSED: u32 = 165;
pub const BUTTON_SWITCH_USER_DISABLED: u32 = 198;
pub const LOG_OFF_BUTTON_HOT: u32 = 0;
pub const SWITCH_USER_BUTTON_HOT: u32 = 1;

// Both of these arrived as "unprovided extern" guesses from the assembler, and both were
// wrong in a way a compile would not necessarily catch:
//   - `IS_INTRESOURCE` is a winuser.h MACRO. There is no such link symbol, so declaring it
//     extern promises the linker something nothing exports.
//   - `SendMessageW` IS a real export, but the guessed signature was `(*mut c_void, u32, u32,
//     isize)`. The island already declares it as `(HWND, UINT, WPARAM, LPARAM)`; a local
//     redeclaration silently shadows the correct one and would pass a 32-bit wparam where the
//     ABI wants a pointer-width one.
// The island's SendMessageW comes in through the prelude; IS_INTRESOURCE becomes the same
// inline predicate open_control_panel.rs already uses.
#[inline]
unsafe fn IS_INTRESOURCE(value: *const u16) -> BOOL {
    ((value as usize) >> 16 == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn EnumPickIconResourceProc(
    hModule: HMODULE,
    lpszType: LPCWSTR,
    lpszName: LPWSTR,
    lParam: LONG_PTR,
) -> BOOL {
    let pIconContext: PPICK_ICON_CONTEXT = lParam as PPICK_ICON_CONTEXT;
    let hDlgCtrl: HWND = (*pIconContext).hDlgCtrl;
    let mut lParam = lParam;

    if IS_INTRESOURCE(lpszName) != 0 {
        lParam = LOWORD(lpszName as usize as DWORD) as LONG_PTR;
    } else {
        lParam = -1;
    }

    SendMessageW(hDlgCtrl, LB_ADDSTRING, 0, lParam);

    TRUE
}
unsafe extern "C" fn DestroyIconList(hDlgCtrl: HWND, pIconContext: PPICK_ICON_CONTEXT) {
    let count = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
    if count == LB_ERR as LRESULT {
        return;
    }
    for index in 0..count {
        let icon_ptr = (*pIconContext).phIcons.offset(index as isize);
        DestroyIcon(*icon_ptr);
        *icon_ptr = core::ptr::null_mut() as HICON;
    }
}
unsafe extern "C" fn DoLoadIcons(hwndDlg: HWND, pIconContext: PPICK_ICON_CONTEXT, pszFile: LPCWSTR) -> i32 {
    let mut szExpandedPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    // Destroy previous icons
    DestroyIconList((*pIconContext).hDlgCtrl, pIconContext);
    SendMessageW((*pIconContext).hDlgCtrl, LB_RESETCONTENT, 0, 0);
    // delete[] pIconContext->phIcons -- the C++ `new HICON[]`/`delete[]` pair becomes
    // SHAlloc/SHFree. Both halves are in this TU (PICK_ICON_CONTEXT never escapes it), so the
    // pair only has to agree with itself. A size-carrying allocator would not have: the
    // nIcons == 0 cleanup below frees a block it sized from an nIcons that is already 0.
    let old_phIcons = (*pIconContext).phIcons;
    if !old_phIcons.is_null() {
        SHFree(old_phIcons as LPVOID);
    }

    // Store the path
    StringCchCopyW(
        (*pIconContext).szPath.as_mut_ptr(),
        _countof!((*pIconContext).szPath) as usize,
        pszFile,
    );
    ExpandEnvironmentStringsW(
        pszFile,
        szExpandedPath.as_mut_ptr(),
        _countof!(szExpandedPath),
    );

    // Load the module if possible
    let hLibrary = LoadLibraryExW(szExpandedPath.as_ptr(), core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
    if !(*pIconContext).hLibrary.is_null() {
        FreeLibrary((*pIconContext).hLibrary);
    }
    (*pIconContext).hLibrary = hLibrary;

    if !(*pIconContext).hLibrary.is_null() {
        // Load the icons from the module
        (*pIconContext).nIcons = ExtractIconExW(
            szExpandedPath.as_ptr(),
            -1,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        ) as i32;

        (*pIconContext).phIcons =
            SHAlloc(((*pIconContext).nIcons as usize * size_of::<HICON>()) as SIZE_T) as *mut HICON;

        if ExtractIconExW(
            szExpandedPath.as_ptr(),
            0,
            (*pIconContext).phIcons,
            core::ptr::null_mut(),
            (*pIconContext).nIcons as u32,
        ) != 0 {
            EnumResourceNamesW(
                (*pIconContext).hLibrary,
                RT_GROUP_ICON,
                // `extern "system"` and `extern "stdcall"` are the same calling convention on
                // i686 Windows (that's what "system" MEANS there), but Rust keeps them as
                // distinct nominal ABI tags on fn-pointer types, so a plain `as` cast between
                // them is rejected as non-primitive. `transmute` is the correct tool: same
                // size, same actual calling convention, only the type-level ABI label differs.
                Some(core::mem::transmute::<
                    unsafe extern "system" fn(HMODULE, LPCWSTR, LPWSTR, LONG_PTR) -> BOOL,
                    BindgenEnumResNameProc,
                >(EnumPickIconResourceProc)),
                pIconContext as LPARAM,
            );
        } else {
            (*pIconContext).nIcons = 0;
        }
    } else {
        // .ico file
        (*pIconContext).nIcons = 1;

        (*pIconContext).phIcons = SHAlloc(size_of::<HICON>() as SIZE_T) as *mut HICON;

        if ExtractIconExW(
            szExpandedPath.as_ptr(),
            0,
            (*pIconContext).phIcons,
            core::ptr::null_mut(),
            (*pIconContext).nIcons as u32,
        ) != 0 {
            SendMessageW(
                (*pIconContext).hDlgCtrl,
                LB_ADDSTRING,
                0,
                0,
            );
        } else {
            (*pIconContext).nIcons = 0;
        }
    }

    SetDlgItemTextW(hwndDlg, IDC_EDIT_PATH as INT, (*pIconContext).szPath.as_ptr());
    SendMessageW(
        (*pIconContext).hDlgCtrl,
        LB_SETCURSEL,
        0,
        0,
    );

    if (*pIconContext).nIcons == 0 {
        let old = (*pIconContext).phIcons;
        if !old.is_null() {
            SHFree(old as LPVOID);
        }
        (*pIconContext).phIcons = core::ptr::null_mut();
    }

    ((*pIconContext).nIcons > 0) as i32
}
unsafe extern "C" fn NoIconsInFile(hwndDlg: HWND, pIconContext: PPICK_ICON_CONTEXT) {
    let mut strText = CStringW::new();
    let strTitle = CStringW::from_resource(IDS_PICK_ICON_TITLE);
    strText.FormatResStr(IDS_NO_ICONS, (*pIconContext).szPath.as_ptr());
    MessageBoxW(hwndDlg, strText.as_ptr(), strTitle.as_ptr(), MB_ICONWARNING);
    DoLoadIcons(hwndDlg, pIconContext, g_pszShell32);
}
#[no_mangle]
pub unsafe extern "system" fn PickIconProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut lpmis: LPMEASUREITEMSTRUCT;
    let mut lpdis: LPDRAWITEMSTRUCT;
    let hIcon: HICON;
    let index: i32;
    let count: i32;
    let mut szText: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut szFilter: [u16; 100] = [0u16; 100];
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let pIconContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as PPICK_ICON_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            let pIconContext = lParam as PPICK_ICON_CONTEXT;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pIconContext as LONG_PTR);
            (*pIconContext).hDlgCtrl = GetDlgItem(hwndDlg, IDC_PICKICON_LIST as INT);
            SendMessageW((*pIconContext).hDlgCtrl, LB_SETCOLUMNWIDTH, CX_ITEM!() as WPARAM, 0);

            if DoLoadIcons(hwndDlg, pIconContext, (*pIconContext).szPath.as_ptr()) == 0 {
                NoIconsInFile(hwndDlg, pIconContext);
            }

            count = SendMessageW((*pIconContext).hDlgCtrl, LB_GETCOUNT, 0, 0) as i32;
            if count != LB_ERR {
                if (*pIconContext).Index < 0 {
                    let lParam = (-(*pIconContext).Index) as LPARAM;
                    (*pIconContext).Index = SendMessageW((*pIconContext).hDlgCtrl, LB_FINDSTRINGEXACT, -1i32 as WPARAM, lParam) as i32;
                }

                if (*pIconContext).Index < 0 || count <= (*pIconContext).Index {
                    (*pIconContext).Index = 0;
                }

                SendMessageW((*pIconContext).hDlgCtrl, LB_SETCURSEL, (*pIconContext).Index as WPARAM, 0);
                SendMessageW((*pIconContext).hDlgCtrl, LB_SETTOPINDEX, (*pIconContext).Index as WPARAM, 0);
            }

            SHAutoComplete(GetDlgItem(hwndDlg, IDC_EDIT_PATH as INT), SHACF_DEFAULT);
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            DestroyIconList((*pIconContext).hDlgCtrl, pIconContext);

            if !(*pIconContext).phIcons.is_null() {
                SHFree((*pIconContext).phIcons as LPVOID);
            }

            if (*pIconContext).hLibrary != 0 as HMODULE {
                FreeLibrary((*pIconContext).hLibrary);
            }
        }

        WM_COMMAND => {
            match LOWORD(wParam as DWORD) as UINT {
                // C's `break` here leaves the inner switch(LOWORD(wParam)); a labeled block
                // scoped to the arm is the exact equivalent.
                IDOK => 'case_idok: {
                    GetDlgItemTextW(hwndDlg, IDC_EDIT_PATH as INT, szText.as_mut_ptr(), szText.len() as i32);
                    if lstrcmpiW(szText.as_ptr(), (*pIconContext).szPath.as_ptr()) != 0 {
                        if DoLoadIcons(hwndDlg, pIconContext, szText.as_ptr()) == 0 {
                            NoIconsInFile(hwndDlg, pIconContext);
                        }
                        break 'case_idok;
                    }

                    (*pIconContext).Index = SendMessageW((*pIconContext).hDlgCtrl, LB_GETCURSEL, 0, 0) as i32;
                    GetDlgItemTextW(hwndDlg, IDC_EDIT_PATH as INT, (*pIconContext).szPath.as_mut_ptr(), (*pIconContext).szPath.len() as i32);
                    EndDialog(hwndDlg, 1);
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                }

                IDC_PICKICON_LIST => {
                    match HIWORD(wParam as DWORD) as UINT {
                        LBN_SELCHANGE => {
                            InvalidateRect(lParam as HWND, core::ptr::null_mut(), TRUE);
                        }
                        LBN_DBLCLK => {
                            SendMessageW(hwndDlg, WM_COMMAND, MAKEWPARAM!(IDOK, 0), 0);
                        }
                        _ => {}
                    }
                }

                IDC_BUTTON_PATH => 'case_path: {
                    // Choose the module path
                    let mut strTitle = CStringW::new();
                    szText[0] = 0;
                    szFilter[0] = 0;
                    ZeroMemory(core::ptr::addr_of_mut!(ofn).cast::<OPENFILENAMEW>(), core::mem::size_of::<OPENFILENAMEW>());
                    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
                    ofn.hwndOwner = hwndDlg;
                    ofn.lpstrFile = szText.as_mut_ptr() as LPWSTR;
                    ofn.nMaxFile = szText.len() as DWORD;
                    strTitle.LoadStringW(IDS_PICK_ICON_TITLE);
                    ofn.lpstrTitle = strTitle.as_ptr();
                    LoadStringW(shell32_hInstance, IDS_PICK_ICON_FILTER, szFilter.as_mut_ptr(), szFilter.len() as i32);
                    ofn.lpstrFilter = szFilter.as_ptr();
                    if GetOpenFileNameW(core::ptr::addr_of_mut!(ofn)) == 0 {
                        break 'case_path;
                    }

                    if DoLoadIcons(hwndDlg, pIconContext, szText.as_ptr()) == 0 {
                        NoIconsInFile(hwndDlg, pIconContext);
                    }
                }

                _ => {}
            }
        }

        WM_MEASUREITEM => {
            lpmis = lParam as LPMEASUREITEMSTRUCT;
            (*lpmis).itemHeight = CY_ITEM!() as u32;
            return TRUE as INT_PTR;
        }

        WM_DRAWITEM => 'case_drawitem: {
            lpdis = lParam as LPDRAWITEMSTRUCT;
            if (*lpdis).itemID == !1 as UINT {
                break 'case_drawitem;
            }
            match (*lpdis).itemAction {
                ODA_SELECT | ODA_DRAWENTIRE => {
                    index = SendMessageW((*pIconContext).hDlgCtrl, LB_GETCURSEL, 0, 0) as i32;
                    hIcon = *(*pIconContext).phIcons.add((*lpdis).itemID as usize);

                    if (*lpdis).itemID == index as UINT {
                        FillRect((*lpdis).hDC, core::ptr::addr_of!((*lpdis).rcItem), (COLOR_HIGHLIGHT + 1) as HBRUSH);
                    } else {
                        FillRect((*lpdis).hDC, core::ptr::addr_of!((*lpdis).rcItem), (COLOR_WINDOW + 1) as HBRUSH);
                    }

                    let x = (*lpdis).rcItem.left + (CX_ITEM!() - CX_ICON!()) / 2;
                    let y = (*lpdis).rcItem.top + (CY_ITEM!() - CY_ICON!()) / 2;

                    DrawIconEx((*lpdis).hDC, x, y, hIcon, 0, 0, 0, core::ptr::null_mut(), DI_NORMAL);
                }
                _ => {}
            }
            return TRUE as INT_PTR;
        }

        _ => {}
    }

    FALSE as INT_PTR
}

#[no_mangle]
pub unsafe extern "system" fn PickIconDlg(
    hWndOwner: HWND,
    lpstrFile: LPWSTR,
    nMaxFile: u32,
    lpdwIconIndex: *mut i32,
) -> i32 {
    let _com_init = CCoInit::new();
    let mut res: INT_PTR;
    let mut sz_expanded_path: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    let mut icon_context: PICK_ICON_CONTEXT = core::mem::zeroed();
    icon_context.hLibrary = core::ptr::null_mut();
    icon_context.Index = *lpdwIconIndex;

    StringCchCopyW(icon_context.szPath.as_mut_ptr(), _countof!(&icon_context.szPath) as usize, lpstrFile);
    ExpandEnvironmentStringsW(lpstrFile, sz_expanded_path.as_mut_ptr(), _countof!(&sz_expanded_path));

    if sz_expanded_path[0] == 0
        || GetFileAttributesW(sz_expanded_path.as_ptr()) == INVALID_FILE_ATTRIBUTES
    {
        if sz_expanded_path[0] != 0 {
            let mut str_text = CStringW::new();
            let str_title = CStringW::from_resource(IDS_PICK_ICON_TITLE);
            str_text.FormatResStr(IDS_FILE_NOT_FOUND, lpstrFile);
            MessageBoxW(
                hWndOwner,
                str_text.as_ptr(),
                str_title.as_ptr(),
                MB_ICONWARNING,
            );
        }
        StringCchCopyW(
            icon_context.szPath.as_mut_ptr(),
            _countof!(&icon_context.szPath) as usize,
            g_pszShell32,
        );
    }

    res = DialogBoxParamW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDD_PICK_ICON),
        hWndOwner,
        // See the EnumPickIconResourceProc call site above for why this is `transmute`
        // rather than `as`: "system" and "stdcall" are the same ABI on i686 Windows, but
        // different nominal fn-pointer types to the type checker.
        Some(core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR,
            BindgenDlgProc,
        >(PickIconProc)),
        &mut icon_context as *mut PICK_ICON_CONTEXT as LPARAM,
    );

    if res != 0 {
        StringCchCopyW(lpstrFile, nMaxFile as usize, icon_context.szPath.as_mut_ptr());
        *lpdwIconIndex = icon_context.Index;
    }

    res as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn RunFileDlg(
    hWndOwner: HWND,
    hIcon: HICON,
    lpstrDirectory: LPCWSTR,
    lpstrTitle: LPCWSTR,
    lpstrDescription: LPCWSTR,
    uFlags: UINT,
) {
    let mut rfdp: RUNFILEDLGPARAMS = core::mem::zeroed();
    rfdp.hwndOwner = hWndOwner;
    rfdp.hIcon = hIcon;
    rfdp.lpstrDirectory = lpstrDirectory;
    rfdp.lpstrTitle = lpstrTitle;
    rfdp.lpstrDescription = lpstrDescription;
    rfdp.uFlags = uFlags;

    DialogBoxParamW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDD_RUN),
        hWndOwner,
        // Same reason as the PickIconProc call site above: "system" and "stdcall" are
        // one ABI on i686 but two nominal fn-pointer types.
        Some(core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR,
            BindgenDlgProc,
        >(RunDlgProc)),
        &raw const rfdp as LPARAM,
    );
}
unsafe extern "C" fn RunDlg_GetParentDir(cmdline: *const u16) -> *mut u16 {
    let mut src = cmdline;
    let mut dest: *mut u16;
    let mut result: *mut u16;
    let mut result_end: *mut u16 = core::ptr::null_mut();

    let heap_handle = GetProcessHeap();
    let len = strlenW(cmdline);
    let alloc_size = (len.wrapping_add(5 as usize)) * core::mem::size_of::<u16>();
    result = HeapAlloc(heap_handle, 0, alloc_size as SIZE_T) as *mut u16;
    if result.is_null() {
        return core::ptr::null_mut();
    }

    dest = result;

    if *src == '"' as u16 {
        src = src.add(1);
        while *src != 0 && *src != '"' as u16 {
            if *src == '\\' as u16 {
                result_end = dest;
            }
            *dest = *src;
            src = src.add(1);
            dest = dest.add(1);
        }
    } else {
        while *src != 0 {
            if isspaceW(*src) != 0 {
                *dest = 0;
                if INVALID_FILE_ATTRIBUTES != GetFileAttributesW(result) {
                    break;
                }
                strcatW(dest, CPP_COM_WSTR_7ABCE230ED71.as_ptr());
                if INVALID_FILE_ATTRIBUTES != GetFileAttributesW(result) {
                    break;
                }
            } else if *src == '\\' as u16 {
                result_end = dest;
            }
            *dest = *src;
            src = src.add(1);
            dest = dest.add(1);
        }
    }

    if !result_end.is_null() {
        *result_end = 0;
        return result;
    } else {
        HeapFree(GetProcessHeap(), 0, result as LPVOID);
        return core::ptr::null_mut();
    }
}
unsafe extern "C" fn EnableOkButtonFromEditContents(hwnd: HWND) {
    let mut enable: BOOL = FALSE;
    let edit: HWND = GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT);
    let length: INT = GetWindowTextLengthW(edit);
    if length > 0 {
        let psz: PWCHAR = HeapAlloc(
            GetProcessHeap(),
            0,
            ((length as usize + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as PWCHAR;
        if !psz.is_null() {
            GetWindowTextW(edit, psz, length + 1);
            let mut n: INT = 0;
            while n < length && enable == 0 {
                enable = if *psz.add(n as usize) != (' ' as u16) { 1 } else { 0 };
                n += 1;
            }
            HeapFree(
                GetProcessHeap(),
                0,
                psz as LPVOID,
            );
        } else {
            enable = TRUE;
        }
    }
    EnableWindow(
        GetDlgItem(hwnd, IDOK as INT),
        enable,
    );
}
unsafe extern "C" fn ConfirmDialog(hWndOwner: HWND, PromptId: u32, TitleId: u32) -> BOOL {
    let mut prompt: [u16; 256] = [0u16; 256];
    let mut title: [u16; 256] = [0u16; 256];
    LoadStringW(shell32_hInstance, PromptId, prompt.as_mut_ptr(), 256);
    LoadStringW(shell32_hInstance, TitleId, title.as_mut_ptr(), 256);
    (MessageBoxW(hWndOwner, prompt.as_ptr() as *const u16, title.as_ptr() as *const u16, MB_YESNO | MB_ICONQUESTION) == IDYES as INT) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn CallShellDimScreen(pUnknown: *mut *mut IUnknown, hWindow: *mut HWND) -> BOOL {
    // Both of these are function-local `static`s in the C++ (dialogs.cpp:1024-1025), so they
    // stay function-local here: the resolve happens once and the result is cached across calls.
    type tShellDimScreen =
        unsafe extern "system" fn(Unknown: *mut *mut IUnknown, hWindow: *mut HWND) -> HRESULT;
    static mut ShellDimScreen: Option<tShellDimScreen> = None;
    static mut Initialized: BOOL = FALSE;

    if Initialized == FALSE {
        let mod_ = LoadLibraryW(CPP_COM_WSTR_1D125463AF58.as_ptr());
        // msgina.dll exports this by ORDINAL 16 only -- the `(LPCSTR)16` in the C++ is an
        // ordinal, not a truncated name pointer.
        ShellDimScreen = core::mem::transmute(GetProcAddress(mod_, 16 as *const i8));
        Initialized = TRUE;
    }
    let mut hr: HRESULT = E_FAIL;
    if let Some(f) = ShellDimScreen {
        hr = f(pUnknown, hWindow);
    }
    (hr >= 0) as BOOL
}
unsafe extern "C" fn EnablePrivilege(lpszPrivilegeName: *const u16, bEnablePrivilege: i32) -> i32 {
    let mut Success: i32;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();

    Success = OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES,
        core::ptr::addr_of_mut!(hToken),
    );
    if Success == 0 {
        return Success;
    }

    'Quit: loop {
        Success = LookupPrivilegeValueW(
            core::ptr::null(),
            lpszPrivilegeName,
            core::ptr::addr_of_mut!(tp.Privileges[0].Luid),
        );
        if Success == 0 {
            break 'Quit;
        }

        tp.PrivilegeCount = 1;
        tp.Privileges[0].Attributes = if bEnablePrivilege != 0 {
            SE_PRIVILEGE_ENABLED
        } else {
            0
        };

        Success = AdjustTokenPrivileges(
            hToken,
            FALSE,
            core::ptr::addr_of_mut!(tp),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        break;
    }

    CloseHandle(hToken);
    return Success;
}

#[no_mangle]
pub unsafe extern "system" fn RestartDialogEx(
    hWndOwner: HWND,
    lpwstrReason: LPCWSTR,
    uFlags: DWORD,
    uReason: DWORD,
) -> i32 {
    let mut fadeHandler: *mut IUnknown = core::ptr::null_mut();
    let mut parent: HWND = core::ptr::null_mut();

    if CallShellDimScreen(&mut fadeHandler, &mut parent) == 0i32 {
        parent = hWndOwner;
    }

    if ConfirmDialog(parent, IDS_RESTART_PROMPT, IDS_RESTART_TITLE) != 0i32 {
        EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), TRUE);
        ExitWindowsEx(EWX_REBOOT, uReason);
        EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), FALSE);
    }

    if !fadeHandler.is_null() {
        (*fadeHandler).Release();
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn DrawIconOnOwnerDrawnButtons(
    pdis: *mut DRAWITEMSTRUCT,
    pContext: *mut LOGOFF_DLG_CONTEXT,
) -> BOOL {
    let mut bRet: BOOL = FALSE;
    let mut y: INT = 0;
    let rect: RECT;

    let hdcMem: HDC = CreateCompatibleDC((*pdis).hDC);
    let hbmOld: HBITMAP = SelectObject(hdcMem, (*pContext).hImageStrip as *mut c_void) as HBITMAP;
    rect = (*pdis).rcItem;

    match (*pdis).CtlID {
        IDC_LOG_OFF_BUTTON => {
            match (*pdis).itemAction {
                ODA_DRAWENTIRE | ODA_FOCUS | ODA_SELECT => {
                    y = BUTTON_LOG_OFF as INT;
                    if ((*pdis).itemState & ODS_SELECTED) != 0 {
                        y = BUTTON_LOG_OFF_PRESSED as INT;
                    } else if (*pContext).bIsButtonHot[LOG_OFF_BUTTON_HOT as usize] != 0
                        || ((*pdis).itemState & ODS_FOCUS) != 0
                    {
                        y = BUTTON_LOG_OFF_FOCUSED as INT;
                    }
                }
                _ => {}
            }
        }
        IDC_SWITCH_USER_BUTTON => {
            match (*pdis).itemAction {
                ODA_DRAWENTIRE | ODA_FOCUS | ODA_SELECT => {
                    y = BUTTON_SWITCH_USER as INT;
                    if ((*pdis).itemState & ODS_SELECTED) != 0 {
                        y = BUTTON_SWITCH_USER_PRESSED as INT;
                    } else if (*pContext).bIsButtonHot[SWITCH_USER_BUTTON_HOT as usize] != 0
                        || ((*pdis).itemState & ODS_FOCUS) != 0
                    {
                        y = BUTTON_SWITCH_USER_FOCUSED as INT;
                    } else if ((*pdis).itemState & ODS_DISABLED) != 0 {
                        y = BUTTON_SWITCH_USER_DISABLED as INT;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    bRet = BitBlt(
        (*pdis).hDC,
        (rect.right - rect.left - CX_BITMAP as LONG) / 2,
        (rect.bottom - rect.top - CY_BITMAP as LONG) / 2,
        CX_BITMAP as INT,
        CY_BITMAP as INT,
        hdcMem,
        0,
        y,
        SRCCOPY,
    );

    SelectObject(hdcMem, hbmOld as *mut c_void);
    DeleteDC(hdcMem);

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn OwnerDrawButtonSubclass(
    hButton: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> isize {
    let pContext = GetWindowLongPtr(
        GetParent(hButton),
        GWLP_USERDATA,
    ) as PLOGOFF_DLG_CONTEXT;

    let buttonID = GetDlgCtrlID(hButton);

    match uMsg {
        WM_MOUSEMOVE => {
            let mut pt = POINT {
                x: GET_X_LPARAM(lParam),
                y: GET_Y_LPARAM(lParam),
            };

            if GetCapture() != hButton {
                SetCapture(hButton);

                match buttonID as UINT {
                    IDC_LOG_OFF_BUTTON => {
                        (*pContext).bIsButtonHot[LOG_OFF_BUTTON_HOT as usize] = TRUE;
                    }
                    IDC_SWITCH_USER_BUTTON => {
                        (*pContext).bIsButtonHot[SWITCH_USER_BUTTON_HOT as usize] = TRUE;
                    }
                    _ => {}
                }
                SetCursor(LoadCursorW(
                    core::ptr::null_mut::<c_void>() as HINSTANCE,
                    IDC_HAND,
                ));
            }

            ClientToScreen(hButton, &raw mut pt);
            let hwndTarget = WindowFromPoint(pt);

            if hwndTarget != hButton {
                ReleaseCapture();

                match buttonID as UINT {
                    IDC_LOG_OFF_BUTTON => {
                        (*pContext).bIsButtonHot[LOG_OFF_BUTTON_HOT as usize] = FALSE;
                    }
                    IDC_SWITCH_USER_BUTTON => {
                        (*pContext).bIsButtonHot[SWITCH_USER_BUTTON_HOT as usize] = FALSE;
                    }
                    _ => {}
                }
            }
            InvalidateRect(hButton, core::ptr::null_mut(), FALSE);
        }

        WM_SETFOCUS => {
            SendMessageW(
                GetParent(hButton),
                DM_SETDEFID,
                buttonID as WPARAM,
                0,
            );
        }

        WM_KILLFOCUS => {
            SendMessageW(
                GetParent(hButton),
                DM_SETDEFID,
                IDCANCEL as WPARAM,
                0,
            );
        }

        _ => {}
    }

    CallWindowProcW(
        (*pContext).OldButtonProc,
        hButton,
        uMsg,
        wParam,
        lParam,
    ) as isize
}
#[no_mangle]
pub unsafe extern "C" fn CreateToolTipForButtons(controlID: i32, detailID: i32, hDlg: HWND, titleID: i32) {
    let mut hwndTool: HWND = core::ptr::null_mut();
    let mut hwndTip: HWND = core::ptr::null_mut();
    let mut szBuffer: [u16; 256] = [0u16; 256];
    let mut tool: TTTOOLINFOW = core::mem::zeroed();

    hwndTool = GetDlgItem(hDlg, controlID);

    tool.cbSize = core::mem::size_of::<TTTOOLINFOW>() as u32;
    tool.hwnd = hDlg;
    tool.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
    tool.uId = hwndTool as UINT_PTR;

    hwndTip = CreateWindowExW(
        0,
        DIALOGS_WSTR_TOOLTIPS_CLASS32.as_ptr(),
        core::ptr::null_mut(),
        WS_POPUP | TTS_ALWAYSTIP | TTS_BALLOON,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        hDlg,
        core::ptr::null_mut(),
        shell32_hInstance,
        core::ptr::null_mut(),
    );

    LoadStringW(
        shell32_hInstance,
        detailID as UINT,
        szBuffer.as_mut_ptr(),
        _countof!(szBuffer) as i32,
    );
    tool.lpszText = szBuffer.as_mut_ptr();
    SendMessageW(
        hwndTip,
        TTM_ADDTOOLW,
        0,
        (&mut tool) as *mut _ as LPARAM,
    );

    LoadStringW(
        shell32_hInstance,
        titleID as UINT,
        szBuffer.as_mut_ptr(),
        _countof!(szBuffer) as i32,
    );
    SendMessageW(
        hwndTip,
        TTM_SETTITLEW,
        TTI_NONE as WPARAM,
        szBuffer.as_mut_ptr() as LPARAM,
    );
    SendMessageW(
        hwndTip,
        TTM_SETMAXTIPWIDTH,
        0,
        250,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EndFriendlyDialog(hwnd: HWND, pContext: PLOGOFF_DLG_CONTEXT) {
    DeleteObject((*pContext).hBrush as *mut c_void);
    DeleteObject((*pContext).hImageStrip as *mut c_void);
    DeleteObject((*pContext).hfFont as *mut c_void);

    for i in 0..SHELL_LOGOFF_NUMBER_OF_BUTTONS {
        SetWindowLongPtr(
            GetDlgItem(hwnd, (IDC_LOG_OFF_BUTTON + i) as INT),
            GWLP_WNDPROC,
            core::mem::transmute::<WNDPROC, LONG_PTR>((*pContext).OldButtonProc),
        );
    }
}
unsafe extern "C" fn IsFriendlyUIActive() -> i32 {
    let mut dwType: u32 = 0;
    let mut dwValue: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lRet: i32 = 0;

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        CPP_COM_WSTR_F76C7D8D0DBB.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    );
    if lRet != ERROR_SUCCESS as i32 {
        return FALSE;
    }

    dwValue = 0;
    dwSize = core::mem::size_of::<u32>() as u32;
    lRet = RegQueryValueExW(
        hKey,
        CPP_COM_WSTR_BB5DB58F6712.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwValue) as *mut u8,
        core::ptr::addr_of_mut!(dwSize),
    );

    if lRet == ERROR_SUCCESS as i32 && dwType == REG_DWORD && dwValue != FRIENDLY_LOGOFF_IS_NOT_ENFORCED {
        RegCloseKey(hKey);
        return TRUE;
    }

    dwValue = 0;
    dwSize = core::mem::size_of::<u32>() as u32;
    lRet = RegQueryValueExW(
        hKey,
        CPP_COM_WSTR_24B5C33A15AA.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwValue) as *mut u8,
        core::ptr::addr_of_mut!(dwSize),
    );
    RegCloseKey(hKey);

    if lRet != ERROR_SUCCESS as i32 || dwType != REG_DWORD || dwValue != IS_PRODUCT_VERSION_WORKSTATION {
        return FALSE;
    }

    lRet = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        CPP_COM_WSTR_AA2E5A31D547.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKey),
    );
    if lRet != ERROR_SUCCESS as i32 {
        return FALSE;
    }

    dwValue = 0;
    dwSize = core::mem::size_of::<u32>() as u32;
    lRet = RegQueryValueExW(
        hKey,
        CPP_COM_WSTR_BC5705039DA5.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(dwValue) as *mut u8,
        core::ptr::addr_of_mut!(dwSize),
    );
    RegCloseKey(hKey);

    if lRet != ERROR_SUCCESS as i32 || dwType != REG_DWORD {
        return FALSE;
    }

    (dwValue != 0) as i32
}
unsafe extern "C" fn FancyLogoffOnInit(hwnd: HWND, pContext: PLOGOFF_DLG_CONTEXT) {
    let mut hdc: HDC = core::ptr::null_mut();
    hdc = GetDC(core::ptr::null_mut());
    let lfHeight: i32 = -(MulDiv(
        FONT_POINT_SIZE as i32,
        GetDeviceCaps(hdc, LOGPIXELSY as i32),
        72,
    ) as i32);
    ReleaseDC(core::ptr::null_mut(), hdc);
    (*pContext).hfFont = CreateFontW(
        lfHeight,
        0,
        0,
        0,
        FW_MEDIUM as i32,
        FALSE as u32,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        CPP_COM_WSTR_491507152B61.as_ptr(),
    );
    SendDlgItemMessageW(
        hwnd,
        IDC_LOG_OFF_TEXT_STATIC as INT,
        WM_SETFONT,
        (*pContext).hfFont as WPARAM,
        TRUE as LPARAM,
    );
    (*pContext).hBrush = CreateSolidBrush(DARK_GREY_COLOR);
    (*pContext).hImageStrip = LoadBitmapW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDB_IMAGE_STRIP),
    );
    (*pContext).OldButtonProc = core::mem::transmute::<LONG_PTR, WNDPROC>(GetWindowLongPtr(
        GetDlgItem(hwnd, IDC_LOG_OFF_BUTTON as INT),
        GWLP_WNDPROC,
    ));

    for i in (0 as i32)..(SHELL_LOGOFF_NUMBER_OF_BUTTONS as i32) {
        (*pContext).bIsButtonHot[i as usize] = FALSE;
        {
            let btn = GetDlgItem(hwnd, IDC_LOG_OFF_BUTTON as i32 + i);
            SetWindowLongPtr(btn, GWLP_WNDPROC, OwnerDrawButtonSubclass as LONG_PTR);
        }
        CreateToolTipForButtons(
            IDC_LOG_OFF_BUTTON as i32 + i,
            IDS_LOG_OFF_DESC as i32 + i,
            hwnd,
            IDS_LOG_OFF_TITLE as i32 + i,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn LogOffDialogProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pdis: *mut DRAWITEMSTRUCT = lParam as *mut DRAWITEMSTRUCT;
    let mut pContext: PLOGOFF_DLG_CONTEXT = GetWindowLongPtr(hwnd, GWLP_USERDATA) as PLOGOFF_DLG_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pContext = lParam as PLOGOFF_DLG_CONTEXT;
            SetWindowLongPtr(hwnd, GWLP_USERDATA, pContext as LONG_PTR);
            if (*pContext).bFriendlyUI != 0i32 {
                FancyLogoffOnInit(hwnd, pContext);
            }
            return TRUE as INT_PTR;
        }
        WM_DESTROY => {
            if (*pContext).bFriendlyUI != 0i32 {
                EndFriendlyDialog(hwnd, pContext);
            }
            return TRUE as INT_PTR;
        }
        WM_ACTIVATE => {
            if LOWORD(wParam as DWORD) as UINT == WA_INACTIVE {
                if (*pContext).bCloseDlg == 0i32 {
                    (*pContext).bCloseDlg = TRUE;
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                }
            }
            return FALSE as INT_PTR;
        }
        WM_CLOSE => {
            (*pContext).bCloseDlg = TRUE;
            EndDialog(hwnd, IDCANCEL as INT_PTR);
        }
        WM_COMMAND => {
            match LOWORD(wParam as DWORD) as UINT {
                IDC_LOG_OFF_BUTTON | IDOK => {
                    ExitWindowsEx(EWX_LOGOFF, 0);
                }
                IDCANCEL => {
                    (*pContext).bCloseDlg = TRUE;
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                }
                _ => {}
            }
        }
        WM_CTLCOLORSTATIC => {
            let hdcStatic: HDC = wParam as HDC;
            let static_id: UINT = GetWindowLongPtr(lParam as HWND, GWL_ID) as UINT;
            match static_id {
                IDC_LOG_OFF_TEXT_STATIC => {
                    SetTextColor(hdcStatic, DARK_GREY_COLOR);
                    SetBkMode(hdcStatic, TRANSPARENT as i32);
                    return GetStockObject(HOLLOW_BRUSH as i32) as INT_PTR;
                }
                IDC_LOG_OFF_STATIC | IDC_SWITCH_USER_STATIC => {
                    SetTextColor(hdcStatic, LIGHT_GREY_COLOR);
                    SetBkMode(hdcStatic, TRANSPARENT as i32);
                    return (*pContext).hBrush as INT_PTR;
                }
                _ => {}
            }
            return FALSE as INT_PTR;
        }
        WM_DRAWITEM => {
            match (*pdis).CtlID {
                IDC_LOG_OFF_BUTTON | IDC_SWITCH_USER_BUTTON => {
                    return DrawIconOnOwnerDrawnButtons(pdis, pContext) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    FALSE as INT_PTR
}

#[no_mangle]
pub unsafe extern "system" fn LogoffWindowsDialog(hWndOwner: HWND) -> c_int {
    let mut fade_handler: *mut IUnknown = core::ptr::null_mut();
    let mut parent: HWND = core::ptr::null_mut();
    let mut LogoffDialogID: u32 = IDD_LOG_OFF;
    let mut Context: LOGOFF_DLG_CONTEXT = core::mem::zeroed();
    let _fact1 = 0;

    if CallShellDimScreen(&mut fade_handler as *mut *mut IUnknown, &mut parent as *mut HWND) == 0 {
        parent = hWndOwner;
    }

    Context.bFriendlyUI = IsFriendlyUIActive();
    if Context.bFriendlyUI != 0 {
        LogoffDialogID = IDD_LOG_OFF_FANCY;
    }

    DialogBoxParamW(
        shell32_hInstance,
        MAKEINTRESOURCEW(LogoffDialogID),
        parent,
        // See the EnumPickIconResourceProc call site (dialogs.rs, DoLoadIcons) for why this
        // is `transmute` rather than `as`.
        Some(core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR,
            BindgenDlgProc,
        >(LogOffDialogProc)),
        core::ptr::addr_of_mut!(Context) as LPARAM,
    );

    if !fade_handler.is_null() {
        ((*(*fade_handler).lpVtbl).Release)(fade_handler);
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn RestartDialog(
    hWndOwner: HWND,
    lpstrReason: LPCWSTR,
    uFlags: DWORD,
) -> i32 {
    RestartDialogEx(hWndOwner, lpstrReason, uFlags, 0)
}
#[no_mangle]
pub unsafe extern "C" fn ExitWindowsDialog_backup(hWndOwner: HWND) {
    if ConfirmDialog(hWndOwner, IDS_SHUTDOWN_PROMPT, IDS_SHUTDOWN_TITLE) != 0 {
        EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), TRUE);
        ExitWindowsEx(EWX_SHUTDOWN, 0);
        EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), FALSE);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExitWindowsDialog(hWndOwner: HWND) {
    type ShellShFunc = Option<unsafe extern "system" fn(HWND, *const u16, u32) -> u32>;
    let msginaDll: HINSTANCE;
    let mut fadeHandler: *mut IUnknown = core::ptr::null_mut();
    let mut parent: HWND = core::ptr::null_mut();

    msginaDll = LoadLibraryW(CPP_COM_WSTR_1D125463AF58.as_ptr());

    if CallShellDimScreen(&mut fadeHandler, &mut parent) == FALSE {
        parent = hWndOwner;
    }

    if msginaDll.is_null() {
        ExitWindowsDialog_backup(parent);
        if !fadeHandler.is_null() {
            (*(fadeHandler)).Release();
        }
        return;
    }

    let pShellShutdownDialog: ShellShFunc = core::mem::transmute(
        GetProcAddress(msginaDll, b"ShellShutdownDialog\0".as_ptr() as *const i8)
    );

    if let Some(pShellShutdownDialog) = pShellShutdownDialog {
        let returnValue: u32 = pShellShutdownDialog(parent, core::ptr::null(), 0);
        match returnValue {
            0x01 => {
                ExitWindowsEx(EWX_LOGOFF, 0);
            }
            0x02 => {
                EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), TRUE);
                ExitWindowsEx(EWX_SHUTDOWN, 0);
                EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), FALSE);
            }
            0x03 => {}
            0x04 => {
                EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), TRUE);
                ExitWindowsEx(EWX_REBOOT, 0);
                EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), FALSE);
            }
            0x10 => {
                if IsPwrSuspendAllowed() != 0 {
                    EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), TRUE);
                    SetSuspendState(FALSE as u8, FALSE as u8, FALSE as u8);
                    EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), FALSE);
                }
            }
            0x40 => {
                if IsPwrHibernateAllowed() != 0 {
                    EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), TRUE);
                    SetSuspendState(TRUE as u8, FALSE as u8, TRUE as u8);
                    EnablePrivilege(CPP_COM_WSTR_5E6CBD2A6368.as_ptr(), FALSE);
                }
            }
            _ => {}
        }
    } else {
        ExitWindowsDialog_backup(parent);
    }

    if !fadeHandler.is_null() {
        (*(fadeHandler)).Release();
    }
    FreeLibrary(msginaDll);
}

// ---- FillList: translated by hand; the paid worker was audit-rejected. ----
// FillList — dll/win32/shell32/dialogs/dialogs.cpp:778-1001.
//
// This snippet is meant to be pasted into sdk/lib/rust/shell32-rust/src/dialogs/dialogs.rs,
// which already has `use crate::prelude::*;` at its top plus the CPP_COM_WSTR_* literal pool.
// It reuses four of those already-declared statics verbatim (byte-for-byte identical to what
// this function needs) instead of redefining them:
//   CPP_COM_WSTR_28276425D458 = "-\0"                                        (L"-")
//   CPP_COM_WSTR_342C9756B5E3 = "Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU\0"
//   CPP_COM_WSTR_BBAEC67C90FF = "MRUList\0"
//   CPP_COM_WSTR_2B4C92AAD354 = "\\1\0"                                      (L"\\1")
//
// Two helpers this TU needs are NOT yet in the crate and must land alongside FillList:

// The CRT entry point ntoskrnl/msvcrt already export. Never hand-roll the copy loop (LLVM folds
// a byte-copy loop back into a call to itself -> infinite self-recursion): see AGENTS.md.
extern "C" {
    fn memmove(dest: PVOID, src: LPCVOID, n: SIZE_T) -> PVOID;
}

/// winuser.h `#define MAKELPARAM(l, h) ((LPARAM)(DWORD)MAKELONG(l, h))`. The island/prelude
/// carry `MAKELONG!` but not this wrapper (mirrors the existing `MAKEWPARAM!` this file already
/// hand-adds for the same reason). Cast through `u32` before `LPARAM` so the extension to a
/// 64-bit LPARAM on amd64/arm64 is zero-extending, matching the real macro's `(DWORD)` cast.
macro_rules! MAKELPARAM {
    ($l:expr, $h:expr $(,)?) => {
        (MAKELONG!($l, $h) as u32) as LPARAM
    };
}

/*
 * This function grabs the MRU list from the registry and fills the combo-list
 * for the "Run" dialog above. fShowDefault is ignored if pszLatest != NULL.
 */
// FIXME: Part of this code should be part of some MRUList API,
// that is scattered amongst shell32, comctl32 (?!) and comdlg32.
unsafe extern "C" fn FillList(hCb: HWND, pszLatest: LPWSTR, cchStr: UINT, fShowDefault: BOOL) {
    // C writes to its own `cchStr` parameter (the `else` branch below) -- shadow it mutable.
    let mut cchStr = cchStr;

    let mut hkey: HKEY = core::ptr::null_mut();
    let mut pszList: *mut WCHAR = core::ptr::null_mut();
    let mut pszCmd: *mut WCHAR = core::ptr::null_mut();
    let mut pszTmp: *mut WCHAR = core::ptr::null_mut();
    let mut cMatch: WCHAR = 0;
    let mut cMax: WCHAR = 0x60;
    // WCHAR szIndex[2] = L"-";  (reuse the sealed literal instead of re-spelling it)
    let mut szIndex: [WCHAR; 2] = CPP_COM_WSTR_28276425D458;
    let mut cchLatest: UINT = 0;
    let mut dwType: DWORD = 0;
    let mut icList: DWORD = 0;
    let mut icCmd: DWORD = 0;
    // Reg*ValueEx/Reg*KeyEx return LSTATUS (signed LONG); the island's ERROR_SUCCESS is
    // (bindgen-inferred) unsigned DWORD. Real winerror.h headers make these interchangeable;
    // the island's split types do not, so `lRet` is kept in the DWORD domain from the point of
    // assignment on, matching how this file already compares Reg* results to ERROR_SUCCESS.
    let mut lRet: DWORD;
    let mut Nix: UINT;

    /*
     * Retrieve the string length of pszLatest and check whether its buffer size
     * (cchStr in number of characters) is large enough to add the terminating "\\1"
     * (and the NULL character).
     */
    if !pszLatest.is_null() {
        cchLatest = wcslen(pszLatest) as UINT;
        if cchStr < cchLatest + 2 + 1 {
            // TRACE: pszLatest buffer is not large enough to hold the MRU terminator.
            return;
        }
    } else {
        cchStr = 0;
    }

    SendMessageW(hCb, CB_RESETCONTENT, 0, 0);

    lRet = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        CPP_COM_WSTR_342C9756B5E3.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hkey,
        core::ptr::null_mut(),
    ) as DWORD;
    if lRet != ERROR_SUCCESS {
        // TRACE: unable to open or create the RunMRU key.
        return;
    }

    lRet = RegQueryValueExW(
        hkey,
        CPP_COM_WSTR_BBAEC67C90FF.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut icList,
    ) as DWORD;
    // `goto Continue;` (C line 825) jumps from inside this `if`'s true branch into the start of
    // its `else` branch -- a cross-branch merge, not a simple forward skip, so it cannot be
    // modelled with the single labelled `loop { break; }` this file otherwise uses for `goto`
    // (see EnablePrivilege's `'Quit:`). `goto_continue` stands in for the jump: it defaults to
    // "take the else/Continue: path" and is cleared only once the true branch fully succeeds.
    let mut goto_continue = true;
    if lRet == ERROR_SUCCESS && dwType == REG_SZ && icList > core::mem::size_of::<WCHAR>() as DWORD {
        pszList = HeapAlloc(GetProcessHeap(), 0, icList as SIZE_T) as *mut WCHAR;
        if pszList.is_null() {
            // TRACE: HeapAlloc failed to allocate icList bytes.
            // goto Continue;
        } else {
            *pszList = 0;

            lRet = RegQueryValueExW(
                hkey,
                CPP_COM_WSTR_BBAEC67C90FF.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                pszList as *mut u8,
                &mut icList,
            ) as DWORD;
            if lRet != ERROR_SUCCESS {
                // TRACE: unable to grab MRUList.
                *pszList = 0;
            }
            goto_continue = false;
        }
    }
    if goto_continue {
        // Continue:
        icList = core::mem::size_of::<WCHAR>() as DWORD;
        pszList = HeapAlloc(GetProcessHeap(), 0, icList as SIZE_T) as *mut WCHAR;
        if pszList.is_null() {
            // TRACE: HeapAlloc failed to allocate icList bytes.
            RegCloseKey(hkey);
            return;
        }
        *pszList = 0;
    }

    /* Convert the number of bytes from MRUList into number of characters (== number of indices) */
    icList /= core::mem::size_of::<WCHAR>() as DWORD;

    Nix = 0;
    while Nix < icList - 1 {
        if *pszList.add(Nix as usize) > cMax {
            cMax = *pszList.add(Nix as usize);
        }

        szIndex[0] = *pszList.add(Nix as usize);

        lRet = RegQueryValueExW(
            hkey,
            szIndex.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            core::ptr::null_mut(),
            &mut icCmd,
        ) as DWORD;
        if lRet != ERROR_SUCCESS || dwType != REG_SZ {
            // TRACE: unable to grab size of index.
            Nix += 1;
            continue;
        }

        if !pszCmd.is_null() {
            pszTmp = HeapReAlloc(GetProcessHeap(), 0, pszCmd.cast(), icCmd as SIZE_T) as *mut WCHAR;
            if pszTmp.is_null() {
                // TRACE: HeapReAlloc failed to reallocate icCmd bytes.
                Nix += 1;
                continue;
            }
            pszCmd = pszTmp;
        } else {
            pszCmd = HeapAlloc(GetProcessHeap(), 0, icCmd as SIZE_T) as *mut WCHAR;
            if pszCmd.is_null() {
                // TRACE: HeapAlloc failed to allocate icCmd bytes.
                Nix += 1;
                continue;
            }
        }

        lRet = RegQueryValueExW(
            hkey,
            szIndex.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pszCmd as *mut u8,
            &mut icCmd,
        ) as DWORD;
        if lRet != ERROR_SUCCESS {
            // TRACE: unable to grab index.
            Nix += 1;
            continue;
        }

        /*
         * Generally the command string will end up with "\\1".
         * Find the last backslash in the string and NULL-terminate.
         * Windows does not seem to check for what comes next, so that
         * a command of the form:
         *     c:\\my_dir\\myfile.exe
         * will be cut just after "my_dir", whereas a command of the form:
         *     c:\\my_dir\\myfile.exe\\1
         * will be cut just after "myfile.exe".
         */
        pszTmp = wcsrchr(pszCmd, '\\' as WCHAR);
        if !pszTmp.is_null() {
            *pszTmp = 0;
        }

        /*
         * In the following we try to add pszLatest to the MRU list.
         * We suppose that our caller has already correctly allocated
         * the string with enough space for us to append a "\\1".
         *
         * FIXME: TODO! (At the moment we don't append it!)
         */

        if !pszLatest.is_null() {
            if _wcsicmp(pszCmd, pszLatest) == 0 {
                SendMessageW(hCb, CB_INSERTSTRING, 0, pszCmd as LPARAM);
                SetWindowTextW(hCb, pszCmd);
                SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM!(0u32, -1i32 as DWORD));

                cMatch = *pszList.add(Nix as usize);
                memmove(
                    pszList.add(1) as PVOID,
                    pszList as LPCVOID,
                    (Nix as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
                );
                *pszList = cMatch;
                Nix += 1;
                continue;
            }
        }

        if icList - 1 != 26 || icList - 2 != Nix || cMatch != 0 || pszLatest.is_null() {
            SendMessageW(hCb, CB_ADDSTRING, 0, pszCmd as LPARAM);
            if Nix == 0 && fShowDefault != 0 {
                SetWindowTextW(hCb, pszCmd);
                SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM!(0u32, -1i32 as DWORD));
            }
        } else {
            SendMessageW(hCb, CB_INSERTSTRING, 0, pszLatest as LPARAM);
            SetWindowTextW(hCb, pszLatest);
            SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM!(0u32, -1i32 as DWORD));

            cMatch = *pszList.add(Nix as usize);
            memmove(
                pszList.add(1) as PVOID,
                pszList as LPCVOID,
                (Nix as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
            );
            *pszList = cMatch;
            szIndex[0] = cMatch;

            wcscpy(pszLatest.add(cchLatest as usize), CPP_COM_WSTR_2B4C92AAD354.as_ptr());
            RegSetValueExW(
                hkey,
                szIndex.as_ptr(),
                0,
                REG_SZ,
                pszLatest as *const u8,
                (cchLatest + 2 + 1) * core::mem::size_of::<WCHAR>() as UINT,
            );
            *pszLatest.add(cchLatest as usize) = 0;
        }

        Nix += 1;
    }

    if cMatch == 0 && !pszLatest.is_null() {
        SendMessageW(hCb, CB_INSERTSTRING, 0, pszLatest as LPARAM);
        SetWindowTextW(hCb, pszLatest);
        SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM!(0u32, -1i32 as DWORD));

        cMax += 1;
        cMatch = cMax;

        // `goto Cleanup;` here IS a simple forward skip (the label sits right after this whole
        // `if`, at the function's unconditional tail) -- same shape as EnablePrivilege's
        // `'Quit:`, so it gets the same single-iteration labelled-loop treatment.
        'Cleanup: loop {
            if !pszList.is_null() {
                icList += 1;
                pszTmp = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    pszList.cast(),
                    (icList as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                if pszTmp.is_null() {
                    // TRACE: HeapReAlloc failed to reallocate enough bytes.
                    break 'Cleanup; // goto Cleanup;
                }
                pszList = pszTmp;
            } else {
                icList += 1;
                pszList = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (icList as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                if pszList.is_null() {
                    // TRACE: HeapAlloc failed to allocate enough bytes.
                    break 'Cleanup; // goto Cleanup;
                }
            }

            memmove(
                pszList.add(1) as PVOID,
                pszList as LPCVOID,
                ((icList - 1) as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
            );
            *pszList = cMatch;
            szIndex[0] = cMatch;

            wcscpy(pszLatest.add(cchLatest as usize), CPP_COM_WSTR_2B4C92AAD354.as_ptr());
            RegSetValueExW(
                hkey,
                szIndex.as_ptr(),
                0,
                REG_SZ,
                pszLatest as *const u8,
                (cchLatest + 2 + 1) * core::mem::size_of::<WCHAR>() as UINT,
            );
            *pszLatest.add(cchLatest as usize) = 0;

            break;
        }
    }

    // Cleanup:
    RegSetValueExW(
        hkey,
        CPP_COM_WSTR_BBAEC67C90FF.as_ptr(),
        0,
        REG_SZ,
        pszList as *const u8,
        ((wcslen(pszList) + 1) * core::mem::size_of::<WCHAR>()) as DWORD,
    );

    HeapFree(GetProcessHeap(), 0, pszCmd.cast());
    HeapFree(GetProcessHeap(), 0, pszList.cast());

    RegCloseKey(hkey);
}

// ---- RunDlgProc: translated by hand; the paid worker was audit-rejected. ----
// RunDlgProc — translated from dll/win32/shell32/dialogs/dialogs.cpp lines 502-770.
//
// Target file: sdk/lib/rust/shell32-rust/src/dialogs/dialogs.rs. That file already has
// `use crate::prelude::*;` at its top and already defines `RUNFILEDLGPARAMS`, `LPFNOFN`,
// `MAKEWPARAM!`, `RunDlg_GetParentDir`, `EnableOkButtonFromEditContents`, `shell32_hInstance`,
// and the two deterministic UTF-16 literals this function needs
// (`CPP_COM_WSTR_731D7EC48EEF` = L" \t", `CPP_COM_WSTR_9E50FE27125E` = L"comdlg32") — none of
// that is repeated here. `FillList` is being translated in parallel with signature
// `unsafe extern "C" fn FillList(hCb: HWND, pszLatest: LPWSTR, cchStr: UINT, fShowDefault: BOOL)`.
//
// Only the four helpers below are genuinely new and belong at file scope alongside the
// existing `MAKEWPARAM!`.

// objbase.h `#define COINIT_APARTMENTTHREADED 0x2` (a `COINIT` enum member). The shell32
// bindgen island blocklists COM enums (see prelude.rs's own module header); the only surviving
// spelling is com-sys's bindgen-mangled `tagCOINIT_COINIT_APARTMENTTHREADED`, typed `tagCOINIT`
// rather than the plain `DWORD` `CoInitializeEx`'s second parameter actually takes. Named here
// instead of inlining `0x2` at the call site.
const COINIT_APARTMENTTHREADED: DWORD = 0x2;

// objbase.h `#define SUCCEEDED(hr) ((HRESULT)(hr) >= 0)`. Do NOT write this as
// `(hr as u32) >= 0` — that is unconditionally true and is a documented shipped bug in this
// same crate's shlexec.rs translation (see shell32/mod.rs's `shlexec` doc comment: "`SUCCEEDED
// (hr)` translated to `(hr as u32) >= 0`, which is unconditionally true, so a FAILED
// `AssocQueryStringW` still selected the 'runas' verb"). Compare the SIGNED HRESULT directly,
// matching the `fn SUCCEEDED(hr: HRESULT) -> bool { hr >= 0 }` helper already duplicated
// per-module in wine/shlmenu.rs, wine/shell32_main.rs, wine/shellord.rs, wine/pidl.rs, and
// wine/shellole.rs. dialogs.rs has no copy of its own yet.
#[inline]
fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}

// winuser.h `#define MAKELPARAM(l, h) ((LPARAM)(DWORD)MAKELONG(l, h))`. Same shape as this
// file's existing `MAKEWPARAM!` (which wraps the same `MAKELONG!` for the WPARAM slot); this
// one targets `LPARAM`, which is what `SendMessageW`'s 4th argument and `PostMessage`'s 4th
// argument need for `CB_SETEDITSEL`'s `MAKELPARAM(0, -1)` "select all" range.
macro_rules! MAKELPARAM {
    ($l:expr, $h:expr) => {
        (MAKELONG!($l as DWORD, $h as DWORD) as DWORD) as LPARAM
    };
}

// debug.h `ASSERT` — inert in a release build, matching the identical no-op `ASSERT!`
// already defined per-module in user_notification.rs / desktop_folder.rs / shell_link.rs
// (`macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }`); dialogs.rs has no copy of its own yet.
macro_rules! ASSERT {
    ($($arg:tt)*) => {{}};
}

/* Dialog procedure for RunFileDlg */
#[no_mangle]
pub unsafe extern "system" fn RunDlgProc(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut prfdp: *mut RUNFILEDLGPARAMS =
        GetWindowLongPtr(hwnd, DWLP_USER) as *mut RUNFILEDLGPARAMS;
    let mut hwndCombo: HWND;
    let mut hwndEdit: HWND;
    let mut ComboInfo: COMBOBOXINFO;

    match message {
        WM_INITDIALOG => {
            prfdp = lParam as *mut RUNFILEDLGPARAMS;
            SetWindowLongPtr(hwnd, DWLP_USER, prfdp as LONG_PTR);

            if !(*prfdp).lpstrTitle.is_null() {
                SetWindowTextW(hwnd, (*prfdp).lpstrTitle);
            }
            if !(*prfdp).lpstrDescription.is_null() {
                SetWindowTextW(GetDlgItem(hwnd, IDC_RUNDLG_DESCRIPTION as INT), (*prfdp).lpstrDescription);
            }
            if ((*prfdp).uFlags & RFF_NOBROWSE) != 0 {
                let browse: HWND = GetDlgItem(hwnd, IDC_RUNDLG_BROWSE as INT);
                ShowWindow(browse, SW_HIDE as i32);
                EnableWindow(browse, FALSE);
            }
            if ((*prfdp).uFlags & RFF_NOLABEL) != 0 {
                ShowWindow(GetDlgItem(hwnd, IDC_RUNDLG_LABEL as INT), SW_HIDE as i32);
            }
            if ((*prfdp).uFlags & RFF_NOSEPARATEMEM) != 0 {
                FIXME!("RFF_NOSEPARATEMEM not supported\n");
            }

            // Use the default Shell Run icon if no one is specified
            if (*prfdp).hIcon.is_null() {
                (*prfdp).hIcon = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_RUN));
            }
            //
            // NOTE: Starting Windows Vista, the "Run File" dialog gets a
            // title icon that remains the same as the default one, even if
            // the user specifies a custom icon.
            // Since we currently imitate Windows 2003, therefore do not show
            // any title icon.
            //
            // SendMessageW(hwnd, WM_SETICON, ICON_BIG, (*prfdp).hIcon as LPARAM);
            // SendMessageW(hwnd, WM_SETICON, ICON_SMALL, (*prfdp).hIcon as LPARAM);
            SendMessageW(GetDlgItem(hwnd, IDC_RUNDLG_ICON as INT), STM_SETICON, (*prfdp).hIcon as WPARAM, 0);

            hwndCombo = GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT);
            FillList(
                hwndCombo,
                core::ptr::null_mut(),
                0,
                (((*prfdp).uFlags & RFF_NODEFAULT) == 0) as BOOL,
            );
            EnableOkButtonFromEditContents(hwnd);

            ComboInfo = core::mem::zeroed();
            ComboInfo.cbSize = core::mem::size_of::<COMBOBOXINFO>() as DWORD;
            GetComboBoxInfo(hwndCombo, core::ptr::addr_of_mut!(ComboInfo));
            hwndEdit = ComboInfo.hwndItem;
            ASSERT!(IsWindow(hwndEdit));

            // SHAutoComplete needs co init
            (*prfdp).bCoInited =
                SUCCEEDED(CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED)) as BOOL;

            SHAutoComplete(hwndEdit, SHACF_FILESYSTEM | SHACF_FILESYS_ONLY | SHACF_URLALL);

            SetFocus(hwndCombo);
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            if (*prfdp).bCoInited != 0 {
                CoUninitialize();
            }
        }

        WM_COMMAND => {
            match LOWORD(wParam as DWORD) as UINT {
                IDOK => {
                    let lRet: LRESULT;
                    let htxt: HWND = GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT);
                    let ic: INT;
                    let psz: PWCHAR;
                    let pszExpanded: PWCHAR;
                    let mut parent: PWCHAR = core::ptr::null_mut();
                    let cchExpand: DWORD;
                    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
                    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD;
                    let mut nmrfd: NMRUNFILEDLGW = core::mem::zeroed();

                    ic = GetWindowTextLengthW(htxt);
                    if ic == 0 {
                        EndDialog(hwnd, IDCANCEL as INT_PTR);
                        return TRUE as INT_PTR;
                    }

                    // Allocate a new MRU entry, we need to add two characters
                    // for the terminating "\1" part, then the NULL character.
                    psz = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        ((ic as usize + 2 + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
                    ) as PWCHAR;
                    if psz.is_null() {
                        EndDialog(hwnd, IDCANCEL as INT_PTR);
                        return TRUE as INT_PTR;
                    }

                    GetWindowTextW(htxt, psz, ic + 1);
                    sei.hwnd = hwnd;
                    sei.nShow = SW_SHOWNORMAL as i32;
                    sei.lpFile = psz;
                    StrTrimW(psz, CPP_COM_WSTR_731D7EC48EEF.as_ptr());

                    if !wcschr(psz, '%' as WCHAR).is_null() {
                        cchExpand = ExpandEnvironmentStringsW(psz, core::ptr::null_mut(), 0);
                        pszExpanded = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            (cchExpand as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
                        ) as PWCHAR;
                        if pszExpanded.is_null() {
                            HeapFree(GetProcessHeap(), 0, psz.cast());
                            EndDialog(hwnd, IDCANCEL as INT_PTR);
                            return TRUE as INT_PTR;
                        }
                        ExpandEnvironmentStringsW(psz, pszExpanded, cchExpand);
                        StrTrimW(pszExpanded, CPP_COM_WSTR_731D7EC48EEF.as_ptr());
                    } else {
                        pszExpanded = psz;
                    }

                    //
                    // The precedence is the following: first the user-given
                    // current directory is used; if there is none, a current
                    // directory is computed if the RFF_CALCDIRECTORY is set,
                    // otherwise no current directory is defined.
                    //
                    let pszStartDir: LPCWSTR;
                    if !(*prfdp).lpstrDirectory.is_null() {
                        sei.lpDirectory = (*prfdp).lpstrDirectory;
                        pszStartDir = (*prfdp).lpstrDirectory;
                    } else if ((*prfdp).uFlags & RFF_CALCDIRECTORY) != 0 {
                        parent = RunDlg_GetParentDir(sei.lpFile);
                        sei.lpDirectory = parent;
                        parent = RunDlg_GetParentDir(pszExpanded);
                        pszStartDir = parent;
                    } else {
                        sei.lpDirectory = core::ptr::null();
                        pszStartDir = core::ptr::null();
                    }

                    // Hide the dialog for now on, we will show it up in case of retry
                    ShowWindow(hwnd, SW_HIDE as i32);

                    //
                    // As shown by manual tests on Windows, modifying the contents
                    // of the notification structure will not modify what the
                    // Run-Dialog will use for the nShow parameter. However the
                    // lpFile and lpDirectory pointers are set to the buffers used
                    // by the Run-Dialog, as a consequence they can be modified by
                    // the notification receiver, as long as it respects the lengths
                    // of the buffers (to avoid buffer overflows).
                    //
                    nmrfd.hdr.code = RFN_VALIDATE as UINT;
                    nmrfd.hdr.hwndFrom = hwnd;
                    nmrfd.hdr.idFrom = 0;
                    nmrfd.lpFile = pszExpanded;
                    nmrfd.lpDirectory = pszStartDir;
                    nmrfd.nShow = SW_SHOWNORMAL;

                    lRet = SendMessageW(
                        (*prfdp).hwndOwner,
                        WM_NOTIFY,
                        0,
                        core::ptr::addr_of_mut!(nmrfd.hdr) as LPARAM,
                    );

                    match lRet as u32 {
                        RF_CANCEL => {
                            EndDialog(hwnd, IDCANCEL as INT_PTR);
                        }

                        RF_OK => {
                            // We use SECL_NO_UI because we don't want to see
                            // errors here, but we will try again below and
                            // there we will output our errors.
                            if SUCCEEDED(crate::shell32::shlexec::ShellExecCmdLine(
                                hwnd,
                                pszExpanded,
                                pszStartDir,
                                SW_SHOWNORMAL as i32,
                                core::ptr::null_mut(),
                                SECL_ALLOW_NONEXE | SECL_NO_UI,
                            )) {
                                // Call GetWindowText again in case the contents of the edit box have changed.
                                GetWindowTextW(htxt, psz, ic + 1);
                                FillList(htxt, psz, (ic + 2 + 1) as UINT, FALSE);
                                EndDialog(hwnd, IDOK as INT_PTR);
                            } else if ShellExecuteExW(core::ptr::addr_of_mut!(sei)) != 0 {
                                // Call GetWindowText again in case the contents of the edit box have changed.
                                GetWindowTextW(htxt, psz, ic + 1);
                                FillList(htxt, psz, (ic + 2 + 1) as UINT, FALSE);
                                EndDialog(hwnd, IDOK as INT_PTR);
                            } else {
                                // Fall-back. C has no `break` before `case RF_RETRY:` here, so a
                                // failed RF_OK falls through into the RF_RETRY/default body below;
                                // Rust `match` never falls through, so that shared body is
                                // duplicated here and in the `RF_RETRY | _` arm.
                                SendMessageW(htxt, CB_SETEDITSEL, 0, MAKELPARAM!(0u32, -1i32 as DWORD));
                                // Show back the dialog
                                ShowWindow(hwnd, SW_SHOW as i32);
                            }
                        }

                        // Fall-back
                        RF_RETRY | _ => {
                            SendMessageW(htxt, CB_SETEDITSEL, 0, MAKELPARAM!(0u32, -1i32 as DWORD));
                            // Show back the dialog
                            ShowWindow(hwnd, SW_SHOW as i32);
                        }
                    }

                    HeapFree(GetProcessHeap(), 0, parent.cast());
                    HeapFree(GetProcessHeap(), 0, psz.cast());
                    if psz != pszExpanded {
                        HeapFree(GetProcessHeap(), 0, pszExpanded.cast());
                    }
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDC_RUNDLG_BROWSE => {
                    let mut ofnProc: LPFNOFN = None;
                    let mut szFName: [WCHAR; 1024] = [0u16; 1024];
                    let mut filter: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
                    let mut szCaption: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
                    let mut ofn: OPENFILENAMEW = core::mem::zeroed();

                    LoadStringW(
                        shell32_hInstance,
                        IDS_RUNDLG_BROWSE_FILTER,
                        filter.as_mut_ptr(),
                        _countof!(filter) as i32,
                    );
                    LoadStringW(
                        shell32_hInstance,
                        IDS_RUNDLG_BROWSE_CAPTION,
                        szCaption.as_mut_ptr(),
                        _countof!(szCaption) as i32,
                    );

                    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
                    ofn.hwndOwner = hwnd;
                    ofn.lpstrFilter = filter.as_ptr();
                    ofn.lpstrFile = szFName.as_mut_ptr();
                    ofn.nMaxFile = _countof!(szFName) - 1;
                    ofn.lpstrTitle = szCaption.as_ptr();
                    ofn.Flags = OFN_ENABLESIZING
                        | OFN_FILEMUSTEXIST
                        | OFN_HIDEREADONLY
                        | OFN_PATHMUSTEXIST
                        | OFN_EXPLORER;
                    ofn.lpstrInitialDir = (*prfdp).lpstrDirectory;

                    // C assigns inside the condition:
                    //   if (NULL == (hComdlg = LoadLibraryExW(L"comdlg32", NULL, 0)) ||
                    //       NULL == (ofnProc = (LPFNOFN)GetProcAddress(hComdlg, "GetOpenFileNameW")))
                    // Rust has no assignment-expression here, so this is split into two
                    // statements with the second reached only when the first `||` operand
                    // (matching C's short-circuit) doesn't already make the whole condition true.
                    let hComdlg: HMODULE =
                        LoadLibraryExW(CPP_COM_WSTR_9E50FE27125E.as_ptr(), core::ptr::null_mut(), 0);
                    let mut load_failed = hComdlg.is_null();
                    if !load_failed {
                        ofnProc = core::mem::transmute(GetProcAddress(
                            hComdlg,
                            b"GetOpenFileNameW\0".as_ptr() as *const i8,
                        ));
                        load_failed = ofnProc.is_none();
                    }
                    if load_failed {
                        ERR!(
                            "Couldn't get GetOpenFileName function entry (lib=%p, proc=%p)\n",
                            hComdlg,
                            ofnProc
                        );
                        // precomp.h:63 is `#define ShellMessageBoxW ShellMessageBoxWrapW`, so
                        // the C call here never reached the export of that name — shell32 does
                        // not import it, and the island's declaration links to nothing (i386:
                        // `_ShellMessageBoxW`, undefined). Third TU in this crate to hit it,
                        // after shlfileop.cpp and drive.cpp.
                        ShellMessageBoxWrapW(
                            shell32_hInstance,
                            hwnd,
                            MAKEINTRESOURCEW(IDS_RUNDLG_BROWSE_ERROR),
                            core::ptr::null(),
                            MB_OK | MB_ICONERROR,
                        );
                        return TRUE as INT_PTR;
                    }

                    if let Some(ofn_fn) = ofnProc {
                        if ofn_fn(core::ptr::addr_of_mut!(ofn)) != 0 {
                            SetFocus(GetDlgItem(hwnd, IDOK as INT));
                            SetWindowTextW(GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT), szFName.as_ptr());
                            SendMessageW(
                                GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT),
                                CB_SETEDITSEL,
                                0,
                                MAKELPARAM!(0u32, -1i32 as DWORD),
                            );
                            EnableOkButtonFromEditContents(hwnd);
                            SetFocus(GetDlgItem(hwnd, IDOK as INT));
                        }
                    }

                    FreeLibrary(hComdlg);

                    return TRUE as INT_PTR;
                }

                IDC_RUNDLG_EDITPATH => {
                    if HIWORD(wParam as DWORD) as UINT == CBN_EDITCHANGE {
                        EnableOkButtonFromEditContents(hwnd);
                    }

                    // Delay handling dropdown changes until the edit box has been updated.
                    if HIWORD(wParam as DWORD) as UINT == CBN_SELCHANGE {
                        PostMessage(
                            hwnd,
                            message,
                            MAKEWPARAM!(IDC_RUNDLG_EDITPATH, CBN_EDITCHANGE),
                            lParam,
                        );
                    }
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return TRUE as INT_PTR;
        }

        _ => {}
    }

    FALSE as INT_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *    common shell dialogs
//  *
//  * Copyright 2000 Juergen Schmied
//  * Copyright 2018 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  * Copyright 2021 Arnav Bhatt <arnavbhatt288@gmail.com>
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "precomp.h"
//
// typedef struct
// {
//     HWND hwndOwner;
//     HICON hIcon;
//     LPCWSTR lpstrDirectory;
//     LPCWSTR lpstrTitle;
//     LPCWSTR lpstrDescription;
//     UINT uFlags;
//     BOOL bCoInited;
// } RUNFILEDLGPARAMS;
//
// typedef struct
// {
//     BOOL bFriendlyUI;
//     BOOL bIsButtonHot[2];
//     HBITMAP hImageStrip;
//     HBRUSH hBrush;
//     HFONT hfFont;
//     BOOL bCloseDlg;
//     WNDPROC OldButtonProc;
// } LOGOFF_DLG_CONTEXT, *PLOGOFF_DLG_CONTEXT;
//
// typedef BOOL (WINAPI * LPFNOFN) (OPENFILENAMEW *);
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
// static INT_PTR __stdcall RunDlgProc(HWND, UINT, WPARAM, LPARAM);
// static void FillList(HWND, LPWSTR, UINT, BOOL);
//
//
// /*************************************************************************
//  * PickIconDlg                    [SHELL32.62]
//  *
//  */
//
// typedef struct
// {
//     HMODULE hLibrary;
//     HWND hDlgCtrl;
//     WCHAR szPath[MAX_PATH];
//     INT Index;
//     INT nIcons;
//     HICON *phIcons;
// } PICK_ICON_CONTEXT, *PPICK_ICON_CONTEXT;
//
// BOOL __stdcall EnumPickIconResourceProc(HMODULE hModule,
//     LPCWSTR lpszType,
//     LPWSTR lpszName,
//     LONG_PTR lParam)
// {
//     PPICK_ICON_CONTEXT pIconContext = PPICK_ICON_CONTEXT(lParam);
//     HWND hDlgCtrl = pIconContext->hDlgCtrl;
//
//     if (IS_INTRESOURCE(lpszName))
//         lParam = LOWORD(lpszName);
//     else
//         lParam = -1;
//
//     SendMessageW(hDlgCtrl, LB_ADDSTRING, 0, lParam);
//
//     return TRUE;
// }
//
// static void
// DestroyIconList(HWND hDlgCtrl, PPICK_ICON_CONTEXT pIconContext)
// {
//     int count;
//     int index;
//
//     count = SendMessageW(hDlgCtrl, LB_GETCOUNT, 0, 0);
//     if (count == LB_ERR)
//         return;
//
//     for(index = 0; index < count; index++)
//     {
//         DestroyIcon(pIconContext->phIcons[index]);
//         pIconContext->phIcons[index] = NULL;
//     }
// }
//
// static BOOL
// DoLoadIcons(HWND hwndDlg, PPICK_ICON_CONTEXT pIconContext, LPCWSTR pszFile)
// {
//     WCHAR szExpandedPath[MAX_PATH];
//
//     // Destroy previous icons
//     DestroyIconList(pIconContext->hDlgCtrl, pIconContext);
//     SendMessageW(pIconContext->hDlgCtrl, LB_RESETCONTENT, 0, 0);
//     delete[] pIconContext->phIcons;
//
//     // Store the path
//     StringCchCopyW(pIconContext->szPath, _countof!(pIconContext->szPath), pszFile);
//     ExpandEnvironmentStringsW(pszFile, szExpandedPath, _countof!(szExpandedPath));
//
//     // Load the module if possible
//     HMODULE hLibrary = LoadLibraryExW(szExpandedPath, NULL, LOAD_LIBRARY_AS_DATAFILE);
//     if (pIconContext->hLibrary)
//         FreeLibrary(pIconContext->hLibrary);
//     pIconContext->hLibrary = hLibrary;
//
//     if (pIconContext->hLibrary)
//     {
//         // Load the icons from the module
//         pIconContext->nIcons = ExtractIconExW(szExpandedPath, -1, NULL, NULL, 0);
//         pIconContext->phIcons = new HICON[pIconContext->nIcons];
//
//         if (ExtractIconExW(szExpandedPath, 0, pIconContext->phIcons, NULL, pIconContext->nIcons))
//         {
//             EnumResourceNamesW(pIconContext->hLibrary, RT_GROUP_ICON, EnumPickIconResourceProc, (LPARAM)pIconContext);
//         }
//         else
//         {
//             pIconContext->nIcons = 0;
//         }
//     }
//     else
//     {
//         // .ico file
//         pIconContext->nIcons = 1;
//         pIconContext->phIcons = new HICON[1];
//
//         if (ExtractIconExW(szExpandedPath, 0, pIconContext->phIcons, NULL, pIconContext->nIcons))
//         {
//             SendMessageW(pIconContext->hDlgCtrl, LB_ADDSTRING, 0, 0);
//         }
//         else
//         {
//             pIconContext->nIcons = 0;
//         }
//     }
//
//     SetDlgItemTextW(hwndDlg, IDC_EDIT_PATH, pIconContext->szPath);
//     SendMessageW(pIconContext->hDlgCtrl, LB_SETCURSEL, 0, 0);
//
//     if (pIconContext->nIcons == 0)
//     {
//         delete[] pIconContext->phIcons;
//         pIconContext->phIcons = NULL;
//     }
//
//     return (pIconContext->nIcons > 0);
// }
//
// static void NoIconsInFile(HWND hwndDlg, PPICK_ICON_CONTEXT pIconContext)
// {
//     // Show an error message
//     CStringW strText, strTitle(MAKEINTRESOURCEW(IDS_PICK_ICON_TITLE));
//     strText.Format(IDS_NO_ICONS, pIconContext->szPath);
//     MessageBoxW(hwndDlg, strText, strTitle, MB_ICONWARNING);
//
//     // Load the default icons
//     DoLoadIcons(hwndDlg, pIconContext, g_pszShell32);
// }
//
// // Icon size
// #define CX_ICON!()     GetSystemMetrics(SM_CXICON)
// #define CY_ICON!()     GetSystemMetrics(SM_CYICON)
//
// // Item size
// #define CX_ITEM!()     (CX_ICON!() + 4)
// #define CY_ITEM!()     (CY_ICON!() + 12)
//
// INT_PTR __stdcall PickIconProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     LPMEASUREITEMSTRUCT lpmis;
//     LPDRAWITEMSTRUCT lpdis;
//     HICON hIcon;
//     INT index, count;
//     WCHAR szText[MAX_PATH], szFilter[100];
//     OPENFILENAMEW ofn;
//     PPICK_ICON_CONTEXT pIconContext = (PPICK_ICON_CONTEXT)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pIconContext = (PPICK_ICON_CONTEXT)lParam;
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pIconContext);
//             pIconContext->hDlgCtrl = GetDlgItem(hwndDlg, IDC_PICKICON_LIST as INT);
//
//             SendMessageW(pIconContext->hDlgCtrl, LB_SETCOLUMNWIDTH, CX_ITEM!(), 0);
//
//             // Load the icons
//             if (!DoLoadIcons(hwndDlg, pIconContext, pIconContext->szPath))
//                 NoIconsInFile(hwndDlg, pIconContext);
//
//             // Set the selection
//             count = SendMessageW(pIconContext->hDlgCtrl, LB_GETCOUNT, 0, 0);
//             if (count != LB_ERR)
//             {
//                 if (pIconContext->Index < 0)
//                 {
//                     // A negative value will be interpreted as a negated resource ID.
//                     LPARAM lParam = -pIconContext->Index;
//                     pIconContext->Index = (INT)SendMessageW(pIconContext->hDlgCtrl, LB_FINDSTRINGEXACT, -1, lParam);
//                 }
//
//                 if (pIconContext->Index < 0 || count <= pIconContext->Index)
//                     pIconContext->Index = 0;
//
//                 SendMessageW(pIconContext->hDlgCtrl, LB_SETCURSEL, pIconContext->Index, 0);
//                 SendMessageW(pIconContext->hDlgCtrl, LB_SETTOPINDEX, pIconContext->Index, 0);
//             }
//
//             SHAutoComplete(GetDlgItem(hwndDlg, IDC_EDIT_PATH as INT), SHACF_DEFAULT);
//             return TRUE;
//         }
//
//         case WM_DESTROY:
//         {
//             DestroyIconList(pIconContext->hDlgCtrl, pIconContext);
//             delete[] pIconContext->phIcons;
//
//             if (pIconContext->hLibrary)
//                 FreeLibrary(pIconContext->hLibrary);
//             break;
//         }
//
//         case WM_COMMAND:
//             switch(LOWORD(wParam))
//             {
//             case IDOK:
//             {
//                 /* Check whether the path edit control has been modified; if so load the icons instead of validating */
//                 GetDlgItemTextW(hwndDlg, IDC_EDIT_PATH, szText, _countof!(szText));
//                 if (lstrcmpiW(szText, pIconContext->szPath))
//                 {
//                     if (!DoLoadIcons(hwndDlg, pIconContext, szText))
//                         NoIconsInFile(hwndDlg, pIconContext);
//                     break;
//                 }
//
//                 /* The path edit control has not been modified, return the selection */
//                 pIconContext->Index = (INT)SendMessageW(pIconContext->hDlgCtrl, LB_GETCURSEL, 0, 0);
//                 GetDlgItemTextW(hwndDlg, IDC_EDIT_PATH, pIconContext->szPath, _countof!(pIconContext->szPath));
//                 EndDialog(hwndDlg, 1);
//                 break;
//             }
//
//             case IDCANCEL:
//                 EndDialog(hwndDlg, 0);
//                 break;
//
//             case IDC_PICKICON_LIST:
//                 switch (HIWORD(wParam))
//                 {
//                     case LBN_SELCHANGE:
//                         InvalidateRect((HWND)lParam, NULL, TRUE);
//                         break;
//
//                     case LBN_DBLCLK:
//                         SendMessageW(hwndDlg, WM_COMMAND, MAKEWPARAM!(IDOK, 0), 0);
//                         break;
//                 }
//                 break;
//
//             case IDC_BUTTON_PATH:
//             {
//                 // Choose the module path
//                 CStringW strTitle;
//                 szText[0] = 0;
//                 szFilter[0] = 0;
//                 ZeroMemory(&ofn, sizeof(ofn));
//                 ofn.lStructSize = sizeof(ofn);
//                 ofn.hwndOwner = hwndDlg;
//                 ofn.lpstrFile = szText;
//                 ofn.nMaxFile = _countof!(szText);
//                 strTitle.LoadStringW(IDS_PICK_ICON_TITLE);
//                 ofn.lpstrTitle = strTitle;
//                 LoadStringW(shell32_hInstance, IDS_PICK_ICON_FILTER, szFilter, _countof!(szFilter));
//                 ofn.lpstrFilter = szFilter;
//                 if (!GetOpenFileNameW(&ofn))
//                     break;
//
//                 // Load the icons
//                 if (!DoLoadIcons(hwndDlg, pIconContext, szText))
//                     NoIconsInFile(hwndDlg, pIconContext);
//                 break;
//             }
//
//             default:
//                 break;
//             }
//             break;
//
//         case WM_MEASUREITEM:
//             lpmis = (LPMEASUREITEMSTRUCT)lParam;
//             lpmis->itemHeight = CY_ITEM!();
//             return TRUE;
//
//         case WM_DRAWITEM:
//         {
//             lpdis = (LPDRAWITEMSTRUCT)lParam;
//             if (lpdis->itemID == (UINT)-1)
//                 break;
//             switch (lpdis->itemAction) // FIXME: MSDN says that more than one of these can be set
//             {
//                 // FIXME: ODA_FOCUS
//                 case ODA_SELECT:
//                 case ODA_DRAWENTIRE:
//                 {
//                     index = SendMessageW(pIconContext->hDlgCtrl, LB_GETCURSEL, 0, 0);
//                     hIcon = pIconContext->phIcons[lpdis->itemID];
//
//                     if (lpdis->itemID == (UINT)index)
//                         FillRect(lpdis->hDC, &lpdis->rcItem, (HBRUSH)(COLOR_HIGHLIGHT + 1));
//                     else
//                         FillRect(lpdis->hDC, &lpdis->rcItem, (HBRUSH)(COLOR_WINDOW + 1));
//
//                     // Centering
//                     INT x = lpdis->rcItem.left + (CX_ITEM!() - CX_ICON!()) / 2;
//                     INT y = lpdis->rcItem.top + (CY_ITEM!() - CY_ICON!()) / 2;
//
//                     DrawIconEx(lpdis->hDC, x, y, hIcon, 0, 0, 0, NULL, DI_NORMAL);
//                     break;
//                 }
//             }
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// BOOL WINAPI PickIconDlg(
//     HWND hWndOwner,
//     LPWSTR lpstrFile,
//     UINT nMaxFile,
//     INT* lpdwIconIndex)
// {
//     CCoInit ComInit; // For SHAutoComplete (CORE-20030)
//     int res;
//     WCHAR szExpandedPath[MAX_PATH];
//
//     // Initialize the dialog
//     PICK_ICON_CONTEXT IconContext = {0};
//     IconContext.Index = *lpdwIconIndex;
//     StringCchCopyW(IconContext.szPath, _countof!(IconContext.szPath), lpstrFile);
//     ExpandEnvironmentStringsW(lpstrFile, szExpandedPath, _countof!(szExpandedPath));
//
//     if (!szExpandedPath[0] ||
//         GetFileAttributesW(szExpandedPath) == INVALID_FILE_ATTRIBUTES)
//     {
//         if (szExpandedPath[0])
//         {
//             // No such file
//             CStringW strText, strTitle(MAKEINTRESOURCEW(IDS_PICK_ICON_TITLE));
//             strText.Format(IDS_FILE_NOT_FOUND, lpstrFile);
//             MessageBoxW(hWndOwner, strText, strTitle, MB_ICONWARNING);
//         }
//
//         // Set the default value
//         StringCchCopyW(IconContext.szPath, _countof!(IconContext.szPath), g_pszShell32);
//     }
//
//     // Show the dialog
//     res = DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(IDD_PICK_ICON), hWndOwner, PickIconProc, (LPARAM)&IconContext);
//     if (res)
//     {
//         // Store the selected icon
//         StringCchCopyW(lpstrFile, nMaxFile, IconContext.szPath);
//         *lpdwIconIndex = IconContext.Index;
//     }
//
//     return res;
// }
//
// /*************************************************************************
//  * RunFileDlg                    [internal]
//  *
//  * The Unicode function that is available as ordinal 61 on Windows NT/2000/XP/...
//  */
// void WINAPI RunFileDlg(
//     HWND hWndOwner,
//     HICON hIcon,
//     LPCWSTR lpstrDirectory,
//     LPCWSTR lpstrTitle,
//     LPCWSTR lpstrDescription,
//     UINT uFlags)
// {
//     TRACE("\n");
//
//     RUNFILEDLGPARAMS rfdp;
//     rfdp.hwndOwner        = hWndOwner;
//     rfdp.hIcon            = hIcon;
//     rfdp.lpstrDirectory   = lpstrDirectory;
//     rfdp.lpstrTitle       = lpstrTitle;
//     rfdp.lpstrDescription = lpstrDescription;
//     rfdp.uFlags           = uFlags;
//
//     DialogBoxParamW(shell32_hInstance, MAKEINTRESOURCEW(IDD_RUN), hWndOwner, RunDlgProc, (LPARAM)&rfdp);
// }
//
//
// /* find the directory that contains the file being run */
// static LPWSTR RunDlg_GetParentDir(LPCWSTR cmdline)
// {
//     const WCHAR *src;
//     WCHAR *dest, *result, *result_end=NULL;
//
//     result = (WCHAR *)HeapAlloc(GetProcessHeap(), 0, sizeof(WCHAR)*(strlenW(cmdline)+5));
//
//     if (NULL == result)
//     {
//         TRACE("HeapAlloc couldn't allocate %d bytes\n", sizeof(WCHAR)*(strlenW(cmdline)+5));
//         return NULL;
//     }
//
//     src = cmdline;
//     dest = result;
//
//     if (*src == '"')
//     {
//         src++;
//         while (*src && *src != '"')
//         {
//             if (*src == '\\')
//                 result_end = dest;
//             *dest++ = *src++;
//         }
//     }
//     else {
//         while (*src)
//         {
//             if (isspaceW(*src))
//             {
//                 *dest = 0;
//                 if (INVALID_FILE_ATTRIBUTES != GetFileAttributesW(result))
//                     break;
//                 strcatW(dest, L".exe");
//                 if (INVALID_FILE_ATTRIBUTES != GetFileAttributesW(result))
//                     break;
//             }
//             else if (*src == '\\')
//                 result_end = dest;
//             *dest++ = *src++;
//         }
//     }
//
//     if (result_end)
//     {
//         *result_end = 0;
//         return result;
//     }
//     else
//     {
//         HeapFree(GetProcessHeap(), 0, result);
//         return NULL;
//     }
// }
//
// static void EnableOkButtonFromEditContents(HWND hwnd)
// {
//     BOOL Enable = FALSE;
//     INT Length, n;
//     HWND Edit = GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT);
//     Length = GetWindowTextLengthW(Edit);
//     if (Length > 0)
//     {
//         PWCHAR psz = (PWCHAR)HeapAlloc(GetProcessHeap(), 0, (Length + 1) * sizeof(WCHAR));
//         if (psz)
//         {
//             GetWindowTextW(Edit, psz, Length + 1);
//             for (n = 0; n < Length && !Enable; ++n)
//                 Enable = psz[n] != ' ';
//             HeapFree(GetProcessHeap(), 0, psz.cast());
//         }
//         else
//             Enable = TRUE;
//     }
//     EnableWindow(GetDlgItem(hwnd, IDOK), Enable);
// }
//
// /* Dialog procedure for RunFileDlg */
// static INT_PTR __stdcall RunDlgProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
// {
//     RUNFILEDLGPARAMS *prfdp = (RUNFILEDLGPARAMS *)GetWindowLongPtrW(hwnd, DWLP_USER);
//     HWND hwndCombo, hwndEdit;
//     COMBOBOXINFO ComboInfo;
//
//     switch (message)
//     {
//         case WM_INITDIALOG:
//             prfdp = (RUNFILEDLGPARAMS *)lParam;
//             SetWindowLongPtrW(hwnd, DWLP_USER, (LONG_PTR)prfdp);
//
//             if (prfdp->lpstrTitle)
//                 SetWindowTextW(hwnd, prfdp->lpstrTitle);
//             if (prfdp->lpstrDescription)
//                 SetWindowTextW(GetDlgItem(hwnd, IDC_RUNDLG_DESCRIPTION as INT), prfdp->lpstrDescription);
//             if (prfdp->uFlags & RFF_NOBROWSE)
//             {
//                 HWND browse = GetDlgItem(hwnd, IDC_RUNDLG_BROWSE as INT);
//                 ShowWindow(browse, SW_HIDE);
//                 EnableWindow(browse, FALSE);
//             }
//             if (prfdp->uFlags & RFF_NOLABEL)
//                 ShowWindow(GetDlgItem(hwnd, IDC_RUNDLG_LABEL as INT), SW_HIDE);
//             if (prfdp->uFlags & RFF_NOSEPARATEMEM)
//             {
//                 FIXME("RFF_NOSEPARATEMEM not supported\n");
//             }
//
//             /* Use the default Shell Run icon if no one is specified */
//             if (prfdp->hIcon == NULL)
//                 prfdp->hIcon = LoadIconW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_RUN));
//             /*
//              * NOTE: Starting Windows Vista, the "Run File" dialog gets a
//              * title icon that remains the same as the default one, even if
//              * the user specifies a custom icon.
//              * Since we currently imitate Windows 2003, therefore do not show
//              * any title icon.
//              */
//             // SendMessageW(hwnd, WM_SETICON, ICON_BIG, (LPARAM)prfdp->hIcon);
//             // SendMessageW(hwnd, WM_SETICON, ICON_SMALL, (LPARAM)prfdp->hIcon);
//             SendMessageW(GetDlgItem(hwnd, IDC_RUNDLG_ICON as INT), STM_SETICON, (WPARAM)prfdp->hIcon, 0);
//
//             hwndCombo = GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT);
//             FillList(hwndCombo, NULL, 0, (prfdp->uFlags & RFF_NODEFAULT) == 0);
//             EnableOkButtonFromEditContents(hwnd);
//
//             ComboInfo.cbSize = sizeof(ComboInfo);
//             GetComboBoxInfo(hwndCombo, &ComboInfo);
//             hwndEdit = ComboInfo.hwndItem;
//             ASSERT(::IsWindow(hwndEdit));
//
//             // SHAutoComplete needs co init
//             prfdp->bCoInited = SUCCEEDED(CoInitializeEx(NULL, COINIT_APARTMENTTHREADED));
//
//             SHAutoComplete(hwndEdit, SHACF_FILESYSTEM | SHACF_FILESYS_ONLY | SHACF_URLALL);
//
//             SetFocus(hwndCombo);
//             return TRUE;
//
//         case WM_DESTROY:
//             if (prfdp->bCoInited)
//                 CoUninitialize();
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                 {
//                     LRESULT lRet;
//                     HWND htxt = GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT);
//                     INT ic;
//                     WCHAR *psz, *pszExpanded, *parent = NULL;
//                     DWORD cchExpand;
//                     SHELLEXECUTEINFOW sei = { sizeof(sei) };
//                     NMRUNFILEDLGW nmrfd;
//
//                     ic = GetWindowTextLengthW(htxt);
//                     if (ic == 0)
//                     {
//                         EndDialog(hwnd, IDCANCEL as INT_PTR);
//                         return TRUE;
//                     }
//
//                     /*
//                      * Allocate a new MRU entry, we need to add two characters
//                      * for the terminating "\\1" part, then the NULL character.
//                      */
//                     psz = (WCHAR*)HeapAlloc(GetProcessHeap(), 0, (ic + 2 + 1)*sizeof(WCHAR));
//                     if (!psz)
//                     {
//                         EndDialog(hwnd, IDCANCEL as INT_PTR);
//                         return TRUE;
//                     }
//
//                     GetWindowTextW(htxt, psz, ic + 1);
//                     sei.hwnd = hwnd;
//                     sei.nShow = SW_SHOWNORMAL;
//                     sei.lpFile = psz;
//                     StrTrimW(psz, L" \t");
//
//                     if (wcschr(psz, L'%') != NULL)
//                     {
//                         cchExpand = ExpandEnvironmentStringsW(psz, NULL, 0);
//                         pszExpanded = (WCHAR*)HeapAlloc(GetProcessHeap(), 0, cchExpand * sizeof(WCHAR));
//                         if (!pszExpanded)
//                         {
//                             HeapFree(GetProcessHeap(), 0, psz.cast());
//                             EndDialog(hwnd, IDCANCEL as INT_PTR);
//                             return TRUE;
//                         }
//                         ExpandEnvironmentStringsW(psz, pszExpanded, cchExpand);
//                         StrTrimW(pszExpanded, L" \t");
//                     }
//                     else
//                     {
//                         pszExpanded = psz;
//                     }
//
//                     /*
//                      * The precedence is the following: first the user-given
//                      * current directory is used; if there is none, a current
//                      * directory is computed if the RFF_CALCDIRECTORY is set,
//                      * otherwise no current directory is defined.
//                      */
//                     LPCWSTR pszStartDir;
//                     if (prfdp->lpstrDirectory)
//                     {
//                         sei.lpDirectory = prfdp->lpstrDirectory;
//                         pszStartDir = prfdp->lpstrDirectory;
//                     }
//                     else if (prfdp->uFlags & RFF_CALCDIRECTORY)
//                     {
//                         sei.lpDirectory = parent = RunDlg_GetParentDir(sei.lpFile);
//                         pszStartDir = parent = RunDlg_GetParentDir(pszExpanded);
//                     }
//                     else
//                     {
//                         sei.lpDirectory = NULL;
//                         pszStartDir = NULL;
//                     }
//
//                     /* Hide the dialog for now on, we will show it up in case of retry */
//                     ShowWindow(hwnd, SW_HIDE);
//
//                     /*
//                      * As shown by manual tests on Windows, modifying the contents
//                      * of the notification structure will not modify what the
//                      * Run-Dialog will use for the nShow parameter. However the
//                      * lpFile and lpDirectory pointers are set to the buffers used
//                      * by the Run-Dialog, as a consequence they can be modified by
//                      * the notification receiver, as long as it respects the lengths
//                      * of the buffers (to avoid buffer overflows).
//                      */
//                     nmrfd.hdr.code = RFN_VALIDATE;
//                     nmrfd.hdr.hwndFrom = hwnd;
//                     nmrfd.hdr.idFrom = 0;
//                     nmrfd.lpFile = pszExpanded;
//                     nmrfd.lpDirectory = pszStartDir;
//                     nmrfd.nShow = SW_SHOWNORMAL;
//
//                     lRet = SendMessageW(prfdp->hwndOwner, WM_NOTIFY, 0, (LPARAM)&nmrfd.hdr);
//
//                     switch (lRet)
//                     {
//                         case RF_CANCEL:
//                             EndDialog(hwnd, IDCANCEL as INT_PTR);
//                             break;
//
//                         case RF_OK:
//                             /* We use SECL_NO_UI because we don't want to see
//                              * errors here, but we will try again below and
//                              * there we will output our errors. */
//                             if (SUCCEEDED(ShellExecCmdLine(hwnd, pszExpanded, pszStartDir, SW_SHOWNORMAL, NULL,
//                                                            SECL_ALLOW_NONEXE | SECL_NO_UI)))
//                             {
//                                 /* Call GetWindowText again in case the contents of the edit box have changed. */
//                                 GetWindowTextW(htxt, psz, ic + 1);
//                                 FillList(htxt, psz, ic + 2 + 1, FALSE);
//                                 EndDialog(hwnd, IDOK as INT_PTR);
//                                 break;
//                             }
//                             else if (ShellExecuteExW(&sei))
//                             {
//                                 /* Call GetWindowText again in case the contents of the edit box have changed. */
//                                 GetWindowTextW(htxt, psz, ic + 1);
//                                 FillList(htxt, psz, ic + 2 + 1, FALSE);
//                                 EndDialog(hwnd, IDOK as INT_PTR);
//                                 break;
//                             }
//
//                         /* Fall-back */
//                         case RF_RETRY:
//                         default:
//                             SendMessageW(htxt, CB_SETEDITSEL, 0, MAKELPARAM (0, -1));
//                             /* Show back the dialog */
//                             ShowWindow(hwnd, SW_SHOW);
//                             break;
//                     }
//
//                     HeapFree(GetProcessHeap(), 0, parent.cast());
//                     HeapFree(GetProcessHeap(), 0, psz.cast());
//                     if (psz != pszExpanded)
//                         HeapFree(GetProcessHeap(), 0, pszExpanded.cast());
//                     return TRUE;
//                 }
//
//                 case IDCANCEL:
//                     EndDialog(hwnd, IDCANCEL as INT_PTR);
//                     return TRUE;
//
//                 case IDC_RUNDLG_BROWSE:
//                 {
//                     HMODULE hComdlg = NULL;
//                     LPFNOFN ofnProc = NULL;
//                     WCHAR szFName[1024] = {0};
//                     WCHAR filter[MAX_PATH], szCaption[MAX_PATH];
//                     OPENFILENAMEW ofn;
//
//                     LoadStringW(shell32_hInstance, IDS_RUNDLG_BROWSE_FILTER, filter, _countof!(filter));
//                     LoadStringW(shell32_hInstance, IDS_RUNDLG_BROWSE_CAPTION, szCaption, _countof!(szCaption));
//
//                     ZeroMemory(&ofn, sizeof(ofn));
//                     ofn.lStructSize = sizeof(ofn);
//                     ofn.hwndOwner = hwnd;
//                     ofn.lpstrFilter = filter;
//                     ofn.lpstrFile = szFName;
//                     ofn.nMaxFile = _countof!(szFName) - 1;
//                     ofn.lpstrTitle = szCaption;
//                     ofn.Flags = OFN_ENABLESIZING | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_EXPLORER;
//                     ofn.lpstrInitialDir = prfdp->lpstrDirectory;
//
//                     if (NULL == (hComdlg = LoadLibraryExW(L"comdlg32", NULL, 0)) ||
//                         NULL == (ofnProc = (LPFNOFN)GetProcAddress(hComdlg, "GetOpenFileNameW")))
//                     {
//                         ERR("Couldn't get GetOpenFileName function entry (lib=%p, proc=%p)\n", hComdlg, ofnProc);
//                         ShellMessageBoxW(shell32_hInstance, hwnd, MAKEINTRESOURCEW(IDS_RUNDLG_BROWSE_ERROR), NULL, MB_OK | MB_ICONERROR);
//                         return TRUE;
//                     }
//
//                     if (ofnProc(&ofn))
//                     {
//                         SetFocus(GetDlgItem(hwnd, IDOK as INT));
//                         SetWindowTextW(GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT), szFName);
//                         SendMessageW(GetDlgItem(hwnd, IDC_RUNDLG_EDITPATH as INT), CB_SETEDITSEL, 0, MAKELPARAM(0, -1));
//                         EnableOkButtonFromEditContents(hwnd);
//                         SetFocus(GetDlgItem(hwnd, IDOK as INT));
//                     }
//
//                     FreeLibrary(hComdlg);
//
//                     return TRUE;
//                 }
//                 case IDC_RUNDLG_EDITPATH:
//                 {
//                     if (HIWORD(wParam) == CBN_EDITCHANGE)
//                         EnableOkButtonFromEditContents(hwnd);
//
//                     // Delay handling dropdown changes until the edit box has been updated.
//                     if (HIWORD(wParam) == CBN_SELCHANGE)
//                         PostMessage(hwnd, message, MAKELONG(IDC_RUNDLG_EDITPATH, CBN_EDITCHANGE), lParam);
//                     return TRUE;
//                 }
//             }
//             return TRUE;
//     }
//     return FALSE;
// }
//
// /*
//  * This function grabs the MRU list from the registry and fills the combo-list
//  * for the "Run" dialog above. fShowDefault is ignored if pszLatest != NULL.
//  */
// // FIXME: Part of this code should be part of some MRUList API,
// // that is scattered amongst shell32, comctl32 (?!) and comdlg32.
// static void FillList(HWND hCb, LPWSTR pszLatest, UINT cchStr, BOOL fShowDefault)
// {
//     HKEY hkey;
//     WCHAR *pszList = NULL, *pszCmd = NULL, *pszTmp = NULL, cMatch = 0, cMax = 0x60;
//     WCHAR szIndex[2] = L"-";
//     UINT cchLatest;
//     DWORD dwType, icList = 0, icCmd = 0;
//     LRESULT lRet;
//     UINT Nix;
//
//     /*
//      * Retrieve the string length of pszLatest and check whether its buffer size
//      * (cchStr in number of characters) is large enough to add the terminating "\\1"
//      * (and the NULL character).
//      */
//     if (pszLatest)
//     {
//         cchLatest = wcslen(pszLatest);
//         if (cchStr < cchLatest + 2 + 1)
//         {
//             TRACE("pszLatest buffer is not large enough (%d) to hold the MRU terminator.\n", cchStr);
//             return;
//         }
//     }
//     else
//     {
//         cchStr = 0;
//     }
//
//     SendMessageW(hCb, CB_RESETCONTENT, 0, 0);
//
//     lRet = RegCreateKeyExW(HKEY_CURRENT_USER,
//                            L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RunMRU",
//                            0, NULL, REG_OPTION_NON_VOLATILE, KEY_ALL_ACCESS, NULL, &hkey, NULL);
//     if (lRet != ERROR_SUCCESS)
//     {
//         TRACE("Unable to open or create the RunMRU key, error %d\n", GetLastError());
//         return;
//     }
//
//     lRet = RegQueryValueExW(hkey, L"MRUList", NULL, &dwType, NULL, &icList);
//     if (lRet == ERROR_SUCCESS && dwType == REG_SZ && icList > sizeof(WCHAR))
//     {
//         pszList = (WCHAR*)HeapAlloc(GetProcessHeap(), 0, icList);
//         if (!pszList)
//         {
//             TRACE("HeapAlloc failed to allocate %d bytes\n", icList);
//             goto Continue;
//         }
//         pszList[0] = L'\0';
//
//         lRet = RegQueryValueExW(hkey, L"MRUList", NULL, NULL, (LPBYTE)pszList, &icList);
//         if (lRet != ERROR_SUCCESS)
//         {
//             TRACE("Unable to grab MRUList, error %d\n", GetLastError());
//             pszList[0] = L'\0';
//         }
//     }
//     else
//     {
// Continue:
//         icList = sizeof(WCHAR);
//         pszList = (WCHAR*)HeapAlloc(GetProcessHeap(), 0, icList);
//         if (!pszList)
//         {
//             TRACE("HeapAlloc failed to allocate %d bytes\n", icList);
//             RegCloseKey(hkey);
//             return;
//         }
//         pszList[0] = L'\0';
//     }
//
//     /* Convert the number of bytes from MRUList into number of characters (== number of indices) */
//     icList /= sizeof(WCHAR);
//
//     for (Nix = 0; Nix < icList - 1; Nix++)
//     {
//         if (pszList[Nix] > cMax)
//             cMax = pszList[Nix];
//
//         szIndex[0] = pszList[Nix];
//
//         lRet = RegQueryValueExW(hkey, szIndex, NULL, &dwType, NULL, &icCmd);
//         if (lRet != ERROR_SUCCESS || dwType != REG_SZ)
//         {
//             TRACE("Unable to grab size of index, error %d\n", GetLastError());
//             continue;
//         }
//
//         if (pszCmd)
//         {
//             pszTmp = (WCHAR*)HeapReAlloc(GetProcessHeap(), 0, pszCmd, icCmd);
//             if (!pszTmp)
//             {
//                 TRACE("HeapReAlloc failed to reallocate %d bytes\n", icCmd);
//                 continue;
//             }
//             pszCmd = pszTmp;
//         }
//         else
//         {
//             pszCmd = (WCHAR*)HeapAlloc(GetProcessHeap(), 0, icCmd);
//             if (!pszCmd)
//             {
//                 TRACE("HeapAlloc failed to allocate %d bytes\n", icCmd);
//                 continue;
//             }
//         }
//
//         lRet = RegQueryValueExW(hkey, szIndex, NULL, NULL, (LPBYTE)pszCmd, &icCmd);
//         if (lRet != ERROR_SUCCESS)
//         {
//             TRACE("Unable to grab index, error %d\n", GetLastError());
//             continue;
//         }
//
//         /*
//          * Generally the command string will end up with "\\1".
//          * Find the last backslash in the string and NULL-terminate.
//          * Windows does not seem to check for what comes next, so that
//          * a command of the form:
//          *     c:\\my_dir\\myfile.exe
//          * will be cut just after "my_dir", whereas a command of the form:
//          *     c:\\my_dir\\myfile.exe\\1
//          * will be cut just after "myfile.exe".
//          */
//         pszTmp = wcsrchr(pszCmd, L'\\');
//         if (pszTmp)
//             *pszTmp = L'\0';
//
//         /*
//          * In the following we try to add pszLatest to the MRU list.
//          * We suppose that our caller has already correctly allocated
//          * the string with enough space for us to append a "\\1".
//          *
//          * FIXME: TODO! (At the moment we don't append it!)
//          */
//
//         if (pszLatest)
//         {
//             if (_wcsicmp(pszCmd, pszLatest) == 0)
//             {
//                 SendMessageW(hCb, CB_INSERTSTRING, 0, (LPARAM)pszCmd);
//                 SetWindowTextW(hCb, pszCmd);
//                 SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM(0, -1));
//
//                 cMatch = pszList[Nix];
//                 memmove(&pszList[1], pszList, Nix * sizeof(WCHAR));
//                 pszList[0] = cMatch;
//                 continue;
//             }
//         }
//
//         if (icList - 1 != 26 || icList - 2 != Nix || cMatch || pszLatest == NULL)
//         {
//             SendMessageW(hCb, CB_ADDSTRING, 0, (LPARAM)pszCmd);
//             if (!Nix && fShowDefault)
//             {
//                 SetWindowTextW(hCb, pszCmd);
//                 SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM(0, -1));
//             }
//         }
//         else
//         {
//             SendMessageW(hCb, CB_INSERTSTRING, 0, (LPARAM)pszLatest);
//             SetWindowTextW(hCb, pszLatest);
//             SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM(0, -1));
//
//             cMatch = pszList[Nix];
//             memmove(&pszList[1], pszList, Nix * sizeof(WCHAR));
//             pszList[0] = cMatch;
//             szIndex[0] = cMatch;
//
//             wcscpy(&pszLatest[cchLatest], L"\\1");
//             RegSetValueExW(hkey, szIndex, 0, REG_SZ, (LPBYTE)pszLatest, (cchLatest + 2 + 1) * sizeof(WCHAR));
//             pszLatest[cchLatest] = L'\0';
//         }
//     }
//
//     if (!cMatch && pszLatest != NULL)
//     {
//         SendMessageW(hCb, CB_INSERTSTRING, 0, (LPARAM)pszLatest);
//         SetWindowTextW(hCb, pszLatest);
//         SendMessageW(hCb, CB_SETEDITSEL, 0, MAKELPARAM (0, -1));
//
//         cMatch = ++cMax;
//
//         if (pszList)
//         {
//             pszTmp = (WCHAR*)HeapReAlloc(GetProcessHeap(), 0, pszList, (++icList) * sizeof(WCHAR));
//             if (!pszTmp)
//             {
//                 TRACE("HeapReAlloc failed to reallocate enough bytes\n");
//                 goto Cleanup;
//             }
//             pszList = pszTmp;
//         }
//         else
//         {
//             pszList = (WCHAR*)HeapAlloc(GetProcessHeap(), 0, (++icList) * sizeof(WCHAR));
//             if (!pszList)
//             {
//                 TRACE("HeapAlloc failed to allocate enough bytes\n");
//                 goto Cleanup;
//             }
//         }
//
//         memmove(&pszList[1], pszList, (icList - 1) * sizeof(WCHAR));
//         pszList[0] = cMatch;
//         szIndex[0] = cMatch;
//
//         wcscpy(&pszLatest[cchLatest], L"\\1");
//         RegSetValueExW(hkey, szIndex, 0, REG_SZ, (LPBYTE)pszLatest, (cchLatest + 2 + 1) * sizeof(WCHAR));
//         pszLatest[cchLatest] = L'\0';
//     }
//
// Cleanup:
//     RegSetValueExW(hkey, L"MRUList", 0, REG_SZ, (LPBYTE)pszList, (wcslen(pszList) + 1) * sizeof(WCHAR));
//
//     HeapFree(GetProcessHeap(), 0, pszCmd);
//     HeapFree(GetProcessHeap(), 0, pszList);
//
//     RegCloseKey(hkey);
// }
//
//
// /*************************************************************************
//  * ConfirmDialog                [internal]
//  *
//  * Put up a confirm box, return TRUE if the user confirmed
//  */
// static BOOL ConfirmDialog(HWND hWndOwner, UINT PromptId, UINT TitleId)
// {
//     WCHAR Prompt[256];
//     WCHAR Title[256];
//
//     LoadStringW(shell32_hInstance, PromptId, Prompt, _countof!(Prompt));
//     LoadStringW(shell32_hInstance, TitleId, Title, _countof!(Title));
//     return MessageBoxW(hWndOwner, Prompt, Title, MB_YESNO | MB_ICONQUESTION) == IDYES;
// }
//
// typedef HRESULT (WINAPI *tShellDimScreen)(IUnknown** Unknown, HWND* hWindow);
//
// BOOL
// CallShellDimScreen(IUnknown** pUnknown, HWND* hWindow)
// {
//     static tShellDimScreen ShellDimScreen;
//     static BOOL Initialized = FALSE;
//     if (!Initialized)
//     {
//         HMODULE mod = LoadLibraryW(L"msgina.dll");
//         ShellDimScreen = (tShellDimScreen)GetProcAddress(mod, (LPCSTR)16);
//         Initialized = TRUE;
//     }
//
//     HRESULT hr = E_FAIL;
//     if (ShellDimScreen)
//         hr = ShellDimScreen(pUnknown, hWindow);
//     return SUCCEEDED(hr);
// }
//
//
// /* Used to get the shutdown privilege */
// static BOOL
// EnablePrivilege(LPCWSTR lpszPrivilegeName, BOOL bEnablePrivilege)
// {
//     BOOL   Success;
//     HANDLE hToken;
//     TOKEN_PRIVILEGES tp;
//
//     Success = OpenProcessToken(GetCurrentProcess(),
//                                TOKEN_ADJUST_PRIVILEGES,
//                                &hToken);
//     if (!Success) return Success;
//
//     Success = LookupPrivilegeValueW(NULL,
//                                     lpszPrivilegeName,
//                                     &tp.Privileges[0].Luid);
//     if (!Success) goto Quit;
//
//     tp.PrivilegeCount = 1;
//     tp.Privileges[0].Attributes = (bEnablePrivilege ? SE_PRIVILEGE_ENABLED : 0);
//
//     Success = AdjustTokenPrivileges(hToken, FALSE, &tp, 0, NULL, NULL);
//
// Quit:
//     CloseHandle(hToken);
//     return Success;
// }
//
// /*************************************************************************
//  * RestartDialogEx                [SHELL32.730]
//  */
//
// int WINAPI RestartDialogEx(HWND hWndOwner, LPCWSTR lpwstrReason, DWORD uFlags, DWORD uReason)
// {
//     TRACE("(%p)\n", hWndOwner);
//
//     CComPtr<IUnknown> fadeHandler;
//     HWND parent;
//
//     if (!CallShellDimScreen(&fadeHandler, &parent))
//         parent = hWndOwner;
//
//     /* FIXME: use lpwstrReason */
//     if (ConfirmDialog(parent, IDS_RESTART_PROMPT, IDS_RESTART_TITLE))
//     {
//         EnablePrivilege(L"SeShutdownPrivilege", TRUE);
//         ExitWindowsEx(EWX_REBOOT, uReason);
//         EnablePrivilege(L"SeShutdownPrivilege", FALSE);
//     }
//
//     return 0;
// }
//
// /* Functions and macros used for fancy log off dialog box */
// #define IS_PRODUCT_VERSION_WORKSTATION          0x300
// #define FRIENDLY_LOGOFF_IS_NOT_ENFORCED         0x0
//
// #define FONT_POINT_SIZE                 13
//
// #define DARK_GREY_COLOR                 RGB(244, 244, 244)
// #define LIGHT_GREY_COLOR                RGB(38, 38, 38)
//
// /* Bitmap's size for buttons */
// #define CX_BITMAP                       33
// #define CY_BITMAP                       33
//
// #define NUMBER_OF_BUTTONS               2
//
// /* After determining the button as well as its state paint the image strip bitmap using these predefined positions */
// #define BUTTON_SWITCH_USER              0
// #define BUTTON_SWITCH_USER_PRESSED      (CY_BITMAP + BUTTON_SWITCH_USER)
// #define BUTTON_SWITCH_USER_FOCUSED      (CY_BITMAP + BUTTON_SWITCH_USER_PRESSED)
// #define BUTTON_LOG_OFF                  (CY_BITMAP + BUTTON_SWITCH_USER_FOCUSED)
// #define BUTTON_LOG_OFF_PRESSED          (CY_BITMAP + BUTTON_LOG_OFF)
// #define BUTTON_LOG_OFF_FOCUSED          (CY_BITMAP + BUTTON_LOG_OFF_PRESSED)
// #define BUTTON_SWITCH_USER_DISABLED     (CY_BITMAP + BUTTON_LOG_OFF_FOCUSED) // Temporary
//
// /* For bIsButtonHot */
// #define LOG_OFF_BUTTON_HOT              0
// #define SWITCH_USER_BUTTON_HOT          1
//
// BOOL DrawIconOnOwnerDrawnButtons(DRAWITEMSTRUCT* pdis, PLOGOFF_DLG_CONTEXT pContext)
// {
//     BOOL bRet = FALSE;
//     HDC hdcMem = NULL;
//     HBITMAP hbmOld = NULL;
//     int y = 0;
//     RECT rect;
//
//     hdcMem = CreateCompatibleDC(pdis->hDC);
//     hbmOld = (HBITMAP)SelectObject(hdcMem, pContext->hImageStrip);
//     rect = pdis->rcItem;
//
//     /* Check the button ID for relevant bitmap to be used */
//     switch (pdis->CtlID)
//     {
//         case IDC_LOG_OFF_BUTTON:
//         {
//             switch (pdis->itemAction)
//             {
//                 case ODA_DRAWENTIRE:
//                 case ODA_FOCUS:
//                 case ODA_SELECT:
//                 {
//                     y = BUTTON_LOG_OFF;
//                     if (pdis->itemState & ODS_SELECTED)
//                     {
//                         y = BUTTON_LOG_OFF_PRESSED;
//                     }
//                     else if (pContext->bIsButtonHot[LOG_OFF_BUTTON_HOT] || (pdis->itemState & ODS_FOCUS))
//                     {
//                         y = BUTTON_LOG_OFF_FOCUSED;
//                     }
//                     break;
//                 }
//             }
//             break;
//         }
//
//         case IDC_SWITCH_USER_BUTTON:
//         {
//             switch (pdis->itemAction)
//             {
//                 case ODA_DRAWENTIRE:
//                 case ODA_FOCUS:
//                 case ODA_SELECT:
//                 {
//                     y = BUTTON_SWITCH_USER;
//                     if (pdis->itemState & ODS_SELECTED)
//                     {
//                         y = BUTTON_SWITCH_USER_PRESSED;
//                     }
//                     else if (pContext->bIsButtonHot[SWITCH_USER_BUTTON_HOT] || (pdis->itemState & ODS_FOCUS))
//                     {
//                         y = BUTTON_SWITCH_USER_FOCUSED;
//                     }
//
//                     /*
//                      * Since switch user functionality isn't implemented yet therefore the button has been disabled
//                      * temporarily hence show the disabled state
//                      */
//                     else if (pdis->itemState & ODS_DISABLED)
//                     {
//                         y = BUTTON_SWITCH_USER_DISABLED;
//                     }
//                     break;
//                 }
//             }
//             break;
//         }
//     }
//
//     /* Draw it on the required button */
//     bRet = BitBlt(pdis->hDC,
//                   (rect.right - rect.left - CX_BITMAP) / 2,
//                   (rect.bottom - rect.top - CY_BITMAP) / 2,
//                   CX_BITMAP, CY_BITMAP, hdcMem, 0, y, SRCCOPY);
//
//     SelectObject(hdcMem, hbmOld);
//     DeleteDC(hdcMem);
//
//     return bRet;
// }
//
// INT_PTR __stdcall OwnerDrawButtonSubclass(HWND hButton, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     PLOGOFF_DLG_CONTEXT pContext;
//     pContext = (PLOGOFF_DLG_CONTEXT)GetWindowLongPtrW(GetParent(hButton), GWLP_USERDATA);
//
//     int buttonID = GetDlgCtrlID(hButton);
//
//     switch (uMsg)
//     {
//         case WM_MOUSEMOVE:
//         {
//             HWND hwndTarget = NULL;
//             POINT pt = {GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam)};;
//
//             if (GetCapture() != hButton)
//             {
//                 SetCapture(hButton);
//
//                 switch (buttonID)
//                 {
//                     case IDC_LOG_OFF_BUTTON:
//                     {
//                         pContext->bIsButtonHot[LOG_OFF_BUTTON_HOT] = TRUE;
//                         break;
//                     }
//                     case IDC_SWITCH_USER_BUTTON:
//                     {
//                         pContext->bIsButtonHot[SWITCH_USER_BUTTON_HOT] = TRUE;
//                         break;
//                     }
//                 }
//                 SetCursor(LoadCursorW(NULL, IDC_HAND));
//             }
//
//             ClientToScreen(hButton, &pt);
//             hwndTarget = WindowFromPoint(pt);
//
//             if (hwndTarget != hButton)
//             {
//                 ReleaseCapture();
//
//                 switch (buttonID)
//                 {
//                     case IDC_LOG_OFF_BUTTON:
//                     {
//                         pContext->bIsButtonHot[LOG_OFF_BUTTON_HOT] = FALSE;
//                         break;
//                     }
//                     case IDC_SWITCH_USER_BUTTON:
//                     {
//                         pContext->bIsButtonHot[SWITCH_USER_BUTTON_HOT] = FALSE;
//                         break;
//                     }
//                 }
//             }
//             InvalidateRect(hButton, NULL, FALSE);
//             break;
//         }
//
//         /* Whenever one of the buttons gets the keyboard focus, set it as default button */
//         case WM_SETFOCUS:
//         {
//             SendMessageW(GetParent(hButton), DM_SETDEFID, buttonID, 0);
//             break;
//         }
//
//         /* Otherwise, set IDCANCEL as default button */
//         case WM_KILLFOCUS:
//         {
//             SendMessageW(GetParent(hButton), DM_SETDEFID, IDCANCEL, 0);
//             break;
//         }
//     }
//     return CallWindowProcW(pContext->OldButtonProc, hButton, uMsg, wParam, lParam);
// }
//
// VOID CreateToolTipForButtons(int controlID, int detailID, HWND hDlg, int titleID)
// {
//     HWND hwndTool = NULL, hwndTip = NULL;
//     WCHAR szBuffer[256];
//     TTTOOLINFOW tool;
//
//     hwndTool = GetDlgItem(hDlg, controlID);
//
//     tool.cbSize = sizeof(tool);
//     tool.hwnd = hDlg;
//     tool.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
//     tool.uId = (UINT_PTR)hwndTool;
//
//     /* Create the tooltip */
//     hwndTip = CreateWindowExW(0, TOOLTIPS_CLASSW, NULL,
//                               WS_POPUP | TTS_ALWAYSTIP | TTS_BALLOON,
//                               CW_USEDEFAULT, CW_USEDEFAULT,
//                               CW_USEDEFAULT, CW_USEDEFAULT,
//                               hDlg, NULL, shell32_hInstance, NULL);
//
//     /* Associate the tooltip with the tool. */
//     LoadStringW(shell32_hInstance, detailID, szBuffer, _countof!(szBuffer));
//     tool.lpszText = szBuffer;
//     SendMessageW(hwndTip, TTM_ADDTOOLW, 0, (LPARAM)&tool);
//     LoadStringW(shell32_hInstance, titleID, szBuffer, _countof!(szBuffer));
//     SendMessageW(hwndTip, TTM_SETTITLEW, TTI_NONE, (LPARAM)szBuffer);
//     SendMessageW(hwndTip, TTM_SETMAXTIPWIDTH, 0, 250);
// }
//
// VOID EndFriendlyDialog(HWND hwnd, PLOGOFF_DLG_CONTEXT pContext)
// {
//     DeleteObject(pContext->hBrush);
//     DeleteObject(pContext->hImageStrip);
//     DeleteObject(pContext->hfFont);
//
//     /* Remove the subclass from the buttons */
//     for (int i = 0; i < NUMBER_OF_BUTTONS; i++)
//     {
//         SetWindowLongPtrW(GetDlgItem(hwnd, IDC_LOG_OFF_BUTTON + i),
//                           GWLP_WNDPROC,
//                           (LONG_PTR)pContext->OldButtonProc);
//     }
// }
//
// static BOOL IsFriendlyUIActive(VOID)
// {
//     DWORD dwType = 0, dwValue = 0, dwSize = 0;
//     HKEY hKey = NULL;
//     LONG lRet = 0;
//
//     lRet = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                          L"SYSTEM\\CurrentControlSet\\Control\\Windows",
//                          0,
//                          KEY_QUERY_VALUE,
//                          &hKey);
//     if (lRet != ERROR_SUCCESS)
//         return FALSE;
//
//     /* First check an optional ReactOS specific override, that Windows does not check.
//        We use this to allow users pairing 'Server'-configuration with FriendlyLogoff.
//        Otherwise users would have to change CSDVersion or LogonType (side-effects AppCompat) */
//     dwValue = 0;
//     dwSize = sizeof(dwValue);
//     lRet = RegQueryValueExW(hKey,
//                             L"EnforceFriendlyLogoff",
//                             NULL,
//                             &dwType,
//                             (LPBYTE)&dwValue,
//                             &dwSize);
//
//     if (lRet == ERROR_SUCCESS && dwType == REG_DWORD && dwValue != FRIENDLY_LOGOFF_IS_NOT_ENFORCED)
//     {
//         RegCloseKey(hKey);
//         return TRUE;
//     }
//
//     /* Check product version number */
//     dwValue = 0;
//     dwSize = sizeof(dwValue);
//     lRet = RegQueryValueExW(hKey,
//                             L"CSDVersion",
//                             NULL,
//                             &dwType,
//                             (LPBYTE)&dwValue,
//                             &dwSize);
//     RegCloseKey(hKey);
//
//     if (lRet != ERROR_SUCCESS || dwType != REG_DWORD || dwValue != IS_PRODUCT_VERSION_WORKSTATION)
//     {
//         /* Allow Friendly UI only on Workstation */
//         return FALSE;
//     }
//
//     /* Check LogonType value */
//     lRet = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                          L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
//                          0,
//                          KEY_QUERY_VALUE,
//                          &hKey);
//     if (lRet != ERROR_SUCCESS)
//         return FALSE;
//
//     dwValue = 0;
//     dwSize = sizeof(dwValue);
//     lRet = RegQueryValueExW(hKey,
//                             L"LogonType",
//                             NULL,
//                             &dwType,
//                             (LPBYTE)&dwValue,
//                             &dwSize);
//     RegCloseKey(hKey);
//
//     if (lRet != ERROR_SUCCESS || dwType != REG_DWORD)
//         return FALSE;
//
//     return (dwValue != 0);
// }
//
// static VOID FancyLogoffOnInit(HWND hwnd, PLOGOFF_DLG_CONTEXT pContext)
// {
//     HDC hdc = NULL;
//     LONG lfHeight = NULL;
//
//     hdc = GetDC(NULL);
//     lfHeight = -MulDiv(FONT_POINT_SIZE, GetDeviceCaps(hdc, LOGPIXELSY), 72);
//     ReleaseDC(NULL, hdc);
//     pContext->hfFont = CreateFontW(lfHeight, 0, 0, 0, FW_MEDIUM, FALSE, 0, 0, 0, 0, 0, 0, 0, L"MS Shell Dlg");
//     SendDlgItemMessageW(hwnd, IDC_LOG_OFF_TEXT_STATIC, WM_SETFONT, (WPARAM)pContext->hfFont, TRUE);
//
//     pContext->hBrush = CreateSolidBrush(DARK_GREY_COLOR);
//
//     pContext->hImageStrip = LoadBitmapW(shell32_hInstance, MAKEINTRESOURCEW(IDB_IMAGE_STRIP));
//
//     /* Gather old button func */
//     pContext->OldButtonProc = (WNDPROC)GetWindowLongPtrW(GetDlgItem(hwnd, IDC_LOG_OFF_BUTTON as INT), GWLP_WNDPROC);
//
//     /* Set bIsButtonHot to false, create tooltips for each buttons and subclass the buttons */
//     for (int i = 0; i < NUMBER_OF_BUTTONS; i++)
//     {
//         pContext->bIsButtonHot[i] = FALSE;
//         SetWindowLongPtrW(GetDlgItem(hwnd, IDC_LOG_OFF_BUTTON + i),
//                           GWLP_WNDPROC,
//                           (LONG_PTR)OwnerDrawButtonSubclass);
//         CreateToolTipForButtons(IDC_LOG_OFF_BUTTON + i,
//                                 IDS_LOG_OFF_DESC + i,
//                                 hwnd,
//                                 IDS_LOG_OFF_TITLE + i);
//     }
// }
//
// /*************************************************************************
//  * LogOffDialogProc
//  *
//  * NOTES: Used to make the Log Off dialog work
//  */
// INT_PTR __stdcall LogOffDialogProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     DRAWITEMSTRUCT* pdis = (DRAWITEMSTRUCT*)lParam;
//     PLOGOFF_DLG_CONTEXT pContext;
//     pContext = (PLOGOFF_DLG_CONTEXT)GetWindowLongPtrW(hwnd, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pContext = (PLOGOFF_DLG_CONTEXT)lParam;
//             SetWindowLongPtrW(hwnd, GWLP_USERDATA, (LONG_PTR)pContext);
//
//             if (pContext->bFriendlyUI)
//                 FancyLogoffOnInit(hwnd, pContext);
//             return TRUE;
//         }
//
//         case WM_DESTROY:
//             if (pContext->bFriendlyUI)
//                 EndFriendlyDialog(hwnd, pContext);
//             return TRUE;
//
//         case WM_ACTIVATE:
//         {
//             /*
//              * If the user deactivates the log-off dialog (it loses its focus
//              * while the dialog is not being closed), then destroy the dialog
//              * and cancel user logoff.
//              */
//             if (LOWORD(wParam) == WA_INACTIVE)
//             {
//                 if (!pContext->bCloseDlg)
//                 {
//                     pContext->bCloseDlg = TRUE;
//                     EndDialog(hwnd, IDCANCEL as INT_PTR);
//                 }
//             }
//             return FALSE;
//         }
//
//         case WM_CLOSE:
//             pContext->bCloseDlg = TRUE;
//             EndDialog(hwnd, IDCANCEL as INT_PTR);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_LOG_OFF_BUTTON:
//                 case IDOK:
//                     ExitWindowsEx(EWX_LOGOFF, 0);
//                     break;
//
//                 case IDCANCEL:
//                     pContext->bCloseDlg = TRUE;
//                     EndDialog(hwnd, IDCANCEL as INT_PTR);
//                     break;
//             }
//             break;
//
//         case WM_CTLCOLORSTATIC:
//         {
//             /* Either make background transparent or fill it with color for required static controls */
//             HDC hdcStatic = (HDC)wParam;
//             UINT StaticID = (UINT)GetWindowLongPtrW((HWND)lParam, GWL_ID);
//
//             switch (StaticID)
//             {
//                 case IDC_LOG_OFF_TEXT_STATIC:
//                    SetTextColor(hdcStatic, DARK_GREY_COLOR);
//                    SetBkMode(hdcStatic, TRANSPARENT);
//                    return (INT_PTR)GetStockObject(HOLLOW_BRUSH);
//
//                 case IDC_LOG_OFF_STATIC:
//                 case IDC_SWITCH_USER_STATIC:
//                     SetTextColor(hdcStatic, LIGHT_GREY_COLOR);
//                     SetBkMode(hdcStatic, TRANSPARENT);
//                     return (LONG_PTR)pContext->hBrush;
//             }
//             return FALSE;
//         }
//         break;
//
//         case WM_DRAWITEM:
//         {
//             /* Draw bitmaps on required buttons */
//             switch (pdis->CtlID)
//             {
//                 case IDC_LOG_OFF_BUTTON:
//                 case IDC_SWITCH_USER_BUTTON:
//                     return DrawIconOnOwnerDrawnButtons(pdis, pContext) as INT_PTR;
//             }
//         }
//         break;
//
//         default:
//             break;
//     }
//     return FALSE;
// }
//
// /*************************************************************************
//  * LogoffWindowsDialog  [SHELL32.54]
//  */
//
// extern "C" int WINAPI LogoffWindowsDialog(HWND hWndOwner)
// {
//     CComPtr<IUnknown> fadeHandler;
//     HWND parent = NULL;
//     DWORD LogoffDialogID = IDD_LOG_OFF;
//     LOGOFF_DLG_CONTEXT Context = {0};
//
//     if (!CallShellDimScreen(&fadeHandler, &parent))
//         parent = hWndOwner;
//
//     Context.bFriendlyUI = IsFriendlyUIActive();
//     if (Context.bFriendlyUI)
//     {
//         LogoffDialogID = IDD_LOG_OFF_FANCY;
//     }
//
//     DialogBoxParamW(shell32_hInstance,
//                     MAKEINTRESOURCEW(LogoffDialogID),
//                     parent,
//                     LogOffDialogProc,
//                     (LPARAM)&Context);
//     return 0;
// }
//
// /*************************************************************************
//  * RestartDialog                [SHELL32.59]
//  */
//
// int WINAPI RestartDialog(HWND hWndOwner, LPCWSTR lpstrReason, DWORD uFlags)
// {
//     return RestartDialogEx(hWndOwner, lpstrReason, uFlags, 0);
// }
//
// /*************************************************************************
//  * ExitWindowsDialog_backup
//  *
//  * NOTES
//  *     Used as a backup solution to shutdown the OS in case msgina.dll
//  *     somehow cannot be found.
//  */
// VOID ExitWindowsDialog_backup(HWND hWndOwner)
// {
//     TRACE("(%p)\n", hWndOwner);
//
//     if (ConfirmDialog(hWndOwner, IDS_SHUTDOWN_PROMPT, IDS_SHUTDOWN_TITLE))
//     {
//         EnablePrivilege(L"SeShutdownPrivilege", TRUE);
//         ExitWindowsEx(EWX_SHUTDOWN, 0);
//         EnablePrivilege(L"SeShutdownPrivilege", FALSE);
//     }
// }
//
// /*************************************************************************
//  * ExitWindowsDialog                [SHELL32.60]
//  *
//  * NOTES
//  *     exported by ordinal
//  */
// /*
//  * TODO:
//  * - Implement the ability to show either the Welcome Screen or the classic dialog boxes based upon the
//  *   registry value: SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\LogonType.
//  */
// void WINAPI ExitWindowsDialog(HWND hWndOwner)
// {
//     typedef DWORD (WINAPI *ShellShFunc)(HWND hWndParent, LPCWSTR pUserName, DWORD dwExcludeOptions);
//     HINSTANCE msginaDll = LoadLibraryW(L"msgina.dll");
//
//     TRACE("(%p)\n", hWndOwner);
//
//     CComPtr<IUnknown> fadeHandler;
//     HWND parent;
//     if (!CallShellDimScreen(&fadeHandler, &parent))
//         parent = hWndOwner;
//
//     /* If the DLL cannot be found for any reason, then it simply uses a
//        dialog box to ask if the user wants to shut down the computer. */
//     if (!msginaDll)
//     {
//         TRACE("Unable to load msgina.dll.\n");
//         ExitWindowsDialog_backup(parent);
//         return;
//     }
//
//     ShellShFunc pShellShutdownDialog = (ShellShFunc)GetProcAddress(msginaDll, "ShellShutdownDialog");
//     if (pShellShutdownDialog)
//     {
//         /* Actually call the function */
//         DWORD returnValue = pShellShutdownDialog(parent, NULL, 0);
//
//         switch (returnValue)
//         {
//         case 0x01: /* Log off user */
//         {
//             ExitWindowsEx(EWX_LOGOFF, 0);
//             break;
//         }
//         case 0x02: /* Shut down */
//         {
//             EnablePrivilege(L"SeShutdownPrivilege", TRUE);
//             ExitWindowsEx(EWX_SHUTDOWN, 0);
//             EnablePrivilege(L"SeShutdownPrivilege", FALSE);
//             break;
//         }
//         case 0x03: /* Install Updates/Shutdown (?) */
//         {
//             break;
//         }
//         case 0x04: /* Reboot */
//         {
//             EnablePrivilege(L"SeShutdownPrivilege", TRUE);
//             ExitWindowsEx(EWX_REBOOT, 0);
//             EnablePrivilege(L"SeShutdownPrivilege", FALSE);
//             break;
//         }
//         case 0x10: /* Sleep */
//         {
//             if (IsPwrSuspendAllowed())
//             {
//                 EnablePrivilege(L"SeShutdownPrivilege", TRUE);
//                 SetSuspendState(FALSE, FALSE, FALSE);
//                 EnablePrivilege(L"SeShutdownPrivilege", FALSE);
//             }
//             break;
//         }
//         case 0x40: /* Hibernate */
//         {
//             if (IsPwrHibernateAllowed())
//             {
//                 EnablePrivilege(L"SeShutdownPrivilege", TRUE);
//                 SetSuspendState(TRUE, FALSE, TRUE);
//                 EnablePrivilege(L"SeShutdownPrivilege", FALSE);
//             }
//             break;
//         }
//         /* If the option is any other value */
//         default:
//             break;
//         }
//     }
//     else
//     {
//         /* If the function cannot be found, then revert to using the backup solution */
//         TRACE("Unable to find the 'ShellShutdownDialog' function");
//         ExitWindowsDialog_backup(parent);
//     }
//
//     FreeLibrary(msginaDll);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
