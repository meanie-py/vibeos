use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::DefSubclassProc;
use crate::comctl32::commctrl::RemoveWindowSubclass;
use crate::comctl32::commctrl::SetWindowSubclass;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_updown);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const UPDOWN_CLASSW: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"m\0s\0c\0t\0l\0s\0_\0u\0p\0d\0o\0w\0n\x003\x002\0\0\0",
    )
};

pub const WC_EDITW: [::core::ffi::c_ushort; 5] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_ushort; 5]>(*b"E\0d\0i\0t\0\0\0") };

pub const WC_LISTBOXW: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"L\0i\0s\0t\0B\0o\0x\0\0\0")
};

// WINE_DEFAULT_DEBUG_CHANNEL(updown): debug channel declaration, no runtime equivalent emitted.

#[repr(C)]
pub struct UPDOWN_INFO {
    pub Self_: HWND,               // Self (Handle to this up-down control)
    pub Notify: HWND,              // Handle to the parent window
    pub dwStyle: DWORD,            // The GWL_STYLE for this window
    pub AccelCount: UINT,          // Number of elements in AccelVect
    pub AccelVect: *mut UDACCEL,   // Vector containing AccelCount elements
    pub AccelIndex: INT,           // Current accel index, -1 if not accel'ing
    pub Base: INT,                 // Base to display nr in the buddy window
    pub CurVal: INT,               // Current up-down value
    pub MinVal: INT,               // Minimum up-down value
    pub MaxVal: INT,               // Maximum up-down value
    pub Buddy: HWND,               // Handle to the buddy window
    pub BuddyType: INT,            // Remembers the buddy type BUDDY_TYPE_*
    pub Flags: INT,                // Internal Flags FLAG_*
    pub UnicodeFormat: BOOL,       // Marks the use of Unicode internally
}

pub const INITIAL_DELAY: i32 = 500;      // initial timer until auto-inc kicks in
pub const AUTOPRESS_DELAY: i32 = 250;    // time to keep arrow pressed on KEY_DOWN
pub const REPEAT_DELAY: i32 = 50;        // delay between auto-increments
pub const DEFAULT_WIDTH: i32 = 16;       // default width of the ctrl
pub const DEFAULT_XSEP: i32 = 0;         // default separation between buddy and ctrl
pub const DEFAULT_ADDTOP: i32 = 0;       // amount to extend above the buddy window
pub const DEFAULT_ADDBOT: i32 = 0;       // amount to extend below the buddy window
pub const DEFAULT_BUDDYBORDER: i32 = 2;  // Width/height of the buddy border
pub const DEFAULT_BUDDYSPACER: i32 = 2;  // Spacer between the buddy and the ctrl
pub const DEFAULT_BUDDYBORDER_THEMED: i32 = 1; // buddy border when theming is enabled
pub const DEFAULT_BUDDYSPACER_THEMED: i32 = 0; // buddy spacer when theming is enabled

pub const FLAG_INCR: i32 = 0x01;
pub const FLAG_DECR: i32 = 0x02;
pub const FLAG_MOUSEIN: i32 = 0x04;
pub const FLAG_PRESSED: i32 = 0x08;
pub const FLAG_BUDDYINT: i32 = 0x10;     // UDS_SETBUDDYINT was set on creation
pub const FLAG_ARROW: i32 = FLAG_INCR | FLAG_DECR;

pub const BUDDY_TYPE_UNKNOWN: i32 = 0;
pub const BUDDY_TYPE_LISTBOX: i32 = 1;
pub const BUDDY_TYPE_EDIT: i32 = 2;

pub const TIMER_AUTOREPEAT: i32 = 1;
pub const TIMER_ACCEL: i32 = 2;
pub const TIMER_AUTOPRESS: i32 = 3;

pub const BUDDY_SUBCLASSID: i32 = 1;


unsafe extern "C" fn UPDOWN_IsBuddyEdit(infoPtr: *const UPDOWN_INFO) -> i32 {
    return ((*infoPtr).BuddyType == BUDDY_TYPE_EDIT) as i32;
}

unsafe extern "C" fn UPDOWN_IsBuddyListbox(infoPtr: *const UPDOWN_INFO) -> BOOL {
    return ((*infoPtr).BuddyType == BUDDY_TYPE_LISTBOX) as BOOL;
}

unsafe extern "C" fn UPDOWN_InBounds(infoPtr: *const UPDOWN_INFO, val: i32) -> BOOL {
    if (*infoPtr).MaxVal > (*infoPtr).MinVal {
        return ((*infoPtr).MinVal <= val && val <= (*infoPtr).MaxVal) as i32;
    } else {
        return ((*infoPtr).MaxVal <= val && val <= (*infoPtr).MinVal) as i32;
    }
}

unsafe extern "C" fn UPDOWN_OffsetVal(infoPtr: *mut UPDOWN_INFO, mut delta: i32) -> BOOL {
    /* check if we can do the modification first */
    if UPDOWN_InBounds(infoPtr, (*infoPtr).CurVal + delta) == 0 {
        if ((*infoPtr).dwStyle & UDS_WRAP) != 0 {
            delta += (if delta < 0 {
                -1
            } else {
                1
            }) * (if (*infoPtr).MaxVal < (*infoPtr).MinVal {
                -1
            } else {
                1
            }) * ((*infoPtr).MinVal - (*infoPtr).MaxVal)
                + (if delta < 0 {
                    1
                } else {
                    -1
                });
        } else if ((*infoPtr).MaxVal > (*infoPtr).MinVal
            && (*infoPtr).CurVal + delta > (*infoPtr).MaxVal)
            || ((*infoPtr).MaxVal < (*infoPtr).MinVal
                && (*infoPtr).CurVal + delta < (*infoPtr).MaxVal)
        {
            delta = (*infoPtr).MaxVal - (*infoPtr).CurVal;
        } else {
            delta = (*infoPtr).MinVal - (*infoPtr).CurVal;
        }
    }

    (*infoPtr).CurVal += delta;
    return (delta != 0) as BOOL;
}

unsafe extern "C" fn UPDOWN_HasBuddyBorder(infoPtr: *const UPDOWN_INFO) -> BOOL {
    return ((((*infoPtr).dwStyle & (UDS_ALIGNLEFT | UDS_ALIGNRIGHT)) != 0)
        && (UPDOWN_IsBuddyEdit(infoPtr) != 0)) as BOOL;
}

