use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::comctl32undoc::Str_GetPtrWtoA;
use crate::comctl32::comctl32undoc::Str_SetPtrAtoW;
use crate::comctl32::comctl32undoc::Str_SetPtrWtoA;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_header);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_HEADERW: [::core::ffi::c_ushort; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_ushort; 12]>(
        *b"S\0y\0s\0H\0e\0a\0d\0e\0r\x003\x002\0\0\0",
    )
};

static mut themeClass: [WCHAR; 7] = [
    'H' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[repr(C)]
pub struct HEADER_ITEM {
    pub cxy: INT,
    pub hbm: HBITMAP,
    pub pszText: LPWSTR,
    pub fmt: INT,
    pub lParam: LPARAM,
    pub iImage: INT,
    pub iOrder: INT,
    pub bDown: BOOL,
    pub rect: RECT,
    pub callbackMask: DWORD,
}
pub type PHEADER_ITEM = *mut HEADER_ITEM;

#[repr(C)]
pub struct HEADER_INFO {
    pub hwndSelf: HWND,
    pub hwndNotify: HWND,
    pub nNotifyFormat: INT,
    pub uNumItem: UINT,
    pub nHeight: INT,
    pub hFont: HFONT,
    pub hcurArrow: HCURSOR,
    pub hcurDivider: HCURSOR,
    pub hcurDivopen: HCURSOR,
    pub bCaptured: BOOL,
    pub bPressed: BOOL,
    pub bDragging: BOOL,
    pub bTracking: BOOL,
    pub ptLButtonDown: POINT,
    pub dwStyle: DWORD,
    pub iMoveItem: INT,
    pub xTrackOffset: INT,
    pub xOldTrack: INT,
    pub iHotItem: INT,
    pub iHotDivider: INT,
    pub iMargin: INT,
    pub filter_change_timeout: INT,
    pub himl: HIMAGELIST,
    pub items: *mut HEADER_ITEM,
    pub order: *mut INT,
    pub bRectsValid: BOOL,
}
pub type PHEADER_INFO = *mut HEADER_INFO;

pub const VERT_BORDER: i32 = 4;

pub const DIVIDER_WIDTH: i32 = 10;

pub const HOT_DIVIDER_WIDTH: i32 = 2;

pub const MAX_HEADER_TEXT_LEN: i32 = 260;

pub const HDN_UNICODE_OFFSET: u32 = 20;

pub const HDN_FIRST: u32 = 0u32.wrapping_sub(300); // (0U-300U)

pub const HDN_FIRST_UNICODE: u32 = HDN_FIRST.wrapping_sub(HDN_UNICODE_OFFSET); // (HDN_FIRST-HDN_UNICODE_OFFSET)

pub const HDI_WIDTH: u32 = 0x0001;
pub const HDI_TEXT: u32 = 0x0002;
pub const HDI_FORMAT: u32 = 0x0004;
pub const HDI_LPARAM: u32 = 0x0008;
pub const HDI_BITMAP: u32 = 0x0010;
pub const HDI_IMAGE: u32 = 0x0020;
pub const HDI_DI_SETITEM: u32 = 0x0040;
pub const HDI_ORDER: u32 = 0x0080;
pub const HDI_FILTER: u32 = 0x0100;

pub const HDI_SUPPORTED_FIELDS: u32 = HDI_WIDTH | HDI_TEXT | HDI_FORMAT | HDI_LPARAM | HDI_BITMAP | HDI_IMAGE | HDI_ORDER;

pub const HDI_UNSUPPORTED_FIELDS: u32 = HDI_FILTER;

pub const HDI_UNKNOWN_FIELDS: u32 = !(HDI_SUPPORTED_FIELDS | HDI_UNSUPPORTED_FIELDS | HDI_DI_SETITEM);

pub const HDI_COMCTL32_4_0_FIELDS: u32 = HDI_WIDTH | HDI_TEXT | HDI_FORMAT | HDI_LPARAM | HDI_BITMAP;

unsafe extern "C" fn HEADER_StoreHDItemInHeader(
    mut lpItem: *mut HEADER_ITEM,
    mut mask: UINT,
    mut phdi: *const HDITEMW,
    mut fUnicode: BOOL,
) {
    if mask & (HDI_UNSUPPORTED_FIELDS as UINT) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/header.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"HEADER_StoreHDItemInHeader\0" as *const u8
                    as *const ::core::ffi::c_char,
                119 as ::core::ffi::c_int,
                b"unsupported header fields %x\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (mask & (HDI_UNSUPPORTED_FIELDS as UINT)) as UINT,
            );
        }
    }
    if mask & (HDI_BITMAP as UINT) != 0 {
        (*lpItem).hbm = (*phdi).hbm;
    }
    if mask & (HDI_FORMAT as UINT) != 0 {
        (*lpItem).fmt = (*phdi).fmt as INT;
    }
    if mask & (HDI_LPARAM as UINT) != 0 {
        (*lpItem).lParam = (*phdi).lParam;
    }
    if mask & (HDI_WIDTH as UINT) != 0 {
        (*lpItem).cxy = (*phdi).cxy as INT;
    }
    if mask & (HDI_IMAGE as UINT) != 0 {
        (*lpItem).iImage = (*phdi).iImage as INT;
        if (*phdi).iImage == (I_IMAGECALLBACK as INT) {
            (*lpItem).callbackMask |= (HDI_IMAGE as DWORD);
        } else {
            (*lpItem).callbackMask &= !(HDI_IMAGE as DWORD);
        }
    }
    if mask & (HDI_TEXT as UINT) != 0 {
        heap_free((*lpItem).pszText as *mut ::core::ffi::c_void);
        (*lpItem).pszText = ::core::ptr::null_mut::<WCHAR>();
        if (*phdi).pszText != (LPSTR_TEXTCALLBACKW as LPWSTR) {
            static mut emptyString: [WCHAR; 1] = [0 as ::core::ffi::c_int as WCHAR];
            let mut pszText: LPCWSTR = if !(*phdi).pszText.is_null() {
                (*phdi).pszText as LPCWSTR
            } else {
                &raw const emptyString as LPCWSTR
            };
            if fUnicode != 0 {
                Str_SetPtrW(&raw mut (*lpItem).pszText, pszText);
            } else {
                Str_SetPtrAtoW(&raw mut (*lpItem).pszText, pszText as LPCSTR);
            }
            (*lpItem).callbackMask &= !(HDI_TEXT as DWORD);
        } else {
            (*lpItem).pszText = ::core::ptr::null_mut::<WCHAR>();
            (*lpItem).callbackMask |= (HDI_TEXT as DWORD);
        }
    }
}

unsafe extern "C" fn HEADER_IndexToOrder(infoPtr: *const HEADER_INFO, iItem: INT) -> LRESULT {
    let lpItem: *mut HEADER_ITEM = (*infoPtr).items.offset(iItem as isize) as *mut HEADER_ITEM;
    return (*lpItem).iOrder as LRESULT;
}

unsafe extern "C" fn HEADER_OrderToIndex(infoPtr: *const HEADER_INFO, iorder: INT) -> INT {
    if (iorder < 0) || iorder as UINT >= (*infoPtr).uNumItem {
        return iorder;
    }
    return *(*infoPtr).order.offset(iorder as isize);
}

unsafe extern "C" fn HEADER_ChangeItemOrder(infoPtr: *const HEADER_INFO, iItem: INT, iNewOrder: INT) {
    let mut lpItem: *mut HEADER_ITEM = (*infoPtr).items.offset(iItem as isize) as *mut HEADER_ITEM;
    let mut i: INT = 0;
    let mut nMin: INT = 0;
    let mut nMax: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_ChangeItemOrder\0" as *const u8 as *const ::core::ffi::c_char,
            188 as ::core::ffi::c_int,
            b"%d: %d->%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            iItem,
            (*lpItem).iOrder,
            iNewOrder,
        );
    }
    if (*lpItem).iOrder < iNewOrder {
        memmove(
            (*infoPtr).order.offset((*lpItem).iOrder as isize) as *mut INT as *mut ::core::ffi::c_void,
            (*infoPtr).order.offset(((*lpItem).iOrder + 1) as isize) as *mut INT
                as *const ::core::ffi::c_void,
            ((iNewOrder - (*lpItem).iOrder) as usize).wrapping_mul(::core::mem::size_of::<INT>()),
        );
    }
    if iNewOrder < (*lpItem).iOrder {
        memmove(
            (*infoPtr).order.offset((iNewOrder + 1) as isize) as *mut INT
                as *mut ::core::ffi::c_void,
            (*infoPtr).order.offset(iNewOrder as isize) as *mut INT as *const ::core::ffi::c_void,
            (((*lpItem).iOrder - iNewOrder) as usize).wrapping_mul(::core::mem::size_of::<INT>()),
        );
    }
    *(*infoPtr).order.offset(iNewOrder as isize) = iItem;
    nMin = min((*lpItem).iOrder, iNewOrder);
    nMax = max((*lpItem).iOrder, iNewOrder);
    i = nMin;
    while i <= nMax {
        (*(*infoPtr).items.offset(*(*infoPtr).order.offset(i as isize) as isize)).iOrder = i;
        i += 1;
    }
}

unsafe extern "C" fn HEADER_NextItem(infoPtr: *const HEADER_INFO, iItem: INT) -> INT {
    return HEADER_OrderToIndex(infoPtr, (HEADER_IndexToOrder(infoPtr, iItem) + 1) as INT);
}

unsafe extern "C" fn HEADER_PrevItem(infoPtr: *const HEADER_INFO, iItem: INT) -> INT {
    return HEADER_OrderToIndex(infoPtr, (HEADER_IndexToOrder(infoPtr, iItem) - 1) as INT);
}

unsafe extern "C" fn HEADER_IsItemFixed(infoPtr: *const HEADER_INFO, iItem: INT) -> BOOL {
    return (((*infoPtr).dwStyle & HDS_NOSIZING as DWORD) != 0
        || ((*(*infoPtr).items.offset(iItem as isize)).fmt & (HDF_FIXEDWIDTH as i32)) != 0) as BOOL;
}

unsafe extern "C" fn HEADER_SetItemBounds(infoPtr: *mut HEADER_INFO) {
    let mut phdi: *mut HEADER_ITEM = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();
    let mut i: u32;
    let mut x: i32;
    (*infoPtr).bRectsValid = TRUE;

    if (*infoPtr).uNumItem == 0 {
        return;
    }

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));

    x = rect.left;
    i = 0;
    while i < (*infoPtr).uNumItem {
        phdi = (*infoPtr).items.offset(HEADER_OrderToIndex(infoPtr, i as INT) as isize);
        (*phdi).rect.top = rect.top;
        (*phdi).rect.bottom = rect.bottom;
        (*phdi).rect.left = x;
        (*phdi).rect.right = (*phdi).rect.left + (if (*phdi).cxy > 0 {
            (*phdi).cxy
        } else {
            0
        });
        x = (*phdi).rect.right;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn HEADER_Size(mut infoPtr: *mut HEADER_INFO) -> LRESULT {
    HEADER_SetItemBounds(infoPtr);
    return 0;
}

unsafe extern "C" fn HEADER_GetHotDividerRect(infoPtr: *const HEADER_INFO, r: *mut RECT) {
    let mut iDivider: INT = (*infoPtr).iHotDivider;
    if (*infoPtr).uNumItem > (0 as UINT) {
        let mut lpItem: *mut HEADER_ITEM = core::ptr::null_mut::<HEADER_ITEM>();

        if (iDivider as UINT) < (*infoPtr).uNumItem {
            lpItem = (*infoPtr).items.offset(iDivider as isize) as *mut HEADER_ITEM;
            (*r).left = ((*lpItem).rect.left as INT
                - (HOT_DIVIDER_WIDTH / 2)) as LONG;
            (*r).right = ((*lpItem).rect.left as INT
                + (HOT_DIVIDER_WIDTH / 2)) as LONG;
        } else {
            lpItem = (*infoPtr).items.offset(
                HEADER_OrderToIndex(
                    infoPtr,
                    (*infoPtr).uNumItem.wrapping_sub(1 as UINT) as INT,
                ) as isize,
            ) as *mut HEADER_ITEM;
            (*r).left = ((*lpItem).rect.right as INT
                - (HOT_DIVIDER_WIDTH / 2)) as LONG;
            (*r).right = ((*lpItem).rect.right as INT
                + (HOT_DIVIDER_WIDTH / 2)) as LONG;
        }
        (*r).top = (*lpItem).rect.top;
        (*r).bottom = (*lpItem).rect.bottom;
    } else {
        let mut clientRect: RECT = core::mem::zeroed();
        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(clientRect));
        *r = clientRect;
        (*r).right = ((*r).left as INT
            + (HOT_DIVIDER_WIDTH / 2)) as LONG;
    }
}

unsafe extern "C" fn HEADER_FillItemFrame(
    mut infoPtr: *mut HEADER_INFO,
    mut hdc: HDC,
    mut r: *mut RECT,
    mut item: *const HEADER_ITEM,
    mut hottrack: BOOL,
) {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    if !theme.is_null() {
        let mut state: ::core::ffi::c_int = if (*item).bDown != 0 {
            (HIS_PRESSED) as i32
        } else if hottrack != 0 {
            (HIS_HOT) as i32
        } else {
            (HIS_NORMAL) as i32
        };
        DrawThemeBackground(
            theme,
            hdc,
            (HP_HEADERITEM) as i32,
            state,
            r,
            core::ptr::null_mut::<RECT>(),
        );
        GetThemeBackgroundContentRect(theme, hdc, (HP_HEADERITEM) as i32, state, r, r);
    } else {
        let mut hbr: HBRUSH = CreateSolidBrush(GetBkColor(hdc));
        FillRect(hdc, r as *const RECT, hbr);
        DeleteObject(hbr as HGDIOBJ);
    }
}

