use crate::regedit::listview::CreateListView;
use crate::regedit::treeview::CreateTreeView;
use crate::regedit::listview::DestroyListView;
use crate::regedit::main::DestroyMainMenu;
use crate::regedit::treeview::DestroyTreeView;
use crate::regedit::treeview::GetItemPath;
use crate::regedit::listview::IsDefaultValue;
use crate::regedit::listview::ListWndNotifyProc;
use crate::regedit::edit::QueryStringValue;
use crate::regedit::listview::RefreshListView;
use crate::regedit::treeview::SelectNode;
use crate::regedit::treeview::TreeWndNotifyProc;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_111FD57AB4CB: [u16; 20] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0043, 0x004F, 0x004E, 0x0046, 0x0049, 0x0047, 0x0000,
];
pub static RUSTCONV_WSTR_2742CD8F7DE1: [u16; 5] = [0x0045u16, 0x0064, 0x0069, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_4E0719E0EA39: [u16; 28] = [
    0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0020, 0x0048, 0x004B,
    0x0045, 0x0059, 0x002C, 0x0020, 0x0050, 0x004C, 0x0045, 0x0041, 0x0053, 0x0045,
    0x0020, 0x0052, 0x0045, 0x0050, 0x004F, 0x0052, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_50C930882486: [u16; 19] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x004C, 0x004F, 0x0043, 0x0041, 0x004C,
    0x005F, 0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E, 0x0045, 0x0000,
];
pub static RUSTCONV_WSTR_5677AD35830D: [u16; 12] = [
    0x0048u16, 0x004B, 0x0043, 0x0052, 0x005C, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044,
    0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_5CDE252B65C7: [u16; 14] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0044, 0x0059, 0x004E, 0x005F, 0x0044,
    0x0041, 0x0054, 0x0041, 0x0000,
];
pub static RUSTCONV_WSTR_69D435449EDE: [u16; 7] = [0x0025u16, 0x0073, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_77F55C91EF10: [u16; 6] = [0x0048u16, 0x004B, 0x0043, 0x0052, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_87D16031626E: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x004C, 0x0041, 0x0053, 0x0053,
    0x0045, 0x0053, 0x005F, 0x0052, 0x004F, 0x004F, 0x0054, 0x0000,
];
pub static RUSTCONV_WSTR_89F38F0B7FC9: [u16; 5] = [0x0048u16, 0x004B, 0x004C, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_8D7C97326263: [u16; 7] = [0x0042u16, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9C98935C198E: [u16; 11] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0053,
    0x0000,
];
pub static RUSTCONV_WSTR_AFF3436D7481: [u16; 6] = [0x0043u16, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_BA2342D760A2: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_CC2767AFEE4B: [u16; 2] = [0x00BBu16, 0x0000];
pub static RUSTCONV_WSTR_D02D0BCFE5D4: [u16; 5] = [0x0048u16, 0x004B, 0x0043, 0x0055, 0x0000];
pub static RUSTCONV_WSTR_D0EE0F231465: [u16; 18] = [
    0x0048u16, 0x004B, 0x0045, 0x0059, 0x005F, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045,
    0x004E, 0x0054, 0x005F, 0x0055, 0x0053, 0x0045, 0x0052, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_pChildWnd: *mut ChildWnd = ::core::ptr::null::<ChildWnd>() as *mut ChildWnd;

static mut last_split: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);

#[no_mangle]
pub static mut SizingPattern: HBITMAP = ::core::ptr::null::<HBITMAP__>() as *mut HBITMAP__;

#[no_mangle]
pub static mut SizingBrush: HBRUSH = ::core::ptr::null::<HBRUSH__>() as *mut HBRUSH__;

#[no_mangle]
pub static mut Suggestions: [WCHAR; 256] = [0; 256];

#[no_mangle]

#[no_mangle]
pub static mut g_DummyEnumStrings: DummyEnumStrings = unsafe {
    DummyEnumStrings {
        lpVtbl: &raw const g_DummyEnumStringsVtbl as *mut DummyEnumStringsVtbl,
    }
};

#[repr(C)]
#[allow(non_snake_case)]
pub struct DummyEnumStringsVtbl {
    pub QI: LPVOID,
    pub AddRef: LPVOID,
    pub Release: LPVOID,
    pub Next: LPVOID,
    pub Skip: LPVOID,
    pub Reset: LPVOID,
    pub Clone: LPVOID,
}
pub type PDummyEnumStringsVtbl = *mut DummyEnumStringsVtbl;

pub static mut g_DummyEnumStringsVtbl: DummyEnumStringsVtbl = DummyEnumStringsVtbl {
    QI: DummyEnumStringsQI as *const () as LPVOID,
    AddRef: DummyEnumStringsAddRefRelease as *const () as LPVOID,
    Release: DummyEnumStringsAddRefRelease as *const () as LPVOID,
    Next: DummyEnumStringsNext as *const () as LPVOID,
    Skip: DummyEnumStringsSkip as *const () as LPVOID,
    Reset: DummyEnumStringsReset as *const () as LPVOID,
    Clone: DummyEnumStringsClone as *const () as LPVOID,
};

#[repr(C)]
pub struct DummyEnumStrings {
    pub lpVtbl: *mut DummyEnumStringsVtbl,
}
pub type PDummyEnumStrings = *mut DummyEnumStrings;

unsafe extern "system" fn DummyEnumStringsQI(This: LPVOID, riid: REFIID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    if !ppv.is_null() {
        *ppv = core::ptr::null_mut();
    }
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IEnumString)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *ppv = This;
        return S_OK;
    }
    return E_NOINTERFACE;
}
unsafe extern "system" fn DummyEnumStringsAddRefRelease(This: LPVOID) -> ULONG {
    1
}
unsafe extern "system" fn DummyEnumStringsNext(This: LPVOID, celt: ULONG, parr: *mut LPWSTR, pceltFetched: *mut ULONG) -> HRESULT {
    if !pceltFetched.is_null() {
        *pceltFetched = 0;
    }
    return S_FALSE;
}
unsafe extern "system" fn DummyEnumStringsSkip(This: LPVOID, celt: ULONG) -> HRESULT {
    S_OK
}
unsafe extern "system" fn DummyEnumStringsReset(This: LPVOID) -> HRESULT {
    S_OK
}
unsafe extern "system" fn DummyEnumStringsClone(
    This: LPVOID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn get_root_key_name(hRootKey: HKEY) -> LPCWSTR {
    if hRootKey == HKEY_CLASSES_ROOT { return RUSTCONV_WSTR_87D16031626E.as_ptr(); }
    if hRootKey == HKEY_CURRENT_USER { return RUSTCONV_WSTR_D0EE0F231465.as_ptr(); }
    if hRootKey == HKEY_LOCAL_MACHINE { return RUSTCONV_WSTR_50C930882486.as_ptr(); }
    if hRootKey == HKEY_USERS { return RUSTCONV_WSTR_9C98935C198E.as_ptr(); }
    if hRootKey == HKEY_CURRENT_CONFIG { return RUSTCONV_WSTR_111FD57AB4CB.as_ptr(); }
    if hRootKey == HKEY_DYN_DATA { return RUSTCONV_WSTR_5CDE252B65C7.as_ptr(); }

    return RUSTCONV_WSTR_4E0719E0EA39.as_ptr();
}

unsafe extern "C" fn ClampSplitBarX(hWnd: HWND, x: INT) -> INT {
    let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
    return min!(max!(x, (SPLIT_MIN as i32)), rc.right - (SPLIT_MIN as i32));
}
#[no_mangle]
pub unsafe extern "C" fn ResizeWnd(mut cx: ::core::ffi::c_int, mut cy: ::core::ffi::c_int) {
    let mut hdwp: HDWP = BeginDeferWindowPos(4);
    let mut rt: RECT = core::mem::zeroed();
    let mut rs: RECT = core::mem::zeroed();
    let mut rb: RECT = core::mem::zeroed();
    let nButtonWidth: i32 = 44;
    let nButtonHeight: i32 = 22;
    let mut cyEdge: i32 = GetSystemMetrics((SM_CYEDGE) as i32);
    let uFlags: UINT = (SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOCOPYBITS) as UINT;

    cy = 0;
    if IsWindowVisible(hStatusBar) != 0 {
        GetWindowRect(hStatusBar, core::ptr::addr_of_mut!(rs));
        cy = rs.bottom - rs.top;
    }

    GetWindowRect((*g_pChildWnd).hAddressBtnWnd, core::ptr::addr_of_mut!(rb));

    GetClientRect((*g_pChildWnd).hWnd, core::ptr::addr_of_mut!(rt));
    RedrawWindow(
        (*g_pChildWnd).hWnd,
        core::ptr::addr_of_mut!(rt) as LPCRECT,
        core::ptr::null_mut(),
        (RDW_INVALIDATE | RDW_NOCHILDREN) as UINT,
    );

    (*g_pChildWnd).nSplitPos = ClampSplitBarX((*g_pChildWnd).hWnd, (*g_pChildWnd).nSplitPos);

    cx = (*g_pChildWnd).nSplitPos + (SPLIT_WIDTH / 2) as i32;
    if !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            (*g_pChildWnd).hAddressBarWnd,
            core::ptr::null_mut(),
            rt.left,
            rt.top,
            rt.right - rt.left - nButtonWidth,
            nButtonHeight,
            uFlags,
        );
    }
    if !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            (*g_pChildWnd).hAddressBtnWnd,
            core::ptr::null_mut(),
            rt.right - nButtonWidth,
            rt.top,
            nButtonWidth,
            nButtonHeight,
            uFlags,
        );
    }
    if !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            (*g_pChildWnd).hTreeWnd,
            core::ptr::null_mut(),
            rt.left,
            rt.top + nButtonHeight + cyEdge,
            (*g_pChildWnd).nSplitPos - (SPLIT_WIDTH / 2) as i32 - rt.left,
            rt.bottom - rt.top - cy - 2 * cyEdge,
            uFlags,
        );
    }
    if !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            (*g_pChildWnd).hListWnd,
            core::ptr::null_mut(),
            rt.left + cx,
            rt.top + nButtonHeight + cyEdge,
            rt.right - cx,
            rt.bottom - rt.top - cy - 2 * cyEdge,
            uFlags,
        );
    }
    if !hdwp.is_null() {
        EndDeferWindowPos(hdwp);
    }
}

