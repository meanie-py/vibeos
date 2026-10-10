use crate::comctl32::imagelist::ImageList_Draw;
use crate::comctl32::imagelist::ImageList_GetIconSize;
use crate::comctl32::imagelist::ImageList_GetImageCount;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
crate::wine_default_debug_channel!(__wine_dbch_button);
pub static RUSTCONV_WSTR_CD8397DE2CFB: [u16; 6] = [0x0041u16, 0x0072, 0x0069, 0x0061, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_FA8A27D23A1A: [u16; 6] = [0x0053u16, 0x0074, 0x0061, 0x0072, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_BUTTONW: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"B\0u\0t\0t\0o\0n\0\0\0")
};

static mut maxCheckState: [WORD; 16] = [
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_CHECKED as WORD,
    BST_CHECKED as WORD,
    BST_CHECKED as WORD,
    BST_INDETERMINATE as WORD,
    BST_INDETERMINATE as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_CHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
    BST_UNCHECKED as WORD,
];

static mut btnPaintFunc: [pfPaint; 16] = unsafe {
    [
        Some(PB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(PB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(CB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(CB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(CB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(CB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(CB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(GB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(UB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(CB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        None,
        Some(OB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(PB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(PB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(PB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
        Some(PB_Paint as unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT) -> ()),
    ]
};

static mut btnThemedPaintFunc: [pfThemedPaint; 16] = unsafe {
    [
        Some(
            PB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            PB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            CB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            CB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            CB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            CB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            CB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        Some(
            GB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        None,
        Some(
            CB_ThemedPaint
                as unsafe extern "C" fn(
                    HTHEME,
                    *const BUTTON_INFO,
                    HDC,
                    ButtonState,
                    UINT,
                    BOOL,
                    LPARAM,
                ) -> (),
        ),
        None,
        None,
        None,
        None,
        None,
        None,
    ]
};

pub const BUTTON_NSTATES: i32 = 0x0F;

pub const BUTTON_BTNPRESSED: i32 = 0x40;

pub const BUTTON_UNKNOWN2: i32 = 0x20;

pub const BUTTON_UNKNOWN3: i32 = 0x10;

pub const BUTTON_BMCLICK: i32 = 0x100; // ReactOS Need to up to wine!

#[repr(C)]
pub union BUTTON_INFO_U {
    pub icon: HICON,
    pub bitmap: HBITMAP,
    pub image: HANDLE,
}

#[repr(C)]
pub struct BUTTON_INFO {
    pub hwnd: HWND,
    pub parent: HWND,
    pub state: LONG,
    pub font: HFONT,
    pub note: *mut WCHAR,
    pub note_length: INT,
    pub u: BUTTON_INFO_U,
    pub ui_state: DWORD,
    pub rcTextMargin: RECT,
    pub imlData: BUTTON_IMAGELIST,
}

pub const MAX_BTN_TYPE: usize = 16;

pub type ButtonState = i32;
pub const STATE_NORMAL: ButtonState = 0;
pub const STATE_DISABLED: ButtonState = 1;
pub const STATE_HOT: ButtonState = 2;
pub const STATE_PRESSED: ButtonState = 3;
pub const STATE_DEFAULTED: ButtonState = 4;

pub type pfPaint = Option<unsafe extern "C" fn(*const BUTTON_INFO, HDC, UINT)>;

pub type pfThemedPaint = Option<unsafe extern "C" fn(
    HTHEME,
    *const BUTTON_INFO,
    HDC,
    ButtonState,
    UINT,
    BOOL,
    LPARAM,
)>;

unsafe extern "C" fn get_button_type(mut window_style: LONG) -> UINT {
    return (window_style as ::core::ffi::c_long & (BS_TYPEMASK) as i32) as UINT;
}

unsafe extern "C" fn get_button_text(infoPtr: *const BUTTON_INFO) -> *mut WCHAR {
    let len: INT = GetWindowTextLengthW((*infoPtr).hwnd) as INT;
    let buffer: *mut WCHAR = heap_alloc(
        ((len + 1) as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if !buffer.is_null() {
        GetWindowTextW((*infoPtr).hwnd, buffer as LPWSTR, len + 1);
    }
    return buffer;
}
#[no_mangle]
pub unsafe extern "C" fn set_control_clipping(hdc: HDC, rect: *const RECT) -> HRGN {
    let mut rc: RECT = *rect;
    let mut hrgn: HRGN = CreateRectRgn(
        0,
        0,
        0,
        0,
    );

    if GetClipRgn(hdc, hrgn) != 1 {
        DeleteObject(hrgn as HGDIOBJ);
        hrgn = core::ptr::null_mut();
    }
    DPtoLP(hdc, core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
    if (GetLayout(hdc) & LAYOUT_RTL) != 0 {
        /* compensate for the shifting done by IntersectClipRect */
        rc.left += 1;
        rc.right += 1;
    }
    IntersectClipRect(hdc, rc.left, rc.top, rc.right, rc.bottom);
    return hrgn;
}

unsafe extern "C" fn heap_strndupW(src: *const WCHAR, length: size_t) -> *mut WCHAR {
    let size: size_t = (length + 1) * ::core::mem::size_of::<WCHAR>() as size_t;
    let mut dst: *mut WCHAR = heap_alloc(size as SIZE_T) as *mut WCHAR;
    if !dst.is_null() {
        memcpy(
            dst as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            size,
        );
    }
    return dst;
}

unsafe extern "C" fn BUTTON_BStoDT(mut style: DWORD, mut ex_style: DWORD) -> UINT {
    let mut dtStyle: UINT = DT_NOCLIP as UINT;  /* We use SelectClipRgn to limit output */

    /* "Convert" pushlike buttons to pushbuttons */
    if style & (BS_PUSHLIKE as DWORD) != 0 {
        style &= !(BS_TYPEMASK as DWORD);
    }

    if style & (BS_MULTILINE as DWORD) == 0 {
        dtStyle |= DT_SINGLELINE as UINT;
    } else {
        dtStyle |= DT_WORDBREAK as UINT;
    }

    match style & (BS_CENTER as DWORD) {
        BS_LEFT => {
            /* DT_LEFT is 0 */
        }
        BS_RIGHT => {
            dtStyle |= DT_RIGHT as UINT;
        }
        BS_CENTER => {
            dtStyle |= DT_CENTER as UINT;
        }
        _ => {
            /* Pushbutton's text is centered by default */
            if get_button_type(style as LONG) <= BS_DEFPUSHBUTTON as UINT {
                dtStyle |= DT_CENTER as UINT;
            }
            /* all other flavours have left aligned text */
        }
    }

    if ex_style & (WS_EX_RIGHT as DWORD) != 0 {
        dtStyle = DT_RIGHT as UINT
            | (dtStyle & !((DT_LEFT as UINT) | (DT_CENTER as UINT)));
    }

    /* DrawText ignores vertical alignment for multiline text,
     * but we use these flags to align label manually.
     */
    if get_button_type(style as LONG) != BS_GROUPBOX as UINT {
        match style & (BS_VCENTER as DWORD) {
            BS_TOP => {
                /* DT_TOP is 0 */
            }
            BS_BOTTOM => {
                dtStyle |= DT_BOTTOM as UINT;
            }
            /* fall through */
            BS_VCENTER | _ => {
                dtStyle |= DT_VCENTER as UINT;
            }
        }
    } else {
        /* GroupBox's text is always single line and is top aligned. */
        dtStyle |= DT_SINGLELINE as UINT;
    }

    return dtStyle;
}
#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "C" fn BUTTON_PaintWithTheme(theme: HTHEME, infoPtr: *const BUTTON_INFO, hParamDC: HDC, prfFlag: LPARAM) -> BOOL {
    let mut dwStyle: DWORD = 0;
    let mut dwStyleEx: DWORD = 0;
    let mut type_: DWORD = 0;
    let mut dtFlags: UINT = 0;
    let mut drawState: ButtonState = STATE_NORMAL;
    let mut paint: pfThemedPaint = None;

    /* Don't draw with themes on a button with BS_ICON or BS_BITMAP */
    if ((*infoPtr).u.image as usize) != 0 as usize {
        return FALSE;
    }

    /* If the button isn't visible, don't paint it (themed or not) */
    if ((IsWindowVisible((*infoPtr).hwnd)) == 0) {
        return TRUE; /* Pretend it has been drawn successfully */
    }

    dwStyle = GetWindowLongW((*infoPtr).hwnd, GWL_STYLE) as DWORD;
    type_ = get_button_type(dwStyle as LONG) as DWORD;

    if type_ != BS_PUSHBUTTON as DWORD
        && type_ != BS_DEFPUSHBUTTON as DWORD
        && (dwStyle & BS_PUSHLIKE as DWORD) != 0
    {
        type_ = BS_PUSHBUTTON as DWORD;
    }

    paint = btnThemedPaintFunc[type_ as usize];
    if paint.is_none() {
        return FALSE;
    }

    dwStyleEx = GetWindowLongW((*infoPtr).hwnd, GWL_EXSTYLE) as DWORD;
    dtFlags = BUTTON_BStoDT(dwStyle, dwStyleEx);

    if (dwStyle & WS_DISABLED as DWORD) != 0 {
        drawState = STATE_DISABLED;
    } else if ((*infoPtr).state as BOOL & BST_PUSHED as BOOL) != 0 {
        drawState = STATE_PRESSED;
    } else if (dwStyle & BS_PUSHLIKE as DWORD) != 0
        && ((*infoPtr).state as BOOL & (BST_CHECKED | BST_INDETERMINATE) as BOOL) != 0
    {
        drawState = STATE_PRESSED;
    } else if ((*infoPtr).state as BOOL & (BST_HOT as i32)) != 0 {
        drawState = STATE_HOT;
    } else if ((*infoPtr).state as BOOL & BST_FOCUS as BOOL) != 0
        || (dwStyle & BS_DEFPUSHBUTTON as DWORD) != 0
    {
        drawState = STATE_DEFAULTED;
    } else {
        drawState = STATE_NORMAL;
    }

    if paint
        == Some(PB_ThemedPaint as unsafe extern "C" fn(HTHEME, *const BUTTON_INFO, HDC, ButtonState, UINT, BOOL, LPARAM) -> ())
        || paint
            == Some(CB_ThemedPaint as unsafe extern "C" fn(HTHEME, *const BUTTON_INFO, HDC, ButtonState, UINT, BOOL, LPARAM) -> ())
    {
        let mut hdc: HDC = core::ptr::null_mut();
        let mut hbmp: HBITMAP = core::ptr::null_mut();
        let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

        GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rc));
        hdc = CreateCompatibleDC(hParamDC);
        hbmp = CreateCompatibleBitmap(hParamDC, rc.right as INT, rc.bottom as INT);
        if !hdc.is_null() && !hbmp.is_null() {
            SelectObject(hdc, hbmp as HGDIOBJ);

            (paint.unwrap_unchecked())(theme, infoPtr, hdc, drawState, dtFlags, (*infoPtr).state as BOOL & (BST_FOCUS) as i32, prfFlag);

            BitBlt(hParamDC, 0 as INT, 0 as INT, rc.right as INT, rc.bottom as INT, hdc, 0 as INT, 0 as INT, SRCCOPY as DWORD);
            DeleteObject(hbmp as HGDIOBJ);
            DeleteDC(hdc);
            return TRUE;
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/button.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"BUTTON_PaintWithTheme\0" as *const u8 as *const ::core::ffi::c_char,
                    386 as ::core::ffi::c_int,
                    b"Failed to create DC and bitmap for double buffering\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if !hbmp.is_null() {
                DeleteObject(hbmp as HGDIOBJ);
            }
            if !hdc.is_null() {
                DeleteDC(hdc);
            }
        }
    }

    (paint.unwrap_unchecked())(theme, infoPtr, hParamDC, drawState, dtFlags, (*infoPtr).state as BOOL & (BST_FOCUS) as i32, prfFlag);
    return TRUE;
}

unsafe extern "C" fn paint_button(infoPtr: *mut BUTTON_INFO, style: LONG, action: UINT) {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwnd);
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let hdc: HDC = GetDC((*infoPtr).hwnd);
    /* GetDC appears to give a dc with a clip rect that includes the whoe parent, not sure if it is correct or not. */
    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rc));
    IntersectClipRect(
        hdc,
        rc.left as ::core::ffi::c_int,
        rc.top as ::core::ffi::c_int,
        rc.right as ::core::ffi::c_int,
        rc.bottom as ::core::ffi::c_int,
    );
    if !theme.is_null()
        && BUTTON_PaintWithTheme(theme, infoPtr, hdc, 0 as LPARAM) != 0
    {
        ReleaseDC((*infoPtr).hwnd, hdc);
        return;
    }
    if let Some(paint) = btnPaintFunc[style as usize].filter(|_| IsWindowVisible((*infoPtr).hwnd) != 0) {
        paint(infoPtr, hdc, action);
    }
    ReleaseDC((*infoPtr).hwnd, hdc);
}
#[no_mangle]
pub unsafe extern "C" fn BUTTON_GetIdealSize(infoPtr: *mut BUTTON_INFO, theme: HTHEME, psize: *mut SIZE) -> BOOL {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut text: *mut WCHAR;
    let mut hFont: HFONT = 0 as HFONT;
    let mut hPrevFont: HFONT = 0 as HFONT;
    let mut TextSize: SIZE = SIZE { cx: 0, cy: 0 };
    let mut ImageSize: SIZE = SIZE { cx: 0, cy: 0 };
    let mut ButtonSize: SIZE = SIZE { cx: 0, cy: 0 };
    let mut ret: BOOL = FALSE as BOOL;
    let mut logfont: LOGFONTW = core::mem::zeroed();

    text = get_button_text(infoPtr);
    hdc = GetDC((*infoPtr).hwnd);
    'cleanup: {
        if text.is_null() || hdc.is_null() || *text.offset(0 as isize) == 0 {
            break 'cleanup;
        }

        /* FIXME : Should use GetThemeTextExtent but unfortunately uses DrawTextW which is broken */
        if !theme.is_null() {
            let mut hr: HRESULT = GetThemeFont(
                theme,
                hdc,
                BP_PUSHBUTTON as i32,
                PBS_NORMAL as i32,
                TMT_FONT as i32,
                core::ptr::addr_of_mut!(logfont),
            );
            if hr >= 0 as HRESULT {
                hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(logfont));
                if !hFont.is_null() {
                    hPrevFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
                }
            }
        } else {
            if !(*infoPtr).font.is_null() {
                hPrevFont = SelectObject(hdc, (*infoPtr).font as HGDIOBJ) as HFONT;
            }
        }

        GetTextExtentPoint32W(
            hdc,
            text,
            wcslen(text) as i32,
            core::ptr::addr_of_mut!(TextSize),
        );

        if logfont.lfHeight == -(1 as LONG)
            && logfont.lfWidth == 0 as LONG
            && wcscmp(
                core::ptr::addr_of_mut!(logfont.lfFaceName) as *mut WCHAR,
                RUSTCONV_WSTR_CD8397DE2CFB.as_ptr(),
            ) == 0 as i32
            && wcsicmp(text, RUSTCONV_WSTR_FA8A27D23A1A.as_ptr()) == 0 as i32
        {
            TextSize.cx = 5 as LONG;
            TextSize.cy = 4 as LONG;
        }

        if !hPrevFont.is_null() {
            SelectObject(hdc, hPrevFont as HGDIOBJ);
        }

        TextSize.cy += (*infoPtr).rcTextMargin.top + (*infoPtr).rcTextMargin.bottom;
        TextSize.cx += (*infoPtr).rcTextMargin.left + (*infoPtr).rcTextMargin.right;

        if !(*infoPtr).imlData.himl.is_null()
            && ImageList_GetIconSize(
                ((*infoPtr).imlData.himl).cast(),
                core::ptr::addr_of_mut!(ImageSize.cx),
                core::ptr::addr_of_mut!(ImageSize.cy),
            ) != 0
        {
            ImageSize.cx += (*infoPtr).imlData.margin.left + (*infoPtr).imlData.margin.right;
            ImageSize.cy += (*infoPtr).imlData.margin.top + (*infoPtr).imlData.margin.bottom;
        } else {
            ImageSize.cy = 0 as LONG;
            ImageSize.cx = ImageSize.cy;
        }

        if !theme.is_null() {
            let mut rcContents: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            let mut rcButtonExtent: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            rcContents.right = ImageSize.cx + TextSize.cx;
            rcContents.bottom = core::cmp::max(ImageSize.cy, TextSize.cy);
            GetThemeBackgroundExtent(
                theme,
                hdc,
                BP_PUSHBUTTON as i32,
                PBS_NORMAL as i32,
                core::ptr::addr_of_mut!(rcContents),
                core::ptr::addr_of_mut!(rcButtonExtent),
            );
            ButtonSize.cx = rcButtonExtent.right - rcButtonExtent.left;
            ButtonSize.cy = rcButtonExtent.bottom - rcButtonExtent.top;
        } else {
            ButtonSize.cx = ImageSize.cx + TextSize.cx + 5 as LONG;
            ButtonSize.cy = core::cmp::max(
                ImageSize.cy,
                TextSize.cy + 7 as LONG,
            );
        }

        *psize = ButtonSize;
        ret = TRUE as BOOL;
    }

    // cleanup:
    if !hFont.is_null() {
        DeleteObject(hFont as HGDIOBJ);
    }
    if !text.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, text as PVOID);
    }
    if !hdc.is_null() {
        ReleaseDC((*infoPtr).hwnd, hdc);
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn BUTTON_DrawIml(hDC: HDC, pimlData: *const BUTTON_IMAGELIST, prc: *mut RECT, bOnlyCalc: BOOL, mut index: core::ffi::c_int) -> BOOL {
    let mut ImageSize: SIZE = SIZE { cx: 0, cy: 0 };
    let mut left: core::ffi::c_int = 0;
    let mut top: core::ffi::c_int = 0;
    let mut count: core::ffi::c_int = 0;

    if (*pimlData).himl.is_null() {
        return FALSE;
    }

    if ImageList_GetIconSize(((*pimlData).himl).cast(), core::ptr::addr_of_mut!(ImageSize.cx), core::ptr::addr_of_mut!(ImageSize.cy)) == 0 {
        return FALSE;
    }

    if (*pimlData).uAlign == BUTTON_IMAGELIST_ALIGN_LEFT {
        left = (*prc).left + (*pimlData).margin.left;
        top = (*prc).top + ((*prc).bottom - (*prc).top - ImageSize.cy) / 2;
        (*prc).left = left + (*pimlData).margin.right + ImageSize.cx;
    }
    else if (*pimlData).uAlign == BUTTON_IMAGELIST_ALIGN_RIGHT {
        left = (*prc).right - (*pimlData).margin.right - ImageSize.cx;
        top = (*prc).top + ((*prc).bottom - (*prc).top - ImageSize.cy) / 2;
        (*prc).right = left - (*pimlData).margin.left;
    }
    else if (*pimlData).uAlign == BUTTON_IMAGELIST_ALIGN_TOP {
        left = (*prc).left + ((*prc).right - (*prc).left - ImageSize.cx) / 2;
        top = (*prc).top + (*pimlData).margin.top;
        (*prc).top = top + ImageSize.cy + (*pimlData).margin.bottom;
    }
    else if (*pimlData).uAlign == BUTTON_IMAGELIST_ALIGN_BOTTOM {
        left = (*prc).left + ((*prc).right - (*prc).left - ImageSize.cx) / 2;
        top = (*prc).bottom - (*pimlData).margin.bottom - ImageSize.cy;
        (*prc).bottom = top - (*pimlData).margin.top;
    }
    else if (*pimlData).uAlign == BUTTON_IMAGELIST_ALIGN_CENTER {
        left = (*prc).left + ((*prc).right - (*prc).left - ImageSize.cx) / 2;
        top = (*prc).top + ((*prc).bottom - (*prc).top - ImageSize.cy) / 2;
    }

    if bOnlyCalc != 0 {
        return TRUE;
    }

    count = ImageList_GetImageCount(((*pimlData).himl).cast());

    if count == 1 {
        index = 0;
    }
    else if index >= count {
        return TRUE;
    }

    ImageList_Draw(((*pimlData).himl).cast(), index, hDC, left, top, 0);

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn BUTTON_SendCustomDraw(infoPtr: *const BUTTON_INFO, hDC: HDC, dwDrawStage: DWORD, prc: *mut RECT) -> DWORD {
    let mut nmcs: NMCUSTOMDRAW = core::mem::zeroed();

    nmcs.hdr.hwndFrom = (*infoPtr).hwnd;
    nmcs.hdr.idFrom = GetWindowLongPtrW((*infoPtr).hwnd, GWLP_ID) as UINT_PTR;
    nmcs.hdr.code = NM_CUSTOMDRAW as UINT;
    nmcs.dwDrawStage = dwDrawStage;
    nmcs.hdc = hDC;
    nmcs.rc = *prc;
    nmcs.dwItemSpec = 0 as DWORD_PTR;
    nmcs.uItemState = 0 as UINT;
    nmcs.lItemlParam = 0 as LPARAM;
    if IsWindowEnabled((*infoPtr).hwnd) == 0 {
        nmcs.uItemState |= CDIS_DISABLED as UINT;
    }
    if (*infoPtr).state as core::ffi::c_int & ((BST_CHECKED | BST_INDETERMINATE) as core::ffi::c_int) != 0 {
        nmcs.uItemState |= CDIS_CHECKED as UINT;
    }
    if (*infoPtr).state as core::ffi::c_int & (BST_FOCUS as core::ffi::c_int) != 0 {
        nmcs.uItemState |= CDIS_FOCUS as UINT;
    }
    if (*infoPtr).state as core::ffi::c_int & (BST_PUSHED as core::ffi::c_int) != 0 {
        nmcs.uItemState |= CDIS_SELECTED as UINT;
    }
    if (*infoPtr).ui_state & UISF_HIDEACCEL as DWORD == 0 {
        nmcs.uItemState |= CDIS_SHOWKEYBOARDCUES as UINT;
    }

    return SendMessageW(
        GetParent((*infoPtr).hwnd),
        WM_NOTIFY as UINT,
        nmcs.hdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmcs) as LPARAM,
    ) as DWORD;
}

unsafe extern "C" fn button_update_uistate(infoPtr: *mut BUTTON_INFO) -> BOOL {
    let flags: LONG = DefWindowProcW((*infoPtr).hwnd, WM_QUERYUISTATE, 0, 0) as LONG;

    if (*infoPtr).ui_state != flags as DWORD
    {
        (*infoPtr).ui_state = flags as DWORD;
        return TRUE;
    }

    return FALSE;
}

unsafe extern "system" fn BUTTON_WindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut wParam = wParam;
    let mut infoPtr: *mut BUTTON_INFO = GetWindowLongPtrW(hWnd, 0) as *mut BUTTON_INFO;
    let mut rect: RECT = core::mem::zeroed::<RECT>();
    let mut pt: POINT = core::mem::zeroed::<POINT>();
    let mut style: LONG = GetWindowLongW(hWnd, GWL_STYLE);
    let mut btn_type: UINT = get_button_type(style);
    let mut state: LONG = 0;
    let mut new_state: LONG = 0;
    let mut oldHbitmap: HANDLE = core::ptr::null_mut();
    let mut theme: HTHEME = core::ptr::null_mut();

    if IsWindow(hWnd) == 0 {
        return 0 as LRESULT;
    }

    if infoPtr.is_null() && (uMsg != WM_NCCREATE) {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    pt.x = LOWORD!(lParam) as ::core::ffi::c_short as LONG;
    pt.y = HIWORD!(lParam) as ::core::ffi::c_short as LONG;

    'sw: {
        match uMsg {
            WM_GETDLGCODE => {
                match btn_type {
                    BS_COMMANDLINK | BS_USERBUTTON | BS_PUSHBUTTON => {
                        return (DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON) as LRESULT;
                    }
                    BS_DEFCOMMANDLINK | BS_DEFPUSHBUTTON => {
                        return (DLGC_BUTTON | DLGC_DEFPUSHBUTTON) as LRESULT;
                    }
                    BS_RADIOBUTTON | BS_AUTORADIOBUTTON => {
                        return (DLGC_BUTTON | DLGC_RADIOBUTTON) as LRESULT;
                    }
                    BS_GROUPBOX => {
                        return DLGC_STATIC as LRESULT;
                    }
                    BS_SPLITBUTTON => {
                        return (DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON | DLGC_WANTARROWS) as LRESULT;
                    }
                    BS_DEFSPLITBUTTON => {
                        return (DLGC_BUTTON | DLGC_DEFPUSHBUTTON | DLGC_WANTARROWS) as LRESULT;
                    }
                    _ => {
                        return DLGC_BUTTON as LRESULT;
                    }
                }
            }

            WM_ENABLE => {
                paint_button(infoPtr, btn_type as LONG, ODA_DRAWENTIRE);
                break 'sw;
            }

            WM_NCCREATE => {
                infoPtr = heap_alloc_zero(core::mem::size_of::<BUTTON_INFO>() as SIZE_T) as *mut BUTTON_INFO;
                SetWindowLongPtrW(hWnd, 0, infoPtr as LONG_PTR);
                (*infoPtr).hwnd = hWnd;
                SetRect(
                    core::ptr::addr_of_mut!((*infoPtr).rcTextMargin),
                    1,
                    1,
                    1,
                    1,
                );
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }

            WM_NCDESTROY => {
                SetWindowLongPtrW(hWnd, 0, 0);
                heap_free((*infoPtr).note as *mut ::core::ffi::c_void);
                heap_free(infoPtr as *mut ::core::ffi::c_void);
                break 'sw;
            }

            WM_CREATE => {
                if btn_type >= (MAX_BTN_TYPE as u32) {
                    return (-(1)) as LRESULT; /* abort */
                }

                /* XP turns a BS_USERBUTTON into BS_PUSHBUTTON */
                if btn_type == BS_USERBUTTON {
                    style = (style & (!BS_TYPEMASK) as i32) | (BS_PUSHBUTTON) as i32;
                    SetWindowLongW(hWnd, GWL_STYLE, style);
                }
                (*infoPtr).state = (BST_UNCHECKED) as i32;
                OpenThemeData(hWnd, (WC_BUTTONW).as_ptr());
                return 0 as LRESULT;
            }

            WM_DESTROY => {
                theme = GetWindowTheme(hWnd);
                CloseThemeData(theme);
                break 'sw;
            }

            WM_THEMECHANGED => {
                theme = GetWindowTheme(hWnd);
                CloseThemeData(theme);
                OpenThemeData(hWnd, (WC_BUTTONW).as_ptr());
                InvalidateRect(hWnd, core::ptr::null::<RECT>(), TRUE);
                break 'sw;
            }

            WM_ERASEBKGND => {
                if btn_type == BS_OWNERDRAW {
                    let hdc: HDC = wParam as HDC;
                    let mut rc: RECT = core::mem::zeroed::<RECT>();
                    let mut hBrush: HBRUSH = core::ptr::null_mut();
                    let mut parent: HWND = GetParent(hWnd);
                    if parent.is_null() {
                        parent = hWnd;
                    }
                    hBrush = SendMessageW(parent, WM_CTLCOLORBTN, hdc as WPARAM, hWnd as LPARAM) as HBRUSH;
                    if hBrush.is_null() {
                        /* did the app forget to call defwindowproc ? */
                        hBrush = DefWindowProcW(parent, WM_CTLCOLORBTN, hdc as WPARAM, hWnd as LPARAM) as HBRUSH;
                    }
                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
                    FillRect(hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hBrush);
                }
                return 1 as LRESULT;
            }

            WM_PRINTCLIENT | WM_PAINT => {
                let mut ps: PAINTSTRUCT = core::mem::zeroed::<PAINTSTRUCT>();
                let mut hdc: HDC = core::ptr::null_mut();

                theme = GetWindowTheme(hWnd);
                hdc = if wParam != 0 {
                    wParam as HDC
                } else {
                    BeginPaint(hWnd, core::ptr::addr_of_mut!(ps))
                };

                if !theme.is_null()
                    && BUTTON_PaintWithTheme(
                        theme,
                        infoPtr,
                        hdc,
                        (if uMsg == WM_PRINTCLIENT { lParam } else { 0 }),
                    ) != 0
                {
                    if wParam == 0 {
                        EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
                    }
                    return 0 as LRESULT;
                } else if let Some(paint) = btnPaintFunc[btn_type as usize] {
                    let mut nOldMode: ::core::ffi::c_int = SetBkMode(hdc, (OPAQUE as i32));
                    paint(
                        infoPtr,
                        hdc,
                        ODA_DRAWENTIRE,
                    );
                    SetBkMode(hdc, nOldMode); /*  reset painting mode */
                }

                if wParam == 0 {
                    EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
                }
                break 'sw;
            }

            WM_KEYDOWN => {
                if wParam == (VK_SPACE) as _ {
                    SendMessageW(hWnd, BM_SETSTATE, (TRUE) as WPARAM, 0);
                    (*infoPtr).state |= BUTTON_BTNPRESSED;
                    SetCapture(hWnd);
                }
                break 'sw;
            }

            WM_LBUTTONDBLCLK => {
                if ((style & (BS_NOTIFY as i32)) != 0)
                    || btn_type == BS_RADIOBUTTON
                    || btn_type == BS_USERBUTTON
                    || btn_type == BS_OWNERDRAW
                {
                    BUTTON_NOTIFY_PARENT!(hWnd, BN_DOUBLECLICKED);
                    break 'sw;
                }
                /* fall through */
                SetCapture(hWnd);
                SetFocus(hWnd);
                (*infoPtr).state |= BUTTON_BTNPRESSED;
                SendMessageW(hWnd, BM_SETSTATE, (TRUE) as WPARAM, 0);
                break 'sw;
            }

            WM_LBUTTONDOWN => {
                SetCapture(hWnd);
                SetFocus(hWnd);
                (*infoPtr).state |= BUTTON_BTNPRESSED;
                SendMessageW(hWnd, BM_SETSTATE, (TRUE) as WPARAM, 0);
                break 'sw;
            }

            WM_KEYUP => {
                if wParam != (VK_SPACE) as _ {
                    break 'sw;
                }
                /* fall through */
                state = (*infoPtr).state;
                if (state & (BUTTON_BTNPRESSED as i32)) == 0 {
                    break 'sw;
                }
                (*infoPtr).state &= BUTTON_NSTATES;
                if (state & (BST_PUSHED as i32)) == 0 {
                    ReleaseCapture();
                    break 'sw;
                }
                SendMessageW(hWnd, BM_SETSTATE, (FALSE) as WPARAM, 0);
                GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                if uMsg == WM_KEYUP || PtInRect(core::ptr::addr_of!(rect), pt) != 0 {
                    match btn_type {
                        BS_AUTOCHECKBOX => {
                            SendMessageW(
                                hWnd,
                                BM_SETCHECK,
                                ((((*infoPtr).state & (BST_CHECKED as i32)) == 0) as WPARAM),
                                0,
                            );
                        }
                        BS_AUTORADIOBUTTON => {
                            BUTTON_CheckAutoRadioButton(hWnd);
                        }
                        BS_AUTO3STATE => {
                            SendMessageW(
                                hWnd,
                                BM_SETCHECK,
                                (if ((*infoPtr).state & (BST_INDETERMINATE as i32)) != 0 {
                                    0 as WPARAM
                                } else {
                                    (((*infoPtr).state & 3) + 1) as WPARAM
                                }),
                                0,
                            );
                        }
                        _ => {}
                    }
                    // Fix CORE-10194, Notify parent after capture is released.
                    ReleaseCapture();
                    BUTTON_NOTIFY_PARENT!(hWnd, BN_CLICKED);
                } else {
                    ReleaseCapture();
                }
                break 'sw;
            }

            WM_LBUTTONUP => {
                state = (*infoPtr).state;
                if (state & (BUTTON_BTNPRESSED as i32)) == 0 {
                    break 'sw;
                }
                (*infoPtr).state &= BUTTON_NSTATES;
                if (state & (BST_PUSHED as i32)) == 0 {
                    ReleaseCapture();
                    break 'sw;
                }
                SendMessageW(hWnd, BM_SETSTATE, (FALSE) as WPARAM, 0);
                GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                if uMsg == WM_KEYUP || PtInRect(core::ptr::addr_of!(rect), pt) != 0 {
                    match btn_type {
                        BS_AUTOCHECKBOX => {
                            SendMessageW(
                                hWnd,
                                BM_SETCHECK,
                                ((((*infoPtr).state & (BST_CHECKED as i32)) == 0) as WPARAM),
                                0,
                            );
                        }
                        BS_AUTORADIOBUTTON => {
                            BUTTON_CheckAutoRadioButton(hWnd);
                        }
                        BS_AUTO3STATE => {
                            SendMessageW(
                                hWnd,
                                BM_SETCHECK,
                                (if ((*infoPtr).state & (BST_INDETERMINATE as i32)) != 0 {
                                    0 as WPARAM
                                } else {
                                    (((*infoPtr).state & 3) + 1) as WPARAM
                                }),
                                0,
                            );
                        }
                        _ => {}
                    }
                    // Fix CORE-10194, Notify parent after capture is released.
                    ReleaseCapture();
                    BUTTON_NOTIFY_PARENT!(hWnd, BN_CLICKED);
                } else {
                    ReleaseCapture();
                }
                break 'sw;
            }

            WM_CAPTURECHANGED => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/button.c\0".as_ptr() as *const ::core::ffi::c_char,
                        b"BUTTON_WindowProc\0".as_ptr() as *const ::core::ffi::c_char,
                        832 as ::core::ffi::c_int,
                        b"WM_CAPTURECHANGED %p\n\0".as_ptr() as *const ::core::ffi::c_char,
                        hWnd,
                    );
                }
                if hWnd == lParam as HWND {
                    break 'sw;
                }
                if ((*infoPtr).state & (BUTTON_BTNPRESSED as i32)) != 0 {
                    (*infoPtr).state &= BUTTON_NSTATES;
                    if ((*infoPtr).state & (BST_PUSHED as i32)) != 0 {
                        SendMessageW(hWnd, BM_SETSTATE, (FALSE) as WPARAM, 0);
                    }
                }
                break 'sw;
            }

            WM_MOUSEMOVE => {
                let mut mouse_event: TRACKMOUSEEVENT = core::mem::zeroed::<TRACKMOUSEEVENT>();
                mouse_event.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
                mouse_event.dwFlags = TME_QUERY;

                if ((*infoPtr).state & (BST_HOT as i32)) == 0 {
                    let mut nmhotitem: NMBCHOTITEM = core::mem::zeroed::<NMBCHOTITEM>();

                    (*infoPtr).state |= (BST_HOT as i32);

                    nmhotitem.hdr.hwndFrom = hWnd;
                    nmhotitem.hdr.idFrom = GetWindowLongPtrW(hWnd, GWLP_ID) as UINT_PTR;
                    nmhotitem.hdr.code = (BCN_HOTITEMCHANGE) as u32;
                    nmhotitem.dwFlags = HICF_ENTERING;
                    SendMessageW(
                        GetParent(hWnd),
                        WM_NOTIFY,
                        nmhotitem.hdr.idFrom,
                        core::ptr::addr_of_mut!(nmhotitem) as LPARAM,
                    );

                    theme = GetWindowTheme(hWnd);
                    if !theme.is_null() {
                        InvalidateRect(hWnd, core::ptr::null::<RECT>(), TRUE);
                    }
                }

                if TrackMouseEvent(core::ptr::addr_of_mut!(mouse_event)) == 0
                    || (mouse_event.dwFlags & TME_LEAVE) == 0
                {
                    mouse_event.dwFlags = TME_LEAVE;
                    mouse_event.hwndTrack = hWnd;
                    mouse_event.dwHoverTime = 1;
                    TrackMouseEvent(core::ptr::addr_of_mut!(mouse_event));
                }

                if (wParam & (MK_LBUTTON as WPARAM)) != 0 && GetCapture() == hWnd {
                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                    SendMessageW(
                        hWnd,
                        BM_SETSTATE,
                        PtInRect(core::ptr::addr_of!(rect), pt) as WPARAM,
                        0,
                    );
                }
                break 'sw;
            }

            WM_MOUSELEAVE => {
                if ((*infoPtr).state & (BST_HOT as i32)) != 0 {
                    let mut nmhotitem: NMBCHOTITEM = core::mem::zeroed::<NMBCHOTITEM>();

                    (*infoPtr).state &= (!BST_HOT) as i32;

                    nmhotitem.hdr.hwndFrom = hWnd;
                    nmhotitem.hdr.idFrom = GetWindowLongPtrW(hWnd, GWLP_ID) as UINT_PTR;
                    nmhotitem.hdr.code = (BCN_HOTITEMCHANGE) as u32;
                    nmhotitem.dwFlags = HICF_LEAVING;
                    SendMessageW(
                        GetParent(hWnd),
                        WM_NOTIFY,
                        nmhotitem.hdr.idFrom,
                        core::ptr::addr_of_mut!(nmhotitem) as LPARAM,
                    );

                    theme = GetWindowTheme(hWnd);
                    if !theme.is_null() {
                        InvalidateRect(hWnd, core::ptr::null::<RECT>(), TRUE);
                    }
                }
                break 'sw;
            }

            BCM_GETTEXTMARGIN => {
                let prc: *mut RECT = lParam as *mut RECT;
                if prc.is_null() {
                    return FALSE as LRESULT;
                }
                *prc = (*infoPtr).rcTextMargin;
                return TRUE as LRESULT;
            }

            BCM_SETTEXTMARGIN => {
                let prc: *mut RECT = lParam as *mut RECT;
                if prc.is_null() {
                    return FALSE as LRESULT;
                }
                (*infoPtr).rcTextMargin = *prc;
                return TRUE as LRESULT;
            }

            BCM_SETIMAGELIST => {
                let pimldata: *mut BUTTON_IMAGELIST = lParam as *mut BUTTON_IMAGELIST;
                if pimldata.is_null() || (*pimldata).himl.is_null() {
                    return FALSE as LRESULT;
                }
                (*infoPtr).imlData = *pimldata;
                return TRUE as LRESULT;
            }

            BCM_GETIMAGELIST => {
                let pimldata: *mut BUTTON_IMAGELIST = lParam as *mut BUTTON_IMAGELIST;
                if pimldata.is_null() {
                    return FALSE as LRESULT;
                }
                *pimldata = (*infoPtr).imlData;
                return TRUE as LRESULT;
            }

            BCM_GETIDEALSIZE => {
                let theme_0: HTHEME = GetWindowTheme(hWnd);
                let mut ret: BOOL = FALSE;
                let pSize: *mut SIZE = lParam as *mut SIZE;

                if pSize.is_null() {
                    return FALSE as LRESULT;
                }

                if btn_type == BS_PUSHBUTTON
                    || btn_type == BS_DEFPUSHBUTTON
                    || btn_type == BS_USERBUTTON
                {
                    ret = BUTTON_GetIdealSize(infoPtr, theme_0, pSize);
                }

                if ret == 0 {
                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                    (*pSize).cx = rect.right;
                    (*pSize).cy = rect.bottom;
                }

                return TRUE as LRESULT;
            }

            WM_SETTEXT => {
                /* Clear an old text here as Windows does */
                //
                // ReactOS Note :
                // wine Bug: http://bugs.winehq.org/show_bug.cgi?id=25790
                // Patch: http://source.winehq.org/patches/data/70889
                // By: Alexander LAW, Replicate Windows behavior of WM_SETTEXT handler regarding WM_CTLCOLOR*
                //
                if (style & (WS_VISIBLE as i32)) != 0 {
                    let hdc: HDC = GetDC(hWnd);
                    let mut hbrush: HBRUSH = core::ptr::null_mut();
                    let mut client: RECT = core::mem::zeroed::<RECT>();
                    let mut rc: RECT = core::mem::zeroed::<RECT>();
                    let mut parent: HWND = GetParent(hWnd);
                    let message: UINT = (if btn_type == BS_PUSHBUTTON
                        || btn_type == BS_DEFPUSHBUTTON
                        || btn_type == BS_USERBUTTON
                        || btn_type == BS_OWNERDRAW
                    {
                        WM_CTLCOLORBTN
                    } else {
                        WM_CTLCOLORSTATIC
                    }) as UINT;

                    if parent.is_null() {
                        parent = hWnd;
                    }
                    hbrush = SendMessageW(parent, message, hdc as WPARAM, hWnd as LPARAM) as HBRUSH;
                    if hbrush.is_null() {
                        /* did the app forget to call DefWindowProc ? */
                        hbrush = DefWindowProcW(parent, message, hdc as WPARAM, hWnd as LPARAM) as HBRUSH;
                    }

                    GetClientRect(hWnd, core::ptr::addr_of_mut!(client));
                    rc = client;
                    /* FIXME: check other BS_* handlers */
                    if btn_type == BS_GROUPBOX {
                        InflateRect(core::ptr::addr_of_mut!(rc), -(7), 1); /* GB_Paint does this */
                    }
                    BUTTON_CalcLabelRect(infoPtr, hdc, core::ptr::addr_of_mut!(rc));
                    /* Clip by client rect bounds */
                    if rc.right > client.right {
                        rc.right = client.right;
                    }
                    if rc.bottom > client.bottom {
                        rc.bottom = client.bottom;
                    }
                    FillRect(hdc, core::ptr::addr_of_mut!(rc) as LPCRECT, hbrush);
                    ReleaseDC(hWnd, hdc);
                }

                DefWindowProcW(hWnd, WM_SETTEXT, wParam, lParam);
                if btn_type == BS_GROUPBOX {
                    /* Yes, only for BS_GROUPBOX */
                    InvalidateRect(hWnd, core::ptr::null::<RECT>(), TRUE);
                } else {
                    paint_button(infoPtr, btn_type as LONG, ODA_DRAWENTIRE);
                }
                return 1 as LRESULT; /* success. FIXME: check text length */
            }

            BCM_SETNOTE => {
                let note: *mut WCHAR = lParam as *mut WCHAR;
                if btn_type != BS_COMMANDLINK && btn_type != BS_DEFCOMMANDLINK {
                    SetLastError(ERROR_NOT_SUPPORTED);
                    return FALSE as LRESULT;
                }

                heap_free((*infoPtr).note as *mut ::core::ffi::c_void);
                if !note.is_null() {
                    (*infoPtr).note_length = lstrlenW(note as LPCWSTR) as INT;
                    (*infoPtr).note = heap_strndupW(note, (*infoPtr).note_length as size_t);
                }

                if note.is_null() || (*infoPtr).note.is_null() {
                    (*infoPtr).note_length = 0;
                    (*infoPtr).note = heap_alloc_zero(core::mem::size_of::<WCHAR>() as SIZE_T) as *mut WCHAR;
                }

                SetLastError(NO_ERROR);
                return TRUE as LRESULT;
            }

            BCM_GETNOTE => {
                let size: *mut DWORD = wParam as *mut DWORD;
                let buffer: *mut WCHAR = lParam as *mut WCHAR;
                let mut length: INT = 0;

                if btn_type != BS_COMMANDLINK && btn_type != BS_DEFCOMMANDLINK {
                    SetLastError(ERROR_NOT_SUPPORTED);
                    return FALSE as LRESULT;
                }

                if buffer.is_null() || size.is_null() || (*infoPtr).note.is_null() {
                    SetLastError(ERROR_INVALID_PARAMETER);
                    return FALSE as LRESULT;
                }

                if *size > 0 {
                    length = min(*size - 1, (*infoPtr).note_length as DWORD) as INT;
                    memcpy(
                        buffer as *mut ::core::ffi::c_void,
                        (*infoPtr).note as *const ::core::ffi::c_void,
                        (length as size_t) * core::mem::size_of::<WCHAR>(),
                    );
                    *buffer.offset(length as isize) = 0 as WCHAR; // '\0'
                }

                if *size < (((*infoPtr).note_length + 1) as DWORD) {
                    *size = ((*infoPtr).note_length + 1) as DWORD;
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                    return FALSE as LRESULT;
                } else {
                    SetLastError(NO_ERROR);
                    return TRUE as LRESULT;
                }
            }

            BCM_GETNOTELENGTH => {
                if btn_type != BS_COMMANDLINK && btn_type != BS_DEFCOMMANDLINK {
                    SetLastError(ERROR_NOT_SUPPORTED);
                    return 0 as LRESULT;
                }

                return (*infoPtr).note_length as LRESULT;
            }

            WM_SETFONT => {
                (*infoPtr).font = wParam as HFONT;
                if lParam != 0 {
                    InvalidateRect(hWnd, core::ptr::null::<RECT>(), TRUE);
                }
                break 'sw;
            }

            WM_GETFONT => {
                return (*infoPtr).font as LRESULT;
            }

            WM_SETFOCUS => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/button.c\0".as_ptr() as *const ::core::ffi::c_char,
                        b"BUTTON_WindowProc\0".as_ptr() as *const ::core::ffi::c_char,
                        1127 as ::core::ffi::c_int,
                        b"WM_SETFOCUS %p\n\0".as_ptr() as *const ::core::ffi::c_char,
                        hWnd,
                    );
                }
                (*infoPtr).state |= (BST_FOCUS) as i32;
                if btn_type != BS_OWNERDRAW {
                    InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE);
                } else {
                    paint_button(infoPtr, btn_type as LONG, ODA_FOCUS);
                }
                if (style & (BS_NOTIFY as i32)) != 0 {
                    BUTTON_NOTIFY_PARENT!(hWnd, BN_SETFOCUS);
                }
                if ((btn_type == BS_RADIOBUTTON) || (btn_type == BS_AUTORADIOBUTTON))
                    && ((*infoPtr).state & (BST_CHECKED as i32)) == 0
                {
                    BUTTON_NOTIFY_PARENT!(hWnd, BN_CLICKED);
                }
                break 'sw;
            }

            WM_KILLFOCUS => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/comctl32/button.c\0".as_ptr() as *const ::core::ffi::c_char,
                        b"BUTTON_WindowProc\0".as_ptr() as *const ::core::ffi::c_char,
                        1147 as ::core::ffi::c_int,
                        b"WM_KILLFOCUS %p\n\0".as_ptr() as *const ::core::ffi::c_char,
                        hWnd,
                    );
                }
                (*infoPtr).state &= (!BST_FOCUS) as i32;

                if ((*infoPtr).state & (BUTTON_BTNPRESSED as i32)) != 0 && GetCapture() == hWnd {
                    ReleaseCapture();
                }
                if (style & (BS_NOTIFY as i32)) != 0 {
                    BUTTON_NOTIFY_PARENT!(hWnd, BN_KILLFOCUS);
                }

                InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE);
                break 'sw;
            }

            WM_SYSCOLORCHANGE => {
                InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE);
                break 'sw;
            }

            BM_SETSTYLE => {
                btn_type = (wParam & (BS_TYPEMASK as WPARAM)) as UINT;
                style = (style & (!BS_TYPEMASK) as i32) | (btn_type as LONG);
                SetWindowLongW(hWnd, GWL_STYLE, style);

                /* Only redraw if lParam flag is set.*/
                if lParam != 0 {
                    InvalidateRect(hWnd, core::ptr::null::<RECT>(), TRUE);
                }

                break 'sw;
            }

            BM_CLICK => {
                /* Fix for core CORE-6024 */
                if ((*infoPtr).state & (BUTTON_BMCLICK as i32)) != 0 {
                    break 'sw;
                }
                (*infoPtr).state |= (BUTTON_BMCLICK as i32);
                SendMessageW(hWnd, WM_LBUTTONDOWN, 0, 0);
                SendMessageW(hWnd, WM_LBUTTONUP, 0, 0);
                (*infoPtr).state &= !BUTTON_BMCLICK;
                break 'sw;
            }

            BM_SETIMAGE => {
                /* Check that image format matches button style */
                match (style & ((BS_BITMAP | BS_ICON)) as i32) as u32 {
                    BS_BITMAP => {
                        if wParam != (IMAGE_BITMAP) as _ {
                            return 0 as LRESULT;
                        }
                    }
                    BS_ICON => {
                        if wParam != (IMAGE_ICON) as _ {
                            return 0 as LRESULT;
                        }
                    }
                    _ => {
                        return 0 as LRESULT;
                    }
                }
                oldHbitmap = (*infoPtr).u.image;
                (*infoPtr).u.image = lParam as HANDLE;
                InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE);
                return oldHbitmap as LRESULT;
            }

            BM_GETIMAGE => {
                return (*infoPtr).u.image as LRESULT;
            }

            BM_GETCHECK => {
                return ((*infoPtr).state & 3) as LRESULT;
            }

            BM_SETCHECK => {
                if wParam > maxCheckState[btn_type as usize] as WPARAM {
                    wParam = maxCheckState[btn_type as usize] as WPARAM;
                }
                if (btn_type == BS_RADIOBUTTON) || (btn_type == BS_AUTORADIOBUTTON) {
                    style = if wParam != 0 {
                        style | (WS_TABSTOP) as i32
                    } else {
                        style & (!WS_TABSTOP) as i32
                    };
                    SetWindowLongW(hWnd, GWL_STYLE, style);
                }
                if (((*infoPtr).state & 3) as WPARAM) != wParam {
                    (*infoPtr).state = ((((*infoPtr).state & !3) as WPARAM) | wParam) as LONG;
                    InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE);
                }
                break 'sw;
            }

            BM_GETSTATE => {
                return (*infoPtr).state as LRESULT;
            }

            BM_SETSTATE => {
                state = (*infoPtr).state;
                new_state = (if wParam != 0 { BST_PUSHED } else { 0 }) as LONG;

                if ((state ^ new_state) & (BST_PUSHED as i32)) != 0 {
                    if wParam != 0 {
                        state |= (BST_PUSHED) as i32;
                    } else {
                        state &= (!BST_PUSHED) as i32;
                    }

                    if btn_type == BS_USERBUTTON {
                        BUTTON_NOTIFY_PARENT!(
                            hWnd,
                            (if (state & (BST_PUSHED) as i32) != 0 { BN_HILITE } else { BN_UNHILITE }),
                        );
                    }
                    (*infoPtr).state = state;

                    InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE);
                }
                break 'sw;
            }

            WM_UPDATEUISTATE => {
                DefWindowProcW(hWnd, uMsg, wParam, lParam);

                if button_update_uistate(infoPtr) != 0 {
                    paint_button(infoPtr, btn_type as LONG, ODA_DRAWENTIRE);
                }
                break 'sw;
            }

            WM_NCHITTEST => {
                if btn_type == BS_GROUPBOX {
                    return HTTRANSPARENT as LRESULT;
                }
                /* fall through */
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }

            _ => {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn BUTTON_CalcLabelRect(mut infoPtr: *const BUTTON_INFO, mut hdc: HDC, mut rc: *mut RECT) -> UINT {
   let mut style: LONG = GetWindowLongW( (*infoPtr).hwnd, GWL_STYLE as core::ffi::c_int );
   let mut ex_style: LONG = GetWindowLongW( (*infoPtr).hwnd, GWL_EXSTYLE as core::ffi::c_int );
   let mut text: *mut WCHAR = core::ptr::null_mut();
   let mut iconInfo: ICONINFO = core::mem::zeroed::<ICONINFO>();
   let mut bm: BITMAP = core::mem::zeroed::<BITMAP>();
   let mut dtStyle: UINT = BUTTON_BStoDT( style as DWORD, ex_style as DWORD );
   let mut r: RECT = *rc;
   let mut n: INT = 0;
    let mut bHasIml: BOOL = BUTTON_DrawIml(hdc, core::ptr::addr_of!((*infoPtr).imlData), core::ptr::addr_of_mut!(r), TRUE, 0 as core::ffi::c_int);

   /* Calculate label rectangle according to label type */
   let mut empty: bool = false;
   let sw: LONG = style & ((BS_ICON | BS_BITMAP) as LONG);
   if sw == (BS_TEXT as LONG)
   {
      let mut hFont: HFONT = core::ptr::null_mut();
      let mut hPrevFont: HFONT = core::ptr::null_mut();

      text = get_button_text( infoPtr );
      if text.is_null() {
          empty = true;
      } else if *text.offset(0 as isize) == 0 {
          heap_free( text as *mut core::ffi::c_void );
          empty = true;
      } else {
          hFont = (*infoPtr).font;
          if !hFont.is_null() { hPrevFont = SelectObject( hdc, hFont as HGDIOBJ ) as HFONT; }
          DrawTextW(hdc, text as LPCWSTR, -(1 as core::ffi::c_int), core::ptr::addr_of_mut!(r), ((dtStyle | (DT_CALCRECT as UINT)) & !((DT_VCENTER | DT_BOTTOM) as UINT)));
          if !hPrevFont.is_null() { SelectObject( hdc, hPrevFont as HGDIOBJ ); }
          heap_free( text as *mut core::ffi::c_void );
          if ((*infoPtr).ui_state & (UISF_HIDEACCEL as DWORD)) != 0 {
              dtStyle |= DT_HIDEPREFIX as UINT;
          }
      }
   }
   else if sw == (BS_ICON as LONG)
   {
      if GetIconInfo((*infoPtr).u.icon, core::ptr::addr_of_mut!(iconInfo)) == 0 {
         empty = true;
      } else {
         GetObjectW (iconInfo.hbmColor as HANDLE, core::mem::size_of::<BITMAP>() as core::ffi::c_int, core::ptr::addr_of_mut!(bm) as LPVOID);

         r.right  = r.left + bm.bmWidth;
         r.bottom = r.top  + bm.bmHeight;

         DeleteObject(iconInfo.hbmColor as HGDIOBJ);
         DeleteObject(iconInfo.hbmMask as HGDIOBJ);
      }
   }
   else if sw == (BS_BITMAP as LONG)
   {
      if GetObjectW( (*infoPtr).u.bitmap as HANDLE, core::mem::size_of::<BITMAP>() as core::ffi::c_int, core::ptr::addr_of_mut!(bm) as LPVOID) == 0 {
         empty = true;
      } else {
         r.right  = r.left + bm.bmWidth;
         r.bottom = r.top  + bm.bmHeight;
      }
   }
   else
   {
      empty = true;
   }

   if empty {
      if bHasIml != 0 {
      } else {
         (*rc).right = r.left;
         (*rc).bottom = r.top;
         return (-(1 as core::ffi::c_int)) as UINT;
      }
   }

   if bHasIml != 0 {
     if (*infoPtr).imlData.uAlign == (BUTTON_IMAGELIST_ALIGN_LEFT as UINT) {
         r.left = (*infoPtr).imlData.margin.left;
     } else if (*infoPtr).imlData.uAlign == (BUTTON_IMAGELIST_ALIGN_RIGHT as UINT) {
         r.right = (*infoPtr).imlData.margin.right;
     } else if (*infoPtr).imlData.uAlign == (BUTTON_IMAGELIST_ALIGN_TOP as UINT) {
         r.top = (*infoPtr).imlData.margin.top;
     } else if (*infoPtr).imlData.uAlign == (BUTTON_IMAGELIST_ALIGN_BOTTOM as UINT) {
         r.bottom = (*infoPtr).imlData.margin.bottom;
     }
   }

   /* Position label inside bounding rectangle according to
    * alignment flags. (calculated rect is always left-top aligned).
    * If label is aligned to any side - shift label in opposite
    * direction to leave extra space for focus rectangle.
    */
   let sw2: UINT = dtStyle & ((DT_CENTER | DT_RIGHT) as UINT);
   if sw2 == (DT_LEFT as UINT) {
      r.left += 1;
      r.right += 1;
   } else if sw2 == (DT_CENTER as UINT) {
      n = r.right - r.left;
      r.left = (*rc).left + (((*rc).right - (*rc).left) - n) / (2 as LONG);
      r.right = r.left + n;
   } else if sw2 == (DT_RIGHT as UINT) {
      n = r.right - r.left;
      r.right = (*rc).right - (1 as LONG);
      r.left = r.right - n;
   }

   let sw3: UINT = dtStyle & ((DT_VCENTER | DT_BOTTOM) as UINT);
   if sw3 == (DT_TOP as UINT) {
      r.top += 1;
      r.bottom += 1;
   } else if sw3 == (DT_VCENTER as UINT) {
      n = r.bottom - r.top;
      r.top = (*rc).top + (((*rc).bottom - (1 as LONG) - (*rc).top) - n) / (2 as LONG);
      r.bottom = r.top + n;
   } else if sw3 == (DT_BOTTOM as UINT) {
      n = r.bottom - r.top;
      r.bottom = (*rc).bottom - (1 as LONG);
      r.top = r.bottom - n;
   }

   *rc = r;
   return dtStyle;
}

unsafe extern "system" fn BUTTON_DrawTextCallback(hdc: HDC, lp: LPARAM, wp: WPARAM, cx: i32, cy: i32) -> BOOL {
    let mut rc: RECT = core::mem::zeroed();

    SetRect(core::ptr::addr_of_mut!(rc), 0, 0, cx, cy);
    DrawTextW(hdc, lp as LPCWSTR, -1, core::ptr::addr_of_mut!(rc), wp as UINT);
    return TRUE;
}

unsafe extern "C" fn BUTTON_DrawLabel(
    infoPtr: *const BUTTON_INFO,
    hdc: HDC,
    dtFlags: UINT,
    rc: *const RECT,
) {
    let mut lpOutputProc: DRAWSTATEPROC = None;
    let mut lp: LPARAM = 0;
    let mut wp: WPARAM = 0 as WPARAM;
    let mut hbr: HBRUSH = 0 as HBRUSH;
    let mut flags: UINT = (if IsWindowEnabled((*infoPtr).hwnd) != 0 {
        DSS_NORMAL
    } else {
        DSS_DISABLED
    }) as UINT;
    let mut state: LONG = (*infoPtr).state;
    let mut style: LONG = GetWindowLongW((*infoPtr).hwnd, GWL_STYLE);
    let mut text: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    let mut rcText: RECT = *rc;
    BUTTON_DrawIml(
        hdc,
        &raw const (*infoPtr).imlData,
        &raw mut rcText,
        FALSE,
        0,
    );

    if (style as core::ffi::c_int & BS_PUSHLIKE as core::ffi::c_int) != 0
        && (state as core::ffi::c_int & BST_INDETERMINATE as core::ffi::c_int) != 0
    {
        hbr = GetSysColorBrush((COLOR_GRAYTEXT) as i32);
        flags |= DSS_MONO as UINT;
    }

    match (style as core::ffi::c_int & (BS_ICON as core::ffi::c_int | BS_BITMAP as core::ffi::c_int)) as u32 {
        BS_TEXT => {
            /* DST_COMPLEX -- is 0 */
            lpOutputProc = Some(BUTTON_DrawTextCallback);
            text = get_button_text(infoPtr);
            if text.is_null() {
                return;
            }
            lp = text as LPARAM;
            wp = dtFlags as WPARAM;
            if (dtFlags & DT_HIDEPREFIX as UINT) != 0 {
                flags |= DSS_HIDEPREFIX as UINT;
            }
        }
        BS_ICON => {
            flags |= DST_ICON as UINT;
            lp = (*infoPtr).u.icon as LPARAM;
        }
        BS_BITMAP => {
            flags |= DST_BITMAP as UINT;
            lp = (*infoPtr).u.bitmap as LPARAM;
        }
        _ => return,
    }

    DrawStateW(
        hdc,
        hbr,
        lpOutputProc,
        lp,
        wp,
        rcText.left as core::ffi::c_int,
        rcText.top as core::ffi::c_int,
        rcText.right as core::ffi::c_int - rcText.left as core::ffi::c_int,
        rcText.bottom as core::ffi::c_int - rcText.top as core::ffi::c_int,
        flags,
    );
    heap_free(text as *mut core::ffi::c_void);
}

unsafe extern "C" fn PB_Paint(infoPtr: *const BUTTON_INFO, hDC: HDC, action: UINT) {
    let mut rc: RECT = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();
    let mut dtFlags: UINT = 0;
    let mut uState: UINT = 0;
    let mut hOldPen: HPEN = core::ptr::null_mut();
    let mut hpen: HPEN = core::ptr::null_mut();
    let mut hOldBrush: HBRUSH = core::ptr::null_mut();
    let mut oldBkMode: INT = 0;
    let mut oldTxtColor: COLORREF = 0;
    let mut hFont: HFONT = core::ptr::null_mut();
    let state: LONG = (*infoPtr).state;
    let style: LONG = GetWindowLongW((*infoPtr).hwnd, GWL_STYLE);
    let pushedState: BOOL = (state as BOOL & (BST_PUSHED) as i32);
    let mut parent: HWND = core::ptr::null_mut();
    let mut hrgn: HRGN = core::ptr::null_mut();
    let mut cdrf: DWORD;

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rc));

    /* Send WM_CTLCOLOR to allow changing the font (the colors are fixed) */
    hFont = (*infoPtr).font;
    if !hFont.is_null() {
        SelectObject(hDC, (hFont).cast());
    }
    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    SendMessageW(
        parent,
        WM_CTLCOLORBTN,
        hDC as WPARAM,
        (*infoPtr).hwnd as LPARAM,
    );

    hrgn = set_control_clipping(hDC, core::ptr::addr_of_mut!(rc));

    hpen = CreatePen(
        (PS_SOLID as i32),
        1 as ::core::ffi::c_int,
        GetSysColor((COLOR_WINDOWFRAME) as i32) as COLORREF,
    );
    hOldPen = SelectObject(hDC, hpen as HGDIOBJ) as HPEN;
    hOldBrush = SelectObject(
        hDC,
        GetSysColorBrush((COLOR_BTNFACE) as i32) as HGDIOBJ,
    ) as HBRUSH;
    oldBkMode = SetBkMode(hDC, (TRANSPARENT as i32)) as INT;

    cdrf = BUTTON_SendCustomDraw(
        infoPtr,
        hDC,
        CDDS_PREERASE as DWORD,
        core::ptr::addr_of_mut!(rc),
    );

    'cleanup: {
        if cdrf == CDRF_SKIPDEFAULT as DWORD {
            break 'cleanup;
        }

        if get_button_type(style) == BS_DEFPUSHBUTTON as UINT {
            if action != ODA_FOCUS as UINT {
                Rectangle(hDC, rc.left, rc.top, rc.right, rc.bottom);
            }
            InflateRect(
                core::ptr::addr_of_mut!(rc),
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            );
        }

        /* completely skip the drawing if only focus has changed */
        'draw_focus: {
            if action == ODA_FOCUS as UINT {
                break 'draw_focus;
            }

            uState = DFCS_BUTTONPUSH as UINT;

            if (style as ::core::ffi::c_int & BS_FLAT as ::core::ffi::c_int) != 0 {
                uState |= DFCS_MONO as UINT;
            } else if pushedState != 0 {
                if get_button_type(style) == BS_DEFPUSHBUTTON as UINT {
                    uState |= DFCS_FLAT as UINT;
                } else {
                    uState |= DFCS_PUSHED as UINT;
                }
            }

            if (state as ::core::ffi::c_int & (BST_CHECKED | BST_INDETERMINATE) as ::core::ffi::c_int) != 0 {
                uState |= DFCS_CHECKED as UINT;
            }

            DrawFrameControl(hDC, core::ptr::addr_of_mut!(rc), DFC_BUTTON as UINT, uState);

            if cdrf == CDRF_NOTIFYPOSTERASE as DWORD {
                BUTTON_SendCustomDraw(
                    infoPtr,
                    hDC,
                    CDDS_POSTERASE as DWORD,
                    core::ptr::addr_of_mut!(rc),
                );
            }

            cdrf = BUTTON_SendCustomDraw(
                infoPtr,
                hDC,
                CDDS_PREPAINT as DWORD,
                core::ptr::addr_of_mut!(rc),
            );
            if cdrf == CDRF_SKIPDEFAULT as DWORD {
                break 'cleanup;
            }

            /* draw button label */
            r = rc;
            dtFlags = BUTTON_CalcLabelRect(infoPtr, hDC, core::ptr::addr_of_mut!(r));

            if dtFlags == (-(1 as ::core::ffi::c_int)) as UINT {
                break 'cleanup;
            }

            if pushedState != 0 {
                OffsetRect(
                    core::ptr::addr_of_mut!(r),
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }

            oldTxtColor = SetTextColor(
                hDC,
                GetSysColor((COLOR_BTNTEXT) as i32) as COLORREF,
            );

            BUTTON_DrawLabel(infoPtr, hDC, dtFlags, core::ptr::addr_of_mut!(r));

            SetTextColor(hDC, oldTxtColor);

            if cdrf == CDRF_NOTIFYPOSTPAINT as DWORD {
                BUTTON_SendCustomDraw(
                    infoPtr,
                    hDC,
                    CDDS_POSTPAINT as DWORD,
                    core::ptr::addr_of_mut!(rc),
                );
            }
        }

        // draw_focus:
        if action == ODA_FOCUS as UINT
            || (state as ::core::ffi::c_int & BST_FOCUS as ::core::ffi::c_int) != 0
        {
            if ((*infoPtr).ui_state & UISF_HIDEFOCUS as DWORD) == 0 {
                InflateRect(
                    core::ptr::addr_of_mut!(rc),
                    -(2 as ::core::ffi::c_int),
                    -(2 as ::core::ffi::c_int),
                );
                DrawFocusRect(hDC, core::ptr::addr_of_mut!(rc));
            }
        }
    }

    // cleanup:
    SelectObject(hDC, hOldPen as HGDIOBJ);
    SelectObject(hDC, hOldBrush as HGDIOBJ);
    SetBkMode(hDC, oldBkMode);
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as HGDIOBJ);
    }
    DeleteObject(hpen as HGDIOBJ);
}

