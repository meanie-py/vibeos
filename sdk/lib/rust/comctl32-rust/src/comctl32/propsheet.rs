use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::commctrl::CreateMappedBitmap;
use crate::comctl32::commctrl::DefSubclassProc;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::imagelist::ImageList_AddIcon;
use crate::comctl32::imagelist::ImageList_Create;
use crate::comctl32::imagelist::ImageList_Destroy;
use crate::comctl32::commctrl::RemoveWindowSubclass;
use crate::comctl32::commctrl::SetWindowSubclass;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_propsheet);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_STATICW: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"S\0t\0a\0t\0i\0c\0\0\0")
};

static mut PropSheetInfoStr: [WCHAR; 18] = [
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// NONAMELESSUNION: object-like macro with no value, no declaration emitted.

// propsheet.c:71-93: between `pshpack2.h` and `poppack.h` -- 2-byte packing, as the C lays it out.
#[repr(C, packed(2))]
pub struct MyDLGTEMPLATEEX {
    pub dlgVer: WORD,
    pub signature: WORD,
    pub helpID: DWORD,
    pub exStyle: DWORD,
    pub style: DWORD,
}

// propsheet.c:71-93: between `pshpack2.h` and `poppack.h` -- 2-byte packing, as the C lays it out.
#[repr(C, packed(2))]
pub struct MyDLGITEMTEMPLATEEX {
    pub helpid: DWORD,
    pub exStyle: DWORD,
    pub style: DWORD,
    pub x: i16,
    pub y: i16,
    pub cx: i16,
    pub cy: i16,
    pub id: DWORD,
}

#[repr(C)]
pub struct tagPropPageInfo {
    pub hpage: HPROPSHEETPAGE, // to keep track of pages not passed to PropertySheet
    pub hwndPage: HWND,
    pub isDirty: BOOL,
    pub pszText: LPCWSTR,
    pub hasHelp: BOOL,
    pub useCallback: BOOL,
    pub hasIcon: BOOL,
}
pub type PropPageInfo = tagPropPageInfo;

#[repr(C)]
pub struct tagPropSheetInfo {
    pub hwnd: HWND,
    pub ppshheader: PROPSHEETHEADERW,
    pub unicode: BOOL,
    pub strPropertiesFor: LPWSTR,
    pub nPages: i32,
    pub active_page: i32,
    pub isModeless: BOOL,
    pub hasHelp: BOOL,
    pub hasApply: BOOL,
    pub hasFinish: BOOL,
    pub usePropPage: BOOL,
    pub useCallback: BOOL,
    pub activeValid: BOOL,
    pub proppage: *mut PropPageInfo,
    pub hFont: HFONT,
    pub hFontBold: HFONT,
    pub width: i32,
    pub height: i32,
    pub hImageList: HIMAGELIST,
    pub ended: BOOL,
    pub result: INT,
}
pub type PropSheetInfo = tagPropSheetInfo;

#[repr(C)]
pub struct PADDING_INFO {
    pub x: i32,
    pub y: i32,
}

pub const PSP_INTERNAL_UNICODE: u32 = 0x80000000;

pub const MAX_CAPTION_LENGTH: i32 = 255;

pub const MAX_TABTEXT_LENGTH: i32 = 255;

pub const MAX_BUTTONTEXT_LENGTH: i32 = 64;

pub const INTRNL_ANY_WIZARD: u32 = PSH_WIZARD | PSH_WIZARD97_OLD | PSH_WIZARD97_NEW | PSH_WIZARD_LITE;

pub const WIZARD_PADDING: i32 = 7;

pub const WIZARD_HEADER_HEIGHT: i32 = 36;

// WINE_DEFAULT_DEBUG_CHANNEL(propsheet): debug-channel registration macro, no runtime data emitted here.
// add_flag(a): function-like macro, skipped. #undef add_flag: no declaration.

unsafe extern "C" fn heap_strdupW(str: *const WCHAR) -> *mut WCHAR {
    let mut len: ::core::ffi::c_int = lstrlenW(str as LPCWSTR) + 1;
    let mut ret: *mut WCHAR =
        Alloc((len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD)
            as *mut WCHAR;
    lstrcpyW(ret as LPWSTR, str as LPCWSTR);
    return ret;
}

unsafe extern "C" fn heap_strdupAtoW(str: *const ::core::ffi::c_char) -> *mut WCHAR {
    let mut ret: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut len: INT = 0;
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str as LPCSTR,
        -(1 as ::core::ffi::c_int),
        0 as LPWSTR,
        0 as ::core::ffi::c_int,
    ) as INT;
    ret = Alloc(
        (len as usize).wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
    ) as *mut WCHAR;
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        str as LPCSTR,
        -(1 as ::core::ffi::c_int),
        ret as LPWSTR,
        len as ::core::ffi::c_int,
    );
    return ret;
}

unsafe extern "C" fn PROPSHEET_SetInitialFocus(hwndDlg: HWND, index: i32, psInfo: *mut PropSheetInfo) {
    let mut psn: PSHNOTIFY = PSHNOTIFY {
        hdr: NMHDR {
            hwndFrom: core::ptr::null_mut(),
            idFrom: 0,
            code: 0,
        },
        lParam: 0,
    };
    let mut focusable_item: HWND = core::ptr::null_mut();
    let mut initial_focus: HWND = core::ptr::null_mut();

    focusable_item = GetNextDlgTabItem(
        (*(*psInfo).proppage.offset(index as isize)).hwndPage,
        core::ptr::null_mut(),
        FALSE as i32,
    );
    if focusable_item.is_null() {
        return;
    }

    psn.hdr.code = PSN_QUERYINITIALFOCUS as UINT;
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0 as UINT_PTR;
    psn.lParam = 0 as LPARAM;
    initial_focus = SendMessageW(
        (*(*psInfo).proppage.offset(index as isize)).hwndPage,
        WM_NOTIFY as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(psn) as LPARAM,
    ) as HWND;
    if !initial_focus.is_null() {
        SetFocus(initial_focus);
    } else if !focusable_item.is_null() {
        SetFocus(focusable_item);
    }
}

unsafe extern "C" fn PROPSHEET_UnImplementedFlags(dwFlags: DWORD) {
    let mut string: [CHAR; 256] = [0; 256];

    string[0 as usize] = '\0' as i32 as CHAR;

    add_flag!(dwFlags, string, PSH_RTLREADING);
    add_flag!(dwFlags, string, PSH_STRETCHWATERMARK);
    add_flag!(dwFlags, string, PSH_USEPAGELANG);
    if string[0 as usize] as ::core::ffi::c_int != '\0' as i32 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/propsheet.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PROPSHEET_UnImplementedFlags\0" as *const u8
                    as *const ::core::ffi::c_char,
                243 as ::core::ffi::c_int,
                b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut string as *mut CHAR,
            );
        }
    }
}

unsafe extern "C" fn PROPSHEET_GetPageRect(psInfo: *const PropSheetInfo, hwndDlg: HWND, rc: *mut RECT, ppshpage: LPCPROPSHEETPAGEW) {
    if (*psInfo).ppshheader.dwFlags & INTRNL_ANY_WIZARD as DWORD != 0 {
        let mut hwndChild: HWND = core::ptr::null_mut();
        let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if ((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_NEW | PSH_WIZARD97_OLD) as DWORD != 0)
            && ((*psInfo).ppshheader.dwFlags & PSH_HEADER as DWORD != 0)
            && ((*ppshpage).dwFlags & PSP_HIDEHEADER as DWORD == 0)
            || ((*psInfo).ppshheader.dwFlags & PSH_WIZARD as DWORD != 0)
        {
            (*rc).top = WIZARD_PADDING as LONG;
            (*rc).left = (*rc).top;
        } else {
            (*rc).top = 0 as LONG;
            (*rc).left = (*rc).top;
        }
        (*rc).right = (*psInfo).width as LONG - (*rc).left;
        (*rc).bottom = (*psInfo).height as LONG - (*rc).top;
        MapDialogRect(hwndDlg, rc as LPRECT);

        if ((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_NEW | PSH_WIZARD97_OLD) as DWORD != 0)
            && ((*psInfo).ppshheader.dwFlags & PSH_HEADER as DWORD != 0)
            && ((*ppshpage).dwFlags & PSP_HIDEHEADER as DWORD == 0)
        {
            hwndChild = GetDlgItem(hwndDlg, IDC_SUNKEN_LINEHEADER as core::ffi::c_int);
            GetClientRect(hwndChild, &raw mut r);
            MapWindowPoints(hwndChild, hwndDlg, &raw mut r as LPPOINT, 2 as UINT);
            (*rc).top += r.bottom as LONG + 1 as LONG;
        }
    } else {
        let mut hwndTabCtrl: HWND = GetDlgItem(hwndDlg, IDC_TABCONTROL as core::ffi::c_int);
        GetClientRect(hwndTabCtrl, rc as LPRECT);
        SendMessageW(
            hwndTabCtrl,
            TCM_ADJUSTRECT as UINT,
            FALSE as WPARAM,
            rc as LPARAM,
        );
        MapWindowPoints(hwndTabCtrl, hwndDlg, rc as LPPOINT, 2 as UINT);
    }
}

unsafe extern "C" fn PROPSHEET_FindPageByResId(
    mut psInfo: *const PropSheetInfo,
    mut resId: LRESULT,
) -> INT {
    let mut i: INT = 0 as INT;
    i = 0 as INT;
    while i < (*psInfo).nPages {
        let mut lppsp: LPCPROPSHEETPAGEA =
            (*(*psInfo).proppage.offset(i as isize)).hpage as LPCPROPSHEETPAGEA;
        if (*lppsp).u.pszTemplate == resId as LPVOID as LPCSTR {
            break;
        }
        i += 1;
    }
    return i;
}

unsafe extern "C" fn PROPSHEET_CollectSheetInfoCommon(psInfo: *mut PropSheetInfo, dwFlags: DWORD) {
    PROPSHEET_UnImplementedFlags(dwFlags);

    (*psInfo).hasHelp = (dwFlags & PSH_HASHELP) as BOOL;
    (*psInfo).hasApply = !(dwFlags & PSH_NOAPPLYNOW) as BOOL;
    (*psInfo).hasFinish = (dwFlags & PSH_WIZARDHASFINISH) as BOOL;
    (*psInfo).isModeless = (dwFlags & PSH_MODELESS) as BOOL;
    (*psInfo).usePropPage = (dwFlags & PSH_PROPSHEETPAGE) as BOOL;
    if (*psInfo).active_page < 0 || (*psInfo).active_page >= (*psInfo).nPages {
        (*psInfo).active_page = 0;
    }

    (*psInfo).result = 0;
    (*psInfo).hImageList = core::ptr::null_mut();
    (*psInfo).activeValid = FALSE;
}

unsafe extern "C" fn PROPSHEET_CollectSheetInfoA(
    mut lppsh: LPCPROPSHEETHEADERA,
    mut psInfo: *mut PropSheetInfo,
) {
    let mut dwSize: DWORD = min((*lppsh).dwSize, core::mem::size_of::<PROPSHEETHEADERA>() as DWORD);
    let mut dwFlags: DWORD = (*lppsh).dwFlags;
    (*psInfo).useCallback = (((dwFlags & PSH_USECALLBACK) != 0) && (*lppsh).pfnCallback.is_some()) as ::core::ffi::c_int as BOOL;
    memcpy(
        core::ptr::addr_of_mut!((*psInfo).ppshheader) as *mut ::core::ffi::c_void,
        lppsh as *const ::core::ffi::c_void,
        dwSize as usize,
    );
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_CollectSheetInfoA\0" as *const u8 as *const ::core::ffi::c_char,
            350 as ::core::ffi::c_int,
            b"\n** PROPSHEETHEADER **\ndwSize\t\t%d\ndwFlags\t\t%08x\nhwndParent\t%p\nhInstance\t%p\npszCaption\t'%s'\nnPages\t\t%d\npfnCallback\t%p\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            (*lppsh).dwSize,
            (*lppsh).dwFlags,
            (*lppsh).hwndParent,
            (*lppsh).hInstance,
            debugstr_a((*lppsh).pszCaption as *const ::core::ffi::c_char),
            (*lppsh).nPages,
            (*lppsh).pfnCallback,
        );
    }
    if ((*lppsh).dwFlags & INTRNL_ANY_WIZARD) != 0 {
        (*psInfo).ppshheader.pszCaption = core::ptr::null::<WCHAR>();
    } else {
        if !IS_INTRESOURCE!((*lppsh).pszCaption) {
            let mut len: ::core::ffi::c_int = MultiByteToWideChar(
                CP_ACP,
                0,
                (*lppsh).pszCaption,
                -(1),
                core::ptr::null_mut::<WCHAR>(),
                0,
            );
            let mut caption: *mut WCHAR = Alloc(
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as *mut WCHAR;
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lppsh).pszCaption,
                -(1),
                caption as LPWSTR,
                len,
            );
            (*psInfo).ppshheader.pszCaption = caption as LPCWSTR;
        }
    }
    (*psInfo).nPages = (*lppsh).nPages as ::core::ffi::c_int;
    if (dwFlags & PSH_USEPSTARTPAGE) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/propsheet.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PROPSHEET_CollectSheetInfoA\0" as *const u8 as *const ::core::ffi::c_char,
                369 as ::core::ffi::c_int,
                b"PSH_USEPSTARTPAGE is on\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        (*psInfo).active_page = 0;
    } else {
        (*psInfo).active_page = (*lppsh).u2.nStartPage as ::core::ffi::c_int;
    }
    PROPSHEET_CollectSheetInfoCommon(psInfo, dwFlags);
}

unsafe extern "C" fn PROPSHEET_CollectSheetInfoW(lppsh: LPCPROPSHEETHEADERW, psInfo: *mut PropSheetInfo) {
    let mut dwSize: DWORD = min((*lppsh).dwSize, core::mem::size_of::<PROPSHEETHEADERW>() as DWORD);
    let mut dwFlags: DWORD = (*lppsh).dwFlags;

    (*psInfo).useCallback = (((dwFlags & PSH_USECALLBACK) != 0) && (*lppsh).pfnCallback.is_some()) as ::core::ffi::c_int as BOOL;

    memcpy(
        core::ptr::addr_of_mut!((*psInfo).ppshheader) as *mut ::core::ffi::c_void,
        lppsh as *const ::core::ffi::c_void,
        dwSize as usize,
    );

    if (*lppsh).dwFlags & INTRNL_ANY_WIZARD != 0 {
        (*psInfo).ppshheader.pszCaption = core::ptr::null::<WCHAR>();
    } else {
        if !IS_INTRESOURCE!((*lppsh).pszCaption) {
            (*psInfo).ppshheader.pszCaption = heap_strdupW((*lppsh).pszCaption as *const WCHAR) as LPCWSTR;
        }
    }
    (*psInfo).nPages = (*lppsh).nPages as ::core::ffi::c_int;

    if dwFlags & PSH_USEPSTARTPAGE != 0 {
        (*psInfo).active_page = 0 as ::core::ffi::c_int;
    } else {
        (*psInfo).active_page = (*lppsh).u2.nStartPage as ::core::ffi::c_int;
    }

    PROPSHEET_CollectSheetInfoCommon(psInfo, dwFlags);
}

