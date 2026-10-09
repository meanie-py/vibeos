use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::DefSubclassProc;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::commctrl::GetWindowSubclass;
use crate::comctl32::commctrl::RemoveWindowSubclass;
use crate::comctl32::commctrl::SetWindowSubclass;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_commctrl);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const DRAGLISTMSGSTRINGW: [WCHAR; 21] = [
    'c' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    '_' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'M' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'g' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[no_mangle]
pub static mut uDragListMessage: UINT = 0 as UINT;

static mut dwLastScrollTime: DWORD = 0 as DWORD;

static mut hDragArrow: HICON = ::core::ptr::null::<HICON__>() as *mut HICON__;

// WINE_DEFAULT_DEBUG_CHANNEL(commctrl): debug-channel declaration, no runtime equivalent emitted.

pub const DRAGLIST_SUBCLASSID: usize = 0;

pub const DRAGLIST_SCROLLPERIOD: DWORD = 200;

pub const DRAGLIST_TIMERID: usize = 666;

pub const DRAGICON_HOTSPOT_X: i32 = 17;

pub const DRAGICON_HOTSPOT_Y: i32 = 7;

pub const DRAGICON_HEIGHT: i32 = 32;

#[repr(C)]
pub struct _DRAGLISTDATA {
    /* are we currently in dragging mode? */
    pub dragging: BOOL,

    /* cursor to use as determined by DL_DRAGGING notification.
     * NOTE: as we use LoadCursor we don't have to use DeleteCursor
     * when we are finished with it */
    pub cursor: HCURSOR,

    /* optimisation so that we don't have to load the cursor
     * all of the time whilst dragging */
    pub last_dragging_response: LRESULT,

    /* prevents flicker with drawing drag arrow */
    pub last_drag_icon_rect: RECT,
}

pub type DRAGLISTDATA = _DRAGLISTDATA;

unsafe extern "C" fn DragList_Notify(hwndLB: HWND, uNotification: UINT) -> LRESULT {
    let mut dli: DRAGLISTINFO = DRAGLISTINFO {
        uNotification: 0,
        hWnd: core::ptr::null_mut(),
        ptCursor: POINT { x: 0, y: 0 },
    };
    dli.hWnd = hwndLB;
    dli.uNotification = uNotification;
    GetCursorPos(core::ptr::addr_of_mut!(dli.ptCursor));
    return SendMessageW(
        GetParent(hwndLB),
        uDragListMessage,
        GetDlgCtrlID(hwndLB) as WPARAM,
        core::ptr::addr_of_mut!(dli) as LPARAM,
    );
}

unsafe extern "C" fn DragList_EndDrag(hwnd: HWND, data: *mut DRAGLISTDATA) {
    KillTimer(hwnd, (DRAGLIST_TIMERID) as UINT_PTR);
    ReleaseCapture();
    InvalidateRect(GetParent(hwnd), core::ptr::addr_of_mut!((*data).last_drag_icon_rect) as LPCRECT, TRUE);
    memset(data as *mut ::core::ffi::c_void, 0, core::mem::size_of::<DRAGLISTDATA>() as size_t);
}
unsafe extern "system" fn DragList_SubclassWindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uIdSubclass: UINT_PTR,
    dwRefData: DWORD_PTR,
) -> LRESULT {
    let data: *mut DRAGLISTDATA = dwRefData as *mut DRAGLISTDATA;
    match uMsg {
        WM_LBUTTONDOWN => {
            SetFocus(hwnd);
            (*data).dragging = DragList_Notify(hwnd, DL_BEGINDRAG as UINT) as BOOL;
            if (*data).dragging != 0 {
                SetCapture(hwnd);
                SetTimer(
                    hwnd,
                    DRAGLIST_TIMERID as UINT_PTR,
                    DRAGLIST_SCROLLPERIOD as UINT,
                    None,
                );
            }
            /* note that we don't absorb this message to let the list box
             * do its thing (normally selecting an item) */
        }
        WM_KEYDOWN | WM_RBUTTONDOWN => {
            /* user cancelled drag by either right clicking or
             * by pressing the escape key */
            if (*data).dragging != 0
                && (uMsg == WM_RBUTTONDOWN as UINT
                    || wParam == VK_ESCAPE as WPARAM)
            {
                /* clean up and absorb message */
                DragList_EndDrag(hwnd, data);
                DragList_Notify(hwnd, DL_CANCELDRAG as UINT);
                return 0 as LRESULT;
            }
        }
        WM_MOUSEMOVE | WM_TIMER => {
            if (*data).dragging != 0 {
                let cursor: LRESULT = DragList_Notify(hwnd, DL_DRAGGING as UINT);
                /* optimisation so that we don't have to load the cursor
                 * all of the time whilst dragging */
                if (*data).last_dragging_response != cursor {
                    match (cursor) as u32 {
                        DL_STOPCURSOR => {
                            (*data).cursor = LoadCursorW(
                                core::ptr::null_mut(),
                                IDC_NO as LPCWSTR,
                            );
                            SetCursor((*data).cursor);
                        }
                        DL_COPYCURSOR => {
                            let reactos: bool = false;
                            if !reactos {
                                (*data).cursor = LoadCursorW(
                                    COMCTL32_hModule as HINSTANCE,
                                    IDC_COPY as LPCWSTR,
                                );
                            } else {
                                (*data).cursor = LoadCursorW(
                                    COMCTL32_hModule as HINSTANCE,
                                    MAKEINTRESOURCEW!(IDC_COPY) as LPCWSTR,
                                );
                            }
                            SetCursor((*data).cursor);
                        }
                        DL_MOVECURSOR => {
                            (*data).cursor = LoadCursorW(
                                core::ptr::null_mut(),
                                IDC_ARROW as LPCWSTR,
                            );
                            SetCursor((*data).cursor);
                        }
                        _ => {}
                    }
                    (*data).last_dragging_response = cursor;
                }
                /* don't pass this message on to List Box */
                return 0 as LRESULT;
            }
        }
        WM_LBUTTONUP => {
            if (*data).dragging != 0 {
                DragList_EndDrag(hwnd, data);
                DragList_Notify(hwnd, DL_DROPPED as UINT);
            }
        }
        WM_GETDLGCODE => {
            /* tell dialog boxes that we want to receive WM_KEYDOWN events
             * for keys like VK_ESCAPE */
            if (*data).dragging != 0 {
                return DLGC_WANTALLKEYS as LRESULT;
            }
        }
        WM_NCDESTROY => {
            RemoveWindowSubclass(
                hwnd,
                Some(
                    DragList_SubclassWindowProc,
                ),
                DRAGLIST_SUBCLASSID as UINT_PTR,
            );
            Free(data as LPVOID);
        }
        _ => {}
    }
    return DefSubclassProc(hwnd, uMsg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn MakeDragList(hwndLB: HWND) -> BOOL {
    let data: *mut DRAGLISTDATA = Alloc(core::mem::size_of::<DRAGLISTDATA>() as DWORD) as *mut DRAGLISTDATA;

    if uDragListMessage == 0 {
        uDragListMessage = RegisterWindowMessageW((DRAGLISTMSGSTRINGW).as_ptr());
    }

    return SetWindowSubclass(
        hwndLB,
        Some(DragList_SubclassWindowProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, UINT_PTR, DWORD_PTR) -> LRESULT),
        DRAGLIST_SUBCLASSID as UINT_PTR,
        data as DWORD_PTR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DrawInsert(hwndParent: HWND, hwndLB: HWND, nItem: INT) {
    let mut rcItem: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcListBox: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcDragIcon: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hdc: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut data: *mut DRAGLISTDATA = ::core::ptr::null_mut::<DRAGLISTDATA>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/draglist.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DrawInsert\0" as *const u8 as *const ::core::ffi::c_char,
            234 as ::core::ffi::c_int,
            b"(%p %p %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwndParent,
            hwndLB,
            nItem,
        );
    }

    if hDragArrow.is_null() {
        hDragArrow = LoadIconW(
            COMCTL32_hModule as HINSTANCE,
            IDI_DRAGARROW as LPCWSTR,
        );
    }

    if (LB_ERR as LRESULT)
        == SendMessageW(
            hwndLB,
            LB_GETITEMRECT as UINT,
            nItem as WPARAM,
            &raw mut rcItem as LPARAM,
        )
    {
        return;
    }

    if GetWindowRect(hwndLB, &raw mut rcListBox) == 0 {
        return;
    }

    /* convert item rect to parent co-ordinates */
    if MapWindowPoints(
        hwndLB,
        hwndParent,
        &raw mut rcItem as LPPOINT,
        2 as UINT,
    ) == 0
    {
        return;
    }

    /* convert list box rect to parent co-ordinates */
    if MapWindowPoints(
        HWND_DESKTOP as HWND,
        hwndParent,
        &raw mut rcListBox as LPPOINT,
        2 as UINT,
    ) == 0
    {
        return;
    }

    rcDragIcon.left = (rcListBox.left as INT - DRAGICON_HOTSPOT_X) as LONG;
    rcDragIcon.top = (rcItem.top as INT - DRAGICON_HOTSPOT_Y) as LONG;
    rcDragIcon.right = rcListBox.left;
    rcDragIcon.bottom = (rcDragIcon.top as INT + DRAGICON_HEIGHT) as LONG;

    if GetWindowSubclass(
        hwndLB,
        Some(
            DragList_SubclassWindowProc,
        ),
        DRAGLIST_SUBCLASSID as UINT_PTR,
        &raw mut data as *mut DWORD_PTR,
    ) == 0
    {
        return;
    }

    if nItem < 0 {
        SetRectEmpty(&raw mut rcDragIcon);
    }

    /* prevent flicker by only redrawing when necessary */
    if EqualRect(
        &raw mut rcDragIcon as LPCRECT,
        &raw mut (*data).last_drag_icon_rect as LPCRECT,
    ) == 0
    {
        /* get rid of any previous inserts drawn */
        RedrawWindow(
            hwndParent,
            &raw mut (*data).last_drag_icon_rect as LPCRECT,
            ::core::ptr::null_mut::<HRGN__>(),
            (RDW_INTERNALPAINT | RDW_ERASE | RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
        );

        (*data).last_drag_icon_rect = rcDragIcon;

        if nItem >= 0 {
            hdc = GetDC(hwndParent);

            DrawIcon(
                hdc,
                rcDragIcon.left as INT,
                rcDragIcon.top as INT,
                hDragArrow,
            );

            ReleaseDC(hwndParent, hdc);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn LBItemFromPt(mut hwndLB: HWND, mut pt: POINT, mut bAutoScroll: BOOL) -> INT {
    let mut rcClient: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut nIndex: INT;
    let mut dwScrollTime: DWORD;

    ScreenToClient(hwndLB, core::ptr::addr_of_mut!(pt));
    GetClientRect(hwndLB, core::ptr::addr_of_mut!(rcClient));
    nIndex = SendMessageW(
        hwndLB,
        LB_GETTOPINDEX as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;

    if PtInRect(core::ptr::addr_of_mut!(rcClient) as LPCRECT, pt) != 0 {
        /* point is inside -- get the item index */
        loop {
            if SendMessageW(
                hwndLB,
                LB_GETITEMRECT as UINT,
                nIndex as WPARAM,
                core::ptr::addr_of_mut!(rcClient) as LPARAM,
            ) == LB_ERR as LRESULT
            {
                return -(1 as INT);
            }

            if PtInRect(core::ptr::addr_of_mut!(rcClient) as LPCRECT, pt) != 0 {
                return nIndex;
            }

            nIndex += 1;
        }
    } else {
        /* point is outside */
        if bAutoScroll == 0 {
            return -(1 as INT);
        }

        if (pt.x > rcClient.right) || (pt.x < rcClient.left) {
            return -(1 as INT);
        }

        if pt.y < 0 as LONG {
            nIndex -= 1;
        } else {
            nIndex += 1;
        }

        dwScrollTime = GetTickCount();

        if dwScrollTime.wrapping_sub(dwLastScrollTime) < DRAGLIST_SCROLLPERIOD as DWORD {
            return -(1 as INT);
        }

        dwLastScrollTime = dwScrollTime;

        SendMessageW(
            hwndLB,
            LB_SETTOPINDEX as UINT,
            nIndex as WPARAM,
            0 as LPARAM,
        );
    }

    return -(1 as INT);
}