unsafe extern "C" fn UPDOWN_GetArrowRect(infoPtr: *const UPDOWN_INFO, rect: *mut RECT, arrow: ::core::ffi::c_uint) {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    let border: ::core::ffi::c_int = if !theme.is_null() { DEFAULT_BUDDYBORDER_THEMED } else { DEFAULT_BUDDYBORDER };
    let spacer: ::core::ffi::c_int = if !theme.is_null() { DEFAULT_BUDDYSPACER_THEMED } else { DEFAULT_BUDDYSPACER };
    let mut size: ::core::ffi::c_int = 0;

    assert!(arrow != 0 && (arrow & ((FLAG_INCR | FLAG_DECR)) as u32) != ((FLAG_INCR | FLAG_DECR)) as u32);

    GetClientRect((*infoPtr).Self_, rect);

    /*
     * Make sure we calculate the rectangle to fit even if we draw the
     * border.
     */
    if UPDOWN_HasBuddyBorder(infoPtr) != 0 {
        if ((*infoPtr).dwStyle & UDS_ALIGNLEFT) != 0 {
            (*rect).left += border;
        } else {
            (*rect).right -= border;
        }

        InflateRect(rect, 0, -border);
    }

    /* now figure out if we need a space away from the buddy */
    if IsWindow((*infoPtr).Buddy) != 0 {
        if ((*infoPtr).dwStyle & UDS_ALIGNLEFT) != 0 {
            (*rect).right -= spacer;
        } else if ((*infoPtr).dwStyle & UDS_ALIGNRIGHT) != 0 {
            (*rect).left += spacer;
        }
    }

    /*
     * We're calculating the midpoint to figure-out where the
     * separation between the buttons will lay.
     */
    if ((*infoPtr).dwStyle & UDS_HORZ) != 0 {
        size = ((*rect).right - (*rect).left) / 2;
        if (arrow & (FLAG_INCR as u32)) != 0 {
            (*rect).left = (*rect).right - size;
        } else if (arrow & (FLAG_DECR as u32)) != 0 {
            (*rect).right = (*rect).left + size;
        }
    } else {
        size = ((*rect).bottom - (*rect).top) / 2;
        if (arrow & (FLAG_INCR as u32)) != 0 {
            (*rect).bottom = (*rect).top + size;
        } else if (arrow & (FLAG_DECR as u32)) != 0 {
            (*rect).top = (*rect).bottom - size;
        }
    }
}

unsafe extern "C" fn UPDOWN_GetArrowFromPoint(infoPtr: *const UPDOWN_INFO, rect: *mut RECT, pt: POINT) -> INT {
    UPDOWN_GetArrowRect(infoPtr, rect, FLAG_INCR as ::core::ffi::c_uint);
    if PtInRect(rect as *const RECT, pt) != 0 {
        return FLAG_INCR;
    }

    UPDOWN_GetArrowRect(infoPtr, rect, FLAG_DECR as ::core::ffi::c_uint);
    if PtInRect(rect as *const RECT, pt) != 0 {
        return FLAG_DECR;
    }

    return 0;
}

unsafe extern "C" fn UPDOWN_GetThousandSep() -> WCHAR {
    let mut sep: [WCHAR; 2] = [0; 2];
    if GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_STHOUSAND as LCTYPE,
        core::ptr::addr_of_mut!(sep) as LPWSTR,
        2 as i32,
    ) != 1
    {
        sep[0 as usize] = ',' as i32 as WCHAR;
    }
    return sep[0 as usize];
}

unsafe extern "C" fn UPDOWN_GetBuddyInt(mut infoPtr: *mut UPDOWN_INFO) -> BOOL {
    let mut txt: [WCHAR; 20] = [0; 20];
    let mut sep: WCHAR = 0;
    let mut src: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut dst: *mut WCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut newVal: ::core::ffi::c_int = 0;
    if !(((*infoPtr).Flags as ::core::ffi::c_int & FLAG_BUDDYINT != 0)
        && IsWindow((*infoPtr).Buddy) != 0)
    {
        return FALSE;
    }
    if UPDOWN_IsBuddyListbox(infoPtr) != 0 {
        newVal = SendMessageW(
            (*infoPtr).Buddy,
            LB_GETCARETINDEX as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as ::core::ffi::c_int;
        if newVal < 0 as ::core::ffi::c_int {
            return FALSE;
        }
    } else {
        if GetWindowTextW(
            (*infoPtr).Buddy,
            &raw mut txt as LPWSTR,
            ARRAY_SIZE!(txt) as ::core::ffi::c_int,
        ) < 0 as ::core::ffi::c_int
        {
            return FALSE;
        }
        sep = UPDOWN_GetThousandSep();
        dst = &raw mut txt as *mut WCHAR;
        src = dst;
        while *src != 0 {
            if *src as ::core::ffi::c_int != sep as ::core::ffi::c_int {
                let fresh18 = dst;
                dst = dst.offset(1);
                *fresh18 = *src;
            }
            src = src.offset(1);
        }
        *dst = 0 as WCHAR;
        newVal = wcstol(
            &raw mut txt as *mut WCHAR,
            &raw mut src,
            (*infoPtr).Base as ::core::ffi::c_int,
        ) as ::core::ffi::c_int;
        if *src as ::core::ffi::c_int != 0 || UPDOWN_InBounds(infoPtr, newVal) == 0 {
            return FALSE;
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/updown.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"UPDOWN_GetBuddyInt\0" as *const u8 as *const ::core::ffi::c_char,
            294 as ::core::ffi::c_int,
            b"new value(%d) from buddy (old=%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            newVal,
            (*infoPtr).CurVal,
        );
    }
    (*infoPtr).CurVal = newVal as INT;
    return TRUE;
}

unsafe extern "C" fn UPDOWN_SetBuddyInt(infoPtr: *const UPDOWN_INFO) -> BOOL {
    static fmt_hex: [WCHAR; 7] = [0x0030u16, 0x0078, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000];
    static fmt_dec_oct: [WCHAR; 3] = [0x0025u16, 0x0064, 0x0000];
    let mut fmt: *const WCHAR = core::ptr::null::<WCHAR>();
    let mut txt: [WCHAR; 20] = [0; 20];
    let mut txt_old: [WCHAR; 20] = [0; 20];
    let mut len: i32 = 0;

    if !(((*infoPtr).Flags & FLAG_BUDDYINT) != 0
        && IsWindow((*infoPtr).Buddy) != 0)
    {
        return FALSE;
    }

    /*if the buddy is a list window, we must set curr index */
    if UPDOWN_IsBuddyListbox(infoPtr) != 0 {
        return (SendMessageW(
            (*infoPtr).Buddy,
            LB_SETCURSEL as UINT,
            (*infoPtr).CurVal as WPARAM,
            0 as LPARAM,
        ) != LB_ERR as LRESULT) as BOOL;
    }

    /* Regular window, so set caption to the number */
    fmt = if (*infoPtr).Base == 16 {
        core::ptr::addr_of!(fmt_hex) as *const WCHAR
    } else {
        core::ptr::addr_of!(fmt_dec_oct) as *const WCHAR
    };
    len = wsprintfW(
        core::ptr::addr_of_mut!(txt) as *mut WCHAR as LPWSTR,
        fmt as LPCWSTR,
        (*infoPtr).CurVal,
    );

    /* Do thousands separation if necessary */
    if (*infoPtr).Base == 10
        && ((*infoPtr).dwStyle & UDS_NOTHOUSANDS) == 0
        && len > 3
    {
        let mut tmp: [WCHAR; 20] = [0; 20];
        let mut src: *mut WCHAR = core::ptr::addr_of_mut!(tmp) as *mut WCHAR;
        let mut dst: *mut WCHAR = core::ptr::addr_of_mut!(txt) as *mut WCHAR;
        let mut sep: WCHAR = UPDOWN_GetThousandSep();
        let mut start: i32 = len % 3;

        memcpy(
            core::ptr::addr_of_mut!(tmp) as *mut WCHAR as *mut core::ffi::c_void,
            core::ptr::addr_of!(txt) as *const WCHAR as *const core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 20]>(),
        );
        if start == 0 {
            start = 3;
        }
        dst = dst.offset(start as isize);
        src = src.offset(start as isize);
        len = 0;
        while *src != 0 {
            if len % 3 == 0 {
                let fresh15 = dst;
                dst = dst.offset(1);
                *fresh15 = sep;
            }
            let fresh16 = src;
            src = src.offset(1);
            let fresh17 = dst;
            dst = dst.offset(1);
            *fresh17 = *fresh16;
            len += 1;
        }
        *dst = 0;
    }

    /* if nothing changed exit earlier */
    GetWindowTextW(
        (*infoPtr).Buddy,
        core::ptr::addr_of_mut!(txt_old) as *mut WCHAR as LPWSTR,
        ARRAY_SIZE!(txt_old) as i32,
    );
    if lstrcmpiW(
        core::ptr::addr_of_mut!(txt_old) as *mut WCHAR as *const WCHAR,
        core::ptr::addr_of_mut!(txt) as *mut WCHAR as *const WCHAR,
    ) == 0
    {
        return FALSE;
    }

    return SetWindowTextW(
        (*infoPtr).Buddy,
        core::ptr::addr_of!(txt) as *const WCHAR,
    );
}