unsafe extern "C" fn HEADER_DrawItemFrame(
    infoPtr: *mut HEADER_INFO,
    hdc: HDC,
    r: *mut RECT,
    item: *const HEADER_ITEM,
) {
    if !GetWindowTheme((*infoPtr).hwndSelf).is_null() {
        return;
    }

    if ((*infoPtr).dwStyle & (HDS_FLAT as DWORD)) == 0 {
        if ((*infoPtr).dwStyle & (HDS_BUTTONS as DWORD)) != 0 {
            if (*item).bDown != 0 {
                DrawEdge(
                    hdc,
                    r as LPRECT,
                    (BDR_RAISEDOUTER as UINT),
                    ((BF_RECT as UINT) | (BF_FLAT as UINT) | (BF_ADJUST as UINT)),
                );
            } else {
                DrawEdge(
                    hdc,
                    r as LPRECT,
                    (EDGE_RAISED as UINT),
                    ((BF_RECT as UINT) | (BF_SOFT as UINT) | (BF_ADJUST as UINT)),
                );
            }
        } else {
            DrawEdge(
                hdc,
                r as LPRECT,
                (EDGE_ETCHED as UINT),
                ((BF_BOTTOM as UINT) | (BF_RIGHT as UINT) | (BF_ADJUST as UINT)),
            );
        }
    }
}

unsafe extern "C" fn create_sort_arrow(mut x: INT, mut y: INT, mut h: INT, mut is_up: BOOL) -> HRGN {
    let mut buffer: [core::ffi::c_char; 256] = [0; 256];
    let mut data: *mut RGNDATA = core::ptr::addr_of_mut!(buffer) as *mut RGNDATA;
    let mut size: DWORD = (core::mem::offset_of!(RGNDATA, Buffer)
        + (h as usize).wrapping_mul(core::mem::size_of::<RECT>())) as DWORD;
    let mut i: INT;
    let mut yinc: INT = 1;
    let mut rgn: HRGN = core::ptr::null_mut();

    if (size as usize) > core::mem::size_of::<[core::ffi::c_char; 256]>() {
        data = heap_alloc(size as SIZE_T) as *mut RGNDATA;
        if data.is_null() {
            return core::ptr::null_mut();
        }
    }
    (*data).rdh.dwSize = core::mem::size_of::<RGNDATAHEADER>() as DWORD;
    (*data).rdh.iType = RDH_RECTANGLES as DWORD;
    (*data).rdh.nCount = 0 as DWORD;
    (*data).rdh.nRgnSize =
        (h as usize).wrapping_mul(core::mem::size_of::<RECT>()) as DWORD;

    if is_up == 0 {
        y += h - 1;
        yinc = -1;
    }

    x += h - 1; /* set x to the centre */

    i = 0;
    while i < h {
        let mut rect: *mut RECT = (core::ptr::addr_of_mut!((*data).Buffer)
            as *mut core::ffi::c_char as *mut RECT)
            .offset((*data).rdh.nCount as isize);
        (*rect).left = x - i;
        (*rect).top = y;
        (*rect).right = x + i + 1;
        (*rect).bottom = y + 1;
        (*data).rdh.nCount = (*data).rdh.nCount.wrapping_add(1);
        i += 1;
        y += yinc;
    }
    rgn = ExtCreateRegion(core::ptr::null::<XFORM>(), size, data);
    if data != core::ptr::addr_of_mut!(buffer) as *mut RGNDATA {
        heap_free(data as *mut core::ffi::c_void);
    }
    return rgn;
}

unsafe extern "C" fn HEADER_DrawItem(
    infoPtr: *mut HEADER_INFO,
    hdc: HDC,
    iItem: INT,
    bHotTrack: BOOL,
    lCDFlags: LRESULT,
) -> INT {
    let phdi: *mut HEADER_ITEM = (*infoPtr).items.offset(iItem as isize) as *mut HEADER_ITEM;
    let mut r: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut oldBkMode: INT = 0;
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let mut nmcd: NMCUSTOMDRAW = tagNMCUSTOMDRAWINFO {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        dwDrawStage: 0,
        hdc: ::core::ptr::null_mut::<HDC__>(),
        rc: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        dwItemSpec: 0,
        uItemState: 0,
        lItemlParam: 0,
    };
    let mut state: INT = 0 as INT;
    if ((*__wine_dbch___default).flags as INT
        & ((1 as INT) << __WINE_DBCL_TRACE as INT))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_DrawItem\0" as *const u8 as *const ::core::ffi::c_char,
            381 as ::core::ffi::c_int,
            b"DrawItem(iItem %d bHotTrack %d unicode flag %d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            iItem,
            bHotTrack,
            ((*infoPtr).nNotifyFormat == 2 as INT) as INT,
        );
    }
    r = (*phdi).rect;
    if r.right - r.left == 0 as INT {
        return (*phdi).rect.right as INT;
    }
    if !theme.is_null() {
        state = if (*phdi).bDown != 0 {
            HIS_PRESSED as INT
        } else if bHotTrack != 0 {
            HIS_HOT as INT
        } else {
            HIS_NORMAL as INT
        };
    }
    SetTextColor(
        hdc,
        if bHotTrack != 0 && theme.is_null() {
            comctl32_color.clrHighlight
        } else {
            comctl32_color.clrBtnText
        },
    );
    SetBkColor(hdc, comctl32_color.clr3dFace);
    if (lCDFlags & (CDRF_NOTIFYITEMDRAW as LRESULT)) != 0
        && (((*phdi).fmt as INT) & (HDF_OWNERDRAW as INT)) == 0
    {
        let mut lCDItemFlags: LRESULT = 0;
        nmcd.dwDrawStage = (CDDS_PREPAINT | CDDS_ITEM) as DWORD;
        nmcd.hdc = hdc;
        nmcd.dwItemSpec = iItem as DWORD_PTR;
        nmcd.rc = r;
        nmcd.uItemState = (if (*phdi).bDown != 0 {
            CDIS_SELECTED
        } else {
            0
        }) as UINT;
        nmcd.lItemlParam = (*phdi).lParam;
        lCDItemFlags = HEADER_SendNotify(
            infoPtr,
            NM_CUSTOMDRAW as UINT,
            &raw mut nmcd as *mut NMHDR,
        );
        if (lCDItemFlags & (CDRF_SKIPDEFAULT as LRESULT)) != 0 {
            return (*phdi).rect.right as INT;
        }
    }
    HEADER_FillItemFrame(infoPtr, hdc, &raw mut r, phdi, bHotTrack);
    if (((*phdi).fmt as INT) & (HDF_OWNERDRAW as INT)) != 0 {
        let mut dis: DRAWITEMSTRUCT = tagDRAWITEMSTRUCT {
            CtlType: 0,
            CtlID: 0,
            itemID: 0,
            itemAction: 0,
            itemState: 0,
            hwndItem: ::core::ptr::null_mut::<HWND__>(),
            hDC: ::core::ptr::null_mut::<HDC__>(),
            rcItem: tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            itemData: 0,
        };
        let mut ret: BOOL = 0;
        dis.CtlType = ODT_HEADER as UINT;
        dis.CtlID = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID as INT) as UINT;
        dis.itemID = iItem as UINT;
        dis.itemAction = ODA_DRAWENTIRE as UINT;
        dis.itemState = (if (*phdi).bDown != 0 {
            ODS_SELECTED
        } else {
            0
        }) as UINT;
        dis.hwndItem = (*infoPtr).hwndSelf;
        dis.hDC = hdc;
        dis.rcItem = (*phdi).rect;
        dis.itemData = (*phdi).lParam as ULONG_PTR;
        oldBkMode = SetBkMode(hdc, TRANSPARENT as INT) as INT;
        ret = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_DRAWITEM as UINT,
            dis.CtlID as WPARAM,
            &raw mut dis as LPARAM,
        ) as BOOL;
        if oldBkMode != (TRANSPARENT as INT) {
            SetBkMode(hdc, oldBkMode as INT);
        }
        if ret == 0 {
            HEADER_FillItemFrame(infoPtr, hdc, &raw mut r, phdi, bHotTrack);
        }
        HEADER_DrawItemFrame(infoPtr, hdc, &raw mut r, phdi);
        if ret != 0 {
            return (*phdi).rect.right as INT;
        }
    } else {
        HEADER_DrawItemFrame(infoPtr, hdc, &raw mut r, phdi);
    }
    if (*phdi).bDown != 0 {
        r.left += 2 as LONG;
        r.top += 2 as LONG;
    }
    let mut rw: INT = 0;
    let mut rh: INT = 0;
    let mut x: *mut INT = ::core::ptr::null_mut::<INT>();
    let mut w: *mut UINT = ::core::ptr::null_mut::<UINT>();
    let mut cx: INT = 0;
    let mut tx: INT = 0;
    let mut ix: INT = 0;
    let mut bx: INT = 0;
    let mut cw: UINT = 0;
    let mut tw: UINT = 0;
    let mut iw: UINT = 0;
    let mut bw: UINT = 0;
    let mut img_cx: INT = 0;
    let mut img_cy: INT = 0;
    let mut sort_w: INT = 0;
    let mut sort_x: INT = 0;
    let mut sort_h: INT = 0;
    let mut bmp: BITMAP = tagBITMAP {
        bmType: 0,
        bmWidth: 0,
        bmHeight: 0,
        bmWidthBytes: 0,
        bmPlanes: 0,
        bmBitsPixel: 0,
        bmBits: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    HEADER_PrepareCallbackItems(infoPtr, iItem, (HDI_TEXT | HDI_IMAGE) as INT);
    sort_h = 0 as INT;
    sort_w = sort_h;
    bw = sort_w as UINT;
    iw = bw;
    cw = iw;
    rw = (r.right - r.left) as INT;
    rh = (r.bottom - r.top) as INT;
    if (((*phdi).fmt as INT) & (HDF_STRING as INT)) != 0 {
        let mut textRect: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        SetRectEmpty(&raw mut textRect);
        if !theme.is_null() {
            GetThemeTextExtent(
                theme,
                hdc,
                HP_HEADERITEM as INT,
                state,
                (*phdi).pszText as LPCWSTR,
                -(1 as INT),
                (DT_LEFT | DT_VCENTER | DT_SINGLELINE) as DWORD,
                ::core::ptr::null::<RECT>(),
                &raw mut textRect,
            );
        } else {
            DrawTextW(
                hdc,
                (*phdi).pszText as LPCWSTR,
                -(1 as INT),
                &raw mut textRect,
                (DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_CALCRECT) as UINT,
            );
        }
        cw = (textRect.right as INT - textRect.left as INT
            + (2 as INT) * (*infoPtr).iMargin) as UINT;
    }
    if (((*phdi).fmt as INT) & ((HDF_SORTUP | HDF_SORTDOWN) as INT)) != 0 {
        sort_h = MulDiv(
            (*infoPtr).nHeight - (VERT_BORDER as INT),
            4 as INT,
            13 as INT,
        ) as INT;
        sort_w = ((2 as INT) * sort_h - (1 as INT)
            + (*infoPtr).iMargin * (2 as INT)) as INT;
        cw = cw.wrapping_add(sort_w as UINT);
    } else {
        if (((*phdi).fmt as INT) & (HDF_IMAGE as INT)) != 0
            && ImageList_GetIconSize((*infoPtr).himl, &raw mut img_cx, &raw mut img_cy) != 0
        {
            iw = (img_cx + (2 as INT) * (*infoPtr).iMargin) as UINT;
            x = &raw mut ix;
            w = &raw mut iw;
        }
        if (((*phdi).fmt as INT) & (HDF_BITMAP as INT)) != 0 && !(*phdi).hbm.is_null() {
            GetObjectW(
                (*phdi).hbm as HANDLE,
                core::mem::size_of::<BITMAP>() as INT,
                &raw mut bmp as LPVOID,
            );
            bw = (bmp.bmWidth as INT + (2 as INT) * (*infoPtr).iMargin) as UINT;
            if iw == 0 {
                x = &raw mut bx;
                w = &raw mut bw;
            }
        }
        if bw != 0 || iw != 0 {
            cw = cw.wrapping_add(*w);
        }
    }
    if (((*phdi).fmt as INT) & (HDF_JUSTIFYMASK as INT)) == (HDF_LEFT as INT) {
        cx = r.left as INT;
    } else if (((*phdi).fmt as INT) & (HDF_JUSTIFYMASK as INT)) == (HDF_CENTER as INT) {
        cx = ((r.left as INT + rw / (2 as INT)) as UINT)
            .wrapping_sub(cw.wrapping_div(2 as UINT)) as INT;
    } else {
        cx = (r.right as UINT).wrapping_sub(cw) as INT;
    }
    if cx < r.left as INT {
        cx = r.left as INT;
    }
    if (cx as UINT).wrapping_add(cw) > r.right as UINT {
        cw = (r.right as INT - cx) as UINT;
    }
    tx = cx + (*infoPtr).iMargin;
    tw = cw.wrapping_sub(((*infoPtr).iMargin * (2 as INT)) as UINT);
    tw = tw.wrapping_sub(sort_w as UINT);
    sort_x = (cx as UINT)
        .wrapping_add(tw)
        .wrapping_add(((*infoPtr).iMargin * (3 as INT)) as UINT) as INT;
    if iw != 0 || bw != 0 {
        tw = tw.wrapping_sub(*w);
        if (((*phdi).fmt as INT) & (HDF_BITMAP_ON_RIGHT as INT)) != 0 {
            *x = (cx as UINT)
                .wrapping_add(tw)
                .wrapping_add(((*infoPtr).iMargin * (3 as INT)) as UINT) as INT;
        } else {
            *x = cx + (*infoPtr).iMargin;
            tx = (tx as UINT).wrapping_add(*w) as INT;
        }
    }
    if iw != 0 && bw != 0 {
        if (((*phdi).fmt as INT) & (HDF_JUSTIFYMASK as INT)) == (HDF_RIGHT as INT) {
            bx = (cx as UINT)
                .wrapping_sub(bw)
                .wrapping_add((*infoPtr).iMargin as UINT) as INT;
        } else {
            bx = (cx as UINT)
                .wrapping_add(cw)
                .wrapping_add((*infoPtr).iMargin as UINT) as INT;
        }
    }
    if sort_w != 0 || iw != 0 || bw != 0 {
        let mut hClipDC: HDC = GetDC((*infoPtr).hwndSelf);
        let mut hClipRgn: HRGN = CreateRectRgn(r.left, r.top, r.right, r.bottom);
        SelectClipRgn(hClipDC, hClipRgn);
        if sort_w != 0 {
            let mut arrow: HRGN = create_sort_arrow(
                sort_x,
                r.top as INT + (rh - sort_h) / (2 as INT),
                sort_h,
                ((*phdi).fmt as BOOL) & (HDF_SORTUP as BOOL),
            );
            if !arrow.is_null() {
                FillRgn(
                    hClipDC,
                    arrow,
                    GetSysColorBrush(COLOR_GRAYTEXT as INT),
                );
                DeleteObject(arrow as HGDIOBJ);
            }
        }
        if bw != 0 {
            let mut hdcBitmap: HDC = CreateCompatibleDC(hClipDC);
            SelectObject(hdcBitmap, (*phdi).hbm as HGDIOBJ);
            BitBlt(
                hClipDC,
                bx,
                r.top as INT + (rh - bmp.bmHeight as INT) / (2 as INT),
                bmp.bmWidth as INT,
                bmp.bmHeight as INT,
                hdcBitmap,
                0 as INT,
                0 as INT,
                SRCCOPY as DWORD,
            );
            DeleteDC(hdcBitmap);
        }
        if iw != 0 {
            ImageList_DrawEx(
                (*infoPtr).himl,
                (*phdi).iImage as INT,
                hClipDC,
                ix as INT,
                r.top as INT + (rh as INT - img_cy as INT) / (2 as INT),
                img_cx as INT,
                img_cy as INT,
                CLR_DEFAULT as COLORREF,
                CLR_DEFAULT as COLORREF,
                0 as UINT,
            );
        }
        DeleteObject(hClipRgn as HGDIOBJ);
        ReleaseDC((*infoPtr).hwndSelf, hClipDC);
    }
    if ((((*phdi).fmt as INT) & (HDF_STRING as INT)) != 0
        || (((*phdi).fmt as INT)
            & ((HDF_OWNERDRAW | HDF_STRING | HDF_BITMAP
                | HDF_BITMAP_ON_RIGHT | HDF_IMAGE) as INT))
            == 0)
        && !(*phdi).pszText.is_null()
    {
        oldBkMode = SetBkMode(hdc, TRANSPARENT as INT) as INT;
        r.left = tx as LONG;
        r.right = (tx as UINT).wrapping_add(tw) as LONG;
        if !theme.is_null() {
            DrawThemeText(
                theme,
                hdc,
                HP_HEADERITEM as INT,
                state,
                (*phdi).pszText as LPCWSTR,
                -(1 as INT),
                (DT_LEFT | DT_END_ELLIPSIS | DT_VCENTER | DT_SINGLELINE) as DWORD,
                0 as DWORD,
                &raw mut r,
            );
        } else {
            DrawTextW(
                hdc,
                (*phdi).pszText as LPCWSTR,
                -(1 as INT),
                &raw mut r,
                (DT_LEFT | DT_END_ELLIPSIS | DT_VCENTER | DT_SINGLELINE) as UINT,
            );
        }
        if oldBkMode != (TRANSPARENT as INT) {
            SetBkMode(hdc, oldBkMode as INT);
        }
    }
    HEADER_FreeCallbackItems(phdi);
    return (*phdi).rect.right as INT;
}

