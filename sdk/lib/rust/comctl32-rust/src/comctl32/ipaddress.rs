use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_ipaddress);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_IPADDRESSW: [::core::ffi::c_ushort; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_ushort; 15]>(
        *b"S\0y\0s\0I\0P\0A\0d\0d\0r\0e\0s\0s\x003\x002\0\0\0",
    )
};

pub const WC_EDITW: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"E\0d\0i\0t\0\0\0") };

static mut IP_SUBCLASS_PROP: [WCHAR; 19] = [
    'C' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'P' as i32 as WCHAR,
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

#[repr(C)]
pub struct IPPART_INFO {
    pub EditHwnd: HWND,
    pub LowerLimit: INT,
    pub UpperLimit: INT,
    pub OrigProc: WNDPROC,
}

#[repr(C)]
pub struct IPADDRESS_INFO {
    pub Self_: HWND,
    pub Notify: HWND,
    pub Enabled: BOOL,
    pub Part: [IPPART_INFO; 4],
}

pub const POS_DEFAULT: i32 = 0;
pub const POS_LEFT: i32 = 1;
pub const POS_RIGHT: i32 = 2;
pub const POS_SELALL: i32 = 3;

unsafe extern "C" fn IPADDRESS_UpdateText(infoPtr: *const IPADDRESS_INFO) {
    static zero: [u16; 2] = [0x0030u16, 0x0000];
    static dot: [u16; 2] = [0x002Eu16, 0x0000];
    let mut field: [WCHAR; 4] = [0; 4];
    let mut ip: [WCHAR; 16] = [0; 16];
    let mut i: INT;

    ip[0] = 0;

    i = 0;
    while i < 4 {
        if GetWindowTextW(
            (*infoPtr).Part[i as usize].EditHwnd,
            core::ptr::addr_of_mut!(field) as LPWSTR,
            4,
        ) != 0
        {
            lstrcatW(
                core::ptr::addr_of_mut!(ip) as LPWSTR,
                core::ptr::addr_of_mut!(field) as *mut WCHAR as LPCWSTR,
            );
        } else {
            /* empty edit treated as zero */
            lstrcatW(
                core::ptr::addr_of_mut!(ip) as LPWSTR,
                core::ptr::addr_of!(zero) as *const WCHAR as LPCWSTR,
            );
        }
        if i != 3 {
            lstrcatW(
                core::ptr::addr_of_mut!(ip) as LPWSTR,
                core::ptr::addr_of!(dot) as *const WCHAR as LPCWSTR,
            );
        }
        i += 1;
    }

    SetWindowTextW(
        (*infoPtr).Self_,
        core::ptr::addr_of_mut!(ip) as *mut WCHAR as LPCWSTR,
    );
}

unsafe extern "C" fn IPADDRESS_Notify(infoPtr: *const IPADDRESS_INFO, command: UINT) -> LRESULT {
    let hwnd: HWND = (*infoPtr).Self_;

    return SendMessageW(
        (*infoPtr).Notify,
        WM_COMMAND,
        MAKEWPARAM!(
            GetWindowLongPtrW(hwnd, GWLP_ID) as WORD,
            command as WORD,
        ) as WPARAM,
        hwnd as LPARAM,
    );
}

unsafe extern "C" fn IPADDRESS_IPNotify(infoPtr: *const IPADDRESS_INFO, field: INT, value: INT) -> INT {
    let mut nmip: NMIPADDRESS = core::mem::zeroed();

    nmip.hdr.hwndFrom = (*infoPtr).Self_;
    nmip.hdr.idFrom = GetWindowLongPtrW((*infoPtr).Self_, GWLP_ID) as UINT_PTR;
    nmip.hdr.code = IPN_FIELDCHANGED as UINT;

    nmip.iField = field;
    nmip.iValue = value;

    SendMessageW(
        (*infoPtr).Notify,
        WM_NOTIFY as UINT,
        nmip.hdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmip) as LPARAM,
    );

    return nmip.iValue as INT;
}