unsafe extern "C" fn UPDOWN_DrawBuddyBackground(infoPtr: *const UPDOWN_INFO, hdc: HDC) -> BOOL {
    let mut br: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let buddyTheme: HTHEME = GetWindowTheme((*infoPtr).Buddy);
    if buddyTheme.is_null() {
        return FALSE;
    }

    GetWindowRect((*infoPtr).Buddy, core::ptr::addr_of_mut!(br));
    MapWindowPoints(
        core::ptr::null_mut(),
        (*infoPtr).Self_,
        core::ptr::addr_of_mut!(br) as LPPOINT,
        2 as UINT,
    );
    GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(r));

    if ((*infoPtr).dwStyle & (UDS_ALIGNLEFT as DWORD)) != 0 {
        br.left = r.left;
    } else if ((*infoPtr).dwStyle & (UDS_ALIGNRIGHT as DWORD)) != 0 {
        br.right = r.right;
    }
    /* FIXME: take disabled etc. into account */
    DrawThemeBackground(
        buddyTheme,
        hdc,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(br),
        core::ptr::null::<RECT>(),
    );
    return TRUE;
}

unsafe extern "C" fn UPDOWN_Draw(infoPtr: *const UPDOWN_INFO, hdc: HDC) -> LRESULT {
    let mut uPressed: BOOL = 0;
    let mut uHot: BOOL = 0;
    let mut dPressed: BOOL = 0;
    let mut dHot: BOOL = 0;
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    let mut uPart: i32 = 0;
    let mut uState: i32 = 0;
    let mut dPart: i32 = 0;
    let mut dState: i32 = 0;
    let mut needBuddyBg: BOOL = FALSE;

    uPressed = ((((*infoPtr).Flags & FLAG_PRESSED) != 0)
        && (((*infoPtr).Flags & FLAG_INCR) != 0)) as i32;
    uHot = ((((*infoPtr).Flags & FLAG_INCR) != 0)
        && (((*infoPtr).Flags & FLAG_MOUSEIN) != 0)) as i32;
    dPressed = ((((*infoPtr).Flags & FLAG_PRESSED) != 0)
        && (((*infoPtr).Flags & FLAG_DECR) != 0)) as i32;
    dHot = ((((*infoPtr).Flags & FLAG_DECR) != 0)
        && (((*infoPtr).Flags & FLAG_MOUSEIN) != 0)) as i32;
    if !theme.is_null() {
        uPart = if ((*infoPtr).dwStyle & UDS_HORZ) != 0 {
            (SPNP_UPHORZ) as i32
        } else {
            (SPNP_UP) as i32
        };
        uState = if ((*infoPtr).dwStyle & WS_DISABLED) != 0 {
            (DNS_DISABLED) as i32
        } else if uPressed != 0 {
            (DNS_PRESSED) as i32
        } else if uHot != 0 {
            (DNS_HOT) as i32
        } else {
            (DNS_NORMAL) as i32
        };
        dPart = if ((*infoPtr).dwStyle & UDS_HORZ) != 0 {
            (SPNP_DOWNHORZ) as i32
        } else {
            (SPNP_DOWN) as i32
        };
        dState = if ((*infoPtr).dwStyle & WS_DISABLED) != 0 {
            (DNS_DISABLED) as i32
        } else if dPressed != 0 {
            (DNS_PRESSED) as i32
        } else if dHot != 0 {
            (DNS_HOT) as i32
        } else {
            (DNS_NORMAL) as i32
        };
        needBuddyBg = (IsWindow((*infoPtr).Buddy) != 0
            && (IsThemeBackgroundPartiallyTransparent(theme, uPart, uState) != 0
                || IsThemeBackgroundPartiallyTransparent(theme, dPart, dState) != 0)) as i32;
    }

    /* Draw the common border between ourselves and our buddy */
    if UPDOWN_HasBuddyBorder(infoPtr) != 0 || needBuddyBg != 0 {
        if theme.is_null() || UPDOWN_DrawBuddyBackground(infoPtr, hdc) == 0 {
            GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(rect));
            DrawEdge(
                hdc,
                core::ptr::addr_of_mut!(rect),
                EDGE_SUNKEN,
                (BF_BOTTOM
                    | BF_TOP
                    | (if ((*infoPtr).dwStyle & UDS_ALIGNLEFT) != 0 {
                        BF_LEFT
                    } else {
                        BF_RIGHT
                    })) as UINT,
            );
        }
    }

    /* Draw the incr button */
    UPDOWN_GetArrowRect(infoPtr, core::ptr::addr_of_mut!(rect), (FLAG_INCR as u32));
    if !theme.is_null() {
        DrawThemeBackground(
            theme,
            hdc,
            uPart,
            uState,
            core::ptr::addr_of_mut!(rect),
            core::ptr::null::<RECT>(),
        );
    } else {
        DrawFrameControl(
            hdc,
            core::ptr::addr_of_mut!(rect),
            DFC_SCROLL,
            ((if ((*infoPtr).dwStyle & UDS_HORZ) != 0 {
                DFCS_SCROLLRIGHT
            } else {
                DFCS_SCROLLUP
            }) | (if ((*infoPtr).dwStyle & UDS_HOTTRACK) != 0 && uHot != 0 {
                DFCS_HOT
            } else {
                0
            }) | (if uPressed != 0 {
                DFCS_PUSHED
            } else {
                0
            }) | (if ((*infoPtr).dwStyle & WS_DISABLED) != 0 {
                DFCS_INACTIVE
            } else {
                0
            })) as UINT,
        );
    }

    /* Draw the decr button */
    UPDOWN_GetArrowRect(infoPtr, core::ptr::addr_of_mut!(rect), (FLAG_DECR as u32));
    if !theme.is_null() {
        DrawThemeBackground(
            theme,
            hdc,
            dPart,
            dState,
            core::ptr::addr_of_mut!(rect),
            core::ptr::null::<RECT>(),
        );
    } else {
        DrawFrameControl(
            hdc,
            core::ptr::addr_of_mut!(rect),
            DFC_SCROLL,
            ((if ((*infoPtr).dwStyle & UDS_HORZ) != 0 {
                DFCS_SCROLLLEFT
            } else {
                DFCS_SCROLLDOWN
            }) | (if ((*infoPtr).dwStyle & UDS_HOTTRACK) != 0 && dHot != 0 {
                DFCS_HOT
            } else {
                0
            }) | (if dPressed != 0 {
                DFCS_PUSHED
            } else {
                0
            }) | (if ((*infoPtr).dwStyle & WS_DISABLED) != 0 {
                DFCS_INACTIVE
            } else {
                0
            })) as UINT,
        );
    }

    return 0;
}