unsafe extern "C" fn HEADER_DrawHotDivider(infoPtr: *const HEADER_INFO, hdc: HDC) {
    let mut brush: HBRUSH = core::ptr::null_mut();
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    HEADER_GetHotDividerRect(infoPtr, core::ptr::addr_of_mut!(r));
    brush = CreateSolidBrush(comctl32_color.clrHighlight);
    FillRect(hdc, core::ptr::addr_of_mut!(r) as LPCRECT, brush);
    DeleteObject(brush as HGDIOBJ);
}

unsafe extern "C" fn HEADER_Refresh(infoPtr: *mut HEADER_INFO, hdc: HDC) {
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcRest: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut hbrBk: HBRUSH = core::ptr::null_mut();
    let mut i: UINT = 0;
    let mut x: INT = 0;
    let mut lCDFlags: LRESULT = 0;
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);

    if (*infoPtr).bRectsValid == 0 {
        HEADER_SetItemBounds(infoPtr);
    }

    /* get rect for the bar, adjusted for the border */
    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));
    lCDFlags = HEADER_SendCtrlCustomDraw(infoPtr, CDDS_PREPAINT as DWORD, hdc, core::ptr::addr_of_mut!(rect));

    if (*infoPtr).bDragging != 0 {
        ImageList_DragShowNolock(FALSE as c_int);
    }

    hFont = if !(*infoPtr).hFont.is_null() {
        (*infoPtr).hFont
    } else {
        GetStockObject(SYSTEM_FONT as c_int) as HFONT
    };
    hOldFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;

    /* draw Background */
    if (*infoPtr).uNumItem == (0 as UINT) && theme.is_null() {
        hbrBk = GetSysColorBrush(COLOR_3DFACE as c_int);
        FillRect(hdc, core::ptr::addr_of_mut!(rect), hbrBk);
    }

    x = rect.left as INT;
    i = 0 as UINT;
    while x <= rect.right && i < (*infoPtr).uNumItem {
        let idx: core::ffi::c_int = HEADER_OrderToIndex(infoPtr, i as INT) as core::ffi::c_int;
        if RectVisible(hdc, core::ptr::addr_of_mut!((*(*infoPtr).items.offset(idx as isize)).rect)) != 0 {
            HEADER_DrawItem(
                infoPtr,
                hdc,
                idx as INT,
                ((*infoPtr).iHotItem == idx) as core::ffi::c_int,
                lCDFlags,
            );
        }
        x = (*(*infoPtr).items.offset(idx as isize)).rect.right as INT;
        i = i.wrapping_add(1);
    }

    rcRest = rect;
    rcRest.left = x as LONG;
    if (x <= rect.right)
        && (RectVisible(hdc, core::ptr::addr_of_mut!(rcRest)) != 0)
        && ((*infoPtr).uNumItem > (0 as UINT))
    {
        if !theme.is_null() {
            DrawThemeBackground(
                theme,
                hdc,
                HP_HEADERITEM as core::ffi::c_int,
                HIS_NORMAL as core::ffi::c_int,
                core::ptr::addr_of_mut!(rcRest),
                core::ptr::null(),
            );
        } else if ((*infoPtr).dwStyle & (HDS_FLAT as DWORD)) != 0 {
            hbrBk = GetSysColorBrush(COLOR_3DFACE as c_int);
            FillRect(hdc, core::ptr::addr_of_mut!(rcRest), hbrBk);
        } else {
            if ((*infoPtr).dwStyle & (HDS_BUTTONS as DWORD)) != 0 {
                DrawEdge(
                    hdc,
                    core::ptr::addr_of_mut!(rcRest),
                    EDGE_RAISED as UINT,
                    (BF_TOP | BF_LEFT | BF_BOTTOM | BF_SOFT | BF_MIDDLE) as UINT,
                );
            } else {
                DrawEdge(
                    hdc,
                    core::ptr::addr_of_mut!(rcRest),
                    EDGE_ETCHED as UINT,
                    (BF_BOTTOM | BF_MIDDLE) as UINT,
                );
            }
        }
    }

    if (*infoPtr).iHotDivider != -(1 as core::ffi::c_int) {
        HEADER_DrawHotDivider(infoPtr, hdc);
    }

    if (*infoPtr).bDragging != 0 {
        ImageList_DragShowNolock(TRUE as c_int);
    }
    SelectObject(hdc, hOldFont as HGDIOBJ);

    if (lCDFlags & (CDRF_NOTIFYPOSTPAINT as LRESULT)) != 0 {
        HEADER_SendCtrlCustomDraw(infoPtr, CDDS_POSTPAINT as DWORD, hdc, core::ptr::addr_of_mut!(rect));
    }
}

unsafe extern "C" fn HEADER_RefreshItem(infoPtr: *mut HEADER_INFO, iItem: INT) {
    if (*infoPtr).bRectsValid == 0 {
        HEADER_SetItemBounds(infoPtr);
    }

    InvalidateRect((*infoPtr).hwndSelf, &raw mut (*(*infoPtr).items.offset(iItem as isize)).rect as LPCRECT, FALSE);
}

unsafe extern "C" fn HEADER_InternalHitTest(infoPtr: *const HEADER_INFO, lpPt: *const POINT, pFlags: *mut UINT, pItem: *mut INT) {
    let mut rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcTest: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut iCount: UINT = 0;
    let mut width: INT = 0;
    let mut bNoWidth: BOOL = 0;

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));

    *pFlags = 0;
    bNoWidth = FALSE;
    if PtInRect(core::ptr::addr_of_mut!(rect), *lpPt) != 0 {
        if (*infoPtr).uNumItem == 0 {
            *pFlags |= HHT_NOWHERE;
            *pItem = 1;
            return;
        } else {
            /* somewhere inside */
            iCount = 0;
            while iCount < (*infoPtr).uNumItem {
                'body: {
                    rect = (*(*infoPtr).items.offset(iCount as isize)).rect;
                    width = rect.right - rect.left;
                    if width == 0 {
                        bNoWidth = TRUE;
                        break 'body;
                    }
                    if PtInRect(core::ptr::addr_of_mut!(rect), *lpPt) != 0 {
                        if width <= 2 * DIVIDER_WIDTH {
                            *pFlags |= HHT_ONHEADER;
                            *pItem = iCount as INT;
                            return;
                        }
                        if HEADER_IndexToOrder(infoPtr, iCount as INT) > 0 {
                            rcTest = rect;
                            rcTest.right = rcTest.left + DIVIDER_WIDTH;
                            if PtInRect(core::ptr::addr_of_mut!(rcTest), *lpPt) != 0 {
                                if HEADER_IsItemFixed(infoPtr, HEADER_PrevItem(infoPtr, iCount as INT)) != 0 {
                                    *pFlags |= HHT_ONHEADER;
                                    *pItem = iCount as INT;
                                    return;
                                }
                                if bNoWidth != 0 {
                                    *pFlags |= HHT_ONDIVOPEN;
                                    *pItem = HEADER_PrevItem(infoPtr, iCount as INT);
                                    return;
                                } else {
                                    *pFlags |= HHT_ONDIVIDER;
                                    *pItem = HEADER_PrevItem(infoPtr, iCount as INT);
                                    return;
                                }
                            }
                        }
                        rcTest = rect;
                        rcTest.left = rcTest.right - DIVIDER_WIDTH;
                        if HEADER_IsItemFixed(infoPtr, iCount as INT) == 0
                            && PtInRect(core::ptr::addr_of_mut!(rcTest), *lpPt) != 0
                        {
                            *pFlags |= HHT_ONDIVIDER;
                            *pItem = iCount as INT;
                            return;
                        }

                        *pFlags |= HHT_ONHEADER;
                        *pItem = iCount as INT;
                        return;
                    }
                }
                iCount = iCount.wrapping_add(1);
            }

            /* check for last divider part (on nowhere) */
            if HEADER_IsItemFixed(infoPtr, (*infoPtr).uNumItem.wrapping_sub(1) as INT) == 0 {
                rect = (*(*infoPtr).items.offset((*infoPtr).uNumItem.wrapping_sub(1) as isize)).rect;
                rect.left = rect.right;
                rect.right += DIVIDER_WIDTH;
                if PtInRect(core::ptr::addr_of_mut!(rect), *lpPt) != 0 {
                    if bNoWidth != 0 {
                        *pFlags |= HHT_ONDIVOPEN;
                        *pItem = (*infoPtr).uNumItem.wrapping_sub(1) as INT;
                        return;
                    } else {
                        *pFlags |= HHT_ONDIVIDER;
                        *pItem = (*infoPtr).uNumItem.wrapping_sub(1) as INT;
                        return;
                    }
                }
            }

            *pFlags |= HHT_NOWHERE;
            *pItem = 1;
            return;
        }
    } else {
        if (*lpPt).x < rect.left {
            *pFlags |= HHT_TOLEFT;
        } else if (*lpPt).x > rect.right {
            *pFlags |= HHT_TORIGHT;
        }

        if (*lpPt).y < rect.top {
            *pFlags |= HHT_ABOVE;
        } else if (*lpPt).y > rect.bottom {
            *pFlags |= HHT_BELOW;
        }
    }

    *pItem = 1;
    return;
}

unsafe extern "C" fn HEADER_DrawTrackLine(infoPtr: *const HEADER_INFO, hdc: HDC, x: INT) {
    let mut rect: RECT = core::mem::zeroed();
    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));
    PatBlt(hdc, x, rect.top, 1, rect.bottom - rect.top, DSTINVERT);
}

unsafe extern "C" fn HEADER_CopyHDItemForNotify(
    infoPtr: *const HEADER_INFO,
    dest: *mut HDITEMW,
    src: *const HDITEMW,
    fSourceUnicode: BOOL,
    ppvScratch: *mut LPVOID,
) {
    *ppvScratch = core::ptr::null_mut();
    *dest = *src;

    if ((*src).mask & HDI_TEXT as UINT) != 0 && (*src).pszText != LPSTR_TEXTCALLBACKW {
        if fSourceUnicode != 0 && (*infoPtr).nNotifyFormat != NFR_UNICODE as INT {
            (*dest).pszText = core::ptr::null_mut();
            Str_SetPtrWtoA(
                core::ptr::addr_of_mut!((*dest).pszText) as *mut LPSTR,
                (*src).pszText as LPCWSTR,
            );
            *ppvScratch = (*dest).pszText as LPVOID;
        }

        if fSourceUnicode == 0 && (*infoPtr).nNotifyFormat == NFR_UNICODE as INT {
            (*dest).pszText = core::ptr::null_mut();
            Str_SetPtrAtoW(
                core::ptr::addr_of_mut!((*dest).pszText),
                (*src).pszText as LPSTR as LPCSTR,
            );
            *ppvScratch = (*dest).pszText as LPVOID;
        }
    }
}