unsafe extern "C" fn PROPSHEET_CollectPageInfo(
    lppsp: LPCPROPSHEETPAGEW,
    psInfo: *mut PropSheetInfo,
    index: ::core::ffi::c_int,
    resize: BOOL,
) -> BOOL {
    let mut pTemplate: *const DLGTEMPLATE = ::core::ptr::null();
    let mut p: *const WORD = ::core::ptr::null();
    let mut dwFlags: DWORD = 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut height: ::core::ffi::c_int = 0;

    if lppsp.is_null() {
        return FALSE;
    }

    (*(*psInfo).proppage.offset(index as isize)).hpage = lppsp as HPROPSHEETPAGE;
    (*(*psInfo).proppage.offset(index as isize)).hwndPage = core::ptr::null_mut();
    (*(*psInfo).proppage.offset(index as isize)).isDirty = FALSE;

    dwFlags = (*lppsp).dwFlags;
    (*(*psInfo).proppage.offset(index as isize)).useCallback =
        ((dwFlags & (PSP_USECALLBACK as DWORD)) != 0 && (*lppsp).pfnCallback.is_some())
            as ::core::ffi::c_int as BOOL;
    (*(*psInfo).proppage.offset(index as isize)).hasHelp =
        (dwFlags & (PSP_HASHELP as DWORD)) as BOOL;
    (*(*psInfo).proppage.offset(index as isize)).hasIcon =
        (dwFlags & ((PSP_USEHICON | PSP_USEICONID) as DWORD)) as BOOL;

    if (*(*psInfo).proppage.offset(index as isize)).hasHelp != 0 {
        (*psInfo).hasHelp = TRUE;
    }

    if (dwFlags & (PSP_DLGINDIRECT as DWORD)) != 0 {
        pTemplate = (*lppsp).u.pResource as *const DLGTEMPLATE;
    } else if (dwFlags & (PSP_INTERNAL_UNICODE as DWORD)) != 0 {
        let hResource: HRSRC = FindResourceW(
            (*lppsp).hInstance,
            (*lppsp).u.pszTemplate,
            RT_DIALOG as LPWSTR,
        );
        let hTemplate: HGLOBAL = LoadResource((*lppsp).hInstance, hResource);
        pTemplate = LockResource(hTemplate) as *const DLGTEMPLATE;
    } else {
        let hResource: HRSRC = FindResourceA(
            (*lppsp).hInstance,
            (*lppsp).u.pszTemplate as LPCSTR,
            RT_DIALOG as LPSTR,
        );
        let hTemplate: HGLOBAL = LoadResource((*lppsp).hInstance, hResource);
        pTemplate = LockResource(hTemplate) as *const DLGTEMPLATE;
    }

    if pTemplate.is_null() {
        return FALSE;
    }

    p = pTemplate as *const WORD;

    if (*(pTemplate as *const MyDLGTEMPLATEEX)).signature as ::core::ffi::c_int
        == 0xFFFF as ::core::ffi::c_int
    {
        p = p.offset(1);
        p = p.offset(1);
        p = p.offset(2 as isize);
        p = p.offset(2 as isize);
        p = p.offset(2 as isize);
    } else {
        p = p.offset(2 as isize);
        p = p.offset(2 as isize);
    }

    p = p.offset(1);
    p = p.offset(1);
    p = p.offset(1);
    width = *p as ::core::ffi::c_int;
    p = p.offset(1);
    height = *p as ::core::ffi::c_int;
    p = p.offset(1);

    if ((*lppsp).dwFlags & ((PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD)) != 0 {
        (*psInfo).ppshheader.dwFlags |= PSH_HEADER as DWORD;
    }

    if ((*psInfo).ppshheader.dwFlags & ((PSH_WIZARD97_OLD | PSH_WIZARD97_NEW) as DWORD)) != 0
        && ((*psInfo).ppshheader.dwFlags & (PSH_HEADER as DWORD)) != 0
        && !((dwFlags & (PSP_HIDEHEADER as DWORD)) != 0)
    {
        height += (2 as ::core::ffi::c_int)
            * (WIZARD_PADDING as ::core::ffi::c_int)
            + (WIZARD_HEADER_HEIGHT as ::core::ffi::c_int);
        width += (2 as ::core::ffi::c_int)
            * (WIZARD_PADDING as ::core::ffi::c_int);
    }
    if ((*psInfo).ppshheader.dwFlags & (PSH_WIZARD as DWORD)) != 0 {
        height += (2 as ::core::ffi::c_int)
            * (WIZARD_PADDING as ::core::ffi::c_int);
        width += (2 as ::core::ffi::c_int)
            * (WIZARD_PADDING as ::core::ffi::c_int);
    }

    if resize != 0 {
        if width > (*psInfo).width {
            (*psInfo).width = width;
        }
        if height > (*psInfo).height {
            (*psInfo).height = height;
        }
    }

    /* menu */
    let swMenu: ::core::ffi::c_int = *p as ::core::ffi::c_int;
    match swMenu {
        _ if swMenu == 0x0000 as ::core::ffi::c_int => {
            p = p.offset(1);
        }
        _ if swMenu == 0xffff as ::core::ffi::c_int => {
            p = p.offset(2 as isize);
        }
        _ => {
            p = p.offset(
                (lstrlenW(p as LPCWSTR) + 1 as ::core::ffi::c_int)
                    as isize,
            );
        }
    }

    /* class */
    let swClass: ::core::ffi::c_int = *p as ::core::ffi::c_int;
    match swClass {
        _ if swClass == 0x0000 as ::core::ffi::c_int => {
            p = p.offset(1);
        }
        _ if swClass == 0xffff as ::core::ffi::c_int => {
            p = p.offset(2 as isize);
        }
        _ => {
            p = p.offset(
                (lstrlenW(p as LPCWSTR) + 1 as ::core::ffi::c_int)
                    as isize,
            );
        }
    }

    /* Extract the caption */
    (*(*psInfo).proppage.offset(index as isize)).pszText = p as LPCWSTR;

    if (dwFlags & (PSP_USETITLE as DWORD)) != 0 {
        let mut szTitle: [WCHAR; 256 as usize] =
            [0; 256 as usize];
        let mut pTitle: *const WCHAR = ::core::ptr::null();
        static pszNull: [WCHAR; 7] = [0x0028u16, 0x006E, 0x0075, 0x006C, 0x006C, 0x0029, 0x0000];

        if IS_INTRESOURCE!((*lppsp).pszTitle) != false {
            if LoadStringW(
                (*lppsp).hInstance,
                (*lppsp).pszTitle as DWORD_PTR as UINT,
                &raw mut szTitle as LPWSTR,
                ARRAY_SIZE!(szTitle) as ::core::ffi::c_int,
            ) != 0
            {
                pTitle = &raw mut szTitle as *mut WCHAR;
            } else if *p != 0 {
                pTitle = p as *const WCHAR;
            } else {
                pTitle = &raw const pszNull as *const WCHAR;
            }
        } else {
            pTitle = (*lppsp).pszTitle as *const WCHAR;
        }

        (*(*psInfo).proppage.offset(index as isize)).pszText =
            heap_strdupW(pTitle) as LPCWSTR;
    }

    /*
     * Build the image list for icons
     */
    if ((dwFlags & (PSP_USEHICON as DWORD)) != 0)
        || ((dwFlags & (PSP_USEICONID as DWORD)) != 0)
    {
        let mut hIcon: HICON = ::core::ptr::null_mut();
        let icon_cx: ::core::ffi::c_int = GetSystemMetrics((SM_CXSMICON) as i32);
        let icon_cy: ::core::ffi::c_int = GetSystemMetrics((SM_CYSMICON) as i32);

        if (dwFlags & (PSP_USEICONID as DWORD)) != 0 {
            hIcon = LoadImageW(
                (*lppsp).hInstance,
                (*lppsp).u2.pszIcon,
                IMAGE_ICON as UINT,
                icon_cx,
                icon_cy,
                LR_DEFAULTCOLOR as UINT,
            ) as HICON;
        } else {
            hIcon = (*lppsp).u2.hIcon;
        }

        if !hIcon.is_null() {
            if (*psInfo).hImageList == core::ptr::null_mut() {
                (*psInfo).hImageList = ImageList_Create(
                    icon_cx,
                    icon_cy,
                    ILC_COLOR as UINT,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }

            ImageList_AddIcon((*psInfo).hImageList, hIcon);
        }
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_CreateDialog(mut psInfo: *mut PropSheetInfo) -> INT_PTR {
    let mut ret: LRESULT = 0;
    let mut template: LPCVOID = core::ptr::null();
    let mut temp: LPVOID = core::ptr::null_mut();
    let mut hRes: HRSRC = core::ptr::null_mut();
    let mut resSize: DWORD = 0;
    let mut resID: WORD = IDD_PROPSHEET as WORD;

    if ((*psInfo).ppshheader.dwFlags & (INTRNL_ANY_WIZARD as DWORD)) != 0 {
        resID = IDD_WIZARD as WORD;
    }

    if (*psInfo).unicode != 0 {
        hRes = FindResourceW(
            COMCTL32_hModule,
            MAKEINTRESOURCEW!(resID),
            RT_DIALOG as LPWSTR as LPCWSTR,
        );
        if hRes.is_null() {
            return -(1) as INT_PTR;
        }
    } else {
        hRes = FindResourceA(
            COMCTL32_hModule,
            MAKEINTRESOURCEA!(resID),
            RT_DIALOG as LPSTR as LPCSTR,
        );
        if hRes.is_null() {
            return -(1) as INT_PTR;
        }
    }

    template = LoadResource(COMCTL32_hModule as HINSTANCE, hRes) as LPCVOID;
    if template.is_null() {
        return -(1) as INT_PTR;
    }

    /*
     * Make a copy of the dialog template.
     */
    resSize = SizeofResource(COMCTL32_hModule as HINSTANCE, hRes);

    temp = Alloc((2 as DWORD).wrapping_mul(resSize));

    if temp.is_null() {
        return -(1) as INT_PTR;
    }

    memcpy(
        temp as *mut ::core::ffi::c_void,
        template as *const ::core::ffi::c_void,
        resSize as usize,
    );

    if ((*psInfo).ppshheader.dwFlags & (PSH_NOCONTEXTHELP as DWORD)) != 0 {
        if (*(temp as *mut MyDLGTEMPLATEEX)).signature == 0xFFFF {
            (*(temp as *mut MyDLGTEMPLATEEX)).style &= !(DS_CONTEXTHELP as DWORD);
        } else {
            (*(temp as *mut DLGTEMPLATE)).style &= !(DS_CONTEXTHELP as DWORD);
        }
    }
    if (((*psInfo).ppshheader.dwFlags & (INTRNL_ANY_WIZARD as DWORD)) != 0)
        && (((*psInfo).ppshheader.dwFlags & (PSH_WIZARDCONTEXTHELP as DWORD)) != 0)
    {
        if (*(temp as *mut MyDLGTEMPLATEEX)).signature == 0xFFFF {
            (*(temp as *mut MyDLGTEMPLATEEX)).style |= (DS_CONTEXTHELP as DWORD);
        } else {
            (*(temp as *mut DLGTEMPLATE)).style |= (DS_CONTEXTHELP as DWORD);
        }
    }

    if (*psInfo).useCallback != 0 {
        (*psInfo)
            .ppshheader
            .pfnCallback
            .unwrap_unchecked()(
            core::ptr::null_mut(),
            PSCB_PRECREATE,
            temp as LPARAM,
        );
    }

    /* NOTE: MSDN states "Returns a positive value if successful, or -1
     * otherwise for modal property sheets.", but this is wrong. The
     * actual return value is either TRUE (success), FALSE (cancel) or
     * -1 (error). */
    if (*psInfo).unicode != 0 {
        ret = CreateDialogIndirectParamW(
            (*psInfo).ppshheader.hInstance,
            temp as LPCDLGTEMPLATE,
            (*psInfo).ppshheader.hwndParent,
            Some(
                PROPSHEET_DialogProc,
            ),
            psInfo as LPARAM,
        ) as INT_PTR as LRESULT;
        if ret == 0 {
            ret = -(1) as LRESULT;
        }
    } else {
        ret = CreateDialogIndirectParamA(
            (*psInfo).ppshheader.hInstance,
            temp as LPCDLGTEMPLATE,
            (*psInfo).ppshheader.hwndParent,
            Some(
                PROPSHEET_DialogProc,
            ),
            psInfo as LPARAM,
        ) as INT_PTR as LRESULT;
        if ret == 0 {
            ret = -(1) as LRESULT;
        }
    }

    Free(temp);

    return ret as INT_PTR;
}

unsafe extern "C" fn PROPSHEET_SizeMismatch(hwndDlg: HWND, psInfo: *const PropSheetInfo) -> BOOL {
    let hwndTabCtrl: HWND = GetDlgItem(hwndDlg, (IDC_TABCONTROL as i32));
    let mut rcOrigTab: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcPage: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    /*
     * Original tab size.
     */
    GetClientRect(hwndTabCtrl, core::ptr::addr_of_mut!(rcOrigTab));

    /*
     * Biggest page size.
     */
    SetRect(
        core::ptr::addr_of_mut!(rcPage),
        0,
        0,
        (*psInfo).width,
        (*psInfo).height,
    );
    MapDialogRect(hwndDlg, core::ptr::addr_of_mut!(rcPage));

    if (rcPage.right - rcPage.left) != (rcOrigTab.right - rcOrigTab.left) {
        return TRUE;
    }
    if (rcPage.bottom - rcPage.top) != (rcOrigTab.bottom - rcOrigTab.top) {
        return TRUE;
    }

    return FALSE;
}

unsafe extern "C" fn PROPSHEET_AdjustSize(hwndDlg: HWND, psInfo: *mut PropSheetInfo) -> BOOL {
    let mut hwndTabCtrl: HWND = GetDlgItem(hwndDlg, (IDC_TABCONTROL as i32));
    let mut hwndButton: HWND = GetDlgItem(hwndDlg, (IDOK) as i32);
    let mut rc: RECT = core::mem::zeroed();
    let mut tabRect: RECT = core::mem::zeroed();
    let mut buttonHeight: i32;
    let padding: PADDING_INFO = PROPSHEET_GetPaddingInfo(hwndDlg);
    let mut units: RECT = core::mem::zeroed();
    let mut style: LONG;

    /* Get the height of buttons */
    GetClientRect(hwndButton, core::ptr::addr_of_mut!(rc));
    buttonHeight = rc.bottom as i32;

    /*
     * Biggest page size.
     */
    SetRect(
        core::ptr::addr_of_mut!(rc),
        0,
        0,
        (*psInfo).width,
        (*psInfo).height,
    );
    MapDialogRect(hwndDlg, core::ptr::addr_of_mut!(rc));

    /* retrieve the dialog units */
    units.left = 4;
    units.right = units.left;
    units.top = 8;
    units.bottom = units.top;
    MapDialogRect(hwndDlg, core::ptr::addr_of_mut!(units));

    /*
     * Resize the tab control.
     */
    GetClientRect(hwndTabCtrl, core::ptr::addr_of_mut!(tabRect));

    SendMessageW(
        hwndTabCtrl,
        TCM_ADJUSTRECT,
        (FALSE) as WPARAM,
        core::ptr::addr_of_mut!(tabRect) as LPARAM,
    );

    if (rc.bottom - rc.top) < (tabRect.bottom - tabRect.top) {
        rc.bottom = rc.top + tabRect.bottom - tabRect.top;
        (*psInfo).height = MulDiv(
            (rc.bottom - rc.top),
            8,
            units.top,
        );
    }

    if (rc.right - rc.left) < (tabRect.right - tabRect.left) {
        rc.right = rc.left + tabRect.right - tabRect.left;
        (*psInfo).width = MulDiv(
            (rc.right - rc.left),
            4,
            units.left,
        );
    }

    SendMessageW(
        hwndTabCtrl,
        TCM_ADJUSTRECT,
        (TRUE) as WPARAM,
        core::ptr::addr_of_mut!(rc) as LPARAM,
    );

    rc.right -= rc.left;
    rc.bottom -= rc.top;

    SetWindowPos(
        hwndTabCtrl,
        core::ptr::null_mut(),
        0,
        0,
        rc.right,
        rc.bottom,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    );

    GetClientRect(hwndTabCtrl, core::ptr::addr_of_mut!(rc));

    rc.right += padding.x * 2;
    rc.bottom += buttonHeight + (3 * padding.y);

    style = GetWindowLongW(hwndDlg, GWL_STYLE);
    if (style & (WS_CHILD as LONG)) == 0 {
        AdjustWindowRect(
            core::ptr::addr_of_mut!(rc),
            style as DWORD,
            FALSE,
        );
    }

    rc.right -= rc.left;
    rc.bottom -= rc.top;

    /*
     * Resize the property sheet.
     */
    SetWindowPos(
        hwndDlg,
        core::ptr::null_mut(),
        0,
        0,
        rc.right,
        rc.bottom,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    );
    return TRUE;
}

unsafe extern "C" fn PROPSHEET_AdjustSizeWizard(hwndDlg: HWND, psInfo: *const PropSheetInfo) -> BOOL {
    let mut hwndLine: HWND = GetDlgItem(hwndDlg, (IDC_SUNKEN_LINE as i32));
    let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut lineRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut dialogRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    /* Biggest page size */
    SetRect(core::ptr::addr_of_mut!(rc), 0, 0, (*psInfo).width, (*psInfo).height);
    MapDialogRect(hwndDlg, core::ptr::addr_of_mut!(rc));

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0".as_ptr() as *const i8,
            b"PROPSHEET_AdjustSizeWizard\0".as_ptr() as *const i8,
            836i32,
            b"Biggest page %s\n\0".as_ptr() as *const i8,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(rc)),
        );
    }

    /* Add space for the buttons row */
    GetWindowRect(hwndLine, core::ptr::addr_of_mut!(lineRect));
    MapWindowPoints(core::ptr::null_mut(), hwndDlg, core::ptr::addr_of_mut!(lineRect) as LPPOINT, 2);
    GetClientRect(hwndDlg, core::ptr::addr_of_mut!(dialogRect));
    rc.bottom += dialogRect.bottom - lineRect.top - 1;

    /* Convert the client coordinates to window coordinates */
    AdjustWindowRect(core::ptr::addr_of_mut!(rc), GetWindowLongW(hwndDlg, GWL_STYLE) as DWORD, FALSE);

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/propsheet.c\0".as_ptr() as *const i8,
            b"PROPSHEET_AdjustSizeWizard\0".as_ptr() as *const i8,
            849i32,
            b"setting dialog %p, rc (0,0)-(%d,%d)\n\0".as_ptr() as *const i8,
            hwndDlg,
            rc.right,
            rc.bottom,
        );
    }

    /* Resize the property sheet */
    SetWindowPos(
        hwndDlg,
        core::ptr::null_mut(),
        0,
        0,
        rc.right - rc.left,
        rc.bottom - rc.top,
        SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
    );

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_AdjustButtons(hwndParent: HWND, psInfo: *const PropSheetInfo) -> BOOL {
    let mut hwndButton: HWND = GetDlgItem(hwndParent, (IDOK) as i32);
    let mut rcSheet: RECT = core::mem::zeroed();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut num_buttons: i32 = 2;
    let mut buttonWidth: i32 = 0;
    let mut buttonHeight: i32 = 0;
    let padding: PADDING_INFO = PROPSHEET_GetPaddingInfo(hwndParent);

    if (*psInfo).hasApply != 0 {
        num_buttons += 1;
    }

    if (*psInfo).hasHelp != 0 {
        num_buttons += 1;
    }

    /*
     * Obtain the size of the buttons.
     */
    GetClientRect(hwndButton, core::ptr::addr_of_mut!(rcSheet));
    buttonWidth = rcSheet.right;
    buttonHeight = rcSheet.bottom;

    /*
     * Get the size of the property sheet.
     */
    GetClientRect(hwndParent, core::ptr::addr_of_mut!(rcSheet));

    /*
     * All buttons will be at this y coordinate.
     */
    y = rcSheet.bottom - (padding.y + buttonHeight);

    /*
     * Position OK button and make it default.
     */
    hwndButton = GetDlgItem(hwndParent, (IDOK) as i32);

    x = rcSheet.right - ((padding.x + buttonWidth) * num_buttons);

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    SendMessageW(hwndParent, DM_SETDEFID, (IDOK) as WPARAM, 0);

    /*
     * Position Cancel button.
     */
    hwndButton = GetDlgItem(hwndParent, (IDCANCEL) as i32);

    x += padding.x + buttonWidth;

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    /*
     * Position Apply button.
     */
    hwndButton = GetDlgItem(hwndParent, (IDC_APPLY_BUTTON as i32));

    if (*psInfo).hasApply != 0 {
        x += padding.x + buttonWidth;
    } else {
        ShowWindow(hwndButton, (SW_HIDE) as i32);
    }

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    EnableWindow(hwndButton, FALSE);

    /*
     * Position Help button.
     */
    hwndButton = GetDlgItem(hwndParent, (IDHELP) as i32);

    x += padding.x + buttonWidth;
    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    if (*psInfo).hasHelp == 0 {
        ShowWindow(hwndButton, (SW_HIDE) as i32);
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_AdjustButtonsWizard(hwndParent: HWND, psInfo: *const PropSheetInfo) -> BOOL {
    let mut hwndButton: HWND = GetDlgItem(hwndParent, (IDCANCEL) as i32);
    let mut hwndLine: HWND = GetDlgItem(hwndParent, (IDC_SUNKEN_LINE as i32));
    let mut hwndLineHeader: HWND = GetDlgItem(hwndParent, (IDC_SUNKEN_LINEHEADER as i32));
    let mut rcSheet: RECT = core::mem::zeroed();
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut num_buttons: ::core::ffi::c_int = 3;
    let mut buttonWidth: ::core::ffi::c_int = 0;
    let mut buttonHeight: ::core::ffi::c_int = 0;
    let mut lineHeight: ::core::ffi::c_int = 0;
    let mut lineWidth: ::core::ffi::c_int = 0;
    let mut padding: PADDING_INFO = PROPSHEET_GetPaddingInfoWizard(hwndParent, psInfo);

    if (*psInfo).hasHelp != 0 {
        num_buttons += 1;
    }
    if (*psInfo).hasFinish != 0 {
        num_buttons += 1;
    }

    GetClientRect(hwndButton, core::ptr::addr_of_mut!(rcSheet));
    buttonWidth = rcSheet.right;
    buttonHeight = rcSheet.bottom;

    GetClientRect(hwndLine, core::ptr::addr_of_mut!(rcSheet));
    lineHeight = rcSheet.bottom;

    GetClientRect(hwndParent, core::ptr::addr_of_mut!(rcSheet));

    y = rcSheet.bottom - (padding.y + buttonHeight);

    hwndButton = GetDlgItem(hwndParent, (IDC_BACK_BUTTON as i32));

    x = rcSheet.right - ((padding.x + buttonWidth) * (num_buttons - 1)) - buttonWidth;

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    hwndButton = GetDlgItem(hwndParent, (IDC_NEXT_BUTTON as i32));

    x += buttonWidth;

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    hwndButton = GetDlgItem(hwndParent, (IDC_FINISH_BUTTON as i32));

    if (*psInfo).hasFinish != 0 {
        x += padding.x + buttonWidth;
    }

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    if (*psInfo).hasFinish == 0 {
        ShowWindow(hwndButton, (SW_HIDE) as i32);
    }

    hwndButton = GetDlgItem(hwndParent, (IDCANCEL) as i32);

    x += padding.x + buttonWidth;

    SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);

    hwndButton = GetDlgItem(hwndParent, (IDHELP) as i32);

    if (*psInfo).hasHelp != 0 {
        x += padding.x + buttonWidth;

        SetWindowPos(hwndButton, core::ptr::null_mut(), x, y, 0, 0,
                     SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    } else {
        ShowWindow(hwndButton, (SW_HIDE) as i32);
    }

    if ((*psInfo).ppshheader.dwFlags &
        (PSH_WIZARD97_OLD | PSH_WIZARD97_NEW | PSH_WIZARD_LITE)) != 0 {
        padding.x = 0;
    }

    x = padding.x;
    y = rcSheet.bottom - ((padding.y * 2) + buttonHeight + lineHeight);

    lineWidth = rcSheet.right - (padding.x * 2);
    SetWindowPos(hwndLine, core::ptr::null_mut(), x, y, lineWidth, 2,
                 SWP_NOZORDER | SWP_NOACTIVATE);

    SetWindowPos(hwndLineHeader, core::ptr::null_mut(), 0, 0, rcSheet.right, 2,
                 SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE);

    if ((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_OLD | PSH_WIZARD97_NEW)) == 0 {
        ShowWindow(hwndLineHeader, (SW_HIDE) as i32);
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_GetPaddingInfo(hwndDlg: HWND) -> PADDING_INFO {
    let hwndTab: HWND = GetDlgItem(hwndDlg, (IDC_TABCONTROL as i32));
    let mut rcTab: RECT = core::mem::zeroed();
    let mut padding: PADDING_INFO = PADDING_INFO { x: 0, y: 0 };

    GetWindowRect(hwndTab, core::ptr::addr_of_mut!(rcTab));
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        core::ptr::addr_of_mut!(rcTab) as LPPOINT,
        2,
    );

    padding.x = rcTab.left as core::ffi::c_int;
    padding.y = rcTab.top as core::ffi::c_int;

    return padding;
}

unsafe extern "C" fn PROPSHEET_GetPaddingInfoWizard(
    mut hwndDlg: HWND,
    mut psInfo: *const PropSheetInfo,
) -> PADDING_INFO {
    let mut padding: PADDING_INFO = PADDING_INFO { x: 0, y: 0 };
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hwndControl: HWND = core::ptr::null_mut();
    let mut idButton: INT = 0;
    let mut ptButton: POINT = POINT { x: 0, y: 0 };
    let mut ptLine: POINT = POINT { x: 0, y: 0 };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_GetPaddingInfoWizard\0" as *const u8
                as *const ::core::ffi::c_char,
            1113 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*psInfo).hasHelp != 0 {
        idButton = IDHELP as INT;
    } else {
        if (*psInfo).ppshheader.dwFlags & INTRNL_ANY_WIZARD as DWORD != 0 {
            idButton = IDC_NEXT_BUTTON as INT;
        } else {
            /* hopefully this is ok */
            idButton = IDCANCEL as INT;
        }
    }

    hwndControl = GetDlgItem(hwndDlg, idButton);
    GetWindowRect(hwndControl, core::ptr::addr_of_mut!(rc));
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        core::ptr::addr_of_mut!(rc) as LPPOINT,
        2 as UINT,
    );
    ptButton.x = rc.left;
    ptButton.y = rc.top;

    /* Line */
    hwndControl = GetDlgItem(hwndDlg, (IDC_SUNKEN_LINE as i32));
    GetWindowRect(hwndControl, core::ptr::addr_of_mut!(rc));
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDlg,
        core::ptr::addr_of_mut!(rc) as LPPOINT,
        2 as UINT,
    );
    ptLine.x = rc.left;
    ptLine.y = rc.bottom;

    padding.y = (ptButton.y - ptLine.y) as ::core::ffi::c_int;

    if padding.y < 0 as ::core::ffi::c_int {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/propsheet.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PROPSHEET_GetPaddingInfoWizard\0" as *const u8
                    as *const ::core::ffi::c_char,
                1147 as ::core::ffi::c_int,
                b"padding negative ! Please report this !\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }

    /* this is most probably not correct, but the best we have now */
    padding.x = padding.y;
    return padding;
}

unsafe extern "C" fn PROPSHEET_CreateTabControl(hwndParent: HWND, psInfo: *const PropSheetInfo) -> BOOL {
    let mut hwndTabCtrl: HWND = GetDlgItem(hwndParent, (IDC_TABCONTROL as i32));
    let mut item: TCITEMW = core::mem::zeroed::<TCITEMW>();
    let mut i: core::ffi::c_int = 0;
    let mut nTabs: core::ffi::c_int = 0;
    let mut iImage: core::ffi::c_int = 0;

    item.mask = TCIF_TEXT;
    item.cchTextMax = MAX_TABTEXT_LENGTH;

    nTabs = (*psInfo).nPages;

    if !(*psInfo).hImageList.is_null() {
        SendMessageW(hwndTabCtrl, TCM_SETIMAGELIST, 0, (*psInfo).hImageList as LPARAM);
    }

    SendMessageW(hwndTabCtrl, WM_SETREDRAW, 0, 0);
    i = 0;
    while i < nTabs {
        if (*(*psInfo).proppage.offset(i as isize)).hasIcon != 0 {
            item.mask |= TCIF_IMAGE;
            let fresh24 = iImage;
            iImage = iImage + 1;
            item.iImage = fresh24;
        } else {
            item.mask &= !TCIF_IMAGE;
        }

        item.pszText = (*(*psInfo).proppage.offset(i as isize)).pszText as LPWSTR;
        SendMessageW(hwndTabCtrl, TCM_INSERTITEMW, i as WPARAM, &raw mut item as LPARAM);
        i += 1;
    }
    SendMessageW(hwndTabCtrl, WM_SETREDRAW, 1, 0);

    return TRUE;
}
unsafe extern "system" fn PROPSHEET_WizardSubclassProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uID: UINT_PTR,
    dwRef: DWORD_PTR,
) -> LRESULT {
    match uMsg {
        WM_ERASEBKGND => {
            return TRUE as LRESULT;
        }
        WM_CTLCOLORSTATIC => {
            SetBkColor(wParam as HDC, GetSysColor((COLOR_WINDOW) as i32) as COLORREF);
            return GetSysColorBrush((COLOR_WINDOW) as i32) as INT_PTR as LRESULT;
        }
        _ => {}
    }

    return DefSubclassProc(hwnd, uMsg, wParam, lParam);
}