unsafe extern "C" fn CB_Paint(infoPtr: *const BUTTON_INFO, hDC: HDC, action: UINT) {
    let mut rbox: RECT = core::mem::zeroed();
    let mut rtext: RECT = core::mem::zeroed();
    let mut client: RECT = core::mem::zeroed();
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut delta: i32 = 0;
    let mut text_offset: i32 = 0;
    let mut checkBoxWidth: i32 = 0;
    let mut checkBoxHeight: i32 = 0;
    let mut dtFlags: UINT = 0;
    let mut hFont: HFONT = core::ptr::null_mut();
    let state: LONG = (*infoPtr).state;
    let style: LONG = GetWindowLongW((*infoPtr).hwnd, GWL_STYLE);
    let ex_style: LONG = GetWindowLongW((*infoPtr).hwnd, GWL_EXSTYLE);
    let mut parent: HWND = core::ptr::null_mut();
    let mut hrgn: HRGN = core::ptr::null_mut();

    if (style & (BS_PUSHLIKE as LONG)) != 0 {
        PB_Paint(infoPtr, hDC, action);
        return;
    }

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(client));
    rtext = client;
    rbox = rtext;

    checkBoxWidth = 12 * GetDeviceCaps(hDC, (LOGPIXELSX as i32)) / 96 + 1;
    checkBoxHeight = 12 * GetDeviceCaps(hDC, (LOGPIXELSY as i32)) / 96 + 1;

    hFont = (*infoPtr).font;
    if !hFont.is_null() {
        SelectObject(hDC, hFont as HGDIOBJ);
    }
    GetCharWidthW(hDC, '0' as UINT, '0' as UINT, core::ptr::addr_of_mut!(text_offset));
    text_offset /= 2;

    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    hBrush = SendMessageW(
        parent,
        WM_CTLCOLORSTATIC as UINT,
        hDC as WPARAM,
        (*infoPtr).hwnd as LPARAM,
    ) as HBRUSH;
    if hBrush.is_null() {
        hBrush = DefWindowProcW(
            parent,
            WM_CTLCOLORSTATIC as UINT,
            hDC as WPARAM,
            (*infoPtr).hwnd as LPARAM,
        ) as HBRUSH;
    }
    hrgn = set_control_clipping(hDC, core::ptr::addr_of_mut!(client));

    if (style & (BS_LEFTTEXT as LONG)) != 0 || (ex_style & (WS_EX_RIGHT as LONG)) != 0 {
        rtext.right -= checkBoxWidth + text_offset;
        rbox.left = rbox.right - checkBoxWidth;
    } else {
        rtext.left += checkBoxWidth + text_offset;
        rbox.right = checkBoxWidth;
    }

    /* Since WM_ERASEBKGND does nothing, first prepare background */
    if action == ODA_SELECT as UINT {
        FillRect(hDC, core::ptr::addr_of_mut!(rbox), hBrush);
    }
    if action == ODA_DRAWENTIRE as UINT {
        FillRect(hDC, core::ptr::addr_of_mut!(client), hBrush);
    }

    /* Draw label */
    client = rtext;
    dtFlags = BUTTON_CalcLabelRect(infoPtr, hDC, core::ptr::addr_of_mut!(rtext));

    /* Only adjust rbox when rtext is valid */
    if dtFlags != (-(1 as ::core::ffi::c_long)) as UINT {
        rbox.top = rtext.top;
        rbox.bottom = rtext.bottom;
    }

    /* Draw the check-box bitmap */
    if action == ODA_DRAWENTIRE as UINT || action == ODA_SELECT as UINT {
        let mut flags: UINT = 0;

        if (get_button_type(style) == BS_RADIOBUTTON as UINT)
            || (get_button_type(style) == BS_AUTORADIOBUTTON as UINT)
        {
            flags = DFCS_BUTTONRADIO as UINT;
        } else if (state & (BST_INDETERMINATE as LONG)) != 0 {
            flags = DFCS_BUTTON3STATE as UINT;
        } else {
            flags = DFCS_BUTTONCHECK as UINT;
        }

        if (state & ((BST_CHECKED as LONG) | (BST_INDETERMINATE as LONG))) != 0 {
            flags |= DFCS_CHECKED as UINT;
        }
        if (state & (BST_PUSHED as LONG)) != 0 {
            flags |= DFCS_PUSHED as UINT;
        }

        if (style & (WS_DISABLED as LONG)) != 0 {
            flags |= DFCS_INACTIVE as UINT;
        }

        /* rbox must have the correct height */
        delta = rbox.bottom - rbox.top - checkBoxHeight;

        if (style & (BS_TOP as LONG)) != 0 {
            if delta > 0 {
                rbox.bottom = rbox.top + checkBoxHeight;
            } else {
                rbox.top -= -delta / 2 + 1;
                rbox.bottom = rbox.top + checkBoxHeight;
            }
        } else if (style & (BS_BOTTOM as LONG)) != 0 {
            if delta > 0 {
                rbox.top = rbox.bottom - checkBoxHeight;
            } else {
                rbox.bottom += -delta / 2 + 1;
                rbox.top = rbox.bottom - checkBoxHeight;
            }
        } else {
            /* Default */
            if delta > 0 {
                let ofs: i32 = delta / 2;
                rbox.bottom -= ofs + 1;
                rbox.top = rbox.bottom - checkBoxHeight;
            } else if delta < 0 {
                let ofs: i32 = -delta / 2;
                rbox.top -= ofs + 1;
                rbox.bottom = rbox.top + checkBoxHeight;
            }
        }

        DrawFrameControl(hDC, core::ptr::addr_of_mut!(rbox), DFC_BUTTON as UINT, flags);
    }

    if dtFlags == (-(1 as ::core::ffi::c_long)) as UINT {
        /* Noting to draw */
        return;
    }

    if action == ODA_DRAWENTIRE as UINT {
        BUTTON_DrawLabel(infoPtr, hDC, dtFlags, core::ptr::addr_of_mut!(rtext));
    }

    /* ... and focus */
    if action == ODA_FOCUS as UINT || (state & (BST_FOCUS as LONG)) != 0 {
        rtext.left -= 1;
        rtext.right += 1;
        IntersectRect(
            core::ptr::addr_of_mut!(rtext),
            core::ptr::addr_of_mut!(rtext),
            core::ptr::addr_of_mut!(client),
        );
        DrawFocusRect(hDC, core::ptr::addr_of_mut!(rtext));
    }
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as HGDIOBJ);
    }
}