unsafe extern "C" fn HEADER_NotifyCodeWtoA(mut code: UINT) -> UINT {
    /* we use the fact that all the unicode messages are in HDN_FIRST_UNICODE..HDN_LAST*/
    if code >= (HDN_LAST) as u32 && code <= HDN_FIRST_UNICODE {
        return code + HDN_UNICODE_OFFSET;
    } else {
        return code;
    };
}

unsafe extern "C" fn HEADER_SendNotify(infoPtr: *const HEADER_INFO, code: UINT, nmhdr: *mut NMHDR) -> LRESULT {
    (*nmhdr).hwndFrom = (*infoPtr).hwndSelf;
    (*nmhdr).idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    (*nmhdr).code = code;

    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        (*nmhdr).idFrom as WPARAM,
        nmhdr as LPARAM,
    );
}

unsafe extern "C" fn HEADER_SendSimpleNotify(infoPtr: *const HEADER_INFO, code: UINT) -> BOOL {
    let mut nmhdr: NMHDR = NMHDR {
        hwndFrom: core::ptr::null_mut(),
        idFrom: 0,
        code: 0,
    };
    return HEADER_SendNotify(infoPtr, code, core::ptr::addr_of_mut!(nmhdr)) as BOOL;
}

unsafe extern "C" fn HEADER_SendCtrlCustomDraw(
    infoPtr: *const HEADER_INFO,
    dwDrawStage: DWORD,
    hdc: HDC,
    rect: *const RECT,
) -> LRESULT {
    let mut nm: NMCUSTOMDRAW = core::mem::zeroed();
    nm.dwDrawStage = dwDrawStage;
    nm.hdc = hdc;
    nm.rc = *rect;
    nm.dwItemSpec = 0;
    nm.uItemState = 0;
    nm.lItemlParam = 0;

    return HEADER_SendNotify(infoPtr, NM_CUSTOMDRAW, core::ptr::addr_of_mut!(nm) as *mut NMHDR);
}

unsafe extern "C" fn HEADER_SendNotifyWithHDItemT(infoPtr: *const HEADER_INFO, mut code: UINT, iItem: INT, lpItem: *mut HDITEMW) -> BOOL {
    let mut nmhdr: NMHEADERW = core::mem::zeroed();

    if ((*infoPtr).nNotifyFormat) as u32 != NFR_UNICODE {
        code = HEADER_NotifyCodeWtoA(code);
    }
    nmhdr.iItem = iItem;
    nmhdr.iButton = 0;
    nmhdr.pitem = lpItem;

    return HEADER_SendNotify(infoPtr, code, core::ptr::addr_of_mut!(nmhdr) as *mut NMHDR) as BOOL;
}

unsafe extern "C" fn HEADER_SendNotifyWithIntFieldT(infoPtr: *const HEADER_INFO, code: UINT, iItem: INT, mask: INT, iValue: INT) -> BOOL {
    let mut nmitem: HDITEMW = core::mem::zeroed();

    /* copying only the iValue should be ok but to make the code more robust we copy everything */
    nmitem.cxy = (*(*infoPtr).items.offset(iItem as isize)).cxy as ::core::ffi::c_int;
    nmitem.hbm = (*(*infoPtr).items.offset(iItem as isize)).hbm;
    nmitem.pszText = core::ptr::null_mut();
    nmitem.cchTextMax = 0;
    nmitem.fmt = (*(*infoPtr).items.offset(iItem as isize)).fmt as ::core::ffi::c_int;
    nmitem.lParam = (*(*infoPtr).items.offset(iItem as isize)).lParam;
    nmitem.iOrder = (*(*infoPtr).items.offset(iItem as isize)).iOrder as ::core::ffi::c_int;
    nmitem.iImage = (*(*infoPtr).items.offset(iItem as isize)).iImage as ::core::ffi::c_int;

    nmitem.mask = mask as UINT;
    match (mask) as u32 {
        HDI_WIDTH => {
            nmitem.cxy = iValue as ::core::ffi::c_int;
        }
        HDI_ORDER => {
            nmitem.iOrder = iValue as ::core::ffi::c_int;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/header.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"HEADER_SendNotifyWithIntFieldT\0" as *const u8
                        as *const ::core::ffi::c_char,
                    954 as ::core::ffi::c_int,
                    b"invalid mask value 0x%x\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    iValue,
                );
            }
        }
    }
    return HEADER_SendNotifyWithHDItemT(infoPtr, code, iItem, core::ptr::addr_of_mut!(nmitem));
}

unsafe extern "C" fn HEADER_PrepareCallbackItems(infoPtr: *const HEADER_INFO, iItem: INT, reqMask: INT) -> BOOL {
    let mut lpItem: *mut HEADER_ITEM = (*infoPtr).items.offset(iItem as isize) as *mut HEADER_ITEM;
    let mut mask: DWORD = (reqMask as DWORD) & (*lpItem).callbackMask;
    let mut dispInfo: NMHDDISPINFOW = core::mem::zeroed();
    let mut pvBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    if mask == (0 as DWORD) {
        return TRUE;
    }
    if (mask & (HDI_TEXT as DWORD)) != 0 && !(*lpItem).pszText.is_null() {
        heap_free((*lpItem).pszText as *mut core::ffi::c_void);
        (*lpItem).pszText = core::ptr::null_mut();
    }

    memset(
        core::ptr::addr_of_mut!(dispInfo) as *mut core::ffi::c_void,
        (0) as core::ffi::c_int,
        (core::mem::size_of::<NMHDDISPINFOW>() as SIZE_T) as _,
    );
    dispInfo.hdr.hwndFrom = (*infoPtr).hwndSelf;
    dispInfo.hdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, (GWLP_ID) as core::ffi::c_int) as UINT_PTR;
    if (*infoPtr).nNotifyFormat == (NFR_UNICODE) as INT {
        dispInfo.hdr.code = (HDN_GETDISPINFOW) as UINT;
        if (mask & (HDI_TEXT as DWORD)) != 0 {
            pvBuffer = heap_alloc_zero(
                (MAX_HEADER_TEXT_LEN as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            );
        }
    } else {
        dispInfo.hdr.code = (HDN_GETDISPINFOA) as UINT;
        if (mask & (HDI_TEXT as DWORD)) != 0 {
            pvBuffer = heap_alloc_zero(
                (MAX_HEADER_TEXT_LEN as SIZE_T).wrapping_mul(core::mem::size_of::<CHAR>() as SIZE_T),
            );
        }
    }
    dispInfo.pszText = pvBuffer as LPWSTR;
    dispInfo.cchTextMax = if !pvBuffer.is_null() {
        MAX_HEADER_TEXT_LEN as INT
    } else {
        0 as INT
    };
    dispInfo.iItem = iItem as INT;
    dispInfo.mask = mask as UINT;
    dispInfo.lParam = (*lpItem).lParam;

    SendMessageW(
        (*infoPtr).hwndNotify,
        (WM_NOTIFY) as UINT,
        dispInfo.hdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(dispInfo) as LPARAM,
    );

    if (mask & (HDI_IMAGE as DWORD)) != 0 {
        (*lpItem).iImage = dispInfo.iImage as INT;
    }
    if (mask & (HDI_TEXT as DWORD)) != 0 {
        if (*infoPtr).nNotifyFormat == (NFR_UNICODE) as INT {
            (*lpItem).pszText = pvBuffer as LPWSTR;

            /* the user might have used his own buffer */
            if dispInfo.pszText != (*lpItem).pszText {
                Str_GetPtrW(
                    dispInfo.pszText as LPCWSTR,
                    (*lpItem).pszText,
                    (MAX_HEADER_TEXT_LEN) as core::ffi::c_int,
                );
            }
        } else {
            Str_SetPtrAtoW(
                core::ptr::addr_of_mut!((*lpItem).pszText),
                dispInfo.pszText as LPSTR as LPCSTR,
            );
            heap_free(pvBuffer);
        }
    }

    if (dispInfo.mask & (HDI_DI_SETITEM as UINT)) != 0 {
        /* make the items permanent */
        (*lpItem).callbackMask &= !dispInfo.mask;
    }

    return TRUE;
}

unsafe extern "C" fn HEADER_FreeCallbackItems(mut lpItem: *mut HEADER_ITEM) {
    if (*lpItem).callbackMask & HDI_TEXT as DWORD != 0 {
        heap_free((*lpItem).pszText as *mut ::core::ffi::c_void);
        (*lpItem).pszText = ::core::ptr::null_mut::<WCHAR>();
    }
    if (*lpItem).callbackMask & HDI_IMAGE as DWORD != 0 {
        (*lpItem).iImage = I_IMAGECALLBACK as INT;
    }
}

unsafe extern "C" fn HEADER_CreateDragImage(infoPtr: *mut HEADER_INFO, iItem: INT) -> HIMAGELIST {
    let mut lpItem: *mut HEADER_ITEM = core::ptr::null_mut();
    let mut himl: HIMAGELIST = core::ptr::null_mut();
    let mut hMemory: HBITMAP = core::ptr::null_mut();
    let mut hOldBitmap: HBITMAP = core::ptr::null_mut();
    let mut lCDFlags: LRESULT = 0;
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hMemoryDC: HDC = core::ptr::null_mut();
    let mut hDeviceDC: HDC = core::ptr::null_mut();
    let mut height: INT = 0;
    let mut width: INT = 0;
    let mut hFont: HFONT = core::ptr::null_mut();

    if iItem as UINT >= (*infoPtr).uNumItem {
        return core::ptr::null_mut();
    }

    if (*infoPtr).bRectsValid == 0 {
        HEADER_SetItemBounds(infoPtr);
    }

    lpItem = (*infoPtr).items.offset(iItem as isize) as *mut HEADER_ITEM;
    width = ((*lpItem).rect.right - (*lpItem).rect.left) as INT;
    height = ((*lpItem).rect.bottom - (*lpItem).rect.top) as INT;

    hDeviceDC = GetDC(core::ptr::null_mut());
    hMemoryDC = CreateCompatibleDC(hDeviceDC);
    hMemory = CreateCompatibleBitmap(hDeviceDC, width, height);
    ReleaseDC(core::ptr::null_mut(), hDeviceDC);
    hOldBitmap = SelectObject(hMemoryDC, hMemory as HGDIOBJ) as HBITMAP;
    SetViewportOrgEx(
        hMemoryDC,
        -((*lpItem).rect.left as INT),
        -((*lpItem).rect.top as INT),
        core::ptr::null_mut(),
    );
    hFont = if !(*infoPtr).hFont.is_null() {
        (*infoPtr).hFont
    } else {
        GetStockObject((SYSTEM_FONT as i32)) as HFONT
    };
    SelectObject(hMemoryDC, hFont as HGDIOBJ);

    GetClientRect((*infoPtr).hwndSelf, &mut rc as *mut RECT);
    lCDFlags = HEADER_SendCtrlCustomDraw(
        infoPtr,
        CDDS_PREPAINT as DWORD,
        hMemoryDC,
        &mut rc as *mut RECT,
    );
    HEADER_DrawItem(
        infoPtr,
        hMemoryDC,
        iItem,
        FALSE as BOOL,
        lCDFlags,
    );
    if (lCDFlags & (CDRF_NOTIFYPOSTPAINT as LRESULT)) != 0 {
        HEADER_SendCtrlCustomDraw(
            infoPtr,
            CDDS_POSTPAINT as DWORD,
            hMemoryDC,
            &mut rc as *mut RECT,
        );
    }

    hMemory = SelectObject(hMemoryDC, hOldBitmap as HGDIOBJ) as HBITMAP;
    DeleteDC(hMemoryDC);

    if hMemory.is_null() {
        return core::ptr::null_mut();
    }

    himl = ImageList_Create(
        width,
        height,
        ILC_COLORDDB as UINT,
        1 as INT,
        1 as INT,
    );
    ImageList_Add(himl, hMemory, core::ptr::null_mut());
    DeleteObject(hMemory as HGDIOBJ);
    return himl;
}

unsafe extern "C" fn HEADER_SetHotDivider(
    mut infoPtr: *mut HEADER_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut iDivider: INT = 0;
    let mut r: RECT = core::mem::zeroed();
    if wParam != 0 {
        let mut pt: POINT = core::mem::zeroed();
        let mut flags: UINT = 0;
        pt.x = (LOWORD!(lParam as DWORD) as SHORT) as INT as LONG;
        pt.y = 0 as LONG;
        HEADER_InternalHitTest(
            infoPtr,
            core::ptr::addr_of_mut!(pt),
            core::ptr::addr_of_mut!(flags),
            core::ptr::addr_of_mut!(iDivider),
        );
        if (flags & (HHT_TOLEFT as UINT)) != 0 {
            iDivider = 0 as INT;
        } else if (flags & (HHT_NOWHERE as UINT)) != 0
            || (flags & (HHT_TORIGHT as UINT)) != 0
        {
            iDivider = (*infoPtr).uNumItem as INT;
        } else {
            let mut lpItem: *mut HEADER_ITEM =
                (*infoPtr).items.offset(iDivider as isize) as *mut HEADER_ITEM;
            if pt.x
                > ((*lpItem).rect.left as INT + (*lpItem).rect.right as INT)
                    / (2 as INT)
            {
                iDivider = HEADER_NextItem(infoPtr, iDivider);
            }
        }
    } else {
        iDivider = lParam as INT;
    }

    /* Note; wParam==FALSE, lParam==-1 is valid and is used to clear the hot divider */
    if iDivider < -(1 as INT) || iDivider > (*infoPtr).uNumItem as INT {
        return iDivider as LRESULT;
    }

    if iDivider != (*infoPtr).iHotDivider {
        if (*infoPtr).iHotDivider != -(1 as INT) {
            HEADER_GetHotDividerRect(infoPtr, core::ptr::addr_of_mut!(r));
            InvalidateRect(
                (*infoPtr).hwndSelf,
                core::ptr::addr_of_mut!(r) as LPCRECT,
                FALSE as BOOL,
            );
        }
        (*infoPtr).iHotDivider = iDivider;
        if iDivider != -(1 as INT) {
            HEADER_GetHotDividerRect(infoPtr, core::ptr::addr_of_mut!(r));
            InvalidateRect(
                (*infoPtr).hwndSelf,
                core::ptr::addr_of_mut!(r) as LPCRECT,
                FALSE as BOOL,
            );
        }
    }
    return iDivider as LRESULT;
}