unsafe extern "C" fn UPDOWN_Paint(infoPtr: *const UPDOWN_INFO, mut hdc: HDC) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    if !hdc.is_null() {
        return UPDOWN_Draw(infoPtr, hdc);
    }
    hdc = BeginPaint((*infoPtr).Self_, core::ptr::addr_of_mut!(ps));
    UPDOWN_Draw(infoPtr, hdc);
    EndPaint((*infoPtr).Self_, core::ptr::addr_of_mut!(ps));
    return 0;
}

unsafe extern "C" fn UPDOWN_KeyPressed(mut infoPtr: *mut UPDOWN_INFO, mut key: ::core::ffi::c_int) -> LRESULT {
    let mut arrow: ::core::ffi::c_int = 0;
    let mut accel: ::core::ffi::c_int = 0;
    if (key) as u32 == VK_UP {
        arrow = FLAG_INCR;
    } else if (key) as u32 == VK_DOWN {
        arrow = FLAG_DECR;
    } else {
        return 1 as LRESULT;
    }
    UPDOWN_GetBuddyInt(infoPtr);
    (*infoPtr).Flags &= !FLAG_ARROW;
    (*infoPtr).Flags |= FLAG_PRESSED | arrow;
    InvalidateRect((*infoPtr).Self_, core::ptr::null::<RECT>(), FALSE);
    SetTimer(
        (*infoPtr).Self_,
        TIMER_AUTOPRESS as UINT_PTR,
        AUTOPRESS_DELAY as UINT,
        None,
    );
    accel = (if (*infoPtr).AccelCount != 0 && !(*infoPtr).AccelVect.is_null() {
        (*(*infoPtr).AccelVect.offset(0 as isize)).nInc
    } else {
        1 as UINT
    }) as ::core::ffi::c_int;
    UPDOWN_DoAction(infoPtr, accel, arrow);
    return 0 as LRESULT;
}

unsafe extern "C" fn UPDOWN_GetPos(infoPtr: *mut UPDOWN_INFO, err: *mut BOOL) -> i32 {
    let mut succ: BOOL = UPDOWN_GetBuddyInt(infoPtr);
    let mut val: i32 = (*infoPtr).CurVal;

    if UPDOWN_InBounds(infoPtr, val) == 0 {
        if ((*infoPtr).MinVal < (*infoPtr).MaxVal && val < (*infoPtr).MinVal)
            || ((*infoPtr).MinVal > (*infoPtr).MaxVal && val > (*infoPtr).MinVal)
        {
            val = (*infoPtr).MinVal;
        } else {
            val = (*infoPtr).MaxVal;
        }

        succ = FALSE;
    }

    if !err.is_null() {
        *err = (succ == 0) as BOOL;
    }
    return val;
}

unsafe extern "C" fn UPDOWN_SetPos(infoPtr: *mut UPDOWN_INFO, mut pos: i32) -> i32 {
    let mut ret: i32 = (*infoPtr).CurVal;

    if UPDOWN_InBounds(infoPtr, pos) == 0 {
        if ((*infoPtr).MinVal < (*infoPtr).MaxVal && pos < (*infoPtr).MinVal)
            || ((*infoPtr).MinVal > (*infoPtr).MaxVal && pos > (*infoPtr).MinVal)
        {
            pos = (*infoPtr).MinVal;
        } else {
            pos = (*infoPtr).MaxVal;
        }
    }

    (*infoPtr).CurVal = pos;
    UPDOWN_SetBuddyInt(infoPtr);

    if UPDOWN_InBounds(infoPtr, ret) == 0 {
        if ((*infoPtr).MinVal < (*infoPtr).MaxVal && ret < (*infoPtr).MinVal)
            || ((*infoPtr).MinVal > (*infoPtr).MaxVal && ret > (*infoPtr).MinVal)
        {
            ret = (*infoPtr).MinVal;
        } else {
            ret = (*infoPtr).MaxVal;
        }
    }
    return ret;
}

unsafe extern "C" fn UPDOWN_SetRange(
    mut infoPtr: *mut UPDOWN_INFO,
    mut Max: INT,
    mut Min: INT,
) -> LRESULT {
    (*infoPtr).MaxVal = Max;
    (*infoPtr).MinVal = Min;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/updown.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"UPDOWN_SetRange\0" as *const u8 as *const ::core::ffi::c_char,
            544 as ::core::ffi::c_int,
            b"UpDown Ctrl new range(%d to %d), hwnd=%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).MinVal,
            (*infoPtr).MaxVal,
            (*infoPtr).Self_,
        );
    }
    return 0;
}

unsafe extern "C" fn UPDOWN_MouseWheel(infoPtr: *mut UPDOWN_INFO, wParam: WPARAM) -> LRESULT {
    let iWheelDelta: ::core::ffi::c_int = GET_WHEEL_DELTA_WPARAM!(wParam) as ::core::ffi::c_int / WHEEL_DELTA as ::core::ffi::c_int;

    if (wParam & (MK_SHIFT | MK_CONTROL) as WPARAM) != 0 {
        return 0 as LRESULT;
    }

    if iWheelDelta != 0 as ::core::ffi::c_int {
        UPDOWN_GetBuddyInt(infoPtr);
        UPDOWN_DoAction(
            infoPtr,
            abs(iWheelDelta),
            if iWheelDelta > 0 as ::core::ffi::c_int {
                FLAG_INCR
            } else {
                FLAG_DECR
            },
        );
    }

    return 1 as LRESULT;
}

unsafe extern "system" fn UPDOWN_Buddy_SubclassProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
    mut uId: UINT_PTR,
    mut ref_data: DWORD_PTR,
) -> LRESULT {
    let mut infoPtr: *mut UPDOWN_INFO = UPDOWN_GetInfoPtr!(ref_data as HWND);

    match uMsg {
        WM_KEYDOWN => {
            if !infoPtr.is_null() {
                UPDOWN_KeyPressed(infoPtr, wParam as ::core::ffi::c_int);
                if wParam == VK_UP as WPARAM || wParam == VK_DOWN as WPARAM {
                    return 0;
                }
            }
        }
        WM_MOUSEWHEEL => {
            if !infoPtr.is_null() {
                UPDOWN_MouseWheel(infoPtr, wParam as ::core::ffi::c_int as WPARAM);
            }
        }
        WM_NCDESTROY => {
            RemoveWindowSubclass(
                hwnd,
                Some(
                    UPDOWN_Buddy_SubclassProc
                        as unsafe extern "system" fn(
                            HWND,
                            UINT,
                            WPARAM,
                            LPARAM,
                            UINT_PTR,
                            DWORD_PTR,
                        ) -> LRESULT,
                ),
                BUDDY_SUBCLASSID as UINT_PTR,
            );
        }
        _ => {}
    }

    return DefSubclassProc(hwnd, uMsg, wParam, lParam);
}