unsafe extern "C" fn BUTTON_CheckAutoRadioButton(mut hwnd: HWND) {
    let mut parent: HWND;
    let mut sibling: HWND;
    let mut start: HWND;
    parent = GetParent(hwnd);
    start = core::ptr::null_mut::<HWND__>();
    sibling = hwnd;
    start = sibling;
    loop {
        if sibling.is_null() {
            break;
        }
        if (SendMessageW(sibling, WM_GETDLGCODE as UINT, 0 as WPARAM, 0 as LPARAM)
            & (DLGC_BUTTON | DLGC_RADIOBUTTON) as LRESULT)
            == (DLGC_BUTTON | DLGC_RADIOBUTTON) as LRESULT
        {
            SendMessageW(
                sibling,
                BM_SETCHECK as UINT,
                (if sibling == hwnd {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as WPARAM,
                0 as LPARAM,
            );
        }
        sibling = GetNextDlgGroupItem(parent, sibling, FALSE);
        if !(sibling != start) {
            break;
        }
    }
}

unsafe extern "C" fn GB_Paint(infoPtr: *const BUTTON_INFO, hDC: HDC, action: UINT) {
    let mut rc: RECT = core::mem::zeroed();
    let mut rcFrame: RECT = core::mem::zeroed();
    let mut hbr: HBRUSH = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut dtFlags: UINT = 0;
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut style: LONG = GetWindowLongW((*infoPtr).hwnd, GWL_STYLE);
    let mut parent: HWND = core::ptr::null_mut();
    let mut hrgn: HRGN = core::ptr::null_mut();

    hFont = (*infoPtr).font;
    if !hFont.is_null() {
        SelectObject(hDC, hFont as HGDIOBJ);
    }
    /* GroupBox acts like static control, so it sends CTLCOLORSTATIC */
    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    hbr = SendMessageW(
        parent,
        WM_CTLCOLORSTATIC as UINT,
        hDC as WPARAM,
        (*infoPtr).hwnd as LPARAM,
    ) as HBRUSH;
    if hbr.is_null() {
        /* did the app forget to call defwindowproc ? */
        hbr = DefWindowProcW(
            parent,
            WM_CTLCOLORSTATIC as UINT,
            hDC as WPARAM,
            (*infoPtr).hwnd as LPARAM,
        ) as HBRUSH;
    }
    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rc));
    rcFrame = rc;
    hrgn = set_control_clipping(hDC, core::ptr::addr_of_mut!(rc));

    GetTextMetricsW(hDC, core::ptr::addr_of_mut!(tm));
    rcFrame.top += (tm.tmHeight / 2) - 1;
    DrawEdge(
        hDC,
        core::ptr::addr_of_mut!(rcFrame),
        EDGE_ETCHED as UINT,
        (BF_RECT
            | (if (style & (BS_FLAT as LONG)) != 0 {
                BF_FLAT
            } else {
                0
            })) as UINT,
    );

    InflateRect(core::ptr::addr_of_mut!(rc), -(7), 1);
    dtFlags = BUTTON_CalcLabelRect(infoPtr, hDC, core::ptr::addr_of_mut!(rc));

    if dtFlags != (-(1) as i32) as UINT {
        /* Because buttons have CS_PARENTDC class style, there is a chance
         * that label will be drawn out of client rect.
         * But Windows doesn't clip label's rect, so do I.
         */

        /* There is 1-pixel margin at the left, right, and bottom */
        rc.left -= 1;
        rc.right += 1;
        rc.bottom += 1;
        FillRect(hDC, core::ptr::addr_of_mut!(rc), hbr);
        rc.left += 1;
        rc.right -= 1;
        rc.bottom -= 1;

        BUTTON_DrawLabel(infoPtr, hDC, dtFlags, core::ptr::addr_of_mut!(rc));
    }
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as HGDIOBJ);
    }
}