unsafe extern "C" fn draw_splitbar(hWnd: HWND, x: ::core::ffi::c_int) {
    let mut rt: RECT = core::mem::zeroed();
    let mut OldObj: HGDIOBJ = core::ptr::null_mut();
    let hdc: HDC = GetDCEx(hWnd, core::ptr::null_mut(), DCX_CACHE as DWORD);

    if SizingPattern.is_null() {
        let Pattern: [DWORD; 4] = [
            0x5555AAAA as DWORD,
            0x5555AAAA as DWORD,
            0x5555AAAA as DWORD,
            0x5555AAAA as DWORD,
        ];
        SizingPattern = CreateBitmap(
            8 as INT,
            8 as INT,
            1 as UINT,
            1 as UINT,
            core::ptr::addr_of!(Pattern) as *const ::core::ffi::c_void,
        );
    }
    if SizingBrush.is_null() {
        SizingBrush = CreatePatternBrush(SizingPattern);
    }

    GetWindowRect((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(rt));
    MapWindowPoints(
        core::ptr::null_mut(),
        hWnd,
        core::ptr::addr_of_mut!(rt) as LPPOINT,
        (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>()) as UINT,
    );

    rt.left = (x - (SPLIT_WIDTH / 2) as i32) as LONG;
    rt.right = (x + (SPLIT_WIDTH / 2) as i32 + 1) as LONG;
    OldObj = SelectObject(hdc, SizingBrush as HGDIOBJ);
    PatBlt(
        hdc,
        rt.left as ::core::ffi::c_int,
        rt.top as ::core::ffi::c_int,
        (rt.right - rt.left) as ::core::ffi::c_int,
        (rt.bottom - rt.top) as ::core::ffi::c_int,
        PATINVERT as DWORD,
    );
    SelectObject(hdc, OldObj);
    ReleaseDC(hWnd, hdc);
}

unsafe extern "C" fn finish_splitbar(hWnd: HWND, x: ::core::ffi::c_int) {
    let mut rt: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    draw_splitbar(hWnd, last_split);
    last_split = -1;
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rt));
    (*g_pChildWnd).nSplitPos = x;
    ResizeWnd(rt.right as ::core::ffi::c_int, rt.bottom as ::core::ffi::c_int);
    InvalidateRect(hWnd, core::ptr::addr_of_mut!(rt) as LPCRECT, (FALSE) as BOOL); // HACK: See CORE-19576
    ReleaseCapture();
}