unsafe extern "C" fn GetTemplateSize(pTemplate: *const DLGTEMPLATE) -> UINT {
    let mut p: *const WORD = pTemplate as *const WORD;
    let istemplateex: BOOL = (((*(pTemplate as *const MyDLGTEMPLATEEX)).signature as i32)
        == 0xFFFF) as i32;
    let mut nrofitems: WORD = 0;
    let mut ret: UINT = 0;

    if istemplateex != 0 {
        p = p.offset(1);
        p = p.offset(1);
        p = p.offset(2 as isize);
        p = p.offset(2 as isize);
        p = p.offset(2 as isize);
    } else {
        p = p.offset(2 as isize);
        p = p.offset(2 as isize);
    }

    nrofitems = *p;
    p = p.offset(1);
    p = p.offset(1);
    p = p.offset(1);
    p = p.offset(1);
    p = p.offset(1);

    /* menu */
    match *p {
        0x0000 => {
            p = p.offset(1);
        }
        0xffff => {
            p = p.offset(2 as isize);
        }
        _ => {
            let _ = debugstr_w(p as *const WCHAR);
            p = p.offset(
                (lstrlenW(p as LPCWSTR) + 1) as isize,
            );
        }
    }

    /* class */
    match *p {
        0x0000 => {
            p = p.offset(1);
        }
        0xffff => {
            p = p.offset(2 as isize);
        }
        _ => {
            let _ = debugstr_w(p as *const WCHAR);
            p = p.offset(
                (lstrlenW(p as LPCWSTR) + 1) as isize,
            );
        }
    }

    /* title */
    let _ = 0;
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);

    /* font, if DS_SETFONT set */
    if (DS_SETFONT as DWORD
        & (if istemplateex != 0 {
            (*(pTemplate as *const MyDLGTEMPLATEEX)).style
        } else {
            (*pTemplate).style
        }))
        != 0
    {
        p = p.offset(
            (if istemplateex != 0 {
                3
            } else {
                1
            }) as isize,
        );
        p = p.offset(
            (lstrlenW(p as LPCWSTR) + 1) as isize,
        );
    }

    /* now process the DLGITEMTEMPLATE(EX) structs (plus custom data)
     * that are following the DLGTEMPLATE(EX) data */
    while (nrofitems as i32) > 0 {
        p = (((p as DWORD_PTR).wrapping_add(3 as DWORD_PTR))
            & !(3 as DWORD_PTR)) as *const WORD;

        /* skip header */
        p = p.offset(
            ((if istemplateex != 0 {
                ::core::mem::size_of::<MyDLGITEMTEMPLATEEX>()
            } else {
                ::core::mem::size_of::<DLGITEMTEMPLATE>()
            }) / ::core::mem::size_of::<WORD>()) as isize,
        );

        /* check class */
        match *p {
            0x0000 => {
                p = p.offset(1);
            }
            0xffff => {
                p = p.offset(2 as isize);
            }
            _ => {
                let _ = debugstr_w(p as *const WCHAR);
                p = p.offset(
                    (lstrlenW(p as LPCWSTR) + 1) as isize,
                );
            }
        }

        /* check title text */
        match *p {
            0x0000 => {
                p = p.offset(1);
            }
            0xffff => {
                p = p.offset(2 as isize);
            }
            _ => {
                let _ = debugstr_w(p as *const WCHAR);
                p = p.offset(
                    (lstrlenW(p as LPCWSTR) + 1) as isize,
                );
            }
        }

        p = p.offset(
            ((*p as usize) / ::core::mem::size_of::<WORD>()
                + 1 as usize) as isize,
        ); /* Skip extra data */
        nrofitems = nrofitems.wrapping_sub(1);
    }

    ret = ((p as *const WORD)
        .offset_from(pTemplate as *const WORD) as usize
        * ::core::mem::size_of::<WORD>()) as UINT;

    return ret;
}