unsafe extern "C" fn IPADDRESS_GetPartIndex(infoPtr: *const IPADDRESS_INFO, hwnd: HWND) -> i32 {
    let mut i: i32;

    i = 0;
    while i < 4 {
        if (*infoPtr).Part[i as usize].EditHwnd == hwnd {
            return i;
        }
        i += 1;
    }
    return -1;
}

unsafe extern "C" fn IPADDRESS_Draw(infoPtr: *const IPADDRESS_INFO, hdc: HDC) -> LRESULT {
    static dotW: [u16; 2] = [0x002Eu16, 0x0000];
    let mut rect: RECT = core::mem::zeroed();
    let mut rcPart: RECT = core::mem::zeroed();
    let mut bgCol: COLORREF = 0;
    let mut fgCol: COLORREF = 0;
    let mut theme: HTHEME = core::ptr::null_mut();
    let mut i: i32 = 0;
    let mut state: i32 = (ETS_NORMAL) as i32;

    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/ipaddress.c\0"
                as *const u8 as *const i8,
            b"IPADDRESS_Draw\0" as *const u8 as *const i8,
            149 as i32,
            b"\n\0" as *const u8 as *const i8,
        );
    }

    GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(rect));

    theme = OpenThemeData((*infoPtr).Self_, (WC_EDITW).as_ptr());

    if !theme.is_null() {
        let mut dwStyle: DWORD = GetWindowLongW((*infoPtr).Self_, GWL_STYLE) as DWORD;

        if (*infoPtr).Enabled == 0 {
            state = (ETS_DISABLED) as i32;
        } else if dwStyle & ES_READONLY as DWORD != 0 {
            state = (ETS_READONLY) as i32;
        } else if GetFocus() == (*infoPtr).Self_ {
            state = (ETS_FOCUSED) as i32;
        }

        GetThemeColor(
            theme,
            (EP_EDITTEXT) as i32,
            state,
            (TMT_FILLCOLOR as i32),
            core::ptr::addr_of_mut!(bgCol),
        );
        GetThemeColor(
            theme,
            (EP_EDITTEXT) as i32,
            state,
            (TMT_TEXTCOLOR as i32),
            core::ptr::addr_of_mut!(fgCol),
        );

        if IsThemeBackgroundPartiallyTransparent(theme, (EP_EDITTEXT) as i32, state) != 0 {
            DrawThemeParentBackground((*infoPtr).Self_, hdc, core::ptr::addr_of_mut!(rect));
        }
        DrawThemeBackground(
            theme,
            hdc,
            (EP_EDITTEXT) as i32,
            state,
            core::ptr::addr_of_mut!(rect),
            core::ptr::null(),
        );
    } else {
        if (*infoPtr).Enabled != 0 {
            bgCol = comctl32_color.clrWindow;
            fgCol = comctl32_color.clrWindowText;
        } else {
            bgCol = comctl32_color.clr3dFace;
            fgCol = comctl32_color.clrGrayText;
        }

        let brush: HBRUSH = CreateSolidBrush(bgCol);
        FillRect(hdc, core::ptr::addr_of_mut!(rect) as LPCRECT, brush);
        DeleteObject(brush as HGDIOBJ);

        if false {
            FillRect(
                hdc,
                core::ptr::addr_of_mut!(rect) as LPCRECT,
                ((bgCol + 1) as DWORD_PTR) as HBRUSH,
            );
        }

        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rect),
            EDGE_SUNKEN as UINT,
            (BF_RECT | BF_ADJUST) as UINT,
        );
    }

    SetBkColor(hdc, bgCol);
    SetTextColor(hdc, fgCol);

    i = 0;
    while i < 3 {
        GetWindowRect(
            (*infoPtr).Part[i as usize].EditHwnd,
            core::ptr::addr_of_mut!(rcPart),
        );
        MapWindowPoints(
            core::ptr::null_mut(),
            (*infoPtr).Self_,
            core::ptr::addr_of_mut!(rcPart) as LPPOINT,
            2 as UINT,
        );
        rect.left = rcPart.right;
        GetWindowRect(
            (*infoPtr).Part[(i + 1) as usize].EditHwnd,
            core::ptr::addr_of_mut!(rcPart),
        );
        MapWindowPoints(
            core::ptr::null_mut(),
            (*infoPtr).Self_,
            core::ptr::addr_of_mut!(rcPart) as LPPOINT,
            2 as UINT,
        );
        rect.right = rcPart.left;

        if !theme.is_null() {
            DrawThemeText(
                theme,
                hdc,
                (EP_EDITTEXT) as i32,
                state,
                dotW.as_ptr() as LPCWSTR,
                1,
                (DT_SINGLELINE | DT_CENTER | DT_BOTTOM) as DWORD,
                0 as DWORD,
                core::ptr::addr_of_mut!(rect),
            );
        } else {
            DrawTextW(
                hdc,
                dotW.as_ptr() as LPCWSTR,
                1,
                core::ptr::addr_of_mut!(rect),
                (DT_SINGLELINE | DT_CENTER | DT_BOTTOM) as UINT,
            );
        }
        i += 1;
    }

    if !theme.is_null() {
        CloseThemeData(theme);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn IPADDRESS_Create(hwnd: HWND, lpCreate: *const CREATESTRUCTA) -> LRESULT {
    let mut infoPtr: *mut IPADDRESS_INFO = core::ptr::null_mut();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut edit: RECT = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;
    let mut fieldsize: ::core::ffi::c_int = 0;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hSysFont: HFONT = core::ptr::null_mut();
    let mut logFont: LOGFONTW = core::mem::zeroed();
    let mut logSysFont: LOGFONTW = core::mem::zeroed();

    SetWindowLongW(
        hwnd,
        GWL_STYLE,
        GetWindowLongW(hwnd, GWL_STYLE) & (!WS_BORDER) as i32,
    );

    infoPtr = heap_alloc_zero(core::mem::size_of::<IPADDRESS_INFO>() as SIZE_T)
        as *mut IPADDRESS_INFO;
    if infoPtr.is_null() {
        return -(1) as LRESULT;
    }
    SetWindowLongPtrW(
        hwnd,
        0,
        infoPtr as DWORD_PTR as LONG_PTR,
    );

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rcClient));

    fieldsize = (rcClient.right - rcClient.left) / 4;

    edit.top = rcClient.top + 2;
    edit.bottom = rcClient.bottom - 2;

    (*infoPtr).Self_ = hwnd;
    (*infoPtr).Enabled = TRUE as BOOL;
    (*infoPtr).Notify = (*lpCreate).hwndParent;

    hSysFont = GetStockObject((ANSI_VAR_FONT as i32)) as HFONT;
    GetObjectW(
        hSysFont as HANDLE,
        core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(logSysFont) as LPVOID,
    );
    SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT as UINT,
        0 as UINT,
        core::ptr::addr_of_mut!(logFont) as PVOID,
        0 as UINT,
    );
    lstrcpyW(
        core::ptr::addr_of_mut!(logFont.lfFaceName) as LPWSTR,
        core::ptr::addr_of_mut!(logSysFont.lfFaceName) as *mut WCHAR as LPCWSTR,
    );
    hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(logFont));

    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        let mut part: *mut IPPART_INFO =
            (core::ptr::addr_of_mut!((*infoPtr).Part) as *mut IPPART_INFO).offset(i as isize)
                as *mut IPPART_INFO;

        (*part).LowerLimit = 0 as INT;
        (*part).UpperLimit = 255 as INT;
        edit.left = (rcClient.left as ::core::ffi::c_int
            + i * fieldsize
            + 6 as ::core::ffi::c_int) as LONG;
        edit.right = (rcClient.left as ::core::ffi::c_int
            + (i + 1 as ::core::ffi::c_int) * fieldsize
            - 2 as ::core::ffi::c_int) as LONG;
        (*part).EditHwnd = CreateWindowW!(
            (WC_EDITW).as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            (WS_CHILD | WS_VISIBLE | ES_CENTER) as DWORD,
            edit.left,
            edit.top,
            edit.right - edit.left,
            edit.bottom - edit.top,
            hwnd,
            1 as HMENU,
            GetWindowLongPtrW(hwnd, GWLP_HINSTANCE) as HINSTANCE,
            core::ptr::null_mut(),
        );
        SendMessageW(
            (*part).EditHwnd,
            WM_SETFONT as UINT,
            hFont as WPARAM,
            FALSE as LPARAM,
        );
        SetPropW(
            (*part).EditHwnd,
            (IP_SUBCLASS_PROP).as_ptr() as LPCWSTR,
            hwnd as HANDLE,
        );
        (*part).OrigProc = core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtrW(
            (*part).EditHwnd,
            GWLP_WNDPROC as ::core::ffi::c_int,
            (IPADDRESS_SubclassProc as DWORD_PTR) as LONG_PTR,
        ));
        EnableWindow((*part).EditHwnd, (*infoPtr).Enabled);
        i += 1;
    }

    IPADDRESS_UpdateText(infoPtr);

    return 0 as LRESULT;
}