unsafe extern "C" fn UB_Paint(infoPtr: *const BUTTON_INFO, hDC: HDC, action: UINT) {
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut state: LONG = (*infoPtr).state;
    let mut parent: HWND = core::ptr::null_mut();

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(rc));

    hFont = (*infoPtr).font;
    if !hFont.is_null() {
        SelectObject(hDC, (hFont).cast());
    }

    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    hBrush = SendMessageW(
        parent,
        WM_CTLCOLORBTN,
        hDC as WPARAM,
        (*infoPtr).hwnd as LPARAM,
    ) as HBRUSH;
    if hBrush.is_null() {
        hBrush = DefWindowProcW(
            parent,
            WM_CTLCOLORBTN,
            hDC as WPARAM,
            (*infoPtr).hwnd as LPARAM,
        ) as HBRUSH;
    }

    FillRect(hDC, core::ptr::addr_of_mut!(rc), hBrush);
    if action == ODA_FOCUS || (state & (BST_FOCUS) as i32) != 0 {
        DrawFocusRect(hDC, core::ptr::addr_of_mut!(rc));
    }

    if action == ODA_FOCUS {
        BUTTON_NOTIFY_PARENT!(
            (*infoPtr).hwnd,
            if (state & (BST_FOCUS) as i32) != 0 {
                BN_SETFOCUS
            } else {
                BN_KILLFOCUS
            },
        );
    } else if action == ODA_SELECT {
        BUTTON_NOTIFY_PARENT!(
            (*infoPtr).hwnd,
            if (state & (BST_PUSHED) as i32) != 0 {
                BN_HILITE
            } else {
                BN_UNHILITE
            },
        );
    } else {
    }
}