unsafe extern "C" fn HEADER_DeleteItem(infoPtr: *mut HEADER_INFO, iItem: INT) -> LRESULT {
    let mut iOrder: INT = 0;
    let mut i: UINT = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            1175 as ::core::ffi::c_int,
            b"[iItem=%d]\n\0" as *const u8 as *const ::core::ffi::c_char,
            iItem,
        );
    }

    if iItem < 0 || iItem >= (*infoPtr).uNumItem as INT {
        return FALSE as LRESULT;
    }

    i = 0 as UINT;
    while i < (*infoPtr).uNumItem {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/header.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"HEADER_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
                1181 as ::core::ffi::c_int,
                b"%d: order=%d, iOrder=%d, ->iOrder=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                i,
                *(*infoPtr).order.offset(i as isize),
                (*(*infoPtr).items.offset(i as isize)).iOrder,
                (*(*infoPtr).items.offset(*(*infoPtr).order.offset(i as isize) as isize))
                    .iOrder,
            );
        }
        i = i.wrapping_add(1);
    }

    iOrder = (*(*infoPtr).items.offset(iItem as isize)).iOrder;
    heap_free((*(*infoPtr).items.offset(iItem as isize)).pszText as *mut ::core::ffi::c_void);

    (*infoPtr).uNumItem = (*infoPtr).uNumItem.wrapping_sub(1);
    memmove(
        (*infoPtr).items.offset(iItem as isize) as *mut HEADER_ITEM as *mut ::core::ffi::c_void,
        (*infoPtr)
            .items
            .offset((iItem + 1) as isize)
            as *mut HEADER_ITEM as *const ::core::ffi::c_void,
        ((*infoPtr).uNumItem.wrapping_sub(iItem as UINT) as size_t)
            .wrapping_mul(::core::mem::size_of::<HEADER_ITEM>() as size_t),
    );
    memmove(
        (*infoPtr).order.offset(iOrder as isize) as *mut INT as *mut ::core::ffi::c_void,
        (*infoPtr)
            .order
            .offset((iOrder + 1) as isize)
            as *mut INT as *const ::core::ffi::c_void,
        ((*infoPtr).uNumItem.wrapping_sub(iOrder as UINT) as size_t)
            .wrapping_mul(::core::mem::size_of::<INT>() as size_t),
    );
    (*infoPtr).items = heap_realloc(
        (*infoPtr).items as *mut ::core::ffi::c_void,
        (::core::mem::size_of::<HEADER_ITEM>() as SIZE_T)
            .wrapping_mul((*infoPtr).uNumItem as SIZE_T),
    ) as *mut HEADER_ITEM;
    (*infoPtr).order = heap_realloc(
        (*infoPtr).order as *mut ::core::ffi::c_void,
        (::core::mem::size_of::<INT>() as SIZE_T).wrapping_mul((*infoPtr).uNumItem as SIZE_T),
    ) as *mut INT;

    i = 0 as UINT;
    while i < (*infoPtr).uNumItem {
        if *(*infoPtr).order.offset(i as isize) > iItem {
            *(*infoPtr).order.offset(i as isize) -= 1;
        }
        if i >= iOrder as UINT {
            (*(*infoPtr)
                .items
                .offset(*(*infoPtr).order.offset(i as isize) as isize))
            .iOrder = i as INT;
        }
        i = i.wrapping_add(1);
    }

    i = 0 as UINT;
    while i < (*infoPtr).uNumItem {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/header.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"HEADER_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
                1203 as ::core::ffi::c_int,
                b"%d: order=%d, iOrder=%d, ->iOrder=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                i,
                *(*infoPtr).order.offset(i as isize),
                (*(*infoPtr).items.offset(i as isize)).iOrder,
                (*(*infoPtr).items.offset(*(*infoPtr).order.offset(i as isize) as isize))
                    .iOrder,
            );
        }
        i = i.wrapping_add(1);
    }

    HEADER_SetItemBounds(infoPtr);
    InvalidateRect(
        (*infoPtr).hwndSelf,
        ::core::ptr::null::<RECT>(),
        FALSE as BOOL,
    );

    return TRUE as LRESULT;
}

unsafe extern "C" fn HEADER_GetImageList(infoPtr: *const HEADER_INFO) -> LRESULT {
    return (*infoPtr).himl as LRESULT;
}

unsafe extern "C" fn HEADER_GetItemT(
    mut infoPtr: *const HEADER_INFO,
    mut nItem: INT,
    mut phdi: LPHDITEMW,
    mut bUnicode: BOOL,
) -> LRESULT {
    let mut lpItem: *mut HEADER_ITEM = ::core::ptr::null_mut::<HEADER_ITEM>();
    let mut mask: UINT = 0;
    if phdi.is_null() {
        return FALSE as LRESULT;
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
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
            1228 as ::core::ffi::c_int,
            b"[nItem=%d]\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItem,
        );
    }
    mask = (*phdi).mask;
    if mask == 0 as UINT {
        return TRUE as LRESULT;
    }
    if nItem < 0 as INT || nItem >= (*infoPtr).uNumItem as INT {
        return FALSE as LRESULT;
    }
    if mask & HDI_UNKNOWN_FIELDS as UINT != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/header.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"HEADER_GetItemT\0" as *const u8 as *const ::core::ffi::c_char,
                1239 as ::core::ffi::c_int,
                b"mask %x contains unknown fields. Using only comctl32 4.0 fields\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                mask,
            );
        }
        mask &= HDI_COMCTL32_4_0_FIELDS as UINT;
    }
    lpItem = (*infoPtr).items.offset(nItem as isize) as *mut HEADER_ITEM;
    HEADER_PrepareCallbackItems(infoPtr, nItem, mask as INT);
    if mask & HDI_BITMAP as UINT != 0 {
        (*phdi).hbm = (*lpItem).hbm;
    }
    if mask & HDI_FORMAT as UINT != 0 {
        (*phdi).fmt = (*lpItem).fmt as ::core::ffi::c_int;
    }
    if mask & HDI_WIDTH as UINT != 0 {
        (*phdi).cxy = (*lpItem).cxy as ::core::ffi::c_int;
    }
    if mask & HDI_LPARAM as UINT != 0 {
        (*phdi).lParam = (*lpItem).lParam;
    }
    if mask & HDI_IMAGE as UINT != 0 {
        (*phdi).iImage = (*lpItem).iImage as ::core::ffi::c_int;
    }
    if mask & HDI_ORDER as UINT != 0 {
        (*phdi).iOrder = (*lpItem).iOrder as ::core::ffi::c_int;
    }
    if mask & HDI_TEXT as UINT != 0 {
        if bUnicode != 0 {
            Str_GetPtrW(
                (*lpItem).pszText as LPCWSTR,
                (*phdi).pszText,
                (*phdi).cchTextMax as INT,
            );
        } else {
            Str_GetPtrWtoA(
                (*lpItem).pszText as LPCWSTR,
                (*phdi).pszText as LPSTR,
                (*phdi).cchTextMax as INT,
            );
        }
    }
    HEADER_FreeCallbackItems(lpItem);
    return TRUE as LRESULT;
}

unsafe extern "C" fn HEADER_GetItemCount(infoPtr: *const HEADER_INFO) -> LRESULT {
    ((*infoPtr).uNumItem) as _
}

unsafe extern "C" fn HEADER_GetItemRect(infoPtr: *const HEADER_INFO, iItem: INT, lpRect: LPRECT) -> LRESULT {
    if (iItem < (0 as INT)) || (iItem >= ((*infoPtr).uNumItem as INT)) {
        return FALSE as LRESULT;
    }

    (*lpRect).left = (*(*infoPtr).items.offset(iItem as isize)).rect.left;
    (*lpRect).right = (*(*infoPtr).items.offset(iItem as isize)).rect.right;
    (*lpRect).top = (*(*infoPtr).items.offset(iItem as isize)).rect.top;
    (*lpRect).bottom = (*(*infoPtr).items.offset(iItem as isize)).rect.bottom;

    return TRUE as LRESULT;
}

unsafe extern "C" fn HEADER_GetOrderArray(infoPtr: *const HEADER_INFO, size: INT, order: LPINT) -> LRESULT {
    if (size as UINT) < (*infoPtr).uNumItem {
        return (FALSE) as LRESULT;
    }
    memcpy(
        order as *mut core::ffi::c_void,
        (*infoPtr).order as *const core::ffi::c_void,
        ((*infoPtr).uNumItem as usize).wrapping_mul(core::mem::size_of::<INT>()),
    );
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn has_duplicate(array: *const INT, to: INT, value: INT) -> INT {
    let mut i: INT;
    i = 0 as INT;
    while i < to {
        if *array.offset(i as isize) == value {
            return i;
        }
        i += 1;
    }
    return -(1 as INT);
}

unsafe extern "C" fn get_nextvalue(array: *const INT, to: INT, max: INT) -> INT {
    let mut i: INT = 0;
    while i < max {
        if has_duplicate(array, to, i) == -1 {
            return i;
        }
        i += 1;
    }
    return 0;
}
unsafe extern "C" fn HEADER_SetOrderArray(infoPtr: *mut HEADER_INFO, size: INT, order: *const INT) -> LRESULT {
    let mut lpItem: *mut HEADER_ITEM = core::ptr::null_mut();
    let mut i: INT;

    if (size as UINT) != (*infoPtr).uNumItem {
        return (FALSE) as LRESULT;
    }

    if TRACE_ON!(header) {
        i = 0;
        while i < size {
            TRACE!(b"%d%c\0".as_ptr() as *const i8, *order.offset(i as isize), if i != size - 1 { ',' as i32 } else { '}' as i32 });
            i += 1;
        }
    }

    i = 0;
    while i < size {
        if *order.offset(i as isize) >= size || *order.offset(i as isize) < 0 {
            /* on invalid index get next available */
            /* FIXME: if i==0 array item is out of range behaviour is
                      different, see tests */
            *(*infoPtr).order.offset(i as isize) = get_nextvalue((*infoPtr).order, i, size);
        } else {
            let mut j: INT = 0;
            let mut dup: INT = 0;

            *(*infoPtr).order.offset(i as isize) = *order.offset(i as isize);
            j = i;
            /* remove duplicates */
            loop {
                dup = has_duplicate((*infoPtr).order, j, *order.offset(j as isize));
                if !(dup != -1) {
                    break;
                }
                let mut next: INT = 0;

                next = get_nextvalue((*infoPtr).order, j, size);
                *(*infoPtr).order.offset(dup as isize) = next;
                j -= 1;
            }
        }
        i += 1;
    }
    /* sync with item data */
    i = 0;
    while i < size {
        lpItem = (*infoPtr).items.offset(*(*infoPtr).order.offset(i as isize) as isize) as *mut HEADER_ITEM;
        (*lpItem).iOrder = i;
        i += 1;
    }
    HEADER_SetItemBounds(infoPtr);
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), FALSE);
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn HEADER_GetUnicodeFormat(infoPtr: *const HEADER_INFO) -> LRESULT {
    return (((*infoPtr).nNotifyFormat) as u32 == NFR_UNICODE) as LRESULT;
}

unsafe extern "C" fn HEADER_HitTest(infoPtr: *const HEADER_INFO, phti: LPHDHITTESTINFO) -> LRESULT {
    let outside: UINT = HHT_NOWHERE | HHT_ABOVE | HHT_BELOW | HHT_TOLEFT | HHT_TORIGHT;

    HEADER_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!((*phti).pt),
        core::ptr::addr_of_mut!((*phti).flags),
        core::ptr::addr_of_mut!((*phti).iItem),
    );

    if ((*phti).flags & outside) != 0 {
        (*phti).iItem = -1;
        return (*phti).iItem as LRESULT;
    } else {
        return (*phti).iItem as LRESULT;
    }
}