unsafe extern "C" fn IPADDRESS_Destroy(infoPtr: *mut IPADDRESS_INFO) -> LRESULT {
    let mut i: ::core::ffi::c_int = 0;

    while i < 4 {
        let part: *mut IPPART_INFO =
            core::ptr::addr_of_mut!((*infoPtr).Part).cast::<IPPART_INFO>().offset(i as isize);
        SetWindowLongPtrW(
            (*part).EditHwnd,
            GWLP_WNDPROC,
            core::mem::transmute::<WNDPROC, DWORD_PTR>((*part).OrigProc) as LONG_PTR,
        );
        i += 1;
    }

    SetWindowLongPtrW((*infoPtr).Self_, 0, 0 as LONG_PTR);
    heap_free(infoPtr as *mut ::core::ffi::c_void);
    return 0 as LRESULT;
}
unsafe extern "C" fn IPADDRESS_Enable(infoPtr: *mut IPADDRESS_INFO, enabled: BOOL) -> LRESULT {
    let mut i: ::core::ffi::c_int = 0;

    (*infoPtr).Enabled = enabled;

    while i < 4 {
        EnableWindow((*infoPtr).Part[i as usize].EditHwnd, enabled);
        i += 1;
    }

    InvalidateRgn((*infoPtr).Self_, core::ptr::null_mut(), FALSE);
    return 0;
}