unsafe extern "C" fn SuggestKeys(
    mut hRootKey: HKEY,
    mut pszKeyPath: LPCWSTR,
    mut pszSuggestions: LPWSTR,
    mut iSuggestionsLength: size_t,
) {
    let mut szBuffer: [WCHAR; 256] = [0; 256];
    let mut szLastFound: [WCHAR; 256] = [0; 256];
    let mut i: size_t = 0;
    let mut hOtherKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut hSubKey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut bFound: BOOL = 0;
    let regsam: REGSAM = (KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE) as REGSAM;

    memset(
        pszSuggestions as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        iSuggestionsLength.wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
    );
    iSuggestionsLength = iSuggestionsLength.wrapping_sub(1);

    /* Are we a root key in HKEY_CLASSES_ROOT? */
    if (hRootKey == HKEY_CLASSES_ROOT as HKEY)
        && *pszKeyPath.offset(0 as isize) != 0
        && wcschr(pszKeyPath, 0x005Cu16).is_null()
    {
        loop {
            bFound = FALSE as BOOL;

            /* Check default key */
            if QueryStringValue(
                hRootKey,
                pszKeyPath,
                ::core::ptr::null::<WCHAR>(),
                core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
                ARRAY_SIZE!(szBuffer) as DWORD,
            ) == ERROR_SUCCESS as LONG
            {
                /* Sanity check this key; it cannot be empty, nor can it be a
                 * loop back */
                if (szBuffer[0 as usize] != 0x0000u16)
                    && _wcsicmp(
                        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                        pszKeyPath as *const WCHAR,
                    ) != 0
                {
                    if RegOpenKeyExW(
                        hRootKey,
                        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
                        0 as DWORD,
                        regsam,
                        core::ptr::addr_of_mut!(hOtherKey),
                    ) == ERROR_SUCCESS as LSTATUS
                    {
                        lstrcpynW(
                            pszSuggestions,
                            RUSTCONV_WSTR_77F55C91EF10.as_ptr(),
                            iSuggestionsLength as ::core::ffi::c_int,
                        );
                        i = wcslen(pszSuggestions as *const WCHAR) as size_t;
                        pszSuggestions = pszSuggestions.offset(i as isize);
                        iSuggestionsLength = iSuggestionsLength.wrapping_sub(i);

                        lstrcpynW(
                            pszSuggestions,
                            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
                            iSuggestionsLength as ::core::ffi::c_int,
                        );
                        i = MIN!(
                            wcslen(pszSuggestions as *const WCHAR) as size_t
                                + 1 as size_t,
                            iSuggestionsLength,
                        );
                        pszSuggestions = pszSuggestions.offset(i as isize);
                        iSuggestionsLength = iSuggestionsLength.wrapping_sub(i);
                        RegCloseKey(hOtherKey);

                        bFound = TRUE as BOOL;
                        StringCbCopyW(
                            core::ptr::addr_of_mut!(szLastFound) as *mut WCHAR,
                            core::mem::size_of::<[WCHAR; 256]>() as size_t,
                            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                        );
                        pszKeyPath = core::ptr::addr_of_mut!(szLastFound) as *mut WCHAR as LPCWSTR;
                    }
                }
            }
            if !(bFound != 0 && iSuggestionsLength > 0 as size_t) {
                break;
            }
        }

        /* Check CLSID key */
        if RegOpenKeyExW(
            hRootKey,
            pszKeyPath,
            0 as DWORD,
            regsam,
            core::ptr::addr_of_mut!(hSubKey),
        ) == ERROR_SUCCESS as LSTATUS
        {
            if QueryStringValue(
                hSubKey,
                RUSTCONV_WSTR_AFF3436D7481.as_ptr(),
                ::core::ptr::null::<WCHAR>(),
                core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
                ARRAY_SIZE!(szBuffer) as DWORD,
            ) == ERROR_SUCCESS as LONG
            {
                lstrcpynW(
                    pszSuggestions,
                    RUSTCONV_WSTR_5677AD35830D.as_ptr(),
                    iSuggestionsLength as ::core::ffi::c_int,
                );
                i = wcslen(pszSuggestions as *const WCHAR) as size_t;
                pszSuggestions = pszSuggestions.offset(i as isize);
                iSuggestionsLength = iSuggestionsLength.wrapping_sub(i);

                lstrcpynW(
                    pszSuggestions,
                    core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
                    iSuggestionsLength as ::core::ffi::c_int,
                );
                i = MIN!(
                    wcslen(pszSuggestions as *const WCHAR) as size_t
                        + 1 as size_t,
                    iSuggestionsLength,
                );
                pszSuggestions = pszSuggestions.offset(i as isize);
                iSuggestionsLength = iSuggestionsLength.wrapping_sub(i);
            }
            RegCloseKey(hSubKey);
        }
    } else if ((hRootKey == HKEY_CURRENT_USER as HKEY) || (hRootKey == HKEY_LOCAL_MACHINE as HKEY))
        && *pszKeyPath != 0
    {
        let rootstr: LPCWSTR = if hRootKey == HKEY_CURRENT_USER as HKEY {
            RUSTCONV_WSTR_89F38F0B7FC9.as_ptr()
        } else {
            RUSTCONV_WSTR_D02D0BCFE5D4.as_ptr()
        };
        hOtherKey = if hRootKey == HKEY_CURRENT_USER as HKEY {
            HKEY_LOCAL_MACHINE as HKEY
        } else {
            HKEY_CURRENT_USER as HKEY
        };
        if RegOpenKeyExW(
            hOtherKey,
            pszKeyPath,
            0 as DWORD,
            regsam,
            core::ptr::addr_of_mut!(hSubKey),
        ) == ERROR_SUCCESS as LSTATUS
        {
            let mut cch: ::core::ffi::c_int = 0;
            RegCloseKey(hSubKey);
            cch = _snwprintf(
                pszSuggestions as *mut WCHAR,
                iSuggestionsLength,
                RUSTCONV_WSTR_BA2342D760A2.as_ptr(),
                rootstr,
                pszKeyPath,
            );
            if cch <= 0 as ::core::ffi::c_int
                || cch as size_t > iSuggestionsLength
            {
                *pszSuggestions.offset(0 as isize) =
                    UNICODE_NULL as WCHAR;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn AddressBarProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut oldwndproc: WNDPROC;
    static mut s_szNode: [WCHAR; 256] = [0; 256];
    oldwndproc = core::mem::transmute::<isize, WNDPROC>(GetWindowLongPtr(hwnd, GWLP_USERDATA) as isize);

    match uMsg {
        WM_KEYUP => {
            if wParam == VK_RETURN as WPARAM {
                GetWindowTextW(hwnd, core::ptr::addr_of_mut!(s_szNode) as LPWSTR, ARRAY_SIZE!(s_szNode) as core::ffi::c_int);
                SelectNode((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(s_szNode) as *mut WCHAR as LPCWSTR);
            }
        }
        _ => {}
    }
    return CallWindowProcW(oldwndproc, hwnd, uMsg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn UpdateAddress(mut hItem: HTREEITEM, mut hRootKey: HKEY, mut pszPath: LPCWSTR, mut bSelectNone: BOOL) {
    let mut keyPath: LPCWSTR;
    let mut rootName: LPCWSTR;
    let mut fullPath: LPWSTR;
    let mut cbFullPath: DWORD;

    /* Wipe the listview, the status bar and the address bar if the root key was selected */
    if TreeView_GetParent!((*g_pChildWnd).hTreeWnd, hItem).is_null()
    {
        ListView_DeleteAllItems!((*g_pChildWnd).hListWnd);
        SendMessageW(hStatusBar, SB_SETTEXTW, 0, core::ptr::null_mut::<core::ffi::c_void>() as LPARAM);
        SendMessageW((*g_pChildWnd).hAddressBarWnd, WM_SETTEXT, 0, core::ptr::null_mut::<core::ffi::c_void>() as LPARAM);
        return;
    }

    if pszPath.is_null()
        { keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, hItem, core::ptr::addr_of_mut!(hRootKey)); }
    else
        { keyPath = pszPath; }

    if !keyPath.is_null()
    {
        RefreshListView((*g_pChildWnd).hListWnd, hRootKey, keyPath, bSelectNone);
        rootName = get_root_key_name(hRootKey);
        cbFullPath = ((wcslen(rootName) as usize)
            .wrapping_add(1 as usize)
            .wrapping_add(wcslen(keyPath) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD;
        fullPath = malloc(cbFullPath as usize) as LPWSTR;
        if !fullPath.is_null()
        {
            /* set (correct) the address bar text */
            if *keyPath.offset(0 as isize) != UNICODE_NULL
                { StringCbPrintfW!(fullPath, cbFullPath as usize, RUSTCONV_WSTR_69D435449EDE.as_ptr(), rootName,
                                (if *keyPath.offset(0 as isize) == 0x005Cu16 { RUSTCONV_WSTR_96A296D224F2.as_ptr() } else { RUSTCONV_WSTR_B7FC02907B1F.as_ptr() }), keyPath); }
            else
                { StringCbCopyW(fullPath, cbFullPath as usize, rootName); }

            SendMessageW(hStatusBar, SB_SETTEXTW, 0, fullPath as LPARAM);
            SendMessageW((*g_pChildWnd).hAddressBarWnd, WM_SETTEXT, 0, fullPath as LPARAM);
            free(fullPath as *mut core::ffi::c_void);

            /* disable hive manipulation items temporarily (enable only if necessary) */
            EnableMenuItem(hMenuFrame, ID_REGISTRY_LOADHIVE, MF_BYCOMMAND | MF_GRAYED);
            EnableMenuItem(hMenuFrame, ID_REGISTRY_UNLOADHIVE, MF_BYCOMMAND | MF_GRAYED);
            /* compare the strings to see if we should enable/disable the "Load Hive" menus accordingly */
            if _wcsicmp(rootName, RUSTCONV_WSTR_50C930882486.as_ptr()) == 0 ||
                _wcsicmp(rootName, RUSTCONV_WSTR_9C98935C198E.as_ptr()) == 0
            {
                /*
                 * enable the unload menu item if at the root, otherwise
                 * enable the load menu item if there is no slash in
                 * keyPath (ie. immediate child selected)
                 */
                if *keyPath.offset(0 as isize) == UNICODE_NULL
                    { EnableMenuItem(hMenuFrame, ID_REGISTRY_LOADHIVE, MF_BYCOMMAND | MF_ENABLED); }
                else if wcschr(keyPath, 0x005Cu16).is_null()
                    { EnableMenuItem(hMenuFrame, ID_REGISTRY_UNLOADHIVE, MF_BYCOMMAND | MF_ENABLED); }
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn ChildWndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut Result: BOOL = 0;
    let mut rc: RECT = core::mem::zeroed();
    'sw: {
        'def: {
            match message {
                WM_CREATE => {
                    let mut oldproc: WNDPROC = None;
                    let mut hFont_local: HFONT = core::ptr::null_mut();
                    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                    let mut style: DWORD = 0;
                    let mut pAutoComplete: *mut IAutoComplete = core::ptr::null_mut();

                    /* Load "My Computer" string */
                    LoadStringW(hInst, IDS_MY_COMPUTER, core::ptr::addr_of_mut!(buffer) as LPWSTR, (ARRAY_SIZE!(buffer)) as i32);

                    g_pChildWnd = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<ChildWnd>()) as SIZE_T) as *mut ChildWnd;
                    if g_pChildWnd.is_null() {
                        return 0;
                    }

                    wcsncpy(core::ptr::addr_of_mut!((*g_pChildWnd).szPath) as *mut WCHAR, core::ptr::addr_of_mut!(buffer) as *mut WCHAR, (MAX_PATH) as _);
                    (*g_pChildWnd).nSplitPos = 190;
                    (*g_pChildWnd).hWnd = hWnd;

                    /* ES_AUTOHSCROLL style enables horizontal scrolling and shrinking */
                    style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL;
                    (*g_pChildWnd).hAddressBarWnd = CreateWindowExW(WS_EX_CLIENTEDGE, RUSTCONV_WSTR_2742CD8F7DE1.as_ptr(), core::ptr::null(), style,
                                                                  CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
                                                                  hWnd, 0 as HMENU, hInst, core::ptr::null_mut());

                    style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_ICON | BS_CENTER |
                            BS_VCENTER | BS_FLAT | BS_DEFPUSHBUTTON;
                    (*g_pChildWnd).hAddressBtnWnd = CreateWindowExW(0, RUSTCONV_WSTR_8D7C97326263.as_ptr(), RUSTCONV_WSTR_CC2767AFEE4B.as_ptr(), style,
                                                                  CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
                                                                  hWnd, 0 as HMENU, hInst, core::ptr::null_mut());
                    (*g_pChildWnd).hArrowIcon = LoadImageW(hInst, MAKEINTRESOURCEW!(IDI_ARROW),
                                                                IMAGE_ICON, 12, 12, 0) as HICON;
                    SendMessageW((*g_pChildWnd).hAddressBtnWnd, BM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*g_pChildWnd).hArrowIcon as LPARAM);

                    if CoCreateInstance(core::ptr::addr_of!(CLSID_AutoComplete), core::ptr::null_mut(), CLSCTX_INPROC_SERVER, core::ptr::addr_of!(IID_IAutoComplete), core::ptr::addr_of_mut!(pAutoComplete) as *mut LPVOID) >= 0
                    {
                        IAutoComplete_Init(pAutoComplete, (*g_pChildWnd).hAddressBarWnd, core::ptr::addr_of_mut!(g_DummyEnumStrings) as *mut IUnknown, core::ptr::null_mut(), core::ptr::null_mut());
                        IAutoComplete_Release(pAutoComplete);
                    }

                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
                    (*g_pChildWnd).hTreeWnd = CreateTreeView(hWnd, core::ptr::addr_of_mut!((*g_pChildWnd).szPath) as LPWSTR, TREE_WINDOW as HMENU);
                    (*g_pChildWnd).hListWnd = CreateListView(hWnd, LIST_WINDOW as HMENU, rc.right - (*g_pChildWnd).nSplitPos);
                    SetFocus((*g_pChildWnd).hTreeWnd);

                    /* set the address bar and button font */
                    if ((*g_pChildWnd).hAddressBarWnd.is_null() == false) && ((*g_pChildWnd).hAddressBtnWnd.is_null() == false)
                    {
                        hFont_local = GetStockObject((DEFAULT_GUI_FONT as i32)) as HFONT;
                        SendMessageW((*g_pChildWnd).hAddressBarWnd,
                                     WM_SETFONT,
                                     hFont_local as WPARAM,
                                     0);
                        SendMessageW((*g_pChildWnd).hAddressBtnWnd,
                                     WM_SETFONT,
                                     hFont_local as WPARAM,
                                     0);
                    }
                    /* Subclass the AddressBar */
                    oldproc = core::mem::transmute::<isize, WNDPROC>((GetWindowLongPtr((*g_pChildWnd).hAddressBarWnd, GWLP_WNDPROC)) as _);
                    SetWindowLongPtr((*g_pChildWnd).hAddressBarWnd, GWLP_USERDATA, core::mem::transmute::<WNDPROC, DWORD_PTR>(oldproc) as LONG_PTR);
                    SetWindowLongPtr((*g_pChildWnd).hAddressBarWnd, GWLP_WNDPROC, AddressBarProc as DWORD_PTR as LONG_PTR);
                    break 'sw;
                }
                WM_COMMAND => {
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED
                    {
                        PostMessageW((*g_pChildWnd).hAddressBarWnd, WM_KEYUP, (VK_RETURN) as WPARAM, 0);
                    }
                    break 'sw; //goto def;
                }
                WM_SETCURSOR => {
                    if (LOWORD!(lParam)) as u32 == HTCLIENT
                    {
                        let mut pt: POINT = core::mem::zeroed();
                        GetCursorPos(core::ptr::addr_of_mut!(pt));
                        ScreenToClient(hWnd, core::ptr::addr_of_mut!(pt));
                        if pt.x >= (*g_pChildWnd).nSplitPos - (SPLIT_WIDTH / 2) as i32 && pt.x < (*g_pChildWnd).nSplitPos + (SPLIT_WIDTH / 2) as i32 + 1
                        {
                            SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_SIZEWE));
                            return (TRUE) as LRESULT;
                        }
                    }
                    break 'def;
                }

                WM_DESTROY => {
                    DestroyListView((*g_pChildWnd).hListWnd);
                    DestroyTreeView((*g_pChildWnd).hTreeWnd);
                    DestroyMainMenu();
                    DestroyIcon((*g_pChildWnd).hArrowIcon);
                    HeapFree(GetProcessHeap(), 0, g_pChildWnd as PVOID);
                    g_pChildWnd = core::ptr::null_mut();
                    PostQuitMessage(0);
                    break 'sw;
                }

                WM_LBUTTONDOWN => {
                    let mut x: INT = LOWORD!(lParam) as SHORT as INT;
                    if x >= (*g_pChildWnd).nSplitPos - (SPLIT_WIDTH / 2) as i32 &&
                        x < (*g_pChildWnd).nSplitPos + (SPLIT_WIDTH / 2) as i32 + 1
                    {
                        x = ClampSplitBarX(hWnd, x);
                        draw_splitbar(hWnd, x);
                        last_split = x;
                        SetCapture(hWnd);
                    }
                    break 'sw;
                }

                WM_LBUTTONUP | WM_RBUTTONDOWN => {
                    if GetCapture() == hWnd
                    {
                        let mut x: INT = LOWORD!(lParam) as SHORT as INT;
                        x = ClampSplitBarX(hWnd, x);
                        finish_splitbar(hWnd, x);
                    }
                    break 'sw;
                }

                WM_CAPTURECHANGED => {
                    if GetCapture() == hWnd && last_split >= 0
                    {
                        draw_splitbar(hWnd, last_split);
                    }
                    break 'sw;
                }

                WM_KEYDOWN => {
                    if wParam == (VK_ESCAPE) as _
                    {
                        if GetCapture() == hWnd
                        {
                            let mut rt: RECT = core::mem::zeroed();
                            draw_splitbar(hWnd, last_split);
                            GetClientRect(hWnd, core::ptr::addr_of_mut!(rt));
                            ResizeWnd(rt.right, rt.bottom);
                            last_split = -1;
                            ReleaseCapture();
                            SetCursor(LoadCursorW(0 as HINSTANCE, IDC_ARROW as LPCWSTR));
                        }
                    }
                    break 'sw;
                }

                WM_MOUSEMOVE => {
                    if GetCapture() == hWnd
                    {
                        let mut x: INT = LOWORD!(lParam) as SHORT as INT;
                        x = ClampSplitBarX(hWnd, x);
                        if last_split != x
                        {
                            draw_splitbar(hWnd, last_split);
                            last_split = x;
                            draw_splitbar(hWnd, last_split);
                        }
                    }
                    break 'sw;
                }

                WM_SETFOCUS => {
                    if !g_pChildWnd.is_null()
                    {
                        SetFocus(if (*g_pChildWnd).nFocusPanel != 0 { (*g_pChildWnd).hListWnd } else { (*g_pChildWnd).hTreeWnd });
                    }
                    break 'sw;
                }

                WM_NOTIFY => {
                    if g_pChildWnd.is_null() { break 'sw; }

                    if (*(lParam as LPNMHDR)).idFrom == TREE_WINDOW as UINT_PTR
                    {
                        if TreeWndNotifyProc((*g_pChildWnd).hListWnd, wParam, lParam, core::ptr::addr_of_mut!(Result)) == 0
                        {
                            break 'def;
                        }

                        return Result as LRESULT;
                    }
                    else
                    {
                        if (*(lParam as LPNMHDR)).idFrom == LIST_WINDOW as UINT_PTR
                        {
                            if ListWndNotifyProc((*g_pChildWnd).hListWnd, wParam, lParam, core::ptr::addr_of_mut!(Result)) == 0
                            {
                                break 'def;
                            }

                            return Result as LRESULT;
                        }
                        else
                        {
                            break 'def;
                        }
                    }
                    #[allow(unreachable_code)]
                    break 'sw;
                }

                WM_CONTEXTMENU => {
                    let mut pt: POINT = core::mem::zeroed();
                    if wParam as HWND == (*g_pChildWnd).hListWnd
                    {
                        let mut i: c_int;
                        let mut cnt: c_int;
                        let mut IsDefault: BOOL;
                        pt.x = LOWORD!(lParam) as SHORT as LONG;
                        pt.y = HIWORD!(lParam) as SHORT as LONG;
                        cnt = ListView_GetSelectedCount!((*g_pChildWnd).hListWnd) as c_int;
                        i = ListView_GetNextItem!((*g_pChildWnd).hListWnd, -1, LVNI_FOCUSED | LVNI_SELECTED);
                        if pt.x == -1 && pt.y == -1
                        {
                            let mut rc_0: RECT = core::mem::zeroed();
                            if i != -1
                            {
                                rc_0.left = (LVIR_BOUNDS as i32);
                                SendMessageW((*g_pChildWnd).hListWnd, LVM_GETITEMRECT, i as WPARAM, core::ptr::addr_of_mut!(rc_0) as LPARAM);
                                pt.x = rc_0.left + 8;
                                pt.y = rc_0.top + 8;
                            }
                            else
                            {
                                pt.y = 0;
                                pt.x = pt.y;
                            }
                            ClientToScreen((*g_pChildWnd).hListWnd, core::ptr::addr_of_mut!(pt));
                        }
                        if i == -1
                        {
                            TrackPopupMenu(GetSubMenu(hPopupMenus, (PM_NEW as i32)), TPM_RIGHTBUTTON, pt.x as c_int, pt.y as c_int, 0, hFrameWnd, core::ptr::null());
                        }
                        else
                        {
                            let mut mnu: HMENU = GetSubMenu(hPopupMenus, (PM_MODIFYVALUE as i32));
                            SetMenuDefaultItem(mnu, ID_EDIT_MODIFY, MF_BYCOMMAND);
                            IsDefault = IsDefaultValue((*g_pChildWnd).hListWnd, i);
                            if cnt == 1
                            {
                                EnableMenuItem(mnu, ID_EDIT_RENAME, MF_BYCOMMAND | (if IsDefault != 0 { MF_DISABLED | MF_GRAYED } else { MF_ENABLED }));
                            }
                            else
                            {
                                EnableMenuItem(mnu, ID_EDIT_RENAME, MF_BYCOMMAND | MF_DISABLED | MF_GRAYED);
                            }
                            EnableMenuItem(mnu, ID_EDIT_MODIFY, MF_BYCOMMAND | (if cnt == 1 { MF_ENABLED } else { MF_DISABLED | MF_GRAYED }));
                            EnableMenuItem(mnu, ID_EDIT_MODIFY_BIN, MF_BYCOMMAND | (if cnt == 1 { MF_ENABLED } else { MF_DISABLED | MF_GRAYED }));

                            TrackPopupMenu(mnu, TPM_RIGHTBUTTON, pt.x as c_int, pt.y as c_int, 0, hFrameWnd, core::ptr::null());
                        }
                    }
                    else if wParam as HWND == (*g_pChildWnd).hTreeWnd
                    {
                        let mut hti: TVHITTESTINFO = core::mem::zeroed();
                        let mut hContextMenu: HMENU;
                        let mut item: TVITEMW = core::mem::zeroed();
                        let mut mii: MENUITEMINFOW = core::mem::zeroed();
                        let mut resource: [WCHAR; 256] = [0; 256];
                        let mut buffer: [WCHAR; 256] = [0; 256];
                        let mut s: LPWSTR;
                        let mut keyPath: LPCWSTR;
                        let mut hRootKey: HKEY = core::mem::zeroed();
                        let mut iLastPos: c_int;
                        let mut wID: WORD;
                        let mut isRoot: BOOL;

                        pt.x = LOWORD!(lParam) as SHORT as LONG;
                        pt.y = HIWORD!(lParam) as SHORT as LONG;

                        if pt.x == -1 && pt.y == -1
                        {
                            let mut rc_1: RECT = core::mem::zeroed();
                            hti.hItem = TreeView_GetSelection!((*g_pChildWnd).hTreeWnd);
                            if !hti.hItem.is_null()
                            {
                                TreeView_GetItemRect!((*g_pChildWnd).hTreeWnd, hti.hItem, core::ptr::addr_of_mut!(rc_1), TRUE);
                                pt.x = rc_1.left + 8;
                                pt.y = rc_1.top + 8;
                                ClientToScreen((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(pt));
                                hti.flags = (TVHT_ONITEM as u32);
                            }
                            else
                            {
                                hti.flags = 0;
                            }
                        }
                        else
                        {
                            hti.pt.x = pt.x;
                            hti.pt.y = pt.y;
                            ScreenToClient((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(hti.pt));
                            TreeView_HitTest!((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(hti));
                        }

                        if (hti.flags & (TVHT_ONITEM as u32)) != 0
                        {
                            TreeView_SelectItem!((*g_pChildWnd).hTreeWnd, hti.hItem);

                            isRoot = (TreeView_GetParent!((*g_pChildWnd).hTreeWnd, hti.hItem) == core::ptr::null_mut()) as BOOL;
                            hContextMenu = GetSubMenu(hPopupMenus, if isRoot != 0 { (PM_ROOTITEM as i32) } else { (PM_TREECONTEXT as i32) });

                            memset(core::ptr::addr_of_mut!(item) as *mut c_void, 0, core::mem::size_of::<TVITEMW>());
                            item.mask = TVIF_STATE | TVIF_CHILDREN;
                            item.hItem = hti.hItem;
                            TreeView_GetItem!((*g_pChildWnd).hTreeWnd, core::ptr::addr_of_mut!(item));

                            /* Set the Expand/Collapse menu item appropriately */
                            LoadStringW(hInst, if (item.state & TVIS_EXPANDED) != 0 { IDS_COLLAPSE } else { IDS_EXPAND }, core::ptr::addr_of_mut!(buffer) as LPWSTR, (ARRAY_SIZE!(buffer)) as i32);
                            memset(core::ptr::addr_of_mut!(mii) as *mut c_void, 0, core::mem::size_of::<MENUITEMINFOW>());
                            mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
                            mii.fMask = MIIM_STRING | MIIM_STATE | MIIM_ID;
                            mii.fState = if item.cChildren > 0 { MFS_DEFAULT } else { MFS_GRAYED };
                            mii.wID = if (item.state & TVIS_EXPANDED) != 0 { ID_TREE_COLLAPSEBRANCH } else { ID_TREE_EXPANDBRANCH };
                            mii.dwTypeData = core::ptr::addr_of_mut!(buffer) as LPWSTR;
                            SetMenuItemInfo(hContextMenu, 0, (TRUE) as i32, core::ptr::addr_of_mut!(mii));

                            if (isRoot) as u32 == FALSE
                            {
                                /* Remove any existing suggestions */
                                memset(core::ptr::addr_of_mut!(mii) as *mut c_void, 0, core::mem::size_of::<MENUITEMINFOW>());
                                mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
                                mii.fMask = MIIM_ID;
                                GetMenuItemInfo(hContextMenu, (GetMenuItemCount(hContextMenu) - 1) as u32, (TRUE) as i32, core::ptr::addr_of_mut!(mii));
                                if mii.wID >= ID_TREE_SUGGESTION_MIN && mii.wID <= ID_TREE_SUGGESTION_MAX
                                {
                                    loop {
                                        iLastPos = GetMenuItemCount(hContextMenu) - 1;
                                        GetMenuItemInfo(hContextMenu, (iLastPos) as u32, (TRUE) as i32, core::ptr::addr_of_mut!(mii));
                                        RemoveMenu(hContextMenu, (iLastPos) as UINT, MF_BYPOSITION);
                                        if !(mii.wID >= ID_TREE_SUGGESTION_MIN && mii.wID <= ID_TREE_SUGGESTION_MAX) {
                                            break;
                                        }
                                    }
                                }

                                /* Come up with suggestions */
                                keyPath = GetItemPath((*g_pChildWnd).hTreeWnd, core::ptr::null_mut(), core::ptr::addr_of_mut!(hRootKey));
                                SuggestKeys(hRootKey, keyPath, core::ptr::addr_of_mut!(Suggestions) as LPWSTR, ARRAY_SIZE!(Suggestions));
                                if Suggestions[0 as usize] != 0
                                {
                                    AppendMenu(hContextMenu, MF_SEPARATOR, 0, core::ptr::null());

                                    LoadStringW(hInst, IDS_GOTO_SUGGESTED_KEY, core::ptr::addr_of_mut!(resource) as LPWSTR, (ARRAY_SIZE!(resource)) as i32);

                                    s = core::ptr::addr_of_mut!(Suggestions) as *mut WCHAR as LPWSTR;
                                    wID = (ID_TREE_SUGGESTION_MIN as u16);
                                    while *s != 0 && (wID as c_int) <= (ID_TREE_SUGGESTION_MAX as i32)
                                    {
                                        let mut path: *mut WCHAR = s as *mut WCHAR;
                                        let mut buf: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                                        if hRootKey == HKEY_CURRENT_USER as HKEY || hRootKey == HKEY_LOCAL_MACHINE as HKEY
                                        {
                                            // Windows 10 only displays the root name
                                            let mut next: LPCWSTR = PathFindNextComponentW(s as LPCWSTR);
                                            if next > s as LPCWSTR
                                            {
                                                lstrcpynW({ path = core::ptr::addr_of_mut!(buf) as *mut WCHAR; path }, s as LPCWSTR, min!(next.offset_from(s as LPCWSTR) as usize, (_countof!(buf)) as usize) as c_int);
                                            }
                                        }
                                        _snwprintf(core::ptr::addr_of_mut!(buffer) as *mut WCHAR, ARRAY_SIZE!(buffer), core::ptr::addr_of_mut!(resource) as *mut WCHAR, path);

                                        memset(core::ptr::addr_of_mut!(mii) as *mut c_void, 0, core::mem::size_of::<MENUITEMINFOW>());
                                        mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
                                        mii.fMask = MIIM_STRING | MIIM_ID;
                                        let fresh = wID;
                                        wID = wID.wrapping_add(1);
                                        mii.wID = fresh as UINT;
                                        mii.dwTypeData = core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR;
                                        InsertMenuItem(hContextMenu, GetMenuItemCount(hContextMenu) as UINT, (TRUE) as i32, core::ptr::addr_of_mut!(mii));

                                        s = s.offset((wcslen(s as *const wchar_t) + 1 as size_t) as isize);
                                    }
                                }
                            }
                            TrackPopupMenu(hContextMenu, TPM_RIGHTBUTTON, pt.x as c_int, pt.y as c_int, 0, hFrameWnd, core::ptr::null());
                        }
                    }
                    break 'sw;
                }

                WM_SIZE => {
                    if wParam != (SIZE_MINIMIZED) as _ && !g_pChildWnd.is_null()
                    {
                        ResizeWnd((LOWORD!(lParam)) as _, (HIWORD!(lParam)) as _);
                    }
                    break 'sw;
                }

                _ => break 'def,
            }
        }
        // def:
        return DefWindowProcW(hWnd, message, wParam, lParam);
    }
    return 0;
}