unsafe extern "C" fn HEADER_InsertItemT(
    infoPtr: *mut HEADER_INFO,
    mut nItem: INT,
    phdi: *const HDITEMW,
    bUnicode: BOOL,
) -> LRESULT {
    let mut lpItem: *mut HEADER_ITEM = core::ptr::null_mut();
    let mut iOrder: INT = 0;
    let mut i: UINT = 0 as UINT;
    let mut copyMask: UINT = 0;

    if phdi.is_null() || nItem < 0 || (*phdi).mask == 0 {
        return -(1 as LRESULT);
    }

    if nItem as UINT > (*infoPtr).uNumItem {
        nItem = (*infoPtr).uNumItem as INT;
    }

    iOrder = (if ((*phdi).mask & (HDI_ORDER as UINT)) != 0 {
        (*phdi).iOrder
    } else {
        nItem
    }) as INT;
    if iOrder < 0 {
        iOrder = 0;
    } else if (*infoPtr).uNumItem < iOrder as UINT {
        iOrder = (*infoPtr).uNumItem as INT;
    }

    (*infoPtr).uNumItem = (*infoPtr).uNumItem.wrapping_add(1);
    (*infoPtr).items = heap_realloc(
        (*infoPtr).items as *mut core::ffi::c_void,
        (core::mem::size_of::<HEADER_ITEM>() as SIZE_T)
            .wrapping_mul((*infoPtr).uNumItem as SIZE_T),
    ) as *mut HEADER_ITEM;
    (*infoPtr).order = heap_realloc(
        (*infoPtr).order as *mut core::ffi::c_void,
        (core::mem::size_of::<INT>() as SIZE_T).wrapping_mul((*infoPtr).uNumItem as SIZE_T),
    ) as *mut INT;

    /* make space for the new item */
    memmove(
        (*infoPtr)
            .items
            .offset((nItem + 1) as isize)
            as *mut core::ffi::c_void,
        (*infoPtr).items.offset(nItem as isize) as *const core::ffi::c_void,
        (((*infoPtr)
            .uNumItem
            .wrapping_sub(nItem as UINT)
            .wrapping_sub(1 as UINT) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<HEADER_ITEM>() as SIZE_T)) as _,
    );
    memmove(
        (*infoPtr)
            .order
            .offset((iOrder + 1) as isize)
            as *mut core::ffi::c_void,
        (*infoPtr).order.offset(iOrder as isize) as *const core::ffi::c_void,
        (((*infoPtr)
            .uNumItem
            .wrapping_sub(iOrder as UINT)
            .wrapping_sub(1 as UINT) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<INT>() as SIZE_T)) as _,
    );

    /* update the order array */
    *(*infoPtr).order.offset(iOrder as isize) = nItem;
    while i < (*infoPtr).uNumItem {
        if i != iOrder as UINT && *(*infoPtr).order.offset(i as isize) >= nItem {
            *(*infoPtr).order.offset(i as isize) += 1;
        }
        (*(*infoPtr)
            .items
            .offset(*(*infoPtr).order.offset(i as isize) as isize))
        .iOrder = i as INT;
        i = i.wrapping_add(1);
    }

    lpItem = (*infoPtr).items.offset(nItem as isize) as *mut HEADER_ITEM;
    ZeroMemory!(
        lpItem as *mut core::ffi::c_void,
        core::mem::size_of::<HEADER_ITEM>() as SIZE_T,
    );
    /* cxy, fmt and lParam are copied even if not in the HDITEM mask */
    copyMask = (*phdi).mask
        | (HDI_WIDTH as UINT)
        | (HDI_FORMAT as UINT)
        | (HDI_LPARAM as UINT);
    HEADER_StoreHDItemInHeader(lpItem, copyMask, phdi, bUnicode);
    (*lpItem).iOrder = iOrder;

    /* set automatically some format bits */
    if ((*phdi).mask & (HDI_TEXT as UINT)) != 0 {
        (*lpItem).fmt |= (HDF_STRING as i32);
    } else {
        (*lpItem).fmt &= (!HDF_STRING) as i32;
    }

    if !(*lpItem).hbm.is_null() {
        (*lpItem).fmt |= (HDF_BITMAP as i32);
    } else {
        (*lpItem).fmt &= (!HDF_BITMAP) as i32;
    }

    if ((*phdi).mask & (HDI_IMAGE as UINT)) != 0 {
        (*lpItem).fmt |= (HDF_IMAGE as i32);
    }

    HEADER_SetItemBounds(infoPtr);
    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        FALSE,
    );

    return nItem as LRESULT;
}

unsafe extern "C" fn HEADER_Layout(
    mut infoPtr: *mut HEADER_INFO,
    mut lpLayout: LPHDLAYOUT,
) -> LRESULT {
    (*(*lpLayout).pwpos).hwnd = (*infoPtr).hwndSelf;
    (*(*lpLayout).pwpos).hwndInsertAfter = core::ptr::null_mut();
    (*(*lpLayout).pwpos).x = (*(*lpLayout).prc).left as ::core::ffi::c_int;
    (*(*lpLayout).pwpos).y = (*(*lpLayout).prc).top as ::core::ffi::c_int;
    (*(*lpLayout).pwpos).cx =
        ((*(*lpLayout).prc).right - (*(*lpLayout).prc).left) as ::core::ffi::c_int;
    if (*infoPtr).dwStyle & HDS_HIDDEN as DWORD != 0 {
        (*(*lpLayout).pwpos).cy = 0 as ::core::ffi::c_int;
    } else {
        (*(*lpLayout).pwpos).cy = (*infoPtr).nHeight as ::core::ffi::c_int;
        (*(*lpLayout).prc).top += (*infoPtr).nHeight as ::core::ffi::c_int;
    }
    (*(*lpLayout).pwpos).flags = SWP_NOZORDER as UINT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_Layout\0" as *const u8 as *const ::core::ffi::c_char,
            1486 as ::core::ffi::c_int,
            b"Layout x=%d y=%d cx=%d cy=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*(*lpLayout).pwpos).x,
            (*(*lpLayout).pwpos).y,
            (*(*lpLayout).pwpos).cx,
            (*(*lpLayout).pwpos).cy,
        );
    }
    (*infoPtr).bRectsValid = FALSE as BOOL;
    return TRUE as LRESULT;
}

unsafe extern "C" fn HEADER_SetImageList(infoPtr: *mut HEADER_INFO, himl: HIMAGELIST) -> LRESULT {
    let mut himlOld: HIMAGELIST;

    himlOld = (*infoPtr).himl;
    (*infoPtr).himl = himl;

    return himlOld as LRESULT;
}

unsafe extern "C" fn HEADER_GetBitmapMargin(infoPtr: *const HEADER_INFO) -> LRESULT {
    ((*infoPtr).iMargin) as _
}

unsafe extern "C" fn HEADER_SetBitmapMargin(
    infoPtr: *mut HEADER_INFO,
    iMargin: INT,
) -> LRESULT {
    let oldMargin: INT = (*infoPtr).iMargin;

    (*infoPtr).iMargin = iMargin;

    return oldMargin as LRESULT;
}

unsafe extern "C" fn HEADER_SetItemT(
    infoPtr: *mut HEADER_INFO,
    nItem: INT,
    phdi: *const HDITEMW,
    bUnicode: BOOL,
) -> LRESULT {
    let mut lpItem: *mut HEADER_ITEM = core::ptr::null_mut();
    let mut hdNotify: HDITEMW = core::mem::zeroed();
    let mut pvScratch: *mut core::ffi::c_void = core::ptr::null_mut();

    if phdi.is_null() {
        return FALSE as LRESULT;
    }
    if nItem < 0 as INT || nItem >= (*infoPtr).uNumItem as INT {
        return FALSE as LRESULT;
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
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_SetItemT\0" as *const u8 as *const ::core::ffi::c_char,
            1537 as ::core::ffi::c_int,
            b"[nItem=%d]\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItem,
        );
    }

    HEADER_CopyHDItemForNotify(
        infoPtr,
        core::ptr::addr_of_mut!(hdNotify),
        phdi,
        bUnicode,
        core::ptr::addr_of_mut!(pvScratch),
    );
    if HEADER_SendNotifyWithHDItemT(
        infoPtr,
        HDN_ITEMCHANGINGW as UINT,
        nItem,
        core::ptr::addr_of_mut!(hdNotify),
    ) != 0
    {
        heap_free(pvScratch);
        return FALSE as LRESULT;
    }

    lpItem = (*infoPtr).items.offset(nItem as isize) as *mut HEADER_ITEM;
    HEADER_StoreHDItemInHeader(lpItem, (*phdi).mask, phdi, bUnicode);

    if ((*phdi).mask & (HDI_ORDER as UINT)) != 0 {
        if (*phdi).iOrder >= 0 as INT
            && ((*phdi).iOrder as UINT) < (*infoPtr).uNumItem
        {
            HEADER_ChangeItemOrder(infoPtr, nItem, (*phdi).iOrder as INT);
        }
    }

    HEADER_SendNotifyWithHDItemT(
        infoPtr,
        HDN_ITEMCHANGEDW as UINT,
        nItem,
        core::ptr::addr_of_mut!(hdNotify),
    );

    HEADER_SetItemBounds(infoPtr);

    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        FALSE as BOOL,
    );

    heap_free(pvScratch);
    return TRUE as LRESULT;
}

unsafe extern "C" fn HEADER_SetUnicodeFormat(
    mut infoPtr: *mut HEADER_INFO,
    mut wParam: WPARAM,
) -> LRESULT {
    let mut bTemp: BOOL = (((*infoPtr).nNotifyFormat) as u32 == NFR_UNICODE) as BOOL;
    (*infoPtr).nNotifyFormat = (if wParam as BOOL != 0 {
        NFR_UNICODE
    } else {
        NFR_ANSI
    }) as INT;
    return bTemp as LRESULT;
}