unsafe extern "C" fn OB_Paint(infoPtr: *const BUTTON_INFO, hDC: HDC, action: UINT) {
    let state: LONG = (*infoPtr).state;
    let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
    let id: LONG_PTR = GetWindowLongPtrW((*infoPtr).hwnd, GWLP_ID);
    let mut parent: HWND = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hrgn: HRGN = core::ptr::null_mut();

    dis.CtlType = ODT_BUTTON;
    dis.CtlID = id as UINT;
    dis.itemID = 0;
    dis.itemAction = action;
    dis.itemState = ((if ((state as UINT) & BST_FOCUS) != 0 {
        ODS_FOCUS
    } else {
        0
    }) | (if ((state as UINT) & BST_PUSHED) != 0 {
        ODS_SELECTED
    } else {
        0
    }) | (if IsWindowEnabled((*infoPtr).hwnd) != 0 {
        0
    } else {
        ODS_DISABLED
    })) as UINT;
    dis.hwndItem = (*infoPtr).hwnd;
    dis.hDC = hDC;
    dis.itemData = 0;
    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(dis.rcItem));

    hFont = (*infoPtr).font;
    if !hFont.is_null() {
        SelectObject(hDC, hFont as HGDIOBJ);
    }
    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    SendMessageW(
        parent,
        WM_CTLCOLORBTN as UINT,
        hDC as WPARAM,
        (*infoPtr).hwnd as LPARAM,
    );

    hrgn = set_control_clipping(hDC, core::ptr::addr_of_mut!(dis.rcItem));

    SendMessageW(
        GetParent((*infoPtr).hwnd),
        WM_DRAWITEM as UINT,
        id as WPARAM,
        core::ptr::addr_of_mut!(dis) as LPARAM,
    );
    SelectClipRgn(hDC, hrgn);
    if !hrgn.is_null() {
        DeleteObject(hrgn as HGDIOBJ);
    }
}