unsafe extern "C" fn IPADDRESS_Paint(infoPtr: *const IPADDRESS_INFO, mut hdc: HDC) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/ipaddress.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IPADDRESS_Paint\0" as *const u8 as *const ::core::ffi::c_char,
            311 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !hdc.is_null() {
        return IPADDRESS_Draw(infoPtr, hdc);
    }
    hdc = BeginPaint((*infoPtr).Self_, &raw mut ps);
    IPADDRESS_Draw(infoPtr, hdc);
    EndPaint((*infoPtr).Self_, &raw mut ps);
    return 0;
}

unsafe extern "C" fn IPADDRESS_IsBlank(infoPtr: *const IPADDRESS_INFO) -> BOOL {
    let mut i: i32;

    i = 0;
    while i < 4 {
        if GetWindowTextLengthW((*infoPtr).Part[i as usize].EditHwnd) != 0 {
            return FALSE;
        }
        i += 1;
    }

    return TRUE;
}

unsafe extern "C" fn IPADDRESS_GetAddress(infoPtr: *const IPADDRESS_INFO, ip_address: LPDWORD) -> ::core::ffi::c_int {
    let mut field: [WCHAR; 5] = [0; 5];
    let mut i: ::core::ffi::c_int = 0;
    let mut invalid: ::core::ffi::c_int = 0;
    let mut ip_addr: DWORD = 0;

    i = 0;
    while i < 4 {
        ip_addr = ip_addr.wrapping_mul(256);
        if GetWindowTextW(
            (*infoPtr).Part[i as usize].EditHwnd,
            core::ptr::addr_of_mut!(field) as LPWSTR,
            4,
        ) != 0
        {
            ip_addr = ip_addr.wrapping_add(
                wcstol(
                    core::ptr::addr_of_mut!(field) as *mut WCHAR,
                    core::ptr::null_mut(),
                    10,
                ) as DWORD,
            );
        } else {
            invalid += 1;
        }
        i += 1;
    }
    *ip_address = ip_addr;

    return 4 - invalid;
}