unsafe extern "C" fn HEADER_Create(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut HEADER_INFO = core::ptr::null_mut();
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();

    infoPtr = heap_alloc_zero(core::mem::size_of::<HEADER_INFO>() as SIZE_T) as *mut HEADER_INFO;
    SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    (*infoPtr).uNumItem = 0 as UINT;
    (*infoPtr).hFont = core::ptr::null_mut();
    (*infoPtr).items = core::ptr::null_mut();
    (*infoPtr).order = core::ptr::null_mut();
    (*infoPtr).bRectsValid = FALSE as BOOL;
    (*infoPtr).hcurArrow = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    (*infoPtr).hcurDivider = LoadCursorW(
        COMCTL32_hModule as HINSTANCE,
        MAKEINTRESOURCEW!(IDC_DIVIDER) as LPCWSTR,
    );
    (*infoPtr).hcurDivopen = LoadCursorW(
        COMCTL32_hModule as HINSTANCE,
        MAKEINTRESOURCEW!(IDC_DIVIDEROPEN) as LPCWSTR,
    );
    (*infoPtr).bPressed = FALSE as BOOL;
    (*infoPtr).bTracking = FALSE as BOOL;
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;
    (*infoPtr).iMoveItem = 0 as INT;
    (*infoPtr).himl = core::ptr::null_mut();
    (*infoPtr).iHotItem = -(1 as INT);
    (*infoPtr).iHotDivider = -(1 as INT);
    (*infoPtr).iMargin = (3 * GetSystemMetrics((SM_CXEDGE) as i32)) as INT;
    (*infoPtr).nNotifyFormat = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFYFORMAT as UINT,
        hwnd as WPARAM,
        NF_QUERY as LPARAM,
    ) as INT;
    (*infoPtr).filter_change_timeout = 1000 as INT;

    hdc = GetDC(core::ptr::null_mut());
    hOldFont = SelectObject(
        hdc,
        GetStockObject((SYSTEM_FONT as i32)),
    ) as HFONT;
    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));
    (*infoPtr).nHeight = (tm.tmHeight + VERT_BORDER) as INT;
    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC(core::ptr::null_mut(), hdc);

    OpenThemeData(hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_Destroy(infoPtr: *mut HEADER_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    return 0;
}

unsafe extern "C" fn HEADER_NCDestroy(mut infoPtr: *mut HEADER_INFO) -> LRESULT {
    let mut lpItem: *mut HEADER_ITEM = core::ptr::null_mut::<HEADER_ITEM>();
    let mut nItem: INT = 0;
    if !(*infoPtr).items.is_null() {
        lpItem = (*infoPtr).items;
        nItem = 0;
        while (nItem as UINT) < (*infoPtr).uNumItem {
            heap_free((*lpItem).pszText as *mut core::ffi::c_void);
            nItem += 1;
            lpItem = lpItem.offset(1);
        }
        heap_free((*infoPtr).items as *mut core::ffi::c_void);
    }
    heap_free((*infoPtr).order as *mut core::ffi::c_void);
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    heap_free(infoPtr as *mut core::ffi::c_void);
    return 0;
}

unsafe extern "C" fn HEADER_GetFont(infoPtr: *const HEADER_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn HEADER_IsDragDistance(infoPtr: *const HEADER_INFO, pt: *const POINT) -> BOOL {
    return ((abs((*infoPtr).ptLButtonDown.x - (*pt).x) > GetSystemMetrics(SM_CXDOUBLECLK as i32))
        || (abs((*infoPtr).ptLButtonDown.y - (*pt).y) > GetSystemMetrics(SM_CYDOUBLECLK as i32))) as BOOL;
}

unsafe extern "C" fn HEADER_LButtonDblClk(infoPtr: *const HEADER_INFO, x: INT, y: INT) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut flags: UINT = 0;
    let mut nItem: INT = 0;

    pt.x = x as LONG;
    pt.y = y as LONG;
    HEADER_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!(pt),
        core::ptr::addr_of_mut!(flags),
        core::ptr::addr_of_mut!(nItem),
    );

    if ((*infoPtr).dwStyle & (HDS_BUTTONS as DWORD)) != 0
        && flags == (HHT_ONHEADER as UINT)
    {
        HEADER_SendNotifyWithHDItemT(
            infoPtr,
            HDN_ITEMDBLCLICKW as UINT,
            nItem,
            core::ptr::null_mut(),
        );
    } else if flags == (HHT_ONDIVIDER as UINT) || flags == (HHT_ONDIVOPEN as UINT) {
        HEADER_SendNotifyWithHDItemT(
            infoPtr,
            HDN_DIVIDERDBLCLICKW as UINT,
            nItem,
            core::ptr::null_mut(),
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_LButtonDown(
    infoPtr: *mut HEADER_INFO,
    x: INT,
    y: INT,
) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut flags: UINT = 0;
    let mut nItem: INT = 0;
    let mut hdc: HDC = core::ptr::null_mut();

    pt.x = x;
    pt.y = y;
    HEADER_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!(pt),
        core::ptr::addr_of_mut!(flags),
        core::ptr::addr_of_mut!(nItem),
    );

    if ((*infoPtr).dwStyle & HDS_BUTTONS as DWORD != 0)
        && (flags == HHT_ONHEADER as UINT)
    {
        SetCapture((*infoPtr).hwndSelf);
        (*infoPtr).bCaptured = TRUE as BOOL;
        (*infoPtr).bPressed = TRUE as BOOL;
        (*infoPtr).bDragging = FALSE as BOOL;
        (*infoPtr).iMoveItem = nItem;
        (*infoPtr).ptLButtonDown = pt;

        (*(*infoPtr).items.offset(nItem as isize)).bDown = TRUE as BOOL;

        /* Send WM_CUSTOMDRAW */
        hdc = GetDC((*infoPtr).hwndSelf);
        HEADER_RefreshItem(infoPtr, nItem);
        ReleaseDC((*infoPtr).hwndSelf, hdc);

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/header.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"HEADER_LButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
                1714 as ::core::ffi::c_int,
                b"Pressed item %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
                nItem,
            );
        }
    } else if (flags == HHT_ONDIVIDER as UINT) || (flags == HHT_ONDIVOPEN as UINT) {
        let iCurrWidth: INT = (*(*infoPtr).items.offset(nItem as isize)).cxy;
        if HEADER_SendNotifyWithIntFieldT(
            infoPtr,
            HDN_BEGINTRACKW as UINT,
            nItem,
            HDI_WIDTH as INT,
            iCurrWidth,
        ) == 0
        {
            SetCapture((*infoPtr).hwndSelf);
            (*infoPtr).bCaptured = TRUE as BOOL;
            (*infoPtr).bTracking = TRUE as BOOL;
            (*infoPtr).iMoveItem = nItem;
            (*infoPtr).xTrackOffset =
                ((*(*infoPtr).items.offset(nItem as isize)).rect.right - pt.x) as INT;

            if (*infoPtr).dwStyle & HDS_FULLDRAG as DWORD == 0 {
                (*infoPtr).xOldTrack = (*(*infoPtr).items.offset(nItem as isize)).rect.right as INT;
                hdc = GetDC((*infoPtr).hwndSelf);
                HEADER_DrawTrackLine(infoPtr, hdc, (*infoPtr).xOldTrack);
                ReleaseDC((*infoPtr).hwndSelf, hdc);
            }

            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/header.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"HEADER_LButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
                    1733 as ::core::ffi::c_int,
                    b"Begin tracking item %d.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    nItem,
                );
            }
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_LButtonUp(infoPtr: *mut HEADER_INFO, x: INT, y: INT) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut flags: UINT = 0;
    let mut nItem: INT = 0;
    let mut hdc: HDC = core::ptr::null_mut();

    pt.x = x as LONG;
    pt.y = y as LONG;
    HEADER_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!(pt),
        core::ptr::addr_of_mut!(flags),
        core::ptr::addr_of_mut!(nItem),
    );

    if (*infoPtr).bPressed != 0 {
        (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).bDown = FALSE as BOOL;

        if (*infoPtr).bDragging != 0 {
            let lpItem: *mut HEADER_ITEM =
                (*infoPtr).items.offset((*infoPtr).iMoveItem as isize) as *mut HEADER_ITEM;
            let mut iNewOrder: INT = 0;

            ImageList_DragShowNolock(FALSE as BOOL);
            ImageList_EndDrag();

            if (*infoPtr).iHotDivider == -1 as INT {
                iNewOrder = -1 as INT;
            } else if (*infoPtr).iHotDivider as UINT == (*infoPtr).uNumItem {
                iNewOrder = ((*infoPtr).uNumItem - 1 as UINT) as INT;
            } else {
                iNewOrder = HEADER_IndexToOrder(infoPtr, (*infoPtr).iHotDivider) as INT;
                if iNewOrder > (*lpItem).iOrder {
                    iNewOrder -= 1;
                }
            }

            if iNewOrder != -1 as INT
                && HEADER_SendNotifyWithIntFieldT(
                    infoPtr,
                    HDN_ENDDRAG as UINT,
                    (*infoPtr).iMoveItem,
                    HDI_ORDER as INT,
                    iNewOrder,
                ) == 0
            {
                HEADER_ChangeItemOrder(infoPtr, (*infoPtr).iMoveItem, iNewOrder);
                (*infoPtr).bRectsValid = FALSE as BOOL;
                InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), FALSE as BOOL);
            } else {
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).rect) as LPCRECT,
                    FALSE as BOOL,
                );
            }

            (*infoPtr).bDragging = FALSE as BOOL;
            HEADER_SetHotDivider(
                infoPtr,
                FALSE as WPARAM,
                -1 as LPARAM,
            );
        } else {
            hdc = GetDC((*infoPtr).hwndSelf);
            HEADER_RefreshItem(infoPtr, (*infoPtr).iMoveItem);
            ReleaseDC((*infoPtr).hwndSelf, hdc);

            if ((*infoPtr).dwStyle & HDS_DRAGDROP as DWORD) == 0
                || HEADER_IsDragDistance(infoPtr, core::ptr::addr_of_mut!(pt)) == 0
            {
                HEADER_SendNotifyWithHDItemT(
                    infoPtr,
                    HDN_ITEMCLICKW as UINT,
                    (*infoPtr).iMoveItem,
                    core::ptr::null_mut::<HDITEMW>(),
                );
            }
        }

        (*infoPtr).bPressed = FALSE as BOOL;
    } else if (*infoPtr).bTracking != 0 {
        let mut iNewWidth: INT = pt.x as INT
            - (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).rect.left as INT
            + (*infoPtr).xTrackOffset;
        if iNewWidth < 0 as INT {
            iNewWidth = 0 as INT;
        }

        (*infoPtr).bTracking = FALSE as BOOL;

        HEADER_SendNotifyWithIntFieldT(
            infoPtr,
            HDN_ENDTRACKW as UINT,
            (*infoPtr).iMoveItem,
            HDI_WIDTH as INT,
            iNewWidth,
        );

        if ((*infoPtr).dwStyle & HDS_FULLDRAG as DWORD) == 0 {
            hdc = GetDC((*infoPtr).hwndSelf);
            HEADER_DrawTrackLine(infoPtr, hdc, (*infoPtr).xOldTrack);
            ReleaseDC((*infoPtr).hwndSelf, hdc);
        }

        if HEADER_SendNotifyWithIntFieldT(
            infoPtr,
            HDN_ITEMCHANGINGW as UINT,
            (*infoPtr).iMoveItem,
            HDI_WIDTH as INT,
            iNewWidth,
        ) == 0
        {
            (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).cxy = iNewWidth;
            HEADER_SendNotifyWithIntFieldT(
                infoPtr,
                HDN_ITEMCHANGEDW as UINT,
                (*infoPtr).iMoveItem,
                HDI_WIDTH as INT,
                iNewWidth,
            );
        }

        HEADER_SetItemBounds(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE as BOOL);
    }

    if (*infoPtr).bCaptured != 0 {
        (*infoPtr).bCaptured = FALSE as BOOL;
        ReleaseCapture();
        HEADER_SendSimpleNotify(infoPtr, NM_RELEASEDCAPTURE as UINT);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_NotifyFormat(
    mut infoPtr: *mut HEADER_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    match (lParam) as u32 {
        NF_QUERY => {
            return (*infoPtr).nNotifyFormat as LRESULT;
        }
        NF_REQUERY => {
            (*infoPtr).nNotifyFormat = SendMessageW(
                wParam as HWND,
                WM_NOTIFYFORMAT as UINT,
                (*infoPtr).hwndSelf as WPARAM,
                NF_QUERY as LPARAM,
            ) as INT;
            return (*infoPtr).nNotifyFormat as LRESULT;
        }
        _ => {}
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_MouseLeave(infoPtr: *mut HEADER_INFO) -> LRESULT {
    let oldHotItem: INT = (*infoPtr).iHotItem;
    let hdc: HDC = GetDC((*infoPtr).hwndSelf);

    (*infoPtr).iHotItem = -1 as INT;
    if oldHotItem != -1 as INT {
        HEADER_RefreshItem(infoPtr, oldHotItem);
    }
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_MouseMove(infoPtr: *mut HEADER_INFO, lParam: LPARAM) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut flags: UINT = 0;
    let mut nItem: INT = 0;
    let mut nWidth: INT = 0;
    let mut hdc: HDC = core::ptr::null_mut();
    /* With theming, hottracking is always enabled */
    let mut hotTrackEnabled: BOOL = ((((*infoPtr).dwStyle & (HDS_BUTTONS as DWORD)) != 0
        && ((*infoPtr).dwStyle & (HDS_HOTTRACK as DWORD)) != 0)
        || !(GetWindowTheme((*infoPtr).hwndSelf)).is_null()) as BOOL;
    let mut oldHotItem: INT = (*infoPtr).iHotItem;

    pt.x = LOWORD!(lParam as DWORD) as SHORT as INT as LONG;
    pt.y = HIWORD!(lParam as DWORD) as SHORT as INT as LONG;
    HEADER_InternalHitTest(infoPtr, core::ptr::addr_of_mut!(pt), core::ptr::addr_of_mut!(flags), core::ptr::addr_of_mut!(nItem));

    if hotTrackEnabled != 0 {
        if flags & ((HHT_ONHEADER | HHT_ONDIVIDER | HHT_ONDIVOPEN) as UINT) != 0 {
            (*infoPtr).iHotItem = nItem;
        } else {
            (*infoPtr).iHotItem = -(1 as INT);
        }
    }

    if (*infoPtr).bCaptured != 0 {
        /* check if we should drag the header */
        if (*infoPtr).bPressed != 0
            && (*infoPtr).bDragging == 0
            && ((*infoPtr).dwStyle & (HDS_DRAGDROP as DWORD)) != 0
            && HEADER_IsDragDistance(infoPtr, core::ptr::addr_of_mut!(pt)) != 0
        {
            if HEADER_SendNotifyWithHDItemT(
                infoPtr,
                HDN_BEGINDRAG as UINT,
                (*infoPtr).iMoveItem,
                core::ptr::null_mut(),
            ) == 0
            {
                let mut hDragItem: HIMAGELIST = HEADER_CreateDragImage(infoPtr, (*infoPtr).iMoveItem);
                if !hDragItem.is_null() {
                    let mut lpItem: *mut HEADER_ITEM =
                        (*infoPtr).items.offset((*infoPtr).iMoveItem as isize) as *mut HEADER_ITEM;
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl,
                            __dbch,
                            b"dll/win32/comctl32/header.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"HEADER_MouseMove\0" as *const u8 as *const ::core::ffi::c_char,
                            1905 as ::core::ffi::c_int,
                            b"Starting item drag\n\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    ImageList_BeginDrag(
                        hDragItem,
                        0 as INT,
                        pt.x - (*lpItem).rect.left,
                        0 as INT,
                    );
                    ImageList_DragShowNolock(TRUE as BOOL);
                    ImageList_Destroy(hDragItem);
                    (*infoPtr).bDragging = TRUE as BOOL;
                }
            }
        }

        if (*infoPtr).bDragging != 0 {
            let mut drag: POINT = POINT { x: 0, y: 0 };
            drag.x = pt.x;
            drag.y = 0 as LONG;
            ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(drag));
            ImageList_DragMove(drag.x as INT, drag.y as INT);
            HEADER_SetHotDivider(infoPtr, TRUE as WPARAM, lParam);
        }

        if (*infoPtr).bPressed != 0 && (*infoPtr).bDragging == 0 {
            let mut oldState: BOOL = (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).bDown;
            if nItem == (*infoPtr).iMoveItem && flags == (HHT_ONHEADER as UINT) {
                (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).bDown = TRUE as BOOL;
            } else {
                (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).bDown = FALSE as BOOL;
            }
            if oldState != (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).bDown {
                hdc = GetDC((*infoPtr).hwndSelf);
                HEADER_RefreshItem(infoPtr, (*infoPtr).iMoveItem);
                ReleaseDC((*infoPtr).hwndSelf, hdc);
            }
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/header.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"HEADER_MouseMove\0" as *const u8 as *const ::core::ffi::c_char,
                    1936 as ::core::ffi::c_int,
                    b"Moving pressed item %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*infoPtr).iMoveItem,
                );
            }
        } else if (*infoPtr).bTracking != 0 {
            if (*infoPtr).dwStyle & (HDS_FULLDRAG as DWORD) != 0 {
                let mut lpItem_0: *mut HEADER_ITEM =
                    (*infoPtr).items.offset((*infoPtr).iMoveItem as isize) as *mut HEADER_ITEM;
                nWidth = (pt.x - (*lpItem_0).rect.left) as INT + (*infoPtr).xTrackOffset;
                if HEADER_SendNotifyWithIntFieldT(
                    infoPtr,
                    HDN_ITEMCHANGINGW as UINT,
                    (*infoPtr).iMoveItem,
                    HDI_WIDTH as INT,
                    nWidth,
                ) == 0
                {
                    let mut nOldWidth: INT = (*lpItem_0).rect.right - (*lpItem_0).rect.left;
                    let mut rcClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    let mut rcScroll: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

                    if nWidth < (0 as INT) {
                        nWidth = 0 as INT;
                    }
                    (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).cxy = nWidth;
                    if true {
                        InvalidateRect(
                            (*infoPtr).hwndSelf,
                            core::ptr::addr_of_mut!((*lpItem_0).rect),
                            FALSE as BOOL,
                        );
                    } else {
                        InvalidateRect(
                            (*infoPtr).hwndSelf,
                            core::ptr::addr_of_mut!((*lpItem_0).rect),
                            FALSE as BOOL,
                        );
                    }
                    HEADER_SetItemBounds(infoPtr);

                    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcClient));
                    rcScroll = rcClient;
                    rcScroll.left = (*lpItem_0).rect.left + nOldWidth;
                    if true {
                        ScrollWindowEx(
                            (*infoPtr).hwndSelf,
                            nWidth - nOldWidth,
                            0 as INT,
                            core::ptr::addr_of_mut!(rcScroll),
                            core::ptr::addr_of_mut!(rcClient),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            SW_INVALIDATE as UINT,
                        );
                    } else {
                        ScrollWindowEx(
                            (*infoPtr).hwndSelf,
                            nWidth - nOldWidth,
                            0 as INT,
                            core::ptr::addr_of_mut!(rcScroll),
                            core::ptr::addr_of_mut!(rcClient),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            0 as UINT,
                        );
                    }
                    UpdateWindow((*infoPtr).hwndSelf);

                    HEADER_SendNotifyWithIntFieldT(
                        infoPtr,
                        HDN_ITEMCHANGEDW as UINT,
                        (*infoPtr).iMoveItem,
                        HDI_WIDTH as INT,
                        nWidth,
                    );
                }
            } else {
                let mut iTrackWidth: INT = 0;
                hdc = GetDC((*infoPtr).hwndSelf);
                HEADER_DrawTrackLine(infoPtr, hdc, (*infoPtr).xOldTrack);
                (*infoPtr).xOldTrack = pt.x as INT + (*infoPtr).xTrackOffset;
                if (*infoPtr).xOldTrack
                    < (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).rect.left
                {
                    (*infoPtr).xOldTrack =
                        (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).rect.left;
                }
                HEADER_DrawTrackLine(infoPtr, hdc, (*infoPtr).xOldTrack);
                ReleaseDC((*infoPtr).hwndSelf, hdc);
                iTrackWidth = (*infoPtr).xOldTrack - (*(*infoPtr).items.offset((*infoPtr).iMoveItem as isize)).rect.left;
                /* FIXME: should stop tracking if HDN_TRACK returns TRUE */
                HEADER_SendNotifyWithIntFieldT(
                    infoPtr,
                    HDN_TRACKW as UINT,
                    (*infoPtr).iMoveItem,
                    HDI_WIDTH as INT,
                    iTrackWidth,
                );
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
                    b"dll/win32/comctl32/header.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"HEADER_MouseMove\0" as *const u8 as *const ::core::ffi::c_char,
                    1984 as ::core::ffi::c_int,
                    b"Tracking item %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*infoPtr).iMoveItem,
                );
            }
        }
    }

    if hotTrackEnabled != 0 {
        let mut tme: TRACKMOUSEEVENT = TRACKMOUSEEVENT {
            cbSize: 0,
            dwFlags: 0,
            hwndTrack: core::ptr::null_mut(),
            dwHoverTime: 0,
        };
        if oldHotItem != (*infoPtr).iHotItem && (*infoPtr).bDragging == 0 {
            hdc = GetDC((*infoPtr).hwndSelf);
            if oldHotItem != -(1 as INT) {
                HEADER_RefreshItem(infoPtr, oldHotItem);
            }
            if (*infoPtr).iHotItem != -(1 as INT) {
                HEADER_RefreshItem(infoPtr, (*infoPtr).iHotItem);
            }
            ReleaseDC((*infoPtr).hwndSelf, hdc);
        }
        tme.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
        tme.dwFlags = TME_LEAVE as DWORD;
        tme.hwndTrack = (*infoPtr).hwndSelf;
        TrackMouseEvent(core::ptr::addr_of_mut!(tme));
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_Paint(infoPtr: *mut HEADER_INFO, hdcParam: HDC) -> LRESULT {
    let mut hdc: HDC;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();

    hdc = if hdcParam == core::ptr::null_mut() {
        BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps))
    } else {
        hdcParam
    };
    HEADER_Refresh(infoPtr, hdc);
    if hdcParam.is_null() {
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }
    return 0;
}