unsafe extern "C" fn PB_ThemedPaint(
    mut theme: HTHEME,
    mut infoPtr: *const BUTTON_INFO,
    mut hDC: HDC,
    mut drawState: ButtonState,
    mut dtFlags: UINT,
    mut focused: BOOL,
    mut prfFlag: LPARAM,
) {
    static states: [::core::ffi::c_int; 5] = [
        PBS_NORMAL as ::core::ffi::c_int,
        PBS_DISABLED as ::core::ffi::c_int,
        PBS_HOT as ::core::ffi::c_int,
        PBS_PRESSED as ::core::ffi::c_int,
        PBS_DEFAULTED as ::core::ffi::c_int,
    ];
    let mut bgRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut textRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut font: HFONT = (*infoPtr).font;
    let mut hPrevFont: HFONT = if !font.is_null() {
        SelectObject(hDC, font as HGDIOBJ) as HFONT
    } else {
        core::ptr::null_mut()
    };
    let mut state: ::core::ffi::c_int = states[drawState as usize];
    let mut text: *mut WCHAR = get_button_text(infoPtr);
    let mut parent: HWND = core::ptr::null_mut::<HWND__>();
    let mut cdrf: DWORD = 0;

    'cleanup: {
        GetClientRect((*infoPtr).hwnd, &raw mut bgRect);
        GetThemeBackgroundContentRect(
            theme,
            hDC,
            BP_PUSHBUTTON as ::core::ffi::c_int,
            state,
            &raw mut bgRect,
            &raw mut textRect,
        );

        if prfFlag == 0 as LPARAM {
            if IsThemeBackgroundPartiallyTransparent(
                theme,
                BP_PUSHBUTTON as ::core::ffi::c_int,
                state,
            ) != 0
            {
                DrawThemeParentBackground((*infoPtr).hwnd, hDC, core::ptr::null_mut::<RECT>());
            }
        }

        parent = GetParent((*infoPtr).hwnd);
        if parent.is_null() {
            parent = (*infoPtr).hwnd;
        }
        SendMessageW(
            parent,
            WM_CTLCOLORBTN as UINT,
            hDC as WPARAM,
            (*infoPtr).hwnd as LPARAM,
        );

        cdrf = BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_PREERASE as DWORD, &raw mut bgRect);
        if cdrf == CDRF_SKIPDEFAULT as DWORD {
            break 'cleanup;
        }

        DrawThemeBackground(
            theme,
            hDC,
            BP_PUSHBUTTON as ::core::ffi::c_int,
            state,
            &raw mut bgRect,
            core::ptr::null::<RECT>(),
        );

        if cdrf == CDRF_NOTIFYPOSTERASE as DWORD {
            BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_POSTERASE as DWORD, &raw mut bgRect);
        }

        cdrf = BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_PREPAINT as DWORD, &raw mut bgRect);
        if cdrf == CDRF_SKIPDEFAULT as DWORD {
            break 'cleanup;
        }

        BUTTON_DrawIml(
            hDC,
            &raw const (*infoPtr).imlData,
            &raw mut textRect,
            FALSE as BOOL,
            drawState as ::core::ffi::c_int,
        );

        if !text.is_null() {
            DrawThemeText(
                theme,
                hDC,
                BP_PUSHBUTTON as ::core::ffi::c_int,
                state,
                text as LPCWSTR,
                lstrlenW(text as LPCWSTR),
                dtFlags as DWORD,
                0 as DWORD,
                &raw mut textRect,
            );
            heap_free(text as *mut ::core::ffi::c_void);
            text = core::ptr::null_mut::<WCHAR>();
        }

        if focused != 0 {
            let mut margins: MARGINS = _MARGINS {
                cxLeftWidth: 0,
                cxRightWidth: 0,
                cyTopHeight: 0,
                cyBottomHeight: 0,
            };
            let mut focusRect: RECT = bgRect;

            GetThemeMargins(
                theme,
                hDC,
                BP_PUSHBUTTON as ::core::ffi::c_int,
                state,
                TMT_CONTENTMARGINS as ::core::ffi::c_int,
                core::ptr::null_mut::<RECT>(),
                &raw mut margins,
            );

            focusRect.left += margins.cxLeftWidth;
            focusRect.top += margins.cyTopHeight;
            focusRect.right -= margins.cxRightWidth;
            focusRect.bottom -= margins.cyBottomHeight;

            DrawFocusRect(hDC, &raw mut focusRect);
        }

        if cdrf == CDRF_NOTIFYPOSTPAINT as DWORD {
            BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_POSTPAINT as DWORD, &raw mut bgRect);
        }
    }
    if !text.is_null() {
        heap_free(text as *mut ::core::ffi::c_void);
    }
    if !hPrevFont.is_null() {
        SelectObject(hDC, hPrevFont as HGDIOBJ);
    }
}