unsafe extern "C" fn PROPSHEET_CreatePage(
    mut hwndParent: HWND,
    mut index: ::core::ffi::c_int,
    mut psInfo: *const PropSheetInfo,
    mut ppshpage: LPCPROPSHEETPAGEW,
) -> BOOL {
    let mut pTemplate: *const DLGTEMPLATE = core::ptr::null();
    let mut hwndPage: HWND = core::ptr::null_mut();
    let mut resSize: DWORD = 0;
    let mut pTemplateCopy: *mut DLGTEMPLATE = core::ptr::null_mut();

    if ppshpage.is_null() {
        return FALSE;
    }

    if ((*ppshpage).dwFlags & (PSP_DLGINDIRECT as DWORD)) != 0 {
        pTemplate = (*ppshpage).u.pResource as *const DLGTEMPLATE;
        resSize = GetTemplateSize(pTemplate) as DWORD;
    } else if ((*ppshpage).dwFlags & (PSP_INTERNAL_UNICODE as DWORD)) != 0 {
        let mut hResource: HRSRC = core::ptr::null_mut();
        let mut hTemplate: HANDLE = core::ptr::null_mut();

        hResource = FindResourceW(
            (*ppshpage).hInstance as HMODULE,
            (*ppshpage).u.pszTemplate,
            (RT_DIALOG as usize) as LPWSTR as LPCWSTR,
        );
        if hResource.is_null() {
            return FALSE;
        }

        resSize = SizeofResource((*ppshpage).hInstance as HMODULE, hResource) as DWORD;

        hTemplate = LoadResource((*ppshpage).hInstance as HMODULE, hResource) as HANDLE;
        if hTemplate.is_null() {
            return FALSE;
        }

        pTemplate = LockResource(hTemplate as HGLOBAL) as *const DLGTEMPLATE;
        /*
         * Make a copy of the dialog template to make it writable
         */
    } else {
        let mut hResource: HRSRC = core::ptr::null_mut();
        let mut hTemplate: HANDLE = core::ptr::null_mut();

        hResource = FindResourceA(
            (*ppshpage).hInstance as HMODULE,
            (*ppshpage).u.pszTemplate as LPCSTR,
            ((RT_DIALOG as usize) as LPSTR) as LPCSTR,
        );
        if hResource.is_null() {
            return FALSE;
        }

        resSize = SizeofResource((*ppshpage).hInstance as HMODULE, hResource) as DWORD;

        hTemplate = LoadResource((*ppshpage).hInstance as HMODULE, hResource) as HANDLE;
        if hTemplate.is_null() {
            return FALSE;
        }

        pTemplate = LockResource(hTemplate as HGLOBAL) as *const DLGTEMPLATE;
        /*
         * Make a copy of the dialog template to make it writable
         */
    }
    pTemplateCopy = Alloc(resSize) as *mut DLGTEMPLATE;
    if pTemplateCopy.is_null() {
        return FALSE;
    }

    memcpy(
        pTemplateCopy as *mut ::core::ffi::c_void,
        pTemplate as *const ::core::ffi::c_void,
        resSize as usize,
    );

    if (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).signature == 0xFFFF {
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style |=
            (WS_CHILD as DWORD) | (WS_TABSTOP as DWORD) | (DS_CONTROL as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(DS_MODALFRAME as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(WS_CAPTION as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(WS_SYSMENU as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(WS_POPUP as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(WS_DISABLED as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(WS_VISIBLE as DWORD);
        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).style &= !(WS_THICKFRAME as DWORD);

        (*(pTemplateCopy as *mut MyDLGTEMPLATEEX)).exStyle |= WS_EX_CONTROLPARENT as DWORD;
    } else {
        (*pTemplateCopy).style |=
            (WS_CHILD as DWORD) | (WS_TABSTOP as DWORD) | (DS_CONTROL as DWORD);
        (*pTemplateCopy).style &= !(DS_MODALFRAME as DWORD);
        (*pTemplateCopy).style &= !(WS_CAPTION as DWORD);
        (*pTemplateCopy).style &= !(WS_SYSMENU as DWORD);
        (*pTemplateCopy).style &= !(WS_POPUP as DWORD);
        (*pTemplateCopy).style &= !(WS_DISABLED as DWORD);
        (*pTemplateCopy).style &= !(WS_VISIBLE as DWORD);
        (*pTemplateCopy).style &= !(WS_THICKFRAME as DWORD);

        (*pTemplateCopy).dwExtendedStyle |= WS_EX_CONTROLPARENT as DWORD;
    }

    if (*(*psInfo).proppage.offset(index as isize)).useCallback != 0 {
        ((*ppshpage).pfnCallback.unwrap_unchecked())(
            core::ptr::null_mut(),
            PSPCB_CREATE as UINT,
            ppshpage as *mut _PROPSHEETPAGEW,
        );
    }

    if ((*ppshpage).dwFlags & (PSP_INTERNAL_UNICODE as DWORD)) != 0 {
        hwndPage = CreateDialogIndirectParamW(
            (*ppshpage).hInstance,
            pTemplateCopy as LPCDLGTEMPLATE,
            hwndParent,
            (*ppshpage).pfnDlgProc,
            ppshpage as LPARAM,
        );
    } else {
        hwndPage = CreateDialogIndirectParamA(
            (*ppshpage).hInstance,
            pTemplateCopy as LPCDLGTEMPLATE,
            hwndParent,
            (*ppshpage).pfnDlgProc,
            ppshpage as LPARAM,
        );
    }
    /* Free a no more needed copy */
    Free(pTemplateCopy as LPVOID);

    if hwndPage.is_null() {
        return FALSE;
    }

    (*(*psInfo).proppage.offset(index as isize)).hwndPage = hwndPage;

    /* Subclass exterior wizard pages */
    if ((*psInfo).ppshheader.dwFlags & ((PSH_WIZARD97_NEW | PSH_WIZARD97_OLD) as DWORD)) != 0
        && ((*psInfo).ppshheader.dwFlags & (PSH_WATERMARK as DWORD)) != 0
        && ((*ppshpage).dwFlags & (PSP_HIDEHEADER as DWORD)) != 0
    {
        if !(*psInfo).ppshheader.u4.hbmWatermark.is_null() {
            SetWindowSubclass(
                hwndPage,
                Some(
                    PROPSHEET_WizardSubclassProc,
                ),
                1 as UINT_PTR,
                ppshpage as DWORD_PTR,
            );
        }
    }
    if ((*psInfo).ppshheader.dwFlags & (INTRNL_ANY_WIZARD as DWORD)) == 0 {
        EnableThemeDialogTexture(hwndPage, ETDT_ENABLETAB as DWORD);
    }

    PROPSHEET_UnChanged(hwndParent, hwndPage);
    return TRUE;
}

unsafe extern "C" fn PROPSHEET_LoadWizardBitmaps(psInfo: *mut PropSheetInfo) {
    if (*psInfo).ppshheader.dwFlags & ((PSH_WIZARD97_NEW as DWORD) | (PSH_WIZARD97_OLD as DWORD)) != 0 {
        /* if PSH_USEHBMWATERMARK is not set, load the resource from pszbmWatermark
           and put the HBITMAP in hbmWatermark. Thus all the rest of the code always
           considers hbmWatermark as valid. */
        if ((*psInfo).ppshheader.dwFlags & (PSH_WATERMARK as DWORD) != 0)
            && ((*psInfo).ppshheader.dwFlags & (PSH_USEHBMWATERMARK as DWORD) == 0)
        {
            (*psInfo).ppshheader.u4.hbmWatermark = CreateMappedBitmap(
                (*psInfo).ppshheader.hInstance,
                (*psInfo).ppshheader.u4.pszbmWatermark as INT_PTR,
                0 as UINT,
                core::ptr::null_mut(),
                0 as core::ffi::c_int,
            );
        }

        /* Same behavior as for watermarks */
        if ((*psInfo).ppshheader.dwFlags & (PSH_HEADER as DWORD) != 0)
            && ((*psInfo).ppshheader.dwFlags & (PSH_USEHBMHEADER as DWORD) == 0)
        {
            (*psInfo).ppshheader.u5.hbmHeader = CreateMappedBitmap(
                (*psInfo).ppshheader.hInstance,
                (*psInfo).ppshheader.u5.pszbmHeader as INT_PTR,
                0 as UINT,
                core::ptr::null_mut(),
                0 as core::ffi::c_int,
            );
        }
    }
}

unsafe extern "C" fn PROPSHEET_ShowPage(
    mut hwndDlg: HWND,
    mut index: ::core::ffi::c_int,
    mut psInfo: *mut PropSheetInfo,
) -> BOOL {
    let mut hwndTabCtrl: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut hwndLineHeader: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut ppshpage: LPCPROPSHEETPAGEW = ::core::ptr::null::<PROPSHEETPAGEW>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_ShowPage\0" as *const u8 as *const ::core::ffi::c_char,
            1554 as ::core::ffi::c_int,
            b"active_page %d, index %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*psInfo).active_page,
            index,
        );
    }
    if index == (*psInfo).active_page {
        if GetTopWindow(hwndDlg)
            != (*(*psInfo).proppage.offset(index as isize)).hwndPage
        {
            SetWindowPos(
                (*(*psInfo).proppage.offset(index as isize)).hwndPage,
                HWND_TOP,
                0,
                0,
                0,
                0,
                (SWP_NOSIZE | SWP_NOMOVE) as UINT,
            );
        }
        return TRUE;
    }

    ppshpage = (*(*psInfo).proppage.offset(index as isize)).hpage as LPCPROPSHEETPAGEW;
    if (*(*psInfo).proppage.offset(index as isize)).hwndPage
        == 0 as HWND
    {
        PROPSHEET_CreatePage(hwndDlg, index, psInfo, ppshpage);
    }

    if (*psInfo).ppshheader.dwFlags & INTRNL_ANY_WIZARD as DWORD != 0 {
        PROPSHEET_SetTitleW(
            hwndDlg,
            (*psInfo).ppshheader.dwFlags,
            (*(*psInfo).proppage.offset(index as isize)).pszText,
        );
    }

    if (*psInfo).active_page != -1 {
        ShowWindow(
            (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage,
            (SW_HIDE) as i32,
        );
    }

    ShowWindow(
        (*(*psInfo).proppage.offset(index as isize)).hwndPage,
        (SW_SHOW) as i32,
    );

    /* Synchronize current selection with tab control
     * It seems to be needed even in case of PSH_WIZARD (no tab controls there) */
    hwndTabCtrl = GetDlgItem(hwndDlg, (IDC_TABCONTROL as i32));
    SendMessageW(
        hwndTabCtrl,
        TCM_SETCURSEL,
        index as WPARAM,
        0 as LPARAM,
    );

    (*psInfo).active_page = index;
    (*psInfo).activeValid = TRUE as BOOL;

    PROPSHEET_SetInitialFocus(hwndDlg, index, psInfo);

    if (*psInfo).ppshheader.dwFlags
        & ((PSH_WIZARD97_OLD | PSH_WIZARD97_NEW) as DWORD)
        != 0
    {
        hwndLineHeader = GetDlgItem(hwndDlg, (IDC_SUNKEN_LINEHEADER as i32));
        ppshpage = (*(*psInfo).proppage.offset(index as isize)).hpage as LPCPROPSHEETPAGEW;

        if (*ppshpage).dwFlags & PSP_HIDEHEADER as DWORD != 0
            || (*psInfo).ppshheader.dwFlags & PSH_HEADER as DWORD == 0
        {
            ShowWindow(hwndLineHeader, (SW_HIDE) as i32);
        } else {
            ShowWindow(hwndLineHeader, (SW_SHOW) as i32);
        }
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_Back(mut hwndDlg: HWND) -> BOOL {
    let mut psn: PSHNOTIFY = _PSHNOTIFY {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        lParam: 0,
    };
    let mut hwndPage: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
    let mut result: LRESULT = 0;
    let mut idx: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_Back\0" as *const u8 as *const ::core::ffi::c_char,
            1622 as ::core::ffi::c_int,
            b"active_page %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*psInfo).active_page,
        );
    }
    if (*psInfo).active_page < 0 as ::core::ffi::c_int {
        return FALSE;
    }
    psn.hdr.code = PSN_WIZBACK as UINT;
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0 as UINT_PTR;
    psn.lParam = 0 as LPARAM;
    hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
    result = SendMessageW(
        hwndPage,
        WM_NOTIFY as UINT,
        0 as WPARAM,
        &raw mut psn as LPARAM,
    );
    if result == -(1 as ::core::ffi::c_int) as LRESULT {
        return FALSE;
    } else if result == 0 as LRESULT {
        idx = (*psInfo).active_page - 1 as ::core::ffi::c_int;
    } else {
        idx = PROPSHEET_FindPageByResId(psInfo, result) as ::core::ffi::c_int;
    }
    if idx >= 0 as ::core::ffi::c_int && idx < (*psInfo).nPages {
        if PROPSHEET_CanSetCurSel(hwndDlg) != 0 {
            SetFocus(GetDlgItem(
                hwndDlg,
                IDC_BACK_BUTTON as ::core::ffi::c_int,
            ));
            SendMessageW(
                hwndDlg,
                DM_SETDEFID as UINT,
                IDC_BACK_BUTTON as WPARAM,
                0 as LPARAM,
            );
            PROPSHEET_SetCurSel(
                hwndDlg,
                idx,
                -(1 as ::core::ffi::c_int),
                core::ptr::null_mut(),
            );
        }
    }
    return TRUE;
}

unsafe extern "C" fn PROPSHEET_Next(mut hwndDlg: HWND) -> BOOL {
    let mut psn: PSHNOTIFY = core::mem::zeroed();
    let mut hwndPage: HWND = core::ptr::null_mut();
    let mut msgResult: LRESULT = 0 as LRESULT;
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut idx: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_Next\0" as *const u8 as *const ::core::ffi::c_char,
            1664 as ::core::ffi::c_int,
            b"active_page %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*psInfo).active_page,
        );
    }
    if (*psInfo).active_page < 0 as ::core::ffi::c_int {
        return FALSE;
    }
    psn.hdr.code = PSN_WIZNEXT as UINT;
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0 as UINT_PTR;
    psn.lParam = 0 as LPARAM;

    hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;

    msgResult = SendMessageW(
        hwndPage,
        WM_NOTIFY as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(psn) as LPARAM,
    );
    if msgResult == (-(1 as ::core::ffi::c_int)) as LRESULT {
        return FALSE;
    } else if msgResult == 0 as LRESULT {
        idx = (*psInfo).active_page + 1 as ::core::ffi::c_int;
    } else {
        idx = PROPSHEET_FindPageByResId(psInfo, msgResult) as ::core::ffi::c_int;
    }

    if idx < (*psInfo).nPages {
        if PROPSHEET_CanSetCurSel(hwndDlg) != FALSE {
            SetFocus(GetDlgItem(hwndDlg, IDC_NEXT_BUTTON as ::core::ffi::c_int));
            SendMessageW(
                hwndDlg,
                DM_SETDEFID as UINT,
                IDC_NEXT_BUTTON as WPARAM,
                0 as LPARAM,
            );
            PROPSHEET_SetCurSel(
                hwndDlg,
                idx,
                1 as ::core::ffi::c_int,
                core::ptr::null_mut(),
            );
        }
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_Finish(hwndDlg: HWND) -> BOOL {
    let mut psn: PSHNOTIFY = core::mem::zeroed();
    let mut hwndPage: HWND = core::ptr::null_mut();
    let mut msgResult: LRESULT = 0;
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    if (*psInfo).active_page < 0 {
        return FALSE;
    }

    psn.hdr.code = (PSN_WIZFINISH) as u32;
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0;
    psn.lParam = 0;

    hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;

    msgResult = SendMessageW(
        hwndPage,
        WM_NOTIFY,
        0,
        core::ptr::addr_of_mut!(psn) as LPARAM,
    );

    if msgResult != 0 {
        return FALSE;
    }

    if (*psInfo).result == 0 {
        (*psInfo).result = (IDOK) as i32;
    }
    if (*psInfo).isModeless != 0 {
        (*psInfo).activeValid = FALSE;
    } else {
        (*psInfo).ended = TRUE;
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_Apply(mut hwndDlg: HWND, mut lParam: LPARAM) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    let mut hwndPage: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut psn: PSHNOTIFY = _PSHNOTIFY {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        lParam: 0,
    };
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_Apply\0" as *const u8 as *const ::core::ffi::c_char,
            1744 as ::core::ffi::c_int,
            b"active_page %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*psInfo).active_page,
        );
    }
    if (*psInfo).active_page < 0 {
        return FALSE;
    }
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0 as UINT_PTR;
    psn.lParam = 0 as LPARAM;
    psn.hdr.code = PSN_KILLACTIVE as UINT;
    hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
    if SendMessageW(
        hwndPage,
        WM_NOTIFY as UINT,
        0 as WPARAM,
        &raw mut psn as LPARAM,
    ) != FALSE as LRESULT
    {
        return FALSE;
    }
    psn.hdr.code = PSN_APPLY as UINT;
    psn.lParam = lParam;
    i = 0 as ::core::ffi::c_int;
    while i < (*psInfo).nPages {
        hwndPage = (*(*psInfo).proppage.offset(i as isize)).hwndPage;
        if !hwndPage.is_null() {
            's_98: {
                match (SendMessageW(
                    hwndPage,
                    WM_NOTIFY as UINT,
                    0 as WPARAM,
                    &raw mut psn as LPARAM,
                )) as u32 {
                    PSNRET_INVALID => {
                        PROPSHEET_ShowPage(hwndDlg, i, psInfo);
                    }
                    PSNRET_INVALID_NOCHANGEPAGE => {}
                    _ => {
                        break 's_98;
                    }
                }
                return FALSE;
            }
        }
        i += 1;
    }
    if lParam != 0 {
        (*psInfo).activeValid = FALSE as BOOL;
    } else if (*psInfo).active_page >= 0 as ::core::ffi::c_int {
        psn.hdr.code = PSN_SETACTIVE as UINT;
        psn.lParam = 0 as LPARAM;
        hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
        SendMessageW(
            hwndPage,
            WM_NOTIFY as UINT,
            0 as WPARAM,
            &raw mut psn as LPARAM,
        );
        PROPSHEET_SetInitialFocus(hwndPage, (*psInfo).active_page, psInfo);
    }
    return TRUE;
}

unsafe extern "C" fn PROPSHEET_Cancel(hwndDlg: HWND, lParam: LPARAM) {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndPage: HWND;
    let mut psn: PSHNOTIFY = core::mem::zeroed();
    let mut i: i32;

    if (*psInfo).active_page < 0 {
        return;
    }

    hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
    psn.hdr.code = (PSN_QUERYCANCEL) as u32;
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0;
    psn.lParam = 0;

    if SendMessageW(hwndPage, WM_NOTIFY, 0, core::ptr::addr_of_mut!(psn) as LPARAM) != 0 {
        return;
    }

    psn.hdr.code = (PSN_RESET) as u32;
    psn.lParam = lParam;

    i = 0;
    while i < (*psInfo).nPages {
        hwndPage = (*(*psInfo).proppage.offset(i as isize)).hwndPage;

        if !hwndPage.is_null() {
            SendMessageW(hwndPage, WM_NOTIFY, 0, core::ptr::addr_of_mut!(psn) as LPARAM);
        }
        i += 1;
    }

    if (*psInfo).isModeless != 0 {
        (*psInfo).activeValid = FALSE;
    } else {
        (*psInfo).ended = TRUE;
    }
}

unsafe extern "C" fn PROPSHEET_Help(hwndDlg: HWND) {
    let psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndPage: HWND;
    let mut psn: PSHNOTIFY = core::mem::zeroed();

    if (*psInfo).active_page < 0 {
        return;
    }

    hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
    psn.hdr.code = PSN_HELP as UINT;
    psn.hdr.hwndFrom = hwndDlg;
    psn.hdr.idFrom = 0 as UINT_PTR;
    psn.lParam = 0 as LPARAM;

    SendMessageW(hwndPage, WM_NOTIFY as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(psn) as LPARAM);
}

unsafe extern "C" fn PROPSHEET_Changed(hwndDlg: HWND, hwndDirtyPage: HWND) {
    let mut i: c_int;
    let psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    if psInfo.is_null() {
        return;
    }
    i = 0;
    while i < (*psInfo).nPages {
        if (*(*psInfo).proppage.offset(i as isize)).hwndPage == hwndDirtyPage {
            (*(*psInfo).proppage.offset(i as isize)).isDirty = TRUE;
        }
        i += 1;
    }

    if (*psInfo).hasApply != 0 {
        let hwndApplyBtn: HWND = GetDlgItem(hwndDlg, (IDC_APPLY_BUTTON as i32));

        EnableWindow(hwndApplyBtn, TRUE);
    }
}

unsafe extern "C" fn PROPSHEET_UnChanged(mut hwndDlg: HWND, mut hwndCleanPage: HWND) {
    let mut i: ::core::ffi::c_int = 0;
    let mut noPageDirty: BOOL = TRUE;
    let mut hwndApplyBtn: HWND = GetDlgItem(hwndDlg, (IDC_APPLY_BUTTON as i32));
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    if psInfo.is_null() {
        return;
    }
    i = 0 as ::core::ffi::c_int;
    while i < (*psInfo).nPages {
        if (*(*psInfo).proppage.offset(i as isize)).hwndPage == hwndCleanPage {
            (*(*psInfo).proppage.offset(i as isize)).isDirty = FALSE as BOOL;
        }
        if (*(*psInfo).proppage.offset(i as isize)).isDirty != 0 {
            noPageDirty = FALSE as BOOL;
        }
        i += 1;
    }
    if noPageDirty != 0 {
        EnableWindow(hwndApplyBtn, FALSE as BOOL);
    }
}

unsafe extern "C" fn PROPSHEET_PressButton(mut hwndDlg: HWND, mut buttonID: ::core::ffi::c_int) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_PressButton\0" as *const u8 as *const ::core::ffi::c_char,
            1933 as ::core::ffi::c_int,
            b"buttonID %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            buttonID,
        );
    }
    match (buttonID) as u32 {
        PSBTN_APPLYNOW => {
            PROPSHEET_DoCommand(hwndDlg, IDC_APPLY_BUTTON as WORD);
        }
        PSBTN_BACK => {
            PROPSHEET_Back(hwndDlg);
        }
        PSBTN_CANCEL => {
            PROPSHEET_DoCommand(hwndDlg, IDCANCEL as WORD);
        }
        PSBTN_FINISH => {
            PROPSHEET_Finish(hwndDlg);
        }
        PSBTN_HELP => {
            PROPSHEET_DoCommand(hwndDlg, IDHELP as WORD);
        }
        PSBTN_NEXT => {
            PROPSHEET_Next(hwndDlg);
        }
        PSBTN_OK => {
            PROPSHEET_DoCommand(hwndDlg, IDOK as WORD);
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/propsheet.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"PROPSHEET_PressButton\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1958 as ::core::ffi::c_int,
                    b"Invalid button index %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    buttonID,
                );
            }
        }
    }
}

unsafe extern "C" fn PROPSHEET_CanSetCurSel(hwndDlg: HWND) -> BOOL {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndPage: HWND = core::ptr::null_mut();
    let mut psn: PSHNOTIFY = core::mem::zeroed();
    let mut res: BOOL = FALSE;

    'end: {
        if psInfo.is_null() {
            res = FALSE;
            break 'end;
        }

        if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/propsheet.c\0" as *const u8 as *const core::ffi::c_char,
                b"PROPSHEET_CanSetCurSel\0" as *const u8 as *const core::ffi::c_char,
                1989 as i32,
                b"active_page %d\n\0" as *const u8 as *const core::ffi::c_char,
                (*psInfo).active_page,
            );
        }

        if (*psInfo).active_page < 0 {
            res = TRUE;
            break 'end;
        }

        /*
         * Notify the current page.
         */
        hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
        psn.hdr.code = (PSN_KILLACTIVE) as u32;
        psn.hdr.hwndFrom = hwndDlg;
        psn.hdr.idFrom = 0;
        psn.lParam = 0;

        res = (SendMessageW(hwndPage, WM_NOTIFY, 0, core::ptr::addr_of_mut!(psn) as LPARAM) == 0) as i32 as BOOL;
    }

    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/propsheet.c\0" as *const u8 as *const core::ffi::c_char,
            b"PROPSHEET_CanSetCurSel\0" as *const u8 as *const core::ffi::c_char,
            2008 as i32,
            b"<-- %d\n\0" as *const u8 as *const core::ffi::c_char,
            res,
        );
    }
    return res;
}