unsafe extern "C" fn IPADDRESS_SetRange(infoPtr: *mut IPADDRESS_INFO, index: INT, range: WORD) -> BOOL {
    if (index < 0) || (index > 3) { return FALSE; }

    (*infoPtr).Part[index as usize].LowerLimit = (range as INT) & 0xFF;
    (*infoPtr).Part[index as usize].UpperLimit = ((range as INT) >> 8) & 0xFF;

    return TRUE;
}

unsafe extern "C" fn IPADDRESS_ClearAddress(infoPtr: *const IPADDRESS_INFO) {
    static nil: [u16; 1] = [0];
    let mut i: i32;

    i = 0;
    while i < 4 {
        SetWindowTextW((*infoPtr).Part[i as usize].EditHwnd, core::ptr::addr_of!(nil) as *const u16);
        i += 1;
    }
}

unsafe extern "C" fn IPADDRESS_SetAddress(infoPtr: *const IPADDRESS_INFO, mut ip_address: DWORD) -> LRESULT {
    let mut buf: [WCHAR; 20] = [0; 20];
    static fmt: [WCHAR; 3] = [0x0025u16, 0x0064, 0x0000];
    let mut i: i32;

    i = 3;
    while i >= 0 {
        let part: *const IPPART_INFO = core::ptr::addr_of!((*infoPtr).Part[i as usize]);
        let value: i32 = (ip_address & 0xff) as i32;
        if (value >= (*part).LowerLimit) && (value <= (*part).UpperLimit) {
            wsprintfW(buf.as_mut_ptr(), fmt.as_ptr(), value);
            SetWindowTextW((*part).EditHwnd, buf.as_mut_ptr());
            IPADDRESS_Notify(infoPtr, EN_CHANGE as UINT);
        }
        ip_address >>= 8;
        i -= 1;
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn IPADDRESS_SetFocusToField(infoPtr: *const IPADDRESS_INFO, mut index: INT) {
    if index > 3 || index < 0 {
        index = 0;
    }
    SetFocus((*infoPtr).Part[index as usize].EditHwnd);
}

unsafe extern "C" fn IPADDRESS_ConstrainField(infoPtr: *const IPADDRESS_INFO, currentfield: i32) -> BOOL {
    static fmt: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
    let mut part: *const IPPART_INFO = core::ptr::null();
    let mut curValue: i32 = 0;
    let mut newValue: i32 = 0;
    let mut field: [WCHAR; 10] = [0; 10];

    if currentfield < 0 || currentfield > 3 {
        return FALSE;
    }

    part = (core::ptr::addr_of!((*infoPtr).Part) as *const IPPART_INFO).offset(currentfield as isize);
    if GetWindowTextW(
        (*part).EditHwnd,
        core::ptr::addr_of_mut!(field) as LPWSTR,
        4,
    ) == 0
    {
        return FALSE;
    }

    curValue = wcstol(
        core::ptr::addr_of_mut!(field) as *mut WCHAR,
        core::ptr::null_mut(),
        10,
    ) as i32;

    newValue = IPADDRESS_IPNotify(infoPtr, currentfield, curValue);

    if newValue < (*part).LowerLimit {
        newValue = (*part).LowerLimit;
    }
    if newValue > (*part).UpperLimit {
        newValue = (*part).UpperLimit;
    }

    if newValue == curValue {
        return FALSE;
    }

    wsprintfW(
        core::ptr::addr_of_mut!(field) as LPWSTR,
        core::ptr::addr_of!(fmt) as LPCWSTR,
        newValue,
    );
    return SetWindowTextW(
        (*part).EditHwnd,
        core::ptr::addr_of_mut!(field) as *mut WCHAR as LPCWSTR,
    );
}

unsafe extern "C" fn IPADDRESS_GotoNextField(infoPtr: *const IPADDRESS_INFO, cur: i32, sel: i32) -> BOOL {
    if cur >= -1 && cur < 4 {
        IPADDRESS_ConstrainField(infoPtr, cur);

        if cur < 3 {
            let next: *const IPPART_INFO = (core::ptr::addr_of!((*infoPtr).Part) as *const IPPART_INFO)
                .offset((cur + 1) as isize);
            let mut start: i32 = 0;
            let mut end: i32 = 0;
            SetFocus((*next).EditHwnd);
            if sel != POS_DEFAULT {
                if sel == POS_RIGHT {
                    end = GetWindowTextLengthW((*next).EditHwnd);
                    start = end;
                } else if sel == POS_SELALL {
                    end = -1;
                }
                SendMessageW(
                    (*next).EditHwnd,
                    EM_SETSEL as UINT,
                    start as WPARAM,
                    end as LPARAM,
                );
            }
            return TRUE;
        }
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn IPADDRESS_SubclassProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut Self_0: HWND = GetPropW(hwnd, (IP_SUBCLASS_PROP).as_ptr()) as HWND;
    let mut infoPtr: *mut IPADDRESS_INFO = GetWindowLongPtrW(Self_0, 0) as *mut IPADDRESS_INFO;
    let mut c: CHAR = wParam as CHAR;
    let mut index: INT = 0;
    let mut len: INT = 0;
    let mut startsel: INT = 0;
    let mut endsel: INT = 0;
    let mut part: *mut IPPART_INFO = core::ptr::null_mut();

    index = IPADDRESS_GetPartIndex(infoPtr, hwnd);
    if index < 0 {
        return 0 as LRESULT;
    }
    part = (&raw mut (*infoPtr).Part as *mut IPPART_INFO).offset(index as isize) as *mut IPPART_INFO;

    if uMsg == WM_CHAR as UINT || uMsg == WM_KEYDOWN as UINT {
        len = GetWindowTextLengthW(hwnd) as INT;
        SendMessageW(hwnd, EM_GETSEL as UINT, &raw mut startsel as WPARAM, &raw mut endsel as LPARAM);
    }

    'sw: {
        if uMsg == WM_CHAR as UINT {
            if isdigit(c as INT) != 0 {
                if len == 2 && startsel == endsel && endsel == len {
                    /* process the digit press before we check the field */
                    let mut return_val: INT = CallWindowProcW((*part).OrigProc, hwnd, uMsg, wParam, lParam) as INT;

                    /* if the field value was changed stay at the current field */
                    if IPADDRESS_ConstrainField(infoPtr, index) == 0 {
                        IPADDRESS_GotoNextField(infoPtr, index, POS_DEFAULT);
                    }

                    return return_val as LRESULT;
                } else if len == 3 && startsel == endsel && endsel == len {
                    IPADDRESS_GotoNextField(infoPtr, index, POS_SELALL);
                } else if len < 3 || startsel != endsel {
                    break 'sw;
                }
            } else if c as INT == '.' as INT || c as INT == ' ' as INT {
                if len != 0 && startsel == endsel && startsel != 0 {
                    IPADDRESS_GotoNextField(infoPtr, index, POS_SELALL);
                }
            } else if (c as INT) as u32 == VK_BACK {
                break 'sw;
            }
            return 0 as LRESULT;
        } else if uMsg == WM_KEYDOWN as UINT {
            // inner switch(c)
            if (c as INT) as u32 == VK_RIGHT {
                if startsel == endsel && startsel == len {
                    IPADDRESS_GotoNextField(infoPtr, index, POS_LEFT);
                    return 0 as LRESULT;
                }
            } else if (c as INT) as u32 == VK_LEFT {
                if startsel == 0 && startsel == endsel && index > 0 {
                    IPADDRESS_GotoNextField(infoPtr, index - 2, POS_RIGHT);
                    return 0 as LRESULT;
                }
            } else if (c as INT) as u32 == VK_BACK {
                if startsel == endsel && startsel == 0 && index > 0 {
                    let mut prev: *mut IPPART_INFO = (&raw mut (*infoPtr).Part as *mut IPPART_INFO)
                        .offset((index - 1) as isize) as *mut IPPART_INFO;
                    let mut val: [WCHAR; 10] = [0 as WCHAR; 10];

                    if GetWindowTextW((*prev).EditHwnd, val.as_mut_ptr(), 5) != 0 {
                        val[(lstrlenW(val.as_mut_ptr() as LPCWSTR) - 1) as usize] = 0;
                        SetWindowTextW((*prev).EditHwnd, val.as_ptr() as LPCWSTR);
                    }

                    IPADDRESS_GotoNextField(infoPtr, index - 2, POS_RIGHT);
                    return 0 as LRESULT;
                }
            }
        } else if uMsg == WM_KILLFOCUS as UINT {
            if IPADDRESS_GetPartIndex(infoPtr, wParam as HWND) < 0 {
                IPADDRESS_Notify(infoPtr, EN_KILLFOCUS as UINT);
            }
        } else if uMsg == WM_SETFOCUS as UINT {
            if IPADDRESS_GetPartIndex(infoPtr, wParam as HWND) < 0 {
                IPADDRESS_Notify(infoPtr, EN_SETFOCUS as UINT);
            }
        }
    }

    return CallWindowProcW((*part).OrigProc, hwnd, uMsg, wParam, lParam);
}

unsafe extern "system" fn IPADDRESS_WindowProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut IPADDRESS_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut IPADDRESS_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/ipaddress.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IPADDRESS_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            593 as ::core::ffi::c_int,
            b"(hwnd=%p msg=0x%x wparam=0x%lx lparam=0x%lx)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && (uMsg != WM_CREATE as UINT) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    match uMsg {
        WM_CREATE => {
            return IPADDRESS_Create(hwnd, lParam as LPCREATESTRUCTA);
        }
        WM_DESTROY => {
            return IPADDRESS_Destroy(infoPtr);
        }
        WM_ENABLE => {
            return IPADDRESS_Enable(infoPtr, wParam as BOOL);
        }
        WM_PAINT => {
            return IPADDRESS_Paint(infoPtr, wParam as HDC);
        }
        WM_SETFOCUS => {
            IPADDRESS_GotoNextField(infoPtr, -1, POS_SELALL);
            return 0;
        }
        WM_COMMAND => {
            match (wParam >> 16) as UINT {
                EN_CHANGE => {
                    IPADDRESS_UpdateText(infoPtr);
                    IPADDRESS_Notify(infoPtr, EN_CHANGE);
                }
                EN_KILLFOCUS => {
                    IPADDRESS_ConstrainField(
                        infoPtr,
                        IPADDRESS_GetPartIndex(infoPtr, lParam as HWND),
                    );
                }
                _ => {}
            }
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0;
        }
        IPM_CLEARADDRESS => {
            IPADDRESS_ClearAddress(infoPtr);
        }
        IPM_SETADDRESS => {
            return IPADDRESS_SetAddress(infoPtr, lParam as DWORD);
        }
        IPM_GETADDRESS => {
            return (IPADDRESS_GetAddress(infoPtr, lParam as LPDWORD)) as LRESULT;
        }
        IPM_SETRANGE => {
            return (IPADDRESS_SetRange(infoPtr, wParam as ::core::ffi::c_int, lParam as WORD)) as LRESULT;
        }
        IPM_SETFOCUS => {
            IPADDRESS_SetFocusToField(infoPtr, wParam as ::core::ffi::c_int);
        }
        IPM_ISBLANK => {
            return (IPADDRESS_IsBlank(infoPtr)) as LRESULT;
        }
        _ => {
            if uMsg >= WM_USER as UINT
                && uMsg < WM_APP as UINT
                && COMCTL32_IsReflectedMessage(uMsg) == 0
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/ipaddress.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"IPADDRESS_WindowProc\0" as *const u8
                            as *const ::core::ffi::c_char,
                        655 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uMsg,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn IPADDRESS_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory! (core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSW>());
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS | CS_HREDRAW | CS_VREDRAW) as UINT;
    wndClass.lpfnWndProc = Some(IPADDRESS_WindowProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT) as WNDPROC;
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut IPADDRESS_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW (core::ptr::null_mut(), IDC_IBEAM as LPWSTR as LPCWSTR);
    wndClass.hbrBackground = (COLOR_WINDOW + 1) as HBRUSH;
    wndClass.lpszClassName = (WC_IPADDRESSW).as_ptr() as LPCWSTR;

    RegisterClassW (core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn IPADDRESS_Unregister() {
    UnregisterClassW((WC_IPADDRESSW).as_ptr(), core::ptr::null_mut());
}
