use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut devMode: HGLOBAL =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut devNames: HGLOBAL =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut margins: RECT = tagRECT {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

static mut preview: previewinfo = _previewinfo {
    page: 0,
    pages_shown: 0,
    saved_pages_shown: 0,
    pageEnds: ::core::ptr::null::<::core::ffi::c_int>() as *mut ::core::ffi::c_int,
    pageCapacity: 0,
    textlength: 0,
    hdc: ::core::ptr::null::<HDC__>() as *mut HDC__,
    hdc2: ::core::ptr::null::<HDC__>() as *mut HDC__,
    window: tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    },
    rcPage: tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    },
    bmSize: tagSIZE { cx: 0, cy: 0 },
    bmScaledSize: tagSIZE { cx: 0, cy: 0 },
    spacing: tagSIZE { cx: 0, cy: 0 },
    zoomratio: 0.,
    zoomlevel: 0,
    wszFileName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
};

static mut var_pagemargin: [WCHAR; 11] = [
    'P' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut var_previewpages: [WCHAR; 13] = [
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut min_spacing: ::core::ffi::c_int = 10 as ::core::ffi::c_int;

#[repr(C)]
pub struct _previewinfo {
    pub page: i32,
    pub pages_shown: i32,
    pub saved_pages_shown: i32,
    pub pageEnds: *mut i32,
    pub pageCapacity: i32,
    pub textlength: i32,
    pub hdc: HDC,
    pub hdc2: HDC,
    pub window: RECT,
    pub rcPage: RECT,
    pub bmSize: SIZE,
    pub bmScaledSize: SIZE,
    pub spacing: SIZE,
    pub zoomratio: f32,
    pub zoomlevel: i32,
    pub wszFileName: LPWSTR,
}
pub type previewinfo = _previewinfo;
pub type ppreviewinfo = *mut previewinfo;

// Defined elsewhere (extern const WCHAR array); element count not known from this preamble.

unsafe extern "C" fn get_print_file_filter(hMainWnd: HWND) -> LPWSTR {
    static mut wszPrintFilter: [WCHAR; (MAX_STRING_LEN * 2 + 6 + 4 + 1) as _] = [0; (MAX_STRING_LEN * 2 + 6 + 4 + 1) as _];
    let files_prn: [u16; 6] = [0x002Au16, 0x002E, 0x0050, 0x0052, 0x004E, 0x0000];
    let files_all: [u16; 4] = [0x002Au16, 0x002E, 0x002A, 0x0000];
    let mut p: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;

    p = core::ptr::addr_of_mut!(wszPrintFilter) as *mut WCHAR as LPWSTR;
    LoadStringW(hInstance, STRING_PRINTER_FILES_PRN, p, (MAX_STRING_LEN as i32));
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    lstrcpyW(p, core::ptr::addr_of!(files_prn) as LPCWSTR);
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    LoadStringW(hInstance, STRING_ALL_FILES, p, (MAX_STRING_LEN as i32));
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    lstrcpyW(p, core::ptr::addr_of!(files_all) as LPCWSTR);
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    *p = 0;

    return core::ptr::addr_of_mut!(wszPrintFilter) as LPWSTR;
}
#[no_mangle]
pub unsafe extern "C" fn registry_set_pagemargins(hKey: HKEY) {
    RegSetValueExW(hKey, core::ptr::addr_of!(var_pagemargin) as LPCWSTR, 0 as DWORD, REG_BINARY as DWORD, core::ptr::addr_of_mut!(margins) as LPBYTE as *const BYTE, core::mem::size_of::<RECT>() as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_pagemargins(hKey: HKEY) {
    let mut size: DWORD = core::mem::size_of::<RECT>() as DWORD;

    if hKey.is_null()
        || RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(var_pagemargin) as LPCWSTR,
            0 as *mut DWORD,
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(margins) as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) != ERROR_SUCCESS as LSTATUS
        || size as usize != core::mem::size_of::<RECT>() as usize
    {
        SetRect(
            core::ptr::addr_of_mut!(margins),
            1757,
            1417,
            1757,
            1417,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn registry_set_previewpages(hKey: HKEY) {
    RegSetValueExW(
        hKey,
        core::ptr::addr_of!(var_previewpages) as LPCWSTR,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(preview.pages_shown) as LPBYTE as *const BYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn registry_read_previewpages(hKey: HKEY) {
    let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    if hKey.is_null()
        || RegQueryValueExW(
            hKey,
            core::ptr::addr_of!(var_previewpages) as *const u16,
            core::ptr::null_mut(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(preview.pages_shown) as LPBYTE,
            core::ptr::addr_of_mut!(size),
        ) != (ERROR_SUCCESS as i32)
        || size as usize != core::mem::size_of::<DWORD>() as usize
    {
        preview.pages_shown = 1;
    } else {
        if preview.pages_shown < 1 {
            preview.pages_shown = 1;
        } else if preview.pages_shown > 2 {
            preview.pages_shown = 2;
        }
    }
}

unsafe extern "C" fn AddTextButton(hRebarWnd: HWND, string: UINT, command: UINT, id: UINT) {
    let mut rb: REBARBANDINFOW = core::mem::zeroed();
    let hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
    let mut text: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let mut hButton: HWND;

    LoadStringW(hInstance, string, core::ptr::addr_of_mut!(text) as LPWSTR, (MAX_STRING_LEN as i32));
    hButton = CreateWindowW!(
        WC_BUTTONW as LPCWSTR,
        core::ptr::addr_of_mut!(text) as *mut WCHAR as LPCWSTR,
        (WS_VISIBLE | WS_CHILD) as DWORD,
        5 as i32,
        5 as i32,
        100 as i32,
        15 as i32,
        hRebarWnd,
        ULongToHandle!(command) as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );

    rb.cbSize = REBARBANDINFOW_V6_SIZE as UINT;
    rb.fMask = (RBBIM_SIZE | RBBIM_CHILDSIZE | RBBIM_STYLE | RBBIM_CHILD | RBBIM_IDEALSIZE | RBBIM_ID) as UINT;
    rb.fStyle = (RBBS_NOGRIPPER | RBBS_VARIABLEHEIGHT) as UINT;
    rb.hwndChild = hButton;
    rb.cyMinChild = 22 as UINT;
    rb.cyChild = rb.cyMinChild;
    rb.cxMinChild = 90 as UINT;
    rb.cx = rb.cxMinChild;
    rb.cxIdeal = 100 as UINT;
    rb.wID = id;

    SendMessageW(
        hRebarWnd,
        RB_INSERTBANDW as UINT,
        (-(1 as i32)) as WPARAM,
        core::ptr::addr_of_mut!(rb) as LPARAM,
    );
}

unsafe extern "C" fn make_dc() -> HDC {
    if !devNames.is_null() && !devMode.is_null() {
        let dn: LPDEVNAMES = GlobalLock(devNames) as LPDEVNAMES;
        let dm: LPDEVMODEW = GlobalLock(devMode) as LPDEVMODEW;
        let ret: HDC;

        ret = CreateDCW(
            (dn as LPWSTR).offset((*dn).wDriverOffset as isize) as LPCWSTR,
            (dn as LPWSTR).offset((*dn).wDeviceOffset as isize) as LPCWSTR,
            core::ptr::null(),
            dm as *const DEVMODEW,
        );

        // The C unlocks the LOCKED POINTERS; GlobalUnlock takes the handles, so the lock count
        // leaked on every call.
        GlobalUnlock(devNames);
        GlobalUnlock(devMode);

        return ret;
    } else {
        return core::ptr::null_mut();
    }
}

unsafe extern "C" fn twips_to_centmm(twips: ::core::ffi::c_int) -> LONG {
    return MulDiv(twips, (CENTMM_PER_INCH as i32), (TWIPS_PER_INCH as i32)) as LONG;
}

unsafe extern "C" fn centmm_to_twips(mm: ::core::ffi::c_int) -> LONG {
    return MulDiv(mm, (TWIPS_PER_INCH as i32), (CENTMM_PER_INCH as i32)) as LONG;
}

unsafe extern "C" fn twips_to_pixels(twips: i32, dpi: i32) -> LONG {
    return MulDiv(twips, dpi, (TWIPS_PER_INCH as i32)) as LONG;
}

unsafe extern "C" fn devunits_to_twips(units: i32, dpi: i32) -> LONG {
    return MulDiv(units, (TWIPS_PER_INCH as i32), dpi);
}

unsafe extern "C" fn get_print_rect(hdc: HDC) -> RECT {
    let mut rc: RECT = core::mem::zeroed();
    let mut width: i32;
    let mut height: i32;

    if !hdc.is_null() {
        let dpiY: i32 = GetDeviceCaps(hdc, (LOGPIXELSY as i32));
        let dpiX: i32 = GetDeviceCaps(hdc, (LOGPIXELSX as i32));
        width = devunits_to_twips(GetDeviceCaps(hdc, (PHYSICALWIDTH as i32)), dpiX);
        height = devunits_to_twips(GetDeviceCaps(hdc, (PHYSICALHEIGHT as i32)), dpiY);
    } else {
        width = centmm_to_twips(18500);
        height = centmm_to_twips(27000);
    }

    SetRect(
        core::ptr::addr_of_mut!(rc),
        margins.left,
        margins.top,
        width - margins.right,
        height - margins.bottom,
    );

    return rc;
}
#[no_mangle]
pub unsafe extern "C" fn target_device(hMainWnd: HWND, wordWrap: DWORD) {
    let hEditorWnd: HWND = GetDlgItem(hMainWnd, (IDC_EDITOR as i32));

    if wordWrap == ID_WORDWRAP_MARGIN {
        let mut width: i32 = 0;
        let mut result: LRESULT;
        let mut hdc: HDC = make_dc();
        let rc: RECT = get_print_rect(hdc);

        width = rc.right - rc.left;
        if hdc.is_null() {
            let hMaindc: HDC = GetDC(hMainWnd);
            hdc = CreateCompatibleDC(hMaindc);
            ReleaseDC(hMainWnd, hMaindc);
        }
        result = SendMessageW(hEditorWnd, EM_SETTARGETDEVICE, hdc as WPARAM, width as LPARAM);
        DeleteDC(hdc);
        if result != 0 {
            return;
        }
        /* otherwise EM_SETTARGETDEVICE failed, so fall back on wrapping
         * to window using the NULL DC. */
    }

    if wordWrap != ID_WORDWRAP_NONE {
        SendMessageW(hEditorWnd, EM_SETTARGETDEVICE, 0, 0);
    } else {
        SendMessageW(hEditorWnd, EM_SETTARGETDEVICE, 0, 1);
    }
}

unsafe extern "C" fn dialog_print_to_file(hMainWnd: HWND) -> LPWSTR {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    // `static WCHAR file[MAX_PATH] = {'O','U','T','P','U','T','.','P','R','N',0};` -- C
    // zero-fills the rest of the MAX_PATH buffer GetSaveFileNameW writes the chosen path into.
    static mut file: [WCHAR; (MAX_PATH) as _] = {
        let init = [
            'O' as u16, 'U' as u16, 'T' as u16, 'P' as u16, 'U' as u16, 'T' as u16,
            '.' as u16, 'P' as u16, 'R' as u16, 'N' as u16,
        ];
        let mut a = [0 as WCHAR; (MAX_PATH) as _];
        let mut i = 0;
        while i < init.len() {
            a[i] = init[i];
            i += 1;
        }
        a
    };
    static mut defExt: [WCHAR; 4] = [0x0050u16, 0x0052, 0x004E, 0x0000];
    static mut file_filter: LPWSTR = core::ptr::null_mut();

    if file_filter.is_null() {
        file_filter = get_print_file_filter(hMainWnd);
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );

    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.Flags = (OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_OVERWRITEPROMPT) as DWORD;
    ofn.hwndOwner = hMainWnd;
    ofn.lpstrFilter = file_filter as LPCWSTR;
    ofn.lpstrFile = core::ptr::addr_of_mut!(file) as *mut WCHAR as LPWSTR;
    ofn.nMaxFile = MAX_PATH as DWORD;
    ofn.lpstrDefExt = core::ptr::addr_of!(defExt) as *const WCHAR as LPCWSTR;

    if GetSaveFileNameW(core::ptr::addr_of_mut!(ofn)) != 0 {
        return core::ptr::addr_of_mut!(file) as *mut WCHAR as LPWSTR;
    } else {
        return core::ptr::null_mut();
    }
}

unsafe extern "C" fn char_from_pagenum(
    hEditorWnd: HWND,
    fr: *mut FORMATRANGE,
    page: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0;
    (*fr).chrg.cpMin = 0 as LONG;
    i = 1 as ::core::ffi::c_int;
    while i < page {
        let mut bottom: ::core::ffi::c_int = (*fr).rc.bottom as ::core::ffi::c_int;
        (*fr).chrg.cpMin = SendMessageW(
            hEditorWnd,
            EM_FORMATRANGE as UINT,
            FALSE as WPARAM,
            fr as LPARAM,
        ) as LONG;
        (*fr).rc.bottom = bottom as LONG;
        i += 1;
    }
}

unsafe extern "C" fn get_ruler_wnd(mut hMainWnd: HWND) -> HWND {
    return GetDlgItem(GetDlgItem(hMainWnd, (IDC_REBAR as i32)), (IDC_RULER as i32));
}
#[no_mangle]
pub unsafe extern "C" fn redraw_ruler(hRulerWnd: HWND) {
    let mut rc: RECT = core::mem::zeroed();

    GetClientRect(hRulerWnd, core::ptr::addr_of_mut!(rc));
    InvalidateRect(hRulerWnd, core::ptr::addr_of_mut!(rc) as *const RECT, (TRUE) as BOOL);
}

unsafe extern "C" fn update_ruler(mut hRulerWnd: HWND) {
    SendMessageW(hRulerWnd, WM_USER, 0, 0);
    redraw_ruler(hRulerWnd);
}

unsafe extern "C" fn add_ruler_units(
    mut hdcRuler: HDC,
    mut drawRect: *mut RECT,
    mut NewMetrics: BOOL,
    mut EditLeftmost: LONG,
) {
    static mut hdc: HDC = core::ptr::null_mut();

    if NewMetrics != 0 {
        static mut hBitmap: HBITMAP = core::ptr::null_mut();
        let mut i: ::core::ffi::c_int = 0;
        let mut x: ::core::ffi::c_int = 0;
        let mut y: ::core::ffi::c_int = 0;
        let mut RulerTextEnd: ::core::ffi::c_int = 0;
        let mut CmPixels: ::core::ffi::c_int = 0;
        let mut QuarterCmPixels: ::core::ffi::c_int = 0;
        let mut hFont: HFONT = core::ptr::null_mut();
        let mut FontName: [WCHAR; 14] = [
    0x004Du16, 0x0053, 0x0020, 0x0053, 0x0061, 0x006E, 0x0073, 0x0020, 0x0053, 0x0065,
    0x0072, 0x0069, 0x0066, 0x0000,
];

        if !hdc.is_null() {
            DeleteDC(hdc);
            DeleteObject(hBitmap as HGDIOBJ);
        }

        hdc = CreateCompatibleDC(core::ptr::null_mut());

        CmPixels = twips_to_pixels(
            centmm_to_twips(1000) as ::core::ffi::c_int,
            GetDeviceCaps(hdc, (LOGPIXELSX as i32)),
        ) as ::core::ffi::c_int;
        QuarterCmPixels = ((CmPixels as ::core::ffi::c_float as ::core::ffi::c_double)
            / (4.0 as ::core::ffi::c_double))
            as ::core::ffi::c_int;

        hBitmap = CreateCompatibleBitmap(
            hdc,
            (*drawRect).right as INT,
            (*drawRect).bottom as INT,
        );
        SelectObject(hdc, hBitmap as HGDIOBJ);
        FillRect(
            hdc,
            drawRect as LPCRECT,
            GetStockObject((WHITE_BRUSH as i32)) as HBRUSH,
        );

        hFont = CreateFontW(
            10,
            6,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            core::ptr::addr_of_mut!(FontName) as *mut WCHAR as LPCWSTR,
        );

        SelectObject(hdc, hFont as HGDIOBJ);
        SetBkMode(hdc, (TRANSPARENT as i32));
        SetTextAlign(hdc, TA_CENTER as UINT);

        y = ((((*drawRect).bottom as ::core::ffi::c_float
            - (*drawRect).top as ::core::ffi::c_float)
            as ::core::ffi::c_double
            / (2.0 as ::core::ffi::c_double))
            as ::core::ffi::c_int)
            + 1;
        RulerTextEnd = ((*drawRect).right - EditLeftmost + 1)
            as ::core::ffi::c_int;

        i = 1;
        x = EditLeftmost as ::core::ffi::c_int;
        while (x as LONG) < (*drawRect).right - EditLeftmost + 1 {
            let mut str: [WCHAR; 3] = [0; 3];
            let mut format: [WCHAR; 3] = [0x0025u16, 0x0064, 0x0000];
            let mut x2: ::core::ffi::c_int = x;

            x2 += QuarterCmPixels;
            if x2 > RulerTextEnd {
                break;
            }

            MoveToEx(hdc, x2, y, core::ptr::null_mut());
            LineTo(hdc, x2, y + 2);

            x2 += QuarterCmPixels;
            if x2 > RulerTextEnd {
                break;
            }

            MoveToEx(
                hdc,
                x2,
                y - 3,
                core::ptr::null_mut(),
            );
            LineTo(hdc, x2, y + 3);

            x2 += QuarterCmPixels;
            if x2 > RulerTextEnd {
                break;
            }

            MoveToEx(hdc, x2, y, core::ptr::null_mut());
            LineTo(hdc, x2, y + 2);

            x += CmPixels;
            if x > RulerTextEnd {
                break;
            }

            wsprintfW(
                core::ptr::addr_of_mut!(str) as *mut WCHAR as LPWSTR,
                core::ptr::addr_of_mut!(format) as *mut WCHAR as LPCWSTR,
                i,
            );
            TextOutW(
                hdc,
                x,
                5,
                core::ptr::addr_of_mut!(str) as *mut WCHAR as LPCWSTR,
                lstrlenW(core::ptr::addr_of_mut!(str) as *mut WCHAR as LPCWSTR),
            );

            i += 1;
        }
        DeleteObject(hFont as HGDIOBJ);
    }

    BitBlt(
        hdcRuler,
        0 as INT,
        0 as INT,
        (*drawRect).right as INT,
        (*drawRect).bottom as INT,
        hdc,
        0 as INT,
        0 as INT,
        SRCAND as DWORD,
    );
}

unsafe extern "C" fn paint_ruler(hWnd: HWND, EditLeftmost: LONG, NewMetrics: BOOL) {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let hdc: HDC = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
    let hdcPrint: HDC = make_dc();
    let printRect: RECT = get_print_rect(hdcPrint);
    let mut drawRect: RECT = core::mem::zeroed();

    GetClientRect(hWnd, core::ptr::addr_of_mut!(drawRect));
    FillRect(
        hdc,
        core::ptr::addr_of_mut!(drawRect) as LPCRECT,
        GetSysColorBrush((COLOR_MENU as i32)),
    );

    InflateRect(
        core::ptr::addr_of_mut!(drawRect),
        0 as i32,
        -3 as i32,
    );
    drawRect.left = EditLeftmost;
    drawRect.right = twips_to_pixels(
        (printRect.right - margins.left) as i32,
        GetDeviceCaps(hdc, (LOGPIXELSX as i32)),
    );
    FillRect(
        hdc,
        core::ptr::addr_of_mut!(drawRect) as LPCRECT,
        GetStockObject((WHITE_BRUSH as i32)) as HBRUSH,
    );

    drawRect.top -= 1;
    drawRect.bottom += 1;
    DrawEdge(
        hdc,
        core::ptr::addr_of_mut!(drawRect),
        EDGE_SUNKEN as UINT,
        BF_RECT as UINT,
    );

    drawRect.left = drawRect.right - 1 as LONG;
    drawRect.right = twips_to_pixels(
        (printRect.right + margins.right - margins.left) as i32,
        GetDeviceCaps(hdc, (LOGPIXELSX as i32)),
    );
    DrawEdge(
        hdc,
        core::ptr::addr_of_mut!(drawRect),
        EDGE_ETCHED as UINT,
        BF_RECT as UINT,
    );

    drawRect.left = 0 as LONG;
    drawRect.top = 0 as LONG;
    add_ruler_units(
        hdc,
        core::ptr::addr_of_mut!(drawRect),
        NewMetrics,
        EditLeftmost,
    );

    SelectObject(
        hdc,
        GetStockObject((BLACK_BRUSH as i32)),
    );
    DeleteDC(hdcPrint);
    EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
}
#[no_mangle]
pub unsafe extern "system" fn ruler_proc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    static mut pPrevRulerProc: WNDPROC = None;
    static mut EditLeftmost: LONG = 0;
    static mut NewMetrics: BOOL = 0;

    match msg {
        WM_USER => {
            if wParam != 0 {
                EditLeftmost = (*(wParam as *mut POINTL)).x;
                pPrevRulerProc = core::mem::transmute::<LPARAM, WNDPROC>(lParam);
            }
            NewMetrics = (TRUE as i32);
        }
        WM_PAINT => {
            paint_ruler(hWnd, EditLeftmost, NewMetrics);
        }
        _ => {
            return CallWindowProcW(pPrevRulerProc, hWnd, msg, wParam, lParam);
        }
    }

    return 0;
}

unsafe extern "C" fn print(pd: LPPRINTDLGW, wszFileName: LPWSTR) {
    let mut fr: FORMATRANGE = core::mem::zeroed();
    let mut di: DOCINFOW = core::mem::zeroed();
    let hEditorWnd: HWND = GetDlgItem((*pd).hwndOwner, (IDC_EDITOR as i32));
    let mut printedPages: ::core::ffi::c_int = 0;

    fr.hdc = (*pd).hDC;
    fr.hdcTarget = (*pd).hDC;

    fr.rc = get_print_rect(fr.hdc);
    SetRect(
        core::ptr::addr_of_mut!(fr.rcPage),
        0,
        0,
        (fr.rc.right + margins.right) as ::core::ffi::c_int,
        (fr.rc.bottom + margins.bottom) as ::core::ffi::c_int,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(di) as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<DOCINFOW>(),
    );
    di.cbSize = ::core::mem::size_of::<DOCINFOW>() as ::core::ffi::c_int;
    di.lpszDocName = wszFileName as LPCWSTR;

    if ((*pd).Flags & (PD_PRINTTOFILE as DWORD)) != 0 {
        di.lpszOutput = dialog_print_to_file((*pd).hwndOwner) as LPCWSTR;
        if di.lpszOutput.is_null() {
            return;
        }
    }

    if ((*pd).Flags & (PD_SELECTION as DWORD)) != 0 {
        SendMessageW(
            hEditorWnd,
            EM_EXGETSEL as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(fr.chrg) as LPARAM,
        );
    } else {
        let mut gt: GETTEXTLENGTHEX = core::mem::zeroed();
        gt.flags = GTL_DEFAULT as DWORD;
        gt.codepage = 1200 as UINT;
        fr.chrg.cpMin = 0 as LONG;
        fr.chrg.cpMax = SendMessageW(
            hEditorWnd,
            EM_GETTEXTLENGTHEX as UINT,
            core::ptr::addr_of_mut!(gt) as WPARAM,
            0 as LPARAM,
        ) as LONG;

        if ((*pd).Flags & (PD_PAGENUMS as DWORD)) != 0 {
            // The C passes nToPage, so a "pages 2-4" job starts at page 4 and prints 4-6; the
            // loop below counts nToPage - nFromPage + 1 pages from here, so start at the first.
            char_from_pagenum(hEditorWnd, core::ptr::addr_of_mut!(fr), (*pd).nFromPage as ::core::ffi::c_int);
        }
    }

    StartDocW(fr.hdc, core::ptr::addr_of_mut!(di));
    loop {
        if StartPage(fr.hdc) <= 0 {
            break;
        }

        fr.chrg.cpMin = SendMessageW(
            hEditorWnd,
            EM_FORMATRANGE as UINT,
            TRUE as WPARAM,
            core::ptr::addr_of_mut!(fr) as LPARAM,
        ) as LONG;

        if EndPage(fr.hdc) <= 0 {
            break;
        }

        printedPages += 1;
        if ((*pd).Flags & (PD_PAGENUMS as DWORD)) != 0
            && (printedPages
                > (*pd).nToPage as ::core::ffi::c_int - (*pd).nFromPage as ::core::ffi::c_int)
        {
            break;
        }

        if !(fr.chrg.cpMin != 0 && fr.chrg.cpMin < fr.chrg.cpMax) {
            break;
        }
    }

    EndDoc(fr.hdc);
    SendMessageW(
        hEditorWnd,
        EM_FORMATRANGE as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn dialog_printsetup(hMainWnd: HWND) {
    let mut ps: PAGESETUPDLGW = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(ps) as *mut core::ffi::c_void, core::mem::size_of::<PAGESETUPDLGW>());
    ps.lStructSize = core::mem::size_of::<PAGESETUPDLGW>() as DWORD;
    ps.hwndOwner = hMainWnd;
    ps.Flags = PSD_INHUNDREDTHSOFMILLIMETERS | PSD_MARGINS;
    SetRect(
        core::ptr::addr_of_mut!(ps.rtMargin),
        twips_to_centmm(margins.left as core::ffi::c_int),
        twips_to_centmm(margins.top as core::ffi::c_int),
        twips_to_centmm(margins.right as core::ffi::c_int),
        twips_to_centmm(margins.bottom as core::ffi::c_int),
    );
    ps.hDevMode = devMode;
    ps.hDevNames = devNames;

    if PageSetupDlgW(core::ptr::addr_of_mut!(ps)) != 0 {
        SetRect(
            core::ptr::addr_of_mut!(margins),
            centmm_to_twips(ps.rtMargin.left as core::ffi::c_int),
            centmm_to_twips(ps.rtMargin.top as core::ffi::c_int),
            centmm_to_twips(ps.rtMargin.right as core::ffi::c_int),
            centmm_to_twips(ps.rtMargin.bottom as core::ffi::c_int),
        );
        devMode = ps.hDevMode;
        devNames = ps.hDevNames;
        update_ruler(get_ruler_wnd(hMainWnd));
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_default_printer_opts() {
    let mut pd: PRINTDLGW = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(pd) as *mut core::ffi::c_void,
        core::mem::size_of::<PRINTDLGW>(),
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(pd) as *mut core::ffi::c_void,
        core::mem::size_of::<PRINTDLGW>(),
    );
    pd.lStructSize = core::mem::size_of::<PRINTDLGW>() as DWORD;
    pd.Flags = PD_RETURNDC | PD_RETURNDEFAULT;
    pd.hDevMode = devMode;

    PrintDlgW(core::ptr::addr_of_mut!(pd));

    devMode = pd.hDevMode;
    devNames = pd.hDevNames;
}
#[no_mangle]
pub unsafe extern "C" fn print_quick(hMainWnd: HWND, wszFileName: LPWSTR) {
    let mut pd: PRINTDLGW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(pd) as *mut core::ffi::c_void,
        core::mem::size_of::<PRINTDLGW>(),
    );
    pd.hwndOwner = hMainWnd;
    pd.hDC = make_dc();

    print(core::ptr::addr_of_mut!(pd), wszFileName);
    DeleteDC(pd.hDC);
}
#[no_mangle]
pub unsafe extern "C" fn dialog_print(hMainWnd: HWND, wszFileName: LPWSTR) {
    let mut pd: PRINTDLGW = core::mem::zeroed();
    let hEditorWnd: HWND = GetDlgItem(hMainWnd, (IDC_EDITOR as i32));
    let mut from: i32 = 0;
    let mut to: i32 = 0;

    ZeroMemory!(
        core::ptr::addr_of_mut!(pd) as *mut core::ffi::c_void,
        core::mem::size_of::<PRINTDLGW>(),
    );
    pd.lStructSize = core::mem::size_of::<PRINTDLGW>() as DWORD;
    pd.hwndOwner = hMainWnd;
    pd.Flags = (PD_RETURNDC | PD_USEDEVMODECOPIESANDCOLLATE) as DWORD;
    pd.nMinPage = 1 as WORD;
    pd.nMaxPage = (-(1 as i32)) as WORD;
    pd.hDevMode = devMode;
    pd.hDevNames = devNames;

    SendMessageW(
        hEditorWnd,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(from) as WPARAM,
        core::ptr::addr_of_mut!(to) as LPARAM,
    );
    if from == to {
        pd.Flags |= PD_NOSELECTION as DWORD;
    }

    if PrintDlgW(core::ptr::addr_of_mut!(pd)) != 0 {
        devMode = pd.hDevMode;
        devNames = pd.hDevNames;
        print(core::ptr::addr_of_mut!(pd), wszFileName);
        update_ruler(get_ruler_wnd(hMainWnd));
    }
}

unsafe extern "C" fn preview_bar_show(hMainWnd: HWND, show: BOOL) {
    let mut hReBar: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32));
    let mut i: ::core::ffi::c_int;

    if show != 0 {
        let mut rb: REBARBANDINFOW = core::mem::zeroed();
        let mut hStatic: HWND;
        let mut num_pages_string: UINT = (if preview.pages_shown > 1 {
            STRING_PREVIEW_ONEPAGE
        } else {
            STRING_PREVIEW_TWOPAGES
        }) as UINT;

        AddTextButton(hReBar, STRING_PREVIEW_PRINT as UINT, ID_PRINT as UINT, BANDID_PREVIEW_BTN1 as UINT);
        AddTextButton(hReBar, STRING_PREVIEW_NEXTPAGE as UINT, ID_PREVIEW_NEXTPAGE as UINT, BANDID_PREVIEW_BTN2 as UINT);
        AddTextButton(hReBar, STRING_PREVIEW_PREVPAGE as UINT, ID_PREVIEW_PREVPAGE as UINT, BANDID_PREVIEW_BTN3 as UINT);
        AddTextButton(hReBar, num_pages_string, ID_PREVIEW_NUMPAGES as UINT, BANDID_PREVIEW_BTN4 as UINT);
        AddTextButton(hReBar, STRING_PREVIEW_ZOOMIN as UINT, ID_PREVIEW_ZOOMIN as UINT, BANDID_PREVIEW_BTN5 as UINT);
        AddTextButton(hReBar, STRING_PREVIEW_ZOOMOUT as UINT, ID_PREVIEW_ZOOMOUT as UINT, BANDID_PREVIEW_BTN6 as UINT);
        AddTextButton(hReBar, STRING_PREVIEW_CLOSE as UINT, ID_FILE_EXIT as UINT, BANDID_PREVIEW_BTN7 as UINT);

        hStatic = CreateWindowW!(
            WC_STATICW,
            core::ptr::null(),
            (WS_VISIBLE | WS_CHILD) as DWORD,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            hReBar,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        rb.cbSize = REBARBANDINFOW_V6_SIZE as UINT;
        rb.fMask = (RBBIM_SIZE | RBBIM_CHILDSIZE | RBBIM_STYLE | RBBIM_CHILD | RBBIM_IDEALSIZE | RBBIM_ID) as UINT;
        rb.fStyle = (RBBS_NOGRIPPER | RBBS_VARIABLEHEIGHT) as UINT;
        rb.hwndChild = hStatic;
        rb.cyMinChild = 22 as UINT;
        rb.cyChild = rb.cyMinChild;
        rb.cxMinChild = 90 as UINT;
        rb.cx = rb.cxMinChild;
        rb.cxIdeal = 100 as UINT;
        rb.wID = BANDID_PREVIEW_BUFFER as UINT;

        SendMessageW(
            hReBar,
            RB_INSERTBANDW as UINT,
            (-(1 as ::core::ffi::c_int)) as WPARAM,
            core::ptr::addr_of_mut!(rb) as LPARAM,
        );
    } else {
        i = 0 as ::core::ffi::c_int;
        while i <= PREVIEW_BUTTONS as ::core::ffi::c_int {
            SendMessageW(
                hReBar,
                RB_DELETEBAND as UINT,
                SendMessageW(
                    hReBar,
                    RB_IDTOINDEX as UINT,
                    (BANDID_PREVIEW_BTN1 + (i) as u32) as WPARAM,
                    0 as LPARAM,
                ) as WPARAM,
                0 as LPARAM,
            );
            i += 1;
        }
    }
}

unsafe extern "C" fn update_preview_scrollbars(hwndPreview: HWND, window: *mut RECT) {
    let mut sbi: SCROLLINFO = core::mem::zeroed();
    sbi.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    sbi.fMask = SIF_PAGE | SIF_RANGE;
    sbi.nMin = 0;
    if preview.zoomlevel == 0 {
        /* Hide scrollbars when zoomed out. */
        sbi.nMax = 0;
        sbi.nPage = (*window).right as UINT;
        SetScrollInfo(hwndPreview, (SB_HORZ as i32), &mut sbi, (TRUE) as BOOL);
        sbi.nPage = (*window).bottom as UINT;
        SetScrollInfo(hwndPreview, (SB_VERT as i32), &mut sbi, (TRUE) as BOOL);
    } else {
        sbi.nMax = preview.bmScaledSize.cx * preview.pages_shown
            + min_spacing * (preview.pages_shown + 1);
        sbi.nPage = (*window).right as UINT;
        SetScrollInfo(hwndPreview, (SB_HORZ as i32), &mut sbi, (TRUE) as BOOL);
        /* Change in the horizontal scrollbar visibility affects the
         * client rect, so update the client rect. */
        GetClientRect(hwndPreview, window);
        sbi.nMax = preview.bmScaledSize.cy + min_spacing * 2;
        sbi.nPage = (*window).bottom as UINT;
        SetScrollInfo(hwndPreview, (SB_VERT as i32), &mut sbi, (TRUE) as BOOL);
    }
}

unsafe extern "C" fn update_preview_sizes(hwndPreview: HWND, zoomLevelUpdated: BOOL) {
    let mut window: RECT = core::mem::zeroed();

    GetClientRect(hwndPreview, &mut window);

    /* The zoom ratio isn't updated for partial zoom because of resizing the window. */
    if zoomLevelUpdated != 0 || preview.zoomlevel != 1 {
        let mut ratio: f32 = 0.0;
        let mut ratioHeight: f32 = 0.0;
        let mut ratioWidth: f32 = 0.0;
        if preview.zoomlevel == 2 {
            ratio = 1.0;
        } else {
            ratioHeight = (window.bottom - min_spacing * 2) as f32
                / preview.bmSize.cy as f32;

            ratioWidth = (window.right
                - min_spacing * (preview.pages_shown + 1)) as f32
                / (preview.pages_shown * preview.bmSize.cx) as f32;

            if ratioWidth > ratioHeight {
                ratio = ratioHeight;
            } else {
                ratio = ratioWidth;
            }

            if preview.zoomlevel == 1 {
                ratio += (1.0 - ratio)
                    / 2 as f32;
            }
        }
        preview.zoomratio = ratio;
    }

    preview.bmScaledSize.cx = (preview.bmSize.cx as f32 * preview.zoomratio) as LONG;
    preview.bmScaledSize.cy = (preview.bmSize.cy as f32 * preview.zoomratio) as LONG;

    preview.spacing.cy = max!(
        min_spacing,
        (window.bottom - preview.bmScaledSize.cy) / 2,
    );

    preview.spacing.cx = (window.right - preview.bmScaledSize.cx * preview.pages_shown)
        / (preview.pages_shown + 1);
    if preview.spacing.cx < min_spacing {
        preview.spacing.cx = min_spacing;
    }

    update_preview_scrollbars(hwndPreview, &mut window);
}

unsafe extern "C" fn draw_margin_lines(hdc: HDC, x: i32, y: i32, ratio: f32) {
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut oldPen: HPEN = core::ptr::null_mut();
    let mut dpi: SIZE = SIZE { cx: 0, cy: 0 };
    let mut page_margin: RECT = preview.rcPage;

    dpi.cx = GetDeviceCaps(hdc, (LOGPIXELSX as i32));
    dpi.cy = GetDeviceCaps(hdc, (LOGPIXELSY as i32));

    SetRect(core::ptr::addr_of_mut!(page_margin), preview.rcPage.left + margins.left, preview.rcPage.top + margins.top,
            preview.rcPage.right - margins.right, preview.rcPage.bottom - margins.bottom);

    page_margin.left = ((twips_to_pixels(page_margin.left, dpi.cx) as f32) * ratio) as i32;
    page_margin.top = ((twips_to_pixels(page_margin.top, dpi.cy) as f32) * ratio) as i32;
    page_margin.bottom = ((twips_to_pixels(page_margin.bottom, dpi.cy) as f32) * ratio) as i32;
    page_margin.right = ((twips_to_pixels(page_margin.right, dpi.cx) as f32) * ratio) as i32;

    OffsetRect(core::ptr::addr_of_mut!(page_margin), x, y);

    hPen = CreatePen((PS_DOT as i32), 1, RGB!(0, 0, 0));
    oldPen = (SelectObject(hdc, (hPen).cast())).cast();

    MoveToEx(hdc, x, page_margin.top, core::ptr::null_mut());
    LineTo(hdc, x + preview.bmScaledSize.cx, page_margin.top);
    MoveToEx(hdc, x, page_margin.bottom, core::ptr::null_mut());
    LineTo(hdc, x + preview.bmScaledSize.cx, page_margin.bottom);

    MoveToEx(hdc, page_margin.left, y, core::ptr::null_mut());
    LineTo(hdc, page_margin.left, y + preview.bmScaledSize.cy);
    MoveToEx(hdc, page_margin.right, y, core::ptr::null_mut());
    LineTo(hdc, page_margin.right, y + preview.bmScaledSize.cy);

    SelectObject(hdc, (oldPen).cast());
    DeleteObject((hPen).cast());
}

unsafe extern "C" fn is_last_preview_page(mut page: ::core::ffi::c_int) -> BOOL {
    return (*preview
        .pageEnds
        .offset((page - 1) as isize)
        >= preview.textlength) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn init_preview(hMainWnd: HWND, wszFileName: LPWSTR) {
    let hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
    preview.page = 1;
    preview.hdc = core::ptr::null_mut();
    preview.hdc2 = core::ptr::null_mut();
    preview.wszFileName = wszFileName;
    preview.zoomratio = 0.0;
    preview.zoomlevel = 0;
    preview_bar_show(hMainWnd, (TRUE) as BOOL);

    CreateWindowExW(
        0,
        core::ptr::addr_of!(wszPreviewWndClass) as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        (WS_VISIBLE | WS_CHILD | WS_VSCROLL | WS_HSCROLL) as DWORD,
        0,
        0,
        200,
        10,
        hMainWnd,
        IDC_PREVIEW as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn close_preview(hMainWnd: HWND) {
    let hwndPreview: HWND = GetDlgItem(hMainWnd, (IDC_PREVIEW as i32));
    preview.window.right = 0;
    preview.window.bottom = 0;
    preview.page = 0;
    HeapFree(GetProcessHeap(), 0, preview.pageEnds as LPVOID);
    preview.pageEnds = core::ptr::null_mut();
    preview.pageCapacity = 0;
    if preview.zoomlevel > 0 {
        preview.pages_shown = preview.saved_pages_shown;
    }
    if !preview.hdc.is_null() {
        let oldbm: HBITMAP = GetCurrentObject(preview.hdc, OBJ_BITMAP) as HBITMAP;
        DeleteDC(preview.hdc);
        DeleteObject(oldbm as HGDIOBJ);
        preview.hdc = core::ptr::null_mut();
    }
    if !preview.hdc2.is_null() {
        let oldbm: HBITMAP = GetCurrentObject(preview.hdc2, OBJ_BITMAP) as HBITMAP;
        DeleteDC(preview.hdc2);
        DeleteObject(oldbm as HGDIOBJ);
        preview.hdc2 = core::ptr::null_mut();
    }

    preview_bar_show(hMainWnd, (FALSE) as BOOL);
    DestroyWindow(hwndPreview);
}
#[no_mangle]
pub unsafe extern "C" fn preview_isactive() -> BOOL {
    return (preview.page != 0) as BOOL;
}

unsafe extern "C" fn draw_preview(
    hEditorWnd: HWND,
    lpFr: *mut FORMATRANGE,
    paper: *mut RECT,
    page: ::core::ffi::c_int,
) {
    let mut bottom: ::core::ffi::c_int = 0;
    if preview.pageEnds.is_null() {
        preview.pageCapacity = 32 as ::core::ffi::c_int;
        preview.pageEnds = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (::core::mem::size_of::<::core::ffi::c_int>() as SIZE_T)
                .wrapping_mul(preview.pageCapacity as SIZE_T),
        ) as *mut ::core::ffi::c_int;
        if preview.pageEnds.is_null() {
            return;
        }
    } else if page >= preview.pageCapacity {
        let mut new_buffer: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
        new_buffer = HeapReAlloc(
            GetProcessHeap(),
            0 as DWORD,
            preview.pageEnds as PVOID,
            (::core::mem::size_of::<::core::ffi::c_int>() as SIZE_T)
                .wrapping_mul(preview.pageCapacity as SIZE_T)
                .wrapping_mul(2 as SIZE_T),
        ) as *mut ::core::ffi::c_int;
        if new_buffer.is_null() {
            return;
        }
        preview.pageCapacity *= 2 as ::core::ffi::c_int;
        preview.pageEnds = new_buffer;
    }

    FillRect(
        (*lpFr).hdc,
        paper as LPCRECT,
        GetStockObject((WHITE_BRUSH as i32)) as HBRUSH,
    );
    if page > 1 as ::core::ffi::c_int
        && is_last_preview_page(page - 1 as ::core::ffi::c_int) != 0
    {
        return;
    }
    (*lpFr).chrg.cpMin = (if page <= 1 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        *preview
            .pageEnds
            .offset((page - 2 as ::core::ffi::c_int) as isize)
    }) as LONG;
    bottom = (*lpFr).rc.bottom as ::core::ffi::c_int;
    *preview
        .pageEnds
        .offset((page - 1 as ::core::ffi::c_int) as isize) = SendMessageW(
        hEditorWnd,
        EM_FORMATRANGE as UINT,
        TRUE as WPARAM,
        lpFr as LPARAM,
    ) as ::core::ffi::c_int;

    /* EM_FORMATRANGE sets fr.rc.bottom to indicate the area printed in,
     * but we want to keep the original for drawing margins */
    (*lpFr).rc.bottom = bottom as LONG;
    SendMessageW(
        hEditorWnd,
        EM_FORMATRANGE as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );
}

unsafe extern "C" fn update_preview_buttons(hMainWnd: HWND) {
    let hReBar: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32));
    EnableWindow(
        GetDlgItem(hReBar, (ID_PREVIEW_PREVPAGE as i32)),
        (preview.page > 1) as i32,
    );
    EnableWindow(
        GetDlgItem(hReBar, (ID_PREVIEW_NEXTPAGE as i32)),
        (is_last_preview_page(preview.page) == 0
            && is_last_preview_page(preview.page + preview.pages_shown - 1) == 0) as i32,
    );
    EnableWindow(
        GetDlgItem(hReBar, (ID_PREVIEW_NUMPAGES as i32)),
        (preview.pages_shown > 1
            || (is_last_preview_page(1) == 0
                && preview.zoomlevel == 0)) as i32,
    );
    EnableWindow(
        GetDlgItem(hReBar, (ID_PREVIEW_ZOOMIN as i32)),
        (preview.zoomlevel < 2) as i32,
    );
    EnableWindow(
        GetDlgItem(hReBar, (ID_PREVIEW_ZOOMOUT as i32)),
        (preview.zoomlevel > 0) as i32,
    );
}

unsafe extern "C" fn print_preview(hwndPreview: HWND) -> LRESULT {
    let mut hPen: HPEN = ::core::ptr::null_mut();
    let mut oldPen: HPEN = ::core::ptr::null_mut();
    let mut hdc: HDC = ::core::ptr::null_mut();
    let mut back_rgn: HRGN = ::core::ptr::null_mut();
    let mut excl_rgn: HRGN = ::core::ptr::null_mut();
    let mut window: RECT = core::mem::zeroed();
    let mut background: RECT = core::mem::zeroed();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut x: i32 = 0;
    let mut y: i32 = 0;

    hdc = BeginPaint(hwndPreview, core::ptr::addr_of_mut!(ps));
    GetClientRect(hwndPreview, core::ptr::addr_of_mut!(window));
    back_rgn = CreateRectRgnIndirect(core::ptr::addr_of_mut!(window));

    x = preview.spacing.cx - GetScrollPos(hwndPreview, (SB_HORZ as i32));
    y = preview.spacing.cy - GetScrollPos(hwndPreview, (SB_VERT as i32));

    /* draw page outlines */
    hPen = CreatePen((PS_SOLID | PS_INSIDEFRAME) as i32, 2, RGB!(0, 0, 0));
    oldPen = (SelectObject(hdc, (hPen).cast())).cast();
    SetRect(core::ptr::addr_of_mut!(background), x - 2, y - 2, x + preview.bmScaledSize.cx + 2,
            y + preview.bmScaledSize.cy + 2);
    Rectangle(hdc, background.left, background.top,
              background.right, background.bottom);
    excl_rgn = CreateRectRgnIndirect(core::ptr::addr_of_mut!(background));
    CombineRgn(back_rgn, back_rgn, excl_rgn, (RGN_DIFF as i32));
    if preview.pages_shown > 1 {
        background.left += preview.bmScaledSize.cx + preview.spacing.cx;
        background.right += preview.bmScaledSize.cx + preview.spacing.cx;
        Rectangle(hdc, background.left, background.top,
                  background.right, background.bottom);
        SetRectRgn(excl_rgn, background.left, background.top,
                   background.right, background.bottom);
        CombineRgn(back_rgn, back_rgn, excl_rgn, (RGN_DIFF as i32));
    }
    SelectObject(hdc, (oldPen).cast());
    DeleteObject((hPen).cast());
    FillRgn(hdc, back_rgn, (GetStockObject((GRAY_BRUSH as i32))).cast());
    DeleteObject((excl_rgn).cast());
    DeleteObject((back_rgn).cast());

    StretchBlt(hdc, x, y, preview.bmScaledSize.cx, preview.bmScaledSize.cy,
               preview.hdc, 0, 0, preview.bmSize.cx, preview.bmSize.cy, SRCCOPY);

    draw_margin_lines(hdc, x, y, preview.zoomratio);

    if preview.pages_shown > 1 {
        if is_last_preview_page(preview.page) == 0 {
            x += preview.spacing.cx + preview.bmScaledSize.cx;
            StretchBlt(hdc, x, y,
                       preview.bmScaledSize.cx, preview.bmScaledSize.cy,
                       preview.hdc2, 0, 0,
                       preview.bmSize.cx, preview.bmSize.cy, SRCCOPY);

            draw_margin_lines(hdc, x, y, preview.zoomratio);
        } else {
            InflateRect(core::ptr::addr_of_mut!(background), -2, -2);
            FillRect(hdc, core::ptr::addr_of_mut!(background), (GetStockObject((WHITE_BRUSH as i32))).cast());
        }
    }

    preview.window = window;

    EndPaint(hwndPreview, core::ptr::addr_of_mut!(ps));

    return 0;
}

unsafe extern "C" fn update_preview_statusbar(hMainWnd: HWND) {
    let hStatusbar: HWND = GetDlgItem(hMainWnd, (IDC_STATUSBAR as i32));
    let hInst: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
    let mut p: *mut WCHAR;
    let mut wstr: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];

    p = core::ptr::addr_of_mut!(wstr) as *mut WCHAR;
    if preview.pages_shown < 2 || is_last_preview_page(preview.page) != 0 {
        static fmt: [WCHAR; 4] = [0x0020u16, 0x0025, 0x0064, 0x0000];
        p = p.offset(LoadStringW(
            hInst,
            STRING_PREVIEW_PAGE,
            core::ptr::addr_of_mut!(wstr) as LPWSTR,
            (MAX_STRING_LEN as i32),
        ) as isize);
        wsprintfW(p as LPWSTR, core::ptr::addr_of!(fmt) as LPCWSTR, preview.page);
    } else {
        static fmt_0: [WCHAR; 7] = [0x0020u16, 0x0025, 0x0064, 0x002D, 0x0025, 0x0064, 0x0000];
        p = p.offset(LoadStringW(
            hInst,
            STRING_PREVIEW_PAGES,
            core::ptr::addr_of_mut!(wstr) as LPWSTR,
            (MAX_STRING_LEN as i32),
        ) as isize);
        wsprintfW(
            p as LPWSTR,
            core::ptr::addr_of!(fmt_0) as LPCWSTR,
            preview.page,
            preview.page + 1,
        );
    }
    SetWindowTextW(hStatusbar, core::ptr::addr_of_mut!(wstr) as *mut WCHAR as LPCWSTR);
}

unsafe extern "C" fn update_preview(hMainWnd: HWND) {
    let mut paper: RECT = core::mem::zeroed();
    let hEditorWnd: HWND = GetDlgItem(hMainWnd, (IDC_EDITOR as i32));
    let hwndPreview: HWND = GetDlgItem(hMainWnd, (IDC_PREVIEW as i32));
    let mut hBitmapCapture: HBITMAP = core::ptr::null_mut();
    let mut fr: FORMATRANGE = core::mem::zeroed();
    let hdc: HDC = GetDC(hwndPreview);

    fr.hdcTarget = make_dc();
    fr.rcPage = preview.rcPage;
    fr.rc = fr.rcPage;
    fr.rc.left += margins.left;
    fr.rc.top += margins.top;
    fr.rc.bottom -= margins.bottom;
    fr.rc.right -= margins.right;

    fr.chrg.cpMin = 0 as LONG;
    fr.chrg.cpMax = preview.textlength as LONG;

    SetRect(
        core::ptr::addr_of_mut!(paper),
        0,
        0,
        preview.bmSize.cx,
        preview.bmSize.cy,
    );

    if preview.hdc.is_null() {
        preview.hdc = CreateCompatibleDC(hdc);
        hBitmapCapture = CreateCompatibleBitmap(hdc, preview.bmSize.cx, preview.bmSize.cy);
        SelectObject(preview.hdc, hBitmapCapture as *mut core::ffi::c_void);
    }

    fr.hdc = preview.hdc;
    draw_preview(
        hEditorWnd,
        core::ptr::addr_of_mut!(fr),
        core::ptr::addr_of_mut!(paper),
        preview.page,
    );

    if preview.pages_shown > 1 {
        if preview.hdc2.is_null() {
            preview.hdc2 = CreateCompatibleDC(hdc);
            hBitmapCapture = CreateCompatibleBitmap(
                hdc,
                preview.bmSize.cx,
                preview.bmSize.cy,
            );
            SelectObject(preview.hdc2, hBitmapCapture as *mut core::ffi::c_void);
        }

        fr.hdc = preview.hdc2;
        draw_preview(
            hEditorWnd,
            core::ptr::addr_of_mut!(fr),
            core::ptr::addr_of_mut!(fr.rcPage),
            preview.page + 1,
        );
    }
    DeleteDC(fr.hdcTarget);
    ReleaseDC(hwndPreview, hdc);

    InvalidateRect(hwndPreview, core::ptr::null::<RECT>(), (FALSE) as BOOL);
    update_preview_buttons(hMainWnd);
    update_preview_statusbar(hMainWnd);
}

unsafe extern "C" fn toggle_num_pages(hMainWnd: HWND) {
    let hReBar: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32));
    let mut name: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let hInst: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
    let mut nPreviewPages: ::core::ffi::c_int;

    preview.pages_shown = if preview.pages_shown > 1 {
        1
    } else {
        2
    };

    nPreviewPages = if preview.zoomlevel > 0 {
        preview.saved_pages_shown
    } else {
        preview.pages_shown
    };

    LoadStringW(
        hInst,
        (if nPreviewPages > 1 {
            STRING_PREVIEW_ONEPAGE
        } else {
            STRING_PREVIEW_TWOPAGES
        }) as UINT,
        core::ptr::addr_of_mut!(name) as LPWSTR,
        (MAX_STRING_LEN as i32),
    );

    SetWindowTextW(
        GetDlgItem(hReBar, (ID_PREVIEW_NUMPAGES as i32)),
        core::ptr::addr_of_mut!(name) as *mut WCHAR as LPCWSTR,
    );
    update_preview_sizes(
        GetDlgItem(hMainWnd, (IDC_PREVIEW as i32)),
        (TRUE) as BOOL,
    );
    update_preview(hMainWnd);
}

unsafe extern "C" fn preview_page_hittest(pt: POINT) -> i32 {
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    rc.left = preview.spacing.cx;
    rc.right = rc.left + preview.bmScaledSize.cx;
    rc.top = preview.spacing.cy;
    rc.bottom = rc.top + preview.bmScaledSize.cy;
    if PtInRect(core::ptr::addr_of_mut!(rc) as LPCRECT, pt) != 0 {
        return 1;
    }

    if preview.pages_shown <= 1 {
        return 0;
    }

    rc.left += preview.bmScaledSize.cx + preview.spacing.cx;
    rc.right += preview.bmScaledSize.cx + preview.spacing.cx;
    if PtInRect(core::ptr::addr_of_mut!(rc) as LPCRECT, pt) != 0 {
        return if is_last_preview_page(preview.page) != 0 {
            1
        } else {
            2
        };
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn preview_proc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let mut hMainWnd: HWND = GetParent(hWnd);
            let mut hEditorWnd: HWND = GetDlgItem(hMainWnd, (IDC_EDITOR as i32));
            let mut fr: FORMATRANGE = core::mem::zeroed();
            let mut gt: GETTEXTLENGTHEX = GETTEXTLENGTHEX {
                flags: GTL_DEFAULT as DWORD,
                codepage: 1200 as UINT,
            };
            let mut hdc: HDC = GetDC(hWnd);
            let mut hdcTarget: HDC = make_dc();

            preview.rcPage = get_print_rect(hdcTarget);
            fr.rc = preview.rcPage;
            preview.rcPage.bottom += margins.bottom;
            preview.rcPage.right += margins.right;
            preview.rcPage.left = 0 as LONG;
            preview.rcPage.top = preview.rcPage.left;
            fr.rcPage = preview.rcPage;

            preview.bmSize.cx = twips_to_pixels(
                preview.rcPage.right as ::core::ffi::c_int,
                GetDeviceCaps(hdc, (LOGPIXELSX as i32)),
            );
            preview.bmSize.cy = twips_to_pixels(
                preview.rcPage.bottom as ::core::ffi::c_int,
                GetDeviceCaps(hdc, (LOGPIXELSY as i32)),
            );

            preview.textlength = SendMessageW(
                hEditorWnd,
                EM_GETTEXTLENGTHEX as UINT,
                core::ptr::addr_of_mut!(gt) as WPARAM,
                0 as LPARAM,
            ) as ::core::ffi::c_int;

            fr.hdc = CreateCompatibleDC(hdc);
            fr.hdcTarget = hdcTarget;
            fr.chrg.cpMin = 0 as LONG;
            fr.chrg.cpMax = preview.textlength as LONG;
            DeleteDC(fr.hdc);
            DeleteDC(hdcTarget);
            ReleaseDC(hWnd, hdc);

            update_preview_sizes(hWnd, (TRUE) as BOOL);
            update_preview(hMainWnd);
        }

        WM_PAINT => {
            return print_preview(hWnd);
        }

        WM_SIZE => {
            update_preview_sizes(hWnd, (FALSE) as BOOL);
            InvalidateRect(hWnd, core::ptr::null_mut(), (FALSE) as BOOL);
        }

        WM_VSCROLL | WM_HSCROLL => {
            let mut si: SCROLLINFO = core::mem::zeroed();
            let mut rc: RECT = core::mem::zeroed();
            let mut nBar: ::core::ffi::c_int = if msg == WM_VSCROLL {
                (SB_VERT as i32)
            } else {
                (SB_HORZ as i32)
            };
            let mut origPos: ::core::ffi::c_int;

            GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
            si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
            si.fMask = SIF_ALL as UINT;
            GetScrollInfo(hWnd, nBar, core::ptr::addr_of_mut!(si));
            origPos = si.nPos;
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                SB_TOP => {
                    si.nPos = si.nMin;
                }
                SB_BOTTOM => {
                    si.nPos = si.nMax;
                }
                SB_LINEUP => {
                    si.nPos = (si.nPos as UINT).wrapping_sub(si.nPage / (10 as UINT)) as ::core::ffi::c_int;
                }
                SB_LINEDOWN => {
                    si.nPos = (si.nPos as UINT).wrapping_add(si.nPage / (10 as UINT)) as ::core::ffi::c_int;
                }
                SB_PAGEUP => {
                    si.nPos = (si.nPos as UINT).wrapping_sub(si.nPage) as ::core::ffi::c_int;
                }
                SB_PAGEDOWN => {
                    si.nPos = (si.nPos as UINT).wrapping_add(si.nPage) as ::core::ffi::c_int;
                }
                SB_THUMBTRACK => {
                    si.nPos = si.nTrackPos;
                }
                _ => {}
            }
            si.fMask = SIF_POS as UINT;
            SetScrollInfo(
                hWnd,
                nBar,
                core::ptr::addr_of_mut!(si) as LPCSCROLLINFO,
                (TRUE) as BOOL,
            );
            GetScrollInfo(hWnd, nBar, core::ptr::addr_of_mut!(si));
            if si.nPos != origPos {
                let mut amount: ::core::ffi::c_int = origPos - si.nPos;
                if msg == WM_VSCROLL {
                    ScrollWindow(
                        hWnd,
                        0 as ::core::ffi::c_int,
                        amount,
                        core::ptr::null(),
                        core::ptr::null(),
                    );
                } else {
                    ScrollWindow(
                        hWnd,
                        amount,
                        0 as ::core::ffi::c_int,
                        core::ptr::null(),
                        core::ptr::null(),
                    );
                }
            }
            return 0 as LRESULT;
        }

        WM_SETCURSOR => {
            let mut pt: POINT = core::mem::zeroed();
            let mut rc: RECT = core::mem::zeroed();
            let mut bHittest: ::core::ffi::c_int = 0;
            let mut messagePos: DWORD = GetMessagePos();
            pt.x = LOWORD!(messagePos) as i16 as LONG;
            pt.y = HIWORD!(messagePos) as i16 as LONG;
            ScreenToClient(hWnd, core::ptr::addr_of_mut!(pt));

            GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
            if PtInRect(core::ptr::addr_of_mut!(rc), pt) != 0 {
                pt.x += GetScrollPos(hWnd, (SB_HORZ as i32)) as LONG;
                pt.y += GetScrollPos(hWnd, (SB_VERT as i32)) as LONG;
                bHittest = preview_page_hittest(pt);
            }

            if bHittest != 0 {
                SetCursor(LoadCursorW(
                    GetModuleHandleW(0 as *const WCHAR) as HINSTANCE,
                    MAKEINTRESOURCEW!(IDC_ZOOM) as LPCWSTR,
                ));
            } else {
                SetCursor(LoadCursorW(
                    core::ptr::null_mut(),
                    IDC_ARROW as *mut WCHAR as LPCWSTR,
                ));
            }

            return TRUE as LRESULT;
        }

        WM_LBUTTONDOWN => {
            let mut page: ::core::ffi::c_int;
            let mut pt: POINT = core::mem::zeroed();
            pt.x = (LOWORD!(lParam) as i16 as ::core::ffi::c_int
                + GetScrollPos(hWnd, (SB_HORZ as i32))) as LONG;
            pt.y = (HIWORD!(lParam) as i16 as ::core::ffi::c_int
                + GetScrollPos(hWnd, (SB_VERT as i32))) as LONG;
            page = preview_page_hittest(pt);
            if page > 0 {
                let mut hMainWnd: HWND = GetParent(hWnd);

                /* Convert point from client coordinate to unzoomed page
                 * coordinate. */
                pt.x -= preview.spacing.cx;
                if page > 1 {
                    pt.x -= preview.bmScaledSize.cx + preview.spacing.cx;
                }
                pt.y -= preview.spacing.cy;
                pt.x = (pt.x as ::core::ffi::c_float / preview.zoomratio) as LONG;
                pt.y = (pt.y as ::core::ffi::c_float / preview.zoomratio) as LONG;

                if preview.zoomlevel == 0 {
                    preview.saved_pages_shown = preview.pages_shown;
                }
                preview.zoomlevel =
                    (preview.zoomlevel + 1) % 3;
                preview.zoomratio = 0 as ::core::ffi::c_float;
                if preview.zoomlevel == 0
                    && preview.saved_pages_shown > 1
                {
                    toggle_num_pages(hMainWnd);
                } else if preview.pages_shown > 1 {
                    if page >= 2 {
                        preview.page += 1;
                    }
                    toggle_num_pages(hMainWnd);
                } else {
                    update_preview_sizes(hWnd, (TRUE) as BOOL);
                    InvalidateRect(hWnd, core::ptr::null_mut(), (FALSE) as BOOL);
                    update_preview_buttons(hMainWnd);
                }

                if preview.zoomlevel > 0 {
                    let mut si: SCROLLINFO = core::mem::zeroed();
                    /* Convert the coordinate back to client coordinate. */
                    pt.x = (pt.x as ::core::ffi::c_float * preview.zoomratio) as LONG;
                    pt.y = (pt.y as ::core::ffi::c_float * preview.zoomratio) as LONG;
                    pt.x += preview.spacing.cx;
                    pt.y += preview.spacing.cy;
                    /* Scroll to center view at that point on the page */
                    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
                    si.fMask = SIF_PAGE as UINT;
                    GetScrollInfo(
                        hWnd,
                        (SB_HORZ as i32),
                        core::ptr::addr_of_mut!(si),
                    );
                    pt.x -= (si.nPage / (2 as UINT)) as LONG;
                    SetScrollPos(
                        hWnd,
                        (SB_HORZ as i32),
                        pt.x,
                        (TRUE) as BOOL,
                    );
                    GetScrollInfo(
                        hWnd,
                        (SB_VERT as i32),
                        core::ptr::addr_of_mut!(si),
                    );
                    pt.y -= (si.nPage / (2 as UINT)) as LONG;
                    SetScrollPos(
                        hWnd,
                        (SB_VERT as i32),
                        pt.y,
                        (TRUE) as BOOL,
                    );
                }
            }
            return DefWindowProcW(hWnd, msg, wParam, lParam);
        }

        _ => {
            return DefWindowProcW(hWnd, msg, wParam, lParam);
        }
    }

    return 0 as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn preview_command(hWnd: HWND, wParam: WPARAM) -> LRESULT {
    match (LOWORD!(wParam) as i32) as u32 {
        ID_FILE_EXIT => {
            PostMessageW(hWnd, WM_CLOSE, 0, 0);
        }

        ID_PREVIEW_NEXTPAGE | ID_PREVIEW_PREVPAGE => {
            if (LOWORD!(wParam) as i32) == (ID_PREVIEW_NEXTPAGE as i32) {
                preview.page += 1;
            } else {
                preview.page -= 1;
            }

            update_preview(hWnd);
        }

        ID_PREVIEW_NUMPAGES => {
            toggle_num_pages(hWnd);
        }

        ID_PREVIEW_ZOOMIN => {
            if preview.zoomlevel < 2 {
                if preview.zoomlevel == 0 {
                    preview.saved_pages_shown = preview.pages_shown;
                }
                preview.zoomlevel += 1;
                preview.zoomratio = 0 as f32;
                if preview.pages_shown > 1 {
                    /* Forced switch to one page when zooming in. */
                    toggle_num_pages(hWnd);
                } else {
                    let mut hwndPreview: HWND = GetDlgItem(hWnd, (IDC_PREVIEW as i32));
                    update_preview_sizes(hwndPreview, (TRUE) as BOOL);
                    InvalidateRect(hwndPreview, core::ptr::null::<RECT>(), (FALSE) as BOOL);
                    update_preview_buttons(hWnd);
                }
            }
        }

        ID_PREVIEW_ZOOMOUT => {
            if preview.zoomlevel > 0 {
                let mut hwndPreview: HWND = GetDlgItem(hWnd, (IDC_PREVIEW as i32));
                preview.zoomlevel -= 1;
                preview.zoomratio = 0 as f32;
                if preview.zoomlevel == 0 && preview.saved_pages_shown > 1 {
                    toggle_num_pages(hWnd);
                } else {
                    update_preview_sizes(hwndPreview, (TRUE) as BOOL);
                    InvalidateRect(hwndPreview, core::ptr::null::<RECT>(), (FALSE) as BOOL);
                    update_preview_buttons(hWnd);
                }
            }
        }

        ID_PRINT => {
            dialog_print(hWnd, preview.wszFileName);
            SendMessageW(hWnd, WM_CLOSE, 0, 0);
        }

        _ => {}
    }

    return 0;
}