unsafe extern "C" fn UPDOWN_ResetSubclass(mut infoPtr: *mut UPDOWN_INFO) {
    SetWindowSubclass(
        (*infoPtr).Buddy,
        Some(
            UPDOWN_Buddy_SubclassProc,
        ),
        BUDDY_SUBCLASSID as UINT_PTR,
        0 as DWORD_PTR,
    );
}

unsafe extern "C" fn UPDOWN_SetBuddy(mut infoPtr: *mut UPDOWN_INFO, mut bud: HWND) -> HWND {
    let mut budRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut x: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut buddyClass: [WCHAR; 40] = [0; 40];
    let mut old_buddy: HWND = ::core::ptr::null_mut::<HWND__>();

    old_buddy = (*infoPtr).Buddy;

    UPDOWN_ResetSubclass(infoPtr);

    if IsWindow(bud) == 0 {
        bud = ::core::ptr::null_mut::<HWND__>();
    }

    /* Store buddy window handle */
    (*infoPtr).Buddy = bud;

    if !bud.is_null() {
        /* Store buddy window class type */
        (*infoPtr).BuddyType = BUDDY_TYPE_UNKNOWN as INT;
        if GetClassNameW(bud, &raw mut buddyClass as LPWSTR, ARRAY_SIZE!(buddyClass) as ::core::ffi::c_int) != 0 {
            if lstrcmpiW(&raw mut buddyClass as LPCWSTR, (WC_EDITW).as_ptr()) == 0 {
                (*infoPtr).BuddyType = BUDDY_TYPE_EDIT as INT;
            } else if lstrcmpiW(&raw mut buddyClass as LPCWSTR, (WC_LISTBOXW).as_ptr()) == 0 {
                (*infoPtr).BuddyType = BUDDY_TYPE_LISTBOX as INT;
            }
        }

        if ((*infoPtr).dwStyle & UDS_ARROWKEYS) != 0 {
            SetWindowSubclass(
                bud,
                Some(
                    UPDOWN_Buddy_SubclassProc,
                ),
                (BUDDY_SUBCLASSID) as UINT_PTR,
                (*infoPtr).Self_ as DWORD_PTR,
            );
        }

        /* Get the rect of the buddy relative to its parent */
        GetWindowRect((*infoPtr).Buddy, &raw mut budRect);
        MapWindowPoints(
            HWND_DESKTOP,
            GetParent((*infoPtr).Buddy),
            &raw mut budRect.left as LPPOINT,
            2,
        );

        /* now do the positioning */
        if ((*infoPtr).dwStyle & UDS_ALIGNLEFT) != 0 {
            x = budRect.left as ::core::ffi::c_int;
            budRect.left += DEFAULT_WIDTH + DEFAULT_XSEP;
        } else if ((*infoPtr).dwStyle & UDS_ALIGNRIGHT) != 0 {
            budRect.right -= DEFAULT_WIDTH + DEFAULT_XSEP;
            x = budRect.right as ::core::ffi::c_int + DEFAULT_XSEP;
        } else {
            /* nothing to do */
            return old_buddy;
        }

        /* first adjust the buddy to accommodate the up/down */
        SetWindowPos(
            (*infoPtr).Buddy,
            core::ptr::null_mut(),
            budRect.left as ::core::ffi::c_int,
            budRect.top as ::core::ffi::c_int,
            budRect.right as ::core::ffi::c_int - budRect.left as ::core::ffi::c_int,
            budRect.bottom as ::core::ffi::c_int - budRect.top as ::core::ffi::c_int,
            (SWP_NOACTIVATE | SWP_NOZORDER) as UINT,
        );

        /* now position the up/down */
        /* Since the UDS_ALIGN* flags were used, */
        /* we will pick the position and size of the window. */
        width = DEFAULT_WIDTH;

        /*
         * If the updown has a buddy border, it has to overlap with the buddy
         * to look as if it is integrated with the buddy control.
         * We nudge the control or change its size to overlap.
         */
        if UPDOWN_HasBuddyBorder(infoPtr) != 0 {
            if ((*infoPtr).dwStyle & UDS_ALIGNLEFT) != 0 {
                width += DEFAULT_BUDDYBORDER;
            } else {
                x -= DEFAULT_BUDDYBORDER;
            }
        }

        SetWindowPos(
            (*infoPtr).Self_,
            core::ptr::null_mut(),
            x,
            budRect.top as ::core::ffi::c_int - DEFAULT_ADDTOP,
            width,
            budRect.bottom as ::core::ffi::c_int - budRect.top as ::core::ffi::c_int
                + DEFAULT_ADDTOP
                + DEFAULT_ADDBOT,
            (SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOZORDER) as UINT,
        );
    } else if ((*infoPtr).dwStyle & UDS_HORZ) == 0 && !old_buddy.is_null() {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect((*infoPtr).Self_, &raw mut rect);
        MapWindowPoints(
            HWND_DESKTOP,
            GetParent((*infoPtr).Self_),
            &raw mut rect as LPPOINT,
            2,
        );
        SetWindowPos(
            (*infoPtr).Self_,
            core::ptr::null_mut(),
            rect.left as ::core::ffi::c_int,
            rect.top as ::core::ffi::c_int,
            DEFAULT_WIDTH,
            rect.bottom as ::core::ffi::c_int - rect.top as ::core::ffi::c_int,
            (SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOZORDER) as UINT,
        );
    }

    return old_buddy;
}

unsafe extern "C" fn UPDOWN_DoAction(mut infoPtr: *mut UPDOWN_INFO, mut delta: i32, mut action: i32) {
    let mut ni: NMUPDOWN = core::mem::zeroed();
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/updown.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"UPDOWN_DoAction\0" as *const u8 as *const core::ffi::c_char,
            724 as i32,
            b"%d by %d\n\0" as *const u8 as *const core::ffi::c_char,
            action,
            delta,
        );
    }
    /* check if we can do the modification first */
    delta *= (if (action & FLAG_INCR) != 0 {
        1
    } else {
        -1
    }) * (if (*infoPtr).MaxVal < (*infoPtr).MinVal {
        -1
    } else {
        1
    });
    if (action & FLAG_INCR) != 0 && (action & FLAG_DECR) != 0 {
        delta = 0;
    }
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/updown.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"UPDOWN_DoAction\0" as *const u8 as *const core::ffi::c_char,
            730 as i32,
            b"current %d, delta: %d\n\0" as *const u8 as *const core::ffi::c_char,
            (*infoPtr).CurVal,
            delta,
        );
    }

    /* We must notify parent now to obtain permission */
    ni.iPos = (*infoPtr).CurVal;
    ni.iDelta = delta;
    ni.hdr.hwndFrom = (*infoPtr).Self_;
    ni.hdr.idFrom = GetWindowLongPtrW((*infoPtr).Self_, GWLP_ID) as UINT_PTR;
    ni.hdr.code = UDN_DELTAPOS as UINT;
    if SendMessageW(
        (*infoPtr).Notify,
        WM_NOTIFY as UINT,
        ni.hdr.idFrom as WPARAM,
        &raw mut ni as LPARAM,
    ) == 0
    {
        /* Parent said: OK to adjust */

        /* Now adjust value with (maybe new) delta */
        if UPDOWN_OffsetVal(infoPtr, ni.iDelta) != 0 {
            if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/updown.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"UPDOWN_DoAction\0" as *const u8 as *const core::ffi::c_char,
                    743 as i32,
                    b"new %d, delta: %d\n\0" as *const u8 as *const core::ffi::c_char,
                    (*infoPtr).CurVal,
                    ni.iDelta,
                );
            }
            /* Now take care about our buddy */
            UPDOWN_SetBuddyInt(infoPtr);
        }
    }

    /* Also, notify it. This message is sent in any case. */
    SendMessageW(
        (*infoPtr).Notify,
        (if ((*infoPtr).dwStyle & UDS_HORZ as DWORD) != 0 {
            WM_HSCROLL
        } else {
            WM_VSCROLL
        }) as UINT,
        MAKELONG!(SB_THUMBPOSITION as WORD, (*infoPtr).CurVal as WORD) as WPARAM,
        (*infoPtr).Self_ as LPARAM,
    );
}