unsafe extern "C" fn CB_ThemedPaint(theme: HTHEME, infoPtr: *const BUTTON_INFO, hDC: HDC, drawState: ButtonState, dtFlags: UINT, focused: BOOL, prfFlag: LPARAM) {
    static cb_states: [[::core::ffi::c_int; 5]; 3] = [
        [(CBS_UNCHECKEDNORMAL) as i32, (CBS_UNCHECKEDDISABLED) as i32, (CBS_UNCHECKEDHOT) as i32, (CBS_UNCHECKEDPRESSED) as i32, (CBS_UNCHECKEDNORMAL) as i32],
        [(CBS_CHECKEDNORMAL) as i32, (CBS_CHECKEDDISABLED) as i32, (CBS_CHECKEDHOT) as i32, (CBS_CHECKEDPRESSED) as i32, (CBS_CHECKEDNORMAL) as i32],
        [(CBS_MIXEDNORMAL) as i32, (CBS_MIXEDDISABLED) as i32, (CBS_MIXEDHOT) as i32, (CBS_MIXEDPRESSED) as i32, (CBS_MIXEDNORMAL) as i32],
    ];
    static rb_states: [[::core::ffi::c_int; 5]; 2] = [
        [(RBS_UNCHECKEDNORMAL) as i32, (RBS_UNCHECKEDDISABLED) as i32, (RBS_UNCHECKEDHOT) as i32, (RBS_UNCHECKEDPRESSED) as i32, (RBS_UNCHECKEDNORMAL) as i32],
        [(RBS_CHECKEDNORMAL) as i32, (RBS_CHECKEDDISABLED) as i32, (RBS_CHECKEDHOT) as i32, (RBS_CHECKEDPRESSED) as i32, (RBS_CHECKEDNORMAL) as i32],
    ];

    let mut sz: SIZE = SIZE { cx: 0, cy: 0 };
    let mut bgRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut textRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut font: HFONT = core::ptr::null_mut();
    let mut hPrevFont: HFONT = core::ptr::null_mut();
    let mut checkState: ::core::ffi::c_int = ((*infoPtr).state as ::core::ffi::c_int) & (3 as ::core::ffi::c_int);
    let mut dwStyle: DWORD = GetWindowLongW((*infoPtr).hwnd, GWL_STYLE) as DWORD;
    let mut btn_type: UINT = get_button_type(dwStyle as LONG);
    let mut part: ::core::ffi::c_int = if (btn_type == (BS_RADIOBUTTON as UINT)) || (btn_type == (BS_AUTORADIOBUTTON as UINT)) {
        BP_RADIOBUTTON as ::core::ffi::c_int
    } else {
        BP_CHECKBOX as ::core::ffi::c_int
    };
    let mut state: ::core::ffi::c_int = if part == (BP_CHECKBOX as ::core::ffi::c_int) {
        cb_states[checkState as usize][drawState as usize]
    } else {
        rb_states[checkState as usize][drawState as usize]
    };
    let mut text: *mut WCHAR = get_button_text(infoPtr);
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut created_font: BOOL = FALSE as BOOL;
    let mut parent: HWND = core::ptr::null_mut();
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut cdrf: DWORD = 0;

    let mut hr: HRESULT = GetThemeFont(theme, hDC, part, state, TMT_FONT as ::core::ffi::c_int, core::ptr::addr_of_mut!(lf));
    if hr >= (0 as HRESULT) {
        font = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
        if font.is_null() {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/button.c\0".as_ptr() as *const ::core::ffi::c_char,
                    b"CB_ThemedPaint\0".as_ptr() as *const ::core::ffi::c_char,
                    2040 as ::core::ffi::c_int,
                    b"Failed to create font\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        } else {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/button.c\0".as_ptr() as *const ::core::ffi::c_char,
                    b"CB_ThemedPaint\0".as_ptr() as *const ::core::ffi::c_char,
                    2042 as ::core::ffi::c_int,
                    b"font = %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    debugstr_w(core::ptr::addr_of_mut!(lf.lfFaceName) as *mut WCHAR),
                );
            }
            hPrevFont = SelectObject(hDC, font as HGDIOBJ) as HFONT;
            created_font = TRUE as BOOL;
        }
    } else {
        font = (*infoPtr).font;
        hPrevFont = SelectObject(hDC, font as HGDIOBJ) as HFONT;
    }

    if GetThemePartSize(theme, hDC, part, state, core::ptr::null_mut::<RECT>(), (TS_DRAW as ::core::ffi::c_int) as THEMESIZE, core::ptr::addr_of_mut!(sz)) < (0 as HRESULT) {
        sz.cy = 13 as LONG;
        sz.cx = sz.cy;
    }

    GetClientRect((*infoPtr).hwnd, core::ptr::addr_of_mut!(bgRect));

    if prfFlag == (0 as LPARAM) {
        DrawThemeParentBackground((*infoPtr).hwnd, hDC, core::ptr::null_mut::<RECT>());
    }

    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    hBrush = SendMessageW(parent, WM_CTLCOLORSTATIC as UINT, hDC as WPARAM, (*infoPtr).hwnd as LPARAM) as HBRUSH;
    if hBrush.is_null() {
        hBrush = DefWindowProcW(parent, WM_CTLCOLORSTATIC as UINT, hDC as WPARAM, (*infoPtr).hwnd as LPARAM) as HBRUSH;
    }
    FillRect(hDC, core::ptr::addr_of_mut!(bgRect) as LPCRECT, hBrush);

    cdrf = BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_PREERASE as DWORD, core::ptr::addr_of_mut!(bgRect));

    'cleanup: {
        if cdrf == (CDRF_SKIPDEFAULT as DWORD) {
            break 'cleanup;
        }

        GetThemeBackgroundContentRect(theme, hDC, part, state, core::ptr::addr_of_mut!(bgRect), core::ptr::addr_of_mut!(textRect));

        if (dtFlags & (DT_SINGLELINE as UINT)) != 0 {
            bgRect.top = bgRect.top + ((textRect.bottom - textRect.top - sz.cy) / (2 as LONG));
        }

        bgRect.bottom = bgRect.top + sz.cy;
        bgRect.right = bgRect.left + sz.cx;
        textRect.left = bgRect.right + (6 as LONG);

        DrawThemeBackground(theme, hDC, part, state, core::ptr::addr_of_mut!(bgRect), core::ptr::null_mut::<RECT>());

        if cdrf == (CDRF_NOTIFYPOSTERASE as DWORD) {
            BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_POSTERASE as DWORD, core::ptr::addr_of_mut!(bgRect));
        }

        cdrf = BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_PREPAINT as DWORD, core::ptr::addr_of_mut!(bgRect));
        if cdrf == (CDRF_SKIPDEFAULT as DWORD) {
            break 'cleanup;
        }

        if !text.is_null() {
            DrawThemeText(
                theme,
                hDC,
                part,
                state,
                text as LPCWSTR,
                lstrlenW(text as LPCWSTR),
                dtFlags as DWORD,
                0 as DWORD,
                core::ptr::addr_of_mut!(textRect),
            );

            if focused != 0 {
                let mut focusRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

                focusRect = textRect;

                DrawTextW(
                    hDC,
                    text as LPCWSTR,
                    lstrlenW(text as LPCWSTR),
                    core::ptr::addr_of_mut!(focusRect),
                    dtFlags | (DT_CALCRECT as UINT),
                );

                if focusRect.right < textRect.right {
                    focusRect.right += 1;
                }
                focusRect.bottom = textRect.bottom;

                DrawFocusRect(hDC, core::ptr::addr_of_mut!(focusRect) as LPCRECT);
            }

            heap_free(text as *mut ::core::ffi::c_void);
            text = core::ptr::null_mut();
        }

        if cdrf == (CDRF_NOTIFYPOSTPAINT as DWORD) {
            BUTTON_SendCustomDraw(infoPtr, hDC, CDDS_POSTPAINT as DWORD, core::ptr::addr_of_mut!(bgRect));
        }
    }

    if !text.is_null() {
        heap_free(text as *mut ::core::ffi::c_void);
    }
    if created_font != 0 {
        DeleteObject(font as HGDIOBJ);
    }
    if !hPrevFont.is_null() {
        SelectObject(hDC, hPrevFont as HGDIOBJ);
    }
}