unsafe extern "C" fn PROPSHEET_SetCurSel(
    mut hwndDlg: HWND,
    mut index: ::core::ffi::c_int,
    mut skipdir: ::core::ffi::c_int,
    mut hpage: HPROPSHEETPAGE,
) -> BOOL {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndHelp: HWND = GetDlgItem(hwndDlg, IDHELP as c_int);
    let mut hwndTabControl: HWND = GetDlgItem(hwndDlg, IDC_TABCONTROL as c_int);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_SetCurSel\0" as *const u8 as *const ::core::ffi::c_char,
            2025 as ::core::ffi::c_int,
            b"index %d, skipdir %d, hpage %p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            index,
            skipdir,
            hpage,
        );
    }

    index = PROPSHEET_GetPageIndex(hpage, psInfo, index);

    if index < 0 || index >= (*psInfo).nPages {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/propsheet.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PROPSHEET_SetCurSel\0" as *const u8 as *const ::core::ffi::c_char,
                2031 as ::core::ffi::c_int,
                b"Could not find page to select!\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return FALSE;
    }

    /* unset active page while doing this transition. */
    if (*psInfo).active_page != -1 {
        ShowWindow(
            (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage,
            (SW_HIDE) as i32,
        );
    }
    (*psInfo).active_page = -1;

    while 1 != 0 {
        let mut result: ::core::ffi::c_int = 0;
        let mut psn: PSHNOTIFY = _PSHNOTIFY {
            hdr: tagNMHDR {
                hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                idFrom: 0,
                code: 0,
            },
            lParam: 0,
        };
        let mut rc: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut ppshpage: LPCPROPSHEETPAGEW =
            (*(*psInfo).proppage.offset(index as isize)).hpage as LPCPROPSHEETPAGEW;

        if !hwndTabControl.is_null() {
            SendMessageW(
                hwndTabControl,
                TCM_SETCURSEL as UINT,
                index as WPARAM,
                0 as LPARAM,
            );
        }

        psn.hdr.code = PSN_SETACTIVE as UINT;
        psn.hdr.hwndFrom = hwndDlg;
        psn.hdr.idFrom = 0 as UINT_PTR;
        psn.lParam = 0 as LPARAM;

        if (*(*psInfo).proppage.offset(index as isize)).hwndPage.is_null() {
            if PROPSHEET_CreatePage(hwndDlg, index, psInfo, ppshpage) == 0 {
                PROPSHEET_RemovePage(hwndDlg, index, core::ptr::null_mut::<HPROPSHEETPAGE__>());

                if (*psInfo).isModeless == 0 {
                    DestroyWindow(hwndDlg);
                    return FALSE;
                }

                if index >= (*psInfo).nPages {
                    index -= 1;
                }
                if index < 0 {
                    return FALSE;
                }
                continue;
            }
        }

        /* Resize the property sheet page to the fit in the Tab control
         * (for regular property sheets) or to fit in the client area (for
         * wizards).
         * NOTE: The resizing happens every time the page is selected and
         * not only when it's created (some applications depend on it). */
        PROPSHEET_GetPageRect(psInfo, hwndDlg, core::ptr::addr_of_mut!(rc), ppshpage);

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/propsheet.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PROPSHEET_SetCurSel\0" as *const u8 as *const ::core::ffi::c_char,
                2080 as ::core::ffi::c_int,
                b"setting page %p, rc (%s) w=%d, h=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*(*psInfo).proppage.offset(index as isize)).hwndPage,
                wine_dbgstr_rect(core::ptr::addr_of_mut!(rc)),
                rc.right - rc.left,
                rc.bottom - rc.top,
            );
        }

        SetWindowPos(
            (*(*psInfo).proppage.offset(index as isize)).hwndPage,
            HWND_TOP,
            rc.left as ::core::ffi::c_int,
            rc.top as ::core::ffi::c_int,
            rc.right as ::core::ffi::c_int - rc.left as ::core::ffi::c_int,
            rc.bottom as ::core::ffi::c_int - rc.top as ::core::ffi::c_int,
            0 as UINT,
        );

        result = SendMessageW(
            (*(*psInfo).proppage.offset(index as isize)).hwndPage,
            WM_NOTIFY as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(psn) as LPARAM,
        ) as ::core::ffi::c_int;
        if result == 0 {
            break;
        }
        if result == -1 {
            index += skipdir;
            if index < 0 {
                index = 0;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/comctl32/propsheet.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PROPSHEET_SetCurSel\0" as *const u8
                            as *const ::core::ffi::c_char,
                        2092 as ::core::ffi::c_int,
                        b"Tried to skip before first property sheet page!\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                break;
            }
            if index >= (*psInfo).nPages {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_3: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_3,
                        __dbch_3,
                        b"dll/win32/comctl32/propsheet.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PROPSHEET_SetCurSel\0" as *const u8
                            as *const ::core::ffi::c_char,
                        2096 as ::core::ffi::c_int,
                        b"Tried to skip after last property sheet page!\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                index = (*psInfo).nPages - 1;
                break;
            }
        } else if result != 0 {
            let mut old_index: ::core::ffi::c_int = index;
            index = PROPSHEET_FindPageByResId(psInfo, result as LRESULT) as ::core::ffi::c_int;
            if index >= (*psInfo).nPages {
                index = old_index;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_4: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_4,
                        __dbch_4,
                        b"dll/win32/comctl32/propsheet.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PROPSHEET_SetCurSel\0" as *const u8 as *const ::core::ffi::c_char,
                        2107 as ::core::ffi::c_int,
                        b"Tried to skip to nonexistent page by res id\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                break;
            }
            continue;
        }
    }

    /* Invalidate the header area */
    if (*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_OLD | PSH_WIZARD97_NEW) as DWORD != 0
        && (*psInfo).ppshheader.dwFlags & PSH_HEADER as DWORD != 0
    {
        let mut hwndLineHeader: HWND = GetDlgItem(hwndDlg, IDC_SUNKEN_LINEHEADER as c_int);
        let mut r: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        GetClientRect(hwndLineHeader, core::ptr::addr_of_mut!(r));
        MapWindowPoints(
            hwndLineHeader,
            hwndDlg,
            core::ptr::addr_of_mut!(r) as LPPOINT,
            2 as UINT,
        );
        SetRect(
            core::ptr::addr_of_mut!(r),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            r.right as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            r.top as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        );

        InvalidateRect(
            hwndDlg,
            core::ptr::addr_of_mut!(r) as LPCRECT,
            TRUE,
        );
    }

    /*
     * Display the new page.
     */
    PROPSHEET_ShowPage(hwndDlg, index, psInfo);

    if (*(*psInfo).proppage.offset(index as isize)).hasHelp != 0 {
        EnableWindow(hwndHelp, TRUE);
    } else {
        EnableWindow(hwndHelp, FALSE);
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_SetCurSelId(hwndDlg: HWND, id: i32) {
    let mut idx: i32 = 0;
    let psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    idx = PROPSHEET_FindPageByResId(psInfo, id as LRESULT) as i32;
    if idx < (*psInfo).nPages {
        if PROPSHEET_CanSetCurSel(hwndDlg) != FALSE {
            PROPSHEET_SetCurSel(hwndDlg, idx, 1, core::ptr::null_mut());
        }
    }
}

unsafe extern "C" fn PROPSHEET_SetTitleA(hwndDlg: HWND, dwStyle: DWORD, lpszText: LPCSTR) {
    if IS_INTRESOURCE!(lpszText) == false {
        let mut szTitle: [WCHAR; 256] = [0; 256];
        MultiByteToWideChar(CP_ACP, 0, lpszText, -1, szTitle.as_mut_ptr(), ARRAY_SIZE!(szTitle) as i32);
        PROPSHEET_SetTitleW(hwndDlg, dwStyle, szTitle.as_ptr() as LPCWSTR);
    } else {
        PROPSHEET_SetTitleW(hwndDlg, dwStyle, lpszText as LPCWSTR);
    }
}

unsafe extern "C" fn PROPSHEET_SetTitleW(mut hwndDlg: HWND, mut dwStyle: DWORD, mut lpszText: LPCWSTR) {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut szTitle: [WCHAR; 256] = [0; 256];
    if IS_INTRESOURCE!(lpszText) != false {
        if LoadStringW((*psInfo).ppshheader.hInstance, LOWORD!(lpszText as DWORD_PTR) as WORD as UINT, szTitle.as_mut_ptr(), ARRAY_SIZE!(szTitle) as core::ffi::c_int) == 0 {
            return;
        }
        lpszText = szTitle.as_mut_ptr() as LPCWSTR;
    }
    if dwStyle & PSH_PROPTITLE as DWORD != 0 {
        let mut dest: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
        let mut lentitle: core::ffi::c_int = lstrlenW(lpszText);
        let mut lenprop: core::ffi::c_int = lstrlenW((*psInfo).strPropertiesFor as LPCWSTR);

        dest = Alloc(
            ((lentitle + lenprop + 1 as core::ffi::c_int) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as *mut WCHAR;
        wsprintfW(dest as LPWSTR, (*psInfo).strPropertiesFor as LPCWSTR, lpszText);

        SetWindowTextW(hwndDlg, dest as LPCWSTR);
        Free(dest as LPVOID);
    } else {
        SetWindowTextW(hwndDlg, lpszText);
    }
}

unsafe extern "C" fn PROPSHEET_SetFinishTextA(hwndDlg: HWND, lpszText: LPCSTR) {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndButton: HWND = GetDlgItem(hwndDlg, (IDC_FINISH_BUTTON as i32));

    /* Set text, show and enable the Finish button */
    SetWindowTextA(hwndButton, lpszText);
    ShowWindow(hwndButton, (SW_SHOW) as i32);
    EnableWindow(hwndButton, TRUE);

    /* Make it default pushbutton */
    SendMessageW(hwndDlg, DM_SETDEFID as UINT, IDC_FINISH_BUTTON as WPARAM, 0 as LPARAM);

    /* Hide Back button */
    hwndButton = GetDlgItem(hwndDlg, (IDC_BACK_BUTTON as i32));
    ShowWindow(hwndButton, (SW_HIDE) as i32);

    if (*psInfo).hasFinish == 0 {
        /* Hide Next button */
        hwndButton = GetDlgItem(hwndDlg, (IDC_NEXT_BUTTON as i32));
        ShowWindow(hwndButton, (SW_HIDE) as i32);
    }
}

unsafe extern "C" fn PROPSHEET_SetFinishTextW(hwndDlg: HWND, lpszText: LPCWSTR) {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndButton: HWND = GetDlgItem(hwndDlg, (IDC_FINISH_BUTTON as i32));

    /* Set text, show and enable the Finish button */
    SetWindowTextW(hwndButton, lpszText);
    ShowWindow(hwndButton, (SW_SHOW) as i32);
    EnableWindow(hwndButton, TRUE);

    /* Make it default pushbutton */
    SendMessageW(
        hwndDlg,
        DM_SETDEFID as UINT,
        IDC_FINISH_BUTTON as WPARAM,
        0 as LPARAM,
    );

    /* Hide Back button */
    hwndButton = GetDlgItem(hwndDlg, (IDC_BACK_BUTTON as i32));
    ShowWindow(hwndButton, (SW_HIDE) as i32);

    if (*psInfo).hasFinish == 0 {
        /* Hide Next button */
        hwndButton = GetDlgItem(hwndDlg, (IDC_NEXT_BUTTON as i32));
        ShowWindow(hwndButton, (SW_HIDE) as i32);
    }
}

unsafe extern "C" fn PROPSHEET_QuerySiblings(
    mut hwndDlg: HWND,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hwndPage: HWND = core::ptr::null_mut();
    let mut msgResult: LRESULT = 0 as LRESULT;
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    while (i < (*psInfo).nPages) && (msgResult == 0 as LRESULT)
    {
        hwndPage = (*(*psInfo).proppage.offset(i as isize)).hwndPage;
        msgResult = SendMessageW(hwndPage, PSM_QUERYSIBLINGS as UINT, wParam, lParam);
        i += 1;
    }

    return msgResult;
}

unsafe extern "C" fn PROPSHEET_InsertPage(
    mut hwndDlg: HWND,
    mut hpageInsertAfter: HPROPSHEETPAGE,
    mut hpage: HPROPSHEETPAGE,
) -> BOOL {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
    let mut ppi: *mut PropPageInfo = core::ptr::null_mut::<PropPageInfo>();
    let mut prev_ppi: *mut PropPageInfo = (*psInfo).proppage;
    let mut hwndTabControl: HWND = GetDlgItem(hwndDlg, (IDC_TABCONTROL as i32));
    let mut ppsp: LPCPROPSHEETPAGEW = hpage as LPCPROPSHEETPAGEW;
    let mut item: TCITEMW = core::mem::zeroed::<TCITEMW>();
    let mut index: core::ffi::c_int = 0;

    if IS_INTRESOURCE!(hpageInsertAfter) {
        index = (LOWORD!(hpageInsertAfter)) as i32;
    } else {
        index = PROPSHEET_GetPageIndex(hpageInsertAfter, psInfo, -1);
        if index < 0 {
            return FALSE;
        }
        index += 1;
    }

    if index > (*psInfo).nPages {
        index = (*psInfo).nPages;
    }

    ppi = Alloc(
        (core::mem::size_of::<PropPageInfo>() as usize)
            .wrapping_mul(((*psInfo).nPages + 1) as usize) as DWORD,
    ) as *mut PropPageInfo;
    if ppi.is_null() {
        return FALSE;
    }

    if index > 0 {
        memcpy(
            ppi as *mut core::ffi::c_void,
            prev_ppi as *const core::ffi::c_void,
            (index as usize).wrapping_mul(core::mem::size_of::<PropPageInfo>() as usize),
        );
    }
    memset(
        ppi.offset(index as isize) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<PropPageInfo>() as usize,
    );
    if index < (*psInfo).nPages {
        memcpy(
            ppi.offset((index + 1) as isize) as *mut core::ffi::c_void,
            prev_ppi.offset(index as isize) as *const core::ffi::c_void,
            (((*psInfo).nPages - index) as usize)
                .wrapping_mul(core::mem::size_of::<PropPageInfo>() as usize),
        );
    }
    (*psInfo).proppage = ppi;

    if PROPSHEET_CollectPageInfo(ppsp, psInfo, index, FALSE) == 0 {
        (*psInfo).proppage = prev_ppi;
        Free(ppi as LPVOID);
        return FALSE;
    }

    let ref mut fresh19 = (*(*psInfo).proppage.offset(index as isize)).hpage;
    *fresh19 = hpage;

    if (*ppsp).dwFlags & PSP_PREMATURE as DWORD != 0 {
        if PROPSHEET_CreatePage(hwndDlg, index, psInfo, ppsp) == 0 {
            (*psInfo).proppage = prev_ppi;
            Free(ppi as LPVOID);
            return FALSE;
        }
    }

    Free(prev_ppi as LPVOID);
    (*psInfo).nPages += 1;
    if index <= (*psInfo).active_page {
        (*psInfo).active_page += 1;
    }

    item.mask = TCIF_TEXT as UINT;
    item.pszText = (*(*psInfo).proppage.offset(index as isize)).pszText as LPWSTR;
    item.cchTextMax = MAX_TABTEXT_LENGTH as core::ffi::c_int;

    if !(*psInfo).hImageList.is_null() {
        SendMessageW(
            hwndTabControl,
            TCM_SETIMAGELIST as UINT,
            0 as WPARAM,
            (*psInfo).hImageList as LPARAM,
        );
    }

    if (*(*psInfo).proppage.offset(index as isize)).hasIcon != 0 {
        item.mask |= TCIF_IMAGE as UINT;
        item.iImage = index;
    }

    SendMessageW(
        hwndTabControl,
        TCM_INSERTITEMW as UINT,
        index as WPARAM,
        &raw mut item as LPARAM,
    );

    if (*psInfo).nPages == 1 {
        PROPSHEET_SetCurSel(
            hwndDlg,
            0 as core::ffi::c_int,
            1 as core::ffi::c_int,
            core::ptr::null_mut(),
        );
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_AddPage(hwndDlg: HWND, hpage: HPROPSHEETPAGE) -> BOOL {
    let psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    return PROPSHEET_InsertPage(hwndDlg, UlongToPtr!((*psInfo).nPages as u32) as HPROPSHEETPAGE, hpage);
}

unsafe extern "C" fn PROPSHEET_RemovePage(
    mut hwndDlg: HWND,
    mut index: core::ffi::c_int,
    mut hpage: HPROPSHEETPAGE,
) -> BOOL {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndTabControl: HWND = GetDlgItem(hwndDlg, (IDC_TABCONTROL as i32));
    let mut oldPages: *mut PropPageInfo;

    if psInfo.is_null() {
        return FALSE;
    }

    index = PROPSHEET_GetPageIndex(hpage, psInfo, index);

    /* Make sure that index is within range */
    if index < 0 || index >= (*psInfo).nPages
    {
        return FALSE;
    }

    /*
     * Check if we're removing the active page.
     */
    if index == (*psInfo).active_page
    {
        if (*psInfo).nPages > 1
        {
            if index > 0
            {
                /* activate previous page  */
                PROPSHEET_SetCurSel(hwndDlg, index - 1, -1, core::ptr::null_mut());
            }
            else
            {
                /* activate the next page */
                PROPSHEET_SetCurSel(hwndDlg, index + 1, 1, core::ptr::null_mut());
                (*psInfo).active_page = index;
            }
        }
        else
        {
            (*psInfo).active_page = -1;
            if (*psInfo).isModeless == 0
            {
                (*psInfo).ended = TRUE;
                return TRUE;
            }
        }
    }
    else if index < (*psInfo).active_page
    {
        (*psInfo).active_page -= 1;
    }

    /* Unsubclass the page dialog window */
    if ((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_NEW | PSH_WIZARD97_OLD)) != 0
        && ((*psInfo).ppshheader.dwFlags & PSH_WATERMARK) != 0
        && ((*((*(*psInfo).proppage.offset(index as isize)).hpage as *mut PROPSHEETPAGEW)).dwFlags & PSP_HIDEHEADER) != 0
    {
        RemoveWindowSubclass(
            (*(*psInfo).proppage.offset(index as isize)).hwndPage,
            Some(
                PROPSHEET_WizardSubclassProc,
            ),
            1,
        );
    }

    /* Destroy page dialog window */
    DestroyWindow((*(*psInfo).proppage.offset(index as isize)).hwndPage);

    /* Free page resources */
    if !(*(*psInfo).proppage.offset(index as isize)).hpage.is_null()
    {
        let mut psp: *mut PROPSHEETPAGEW =
            (*(*psInfo).proppage.offset(index as isize)).hpage as *mut PROPSHEETPAGEW;

        if (*psp).dwFlags & PSP_USETITLE != 0
        {
            Free((*(*psInfo).proppage.offset(index as isize)).pszText as LPVOID);
        }

        DestroyPropertySheetPage((*(*psInfo).proppage.offset(index as isize)).hpage);
    }

    /* Remove the tab */
    SendMessageW(hwndTabControl, TCM_DELETEITEM as UINT, index as WPARAM, 0 as LPARAM);

    oldPages = (*psInfo).proppage;
    (*psInfo).nPages -= 1;
    (*psInfo).proppage = Alloc(
        (core::mem::size_of::<PropPageInfo>() as usize).wrapping_mul((*psInfo).nPages as usize) as DWORD,
    ) as *mut PropPageInfo;

    if index > 0
    {
        memcpy(
            (*psInfo).proppage.offset(0 as isize) as *mut core::ffi::c_void,
            oldPages.offset(0 as isize) as *const core::ffi::c_void,
            (index as usize).wrapping_mul(core::mem::size_of::<PropPageInfo>()),
        );
    }

    if index < (*psInfo).nPages
    {
        memcpy(
            (*psInfo).proppage.offset(index as isize) as *mut core::ffi::c_void,
            oldPages.offset((index + 1) as isize) as *const core::ffi::c_void,
            (((*psInfo).nPages - index) as usize).wrapping_mul(core::mem::size_of::<PropPageInfo>()),
        );
    }

    Free(oldPages as LPVOID);

    return FALSE;
}

unsafe extern "C" fn PROPSHEET_SetWizButtons(mut hwndDlg: HWND, mut dwFlags: DWORD) {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut hwndBack: HWND = GetDlgItem(hwndDlg, IDC_BACK_BUTTON as INT);
    let mut hwndNext: HWND = GetDlgItem(hwndDlg, IDC_NEXT_BUTTON as INT);
    let mut hwndFinish: HWND = GetDlgItem(hwndDlg, IDC_FINISH_BUTTON as INT);
    let mut enable_finish: BOOL =
        (((dwFlags & (PSWIZB_FINISH as DWORD)) != 0 || (*psInfo).hasFinish != 0)
            && (dwFlags & (PSWIZB_DISABLEDFINISH as DWORD)) == 0) as BOOL;
    let mut hwndCancel: HWND = GetDlgItem(hwndDlg, IDCANCEL as INT);
    let mut iDefItem: INT = 0 as INT;
    let mut hwndFocus: HWND;

    EnableWindow(hwndBack, (dwFlags & (PSWIZB_BACK as DWORD)) as BOOL);
    EnableWindow(hwndNext, (dwFlags & (PSWIZB_NEXT as DWORD)) as BOOL);
    EnableWindow(hwndFinish, enable_finish);

    if (*psInfo).hasFinish == 0 {
        if (dwFlags & (PSWIZB_FINISH as DWORD)) != 0
            || (dwFlags & (PSWIZB_DISABLEDFINISH as DWORD)) != 0
        {
            /* Hide the Next button */
            ShowWindow(hwndNext, SW_HIDE as INT);

            /* Show the Finish button */
            ShowWindow(hwndFinish, SW_SHOW as INT);
        } else {
            /* Hide the Finish button */
            ShowWindow(hwndFinish, SW_HIDE as INT);
            /* Show the Next button */
            ShowWindow(hwndNext, SW_SHOW as INT);
        }
    }

    /* set the default pushbutton to an enabled button */
    if ((dwFlags & (PSWIZB_FINISH as DWORD)) != 0 || (*psInfo).hasFinish != 0)
        && (dwFlags & (PSWIZB_DISABLEDFINISH as DWORD)) == 0
    {
        iDefItem = IDC_FINISH_BUTTON as INT;
    } else if (dwFlags & (PSWIZB_NEXT as DWORD)) != 0 {
        iDefItem = IDC_NEXT_BUTTON as INT;
    } else if (dwFlags & (PSWIZB_BACK as DWORD)) != 0 {
        iDefItem = IDC_BACK_BUTTON as INT;
    } else {
        iDefItem = IDCANCEL as INT;
    }
    SendMessageW(
        hwndDlg,
        DM_SETDEFID as UINT,
        iDefItem as WPARAM,
        0 as LPARAM,
    );

    /* Set focus if no control has it */
    hwndFocus = GetFocus();
    if hwndFocus.is_null() || hwndFocus == hwndCancel {
        SetFocus(GetDlgItem(hwndDlg, iDefItem));
    }
}

unsafe extern "C" fn PROPSHEET_SetHeaderTitleW(hwndDlg: HWND, page_index: UINT, title: *const WCHAR) {
    let psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut page: *mut PROPSHEETPAGEW = core::ptr::null_mut();

    if page_index >= (*psInfo).nPages as UINT {
        return;
    }
    page = (*(*psInfo).proppage.offset(page_index as isize)).hpage as *mut PROPSHEETPAGEW;
    if !(((*page).pszHeaderTitle as ULONG_PTR >> 16) == 0) {
        Free((*page).pszHeaderTitle as LPVOID);
    }
    (*page).pszHeaderTitle = heap_strdupW(title) as LPCWSTR;
    (*page).dwFlags |= PSP_USEHEADERTITLE as DWORD;
}

unsafe extern "C" fn PROPSHEET_SetHeaderTitleA(hwndDlg: HWND, page_index: UINT, title: *const i8) {
    let mut titleW: *mut WCHAR = core::ptr::null_mut();
    titleW = heap_strdupAtoW(title);
    PROPSHEET_SetHeaderTitleW(hwndDlg, page_index, titleW);
    Free(titleW as LPVOID);
}

unsafe extern "C" fn PROPSHEET_SetHeaderSubTitleW(hwndDlg: HWND, page_index: UINT, subtitle: *const WCHAR) {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut page: *mut PROPSHEETPAGEW = core::ptr::null_mut::<PROPSHEETPAGEW>();

    if page_index >= (*psInfo).nPages as UINT {
        return;
    }

    page = (*(*psInfo).proppage.offset(page_index as isize)).hpage as *mut PROPSHEETPAGEW;

    if !IS_INTRESOURCE!((*page).pszHeaderSubTitle) {
        Free((*page).pszHeaderSubTitle as *mut core::ffi::c_void);
    }

    (*page).pszHeaderSubTitle = heap_strdupW(subtitle) as LPCWSTR;
    (*page).dwFlags |= PSP_USEHEADERSUBTITLE as DWORD;
}

unsafe extern "C" fn PROPSHEET_SetHeaderSubTitleA(hwndDlg: HWND, page_index: UINT, subtitle: *const core::ffi::c_char) {
    let mut subtitleW: *mut WCHAR = core::ptr::null_mut();

    subtitleW = heap_strdupAtoW(subtitle);
    PROPSHEET_SetHeaderSubTitleW(hwndDlg, page_index, subtitleW);
    Free(subtitleW as LPVOID);
}

unsafe extern "C" fn PROPSHEET_HwndToIndex(hwndDlg: HWND, hPageDlg: HWND) -> LRESULT {
    let mut index: ::core::ffi::c_int;
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    index = 0;
    while index < (*psInfo).nPages {
        if (*(*psInfo).proppage.offset(index as isize)).hwndPage == hPageDlg {
            return index as LRESULT;
        }
        index += 1;
    }
    return -(1) as LRESULT;
}

unsafe extern "C" fn PROPSHEET_IndexToHwnd(hwndDlg: HWND, iPageIndex: ::core::ffi::c_int) -> LRESULT {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    if psInfo.is_null() {
        return 0 as LRESULT;
    }
    if iPageIndex < 0 || iPageIndex >= (*psInfo).nPages {
        return 0 as LRESULT;
    }
    return (*(*psInfo).proppage.offset(iPageIndex as isize)).hwndPage as LRESULT;
}

unsafe extern "C" fn PROPSHEET_PageToIndex(hwndDlg: HWND, hPage: HPROPSHEETPAGE) -> LRESULT {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    return PROPSHEET_GetPageIndex(hPage, psInfo, -1) as LRESULT;
}

unsafe extern "C" fn PROPSHEET_IndexToPage(hwndDlg: HWND, iPageIndex: core::ffi::c_int) -> LRESULT {
    let psInfo: *mut PropSheetInfo = GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    if iPageIndex < 0 || iPageIndex >= (*psInfo).nPages {
        return 0;
    }
    return (*(*psInfo).proppage.offset(iPageIndex as isize)).hpage as LRESULT;
}

unsafe extern "C" fn PROPSHEET_IdToIndex(
    mut hwndDlg: HWND,
    mut iPageId: ::core::ffi::c_int,
) -> LRESULT {
    let mut index: ::core::ffi::c_int;
    let mut psp: LPCPROPSHEETPAGEW = ::core::ptr::null::<PROPSHEETPAGEW>();
    let mut psInfo: *mut PropSheetInfo = GetPropW(
        hwndDlg,
        core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR,
    ) as *mut PropSheetInfo;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PROPSHEET_IdToIndex\0" as *const u8 as *const ::core::ffi::c_char,
            2708 as ::core::ffi::c_int,
            b"(%p, %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwndDlg,
            iPageId,
        );
    }
    index = 0;
    while index < (*psInfo).nPages {
        psp = (*(*psInfo).proppage.offset(index as isize)).hpage as LPCPROPSHEETPAGEW;
        if (*psp).u.pszTemplate == MAKEINTRESOURCEW!(iPageId) as LPCWSTR {
            return index as LRESULT;
        }
        index += 1;
    }
    return -(1) as LRESULT;
}

unsafe extern "C" fn PROPSHEET_IndexToId(mut hwndDlg: HWND, mut iPageIndex: i32) -> LRESULT {
    let mut psInfo: *mut PropSheetInfo = GetPropW(
        hwndDlg,
        core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR,
    ) as *mut PropSheetInfo;
    let mut psp: LPCPROPSHEETPAGEW = core::ptr::null::<PROPSHEETPAGEW>();
    if iPageIndex < 0 || iPageIndex >= (*psInfo).nPages {
        return 0;
    }
    psp = (*(*psInfo).proppage.offset(iPageIndex as isize)).hpage as LPCPROPSHEETPAGEW;
    if ((*psp).dwFlags & (PSP_DLGINDIRECT as DWORD)) != 0
        || !IS_INTRESOURCE!((*psp).u.pszTemplate)
    {
        return 0;
    }
    return (*psp).u.pszTemplate as LRESULT;
}

unsafe extern "C" fn PROPSHEET_GetResult(hwndDlg: HWND) -> LRESULT {
    let mut psInfo: *mut PropSheetInfo =
        GetPropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    return (*psInfo).result as LRESULT;
}

unsafe extern "C" fn PROPSHEET_RecalcPageSizes(hwndDlg: HWND) -> BOOL {
    // FIXME("(%p): stub\n", hwndDlg);
    FALSE
}

unsafe extern "C" fn PROPSHEET_GetPageIndex(page: HPROPSHEETPAGE, psInfo: *const PropSheetInfo, original_index: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut index: ::core::ffi::c_int = 0;

    index = 0;
    while index < (*psInfo).nPages {
        if (*(*psInfo).proppage.offset(index as isize)).hpage == page {
            return index;
        }
        index += 1;
    }

    return original_index;
}

unsafe extern "C" fn PROPSHEET_CleanUp(hwndDlg: HWND) {
    let mut i: ::core::ffi::c_int = 0;
    let mut psInfo: *mut PropSheetInfo =
        RemovePropW(hwndDlg, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    if psInfo.is_null() {
        return;
    }
    if !IS_INTRESOURCE!((*psInfo).ppshheader.pszCaption) {
        Free((*psInfo).ppshheader.pszCaption as LPVOID);
    }

    i = 0;
    while i < (*psInfo).nPages {
        let mut psp: *mut PROPSHEETPAGEA =
            (*(*psInfo).proppage.offset(i as isize)).hpage as *mut PROPSHEETPAGEA;

        /* Unsubclass the page dialog window */
        if ((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_NEW | PSH_WIZARD97_OLD) as DWORD) != 0
            && ((*psInfo).ppshheader.dwFlags & PSH_WATERMARK as DWORD) != 0
            && ((*psp).dwFlags & PSP_HIDEHEADER as DWORD) != 0
        {
            RemoveWindowSubclass(
                (*(*psInfo).proppage.offset(i as isize)).hwndPage,
                Some(
                    PROPSHEET_WizardSubclassProc,
                ),
                1 as UINT_PTR,
            );
        }

        if !(*(*psInfo).proppage.offset(i as isize)).hwndPage.is_null() {
            DestroyWindow((*(*psInfo).proppage.offset(i as isize)).hwndPage);
        }

        if !psp.is_null() {
            if ((*psp).dwFlags & PSP_USETITLE as DWORD) != 0 {
                Free((*(*psInfo).proppage.offset(i as isize)).pszText as LPVOID);
            }
            DestroyPropertySheetPage((*(*psInfo).proppage.offset(i as isize)).hpage);
        }

        i += 1;
    }

    DeleteObject((*psInfo).hFont as HGDIOBJ);
    DeleteObject((*psInfo).hFontBold as HGDIOBJ);
    /* If we created the bitmaps, destroy them */
    if ((*psInfo).ppshheader.dwFlags & PSH_WATERMARK as DWORD) != 0
        && ((*psInfo).ppshheader.dwFlags & PSH_USEHBMWATERMARK as DWORD) == 0
    {
        DeleteObject((*psInfo).ppshheader.u4.hbmWatermark as HGDIOBJ);
    }
    if ((*psInfo).ppshheader.dwFlags & PSH_HEADER as DWORD) != 0
        && ((*psInfo).ppshheader.dwFlags & PSH_USEHBMHEADER as DWORD) == 0
    {
        DeleteObject((*psInfo).ppshheader.u5.hbmHeader as HGDIOBJ);
    }

    Free((*psInfo).proppage as LPVOID);
    Free((*psInfo).strPropertiesFor as LPVOID);
    ImageList_Destroy((*psInfo).hImageList);

    GlobalFree(psInfo as HGLOBAL);
}

unsafe extern "C" fn do_loop(psInfo: *const PropSheetInfo) -> INT {
    let mut msg: MSG = core::mem::zeroed();
    let mut ret: INT = 0;
    let mut hwnd: HWND = (*psInfo).hwnd;
    let mut parent: HWND = (*psInfo).ppshheader.hwndParent;

    while IsWindow(hwnd) != 0
        && (*psInfo).ended == 0
        && {
            ret = GetMessageW(
                core::ptr::addr_of_mut!(msg),
                core::ptr::null_mut(),
                0,
                0,
            ) as INT;
            ret != 0
        }
    {
        if ret == -1 {
            break;
        }

        if PROPSHEET_IsDialogMessage(hwnd, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }

    if ret == 0 && msg.message != 0 {
        PostQuitMessage(msg.wParam as INT);
    }

    if ret != -1 {
        ret = (*psInfo).result;
    }

    if !parent.is_null() {
        EnableWindow(parent, TRUE);
    }

    DestroyWindow(hwnd);
    return ret;
}

unsafe extern "C" fn PROPSHEET_PropertySheet(psInfo: *mut PropSheetInfo, unicode: BOOL) -> INT_PTR {
    let mut bRet: INT_PTR = 0;
    let mut parent: HWND = core::ptr::null_mut();
    if (*psInfo).active_page >= (*psInfo).nPages {
        (*psInfo).active_page = 0;
    }
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/propsheet.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"PROPSHEET_PropertySheet\0" as *const u8 as *const core::ffi::c_char,
            2880 as core::ffi::c_int,
            b"startpage: %d of %d pages\n\0" as *const u8 as *const core::ffi::c_char,
            (*psInfo).active_page,
            (*psInfo).nPages,
        );
    }
    (*psInfo).unicode = unicode;
    (*psInfo).ended = FALSE;
    if (*psInfo).isModeless == 0 {
        parent = (*psInfo).ppshheader.hwndParent;
        if !parent.is_null() {
            EnableWindow(parent, FALSE);
        }
    }
    bRet = PROPSHEET_CreateDialog(psInfo);
    if (*psInfo).isModeless == 0 {
        bRet = do_loop(psInfo) as INT_PTR;
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn PropertySheetA(lppsh: LPCPROPSHEETHEADERA) -> INT_PTR {
    let psInfo: *mut PropSheetInfo = GlobalAlloc(GPTR as UINT, core::mem::size_of::<PropSheetInfo>() as SIZE_T) as *mut PropSheetInfo;
    let mut i: UINT;
    let mut n: UINT;
    let mut pByte: *const BYTE;

    PROPSHEET_CollectSheetInfoA(lppsh, psInfo);

    (*psInfo).proppage = Alloc((core::mem::size_of::<PropPageInfo>() as usize).wrapping_mul((*lppsh).nPages as usize) as DWORD) as *mut PropPageInfo;
    pByte = (*psInfo).ppshheader.u3.ppsp as *const BYTE;

    i = 0 as UINT;
    n = i;
    while i < (*lppsh).nPages {
        if (*psInfo).usePropPage == 0 {
            let fresh15 = &mut (*(*psInfo).proppage.offset(n as isize)).hpage;
            *fresh15 = *(*psInfo).ppshheader.u3.phpage.offset(i as isize);
        } else {
            let fresh16 = &mut (*(*psInfo).proppage.offset(n as isize)).hpage;
            *fresh16 = CreatePropertySheetPageA(pByte as LPCPROPSHEETPAGEA);
            pByte = pByte.offset((*(pByte as LPCPROPSHEETPAGEA)).dwSize as isize);
        }
        if PROPSHEET_CollectPageInfo(
            (*(*psInfo).proppage.offset(n as isize)).hpage as LPCPROPSHEETPAGEW,
            psInfo,
            n as i32,
            TRUE as BOOL,
        ) == 0
        {
            if (*psInfo).usePropPage != 0 {
                DestroyPropertySheetPage((*(*psInfo).proppage.offset(n as isize)).hpage);
            }
            n = n.wrapping_sub(1);
            (*psInfo).nPages -= 1;
        }
        i = i.wrapping_add(1);
        n = n.wrapping_add(1);
    }

    return PROPSHEET_PropertySheet(psInfo, FALSE as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn PropertySheetW(lppsh: LPCPROPSHEETHEADERW) -> INT_PTR {
    let mut psInfo: *mut PropSheetInfo = GlobalAlloc(GPTR as UINT, core::mem::size_of::<PropSheetInfo>() as SIZE_T) as *mut PropSheetInfo;
    let mut i: UINT = 0;
    let mut n: UINT = 0;
    let mut pByte: *const BYTE = core::ptr::null();

    PROPSHEET_CollectSheetInfoW(lppsh, psInfo);

    (*psInfo).proppage = Alloc((core::mem::size_of::<PropPageInfo>()).wrapping_mul((*lppsh).nPages as usize) as DWORD) as *mut PropPageInfo;
    pByte = (*psInfo).ppshheader.u3.ppsp as *const BYTE;

    i = 0 as UINT;
    n = i;
    while i < (*lppsh).nPages {
        if (*psInfo).usePropPage == 0 {
            (*(*psInfo).proppage.offset(n as isize)).hpage = *(*psInfo).ppshheader.u3.phpage.offset(i as isize);
        } else {
            (*(*psInfo).proppage.offset(n as isize)).hpage = CreatePropertySheetPageW(pByte as LPCPROPSHEETPAGEW);
            pByte = pByte.offset((*(pByte as LPCPROPSHEETPAGEW)).dwSize as isize);
        }

        if PROPSHEET_CollectPageInfo(
            (*(*psInfo).proppage.offset(n as isize)).hpage as LPCPROPSHEETPAGEW,
            psInfo,
            n as i32,
            TRUE as i32,
        ) == 0
        {
            if (*psInfo).usePropPage != 0 {
                DestroyPropertySheetPage((*(*psInfo).proppage.offset(n as isize)).hpage);
            }
            n = n.wrapping_sub(1);
            (*psInfo).nPages -= 1;
        }
        i = i.wrapping_add(1);
        n = n.wrapping_add(1);
    }

    return PROPSHEET_PropertySheet(psInfo, TRUE as i32);
}

unsafe extern "C" fn load_string(mut instance: HINSTANCE, mut str: LPCWSTR) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut::<WCHAR>();
    if IS_INTRESOURCE!(str) {
        let mut hrsrc: HRSRC;
        let mut hmem: HGLOBAL;
        let mut ptr: *mut WCHAR;
        let mut i: WORD;
        let id: WORD = LOWORD!(str);
        let mut len: UINT;

        hrsrc = FindResourceW(
            instance,
            MAKEINTRESOURCEW!((((id as core::ffi::c_int) >> 4) + 1) as WORD),
            (RT_STRING as usize) as LPWSTR,
        );
        if hrsrc.is_null() {
            return core::ptr::null_mut::<WCHAR>();
        }
        hmem = LoadResource(instance, hrsrc);
        if hmem.is_null() {
            return core::ptr::null_mut::<WCHAR>();
        }
        ptr = LockResource(hmem) as *mut WCHAR;
        if ptr.is_null() {
            return core::ptr::null_mut::<WCHAR>();
        }
        i = ((id as core::ffi::c_int) & (0x0f as core::ffi::c_int)) as WORD;
        while (i as core::ffi::c_int) > (0 as core::ffi::c_int) {
            ptr = ptr.offset(((*ptr) as core::ffi::c_int + (1 as core::ffi::c_int)) as isize);
            i = i.wrapping_sub(1);
        }
        len = *ptr as UINT;
        if len == 0 {
            return core::ptr::null_mut::<WCHAR>();
        }
        ret = Alloc(
            ((len.wrapping_add(1 as UINT) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as DWORD,
        ) as LPWSTR;
        if !ret.is_null() {
            memcpy(
                ret as *mut core::ffi::c_void,
                ptr.offset(1 as isize) as *const core::ffi::c_void,
                (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
            );
            *ret.offset(len as isize) = 0 as WCHAR;
        }
    } else {
        let len: core::ffi::c_int = (((lstrlenW(str) + (1 as core::ffi::c_int)) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)) as core::ffi::c_int;
        ret = Alloc(len as DWORD) as LPWSTR;
        if !ret.is_null() {
            memcpy(
                ret as *mut core::ffi::c_void,
                str as *const core::ffi::c_void,
                len as usize,
            );
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreatePropertySheetPageA(
    lpPropSheetPage: LPCPROPSHEETPAGEA,
) -> HPROPSHEETPAGE {
    let mut ppsp: *mut PROPSHEETPAGEW;

    if (*lpPropSheetPage).dwSize < (PROPSHEETPAGEA_V1_SIZE) as _ {
        return core::ptr::null_mut();
    }

    /* original data is used for callback notifications */
    if ((*lpPropSheetPage).dwFlags & PSP_USECALLBACK != 0) && (*lpPropSheetPage).pfnCallback.is_some() {
        ppsp = Alloc(
            (2 as usize).wrapping_mul(core::mem::size_of::<PROPSHEETPAGEW>()) as DWORD,
        ) as *mut PROPSHEETPAGEW;
        memcpy(
            ppsp as *mut core::ffi::c_void,
            lpPropSheetPage as *const core::ffi::c_void,
            min(
                (*lpPropSheetPage).dwSize as usize,
                core::mem::size_of::<PROPSHEETPAGEA>(),
            ) as size_t,
        );
        memcpy(
            ppsp.offset(1 as isize) as *mut core::ffi::c_void,
            lpPropSheetPage as *const core::ffi::c_void,
            min(
                (*lpPropSheetPage).dwSize as usize,
                core::mem::size_of::<PROPSHEETPAGEA>(),
            ) as size_t,
        );
    } else {
        ppsp = Alloc(core::mem::size_of::<PROPSHEETPAGEW>() as DWORD)
            as *mut PROPSHEETPAGEW;
        memcpy(
            ppsp as *mut core::ffi::c_void,
            lpPropSheetPage as *const core::ffi::c_void,
            min(
                (*lpPropSheetPage).dwSize as usize,
                core::mem::size_of::<PROPSHEETPAGEA>(),
            ) as size_t,
        );
    }

    (*ppsp).dwFlags &= !PSP_INTERNAL_UNICODE;

    if !((*ppsp).dwFlags & PSP_DLGINDIRECT != 0) {
        if !IS_INTRESOURCE!((*ppsp).u.pszTemplate) {
            let len: core::ffi::c_int = (strlen((*lpPropSheetPage).u.pszTemplate)
                as core::ffi::c_int)
                + 1;
            let template: *mut core::ffi::c_char =
                Alloc(len as DWORD) as *mut core::ffi::c_char;

            (*ppsp).u.pszTemplate = strcpy(
                template,
                (*lpPropSheetPage).u.pszTemplate as *const core::ffi::c_char,
            ) as LPCWSTR;
        }
    }

    if (*ppsp).dwFlags & PSP_USEICONID != 0 {
        if !IS_INTRESOURCE!((*ppsp).u2.pszIcon) {
            (*ppsp).u2.pszIcon = heap_strdupAtoW((*lpPropSheetPage).u2.pszIcon) as LPCWSTR;
        }
    }

    if (*ppsp).dwFlags & PSP_USETITLE != 0 {
        if IS_INTRESOURCE!((*ppsp).pszTitle) {
            (*ppsp).pszTitle = load_string((*ppsp).hInstance, (*ppsp).pszTitle) as LPCWSTR;
        } else {
            (*ppsp).pszTitle = heap_strdupAtoW((*lpPropSheetPage).pszTitle) as LPCWSTR;
        }
    } else {
        (*ppsp).pszTitle = core::ptr::null();
    }

    if (*ppsp).dwFlags & PSP_HIDEHEADER != 0 {
        (*ppsp).dwFlags &= !(PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE);
    }

    if (*ppsp).dwFlags & PSP_USEHEADERTITLE != 0 {
        if IS_INTRESOURCE!((*ppsp).pszHeaderTitle) {
            (*ppsp).pszHeaderTitle =
                load_string((*ppsp).hInstance, (*ppsp).pszHeaderTitle) as LPCWSTR;
        } else {
            (*ppsp).pszHeaderTitle =
                heap_strdupAtoW((*lpPropSheetPage).pszHeaderTitle) as LPCWSTR;
        }
    } else {
        (*ppsp).pszHeaderTitle = core::ptr::null();
    }

    if (*ppsp).dwFlags & PSP_USEHEADERSUBTITLE != 0 {
        if IS_INTRESOURCE!((*ppsp).pszHeaderSubTitle) {
            (*ppsp).pszHeaderSubTitle =
                load_string((*ppsp).hInstance, (*ppsp).pszHeaderSubTitle) as LPCWSTR;
        } else {
            (*ppsp).pszHeaderSubTitle =
                heap_strdupAtoW((*lpPropSheetPage).pszHeaderSubTitle) as LPCWSTR;
        }
    } else {
        (*ppsp).pszHeaderSubTitle = core::ptr::null();
    }

    if ((*ppsp).dwFlags & PSP_USECALLBACK != 0)
        && (*ppsp).dwSize > (PROPSHEETPAGEA_V1_SIZE) as _
        && (*ppsp).pfnCallback.is_some()
    {
        (*ppsp).pfnCallback.unwrap_unchecked()(
            core::ptr::null_mut(),
            PSPCB_ADDREF as UINT,
            ppsp.offset(1 as isize),
        );
    }

    return ppsp as HPROPSHEETPAGE;
}

#[no_mangle]
pub unsafe extern "system" fn CreatePropertySheetPageW(lpPropSheetPage: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE {
    let mut ppsp: *mut PROPSHEETPAGEW = core::ptr::null_mut();

    if (*lpPropSheetPage).dwSize < (PROPSHEETPAGEW_V1_SIZE) as _ {
        return core::ptr::null_mut();
    }

    /* original data is used for callback notifications */
    if ((*lpPropSheetPage).dwFlags & PSP_USECALLBACK) != 0 && (*lpPropSheetPage).pfnCallback.is_some() {
        ppsp = Alloc((2 as usize * core::mem::size_of::<PROPSHEETPAGEW>()) as DWORD) as *mut PROPSHEETPAGEW;
        memcpy(
            ppsp as *mut core::ffi::c_void,
            lpPropSheetPage as *const core::ffi::c_void,
            min((*lpPropSheetPage).dwSize as size_t, core::mem::size_of::<PROPSHEETPAGEW>() as size_t),
        );
        memcpy(
            ppsp.add(1 as usize) as *mut core::ffi::c_void,
            lpPropSheetPage as *const core::ffi::c_void,
            min((*lpPropSheetPage).dwSize as size_t, core::mem::size_of::<PROPSHEETPAGEW>() as size_t),
        );
    } else {
        ppsp = Alloc(core::mem::size_of::<PROPSHEETPAGEW>() as DWORD) as *mut PROPSHEETPAGEW;
        memcpy(
            ppsp as *mut core::ffi::c_void,
            lpPropSheetPage as *const core::ffi::c_void,
            min((*lpPropSheetPage).dwSize as size_t, core::mem::size_of::<PROPSHEETPAGEW>() as size_t),
        );
    }

    (*ppsp).dwFlags |= PSP_INTERNAL_UNICODE;

    if ((*ppsp).dwFlags & PSP_DLGINDIRECT) == 0 {
        if !IS_INTRESOURCE!((*ppsp).u.pszTemplate) {
            (*ppsp).u.pszTemplate = heap_strdupW((*lpPropSheetPage).u.pszTemplate) as LPCWSTR;
        }
    }

    if ((*ppsp).dwFlags & PSP_USEICONID) != 0 {
        if !IS_INTRESOURCE!((*ppsp).u2.pszIcon) {
            (*ppsp).u2.pszIcon = heap_strdupW((*lpPropSheetPage).u2.pszIcon) as LPCWSTR;
        }
    }

    if ((*ppsp).dwFlags & PSP_USETITLE) != 0 {
        (*ppsp).pszTitle = load_string((*ppsp).hInstance, (*ppsp).pszTitle as LPCWSTR) as LPCWSTR;
    } else {
        (*ppsp).pszTitle = core::ptr::null();
    }

    if ((*ppsp).dwFlags & PSP_HIDEHEADER) != 0 {
        (*ppsp).dwFlags &= !(PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE);
    }

    if ((*ppsp).dwFlags & PSP_USEHEADERTITLE) != 0 {
        (*ppsp).pszHeaderTitle = load_string((*ppsp).hInstance, (*ppsp).pszHeaderTitle as LPCWSTR) as LPCWSTR;
    } else {
        (*ppsp).pszHeaderTitle = core::ptr::null();
    }

    if ((*ppsp).dwFlags & PSP_USEHEADERSUBTITLE) != 0 {
        (*ppsp).pszHeaderSubTitle = load_string((*ppsp).hInstance, (*ppsp).pszHeaderSubTitle as LPCWSTR) as LPCWSTR;
    } else {
        (*ppsp).pszHeaderSubTitle = core::ptr::null();
    }

    if ((*ppsp).dwFlags & PSP_USECALLBACK) != 0
        && (*ppsp).dwSize > (PROPSHEETPAGEW_V1_SIZE) as _
        && (*ppsp).pfnCallback.is_some()
    {
        (*ppsp).pfnCallback.unwrap_unchecked()(
            core::ptr::null_mut(),
            PSPCB_ADDREF,
            ppsp.add(1 as usize),
        );
    }

    return ppsp as HPROPSHEETPAGE;
}

#[no_mangle]
pub unsafe extern "system" fn DestroyPropertySheetPage(hPropPage: HPROPSHEETPAGE) -> BOOL {
    let mut psp: *mut PROPSHEETPAGEW = hPropPage as *mut PROPSHEETPAGEW;

    if psp.is_null() {
        return FALSE;
    }

    if ((*psp).dwFlags & (PSP_USECALLBACK as DWORD) != 0) && (*psp).pfnCallback.is_some() {
        (*psp).pfnCallback.unwrap_unchecked()(
            PSPCB_RELEASE as HWND,
            0 as UINT,
            psp.offset(1 as isize),
        );
    }

    if !((*psp).dwFlags & (PSP_DLGINDIRECT as DWORD) != 0)
        && !IS_INTRESOURCE!((*psp).u.pszTemplate)
    {
        Free((*psp).u.pszTemplate as LPVOID);
    }

    if ((*psp).dwFlags & (PSP_USEICONID as DWORD) != 0)
        && !IS_INTRESOURCE!((*psp).u2.pszIcon)
    {
        Free((*psp).u2.pszIcon as LPVOID);
    }

    if ((*psp).dwFlags & (PSP_USETITLE as DWORD) != 0)
        && !IS_INTRESOURCE!((*psp).pszTitle)
    {
        Free((*psp).pszTitle as LPVOID);
    }

    if ((*psp).dwFlags & (PSP_USEHEADERTITLE as DWORD) != 0)
        && !IS_INTRESOURCE!((*psp).pszHeaderTitle)
    {
        Free((*psp).pszHeaderTitle as LPVOID);
    }

    if ((*psp).dwFlags & (PSP_USEHEADERSUBTITLE as DWORD) != 0)
        && !IS_INTRESOURCE!((*psp).pszHeaderSubTitle)
    {
        Free((*psp).pszHeaderSubTitle as LPVOID);
    }

    Free((hPropPage).cast());

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_IsDialogMessage(hwnd: HWND, lpMsg: LPMSG) -> BOOL {
    let psInfo: *mut PropSheetInfo = GetPropW(hwnd, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;

    if psInfo.is_null() || (hwnd != (*lpMsg).hwnd && IsChild(hwnd, (*lpMsg).hwnd) == 0) {
        return FALSE;
    }

    if (*lpMsg).message == WM_KEYDOWN as UINT
        && ((GetKeyState(VK_CONTROL as ::core::ffi::c_int) as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int) != 0
    {
        let mut new_page: ::core::ffi::c_int = 0;
        let dlgCode: INT = SendMessageW((*lpMsg).hwnd, WM_GETDLGCODE as UINT, 0 as WPARAM, lpMsg as LPARAM) as INT;

        if (dlgCode & DLGC_WANTMESSAGE as INT) == 0 {
            match (*lpMsg).wParam as UINT {
                VK_TAB => {
                    if ((GetKeyState(VK_SHIFT as ::core::ffi::c_int) as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int) != 0 {
                        new_page = -1;
                    } else {
                        new_page = 1;
                    }
                }

                VK_NEXT => {
                    new_page = 1;
                }
                VK_PRIOR => {
                    new_page = -1;
                }
                _ => {}
            }
        }

        if new_page != 0 {
            if PROPSHEET_CanSetCurSel(hwnd) != FALSE {
                new_page += (*psInfo).active_page;

                if new_page < 0 {
                    new_page = (*psInfo).nPages - 1;
                } else if new_page >= (*psInfo).nPages {
                    new_page = 0;
                }

                PROPSHEET_SetCurSel(hwnd, new_page, 1, core::ptr::null_mut());
            }

            return TRUE;
        }
    }

    return IsDialogMessageW(hwnd, lpMsg);
}

unsafe extern "C" fn PROPSHEET_DoCommand(hwnd: HWND, wID: WORD) -> BOOL {
    'sw: {
        match (wID) as u32 {
            IDOK | IDC_APPLY_BUTTON => {
                let hwndApplyBtn: HWND = GetDlgItem(hwnd, (IDC_APPLY_BUTTON as i32));

                if PROPSHEET_Apply(
                    hwnd,
                    if (wID) as u32 == IDOK { 1 } else { 0 },
                ) == FALSE
                {
                    break 'sw;
                }

                if (wID) as u32 == IDOK {
                    let psInfo: *mut PropSheetInfo =
                        GetPropW(hwnd, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR)
                            as *mut PropSheetInfo;

                    if psInfo.is_null() {
                        break 'sw;
                    }

                    /* don't overwrite ID_PSRESTARTWINDOWS or ID_PSREBOOTSYSTEM */
                    if (*psInfo).result == 0 {
                        (*psInfo).result = (IDOK) as i32;
                    }

                    if (*psInfo).isModeless != 0 {
                        (*psInfo).activeValid = FALSE;
                    } else {
                        (*psInfo).ended = TRUE;
                    }
                } else {
                    EnableWindow(hwndApplyBtn, FALSE);
                }

                break 'sw;
            }

            IDC_BACK_BUTTON => {
                PROPSHEET_Back(hwnd);
                break 'sw;
            }

            IDC_NEXT_BUTTON => {
                PROPSHEET_Next(hwnd);
                break 'sw;
            }

            IDC_FINISH_BUTTON => {
                PROPSHEET_Finish(hwnd);
                break 'sw;
            }

            IDCANCEL => {
                PROPSHEET_Cancel(hwnd, 0);
                break 'sw;
            }

            IDHELP => {
                PROPSHEET_Help(hwnd);
                break 'sw;
            }

            _ => {
                return FALSE;
            }
        }
    }

    return TRUE;
}

unsafe extern "C" fn PROPSHEET_Paint(hwnd: HWND, hdcParam: HDC) -> LRESULT {
    let mut psInfo: *mut PropSheetInfo = GetPropW(hwnd, core::ptr::addr_of!(PropSheetInfoStr) as LPCWSTR) as *mut PropSheetInfo;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hdcSrc: HDC = core::ptr::null_mut();
    let mut bm: BITMAP = core::mem::zeroed();
    let mut hbmp: HBITMAP = core::ptr::null_mut();
    let mut hOldPal: HPALETTE = core::ptr::null_mut();
    let mut offsety: core::ffi::c_int = 0;
    let mut hbr: HBRUSH = core::ptr::null_mut();
    let mut r: RECT = core::mem::zeroed();
    let mut rzone: RECT = core::mem::zeroed();
    let mut ppshpage: LPCPROPSHEETPAGEW = core::ptr::null();
    let mut szBuffer: [WCHAR; 256] = [0; 256];
    let mut nLength: core::ffi::c_int = 0;

    hdc = if !hdcParam.is_null() {
        hdcParam
    } else {
        BeginPaint(hwnd, core::ptr::addr_of_mut!(ps))
    };
    if hdc.is_null() {
        return 1;
    }

    hdcSrc = CreateCompatibleDC(core::ptr::null_mut());

    if ((*psInfo).ppshheader.dwFlags & PSH_USEHPLWATERMARK) != 0 {
        hOldPal = SelectPalette(hdc, (*psInfo).ppshheader.hplWatermark, FALSE);
    }

    if (*psInfo).active_page < 0 {
        ppshpage = core::ptr::null();
    } else {
        ppshpage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hpage as LPCPROPSHEETPAGEW;
    }

    if (!ppshpage.is_null() && ((*ppshpage).dwFlags & PSP_HIDEHEADER) == 0)
        && (((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_OLD | PSH_WIZARD97_NEW)) != 0)
        && (((*psInfo).ppshheader.dwFlags & PSH_HEADER) != 0)
    {
        let mut hwndLineHeader: HWND = GetDlgItem(hwnd, (IDC_SUNKEN_LINEHEADER as i32));
        let mut hOldFont: HFONT = core::ptr::null_mut();
        let mut clrOld: COLORREF = 0;
        let mut oldBkMode: core::ffi::c_int = 0;

        GetClientRect(hwndLineHeader, core::ptr::addr_of_mut!(r));
        MapWindowPoints(hwndLineHeader, hwnd, core::ptr::addr_of_mut!(r) as LPPOINT, 2);
        SetRect(
            core::ptr::addr_of_mut!(rzone),
            0,
            0,
            r.right as core::ffi::c_int + 1,
            r.top as core::ffi::c_int - 1,
        );

        hOldFont = SelectObject(hdc, (*psInfo).hFontBold as HGDIOBJ) as HFONT;

        if !(*psInfo).ppshheader.u5.hbmHeader.is_null() {
            hbmp = SelectObject(hdcSrc, (*psInfo).ppshheader.u5.hbmHeader as HGDIOBJ) as HBITMAP;

            GetObjectW(
                (*psInfo).ppshheader.u5.hbmHeader as HANDLE,
                core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                core::ptr::addr_of_mut!(bm) as LPVOID,
            );
            if ((*psInfo).ppshheader.dwFlags & PSH_WIZARD97_OLD) != 0 {
                /* Fill the unoccupied part of the header with color of the
                 * left-top pixel, but do it only when needed.
                 */
                if bm.bmWidth < r.right || bm.bmHeight < r.bottom {
                    hbr = CreateSolidBrush(GetPixel(hdcSrc, 0, 0));
                    r = rzone;
                    if bm.bmWidth < r.right {
                        r.left = bm.bmWidth;
                        FillRect(hdc, core::ptr::addr_of_mut!(r) as LPCRECT, hbr);
                    }
                    if bm.bmHeight < r.bottom {
                        r.left = 0;
                        r.top = bm.bmHeight;
                        FillRect(hdc, core::ptr::addr_of_mut!(r) as LPCRECT, hbr);
                    }
                    DeleteObject(hbr as HGDIOBJ);
                }

                /* Draw the header itself. */
                BitBlt(
                    hdc,
                    0,
                    0,
                    bm.bmWidth,
                    min(bm.bmHeight, rzone.bottom),
                    hdcSrc,
                    0,
                    0,
                    SRCCOPY,
                );
            } else {
                let mut margin: core::ffi::c_int = 0;
                hbr = GetSysColorBrush((COLOR_WINDOW) as i32);
                FillRect(hdc, core::ptr::addr_of_mut!(rzone) as LPCRECT, hbr);

                /* Draw the header bitmap. It's always centered like a
                 * common 49 x 49 bitmap. */
                margin = (rzone.bottom - 49) / 2;
                BitBlt(
                    hdc,
                    rzone.right - 49 - margin,
                    margin,
                    min(bm.bmWidth, 49),
                    min(bm.bmHeight, 49),
                    hdcSrc,
                    0,
                    0,
                    SRCCOPY,
                );

                /* NOTE: Native COMCTL32 draws a white stripe over the bitmap
                 * if its height is smaller than 49 pixels. Because the reason
                 * for this bug is unknown the current code doesn't try to
                 * replicate it. */
            }

            SelectObject(hdcSrc, hbmp as HGDIOBJ);
        }

        clrOld = SetTextColor(hdc, 0x00000000);
        oldBkMode = SetBkMode(hdc, (TRANSPARENT as i32));

        if ((*ppshpage).dwFlags & PSP_USEHEADERTITLE) != 0 {
            SetRect(
                core::ptr::addr_of_mut!(r),
                20,
                10,
                0,
                0,
            );
            if IS_INTRESOURCE!((*ppshpage).pszHeaderTitle) == false {
                DrawTextW(
                    hdc,
                    (*ppshpage).pszHeaderTitle,
                    -(1),
                    core::ptr::addr_of_mut!(r),
                    DT_LEFT | DT_SINGLELINE | DT_NOCLIP,
                );
            } else {
                nLength = LoadStringW(
                    (*ppshpage).hInstance,
                    (*ppshpage).pszHeaderTitle as UINT_PTR as UINT,
                    core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
                    256,
                );
                if nLength != 0 {
                    DrawTextW(
                        hdc,
                        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
                        nLength,
                        core::ptr::addr_of_mut!(r),
                        DT_LEFT | DT_SINGLELINE | DT_NOCLIP,
                    );
                }
            }
        }

        if ((*ppshpage).dwFlags & PSP_USEHEADERSUBTITLE) != 0 {
            SelectObject(hdc, (*psInfo).hFont as HGDIOBJ);
            SetRect(
                core::ptr::addr_of_mut!(r),
                40,
                25,
                rzone.right - 69,
                rzone.bottom,
            );
            if IS_INTRESOURCE!((*ppshpage).pszHeaderSubTitle) == false {
                DrawTextW(
                    hdc,
                    (*ppshpage).pszHeaderSubTitle,
                    -(1),
                    core::ptr::addr_of_mut!(r),
                    DT_LEFT | DT_WORDBREAK,
                );
            } else {
                nLength = LoadStringW(
                    (*ppshpage).hInstance,
                    (*ppshpage).pszHeaderSubTitle as UINT_PTR as UINT,
                    core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
                    256,
                );
                if nLength != 0 {
                    DrawTextW(
                        hdc,
                        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
                        nLength,
                        core::ptr::addr_of_mut!(r),
                        DT_LEFT | DT_WORDBREAK,
                    );
                }
            }
        }

        offsety = rzone.bottom + 2;

        SetTextColor(hdc, clrOld);
        SetBkMode(hdc, oldBkMode);
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }

    if (!ppshpage.is_null() && ((*ppshpage).dwFlags & PSP_HIDEHEADER) != 0)
        && (((*psInfo).ppshheader.dwFlags & (PSH_WIZARD97_OLD | PSH_WIZARD97_NEW)) != 0)
        && (((*psInfo).ppshheader.dwFlags & PSH_WATERMARK) != 0)
        && (!(*psInfo).ppshheader.u4.hbmWatermark.is_null())
    {
        let mut hwndLine: HWND = GetDlgItem(hwnd, (IDC_SUNKEN_LINE as i32));

        GetClientRect(hwndLine, core::ptr::addr_of_mut!(r));
        MapWindowPoints(hwndLine, hwnd, core::ptr::addr_of_mut!(r) as LPPOINT, 2);
        SetRect(
            core::ptr::addr_of_mut!(rzone),
            0,
            0,
            r.right,
            r.top - 1,
        );

        hbr = GetSysColorBrush((COLOR_WINDOW) as i32);
        FillRect(hdc, core::ptr::addr_of_mut!(rzone) as LPCRECT, hbr);

        GetObjectW(
            (*psInfo).ppshheader.u4.hbmWatermark as HANDLE,
            core::mem::size_of::<BITMAP>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        );
        hbmp = SelectObject(hdcSrc, (*psInfo).ppshheader.u4.hbmWatermark as HGDIOBJ) as HBITMAP;

        /* The watermark is truncated to a width of 164 pixels */
        r.right = min(r.right, 164);
        BitBlt(
            hdc,
            0,
            offsety,
            min(bm.bmWidth, r.right),
            min(bm.bmHeight, r.bottom),
            hdcSrc,
            0,
            0,
            SRCCOPY,
        );

        /* If the bitmap is not big enough, fill the remaining area
           with the color of pixel (0,0) of bitmap - see MSDN */
        if r.top > bm.bmHeight {
            r.bottom = r.top - 1;
            r.top = bm.bmHeight;
            r.left = 0;
            r.right = bm.bmWidth;
            hbr = CreateSolidBrush(GetPixel(hdcSrc, 0, 0));
            FillRect(hdc, core::ptr::addr_of_mut!(r) as LPCRECT, hbr);
            DeleteObject(hbr as HGDIOBJ);
        }

        SelectObject(hdcSrc, hbmp as HGDIOBJ);
    }

    if ((*psInfo).ppshheader.dwFlags & PSH_USEHPLWATERMARK) != 0 {
        SelectPalette(hdc, hOldPal, FALSE);
    }

    DeleteDC(hdcSrc);

    if hdcParam.is_null() {
        EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
    }

    return 0;
}

unsafe extern "system" fn PROPSHEET_DialogProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let mut psInfo: *mut PropSheetInfo = lParam as *mut PropSheetInfo;
            let mut strCaption: *mut WCHAR = Alloc(
                (MAX_CAPTION_LENGTH as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
            ) as *mut WCHAR;
            let mut hwndTabCtrl: HWND = GetDlgItem(hwnd, (IDC_TABCONTROL as i32));
            let mut idx: ::core::ffi::c_int = 0;
            let mut logFont: LOGFONTW = core::mem::zeroed();

            SetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR, psInfo as HANDLE);

            (*psInfo).hwnd = hwnd;
            SetWindowLongPtrW(hwnd, DWLP_USER, (psInfo as DWORD_PTR) as LONG_PTR);

            if (*psInfo).ppshheader.dwFlags & INTRNL_ANY_WIZARD != 0 {
                PROPSHEET_SetWizButtons(hwnd, PSWIZB_BACK | PSWIZB_NEXT);
            }

            SystemParametersInfoW(
                SPI_GETICONTITLELOGFONT,
                0,
                &raw mut logFont as PVOID,
                0,
            );
            (*psInfo).hFont = CreateFontIndirectW(&raw mut logFont);
            logFont.lfWeight = (FW_BOLD as i32);
            (*psInfo).hFontBold = CreateFontIndirectW(&raw mut logFont);

            if ((*psInfo).ppshheader.dwFlags & PSH_USEICONID != 0)
                || ((*psInfo).ppshheader.dwFlags & PSH_USEHICON != 0)
            {
                let mut hIcon: HICON = core::ptr::null_mut();
                let mut icon_cx: ::core::ffi::c_int = GetSystemMetrics((SM_CXSMICON) as i32);
                let mut icon_cy: ::core::ffi::c_int = GetSystemMetrics((SM_CYSMICON) as i32);

                if (*psInfo).ppshheader.dwFlags & PSH_USEICONID != 0 {
                    hIcon = LoadImageW(
                        (*psInfo).ppshheader.hInstance,
                        (*psInfo).ppshheader.u.pszIcon,
                        IMAGE_ICON,
                        icon_cx,
                        icon_cy,
                        LR_DEFAULTCOLOR,
                    ) as HICON;
                } else {
                    hIcon = (*psInfo).ppshheader.u.hIcon;
                }

                SendMessageW(hwnd, WM_SETICON, 0, hIcon as LPARAM);
            }

            if (*psInfo).ppshheader.dwFlags & PSH_USEHICON != 0 {
                SendMessageW(
                    hwnd,
                    WM_SETICON,
                    0,
                    (*psInfo).ppshheader.u.hIcon as LPARAM,
                );
            }

            (*psInfo).strPropertiesFor = strCaption as LPWSTR;

            GetWindowTextW(hwnd, (*psInfo).strPropertiesFor, MAX_CAPTION_LENGTH);

            PROPSHEET_CreateTabControl(hwnd, psInfo);

            PROPSHEET_LoadWizardBitmaps(psInfo);

            if (*psInfo).ppshheader.dwFlags & INTRNL_ANY_WIZARD != 0 {
                ShowWindow(hwndTabCtrl, (SW_HIDE) as i32);
                PROPSHEET_AdjustSizeWizard(hwnd, psInfo);
                PROPSHEET_AdjustButtonsWizard(hwnd, psInfo);
                SetFocus(GetDlgItem(hwnd, (IDC_NEXT_BUTTON as i32)));
            } else {
                if PROPSHEET_SizeMismatch(hwnd, psInfo) != 0 {
                    PROPSHEET_AdjustSize(hwnd, psInfo);
                    PROPSHEET_AdjustButtons(hwnd, psInfo);
                }
                SetFocus(GetDlgItem(hwnd, (IDOK) as i32));
            }

            /* Move the window position if necessary */
            {
                let mut dx: INT = 0;
                let mut dy: INT = 0;
                let mut rcInit: RECT = core::mem::zeroed();
                let mut hwndParent: HWND = (*psInfo).ppshheader.hwndParent;
                let mut bMove: BOOL = FALSE;

                GetWindowRect(hwnd, &raw mut rcInit);
                dx = rcInit.right - rcInit.left;
                dy = rcInit.bottom - rcInit.top;

                if IsWindow(hwndParent) != 0 {
                    let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
                    wndpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
                    bMove = TRUE;

                    /* hwndParent can be minimized (See Control_ShowAppletInTaskbar).
                       Use normal position. */
                    GetWindowPlacement(hwndParent, &raw mut wndpl);
                    rcInit = wndpl.rcNormalPosition;
                    if IsWindowVisible(hwndParent) != 0 && IsIconic(hwndParent) == 0 {
                        /* Is it Right-to-Left layout? */
                        if (GetWindowLongPtrW(hwndParent, GWL_EXSTYLE) & (WS_EX_RTLREADING) as LONG_PTR) != 0 {
                            rcInit.left = rcInit.right - dx - GetSystemMetrics((SM_CXSMICON) as i32);
                        } else {
                            rcInit.left += GetSystemMetrics((SM_CXSMICON) as i32);
                        }

                        rcInit.top += GetSystemMetrics((SM_CYSMICON) as i32);
                    }
                } else {
                    /* We cannot foresee CW_USEDEFAULT's position without communicating with USER32.
                       Use a top-level STATIC control to get the proper position. */
                    let mut hwndDummy: HWND = CreateWindowExW(
                        0,
                        (WC_STATICW).as_ptr(),
                        core::ptr::null_mut(),
                        0,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        dx,
                        dy,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        GetModuleHandleW(core::ptr::null_mut()),
                        core::ptr::null_mut(),
                    );
                    if !hwndDummy.is_null() {
                        bMove = TRUE;
                        GetWindowRect(hwndDummy, &raw mut rcInit);
                        DestroyWindow(hwndDummy);
                    }
                }

                if bMove != 0 {
                    let mut mi: MONITORINFO = core::mem::zeroed();
                    mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
                    let mut hMonitor: HMONITOR = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                    if GetMonitorInfo(hMonitor, &raw mut mi) != 0 {
                        /* Try to fit it onto the desktop */
                        if mi.rcWork.right < rcInit.left + dx {
                            rcInit.left = mi.rcWork.right - dx;
                        }
                        if mi.rcWork.bottom < rcInit.top + dy {
                            rcInit.top = mi.rcWork.bottom - dy;
                        }
                        if rcInit.left < mi.rcWork.left {
                            rcInit.left = mi.rcWork.left;
                        }
                        if rcInit.top < mi.rcWork.top {
                            rcInit.top = mi.rcWork.top;
                        }
                        SetWindowPos(
                            hwnd,
                            core::ptr::null_mut(),
                            rcInit.left,
                            rcInit.top,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
                        );
                    }
                }
            }

            if IS_INTRESOURCE!((*psInfo).ppshheader.pszCaption) != false
                && !(*psInfo).ppshheader.hInstance.is_null()
            {
                let mut szText: [WCHAR; 256] = [0; 256];

                if LoadStringW(
                    (*psInfo).ppshheader.hInstance,
                    (*psInfo).ppshheader.pszCaption as UINT_PTR as UINT,
                    &raw mut szText as LPWSTR,
                    255,
                ) != 0
                {
                    PROPSHEET_SetTitleW(
                        hwnd,
                        (*psInfo).ppshheader.dwFlags,
                        &raw mut szText as *mut WCHAR as LPCWSTR,
                    );
                }
            } else {
                PROPSHEET_SetTitleW(
                    hwnd,
                    (*psInfo).ppshheader.dwFlags,
                    (*psInfo).ppshheader.pszCaption,
                );
            }

            if (*psInfo).useCallback != 0 {
                ((*psInfo).ppshheader.pfnCallback.unwrap_unchecked())(
                    hwnd,
                    PSCB_INITIALIZED as UINT,
                    0 as LPARAM,
                );
            }

            idx = (*psInfo).active_page;
            (*psInfo).active_page = -1;

            PROPSHEET_SetCurSel(
                hwnd,
                idx,
                1,
                (*(*psInfo).proppage.offset(idx as isize)).hpage,
            );

            /* doing TCM_SETCURSEL seems to be needed even in case of PSH_WIZARD,
             * as some programs call TCM_GETCURSEL to get the current selection
             * from which to switch to the next page */
            SendMessageW(
                hwndTabCtrl,
                TCM_SETCURSEL,
                (*psInfo).active_page as WPARAM,
                0,
            );

            PROPSHEET_UnChanged(hwnd, core::ptr::null_mut());

            /* wizards set their focus during init */
            if (*psInfo).ppshheader.dwFlags & INTRNL_ANY_WIZARD != 0 {
                return (FALSE) as INT_PTR;
            }

            return (FALSE) as INT_PTR;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            PROPSHEET_Paint(hwnd, wParam as HDC);
            return (TRUE) as INT_PTR;
        }

        WM_DESTROY => {
            PROPSHEET_CleanUp(hwnd);
            return (TRUE) as INT_PTR;
        }

        WM_CLOSE => {
            PROPSHEET_Cancel(hwnd, 1);
            return (FALSE) as INT_PTR; /* let DefDlgProc post us WM_COMMAND/IDCANCEL */
        }

        WM_COMMAND => {
            if PROPSHEET_DoCommand(hwnd, LOWORD!(wParam)) == 0 {
                let mut psInfo: *mut PropSheetInfo =
                    GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;

                if psInfo.is_null() {
                    return (FALSE) as INT_PTR;
                }

                /* No default handler, forward notification to active page */
                if (*psInfo).activeValid != 0 && (*psInfo).active_page != -1 {
                    let mut hwndPage: HWND =
                        (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
                    SendMessageW(hwndPage, WM_COMMAND, wParam, lParam);
                }
            }
            return (TRUE) as INT_PTR;
        }

        WM_NOTIFY => {
            let mut pnmh: *mut NMHDR = lParam as LPNMHDR;

            if (*pnmh).code == (TCN_SELCHANGE) as u32 {
                let mut index: ::core::ffi::c_int = SendMessageW(
                    (*pnmh).hwndFrom,
                    TCM_GETCURSEL,
                    0,
                    0,
                ) as ::core::ffi::c_int;
                PROPSHEET_SetCurSel(hwnd, index, 1, core::ptr::null_mut());
            }

            if (*pnmh).code == (TCN_SELCHANGING) as u32 {
                let mut bRet: BOOL = PROPSHEET_CanSetCurSel(hwnd);
                SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (bRet == 0) as LONG_PTR);
                return (TRUE) as INT_PTR;
            }

            return (FALSE) as INT_PTR;
        }

        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            /* fall through into the COMMENT_TOKEN_ECHO____/COMMENT_TOKEN_ECHO____ block */
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
            let mut i: INT = 0;
            i = 0;
            while i < (*psInfo).nPages {
                let mut hwndPage: HWND = (*(*psInfo).proppage.offset(i as isize)).hwndPage;
                SendMessageW(hwndPage, uMsg, wParam, lParam);
                i += 1;
            }
            SendDlgItemMessageW(hwnd, (IDC_TABCONTROL as i32), uMsg, wParam, lParam);
            return (FALSE) as INT_PTR;
        }

        WM_DISPLAYCHANGE | WM_WININICHANGE => {
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
            let mut i: INT = 0;
            i = 0;
            while i < (*psInfo).nPages {
                let mut hwndPage: HWND = (*(*psInfo).proppage.offset(i as isize)).hwndPage;
                SendMessageW(hwndPage, uMsg, wParam, lParam);
                i += 1;
            }
            SendDlgItemMessageW(hwnd, (IDC_TABCONTROL as i32), uMsg, wParam, lParam);
            return (FALSE) as INT_PTR;
        }

        WM_ENABLE | WM_QUERYENDSESSION | WM_ENDSESSION | WM_DEVICECHANGE => {
            /* Send to the tab control, before forwarding it to the active page */
            SendDlgItemMessageW(hwnd, (IDC_TABCONTROL as i32), uMsg, wParam, lParam);
            /* fall through into the COMMENT_TOKEN_ECHO____/COMMENT_TOKEN_ECHO____ block */
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
            if psInfo.is_null() {
                return (FALSE) as INT_PTR;
            }

            /* Forward notification to active page */
            if (*psInfo).activeValid != 0 && (*psInfo).active_page != -1 {
                let mut hwndPage: HWND =
                    (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
                let mut msgResult: LRESULT = SendMessageW(hwndPage, uMsg, wParam, lParam);
                /* The message is handled, set the dialog return value
                 * to whatever the page returned */
                SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
                return (TRUE) as INT_PTR;
            }
            return (FALSE) as INT_PTR;
        }

        WM_ACTIVATE | WM_ACTIVATEAPP => {
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
            if psInfo.is_null() {
                return (FALSE) as INT_PTR;
            }

            /* Forward notification to active page */
            if (*psInfo).activeValid != 0 && (*psInfo).active_page != -1 {
                let mut hwndPage: HWND =
                    (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
                let mut msgResult: LRESULT = SendMessageW(hwndPage, uMsg, wParam, lParam);
                /* The message is handled, set the dialog return value
                 * to whatever the page returned */
                SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
                return (TRUE) as INT_PTR;
            }
            return (FALSE) as INT_PTR;
        }

        PSM_GETCURRENTPAGEHWND => {
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;
            let mut hwndPage: HWND = 0 as HWND;

            if psInfo.is_null() {
                return (FALSE) as INT_PTR;
            }

            if (*psInfo).activeValid != 0 && (*psInfo).active_page != -1 {
                hwndPage = (*(*psInfo).proppage.offset((*psInfo).active_page as isize)).hwndPage;
            }

            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (hwndPage as DWORD_PTR) as LONG_PTR);

            return (TRUE) as INT_PTR;
        }

        PSM_CHANGED => {
            PROPSHEET_Changed(hwnd, wParam as HWND);
            return (TRUE) as INT_PTR;
        }

        PSM_UNCHANGED => {
            PROPSHEET_UnChanged(hwnd, wParam as HWND);
            return (TRUE) as INT_PTR;
        }

        PSM_GETTABCONTROL => {
            let mut hwndTabCtrl: HWND = GetDlgItem(hwnd, (IDC_TABCONTROL as i32));

            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (hwndTabCtrl as DWORD_PTR) as LONG_PTR);

            return (TRUE) as INT_PTR;
        }

        PSM_SETCURSEL => {
            let mut msgResult: BOOL = 0;

            msgResult = PROPSHEET_CanSetCurSel(hwnd);
            if msgResult != FALSE {
                msgResult = PROPSHEET_SetCurSel(
                    hwnd,
                    wParam as ::core::ffi::c_int,
                    1,
                    lParam as HPROPSHEETPAGE,
                );
            }

            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (msgResult) as LONG_PTR);

            return (TRUE) as INT_PTR;
        }

        PSM_CANCELTOCLOSE => {
            let mut buf: [WCHAR; (MAX_BUTTONTEXT_LENGTH as usize)] = [0; (MAX_BUTTONTEXT_LENGTH as usize)];
            let mut hwndOK: HWND = GetDlgItem(hwnd, (IDOK) as i32);
            let mut hwndCancel: HWND = GetDlgItem(hwnd, (IDCANCEL) as i32);

            EnableWindow(hwndCancel, FALSE);
            if LoadStringW(
                COMCTL32_hModule,
                IDS_CLOSE,
                &raw mut buf as LPWSTR,
                ARRAY_SIZE!(buf) as ::core::ffi::c_int,
            ) != 0
            {
                SetWindowTextW(hwndOK, &raw mut buf as *mut WCHAR as LPCWSTR);
            }

            return (FALSE) as INT_PTR;
        }

        PSM_RESTARTWINDOWS => {
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;

            if psInfo.is_null() {
                return (FALSE) as INT_PTR;
            }

            /* reboot system takes precedence over restart windows */
            if (*psInfo).result != (ID_PSREBOOTSYSTEM as i32) {
                (*psInfo).result = (ID_PSRESTARTWINDOWS as i32);
            }

            return (TRUE) as INT_PTR;
        }

        PSM_REBOOTSYSTEM => {
            let mut psInfo: *mut PropSheetInfo =
                GetPropW(hwnd, &raw const PropSheetInfoStr as LPCWSTR) as *mut PropSheetInfo;

            if psInfo.is_null() {
                return (FALSE) as INT_PTR;
            }

            (*psInfo).result = (ID_PSREBOOTSYSTEM as i32);

            return (TRUE) as INT_PTR;
        }

        PSM_SETTITLEA => {
            PROPSHEET_SetTitleA(hwnd, wParam as DWORD, lParam as LPCSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_SETTITLEW => {
            PROPSHEET_SetTitleW(hwnd, wParam as DWORD, lParam as LPCWSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_APPLY => {
            let mut msgResult: BOOL = PROPSHEET_Apply(hwnd, 0);

            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (msgResult) as LONG_PTR);

            return (TRUE) as INT_PTR;
        }

        PSM_QUERYSIBLINGS => {
            let mut msgResult: LRESULT = PROPSHEET_QuerySiblings(hwnd, wParam, lParam);

            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);

            return (TRUE) as INT_PTR;
        }

        PSM_ADDPAGE => {
            /*
             * Note: MSVC++ 6.0 documentation says that PSM_ADDPAGE does not have
             *       a return value. This is not true. PSM_ADDPAGE returns TRUE
             *       on success or FALSE otherwise, as specified on MSDN Online.
             *       Also see the MFC code for
             *       CPropertySheet::AddPage(CPropertyPage* pPage).
             */

            let mut msgResult: BOOL = PROPSHEET_AddPage(hwnd, lParam as HPROPSHEETPAGE);

            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (msgResult) as LONG_PTR);

            return (TRUE) as INT_PTR;
        }

        PSM_REMOVEPAGE => {
            PROPSHEET_RemovePage(hwnd, wParam as ::core::ffi::c_int, lParam as HPROPSHEETPAGE);
            return (TRUE) as INT_PTR;
        }

        PSM_ISDIALOGMESSAGE => {
            let mut msgResult: BOOL = PROPSHEET_IsDialogMessage(hwnd, lParam as LPMSG);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (msgResult) as LONG_PTR);
            return (TRUE) as INT_PTR;
        }

        PSM_PRESSBUTTON => {
            PROPSHEET_PressButton(hwnd, wParam as ::core::ffi::c_int);
            return (TRUE) as INT_PTR;
        }

        PSM_SETFINISHTEXTA => {
            PROPSHEET_SetFinishTextA(hwnd, lParam as LPCSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_SETWIZBUTTONS => {
            PROPSHEET_SetWizButtons(hwnd, lParam as DWORD);
            return (TRUE) as INT_PTR;
        }

        PSM_SETCURSELID => {
            PROPSHEET_SetCurSelId(hwnd, lParam as ::core::ffi::c_int);
            return (TRUE) as INT_PTR;
        }

        PSM_SETFINISHTEXTW => {
            PROPSHEET_SetFinishTextW(hwnd, lParam as LPCWSTR);
            return (FALSE) as INT_PTR;
        }

        PSM_INSERTPAGE => {
            let mut msgResult: BOOL = PROPSHEET_InsertPage(
                hwnd,
                wParam as HPROPSHEETPAGE,
                lParam as HPROPSHEETPAGE,
            );
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, (msgResult) as LONG_PTR);
            return (TRUE) as INT_PTR;
        }

        PSM_SETHEADERTITLEW => {
            PROPSHEET_SetHeaderTitleW(hwnd, (wParam) as UINT, lParam as LPCWSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_SETHEADERTITLEA => {
            PROPSHEET_SetHeaderTitleA(hwnd, (wParam) as UINT, lParam as LPCSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_SETHEADERSUBTITLEW => {
            PROPSHEET_SetHeaderSubTitleW(hwnd, (wParam) as UINT, lParam as LPCWSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_SETHEADERSUBTITLEA => {
            PROPSHEET_SetHeaderSubTitleA(hwnd, (wParam) as UINT, lParam as LPCSTR);
            return (TRUE) as INT_PTR;
        }

        PSM_HWNDTOINDEX => {
            let mut msgResult: LRESULT = PROPSHEET_HwndToIndex(hwnd, wParam as HWND);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_INDEXTOHWND => {
            let mut msgResult: LRESULT = PROPSHEET_IndexToHwnd(hwnd, wParam as ::core::ffi::c_int);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_PAGETOINDEX => {
            let mut msgResult: LRESULT = PROPSHEET_PageToIndex(hwnd, wParam as HPROPSHEETPAGE);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_INDEXTOPAGE => {
            let mut msgResult: LRESULT = PROPSHEET_IndexToPage(hwnd, wParam as ::core::ffi::c_int);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_IDTOINDEX => {
            let mut msgResult: LRESULT = PROPSHEET_IdToIndex(hwnd, lParam as ::core::ffi::c_int);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_INDEXTOID => {
            let mut msgResult: LRESULT = PROPSHEET_IndexToId(hwnd, wParam as ::core::ffi::c_int);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_GETRESULT => {
            let mut msgResult: LRESULT = PROPSHEET_GetResult(hwnd);
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        PSM_RECALCPAGESIZES => {
            let mut msgResult: LRESULT = (PROPSHEET_RecalcPageSizes(hwnd)) as _;
            SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, msgResult);
            return (TRUE) as INT_PTR;
        }

        _ => return (FALSE) as INT_PTR,
    }
}