unsafe extern "C" fn UPDOWN_IsEnabled(infoPtr: *const UPDOWN_INFO) -> BOOL {
    if !IsWindowEnabled((*infoPtr).Self_) != 0 {
        return FALSE;
    }
    if !(*infoPtr).Buddy.is_null() {
        return IsWindowEnabled((*infoPtr).Buddy);
    }
    return TRUE;
}

unsafe extern "C" fn UPDOWN_CancelMode(infoPtr: *mut UPDOWN_INFO) -> BOOL {
    if ((*infoPtr).Flags & FLAG_PRESSED) == 0 {
        return FALSE;
    }
    KillTimer((*infoPtr).Self_, TIMER_AUTOREPEAT as UINT_PTR);
    KillTimer((*infoPtr).Self_, TIMER_ACCEL as UINT_PTR);
    KillTimer((*infoPtr).Self_, TIMER_AUTOPRESS as UINT_PTR);
    if GetCapture() == (*infoPtr).Self_ {
        ReleaseCapture();
    }
    (*infoPtr).Flags &= !FLAG_PRESSED;
    InvalidateRect((*infoPtr).Self_, ::core::ptr::null::<RECT>(), FALSE);
    return TRUE;
}

unsafe extern "C" fn UPDOWN_HandleMouseEvent(
    mut infoPtr: *mut UPDOWN_INFO,
    mut msg: UINT,
    mut x: INT,
    mut y: INT,
) {
    let mut pt: POINT = POINT {
        x: x as LONG,
        y: y as LONG,
    };
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut temp: ::core::ffi::c_int = 0;
    let mut arrow: ::core::ffi::c_int = 0;
    let mut tme: TRACKMOUSEEVENT = TRACKMOUSEEVENT {
        cbSize: 0,
        dwFlags: 0,
        hwndTrack: ::core::ptr::null_mut::<HWND__>(),
        dwHoverTime: 0,
    };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/updown.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"UPDOWN_HandleMouseEvent\0" as *const u8 as *const ::core::ffi::c_char,
            809 as ::core::ffi::c_int,
            b"msg %04x point %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            msg,
            wine_dbgstr_point(&raw mut pt),
        );
    }
    match msg {
        WM_LBUTTONDOWN => {
            if UPDOWN_IsBuddyEdit(infoPtr) != 0 {
                SetFocus((*infoPtr).Buddy);
            }
            arrow = UPDOWN_GetArrowFromPoint(infoPtr, &raw mut rect, pt) as ::core::ffi::c_int;
            (*infoPtr).Flags &= !(FLAG_MOUSEIN | FLAG_ARROW);
            if arrow != 0 {
                (*infoPtr).Flags |= FLAG_MOUSEIN | arrow;
            } else if (*infoPtr).AccelIndex != -(1) {
                (*infoPtr).AccelIndex = 0 as INT;
            }
            if ((*infoPtr).Flags as ::core::ffi::c_int & FLAG_ARROW) != 0 {
                UPDOWN_GetBuddyInt(infoPtr);
                (*infoPtr).Flags |= FLAG_PRESSED;
                InvalidateRect(
                    (*infoPtr).Self_,
                    ::core::ptr::null::<RECT>(),
                    FALSE,
                );
                temp = (if (*infoPtr).AccelCount != 0 && !(*infoPtr).AccelVect.is_null() {
                    (*(*infoPtr)
                        .AccelVect
                        .offset(0 as isize))
                    .nInc
                } else {
                    1 as UINT
                }) as ::core::ffi::c_int;
                UPDOWN_DoAction(
                    infoPtr,
                    temp,
                    (*infoPtr).Flags as ::core::ffi::c_int & FLAG_ARROW,
                );
                SetCapture((*infoPtr).Self_);
                SetTimer(
                    (*infoPtr).Self_,
                    TIMER_AUTOREPEAT as UINT_PTR,
                    INITIAL_DELAY as UINT,
                    None,
                );
            }
        }
        WM_MOUSEMOVE => {
            temp = (*infoPtr).Flags as ::core::ffi::c_int;
            arrow = UPDOWN_GetArrowFromPoint(infoPtr, &raw mut rect, pt) as ::core::ffi::c_int;
            (*infoPtr).Flags &= !(FLAG_MOUSEIN | FLAG_ARROW);
            if arrow != 0 {
                (*infoPtr).Flags |= FLAG_MOUSEIN | arrow;
            } else {
                if (*infoPtr).AccelIndex != -(1) {
                    (*infoPtr).AccelIndex = 0 as INT;
                }
            }
            if temp != (*infoPtr).Flags {
                InvalidateRect(
                    (*infoPtr).Self_,
                    ::core::ptr::null::<RECT>(),
                    FALSE,
                );
            }
            tme.cbSize = ::core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
            tme.dwFlags = TME_LEAVE as DWORD;
            tme.hwndTrack = (*infoPtr).Self_;
            TrackMouseEvent(&raw mut tme);
        }
        WM_MOUSELEAVE => {
            (*infoPtr).Flags &= !(FLAG_MOUSEIN | FLAG_ARROW);
            InvalidateRect(
                (*infoPtr).Self_,
                ::core::ptr::null::<RECT>(),
                FALSE,
            );
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/updown.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"UPDOWN_HandleMouseEvent\0" as *const u8
                        as *const ::core::ffi::c_char,
                    884 as ::core::ffi::c_int,
                    b"Impossible case (msg=%x)!\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    msg,
                );
            }
        }
    }
}