unsafe extern "C" fn HEADER_RButtonUp(mut infoPtr: *mut HEADER_INFO, mut x: INT, mut y: INT) -> LRESULT {
    let mut bRet: BOOL = 0;
    let mut pt: POINT = core::mem::zeroed();

    pt.x = x as LONG;
    pt.y = y as LONG;

    /* Send a Notify message */
    bRet = HEADER_SendSimpleNotify(infoPtr, NM_RCLICK);

    /* Change to screen coordinate for WM_CONTEXTMENU */
    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));

    /* Send a WM_CONTEXTMENU message in response to the RBUTTONUP */
    SendMessageW(
        (*infoPtr).hwndSelf,
        WM_CONTEXTMENU,
        (*infoPtr).hwndSelf as WPARAM,
        MAKELPARAM!(pt.x as WORD, pt.y as WORD),
    );

    return bRet as LRESULT;
}

unsafe extern "C" fn HEADER_SetCursor(
    infoPtr: *mut HEADER_INFO,
    lParam: LPARAM,
) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut flags: UINT = 0;
    let mut nItem: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_SetCursor\0" as *const u8 as *const ::core::ffi::c_char,
            2049 as ::core::ffi::c_int,
            b"code=0x%X  id=0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
            (lParam as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ::core::ffi::c_int,
            (lParam as DWORD_PTR >> 16 as ::core::ffi::c_int & 0xffff as DWORD_PTR)
                as WORD as ::core::ffi::c_int,
        );
    }

    GetCursorPos(&raw mut pt);
    ScreenToClient((*infoPtr).hwndSelf, &raw mut pt);

    HEADER_InternalHitTest(infoPtr, &raw mut pt, &raw mut flags, &raw mut nItem);

    if flags == HHT_ONDIVIDER as UINT {
        SetCursor((*infoPtr).hcurDivider);
    } else if flags == HHT_ONDIVOPEN as UINT {
        SetCursor((*infoPtr).hcurDivopen);
    } else {
        SetCursor((*infoPtr).hcurArrow);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_SetFont(infoPtr: *mut HEADER_INFO, hFont: HFONT, Redraw: WORD) -> LRESULT {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();

    (*infoPtr).hFont = hFont;

    hdc = GetDC(core::ptr::null_mut());
    hOldFont = SelectObject(
        hdc,
        if !(*infoPtr).hFont.is_null() {
            (*infoPtr).hFont as HGDIOBJ
        } else {
            GetStockObject((SYSTEM_FONT as i32))
        },
    ) as HFONT;
    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));
    (*infoPtr).nHeight = (tm.tmHeight as i32 + VERT_BORDER) as INT;
    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC(core::ptr::null_mut(), hdc);

    (*infoPtr).bRectsValid = FALSE as BOOL;

    if Redraw != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), FALSE);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HEADER_SetRedraw(
    mut infoPtr: *mut HEADER_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut ret: LRESULT = 0;
    ret = DefWindowProcW((*infoPtr).hwndSelf, WM_SETREDRAW, wParam, lParam);
    if wParam != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    }
    return ret;
}

unsafe extern "C" fn HEADER_StyleChanged(
    infoPtr: *mut HEADER_INFO,
    wStyleType: WPARAM,
    lpss: *const STYLESTRUCT,
) -> INT {
    if wStyleType != GWL_STYLE as WPARAM {
        return 0 as INT;
    }

    (*infoPtr).dwStyle = (*lpss).styleNew;

    return 0 as INT;
}

unsafe extern "C" fn HEADER_ThemeChanged(infoPtr: *const HEADER_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwndSelf, core::ptr::addr_of!(themeClass) as LPCWSTR);
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), FALSE);
    return 0;
}

unsafe extern "C" fn HEADER_SetFilterChangeTimeout(infoPtr: *mut HEADER_INFO, timeout: INT) -> INT {
    let mut old_timeout: INT = (*infoPtr).filter_change_timeout;

    if timeout != 0 {
        (*infoPtr).filter_change_timeout = timeout;
    }
    return old_timeout;
}

unsafe extern "system" fn HEADER_WindowProc(
    hwnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut HEADER_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut HEADER_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/header.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HEADER_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            2141 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lParam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            msg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && (msg != WM_CREATE) {
        return DefWindowProcW(hwnd, msg, wParam, lParam);
    }
    match msg {
        HDM_CREATEDRAGIMAGE => {
            return HEADER_CreateDragImage(infoPtr, wParam as INT) as LRESULT;
        }
        HDM_DELETEITEM => {
            return HEADER_DeleteItem(infoPtr, wParam as INT);
        }
        HDM_GETBITMAPMARGIN => {
            return HEADER_GetBitmapMargin(infoPtr);
        }
        HDM_GETIMAGELIST => {
            return HEADER_GetImageList(infoPtr);
        }
        HDM_GETITEMA | HDM_GETITEMW => {
            return HEADER_GetItemT(
                infoPtr,
                wParam as INT,
                lParam as LPHDITEMW,
                (msg == HDM_GETITEMW) as ::core::ffi::c_int,
            );
        }
        HDM_GETITEMCOUNT => {
            return HEADER_GetItemCount(infoPtr);
        }
        HDM_GETITEMRECT => {
            return HEADER_GetItemRect(infoPtr, wParam as INT, lParam as LPRECT);
        }
        HDM_GETORDERARRAY => {
            return HEADER_GetOrderArray(infoPtr, wParam as INT, lParam as LPINT);
        }
        HDM_GETUNICODEFORMAT => {
            return HEADER_GetUnicodeFormat(infoPtr);
        }
        HDM_HITTEST => {
            return HEADER_HitTest(infoPtr, lParam as LPHDHITTESTINFO);
        }
        HDM_INSERTITEMA | HDM_INSERTITEMW => {
            return HEADER_InsertItemT(
                infoPtr,
                wParam as INT,
                lParam as LPHDITEMW as *const HDITEMW,
                (msg == HDM_INSERTITEMW) as ::core::ffi::c_int,
            );
        }
        HDM_LAYOUT => {
            return HEADER_Layout(infoPtr, lParam as LPHDLAYOUT);
        }
        HDM_ORDERTOINDEX => {
            return HEADER_OrderToIndex(infoPtr, wParam as INT) as LRESULT;
        }
        HDM_SETBITMAPMARGIN => {
            return HEADER_SetBitmapMargin(infoPtr, wParam as INT);
        }
        HDM_SETFILTERCHANGETIMEOUT => {
            return HEADER_SetFilterChangeTimeout(infoPtr, lParam as INT) as LRESULT;
        }
        HDM_SETHOTDIVIDER => {
            return HEADER_SetHotDivider(infoPtr, wParam, lParam);
        }
        HDM_SETIMAGELIST => {
            return HEADER_SetImageList(infoPtr, lParam as HIMAGELIST);
        }
        HDM_SETITEMA | HDM_SETITEMW => {
            return HEADER_SetItemT(
                infoPtr,
                wParam as INT,
                lParam as LPHDITEMW as *const HDITEMW,
                (msg == HDM_SETITEMW) as ::core::ffi::c_int,
            );
        }
        HDM_SETORDERARRAY => {
            return HEADER_SetOrderArray(infoPtr, wParam as INT, lParam as LPINT as *const INT);
        }
        HDM_SETUNICODEFORMAT => {
            return HEADER_SetUnicodeFormat(infoPtr, wParam);
        }
        WM_CREATE => {
            return HEADER_Create(hwnd, lParam as LPCREATESTRUCTW as *const CREATESTRUCTW);
        }
        WM_DESTROY => {
            return HEADER_Destroy(infoPtr);
        }
        WM_NCDESTROY => {
            return HEADER_NCDestroy(infoPtr);
        }
        WM_ERASEBKGND => {
            return 1 as LRESULT;
        }
        WM_GETDLGCODE => {
            return (DLGC_WANTTAB | DLGC_WANTARROWS) as LRESULT;
        }
        WM_GETFONT => {
            return HEADER_GetFont(infoPtr);
        }
        WM_LBUTTONDBLCLK => {
            return HEADER_LButtonDblClk(
                infoPtr,
                LOWORD!(lParam) as SHORT as INT,
                HIWORD!(lParam) as SHORT as INT,
            );
        }
        WM_LBUTTONDOWN => {
            return HEADER_LButtonDown(
                infoPtr,
                LOWORD!(lParam) as SHORT as INT,
                HIWORD!(lParam) as SHORT as INT,
            );
        }
        WM_LBUTTONUP => {
            return HEADER_LButtonUp(
                infoPtr,
                LOWORD!(lParam) as SHORT as INT,
                HIWORD!(lParam) as SHORT as INT,
            );
        }
        WM_MOUSELEAVE => {
            return HEADER_MouseLeave(infoPtr);
        }
        WM_MOUSEMOVE => {
            return HEADER_MouseMove(infoPtr, lParam);
        }
        WM_NOTIFYFORMAT => {
            return HEADER_NotifyFormat(infoPtr, wParam, lParam);
        }
        WM_SIZE => {
            return HEADER_Size(infoPtr);
        }
        WM_THEMECHANGED => {
            return HEADER_ThemeChanged(infoPtr);
        }
        WM_PRINTCLIENT | WM_PAINT => {
            return HEADER_Paint(infoPtr, wParam as HDC);
        }
        WM_RBUTTONUP => {
            return HEADER_RButtonUp(
                infoPtr,
                LOWORD!(lParam) as SHORT as INT,
                HIWORD!(lParam) as SHORT as INT,
            );
        }
        WM_SETCURSOR => {
            return HEADER_SetCursor(infoPtr, lParam);
        }
        WM_SETFONT => {
            return HEADER_SetFont(infoPtr, wParam as HFONT, lParam as WORD);
        }
        WM_SETREDRAW => {
            return HEADER_SetRedraw(infoPtr, wParam, lParam);
        }
        WM_STYLECHANGED => {
            return HEADER_StyleChanged(
                infoPtr,
                wParam,
                lParam as LPSTYLESTRUCT as *const STYLESTRUCT,
            ) as LRESULT;
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            return 0 as LRESULT;
        }
        _ => {
            if (msg >= WM_USER)
                && (msg < WM_APP)
                && (COMCTL32_IsReflectedMessage(msg) == 0)
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
                        b"dll/win32/comctl32/header.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"HEADER_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        2280 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%04lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        msg,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, msg, wParam, lParam);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn HEADER_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc =
        Some(HEADER_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut HEADER_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.lpszClassName = (WC_HEADERW).as_ptr();

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn HEADER_Unregister() {
    UnregisterClassW((WC_HEADERW).as_ptr(), core::ptr::null_mut());
}