unsafe extern "C" fn GB_ThemedPaint(
    mut theme: HTHEME,
    mut infoPtr: *const BUTTON_INFO,
    mut hDC: HDC,
    mut drawState: ButtonState,
    mut dtFlags: UINT,
    mut focused: BOOL,
    mut prfFlag: LPARAM,
) {
    static mut states: [::core::ffi::c_int; 5] = [
        GBS_NORMAL as ::core::ffi::c_int,
        GBS_DISABLED as ::core::ffi::c_int,
        GBS_NORMAL as ::core::ffi::c_int,
        GBS_NORMAL as ::core::ffi::c_int,
        GBS_NORMAL as ::core::ffi::c_int,
    ];
    let mut bgRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut textRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut contentRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut state: ::core::ffi::c_int = states[drawState as usize];
    let mut text: *mut WCHAR = get_button_text(infoPtr);
    let mut lf: LOGFONTW = tagLOGFONTW {
        lfHeight: 0,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: 0,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: 0,
        lfOutPrecision: 0,
        lfClipPrecision: 0,
        lfQuality: 0,
        lfPitchAndFamily: 0,
        lfFaceName: [0; 32],
    };
    let mut font: HFONT = ::core::ptr::null_mut::<HFONT__>();
    let mut hPrevFont: HFONT = ::core::ptr::null_mut::<HFONT__>();
    let mut created_font: BOOL = FALSE as BOOL;
    let mut parent: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut hBrush: HBRUSH = ::core::ptr::null_mut::<HBRUSH__>();
    let mut clientRect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hr: HRESULT = GetThemeFont(
        theme,
        hDC,
        BP_GROUPBOX as ::core::ffi::c_int,
        state,
        (TMT_FONT as i32),
        &raw mut lf,
    );
    if hr >= 0 as ::core::ffi::c_int {
        font = CreateFontIndirectW(&raw mut lf);
        if font.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/button.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"GB_ThemedPaint\0" as *const u8 as *const ::core::ffi::c_char,
                    2162 as ::core::ffi::c_int,
                    b"Failed to create font\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        } else {
            hPrevFont = SelectObject(hDC, font as HGDIOBJ) as HFONT;
            created_font = TRUE as BOOL;
        }
    } else {
        font = (*infoPtr).font;
        hPrevFont = SelectObject(hDC, font as HGDIOBJ) as HFONT;
    }
    GetClientRect((*infoPtr).hwnd, &raw mut bgRect);
    textRect = bgRect;
    if !text.is_null() {
        let mut textExtent: SIZE = tagSIZE { cx: 0, cy: 0 };
        GetTextExtentPoint32W(
            hDC,
            text as LPCWSTR,
            lstrlenW(text as LPCWSTR),
            &raw mut textExtent,
        );
        bgRect.top += textExtent.cy as ::core::ffi::c_int
            / 2 as ::core::ffi::c_int;
        textRect.left += 10 as ::core::ffi::c_int;
        textRect.bottom = textRect.top + textExtent.cy;
        textRect.right = (textRect.left as ::core::ffi::c_int
            + textExtent.cx as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as LONG;
        ExcludeClipRect(
            hDC,
            textRect.left as ::core::ffi::c_int,
            textRect.top as ::core::ffi::c_int,
            textRect.right as ::core::ffi::c_int,
            textRect.bottom as ::core::ffi::c_int,
        );
    }
    GetThemeBackgroundContentRect(
        theme,
        hDC,
        BP_GROUPBOX as ::core::ffi::c_int,
        state,
        &raw mut bgRect,
        &raw mut contentRect,
    );
    ExcludeClipRect(
        hDC,
        contentRect.left as ::core::ffi::c_int,
        contentRect.top as ::core::ffi::c_int,
        contentRect.right as ::core::ffi::c_int,
        contentRect.bottom as ::core::ffi::c_int,
    );
    if prfFlag == 0 as LPARAM {
        if IsThemeBackgroundPartiallyTransparent(
            theme,
            BP_GROUPBOX as ::core::ffi::c_int,
            state,
        ) != 0
        {
            DrawThemeParentBackground((*infoPtr).hwnd, hDC, ::core::ptr::null_mut::<RECT>());
        }
    }
    parent = GetParent((*infoPtr).hwnd);
    if parent.is_null() {
        parent = (*infoPtr).hwnd;
    }
    hBrush = SendMessageW(
        parent,
        WM_CTLCOLORSTATIC as UINT,
        hDC as WPARAM,
        (*infoPtr).hwnd as LPARAM,
    ) as HBRUSH;
    if hBrush.is_null() {
        hBrush = DefWindowProcW(
            parent,
            WM_CTLCOLORSTATIC as UINT,
            hDC as WPARAM,
            (*infoPtr).hwnd as LPARAM,
        ) as HBRUSH;
    }
    GetClientRect((*infoPtr).hwnd, &raw mut clientRect);
    FillRect(hDC, &raw mut clientRect as LPCRECT, hBrush);
    DrawThemeBackground(
        theme,
        hDC,
        BP_GROUPBOX as ::core::ffi::c_int,
        state,
        &raw mut bgRect,
        ::core::ptr::null::<RECT>(),
    );
    SelectClipRgn(hDC, ::core::ptr::null_mut::<HRGN__>());
    if !text.is_null() {
        InflateRect(
            &raw mut textRect,
            -(2 as ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        DrawThemeText(
            theme,
            hDC,
            BP_GROUPBOX as ::core::ffi::c_int,
            state,
            text as LPCWSTR,
            lstrlenW(text as LPCWSTR),
            0 as DWORD,
            0 as DWORD,
            &raw mut textRect,
        );
        heap_free(text as *mut ::core::ffi::c_void);
    }
    if created_font != 0 {
        DeleteObject(font as HGDIOBJ);
    }
    if !hPrevFont.is_null() {
        SelectObject(hDC, hPrevFont as HGDIOBJ);
    }
}
#[no_mangle]
pub unsafe extern "C" fn BUTTON_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    memset(
        &raw mut wndClass as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<WNDCLASSW>() as size_t,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS | CS_VREDRAW | CS_HREDRAW | CS_PARENTDC) as UINT;
    wndClass.lpfnWndProc = Some(BUTTON_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut BUTTON_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = core::ptr::null_mut::<HBRUSH__>();
    wndClass.lpszClassName = WC_BUTTONW.as_ptr() as LPCWSTR;
    RegisterClassW(&raw mut wndClass);
}

#[no_mangle]
pub unsafe extern "C" fn BUTTON_Unregister() {
    UnregisterClassW((WC_BUTTONW).as_ptr(), core::ptr::null_mut());
}