unsafe extern "system" fn UpDownWindowProc(hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut UPDOWN_INFO = UPDOWN_GetInfoPtr!(hwnd) as *mut UPDOWN_INFO;
    static themeClass: [WCHAR; 5] = [0x0053u16, 0x0070, 0x0069, 0x006E, 0x0000];
    let mut theme: HTHEME = core::ptr::null_mut();

    if infoPtr.is_null() && (message != WM_CREATE) {
        return DefWindowProcW(hwnd, message, wParam, lParam);
    }

    'sw: {
        match message {
            WM_CREATE => {
                let pcs: *mut CREATESTRUCTW = lParam as *mut CREATESTRUCTW;

                infoPtr = heap_alloc_zero(core::mem::size_of::<UPDOWN_INFO>() as SIZE_T) as *mut UPDOWN_INFO;
                SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

                /* initialize the info struct */
                (*infoPtr).Self_ = hwnd;
                (*infoPtr).Notify = (*pcs).hwndParent;
                (*infoPtr).dwStyle = (*pcs).style as DWORD;
                (*infoPtr).AccelCount = 0 as UINT;
                (*infoPtr).AccelVect = 0 as *mut UDACCEL;
                (*infoPtr).AccelIndex = -(1 as INT);
                (*infoPtr).CurVal = 0 as INT;
                (*infoPtr).MinVal = 100 as INT;
                (*infoPtr).MaxVal = 0 as INT;
                (*infoPtr).Base = 10 as INT; /* Default to base 10  */
                (*infoPtr).Buddy = 0 as HWND; /* No buddy window yet */
                (*infoPtr).Flags = (if ((*infoPtr).dwStyle & (UDS_SETBUDDYINT as DWORD)) != 0 {
                    FLAG_BUDDYINT
                } else {
                    0
                }) as INT;

                SetWindowLongW(hwnd, GWL_STYLE, ((*infoPtr).dwStyle & !(WS_BORDER as DWORD)) as LONG);
                if ((*infoPtr).dwStyle & (UDS_HORZ as DWORD)) == 0 {
                    SetWindowPos(
                        hwnd,
                        core::ptr::null_mut(),
                        0,
                        0,
                        DEFAULT_WIDTH,
                        (*pcs).cy,
                        SWP_NOOWNERZORDER | SWP_NOZORDER | SWP_NOMOVE,
                    );
                }

                /* Do we pick the buddy win ourselves? */
                if ((*infoPtr).dwStyle & (UDS_AUTOBUDDY as DWORD)) != 0 {
                    UPDOWN_SetBuddy(infoPtr, GetWindow(hwnd, GW_HWNDPREV as UINT));
                }

                OpenThemeData(hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);
            }
            WM_DESTROY => {
                heap_free((*infoPtr).AccelVect as *mut core::ffi::c_void);
                UPDOWN_ResetSubclass(infoPtr);
                heap_free(infoPtr as *mut core::ffi::c_void);
                SetWindowLongPtrW(hwnd, 0, 0);
                theme = GetWindowTheme(hwnd);
                CloseThemeData(theme);
                break 'sw;
            }
            WM_ENABLE => {
                if wParam != 0 {
                    (*infoPtr).dwStyle &= !(WS_DISABLED as DWORD);
                } else {
                    (*infoPtr).dwStyle |= WS_DISABLED as DWORD;
                    UPDOWN_CancelMode(infoPtr);
                }
                InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), FALSE);
                break 'sw;
            }
            WM_STYLECHANGED => {
                if wParam == GWL_STYLE as WPARAM {
                    (*infoPtr).dwStyle = (*(lParam as LPSTYLESTRUCT)).styleNew;
                    InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), FALSE);
                }
                break 'sw;
            }
            WM_THEMECHANGED => {
                theme = GetWindowTheme(hwnd);
                CloseThemeData(theme);
                OpenThemeData(hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);
                InvalidateRect(hwnd, core::ptr::null_mut(), FALSE);
                break 'sw;
            }
            WM_TIMER => {
                /* is this the auto-press timer? */
                if wParam == TIMER_AUTOPRESS as WPARAM {
                    KillTimer(hwnd, (TIMER_AUTOPRESS) as UINT_PTR);
                    (*infoPtr).Flags &= !(FLAG_PRESSED | FLAG_ARROW) as INT;
                    InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), FALSE);
                }

                /* if initial timer, kill it and start the repeat timer */
                if wParam == TIMER_AUTOREPEAT as WPARAM {
                    let mut delay: INT = 0;

                    KillTimer(hwnd, (TIMER_AUTOREPEAT) as UINT_PTR);
                    /* if no accel info given, used default timer */
                    if (*infoPtr).AccelCount == 0 as UINT
                        || (*infoPtr).AccelVect == 0 as *mut UDACCEL
                    {
                        (*infoPtr).AccelIndex = -(1 as INT);
                        delay = REPEAT_DELAY as INT;
                    } else {
                        (*infoPtr).AccelIndex = 0 as INT; /* otherwise, use it */
                        delay = (*(*infoPtr).AccelVect.offset((*infoPtr).AccelIndex as isize))
                            .nSec
                            .wrapping_mul(1000 as UINT)
                            .wrapping_add(1 as UINT) as INT;
                    }
                    SetTimer(hwnd, TIMER_ACCEL as UINT_PTR, delay as UINT, None);
                }

                /* now, if the mouse is above us, do the thing...*/
                if ((*infoPtr).Flags & (FLAG_MOUSEIN as INT)) != 0 {
                    let mut temp: INT = 0;

                    temp = (if (*infoPtr).AccelIndex == -(1 as INT) {
                        1 as UINT
                    } else {
                        (*(*infoPtr).AccelVect.offset((*infoPtr).AccelIndex as isize)).nInc
                    }) as INT;
                    UPDOWN_DoAction(infoPtr, temp, (*infoPtr).Flags & (FLAG_ARROW as INT));

                    if (*infoPtr).AccelIndex != -(1 as INT)
                        && ((*infoPtr).AccelIndex as UINT)
                            < (*infoPtr).AccelCount.wrapping_sub(1 as UINT)
                    {
                        KillTimer(hwnd, (TIMER_ACCEL) as UINT_PTR);
                        (*infoPtr).AccelIndex += 1; /* move to the next accel info */
                        temp = (*(*infoPtr).AccelVect.offset((*infoPtr).AccelIndex as isize))
                            .nSec
                            .wrapping_mul(1000 as UINT)
                            .wrapping_add(1 as UINT) as INT;
                        /* make sure we have at least 1ms intervals */
                        SetTimer(hwnd, TIMER_ACCEL as UINT_PTR, temp as UINT, None);
                    }
                }
                break 'sw;
            }
            WM_CANCELMODE => {
                return UPDOWN_CancelMode(infoPtr) as LRESULT;
            }
            WM_LBUTTONUP => {
                if GetCapture() != (*infoPtr).Self_ {
                    break 'sw;
                }

                if ((*infoPtr).Flags & (FLAG_MOUSEIN as INT)) != 0
                    && ((*infoPtr).Flags & (FLAG_ARROW as INT)) != 0
                {
                    SendMessageW(
                        (*infoPtr).Notify,
                        (if ((*infoPtr).dwStyle & (UDS_HORZ as DWORD)) != 0 {
                            WM_HSCROLL
                        } else {
                            WM_VSCROLL
                        }) as UINT,
                        MAKELONG!(SB_ENDSCROLL, (*infoPtr).CurVal) as WPARAM,
                        hwnd as LPARAM,
                    );
                    if UPDOWN_IsBuddyEdit(infoPtr) != 0 {
                        SendMessageW(
                            (*infoPtr).Buddy,
                            EM_SETSEL,
                            0,
                            MAKELONG!(0, -(1 as INT)) as LPARAM,
                        );
                    }
                }
                UPDOWN_CancelMode(infoPtr);
                break 'sw;
            }
            WM_LBUTTONDOWN | WM_MOUSEMOVE | WM_MOUSELEAVE => {
                if UPDOWN_IsEnabled(infoPtr) != 0 {
                    UPDOWN_HandleMouseEvent(
                        infoPtr,
                        message,
                        LOWORD!(lParam) as SHORT as INT,
                        HIWORD!(lParam) as SHORT as INT,
                    );
                }
                break 'sw;
            }
            WM_MOUSEWHEEL => {
                UPDOWN_MouseWheel(infoPtr, wParam);
                break 'sw;
            }
            WM_KEYDOWN => {
                if ((*infoPtr).dwStyle & (UDS_ARROWKEYS as DWORD)) != 0 && UPDOWN_IsEnabled(infoPtr) != 0 {
                    return UPDOWN_KeyPressed(infoPtr, wParam as INT);
                }
                break 'sw;
            }
            WM_PRINTCLIENT | WM_PAINT => {
                return UPDOWN_Paint(infoPtr, wParam as HDC);
            }
            UDM_GETACCEL => {
                if wParam == 0 as WPARAM && lParam == 0 as LPARAM {
                    return (*infoPtr).AccelCount as LRESULT;
                }
                if wParam != 0 && lParam != 0 {
                    let temp: INT = min((*infoPtr).AccelCount as WPARAM, wParam) as INT;
                    memcpy(
                        lParam as *mut core::ffi::c_void,
                        (*infoPtr).AccelVect as *const core::ffi::c_void,
                        (temp as usize).wrapping_mul(core::mem::size_of::<UDACCEL>()),
                    );
                    return temp as LRESULT;
                }
                return 0 as LRESULT;
            }
            UDM_SETACCEL => {
                if !(*infoPtr).AccelVect.is_null() {
                    heap_free((*infoPtr).AccelVect as *mut core::ffi::c_void);
                    (*infoPtr).AccelCount = 0 as UINT;
                    (*infoPtr).AccelVect = 0 as *mut UDACCEL;
                }
                if wParam == 0 as WPARAM {
                    return TRUE as LRESULT;
                }
                (*infoPtr).AccelVect = heap_alloc(
                    wParam.wrapping_mul((core::mem::size_of::<UDACCEL>()) as _),
                ) as *mut UDACCEL;
                if (*infoPtr).AccelVect.is_null() {
                    return FALSE as LRESULT;
                }
                memcpy(
                    (*infoPtr).AccelVect as *mut core::ffi::c_void,
                    lParam as *mut core::ffi::c_void,
                    (wParam.wrapping_mul((core::mem::size_of::<UDACCEL>()) as _)) as _,
                );
                (*infoPtr).AccelCount = wParam as UINT;

                if TRACE_ON!(updown) {
                    let mut i: UINT = 0;

                    i = 0 as UINT;
                    while (i as WPARAM) < wParam {
                        TRACE!(
                            b"%u: nSec %u nInc %u\n\0".as_ptr() as *const i8,
                            i,
                            (*(*infoPtr).AccelVect.offset(i as isize)).nSec,
                            (*(*infoPtr).AccelVect.offset(i as isize)).nInc,
                        );
                        i = i.wrapping_add(1);
                    }
                }

                return TRUE as LRESULT;
            }
            UDM_GETBASE => {
                return (*infoPtr).Base as LRESULT;
            }
            UDM_SETBASE => {
                TRACE!(
                    b"UpDown Ctrl new base(%ld), hwnd=%p\n\0".as_ptr() as *const i8,
                    wParam,
                    hwnd,
                );
                if wParam == 10 as WPARAM || wParam == 16 as WPARAM {
                    let old_base: WPARAM = (*infoPtr).Base as WPARAM;
                    (*infoPtr).Base = wParam as INT;

                    if old_base != (*infoPtr).Base as WPARAM {
                        UPDOWN_SetBuddyInt(infoPtr);
                    }

                    return old_base as LRESULT;
                }
                break 'sw;
            }
            UDM_GETBUDDY => {
                return (*infoPtr).Buddy as LRESULT;
            }
            UDM_SETBUDDY => {
                return UPDOWN_SetBuddy(infoPtr, wParam as HWND) as LRESULT;
            }
            UDM_GETPOS => {
                let mut err: BOOL = 0;
                let mut pos: INT = 0;

                pos = UPDOWN_GetPos(infoPtr, core::ptr::addr_of_mut!(err));
                return MAKELONG!(pos, err) as LRESULT;
            }
            UDM_SETPOS => {
                return UPDOWN_SetPos(infoPtr, LOWORD!(lParam) as SHORT as INT) as LRESULT;
            }
            UDM_GETRANGE => {
                return MAKELONG!((*infoPtr).MaxVal, (*infoPtr).MinVal) as LRESULT;
            }
            UDM_SETRANGE => {
                /* we must have:
                UD_MINVAL <= Max <= UD_MAXVAL
                UD_MINVAL <= Min <= UD_MAXVAL
                |Max-Min| <= UD_MAXVAL */
                UPDOWN_SetRange(infoPtr, lParam as SHORT as INT, HIWORD!(lParam) as SHORT as INT);
                break 'sw;
            }
            UDM_GETRANGE32 => {
                if wParam != 0 {
                    *(wParam as LPINT) = (*infoPtr).MinVal;
                }
                if lParam != 0 {
                    *(lParam as LPINT) = (*infoPtr).MaxVal;
                }
                break 'sw;
            }
            UDM_SETRANGE32 => {
                UPDOWN_SetRange(infoPtr, lParam as INT, wParam as INT);
                break 'sw;
            }
            UDM_GETPOS32 => {
                return UPDOWN_GetPos(infoPtr, lParam as *mut BOOL) as LRESULT;
            }
            UDM_SETPOS32 => {
                return UPDOWN_SetPos(infoPtr, lParam as INT) as LRESULT;
            }
            UDM_GETUNICODEFORMAT => {
                /* we lie a bit here, we're always using Unicode internally */
                return (*infoPtr).UnicodeFormat as LRESULT;
            }
            UDM_SETUNICODEFORMAT => {
                /* do we really need to honour this flag? */
                let temp: INT = (*infoPtr).UnicodeFormat as INT;
                (*infoPtr).UnicodeFormat = wParam as BOOL;
                return temp as LRESULT;
            }
            _ => {
                if (message >= WM_USER as UINT)
                    && (message < WM_APP as UINT)
                    && COMCTL32_IsReflectedMessage(message) == 0
                {
                    ERR!(
                        b"unknown msg %04x wp=%04lx lp=%08lx\n\0".as_ptr() as *const i8,
                        message,
                        wParam,
                        lParam,
                    );
                }
                return DefWindowProcW(hwnd, message, wParam, lParam);
            }
        }
    }

    return 0 as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn UPDOWN_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_VREDRAW | CS_HREDRAW) as UINT;
    wndClass.lpfnWndProc = Some(UpDownWindowProc);
    wndClass.cbClsExtra = 0 as i32;
    wndClass.cbWndExtra = core::mem::size_of::<UPDOWN_INFO>() as i32;
    wndClass.hCursor = LoadCursorW(
        0 as usize as HINSTANCE,
        IDC_ARROW as usize as LPWSTR,
    );
    wndClass.hbrBackground =
        ((COLOR_BTNFACE + 1) as usize) as HBRUSH;
    wndClass.lpszClassName = (UPDOWN_CLASSW).as_ptr();

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn UPDOWN_Unregister() {
    UnregisterClassW((UPDOWN_CLASSW).as_ptr(), core::ptr::null_mut());
}
